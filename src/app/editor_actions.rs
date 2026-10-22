// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditorAction {
    Detect,
    Connect,
    GoogleOAuth,
    Test,
    Save,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Blue,
    Grey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Availability {
    Ready,
    Blocked(String),
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionState {
    tone: Tone,
    availability: Availability,
}

impl ActionState {
    fn enabled(&self) -> bool {
        self.availability == Availability::Ready
    }
}

#[derive(Debug, Clone)]
pub(super) struct EditorRequest {
    sequence: u64,
    revision: u64,
    action: EditorAction,
    tone: Tone,
}

// Only non-secret setup values are retained for comparison. Transient secrets
// are checked as update intent; they are never copied into this baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SetupInputs {
    provider: Provider,
    remote: String,
    values: Vec<String>,
}

impl SetupInputs {
    pub(super) fn from_draft(draft: &ConnectionDraft) -> Self {
        let values = match draft.provider {
            Provider::Smb => vec![
                draft.smb_host.trim().to_owned(),
                draft.smb_user.trim().to_owned(),
                draft.smb_domain.trim().to_owned(),
            ],
            Provider::Sftp => draft.sftp.setup_values(),
            Provider::OneDrive => vec![
                format!("{:?}", draft.onedrive_drive_type),
                draft.onedrive_expected_account.trim().to_owned(),
            ],
            Provider::Teams => vec![draft.sharepoint_library_url.trim().to_owned()],
            Provider::GoogleDrive | Provider::Box => Vec::new(),
        };
        Self {
            provider: draft.provider,
            remote: draft.remote_reference.trim().to_owned(),
            values,
        }
    }
}

impl AppModel {
    fn editor_saved(&self) -> Option<&Connection> {
        self.draft
            .id
            .and_then(|id| self.config.document.connections.iter().find(|c| c.id == id))
    }

    pub(super) fn editor_connection(&self) -> Result<Connection, String> {
        let mut draft = self.draft.clone();
        // Keep the identity stable even before Add assigns its first ID.
        if draft.id.is_none() {
            draft.id = self.validated_draft.as_ref().map(|v| v.connection.id);
        }
        connection_from_draft(&draft)
    }

    fn editor_changed(&self) -> bool {
        match (self.editor_saved(), self.editor_connection()) {
            (Some(saved), Ok(current)) => *saved != current,
            _ => true,
        }
    }

    pub(super) fn editor_disable_only(&self) -> bool {
        let (Some(saved), Ok(mut current)) = (self.editor_saved(), self.editor_connection()) else {
            return false;
        };
        if !saved.enabled || current.enabled {
            return false;
        }
        current.enabled = true;
        current == *saved
    }

    fn editor_test_passed(&self) -> bool {
        self.validated_draft.as_ref().is_some_and(|validated| {
            self.editor_connection()
                .is_ok_and(|current| current == validated.connection)
        })
    }

    pub(super) fn editor_remote_update_pending(&self) -> bool {
        match self.draft.provider {
            Provider::GoogleDrive => {
                !self.draft.google_client_id.is_empty()
                    || !self.draft.google_client_secret.is_empty()
            }
            Provider::Smb if !self.draft.smb_password.is_empty() => true,
            Provider::Sftp
                if self.draft.sftp.has_secrets() || self.sftp_retry_secret_auth.is_some() =>
            {
                true
            }
            _ => {
                let current = SetupInputs::from_draft(&self.draft);
                let baseline = self.editor_setup_baseline.clone().or_else(|| {
                    self.editor_saved()
                        .map(|saved| SetupInputs::from_draft(&draft_from_connection(saved)))
                });
                baseline.is_some_and(|baseline| {
                    baseline.provider == current.provider
                        && baseline.remote == current.remote
                        && baseline.values != current.values
                })
            }
        }
    }

    fn editor_setup_error(&self) -> Option<String> {
        match self.draft.provider {
            Provider::GoogleDrive => GoogleDriveRemoteSetup::from_draft(&self.draft)
                .err()
                .or_else(|| {
                    (matches!(self.window_mode, WindowMode::ModifyConnection(_))
                        && (self.draft.google_client_id.trim().is_empty()
                            || self.draft.google_client_secret.trim().is_empty()))
                    .then(|| fl!("editor-oauth-required"))
                }),
            Provider::Box => BoxRemoteSetup::from_draft(&self.draft).err(),
            Provider::Smb => SmbRemoteSetup::from_draft(&self.draft).err(),
            Provider::Sftp => self.draft.sftp.setup_error(&self.draft.remote_reference),
            Provider::Teams => SharePointSetup::new(
                &self.draft.remote_reference,
                &self.draft.sharepoint_library_url,
            )
            .err(),
            Provider::OneDrive if draft_is_onedrive_rclone(&self.draft) => {
                let mut draft = self.draft.clone();
                draft.id.get_or_insert_with(ConnectionId::new);
                OneDriveEditorRequest::from_draft(&draft, &self.config.document.connections, true)
                    .err()
            }
            Provider::OneDrive => self.editor_connection().err(),
        }
    }

    fn editor_action_state(&self, action: EditorAction) -> ActionState {
        let adding = self.window_mode == WindowMode::AddConnection;
        let changed = self.editor_changed();
        let pending_update = self.editor_remote_update_pending();
        let existing_remote = self
            .matching_rclone_remotes(self.draft.provider)
            .iter()
            .any(|r| r.name == self.draft.remote_reference.trim());
        let tone = match action {
            EditorAction::Detect => Tone::Blue,
            EditorAction::GoogleOAuth => Tone::Grey,
            EditorAction::Connect
                if !adding
                    && matches!(self.draft.provider, Provider::GoogleDrive | Provider::Box) =>
            {
                Tone::Grey
            }
            EditorAction::Connect
                if pending_update
                    || (adding && !existing_remote)
                    || (self.draft.provider == Provider::Sftp
                        && self.sftp_action_highlighted(adding)) =>
            {
                Tone::Blue
            }
            EditorAction::Test
                if (adding || changed || self.editor_test_required)
                    && !self.editor_test_passed() =>
            {
                Tone::Blue
            }
            EditorAction::Save if adding || changed => Tone::Blue,
            _ => Tone::Grey,
        };
        if let Some(request) = &self.editor_request {
            return ActionState {
                tone: if request.action == action {
                    request.tone
                } else {
                    tone
                },
                availability: if request.action == action {
                    Availability::Running
                } else {
                    Availability::Blocked(fl!("editor-action-busy"))
                },
            };
        }
        let reason = if self.removing_connection.is_some() || self.removing_rclone_remote.is_some()
        {
            Some(fl!("editor-action-busy"))
        } else if self.editor_prefill_pending && action != EditorAction::Detect {
            Some(fl!("editor-details-loading"))
        } else {
            match action {
                EditorAction::Detect => None,
                EditorAction::Connect if self.draft.legacy_onedrive => {
                    Some(fl!("onedrive-legacy-engine-retired"))
                }
                EditorAction::Connect
                    if !adding && self.draft.provider == Provider::GoogleDrive =>
                {
                    Some(fl!("google-drive-reconnect-unavailable-help"))
                }
                EditorAction::Connect if !adding && self.draft.provider == Provider::Box => {
                    Some(fl!("box-reconnect-unavailable-help"))
                }
                EditorAction::Connect | EditorAction::GoogleOAuth => self.editor_setup_error(),
                EditorAction::Test | EditorAction::Save if self.draft.legacy_onedrive => {
                    Some(fl!("onedrive-legacy-engine-retired"))
                }
                EditorAction::Test | EditorAction::Save
                    if pending_update
                        && !matches!(self.draft.provider, Provider::OneDrive | Provider::Teams) =>
                {
                    Some(fl!("editor-apply-settings-first"))
                }
                EditorAction::Test if draft_is_onedrive_rclone(&self.draft) => {
                    // Test itself supplies OneDrive identity verification.
                    self.editor_connection()
                        .err()
                        .or_else(|| self.editor_setup_error())
                }
                EditorAction::Test => self.editor_connection().err(),
                EditorAction::Save if !changed && !adding => Some(fl!("editor-no-changes")),
                EditorAction::Save if self.editor_disable_only() => None,
                EditorAction::Save if !self.editor_test_passed() => Some(fl!("editor-test-first")),
                EditorAction::Save => self.editor_connection().err(),
            }
        };
        ActionState {
            tone,
            availability: reason.map_or(Availability::Ready, Availability::Blocked),
        }
    }

    pub(super) fn editor_button(
        &self,
        action: EditorAction,
        label: String,
        message: Message,
    ) -> Element<'static, Message> {
        let state = self.editor_action_state(action);
        let label = self.editor_button_label(action, label);
        let button = match state.tone {
            Tone::Blue => widget::button::suggested(label),
            Tone::Grey => widget::button::standard(label),
        }
        .on_press_maybe(state.enabled().then_some(message));
        match state.availability {
            Availability::Blocked(reason) => field_with_help(button, reason),
            _ => button.into(),
        }
    }

    fn editor_button_label(&self, action: EditorAction, label: String) -> String {
        let state = self.editor_action_state(action);
        if state.availability == Availability::Running {
            match action {
                EditorAction::Detect => fl!("editor-detecting"),
                EditorAction::Connect => fl!("editor-connecting"),
                EditorAction::GoogleOAuth => fl!("editor-updating-oauth"),
                EditorAction::Test => fl!("editor-testing"),
                EditorAction::Save => fl!("editor-saving"),
            }
        } else if action == EditorAction::Save
            && self.pending_shared_remote_ack.is_some()
            && self.pending_shared_remote_ack == self.draft.id
        {
            fl!("editor-confirm-save")
        } else {
            label
        }
    }

    fn editor_runtime_block_reason(
        &self,
        id: ConnectionId,
        operation: Operation,
    ) -> Option<String> {
        if self.editor_changed() || self.editor_remote_update_pending() {
            return Some(fl!("editor-runtime-unsaved"));
        }
        if self.editor_request.is_some()
            || self.runtime_pending
            || self.runtime_operation_wait.is_some()
            || self.mirror_operations_pending.contains(&id)
            || self.unit_operations_waiting.contains_key(&id)
        {
            return Some(fl!("editor-action-busy"));
        }
        let state = self.view_state();
        let Some(row) = state.rows.iter().find(|row| row.id == id) else {
            return Some(fl!("notice-connection-unavailable"));
        };
        let decision = decide_operation(row, operation);
        if decision.allowed {
            None
        } else {
            Some(
                decision
                    .reason
                    .unwrap_or_else(|| fl!("operation-unavailable-state")),
            )
        }
    }

    pub(super) fn editor_runtime_button(
        &self,
        id: ConnectionId,
        operation: Operation,
    ) -> Element<'static, Message> {
        let reason = self.editor_runtime_block_reason(id, operation);
        let running = self
            .runtime_operation_wait
            .as_ref()
            .is_some_and(|wait| wait.connection_id == id && wait.operation == operation)
            || self.runtime_routed_operations.get(&id) == Some(&operation);
        let label = if running {
            fl!(
                "editor-operation-running",
                action = operation_label(operation)
            )
        } else {
            operation_label(operation)
        };
        let button = widget::button::standard(label).on_press_maybe(
            reason
                .is_none()
                .then_some(Message::OperationRequested(id, operation)),
        );
        if let Some(reason) = reason {
            field_with_help(button, reason)
        } else {
            button.into()
        }
    }

    fn editor_request_revision(&self, action: EditorAction) -> u64 {
        match action {
            EditorAction::Connect | EditorAction::GoogleOAuth => self.editor_setup_revision,
            EditorAction::Detect => 0,
            EditorAction::Test | EditorAction::Save => self.editor_revision,
        }
    }

    fn track_editor_task(
        &self,
        sequence: u64,
        task: Task<cosmic::Action<Message>>,
    ) -> Task<cosmic::Action<Message>> {
        let kind = self.editor_request.as_ref().map(|request| request.action);
        task.map(move |action| match action {
            cosmic::Action::App(message) if tracked_completion(kind, &message) => {
                cosmic::Action::App(Message::EditorActionFinished(sequence, Box::new(message)))
            }
            other => other,
        })
    }

    // The same policy guards messages as buttons. The envelope tags every async
    // completion, including errors and intermediate verify/save continuations.
    pub(super) fn update_message(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        if let Message::OperationRequested(id, operation) = &message
            && matches!(self.window_mode, WindowMode::ModifyConnection(current) if current == *id)
            && matches!(
                operation,
                Operation::PreviewInitialSync | Operation::SyncNow
            )
            && let Some(reason) = self.editor_runtime_block_reason(*id, *operation)
        {
            self.last_notice = Some(reason);
            return Task::none();
        }
        if matches!(message, Message::RemoveConnection(_)) && self.editor_request.is_some() {
            return Task::none();
        }
        if let Message::EditorActionFinished(sequence, message) = message {
            let Some(request) = self
                .editor_request
                .clone()
                .filter(|r| r.sequence == sequence)
            else {
                return Task::none();
            };
            let stale = request.revision != self.editor_request_revision(request.action);
            let cancelled = self
                .onedrive_editor_pending
                .as_ref()
                .is_some_and(|p| p.cancellation.is_cancelled());
            if stale || cancelled {
                self.editor_request = None;
                self.sftp_setup_pending = false;
                self.sharepoint_setup_pending = false;
                self.onedrive_editor_pending = None;
                self.validated_draft = None;
                self.last_notice = Some(fl!("editor-result-obsolete"));
                // Saving may already have persisted configuration/installed units.
                // Preserve the edited draft, but still notify the runtime owner.
                return if request.action == EditorAction::Save
                    && matches!(message.as_ref(), Message::DraftSaved(_))
                {
                    notify_runtime_task()
                } else {
                    Task::none()
                };
            }
            let continuing = matches!(
                (&request.action, message.as_ref()),
                (
                    EditorAction::Test,
                    Message::OneDriveRcloneCompleted(_, _, Ok(_))
                ) | (EditorAction::Save, Message::SaveDraftValidated(_, Ok(_)))
            );
            let setup_succeeded = matches!(
                message.as_ref(),
                Message::GoogleDriveRcloneRemoteApplied(Ok(_))
                    | Message::BoxRcloneRemoteCreated(Ok(_))
                    | Message::SmbRcloneRemoteCreated(Ok(_))
                    | Message::SftpRemoteApplied(_, _, _, Ok(_))
                    | Message::SharePointRcloneRemoteCreated(Ok(_))
                    | Message::OneDriveRcloneCompleted(_, _, Ok(_))
                    | Message::OnedriverSetupCompleted(Ok(_))
            );
            if !continuing {
                self.editor_request = None;
            }
            if let Message::DraftTested(_, result) = message.as_ref() {
                self.editor_test_required = result.is_err();
            }
            let task = self.dispatch_message(*message);
            if setup_succeeded {
                self.editor_test_required = true;
                self.editor_setup_baseline = Some(SetupInputs::from_draft(&self.draft));
            }
            if continuing {
                if task.units() == 0 {
                    self.editor_request = None;
                } else {
                    return self.track_editor_task(sequence, task);
                }
            }
            return task;
        }
        if matches!(
            message,
            Message::DraftRemote(_) | Message::DraftProvider(_) | Message::DraftAccessMode(_)
        ) {
            self.editor_setup_baseline = None;
            self.editor_prefill_pending = false;
        } else if edits_draft(&message)
            && self.editor_setup_baseline.as_ref().is_none_or(|baseline| {
                baseline.provider != self.draft.provider
                    || baseline.remote != self.draft.remote_reference.trim()
            })
        {
            self.editor_setup_baseline = Some(SetupInputs::from_draft(&self.draft));
        }
        if edits_setup(&message) {
            self.editor_setup_revision = self.editor_setup_revision.wrapping_add(1);
        }
        if edits_draft(&message) || matches!(message, Message::CancelOneDriveRclone) {
            self.pending_shared_remote_ack = None;
        }
        if edits_draft(&message) {
            self.editor_revision = self.editor_revision.wrapping_add(1);
            self.validated_draft = None;
        }
        let action = action_for_message(&message, self.window_mode);
        if let Some(action) = action {
            let state = self.editor_action_state(action);
            if !state.enabled() {
                if let Availability::Blocked(reason) = state.availability {
                    self.last_notice = Some(reason);
                }
                return Task::none();
            }
            if matches!(action, EditorAction::Test | EditorAction::Save) {
                self.draft.id.get_or_insert_with(ConnectionId::new);
            }
            if action != EditorAction::Detect {
                self.validated_draft = if action == EditorAction::Save {
                    self.validated_draft.take()
                } else {
                    None
                };
            }
            let task = self.dispatch_message(message);
            if task.units() > 0 {
                self.editor_sequence = self.editor_sequence.wrapping_add(1);
                self.editor_request = Some(EditorRequest {
                    sequence: self.editor_sequence,
                    revision: self.editor_request_revision(action),
                    action,
                    tone: state.tone,
                });
                return self.track_editor_task(self.editor_sequence, task);
            }
            return task;
        }
        self.dispatch_message(message)
    }
}

fn action_for_message(message: &Message, mode: WindowMode) -> Option<EditorAction> {
    Some(match message {
        Message::DetectRcloneRemotes => EditorAction::Detect,
        Message::ApplyGoogleDriveRcloneRemote
            if matches!(mode, WindowMode::ModifyConnection(_)) =>
        {
            EditorAction::GoogleOAuth
        }
        Message::ApplyGoogleDriveRcloneRemote
        | Message::CreateBoxRcloneRemote
        | Message::CreateSmbRcloneRemote
        | Message::ApplySftpRemote
        | Message::CreateSharePointRcloneRemote
        | Message::StartOnedriverSetup
        | Message::ConnectOneDriveRclone => EditorAction::Connect,
        Message::TestDraft => EditorAction::Test,
        Message::SaveDraft => EditorAction::Save,
        _ => return None,
    })
}

fn edits_draft(message: &Message) -> bool {
    matches!(
        message,
        Message::DraftProvider(_)
            | Message::DraftAccessMode(_)
            | Message::DraftName(_)
            | Message::DraftRemote(_)
            | Message::DraftSubpath(_)
            | Message::DraftSharePointLibraryUrl(_)
            | Message::DraftGoogleClientId(_)
            | Message::DraftGoogleClientSecret(_)
            | Message::DraftSftp(..)
            | Message::DraftSftpAuth(_)
            | Message::DraftSmbHost(_)
            | Message::DraftSmbUser(_)
            | Message::DraftSmbDomain(_)
            | Message::DraftSmbPassword(_)
            | Message::DraftLocalPath(_)
            | Message::DraftRecoveryDirectory(_)
            | Message::DraftCacheLimit(_)
            | Message::DraftSyncInterval(_)
            | Message::DraftEnabled(_)
            | Message::DraftSyncOnMetered(_)
            | Message::DraftConnectionPreloadUseGlobal(_)
            | Message::DraftConnectionPreloadEnabled(_)
            | Message::DraftConnectionPreloadSeconds(_)
            | Message::DraftConnectionPreloadDepth(_)
            | Message::DraftVpn(_)
            | Message::DraftDisconnectVpn(_)
            | Message::DraftOneDriveType(_)
            | Message::DraftOneDriveAccount(_)
            | Message::LocalFolderPicked(Ok(Some(_)))
            | Message::RecoveryFolderPicked(Ok(Some(_)))
    )
}

fn tracked_completion(action: Option<EditorAction>, message: &Message) -> bool {
    match action {
        Some(EditorAction::Detect) => matches!(message, Message::RcloneInspectionCompleted(..)),
        Some(EditorAction::Test) => matches!(
            message,
            Message::DraftTested(..) | Message::OneDriveRcloneCompleted(..)
        ),
        Some(EditorAction::Save) => matches!(
            message,
            Message::SaveDraftValidated(..) | Message::DraftSaved(..) | Message::EditorNotified(..)
        ),
        Some(EditorAction::Connect | EditorAction::GoogleOAuth) => matches!(
            message,
            Message::GoogleDriveRcloneRemoteApplied(..)
                | Message::BoxRcloneRemoteCreated(..)
                | Message::SmbRcloneRemoteCreated(..)
                | Message::SftpRemoteApplied(..)
                | Message::SharePointRcloneRemoteCreated(..)
                | Message::OneDriveRcloneCompleted(..)
                | Message::OnedriverSetupCompleted(..)
        ),
        None => false,
    }
}

fn edits_setup(message: &Message) -> bool {
    matches!(
        message,
        Message::DraftProvider(_)
            | Message::DraftAccessMode(_)
            | Message::DraftRemote(_)
            | Message::DraftSharePointLibraryUrl(_)
            | Message::DraftGoogleClientId(_)
            | Message::DraftGoogleClientSecret(_)
            | Message::DraftSftp(..)
            | Message::DraftSftpAuth(_)
            | Message::DraftSmbHost(_)
            | Message::DraftSmbUser(_)
            | Message::DraftSmbDomain(_)
            | Message::DraftSmbPassword(_)
            | Message::DraftOneDriveType(_)
            | Message::DraftOneDriveAccount(_)
            | Message::DraftSubpath(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(modifying: bool) -> AppModel {
        let mut app = AppModel {
            draft: ConnectionDraft {
                id: Some(ConnectionId::new()),
                name: "Synthetic connection".into(),
                remote_reference: "example_remote".into(),
                local_path: "/tmp/synthetic-target".into(),
                ..ConnectionDraft::default()
            },
            ..AppModel::default()
        };
        if modifying {
            let saved = connection_from_draft(&app.draft).unwrap();
            app.window_mode = WindowMode::ModifyConnection(saved.id);
            app.config.document.connections.push(saved);
        }
        app
    }

    fn arm(app: &mut AppModel, action: EditorAction) -> u64 {
        let tone = app.editor_action_state(action).tone;
        app.editor_sequence += 1;
        app.editor_request = Some(EditorRequest {
            action,
            tone,
            sequence: app.editor_sequence,
            revision: app.editor_request_revision(action),
        });
        app.editor_sequence
    }

    fn complete_test(app: &mut AppModel, result: Result<String, String>) {
        let current = app.editor_connection().unwrap();
        let sequence = arm(app, EditorAction::Test);
        let _ = app.update_message(Message::EditorActionFinished(
            sequence,
            Box::new(Message::DraftTested(current, result)),
        ));
    }

    #[test]
    fn editor_actions_add_existing_remote_can_test_without_detect_or_connect() {
        let mut app = editor(false);
        assert!(app.rclone_remotes.is_empty());
        assert!(app.editor_action_state(EditorAction::Test).enabled());
        let save = app.editor_action_state(EditorAction::Save);
        assert_eq!(save.tone, Tone::Blue);
        assert!(!save.enabled());
        complete_test(&mut app, Ok("Synthetic pass".into()));
        assert!(app.editor_action_state(EditorAction::Save).enabled());
        let _ = app.update_message(Message::DraftSubpath("other-folder".into()));
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
        assert_eq!(app.editor_action_state(EditorAction::Test).tone, Tone::Blue);
        assert_eq!(
            app.editor_action_state(EditorAction::Connect).tone,
            Tone::Blue
        );
    }

    #[test]
    fn editor_actions_modify_save_needs_changes_and_matching_test() {
        let mut app = editor(true);
        assert_eq!(app.editor_action_state(EditorAction::Test).tone, Tone::Grey);
        complete_test(&mut app, Ok("Synthetic pass".into()));
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
        assert_eq!(app.editor_action_state(EditorAction::Save).tone, Tone::Grey);
        let original = app.draft.name.clone();
        let _ = app.update_message(Message::DraftName("Renamed".into()));
        assert_eq!(app.editor_action_state(EditorAction::Test).tone, Tone::Blue);
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
        complete_test(&mut app, Ok("Synthetic pass".into()));
        assert!(app.editor_action_state(EditorAction::Save).enabled());
        let _ = app.update_message(Message::DraftName(original));
        assert!(!app.editor_changed());
        assert!(!app.editor_test_passed());
    }

    #[test]
    fn editor_actions_sharepoint_whole_library_edit_needs_matching_test_before_save() {
        let mut app = editor(false);
        app.draft.provider = Provider::Teams;
        app.draft.access_mode = AccessMode::OfflineMirror;
        app.draft.remote_subpath = "Reports".into();
        app.draft.sharepoint_library_url =
            "https://example.sharepoint.com/sites/Work/Documents".into();
        app.draft.teams_drive_id = "b!verified".into();
        app.draft.recovery_directory = "/tmp/synthetic-recovery".into();
        let saved = connection_from_draft(&app.draft).unwrap();
        app.window_mode = WindowMode::ModifyConnection(saved.id);
        app.config.document.connections.push(saved);
        let _ = app.update_message(Message::DraftSubpath(String::new()));
        let root = app.editor_connection().unwrap();
        assert_eq!(root.remote_subpath, None);
        assert!(rclone_bisync_plan(&root, Path::new("/tmp/synthetic-work")).is_ok());
        assert!(app.editor_action_state(EditorAction::Test).enabled());
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
        complete_test(&mut app, Ok("Synthetic whole-library access pass".into()));
        assert!(app.editor_action_state(EditorAction::Save).enabled());
        let _ = app.update_message(Message::DraftSubpath("Other".into()));
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
    }

    fn shared_remote_editor(provider: Provider, mode: AccessMode, modifying: bool) -> AppModel {
        let mut app = editor(false);
        app.draft.provider = provider;
        app.draft.access_mode = mode;
        app.draft.recovery_directory = "/tmp/synthetic-recovery".into();
        if provider == Provider::OneDrive {
            if mode == AccessMode::OnlineMount {
                app.draft.remote_reference = "https://onedrive.live.com/?id=synthetic".into();
            } else {
                app.draft.offline_engine = Some(OfflineEngine::Rclone);
                app.draft.onedrive_identity = Some(OneDriveIdentity {
                    drive_id: "synthetic-drive".into(),
                    drive_type: cosmic_ext_applet_mounter::model::OneDriveDriveType::Personal,
                });
            }
        }
        if provider == Provider::Teams {
            app.draft.sharepoint_library_url =
                "https://example.sharepoint.com/sites/Work/Documents".into();
            app.draft.teams_drive_id = "b!synthetic".into();
        }
        let current = connection_from_draft(&app.draft).unwrap();
        let mut other = current.clone();
        other.id = ConnectionId::new();
        other.name = "Other connection".into();
        other.local_path = PathBuf::from("/tmp/other-synthetic-target");
        app.config.document.connections.push(other);
        if modifying {
            app.window_mode = WindowMode::ModifyConnection(current.id);
            app.config.document.connections.push(current);
            app.draft.name = "Renamed connection".into();
        }
        // Simulate an editor whose remote settings have already been loaded.
        app.editor_setup_baseline = Some(SetupInputs::from_draft(&app.draft));
        complete_test(&mut app, Ok("Synthetic pass".into()));
        app
    }

    #[test]
    fn editor_actions_confirm_save_covers_all_providers_modes_and_editors() {
        for provider in [
            Provider::OneDrive,
            Provider::Teams,
            Provider::GoogleDrive,
            Provider::Box,
            Provider::Smb,
            Provider::Sftp,
        ] {
            for mode in [AccessMode::OnlineMount, AccessMode::OfflineMirror] {
                for modifying in [false, true] {
                    let mut app = shared_remote_editor(provider, mode, modifying);
                    let current = app.editor_connection().unwrap();
                    assert_eq!(
                        app.editor_button_label(EditorAction::Save, "Save".into()),
                        "Save"
                    );
                    assert!(
                        app.editor_action_state(EditorAction::Save).enabled(),
                        "{provider:?} {mode:?} {modifying}"
                    );
                    assert!(!app.confirm_shared_remote_save(&current));
                    assert!(app.last_notice.as_ref().unwrap().contains("Confirm Save"));
                    assert_eq!(
                        app.editor_button_label(EditorAction::Save, "Save".into()),
                        "Confirm Save"
                    );
                    assert_eq!(app.editor_action_state(EditorAction::Save).tone, Tone::Blue);
                    assert!(app.editor_action_state(EditorAction::Save).enabled());
                    arm(&mut app, EditorAction::Save);
                    assert_eq!(
                        app.editor_button_label(EditorAction::Save, "Save".into()),
                        "Saving…"
                    );
                    assert!(!app.editor_action_state(EditorAction::Save).enabled());
                    app.editor_request = None;
                    assert!(app.confirm_shared_remote_save(&current));
                    assert_eq!(
                        app.editor_button_label(EditorAction::Save, "Save".into()),
                        "Save"
                    );
                    // A later failed/cancelled attempt requires new confirmation.
                    assert!(!app.confirm_shared_remote_save(&current));
                    let _ = app.update_message(Message::DraftSaved("Synthetic saved".into()));
                    assert_eq!(app.pending_shared_remote_ack, None);
                    assert_eq!(
                        app.editor_button_label(EditorAction::Save, "Save".into()),
                        "Save"
                    );
                    app.config
                        .document
                        .connections
                        .retain(|saved| saved.id == current.id);
                    assert!(app.confirm_shared_remote_save(&current));
                    assert_eq!(app.pending_shared_remote_ack, None);
                }
            }
        }
    }

    #[test]
    fn editor_actions_confirm_save_resets_on_edits_and_cancellation() {
        for message in [
            Message::DraftName("Changed".into()),
            Message::DraftRemote("another_remote".into()),
            Message::DraftSubpath("archive".into()),
            Message::DraftLocalPath("/tmp/changed-target".into()),
            Message::DraftRecoveryDirectory("/tmp/changed-recovery".into()),
            Message::DraftSyncInterval("30".into()),
            Message::DraftSyncOnMetered(true),
            Message::DraftCacheLimit("10".into()),
            Message::DraftGoogleClientId("synthetic".into()),
            Message::DraftVpn(None),
            Message::DraftDisconnectVpn(true),
            Message::DraftEnabled(false),
            Message::RecoveryFolderPicked(Ok(Some("/tmp/picked-recovery".into()))),
            Message::CancelOneDriveRclone,
        ] {
            let mut app =
                shared_remote_editor(Provider::GoogleDrive, AccessMode::OfflineMirror, true);
            let current = app.editor_connection().unwrap();
            assert!(!app.confirm_shared_remote_save(&current));
            let _ = app.update_message(message);
            assert_eq!(app.pending_shared_remote_ack, None);
            assert_eq!(
                app.editor_button_label(EditorAction::Save, "Save".into()),
                "Save"
            );
        }
    }

    #[test]
    fn editor_actions_first_shared_save_requests_confirmation_before_persistence() {
        let mut app = shared_remote_editor(Provider::GoogleDrive, AccessMode::OnlineMount, true);
        let before = app.config.document.clone();
        assert_eq!(app.update_message(Message::SaveDraft).units(), 0);
        assert_eq!(app.config.document, before);
        assert_eq!(app.pending_shared_remote_ack, app.draft.id);
        assert_eq!(
            app.editor_button_label(EditorAction::Save, "Save".into()),
            "Confirm Save"
        );
    }

    #[test]
    fn editor_actions_running_preserves_blue_and_grey_and_rejects_duplicates() {
        for (action, message, tone) in [
            (EditorAction::Test, Message::TestDraft, Tone::Blue),
            (
                EditorAction::GoogleOAuth,
                Message::ApplyGoogleDriveRcloneRemote,
                Tone::Grey,
            ),
        ] {
            let mut app = editor(false);
            arm(&mut app, action);
            let state = app.editor_action_state(action);
            assert_eq!(state.tone, tone);
            assert_eq!(state.availability, Availability::Running);
            assert!(!state.enabled());
            assert_eq!(app.update_message(message).units(), 0);
            assert!(!app.editor_action_state(EditorAction::Save).enabled());
            // Editing while running does not change its launch style.
            let _ = app.update_message(Message::DraftName("Changed while running".into()));
            assert_eq!(app.editor_action_state(action).tone, tone);
        }
    }

    #[test]
    fn editor_actions_google_oauth_stays_grey_and_blocks_testing_unapplied_credentials() {
        let mut app = editor(true);
        assert!(!app.editor_action_state(EditorAction::GoogleOAuth).enabled());
        let _ = app.update_message(Message::DraftGoogleClientId("synthetic-client".into()));
        let _ = app.update_message(Message::DraftGoogleClientSecret("synthetic-secret".into()));
        assert!(app.editor_remote_update_pending());
        assert!(!app.editor_action_state(EditorAction::Test).enabled());
        let state = app.editor_action_state(EditorAction::GoogleOAuth);
        assert_eq!(state.tone, Tone::Grey);
        assert!(state.enabled());
        assert_eq!(
            app.editor_action_state(EditorAction::Connect).tone,
            Tone::Grey
        );
        assert!(!app.editor_action_state(EditorAction::Connect).enabled());
    }

    #[test]
    fn editor_actions_stale_success_and_failure_cannot_change_new_draft() {
        for result in [Ok("Old success".into()), Err("Old failure".into())] {
            let mut app = editor(false);
            let original = app.editor_connection().unwrap();
            let sequence = arm(&mut app, EditorAction::Test);
            let _ = app.update_message(Message::DraftSubpath("changed".into()));
            let _ = app.update_message(Message::EditorActionFinished(
                sequence,
                Box::new(Message::DraftTested(original, result)),
            ));
            assert!(app.validated_draft.is_none());
            assert!(app.editor_request.is_none());
            assert!(!app.editor_action_state(EditorAction::Save).enabled());
        }
    }

    #[test]
    fn editor_actions_old_operation_id_does_not_release_new_operation() {
        let mut app = editor(false);
        let old = arm(&mut app, EditorAction::Test);
        let current = arm(&mut app, EditorAction::Test);
        let connection = app.editor_connection().unwrap();
        let _ = app.update_message(Message::EditorActionFinished(
            old,
            Box::new(Message::DraftTested(connection, Err("Old failure".into()))),
        ));
        assert_eq!(app.editor_request.as_ref().unwrap().sequence, current);
    }

    #[test]
    fn editor_actions_failure_recommends_retry_and_disable_only_needs_no_test() {
        let mut app = editor(true);
        complete_test(&mut app, Err("Synthetic failure".into()));
        assert_eq!(app.editor_action_state(EditorAction::Test).tone, Tone::Blue);
        let _ = app.update_message(Message::DraftEnabled(false));
        assert!(app.editor_disable_only());
        assert!(app.editor_action_state(EditorAction::Save).enabled());
        let _ = app.update_message(Message::DraftName("Also renamed".into()));
        assert!(!app.editor_disable_only());
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
    }

    #[test]
    fn editor_actions_server_edit_requires_apply_but_folder_and_policy_only_require_test() {
        let mut app = editor(true);
        app.draft.provider = Provider::Smb;
        app.draft.smb_host = "server.example.invalid".into();
        app.config.document.connections[0] = app.editor_connection().unwrap();
        app.editor_setup_baseline = Some(SetupInputs::from_draft(&app.draft));
        let _ = app.update_message(Message::DraftSubpath("folder".into()));
        assert!(!app.editor_remote_update_pending());
        assert!(app.editor_action_state(EditorAction::Test).enabled());
        let original = app.draft.smb_host.clone();
        let _ = app.update_message(Message::DraftSmbHost("other.example.invalid".into()));
        assert!(app.editor_remote_update_pending());
        assert_eq!(
            app.editor_action_state(EditorAction::Connect).tone,
            Tone::Blue
        );
        assert!(!app.editor_action_state(EditorAction::Test).enabled());
        let _ = app.update_message(Message::DraftSmbHost(original));
        assert!(!app.editor_remote_update_pending());
        assert!(app.editor_action_state(EditorAction::Test).enabled());
    }

    #[test]
    fn editor_actions_preview_sync_do_not_run_against_unsaved_target() {
        let mut app = editor(true);
        let id = app.draft.id.unwrap();
        let _ = app.update_message(Message::DraftLocalPath(
            "/tmp/changed-synthetic-target".into(),
        ));
        for operation in [Operation::PreviewInitialSync, Operation::SyncNow] {
            assert_eq!(
                app.editor_runtime_block_reason(id, operation),
                Some(fl!("editor-runtime-unsaved"))
            );
            assert_eq!(
                app.update_message(Message::OperationRequested(id, operation))
                    .units(),
                0
            );
        }
    }

    #[test]
    fn editor_actions_intermediate_inspection_cannot_finish_test_or_save() {
        let inspection = Message::RcloneInspectionCompleted(0, false, Ok(Vec::new()));
        assert!(!tracked_completion(Some(EditorAction::Test), &inspection));
        assert!(!tracked_completion(Some(EditorAction::Save), &inspection));
        assert!(tracked_completion(Some(EditorAction::Detect), &inspection));
    }
    #[test]
    fn editor_actions_retired_legacy_setup_remains_blocked() {
        let mut app = editor(true);
        app.draft.provider = Provider::OneDrive;
        app.draft.access_mode = AccessMode::OfflineMirror;
        app.draft.legacy_onedrive = true;
        let state = app.editor_action_state(EditorAction::Connect);
        assert_eq!(state.tone, Tone::Grey);
        assert!(!state.enabled());
        assert_eq!(app.update_message(Message::StartOnedriverSetup).units(), 0);
        assert!(app.editor_request.is_none());
    }

    #[test]
    fn editor_actions_stale_save_still_notifies_runtime_without_overwriting_edits() {
        let mut app = editor(true);
        let sequence = arm(&mut app, EditorAction::Save);
        let _ = app.update_message(Message::DraftName("Newer unsaved name".into()));
        let notification = app.update_message(Message::EditorActionFinished(
            sequence,
            Box::new(Message::DraftSaved("Earlier save completed".into())),
        ));
        assert!(notification.units() > 0);
        assert_eq!(app.draft.name, "Newer unsaved name");
        assert!(app.validated_draft.is_none());
        assert!(app.editor_request.is_none());
    }

    #[test]
    fn editor_actions_provider_switch_and_reverted_setup_use_current_baseline() {
        let mut app = editor(false);
        let _ = app.update_message(Message::DraftProvider(Provider::Smb));
        let _ = app.update_message(Message::DraftSmbHost("server.example.invalid".into()));
        assert!(app.editor_remote_update_pending());
        assert!(!app.editor_action_state(EditorAction::Test).enabled());
        let _ = app.update_message(Message::DraftSmbHost(String::new()));
        assert!(!app.editor_remote_update_pending());

        let _ = app.update_message(Message::DraftProvider(Provider::Sftp));
        app.editor_setup_baseline = Some(SetupInputs::from_draft(&app.draft));
        let original = app.draft.sftp.setup_values();
        let _ = app.update_message(Message::DraftSftp(
            SftpField::Host,
            "server.example.invalid".into(),
        ));
        assert!(app.editor_remote_update_pending());
        let _ = app.update_message(Message::DraftSftp(SftpField::Host, original[0].clone()));
        assert!(!app.editor_remote_update_pending());
        assert!(!app.sftp_action_highlighted(false));
    }

    #[test]
    fn editor_actions_google_success_clears_transient_intent_and_recommends_test() {
        let mut app = editor(true);
        let _ = app.update_message(Message::DraftGoogleClientId("synthetic-client".into()));
        let _ = app.update_message(Message::DraftGoogleClientSecret("synthetic-secret".into()));
        let sequence = arm(&mut app, EditorAction::GoogleOAuth);
        // A display-name edit doesn't invalidate a remote authorization request.
        let _ = app.update_message(Message::DraftName("New display name".into()));
        let _ = app.update_message(Message::EditorActionFinished(
            sequence,
            Box::new(Message::GoogleDriveRcloneRemoteApplied(Ok(
                GoogleDriveRemoteApplyResult {
                    remote_name: "example_remote".into(),
                    updated: true,
                },
            ))),
        ));
        assert!(app.editor_request.is_none());
        assert!(app.draft.google_client_id.is_empty());
        assert!(app.draft.google_client_secret.is_empty());
        assert!(!app.editor_remote_update_pending());
        assert!(app.editor_action_state(EditorAction::Test).enabled());
        assert_eq!(app.editor_action_state(EditorAction::Test).tone, Tone::Blue);
        assert!(!app.editor_action_state(EditorAction::Save).enabled());
    }

    #[test]
    fn editor_actions_late_oauth_completion_cannot_clear_new_credentials() {
        let mut app = editor(true);
        let sequence = arm(&mut app, EditorAction::GoogleOAuth);
        let _ = app.update_message(Message::DraftGoogleClientSecret(
            "synthetic-new-secret".into(),
        ));
        let _ = app.update_message(Message::EditorActionFinished(
            sequence,
            Box::new(Message::GoogleDriveRcloneRemoteApplied(Err(
                "Old failure".into()
            ))),
        ));
        assert_eq!(app.draft.google_client_secret, "synthetic-new-secret");
        assert!(app.validated_draft.is_none());
        assert!(app.editor_request.is_none());
    }

    #[test]
    fn editor_actions_cancelled_onedrive_request_never_grants_test_success() {
        let mut app = editor(false);
        let cancellation = CancellationToken::new();
        app.onedrive_editor_pending = Some(OneDriveEditorPending {
            cancellation: cancellation.clone(),
            continue_test: true,
        });
        let sequence = arm(&mut app, EditorAction::Test);
        cancellation.cancel();
        let connection = app.editor_connection().unwrap();
        let _ = app.update_message(Message::EditorActionFinished(
            sequence,
            Box::new(Message::DraftTested(connection, Ok("Late success".into()))),
        ));
        assert!(app.validated_draft.is_none());
        assert!(app.editor_request.is_none());
        assert!(app.onedrive_editor_pending.is_none());
    }
}
