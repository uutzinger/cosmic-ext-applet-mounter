// SPDX-License-Identifier: MIT

//! Offline mirror preview, scheduling, conflicts, recovery, and synchronization.

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use crate::model::{
    Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, OfflineMirrorStatus, Provider,
    RecoveryReason, RecoveryRecord,
};
use crate::process::{CommandError, CommandRequest, Executable, RetryPolicy};
use crate::services::{ServiceSpec, TimerSpec};

const DEFAULT_RCLONE: &str = "/usr/bin/rclone";
const DEFAULT_ONEDRIVE: &str = "/usr/bin/onedrive";
const RETENTION_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    InvalidMode,
    UnsupportedProvider(Provider),
    InvalidRemoteReference,
    InvalidRemoteSubpath,
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
            Self::InvalidMode => write!(formatter, "connection is not an offline mirror"),
            Self::UnsupportedProvider(provider) => {
                write!(formatter, "unsupported provider {provider:?}")
            }
            Self::InvalidRemoteReference => write!(formatter, "invalid remote reference"),
            Self::InvalidRemoteSubpath => write!(formatter, "invalid remote subpath"),
            Self::InvalidWorkDirectory => write!(formatter, "invalid work directory"),
            Self::InvalidRecoveryDirectory => write!(formatter, "invalid recovery directory"),
            Self::InitialPreviewRequired => {
                write!(formatter, "initial synchronization requires a preview")
            }
            Self::ExplicitConfirmationRequired => {
                write!(formatter, "synchronization requires explicit confirmation")
            }
            Self::ResyncPreviewRequired => write!(
                formatter,
                "state rebuild or resync requires preview and confirmation"
            ),
            Self::MeteredNetworkPaused => write!(
                formatter,
                "automatic synchronization is paused on metered networks"
            ),
            Self::ConcurrentRun => write!(formatter, "a synchronization is already running"),
            Self::OverlapsOnedriverPath(path) => {
                write!(
                    formatter,
                    "OneDrive mirror overlaps active onedriver path {}",
                    path.display()
                )
            }
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
    pub service: ServiceSpec,
    pub timer: TimerSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneDriveMirrorPlan {
    pub connection_id: ConnectionId,
    pub sync_directory: PathBuf,
    pub config_directory: PathBuf,
    pub recovery_directory: PathBuf,
    pub single_directory: Option<String>,
    pub service: ServiceSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneDriveNativeState {
    Idle,
    Syncing,
    Offline,
    Conflict,
    RecoveryRequired,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OneDriveStatusSnapshot {
    pub authenticated: bool,
    pub monitor_running: bool,
    pub native_state: OneDriveNativeState,
    pub readiness: SyncReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneDriveIsolationReport {
    pub active_onedriver_paths: Vec<PathBuf>,
}

pub fn rclone_bisync_plan(
    connection: &Connection,
    work_root: &Path,
) -> Result<RcloneBisyncPlan, SyncError> {
    match connection.provider {
        Provider::GoogleDrive | Provider::Box | Provider::Smb => {}
        provider => return Err(SyncError::UnsupportedProvider(provider)),
    }
    let options = offline_options(connection)?;
    validate_directory_roots(
        &connection.local_path,
        work_root,
        &options.recovery_directory,
    )?;
    let remote = rclone_remote_path(
        &connection.remote_reference,
        connection.remote_subpath.as_deref(),
    )?;
    let work_directory = work_root
        .join("rclone-bisync")
        .join(connection.id.to_string());
    let remote_recovery_path = rclone_remote_recovery_path(
        connection.provider,
        &connection.remote_reference,
        connection.remote_subpath.as_deref(),
        connection.id,
    )?;
    let filters_file = work_directory.join("filters.txt");
    let arguments = vec![
        "bisync".to_owned(),
        remote.clone(),
        connection.local_path.display().to_string(),
        "--workdir".to_owned(),
        work_directory.display().to_string(),
        "--backup-dir1".to_owned(),
        remote_recovery_path.clone(),
        "--backup-dir2".to_owned(),
        options.recovery_directory.display().to_string(),
        "--check-access".to_owned(),
        "--resilient".to_owned(),
        "--recover".to_owned(),
        "--conflict-resolve".to_owned(),
        "none".to_owned(),
        "--conflict-loser".to_owned(),
        "pathname".to_owned(),
        "--conflict-suffix".to_owned(),
        "conflict".to_owned(),
        "--create-empty-src-dirs".to_owned(),
        "--compare".to_owned(),
        "size,modtime".to_owned(),
        "--max-delete".to_owned(),
        "1000".to_owned(),
        "--filters-file".to_owned(),
        filters_file.display().to_string(),
        "--log-level".to_owned(),
        "INFO".to_owned(),
    ];
    Ok(RcloneBisyncPlan {
        connection_id: connection.id,
        path1_remote: remote,
        path2_local: connection.local_path.clone(),
        work_directory,
        recovery_directory: options.recovery_directory.clone(),
        remote_recovery_path,
        filters_file,
        service: ServiceSpec {
            connection_id: connection.id,
            description: format!("COSMIC Cloud Mounter sync: {}", connection.name),
            executable: PathBuf::from(DEFAULT_RCLONE),
            arguments,
            restart_on_failure: false,
        },
        timer: TimerSpec {
            connection_id: connection.id,
            description: format!("COSMIC Cloud Mounter sync schedule: {}", connection.name),
            interval: Duration::from_secs(u64::from(options.sync_interval_minutes) * 60),
            persistent: true,
        },
    })
}

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

pub fn one_drive_mirror_plan(
    connection: &Connection,
    config_root: &Path,
    isolation: &OneDriveIsolationReport,
) -> Result<OneDriveMirrorPlan, SyncError> {
    if connection.provider != Provider::OneDrive {
        return Err(SyncError::UnsupportedProvider(connection.provider));
    }
    let options = offline_options(connection)?;
    validate_directory_roots(
        &connection.local_path,
        config_root,
        &options.recovery_directory,
    )?;
    if let Some(path) = isolation
        .active_onedriver_paths
        .iter()
        .find(|path| paths_overlap(path, &connection.local_path))
    {
        return Err(SyncError::OverlapsOnedriverPath(path.clone()));
    }
    let config_directory = config_root
        .join("onedrive-sync")
        .join(connection.id.to_string());
    let mut arguments = vec![
        "--confdir".to_owned(),
        config_directory.display().to_string(),
        "--syncdir".to_owned(),
        connection.local_path.display().to_string(),
        "--monitor".to_owned(),
        "--monitor-interval".to_owned(),
        "300".to_owned(),
        "--disable-notifications".to_owned(),
    ];
    if let Some(subpath) = &connection.remote_subpath {
        validate_remote_subpath(subpath)?;
        arguments.push("--single-directory".to_owned());
        arguments.push(subpath.clone());
    }
    Ok(OneDriveMirrorPlan {
        connection_id: connection.id,
        sync_directory: connection.local_path.clone(),
        config_directory,
        recovery_directory: options.recovery_directory.clone(),
        single_directory: connection.remote_subpath.clone(),
        service: ServiceSpec {
            connection_id: connection.id,
            description: format!("COSMIC Cloud Mounter OneDrive mirror: {}", connection.name),
            executable: PathBuf::from(DEFAULT_ONEDRIVE),
            arguments,
            restart_on_failure: true,
        },
    })
}

pub fn one_drive_preview_request(
    plan: &OneDriveMirrorPlan,
) -> Result<CommandRequest, CommandError> {
    let mut request = CommandRequest::new(Executable::OneDrive)
        .arg("--confdir")?
        .arg(plan.config_directory.as_os_str())?
        .arg("--syncdir")?
        .arg(plan.sync_directory.as_os_str())?
        .arg("--sync")?
        .arg("--dry-run")?
        .arg("--verbose")?;
    if let Some(single_directory) = &plan.single_directory {
        request = request.arg("--single-directory")?.arg(single_directory)?;
    }
    Ok(request.with_timeout(Duration::from_secs(120)))
}

pub fn one_drive_auth_request(plan: &OneDriveMirrorPlan) -> Result<CommandRequest, CommandError> {
    CommandRequest::new(Executable::OneDrive)
        .arg("--confdir")?
        .arg(plan.config_directory.as_os_str())?
        .arg("--reauth")?
        .with_timeout(Duration::from_secs(10 * 60))
        .pipe(Ok)
}

pub fn one_drive_auth_files_request(
    plan: &OneDriveMirrorPlan,
    auth_url_file: &Path,
    response_url_file: &Path,
) -> Result<CommandRequest, CommandError> {
    CommandRequest::new(Executable::OneDrive)
        .arg("--confdir")?
        .arg(plan.config_directory.as_os_str())?
        .arg("--reauth")?
        .arg("--auth-files")?
        .arg(format!(
            "{}:{}",
            auth_url_file.display(),
            response_url_file.display()
        ))?
        .with_timeout(Duration::from_secs(10 * 60))
        .pipe(Ok)
}

pub fn one_drive_sync_request(plan: &OneDriveMirrorPlan) -> Result<CommandRequest, CommandError> {
    let mut request = CommandRequest::new(Executable::OneDrive)
        .arg("--confdir")?
        .arg(plan.config_directory.as_os_str())?
        .arg("--syncdir")?
        .arg(plan.sync_directory.as_os_str())?
        .arg("--sync")?;
    if let Some(single_directory) = &plan.single_directory {
        request = request.arg("--single-directory")?.arg(single_directory)?;
    }
    Ok(request
        .with_timeout(Duration::from_secs(15 * 60))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        }))
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
        "# COSMIC Cloud Mounter: Google cloud-native documents stay browser-accessible.",
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
pub fn parse_preview(output: &str) -> PreviewSummary {
    let mut summary = PreviewSummary::default();
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
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
        if lower.contains("delete") || lower.contains("remove") {
            summary.deletes += 1;
            summary.destructive = true;
        }
        if lower.contains("path2 to path1") && lower.contains("copy") {
            summary.uploads += 1;
        } else if lower.contains("path1 to path2") && lower.contains("copy") {
            summary.downloads += 1;
        } else if lower.contains("upload") {
            summary.uploads += 1;
        } else if lower.contains("download") || lower.contains("transfer") || lower.contains("copy")
        {
            summary.downloads += 1;
        }
        if let Some(bytes) = parse_first_bytes(&lower) {
            summary.transfer_bytes =
                Some(summary.transfer_bytes.unwrap_or(0).saturating_add(bytes));
        }
    }
    summary
}

#[must_use]
pub fn onedrive_status(snapshot: OneDriveStatusSnapshot) -> OfflineMirrorStatus {
    if !snapshot.authenticated {
        return OfflineMirrorStatus::Unavailable;
    }
    if !snapshot.readiness.network_ready {
        return OfflineMirrorStatus::Offline;
    }
    if !snapshot.readiness.vpn_ready {
        return OfflineMirrorStatus::WaitingForVpn;
    }
    match snapshot.native_state {
        OneDriveNativeState::Idle if snapshot.monitor_running => OfflineMirrorStatus::Idle,
        OneDriveNativeState::Syncing => OfflineMirrorStatus::Syncing,
        OneDriveNativeState::Offline => OfflineMirrorStatus::Offline,
        OneDriveNativeState::Conflict => OfflineMirrorStatus::Conflict,
        OneDriveNativeState::RecoveryRequired | OneDriveNativeState::Error => {
            OfflineMirrorStatus::Error
        }
        OneDriveNativeState::Idle => OfflineMirrorStatus::Paused,
    }
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

fn validate_remote_name(value: &str) -> Result<(), SyncError> {
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

fn validate_remote_subpath(value: &str) -> Result<(), SyncError> {
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

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use uuid::Uuid;

    use super::*;
    use crate::model::{ConnectionMode, OfflineMirrorConfig, TuningProfile};
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
            }
            .into(),
            remote_subpath: Some("Projects".into()),
            local_path: PathBuf::from("/home/example/Cloud/Engineering"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
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
        assert!(
            plan.service
                .arguments
                .contains(&"--check-access".to_owned())
        );
        assert!(plan.service.arguments.contains(&"--resilient".to_owned()));
        assert!(plan.service.arguments.contains(&"--recover".to_owned()));
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
                .any(|pair| pair == ["--max-delete", "1000"])
        );
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
    fn onedrive_plan_blocks_active_onedriver_overlap() {
        let connection = offline(Provider::OneDrive);
        let isolation = OneDriveIsolationReport {
            active_onedriver_paths: vec![PathBuf::from("/home/example/Cloud")],
        };
        assert!(matches!(
            one_drive_mirror_plan(
                &connection,
                Path::new("/home/example/.config/cosmic-mounter"),
                &isolation,
            ),
            Err(SyncError::OverlapsOnedriverPath(_))
        ));
    }

    #[test]
    fn onedrive_plan_uses_isolated_config_and_monitor_mode() {
        let connection = offline(Provider::OneDrive);
        let plan = one_drive_mirror_plan(
            &connection,
            Path::new("/home/example/.config/cosmic-mounter"),
            &OneDriveIsolationReport {
                active_onedriver_paths: Vec::new(),
            },
        )
        .expect("plan");
        assert_eq!(plan.service.executable, PathBuf::from(DEFAULT_ONEDRIVE));
        assert!(plan.service.arguments.contains(&"--monitor".to_owned()));
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--single-directory", "Projects"])
        );
        assert!(
            plan.config_directory
                .starts_with("/home/example/.config/cosmic-mounter/onedrive-sync")
        );
        assert_eq!(
            plan.sync_directory,
            PathBuf::from("/home/example/Cloud/Engineering")
        );
        assert_eq!(
            one_drive_auth_request(&plan)
                .expect("auth")
                .sanitized_command(),
            "onedrive --confdir /home/example/.config/cosmic-mounter/onedrive-sync/2a3f5d45-e867-47e7-943f-66cf60e777ad --reauth"
        );
        assert_eq!(
            one_drive_auth_files_request(
                &plan,
                Path::new("/tmp/cosmic-onedrive-auth-url"),
                Path::new("/tmp/cosmic-onedrive-response-url"),
            )
            .expect("auth files")
            .sanitized_command(),
            "onedrive --confdir /home/example/.config/cosmic-mounter/onedrive-sync/2a3f5d45-e867-47e7-943f-66cf60e777ad --reauth --auth-files /tmp/cosmic-onedrive-auth-url:/tmp/cosmic-onedrive-response-url"
        );
        assert!(
            one_drive_preview_request(&plan)
                .expect("preview")
                .sanitized_command()
                .contains(" --dry-run")
        );
        assert!(
            one_drive_sync_request(&plan)
                .expect("sync")
                .sanitized_command()
                .contains(" --sync")
        );
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
    fn onedrive_native_status_maps_to_applet_status() {
        let snapshot = OneDriveStatusSnapshot {
            authenticated: true,
            monitor_running: true,
            native_state: OneDriveNativeState::Conflict,
            readiness: SyncReadiness {
                network_ready: true,
                vpn_ready: true,
            },
        };
        assert_eq!(onedrive_status(snapshot), OfflineMirrorStatus::Conflict);
        assert_eq!(
            onedrive_status(OneDriveStatusSnapshot {
                authenticated: false,
                ..snapshot
            }),
            OfflineMirrorStatus::Unavailable
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
