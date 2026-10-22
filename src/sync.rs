// SPDX-License-Identifier: MIT

//! Offline mirror preview, scheduling, conflicts, recovery, and synchronization.

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::model::{
    Connection, ConnectionId, ConnectionMode, OfflineEngine, OfflineMirrorConfig,
    OfflineMirrorStatus, Provider, RecoveryReason, RecoveryRecord,
};
use crate::process::CommandError;
#[cfg(test)]
use crate::process::{CommandRequest, Executable, RetryPolicy};
use crate::services::{ServiceCommand, ServiceSpec, TimerSpec};

const DEFAULT_RCLONE: &str = "/usr/bin/rclone";
const RETENTION_SECONDS: i64 = 30 * 24 * 60 * 60;
// Bisync interprets max-delete as a percentage of each previous listing, unlike
// one-way sync's file-count flag. Match bisync's default; never bypass with force.
const BISYNC_MAX_DELETE_PERCENT: u8 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    InvalidMode,
    UnsupportedEngine(crate::model::UnsupportedEngine),
    UnsupportedProvider(Provider),
    InvalidRemoteReference,
    InvalidRemoteSubpath,
    InvalidSharePointIdentity,
    InvalidOneDriveIdentity,
    InvalidWorkDirectory,
    InvalidRecoveryDirectory,
    InitialPreviewRequired,
    ExplicitConfirmationRequired,
    ResyncPreviewRequired,
    MeteredNetworkPaused,
    ConcurrentRun,
    OverlapsOnedriverPath(PathBuf),
    Command(CommandError),
}

impl fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedEngine(error) => error.fmt(formatter),
            Self::InvalidMode => {
                formatter.write_str(&crate::fl!("ui-sync-connection-is-not-an-offline-mirror"))
            }
            Self::UnsupportedProvider(provider) => formatter.write_str(&crate::fl!(
                "ui-sync-unsupported-provider-provider",
                provider = format!("{:?}", provider)
            )),
            Self::InvalidRemoteReference => {
                formatter.write_str(&crate::fl!("ui-sync-invalid-remote-reference"))
            }
            Self::InvalidRemoteSubpath => {
                formatter.write_str(&crate::fl!("ui-sync-invalid-remote-subpath"))
            }
            Self::InvalidOneDriveIdentity => {
                formatter.write_str(&crate::fl!("onedrive-wrong-drive"))
            }
            Self::InvalidSharePointIdentity => {
                formatter.write_str(&crate::fl!("ui-sync-invalid-sharepoint-library-identity"))
            }
            Self::InvalidWorkDirectory => {
                formatter.write_str(&crate::fl!("ui-sync-invalid-work-directory"))
            }
            Self::InvalidRecoveryDirectory => {
                formatter.write_str(&crate::fl!("ui-sync-invalid-recovery-directory"))
            }
            Self::InitialPreviewRequired => formatter.write_str(&crate::fl!(
                "ui-sync-initial-synchronization-requires-a-preview"
            )),
            Self::ExplicitConfirmationRequired => formatter.write_str(&crate::fl!(
                "ui-sync-synchronization-requires-explicit-confirmation"
            )),
            Self::ResyncPreviewRequired => formatter.write_str(&crate::fl!(
                "ui-sync-state-rebuild-or-resync-requires-preview-and"
            )),
            Self::MeteredNetworkPaused => formatter.write_str(&crate::fl!(
                "ui-sync-automatic-synchronization-is-paused-on-metered-networks"
            )),
            Self::ConcurrentRun => {
                formatter.write_str(&crate::fl!("ui-sync-a-synchronization-is-already-running"))
            }
            Self::OverlapsOnedriverPath(path) => formatter.write_str(&crate::fl!(
                "ui-sync-onedrive-mirror-overlaps-active-onedriver-path-value1",
                value1 = (path.display()).to_string()
            )),
            Self::Command(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<CommandError> for SyncError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskEstimate {
    pub remote_bytes: u64,
    pub local_available_bytes: u64,
    pub required_bytes: u64,
}

impl DiskEstimate {
    #[must_use]
    pub const fn sufficient(&self) -> bool {
        self.local_available_bytes >= self.required_bytes
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewSummary {
    pub uploads: u64,
    pub downloads: u64,
    pub deletes: u64,
    pub conflicts: u64,
    pub skipped: u64,
    pub transfer_bytes: Option<u64>,
    pub google_native_skips: Vec<PathBuf>,
    pub destructive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTrigger {
    Initial,
    Manual,
    ConnectivityRestored,
    Scheduled,
    ResyncOrStateRebuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncRequest {
    pub trigger: SyncTrigger,
    pub preview_completed: bool,
    pub user_confirmed: bool,
    pub metered_network: bool,
    pub running: bool,
    pub readiness: SyncReadiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncReadiness {
    pub network_ready: bool,
    pub vpn_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDecision {
    Run,
    PauseMetered,
    WaitForNetwork,
    WaitForVpn,
    Reject(SyncDecisionRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDecisionRejection {
    ConcurrentRun,
    PreviewRequired,
    ConfirmationRequired,
    ResyncPreviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcloneBisyncPlan {
    pub connection_id: ConnectionId,
    pub path1_remote: String,
    pub path2_local: PathBuf,
    pub work_directory: PathBuf,
    pub recovery_directory: PathBuf,
    pub remote_recovery_path: String,
    pub filters_file: PathBuf,
    pub preflight_command: Option<ServiceCommand>,
    pub verify_after_sync: bool,
    pub access_marker_name: Option<String>,
    pub service: ServiceSpec,
    pub timer: TimerSpec,
}

pub fn rclone_bisync_plan(
    connection: &Connection,
    work_root: &Path,
) -> Result<RcloneBisyncPlan, SyncError> {
    connection
        .ensure_supported_engine()
        .map_err(SyncError::UnsupportedEngine)?;
    match connection.provider {
        Provider::OneDrive if connection.offline_engine() == Some(OfflineEngine::Rclone) => {
            crate::onedrive::MirrorGuardBinding::from_connection(connection)
                .map_err(|_| SyncError::InvalidOneDriveIdentity)?;
        }
        Provider::Teams
        | Provider::GoogleDrive
        | Provider::Box
        | Provider::Smb
        | Provider::Sftp => {}
        provider => return Err(SyncError::UnsupportedProvider(provider)),
    }
    rclone_bisync_plan_inner(connection, work_root)
}

// Shared planner for production providers and the explicitly gated development
// prototype. Production entry points still reject legacy OneDrive connections.
fn rclone_bisync_plan_inner(
    connection: &Connection,
    work_root: &Path,
) -> Result<RcloneBisyncPlan, SyncError> {
    let options = offline_options(connection)?;
    if connection.provider == Provider::Teams {
        let identity = connection
            .teams_identity
            .as_ref()
            .ok_or(SyncError::InvalidSharePointIdentity)?;
        if identity.drive_id.trim().is_empty()
            || crate::sharepoint::parse_library_url(&identity.library_url)
                .map_or(true, |parsed| parsed.site_url != identity.site_url)
        {
            return Err(SyncError::InvalidSharePointIdentity);
        }
    }
    validate_directory_roots(
        &connection.local_path,
        work_root,
        &options.recovery_directory,
    )?;
    let remote = if connection.provider == Provider::Sftp {
        crate::providers::sftp_remote_path(
            &connection.remote_reference,
            connection.remote_subpath.as_deref(),
        )
        .map_err(|_| SyncError::InvalidRemoteSubpath)?
    } else {
        rclone_remote_path(
            &connection.remote_reference,
            connection.remote_subpath.as_deref(),
        )?
    };
    let mut work_directory = work_root
        .join("rclone-bisync")
        .join(connection.id.to_string());
    if connection.provider == Provider::Teams {
        let binding = crate::sharepoint::MirrorGuardBinding::from_connection(connection)
            .map_err(|_| SyncError::InvalidSharePointIdentity)?;
        let encoded =
            serde_json::to_vec(&binding).map_err(|_| SyncError::InvalidSharePointIdentity)?;
        work_directory = work_directory.join(format!("target-{:x}", Sha256::digest(encoded)));
    }
    if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(OfflineEngine::Rclone)
    {
        let binding = crate::onedrive::MirrorGuardBinding::from_connection(connection)
            .map_err(|_| SyncError::InvalidOneDriveIdentity)?;
        let encoded =
            serde_json::to_vec(&binding).map_err(|_| SyncError::InvalidOneDriveIdentity)?;
        work_directory = work_directory.join(format!("target-{:x}", Sha256::digest(encoded)));
    }
    let remote_recovery_path = rclone_remote_recovery_path(
        connection.provider,
        &connection.remote_reference,
        connection.remote_subpath.as_deref(),
        connection.id,
    )?;
    let filters_file = work_directory.join("filters.txt");
    let mut arguments = vec![
        "bisync".to_owned(),
        remote.clone(),
        connection.local_path.display().to_string(),
        "--workdir".to_owned(),
        work_directory.display().to_string(),
        "--backup-dir1".to_owned(),
        remote_recovery_path.clone(),
        "--backup-dir2".to_owned(),
        options.recovery_directory.display().to_string(),
        "--resilient".to_owned(),
        "--recover".to_owned(),
        "--max-lock".to_owned(),
        "2m".to_owned(),
        "--conflict-resolve".to_owned(),
        "none".to_owned(),
        "--conflict-loser".to_owned(),
        "pathname".to_owned(),
        "--conflict-suffix".to_owned(),
        "conflict".to_owned(),
        "--create-empty-src-dirs".to_owned(),
        "--compare".to_owned(),
        if connection.provider == Provider::Teams {
            "modtime,checksum".to_owned()
        } else {
            "size,modtime".to_owned()
        },
        "--max-delete".to_owned(),
        BISYNC_MAX_DELETE_PERCENT.to_string(),
    ];
    if connection.provider == Provider::Teams {
        // SharePoint may rewrite Office files after upload. Bisync still tracks
        // each side's changes by checksum; copy-time size/hash checks must not
        // reject the rewritten upload. A one-second window covers SharePoint's
        // timestamp precision. The final equal-listing check cannot compare
        // the original upload with SharePoint's rewritten bytes.
        arguments.extend(
            [
                "--modify-window",
                "1s",
                "--ignore-size",
                "--ignore-checksum",
                "--check-sync",
                "false",
            ]
            .map(str::to_owned),
        );
    }
    arguments.extend([
        "--filters-file".to_owned(),
        filters_file.display().to_string(),
        "--log-level".to_owned(),
        "INFO".to_owned(),
    ]);
    let preflight_command = if connection.provider == Provider::Teams {
        Some(
            crate::sharepoint::mirror_guard_command(connection)
                .map_err(|_| SyncError::InvalidSharePointIdentity)?,
        )
    } else if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(OfflineEngine::Rclone)
    {
        Some(
            crate::onedrive::mirror_guard_command(connection)
                .map_err(|_| SyncError::InvalidOneDriveIdentity)?,
        )
    } else {
        None
    };
    Ok(RcloneBisyncPlan {
        connection_id: connection.id,
        path1_remote: remote,
        path2_local: connection.local_path.clone(),
        work_directory,
        recovery_directory: options.recovery_directory.clone(),
        remote_recovery_path,
        filters_file,
        preflight_command,
        verify_after_sync: connection.provider == Provider::Teams,
        access_marker_name: match connection.provider {
            Provider::Teams => Some(format!(
                ".cloud-mounter-sharepoint-anchor-{}",
                connection.id
            )),
            Provider::OneDrive => Some(format!(".cloud-mounter-onedrive-anchor-{}", connection.id)),
            _ => None,
        },
        service: ServiceSpec {
            connection_id: connection.id,
            description: crate::fl!(
                "ui-sync-cloud-mounter-sync-value1",
                value1 = (connection.name).to_string()
            ),
            executable: PathBuf::from(DEFAULT_RCLONE),
            arguments,
            pre_start_condition: Some(
                crate::vpn::offline_readiness_guard_command(connection.id)
                    .map_err(|_| SyncError::InvalidRemoteReference)?,
            ),
            restart_on_failure: false,
            skip_on_metered: !options.sync_on_metered,
        },
        timer: TimerSpec {
            connection_id: connection.id,
            description: crate::fl!(
                "ui-sync-cloud-mounter-sync-schedule-value1",
                value1 = (connection.name).to_string()
            ),
            interval: Duration::from_secs(u64::from(options.sync_interval_minutes) * 60),
            persistent: true,
        },
    })
}

#[cfg(any(test, feature = "development-probes"))]
pub(crate) fn onedrive_prototype_plan(
    connection: &Connection,
    work_root: &Path,
    remote_recovery: String,
) -> Result<RcloneBisyncPlan, SyncError> {
    if connection.provider != Provider::OneDrive {
        return Err(SyncError::UnsupportedProvider(connection.provider));
    }
    let mut plan = rclone_bisync_plan_inner(connection, work_root)?;
    let backup = plan
        .service
        .arguments
        .iter()
        .position(|a| a == "--backup-dir1")
        .ok_or(SyncError::InvalidRecoveryDirectory)?;
    plan.service.arguments[backup + 1] = remote_recovery.clone();
    plan.remote_recovery_path = remote_recovery;
    plan.access_marker_name = Some(format!(".cloud-mounter-onedrive-anchor-{}", connection.id));
    // Full verification is explicit/initial-only; no SharePoint convergence.
    plan.verify_after_sync = false;
    Ok(plan)
}

#[cfg(test)]
pub fn rclone_bisync_preview_request(
    plan: &RcloneBisyncPlan,
) -> Result<CommandRequest, CommandError> {
    plan.service
        .arguments
        .iter()
        .try_fold(
            CommandRequest::new(Executable::Rclone),
            |request, argument| request.arg(argument),
        )?
        .arg("--dry-run")?
        .with_timeout(Duration::from_secs(120))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        })
        .pipe(Ok)
}

#[cfg(test)]
pub fn rclone_bisync_initial_preview_request(
    plan: &RcloneBisyncPlan,
) -> Result<CommandRequest, CommandError> {
    plan.service
        .arguments
        .iter()
        .try_fold(
            CommandRequest::new(Executable::Rclone),
            |request, argument| request.arg(argument),
        )?
        .arg("--resync")?
        .arg("--dry-run")?
        .with_timeout(Duration::from_secs(120))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        })
        .pipe(Ok)
}

#[cfg(test)]
pub fn rclone_bisync_initial_sync_request(
    plan: &RcloneBisyncPlan,
) -> Result<CommandRequest, CommandError> {
    plan.service
        .arguments
        .iter()
        .try_fold(
            CommandRequest::new(Executable::Rclone),
            |request, argument| request.arg(argument),
        )?
        .arg("--resync")?
        .with_timeout(Duration::from_secs(15 * 60))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        })
        .pipe(Ok)
}

#[cfg(test)]
pub fn rclone_bisync_sync_request(plan: &RcloneBisyncPlan) -> Result<CommandRequest, CommandError> {
    plan.service
        .arguments
        .iter()
        .try_fold(
            CommandRequest::new(Executable::Rclone),
            |request, argument| request.arg(argument),
        )?
        .with_timeout(Duration::from_secs(15 * 60))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        })
        .pipe(Ok)
}

pub fn sync_now_request(
    request: SyncRequest,
    options: &OfflineMirrorConfig,
) -> Result<SyncDecision, SyncError> {
    if request.running {
        return Ok(SyncDecision::Reject(SyncDecisionRejection::ConcurrentRun));
    }
    if !request.readiness.network_ready {
        return Ok(SyncDecision::WaitForNetwork);
    }
    if !request.readiness.vpn_ready {
        return Ok(SyncDecision::WaitForVpn);
    }
    if request.metered_network && !options.sync_on_metered && request.trigger != SyncTrigger::Manual
    {
        return Ok(SyncDecision::PauseMetered);
    }
    match request.trigger {
        SyncTrigger::Initial if !request.preview_completed => {
            Ok(SyncDecision::Reject(SyncDecisionRejection::PreviewRequired))
        }
        SyncTrigger::Initial if !request.user_confirmed => Ok(SyncDecision::Reject(
            SyncDecisionRejection::ConfirmationRequired,
        )),
        SyncTrigger::ResyncOrStateRebuild
            if !request.preview_completed || !request.user_confirmed =>
        {
            Ok(SyncDecision::Reject(
                SyncDecisionRejection::ResyncPreviewRequired,
            ))
        }
        _ => Ok(SyncDecision::Run),
    }
}

#[must_use]
pub fn offline_status(
    paused: bool,
    request: SyncRequest,
    conflicts: &[crate::model::ConflictRecord],
) -> OfflineMirrorStatus {
    if !conflicts.is_empty() {
        return OfflineMirrorStatus::Conflict;
    }
    if paused {
        return OfflineMirrorStatus::Paused;
    }
    if request.running {
        return OfflineMirrorStatus::Syncing;
    }
    if !request.readiness.network_ready {
        return OfflineMirrorStatus::Offline;
    }
    if !request.readiness.vpn_ready {
        return OfflineMirrorStatus::WaitingForVpn;
    }
    if request.metered_network {
        return OfflineMirrorStatus::MeteredPaused;
    }
    OfflineMirrorStatus::Idle
}

#[must_use]
pub const fn recovery_record(
    connection_id: ConnectionId,
    original_relative_path: PathBuf,
    recovery_path: PathBuf,
    reason: RecoveryReason,
    now_unix_seconds: i64,
) -> RecoveryRecord {
    RecoveryRecord {
        connection_id,
        original_relative_path,
        recovery_path,
        reason,
        retained_until_unix_seconds: now_unix_seconds + RETENTION_SECONDS,
    }
}

#[must_use]
pub fn expired_recovery_records(
    records: &[RecoveryRecord],
    now_unix_seconds: i64,
    sync_running: bool,
) -> Vec<RecoveryRecord> {
    if sync_running {
        return Vec::new();
    }
    records
        .iter()
        .filter(|record| record.retained_until_unix_seconds <= now_unix_seconds)
        .cloned()
        .collect()
}

#[must_use]
pub fn google_native_filter_file() -> String {
    [
        "# Cloud Mounter: Google cloud-native documents stay browser-accessible.",
        "- **.gdoc",
        "- **.gsheet",
        "- **.gslides",
        "- **.gdraw",
        "- **.gform",
        "- **.gmap",
        "- **.gsite",
        "+ **",
    ]
    .join("\n")
        + "\n"
}

#[must_use]
pub fn rclone_bisync_filter_file(provider: Provider) -> String {
    match provider {
        Provider::GoogleDrive => google_native_filter_file(),
        Provider::OneDrive => "# OneDrive recovery stays outside the synchronized set.\n- /.cosmic-mounter-recovery/\n- **.one\n- **.onepkg\n- **.lnk\n- **.url\n+ **\n".into(),
        Provider::Teams => "# Keep SharePoint recovery outside the synchronized set.\n- /.cosmic-mounter-recovery/\n+ **\n".into(),
        _ => "+ **\n".into(),
    }
}

/// Root mirrors contain the app's remote recovery directory. Exclude it before
/// the catch-all include; retain existing subfolder filters byte-for-byte so
/// initialized mirrors do not acquire an unrelated bisync filter-hash change.
#[must_use]
pub fn rclone_bisync_filter_file_for_connection(connection: &Connection) -> String {
    let filters = rclone_bisync_filter_file(connection.provider);
    let root = connection.remote_subpath.as_deref().is_none_or(|path| {
        Path::new(path)
            .components()
            .all(|component| component == Component::CurDir)
    });
    if root && matches!(connection.provider, Provider::Box | Provider::GoogleDrive) {
        format!("- /.cosmic-mounter-recovery/\n{filters}")
    } else {
        filters
    }
}

#[must_use]
pub fn parse_preview(output: &str) -> PreviewSummary {
    let mut summary = PreviewSummary::default();
    let mut rclone_copy_direction = None;
    let mut queued_copies = 0_u64;
    let mut queued_deletes = 0_u64;
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains(" changes:")
            || lower.contains("skipped move to")
            || lower.contains("file was deleted")
            || lower.contains("deleted:")
        {
            // Bisync's change totals include zero-valued "deleted" fields.
            // Detection and totals repeat the queued action. A dry-run backup
            // move is part of an overwrite, not a skipped file.
            continue;
        }
        if lower.contains("queue delete") {
            summary.deletes += 1;
            summary.destructive = true;
            queued_deletes += 1;
            continue;
        }
        if lower.contains("skipped delete as --dry-run is set") {
            if queued_deletes > 0 {
                queued_deletes -= 1;
            } else {
                summary.deletes += 1;
                summary.destructive = true;
            }
            continue;
        }
        if lower.contains("queue copy to") && lower.contains("path1") {
            summary.uploads += 1;
            queued_copies += 1;
            continue;
        }
        if lower.contains("queue copy to") && lower.contains("path2") {
            summary.downloads += 1;
            queued_copies += 1;
            continue;
        }
        if (lower.contains("copying files to") || lower.contains("copying path"))
            && let (Some(path1), Some(path2)) = (lower.find("path1"), lower.find("path2"))
        {
            rclone_copy_direction = Some(path2 < path1);
            continue;
        }
        if lower.contains("skipped copy as --dry-run is set") {
            if queued_copies > 0 {
                queued_copies -= 1;
            } else {
                match rclone_copy_direction {
                    Some(true) => summary.uploads += 1,
                    Some(false) => summary.downloads += 1,
                    None => {}
                }
            }
            continue;
        }
        if lower.contains("skipped storing filters file hash") {
            continue;
        }
        if is_google_native_document(&lower) {
            summary.skipped += 1;
            summary.google_native_skips.push(PathBuf::from(line.trim()));
            continue;
        }
        if lower.contains("conflict") {
            summary.conflicts += 1;
        }
        if lower.contains("skip") || lower.contains("excluded") || lower.contains("filtered") {
            summary.skipped += 1;
        }
        if lower.contains("would delete")
            || lower.contains("would remove")
            || lower.contains(": deleting ")
            || lower.contains(": removing ")
            || lower.trim_start().starts_with("delete ")
            || lower.trim_start().starts_with("remove ")
        {
            summary.deletes += 1;
            summary.destructive = true;
        }
        if lower.contains("path2 to path1") && lower.contains("copy") {
            summary.uploads += 1;
        } else if lower.contains("path1 to path2") && lower.contains("copy") {
            summary.downloads += 1;
        } else if lower.contains("would upload") || lower.contains(": uploading ") {
            summary.uploads += 1;
        } else if lower.contains("would download") || lower.contains(": downloading ") {
            summary.downloads += 1;
        }
        if let Some(bytes) = parse_first_bytes(&lower) {
            summary.transfer_bytes =
                Some(summary.transfer_bytes.unwrap_or(0).saturating_add(bytes));
        }
    }
    summary
}

fn offline_options(connection: &Connection) -> Result<&OfflineMirrorConfig, SyncError> {
    match &connection.mode {
        ConnectionMode::OfflineMirror(options) => Ok(options),
        ConnectionMode::OnlineMount(_) => Err(SyncError::InvalidMode),
    }
}

fn validate_directory_roots(
    mirror: &Path,
    work_root: &Path,
    recovery: &Path,
) -> Result<(), SyncError> {
    if !work_root.is_absolute() || paths_overlap(mirror, work_root) {
        return Err(SyncError::InvalidWorkDirectory);
    }
    if !recovery.is_absolute()
        || paths_overlap(mirror, recovery)
        || paths_overlap(work_root, recovery)
    {
        return Err(SyncError::InvalidRecoveryDirectory);
    }
    Ok(())
}

fn rclone_remote_path(reference: &str, subpath: Option<&str>) -> Result<String, SyncError> {
    validate_remote_name(reference)?;
    match subpath {
        Some(subpath) => {
            validate_remote_subpath(subpath)?;
            Ok(format!("{reference}:{subpath}"))
        }
        None => Ok(format!("{reference}:")),
    }
}

fn rclone_remote_recovery_path(
    provider: Provider,
    reference: &str,
    subpath: Option<&str>,
    connection_id: ConnectionId,
) -> Result<String, SyncError> {
    validate_remote_name(reference)?;
    let recovery_name = format!(".cosmic-mounter-recovery/{connection_id}");
    if matches!(provider, Provider::Teams | Provider::OneDrive) && subpath.is_some() {
        let subpath = subpath.ok_or(SyncError::InvalidRemoteSubpath)?;
        validate_remote_subpath(subpath)?;
        return rclone_remote_path(reference, Some(&format!("{subpath}/{recovery_name}")));
    }
    if provider != Provider::Smb {
        return rclone_remote_path(reference, Some(&recovery_name));
    }

    let Some(subpath) = subpath else {
        return rclone_remote_path(reference, Some(&recovery_name));
    };
    validate_remote_subpath(subpath)?;
    let mut components = subpath
        .split('/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.len() > 1 {
        components.pop();
    }
    let recovery_subpath = if components.is_empty() {
        recovery_name
    } else {
        format!("{}/{}", components.join("/"), recovery_name)
    };
    rclone_remote_path(reference, Some(&recovery_subpath))
}

pub(crate) fn validate_remote_name(value: &str) -> Result<(), SyncError> {
    let valid = !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ' ')
        });
    if valid {
        Ok(())
    } else {
        Err(SyncError::InvalidRemoteReference)
    }
}

pub(crate) fn validate_remote_subpath(value: &str) -> Result<(), SyncError> {
    let path = Path::new(value);
    let valid = !value.trim().is_empty()
        && !path.is_absolute()
        && !value.contains('\\')
        && !value.chars().any(char::is_control)
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir));
    if valid {
        Ok(())
    } else {
        Err(SyncError::InvalidRemoteSubpath)
    }
}

fn paths_overlap(first: &Path, second: &Path) -> bool {
    first == second || first.starts_with(second) || second.starts_with(first)
}

fn is_google_native_document(lowercase_line: &str) -> bool {
    [
        ".gdoc", ".gsheet", ".gslides", ".gdraw", ".gform", ".gmap", ".gsite",
    ]
    .iter()
    .any(|extension| lowercase_line.contains(extension))
}

fn parse_first_bytes(lowercase_line: &str) -> Option<u64> {
    let parts = lowercase_line.split_whitespace().collect::<Vec<_>>();
    parts.windows(2).find_map(|window| {
        let value = window[0].replace(',', "").parse::<u64>().ok()?;
        let multiplier = match window[1] {
            "b" | "byte" | "bytes" => 1,
            "kb" | "kib" => 1024,
            "mb" | "mib" => 1024 * 1024,
            "gb" | "gib" => 1024 * 1024 * 1024,
            _ => return None,
        };
        Some(value.saturating_mul(multiplier))
    })
}

#[cfg(test)]
trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

#[cfg(test)]
impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uuid::Uuid;

    use super::*;
    use crate::model::{
        ConnectionMode, OfflineMirrorConfig, SharePointLibraryIdentity, TuningProfile,
    };
    use crate::services::UnitDocument;

    fn id() -> ConnectionId {
        ConnectionId::from_uuid(
            Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
        )
    }

    fn offline(provider: Provider) -> Connection {
        Connection {
            id: id(),
            name: "Engineering Mirror".into(),
            provider,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                engine: None,
                recovery_directory: PathBuf::from(
                    "/home/example/.local/share/cosmic-mounter/recovery/eng",
                ),
                sync_interval_minutes: 15,
                sync_on_metered: false,
            }),
            remote_reference: match provider {
                Provider::OneDrive => "unused",
                Provider::GoogleDrive => "ua_gdrive",
                Provider::Box => "ua_box",
                Provider::Smb => "ua_engr",
                Provider::Sftp => "test_sftp",
                Provider::Teams => "test_teams",
            }
            .into(),
            remote_subpath: Some("Projects".into()),
            local_path: PathBuf::from("/home/example/Cloud/Engineering"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            onedrive_identity: None,
            teams_identity: None,
        }
    }

    #[test]
    fn root_recovery_filters_exclude_backups_without_changing_subfolder_filters() {
        for provider in [Provider::Box, Provider::GoogleDrive] {
            let mut connection = offline(provider);
            let original = rclone_bisync_filter_file(provider);
            assert_eq!(
                rclone_bisync_filter_file_for_connection(&connection),
                original
            );
            for subpath in [None, Some(".".to_owned()), Some("./".to_owned())] {
                connection.remote_subpath = subpath;
                let filters = rclone_bisync_filter_file_for_connection(&connection);
                assert!(filters.starts_with("- /.cosmic-mounter-recovery/\n"));
                assert!(filters.ends_with(&original));
                assert!(
                    filters.find("- /.cosmic-mounter-recovery/").unwrap()
                        < filters.find("+ **").unwrap()
                );
            }
            connection.remote_subpath = Some("Projects".into());
            assert_eq!(
                rclone_bisync_filter_file_for_connection(&connection),
                original
            );
        }
        for provider in [
            Provider::OneDrive,
            Provider::Teams,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let mut connection = offline(provider);
            connection.remote_subpath = None;
            assert_eq!(
                rclone_bisync_filter_file_for_connection(&connection),
                rclone_bisync_filter_file(provider)
            );
        }
    }

    #[test]
    fn rclone_bisync_plan_preserves_conflicts_recovery_and_schedule() {
        let plan = rclone_bisync_plan(
            &offline(Provider::GoogleDrive),
            Path::new("/home/example/.local/state/cosmic-mounter"),
        )
        .expect("plan");
        assert_eq!(plan.path1_remote, "ua_gdrive:Projects");
        assert_eq!(plan.timer.interval, Duration::from_secs(900));
        assert!(plan.service.skip_on_metered);
        assert!(
            !plan
                .service
                .arguments
                .contains(&"--check-access".to_owned())
        );
        assert!(plan.service.arguments.contains(&"--resilient".to_owned()));
        assert!(plan.service.arguments.contains(&"--recover".to_owned()));
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--max-lock", "2m"])
        );
        let mut allowed = offline(Provider::Sftp);
        if let ConnectionMode::OfflineMirror(options) = &mut allowed.mode {
            options.sync_on_metered = true;
        }
        assert!(
            !rclone_bisync_plan(&allowed, Path::new("/state"))
                .unwrap()
                .service
                .skip_on_metered
        );
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--conflict-loser", "pathname"])
        );
        assert!(plan.service.arguments.windows(2).any(|pair| pair
            == [
                "--backup-dir2",
                "/home/example/.local/share/cosmic-mounter/recovery/eng"
            ]));
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--max-delete", "50"])
        );
    }

    #[test]
    fn bisync_deletion_limit_is_shared_without_disabling_recovery() {
        for provider in [
            Provider::Teams,
            Provider::OneDrive,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let mut connection = offline(provider);
            if provider == Provider::Teams {
                connection.teams_identity = Some(crate::model::SharePointLibraryIdentity {
                    site_url: "https://example.sharepoint.com/sites/test".into(),
                    library_url: "https://example.sharepoint.com/sites/test/Documents".into(),
                    drive_id: "b!synthetic-drive".into(),
                });
            }
            let plan = if provider == Provider::OneDrive {
                onedrive_prototype_plan(
                    &connection,
                    Path::new("/state"),
                    "unused:disposable-recovery".into(),
                )
                .unwrap()
            } else {
                rclone_bisync_plan(&connection, Path::new("/state")).unwrap()
            };
            let args = &plan.service.arguments;
            assert_eq!(args.iter().filter(|arg| *arg == "--max-delete").count(), 1);
            assert!(
                args.windows(2).any(|pair| pair == ["--max-delete", "50"]),
                "{provider:?}"
            );
            assert!(!args.iter().any(|arg| arg == "--force" || arg == "--resync"));
            for (flag, path) in [
                ("--backup-dir1", plan.remote_recovery_path),
                (
                    "--backup-dir2",
                    plan.recovery_directory.display().to_string(),
                ),
            ] {
                assert!(
                    args.windows(2).any(|pair| pair == [flag, &path]),
                    "{provider:?}"
                );
            }
        }
    }

    #[test]
    fn every_offline_provider_service_requires_saved_connection_readiness() {
        for provider in [
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let mut connection = offline(provider);
            if provider == Provider::Teams {
                connection.teams_identity = Some(SharePointLibraryIdentity {
                    site_url: "https://example.sharepoint.com/sites/Engineering".into(),
                    library_url:
                        "https://example.sharepoint.com/sites/Engineering/Shared%20Documents".into(),
                    drive_id: "b!test-drive".into(),
                });
            }
            let plan = rclone_bisync_plan(&connection, Path::new("/state")).expect("plan");
            assert_eq!(
                plan.service
                    .pre_start_condition
                    .as_ref()
                    .expect("offline readiness condition")
                    .arguments,
                vec![
                    crate::vpn::OFFLINE_READINESS_GUARD_FLAG.to_owned(),
                    connection.id.to_string(),
                ],
                "{provider:?}"
            );
        }
    }

    #[test]
    fn sftp_mirror_preserves_absolute_path_and_recovery_stays_outside_subtree() {
        let mut connection = offline(Provider::Sftp);
        connection.remote_subpath = Some("/srv/files".into());
        let plan = rclone_bisync_plan(&connection, Path::new("/state")).unwrap();
        assert_eq!(plan.path1_remote, "test_sftp:/srv/files");
        assert!(
            plan.remote_recovery_path
                .starts_with("test_sftp:.cosmic-mounter-recovery/")
        );
        connection.remote_subpath = Some("/srv/../etc".into());
        assert!(rclone_bisync_plan(&connection, Path::new("/state")).is_err());
    }

    #[test]
    fn rclone_preview_request_is_dry_run_and_bounded() {
        let plan = rclone_bisync_plan(
            &offline(Provider::Box),
            Path::new("/home/example/.local/state/cosmic-mounter"),
        )
        .expect("plan");
        let request = rclone_bisync_preview_request(&plan).expect("request");
        assert!(request.sanitized_command().contains(" --dry-run"));
        assert!(!request.sanitized_command().contains(" --resync"));
        assert_eq!(request.timeout, Duration::from_secs(120));
        let initial_preview =
            rclone_bisync_initial_preview_request(&plan).expect("initial preview");
        assert!(
            initial_preview
                .sanitized_command()
                .contains(" --resync --dry-run")
        );
        let initial_sync = rclone_bisync_initial_sync_request(&plan).expect("initial sync");
        assert!(initial_sync.sanitized_command().contains(" --resync"));
        assert!(!initial_sync.sanitized_command().contains(" --dry-run"));
        let sync = rclone_bisync_sync_request(&plan).expect("sync");
        assert!(!sync.sanitized_command().contains(" --dry-run"));
        assert!(!sync.sanitized_command().contains(" --resync"));
        assert_eq!(sync.timeout, Duration::from_secs(15 * 60));
    }

    #[test]
    fn rclone_bisync_unit_contains_no_shell_or_secret() {
        let plan = rclone_bisync_plan(
            &offline(Provider::Smb),
            Path::new("/home/example/.local/state/cosmic-mounter"),
        )
        .expect("plan");
        let document = UnitDocument::service(&plan.service).expect("service");
        assert!(
            document
                .content
                .contains("ExecStart=\"/usr/bin/rclone\" \"bisync\"")
        );
        assert!(!document.content.contains("password"));
        assert!(!document.content.contains("token"));
        assert!(!document.content.contains("sh -c"));
    }

    #[test]
    fn smb_bisync_recovery_stays_inside_selected_share_but_outside_subtree() {
        let mut connection = offline(Provider::Smb);
        connection.remote_subpath = Some("Research/Utzinger/Projects".into());
        let plan = rclone_bisync_plan(
            &connection,
            Path::new("/home/example/.local/state/cosmic-mounter"),
        )
        .expect("plan");

        assert_eq!(plan.path1_remote, "ua_engr:Research/Utzinger/Projects");
        assert_eq!(
            plan.remote_recovery_path,
            format!(
                "ua_engr:Research/Utzinger/.cosmic-mounter-recovery/{}",
                id()
            )
        );
        assert!(!plan.remote_recovery_path.contains("Projects/.cosmic"));
        assert!(plan.service.arguments.windows(2).any(|pair| pair
            == [
                "--backup-dir1",
                &format!(
                    "ua_engr:Research/Utzinger/.cosmic-mounter-recovery/{}",
                    id()
                )
            ]));
    }

    #[test]
    fn teams_bisync_recovery_stays_inside_selected_folder_and_is_excluded() {
        let mut connection = offline(Provider::Teams);
        connection.remote_subpath = Some("CloudMounter-TeamsMirror-Test-20261002".into());
        connection.teams_identity = Some(SharePointLibraryIdentity {
            site_url: "https://example.sharepoint.com/sites/assessment".into(),
            library_url: "https://example.sharepoint.com/sites/assessment/Documents".into(),
            drive_id: "b!verified-drive".into(),
        });
        let plan = rclone_bisync_plan(&connection, Path::new("/state")).unwrap();
        let arguments = &plan.service.arguments;
        for expected in [
            ["--compare", "modtime,checksum"],
            ["--modify-window", "1s"],
            ["--check-sync", "false"],
        ] {
            assert!(arguments.windows(2).any(|pair| pair == expected));
        }
        assert!(arguments.contains(&"--ignore-size".to_owned()));
        assert!(arguments.contains(&"--ignore-checksum".to_owned()));
        assert!(plan.verify_after_sync);
        assert!(plan.preflight_command.is_some());
        assert_eq!(
            plan.remote_recovery_path,
            format!(
                "test_teams:CloudMounter-TeamsMirror-Test-20261002/.cosmic-mounter-recovery/{}",
                id()
            )
        );
        assert!(
            rclone_bisync_filter_file(Provider::Teams).contains("- /.cosmic-mounter-recovery/\n")
        );
        connection.remote_subpath = None;
        let root_plan = rclone_bisync_plan(&connection, Path::new("/state")).unwrap();
        assert_eq!(root_plan.path1_remote, "test_teams:");
        assert_eq!(root_plan.service.arguments[1], "test_teams:");
        assert_eq!(
            root_plan.remote_recovery_path,
            format!("test_teams:.cosmic-mounter-recovery/{}", id())
        );
        assert_ne!(root_plan.work_directory, plan.work_directory);
        assert_ne!(root_plan.preflight_command, plan.preflight_command);
        assert_eq!(root_plan.access_marker_name, plan.access_marker_name);
        assert!(root_plan.verify_after_sync);
        assert!(root_plan.preflight_command.is_some());
    }

    #[test]
    fn sharepoint_bisync_state_is_bound_to_the_verified_target() {
        let mut connection = offline(Provider::Teams);
        connection.remote_subpath = Some("Disposable".into());
        connection.teams_identity = Some(SharePointLibraryIdentity {
            site_url: "https://example.sharepoint.com/sites/assessment".into(),
            library_url: "https://example.sharepoint.com/sites/assessment/Documents".into(),
            drive_id: "b!verified-drive".into(),
        });
        let root = Path::new("/state");
        let original = rclone_bisync_plan(&connection, root).unwrap();
        assert!(
            original
                .work_directory
                .starts_with(root.join("rclone-bisync"))
        );
        assert_eq!(
            rclone_bisync_plan(&connection, root)
                .unwrap()
                .work_directory,
            original.work_directory
        );

        for changed in [
            "remote", "drive", "library", "folder", "root", "local", "recovery",
        ] {
            let mut edited = connection.clone();
            match changed {
                "remote" => edited.remote_reference = "another_remote".into(),
                "drive" => edited.teams_identity.as_mut().unwrap().drive_id = "b!other".into(),
                "library" => {
                    edited.teams_identity.as_mut().unwrap().library_url =
                        "https://example.sharepoint.com/sites/assessment/Other".into();
                }
                "folder" => edited.remote_subpath = Some("AnotherDisposable".into()),
                "root" => edited.remote_subpath = None,
                "local" => edited.local_path = PathBuf::from("/home/example/Cloud/Other"),
                "recovery" => {
                    let ConnectionMode::OfflineMirror(options) = &mut edited.mode else {
                        unreachable!()
                    };
                    options.recovery_directory = PathBuf::from("/home/example/Recovery/Other");
                }
                _ => unreachable!(),
            }
            let next = rclone_bisync_plan(&edited, root).unwrap();
            assert_ne!(next.work_directory, original.work_directory, "{changed}");
        }
    }

    #[test]
    fn rclone_plan_rejects_overlap_and_unsafe_remote_values() {
        let mut connection = offline(Provider::GoogleDrive);
        connection.remote_subpath = Some("../bad".into());
        assert!(matches!(
            rclone_bisync_plan(
                &connection,
                Path::new("/home/example/.local/state/cosmic-mounter")
            ),
            Err(SyncError::InvalidRemoteSubpath)
        ));
        connection.remote_subpath = Some("Projects".into());
        assert!(matches!(
            rclone_bisync_plan(
                &connection,
                Path::new("/home/example/Cloud/Engineering/work")
            ),
            Err(SyncError::InvalidWorkDirectory)
        ));
    }

    #[test]
    fn sync_decision_requires_preview_confirmation_and_metered_override() {
        let options = OfflineMirrorConfig::default();
        let base = SyncRequest {
            trigger: SyncTrigger::Initial,
            preview_completed: false,
            user_confirmed: false,
            metered_network: false,
            running: false,
            readiness: SyncReadiness {
                network_ready: true,
                vpn_ready: true,
            },
        };
        assert_eq!(
            sync_now_request(base, &options).expect("decision"),
            SyncDecision::Reject(SyncDecisionRejection::PreviewRequired)
        );
        assert_eq!(
            sync_now_request(
                SyncRequest {
                    preview_completed: true,
                    ..base
                },
                &options,
            )
            .expect("decision"),
            SyncDecision::Reject(SyncDecisionRejection::ConfirmationRequired)
        );
        assert_eq!(
            sync_now_request(
                SyncRequest {
                    trigger: SyncTrigger::Scheduled,
                    metered_network: true,
                    preview_completed: true,
                    user_confirmed: true,
                    ..base
                },
                &options,
            )
            .expect("decision"),
            SyncDecision::PauseMetered
        );
        assert_eq!(
            sync_now_request(
                SyncRequest {
                    trigger: SyncTrigger::Manual,
                    metered_network: true,
                    preview_completed: true,
                    user_confirmed: true,
                    ..base
                },
                &options,
            )
            .expect("decision"),
            SyncDecision::Run
        );
    }

    #[test]
    fn resync_requires_preview_and_confirmation() {
        let request = SyncRequest {
            trigger: SyncTrigger::ResyncOrStateRebuild,
            preview_completed: true,
            user_confirmed: false,
            metered_network: false,
            running: false,
            readiness: SyncReadiness {
                network_ready: true,
                vpn_ready: true,
            },
        };
        assert_eq!(
            sync_now_request(request, &OfflineMirrorConfig::default()).expect("decision"),
            SyncDecision::Reject(SyncDecisionRejection::ResyncPreviewRequired)
        );
    }

    #[test]
    fn parse_preview_counts_destructive_conflicts_and_google_skips() {
        let preview = parse_preview(
            "Would copy Path2 to Path1: local.txt 10 MB\n\
Would copy Path1 to Path2: remote.txt 5 MB\n\
Would delete old.txt\n\
Conflict detected: both.txt\n\
Excluded proposal.gdoc\n",
        );
        assert_eq!(preview.uploads, 1);
        assert_eq!(preview.downloads, 1);
        assert_eq!(preview.deletes, 1);
        assert_eq!(preview.conflicts, 1);
        assert_eq!(preview.skipped, 1);
        assert!(preview.destructive);
        assert_eq!(preview.transfer_bytes, Some(15 * 1024 * 1024));
        assert_eq!(preview.google_native_skips.len(), 1);
    }

    #[test]
    fn parse_preview_counts_dry_run_files_without_counting_rclone_progress() {
        let preview = parse_preview(
            "\"DownloadHash\": false\n\
Copying Path2 files to Path1\n\
There was nothing to transfer\n\
Skipped storing filters file hash to filters.txt.md5 as --dry-run is set\n\
Path1 Resync is copying files to Path2\n\
one.txt: Skipped copy as --dry-run is set (size 4)\n\
Transferred: 4 B / 4 B, 100%\n\
Transferred: 1 / 1, 100%\n",
        );
        assert_eq!(preview.uploads, 0);
        assert_eq!(preview.downloads, 1);
        assert_eq!(preview.skipped, 0);
        assert_eq!(preview.transfer_bytes, Some(4));

        let upload = parse_preview(
            "Copying Path2 files to Path1\nlocal.txt: Skipped copy as --dry-run is set (size 3)\n",
        );
        assert_eq!(upload.uploads, 1);
        assert_eq!(upload.downloads, 0);
    }

    #[test]
    fn parse_preview_does_not_report_backup_moves_or_zero_deletions() {
        let preview = parse_preview(
            "Path2: 1 changes: 0 new, 1 modified, 0 deleted\n\
\x1b[32mQueue copy to\x1b[0m Path1 - remote/probe.txt\n\
probe.txt: Skipped move to probe.txt-backup as --dry-run is set (size 48)\n\
probe.txt: Skipped copy as --dry-run is set (size 103)\n\
Transferred: 151 B / 151 B, 100%\n",
        );
        assert_eq!(preview.uploads, 1);
        assert_eq!(preview.downloads, 0);
        assert_eq!(preview.deletes, 0);
        assert_eq!(preview.skipped, 0);
        assert!(!preview.destructive);
        assert_eq!(preview.transfer_bytes, Some(151));
    }

    #[test]
    fn parse_preview_counts_queued_deletion_once() {
        let preview = parse_preview(
            "File was deleted - delete-probe.txt\n\
Path1: 1 changes: 0 new, 0 modified, 1 deleted\n\
\x1b[31mQueue delete\x1b[0m - right/delete-probe.txt\n\
delete-probe.txt: Skipped delete as --dry-run is set (size 13)\n\
Deleted: 1 (files), 0 (dirs), 13 B (freed)\n",
        );
        assert_eq!(preview.deletes, 1);
        assert_eq!(preview.skipped, 0);
        assert!(preview.destructive);
    }

    #[test]
    fn recovery_records_expire_after_thirty_days_but_not_while_running() {
        let record = recovery_record(
            id(),
            PathBuf::from("old.txt"),
            PathBuf::from("/recovery/old.txt"),
            RecoveryReason::Deleted,
            1_000,
        );
        assert_eq!(
            record.retained_until_unix_seconds,
            1_000 + RETENTION_SECONDS
        );
        assert!(
            expired_recovery_records(
                std::slice::from_ref(&record),
                record.retained_until_unix_seconds,
                true,
            )
            .is_empty()
        );
        assert_eq!(
            expired_recovery_records(
                std::slice::from_ref(&record),
                record.retained_until_unix_seconds,
                false,
            ),
            vec![record]
        );
    }

    #[test]
    fn google_native_filter_excludes_browser_only_documents() {
        let filters = google_native_filter_file();
        for extension in [".gdoc", ".gsheet", ".gslides"] {
            assert!(filters.contains(extension));
        }
        assert!(filters.contains("+ **"));
    }

    #[test]
    fn offline_status_maps_common_states() {
        let request = SyncRequest {
            trigger: SyncTrigger::Scheduled,
            preview_completed: true,
            user_confirmed: true,
            metered_network: false,
            running: false,
            readiness: SyncReadiness {
                network_ready: false,
                vpn_ready: true,
            },
        };
        assert_eq!(
            offline_status(false, request, &[]),
            OfflineMirrorStatus::Offline
        );
        assert_eq!(
            offline_status(
                false,
                SyncRequest {
                    running: true,
                    readiness: SyncReadiness {
                        network_ready: true,
                        vpn_ready: true,
                    },
                    ..request
                },
                &[],
            ),
            OfflineMirrorStatus::Syncing
        );
    }

    #[test]
    fn disk_estimate_reports_low_space() {
        let estimate = DiskEstimate {
            remote_bytes: 100,
            local_available_bytes: 50,
            required_bytes: 100,
        };
        assert!(!estimate.sufficient());
    }
}
