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
    Teams,
    GoogleDrive,
    Box,
    Smb,
    Sftp,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<OfflineEngine>,
    pub recovery_directory: PathBuf,
    pub sync_interval_minutes: u32,
    pub sync_on_metered: bool,
}

impl Default for OfflineMirrorConfig {
    fn default() -> Self {
        Self {
            engine: None,
            recovery_directory: PathBuf::new(),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }
    }
}

/// Absent on old documents; resolved using the connection's provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfflineEngine {
    Rclone,
    LegacyOnedrive,
}

/// Compatibility profiles remain readable even when their engine is retired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedEngine(pub OfflineEngine);

impl fmt::Display for UnsupportedEngine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&crate::fl!("onedrive-legacy-engine-retired"))
    }
}

impl std::error::Error for UnsupportedEngine {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OneDriveDriveType {
    Personal,
    Business,
}

/// Stable drive binding, never an OAuth token or an account credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneDriveIdentity {
    pub drive_id: String,
    pub drive_type: OneDriveDriveType,
}

impl OneDriveIdentity {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.drive_id.is_empty()
            && self.drive_id.len() <= 512
            && self.drive_id.is_ascii()
            && !self
                .drive_id
                .chars()
                .any(|ch| ch.is_control() || ch.is_whitespace() || matches!(ch, '/' | '\\'))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TuningProfile {
    #[default]
    Balanced,
    Conservative,
    Custom,
}

pub const DEFAULT_PRELOAD_SECONDS: u64 = 60;
pub const DEFAULT_BOX_PRELOAD_DEPTH: u8 = 2;
pub const DEFAULT_SMB_PRELOAD_DEPTH: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreloadPolicy {
    pub enabled: bool,
    pub maximum_seconds: u64,
    pub maximum_depth: Option<u8>,
}

impl PreloadPolicy {
    #[must_use]
    pub const fn sftp_default() -> Self {
        Self {
            enabled: false,
            maximum_seconds: 30,
            maximum_depth: Some(2),
        }
    }

    #[must_use]
    pub const fn without_depth() -> Self {
        Self {
            enabled: false,
            maximum_seconds: DEFAULT_PRELOAD_SECONDS,
            maximum_depth: None,
        }
    }

    #[must_use]
    pub const fn with_depth(maximum_depth: u8) -> Self {
        Self {
            enabled: false,
            maximum_seconds: DEFAULT_PRELOAD_SECONDS,
            maximum_depth: Some(maximum_depth),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreloadSettings {
    pub google_drive: PreloadPolicy,
    pub onedrive: PreloadPolicy,
    #[serde(rename = "box", alias = "box_provider")]
    pub box_provider: PreloadPolicy,
    #[serde(default = "PreloadPolicy::sftp_default")]
    pub sharepoint: PreloadPolicy,
    pub smb: PreloadPolicy,
    #[serde(default = "PreloadPolicy::sftp_default")]
    pub sftp: PreloadPolicy,
}

impl Default for PreloadSettings {
    fn default() -> Self {
        Self {
            google_drive: PreloadPolicy::without_depth(),
            onedrive: PreloadPolicy::without_depth(),
            box_provider: PreloadPolicy::with_depth(DEFAULT_BOX_PRELOAD_DEPTH),
            sharepoint: PreloadPolicy::sftp_default(),
            smb: PreloadPolicy::with_depth(DEFAULT_SMB_PRELOAD_DEPTH),
            sftp: PreloadPolicy::sftp_default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionPreloadOverride {
    pub enabled: bool,
    pub maximum_seconds: u64,
    pub maximum_depth: u8,
}

pub type SmbPreloadOverride = ConnectionPreloadOverride;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "TeamsLibraryIdentity")]
pub struct SharePointLibraryIdentity {
    pub site_url: String,
    pub library_url: String,
    pub drive_id: String,
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
    #[serde(default)]
    pub smb_preload_override: Option<SmbPreloadOverride>,
    #[serde(default)]
    pub sftp_preload_override: Option<ConnectionPreloadOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teams_identity: Option<SharePointLibraryIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onedrive_identity: Option<OneDriveIdentity>,
}

impl Connection {
    /// Check runtime support without changing stored identity or touching files.
    pub fn ensure_supported_engine(&self) -> Result<(), UnsupportedEngine> {
        match self.offline_engine() {
            Some(OfflineEngine::LegacyOnedrive) => {
                Err(UnsupportedEngine(OfflineEngine::LegacyOnedrive))
            }
            _ => Ok(()),
        }
    }

    /// Resolves legacy documents without migrating or silently changing engines.
    #[must_use]
    pub fn offline_engine(&self) -> Option<OfflineEngine> {
        match &self.mode {
            ConnectionMode::OnlineMount(_) => None,
            ConnectionMode::OfflineMirror(options) => Some(options.engine.unwrap_or(
                if self.provider == Provider::OneDrive {
                    OfflineEngine::LegacyOnedrive
                } else {
                    OfflineEngine::Rclone
                },
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VpnKind {
    NetworkManager,
    /// Read-only compatibility marker for configurations created before VPN
    /// handling became NetworkManager-only. Production code never activates
    /// this kind; the user must select a saved NetworkManager profile.
    #[serde(rename = "Cisco")]
    LegacyCisco,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    Degraded,
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
