// SPDX-License-Identifier: MIT

//! Mount-table and synchronization runtime interfaces.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::model::ConnectionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    pub target: PathBuf,
    pub source: String,
    pub filesystem: String,
    pub options: Vec<String>,
}

pub trait MountTable: Send + Sync {
    fn entries(&self) -> Result<Vec<MountEntry>, MountTableError>;

    fn find_target(&self, target: &Path) -> Result<Option<MountEntry>, MountTableError> {
        Ok(self
            .entries()?
            .into_iter()
            .find(|entry| entry.target == target))
    }
}

#[derive(Debug)]
pub struct MountTableError(pub String);

impl std::fmt::Display for MountTableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MountTableError {}

#[derive(Debug, Clone)]
pub struct ProcMountTable {
    path: PathBuf,
}

impl Default for ProcMountTable {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/proc/self/mountinfo"),
        }
    }
}

impl ProcMountTable {
    #[must_use]
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }
}

impl MountTable for ProcMountTable {
    fn entries(&self) -> Result<Vec<MountEntry>, MountTableError> {
        let content = fs::read_to_string(&self.path)
            .map_err(|error| MountTableError(format!("read mount table: {error}")))?;
        content
            .lines()
            .map(parse_mountinfo_line)
            .collect::<Result<Vec<_>, _>>()
    }
}

fn parse_mountinfo_line(line: &str) -> Result<MountEntry, MountTableError> {
    let (left, right) = line
        .split_once(" - ")
        .ok_or_else(|| MountTableError("mountinfo line lacks separator".into()))?;
    let left: Vec<_> = left.split_whitespace().collect();
    let right: Vec<_> = right.split_whitespace().collect();
    if left.len() < 6 || right.len() < 2 {
        return Err(MountTableError("mountinfo line is incomplete".into()));
    }
    Ok(MountEntry {
        target: PathBuf::from(unescape_mount_field(left[4])),
        options: left[5].split(',').map(str::to_owned).collect(),
        filesystem: right[0].to_owned(),
        source: unescape_mount_field(right[1]),
    })
}

fn unescape_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncRuntimeState {
    Idle,
    Running,
    Paused,
    Failed,
    Unknown,
}

pub trait SyncRuntime: Send + Sync {
    fn state(&self, connection_id: ConnectionId) -> SyncRuntimeState;
}

#[derive(Clone, Default)]
pub struct FakeMountTable {
    entries: Arc<Mutex<Vec<MountEntry>>>,
}

impl FakeMountTable {
    pub fn set(&self, entries: Vec<MountEntry>) {
        *self.entries.lock().expect("fake mount table") = entries;
    }
}

impl MountTable for FakeMountTable {
    fn entries(&self) -> Result<Vec<MountEntry>, MountTableError> {
        Ok(self.entries.lock().expect("fake mount table").clone())
    }
}

#[derive(Clone, Default)]
pub struct FakeSyncRuntime {
    states: Arc<Mutex<HashMap<ConnectionId, SyncRuntimeState>>>,
}

impl FakeSyncRuntime {
    pub fn set(&self, connection_id: ConnectionId, state: SyncRuntimeState) {
        self.states
            .lock()
            .expect("fake sync runtime")
            .insert(connection_id, state);
    }
}

impl SyncRuntime for FakeSyncRuntime {
    fn state(&self, connection_id: ConnectionId) -> SyncRuntimeState {
        self.states
            .lock()
            .expect("fake sync runtime")
            .get(&connection_id)
            .copied()
            .unwrap_or(SyncRuntimeState::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    use super::*;

    fn id() -> ConnectionId {
        ConnectionId::from_uuid(
            Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
        )
    }

    #[test]
    fn parses_mountinfo_and_decodes_paths() {
        let file = NamedTempFile::new().expect("temp");
        fs::write(
            file.path(),
            "36 25 0:32 / /home/example/Cloud\\040Drive rw,nosuid - fuse.rclone remote: rw\n",
        )
        .expect("fixture");
        let table = ProcMountTable::with_path(file.path().into());
        let entries = table.entries().expect("parse mount table");
        assert_eq!(
            entries,
            vec![MountEntry {
                target: PathBuf::from("/home/example/Cloud Drive"),
                source: "remote:".into(),
                filesystem: "fuse.rclone".into(),
                options: vec!["rw".into(), "nosuid".into()],
            }]
        );
    }

    #[test]
    fn fake_mount_and_sync_runtime_are_isolated() {
        let mounts = FakeMountTable::default();
        mounts.set(vec![MountEntry {
            target: PathBuf::from("/tmp/mount"),
            source: "fake:".into(),
            filesystem: "fuse.fake".into(),
            options: vec!["rw".into()],
        }]);
        assert!(
            mounts
                .find_target(Path::new("/tmp/mount"))
                .expect("find mount")
                .is_some()
        );

        let sync = FakeSyncRuntime::default();
        sync.set(id(), SyncRuntimeState::Running);
        assert_eq!(sync.state(id()), SyncRuntimeState::Running);
    }
}
