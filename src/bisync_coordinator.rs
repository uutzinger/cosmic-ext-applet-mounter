// SPDX-License-Identifier: MIT

//! Typed rclone bisync transaction shared by the UI and scheduled worker.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::{Config, LoadSource};
use crate::mirror_access::LocalMirrorPaths;
use crate::mirror_script::{live_progress_log_path, live_progress_state_path};
use crate::model::{Connection, ConnectionId, ConnectionMode, Provider};
use crate::offline_coordinator::{
    OfflineCoordinatorState, OfflineOperation, OfflineOperationState, read_state, state_path,
    write_state,
};
use crate::process::{CommandError, CommandOutput, CommandRequest, CommandRunner, Executable};
use crate::sync::{RcloneBisyncPlan, rclone_bisync_filter_file, rclone_bisync_plan};

const OWNER_MARKER: &str = ".cosmic-mounter-owner";
const RETENTION_DAYS: u64 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BisyncMode {
    InitialPreview,
    Preview,
    InitialSync,
    Sync,
    Scheduled,
}

impl BisyncMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitialPreview => "initial-preview",
            Self::Preview => "preview",
            Self::InitialSync => "initial-sync",
            Self::Sync => "sync",
            Self::Scheduled => "scheduled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "initial-preview" => Some(Self::InitialPreview),
            "preview" => Some(Self::Preview),
            "initial-sync" => Some(Self::InitialSync),
            "sync" => Some(Self::Sync),
            "scheduled" => Some(Self::Scheduled),
            _ => None,
        }
    }

    const fn operation(self) -> OfflineOperation {
        match self {
            Self::InitialPreview => OfflineOperation::InitialPreview,
            Self::Preview => OfflineOperation::Preview,
            Self::InitialSync => OfflineOperation::InitialSync,
            Self::Sync => OfflineOperation::Sync,
            Self::Scheduled => OfflineOperation::Scheduled,
        }
    }

    const fn preview(self) -> bool {
        matches!(self, Self::InitialPreview | Self::Preview)
    }

    const fn initial(self) -> bool {
        matches!(self, Self::InitialPreview | Self::InitialSync)
    }
}

#[derive(Debug)]
pub struct BisyncOutcome {
    pub output: CommandOutput,
    pub preview_pending: Option<Result<u64, String>>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Default)]
struct RetentionCleanupIssues {
    local_listing_failed: bool,
    local_entries_unreadable: usize,
    local_ownership_unverified: usize,
    local_deletions_failed: usize,
    remote_listing_failed: bool,
    remote_ownership_unverified: usize,
    remote_deletions_failed: usize,
    cancelled: bool,
}

impl RetentionCleanupIssues {
    fn warnings(self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.cancelled {
            warnings.push(
                crate::fl!("ui-bisync-coordinator-retention-cleanup-was-interrupted-after-synchronization-succeeded"),
            );
        }
        if self.local_listing_failed {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-local-retention-cleanup-could-not-inspect-the"
            ));
        }
        if self.local_entries_unreadable > 0 {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-local-retention-cleanup-could-not-inspect-value1",
                count = self.local_entries_unreadable
            ));
        }
        if self.local_ownership_unverified > 0 {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-local-retention-cleanup-preserved-value1-expired-recovery",
                count = self.local_ownership_unverified
            ));
        }
        if self.local_deletions_failed > 0 {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-local-retention-cleanup-could-not-remove-value1",
                count = self.local_deletions_failed
            ));
        }
        if self.remote_listing_failed {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-remote-retention-cleanup-could-not-list-the"
            ));
        }
        if self.remote_ownership_unverified > 0 {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-remote-retention-cleanup-preserved-value1-expired-recovery",
                count = self.remote_ownership_unverified
            ));
        }
        if self.remote_deletions_failed > 0 {
            warnings.push(crate::fl!(
                "ui-bisync-coordinator-remote-retention-cleanup-could-not-remove-value1",
                count = self.remote_deletions_failed
            ));
        }
        warnings
    }
}

pub fn default_work_root() -> PathBuf {
    work_root_for(
        crate::process::CommandExecutionMode::detect_current(),
        std::env::var_os("HOME").map(PathBuf::from),
        std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
    )
}

fn work_root_for(
    mode: crate::process::CommandExecutionMode,
    home: Option<PathBuf>,
    state: Option<PathBuf>,
) -> PathBuf {
    let host_state = home.map(|home| home.join(".local/state"));
    (if mode == crate::process::CommandExecutionMode::FlatpakSpawnHost {
        host_state
    } else {
        state.or(host_state)
    })
    .unwrap_or_else(std::env::temp_dir)
    .join("cosmic-ext-applet-mounter")
}

pub async fn run_saved<R: CommandRunner>(
    connection_id: ConnectionId,
    mode: BisyncMode,
    work_root: &Path,
    runner: &R,
    cancellation: CancellationToken,
) -> Result<BisyncOutcome, String> {
    let report = Config::load_runtime();
    if report.source != LoadSource::Current {
        return Err(crate::fl!(
            "ui-bisync-coordinator-could-not-load-the-saved-offline-mirror"
        ));
    }
    let connection = report
        .config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .ok_or_else(|| {
            crate::fl!("ui-bisync-coordinator-the-offline-mirror-connection-was-removed")
        })?;
    run(connection, mode, work_root, runner, cancellation).await
}

pub async fn run<R: CommandRunner>(
    connection: &Connection,
    mode: BisyncMode,
    work_root: &Path,
    runner: &R,
    cancellation: CancellationToken,
) -> Result<BisyncOutcome, String> {
    validate_connection(connection)?;
    let plan = rclone_bisync_plan(connection, work_root).map_err(|error| error.to_string())?;
    let graph = if connection.provider == Provider::OneDrive {
        Some(crate::onedrive::MicrosoftGraph::new().map_err(|error| error.to_string())?)
    } else {
        None
    };
    run_plan(
        connection,
        mode,
        work_root,
        &plan,
        runner,
        cancellation,
        graph
            .as_ref()
            .map(|graph| graph as &dyn crate::onedrive::Graph),
    )
    .await
}

// Called only after the prototype's in-process identity/ownership guard. The
// production entry point above keeps its existing provider validation.
#[cfg(any(test, feature = "development-probes"))]
pub(crate) async fn run_onedrive_prototype<R: CommandRunner>(
    connection: &Connection,
    mode: BisyncMode,
    work_root: &Path,
    plan: &RcloneBisyncPlan,
    runner: &R,
    cancellation: CancellationToken,
) -> Result<BisyncOutcome, String> {
    if connection.provider != Provider::OneDrive || plan.connection_id != connection.id {
        return Err(crate::fl!(
            "ui-bisync-coordinator-the-saved-connection-is-not-an-rclone"
        ));
    }
    run_plan(
        connection,
        mode,
        work_root,
        plan,
        runner,
        cancellation,
        None,
    )
    .await
}

async fn run_plan<R: CommandRunner>(
    connection: &Connection,
    mode: BisyncMode,
    work_root: &Path,
    plan: &RcloneBisyncPlan,
    runner: &R,
    cancellation: CancellationToken,
    graph: Option<&dyn crate::onedrive::Graph>,
) -> Result<BisyncOutcome, String> {
    // Serialize before publishing filters, anchors, markers, or result state.
    // This makes the lock the first coordinator-owned mutation and prevents a
    // timer and a manual request from racing during input preparation.
    let _lock = acquire_operation_guard(work_root, connection.id)?;
    validate_mode_markers(mode, plan)?;
    if connection.provider == Provider::Teams {
        let binding = crate::sharepoint::MirrorGuardBinding::from_connection(connection)?;
        crate::sharepoint::verify_mirror_binding(&binding, connection, runner).await?;
    }
    if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone)
    {
        let binding = crate::onedrive::MirrorGuardBinding::from_connection(connection)
            .map_err(|error| error.to_string())?;
        crate::onedrive::verify_mirror_binding(
            &binding,
            connection,
            runner,
            graph.ok_or_else(|| crate::onedrive::OneDriveError::InvalidState.to_string())?,
            cancellation.child_token(),
        )
        .await
        .map_err(|error| error.to_string())?;
    }
    let local_paths = LocalMirrorPaths::resolve(connection, cancellation.child_token()).await?;
    let mut local_connection = connection.clone();
    local_connection.local_path = local_paths.mirror.clone();
    let mut local_plan = plan.clone();
    local_plan.recovery_directory = local_paths.recovery.clone();
    prepare_inputs(&local_connection, &local_plan, mode)?;
    if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone)
    {
        remove_if_exists(&crate::pending_count::preview_count_path(
            &plan.work_directory,
        ))?;
    }
    let state = state_path(work_root, connection.id);
    if read_state(&state)?.is_some_and(|existing| existing.state == OfflineOperationState::Running)
    {
        return Err(crate::fl!(
            "ui-bisync-coordinator-another-offline-mirror-operation-is-already-running"
        ));
    }
    write_state(
        &state,
        &OfflineCoordinatorState::new(
            connection.id,
            mode.operation(),
            OfflineOperationState::Running,
        ),
    )?;
    let result = run_transaction(
        connection,
        plan,
        mode,
        runner,
        cancellation.clone(),
        &local_paths,
    )
    .await;
    let final_state = if result.is_ok() {
        OfflineOperationState::Succeeded
    } else if cancellation.is_cancelled() {
        OfflineOperationState::Cancelled
    } else {
        OfflineOperationState::Failed
    };
    let warnings = result
        .as_ref()
        .map(|outcome| outcome.warnings.clone())
        .unwrap_or_default();
    let state_result = write_state(
        &state,
        &OfflineCoordinatorState::new(connection.id, mode.operation(), final_state)
            .with_warnings(warnings),
    );
    match (result, state_result) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn validate_connection(connection: &Connection) -> Result<(), String> {
    if !connection.enabled || !matches!(connection.mode, ConnectionMode::OfflineMirror(_)) {
        return Err(crate::fl!(
            "ui-bisync-coordinator-the-saved-connection-is-not-an-enabled"
        ));
    }
    match connection.provider {
        Provider::OneDrive
            if connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone) =>
        {
            Ok(())
        }
        Provider::Teams
        | Provider::GoogleDrive
        | Provider::Box
        | Provider::Smb
        | Provider::Sftp => Ok(()),
        _ => Err(crate::fl!(
            "ui-bisync-coordinator-the-saved-connection-is-not-an-rclone"
        )),
    }
}

fn initial_preview_marker(plan: &RcloneBisyncPlan) -> PathBuf {
    plan.work_directory.join("initial-preview-confirmable")
}

fn initial_sync_marker(plan: &RcloneBisyncPlan) -> PathBuf {
    plan.work_directory.join("initial-sync-complete")
}

fn validate_mode_markers(mode: BisyncMode, plan: &RcloneBisyncPlan) -> Result<(), String> {
    let initialized = initial_sync_marker(plan).is_file();
    match mode {
        BisyncMode::InitialPreview if initialized => Err(crate::fl!(
            "ui-bisync-coordinator-initial-preview-is-unavailable-after-initialization"
        )),
        BisyncMode::Preview if !initialized => Err(crate::fl!(
            "ui-bisync-coordinator-normal-preview-requires-completed-initial-synchronization"
        )),
        BisyncMode::InitialSync if initialized => Err(crate::fl!(
            "ui-bisync-coordinator-initial-synchronization-is-already-complete"
        )),
        BisyncMode::InitialSync if !initial_preview_marker(plan).is_file() => Err(crate::fl!(
            "ui-bisync-coordinator-initial-synchronization-requires-a-successful-preview"
        )),
        BisyncMode::Sync | BisyncMode::Scheduled if !initialized => Err(crate::fl!(
            "ui-bisync-coordinator-synchronization-requires-completed-initial-synchronization"
        )),
        _ => Ok(()),
    }
}

fn prepare_inputs(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
    mode: BisyncMode,
) -> Result<(), String> {
    if fs::symlink_metadata(&connection.local_path)
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(crate::fl!("mirror-access-unavailable"));
    }
    if fs::symlink_metadata(&plan.recovery_directory)
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(crate::fl!(
            "ui-bisync-coordinator-recovery-root-is-a-symlink"
        ));
    }
    fs::create_dir_all(&connection.local_path).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-mirror-directory-error",
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(&plan.work_directory).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-bisync-work-directory-error",
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(&plan.recovery_directory).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-recovery-directory-error",
            error = (error).to_string()
        )
    })?;
    write_atomic(
        &plan.filters_file,
        rclone_bisync_filter_file(connection.provider).as_bytes(),
    )?;
    if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone)
        && mode == BisyncMode::InitialPreview
    {
        return Ok(());
    }
    ensure_local_access_marker(connection, plan).map_err(|error| {
        if connection.provider == Provider::OneDrive {
            crate::onedrive::OneDriveError::Ownership.to_string()
        } else {
            error
        }
    })
}

fn ensure_local_access_marker(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
) -> Result<(), String> {
    let Some(name) = &plan.access_marker_name else {
        return Ok(());
    };
    let path = connection.local_path.join(name);
    let expected = format!("{}\n", connection.id);
    if !path.exists() && !initial_sync_marker(plan).exists() {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|error| {
                crate::fl!(
                    "ui-bisync-coordinator-could-not-create-sharepoint-access-marker-error",
                    error = (error).to_string()
                )
            })?;
        file.write_all(expected.as_bytes()).map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-write-sharepoint-access-marker-error",
                error = (error).to_string()
            )
        })?;
    }
    validate_local_access_marker(connection, plan)
}

fn validate_local_access_marker(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
) -> Result<(), String> {
    let Some(name) = &plan.access_marker_name else {
        return Ok(());
    };
    let path = connection.local_path.join(name);
    let expected = format!("{}\n", connection.id);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| crate::fl!("ui-bisync-coordinator-sharepoint-access-marker-is-missing"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(crate::fl!(
            "ui-bisync-coordinator-sharepoint-access-marker-is-unsafe"
        ));
    }
    if fs::read_to_string(path).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-read-sharepoint-access-marker-error",
            error = (error).to_string()
        )
    })? != expected
    {
        return Err(crate::fl!(
            "ui-bisync-coordinator-sharepoint-access-marker-changed"
        ));
    }
    Ok(())
}

async fn run_transaction<R: CommandRunner>(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
    mode: BisyncMode,
    runner: &R,
    cancellation: CancellationToken,
    local_paths: &LocalMirrorPaths,
) -> Result<BisyncOutcome, String> {
    let mut local_plan = plan.clone();
    local_plan.recovery_directory = local_paths.recovery.clone();
    let (day, stamp) = utc_day_and_stamp(SystemTime::now())?;
    if mode == BisyncMode::Scheduled {
        prepare_scheduled_progress(plan)?;
    }
    validate_remote_access_marker(connection, plan, mode, runner, cancellation.child_token())
        .await
        .map_err(|error| {
            if connection.provider == Provider::OneDrive {
                crate::onedrive::OneDriveError::Ownership.to_string()
            } else {
                error
            }
        })?;
    let request = bisync_request(plan, mode, &day, &stamp)?;
    if mode.preview() {
        let output = runner
            .run(request, cancellation.child_token())
            .await
            .map_err(|error| {
                command_failure(
                    &crate::fl!(
                        "ui-bisync-coordinator-bisync-value1",
                        value1 = (mode.as_str()).to_string()
                    ),
                    error,
                )
            })?;
        let preview_pending = if connection.provider == Provider::OneDrive
            && connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone)
        {
            let estimate = crate::pending_count::rclone_preview_snapshot(
                runner,
                &plan.path1_remote,
                &plan.path2_local,
                &plan.filters_file,
                cancellation.child_token(),
            )
            .await;
            if cancellation.is_cancelled() {
                return Err(crate::fl!("microsoft-oauth-cancelled"));
            }
            let report = crate::pending_count::PreviewCount {
                version: 1,
                connection_id: connection.id,
                observed_unix_seconds: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| crate::fl!("pending-count-response-incomplete"))?
                    .as_secs(),
                files: estimate.as_ref().ok().copied(),
            };
            let bytes = serde_json::to_vec(&report)
                .map_err(|_| crate::fl!("pending-count-response-incomplete"))?;
            write_atomic(
                &crate::pending_count::preview_count_path(&plan.work_directory),
                &bytes,
            )?;
            Some(estimate)
        } else {
            None
        };
        if mode == BisyncMode::InitialPreview {
            write_atomic(
                &initial_preview_marker(plan),
                b"Cloud Mounter initial Preview completed.\n",
            )?;
        }
        return Ok(BisyncOutcome {
            output,
            preview_pending,
            warnings: Vec::new(),
        });
    }
    prepare_recovery(
        connection,
        &local_plan,
        &day,
        runner,
        cancellation.child_token(),
    )
    .await?;
    let output = runner
        .run(request, cancellation.child_token())
        .await
        .map_err(|error| {
            command_failure(
                &crate::fl!(
                    "ui-bisync-coordinator-bisync-value1",
                    value1 = (mode.as_str()).to_string()
                ),
                error,
            )
        })?;
    if plan.verify_after_sync {
        let convergence_mode = if mode == BisyncMode::InitialSync {
            BisyncMode::Sync
        } else {
            mode
        };
        runner
            .run(
                bisync_request(plan, convergence_mode, &day, &stamp)?,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| {
                command_failure(
                    &crate::fl!("ui-bisync-coordinator-sharepoint-convergence"),
                    error,
                )
            })?;
        runner
            .run(
                rclone_request(
                    [
                        "check",
                        &plan.path1_remote,
                        &plan.path2_local.display().to_string(),
                        "--checksum",
                        "--filter-from",
                        &plan.filters_file.display().to_string(),
                    ],
                    Duration::from_secs(15 * 60),
                )?,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| {
                command_failure(
                    &crate::fl!("ui-bisync-coordinator-sharepoint-verification"),
                    error,
                )
            })?;
    }
    if connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(crate::model::OfflineEngine::Rclone)
        && matches!(mode, BisyncMode::InitialSync | BisyncMode::Sync)
    {
        // Engine success must not initialize a mirror whose anchors exist only
        // in a sandbox shadow directory, or were removed during the transfer.
        let mut local_connection = connection.clone();
        local_connection.local_path = local_paths.mirror.clone();
        validate_local_access_marker(&local_connection, &local_plan)
            .map_err(|_| crate::onedrive::OneDriveError::Ownership.to_string())?;
        validate_remote_access_marker(
            connection,
            plan,
            BisyncMode::Sync,
            runner,
            cancellation.child_token(),
        )
        .await
        .map_err(|_| crate::onedrive::OneDriveError::Ownership.to_string())?;
        runner
            .run(
                rclone_request(
                    [
                        "check",
                        &plan.path1_remote,
                        &plan.path2_local.display().to_string(),
                        "--checksum",
                        "--filter-from",
                        &plan.filters_file.display().to_string(),
                    ],
                    Duration::from_secs(15 * 60),
                )?,
                cancellation.child_token(),
            )
            .await
            .map_err(|_| crate::fl!("onedrive-checksum-verification-failed"))?;
    }
    let warnings =
        cleanup_retention(connection, &local_plan, runner, cancellation.child_token()).await;
    if mode == BisyncMode::InitialSync {
        write_atomic(
            &initial_sync_marker(plan),
            b"Cloud Mounter initial synchronization completed.\n",
        )?;
        remove_if_exists(&initial_preview_marker(plan))?;
    }
    Ok(BisyncOutcome {
        output,
        warnings,
        preview_pending: None,
    })
}

fn bisync_request(
    plan: &RcloneBisyncPlan,
    mode: BisyncMode,
    day: &str,
    stamp: &str,
) -> Result<CommandRequest, String> {
    let mut arguments = Vec::new();
    let mut index = 0;
    while index < plan.service.arguments.len() {
        let argument = &plan.service.arguments[index];
        if matches!(argument.as_str(), "--backup-dir1" | "--backup-dir2") {
            let value = if argument == "--backup-dir1" {
                format!("{}/{day}", plan.remote_recovery_path.trim_end_matches('/'))
            } else {
                plan.recovery_directory.join(day).display().to_string()
            };
            arguments.push(argument.clone());
            arguments.push(value);
            index += 2;
        } else {
            arguments.push(argument.clone());
            index += 1;
        }
    }
    arguments.extend(["--suffix".into(), format!("-{stamp}")]);
    if mode.initial() {
        arguments.push("--resync".into());
    }
    if mode.preview() {
        arguments.push("--dry-run".into());
    }
    if mode == BisyncMode::Scheduled {
        arguments.extend([
            "--use-json-log".into(),
            "--log-file".into(),
            live_progress_log_path(&plan.work_directory)
                .display()
                .to_string(),
            "--log-level".into(),
            "INFO".into(),
        ]);
    }
    rclone_request(
        arguments.iter().map(String::as_str),
        if mode.preview() {
            Duration::from_secs(120)
        } else {
            Duration::from_secs(15 * 60)
        },
    )
}

fn rclone_request<'a>(
    arguments: impl IntoIterator<Item = &'a str>,
    timeout: Duration,
) -> Result<CommandRequest, String> {
    let mut request = CommandRequest::new(Executable::Rclone)
        .arg("--contimeout")
        .and_then(|request| request.arg("10s"))
        .and_then(|request| request.arg("--timeout"))
        .and_then(|request| request.arg("30s"))
        .map_err(|error| error.to_string())?;
    for argument in arguments {
        request = request.arg(argument).map_err(|error| error.to_string())?;
    }
    Ok(request.with_timeout(timeout).with_output_limit(256 * 1024))
}

async fn validate_remote_access_marker<R: CommandRunner>(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
    mode: BisyncMode,
    runner: &R,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let Some(name) = &plan.access_marker_name else {
        return Ok(());
    };
    if mode.initial() {
        return Ok(());
    }
    let remote = format!("{}/{}", plan.path1_remote.trim_end_matches('/'), name);
    let output = runner
        .run(
            rclone_request(["cat", &remote], Duration::from_secs(60))?,
            cancellation,
        )
        .await
        .map_err(|_| {
            crate::fl!("ui-bisync-coordinator-sharepoint-remote-access-marker-is-missing")
        })?;
    if complete_stdout(
        &output,
        &crate::fl!("ui-bisync-coordinator-sharepoint-remote-access-marker"),
    )?
    .trim_end()
        != connection.id.to_string()
    {
        return Err(crate::fl!(
            "ui-bisync-coordinator-sharepoint-remote-access-marker-changed"
        ));
    }
    Ok(())
}

async fn prepare_recovery<R: CommandRunner>(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
    day: &str,
    runner: &R,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let local = plan.recovery_directory.join(day);
    if fs::symlink_metadata(&local).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(crate::fl!(
            "ui-bisync-coordinator-recovery-path-is-a-symlink"
        ));
    }
    fs::create_dir_all(&local).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-local-recovery-directory-error",
            error = (error).to_string()
        )
    })?;
    validate_local_owner(&local, connection.id)?;
    write_atomic(
        &local.join(OWNER_MARKER),
        format!("{}\n", connection.id).as_bytes(),
    )?;

    let remote = format!("{}/{day}", plan.remote_recovery_path.trim_end_matches('/'));
    runner
        .run(
            rclone_request(["mkdir", &remote], Duration::from_secs(60))?,
            cancellation.child_token(),
        )
        .await
        .map_err(|error| {
            command_failure(
                &crate::fl!("ui-bisync-coordinator-remote-recovery-preparation"),
                error,
            )
        })?;
    let remote_marker = format!("{remote}/{OWNER_MARKER}");
    let owner = match runner
        .run(
            rclone_request(["cat", &remote_marker], Duration::from_secs(60))?,
            cancellation.child_token(),
        )
        .await
    {
        Ok(output) => complete_stdout(
            &output,
            &crate::fl!("ui-bisync-coordinator-remote-recovery-owner"),
        )?
        .trim_end()
        .to_owned(),
        Err(CommandError::NonZero { .. }) => String::new(),
        Err(error) => {
            return Err(command_failure(
                &crate::fl!("ui-bisync-coordinator-remote-recovery-owner-inspection"),
                error,
            ));
        }
    };
    if !owner.is_empty() && owner != connection.id.to_string() {
        return Err(crate::fl!(
            "ui-bisync-coordinator-remote-recovery-directory-belongs-to-another-connection"
        ));
    }
    if owner.is_empty() {
        let listing = runner
            .run(
                rclone_request(["lsf", &remote], Duration::from_secs(60))?,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| {
                command_failure(
                    &crate::fl!("ui-bisync-coordinator-remote-recovery-ownership-check"),
                    error,
                )
            })?;
        if !complete_stdout(
            &listing,
            &crate::fl!("ui-bisync-coordinator-remote-recovery-listing"),
        )?
        .trim()
        .is_empty()
        {
            return Err(crate::fl!(
                "ui-bisync-coordinator-remote-recovery-directory-is-not-applet-owned"
            ));
        }
    }
    let marker = plan
        .work_directory
        .join(format!(".remote-owner-{}.tmp", Uuid::new_v4()));
    write_private(&marker, format!("{}\n", connection.id).as_bytes())?;
    let result = runner
        .run(
            rclone_request(
                ["copyto", &marker.display().to_string(), &remote_marker],
                Duration::from_secs(60),
            )?,
            cancellation,
        )
        .await
        .map_err(|error| {
            command_failure(
                &crate::fl!("ui-bisync-coordinator-remote-recovery-owner-publication"),
                error,
            )
        });
    let _ = fs::remove_file(marker);
    result.map(|_| ())
}

fn validate_local_owner(directory: &Path, id: ConnectionId) -> Result<(), String> {
    let marker = directory.join(OWNER_MARKER);
    if fs::symlink_metadata(&marker).is_ok() {
        if fs::symlink_metadata(&marker).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(crate::fl!(
                "ui-bisync-coordinator-local-recovery-owner-marker-is-a-symlink"
            ));
        }
        if fs::read_to_string(marker)
            .map_err(|error| {
                crate::fl!(
                    "ui-bisync-coordinator-could-not-read-recovery-owner-error",
                    error = (error).to_string()
                )
            })?
            .trim_end()
            != id.to_string()
        {
            return Err(crate::fl!(
                "ui-bisync-coordinator-local-recovery-directory-belongs-to-another-connection"
            ));
        }
    } else if fs::read_dir(directory)
        .map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-inspect-recovery-directory-error",
                error = (error).to_string()
            )
        })?
        .next()
        .is_some()
    {
        return Err(crate::fl!(
            "ui-bisync-coordinator-local-recovery-directory-is-not-applet-owned"
        ));
    }
    Ok(())
}

async fn cleanup_retention<R: CommandRunner>(
    connection: &Connection,
    plan: &RcloneBisyncPlan,
    runner: &R,
    cancellation: CancellationToken,
) -> Vec<String> {
    let mut issues = RetentionCleanupIssues::default();
    if cancellation.is_cancelled() {
        issues.cancelled = true;
        return issues.warnings();
    }
    let cutoff = match utc_day(SystemTime::now() - Duration::from_secs(RETENTION_DAYS * 86_400)) {
        Ok(cutoff) => cutoff,
        Err(_) => {
            issues.local_listing_failed = true;
            issues.remote_listing_failed = true;
            return issues.warnings();
        }
    };
    match fs::read_dir(&plan.recovery_directory) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => {
                        issues.local_entries_unreadable += 1;
                        continue;
                    }
                };
                if cancellation.is_cancelled() {
                    issues.cancelled = true;
                    return issues.warnings();
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                let path = entry.path();
                if !strict_day(&name) || name >= cutoff {
                    continue;
                }
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        issues.local_ownership_unverified += 1;
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    issues.local_ownership_unverified += 1;
                    continue;
                }
                let marker = path.join(OWNER_MARKER);
                let marker_metadata = match fs::symlink_metadata(&marker) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        issues.local_ownership_unverified += 1;
                        continue;
                    }
                };
                if marker_metadata.file_type().is_symlink() || !marker_metadata.is_file() {
                    issues.local_ownership_unverified += 1;
                    continue;
                }
                let owned = match fs::read_to_string(&marker) {
                    Ok(owner) => owner.trim_end() == connection.id.to_string(),
                    Err(_) => {
                        issues.local_ownership_unverified += 1;
                        continue;
                    }
                };
                if owned && fs::remove_dir_all(&path).is_err() {
                    issues.local_deletions_failed += 1;
                }
            }
        }
        Err(_) => issues.local_listing_failed = true,
    }
    if cancellation.is_cancelled() {
        issues.cancelled = true;
        return issues.warnings();
    }
    let listing_request = match rclone_request(
        [
            "lsf",
            "--dirs-only",
            "--max-depth",
            "1",
            &plan.remote_recovery_path,
        ],
        Duration::from_secs(60),
    ) {
        Ok(request) => request,
        Err(_) => {
            issues.remote_listing_failed = true;
            return issues.warnings();
        }
    };
    let listing = runner
        .run(listing_request, cancellation.child_token())
        .await;
    let listing = match listing {
        Ok(listing)
            if complete_stdout(
                &listing,
                &crate::fl!("ui-bisync-coordinator-retention-listing"),
            )
            .is_ok() =>
        {
            listing
        }
        Ok(_) => {
            issues.remote_listing_failed = true;
            return issues.warnings();
        }
        Err(_) => {
            issues.remote_listing_failed = true;
            return issues.warnings();
        }
    };
    for entry in listing.stdout.text.lines() {
        if cancellation.is_cancelled() {
            issues.cancelled = true;
            return issues.warnings();
        }
        let name = entry.trim_end_matches('/');
        if !strict_day(name) || name >= cutoff.as_str() {
            continue;
        }
        let target = format!(
            "{}/{}",
            plan.remote_recovery_path.trim_end_matches('/'),
            name
        );
        let marker = format!("{target}/{OWNER_MARKER}");
        let marker_request = match rclone_request(["cat", &marker], Duration::from_secs(60)) {
            Ok(request) => request,
            Err(_) => {
                issues.remote_ownership_unverified += 1;
                continue;
            }
        };
        let owned = match runner.run(marker_request, cancellation.child_token()).await {
            Ok(output) => match complete_stdout(
                &output,
                &crate::fl!("ui-bisync-coordinator-retention-owner"),
            ) {
                Ok(owner) => owner.trim_end() == connection.id.to_string(),
                Err(_) => {
                    issues.remote_ownership_unverified += 1;
                    continue;
                }
            },
            Err(_) => {
                issues.remote_ownership_unverified += 1;
                continue;
            }
        };
        if !owned {
            continue;
        }
        let purge_request = match rclone_request(["purge", &target], Duration::from_secs(60)) {
            Ok(request) => request,
            Err(_) => {
                issues.remote_deletions_failed += 1;
                continue;
            }
        };
        if runner
            .run(purge_request, cancellation.child_token())
            .await
            .is_err()
        {
            issues.remote_deletions_failed += 1;
        }
    }
    issues.warnings()
}

fn prepare_scheduled_progress(plan: &RcloneBisyncPlan) -> Result<(), String> {
    let invocation = std::env::var("INVOCATION_ID")
        .map_err(|_| crate::fl!("ui-bisync-coordinator-systemd-invocation-id-is-unavailable"))?;
    prepare_scheduled_progress_for_invocation(plan, &invocation)
}

fn prepare_scheduled_progress_for_invocation(
    plan: &RcloneBisyncPlan,
    invocation: &str,
) -> Result<(), String> {
    if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(crate::fl!(
            "ui-bisync-coordinator-systemd-invocation-id-is-invalid"
        ));
    }
    write_atomic(&live_progress_log_path(&plan.work_directory), b"")?;
    write_atomic(
        &live_progress_state_path(&plan.work_directory),
        format!("{{\"version\":1,\"invocation_id\":\"{invocation}\"}}\n").as_bytes(),
    )
}

fn strict_day(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit()))
    {
        return false;
    }
    let Ok(year) = value[0..4].parse::<u32>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<u32>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u32>() else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximum).contains(&day)
}

fn utc_day_and_stamp(now: SystemTime) -> Result<(String, String), String> {
    let duration = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| crate::fl!("ui-bisync-coordinator-system-clock-precedes-unix-epoch"))?;
    let day = utc_day(now)?;
    let tm = utc_tm(duration.as_secs())?;
    Ok((
        day,
        format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}.{:09}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec,
            duration.subsec_nanos()
        ),
    ))
}

fn utc_day(now: SystemTime) -> Result<String, String> {
    let duration = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| crate::fl!("ui-bisync-coordinator-system-clock-precedes-unix-epoch"))?;
    let tm = utc_tm(duration.as_secs())?;
    Ok(format!(
        "{:04}-{:02}-{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday
    ))
}

fn utc_tm(seconds: u64) -> Result<libc::tm, String> {
    let raw: libc::time_t = seconds
        .try_into()
        .map_err(|_| crate::fl!("ui-bisync-coordinator-system-time-is-out-of-range"))?;
    let mut output = unsafe { std::mem::zeroed::<libc::tm>() };
    if unsafe { libc::gmtime_r(&raw, &raw mut output) }.is_null() {
        Err(crate::fl!(
            "ui-bisync-coordinator-could-not-convert-utc-time"
        ))
    } else {
        Ok(output)
    }
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| crate::fl!("ui-bisync-coordinator-managed-file-has-no-parent"))?;
    fs::create_dir_all(parent).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-managed-directory-error",
            error = (error).to_string()
        )
    })?;
    let staging = parent.join(format!(".managed-{}.tmp", Uuid::new_v4()));
    let result = write_private(&staging, contents).and_then(|()| {
        fs::rename(&staging, path).map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-publish-managed-file-error",
                error = (error).to_string()
            )
        })
    });
    if result.is_err() {
        let _ = fs::remove_file(staging);
    }
    result
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-stage-managed-file-error",
                error = (error).to_string()
            )
        })?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-restrict-managed-file-error",
                error = (error).to_string()
            )
        })?;
    file.write_all(contents).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-write-managed-file-error",
            error = (error).to_string()
        )
    })?;
    file.sync_all().map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-sync-managed-file-error",
            error = (error).to_string()
        )
    })
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(crate::fl!(
            "ui-bisync-coordinator-could-not-remove-managed-marker-error",
            error = (error).to_string()
        )),
    }
}

fn complete_stdout<'a>(output: &'a CommandOutput, stage: &str) -> Result<&'a str, String> {
    if output.stdout.truncated || output.stdout.invalid_utf8 {
        return Err(crate::fl!(
            "ui-bisync-coordinator-stage-returned-incomplete-or-invalid-output",
            stage = (stage).to_string()
        ));
    }
    Ok(&output.stdout.text)
}

fn command_failure(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(_) => crate::fl!(
            "ui-bisync-coordinator-stage-failed-because-rclone-is-missing",
            stage = (stage).to_string()
        ),
        CommandError::InvalidArgument => crate::fl!(
            "ui-bisync-coordinator-stage-rejected-an-unsafe-argument",
            stage = (stage).to_string()
        ),
        CommandError::Timeout { timeout, .. } => {
            crate::fl!(
                "ui-bisync-coordinator-stage-timed-out-after-value2-seconds",
                stage = (stage).to_string(),
                value2 = (timeout.as_secs()).to_string()
            )
        }
        CommandError::Cancelled { .. } => crate::fl!(
            "ui-bisync-coordinator-stage-was-cancelled",
            stage = (stage).to_string()
        ),
        CommandError::Spawn { .. } => crate::fl!(
            "ui-bisync-coordinator-stage-could-not-start-rclone",
            stage = (stage).to_string()
        ),
        CommandError::NonZero { code, .. } => match code {
            Some(code) => crate::fl!(
                "ui-bisync-coordinator-stage-failed-with-exit-status-code",
                stage = (stage).to_string(),
                code = (code).to_string()
            ),
            None => crate::fl!(
                "ui-bisync-coordinator-stage-failed",
                stage = (stage).to_string()
            ),
        },
    }
}

struct OperationLock {
    file: File,
}

impl OperationLock {
    fn acquire(work_root: &Path, id: ConnectionId) -> Result<Self, String> {
        let directory = work_root.join("offline-coordinator").join(id.to_string());
        fs::create_dir_all(&directory).map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-create-coordinator-lock-directory-error",
                error = (error).to_string()
            )
        })?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("operation.lock"))
            .map_err(|error| {
                crate::fl!(
                    "ui-bisync-coordinator-could-not-open-coordinator-lock-error",
                    error = (error).to_string()
                )
            })?;
        #[cfg(unix)]
        {
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|error| {
                    crate::fl!(
                        "ui-bisync-coordinator-could-not-restrict-coordinator-lock-error",
                        error = (error).to_string()
                    )
                })?;
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err(crate::fl!(
                    "ui-bisync-coordinator-another-offline-mirror-operation-is-already-running"
                ));
            }
        }
        Ok(Self { file })
    }
}

impl Drop for OperationLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

/// Exclusive cross-process ownership of one rclone Offline Mirror operation.
///
/// The fields remain private so callers can only hold and release ownership;
/// they cannot substitute a different lock file or unlock it early.
pub struct OperationGuard {
    _lock: OperationLock,
}

pub fn acquire_operation_guard(
    work_root: &Path,
    id: ConnectionId,
) -> Result<OperationGuard, String> {
    OperationLock::acquire(work_root, id).map(|_lock| OperationGuard { _lock })
}

pub fn acquire_operation_guard_for_state(state: &Path) -> Result<OperationGuard, String> {
    let directory = state.parent().ok_or_else(|| {
        crate::fl!("ui-bisync-coordinator-coordinator-state-has-no-parent-directory")
    })?;
    fs::create_dir_all(directory).map_err(|error| {
        crate::fl!(
            "ui-bisync-coordinator-could-not-create-coordinator-lock-directory-error",
            error = (error).to_string()
        )
    })?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("operation.lock"))
        .map_err(|error| {
            crate::fl!(
                "ui-bisync-coordinator-could-not-open-coordinator-lock-error",
                error = (error).to_string()
            )
        })?;
    #[cfg(unix)]
    {
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                crate::fl!(
                    "ui-bisync-coordinator-could-not-restrict-coordinator-lock-error",
                    error = (error).to_string()
                )
            })?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(crate::fl!(
                "ui-bisync-coordinator-another-offline-mirror-operation-is-still-running"
            ));
        }
    }
    Ok(OperationGuard {
        _lock: OperationLock { file },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{OfflineMirrorConfig, TuningProfile};
    use crate::process::{CapturedOutput, FakeCommandRunner};

    #[test]
    fn flatpak_worker_uses_the_same_host_shared_state_root_as_the_applet() {
        use crate::process::CommandExecutionMode::{FlatpakSpawnHost, Native};
        let home = Some(PathBuf::from("/home/example"));
        let sandbox_state = Some(PathBuf::from("/home/example/.var/app/example/state"));
        assert_eq!(
            work_root_for(FlatpakSpawnHost, home.clone(), sandbox_state.clone()),
            PathBuf::from("/home/example/.local/state/cosmic-ext-applet-mounter")
        );
        assert_eq!(
            work_root_for(Native, home, sandbox_state.clone()),
            PathBuf::from("/home/example/.var/app/example/state/cosmic-ext-applet-mounter")
        );
    }

    #[test]
    fn authorized_local_paths_do_not_change_host_engine_arguments() {
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_connection(root.path());
        let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
        let paths = LocalMirrorPaths {
            mirror: root.path().join("portal/mirror"),
            recovery: root.path().join("portal/recovery"),
        };
        let mut local_connection = connection.clone();
        local_connection.local_path = paths.mirror.clone();
        let mut local_plan = plan.clone();
        local_plan.recovery_directory = paths.recovery.clone();
        prepare_inputs(&local_connection, &local_plan, BisyncMode::InitialSync).unwrap();
        assert!(
            paths
                .mirror
                .join(plan.access_marker_name.as_ref().unwrap())
                .is_file()
        );
        assert!(!connection.local_path.exists());
        assert!(!plan.recovery_directory.exists());
        let request =
            bisync_request(&plan, BisyncMode::InitialSync, "2026-10-17", "stamp").unwrap();
        let command = request.sanitized_command();
        assert!(command.contains(&plan.path2_local.display().to_string()));
        assert!(command.contains(&plan.recovery_directory.display().to_string()));
        assert!(!command.contains("portal/mirror"));
        assert!(!command.contains("portal/recovery"));
    }

    #[tokio::test]
    async fn missing_post_sync_anchor_never_initializes_or_runs_checksum() {
        for missing_local in [true, false] {
            let root = tempfile::tempdir().unwrap();
            let connection = onedrive_connection(root.path());
            let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
            prepare_inputs(&connection, &plan, BisyncMode::InitialSync).unwrap();
            if missing_local {
                fs::remove_file(
                    connection
                        .local_path
                        .join(plan.access_marker_name.as_ref().unwrap()),
                )
                .unwrap();
            }
            let runner = FakeCommandRunner::default();
            push_successful_sync_prefix(&runner);
            if !missing_local {
                runner.push(Ok(output("foreign-owner")));
            }
            assert!(
                run_transaction(
                    &connection,
                    &plan,
                    BisyncMode::InitialSync,
                    &runner,
                    CancellationToken::new(),
                    &LocalMirrorPaths::native(&connection).unwrap()
                )
                .await
                .is_err()
            );
            assert!(!initial_sync_marker(&plan).exists());
            assert!(
                !runner
                    .requests()
                    .iter()
                    .any(|r| r.sanitized_command().contains(" check "))
            );
            assert!(
                !connection
                    .local_path
                    .join(plan.access_marker_name.as_ref().unwrap())
                    .exists()
                    || !missing_local
            );
        }
    }

    #[test]
    fn retention_counts_use_fluent_plural_selection() {
        for (count, expected) in [(1, "1 recovery entry;"), (2, "2 recovery entries;")] {
            let warnings = RetentionCleanupIssues {
                local_entries_unreadable: count,
                ..RetentionCleanupIssues::default()
            }
            .warnings();
            assert_eq!(warnings.len(), 1);
            assert!(warnings[0].contains(expected), "{}", warnings[0]);
        }
    }

    fn connection(root: &Path) -> Connection {
        Connection {
            id: ConnectionId::new(),
            name: "Coordinator test".into(),
            provider: Provider::GoogleDrive,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                engine: None,
                recovery_directory: root.join("recovery"),
                sync_interval_minutes: 15,
                sync_on_metered: false,
            }),
            remote_reference: "test_remote".into(),
            remote_subpath: Some("mirror".into()),
            local_path: root.join("local"),
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

    fn output(stdout: &str) -> CommandOutput {
        CommandOutput {
            command: "rclone".into(),
            stdout: CapturedOutput {
                text: stdout.into(),
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

    fn push_successful_sync_prefix(runner: &FakeCommandRunner) {
        runner.push(Ok(output("")));
        runner.push(Err(CommandError::NonZero {
            command: "rclone cat".into(),
            code: Some(1),
            stdout: output("").stdout,
            stderr: output("").stderr,
            attempts: 1,
        }));
        runner.push(Ok(output("")));
        runner.push(Ok(output("")));
        runner.push(Ok(output("")));
    }

    #[tokio::test]
    async fn required_phases_stop_on_failure_timeout_and_cancellation() {
        // Remote access anchor, recovery mkdir/owner/publication, bisync,
        // SharePoint convergence, and checksum verification are all required.
        for phase in 0..7 {
            for failure in 0..3 {
                // A nonzero owner lookup is deliberately allowed only after
                // a complete empty listing proves there is no unowned data.
                if phase == 2 && failure == 0 {
                    continue;
                }
                let root = tempfile::tempdir().unwrap();
                let connection = connection(root.path());
                let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
                let mut plan = plan;
                plan.access_marker_name = Some("anchor".into());
                plan.verify_after_sync = true;
                fs::create_dir_all(&plan.work_directory).unwrap();
                let runner = FakeCommandRunner::default();
                let responses = [
                    connection.id.to_string(),
                    String::new(),
                    connection.id.to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ];
                for response in responses.iter().take(phase) {
                    runner.push(Ok(output(response)));
                }
                let error = match failure {
                    0 => CommandError::NonZero {
                        command: "secret:path password=private".into(),
                        code: Some(9),
                        stdout: output("secret").stdout,
                        stderr: output("secret").stderr,
                        attempts: 1,
                    },
                    1 => CommandError::Timeout {
                        command: "secret:path".into(),
                        timeout: Duration::from_secs(60),
                    },
                    _ => CommandError::Cancelled {
                        command: "secret:path".into(),
                    },
                };
                runner.push(Err(error));
                let error = run_transaction(
                    &connection,
                    &plan,
                    BisyncMode::Sync,
                    &runner,
                    CancellationToken::new(),
                    &LocalMirrorPaths::native(&connection).unwrap(),
                )
                .await
                .unwrap_err();
                assert_eq!(
                    runner.requests().len(),
                    phase + 1,
                    "phase {phase}, failure {failure}"
                );
                assert!(!error.contains("secret"));
                assert!(!initial_sync_marker(&plan).exists());
                assert!(
                    !runner
                        .requests()
                        .iter()
                        .any(|request| request.sanitized_command().contains("lsf --dirs-only"))
                );
            }
        }
    }

    #[tokio::test]
    async fn ownership_and_anchor_reads_require_complete_output() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let mut plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        for invalid_utf8 in [false, true] {
            let mut incomplete = output(&connection.id.to_string());
            incomplete.stdout.truncated = !invalid_utf8;
            incomplete.stdout.invalid_utf8 = invalid_utf8;
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output("")));
            runner.push(Ok(incomplete.clone()));
            assert!(
                prepare_recovery(
                    &connection,
                    &plan,
                    "2026-10-11",
                    &runner,
                    CancellationToken::new()
                )
                .await
                .unwrap_err()
                .contains("incomplete or invalid")
            );
            assert_eq!(runner.requests().len(), 2);
            plan.access_marker_name = Some("anchor".into());
            let runner = FakeCommandRunner::default();
            runner.push(Ok(incomplete));
            assert!(
                validate_remote_access_marker(
                    &connection,
                    &plan,
                    BisyncMode::Sync,
                    &runner,
                    CancellationToken::new()
                )
                .await
                .is_err()
            );
            assert_eq!(runner.requests().len(), 1);
        }
        for text in ["", "foreign-id"] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(text)));
            assert!(
                validate_remote_access_marker(
                    &connection,
                    &plan,
                    BisyncMode::Sync,
                    &runner,
                    CancellationToken::new()
                )
                .await
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn initial_operations_never_publish_success_markers_after_required_failure() {
        for mode in [BisyncMode::InitialPreview, BisyncMode::InitialSync] {
            let phases = if mode.preview() { 1 } else { 4 };
            for phase in 0..phases {
                for failure in 0..3 {
                    if phase == 1 && failure == 0 && !mode.preview() {
                        continue;
                    }
                    let root = tempfile::tempdir().unwrap();
                    let connection = connection(root.path());
                    let work = root.path().join("state");
                    let plan = rclone_bisync_plan(&connection, &work).unwrap();
                    fs::create_dir_all(&plan.work_directory).unwrap();
                    if !mode.preview() {
                        write_atomic(&initial_preview_marker(&plan), b"preview").unwrap();
                    }
                    let runner = FakeCommandRunner::default();
                    let responses = [
                        String::new(),
                        connection.id.to_string(),
                        String::new(),
                        String::new(),
                    ];
                    for response in responses.iter().take(phase) {
                        runner.push(Ok(output(response)));
                    }
                    runner.push(Err(match failure {
                        0 => CommandError::NonZero {
                            command: "secret".into(),
                            code: Some(9),
                            stdout: output("").stdout,
                            stderr: output("").stderr,
                            attempts: 1,
                        },
                        1 => CommandError::Timeout {
                            command: "secret".into(),
                            timeout: Duration::from_secs(60),
                        },
                        _ => CommandError::Cancelled {
                            command: "secret".into(),
                        },
                    }));
                    assert!(
                        run(&connection, mode, &work, &runner, CancellationToken::new())
                            .await
                            .is_err()
                    );
                    assert_eq!(runner.requests().len(), phase + 1);
                    assert!(!initial_sync_marker(&plan).exists());
                    assert_eq!(initial_preview_marker(&plan).exists(), !mode.preview());
                    assert_eq!(
                        read_state(&state_path(&work, connection.id))
                            .unwrap()
                            .unwrap()
                            .state,
                        OfflineOperationState::Failed
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn unsafe_recovery_paths_and_foreign_remote_owner_stop_before_bisync() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("")));
        runner.push(Ok(output("foreign-owner")));
        assert!(
            prepare_recovery(
                &connection,
                &plan,
                "2026-10-11",
                &runner,
                CancellationToken::new()
            )
            .await
            .unwrap_err()
            .contains("another connection")
        );
        assert_eq!(runner.requests().len(), 2);
        #[cfg(unix)]
        {
            let target = root.path().join("external");
            fs::create_dir_all(&target).unwrap();
            let link = plan.recovery_directory.join("2026-10-12");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            let runner = FakeCommandRunner::default();
            assert!(
                prepare_recovery(
                    &connection,
                    &plan,
                    "2026-10-12",
                    &runner,
                    CancellationToken::new()
                )
                .await
                .unwrap_err()
                .contains("symlink")
            );
            assert!(runner.requests().is_empty());
            assert!(!target.join(OWNER_MARKER).exists());
        }
    }

    #[tokio::test]
    async fn incomplete_retention_listing_or_owner_cannot_authorize_purge() {
        for owner_read in [false, true] {
            for invalid_utf8 in [false, true] {
                let root = tempfile::tempdir().unwrap();
                let connection = connection(root.path());
                let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
                fs::create_dir_all(&plan.recovery_directory).unwrap();
                let runner = FakeCommandRunner::default();
                let mut incomplete = output(if owner_read { "" } else { "2000-01-01/" });
                if owner_read {
                    incomplete.stdout.text = connection.id.to_string();
                    runner.push(Ok(output("2000-01-01/")));
                }
                incomplete.stdout.truncated = !invalid_utf8;
                incomplete.stdout.invalid_utf8 = invalid_utf8;
                runner.push(Ok(incomplete));
                let warnings =
                    cleanup_retention(&connection, &plan, &runner, CancellationToken::new()).await;
                assert!(!warnings.is_empty());
                assert_eq!(runner.requests().len(), if owner_read { 2 } else { 1 });
                assert!(
                    !runner
                        .requests()
                        .iter()
                        .any(|r| r.sanitized_command().contains(" purge "))
                );
            }
        }
    }

    #[tokio::test]
    async fn unowned_or_incomplete_recovery_listing_never_publishes_owner() {
        for case in 0..6 {
            let root = tempfile::tempdir().unwrap();
            let connection = connection(root.path());
            let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
            fs::create_dir_all(&plan.work_directory).unwrap();
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output("")));
            runner.push(Err(CommandError::NonZero {
                command: "cat".into(),
                code: Some(1),
                stdout: output("").stdout,
                stderr: output("").stderr,
                attempts: 1,
            }));
            let mut listing = output(if case == 0 { "user-data.txt" } else { "" });
            listing.stdout.truncated = case == 1;
            listing.stdout.invalid_utf8 = case == 2;
            if case >= 3 {
                runner.push(Err(match case {
                    3 => CommandError::Timeout {
                        command: "lsf".into(),
                        timeout: Duration::from_secs(60),
                    },
                    4 => CommandError::Cancelled {
                        command: "lsf".into(),
                    },
                    _ => CommandError::NonZero {
                        command: "lsf".into(),
                        code: Some(9),
                        stdout: output("").stdout,
                        stderr: output("").stderr,
                        attempts: 1,
                    },
                }));
            } else {
                runner.push(Ok(listing));
            }
            assert!(
                prepare_recovery(
                    &connection,
                    &plan,
                    "2026-10-11",
                    &runner,
                    CancellationToken::new()
                )
                .await
                .is_err()
            );
            assert_eq!(runner.requests().len(), 3);
        }
    }

    #[cfg(unix)]
    #[test]
    fn local_anchors_and_owners_reject_missing_changed_and_linked_data() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let mut plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
        plan.access_marker_name = Some("anchor".into());
        fs::create_dir_all(&plan.work_directory).unwrap();
        fs::create_dir_all(&connection.local_path).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized").unwrap();
        assert!(
            ensure_local_access_marker(&connection, &plan)
                .unwrap_err()
                .contains("missing")
        );
        fs::write(connection.local_path.join("anchor"), "foreign").unwrap();
        assert!(
            ensure_local_access_marker(&connection, &plan)
                .unwrap_err()
                .contains("changed")
        );
        fs::remove_file(connection.local_path.join("anchor")).unwrap();
        std::os::unix::fs::symlink(
            root.path().join("missing"),
            connection.local_path.join("anchor"),
        )
        .unwrap();
        assert!(ensure_local_access_marker(&connection, &plan).is_err());
        let recovery = root.path().join("owner-test");
        fs::create_dir_all(&recovery).unwrap();
        std::os::unix::fs::symlink(root.path().join("missing"), recovery.join(OWNER_MARKER))
            .unwrap();
        assert!(
            validate_local_owner(&recovery, connection.id)
                .unwrap_err()
                .contains("symlink")
        );
        fs::remove_file(recovery.join(OWNER_MARKER)).unwrap();
        fs::write(recovery.join(OWNER_MARKER), "foreign").unwrap();
        assert!(
            validate_local_owner(&recovery, connection.id)
                .unwrap_err()
                .contains("another connection")
        );
    }

    #[test]
    fn modes_are_fixed_and_round_trip() {
        for mode in [
            BisyncMode::InitialPreview,
            BisyncMode::Preview,
            BisyncMode::InitialSync,
            BisyncMode::Sync,
            BisyncMode::Scheduled,
        ] {
            assert_eq!(BisyncMode::parse(mode.as_str()), Some(mode));
        }
        assert_eq!(BisyncMode::parse("shell"), None);
    }

    #[test]
    fn process_lock_rejects_overlap_and_recovers_after_drop() {
        let root = tempfile::tempdir().unwrap();
        let id = ConnectionId::new();
        let first = OperationLock::acquire(root.path(), id).unwrap();
        assert!(OperationLock::acquire(root.path(), id).is_err());
        drop(first);
        OperationLock::acquire(root.path(), id).unwrap();
    }

    #[tokio::test]
    async fn overlapping_run_is_rejected_before_mirror_inputs_are_created() {
        let root = tempfile::tempdir().unwrap();
        let work_root = root.path().join("state");
        let connection = connection(root.path());
        let _guard = acquire_operation_guard(&work_root, connection.id).unwrap();
        let error = run(
            &connection,
            BisyncMode::InitialPreview,
            &work_root,
            &FakeCommandRunner::default(),
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
        assert!(error.contains("already running"));
        assert!(!connection.local_path.exists());
    }

    #[test]
    fn utc_names_and_strict_dates_are_bounded() {
        let now = UNIX_EPOCH + Duration::from_secs(1_791_740_000);
        let (day, stamp) = utc_day_and_stamp(now).unwrap();
        assert!(strict_day(&day));
        assert_eq!(stamp.len(), 25);
        assert!(!strict_day("2026-1-01"));
        assert!(!strict_day("2026-02-30"));
        assert!(strict_day("2024-02-29"));
        assert!(!strict_day("../../2026"));
    }

    #[test]
    fn scheduled_progress_requires_a_valid_invocation_and_publishes_it_atomically() {
        let root = tempfile::tempdir().unwrap();
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection(root.path()), &work_root).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        assert!(prepare_scheduled_progress_for_invocation(&plan, "short").is_err());
        let invocation = "0123456789abcdef0123456789abcdef";
        prepare_scheduled_progress_for_invocation(&plan, invocation).unwrap();
        let value: serde_json::Value = serde_json::from_slice(
            &fs::read(live_progress_state_path(&plan.work_directory)).unwrap(),
        )
        .unwrap();
        assert_eq!(value["invocation_id"], invocation);
        assert_eq!(
            fs::read(live_progress_log_path(&plan.work_directory)).unwrap(),
            b""
        );
    }

    #[test]
    fn command_failures_do_not_expose_remote_or_local_paths() {
        let error = CommandError::NonZero {
            command: "rclone bisync secret_remote:path /home/person/private".into(),
            code: Some(7),
            stdout: output("provider secret").stdout,
            stderr: output("/home/person/private").stderr,
            attempts: 1,
        };
        let message = command_failure("bisync sync", error);
        assert_eq!(message, "bisync sync failed with exit status 7");
        assert!(!message.contains("secret_remote"));
        assert!(!message.contains("/home"));
    }

    #[test]
    fn all_five_modes_have_fixed_typed_rclone_arguments() {
        let root = tempfile::tempdir().unwrap();
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection(root.path()), &work_root).unwrap();
        for (mode, required, forbidden) in [
            (
                BisyncMode::InitialPreview,
                vec!["--resync", "--dry-run"],
                vec!["--use-json-log"],
            ),
            (
                BisyncMode::Preview,
                vec!["--dry-run"],
                vec!["--resync", "--use-json-log"],
            ),
            (
                BisyncMode::InitialSync,
                vec!["--resync"],
                vec!["--dry-run", "--use-json-log"],
            ),
            (
                BisyncMode::Sync,
                vec![],
                vec!["--resync", "--dry-run", "--use-json-log"],
            ),
            (
                BisyncMode::Scheduled,
                vec!["--use-json-log", "--log-file"],
                vec!["--resync", "--dry-run"],
            ),
        ] {
            let request =
                bisync_request(&plan, mode, "2026-10-11", "20261011T120000.000000000").unwrap();
            assert_eq!(request.executable, Executable::Rclone);
            let command = request.sanitized_command();
            assert!(command.starts_with("rclone --contimeout 10s --timeout 30s bisync "));
            assert!(command.contains("--backup-dir1"));
            assert!(command.contains("/2026-10-11"));
            assert!(command.contains("--suffix -20261011T120000.000000000"));
            for argument in required {
                assert!(command.contains(argument), "{mode:?}: {argument}");
            }
            for argument in forbidden {
                assert!(!command.contains(argument), "{mode:?}: {argument}");
            }
        }
    }

    #[tokio::test]
    async fn sync_orders_owned_recovery_before_bisync_and_retention() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let runner = FakeCommandRunner::default();
        // mkdir recovery, absent owner marker, empty recovery listing,
        // copy owner marker, bisync, then empty retention listing.
        runner.push(Ok(output("")));
        runner.push(Err(crate::process::CommandError::NonZero {
            command: "rclone cat".into(),
            code: Some(1),
            stdout: output("").stdout,
            stderr: output("").stderr,
            attempts: 1,
        }));
        for _ in 0..4 {
            runner.push(Ok(output("")));
        }

        run(
            &connection,
            BisyncMode::Sync,
            &work_root,
            &runner,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let commands = runner
            .requests()
            .iter()
            .map(CommandRequest::sanitized_command)
            .collect::<Vec<_>>();
        assert!(commands[0].contains(" mkdir "));
        assert!(commands[1].contains(" cat "));
        assert!(commands[2].contains(" lsf "));
        assert!(commands[3].contains(" copyto "));
        assert!(commands[4].contains(" bisync "));
        assert!(commands[5].contains(" lsf --dirs-only"));
        let state = read_state(&state_path(&work_root, connection.id))
            .unwrap()
            .unwrap();
        assert_eq!(state.operation, OfflineOperation::Sync);
        assert_eq!(state.state, OfflineOperationState::Succeeded);
    }

    #[tokio::test]
    async fn failed_bisync_records_failure_and_never_starts_retention() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("")));
        runner.push(Err(CommandError::NonZero {
            command: "rclone cat".into(),
            code: Some(1),
            stdout: output("").stdout,
            stderr: output("").stderr,
            attempts: 1,
        }));
        runner.push(Ok(output("")));
        runner.push(Ok(output("")));
        runner.push(Err(CommandError::NonZero {
            command: "rclone bisync secret:path /home/person/private".into(),
            code: Some(9),
            stdout: output("provider details").stdout,
            stderr: output("private path").stderr,
            attempts: 1,
        }));

        let error = run(
            &connection,
            BisyncMode::Sync,
            &work_root,
            &runner,
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
        assert_eq!(error, "bisync sync failed with exit status 9");
        assert_eq!(runner.requests().len(), 5);
        let command = runner.requests()[4].sanitized_command();
        assert!(command.contains("--max-delete 50"));
        assert!(!command.contains("--force"));
        assert!(!command.contains("--resync"));
        assert!(initial_sync_marker(&plan).exists());
        assert!(
            !runner
                .requests()
                .iter()
                .any(|request| request.sanitized_command().contains("lsf --dirs-only"))
        );
        let state = read_state(&state_path(&work_root, connection.id))
            .unwrap()
            .unwrap();
        assert_eq!(state.state, OfflineOperationState::Failed);
    }

    #[tokio::test]
    async fn retention_listing_failure_warns_without_failing_successful_sync() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let runner = FakeCommandRunner::default();
        push_successful_sync_prefix(&runner);
        runner.push(Err(CommandError::NonZero {
            command: "rclone lsf secret_remote:private".into(),
            code: Some(5),
            stdout: output("credential-secret").stdout,
            stderr: output("/home/person/private").stderr,
            attempts: 1,
        }));

        let outcome = run(
            &connection,
            BisyncMode::Sync,
            &work_root,
            &runner,
            CancellationToken::new(),
        )
        .await
        .expect("cleanup failure must not fail a successful sync");

        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("Remote retention cleanup"));
        assert!(outcome.warnings[0].contains("run Sync Now again"));
        assert!(!outcome.warnings[0].contains("secret_remote"));
        assert!(!outcome.warnings[0].contains("/home/person"));
        let state = read_state(&state_path(&work_root, connection.id))
            .unwrap()
            .unwrap();
        assert_eq!(state.state, OfflineOperationState::Succeeded);
        assert_eq!(state.warnings, outcome.warnings);
    }

    #[tokio::test]
    async fn local_retention_inspection_failure_is_actionable_and_non_destructive() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::write(&plan.recovery_directory, b"not a directory").unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("")));

        let warnings =
            cleanup_retention(&connection, &plan, &runner, CancellationToken::new()).await;

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("could not inspect the recovery directory"));
        assert!(warnings[0].contains("run Sync Now again"));
        assert_eq!(
            fs::read(&plan.recovery_directory).unwrap(),
            b"not a directory"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn retention_never_follows_a_dated_local_symlink() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
        fs::create_dir_all(&plan.recovery_directory).unwrap();
        let outside = root.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("keep.txt"), b"preserve").unwrap();
        let dated_link = plan.recovery_directory.join("2000-01-01");
        symlink(&outside, &dated_link).unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("")));

        let warnings =
            cleanup_retention(&connection, &plan, &runner, CancellationToken::new()).await;

        assert!(dated_link.is_symlink());
        assert_eq!(fs::read(outside.join("keep.txt")).unwrap(), b"preserve");
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("ownership could not be verified"))
        );
    }

    #[tokio::test]
    async fn retention_preserves_unowned_local_data_and_reports_ownership_warning() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::create_dir_all(&plan.recovery_directory).unwrap();
        fs::write(
            plan.recovery_directory.join(OWNER_MARKER),
            format!("{}\n", connection.id),
        )
        .unwrap();
        let unowned = plan.recovery_directory.join("2000-01-01");
        fs::create_dir_all(&unowned).unwrap();
        fs::write(unowned.join("keep.txt"), b"unowned").unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let runner = FakeCommandRunner::default();
        push_successful_sync_prefix(&runner);
        runner.push(Ok(output("")));

        let outcome = run(
            &connection,
            BisyncMode::Sync,
            &work_root,
            &runner,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(unowned.join("keep.txt").is_file());
        assert!(
            outcome
                .warnings
                .iter()
                .any(|warning| warning.contains("ownership could not be verified"))
        );
    }

    #[tokio::test]
    async fn retention_purge_failure_preserves_success_and_is_bounded() {
        let root = tempfile::tempdir().unwrap();
        let connection = connection(root.path());
        let work_root = root.path().join("state");
        let plan = rclone_bisync_plan(&connection, &work_root).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let runner = FakeCommandRunner::default();
        push_successful_sync_prefix(&runner);
        runner.push(Ok(output("2000-01-01/\n")));
        runner.push(Ok(output(&format!("{}\n", connection.id))));
        runner.push(Err(CommandError::NonZero {
            command: "rclone purge secret_remote:private".into(),
            code: Some(9),
            stdout: output("token=secret").stdout,
            stderr: output("private path").stderr,
            attempts: 1,
        }));

        let outcome = run(
            &connection,
            BisyncMode::Sync,
            &work_root,
            &runner,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("could not remove 1"));
        assert!(!outcome.warnings[0].contains("secret"));
        assert!(outcome.warnings[0].len() < 300);
    }
    fn onedrive_connection(root: &Path) -> Connection {
        let mut connection = connection(root);
        connection.provider = Provider::OneDrive;
        let ConnectionMode::OfflineMirror(ref mut options) = connection.mode else {
            unreachable!()
        };
        options.engine = Some(crate::model::OfflineEngine::Rclone);
        connection.onedrive_identity = Some(crate::model::OneDriveIdentity {
            drive_id: "synthetic-primary".into(),
            drive_type: crate::model::OneDriveDriveType::Personal,
        });
        connection
    }

    struct OneDriveGraph;
    impl crate::onedrive::Graph for OneDriveGraph {
        fn get<'a>(
            &'a self,
            target: crate::onedrive::GraphTarget<'a>,
            _token: &'a str,
            _cancel: CancellationToken,
        ) -> crate::onedrive::GraphFuture<'a> {
            Box::pin(async move {
                Ok(match target {
                    crate::onedrive::GraphTarget::PrimaryDrive => {
                        serde_json::json!({"id":"synthetic-primary","driveType":"personal"})
                    }
                    crate::onedrive::GraphTarget::Folder { .. } => {
                        serde_json::json!({"id":"folder","folder":{},"parentReference":{"driveId":"synthetic-primary"}})
                    }
                    _ => panic!("recurring guard must not query an account profile"),
                })
            })
        }
    }

    #[tokio::test]
    async fn onedrive_initial_preview_runs_required_guard_without_creating_anchor() {
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_connection(root.path());
        let work = root.path().join("work");
        let plan = rclone_bisync_plan(&connection, &work).unwrap();
        let runner = FakeCommandRunner::default();
        for text in [
            "{}",
            r#"{"test_remote":{"type":"onedrive","drive_type":"personal","drive_id":"synthetic-primary","token":{"access_token":"synthetic"}}}"#,
            r#"{"IsDir":true}"#,
            "",
            "[]",
            "[]",
        ] {
            runner.push(Ok(output(text)));
        }
        run_plan(
            &connection,
            BisyncMode::InitialPreview,
            &work,
            &plan,
            &runner,
            CancellationToken::new(),
            Some(&OneDriveGraph),
        )
        .await
        .unwrap();
        assert!(initial_preview_marker(&plan).is_file());
        assert!(!initial_sync_marker(&plan).exists());
        assert!(
            !connection
                .local_path
                .join(plan.access_marker_name.as_ref().unwrap())
                .exists()
        );
        assert_eq!(runner.requests().len(), 6);
        assert!(
            runner
                .requests()
                .get(3)
                .unwrap()
                .sanitized_command()
                .contains("--dry-run")
        );
    }

    #[tokio::test]
    async fn onedrive_initial_checksum_failure_never_initializes_or_repeats_bisync() {
        for success in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let connection = onedrive_connection(root.path());
            let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
            prepare_inputs(&connection, &plan, BisyncMode::InitialSync).unwrap();
            let runner = FakeCommandRunner::default();
            push_successful_sync_prefix(&runner);
            runner.push(Ok(output(&connection.id.to_string())));
            runner.push(if success {
                Ok(output(""))
            } else {
                Err(CommandError::NonZero {
                    command: "synthetic".into(),
                    code: Some(1),
                    stdout: output("").stdout,
                    stderr: output("synthetic-private-diagnostic").stdout,
                    attempts: 1,
                })
            });
            if success {
                runner.push(Ok(output("")));
            }
            let result = run_transaction(
                &connection,
                &plan,
                BisyncMode::InitialSync,
                &runner,
                CancellationToken::new(),
                &LocalMirrorPaths::native(&connection).unwrap(),
            )
            .await;
            assert_eq!(result.is_ok(), success);
            assert_eq!(initial_sync_marker(&plan).is_file(), success);
            if let Err(error) = result {
                assert!(!error.contains("synthetic-private-diagnostic"));
            }
            let requests = runner.requests();
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.sanitized_command().contains(" bisync "))
                    .count(),
                1
            );
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.sanitized_command().contains(" check "))
                    .count(),
                1
            );
            assert!(
                fs::read_to_string(
                    connection
                        .local_path
                        .join(plan.access_marker_name.as_ref().unwrap())
                )
                .unwrap()
                .contains(&connection.id.to_string())
            );
        }
    }

    #[tokio::test]
    async fn onedrive_anchor_guards_reject_missing_changed_and_linked_markers() {
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_connection(root.path());
        let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
        prepare_inputs(&connection, &plan, BisyncMode::InitialSync).unwrap();
        write_atomic(&initial_sync_marker(&plan), b"initialized\n").unwrap();
        let path = connection
            .local_path
            .join(plan.access_marker_name.as_ref().unwrap());
        fs::write(&path, "wrong-uuid\n").unwrap();
        assert!(ensure_local_access_marker(&connection, &plan).is_err());
        fs::remove_file(&path).unwrap();
        assert!(ensure_local_access_marker(&connection, &plan).is_err());
        std::os::unix::fs::symlink(root.path(), &path).unwrap();
        assert!(ensure_local_access_marker(&connection, &plan).is_err());
        for text in ["wrong-uuid", ""] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(text)));
            assert!(
                validate_remote_access_marker(
                    &connection,
                    &plan,
                    BisyncMode::Scheduled,
                    &runner,
                    CancellationToken::new()
                )
                .await
                .is_err()
            );
            assert_eq!(runner.requests().len(), 1);
        }
    }
    #[tokio::test]
    async fn onedrive_runtime_normal_manual_sync_requires_one_checksum_check() {
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_connection(root.path());
        let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
        prepare_inputs(&connection, &plan, BisyncMode::InitialSync).unwrap();
        for success in [false, true] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(&connection.id.to_string())));
            push_successful_sync_prefix(&runner);
            runner.push(Ok(output(&connection.id.to_string())));
            runner.push(if success {
                Ok(output(""))
            } else {
                Err(CommandError::Timeout {
                    command: "synthetic".into(),
                    timeout: Duration::from_secs(60),
                })
            });
            if success {
                runner.push(Ok(output("")));
            }
            let result = run_transaction(
                &connection,
                &plan,
                BisyncMode::Sync,
                &runner,
                CancellationToken::new(),
                &LocalMirrorPaths::native(&connection).unwrap(),
            )
            .await;
            assert_eq!(result.is_ok(), success);
            let commands = runner
                .requests()
                .iter()
                .map(CommandRequest::sanitized_command)
                .collect::<Vec<_>>();
            assert_eq!(
                commands
                    .iter()
                    .filter(|command| command.contains(" bisync "))
                    .count(),
                1
            );
            assert_eq!(
                commands
                    .iter()
                    .filter(|command| command.contains(" check "))
                    .count(),
                1
            );
            assert!(!commands.iter().any(|command| command.contains("--resync")));
        }
    }
}
