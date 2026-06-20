// SPDX-License-Identifier: MIT

//! Core storage connection, provider, VPN, operation, and status types.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

uuid_id!(ConnectionId);
uuid_id!(VpnProfileId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    OneDrive,
    GoogleDrive,
    Box,
    Smb,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionMode {
    OnlineMount(OnlineMountConfig),
    OfflineMirror(OfflineMirrorConfig),
}

impl ConnectionMode {
    #[must_use]
    pub const fn kind(&self) -> AccessMode {
        match self {
            Self::OnlineMount(_) => AccessMode::OnlineMount,
            Self::OfflineMirror(_) => AccessMode::OfflineMirror,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMode {
    OnlineMount,
    OfflineMirror,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineMountConfig {
    pub cache_directory: Option<PathBuf>,
    pub cache_limit_bytes: u64,
    pub start_at_login: bool,
}

impl Default for OnlineMountConfig {
    fn default() -> Self {
        Self {
            cache_directory: None,
            cache_limit_bytes: 20 * 1024 * 1024 * 1024,
            start_at_login: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineMirrorConfig {
    pub recovery_directory: PathBuf,
    pub sync_interval_minutes: u32,
    pub sync_on_metered: bool,
}

impl Default for OfflineMirrorConfig {
    fn default() -> Self {
        Self {
            recovery_directory: PathBuf::new(),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TuningProfile {
    #[default]
    Balanced,
    Conservative,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub name: String,
    pub provider: Provider,
    pub mode: ConnectionMode,
    pub remote_reference: String,
    pub remote_subpath: Option<String>,
    pub local_path: PathBuf,
    pub enabled: bool,
    pub vpn_profile_id: Option<VpnProfileId>,
    pub disconnect_vpn_when_unused: bool,
    pub tuning_profile: TuningProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VpnKind {
    NetworkManager,
    Cisco,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessCheck {
    NetworkManagerState,
    Interface(String),
    Route(String),
    DnsName(String),
    Endpoint(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VpnProfile {
    pub id: VpnProfileId,
    pub name: String,
    pub kind: VpnKind,
    pub external_profile_id: Option<String>,
    pub readiness_checks: Vec<ReadinessCheck>,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Mount,
    Unmount,
    SyncNow,
    PauseSync,
    ResumeSync,
    PreviewInitialSync,
    Repair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnlineMountStatus {
    Unmounted,
    WaitingForNetwork,
    WaitingForVpn,
    Mounting,
    Mounted,
    PendingWrites,
    Detaching,
    Error,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineMirrorStatus {
    Idle,
    Offline,
    WaitingForVpn,
    Previewing,
    Syncing,
    Paused,
    MeteredPaused,
    Conflict,
    Error,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    OnlineMount(OnlineMountStatus),
    OfflineMirror(OfflineMirrorStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictRecord {
    pub connection_id: ConnectionId,
    pub relative_path: PathBuf,
    pub preserved_local_path: PathBuf,
    pub preserved_remote_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReason {
    Deleted,
    Overwritten,
    InterruptedSync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRecord {
    pub connection_id: ConnectionId,
    pub original_relative_path: PathBuf,
    pub recovery_path: PathBuf,
    pub reason: RecoveryReason,
    pub retained_until_unix_seconds: i64,
}
