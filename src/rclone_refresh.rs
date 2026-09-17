// SPDX-License-Identifier: MIT

//! Background Google Drive VFS directory-cache refresh lifecycle.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use serde_json::Value;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::model::ConnectionId;
use crate::process::{CommandRequest, CommandRunner, Executable, RuntimeCommandRunner};
use crate::sleep::begin_online_operation;

const RC_READY_TIMEOUT: Duration = Duration::from_secs(15);
const RC_PROBE_INTERVAL: Duration = Duration::from_millis(250);
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(1);
const RC_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const CANCELLED: &str = "Google Drive directory refresh canceled";

#[derive(Debug, Clone, PartialEq)]
pub struct RcloneRefreshJob {
    pub connection_id: ConnectionId,
    pub job_id: u64,
    pub execute_id: String,
    rc_socket: PathBuf,
    timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RcloneRefreshOutcome {
    Completed { duration_seconds: f64 },
    TimedOut { duration_seconds: f64 },
    Cancelled,
}

#[derive(Debug)]
struct ActiveRefresh {
    rc_socket: PathBuf,
    cancellation: CancellationToken,
    job_id: Mutex<Option<u64>>,
}

static ACTIVE_REFRESHES: LazyLock<Mutex<HashMap<ConnectionId, Arc<ActiveRefresh>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn start(
    connection_id: ConnectionId,
    rc_socket: PathBuf,
    mountpoint: PathBuf,
    timeout: Duration,
) -> Result<RcloneRefreshJob, String> {
    let runner = RuntimeCommandRunner::detect_current();
    start_with(&runner, connection_id, rc_socket, mountpoint, timeout).await
}

pub async fn monitor(job: RcloneRefreshJob) -> Result<RcloneRefreshOutcome, String> {
    let runner = RuntimeCommandRunner::detect_current();
    monitor_with(&runner, job).await
}

pub async fn cancel(connection_id: ConnectionId) -> Result<bool, String> {
    let runner = RuntimeCommandRunner::detect_current();
    cancel_with(&runner, connection_id).await
}

async fn start_with(
    runner: &dyn CommandRunner,
    connection_id: ConnectionId,
    rc_socket: PathBuf,
    mountpoint: PathBuf,
    timeout: Duration,
) -> Result<RcloneRefreshJob, String> {
    let _ = cancel_with(runner, connection_id).await;
    let operation = begin_online_operation()?;
    let control = Arc::new(ActiveRefresh {
        rc_socket: rc_socket.clone(),
        cancellation: CancellationToken::new(),
        job_id: Mutex::new(None),
    });
    ACTIVE_REFRESHES
        .lock()
        .expect("rclone refresh registry")
        .insert(connection_id, Arc::clone(&control));

    let result = async {
        let deadline = Instant::now() + RC_READY_TIMEOUT;
        loop {
            let mount_probe = CommandRequest::new(Executable::Mountpoint)
                .arg("--quiet")
                .map_err(|error| error.to_string())?
                .arg(mountpoint.as_os_str())
                .map_err(|error| error.to_string())?
                .with_timeout(RC_COMMAND_TIMEOUT);
            let mount_ready = tokio::select! {
                biased;
                () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
                () = operation.cancelled() => return Err(CANCELLED.into()),
                result = runner.run(mount_probe, control.cancellation.child_token()) => result.is_ok(),
            };
            let rc_ready = if mount_ready {
                let probe = rc_request(&rc_socket, "rc/noop")?;
                tokio::select! {
                    biased;
                    () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
                    () = operation.cancelled() => return Err(CANCELLED.into()),
                    result = runner.run(probe, control.cancellation.child_token()) => result.is_ok(),
                }
            } else {
                false
            };
            if mount_ready && rc_ready {
                break;
            }
            if Instant::now() >= deadline {
                return Err("rclone RC socket did not become ready within 15 seconds".into());
            }
            tokio::select! {
                biased;
                () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
                () = operation.cancelled() => return Err(CANCELLED.into()),
                () = tokio::time::sleep(RC_PROBE_INTERVAL) => {}
            }
        }

        let request = rc_request(&rc_socket, "vfs/refresh")?
            .arg("recursive=true")
            .map_err(|error| error.to_string())?
            .arg("fast_list=true")
            .map_err(|error| error.to_string())?
            .arg("_async=true")
            .map_err(|error| error.to_string())?;
        let output = tokio::select! {
            biased;
            () = control.cancellation.cancelled() => return Err(CANCELLED.into()),
            () = operation.cancelled() => return Err(CANCELLED.into()),
            result = runner.run(request, control.cancellation.child_token()) => {
                result.map_err(|error| error.to_string())?
            }
        };
        let (job_id, execute_id) = parse_job_start(&output.stdout.text)?;
        *control.job_id.lock().expect("rclone refresh job id") = Some(job_id);
        if control.cancellation.is_cancelled() || !is_current(connection_id, &control) {
            let _ = stop_job(runner, &rc_socket, job_id).await;
            return Err(CANCELLED.into());
        }
        Ok(RcloneRefreshJob {
            connection_id,
            job_id,
            execute_id,
            rc_socket,
            timeout,
        })
    }
    .await;

    if result.is_err() {
        finish(connection_id, &control);
    }
    result
}

async fn monitor_with(
    runner: &dyn CommandRunner,
    job: RcloneRefreshJob,
) -> Result<RcloneRefreshOutcome, String> {
    let Some(control) = current_control(job.connection_id, job.job_id) else {
        return Ok(RcloneRefreshOutcome::Cancelled);
    };
    let operation = match begin_online_operation() {
        Ok(operation) => operation,
        Err(_) => {
            let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
            finish(job.connection_id, &control);
            return Ok(RcloneRefreshOutcome::Cancelled);
        }
    };

    let started = Instant::now();
    let deadline = started + job.timeout;
    let result = async {
        loop {
            if Instant::now() >= deadline {
                let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
                break Ok(RcloneRefreshOutcome::TimedOut {
                    duration_seconds: started.elapsed().as_secs_f64(),
                });
            }
            let request = rc_request(&job.rc_socket, "job/status")?
                .arg(format!("jobid={}", job.job_id))
                .map_err(|error| error.to_string())?;
            let output = tokio::select! {
                biased;
                () = control.cancellation.cancelled() => {
                    let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
                    break Ok(RcloneRefreshOutcome::Cancelled);
                }
                () = operation.cancelled() => {
                    let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
                    break Ok(RcloneRefreshOutcome::Cancelled);
                }
                result = runner.run(request, control.cancellation.child_token()) => {
                    result.map_err(|error| error.to_string())?
                }
            };
            let status = parse_job_status(&output.stdout.text, &job.execute_id, job.job_id)?;
            if status.finished {
                if status.success {
                    break Ok(RcloneRefreshOutcome::Completed {
                        duration_seconds: status.duration_seconds,
                    });
                }
                break Err(if status.error.is_empty() {
                    "rclone directory refresh finished without success".into()
                } else {
                    format!("rclone directory refresh failed: {}", status.error)
                });
            }
            tokio::select! {
                biased;
                () = control.cancellation.cancelled() => {
                    let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
                    break Ok(RcloneRefreshOutcome::Cancelled);
                }
                () = operation.cancelled() => {
                    let _ = stop_job(runner, &job.rc_socket, job.job_id).await;
                    break Ok(RcloneRefreshOutcome::Cancelled);
                }
                () = tokio::time::sleep(JOB_POLL_INTERVAL) => {}
            }
        }
    }
    .await;
    finish(job.connection_id, &control);
    result
}

async fn cancel_with(
    runner: &dyn CommandRunner,
    connection_id: ConnectionId,
) -> Result<bool, String> {
    let control = ACTIVE_REFRESHES
        .lock()
        .expect("rclone refresh registry")
        .get(&connection_id)
        .cloned();
    let Some(control) = control else {
        return Ok(false);
    };
    control.cancellation.cancel();
    let job_id = *control.job_id.lock().expect("rclone refresh job id");
    let stop_result = if let Some(job_id) = job_id {
        stop_job(runner, &control.rc_socket, job_id).await
    } else {
        Ok(())
    };
    finish(connection_id, &control);
    stop_result.map(|()| true)
}

fn rc_request(rc_socket: &Path, method: &str) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("rc")
        .map_err(|error| error.to_string())?
        .arg("--unix-socket")
        .map_err(|error| error.to_string())?
        .arg(rc_socket.as_os_str())
        .map_err(|error| error.to_string())?
        .arg(method)
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(RC_COMMAND_TIMEOUT))
}

async fn stop_job(runner: &dyn CommandRunner, rc_socket: &Path, job_id: u64) -> Result<(), String> {
    let request = rc_request(rc_socket, "job/stop")?
        .arg(format!("jobid={job_id}"))
        .map_err(|error| error.to_string())?;
    runner
        .run(request, CancellationToken::new())
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn current_control(connection_id: ConnectionId, job_id: u64) -> Option<Arc<ActiveRefresh>> {
    ACTIVE_REFRESHES
        .lock()
        .expect("rclone refresh registry")
        .get(&connection_id)
        .filter(|control| *control.job_id.lock().expect("rclone refresh job id") == Some(job_id))
        .cloned()
}

fn is_current(connection_id: ConnectionId, control: &Arc<ActiveRefresh>) -> bool {
    ACTIVE_REFRESHES
        .lock()
        .expect("rclone refresh registry")
        .get(&connection_id)
        .is_some_and(|current| Arc::ptr_eq(current, control))
}

fn finish(connection_id: ConnectionId, control: &Arc<ActiveRefresh>) {
    let mut active = ACTIVE_REFRESHES.lock().expect("rclone refresh registry");
    if active
        .get(&connection_id)
        .is_some_and(|current| Arc::ptr_eq(current, control))
    {
        active.remove(&connection_id);
    }
}

fn parse_job_start(output: &str) -> Result<(u64, String), String> {
    let value: Value = serde_json::from_str(output)
        .map_err(|error| format!("invalid rclone refresh response: {error}"))?;
    let job_id = value
        .get("jobid")
        .and_then(Value::as_u64)
        .ok_or_else(|| "rclone refresh response did not contain a job ID".to_owned())?;
    let execute_id = value
        .get("executeId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "rclone refresh response did not contain an execution ID".to_owned())?
        .to_owned();
    Ok((job_id, execute_id))
}

#[derive(Debug, Clone, PartialEq)]
struct JobStatus {
    finished: bool,
    success: bool,
    duration_seconds: f64,
    error: String,
}

fn parse_job_status(output: &str, execute_id: &str, job_id: u64) -> Result<JobStatus, String> {
    let value: Value = serde_json::from_str(output)
        .map_err(|error| format!("invalid rclone job status: {error}"))?;
    if value.get("id").and_then(Value::as_u64) != Some(job_id) {
        return Err("rclone job status returned a different job ID".into());
    }
    let returned_execute_id = value
        .get("executeId")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if returned_execute_id != execute_id {
        return Err("rclone restarted while the directory refresh was running".into());
    }
    let provider_result_failed = value
        .pointer("/output/result")
        .and_then(Value::as_object)
        .is_some_and(|result| {
            result.values().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| !message.eq_ignore_ascii_case("OK"))
            })
        });
    let mut success = value
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut error = value
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    if provider_result_failed {
        success = false;
        error = "the storage provider rejected part of the directory refresh".into();
    }
    Ok(JobStatus {
        finished: value
            .get("finished")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        success,
        duration_seconds: value.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
        error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};
    use uuid::Uuid;

    fn id(value: &str) -> ConnectionId {
        ConnectionId::from_uuid(Uuid::parse_str(value).expect("uuid"))
    }

    fn output(text: &str) -> CommandOutput {
        CommandOutput {
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
        }
    }

    #[tokio::test]
    async fn refresh_start_and_completion_retain_job_identity() {
        let runner = FakeCommandRunner::default()
            .with_resolved([Executable::Rclone, Executable::Mountpoint]);
        runner.push(Ok(output("")));
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(r#"{"jobid":7,"executeId":"instance-a"}"#)));
        runner.push(Ok(output(
            r#"{"id":7,"executeId":"instance-a","finished":true,"success":true,"duration":2.5,"error":""}"#,
        )));
        let connection_id = id("2a3f5d45-e867-47e7-943f-66cf60e777ad");
        let socket = PathBuf::from("/run/user/1000/rclone-test.sock");
        let mountpoint = PathBuf::from("/home/example/Google Drive");

        let job = start_with(
            &runner,
            connection_id,
            socket,
            mountpoint,
            Duration::from_secs(60),
        )
        .await
        .expect("start");
        assert_eq!(job.job_id, 7);
        assert_eq!(job.execute_id, "instance-a");
        assert_eq!(
            monitor_with(&runner, job).await.expect("monitor"),
            RcloneRefreshOutcome::Completed {
                duration_seconds: 2.5
            }
        );

        let requests = runner.requests();
        assert_eq!(
            requests[0].sanitized_command(),
            "mountpoint --quiet /home/example/Google Drive"
        );
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone rc --unix-socket /run/user/1000/rclone-test.sock rc/noop"
        );
        assert_eq!(
            requests[2].sanitized_command(),
            "rclone rc --unix-socket /run/user/1000/rclone-test.sock vfs/refresh recursive=true fast_list=true _async=true"
        );
        assert_eq!(
            requests[3].sanitized_command(),
            "rclone rc --unix-socket /run/user/1000/rclone-test.sock job/status jobid=7"
        );
    }

    #[tokio::test]
    async fn cancel_stops_the_tracked_job() {
        let runner = FakeCommandRunner::default()
            .with_resolved([Executable::Rclone, Executable::Mountpoint]);
        runner.push(Ok(output("")));
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(r#"{"jobid":9,"executeId":"instance-b"}"#)));
        runner.push(Ok(output("{}")));
        let connection_id = id("3a3f5d45-e867-47e7-943f-66cf60e777ad");
        let job = start_with(
            &runner,
            connection_id,
            PathBuf::from("/run/user/1000/rclone-cancel.sock"),
            PathBuf::from("/home/example/Google Drive"),
            Duration::from_secs(60),
        )
        .await
        .expect("start");
        assert_eq!(job.job_id, 9);
        assert!(cancel_with(&runner, connection_id).await.expect("cancel"));
        assert_eq!(
            runner.requests()[3].sanitized_command(),
            "rclone rc --unix-socket /run/user/1000/rclone-cancel.sock job/stop jobid=9"
        );
    }

    #[test]
    fn job_status_rejects_a_restarted_rclone_instance() {
        let error = parse_job_status(
            r#"{"id":4,"executeId":"new-instance","finished":false,"success":false}"#,
            "old-instance",
            4,
        )
        .expect_err("restart must be detected");
        assert!(error.contains("restarted"));

        let missing_execute_id = parse_job_status(
            r#"{"id":4,"finished":false,"success":false}"#,
            "old-instance",
            4,
        )
        .expect_err("missing instance identity must be rejected");
        assert!(missing_execute_id.contains("restarted"));
    }

    #[test]
    fn job_status_treats_nested_provider_errors_as_failure() {
        let status = parse_job_status(
            r#"{"id":4,"executeId":"instance","finished":true,"success":true,"output":{"result":{"":"directory refresh failed for a private path"}}}"#,
            "instance",
            4,
        )
        .expect("parse status");
        assert!(!status.success);
        assert_eq!(
            status.error,
            "the storage provider rejected part of the directory refresh"
        );
        assert!(!status.error.contains("private path"));
    }
}
