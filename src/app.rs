// SPDX-License-Identifier: MIT

mod sftp;
use sftp::{SftpAuth, SftpDraft, SftpField};

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use crate::fl;
use crate::runtime_ipc::{self, Command as RuntimeCommand};
use cosmic::dialog::file_chooser;
use cosmic::iced::Background;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{Alignment, Color, Length, Limits, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic_ext_applet_mounter::bisync_coordinator::{self, BisyncMode, BisyncOutcome};
use cosmic_ext_applet_mounter::config::{
    APP_ID, AppConfigStorage, Config, ConfigDocument, MAX_PRELOAD_DEPTH, MAX_PRELOAD_SECONDS,
    MIN_PRELOAD_DEPTH, MIN_PRELOAD_SECONDS,
};
use cosmic_ext_applet_mounter::controller::{
    AggregateKind, ConnectionRowState, ControllerSnapshot, aggregate_label, decide_operation,
    operation_label, provider_label, restore, status_label,
};
use cosmic_ext_applet_mounter::directory_preload::{
    self, DirectoryPreloadJob, DirectoryPreloadOutcome,
};
use cosmic_ext_applet_mounter::host_inspection::HostInspection;
use cosmic_ext_applet_mounter::import::{
    ImportPreview, ImportReplacementPlan, default_scan_directory, parse_rclone_backends,
    preview_import, replacement_plan, scan_legacy_units,
};
use cosmic_ext_applet_mounter::mirror_script;
use cosmic_ext_applet_mounter::model::{
    AccessMode, Connection, ConnectionId, ConnectionMode, ConnectionStatus, OfflineEngine,
    OfflineMirrorConfig, OneDriveIdentity, OnlineMountConfig, OnlineMountStatus, Operation,
    PreloadPolicy, PreloadSettings, Provider, SmbPreloadOverride, TuningProfile, VpnKind,
    VpnProfile, VpnProfileId,
};
use cosmic_ext_applet_mounter::mount_guard;
#[cfg(test)]
use cosmic_ext_applet_mounter::mounts::ProcMountTable;
use cosmic_ext_applet_mounter::mounts::{
    MountEntry, MountTable, MountTableError, SyncRuntimeState,
};
use cosmic_ext_applet_mounter::offline_coordinator::{
    ManualOperationControl, OfflineCoordinatorState, OfflineOperation, OfflineOperationState,
};
use cosmic_ext_applet_mounter::pending_count;
#[cfg(test)]
#[allow(dead_code)]
use cosmic_ext_applet_mounter::process::redact_text;
use cosmic_ext_applet_mounter::process::{
    CommandError, CommandOutput, CommandRequest, CommandRunner, Executable, RuntimeCommandRunner,
};
use cosmic_ext_applet_mounter::providers::{
    CommandRcloneProvider, OnedriverAuthState, ProviderError, clean_unmount_request,
    lazy_unmount_request, onedriver_auth_state_for_plan, onedriver_mount_plan, rclone_mount_plan,
};
use cosmic_ext_applet_mounter::rclone_refresh::{self, RcloneRefreshJob, RcloneRefreshOutcome};
use cosmic_ext_applet_mounter::services::{
    ActiveState, DbusSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction,
    SystemdManager, UnitController, UnitDocument, UnitFileState, UnitKind, UnitName, UnitStatus,
    UnitStore, UnitValidator, start_managed_online_service, systemd_user_status_batch,
    systemd_user_status_blocking,
};
use cosmic_ext_applet_mounter::sharepoint;
use cosmic_ext_applet_mounter::sharepoint_setup::{self, SharePointSetup, SharePointSetupOutcome};
use cosmic_ext_applet_mounter::sync::{
    OneDriveIsolationReport, SyncDecision, SyncDecisionRejection, SyncReadiness, SyncRequest,
    SyncTrigger, one_drive_auth_files_request, one_drive_auth_request,
    one_drive_initial_sync_request, one_drive_mirror_plan, one_drive_preview_request,
    one_drive_sync_request, parse_preview, rclone_bisync_filter_file, rclone_bisync_plan,
    sync_now_request,
};
use cosmic_ext_applet_mounter::vpn::{
    DbusNetworkManagerVpn, InProcessReadinessProbe, NetworkManagerVpn, VpnShutdownDecision,
    readiness_report, shutdown_decision,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const POPUP_CONNECTION_LIST_MAX_HEIGHT: f32 = 640.0;
const POPUP_CONNECTION_NAME_MAX_CHARS: usize = 36;
const POPUP_CONNECTION_ROW_VERTICAL_PADDING: u16 = 0;
const POPUP_CONNECTION_ROW_HEIGHT: f32 = 50.0;
const POPUP_EMPTY_ROW_HEIGHT: f32 = 40.0;
const POPUP_ACTION_HORIZONTAL_PADDING: u16 = 24;
const POPUP_CONNECTION_ROW_HORIZONTAL_PADDING: u16 = 0;
const POPUP_NOTICE_TIMEOUT: Duration = Duration::from_secs(10);
const POPUP_HEALTHY_AGGREGATE_HIDE_AFTER: Duration = Duration::from_secs(5);
const PENDING_COUNT_STALE_AFTER: Duration = Duration::from_secs(15);
const CLEAN_UNMOUNT_SETTLE_TIMEOUT: Duration = Duration::from_secs(2);
const REPAIR_CONFIRM_MIN_DELAY: Duration = Duration::from_secs(1);
const REPAIR_CONFIRM_MAX_DELAY: Duration = Duration::from_secs(30);
const RUNTIME_OPERATION_COMPLETION_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepairConfirmationDecision {
    Arm,
    Wait,
    Confirm,
}

fn repair_confirmation_decision(
    pending: Option<(ConnectionId, Instant)>,
    connection_id: ConnectionId,
    now: Instant,
) -> RepairConfirmationDecision {
    let Some((pending_id, armed_at)) = pending else {
        return RepairConfirmationDecision::Arm;
    };
    if pending_id != connection_id {
        return RepairConfirmationDecision::Arm;
    }
    let elapsed = now.duration_since(armed_at);
    if elapsed < REPAIR_CONFIRM_MIN_DELAY {
        RepairConfirmationDecision::Wait
    } else if elapsed <= REPAIR_CONFIRM_MAX_DELAY {
        RepairConfirmationDecision::Confirm
    } else {
        RepairConfirmationDecision::Arm
    }
}

fn popup_notice_expired(
    notice_started_at: Instant,
    popup_opened_at: Option<Instant>,
    now: Instant,
) -> bool {
    let visible_since = popup_opened_at
        .filter(|opened_at| *opened_at > notice_started_at)
        .unwrap_or(notice_started_at);
    now.duration_since(visible_since) >= POPUP_NOTICE_TIMEOUT
}
const CLEAN_UNMOUNT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SETTINGS_SECTION_TITLE_WIDTH: f32 = 150.0;
const SETTINGS_SECTION_TITLE_TOP_PADDING: u16 = 8;
const SETTINGS_RCLONE_REMOTE_BUTTONS_PER_ROW: usize = 3;
const SETTINGS_DISABLE_BUTTON_ACTIVE_LIGHTENING: f32 = 0.35;
const SETTINGS_DISABLE_BUTTON_HOVER_LIGHTENING: f32 = 0.25;
const SETTINGS_DISABLE_BUTTON_PRESSED_LIGHTENING: f32 = 0.15;
const SETTINGS_DISABLE_BUTTON_DISABLED_LIGHTENING: f32 = 0.15;
const DOCUMENTS_DESTINATION: &str = "org.freedesktop.portal.Documents";
const DOCUMENTS_PATH: &str = "/org/freedesktop/portal/documents";
const DOCUMENTS_INTERFACE: &str = "org.freedesktop.portal.Documents";
const DOCUMENTS_GET_HOST_PATHS_VERSION: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLaunchMode {
    Applet,
    GeneralSettings,
    AddConnection,
    ModifyConnection(ConnectionId),
    ImportLegacy,
    ReorderConnections,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum WindowMode {
    GeneralSettings,
    #[default]
    AddConnection,
    ModifyConnection(ConnectionId),
    ImportLegacy,
    ReorderConnections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadProvider {
    GoogleDrive,
    OneDrive,
    Box,
    SharePoint,
    Smb,
    Sftp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreloadPolicyDraft {
    enabled: bool,
    maximum_seconds: String,
    maximum_depth: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreloadSettingsDraft {
    google_drive: PreloadPolicyDraft,
    onedrive: PreloadPolicyDraft,
    box_provider: PreloadPolicyDraft,
    sharepoint: PreloadPolicyDraft,
    smb: PreloadPolicyDraft,
    sftp: PreloadPolicyDraft,
}

impl From<PreloadPolicy> for PreloadPolicyDraft {
    fn from(value: PreloadPolicy) -> Self {
        Self {
            enabled: value.enabled,
            maximum_seconds: value.maximum_seconds.to_string(),
            maximum_depth: value.maximum_depth.map(|depth| depth.to_string()),
        }
    }
}

impl Default for PreloadSettingsDraft {
    fn default() -> Self {
        PreloadSettings::default().into()
    }
}

impl From<PreloadSettings> for PreloadSettingsDraft {
    fn from(settings: PreloadSettings) -> Self {
        Self {
            google_drive: settings.google_drive.into(),
            onedrive: settings.onedrive.into(),
            box_provider: settings.box_provider.into(),
            sharepoint: settings.sharepoint.into(),
            smb: settings.smb.into(),
            sftp: settings.sftp.into(),
        }
    }
}

impl PreloadSettingsDraft {
    fn policy_mut(&mut self, provider: PreloadProvider) -> &mut PreloadPolicyDraft {
        match provider {
            PreloadProvider::GoogleDrive => &mut self.google_drive,
            PreloadProvider::OneDrive => &mut self.onedrive,
            PreloadProvider::Box => &mut self.box_provider,
            PreloadProvider::SharePoint => &mut self.sharepoint,
            PreloadProvider::Smb => &mut self.smb,
            PreloadProvider::Sftp => &mut self.sftp,
        }
    }

    fn validated(&self) -> Result<PreloadSettings, String> {
        fn policy(draft: &PreloadPolicyDraft) -> Result<PreloadPolicy, String> {
            let maximum_seconds = draft.maximum_seconds.parse::<u64>().map_err(|_| {
                fl!(
                    "validation-preload-time",
                    min = MIN_PRELOAD_SECONDS,
                    max = MAX_PRELOAD_SECONDS
                )
            })?;
            if !(MIN_PRELOAD_SECONDS..=MAX_PRELOAD_SECONDS).contains(&maximum_seconds) {
                return Err(fl!(
                    "validation-preload-time",
                    min = MIN_PRELOAD_SECONDS,
                    max = MAX_PRELOAD_SECONDS
                ));
            }
            let maximum_depth = draft
                .maximum_depth
                .as_ref()
                .map(|value| {
                    let depth = value.parse::<u8>().map_err(|_| {
                        fl!(
                            "validation-preload-depth",
                            min = MIN_PRELOAD_DEPTH,
                            max = MAX_PRELOAD_DEPTH
                        )
                    })?;
                    if !(MIN_PRELOAD_DEPTH..=MAX_PRELOAD_DEPTH).contains(&depth) {
                        return Err(fl!(
                            "validation-preload-depth",
                            min = MIN_PRELOAD_DEPTH,
                            max = MAX_PRELOAD_DEPTH
                        ));
                    }
                    Ok(depth)
                })
                .transpose()?;
            Ok(PreloadPolicy {
                enabled: draft.enabled,
                maximum_seconds,
                maximum_depth,
            })
        }
        Ok(PreloadSettings {
            google_drive: policy(&self.google_drive)?,
            onedrive: policy(&self.onedrive)?,
            box_provider: policy(&self.box_provider)?,
            sharepoint: policy(&self.sharepoint)?,
            smb: policy(&self.smb)?,
            sftp: policy(&self.sftp)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectionDraft {
    id: Option<ConnectionId>,
    name: String,
    provider: Provider,
    access_mode: AccessMode,
    offline_engine: Option<OfflineEngine>,
    legacy_onedrive: bool,
    onedrive_identity: Option<OneDriveIdentity>,
    onedrive_drive_type: cosmic_ext_applet_mounter::model::OneDriveDriveType,
    onedrive_expected_account: String,
    remote_reference: String,
    remote_subpath: String,
    sharepoint_library_url: String,
    teams_drive_id: String,
    google_client_id: String,
    google_client_secret: String,
    smb_host: String,
    smb_user: String,
    smb_domain: String,
    smb_password: String,
    sftp: SftpDraft,
    local_path: String,
    enabled: bool,
    start_at_login: bool,
    cache_limit_gib: String,
    sync_interval_minutes: String,
    sync_on_metered: bool,
    recovery_directory: String,
    vpn_profile_id: Option<VpnProfileId>,
    disconnect_vpn_when_unused: bool,
    connection_preload_use_global: bool,
    connection_preload_enabled: bool,
    connection_preload_seconds: String,
    connection_preload_depth: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcloneDraftRemote {
    name: String,
    backend: String,
    drive_type: Option<cosmic_ext_applet_mounter::model::OneDriveDriveType>,
}

fn draft_is_onedrive_rclone(draft: &ConnectionDraft) -> bool {
    draft.provider == Provider::OneDrive
        && draft.access_mode == AccessMode::OfflineMirror
        && !draft.legacy_onedrive
}

fn editor_remote_matches(
    draft: &ConnectionDraft,
    provider: Provider,
    remote: &RcloneDraftRemote,
) -> bool {
    if provider == Provider::OneDrive {
        return draft_is_onedrive_rclone(draft)
            && remote.backend == "onedrive"
            && remote.drive_type == Some(draft.onedrive_drive_type);
    }
    rclone_backend_name(provider) == Some(remote.backend.as_str())
        && (provider != Provider::Teams || remote.drive_type.is_none())
}

#[derive(Debug, Clone)]
pub struct OneDriveEditorRequest {
    id: ConnectionId,
    setup: cosmic_ext_applet_mounter::onedrive::RemoteSetup,
    create: bool,
}

impl OneDriveEditorRequest {
    fn from_draft(
        draft: &ConnectionDraft,
        saved: &[Connection],
        create: bool,
    ) -> Result<Self, String> {
        if !draft_is_onedrive_rclone(draft) {
            return Err(fl!("notice-onedrive-only"));
        }
        let id = draft.id.ok_or_else(|| fl!("onedrive-invalid-setup"))?;
        let mut setup = cosmic_ext_applet_mounter::onedrive::RemoteSetup::new(
            &draft.remote_reference,
            draft.onedrive_drive_type,
            &draft.onedrive_expected_account,
        )
        .map_err(|error| error.to_string())?;
        setup.remote_subpath = (!draft.remote_subpath.trim().is_empty())
            .then(|| draft.remote_subpath.trim().to_owned());
        // Never accept the editable draft as the authority for reconnect identity.
        setup.previous_identity = match saved.iter().find(|connection| connection.id == id) {
            Some(connection) if is_onedrive_rclone_mirror(connection) => Some(
                connection
                    .onedrive_identity
                    .clone()
                    .ok_or_else(|| fl!("onedrive-binding-changed"))?,
            ),
            Some(_) => return Err(fl!("onedrive-binding-changed")),
            None => None,
        };
        if setup
            .previous_identity
            .as_ref()
            .is_some_and(|identity| identity.drive_type != setup.drive_type)
        {
            return Err(fl!("onedrive-wrong-drive"));
        }
        Ok(Self { id, setup, create })
    }

    fn matches(&self, draft: &ConnectionDraft) -> bool {
        draft_is_onedrive_rclone(draft)
            && draft.id == Some(self.id)
            && draft.remote_reference.trim() == self.setup.remote_name
            && draft.remote_subpath.trim() == self.setup.remote_subpath.as_deref().unwrap_or("")
            && draft.onedrive_drive_type == self.setup.drive_type
            && draft.onedrive_expected_account.trim() == self.setup.expected_account
    }
}

#[derive(Debug)]
struct OneDriveEditorPending {
    cancellation: CancellationToken,
}

impl Drop for OneDriveEditorPending {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

fn accept_onedrive_editor_result(
    draft: &mut ConnectionDraft,
    request: &OneDriveEditorRequest,
    result: Result<OneDriveIdentity, String>,
) -> Result<(), String> {
    if !request.matches(draft) {
        return Err(fl!("onedrive-editor-result-discarded"));
    }
    let identity = match result {
        Ok(identity)
            if identity.is_valid()
                && identity.drive_type == request.setup.drive_type
                && request
                    .setup
                    .previous_identity
                    .as_ref()
                    .is_none_or(|saved| saved == &identity) =>
        {
            identity
        }
        Ok(_) => {
            draft.onedrive_identity = None;
            draft.enabled = false;
            return Err(fl!("onedrive-wrong-drive"));
        }
        Err(error) => {
            draft.onedrive_identity = None;
            draft.enabled = false;
            return Err(error);
        }
    };
    draft.offline_engine = Some(OfflineEngine::Rclone);
    draft.onedrive_identity = Some(identity);
    Ok(())
}

fn onedrive_validation_matches_draft(draft: &ConnectionDraft, connection: &Connection) -> bool {
    draft_is_onedrive_rclone(draft)
        && connection_from_draft(draft).is_ok_and(|current| current == *connection)
}

async fn run_onedrive_editor_setup<
    R: CommandRunner,
    G: cosmic_ext_applet_mounter::onedrive::Graph,
>(
    runner: &R,
    graph: &G,
    request: &OneDriveEditorRequest,
    cancellation: CancellationToken,
) -> Result<OneDriveIdentity, String> {
    use cosmic_ext_applet_mounter::onedrive;
    let verified = if request.create {
        onedrive::authenticate(runner, graph, &request.setup, cancellation).await
    } else {
        onedrive::verify_existing(runner, graph, &request.setup, cancellation).await
    }
    .map_err(|error| error.to_string())?;
    Ok(verified.identity)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedDraft {
    connection: Connection,
    summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmbRemoteDetails {
    host: String,
    user: String,
    domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePrefillContext {
    id: Option<ConnectionId>,
    provider: Provider,
    remote: String,
    sftp_revision: u64,
    smb: SmbRemoteDetails,
}

#[derive(Debug, Clone)]
pub struct RemotePrefillResult {
    smb: Option<SmbRemoteDetails>,
    sftp: Option<SftpDraft>,
}

#[derive(Debug, Default, Clone)]
pub struct UnitObservation {
    services: BTreeMap<ConnectionId, UnitStatus>,
    timers: BTreeMap<ConnectionId, UnitStatus>,
    warnings: Option<BTreeMap<ConnectionId, Vec<String>>>,
    coordinator_states: BTreeMap<ConnectionId, OfflineOperationState>,
    failed: BTreeSet<ConnectionId>,
}

impl Default for ConnectionDraft {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            provider: Provider::GoogleDrive,
            access_mode: AccessMode::OnlineMount,
            offline_engine: None,
            legacy_onedrive: false,
            onedrive_identity: None,
            onedrive_drive_type: cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
            onedrive_expected_account: String::new(),
            remote_reference: String::new(),
            remote_subpath: String::new(),
            sharepoint_library_url: String::new(),
            teams_drive_id: String::new(),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            smb_host: String::new(),
            smb_user: std::env::var("USER").unwrap_or_default(),
            smb_domain: "WORKGROUP".into(),
            smb_password: String::new(),
            sftp: SftpDraft::default(),
            local_path: String::new(),
            enabled: true,
            start_at_login: false,
            cache_limit_gib: "20".into(),
            sync_interval_minutes: "15".into(),
            sync_on_metered: false,
            recovery_directory: String::new(),
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            connection_preload_use_global: true,
            connection_preload_enabled: false,
            connection_preload_seconds: "60".into(),
            connection_preload_depth: "3".into(),
        }
    }
}

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    standalone: bool,
    popup: Option<Id>,
    window_mode: WindowMode,
    draft: ConnectionDraft,
    config: Config,
    import_previews: Vec<ImportPreview>,
    rclone_remotes: Vec<RcloneDraftRemote>,
    pending_remove: Option<ConnectionId>,
    pending_repair: Option<(ConnectionId, Instant)>,
    pending_shared_remote_ack: Option<ConnectionId>,
    pending_rclone_remote_remove: Option<String>,
    removing_connection: Option<ConnectionId>,
    removing_rclone_remote: Option<String>,
    sftp_setup_pending: bool,
    sharepoint_setup_pending: bool,
    onedrive_editor_pending: Option<OneDriveEditorPending>,
    onedrive_editor_generation: u64,
    sftp_settings_dirty: bool,
    sftp_settings_revision: u64,
    sftp_update_ack: Option<(String, SftpDraft)>,
    sftp_retry_secret_auth: Option<SftpAuth>,
    onedrive_auth_open_command: String,
    onedrive_auth_response_url: String,
    validated_draft: Option<ValidatedDraft>,
    last_notice: Option<String>,
    last_notice_at: Option<Instant>,
    popup_opened_at: Option<Instant>,
    vpn_ready: BTreeMap<VpnProfileId, bool>,
    network_ready: Option<bool>,
    sleep_status: Option<String>,
    sleep_needs_attention: bool,
    sleep_notice: Option<String>,
    sleep_settings_pending: bool,
    preload_notice: Option<String>,
    preload_settings_pending: bool,
    runtime_owner: bool,
    runtime_available: bool,
    runtime_pending: bool,
    runtime_poll_pending: bool,
    runtime_poll_obsolete: bool,
    runtime_error: Option<String>,
    runtime_operation_revision: u64,
    runtime_operation_completions: Vec<runtime_ipc::OperationCompletion>,
    runtime_routed_operations: BTreeMap<ConnectionId, Operation>,
    runtime_operation_wait: Option<RuntimeOperationWait>,
    rclone_refresh_starting: BTreeSet<ConnectionId>,
    rclone_refresh_jobs: BTreeMap<ConnectionId, u64>,
    preload_draft: PreloadSettingsDraft,
    preload_input_dirty: bool,
    directory_preload_jobs: BTreeMap<ConnectionId, u64>,
    unmount_pending: BTreeSet<ConnectionId>,
    online_repair_required: BTreeSet<ConnectionId>,
    repair_operations_pending: BTreeSet<ConnectionId>,
    pending_counts: BTreeMap<ConnectionId, PendingObservation>,
    pending_poll_inflight: BTreeSet<ConnectionId>,
    pending_poll_generation: u64,
    mirror_operations_pending: BTreeSet<ConnectionId>,
    mirror_operation_controls: BTreeMap<ConnectionId, Arc<ManualOperationControl>>,
    mount_entries: Vec<MountEntry>,
    mount_inspection_pending: bool,
    mount_inspection_obsolete: bool,
    mount_inspection_error: Option<String>,
    rclone_inspection_generation: u64,
    prefill_generation: u64,
    rclone_inspection_cancellation: CancellationToken,
    prefill_cancellation: CancellationToken,
    unit_observation: UnitObservation,
    unit_observation_connections: Vec<Connection>,
    unit_observed_at: Option<Instant>,
    unit_observation_pending: bool,
    unit_observation_invalidated: bool,
    unit_observation_revision: u64,
    unit_observation_cancellation: CancellationToken,
    unit_operations_waiting: BTreeMap<ConnectionId, (Connection, Operation)>,
}

#[derive(Debug, Clone)]
struct PendingObservation {
    files: Option<u64>,
    checked_at: Instant,
    reason: Option<String>,
    source: PendingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeOperationWait {
    connection_id: ConnectionId,
    operation: Operation,
    after_revision: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum PendingSource {
    RcloneOnline,
    RcloneMirrorLive,
    OneDriveRclonePreview { age: Duration },
    OneDriveMirror,
}

#[derive(Debug, Clone)]
pub enum Message {
    UnitObservationTick,
    UnitObservationCompleted(u64, Vec<Connection>, UnitObservation),
    MountInspectionTick,
    MountInspectionCompleted(Result<Vec<MountEntry>, String>),
    RcloneInspectionCompleted(u64, bool, Result<Vec<RcloneDraftRemote>, String>),
    RemotePrefillCompleted(
        u64,
        RemotePrefillContext,
        Result<RemotePrefillResult, String>,
    ),
    ImportInspectionCompleted(Result<BTreeMap<String, Provider>, String>),
    TogglePopup,
    OpenGeneralSettings,
    RuntimeEvent(runtime_ipc::Event),
    RuntimePoll,
    RuntimeResult(RuntimeCommand, Result<runtime_ipc::Status, String>),
    RuntimeRefreshCompleted(runtime_ipc::Reply, BTreeMap<VpnProfileId, bool>),
    OnlineGuardsRefreshed(Vec<String>),
    EditorNotified(Result<runtime_ipc::Status, String>),
    UnmountBeforeSleep(bool),
    RestoreAfterWake(bool),
    PreloadEnabled(PreloadProvider, bool),
    PreloadSecondsInput(PreloadProvider, String),
    PreloadDepthInput(PreloadProvider, String),
    SavePreloadSettings,
    SleepEvent(cosmic_ext_applet_mounter::sleep::Event),
    OpenAddConnection,
    OpenModifyConnection(ConnectionId),
    OpenReorderConnections,
    ReorderConnection(ConnectionId, usize),
    PopupClosed(Id),
    OperationRequested(ConnectionId, Operation),
    OnlineRepairCompleted(ConnectionId, String),
    OfflineRepairCompleted(ConnectionId, String),
    MirrorOperationCompleted(ConnectionId, String),
    ManagedOnlineOperationCompleted(Connection, Operation, Result<(), String>),
    RcloneRefreshStarted(String, ConnectionId, Result<RcloneRefreshJob, String>),
    RcloneRefreshCompleted(
        String,
        ConnectionId,
        u64,
        Result<RcloneRefreshOutcome, String>,
    ),
    DirectoryPreloadCompleted(
        String,
        ConnectionId,
        u64,
        Result<DirectoryPreloadOutcome, String>,
    ),
    DraftProvider(Provider),
    DraftAccessMode(AccessMode),
    DraftName(String),
    DraftRemote(String),
    DraftSubpath(String),
    DraftSharePointLibraryUrl(String),
    DraftGoogleClientId(String),
    DraftGoogleClientSecret(String),
    DraftSftp(SftpField, String),
    DraftSftpAuth(SftpAuth),
    ApplySftpRemote,
    SftpRemoteApplied(String, u64, Option<SftpAuth>, Result<bool, String>),
    DraftSmbHost(String),
    DraftSmbUser(String),
    DraftSmbDomain(String),
    DraftSmbPassword(String),
    DraftLocalPath(String),
    OpenLocalFolderPicker,
    LocalFolderPicked(Result<Option<String>, String>),
    OpenRecoveryFolderPicker,
    RecoveryFolderPicked(Result<Option<String>, String>),
    DraftRecoveryDirectory(String),
    DraftCacheLimit(String),
    DraftSyncInterval(String),
    DraftEnabled(bool),
    DraftSyncOnMetered(bool),
    DraftConnectionPreloadUseGlobal(bool),
    DraftConnectionPreloadEnabled(bool),
    DraftConnectionPreloadSeconds(String),
    DraftConnectionPreloadDepth(String),
    DraftVpn(Option<VpnProfileId>),
    DraftDisconnectVpn(bool),
    DraftOneDriveAuthResponse(String),
    DetectVpns,
    DetectRcloneRemotes,
    ApplyGoogleDriveRcloneRemote,
    GoogleDriveRcloneRemoteApplied(Result<GoogleDriveRemoteApplyResult, String>),
    CreateBoxRcloneRemote,
    BoxRcloneRemoteCreated(Result<String, String>),
    CreateSharePointRcloneRemote,
    SharePointRcloneRemoteCreated(Result<SharePointSetupOutcome, String>),
    CreateSmbRcloneRemote,
    SmbRcloneRemoteCreated(Result<SmbRemoteApplyResult, String>),
    RequestRemoveRcloneRemote(String),
    RcloneRemoteRemoved(Result<String, String>),
    StartOnedriverSetup,
    OnedriverSetupCompleted(Result<String, String>),
    DraftOneDriveType(cosmic_ext_applet_mounter::model::OneDriveDriveType),
    DraftOneDriveAccount(String),
    SetupOneDriveRclone(bool),
    CancelOneDriveRclone,
    OneDriveRcloneCompleted(u64, OneDriveEditorRequest, Result<OneDriveIdentity, String>),
    StartOneDriveMirrorSetup,
    StartOneDriveMirrorManualSetup,
    OneDriveMirrorSetupCompleted(Result<String, String>),
    OpenOneDriveMirrorAuthUrl,
    OneDriveAuthUrlOpened(ConnectionId, Result<String, String>),
    SubmitOneDriveMirrorAuthResponse,
    TestDraft,
    DraftTested(Connection, Result<String, String>),
    SaveDraft,
    SaveDraftValidated(Connection, Result<String, String>),
    DraftSaved(String),
    ConfirmImport(usize),
    RemoveConnection(ConnectionId),
    RemoveCompleted(String),
    Refresh,
    NoticeTick(Instant),
    PendingCountTick,
    PendingCountPolled(u64, Connection, PendingSource, Result<u64, String>),
    VpnStatusChecked(BTreeMap<VpnProfileId, bool>),
    NetworkStatusChecked(bool),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = AppLaunchMode;
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config = Config::load_runtime().config;
        let preload_draft = config.document.preload.into();
        let main_window_id = core.main_window_id();
        let standalone = flags != AppLaunchMode::Applet;
        let window_mode = match flags {
            AppLaunchMode::Applet | AppLaunchMode::AddConnection => WindowMode::AddConnection,
            AppLaunchMode::ModifyConnection(id) => WindowMode::ModifyConnection(id),
            AppLaunchMode::ImportLegacy => WindowMode::ImportLegacy,
            AppLaunchMode::GeneralSettings => WindowMode::GeneralSettings,
            AppLaunchMode::ReorderConnections => WindowMode::ReorderConnections,
        };

        let mut app = Self {
            core,
            standalone,
            popup: None,
            window_mode,
            draft: ConnectionDraft::default(),
            config,
            import_previews: Vec::new(),
            rclone_remotes: Vec::new(),
            pending_remove: None,
            pending_repair: None,
            pending_shared_remote_ack: None,
            pending_rclone_remote_remove: None,
            removing_connection: None,
            removing_rclone_remote: None,
            sftp_setup_pending: false,
            sharepoint_setup_pending: false,
            onedrive_editor_pending: None,
            onedrive_editor_generation: 0,
            sftp_settings_dirty: false,
            sftp_settings_revision: 0,
            sftp_update_ack: None,
            sftp_retry_secret_auth: None,
            onedrive_auth_open_command: String::new(),
            onedrive_auth_response_url: String::new(),
            validated_draft: None,
            last_notice: None,
            last_notice_at: None,
            popup_opened_at: None,
            vpn_ready: BTreeMap::new(),
            network_ready: None,
            sleep_status: None,
            sleep_needs_attention: false,
            sleep_notice: None,
            sleep_settings_pending: false,
            preload_notice: None,
            preload_settings_pending: false,
            runtime_owner: false,
            runtime_available: false,
            runtime_pending: false,
            runtime_poll_pending: false,
            runtime_poll_obsolete: false,
            runtime_error: None,
            runtime_operation_revision: 0,
            runtime_operation_completions: Vec::new(),
            runtime_routed_operations: BTreeMap::new(),
            runtime_operation_wait: None,
            rclone_refresh_starting: BTreeSet::new(),
            rclone_refresh_jobs: BTreeMap::new(),
            preload_draft,
            preload_input_dirty: false,
            directory_preload_jobs: BTreeMap::new(),
            unmount_pending: BTreeSet::new(),
            online_repair_required: BTreeSet::new(),
            repair_operations_pending: BTreeSet::new(),
            pending_counts: BTreeMap::new(),
            pending_poll_inflight: BTreeSet::new(),
            pending_poll_generation: 0,
            mirror_operations_pending: BTreeSet::new(),
            mirror_operation_controls: BTreeMap::new(),
            mount_entries: Vec::new(),
            mount_inspection_pending: false,
            mount_inspection_obsolete: false,
            mount_inspection_error: None,
            rclone_inspection_generation: 0,
            prefill_generation: 0,
            rclone_inspection_cancellation: CancellationToken::new(),
            prefill_cancellation: CancellationToken::new(),
            unit_observation: UnitObservation::default(),
            unit_observation_connections: Vec::new(),
            unit_observed_at: None,
            unit_observation_pending: false,
            unit_observation_invalidated: false,
            unit_observation_revision: 0,
            unit_observation_cancellation: CancellationToken::new(),
            unit_operations_waiting: BTreeMap::new(),
        };
        let inspection = match flags {
            AppLaunchMode::ModifyConnection(id) => app.load_draft(id),
            AppLaunchMode::ImportLegacy => app.scan_imports(),
            AppLaunchMode::Applet
            | AppLaunchMode::AddConnection
            | AppLaunchMode::GeneralSettings
            | AppLaunchMode::ReorderConnections => Task::none(),
        };
        let window_title = window_title(window_mode);
        let title = if standalone {
            app.set_header_title(window_title.clone());
            if app.last_notice.is_none() {
                app.last_notice = Some(window_mode_notice(window_mode));
            }
            app.set_window_title(window_title, main_window_id.unwrap_or(Id::RESERVED))
        } else if let Some(id) = main_window_id {
            app.set_window_title(fl!("app-title"), id)
        } else {
            Task::none()
        };

        let initial = if window_mode == WindowMode::GeneralSettings {
            app.runtime_poll_pending = true;
            runtime_request_task(RuntimeCommand::Status)
        } else {
            Task::none()
        };
        let mounts = app.refresh_mount_inspection();
        let units = app.refresh_unit_observation(false);
        (
            app,
            Task::batch([title, initial, inspection, mounts, units]),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        if self.standalone {
            return None;
        }
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.standalone {
            match self.window_mode {
                WindowMode::GeneralSettings => return self.view_general_settings(),
                WindowMode::ReorderConnections => return self.view_reorder_connections(),
                _ => return self.view_settings_window(),
            }
        }
        self.core
            .applet
            .icon_button("folder-remote-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        self.view_popup()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = Vec::new();
        if self.standalone || self.popup.is_some() {
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(5))
                    .map(|_| Message::MountInspectionTick),
            );
        }
        if !self.standalone {
            subscriptions.push(runtime_ipc::runtime_subscription().map(Message::RuntimeEvent));
        } else {
            if self.window_mode == WindowMode::GeneralSettings {
                subscriptions.push(runtime_ipc::settings_subscription().map(Message::RuntimeEvent));
            }
            if self.window_mode == WindowMode::GeneralSettings
                || self.runtime_operation_wait.is_some()
            {
                subscriptions.push(
                    cosmic::iced::time::every(Duration::from_secs(2)).map(|_| Message::RuntimePoll),
                );
            }
        }
        if !self.standalone && self.popup.is_some() {
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(5))
                    .map(|_| Message::UnitObservationTick),
            );
            subscriptions
                .push(cosmic::iced::time::every(Duration::from_secs(1)).map(Message::NoticeTick));
        }
        if !self.standalone && self.popup.is_some() {
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(5))
                    .map(|_| Message::PendingCountTick),
            );
        }
        if !self.standalone && self.runtime_owner && self.config.document.unmount_before_sleep {
            subscriptions
                .push(cosmic_ext_applet_mounter::sleep::subscription().map(Message::SleepEvent));
        }
        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        let force_unit_refresh = !matches!(&message, Message::TogglePopup);
        let refresh_mounts = matches!(
            &message,
            Message::TogglePopup
                | Message::ManagedOnlineOperationCompleted(..)
                | Message::OnlineRepairCompleted(..)
                | Message::OfflineRepairCompleted(..)
                | Message::MirrorOperationCompleted(..)
                | Message::RuntimeRefreshCompleted(..)
        );
        let task = self.update_message(message);

        if refresh_mounts {
            self.mount_inspection_obsolete |= self.mount_inspection_pending;
            let units = if !force_unit_refresh && self.popup.is_none() {
                Task::none()
            } else {
                self.refresh_unit_observation(force_unit_refresh)
            };
            Task::batch([task, self.refresh_mount_inspection(), units])
        } else {
            task
        }
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        if self.standalone {
            None
        } else {
            Some(cosmic::applet::style())
        }
    }
}

impl AppModel {
    fn update_message(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        let mut inspection_tasks = Vec::new();
        match message {
            Message::UnitObservationTick => return self.refresh_unit_observation(false),
            Message::UnitObservationCompleted(revision, connections, observation) => {
                if revision != self.unit_observation_revision {
                    return Task::none();
                }
                self.unit_observation_pending = false;
                if connections != self.config.document.connections {
                    return self.refresh_unit_observation(true);
                }
                self.unit_observation.services.extend(observation.services);
                self.unit_observation.timers.extend(observation.timers);
                self.unit_observation.coordinator_states = observation.coordinator_states;
                if let Some(warnings) = observation.warnings {
                    self.unit_observation.warnings = Some(warnings);
                }
                self.unit_observation.failed = observation.failed;
                let ids = connections
                    .iter()
                    .map(|connection| connection.id)
                    .collect::<BTreeSet<_>>();
                self.unit_observation
                    .services
                    .retain(|id, _| ids.contains(id));
                self.unit_observation
                    .timers
                    .retain(|id, _| ids.contains(id));
                self.unit_observation_connections = connections;
                self.unit_observed_at = Some(Instant::now());
                self.unit_observation_invalidated = false;
                let waiting = std::mem::take(&mut self.unit_operations_waiting);
                for (id, (connection, operation)) in waiting {
                    if self.unit_status_fresh(id)
                        && self
                            .config
                            .document
                            .connections
                            .iter()
                            .any(|saved| saved == &connection)
                    {
                        inspection_tasks.push(self.record_operation_request(id, operation));
                    } else {
                        self.last_notice = Some(fl!("unit-status-refresh-unavailable"));
                        self.last_notice_at = Some(Instant::now());
                        self.complete_routed_operation(id, operation);
                    }
                }
            }
            Message::MountInspectionTick => return self.refresh_mount_inspection(),
            Message::MountInspectionCompleted(result) => {
                self.mount_inspection_pending = false;
                if std::mem::take(&mut self.mount_inspection_obsolete) {
                    return self.refresh_mount_inspection();
                }
                match result {
                    Ok(entries) => {
                        self.mount_entries = entries;
                        self.mount_inspection_error = None;
                    }
                    Err(error) => {
                        if self.mount_inspection_error.as_ref() != Some(&error) {
                            self.last_notice = Some(fl!(
                                "ui-app-read-host-mount-table-error",
                                error = error.as_str()
                            ));
                            self.last_notice_at = Some(Instant::now());
                        }
                        self.mount_inspection_error = Some(error);
                    }
                }
                // Failed inspection is not evidence that a mount disappeared.
            }
            Message::RcloneInspectionCompleted(generation, show_notice, result) => {
                if generation == self.rclone_inspection_generation {
                    self.finish_rclone_inspection(show_notice, result);
                }
            }
            Message::RemotePrefillCompleted(generation, context, result) => {
                if generation == self.prefill_generation && context == self.prefill_context() {
                    self.finish_remote_prefill(result);
                }
            }
            Message::ImportInspectionCompleted(result) => self.finish_import_scan(result),
            Message::OpenGeneralSettings => {
                self.launch_settings_process(AppLaunchMode::GeneralSettings);
            }
            Message::OpenReorderConnections => {
                self.launch_settings_process(AppLaunchMode::ReorderConnections);
            }
            Message::RuntimeEvent(runtime_ipc::Event::Ready) => {
                self.runtime_owner = !self.standalone;
                self.runtime_error = None;
                if self.runtime_owner {
                    let connections = self.config.document.connections.clone();
                    return Task::batch([
                        self.start_refresh_for_active_google_mounts(),
                        self.start_preload_for_active_directory_mounts(),
                        Task::perform(
                            async move { refresh_online_mount_guards(connections).await },
                            |failures| {
                                cosmic::Action::App(Message::OnlineGuardsRefreshed(failures))
                            },
                        ),
                    ]);
                }
            }
            Message::OnlineGuardsRefreshed(failures) => {
                if !failures.is_empty() {
                    self.last_notice = Some(fl!(
                        "notice-guard-refresh-failed",
                        errors = failures.join("; ")
                    ));
                }
            }
            Message::RuntimeEvent(runtime_ipc::Event::ClientReady) => {
                self.runtime_owner = false;
                self.runtime_error = None;
            }
            Message::RuntimeEvent(runtime_ipc::Event::Activate) => {
                if let Some(id) = self.core.main_window_id() {
                    return cosmic::iced::window::gain_focus(id);
                }
            }
            Message::RuntimeEvent(runtime_ipc::Event::DuplicateWindow) => {
                return cosmic::iced::exit();
            }
            Message::RuntimeEvent(runtime_ipc::Event::Error(error)) => {
                if self.standalone {
                    self.last_notice = Some(error.clone());
                }
                self.runtime_error = Some(error);
            }
            Message::RuntimeEvent(runtime_ipc::Event::Request(command, reply)) => {
                if self.standalone {
                    reply.complete(Err(crate::fl!(
                        "ui-app-this-window-is-not-the-runtime-owner"
                    )));
                } else if let RuntimeCommand::RunOperation(connection_id, operation) = command {
                    if self.runtime_routed_operations.contains_key(&connection_id) {
                        reply.complete(Err(crate::fl!(
                            "ui-app-another-routed-operation-is-already-pending-for"
                        )));
                        return Task::none();
                    }
                    match load_runtime_config() {
                        Ok(config) => self.config = config,
                        Err(error) => {
                            reply.complete(Err(error));
                            return Task::none();
                        }
                    }
                    self.runtime_routed_operations
                        .insert(connection_id, operation);
                    reply.complete(Ok(self.runtime_status()));
                    return self.record_operation_request(connection_id, operation);
                } else if matches!(
                    command,
                    RuntimeCommand::Refresh | RuntimeCommand::ConfigurationChanged
                ) {
                    match load_runtime_config() {
                        Ok(config) => {
                            self.config = config;
                            self.pending_counts.clear();
                            self.pending_poll_generation =
                                self.pending_poll_generation.wrapping_add(1);
                        }
                        Err(error) => {
                            reply.complete(Err(error));
                            return Task::none();
                        }
                    }
                    let config = self.config.document.clone();
                    return Task::perform(
                        async move { current_vpn_ready_states(config).await },
                        move |ready| {
                            cosmic::Action::App(Message::RuntimeRefreshCompleted(
                                reply.clone(),
                                ready,
                            ))
                        },
                    );
                } else {
                    let result = self.apply_runtime_command(command);
                    reply.complete(result);
                }
            }
            Message::RuntimeRefreshCompleted(reply, ready) => {
                self.vpn_ready = ready;
                reply.complete(Ok(self.runtime_status()));
            }
            Message::RuntimePoll => {
                if !self.runtime_pending && !self.runtime_poll_pending {
                    self.runtime_poll_pending = true;
                    return Task::batch([
                        runtime_request_task(RuntimeCommand::Status),
                        Task::perform(current_network_ready(), |ready| {
                            cosmic::Action::App(Message::NetworkStatusChecked(ready))
                        }),
                    ]);
                }
            }
            Message::RuntimeResult(command, result) => {
                if command == RuntimeCommand::Status {
                    self.runtime_poll_pending = false;
                    // A user action may have completed since this poll started.
                    // Discard the entire stale reply, including availability/errors.
                    if std::mem::take(&mut self.runtime_poll_obsolete) {
                        return Task::none();
                    }
                } else {
                    self.runtime_pending = false;
                    self.sleep_settings_pending = false;
                    self.preload_settings_pending = false;
                }
                match result {
                    Ok(status) => {
                        if command == RuntimeCommand::Status {
                            self.reconcile_runtime_operation_completion(&status);
                        }
                        self.runtime_available = true;
                        self.runtime_error = None;
                        // Ignore a status poll that started before a setting write.
                        if !self.runtime_pending {
                            self.config.document.unmount_before_sleep = status.unmount_before_sleep;
                            self.config.document.restore_after_wake = status.restore_after_wake;
                            self.config.document.preload = status.preload;
                            if !self.preload_input_dirty {
                                self.preload_draft = status.preload.into();
                            }
                            self.sleep_status = status.sleep_status;
                            self.sleep_needs_attention = status.sleep_needs_attention;
                        }
                        match command {
                            RuntimeCommand::SetUnmount(_) | RuntimeCommand::SetRestore(_) => {
                                self.sleep_notice = Some(fl!("notice-setting-saved"));
                            }
                            RuntimeCommand::SetPreload(value) => {
                                self.preload_input_dirty = false;
                                self.preload_draft = value.into();
                                self.preload_notice = Some(fl!("notice-preload-saved"));
                            }
                            RuntimeCommand::Refresh => {
                                self.last_notice = Some(fl!("notice-refreshed"));
                            }
                            RuntimeCommand::RunOperation(connection_id, operation) => {
                                if let Some(wait) = self.runtime_operation_wait.as_mut()
                                    && wait.connection_id == connection_id
                                    && wait.operation == operation
                                {
                                    wait.after_revision = Some(status.operation_revision);
                                }
                                let name = self
                                    .config
                                    .document
                                    .connections
                                    .iter()
                                    .find(|connection| connection.id == connection_id)
                                    .map(|connection| connection.name.clone())
                                    .unwrap_or_else(|| fl!("access-offline"));
                                self.last_notice = Some(fl!(
                                    "notice-operation-requested",
                                    operation = operation_label(operation),
                                    name = name
                                ));
                            }
                            _ => {}
                        }
                    }
                    Err(error) => {
                        self.runtime_available = false;
                        match command {
                            RuntimeCommand::SetUnmount(_) | RuntimeCommand::SetRestore(_) => {
                                self.sleep_notice = Some(error);
                            }
                            RuntimeCommand::SetPreload(_) => {
                                self.preload_notice = Some(error);
                            }
                            RuntimeCommand::Status => self.runtime_error = Some(error),
                            RuntimeCommand::RunOperation(connection_id, operation) => {
                                if self.runtime_operation_wait.is_some_and(|wait| {
                                    wait.connection_id == connection_id
                                        && wait.operation == operation
                                }) {
                                    self.runtime_operation_wait = None;
                                }
                                self.last_notice = Some(error);
                            }
                            _ => self.last_notice = Some(error),
                        }
                    }
                }
            }
            Message::EditorNotified(result) => {
                if let Err(error) = result {
                    let notice = self.last_notice.get_or_insert_with(String::new);
                    let _ = write!(
                        notice,
                        " Applet refresh unavailable: {error}. Reopen the popup to reload."
                    );
                }
            }
            Message::UnmountBeforeSleep(value) => {
                self.runtime_poll_obsolete = self.runtime_poll_pending;
                self.runtime_pending = true;
                self.sleep_settings_pending = true;
                self.sleep_notice = None;
                return runtime_request_task(RuntimeCommand::SetUnmount(value));
            }
            Message::RestoreAfterWake(value) => {
                self.runtime_poll_obsolete = self.runtime_poll_pending;
                self.runtime_pending = true;
                self.sleep_settings_pending = true;
                self.sleep_notice = None;
                return runtime_request_task(RuntimeCommand::SetRestore(value));
            }
            Message::PreloadEnabled(provider, value) => {
                self.preload_draft.policy_mut(provider).enabled = value;
                self.preload_input_dirty = true;
                self.preload_notice = None;
            }
            Message::PreloadSecondsInput(provider, value) => {
                if value.len() <= 3 && value.chars().all(|character| character.is_ascii_digit()) {
                    self.preload_draft.policy_mut(provider).maximum_seconds = value;
                    self.preload_input_dirty = true;
                    self.preload_notice = None;
                }
            }
            Message::PreloadDepthInput(provider, value) => {
                if value.len() <= 2 && value.chars().all(|character| character.is_ascii_digit()) {
                    self.preload_draft.policy_mut(provider).maximum_depth = Some(value);
                    self.preload_input_dirty = true;
                    self.preload_notice = None;
                }
            }
            Message::SavePreloadSettings => {
                let value = match self.preload_draft.validated() {
                    Ok(value) => value,
                    Err(error) => {
                        self.preload_notice = Some(error);
                        return Task::none();
                    }
                };
                self.runtime_poll_obsolete = self.runtime_poll_pending;
                self.runtime_pending = true;
                self.preload_settings_pending = true;
                self.preload_notice = None;
                return runtime_request_task(RuntimeCommand::SetPreload(value));
            }
            Message::SleepEvent(cosmic_ext_applet_mounter::sleep::Event::Status {
                text,
                needs_attention,
            }) => {
                self.sleep_status = Some(text);
                self.sleep_needs_attention = needs_attention;
                self.pending_counts.clear();
                self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
            }
            Message::SleepEvent(cosmic_ext_applet_mounter::sleep::Event::Wake(ids)) => {
                self.config = Config::load_runtime().config;
                self.pending_counts.clear();
                self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                if self.config.document.unmount_before_sleep
                    && self.config.document.restore_after_wake
                {
                    let connections = self
                        .config
                        .document
                        .connections
                        .iter()
                        .filter(|connection| {
                            ids.contains(&connection.id)
                                && connection.enabled
                                && matches!(connection.mode, ConnectionMode::OnlineMount(_))
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    return Task::batch(connections.into_iter().map(|connection| {
                        Task::perform(
                            async move {
                                let result = restore_online_after_wake_result(&connection).await;
                                (connection, result)
                            },
                            |(connection, result)| {
                                cosmic::Action::App(Message::ManagedOnlineOperationCompleted(
                                    connection,
                                    Operation::Mount,
                                    result,
                                ))
                            },
                        )
                    }));
                }
            }
            Message::TogglePopup => {
                return if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    self.config = Config::load_runtime().config;
                    self.pending_counts.clear();
                    self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                    self.pending_poll_inflight.clear();
                    self.pending_remove = None;
                    self.pending_repair = None;
                    self.pending_shared_remote_ack = None;
                    self.pending_rclone_remote_remove = None;
                    self.vpn_ready = BTreeMap::new();
                    self.popup_opened_at = Some(Instant::now());
                    let id = Id::unique();
                    self.popup = Some(id);
                    let mut settings = self.core.applet.get_popup_settings(
                        self.core
                            .main_window_id()
                            .expect("applet must have a main window"),
                        id,
                        None,
                        None,
                        None,
                    );
                    settings.positioner.size_limits = Limits::NONE
                        .min_width(640.0)
                        .max_width(920.0)
                        .min_height(180.0)
                        .max_height(560.0);
                    let config = self.config.document.clone();
                    Task::batch([
                        get_popup(settings),
                        self.set_window_title(fl!("app-title"), id),
                        Task::perform(
                            async move { current_vpn_ready_states(config).await },
                            |ready| cosmic::Action::App(Message::VpnStatusChecked(ready)),
                        ),
                        Task::perform(current_network_ready(), |ready| {
                            cosmic::Action::App(Message::NetworkStatusChecked(ready))
                        }),
                        Task::perform(async {}, |_| cosmic::Action::App(Message::PendingCountTick)),
                    ])
                };
            }
            Message::OpenAddConnection => {
                self.pending_remove = None;
                self.pending_repair = None;
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.removing_connection = None;
                self.removing_rclone_remote = None;
                self.launch_settings_process(AppLaunchMode::AddConnection);
            }
            Message::OpenModifyConnection(connection_id) => {
                self.pending_remove = None;
                self.pending_repair = None;
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.removing_connection = None;
                self.removing_rclone_remote = None;
                self.launch_settings_process(AppLaunchMode::ModifyConnection(connection_id));
            }
            Message::ReorderConnection(connection_id, new_position) => {
                return self.reorder_connection(connection_id, new_position);
            }
            Message::PopupClosed(id) if self.popup == Some(id) => {
                self.popup = None;
                self.popup_opened_at = None;
            }
            Message::PopupClosed(_) => {}
            Message::OperationRequested(connection_id, operation) => {
                self.pending_counts.remove(&connection_id);
                self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                if !self.runtime_owner {
                    self.runtime_poll_obsolete = self.runtime_poll_pending;
                    self.runtime_pending = true;
                    self.runtime_operation_wait = Some(RuntimeOperationWait {
                        connection_id,
                        operation,
                        after_revision: None,
                    });
                    return runtime_request_task(RuntimeCommand::RunOperation(
                        connection_id,
                        operation,
                    ));
                }
                return self.record_operation_request(connection_id, operation);
            }
            Message::OnlineRepairCompleted(id, notice) => {
                self.repair_operations_pending.remove(&id);
                self.online_repair_required.remove(&id);
                self.config = Config::load_runtime().config;
                self.pending_repair = None;
                self.last_notice = Some(notice);
                self.last_notice_at = Some(Instant::now());
                self.complete_routed_operation(id, Operation::Repair);
            }
            Message::OfflineRepairCompleted(id, notice) => {
                self.repair_operations_pending.remove(&id);
                self.config = Config::load_runtime().config;
                self.pending_repair = None;
                self.last_notice = Some(notice);
                self.last_notice_at = Some(Instant::now());
                self.complete_routed_operation(id, Operation::Repair);
            }
            Message::MirrorOperationCompleted(id, notice) => {
                self.mirror_operations_pending.remove(&id);
                self.mirror_operation_controls.remove(&id);
                self.pending_counts.remove(&id);
                self.config = Config::load_runtime().config;
                self.last_notice = Some(notice);
                self.last_notice_at = Some(Instant::now());
                if let Some(operation) = self.runtime_routed_operations.get(&id).copied() {
                    self.complete_routed_operation(id, operation);
                }
                return Task::perform(async {}, |_| cosmic::Action::App(Message::PendingCountTick));
            }
            Message::ManagedOnlineOperationCompleted(connection, operation, result) => {
                if operation == Operation::Unmount {
                    self.unmount_pending.remove(&connection.id);
                }
                update_online_repair_requirement(
                    &mut self.online_repair_required,
                    connection.id,
                    operation,
                    &result,
                );
                self.config = Config::load_runtime().config;
                self.pending_repair = None;
                let label = operation_label(operation);
                match result {
                    Ok(()) => {
                        self.last_notice = Some(fl!(
                            "notice-operation-completed",
                            operation = label,
                            name = connection.name.as_str()
                        ));
                        self.last_notice_at = Some(Instant::now());
                        self.complete_routed_operation(connection.id, operation);
                        if operation == Operation::Mount
                            && connection.provider == Provider::GoogleDrive
                            && self.config.document.preload.google_drive.enabled
                        {
                            let plan = match rclone_mount_plan(
                                &connection,
                                &default_runtime_root(),
                                &default_cache_root(),
                            ) {
                                Ok(plan) => plan,
                                Err(error) => {
                                    self.last_notice = Some(fl!(
                                        "notice-refresh-plan-failed",
                                        name = connection.name.as_str(),
                                        error = error.to_string()
                                    ));
                                    return Task::none();
                                }
                            };
                            let name = connection.name.clone();
                            let connection_id = connection.id;
                            let timeout = Duration::from_secs(
                                self.config.document.preload.google_drive.maximum_seconds,
                            );
                            self.rclone_refresh_starting.insert(connection_id);
                            self.last_notice =
                                Some(fl!("notice-refresh-starting", name = name.as_str()));
                            return Task::perform(
                                async move {
                                    rclone_refresh::start(
                                        connection.id,
                                        plan.rc_socket,
                                        connection.local_path,
                                        timeout,
                                    )
                                    .await
                                },
                                move |result| {
                                    cosmic::Action::App(Message::RcloneRefreshStarted(
                                        name.clone(),
                                        connection_id,
                                        result,
                                    ))
                                },
                            );
                        }
                        let preload_policy = self.config.document.preload_policy_for(&connection);
                        if operation == Operation::Mount
                            && matches!(
                                connection.provider,
                                Provider::OneDrive
                                    | Provider::Teams
                                    | Provider::Box
                                    | Provider::Smb
                                    | Provider::Sftp
                            )
                            && matches!(connection.mode, ConnectionMode::OnlineMount(_))
                            && preload_policy.enabled
                        {
                            let name = connection.name.clone();
                            let timeout = Duration::from_secs(preload_policy.maximum_seconds);
                            let maximum_depth = preload_policy.maximum_depth;
                            let wait_for_root = directory_preload_waits_for_root(&connection);
                            self.last_notice =
                                Some(fl!("notice-preload-starting", name = name.as_str()));
                            let result = directory_preload::start_for_connection(
                                &connection,
                                timeout,
                                maximum_depth,
                                wait_for_root,
                            );
                            return self.register_directory_preload(name, result);
                        }
                        if operation == Operation::Unmount {
                            self.rclone_refresh_starting.remove(&connection.id);
                            self.rclone_refresh_jobs.remove(&connection.id);
                            self.directory_preload_jobs.remove(&connection.id);
                        }
                    }
                    Err(error) => {
                        self.last_notice = Some(fl!(
                            "notice-operation-failed",
                            operation = label,
                            name = connection.name.as_str(),
                            error = error
                        ));
                        self.last_notice_at = Some(Instant::now());
                        self.complete_routed_operation(connection.id, operation);
                    }
                }
            }
            Message::RcloneRefreshStarted(name, connection_id, result) => {
                self.rclone_refresh_starting.remove(&connection_id);
                if self.unmount_pending.contains(&connection_id) {
                    if result.is_ok() {
                        return Task::perform(
                            async move {
                                let _ = rclone_refresh::cancel(connection_id).await;
                            },
                            |_| cosmic::Action::App(Message::RuntimePoll),
                        );
                    }
                    return Task::none();
                }
                match result {
                    Ok(job) => {
                        self.rclone_refresh_jobs
                            .insert(job.connection_id, job.job_id);
                        self.last_notice = Some(fl!(
                            "notice-refresh-running",
                            name = name.as_str(),
                            job = job.job_id
                        ));
                        self.last_notice_at = Some(Instant::now());
                        let connection_id = job.connection_id;
                        let job_id = job.job_id;
                        return Task::perform(
                            async move { rclone_refresh::monitor(job).await },
                            move |result| {
                                cosmic::Action::App(Message::RcloneRefreshCompleted(
                                    name.clone(),
                                    connection_id,
                                    job_id,
                                    result,
                                ))
                            },
                        );
                    }
                    Err(error) if error.contains("canceled") => {}
                    Err(error) => {
                        self.last_notice = Some(fl!(
                            "notice-refresh-start-failed",
                            name = name.as_str(),
                            error = error
                        ));
                        self.last_notice_at = Some(Instant::now());
                    }
                }
            }
            Message::RcloneRefreshCompleted(name, connection_id, job_id, result) => {
                if self.rclone_refresh_jobs.get(&connection_id) != Some(&job_id) {
                    return Task::none();
                }
                self.rclone_refresh_jobs.remove(&connection_id);
                if self.unmount_pending.contains(&connection_id) {
                    return Task::none();
                }
                match result {
                    Ok(outcome) => {
                        if let Some(notice) = outcome.notice(&name) {
                            self.last_notice = Some(notice);
                            self.last_notice_at = Some(Instant::now());
                        }
                    }
                    Err(error) => {
                        self.last_notice = Some(fl!(
                            "notice-refresh-failed",
                            name = name.as_str(),
                            error = error
                        ));
                        self.last_notice_at = Some(Instant::now());
                    }
                }
            }
            Message::DirectoryPreloadCompleted(name, connection_id, generation, result) => {
                if self.directory_preload_jobs.get(&connection_id) != Some(&generation) {
                    return Task::none();
                }
                self.directory_preload_jobs.remove(&connection_id);
                if self.unmount_pending.contains(&connection_id) {
                    return Task::none();
                }
                match result {
                    Ok(outcome) => {
                        if let Some(notice) = outcome.notice(&name) {
                            self.last_notice = Some(notice);
                            self.last_notice_at = Some(Instant::now());
                        }
                    }
                    Err(error) => {
                        self.last_notice = Some(fl!(
                            "notice-preload-failed",
                            name = name.as_str(),
                            error = error
                        ));
                        self.last_notice_at = Some(Instant::now());
                    }
                }
            }
            Message::DraftProvider(provider) => {
                if matches!(self.window_mode, WindowMode::ModifyConnection(_)) {
                    self.last_notice = Some(fl!("notice-provider-locked"));
                    return Task::none();
                }
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.validated_draft = None;
                self.sftp_update_ack = None;
                self.draft.sftp.clear_secrets();
                self.reset_sftp_settings_tracking();
                self.draft.provider = provider;
                self.draft.onedrive_identity = None;
                self.draft.onedrive_expected_account.clear();
                self.draft.legacy_onedrive = false;
                self.draft.offline_engine = None;
                if provider == Provider::Teams {
                    if self.draft.id.is_none() {
                        self.draft.id = Some(ConnectionId::new());
                    }
                    self.draft.access_mode = AccessMode::OnlineMount;
                    self.draft.start_at_login = false;
                }
                self.draft.connection_preload_use_global = true;
                let policy = if provider == Provider::Sftp {
                    self.config.document.preload.sftp
                } else {
                    self.config.document.preload.smb
                };
                self.draft.connection_preload_enabled = policy.enabled;
                self.draft.connection_preload_seconds = policy.maximum_seconds.to_string();
                self.draft.connection_preload_depth = policy.maximum_depth.unwrap_or(2).to_string();
                if provider != Provider::Smb {
                    self.draft.smb_password.clear();
                }
                if provider != Provider::GoogleDrive {
                    self.draft.google_client_id.clear();
                    self.draft.google_client_secret.clear();
                }
                if provider == Provider::OneDrive {
                    let id = *self.draft.id.get_or_insert_with(ConnectionId::new);
                    self.draft.remote_reference = default_onedrive_account_label(id);
                }
            }
            Message::DraftAccessMode(mode) => {
                if matches!(self.window_mode, WindowMode::ModifyConnection(_)) {
                    self.last_notice = Some(fl!("notice-access-mode-locked"));
                    return Task::none();
                }
                self.validated_draft = None;
                self.draft.access_mode = mode;
                self.draft.onedrive_identity = None;
            }
            Message::DraftSharePointLibraryUrl(value) => {
                self.validated_draft = None;
                self.draft.sharepoint_library_url = value;
                self.draft.teams_drive_id.clear();
            }
            Message::DraftName(value) => {
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.validated_draft = None;
                self.draft.name = value;
            }
            Message::DraftRemote(value) => {
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.validated_draft = None;
                self.draft.remote_reference = value;
                if draft_is_onedrive_rclone(&self.draft) {
                    self.draft.onedrive_identity = None;
                }
                if self.draft.provider == Provider::Teams {
                    self.draft.teams_drive_id.clear();
                }
                self.sftp_update_ack = None;
                if self.draft.provider == Provider::Sftp {
                    inspection_tasks.push(self.remote_prefill());
                }
            }
            Message::DraftSubpath(value) => {
                self.pending_shared_remote_ack = None;
                self.validated_draft = None;
                self.draft.remote_subpath = value;
                if draft_is_onedrive_rclone(&self.draft) {
                    self.draft.onedrive_identity = None;
                }
            }
            Message::DraftGoogleClientId(value) => {
                self.validated_draft = None;
                self.draft.google_client_id = value;
            }
            Message::DraftGoogleClientSecret(value) => {
                self.validated_draft = None;
                self.draft.google_client_secret = value;
            }
            Message::DraftSftp(field, value) => {
                self.validated_draft = None;
                self.sftp_update_ack = None;
                let before = self.draft.sftp.clone();
                self.draft.sftp.set(field, value);
                if self.draft.sftp != before {
                    self.mark_sftp_settings_changed();
                }
            }
            Message::DraftSftpAuth(auth) => {
                self.validated_draft = None;
                self.sftp_update_ack = None;
                if self.draft.sftp.auth != auth {
                    self.draft.sftp.clear_secrets();
                    self.draft.sftp.auth = auth;
                    self.sftp_retry_secret_auth = None;
                    self.mark_sftp_settings_changed();
                }
            }
            Message::ApplySftpRemote => return self.apply_sftp_remote(),
            Message::SftpRemoteApplied(name, revision, attempted_secret, result) => {
                self.finish_sftp_settings_update(&name, revision, result.is_ok());
                self.sftp_setup_pending = false;
                self.sftp_update_ack = None;
                self.validated_draft = None;
                inspection_tasks.push(self.detect_rclone_remotes(false));
                self.sftp_retry_secret_auth = if result.is_err()
                    && self.draft.provider == Provider::Sftp
                    && self.draft.remote_reference.trim() == name
                    && attempted_secret == Some(self.draft.sftp.auth)
                {
                    attempted_secret
                } else {
                    None
                };
                self.last_notice = Some(match result {
                    Ok(true) => fl!("sftp-updated", name = name.as_str()),
                    Ok(false) => fl!("sftp-created", name = name.as_str()),
                    Err(error) if attempted_secret.is_some() => fl!(
                        "sftp-apply-failed-secret",
                        name = name.as_str(),
                        error = error
                    ),
                    Err(error) => fl!("sftp-apply-failed", name = name.as_str(), error = error),
                });
            }
            Message::DraftSmbHost(value) => {
                self.validated_draft = None;
                self.draft.smb_host = value;
            }
            Message::DraftSmbUser(value) => {
                self.validated_draft = None;
                self.draft.smb_user = value;
            }
            Message::DraftSmbDomain(value) => {
                self.validated_draft = None;
                self.draft.smb_domain = value;
            }
            Message::DraftSmbPassword(value) => {
                self.validated_draft = None;
                self.draft.smb_password = value;
            }
            Message::DraftLocalPath(value) => {
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.validated_draft = None;
                self.draft.local_path = value;
            }
            Message::OpenLocalFolderPicker => {
                return self.open_local_folder_picker();
            }
            Message::OpenRecoveryFolderPicker => {
                self.last_notice = Some(fl!(
                    "notice-picker-opening",
                    target = fl!("ui-app-recovery-directory")
                ));
                return Task::perform(
                    async { pick_folder(fl!("ui-app-recovery-directory")).await },
                    |result| cosmic::Action::App(Message::RecoveryFolderPicked(result)),
                );
            }
            Message::RecoveryFolderPicked(result) => match result {
                Ok(Some(path)) => {
                    self.validated_draft = None;
                    self.draft.recovery_directory = path.clone();
                    self.last_notice = Some(fl!(
                        "notice-folder-selected",
                        target = fl!("ui-app-recovery-directory"),
                        path = path
                    ));
                }
                Ok(None) => self.last_notice = Some(fl!("notice-folder-cancelled")),
                Err(error) => self.last_notice = Some(fl!("notice-folder-failed", error = error)),
            },
            Message::LocalFolderPicked(result) => match result {
                Ok(Some(path)) => {
                    self.pending_shared_remote_ack = None;
                    self.pending_rclone_remote_remove = None;
                    self.validated_draft = None;
                    self.draft.local_path = path.clone();
                    self.last_notice = Some(fl!(
                        "notice-folder-selected",
                        target = local_target_label(self.draft.access_mode),
                        path = path
                    ));
                }
                Ok(None) => {
                    self.last_notice = Some(fl!("notice-folder-cancelled"));
                }
                Err(error) => {
                    self.last_notice = Some(fl!("notice-folder-failed", error = error));
                }
            },
            Message::DraftRecoveryDirectory(value) => {
                self.validated_draft = None;
                self.draft.recovery_directory = value;
            }
            Message::DraftCacheLimit(value) => {
                self.validated_draft = None;
                self.draft.cache_limit_gib = value;
            }
            Message::DraftSyncInterval(value) => {
                self.validated_draft = None;
                self.draft.sync_interval_minutes = value;
            }
            Message::DraftEnabled(value) => {
                self.validated_draft = None;
                self.draft.enabled = value;
            }
            Message::DraftSyncOnMetered(value) => {
                self.validated_draft = None;
                self.draft.sync_on_metered = value;
            }
            Message::DraftConnectionPreloadUseGlobal(value) => {
                self.validated_draft = None;
                self.draft.connection_preload_use_global = value;
            }
            Message::DraftConnectionPreloadEnabled(value) => {
                self.validated_draft = None;
                self.draft.connection_preload_enabled = value;
            }
            Message::DraftConnectionPreloadSeconds(value) => {
                if value.len() <= 3 && value.chars().all(|c| c.is_ascii_digit()) {
                    self.validated_draft = None;
                    self.draft.connection_preload_seconds = value;
                }
            }
            Message::DraftConnectionPreloadDepth(value) => {
                if value.len() <= 2 && value.chars().all(|c| c.is_ascii_digit()) {
                    self.validated_draft = None;
                    self.draft.connection_preload_depth = value;
                }
            }
            Message::DraftVpn(id) => {
                self.validated_draft = None;
                self.draft.vpn_profile_id = id;
            }
            Message::DraftDisconnectVpn(value) => {
                self.validated_draft = None;
                self.draft.disconnect_vpn_when_unused = value;
            }
            Message::DraftOneDriveAuthResponse(value) => {
                self.onedrive_auth_response_url = value;
            }
            Message::DetectVpns => {
                self.detect_and_import_vpns();
            }
            Message::DetectRcloneRemotes => {
                inspection_tasks.push(self.detect_rclone_remotes(true));
            }
            Message::ApplyGoogleDriveRcloneRemote => {
                return self.apply_google_drive_rclone_remote();
            }
            Message::GoogleDriveRcloneRemoteApplied(result) => {
                self.draft.google_client_id.clear();
                self.draft.google_client_secret.clear();
                match result {
                    Ok(result) => {
                        self.validated_draft = None;
                        self.draft.remote_reference = result.remote_name.clone();
                        inspection_tasks.push(self.detect_rclone_remotes(false));
                        self.last_notice = Some(if result.updated {
                            fl!("notice-google-updated", name = result.remote_name)
                        } else {
                            fl!("notice-google-created", name = result.remote_name)
                        });
                    }
                    Err(error) => {
                        self.last_notice = Some(fl!("notice-google-failed", error = error));
                    }
                }
            }
            Message::CreateBoxRcloneRemote => {
                return self.create_box_rclone_remote();
            }
            Message::BoxRcloneRemoteCreated(result) => match result {
                Ok(remote_name) => {
                    self.validated_draft = None;
                    self.draft.remote_reference = remote_name.clone();
                    inspection_tasks.push(self.detect_rclone_remotes(false));
                    self.last_notice = Some(fl!("notice-box-created", name = remote_name));
                }
                Err(error) => {
                    self.last_notice = Some(fl!("notice-box-failed", error = error));
                }
            },
            Message::CreateSharePointRcloneRemote => {
                return self.create_sharepoint_rclone_remote();
            }
            Message::SharePointRcloneRemoteCreated(result) => {
                self.sharepoint_setup_pending = false;
                match result {
                    Ok(outcome) => {
                        inspection_tasks.push(self.detect_rclone_remotes(false));
                        if self.draft.provider == Provider::Teams
                            && self.draft.remote_reference.trim() == outcome.remote_name
                            && sharepoint::same_library_url(
                                &self.draft.sharepoint_library_url,
                                &outcome.library_url,
                            )
                        {
                            self.validated_draft = None;
                            self.draft.teams_drive_id = outcome.verified.identity.drive_id;
                            let account = outcome
                                .verified
                                .account
                                .unwrap_or_else(|| fl!("teams-oauth-account-unavailable"));
                            self.last_notice = Some(fl!(
                                "teams-oauth-created",
                                name = outcome.remote_name,
                                account = account
                            ));
                        } else {
                            self.last_notice = Some(fl!(
                                "teams-oauth-editor-changed",
                                name = outcome.remote_name
                            ));
                        }
                    }
                    Err(error) => {
                        self.last_notice = Some(fl!("teams-oauth-create-failed", error = error));
                    }
                }
            }
            Message::CreateSmbRcloneRemote => {
                return self.create_smb_rclone_remote();
            }
            Message::SmbRcloneRemoteCreated(result) => match result {
                Ok(result) => {
                    self.validated_draft = None;
                    self.draft.remote_reference = result.remote_name.clone();
                    self.draft.smb_password.clear();
                    inspection_tasks.push(self.detect_rclone_remotes(false));
                    let password_status = if result.password_updated {
                        fl!("notice-smb-password-updated")
                    } else {
                        fl!("notice-smb-password-unchanged")
                    };
                    self.last_notice = Some(fl!(
                        "notice-smb-saved",
                        name = result.remote_name,
                        password_status = password_status
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(fl!("notice-smb-failed", error = error));
                }
            },
            Message::RequestRemoveRcloneRemote(remote_name) => {
                return self.request_remove_rclone_remote(remote_name);
            }
            Message::RcloneRemoteRemoved(result) => match result {
                Ok(remote_name) => {
                    self.pending_rclone_remote_remove = None;
                    self.removing_rclone_remote = None;
                    self.validated_draft = None;
                    if self.draft.remote_reference == remote_name {
                        self.draft.remote_reference.clear();
                    }
                    inspection_tasks.push(self.detect_rclone_remotes(false));
                    self.last_notice = Some(fl!("notice-rclone-removed", name = remote_name));
                }
                Err(error) => {
                    self.pending_rclone_remote_remove = None;
                    self.removing_rclone_remote = None;
                    self.last_notice = Some(fl!("notice-rclone-remove-failed", error = error));
                }
            },
            Message::StartOnedriverSetup => {
                return self.start_onedriver_setup();
            }
            Message::OnedriverSetupCompleted(result) => match result {
                Ok(summary) => {
                    self.last_notice =
                        Some(fl!("notice-onedriver-setup-complete", summary = summary));
                }
                Err(error) => {
                    self.last_notice = Some(fl!("notice-onedriver-setup-failed", error = error));
                }
            },
            Message::DraftOneDriveType(kind) => {
                self.draft.onedrive_drive_type = kind;
                self.draft.onedrive_identity = None;
                self.validated_draft = None;
            }
            Message::DraftOneDriveAccount(account) => {
                self.draft.onedrive_expected_account = account;
                if draft_is_onedrive_rclone(&self.draft) {
                    self.draft.onedrive_identity = None;
                }
                self.validated_draft = None;
            }
            Message::SetupOneDriveRclone(create) => return self.setup_onedrive_rclone(create),
            Message::CancelOneDriveRclone => {
                if let Some(pending) = &self.onedrive_editor_pending {
                    pending.cancellation.cancel();
                    self.last_notice = Some(fl!("onedrive-editor-cancelling"));
                }
            }
            Message::OneDriveRcloneCompleted(generation, request, result) => {
                if generation != self.onedrive_editor_generation {
                    return Task::none();
                }
                let cancelled = self
                    .onedrive_editor_pending
                    .take()
                    .is_none_or(|pending| pending.cancellation.is_cancelled());
                let saved_binding_matches = OneDriveEditorRequest::from_draft(
                    &self.draft,
                    &self.config.document.connections,
                    request.create,
                )
                .is_ok_and(|current| {
                    current.setup.previous_identity == request.setup.previous_identity
                });
                if cancelled || !request.matches(&self.draft) || !saved_binding_matches {
                    self.last_notice = Some(match result {
                        Err(error) => fl!("onedrive-editor-result-discarded-error", error = error),
                        Ok(_) => fl!("onedrive-editor-result-discarded"),
                    });
                    return Task::none();
                }
                self.validated_draft = None;
                match accept_onedrive_editor_result(&mut self.draft, &request, result) {
                    Ok(()) => {
                        inspection_tasks.push(self.detect_rclone_remotes(false));
                        self.last_notice = Some(fl!("onedrive-editor-verified"));
                    }
                    Err(error) => self.last_notice = Some(error),
                }
            }
            Message::StartOneDriveMirrorSetup => {
                return self.start_onedrive_mirror_setup();
            }
            Message::StartOneDriveMirrorManualSetup => {
                return self.start_onedrive_mirror_manual_setup();
            }
            Message::OneDriveMirrorSetupCompleted(result) => match result {
                Ok(summary) => {
                    self.onedrive_auth_open_command.clear();
                    self.onedrive_auth_response_url.clear();
                    if let Ok(connection) = connection_from_draft(&self.draft) {
                        self.validated_draft = Some(ValidatedDraft {
                            connection,
                            summary: summary.clone(),
                        });
                    }
                    self.last_notice =
                        Some(fl!("notice-onedrive-setup-complete", summary = summary));
                }
                Err(error) => {
                    self.last_notice = Some(fl!("notice-onedrive-setup-failed", error = error));
                }
            },
            Message::OpenOneDriveMirrorAuthUrl => {
                return self.open_onedrive_mirror_auth_url();
            }
            Message::OneDriveAuthUrlOpened(id, result) => {
                if self.draft.id == Some(id) {
                    self.last_notice = Some(result.unwrap_or_else(|error| error));
                }
            }
            Message::SubmitOneDriveMirrorAuthResponse => {
                self.submit_onedrive_mirror_auth_response();
            }
            Message::TestDraft | Message::SaveDraft if self.onedrive_editor_pending.is_some() => {
                self.last_notice = Some(fl!("onedrive-editor-wait"));
            }
            Message::TestDraft if self.sftp_setup_pending => {
                self.last_notice = Some(fl!("sftp-wait-test"));
            }
            Message::TestDraft if self.sharepoint_setup_pending => {
                self.last_notice = Some(fl!("teams-oauth-wait-test"));
            }
            Message::SaveDraft if self.sftp_setup_pending => {
                self.last_notice = Some(fl!("sftp-wait-save"));
            }
            Message::SaveDraft if self.sharepoint_setup_pending => {
                self.last_notice = Some(fl!("teams-oauth-wait-save"));
            }
            Message::TestDraft => {
                return self.test_draft_plan();
            }
            Message::DraftTested(connection, result) => match result {
                Ok(summary) => {
                    if is_onedrive_rclone_mirror(&connection)
                        && (self.onedrive_editor_pending.is_some()
                            || !onedrive_validation_matches_draft(&self.draft, &connection))
                    {
                        self.last_notice = Some(fl!("onedrive-editor-result-discarded"));
                        return Task::none();
                    }
                    if connection.provider == Provider::Teams {
                        if !sharepoint_verification_matches_draft(&self.draft, &connection) {
                            self.last_notice = Some(fl!("notice-sharepoint-test-changed"));
                            return Task::none();
                        }
                        self.draft.teams_drive_id = connection
                            .teams_identity
                            .as_ref()
                            .map_or_else(String::new, |identity| identity.drive_id.clone());
                    }
                    self.validated_draft = Some(ValidatedDraft {
                        connection: connection.clone(),
                        summary: summary.clone(),
                    });
                    self.last_notice = Some(fl!(
                        "notice-test-passed",
                        name = connection.name.as_str(),
                        summary = summary
                    ));
                }
                Err(error) => {
                    self.validated_draft = None;
                    self.last_notice = Some(fl!(
                        "notice-test-failed",
                        name = connection.name.as_str(),
                        error = error
                    ));
                }
            },
            Message::SaveDraft => {
                return self.save_draft();
            }
            Message::SaveDraftValidated(connection, validation) => {
                if is_onedrive_rclone_mirror(&connection)
                    && (self.onedrive_editor_pending.is_some()
                        || !onedrive_validation_matches_draft(&self.draft, &connection))
                {
                    self.last_notice = Some(fl!("onedrive-editor-result-discarded"));
                    return Task::none();
                }
                if connection.provider == Provider::Teams
                    && !sharepoint_verification_matches_draft(&self.draft, &connection)
                {
                    self.last_notice = Some(fl!("notice-sharepoint-save-changed"));
                    return Task::none();
                }
                return match validation {
                    Ok(summary) => self.save_validated_draft(connection, Some(summary)),
                    Err(error) => {
                        self.last_notice = Some(fl!(
                            "notice-save-blocked",
                            name = connection.name.as_str(),
                            error = error
                        ));
                        Task::none()
                    }
                };
            }
            Message::DraftSaved(notice) => {
                self.pending_shared_remote_ack = None;
                self.last_notice = Some(notice);
                return notify_runtime_task();
            }
            Message::ConfirmImport(index) => {
                return self.confirm_import(index);
            }
            Message::RemoveConnection(connection_id) => {
                return self.remove_connection_with_confirmation(connection_id);
            }
            Message::RemoveCompleted(notice) => {
                self.pending_remove = None;
                self.removing_connection = None;
                self.last_notice = Some(notice);
                return notify_runtime_task();
            }
            Message::Refresh => {
                self.runtime_poll_obsolete = self.runtime_poll_pending;
                self.runtime_pending = true;
                return runtime_request_task(RuntimeCommand::Refresh);
            }
            Message::NoticeTick(now) => {
                if let Some(started_at) = self.last_notice_at
                    && popup_notice_expired(started_at, self.popup_opened_at, now)
                {
                    self.last_notice = None;
                    self.last_notice_at = None;
                }
            }
            Message::PendingCountTick => {
                if self.popup.is_none() || self.standalone || self.network_ready == Some(false) {
                    return Task::none();
                }
                let mounts = self.mount_entries.clone();
                let active = self
                    .config
                    .document
                    .connections
                    .iter()
                    .filter(|connection| connection.enabled && is_rclone_online_mount(connection))
                    .filter(|connection| {
                        mounts
                            .iter()
                            .any(|entry| entry.target == connection.local_path)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let mirrors = self
                    .config
                    .document
                    .connections
                    .iter()
                    .filter(|connection| {
                        connection.enabled
                            && (is_legacy_onedrive_mirror(connection)
                                || is_rclone_offline_mirror(connection))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let active_ids = active
                    .iter()
                    .chain(mirrors.iter())
                    .map(|connection| connection.id)
                    .collect::<BTreeSet<_>>();
                self.pending_counts.retain(|id, _| active_ids.contains(id));
                let generation = self.pending_poll_generation;
                let mut tasks = active
                    .into_iter()
                    .filter_map(|connection| {
                        if !self.pending_poll_inflight.insert(connection.id) {
                            return None;
                        }
                        let rc_socket = match rclone_mount_plan(
                            &connection,
                            &default_runtime_root(),
                            &default_cache_root(),
                        ) {
                            Ok(plan) => plan.rc_socket,
                            Err(_) => {
                                self.pending_poll_inflight.remove(&connection.id);
                                return None;
                            }
                        };
                        Some(Task::perform(
                            async move {
                                let result = pending_count::rclone_online_pending(
                                    &app_command_runner(),
                                    &rc_socket,
                                )
                                .await
                                .map(|count| count.files);
                                (connection, result)
                            },
                            move |(connection, result)| {
                                cosmic::Action::App(Message::PendingCountPolled(
                                    generation,
                                    connection,
                                    PendingSource::RcloneOnline,
                                    result,
                                ))
                            },
                        ))
                    })
                    .collect::<Vec<_>>();
                for connection in mirrors {
                    let live_rclone = is_rclone_offline_mirror(&connection)
                        && self.cached_offline_state(&connection) == SyncRuntimeState::Running;
                    let preview_count = is_onedrive_rclone_mirror(&connection);
                    if preview_count && live_rclone {
                        self.pending_counts.remove(&connection.id);
                        continue;
                    }
                    if is_rclone_offline_mirror(&connection) && !live_rclone && !preview_count {
                        self.pending_counts.remove(&connection.id);
                        continue;
                    }
                    if self.mirror_operations_pending.contains(&connection.id)
                        || self.pending_poll_inflight.contains(&connection.id)
                        || self
                            .pending_counts
                            .get(&connection.id)
                            .is_some_and(|observation| {
                                observation.checked_at.elapsed()
                                    < if live_rclone {
                                        Duration::from_secs(5)
                                    } else {
                                        Duration::from_secs(60)
                                    }
                            })
                    {
                        continue;
                    }
                    self.pending_poll_inflight.insert(connection.id);
                    let source = if preview_count {
                        PendingSource::OneDriveRclonePreview {
                            age: Duration::ZERO,
                        }
                    } else if live_rclone {
                        PendingSource::RcloneMirrorLive
                    } else {
                        PendingSource::OneDriveMirror
                    };
                    tasks.push(Task::perform(
                        async move {
                            let mut source = source;
                            let result = if preview_count {
                                let plan = rclone_bisync_plan(&connection, &default_work_root());
                                match plan {
                                    Ok(plan) => pending_count::rclone_preview_pending(
                                        &plan.work_directory,
                                        connection.id,
                                    )
                                    .await
                                    .map(|count| {
                                        source =
                                            PendingSource::OneDriveRclonePreview { age: count.age };
                                        count.files
                                    }),
                                    Err(_) => Err(fl!("onedrive-preview-count-unavailable")),
                                }
                            } else if live_rclone {
                                rclone_mirror_live_pending_count(&connection).await
                            } else {
                                onedrive_mirror_pending_count(&connection).await
                            };
                            (connection, source, result)
                        },
                        move |(connection, source, result)| {
                            cosmic::Action::App(Message::PendingCountPolled(
                                generation, connection, source, result,
                            ))
                        },
                    ));
                }
                return Task::batch(tasks);
            }
            Message::PendingCountPolled(generation, connection, source, result) => {
                self.pending_poll_inflight.remove(&connection.id);
                if generation != self.pending_poll_generation
                    || self.network_ready == Some(false)
                    || !self
                        .config
                        .document
                        .connections
                        .iter()
                        .any(|saved| saved == &connection)
                    || (is_rclone_online_mount(&connection)
                        && !self
                            .mount_entries
                            .iter()
                            .any(|entry| entry.target == connection.local_path))
                    || (matches!(source, PendingSource::RcloneMirrorLive)
                        && self.cached_offline_state(&connection) != SyncRuntimeState::Running)
                    || (matches!(source, PendingSource::OneDriveRclonePreview { .. })
                        && (self.mirror_operations_pending.contains(&connection.id)
                            || self.cached_offline_state(&connection) == SyncRuntimeState::Running))
                {
                    return Task::none();
                }
                self.pending_counts.insert(
                    connection.id,
                    PendingObservation {
                        files: result.as_ref().ok().copied(),
                        checked_at: match source {
                            PendingSource::OneDriveRclonePreview { age } => {
                                Instant::now().checked_sub(age).unwrap_or_else(Instant::now)
                            }
                            _ => Instant::now(),
                        },
                        reason: result.err(),
                        source,
                    },
                );
            }
            Message::VpnStatusChecked(ready) => {
                self.vpn_ready = ready;
                let mounts = self.mount_entries.clone();
                let degraded = self
                    .config
                    .document
                    .connections
                    .iter()
                    .filter(|connection| {
                        connection.enabled
                            && matches!(connection.mode, ConnectionMode::OnlineMount(_))
                            && mounts
                                .iter()
                                .any(|entry| entry.target == connection.local_path)
                            && connection.vpn_profile_id.is_some_and(|profile_id| {
                                self.vpn_ready.get(&profile_id) == Some(&false)
                            })
                    })
                    .map(|connection| {
                        self.pending_counts.remove(&connection.id);
                        connection.name.clone()
                    })
                    .collect::<Vec<_>>();
                if !degraded.is_empty() {
                    self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                    self.last_notice = Some(fl!(
                        "notice-mounted-vpn-unavailable",
                        names = degraded.join(", ")
                    ));
                    self.last_notice_at = Some(Instant::now());
                }
            }
            Message::NetworkStatusChecked(ready) => {
                self.network_ready = Some(ready);
                if !ready {
                    self.pending_counts.clear();
                    self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                }
            }
        }

        Task::batch(inspection_tasks)
    }

    fn runtime_status(&self) -> runtime_ipc::Status {
        runtime_ipc::Status {
            unmount_before_sleep: self.config.document.unmount_before_sleep,
            restore_after_wake: self.config.document.restore_after_wake,
            preload: self.config.document.preload,
            sleep_status: self.sleep_status.clone(),
            sleep_needs_attention: self.sleep_needs_attention,
            operation_revision: self.runtime_operation_revision,
            operation_completions: self.runtime_operation_completions.clone(),
        }
    }

    fn complete_routed_operation(&mut self, connection_id: ConnectionId, operation: Operation) {
        if self.runtime_routed_operations.get(&connection_id) != Some(&operation) {
            return;
        }
        self.runtime_routed_operations.remove(&connection_id);
        self.runtime_operation_revision = self.runtime_operation_revision.wrapping_add(1).max(1);
        self.runtime_operation_completions
            .push(runtime_ipc::OperationCompletion {
                revision: self.runtime_operation_revision,
                connection_id,
                operation,
                notice: self
                    .last_notice
                    .clone()
                    .unwrap_or_else(|| crate::fl!("ui-app-operation-completed")),
            });
        if self.runtime_operation_completions.len() > RUNTIME_OPERATION_COMPLETION_LIMIT {
            let excess =
                self.runtime_operation_completions.len() - RUNTIME_OPERATION_COMPLETION_LIMIT;
            self.runtime_operation_completions.drain(..excess);
        }
    }

    fn reconcile_runtime_operation_completion(&mut self, status: &runtime_ipc::Status) {
        let Some(wait) = self.runtime_operation_wait else {
            return;
        };
        let Some(after_revision) = wait.after_revision else {
            return;
        };
        let Some(completion) = status
            .operation_completions
            .iter()
            .rev()
            .find(|completion| {
                completion.revision > after_revision
                    && completion.connection_id == wait.connection_id
                    && completion.operation == wait.operation
            })
        else {
            return;
        };
        self.last_notice = Some(completion.notice.clone());
        self.last_notice_at = Some(Instant::now());
        self.runtime_operation_wait = None;
    }

    fn apply_runtime_command(
        &mut self,
        command: RuntimeCommand,
    ) -> Result<runtime_ipc::Status, String> {
        if command == RuntimeCommand::Status {
            return Ok(self.runtime_status());
        }
        self.config = load_runtime_config()?;
        let storage = AppConfigStorage::runtime()?;
        save_runtime_preference(&mut self.config, &storage, command)?;
        if let RuntimeCommand::SetUnmount(value) = command {
            self.sleep_status = value.then(|| fl!("sleep-listener-starting"));
        }
        Ok(self.runtime_status())
    }

    fn view_general_settings(&self) -> Element<'_, Message> {
        let enabled = self.runtime_available && !self.runtime_pending;
        let actions = widget::Row::new()
            .spacing(8)
            .push(field_with_help(
                widget::button::suggested(fl!("add-connection"))
                    .on_press(Message::OpenAddConnection),
                fl!("settings-add-help"),
            ))
            .push(field_with_help(
                widget::button::standard(fl!("refresh"))
                    .on_press_maybe((!self.runtime_pending).then_some(Message::Refresh)),
                fl!("settings-refresh-help"),
            ))
            .push(field_with_help(
                widget::button::standard(fl!("reorder-connections"))
                    .on_press(Message::OpenReorderConnections),
                fl!("settings-reorder-help"),
            ));
        let mut action_section = widget::Column::new()
            .spacing(8)
            .push(widget::text::title4(fl!("settings-connections")))
            .push(actions);
        if self.runtime_pending && !self.sleep_settings_pending {
            action_section = action_section.push(widget::text::body(fl!("runtime-waiting")));
        }
        if let Some(error) = &self.runtime_error {
            action_section = action_section.push(widget::text::body(error.clone()));
        } else if !self.runtime_available {
            action_section = action_section.push(widget::text::body(fl!("runtime-connecting")));
        }
        if let Some(notice) = &self.last_notice {
            action_section = action_section.push(widget::text::body(notice.clone()));
        }
        let mut sleep_section = widget::Column::new()
            .spacing(8)
            .push(widget::text::title4(fl!("sleep-title")))
            .push(field_with_help(
                widget::toggler(self.config.document.unmount_before_sleep)
                    .label(fl!("sleep-unmount"))
                    .spacing(8)
                    .on_toggle_maybe(enabled.then_some(Message::UnmountBeforeSleep)),
                fl!("sleep-unmount-help"),
            ))
            .push(field_with_help(
                widget::toggler(self.config.document.restore_after_wake)
                    .label(fl!("sleep-restore"))
                    .spacing(8)
                    .on_toggle_maybe(
                        (enabled && self.config.document.unmount_before_sleep)
                            .then_some(Message::RestoreAfterWake),
                    ),
                fl!("sleep-restore-help"),
            ))
            .push(widget::text::body(fl!("sleep-online-only")));
        if self.sleep_settings_pending {
            sleep_section = sleep_section.push(widget::text::body(fl!("sleep-saving")));
        }
        if let Some(notice) = &self.sleep_notice {
            sleep_section = sleep_section.push(widget::text::body(notice.clone()));
        }
        if let Some(status) = &self.sleep_status {
            sleep_section = sleep_section.push(widget::text::body(status.clone()));
        }
        let preload_save = widget::button::standard(fl!("preload-save")).on_press_maybe(
            (enabled && self.preload_input_dirty).then_some(Message::SavePreloadSettings),
        );
        let mut preload_section = widget::Column::new()
            .spacing(8)
            .push(widget::text::title4(fl!("preload-title")))
            .push(preload_policy_row(
                fl!("provider-google-drive"),
                PreloadProvider::GoogleDrive,
                &self.preload_draft.google_drive,
            ))
            .push(preload_policy_row(
                fl!("provider-onedrive"),
                PreloadProvider::OneDrive,
                &self.preload_draft.onedrive,
            ))
            .push(preload_policy_row(
                fl!("provider-box"),
                PreloadProvider::Box,
                &self.preload_draft.box_provider,
            ))
            .push(preload_policy_row(
                fl!("provider-teams"),
                PreloadProvider::SharePoint,
                &self.preload_draft.sharepoint,
            ))
            .push(preload_policy_row(
                fl!("provider-smb"),
                PreloadProvider::Smb,
                &self.preload_draft.smb,
            ))
            .push(preload_policy_row(
                fl!("sftp-provider"),
                PreloadProvider::Sftp,
                &self.preload_draft.sftp,
            ))
            .push(preload_save);
        if self.preload_settings_pending {
            preload_section = preload_section.push(widget::text::body(fl!("preload-saving")));
        }
        if let Some(notice) = &self.preload_notice {
            preload_section = preload_section.push(widget::text::body(notice.clone()));
        }
        let content = widget::Column::new()
            .spacing(16)
            .push(action_section)
            .push(sleep_section)
            .push(preload_section);
        // Keep the scrollbar at the window edge while padding only its content.
        widget::container(widget::scrollable(
            widget::container(content).padding(24).width(Length::Fill),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .class(cosmic::style::Container::List)
        .into()
    }

    fn view_reorder_connections(&self) -> Element<'_, Message> {
        let mut content = widget::Column::new().spacing(16);
        content = content.push(widget::text::title2(fl!("reorder-connections-title")));
        if let Some(notice) = &self.last_notice {
            content = content.push(widget::text::body(notice.clone()));
        }

        let count = self.config.document.connections.len();
        if count == 0 {
            content = content.push(widget::text::body(fl!("reorder-connections-empty")));
        } else {
            let position_labels: Vec<String> = (1..=count).map(|n| n.to_string()).collect();
            let mut rows = widget::list_column().list_item_padding([8, 0]);
            for (index, connection) in self.config.document.connections.iter().enumerate() {
                let connection_id = connection.id;
                let connection_name = connection.name.clone();
                let row = widget::Row::new()
                    .spacing(6)
                    .width(Length::Fill)
                    .align_y(Alignment::Center)
                    .push(
                        widget::container(widget::text::body(format!("{}.", index + 1)))
                            .width(Length::Fixed(24.0)),
                    )
                    .push(
                        widget::container(widget::text::body(connection_name))
                            .width(Length::Fill)
                            .align_x(Alignment::Start),
                    )
                    .push(widget::dropdown(
                        position_labels.clone(),
                        Some(index),
                        move |selected_index| {
                            Message::ReorderConnection(connection_id, selected_index + 1)
                        },
                    ));
                rows = rows.add(row);
            }
            content = content.push(widget::container(rows).width(Length::Fill));
        }

        widget::container(widget::scrollable(
            widget::container(content).padding(24).width(Length::Fill),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .class(cosmic::style::Container::List)
        .into()
    }

    fn view_popup(&self) -> Element<'_, Message> {
        let snapshot = self.controller_snapshot();
        let state = restore(&snapshot);
        let mut header = widget::list_column()
            .style(cosmic::style::Container::Transparent)
            .add(
                widget::Row::new()
                    .width(Length::Fill)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        widget::container(widget::text::title4(fl!("app-title")))
                            .width(Length::Fill),
                    )
                    .push(field_with_help_at(
                        widget::button::icon(
                            widget::icon::from_svg_bytes(include_bytes!(
                                "../resources/settings-symbolic.svg"
                            ))
                            .symbolic(true),
                        )
                        .name(fl!("settings-button-name"))
                        .on_press(Message::OpenGeneralSettings),
                        fl!("settings-button-help"),
                        widget::tooltip::Position::Bottom,
                    )),
            );
        let show_aggregate = state.aggregate.kind != AggregateKind::Healthy
            || self
                .popup_opened_at
                .is_none_or(|opened_at| opened_at.elapsed() < POPUP_HEALTHY_AGGREGATE_HIDE_AFTER);
        if self
            .config
            .document
            .connections
            .iter()
            .any(|connection| !self.unit_status_fresh(connection.id))
        {
            header = header.add(widget::text::body(
                if self.unit_observation.failed.is_empty() {
                    fl!("unit-status-refreshing")
                } else {
                    fl!("unit-status-refresh-unavailable")
                },
            ));
        } else if show_aggregate {
            header = header.add(widget::text::body(aggregate_label(&state.aggregate)));
        }

        let mut rows = widget::list_column().style(cosmic::style::Container::Transparent);
        if state.rows.is_empty() {
            rows = rows.add(widget::text::body(fl!("popup-no-connections")));
        }

        for row in &state.rows {
            rows = rows.add(self.view_connection_row(row));
        }

        let mut content = widget::Column::new().spacing(0).push(header);

        if let Some(notice) = &self.last_notice {
            content = content.push(
                widget::container(widget::text::body(notice.clone()))
                    .padding([8, POPUP_ACTION_HORIZONTAL_PADDING])
                    .width(Length::Fill),
            );
        }

        let repair_rows = state
            .rows
            .iter()
            .filter(|row| repair_visible(row, &self.repair_operations_pending))
            .collect::<Vec<_>>();
        for row in &repair_rows {
            let armed = self
                .pending_repair
                .is_some_and(|(connection_id, _)| connection_id == row.id);
            let label = if armed {
                fl!(
                    "operation-confirm-repair-connection",
                    name = row.name.as_str()
                )
            } else {
                fl!("operation-repair-connection", name = row.name.as_str())
            };
            content = content.push(
                widget::container(
                    widget::button::text(label)
                        .on_press(Message::OperationRequested(row.id, Operation::Repair)),
                )
                .padding([4, POPUP_ACTION_HORIZONTAL_PADDING])
                .width(Length::Fill),
            );
        }

        if let Some(error) = &self.runtime_error {
            content = content.push(field_with_help(
                widget::text::caption(fl!("settings-unavailable")),
                error.clone(),
            ));
        }
        if self.sleep_needs_attention {
            content = content.push(field_with_help(
                widget::button::text(fl!("sleep-attention")).on_press(Message::OpenGeneralSettings),
                self.sleep_status.clone().unwrap_or_default(),
            ));
        }
        let list_limit = 360.0
            - if self.last_notice.is_some() {
                60.0
            } else {
                0.0
            }
            - if self.sleep_needs_attention {
                40.0
            } else {
                0.0
            }
            - if self.runtime_error.is_some() {
                40.0
            } else {
                0.0
            }
            - repair_rows.len() as f32 * 40.0;
        content = content.push(widget::divider::horizontal::default());
        content = content.push(widget::scrollable(rows).height(Length::Fixed(
            popup_connection_scroll_height(state.rows.len(), state.rows.is_empty()).min(list_limit),
        )));
        self.core
            .applet
            .popup_container(widget::container(content).class(cosmic::style::Container::List))
            .into()
    }

    fn view_connection_row(&self, row: &ConnectionRowState) -> Element<'static, Message> {
        let primary = primary_operation(row);
        let decision = primary.map(|operation| decide_operation(row, operation));
        let primary_enabled = row.persistent_active;
        let primary_help = primary.map_or_else(
            || fl!("current-status", status = status_label(&row.status)),
            |operation| {
                let action_label = row_operation_label(&row.status, operation);
                let status = status_label(&row.status);
                let availability = decision
                    .as_ref()
                    .and_then(|decision| decision.reason.as_deref())
                    .map_or_else(
                        || fl!("toggle-operation", operation = action_label.clone()),
                        |reason| {
                            fl!(
                                "operation-unavailable-help",
                                operation = action_label.clone(),
                                reason = reason
                            )
                        },
                    );
                fl!(
                    "current-status-detail",
                    status = status,
                    availability = availability
                )
            },
        );
        let row_id = row.id;
        let fresh = self.unit_status_fresh(row.id);
        let primary_help = if fresh {
            primary_help
        } else {
            fl!("unit-status-wait")
        };
        let primary_control: Element<'static, Message> =
            if let Some(operation) = primary.filter(|_| fresh) {
                widget::toggler(primary_enabled)
                    .on_toggle(move |_| Message::OperationRequested(row_id, operation))
                    .into()
            } else {
                widget::toggler(primary_enabled).into()
            };
        let display_name = popup_connection_display_name(&row.name);
        let stale_after = if row.mode == AccessMode::OfflineMirror {
            Duration::from_secs(180)
        } else {
            PENDING_COUNT_STALE_AFTER
        };
        let unknown_label = if primary_enabled { "+" } else { "-" };
        let unknown_help = |reason: String| {
            if primary_enabled {
                fl!("pending-count-active-unknown", reason = reason)
            } else {
                fl!("pending-count-inactive-unknown", reason = reason)
            }
        };
        let (pending_label, pending_help) = if matches!(
            row.status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Degraded)
        ) {
            (
                "!".to_owned(),
                fl!("pending-count-mounted-connection-unavailable"),
            )
        } else {
            match self.pending_counts.get(&row.id) {
                Some(observation) if observation.checked_at.elapsed() < stale_after => {
                    match observation.files {
                        Some(files) => (
                            files.to_string(),
                            if matches!(observation.source, PendingSource::RcloneMirrorLive) {
                                fl!("pending-count-mirror-live-help", count = files)
                            } else if matches!(
                                observation.source,
                                PendingSource::OneDriveRclonePreview { .. }
                            ) {
                                fl!(
                                    "onedrive-preview-count-help",
                                    count = files,
                                    age = observation.checked_at.elapsed().as_secs()
                                )
                            } else if matches!(observation.source, PendingSource::OneDriveMirror) {
                                fl!(
                                    "pending-count-mirror-help",
                                    count = files,
                                    age = observation.checked_at.elapsed().as_secs()
                                )
                            } else {
                                fl!("pending-count-online-help", count = files)
                            },
                        ),
                        None => (
                            unknown_label.to_owned(),
                            unknown_help(
                                observation
                                    .reason
                                    .clone()
                                    .unwrap_or_else(|| fl!("pending-count-unknown")),
                            ),
                        ),
                    }
                }
                _ if row.provider == Provider::OneDrive && row.mode == AccessMode::OnlineMount => (
                    unknown_label.to_owned(),
                    unknown_help(fl!("pending-count-onedriver-unavailable")),
                ),
                _ => (
                    unknown_label.to_owned(),
                    unknown_help(fl!("pending-count-unknown")),
                ),
            }
        };
        let name_button = widget::button::custom(
            widget::container(widget::text::body(display_name))
                .width(Length::Fill)
                .align_x(Alignment::Start),
        )
        .width(Length::Fill)
        .class(cosmic::theme::Button::Text)
        .on_press(Message::OpenModifyConnection(row.id));
        let row_content = widget::Row::new()
            .spacing(8)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .push(
                widget::container(field_with_help(
                    name_button,
                    fl!("modify-connection-help", name = row.name.as_str()),
                ))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            )
            .push(
                widget::container(field_with_help(
                    widget::text::body(pending_label),
                    pending_help,
                ))
                .width(Length::Fixed(48.0))
                .align_x(Alignment::Center),
            )
            .push(
                widget::container(field_with_help(primary_control, primary_help))
                    .align_x(Alignment::End),
            );

        widget::container(row_content)
            .padding([
                POPUP_CONNECTION_ROW_VERTICAL_PADDING,
                POPUP_CONNECTION_ROW_HORIZONTAL_PADDING,
            ])
            .width(Length::Fill)
            .into()
    }

    fn view_settings_window(&self) -> Element<'_, Message> {
        let mut content = widget::list_column().add(widget::text::title2(match self.window_mode {
            WindowMode::AddConnection => fl!("add-connection"),
            WindowMode::ModifyConnection(_) => fl!("modify-connection"),
            WindowMode::ImportLegacy => fl!("settings-import-title"),
            WindowMode::GeneralSettings | WindowMode::ReorderConnections => {
                window_title(self.window_mode)
            }
        }));

        if let Some(notice) = &self.last_notice {
            content = content.add(widget::text::body(notice.clone()));
        }

        match self.window_mode {
            WindowMode::GeneralSettings => unreachable!("General Settings has its own view"),
            WindowMode::ReorderConnections => unreachable!("Reorder Connections has its own view"),
            WindowMode::AddConnection | WindowMode::ModifyConnection(_) => {
                content = content.add(self.view_editor_actions());
                content = self.view_wizard(content);
            }
            WindowMode::ImportLegacy => {
                content = content.add(widget::settings::item(
                    fl!("settings-import-title"),
                    widget::text::body(fl!("settings-import-guidance")),
                ));
                if self.import_previews.is_empty() {
                    content = content.add(widget::settings::item(
                        fl!("import-preview"),
                        widget::text::body(fl!("import-preview-empty")),
                    ));
                } else {
                    for (index, preview) in self.import_previews.iter().enumerate() {
                        content = content.add(widget::settings::item(
                            preview.original_unit_name.clone(),
                            widget::Column::new()
                                .spacing(8)
                                .push(widget::text::body(import_preview_summary(preview)))
                                .push(
                                    widget::button::suggested(fl!("review-connection"))
                                        .on_press_maybe(
                                            (!preview.active_conflict
                                                && !preview.local_target_conflict)
                                                .then_some(Message::ConfirmImport(index)),
                                        ),
                                ),
                        ));
                    }
                }
            }
        }

        let window_content = widget::container(widget::scrollable(content))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .class(cosmic::style::Container::Background);
        window_content.into()
    }

    fn view_wizard<'a>(
        &'a self,
        mut content: widget::ListColumn<'a, Message>,
    ) -> widget::ListColumn<'a, Message> {
        let modify_locked = matches!(self.window_mode, WindowMode::ModifyConnection(_));
        if modify_locked
            && self.draft.provider == Provider::OneDrive
            && self.draft.access_mode == AccessMode::OfflineMirror
            && self.draft.legacy_onedrive
        {
            content = content.add(section_row(
                fl!("onedrive-legacy-deprecation-title"),
                widget::text::body(fl!("onedrive-legacy-deprecation-help")),
            ));
        }
        let provider_choices = choice_row(vec![
            provider_choice(
                fl!("provider-onedrive"),
                Provider::OneDrive,
                self.draft.provider,
                modify_locked,
            ),
            provider_choice(
                fl!("provider-teams"),
                Provider::Teams,
                self.draft.provider,
                modify_locked,
            ),
            provider_choice(
                fl!("provider-google-drive"),
                Provider::GoogleDrive,
                self.draft.provider,
                modify_locked,
            ),
            provider_choice(
                fl!("provider-box"),
                Provider::Box,
                self.draft.provider,
                modify_locked,
            ),
            provider_choice(
                fl!("provider-smb"),
                Provider::Smb,
                self.draft.provider,
                modify_locked,
            ),
            provider_choice(
                fl!("sftp-provider"),
                Provider::Sftp,
                self.draft.provider,
                modify_locked,
            ),
        ]);
        let mode_choices = choice_row(vec![
            mode_choice(
                fl!("access-online"),
                AccessMode::OnlineMount,
                self.draft.access_mode,
                modify_locked,
            ),
            mode_choice(
                fl!("access-offline"),
                AccessMode::OfflineMirror,
                self.draft.access_mode,
                modify_locked,
            ),
        ]);

        content = content
            .add(if modify_locked {
                section_row(fl!("field-provider"), provider_choices)
            } else {
                section_row_with_help(
                    fl!("field-provider"),
                    fl!("provider-choice-help"),
                    provider_choices,
                )
            })
            .add(if modify_locked {
                section_row(fl!("field-access-mode"), mode_choices)
            } else {
                section_row_with_help(
                    fl!("field-access-mode"),
                    fl!("access-mode-help"),
                    mode_choices,
                )
            })
            .add(section_row(
                fl!("field-connection"),
                widget::Column::new()
                    .spacing(8)
                    .push(field_with_help(
                        widget::text_input::text_input(
                            fl!("connection-name-placeholder"),
                            &self.draft.name,
                        )
                        .on_input(Message::DraftName),
                        fl!("connection-name-help"),
                    ))
                    .push(self.view_remote_account_fields()),
            ))
            .add(section_row(
                local_target_label(self.draft.access_mode),
                self.view_local_target_picker(),
            ));

        content = match self.draft.access_mode {
            AccessMode::OnlineMount => {
                let mut online_settings = widget::Column::new().spacing(8).push(field_with_help(
                    widget::text_input::text_input(
                        fl!("cache-limit-placeholder"),
                        &self.draft.cache_limit_gib,
                    )
                    .on_input(Message::DraftCacheLimit),
                    fl!("cache-limit-help"),
                ));
                if matches!(self.draft.provider, Provider::Smb | Provider::Sftp) {
                    online_settings = online_settings.push(field_with_help(
                        toggle_button(
                            if self.draft.provider == Provider::Sftp {
                                sftp_preload_global_label()
                            } else {
                                smb_preload_global_label()
                            },
                            self.draft.connection_preload_use_global,
                            Message::DraftConnectionPreloadUseGlobal,
                        ),
                        fl!("preload-global-help"),
                    ));
                    if !self.draft.connection_preload_use_global {
                        online_settings = online_settings
                            .push(toggle_button(
                                fl!("preload-after-mount"),
                                self.draft.connection_preload_enabled,
                                Message::DraftConnectionPreloadEnabled,
                            ))
                            .push(
                                widget::Row::new()
                                    .spacing(8)
                                    .push(
                                        widget::text_input::text_input(
                                            "60",
                                            &self.draft.connection_preload_seconds,
                                        )
                                        .on_input(Message::DraftConnectionPreloadSeconds)
                                        .width(Length::Fixed(80.0)),
                                    )
                                    .push(widget::text::body(fl!("unit-seconds")))
                                    .push(
                                        widget::text_input::text_input(
                                            "3",
                                            &self.draft.connection_preload_depth,
                                        )
                                        .on_input(Message::DraftConnectionPreloadDepth)
                                        .width(Length::Fixed(60.0)),
                                    )
                                    .push(widget::text::body(fl!("unit-levels"))),
                            );
                    }
                }
                content.add(section_row(fl!("online-settings"), online_settings))
            }
            AccessMode::OfflineMirror => {
                let settings = widget::Column::new()
                    .spacing(8)
                    .push(field_with_help(
                        widget::text_input::text_input(
                            fl!("sync-interval-placeholder"),
                            &self.draft.sync_interval_minutes,
                        )
                        .on_input(Message::DraftSyncInterval),
                        fl!("sync-interval-help"),
                    ))
                    .push(field_with_help(
                        toggle_button(
                            fl!("sync-metered"),
                            self.draft.sync_on_metered,
                            Message::DraftSyncOnMetered,
                        ),
                        fl!("sync-metered-help"),
                    ));
                let requires_recovery_selection = running_in_flatpak()
                    && !(self.draft.provider == Provider::OneDrive && self.draft.legacy_onedrive);
                let mut settings = settings.push(field_with_safety_help(
                    widget::row::with_capacity(2)
                        .spacing(8)
                        .push(
                            widget::text_input::text_input(
                                if requires_recovery_selection {
                                    fl!("recovery-placeholder-browse")
                                } else {
                                    fl!("recovery-placeholder")
                                },
                                &self.draft.recovery_directory,
                            )
                            .on_input(Message::DraftRecoveryDirectory)
                            .width(Length::Fill),
                        )
                        .push(field_with_help(
                            widget::button::standard(fl!("browse"))
                                .on_press(Message::OpenRecoveryFolderPicker),
                            fl!("recovery-browse-help"),
                        )),
                    fl!("recovery-safety"),
                    if requires_recovery_selection {
                        fl!("recovery-help-browse")
                    } else {
                        fl!("recovery-help")
                    },
                ));
                if !requires_recovery_selection && self.draft.recovery_directory.trim().is_empty() {
                    settings = settings.push(widget::text::body(fl!(
                        "recovery-automatic",
                        path = recovery_directory_placeholder(&self.draft)
                    )));
                }
                content.add(section_row(fl!("offline-settings"), settings))
            }
        };

        let mut vpn_section = widget::Column::new()
            .spacing(8)
            .push(self.view_vpn_choices());

        if self.draft.vpn_profile_id.is_some() {
            vpn_section = vpn_section.push(field_with_help(
                toggle_button(
                    fl!("vpn-disconnect"),
                    self.draft.disconnect_vpn_when_unused,
                    Message::DraftDisconnectVpn,
                ),
                fl!("vpn-disconnect-help"),
            ));
        }

        content
            .add(section_row(fl!("vpn-dependency"), vpn_section))
            .add(section_row(
                fl!("information"),
                widget::text::body(draft_summary_text(&self.draft)),
            ))
    }

    fn view_editor_actions(&self) -> Element<'static, Message> {
        let primary_ready = self.draft_primary_actions_ready();
        let mut primary_row = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_help(
                action_button(fl!("test-connection"), primary_ready, Message::TestDraft),
                fl!("test-connection-help"),
            ))
            .push(field_with_safety_help(
                action_button(
                    fl!("save-connection"),
                    self.draft.provider == Provider::Sftp || primary_ready,
                    Message::SaveDraft,
                ),
                fl!("save-connection-safety"),
                fl!("save-connection-help"),
            ));

        primary_row = primary_row.push_maybe(self.view_onedrive_setup_actions());

        match self.window_mode {
            WindowMode::AddConnection => {
                if self.draft_uses_rclone() {
                    primary_row = primary_row
                        .push(field_with_help(
                            widget::button::standard(fl!("rclone-detect"))
                                .on_press(Message::DetectRcloneRemotes),
                            fl!(
                                "rclone-detect-help",
                                provider = provider_editor_label(self.draft.provider)
                            ),
                        ))
                        .push(self.view_create_rclone_remote_action());
                }
            }
            WindowMode::ModifyConnection(connection_id) => {
                let enable_label = if self.draft.enabled {
                    fl!("disable")
                } else {
                    fl!("enable")
                };
                let enable_button: Element<'_, Message> = if self.draft.enabled {
                    soft_destructive_button(enable_label, Message::DraftEnabled(false))
                } else {
                    widget::button::suggested(enable_label)
                        .on_press(Message::DraftEnabled(true))
                        .into()
                };
                let mut modify_row = widget::Row::new().spacing(8).align_y(Alignment::Center);
                if self.draft.provider == Provider::Sftp {
                    modify_row = modify_row.push(self.view_sftp_action(false));
                }
                if self.draft.provider == Provider::Smb {
                    modify_row = modify_row.push(field_with_help(
                        widget::button::standard(fl!("smb-update"))
                            .on_press(Message::CreateSmbRcloneRemote),
                        fl!("smb-update-help"),
                    ));
                }
                if self.draft.provider == Provider::GoogleDrive {
                    modify_row = modify_row.push(field_with_help(
                        widget::button::standard(fl!("google-drive-update"))
                            .on_press(Message::ApplyGoogleDriveRcloneRemote),
                        fl!("google-drive-update-help"),
                    ));
                }
                if self.draft.provider == Provider::Teams {
                    modify_row = modify_row.push(self.view_sharepoint_setup_action());
                }
                if self.saved_connection_is_offline_mirror(connection_id) {
                    modify_row = modify_row
                        .push(field_with_help(
                            widget::button::standard(operation_label(
                                Operation::PreviewInitialSync,
                            ))
                            .on_press(Message::OperationRequested(
                                connection_id,
                                Operation::PreviewInitialSync,
                            )),
                            fl!("preview-help"),
                        ))
                        .push(field_with_safety_help(
                            widget::button::standard(operation_label(Operation::SyncNow)).on_press(
                                Message::OperationRequested(connection_id, Operation::SyncNow),
                            ),
                            fl!("sync-now-safety"),
                            fl!("sync-now-help"),
                        ));
                }
                modify_row = modify_row
                    .push(field_with_help(enable_button, fl!("disable-help")))
                    .push(field_with_help(
                        self.view_remove_connection_button(connection_id),
                        fl!("remove-help"),
                    ));

                if self.draft.provider == Provider::OneDrive
                    && self.draft.access_mode == AccessMode::OfflineMirror
                {
                    return widget::Column::new()
                        .spacing(8)
                        .push(primary_row)
                        .push(modify_row)
                        .into();
                }

                return primary_row.push(modify_row).into();
            }
            WindowMode::ImportLegacy
            | WindowMode::GeneralSettings
            | WindowMode::ReorderConnections => {}
        }

        primary_row.into()
    }

    fn view_remove_connection_button(
        &self,
        connection_id: ConnectionId,
    ) -> Element<'static, Message> {
        if self.removing_connection == Some(connection_id) {
            widget::button::standard(fl!("removing"))
                .on_press_maybe(None)
                .into()
        } else if self.pending_remove == Some(connection_id) {
            widget::button::destructive(fl!("confirm-remove"))
                .on_press(Message::RemoveConnection(connection_id))
                .into()
        } else {
            widget::button::destructive(fl!("remove"))
                .on_press(Message::RemoveConnection(connection_id))
                .into()
        }
    }

    fn view_onedrive_setup_actions(&self) -> Option<Element<'static, Message>> {
        if self.draft.provider != Provider::OneDrive {
            return None;
        }
        if draft_is_onedrive_rclone(&self.draft) {
            return Some(self.view_onedrive_rclone_actions());
        }

        Some(match self.draft.access_mode {
            AccessMode::OnlineMount => field_with_help(
                widget::button::suggested(fl!("onedrive-setup-online"))
                    .on_press(Message::StartOnedriverSetup),
                onedrive_setup_guidance(self.draft.access_mode),
            ),
            AccessMode::OfflineMirror => choice_row(vec![
                field_with_help(
                    widget::button::suggested(fl!("onedrive-setup-offline"))
                        .on_press(Message::StartOneDriveMirrorSetup),
                    onedrive_setup_guidance(self.draft.access_mode),
                ),
                field_with_help(
                    widget::button::standard(fl!("onedrive-auth-manual"))
                        .on_press(Message::StartOneDriveMirrorManualSetup),
                    fl!("onedrive-auth-manual-help"),
                ),
            ]),
        })
    }

    fn view_onedrive_rclone_actions(&self) -> Element<'static, Message> {
        let ready = self.onedrive_editor_pending.is_none();
        let mut row = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_help(
                action_button(
                    fl!("onedrive-editor-create"),
                    ready,
                    Message::SetupOneDriveRclone(true),
                ),
                fl!("onedrive-editor-create-help"),
            ))
            .push(field_with_help(
                action_button(
                    fl!("onedrive-editor-select"),
                    ready,
                    Message::SetupOneDriveRclone(false),
                ),
                fl!("onedrive-editor-select-help"),
            ));
        if matches!(self.window_mode, WindowMode::ModifyConnection(_)) {
            row = row.push(
                widget::button::standard(fl!("rclone-detect"))
                    .on_press(Message::DetectRcloneRemotes),
            );
        }
        if !ready {
            row = row.push(
                widget::button::standard(fl!("onedrive-editor-cancel"))
                    .on_press(Message::CancelOneDriveRclone),
            );
        }
        row.into()
    }

    fn view_onedrive_rclone_fields(&self) -> Element<'_, Message> {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        widget::Column::new()
            .spacing(8)
            .push(choice_row(vec![
                select_button(
                    fl!("onedrive-editor-personal"),
                    self.draft.onedrive_drive_type == DriveType::Personal,
                    Message::DraftOneDriveType(DriveType::Personal),
                ),
                select_button(
                    fl!("onedrive-editor-business"),
                    self.draft.onedrive_drive_type == DriveType::Business,
                    Message::DraftOneDriveType(DriveType::Business),
                ),
            ]))
            .push(field_with_help(
                widget::text_input::text_input(
                    fl!("onedrive-editor-account"),
                    &self.draft.onedrive_expected_account,
                )
                .on_input(Message::DraftOneDriveAccount),
                fl!("onedrive-editor-account-help"),
            ))
            .push(widget::text::body(
                if self.draft.onedrive_identity.is_some() {
                    fl!("onedrive-editor-verified")
                } else {
                    fl!("onedrive-editor-verification-required")
                },
            ))
            .into()
    }

    fn view_local_target_picker(&self) -> Element<'_, Message> {
        widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_safety_help(
                widget::text_input::text_input(fl!("local-target-example"), &self.draft.local_path)
                    .on_input(Message::DraftLocalPath),
                fl!("local-target-safety"),
                fl!("local-target-help"),
            ))
            .push(field_with_help_at(
                widget::button::standard(fl!("browse")).on_press(Message::OpenLocalFolderPicker),
                TooltipHelp::Plain(Cow::Owned(fl!("browse-local-help"))),
                widget::tooltip::Position::Bottom,
            ))
            .into()
    }

    fn view_create_rclone_remote_action(&self) -> Element<'static, Message> {
        match self.draft.provider {
            Provider::GoogleDrive => field_with_help(
                widget::button::suggested(fl!("google-drive-create"))
                    .on_press(Message::ApplyGoogleDriveRcloneRemote),
                fl!("google-drive-create-help"),
            ),
            Provider::Box => field_with_help(
                widget::button::suggested(fl!("box-create"))
                    .on_press(Message::CreateBoxRcloneRemote),
                fl!("box-create-help"),
            ),
            Provider::Smb => field_with_help(
                widget::button::suggested(fl!("smb-create-update"))
                    .on_press(Message::CreateSmbRcloneRemote),
                fl!("smb-create-update-help"),
            ),
            Provider::Sftp => self.view_sftp_action(true),
            Provider::Teams => self.view_sharepoint_setup_action(),
            Provider::OneDrive => widget::Space::new().width(Length::Shrink).into(),
        }
    }

    fn view_sharepoint_setup_action(&self) -> Element<'static, Message> {
        field_with_help(
            widget::button::suggested(fl!("teams-oauth-connect")).on_press_maybe(
                (!self.sharepoint_setup_pending).then_some(Message::CreateSharePointRcloneRemote),
            ),
            fl!("teams-oauth-connect-help"),
        )
    }

    fn draft_uses_rclone(&self) -> bool {
        draft_is_onedrive_rclone(&self.draft)
            || matches!(
                self.draft.provider,
                Provider::Teams
                    | Provider::GoogleDrive
                    | Provider::Box
                    | Provider::Smb
                    | Provider::Sftp
            )
    }

    fn draft_primary_actions_ready(&self) -> bool {
        if self.sftp_setup_pending
            || self.sharepoint_setup_pending
            || self.onedrive_editor_pending.is_some()
        {
            return false;
        }
        if self.draft.provider == Provider::OneDrive {
            return self.draft_onedrive_setup_ready();
        }
        if self.window_mode != WindowMode::AddConnection || !self.draft_uses_rclone() {
            return true;
        }
        self.matching_rclone_remotes(self.draft.provider)
            .iter()
            .any(|remote| remote.name == self.draft.remote_reference)
    }

    fn draft_onedrive_setup_ready(&self) -> bool {
        if draft_is_onedrive_rclone(&self.draft) {
            return self
                .draft
                .onedrive_identity
                .as_ref()
                .is_some_and(|identity| {
                    identity.is_valid() && identity.drive_type == self.draft.onedrive_drive_type
                });
        }
        let Ok(connection) = connection_from_draft(&self.draft) else {
            return false;
        };
        match connection.mode {
            ConnectionMode::OnlineMount(_) => {
                let Ok(plan) = onedriver_mount_plan(
                    &connection,
                    &default_cache_root(),
                    &default_config_root(),
                ) else {
                    return false;
                };
                match onedriver_auth_state_for_plan(&plan) {
                    OnedriverAuthState::Unauthenticated => false,
                    OnedriverAuthState::Authenticated { config_file } => config_file
                        .metadata()
                        .map(|metadata| metadata.len() > 0)
                        .unwrap_or(false),
                }
            }
            ConnectionMode::OfflineMirror(_) => {
                let Ok(plan) = one_drive_mirror_plan(
                    &connection,
                    &default_config_root(),
                    &OneDriveIsolationReport {
                        active_onedriver_paths: Vec::new(),
                    },
                ) else {
                    return false;
                };
                plan.config_directory
                    .join("refresh_token")
                    .metadata()
                    .map(|metadata| metadata.len() > 0)
                    .unwrap_or(false)
            }
        }
    }

    fn view_remote_account_fields(&self) -> Element<'_, Message> {
        match self.draft.provider {
            Provider::OneDrive if draft_is_onedrive_rclone(&self.draft) => {
                self.view_rclone_remote_fields()
            }
            Provider::OneDrive => self.view_onedrive_account_fields(),
            Provider::Teams
            | Provider::GoogleDrive
            | Provider::Box
            | Provider::Smb
            | Provider::Sftp => self.view_rclone_remote_fields(),
        }
    }

    fn view_onedrive_account_fields(&self) -> Element<'_, Message> {
        let mut column = widget::Column::new()
            .spacing(8)
            .push(field_with_safety_help(
                widget::text_input::text_input(
                    onedrive_account_placeholder(self.draft.access_mode),
                    &self.draft.remote_reference,
                )
                .on_input(Message::DraftRemote),
                onedrive_account_safety_warning(self.draft.access_mode),
                onedrive_account_help(self.draft.access_mode),
            ))
            .push(field_with_help(
                widget::text_input::text_input(
                    onedrive_folder_placeholder(),
                    &self.draft.remote_subpath,
                )
                .on_input(Message::DraftSubpath),
                fl!("onedrive-folder-help"),
            ));
        if self.draft.access_mode == AccessMode::OfflineMirror
            && let Some(connection_id) = self.draft.id
        {
            column = column.push(self.view_onedrive_mirror_auth_handoff(connection_id));
        }
        column.into()
    }

    fn view_onedrive_mirror_auth_handoff(
        &self,
        _connection_id: ConnectionId,
    ) -> Element<'_, Message> {
        if self.onedrive_auth_open_command.is_empty() {
            return widget::text::body(fl!("onedrive-auth-prompt")).into();
        }
        widget::container(
            widget::Column::new()
                .spacing(8)
                .push(widget::text::body(fl!("onedrive-auth-title")))
                .push(widget::text::body(fl!("onedrive-auth-intro")))
                .push(field_with_help(
                    widget::button::suggested(fl!("onedrive-auth-open-browser"))
                        .on_press(Message::OpenOneDriveMirrorAuthUrl),
                    fl!("onedrive-auth-open-browser-help"),
                ))
                .push(widget::text::body(fl!("onedrive-auth-shell-fallback")))
                .push(field_with_help(
                    widget::text_input::text_input(
                        onedrive_auth_shell_placeholder(),
                        &self.onedrive_auth_open_command,
                    )
                    .on_input(|_| Message::DraftOneDriveAuthResponse(String::new())),
                    fl!("onedrive-auth-shell-help"),
                ))
                .push(widget::text::body(fl!("onedrive-auth-redirect-intro")))
                .push(field_with_help(
                    widget::text_input::text_input(
                        onedrive_auth_redirect_placeholder(),
                        &self.onedrive_auth_response_url,
                    )
                    .on_input(Message::DraftOneDriveAuthResponse),
                    fl!("onedrive-auth-redirect-help"),
                ))
                .push(field_with_help(
                    widget::button::suggested(fl!("onedrive-auth-submit"))
                        .on_press(Message::SubmitOneDriveMirrorAuthResponse),
                    fl!("onedrive-auth-submit-help"),
                )),
        )
        .padding(12)
        .width(Length::Fill)
        .class(cosmic::style::Container::Secondary)
        .into()
    }

    fn view_rclone_remote_fields(&self) -> Element<'_, Message> {
        let provider = self.draft.provider;
        let mut column = widget::Column::new().spacing(8).push(field_with_help(
            widget::text_input::text_input(
                rclone_remote_placeholder(provider),
                &self.draft.remote_reference,
            )
            .on_input(Message::DraftRemote),
            if draft_is_onedrive_rclone(&self.draft) {
                fl!("onedrive-editor-remote-help")
            } else {
                rclone_remote_help(provider, self.window_mode == WindowMode::AddConnection)
            },
        ));

        let matching = self.matching_rclone_remotes(provider);
        if !matching.is_empty() {
            column = column.push(widget::text::body(fl!(
                "rclone-detected-remotes",
                provider = provider_editor_label(provider)
            )));
            column = column.push(self.view_rclone_remote_choices(matching.clone()));
            if self.window_mode == WindowMode::AddConnection {
                column = column.push(self.view_rclone_remote_management(matching));
            }
        }

        if draft_is_onedrive_rclone(&self.draft) {
            column = column.push(self.view_onedrive_rclone_fields());
        }
        if provider == Provider::Smb {
            column = column.push(self.view_smb_remote_setup_fields());
        } else if provider == Provider::Sftp {
            column = column.push(self.view_sftp_fields());
        } else if provider == Provider::GoogleDrive {
            column = column.push(self.view_google_drive_remote_setup_fields());
        } else if provider == Provider::Teams {
            column = column.push(field_with_help(
                widget::text_input::text_input(
                    fl!("sharepoint-library-url-example"),
                    &self.draft.sharepoint_library_url,
                )
                .on_input(Message::DraftSharePointLibraryUrl),
                fl!("sharepoint-library-url-help"),
            ));
        }

        column
            .push(field_with_help(
                widget::text_input::text_input(
                    if provider == Provider::Sftp {
                        sftp_remote_directory_placeholder().to_owned()
                    } else if provider == Provider::Teams {
                        fl!("sharepoint-library-folder-placeholder")
                    } else {
                        rclone_remote_directory_placeholder().to_owned()
                    },
                    &self.draft.remote_subpath,
                )
                .on_input(Message::DraftSubpath),
                if provider == Provider::Sftp {
                    fl!("sftp-remote-directory-help")
                } else {
                    fl!("rclone-remote-directory-help")
                },
            ))
            .into()
    }

    fn view_google_drive_remote_setup_fields(&self) -> Element<'_, Message> {
        widget::Column::new()
            .spacing(8)
            .push(field_with_help(
                widget::text_input::text_input(
                    google_drive_client_id_placeholder(),
                    &self.draft.google_client_id,
                )
                .on_input(Message::DraftGoogleClientId),
                fl!("google-drive-client-id-help"),
            ))
            .push(field_with_safety_help(
                widget::text_input::text_input(
                    google_drive_client_secret_placeholder(),
                    &self.draft.google_client_secret,
                )
                .password()
                .on_input(Message::DraftGoogleClientSecret),
                fl!("google-drive-client-secret-warning"),
                fl!("google-drive-client-secret-help"),
            ))
            .into()
    }

    fn view_smb_remote_setup_fields(&self) -> Element<'_, Message> {
        widget::Column::new()
            .spacing(8)
            .push(field_with_help(
                widget::text_input::text_input(smb_host_placeholder(), &self.draft.smb_host)
                    .on_input(Message::DraftSmbHost),
                fl!("smb-host-help"),
            ))
            .push(field_with_help(
                widget::text_input::text_input(smb_username_placeholder(), &self.draft.smb_user)
                    .on_input(Message::DraftSmbUser),
                fl!("smb-username-help"),
            ))
            .push(field_with_help(
                widget::text_input::text_input(smb_domain_placeholder(), &self.draft.smb_domain)
                    .on_input(Message::DraftSmbDomain),
                fl!("smb-domain-help"),
            ))
            .push(field_with_safety_help(
                widget::text_input::text_input(
                    smb_password_placeholder(),
                    &self.draft.smb_password,
                )
                .password()
                .on_input(Message::DraftSmbPassword),
                fl!("smb-password-warning"),
                fl!("smb-password-help"),
            ))
            .into()
    }

    fn matching_rclone_remotes(&self, provider: Provider) -> Vec<RcloneDraftRemote> {
        self.matching_rclone_remotes_in(&self.rclone_remotes, provider)
    }

    fn view_rclone_remote_choices(
        &self,
        remotes: Vec<RcloneDraftRemote>,
    ) -> Element<'static, Message> {
        let mut column = widget::Column::new().spacing(8);
        for row_remotes in remotes.chunks(rclone_remote_buttons_per_row()) {
            let mut row = widget::Row::new().spacing(8).align_y(Alignment::Center);
            for remote in row_remotes {
                let selected = self.draft.remote_reference == remote.name;
                let label = remote.name.clone();
                let help = fl!(
                    "rclone-use-remote-help",
                    name = remote.name.clone(),
                    backend = remote.backend.clone()
                );
                row = row.push(field_with_help(
                    select_button(label, selected, Message::DraftRemote(remote.name.clone())),
                    help,
                ));
            }
            column = column.push(row);
        }
        column.into()
    }

    fn view_rclone_remote_management(
        &self,
        remotes: Vec<RcloneDraftRemote>,
    ) -> Element<'static, Message> {
        let mut column = widget::Column::new()
            .spacing(6)
            .push(widget::text::body(fl!("rclone-manage-remotes")));
        for remote in remotes {
            let in_use = self.rclone_remote_is_referenced(&remote.name);
            let confirm =
                self.pending_rclone_remote_remove.as_deref() == Some(remote.name.as_str());
            let removing = self.removing_rclone_remote.as_deref() == Some(remote.name.as_str());
            let button: Element<'_, Message> = if in_use {
                widget::button::standard(fl!("rclone-remote-in-use"))
                    .on_press_maybe(None)
                    .into()
            } else if removing {
                widget::button::standard(fl!("rclone-remote-removing"))
                    .on_press_maybe(None)
                    .into()
            } else if confirm {
                widget::button::destructive(fl!("rclone-remote-confirm-remove"))
                    .on_press(Message::RequestRemoveRcloneRemote(remote.name.clone()))
                    .into()
            } else {
                widget::button::standard(fl!("rclone-remote-remove"))
                    .on_press(Message::RequestRemoveRcloneRemote(remote.name.clone()))
                    .into()
            };
            let help = if in_use {
                fl!("rclone-remote-in-use-help", name = remote.name.clone())
            } else {
                fl!("rclone-remote-remove-help", name = remote.name.clone())
            };
            column = column.push(
                widget::Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(widget::text::body(format!(
                        "{} ({})",
                        remote.name, remote.backend
                    )))
                    .push(field_with_help(button, help)),
            );
        }
        column.into()
    }

    fn view_vpn_choices(&self) -> Element<'static, Message> {
        let mut choices = vec![field_with_help(
            select_button(
                fl!("no-vpn"),
                self.draft.vpn_profile_id.is_none(),
                Message::DraftVpn(None),
            ),
            fl!("no-vpn-help"),
        )];

        choices.extend(vpn_profile_choices(
            VpnKind::NetworkManager,
            &self.config.document.vpn_profiles,
            self.draft.vpn_profile_id,
        ));
        choices.push(field_with_help(
            widget::button::standard(fl!("detect-vpns")).on_press(Message::DetectVpns),
            fl!("detect-vpns-help"),
        ));

        choice_row(choices)
    }

    fn record_operation_request(
        &mut self,
        connection_id: ConnectionId,
        operation: Operation,
    ) -> Task<cosmic::Action<Message>> {
        if !self.unit_status_fresh(connection_id)
            && let Some(connection) = self
                .config
                .document
                .connections
                .iter()
                .find(|connection| connection.id == connection_id)
                .cloned()
        {
            self.unit_operations_waiting
                .insert(connection_id, (connection, operation));
            self.last_notice = Some(fl!("unit-status-wait"));
            return self.refresh_unit_observation(false);
        }
        let state = self.view_state();
        let Some(row) = state.rows.iter().find(|row| row.id == connection_id) else {
            self.last_notice = Some(fl!("notice-connection-unavailable"));
            self.complete_routed_operation(connection_id, operation);
            return Task::none();
        };
        let decision = decide_operation(row, operation);
        let label = row_operation_label(&row.status, operation);
        if !decision.allowed {
            self.last_notice = Some(fl!(
                "notice-action-unavailable",
                operation = label,
                name = row.name.as_str(),
                reason = decision
                    .reason
                    .unwrap_or_else(|| fl!("operation-unavailable-state"))
            ));
            self.complete_routed_operation(connection_id, operation);
            return Task::none();
        }

        let Some(connection) = self
            .config
            .document
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .cloned()
        else {
            self.last_notice = Some(fl!("notice-connection-unavailable"));
            self.complete_routed_operation(connection_id, operation);
            return Task::none();
        };

        if self.repair_operations_pending.contains(&connection_id) {
            self.last_notice = Some(fl!(
                "notice-action-unavailable",
                operation = label,
                name = connection.name.as_str(),
                reason = fl!("operation-unavailable-state")
            ));
            self.complete_routed_operation(connection_id, operation);
            return Task::none();
        }
        if operation == Operation::Repair {
            let now = Instant::now();
            let confirmation =
                repair_confirmation_decision(self.pending_repair, connection_id, now);
            match confirmation {
                RepairConfirmationDecision::Confirm => {}
                RepairConfirmationDecision::Arm | RepairConfirmationDecision::Wait => {
                    if confirmation == RepairConfirmationDecision::Arm {
                        self.pending_repair = Some((connection_id, now));
                    }
                    self.last_notice = Some(match connection.mode {
                        ConnectionMode::OnlineMount(_) => fl!(
                            "notice-repair-confirm",
                            name = connection.name.as_str(),
                            path = connection.local_path.display().to_string()
                        ),
                        ConnectionMode::OfflineMirror(_) => fl!(
                            "notice-offline-repair-confirm",
                            name = connection.name.as_str()
                        ),
                    });
                    self.last_notice_at = Some(now);
                    self.complete_routed_operation(connection_id, operation);
                    return Task::none();
                }
            }
        } else {
            self.pending_repair = None;
        }

        match (&connection.mode, operation) {
            (ConnectionMode::OnlineMount(_), Operation::Mount | Operation::Unmount) => {
                if operation == Operation::Unmount {
                    self.unmount_pending.insert(connection_id);
                    let preload_active = self.directory_preload_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_starting.contains(&connection_id);
                    self.last_notice = Some(if preload_active {
                        fl!("notice-unmounting-preload", name = connection.name.as_str())
                    } else {
                        fl!("notice-unmounting", name = connection.name.as_str())
                    });
                } else {
                    self.last_notice = Some(fl!(
                        "notice-operation-requested",
                        operation = label,
                        name = connection.name.as_str()
                    ));
                }
                self.last_notice_at = Some(Instant::now());
                Task::perform(
                    async move {
                        let result =
                            run_managed_online_mount_operation_result(&connection, operation).await;
                        (connection, result)
                    },
                    move |(connection, result)| {
                        cosmic::Action::App(Message::ManagedOnlineOperationCompleted(
                            connection, operation, result,
                        ))
                    },
                )
            }
            (ConnectionMode::OnlineMount(_), Operation::Repair) => {
                self.pending_repair = None;
                self.repair_operations_pending.insert(connection.id);
                self.last_notice = Some(fl!(
                    "notice-repair-requested",
                    operation = label,
                    name = connection.name.as_str()
                ));
                let connection_id = connection.id;
                Task::perform(
                    async move { run_online_mount_repair_operation(connection).await },
                    move |notice| {
                        cosmic::Action::App(Message::OnlineRepairCompleted(connection_id, notice))
                    },
                )
            }
            (ConnectionMode::OfflineMirror(_), Operation::Repair) => {
                self.pending_repair = None;
                self.repair_operations_pending.insert(connection.id);
                self.last_notice = Some(fl!(
                    "notice-offline-repair-requested",
                    operation = label,
                    name = connection.name.as_str()
                ));
                let connection_id = connection.id;
                let manual_operation = self.mirror_operation_controls.get(&connection_id).cloned();
                Task::perform(
                    async move {
                        run_offline_mirror_repair_operation(connection, manual_operation).await
                    },
                    move |notice| {
                        cosmic::Action::App(Message::OfflineRepairCompleted(connection_id, notice))
                    },
                )
            }
            (
                ConnectionMode::OfflineMirror(_),
                Operation::PreviewInitialSync
                | Operation::SyncNow
                | Operation::PauseSync
                | Operation::ResumeSync,
            ) if is_rclone_offline_mirror(&connection) => {
                self.last_notice = Some(fl!(
                    "notice-operation-requested",
                    operation = label,
                    name = connection.name.as_str()
                ));
                self.mirror_operations_pending.insert(connection_id);
                let control = Arc::new(ManualOperationControl::new());
                self.mirror_operation_controls
                    .insert(connection_id, Arc::clone(&control));
                Task::perform(
                    async move {
                        run_managed_offline_mirror_operation(connection, operation, control).await
                    },
                    move |notice| {
                        cosmic::Action::App(Message::MirrorOperationCompleted(
                            connection_id,
                            notice,
                        ))
                    },
                )
            }
            (
                ConnectionMode::OfflineMirror(_),
                Operation::PreviewInitialSync
                | Operation::SyncNow
                | Operation::PauseSync
                | Operation::ResumeSync,
            ) if is_legacy_onedrive_mirror(&connection) => {
                self.last_notice = Some(fl!(
                    "notice-operation-requested",
                    operation = label,
                    name = connection.name.as_str()
                ));
                self.mirror_operations_pending.insert(connection_id);
                let control = Arc::new(ManualOperationControl::new());
                self.mirror_operation_controls
                    .insert(connection_id, Arc::clone(&control));
                Task::perform(
                    async move {
                        run_managed_onedrive_offline_mirror_operation(
                            connection, operation, control,
                        )
                        .await
                    },
                    move |notice| {
                        cosmic::Action::App(Message::MirrorOperationCompleted(
                            connection_id,
                            notice,
                        ))
                    },
                )
            }
            _ => {
                self.last_notice = Some(fl!(
                    "notice-operation-not-wired",
                    operation = label,
                    name = row.name.as_str()
                ));
                self.complete_routed_operation(connection_id, operation);
                Task::none()
            }
        }
    }

    fn start_refresh_for_active_google_mounts(&mut self) -> Task<cosmic::Action<Message>> {
        let mounts = self.mount_entries.clone();
        if !self.config.document.preload.google_drive.enabled {
            return Task::none();
        }
        let timeout =
            Duration::from_secs(self.config.document.preload.google_drive.maximum_seconds);
        let connections = self
            .config
            .document
            .connections
            .iter()
            .filter(|connection| {
                connection.enabled
                    && connection.provider == Provider::GoogleDrive
                    && matches!(connection.mode, ConnectionMode::OnlineMount(_))
                    && !self.rclone_refresh_starting.contains(&connection.id)
                    && !self.rclone_refresh_jobs.contains_key(&connection.id)
                    && mounts.iter().any(|mount| {
                        mount.target == connection.local_path && mount.filesystem == "fuse.rclone"
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut tasks = Vec::new();
        for connection in connections {
            let Ok(plan) =
                rclone_mount_plan(&connection, &default_runtime_root(), &default_cache_root())
            else {
                continue;
            };
            let name = connection.name.clone();
            let connection_id = connection.id;
            self.rclone_refresh_starting.insert(connection_id);
            tasks.push(Task::perform(
                async move {
                    rclone_refresh::start(
                        connection_id,
                        plan.rc_socket,
                        connection.local_path,
                        timeout,
                    )
                    .await
                },
                move |result| {
                    cosmic::Action::App(Message::RcloneRefreshStarted(
                        name.clone(),
                        connection_id,
                        result,
                    ))
                },
            ));
        }
        Task::batch(tasks)
    }

    fn register_directory_preload(
        &mut self,
        name: String,
        result: Result<DirectoryPreloadJob, String>,
    ) -> Task<cosmic::Action<Message>> {
        match result {
            Ok(job) => {
                let connection_id = job.connection_id;
                let generation = job.generation;
                self.directory_preload_jobs
                    .insert(connection_id, generation);
                if !self.unmount_pending.contains(&connection_id) {
                    self.last_notice = Some(fl!("notice-preload-running", name = name.as_str()));
                    self.last_notice_at = Some(Instant::now());
                }
                Task::perform(
                    async move { directory_preload::monitor(job).await },
                    move |result| {
                        cosmic::Action::App(Message::DirectoryPreloadCompleted(
                            name.clone(),
                            connection_id,
                            generation,
                            result,
                        ))
                    },
                )
            }
            Err(error) => {
                self.last_notice = Some(fl!(
                    "notice-preload-start-failed",
                    name = name.as_str(),
                    error = error
                ));
                self.last_notice_at = Some(Instant::now());
                Task::none()
            }
        }
    }

    fn start_preload_for_active_directory_mounts(&mut self) -> Task<cosmic::Action<Message>> {
        let mounts = self.mount_entries.clone();
        let connections = self
            .config
            .document
            .connections
            .iter()
            .filter(|connection| {
                connection.enabled
                    && matches!(
                        connection.provider,
                        Provider::OneDrive
                            | Provider::Teams
                            | Provider::Box
                            | Provider::Smb
                            | Provider::Sftp
                    )
                    && matches!(connection.mode, ConnectionMode::OnlineMount(_))
                    && self.config.document.preload_policy_for(connection).enabled
                    && !self.directory_preload_jobs.contains_key(&connection.id)
                    && mounts
                        .iter()
                        .any(|mount| mount.target == connection.local_path)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut tasks = Vec::new();
        for connection in connections {
            let name = connection.name.clone();
            let policy = self.config.document.preload_policy_for(&connection);
            let timeout = Duration::from_secs(policy.maximum_seconds);
            let wait_for_root = directory_preload_waits_for_root(&connection);
            let result = directory_preload::start_for_connection(
                &connection,
                timeout,
                policy.maximum_depth,
                wait_for_root,
            );
            tasks.push(self.register_directory_preload(name, result));
        }
        Task::batch(tasks)
    }

    fn load_draft(&mut self, connection_id: ConnectionId) -> Task<cosmic::Action<Message>> {
        let Some(connection) = self
            .config
            .document
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .cloned()
        else {
            self.last_notice = Some(fl!("notice-connection-unavailable"));
            return Task::none();
        };
        self.pending_shared_remote_ack = None;
        if let Some(pending) = self.onedrive_editor_pending.take() {
            pending.cancellation.cancel();
        }
        self.onedrive_editor_generation = self.onedrive_editor_generation.wrapping_add(1);
        self.draft = draft_from_connection(&connection);
        self.last_notice = Some(fl!("notice-modify", name = connection.name.as_str()));
        if matches!(connection.provider, Provider::Smb | Provider::Sftp) {
            return self.remote_prefill();
        }
        Task::none()
    }

    fn prefill_context(&self) -> RemotePrefillContext {
        RemotePrefillContext {
            id: self.draft.id,
            provider: self.draft.provider,
            remote: self.draft.remote_reference.clone(),
            sftp_revision: self.sftp_settings_revision,
            smb: SmbRemoteDetails {
                host: self.draft.smb_host.clone(),
                user: self.draft.smb_user.clone(),
                domain: self.draft.smb_domain.clone(),
            },
        }
    }

    fn remote_prefill(&mut self) -> Task<cosmic::Action<Message>> {
        self.prefill_cancellation.cancel();
        self.prefill_cancellation = CancellationToken::new();
        let cancellation = self.prefill_cancellation.clone();
        self.prefill_generation = self.prefill_generation.wrapping_add(1);
        if self.draft.provider == Provider::Sftp {
            self.reset_sftp_settings_tracking();
            self.draft.sftp = SftpDraft::default();
        }
        let context = self.prefill_context();
        let remote = context.remote.clone();
        let provider = context.provider;
        let generation = self.prefill_generation;
        Task::perform(
            async move {
                let text = HostInspection::RcloneConfigDump
                    .run_current(cancellation)
                    .await?;
                match provider {
                    Provider::Sftp => Ok(RemotePrefillResult {
                        smb: None,
                        sftp: sftp::prefill_from_dump(&text, &remote)?,
                    }),
                    Provider::Smb => Ok(RemotePrefillResult {
                        smb: parse_smb_remote_details(&text, &remote)?,
                        sftp: None,
                    }),
                    _ => unreachable!("prefill is only requested for SMB/SFTP"),
                }
            },
            move |result| {
                cosmic::Action::App(Message::RemotePrefillCompleted(
                    generation,
                    context.clone(),
                    result,
                ))
            },
        )
    }

    fn finish_remote_prefill(&mut self, result: Result<RemotePrefillResult, String>) {
        if self.draft.provider == Provider::Sftp {
            self.finish_sftp_details(result.map(|prefill| prefill.sftp));
            return;
        }
        let result = result.map(|prefill| prefill.smb);
        let name = self.draft.name.as_str();
        self.last_notice = Some(match result {
            Ok(Some(details)) => {
                self.draft.smb_host = details.host;
                self.draft.smb_user = details.user;
                self.draft.smb_domain = details.domain;
                fl!("notice-modify-smb-loaded", name = name)
            }
            Ok(None) => fl!(
                "notice-modify-smb-missing",
                name = name,
                remote = self.draft.remote_reference.as_str()
            ),
            Err(error) => fl!("notice-modify-smb-failed", name = name, error = error),
        });
    }

    fn refresh_mount_inspection(&mut self) -> Task<cosmic::Action<Message>> {
        if self.mount_inspection_pending {
            return Task::none();
        }
        self.mount_inspection_pending = true;
        Task::perform(host_mount_entries(), |result| {
            cosmic::Action::App(Message::MountInspectionCompleted(result))
        })
    }

    fn save_draft(&mut self) -> Task<cosmic::Action<Message>> {
        if draft_is_onedrive_rclone(&self.draft) && !self.draft_primary_actions_ready() {
            self.last_notice = Some(fl!("onedrive-editor-verification-required"));
            return Task::none();
        }
        if self.draft.provider == Provider::Teams && self.draft.id.is_none() {
            self.draft.id = Some(ConnectionId::new());
        }
        let connection = match connection_from_draft(&self.draft) {
            Ok(connection) => connection,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        if let Err(error) = managed_plan_summary(&connection) {
            self.last_notice = Some(fl!("notice-plan-validation-failed", error = error));
            return Task::none();
        }
        if let Err(error) = self.validate_connection_edit(&connection) {
            self.last_notice = Some(error);
            return Task::none();
        }
        if let Some(warning) = self.shared_remote_warning(&connection)
            && self.pending_shared_remote_ack != Some(connection.id)
        {
            self.pending_shared_remote_ack = Some(connection.id);
            self.last_notice = Some(fl!("notice-shared-remote-confirm", warning = warning));
            return Task::none();
        }
        if connection.provider == Provider::OneDrive {
            if let Some(validated) = &self.validated_draft
                && validated.connection == connection
            {
                self.last_notice = Some(fl!(
                    "notice-validation-reused",
                    name = connection.name.as_str()
                ));
                return self.save_validated_draft(connection, Some(validated.summary.clone()));
            }
            self.last_notice = Some(fl!(
                "notice-onedrive-validating",
                name = connection.name.as_str()
            ));
            return Task::perform(
                async move {
                    let validation = validate_onedrive_connection_for_save(&connection).await;
                    (connection, validation)
                },
                |(connection, validation)| {
                    cosmic::Action::App(Message::SaveDraftValidated(connection, validation))
                },
            );
        }
        if connection.provider == Provider::Teams {
            self.last_notice = Some(fl!(
                "notice-sharepoint-verifying",
                name = connection.name.as_str()
            ));
            return Task::perform(
                async move {
                    let mut connection = connection;
                    let validation = match sharepoint::verify_remote(
                        &app_command_runner(),
                        &connection.remote_reference,
                        connection
                            .teams_identity
                            .as_ref()
                            .expect("SharePoint draft has identity"),
                    )
                    .await
                    {
                        Ok(verified) => {
                            let account = verified.account.clone().unwrap_or_else(|| {
                                crate::fl!("ui-app-account-unavailable-from-microsoft-graph")
                            });
                            let identity = verified.identity.clone();
                            let summary = crate::fl!(
                                "ui-app-verified-account-account-site-value2-library-value3",
                                account = (account).to_string(),
                                value2 = (identity.site_url).to_string(),
                                value3 = (identity.library_url).to_string(),
                                value4 = (identity.drive_id).to_string()
                            );
                            connection.teams_identity = Some(identity);
                            verify_rclone_access_with_verified(&connection, Some(&verified))
                                .await
                                .map(|access| format!("{summary} {access}"))
                        }
                        Err(error) => Err(error),
                    };
                    (connection, validation)
                },
                |(connection, validation)| {
                    cosmic::Action::App(Message::SaveDraftValidated(connection, validation))
                },
            );
        }
        self.save_validated_draft(connection, None)
    }

    fn save_validated_draft(
        &mut self,
        connection: Connection,
        validation_summary: Option<String>,
    ) -> Task<cosmic::Action<Message>> {
        let storage = match AppConfigStorage::runtime() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        let id = connection.id;
        let name = connection.name.clone();
        let should_install_rclone_online_mount = is_rclone_online_mount(&connection);
        let should_install_rclone_offline_mirror = is_rclone_offline_mirror(&connection);
        let should_install_onedriver_online_mount = is_onedriver_online_mount(&connection);
        let should_install_onedrive_offline_mirror = is_legacy_onedrive_mirror(&connection);
        let saved_connection = connection.clone();
        match load_runtime_config() {
            Ok(config) => self.config = config,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        }
        if let Some(existing) = self
            .config
            .document
            .connections
            .iter()
            .find(|existing| existing.id == id)
            && let Err(error) = invalidate_sharepoint_mirror_confirmation_on_retarget(
                existing,
                &connection,
                &default_work_root(),
            )
        {
            self.last_notice = Some(error);
            return Task::none();
        }
        let result = self.config.update_validated_with(&storage, |document| {
            if let Some(existing) = document
                .connections
                .iter_mut()
                .find(|existing| existing.id == id)
            {
                *existing = connection;
            } else {
                document.connections.push(connection);
            }
        });
        match result {
            Ok(true) | Ok(false) if should_install_rclone_online_mount => {
                self.last_notice = Some(fl!("notice-install-mount", name = name.as_str()));
                Task::perform(
                    async move { install_rclone_online_mount_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_rclone_offline_mirror => {
                self.last_notice = Some(fl!("notice-install-mirror", name = name.as_str()));
                Task::perform(
                    async move { install_rclone_offline_mirror_units(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_onedriver_online_mount => {
                self.last_notice = Some(fl!(
                    "notice-install-onedriver",
                    name = save_notice_name(&name, validation_summary.as_deref())
                ));
                Task::perform(
                    async move { install_onedriver_online_mount_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_onedrive_offline_mirror => {
                self.last_notice = Some(fl!(
                    "notice-install-onedrive",
                    name = save_notice_name(&name, validation_summary.as_deref())
                ));
                Task::perform(
                    async move { install_onedrive_offline_mirror_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) => {
                self.last_notice = Some(fl!("notice-saved", name = name.as_str()));
                notify_runtime_task()
            }
            Ok(false) => {
                self.last_notice = Some(fl!("notice-unchanged", name = name.as_str()));
                notify_runtime_task()
            }
            Err(error) => {
                self.last_notice = Some(fl!(
                    "notice-save-failed",
                    name = name.as_str(),
                    error = error.to_string()
                ));
                Task::none()
            }
        }
    }

    fn validate_connection_edit(&self, connection: &Connection) -> Result<(), String> {
        if let Some(saved) = self
            .config
            .document
            .connections
            .iter()
            .find(|saved| saved.id == connection.id)
        {
            if saved.provider != connection.provider {
                return Err(crate::fl!(
                    "ui-app-provider-changes-are-disabled-for-existing-connections"
                ));
            }
            if access_mode_for_connection(saved) != access_mode_for_connection(connection) {
                return Err(crate::fl!(
                    "ui-app-access-mode-changes-are-disabled-for-existing"
                ));
            }
        }

        if let Some(other) = self.config.document.connections.iter().find(|saved| {
            saved.id != connection.id
                && saved
                    .name
                    .trim()
                    .eq_ignore_ascii_case(connection.name.trim())
        }) {
            return Err(crate::fl!(
                "ui-app-connection-name-value1-is-already-used-by",
                value1 = (connection.name).to_string(),
                value2 = (other.name).to_string()
            ));
        }

        if let Some(other) = self.config.document.connections.iter().find(|saved| {
            saved.id != connection.id
                && (paths_overlap(&saved.local_path, &connection.local_path)
                    || paths_overlap(&connection.local_path, &saved.local_path))
        }) {
            return Err(crate::fl!(
                "ui-app-local-target-value1-overlaps-value2-used-by",
                value1 = (connection.local_path.display()).to_string(),
                value2 = (other.local_path.display()).to_string(),
                value3 = (other.name).to_string()
            ));
        }

        let mut candidate = self.config.clone();
        candidate
            .document
            .connections
            .retain(|saved| saved.id != connection.id);
        candidate.document.connections.push(connection.clone());
        candidate
            .validate_with_work_root(&default_work_root())
            .map_err(|errors| {
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }

    fn shared_remote_warning(&self, connection: &Connection) -> Option<String> {
        self.config
            .document
            .connections
            .iter()
            .find(|saved| {
                saved.id != connection.id
                    && saved.provider == connection.provider
                    && saved
                        .remote_reference
                        .trim()
                        .eq_ignore_ascii_case(connection.remote_reference.trim())
            })
            .map(|other| {
                crate::fl!(
                    "ui-app-value1-uses-the-same-value2-account-remote",
                    value1 = (connection.name).to_string(),
                    value2 = (provider_label(connection.provider)).to_string(),
                    value3 = (connection.remote_reference).to_string(),
                    value4 = (other.name).to_string()
                )
            })
    }

    fn detect_and_import_vpns(&mut self) {
        let detection = detect_vpn_profiles();
        let storage = match AppConfigStorage::runtime() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return;
            }
        };

        let mut imported = 0usize;
        let mut updated = 0usize;
        let mut selected = None;
        match load_runtime_config() {
            Ok(config) => self.config = config,
            Err(error) => {
                self.last_notice = Some(error);
                return;
            }
        }
        let result = self.config.update_validated_with(&storage, |document| {
            for detected in &detection.profiles {
                if let Some(existing) = document
                    .vpn_profiles
                    .iter_mut()
                    .find(|profile| same_vpn_reference(profile, detected))
                {
                    update_detected_vpn_profile(existing, detected);
                    selected.get_or_insert(existing.id);
                    updated += 1;
                } else {
                    let id = detected.id;
                    document.vpn_profiles.push(detected.clone());
                    selected.get_or_insert(id);
                    imported += 1;
                }
            }
        });

        match result {
            Ok(_) => {
                if self.draft.vpn_profile_id.is_none() {
                    self.draft.vpn_profile_id = selected;
                }
                let warning = if detection.warnings.is_empty() {
                    String::new()
                } else {
                    format!(" {}", detection.warnings.join(" "))
                };
                let notice = crate::fl!(
                    "ui-app-vpn-detection-complete-imported-imported-updated-already",
                    imported = (imported).to_string(),
                    updated = (updated).to_string(),
                    warning = (warning).to_string()
                );
                self.last_notice = Some(notice);
            }
            Err(error) => {
                let notice = crate::fl!(
                    "ui-app-vpn-detection-could-not-be-saved-error",
                    error = (error).to_string()
                );
                self.last_notice = Some(notice);
            }
        }
    }

    fn detect_rclone_remotes(&mut self, show_notice: bool) -> Task<cosmic::Action<Message>> {
        self.rclone_inspection_cancellation.cancel();
        self.rclone_inspection_cancellation = CancellationToken::new();
        let cancellation = self.rclone_inspection_cancellation.clone();
        self.rclone_inspection_generation = self.rclone_inspection_generation.wrapping_add(1);
        let generation = self.rclone_inspection_generation;
        Task::perform(
            async move {
                let text = HostInspection::RcloneConfigDump
                    .run_current(cancellation)
                    .await?;
                parse_rclone_remotes_for_app(&text)
            },
            move |result| {
                cosmic::Action::App(Message::RcloneInspectionCompleted(
                    generation,
                    show_notice,
                    result,
                ))
            },
        )
    }

    fn finish_rclone_inspection(
        &mut self,
        show_notice: bool,
        result: Result<Vec<RcloneDraftRemote>, String>,
    ) {
        match result {
            Ok(remotes) => {
                let total = remotes.len();
                let matching = self
                    .matching_rclone_remotes_in(&remotes, self.draft.provider)
                    .len();
                self.rclone_remotes = remotes;
                if show_notice {
                    self.last_notice = Some(fl!(
                        "notice-rclone-detected",
                        total = total,
                        matching = matching,
                        provider = provider_label(self.draft.provider)
                    ));
                }
            }
            Err(error) => self.last_notice = Some(fl!("notice-rclone-run-failed", error = error)),
        }
    }

    fn rclone_remote_is_referenced(&self, remote_name: &str) -> bool {
        self.config.document.connections.iter().any(|connection| {
            (connection.provider != Provider::OneDrive || is_onedrive_rclone_mirror(connection))
                && connection
                    .remote_reference
                    .eq_ignore_ascii_case(remote_name)
        })
    }

    fn request_remove_rclone_remote(
        &mut self,
        remote_name: String,
    ) -> Task<cosmic::Action<Message>> {
        if self.rclone_remote_is_referenced(&remote_name) {
            self.pending_rclone_remote_remove = None;
            self.last_notice = Some(fl!("notice-rclone-in-use", name = remote_name.as_str()));
            return Task::none();
        }
        if self.pending_rclone_remote_remove.as_deref() != Some(remote_name.as_str()) {
            self.pending_rclone_remote_remove = Some(remote_name.clone());
            self.last_notice = Some(fl!(
                "notice-rclone-confirm-remove",
                name = remote_name.as_str()
            ));
            return Task::none();
        }

        self.last_notice = Some(fl!("notice-rclone-removing", name = remote_name.as_str()));
        self.removing_rclone_remote = Some(remote_name.clone());
        Task::perform(
            async move { remove_rclone_remote_result(remote_name).await },
            |result| cosmic::Action::App(Message::RcloneRemoteRemoved(result)),
        )
    }

    fn create_smb_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Smb {
            self.last_notice = Some(fl!("notice-smb-only"));
            return Task::none();
        }
        let setup = match SmbRemoteSetup::from_draft(&self.draft) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(fl!("notice-smb-creating", name = setup.name.as_str()));
        Task::perform(
            async move { create_smb_rclone_remote_result(setup).await },
            |result| cosmic::Action::App(Message::SmbRcloneRemoteCreated(result)),
        )
    }

    fn create_box_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Box {
            self.last_notice = Some(fl!("notice-box-only"));
            return Task::none();
        }
        let setup = match BoxRemoteSetup::from_draft(&self.draft) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(fl!("notice-box-starting", name = setup.name.as_str()));
        Task::perform(
            async move { create_box_rclone_remote_result(setup).await },
            |result| cosmic::Action::App(Message::BoxRcloneRemoteCreated(result)),
        )
    }

    fn setup_onedrive_rclone(&mut self, create: bool) -> Task<cosmic::Action<Message>> {
        if self.onedrive_editor_pending.is_some() {
            return Task::none();
        }
        if self.draft.id.is_none() {
            self.draft.id = Some(ConnectionId::new());
        }
        let request = match OneDriveEditorRequest::from_draft(
            &self.draft,
            &self.config.document.connections,
            create,
        ) {
            Ok(request) => request,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.validated_draft = None;
        self.draft.onedrive_identity = None;
        self.onedrive_editor_generation = self.onedrive_editor_generation.wrapping_add(1);
        let generation = self.onedrive_editor_generation;
        let cancellation = CancellationToken::new();
        self.onedrive_editor_pending = Some(OneDriveEditorPending {
            cancellation: cancellation.clone(),
        });
        self.last_notice = Some(fl!("onedrive-editor-working"));
        Task::perform(
            async move {
                let result = match cosmic_ext_applet_mounter::onedrive::MicrosoftGraph::new() {
                    Ok(graph) => {
                        run_onedrive_editor_setup(
                            &app_command_runner(),
                            &graph,
                            &request,
                            cancellation,
                        )
                        .await
                    }
                    Err(error) => Err(error.to_string()),
                };
                (request, result)
            },
            move |(request, result)| {
                cosmic::Action::App(Message::OneDriveRcloneCompleted(
                    generation, request, result,
                ))
            },
        )
    }

    fn create_sharepoint_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Teams {
            self.last_notice = Some(fl!("teams-oauth-teams-only"));
            return Task::none();
        }
        if self.sharepoint_setup_pending {
            return Task::none();
        }
        let setup = match SharePointSetup::new(
            &self.draft.remote_reference,
            &self.draft.sharepoint_library_url,
        ) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.sharepoint_setup_pending = true;
        self.last_notice = Some(fl!("teams-oauth-opening", name = setup.remote_name.clone()));
        Task::perform(
            async move {
                sharepoint_setup::create_sharepoint_remote(&app_command_runner(), setup).await
            },
            |result| cosmic::Action::App(Message::SharePointRcloneRemoteCreated(result)),
        )
    }

    fn apply_google_drive_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::GoogleDrive {
            self.last_notice = Some(fl!("notice-google-only"));
            return Task::none();
        }
        let setup = match GoogleDriveRemoteSetup::from_draft(&self.draft) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        let action = match self.window_mode {
            WindowMode::ModifyConnection(_) => GoogleDriveRemoteAction::Update,
            _ => GoogleDriveRemoteAction::Create,
        };
        self.last_notice = Some(fl!("notice-google-starting", name = setup.name.as_str()));
        Task::perform(
            async move { apply_google_drive_rclone_remote_result(setup, action).await },
            |result| cosmic::Action::App(Message::GoogleDriveRcloneRemoteApplied(result)),
        )
    }

    fn start_onedriver_setup(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::OneDrive
            || self.draft.access_mode != AccessMode::OnlineMount
        {
            self.last_notice = Some(fl!("notice-onedriver-only"));
            return Task::none();
        }
        if self.draft.id.is_none() {
            self.draft.id = Some(ConnectionId::new());
        }
        let connection = match connection_from_draft(&self.draft) {
            Ok(connection) => connection,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(fl!(
            "notice-onedriver-starting",
            name = connection.name.as_str()
        ));
        Task::perform(
            async move { run_onedriver_online_setup_result(connection).await },
            |result| cosmic::Action::App(Message::OnedriverSetupCompleted(result)),
        )
    }

    fn open_local_folder_picker(&mut self) -> Task<cosmic::Action<Message>> {
        let mode = self.draft.access_mode;
        self.last_notice = Some(fl!(
            "notice-picker-opening",
            target = local_target_label(mode)
        ));
        Task::perform(async move { pick_local_folder(mode).await }, |result| {
            cosmic::Action::App(Message::LocalFolderPicked(result))
        })
    }

    fn start_onedrive_mirror_setup(&mut self) -> Task<cosmic::Action<Message>> {
        self.start_onedrive_mirror_setup_with_mode(default_onedrive_auth_mode(running_in_flatpak()))
    }

    fn start_onedrive_mirror_manual_setup(&mut self) -> Task<cosmic::Action<Message>> {
        self.start_onedrive_mirror_setup_with_mode(OneDriveMirrorSetupMode::ManualAuthFiles)
    }

    fn start_onedrive_mirror_setup_with_mode(
        &mut self,
        mode: OneDriveMirrorSetupMode,
    ) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::OneDrive
            || self.draft.access_mode != AccessMode::OfflineMirror
            || !self.draft.legacy_onedrive
        {
            self.last_notice = Some(fl!("notice-onedrive-only"));
            return Task::none();
        }
        if self.draft.id.is_none() {
            self.draft.id = Some(ConnectionId::new());
        }
        let connection = match connection_from_draft(&self.draft) {
            Ok(connection) => connection,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        match mode {
            OneDriveMirrorSetupMode::Interactive => {
                self.onedrive_auth_open_command.clear();
                self.onedrive_auth_response_url.clear();
                self.last_notice = Some(fl!(
                    "notice-onedrive-starting",
                    name = connection.name.as_str()
                ));
                Task::perform(
                    async move { run_onedrive_mirror_interactive_setup_result(connection).await },
                    |result| cosmic::Action::App(Message::OneDriveMirrorSetupCompleted(result)),
                )
            }
            OneDriveMirrorSetupMode::ManualAuthFiles => {
                let auth_files = onedrive_auth_files_for_connection(connection.id);
                self.onedrive_auth_open_command = onedrive_auth_open_command(&auth_files);
                self.onedrive_auth_response_url.clear();
                self.last_notice = Some(fl!(
                    "notice-onedrive-manual-starting",
                    name = connection.name.as_str()
                ));
                Task::perform(
                    async move { run_onedrive_mirror_manual_setup_result(connection, auth_files).await },
                    |result| cosmic::Action::App(Message::OneDriveMirrorSetupCompleted(result)),
                )
            }
        }
    }

    fn submit_onedrive_mirror_auth_response(&mut self) {
        if !self.draft.legacy_onedrive {
            return;
        }
        let Some(connection_id) = self.draft.id else {
            self.last_notice = Some(fl!("notice-onedrive-start-before-submit"));
            return;
        };
        let response = self.onedrive_auth_response_url.trim();
        if let Err(error) = validate_onedrive_auth_response_url(response) {
            self.last_notice = Some(error);
            return;
        }
        let auth_files = onedrive_auth_files_for_connection(connection_id);
        match write_onedrive_auth_response(&auth_files, response) {
            Ok(()) => {
                self.onedrive_auth_response_url.clear();
                self.last_notice = Some(fl!("notice-onedrive-response-written"));
            }
            Err(error) => {
                self.last_notice = Some(fl!(
                    "notice-onedrive-response-failed",
                    error = error.to_string()
                ));
            }
        }
    }

    fn open_onedrive_mirror_auth_url(&mut self) -> Task<cosmic::Action<Message>> {
        if !self.draft.legacy_onedrive {
            return Task::none();
        }
        let Some(connection_id) = self.draft.id else {
            self.last_notice = Some(fl!("notice-onedrive-start-before-open"));
            return Task::none();
        };
        let auth_files = onedrive_auth_files_for_connection(connection_id);
        Task::perform(
            async move { open_onedrive_auth_url(&auth_files).await },
            move |result| {
                cosmic::Action::App(Message::OneDriveAuthUrlOpened(connection_id, result))
            },
        )
    }

    fn matching_rclone_remotes_in(
        &self,
        remotes: &[RcloneDraftRemote],
        provider: Provider,
    ) -> Vec<RcloneDraftRemote> {
        remotes
            .iter()
            .filter(|remote| editor_remote_matches(&self.draft, provider, remote))
            .cloned()
            .collect()
    }

    fn confirm_import(&mut self, index: usize) -> Task<cosmic::Action<Message>> {
        let Some(preview) = self.import_previews.get(index).cloned() else {
            self.last_notice = Some(fl!("notice-import-unavailable"));
            return Task::none();
        };
        if preview.active_conflict || preview.local_target_conflict {
            self.last_notice = Some(fl!(
                "notice-import-conflict",
                unit = preview.original_unit_name
            ));
            return Task::none();
        }
        let name = preview.connection.name.clone();
        let original_unit_name = preview.original_unit_name.clone();
        let plan = match replacement_plan(
            preview,
            true,
            true,
            false,
            &default_runtime_root(),
            &default_cache_root(),
            &default_config_root(),
        ) {
            Ok(plan) => plan,
            Err(error) => {
                self.last_notice =
                    Some(fl!("notice-import-plan-failed", error = error.to_string()));
                return Task::none();
            }
        };
        let connection = plan.preview.connection.clone();
        let storage = match AppConfigStorage::runtime() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        match load_runtime_config() {
            Ok(config) => self.config = config,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        }
        let result = self.config.update_validated_with(&storage, |document| {
            if !document
                .connections
                .iter()
                .any(|existing| existing.id == connection.id)
            {
                document.connections.push(connection);
            }
        });
        match result {
            Ok(_) => {
                self.last_notice = Some(fl!(
                    "notice-import-installing",
                    unit = original_unit_name,
                    name = name.as_str()
                ));
                Task::perform(
                    async move { install_import_replacement_unit(plan).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Err(error) => {
                self.last_notice = Some(fl!(
                    "notice-import-save-failed",
                    name = name.as_str(),
                    error = error.to_string()
                ));
                Task::none()
            }
        }
    }

    fn test_draft_plan(&mut self) -> Task<cosmic::Action<Message>> {
        if draft_is_onedrive_rclone(&self.draft) && !self.draft_primary_actions_ready() {
            self.last_notice = Some(fl!("onedrive-editor-verification-required"));
            return Task::none();
        }
        if self.draft.provider == Provider::Teams && self.draft.id.is_none() {
            self.draft.id = Some(ConnectionId::new());
        }
        let connection = match connection_from_draft(&self.draft) {
            Ok(connection) => connection,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(if connection.provider == Provider::Teams {
            crate::fl!(
                "ui-app-checking-the-sharepoint-library-and-selected-folder",
                value1 = (connection.name).to_string()
            )
        } else {
            crate::fl!(
                "ui-app-testing-value1-onedrive-mirror-validation-runs-a",
                value1 = (connection.name).to_string()
            )
        });
        Task::perform(
            async move {
                let mut connection = connection;
                let result = if connection.provider == Provider::Teams {
                    test_sharepoint_draft_connection(&mut connection).await
                } else {
                    test_connection_plan_and_access(&connection, None).await
                };
                (connection, result)
            },
            |(connection, result)| cosmic::Action::App(Message::DraftTested(connection, result)),
        )
    }

    fn remove_connection_with_confirmation(
        &mut self,
        connection_id: ConnectionId,
    ) -> Task<cosmic::Action<Message>> {
        let Some(connection) = self
            .config
            .document
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
            .cloned()
        else {
            self.pending_remove = None;
            self.last_notice = Some(fl!("notice-connection-unavailable"));
            return Task::none();
        };
        let name = connection.name.clone();
        if self.pending_remove != Some(connection_id) {
            self.pending_remove = Some(connection_id);
            self.last_notice = Some(fl!("notice-remove-confirm", name = name.as_str()));
            return Task::none();
        }
        let storage = match AppConfigStorage::runtime() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        match load_runtime_config() {
            Ok(config) => self.config = config,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        }
        let result = self.config.update_validated_with(&storage, |document| {
            document
                .connections
                .retain(|connection| connection.id != connection_id);
        });
        self.pending_remove = None;
        match result {
            Ok(true) => {
                self.removing_connection = Some(connection_id);
                self.last_notice = Some(fl!("notice-remove-units", name = name.as_str()));
                Task::perform(
                    async move { remove_generated_units_for_connection(connection).await },
                    |notice| cosmic::Action::App(Message::RemoveCompleted(notice)),
                )
            }
            Ok(false) => {
                self.removing_connection = None;
                self.last_notice = Some(fl!("notice-already-absent", name = name.as_str()));
                Task::none()
            }
            Err(error) => {
                self.removing_connection = None;
                self.last_notice = Some(fl!(
                    "notice-remove-failed",
                    name = name.as_str(),
                    error = error.to_string()
                ));
                Task::none()
            }
        }
    }

    fn scan_imports(&mut self) -> Task<cosmic::Action<Message>> {
        Task::perform(
            async {
                let text = HostInspection::RcloneConfigDump.run_owned().await?;
                parse_rclone_backends(&text).map_err(|error| error.to_string())
            },
            |result| cosmic::Action::App(Message::ImportInspectionCompleted(result)),
        )
    }

    fn finish_import_scan(&mut self, result: Result<BTreeMap<String, Provider>, String>) {
        let Some(home) = home_dir() else {
            self.import_previews.clear();
            self.last_notice = Some(fl!("notice-import-home-unavailable"));
            return;
        };
        let directory = default_scan_directory(&home);
        let active_units = BTreeSet::new();
        let rclone_remotes = result.ok();
        let rclone_config_unavailable = rclone_remotes.is_none();
        let rclone_backends = rclone_remotes.unwrap_or_default();
        match scan_legacy_units(&directory) {
            Ok(units) => {
                let mut previews = Vec::new();
                let mut errors = Vec::new();
                for unit in units {
                    match preview_import(
                        &unit,
                        &self.config.document.connections,
                        &active_units,
                        &home,
                        &rclone_backends,
                    ) {
                        Ok(preview) => previews.push(preview),
                        Err(error) => errors.push(format!("{}: {error}", unit.name)),
                    }
                }
                let count = previews.len();
                self.import_previews = previews;
                self.last_notice = Some(if rclone_config_unavailable {
                    crate::fl!(
                        "ui-app-import-scan-found-count-service-preview-s",
                        count = (count).to_string()
                    )
                } else if errors.is_empty() {
                    crate::fl!(
                        "ui-app-import-scan-found-count-compatible-service-preview",
                        count = (count).to_string()
                    )
                } else {
                    crate::fl!(
                        "ui-app-import-scan-found-count-compatible-service-preview-2",
                        count = (count).to_string(),
                        value2 = (errors.len()).to_string()
                    )
                });
            }
            Err(error) => {
                self.import_previews.clear();
                self.last_notice =
                    Some(fl!("notice-import-scan-failed", error = error.to_string()));
            }
        }
    }

    fn saved_connection_is_offline_mirror(&self, connection_id: ConnectionId) -> bool {
        self.config.document.connections.iter().any(|connection| {
            connection.id == connection_id
                && matches!(connection.mode, ConnectionMode::OfflineMirror(_))
        })
    }

    fn view_state(&self) -> cosmic_ext_applet_mounter::controller::ControllerViewState {
        restore(&self.controller_snapshot())
    }

    fn unit_status_fresh(&self, id: ConnectionId) -> bool {
        !self.unit_observation_invalidated
            && self.unit_observation_connections == self.config.document.connections
            && self
                .unit_observed_at
                .is_some_and(|time| time.elapsed() < Duration::from_secs(15))
            && !self.unit_observation.failed.contains(&id)
            && self.unit_observation.services.contains_key(&id)
    }

    fn refresh_unit_observation(&mut self, force: bool) -> Task<cosmic::Action<Message>> {
        if self.standalone || (self.unit_observation_pending && !force) {
            return Task::none();
        }
        self.unit_observation_cancellation.cancel();
        self.unit_observation_cancellation = CancellationToken::new();
        self.unit_observation_revision = self.unit_observation_revision.wrapping_add(1);
        self.unit_observation_pending = true;
        self.unit_observation_invalidated |= force;
        let revision = self.unit_observation_revision;
        let connections = self.config.document.connections.clone();
        let source = connections.clone();
        let cancellation = self.unit_observation_cancellation.clone();
        let work_root = default_work_root();
        Task::perform(
            observe_ui_units(source, work_root, cancellation),
            move |observation| {
                cosmic::Action::App(Message::UnitObservationCompleted(
                    revision,
                    connections.clone(),
                    observation,
                ))
            },
        )
    }

    fn cached_offline_state(&self, connection: &Connection) -> SyncRuntimeState {
        let service = self.unit_observation.services.get(&connection.id);
        let timer = self.unit_observation.timers.get(&connection.id);
        // Live service activity remains controllable even if a previous manual
        // operation failed. Otherwise coordinator failures outlive a paused timer.
        if !service.is_some_and(|status| {
            matches!(status.active, ActiveState::Active | ActiveState::Activating)
        }) && matches!(
            self.unit_observation.coordinator_states.get(&connection.id),
            Some(OfflineOperationState::Failed | OfflineOperationState::Cancelled)
        ) {
            return SyncRuntimeState::Failed;
        }
        if service.is_none()
            || (is_rclone_offline_mirror(connection)
                && timer.is_none()
                && !service.is_some_and(|status| {
                    matches!(status.active, ActiveState::Active | ActiveState::Activating)
                }))
        {
            return SyncRuntimeState::Unknown;
        }
        offline_mirror_state_from_status(
            connection.offline_engine().expect("offline mirror"),
            service,
            timer,
        )
    }

    fn cached_offline_mirror_states(&self) -> RuntimeOfflineMirrorStates {
        let mut states = BTreeMap::new();
        let mut paused = BTreeSet::new();
        for connection in self
            .config
            .document
            .connections
            .iter()
            .filter(|connection| matches!(connection.mode, ConnectionMode::OfflineMirror(_)))
        {
            let state = self.cached_offline_state(connection);
            if state == SyncRuntimeState::Paused {
                paused.insert(connection.id);
            }
            states.insert(connection.id, state);
        }
        RuntimeOfflineMirrorStates {
            sync_state: states,
            paused_syncs: paused,
            warnings: self.unit_observation.warnings.clone().unwrap_or_default(),
        }
    }

    fn controller_snapshot(&self) -> ControllerSnapshot {
        let RuntimeOfflineMirrorStates {
            mut sync_state,
            mut paused_syncs,
            warnings: offline_warnings,
        } = self.cached_offline_mirror_states();
        for connection_id in &self.mirror_operations_pending {
            sync_state.insert(*connection_id, SyncRuntimeState::Running);
            paused_syncs.remove(connection_id);
        }
        ControllerSnapshot {
            config: self.config.document.clone(),
            service_status: self.unit_observation.services.clone(),
            sync_state,
            paused_syncs,
            offline_warnings,
            mount_entries: self.mount_entries.clone(),
            online_repair_required: self.online_repair_required.clone(),
            import_previews: self.import_previews.clone(),
            vpn_ready: self.vpn_ready.clone(),
            network_ready: self.network_ready.unwrap_or(true),
            ..ControllerSnapshot::default()
        }
    }
}

struct RuntimeOfflineMirrorStates {
    sync_state: BTreeMap<ConnectionId, SyncRuntimeState>,
    paused_syncs: BTreeSet<ConnectionId>,
    warnings: BTreeMap<ConnectionId, Vec<String>>,
}

async fn observe_ui_units(
    connections: Vec<Connection>,
    work_root: PathBuf,
    cancellation: CancellationToken,
) -> UnitObservation {
    let mut result = UnitObservation::default();
    let mut keys = Vec::new();
    let mut units = Vec::new();
    for connection in &connections {
        keys.push((connection.id, UnitKind::Service));
        units.push(UnitName::new(connection.id, UnitKind::Service));
        if is_rclone_offline_mirror(connection) {
            keys.push((connection.id, UnitKind::Timer));
            units.push(UnitName::new(connection.id, UnitKind::Timer));
        }
    }
    match systemd_user_status_batch(&units, cancellation.clone()).await {
        Ok(observations) => {
            for ((id, kind), (_, status)) in keys.into_iter().zip(observations) {
                if let Some(status) = status {
                    match kind {
                        UnitKind::Service => {
                            result.services.insert(id, status);
                        }
                        UnitKind::Timer => {
                            result.timers.insert(id, status);
                        }
                    }
                } else {
                    result.failed.insert(id);
                }
            }
        }
        Err(_) => result
            .failed
            .extend(connections.iter().map(|connection| connection.id)),
    }
    if cancellation.is_cancelled() {
        return result;
    }
    let offline_ids = connections
        .iter()
        .filter(|connection| matches!(connection.mode, ConnectionMode::OfflineMirror(_)))
        .map(|connection| connection.id)
        .collect::<BTreeSet<_>>();
    let states =
        tokio::task::spawn_blocking(move || observe_offline_state_files(&connections, &work_root));
    let observation = tokio::select! {
        () = cancellation.cancelled() => None,
        result = tokio::time::timeout(Duration::from_secs(1), states) => result.ok().and_then(Result::ok),
    };
    if let Some(observation) = observation {
        result.coordinator_states = observation.states;
        result.warnings = Some(observation.warnings);
        result.failed.extend(observation.failed);
    } else {
        // A timed-out/failed read must not authorize controls from cached success.
        result.failed.extend(offline_ids);
    }
    result
}

#[derive(Default)]
struct OfflineStateObservation {
    states: BTreeMap<ConnectionId, OfflineOperationState>,
    warnings: BTreeMap<ConnectionId, Vec<String>>,
    failed: BTreeSet<ConnectionId>,
}

fn observe_offline_state_files(
    connections: &[Connection],
    work_root: &Path,
) -> OfflineStateObservation {
    let mut observation = OfflineStateObservation::default();
    for connection in connections
        .iter()
        .filter(|connection| matches!(connection.mode, ConnectionMode::OfflineMirror(_)))
    {
        let path =
            cosmic_ext_applet_mounter::offline_coordinator::state_path(work_root, connection.id);
        match cosmic_ext_applet_mounter::offline_coordinator::read_state(&path) {
            Ok(Some(state)) if state.connection_id == connection.id => {
                observation.states.insert(connection.id, state.state);
                if !state.warnings.is_empty() {
                    observation.warnings.insert(connection.id, state.warnings);
                }
            }
            Ok(None) => {}
            _ => {
                observation.failed.insert(connection.id);
            }
        }
    }
    observation
}

async fn current_vpn_ready_states(config: ConfigDocument) -> BTreeMap<VpnProfileId, bool> {
    let mounts = host_mount_entries().await.unwrap_or_default();
    let active_connection_ids = config
        .connections
        .iter()
        .filter(|connection| connection_appears_active(connection, &mounts))
        .map(|connection| connection.id)
        .collect::<BTreeSet<_>>();
    let referenced = referenced_active_vpn_profiles(&config.connections, &active_connection_ids);
    let profiles = config
        .vpn_profiles
        .iter()
        .filter(|profile| referenced.contains(&profile.id))
        .cloned()
        .collect::<Vec<_>>();
    let mut ready = BTreeMap::new();
    for profile in profiles {
        let is_ready = match profile.kind {
            VpnKind::NetworkManager => DbusNetworkManagerVpn
                .state(&profile, CancellationToken::new())
                .await
                .is_ok_and(|state| state.ready()),
            VpnKind::LegacyCisco => false,
        };
        if !is_ready {
            unmark_vpn_applet_activated(profile.id);
        }
        ready.insert(profile.id, is_ready);
    }
    ready
}

fn referenced_active_vpn_profiles(
    connections: &[Connection],
    active_connection_ids: &BTreeSet<ConnectionId>,
) -> BTreeSet<VpnProfileId> {
    connections
        .iter()
        .filter(|connection| connection.enabled)
        .filter(|connection| active_connection_ids.contains(&connection.id))
        .filter_map(|connection| connection.vpn_profile_id)
        .collect()
}

fn offline_mirror_state_from_status(
    engine: OfflineEngine,
    service_status: Option<&UnitStatus>,
    timer_status: Option<&UnitStatus>,
) -> SyncRuntimeState {
    match engine {
        OfflineEngine::LegacyOnedrive => {
            if service_status.as_ref().is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) {
                SyncRuntimeState::Idle
            } else if service_status
                .as_ref()
                .is_some_and(|status| status.active == ActiveState::Failed)
            {
                SyncRuntimeState::Failed
            } else {
                SyncRuntimeState::Paused
            }
        }
        OfflineEngine::Rclone => {
            if service_status.as_ref().is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) {
                return SyncRuntimeState::Running;
            }
            if !timer_status.is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) {
                return SyncRuntimeState::Paused;
            }
            if service_status.is_some_and(|status| status.active == ActiveState::Failed) {
                SyncRuntimeState::Failed
            } else {
                SyncRuntimeState::Idle
            }
        }
    }
}

fn runtime_service_status(connection_id: ConnectionId) -> Option<UnitStatus> {
    runtime_unit_status(connection_id, UnitKind::Service)
}

fn runtime_unit_status(connection_id: ConnectionId, unit_kind: UnitKind) -> Option<UnitStatus> {
    let unit = UnitName::new(connection_id, unit_kind);
    systemd_user_status_blocking(&unit).ok()
}

#[cfg(test)]
fn parse_runtime_systemd_status(output: &str) -> UnitStatus {
    fn property<'a>(output: &'a str, name: &str) -> &'a str {
        output
            .lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix('='))
            .unwrap_or_default()
    }

    let active = match property(output, "ActiveState") {
        "active" => ActiveState::Active,
        "activating" => ActiveState::Activating,
        "inactive" => ActiveState::Inactive,
        "failed" => ActiveState::Failed,
        _ => ActiveState::Unknown,
    };
    let enabled = matches!(
        property(output, "UnitFileState"),
        "enabled" | "enabled-runtime"
    );
    let detail = property(output, "SubState").to_owned();
    UnitStatus {
        active,
        enabled,
        detail,
        result: property(output, "Result").to_owned(),
    }
}

impl AppModel {
    pub fn launch_mode_from_args() -> AppLaunchMode {
        let mut args = env::args().skip(1);
        match args.next().as_deref() {
            Some("--settings") | Some("settings") | Some("--add-connection") => {
                AppLaunchMode::AddConnection
            }
            Some("--app-settings") => AppLaunchMode::GeneralSettings,
            Some("--import") => AppLaunchMode::ImportLegacy,
            Some("--modify-connection") => args
                .next()
                .and_then(|value| Uuid::parse_str(&value).ok())
                .map(ConnectionId::from_uuid)
                .map(AppLaunchMode::ModifyConnection)
                .unwrap_or(AppLaunchMode::AddConnection),
            Some("--reorder-connections") => AppLaunchMode::ReorderConnections,
            _ => AppLaunchMode::Applet,
        }
    }

    pub fn is_standalone_mode(mode: AppLaunchMode) -> bool {
        mode != AppLaunchMode::Applet
    }

    fn launch_settings_process(&mut self, mode: AppLaunchMode) {
        let executable = settings_executable_path();
        let Some(executable) = executable else {
            self.last_notice = Some(fl!("notice-settings-missing"));
            self.last_notice_at = Some(Instant::now());
            return;
        };

        let mut command = Command::new(executable);
        match mode {
            AppLaunchMode::GeneralSettings => {
                command.arg("--app-settings");
            }
            AppLaunchMode::Applet | AppLaunchMode::AddConnection => {
                command.arg("--settings");
            }
            AppLaunchMode::ModifyConnection(id) => {
                command.arg("--modify-connection").arg(id.to_string());
            }
            AppLaunchMode::ImportLegacy => {
                command.arg("--import");
            }
            AppLaunchMode::ReorderConnections => {
                command.arg("--reorder-connections");
            }
        }

        if let Err(error) = command.spawn() {
            self.last_notice = Some(fl!(
                "notice-settings-open-failed",
                error = error.to_string()
            ));
            self.last_notice_at = Some(Instant::now());
        }
    }

    fn reorder_connection(
        &mut self,
        connection_id: ConnectionId,
        new_position: usize,
    ) -> Task<cosmic::Action<Message>> {
        let storage = match AppConfigStorage::runtime() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.reorder_connection_with_storage(connection_id, new_position, &storage)
    }

    fn reorder_connection_with_storage(
        &mut self,
        connection_id: ConnectionId,
        new_position: usize,
        storage: &AppConfigStorage,
    ) -> Task<cosmic::Action<Message>> {
        let current_index = self
            .config
            .document
            .connections
            .iter()
            .position(|connection| connection.id == connection_id);
        let Some(current_index) = current_index else {
            self.last_notice = Some(fl!("notice-connection-unavailable"));
            return Task::none();
        };
        let count = self.config.document.connections.len();
        if count == 0 || new_position == 0 || new_position > count {
            return Task::none();
        }
        let target_index = new_position - 1;
        if current_index == target_index {
            return Task::none();
        }

        let result = self.config.update_validated_with(storage, |document| {
            let connection = document.connections.remove(current_index);
            let insert_index = target_index.min(document.connections.len());
            document.connections.insert(insert_index, connection);
        });
        match result {
            Ok(_) => {
                self.last_notice = Some(fl!("notice-order-saved"));
                notify_runtime_task()
            }
            Err(error) => {
                self.last_notice = Some(fl!("notice-order-save-failed", error = error.to_string()));
                Task::none()
            }
        }
    }
}

fn save_runtime_preference(
    config: &mut Config,
    storage: &AppConfigStorage,
    command: RuntimeCommand,
) -> Result<(), String> {
    config
        .update_validated_with(storage, |document| match command {
            RuntimeCommand::SetUnmount(value) => document.unmount_before_sleep = value,
            RuntimeCommand::SetRestore(value) => document.restore_after_wake = value,
            RuntimeCommand::SetPreload(value) => document.preload = value,
            _ => {}
        })
        .map(|_| ())
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-save-setting-error",
                error = (error).to_string()
            )
        })
}

fn load_runtime_config() -> Result<Config, String> {
    let report = Config::load_runtime();
    if report.warnings.is_empty() {
        Ok(report.config)
    } else {
        Err(crate::fl!(
            "ui-app-could-not-reload-configuration-value1",
            value1 = (report.warnings.join("; ")).to_string()
        ))
    }
}

async fn persist_online_popup_state_with<M: SystemdManager>(
    config: &mut Config,
    storage: &AppConfigStorage,
    manager: &M,
    connection_id: ConnectionId,
    desired_on: bool,
) -> Result<(), String> {
    let previous = config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .ok_or_else(|| crate::fl!("ui-app-connection-is-no-longer-available"))?
        .mode
        .clone();
    let ConnectionMode::OnlineMount(previous_options) = previous else {
        return Err(crate::fl!(
            "ui-app-persistent-popup-state-applies-only-to-online"
        ));
    };
    let previous_on = previous_options.start_at_login;
    let unit = UnitName::new(connection_id, UnitKind::Service);
    let requested_action = if desired_on {
        SystemdAction::Enable
    } else {
        SystemdAction::Disable
    };
    manager
        .action(requested_action, Some(&unit), CancellationToken::new())
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-save-popup-startup-state-error",
                error = (error).to_string()
            )
        })?;

    let update = config.update_validated_with(storage, |document| {
        if let Some(connection) = document
            .connections
            .iter_mut()
            .find(|connection| connection.id == connection_id)
            && let ConnectionMode::OnlineMount(options) = &mut connection.mode
        {
            options.start_at_login = desired_on;
        }
    });
    if let Err(error) = update {
        let rollback = if previous_on {
            SystemdAction::Enable
        } else {
            SystemdAction::Disable
        };
        let rollback_result = manager
            .action(rollback, Some(&unit), CancellationToken::new())
            .await;
        let rollback_note = if rollback_result.is_ok() {
            fl!("startup-policy-restored")
        } else {
            fl!("startup-policy-restore-failed")
        };
        return Err(crate::fl!(
            "ui-app-could-not-persist-popup-startup-state-error",
            error = (error).to_string(),
            rollback_note = (rollback_note).to_string()
        ));
    }
    Ok(())
}

async fn persist_online_popup_state(
    connection_id: ConnectionId,
    desired_on: bool,
) -> Result<(), String> {
    let storage = AppConfigStorage::runtime()?;
    let mut config = load_runtime_config()?;
    let manager = DbusSystemdManager;
    persist_online_popup_state_with(&mut config, &storage, &manager, connection_id, desired_on)
        .await
}

fn runtime_request_task(command: RuntimeCommand) -> Task<cosmic::Action<Message>> {
    Task::perform(runtime_ipc::request(command), move |result| {
        cosmic::Action::App(Message::RuntimeResult(command, result))
    })
}

fn notify_runtime_task() -> Task<cosmic::Action<Message>> {
    Task::perform(
        runtime_ipc::request(RuntimeCommand::ConfigurationChanged),
        |result| cosmic::Action::App(Message::EditorNotified(result)),
    )
}

fn primary_operation(row: &ConnectionRowState) -> Option<Operation> {
    let operation = match (row.mode, row.persistent_active) {
        (AccessMode::OnlineMount, true) => Operation::Unmount,
        (AccessMode::OnlineMount, false) => Operation::Mount,
        (AccessMode::OfflineMirror, true) => Operation::PauseSync,
        (AccessMode::OfflineMirror, false) => Operation::ResumeSync,
    };
    Some(operation)
}

fn repair_available(row: &ConnectionRowState) -> bool {
    row.actions
        .iter()
        .any(|action| action.operation == Operation::Repair && action.enabled)
}

fn repair_visible(
    row: &ConnectionRowState,
    repair_operations_pending: &BTreeSet<ConnectionId>,
) -> bool {
    repair_available(row) && !repair_operations_pending.contains(&row.id)
}

fn update_online_repair_requirement(
    required: &mut BTreeSet<ConnectionId>,
    connection_id: ConnectionId,
    operation: Operation,
    result: &Result<(), String>,
) {
    if result.is_ok() {
        required.remove(&connection_id);
    } else if operation == Operation::Unmount {
        required.insert(connection_id);
    }
}

fn popup_connection_display_name(name: &str) -> String {
    if name.chars().count() <= POPUP_CONNECTION_NAME_MAX_CHARS {
        return name.to_owned();
    }

    let prefix_len = POPUP_CONNECTION_NAME_MAX_CHARS.saturating_sub(3);
    let mut display = name.chars().take(prefix_len).collect::<String>();
    display.push_str("...");
    display
}

fn popup_connection_scroll_height(connection_count: usize, show_empty_state: bool) -> f32 {
    if show_empty_state {
        return POPUP_EMPTY_ROW_HEIGHT;
    }

    (connection_count as f32 * POPUP_CONNECTION_ROW_HEIGHT).min(POPUP_CONNECTION_LIST_MAX_HEIGHT)
}

fn settings_executable_path() -> Option<PathBuf> {
    if let Ok(current) = env::current_exe()
        && current.is_file()
    {
        return Some(current);
    }
    if let Ok(home) = env::var("HOME") {
        let user_install = PathBuf::from(home)
            .join(".local")
            .join("bin")
            .join("cosmic-ext-applet-mounter");
        if user_install.is_file() {
            return Some(user_install);
        }
    }
    Some(PathBuf::from("cosmic-ext-applet-mounter"))
}

fn app_command_runner() -> RuntimeCommandRunner {
    RuntimeCommandRunner::detect_current()
}

#[derive(Debug, Clone)]
struct HostVisibleMountTable(Vec<MountEntry>);

impl HostVisibleMountTable {
    async fn observe() -> Result<Self, String> {
        host_mount_entries().await.map(Self)
    }
}

impl MountTable for HostVisibleMountTable {
    fn entries(&self) -> Result<Vec<MountEntry>, MountTableError> {
        Ok(self.0.clone())
    }
}

fn row_operation_label(status: &ConnectionStatus, operation: Operation) -> String {
    match (status, operation) {
        (ConnectionStatus::OfflineMirror(_), Operation::PauseSync) => fl!("operation-stop"),
        (ConnectionStatus::OfflineMirror(_), Operation::ResumeSync) => fl!("operation-start"),
        _ => operation_label(operation),
    }
}

fn yes_no(value: bool) -> String {
    if value {
        fl!("value-yes")
    } else {
        fl!("value-no")
    }
}

fn window_title(mode: WindowMode) -> String {
    match mode {
        WindowMode::GeneralSettings => fl!("settings-title"),
        WindowMode::ReorderConnections => fl!("reorder-connections-title"),
        _ => fl!("connection-settings-title"),
    }
}

fn window_mode_notice(mode: WindowMode) -> String {
    match mode {
        WindowMode::GeneralSettings => fl!("settings-open-notice"),
        WindowMode::AddConnection => fl!("add-open-notice"),
        WindowMode::ModifyConnection(_) => fl!("modify-open-notice"),
        WindowMode::ImportLegacy => fl!("import-open-notice"),
        WindowMode::ReorderConnections => fl!("reorder-open-notice"),
    }
}

fn access_mode_for_connection(connection: &Connection) -> AccessMode {
    match connection.mode {
        ConnectionMode::OnlineMount(_) => AccessMode::OnlineMount,
        ConnectionMode::OfflineMirror(_) => AccessMode::OfflineMirror,
    }
}

fn draft_from_connection(connection: &Connection) -> ConnectionDraft {
    let (
        access_mode,
        start_at_login,
        cache_limit_gib,
        sync_interval_minutes,
        sync_on_metered,
        recovery_directory,
    ) = match &connection.mode {
        ConnectionMode::OnlineMount(options) => (
            AccessMode::OnlineMount,
            options.start_at_login,
            (options.cache_limit_bytes / (1024 * 1024 * 1024)).to_string(),
            "15".into(),
            false,
            String::new(),
        ),
        ConnectionMode::OfflineMirror(options) => (
            AccessMode::OfflineMirror,
            false,
            "20".into(),
            options.sync_interval_minutes.to_string(),
            options.sync_on_metered,
            options.recovery_directory.display().to_string(),
        ),
    };
    let (smb_user, smb_domain) = if connection.provider == Provider::Smb {
        (String::new(), String::new())
    } else {
        (
            std::env::var("USER").unwrap_or_default(),
            "WORKGROUP".into(),
        )
    };
    let smb_override = if connection.provider == Provider::Sftp {
        connection.sftp_preload_override
    } else {
        connection.smb_preload_override
    };
    let preload_default = if connection.provider == Provider::Sftp {
        PreloadPolicy::sftp_default()
    } else {
        PreloadSettings::default().smb
    };
    ConnectionDraft {
        id: Some(connection.id),
        name: connection.name.clone(),
        provider: connection.provider,
        access_mode,
        offline_engine: match &connection.mode {
            ConnectionMode::OfflineMirror(options) => options.engine,
            ConnectionMode::OnlineMount(_) => None,
        },
        legacy_onedrive: connection.provider == Provider::OneDrive
            && connection.offline_engine() == Some(OfflineEngine::LegacyOnedrive),
        onedrive_identity: connection.onedrive_identity.clone(),
        onedrive_drive_type: connection.onedrive_identity.as_ref().map_or(
            cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
            |identity| identity.drive_type,
        ),
        onedrive_expected_account: String::new(),
        remote_reference: connection.remote_reference.clone(),
        remote_subpath: connection.remote_subpath.clone().unwrap_or_default(),
        sharepoint_library_url: connection
            .teams_identity
            .as_ref()
            .map_or_else(String::new, |identity| identity.library_url.clone()),
        teams_drive_id: connection
            .teams_identity
            .as_ref()
            .map_or_else(String::new, |identity| identity.drive_id.clone()),
        google_client_id: String::new(),
        google_client_secret: String::new(),
        smb_host: String::new(),
        smb_user,
        smb_domain,
        smb_password: String::new(),
        sftp: SftpDraft::default(),
        local_path: connection.local_path.display().to_string(),
        enabled: connection.enabled,
        start_at_login,
        cache_limit_gib,
        sync_interval_minutes,
        sync_on_metered,
        recovery_directory,
        vpn_profile_id: connection.vpn_profile_id,
        disconnect_vpn_when_unused: connection.disconnect_vpn_when_unused,
        connection_preload_use_global: smb_override.is_none(),
        connection_preload_enabled: smb_override
            .map_or(preload_default.enabled, |value| value.enabled),
        connection_preload_seconds: smb_override
            .map_or(preload_default.maximum_seconds, |value| {
                value.maximum_seconds
            })
            .to_string(),
        connection_preload_depth: smb_override
            .map_or(preload_default.maximum_depth.unwrap_or(2), |value| {
                value.maximum_depth
            })
            .to_string(),
    }
}

fn connection_from_draft(draft: &ConnectionDraft) -> Result<Connection, String> {
    let id = draft.id.unwrap_or_default();
    let name = draft.name.trim();
    if name.is_empty() {
        return Err(fl!("validation-name-required"));
    }
    let remote_reference = draft.remote_reference.trim();
    if remote_reference.is_empty() {
        return Err(fl!("validation-remote-required"));
    }
    if draft.local_path.trim().is_empty() {
        return Err(fl!("validation-local-required"));
    }
    let local_path = expand_user_path(draft.local_path.trim());
    let remote_subpath =
        (!draft.remote_subpath.trim().is_empty()).then(|| draft.remote_subpath.trim().to_owned());
    let teams_identity = if draft.provider == Provider::Teams {
        let mut identity = sharepoint::parse_library_url(&draft.sharepoint_library_url)?;
        identity.drive_id = draft.teams_drive_id.clone();
        Some(identity)
    } else {
        None
    };
    let mode = match draft.access_mode {
        AccessMode::OnlineMount => {
            let gib = draft
                .cache_limit_gib
                .trim()
                .parse::<u64>()
                .map_err(|_| fl!("validation-cache-limit"))?;
            ConnectionMode::OnlineMount(OnlineMountConfig {
                cache_directory: None,
                cache_limit_bytes: gib.saturating_mul(1024 * 1024 * 1024),
                start_at_login: draft.start_at_login,
            })
        }
        AccessMode::OfflineMirror => {
            let sync_interval_minutes = draft
                .sync_interval_minutes
                .trim()
                .parse::<u32>()
                .map_err(|_| fl!("validation-sync-interval"))?;
            let recovery_directory = if draft.recovery_directory.trim().is_empty() {
                default_recovery_directory_for(&local_path, id)
            } else {
                expand_user_path(draft.recovery_directory.trim())
            };
            ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                engine: if draft.provider == Provider::OneDrive && !draft.legacy_onedrive {
                    Some(OfflineEngine::Rclone)
                } else {
                    draft.offline_engine
                },
                recovery_directory,
                sync_interval_minutes,
                sync_on_metered: draft.sync_on_metered,
            })
        }
    };
    let preload_override = if matches!(draft.provider, Provider::Smb | Provider::Sftp)
        && draft.access_mode == AccessMode::OnlineMount
        && !draft.connection_preload_use_global
    {
        let maximum_seconds = draft
            .connection_preload_seconds
            .trim()
            .parse::<u64>()
            .map_err(|_| {
                fl!(
                    "validation-connection-preload-time",
                    min = MIN_PRELOAD_SECONDS,
                    max = MAX_PRELOAD_SECONDS
                )
            })?;
        let maximum_depth = draft
            .connection_preload_depth
            .trim()
            .parse::<u8>()
            .map_err(|_| {
                fl!(
                    "validation-connection-preload-depth",
                    min = MIN_PRELOAD_DEPTH,
                    max = MAX_PRELOAD_DEPTH
                )
            })?;
        if !(MIN_PRELOAD_SECONDS..=MAX_PRELOAD_SECONDS).contains(&maximum_seconds) {
            return Err(fl!(
                "validation-connection-preload-time",
                min = MIN_PRELOAD_SECONDS,
                max = MAX_PRELOAD_SECONDS
            ));
        }
        if !(MIN_PRELOAD_DEPTH..=MAX_PRELOAD_DEPTH).contains(&maximum_depth) {
            return Err(fl!(
                "validation-connection-preload-depth",
                min = MIN_PRELOAD_DEPTH,
                max = MAX_PRELOAD_DEPTH
            ));
        }
        Some(SmbPreloadOverride {
            enabled: draft.connection_preload_enabled,
            maximum_seconds,
            maximum_depth,
        })
    } else {
        None
    };
    Ok(Connection {
        id,
        name: name.into(),
        provider: draft.provider,
        mode,
        remote_reference: remote_reference.into(),
        remote_subpath,
        local_path,
        enabled: draft.enabled,
        vpn_profile_id: draft.vpn_profile_id,
        disconnect_vpn_when_unused: draft.disconnect_vpn_when_unused,
        tuning_profile: TuningProfile::Balanced,
        smb_preload_override: if draft.provider == Provider::Smb {
            preload_override
        } else {
            None
        },
        sftp_preload_override: if draft.provider == Provider::Sftp {
            preload_override
        } else {
            None
        },
        teams_identity,
        onedrive_identity: draft.onedrive_identity.clone(),
    })
}

fn sharepoint_verification_matches_draft(draft: &ConnectionDraft, verified: &Connection) -> bool {
    connection_from_draft(draft).is_ok_and(|mut current| {
        current.id = verified.id;
        if let (Some(current_identity), Some(verified_identity)) =
            (&mut current.teams_identity, &verified.teams_identity)
        {
            current_identity.drive_id = verified_identity.drive_id.clone();
        }
        current == *verified
    })
}

fn expand_user_path(value: &str) -> PathBuf {
    if value == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(value));
    }
    PathBuf::from(value)
}

fn default_recovery_directory_for(local_path: &Path, connection_id: ConnectionId) -> PathBuf {
    let parent = local_path.parent().unwrap_or_else(|| Path::new("."));
    let leaf = local_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(safe_path_component)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "mirror".into());
    parent
        .join(".cosmic-mounter-recovery")
        .join(format!("{leaf}-{connection_id}"))
}

fn recovery_directory_placeholder(draft: &ConnectionDraft) -> String {
    let id = draft.id.unwrap_or_default();
    let local_path = draft.local_path.trim();
    if local_path.is_empty() {
        crate::fl!("ui-app-auto-sibling-cosmic-mounter-recovery-directory")
    } else {
        crate::fl!(
            "ui-app-auto-value1",
            value1 = (default_recovery_directory_for(&expand_user_path(local_path), id).display())
                .to_string()
        )
    }
}

fn draft_summary_text(draft: &ConnectionDraft) -> String {
    let engine = if draft.provider == Provider::OneDrive
        && draft.access_mode == AccessMode::OfflineMirror
        && !draft.legacy_onedrive
    {
        "rclone bisync"
    } else {
        provider_engine_summary(draft.provider, draft.access_mode)
    };
    let mut parts = vec![crate::fl!(
        "ui-app-engine-engine",
        engine = (engine).to_string()
    )];
    match connection_from_draft(draft) {
        Ok(connection) => match managed_plan_summary(&connection) {
            Ok(summary) => parts.push(summary),
            Err(error) => parts.push(crate::fl!(
                "ui-app-generated-unit-preview-pending-error",
                error = (error).to_string()
            )),
        },
        Err(_) => {
            parts.push(crate::fl!(
                "ui-app-generated-unit-preview-appears-after-required-fields"
            ));
        }
    }
    parts.push(match draft.access_mode {
        AccessMode::OnlineMount => {
            crate::fl!("ui-app-safety-test-connection-validates-dependency-remote-subtree")
        }
        AccessMode::OfflineMirror => {
            if draft.provider == Provider::Teams {
                fl!("sharepoint-schedule-help")
            } else {
                crate::fl!("ui-app-safety-preview-is-dry-run-initial-synchronization")
            }
        }
    });
    parts.join(" ")
}

fn safe_path_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn choice_row(choices: Vec<Element<'static, Message>>) -> Element<'static, Message> {
    let mut row = widget::Row::new().spacing(8).align_y(Alignment::Center);
    for choice in choices {
        row = row.push(choice);
    }
    row.into()
}

fn rclone_remote_buttons_per_row() -> usize {
    SETTINGS_RCLONE_REMOTE_BUTTONS_PER_ROW.max(1)
}

#[derive(Default)]
struct VpnDetection {
    profiles: Vec<VpnProfile>,
    warnings: Vec<String>,
    debug: String,
}

fn detect_vpn_profiles() -> VpnDetection {
    match std::thread::Builder::new()
        .name("vpn-profile-detection".into())
        .spawn(detect_vpn_profiles_on_worker)
    {
        Ok(worker) => worker.join().unwrap_or_else(|_| VpnDetection {
            warnings: vec![crate::fl!(
                "ui-app-vpn-detection-worker-stopped-unexpectedly"
            )],
            ..VpnDetection::default()
        }),
        Err(error) => VpnDetection {
            warnings: vec![crate::fl!(
                "ui-app-could-not-start-vpn-detection-worker-error",
                error = (error).to_string()
            )],
            ..VpnDetection::default()
        },
    }
}

fn detect_vpn_profiles_on_worker() -> VpnDetection {
    let mut detection = VpnDetection::default();
    writeln!(
        detection.debug,
        "Cloud Mounter VPN detection started at {:?}",
        std::time::SystemTime::now()
    )
    .ok();
    writeln!(
        detection.debug,
        "PATH={}",
        std::env::var("PATH").unwrap_or_else(|_| "<unset>".into())
    )
    .ok();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            detection.warnings.push(crate::fl!(
                "ui-app-could-not-start-vpn-detector-error",
                error = (error).to_string()
            ));
            return detection;
        }
    };

    runtime.block_on(async {
        let cancellation = CancellationToken::new();
        let network_manager = DbusNetworkManagerVpn;
        match network_manager.list_profiles(cancellation.clone()).await {
            Ok(profiles) => {
                writeln!(
                    detection.debug,
                    "adapter NetworkManager profiles: {}",
                    profiles
                        .iter()
                        .map(|profile| format!("{} [{}]", profile.name, profile.vpn_type))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .ok();
                detection
                    .profiles
                    .extend(profiles.into_iter().map(network_manager_profile));
            }
            Err(error) => {
                writeln!(detection.debug, "adapter NetworkManager error: {error}").ok();
                detection.warnings.push(crate::fl!(
                    "ui-app-networkmanager-vpn-detection-failed-error",
                    error = (error).to_string()
                ));
            }
        }
    });

    dedupe_detected_vpns(&mut detection.profiles);
    writeln!(
        detection.debug,
        "final detected profiles: {}",
        detection
            .profiles
            .iter()
            .map(|profile| format!("{} ({})", profile.name, vpn_kind_label(profile.kind)))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .ok();
    write_vpn_detection_log(&detection.debug);
    detection
}

#[cfg(all(test, feature = "development-probes"))]
#[allow(dead_code)]
async fn detect_network_manager_vpns_with_debug(
    runner: &dyn CommandRunner,
    detection: &mut VpnDetection,
    cancellation: CancellationToken,
) {
    for request in [
        CommandRequest::new(Executable::Nmcli)
            .arg("-t")
            .and_then(|request| request.arg("-f"))
            .and_then(|request| request.arg("NAME,TYPE,UUID"))
            .and_then(|request| request.arg("connection"))
            .and_then(|request| request.arg("show"))
            .map(|request| request.with_timeout(Duration::from_secs(5))),
        CommandRequest::new(Executable::Nmcli)
            .arg("-g")
            .and_then(|request| request.arg("NAME,UUID,TYPE"))
            .and_then(|request| request.arg("connection"))
            .and_then(|request| request.arg("show"))
            .map(|request| request.with_timeout(Duration::from_secs(5))),
    ] {
        match request {
            Ok(request) => {
                let command = request.sanitized_command();
                match runner.run(request, cancellation.clone()).await {
                    Ok(output) => {
                        let profiles = log_nmcli_success(&mut detection.debug, &command, &output);
                        detection
                            .profiles
                            .extend(profiles.into_iter().map(network_manager_profile));
                    }
                    Err(error) => log_nmcli_error(&mut detection.debug, &command, &error),
                }
            }
            Err(error) => {
                writeln!(detection.debug, "failed to build nmcli request: {error}").ok();
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn log_nmcli_success(
    debug: &mut String,
    command: &str,
    output: &CommandOutput,
) -> Vec<cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile> {
    let combined = if output.stderr.text.trim().is_empty() {
        output.stdout.text.clone()
    } else {
        format!("{}\nSTDERR:\n{}", output.stdout.text, output.stderr.text)
    };
    let profiles = parse_nmcli_profiles_for_app(&output.stdout.text);
    writeln!(
        debug,
        "command: {command}\n  stdout lines: {}\n  stderr lines: {}\n  parsed profiles: {}\n  sanitized output:\n{}",
        output.stdout.text.lines().count(),
        output.stderr.text.lines().count(),
        profiles
            .iter()
            .map(|profile| format!("{} [{}] {}", profile.name, profile.vpn_type, profile.uuid))
            .collect::<Vec<_>>()
            .join(", "),
        redact_text(&combined)
    )
    .ok();
    profiles
}

#[cfg(test)]
#[allow(dead_code)]
fn log_nmcli_error(debug: &mut String, command: &str, error: &CommandError) {
    writeln!(debug, "command: {command}\n  error: {error}").ok();
}

#[cfg(test)]
fn parse_nmcli_profiles_for_app(
    output: &str,
) -> Vec<cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile> {
    nmcli_profile_records_for_app(output)
        .into_iter()
        .filter_map(|line| {
            let parts = split_nmcli_for_app(&line);
            let (name, vpn_type, uuid) = match parts.as_slice() {
                [name, vpn_type, uuid] => (name, vpn_type, uuid),
                [name, uuid, vpn_type, ..] if looks_like_uuid_for_app(uuid) => {
                    (name, vpn_type, uuid)
                }
                _ => return None,
            };
            valid_nmcli_profile_fields_for_app(name, vpn_type, uuid).then(|| {
                cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile {
                    name: name.trim().to_owned(),
                    vpn_type: vpn_type.trim().to_owned(),
                    uuid: uuid.trim().to_owned(),
                    service_type: None,
                }
            })
        })
        .collect()
}

#[cfg(test)]
fn valid_nmcli_profile_fields_for_app(name: &str, vpn_type: &str, uuid: &str) -> bool {
    !name.trim().is_empty()
        && looks_like_uuid_for_app(uuid.trim())
        && !vpn_type.chars().any(char::is_whitespace)
        && is_vpn_type_for_app(vpn_type)
}

#[cfg(test)]
fn nmcli_profile_records_for_app(output: &str) -> Vec<String> {
    let line_records: Vec<_> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if line_records.len() > 1 {
        line_records
    } else {
        line_records
            .into_iter()
            .chain(flattened_nmcli_records_for_app(output))
            .collect()
    }
}

#[cfg(test)]
fn flattened_nmcli_records_for_app(output: &str) -> impl Iterator<Item = String> + '_ {
    let mut records = Vec::new();
    let mut start = 0;
    while let Some((uuid_start, uuid_end)) = find_uuid_range_for_app(&output[start..]) {
        let record_end = start + uuid_end;
        let record = output[start..record_end].trim();
        if record.contains(':') {
            records.push(record.to_owned());
        }
        start += uuid_start + 36;
        while output[start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            start += output[start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
        }
    }
    records.into_iter()
}

#[cfg(test)]
fn find_uuid_range_for_app(value: &str) -> Option<(usize, usize)> {
    for (index, _) in value.char_indices() {
        let Some(candidate) = value.get(index..index + 36) else {
            continue;
        };
        if looks_like_uuid_for_app(candidate) {
            return Some((index, index + 36));
        }
    }
    None
}

#[cfg(test)]
fn split_nmcli_for_app(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ':' {
            values.push(current);
            current = String::new();
        } else {
            current.push(character);
        }
    }
    values.push(current);
    values
}

#[cfg(test)]
fn looks_like_uuid_for_app(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

#[cfg(test)]
fn is_vpn_type_for_app(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("vpn")
        || lower.contains("wireguard")
        || lower.contains("openvpn")
        || lower.contains("anyconnect")
}

fn write_vpn_detection_log(content: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/cosmic-ext-applet-mounter-vpn-detect.log")
    {
        writeln!(file, "{content}\n---").ok();
    }
}

fn dedupe_detected_vpns(profiles: &mut Vec<VpnProfile>) {
    let mut deduped = Vec::new();
    for profile in profiles.drain(..) {
        if !deduped
            .iter()
            .any(|existing| same_vpn_reference(existing, &profile))
        {
            deduped.push(profile);
        }
    }
    *profiles = deduped;
}

fn network_manager_profile(
    profile: cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile,
) -> VpnProfile {
    let timeout_seconds = if profile.is_openconnect() { 90 } else { 30 };
    VpnProfile {
        id: VpnProfileId::new(),
        name: profile.name,
        kind: VpnKind::NetworkManager,
        external_profile_id: Some(profile.uuid),
        readiness_checks: vec![
            cosmic_ext_applet_mounter::model::ReadinessCheck::NetworkManagerState,
        ],
        timeout_seconds,
    }
}

fn same_vpn_reference(existing: &VpnProfile, detected: &VpnProfile) -> bool {
    existing.kind == detected.kind
        && match detected.kind {
            VpnKind::NetworkManager => {
                existing.external_profile_id == detected.external_profile_id
                    && detected.external_profile_id.is_some()
            }
            VpnKind::LegacyCisco => false,
        }
}

fn update_detected_vpn_profile(existing: &mut VpnProfile, detected: &VpnProfile) {
    existing.name = detected.name.clone();
    existing.readiness_checks = detected.readiness_checks.clone();
    existing.timeout_seconds = detected.timeout_seconds;
}

fn vpn_profile_choices(
    kind: VpnKind,
    profiles: &[VpnProfile],
    selected: Option<VpnProfileId>,
) -> Vec<Element<'static, Message>> {
    profiles
        .iter()
        .filter(|profile| profile.kind == kind)
        .map(|profile| {
            field_with_help(
                select_button(
                    profile.name.clone(),
                    selected == Some(profile.id),
                    Message::DraftVpn(Some(profile.id)),
                ),
                vpn_profile_summary(profile),
            )
        })
        .collect()
}

fn vpn_profile_summary(profile: &VpnProfile) -> String {
    let external = profile
        .external_profile_id
        .clone()
        .unwrap_or_else(|| fl!("vpn-external-client-selected"));
    crate::fl!(
        "ui-app-value1-value2-external-profile-value3-readiness-value4",
        value1 = (vpn_kind_label(profile.kind)).to_string(),
        value2 = (profile.name).to_string(),
        value3 = (external).to_string(),
        value4 = (readiness_summary(profile)).to_string(),
        value5 = (profile.timeout_seconds).to_string()
    )
}

fn readiness_summary(profile: &VpnProfile) -> String {
    if profile.readiness_checks.is_empty() {
        return crate::fl!("ui-app-default-tunnel-readiness");
    }
    profile
        .readiness_checks
        .iter()
        .map(|check| match check {
            cosmic_ext_applet_mounter::model::ReadinessCheck::NetworkManagerState => {
                crate::fl!("ui-app-networkmanager-state")
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Interface(value) => {
                crate::fl!("ui-app-interface-value", value = (value).to_string())
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Route(value) => {
                crate::fl!("ui-app-route-value", value = (value).to_string())
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::DnsName(value) => {
                crate::fl!("ui-app-dns-value", value = (value).to_string())
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Endpoint(value) => {
                crate::fl!("ui-app-endpoint-value", value = (value).to_string())
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn vpn_kind_label(kind: VpnKind) -> String {
    match kind {
        VpnKind::NetworkManager => fl!("vpn-kind-network-manager"),
        VpnKind::LegacyCisco => fl!("vpn-kind-legacy-cisco"),
    }
}

fn section_row<'a>(
    title: impl Into<Cow<'a, str>> + 'a,
    body: impl Into<Element<'a, Message>> + 'a,
) -> Element<'a, Message> {
    widget::container(
        widget::Row::new()
            .spacing(16)
            .align_y(Alignment::Start)
            .push(
                widget::container(widget::text::body(title))
                    .width(Length::Fixed(SETTINGS_SECTION_TITLE_WIDTH))
                    .padding([SETTINGS_SECTION_TITLE_TOP_PADDING, 0])
                    .align_x(Alignment::Start),
            )
            .push(
                widget::container(body)
                    .width(Length::Fill)
                    .align_x(Alignment::Start),
            ),
    )
    .padding([8, 0])
    .width(Length::Fill)
    .into()
}

fn section_row_with_help<'a>(
    title: impl Into<Cow<'a, str>> + 'a,
    help: impl Into<Cow<'a, str>> + 'a,
    body: impl Into<Element<'a, Message>> + 'a,
) -> Element<'a, Message> {
    section_row(title, field_with_help(body, help))
}

fn preload_policy_row<'a>(
    label: impl Into<String>,
    provider: PreloadProvider,
    draft: &'a PreloadPolicyDraft,
) -> Element<'a, Message> {
    let provider_toggle = field_with_help(
        widget::toggler(draft.enabled)
            .label(label.into())
            .spacing(8)
            .on_toggle(move |value| Message::PreloadEnabled(provider, value)),
        preload_provider_help(provider),
    );
    let duration = field_with_help(
        widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                widget::text_input::text_input("60", &draft.maximum_seconds)
                    .on_input(move |value| Message::PreloadSecondsInput(provider, value))
                    .width(Length::Fixed(80.0)),
            )
            .push(widget::text::body(fl!("unit-seconds"))),
        fl!("preload-duration-help"),
    );
    let mut row = widget::Row::new()
        .spacing(16)
        .align_y(Alignment::Center)
        .push(provider_toggle)
        .push(duration);
    if let Some(depth) = &draft.maximum_depth {
        row = row.push(field_with_help(
            widget::Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    widget::text_input::text_input("2", depth)
                        .on_input(move |value| Message::PreloadDepthInput(provider, value))
                        .width(Length::Fixed(60.0)),
                )
                .push(widget::text::body(fl!("unit-levels"))),
            fl!("preload-depth-help"),
        ));
    }
    row.into()
}

fn preload_provider_help(provider: PreloadProvider) -> Cow<'static, str> {
    match provider {
        PreloadProvider::GoogleDrive => fl!("preload-google-drive-help").into(),
        PreloadProvider::OneDrive => fl!("preload-onedrive-help").into(),
        PreloadProvider::Box => fl!("preload-box-help").into(),
        PreloadProvider::SharePoint => fl!("preload-sharepoint-help").into(),
        PreloadProvider::Sftp => fl!("sftp-preload-help").into(),
        PreloadProvider::Smb => fl!("preload-smb-help").into(),
    }
}

fn field_with_help<'a, H>(
    body: impl Into<Element<'a, Message>> + 'a,
    help: H,
) -> Element<'a, Message>
where
    H: Into<Cow<'a, str>> + 'a,
{
    field_with_help_at(
        body,
        TooltipHelp::Plain(help.into()),
        widget::tooltip::Position::Top,
    )
}

fn field_with_safety_help<'a, W, H>(
    body: impl Into<Element<'a, Message>> + 'a,
    warning: W,
    help: H,
) -> Element<'a, Message>
where
    W: Into<Cow<'a, str>> + 'a,
    H: Into<Cow<'a, str>> + 'a,
{
    field_with_help_at(
        body,
        TooltipHelp::Safety {
            warning: warning.into(),
            body: help.into(),
        },
        widget::tooltip::Position::Top,
    )
}

fn field_with_help_at<'a, H>(
    body: impl Into<Element<'a, Message>> + 'a,
    help: H,
    position: widget::tooltip::Position,
) -> Element<'a, Message>
where
    H: Into<TooltipHelp<'a>> + 'a,
{
    let tooltip_body: Element<'a, Message> = match help.into() {
        TooltipHelp::Plain(help) => widget::text::body(help).into(),
        TooltipHelp::Safety { warning, body } => widget::Column::new()
            .spacing(4)
            .push(widget::text::body(warning).font(cosmic::font::bold()))
            .push(widget::text::body(body))
            .into(),
    };
    let tooltip = widget::container(tooltip_body)
        .padding(8)
        .width(Length::Fixed(320.0));

    widget::tooltip(body, tooltip, position)
        .delay(Duration::from_secs(1))
        .into()
}

enum TooltipHelp<'a> {
    Plain(Cow<'a, str>),
    Safety {
        warning: Cow<'a, str>,
        body: Cow<'a, str>,
    },
}

impl<'a> From<&'a str> for TooltipHelp<'a> {
    fn from(value: &'a str) -> Self {
        Self::Plain(Cow::Borrowed(value))
    }
}

impl From<String> for TooltipHelp<'static> {
    fn from(value: String) -> Self {
        Self::Plain(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for TooltipHelp<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Plain(value)
    }
}

fn provider_choice(
    label: impl Into<Cow<'static, str>>,
    provider: Provider,
    selected: Provider,
    locked: bool,
) -> Element<'static, Message> {
    locked_select_button(
        label,
        provider == selected,
        Message::DraftProvider(provider),
        locked,
        fl!("provider-locked-help"),
    )
}

fn mode_choice(
    label: impl Into<Cow<'static, str>>,
    mode: AccessMode,
    selected: AccessMode,
    locked: bool,
) -> Element<'static, Message> {
    locked_select_button(
        label,
        mode == selected,
        Message::DraftAccessMode(mode),
        locked,
        fl!("access-mode-locked-help"),
    )
}

fn select_button<'a>(
    label: impl Into<std::borrow::Cow<'a, str>>,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    if selected {
        widget::button::suggested(label).on_press(message).into()
    } else {
        widget::button::standard(label).on_press(message).into()
    }
}

fn locked_select_button<'a>(
    label: impl Into<std::borrow::Cow<'a, str>>,
    selected: bool,
    message: Message,
    locked: bool,
    locked_help: impl Into<Cow<'a, str>> + 'a,
) -> Element<'a, Message> {
    let button: Element<'a, Message> = if selected {
        widget::button::suggested(label)
            .on_press_maybe((!locked).then_some(message))
            .into()
    } else {
        widget::button::standard(label)
            .on_press_maybe((!locked).then_some(message))
            .into()
    };

    if locked {
        field_with_help(button, locked_help)
    } else {
        button
    }
}

fn action_button<'a>(
    label: impl Into<std::borrow::Cow<'a, str>>,
    primary: bool,
    message: Message,
) -> Element<'a, Message> {
    if primary {
        widget::button::suggested(label).on_press(message).into()
    } else {
        widget::button::standard(label).on_press(message).into()
    }
}

fn soft_destructive_button<'a>(
    label: impl Into<std::borrow::Cow<'a, str>>,
    message: Message,
) -> Element<'a, Message> {
    widget::button::standard(label)
        .class(cosmic::theme::Button::Custom {
            active: Box::new(|focused, theme| {
                let destructive = cosmic::theme::Button::Destructive;
                let mut style = <cosmic::Theme as widget::button::Catalog>::active(
                    theme,
                    focused,
                    false,
                    &destructive,
                );
                soften_button_background(&mut style, SETTINGS_DISABLE_BUTTON_ACTIVE_LIGHTENING);
                style
            }),
            disabled: Box::new(|theme| {
                let destructive = cosmic::theme::Button::Destructive;
                let mut style =
                    <cosmic::Theme as widget::button::Catalog>::disabled(theme, &destructive);
                soften_button_background(&mut style, SETTINGS_DISABLE_BUTTON_DISABLED_LIGHTENING);
                style
            }),
            hovered: Box::new(|focused, theme| {
                let destructive = cosmic::theme::Button::Destructive;
                let mut style = <cosmic::Theme as widget::button::Catalog>::hovered(
                    theme,
                    focused,
                    false,
                    &destructive,
                );
                soften_button_background(&mut style, SETTINGS_DISABLE_BUTTON_HOVER_LIGHTENING);
                style
            }),
            pressed: Box::new(|focused, theme| {
                let destructive = cosmic::theme::Button::Destructive;
                let mut style = <cosmic::Theme as widget::button::Catalog>::pressed(
                    theme,
                    focused,
                    false,
                    &destructive,
                );
                soften_button_background(&mut style, SETTINGS_DISABLE_BUTTON_PRESSED_LIGHTENING);
                style
            }),
        })
        .on_press(message)
        .into()
}

fn soften_button_background(style: &mut widget::button::Style, amount: f32) {
    if let Some(Background::Color(color)) = style.background {
        style.background = Some(Background::Color(lighten_color(color, amount)));
    }
}

fn lighten_color(color: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: color.r + ((1.0 - color.r) * amount),
        g: color.g + ((1.0 - color.g) * amount),
        b: color.b + ((1.0 - color.b) * amount),
        a: color.a,
    }
}

fn local_target_label(mode: AccessMode) -> String {
    match mode {
        AccessMode::OnlineMount => fl!("local-target-mountpoint"),
        AccessMode::OfflineMirror => fl!("local-target-mirror"),
    }
}

async fn pick_local_folder(mode: AccessMode) -> Result<Option<String>, String> {
    pick_folder(local_target_label(mode)).await
}

async fn pick_folder(target: String) -> Result<Option<String>, String> {
    let dialog = file_chooser::open::Dialog::new()
        .title(crate::fl!("ui-app-choose-value1", value1 = target));

    match dialog.open_folder().await {
        Ok(response) => {
            let path = response
                .url()
                .to_file_path()
                .map_err(|()| crate::fl!("ui-app-selected-folder-is-not-a-local-filesystem"))?;
            host_visible_selected_folder_path(&path).await.map(Some)
        }
        Err(file_chooser::Error::Cancelled) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn selected_folder_path(path: &Path) -> String {
    path.display().to_string()
}

async fn host_visible_selected_folder_path(path: &Path) -> Result<String, String> {
    let origin = document_portal_origin_path(path).await?;
    if let Some(host) = &origin {
        cosmic_ext_applet_mounter::mirror_access::remember_directory(host, path)?;
    }
    Ok(origin
        .as_deref()
        .map(selected_folder_path)
        .unwrap_or_else(|| selected_folder_path(path)))
}

async fn document_portal_origin_path(path: &Path) -> Result<Option<PathBuf>, String> {
    if !is_document_portal_path(path) {
        return Ok(None);
    }
    let sandboxed = running_in_flatpak();
    let dbus_result = tokio::time::timeout(
        Duration::from_secs(5),
        document_portal_origin_path_dbus(path, sandboxed),
    )
    .await
    .map_err(|_| crate::fl!("ui-app-selected-folder-is-a-document-portal-path"))?;
    match dbus_result {
        Ok(origin) => return Ok(Some(origin)),
        Err(error)
            if sandboxed && error.kind == DocumentPortalErrorKind::UnsupportedSandboxVersion =>
        {
            // Documents.GetHostPaths is the first host-path API available to
            // sandboxed callers. Preserve compatibility with portal v4 hosts
            // until the Flatpak runtime can require interface version 5.
        }
        Err(error) => {
            return Err(crate::fl!(
                "ui-app-selected-folder-is-a-document-portal-path-2",
                value1 = (error.message).to_string()
            ));
        }
    }

    legacy_document_portal_origin_path(path).await.map(Some)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentPortalErrorKind {
    InvalidPath,
    Dbus,
    UnsupportedSandboxVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentPortalError {
    kind: DocumentPortalErrorKind,
    message: String,
}

async fn document_portal_origin_path_dbus(
    path: &Path,
    sandboxed: bool,
) -> Result<PathBuf, DocumentPortalError> {
    let connection = zbus::Connection::session()
        .await
        .map_err(|error| DocumentPortalError {
            kind: DocumentPortalErrorKind::Dbus,
            message: crate::fl!(
                "ui-app-connect-to-documents-portal-error",
                error = (error).to_string()
            ),
        })?;
    let proxy = zbus::Proxy::new(
        &connection,
        DOCUMENTS_DESTINATION,
        DOCUMENTS_PATH,
        DOCUMENTS_INTERFACE,
    )
    .await
    .map_err(|error| DocumentPortalError {
        kind: DocumentPortalErrorKind::Dbus,
        message: crate::fl!(
            "ui-app-connect-to-documents-portal-error",
            error = (error).to_string()
        ),
    })?;
    let mount_point: Vec<u8> =
        proxy
            .call("GetMountPoint", &())
            .await
            .map_err(|error| DocumentPortalError {
                kind: DocumentPortalErrorKind::Dbus,
                message: crate::fl!(
                    "ui-app-read-documents-portal-mount-point-error",
                    error = (error).to_string()
                ),
            })?;
    let mount_point = portal_file_path(mount_point)?;
    let document_id =
        portal_document_id(path, &mount_point).ok_or_else(|| DocumentPortalError {
            kind: DocumentPortalErrorKind::InvalidPath,
            message: crate::fl!("ui-app-selected-path-does-not-contain-a-valid"),
        })?;
    let version: u32 =
        proxy
            .get_property("version")
            .await
            .map_err(|error| DocumentPortalError {
                kind: DocumentPortalErrorKind::Dbus,
                message: crate::fl!(
                    "ui-app-read-documents-portal-version-error",
                    error = (error).to_string()
                ),
            })?;

    let origin = if sandboxed {
        if version < DOCUMENTS_GET_HOST_PATHS_VERSION {
            return Err(DocumentPortalError {
                kind: DocumentPortalErrorKind::UnsupportedSandboxVersion,
                message: crate::fl!(
                    "ui-app-documents-portal-version-version-does-not-provide",
                    version = (version).to_string()
                ),
            });
        }
        let paths: HashMap<String, Vec<u8>> = proxy
            .call("GetHostPaths", &(vec![document_id.as_str()],))
            .await
            .map_err(|error| DocumentPortalError {
                kind: DocumentPortalErrorKind::Dbus,
                message: crate::fl!(
                    "ui-app-resolve-documents-portal-host-path-error",
                    error = (error).to_string()
                ),
            })?;
        paths
            .get(&document_id)
            .cloned()
            .ok_or_else(|| DocumentPortalError {
                kind: DocumentPortalErrorKind::InvalidPath,
                message: crate::fl!("ui-app-documents-portal-did-not-return-the-selected"),
            })?
    } else {
        let (origin, _permissions): (Vec<u8>, HashMap<String, Vec<String>>) = proxy
            .call("Info", &(document_id.as_str(),))
            .await
            .map_err(|error| DocumentPortalError {
                kind: DocumentPortalErrorKind::Dbus,
                message: crate::fl!(
                    "ui-app-resolve-documents-portal-host-path-error",
                    error = (error).to_string()
                ),
            })?;
        origin
    };
    portal_file_path(origin)
}

fn portal_file_path(mut bytes: Vec<u8>) -> Result<PathBuf, DocumentPortalError> {
    if bytes.last() == Some(&0) {
        bytes.pop();
    }
    if bytes.is_empty() || bytes.contains(&0) {
        return Err(DocumentPortalError {
            kind: DocumentPortalErrorKind::InvalidPath,
            message: crate::fl!("ui-app-documents-portal-returned-an-invalid-filesystem-path"),
        });
    }
    #[cfg(unix)]
    let path = PathBuf::from(OsString::from_vec(bytes));
    #[cfg(not(unix))]
    let path = PathBuf::from(String::from_utf8(bytes).map_err(|_| DocumentPortalError {
        kind: DocumentPortalErrorKind::InvalidPath,
        message: crate::fl!("ui-app-documents-portal-returned-a-non-utf-8"),
    })?);
    if !path.is_absolute() {
        return Err(DocumentPortalError {
            kind: DocumentPortalErrorKind::InvalidPath,
            message: crate::fl!("ui-app-documents-portal-returned-a-non-absolute-filesystem"),
        });
    }
    Ok(path)
}

fn portal_document_id(path: &Path, mount_point: &Path) -> Option<String> {
    let mut components = path.strip_prefix(mount_point).ok()?.components();
    let first = components.next()?.as_os_str().to_str()?;
    let id = if first == "by-app" {
        components.next()?;
        components.next()?.as_os_str().to_str()?
    } else {
        first
    };
    (!id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
    .then(|| id.to_owned())
}

async fn legacy_document_portal_origin_path(path: &Path) -> Result<PathBuf, String> {
    let path_text = path.display().to_string();
    let request = CommandRequest::new(Executable::Flatpak)
        .arg("document-info")
        .and_then(|request| request.arg(path_text))
        .map_err(|error| error.to_string())?
        .with_timeout(Duration::from_secs(5))
        .with_output_limit(64 * 1024);
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-flatpak-document-info-failed-error",
                error = (error).to_string()
            )
        })?;
    if output.stdout.truncated || output.stdout.invalid_utf8 {
        return Err(crate::fl!(
            "ui-app-flatpak-document-info-returned-an-incomplete-response"
        ));
    }
    parse_document_portal_origin(&output.stdout.text)
        .ok_or_else(|| crate::fl!("ui-app-selected-folder-is-a-document-portal-path-3"))
}

fn is_document_portal_path(path: &Path) -> bool {
    let parts = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    matches!(parts.as_slice(), ["/", "run", "user", _, "doc", ..])
}

fn parse_document_portal_origin(output: &str) -> Option<PathBuf> {
    output.lines().find_map(|line| {
        line.trim()
            .strip_prefix("origin:")
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(PathBuf::from)
    })
}

fn rclone_backend_name(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::GoogleDrive => Some("drive"),
        Provider::Box => Some("box"),
        Provider::Smb => Some("smb"),
        Provider::Sftp => Some("sftp"),
        Provider::Teams => Some("onedrive"),
        Provider::OneDrive => None,
    }
}

fn sftp_preload_global_label() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("sftp-preload-global"));
    &LABEL
}

fn smb_preload_global_label() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("preload-smb-global"));
    &LABEL
}

fn rclone_remote_directory_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("rclone-remote-directory"));
    &LABEL
}

fn onedrive_folder_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("onedrive-folder"));
    &LABEL
}

fn onedrive_auth_shell_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("onedrive-auth-shell-command"));
    &LABEL
}

fn onedrive_auth_redirect_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("onedrive-auth-redirect"));
    &LABEL
}

fn google_drive_client_id_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("google-drive-client-id"));
    &LABEL
}

fn google_drive_client_secret_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("google-drive-client-secret"));
    &LABEL
}

fn smb_host_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("smb-host"));
    &LABEL
}

fn smb_username_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("smb-username"));
    &LABEL
}

fn smb_domain_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("smb-domain"));
    &LABEL
}

fn smb_password_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("smb-password"));
    &LABEL
}

fn provider_editor_label(provider: Provider) -> String {
    match provider {
        Provider::OneDrive => fl!("provider-onedrive"),
        Provider::Teams => fl!("provider-teams"),
        Provider::GoogleDrive => fl!("provider-google-drive"),
        Provider::Box => fl!("provider-box"),
        Provider::Smb => fl!("provider-smb"),
        Provider::Sftp => fl!("sftp-provider"),
    }
}

fn sftp_remote_directory_placeholder() -> &'static str {
    static LABEL: LazyLock<String> = LazyLock::new(|| fl!("sftp-remote-directory"));
    &LABEL
}

fn rclone_remote_placeholder(provider: Provider) -> &'static str {
    static REMOTE_LABEL: LazyLock<String> = LazyLock::new(|| fl!("remote-name-placeholder"));
    static GOOGLE_DRIVE_LABEL: LazyLock<String> = LazyLock::new(|| fl!("google-drive-remote-name"));
    static BOX_LABEL: LazyLock<String> = LazyLock::new(|| fl!("box-remote-name"));
    static SMB_LABEL: LazyLock<String> = LazyLock::new(|| fl!("smb-remote-name"));
    static SFTP_LABEL: LazyLock<String> = LazyLock::new(|| fl!("sftp-remote-name"));
    match provider {
        Provider::GoogleDrive => &GOOGLE_DRIVE_LABEL,
        Provider::Box => &BOX_LABEL,
        Provider::Smb => &SMB_LABEL,
        Provider::Sftp => &SFTP_LABEL,
        Provider::Teams => {
            static TEAMS_LABEL: LazyLock<String> = LazyLock::new(|| fl!("teams-remote-name"));
            &TEAMS_LABEL
        }
        Provider::OneDrive => &REMOTE_LABEL,
    }
}

fn rclone_remote_help(provider: Provider, adding: bool) -> String {
    if !adding {
        return match provider {
            Provider::GoogleDrive => fl!("google-drive-remote-help-modify"),
            Provider::Box => fl!("box-remote-help-modify"),
            Provider::Smb => fl!("smb-remote-help-modify"),
            Provider::Sftp => fl!("sftp-remote-help-modify"),
            Provider::Teams => fl!("teams-remote-help-modify"),
            Provider::OneDrive => {
                crate::fl!("ui-app-onedrive-uses-its-own-authentication-workflow")
            }
        };
    }

    match provider {
        Provider::GoogleDrive => fl!("google-drive-remote-help-add"),
        Provider::Box => fl!("box-remote-help-add"),
        Provider::Smb => fl!("smb-remote-help-add"),
        Provider::Sftp => fl!("sftp-remote-help-add"),
        Provider::Teams => fl!("teams-remote-help-add"),
        Provider::OneDrive => {
            crate::fl!("ui-app-onedrive-does-not-use-rclone-in-the")
        }
    }
}

fn onedrive_account_placeholder(mode: AccessMode) -> &'static str {
    static ONLINE: LazyLock<String> = LazyLock::new(|| fl!("onedrive-account-online"));
    static OFFLINE: LazyLock<String> = LazyLock::new(|| fl!("onedrive-account-offline"));
    match mode {
        AccessMode::OnlineMount => &ONLINE,
        AccessMode::OfflineMirror => &OFFLINE,
    }
}

fn default_onedrive_account_label(id: ConnectionId) -> String {
    format!(
        "onedrive-{}",
        id.to_string().split('-').next().unwrap_or_default()
    )
}

fn onedrive_account_help(mode: AccessMode) -> String {
    match mode {
        AccessMode::OnlineMount => fl!("onedrive-account-online-help"),
        AccessMode::OfflineMirror => fl!("onedrive-account-offline-help"),
    }
}

fn onedrive_account_safety_warning(mode: AccessMode) -> String {
    match mode {
        AccessMode::OnlineMount => fl!("onedrive-account-online-warning"),
        AccessMode::OfflineMirror => fl!("onedrive-account-offline-warning"),
    }
}

fn onedrive_setup_guidance(mode: AccessMode) -> String {
    match mode {
        AccessMode::OnlineMount => fl!("onedrive-setup-online-help"),
        AccessMode::OfflineMirror => fl!("onedrive-setup-offline-help"),
    }
}

fn parse_rclone_remotes_for_app(output: &str) -> Result<Vec<RcloneDraftRemote>, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| crate::fl!("ui-app-invalid-json-error", error = (error).to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| crate::fl!("ui-app-rclone-config-dump-did-not-return-an"))?;
    let mut remotes = Vec::new();
    for (name, config) in object {
        let Some(remote_config) = config.as_object() else {
            continue;
        };
        let Some(backend) = remote_config.get("type").and_then(|value| value.as_str()) else {
            continue;
        };
        if matches!(backend, "drive" | "box" | "smb" | "sftp")
            || (backend == "onedrive"
                && remote_config
                    .get("drive_type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|kind| {
                        matches!(kind, "documentLibrary" | "personal" | "business")
                    }))
        {
            remotes.push(RcloneDraftRemote {
                name: name.clone(),
                backend: backend.to_owned(),
                drive_type: remote_config
                    .get("drive_type")
                    .and_then(|value| value.as_str())
                    .and_then(|kind| {
                        cosmic_ext_applet_mounter::onedrive::DriveType::parse(kind).ok()
                    }),
            });
        }
    }
    remotes.sort_by(|left, right| {
        left.backend
            .cmp(&right.backend)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(remotes)
}

fn parse_smb_remote_details(
    output: &str,
    remote_name: &str,
) -> Result<Option<SmbRemoteDetails>, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| crate::fl!("ui-app-invalid-json-error", error = (error).to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| crate::fl!("ui-app-rclone-config-dump-did-not-return-an"))?;
    let Some(remote_config) = object.get(remote_name).and_then(|value| value.as_object()) else {
        return Ok(None);
    };
    let Some("smb") = remote_config.get("type").and_then(|value| value.as_str()) else {
        return Ok(None);
    };
    Ok(Some(SmbRemoteDetails {
        host: remote_config
            .get("host")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
        user: remote_config
            .get("user")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
        domain: remote_config
            .get("domain")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
    }))
}

fn rclone_config_has_section(output: &str, remote_name: &str) -> Result<bool, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| crate::fl!("ui-app-invalid-json-error", error = (error).to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| crate::fl!("ui-app-rclone-config-dump-did-not-return-an"))?;
    Ok(object.contains_key(remote_name))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmbRemoteSetup {
    name: String,
    host: Option<String>,
    user: Option<String>,
    domain: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmbRemoteApplyResult {
    remote_name: String,
    password_updated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoxRemoteSetup {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoogleDriveRemoteSetup {
    name: String,
    client_id: Option<String>,
    client_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleDriveRemoteApplyResult {
    remote_name: String,
    updated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoogleDriveRemoteAction {
    Create,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OneDriveMirrorAuthFiles {
    auth_url_file: PathBuf,
    response_url_file: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OneDriveMirrorSetupMode {
    Interactive,
    ManualAuthFiles,
}

impl GoogleDriveRemoteSetup {
    fn from_draft(draft: &ConnectionDraft) -> Result<Self, String> {
        if draft.provider != Provider::GoogleDrive {
            return Err(crate::fl!(
                "ui-app-google-drive-remote-setup-requires-the-google"
            ));
        }
        let name = draft.remote_reference.trim();
        validate_rclone_remote_create_name(name)?;
        let client_id = optional_setup_value(
            &crate::fl!("ui-app-google-oauth-client-id"),
            &draft.google_client_id,
        )?;
        let client_secret = optional_secret_value(
            &crate::fl!("ui-app-google-oauth-client-secret"),
            &draft.google_client_secret,
        )?;
        if client_id.is_some() != client_secret.is_some() {
            return Err(crate::fl!(
                "ui-app-google-oauth-client-id-and-client-secret"
            ));
        }
        Ok(Self {
            name: name.to_owned(),
            client_id,
            client_secret,
        })
    }
}

impl BoxRemoteSetup {
    fn from_draft(draft: &ConnectionDraft) -> Result<Self, String> {
        if draft.provider != Provider::Box {
            return Err(crate::fl!(
                "ui-app-box-remote-setup-requires-the-box-provider"
            ));
        }
        let name = draft.remote_reference.trim();
        validate_rclone_remote_create_name(name)?;
        Ok(Self {
            name: name.to_owned(),
        })
    }
}

impl SmbRemoteSetup {
    fn from_draft(draft: &ConnectionDraft) -> Result<Self, String> {
        if draft.provider != Provider::Smb {
            return Err(crate::fl!(
                "ui-app-smb-remote-setup-requires-the-smb-provider"
            ));
        }
        let name = draft.remote_reference.trim();
        validate_rclone_remote_create_name(name)?;
        let host = optional_setup_value(&crate::fl!("ui-app-smb-host"), &draft.smb_host)?;
        let user = optional_setup_value(&crate::fl!("ui-app-smb-username"), &draft.smb_user)?;
        let domain = optional_setup_value(&crate::fl!("ui-app-smb-domain"), &draft.smb_domain)?;
        let password =
            optional_secret_value(&crate::fl!("ui-app-smb-password"), &draft.smb_password)?;
        Ok(Self {
            name: name.to_owned(),
            host,
            user,
            domain,
            password,
        })
    }
}

async fn apply_google_drive_rclone_remote_result(
    setup: GoogleDriveRemoteSetup,
    action: GoogleDriveRemoteAction,
) -> Result<GoogleDriveRemoteApplyResult, String> {
    apply_google_drive_rclone_remote_with(&app_command_runner(), setup, action).await
}

async fn apply_google_drive_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: GoogleDriveRemoteSetup,
    action: GoogleDriveRemoteAction,
) -> Result<GoogleDriveRemoteApplyResult, String> {
    let backend = rclone_remote_backend(runner, &setup.name).await?;
    let updated = match (action, backend) {
        (GoogleDriveRemoteAction::Create, None) => {
            let create_result = runner
                .run(
                    google_drive_rclone_config_create_request(&setup)?,
                    CancellationToken::new(),
                )
                .await;
            if let Err(error) = create_result {
                cleanup_failed_rclone_remote_create(runner, &setup.name).await;
                return Err(rclone_setup_command_error(
                    &crate::fl!("ui-app-google-drive-oauth-setup"),
                    error,
                ));
            }
            false
        }
        (GoogleDriveRemoteAction::Create, Some(backend)) => {
            return Err(crate::fl!(
                "ui-app-rclone-remote-value1-already-exists-with-backend",
                value1 = (setup.name).to_string(),
                backend = (backend).to_string()
            ));
        }
        (GoogleDriveRemoteAction::Update, Some(backend)) if backend == "drive" => {
            if setup.client_id.is_none() {
                return Err(crate::fl!("ui-app-enter-the-google-oauth-client-id-and"));
            }
            let output = runner
                .run(
                    google_drive_rclone_config_update_request(&setup)?,
                    CancellationToken::new(),
                )
                .await
                .map_err(|error| {
                    rclone_setup_command_error(
                        &crate::fl!("ui-app-google-drive-oauth-update"),
                        error,
                    )
                })?;
            ensure_rclone_config_completed(
                &crate::fl!("ui-app-google-drive-oauth-update"),
                &output,
            )?;
            true
        }
        (GoogleDriveRemoteAction::Update, Some(backend)) => {
            return Err(crate::fl!(
                "ui-app-rclone-remote-value1-uses-backend-backend-not",
                value1 = (setup.name).to_string(),
                backend = (backend).to_string()
            ));
        }
        (GoogleDriveRemoteAction::Update, None) => {
            return Err(crate::fl!(
                "ui-app-google-drive-rclone-remote-value1-no-longer",
                value1 = (setup.name).to_string()
            ));
        }
    };
    Ok(GoogleDriveRemoteApplyResult {
        remote_name: setup.name,
        updated,
    })
}

async fn create_box_rclone_remote_result(setup: BoxRemoteSetup) -> Result<String, String> {
    create_box_rclone_remote_with(&app_command_runner(), setup).await
}

async fn create_box_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: BoxRemoteSetup,
) -> Result<String, String> {
    ensure_rclone_remote_name_available(runner, &setup.name).await?;
    let create_result = runner
        .run(
            box_rclone_config_create_request(&setup)?,
            CancellationToken::new(),
        )
        .await;
    if let Err(error) = create_result {
        cleanup_failed_rclone_remote_create(runner, &setup.name).await;
        return Err(rclone_setup_command_error(
            &crate::fl!("ui-app-box-oauth-setup"),
            error,
        ));
    }
    Ok(setup.name)
}

async fn create_smb_rclone_remote_result(
    setup: SmbRemoteSetup,
) -> Result<SmbRemoteApplyResult, String> {
    create_smb_rclone_remote_with(&app_command_runner(), setup).await
}

async fn create_smb_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: SmbRemoteSetup,
) -> Result<SmbRemoteApplyResult, String> {
    let metadata_updated = setup.has_metadata();
    match rclone_remote_backend(runner, &setup.name).await? {
        None => {
            let create_result = runner
                .run(
                    smb_rclone_config_create_request(&setup)?,
                    CancellationToken::new(),
                )
                .await;
            if let Err(error) = create_result {
                cleanup_failed_rclone_remote_create(runner, &setup.name).await;
                return Err(rclone_setup_command_error(
                    &crate::fl!("ui-app-config-create"),
                    error,
                ));
            }
        }
        Some(backend) if backend == "smb" => {
            if !metadata_updated && setup.password.is_none() {
                return Err(crate::fl!(
                    "ui-app-enter-smb-host-username-domain-or-password"
                ));
            }
            if metadata_updated {
                runner
                    .run(
                        smb_rclone_config_update_request(&setup)?,
                        CancellationToken::new(),
                    )
                    .await
                    .map_err(|error| {
                        rclone_setup_command_error(&crate::fl!("ui-app-config-update"), error)
                    })?;
            }
        }
        Some(backend) => {
            return Err(crate::fl!(
                "ui-app-rclone-remote-value1-already-exists-with-backend-2",
                value1 = (setup.name).to_string(),
                backend = (backend).to_string()
            ));
        }
    }
    let password_updated = setup.password.is_some();
    if password_updated {
        runner
            .run(
                smb_rclone_config_password_request(&setup)?,
                CancellationToken::new(),
            )
            .await
            .map_err(|error| {
                rclone_setup_command_error(&crate::fl!("ui-app-config-password"), error)
            })?;
    }
    Ok(SmbRemoteApplyResult {
        remote_name: setup.name,
        password_updated,
    })
}

impl SmbRemoteSetup {
    fn has_metadata(&self) -> bool {
        self.host.is_some() || self.user.is_some() || self.domain.is_some()
    }
}

async fn remove_rclone_remote_result(remote_name: String) -> Result<String, String> {
    remove_rclone_remote_with(&app_command_runner(), remote_name).await
}

async fn remove_rclone_remote_with(
    runner: &dyn CommandRunner,
    remote_name: String,
) -> Result<String, String> {
    validate_rclone_remote_create_name(&remote_name)?;
    if runner.resolve(Executable::Rclone).is_none() {
        return Err(crate::fl!("ui-app-rclone-is-missing-install-rclone-1-74"));
    }
    runner
        .run(
            rclone_config_delete_request(&remote_name)?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error(&crate::fl!("ui-app-config-delete"), error))?;
    Ok(remote_name)
}

async fn run_onedriver_online_setup_result(connection: Connection) -> Result<String, String> {
    let mounts = HostVisibleMountTable::observe().await?;
    let summary = run_onedriver_online_setup_with(
        &app_command_runner(),
        &mounts,
        &connection,
        &default_cache_root(),
        &default_config_root(),
    )
    .await?;
    verify_fresh_onedrive_mount_isolation(&connection).await?;
    Ok(summary)
}

async fn run_onedriver_online_setup_with(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    cache_root: &Path,
    config_root: &Path,
) -> Result<String, String> {
    if !is_onedriver_online_mount(connection) {
        return Err(crate::fl!(
            "ui-app-onedriver-setup-only-applies-to-onedrive-online"
        ));
    }
    if runner.resolve(Executable::Onedriver).is_none() {
        return Err(crate::fl!(
            "ui-app-onedriver-is-missing-install-jstaf-onedriver-0"
        ));
    }
    prepare_onedriver_online_mount_runtime_with(connection, cache_root, config_root)?;
    let plan = onedriver_mount_plan(connection, cache_root, config_root).map_err(|error| {
        crate::fl!(
            "ui-app-could-not-build-onedriver-plan-error",
            error = (error).to_string()
        )
    })?;
    let mount_entries = mount_table.entries().map_err(|error| {
        crate::fl!(
            "ui-app-could-not-inspect-active-mounts-error",
            error = (error).to_string()
        )
    })?;
    if let Some(entry) = conflicting_onedriver_mount(&plan.mountpoint, &mount_entries) {
        return Err(crate::fl!(
            "ui-app-mountpoint-value1-overlaps-an-active-onedriver-mount",
            value1 = (plan.mountpoint.display()).to_string(),
            value2 = (entry.target.display()).to_string()
        ));
    }
    runner
        .run(onedriver_auth_request(&plan)?, CancellationToken::new())
        .await
        .map_err(onedriver_setup_command_error)?;
    verify_onedriver_online_mount_setup_with(
        connection,
        runner,
        mount_table,
        cache_root,
        config_root,
    )
}

async fn run_onedrive_mirror_interactive_setup_result(
    connection: Connection,
) -> Result<String, String> {
    let mounts = HostVisibleMountTable::observe().await?;
    let summary = run_onedrive_mirror_interactive_setup_with(
        &app_command_runner(),
        &mounts,
        &connection,
        &default_config_root(),
    )
    .await?;
    verify_fresh_onedrive_mount_isolation(&connection).await?;
    Ok(summary)
}

async fn run_onedrive_mirror_manual_setup_result(
    connection: Connection,
    auth_files: OneDriveMirrorAuthFiles,
) -> Result<String, String> {
    let mounts = HostVisibleMountTable::observe().await?;
    let summary = run_onedrive_mirror_manual_setup_with(
        &app_command_runner(),
        &mounts,
        &connection,
        &default_config_root(),
        &auth_files,
    )
    .await?;
    verify_fresh_onedrive_mount_isolation(&connection).await?;
    Ok(summary)
}

async fn verify_fresh_onedrive_mount_isolation(connection: &Connection) -> Result<(), String> {
    // Browser authentication can take minutes. Never certify its result using
    // only the mount observation taken before the authentication dialog.
    let entries = host_mount_entries().await?;
    match connection.mode {
        ConnectionMode::OnlineMount(_) => {
            let plan =
                onedriver_mount_plan(connection, &default_cache_root(), &default_config_root())
                    .map_err(|error| error.to_string())?;
            if let Some(entry) = conflicting_onedriver_mount(&plan.mountpoint, &entries) {
                return Err(fl!(
                    "ui-app-mountpoint-value1-overlaps-an-active-onedriver-mount",
                    value1 = plan.mountpoint.display().to_string(),
                    value2 = entry.target.display().to_string()
                ));
            }
        }
        ConnectionMode::OfflineMirror(_) => {
            one_drive_mirror_plan(
                connection,
                &default_config_root(),
                &OneDriveIsolationReport {
                    active_onedriver_paths: entries
                        .into_iter()
                        .filter(|entry| entry.filesystem == "fuse.onedriver")
                        .map(|entry| entry.target)
                        .collect(),
                },
            )
            .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

async fn run_onedrive_mirror_interactive_setup_with(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    config_root: &Path,
) -> Result<String, String> {
    let plan = prepare_onedrive_mirror_setup_plan(runner, mount_table, connection, config_root)?;
    let auth_result = runner
        .run(
            one_drive_auth_request(&plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| {
            if onedrive_auth_files_completion_error(&error)
                && plan.config_directory.join("refresh_token").exists()
            {
                None
            } else {
                Some(onedrive_validation_command_error(
                    &fl!("onedrive-stage-interactive-authentication"),
                    error,
                ))
            }
        });
    if let Err(Some(error)) = auth_result {
        return Err(crate::fl!(
            "ui-app-error-if-the-browser-did-not-return",
            error = (error).to_string()
        ));
    }
    let token_file = plan.config_directory.join("refresh_token");
    if token_file
        .metadata()
        .map(|metadata| metadata.len() == 0)
        .unwrap_or(true)
    {
        return Err(crate::fl!(
            "ui-app-onedrive-interactive-authorization-did-not-create-value1",
            value1 = (token_file.display()).to_string()
        ));
    }
    verify_onedrive_offline_mirror_setup_with(connection, runner, mount_table, config_root)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-onedrive-interactive-authorization-completed-but-post-auth",
                error = (error).to_string()
            )
        })
}

async fn run_onedrive_mirror_manual_setup_with(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    config_root: &Path,
    auth_files: &OneDriveMirrorAuthFiles,
) -> Result<String, String> {
    let plan = prepare_onedrive_mirror_setup_plan(runner, mount_table, connection, config_root)?;
    prepare_onedrive_auth_files(auth_files)?;
    let auth_result = runner
        .run(
            one_drive_auth_files_request(
                &plan,
                &auth_files.auth_url_file,
                &auth_files.response_url_file,
            )
            .map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| {
            if onedrive_auth_files_completion_error(&error)
                && plan.config_directory.join("refresh_token").exists()
            {
                None
            } else {
                Some(onedrive_validation_command_error(
                    &fl!("onedrive-stage-manual-authentication"),
                    error,
                ))
            }
        });
    cleanup_onedrive_auth_files(auth_files);
    if let Err(Some(error)) = auth_result {
        return Err(error);
    }
    verify_onedrive_offline_mirror_setup_with(connection, runner, mount_table, config_root)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-onedrive-manual-authorization-completed-but-post",
                error = (error).to_string()
            )
        })
}

fn prepare_onedrive_mirror_setup_plan(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    config_root: &Path,
) -> Result<cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan, String> {
    if !is_legacy_onedrive_mirror(connection) {
        return Err(crate::fl!(
            "ui-app-onedrive-mirror-setup-only-applies-to-onedrive"
        ));
    }
    if runner.resolve(Executable::OneDrive).is_none() {
        return Err(crate::fl!(
            "ui-app-onedrive-is-missing-install-abraunegg-onedrive-2"
        ));
    }
    prepare_onedrive_offline_mirror_runtime_with(connection, config_root)?;
    let active_onedriver_paths = mount_table
        .entries()
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-inspect-active-mounts-error",
                error = (error).to_string()
            )
        })?
        .into_iter()
        .filter(|entry| entry.filesystem == "fuse.onedriver")
        .map(|entry| entry.target)
        .collect();
    one_drive_mirror_plan(
        connection,
        config_root,
        &OneDriveIsolationReport {
            active_onedriver_paths,
        },
    )
    .map_err(onedrive_mirror_plan_validation_error)
}

async fn ensure_rclone_remote_name_available(
    runner: &dyn CommandRunner,
    remote_name: &str,
) -> Result<(), String> {
    if runner.resolve(Executable::Rclone).is_none() {
        return Err(crate::fl!("ui-app-rclone-is-missing-install-rclone-1-74-2"));
    }
    let dump = runner
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("config")
                .map_err(|error| error.to_string())?
                .arg("dump")
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(5)),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error(&crate::fl!("ui-app-config-dump"), error))?;
    if rclone_config_has_section(&dump.stdout.text, remote_name)? {
        return Err(crate::fl!(
            "ui-app-rclone-remote-remote-name-already-exists-choose",
            remote_name = (remote_name).to_string()
        ));
    }
    Ok(())
}

async fn rclone_remote_backend(
    runner: &dyn CommandRunner,
    remote_name: &str,
) -> Result<Option<String>, String> {
    if runner.resolve(Executable::Rclone).is_none() {
        return Err(crate::fl!("ui-app-rclone-is-missing-install-rclone-1-74-2"));
    }
    let dump = runner
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("config")
                .map_err(|error| error.to_string())?
                .arg("dump")
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(5)),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error(&crate::fl!("ui-app-config-dump"), error))?;
    rclone_config_section_backend(&dump.stdout.text, remote_name)
}

fn rclone_config_section_backend(
    output: &str,
    remote_name: &str,
) -> Result<Option<String>, String> {
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|error| crate::fl!("ui-app-invalid-json-error", error = (error).to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| crate::fl!("ui-app-rclone-config-dump-did-not-return-an"))?;
    let Some(config) = object.get(remote_name) else {
        return Ok(None);
    };
    Ok(Some(
        config
            .as_object()
            .and_then(|remote_config| remote_config.get("type"))
            .and_then(|value| value.as_str())
            .unwrap_or("<missing>")
            .to_owned(),
    ))
}

async fn cleanup_failed_rclone_remote_create(runner: &dyn CommandRunner, remote_name: &str) {
    if validate_rclone_remote_create_name(remote_name).is_err() {
        return;
    }
    let Ok(request) = rclone_config_delete_request(remote_name) else {
        return;
    };
    let _ = runner
        .run(
            request.with_timeout(Duration::from_secs(10)),
            CancellationToken::new(),
        )
        .await;
}

fn google_drive_rclone_config_create_request(
    setup: &GoogleDriveRemoteSetup,
) -> Result<CommandRequest, String> {
    let mut request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("create")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("drive")
        .map_err(|error| error.to_string())?;
    if let (Some(client_id), Some(client_secret)) = (&setup.client_id, &setup.client_secret) {
        request = request
            .arg("client_id")
            .map_err(|error| error.to_string())?
            .sensitive_arg(client_id)
            .map_err(|error| error.to_string())?
            .arg("client_secret")
            .map_err(|error| error.to_string())?
            .sensitive_arg(client_secret)
            .map_err(|error| error.to_string())?;
    }
    request
        .arg("scope")
        .map_err(|error| error.to_string())?
        .arg("drive")
        .map_err(|error| error.to_string())?
        .arg("config_is_local")
        .map_err(|error| error.to_string())?
        .arg("true")
        .map_err(|error| error.to_string())?
        .arg("--obscure")
        .map_err(|error| error.to_string())?
        .arg("--non-interactive")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(5 * 60)))
}

fn google_drive_rclone_config_update_request(
    setup: &GoogleDriveRemoteSetup,
) -> Result<CommandRequest, String> {
    let client_id = setup
        .client_id
        .as_ref()
        .ok_or_else(|| crate::fl!("ui-app-google-oauth-client-id-is-required-to"))?;
    let client_secret = setup
        .client_secret
        .as_ref()
        .ok_or_else(|| crate::fl!("ui-app-google-oauth-client-secret-is-required-to"))?;
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("update")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("client_id")
        .map_err(|error| error.to_string())?
        .sensitive_arg(client_id)
        .map_err(|error| error.to_string())?
        .arg("client_secret")
        .map_err(|error| error.to_string())?
        .sensitive_arg(client_secret)
        .map_err(|error| error.to_string())?
        .arg("config_is_local")
        .map_err(|error| error.to_string())?
        .arg("true")
        .map_err(|error| error.to_string())?
        .arg("--obscure")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(5 * 60)))
}

fn ensure_rclone_config_completed(stage: &str, output: &CommandOutput) -> Result<(), String> {
    let stdout = output.stdout.text.trim();
    if stdout.is_empty() {
        return Ok(());
    }
    let Ok(response) = serde_json::from_str::<serde_json::Value>(stdout) else {
        return Ok(());
    };
    let pending_state = response
        .get("State")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|state| !state.is_empty());
    let pending_option = response
        .get("Option")
        .is_some_and(|option| !option.is_null());
    if pending_state || pending_option {
        return Err(crate::fl!(
            "ui-app-rclone-stage-requires-another-configuration-response-and",
            stage = (stage).to_string()
        ));
    }
    Ok(())
}

fn box_rclone_config_create_request(setup: &BoxRemoteSetup) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("create")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("box")
        .map_err(|error| error.to_string())?
        .arg("config_is_local")
        .map_err(|error| error.to_string())?
        .arg("true")
        .map_err(|error| error.to_string())?
        .arg("--non-interactive")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(5 * 60)))
}

fn smb_rclone_config_create_request(setup: &SmbRemoteSetup) -> Result<CommandRequest, String> {
    let host = setup
        .host
        .as_ref()
        .ok_or_else(|| crate::fl!("ui-app-smb-host-is-required-to-create-a"))?;
    let mut request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("create")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("smb")
        .map_err(|error| error.to_string())?
        .arg("host")
        .map_err(|error| error.to_string())?
        .sensitive_arg(host)
        .map_err(|error| error.to_string())?;
    if let Some(user) = &setup.user {
        request = request
            .arg("user")
            .map_err(|error| error.to_string())?
            .sensitive_arg(user)
            .map_err(|error| error.to_string())?;
    }
    if let Some(domain) = &setup.domain {
        request = request
            .arg("domain")
            .map_err(|error| error.to_string())?
            .sensitive_arg(domain)
            .map_err(|error| error.to_string())?;
    }
    request
        .arg("--non-interactive")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(30)))
}

fn smb_rclone_config_update_request(setup: &SmbRemoteSetup) -> Result<CommandRequest, String> {
    let mut request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("update")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?;
    if let Some(host) = &setup.host {
        request = request
            .arg("host")
            .map_err(|error| error.to_string())?
            .sensitive_arg(host)
            .map_err(|error| error.to_string())?;
    }
    if let Some(user) = &setup.user {
        request = request
            .arg("user")
            .map_err(|error| error.to_string())?
            .sensitive_arg(user)
            .map_err(|error| error.to_string())?;
    }
    if let Some(domain) = &setup.domain {
        request = request
            .arg("domain")
            .map_err(|error| error.to_string())?
            .sensitive_arg(domain)
            .map_err(|error| error.to_string())?;
    }
    request
        .arg("--non-interactive")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(30)))
}

fn smb_rclone_config_password_request(setup: &SmbRemoteSetup) -> Result<CommandRequest, String> {
    let password = setup
        .password
        .as_ref()
        .ok_or_else(|| crate::fl!("ui-app-smb-password-is-not-set"))?;
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("password")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("pass")
        .map_err(|error| error.to_string())?
        .sensitive_arg(password)
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(30)))
}

fn rclone_config_delete_request(remote_name: &str) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("delete")
        .map_err(|error| error.to_string())?
        .arg(remote_name)
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(30)))
}

fn onedriver_auth_request(
    plan: &cosmic_ext_applet_mounter::providers::OnedriverMountPlan,
) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Onedriver)
        .arg("--auth-only")
        .map_err(|error| error.to_string())?
        .arg("--config-file")
        .map_err(|error| error.to_string())?
        .arg(plan.config_file.as_os_str())
        .map_err(|error| error.to_string())?
        .arg("--cache-dir")
        .map_err(|error| error.to_string())?
        .arg(plan.cache_directory.as_os_str())
        .map_err(|error| error.to_string())?
        .arg(plan.mountpoint.as_os_str())
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(5 * 60)))
}

fn validate_rclone_remote_create_name(value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        });
    if valid {
        Ok(())
    } else {
        Err(fl!("validation-rclone-name"))
    }
}

fn optional_setup_value(label: &str, value: &str) -> Result<Option<String>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        validate_setup_value(label, trimmed, false)?;
        Ok(Some(trimmed.to_owned()))
    }
}

fn optional_secret_value(label: &str, value: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        Ok(None)
    } else {
        validate_setup_value(label, value, false)?;
        Ok(Some(value.to_owned()))
    }
}

fn validate_setup_value(label: &str, value: &str, required: bool) -> Result<(), String> {
    if required && value.is_empty() {
        return Err(fl!("validation-field-required", field = label));
    }
    if value
        .chars()
        .any(|character| character == '\0' || character == '\n' || character == '\r')
    {
        return Err(fl!("validation-control-characters", field = label));
    }
    Ok(())
}

fn rclone_setup_command_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            crate::fl!(
                "ui-app-value1-is-missing",
                value1 = (executable.display_name()).to_string()
            )
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-app-rclone-setup-command-argument-contains-unsupported-characters")
        }
        CommandError::Timeout { timeout, .. } => {
            crate::fl!(
                "ui-app-rclone-stage-timed-out-after-value2-seconds",
                stage = (stage).to_string(),
                value2 = (timeout.as_secs()).to_string()
            )
        }
        CommandError::Cancelled { .. } => crate::fl!(
            "ui-app-rclone-stage-was-cancelled",
            stage = (stage).to_string()
        ),
        CommandError::Spawn { message, .. } => crate::fl!(
            "ui-app-could-not-start-rclone-message",
            message = (message).to_string()
        ),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            if detail.is_empty() {
                crate::fl!(
                    "ui-app-rclone-stage-failed-without-diagnostic-output",
                    stage = (stage).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-rclone-stage-failed-detail",
                    stage = (stage).to_string(),
                    detail = (detail).to_string()
                )
            }
        }
    }
}

fn onedriver_setup_command_error(error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            crate::fl!(
                "ui-app-value1-is-missing",
                value1 = (executable.display_name()).to_string()
            )
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-app-onedriver-setup-command-argument-contains-unsupported-characters")
        }
        CommandError::Timeout { timeout, .. } => {
            crate::fl!(
                "ui-app-onedriver-authentication-timed-out-after-value1-seconds",
                value1 = (timeout.as_secs()).to_string()
            )
        }
        CommandError::Cancelled { .. } => {
            crate::fl!("ui-app-onedriver-authentication-was-cancelled")
        }
        CommandError::Spawn { message, .. } => crate::fl!(
            "ui-app-could-not-start-onedriver-message",
            message = (message).to_string()
        ),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            if detail.is_empty() {
                crate::fl!("ui-app-onedriver-authentication-failed-without-diagnostic-output")
            } else {
                crate::fl!(
                    "ui-app-onedriver-authentication-failed-detail",
                    detail = (detail).to_string()
                )
            }
        }
    }
}

fn toggle_button(
    label: impl Into<Cow<'static, str>> + 'static,
    current: bool,
    message: fn(bool) -> Message,
) -> Element<'static, Message> {
    widget::Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text::body(label))
        .push(widget::toggler(current).on_toggle(message))
        .into()
}

fn managed_plan_summary(connection: &Connection) -> Result<String, String> {
    match &connection.mode {
        ConnectionMode::OnlineMount(_) => {
            let document = match connection.provider {
                Provider::OneDrive => {
                    let plan = onedriver_mount_plan(
                        connection,
                        &default_cache_root(),
                        &default_config_root(),
                    )
                    .map_err(|error| error.to_string())?;
                    UnitDocument::service(&plan.service).map_err(|error| error.to_string())?
                }
                Provider::Teams
                | Provider::GoogleDrive
                | Provider::Box
                | Provider::Smb
                | Provider::Sftp => {
                    let plan = rclone_mount_plan(
                        connection,
                        &default_runtime_root(),
                        &default_cache_root(),
                    )
                    .map_err(|error| error.to_string())?;
                    UnitDocument::service(&plan.service).map_err(|error| error.to_string())?
                }
            };
            Ok(crate::fl!(
                "ui-app-managed-online-mount-unit-value1-validates-structurally",
                value1 = (document.name.file_name()).to_string()
            ))
        }
        ConnectionMode::OfflineMirror(_) => match connection.provider {
            Provider::OneDrive if is_legacy_onedrive_mirror(connection) => {
                let plan = one_drive_mirror_plan(
                    connection,
                    &default_config_root(),
                    &OneDriveIsolationReport {
                        active_onedriver_paths: Vec::new(),
                    },
                )
                .map_err(|error| error.to_string())?;
                let document =
                    UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
                Ok(crate::fl!(
                    "ui-app-managed-onedrive-mirror-unit-value1-validates-structurally",
                    value1 = (document.name.file_name()).to_string()
                ))
            }
            Provider::OneDrive
            | Provider::Teams
            | Provider::GoogleDrive
            | Provider::Box
            | Provider::Smb
            | Provider::Sftp => {
                let plan = rclone_bisync_plan(connection, &default_work_root())
                    .map_err(|error| error.to_string())?;
                let service = UnitDocument::service(&mirror_script::service_spec(&plan)?)
                    .map_err(|error| error.to_string())?;
                let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
                if connection.provider == Provider::Teams {
                    return Ok(fl!(
                        "sharepoint-mirror-plan-ready",
                        name = service.name.file_name()
                    ));
                }
                Ok(crate::fl!(
                    "ui-app-managed-bisync-service-value1-and-timer-value2",
                    value1 = (service.name.file_name()).to_string(),
                    value2 = (timer.name.file_name()).to_string()
                ))
            }
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnlineMountEngine {
    Rclone,
    Onedriver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OnlineRepairPlan {
    engine: OnlineMountEngine,
    cancel_rclone_refresh: bool,
    cancel_directory_preload: bool,
}

impl OnlineRepairPlan {
    const fn expected_filesystem(self) -> &'static str {
        match self.engine {
            OnlineMountEngine::Rclone => "fuse.rclone",
            OnlineMountEngine::Onedriver => "fuse.onedriver",
        }
    }
}

const fn online_mount_engine(connection: &Connection) -> Option<OnlineMountEngine> {
    match (&connection.provider, &connection.mode) {
        (Provider::OneDrive, ConnectionMode::OnlineMount(_)) => Some(OnlineMountEngine::Onedriver),
        (
            Provider::Teams
            | Provider::GoogleDrive
            | Provider::Box
            | Provider::Smb
            | Provider::Sftp,
            ConnectionMode::OnlineMount(_),
        ) => Some(OnlineMountEngine::Rclone),
        (_, ConnectionMode::OfflineMirror(_)) => None,
    }
}

const fn online_repair_plan(connection: &Connection) -> Option<OnlineRepairPlan> {
    let Some(engine) = online_mount_engine(connection) else {
        return None;
    };
    Some(OnlineRepairPlan {
        engine,
        cancel_rclone_refresh: matches!(connection.provider, Provider::GoogleDrive),
        cancel_directory_preload: uses_directory_preload(connection),
    })
}

const fn is_rclone_online_mount(connection: &Connection) -> bool {
    matches!(
        online_mount_engine(connection),
        Some(OnlineMountEngine::Rclone)
    )
}

fn is_rclone_offline_mirror(connection: &Connection) -> bool {
    connection.offline_engine() == Some(OfflineEngine::Rclone)
}

fn is_onedrive_rclone_mirror(connection: &Connection) -> bool {
    connection.provider == Provider::OneDrive && is_rclone_offline_mirror(connection)
}

const fn is_onedriver_online_mount(connection: &Connection) -> bool {
    matches!(
        online_mount_engine(connection),
        Some(OnlineMountEngine::Onedriver)
    )
}

const fn uses_directory_preload(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::OneDrive | Provider::Teams | Provider::Box | Provider::Smb | Provider::Sftp,
            ConnectionMode::OnlineMount(_)
        )
    )
}

const fn directory_preload_waits_for_root(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::Teams | Provider::Box | Provider::Smb | Provider::Sftp,
            ConnectionMode::OnlineMount(_)
        )
    )
}

fn is_legacy_onedrive_mirror(connection: &Connection) -> bool {
    connection.provider == Provider::OneDrive
        && connection.offline_engine() == Some(OfflineEngine::LegacyOnedrive)
}

async fn install_rclone_online_mount_unit(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_rclone_online_mount_unit_result(&connection).await {
        Ok(enabled) => {
            if enabled {
                crate::fl!(
                    "ui-app-name-saved-and-managed-mount-unit-installed",
                    name = (name).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-name-saved-and-managed-mount-unit-installed-2",
                    name = (name).to_string()
                )
            }
        }
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-saved-but-managed-mount-unit",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

async fn install_rclone_online_mount_unit_result(connection: &Connection) -> Result<bool, String> {
    prepare_online_mount_runtime(connection)?;
    let plan = rclone_mount_plan(connection, &default_runtime_root(), &default_cache_root())
        .map_err(|error| error.to_string())?;
    let document = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;

    let start_at_login = match &connection.mode {
        ConnectionMode::OnlineMount(options) => options.start_at_login,
        ConnectionMode::OfflineMirror(_) => false,
    };
    let manager = DbusSystemdManager;
    let action = if start_at_login {
        SystemdAction::Enable
    } else {
        SystemdAction::Disable
    };
    manager
        .action(action, Some(&document.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(start_at_login)
}

async fn refresh_online_mount_guards(connections: Vec<Connection>) -> Vec<String> {
    let mut failures = Vec::new();
    for connection in connections {
        if !matches!(connection.mode, ConnectionMode::OnlineMount(_)) {
            continue;
        }
        let result = match online_mount_engine(&connection) {
            Some(OnlineMountEngine::Onedriver) => {
                install_onedriver_online_mount_unit_result(&connection).await
            }
            Some(OnlineMountEngine::Rclone) => {
                install_rclone_online_mount_unit_result(&connection).await
            }
            None => continue,
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error}", connection.name));
        }
    }
    failures
}

async fn install_rclone_offline_mirror_units(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_rclone_offline_mirror_units_result(&connection).await {
        Ok(()) => offline_mirror_saved_notice(
            &connection,
            rclone_bisync_plan(&connection, &default_work_root())
                .is_ok_and(|plan| initial_sync_marker(&plan).is_file()),
        ),
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-saved-but-managed-mirror-unit",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

async fn install_rclone_offline_mirror_units_result(connection: &Connection) -> Result<(), String> {
    prepare_offline_mirror_runtime(connection).await?;
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
    prepare_rclone_bisync_work_files(connection, &plan)?;
    let service = UnitDocument::service(&mirror_script::service_spec(&plan)?)
        .map_err(|error| error.to_string())?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    install_paused_rclone_units(store, manager, &service, &timer, CancellationToken::new()).await
}

async fn install_paused_rclone_units<S: UnitStore + Clone, M: SystemdManager + Clone>(
    store: S,
    manager: M,
    service: &UnitDocument,
    timer: &UnitDocument,
    cancellation: CancellationToken,
) -> Result<(), String> {
    for document in [service, timer] {
        match store
            .state(&document.name)
            .map_err(|error| error.to_string())?
        {
            UnitFileState::External => return Err(fl!("onedrive-runtime-unit-not-owned")),
            UnitFileState::Managed(owner) if owner != document.name.connection_id() => {
                return Err(fl!("onedrive-runtime-unit-not-owned"));
            }
            _ => {}
        }
    }
    let previous_service = store
        .read(&service.name)
        .map_err(|error| error.to_string())?;
    // Stop the existing owned trigger before replacing either unit. A new timer
    // is also stopped/disabled after install; Save never starts a background job.
    if !matches!(
        store
            .state(&timer.name)
            .map_err(|error| error.to_string())?,
        UnitFileState::Missing
    ) {
        let repair = OfflineRepairPlan {
            engine: OfflineRepairEngine::RcloneBisync,
            service: service.name.clone(),
            timer: Some(timer.name.clone()),
        };
        pause_offline_repair_trigger(&manager, &repair, cancellation.child_token()).await?;
    }
    let controller = UnitController::new(store, manager.clone());
    controller
        .install(service, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;
    if let Err(error) = controller.install(timer, cancellation.child_token()).await {
        let rollback = if let Some(content) = previous_service {
            controller
                .install(
                    &UnitDocument {
                        name: service.name.clone(),
                        content,
                    },
                    CancellationToken::new(),
                )
                .await
        } else {
            controller
                .remove(&service.name, CancellationToken::new())
                .await
        };
        return Err(if rollback.is_err() {
            fl!("onedrive-runtime-install-rollback-failed")
        } else {
            error.to_string()
        });
    }
    manager
        .action(
            SystemdAction::Disable,
            Some(&service.name),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let repair = OfflineRepairPlan {
        engine: OfflineRepairEngine::RcloneBisync,
        service: service.name.clone(),
        timer: Some(timer.name.clone()),
    };
    pause_offline_repair_trigger(&manager, &repair, cancellation).await?;
    Ok(())
}

async fn install_onedriver_online_mount_unit(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_onedriver_online_mount_unit_result(&connection).await {
        Ok(enabled) => {
            if enabled {
                crate::fl!(
                    "ui-app-name-saved-and-managed-onedriver-mount-unit",
                    name = (name).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-name-saved-and-managed-onedriver-mount-unit-2",
                    name = (name).to_string()
                )
            }
        }
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-saved-but-managed-onedriver-unit",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

async fn install_onedriver_online_mount_unit_result(
    connection: &Connection,
) -> Result<bool, String> {
    prepare_onedriver_online_mount_runtime(connection)?;
    let plan = onedriver_mount_plan(connection, &default_cache_root(), &default_config_root())
        .map_err(|error| error.to_string())?;
    let document = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;

    let start_at_login = match &connection.mode {
        ConnectionMode::OnlineMount(options) => options.start_at_login,
        ConnectionMode::OfflineMirror(_) => false,
    };
    let manager = DbusSystemdManager;
    let action = if start_at_login {
        SystemdAction::Enable
    } else {
        SystemdAction::Disable
    };
    manager
        .action(action, Some(&document.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(start_at_login)
}

async fn install_onedrive_offline_mirror_unit(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_onedrive_offline_mirror_unit_result(&connection).await {
        Ok(()) => offline_mirror_saved_notice(
            &connection,
            onedrive_mirror_plan_for_app(&connection)
                .is_ok_and(|plan| onedrive_initial_sync_marker(&plan).is_file()),
        ),
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-saved-but-managed-onedrive-mirror",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

fn offline_mirror_saved_notice(connection: &Connection, initialized: bool) -> String {
    match (is_legacy_onedrive_mirror(connection), initialized) {
        (true, true) => fl!(
            "onedrive-mirror-saved-initialized",
            name = connection.name.as_str()
        ),
        (true, false) => fl!(
            "onedrive-mirror-saved-uninitialized",
            name = connection.name.as_str()
        ),
        (_, true) => fl!(
            "rclone-mirror-saved-initialized",
            name = connection.name.as_str()
        ),
        (_, false) => fl!(
            "rclone-mirror-saved-uninitialized",
            name = connection.name.as_str()
        ),
    }
}

async fn install_onedrive_offline_mirror_unit_result(
    connection: &Connection,
) -> Result<(), String> {
    prepare_onedrive_offline_mirror_runtime(connection)?;
    let plan = onedrive_mirror_plan_for_app(connection)?;
    let document = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    manager
        .action(SystemdAction::Disable, Some(&document.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn install_import_replacement_unit(plan: ImportReplacementPlan) -> String {
    let name = plan.preview.connection.name.clone();
    match install_import_replacement_unit_result(&plan).await {
        Ok(enabled) => {
            let login = if enabled {
                fl!("value-enabled")
            } else {
                fl!("value-disabled")
            };
            let original = if plan.preserve_original {
                fl!("legacy-service-preserved")
            } else {
                String::new()
            };
            crate::fl!(
                "ui-app-name-imported-and-applet-managed-replacement-unit",
                name = (name).to_string(),
                login = (login).to_string(),
                original = (original).to_string()
            )
        }
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-imported-into-applet-configuration-but",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

async fn install_import_replacement_unit_result(
    plan: &ImportReplacementPlan,
) -> Result<bool, String> {
    let connection = &plan.preview.connection;
    match connection.provider {
        Provider::OneDrive => prepare_onedriver_online_mount_runtime(connection)?,
        Provider::Teams
        | Provider::GoogleDrive
        | Provider::Box
        | Provider::Smb
        | Provider::Sftp => {
            prepare_online_mount_runtime(connection)?;
        }
    }
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&plan.managed_service, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;

    let start_at_login = match &connection.mode {
        ConnectionMode::OnlineMount(options) => options.start_at_login,
        ConnectionMode::OfflineMirror(_) => false,
    };
    let manager = DbusSystemdManager;
    let action = if start_at_login {
        SystemdAction::Enable
    } else {
        SystemdAction::Disable
    };
    manager
        .action(action, Some(&plan.managed_service.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(start_at_login)
}

async fn remove_generated_units_for_connection(connection: Connection) -> String {
    let name = connection.name.clone();
    match remove_generated_units_for_connection_result(&connection).await {
        Ok(removed) => {
            if removed == 0 {
                crate::fl!(
                    "ui-app-name-was-removed-no-applet-owned-generated",
                    name = (name).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-name-was-removed-removed-removed-applet-owned",
                    name = (name).to_string(),
                    removed = (removed).to_string()
                )
            }
        }
        Err(error) => {
            crate::fl!(
                "ui-app-name-was-removed-from-applet-configuration-but",
                name = (name).to_string(),
                error = (error).to_string()
            )
        }
    }
}

async fn remove_generated_units_for_connection_result(
    connection: &Connection,
) -> Result<usize, String> {
    if connection.provider == Provider::GoogleDrive
        && matches!(connection.mode, ConnectionMode::OnlineMount(_))
    {
        let _ = rclone_refresh::cancel(connection.id).await;
    }
    if uses_directory_preload(connection) {
        directory_preload::cancel(connection.id).await;
    }
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let controller = UnitController::new(store, DbusSystemdManager);
    let cancellation = CancellationToken::new();
    let mut removed = 0usize;
    for unit in managed_unit_names_for_connection(connection) {
        let manager = DbusSystemdManager;
        let _ = manager
            .action(SystemdAction::Stop, Some(&unit), cancellation.child_token())
            .await;
        let manager = DbusSystemdManager;
        let _ = manager
            .action(
                SystemdAction::Disable,
                Some(&unit),
                cancellation.child_token(),
            )
            .await;
        controller
            .remove(&unit, cancellation.child_token())
            .await
            .map_err(|error| error.to_string())?;
        removed += 1;
    }
    Ok(removed)
}

fn managed_unit_names_for_connection(connection: &Connection) -> Vec<UnitName> {
    let service = UnitName::new(connection.id, UnitKind::Service);
    if is_rclone_offline_mirror(connection) {
        vec![UnitName::new(connection.id, UnitKind::Timer), service]
    } else {
        vec![service]
    }
}

async fn restore_online_after_wake_result(connection: &Connection) -> Result<(), String> {
    let Ok(token) = cosmic_ext_applet_mounter::sleep::online_operation_token() else {
        return Err(crate::fl!(
            "ui-app-wake-restoration-canceled-for-value1",
            value1 = (connection.name).to_string()
        ));
    };
    let wait = async {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        loop {
            if current_network_ready().await {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(crate::fl!(
                    "ui-app-wake-restoration-for-value1-is-waiting-for",
                    value1 = (connection.name).to_string()
                ));
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        // Re-read after waiting: a removal or disable must override restoration.
        let config = Config::load_runtime().config;
        if !config.document.unmount_before_sleep || !config.document.restore_after_wake {
            return Err(crate::fl!(
                "ui-app-wake-restoration-disabled-for-value1",
                value1 = (connection.name).to_string()
            ));
        }
        let Some(current) = config.document.connections.into_iter().find(|item| {
            item.id == connection.id
                && item.enabled
                && matches!(item.mode, ConnectionMode::OnlineMount(_))
        }) else {
            return Err(crate::fl!(
                "ui-app-wake-restoration-skipped-for-value1",
                value1 = (connection.name).to_string()
            ));
        };
        run_managed_online_mount_operation_result(&current, Operation::Mount).await
    };
    tokio::select! {
        biased;
        () = token.cancelled() => Err(fl!("wake-restoration-cancelled", name = connection.name.as_str())),
        result = wait => result,
    }
}

async fn run_managed_online_mount_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    match online_mount_engine(connection).ok_or_else(|| {
        crate::fl!(
            "ui-app-value1-is-not-an-online-mount",
            value1 = (connection.name).to_string()
        )
    })? {
        OnlineMountEngine::Rclone => {
            run_managed_rclone_online_mount_operation_result(connection, operation).await
        }
        OnlineMountEngine::Onedriver => {
            run_managed_onedriver_online_mount_operation_result(connection, operation).await
        }
    }
}

async fn run_managed_rclone_online_mount_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let token = cosmic_ext_applet_mounter::sleep::begin_online_operation()?;
    tokio::select! {
        biased;
        () = token.cancelled() => Err(fl!("online-operation-cancelled-for-sleep")),
        result = run_managed_rclone_online_mount_operation_result_inner(connection, operation) => result,
    }
}

async fn run_managed_rclone_online_mount_operation_result_inner(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let action = match operation {
        Operation::Mount => SystemdAction::Start,
        Operation::Unmount => SystemdAction::Stop,
        _ => {
            return Err(crate::fl!(
                "ui-app-value1-is-not-a-managed-mount-operation",
                value1 = (operation_label(operation)).to_string()
            ));
        }
    };
    let unit = UnitName::new(connection.id, UnitKind::Service);
    if action == SystemdAction::Start {
        prepare_online_mount_runtime(connection)?;
        mount_guard::check_empty_mountpoint(&app_command_runner(), &connection.local_path).await?;
        ensure_vpn_ready_for_connection(connection).await?;
        verify_rclone_access(connection).await.map_err(|error| {
            crate::fl!(
                "ui-app-remote-access-check-failed-before-mounting-error",
                error = (error).to_string()
            )
        })?;
        let current = Config::load_runtime().config;
        if !current
            .document
            .connections
            .iter()
            .any(|saved| saved == connection && saved.enabled)
        {
            return Err(crate::fl!(
                "ui-app-connection-was-disabled-removed-or-changed-while"
            ));
        }
        // Refresh applet-owned units so provider tuning added by an app update
        // applies to saved connections without requiring an edit-and-save cycle.
        install_rclone_online_mount_unit_result(connection)
            .await
            .map_err(|error| fl!("online-service-refresh-failed", error = error))?;
    } else {
        persist_online_popup_state(connection.id, false).await?;
    }
    let cancellation = CancellationToken::new();
    if action == SystemdAction::Start {
        return start_online_service_and_persist(connection, &unit, cancellation).await;
    } else {
        if connection.provider == Provider::GoogleDrive {
            // Stop cache warm-up while its RC socket is still available. A failed
            // job/stop does not block service stop, which also ends the job.
            let _ = rclone_refresh::cancel(connection.id).await;
        }
        if uses_directory_preload(connection) {
            directory_preload::cancel_before_unmount(connection.id).await?;
        }
        clean_detach_before_service_stop(connection, cancellation.child_token()).await?;
    }
    stop_online_service_and_release_vpn(connection, &unit, cancellation).await
}

async fn run_managed_onedriver_online_mount_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let token = cosmic_ext_applet_mounter::sleep::begin_online_operation()?;
    tokio::select! {
        biased;
        () = token.cancelled() => Err(fl!("online-operation-cancelled-for-sleep")),
        result = run_managed_onedriver_online_mount_operation_result_inner(connection, operation) => result,
    }
}

async fn run_managed_onedriver_online_mount_operation_result_inner(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let action = match operation {
        Operation::Mount => SystemdAction::Start,
        Operation::Unmount => SystemdAction::Stop,
        _ => {
            return Err(crate::fl!(
                "ui-app-value1-is-not-a-managed-onedriver-mount",
                value1 = (operation_label(operation)).to_string()
            ));
        }
    };
    let unit = UnitName::new(connection.id, UnitKind::Service);
    if action == SystemdAction::Start {
        prepare_onedriver_online_mount_runtime(connection)?;
        mount_guard::check_empty_mountpoint(&app_command_runner(), &connection.local_path).await?;
        ensure_vpn_ready_for_connection(connection).await?;
        let current = Config::load_runtime().config;
        if !current
            .document
            .connections
            .iter()
            .any(|saved| saved == connection && saved.enabled)
        {
            return Err(crate::fl!(
                "ui-app-connection-was-disabled-removed-or-changed-while"
            ));
        }
        install_onedriver_online_mount_unit_result(connection)
            .await
            .map_err(|error| fl!("online-service-refresh-failed", error = error))?;
    } else {
        persist_online_popup_state(connection.id, false).await?;
    }
    let manager = DbusSystemdManager;
    let cancellation = CancellationToken::new();
    if action == SystemdAction::Start {
        manager
            .action(
                SystemdAction::DaemonReload,
                None,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| error.to_string())?;
        return start_online_service_and_persist(connection, &unit, cancellation).await;
    } else {
        directory_preload::cancel_before_unmount(connection.id).await?;
        clean_detach_before_service_stop(connection, cancellation.child_token()).await?;
    }
    stop_online_service_and_release_vpn(connection, &unit, cancellation).await
}

async fn start_online_service_and_persist(
    connection: &Connection,
    unit: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    start_managed_online_service(&DbusSystemdManager, unit, cancellation).await?;
    persist_online_popup_state(connection.id, true)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-mount-is-active-for-this-session-but",
                error = (error).to_string()
            )
        })
}

async fn stop_online_service_and_release_vpn(
    connection: &Connection,
    unit: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    DbusSystemdManager
        .action(SystemdAction::Stop, Some(unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    require_mount_disappearance_after_clean_stop(connection).await?;
    maybe_shutdown_vpn_after_unmount(connection).await
}

async fn clean_detach_before_service_stop(
    connection: &Connection,
    cancellation: CancellationToken,
) -> Result<(), String> {
    // Keep the service alive when a holder still has the FUSE mount open.
    // A preload child can release its handle just after it exits, so briefly
    // retry a busy clean detach before reporting failure.
    let deadline = tokio::time::Instant::now() + CLEAN_UNMOUNT_SETTLE_TIMEOUT;
    loop {
        let mounted = host_mount_entries()
            .await
            .map_err(|error| {
                crate::fl!(
                    "ui-app-could-not-check-mount-before-unmount-error",
                    error = (error).to_string()
                )
            })?
            .iter()
            .any(|entry| entry.target == connection.local_path);
        if !mounted {
            return Ok(());
        }
        let result = app_command_runner()
            .run(
                clean_unmount_request(&connection.local_path).map_err(|error| error.to_string())?,
                cancellation.child_token(),
            )
            .await;
        if result.is_ok() {
            return Ok(());
        }
        let error = result.expect_err("checked unsuccessful clean unmount");
        // Reconcile the helper exit with the actual mount table. A helper can
        // report failure after another process has already detached the mount.
        let still_mounted = host_mount_entries()
            .await
            .map_err(|check_error| {
                crate::fl!(
                    "ui-app-clean-unmount-failed-for-value1-error-could",
                    value1 = (connection.local_path.display()).to_string(),
                    error = (error).to_string(),
                    check_error = (check_error).to_string()
                )
            })?
            .iter()
            .any(|entry| entry.target == connection.local_path);
        if !still_mounted {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline || !matches!(error, CommandError::NonZero { .. })
        {
            return Err(crate::fl!(
                "ui-app-clean-unmount-could-not-detach-value1-error",
                value1 = (connection.local_path.display()).to_string(),
                error = (error).to_string()
            ));
        }
        tokio::time::sleep(CLEAN_UNMOUNT_POLL_INTERVAL).await;
    }
}

async fn require_mount_disappearance_after_clean_stop(
    connection: &Connection,
) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + CLEAN_UNMOUNT_SETTLE_TIMEOUT;
    loop {
        let mounted = host_mount_entries()
            .await
            .map_err(|error| {
                crate::fl!(
                    "ui-app-could-not-verify-unmount-error",
                    error = (error).to_string()
                )
            })?
            .iter()
            .any(|entry| entry.target == connection.local_path);
        if !mounted {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(crate::fl!(
                "ui-app-clean-unmount-left-a-busy-fuse-endpoint",
                value1 = (connection.local_path.display()).to_string()
            ));
        }
        tokio::time::sleep(CLEAN_UNMOUNT_POLL_INTERVAL).await;
    }
}

async fn run_online_mount_repair_operation(connection: Connection) -> String {
    match run_online_mount_repair_operation_result(&connection).await {
        Ok(()) => crate::fl!(
            "ui-app-repair-completed-for-value1-the-mountpoint-is",
            value1 = (connection.name).to_string()
        ),
        Err(error) => crate::fl!(
            "ui-app-repair-failed-for-value1-error",
            value1 = (connection.name).to_string(),
            error = (error).to_string()
        ),
    }
}

async fn run_online_mount_repair_operation_result(connection: &Connection) -> Result<(), String> {
    let plan = online_repair_plan(connection)
        .ok_or_else(|| crate::fl!("ui-app-repair-is-only-available-for-online-mounts"))?;

    let unit = UnitName::new(connection.id, UnitKind::Service);
    let manager = DbusSystemdManager;
    let cancellation = CancellationToken::new();

    persist_online_popup_state(connection.id, false).await?;

    if plan.cancel_rclone_refresh {
        let _ = rclone_refresh::cancel(connection.id).await;
    }
    if plan.cancel_directory_preload {
        directory_preload::cancel(connection.id).await;
    }
    stop_online_repair_service(&manager, &unit, cancellation.child_token()).await;

    if repair_needs_lazy_detach(connection, &host_mount_entries().await?)? {
        app_command_runner()
            .run(
                lazy_unmount_request(&connection.local_path).map_err(|error| error.to_string())?,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| error.to_string())?;
    }

    let deadline = tokio::time::Instant::now() + CLEAN_UNMOUNT_SETTLE_TIMEOUT;
    while repair_needs_lazy_detach(connection, &host_mount_entries().await?)? {
        if tokio::time::Instant::now() >= deadline {
            return Err(crate::fl!(
                "ui-app-repair-could-not-detach-the-mountpoint-inspect"
            ));
        }
        tokio::time::sleep(CLEAN_UNMOUNT_POLL_INTERVAL).await;
    }

    reconcile_online_repair_service(&manager, &unit, cancellation).await
}

async fn stop_online_repair_service<M: SystemdManager>(
    manager: &M,
    unit: &UnitName,
    cancellation: CancellationToken,
) {
    // Continue to ownership-checked detach even when a failed unit cannot be
    // stopped; its final state is read and reset after the mount disappears.
    let _ = manager
        .action(SystemdAction::Stop, Some(unit), cancellation)
        .await;
}

async fn reconcile_online_repair_service<M: SystemdManager>(
    manager: &M,
    unit: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let status = manager
        .action(
            SystemdAction::Status,
            Some(unit),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| {
            fl!(
                "online-service-repair-read-failed",
                error = error.to_string()
            )
        })?
        .ok_or_else(|| fl!("online-service-repair-unknown-state"))?;
    if repair_requires_failed_reset(Some(&status)) {
        manager
            .action(SystemdAction::ResetFailed, Some(unit), cancellation)
            .await
            .map_err(|error| {
                fl!(
                    "online-service-repair-reset-failed",
                    error = error.to_string()
                )
            })?;
    }

    Ok(())
}

async fn host_mount_entries() -> Result<Vec<MountEntry>, String> {
    let text = HostInspection::HostMountInfo.run_owned().await?;
    text.lines()
        .map(|line| {
            cosmic_ext_applet_mounter::mounts::parse_mountinfo_line(line)
                .map_err(|error| error.to_string())
        })
        .collect()
}

fn repair_needs_lazy_detach(
    connection: &Connection,
    mounts: &[cosmic_ext_applet_mounter::mounts::MountEntry],
) -> Result<bool, String> {
    let Some(mount) = mounts
        .iter()
        .find(|entry| entry.target == connection.local_path)
    else {
        return Ok(false);
    };
    let plan = online_repair_plan(connection)
        .ok_or_else(|| crate::fl!("ui-app-repair-is-only-available-for-online-mounts"))?;
    if mount.filesystem != plan.expected_filesystem() {
        return Err(crate::fl!(
            "ui-app-repair-found-an-unexpected-filesystem-at-the"
        ));
    }
    Ok(true)
}

fn repair_requires_failed_reset(status: Option<&UnitStatus>) -> bool {
    status.is_some_and(|status| status.active == ActiveState::Failed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OfflineRepairEngine {
    RcloneBisync,
    OneDriveMonitor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OfflineRepairPlan {
    engine: OfflineRepairEngine,
    service: UnitName,
    timer: Option<UnitName>,
}

fn offline_repair_plan(connection: &Connection) -> Result<OfflineRepairPlan, String> {
    let engine = if is_rclone_offline_mirror(connection) {
        OfflineRepairEngine::RcloneBisync
    } else if is_legacy_onedrive_mirror(connection) {
        OfflineRepairEngine::OneDriveMonitor
    } else {
        return Err(crate::fl!(
            "ui-app-repair-is-only-available-for-supported-offline"
        ));
    };
    Ok(OfflineRepairPlan {
        engine,
        service: UnitName::new(connection.id, UnitKind::Service),
        timer: (engine == OfflineRepairEngine::RcloneBisync)
            .then(|| UnitName::new(connection.id, UnitKind::Timer)),
    })
}

fn authoritative_offline_repair_connection(id: ConnectionId) -> Result<Connection, String> {
    let report = Config::load_runtime();
    let connection = report
        .config
        .document
        .connections
        .into_iter()
        .find(|connection| connection.id == id)
        .ok_or_else(|| crate::fl!("ui-app-offline-mirror-connection-is-no-longer-available"))?;
    offline_repair_plan(&connection)?;
    Ok(connection)
}

fn verify_offline_repair_unit_ownership(plan: &OfflineRepairPlan) -> Result<(), String> {
    let validator = StructuralUnitValidator;
    let store = FileUnitStore::user(Arc::new(validator)).map_err(|error| error.to_string())?;
    for unit in std::iter::once(&plan.service).chain(plan.timer.iter()) {
        match store.state(unit).map_err(|error| error.to_string())? {
            UnitFileState::Managed(owner) if owner == unit.connection_id() => {
                let content = store
                    .read(unit)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| {
                        crate::fl!(
                            "ui-app-managed-offline-mirror-unit-value1-is-missing",
                            value1 = (unit.file_name()).to_string()
                        )
                    })?;
                validator
                    .validate(&UnitDocument {
                        name: unit.clone(),
                        content,
                    })
                    .map_err(|error| {
                        crate::fl!(
                            "ui-app-managed-offline-mirror-unit-value1-is-invalid",
                            value1 = (unit.file_name()).to_string(),
                            error = (error).to_string()
                        )
                    })?;
            }
            UnitFileState::Missing => {
                return Err(crate::fl!(
                    "ui-app-managed-offline-mirror-unit-value1-is-missing",
                    value1 = (unit.file_name()).to_string()
                ));
            }
            UnitFileState::External | UnitFileState::Managed(_) => {
                return Err(crate::fl!(
                    "ui-app-offline-mirror-unit-value1-is-not-owned",
                    value1 = (unit.file_name()).to_string()
                ));
            }
        }
    }
    Ok(())
}

async fn offline_repair_action<M: SystemdManager>(
    manager: &M,
    action: SystemdAction,
    unit: &UnitName,
    stage: &str,
    cancellation: CancellationToken,
) -> Result<Option<UnitStatus>, String> {
    manager
        .action(action, Some(unit), cancellation)
        .await
        .map_err(|error| format!("{stage}: {error}"))
}

async fn offline_repair_status<M: SystemdManager>(
    manager: &M,
    unit: &UnitName,
    stage: &str,
    cancellation: CancellationToken,
) -> Result<UnitStatus, String> {
    offline_repair_action(manager, SystemdAction::Status, unit, stage, cancellation)
        .await?
        .ok_or_else(|| {
            crate::fl!(
                "ui-app-stage-unit-state-is-unavailable",
                stage = (stage).to_string()
            )
        })
}

fn require_repaired_unit(
    status: &UnitStatus,
    role: &str,
    must_be_disabled: bool,
) -> Result<(), String> {
    if status.active != ActiveState::Inactive {
        return Err(crate::fl!(
            "ui-app-role-did-not-become-inactive-observed-value2",
            role = (role).to_string(),
            value2 = format!("{:?}", status.active)
        ));
    }
    if must_be_disabled && status.enabled {
        return Err(crate::fl!(
            "ui-app-role-remains-enabled",
            role = (role).to_string()
        ));
    }
    Ok(())
}

async fn stop_disable_and_reconcile_offline_service<M: SystemdManager>(
    manager: &M,
    service: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    offline_repair_action(
        manager,
        SystemdAction::Stop,
        service,
        &crate::fl!("ui-app-could-not-stop-offline-mirror-service"),
        cancellation.child_token(),
    )
    .await?;
    offline_repair_action(
        manager,
        SystemdAction::Disable,
        service,
        &crate::fl!("ui-app-could-not-disable-offline-mirror-service"),
        cancellation.child_token(),
    )
    .await?;
    let mut status = offline_repair_status(
        manager,
        service,
        &crate::fl!("ui-app-could-not-inspect-stopped-offline-mirror-service"),
        cancellation.child_token(),
    )
    .await?;
    if repair_requires_failed_reset(Some(&status)) {
        offline_repair_action(
            manager,
            SystemdAction::ResetFailed,
            service,
            &crate::fl!("ui-app-could-not-reset-failed-offline-mirror-service"),
            cancellation.child_token(),
        )
        .await?;
        status = offline_repair_status(
            manager,
            service,
            &crate::fl!("ui-app-could-not-verify-reset-offline-mirror-service"),
            cancellation,
        )
        .await?;
    }
    require_repaired_unit(&status, &fl!("offline-service-role"), true)
}

#[cfg(test)]
async fn execute_offline_repair_plan<M: SystemdManager>(
    manager: &M,
    plan: &OfflineRepairPlan,
    cancellation: CancellationToken,
) -> Result<(), String> {
    pause_offline_repair_trigger(manager, plan, cancellation.child_token()).await?;
    stop_disable_and_reconcile_offline_service(manager, &plan.service, cancellation.child_token())
        .await
}

async fn pause_offline_repair_trigger<M: SystemdManager>(
    manager: &M,
    plan: &OfflineRepairPlan,
    cancellation: CancellationToken,
) -> Result<(), String> {
    if let Some(timer) = &plan.timer {
        offline_repair_action(
            manager,
            SystemdAction::Stop,
            timer,
            &crate::fl!("ui-app-could-not-stop-offline-mirror-timer"),
            cancellation.child_token(),
        )
        .await?;
        offline_repair_action(
            manager,
            SystemdAction::Disable,
            timer,
            &crate::fl!("ui-app-could-not-disable-offline-mirror-timer"),
            cancellation.child_token(),
        )
        .await?;
        let status = offline_repair_status(
            manager,
            timer,
            &crate::fl!("ui-app-could-not-inspect-paused-offline-mirror-timer"),
            cancellation.child_token(),
        )
        .await?;
        require_repaired_unit(&status, &fl!("offline-timer-role"), true)?;
    }
    Ok(())
}

async fn run_managed_offline_mirror_operation(
    connection: Connection,
    operation: Operation,
    control: Arc<ManualOperationControl>,
) -> String {
    let label = operation_label(operation);
    let finite = matches!(
        operation,
        Operation::PreviewInitialSync | Operation::SyncNow
    );
    if finite && let Err(error) = begin_finite_offline_vpn_use(&connection).await {
        control.finish();
        return crate::fl!(
            "ui-app-label-failed-for-value2-error",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            error = (error).to_string()
        );
    }
    let mut result = if matches!(operation, Operation::PauseSync | Operation::ResumeSync) {
        run_tracked_offline_operation(
            &connection,
            operation,
            &control,
            run_managed_offline_mirror_operation_result_with_cancellation(
                &connection,
                operation,
                control.cancellation(),
            ),
        )
        .await
    } else {
        let mode = rclone_bisync_mode(&connection, operation);
        let result = match mode {
            Ok(mode) => bisync_coordinator::run_saved(
                connection.id,
                mode,
                &default_work_root(),
                &app_command_runner(),
                control.cancellation(),
            )
            .await
            .map(|outcome| rclone_bisync_outcome_summary(&connection, mode, &outcome)),
            Err(error) => Err(error),
        };
        control.finish();
        result
    };
    if finite || (operation == Operation::PauseSync && result.is_ok()) {
        result = merge_offline_vpn_release(result, finish_offline_vpn_use(&connection).await);
    }
    match result {
        Ok(summary) => crate::fl!(
            "ui-app-label-completed-for-value2-summary",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            summary = (summary).to_string()
        ),
        Err(error) => crate::fl!(
            "ui-app-label-failed-for-value2-error",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            error = (error).to_string()
        ),
    }
}

async fn run_offline_mirror_repair_operation(
    connection: Connection,
    manual_operation: Option<Arc<ManualOperationControl>>,
) -> String {
    let path = offline_coordinator_state_path(connection.id);
    let result = run_offline_mirror_repair_operation_result(&connection, manual_operation).await;
    let state = if result.is_ok() {
        OfflineOperationState::Paused
    } else {
        OfflineOperationState::Failed
    };
    let state_result = write_offline_coordinator_state(
        &path,
        OfflineCoordinatorState::new(connection.id, OfflineOperation::Repair, state),
    );
    let result = match (result, state_result) {
        (Ok(engine), Ok(())) => Ok(engine),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    };
    let release = if result.is_ok() {
        finish_offline_vpn_use(&connection).await.err()
    } else {
        None
    };
    let mut notice = match result {
        Ok(OfflineRepairEngine::RcloneBisync) => crate::fl!(
            "ui-app-repair-completed-for-value1-its-timer-and",
            value1 = (connection.name).to_string()
        ),
        Ok(OfflineRepairEngine::OneDriveMonitor) => crate::fl!(
            "ui-app-repair-completed-for-value1-its-onedrive-monitor",
            value1 = (connection.name).to_string()
        ),
        Err(error) => crate::fl!(
            "ui-app-repair-failed-for-value1-error",
            value1 = (connection.name).to_string(),
            error = (error).to_string()
        ),
    };
    if let Some(error) = release {
        let _ = write!(
            notice,
            " VPN remains connected because automatic release failed: {error}"
        );
    }
    notice
}

async fn run_offline_mirror_repair_operation_result(
    connection: &Connection,
    manual_operation: Option<Arc<ManualOperationControl>>,
) -> Result<OfflineRepairEngine, String> {
    let connection = authoritative_offline_repair_connection(connection.id)?;
    let plan = offline_repair_plan(&connection)?;
    verify_offline_repair_unit_ownership(&plan)?;
    let manager = DbusSystemdManager;
    execute_coordinated_offline_repair(
        &manager,
        &plan,
        connection.id,
        manual_operation.as_deref(),
        &offline_coordinator_state_path(connection.id),
        CancellationToken::new(),
    )
    .await?;
    Ok(plan.engine)
}

async fn execute_coordinated_offline_repair<M: SystemdManager>(
    manager: &M,
    plan: &OfflineRepairPlan,
    connection_id: ConnectionId,
    manual_operation: Option<&ManualOperationControl>,
    state_path: &Path,
    cancellation: CancellationToken,
) -> Result<(), String> {
    pause_offline_repair_trigger(manager, plan, cancellation.child_token()).await?;
    cancel_manual_offline_operation(manual_operation, Duration::from_secs(5)).await?;
    if plan.engine == OfflineRepairEngine::RcloneBisync {
        // A scheduled worker is owned by this service rather than by the UI's
        // in-process cancellation handle. Stop it first, then require its
        // cross-process coordinator lock to be released. If another process
        // still owns the lock, Repair must not overwrite its running state.
        stop_disable_and_reconcile_offline_service(
            manager,
            &plan.service,
            cancellation.child_token(),
        )
        .await?;
        let _guard = bisync_coordinator::acquire_operation_guard_for_state(state_path)
            .map_err(|_| crate::fl!("ui-app-another-process-still-owns-a-running-offline"))?;
        write_offline_coordinator_state(
            state_path,
            OfflineCoordinatorState::new(
                connection_id,
                OfflineOperation::Repair,
                OfflineOperationState::Running,
            ),
        )?;
        return Ok(());
    }
    reject_unowned_running_operation(connection_id, state_path)?;
    write_offline_coordinator_state(
        state_path,
        OfflineCoordinatorState::new(
            connection_id,
            OfflineOperation::Repair,
            OfflineOperationState::Running,
        ),
    )?;
    stop_disable_and_reconcile_offline_service(manager, &plan.service, cancellation.child_token())
        .await
}

fn offline_coordinator_state_path(connection_id: ConnectionId) -> PathBuf {
    cosmic_ext_applet_mounter::offline_coordinator::state_path(&default_work_root(), connection_id)
}

fn write_offline_coordinator_state(
    path: &Path,
    state: OfflineCoordinatorState,
) -> Result<(), String> {
    cosmic_ext_applet_mounter::offline_coordinator::write_state(path, &state)
}

async fn cancel_manual_offline_operation(
    control: Option<&ManualOperationControl>,
    timeout: Duration,
) -> Result<(), String> {
    if let Some(control) = control
        && !control.cancel_and_wait(timeout).await
    {
        return Err(crate::fl!(
            "ui-app-the-manual-offline-mirror-operation-did-not"
        ));
    }
    Ok(())
}

fn reject_unowned_running_operation(
    connection_id: ConnectionId,
    state_path: &Path,
) -> Result<(), String> {
    if let Some(state) = cosmic_ext_applet_mounter::offline_coordinator::read_state(state_path)?
        && state.connection_id == connection_id
        && state.state == OfflineOperationState::Running
    {
        return Err(crate::fl!(
            "ui-app-another-process-still-owns-a-running-offline-2"
        ));
    }
    Ok(())
}

fn tracked_offline_operation(operation: Operation) -> OfflineOperation {
    match operation {
        Operation::PreviewInitialSync => OfflineOperation::Preview,
        Operation::SyncNow => OfflineOperation::Sync,
        Operation::ResumeSync => OfflineOperation::Resume,
        Operation::PauseSync => OfflineOperation::Pause,
        _ => OfflineOperation::Repair,
    }
}

async fn run_tracked_offline_operation<F>(
    connection: &Connection,
    operation: Operation,
    control: &ManualOperationControl,
    future: F,
) -> Result<String, String>
where
    F: std::future::Future<Output = Result<String, String>>,
{
    let path = offline_coordinator_state_path(connection.id);
    let tracked_operation = tracked_offline_operation(operation);
    let result = match write_offline_coordinator_state(
        &path,
        OfflineCoordinatorState::new(
            connection.id,
            tracked_operation,
            OfflineOperationState::Running,
        ),
    ) {
        Ok(()) => future.await,
        Err(error) => Err(error),
    };
    let final_state = if result.is_ok() {
        OfflineOperationState::Succeeded
    } else if control.cancellation().is_cancelled() {
        OfflineOperationState::Cancelled
    } else {
        OfflineOperationState::Failed
    };
    let state_result = write_offline_coordinator_state(
        &path,
        OfflineCoordinatorState::new(connection.id, tracked_operation, final_state),
    );
    control.finish();
    match (result, state_result) {
        (Ok(summary), Ok(())) => Ok(summary),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

async fn run_managed_offline_mirror_operation_result_with_cancellation(
    connection: &Connection,
    operation: Operation,
    cancellation: CancellationToken,
) -> Result<String, String> {
    prepare_offline_mirror_runtime(connection).await?;
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
    match operation {
        Operation::PreviewInitialSync | Operation::SyncNow => {
            let mode = rclone_bisync_mode(connection, operation)?;
            bisync_coordinator::run(
                connection,
                mode,
                &default_work_root(),
                &app_command_runner(),
                cancellation,
            )
            .await
            .map(|outcome| rclone_bisync_outcome_summary(connection, mode, &outcome))
        }
        Operation::ResumeSync => {
            start_rclone_offline_mirror_background(connection, &plan, cancellation).await
        }
        Operation::PauseSync => stop_rclone_offline_mirror_background(&plan, cancellation).await,
        _ => Err(crate::fl!(
            "ui-app-value1-is-not-a-managed-offline-mirror",
            value1 = (operation_label(operation)).to_string()
        )),
    }
}

fn rclone_bisync_mode(connection: &Connection, operation: Operation) -> Result<BisyncMode, String> {
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
    let initialized = initial_sync_marker(&plan).is_file();
    match operation {
        Operation::PreviewInitialSync => Ok(if initialized {
            BisyncMode::Preview
        } else {
            BisyncMode::InitialPreview
        }),
        Operation::SyncNow => Ok(if initialized {
            BisyncMode::Sync
        } else {
            BisyncMode::InitialSync
        }),
        _ => Err(crate::fl!("ui-app-operation-is-not-a-finite-rclone-bisync")),
    }
}

fn rclone_bisync_outcome_summary(
    connection: &Connection,
    mode: BisyncMode,
    outcome: &BisyncOutcome,
) -> String {
    if is_onedrive_rclone_mirror(connection) {
        let summary = if matches!(mode, BisyncMode::InitialPreview | BisyncMode::Preview) {
            match outcome.preview_pending.as_ref() {
                Some(Ok(files)) => fl!("onedrive-preview-differences", count = files.to_string()),
                _ => fl!("onedrive-preview-count-unavailable"),
            }
        } else if mode == BisyncMode::Scheduled {
            fl!("onedrive-sync-completed")
        } else {
            fl!("onedrive-sync-checksum-verified")
        };
        let summary = if mode == BisyncMode::InitialPreview {
            fl!(
                "ui-app-summary-initial-sync-has-not-run-yet",
                summary = summary
            )
        } else {
            summary
        };
        return if outcome.warnings.is_empty() {
            summary
        } else {
            fl!(
                "onedrive-runtime-warning",
                summary = summary,
                warning = outcome.warnings.join(" ")
            )
        };
    }
    let mut summary = if matches!(mode, BisyncMode::InitialPreview | BisyncMode::Preview) {
        let summary = preview_summary_text(&outcome.output);
        if mode == BisyncMode::InitialPreview {
            crate::fl!(
                "ui-app-summary-initial-sync-has-not-run-yet",
                summary = (summary).to_string()
            )
        } else {
            summary
        }
    } else {
        let summary = sync_output_summary(&outcome.output);
        if mode == BisyncMode::InitialSync {
            crate::fl!(
                "ui-app-summary-initial-synchronization-is-now-recorded-as",
                summary = (summary).to_string()
            )
        } else {
            summary
        }
    };
    if !outcome.warnings.is_empty() {
        let _ = write!(summary, " Warning: {}", outcome.warnings.join(" "));
    }
    summary
}

async fn run_managed_onedrive_offline_mirror_operation(
    connection: Connection,
    operation: Operation,
    control: Arc<ManualOperationControl>,
) -> String {
    let label = operation_label(operation);
    let finite = matches!(
        operation,
        Operation::PreviewInitialSync | Operation::SyncNow
    );
    if finite && let Err(error) = begin_finite_offline_vpn_use(&connection).await {
        control.finish();
        return crate::fl!(
            "ui-app-label-failed-for-value2-error",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            error = (error).to_string()
        );
    }
    let mut result = run_tracked_offline_operation(
        &connection,
        operation,
        &control,
        run_managed_onedrive_offline_mirror_operation_result_with_cancellation(
            &connection,
            operation,
            control.cancellation(),
        ),
    )
    .await;
    if finite || (operation == Operation::PauseSync && result.is_ok()) {
        result = merge_offline_vpn_release(result, finish_offline_vpn_use(&connection).await);
    }
    match result {
        Ok(summary) => crate::fl!(
            "ui-app-label-completed-for-value2-summary",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            summary = (summary).to_string()
        ),
        Err(error) => crate::fl!(
            "ui-app-label-failed-for-value2-error",
            label = (label).to_string(),
            value2 = (connection.name).to_string(),
            error = (error).to_string()
        ),
    }
}

async fn run_managed_onedrive_offline_mirror_operation_result_with_cancellation(
    connection: &Connection,
    operation: Operation,
    cancellation: CancellationToken,
) -> Result<String, String> {
    prepare_onedrive_offline_mirror_runtime(connection)?;
    let plan = onedrive_mirror_plan_for_app(connection)?;
    match operation {
        Operation::PreviewInitialSync => preview_onedrive_offline_mirror(&plan, cancellation).await,
        Operation::SyncNow => sync_onedrive_offline_mirror(connection, &plan, cancellation).await,
        Operation::ResumeSync => {
            start_onedrive_offline_mirror_background(connection, &plan, cancellation).await
        }
        Operation::PauseSync => stop_onedrive_offline_mirror_background(&plan, cancellation).await,
        _ => Err(crate::fl!(
            "ui-app-value1-is-not-a-managed-onedrive-mirror",
            value1 = (operation_label(operation)).to_string()
        )),
    }
}

async fn start_rclone_offline_mirror_background(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    if !initial_sync_marker(plan).exists() {
        return Err(crate::fl!(
            "ui-app-background-sync-requires-a-successful-preview-and"
        ));
    }
    ensure_background_sync_may_start(connection).await?;
    let result = async {
        if is_onedrive_rclone_mirror(connection) {
            verify_rclone_access(connection).await?;
        }
        install_rclone_offline_mirror_units_result(connection).await?;
        let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
        let manager = DbusSystemdManager;
        manager
            .action(
                SystemdAction::DaemonReload,
                None,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| error.to_string())?;
        resume_rclone_timer_with(&manager, &timer.name, cancellation).await?;
        Ok(crate::fl!(
            "ui-app-background-sync-timer-started-use-sync-now"
        ))
    }
    .await;
    if result.is_err() {
        merge_offline_vpn_release(result, finish_offline_vpn_use(connection).await)
    } else {
        result
    }
}

async fn resume_rclone_timer_with<M: SystemdManager>(
    manager: &M,
    timer: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err(fl!("microsoft-oauth-cancelled"));
    }
    let result = async {
        manager
            .action(
                SystemdAction::Enable,
                Some(timer),
                cancellation.child_token(),
            )
            .await
            .map_err(|_| fl!("mirror-background-start-incomplete"))?;
        manager
            .action(
                SystemdAction::Start,
                Some(timer),
                cancellation.child_token(),
            )
            .await
            .map_err(|_| fl!("mirror-background-start-incomplete"))?;
        let status = manager
            .action(SystemdAction::Status, Some(timer), cancellation)
            .await
            .map_err(|_| fl!("mirror-background-start-incomplete"))?;
        if !status.is_some_and(|status| status.enabled && status.active == ActiveState::Active) {
            return Err(fl!("mirror-background-start-incomplete"));
        }
        Ok(())
    }
    .await;
    if let Err(error) = result {
        let stop = manager
            .action(SystemdAction::Stop, Some(timer), CancellationToken::new())
            .await;
        let disable = manager
            .action(
                SystemdAction::Disable,
                Some(timer),
                CancellationToken::new(),
            )
            .await;
        return Err(if stop.is_err() || disable.is_err() {
            fl!("mirror-background-start-rollback-failed")
        } else {
            error
        });
    }
    Ok(())
}

async fn stop_rclone_offline_mirror_background(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    pause_rclone_background_with(&DbusSystemdManager, plan, cancellation).await
}

async fn pause_rclone_background_with<M: SystemdManager>(
    manager: &M,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    let repair = OfflineRepairPlan {
        engine: OfflineRepairEngine::RcloneBisync,
        service: UnitName::new(plan.connection_id, UnitKind::Service),
        timer: Some(UnitName::new(plan.connection_id, UnitKind::Timer)),
    };
    pause_offline_repair_trigger(manager, &repair, cancellation.child_token()).await?;
    offline_repair_action(
        manager,
        SystemdAction::Stop,
        &repair.service,
        &fl!("ui-app-could-not-stop-offline-mirror-service"),
        cancellation.child_token(),
    )
    .await?;
    offline_repair_action(
        manager,
        SystemdAction::Disable,
        &repair.service,
        &fl!("ui-app-could-not-disable-offline-mirror-service"),
        cancellation.child_token(),
    )
    .await?;
    let status = offline_repair_status(
        manager,
        &repair.service,
        &fl!("ui-app-could-not-inspect-stopped-offline-mirror-service"),
        cancellation,
    )
    .await?;
    require_repaired_unit(&status, &fl!("offline-service-role"), true)?;
    Ok(fl!("ui-app-background-sync-timer-stopped-manual-sync-now"))
}

async fn start_onedrive_offline_mirror_background(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    if !onedrive_initial_sync_marker(plan).exists() {
        return Err(crate::fl!(
            "ui-app-background-sync-requires-a-successful-preview-and"
        ));
    }
    ensure_background_sync_may_start(connection).await?;
    let result = async {
        install_onedrive_offline_mirror_unit_result(connection).await?;
        let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
        let manager = DbusSystemdManager;
        manager
            .action(
                SystemdAction::DaemonReload,
                None,
                cancellation.child_token(),
            )
            .await
            .map_err(|error| error.to_string())?;
        manager
            .action(
                SystemdAction::Enable,
                Some(&service.name),
                cancellation.child_token(),
            )
            .await
            .map_err(|error| error.to_string())?;
        manager
            .action(SystemdAction::Start, Some(&service.name), cancellation)
            .await
            .map_err(|error| error.to_string())?;
        Ok(crate::fl!(
            "ui-app-onedrive-background-monitor-started-use-sync-now"
        ))
    }
    .await;
    if result.is_err() {
        merge_offline_vpn_release(result, finish_offline_vpn_use(connection).await)
    } else {
        result
    }
}

async fn ensure_background_sync_may_start(connection: &Connection) -> Result<(), String> {
    let options = match &connection.mode {
        ConnectionMode::OfflineMirror(options) => options,
        ConnectionMode::OnlineMount(_) => {
            return Err(crate::fl!("ui-app-connection-is-not-an-offline-mirror"));
        }
    };
    let network_ready = strict_network_ready().await?;
    let request = SyncRequest {
        trigger: SyncTrigger::Scheduled,
        preview_completed: true,
        user_confirmed: true,
        metered_network: current_metered_network().await,
        running: false,
        readiness: SyncReadiness {
            network_ready,
            vpn_ready: true,
        },
    };
    match sync_now_request(request, options).map_err(|error| error.to_string())? {
        SyncDecision::Run => ensure_vpn_ready_for_connection(connection).await,
        SyncDecision::WaitForNetwork => Err(crate::fl!(
            "ui-app-background-sync-is-waiting-for-network-readiness"
        )),
        SyncDecision::WaitForVpn => Err(crate::fl!(
            "ui-app-background-sync-is-waiting-for-vpn-readiness"
        )),
        SyncDecision::PauseMetered => Err(crate::fl!(
            "ui-app-background-sync-is-paused-on-metered-networks"
        )),
        SyncDecision::Reject(rejection) => Err(sync_rejection_message(rejection)),
    }
}

async fn strict_network_ready() -> Result<bool, String> {
    DbusNetworkManagerVpn::connected().await.map_err(|error| {
        crate::fl!(
            "ui-app-could-not-inspect-networkmanager-readiness-error",
            error = (error).to_string()
        )
    })
}

async fn begin_finite_offline_vpn_use(connection: &Connection) -> Result<(), String> {
    if !strict_network_ready().await? {
        return Err(crate::fl!(
            "ui-app-offline-synchronization-is-waiting-for-network-readiness"
        ));
    }
    ensure_vpn_ready_for_connection(connection).await?;
    if let Some(profile_id) = connection.vpn_profile_id {
        register_finite_offline_vpn_user(profile_id, connection.id);
    }
    Ok(())
}

async fn finish_offline_vpn_use(connection: &Connection) -> Result<(), String> {
    if let Some(profile_id) = connection.vpn_profile_id {
        unregister_finite_offline_vpn_user(profile_id, connection.id);
    }
    maybe_shutdown_vpn_after_unmount(connection).await
}

fn merge_offline_vpn_release(
    operation: Result<String, String>,
    release: Result<(), String>,
) -> Result<String, String> {
    match (operation, release) {
        (result, Ok(())) => result,
        (Ok(summary), Err(error)) => Ok(crate::fl!(
            "ui-app-summary-vpn-remains-connected-because-automatic-release",
            summary = (summary).to_string(),
            error = (error).to_string()
        )),
        (Err(operation_error), Err(release_error)) => Err(crate::fl!(
            "ui-app-operation-error-vpn-remains-connected-because-automatic",
            operation_error = (operation_error).to_string(),
            release_error = (release_error).to_string()
        )),
    }
}

async fn current_network_ready() -> bool {
    DbusNetworkManagerVpn::connected().await.unwrap_or(true)
}

pub(crate) async fn run_offline_readiness_guard(arguments: &[String]) -> Result<(), String> {
    let [connection_id] = arguments else {
        return Err(crate::fl!(
            "ui-app-offline-readiness-guard-requires-exactly-one-saved"
        ));
    };
    let connection_id = Uuid::parse_str(connection_id)
        .map(ConnectionId::from_uuid)
        .map_err(|_| crate::fl!("ui-app-offline-readiness-guard-received-an-invalid-connection"))?;
    let config = load_runtime_config()?;
    let connection = config
        .document
        .connections
        .iter()
        .find(|connection| {
            connection.id == connection_id
                && connection.enabled
                && matches!(connection.mode, ConnectionMode::OfflineMirror(_))
        })
        .ok_or_else(|| crate::fl!("ui-app-saved-enabled-offline-mirror-connection-was-not"))?;
    match DbusNetworkManagerVpn::connected().await {
        Ok(true) => {}
        Ok(false) => {
            return Err(crate::fl!(
                "ui-app-offline-synchronization-is-waiting-for-network-readiness"
            ));
        }
        Err(error) => {
            return Err(crate::fl!(
                "ui-app-could-not-inspect-networkmanager-readiness-before-offline",
                error = (error).to_string()
            ));
        }
    }
    let Some(profile_id) = connection.vpn_profile_id else {
        return Ok(());
    };
    let profile = config
        .document
        .vpn_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| crate::fl!("ui-app-configured-vpn-profile-is-no-longer-available"))?;
    if profile.kind != VpnKind::NetworkManager {
        return Err(crate::fl!(
            "ui-app-scheduled-offline-mirror-requires-a-saved-networkmanager"
        ));
    }
    if vpn_profile_ready(profile).await? {
        Ok(())
    } else {
        Err(crate::fl!(
            "ui-app-scheduled-offline-mirror-is-waiting-for-vpn",
            value1 = (profile.name).to_string()
        ))
    }
}

#[cfg(test)]
fn network_manager_connected(state: &str) -> bool {
    state.trim().to_ascii_lowercase().starts_with("connected")
}

async fn current_metered_network() -> bool {
    DbusNetworkManagerVpn::metered().await.unwrap_or(false)
}

async fn ensure_vpn_ready_for_connection(connection: &Connection) -> Result<(), String> {
    let Some(profile_id) = connection.vpn_profile_id else {
        return Ok(());
    };
    let Some(mut profile) = Config::load_runtime()
        .config
        .document
        .vpn_profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
    else {
        return Err(crate::fl!(
            "ui-app-configured-vpn-profile-is-no-longer-available"
        ));
    };
    let is_openconnect = if profile.kind == VpnKind::NetworkManager {
        if let Some(uuid) = profile.external_profile_id.as_deref() {
            DbusNetworkManagerVpn::profile_is_openconnect(uuid)
                .await
                .unwrap_or(false)
        } else {
            false
        }
    } else {
        false
    };
    apply_openconnect_timeout(&mut profile, is_openconnect);
    let deadline =
        tokio::time::Instant::now() + Duration::from_secs(u64::from(profile.timeout_seconds));
    ensure_vpn_profile_ready(&profile, deadline, is_openconnect).await
}

fn apply_openconnect_timeout(profile: &mut VpnProfile, is_openconnect: bool) {
    if is_openconnect {
        profile.timeout_seconds = profile.timeout_seconds.max(90);
    }
}

async fn ensure_vpn_profile_ready(
    profile: &VpnProfile,
    deadline: tokio::time::Instant,
    is_openconnect: bool,
) -> Result<(), String> {
    // Serialize activation for each profile. Other waiting storage requests
    // recheck the tunnel before considering opening another authentication UI.
    static ACTIVATIONS: std::sync::LazyLock<
        std::sync::Mutex<BTreeMap<VpnProfileId, Arc<tokio::sync::Mutex<()>>>>,
    > = std::sync::LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));
    let lock = ACTIVATIONS
        .lock()
        .expect("VPN activation locks")
        .entry(profile.id)
        .or_default()
        .clone();
    let _guard = lock.lock().await;
    let already_connected = vpn_tunnel_ready(profile).await.unwrap_or(false);

    if !already_connected {
        match profile.kind {
            VpnKind::NetworkManager => {
                DbusNetworkManagerVpn
                    .activate(profile, CancellationToken::new())
                    .await
                    .map_err(|error| {
                        crate::fl!(
                            "ui-app-could-not-activate-networkmanager-vpn-error",
                            error = (error).to_string()
                        )
                    })?;
            }
            VpnKind::LegacyCisco => {
                return Err(crate::fl!(
                    "ui-app-this-connection-still-references-an-unsupported-legacy"
                ));
            }
        }
    }
    match wait_for_vpn_ready(profile, deadline, !already_connected).await {
        Ok(true) => {
            if !already_connected {
                mark_vpn_applet_activated(profile.id);
            }
            Ok(())
        }
        Ok(false) => {
            let cleanup = if already_connected {
                Ok(())
            } else {
                cleanup_timed_out_vpn_activation(profile, is_openconnect).await
            };
            let mut error = crate::fl!(
                "ui-app-value1-did-not-become-ready-within-value2",
                value1 = (profile.name).to_string(),
                value2 = (profile.timeout_seconds).to_string()
            );
            if let Err(cleanup_error) = cleanup {
                error.push_str(&crate::fl!(
                    "ui-app-timeout-cleanup-failed-cleanup-error",
                    cleanup_error = (cleanup_error).to_string()
                ));
            }
            Err(error)
        }
        Err(error) => Err(error),
    }
}

async fn cleanup_timed_out_vpn_activation(
    profile: &VpnProfile,
    is_openconnect: bool,
) -> Result<(), String> {
    DbusNetworkManagerVpn
        .deactivate(profile, CancellationToken::new())
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-deactivate-timed-out-vpn-error",
                error = (error).to_string()
            )
        })?;
    if is_openconnect {
        DbusNetworkManagerVpn::restart_openconnect_agent()
            .await
            .map_err(|error| {
                crate::fl!(
                    "ui-app-could-not-close-the-timed-out-authentication",
                    error = (error).to_string()
                )
            })?;
    }
    Ok(())
}

async fn vpn_tunnel_ready(profile: &VpnProfile) -> Result<bool, String> {
    match profile.kind {
        VpnKind::NetworkManager => DbusNetworkManagerVpn
            .state(profile, CancellationToken::new())
            .await
            .map(|state| state.ready())
            .map_err(|error| error.to_string()),
        VpnKind::LegacyCisco => Err(crate::fl!(
            "ui-app-legacy-cisco-profiles-are-unsupported-select-a"
        )),
    }
}

async fn vpn_profile_ready(profile: &VpnProfile) -> Result<bool, String> {
    let connected = vpn_tunnel_ready(profile).await?;

    if !connected {
        return Ok(false);
    }
    readiness_report(
        &InProcessReadinessProbe,
        &profile.readiness_checks,
        CancellationToken::new(),
    )
    .await
    .map(|report| report.ready)
    .map_err(|error| error.to_string())
}

async fn wait_for_vpn_ready(
    profile: &VpnProfile,
    deadline: tokio::time::Instant,
    activation_requested: bool,
) -> Result<bool, String> {
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Ok(false);
        }
        match profile.kind {
            VpnKind::NetworkManager => {
                match DbusNetworkManagerVpn
                    .state(profile, CancellationToken::new())
                    .await
                {
                    Ok(state) => {
                        if let Some(error) =
                            vpn_activation_terminal_error(state, activation_requested)
                        {
                            return Err(error);
                        }
                        if state.ready() && vpn_profile_ready(profile).await.unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                    Err(_) => {
                        // Retain the existing tolerance for transient D-Bus inspection
                        // failures while the overall authentication deadline remains.
                    }
                }
            }
            VpnKind::LegacyCisco => return vpn_profile_ready(profile).await,
        }
        tokio::time::sleep_until(
            (tokio::time::Instant::now() + Duration::from_secs(2)).min(deadline),
        )
        .await;
    }
}

fn vpn_activation_terminal_error(
    state: cosmic_ext_applet_mounter::vpn::VpnConnectionState,
    activation_requested: bool,
) -> Option<String> {
    use cosmic_ext_applet_mounter::vpn::VpnConnectionState;

    match state {
        VpnConnectionState::Failed => Some(fl!("vpn-activation-failed")),
        VpnConnectionState::Disconnected if activation_requested => {
            Some(fl!("vpn-activation-cancelled"))
        }
        VpnConnectionState::Unknown
        | VpnConnectionState::Disconnected
        | VpnConnectionState::Activating
        | VpnConnectionState::Activated => None,
    }
}

async fn maybe_shutdown_vpn_after_unmount(connection: &Connection) -> Result<(), String> {
    let Some(profile_id) = connection.vpn_profile_id else {
        return Ok(());
    };
    if !connection.disconnect_vpn_when_unused {
        return Ok(());
    }
    let activated = applet_activated_vpns();
    let snapshot = cosmic_ext_applet_mounter::vpn::VpnUsageSnapshot {
        profile_id,
        applet_activated: activated.contains(&profile_id),
        active_connection_ids: active_connections_using_vpn(profile_id, Some(connection.id))
            .await?,
    };
    match shutdown_decision(&snapshot) {
        VpnShutdownDecision::Disconnect => {
            disconnect_vpn_profile(profile_id).await?;
            unmark_vpn_applet_activated(profile_id);
        }
        VpnShutdownDecision::NoAction
        | VpnShutdownDecision::KeepAliveShared
        | VpnShutdownDecision::KeepAlivePreExisting => {}
    }
    Ok(())
}

async fn disconnect_vpn_profile(profile_id: VpnProfileId) -> Result<(), String> {
    let Some(profile) = Config::load_runtime()
        .config
        .document
        .vpn_profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
    else {
        return Ok(());
    };
    match profile.kind {
        VpnKind::NetworkManager => DbusNetworkManagerVpn
            .deactivate(&profile, CancellationToken::new())
            .await
            .map_err(|error| {
                crate::fl!(
                    "ui-app-could-not-disconnect-networkmanager-vpn-error",
                    error = (error).to_string()
                )
            }),
        VpnKind::LegacyCisco => Err(crate::fl!(
            "ui-app-legacy-cisco-profiles-are-unsupported-no-proprietary"
        )),
    }
}

async fn active_connections_using_vpn(
    profile_id: VpnProfileId,
    exclude_connection: Option<ConnectionId>,
) -> Result<BTreeSet<ConnectionId>, String> {
    let config = Config::load_runtime().config.document;
    let mount_entries = host_mount_entries().await?;
    let mut active = config
        .connections
        .iter()
        .filter(|connection| connection.vpn_profile_id == Some(profile_id))
        .filter(|connection| Some(connection.id) != exclude_connection)
        .filter(|connection| connection_appears_active(connection, &mount_entries))
        .map(|connection| connection.id)
        .collect::<BTreeSet<_>>();
    if let Some(finite_users) = active_finite_offline_vpn_users()
        .lock()
        .expect("finite Offline Mirror VPN users")
        .get(&profile_id)
    {
        active.extend(
            finite_users
                .iter()
                .copied()
                .filter(|connection_id| Some(*connection_id) != exclude_connection),
        );
    }
    Ok(active)
}

fn active_finite_offline_vpn_users()
-> &'static std::sync::Mutex<BTreeMap<VpnProfileId, BTreeSet<ConnectionId>>> {
    static USERS: LazyLock<std::sync::Mutex<BTreeMap<VpnProfileId, BTreeSet<ConnectionId>>>> =
        LazyLock::new(Default::default);
    &USERS
}

fn register_finite_offline_vpn_user(profile_id: VpnProfileId, connection_id: ConnectionId) {
    active_finite_offline_vpn_users()
        .lock()
        .expect("finite Offline Mirror VPN users")
        .entry(profile_id)
        .or_default()
        .insert(connection_id);
}

fn unregister_finite_offline_vpn_user(profile_id: VpnProfileId, connection_id: ConnectionId) {
    let mut users = active_finite_offline_vpn_users()
        .lock()
        .expect("finite Offline Mirror VPN users");
    if let Some(connection_ids) = users.get_mut(&profile_id) {
        connection_ids.remove(&connection_id);
        if connection_ids.is_empty() {
            users.remove(&profile_id);
        }
    }
}

fn connection_appears_active(connection: &Connection, mount_entries: &[MountEntry]) -> bool {
    match connection.mode {
        ConnectionMode::OnlineMount(_) => {
            mount_entries
                .iter()
                .any(|entry| entry.target == connection.local_path)
                || runtime_service_status(connection.id).is_some_and(|status| {
                    matches!(status.active, ActiveState::Active | ActiveState::Activating)
                })
        }
        ConnectionMode::OfflineMirror(_) => {
            runtime_service_status(connection.id).is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) || runtime_unit_status(connection.id, UnitKind::Timer).is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            })
        }
    }
}

fn applet_activated_vpns_path() -> PathBuf {
    default_work_root().join("vpn-applet-activated.ron")
}

fn applet_activated_vpns() -> BTreeSet<VpnProfileId> {
    let path = applet_activated_vpns_path();
    let Ok(contents) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    ron::from_str(&contents).unwrap_or_default()
}

fn write_applet_activated_vpns(profiles: &BTreeSet<VpnProfileId>) {
    let path = applet_activated_vpns_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if profiles.is_empty() {
        let _ = fs::remove_file(path);
    } else if let Ok(contents) = ron::to_string(profiles) {
        let _ = fs::write(path, contents);
    }
}

fn mark_vpn_applet_activated(profile_id: VpnProfileId) {
    let mut profiles = applet_activated_vpns();
    profiles.insert(profile_id);
    write_applet_activated_vpns(&profiles);
}

fn unmark_vpn_applet_activated(profile_id: VpnProfileId) {
    let mut profiles = applet_activated_vpns();
    profiles.remove(&profile_id);
    write_applet_activated_vpns(&profiles);
}

async fn stop_onedrive_offline_mirror_background(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let manager = DbusSystemdManager;
    let _ = manager
        .action(
            SystemdAction::Stop,
            Some(&service.name),
            cancellation.child_token(),
        )
        .await;
    manager
        .action(SystemdAction::Disable, Some(&service.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(crate::fl!(
        "ui-app-onedrive-background-monitor-stopped-manual-sync-now"
    ))
}

async fn preview_onedrive_offline_mirror(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    let output = app_command_runner()
        .run(
            one_drive_preview_request(plan).map_err(|error| error.to_string())?,
            cancellation,
        )
        .await
        .map_err(|error| onedrive_runtime_command_error("preview", error))?;
    let summary = onedrive_output_summary(&fl!("operation-preview"), &output);
    if !onedrive_initial_sync_marker(plan).exists() {
        write_onedrive_initial_preview_marker(plan, &summary)?;
        Ok(crate::fl!(
            "ui-app-summary-initial-sync-has-not-run-yet",
            summary = (summary).to_string()
        ))
    } else {
        Ok(summary)
    }
}

async fn sync_onedrive_offline_mirror(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    cancellation: CancellationToken,
) -> Result<String, String> {
    let options = match &connection.mode {
        ConnectionMode::OfflineMirror(options) => options,
        ConnectionMode::OnlineMount(_) => {
            return Err(crate::fl!("ui-app-connection-is-not-an-offline-mirror"));
        }
    };
    let initialized = onedrive_initial_sync_marker(plan).exists();
    let preview_completed = initialized || onedrive_initial_preview_marker(plan).exists();
    let decision = sync_now_request(
        SyncRequest {
            trigger: if initialized {
                SyncTrigger::Manual
            } else {
                SyncTrigger::Initial
            },
            preview_completed,
            user_confirmed: preview_completed,
            metered_network: false,
            running: false,
            readiness: SyncReadiness {
                network_ready: true,
                vpn_ready: true,
            },
        },
        options,
    )
    .map_err(|error| error.to_string())?;
    match decision {
        SyncDecision::Run => {}
        SyncDecision::WaitForNetwork => {
            return Err(crate::fl!("ui-app-waiting-for-network-readiness"));
        }
        SyncDecision::WaitForVpn => return Err(crate::fl!("ui-app-waiting-for-vpn-readiness")),
        SyncDecision::PauseMetered => {
            return Err(crate::fl!(
                "ui-app-automatic-synchronization-is-paused-on-metered-networks"
            ));
        }
        SyncDecision::Reject(rejection) => return Err(sync_rejection_message(rejection)),
    }

    let request = if initialized {
        one_drive_sync_request(plan).map_err(|error| error.to_string())?
    } else {
        one_drive_initial_sync_request(plan).map_err(|error| error.to_string())?
    };
    let output = app_command_runner()
        .run(request, cancellation)
        .await
        .map_err(|error| onedrive_runtime_command_error("sync", error))?;
    if initialized {
        Ok(onedrive_output_summary(&fl!("sync-stage-sync"), &output))
    } else {
        write_onedrive_initial_sync_marker(plan)?;
        let _ = fs::remove_file(onedrive_initial_preview_marker(plan));
        Ok(crate::fl!(
            "ui-app-value1-initial-synchronization-is-now-recorded-as",
            value1 = onedrive_output_summary(&fl!("sync-stage-sync"), &output)
        ))
    }
}

fn prepare_online_mount_runtime(connection: &Connection) -> Result<(), String> {
    fs::create_dir_all(&connection.local_path).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-mountpoint-value1-error",
            value1 = (connection.local_path.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(default_runtime_root()).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-runtime-directory-value1-error",
            value1 = (default_runtime_root().display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(
        default_cache_root()
            .join("rclone")
            .join(connection.id.to_string()),
    )
    .map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-rclone-cache-directory-error",
            error = (error).to_string()
        )
    })?;
    Ok(())
}

fn prepare_onedriver_online_mount_runtime(connection: &Connection) -> Result<(), String> {
    prepare_onedriver_online_mount_runtime_with(
        connection,
        &default_cache_root(),
        &default_config_root(),
    )
}

fn prepare_onedriver_online_mount_runtime_with(
    connection: &Connection,
    cache_root: &Path,
    config_root: &Path,
) -> Result<(), String> {
    let plan = onedriver_mount_plan(connection, cache_root, config_root)
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&plan.mountpoint).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-mountpoint-value1-error",
            value1 = (plan.mountpoint.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(&plan.cache_directory).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-onedriver-cache-directory-value1",
            value1 = (plan.cache_directory.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    if let Some(config_directory) = plan.config_file.parent() {
        fs::create_dir_all(config_directory).map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-create-onedriver-config-directory-value1",
                value1 = (config_directory.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    }
    Ok(())
}

async fn prepare_offline_mirror_runtime(connection: &Connection) -> Result<(), String> {
    let paths = if is_legacy_onedrive_mirror(connection) {
        cosmic_ext_applet_mounter::mirror_access::LocalMirrorPaths::native(connection)?
    } else {
        cosmic_ext_applet_mounter::mirror_access::LocalMirrorPaths::resolve(
            connection,
            CancellationToken::new(),
        )
        .await?
    };
    fs::create_dir_all(&paths.mirror).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-mirror-directory-value1-error",
            value1 = (connection.local_path.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(default_work_root()).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-work-directory-value1-error",
            value1 = (default_work_root().display()).to_string(),
            error = (error).to_string()
        )
    })?;
    if let ConnectionMode::OfflineMirror(options) = &connection.mode {
        fs::create_dir_all(&paths.recovery).map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-create-recovery-directory-value1-error",
                value1 = (options.recovery_directory.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
fn ensure_sharepoint_access_marker(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<(), String> {
    let Some(name) = &plan.access_marker_name else {
        return Ok(());
    };
    let path = connection.local_path.join(name);
    let expected = format!("{}\n", connection.id);
    if !path.exists() && !initial_sync_marker(plan).exists() {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => file.write_all(expected.as_bytes()).map_err(|error| {
                fl!("sharepoint-anchor-create-error", error = error.to_string())
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(fl!(
                    "sharepoint-anchor-create-error",
                    error = error.to_string()
                ));
            }
        }
    }
    let metadata = fs::symlink_metadata(&path).map_err(|_| fl!("sharepoint-anchor-missing"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(fl!("sharepoint-anchor-invalid"));
    }
    if fs::read_to_string(&path)
        .map_err(|error| fl!("sharepoint-anchor-read-error", error = error.to_string()))?
        != expected
    {
        return Err(fl!("sharepoint-anchor-changed"));
    }
    Ok(())
}

fn prepare_onedrive_offline_mirror_runtime(connection: &Connection) -> Result<(), String> {
    prepare_onedrive_offline_mirror_runtime_with(connection, &default_config_root())
}

fn prepare_onedrive_offline_mirror_runtime_with(
    connection: &Connection,
    config_root: &Path,
) -> Result<(), String> {
    let plan = one_drive_mirror_plan(
        connection,
        config_root,
        &OneDriveIsolationReport {
            active_onedriver_paths: Vec::new(),
        },
    )
    .map_err(|error| error.to_string())?;
    fs::create_dir_all(&plan.sync_directory).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-onedrive-sync-directory-value1",
            value1 = (plan.sync_directory.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(&plan.config_directory).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-onedrive-config-directory-value1",
            value1 = (plan.config_directory.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    fs::create_dir_all(&plan.recovery_directory).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-onedrive-recovery-directory-value1",
            value1 = (plan.recovery_directory.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    Ok(())
}

fn onedrive_auth_files_for_connection(connection_id: ConnectionId) -> OneDriveMirrorAuthFiles {
    onedrive_auth_files_in(connection_id, &default_work_root().join("onedrive-auth"))
}

fn onedrive_auth_files_in(connection_id: ConnectionId, root: &Path) -> OneDriveMirrorAuthFiles {
    let stem = format!("cosmic-mounter-onedrive-auth-{connection_id}");
    OneDriveMirrorAuthFiles {
        auth_url_file: root.join(format!("{stem}-url")),
        response_url_file: root.join(format!("{stem}-response")),
    }
}

#[allow(dead_code)]
fn onedrive_auth_open_command(auth_files: &OneDriveMirrorAuthFiles) -> String {
    format!("xdg-open \"$(cat {})\"", auth_files.auth_url_file.display())
}

fn validate_onedrive_auth_url(value: &str) -> Result<(), String> {
    validate_setup_value(&crate::fl!("ui-app-onedrive-auth-url"), value, true)?;
    if value.starts_with("https://login.microsoftonline.com/")
        || value.starts_with("https://login.live.com/")
    {
        Ok(())
    } else {
        Err(crate::fl!("ui-app-the-generated-onedrive-auth-url-is-not"))
    }
}

fn default_onedrive_auth_mode(flatpak: bool) -> OneDriveMirrorSetupMode {
    if flatpak {
        OneDriveMirrorSetupMode::ManualAuthFiles
    } else {
        OneDriveMirrorSetupMode::Interactive
    }
}

async fn open_onedrive_auth_url(auth_files: &OneDriveMirrorAuthFiles) -> Result<String, String> {
    let url = tokio::fs::read_to_string(&auth_files.auth_url_file)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-app-auth-url-file-is-not-ready-yet",
                value1 = (auth_files.auth_url_file.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    let url = url.trim();
    validate_onedrive_auth_url(url)?;
    let request = async {
        let connection = zbus::Connection::session().await?;
        let proxy = zbus::Proxy::new(
            &connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.OpenURI",
        )
        .await?;
        let options = BTreeMap::<String, zbus::zvariant::OwnedValue>::new();
        let _: zbus::zvariant::OwnedObjectPath = proxy.call("OpenURI", &("", url, options)).await?;
        Ok::<(), zbus::Error>(())
    };
    match tokio::time::timeout(Duration::from_secs(10), request).await {
        Ok(Ok(())) => Ok(fl!("onedrive-auth-browser-requested")),
        _ => Err(fl!("onedrive-auth-browser-request-failed")),
    }
}

fn validate_onedrive_auth_response_url(value: &str) -> Result<(), String> {
    validate_setup_value(&crate::fl!("ui-app-onedrive-response-url"), value, true)?;
    if value.starts_with("https://login.microsoftonline.com/") && value.contains("code=") {
        Ok(())
    } else {
        Err(
            "Paste the full Microsoft native-client redirect URL. It should begin with https://login.microsoftonline.com/ and contain code=."
                .into(),
        )
    }
}

fn write_onedrive_auth_response(
    auth_files: &OneDriveMirrorAuthFiles,
    response_url: &str,
) -> Result<(), String> {
    if let Some(parent) = auth_files.response_url_file.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-create-onedrive-response-handoff-directory",
                value1 = (parent.display()).to_string(),
                error = (error).to_string()
            )
        })?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| fl!("onedrive-auth-directory-private-failed"))?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&auth_files.response_url_file)
        .map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-open-response-file-value1-error",
                value1 = (auth_files.response_url_file.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-restrict-response-file-permissions-value1",
                value1 = (auth_files.response_url_file.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    file.write_all(response_url.as_bytes()).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-write-response-file-value1-error",
            value1 = (auth_files.response_url_file.display()).to_string(),
            error = (error).to_string()
        )
    })
}

#[allow(dead_code)]
fn prepare_onedrive_auth_files(auth_files: &OneDriveMirrorAuthFiles) -> Result<(), String> {
    if let Some(parent) = auth_files.auth_url_file.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            crate::fl!(
                "ui-app-failed-to-create-onedrive-auth-handoff-directory",
                value1 = (parent.display()).to_string(),
                error = (error).to_string()
            )
        })?;
    }
    if let Some(parent) = auth_files.auth_url_file.parent() {
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| fl!("onedrive-auth-directory-private-failed"))?;
    }
    let _ = fs::remove_file(&auth_files.auth_url_file);
    let _ = fs::remove_file(&auth_files.response_url_file);
    Ok(())
}

#[allow(dead_code)]
fn cleanup_onedrive_auth_files(auth_files: &OneDriveMirrorAuthFiles) {
    let _ = fs::remove_file(&auth_files.auth_url_file);
    let _ = fs::remove_file(&auth_files.response_url_file);
}

fn onedrive_mirror_plan_for_app(
    connection: &Connection,
) -> Result<cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan, String> {
    one_drive_mirror_plan(
        connection,
        &default_config_root(),
        &OneDriveIsolationReport {
            active_onedriver_paths: Vec::new(),
        },
    )
    .map_err(|error| error.to_string())
}

async fn onedrive_mirror_pending_count(connection: &Connection) -> Result<u64, String> {
    if !is_legacy_onedrive_mirror(connection) {
        return Err(fl!("pending-count-request-failed"));
    }
    let confdir = default_config_root()
        .join("onedrive-sync")
        .join(connection.id.to_string());
    let mut request = CommandRequest::new(Executable::OneDrive)
        .arg("--confdir")
        .and_then(|request| request.arg(confdir.as_os_str()))
        .and_then(|request| request.arg("--syncdir"))
        .and_then(|request| request.arg(connection.local_path.as_os_str()))
        .and_then(|request| request.arg("--display-sync-status"))
        .map_err(|_| fl!("pending-count-request-failed"))?;
    if let Some(folder) = &connection.remote_subpath {
        request = request
            .arg("--single-directory")
            .and_then(|request| request.arg(folder))
            .map_err(|_| fl!("pending-count-request-failed"))?;
    }
    let output = app_command_runner()
        .run(
            request
                .with_timeout(Duration::from_secs(90))
                .with_output_limit(256 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|_| fl!("pending-count-mirror-unavailable"))?;
    if output.stdout.truncated || output.stdout.invalid_utf8 {
        return Err(fl!("pending-count-response-incomplete"));
    }
    pending_count::parse_onedrive_mirror_status(&output.stdout.text)
}

async fn rclone_mirror_live_pending_count(connection: &Connection) -> Result<u64, String> {
    let plan = rclone_bisync_plan(connection, &default_work_root())
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    let service = UnitName::new(connection.id, UnitKind::Service).file_name();
    pending_count::rclone_mirror_live_pending(&plan.work_directory, &service).await
}

fn prepare_rclone_bisync_work_files(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<(), String> {
    fs::create_dir_all(&plan.work_directory).map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-create-rclone-bisync-work-directory",
            value1 = (plan.work_directory.display()).to_string(),
            error = (error).to_string()
        )
    })?;
    let filter_content = rclone_bisync_filter_file(connection.provider);
    let staging = plan
        .filters_file
        .with_extension(format!("txt.tmp.{}", Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-stage-managed-mirror-filter-error",
                error = (error).to_string()
            )
        })?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| {
            crate::fl!(
                "ui-app-could-not-restrict-managed-mirror-filter-error",
                error = (error).to_string()
            )
        })?;
    file.write_all(filter_content.as_bytes()).map_err(|error| {
        crate::fl!(
            "ui-app-could-not-write-managed-mirror-filter-error",
            error = (error).to_string()
        )
    })?;
    file.sync_all().map_err(|error| {
        crate::fl!(
            "ui-app-could-not-sync-managed-mirror-filter-error",
            error = (error).to_string()
        )
    })?;
    fs::rename(&staging, &plan.filters_file).map_err(|error| {
        crate::fl!(
            "ui-app-could-not-install-managed-mirror-filter-error",
            error = (error).to_string()
        )
    })?;
    Ok(())
}

fn initial_preview_marker(plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan) -> PathBuf {
    plan.work_directory.join("initial-preview-confirmable")
}

fn initial_sync_marker(plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan) -> PathBuf {
    plan.work_directory.join("initial-sync-complete")
}

fn invalidate_sharepoint_mirror_confirmation_on_retarget(
    existing: &Connection,
    next: &Connection,
    work_root: &Path,
) -> Result<(), String> {
    if existing.provider != Provider::Teams
        || next.provider != Provider::Teams
        || !matches!(existing.mode, ConnectionMode::OfflineMirror(_))
        || !matches!(next.mode, ConnectionMode::OfflineMirror(_))
    {
        return Ok(());
    }
    let old_plan = rclone_bisync_plan(existing, work_root).map_err(|error| error.to_string())?;
    let new_plan = rclone_bisync_plan(next, work_root).map_err(|error| error.to_string())?;
    if old_plan.work_directory == new_plan.work_directory {
        return Ok(());
    }
    for marker in [
        initial_preview_marker(&old_plan),
        initial_sync_marker(&old_plan),
    ] {
        match fs::remove_file(marker) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(fl!(
                    "sharepoint-confirmation-invalidate-error",
                    error = error.to_string()
                ));
            }
        }
    }
    Ok(())
}

fn onedrive_initial_preview_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> PathBuf {
    plan.config_directory.join("initial-preview-confirmable")
}

fn onedrive_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> PathBuf {
    plan.config_directory.join("initial-sync-complete")
}

#[cfg(test)]
fn write_initial_preview_marker(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
    summary: &str,
) -> Result<(), String> {
    fs::write(
        initial_preview_marker(plan),
        format!("Cloud Mounter initial preview completed.\n{summary}\n"),
    )
    .map_err(|error| format!("failed to record initial preview completion: {error}"))
}

#[cfg(test)]
fn write_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<(), String> {
    fs::write(
        initial_sync_marker(plan),
        "Cloud Mounter initial sync completed.\n",
    )
    .map_err(|error| format!("failed to record initial sync completion: {error}"))
}

fn write_onedrive_initial_preview_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    summary: &str,
) -> Result<(), String> {
    fs::write(
        onedrive_initial_preview_marker(plan),
        format!("Cloud Mounter OneDrive initial preview completed.\n{summary}\n"),
    )
    .map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-record-onedrive-initial-preview-completion",
            error = (error).to_string()
        )
    })
}

fn write_onedrive_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<(), String> {
    fs::write(
        onedrive_initial_sync_marker(plan),
        "Cloud Mounter OneDrive initial sync completed.\n",
    )
    .map_err(|error| {
        crate::fl!(
            "ui-app-failed-to-record-onedrive-initial-sync-completion",
            error = (error).to_string()
        )
    })
}

fn preview_summary_text(output: &CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.text, output.stderr.text);
    let summary = parse_preview(&combined);
    crate::fl!(
        "ui-app-preview-uploads-value1-downloads-value2-deletes-value3",
        value1 = (summary.uploads).to_string(),
        value2 = (summary.downloads).to_string(),
        value3 = (summary.deletes).to_string(),
        value4 = (summary.conflicts).to_string(),
        value5 = (summary.skipped).to_string(),
        value6 = (summary
            .transfer_bytes
            .map(|bytes| fl!("preview-transfer-estimate", bytes = human_bytes(bytes)))
            .unwrap_or_default())
        .to_string(),
        value7 = (if summary.destructive {
            fl!("preview-destructive-changes")
        } else {
            String::new()
        })
        .to_string()
    )
}

fn onedrive_output_summary(stage: &str, output: &CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.text, output.stderr.text);
    let changed_lines = combined
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("upload")
                || lower.contains("download")
                || lower.contains("delete")
                || lower.contains("created")
                || lower.contains("updated")
                || lower.contains("renamed")
                || lower.contains("skipping")
        })
        .count();
    if changed_lines == 0 {
        crate::fl!(
            "ui-app-stage-completed-onedrive-did-not-report-file",
            stage = (stage).to_string()
        )
    } else {
        crate::fl!(
            "ui-app-stage-completed-onedrive-reported-changed-lines-notable",
            stage = (stage).to_string(),
            changed_lines = (changed_lines).to_string()
        )
    }
}

fn sync_output_summary(output: &CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.text, output.stderr.text);
    let summary = parse_preview(&combined);
    if summary.uploads + summary.downloads + summary.deletes + summary.conflicts + summary.skipped
        > 0
    {
        crate::fl!(
            "ui-app-sync-summary-uploads-value1-downloads-value2-deletes",
            value1 = (summary.uploads).to_string(),
            value2 = (summary.downloads).to_string(),
            value3 = (summary.deletes).to_string(),
            value4 = (summary.conflicts).to_string(),
            value5 = (summary.skipped).to_string()
        )
    } else {
        crate::fl!("ui-app-sync-completed-rclone-did-not-report-transfer")
    }
}

fn human_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= GIB {
        crate::fl!(
            "ui-app-value1-gib",
            value1 = format!("{:.1}", bytes as f64 / GIB as f64)
        )
    } else if bytes >= MIB {
        crate::fl!(
            "ui-app-value1-mib",
            value1 = format!("{:.1}", bytes as f64 / MIB as f64)
        )
    } else {
        crate::fl!("ui-app-bytes-bytes", bytes = (bytes).to_string())
    }
}

fn sync_rejection_message(rejection: SyncDecisionRejection) -> String {
    match rejection {
        SyncDecisionRejection::ConcurrentRun => fl!("sync-rejection-concurrent"),
        SyncDecisionRejection::PreviewRequired => {
            fl!("sync-rejection-preview")
        }
        SyncDecisionRejection::ConfirmationRequired => {
            fl!("sync-rejection-confirmation")
        }
        SyncDecisionRejection::ResyncPreviewRequired => {
            fl!("sync-rejection-resync")
        }
    }
}

#[cfg(test)]
fn offline_mirror_command_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        CommandError::InvalidArgument => "rclone command argument contains unsafe bytes".into(),
        CommandError::Timeout { timeout, .. } => {
            format!(
                "rclone {stage} timed out after {} seconds",
                timeout.as_secs()
            )
        }
        CommandError::Cancelled { .. } => format!("rclone {stage} was cancelled"),
        CommandError::Spawn { message, .. } => format!("could not start rclone: {message}"),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            let lower = detail.to_ascii_lowercase();
            if lower.contains("prior lock file found") {
                "Sync is locked. Retry after the active run or two minutes; older locks may need manual recovery.".into()
            } else if detail.is_empty() {
                format!("rclone {stage} failed without diagnostic output")
            } else if lower.contains("unauthorized")
                || lower.contains("forbidden")
                || lower.contains("authentication")
                || lower.contains("token")
            {
                format!(
                    "rclone {stage} failed because storage authentication or access was rejected"
                )
            } else if lower.contains("quota")
                || lower.contains("insufficient storage")
                || lower.contains("no space left")
            {
                format!(
                    "rclone {stage} failed because local or remote storage capacity is unavailable"
                )
            } else if lower.contains("checksum") || lower.contains("verification failed") {
                format!(
                    "rclone {stage} failed post-sync verification; review the app-owned mirror log before retrying"
                )
            } else {
                format!(
                    "rclone {stage} failed; review the app-owned mirror log and use Repair before retrying if the operation was interrupted"
                )
            }
        }
    }
}

fn onedrive_runtime_command_error(stage: &str, error: CommandError) -> String {
    onedrive_validation_command_error(stage, error)
}

async fn test_sharepoint_draft_connection(connection: &mut Connection) -> Result<String, String> {
    let verified = sharepoint::verify_remote(
        &app_command_runner(),
        &connection.remote_reference,
        connection
            .teams_identity
            .as_ref()
            .ok_or(crate::fl!("ui-app-sharepoint-library-url-is-missing"))?,
    )
    .await?;
    connection.teams_identity = Some(verified.identity.clone());
    test_connection_plan_and_access(connection, Some(&verified)).await
}

async fn test_connection_plan_and_access(
    connection: &Connection,
    teams_verified: Option<&sharepoint::VerifiedSharePointLibrary>,
) -> Result<String, String> {
    let plan_summary = match managed_plan_summary(connection) {
        Ok(summary) => summary,
        Err(error) => return Err(error),
    };

    ensure_vpn_ready_for_connection(connection).await?;

    if is_onedrive_rclone_mirror(connection) {
        let summary = verify_rclone_access(connection).await?;
        return Ok(format!("{plan_summary} {summary}"));
    }
    match connection.provider {
        Provider::Teams
        | Provider::GoogleDrive
        | Provider::Box
        | Provider::Smb
        | Provider::Sftp => {
            match verify_rclone_access_with_verified(connection, teams_verified).await {
                Ok(access_summary) => Ok(format!("{plan_summary} {access_summary}")),
                Err(error) => Err(error),
            }
        }
        Provider::OneDrive => match connection.mode {
            ConnectionMode::OnlineMount(_) => {
                match verify_onedriver_online_mount_setup(connection).await {
                    Ok(access_summary) => Ok(format!("{plan_summary} {access_summary}")),
                    Err(error) => Err(error),
                }
            }
            ConnectionMode::OfflineMirror(_) => {
                match verify_onedrive_offline_mirror_setup(connection).await {
                    Ok(access_summary) => Ok(format!("{plan_summary} {access_summary}")),
                    Err(error) => Err(error),
                }
            }
        },
    }
}

async fn validate_onedrive_connection_for_save(connection: &Connection) -> Result<String, String> {
    if is_onedrive_rclone_mirror(connection) {
        ensure_vpn_ready_for_connection(connection).await?;
        return verify_rclone_access(connection).await;
    }
    match connection.mode {
        ConnectionMode::OnlineMount(_) => verify_onedriver_online_mount_setup(connection).await,
        ConnectionMode::OfflineMirror(_) => verify_onedrive_offline_mirror_setup(connection).await,
    }
}

fn save_notice_name(name: &str, validation_summary: Option<&str>) -> String {
    match validation_summary {
        Some(summary) => crate::fl!(
            "ui-app-name-passed-validation-summary-and",
            name = (name).to_string(),
            summary = (summary).to_string()
        ),
        None => name.to_owned(),
    }
}

async fn verify_onedriver_online_mount_setup(connection: &Connection) -> Result<String, String> {
    let mounts = HostVisibleMountTable::observe().await?;
    verify_onedriver_online_mount_setup_with(
        connection,
        &app_command_runner(),
        &mounts,
        &default_cache_root(),
        &default_config_root(),
    )
}

fn verify_onedriver_online_mount_setup_with(
    connection: &Connection,
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    cache_root: &std::path::Path,
    config_root: &std::path::Path,
) -> Result<String, String> {
    if !is_onedriver_online_mount(connection) {
        return Err(crate::fl!(
            "ui-app-onedrive-online-mount-validation-only-applies-to"
        ));
    }
    if runner.resolve(Executable::Onedriver).is_none() {
        return Err(crate::fl!(
            "ui-app-onedriver-is-missing-install-jstaf-onedriver-0-2"
        ));
    }

    let plan = onedriver_mount_plan(connection, cache_root, config_root).map_err(|error| {
        crate::fl!(
            "ui-app-could-not-build-onedriver-plan-error",
            error = (error).to_string()
        )
    })?;
    if let Some(parent) = plan.mountpoint.parent()
        && parent.exists()
        && !parent.is_dir()
    {
        return Err(crate::fl!(
            "ui-app-mountpoint-parent-value1-is-not-a-directory",
            value1 = (parent.display()).to_string()
        ));
    }
    if plan.mountpoint.exists() && !plan.mountpoint.is_dir() {
        return Err(crate::fl!(
            "ui-app-mountpoint-value1-exists-but-is-not-a",
            value1 = (plan.mountpoint.display()).to_string()
        ));
    }

    let mount_entries = mount_table.entries().map_err(|error| {
        crate::fl!(
            "ui-app-could-not-inspect-active-mounts-error",
            error = (error).to_string()
        )
    })?;
    if let Some(entry) = conflicting_onedriver_mount(&plan.mountpoint, &mount_entries) {
        return Err(crate::fl!(
            "ui-app-mountpoint-value1-overlaps-an-active-onedriver-mount",
            value1 = (plan.mountpoint.display()).to_string(),
            value2 = (entry.target.display()).to_string()
        ));
    }

    match onedriver_auth_state_for_plan(&plan) {
        OnedriverAuthState::Unauthenticated => {
            return Err(crate::fl!(
                "ui-app-onedriver-is-not-authenticated-for-this-applet",
                value1 = (connection.name).to_string(),
                value2 = (plan.cache_directory.display()).to_string()
            ));
        }
        OnedriverAuthState::Authenticated { config_file } => {
            if config_file
                .metadata()
                .map(|metadata| metadata.len() == 0)
                .unwrap_or(true)
            {
                return Err(crate::fl!(
                    "ui-app-onedriver-account-metadata-at-value1-is-unavailable",
                    value1 = (config_file.display()).to_string()
                ));
            }
        }
    }

    Ok(crate::fl!(
        "ui-app-onedriver-setup-is-present-for-mountpoint-value1",
        value1 = (plan.mountpoint.display()).to_string(),
        value2 = (plan.cache_directory.display()).to_string()
    ))
}

fn conflicting_onedriver_mount<'a>(
    mountpoint: &std::path::Path,
    entries: &'a [MountEntry],
) -> Option<&'a MountEntry> {
    entries.iter().find(|entry| {
        entry.filesystem == "fuse.onedriver"
            && (paths_overlap(mountpoint, &entry.target)
                || paths_overlap(&entry.target, mountpoint))
    })
}

fn paths_overlap(left: &std::path::Path, right: &std::path::Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

async fn verify_onedrive_offline_mirror_setup(connection: &Connection) -> Result<String, String> {
    let mounts = HostVisibleMountTable::observe().await?;
    verify_onedrive_offline_mirror_setup_with(
        connection,
        &app_command_runner(),
        &mounts,
        &default_config_root(),
    )
    .await
}

async fn verify_onedrive_offline_mirror_setup_with(
    connection: &Connection,
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    config_root: &std::path::Path,
) -> Result<String, String> {
    if !is_legacy_onedrive_mirror(connection) {
        return Err(crate::fl!(
            "ui-app-onedrive-offline-mirror-validation-only-applies-to"
        ));
    }
    if runner.resolve(Executable::OneDrive).is_none() {
        return Err(crate::fl!(
            "ui-app-onedrive-is-missing-install-abraunegg-onedrive-2-2"
        ));
    }

    let mount_entries = mount_table.entries().map_err(|error| {
        crate::fl!(
            "ui-app-could-not-inspect-active-mounts-error",
            error = (error).to_string()
        )
    })?;
    let active_onedriver_paths = mount_entries
        .iter()
        .filter(|entry| entry.filesystem == "fuse.onedriver")
        .map(|entry| entry.target.clone())
        .collect();
    let plan = one_drive_mirror_plan(
        connection,
        config_root,
        &OneDriveIsolationReport {
            active_onedriver_paths,
        },
    )
    .map_err(onedrive_mirror_plan_validation_error)?;

    validate_directory_available(&crate::fl!("ui-app-sync-directory"), &plan.sync_directory)?;
    validate_directory_available(
        &crate::fl!("ui-app-onedrive-config-directory"),
        &plan.config_directory,
    )?;
    validate_directory_available(
        &crate::fl!("ui-app-recovery-directory"),
        &plan.recovery_directory,
    )?;

    let token_file = plan.config_directory.join("refresh_token");
    if token_file
        .metadata()
        .map(|metadata| metadata.len() == 0)
        .unwrap_or(true)
    {
        return Err(crate::fl!(
            "ui-app-abraunegg-onedrive-is-not-authenticated-for-this",
            value1 = (token_file.display()).to_string()
        ));
    }

    let output = runner
        .run(
            one_drive_preview_request(&plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| {
            onedrive_validation_command_error(&fl!("onedrive-stage-dry-preview"), error)
        })?;

    Ok(crate::fl!(
        "ui-app-abraunegg-onedrive-setup-is-authenticated-for-sync",
        value1 = (plan.sync_directory.display()).to_string(),
        value2 = onedrive_output_summary(&fl!("onedrive-stage-validation-preview"), &output)
    ))
}

fn validate_directory_available(label: &str, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && parent.exists()
        && !parent.is_dir()
    {
        return Err(crate::fl!(
            "ui-app-label-parent-value2-is-not-a-directory",
            label = (label).to_string(),
            value2 = (parent.display()).to_string()
        ));
    }
    if path.exists() && !path.is_dir() {
        return Err(crate::fl!(
            "ui-app-label-value2-exists-but-is-not-a",
            label = (label).to_string(),
            value2 = (path.display()).to_string()
        ));
    }
    Ok(())
}

fn onedrive_mirror_plan_validation_error(
    error: cosmic_ext_applet_mounter::sync::SyncError,
) -> String {
    match error {
        cosmic_ext_applet_mounter::sync::SyncError::OverlapsOnedriverPath(path) => crate::fl!(
            "ui-app-onedrive-offline-mirror-overlaps-active-onedriver-mount",
            value1 = (path.display()).to_string()
        ),
        cosmic_ext_applet_mounter::sync::SyncError::InvalidWorkDirectory => {
            crate::fl!("ui-app-onedrive-config-work-directory-must-be-absolute")
        }
        cosmic_ext_applet_mounter::sync::SyncError::InvalidRecoveryDirectory => {
            crate::fl!("ui-app-recovery-directory-must-be-absolute-and-separate")
        }
        cosmic_ext_applet_mounter::sync::SyncError::InvalidRemoteSubpath => {
            crate::fl!("ui-app-remote-subtree-contains-unsupported-characters")
        }
        other => other.to_string(),
    }
}

fn onedrive_validation_command_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            crate::fl!(
                "ui-app-value1-is-missing",
                value1 = (executable.display_name()).to_string()
            )
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-app-onedrive-command-argument-contains-unsupported-characters")
        }
        CommandError::Timeout { timeout, .. } => crate::fl!(
            "ui-app-onedrive-stage-timed-out-after-value2-seconds",
            stage = (stage).to_string(),
            value2 = (timeout.as_secs()).to_string()
        ),
        CommandError::Cancelled { .. } => crate::fl!(
            "ui-app-onedrive-stage-was-cancelled",
            stage = (stage).to_string()
        ),
        CommandError::Spawn { message, .. } => crate::fl!(
            "ui-app-could-not-start-onedrive-message",
            message = (message).to_string()
        ),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            let lower = detail.to_ascii_lowercase();
            if detail.is_empty() {
                crate::fl!(
                    "ui-app-onedrive-stage-failed-without-diagnostic-output",
                    stage = (stage).to_string()
                )
            } else if lower.contains("authorization is required")
                || lower.contains("authorisation is required")
                || lower.contains("requires authorisation")
                || lower.contains("requires authorization")
                || lower.contains("application authorisation cannot be completed")
                || lower.contains("application authorization cannot be completed")
                || lower.contains("not authenticated")
                || lower.contains("reauth")
                || lower.contains("refresh_token")
            {
                crate::fl!(
                    "ui-app-abraunegg-onedrive-is-not-authenticated-for-this-2",
                    detail = (detail).to_string()
                )
            } else if lower.contains("code has expired")
                || lower.contains("code is not valid")
                || lower.contains("invalid_grant")
                || lower.contains("invalid_request")
                || lower.contains("auth-response")
            {
                crate::fl!(
                    "ui-app-onedrive-oauth-response-is-expired-or-invalid",
                    detail = (detail).to_string()
                )
            } else if lower.contains("aadsts")
                || lower.contains("admin consent")
                || lower.contains("tenant")
                || lower.contains("consent")
            {
                crate::fl!(
                    "ui-app-microsoft-tenant-or-admin-consent-policy-blocked",
                    detail = (detail).to_string()
                )
            } else if lower.contains("resync")
                || lower.contains("application configuration change")
                || lower.contains("sync state")
            {
                crate::fl!(
                    "ui-app-abraunegg-onedrive-requires-a-preview-and-explicit",
                    detail = (detail).to_string()
                )
            } else if lower.contains("does not exist online")
                || lower.contains("requested path")
                || lower.contains("single-directory")
            {
                crate::fl!(
                    "ui-app-selected-onedrive-remote-subtree-is-not-accessible",
                    detail = (detail).to_string()
                )
            } else if lower.contains("network")
                || lower.contains("timeout")
                || lower.contains("connection")
                || lower.contains("dns")
                || lower.contains("host")
            {
                crate::fl!(
                    "ui-app-network-or-vpn-readiness-failed-while-checking",
                    detail = (detail).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-onedrive-stage-failed-detail",
                    stage = (stage).to_string(),
                    detail = (detail).to_string()
                )
            }
        }
    }
}

fn onedrive_auth_files_completion_error(error: &CommandError) -> bool {
    let CommandError::NonZero { stderr, stdout, .. } = error else {
        return false;
    };
    let detail = if stderr.text.trim().is_empty() {
        stdout.text.trim()
    } else {
        stderr.text.trim()
    }
    .to_ascii_lowercase();
    detail.contains("missing either the \"--sync\" or \"--monitor\"")
        || detail.contains("missing either the '--sync' or '--monitor'")
}

async fn verify_rclone_access(connection: &Connection) -> Result<String, String> {
    verify_rclone_access_with_verified(connection, None).await
}

async fn verify_rclone_access_with_verified(
    connection: &Connection,
    teams_verified: Option<&sharepoint::VerifiedSharePointLibrary>,
) -> Result<String, String> {
    if matches!(connection.mode, ConnectionMode::OfflineMirror(_)) {
        cosmic_ext_applet_mounter::mirror_access::LocalMirrorPaths::resolve(
            connection,
            CancellationToken::new(),
        )
        .await?;
    }
    if is_onedrive_rclone_mirror(connection) {
        // Validation is read-only and also applies to disabled drafts. Run guards
        // still require the authoritative saved connection to be enabled.
        let mut candidate = connection.clone();
        candidate.enabled = true;
        let binding =
            cosmic_ext_applet_mounter::onedrive::MirrorGuardBinding::from_connection(&candidate)
                .map_err(|error| error.to_string())?;
        cosmic_ext_applet_mounter::onedrive::verify_mirror_binding(
            &binding,
            &candidate,
            &app_command_runner(),
            &cosmic_ext_applet_mounter::onedrive::MicrosoftGraph::new()
                .map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| error.to_string())?;
        return Ok(fl!("onedrive-runtime-remote-verified"));
    }
    let teams_summary = if connection.provider == Provider::Teams {
        let verified = match teams_verified {
            Some(verified) => verified.clone(),
            None => {
                sharepoint::verify_remote(
                    &app_command_runner(),
                    &connection.remote_reference,
                    connection.teams_identity.as_ref().ok_or(crate::fl!(
                        "ui-app-sharepoint-library-identity-is-missing-edit-and"
                    ))?,
                )
                .await?
            }
        };
        let account = verified
            .account
            .unwrap_or_else(|| crate::fl!("ui-app-account-unavailable-from-microsoft-graph"));
        Some(crate::fl!(
            "ui-app-sharepoint-library-value1-at-site-value2-and",
            value1 = (verified.identity.library_url).to_string(),
            value2 = (verified.identity.site_url).to_string(),
            value3 = (verified.identity.drive_id).to_string(),
            account = (account).to_string()
        ))
    } else {
        None
    };
    if connection.provider == Provider::Sftp {
        sftp::verify_host_verification(&app_command_runner(), &connection.remote_reference).await?;
    }
    let expected_backend = rclone_backend_name(connection.provider)
        .ok_or_else(|| crate::fl!("ui-app-selected-provider-does-not-use-rclone"))?;
    let provider = CommandRcloneProvider::new(app_command_runner());
    provider
        .validate_remote(connection, CancellationToken::new())
        .await
        .map_err(|error| rclone_remote_validation_error(connection, error))?;

    let target = rclone_access_target(connection);
    let output = app_command_runner()
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("lsf")
                .map_err(|error| error.to_string())?
                .arg(target.clone())
                .map_err(|error| error.to_string())?
                .arg("--max-depth")
                .map_err(|error| error.to_string())?
                .arg("1")
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(20))
                .with_output_limit(16 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| {
            if connection.provider == Provider::Sftp {
                sftp_access_error(error)
            } else if connection.provider == Provider::Teams {
                sharepoint::rclone_read_error(&error, connection.remote_subpath.is_some())
            } else {
                rclone_access_error(&target, error)
            }
        })?;
    let visible_items = output.stdout.text.lines().count();
    Ok(crate::fl!(
        "ui-app-rclone-remote-value1-exists-backend-expected-backend",
        value1 = (connection.remote_reference).to_string(),
        expected_backend = (expected_backend).to_string(),
        value3 = (provider_label(connection.provider)).to_string(),
        target = (target).to_string(),
        visible_items = (visible_items).to_string()
    ) + &teams_summary.unwrap_or_default())
}

fn rclone_access_target(connection: &Connection) -> String {
    match connection.remote_subpath.as_deref() {
        Some(subpath) if !subpath.trim().is_empty() => {
            format!("{}:{}", connection.remote_reference, subpath.trim())
        }
        _ => format!("{}:", connection.remote_reference),
    }
}

fn rclone_remote_validation_error(connection: &Connection, error: ProviderError) -> String {
    match error {
        ProviderError::InvalidRemoteReference => crate::fl!(
            "ui-app-rclone-remote-value1-was-not-found-run",
            value1 = (connection.remote_reference).to_string()
        ),
        ProviderError::UnsupportedProvider(_) => crate::fl!(
            "ui-app-rclone-remote-value1-exists-but-does-not",
            value1 = (connection.remote_reference).to_string(),
            value2 = rclone_backend_name(connection.provider)
                .map(str::to_owned)
                .unwrap_or_else(|| fl!("value-unknown")),
            value3 = (provider_label(connection.provider)).to_string()
        ),
        ProviderError::MissingExecutable(executable) => {
            crate::fl!(
                "ui-app-value1-is-missing",
                value1 = (executable.display_name()).to_string()
            )
        }
        ProviderError::Command(error) => {
            rclone_access_error(&fl!("rclone-stage-config-dump"), error)
        }
        ProviderError::InvalidResponse(message) => {
            crate::fl!(
                "ui-app-rclone-config-dump-returned-an-unexpected-response",
                message = (message).to_string()
            )
        }
        ProviderError::InvalidMode => {
            crate::fl!("ui-app-selected-connection-mode-is-invalid-for-rclone")
        }
        ProviderError::InvalidRemoteSubpath => {
            crate::fl!("ui-app-remote-subtree-contains-unsupported-characters-2")
        }
        ProviderError::InvalidSharePointIdentity(message) => message,
        ProviderError::InvalidMountpointGuard(message) => message,
        ProviderError::Unauthenticated => {
            crate::fl!("ui-app-rclone-remote-is-not-authenticated-reauthorize-it")
        }
    }
}

fn rclone_access_error(target: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            crate::fl!(
                "ui-app-value1-is-missing",
                value1 = (executable.display_name()).to_string()
            )
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-app-rclone-command-argument-contains-unsupported-characters")
        }
        CommandError::Timeout { timeout, .. } => crate::fl!(
            "ui-app-read-only-rclone-access-check-for-target",
            target = (target).to_string(),
            value2 = (timeout.as_secs()).to_string()
        ),
        CommandError::Cancelled { .. } => crate::fl!("ui-app-rclone-access-check-was-cancelled"),
        CommandError::Spawn { message, .. } => crate::fl!(
            "ui-app-could-not-start-rclone-message",
            message = (message).to_string()
        ),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            let lower = detail.to_ascii_lowercase();
            if lower.contains("directory not found")
                || lower.contains("object not found")
                || lower.contains("not found")
                || lower.contains("doesn't exist")
            {
                crate::fl!(
                    "ui-app-remote-subtree-target-is-not-accessible-or",
                    target = (target).to_string(),
                    detail = (detail).to_string()
                )
            } else if lower.contains("auth")
                || lower.contains("token")
                || lower.contains("unauthorized")
                || lower.contains("forbidden")
                || lower.contains("permission")
            {
                crate::fl!(
                    "ui-app-rclone-remote-for-target-is-not-authorized",
                    target = (target).to_string(),
                    detail = (detail).to_string()
                )
            } else if lower.contains("network")
                || lower.contains("connection")
                || lower.contains("timeout")
                || lower.contains("no route")
                || lower.contains("host")
                || lower.contains("dns")
            {
                crate::fl!(
                    "ui-app-network-or-vpn-readiness-failed-while-checking-2",
                    target = (target).to_string(),
                    detail = (detail).to_string()
                )
            } else {
                crate::fl!(
                    "ui-app-read-only-rclone-access-check-failed-for",
                    target = (target).to_string(),
                    detail = (detail).to_string()
                )
            }
        }
    }
}

fn sftp_access_error(error: CommandError) -> String {
    // Rclone diagnostics may include paths or credential-bearing option text.
    // Match known signatures, but never display the raw command output.
    match error {
        CommandError::MissingExecutable(_) => {
            crate::fl!("ui-app-rclone-is-missing-install-it-before-testing")
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-app-sftp-access-request-contains-unsupported-characters")
        }
        CommandError::Timeout { .. } => {
            crate::fl!("ui-app-sftp-access-timed-out-check-the-server")
        }
        CommandError::Cancelled { .. } => crate::fl!("ui-app-sftp-access-check-was-cancelled"),
        CommandError::Spawn { .. } => crate::fl!("ui-app-could-not-start-rclone-for-the-sftp"),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.as_str()
            } else {
                stderr.text.as_str()
            };
            let lower = detail.to_ascii_lowercase();
            if lower.contains("known_hosts_file")
                && (lower.contains("no such file")
                    || lower.contains("permission denied")
                    || lower.contains("couldn't parse")
                    || lower.contains("cannot read"))
            {
                crate::fl!("ui-app-sftp-known-hosts-file-is-missing-unreadable")
            } else if lower.contains("knownhosts:")
                || lower.contains("host key")
                || lower.contains("hostkey")
                || lower.contains("remote host identification has changed")
                || lower.contains("no authorities for hostname")
            {
                crate::fl!("ui-app-sftp-host-key-verification-failed-verify-the")
            } else if (lower.contains("key_file") || lower.contains("private key"))
                && (lower.contains("no such file")
                    || lower.contains("permission denied")
                    || lower.contains("cannot read")
                    || lower.contains("failed to load"))
            {
                crate::fl!("ui-app-sftp-private-key-file-is-missing-or")
            } else if lower.contains("ssh_auth_sock")
                || lower.contains("ssh agent")
                || lower.contains("ssh-agent")
            {
                crate::fl!("ui-app-sftp-ssh-agent-is-unavailable-or-has")
            } else if lower.contains("i/o timeout")
                || lower.contains("connection timed out")
                || lower.contains("context deadline exceeded")
            {
                crate::fl!("ui-app-sftp-connection-timed-out-check-the-server")
            } else if lower.contains("unable to authenticate")
                || lower.contains("authentication failed")
                || lower.contains("no supported methods remain")
                || lower.contains("too many authentication failures")
                || (lower.contains("handshake") && lower.contains("permission denied"))
            {
                crate::fl!("ui-app-sftp-authentication-failed-check-the-username-and")
            } else if lower.contains("permission denied")
                || lower.contains("access denied")
                || lower.contains("operation not permitted")
            {
                crate::fl!("ui-app-sftp-login-succeeded-but-the-selected-directory")
            } else if lower.contains("directory not found")
                || lower.contains("object not found")
                || lower.contains("path not found")
                || lower.contains("no such file")
                || lower.contains("not a directory")
                || lower.contains("doesn't exist")
            {
                crate::fl!("ui-app-sftp-directory-was-not-found-check-the")
            } else if lower.contains("connection refused")
                || lower.contains("no route to host")
                || lower.contains("network is unreachable")
                || lower.contains("no such host")
                || lower.contains("dial tcp")
            {
                crate::fl!("ui-app-sftp-server-could-not-be-reached-check")
            } else {
                crate::fl!("ui-app-sftp-access-check-failed-check-the-server")
            }
        }
    }
}

fn import_preview_summary(preview: &ImportPreview) -> String {
    let status = if preview.active_conflict || preview.local_target_conflict {
        fl!("import-state-blocked")
    } else {
        fl!("import-state-ready")
    };
    let unsupported = if preview.unsupported_options.is_empty() {
        fl!("value-none")
    } else {
        preview.unsupported_options.join(", ")
    };
    let subtree = preview.remote_subpath.clone().unwrap_or_else(|| {
        if preview.provider == Provider::Sftp {
            fl!("import-sftp-login-directory")
        } else {
            fl!("import-whole-remote")
        }
    });
    crate::fl!(
        "ui-app-value1-value2-value3-remote-subtree-value4-start",
        value1 = (provider_label(preview.provider)).to_string(),
        value2 = (preview.remote_reference).to_string(),
        value3 = (preview.local_target.display()).to_string(),
        value4 = (subtree).to_string(),
        value5 = (yes_no(preview.start_at_login)).to_string(),
        unsupported = (unsupported).to_string(),
        status = (status).to_string()
    )
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn running_in_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

fn host_visible_home_dir() -> Option<PathBuf> {
    home_dir()
}

fn default_cache_root() -> PathBuf {
    default_cache_root_for(running_in_flatpak(), host_visible_home_dir())
}

fn default_cache_root_for(flatpak: bool, host_home: Option<PathBuf>) -> PathBuf {
    if flatpak {
        return host_home
            .map(|home| home.join(".cache"))
            .unwrap_or_else(std::env::temp_dir)
            .join("cosmic-ext-applet-mounter");
    }
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

fn default_config_root() -> PathBuf {
    default_config_root_for(running_in_flatpak(), host_visible_home_dir())
}

fn default_config_root_for(flatpak: bool, host_home: Option<PathBuf>) -> PathBuf {
    if flatpak {
        return host_home
            .map(|home| home.join(".config"))
            .unwrap_or_else(std::env::temp_dir)
            .join("cosmic-ext-applet-mounter");
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

fn default_work_root() -> PathBuf {
    default_work_root_for(running_in_flatpak(), host_visible_home_dir())
}

fn default_work_root_for(flatpak: bool, host_home: Option<PathBuf>) -> PathBuf {
    if flatpak {
        return host_home
            .map(|home| home.join(".local/state"))
            .unwrap_or_else(std::env::temp_dir)
            .join("cosmic-ext-applet-mounter");
    }
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".local/state")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

fn default_runtime_root() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

#[allow(dead_code)]
fn provider_engine_summary(provider: Provider, mode: AccessMode) -> &'static str {
    match (provider, mode) {
        (Provider::OneDrive, AccessMode::OnlineMount) => "onedriver",
        (Provider::OneDrive, AccessMode::OfflineMirror) => "abraunegg/onedrive",
        (_, AccessMode::OnlineMount) => "rclone mount",
        (_, AccessMode::OfflineMirror) => "rclone bisync",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::Application;
    use cosmic_ext_applet_mounter::process::FakeCommandRunner;

    #[test]
    fn onedrive_configuration_editor_round_trip_preserves_engine_and_identity() {
        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: Some(OfflineEngine::Rclone),
            recovery_directory: "/home/example/Recovery/test".into(),
            ..OfflineMirrorConfig::default()
        });
        connection.onedrive_identity = Some(OneDriveIdentity {
            drive_id: "synthetic-personal-drive".into(),
            drive_type: cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
        });
        let restored = connection_from_draft(&draft_from_connection(&connection)).unwrap();
        assert_eq!(restored.mode, connection.mode);
        assert_eq!(restored.onedrive_identity, connection.onedrive_identity);

        if let ConnectionMode::OfflineMirror(options) = &mut connection.mode {
            options.engine = None;
        }
        connection.onedrive_identity = None;
        let restored = connection_from_draft(&draft_from_connection(&connection)).unwrap();
        assert_eq!(restored.mode, connection.mode);
        assert_eq!(
            restored.offline_engine(),
            Some(OfflineEngine::LegacyOnedrive)
        );
    }

    #[test]
    fn onedrive_configuration_new_editor_mirror_explicitly_selects_rclone() {
        let connection = connection_from_draft(&ConnectionDraft {
            name: "New mirror".into(),
            provider: Provider::OneDrive,
            access_mode: AccessMode::OfflineMirror,
            remote_reference: "synthetic-remote".into(),
            local_path: "/home/example/Cloud/new".into(),
            ..ConnectionDraft::default()
        })
        .unwrap();
        assert_eq!(connection.offline_engine(), Some(OfflineEngine::Rclone));
        // The later setup step must verify identity before this can be saved.
        let mut config = Config::default();
        config.document.connections.push(connection);
        assert!(config.validate().is_err());
    }

    #[test]
    fn onedrive_configuration_preallocated_draft_id_does_not_select_legacy() {
        let connection = connection_from_draft(&ConnectionDraft {
            id: Some(ConnectionId::new()),
            name: "New mirror".into(),
            provider: Provider::OneDrive,
            access_mode: AccessMode::OfflineMirror,
            remote_reference: "synthetic-remote".into(),
            local_path: "/home/example/Cloud/new".into(),
            ..ConnectionDraft::default()
        })
        .unwrap();
        assert_eq!(connection.offline_engine(), Some(OfflineEngine::Rclone));
    }

    fn cached_status_app(connection: Connection) -> AppModel {
        let mut config = Config::default();
        config.document.connections.push(connection.clone());
        AppModel {
            config,
            unit_observation_connections: vec![connection.clone()],
            unit_observed_at: Some(Instant::now()),
            unit_observation: UnitObservation {
                services: [(connection.id, repair_unit_status(ActiveState::Active, true))].into(),
                ..UnitObservation::default()
            },
            ..AppModel::default()
        }
    }

    #[test]
    fn popup_snapshot_uses_cached_units_and_warnings_without_backend_io() {
        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let id = connection.id;
        let mut app = cached_status_app(connection);
        app.unit_observation.warnings = Some([(id, vec!["cached warning".into()])].into());
        let snapshot = app.controller_snapshot();
        assert_eq!(snapshot.service_status[&id].active, ActiveState::Active);
        assert_eq!(snapshot.sync_state[&id], SyncRuntimeState::Idle);
        assert_eq!(snapshot.offline_warnings[&id], vec!["cached warning"]);
        assert!(app.unit_status_fresh(id));
    }

    fn paused_rclone_status_app(provider: Provider) -> AppModel {
        let mut connection = test_connection(provider);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: Some(OfflineEngine::Rclone),
            ..OfflineMirrorConfig::default()
        });
        let id = connection.id;
        let mut app = cached_status_app(connection);
        app.unit_observation
            .services
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        app.unit_observation
            .timers
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        app
    }

    #[test]
    fn popup_coordinator_failure_exposes_repair_for_paused_rclone_mirrors() {
        for provider in [
            Provider::OneDrive,
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let mut app = paused_rclone_status_app(provider);
            let id = app.config.document.connections[0].id;
            assert!(app.controller_snapshot().paused_syncs.contains(&id));
            for state in [
                OfflineOperationState::Failed,
                OfflineOperationState::Cancelled,
            ] {
                app.unit_observation.coordinator_states.insert(id, state);
                assert_eq!(
                    app.controller_snapshot().sync_state[&id],
                    SyncRuntimeState::Failed
                );
                assert!(!app.controller_snapshot().paused_syncs.contains(&id));
                let row = &app.view_state().rows[0];
                assert!(repair_available(row));
                assert!(!decide_operation(row, Operation::SyncNow).allowed);
            }
            app.unit_observation
                .services
                .insert(id, repair_unit_status(ActiveState::Active, false));
            assert_eq!(
                app.controller_snapshot().sync_state[&id],
                SyncRuntimeState::Running
            );
            app.unit_observation
                .services
                .insert(id, repair_unit_status(ActiveState::Inactive, false));
            app.mirror_operations_pending.insert(id);
            assert_eq!(
                app.controller_snapshot().sync_state[&id],
                SyncRuntimeState::Running
            );
        }
    }

    #[test]
    fn popup_coordinator_repair_observation_clears_failure_but_ignores_stale_reply() {
        let mut app = paused_rclone_status_app(Provider::OneDrive);
        let connection = app.config.document.connections[0].clone();
        let id = connection.id;
        app.unit_observation
            .coordinator_states
            .insert(id, OfflineOperationState::Failed);
        app.unit_observation_revision = 2;
        let repaired = || UnitObservation {
            services: [(id, repair_unit_status(ActiveState::Inactive, false))].into(),
            timers: [(id, repair_unit_status(ActiveState::Inactive, false))].into(),
            coordinator_states: [(id, OfflineOperationState::Paused)].into(),
            ..UnitObservation::default()
        };
        let _ = app.update_message(Message::UnitObservationCompleted(
            1,
            vec![connection.clone()],
            repaired(),
        ));
        assert!(repair_available(&app.view_state().rows[0]));
        let _ = app.update_message(Message::UnitObservationCompleted(
            2,
            vec![connection],
            repaired(),
        ));
        assert!(app.controller_snapshot().paused_syncs.contains(&id));
        assert!(!repair_available(&app.view_state().rows[0]));
    }

    #[test]
    fn popup_coordinator_observation_rejects_foreign_and_malformed_state_per_connection() {
        use cosmic_ext_applet_mounter::offline_coordinator::{state_path, write_state};
        let root = tempfile::tempdir().unwrap();
        let connections = (0..4)
            .map(|_| {
                let mut connection = test_connection(Provider::Smb);
                connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
                connection
            })
            .collect::<Vec<_>>();
        let valid = &connections[0];
        write_state(
            &state_path(root.path(), valid.id),
            &OfflineCoordinatorState::new(
                valid.id,
                OfflineOperation::Sync,
                OfflineOperationState::Failed,
            )
            .with_warnings(vec!["synthetic warning".into()]),
        )
        .unwrap();
        write_state(
            &state_path(root.path(), connections[1].id),
            &OfflineCoordinatorState::new(
                ConnectionId::new(),
                OfflineOperation::Sync,
                OfflineOperationState::Failed,
            ),
        )
        .unwrap();
        let malformed = state_path(root.path(), connections[2].id);
        std::fs::create_dir_all(malformed.parent().unwrap()).unwrap();
        std::fs::write(malformed, b"malformed state").unwrap();
        let result = observe_offline_state_files(&connections, root.path());
        assert_eq!(result.states[&valid.id], OfflineOperationState::Failed);
        assert_eq!(result.warnings[&valid.id], ["synthetic warning"]);
        assert_eq!(result.failed, [connections[1].id, connections[2].id].into());
        assert!(!result.states.contains_key(&connections[3].id));
    }

    #[test]
    fn failed_observation_keeps_last_state_but_cannot_authorize_controls() {
        let connection = test_connection(Provider::Sftp);
        let id = connection.id;
        let mut app = cached_status_app(connection.clone());
        let failed = UnitObservation {
            failed: [id].into(),
            ..UnitObservation::default()
        };
        let _ = app.update_message(Message::UnitObservationCompleted(
            0,
            vec![connection],
            failed,
        ));
        assert_eq!(
            app.unit_observation.services[&id].active,
            ActiveState::Active
        );
        assert!(!app.unit_status_fresh(id));
        assert!(!app.unit_observation_pending);
    }

    #[test]
    fn cached_timer_and_finite_service_keep_offline_control_semantics() {
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let id = connection.id;
        let mut app = cached_status_app(connection);
        app.unit_observation
            .services
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        app.unit_observation
            .timers
            .insert(id, repair_unit_status(ActiveState::Active, true));
        assert_eq!(
            app.controller_snapshot().sync_state[&id],
            SyncRuntimeState::Idle
        );
        app.unit_observation
            .timers
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        assert!(app.controller_snapshot().paused_syncs.contains(&id));
        app.unit_observation
            .services
            .insert(id, repair_unit_status(ActiveState::Active, false));
        assert_eq!(
            app.controller_snapshot().sync_state[&id],
            SyncRuntimeState::Running
        );
        assert!(!app.controller_snapshot().paused_syncs.contains(&id));
        app.mirror_operations_pending.insert(id);
        app.unit_observation
            .services
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        assert_eq!(
            app.controller_snapshot().sync_state[&id],
            SyncRuntimeState::Running
        );
    }

    #[test]
    fn unobserved_offline_units_are_unknown_not_paused() {
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let id = connection.id;
        let mut app = cached_status_app(connection.clone());
        app.unit_observation.services.clear();
        assert_eq!(
            app.cached_offline_state(&connection),
            SyncRuntimeState::Unknown
        );
        assert!(!app.controller_snapshot().paused_syncs.contains(&id));
        app.unit_observation
            .services
            .insert(id, repair_unit_status(ActiveState::Inactive, false));
        assert_eq!(
            app.cached_offline_state(&connection),
            SyncRuntimeState::Unknown
        );
    }

    #[test]
    fn stale_generation_does_not_clear_new_pending_refresh() {
        let connection = test_connection(Provider::Sftp);
        let mut app = cached_status_app(connection.clone());
        app.unit_observation_revision = 2;
        app.unit_observation_pending = true;
        let _ = app.update_message(Message::UnitObservationCompleted(
            1,
            vec![connection],
            UnitObservation::default(),
        ));
        assert!(app.unit_observation_pending);
        assert_eq!(app.unit_observation_revision, 2);
    }

    #[test]
    fn stale_configuration_and_expired_observations_require_refresh() {
        let connection = test_connection(Provider::Sftp);
        let id = connection.id;
        let mut app = cached_status_app(connection.clone());
        app.unit_observed_at = Some(Instant::now() - Duration::from_secs(16));
        assert!(!app.unit_status_fresh(id));
        app.config.document.connections[0].local_path = PathBuf::from("/tmp/new-target");
        let _ = app.update_message(Message::UnitObservationCompleted(
            0,
            vec![connection],
            UnitObservation::default(),
        ));
        assert!(app.unit_observation_invalidated);
        assert!(app.unit_observation_pending);
        assert_eq!(app.unit_observation_revision, 1);
    }

    #[test]
    fn controls_wait_for_fresh_state_and_reject_changed_identity() {
        let connection = test_connection(Provider::Sftp);
        let id = connection.id;
        let mut app = cached_status_app(connection.clone());
        app.unit_observed_at = None;
        let _ = app.record_operation_request(id, Operation::Mount);
        assert!(app.unit_operations_waiting.contains_key(&id));
        assert!(app.unit_observation_pending);
        app.config.document.connections[0].remote_reference = "edited".into();
        let current = app.config.document.connections.clone();
        let observation = UnitObservation {
            services: [(id, repair_unit_status(ActiveState::Inactive, false))].into(),
            ..UnitObservation::default()
        };
        let _ = app.update_message(Message::UnitObservationCompleted(1, current, observation));
        assert!(app.unit_operations_waiting.is_empty());
        assert_eq!(
            app.last_notice,
            Some(fl!("unit-status-refresh-unavailable"))
        );
        assert!(app.mirror_operations_pending.is_empty());
    }

    #[test]
    fn force_refresh_cancels_old_work_and_retains_cached_data() {
        let connection = test_connection(Provider::Sftp);
        let id = connection.id;
        let mut app = cached_status_app(connection);
        let old = app.unit_observation_cancellation.clone();
        let _ = app.refresh_unit_observation(true);
        assert!(old.is_cancelled());
        assert!(!app.unit_status_fresh(id));
        assert_eq!(
            app.unit_observation.services[&id].active,
            ActiveState::Active
        );
        let revision = app.unit_observation_revision;
        let _ = app.refresh_unit_observation(false);
        assert_eq!(revision, app.unit_observation_revision);
    }

    #[test]
    fn inspection_late_discovery_cannot_replace_newer_list_or_notice() {
        let mut app = AppModel {
            rclone_inspection_generation: 2,
            rclone_remotes: vec![RcloneDraftRemote {
                name: "new".into(),
                backend: "smb".into(),
                drive_type: None,
            }],
            last_notice: Some("new notice".into()),
            ..AppModel::default()
        };
        let _ = app.update_message(Message::RcloneInspectionCompleted(
            1,
            true,
            Err("old failure".into()),
        ));
        assert_eq!(app.rclone_remotes[0].name, "new");
        assert_eq!(app.last_notice.as_deref(), Some("new notice"));
        let _ = app.update_message(Message::RcloneInspectionCompleted(2, false, Ok(Vec::new())));
        assert!(app.rclone_remotes.is_empty());
        assert_eq!(app.last_notice.as_deref(), Some("new notice"));
    }

    #[test]
    fn inspection_prefill_preserves_user_edits_and_remote_identity() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Smb;
        app.draft.remote_reference = "first".into();
        app.prefill_generation = 1;
        let context = app.prefill_context();
        app.draft.smb_host = "edited".into();
        let result = RemotePrefillResult {
            smb: Some(SmbRemoteDetails {
                host: "old".into(),
                user: "user".into(),
                domain: "domain".into(),
            }),
            sftp: None,
        };
        let _ = app.update_message(Message::RemotePrefillCompleted(
            1,
            context.clone(),
            Ok(result.clone()),
        ));
        assert_eq!(app.draft.smb_host, "edited");
        app.draft.smb_host.clear();
        app.draft.remote_reference = "second".into();
        let _ = app.update_message(Message::RemotePrefillCompleted(1, context, Ok(result)));
        assert!(app.draft.smb_host.is_empty());
    }

    #[test]
    fn inspection_sftp_prefill_preserves_newer_settings_revision() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        app.prefill_generation = 1;
        let context = app.prefill_context();
        app.mark_sftp_settings_changed();
        let _ = app.update_message(Message::RemotePrefillCompleted(
            1,
            context,
            Err("old failure".into()),
        ));
        assert!(app.sftp_settings_dirty);
        assert!(app.last_notice.is_none());
    }

    #[test]
    fn inspection_mount_failure_and_obsolete_result_preserve_last_observation() {
        let mut app = AppModel::default();
        let entry = MountEntry {
            target: PathBuf::from("/tmp/test"),
            source: "remote:".into(),
            filesystem: "fuse.rclone".into(),
            options: vec![],
        };
        app.mount_entries = vec![entry.clone()];
        let _ = app.update_message(Message::MountInspectionCompleted(Err("timeout".into())));
        assert_eq!(app.mount_entries, vec![entry.clone()]);
        app.mount_inspection_obsolete = true;
        let _ = app.update_message(Message::MountInspectionCompleted(Ok(vec![])));
        assert_eq!(app.mount_entries, vec![entry]);
        assert!(app.mount_inspection_pending);
        let _ = app.update_message(Message::MountInspectionCompleted(Ok(vec![])));
        assert!(app.mount_entries.is_empty());
        assert!(app.mount_inspection_error.is_none());
    }

    #[test]
    fn inspection_new_requests_cancel_superseded_work_without_running_on_gui() {
        let mut app = AppModel::default();
        let first = app.rclone_inspection_cancellation.clone();
        let _ = app.detect_rclone_remotes(true);
        assert!(first.is_cancelled());
        assert!(!app.rclone_inspection_cancellation.is_cancelled());
        app.draft.provider = Provider::Smb;
        let first = app.prefill_cancellation.clone();
        let _ = app.remote_prefill();
        assert!(first.is_cancelled());
        assert!(!app.prefill_cancellation.is_cancelled());
        // No executor/process/D-Bus is required to prepare these GUI tasks.
    }
    use cosmic_ext_applet_mounter::config::HostVisibleConfigStorage;
    use cosmic_ext_applet_mounter::model::{OfflineMirrorStatus, OnlineMountStatus};
    use cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile;

    #[tokio::test]
    async fn refresh_completion_preserves_operation_notice_and_pending_action() {
        use cosmic::Application;
        let connection_id = ConnectionId::from_uuid(Uuid::new_v4());
        let mut app = AppModel {
            last_notice: Some("Mount failed: access denied".into()),
            pending_repair: Some((connection_id, Instant::now())),
            ..AppModel::default()
        };
        let (reply, receiver) = runtime_ipc::Reply::channel();
        let _ = app.update(Message::RuntimeRefreshCompleted(reply, BTreeMap::new()));
        assert!(receiver.await.unwrap().is_ok());
        assert_eq!(
            app.last_notice.as_deref(),
            Some("Mount failed: access denied")
        );
        assert_eq!(
            app.pending_repair.map(|(pending_id, _)| pending_id),
            Some(connection_id)
        );
    }

    #[test]
    fn repair_confirmation_rejects_duplicate_first_click_and_expires() {
        let connection_id = ConnectionId::from_uuid(Uuid::new_v4());
        let other_id = ConnectionId::from_uuid(Uuid::new_v4());
        let armed_at = Instant::now();

        assert_eq!(
            repair_confirmation_decision(None, connection_id, armed_at),
            RepairConfirmationDecision::Arm
        );
        assert_eq!(
            repair_confirmation_decision(
                Some((connection_id, armed_at)),
                connection_id,
                armed_at + Duration::from_millis(100)
            ),
            RepairConfirmationDecision::Wait
        );
        assert_eq!(
            repair_confirmation_decision(
                Some((connection_id, armed_at)),
                connection_id,
                armed_at + REPAIR_CONFIRM_MIN_DELAY
            ),
            RepairConfirmationDecision::Confirm
        );
        assert_eq!(
            repair_confirmation_decision(
                Some((connection_id, armed_at)),
                connection_id,
                armed_at + REPAIR_CONFIRM_MAX_DELAY + Duration::from_millis(1)
            ),
            RepairConfirmationDecision::Arm
        );
        assert_eq!(
            repair_confirmation_decision(
                Some((other_id, armed_at)),
                connection_id,
                armed_at + REPAIR_CONFIRM_MIN_DELAY
            ),
            RepairConfirmationDecision::Arm
        );
    }

    #[test]
    fn stale_rclone_refresh_completion_cannot_replace_current_job_status() {
        use cosmic::Application;
        let connection_id = ConnectionId::from_uuid(Uuid::new_v4());
        let mut app = AppModel {
            last_notice: Some("Current directory refresh job 8 is running.".into()),
            rclone_refresh_jobs: BTreeMap::from([(connection_id, 8)]),
            ..AppModel::default()
        };

        let _ = app.update(Message::RcloneRefreshCompleted(
            "Google Drive".into(),
            connection_id,
            7,
            Ok(RcloneRefreshOutcome::Completed(
                cosmic_ext_applet_mounter::preload_report::PreloadReport {
                    duration_seconds: 2.0,
                    limit_seconds: 60.0,
                    maximum_depth: None,
                    excluded_paths: 0,
                },
            )),
        ));

        assert_eq!(app.rclone_refresh_jobs.get(&connection_id), Some(&8));
        assert_eq!(
            app.last_notice.as_deref(),
            Some("Current directory refresh job 8 is running.")
        );
    }

    #[test]
    fn preload_completion_does_not_replace_unmount_progress_notice() {
        use cosmic::Application;
        let connection_id = ConnectionId::new();
        let mut app = AppModel {
            last_notice: Some("Unmounting VPS… stopping background preload first.".into()),
            unmount_pending: BTreeSet::from([connection_id]),
            directory_preload_jobs: BTreeMap::from([(connection_id, 5)]),
            ..AppModel::default()
        };
        let _ = app.update(Message::DirectoryPreloadCompleted(
            "VPS".into(),
            connection_id,
            5,
            Ok(DirectoryPreloadOutcome::Cancelled),
        ));
        assert_eq!(
            app.last_notice.as_deref(),
            Some("Unmounting VPS… stopping background preload first.")
        );
        assert!(!app.directory_preload_jobs.contains_key(&connection_id));
    }

    #[test]
    fn stale_onedriver_preload_completion_cannot_replace_current_status() {
        use cosmic::Application;
        let connection_id = ConnectionId::from_uuid(Uuid::new_v4());
        let mut app = AppModel {
            last_notice: Some("Current OneDrive preload is running.".into()),
            directory_preload_jobs: BTreeMap::from([(connection_id, 8)]),
            ..AppModel::default()
        };

        let _ = app.update(Message::DirectoryPreloadCompleted(
            "OneDrive".into(),
            connection_id,
            7,
            Ok(DirectoryPreloadOutcome::Completed(
                cosmic_ext_applet_mounter::preload_report::PreloadReport {
                    duration_seconds: 2.0,
                    limit_seconds: 60.0,
                    maximum_depth: None,
                    excluded_paths: 0,
                },
            )),
        ));

        assert_eq!(app.directory_preload_jobs.get(&connection_id), Some(&8));
        assert_eq!(
            app.last_notice.as_deref(),
            Some("Current OneDrive preload is running.")
        );
    }

    #[test]
    fn sftp_preload_settings_and_connection_override_round_trip() {
        let mut app = AppModel::default();
        use cosmic::Application;
        let _ = app.update(Message::DraftProvider(Provider::Sftp));
        assert!(!app.draft.connection_preload_enabled);
        assert_eq!(app.draft.connection_preload_seconds, "30");
        assert_eq!(app.draft.connection_preload_depth, "2");
        let mut settings = PreloadSettingsDraft::default();
        settings.policy_mut(PreloadProvider::Sftp).enabled = true;
        let policy = settings.validated().unwrap();
        assert!(policy.sftp.enabled);
        assert_eq!(policy.sftp.maximum_seconds, 30);
        let mut draft = ConnectionDraft {
            provider: Provider::Sftp,
            name: "SFTP".into(),
            remote_reference: "sftp".into(),
            local_path: "/tmp/server".into(),
            ..ConnectionDraft::default()
        };
        draft.connection_preload_use_global = false;
        draft.connection_preload_enabled = true;
        draft.connection_preload_seconds = "20".into();
        draft.connection_preload_depth = "2".into();
        let connection = connection_from_draft(&draft).unwrap();
        assert!(connection.smb_preload_override.is_none());
        assert_eq!(
            connection.sftp_preload_override.unwrap().maximum_seconds,
            20
        );
        let restored = draft_from_connection(&connection);
        assert!(!restored.connection_preload_use_global);
        assert_eq!(restored.connection_preload_seconds, "20");
        assert!(uses_directory_preload(&connection));
    }

    #[test]
    fn preload_setting_rejects_out_of_range_input() {
        use cosmic::Application;
        let mut app = AppModel {
            preload_draft: PreloadSettingsDraft {
                onedrive: PreloadPolicyDraft {
                    maximum_seconds: "4".into(),
                    ..PreloadSettings::default().onedrive.into()
                },
                ..PreloadSettings::default().into()
            },
            preload_input_dirty: true,
            runtime_available: true,
            ..AppModel::default()
        };

        let _ = app.update(Message::SavePreloadSettings);

        assert!(!app.runtime_pending);
        assert!(
            app.preload_notice
                .as_deref()
                .is_some_and(|notice| notice.contains("5 to 600"))
        );
    }

    #[test]
    fn runtime_preferences_persist_only_requested_fields_and_roll_back_on_failure() {
        use cosmic_ext_applet_mounter::config::HostVisibleConfigStorage;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.ron");
        let storage = AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(path.clone()));
        let mut config = Config::default();
        config.document.notifications_enabled = true;
        save_runtime_preference(&mut config, &storage, RuntimeCommand::SetRestore(true)).unwrap();
        save_runtime_preference(&mut config, &storage, RuntimeCommand::SetUnmount(true)).unwrap();
        save_runtime_preference(
            &mut config,
            &storage,
            RuntimeCommand::SetPreload(PreloadSettings {
                onedrive: PreloadPolicy {
                    maximum_seconds: 90,
                    ..PreloadSettings::default().onedrive
                },
                ..PreloadSettings::default()
            }),
        )
        .unwrap();
        save_runtime_preference(&mut config, &storage, RuntimeCommand::SetUnmount(false)).unwrap();
        let saved: ConfigDocument = ron::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved, config.document);
        assert!(saved.notifications_enabled);
        assert!(saved.restore_after_wake);
        assert!(!saved.unmount_before_sleep);
        assert_eq!(saved.preload.onedrive.maximum_seconds, 90);
        let invalid = AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(
            path.join("impossible.ron"),
        ));
        assert!(
            save_runtime_preference(&mut config, &invalid, RuntimeCommand::SetUnmount(true))
                .is_err()
        );
        assert_eq!(config.document, saved);
    }

    #[test]
    fn background_poll_does_not_start_a_user_action() {
        use cosmic::Application;
        let mut app = AppModel {
            runtime_available: true,
            ..AppModel::default()
        };
        let _ = app.update(Message::RuntimePoll);
        assert!(app.runtime_poll_pending);
        assert!(!app.runtime_pending);
        assert!(app.runtime_available);
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Status,
            Ok(runtime_ipc::Status::default()),
        ));
        assert!(!app.runtime_poll_pending);
        assert!(!app.runtime_pending);
        assert!(app.runtime_available);
    }

    #[test]
    fn routed_operation_completion_is_revisioned_and_bounded() {
        let mut app = AppModel::default();
        let operation = Operation::PreviewInitialSync;
        for revision in 1..=(RUNTIME_OPERATION_COMPLETION_LIMIT + 2) {
            let connection_id = ConnectionId::new();
            app.runtime_routed_operations
                .insert(connection_id, operation);
            app.last_notice = Some(format!("completion {revision}"));
            app.complete_routed_operation(connection_id, operation);
        }

        assert_eq!(
            app.runtime_operation_revision,
            (RUNTIME_OPERATION_COMPLETION_LIMIT + 2) as u64
        );
        assert_eq!(
            app.runtime_operation_completions.len(),
            RUNTIME_OPERATION_COMPLETION_LIMIT
        );
        assert_eq!(
            app.runtime_operation_completions
                .first()
                .map(|completion| completion.revision),
            Some(3)
        );
        let expected_last_notice = format!("completion {}", RUNTIME_OPERATION_COMPLETION_LIMIT + 2);
        assert_eq!(
            app.runtime_operation_completions
                .last()
                .map(|completion| completion.notice.as_str()),
            Some(expected_last_notice.as_str())
        );
        assert!(app.runtime_routed_operations.is_empty());
    }

    #[test]
    fn routed_operation_wait_ignores_stale_and_unrelated_completions() {
        use cosmic::Application;
        let connection_id = ConnectionId::new();
        let unrelated_id = ConnectionId::new();
        let operation = Operation::PreviewInitialSync;
        let command = RuntimeCommand::RunOperation(connection_id, operation);
        let mut app = AppModel {
            standalone: true,
            runtime_pending: true,
            runtime_operation_wait: Some(RuntimeOperationWait {
                connection_id,
                operation,
                after_revision: None,
            }),
            ..AppModel::default()
        };

        let _ = app.update(Message::RuntimeResult(
            command,
            Ok(runtime_ipc::Status {
                operation_revision: 7,
                ..runtime_ipc::Status::default()
            }),
        ));
        let requested_notice = app.last_notice.clone();
        assert_eq!(
            app.runtime_operation_wait
                .and_then(|wait| wait.after_revision),
            Some(7)
        );

        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Status,
            Ok(runtime_ipc::Status {
                operation_revision: 9,
                operation_completions: vec![
                    runtime_ipc::OperationCompletion {
                        revision: 7,
                        connection_id,
                        operation,
                        notice: "stale matching completion".into(),
                    },
                    runtime_ipc::OperationCompletion {
                        revision: 8,
                        connection_id: unrelated_id,
                        operation,
                        notice: "unrelated connection completion".into(),
                    },
                    runtime_ipc::OperationCompletion {
                        revision: 9,
                        connection_id,
                        operation: Operation::SyncNow,
                        notice: "unrelated operation completion".into(),
                    },
                ],
                ..runtime_ipc::Status::default()
            }),
        ));
        assert!(app.runtime_operation_wait.is_some());
        assert_eq!(app.last_notice, requested_notice);

        let completion_notice = "Preview completed with no changes.";
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Status,
            Ok(runtime_ipc::Status {
                operation_revision: 10,
                operation_completions: vec![runtime_ipc::OperationCompletion {
                    revision: 10,
                    connection_id,
                    operation,
                    notice: completion_notice.into(),
                }],
                ..runtime_ipc::Status::default()
            }),
        ));
        assert!(app.runtime_operation_wait.is_none());
        assert_eq!(app.last_notice.as_deref(), Some(completion_notice));
    }

    #[test]
    fn late_poll_cannot_undo_a_completed_user_action() {
        use cosmic::Application;
        for command in [
            RuntimeCommand::SetUnmount(true),
            RuntimeCommand::SetRestore(true),
            RuntimeCommand::Refresh,
        ] {
            for poll_fails in [false, true] {
                let mut app = AppModel {
                    runtime_available: true,
                    ..AppModel::default()
                };
                let _ = app.update(Message::RuntimePoll);
                let action = match command {
                    RuntimeCommand::SetUnmount(value) => Message::UnmountBeforeSleep(value),
                    RuntimeCommand::SetRestore(value) => Message::RestoreAfterWake(value),
                    _ => Message::Refresh,
                };
                let _ = app.update(action);
                assert!(app.runtime_pending);
                let updated = runtime_ipc::Status {
                    unmount_before_sleep: true,
                    restore_after_wake: true,
                    preload: PreloadSettings::default(),
                    sleep_status: Some("Current cleanup status".into()),
                    ..runtime_ipc::Status::default()
                };
                let _ = app.update(Message::RuntimeResult(command, Ok(updated.clone())));
                let stale = if poll_fails {
                    Err("old poll timed out".into())
                } else {
                    Ok(runtime_ipc::Status::default())
                };
                let _ = app.update(Message::RuntimeResult(RuntimeCommand::Status, stale));
                assert_eq!(app.runtime_status(), updated);
                assert!(app.runtime_available);
                assert!(app.runtime_error.is_none());
                assert!(!app.runtime_pending);
                assert!(!app.runtime_poll_pending);
                // The next fresh poll must still be applied.
                let _ = app.update(Message::RuntimePoll);
                let _ = app.update(Message::RuntimeResult(
                    RuntimeCommand::Status,
                    Ok(runtime_ipc::Status::default()),
                ));
                assert_eq!(app.runtime_status(), runtime_ipc::Status::default());
            }
        }
    }

    #[test]
    fn obsolete_poll_does_not_clear_in_flight_action_or_connection_error() {
        use cosmic::Application;
        let mut app = AppModel {
            runtime_error: Some("last connection error".into()),
            ..AppModel::default()
        };
        let _ = app.update(Message::RuntimePoll);
        let _ = app.update(Message::Refresh);
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Status,
            Ok(runtime_ipc::Status::default()),
        ));
        assert!(app.runtime_pending);
        assert!(!app.runtime_available);
        assert_eq!(app.runtime_error.as_deref(), Some("last connection error"));
        assert!(!app.runtime_poll_pending);
    }

    #[test]
    fn settings_failures_do_not_change_effective_preferences_or_vanish_on_poll() {
        use cosmic::Application;
        let mut app = AppModel {
            standalone: true,
            window_mode: WindowMode::GeneralSettings,
            runtime_pending: true,
            ..AppModel::default()
        };
        app.config.document.restore_after_wake = true;
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::SetUnmount(true),
            Err("save failed".into()),
        ));
        assert!(!app.config.document.unmount_before_sleep);
        assert!(app.config.document.restore_after_wake);
        assert!(!app.runtime_pending);
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Status,
            Ok(runtime_ipc::Status {
                restore_after_wake: true,
                ..runtime_ipc::Status::default()
            }),
        ));
        assert_eq!(app.sleep_notice.as_deref(), Some("save failed"));
        assert!(app.last_notice.is_none());
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::Refresh,
            Ok(runtime_ipc::Status::default()),
        ));
        assert!(app.last_notice.as_deref().unwrap().contains("refreshed"));
        assert_eq!(app.sleep_notice.as_deref(), Some("save failed"));
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::SetRestore(true),
            Ok(runtime_ipc::Status::default()),
        ));
        assert_eq!(app.sleep_notice.as_deref(), Some("Setting saved."));
        assert!(app.last_notice.as_deref().unwrap().contains("refreshed"));
    }

    #[test]
    fn settings_acknowledgment_updates_flags_without_owning_sleep_listener() {
        use cosmic::Application;
        let mut app = AppModel {
            standalone: true,
            window_mode: WindowMode::GeneralSettings,
            runtime_pending: true,
            ..AppModel::default()
        };
        let _ = app.update(Message::RuntimeEvent(runtime_ipc::Event::Ready));
        let _ = app.update(Message::RuntimeResult(
            RuntimeCommand::SetUnmount(true),
            Ok(runtime_ipc::Status {
                unmount_before_sleep: true,
                restore_after_wake: true,
                preload: PreloadSettings::default(),
                sleep_status: Some("ready".into()),
                ..runtime_ipc::Status::default()
            }),
        ));
        assert!(app.config.document.unmount_before_sleep);
        assert!(app.config.document.restore_after_wake);
        assert!(!app.runtime_owner);
        assert!(!app.runtime_pending);
        assert!(app.runtime_available);
    }

    #[test]
    fn additional_panel_instance_uses_existing_runtime_without_warning() {
        use cosmic::Application;
        let mut app = AppModel {
            runtime_owner: true,
            runtime_error: Some("runtime name is already owned".into()),
            ..AppModel::default()
        };

        let _ = app.update(Message::RuntimeEvent(runtime_ipc::Event::ClientReady));

        assert!(!app.runtime_owner);
        assert!(app.runtime_error.is_none());
    }

    #[test]
    fn sleep_warning_stays_visible_while_normal_status_is_only_in_settings() {
        use cosmic::Application;
        let mut app = AppModel::default();
        let _ = app.update(Message::SleepEvent(
            cosmic_ext_applet_mounter::sleep::Event::Status {
                text: "睡眠前のクリーンアップに注意が必要です".into(),
                needs_attention: true,
            },
        ));
        assert!(app.sleep_needs_attention);
        assert!(app.runtime_status().sleep_needs_attention);
        let _ = app.update(Message::SleepEvent(
            cosmic_ext_applet_mounter::sleep::Event::Status {
                text: "準備完了".into(),
                needs_attention: false,
            },
        ));
        assert!(!app.sleep_needs_attention);
    }

    #[test]
    fn remote_help_only_suggests_creation_in_add_mode() {
        for provider in [Provider::GoogleDrive, Provider::Box, Provider::Smb] {
            assert!(rclone_remote_help(provider, true).contains("Enter a new"));
            assert!(rclone_remote_help(provider, true).contains("Create"));
            assert!(!rclone_remote_help(provider, false).contains("Create"));
            assert!(rclone_remote_help(provider, false).contains("rclone config"));
        }
    }

    #[test]
    fn flatpak_durable_roots_use_host_visible_home_paths() {
        let home = PathBuf::from("/home/example");

        assert_eq!(
            default_config_root_for(true, Some(home.clone())),
            PathBuf::from("/home/example/.config/cosmic-ext-applet-mounter")
        );
        assert_eq!(
            default_cache_root_for(true, Some(home.clone())),
            PathBuf::from("/home/example/.cache/cosmic-ext-applet-mounter")
        );
        assert_eq!(
            default_work_root_for(true, Some(home)),
            PathBuf::from("/home/example/.local/state/cosmic-ext-applet-mounter")
        );
    }

    #[test]
    fn popup_scroll_height_depends_on_connection_count_not_notice_text() {
        let empty = popup_connection_scroll_height(0, true);
        let one_row = popup_connection_scroll_height(1, false);
        let five_rows = popup_connection_scroll_height(5, false);
        let full_list = popup_connection_scroll_height(20, false);

        assert_eq!(empty, POPUP_EMPTY_ROW_HEIGHT);
        assert_eq!(one_row, POPUP_CONNECTION_ROW_HEIGHT);
        assert_eq!(five_rows, 5.0 * POPUP_CONNECTION_ROW_HEIGHT);
        assert_eq!(full_list, POPUP_CONNECTION_LIST_MAX_HEIGHT);
    }

    #[test]
    fn popup_connection_display_name_truncates_long_names() {
        assert_eq!(
            popup_connection_display_name("Rclone mount for UA Box"),
            "Rclone mount for UA Box"
        );
        assert_eq!(
            popup_connection_display_name("Google Drive OAuth live verify offline"),
            "Google Drive OAuth live verify of..."
        );
    }

    #[test]
    fn popup_notice_timeout_starts_when_a_closed_popup_is_reopened() {
        let notice_started_at = Instant::now();
        let popup_opened_at = notice_started_at + Duration::from_secs(30);

        assert!(!popup_notice_expired(
            notice_started_at,
            Some(popup_opened_at),
            popup_opened_at + Duration::from_secs(9),
        ));
        assert!(popup_notice_expired(
            notice_started_at,
            Some(popup_opened_at),
            popup_opened_at + POPUP_NOTICE_TIMEOUT,
        ));
        assert!(popup_notice_expired(
            notice_started_at,
            None,
            notice_started_at + POPUP_NOTICE_TIMEOUT,
        ));
    }

    #[test]
    fn requested_vpn_activation_stops_waiting_after_terminal_state() {
        use cosmic_ext_applet_mounter::vpn::VpnConnectionState;

        assert_eq!(
            vpn_activation_terminal_error(VpnConnectionState::Disconnected, true).as_deref(),
            Some("VPN authentication was canceled or activation failed")
        );
        assert_eq!(
            vpn_activation_terminal_error(VpnConnectionState::Failed, true).as_deref(),
            Some("VPN authentication or activation failed")
        );
        assert_eq!(
            vpn_activation_terminal_error(VpnConnectionState::Activating, true),
            None
        );
        assert_eq!(
            vpn_activation_terminal_error(VpnConnectionState::Activated, true),
            None
        );
        assert_eq!(
            vpn_activation_terminal_error(VpnConnectionState::Disconnected, false),
            None
        );
    }

    #[test]
    fn finite_offline_vpn_users_are_reference_counted_and_release_is_non_destructive() {
        let profile_id = VpnProfileId::new();
        let first = ConnectionId::new();
        let second = ConnectionId::new();
        unregister_finite_offline_vpn_user(profile_id, first);
        unregister_finite_offline_vpn_user(profile_id, second);

        register_finite_offline_vpn_user(profile_id, first);
        register_finite_offline_vpn_user(profile_id, second);
        {
            let users = active_finite_offline_vpn_users()
                .lock()
                .expect("finite Offline Mirror VPN users");
            assert_eq!(
                users.get(&profile_id),
                Some(&BTreeSet::from([first, second]))
            );
        }
        unregister_finite_offline_vpn_user(profile_id, first);
        {
            let users = active_finite_offline_vpn_users()
                .lock()
                .expect("finite Offline Mirror VPN users");
            assert_eq!(users.get(&profile_id), Some(&BTreeSet::from([second])));
        }
        unregister_finite_offline_vpn_user(profile_id, second);
        assert!(
            !active_finite_offline_vpn_users()
                .lock()
                .expect("finite Offline Mirror VPN users")
                .contains_key(&profile_id)
        );

        let completed = merge_offline_vpn_release(
            Ok("Synchronization succeeded.".into()),
            Err("deactivation refused".into()),
        )
        .expect("successful storage operation remains successful");
        assert!(completed.contains("Synchronization succeeded"));
        assert!(completed.contains("VPN remains connected"));
        let failed = merge_offline_vpn_release(
            Err("Synchronization failed".into()),
            Err("deactivation refused".into()),
        )
        .expect_err("storage failure remains a failure");
        assert!(failed.contains("Synchronization failed"));
        assert!(failed.contains("VPN remains connected"));
    }

    #[tokio::test]
    async fn offline_readiness_guard_rejects_malformed_requests_before_dbus() {
        assert!(run_offline_readiness_guard(&[]).await.is_err());
        assert!(
            run_offline_readiness_guard(&["not-a-uuid".into()])
                .await
                .is_err()
        );
        assert!(
            run_offline_readiness_guard(&[ConnectionId::new().to_string(), "extra".into()])
                .await
                .is_err()
        );
    }

    #[test]
    fn vpn_status_polling_includes_only_active_enabled_connections() {
        let enabled_profile = VpnProfileId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("UUID"),
        );
        let disabled_profile = VpnProfileId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("UUID"),
        );
        let mut enabled = test_connection(Provider::GoogleDrive);
        enabled.id = ConnectionId::from_uuid(
            Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("UUID"),
        );
        enabled.vpn_profile_id = Some(enabled_profile);
        let mut disabled = test_connection(Provider::Box);
        disabled.id = ConnectionId::from_uuid(
            Uuid::parse_str("44444444-4444-4444-8444-444444444444").expect("UUID"),
        );
        disabled.enabled = false;
        disabled.vpn_profile_id = Some(disabled_profile);

        assert_eq!(
            referenced_active_vpn_profiles(
                &[enabled.clone(), disabled.clone()],
                &BTreeSet::from([enabled.id, disabled.id])
            ),
            BTreeSet::from([enabled_profile])
        );
        assert!(referenced_active_vpn_profiles(&[enabled, disabled], &BTreeSet::new()).is_empty());
    }

    #[test]
    fn rclone_remote_button_rows_have_bounded_capacity() {
        assert_eq!(rclone_remote_buttons_per_row(), 3);
        let remote_count = 7usize;
        let rows = remote_count.div_ceil(rclone_remote_buttons_per_row());
        assert_eq!(rows, 3);
    }

    #[test]
    fn popup_repair_uses_controller_eligibility_and_pending_state_for_both_modes() {
        let mut row = ConnectionRowState {
            id: ConnectionId::from_uuid(
                Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
            ),
            name: "OneDrive online mount test".into(),
            provider: Provider::OneDrive,
            mode: AccessMode::OnlineMount,
            persistent_active: false,
            local_path: PathBuf::from("/home/example/Cloud/OneDrive"),
            vpn_profile_id: None,
            status: ConnectionStatus::OnlineMount(OnlineMountStatus::Error),
            warnings: vec![],
            actions: vec![
                cosmic_ext_applet_mounter::controller::OperationAction {
                    operation: Operation::Mount,
                    enabled: true,
                },
                cosmic_ext_applet_mounter::controller::OperationAction {
                    operation: Operation::Repair,
                    enabled: true,
                },
            ],
            settings: cosmic_ext_applet_mounter::controller::SettingsSummary {
                remote: "onedrive".into(),
                remote_subpath: None,
                start_at_login: Some(false),
                sync_interval_minutes: None,
                sync_on_metered: None,
            },
        };

        assert_eq!(primary_operation(&row), Some(Operation::Mount));
        assert!(repair_available(&row));
        let mut pending_repairs = BTreeSet::new();
        assert!(repair_visible(&row, &pending_repairs));

        row.persistent_active = true;
        assert_eq!(primary_operation(&row), Some(Operation::Unmount));
        row.status = ConnectionStatus::OnlineMount(OnlineMountStatus::Degraded);
        assert!(repair_available(&row));

        pending_repairs.insert(row.id);
        assert!(!repair_visible(&row, &pending_repairs));

        pending_repairs.clear();
        row.actions
            .iter_mut()
            .find(|action| action.operation == Operation::Repair)
            .expect("repair action")
            .enabled = false;
        assert!(!repair_available(&row));

        row.actions
            .iter_mut()
            .find(|action| action.operation == Operation::Repair)
            .expect("repair action")
            .enabled = true;
        row.mode = AccessMode::OfflineMirror;
        assert!(repair_available(&row));
        assert!(repair_visible(&row, &pending_repairs));
    }

    #[test]
    fn failed_unmount_requires_repair_until_an_online_operation_succeeds() {
        let connection_id = ConnectionId::new();
        let mut required = BTreeSet::new();

        update_online_repair_requirement(
            &mut required,
            connection_id,
            Operation::Mount,
            &Err("mount failed".into()),
        );
        assert!(required.is_empty());

        update_online_repair_requirement(
            &mut required,
            connection_id,
            Operation::Unmount,
            &Err("clean detach remained busy".into()),
        );
        assert!(required.contains(&connection_id));

        update_online_repair_requirement(&mut required, connection_id, Operation::Mount, &Ok(()));
        assert!(required.is_empty());
    }

    #[test]
    fn sharepoint_offline_popup_uses_start_and_stop_schedule_actions() {
        let mut row = ConnectionRowState {
            id: ConnectionId::from_uuid(
                Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
            ),
            name: "SharePoint offline test".into(),
            provider: Provider::Teams,
            mode: AccessMode::OfflineMirror,
            persistent_active: false,
            local_path: PathBuf::from("/home/example/Cloud/SharePoint"),
            vpn_profile_id: None,
            status: ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Paused),
            warnings: vec![],
            actions: vec![
                cosmic_ext_applet_mounter::controller::OperationAction {
                    operation: Operation::ResumeSync,
                    enabled: true,
                },
                cosmic_ext_applet_mounter::controller::OperationAction {
                    operation: Operation::SyncNow,
                    enabled: true,
                },
            ],
            settings: cosmic_ext_applet_mounter::controller::SettingsSummary {
                remote: "sharepoint".into(),
                remote_subpath: Some("Disposable".into()),
                start_at_login: None,
                sync_interval_minutes: Some(15),
                sync_on_metered: Some(false),
            },
        };

        assert_eq!(primary_operation(&row), Some(Operation::ResumeSync));

        row.status = ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Idle);
        row.persistent_active = true;
        row.actions = vec![
            cosmic_ext_applet_mounter::controller::OperationAction {
                operation: Operation::SyncNow,
                enabled: true,
            },
            cosmic_ext_applet_mounter::controller::OperationAction {
                operation: Operation::PauseSync,
                enabled: true,
            },
        ];
        assert_eq!(primary_operation(&row), Some(Operation::PauseSync));

        row.status = ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Error);
        row.actions = vec![
            cosmic_ext_applet_mounter::controller::OperationAction {
                operation: Operation::SyncNow,
                enabled: false,
            },
            cosmic_ext_applet_mounter::controller::OperationAction {
                operation: Operation::Repair,
                enabled: true,
            },
        ];
        assert_eq!(primary_operation(&row), Some(Operation::PauseSync));
    }

    #[test]
    fn repair_resets_only_a_failed_service() {
        let status = |active| UnitStatus {
            active,
            enabled: false,
            detail: "test".into(),
            result: "success".into(),
        };
        assert!(repair_requires_failed_reset(Some(&status(
            ActiveState::Failed
        ))));
        assert!(!repair_requires_failed_reset(Some(&status(
            ActiveState::Inactive
        ))));
        assert!(!repair_requires_failed_reset(Some(&status(
            ActiveState::Active
        ))));
        assert!(!repair_requires_failed_reset(None));
    }

    #[test]
    fn online_repair_provider_matrix_owns_only_the_expected_fuse_engine() {
        let cases = [
            (
                Provider::OneDrive,
                OnlineMountEngine::Onedriver,
                "fuse.onedriver",
                false,
                true,
            ),
            (
                Provider::Teams,
                OnlineMountEngine::Rclone,
                "fuse.rclone",
                false,
                true,
            ),
            (
                Provider::GoogleDrive,
                OnlineMountEngine::Rclone,
                "fuse.rclone",
                true,
                false,
            ),
            (
                Provider::Box,
                OnlineMountEngine::Rclone,
                "fuse.rclone",
                false,
                true,
            ),
            (
                Provider::Smb,
                OnlineMountEngine::Rclone,
                "fuse.rclone",
                false,
                true,
            ),
            (
                Provider::Sftp,
                OnlineMountEngine::Rclone,
                "fuse.rclone",
                false,
                true,
            ),
        ];

        for (provider, engine, filesystem, cancel_refresh, cancel_preload) in cases {
            let connection = test_connection(provider);
            assert_eq!(
                online_repair_plan(&connection),
                Some(OnlineRepairPlan {
                    engine,
                    cancel_rclone_refresh: cancel_refresh,
                    cancel_directory_preload: cancel_preload,
                })
            );
            assert_eq!(
                online_repair_plan(&connection)
                    .expect("online repair plan")
                    .expected_filesystem(),
                filesystem
            );
            assert!(!repair_needs_lazy_detach(&connection, &[]).unwrap());

            let mount = cosmic_ext_applet_mounter::mounts::MountEntry {
                target: connection.local_path.clone(),
                source: "managed online engine".into(),
                filesystem: filesystem.into(),
                options: Vec::new(),
            };
            assert!(repair_needs_lazy_detach(&connection, std::slice::from_ref(&mount)).unwrap());

            let mut foreign_mount = mount;
            foreign_mount.filesystem = "ext4".into();
            assert!(repair_needs_lazy_detach(&connection, &[foreign_mount]).is_err());

            let mut offline = connection;
            offline.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
            assert_eq!(online_repair_plan(&offline), None);
        }

        assert_eq!(CLEAN_UNMOUNT_SETTLE_TIMEOUT, Duration::from_secs(2));
        assert!(CLEAN_UNMOUNT_POLL_INTERVAL < CLEAN_UNMOUNT_SETTLE_TIMEOUT);
    }

    #[tokio::test]
    async fn online_repair_stops_service_and_resets_only_failed_state() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let connection = test_connection(Provider::GoogleDrive);
        let unit = UnitName::new(connection.id, UnitKind::Service);
        let failed = UnitStatus {
            active: ActiveState::Failed,
            enabled: false,
            detail: "failed".into(),
            result: "exit-code".into(),
        };
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(Some(failed)));
        manager.push(Ok(None));

        stop_online_repair_service(&manager, &unit, CancellationToken::new()).await;
        reconcile_online_repair_service(&manager, &unit, CancellationToken::new())
            .await
            .expect("reconcile failed service");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Stop,
                SystemdAction::Status,
                SystemdAction::ResetFailed
            ]
        );

        let inactive = UnitStatus {
            active: ActiveState::Inactive,
            enabled: false,
            detail: "stopped".into(),
            result: "success".into(),
        };
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(Some(inactive)));
        stop_online_repair_service(&manager, &unit, CancellationToken::new()).await;
        reconcile_online_repair_service(&manager, &unit, CancellationToken::new())
            .await
            .expect("reconcile inactive service");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [SystemdAction::Stop, SystemdAction::Status]
        );
    }

    fn repair_unit_status(active: ActiveState, enabled: bool) -> UnitStatus {
        UnitStatus {
            active,
            enabled,
            detail: format!("{active:?}"),
            result: if active == ActiveState::Failed {
                "exit-code".into()
            } else {
                "success".into()
            },
        }
    }

    #[test]
    fn offline_repair_plan_separates_rclone_timer_from_onedrive_monitor() {
        for provider in [
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let mut connection = test_connection(provider);
            connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
            connection.enabled = false;
            let plan = offline_repair_plan(&connection).expect("rclone repair plan");
            assert_eq!(plan.engine, OfflineRepairEngine::RcloneBisync);
            assert!(plan.timer.is_some());
        }

        let mut onedrive = test_connection(Provider::OneDrive);
        onedrive.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        onedrive.enabled = false;
        let plan = offline_repair_plan(&onedrive).expect("OneDrive repair plan");
        assert_eq!(plan.engine, OfflineRepairEngine::OneDriveMonitor);
        assert!(plan.timer.is_none());

        onedrive.mode = ConnectionMode::OnlineMount(OnlineMountConfig::default());
        assert!(offline_repair_plan(&onedrive).is_err());
    }

    #[tokio::test]
    async fn rclone_offline_repair_pauses_timer_then_resets_failed_service() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::Smb);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Failed, false))));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));

        execute_offline_repair_plan(&manager, &plan, CancellationToken::new())
            .await
            .expect("rclone repair");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Stop,
                SystemdAction::Disable,
                SystemdAction::Status,
                SystemdAction::Stop,
                SystemdAction::Disable,
                SystemdAction::Status,
                SystemdAction::ResetFailed,
                SystemdAction::Status,
            ]
        );
        let units = manager
            .actions()
            .iter()
            .map(|(_, unit)| unit.clone().unwrap())
            .collect::<Vec<_>>();
        assert!(units[..3].iter().all(|unit| unit.ends_with(".timer")));
        assert!(units[3..].iter().all(|unit| unit.ends_with(".service")));
    }

    #[tokio::test]
    async fn coordinated_repair_pauses_trigger_then_waits_for_manual_cancellation() {
        use cosmic_ext_applet_mounter::offline_coordinator::{read_state, write_state};
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::Smb);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        for response in [
            Ok(None),
            Ok(None),
            Ok(Some(repair_unit_status(ActiveState::Inactive, false))),
            Ok(None),
            Ok(None),
            Ok(Some(repair_unit_status(ActiveState::Inactive, false))),
        ] {
            manager.push(response);
        }

        let directory = tempfile::tempdir().unwrap();
        let state_path = directory.path().join("state.json");
        write_state(
            &state_path,
            &OfflineCoordinatorState::new(
                connection.id,
                OfflineOperation::Sync,
                OfflineOperationState::Running,
            ),
        )
        .unwrap();
        let control = Arc::new(ManualOperationControl::new());
        let operation = Arc::clone(&control);
        let completed_path = state_path.clone();
        let id = connection.id;
        let operation_task = tokio::spawn(async move {
            operation.cancellation().cancelled().await;
            write_state(
                &completed_path,
                &OfflineCoordinatorState::new(
                    id,
                    OfflineOperation::Sync,
                    OfflineOperationState::Cancelled,
                ),
            )
            .unwrap();
            operation.finish();
        });

        execute_coordinated_offline_repair(
            &manager,
            &plan,
            connection.id,
            Some(&control),
            &state_path,
            CancellationToken::new(),
        )
        .await
        .expect("coordinated repair");
        operation_task.await.unwrap();

        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Stop,
                SystemdAction::Disable,
                SystemdAction::Status,
                SystemdAction::Stop,
                SystemdAction::Disable,
                SystemdAction::Status,
            ]
        );
        let state = read_state(&state_path).unwrap().unwrap();
        assert_eq!(state.operation, OfflineOperation::Repair);
        assert_eq!(state.state, OfflineOperationState::Running);
    }

    #[tokio::test]
    async fn coordinated_repair_rejects_unowned_running_state_before_service_stop() {
        use cosmic_ext_applet_mounter::offline_coordinator::write_state;
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        let directory = tempfile::tempdir().unwrap();
        let state_path = directory.path().join("state.json");
        write_state(
            &state_path,
            &OfflineCoordinatorState::new(
                connection.id,
                OfflineOperation::Sync,
                OfflineOperationState::Running,
            ),
        )
        .unwrap();

        let error = execute_coordinated_offline_repair(
            &manager,
            &plan,
            connection.id,
            None,
            &state_path,
            CancellationToken::new(),
        )
        .await
        .expect_err("unowned operation must not be guessed stopped");
        assert!(error.contains("another process"));
        assert!(manager.actions().is_empty());
    }

    #[tokio::test]
    async fn rclone_repair_stops_scheduled_service_then_replaces_stale_running_state() {
        use cosmic_ext_applet_mounter::offline_coordinator::{read_state, write_state};
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::Smb);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        for response in [
            Ok(None),
            Ok(None),
            Ok(Some(repair_unit_status(ActiveState::Inactive, false))),
            Ok(None),
            Ok(None),
            Ok(Some(repair_unit_status(ActiveState::Inactive, false))),
        ] {
            manager.push(response);
        }
        let directory = tempfile::tempdir().unwrap();
        let state_path = directory.path().join("state.json");
        write_state(
            &state_path,
            &OfflineCoordinatorState::new(
                connection.id,
                OfflineOperation::Scheduled,
                OfflineOperationState::Running,
            ),
        )
        .unwrap();

        execute_coordinated_offline_repair(
            &manager,
            &plan,
            connection.id,
            None,
            &state_path,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        let state = read_state(&state_path).unwrap().unwrap();
        assert_eq!(state.operation, OfflineOperation::Repair);
        assert_eq!(state.state, OfflineOperationState::Running);
        assert_eq!(manager.actions().len(), 6);
    }

    #[tokio::test]
    async fn onedrive_offline_repair_touches_only_its_monitor_service() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));

        execute_offline_repair_plan(&manager, &plan, CancellationToken::new())
            .await
            .expect("OneDrive repair");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Stop,
                SystemdAction::Disable,
                SystemdAction::Status,
            ]
        );
        assert!(manager.actions().iter().all(|(_, unit)| {
            unit.as_deref()
                .is_some_and(|unit| unit.ends_with(".service"))
        }));
    }

    #[tokio::test]
    async fn offline_repair_rejects_a_still_running_service_without_reset() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let plan = offline_repair_plan(&connection).unwrap();
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Active, false))));

        let error = execute_offline_repair_plan(&manager, &plan, CancellationToken::new())
            .await
            .expect_err("running service must not be reported repaired");
        assert!(error.contains("did not become inactive"));
        assert!(
            !manager
                .actions()
                .iter()
                .any(|(action, _)| *action == SystemdAction::ResetFailed)
        );
    }

    #[test]
    fn disconnected_network_manager_state_updates_online_status() {
        use cosmic::Application;

        assert!(network_manager_connected("connected\n"));
        assert!(network_manager_connected("connected (site only)\n"));
        assert!(!network_manager_connected("disconnected\n"));
        assert!(!network_manager_connected("connecting\n"));

        let mut app = AppModel::default();
        app.config
            .document
            .connections
            .push(test_connection(Provider::Sftp));
        let _ = app.update(Message::NetworkStatusChecked(false));
        assert_eq!(
            app.view_state().rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForNetwork)
        );
        let _ = app.update(Message::NetworkStatusChecked(true));
        assert_ne!(
            app.view_state().rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForNetwork)
        );
    }

    #[tokio::test]
    #[ignore = "requires disposable localhost SFTP, host FUSE, and user systemd"]
    async fn live_sftp_online_unmount_repair_and_sleep_cleanup() {
        use cosmic_ext_applet_mounter::process::SystemCommandRunner;
        use cosmic_ext_applet_mounter::services::UnitStore;

        let root = PathBuf::from(std::env::var("COSMIC_SFTP_LIFECYCLE_ROOT").expect("probe root"));
        let config = PathBuf::from(std::env::var("COSMIC_SFTP_LIFECYCLE_CONFIG").expect("config"));
        let mut connection = test_connection(Provider::Sftp);
        connection.id = ConnectionId::new();
        connection.remote_reference = "probe".into();
        connection.local_path = root.join("mount");
        connection.mode = ConnectionMode::OnlineMount(OnlineMountConfig {
            cache_directory: Some(root.join("cache")),
            cache_limit_bytes: 64 * 1024 * 1024,
            start_at_login: false,
        });
        prepare_online_mount_runtime(&connection).unwrap();
        let mut plan =
            rclone_mount_plan(&connection, &default_runtime_root(), &default_cache_root()).unwrap();
        // Keep a disposable write queued long enough to exercise sleep safety.
        plan.service
            .arguments
            .extend(["--vfs-write-back".into(), "20s".into()]);
        plan.service
            .arguments
            .extend(["--config".into(), config.display().to_string()]);
        let document = UnitDocument::service(&plan.service).unwrap();

        struct ProbeGuard {
            unit: UnitName,
            mountpoint: PathBuf,
        }
        impl Drop for ProbeGuard {
            fn drop(&mut self) {
                let name = self.unit.file_name();
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "stop", &name])
                    .status();
                if ProcMountTable::default()
                    .find_target(&self.mountpoint)
                    .is_ok_and(|mount| mount.is_some())
                {
                    let _ = std::process::Command::new("fusermount3")
                        .arg("-uz")
                        .arg(&self.mountpoint)
                        .status();
                }
                if let Ok(store) = FileUnitStore::user(Arc::new(StructuralUnitValidator)) {
                    let _ = store.remove(&self.unit);
                }
                if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
                    let directory = PathBuf::from(runtime)
                        .join("systemd/user")
                        .join(format!("{name}.d"));
                    let config_path = directory.join("91-disposable-sftp-config.conf");
                    if fs::read_to_string(&config_path).is_ok_and(|content| {
                        content.starts_with("# Disposable SFTP lifecycle config")
                    }) {
                        let _ = fs::remove_file(config_path);
                    }
                    let path = directory.join("90-cosmic-mounter-sleep.conf");
                    if fs::read_to_string(&path).is_ok_and(|content| {
                        content.starts_with("# Cloud Mounter Online sleep cleanup")
                    }) {
                        let _ = fs::remove_file(path);
                        let _ = fs::remove_dir(directory);
                    }
                }
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .status();
            }
        }
        let _guard = ProbeGuard {
            unit: document.name.clone(),
            mountpoint: connection.local_path.clone(),
        };
        let unit_dropin = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap())
            .join("systemd/user")
            .join(format!("{}.d", document.name.file_name()));
        fs::create_dir_all(&unit_dropin).unwrap();
        fs::write(
            unit_dropin.join("91-disposable-sftp-config.conf"),
            format!(
                "# Disposable SFTP lifecycle config\n[Service]\nEnvironment=RCLONE_CONFIG={}\n",
                config.display()
            ),
        )
        .unwrap();
        let store = FileUnitStore::user(Arc::new(StructuralUnitValidator)).unwrap();
        let manager = DbusSystemdManager;
        let controller = UnitController::new(store, manager);
        controller
            .install(&document, CancellationToken::new())
            .await
            .unwrap();
        let manager = DbusSystemdManager;
        let start = || async {
            manager
                .action(
                    SystemdAction::Start,
                    Some(&document.name),
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            for _ in 0..50 {
                if ProcMountTable::default()
                    .find_target(&connection.local_path)
                    .unwrap()
                    .is_some()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            panic!("disposable SFTP mount did not appear");
        };

        start().await;
        assert_eq!(
            fs::read_to_string(connection.local_path.join("probe.txt")).unwrap(),
            "probe\n"
        );
        let health = CommandRcloneProvider::new(SystemCommandRunner)
            .vfs_health(&plan.rc_socket, CancellationToken::new())
            .await
            .unwrap();
        assert!(health.healthy_cache());
        assert!(!health.pending_or_active_writes());
        let status = runtime_service_status(connection.id).unwrap();
        assert_eq!(status.active, ActiveState::Active);
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            service_status: BTreeMap::from([(connection.id, status)]),
            mount_entries: ProcMountTable::default().entries().unwrap(),
            ..ControllerSnapshot::default()
        };
        assert_eq!(
            restore(&snapshot).rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted)
        );
        let flatpak_mounts = std::process::Command::new("flatpak-builder")
            .args([
                "--run",
                "target/flatpak-gui-prototype",
                "packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json",
                "/usr/bin/cat",
                "/proc/self/mountinfo",
            ])
            .output()
            .unwrap();
        assert!(flatpak_mounts.status.success());
        println!(
            "Flatpak sandbox sees disposable mount: {}",
            String::from_utf8_lossy(&flatpak_mounts.stdout)
                .contains(&connection.local_path.display().to_string())
        );
        let host_mounts = std::process::Command::new("flatpak-builder")
            .args([
                "--run",
                "target/flatpak-gui-prototype",
                "packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json",
                "flatpak-spawn",
                "--host",
                "/usr/bin/cat",
                "/proc/self/mountinfo",
            ])
            .output()
            .unwrap();
        assert!(host_mounts.status.success());
        assert!(
            String::from_utf8_lossy(&host_mounts.stdout)
                .contains(&connection.local_path.display().to_string())
        );
        run_managed_rclone_online_mount_operation_result_inner(&connection, Operation::Unmount)
            .await
            .unwrap();
        assert!(
            ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_none()
        );

        start().await;
        run_online_mount_repair_operation_result(&connection)
            .await
            .unwrap();
        assert!(
            ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_none()
        );

        start().await;
        fs::write(connection.local_path.join("queued.txt"), "queued upload\n").unwrap();
        let health = CommandRcloneProvider::new(SystemCommandRunner)
            .vfs_health(&plan.rc_socket, CancellationToken::new())
            .await
            .unwrap();
        assert!(health.pending_or_active_writes(), "{health:?}");
        let pending = cosmic_ext_applet_mounter::sleep::cleanup(
            &SystemCommandRunner,
            &[connection.clone()],
            tokio::time::Instant::now() + Duration::from_secs(15),
        )
        .await;
        assert!(pending.active.is_empty(), "{}", pending.summary);
        assert!(
            pending.summary.contains("pending uploads"),
            "{}",
            pending.summary
        );
        assert!(
            ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_some()
        );
        for _ in 0..300 {
            let health = CommandRcloneProvider::new(SystemCommandRunner)
                .vfs_health(&plan.rc_socket, CancellationToken::new())
                .await
                .unwrap();
            if !health.pending_or_active_writes() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(
            fs::read_to_string(root.join("data/queued.txt")).unwrap(),
            "queued upload\n"
        );

        let mut runtime_config = Config::default();
        runtime_config
            .update_validated_runtime(|document| {
                document.unmount_before_sleep = true;
                document.restore_after_wake = true;
                document.connections.push(connection.clone());
            })
            .unwrap();
        let report = cosmic_ext_applet_mounter::sleep::cleanup(
            &SystemCommandRunner,
            &[connection.clone()],
            tokio::time::Instant::now() + Duration::from_secs(15),
        )
        .await;
        assert_eq!(report.active, vec![connection.id], "{}", report.summary);
        assert!(
            ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_none()
        );
        restore_online_after_wake_result(&connection).await.unwrap();
        for _ in 0..50 {
            if ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_some()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(
            ProcMountTable::default()
                .find_target(&connection.local_path)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            fs::read_to_string(connection.local_path.join("probe.txt")).unwrap(),
            "probe\n"
        );
    }

    #[test]
    fn runtime_systemd_status_parser_recognizes_active_disabled_units() {
        let status = parse_runtime_systemd_status(
            "ActiveState=active\nSubState=running\nUnitFileState=disabled\nResult=success\n",
        );

        assert_eq!(status.active, ActiveState::Active);
        assert!(!status.enabled);
        assert_eq!(status.detail, "running");
        assert_eq!(status.result, "success");
    }

    #[test]
    fn scheduled_mirror_reports_failed_run_while_timer_remains_active() {
        let failed = UnitStatus {
            active: ActiveState::Failed,
            enabled: false,
            detail: "failed".into(),
            result: "exit-code".into(),
        };
        let waiting_timer = UnitStatus {
            active: ActiveState::Active,
            enabled: true,
            detail: "waiting".into(),
            result: "success".into(),
        };

        assert_eq!(
            offline_mirror_state_from_status(
                OfflineEngine::Rclone,
                Some(&failed),
                Some(&waiting_timer)
            ),
            SyncRuntimeState::Failed
        );
    }

    #[test]
    fn stopped_schedule_is_paused_even_with_stale_service_failure() {
        let failed = UnitStatus {
            active: ActiveState::Failed,
            enabled: false,
            detail: "failed".into(),
            result: "exit-code".into(),
        };
        let stopped_timer = UnitStatus {
            active: ActiveState::Inactive,
            enabled: false,
            detail: "dead".into(),
            result: "success".into(),
        };

        assert_eq!(
            offline_mirror_state_from_status(
                OfflineEngine::Rclone,
                Some(&failed),
                Some(&stopped_timer)
            ),
            SyncRuntimeState::Paused
        );
    }

    #[test]
    fn detected_network_manager_profiles_use_uuid_identity() {
        let profile = network_manager_profile(NetworkManagerVpnProfile {
            name: "Work".into(),
            uuid: "91a601dd-2df4-4b32-bc66-25a16a7612fe".into(),
            vpn_type: "wireguard".into(),
            service_type: None,
        });

        assert_eq!(profile.name, "Work");
        assert_eq!(profile.kind, VpnKind::NetworkManager);
        assert_eq!(
            profile.external_profile_id.as_deref(),
            Some("91a601dd-2df4-4b32-bc66-25a16a7612fe")
        );
        assert!(same_vpn_reference(&profile, &profile));
    }

    #[test]
    fn detected_openconnect_profiles_allow_interactive_authentication_time() {
        let detected = network_manager_profile(NetworkManagerVpnProfile {
            name: "UA Engineering OpenConnect".into(),
            uuid: "02af3f27-a55b-48e5-9ffd-518b37e02b6d".into(),
            vpn_type: "vpn".into(),
            service_type: Some("org.freedesktop.NetworkManager.openconnect".into()),
        });
        assert_eq!(detected.timeout_seconds, 90);

        let mut existing = detected.clone();
        existing.name = "Old name".into();
        existing.timeout_seconds = 30;
        existing.readiness_checks.clear();
        update_detected_vpn_profile(&mut existing, &detected);
        assert_eq!(existing.name, "UA Engineering OpenConnect");
        assert_eq!(existing.timeout_seconds, 90);
        assert_eq!(
            existing.readiness_checks,
            vec![cosmic_ext_applet_mounter::model::ReadinessCheck::NetworkManagerState]
        );

        existing.timeout_seconds = 30;
        apply_openconnect_timeout(&mut existing, true);
        assert_eq!(existing.timeout_seconds, 90);
        existing.timeout_seconds = 30;
        apply_openconnect_timeout(&mut existing, false);
        assert_eq!(existing.timeout_seconds, 30);
    }

    #[test]
    fn app_nmcli_parser_recovers_flattened_applet_output() {
        let profiles = parse_nmcli_profiles_for_app(
            "Jarvis-5G:802-11-wireless:9000c4f8-da8d-4cf5-baea-9d747e3161ee \
             Pixel 6 Network:bluetooth:01637915-76fb-44ee-a7d8-50519b92176f \
             SalterLab:wireguard:51424a59-495c-4483-ad44-a0bf49327d5e \
             Wired connection 1:802-3-ethernet:7097f8f6-a5d9-3425-8e2c-e7d7ef12b8a0",
        );

        assert_eq!(
            profiles,
            vec![NetworkManagerVpnProfile {
                name: "SalterLab".into(),
                uuid: "51424a59-495c-4483-ad44-a0bf49327d5e".into(),
                vpn_type: "wireguard".into(),
                service_type: None,
            }]
        );
        let fallback_profiles = parse_nmcli_profiles_for_app(
            "Jarvis-5G:9000c4f8-da8d-4cf5-baea-9d747e3161ee:802-11-wireless \
             Pixel 6 Network:01637915-76fb-44ee-a7d8-50519b92176f:bluetooth \
             SalterLab:51424a59-495c-4483-ad44-a0bf49327d5e:wireguard \
             Wired connection 1:7097f8f6-a5d9-3425-8e2c-e7d7ef12b8a0:802-3-ethernet",
        );
        assert!(
            fallback_profiles
                .iter()
                .all(|profile| !profile.name.is_empty())
        );
    }

    #[test]
    fn vpn_import_dedupes_by_backend_reference() {
        let mut first = network_manager_profile(NetworkManagerVpnProfile {
            name: "Work".into(),
            uuid: "91a601dd-2df4-4b32-bc66-25a16a7612fe".into(),
            vpn_type: "wireguard".into(),
            service_type: None,
        });
        let second = network_manager_profile(NetworkManagerVpnProfile {
            name: "Renamed Work".into(),
            uuid: "91a601dd-2df4-4b32-bc66-25a16a7612fe".into(),
            vpn_type: "wireguard".into(),
            service_type: None,
        });
        let other = network_manager_profile(NetworkManagerVpnProfile {
            name: "Other".into(),
            uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            vpn_type: "wireguard".into(),
            service_type: None,
        });

        assert!(same_vpn_reference(&first, &second));
        assert!(!same_vpn_reference(&first, &other));

        first.kind = VpnKind::LegacyCisco;
        assert!(!same_vpn_reference(&first, &second));
    }

    #[test]
    fn rclone_remote_parser_keeps_provider_backends_only() {
        let remotes = parse_rclone_remotes_for_app(
            r#"{
                "ua_box": {"type": "box", "token": "secret"},
                "ua_gdrive": {"type": "drive", "client_secret": "secret"},
                "ua_engr": {"type": "smb", "pass": "secret"},
                "work_teams": {"type": "onedrive", "drive_type": "documentLibrary", "drive_id": "b!verified", "token": "secret"},
                "personal_onedrive": {"type": "onedrive", "drive_type": "personal"},
                "scratch": {"type": "local"},
                "malformed": "ignored"
            }"#,
        )
        .unwrap();

        assert_eq!(
            remotes,
            vec![
                RcloneDraftRemote {
                    name: "ua_box".into(),
                    backend: "box".into(),
                    drive_type: None,
                },
                RcloneDraftRemote {
                    name: "ua_gdrive".into(),
                    backend: "drive".into(),
                    drive_type: None,
                },
                RcloneDraftRemote {
                    name: "personal_onedrive".into(),
                    backend: "onedrive".into(),
                    drive_type: Some(cosmic_ext_applet_mounter::onedrive::DriveType::Personal),
                },
                RcloneDraftRemote {
                    name: "work_teams".into(),
                    backend: "onedrive".into(),
                    drive_type: None,
                },
                RcloneDraftRemote {
                    name: "ua_engr".into(),
                    backend: "smb".into(),
                    drive_type: None,
                },
            ]
        );
    }

    #[test]
    fn smb_remote_details_parser_loads_non_secret_fields_only() {
        let details = parse_smb_remote_details(
            r#"{
                "ua_engr": {
                    "type": "smb",
                    "host": "engr-drive.bluecat.arizona.edu",
                    "user": "uutzinger",
                    "domain": "BLUECAT",
                    "pass": "*** ENCRYPTED ***"
                },
                "ua_box": {"type": "box"}
            }"#,
            "ua_engr",
        )
        .expect("valid rclone dump")
        .expect("SMB remote details");

        assert_eq!(details.host, "engr-drive.bluecat.arizona.edu");
        assert_eq!(details.user, "uutzinger");
        assert_eq!(details.domain, "BLUECAT");
        assert_eq!(
            parse_smb_remote_details(r#"{"ua_box": {"type": "box"}}"#, "ua_box")
                .expect("valid rclone dump"),
            None
        );
        assert_eq!(
            parse_smb_remote_details(r#"{"ua_engr": {"type": "smb"}}"#, "missing")
                .expect("valid rclone dump"),
            None
        );
    }

    #[test]
    fn rclone_remote_matching_is_provider_specific() {
        let app = AppModel {
            rclone_remotes: vec![
                RcloneDraftRemote {
                    name: "box".into(),
                    backend: "box".into(),
                    drive_type: None,
                },
                RcloneDraftRemote {
                    name: "drive".into(),
                    backend: "drive".into(),
                    drive_type: None,
                },
                RcloneDraftRemote {
                    name: "share".into(),
                    backend: "smb".into(),
                    drive_type: None,
                },
            ],
            ..AppModel::default()
        };

        assert_eq!(
            app.matching_rclone_remotes(Provider::Box),
            vec![RcloneDraftRemote {
                name: "box".into(),
                backend: "box".into(),
                drive_type: None,
            }]
        );
        assert_eq!(
            app.matching_rclone_remotes(Provider::GoogleDrive),
            vec![RcloneDraftRemote {
                name: "drive".into(),
                backend: "drive".into(),
                drive_type: None,
            }]
        );
        assert!(app.matching_rclone_remotes(Provider::OneDrive).is_empty());
    }

    #[test]
    fn teams_draft_keeps_library_identity_separate_from_local_target() {
        let draft = ConnectionDraft {
            name: "Work Teams".into(),
            provider: Provider::Teams,
            remote_reference: "work_teams".into(),
            sharepoint_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
                .into(),
            local_path: "/home/example/Cloud/Work Teams".into(),
            ..ConnectionDraft::default()
        };
        let connection = connection_from_draft(&draft).unwrap();
        assert_eq!(connection.remote_reference, "work_teams");
        assert_eq!(
            connection.local_path,
            PathBuf::from("/home/example/Cloud/Work Teams")
        );
        assert_eq!(
            connection.teams_identity.as_ref().unwrap().library_url,
            draft.sharepoint_library_url
        );
        let mut offline = draft;
        offline.access_mode = AccessMode::OfflineMirror;
        offline.remote_subpath = "Disposable".into();
        offline.teams_drive_id = "b!verified".into();
        let offline_connection = connection_from_draft(&offline).unwrap();
        assert!(is_rclone_offline_mirror(&offline_connection));
        assert!(managed_plan_summary(&offline_connection).is_ok());
    }

    #[tokio::test]
    async fn sharepoint_background_schedule_requires_initial_sync() {
        let temporary = tempfile::tempdir().unwrap();
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: None,
            recovery_directory: temporary.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = temporary.path().join("local");
        connection.remote_subpath = Some("Disposable".into());
        connection.teams_identity = Some(
            cosmic_ext_applet_mounter::model::SharePointLibraryIdentity {
                site_url: "https://example.sharepoint.com/sites/Work".into(),
                library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
                drive_id: "b!verified".into(),
            },
        );
        let plan = rclone_bisync_plan(&connection, &temporary.path().join("work")).unwrap();
        assert!(
            start_rclone_offline_mirror_background(&connection, &plan, CancellationToken::new(),)
                .await
                .unwrap_err()
                .contains("requires a successful Preview")
        );
    }

    #[test]
    fn teams_test_uses_verified_drive_and_discards_changed_drafts() {
        let draft = ConnectionDraft {
            name: "Work Teams".into(),
            provider: Provider::Teams,
            remote_reference: "work_teams".into(),
            sharepoint_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
                .into(),
            local_path: "/home/example/Cloud/Work Teams".into(),
            ..ConnectionDraft::default()
        };
        let mut verified = connection_from_draft(&draft).unwrap();
        assert!(managed_plan_summary(&verified).is_err());
        verified.teams_identity.as_mut().unwrap().drive_id = "b!verified".into();
        assert!(managed_plan_summary(&verified).is_ok());
        assert!(sharepoint_verification_matches_draft(&draft, &verified));

        let mut changed = draft.clone();
        changed.remote_reference = "other_remote".into();
        assert!(!sharepoint_verification_matches_draft(&changed, &verified));
        changed = draft.clone();
        changed.sharepoint_library_url =
            "https://example.sharepoint.com/sites/Other/Shared%20Documents".into();
        assert!(!sharepoint_verification_matches_draft(&changed, &verified));
    }

    #[test]
    fn sharepoint_offline_test_retains_automatic_recovery_target() {
        use cosmic::Application;
        let mut app = AppModel::default();
        let _ = app.update(Message::DraftProvider(Provider::Teams));
        let stable_id = app.draft.id.expect("SharePoint draft needs a stable ID");
        let _ = app.update(Message::DraftAccessMode(AccessMode::OfflineMirror));
        assert_eq!(app.draft.id, Some(stable_id));

        let draft = ConnectionDraft {
            id: app.draft.id,
            name: "SharePoint Offline UI Test".into(),
            provider: Provider::Teams,
            access_mode: AccessMode::OfflineMirror,
            remote_reference: "work_teams".into(),
            remote_subpath: "Disposable/test".into(),
            sharepoint_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
                .into(),
            local_path: "/tmp/sharepoint-offline-ui-test".into(),
            ..ConnectionDraft::default()
        };
        let mut verified = connection_from_draft(&draft).unwrap();
        verified.teams_identity.as_mut().unwrap().drive_id = "b!verified".into();
        assert!(sharepoint_verification_matches_draft(&draft, &verified));

        let mut changed = draft.clone();
        changed.recovery_directory = "/tmp/another-recovery".into();
        assert!(!sharepoint_verification_matches_draft(&changed, &verified));
    }

    #[test]
    fn teams_online_service_routes_through_guarded_rclone_with_bounded_preload_support() {
        let mut connection = test_connection(Provider::Teams);
        connection.remote_subpath = Some("General".into());
        connection.teams_identity = Some(
            cosmic_ext_applet_mounter::model::SharePointLibraryIdentity {
                site_url: "https://example.sharepoint.com/sites/Work".into(),
                library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
                drive_id: "b!verified".into(),
            },
        );
        if let ConnectionMode::OnlineMount(options) = &mut connection.mode {
            options.start_at_login = true;
        }
        assert!(is_rclone_online_mount(&connection));
        assert!(!is_onedriver_online_mount(&connection));
        assert!(uses_directory_preload(&connection));
        assert!(directory_preload_waits_for_root(&connection));
        let mut document = ConfigDocument::default();
        let default_policy = document.preload_policy_for(&connection);
        assert!(!default_policy.enabled);
        assert_eq!(default_policy.maximum_seconds, 30);
        assert_eq!(default_policy.maximum_depth, Some(2));
        document.preload.sharepoint.enabled = true;
        assert!(document.preload_policy_for(&connection).enabled);

        let mut mirror = connection.clone();
        mirror.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        assert!(!uses_directory_preload(&mirror));
        assert!(!directory_preload_waits_for_root(&mirror));
        assert_eq!(
            managed_unit_names_for_connection(&connection),
            vec![UnitName::new(connection.id, UnitKind::Service)]
        );

        let plan = rclone_mount_plan(
            &connection,
            Path::new("/run/user/1000/cosmic-mounter"),
            Path::new("/home/example/.cache/cosmic-mounter"),
        )
        .expect("Teams rclone plan");
        assert_eq!(plan.remote, "remote:General");
        let unit = UnitDocument::service(&plan.service).expect("Teams service");
        assert!(unit.content.contains("--verify-teams-mount"));
        assert!(unit.content.contains("b!verified"));
        assert!(unit.content.contains("WantedBy=default.target"));
        assert!(unit.content.find("ExecCondition=") < unit.content.find("ExecStart="));
    }

    #[test]
    fn every_online_provider_routes_through_a_managed_mount_engine() {
        for provider in [
            Provider::OneDrive,
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            let connection = test_connection(provider);
            let expected = if provider == Provider::OneDrive {
                OnlineMountEngine::Onedriver
            } else {
                OnlineMountEngine::Rclone
            };
            assert_eq!(online_mount_engine(&connection), Some(expected));

            let mut offline = connection;
            offline.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
            assert_eq!(online_mount_engine(&offline), None);
        }
    }

    #[test]
    fn sharepoint_marker_and_retarget_confirmation_are_safeguarded() {
        let temporary = tempfile::tempdir().unwrap();
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: None,
            recovery_directory: temporary.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = temporary.path().join("local");
        connection.remote_subpath = Some("Disposable".into());
        connection.teams_identity = Some(
            cosmic_ext_applet_mounter::model::SharePointLibraryIdentity {
                site_url: "https://example.sharepoint.com/sites/Work".into(),
                library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
                drive_id: "b!verified".into(),
            },
        );
        fs::create_dir_all(&connection.local_path).unwrap();
        let plan = rclone_bisync_plan(&connection, &temporary.path().join("work")).unwrap();
        let marker = connection
            .local_path
            .join(plan.access_marker_name.as_ref().unwrap());
        ensure_sharepoint_access_marker(&connection, &plan).unwrap();
        assert_eq!(
            fs::read_to_string(&marker).unwrap(),
            format!("{}\n", connection.id)
        );
        fs::write(&marker, "changed\n").unwrap();
        assert!(ensure_sharepoint_access_marker(&connection, &plan).is_err());
        fs::write(&marker, format!("{}\n", connection.id)).unwrap();
        fs::create_dir_all(&plan.work_directory).unwrap();
        fs::write(initial_sync_marker(&plan), "complete\n").unwrap();
        fs::remove_file(&marker).unwrap();
        assert!(ensure_sharepoint_access_marker(&connection, &plan).is_err());

        let old_preview = initial_preview_marker(&plan);
        let old_sync = initial_sync_marker(&plan);
        let listing = plan.work_directory.join("path1.lst");
        fs::write(&old_preview, "preview\n").unwrap();
        fs::write(&listing, "listing\n").unwrap();
        let mut changed = connection.clone();
        changed.remote_subpath = Some("Other-Disposable".into());
        let work_root = temporary.path().join("work");
        let changed_plan = rclone_bisync_plan(&changed, &work_root).unwrap();
        assert_ne!(plan.work_directory, changed_plan.work_directory);
        invalidate_sharepoint_mirror_confirmation_on_retarget(&connection, &changed, &work_root)
            .unwrap();
        assert!(!old_preview.exists());
        assert!(!old_sync.exists());
        assert!(listing.exists());
        fs::create_dir_all(&changed_plan.work_directory).unwrap();
        fs::write(initial_preview_marker(&changed_plan), "other preview\n").unwrap();
        invalidate_sharepoint_mirror_confirmation_on_retarget(&changed, &connection, &work_root)
            .unwrap();
        assert!(!initial_preview_marker(&changed_plan).exists());
        assert!(!old_preview.exists());
    }

    #[test]
    fn smb_remote_setup_builds_redacted_noninteractive_request() {
        let mut draft = ConnectionDraft {
            provider: Provider::Smb,
            remote_reference: "test_smb".into(),
            smb_host: "files.example.edu".into(),
            smb_user: "uutzinger".into(),
            smb_domain: "UA".into(),
            smb_password: "secret passphrase".into(),
            ..ConnectionDraft::default()
        };

        let setup = SmbRemoteSetup::from_draft(&draft).expect("valid setup");
        let request = smb_rclone_config_create_request(&setup).expect("request");
        let command = request.sanitized_command();

        assert!(command.contains("rclone config create test_smb smb host [REDACTED]"));
        assert!(command.contains(" user [REDACTED]"));
        assert!(command.contains(" domain [REDACTED]"));
        assert!(command.contains(" --non-interactive"));
        assert!(!command.contains("files.example.edu"));
        assert!(!command.contains("uutzinger"));
        assert!(!command.contains("UA"));
        assert_eq!(setup.password.as_deref(), Some("secret passphrase"));
        let password_request =
            smb_rclone_config_password_request(&setup).expect("password request");
        assert_eq!(
            password_request.sanitized_command(),
            "rclone config password test_smb pass [REDACTED]"
        );
        assert!(
            !password_request
                .sanitized_command()
                .contains("secret passphrase")
        );

        draft.smb_user.clear();
        draft.smb_domain.clear();
        draft.smb_password.clear();
        let setup = SmbRemoteSetup::from_draft(&draft).expect("optional user/domain");
        assert_eq!(setup.user, None);
        assert_eq!(setup.domain, None);
        assert_eq!(setup.password, None);
    }

    #[test]
    fn smb_remote_setup_rejects_missing_host_and_bad_name() {
        let mut draft = ConnectionDraft {
            provider: Provider::Smb,
            remote_reference: "test_smb".into(),
            smb_host: String::new(),
            ..ConnectionDraft::default()
        };

        let setup = SmbRemoteSetup::from_draft(&draft).expect("missing host is allowed for update");
        let error = smb_rclone_config_create_request(&setup).expect_err("create needs host");
        assert!(error.contains("SMB host is required"));

        draft.smb_host = "files.example.edu".into();
        draft.remote_reference = "bad remote".into();
        let error = SmbRemoteSetup::from_draft(&draft).expect_err("bad name must fail");
        assert!(error.contains("rclone remote name"));

        draft.remote_reference = "test_smb".into();
        draft.smb_host = "files.example.edu\nshare".into();
        let error = SmbRemoteSetup::from_draft(&draft).expect_err("control char must fail");
        assert!(error.contains("unsupported control characters"));
    }

    #[test]
    fn box_remote_setup_builds_local_browser_oauth_request() {
        let draft = ConnectionDraft {
            provider: Provider::Box,
            remote_reference: "test_box".into(),
            ..ConnectionDraft::default()
        };

        let setup = BoxRemoteSetup::from_draft(&draft).expect("valid setup");
        let request = box_rclone_config_create_request(&setup).expect("request");

        assert_eq!(
            request.sanitized_command(),
            "rclone config create test_box box config_is_local true --non-interactive"
        );
        assert_eq!(request.timeout, Duration::from_secs(5 * 60));
    }

    #[test]
    fn google_drive_remote_setup_builds_local_browser_oauth_request() {
        let draft = ConnectionDraft {
            provider: Provider::GoogleDrive,
            remote_reference: "test_drive".into(),
            ..ConnectionDraft::default()
        };

        let setup = GoogleDriveRemoteSetup::from_draft(&draft).expect("valid setup");
        let request = google_drive_rclone_config_create_request(&setup).expect("request");

        assert_eq!(
            request.sanitized_command(),
            "rclone config create test_drive drive scope drive config_is_local true --obscure --non-interactive"
        );
        assert_eq!(request.timeout, Duration::from_secs(5 * 60));

        let custom = GoogleDriveRemoteSetup {
            name: "test_drive".into(),
            client_id: Some("private-client.apps.googleusercontent.com".into()),
            client_secret: Some("private-secret".into()),
        };
        let request = google_drive_rclone_config_create_request(&custom).expect("custom request");
        assert_eq!(
            request.sanitized_command(),
            "rclone config create test_drive drive client_id [REDACTED] client_secret [REDACTED] scope drive config_is_local true --obscure --non-interactive"
        );
        assert!(!request.sanitized_command().contains("private-client"));
        assert!(!request.sanitized_command().contains("private-secret"));
    }

    #[test]
    fn box_remote_setup_rejects_bad_name() {
        let draft = ConnectionDraft {
            provider: Provider::Box,
            remote_reference: "bad remote".into(),
            ..ConnectionDraft::default()
        };

        let error = BoxRemoteSetup::from_draft(&draft).expect_err("bad name must fail");
        assert!(error.contains("rclone remote name"));
    }

    #[test]
    fn google_drive_remote_setup_rejects_bad_name() {
        let mut draft = ConnectionDraft {
            provider: Provider::GoogleDrive,
            remote_reference: "bad remote".into(),
            ..ConnectionDraft::default()
        };

        let error = GoogleDriveRemoteSetup::from_draft(&draft).expect_err("bad name must fail");
        assert!(error.contains("rclone remote name"));

        draft.remote_reference = "valid_remote".into();
        draft.google_client_id = "client.apps.googleusercontent.com".into();
        let error = GoogleDriveRemoteSetup::from_draft(&draft)
            .expect_err("client ID without secret must fail");
        assert!(error.contains("must be entered together"));

        draft.google_client_id.clear();
        draft.google_client_secret = "secret".into();
        let error = GoogleDriveRemoteSetup::from_draft(&draft)
            .expect_err("client secret without ID must fail");
        assert!(error.contains("must be entered together"));
    }

    #[tokio::test]
    async fn create_box_remote_blocks_duplicates_and_uses_fixed_commands() {
        let duplicate_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        duplicate_runner.push(Ok(command_output(r#"{"existing": {"type": "box"}}"#)));
        let duplicate = BoxRemoteSetup {
            name: "existing".into(),
        };

        let error = create_box_rclone_remote_with(&duplicate_runner, duplicate)
            .await
            .expect_err("duplicate must fail");
        assert!(error.contains("already exists"));
        let requests = duplicate_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let malformed_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        malformed_runner.push(Ok(command_output(r#"{"broken": {}}"#)));
        let malformed = BoxRemoteSetup {
            name: "broken".into(),
        };

        let error = create_box_rclone_remote_with(&malformed_runner, malformed)
            .await
            .expect_err("malformed existing section must block create");
        assert!(error.contains("already exists"));
        let requests = malformed_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("{}")));
        runner.push(Ok(command_output("")));
        let setup = BoxRemoteSetup {
            name: "new_box".into(),
        };

        let created = create_box_rclone_remote_with(&runner, setup)
            .await
            .expect("new remote should be created");
        assert_eq!(created, "new_box");
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config create new_box box config_is_local true --non-interactive"
        );

        let cleanup_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        cleanup_runner.push(Ok(command_output("{}")));
        cleanup_runner.push(Err(nonzero_command_error("failed to start auth webserver")));
        cleanup_runner.push(Ok(command_output("")));
        let setup = BoxRemoteSetup {
            name: "failed_box".into(),
        };

        let error = create_box_rclone_remote_with(&cleanup_runner, setup)
            .await
            .expect_err("failed create must be reported");
        assert!(error.contains("failed to start auth webserver"));
        let requests = cleanup_runner.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config create failed_box box config_is_local true --non-interactive"
        );
        assert_eq!(
            requests[2].sanitized_command(),
            "rclone config delete failed_box"
        );
    }

    #[tokio::test]
    async fn google_drive_remote_create_and_update_use_fixed_redacted_commands() {
        let duplicate_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        duplicate_runner.push(Ok(command_output(r#"{"existing": {"type": "drive"}}"#)));
        let duplicate = GoogleDriveRemoteSetup {
            name: "existing".into(),
            client_id: None,
            client_secret: None,
        };

        let error = apply_google_drive_rclone_remote_with(
            &duplicate_runner,
            duplicate,
            GoogleDriveRemoteAction::Create,
        )
        .await
        .expect_err("duplicate create must fail");
        assert!(error.contains("already exists"));
        let requests = duplicate_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let missing_credentials_runner =
            cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
                .with_resolved([Executable::Rclone]);
        missing_credentials_runner.push(Ok(command_output(r#"{"existing": {"type": "drive"}}"#)));
        let error = apply_google_drive_rclone_remote_with(
            &missing_credentials_runner,
            GoogleDriveRemoteSetup {
                name: "existing".into(),
                client_id: None,
                client_secret: None,
            },
            GoogleDriveRemoteAction::Update,
        )
        .await
        .expect_err("update without credentials must fail");
        assert!(error.contains("client ID and matching client secret"));
        assert_eq!(missing_credentials_runner.requests().len(), 1);

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("{}")));
        runner.push(Ok(command_output("")));
        let setup = GoogleDriveRemoteSetup {
            name: "new_drive".into(),
            client_id: None,
            client_secret: None,
        };

        let created =
            apply_google_drive_rclone_remote_with(&runner, setup, GoogleDriveRemoteAction::Create)
                .await
                .expect("new remote should be created");
        assert_eq!(created.remote_name, "new_drive");
        assert!(!created.updated);
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config create new_drive drive scope drive config_is_local true --obscure --non-interactive"
        );

        let update_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        update_runner.push(Ok(command_output(r#"{"existing": {"type": "drive"}}"#)));
        update_runner.push(Ok(command_output("")));
        let updated = apply_google_drive_rclone_remote_with(
            &update_runner,
            GoogleDriveRemoteSetup {
                name: "existing".into(),
                client_id: Some("private-client.apps.googleusercontent.com".into()),
                client_secret: Some("private-secret".into()),
            },
            GoogleDriveRemoteAction::Update,
        )
        .await
        .expect("existing Drive remote should update");
        assert!(updated.updated);
        let requests = update_runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config update existing client_id [REDACTED] client_secret [REDACTED] config_is_local true --obscure"
        );
        assert!(!requests[1].sanitized_command().contains("private-client"));
        assert!(!requests[1].sanitized_command().contains("private-secret"));
    }

    #[tokio::test]
    async fn google_drive_oauth_update_rejects_an_unanswered_rclone_prompt() {
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output(r#"{"existing": {"type": "drive"}}"#)));
        runner.push(Ok(command_output(
            r#"{"State":"*oauth-confirm,teamdrive,oauth,,","Option":{"Name":"config_refresh_token"},"Error":"","Result":""}"#,
        )));

        let error = apply_google_drive_rclone_remote_with(
            &runner,
            GoogleDriveRemoteSetup {
                name: "existing".into(),
                client_id: Some("private-client.apps.googleusercontent.com".into()),
                client_secret: Some("private-secret".into()),
            },
            GoogleDriveRemoteAction::Update,
        )
        .await
        .expect_err("an unanswered OAuth prompt must not report success");

        assert!(error.contains("did not complete OAuth"));
    }

    #[tokio::test]
    async fn create_smb_remote_blocks_duplicates_and_uses_fixed_commands() {
        let duplicate_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        duplicate_runner.push(Ok(command_output(r#"{"existing": {"type": "box"}}"#)));
        let duplicate = SmbRemoteSetup {
            name: "existing".into(),
            host: Some("files.example.edu".into()),
            user: Some("uutzinger".into()),
            domain: Some("UA".into()),
            password: None,
        };

        let error = create_smb_rclone_remote_with(&duplicate_runner, duplicate)
            .await
            .expect_err("duplicate must fail");
        assert!(error.contains("already exists"));
        let requests = duplicate_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let update_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        update_runner.push(Ok(command_output(r#"{"existing_smb": {"type": "smb"}}"#)));
        update_runner.push(Ok(command_output("")));
        let update = SmbRemoteSetup {
            name: "existing_smb".into(),
            host: Some("files.example.edu".into()),
            user: Some("uutzinger".into()),
            domain: Some("BLUECAT".into()),
            password: None,
        };

        let updated = create_smb_rclone_remote_with(&update_runner, update)
            .await
            .expect("existing SMB remote should update metadata");
        assert_eq!(updated.remote_name, "existing_smb");
        assert!(!updated.password_updated);
        let requests = update_runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        let command = requests[1].sanitized_command();
        assert!(command.contains("rclone config update existing_smb"));
        assert!(command.contains(" --non-interactive"));
        assert!(!command.contains("files.example.edu"));
        assert!(!command.contains("uutzinger"));
        assert!(!command.contains("BLUECAT"));

        let password_update_runner =
            cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
                .with_resolved([Executable::Rclone]);
        password_update_runner.push(Ok(command_output(r#"{"existing_smb": {"type": "smb"}}"#)));
        password_update_runner.push(Ok(command_output("")));
        let password_update = SmbRemoteSetup {
            name: "existing_smb".into(),
            host: None,
            user: None,
            domain: None,
            password: Some("rotated secret".into()),
        };

        let updated = create_smb_rclone_remote_with(&password_update_runner, password_update)
            .await
            .expect("existing SMB remote should allow password-only update");
        assert_eq!(updated.remote_name, "existing_smb");
        assert!(updated.password_updated);
        let requests = password_update_runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config password existing_smb pass [REDACTED]"
        );
        assert!(!requests[1].sanitized_command().contains("rotated"));

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("{}")));
        runner.push(Ok(command_output("")));
        runner.push(Ok(command_output("")));
        let setup = SmbRemoteSetup {
            name: "new_smb".into(),
            host: Some("files.example.edu".into()),
            user: Some("uutzinger".into()),
            domain: Some("UA".into()),
            password: Some("secret passphrase".into()),
        };

        let created = create_smb_rclone_remote_with(&runner, setup)
            .await
            .expect("new remote should be created");
        assert_eq!(created.remote_name, "new_smb");
        assert!(created.password_updated);
        let requests = runner.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let command = requests[1].sanitized_command();
        assert!(command.contains("rclone config create new_smb smb"));
        assert!(command.contains(" --non-interactive"));
        assert!(!command.contains("files.example.edu"));
        assert!(!command.contains("uutzinger"));
        assert!(!command.contains("UA"));
        assert_eq!(
            requests[2].sanitized_command(),
            "rclone config password new_smb pass [REDACTED]"
        );
        assert!(!requests[2].sanitized_command().contains("secret"));
    }

    #[tokio::test]
    async fn remove_rclone_remote_uses_config_delete() {
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("")));

        let removed = remove_rclone_remote_with(&runner, "unused_box".into())
            .await
            .expect("unused remote should be removed");

        assert_eq!(removed, "unused_box");
        let requests = runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].sanitized_command(),
            "rclone config delete unused_box"
        );
    }

    #[test]
    fn rclone_remote_removal_blocks_saved_connection_references() {
        let mut app = AppModel::default();
        let mut connection = test_connection(Provider::Box);
        connection.remote_reference = "ua_box".into();
        app.config.document.connections = vec![connection];

        assert!(app.rclone_remote_is_referenced("UA_BOX"));
        assert!(!app.rclone_remote_is_referenced("unused_box"));
    }

    #[test]
    fn rclone_access_target_uses_optional_subtree() {
        let mut connection = test_connection(Provider::Box);
        connection.remote_reference = "ua_box".into();
        connection.remote_subpath = Some("Utzinger/cosmic-mounter-ui-test".into());
        assert_eq!(
            rclone_access_target(&connection),
            "ua_box:Utzinger/cosmic-mounter-ui-test"
        );

        connection.remote_subpath = None;
        assert_eq!(rclone_access_target(&connection), "ua_box:");
    }

    #[test]
    fn rclone_access_errors_are_actionable() {
        let not_found = CommandError::NonZero {
            command: "rclone lsf".into(),
            code: Some(1),
            stdout: cosmic_ext_applet_mounter::process::CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: cosmic_ext_applet_mounter::process::CapturedOutput {
                text: "directory not found".into(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
        };
        assert!(rclone_access_error("remote:path", not_found).contains("does not exist"));

        let timeout = CommandError::Timeout {
            command: "rclone lsf".into(),
            timeout: Duration::from_secs(20),
        };
        assert!(rclone_access_error("remote:path", timeout).contains("timed out"));
    }

    #[test]
    fn sftp_access_errors_identify_failures_without_echoing_rclone_output() {
        let cases = [
            ("knownhosts: key mismatch", "host-key verification"),
            ("knownhosts: key is unknown", "host-key verification"),
            (
                "couldn't parse known_hosts_file: open /secret/path: no such file",
                "known-hosts file",
            ),
            (
                "failed to load key_file /secret/key: permission denied",
                "private-key file",
            ),
            (
                "ssh: handshake failed: ssh: unable to authenticate, attempted methods [none publickey], no supported methods remain",
                "authentication failed",
            ),
            (
                "SSH_AUTH_SOCK is not set; ssh agent unavailable",
                "SSH agent",
            ),
            ("failed to list: permission denied", "not readable"),
            (
                "failed to list: directory not found",
                "directory was not found",
            ),
            ("dial tcp: connection refused", "could not be reached"),
            (
                "ssh: handshake failed: read tcp 127.0.0.1:1234->127.0.0.1:2222: i/o timeout",
                "connection timed out",
            ),
            (
                "unknown failure with password=hunter2",
                "access check failed",
            ),
        ];
        for (detail, expected) in cases {
            let error = CommandError::NonZero {
                command: "rclone lsf private-remote:".into(),
                code: Some(1),
                stdout: cosmic_ext_applet_mounter::process::CapturedOutput {
                    text: String::new(),
                    truncated: false,
                    invalid_utf8: false,
                },
                stderr: cosmic_ext_applet_mounter::process::CapturedOutput {
                    text: detail.into(),
                    truncated: false,
                    invalid_utf8: false,
                },
                attempts: 1,
            };
            let message = sftp_access_error(error);
            assert!(message.contains(expected), "{detail}: {message}");
            assert!(!message.contains("/secret/") && !message.contains("hunter2"));
            assert!(!message.contains("private-remote"));
        }

        let timeout = sftp_access_error(CommandError::Timeout {
            command: "rclone lsf private-remote:".into(),
            timeout: Duration::from_secs(20),
        });
        assert!(timeout.contains("timed out") && !timeout.contains("private-remote"));
        let spawn = sftp_access_error(CommandError::Spawn {
            command: "rclone lsf private-remote:".into(),
            message: "password=hunter2".into(),
        });
        assert!(spawn.contains("Could not start rclone") && !spawn.contains("hunter2"));
    }

    #[test]
    fn offline_mirror_preview_summary_reports_confirmation_context() {
        let output = command_output(
            "Path2 to Path1: copy local.txt\nPath1 to Path2: copy remote.txt\nDelete old.txt\nConflict same.txt\nSkipped native.gdoc\nTransferred: 15 MiB\n",
        );

        let summary = preview_summary_text(&output);

        assert!(summary.contains("uploads 1"));
        assert!(summary.contains("downloads 1"));
        assert!(summary.contains("deletes 1"));
        assert!(summary.contains("conflicts 1"));
        assert!(summary.contains("skipped 1"));
        assert!(summary.contains("15.0 MiB"));
        assert!(summary.contains("destructive changes detected"));
        assert!(
            sync_rejection_message(SyncDecisionRejection::PreviewRequired)
                .contains("Preview first")
        );
    }

    #[test]
    fn offline_mirror_save_notice_reflects_preserved_initialization_state() {
        let mut rclone = test_connection(Provider::Teams);
        rclone.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        rclone.name = "SharePoint mirror".into();
        let uninitialized = offline_mirror_saved_notice(&rclone, false);
        assert!(uninitialized.contains("confirmed initial Sync Now"));
        assert!(!uninitialized.contains("normal bisync"));
        let initialized = offline_mirror_saved_notice(&rclone, true);
        assert!(initialized.contains("Existing initialization state was preserved"));
        assert!(initialized.contains("normal bisync"));
        assert!(!initialized.contains("confirmed initial Sync Now"));

        let mut onedrive = test_connection(Provider::OneDrive);
        onedrive.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        onedrive.name = "OneDrive mirror".into();
        let uninitialized = offline_mirror_saved_notice(&onedrive, false);
        assert!(uninitialized.contains("confirmed initial Sync Now"));
        assert!(!uninitialized.contains("normal OneDrive synchronization"));
        let initialized = offline_mirror_saved_notice(&onedrive, true);
        assert!(initialized.contains("Existing initialization state was preserved"));
        assert!(initialized.contains("normal OneDrive synchronization"));
        assert!(!initialized.contains("confirmed initial Sync Now"));
    }

    #[test]
    fn offline_mirror_markers_and_filters_stay_in_work_directory() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mirror = temp.path().join("mirror");
        let work = temp.path().join("work");
        let recovery = temp.path().join("recovery");
        let mut connection = test_connection(Provider::GoogleDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: None,
            recovery_directory: recovery,
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.remote_reference = "uutzinger_gdrive".into();
        connection.remote_subpath = Some("cosmic-mounter-ui-test".into());
        connection.local_path = mirror.clone();

        let plan = rclone_bisync_plan(&connection, &work).expect("plan");
        prepare_rclone_bisync_work_files(&connection, &plan).expect("work files");
        write_initial_preview_marker(&plan, "Preview: uploads 0.").expect("preview marker");
        write_initial_sync_marker(&plan).expect("sync marker");

        assert!(plan.filters_file.starts_with(&plan.work_directory));
        assert!(initial_preview_marker(&plan).starts_with(&plan.work_directory));
        assert!(initial_sync_marker(&plan).starts_with(&plan.work_directory));
        assert!(!initial_preview_marker(&plan).starts_with(&mirror));
        assert!(!initial_sync_marker(&plan).starts_with(&mirror));
        assert!(
            fs::read_to_string(&plan.filters_file)
                .expect("filters")
                .contains("Google cloud-native documents")
        );
    }

    #[test]
    fn onedriver_online_validation_reports_missing_binary() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_connection(Provider::OneDrive);
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default();
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = verify_onedriver_online_mount_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .expect_err("missing onedriver must fail");

        assert!(error.contains("onedriver is missing"));
    }

    #[test]
    fn onedriver_online_validation_reports_missing_auth_state() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_connection(Provider::OneDrive);
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = verify_onedriver_online_mount_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .expect_err("missing app-owned config must fail");

        assert!(error.contains("not authenticated"));
        assert!(error.contains("cache"));
    }

    #[test]
    fn onedriver_online_validation_reports_active_mount_overlap() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();
        mounts.set(vec![MountEntry {
            target: temp.path().join("Cloud"),
            source: "onedriver".into(),
            filesystem: "fuse.onedriver".into(),
            options: vec!["rw".into()],
        }]);

        let error = verify_onedriver_online_mount_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .expect_err("active overlapping onedriver mount must fail");

        assert!(error.contains("overlaps an active onedriver mount"));
    }

    #[test]
    fn onedriver_online_validation_accepts_authenticated_metadata() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let cache_root = temp.path().join("cache");
        let config_root = temp.path().join("config");
        let plan = onedriver_mount_plan(&connection, &cache_root, &config_root).expect("plan");
        fs::create_dir_all(plan.config_file.parent().expect("config parent")).expect("config dir");
        fs::write(&plan.config_file, "{}").expect("config");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message = verify_onedriver_online_mount_setup_with(
            &connection,
            &runner,
            &mounts,
            &cache_root,
            &config_root,
        )
        .expect("authenticated metadata should pass");

        assert!(message.contains("onedriver setup is present"));
        assert!(message.contains("Cache directory"));
    }

    #[test]
    fn onedriver_online_validation_accepts_cache_token_metadata() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let cache_root = temp.path().join("cache");
        let config_root = temp.path().join("config");
        let plan = onedriver_mount_plan(&connection, &cache_root, &config_root).expect("plan");
        let token_directory = plan.cache_directory.join("mount-cache");
        fs::create_dir_all(&token_directory).expect("token dir");
        fs::write(token_directory.join("auth_tokens.json"), "{}").expect("token metadata");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message = verify_onedriver_online_mount_setup_with(
            &connection,
            &runner,
            &mounts,
            &cache_root,
            &config_root,
        )
        .expect("cache token metadata should pass");

        assert!(message.contains("onedriver setup is present"));
    }

    #[test]
    fn draft_paths_expand_home_shorthand() {
        let home = home_dir().expect("HOME should be available in tests");
        assert_eq!(expand_user_path("~/Cloud/Test"), home.join("Cloud/Test"));
        assert_eq!(expand_user_path("~"), home);
        assert_eq!(
            expand_user_path("/tmp/cosmic-mounter-test"),
            PathBuf::from("/tmp/cosmic-mounter-test")
        );
    }

    #[test]
    fn offline_mirror_blank_recovery_defaults_next_to_mirror_directory() {
        let mut draft = ConnectionDraft {
            id: Some(ConnectionId::from_uuid(
                Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("uuid"),
            )),
            provider: Provider::OneDrive,
            access_mode: AccessMode::OfflineMirror,
            name: "OneDrive Mirror".into(),
            remote_reference: "onedrive-personal-test".into(),
            local_path: "/home/example/Cloud/OneDrive Mirror".into(),
            recovery_directory: String::new(),
            ..ConnectionDraft::default()
        };

        let connection = connection_from_draft(&draft).expect("connection");

        let ConnectionMode::OfflineMirror(options) = connection.mode else {
            panic!("expected offline mirror");
        };
        assert_eq!(
            options.recovery_directory,
            PathBuf::from(
                "/home/example/Cloud/.cosmic-mounter-recovery/OneDrive_Mirror-11111111-2222-3333-4444-555555555555"
            )
        );
        assert!(recovery_directory_placeholder(&draft).contains(".cosmic-mounter-recovery"));

        draft.recovery_directory = "/tmp/custom-recovery".into();
        let connection = connection_from_draft(&draft).expect("custom connection");
        let ConnectionMode::OfflineMirror(options) = connection.mode else {
            panic!("expected offline mirror");
        };
        assert_eq!(
            options.recovery_directory,
            PathBuf::from("/tmp/custom-recovery")
        );
    }

    #[test]
    fn onedriver_auth_request_uses_app_owned_paths_and_auth_only() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let plan = onedriver_mount_plan(
            &connection,
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .expect("plan");

        let request = onedriver_auth_request(&plan).expect("auth request");
        let command = request.sanitized_command();

        assert!(command.starts_with("onedriver --auth-only --config-file "));
        assert!(command.contains(" --cache-dir "));
        assert!(command.contains(&plan.config_file.display().to_string()));
        assert!(command.contains(&plan.cache_directory.display().to_string()));
        assert!(command.ends_with(&plan.mountpoint.display().to_string()));
        assert_eq!(request.timeout, Duration::from_secs(5 * 60));
    }

    #[tokio::test]
    async fn onedriver_online_setup_runs_auth_and_reuses_validation() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let cache_root = temp.path().join("cache");
        let config_root = temp.path().join("config");
        let plan = onedriver_mount_plan(&connection, &cache_root, &config_root).expect("plan");
        fs::create_dir_all(plan.config_file.parent().expect("config parent")).expect("config dir");
        fs::write(&plan.config_file, "{}").expect("config");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        runner.push(Ok(command_output("authenticated")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message = run_onedriver_online_setup_with(
            &runner,
            &mounts,
            &connection,
            &cache_root,
            &config_root,
        )
        .await
        .expect("setup should pass");

        assert!(message.contains("onedriver setup is present"));
        assert!(plan.mountpoint.is_dir());
        assert!(plan.cache_directory.is_dir());
        let requests = runner.requests();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].sanitized_command().contains(" --auth-only "));
    }

    #[tokio::test]
    async fn onedriver_online_setup_reports_auth_command_failure() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mut connection = test_connection(Provider::OneDrive);
        connection.local_path = temp.path().join("Cloud/OneDrive");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Onedriver]);
        runner.push(Err(CommandError::NonZero {
            command: "onedriver --auth-only".into(),
            code: Some(1),
            stdout: cosmic_ext_applet_mounter::process::CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: cosmic_ext_applet_mounter::process::CapturedOutput {
                text: "browser authorization failed".into(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
        }));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = run_onedriver_online_setup_with(
            &runner,
            &mounts,
            &connection,
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .await
        .expect_err("auth failure should be reported");

        assert!(error.contains("browser authorization failed"));
    }

    #[tokio::test]
    async fn onedrive_offline_validation_reports_missing_binary() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default();
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = verify_onedrive_offline_mirror_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("config"),
        )
        .await
        .expect_err("missing onedrive must fail");

        assert!(error.contains("onedrive is missing"));
    }

    #[tokio::test]
    async fn onedrive_offline_validation_reports_missing_auth_state() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = verify_onedrive_offline_mirror_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("config"),
        )
        .await
        .expect_err("missing refresh token must fail");

        assert!(error.contains("not authenticated"));
        assert!(error.contains("refresh_token"));
    }

    #[tokio::test]
    async fn onedrive_offline_validation_reports_onedriver_overlap() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();
        mounts.set(vec![MountEntry {
            target: temp.path().join("mirror"),
            source: "onedriver".into(),
            filesystem: "fuse.onedriver".into(),
            options: vec!["rw".into()],
        }]);

        let error = verify_onedrive_offline_mirror_setup_with(
            &connection,
            &runner,
            &mounts,
            &temp.path().join("config"),
        )
        .await
        .expect_err("active onedriver overlap must fail");

        assert!(error.contains("overlaps active onedriver mount"));
    }

    #[tokio::test]
    async fn onedrive_offline_validation_maps_oauth_and_resync_errors() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        write_onedrive_refresh_token(&connection, &config_root);
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Err(nonzero_command_error(
            "AADSTS70000: The provided value for the code parameter is not valid. The code has expired.",
        )));
        let error =
            verify_onedrive_offline_mirror_setup_with(&connection, &runner, &mounts, &config_root)
                .await
                .expect_err("expired OAuth response must fail");
        assert!(error.contains("expired or invalid"));

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Err(nonzero_command_error(
            "The application requires authorisation, which involves saving authentication data on your system. Application authorisation cannot be completed when using the '--dry-run' option.",
        )));
        let error =
            verify_onedrive_offline_mirror_setup_with(&connection, &runner, &mounts, &config_root)
                .await
                .expect_err("authorization-required dry-run must fail");
        assert!(error.contains("not authenticated"));

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Err(nonzero_command_error(
            "An application configuration change has been detected where a --resync is required",
        )));
        let error =
            verify_onedrive_offline_mirror_setup_with(&connection, &runner, &mounts, &config_root)
                .await
                .expect_err("resync requirement must fail");
        assert!(error.contains("resync/state rebuild"));
    }

    #[tokio::test]
    async fn onedrive_offline_validation_accepts_authenticated_dry_run() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        write_onedrive_refresh_token(&connection, &config_root);
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Ok(command_output(
            "Sync with Microsoft OneDrive is complete\n",
        )));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message =
            verify_onedrive_offline_mirror_setup_with(&connection, &runner, &mounts, &config_root)
                .await
                .expect("authenticated dry-run should pass");

        assert!(message.contains("setup is authenticated"));
        let requests = runner.requests();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].sanitized_command().contains(" --dry-run"));
        assert!(
            requests[0]
                .sanitized_command()
                .contains(" --single-directory ")
        );
    }

    #[tokio::test]
    async fn onedrive_mirror_setup_runs_interactive_auth_and_validates_preview() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        write_onedrive_refresh_token(&connection, &config_root);
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Ok(command_output("authorized")));
        runner.push(Ok(command_output("No changes required")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message =
            run_onedrive_mirror_interactive_setup_with(&runner, &mounts, &connection, &config_root)
                .await
                .expect("setup should pass");

        assert!(message.contains("abraunegg/onedrive setup is authenticated"));
        assert!(connection.local_path.is_dir());
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        let auth_command = requests[0].sanitized_command();
        assert!(auth_command.contains("onedrive --confdir "));
        assert!(auth_command.contains(" --reauth"));
        assert!(!auth_command.contains(" --auth-files "));
        assert!(!auth_command.contains(" --dry-run "));
        assert!(!auth_command.contains(" --syncdir "));
        assert!(requests[1].sanitized_command().contains(" --dry-run"));
    }

    #[tokio::test]
    async fn onedrive_mirror_interactive_setup_requires_created_refresh_token() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Ok(command_output("authorization window closed")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error =
            run_onedrive_mirror_interactive_setup_with(&runner, &mounts, &connection, &config_root)
                .await
                .expect_err("missing refresh token should fail clearly");

        assert!(error.contains("interactive authorization did not create"));
        assert!(error.contains("Manual Auth Handoff"));
        assert_eq!(runner.requests().len(), 1);
    }

    #[tokio::test]
    async fn onedrive_mirror_manual_setup_runs_auth_files_and_validates_preview() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        write_onedrive_refresh_token(&connection, &config_root);
        let auth_files = OneDriveMirrorAuthFiles {
            auth_url_file: temp.path().join("auth-url"),
            response_url_file: temp.path().join("response-url"),
        };
        fs::write(&auth_files.auth_url_file, "stale url").expect("stale url");
        fs::write(&auth_files.response_url_file, "secret response").expect("stale response");
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Ok(command_output("authorized")));
        runner.push(Ok(command_output("No changes required")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message = run_onedrive_mirror_manual_setup_with(
            &runner,
            &mounts,
            &connection,
            &config_root,
            &auth_files,
        )
        .await
        .expect("setup should pass");

        assert!(message.contains("abraunegg/onedrive setup is authenticated"));
        assert!(connection.local_path.is_dir());
        assert!(!auth_files.auth_url_file.exists());
        assert!(!auth_files.response_url_file.exists());
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        let auth_command = requests[0].sanitized_command();
        assert!(auth_command.contains("onedrive --confdir "));
        assert!(auth_command.contains(" --reauth --auth-files "));
        assert!(!auth_command.contains(" --dry-run "));
        assert!(!auth_command.contains(" --syncdir "));
        assert!(auth_command.contains(&auth_files.auth_url_file.display().to_string()));
        assert!(auth_command.contains(&auth_files.response_url_file.display().to_string()));
        assert!(requests[1].sanitized_command().contains(" --dry-run"));
    }

    #[tokio::test]
    async fn onedrive_mirror_manual_setup_reports_auth_failure_and_cleans_response() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        let auth_files = OneDriveMirrorAuthFiles {
            auth_url_file: temp.path().join("auth-url"),
            response_url_file: temp.path().join("response-url"),
        };
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Err(nonzero_command_error("authentication failed")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let error = run_onedrive_mirror_manual_setup_with(
            &runner,
            &mounts,
            &connection,
            &config_root,
            &auth_files,
        )
        .await
        .expect_err("auth failure should be reported");

        assert!(error.contains("authentication failed"));
        assert!(!auth_files.response_url_file.exists());
    }

    #[tokio::test]
    async fn onedrive_mirror_manual_setup_accepts_auth_files_post_token_usage_error() {
        let temp = tempfile::TempDir::new().expect("temp");
        let connection = test_onedrive_offline_connection(temp.path());
        let config_root = temp.path().join("config");
        write_onedrive_refresh_token(&connection, &config_root);
        let auth_files = OneDriveMirrorAuthFiles {
            auth_url_file: temp.path().join("auth-url"),
            response_url_file: temp.path().join("response-url"),
        };
        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::OneDrive]);
        runner.push(Err(nonzero_command_error(
            "Your command line input is missing either the \"--sync\" or \"--monitor\" switches.",
        )));
        runner.push(Ok(command_output("No changes required")));
        let mounts = cosmic_ext_applet_mounter::mounts::FakeMountTable::default();

        let message = run_onedrive_mirror_manual_setup_with(
            &runner,
            &mounts,
            &connection,
            &config_root,
            &auth_files,
        )
        .await
        .expect("post-token usage error should continue to validation");

        assert!(message.contains("abraunegg/onedrive setup is authenticated"));
        assert!(!auth_files.auth_url_file.exists());
        assert!(!auth_files.response_url_file.exists());
    }

    #[test]
    fn onedrive_auth_handoff_command_and_response_file_are_transient() {
        let temp = tempfile::TempDir::new().expect("temp");
        let auth_files = OneDriveMirrorAuthFiles {
            auth_url_file: temp.path().join("auth-url"),
            response_url_file: temp.path().join("response-url"),
        };
        assert_eq!(
            onedrive_auth_open_command(&auth_files),
            format!("xdg-open \"$(cat {})\"", auth_files.auth_url_file.display())
        );
        validate_onedrive_auth_url(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
        )
        .expect("valid auth url");
        assert!(validate_onedrive_auth_url("https://example.com/authorize").is_err());
        let response = "https://login.microsoftonline.com/common/oauth2/nativeclient?code=abc";
        validate_onedrive_auth_response_url(response).expect("valid response");
        assert!(validate_onedrive_auth_response_url("https://example.com/?code=abc").is_err());

        write_onedrive_auth_response(&auth_files, response).expect("write response");

        assert_eq!(
            fs::read_to_string(&auth_files.response_url_file).expect("response file"),
            response
        );
    }

    #[test]
    fn onedrive_auth_flatpak_defaults_to_manual_native_keeps_interactive() {
        assert_eq!(
            default_onedrive_auth_mode(true),
            OneDriveMirrorSetupMode::ManualAuthFiles
        );
        assert_eq!(
            default_onedrive_auth_mode(false),
            OneDriveMirrorSetupMode::Interactive
        );
    }

    #[test]
    fn onedrive_auth_handoff_uses_scoped_private_shared_directory() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("state/onedrive-auth");
        let first = onedrive_auth_files_in(ConnectionId::new(), &root);
        let second = onedrive_auth_files_in(ConnectionId::new(), &root);
        assert_eq!(first.auth_url_file.parent(), Some(root.as_path()));
        assert_ne!(first, second);
        prepare_onedrive_auth_files(&first).unwrap();
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::write(&first.auth_url_file, "synthetic auth URL").unwrap();
        fs::write(&first.response_url_file, "synthetic response").unwrap();
        cleanup_onedrive_auth_files(&first);
        assert!(!first.auth_url_file.exists());
        assert!(!first.response_url_file.exists());
    }

    #[tokio::test]
    async fn onedrive_auth_open_rejects_invalid_url_before_launch() {
        let temp = tempfile::tempdir().unwrap();
        let files = onedrive_auth_files_in(ConnectionId::new(), temp.path());
        fs::write(&files.auth_url_file, "https://example.test/authorize").unwrap();
        assert!(open_onedrive_auth_url(&files).await.is_err());
    }

    #[test]
    fn onedrive_auth_open_completion_does_not_update_changed_draft() {
        let mut app = AppModel::default();
        app.draft.id = Some(ConnectionId::new());
        app.last_notice = Some("keep current draft notice".into());
        let _ = cosmic::Application::update(
            &mut app,
            Message::OneDriveAuthUrlOpened(ConnectionId::new(), Ok("old request".into())),
        );
        assert_eq!(
            app.last_notice.as_deref(),
            Some("keep current draft notice")
        );
    }

    #[test]
    fn onedrive_setup_guidance_matches_modes_and_save_notice_mentions_validation() {
        assert!(onedrive_setup_guidance(AccessMode::OnlineMount).contains("onedriver"));
        assert!(onedrive_setup_guidance(AccessMode::OfflineMirror).contains("authorization"));
        assert!(
            onedrive_account_help(AccessMode::OnlineMount).contains("Test Connection and Save")
        );
        assert!(
            onedrive_account_help(AccessMode::OfflineMirror).contains("Test Connection and Save")
        );
        assert_eq!(save_notice_name("Test", None), "Test");
        assert!(
            save_notice_name("Test", Some("onedriver setup is present"))
                .contains("passed validation")
        );
    }

    #[test]
    fn modify_validation_rejects_duplicate_name_and_local_overlap() {
        let mut app = AppModel::default();
        let mut first = test_connection(Provider::Box);
        first.id = ConnectionId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("uuid"),
        );
        first.name = "Existing Box".into();
        first.local_path = PathBuf::from("/home/example/Cloud/Box");

        let mut second = test_connection(Provider::GoogleDrive);
        second.id = ConnectionId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("uuid"),
        );
        second.name = "Google".into();
        second.local_path = PathBuf::from("/home/example/Cloud/Google");

        app.config.document.connections = vec![first.clone(), second.clone()];

        let mut duplicate_name = second.clone();
        duplicate_name.name = "existing box".into();
        let error = app
            .validate_connection_edit(&duplicate_name)
            .expect_err("duplicate name must fail");
        assert!(error.contains("already used"));

        let mut nested_path = second;
        nested_path.local_path = PathBuf::from("/home/example/Cloud/Box/Nested");
        let error = app
            .validate_connection_edit(&nested_path)
            .expect_err("nested local target must fail");
        assert!(error.contains("overlaps"));
    }

    #[test]
    fn add_validation_rejects_duplicate_name_and_local_overlap() {
        let mut app = AppModel::default();
        let mut saved = test_connection(Provider::Box);
        saved.name = "Saved Box".into();
        saved.local_path = PathBuf::from("/home/example/Cloud/Box");
        app.config.document.connections = vec![saved];

        let mut duplicate_name = test_connection(Provider::GoogleDrive);
        duplicate_name.id = ConnectionId::new();
        duplicate_name.name = "saved box".into();
        duplicate_name.local_path = PathBuf::from("/home/example/Cloud/Google");
        let error = app
            .validate_connection_edit(&duplicate_name)
            .expect_err("duplicate add name must fail");
        assert!(error.contains("already used"));

        let mut nested_path = test_connection(Provider::GoogleDrive);
        nested_path.id = ConnectionId::new();
        nested_path.name = "New Google".into();
        nested_path.local_path = PathBuf::from("/home/example/Cloud/Box/Nested");
        let error = app
            .validate_connection_edit(&nested_path)
            .expect_err("nested add local target must fail");
        assert!(error.contains("overlaps"));
    }

    #[test]
    fn selected_folder_path_reuses_existing_local_target_validation() {
        let mut app = AppModel::default();
        let mut saved = test_connection(Provider::Box);
        saved.name = "Saved Box".into();
        saved.local_path = PathBuf::from("/home/example/Cloud/Box");
        app.config.document.connections = vec![saved];

        let mut draft = ConnectionDraft {
            name: "New Box".into(),
            provider: Provider::Box,
            access_mode: AccessMode::OnlineMount,
            remote_reference: "new_box".into(),
            local_path: selected_folder_path(Path::new("/home/example/Cloud/Box/Nested")),
            ..ConnectionDraft::default()
        };
        draft.id = Some(ConnectionId::new());
        let connection = connection_from_draft(&draft).expect("selected folder draft");

        let error = app
            .validate_connection_edit(&connection)
            .expect_err("selected nested folder must fail existing validation");
        assert!(error.contains("overlaps"));
    }

    #[test]
    fn document_portal_folder_paths_parse_to_original_host_paths() {
        assert!(is_document_portal_path(Path::new(
            "/run/user/1000/doc/edc79c37/box_test"
        )));
        assert!(!is_document_portal_path(Path::new(
            "/home/example/Cloud/box_test"
        )));
        assert_eq!(
            parse_document_portal_origin(
                "id: edc79c37\npath: /run/user/1000/doc/edc79c37/box_test\norigin: /home/example/Cloud/box_test\n"
            ),
            Some(PathBuf::from("/home/example/Cloud/box_test"))
        );
    }

    #[test]
    fn document_portal_ids_are_extracted_from_direct_and_app_scoped_paths() {
        let mount = Path::new("/run/user/1000/doc");
        assert_eq!(
            portal_document_id(Path::new("/run/user/1000/doc/edc79c37/box_test"), mount),
            Some("edc79c37".into())
        );
        assert_eq!(
            portal_document_id(
                Path::new(
                    "/run/user/1000/doc/by-app/io.github.uutzinger.cosmic-ext-applet-mounter/edc79c37/box_test"
                ),
                mount
            ),
            Some("edc79c37".into())
        );
        assert_eq!(
            portal_document_id(Path::new("/run/user/1000/doc/../secret"), mount),
            None
        );
    }

    #[test]
    fn document_portal_file_paths_accept_one_terminal_nul_only() {
        assert_eq!(
            portal_file_path(b"/home/example/Cloud\0".to_vec()).unwrap(),
            PathBuf::from("/home/example/Cloud")
        );
        assert!(portal_file_path(b"relative/path\0".to_vec()).is_err());
        assert!(portal_file_path(b"/home/example\0/Cloud".to_vec()).is_err());
    }

    #[tokio::test]
    #[ignore = "requires the live Documents portal and an existing exported document"]
    async fn live_documents_portal_dbus_resolves_existing_entry() {
        let connection = zbus::Connection::session().await.expect("session bus");
        let proxy = zbus::Proxy::new(
            &connection,
            DOCUMENTS_DESTINATION,
            DOCUMENTS_PATH,
            DOCUMENTS_INTERFACE,
        )
        .await
        .expect("Documents portal");
        let mount = portal_file_path(proxy.call("GetMountPoint", &()).await.expect("mount point"))
            .expect("valid mount point");
        let documents: HashMap<String, Vec<u8>> =
            proxy.call("List", &("",)).await.expect("document list");
        let (document_id, expected) = documents.iter().next().expect("an exported document");
        let expected = portal_file_path(expected.clone()).expect("valid host path");
        let portal_path = mount.join(document_id);
        assert_eq!(
            document_portal_origin_path_dbus(&portal_path, false)
                .await
                .expect("resolve document"),
            expected
        );
    }

    #[test]
    fn modify_validation_rejects_provider_and_mode_changes() {
        let mut app = AppModel::default();
        let saved = test_connection(Provider::Box);
        app.config.document.connections = vec![saved.clone()];

        let mut provider_changed = saved.clone();
        provider_changed.provider = Provider::GoogleDrive;
        let error = app
            .validate_connection_edit(&provider_changed)
            .expect_err("provider change must fail");
        assert!(error.contains("Provider changes are disabled"));

        let mut mode_changed = saved;
        mode_changed.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig::default());
        let error = app
            .validate_connection_edit(&mode_changed)
            .expect_err("mode change must fail");
        assert!(error.contains("Access mode changes are disabled"));
    }

    #[test]
    fn shared_remote_requires_acknowledgement_warning() {
        let mut app = AppModel::default();
        let mut first = test_connection(Provider::Box);
        first.id = ConnectionId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("uuid"),
        );
        first.name = "Box One".into();
        first.remote_reference = "shared_box".into();

        let mut second = test_connection(Provider::Box);
        second.id = ConnectionId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("uuid"),
        );
        second.name = "Box Two".into();
        second.remote_reference = "SHARED_BOX".into();

        app.config.document.connections = vec![first, second.clone()];

        let warning = app
            .shared_remote_warning(&second)
            .expect("same provider remote should warn");
        assert!(warning.contains("same Box account/remote"));
        assert!(warning.contains("Box One"));
    }

    #[test]
    fn new_onedrive_account_labels_are_distinct() {
        let first = ConnectionId::from_uuid(
            Uuid::parse_str("990cc48f-4e4e-4ed7-a07b-c545ad3d3f9d").expect("uuid"),
        );
        let second = ConnectionId::from_uuid(
            Uuid::parse_str("5ffc5d9b-6721-49f6-81d2-5f39c15061b7").expect("uuid"),
        );
        assert_ne!(
            default_onedrive_account_label(first),
            default_onedrive_account_label(second)
        );
    }

    #[tokio::test]
    async fn popup_switch_persists_online_startup_intent_and_unit_enablement() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let document_path = temporary.path().join("document");
        let storage =
            AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(document_path.clone()));
        let connection = test_connection(Provider::GoogleDrive);
        let connection_id = connection.id;
        let mut config = Config {
            document: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
        };
        let manager = FakeSystemdManager::default();

        persist_online_popup_state_with(&mut config, &storage, &manager, connection_id, true)
            .await
            .expect("persist popup on state");
        let ConnectionMode::OnlineMount(options) = &config.document.connections[0].mode else {
            panic!("expected Online mount");
        };
        assert!(options.start_at_login);
        let persisted: ConfigDocument =
            ron::from_str(&fs::read_to_string(&document_path).expect("read persisted document"))
                .expect("parse persisted document");
        let ConnectionMode::OnlineMount(options) = &persisted.connections[0].mode else {
            panic!("expected persisted Online mount");
        };
        assert!(options.start_at_login);

        persist_online_popup_state_with(&mut config, &storage, &manager, connection_id, false)
            .await
            .expect("persist popup off state");
        let ConnectionMode::OnlineMount(options) = &config.document.connections[0].mode else {
            panic!("expected Online mount");
        };
        assert!(!options.start_at_login);
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [SystemdAction::Enable, SystemdAction::Disable]
        );
    }

    #[tokio::test]
    async fn online_repair_state_reconciliation_disables_every_provider() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let document_path = temporary.path().join("document");
        let storage =
            AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(document_path.clone()));
        let providers = [
            Provider::OneDrive,
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ];
        let connections = providers
            .into_iter()
            .enumerate()
            .map(|(index, provider)| {
                let mut connection = test_connection(provider);
                connection.name = format!("Repair matrix {index}");
                connection.local_path = PathBuf::from(format!("/tmp/cosmic-repair-{index}"));
                if provider == Provider::Teams {
                    connection.teams_identity = Some(
                        cosmic_ext_applet_mounter::model::SharePointLibraryIdentity {
                            site_url: "https://example.sharepoint.com/sites/Work".into(),
                            library_url: "https://example.sharepoint.com/sites/Work/Documents"
                                .into(),
                            drive_id: "b!repair-matrix".into(),
                        },
                    );
                }
                let ConnectionMode::OnlineMount(options) = &mut connection.mode else {
                    unreachable!("test connection is online")
                };
                options.start_at_login = true;
                connection
            })
            .collect::<Vec<_>>();
        let ids = connections
            .iter()
            .map(|connection| connection.id)
            .collect::<Vec<_>>();
        let mut config = Config {
            document: ConfigDocument {
                connections,
                ..ConfigDocument::default()
            },
        };
        let manager = FakeSystemdManager::default();

        for connection_id in &ids {
            persist_online_popup_state_with(&mut config, &storage, &manager, *connection_id, false)
                .await
                .expect("persist repaired connection off state");
        }

        assert!(config.document.connections.iter().all(|connection| {
            matches!(
                &connection.mode,
                ConnectionMode::OnlineMount(options) if !options.start_at_login
            )
        }));
        let persisted: ConfigDocument =
            ron::from_str(&fs::read_to_string(&document_path).expect("read persisted document"))
                .expect("parse persisted document");
        assert!(persisted.connections.iter().all(|connection| {
            matches!(
                &connection.mode,
                ConnectionMode::OnlineMount(options) if !options.start_at_login
            )
        }));
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            vec![SystemdAction::Disable; providers.len()]
        );
    }

    #[tokio::test]
    async fn popup_switch_restores_unit_policy_when_config_write_fails() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let blocked_parent = temporary.path().join("not-a-directory");
        fs::write(&blocked_parent, "blocking file").expect("write blocking file");
        let storage = AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(
            blocked_parent.join("document"),
        ));
        let connection = test_connection(Provider::OneDrive);
        let connection_id = connection.id;
        let mut config = Config {
            document: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
        };
        let manager = FakeSystemdManager::default();

        let error =
            persist_online_popup_state_with(&mut config, &storage, &manager, connection_id, true)
                .await
                .expect_err("configuration write must fail");
        assert!(error.contains("previous startup policy was restored"));
        let ConnectionMode::OnlineMount(options) = &config.document.connections[0].mode else {
            panic!("expected Online mount");
        };
        assert!(!options.start_at_login);
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [SystemdAction::Enable, SystemdAction::Disable]
        );
    }

    fn test_connection(provider: Provider) -> Connection {
        Connection {
            id: ConnectionId::default(),
            name: "Test".into(),
            provider,
            mode: ConnectionMode::OnlineMount(OnlineMountConfig::default()),
            remote_reference: "remote".into(),
            remote_subpath: None,
            local_path: PathBuf::from("/tmp/cosmic-test"),
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

    fn test_onedrive_offline_connection(root: &std::path::Path) -> Connection {
        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: None,
            recovery_directory: root.join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = root.join("mirror");
        connection.remote_subpath = Some("cosmic-mounter-test".into());
        connection
    }

    fn write_onedrive_refresh_token(connection: &Connection, config_root: &std::path::Path) {
        let plan = one_drive_mirror_plan(
            connection,
            config_root,
            &OneDriveIsolationReport {
                active_onedriver_paths: Vec::new(),
            },
        )
        .expect("plan");
        fs::create_dir_all(&plan.config_directory).expect("config dir");
        fs::write(
            plan.config_directory.join("refresh_token"),
            "token metadata only",
        )
        .expect("refresh token");
    }

    fn nonzero_command_error(stderr_text: &str) -> CommandError {
        use cosmic_ext_applet_mounter::process::CapturedOutput;

        CommandError::NonZero {
            command: "onedrive --sync --dry-run".into(),
            code: Some(1),
            stdout: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: stderr_text.into(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
        }
    }

    #[test]
    fn bisync_lock_error_gives_bounded_recovery_guidance_without_a_work_path() {
        let message = offline_mirror_command_error(
            "sync",
            nonzero_command_error(
                "Failed to bisync: prior lock file found: /home/user/private/work/state.lck",
            ),
        );
        assert!(message.contains("two minutes"));
        assert!(message.contains("older locks may need manual recovery"));
        assert!(!message.contains("/home/user/private"));
    }

    #[test]
    fn bisync_failure_is_short_and_does_not_echo_logs_or_paths() {
        let message = offline_mirror_command_error(
            "sync",
            nonzero_command_error(
                "INFO: syncing /home/user/private/mirror with secret_remote:folder\n"
                    .repeat(200)
                    .as_str(),
            ),
        );
        assert!(message.contains("app-owned mirror log"));
        assert!(message.len() < 200);
        assert!(!message.contains("/home/user/private"));
        assert!(!message.contains("secret_remote"));
    }

    #[test]
    fn reorder_connection_moves_item_to_selected_position() {
        let mut first = test_connection(Provider::Box);
        first.id = ConnectionId::new();
        first.local_path = PathBuf::from("/tmp/cosmic-test/first");
        let mut second = test_connection(Provider::GoogleDrive);
        second.id = ConnectionId::new();
        second.local_path = PathBuf::from("/tmp/cosmic-test/second");
        let mut third = test_connection(Provider::Smb);
        third.id = ConnectionId::new();
        third.local_path = PathBuf::from("/tmp/cosmic-test/third");
        let mut app = AppModel {
            config: {
                let mut config = Config::default();
                config.document.connections = vec![first.clone(), second.clone(), third.clone()];
                config
            },
            ..AppModel::default()
        };
        let temp = tempfile::tempdir().expect("temporary directory");
        let storage = AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(
            temp.path().join("document"),
        ));

        let _ = app.reorder_connection_with_storage(third.id, 1, &storage);
        assert_eq!(app.config.document.connections[0].id, third.id);
        assert_eq!(app.config.document.connections[1].id, first.id);
        assert_eq!(app.config.document.connections[2].id, second.id);

        let _ = app.reorder_connection_with_storage(first.id, 3, &storage);
        assert_eq!(app.config.document.connections[0].id, third.id);
        assert_eq!(app.config.document.connections[1].id, second.id);
        assert_eq!(app.config.document.connections[2].id, first.id);
    }

    #[test]
    fn reorder_connection_ignores_invalid_and_unchanged_positions() {
        let mut first = test_connection(Provider::Box);
        first.id = ConnectionId::new();
        first.local_path = PathBuf::from("/tmp/cosmic-test/first");
        let mut second = test_connection(Provider::GoogleDrive);
        second.id = ConnectionId::new();
        second.local_path = PathBuf::from("/tmp/cosmic-test/second");
        let mut app = AppModel {
            config: {
                let mut config = Config::default();
                config.document.connections = vec![first.clone(), second.clone()];
                config
            },
            ..AppModel::default()
        };
        let temp = tempfile::tempdir().expect("temporary directory");
        let storage = AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(
            temp.path().join("document"),
        ));

        let _ = app.reorder_connection_with_storage(first.id, 1, &storage);
        assert_eq!(app.config.document.connections[0].id, first.id);
        assert_eq!(app.config.document.connections[1].id, second.id);

        let _ = app.reorder_connection_with_storage(first.id, 0, &storage);
        assert_eq!(app.config.document.connections[0].id, first.id);

        let _ = app.reorder_connection_with_storage(first.id, 99, &storage);
        assert_eq!(app.config.document.connections[0].id, first.id);
    }

    fn command_output(stdout: &str) -> CommandOutput {
        use cosmic_ext_applet_mounter::process::CapturedOutput;

        CommandOutput {
            command: "rclone bisync".into(),
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

    #[test]
    fn successful_bisync_summary_keeps_retention_warning_actionable() {
        let outcome = BisyncOutcome {
            preview_pending: None,
            output: command_output(""),
            warnings: vec!["Remote retention cleanup was incomplete; run Sync Now again.".into()],
        };

        let summary = rclone_bisync_outcome_summary(
            &test_connection(Provider::Smb),
            BisyncMode::Sync,
            &outcome,
        );
        assert!(summary.contains("Sync completed"));
        assert!(summary.contains("Warning:"));
        assert!(summary.contains("run Sync Now again"));
    }
    fn onedrive_runtime_connection(
        root: &Path,
        kind: cosmic_ext_applet_mounter::model::OneDriveDriveType,
    ) -> Connection {
        let mut connection = test_connection(Provider::OneDrive);
        connection.name = "Runtime mirror".into();
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: Some(OfflineEngine::Rclone),
            recovery_directory: root.join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = root.join("local");
        connection.remote_reference = "synthetic_remote".into();
        connection.remote_subpath = Some("Test".into());
        connection.onedrive_identity = Some(OneDriveIdentity {
            drive_id: "synthetic-primary".into(),
            drive_type: kind,
        });
        connection
    }

    #[tokio::test]
    async fn onedrive_runtime_dispatch_uses_engine_and_preserves_legacy_and_online() {
        use cosmic_ext_applet_mounter::model::OneDriveDriveType;
        let root = tempfile::tempdir().unwrap();
        for kind in [OneDriveDriveType::Personal, OneDriveDriveType::Business] {
            let mut connection = onedrive_runtime_connection(root.path(), kind);
            assert!(is_rclone_offline_mirror(&connection));
            assert!(!is_legacy_onedrive_mirror(&connection));
            assert!(managed_plan_summary(&connection).unwrap().contains("timer"));
            let sync = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
            assert_eq!(sync.timer.interval, Duration::from_secs(15 * 60));
            assert!(
                start_rclone_offline_mirror_background(
                    &connection,
                    &sync,
                    CancellationToken::new()
                )
                .await
                .is_err()
            );
            assert!(
                draft_summary_text(&draft_from_connection(&connection)).contains("rclone bisync")
            );
            let plan = offline_repair_plan(&connection).unwrap();
            assert_eq!(plan.engine, OfflineRepairEngine::RcloneBisync);
            assert!(plan.timer.is_some());
            assert_eq!(
                managed_unit_names_for_connection(&connection),
                vec![
                    UnitName::new(connection.id, UnitKind::Timer),
                    UnitName::new(connection.id, UnitKind::Service)
                ]
            );
            connection.enabled = false;
            assert!(offline_repair_plan(&connection).is_ok());
            if let ConnectionMode::OfflineMirror(ref mut options) = connection.mode {
                options.engine = None;
            }
            assert!(!is_rclone_offline_mirror(&connection));
            assert!(is_legacy_onedrive_mirror(&connection));
            assert_eq!(
                offline_repair_plan(&connection).unwrap().engine,
                OfflineRepairEngine::OneDriveMonitor
            );
            assert_eq!(managed_unit_names_for_connection(&connection).len(), 1);
            connection.mode = ConnectionMode::OnlineMount(OnlineMountConfig::default());
            assert_eq!(
                online_mount_engine(&connection),
                Some(OnlineMountEngine::Onedriver)
            );
            assert!(!is_rclone_offline_mirror(&connection));
            assert!(!is_legacy_onedrive_mirror(&connection));
        }
    }

    #[test]
    fn onedrive_runtime_status_distinguishes_monitor_from_oneshot_and_timer() {
        let active = repair_unit_status(ActiveState::Active, true);
        let inactive = repair_unit_status(ActiveState::Inactive, false);
        let failed = repair_unit_status(ActiveState::Failed, false);
        assert_eq!(
            offline_mirror_state_from_status(OfflineEngine::LegacyOnedrive, Some(&active), None),
            SyncRuntimeState::Idle
        );
        assert_eq!(
            offline_mirror_state_from_status(OfflineEngine::Rclone, Some(&active), Some(&active)),
            SyncRuntimeState::Running
        );
        assert_eq!(
            offline_mirror_state_from_status(OfflineEngine::Rclone, Some(&inactive), Some(&active)),
            SyncRuntimeState::Idle
        );
        assert_eq!(
            offline_mirror_state_from_status(OfflineEngine::Rclone, Some(&failed), Some(&active)),
            SyncRuntimeState::Failed
        );
        assert_eq!(
            offline_mirror_state_from_status(
                OfflineEngine::Rclone,
                Some(&inactive),
                Some(&inactive)
            ),
            SyncRuntimeState::Paused
        );
    }

    #[tokio::test]
    async fn onedrive_runtime_save_installs_worker_and_timer_without_starting_or_enabling() {
        use cosmic_ext_applet_mounter::services::FakeSystemdManager;
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_runtime_connection(
            root.path(),
            cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
        );
        let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
        let service = UnitDocument::service(&mirror_script::service_spec(&plan).unwrap()).unwrap();
        let timer = UnitDocument::timer(&plan.timer).unwrap();
        let store =
            FileUnitStore::new(root.path().join("units"), Arc::new(StructuralUnitValidator));
        let manager = FakeSystemdManager::default();
        for _ in 0..7 {
            manager.push(Ok(None));
        }
        manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));
        install_paused_rclone_units(
            store.clone(),
            manager.clone(),
            &service,
            &timer,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert!(
            store
                .read(&service.name)
                .unwrap()
                .unwrap()
                .contains("--run-bisync")
        );
        assert!(store.read(&timer.name).unwrap().is_some());
        assert!(
            manager
                .actions()
                .iter()
                .all(|(action, _)| !matches!(action, SystemdAction::Start | SystemdAction::Enable))
        );
        assert!(!initial_sync_marker(&plan).exists());
        assert!(!initial_preview_marker(&plan).exists());
    }

    #[tokio::test]
    async fn onedrive_runtime_save_rejects_external_units_and_restores_previous_service_on_partial_failure()
     {
        use cosmic_ext_applet_mounter::services::{FakeSystemdManager, ServiceError};
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_runtime_connection(
            root.path(),
            cosmic_ext_applet_mounter::model::OneDriveDriveType::Business,
        );
        let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
        let service = UnitDocument::service(&mirror_script::service_spec(&plan).unwrap()).unwrap();
        let timer = UnitDocument::timer(&plan.timer).unwrap();
        let unit_dir = root.path().join("units");
        let store = FileUnitStore::new(unit_dir.clone(), Arc::new(StructuralUnitValidator));
        fs::create_dir_all(&unit_dir).unwrap();
        fs::write(
            unit_dir.join(service.name.file_name()),
            "[Unit]\nDescription=External\n",
        )
        .unwrap();
        let manager = FakeSystemdManager::default();
        assert!(
            install_paused_rclone_units(
                store.clone(),
                manager.clone(),
                &service,
                &timer,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert!(manager.actions().is_empty());
        fs::remove_file(unit_dir.join(service.name.file_name())).unwrap();
        let mut original = service.clone();
        original.content.push_str("# original managed service\n");
        store.write(&original).unwrap();
        store.write(&timer).unwrap();
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));
        manager.push(Ok(None));
        manager.push(Err(ServiceError::Runtime(
            "synthetic reload failure".into(),
        )));
        assert!(
            install_paused_rclone_units(
                store.clone(),
                manager.clone(),
                &service,
                &timer,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert_eq!(
            store.read(&service.name).unwrap().unwrap(),
            original.content
        );
        assert_eq!(store.read(&timer.name).unwrap().unwrap(), timer.content);
        assert!(
            manager
                .actions()
                .iter()
                .all(|(action, _)| !matches!(action, SystemdAction::Enable | SystemdAction::Start))
        );
    }

    #[tokio::test]
    async fn onedrive_runtime_resume_checks_observed_timer_and_rolls_back_failed_start() {
        use cosmic_ext_applet_mounter::services::{FakeSystemdManager, ServiceError};
        let timer = UnitName::new(ConnectionId::new(), UnitKind::Timer);
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(repair_unit_status(ActiveState::Active, true))));
        resume_rclone_timer_with(&manager, &timer, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Enable,
                SystemdAction::Start,
                SystemdAction::Status
            ]
        );
        for failure in [0, 1, 2] {
            let manager = FakeSystemdManager::default();
            for _ in 0..failure {
                manager.push(Ok(None));
            }
            manager.push(Err(ServiceError::Runtime("synthetic start failure".into())));
            assert!(
                resume_rclone_timer_with(&manager, &timer, CancellationToken::new())
                    .await
                    .is_err()
            );
            let actions = manager.actions();
            assert_eq!(actions[actions.len() - 2].0, SystemdAction::Stop);
            assert_eq!(actions.last().unwrap().0, SystemdAction::Disable);
        }
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let manager = FakeSystemdManager::default();
        assert!(
            resume_rclone_timer_with(&manager, &timer, cancelled)
                .await
                .is_err()
        );
        assert!(manager.actions().is_empty());
    }

    #[tokio::test]
    async fn onedrive_runtime_pause_requires_stopped_timer_and_service_without_resetting_failure() {
        use cosmic_ext_applet_mounter::services::{FakeSystemdManager, ServiceError};
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_runtime_connection(
            root.path(),
            cosmic_ext_applet_mounter::model::OneDriveDriveType::Business,
        );
        let plan = rclone_bisync_plan(&connection, &root.path().join("work")).unwrap();
        let manager = FakeSystemdManager::default();
        for _ in 0..2 {
            manager.push(Ok(None));
            manager.push(Ok(None));
            manager.push(Ok(Some(repair_unit_status(ActiveState::Inactive, false))));
        }
        pause_rclone_background_with(&manager, &plan, CancellationToken::new())
            .await
            .unwrap();
        assert!(
            manager
                .actions()
                .iter()
                .all(|(action, _)| *action != SystemdAction::ResetFailed)
        );
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Err(ServiceError::Runtime(
            "synthetic disable failure".into(),
        )));
        assert!(
            pause_rclone_background_with(&manager, &plan, CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(manager.actions().len(), 2);
        assert!(
            manager
                .actions()
                .iter()
                .all(|(_, unit)| unit.as_ref().unwrap().ends_with(".timer"))
        );
    }

    #[test]
    fn onedrive_runtime_summary_uses_structured_count_and_never_echoes_engine_logs() {
        let root = tempfile::tempdir().unwrap();
        let connection = onedrive_runtime_connection(
            root.path(),
            cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
        );
        let outcome = BisyncOutcome {
            output: command_output("private-secret Path1: 999 changes: Bisync successful"),
            warnings: vec![],
            preview_pending: Some(Ok(3)),
        };
        let summary = rclone_bisync_outcome_summary(&connection, BisyncMode::Preview, &outcome);
        assert!(summary.contains('3'));
        assert!(!summary.contains("999"));
        assert!(!summary.contains("private-secret"));
        assert!(offline_mirror_saved_notice(&connection, true).contains("normal bisync"));
    }

    fn onedrive_editor_draft(
        kind: cosmic_ext_applet_mounter::onedrive::DriveType,
    ) -> ConnectionDraft {
        ConnectionDraft {
            id: Some(ConnectionId::new()),
            name: "Synthetic mirror".into(),
            provider: Provider::OneDrive,
            access_mode: AccessMode::OfflineMirror,
            onedrive_drive_type: kind,
            onedrive_expected_account: "person@example.invalid".into(),
            remote_reference: "example_remote".into(),
            remote_subpath: "Test folder".into(),
            local_path: "/tmp/synthetic-mirror".into(),
            ..ConnectionDraft::default()
        }
    }

    #[test]
    fn onedrive_editor_discovery_separates_primary_types_libraries_and_online() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        let remotes = parse_rclone_remotes_for_app(
            r#"{
            "personal":{"type":"onedrive","drive_type":"personal"},
            "business":{"type":"onedrive","drive_type":"business"},
            "library":{"type":"onedrive","drive_type":"documentLibrary"},
            "incomplete":{"type":"onedrive"},
            "other":{"type":"box"}
        }"#,
        )
        .unwrap();
        for (kind, name) in [
            (DriveType::Personal, "personal"),
            (DriveType::Business, "business"),
        ] {
            let app = AppModel {
                draft: onedrive_editor_draft(kind),
                rclone_remotes: remotes.clone(),
                ..AppModel::default()
            };
            assert!(app.draft_uses_rclone());
            assert!(!app.draft_primary_actions_ready());
            assert_eq!(
                app.matching_rclone_remotes(Provider::OneDrive)[0].name,
                name
            );
            assert_eq!(app.matching_rclone_remotes(Provider::OneDrive).len(), 1);
            assert_eq!(
                app.matching_rclone_remotes(Provider::Teams)[0].name,
                "library"
            );
        }
        let mut app = AppModel {
            draft: onedrive_editor_draft(DriveType::Personal),
            rclone_remotes: remotes,
            ..AppModel::default()
        };
        app.draft.access_mode = AccessMode::OnlineMount;
        assert!(!app.draft_uses_rclone());
        assert!(app.matching_rclone_remotes(Provider::OneDrive).is_empty());
        app.draft.access_mode = AccessMode::OfflineMirror;
        app.draft.legacy_onedrive = true;
        assert!(!app.draft_uses_rclone());
        assert!(OneDriveEditorRequest::from_draft(&app.draft, &[], true).is_err());
        // Even a forged new-setup message cannot convert a legacy profile.
        let _ = app.setup_onedrive_rclone(true);
        assert!(app.onedrive_editor_pending.is_none());
    }

    #[test]
    fn onedrive_editor_acceptance_binds_identity_and_clears_on_edits() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        for kind in [DriveType::Personal, DriveType::Business] {
            let mut app = AppModel {
                draft: onedrive_editor_draft(kind),
                ..AppModel::default()
            };
            let request = OneDriveEditorRequest::from_draft(&app.draft, &[], false).unwrap();
            let identity = OneDriveIdentity {
                drive_id: "primary-drive".into(),
                drive_type: kind,
            };
            accept_onedrive_editor_result(&mut app.draft, &request, Ok(identity.clone())).unwrap();
            assert!(app.draft_primary_actions_ready());
            let saved = connection_from_draft(&app.draft).unwrap();
            assert_eq!(saved.offline_engine(), Some(OfflineEngine::Rclone));
            assert_eq!(saved.onedrive_identity, Some(identity));
            assert!(
                !serde_json::to_string(&saved)
                    .unwrap()
                    .contains("person@example.invalid")
            );
            let loaded = draft_from_connection(&saved);
            assert!(loaded.onedrive_expected_account.is_empty());
            assert_eq!(loaded.onedrive_drive_type, kind);
            assert!(onedrive_validation_matches_draft(&app.draft, &saved));
            let _ = app.update(Message::DraftSubpath("Another folder".into()));
            assert!(app.draft.onedrive_identity.is_none());
            assert!(!app.draft_primary_actions_ready());
            assert!(!onedrive_validation_matches_draft(&app.draft, &saved));
            assert!(
                accept_onedrive_editor_result(
                    &mut app.draft,
                    &request,
                    Ok(saved.onedrive_identity.unwrap())
                )
                .is_err()
            );
        }
    }

    #[test]
    fn onedrive_editor_reconnect_uses_saved_identity_and_failure_does_not_mutate_saved_profile() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        let mut draft = onedrive_editor_draft(DriveType::Business);
        draft.onedrive_identity = Some(OneDriveIdentity {
            drive_id: "saved-drive".into(),
            drive_type: DriveType::Business,
        });
        let saved = connection_from_draft(&draft).unwrap();
        draft.remote_reference = "replacement_remote".into();
        // A forged draft identity cannot authorize a different account.
        draft.onedrive_identity = Some(OneDriveIdentity {
            drive_id: "edited-drive".into(),
            drive_type: DriveType::Business,
        });
        let request =
            OneDriveEditorRequest::from_draft(&draft, std::slice::from_ref(&saved), true).unwrap();
        assert_eq!(request.setup.previous_identity, saved.onedrive_identity);
        let wrong = draft.onedrive_identity.clone().unwrap();
        assert!(accept_onedrive_editor_result(&mut draft, &request, Ok(wrong)).is_err());
        assert!(!draft.enabled);
        assert!(draft.onedrive_identity.is_none());
        assert!(saved.enabled);
        assert_eq!(
            saved.onedrive_identity.as_ref().unwrap().drive_id,
            "saved-drive"
        );
        accept_onedrive_editor_result(
            &mut draft,
            &request,
            Ok(saved.onedrive_identity.clone().unwrap()),
        )
        .unwrap();
        assert!(!draft.enabled); // Reconnect does not enable a failed/disabled draft.
        draft.onedrive_drive_type = DriveType::Personal;
        assert!(OneDriveEditorRequest::from_draft(&draft, &[saved], false).is_err());
    }

    #[test]
    fn onedrive_editor_cancel_and_late_completion_cannot_accept_or_save() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        let mut app = AppModel {
            draft: onedrive_editor_draft(DriveType::Personal),
            ..AppModel::default()
        };
        let request = OneDriveEditorRequest::from_draft(&app.draft, &[], false).unwrap();
        let token = CancellationToken::new();
        app.onedrive_editor_pending = Some(OneDriveEditorPending {
            cancellation: token.clone(),
        });
        app.onedrive_editor_generation = 2;
        let identity = OneDriveIdentity {
            drive_id: "primary-drive".into(),
            drive_type: DriveType::Personal,
        };
        let _ = app.update(Message::OneDriveRcloneCompleted(
            1,
            request.clone(),
            Ok(identity.clone()),
        ));
        assert!(app.onedrive_editor_pending.is_some());
        assert!(!token.is_cancelled());
        let _ = app.update(Message::SaveDraft);
        assert!(app.config.document.connections.is_empty());
        assert!(!app.draft_primary_actions_ready());
        let _ = app.update(Message::CancelOneDriveRclone);
        assert!(token.is_cancelled());
        let _ = app.update(Message::OneDriveRcloneCompleted(2, request, Ok(identity)));
        assert!(app.onedrive_editor_pending.is_none());
        assert!(app.draft.onedrive_identity.is_none());
        assert!(app.config.document.connections.is_empty());
    }

    #[test]
    fn onedrive_editor_rclone_remote_reference_is_protected_from_removal() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        let mut draft = onedrive_editor_draft(DriveType::Personal);
        draft.onedrive_identity = Some(OneDriveIdentity {
            drive_id: "primary-drive".into(),
            drive_type: DriveType::Personal,
        });
        let mut app = AppModel::default();
        app.config
            .document
            .connections
            .push(connection_from_draft(&draft).unwrap());
        assert!(app.rclone_remote_is_referenced("example_remote"));
        app.config.document.connections[0].mode =
            ConnectionMode::OnlineMount(OnlineMountConfig::default());
        assert!(!app.rclone_remote_is_referenced("example_remote"));
    }

    struct EditorGraph(cosmic_ext_applet_mounter::onedrive::DriveType);
    impl cosmic_ext_applet_mounter::onedrive::Graph for EditorGraph {
        fn get<'a>(
            &'a self,
            target: cosmic_ext_applet_mounter::onedrive::GraphTarget<'a>,
            _token: &'a str,
            _cancel: CancellationToken,
        ) -> cosmic_ext_applet_mounter::onedrive::GraphFuture<'a> {
            Box::pin(async move {
                use cosmic_ext_applet_mounter::onedrive::GraphTarget;
                Ok(match target {
                    GraphTarget::Profile => {
                        serde_json::json!({"id":"synthetic-user","mail":"person@example.invalid"})
                    }
                    GraphTarget::PrimaryDrive | GraphTarget::ExpectedBusinessDrive(_) => {
                        serde_json::json!({"id":"primary-drive","driveType":self.0.as_str(),"owner":{"user":{"id":"synthetic-owner"}}})
                    }
                    GraphTarget::Folder { .. } => {
                        serde_json::json!({"id":"folder-id","folder":{},"parentReference":{"driveId":"primary-drive"}})
                    }
                })
            })
        }
    }

    #[tokio::test]
    async fn onedrive_editor_setup_adapter_create_select_and_cancel_use_production_backend() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        for kind in [DriveType::Personal, DriveType::Business] {
            for create in [false, true] {
                let mut draft = onedrive_editor_draft(kind);
                let request = OneDriveEditorRequest::from_draft(&draft, &[], create).unwrap();
                let config = serde_json::json!({"example_remote":{"type":"onedrive","drive_type":kind.as_str(),
                    "drive_id":"primary-drive","token":{"access_token":"synthetic-token"}}}).to_string();
                let runner = FakeCommandRunner::default();
                if create {
                    runner.push(Ok(command_output("{}")));
                    runner.push(Ok(command_output(r#"{"State":""}"#)));
                }
                runner.push(Ok(command_output("{}")));
                runner.push(Ok(command_output(&config)));
                runner.push(Ok(command_output(&config)));
                runner.push(Ok(command_output(r#"{"IsDir":true}"#)));
                let identity = run_onedrive_editor_setup(
                    &runner,
                    &EditorGraph(kind),
                    &request,
                    CancellationToken::new(),
                )
                .await
                .unwrap();
                accept_onedrive_editor_result(&mut draft, &request, Ok(identity)).unwrap();
                assert!(draft.onedrive_identity.is_some());
                let commands = runner
                    .requests()
                    .iter()
                    .map(|r| r.sanitized_command())
                    .collect::<Vec<_>>();
                assert_eq!(commands.iter().any(|c| c.contains("config create")), create);
                assert!(commands.iter().all(|c| !c.contains("bisync")
                    && !c.contains("purge")
                    && !c.contains("config delete")));
                if !create {
                    assert!(
                        commands.iter().all(|c| c.starts_with("rclone lsjson ")
                            || c.starts_with("rclone config dump"))
                    );
                }
                let runner = FakeCommandRunner::default();
                let cancel = CancellationToken::new();
                cancel.cancel();
                assert!(
                    run_onedrive_editor_setup(&runner, &EditorGraph(kind), &request, cancel)
                        .await
                        .is_err()
                );
                assert!(runner.requests().is_empty());
            }
        }
    }

    #[test]
    fn onedrive_focused_late_test_and_save_validation_cannot_accept_edited_draft() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        for kind in [DriveType::Personal, DriveType::Business] {
            let mut draft = onedrive_editor_draft(kind);
            draft.onedrive_identity = Some(OneDriveIdentity {
                drive_id: "primary-drive".into(),
                drive_type: kind,
            });
            let verified = connection_from_draft(&draft).unwrap();
            for save in [false, true] {
                let mut app = AppModel {
                    draft: draft.clone(),
                    ..AppModel::default()
                };
                let _ = app.update(Message::DraftRemote("another_remote".into()));
                assert!(app.draft.onedrive_identity.is_none());
                let _ = app.update(if save {
                    Message::SaveDraftValidated(verified.clone(), Ok("stale success".into()))
                } else {
                    Message::DraftTested(verified.clone(), Ok("stale success".into()))
                });
                assert!(app.validated_draft.is_none());
                assert!(app.draft.onedrive_identity.is_none());
                assert!(app.config.document.connections.is_empty());
                assert_eq!(app.draft.remote_reference, "another_remote");
                assert!(!app.draft_primary_actions_ready());
            }
        }
    }

    #[test]
    fn onedrive_focused_setup_result_rejects_saved_identity_changed_during_authentication() {
        use cosmic_ext_applet_mounter::onedrive::DriveType;
        let mut draft = onedrive_editor_draft(DriveType::Business);
        let original_identity = OneDriveIdentity {
            drive_id: "original-drive".into(),
            drive_type: DriveType::Business,
        };
        draft.onedrive_identity = Some(original_identity.clone());
        let saved = connection_from_draft(&draft).unwrap();
        let mut app = AppModel {
            draft,
            ..AppModel::default()
        };
        app.config.document.connections.push(saved);
        let request =
            OneDriveEditorRequest::from_draft(&app.draft, &app.config.document.connections, true)
                .unwrap();
        app.draft.onedrive_identity = None;
        app.onedrive_editor_generation = 1;
        app.onedrive_editor_pending = Some(OneDriveEditorPending {
            cancellation: CancellationToken::new(),
        });
        let changed_identity = OneDriveIdentity {
            drive_id: "changed-drive".into(),
            drive_type: DriveType::Business,
        };
        app.config.document.connections[0].onedrive_identity = Some(changed_identity.clone());
        let _ = app.update(Message::OneDriveRcloneCompleted(
            1,
            request,
            Ok(original_identity),
        ));
        assert!(app.onedrive_editor_pending.is_none());
        assert!(app.draft.onedrive_identity.is_none());
        assert!(app.validated_draft.is_none());
        assert_eq!(
            app.config.document.connections[0].onedrive_identity,
            Some(changed_identity)
        );
    }
}
