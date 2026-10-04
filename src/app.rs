// SPDX-License-Identifier: MIT

mod sftp;
use sftp::{SftpAuth, SftpDraft, SftpField};

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
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
use cosmic_ext_applet_mounter::import::{
    ImportPreview, ImportReplacementPlan, default_scan_directory, parse_rclone_backends,
    preview_import, replacement_plan, scan_legacy_units,
};
use cosmic_ext_applet_mounter::mirror_script;
use cosmic_ext_applet_mounter::model::{
    AccessMode, Connection, ConnectionId, ConnectionMode, ConnectionStatus, OfflineMirrorConfig,
    OfflineMirrorStatus, OnlineMountConfig, OnlineMountStatus, Operation, PreloadPolicy,
    PreloadSettings, Provider, SmbPreloadOverride, TuningProfile, VpnKind, VpnProfile,
    VpnProfileId,
};
use cosmic_ext_applet_mounter::mount_guard;
use cosmic_ext_applet_mounter::mounts::{
    MountEntry, MountTable, MountTableError, ProcMountTable, SyncRuntimeState,
};
use cosmic_ext_applet_mounter::pending_count;
use cosmic_ext_applet_mounter::process::{
    CommandError, CommandExecutionMode, CommandOutput, CommandRequest, CommandRunner, Executable,
    RuntimeCommandRunner, redact_text,
};
use cosmic_ext_applet_mounter::providers::{
    CommandRcloneProvider, OnedriverAuthState, ProviderError, clean_unmount_request,
    lazy_unmount_request, onedriver_auth_state_for_plan, onedriver_mount_plan, rclone_mount_plan,
};
use cosmic_ext_applet_mounter::rclone_refresh::{self, RcloneRefreshJob, RcloneRefreshOutcome};
use cosmic_ext_applet_mounter::services::{
    ActiveState, CommandSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction,
    SystemdManager, UnitController, UnitDocument, UnitKind, UnitName, UnitStatus,
    start_managed_online_service,
};
use cosmic_ext_applet_mounter::sync::{
    OneDriveIsolationReport, SyncDecision, SyncDecisionRejection, SyncReadiness, SyncRequest,
    SyncTrigger, one_drive_auth_files_request, one_drive_auth_request,
    one_drive_initial_sync_request, one_drive_mirror_plan, one_drive_preview_request,
    one_drive_sync_request, parse_preview, rclone_bisync_filter_file,
    rclone_bisync_initial_preview_request, rclone_bisync_initial_sync_request, rclone_bisync_plan,
    rclone_bisync_preview_request, rclone_bisync_sync_request, sync_now_request,
};
use cosmic_ext_applet_mounter::teams;
use cosmic_ext_applet_mounter::teams_oauth::{self, TeamsOAuthOutcome, TeamsOAuthSetup};
use cosmic_ext_applet_mounter::vpn::{
    CiscoTunnelState, CiscoVpn, CommandCiscoVpn, CommandNetworkManagerVpn, CommandReadinessProbe,
    NetworkManagerVpn, VpnShutdownDecision, readiness_report, shutdown_decision,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const GENERAL_SETTINGS_TITLE: &str = "Cloud Mounter Settings";
const CONNECTION_SETTINGS_TITLE: &str = "Cloud Mounter Connection Settings";
const REORDER_CONNECTIONS_TITLE: &str = "Reorder Connections";
const APP_DISPLAY_NAME: &str = "Cloud Mounter";
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
const CLEAN_UNMOUNT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SETTINGS_SECTION_TITLE_WIDTH: f32 = 150.0;
const SETTINGS_SECTION_TITLE_TOP_PADDING: u16 = 8;
const SETTINGS_RCLONE_REMOTE_BUTTONS_PER_ROW: usize = 3;
const SETTINGS_DISABLE_BUTTON_ACTIVE_LIGHTENING: f32 = 0.35;
const SETTINGS_DISABLE_BUTTON_HOVER_LIGHTENING: f32 = 0.25;
const SETTINGS_DISABLE_BUTTON_PRESSED_LIGHTENING: f32 = 0.15;
const SETTINGS_DISABLE_BUTTON_DISABLED_LIGHTENING: f32 = 0.15;

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
                format!("Enter a preload time from {MIN_PRELOAD_SECONDS} to {MAX_PRELOAD_SECONDS} seconds.")
            })?;
            if !(MIN_PRELOAD_SECONDS..=MAX_PRELOAD_SECONDS).contains(&maximum_seconds) {
                return Err(format!(
                    "Enter a preload time from {MIN_PRELOAD_SECONDS} to {MAX_PRELOAD_SECONDS} seconds."
                ));
            }
            let maximum_depth = draft.maximum_depth.as_ref().map(|value| {
                let depth = value.parse::<u8>().map_err(|_| format!("Enter a directory depth from {MIN_PRELOAD_DEPTH} to {MAX_PRELOAD_DEPTH}."))?;
                if !(MIN_PRELOAD_DEPTH..=MAX_PRELOAD_DEPTH).contains(&depth) {
                    return Err(format!("Enter a directory depth from {MIN_PRELOAD_DEPTH} to {MAX_PRELOAD_DEPTH}."));
                }
                Ok(depth)
            }).transpose()?;
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
    remote_reference: String,
    remote_subpath: String,
    teams_library_url: String,
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
struct RcloneDraftRemote {
    name: String,
    backend: String,
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

impl Default for ConnectionDraft {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            provider: Provider::GoogleDrive,
            access_mode: AccessMode::OnlineMount,
            remote_reference: String::new(),
            remote_subpath: String::new(),
            teams_library_url: String::new(),
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
            connection_preload_enabled: true,
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
    pending_repair: Option<ConnectionId>,
    pending_shared_remote_ack: Option<ConnectionId>,
    pending_rclone_remote_remove: Option<String>,
    removing_connection: Option<ConnectionId>,
    removing_rclone_remote: Option<String>,
    sftp_setup_pending: bool,
    teams_oauth_pending: bool,
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
    rclone_refresh_starting: BTreeSet<ConnectionId>,
    rclone_refresh_jobs: BTreeMap<ConnectionId, u64>,
    preload_draft: PreloadSettingsDraft,
    preload_input_dirty: bool,
    directory_preload_jobs: BTreeMap<ConnectionId, u64>,
    unmount_pending: BTreeSet<ConnectionId>,
    pending_counts: BTreeMap<ConnectionId, PendingObservation>,
    pending_poll_inflight: BTreeSet<ConnectionId>,
    pending_poll_generation: u64,
    mirror_operations_pending: BTreeSet<ConnectionId>,
}

#[derive(Debug, Clone)]
struct PendingObservation {
    files: Option<u64>,
    checked_at: Instant,
    reason: Option<String>,
    source: PendingSource,
}

#[derive(Debug, Clone, Copy)]
pub enum PendingSource {
    RcloneOnline,
    RcloneMirror,
    RcloneMirrorLive,
    OneDriveMirror,
}

#[derive(Debug, Clone)]
pub enum Message {
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
    OperationCompleted(String),
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
    DraftTeamsLibraryUrl(String),
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
    DraftRecoveryDirectory(String),
    DraftCacheLimit(String),
    DraftSyncInterval(String),
    DraftEnabled(bool),
    DraftStartAtLogin(bool),
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
    CreateTeamsRcloneRemote,
    TeamsRcloneRemoteCreated(Result<TeamsOAuthOutcome, String>),
    CreateSmbRcloneRemote,
    SmbRcloneRemoteCreated(Result<SmbRemoteApplyResult, String>),
    RequestRemoveRcloneRemote(String),
    RcloneRemoteRemoved(Result<String, String>),
    StartOnedriverSetup,
    OnedriverSetupCompleted(Result<String, String>),
    StartOneDriveMirrorSetup,
    StartOneDriveMirrorManualSetup,
    OneDriveMirrorSetupCompleted(Result<String, String>),
    OpenOneDriveMirrorAuthUrl,
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
            teams_oauth_pending: false,
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
            rclone_refresh_starting: BTreeSet::new(),
            rclone_refresh_jobs: BTreeMap::new(),
            preload_draft,
            preload_input_dirty: false,
            directory_preload_jobs: BTreeMap::new(),
            unmount_pending: BTreeSet::new(),
            pending_counts: BTreeMap::new(),
            pending_poll_inflight: BTreeSet::new(),
            pending_poll_generation: 0,
            mirror_operations_pending: BTreeSet::new(),
        };
        match flags {
            AppLaunchMode::ModifyConnection(id) => app.load_draft(id),
            AppLaunchMode::ImportLegacy => app.scan_imports(),
            AppLaunchMode::Applet
            | AppLaunchMode::AddConnection
            | AppLaunchMode::GeneralSettings
            | AppLaunchMode::ReorderConnections => {}
        }
        let window_title = match window_mode {
            WindowMode::GeneralSettings => GENERAL_SETTINGS_TITLE,
            WindowMode::ReorderConnections => REORDER_CONNECTIONS_TITLE,
            _ => CONNECTION_SETTINGS_TITLE,
        };
        let title = if standalone {
            app.set_header_title(window_title.into());
            if app.last_notice.is_none() {
                app.last_notice = Some(window_mode_notice(window_mode));
            }
            app.set_window_title(window_title.into(), main_window_id.unwrap_or(Id::RESERVED))
        } else if let Some(id) = main_window_id {
            app.set_window_title(APP_DISPLAY_NAME.into(), id)
        } else {
            Task::none()
        };

        let initial = if window_mode == WindowMode::GeneralSettings {
            app.runtime_poll_pending = true;
            runtime_request_task(RuntimeCommand::Status)
        } else {
            Task::none()
        };
        (app, Task::batch([title, initial]))
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
        if !self.standalone {
            subscriptions.push(runtime_ipc::runtime_subscription().map(Message::RuntimeEvent));
        } else if self.window_mode == WindowMode::GeneralSettings {
            subscriptions.push(runtime_ipc::settings_subscription().map(Message::RuntimeEvent));
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(2)).map(|_| Message::RuntimePoll),
            );
        }
        if !self.standalone && self.popup.is_some() {
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
        match message {
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
                    self.last_notice = Some(format!(
                        "Could not refresh some Online mount safety checks: {}",
                        failures.join("; ")
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
                    reply.complete(Err("This window is not the runtime owner".into()));
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
                        }
                        match command {
                            RuntimeCommand::SetUnmount(_) | RuntimeCommand::SetRestore(_) => {
                                self.sleep_notice = Some("Setting saved.".into());
                            }
                            RuntimeCommand::SetPreload(value) => {
                                self.preload_input_dirty = false;
                                self.preload_draft = value.into();
                                self.preload_notice = Some("Preload settings saved.".into());
                            }
                            RuntimeCommand::Refresh => {
                                self.last_notice =
                                    Some("Applet configuration and VPN status refreshed.".into());
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
            Message::SleepEvent(cosmic_ext_applet_mounter::sleep::Event::Status(status)) => {
                self.sleep_status = Some(status);
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
                        self.set_window_title(APP_DISPLAY_NAME.into(), id),
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
                return self.record_operation_request(connection_id, operation);
            }
            Message::OperationCompleted(notice) => {
                self.config = Config::load_runtime().config;
                self.pending_repair = None;
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.last_notice = Some(notice);
                self.last_notice_at = Some(Instant::now());
            }
            Message::MirrorOperationCompleted(id, notice) => {
                self.mirror_operations_pending.remove(&id);
                self.pending_counts.remove(&id);
                self.config = Config::load_runtime().config;
                self.last_notice = Some(notice);
                self.last_notice_at = Some(Instant::now());
                return Task::perform(async {}, |_| cosmic::Action::App(Message::PendingCountTick));
            }
            Message::ManagedOnlineOperationCompleted(connection, operation, result) => {
                if operation == Operation::Unmount {
                    self.unmount_pending.remove(&connection.id);
                }
                self.config = Config::load_runtime().config;
                self.pending_repair = None;
                let label = operation_label(operation);
                match result {
                    Ok(()) => {
                        self.last_notice =
                            Some(format!("{label} completed for {}.", connection.name));
                        self.last_notice_at = Some(Instant::now());
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
                                    self.last_notice = Some(format!(
                                        "Mount completed for {}, but directory refresh planning failed: {error}",
                                        connection.name
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
                            self.last_notice = Some(format!(
                                "Mount completed for {name}. Starting background directory refresh…"
                            ));
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
                            let wait_for_root = matches!(
                                connection.provider,
                                Provider::Box | Provider::Smb | Provider::Sftp
                            );
                            self.last_notice = Some(format!(
                                "Mount completed for {name}. Starting background directory preload…"
                            ));
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
                        self.last_notice =
                            Some(format!("{label} failed for {}: {error}", connection.name));
                        self.last_notice_at = Some(Instant::now());
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
                        self.last_notice = Some(format!(
                            "{name} is mounted. Background directory refresh job {} is running.",
                            job.job_id
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
                        self.last_notice = Some(format!(
                            "{name} is mounted, but its background directory refresh could not start: {error}"
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
                        self.last_notice = Some(format!(
                            "Background directory refresh failed for {name}: {error}"
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
                        self.last_notice =
                            Some(format!("Directory preload failed for {name}: {error}"));
                        self.last_notice_at = Some(Instant::now());
                    }
                }
            }
            Message::DraftProvider(provider) => {
                if matches!(self.window_mode, WindowMode::ModifyConnection(_)) {
                    self.last_notice = Some(
                        "Provider changes are disabled while modifying an existing connection."
                            .into(),
                    );
                    return Task::none();
                }
                self.pending_shared_remote_ack = None;
                self.pending_rclone_remote_remove = None;
                self.validated_draft = None;
                self.sftp_update_ack = None;
                self.draft.sftp.clear_secrets();
                self.reset_sftp_settings_tracking();
                self.draft.provider = provider;
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
                    self.last_notice = Some(
                        "Access mode changes are disabled while modifying an existing connection."
                            .into(),
                    );
                    return Task::none();
                }
                self.validated_draft = None;
                self.draft.access_mode = mode;
            }
            Message::DraftTeamsLibraryUrl(value) => {
                self.validated_draft = None;
                self.draft.teams_library_url = value;
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
                if self.draft.provider == Provider::Teams {
                    self.draft.teams_drive_id.clear();
                }
                self.sftp_update_ack = None;
                if self.draft.provider == Provider::Sftp {
                    self.load_sftp_details();
                }
            }
            Message::DraftSubpath(value) => {
                self.pending_shared_remote_ack = None;
                self.validated_draft = None;
                self.draft.remote_subpath = value;
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
                self.detect_rclone_remotes();
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
                    Ok(updated) => format!(
                        "{} SFTP remote `{name}`. Credential fields cleared. Restart connections using this remote, then run Test Connection.",
                        if updated { "Updated" } else { "Created" }
                    ),
                    Err(error) => format!(
                        "Could not apply SFTP remote `{name}`: {error}. Credential fields cleared.{}",
                        if attempted_secret.is_some() {
                            " Re-enter the credential before retrying."
                        } else {
                            ""
                        }
                    ),
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
            Message::LocalFolderPicked(result) => match result {
                Ok(Some(path)) => {
                    self.pending_shared_remote_ack = None;
                    self.pending_rclone_remote_remove = None;
                    self.validated_draft = None;
                    self.draft.local_path = path.clone();
                    self.last_notice = Some(format!(
                        "{} selected: {path}",
                        local_target_label(self.draft.access_mode)
                    ));
                }
                Ok(None) => {
                    self.last_notice = Some("Folder selection cancelled.".into());
                }
                Err(error) => {
                    self.last_notice = Some(format!("Could not select folder: {error}"));
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
            Message::DraftStartAtLogin(value) => {
                self.validated_draft = None;
                self.draft.start_at_login = value;
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
                self.detect_rclone_remotes();
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
                        self.detect_rclone_remotes();
                        let action = if result.updated { "Updated" } else { "Created" };
                        let remount = if result.updated {
                            " Remount active connections that use this remote."
                        } else {
                            ""
                        };
                        self.last_notice = Some(format!(
                            "{action} Google Drive rclone remote `{}` with browser OAuth. The credential fields were cleared.{remount} Run Test Connection to verify access.",
                            result.remote_name
                        ));
                    }
                    Err(error) => {
                        self.last_notice = Some(format!(
                            "Could not create or update Google Drive rclone remote: {error}. The credential fields were cleared."
                        ));
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
                    self.detect_rclone_remotes();
                    self.last_notice = Some(format!(
                        "Created Box rclone remote `{remote_name}`. It is selected; run Test Connection to verify access."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!("Could not create Box rclone remote: {error}"));
                }
            },
            Message::CreateTeamsRcloneRemote => {
                return self.create_teams_rclone_remote();
            }
            Message::TeamsRcloneRemoteCreated(result) => {
                self.teams_oauth_pending = false;
                match result {
                    Ok(outcome) => {
                        self.detect_rclone_remotes();
                        if self.draft.provider == Provider::Teams
                            && self.draft.remote_reference.trim() == outcome.remote_name
                            && teams::same_library_url(
                                &self.draft.teams_library_url,
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
                    self.detect_rclone_remotes();
                    let password_status = if result.password_updated {
                        "The SMB password was updated in rclone and cleared from the applet form."
                    } else {
                        "No password was entered; the existing rclone password was left unchanged."
                    };
                    self.last_notice = Some(format!(
                        "Created or updated SMB rclone remote `{}` with host/user/domain metadata. {password_status} Run Test Connection to verify access.",
                        result.remote_name
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!("Could not create SMB rclone remote: {error}"));
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
                    self.detect_rclone_remotes();
                    self.last_notice = Some(format!(
                        "Removed unused rclone remote `{remote_name}` from rclone configuration."
                    ));
                }
                Err(error) => {
                    self.pending_rclone_remote_remove = None;
                    self.removing_rclone_remote = None;
                    self.last_notice = Some(format!("Could not remove rclone remote: {error}"));
                }
            },
            Message::StartOnedriverSetup => {
                return self.start_onedriver_setup();
            }
            Message::OnedriverSetupCompleted(result) => match result {
                Ok(summary) => {
                    self.last_notice = Some(format!(
                        "OneDrive Online Mount setup completed. {summary} Run Test Connection, then Save Connection."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!(
                        "Could not complete OneDrive Online Mount setup: {error}"
                    ));
                }
            },
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
                    self.last_notice = Some(format!(
                        "OneDrive Offline Mirror authorization and validation completed. {summary} Save Connection can reuse this validation. After saving, run Preview, then Sync Now to start the initial mirror."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!(
                        "Could not complete OneDrive Offline Mirror setup: {error}. If browser redirect capture failed or authentication timed out, use Manual Auth Handoff."
                    ));
                }
            },
            Message::OpenOneDriveMirrorAuthUrl => {
                self.open_onedrive_mirror_auth_url();
            }
            Message::SubmitOneDriveMirrorAuthResponse => {
                self.submit_onedrive_mirror_auth_response();
            }
            Message::TestDraft if self.sftp_setup_pending => {
                self.last_notice =
                    Some("Wait for SFTP remote setup to finish before testing.".into());
            }
            Message::TestDraft if self.teams_oauth_pending => {
                self.last_notice = Some(fl!("teams-oauth-wait-test"));
            }
            Message::SaveDraft if self.sftp_setup_pending => {
                self.last_notice =
                    Some("Wait for SFTP remote setup to finish before saving.".into());
            }
            Message::SaveDraft if self.teams_oauth_pending => {
                self.last_notice = Some(fl!("teams-oauth-wait-save"));
            }
            Message::TestDraft => {
                return self.test_draft_plan();
            }
            Message::DraftTested(connection, result) => match result {
                Ok(summary) => {
                    if connection.provider == Provider::Teams {
                        if !teams_verification_matches_draft(&self.draft, &connection) {
                            self.last_notice = Some("SharePoint details changed during testing. Test the current draft again.".into());
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
                    self.last_notice = Some(format!(
                        "Draft test passed for {}. {summary}",
                        connection.name
                    ));
                }
                Err(error) => {
                    self.validated_draft = None;
                    self.last_notice = Some(format!(
                        "Draft test failed for {}: {error}",
                        connection.name
                    ));
                }
            },
            Message::SaveDraft => {
                return self.save_draft();
            }
            Message::SaveDraftValidated(connection, validation) => {
                if connection.provider == Provider::Teams
                    && !teams_verification_matches_draft(&self.draft, &connection)
                {
                    self.last_notice = Some("SharePoint details changed during verification. Save again to check the current draft.".into());
                    return Task::none();
                }
                return match validation {
                    Ok(summary) => self.save_validated_draft(connection, Some(summary)),
                    Err(error) => {
                        self.last_notice =
                            Some(format!("Save blocked for {}: {error}", connection.name));
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
                    && now.duration_since(started_at) >= POPUP_NOTICE_TIMEOUT
                {
                    self.last_notice = None;
                    self.last_notice_at = None;
                }
            }
            Message::PendingCountTick => {
                if self.popup.is_none() || self.standalone || self.network_ready == Some(false) {
                    return Task::none();
                }
                let mounts = HostVisibleMountTable.entries().unwrap_or_default();
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
                            && (is_onedrive_offline_mirror(connection)
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
                        && runtime_offline_mirror_state(&connection) == SyncRuntimeState::Running;
                    if !live_rclone
                        && self
                            .pending_counts
                            .get(&connection.id)
                            .is_some_and(|observation| {
                                matches!(observation.source, PendingSource::RcloneMirrorLive)
                            })
                    {
                        self.pending_counts.remove(&connection.id);
                    }
                    if is_rclone_offline_mirror(&connection)
                        && !live_rclone
                        && runtime_unit_status(connection.id, UnitKind::Timer).is_some_and(
                            |status| {
                                matches!(
                                    status.active,
                                    ActiveState::Active | ActiveState::Activating
                                )
                            },
                        )
                    {
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
                    let source = if live_rclone {
                        PendingSource::RcloneMirrorLive
                    } else if is_onedrive_offline_mirror(&connection) {
                        PendingSource::OneDriveMirror
                    } else {
                        PendingSource::RcloneMirror
                    };
                    tasks.push(Task::perform(
                        async move {
                            let result = if live_rclone {
                                rclone_mirror_live_pending_count(&connection).await
                            } else if is_onedrive_offline_mirror(&connection) {
                                onedrive_mirror_pending_count(&connection).await
                            } else {
                                rclone_mirror_pending_count(&connection).await
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
                        && !HostVisibleMountTable
                            .entries()
                            .unwrap_or_default()
                            .iter()
                            .any(|entry| entry.target == connection.local_path))
                    || (matches!(source, PendingSource::RcloneMirrorLive)
                        && runtime_offline_mirror_state(&connection) != SyncRuntimeState::Running)
                {
                    return Task::none();
                }
                self.pending_counts.insert(
                    connection.id,
                    PendingObservation {
                        files: result.as_ref().ok().copied(),
                        checked_at: Instant::now(),
                        reason: result.err(),
                        source,
                    },
                );
            }
            Message::VpnStatusChecked(ready) => {
                self.vpn_ready = ready;
            }
            Message::NetworkStatusChecked(ready) => {
                self.network_ready = Some(ready);
                if !ready {
                    self.pending_counts.clear();
                    self.pending_poll_generation = self.pending_poll_generation.wrapping_add(1);
                }
            }
        }

        Task::none()
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
    fn runtime_status(&self) -> runtime_ipc::Status {
        runtime_ipc::Status {
            unmount_before_sleep: self.config.document.unmount_before_sleep,
            restore_after_wake: self.config.document.restore_after_wake,
            preload: self.config.document.preload,
            sleep_status: self.sleep_status.clone(),
        }
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
            self.sleep_status = value.then(|| "Starting sleep listener…".into());
        }
        Ok(self.runtime_status())
    }

    fn view_general_settings(&self) -> Element<'_, Message> {
        let enabled = self.runtime_available && !self.runtime_pending;
        let actions = widget::Row::new().spacing(8)
            .push(field_with_help(
                widget::button::suggested(fl!("add-connection")).on_press(Message::OpenAddConnection),
                "Open the Add Connection workflow to create a new storage connection.",
            ))
            .push(field_with_help(
                widget::button::standard(fl!("refresh"))
                    .on_press_maybe((!self.runtime_pending).then_some(Message::Refresh)),
                "Reload saved connections and refresh VPN status in the running applet. Existing operations continue.",
            ))
            .push(field_with_help(
                widget::button::standard(fl!("reorder-connections"))
                    .on_press(Message::OpenReorderConnections),
                "Change the display order of connections in the main applet popup.",
            ));
        let mut action_section = widget::Column::new()
            .spacing(8)
            .push(widget::text::title4(fl!("settings-connections")))
            .push(actions);
        if self.runtime_pending && !self.sleep_settings_pending {
            action_section =
                action_section.push(widget::text::body("Waiting for the running applet…"));
        }
        if let Some(error) = &self.runtime_error {
            action_section = action_section.push(widget::text::body(error.clone()));
        } else if !self.runtime_available {
            action_section =
                action_section.push(widget::text::body("Connecting to the running applet…"));
        }
        if let Some(notice) = &self.last_notice {
            action_section = action_section.push(widget::text::body(notice.clone()));
        }
        let mut sleep_section = widget::Column::new().spacing(8)
            .push(widget::text::title4("Sleep and wake"))
            .push(field_with_help(
                widget::toggler(self.config.document.unmount_before_sleep)
                    .label("Unmount when sleep".to_string())
                    .spacing(8)
                    .on_toggle_maybe(enabled.then_some(Message::UnmountBeforeSleep)),
                "Cleanly unmount applet-managed Online connections before system sleep. Offline mirrors and synchronization remain untouched. Busy mounts may outlast the system’s sleep delay; caches are preserved.",
            ))
            .push(field_with_help(
                widget::toggler(self.config.document.restore_after_wake)
                    .label("Restore after wake up".to_string())
                    .spacing(8)
                    .on_toggle_maybe((enabled && self.config.document.unmount_before_sleep).then_some(Message::RestoreAfterWake)),
                "Restore only previously active, still-enabled Online connections after network and VPN readiness checks pass. Enable Unmount when sleep first; the saved restore preference is retained while disabled.",
            ))
            .push(widget::text::body("Applies only to Online connections."));
        if self.sleep_settings_pending {
            sleep_section = sleep_section.push(widget::text::body("Saving sleep setting…"));
        }
        if let Some(notice) = &self.sleep_notice {
            sleep_section = sleep_section.push(widget::text::body(notice.clone()));
        }
        if let Some(status) = &self.sleep_status {
            sleep_section = sleep_section.push(widget::text::body(status.clone()));
        }
        let preload_save = widget::button::standard("Save preload settings").on_press_maybe(
            (enabled && self.preload_input_dirty).then_some(Message::SavePreloadSettings),
        );
        let mut preload_section = widget::Column::new()
            .spacing(8)
            .push(widget::text::title4("Directory preload"))
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
            preload_section = preload_section.push(widget::text::body("Saving preload time…"));
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
                    .push(widget::container(widget::text::title4(fl!("app-title"))).width(Length::Fill))
                    .push(field_with_help_at(
                        widget::button::icon(widget::icon::from_svg_bytes(include_bytes!(
                            "../resources/settings-symbolic.svg"
                        )).symbolic(true))
                        .name("Settings")
                        .on_press(Message::OpenGeneralSettings),
                        "Open Settings to add connections, refresh the applet, and configure sleep and directory preload behavior.",
                        widget::tooltip::Position::Bottom,
                    )),
            );
        let show_aggregate = state.aggregate.kind != AggregateKind::Healthy
            || self
                .popup_opened_at
                .is_none_or(|opened_at| opened_at.elapsed() < POPUP_HEALTHY_AGGREGATE_HIDE_AFTER);
        if show_aggregate {
            header = header.add(widget::text::body(aggregate_label(&state.aggregate)));
        }

        let mut rows = widget::list_column().style(cosmic::style::Container::Transparent);
        if state.rows.is_empty() {
            rows = rows.add(widget::text::body(
                "No connections. Open Settings > Add Connection to get started.",
            ));
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

        if let Some(error) = &self.runtime_error {
            content = content.push(field_with_help(
                widget::text::caption("Settings communication unavailable"),
                error.clone(),
            ));
        }
        if sleep_needs_attention(self.sleep_status.as_deref()) {
            content = content.push(field_with_help(
                widget::button::text("Sleep cleanup needs attention — open Settings")
                    .on_press(Message::OpenGeneralSettings),
                self.sleep_status.clone().unwrap_or_default(),
            ));
        }
        let list_limit = 360.0
            - if self.last_notice.is_some() {
                60.0
            } else {
                0.0
            }
            - if sleep_needs_attention(self.sleep_status.as_deref()) {
                40.0
            } else {
                0.0
            }
            - if self.runtime_error.is_some() {
                40.0
            } else {
                0.0
            };
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
        let primary_enabled = primary_control_enabled(&row.status);
        let primary_help = primary.map_or_else(
            || format!("Current status: {}.", status_label(&row.status)),
            |operation| {
                let action_label = row_operation_label(&row.status, operation);
                let status = status_label(&row.status);
                let availability = decision
                    .as_ref()
                    .and_then(|decision| decision.reason.as_deref())
                    .map_or_else(
                        || format!("Toggle to {action_label}."),
                        |reason| format!("{action_label} is unavailable: {reason}."),
                    );
                format!("Current status: {status}. {availability}")
            },
        );
        let row_id = row.id;
        let primary_control: Element<'static, Message> =
            if row.provider == Provider::Teams && row.mode == AccessMode::OfflineMirror {
                widget::button::standard(fl!("sharepoint-sync-now"))
                    .on_press_maybe(
                        decision
                            .as_ref()
                            .is_some_and(|decision| decision.allowed)
                            .then_some(Message::OperationRequested(row_id, Operation::SyncNow)),
                    )
                    .into()
            } else if let Some(operation) = primary {
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
        let (pending_label, pending_help) = match self.pending_counts.get(&row.id) {
            Some(observation) if observation.checked_at.elapsed() < stale_after => {
                match observation.files {
                    Some(files) => (
                        files.to_string(),
                        if matches!(observation.source, PendingSource::RcloneMirrorLive) {
                            fl!("pending-count-mirror-live-help", count = files)
                        } else if matches!(
                            observation.source,
                            PendingSource::RcloneMirror | PendingSource::OneDriveMirror
                        ) {
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
                    format!("Open `{}` in the Modify workflow.", row.name),
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
            WindowMode::ModifyConnection(_) => "Modify Connection".into(),
            WindowMode::ImportLegacy => fl!("settings-import-title"),
            WindowMode::GeneralSettings => GENERAL_SETTINGS_TITLE.into(),
            WindowMode::ReorderConnections => REORDER_CONNECTIONS_TITLE.into(),
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
                    "Import preview",
                    widget::text::body(
                        "No compatible rclone or onedriver service previews are currently available.",
                    ),
                ));
                } else {
                    for (index, preview) in self.import_previews.iter().enumerate() {
                        content = content.add(widget::settings::item(
                            preview.original_unit_name.clone(),
                            widget::Column::new()
                                .spacing(8)
                                .push(widget::text::body(import_preview_summary(preview)))
                                .push(
                                    widget::button::suggested("Review Connection").on_press_maybe(
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
                "Online mount",
                AccessMode::OnlineMount,
                self.draft.access_mode,
                modify_locked,
            ),
            mode_choice(
                "Offline mirror",
                AccessMode::OfflineMirror,
                self.draft.access_mode,
                modify_locked,
            ),
        ]);

        content = content
            .add(if modify_locked {
                section_row("Provider", provider_choices)
            } else {
                section_row_with_help(
                    "Provider",
                    fl!("provider-choice-help"),
                    provider_choices,
                )
            })
            .add(if modify_locked {
                section_row("Access mode", mode_choices)
            } else {
                section_row_with_help(
                    "Access mode",
                    "Online mount gives on-demand network-backed access. Offline mirror keeps a local copy and synchronizes later.",
                    mode_choices,
                )
            })
            .add(section_row(
                "Connection",
                widget::Column::new()
                    .spacing(8)
                    .push(field_with_help(
                        widget::text_input::text_input(
                            "suggested: My Cloud Connection",
                            &self.draft.name,
                        )
                        .on_input(Message::DraftName),
                        "Display name shown in the applet popup.",
                    ))
                    .push(self.view_remote_account_fields()),
            ))
            .add(section_row(
                local_target_label(self.draft.access_mode),
                self.view_local_target_picker(),
            ));

        content = match self.draft.access_mode {
            AccessMode::OnlineMount => {
                let mut online_settings = widget::Column::new()
                    .spacing(8)
                    .push(field_with_help(
                        toggle_button(
                            "Start at login",
                            self.draft.start_at_login,
                            Message::DraftStartAtLogin,
                        ),
                        "Manual startup is the default. Enable this only for connections that should start when you log in.",
                    ))
                    .push(field_with_help(
                        widget::text_input::text_input(
                            "rclone VFS cache limit in GiB",
                            &self.draft.cache_limit_gib,
                        )
                        .on_input(Message::DraftCacheLimit),
                        "Maximum rclone VFS cache size. The approved default is 20 GiB.",
                    ));
                if matches!(self.draft.provider, Provider::Smb | Provider::Sftp) {
                    online_settings = online_settings.push(field_with_help(
                        toggle_button(
                            if self.draft.provider == Provider::Sftp { sftp_preload_global_label() } else { smb_preload_global_label() },
                            self.draft.connection_preload_use_global,
                            Message::DraftConnectionPreloadUseGlobal,
                        ),
                        "Use this provider's preload policy from Cloud Mounter Settings for this connection.",
                    ));
                    if !self.draft.connection_preload_use_global {
                        online_settings = online_settings
                            .push(toggle_button(
                                "Preload directories after mount",
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
                                    .push(widget::text::body("seconds"))
                                    .push(
                                        widget::text_input::text_input(
                                            "3",
                                            &self.draft.connection_preload_depth,
                                        )
                                        .on_input(Message::DraftConnectionPreloadDepth)
                                        .width(Length::Fixed(60.0)),
                                    )
                                    .push(widget::text::body("levels")),
                            );
                    }
                }
                content.add(section_row("Online mount settings", online_settings))
            }
            AccessMode::OfflineMirror => {
                let mut settings = widget::Column::new().spacing(8);
                if self.draft.provider == Provider::Teams {
                    settings =
                        settings.push(widget::text::body(fl!("sharepoint-manual-only-help")));
                } else {
                    settings = settings
                        .push(field_with_help(
                            widget::text_input::text_input(
                                "Sync interval in minutes",
                                &self.draft.sync_interval_minutes,
                            )
                            .on_input(Message::DraftSyncInterval),
                            "How often to run background synchronization while connected. Manual Sync Now remains available.",
                        ))
                        .push(field_with_help(
                            toggle_button(
                                "Allow automatic sync on metered networks",
                                self.draft.sync_on_metered,
                                Message::DraftSyncOnMetered,
                            ),
                            "Disabled by default so automatic sync pauses on metered networks. Manual Sync Now can still be used.",
                        ));
                }
                content.add(section_row(
                    "Offline mirror settings",
                    settings
                        .push(field_with_safety_help(
                            widget::text_input::text_input(
                                "leave blank for automatic recovery directory",
                                &self.draft.recovery_directory,
                            )
                            .on_input(Message::DraftRecoveryDirectory),
                            "Keep recovery data outside the mirror tree.",
                            "Optional. Leave blank to auto-generate a sibling recovery directory based on the mirror directory.",
                        ))
                        .push(widget::text::body(format!(
                            "Automatic recovery directory: {}",
                            recovery_directory_placeholder(&self.draft)
                        ))),
                ))
            }
        };

        let mut vpn_section = widget::Column::new()
            .spacing(8)
            .push(self.view_vpn_choices());

        if self.draft.vpn_profile_id.is_some() {
            vpn_section = vpn_section.push(field_with_help(
                toggle_button(
                    "Allow applet to disconnect VPN it activated",
                    self.draft.disconnect_vpn_when_unused,
                    Message::DraftDisconnectVpn,
                ),
                "The applet may disconnect only a VPN it activated, and only after no active connection still needs it.",
            ));
        }

        content
            .add(section_row("VPN dependency", vpn_section))
            .add(section_row(
                "Information",
                widget::text::body(draft_summary_text(&self.draft)),
            ))
    }

    fn view_editor_actions(&self) -> Element<'static, Message> {
        let primary_ready = self.draft_primary_actions_ready();
        let mut primary_row = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_help(
                action_button("Test Connection", primary_ready, Message::TestDraft),
                "Validate the current form values, dependencies, remote/account access, and generated plan before saving.",
            ))
            .push(field_with_safety_help(
                action_button("Save Connection", self.draft.provider == Provider::Sftp || primary_ready, Message::SaveDraft),
                "Preview and confirm before initial synchronization.",
                "Save this connection after validation. Potentially destructive sync setup still requires preview and confirmation.",
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
                    "Disable"
                } else {
                    "Enable"
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
                    modify_row = modify_row.push(self.view_teams_oauth_action());
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
                            "Run a dry-run preview for the saved Offline Mirror connection. Save pending form changes first if they should be included.",
                        ))
                        .push(field_with_safety_help(
                            widget::button::standard(operation_label(Operation::SyncNow)).on_press(
                                Message::OperationRequested(connection_id, Operation::SyncNow),
                            ),
                            "Initial synchronization requires a successful preview first.",
                            "Run synchronization now for the saved Offline Mirror connection.",
                        ));
                }
                modify_row = modify_row
                    .push(field_with_help(
                        enable_button,
                        "Disable prevents automatic use without deleting credentials, data, cache, recovery, or imported originals.",
                    ))
                    .push(field_with_help(
                        self.view_remove_connection_button(connection_id),
                        "Remove this applet-managed connection after confirmation. User data and external credentials are preserved.",
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
            widget::button::standard("Removing...")
                .on_press_maybe(None)
                .into()
        } else if self.pending_remove == Some(connection_id) {
            widget::button::destructive("Confirm Remove")
                .on_press(Message::RemoveConnection(connection_id))
                .into()
        } else {
            widget::button::destructive("Remove")
                .on_press(Message::RemoveConnection(connection_id))
                .into()
        }
    }

    fn view_onedrive_setup_actions(&self) -> Option<Element<'static, Message>> {
        if self.draft.provider != Provider::OneDrive {
            return None;
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

    fn view_local_target_picker(&self) -> Element<'_, Message> {
        widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_safety_help(
                widget::text_input::text_input("/home/user/Cloud/Example", &self.draft.local_path)
                    .on_input(Message::DraftLocalPath),
                "Do not reuse mountpoints and mirror directories.",
                "Online mounts use a mountpoint. Offline mirrors use an ordinary local directory.",
            ))
            .push(field_with_help_at(
                widget::button::standard("Browse").on_press(Message::OpenLocalFolderPicker),
                TooltipHelp::Plain(Cow::Borrowed(
                "Choose an existing local mountpoint or mirror directory with the desktop folder picker. To use a new folder, create it in the chooser if your portal supports that, or type the desired path in the text field and let the applet validate it before saving.",
                )),
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
            Provider::Teams => self.view_teams_oauth_action(),
            Provider::OneDrive => widget::Space::new().width(Length::Shrink).into(),
        }
    }

    fn view_teams_oauth_action(&self) -> Element<'static, Message> {
        field_with_help(
            widget::button::suggested(fl!("teams-oauth-connect")).on_press_maybe(
                (!self.teams_oauth_pending).then_some(Message::CreateTeamsRcloneRemote),
            ),
            fl!("teams-oauth-connect-help"),
        )
    }

    fn draft_uses_rclone(&self) -> bool {
        matches!(
            self.draft.provider,
            Provider::Teams
                | Provider::GoogleDrive
                | Provider::Box
                | Provider::Smb
                | Provider::Sftp
        )
    }

    fn draft_primary_actions_ready(&self) -> bool {
        if self.sftp_setup_pending || self.teams_oauth_pending {
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
                    widget::button::suggested(fl!("onedrive-auth-open"))
                        .on_press(Message::OpenOneDriveMirrorAuthUrl),
                    fl!("onedrive-auth-open-help"),
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
            rclone_remote_help(provider, self.window_mode == WindowMode::AddConnection),
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

        if provider == Provider::Smb {
            column = column.push(self.view_smb_remote_setup_fields());
        } else if provider == Provider::Sftp {
            column = column.push(self.view_sftp_fields());
        } else if provider == Provider::GoogleDrive {
            column = column.push(self.view_google_drive_remote_setup_fields());
        } else if provider == Provider::Teams {
            column = column.push(field_with_help(
                widget::text_input::text_input(
                    "https://tenant.sharepoint.com/sites/site/Shared%20Documents",
                    &self.draft.teams_library_url,
                )
                .on_input(Message::DraftTeamsLibraryUrl),
                "Paste the document library URL, not a Teams channel or local mountpoint. The library identity is checked with Microsoft before saving and mounting.",
            ));
        }

        column
            .push(field_with_help(
                widget::text_input::text_input(
                    if provider == Provider::Sftp {
                        sftp_remote_directory_placeholder()
                    } else if provider == Provider::Teams {
                        "optional folder within the library"
                    } else {
                        rclone_remote_directory_placeholder()
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
        let Some(expected_backend) = rclone_backend_name(provider) else {
            return Vec::new();
        };
        self.rclone_remotes
            .iter()
            .filter(|remote| remote.backend == expected_backend)
            .cloned()
            .collect()
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
                "No VPN",
                self.draft.vpn_profile_id.is_none(),
                Message::DraftVpn(None),
            ),
            "No VPN will be started or checked before this connection runs.",
        )];

        choices.extend(vpn_profile_choices(
            VpnKind::NetworkManager,
            &self.config.document.vpn_profiles,
            self.draft.vpn_profile_id,
        ));
        choices.extend(vpn_profile_choices(
            VpnKind::Cisco,
            &self.config.document.vpn_profiles,
            self.draft.vpn_profile_id,
        ));
        choices.push(field_with_help(
            widget::button::standard("Detect VPNs").on_press(Message::DetectVpns),
            "Detect existing NetworkManager VPN profiles and Cisco Secure Client availability, then import them as applet VPN references without storing credentials.",
        ));

        choice_row(choices)
    }

    fn record_operation_request(
        &mut self,
        connection_id: ConnectionId,
        operation: Operation,
    ) -> Task<cosmic::Action<Message>> {
        let state = self.view_state();
        let Some(row) = state.rows.iter().find(|row| row.id == connection_id) else {
            self.last_notice = Some("Connection is no longer available.".into());
            return Task::none();
        };
        let decision = decide_operation(row, operation);
        let label = row_operation_label(&row.status, operation);
        if !decision.allowed {
            self.last_notice = Some(format!(
                "{label} is unavailable for {}: {}",
                row.name,
                decision
                    .reason
                    .unwrap_or_else(|| "operation is not available".into())
            ));
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
            self.last_notice = Some("Connection is no longer available.".into());
            return Task::none();
        };

        if operation == Operation::Repair
            && matches!(connection.mode, ConnectionMode::OnlineMount(_))
        {
            if self.pending_repair != Some(connection_id) {
                self.pending_repair = Some(connection_id);
                self.last_notice = Some(format!(
                    "Repair selected for {}. Press Repair again to confirm lazy unmount recovery. This runs `fusermount3 -uz` on `{}` and resets the generated service; use it only after clean unmount failed and no writes are pending.",
                    connection.name,
                    connection.local_path.display()
                ));
                return Task::none();
            }
        } else {
            self.pending_repair = None;
        }

        match (&connection.mode, operation) {
            (ConnectionMode::OnlineMount(_), Operation::Mount | Operation::Unmount)
                if is_rclone_online_mount(&connection) =>
            {
                if operation == Operation::Unmount {
                    self.unmount_pending.insert(connection_id);
                    let preload_active = self.directory_preload_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_starting.contains(&connection_id);
                    self.last_notice = Some(if preload_active {
                        format!(
                            "Unmounting {}… stopping background preload first.",
                            connection.name
                        )
                    } else {
                        format!("Unmounting {}…", connection.name)
                    });
                } else {
                    self.last_notice =
                        Some(format!("{label} requested for {}...", connection.name));
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
            (ConnectionMode::OnlineMount(_), Operation::Mount | Operation::Unmount)
                if is_onedriver_online_mount(&connection) =>
            {
                if operation == Operation::Unmount {
                    self.unmount_pending.insert(connection_id);
                    let preload_active = self.directory_preload_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_jobs.contains_key(&connection_id)
                        || self.rclone_refresh_starting.contains(&connection_id);
                    self.last_notice = Some(if preload_active {
                        format!(
                            "Unmounting {}… stopping background preload first.",
                            connection.name
                        )
                    } else {
                        format!("Unmounting {}…", connection.name)
                    });
                } else {
                    self.last_notice =
                        Some(format!("{label} requested for {}...", connection.name));
                }
                self.last_notice_at = Some(Instant::now());
                Task::perform(
                    async move {
                        let result = run_managed_onedriver_online_mount_operation_result(
                            &connection,
                            operation,
                        )
                        .await;
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
                self.last_notice = Some(format!(
                    "{label} requested for {}. Attempting confirmed lazy-unmount recovery...",
                    connection.name
                ));
                Task::perform(
                    async move { run_online_mount_repair_operation(connection).await },
                    |notice| cosmic::Action::App(Message::OperationCompleted(notice)),
                )
            }
            (
                ConnectionMode::OfflineMirror(_),
                Operation::PreviewInitialSync
                | Operation::SyncNow
                | Operation::PauseSync
                | Operation::ResumeSync,
            ) if is_rclone_offline_mirror(&connection) => {
                self.last_notice = Some(format!("{label} requested for {}...", connection.name));
                self.mirror_operations_pending.insert(connection_id);
                Task::perform(
                    async move { run_managed_offline_mirror_operation(connection, operation).await },
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
            ) if is_onedrive_offline_mirror(&connection) => {
                self.last_notice = Some(format!("{label} requested for {}...", connection.name));
                self.mirror_operations_pending.insert(connection_id);
                Task::perform(
                    async move {
                        run_managed_onedrive_offline_mirror_operation(connection, operation).await
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
                self.last_notice = Some(format!(
                    "{label} requested for {}. This operation is not wired to the managed runtime backend yet.",
                    row.name
                ));
                Task::none()
            }
        }
    }

    fn start_refresh_for_active_google_mounts(&mut self) -> Task<cosmic::Action<Message>> {
        let mounts = HostVisibleMountTable.entries().unwrap_or_default();
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
                    self.last_notice = Some(format!(
                        "{name} is mounted. Background directory preload is running."
                    ));
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
                self.last_notice = Some(format!(
                    "{name} is mounted, but its directory preload could not start: {error}"
                ));
                self.last_notice_at = Some(Instant::now());
                Task::none()
            }
        }
    }

    fn start_preload_for_active_directory_mounts(&mut self) -> Task<cosmic::Action<Message>> {
        let mounts = HostVisibleMountTable.entries().unwrap_or_default();
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
            let wait_for_root = matches!(
                connection.provider,
                Provider::Box | Provider::Smb | Provider::Sftp
            );
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

    fn load_draft(&mut self, connection_id: ConnectionId) {
        let Some(connection) = self
            .config
            .document
            .connections
            .iter()
            .find(|connection| connection.id == connection_id)
        else {
            self.last_notice = Some("Connection is no longer available.".into());
            return;
        };
        self.pending_shared_remote_ack = None;
        self.draft = draft_from_connection(connection);
        if connection.provider == Provider::Sftp {
            self.load_sftp_details();
            return;
        }
        if connection.provider == Provider::Smb {
            match load_smb_remote_details(&connection.remote_reference) {
                Ok(Some(details)) => {
                    self.draft.smb_host = details.host;
                    self.draft.smb_user = details.user;
                    self.draft.smb_domain = details.domain;
                    self.last_notice = Some(format!(
                        "Modify {}. SMB host/user/domain were loaded from rclone; password remains blank.",
                        connection.name
                    ));
                    return;
                }
                Ok(None) => {
                    self.last_notice = Some(format!(
                        "Modify {}. The rclone SMB remote `{}` was not found; host/user/domain are blank.",
                        connection.name, connection.remote_reference
                    ));
                    return;
                }
                Err(error) => {
                    self.last_notice = Some(format!(
                        "Modify {}. Could not load SMB details from rclone: {error}",
                        connection.name
                    ));
                    return;
                }
            }
        }
        self.last_notice = Some(format!("Modify {}.", connection.name));
    }

    fn save_draft(&mut self) -> Task<cosmic::Action<Message>> {
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
            self.last_notice = Some(format!("Plan validation failed: {error}"));
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
            self.last_notice = Some(format!(
                "{warning} Press Save Connection again to confirm this shared account/remote choice."
            ));
            return Task::none();
        }
        if connection.provider == Provider::OneDrive {
            if let Some(validated) = &self.validated_draft
                && validated.connection == connection
            {
                self.last_notice = Some(format!(
                    "{} already passed validation in this window; saving without repeating the dry-run.",
                    connection.name
                ));
                return self.save_validated_draft(connection, Some(validated.summary.clone()));
            }
            self.last_notice = Some(format!(
                "Validating {} before saving. OneDrive Offline Mirror validation runs a bounded dry-run preview and can take several minutes; keep this window open until the message changes...",
                connection.name
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
            self.last_notice = Some(format!(
                "Verifying the SharePoint library for {} before saving...",
                connection.name
            ));
            return Task::perform(
                async move {
                    let mut connection = connection;
                    let validation = match teams::verify_remote(
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
                                "account unavailable from Microsoft Graph".into()
                            });
                            let identity = verified.identity.clone();
                            let summary = format!(
                                "Verified account {account}, site {}, library {}, drive {}.",
                                identity.site_url, identity.library_url, identity.drive_id
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
        let should_install_onedrive_offline_mirror = is_onedrive_offline_mirror(&connection);
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
                self.last_notice = Some(format!(
                    "{name} saved to applet configuration. Installing managed mount unit..."
                ));
                Task::perform(
                    async move { install_rclone_online_mount_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_rclone_offline_mirror => {
                self.last_notice = Some(format!(
                    "{name} saved to applet configuration. Installing managed mirror units..."
                ));
                Task::perform(
                    async move { install_rclone_offline_mirror_units(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_onedriver_online_mount => {
                self.last_notice = Some(format!(
                    "{} saved to applet configuration. Installing managed onedriver mount unit...",
                    save_notice_name(&name, validation_summary.as_deref())
                ));
                Task::perform(
                    async move { install_onedriver_online_mount_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) | Ok(false) if should_install_onedrive_offline_mirror => {
                self.last_notice = Some(format!(
                    "{} saved to applet configuration. Installing managed OneDrive mirror unit...",
                    save_notice_name(&name, validation_summary.as_deref())
                ));
                Task::perform(
                    async move { install_onedrive_offline_mirror_unit(saved_connection).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Ok(true) => {
                self.last_notice = Some(format!("{name} saved to applet configuration."));
                notify_runtime_task()
            }
            Ok(false) => {
                self.last_notice = Some(format!("{name} was unchanged."));
                notify_runtime_task()
            }
            Err(error) => {
                self.last_notice = Some(format!("Failed to save {name}: {error}"));
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
                return Err(
                    "Provider changes are disabled for existing connections. Create or duplicate a connection to use a different provider."
                        .into(),
                );
            }
            if access_mode_for_connection(saved) != access_mode_for_connection(connection) {
                return Err(
                    "Access mode changes are disabled for existing connections. Create or duplicate a connection to switch between Online mount and Offline mirror."
                        .into(),
                );
            }
        }

        if let Some(other) = self.config.document.connections.iter().find(|saved| {
            saved.id != connection.id
                && saved
                    .name
                    .trim()
                    .eq_ignore_ascii_case(connection.name.trim())
        }) {
            return Err(format!(
                "Connection name `{}` is already used by `{}`. Choose a unique name.",
                connection.name, other.name
            ));
        }

        if let Some(other) = self.config.document.connections.iter().find(|saved| {
            saved.id != connection.id
                && (paths_overlap(&saved.local_path, &connection.local_path)
                    || paths_overlap(&connection.local_path, &saved.local_path))
        }) {
            return Err(format!(
                "Local target `{}` overlaps `{}` used by `{}`. Choose a separate mountpoint or mirror directory.",
                connection.local_path.display(),
                other.local_path.display(),
                other.name
            ));
        }

        Ok(())
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
                format!(
                    "`{}` uses the same {} account/remote `{}` as `{}`.",
                    connection.name,
                    provider_label(connection.provider),
                    connection.remote_reference,
                    other.name
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
                    existing.name = detected.name.clone();
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
                let notice = format!(
                    "VPN detection complete: {imported} imported, {updated} already known/updated.{warning}"
                );
                self.last_notice = Some(notice);
            }
            Err(error) => {
                let notice = format!("VPN detection could not be saved: {error}");
                self.last_notice = Some(notice);
            }
        }
    }

    fn detect_rclone_remotes(&mut self) {
        match run_sync_host_command("rclone", &["config", "dump"]) {
            Ok(output) if output.status.success() => {
                match parse_rclone_remotes_for_app(&String::from_utf8_lossy(&output.stdout)) {
                    Ok(remotes) => {
                        let total = remotes.len();
                        let matching = self
                            .matching_rclone_remotes_in(&remotes, self.draft.provider)
                            .len();
                        self.rclone_remotes = remotes;
                        self.last_notice = Some(format!(
                            "Detected {total} rclone remote(s); {matching} match {}.",
                            provider_label(self.draft.provider)
                        ));
                    }
                    Err(error) => {
                        self.last_notice = Some(format!("Could not parse rclone remotes: {error}"));
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let message = stderr.trim();
                self.last_notice = Some(format!(
                    "Could not detect rclone remotes: {}",
                    if message.is_empty() {
                        "rclone config dump failed"
                    } else {
                        message
                    }
                ));
            }
            Err(error) => {
                self.last_notice = Some(format!("Could not run rclone config dump: {error}"));
            }
        }
    }

    fn rclone_remote_is_referenced(&self, remote_name: &str) -> bool {
        self.config.document.connections.iter().any(|connection| {
            connection.provider != Provider::OneDrive
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
            self.last_notice = Some(format!(
                "Cannot remove rclone remote `{remote_name}` because a saved connection still uses it."
            ));
            return Task::none();
        }
        if self.pending_rclone_remote_remove.as_deref() != Some(remote_name.as_str()) {
            self.pending_rclone_remote_remove = Some(remote_name.clone());
            self.last_notice = Some(format!(
                "Click Confirm remove for `{remote_name}` to delete it from rclone configuration. This affects rclone, not only this applet."
            ));
            return Task::none();
        }

        self.last_notice = Some(format!("Removing rclone remote `{remote_name}`..."));
        self.removing_rclone_remote = Some(remote_name.clone());
        Task::perform(
            async move { remove_rclone_remote_result(remote_name).await },
            |result| cosmic::Action::App(Message::RcloneRemoteRemoved(result)),
        )
    }

    fn create_smb_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Smb {
            self.last_notice =
                Some("SMB remote creation is only available for SMB connections.".into());
            return Task::none();
        }
        let setup = match SmbRemoteSetup::from_draft(&self.draft) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(format!("Creating SMB rclone remote `{}`...", setup.name));
        Task::perform(
            async move { create_smb_rclone_remote_result(setup).await },
            |result| cosmic::Action::App(Message::SmbRcloneRemoteCreated(result)),
        )
    }

    fn create_box_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Box {
            self.last_notice =
                Some("Box remote creation is only available for Box connections.".into());
            return Task::none();
        }
        let setup = match BoxRemoteSetup::from_draft(&self.draft) {
            Ok(setup) => setup,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(format!(
            "Starting Box OAuth for rclone remote `{}`. Complete the browser authorization window; this can take a minute.",
            setup.name
        ));
        Task::perform(
            async move { create_box_rclone_remote_result(setup).await },
            |result| cosmic::Action::App(Message::BoxRcloneRemoteCreated(result)),
        )
    }

    fn create_teams_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Teams {
            self.last_notice = Some(fl!("teams-oauth-teams-only"));
            return Task::none();
        }
        if self.teams_oauth_pending {
            return Task::none();
        }
        let setup =
            match TeamsOAuthSetup::new(&self.draft.remote_reference, &self.draft.teams_library_url)
            {
                Ok(setup) => setup,
                Err(error) => {
                    self.last_notice = Some(error);
                    return Task::none();
                }
            };
        self.teams_oauth_pending = true;
        self.last_notice = Some(fl!("teams-oauth-opening", name = setup.remote_name.clone()));
        Task::perform(
            async move { teams_oauth::create_teams_remote(&app_command_runner(), setup).await },
            |result| cosmic::Action::App(Message::TeamsRcloneRemoteCreated(result)),
        )
    }

    fn apply_google_drive_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::GoogleDrive {
            self.last_notice = Some(
                "Google Drive remote creation is only available for Google Drive connections."
                    .into(),
            );
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
        self.last_notice = Some(format!(
            "Starting Google Drive OAuth setup for rclone remote `{}`. Complete the browser authorization window; this can take a minute.",
            setup.name
        ));
        Task::perform(
            async move { apply_google_drive_rclone_remote_result(setup, action).await },
            |result| cosmic::Action::App(Message::GoogleDriveRcloneRemoteApplied(result)),
        )
    }

    fn start_onedriver_setup(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::OneDrive
            || self.draft.access_mode != AccessMode::OnlineMount
        {
            self.last_notice = Some(
                "onedriver setup is only available for OneDrive Online Mount connections.".into(),
            );
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
        self.last_notice = Some(format!(
            "Starting onedriver setup for `{}`. Complete the browser authorization window; this can take a minute.",
            connection.name
        ));
        Task::perform(
            async move { run_onedriver_online_setup_result(connection).await },
            |result| cosmic::Action::App(Message::OnedriverSetupCompleted(result)),
        )
    }

    fn open_local_folder_picker(&mut self) -> Task<cosmic::Action<Message>> {
        let mode = self.draft.access_mode;
        self.last_notice = Some(format!("Opening {} picker...", local_target_label(mode)));
        Task::perform(async move { pick_local_folder(mode).await }, |result| {
            cosmic::Action::App(Message::LocalFolderPicked(result))
        })
    }

    fn start_onedrive_mirror_setup(&mut self) -> Task<cosmic::Action<Message>> {
        self.start_onedrive_mirror_setup_with_mode(OneDriveMirrorSetupMode::Interactive)
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
        {
            self.last_notice = Some(
                "OneDrive mirror setup is only available for OneDrive Offline Mirror connections."
                    .into(),
            );
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
                self.last_notice = Some(format!(
                    "Starting OneDrive mirror setup for `{}`. Complete browser authorization and keep this window open. After the browser step, onedrive may take several minutes to finish and Cloud Mounter will run a dry-run validation before reporting completion.",
                    connection.name
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
                self.last_notice = Some(format!(
                    "Starting manual OneDrive mirror auth handoff for `{}`. When the auth URL is ready, press Open OneDrive Auth Helper. After authorization, Cloud Mounter waits for onedrive and then runs dry-run validation; this can take several minutes.",
                    connection.name
                ));
                Task::perform(
                    async move { run_onedrive_mirror_manual_setup_result(connection, auth_files).await },
                    |result| cosmic::Action::App(Message::OneDriveMirrorSetupCompleted(result)),
                )
            }
        }
    }

    fn submit_onedrive_mirror_auth_response(&mut self) {
        let Some(connection_id) = self.draft.id else {
            self.last_notice =
                Some("Start OneDrive Mirror Setup before submitting a response URL.".into());
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
                self.last_notice = Some(
                    "OneDrive response URL was written to the transient response file. Waiting for onedrive to finish authentication for this connection."
                        .into(),
                );
            }
            Err(error) => {
                self.last_notice = Some(format!(
                    "Could not write OneDrive response URL handoff file: {error}"
                ));
            }
        }
    }

    fn open_onedrive_mirror_auth_url(&mut self) {
        let Some(connection_id) = self.draft.id else {
            self.last_notice =
                Some("Start OneDrive Mirror Setup before opening the auth URL.".into());
            return;
        };
        let auth_files = onedrive_auth_files_for_connection(connection_id);
        match open_onedrive_auth_url(&auth_files) {
            Ok(message) => {
                self.last_notice = Some(message.into());
            }
            Err(error) => {
                self.last_notice = Some(format!("Could not open OneDrive auth URL: {error}"));
            }
        }
    }

    fn matching_rclone_remotes_in(
        &self,
        remotes: &[RcloneDraftRemote],
        provider: Provider,
    ) -> Vec<RcloneDraftRemote> {
        let Some(expected_backend) = rclone_backend_name(provider) else {
            return Vec::new();
        };
        remotes
            .iter()
            .filter(|remote| remote.backend == expected_backend)
            .cloned()
            .collect()
    }

    fn confirm_import(&mut self, index: usize) -> Task<cosmic::Action<Message>> {
        let Some(preview) = self.import_previews.get(index).cloned() else {
            self.last_notice = Some("Import preview is no longer available.".into());
            return Task::none();
        };
        if preview.active_conflict || preview.local_target_conflict {
            self.last_notice = Some(format!(
                "{} cannot be imported until active-service or local-target conflicts are resolved.",
                preview.original_unit_name
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
                self.last_notice = Some(format!("Import replacement plan failed: {error}"));
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
                self.last_notice = Some(format!(
                    "Importing {original_unit_name} as {name}. Installing applet-managed replacement unit; original service is preserved."
                ));
                Task::perform(
                    async move { install_import_replacement_unit(plan).await },
                    |notice| cosmic::Action::App(Message::DraftSaved(notice)),
                )
            }
            Err(error) => {
                self.last_notice = Some(format!("Failed to save imported {name}: {error}"));
                Task::none()
            }
        }
    }

    fn test_draft_plan(&mut self) -> Task<cosmic::Action<Message>> {
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
            format!(
                "Checking the SharePoint library and selected folder for {}...",
                connection.name
            )
        } else {
            format!(
                "Testing {}. OneDrive mirror validation runs a dry-run preview and can take several minutes for a large drive or slow Microsoft response...",
                connection.name
            )
        });
        Task::perform(
            async move {
                let mut connection = connection;
                let result = if connection.provider == Provider::Teams {
                    test_teams_draft_connection(&mut connection).await
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
            self.last_notice = Some("Connection is no longer available.".into());
            return Task::none();
        };
        let name = connection.name.clone();
        if self.pending_remove != Some(connection_id) {
            self.pending_remove = Some(connection_id);
            self.last_notice = Some(format!(
                "Remove selected for {name}. Press Remove again to confirm removing the applet configuration and applet-owned generated units. Credentials, local data, cache, recovery data, and external services are preserved."
            ));
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
                self.last_notice = Some(format!(
                    "{name} was removed from applet configuration. Removing applet-owned generated units..."
                ));
                Task::perform(
                    async move { remove_generated_units_for_connection(connection).await },
                    |notice| cosmic::Action::App(Message::RemoveCompleted(notice)),
                )
            }
            Ok(false) => {
                self.removing_connection = None;
                self.last_notice = Some(format!("{name} was already absent."));
                Task::none()
            }
            Err(error) => {
                self.removing_connection = None;
                self.last_notice = Some(format!("Failed to remove {name}: {error}"));
                Task::none()
            }
        }
    }

    fn scan_imports(&mut self) {
        let Some(home) = home_dir() else {
            self.import_previews.clear();
            self.last_notice =
                Some("Cannot scan legacy services because HOME is unavailable.".into());
            return;
        };
        let directory = default_scan_directory(&home);
        let active_units = BTreeSet::new();
        let rclone_remotes = run_sync_host_command("rclone", &["config", "dump"])
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| {
                parse_rclone_backends(&String::from_utf8_lossy(&output.stdout)).ok()
            });
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
                    format!(
                        "Import scan found {count} service preview(s). Could not read host rclone configuration; rclone mounts cannot be identified for import."
                    )
                } else if errors.is_empty() {
                    format!("Import scan found {count} compatible service preview(s).")
                } else {
                    format!(
                        "Import scan found {count} compatible service preview(s); {} unit(s) could not be previewed.",
                        errors.len()
                    )
                });
            }
            Err(error) => {
                self.import_previews.clear();
                self.last_notice = Some(format!("Import scan failed: {error}"));
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

    fn controller_snapshot(&self) -> ControllerSnapshot {
        let (sync_state, paused_syncs) =
            runtime_offline_mirror_states(&self.config.document.connections);
        ControllerSnapshot {
            config: self.config.document.clone(),
            service_status: runtime_service_statuses(&self.config.document.connections),
            sync_state,
            paused_syncs,
            mount_entries: HostVisibleMountTable.entries().unwrap_or_default(),
            import_previews: self.import_previews.clone(),
            vpn_ready: self.vpn_ready.clone(),
            network_ready: self.network_ready.unwrap_or(true),
            ..ControllerSnapshot::default()
        }
    }
}

fn runtime_offline_mirror_states(
    connections: &[Connection],
) -> (
    BTreeMap<ConnectionId, SyncRuntimeState>,
    BTreeSet<ConnectionId>,
) {
    let mut states = BTreeMap::new();
    let mut paused = BTreeSet::new();
    for connection in connections
        .iter()
        .filter(|connection| matches!(connection.mode, ConnectionMode::OfflineMirror(_)))
    {
        let state = runtime_offline_mirror_state(connection);
        if state == SyncRuntimeState::Paused {
            paused.insert(connection.id);
        }
        states.insert(connection.id, state);
    }
    (states, paused)
}

async fn current_vpn_ready_states(config: ConfigDocument) -> BTreeMap<VpnProfileId, bool> {
    let mounts = HostVisibleMountTable.entries().unwrap_or_default();
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
    let runner = app_command_runner();
    for profile in profiles {
        let is_ready = match profile.kind {
            VpnKind::NetworkManager => CommandNetworkManagerVpn::new(runner)
                .state(&profile, CancellationToken::new())
                .await
                .is_ok_and(|state| state.ready()),
            VpnKind::Cisco => CommandCiscoVpn::new(runner)
                .components(CancellationToken::new())
                .await
                .is_ok_and(|components| components.tunnel == CiscoTunnelState::Connected),
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

#[cfg(test)]
fn runtime_cisco_tunnel_state(output: &str) -> CiscoTunnelState {
    let lower = output.to_ascii_lowercase();
    if lower.contains("cannot contact the vpn service") {
        return CiscoTunnelState::ServiceUnavailable;
    }
    for line in lower.lines() {
        let Some((_, state)) = line.split_once("connection state:") else {
            continue;
        };
        let state = state.trim();
        if state.starts_with("connected") {
            return CiscoTunnelState::Connected;
        }
        if state.starts_with("connecting") {
            return CiscoTunnelState::Connecting;
        }
        if state.starts_with("disconnected") || state.starts_with("not available") {
            return CiscoTunnelState::Disconnected;
        }
    }
    CiscoTunnelState::Unknown
}

fn runtime_offline_mirror_state(connection: &Connection) -> SyncRuntimeState {
    let service_status = runtime_unit_status(connection.id, UnitKind::Service);
    match connection.provider {
        Provider::OneDrive => {
            if service_status.as_ref().is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) {
                SyncRuntimeState::Idle
            } else {
                SyncRuntimeState::Paused
            }
        }
        Provider::Teams
        | Provider::GoogleDrive
        | Provider::Box
        | Provider::Smb
        | Provider::Sftp => {
            if service_status.as_ref().is_some_and(|status| {
                matches!(status.active, ActiveState::Active | ActiveState::Activating)
            }) {
                return SyncRuntimeState::Running;
            }
            if runtime_unit_status(connection.id, UnitKind::Timer)
                .as_ref()
                .is_some_and(|status| {
                    matches!(status.active, ActiveState::Active | ActiveState::Activating)
                })
            {
                SyncRuntimeState::Idle
            } else {
                SyncRuntimeState::Paused
            }
        }
    }
}

fn runtime_service_statuses(connections: &[Connection]) -> BTreeMap<ConnectionId, UnitStatus> {
    connections
        .iter()
        .filter(|connection| matches!(connection.mode, ConnectionMode::OnlineMount(_)))
        .filter_map(|connection| {
            runtime_service_status(connection.id).map(|status| (connection.id, status))
        })
        .collect()
}

fn runtime_service_status(connection_id: ConnectionId) -> Option<UnitStatus> {
    runtime_unit_status(connection_id, UnitKind::Service)
}

fn runtime_unit_status(connection_id: ConnectionId, unit_kind: UnitKind) -> Option<UnitStatus> {
    let unit = UnitName::new(connection_id, unit_kind);
    let output = run_sync_host_command(
        "systemctl",
        &[
            "--user",
            "show",
            "--property=ActiveState",
            "--property=UnitFileState",
            "--property=SubState",
            "--property=Result",
            unit.file_name().as_str(),
        ],
    )
    .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_runtime_systemd_status(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

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
            self.last_notice = Some("Could not locate the settings executable.".into());
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
            self.last_notice = Some(format!("Could not open settings: {error}"));
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
            self.last_notice = Some("Connection is no longer available.".into());
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
                self.last_notice = Some("Connection order saved.".into());
                notify_runtime_task()
            }
            Err(error) => {
                self.last_notice = Some(format!("Could not save connection order: {error}"));
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
        .map_err(|error| format!("Could not save setting: {error}"))
}

fn load_runtime_config() -> Result<Config, String> {
    let report = Config::load_runtime();
    if report.warnings.is_empty() {
        Ok(report.config)
    } else {
        Err(format!(
            "Could not reload configuration: {}",
            report.warnings.join("; ")
        ))
    }
}

fn sleep_needs_attention(status: Option<&str>) -> bool {
    status.is_some_and(|status| {
        status.starts_with("Sleep cleanup unavailable")
            || status.starts_with("Sleep cleanup incomplete")
            || status.starts_with("Sleep already in progress")
    })
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
    if row.provider == Provider::Teams && row.mode == AccessMode::OfflineMirror {
        return Some(Operation::SyncNow);
    }
    let preferred = match row.status {
        ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted | OnlineMountStatus::Mounting) => {
            Operation::Unmount
        }
        ConnectionStatus::OnlineMount(OnlineMountStatus::Error) => Operation::Repair,
        ConnectionStatus::OnlineMount(_) => Operation::Mount,
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Paused) => Operation::ResumeSync,
        ConnectionStatus::OfflineMirror(_) => Operation::PauseSync,
    };
    row.actions
        .iter()
        .find(|action| action.operation == preferred)
        .or_else(|| {
            row.actions
                .iter()
                .find(|action| action.operation != Operation::Repair)
        })
        .map(|action| action.operation)
}

fn primary_control_enabled(status: &ConnectionStatus) -> bool {
    match status {
        ConnectionStatus::OnlineMount(
            OnlineMountStatus::Mounted
            | OnlineMountStatus::Mounting
            | OnlineMountStatus::PendingWrites
            | OnlineMountStatus::Detaching,
        ) => true,
        ConnectionStatus::OnlineMount(_) => false,
        ConnectionStatus::OfflineMirror(
            OfflineMirrorStatus::Idle
            | OfflineMirrorStatus::Previewing
            | OfflineMirrorStatus::Syncing
            | OfflineMirrorStatus::Conflict,
        ) => true,
        ConnectionStatus::OfflineMirror(_) => false,
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

fn run_sync_host_command(executable: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    match CommandExecutionMode::detect_current() {
        CommandExecutionMode::Native => Command::new(executable).args(args).output(),
        CommandExecutionMode::FlatpakSpawnHost => Command::new("flatpak-spawn")
            .arg("--host")
            .arg(executable)
            .args(args)
            .output(),
    }
}

struct HostVisibleMountTable;

impl MountTable for HostVisibleMountTable {
    fn entries(&self) -> Result<Vec<MountEntry>, MountTableError> {
        if !running_in_flatpak() {
            return ProcMountTable::default().entries();
        }
        let output = run_sync_host_command("cat", &["/proc/self/mountinfo"])
            .map_err(|error| MountTableError(format!("read host mount table: {error}")))?;
        if !output.status.success() || output.stdout.len() > 4 * 1024 * 1024 {
            return Err(MountTableError(
                "host mount table is unavailable or too large".into(),
            ));
        }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(cosmic_ext_applet_mounter::mounts::parse_mountinfo_line)
            .collect()
    }
}

fn row_operation_label(status: &ConnectionStatus, operation: Operation) -> &'static str {
    match (status, operation) {
        (ConnectionStatus::OfflineMirror(_), Operation::PauseSync) => "Stop",
        (ConnectionStatus::OfflineMirror(_), Operation::ResumeSync) => "Start",
        _ => operation_label(operation),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn window_mode_notice(mode: WindowMode) -> String {
    match mode {
        WindowMode::GeneralSettings => "Manage connections and choose sleep behavior.".into(),
        WindowMode::AddConnection => {
            "Add connection selected. Choose provider, mode, remote/subtree, local target, VPN, and start at login.".into()
        }
        WindowMode::ModifyConnection(_) => "Modify connection selected.".into(),
        WindowMode::ImportLegacy => {
            "Import selected. Scan ~/.config/systemd/user, preview compatible services, and confirm replacements before any changes.".into()
        }
        WindowMode::ReorderConnections => {
            "Choose a new position from a connection's dropdown.".into()
        }
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
        remote_reference: connection.remote_reference.clone(),
        remote_subpath: connection.remote_subpath.clone().unwrap_or_default(),
        teams_library_url: connection
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
        return Err("Connection name is required.".into());
    }
    let remote_reference = draft.remote_reference.trim();
    if remote_reference.is_empty() {
        return Err("Remote/account is required.".into());
    }
    if draft.local_path.trim().is_empty() {
        return Err("Local target is required.".into());
    }
    let local_path = expand_user_path(draft.local_path.trim());
    let remote_subpath =
        (!draft.remote_subpath.trim().is_empty()).then(|| draft.remote_subpath.trim().to_owned());
    let teams_identity = if draft.provider == Provider::Teams {
        let mut identity = teams::parse_library_url(&draft.teams_library_url)?;
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
                .map_err(|_| "Cache limit must be a whole number of GiB.".to_owned())?;
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
                .map_err(|_| "Sync interval must be a whole number of minutes.".to_owned())?;
            let recovery_directory = if draft.recovery_directory.trim().is_empty() {
                default_recovery_directory_for(&local_path, id)
            } else {
                expand_user_path(draft.recovery_directory.trim())
            };
            ConnectionMode::OfflineMirror(OfflineMirrorConfig {
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
        let maximum_seconds = draft.connection_preload_seconds.trim().parse::<u64>().map_err(|_| {
            format!("Preload time must be from {MIN_PRELOAD_SECONDS} to {MAX_PRELOAD_SECONDS} seconds.")
        })?;
        let maximum_depth = draft
            .connection_preload_depth
            .trim()
            .parse::<u8>()
            .map_err(|_| {
                format!(
                    "Preload depth must be from {MIN_PRELOAD_DEPTH} to {MAX_PRELOAD_DEPTH} levels."
                )
            })?;
        if !(MIN_PRELOAD_SECONDS..=MAX_PRELOAD_SECONDS).contains(&maximum_seconds) {
            return Err(format!(
                "Preload time must be from {MIN_PRELOAD_SECONDS} to {MAX_PRELOAD_SECONDS} seconds."
            ));
        }
        if !(MIN_PRELOAD_DEPTH..=MAX_PRELOAD_DEPTH).contains(&maximum_depth) {
            return Err(format!(
                "Preload depth must be from {MIN_PRELOAD_DEPTH} to {MAX_PRELOAD_DEPTH} levels."
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
    })
}

fn teams_verification_matches_draft(draft: &ConnectionDraft, verified: &Connection) -> bool {
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
        "auto: sibling .cosmic-mounter-recovery directory".into()
    } else {
        format!(
            "auto: {}",
            default_recovery_directory_for(&expand_user_path(local_path), id).display()
        )
    }
}

fn draft_summary_text(draft: &ConnectionDraft) -> String {
    let engine = provider_engine_summary(draft.provider, draft.access_mode);
    let mut parts = vec![format!("Engine: {engine}.")];
    match connection_from_draft(draft) {
        Ok(connection) => match managed_plan_summary(&connection) {
            Ok(summary) => parts.push(summary),
            Err(error) => parts.push(format!("Generated unit preview pending: {error}")),
        },
        Err(_) => {
            parts.push("Generated unit preview appears after required fields are complete.".into());
        }
    }
    parts.push(match draft.access_mode {
        AccessMode::OnlineMount => {
            "Safety: Test Connection validates dependency, remote/subtree access, mountpoint, and generated unit before Save.".into()
        }
        AccessMode::OfflineMirror => {
            if draft.provider == Provider::Teams {
                fl!("sharepoint-manual-only-help")
            } else {
                "Safety: Preview is dry-run; initial synchronization requires Preview plus confirmed Sync Now before background Start.".into()
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
            detection
                .warnings
                .push(format!("Could not start VPN detector: {error}."));
            return detection;
        }
    };

    runtime.block_on(async {
        let cancellation = CancellationToken::new();
        let runner = app_command_runner();
        detect_network_manager_vpns_with_debug(&runner, &mut detection, cancellation.clone()).await;

        let network_manager = CommandNetworkManagerVpn::new(runner);
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
                detection
                    .warnings
                    .push(format!("NetworkManager VPN detection failed: {error}."));
            }
        }

        let cisco = CommandCiscoVpn::new(runner);
        match cisco.components(cancellation).await {
            Ok(components) if components.cli || components.gui || components.agent => {
                writeln!(detection.debug, "Cisco detected: {components:?}").ok();
                detection.profiles.push(cisco_profile());
            }
            Ok(components) => {
                writeln!(detection.debug, "Cisco not available: {components:?}").ok();
            }
            Err(error) => {
                writeln!(detection.debug, "Cisco error: {error}").ok();
                detection
                    .warnings
                    .push(format!("Cisco Secure Client detection failed: {error}."));
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

fn log_nmcli_error(debug: &mut String, command: &str, error: &CommandError) {
    writeln!(debug, "command: {command}\n  error: {error}").ok();
}

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
                }
            })
        })
        .collect()
}

fn valid_nmcli_profile_fields_for_app(name: &str, vpn_type: &str, uuid: &str) -> bool {
    !name.trim().is_empty()
        && looks_like_uuid_for_app(uuid.trim())
        && !vpn_type.chars().any(char::is_whitespace)
        && is_vpn_type_for_app(vpn_type)
}

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

fn looks_like_uuid_for_app(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

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
    VpnProfile {
        id: VpnProfileId::new(),
        name: profile.name,
        kind: VpnKind::NetworkManager,
        external_profile_id: Some(profile.uuid),
        readiness_checks: vec![
            cosmic_ext_applet_mounter::model::ReadinessCheck::NetworkManagerState,
        ],
        timeout_seconds: 30,
    }
}

fn cisco_profile() -> VpnProfile {
    VpnProfile {
        id: VpnProfileId::new(),
        name: "Cisco Secure Client".into(),
        kind: VpnKind::Cisco,
        external_profile_id: None,
        readiness_checks: Vec::new(),
        timeout_seconds: 90,
    }
}

fn same_vpn_reference(existing: &VpnProfile, detected: &VpnProfile) -> bool {
    existing.kind == detected.kind
        && match detected.kind {
            VpnKind::NetworkManager => {
                existing.external_profile_id == detected.external_profile_id
                    && detected.external_profile_id.is_some()
            }
            VpnKind::Cisco => true,
        }
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
        .as_deref()
        .unwrap_or("interactive/client-selected");
    format!(
        "{}: {}. External profile: {}. Readiness: {}. Timeout: {} seconds. The applet may request activation before mount/sync; authentication remains with the VPN client.",
        vpn_kind_label(profile.kind),
        profile.name,
        external,
        readiness_summary(profile),
        profile.timeout_seconds
    )
}

fn readiness_summary(profile: &VpnProfile) -> String {
    if profile.readiness_checks.is_empty() {
        return "default tunnel readiness".into();
    }
    profile
        .readiness_checks
        .iter()
        .map(|check| match check {
            cosmic_ext_applet_mounter::model::ReadinessCheck::NetworkManagerState => {
                "NetworkManager state".to_owned()
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Interface(value) => {
                format!("interface {value}")
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Route(value) => {
                format!("route {value}")
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::DnsName(value) => {
                format!("DNS {value}")
            }
            cosmic_ext_applet_mounter::model::ReadinessCheck::Endpoint(value) => {
                format!("endpoint {value}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

const fn vpn_kind_label(kind: VpnKind) -> &'static str {
    match kind {
        VpnKind::NetworkManager => "NetworkManager VPN",
        VpnKind::Cisco => "Cisco Secure Client",
    }
}

fn section_row<'a>(
    title: &'static str,
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
    title: &'static str,
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
            .push(widget::text::body("seconds")),
        "Maximum duration of the background preload task: 5–600 seconds. Browsing remains available while the preload runs and after it stops.",
    );
    let mut row = widget::Row::new()
        .spacing(16)
        .align_y(Alignment::Center)
        .push(provider_toggle)
        .push(duration);
    if let Some(depth) = &draft.maximum_depth {
        row = row.push(
            field_with_help(
                widget::Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        widget::text_input::text_input("2", depth)
                            .on_input(move |value| {
                                Message::PreloadDepthInput(provider, value)
                            })
                            .width(Length::Fixed(60.0)),
                    )
                    .push(widget::text::body("levels")),
                "Maximum recursive directory depth: 1–10 levels. Higher values create more storage-server requests, may trigger provider rate limits, and can consume the entire preload time.",
            ),
        );
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
        "Provider changes are disabled while modifying an existing connection. Create or duplicate a connection to use a different provider.",
    )
}

fn mode_choice(
    label: &'static str,
    mode: AccessMode,
    selected: AccessMode,
    locked: bool,
) -> Element<'static, Message> {
    locked_select_button(
        label,
        mode == selected,
        Message::DraftAccessMode(mode),
        locked,
        "Access mode changes are disabled while modifying an existing connection. Create or duplicate a connection to switch between Online mount and Offline mirror.",
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
    locked_help: &'a str,
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

fn local_target_label(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => "Mountpoint",
        AccessMode::OfflineMirror => "Mirror directory",
    }
}

async fn pick_local_folder(mode: AccessMode) -> Result<Option<String>, String> {
    let dialog =
        file_chooser::open::Dialog::new().title(format!("Choose {}", local_target_label(mode)));

    match dialog.open_folder().await {
        Ok(response) => response
            .url()
            .to_file_path()
            .map_err(|()| "selected folder is not a local filesystem path".to_string())
            .and_then(|path| host_visible_selected_folder_path(&path).map(Some)),
        Err(file_chooser::Error::Cancelled) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn selected_folder_path(path: &Path) -> String {
    path.display().to_string()
}

fn host_visible_selected_folder_path(path: &Path) -> Result<String, String> {
    document_portal_origin_path(path).map(|origin| {
        origin
            .as_deref()
            .map(selected_folder_path)
            .unwrap_or_else(|| selected_folder_path(path))
    })
}

fn document_portal_origin_path(path: &Path) -> Result<Option<PathBuf>, String> {
    if !is_document_portal_path(path) {
        return Ok(None);
    }
    let path_text = path.display().to_string();
    let output = run_sync_host_command("flatpak", &["document-info", &path_text]).map_err(
        |error| {
            format!(
                "selected folder is a document-portal path and could not be resolved to its original host path: {error}"
            )
        },
    )?;
    if !output.status.success() {
        return Err(format!(
            "selected folder is a document-portal path and `flatpak document-info` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    parse_document_portal_origin(&String::from_utf8_lossy(&output.stdout))
        .map(Some)
        .ok_or_else(|| {
            "selected folder is a document-portal path but its original host path was not reported"
                .into()
        })
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
        Provider::OneDrive => "remote name",
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
            Provider::OneDrive => "OneDrive uses its own authentication workflow.".into(),
        };
    }

    match provider {
        Provider::GoogleDrive => fl!("google-drive-remote-help-add"),
        Provider::Box => fl!("box-remote-help-add"),
        Provider::Smb => fl!("smb-remote-help-add"),
        Provider::Sftp => fl!("sftp-remote-help-add"),
        Provider::Teams => fl!("teams-remote-help-add"),
        Provider::OneDrive => {
            "OneDrive does not use rclone in the approved provider matrix.".into()
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
    let value: serde_json::Value =
        serde_json::from_str(output).map_err(|error| format!("invalid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "rclone config dump did not return an object".to_owned())?;
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
                    == Some("documentLibrary"))
        {
            remotes.push(RcloneDraftRemote {
                name: name.clone(),
                backend: backend.to_owned(),
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

fn load_smb_remote_details(remote_name: &str) -> Result<Option<SmbRemoteDetails>, String> {
    match run_sync_host_command("rclone", &["config", "dump"]) {
        Ok(output) if output.status.success() => {
            parse_smb_remote_details(&String::from_utf8_lossy(&output.stdout), remote_name)
        }
        Ok(output) => Err(format!(
            "rclone config dump failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => Err(format!("could not run rclone config dump: {error}")),
    }
}

fn parse_smb_remote_details(
    output: &str,
    remote_name: &str,
) -> Result<Option<SmbRemoteDetails>, String> {
    let value: serde_json::Value =
        serde_json::from_str(output).map_err(|error| format!("invalid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "rclone config dump did not return an object".to_owned())?;
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
    let value: serde_json::Value =
        serde_json::from_str(output).map_err(|error| format!("invalid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "rclone config dump did not return an object".to_owned())?;
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
            return Err("Google Drive remote setup requires the Google Drive provider.".into());
        }
        let name = draft.remote_reference.trim();
        validate_rclone_remote_create_name(name)?;
        let client_id = optional_setup_value("Google OAuth client ID", &draft.google_client_id)?;
        let client_secret =
            optional_secret_value("Google OAuth client secret", &draft.google_client_secret)?;
        if client_id.is_some() != client_secret.is_some() {
            return Err(
                "Google OAuth client ID and client secret must be entered together.".into(),
            );
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
            return Err("Box remote setup requires the Box provider.".into());
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
            return Err("SMB remote setup requires the SMB provider.".into());
        }
        let name = draft.remote_reference.trim();
        validate_rclone_remote_create_name(name)?;
        let host = optional_setup_value("SMB host", &draft.smb_host)?;
        let user = optional_setup_value("SMB username", &draft.smb_user)?;
        let domain = optional_setup_value("SMB domain", &draft.smb_domain)?;
        let password = optional_secret_value("SMB password", &draft.smb_password)?;
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
                    "Google Drive OAuth setup",
                    error,
                ));
            }
            false
        }
        (GoogleDriveRemoteAction::Create, Some(backend)) => {
            return Err(format!(
                "rclone remote `{}` already exists with backend `{backend}`. Select it without recreating it, or use Modify to update its Google OAuth client.",
                setup.name
            ));
        }
        (GoogleDriveRemoteAction::Update, Some(backend)) if backend == "drive" => {
            if setup.client_id.is_none() {
                return Err(
                    "Enter the Google OAuth client ID and matching client secret before updating this remote."
                        .into(),
                );
            }
            runner
                .run(
                    google_drive_rclone_config_update_request(&setup)?,
                    CancellationToken::new(),
                )
                .await
                .map_err(|error| rclone_setup_command_error("Google Drive OAuth update", error))?;
            true
        }
        (GoogleDriveRemoteAction::Update, Some(backend)) => {
            return Err(format!(
                "rclone remote `{}` uses backend `{backend}`, not Google Drive. Its OAuth client was not changed.",
                setup.name
            ));
        }
        (GoogleDriveRemoteAction::Update, None) => {
            return Err(format!(
                "Google Drive rclone remote `{}` no longer exists. Detect remotes or create it again before updating its OAuth client.",
                setup.name
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
        return Err(rclone_setup_command_error("Box OAuth setup", error));
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
                return Err(rclone_setup_command_error("config create", error));
            }
        }
        Some(backend) if backend == "smb" => {
            if !metadata_updated && setup.password.is_none() {
                return Err(
                    "Enter SMB host, username, domain, or password before updating this remote."
                        .into(),
                );
            }
            if metadata_updated {
                runner
                    .run(
                        smb_rclone_config_update_request(&setup)?,
                        CancellationToken::new(),
                    )
                    .await
                    .map_err(|error| rclone_setup_command_error("config update", error))?;
            }
        }
        Some(backend) => {
            return Err(format!(
                "rclone remote `{}` already exists with backend `{backend}`. Choose a different SMB remote name.",
                setup.name
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
            .map_err(|error| rclone_setup_command_error("config password", error))?;
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
        return Err(
            "rclone is missing. Install rclone 1.74.3 or newer before removing remotes.".into(),
        );
    }
    runner
        .run(
            rclone_config_delete_request(&remote_name)?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error("config delete", error))?;
    Ok(remote_name)
}

async fn run_onedriver_online_setup_result(connection: Connection) -> Result<String, String> {
    run_onedriver_online_setup_with(
        &app_command_runner(),
        &HostVisibleMountTable,
        &connection,
        &default_cache_root(),
        &default_config_root(),
    )
    .await
}

async fn run_onedriver_online_setup_with(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    cache_root: &Path,
    config_root: &Path,
) -> Result<String, String> {
    if !is_onedriver_online_mount(connection) {
        return Err("onedriver setup only applies to OneDrive Online Mount connections.".into());
    }
    if runner.resolve(Executable::Onedriver).is_none() {
        return Err(
            "onedriver is missing. Install jstaf/onedriver 0.15.0 or newer before starting OneDrive Online Mount setup."
                .into(),
        );
    }
    prepare_onedriver_online_mount_runtime_with(connection, cache_root, config_root)?;
    let plan = onedriver_mount_plan(connection, cache_root, config_root)
        .map_err(|error| format!("could not build onedriver plan: {error}"))?;
    let mount_entries = mount_table
        .entries()
        .map_err(|error| format!("could not inspect active mounts: {error}"))?;
    if let Some(entry) = conflicting_onedriver_mount(&plan.mountpoint, &mount_entries) {
        return Err(format!(
            "mountpoint `{}` overlaps an active onedriver mount at `{}`. Unmount the existing OneDrive mount or choose a separate mountpoint.",
            plan.mountpoint.display(),
            entry.target.display()
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
    run_onedrive_mirror_interactive_setup_with(
        &app_command_runner(),
        &HostVisibleMountTable,
        &connection,
        &default_config_root(),
    )
    .await
}

async fn run_onedrive_mirror_manual_setup_result(
    connection: Connection,
    auth_files: OneDriveMirrorAuthFiles,
) -> Result<String, String> {
    run_onedrive_mirror_manual_setup_with(
        &app_command_runner(),
        &HostVisibleMountTable,
        &connection,
        &default_config_root(),
        &auth_files,
    )
    .await
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
                    "interactive authentication",
                    error,
                ))
            }
        });
    if let Err(Some(error)) = auth_result {
        return Err(format!(
            "{error}. If the browser did not return to onedrive automatically, use Manual Auth Handoff."
        ));
    }
    let token_file = plan.config_directory.join("refresh_token");
    if token_file
        .metadata()
        .map(|metadata| metadata.len() == 0)
        .unwrap_or(true)
    {
        return Err(format!(
            "OneDrive interactive authorization did not create `{}`. The browser may not have returned to onedrive in this applet session; use Manual Auth Handoff for this connection.",
            token_file.display()
        ));
    }
    verify_onedrive_offline_mirror_setup_with(connection, runner, mount_table, config_root)
        .await
        .map_err(|error| {
            format!(
                "OneDrive interactive authorization completed, but post-auth validation failed: {error}"
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
                    "manual authentication",
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
            format!(
                "OneDrive manual/WebKit authorization completed, but post-auth validation failed: {error}"
            )
        })
}

fn prepare_onedrive_mirror_setup_plan(
    runner: &dyn CommandRunner,
    mount_table: &dyn MountTable,
    connection: &Connection,
    config_root: &Path,
) -> Result<cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan, String> {
    if !is_onedrive_offline_mirror(connection) {
        return Err(
            "OneDrive mirror setup only applies to OneDrive Offline Mirror connections.".into(),
        );
    }
    if runner.resolve(Executable::OneDrive).is_none() {
        return Err(
            "onedrive is missing. Install abraunegg/onedrive 2.5.10 or newer before starting OneDrive Offline Mirror setup."
                .into(),
        );
    }
    prepare_onedrive_offline_mirror_runtime_with(connection, config_root)?;
    let active_onedriver_paths = mount_table
        .entries()
        .map_err(|error| format!("could not inspect active mounts: {error}"))?
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
        return Err(
            "rclone is missing. Install rclone 1.74.3 or newer before creating remotes.".into(),
        );
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
        .map_err(|error| rclone_setup_command_error("config dump", error))?;
    if rclone_config_has_section(&dump.stdout.text, remote_name)? {
        return Err(format!(
            "rclone remote `{remote_name}` already exists. Choose a different remote name or select the existing remote."
        ));
    }
    Ok(())
}

async fn rclone_remote_backend(
    runner: &dyn CommandRunner,
    remote_name: &str,
) -> Result<Option<String>, String> {
    if runner.resolve(Executable::Rclone).is_none() {
        return Err(
            "rclone is missing. Install rclone 1.74.3 or newer before creating remotes.".into(),
        );
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
        .map_err(|error| rclone_setup_command_error("config dump", error))?;
    rclone_config_section_backend(&dump.stdout.text, remote_name)
}

fn rclone_config_section_backend(
    output: &str,
    remote_name: &str,
) -> Result<Option<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(output).map_err(|error| format!("invalid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "rclone config dump did not return an object".to_owned())?;
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
        .ok_or_else(|| "Google OAuth client ID is required to update a remote.".to_owned())?;
    let client_secret = setup
        .client_secret
        .as_ref()
        .ok_or_else(|| "Google OAuth client secret is required to update a remote.".to_owned())?;
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
        .map_err(|error| error.to_string())?
        .arg("--non-interactive")
        .map_err(|error| error.to_string())
        .map(|request| request.with_timeout(Duration::from_secs(5 * 60)))
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
        .ok_or_else(|| "SMB host is required to create a new SMB rclone remote.".to_owned())?;
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
        .ok_or_else(|| "SMB password is not set.".to_owned())?;
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
        Err("rclone remote name must use letters, numbers, dots, dashes, or underscores.".into())
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
        return Err(format!("{label} is required."));
    }
    if value
        .chars()
        .any(|character| character == '\0' || character == '\n' || character == '\r')
    {
        return Err(format!("{label} contains unsupported control characters."));
    }
    Ok(())
}

fn rclone_setup_command_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        CommandError::InvalidArgument => {
            "rclone setup command argument contains unsupported characters".into()
        }
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
            if detail.is_empty() {
                format!("rclone {stage} failed without diagnostic output")
            } else {
                format!("rclone {stage} failed: {detail}")
            }
        }
    }
}

fn onedriver_setup_command_error(error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        CommandError::InvalidArgument => {
            "onedriver setup command argument contains unsupported characters".into()
        }
        CommandError::Timeout { timeout, .. } => {
            format!(
                "onedriver authentication timed out after {} seconds",
                timeout.as_secs()
            )
        }
        CommandError::Cancelled { .. } => "onedriver authentication was cancelled".into(),
        CommandError::Spawn { message, .. } => format!("could not start onedriver: {message}"),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            if detail.is_empty() {
                "onedriver authentication failed without diagnostic output".into()
            } else {
                format!("onedriver authentication failed: {detail}")
            }
        }
    }
}

fn toggle_button(
    label: &'static str,
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
            Ok(format!(
                "Managed online mount unit {} validates structurally.",
                document.name.file_name()
            ))
        }
        ConnectionMode::OfflineMirror(_) => match connection.provider {
            Provider::OneDrive => {
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
                Ok(format!(
                    "Managed OneDrive mirror unit {} validates structurally.",
                    document.name.file_name()
                ))
            }
            Provider::Teams
            | Provider::GoogleDrive
            | Provider::Box
            | Provider::Smb
            | Provider::Sftp => {
                let plan = rclone_bisync_plan(connection, &default_work_root())
                    .map_err(|error| error.to_string())?;
                let service = UnitDocument::service(&mirror_script::service_spec(&plan))
                    .map_err(|error| error.to_string())?;
                let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
                if connection.provider == Provider::Teams {
                    return Ok(fl!(
                        "sharepoint-mirror-plan-ready",
                        name = service.name.file_name()
                    ));
                }
                Ok(format!(
                    "Managed bisync service {} and timer {} validate structurally.",
                    service.name.file_name(),
                    timer.name.file_name()
                ))
            }
        },
    }
}

const fn is_rclone_online_mount(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::Teams
                | Provider::GoogleDrive
                | Provider::Box
                | Provider::Smb
                | Provider::Sftp,
            ConnectionMode::OnlineMount(_)
        )
    )
}

const fn is_rclone_offline_mirror(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::Teams
                | Provider::GoogleDrive
                | Provider::Box
                | Provider::Smb
                | Provider::Sftp,
            ConnectionMode::OfflineMirror(_)
        )
    )
}

const fn is_onedriver_online_mount(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (Provider::OneDrive, ConnectionMode::OnlineMount(_))
    )
}

const fn uses_directory_preload(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::OneDrive | Provider::Box | Provider::Smb | Provider::Sftp,
            ConnectionMode::OnlineMount(_)
        )
    )
}

const fn is_onedrive_offline_mirror(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (Provider::OneDrive, ConnectionMode::OfflineMirror(_))
    )
}

async fn install_rclone_online_mount_unit(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_rclone_online_mount_unit_result(&connection).await {
        Ok(enabled) => {
            if enabled {
                format!(
                    "{name} saved and managed mount unit installed. Start at login is enabled; the unit was not started."
                )
            } else {
                format!(
                    "{name} saved and managed mount unit installed. Start at login is disabled; the unit was not started."
                )
            }
        }
        Err(error) => {
            format!("{name} was saved, but managed mount unit installation failed: {error}")
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
        let result = if connection.provider == Provider::OneDrive {
            install_onedriver_online_mount_unit_result(&connection).await
        } else {
            install_rclone_online_mount_unit_result(&connection).await
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
        Ok(()) => {
            if connection.provider == Provider::Teams {
                return fl!("sharepoint-mirror-saved", name = name);
            }
            format!(
                "{name} saved and managed mirror service/timer installed. Automatic sync remains disabled until preview and initial sync are confirmed."
            )
        }
        Err(error) => {
            format!("{name} was saved, but managed mirror unit installation failed: {error}")
        }
    }
}

async fn install_rclone_offline_mirror_units_result(connection: &Connection) -> Result<(), String> {
    prepare_offline_mirror_runtime(connection)?;
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
    prepare_rclone_bisync_work_files(connection, &plan)?;
    let service = UnitDocument::service(&mirror_script::service_spec(&plan))
        .map_err(|error| error.to_string())?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&service, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;
    if let Err(error) = controller.install(&timer, cancellation.child_token()).await {
        let _ = controller
            .remove(&service.name, CancellationToken::new())
            .await;
        return Err(error.to_string());
    }

    let manager = CommandSystemdManager::new(app_command_runner());
    manager
        .action(
            SystemdAction::Disable,
            Some(&service.name),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| error.to_string())?;
    manager
        .action(SystemdAction::Disable, Some(&timer.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn install_onedriver_online_mount_unit(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_onedriver_online_mount_unit_result(&connection).await {
        Ok(enabled) => {
            if enabled {
                format!(
                    "{name} saved and managed onedriver mount unit installed. Start at login is enabled; the unit was not started."
                )
            } else {
                format!(
                    "{name} saved and managed onedriver mount unit installed. Start at login is disabled; the unit was not started."
                )
            }
        }
        Err(error) => {
            format!("{name} was saved, but managed onedriver unit installation failed: {error}")
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
        Ok(()) => {
            format!(
                "{name} saved and managed OneDrive mirror unit installed. Synchronization was not started."
            )
        }
        Err(error) => {
            format!(
                "{name} was saved, but managed OneDrive mirror unit installation failed: {error}"
            )
        }
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
    let manager = CommandSystemdManager::new(app_command_runner());
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
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
            let login = if enabled { "enabled" } else { "disabled" };
            let original = if plan.preserve_original {
                " Original legacy service was preserved."
            } else {
                ""
            };
            format!(
                "{name} imported and applet-managed replacement unit installed. Start at login is {login}.{original}"
            )
        }
        Err(error) => {
            format!(
                "{name} was imported into applet configuration, but managed replacement unit installation failed: {error}"
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
    let manager = CommandSystemdManager::new(app_command_runner());
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
                format!(
                    "{name} was removed. No applet-owned generated units were present. User data, credentials, cache, recovery data, and external services were preserved."
                )
            } else {
                format!(
                    "{name} was removed. Removed {removed} applet-owned generated unit(s). User data, credentials, cache, recovery data, and external services were preserved."
                )
            }
        }
        Err(error) => {
            format!(
                "{name} was removed from applet configuration, but generated unit cleanup needs attention: {error}"
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
    let controller = UnitController::new(store, CommandSystemdManager::new(app_command_runner()));
    let cancellation = CancellationToken::new();
    let mut removed = 0usize;
    for unit in managed_unit_names_for_connection(connection) {
        let manager = CommandSystemdManager::new(app_command_runner());
        let _ = manager
            .action(SystemdAction::Stop, Some(&unit), cancellation.child_token())
            .await;
        let manager = CommandSystemdManager::new(app_command_runner());
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
    match &connection.mode {
        ConnectionMode::OnlineMount(_) => vec![UnitName::new(connection.id, UnitKind::Service)],
        ConnectionMode::OfflineMirror(_) => match connection.provider {
            Provider::Teams => vec![UnitName::new(connection.id, UnitKind::Service)],
            Provider::OneDrive => vec![UnitName::new(connection.id, UnitKind::Service)],
            Provider::GoogleDrive | Provider::Box | Provider::Smb | Provider::Sftp => vec![
                UnitName::new(connection.id, UnitKind::Timer),
                UnitName::new(connection.id, UnitKind::Service),
            ],
        },
    }
}

async fn restore_online_after_wake_result(connection: &Connection) -> Result<(), String> {
    let Ok(token) = cosmic_ext_applet_mounter::sleep::online_operation_token() else {
        return Err(format!("Wake restoration canceled for {}", connection.name));
    };
    let wait = async {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        loop {
            if current_network_ready().await {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(format!(
                    "Wake restoration for {} is waiting for network; retry manually",
                    connection.name
                ));
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        // Re-read after waiting: a removal or disable must override restoration.
        let config = Config::load_runtime().config;
        if !config.document.unmount_before_sleep || !config.document.restore_after_wake {
            return Err(format!("Wake restoration disabled for {}", connection.name));
        }
        let Some(current) = config.document.connections.into_iter().find(|item| {
            item.id == connection.id
                && item.enabled
                && matches!(item.mode, ConnectionMode::OnlineMount(_))
        }) else {
            return Err(format!("Wake restoration skipped for {}", connection.name));
        };
        if current.provider == Provider::OneDrive {
            run_managed_onedriver_online_mount_operation_result(&current, Operation::Mount).await
        } else {
            run_managed_online_mount_operation_result(&current, Operation::Mount).await
        }
    };
    tokio::select! {
        biased;
        () = token.cancelled() => Err(format!("Wake restoration canceled for {}", connection.name)),
        result = wait => result,
    }
}

async fn run_managed_online_mount_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let token = cosmic_ext_applet_mounter::sleep::begin_online_operation()?;
    tokio::select! {
        biased;
        () = token.cancelled() => Err("Online operation canceled for sleep; retry after wake".into()),
        result = run_managed_online_mount_operation_result_inner(connection, operation) => result,
    }
}

async fn run_managed_online_mount_operation_result_inner(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let action = match operation {
        Operation::Mount => SystemdAction::Start,
        Operation::Unmount => SystemdAction::Stop,
        _ => {
            return Err(format!(
                "{} is not a managed mount operation",
                operation_label(operation)
            ));
        }
    };
    let unit = UnitName::new(connection.id, UnitKind::Service);
    if action == SystemdAction::Start {
        prepare_online_mount_runtime(connection)?;
        mount_guard::check_empty_mountpoint(&app_command_runner(), &connection.local_path).await?;
        ensure_vpn_ready_for_connection(connection).await?;
        verify_rclone_access(connection)
            .await
            .map_err(|error| format!("Remote access check failed before mounting: {error}"))?;
        let current = Config::load_runtime().config;
        if !current
            .document
            .connections
            .iter()
            .any(|saved| saved == connection && saved.enabled)
        {
            return Err("Connection was disabled, removed or changed while waiting; retry with its current settings".into());
        }
        // Refresh applet-owned units so provider tuning added by an app update
        // applies to saved connections without requiring an edit-and-save cycle.
        install_rclone_online_mount_unit_result(connection)
            .await
            .map_err(|error| fl!("online-service-refresh-failed", error = error))?;
    }
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();
    if action == SystemdAction::Start {
        return start_managed_online_service(&manager, &unit, cancellation).await;
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
    manager
        .action(action, Some(&unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    if action == SystemdAction::Stop {
        require_mount_disappearance_after_clean_stop(connection).await?;
        maybe_shutdown_vpn_after_unmount(connection).await?;
    }
    Ok(())
}

async fn run_managed_onedriver_online_mount_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<(), String> {
    let token = cosmic_ext_applet_mounter::sleep::begin_online_operation()?;
    tokio::select! {
        biased;
        () = token.cancelled() => Err("Online operation canceled for sleep; retry after wake".into()),
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
            return Err(format!(
                "{} is not a managed onedriver mount operation",
                operation_label(operation)
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
            return Err("Connection was disabled, removed or changed while waiting; retry with its current settings".into());
        }
        install_onedriver_online_mount_unit_result(connection)
            .await
            .map_err(|error| fl!("online-service-refresh-failed", error = error))?;
    }
    let manager = CommandSystemdManager::new(app_command_runner());
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
        return start_managed_online_service(&manager, &unit, cancellation).await;
    } else {
        directory_preload::cancel_before_unmount(connection.id).await?;
        clean_detach_before_service_stop(connection, cancellation.child_token()).await?;
    }
    manager
        .action(action, Some(&unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    if action == SystemdAction::Stop {
        require_mount_disappearance_after_clean_stop(connection).await?;
        maybe_shutdown_vpn_after_unmount(connection).await?;
    }
    Ok(())
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
        let mounted = HostVisibleMountTable
            .entries()
            .map_err(|error| format!("could not check mount before unmount: {error}"))?
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
        let still_mounted = HostVisibleMountTable
            .entries()
            .map_err(|check_error| {
                format!(
                    "Clean unmount failed for {}: {error}; could not verify mount: {check_error}",
                    connection.local_path.display()
                )
            })?
            .iter()
            .any(|entry| entry.target == connection.local_path);
        if !still_mounted {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline || !matches!(error, CommandError::NonZero { .. })
        {
            return Err(format!(
                "Clean unmount could not detach {}: {error}. The mount service was left running; close processes using it and retry",
                connection.local_path.display()
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
        let mounted = HostVisibleMountTable
            .entries()
            .map_err(|error| format!("could not verify unmount: {error}"))?
            .iter()
            .any(|entry| entry.target == connection.local_path);
        if !mounted {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "clean unmount left a busy FUSE endpoint at {}. Close file-browser windows and terminals using it, then select Repair and confirm the lazy unmount",
                connection.local_path.display()
            ));
        }
        tokio::time::sleep(CLEAN_UNMOUNT_POLL_INTERVAL).await;
    }
}

async fn run_online_mount_repair_operation(connection: Connection) -> String {
    match run_online_mount_repair_operation_result(&connection).await {
        Ok(()) => format!(
            "Repair completed for {}. The mountpoint is detached and the generated service is ready for the next mount.",
            connection.name
        ),
        Err(error) => format!("Repair failed for {}: {error}", connection.name),
    }
}

async fn run_online_mount_repair_operation_result(connection: &Connection) -> Result<(), String> {
    if !matches!(connection.mode, ConnectionMode::OnlineMount(_)) {
        return Err("repair is only available for online mounts".into());
    }

    let unit = UnitName::new(connection.id, UnitKind::Service);
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();

    if connection.provider == Provider::GoogleDrive {
        let _ = rclone_refresh::cancel(connection.id).await;
    }
    if uses_directory_preload(connection) {
        directory_preload::cancel(connection.id).await;
    }
    let _ = manager
        .action(SystemdAction::Stop, Some(&unit), cancellation.child_token())
        .await;

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
            return Err(
                "repair could not detach the mountpoint; inspect the mount before retrying".into(),
            );
        }
        tokio::time::sleep(CLEAN_UNMOUNT_POLL_INTERVAL).await;
    }

    let status = manager
        .action(
            SystemdAction::Status,
            Some(&unit),
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
            .action(SystemdAction::ResetFailed, Some(&unit), cancellation)
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

async fn host_mount_entries() -> Result<Vec<cosmic_ext_applet_mounter::mounts::MountEntry>, String>
{
    let request = CommandRequest::new(Executable::Cat)
        .arg("/proc/self/mountinfo")
        .map_err(|error| error.to_string())?
        .with_output_limit(4 * 1024 * 1024);
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| error.to_string())?;
    if output.stdout.truncated {
        return Err("host mount table is truncated".into());
    }
    output
        .stdout
        .text
        .lines()
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
    let expected = if connection.provider == Provider::OneDrive {
        "fuse.onedriver"
    } else {
        "fuse.rclone"
    };
    if mount.filesystem != expected {
        return Err(
            "repair found an unexpected filesystem at the mountpoint; left it untouched".into(),
        );
    }
    Ok(true)
}

fn repair_requires_failed_reset(status: Option<&UnitStatus>) -> bool {
    status.is_some_and(|status| status.active == ActiveState::Failed)
}

async fn run_managed_offline_mirror_operation(
    connection: Connection,
    operation: Operation,
) -> String {
    let label = operation_label(operation);
    match run_managed_offline_mirror_operation_result(&connection, operation).await {
        Ok(summary) => format!("{label} completed for {}. {summary}", connection.name),
        Err(error) => format!("{label} failed for {}: {error}", connection.name),
    }
}

async fn run_managed_offline_mirror_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<String, String> {
    if connection.provider == Provider::Teams
        && matches!(operation, Operation::ResumeSync | Operation::PauseSync)
    {
        return Err(fl!("sharepoint-schedule-disabled"));
    }
    prepare_offline_mirror_runtime(connection)?;
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
    ensure_sharepoint_access_marker(connection, &plan)?;
    prepare_rclone_bisync_work_files(connection, &plan)?;
    match operation {
        Operation::PreviewInitialSync => preview_rclone_offline_mirror(&plan).await,
        Operation::SyncNow => sync_rclone_offline_mirror(connection, &plan).await,
        Operation::ResumeSync => start_rclone_offline_mirror_background(connection, &plan).await,
        Operation::PauseSync => stop_rclone_offline_mirror_background(&plan).await,
        _ => Err(format!(
            "{} is not a managed offline mirror operation",
            operation_label(operation)
        )),
    }
}

async fn run_managed_onedrive_offline_mirror_operation(
    connection: Connection,
    operation: Operation,
) -> String {
    let label = operation_label(operation);
    match run_managed_onedrive_offline_mirror_operation_result(&connection, operation).await {
        Ok(summary) => format!("{label} completed for {}. {summary}", connection.name),
        Err(error) => format!("{label} failed for {}: {error}", connection.name),
    }
}

async fn run_managed_onedrive_offline_mirror_operation_result(
    connection: &Connection,
    operation: Operation,
) -> Result<String, String> {
    prepare_onedrive_offline_mirror_runtime(connection)?;
    let plan = onedrive_mirror_plan_for_app(connection)?;
    match operation {
        Operation::PreviewInitialSync => preview_onedrive_offline_mirror(&plan).await,
        Operation::SyncNow => sync_onedrive_offline_mirror(connection, &plan).await,
        Operation::ResumeSync => start_onedrive_offline_mirror_background(connection, &plan).await,
        Operation::PauseSync => stop_onedrive_offline_mirror_background(&plan).await,
        _ => Err(format!(
            "{} is not a managed OneDrive mirror operation",
            operation_label(operation)
        )),
    }
}

async fn start_rclone_offline_mirror_background(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<String, String> {
    if connection.provider == Provider::Teams {
        return Err(fl!("sharepoint-schedule-disabled"));
    }
    if !initial_sync_marker(plan).exists() {
        return Err(
            "background sync requires a successful Preview and confirmed initial Sync Now first"
                .into(),
        );
    }
    ensure_background_sync_may_start(connection).await?;
    install_rclone_offline_mirror_units_result(connection).await?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();
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
            Some(&timer.name),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| error.to_string())?;
    manager
        .action(SystemdAction::Start, Some(&timer.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok("Background sync timer started. Use Sync Now for an immediate one-shot sync.".into())
}

async fn stop_rclone_offline_mirror_background(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<String, String> {
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();
    let _ = manager
        .action(
            SystemdAction::Stop,
            Some(&timer.name),
            cancellation.child_token(),
        )
        .await;
    let _ = manager
        .action(
            SystemdAction::Stop,
            Some(&service.name),
            cancellation.child_token(),
        )
        .await;
    manager
        .action(SystemdAction::Disable, Some(&timer.name), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok("Background sync timer stopped. Manual Sync Now remains available.".into())
}

async fn start_onedrive_offline_mirror_background(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<String, String> {
    if !onedrive_initial_sync_marker(plan).exists() {
        return Err(
            "background sync requires a successful Preview and confirmed initial Sync Now first"
                .into(),
        );
    }
    ensure_background_sync_may_start(connection).await?;
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();
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
    Ok("OneDrive background monitor started. Use Sync Now for an immediate one-shot sync.".into())
}

async fn ensure_background_sync_may_start(connection: &Connection) -> Result<(), String> {
    let options = match &connection.mode {
        ConnectionMode::OfflineMirror(options) => options,
        ConnectionMode::OnlineMount(_) => return Err("connection is not an offline mirror".into()),
    };
    let request = SyncRequest {
        trigger: SyncTrigger::Scheduled,
        preview_completed: true,
        user_confirmed: true,
        metered_network: current_metered_network().await,
        running: false,
        readiness: SyncReadiness {
            network_ready: current_network_ready().await,
            vpn_ready: current_vpn_ready(connection).await?,
        },
    };
    match sync_now_request(request, options).map_err(|error| error.to_string())? {
        SyncDecision::Run => Ok(()),
        SyncDecision::WaitForNetwork => {
            Err("background sync is waiting for network readiness".into())
        }
        SyncDecision::WaitForVpn => Err("background sync is waiting for VPN readiness".into()),
        SyncDecision::PauseMetered => {
            Err("background sync is paused on metered networks by policy".into())
        }
        SyncDecision::Reject(rejection) => Err(sync_rejection_message(rejection).into()),
    }
}

async fn current_network_ready() -> bool {
    let request = match CommandRequest::new(Executable::Nmcli)
        .arg("-t")
        .and_then(|request| request.arg("-f"))
        .and_then(|request| request.arg("STATE"))
        .and_then(|request| request.arg("general"))
    {
        Ok(request) => request.with_timeout(Duration::from_secs(5)),
        Err(_) => return true,
    };
    match app_command_runner()
        .run(request, CancellationToken::new())
        .await
    {
        Ok(output) => network_manager_connected(&output.stdout.text),
        Err(_) => true,
    }
}

fn network_manager_connected(state: &str) -> bool {
    state.trim().to_ascii_lowercase().starts_with("connected")
}

async fn current_metered_network() -> bool {
    let request = match CommandRequest::new(Executable::Nmcli)
        .arg("-t")
        .and_then(|request| request.arg("-f"))
        .and_then(|request| request.arg("GENERAL.METERED"))
        .and_then(|request| request.arg("device"))
        .and_then(|request| request.arg("show"))
    {
        Ok(request) => request.with_timeout(Duration::from_secs(5)),
        Err(_) => return false,
    };
    match app_command_runner()
        .run(request, CancellationToken::new())
        .await
    {
        Ok(output) => output.stdout.text.lines().any(|line| {
            let value = line
                .split_once(':')
                .map(|(_, value)| value)
                .unwrap_or(line)
                .trim()
                .to_ascii_lowercase();
            matches!(value.as_str(), "yes" | "2")
        }),
        Err(_) => false,
    }
}

async fn current_vpn_ready(connection: &Connection) -> Result<bool, String> {
    let Some(profile_id) = connection.vpn_profile_id else {
        return Ok(true);
    };
    let config = Config::load_runtime().config;
    let Some(profile) = config
        .document
        .vpn_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
    else {
        return Ok(false);
    };
    if profile.readiness_checks.is_empty() {
        return Ok(true);
    }
    let probe = CommandReadinessProbe::new(app_command_runner());
    let ready = readiness_report(&probe, &profile.readiness_checks, CancellationToken::new())
        .await
        .map(|report| report.ready)
        .map_err(|error| error.to_string())?;
    if !ready {
        unmark_vpn_applet_activated(profile.id);
    }
    Ok(ready)
}

async fn ensure_vpn_ready_for_connection(connection: &Connection) -> Result<(), String> {
    let Some(profile_id) = connection.vpn_profile_id else {
        return Ok(());
    };
    let Some(profile) = Config::load_runtime()
        .config
        .document
        .vpn_profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
    else {
        return Err("configured VPN profile is no longer available".into());
    };
    let deadline =
        tokio::time::Instant::now() + Duration::from_secs(u64::from(profile.timeout_seconds));
    tokio::time::timeout_at(deadline, ensure_vpn_profile_ready(&profile, deadline))
        .await.map_err(|_| format!("{} did not become ready within {} seconds; complete authentication and retry the mount", profile.name, profile.timeout_seconds))?
}

async fn ensure_vpn_profile_ready(
    profile: &VpnProfile,
    deadline: tokio::time::Instant,
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
                CommandNetworkManagerVpn::new(app_command_runner())
                    .activate(profile, CancellationToken::new())
                    .await
                    .map_err(|error| format!("could not activate NetworkManager VPN: {error}"))?;
            }
            VpnKind::Cisco => {
                let cisco = CommandCiscoVpn::new(app_command_runner());
                let components = cisco
                    .components(CancellationToken::new())
                    .await
                    .map_err(|error| format!("could not inspect Cisco Secure Client: {error}"))?;
                if matches!(components.tunnel, CiscoTunnelState::NotInstalled) {
                    return Err("Cisco Secure Client is not installed".into());
                }
                if matches!(components.tunnel, CiscoTunnelState::ServiceUnavailable)
                    && let Ok(request) = cisco.start_agent_request()
                {
                    let _ = app_command_runner()
                        .run(request, CancellationToken::new())
                        .await;
                }
                app_command_runner()
                    .run(
                        cisco
                            .open_gui_request()
                            .map_err(|error| error.to_string())?,
                        CancellationToken::new(),
                    )
                    .await
                    .map_err(|error| format!("could not open Cisco Secure Client: {error}"))?;
            }
        }
    }
    let ready = wait_for_vpn_ready(profile, deadline).await?;
    if ready {
        if !already_connected {
            mark_vpn_applet_activated(profile.id);
        }
        Ok(())
    } else {
        Err(format!(
            "{} did not become ready within {} seconds",
            profile.name, profile.timeout_seconds
        ))
    }
}

async fn vpn_tunnel_ready(profile: &VpnProfile) -> Result<bool, String> {
    match profile.kind {
        VpnKind::NetworkManager => CommandNetworkManagerVpn::new(app_command_runner())
            .state(profile, CancellationToken::new())
            .await
            .map(|state| state.ready())
            .map_err(|error| error.to_string()),
        VpnKind::Cisco => CommandCiscoVpn::new(app_command_runner())
            .components(CancellationToken::new())
            .await
            .map(|components| components.tunnel == CiscoTunnelState::Connected)
            .map_err(|error| error.to_string()),
    }
}

async fn vpn_profile_ready(profile: &VpnProfile) -> Result<bool, String> {
    let connected = vpn_tunnel_ready(profile).await?;

    if !connected {
        return Ok(false);
    }
    readiness_report(
        &CommandReadinessProbe::new(app_command_runner()),
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
) -> Result<bool, String> {
    Ok(cosmic_ext_applet_mounter::vpn::poll_readiness_until(
        deadline,
        Duration::from_secs(2),
        || vpn_profile_ready(profile),
    )
    .await)
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
        active_connection_ids: active_connections_using_vpn(profile_id, Some(connection.id)),
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
        VpnKind::NetworkManager => CommandNetworkManagerVpn::new(app_command_runner())
            .deactivate(&profile, CancellationToken::new())
            .await
            .map_err(|error| format!("could not disconnect NetworkManager VPN: {error}")),
        VpnKind::Cisco => {
            let request = CommandRequest::new(Executable::CiscoVpn)
                .arg("disconnect")
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(20));
            app_command_runner()
                .run(request, CancellationToken::new())
                .await
                .map(|_| ())
                .map_err(|error| format!("could not disconnect Cisco Secure Client: {error}"))
        }
    }
}

fn active_connections_using_vpn(
    profile_id: VpnProfileId,
    exclude_connection: Option<ConnectionId>,
) -> BTreeSet<ConnectionId> {
    let config = Config::load_runtime().config.document;
    let mount_entries = HostVisibleMountTable.entries().unwrap_or_default();
    config
        .connections
        .iter()
        .filter(|connection| connection.enabled)
        .filter(|connection| connection.vpn_profile_id == Some(profile_id))
        .filter(|connection| Some(connection.id) != exclude_connection)
        .filter(|connection| connection_appears_active(connection, &mount_entries))
        .map(|connection| connection.id)
        .collect()
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
) -> Result<String, String> {
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(app_command_runner());
    let cancellation = CancellationToken::new();
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
    Ok("OneDrive background monitor stopped. Manual Sync Now remains available.".into())
}

async fn preview_rclone_offline_mirror(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<String, String> {
    let initialized = initial_sync_marker(plan).exists();
    let request = if initialized {
        rclone_bisync_preview_request(plan).map_err(|error| error.to_string())?
    } else {
        rclone_bisync_initial_preview_request(plan).map_err(|error| error.to_string())?
    };
    let request = managed_rclone_bisync_request(
        plan,
        if initialized {
            "preview"
        } else {
            "initial-preview"
        },
        request,
    )?;
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| offline_mirror_command_error("preview", error))?;
    let summary = preview_summary_text(&output);
    if !initialized {
        write_initial_preview_marker(plan, &summary)?;
        Ok(format!(
            "{summary} Initial sync has not run yet; press Sync Now to confirm and run the initial synchronization."
        ))
    } else {
        Ok(summary)
    }
}

async fn preview_onedrive_offline_mirror(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<String, String> {
    let output = app_command_runner()
        .run(
            one_drive_preview_request(plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| onedrive_runtime_command_error("preview", error))?;
    let summary = onedrive_output_summary("Preview", &output);
    if !onedrive_initial_sync_marker(plan).exists() {
        write_onedrive_initial_preview_marker(plan, &summary)?;
        Ok(format!(
            "{summary} Initial sync has not run yet; press Sync Now to confirm and run the initial synchronization."
        ))
    } else {
        Ok(summary)
    }
}

async fn sync_rclone_offline_mirror(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<String, String> {
    let options = match &connection.mode {
        ConnectionMode::OfflineMirror(options) => options,
        ConnectionMode::OnlineMount(_) => return Err("connection is not an offline mirror".into()),
    };
    let initialized = initial_sync_marker(plan).exists();
    let preview_completed = initialized || initial_preview_marker(plan).exists();
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
        SyncDecision::WaitForNetwork => return Err("waiting for network readiness".into()),
        SyncDecision::WaitForVpn => return Err("waiting for VPN readiness".into()),
        SyncDecision::PauseMetered => {
            return Err("automatic synchronization is paused on metered networks".into());
        }
        SyncDecision::Reject(rejection) => return Err(sync_rejection_message(rejection).into()),
    }

    let request = if initialized {
        rclone_bisync_sync_request(plan).map_err(|error| error.to_string())?
    } else {
        rclone_bisync_initial_sync_request(plan).map_err(|error| error.to_string())?
    };
    let request = managed_rclone_bisync_request(
        plan,
        if initialized { "sync" } else { "initial-sync" },
        request,
    )?;
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| offline_mirror_command_error("sync", error))?;
    if initialized {
        Ok(sync_output_summary(&output))
    } else {
        write_initial_sync_marker(plan)?;
        let _ = fs::remove_file(initial_preview_marker(plan));
        Ok(format!(
            "{} Initial synchronization is now recorded as complete; future Sync Now runs use normal bisync.",
            sync_output_summary(&output)
        ))
    }
}

fn managed_rclone_bisync_request(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
    mode: &str,
    original: CommandRequest,
) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Sh)
        .arg(mirror_script::script_path(plan))
        .and_then(|request| request.arg(mode))
        .map(|request| {
            request
                .with_timeout(original.timeout)
                .with_retry(original.retry)
                .with_output_limit(original.output_limit)
        })
        .map_err(|error| error.to_string())
}

async fn sync_onedrive_offline_mirror(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<String, String> {
    let options = match &connection.mode {
        ConnectionMode::OfflineMirror(options) => options,
        ConnectionMode::OnlineMount(_) => return Err("connection is not an offline mirror".into()),
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
        SyncDecision::WaitForNetwork => return Err("waiting for network readiness".into()),
        SyncDecision::WaitForVpn => return Err("waiting for VPN readiness".into()),
        SyncDecision::PauseMetered => {
            return Err("automatic synchronization is paused on metered networks".into());
        }
        SyncDecision::Reject(rejection) => return Err(sync_rejection_message(rejection).into()),
    }

    let request = if initialized {
        one_drive_sync_request(plan).map_err(|error| error.to_string())?
    } else {
        one_drive_initial_sync_request(plan).map_err(|error| error.to_string())?
    };
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| onedrive_runtime_command_error("sync", error))?;
    if initialized {
        Ok(onedrive_output_summary("Sync", &output))
    } else {
        write_onedrive_initial_sync_marker(plan)?;
        let _ = fs::remove_file(onedrive_initial_preview_marker(plan));
        Ok(format!(
            "{} Initial synchronization is now recorded as complete; future Sync Now runs normal OneDrive sync.",
            onedrive_output_summary("Sync", &output)
        ))
    }
}

fn prepare_online_mount_runtime(connection: &Connection) -> Result<(), String> {
    fs::create_dir_all(&connection.local_path).map_err(|error| {
        format!(
            "failed to create mountpoint {}: {error}",
            connection.local_path.display()
        )
    })?;
    fs::create_dir_all(default_runtime_root()).map_err(|error| {
        format!(
            "failed to create runtime directory {}: {error}",
            default_runtime_root().display()
        )
    })?;
    fs::create_dir_all(
        default_cache_root()
            .join("rclone")
            .join(connection.id.to_string()),
    )
    .map_err(|error| format!("failed to create rclone cache directory: {error}"))?;
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
        format!(
            "failed to create mountpoint {}: {error}",
            plan.mountpoint.display()
        )
    })?;
    fs::create_dir_all(&plan.cache_directory).map_err(|error| {
        format!(
            "failed to create onedriver cache directory {}: {error}",
            plan.cache_directory.display()
        )
    })?;
    if let Some(config_directory) = plan.config_file.parent() {
        fs::create_dir_all(config_directory).map_err(|error| {
            format!(
                "failed to create onedriver config directory {}: {error}",
                config_directory.display()
            )
        })?;
    }
    Ok(())
}

fn prepare_offline_mirror_runtime(connection: &Connection) -> Result<(), String> {
    fs::create_dir_all(&connection.local_path).map_err(|error| {
        format!(
            "failed to create mirror directory {}: {error}",
            connection.local_path.display()
        )
    })?;
    fs::create_dir_all(default_work_root()).map_err(|error| {
        format!(
            "failed to create work directory {}: {error}",
            default_work_root().display()
        )
    })?;
    if let ConnectionMode::OfflineMirror(options) = &connection.mode {
        fs::create_dir_all(&options.recovery_directory).map_err(|error| {
            format!(
                "failed to create recovery directory {}: {error}",
                options.recovery_directory.display()
            )
        })?;
    }
    Ok(())
}

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
        format!(
            "failed to create OneDrive sync directory {}: {error}",
            plan.sync_directory.display()
        )
    })?;
    fs::create_dir_all(&plan.config_directory).map_err(|error| {
        format!(
            "failed to create OneDrive config directory {}: {error}",
            plan.config_directory.display()
        )
    })?;
    fs::create_dir_all(&plan.recovery_directory).map_err(|error| {
        format!(
            "failed to create OneDrive recovery directory {}: {error}",
            plan.recovery_directory.display()
        )
    })?;
    Ok(())
}

fn onedrive_auth_files_for_connection(connection_id: ConnectionId) -> OneDriveMirrorAuthFiles {
    let stem = format!("cosmic-mounter-onedrive-auth-{connection_id}");
    OneDriveMirrorAuthFiles {
        auth_url_file: std::env::temp_dir().join(format!("{stem}-url")),
        response_url_file: std::env::temp_dir().join(format!("{stem}-response")),
    }
}

#[allow(dead_code)]
fn onedrive_auth_open_command(auth_files: &OneDriveMirrorAuthFiles) -> String {
    format!("xdg-open \"$(cat {})\"", auth_files.auth_url_file.display())
}

fn validate_onedrive_auth_url(value: &str) -> Result<(), String> {
    validate_setup_value("OneDrive auth URL", value, true)?;
    if value.starts_with("https://login.microsoftonline.com/")
        || value.starts_with("https://login.live.com/")
    {
        Ok(())
    } else {
        Err("The generated OneDrive auth URL is not a recognized Microsoft login URL.".into())
    }
}

fn open_onedrive_auth_url(auth_files: &OneDriveMirrorAuthFiles) -> Result<&'static str, String> {
    let url = fs::read_to_string(&auth_files.auth_url_file).map_err(|error| {
        format!(
            "auth URL file is not ready yet at {}: {error}",
            auth_files.auth_url_file.display()
        )
    })?;
    let url = url.trim();
    validate_onedrive_auth_url(url)?;
    if let Some(helper) = onedrive_auth_helper_path() {
        Command::new(&helper)
            .arg(&auth_files.auth_url_file)
            .arg(&auth_files.response_url_file)
            .spawn()
            .map_err(|error| {
                format!(
                    "failed to start OneDrive auth helper `{}`: {error}",
                    helper.display()
                )
            })?;
        return Ok(
            "Opened the OneDrive WebKit auth helper. Complete Microsoft sign-in there; the helper will capture the final redirect automatically if WebKit permits it.",
        );
    }
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|error| format!("failed to start xdg-open: {error}"))?;
    Ok(
        "Opened the OneDrive authentication page in your browser. If the WebKit helper is unavailable, paste the final native-client URL into the response field.",
    )
}

fn onedrive_auth_helper_path() -> Option<PathBuf> {
    let helper_name = "cosmic-ext-applet-mounter-onedrive-auth-helper";
    if let Ok(current) = env::current_exe()
        && let Some(directory) = current.parent()
    {
        let sibling = directory.join(helper_name);
        if sibling.is_file() {
            return Some(sibling);
        }
    }
    if let Ok(home) = env::var("HOME") {
        let user_install = PathBuf::from(home)
            .join(".local")
            .join("bin")
            .join(helper_name);
        if user_install.is_file() {
            return Some(user_install);
        }
    }
    None
}

fn validate_onedrive_auth_response_url(value: &str) -> Result<(), String> {
    validate_setup_value("OneDrive response URL", value, true)?;
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
            format!(
                "failed to create OneDrive response handoff directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&auth_files.response_url_file)
        .map_err(|error| {
            format!(
                "failed to open response file {}: {error}",
                auth_files.response_url_file.display()
            )
        })?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| {
            format!(
                "failed to restrict response file permissions {}: {error}",
                auth_files.response_url_file.display()
            )
        })?;
    file.write_all(response_url.as_bytes()).map_err(|error| {
        format!(
            "failed to write response file {}: {error}",
            auth_files.response_url_file.display()
        )
    })
}

#[allow(dead_code)]
fn prepare_onedrive_auth_files(auth_files: &OneDriveMirrorAuthFiles) -> Result<(), String> {
    if let Some(parent) = auth_files.auth_url_file.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create OneDrive auth handoff directory {}: {error}",
                parent.display()
            )
        })?;
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

async fn rclone_mirror_pending_count(connection: &Connection) -> Result<u64, String> {
    let plan = rclone_bisync_plan(connection, &default_work_root())
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    if !plan.filters_file.is_file() {
        return Err(fl!("pending-count-rclone-mirror-unavailable"));
    }
    let initialized = initial_sync_marker(&plan).exists();
    let request = if initialized {
        rclone_bisync_preview_request(&plan)
    } else {
        rclone_bisync_initial_preview_request(&plan)
    }
    .map_err(|_| fl!("pending-count-request-failed"))?;
    let request = request
        .arg("--color")
        .and_then(|request| request.arg("NEVER"))
        .map_err(|_| fl!("pending-count-request-failed"))?;
    let output = app_command_runner()
        .run(request, CancellationToken::new())
        .await
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    if output.stdout.truncated
        || output.stderr.truncated
        || output.stdout.invalid_utf8
        || output.stderr.invalid_utf8
    {
        return Err(fl!("pending-count-response-incomplete"));
    }
    pending_count::parse_rclone_mirror_status(&format!(
        "{}\n{}",
        output.stdout.text, output.stderr.text
    ))
}

async fn rclone_mirror_live_pending_count(connection: &Connection) -> Result<u64, String> {
    let service = UnitName::new(connection.id, UnitKind::Service).file_name();
    pending_count::rclone_mirror_live_pending(&app_command_runner(), &service).await
}

fn prepare_rclone_bisync_work_files(
    connection: &Connection,
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<(), String> {
    fs::create_dir_all(&plan.work_directory).map_err(|error| {
        format!(
            "failed to create rclone bisync work directory {}: {error}",
            plan.work_directory.display()
        )
    })?;
    let filter_content = rclone_bisync_filter_file(connection.provider);
    fs::write(&plan.filters_file, filter_content).map_err(|error| {
        format!(
            "failed to write rclone bisync filter file {}: {error}",
            plan.filters_file.display()
        )
    })?;
    let script_path = mirror_script::script_path(plan);
    if script_path.exists() {
        let existing = fs::read_to_string(&script_path)
            .map_err(|error| format!("could not inspect managed mirror script: {error}"))?;
        let owner_line = format!("# Cloud Mounter managed bisync script: '{}'", connection.id);
        if !existing.lines().any(|line| line == owner_line) {
            return Err("mirror script path contains an unowned file".into());
        }
    }
    let content = mirror_script::render(plan).map_err(|error| error.to_string())?;
    let staging = script_path.with_extension(format!("sh.tmp.{}", Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(|error| format!("could not stage managed mirror script: {error}"))?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("could not restrict managed mirror script: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("could not write managed mirror script: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("could not sync managed mirror script: {error}"))?;
    fs::rename(&staging, &script_path)
        .map_err(|error| format!("could not install managed mirror script: {error}"))?;
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
    .map_err(|error| format!("failed to record OneDrive initial preview completion: {error}"))
}

fn write_onedrive_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<(), String> {
    fs::write(
        onedrive_initial_sync_marker(plan),
        "Cloud Mounter OneDrive initial sync completed.\n",
    )
    .map_err(|error| format!("failed to record OneDrive initial sync completion: {error}"))
}

fn preview_summary_text(output: &CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.text, output.stderr.text);
    let summary = parse_preview(&combined);
    format!(
        "Preview: uploads {}, downloads {}, deletes {}, conflicts {}, skipped {}{}{}.",
        summary.uploads,
        summary.downloads,
        summary.deletes,
        summary.conflicts,
        summary.skipped,
        summary
            .transfer_bytes
            .map(|bytes| format!(", transfer estimate {}", human_bytes(bytes)))
            .unwrap_or_default(),
        if summary.destructive {
            " (destructive changes detected)"
        } else {
            ""
        }
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
        format!("{stage} completed; onedrive did not report file-change counts.")
    } else {
        format!("{stage} completed; onedrive reported {changed_lines} notable file-change line(s).")
    }
}

fn sync_output_summary(output: &CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.text, output.stderr.text);
    let summary = parse_preview(&combined);
    if summary.uploads + summary.downloads + summary.deletes + summary.conflicts + summary.skipped
        > 0
    {
        format!(
            "Sync summary: uploads {}, downloads {}, deletes {}, conflicts {}, skipped {}.",
            summary.uploads, summary.downloads, summary.deletes, summary.conflicts, summary.skipped
        )
    } else {
        "Sync completed; rclone did not report transfer counts.".into()
    }
}

fn human_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

fn sync_rejection_message(rejection: SyncDecisionRejection) -> &'static str {
    match rejection {
        SyncDecisionRejection::ConcurrentRun => "a synchronization is already running",
        SyncDecisionRejection::PreviewRequired => {
            "initial synchronization requires Preview first; then press Sync Now to confirm"
        }
        SyncDecisionRejection::ConfirmationRequired => {
            "initial synchronization requires explicit confirmation through Sync Now"
        }
        SyncDecisionRejection::ResyncPreviewRequired => {
            "state rebuild or resync requires preview and confirmation"
        }
    }
}

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
            if detail.contains("prior lock file found") {
                "Sync is locked. Retry after the active run or two minutes; older locks may need manual recovery.".into()
            } else if detail.is_empty() {
                format!("rclone {stage} failed without diagnostic output")
            } else {
                format!("rclone {stage} failed: {detail}")
            }
        }
    }
}

fn onedrive_runtime_command_error(stage: &str, error: CommandError) -> String {
    onedrive_validation_command_error(stage, error)
}

async fn test_teams_draft_connection(connection: &mut Connection) -> Result<String, String> {
    let verified = teams::verify_remote(
        &app_command_runner(),
        &connection.remote_reference,
        connection
            .teams_identity
            .as_ref()
            .ok_or("SharePoint library URL is missing.")?,
    )
    .await?;
    connection.teams_identity = Some(verified.identity.clone());
    test_connection_plan_and_access(connection, Some(&verified)).await
}

async fn test_connection_plan_and_access(
    connection: &Connection,
    teams_verified: Option<&teams::VerifiedTeamsLibrary>,
) -> Result<String, String> {
    let plan_summary = match managed_plan_summary(connection) {
        Ok(summary) => summary,
        Err(error) => return Err(error),
    };

    ensure_vpn_ready_for_connection(connection).await?;

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
                match verify_onedriver_online_mount_setup(connection) {
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
    match connection.mode {
        ConnectionMode::OnlineMount(_) => verify_onedriver_online_mount_setup(connection),
        ConnectionMode::OfflineMirror(_) => verify_onedrive_offline_mirror_setup(connection).await,
    }
}

fn save_notice_name(name: &str, validation_summary: Option<&str>) -> String {
    match validation_summary {
        Some(summary) => format!("{name} passed validation ({summary}) and"),
        None => name.to_owned(),
    }
}

fn verify_onedriver_online_mount_setup(connection: &Connection) -> Result<String, String> {
    verify_onedriver_online_mount_setup_with(
        connection,
        &app_command_runner(),
        &HostVisibleMountTable,
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
        return Err(
            "OneDrive Online Mount validation only applies to onedriver connections".into(),
        );
    }
    if runner.resolve(Executable::Onedriver).is_none() {
        return Err(
            "onedriver is missing. Install jstaf/onedriver 0.15.0 or newer before testing this OneDrive Online Mount."
                .into(),
        );
    }

    let plan = onedriver_mount_plan(connection, cache_root, config_root)
        .map_err(|error| format!("could not build onedriver plan: {error}"))?;
    if let Some(parent) = plan.mountpoint.parent()
        && parent.exists()
        && !parent.is_dir()
    {
        return Err(format!(
            "mountpoint parent `{}` is not a directory.",
            parent.display()
        ));
    }
    if plan.mountpoint.exists() && !plan.mountpoint.is_dir() {
        return Err(format!(
            "mountpoint `{}` exists but is not a directory. Choose an empty directory or remove the file first.",
            plan.mountpoint.display()
        ));
    }

    let mount_entries = mount_table
        .entries()
        .map_err(|error| format!("could not inspect active mounts: {error}"))?;
    if let Some(entry) = conflicting_onedriver_mount(&plan.mountpoint, &mount_entries) {
        return Err(format!(
            "mountpoint `{}` overlaps an active onedriver mount at `{}`. Unmount the existing OneDrive mount or choose a separate mountpoint.",
            plan.mountpoint.display(),
            entry.target.display()
        ));
    }

    match onedriver_auth_state_for_plan(&plan) {
        OnedriverAuthState::Unauthenticated => {
            return Err(format!(
                "onedriver is not authenticated for this applet-owned connection. Start OneDrive setup for `{}` so onedriver can create app-owned auth metadata under `{}`.",
                connection.name,
                plan.cache_directory.display()
            ));
        }
        OnedriverAuthState::Authenticated { config_file } => {
            if config_file
                .metadata()
                .map(|metadata| metadata.len() == 0)
                .unwrap_or(true)
            {
                return Err(format!(
                    "onedriver account metadata at `{}` is unavailable or empty. Reauthorize this OneDrive setup before mounting.",
                    config_file.display()
                ));
            }
        }
    }

    Ok(format!(
        "onedriver setup is present for mountpoint `{}`. Cache directory `{}` will be created if needed.",
        plan.mountpoint.display(),
        plan.cache_directory.display()
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
    verify_onedrive_offline_mirror_setup_with(
        connection,
        &app_command_runner(),
        &HostVisibleMountTable,
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
    if !is_onedrive_offline_mirror(connection) {
        return Err(
            "OneDrive Offline Mirror validation only applies to abraunegg/onedrive connections."
                .into(),
        );
    }
    if runner.resolve(Executable::OneDrive).is_none() {
        return Err(
            "onedrive is missing. Install abraunegg/onedrive 2.5.10 or newer before testing this OneDrive Offline Mirror."
                .into(),
        );
    }

    let mount_entries = mount_table
        .entries()
        .map_err(|error| format!("could not inspect active mounts: {error}"))?;
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

    validate_directory_available("sync directory", &plan.sync_directory)?;
    validate_directory_available("OneDrive config directory", &plan.config_directory)?;
    validate_directory_available("recovery directory", &plan.recovery_directory)?;

    let token_file = plan.config_directory.join("refresh_token");
    if token_file
        .metadata()
        .map(|metadata| metadata.len() == 0)
        .unwrap_or(true)
    {
        return Err(format!(
            "abraunegg/onedrive is not authenticated for this applet-owned mirror. Authorize this connection so onedrive can create `{}`.",
            token_file.display()
        ));
    }

    let output = runner
        .run(
            one_drive_preview_request(&plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| onedrive_validation_command_error("dry-run preview", error))?;

    Ok(format!(
        "abraunegg/onedrive setup is authenticated for sync directory `{}`. {}",
        plan.sync_directory.display(),
        onedrive_output_summary("Validation preview", &output)
    ))
}

fn validate_directory_available(label: &str, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && parent.exists()
        && !parent.is_dir()
    {
        return Err(format!(
            "{label} parent `{}` is not a directory.",
            parent.display()
        ));
    }
    if path.exists() && !path.is_dir() {
        return Err(format!(
            "{label} `{}` exists but is not a directory.",
            path.display()
        ));
    }
    Ok(())
}

fn onedrive_mirror_plan_validation_error(
    error: cosmic_ext_applet_mounter::sync::SyncError,
) -> String {
    match error {
        cosmic_ext_applet_mounter::sync::SyncError::OverlapsOnedriverPath(path) => format!(
            "OneDrive Offline Mirror overlaps active onedriver mount `{}`. Unmount the Online Mount or choose a separate local mirror directory.",
            path.display()
        ),
        cosmic_ext_applet_mounter::sync::SyncError::InvalidWorkDirectory => {
            "OneDrive config/work directory must be absolute and separate from the local mirror directory.".into()
        }
        cosmic_ext_applet_mounter::sync::SyncError::InvalidRecoveryDirectory => {
            "recovery directory must be absolute and separate from both the local mirror and applet config/work directories.".into()
        }
        cosmic_ext_applet_mounter::sync::SyncError::InvalidRemoteSubpath => {
            "remote subtree contains unsupported characters.".into()
        }
        other => other.to_string(),
    }
}

fn onedrive_validation_command_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        CommandError::InvalidArgument => {
            "onedrive command argument contains unsupported characters".into()
        }
        CommandError::Timeout { timeout, .. } => format!(
            "onedrive {stage} timed out after {} seconds. Authentication may still be valid, but the initial dry-run can take several minutes for a whole drive. Check network/VPN readiness, Microsoft OneDrive responsiveness, and consider selecting a smaller remote subtree for first validation.",
            timeout.as_secs()
        ),
        CommandError::Cancelled { .. } => format!("onedrive {stage} was cancelled"),
        CommandError::Spawn { message, .. } => format!("could not start onedrive: {message}"),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                stdout.text.trim()
            } else {
                stderr.text.trim()
            };
            let lower = detail.to_ascii_lowercase();
            if detail.is_empty() {
                format!("onedrive {stage} failed without diagnostic output")
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
                format!("abraunegg/onedrive is not authenticated for this mirror: {detail}")
            } else if lower.contains("code has expired")
                || lower.contains("code is not valid")
                || lower.contains("invalid_grant")
                || lower.contains("invalid_request")
                || lower.contains("auth-response")
            {
                format!(
                    "OneDrive OAuth response is expired or invalid. Restart authorization and submit the final nativeclient URL immediately: {detail}"
                )
            } else if lower.contains("aadsts")
                || lower.contains("admin consent")
                || lower.contains("tenant")
                || lower.contains("consent")
            {
                format!(
                    "Microsoft tenant or admin-consent policy blocked OneDrive authorization: {detail}"
                )
            } else if lower.contains("resync")
                || lower.contains("application configuration change")
                || lower.contains("sync state")
            {
                format!(
                    "abraunegg/onedrive requires a preview and explicit resync/state rebuild before syncing: {detail}"
                )
            } else if lower.contains("does not exist online")
                || lower.contains("requested path")
                || lower.contains("single-directory")
            {
                format!("selected OneDrive remote subtree is not accessible: {detail}")
            } else if lower.contains("network")
                || lower.contains("timeout")
                || lower.contains("connection")
                || lower.contains("dns")
                || lower.contains("host")
            {
                format!("network or VPN readiness failed while checking OneDrive: {detail}")
            } else {
                format!("onedrive {stage} failed: {detail}")
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
    teams_verified: Option<&teams::VerifiedTeamsLibrary>,
) -> Result<String, String> {
    let teams_summary = if connection.provider == Provider::Teams {
        let verified = match teams_verified {
            Some(verified) => verified.clone(),
            None => teams::verify_remote(
                &app_command_runner(),
                &connection.remote_reference,
                connection.teams_identity.as_ref().ok_or(
                    "SharePoint library identity is missing; edit and save the connection again.",
                )?,
            )
            .await?,
        };
        let account = verified
            .account
            .unwrap_or_else(|| "account unavailable from Microsoft Graph".into());
        Some(format!(
            "SharePoint library {} at site {} and drive {} matches the saved identity. Signed-in account: {account}.",
            verified.identity.library_url, verified.identity.site_url, verified.identity.drive_id
        ))
    } else {
        None
    };
    if connection.provider == Provider::Sftp {
        sftp::verify_host_verification(&app_command_runner(), &connection.remote_reference).await?;
    }
    let expected_backend = rclone_backend_name(connection.provider)
        .ok_or_else(|| "selected provider does not use rclone".to_owned())?;
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
                teams::rclone_read_error(&error, connection.remote_subpath.is_some())
            } else {
                rclone_access_error(&target, error)
            }
        })?;
    let visible_items = output.stdout.text.lines().count();
    Ok(format!(
        "Rclone remote `{}` exists, backend `{expected_backend}` matches {}, and `{target}` is accessible with {visible_items} visible item(s) at depth 1.",
        connection.remote_reference,
        provider_label(connection.provider)
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
        ProviderError::InvalidRemoteReference => format!(
            "rclone remote `{}` was not found. Run Detect rclone remotes, choose a listed remote, or create the remote before testing.",
            connection.remote_reference
        ),
        ProviderError::UnsupportedProvider(_) => format!(
            "rclone remote `{}` exists but does not use the expected `{}` backend for {}.",
            connection.remote_reference,
            rclone_backend_name(connection.provider).unwrap_or("unknown"),
            provider_label(connection.provider)
        ),
        ProviderError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        ProviderError::Command(error) => rclone_access_error("rclone config dump", error),
        ProviderError::InvalidResponse(message) => {
            format!("rclone config dump returned an unexpected response: {message}")
        }
        ProviderError::InvalidMode => "selected connection mode is invalid for rclone".into(),
        ProviderError::InvalidRemoteSubpath => {
            "remote subtree contains unsupported characters".into()
        }
        ProviderError::InvalidTeamsIdentity(message) => message,
        ProviderError::InvalidMountpointGuard(message) => message,
        ProviderError::Unauthenticated => {
            "rclone remote is not authenticated. Reauthorize it before testing.".into()
        }
    }
}

fn rclone_access_error(target: &str, error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(executable) => {
            format!("{} is missing.", executable.display_name())
        }
        CommandError::InvalidArgument => {
            "rclone command argument contains unsupported characters".into()
        }
        CommandError::Timeout { timeout, .. } => format!(
            "read-only rclone access check for `{target}` timed out after {} seconds. Check network/VPN readiness and provider responsiveness.",
            timeout.as_secs()
        ),
        CommandError::Cancelled { .. } => "rclone access check was cancelled".into(),
        CommandError::Spawn { message, .. } => format!("could not start rclone: {message}"),
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
                format!("remote subtree `{target}` is not accessible or does not exist: {detail}")
            } else if lower.contains("auth")
                || lower.contains("token")
                || lower.contains("unauthorized")
                || lower.contains("forbidden")
                || lower.contains("permission")
            {
                format!("rclone remote for `{target}` is not authorized for this access: {detail}")
            } else if lower.contains("network")
                || lower.contains("connection")
                || lower.contains("timeout")
                || lower.contains("no route")
                || lower.contains("host")
                || lower.contains("dns")
            {
                format!("network or VPN readiness failed while checking `{target}`: {detail}")
            } else {
                format!("read-only rclone access check failed for `{target}`: {detail}")
            }
        }
    }
}

fn sftp_access_error(error: CommandError) -> String {
    // Rclone diagnostics may include paths or credential-bearing option text.
    // Match known signatures, but never display the raw command output.
    match error {
        CommandError::MissingExecutable(_) => {
            "rclone is missing; install it before testing SFTP access.".into()
        }
        CommandError::InvalidArgument => {
            "SFTP access request contains unsupported characters.".into()
        }
        CommandError::Timeout { .. } => {
            "SFTP access timed out. Check the server address, port, network, and VPN.".into()
        }
        CommandError::Cancelled { .. } => "SFTP access check was cancelled.".into(),
        CommandError::Spawn { .. } => "Could not start rclone for the SFTP access check.".into(),
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
                "SFTP known-hosts file is missing, unreadable, or invalid. Check its host path and permissions.".into()
            } else if lower.contains("knownhosts:")
                || lower.contains("host key")
                || lower.contains("hostkey")
                || lower.contains("remote host identification has changed")
                || lower.contains("no authorities for hostname")
            {
                "SFTP host-key verification failed. Verify the server identity and its entry in the known-hosts file; do not accept an unexpected key.".into()
            } else if (lower.contains("key_file") || lower.contains("private key"))
                && (lower.contains("no such file")
                    || lower.contains("permission denied")
                    || lower.contains("cannot read")
                    || lower.contains("failed to load"))
            {
                "SFTP private-key file is missing or unreadable. Check its host path and permissions.".into()
            } else if lower.contains("ssh_auth_sock")
                || lower.contains("ssh agent")
                || lower.contains("ssh-agent")
            {
                "SFTP SSH agent is unavailable or has no usable unlocked key. Check SSH_AUTH_SOCK and loaded identities.".into()
            } else if lower.contains("i/o timeout")
                || lower.contains("connection timed out")
                || lower.contains("context deadline exceeded")
            {
                "SFTP connection timed out. Check the server response, port, network, and VPN."
                    .into()
            } else if lower.contains("unable to authenticate")
                || lower.contains("authentication failed")
                || lower.contains("no supported methods remain")
                || lower.contains("too many authentication failures")
                || (lower.contains("handshake") && lower.contains("permission denied"))
            {
                "SFTP authentication failed. Check the username and password, private key, or SSH-agent identity.".into()
            } else if lower.contains("permission denied")
                || lower.contains("access denied")
                || lower.contains("operation not permitted")
            {
                "SFTP login succeeded, but the selected directory is not readable. Check server permissions.".into()
            } else if lower.contains("directory not found")
                || lower.contains("object not found")
                || lower.contains("path not found")
                || lower.contains("no such file")
                || lower.contains("not a directory")
                || lower.contains("doesn't exist")
            {
                "SFTP directory was not found. Check the remote directory and whether it is relative to the login directory or starts at server root.".into()
            } else if lower.contains("connection refused")
                || lower.contains("no route to host")
                || lower.contains("network is unreachable")
                || lower.contains("no such host")
                || lower.contains("dial tcp")
            {
                "SFTP server could not be reached. Check its address, port, network, and VPN."
                    .into()
            } else {
                "SFTP access check failed. Check the server settings and rclone diagnostics.".into()
            }
        }
    }
}

fn import_preview_summary(preview: &ImportPreview) -> String {
    let status = if preview.active_conflict || preview.local_target_conflict {
        "blocked"
    } else {
        "ready"
    };
    let unsupported = if preview.unsupported_options.is_empty() {
        "none".into()
    } else {
        preview.unsupported_options.join(", ")
    };
    let subtree =
        preview
            .remote_subpath
            .as_deref()
            .unwrap_or(if preview.provider == Provider::Sftp {
                "SFTP login directory"
            } else {
                "Whole remote"
            });
    format!(
        "{} {} -> {}\nRemote subtree: {}\nStart at login: {}\nUnsupported options: {unsupported}\nStatus: {status}. Import replacement still requires explicit confirmation.",
        provider_label(preview.provider),
        preview.remote_reference,
        preview.local_target.display(),
        subtree,
        yes_no(preview.start_at_login),
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
    use cosmic_ext_applet_mounter::config::HostVisibleConfigStorage;
    use cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile;

    #[tokio::test]
    async fn refresh_completion_preserves_operation_notice_and_pending_action() {
        use cosmic::Application;
        let connection_id = ConnectionId::from_uuid(Uuid::new_v4());
        let mut app = AppModel {
            last_notice: Some("Mount failed: access denied".into()),
            pending_repair: Some(connection_id),
            ..AppModel::default()
        };
        let (reply, receiver) = runtime_ipc::Reply::channel();
        let _ = app.update(Message::RuntimeRefreshCompleted(reply, BTreeMap::new()));
        assert!(receiver.await.unwrap().is_ok());
        assert_eq!(
            app.last_notice.as_deref(),
            Some("Mount failed: access denied")
        );
        assert_eq!(app.pending_repair, Some(connection_id));
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
        assert!(!sleep_needs_attention(None));
        assert!(!sleep_needs_attention(Some(
            "Online mount cleanup before sleep is ready"
        )));
        assert!(sleep_needs_attention(Some(
            "Sleep cleanup unavailable: no inhibitor"
        )));
        assert!(sleep_needs_attention(Some(
            "Sleep cleanup incomplete (0 unmounted): busy"
        )));
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
    fn runtime_cisco_tunnel_state_reads_exact_connection_state() {
        assert_eq!(
            runtime_cisco_tunnel_state("Connection State:            Connected\n"),
            CiscoTunnelState::Connected
        );
        assert_eq!(
            runtime_cisco_tunnel_state("Connection State:            Disconnected\n"),
            CiscoTunnelState::Disconnected
        );
        assert_eq!(
            runtime_cisco_tunnel_state("Connection State:            Not Available\n"),
            CiscoTunnelState::Disconnected
        );
        assert_eq!(
            runtime_cisco_tunnel_state(
                "Cannot contact the VPN service.\nConnection State:            Not Available\n"
            ),
            CiscoTunnelState::ServiceUnavailable
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
    fn popup_error_online_mount_uses_repair_as_primary_action() {
        let row = ConnectionRowState {
            id: ConnectionId::from_uuid(
                Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
            ),
            name: "OneDrive online mount test".into(),
            provider: Provider::OneDrive,
            mode: AccessMode::OnlineMount,
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

        assert_eq!(primary_operation(&row), Some(Operation::Repair));
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
    fn repair_skips_an_already_detached_mount_and_rejects_foreign_filesystems() {
        let connection = test_connection(Provider::Sftp);
        assert!(!repair_needs_lazy_detach(&connection, &[]).unwrap());
        let mut mount = cosmic_ext_applet_mounter::mounts::MountEntry {
            target: connection.local_path.clone(),
            source: "rclone".into(),
            filesystem: "fuse.rclone".into(),
            options: Vec::new(),
        };
        assert!(repair_needs_lazy_detach(&connection, &[mount.clone()]).unwrap());
        mount.filesystem = "ext4".into();
        assert!(repair_needs_lazy_detach(&connection, &[mount]).is_err());
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
        let manager = CommandSystemdManager::new(SystemCommandRunner);
        let controller = UnitController::new(store, manager);
        controller
            .install(&document, CancellationToken::new())
            .await
            .unwrap();
        let manager = CommandSystemdManager::new(SystemCommandRunner);
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
        run_managed_online_mount_operation_result_inner(&connection, Operation::Unmount)
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

    #[tokio::test]
    #[ignore = "requires disposable localhost SFTP, host rclone, and user systemd"]
    async fn live_sftp_offline_mirror_controls_and_recovery() {
        let root = PathBuf::from(std::env::var("COSMIC_SFTP_MIRROR_ROOT").expect("probe root"));
        let remote_data = root.join("data");
        let mut connection = test_connection(Provider::Sftp);
        connection.id = ConnectionId::new();
        connection.name = "Disposable SFTP mirror".into();
        connection.remote_reference = "probe".into();
        connection.remote_subpath = Some("share".into());
        connection.local_path = root.join("local");
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: root.join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: true,
        });
        let plan = rclone_bisync_plan(&connection, &default_work_root()).unwrap();

        let missing_preview =
            run_managed_offline_mirror_operation_result(&connection, Operation::SyncNow)
                .await
                .expect_err("initial sync needs a preview");
        assert!(missing_preview.contains("Preview first"));
        let preview =
            run_managed_offline_mirror_operation_result(&connection, Operation::PreviewInitialSync)
                .await
                .unwrap();
        assert!(preview.contains("press Sync Now"));
        assert!(initial_preview_marker(&plan).exists());
        assert!(!connection.local_path.join("remote-only.txt").exists());
        run_managed_offline_mirror_operation_result(&connection, Operation::SyncNow)
            .await
            .unwrap();
        assert!(initial_sync_marker(&plan).exists());
        assert!(!initial_preview_marker(&plan).exists());
        assert_eq!(
            fs::read_to_string(connection.local_path.join("remote-only.txt")).unwrap(),
            "remote\n"
        );
        fs::write(connection.local_path.join("local-only.txt"), "local\n").unwrap();
        run_managed_offline_mirror_operation_result(&connection, Operation::PreviewInitialSync)
            .await
            .unwrap();
        assert!(!remote_data.join("share/local-only.txt").exists());
        run_managed_offline_mirror_operation_result(&connection, Operation::SyncNow)
            .await
            .unwrap();
        assert_eq!(
            fs::read_to_string(remote_data.join("share/local-only.txt")).unwrap(),
            "local\n"
        );

        let copy_remote = |source: &Path, destination: &str| {
            let output = Command::new("rclone")
                .arg("copyto")
                .arg(source)
                .arg(destination)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        let remote_recovery = remote_data
            .join(".cosmic-mounter-recovery")
            .join(connection.id.to_string());
        let old_remote = remote_recovery.join("2000-01-01");
        let unowned_remote = remote_recovery.join("2000-01-02");
        let old_local = root.join("recovery/2000-01-01");
        let unowned_local = root.join("recovery/2000-01-02");
        for directory in [&old_local, &unowned_local] {
            fs::create_dir_all(directory).unwrap();
            fs::write(directory.join("old.txt"), "old\n").unwrap();
        }
        fs::write(
            old_local.join(".cosmic-mounter-owner"),
            connection.id.to_string(),
        )
        .unwrap();
        for day in ["2000-01-01", "2000-01-02"] {
            copy_remote(
                &root.join("recovery").join(day).join("old.txt"),
                &format!("{}/{day}/old.txt", plan.remote_recovery_path),
            );
        }
        copy_remote(
            &old_local.join(".cosmic-mounter-owner"),
            &format!(
                "{}/2000-01-01/.cosmic-mounter-owner",
                plan.remote_recovery_path
            ),
        );
        let delete_source = root.join("delete-source.txt");
        fs::write(&delete_source, "delete\n").unwrap();
        copy_remote(&delete_source, "probe:share/delete-me.txt");
        run_managed_offline_mirror_operation_result(&connection, Operation::SyncNow)
            .await
            .unwrap();
        assert!(connection.local_path.join("delete-me.txt").exists());
        let delete = Command::new("rclone")
            .args(["deletefile", "probe:share/delete-me.txt"])
            .output()
            .unwrap();
        assert!(delete.status.success());
        run_managed_offline_mirror_operation_result(&connection, Operation::SyncNow)
            .await
            .unwrap();
        assert!(!connection.local_path.join("delete-me.txt").exists());
        assert!(!old_remote.exists());
        assert!(!old_local.exists());
        assert!(unowned_remote.exists());
        assert!(unowned_local.exists());
        assert!(
            fs::read_dir(root.join("recovery"))
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.path().is_dir() && entry.path() != unowned_local)
        );

        use cosmic_ext_applet_mounter::services::UnitStore;
        use std::os::unix::fs::PermissionsExt;

        struct MirrorProbeGuard {
            service: UnitName,
            timer: UnitName,
            service_dropin: PathBuf,
            timer_dropin: PathBuf,
        }
        impl Drop for MirrorProbeGuard {
            fn drop(&mut self) {
                let timer = self.timer.file_name();
                let service = self.service.file_name();
                let _ = Command::new("systemctl")
                    .args(["--user", "disable", "--now", &timer])
                    .output();
                let _ = Command::new("systemctl")
                    .args(["--user", "stop", &service])
                    .output();
                for path in [&self.service_dropin, &self.timer_dropin] {
                    let _ = fs::remove_file(path);
                    if let Some(parent) = path.parent() {
                        let _ = fs::remove_dir(parent);
                    }
                }
                if let Ok(store) = FileUnitStore::user(Arc::new(StructuralUnitValidator)) {
                    let _ = store.remove(&self.timer);
                    let _ = store.remove(&self.service);
                }
                let _ = Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .output();
            }
        }

        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: root.join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        let runtime = PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap());
        let service = UnitName::new(connection.id, UnitKind::Service);
        let timer = UnitName::new(connection.id, UnitKind::Timer);
        let service_dropin = runtime
            .join("systemd/user")
            .join(format!("{}.d/90-mirror-probe.conf", service.file_name()));
        let timer_dropin = runtime
            .join("systemd/user")
            .join(format!("{}.d/90-mirror-probe.conf", timer.file_name()));
        let _guard = MirrorProbeGuard {
            service: service.clone(),
            timer: timer.clone(),
            service_dropin: service_dropin.clone(),
            timer_dropin: timer_dropin.clone(),
        };
        install_rclone_offline_mirror_units_result(&connection)
            .await
            .unwrap();
        let store = FileUnitStore::user(Arc::new(StructuralUnitValidator)).unwrap();
        let service_content = store.read(&service).unwrap().unwrap();
        let timer_content = store.read(&timer).unwrap().unwrap();
        assert!(service_content.contains("ExecCondition="));
        assert!(service_content.contains("managed-bisync.sh"));
        assert!(timer_content.contains("OnUnitInactiveSec=900s"));
        for unit in [service.file_name(), timer.file_name()] {
            let host_unit = Command::new("flatpak-builder")
                .args([
                    "--run",
                    "target/flatpak-gui-prototype",
                    "packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json",
                    "flatpak-spawn",
                    "--host",
                    "/usr/bin/systemctl",
                    "--user",
                    "cat",
                    &unit,
                ])
                .output()
                .unwrap();
            assert!(
                host_unit.status.success(),
                "{}",
                String::from_utf8_lossy(&host_unit.stderr)
            );
            assert!(String::from_utf8_lossy(&host_unit.stdout).contains(&unit));
        }

        let fake_bin = root.join("fake-bin");
        fs::create_dir_all(&fake_bin).unwrap();
        let fake_nmcli = fake_bin.join("nmcli");
        let metered_flag = root.join("metered-state");
        fs::write(&metered_flag, "no\n").unwrap();
        fs::write(
            &fake_nmcli,
            format!(
                "#!/bin/sh\nprintf 'GENERAL.METERED:'\ncat '{}'\n",
                metered_flag.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&fake_nmcli, fs::Permissions::from_mode(0o755)).unwrap();
        fs::create_dir_all(service_dropin.parent().unwrap()).unwrap();
        fs::create_dir_all(timer_dropin.parent().unwrap()).unwrap();
        let rclone_config = std::env::var("RCLONE_CONFIG").unwrap();
        fs::write(
            &service_dropin,
            format!(
                "[Service]\nEnvironment=RCLONE_CONFIG={rclone_config}\nEnvironment=PATH={}:{}\n",
                fake_bin.display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .unwrap();
        fs::write(
            &timer_dropin,
            "[Timer]\nOnUnitInactiveSec=\nOnActiveSec=3s\n",
        )
        .unwrap();
        let reload = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()
            .unwrap();
        assert!(reload.status.success());

        fs::write(connection.local_path.join("timer-only.txt"), "timer\n").unwrap();
        run_managed_offline_mirror_operation_result(&connection, Operation::ResumeSync)
            .await
            .unwrap();
        for _ in 0..600 {
            if remote_data.join("share/timer-only.txt").exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if !remote_data.join("share/timer-only.txt").exists() {
            for unit in [timer.file_name(), service.file_name()] {
                let status = Command::new("systemctl")
                    .args(["--user", "status", &unit, "--no-pager"])
                    .output()
                    .unwrap();
                println!("{}", String::from_utf8_lossy(&status.stdout));
                let journal = Command::new("journalctl")
                    .args(["--user", "-u", &unit, "-n", "25", "--no-pager"])
                    .output()
                    .unwrap();
                println!("{}", String::from_utf8_lossy(&journal.stdout));
            }
        }
        assert_eq!(
            fs::read_to_string(remote_data.join("share/timer-only.txt")).unwrap(),
            "timer\n"
        );
        run_managed_offline_mirror_operation_result(&connection, Operation::PauseSync)
            .await
            .unwrap();
        fs::write(&metered_flag, "yes\n").unwrap();
        fs::write(connection.local_path.join("metered-only.txt"), "metered\n").unwrap();
        let guarded_start = Command::new("systemctl")
            .args(["--user", "start", &service.file_name()])
            .output()
            .unwrap();
        assert!(guarded_start.status.success());
        let journal = Command::new("journalctl")
            .args([
                "--user",
                "-u",
                &service.file_name(),
                "-n",
                "8",
                "--no-pager",
            ])
            .output()
            .unwrap();
        assert!(journal.status.success());
        assert!(
            String::from_utf8_lossy(&journal.stdout).contains("Skipped due to 'exec-condition'")
        );
        assert!(!remote_data.join("share/metered-only.txt").exists());
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
    fn detected_network_manager_profiles_use_uuid_identity() {
        let profile = network_manager_profile(NetworkManagerVpnProfile {
            name: "Work".into(),
            uuid: "91a601dd-2df4-4b32-bc66-25a16a7612fe".into(),
            vpn_type: "wireguard".into(),
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
        });
        let second = network_manager_profile(NetworkManagerVpnProfile {
            name: "Renamed Work".into(),
            uuid: "91a601dd-2df4-4b32-bc66-25a16a7612fe".into(),
            vpn_type: "wireguard".into(),
        });
        let other = network_manager_profile(NetworkManagerVpnProfile {
            name: "Other".into(),
            uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            vpn_type: "wireguard".into(),
        });

        assert!(same_vpn_reference(&first, &second));
        assert!(!same_vpn_reference(&first, &other));

        first.kind = VpnKind::Cisco;
        assert!(!same_vpn_reference(&first, &second));
        assert!(same_vpn_reference(&cisco_profile(), &cisco_profile()));
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
                },
                RcloneDraftRemote {
                    name: "ua_gdrive".into(),
                    backend: "drive".into(),
                },
                RcloneDraftRemote {
                    name: "work_teams".into(),
                    backend: "onedrive".into(),
                },
                RcloneDraftRemote {
                    name: "ua_engr".into(),
                    backend: "smb".into(),
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
                },
                RcloneDraftRemote {
                    name: "drive".into(),
                    backend: "drive".into(),
                },
                RcloneDraftRemote {
                    name: "share".into(),
                    backend: "smb".into(),
                },
            ],
            ..AppModel::default()
        };

        assert_eq!(
            app.matching_rclone_remotes(Provider::Box),
            vec![RcloneDraftRemote {
                name: "box".into(),
                backend: "box".into(),
            }]
        );
        assert_eq!(
            app.matching_rclone_remotes(Provider::GoogleDrive),
            vec![RcloneDraftRemote {
                name: "drive".into(),
                backend: "drive".into(),
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
            teams_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
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
            draft.teams_library_url
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
    async fn sharepoint_manual_mirror_rejects_background_schedule() {
        let temporary = tempfile::tempdir().unwrap();
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: temporary.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = temporary.path().join("local");
        connection.remote_subpath = Some("Disposable".into());
        connection.teams_identity = Some(cosmic_ext_applet_mounter::model::TeamsLibraryIdentity {
            site_url: "https://example.sharepoint.com/sites/Work".into(),
            library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
            drive_id: "b!verified".into(),
        });
        let plan = rclone_bisync_plan(&connection, &temporary.path().join("work")).unwrap();
        assert!(
            start_rclone_offline_mirror_background(&connection, &plan)
                .await
                .unwrap_err()
                .contains("not available yet")
        );
    }

    #[test]
    fn teams_test_uses_verified_drive_and_discards_changed_drafts() {
        let draft = ConnectionDraft {
            name: "Work Teams".into(),
            provider: Provider::Teams,
            remote_reference: "work_teams".into(),
            teams_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
                .into(),
            local_path: "/home/example/Cloud/Work Teams".into(),
            ..ConnectionDraft::default()
        };
        let mut verified = connection_from_draft(&draft).unwrap();
        assert!(managed_plan_summary(&verified).is_err());
        verified.teams_identity.as_mut().unwrap().drive_id = "b!verified".into();
        assert!(managed_plan_summary(&verified).is_ok());
        assert!(teams_verification_matches_draft(&draft, &verified));

        let mut changed = draft.clone();
        changed.remote_reference = "other_remote".into();
        assert!(!teams_verification_matches_draft(&changed, &verified));
        changed = draft.clone();
        changed.teams_library_url =
            "https://example.sharepoint.com/sites/Other/Shared%20Documents".into();
        assert!(!teams_verification_matches_draft(&changed, &verified));
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
            teams_library_url: "https://example.sharepoint.com/sites/Work/Shared%20Documents"
                .into(),
            local_path: "/tmp/sharepoint-offline-ui-test".into(),
            ..ConnectionDraft::default()
        };
        let mut verified = connection_from_draft(&draft).unwrap();
        verified.teams_identity.as_mut().unwrap().drive_id = "b!verified".into();
        assert!(teams_verification_matches_draft(&draft, &verified));

        let mut changed = draft.clone();
        changed.recovery_directory = "/tmp/another-recovery".into();
        assert!(!teams_verification_matches_draft(&changed, &verified));
    }

    #[test]
    fn teams_online_service_routes_through_guarded_rclone_without_preload() {
        let mut connection = test_connection(Provider::Teams);
        connection.remote_subpath = Some("General".into());
        connection.teams_identity = Some(cosmic_ext_applet_mounter::model::TeamsLibraryIdentity {
            site_url: "https://example.sharepoint.com/sites/Work".into(),
            library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
            drive_id: "b!verified".into(),
        });
        if let ConnectionMode::OnlineMount(options) = &mut connection.mode {
            options.start_at_login = true;
        }
        assert!(is_rclone_online_mount(&connection));
        assert!(!is_onedriver_online_mount(&connection));
        assert!(!uses_directory_preload(&connection));
        assert!(
            !ConfigDocument::default()
                .preload_policy_for(&connection)
                .enabled
        );
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
    fn sharepoint_marker_and_retarget_confirmation_are_safeguarded() {
        let temporary = tempfile::tempdir().unwrap();
        let mut connection = test_connection(Provider::Teams);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: temporary.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.local_path = temporary.path().join("local");
        connection.remote_subpath = Some("Disposable".into());
        connection.teams_identity = Some(cosmic_ext_applet_mounter::model::TeamsLibraryIdentity {
            site_url: "https://example.sharepoint.com/sites/Work".into(),
            library_url: "https://example.sharepoint.com/sites/Work/Documents".into(),
            drive_id: "b!verified".into(),
        });
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
            "rclone config update existing client_id [REDACTED] client_secret [REDACTED] config_is_local true --obscure --non-interactive"
        );
        assert!(!requests[1].sanitized_command().contains("private-client"));
        assert!(!requests[1].sanitized_command().contains("private-secret"));
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
    fn offline_mirror_markers_and_filters_stay_in_work_directory() {
        let temp = tempfile::TempDir::new().expect("temp");
        let mirror = temp.path().join("mirror");
        let work = temp.path().join("work");
        let recovery = temp.path().join("recovery");
        let mut connection = test_connection(Provider::GoogleDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
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
        assert!(mirror_script::script_path(&plan).starts_with(&plan.work_directory));
        assert!(!initial_preview_marker(&plan).starts_with(&mirror));
        assert!(!initial_sync_marker(&plan).starts_with(&mirror));
        assert!(
            fs::read_to_string(mirror_script::script_path(&plan))
                .expect("managed script")
                .contains("Cloud Mounter managed bisync script")
        );
        let managed = managed_rclone_bisync_request(
            &plan,
            "preview",
            rclone_bisync_preview_request(&plan).unwrap(),
        )
        .unwrap();
        assert_eq!(managed.executable, Executable::Sh);
        assert!(managed.sanitized_command().contains("managed-bisync.sh"));
        assert!(
            fs::read_to_string(&plan.filters_file)
                .expect("filters")
                .contains("Google cloud-native documents")
        );
        fs::write(mirror_script::script_path(&plan), "unowned script").unwrap();
        assert!(
            prepare_rclone_bisync_work_files(&connection, &plan)
                .unwrap_err()
                .contains("unowned")
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
            teams_identity: None,
        }
    }

    fn test_onedrive_offline_connection(root: &std::path::Path) -> Connection {
        let mut connection = test_connection(Provider::OneDrive);
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
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
}
