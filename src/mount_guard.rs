// SPDX-License-Identifier: MIT

//! Read-only host checks before an Online mount covers a local directory.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::config::{APP_ID, Config, LoadSource};
use crate::fl;
use crate::model::{Connection, ConnectionId, ConnectionMode};
use crate::mounts::parse_mountinfo_line;
use crate::process::{
    CommandExecutionMode, CommandRequest, CommandRunner, Executable, RuntimeCommandRunner,
};
use crate::services::ServiceCommand;

pub const SERVICE_GUARD_FLAG: &str = "--verify-online-mountpoint";

pub fn service_command(arguments: Vec<String>) -> Result<ServiceCommand, String> {
    match CommandExecutionMode::detect_current() {
        CommandExecutionMode::Native => Ok(ServiceCommand {
            executable: std::env::current_exe()
                .map_err(|_| fl!("mount-guard-missing-executable"))?,
            arguments,
        }),
        CommandExecutionMode::FlatpakSpawnHost => Ok(ServiceCommand {
            executable: PathBuf::from("/usr/bin/flatpak"),
            arguments: [
                vec![
                    "run".into(),
                    "--command=cosmic-ext-applet-mounter".into(),
                    APP_ID.into(),
                ],
                arguments,
            ]
            .concat(),
        }),
    }
}

pub fn service_guard_command(connection: &Connection) -> Result<ServiceCommand, String> {
    if !matches!(connection.mode, ConnectionMode::OnlineMount(_)) {
        return Err(fl!("mount-guard-online-only"));
    }
    service_command(vec![
        SERVICE_GUARD_FLAG.into(),
        connection.id.to_string(),
        connection.local_path.display().to_string(),
    ])
}

pub async fn run_service_guard(arguments: &[String]) -> Result<(), String> {
    let [id, mountpoint] = arguments else {
        return Err(fl!("mount-guard-invalid-arguments"));
    };
    let id = uuid::Uuid::parse_str(id)
        .map(ConnectionId::from_uuid)
        .map_err(|_| fl!("mount-guard-invalid-id"))?;
    let report = Config::load_runtime();
    if report.source != LoadSource::Current {
        return Err(fl!("mount-guard-load-failed"));
    }
    let connection = report
        .config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == id)
        .ok_or_else(|| fl!("mount-guard-connection-removed"))?;
    if !connection.enabled
        || !matches!(connection.mode, ConnectionMode::OnlineMount(_))
        || connection.local_path != Path::new(mountpoint)
    {
        return Err(fl!("mount-guard-connection-changed"));
    }
    check_empty_mountpoint(
        &RuntimeCommandRunner::detect_current(),
        &connection.local_path,
    )
    .await
}

pub async fn check_empty_mountpoint<R: CommandRunner>(
    runner: &R,
    mountpoint: &Path,
) -> Result<(), String> {
    let mountinfo = runner
        .run(
            CommandRequest::new(Executable::Cat)
                .arg("/proc/self/mountinfo")
                .map_err(|error| error.to_string())?
                .with_output_limit(4 * 1024 * 1024)
                .with_timeout(Duration::from_secs(5)),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| fl!("mount-guard-inspect-mounts", error = error.to_string()))?;
    if mountinfo.stdout.truncated || mountinfo.stdout.invalid_utf8 {
        return Err(fl!("mount-guard-unreadable-table"));
    }
    for line in mountinfo.stdout.text.lines() {
        let entry = parse_mountinfo_line(line)
            .map_err(|error| fl!("mount-guard-inspect-mounts", error = error.to_string()))?;
        if entry.target == mountpoint {
            return Err(fl!(
                "mount-guard-already-mounted",
                path = mountpoint.display().to_string()
            ));
        }
    }

    let directory = runner
        .run(
            CommandRequest::new(Executable::Find)
                .arg(mountpoint.as_os_str())
                .and_then(|request| request.arg("-maxdepth"))
                .and_then(|request| request.arg("0"))
                .and_then(|request| request.arg("-type"))
                .and_then(|request| request.arg("d"))
                .and_then(|request| request.arg("-print"))
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(5)),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| fl!("mount-guard-inspect-directory", error = error.to_string()))?;
    if directory.stdout.truncated
        || directory.stdout.invalid_utf8
        || directory.stdout.text.is_empty()
    {
        return Err(fl!(
            "mount-guard-not-directory",
            path = mountpoint.display().to_string()
        ));
    }

    let entries = runner
        .run(
            CommandRequest::new(Executable::Find)
                .arg(mountpoint.as_os_str())
                .and_then(|request| request.arg("-mindepth"))
                .and_then(|request| request.arg("1"))
                .and_then(|request| request.arg("-maxdepth"))
                .and_then(|request| request.arg("1"))
                .and_then(|request| request.arg("-print"))
                .and_then(|request| request.arg("-quit"))
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(5))
                .with_output_limit(4096),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| fl!("mount-guard-inspect-contents", error = error.to_string()))?;
    if entries.stdout.truncated || entries.stdout.invalid_utf8 || !entries.stdout.text.is_empty() {
        return Err(fl!(
            "mount-guard-local-entries",
            path = mountpoint.display().to_string()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner, SystemCommandRunner};

    #[tokio::test]
    async fn host_check_preserves_local_entries_and_rejects_symlinks() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("mount");
        std::fs::create_dir(&target).unwrap();
        assert!(
            check_empty_mountpoint(&SystemCommandRunner, &target)
                .await
                .is_ok()
        );

        let local = target.join("BME");
        std::fs::create_dir(&local).unwrap();
        let error = check_empty_mountpoint(&SystemCommandRunner, &target)
            .await
            .unwrap_err();
        assert!(error.contains("contains local entries"));
        assert!(local.is_dir());

        let link = root.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(
            check_empty_mountpoint(&SystemCommandRunner, &link)
                .await
                .unwrap_err()
                .contains("not an ordinary directory")
        );
    }

    #[tokio::test]
    async fn existing_mount_is_blocked_before_directory_scan() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(CommandOutput {
            command: "fake".into(),
            stdout: CapturedOutput {
                text: "1 2 0:1 / /tmp/mount rw - fuse.rclone remote: rw\n".into(),
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
        }));
        assert!(
            check_empty_mountpoint(&runner, Path::new("/tmp/mount"))
                .await
                .unwrap_err()
                .contains("already mounted")
        );
        assert_eq!(runner.requests().len(), 1);
    }
}
