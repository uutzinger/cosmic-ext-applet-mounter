// SPDX-License-Identifier: MIT

//! Shared cancellation and atomic state for Offline Mirror operations.

use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tokio::time;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::model::ConnectionId;

pub const STATE_VERSION: u32 = 1;

#[derive(Debug)]
pub struct ManualOperationControl {
    cancellation: CancellationToken,
    completed: AtomicBool,
    completion: Notify,
}

impl ManualOperationControl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancellation: CancellationToken::new(),
            completed: AtomicBool::new(false),
            completion: Notify::new(),
        }
    }

    #[must_use]
    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub fn finish(&self) {
        self.completed.store(true, Ordering::Release);
        self.completion.notify_waiters();
    }

    pub async fn cancel_and_wait(&self, timeout: Duration) -> bool {
        self.cancellation.cancel();
        if self.completed.load(Ordering::Acquire) {
            return true;
        }
        let notified = self.completion.notified();
        if self.completed.load(Ordering::Acquire) {
            return true;
        }
        let _ = time::timeout(timeout, notified).await;
        self.completed.load(Ordering::Acquire)
    }
}

impl Default for ManualOperationControl {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OfflineOperation {
    InitialPreview,
    Preview,
    InitialSync,
    Sync,
    Scheduled,
    Resume,
    Pause,
    Repair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OfflineOperationState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineCoordinatorState {
    pub version: u32,
    pub connection_id: ConnectionId,
    pub operation: OfflineOperation,
    pub state: OfflineOperationState,
    pub updated_unix_seconds: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl OfflineCoordinatorState {
    #[must_use]
    pub fn new(
        connection_id: ConnectionId,
        operation: OfflineOperation,
        state: OfflineOperationState,
    ) -> Self {
        Self {
            version: STATE_VERSION,
            connection_id,
            operation,
            state,
            updated_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

#[must_use]
pub fn state_path(work_root: &Path, connection_id: ConnectionId) -> PathBuf {
    work_root
        .join("offline-coordinator")
        .join(connection_id.to_string())
        .join("state.json")
}

pub fn read_state(path: &Path) -> Result<Option<OfflineCoordinatorState>, String> {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(crate::fl!(
                "ui-offline-coordinator-could-not-read-offline-mirror-state-error",
                error = (error).to_string()
            ));
        }
    };
    let state: OfflineCoordinatorState = serde_json::from_slice(&contents).map_err(|error| {
        crate::fl!(
            "ui-offline-coordinator-offline-mirror-state-is-invalid-error",
            error = (error).to_string()
        )
    })?;
    if state.version != STATE_VERSION {
        return Err(crate::fl!(
            "ui-offline-coordinator-offline-mirror-state-version-value1-is-unsupported",
            value1 = (state.version).to_string()
        ));
    }
    Ok(Some(state))
}

pub fn write_state(path: &Path, state: &OfflineCoordinatorState) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        crate::fl!("ui-offline-coordinator-offline-mirror-state-path-has-no-parent")
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        crate::fl!(
            "ui-offline-coordinator-could-not-create-offline-mirror-state-directory",
            error = (error).to_string()
        )
    })?;
    #[cfg(unix)]
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|error| {
        crate::fl!(
            "ui-offline-coordinator-could-not-restrict-offline-mirror-state-directory",
            error = (error).to_string()
        )
    })?;

    let staging = parent.join(format!(".state-{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)
            .map_err(|error| {
                crate::fl!(
                    "ui-offline-coordinator-could-not-stage-offline-mirror-state-error",
                    error = (error).to_string()
                )
            })?;
        #[cfg(unix)]
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                crate::fl!(
                    "ui-offline-coordinator-could-not-restrict-offline-mirror-state-error",
                    error = (error).to_string()
                )
            })?;
        let contents = serde_json::to_vec(state).map_err(|error| {
            crate::fl!(
                "ui-offline-coordinator-could-not-encode-offline-mirror-state-error",
                error = (error).to_string()
            )
        })?;
        file.write_all(&contents).map_err(|error| {
            crate::fl!(
                "ui-offline-coordinator-could-not-write-offline-mirror-state-error",
                error = (error).to_string()
            )
        })?;
        file.sync_all().map_err(|error| {
            crate::fl!(
                "ui-offline-coordinator-could-not-sync-offline-mirror-state-error",
                error = (error).to_string()
            )
        })?;
        fs::rename(&staging, path).map_err(|error| {
            crate::fl!(
                "ui-offline-coordinator-could-not-publish-offline-mirror-state-error",
                error = (error).to_string()
            )
        })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(staging);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn cancellation_wait_observes_completion_without_lost_wakeup() {
        let control = Arc::new(ManualOperationControl::new());
        let operation = Arc::clone(&control);
        let waiter =
            tokio::spawn(
                async move { operation.cancel_and_wait(Duration::from_millis(250)).await },
            );
        control.cancellation().cancelled().await;
        control.finish();
        assert!(waiter.await.unwrap());

        assert!(control.cancel_and_wait(Duration::from_millis(1)).await);
    }

    #[tokio::test]
    async fn cancellation_wait_is_bounded() {
        let control = ManualOperationControl::new();
        assert!(!control.cancel_and_wait(Duration::from_millis(10)).await);
    }

    #[test]
    fn state_is_atomically_replaced_and_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let id = ConnectionId::new();
        let path = state_path(directory.path(), id);
        let running = OfflineCoordinatorState::new(
            id,
            OfflineOperation::Sync,
            OfflineOperationState::Running,
        );
        write_state(&path, &running).unwrap();
        assert_eq!(read_state(&path).unwrap(), Some(running));

        let paused = OfflineCoordinatorState::new(
            id,
            OfflineOperation::Repair,
            OfflineOperationState::Paused,
        )
        .with_warnings(vec!["cleanup warning".into()]);
        write_state(&path, &paused).unwrap();
        assert_eq!(read_state(&path).unwrap(), Some(paused));
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn state_without_warnings_remains_backward_compatible() {
        let directory = tempfile::tempdir().unwrap();
        let id = ConnectionId::new();
        let path = state_path(directory.path(), id);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            format!(
                "{{\"version\":1,\"connection_id\":\"{id}\",\"operation\":\"sync\",\"state\":\"succeeded\",\"updated_unix_seconds\":1}}"
            ),
        )
        .unwrap();

        assert!(read_state(&path).unwrap().unwrap().warnings.is_empty());
    }
}
