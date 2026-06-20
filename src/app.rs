// SPDX-License-Identifier: MIT

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
use std::sync::Arc;
use std::time::Duration;

use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{Alignment, Length, Limits, window::Id};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic_ext_applet_mounter::config::{APP_ID, Config};
use cosmic_ext_applet_mounter::controller::{
    ConnectionRowState, ControllerSnapshot, aggregate_label, decide_operation, operation_label,
    provider_label, restore, status_label,
};
use cosmic_ext_applet_mounter::import::{
    ImportPreview, ImportReplacementPlan, default_scan_directory, preview_import, replacement_plan,
    scan_legacy_units,
};
use cosmic_ext_applet_mounter::model::{
    AccessMode, Connection, ConnectionId, ConnectionMode, ConnectionStatus, OfflineMirrorConfig,
    OfflineMirrorStatus, OnlineMountConfig, OnlineMountStatus, Operation, Provider, TuningProfile,
    VpnKind, VpnProfile, VpnProfileId,
};
use cosmic_ext_applet_mounter::mounts::{MountEntry, MountTable, ProcMountTable, SyncRuntimeState};
use cosmic_ext_applet_mounter::process::{
    CommandError, CommandOutput, CommandRequest, CommandRunner, Executable, SystemCommandRunner,
    redact_text,
};
use cosmic_ext_applet_mounter::providers::{
    CommandRcloneProvider, OnedriverAuthState, ProviderError, lazy_unmount_request,
    onedriver_auth_state_for_plan, onedriver_mount_plan, rclone_mount_plan,
};
use cosmic_ext_applet_mounter::services::{
    ActiveState, CommandSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction,
    SystemdManager, UnitController, UnitDocument, UnitKind, UnitName, UnitStatus,
};
use cosmic_ext_applet_mounter::sync::{
    OneDriveIsolationReport, SyncDecision, SyncDecisionRejection, SyncReadiness, SyncRequest,
    SyncTrigger, google_native_filter_file, one_drive_auth_files_request, one_drive_auth_request,
    one_drive_mirror_plan, one_drive_preview_request, one_drive_sync_request, parse_preview,
    rclone_bisync_initial_preview_request, rclone_bisync_initial_sync_request, rclone_bisync_plan,
    rclone_bisync_preview_request, rclone_bisync_sync_request, sync_now_request,
};
use cosmic_ext_applet_mounter::vpn::{
    CiscoVpn, CommandCiscoVpn, CommandNetworkManagerVpn, CommandReadinessProbe, NetworkManagerVpn,
    readiness_report,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const CONNECTION_SETTINGS_TITLE: &str = "Cloud Mounter Connection Settings";
const APP_DISPLAY_NAME: &str = "COSMIC Cloud Mounter";
const POPUP_HEIGHT_BUDGET: f32 = 760.0;
const POPUP_FIXED_HEADER_ESTIMATE: f32 = 164.0;
const POPUP_NOTICE_LINE_CHARS: usize = 46;
const POPUP_NOTICE_LINE_HEIGHT: f32 = 28.0;
const POPUP_NOTICE_VERTICAL_PADDING: f32 = 18.0;
const POPUP_SCROLL_MAX_HEIGHT: f32 = 540.0;
const POPUP_CONNECTION_NAME_MAX_CHARS: usize = 36;
const POPUP_CONNECTION_ROW_HEIGHT: f32 = 48.0;
const POPUP_EMPTY_ROW_HEIGHT: f32 = 40.0;
const POPUP_CONNECTION_ROW_HORIZONTAL_PADDING: u16 = 0;
const SETTINGS_SECTION_TITLE_WIDTH: f32 = 150.0;
const SETTINGS_SECTION_TITLE_TOP_PADDING: u16 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLaunchMode {
    Applet,
    AddConnection,
    ModifyConnection(ConnectionId),
    ImportLegacy,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum WindowMode {
    #[default]
    AddConnection,
    ModifyConnection(ConnectionId),
    ImportLegacy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectionDraft {
    id: Option<ConnectionId>,
    name: String,
    provider: Provider,
    access_mode: AccessMode,
    remote_reference: String,
    remote_subpath: String,
    smb_host: String,
    smb_user: String,
    smb_domain: String,
    local_path: String,
    enabled: bool,
    start_at_login: bool,
    cache_limit_gib: String,
    sync_interval_minutes: String,
    sync_on_metered: bool,
    recovery_directory: String,
    vpn_profile_id: Option<VpnProfileId>,
    disconnect_vpn_when_unused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RcloneDraftRemote {
    name: String,
    backend: String,
}

impl Default for ConnectionDraft {
    fn default() -> Self {
        Self {
            id: None,
            name: "New storage connection".into(),
            provider: Provider::GoogleDrive,
            access_mode: AccessMode::OnlineMount,
            remote_reference: String::new(),
            remote_subpath: String::new(),
            smb_host: String::new(),
            smb_user: std::env::var("USER").unwrap_or_default(),
            smb_domain: "WORKGROUP".into(),
            local_path: String::new(),
            enabled: true,
            start_at_login: false,
            cache_limit_gib: "20".into(),
            sync_interval_minutes: "15".into(),
            sync_on_metered: false,
            recovery_directory: String::new(),
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
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
    onedrive_auth_open_command: String,
    onedrive_auth_response_url: String,
    last_notice: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    OpenAddConnection,
    OpenModifyConnection(ConnectionId),
    OpenImport,
    PopupClosed(Id),
    OperationRequested(ConnectionId, Operation),
    OperationCompleted(String),
    DraftProvider(Provider),
    DraftAccessMode(AccessMode),
    DraftName(String),
    DraftRemote(String),
    DraftSubpath(String),
    DraftSmbHost(String),
    DraftSmbUser(String),
    DraftSmbDomain(String),
    DraftLocalPath(String),
    DraftRecoveryDirectory(String),
    DraftCacheLimit(String),
    DraftSyncInterval(String),
    DraftEnabled(bool),
    DraftStartAtLogin(bool),
    DraftSyncOnMetered(bool),
    DraftVpn(Option<VpnProfileId>),
    DraftDisconnectVpn(bool),
    DraftOneDriveAuthResponse(String),
    DetectVpns,
    DetectRcloneRemotes,
    CreateGoogleDriveRcloneRemote,
    GoogleDriveRcloneRemoteCreated(Result<String, String>),
    CreateBoxRcloneRemote,
    BoxRcloneRemoteCreated(Result<String, String>),
    CreateSmbRcloneRemote,
    SmbRcloneRemoteCreated(Result<String, String>),
    StartOnedriverSetup,
    OnedriverSetupCompleted(Result<String, String>),
    StartOneDriveMirrorSetup,
    StartOneDriveMirrorManualSetup,
    OneDriveMirrorSetupCompleted(Result<String, String>),
    OpenOneDriveMirrorAuthUrl,
    SubmitOneDriveMirrorAuthResponse,
    TestDraft,
    DraftTested(String),
    SaveDraft,
    SaveDraftValidated(Connection, Result<String, String>),
    DraftSaved(String),
    ConfirmImport(usize),
    RemoveConnection(ConnectionId),
    RemoveCompleted(String),
    Refresh,
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
        let config = Config::load().config;
        let main_window_id = core.main_window_id();
        let standalone = flags != AppLaunchMode::Applet;
        let window_mode = match flags {
            AppLaunchMode::Applet | AppLaunchMode::AddConnection => WindowMode::AddConnection,
            AppLaunchMode::ModifyConnection(id) => WindowMode::ModifyConnection(id),
            AppLaunchMode::ImportLegacy => WindowMode::ImportLegacy,
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
            onedrive_auth_open_command: String::new(),
            onedrive_auth_response_url: String::new(),
            last_notice: None,
        };
        match flags {
            AppLaunchMode::ModifyConnection(id) => app.load_draft(id),
            AppLaunchMode::ImportLegacy => app.scan_imports(),
            AppLaunchMode::Applet | AppLaunchMode::AddConnection => {}
        }
        let title = if standalone {
            app.set_header_title(CONNECTION_SETTINGS_TITLE.into());
            if app.last_notice.is_none() {
                app.last_notice = Some(window_mode_notice(window_mode));
            }
            app.set_window_title(
                CONNECTION_SETTINGS_TITLE.into(),
                main_window_id.unwrap_or(Id::RESERVED),
            )
        } else if let Some(id) = main_window_id {
            app.set_window_title(APP_DISPLAY_NAME.into(), id)
        } else {
            Task::none()
        };

        (app, title)
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        if self.standalone {
            return None;
        }
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.standalone {
            return self.view_settings_window();
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

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    self.config = Config::load().config;
                    self.pending_remove = None;
                    self.pending_repair = None;
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
                    Task::batch([
                        get_popup(settings),
                        self.set_window_title(APP_DISPLAY_NAME.into(), id),
                    ])
                };
            }
            Message::OpenAddConnection => {
                self.pending_remove = None;
                self.pending_repair = None;
                self.launch_settings_process(AppLaunchMode::AddConnection);
            }
            Message::OpenModifyConnection(connection_id) => {
                self.pending_remove = None;
                self.pending_repair = None;
                self.launch_settings_process(AppLaunchMode::ModifyConnection(connection_id));
            }
            Message::OpenImport => {
                self.pending_remove = None;
                self.pending_repair = None;
                self.launch_settings_process(AppLaunchMode::ImportLegacy);
            }
            Message::PopupClosed(id) if self.popup == Some(id) => {
                self.popup = None;
            }
            Message::PopupClosed(_) => {}
            Message::OperationRequested(connection_id, operation) => {
                return self.record_operation_request(connection_id, operation);
            }
            Message::OperationCompleted(notice) => {
                self.config = Config::load().config;
                self.pending_repair = None;
                self.last_notice = Some(notice);
            }
            Message::DraftProvider(provider) => {
                self.draft.provider = provider;
                if provider == Provider::OneDrive {
                    self.draft.remote_reference = "onedrive".into();
                }
            }
            Message::DraftAccessMode(mode) => {
                self.draft.access_mode = mode;
            }
            Message::DraftName(value) => {
                self.draft.name = value;
            }
            Message::DraftRemote(value) => {
                self.draft.remote_reference = value;
            }
            Message::DraftSubpath(value) => {
                self.draft.remote_subpath = value;
            }
            Message::DraftSmbHost(value) => {
                self.draft.smb_host = value;
            }
            Message::DraftSmbUser(value) => {
                self.draft.smb_user = value;
            }
            Message::DraftSmbDomain(value) => {
                self.draft.smb_domain = value;
            }
            Message::DraftLocalPath(value) => {
                self.draft.local_path = value;
            }
            Message::DraftRecoveryDirectory(value) => {
                self.draft.recovery_directory = value;
            }
            Message::DraftCacheLimit(value) => {
                self.draft.cache_limit_gib = value;
            }
            Message::DraftSyncInterval(value) => {
                self.draft.sync_interval_minutes = value;
            }
            Message::DraftEnabled(value) => {
                self.draft.enabled = value;
            }
            Message::DraftStartAtLogin(value) => {
                self.draft.start_at_login = value;
            }
            Message::DraftSyncOnMetered(value) => {
                self.draft.sync_on_metered = value;
            }
            Message::DraftVpn(id) => {
                self.draft.vpn_profile_id = id;
            }
            Message::DraftDisconnectVpn(value) => {
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
            Message::CreateGoogleDriveRcloneRemote => {
                return self.create_google_drive_rclone_remote();
            }
            Message::GoogleDriveRcloneRemoteCreated(result) => match result {
                Ok(remote_name) => {
                    self.draft.remote_reference = remote_name.clone();
                    self.detect_rclone_remotes();
                    self.last_notice = Some(format!(
                        "Created Google Drive rclone remote `{remote_name}`. It is selected; run Test Connection to verify access."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!(
                        "Could not create Google Drive rclone remote: {error}"
                    ));
                }
            },
            Message::CreateBoxRcloneRemote => {
                return self.create_box_rclone_remote();
            }
            Message::BoxRcloneRemoteCreated(result) => match result {
                Ok(remote_name) => {
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
            Message::CreateSmbRcloneRemote => {
                return self.create_smb_rclone_remote();
            }
            Message::SmbRcloneRemoteCreated(result) => match result {
                Ok(remote_name) => {
                    self.draft.remote_reference = remote_name.clone();
                    self.detect_rclone_remotes();
                    self.last_notice = Some(format!(
                        "Created SMB rclone remote `{remote_name}`. It is selected; run Test Connection to verify access."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!("Could not create SMB rclone remote: {error}"));
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
                    self.last_notice = Some(format!(
                        "OneDrive Offline Mirror setup completed. {summary} Run Test Connection, preview the initial synchronization, then Save Connection."
                    ));
                }
                Err(error) => {
                    self.last_notice = Some(format!(
                        "Could not complete OneDrive Offline Mirror setup: {error}"
                    ));
                }
            },
            Message::OpenOneDriveMirrorAuthUrl => {
                self.open_onedrive_mirror_auth_url();
            }
            Message::SubmitOneDriveMirrorAuthResponse => {
                self.submit_onedrive_mirror_auth_response();
            }
            Message::TestDraft => {
                return self.test_draft_plan();
            }
            Message::DraftTested(notice) => {
                self.last_notice = Some(notice);
            }
            Message::SaveDraft => {
                return self.save_draft();
            }
            Message::SaveDraftValidated(connection, validation) => {
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
                self.last_notice = Some(notice);
            }
            Message::ConfirmImport(index) => {
                return self.confirm_import(index);
            }
            Message::RemoveConnection(connection_id) => {
                return self.remove_connection_with_confirmation(connection_id);
            }
            Message::RemoveCompleted(notice) => {
                self.last_notice = Some(notice);
            }
            Message::Refresh => {
                self.config = Config::load().config;
                self.pending_remove = None;
                self.last_notice = Some("Configuration reloaded.".into());
            }
        }

        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn view_popup(&self) -> Element<'_, Message> {
        let notification_status = if self.config.notifications_enabled() {
            fl!("notifications-enabled")
        } else {
            fl!("notifications-disabled")
        };
        let state = self.view_state();
        let controls = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(field_with_help(
                widget::button::suggested(fl!("add-connection"))
                    .on_press(Message::OpenAddConnection),
                "Open the Add Connection workflow to create a new storage connection.",
            ))
            .push(field_with_help(
                widget::button::standard(fl!("refresh")).on_press(Message::Refresh),
                "Reload saved configuration and refresh the applet view.",
            ));
        let header = widget::list_column()
            .add(widget::text::title4(fl!("app-title")))
            .add(widget::text::body(format!(
                "{}\n{}\n{}",
                aggregate_label(&state.aggregate),
                notification_status,
                self.vpn_summary(&state.rows)
            )))
            .add(controls);

        let mut rows = widget::list_column();
        if let Some(notice) = &self.last_notice {
            rows = rows.add(
                widget::container(widget::text::body(notice.clone()))
                    .padding([8, 0])
                    .width(Length::Fill),
            );
        }

        if state.rows.is_empty() {
            rows = rows.add(widget::text::body(fl!("no-connections")));
        }

        for row in &state.rows {
            rows = rows.add(self.view_connection_row(row));
        }

        let content =
            widget::Column::new()
                .spacing(12)
                .push(header)
                .push(widget::scrollable(rows).height(Length::Fixed(
                    popup_connection_scroll_height(
                        self.last_notice.as_deref(),
                        state.rows.len(),
                        state.rows.is_empty(),
                    ),
                )));

        self.core.applet.popup_container(content).into()
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
        let primary_control: Element<'static, Message> = if let Some(operation) = primary {
            widget::toggler(primary_enabled)
                .on_toggle(move |_| Message::OperationRequested(row_id, operation))
                .into()
        } else {
            widget::toggler(primary_enabled).into()
        };
        let display_name = popup_connection_display_name(&row.name);
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
                widget::container(field_with_help(primary_control, primary_help))
                    .align_x(Alignment::End),
            );

        widget::container(row_content)
            .padding([4, POPUP_CONNECTION_ROW_HORIZONTAL_PADDING])
            .width(Length::Fill)
            .into()
    }

    fn view_settings_window(&self) -> Element<'_, Message> {
        let mut content = widget::list_column().add(widget::text::title2(match self.window_mode {
            WindowMode::AddConnection => fl!("add-connection"),
            WindowMode::ModifyConnection(_) => "Modify Connection".into(),
            WindowMode::ImportLegacy => fl!("settings-import-title"),
        }));

        if let Some(notice) = &self.last_notice {
            content = content.add(widget::text::body(notice.clone()));
        }

        match self.window_mode {
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
        content = content
            .add(section_row_with_help(
                "Provider",
                "Choose the storage provider. OneDrive uses OneDrive-specific engines; Google Drive, Box, and SMB use rclone.",
                choice_row(vec![
                    provider_choice("OneDrive", Provider::OneDrive, self.draft.provider),
                    provider_choice("Google Drive", Provider::GoogleDrive, self.draft.provider),
                    provider_choice("Box", Provider::Box, self.draft.provider),
                    provider_choice("SMB", Provider::Smb, self.draft.provider),
                ]),
            ))
            .add(section_row_with_help(
                "Access mode",
                "Online mount gives on-demand network-backed access. Offline mirror keeps a local copy and synchronizes later.",
                choice_row(vec![
                    mode_choice(
                        "Online mount",
                        AccessMode::OnlineMount,
                        self.draft.access_mode,
                    ),
                    mode_choice(
                        "Offline mirror",
                        AccessMode::OfflineMirror,
                        self.draft.access_mode,
                    ),
                ]),
            ))
            .add(section_row(
                "Connection",
                widget::Column::new()
                    .spacing(8)
                    .push(field_with_help(
                        widget::text_input::text_input("Connection name", &self.draft.name)
                            .on_input(Message::DraftName),
                        "Display name shown in the applet popup.",
                    ))
                    .push(self.view_remote_account_fields()),
            ))
            .add(section_row_with_safety_help(
                local_target_label(self.draft.access_mode),
                "Do not reuse mountpoints and mirror directories.",
                "Online mounts use a mountpoint. Offline mirrors use an ordinary local directory.",
                widget::text_input::text_input("/home/user/Cloud/Example", &self.draft.local_path)
                    .on_input(Message::DraftLocalPath),
            ));

        content = match self.draft.access_mode {
            AccessMode::OnlineMount => content.add(section_row(
                "Online mount settings",
                widget::Column::new()
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
                    )),
            )),
            AccessMode::OfflineMirror => content.add(section_row(
                "Offline mirror settings",
                widget::Column::new()
                    .spacing(8)
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
                    ))
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
            )),
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
                action_button("Save Connection", primary_ready, Message::SaveDraft),
                "Preview and confirm before initial synchronization.",
                "Save this connection after validation. Potentially destructive sync setup still requires preview and confirmation.",
            ));

        primary_row = primary_row.push_maybe(self.view_onedrive_setup_actions());

        match self.window_mode {
            WindowMode::AddConnection => {
                if self.draft_uses_rclone() {
                    primary_row = primary_row
                        .push(field_with_help(
                            widget::button::standard("Detect rclone remotes")
                                .on_press(Message::DetectRcloneRemotes),
                            format!(
                                "Read rclone config dump, filter remotes by provider backend, and offer matching remotes as selectable account choices. If no {} remotes are detected, enter an existing remote name or create one here.",
                                provider_label(self.draft.provider)
                            ),
                        ))
                        .push(self.view_create_rclone_remote_action());
                }
                primary_row = primary_row.push(field_with_help(
                    widget::button::standard("Import").on_press(Message::OpenImport),
                    "Scan existing user services, preview compatible rclone or onedriver mounts, then map one into this wizard.",
                ));
            }
            WindowMode::ModifyConnection(connection_id) => {
                let enable_label = if self.draft.enabled {
                    "Disable"
                } else {
                    "Enable"
                };
                let mut modify_row = widget::Row::new().spacing(8).align_y(Alignment::Center);
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
                        widget::button::standard(enable_label)
                            .on_press(Message::DraftEnabled(!self.draft.enabled)),
                        "Disable prevents automatic use without deleting credentials, data, cache, recovery, or imported originals.",
                    ))
                    .push(field_with_help(
                        widget::button::destructive("Remove")
                            .on_press(Message::RemoveConnection(connection_id)),
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
            WindowMode::ImportLegacy => {}
        }

        primary_row.into()
    }

    fn view_onedrive_setup_actions(&self) -> Option<Element<'static, Message>> {
        if self.draft.provider != Provider::OneDrive {
            return None;
        }

        Some(match self.draft.access_mode {
            AccessMode::OnlineMount => field_with_help(
                widget::button::suggested("Start OneDrive Setup")
                    .on_press(Message::StartOnedriverSetup),
                format!(
                    "{} Runs `onedriver --auth-only` with this connection's app-owned config file and cache directory. Complete authorization in the browser, then run Test Connection.",
                    onedrive_setup_guidance(self.draft.access_mode)
                ),
            ),
            AccessMode::OfflineMirror => choice_row(vec![
                field_with_help(
                    widget::button::suggested("Start OneDrive Mirror Setup")
                        .on_press(Message::StartOneDriveMirrorSetup),
                    format!(
                        "{} Runs `onedrive --reauth` with this connection's app-owned config directory. Complete authorization in the browser; onedrive should receive the local redirect itself.",
                        onedrive_setup_guidance(self.draft.access_mode)
                    ),
                ),
                field_with_help(
                    widget::button::standard("Use Manual Auth Handoff")
                        .on_press(Message::StartOneDriveMirrorManualSetup),
                    "Fallback for browser or tenant cases where onedrive cannot capture the redirect automatically. The applet prepares auth-files and a response URL field.",
                ),
            ]),
        })
    }

    fn view_create_rclone_remote_action(&self) -> Element<'static, Message> {
        match self.draft.provider {
            Provider::GoogleDrive => field_with_help(
                widget::button::suggested("Create Google Drive Remote")
                    .on_press(Message::CreateGoogleDriveRcloneRemote),
                "Create the rclone Google Drive remote with full-drive scope and local browser OAuth. Complete the browser authorization window, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.",
            ),
            Provider::Box => field_with_help(
                widget::button::suggested("Create Box Remote")
                    .on_press(Message::CreateBoxRcloneRemote),
                "Create the rclone Box remote with local browser OAuth. Complete the browser authorization window that rclone opens, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.",
            ),
            Provider::Smb => field_with_help(
                widget::button::suggested("Create SMB Remote")
                    .on_press(Message::CreateSmbRcloneRemote),
                "Create the rclone SMB remote with host/user/domain only, then detect and select it. Passwords stay in rclone, not applet config.",
            ),
            Provider::OneDrive => widget::Space::new().width(Length::Shrink).into(),
        }
    }

    fn draft_uses_rclone(&self) -> bool {
        matches!(
            self.draft.provider,
            Provider::GoogleDrive | Provider::Box | Provider::Smb
        )
    }

    fn draft_primary_actions_ready(&self) -> bool {
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
            Provider::GoogleDrive | Provider::Box | Provider::Smb => {
                self.view_rclone_remote_fields()
            }
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
                    "optional OneDrive folder/subtree",
                    &self.draft.remote_subpath,
                )
                .on_input(Message::DraftSubpath),
                "Leave empty for the whole OneDrive account, or enter an existing folder/subtree to limit this connection.",
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
            return widget::text::body(
                "Press Start OneDrive Mirror Setup to prepare the auth URL handoff files.",
            )
            .into();
        }
        widget::container(
            widget::Column::new()
                .spacing(8)
                .push(widget::text::body(
                    "OneDrive mirror authentication handoff",
                ))
                .push(widget::text::body(
                    "After pressing Start OneDrive Mirror Setup, wait a moment for onedrive to create the auth URL, then open it in your browser:",
                ))
                .push(field_with_help(
                widget::button::suggested("Open OneDrive Auth Helper")
                        .on_press(Message::OpenOneDriveMirrorAuthUrl),
                    "Open the generated onedrive auth URL in the WebKitGTK helper when available. The helper attempts to capture the final Microsoft redirect automatically; otherwise the applet falls back to xdg-open.",
                ))
                .push(widget::text::body(
                    "Shell fallback:",
                ))
                .push(field_with_help(
                    widget::text_input::text_input(
                        "shell command",
                        &self.onedrive_auth_open_command,
                    )
                        .on_input(|_| Message::DraftOneDriveAuthResponse(String::new())),
                    "This selectable command opens the same auth URL in your normal browser if the WebKitGTK helper cannot be used.",
                ))
                .push(widget::text::body(
                    "After Microsoft redirects to the native-client page, paste the full browser address-bar URL below. The applet writes it to the transient response file for onedrive and does not save it in configuration.",
                ))
                .push(field_with_help(
                    widget::text_input::text_input(
                        "paste full Microsoft redirect URL",
                        &self.onedrive_auth_response_url,
                    )
                    .on_input(Message::DraftOneDriveAuthResponse),
                    "Paste the full URL beginning with https://login.microsoftonline.com/... and containing code=.",
                ))
                .push(field_with_help(
                    widget::button::suggested("Submit OneDrive Response URL")
                        .on_press(Message::SubmitOneDriveMirrorAuthResponse),
                    "Write the pasted response URL to the transient response file expected by the running onedrive authentication process.",
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
            rclone_remote_help(provider),
        ));

        let matching = self.matching_rclone_remotes(provider);
        if !matching.is_empty() {
            column = column.push(widget::text::body(format!(
                "Detected {} rclone remotes:",
                provider_label(provider)
            )));
            column = column.push(self.view_rclone_remote_choices(matching));
        }

        if provider == Provider::Smb && self.window_mode == WindowMode::AddConnection {
            column = column.push(self.view_smb_remote_setup_fields());
        }

        column
            .push(field_with_help(
                widget::text_input::text_input(
                    "optional remote subtree/folder",
                    &self.draft.remote_subpath,
                )
                .on_input(Message::DraftSubpath),
                "Leave empty for the whole rclone remote, or enter an existing folder/subtree to limit this connection.",
            ))
            .into()
    }

    fn view_smb_remote_setup_fields(&self) -> Element<'_, Message> {
        widget::Column::new()
            .spacing(8)
            .push(field_with_help(
                widget::text_input::text_input("SMB server host", &self.draft.smb_host)
                    .on_input(Message::DraftSmbHost),
                "Server DNS name or IP address used for rclone's SMB `host` option. Create SMB Remote uses these fields and leaves passwords in rclone, not applet configuration.",
            ))
            .push(field_with_help(
                widget::text_input::text_input("SMB username", &self.draft.smb_user)
                    .on_input(Message::DraftSmbUser),
                "Optional SMB username. Leave blank for guest or rclone defaults.",
            ))
            .push(field_with_help(
                widget::text_input::text_input("SMB domain or WORKGROUP", &self.draft.smb_domain)
                    .on_input(Message::DraftSmbDomain),
                "Optional NTLM domain. WORKGROUP is rclone's default.",
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
        let mut row = widget::Row::new().spacing(8).align_y(Alignment::Center);
        for remote in remotes {
            let selected = self.draft.remote_reference == remote.name;
            let label = remote.name.clone();
            let help = format!(
                "Use rclone remote `{}` with backend `{}` for this connection.",
                remote.name, remote.backend
            );
            row = row.push(field_with_help(
                select_button(label, selected, Message::DraftRemote(remote.name)),
                help,
            ));
        }
        row.into()
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
                self.last_notice = Some(format!("{label} requested for {}...", connection.name));
                Task::perform(
                    async move { run_managed_online_mount_operation(connection, operation).await },
                    |notice| cosmic::Action::App(Message::OperationCompleted(notice)),
                )
            }
            (ConnectionMode::OnlineMount(_), Operation::Mount | Operation::Unmount)
                if is_onedriver_online_mount(&connection) =>
            {
                self.last_notice = Some(format!("{label} requested for {}...", connection.name));
                Task::perform(
                    async move {
                        run_managed_onedriver_online_mount_operation(connection, operation).await
                    },
                    |notice| cosmic::Action::App(Message::OperationCompleted(notice)),
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
                Task::perform(
                    async move { run_managed_offline_mirror_operation(connection, operation).await },
                    |notice| cosmic::Action::App(Message::OperationCompleted(notice)),
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
                Task::perform(
                    async move {
                        run_managed_onedrive_offline_mirror_operation(connection, operation).await
                    },
                    |notice| cosmic::Action::App(Message::OperationCompleted(notice)),
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
        self.draft = draft_from_connection(connection);
        self.last_notice = Some(format!("Modify {}.", connection.name));
    }

    fn save_draft(&mut self) -> Task<cosmic::Action<Message>> {
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
        if connection.provider == Provider::OneDrive {
            self.last_notice = Some(format!("Validating {} before saving...", connection.name));
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
        self.save_validated_draft(connection, None)
    }

    fn save_validated_draft(
        &mut self,
        connection: Connection,
        validation_summary: Option<String>,
    ) -> Task<cosmic::Action<Message>> {
        let storage = match config_storage() {
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
        let result = self.config.update_validated(&storage, |document| {
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
                Task::none()
            }
            Ok(false) => {
                self.last_notice = Some(format!("{name} was unchanged."));
                Task::none()
            }
            Err(error) => {
                self.last_notice = Some(format!("Failed to save {name}: {error}"));
                Task::none()
            }
        }
    }

    fn detect_and_import_vpns(&mut self) {
        let detection = detect_vpn_profiles();
        let storage = match config_storage() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return;
            }
        };

        let mut imported = 0usize;
        let mut updated = 0usize;
        let mut selected = None;
        let result = self.config.update_validated(&storage, |document| {
            for detected in &detection.profiles {
                if let Some(existing) = document
                    .vpn_profiles
                    .iter_mut()
                    .find(|profile| same_vpn_reference(profile, detected))
                {
                    existing.name = detected.name.clone();
                    existing.readiness_checks = detected.readiness_checks.clone();
                    existing.timeout_seconds = detected.timeout_seconds;
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
        match Command::new("rclone").args(["config", "dump"]).output() {
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

    fn create_google_drive_rclone_remote(&mut self) -> Task<cosmic::Action<Message>> {
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
        self.last_notice = Some(format!(
            "Starting Google Drive OAuth for rclone remote `{}`. Complete the browser authorization window; this can take a minute.",
            setup.name
        ));
        Task::perform(
            async move { create_google_drive_rclone_remote_result(setup).await },
            |result| cosmic::Action::App(Message::GoogleDriveRcloneRemoteCreated(result)),
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
                    "Starting OneDrive mirror setup for `{}`. Complete the browser authorization window; onedrive should capture the local redirect automatically. If it fails or times out, use Manual Auth Handoff.",
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
                    "Starting manual OneDrive mirror auth handoff for `{}`. When the auth URL is ready, press Open OneDrive Auth Helper. If automatic capture fails, paste the final Microsoft redirect URL into the response field.",
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
        let storage = match config_storage() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        let result = self.config.update_validated(&storage, |document| {
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
        let connection = match connection_from_draft(&self.draft) {
            Ok(connection) => connection,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        self.last_notice = Some(format!("Testing {}...", connection.name));
        Task::perform(
            async move { test_connection_plan_and_access(connection).await },
            |notice| cosmic::Action::App(Message::DraftTested(notice)),
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
        let storage = match config_storage() {
            Ok(storage) => storage,
            Err(error) => {
                self.last_notice = Some(error);
                return Task::none();
            }
        };
        let result = self.config.update_validated(&storage, |document| {
            document
                .connections
                .retain(|connection| connection.id != connection_id);
        });
        self.pending_remove = None;
        match result {
            Ok(true) => {
                self.last_notice = Some(format!(
                    "{name} was removed from applet configuration. Removing applet-owned generated units..."
                ));
                Task::perform(
                    async move { remove_generated_units_for_connection(connection).await },
                    |notice| cosmic::Action::App(Message::RemoveCompleted(notice)),
                )
            }
            Ok(false) => {
                self.last_notice = Some(format!("{name} was already absent."));
                Task::none()
            }
            Err(error) => {
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
                    ) {
                        Ok(preview) => previews.push(preview),
                        Err(error) => errors.push(format!("{}: {error}", unit.name)),
                    }
                }
                let count = previews.len();
                self.import_previews = previews;
                self.last_notice = Some(if errors.is_empty() {
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

    fn connection_vpn_label(&self, vpn_profile_id: Option<VpnProfileId>) -> String {
        let Some(profile_id) = vpn_profile_id else {
            return "None".into();
        };
        self.config
            .document
            .vpn_profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| profile_id.to_string())
    }

    fn vpn_summary(&self, rows: &[ConnectionRowState]) -> String {
        let configured = rows
            .iter()
            .filter_map(|row| row.vpn_profile_id)
            .collect::<BTreeSet<_>>();
        if configured.is_empty() {
            return "No VPN enabled".into();
        }
        let names = configured
            .iter()
            .map(|profile_id| self.connection_vpn_label(Some(*profile_id)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("VPN configured: {names}")
    }

    fn view_state(&self) -> cosmic_ext_applet_mounter::controller::ControllerViewState {
        let (sync_state, paused_syncs) =
            runtime_offline_mirror_states(&self.config.document.connections);
        restore(&ControllerSnapshot {
            config: self.config.document.clone(),
            service_status: runtime_service_statuses(&self.config.document.connections),
            sync_state,
            paused_syncs,
            mount_entries: ProcMountTable::default().entries().unwrap_or_default(),
            import_previews: self.import_previews.clone(),
            ..ControllerSnapshot::default()
        })
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
        Provider::GoogleDrive | Provider::Box | Provider::Smb => {
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
    let output = Command::new("systemctl")
        .arg("--user")
        .arg("show")
        .arg("--property=ActiveState")
        .arg("--property=UnitFileState")
        .arg("--property=SubState")
        .arg(unit.file_name())
        .output()
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
    }
}

impl AppModel {
    pub fn launch_mode_from_args() -> AppLaunchMode {
        let mut args = env::args().skip(1);
        match args.next().as_deref() {
            Some("--settings") | Some("settings") | Some("--add-connection") => {
                AppLaunchMode::AddConnection
            }
            Some("--import") => AppLaunchMode::ImportLegacy,
            Some("--modify-connection") => args
                .next()
                .and_then(|value| Uuid::parse_str(&value).ok())
                .map(ConnectionId::from_uuid)
                .map(AppLaunchMode::ModifyConnection)
                .unwrap_or(AppLaunchMode::AddConnection),
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
            return;
        };

        let mut command = Command::new(executable);
        match mode {
            AppLaunchMode::Applet | AppLaunchMode::AddConnection => {
                command.arg("--settings");
            }
            AppLaunchMode::ModifyConnection(id) => {
                command.arg("--modify-connection").arg(id.to_string());
            }
            AppLaunchMode::ImportLegacy => {
                command.arg("--import");
            }
        }

        self.last_notice = match command.spawn() {
            Ok(_) => Some(window_mode_notice(match mode {
                AppLaunchMode::Applet | AppLaunchMode::AddConnection => WindowMode::AddConnection,
                AppLaunchMode::ModifyConnection(id) => WindowMode::ModifyConnection(id),
                AppLaunchMode::ImportLegacy => WindowMode::ImportLegacy,
            })),
            Err(error) => Some(format!("Could not open connection settings: {error}")),
        };
    }
}

fn primary_operation(row: &ConnectionRowState) -> Option<Operation> {
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

fn popup_connection_scroll_height(
    notice: Option<&str>,
    connection_count: usize,
    show_empty_state: bool,
) -> f32 {
    let notice_height = notice.map_or(0.0, |text| {
        let lines = text
            .lines()
            .map(|line| (line.chars().count() / POPUP_NOTICE_LINE_CHARS) + 1)
            .sum::<usize>()
            .max(1) as f32;
        (lines * POPUP_NOTICE_LINE_HEIGHT) + POPUP_NOTICE_VERTICAL_PADDING
    });
    let row_height = if show_empty_state {
        POPUP_EMPTY_ROW_HEIGHT
    } else {
        connection_count as f32 * POPUP_CONNECTION_ROW_HEIGHT
    };
    let content_height = notice_height + row_height;
    let max_height = (POPUP_HEIGHT_BUDGET - POPUP_FIXED_HEADER_ESTIMATE)
        .clamp(POPUP_EMPTY_ROW_HEIGHT, POPUP_SCROLL_MAX_HEIGHT);

    content_height.clamp(POPUP_EMPTY_ROW_HEIGHT, max_height)
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
        WindowMode::AddConnection => {
            "Add connection selected. Choose provider, mode, remote/subtree, local target, VPN, and start at login.".into()
        }
        WindowMode::ModifyConnection(_) => "Modify connection selected.".into(),
        WindowMode::ImportLegacy => {
            "Import selected. Scan ~/.config/systemd/user, preview compatible services, and confirm replacements before any changes.".into()
        }
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
    ConnectionDraft {
        id: Some(connection.id),
        name: connection.name.clone(),
        provider: connection.provider,
        access_mode,
        remote_reference: connection.remote_reference.clone(),
        remote_subpath: connection.remote_subpath.clone().unwrap_or_default(),
        smb_host: String::new(),
        smb_user: std::env::var("USER").unwrap_or_default(),
        smb_domain: "WORKGROUP".into(),
        local_path: connection.local_path.display().to_string(),
        enabled: connection.enabled,
        start_at_login,
        cache_limit_gib,
        sync_interval_minutes,
        sync_on_metered,
        recovery_directory,
        vpn_profile_id: connection.vpn_profile_id,
        disconnect_vpn_when_unused: connection.disconnect_vpn_when_unused,
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
            "Safety: Preview is dry-run; initial synchronization requires Preview plus confirmed Sync Now before background Start.".into()
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
        "COSMIC Mounter VPN detection started at {:?}",
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
        let runner = SystemCommandRunner;
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
    runner: &SystemCommandRunner,
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
    help: &'static str,
    body: impl Into<Element<'a, Message>> + 'a,
) -> Element<'a, Message> {
    section_row(title, field_with_help(body, help))
}

fn section_row_with_safety_help<'a>(
    title: &'static str,
    warning: &'static str,
    help: &'static str,
    body: impl Into<Element<'a, Message>> + 'a,
) -> Element<'a, Message> {
    section_row(title, field_with_safety_help(body, warning, help))
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
    label: &'static str,
    provider: Provider,
    selected: Provider,
) -> Element<'static, Message> {
    select_button(
        label,
        provider == selected,
        Message::DraftProvider(provider),
    )
}

fn mode_choice(
    label: &'static str,
    mode: AccessMode,
    selected: AccessMode,
) -> Element<'static, Message> {
    select_button(label, mode == selected, Message::DraftAccessMode(mode))
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

fn local_target_label(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => "Mountpoint",
        AccessMode::OfflineMirror => "Mirror directory",
    }
}

fn rclone_backend_name(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::GoogleDrive => Some("drive"),
        Provider::Box => Some("box"),
        Provider::Smb => Some("smb"),
        Provider::OneDrive => None,
    }
}

fn rclone_remote_placeholder(provider: Provider) -> &'static str {
    match provider {
        Provider::GoogleDrive => "Google Drive rclone remote name",
        Provider::Box => "Box rclone remote name",
        Provider::Smb => "SMB rclone remote name",
        Provider::OneDrive => "remote name",
    }
}

fn rclone_remote_help(provider: Provider) -> &'static str {
    match provider {
        Provider::GoogleDrive => {
            "Select a detected Google Drive rclone remote, or enter the exact remote name from `rclone config`. Use a clear name such as `personal_gdrive` or `work_gdrive`; the applet verifies backend type `drive`, authentication, and subtree access before saving."
        }
        Provider::Box => {
            "Select a detected Box rclone remote, or enter the exact remote name from `rclone config`. Use a clear name such as `box_personal` or `ua_box`; the applet verifies backend type `box`, authentication, and subtree access before saving."
        }
        Provider::Smb => {
            "Select a detected SMB rclone remote, or enter the exact remote name from `rclone config`. Use a clear name such as `office_smb`; the applet verifies backend type `smb`. Passwords stay in rclone, not applet configuration."
        }
        Provider::OneDrive => "OneDrive does not use rclone in the approved provider matrix.",
    }
}

fn onedrive_account_placeholder(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => "onedriver account/setup reference",
        AccessMode::OfflineMirror => "abraunegg/onedrive account/setup reference",
    }
}

fn onedrive_account_help(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => {
            "Label this OneDrive Online Mount account so you can recognize it later, for example `onedriver-work`. Test Connection and Save validate jstaf/onedriver, app-owned auth metadata, mountpoint safety, and active onedriver overlaps without reading provider tokens."
        }
        AccessMode::OfflineMirror => {
            "Label this OneDrive Offline Mirror account so you can recognize it later, for example `onedrive-personal`. Test Connection and Save validate auth metadata, directory safety, active onedriver overlap, and a bounded dry-run preview."
        }
    }
}

fn onedrive_account_safety_warning(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => {
            "Do not reuse this mountpoint as an abraunegg/onedrive sync directory."
        }
        AccessMode::OfflineMirror => {
            "Do not run onedriver and abraunegg/onedrive against overlapping OneDrive trees."
        }
    }
}

fn onedrive_setup_guidance(mode: AccessMode) -> &'static str {
    match mode {
        AccessMode::OnlineMount => {
            "Setup guidance: Online Mount uses jstaf/onedriver with app-owned config/cache paths. Test Connection and Save validate setup metadata and mountpoint safety; credentials remain with onedriver."
        }
        AccessMode::OfflineMirror => {
            "Setup guidance: Offline Mirror uses abraunegg/onedrive with app-owned confdir/syncdir/recovery paths. Test Connection and Save validate auth state and run a dry-run preview; credentials remain with onedrive."
        }
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
        if matches!(backend, "drive" | "box" | "smb") {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmbRemoteSetup {
    name: String,
    host: String,
    user: Option<String>,
    domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoxRemoteSetup {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoogleDriveRemoteSetup {
    name: String,
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
        Ok(Self {
            name: name.to_owned(),
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
        let host = draft.smb_host.trim();
        validate_setup_value("SMB host", host, true)?;
        let user = optional_setup_value("SMB username", &draft.smb_user)?;
        let domain = optional_setup_value("SMB domain", &draft.smb_domain)?;
        Ok(Self {
            name: name.to_owned(),
            host: host.to_owned(),
            user,
            domain,
        })
    }
}

async fn create_google_drive_rclone_remote_result(
    setup: GoogleDriveRemoteSetup,
) -> Result<String, String> {
    create_google_drive_rclone_remote_with(&SystemCommandRunner, setup).await
}

async fn create_google_drive_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: GoogleDriveRemoteSetup,
) -> Result<String, String> {
    ensure_rclone_remote_name_available(runner, &setup.name).await?;
    runner
        .run(
            google_drive_rclone_config_create_request(&setup)?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error("Google Drive OAuth setup", error))?;
    Ok(setup.name)
}

async fn create_box_rclone_remote_result(setup: BoxRemoteSetup) -> Result<String, String> {
    create_box_rclone_remote_with(&SystemCommandRunner, setup).await
}

async fn create_box_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: BoxRemoteSetup,
) -> Result<String, String> {
    ensure_rclone_remote_name_available(runner, &setup.name).await?;
    runner
        .run(
            box_rclone_config_create_request(&setup)?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error("Box OAuth setup", error))?;
    Ok(setup.name)
}

async fn create_smb_rclone_remote_result(setup: SmbRemoteSetup) -> Result<String, String> {
    create_smb_rclone_remote_with(&SystemCommandRunner, setup).await
}

async fn create_smb_rclone_remote_with(
    runner: &dyn CommandRunner,
    setup: SmbRemoteSetup,
) -> Result<String, String> {
    ensure_rclone_remote_name_available(runner, &setup.name).await?;
    runner
        .run(
            smb_rclone_config_create_request(&setup)?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_setup_command_error("config create", error))?;
    Ok(setup.name)
}

async fn run_onedriver_online_setup_result(connection: Connection) -> Result<String, String> {
    run_onedriver_online_setup_with(
        &SystemCommandRunner,
        &ProcMountTable::default(),
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
        &SystemCommandRunner,
        &ProcMountTable::default(),
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
        &SystemCommandRunner,
        &ProcMountTable::default(),
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
    verify_onedrive_offline_mirror_setup_with(connection, runner, mount_table, config_root).await
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
    verify_onedrive_offline_mirror_setup_with(connection, runner, mount_table, config_root).await
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
    let remotes = parse_rclone_remotes_for_app(&dump.stdout.text)?;
    if remotes.iter().any(|remote| remote.name == remote_name) {
        return Err(format!(
            "rclone remote `{remote_name}` already exists. Choose a different remote name or select the existing remote."
        ));
    }
    Ok(())
}

fn google_drive_rclone_config_create_request(
    setup: &GoogleDriveRemoteSetup,
) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .map_err(|error| error.to_string())?
        .arg("create")
        .map_err(|error| error.to_string())?
        .arg(&setup.name)
        .map_err(|error| error.to_string())?
        .arg("drive")
        .map_err(|error| error.to_string())?
        .arg("scope")
        .map_err(|error| error.to_string())?
        .arg("drive")
        .map_err(|error| error.to_string())?
        .arg("config_is_local")
        .map_err(|error| error.to_string())?
        .arg("true")
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
        .sensitive_arg(&setup.host)
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

fn config_storage() -> Result<cosmic_config::Config, String> {
    cosmic_config::Config::new(APP_ID, Config::VERSION)
        .map_err(|error| format!("Failed to open applet configuration storage: {error}"))
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
                Provider::GoogleDrive | Provider::Box | Provider::Smb => {
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
            Provider::GoogleDrive | Provider::Box | Provider::Smb => {
                let plan = rclone_bisync_plan(connection, &default_work_root())
                    .map_err(|error| error.to_string())?;
                let service =
                    UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
                let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
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
            Provider::GoogleDrive | Provider::Box | Provider::Smb,
            ConnectionMode::OnlineMount(_)
        )
    )
}

const fn is_rclone_offline_mirror(connection: &Connection) -> bool {
    matches!(
        (&connection.provider, &connection.mode),
        (
            Provider::GoogleDrive | Provider::Box | Provider::Smb,
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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

async fn install_rclone_offline_mirror_units(connection: Connection) -> String {
    let name = connection.name.clone();
    match install_rclone_offline_mirror_units_result(&connection).await {
        Ok(()) => {
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
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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

    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
        Provider::GoogleDrive | Provider::Box | Provider::Smb => {
            prepare_online_mount_runtime(connection)?;
        }
    }
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| error.to_string())?;
    let controller = UnitController::new(store, CommandSystemdManager::new(SystemCommandRunner));
    let cancellation = CancellationToken::new();
    let mut removed = 0usize;
    for unit in managed_unit_names_for_connection(connection) {
        let manager = CommandSystemdManager::new(SystemCommandRunner);
        let _ = manager
            .action(SystemdAction::Stop, Some(&unit), cancellation.child_token())
            .await;
        let manager = CommandSystemdManager::new(SystemCommandRunner);
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
            Provider::OneDrive => vec![UnitName::new(connection.id, UnitKind::Service)],
            Provider::GoogleDrive | Provider::Box | Provider::Smb => vec![
                UnitName::new(connection.id, UnitKind::Timer),
                UnitName::new(connection.id, UnitKind::Service),
            ],
        },
    }
}

async fn run_managed_online_mount_operation(
    connection: Connection,
    operation: Operation,
) -> String {
    let label = operation_label(operation);
    match run_managed_online_mount_operation_result(&connection, operation).await {
        Ok(()) => format!("{label} completed for {}.", connection.name),
        Err(error) => format!("{label} failed for {}: {error}", connection.name),
    }
}

async fn run_managed_online_mount_operation_result(
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
    prepare_online_mount_runtime(connection)?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
        let _ = manager
            .action(
                SystemdAction::ResetFailed,
                Some(&unit),
                cancellation.child_token(),
            )
            .await;
    }
    manager
        .action(action, Some(&unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn run_managed_onedriver_online_mount_operation(
    connection: Connection,
    operation: Operation,
) -> String {
    let label = operation_label(operation);
    match run_managed_onedriver_online_mount_operation_result(&connection, operation).await {
        Ok(()) => format!("{label} completed for {}.", connection.name),
        Err(error) => format!("{label} failed for {}: {error}", connection.name),
    }
}

async fn run_managed_onedriver_online_mount_operation_result(
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
    prepare_onedriver_online_mount_runtime(connection)?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
        let _ = manager
            .action(
                SystemdAction::ResetFailed,
                Some(&unit),
                cancellation.child_token(),
            )
            .await;
    }
    manager
        .action(action, Some(&unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn run_online_mount_repair_operation(connection: Connection) -> String {
    match run_online_mount_repair_operation_result(&connection).await {
        Ok(()) => format!(
            "Repair completed for {}. Lazy unmount recovery detached the mountpoint and reset the generated service state.",
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
    let cancellation = CancellationToken::new();

    let _ = manager
        .action(SystemdAction::Stop, Some(&unit), cancellation.child_token())
        .await;

    SystemCommandRunner
        .run(
            lazy_unmount_request(&connection.local_path).map_err(|error| error.to_string())?,
            cancellation.child_token(),
        )
        .await
        .map_err(|error| error.to_string())?;

    manager
        .action(SystemdAction::ResetFailed, Some(&unit), cancellation)
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
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
    prepare_offline_mirror_runtime(connection)?;
    let plan =
        rclone_bisync_plan(connection, &default_work_root()).map_err(|error| error.to_string())?;
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
    if !initial_sync_marker(plan).exists() {
        return Err(
            "background sync requires a successful Preview and confirmed initial Sync Now first"
                .into(),
        );
    }
    ensure_background_sync_may_start(connection).await?;
    let timer = UnitDocument::timer(&plan.timer).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    match SystemCommandRunner
        .run(request, CancellationToken::new())
        .await
    {
        Ok(output) => {
            let state = output.stdout.text.to_ascii_lowercase();
            state.contains("connected")
        }
        Err(_) => true,
    }
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
    match SystemCommandRunner
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
    let config = Config::load().config;
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
    let probe = CommandReadinessProbe::new(SystemCommandRunner);
    readiness_report(&probe, &profile.readiness_checks, CancellationToken::new())
        .await
        .map(|report| report.ready)
        .map_err(|error| error.to_string())
}

async fn stop_onedrive_offline_mirror_background(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<String, String> {
    let service = UnitDocument::service(&plan.service).map_err(|error| error.to_string())?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
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
    let output = SystemCommandRunner
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
    let output = SystemCommandRunner
        .run(
            one_drive_preview_request(plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| offline_mirror_command_error("OneDrive preview", error))?;
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
    let output = SystemCommandRunner
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

    let output = SystemCommandRunner
        .run(
            one_drive_sync_request(plan).map_err(|error| error.to_string())?,
            CancellationToken::new(),
        )
        .await
        .map_err(|error| offline_mirror_command_error("OneDrive sync", error))?;
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
    let filter_content = if connection.provider == Provider::GoogleDrive {
        google_native_filter_file()
    } else {
        "+ **\n".into()
    };
    fs::write(&plan.filters_file, filter_content).map_err(|error| {
        format!(
            "failed to write rclone bisync filter file {}: {error}",
            plan.filters_file.display()
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
        format!("COSMIC Cloud Mounter initial preview completed.\n{summary}\n"),
    )
    .map_err(|error| format!("failed to record initial preview completion: {error}"))
}

fn write_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::RcloneBisyncPlan,
) -> Result<(), String> {
    fs::write(
        initial_sync_marker(plan),
        "COSMIC Cloud Mounter initial sync completed.\n",
    )
    .map_err(|error| format!("failed to record initial sync completion: {error}"))
}

fn write_onedrive_initial_preview_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
    summary: &str,
) -> Result<(), String> {
    fs::write(
        onedrive_initial_preview_marker(plan),
        format!("COSMIC Cloud Mounter OneDrive initial preview completed.\n{summary}\n"),
    )
    .map_err(|error| format!("failed to record OneDrive initial preview completion: {error}"))
}

fn write_onedrive_initial_sync_marker(
    plan: &cosmic_ext_applet_mounter::sync::OneDriveMirrorPlan,
) -> Result<(), String> {
    fs::write(
        onedrive_initial_sync_marker(plan),
        "COSMIC Cloud Mounter OneDrive initial sync completed.\n",
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
            if detail.is_empty() {
                format!("rclone {stage} failed without diagnostic output")
            } else {
                format!("rclone {stage} failed: {detail}")
            }
        }
    }
}

async fn test_connection_plan_and_access(connection: Connection) -> String {
    let plan_summary = match managed_plan_summary(&connection) {
        Ok(summary) => summary,
        Err(error) => return format!("Draft test failed for {}: {error}", connection.name),
    };

    match connection.provider {
        Provider::GoogleDrive | Provider::Box | Provider::Smb => {
            match verify_rclone_access(&connection).await {
                Ok(access_summary) => {
                    format!(
                        "Draft test passed for {}. {plan_summary} {access_summary}",
                        connection.name
                    )
                }
                Err(error) => {
                    format!("Draft test failed for {}: {error}", connection.name)
                }
            }
        }
        Provider::OneDrive => match connection.mode {
            ConnectionMode::OnlineMount(_) => {
                match verify_onedriver_online_mount_setup(&connection) {
                    Ok(access_summary) => {
                        format!(
                            "Draft test passed for {}. {plan_summary} {access_summary}",
                            connection.name
                        )
                    }
                    Err(error) => {
                        format!("Draft test failed for {}: {error}", connection.name)
                    }
                }
            }
            ConnectionMode::OfflineMirror(_) => {
                match verify_onedrive_offline_mirror_setup(&connection).await {
                    Ok(access_summary) => {
                        format!(
                            "Draft test passed for {}. {plan_summary} {access_summary}",
                            connection.name
                        )
                    }
                    Err(error) => {
                        format!("Draft test failed for {}: {error}", connection.name)
                    }
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
        &SystemCommandRunner,
        &ProcMountTable::default(),
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
        &SystemCommandRunner,
        &ProcMountTable::default(),
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
            "onedrive {stage} timed out after {} seconds. Check network/VPN readiness and Microsoft OneDrive responsiveness.",
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
    let expected_backend = rclone_backend_name(connection.provider)
        .ok_or_else(|| "selected provider does not use rclone".to_owned())?;
    let provider = CommandRcloneProvider::new(SystemCommandRunner);
    provider
        .validate_remote(connection, CancellationToken::new())
        .await
        .map_err(|error| rclone_remote_validation_error(connection, error))?;

    let target = rclone_access_target(connection);
    let output = SystemCommandRunner
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
        .map_err(|error| rclone_access_error(&target, error))?;
    let visible_items = output.stdout.text.lines().count();
    Ok(format!(
        "Rclone remote `{}` exists, backend `{expected_backend}` matches {}, and `{target}` is accessible with {visible_items} visible item(s) at depth 1.",
        connection.remote_reference,
        provider_label(connection.provider)
    ))
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
    format!(
        "{} {} -> {}\nRemote subtree: {}\nStart at login: {}\nUnsupported options: {unsupported}\nStatus: {status}. Import replacement still requires explicit confirmation.",
        provider_label(preview.provider),
        preview.remote_reference,
        preview.local_target.display(),
        preview.remote_subpath.as_deref().unwrap_or("Whole remote"),
        yes_no(preview.start_at_login),
    )
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn default_cache_root() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

fn default_config_root() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter")
}

fn default_work_root() -> PathBuf {
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
    use cosmic_ext_applet_mounter::vpn::NetworkManagerVpnProfile;

    #[test]
    fn popup_scroll_height_accounts_for_notice_length() {
        let empty = popup_connection_scroll_height(None, 0, true);
        let short_list = popup_connection_scroll_height(None, 3, false);
        let full_list = popup_connection_scroll_height(None, 20, false);
        let with_short_notice =
            popup_connection_scroll_height(Some("Preview completed."), 3, false);
        let with_long_notice = popup_connection_scroll_height(
            Some(
                "Preview completed for OneDrive primary auth live verify. Preview completed; onedrive reported 1 notable file-change line. Initial sync has not run yet; press Sync Now to confirm and run the initial synchronization.",
            ),
            3,
            false,
        );

        assert_eq!(empty, POPUP_EMPTY_ROW_HEIGHT);
        assert_eq!(short_list, 3.0 * POPUP_CONNECTION_ROW_HEIGHT);
        assert_eq!(full_list, POPUP_SCROLL_MAX_HEIGHT);
        assert!(with_short_notice > short_list);
        assert!(with_long_notice > with_short_notice);
        assert!(with_long_notice <= POPUP_SCROLL_MAX_HEIGHT);
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
    fn runtime_systemd_status_parser_recognizes_active_disabled_units() {
        let status = parse_runtime_systemd_status(
            "ActiveState=active\nSubState=running\nUnitFileState=disabled\n",
        );

        assert_eq!(status.active, ActiveState::Active);
        assert!(!status.enabled);
        assert_eq!(status.detail, "running");
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
                    name: "ua_engr".into(),
                    backend: "smb".into(),
                },
            ]
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
    fn smb_remote_setup_builds_redacted_noninteractive_request() {
        let mut draft = ConnectionDraft {
            provider: Provider::Smb,
            remote_reference: "test_smb".into(),
            smb_host: "files.example.edu".into(),
            smb_user: "uutzinger".into(),
            smb_domain: "UA".into(),
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

        draft.smb_user.clear();
        draft.smb_domain.clear();
        let setup = SmbRemoteSetup::from_draft(&draft).expect("optional user/domain");
        assert_eq!(setup.user, None);
        assert_eq!(setup.domain, None);
    }

    #[test]
    fn smb_remote_setup_rejects_missing_host_and_bad_name() {
        let mut draft = ConnectionDraft {
            provider: Provider::Smb,
            remote_reference: "test_smb".into(),
            smb_host: String::new(),
            ..ConnectionDraft::default()
        };

        let error = SmbRemoteSetup::from_draft(&draft).expect_err("missing host must fail");
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
            "rclone config create test_drive drive scope drive config_is_local true --non-interactive"
        );
        assert_eq!(request.timeout, Duration::from_secs(5 * 60));
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
        let draft = ConnectionDraft {
            provider: Provider::GoogleDrive,
            remote_reference: "bad remote".into(),
            ..ConnectionDraft::default()
        };

        let error = GoogleDriveRemoteSetup::from_draft(&draft).expect_err("bad name must fail");
        assert!(error.contains("rclone remote name"));
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
    }

    #[tokio::test]
    async fn create_google_drive_remote_blocks_duplicates_and_uses_fixed_commands() {
        let duplicate_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        duplicate_runner.push(Ok(command_output(r#"{"existing": {"type": "drive"}}"#)));
        let duplicate = GoogleDriveRemoteSetup {
            name: "existing".into(),
        };

        let error = create_google_drive_rclone_remote_with(&duplicate_runner, duplicate)
            .await
            .expect_err("duplicate must fail");
        assert!(error.contains("already exists"));
        let requests = duplicate_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("{}")));
        runner.push(Ok(command_output("")));
        let setup = GoogleDriveRemoteSetup {
            name: "new_drive".into(),
        };

        let created = create_google_drive_rclone_remote_with(&runner, setup)
            .await
            .expect("new remote should be created");
        assert_eq!(created, "new_drive");
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");
        assert_eq!(
            requests[1].sanitized_command(),
            "rclone config create new_drive drive scope drive config_is_local true --non-interactive"
        );
    }

    #[tokio::test]
    async fn create_smb_remote_blocks_duplicates_and_uses_fixed_commands() {
        let duplicate_runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        duplicate_runner.push(Ok(command_output(r#"{"existing": {"type": "smb"}}"#)));
        let duplicate = SmbRemoteSetup {
            name: "existing".into(),
            host: "files.example.edu".into(),
            user: Some("uutzinger".into()),
            domain: Some("UA".into()),
        };

        let error = create_smb_rclone_remote_with(&duplicate_runner, duplicate)
            .await
            .expect_err("duplicate must fail");
        assert!(error.contains("already exists"));
        let requests = duplicate_runner.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let runner = cosmic_ext_applet_mounter::process::FakeCommandRunner::default()
            .with_resolved([Executable::Rclone]);
        runner.push(Ok(command_output("{}")));
        runner.push(Ok(command_output("")));
        let setup = SmbRemoteSetup {
            name: "new_smb".into(),
            host: "files.example.edu".into(),
            user: Some("uutzinger".into()),
            domain: Some("UA".into()),
        };

        let created = create_smb_rclone_remote_with(&runner, setup)
            .await
            .expect("new remote should be created");
        assert_eq!(created, "new_smb");
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "rclone config dump");

        let command = requests[1].sanitized_command();
        assert!(command.contains("rclone config create new_smb smb"));
        assert!(command.contains(" --non-interactive"));
        assert!(!command.contains("files.example.edu"));
        assert!(!command.contains("uutzinger"));
        assert!(!command.contains("UA"));
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
    fn offline_mirror_preview_summary_reports_confirmation_context() {
        let output = command_output(
            "Path2 to Path1: copy local.txt\nPath1 to Path2: copy remote.txt\nDelete old.txt\nConflict same.txt\nSkipped native.gdoc\nTransferred: 15 MiB\n",
        );

        let summary = preview_summary_text(&output);

        assert!(summary.contains("uploads 1"));
        assert!(summary.contains("downloads 2"));
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
    fn onedrive_setup_guidance_matches_modes_and_save_notice_mentions_validation() {
        assert!(onedrive_setup_guidance(AccessMode::OnlineMount).contains("jstaf/onedriver"));
        assert!(onedrive_setup_guidance(AccessMode::OfflineMirror).contains("abraunegg/onedrive"));
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
