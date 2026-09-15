// SPDX-License-Identifier: MIT

//! Opt-in logind sleep preparation. Only Online mounts are touched.

use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use cosmic::iced::{
    Subscription,
    futures::{FutureExt, SinkExt, StreamExt, channel::mpsc::Sender},
    stream,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use zbus::zvariant::OwnedFd;

use crate::config::Config;
use crate::model::{Connection, ConnectionId, ConnectionMode, Provider};
use crate::mounts::{MountEntry, parse_mountinfo_line};
use crate::process::{CommandRequest, CommandRunner, Executable, RuntimeCommandRunner};
use crate::providers::clean_unmount_request;
use crate::services::{
    FileUnitStore, StructuralUnitValidator, UnitFileState, UnitKind, UnitName, UnitStore,
};

#[derive(Debug, Clone)]
pub enum Event {
    Status(String),
    Wake(Vec<ConnectionId>),
}

#[derive(Default)]
struct OnlineGate {
    preparing: bool,
    cancellation: CancellationToken,
    active_operations: usize,
}
static ONLINE_GATE: LazyLock<Mutex<OnlineGate>> =
    LazyLock::new(|| Mutex::new(OnlineGate::default()));

/// Keep a token from before authentication: a sleep/wake cycle invalidates the
/// entire old operation, including a late VPN or systemctl completion.
pub fn online_operation_token() -> Result<CancellationToken, String> {
    let gate = ONLINE_GATE.lock().expect("online gate");
    if gate.preparing {
        Err("Online mounts are paused while preparing for sleep".into())
    } else {
        Ok(gate.cancellation.child_token())
    }
}

/// Registration keeps cleanup from overtaking an in-flight mount command.
pub struct OnlineOperation {
    token: CancellationToken,
}
impl OnlineOperation {
    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }
}
impl Drop for OnlineOperation {
    fn drop(&mut self) {
        ONLINE_GATE.lock().expect("online gate").active_operations -= 1;
    }
}
pub fn begin_online_operation() -> Result<OnlineOperation, String> {
    let mut gate = ONLINE_GATE.lock().expect("online gate");
    if gate.preparing {
        return Err("Online mounts are paused while preparing for sleep".into());
    }
    gate.active_operations += 1;
    Ok(OnlineOperation {
        token: gate.cancellation.child_token(),
    })
}
async fn wait_for_online_idle() {
    loop {
        if ONLINE_GATE.lock().expect("online gate").active_operations == 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn preparing() {
    let mut gate = ONLINE_GATE.lock().expect("online gate");
    gate.preparing = true;
    gate.cancellation.cancel();
}

fn awake() {
    let mut gate = ONLINE_GATE.lock().expect("online gate");
    gate.preparing = false;
    if gate.cancellation.is_cancelled() {
        gate.cancellation = CancellationToken::new();
    }
}

pub fn subscription() -> Subscription<Event> {
    Subscription::run(|| {
        stream::channel(16, |mut output| async move {
            // Subscription drop also closes the inhibitor FD. Do not let a removed
            // listener leave the in-process online gate permanently closed.
            struct ResetGate;
            impl Drop for ResetGate {
                fn drop(&mut self) {
                    awake();
                }
            }
            let _reset = ResetGate;
            loop {
                if let Err(error) = listen(&mut output).await
                    && output
                        .send(Event::Status(format!(
                            "Sleep cleanup unavailable: {error}. Retrying…"
                        )))
                        .await
                        .is_err()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        })
    })
}

async fn inhibit(proxy: &zbus::Proxy<'_>) -> Result<OwnedFd, String> {
    proxy
        .call(
            "Inhibit",
            &(
                String::from("sleep"),
                String::from("Cloud Mounter"),
                String::from("Unmount Online connections before sleep"),
                String::from("delay"),
            ),
        )
        .await
        .map_err(|error| error.to_string())
}

async fn listen(output: &mut Sender<Event>) -> Result<(), String> {
    let bus = zbus::Connection::system()
        .await
        .map_err(|error| error.to_string())?;
    let proxy = zbus::Proxy::new(
        &bus,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await
    .map_err(|error| error.to_string())?;
    let mut signals = proxy
        .receive_signal("PrepareForSleep")
        .await
        .map_err(|error| error.to_string())?;
    let mut inhibitor = Some(inhibit(&proxy).await?);
    let mut sleeping = proxy
        .get_property::<bool>("PreparingForSleep")
        .await
        .map_err(|error| error.to_string())?;
    if sleeping {
        // Too late to promise preparation for a sleep already in progress.
        preparing();
        inhibitor.take();
    }
    if !sleeping {
        awake();
    }
    let mut restore = Vec::new();
    let _ = output.try_send(Event::Status(if sleeping {
        "Sleep already in progress; cleanup will be armed after wake".into()
    } else {
        "Online mount cleanup before sleep is ready".into()
    }));
    while let Some(signal) = signals.next().await {
        let (going_to_sleep,): (bool,) = signal
            .body()
            .deserialize()
            .map_err(|error| error.to_string())?;
        if going_to_sleep {
            if sleeping {
                continue;
            }
            sleeping = true;
            preparing();
            let started = Instant::now();
            let _ = output.try_send(Event::Status(
                "Preparing for sleep: unmounting Online connections…".into(),
            ));
            // Re-read the host policy for each cycle; reserve time to close the FD.
            let limit = tokio::time::timeout(
                Duration::from_millis(250),
                proxy.get_property::<u64>("InhibitDelayMaxUSec"),
            )
            .await
            .map_err(|_| "could not read sleep delay deadline".to_string())?
            .map_err(|error| error.to_string())?;
            let budget = cleanup_budget(Duration::from_micros(limit));
            let config = Config::load_runtime().config;
            let deadline = started + budget;
            let report = if tokio::time::timeout_at(deadline, wait_for_online_idle())
                .await
                .is_ok()
            {
                cleanup(
                    &RuntimeCommandRunner::detect_current(),
                    &config.document.connections,
                    deadline,
                )
                .await
            } else {
                CleanupReport { active: Vec::new(), summary: "Sleep cleanup incomplete: an Online operation did not cancel before the sleep deadline".into() }
            };
            restore = if config.document.restore_after_wake {
                report.active
            } else {
                Vec::new()
            };
            inhibitor.take(); // Never hold sleep beyond the finite cleanup window.
            let _ = output.try_send(Event::Status(report.summary));
        } else {
            if !sleeping {
                continue;
            }
            sleeping = false;
            awake();
            inhibitor = Some(inhibit(&proxy).await?);
            let _ = output.try_send(Event::Wake(std::mem::take(&mut restore)));
        }
    }
    Err("logind signal stream ended".into())
}

fn cleanup_budget(host_limit: Duration) -> Duration {
    host_limit
        .saturating_sub(Duration::from_millis(500))
        .min(Duration::from_secs(30))
}

#[derive(Debug)]
pub struct CleanupReport {
    pub active: Vec<ConnectionId>,
    pub summary: String,
}

pub fn online_connections(connections: &[Connection]) -> impl Iterator<Item = &Connection> {
    connections
        .iter()
        .filter(|connection| matches!(connection.mode, ConnectionMode::OnlineMount(_)))
}

async fn host_mounts(runner: &impl CommandRunner) -> Result<Vec<MountEntry>, String> {
    let request = CommandRequest::new(Executable::Cat)
        .arg("/proc/self/mountinfo")
        .map_err(|error| error.to_string())?;
    let output = runner
        .run(
            request.with_output_limit(4 * 1024 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| error.to_string())?;
    if output.stdout.truncated {
        return Err("host mount table is truncated".into());
    }
    output
        .stdout
        .text
        .lines()
        .map(|line| parse_mountinfo_line(line).map_err(|error| error.to_string()))
        .collect()
}

async fn systemctl(runner: &impl CommandRunner, args: &[String]) -> Result<String, String> {
    let mut request = CommandRequest::new(Executable::Systemctl)
        .arg("--user")
        .map_err(|error| error.to_string())?;
    for arg in args {
        request = request.arg(arg).map_err(|error| error.to_string())?;
    }
    runner
        .run(request, CancellationToken::new())
        .await
        .map(|output| output.stdout.text)
        .map_err(|error| error.to_string())
}

async fn active(runner: &impl CommandRunner, unit: &str) -> Result<bool, String> {
    let state = systemctl(
        runner,
        &[
            "show".into(),
            "--property=ActiveState".into(),
            "--value".into(),
            unit.into(),
        ],
    )
    .await?;
    match state.trim() {
        "inactive" | "failed" => Ok(false),
        "active" | "activating" | "deactivating" | "reloading" => Ok(true),
        _ => Err(format!("unknown service state: {}", state.trim())),
    }
}

pub async fn cleanup(
    runner: &impl CommandRunner,
    connections: &[Connection],
    deadline: Instant,
) -> CleanupReport {
    let mut jobs = Vec::new();
    for connection in online_connections(connections) {
        jobs.push(
            async move {
                let mut was_active = false;
                let result = tokio::time::timeout_at(
                    deadline,
                    clean_one(runner, connection, deadline, &mut was_active),
                )
                .await;
                let result = result.unwrap_or_else(|_| {
                    Err("sleep cleanup deadline reached; inspect mount/service after wake".into())
                });
                (connection, was_active, result)
            }
            .boxed(),
        );
    }
    let mut tasks = cosmic::iced::futures::stream::iter(jobs).buffer_unordered(4);
    let mut restored = Vec::new();
    let mut failures = Vec::new();
    let mut cleaned = 0;
    while let Some((connection, was_active, result)) = tasks.next().await {
        match result {
            Ok(true) => {
                cleaned += 1;
                if was_active {
                    restored.push(connection.id);
                }
            }
            Ok(false) => {}
            Err(error) => failures.push(format!("{}: {error}", connection.name)),
        }
    }
    let summary = if failures.is_empty() {
        format!(
            "Sleep cleanup: {cleaned} Online connection(s) unmounted. Offline mirrors were untouched. Cached data is preserved; remote upload completion is not guaranteed."
        )
    } else {
        format!(
            "Sleep cleanup incomplete ({cleaned} unmounted): {}. Caches preserved; no forced or lazy unmount was used.",
            failures.join("; ")
        )
    };
    CleanupReport {
        active: restored,
        summary,
    }
}

async fn clean_one(
    runner: &impl CommandRunner,
    connection: &Connection,
    deadline: Instant,
    was_active: &mut bool,
) -> Result<bool, String> {
    let unit = UnitName::new(connection.id, UnitKind::Service);
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    if store.state(&unit).map_err(|error| error.to_string())?
        != UnitFileState::Managed(connection.id)
    {
        return Err("service ownership could not be verified; left untouched".into());
    }
    let root =
        PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").ok_or("runtime directory unavailable")?)
            .join("systemd/user");
    prepare_stop_policy(&root, &unit)?;
    // Reload even if the file already exists: a previous cycle may have
    // exhausted its deadline after writing it but before completing reload.
    systemctl(runner, &["daemon-reload".into()]).await?;
    clean_owned(runner, connection, &unit.file_name(), deadline, was_active).await
}

async fn clean_owned(
    runner: &impl CommandRunner,
    connection: &Connection,
    unit: &str,
    deadline: Instant,
    was_active: &mut bool,
) -> Result<bool, String> {
    let mounts = host_mounts(runner).await?;
    let mounted = mounts
        .iter()
        .find(|mount| mount.target == connection.local_path);
    if let Some(mount) = mounted {
        let expected = if connection.provider == Provider::OneDrive {
            "fuse.onedriver"
        } else {
            "fuse.rclone"
        };
        if mount.filesystem != expected {
            return Err("unexpected filesystem at mountpoint; left untouched".into());
        }
    }
    *was_active = mounted.is_some() || active(runner, unit).await?;
    if !*was_active {
        return Ok(false);
    }
    if mounted.is_some() && connection.provider != Provider::OneDrive {
        // Do not infer an empty queue from missing/unrecognized health fields.
        let socket = PathBuf::from(
            std::env::var_os("XDG_RUNTIME_DIR").ok_or("runtime directory unavailable")?,
        )
        .join("cosmic-ext-applet-mounter")
        .join(format!("rclone-{}.sock", connection.id));
        let request = CommandRequest::new(Executable::Rclone)
            .arg("rc")
            .and_then(|r| r.arg("--unix-socket"))
            .and_then(|r| r.arg(socket))
            .and_then(|r| r.arg("vfs/stats"))
            .map_err(|error| error.to_string())?;
        let health = runner
            .run(request, CancellationToken::new())
            .await
            .map_err(|error| error.to_string())?;
        if !uploads_idle(&health.stdout.text)? {
            return Err("pending uploads; mount left active".into());
        }
    }
    // Stop jobs outlive their systemctl client. Verify the effective unit
    // settings before enqueueing a stop; other user drop-ins may override ours.
    if deadline.saturating_duration_since(Instant::now()) < Duration::from_millis(2500) {
        return Err("insufficient sleep delay for a clean stop".into());
    }
    let policy = systemctl(
        runner,
        &[
            "show".into(),
            "--property=SendSIGKILL,TimeoutStopUSec,TimeoutStopFailureMode,KillSignal".into(),
            unit.into(),
        ],
    )
    .await?;
    if !stop_policy_applied(&policy) {
        return Err("nonforcing sleep stop policy was not applied; mount left active".into());
    }
    systemctl(runner, &["stop".into(), "--no-block".into(), unit.into()]).await?;
    loop {
        let still_active = active(runner, unit).await?;
        let mounts = host_mounts(runner).await?;
        let still_mounted = mounts
            .iter()
            .any(|mount| mount.target == connection.local_path);
        if !still_active && !still_mounted {
            return Ok(true);
        }
        if !still_active && still_mounted {
            runner
                .run(
                    clean_unmount_request(&connection.local_path)
                        .map_err(|error| error.to_string())?,
                    CancellationToken::new(),
                )
                .await
                .map_err(|error| error.to_string())?;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

const STOP_POLICY: &str = "# Cloud Mounter Online sleep cleanup\n[Service]\nTimeoutStopSec=2s\nTimeoutStopFailureMode=terminate\nKillSignal=SIGTERM\nSendSIGKILL=no\n";

fn prepare_stop_policy(root: &std::path::Path, unit: &UnitName) -> Result<bool, String> {
    use std::io::Write;
    let directory = root.join(format!("{}.d", unit.file_name()));
    let path = directory.join("90-cosmic-mounter-sleep.conf");
    match std::fs::read_to_string(&path) {
        Ok(existing) if existing == STOP_POLICY => return Ok(false),
        Ok(_) => return Err("sleep stop policy file was modified; left untouched".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let temporary = directory.join(format!(".sleep-{}", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(STOP_POLICY.as_bytes())?;
        std::fs::rename(&temporary, &path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
        .map(|()| true)
        .map_err(|error: std::io::Error| error.to_string())
}

fn stop_policy_applied(text: &str) -> bool {
    [
        "SendSIGKILL=no",
        "TimeoutStopUSec=2s",
        "TimeoutStopFailureMode=terminate",
        "KillSignal=15",
    ]
    .iter()
    .all(|expected| text.lines().any(|line| line == *expected))
}

fn uploads_idle(text: &str) -> Result<bool, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let disk = value
        .get("diskCache")
        .ok_or("upload status unavailable; mount left active")?;
    let queued = disk
        .get("uploadsQueued")
        .and_then(serde_json::Value::as_u64)
        .ok_or("upload queue unknown; mount left active")?;
    let active = disk
        .get("uploadsInProgress")
        .and_then(serde_json::Value::as_u64)
        .ok_or("active uploads unknown; mount left active")?;
    Ok(queued == 0 && active == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{OfflineMirrorConfig, OnlineMountConfig, TuningProfile};
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};

    fn connection(online: bool) -> Connection {
        Connection {
            id: ConnectionId::new(),
            name: "test".into(),
            provider: Provider::OneDrive,
            mode: if online {
                ConnectionMode::OnlineMount(OnlineMountConfig::default())
            } else {
                ConnectionMode::OfflineMirror(OfflineMirrorConfig::default())
            },
            remote_reference: "test".into(),
            remote_subpath: None,
            local_path: "/tmp/test-mount".into(),
            enabled: false,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::default(),
        }
    }
    fn output(text: &str) -> Result<CommandOutput, crate::process::CommandError> {
        Ok(CommandOutput {
            command: "fake".into(),
            stdout: CapturedOutput {
                text: text.into(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
            duration: Duration::ZERO,
        })
    }

    #[test]
    fn selects_only_online_even_if_saved_disabled() {
        let online = connection(true);
        let connections = vec![connection(false), online.clone(), connection(false)];
        assert_eq!(
            online_connections(&connections)
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![online.id]
        );
    }

    #[tokio::test]
    async fn mirror_only_cleanup_issues_no_commands() {
        let runner = FakeCommandRunner::default();
        let report = cleanup(
            &runner,
            &[connection(false)],
            Instant::now() + Duration::from_secs(1),
        )
        .await;
        assert!(runner.requests().is_empty());
        assert!(report.active.is_empty());
    }

    #[test]
    fn runtime_stop_policy_is_owned_atomic_and_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let unit = UnitName::new(ConnectionId::new(), UnitKind::Service);
        assert!(prepare_stop_policy(temp.path(), &unit).unwrap());
        assert!(!prepare_stop_policy(temp.path(), &unit).unwrap());
        let path = temp
            .path()
            .join(format!("{}.d", unit.file_name()))
            .join("90-cosmic-mounter-sleep.conf");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("TimeoutStopSec=2s"));
        assert!(text.contains("SendSIGKILL=no"));
        std::fs::write(&path, "user changes").unwrap();
        assert!(prepare_stop_policy(temp.path(), &unit).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "user changes");
    }

    #[test]
    fn stop_policy_overrides_cannot_enable_forced_killing() {
        let safe =
            "SendSIGKILL=no\nTimeoutStopUSec=2s\nTimeoutStopFailureMode=terminate\nKillSignal=15\n";
        assert!(stop_policy_applied(safe));
        assert!(!stop_policy_applied(
            &safe.replace("SendSIGKILL=no", "SendSIGKILL=yes")
        ));
        assert!(!stop_policy_applied(
            &safe.replace("KillSignal=15", "KillSignal=9")
        ));
        assert!(!stop_policy_applied(
            &safe.replace("TimeoutStopUSec=2s", "TimeoutStopUSec=1min 30s")
        ));
    }

    #[test]
    fn cleanup_budget_always_leaves_release_margin() {
        assert_eq!(
            cleanup_budget(Duration::from_secs(5)),
            Duration::from_millis(4500)
        );
        assert_eq!(cleanup_budget(Duration::from_millis(100)), Duration::ZERO);
        assert_eq!(
            cleanup_budget(Duration::from_secs(120)),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn missing_or_pending_uploads_are_never_treated_as_idle() {
        assert!(uploads_idle("{}").is_err());
        assert!(uploads_idle(r#"{"diskCache":{}}"#).is_err());
        assert!(
            !uploads_idle(r#"{"diskCache":{"uploadsQueued":1,"uploadsInProgress":0}}"#).unwrap()
        );
        assert!(
            !uploads_idle(r#"{"diskCache":{"uploadsQueued":0,"uploadsInProgress":1}}"#).unwrap()
        );
        assert!(
            uploads_idle(r#"{"diskCache":{"uploadsQueued":0,"uploadsInProgress":0}}"#).unwrap()
        );
    }

    #[test]
    fn sleep_invalidates_old_operations_even_after_wake() {
        awake();
        let old = online_operation_token().unwrap();
        preparing();
        preparing(); // repeated preparation cannot reopen the gate
        assert!(old.is_cancelled());
        assert!(online_operation_token().is_err());
        awake();
        assert!(old.is_cancelled());
        assert!(!online_operation_token().unwrap().is_cancelled());
    }

    #[tokio::test]
    async fn stop_is_nonblocking_and_nonforcing_before_mount_disappearance() {
        let runner = FakeCommandRunner::default();
        runner.push(output(
            "1 2 0:1 / /tmp/test-mount rw - fuse.onedriver onedriver rw\n",
        ));
        runner.push(output(
            "SendSIGKILL=no\nTimeoutStopUSec=2s\nTimeoutStopFailureMode=terminate\nKillSignal=15\n",
        )); // effective stop safeguards
        runner.push(output("")); // enqueue stop
        runner.push(output("inactive"));
        runner.push(output("")); // mount disappeared
        let mut was_active = false;
        assert!(
            clean_owned(
                &runner,
                &connection(true),
                "cosmic-mounter-test.service",
                Instant::now() + Duration::from_secs(4),
                &mut was_active
            )
            .await
            .unwrap()
        );
        assert!(was_active);
        let commands = runner
            .requests()
            .iter()
            .map(CommandRequest::sanitized_command)
            .collect::<Vec<_>>();
        assert!(
            commands[1].contains(
                "--property=SendSIGKILL,TimeoutStopUSec,TimeoutStopFailureMode,KillSignal"
            )
        );
        assert!(commands[2].contains("stop --no-block"));
        assert!(!commands.iter().any(|command| command.contains("-uz")));
    }

    #[tokio::test]
    async fn foreign_filesystem_is_left_untouched() {
        let runner = FakeCommandRunner::default();
        runner.push(output("1 2 0:1 / /tmp/test-mount rw - ext4 /dev/fake rw\n"));
        let mut was_active = false;
        assert!(
            clean_owned(
                &runner,
                &connection(true),
                "cosmic-mounter-test.service",
                Instant::now() + Duration::from_secs(4),
                &mut was_active
            )
            .await
            .is_err()
        );
        assert_eq!(runner.requests().len(), 1);
    }

    #[tokio::test]
    async fn short_deadline_never_enqueues_a_stop() {
        let runner = FakeCommandRunner::default();
        runner.push(output(
            "1 2 0:1 / /tmp/test-mount rw - fuse.onedriver onedriver rw\n",
        ));
        let mut was_active = false;
        let result = clean_owned(
            &runner,
            &connection(true),
            "cosmic-mounter-test.service",
            Instant::now() + Duration::from_millis(100),
            &mut was_active,
        )
        .await;
        assert!(result.unwrap_err().contains("insufficient"));
        assert_eq!(runner.requests().len(), 1);
    }
}
