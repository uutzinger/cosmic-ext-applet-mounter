// SPDX-License-Identifier: MIT

//! Bounded, directory-only warm-up for Online OneDrive, Box, and SMB mounts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::model::ConnectionId;
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryPreloadOutcome {
    Completed { duration_seconds: f64 },
    TimedOut { duration_seconds: f64 },
    Cancelled,
}

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
    })
}

pub async fn monitor(job: DirectoryPreloadJob) -> Result<DirectoryPreloadOutcome, String> {
    let runner = RuntimeCommandRunner::detect_current();
    monitor_with(&runner, job).await
}

pub async fn cancel(connection_id: ConnectionId) -> bool {
    let control = ACTIVE_PRELOADS
        .lock()
        .expect("directory preload registry")
        .get(&connection_id)
        .cloned();
    let Some(control) = control else {
        return false;
    };
    control.cancellation.cancel();
    let completion = control.completion.notified();
    if !control.done.load(Ordering::Acquire) {
        let _ = tokio::time::timeout(CANCEL_WAIT, completion).await;
    }
    finish(connection_id, &control);
    true
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
    let result = async {
        if let Err(error) =
            wait_for_mount(runner, &job.mountpoint, deadline, &control, &operation).await
        {
            return if error == CANCELLED {
                Ok(DirectoryPreloadOutcome::Cancelled)
            } else if error == TIMED_OUT {
                Ok(DirectoryPreloadOutcome::TimedOut {
                    duration_seconds: started.elapsed().as_secs_f64(),
                })
            } else {
                Err(error)
            };
        }
        if job.wait_for_root
            && let Err(error) =
                wait_for_root_entries(runner, &job.mountpoint, deadline, &control, &operation).await
        {
            return if error == TIMED_OUT {
                Ok(DirectoryPreloadOutcome::TimedOut {
                    duration_seconds: started.elapsed().as_secs_f64(),
                })
            } else if error == CANCELLED {
                Ok(DirectoryPreloadOutcome::Cancelled)
            } else {
                Err(error)
            };
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(DirectoryPreloadOutcome::TimedOut {
                duration_seconds: started.elapsed().as_secs_f64(),
            });
        }
        let request = traversal_request(&job.mountpoint, remaining, job.maximum_depth)?;
        let result = tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Ok(DirectoryPreloadOutcome::Cancelled),
            () = operation.cancelled() => return Ok(DirectoryPreloadOutcome::Cancelled),
            result = runner.run(request, control.cancellation.child_token()) => result,
        };
        match result {
            Ok(_) => Ok(DirectoryPreloadOutcome::Completed {
                duration_seconds: started.elapsed().as_secs_f64(),
            }),
            Err(CommandError::Timeout { .. }) => Ok(DirectoryPreloadOutcome::TimedOut {
                duration_seconds: started.elapsed().as_secs_f64(),
            }),
            Err(CommandError::Cancelled { .. }) if control.cancellation.is_cancelled() => {
                Ok(DirectoryPreloadOutcome::Cancelled)
            }
            Err(error) => Err(error.to_string()),
        }
    }
    .await;
    finish(job.connection_id, &control);
    result
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
        let result = tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
            () = operation.cancelled() => return Err(CANCELLED.into()),
            result = runner.run(probe, control.cancellation.child_token()) => result,
        };
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
        let probe_result = tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
            () = operation.cancelled() => return Err(CANCELLED.into()),
            result = runner.run(probe, control.cancellation.child_token()) => result,
        };
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

fn traversal_request(
    mountpoint: &Path,
    timeout: Duration,
    maximum_depth: Option<u8>,
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
            DirectoryPreloadOutcome::Completed { .. }
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
            None,
            false,
        )
        .expect("start");
        assert!(matches!(
            monitor_with(&runner, job).await.expect("monitor"),
            DirectoryPreloadOutcome::TimedOut { .. }
        ));
    }

    #[test]
    fn bounded_traversal_limits_depth_and_reads_directories_only() {
        let request = traversal_request(
            Path::new("/home/user/Cloud/Box"),
            Duration::from_secs(60),
            Some(2),
        )
        .expect("request");
        assert_eq!(
            request.sanitized_command(),
            "find -P /home/user/Cloud/Box -xdev -maxdepth 2 -type d -printf "
        );
    }
}
