// SPDX-License-Identifier: MIT

//! Bounded, directory-only warm-up for Online OneDrive, Box, SMB, and SFTP mounts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::model::{Connection, ConnectionId, Provider};
use crate::process::{
    CommandError, CommandRequest, CommandRunner, Executable, RuntimeCommandRunner,
};
use crate::sleep::begin_online_operation;

const MOUNT_READY_TIMEOUT: Duration = Duration::from_secs(15);
const MOUNT_PROBE_INTERVAL: Duration = Duration::from_millis(250);
const MOUNT_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const ROOT_READY_TIMEOUT: Duration = Duration::from_secs(30);
const ROOT_READY_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const CANCEL_WAIT: Duration = Duration::from_secs(2);
// FUSE operations can remain in kernel wait after `find` receives SIGKILL.
// Allow the mount's network timeout and process reap to finish before detaching.
const UNMOUNT_CANCEL_WAIT: Duration = Duration::from_secs(60);
const CANCELLED: &str = "Directory preload canceled";
const TIMED_OUT: &str = "Directory preload deadline reached";
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryPreloadJob {
    pub connection_id: ConnectionId,
    pub generation: u64,
    mountpoint: PathBuf,
    timeout: Duration,
    maximum_depth: Option<u8>,
    wait_for_root: bool,
    excluded_paths: Vec<PathBuf>,
}

pub use crate::preload_report::PreloadOutcome as DirectoryPreloadOutcome;
use crate::preload_report::PreloadReport;

#[derive(Debug)]
struct ActivePreload {
    generation: u64,
    cancellation: CancellationToken,
    done: AtomicBool,
    completion: Notify,
}

static ACTIVE_PRELOADS: LazyLock<Mutex<HashMap<ConnectionId, Arc<ActivePreload>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn start(
    connection_id: ConnectionId,
    mountpoint: PathBuf,
    timeout: Duration,
    maximum_depth: Option<u8>,
    wait_for_root: bool,
) -> Result<DirectoryPreloadJob, String> {
    if timeout.is_zero() {
        return Err("Directory preload time must be greater than zero".into());
    }
    let mut active = ACTIVE_PRELOADS.lock().expect("directory preload registry");
    if let Some(previous) = active.remove(&connection_id) {
        previous.cancellation.cancel();
    }
    let generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
    active.insert(
        connection_id,
        Arc::new(ActivePreload {
            generation,
            cancellation: CancellationToken::new(),
            done: AtomicBool::new(false),
            completion: Notify::new(),
        }),
    );
    drop(active);
    Ok(DirectoryPreloadJob {
        connection_id,
        generation,
        mountpoint,
        timeout,
        maximum_depth,
        wait_for_root,
        excluded_paths: Vec::new(),
    })
}

/// Build a preload using the same lifecycle, with server-system directories
/// pruned for SFTP. Exclusions refer to remote absolute paths, not local names.
pub fn start_for_connection(
    connection: &Connection,
    timeout: Duration,
    maximum_depth: Option<u8>,
    wait_for_root: bool,
) -> Result<DirectoryPreloadJob, String> {
    let mut job = start(
        connection.id,
        connection.local_path.clone(),
        timeout,
        maximum_depth,
        wait_for_root,
    )?;
    job.excluded_paths = sftp_excluded_paths(connection);
    Ok(job)
}

fn sftp_excluded_paths(connection: &Connection) -> Vec<PathBuf> {
    if connection.provider != Provider::Sftp {
        return Vec::new();
    }
    let Some(remote) = connection.remote_subpath.as_deref() else {
        return Vec::new();
    };
    let remote = Path::new(remote);
    if !remote.is_absolute() {
        return Vec::new();
    }
    let mut excluded = Vec::new();
    for system in ["/proc", "/sys", "/dev"] {
        let system = Path::new(system);
        if remote.starts_with(system) {
            return vec![connection.local_path.clone()];
        }
        if let Ok(relative) = system.strip_prefix(remote) {
            excluded.push(connection.local_path.join(relative));
        }
    }
    excluded
}

pub async fn monitor(job: DirectoryPreloadJob) -> Result<DirectoryPreloadOutcome, String> {
    let runner = RuntimeCommandRunner::detect_current();
    monitor_with(&runner, job).await
}

pub async fn cancel(connection_id: ConnectionId) -> bool {
    cancel_with_timeout(connection_id, CANCEL_WAIT)
        .await
        .is_ok_and(|active| active)
}

/// A clean unmount must not stop the service while a preload child can still
/// hold an open FUSE handle. Keep the registration if cleanup takes too long.
pub async fn cancel_before_unmount(connection_id: ConnectionId) -> Result<(), String> {
    cancel_with_timeout(connection_id, UNMOUNT_CANCEL_WAIT)
        .await
        .map(|_| ())
}

async fn cancel_with_timeout(connection_id: ConnectionId, wait: Duration) -> Result<bool, String> {
    let control = ACTIVE_PRELOADS
        .lock()
        .expect("directory preload registry")
        .get(&connection_id)
        .cloned();
    let Some(control) = control else {
        return Ok(false);
    };
    control.cancellation.cancel();
    let completion = control.completion.notified();
    tokio::pin!(completion);
    completion.as_mut().enable();
    if !control.done.load(Ordering::Acquire)
        && tokio::time::timeout(wait, completion).await.is_err()
        && !control.done.load(Ordering::Acquire)
    {
        return Err("Directory preload is still stopping; the mount service was left running. Retry unmount after it finishes".into());
    }
    finish(connection_id, &control);
    Ok(true)
}

async fn monitor_with(
    runner: &dyn CommandRunner,
    job: DirectoryPreloadJob,
) -> Result<DirectoryPreloadOutcome, String> {
    let Some(control) = current_control(job.connection_id, job.generation) else {
        return Ok(DirectoryPreloadOutcome::Cancelled);
    };
    let operation = match begin_online_operation() {
        Ok(operation) => operation,
        Err(_) => {
            finish(job.connection_id, &control);
            return Ok(DirectoryPreloadOutcome::Cancelled);
        }
    };
    let started = Instant::now();
    let deadline = started + job.timeout;
    let report = || PreloadReport {
        duration_seconds: started.elapsed().as_secs_f64(),
        limit_seconds: job.timeout.as_secs_f64(),
        maximum_depth: job.maximum_depth,
        excluded_paths: job.excluded_paths.len(),
    };
    let result = async {
        if job.excluded_paths.contains(&job.mountpoint) {
            return Ok(DirectoryPreloadOutcome::Skipped);
        }
        if let Err(error) =
            wait_for_mount(runner, &job.mountpoint, deadline, &control, &operation).await
        {
            return if error == CANCELLED {
                Ok(DirectoryPreloadOutcome::Cancelled)
            } else if error == TIMED_OUT {
                Ok(DirectoryPreloadOutcome::TimedOut(report()))
            } else {
                Err(error)
            };
        }
        if job.wait_for_root
            && let Err(error) =
                wait_for_root_entries(runner, &job.mountpoint, deadline, &control, &operation).await
        {
            return if error == TIMED_OUT {
                Ok(DirectoryPreloadOutcome::TimedOut(report()))
            } else if error == CANCELLED {
                Ok(DirectoryPreloadOutcome::Cancelled)
            } else {
                Err(error)
            };
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(DirectoryPreloadOutcome::TimedOut(report()));
        }
        let request = traversal_request_excluding(
            &job.mountpoint,
            remaining,
            job.maximum_depth,
            &job.excluded_paths,
        )?;
        let result = run_preload_command(runner, request, &control, &operation).await;
        if control.cancellation.is_cancelled() {
            return Ok(DirectoryPreloadOutcome::Cancelled);
        }
        match result {
            Ok(_) => Ok(DirectoryPreloadOutcome::Completed(report())),
            Err(CommandError::Timeout { .. }) => Ok(DirectoryPreloadOutcome::TimedOut(report())),
            Err(CommandError::Cancelled { .. }) => Ok(DirectoryPreloadOutcome::Cancelled),
            Err(error) => Err(error.to_string()),
        }
    }
    .await;
    finish(job.connection_id, &control);
    result
}

/// Keep the runner future alive until its child has been killed and reaped.
/// Dropping it from a select branch can leave `find` holding the mount busy.
async fn run_preload_command(
    runner: &dyn CommandRunner,
    request: CommandRequest,
    control: &ActivePreload,
    operation: &crate::sleep::OnlineOperation,
) -> Result<crate::process::CommandOutput, CommandError> {
    let command = request.sanitized_command();
    let child_token = control.cancellation.child_token();
    let run = runner.run(request, child_token.clone());
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => result,
        () = operation.cancelled() => {
            child_token.cancel();
            let _ = run.await;
            Err(CommandError::Cancelled { command })
        }
    }
}

async fn wait_for_root_entries(
    runner: &dyn CommandRunner,
    mountpoint: &Path,
    overall_deadline: Instant,
    control: &ActivePreload,
    operation: &crate::sleep::OnlineOperation,
) -> Result<(), String> {
    let root_deadline = Instant::now() + ROOT_READY_TIMEOUT;
    let deadline = root_deadline.min(overall_deadline);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return if deadline == overall_deadline {
                Err(TIMED_OUT.into())
            } else {
                Ok(())
            };
        }
        let probe = CommandRequest::new(Executable::Find)
            .arg("-P")
            .map_err(|e| e.to_string())?
            .arg(mountpoint.as_os_str())
            .map_err(|e| e.to_string())?
            .arg("-mindepth")
            .map_err(|e| e.to_string())?
            .arg("1")
            .map_err(|e| e.to_string())?
            .arg("-maxdepth")
            .map_err(|e| e.to_string())?
            .arg("1")
            .map_err(|e| e.to_string())?
            .arg("-print")
            .map_err(|e| e.to_string())?
            .arg("-quit")
            .map_err(|e| e.to_string())?
            .with_timeout(ROOT_READY_PROBE_TIMEOUT.min(remaining))
            .with_output_limit(1024);
        let result = run_preload_command(runner, probe, control, operation).await;
        if control.cancellation.is_cancelled()
            || matches!(result, Err(CommandError::Cancelled { .. }))
        {
            return Err(CANCELLED.into());
        }
        if result.is_ok_and(|output| !output.stdout.text.trim().is_empty()) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return if deadline == overall_deadline {
                Err(TIMED_OUT.into())
            } else {
                // An empty remote is valid. Continue with the bounded traversal.
                Ok(())
            };
        }
        tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
            () = operation.cancelled() => return Err(CANCELLED.into()),
            () = tokio::time::sleep(MOUNT_PROBE_INTERVAL) => {}
        }
    }
}

async fn wait_for_mount(
    runner: &dyn CommandRunner,
    mountpoint: &Path,
    overall_deadline: Instant,
    control: &ActivePreload,
    operation: &crate::sleep::OnlineOperation,
) -> Result<(), String> {
    let mount_deadline = Instant::now() + MOUNT_READY_TIMEOUT;
    let deadline = mount_deadline.min(overall_deadline);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return if deadline == overall_deadline {
                Err(TIMED_OUT.into())
            } else {
                Err("Online mount did not become ready within 15 seconds".into())
            };
        }
        let probe = CommandRequest::new(Executable::Mountpoint)
            .arg("--quiet")
            .map_err(|error| error.to_string())?
            .arg(mountpoint.as_os_str())
            .map_err(|error| error.to_string())?
            .with_timeout(MOUNT_PROBE_TIMEOUT.min(remaining));
        let probe_result = run_preload_command(runner, probe, control, operation).await;
        if control.cancellation.is_cancelled()
            || matches!(probe_result, Err(CommandError::Cancelled { .. }))
        {
            return Err(CANCELLED.into());
        }
        if probe_result.is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return if deadline == overall_deadline {
                Err(TIMED_OUT.into())
            } else {
                Err("Online mount did not become ready within 15 seconds".into())
            };
        }
        tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
            () = operation.cancelled() => return Err(CANCELLED.into()),
            () = tokio::time::sleep(MOUNT_PROBE_INTERVAL) => {}
        }
    }
}

fn traversal_request_excluding(
    mountpoint: &Path,
    timeout: Duration,
    maximum_depth: Option<u8>,
    excluded_paths: &[PathBuf],
) -> Result<CommandRequest, String> {
    let mut request = CommandRequest::new(Executable::Find)
        .arg("-P")
        .map_err(|error| error.to_string())?
        .arg(mountpoint.as_os_str())
        .map_err(|error| error.to_string())?
        .arg("-xdev")
        .map_err(|error| error.to_string())?;
    if let Some(depth) = maximum_depth {
        request = request
            .arg("-maxdepth")
            .map_err(|error| error.to_string())?
            .arg(depth.to_string())
            .map_err(|error| error.to_string())?;
    }
    if !excluded_paths.is_empty() {
        request = request.arg("(").map_err(|e| e.to_string())?;
        for (index, path) in excluded_paths.iter().enumerate() {
            if index > 0 {
                request = request.arg("-o").map_err(|e| e.to_string())?;
            }
            // find -path interprets glob metacharacters even without a shell.
            let pattern = path
                .to_string_lossy()
                .chars()
                .flat_map(|c| {
                    if matches!(c, '\\' | '*' | '?' | '[' | ']') {
                        vec!['\\', c]
                    } else {
                        vec![c]
                    }
                })
                .collect::<String>();
            request = request
                .arg("-path")
                .and_then(|r| r.arg(pattern))
                .map_err(|e| e.to_string())?;
        }
        request = request
            .arg(")")
            .and_then(|r| r.arg("-prune"))
            .and_then(|r| r.arg("-o"))
            .map_err(|e| e.to_string())?;
    }
    request
        .arg("-type")
        .map_err(|error| error.to_string())?
        .arg("d")
        .map_err(|error| error.to_string())?
        .arg("-printf")
        .map_err(|error| error.to_string())?
        .arg("")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(timeout).with_output_limit(1024))
}

fn current_control(connection_id: ConnectionId, generation: u64) -> Option<Arc<ActivePreload>> {
    ACTIVE_PRELOADS
        .lock()
        .expect("directory preload registry")
        .get(&connection_id)
        .filter(|control| control.generation == generation)
        .cloned()
}

fn finish(connection_id: ConnectionId, control: &Arc<ActivePreload>) {
    control.done.store(true, Ordering::Release);
    control.completion.notify_waiters();
    let mut active = ACTIVE_PRELOADS.lock().expect("directory preload registry");
    if active
        .get(&connection_id)
        .is_some_and(|current| Arc::ptr_eq(current, control))
    {
        active.remove(&connection_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};
    use uuid::Uuid;

    fn id(value: &str) -> ConnectionId {
        ConnectionId::from_uuid(Uuid::parse_str(value).expect("uuid"))
    }

    fn output() -> CommandOutput {
        CommandOutput {
            command: "fake".into(),
            stdout: CapturedOutput {
                text: String::new(),
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
        }
    }

    fn sftp_connection(mountpoint: PathBuf, remote: &str) -> Connection {
        Connection {
            id: ConnectionId::new(),
            name: "SFTP".into(),
            provider: Provider::Sftp,
            mode: crate::model::ConnectionMode::OnlineMount(Default::default()),
            remote_reference: "sftp".into(),
            remote_subpath: Some(remote.into()),
            local_path: mountpoint,
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: Default::default(),
            smb_preload_override: None,
            sftp_preload_override: None,
        }
    }

    #[tokio::test]
    async fn sftp_preload_prunes_system_trees_with_literal_mountpoint_and_depth() {
        use crate::process::SystemCommandRunner;
        let temp = tempfile::tempdir().unwrap();
        let mount = temp.path().join("server [1]*?");
        for dir in [
            "proc/hidden",
            "sys/hidden",
            "dev/hidden",
            "home/user/deeper",
            "projects/sys/ordinary",
        ] {
            std::fs::create_dir_all(mount.join(dir)).unwrap();
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(mount.join("proc"), mount.join("link")).unwrap();
        let connection = sftp_connection(mount.clone(), "/");
        let excluded = sftp_excluded_paths(&connection);
        assert_eq!(
            excluded,
            vec![mount.join("proc"), mount.join("sys"), mount.join("dev")]
        );
        let mut request =
            traversal_request_excluding(&mount, Duration::from_secs(5), Some(2), &excluded)
                .unwrap();
        // Inspect directory output using the exact production traversal expression.
        request.args.pop();
        request = request.arg("%p|").unwrap();
        let output = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .unwrap();
        let lines: Vec<_> = output.stdout.text.split('|').map(str::trim).collect();
        assert!(lines.contains(&mount.join("home/user").to_str().unwrap()));
        assert!(lines.contains(&mount.join("projects/sys").to_str().unwrap()));
        for path in ["proc", "sys", "dev", "home/user/deeper", "link"] {
            assert!(
                !lines
                    .iter()
                    .any(|line| Path::new(line).starts_with(mount.join(path)))
            );
        }
    }

    #[tokio::test]
    async fn sftp_preload_skips_mounts_inside_system_trees_before_any_probe() {
        for remote in ["/proc", "/proc/123", "/sys/", "/dev/pts"] {
            let connection = sftp_connection(PathBuf::from("/tmp/server"), remote);
            let job =
                start_for_connection(&connection, Duration::from_secs(30), Some(2), true).unwrap();
            let runner = FakeCommandRunner::default();
            assert!(matches!(
                monitor_with(&runner, job).await.unwrap(),
                DirectoryPreloadOutcome::Skipped
            ));
            assert!(runner.requests().is_empty());
        }
        for remote in ["/projects", "/proc-projects", "projects/sys"] {
            let connection = sftp_connection(PathBuf::from("/tmp/server"), remote);
            assert!(sftp_excluded_paths(&connection).is_empty());
        }
        let mut connection = sftp_connection(PathBuf::from("/tmp/server"), "/");
        connection.provider = Provider::Box;
        assert!(sftp_excluded_paths(&connection).is_empty());
    }

    #[tokio::test]
    async fn traversal_is_directory_only_and_does_not_follow_links() {
        let runner =
            FakeCommandRunner::default().with_resolved([Executable::Mountpoint, Executable::Find]);
        runner.push(Ok(output()));
        runner.push(Ok(output()));
        let connection_id = id("5a3f5d45-e867-47e7-943f-66cf60e777ad");
        let job = start(
            connection_id,
            PathBuf::from("/home/user/OneDrive"),
            Duration::from_secs(60),
            None,
            false,
        )
        .expect("start");
        let generation = job.generation;
        assert!(matches!(
            monitor_with(&runner, job).await.expect("monitor"),
            DirectoryPreloadOutcome::Completed(_)
        ));

        let requests = runner.requests();
        assert_eq!(
            requests[0].sanitized_command(),
            "mountpoint --quiet /home/user/OneDrive"
        );
        assert_eq!(
            requests[1].sanitized_command(),
            "find -P /home/user/OneDrive -xdev -type d -printf "
        );
        assert!(current_control(connection_id, generation).is_none());
    }

    #[tokio::test]
    async fn cancellation_finishes_registered_preload() {
        let connection_id = id("6a3f5d45-e867-47e7-943f-66cf60e777ad");
        let job = start(
            connection_id,
            PathBuf::from("/home/user/OneDrive"),
            Duration::from_secs(60),
            None,
            false,
        )
        .expect("start");
        let generation = job.generation;
        let monitor = tokio::spawn(async move {
            let runner = FakeCommandRunner::default()
                .with_resolved([Executable::Mountpoint, Executable::Find]);
            monitor_with(&runner, job).await
        });
        assert!(cancel(connection_id).await);
        assert_eq!(
            monitor.await.expect("join").expect("monitor"),
            DirectoryPreloadOutcome::Cancelled
        );
        assert!(current_control(connection_id, generation).is_none());
    }

    #[tokio::test]
    async fn unmount_cancellation_waits_for_traversal_child_to_exit() {
        use crate::process::CommandFuture;

        struct DelayedRunner {
            entered: Arc<Notify>,
            exited: Arc<AtomicBool>,
        }
        impl CommandRunner for DelayedRunner {
            fn resolve(&self, executable: Executable) -> Option<PathBuf> {
                Some(PathBuf::from(executable.display_name()))
            }
            fn run<'a>(
                &'a self,
                request: CommandRequest,
                cancellation: CancellationToken,
            ) -> CommandFuture<'a> {
                Box::pin(async move {
                    if request.executable == Executable::Mountpoint {
                        return Ok(output());
                    }
                    self.entered.notify_one();
                    cancellation.cancelled().await;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    self.exited.store(true, Ordering::Release);
                    Err(CommandError::Cancelled {
                        command: "find".into(),
                    })
                })
            }
        }
        let runner = DelayedRunner {
            entered: Arc::new(Notify::new()),
            exited: Arc::new(AtomicBool::new(false)),
        };
        let entered = runner.entered.clone();
        let exited = runner.exited.clone();
        let connection_id = ConnectionId::new();
        let job = start(
            connection_id,
            PathBuf::from("/tmp/delayed-preload"),
            Duration::from_secs(10),
            Some(2),
            false,
        )
        .unwrap();
        let monitor = tokio::spawn(async move { monitor_with(&runner, job).await });
        tokio::time::timeout(Duration::from_secs(2), entered.notified())
            .await
            .unwrap();
        cancel_before_unmount(connection_id).await.unwrap();
        assert!(exited.load(Ordering::Acquire));
        assert_eq!(
            monitor.await.unwrap().unwrap(),
            DirectoryPreloadOutcome::Cancelled
        );
    }

    #[tokio::test]
    async fn traversal_timeout_is_a_normal_bounded_outcome() {
        let runner =
            FakeCommandRunner::default().with_resolved([Executable::Mountpoint, Executable::Find]);
        runner.push(Ok(output()));
        runner.push(Err(CommandError::Timeout {
            command: "find".into(),
            timeout: Duration::from_secs(5),
        }));
        let connection_id = id("7a3f5d45-e867-47e7-943f-66cf60e777ad");
        let job = start(
            connection_id,
            PathBuf::from("/home/user/OneDrive"),
            Duration::from_secs(5),
            Some(3),
            false,
        )
        .expect("start");
        let DirectoryPreloadOutcome::TimedOut(report) =
            monitor_with(&runner, job).await.expect("monitor")
        else {
            panic!("expected timeout report");
        };
        assert_eq!(report.limit_seconds, 5.0);
        assert_eq!(report.maximum_depth, Some(3));
        assert_eq!(report.excluded_paths, 0);
    }

    #[test]
    fn bounded_traversal_limits_depth_and_reads_directories_only() {
        let request = traversal_request_excluding(
            Path::new("/home/user/Cloud/Box"),
            Duration::from_secs(60),
            Some(2),
            &[],
        )
        .expect("request");
        assert_eq!(
            request.sanitized_command(),
            "find -P /home/user/Cloud/Box -xdev -maxdepth 2 -type d -printf "
        );
    }
}
