// SPDX-License-Identifier: MIT

//! SFTP connection-editor state and explicit rclone remote setup.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SftpAuth {
    #[default]
    Password,
    Key,
    Agent,
}

#[derive(Debug, Clone, Copy)]
pub enum SftpField {
    Host,
    Port,
    User,
    Password,
    KeyFile,
    KeyPassphrase,
    KnownHosts,
}

fn sftp_field_placeholder(field: SftpField) -> &'static str {
    static HOST: LazyLock<String> = LazyLock::new(|| fl!("sftp-host"));
    static PORT: LazyLock<String> = LazyLock::new(|| fl!("sftp-port"));
    static USER: LazyLock<String> = LazyLock::new(|| fl!("sftp-username"));
    static PASSWORD: LazyLock<String> = LazyLock::new(|| fl!("sftp-password"));
    static KEY_FILE: LazyLock<String> = LazyLock::new(|| fl!("sftp-key-file"));
    static KEY_PASSPHRASE: LazyLock<String> = LazyLock::new(|| fl!("sftp-key-passphrase"));
    static KNOWN_HOSTS: LazyLock<String> = LazyLock::new(|| fl!("sftp-known-hosts"));
    match field {
        SftpField::Host => &HOST,
        SftpField::Port => &PORT,
        SftpField::User => &USER,
        SftpField::Password => &PASSWORD,
        SftpField::KeyFile => &KEY_FILE,
        SftpField::KeyPassphrase => &KEY_PASSPHRASE,
        SftpField::KnownHosts => &KNOWN_HOSTS,
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SftpDraft {
    host: String,
    port: String,
    user: String,
    pub auth: SftpAuth,
    password: String,
    key_file: String,
    key_passphrase: String,
    known_hosts: String,
}

// ConnectionDraft is Debug: never let it expose the new credential inputs.
impl std::fmt::Debug for SftpDraft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SftpDraft")
            .field("auth", &self.auth)
            .finish_non_exhaustive()
    }
}

impl Default for SftpDraft {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: "22".into(),
            user: std::env::var("USER").unwrap_or_default(),
            auth: SftpAuth::Password,
            password: String::new(),
            key_file: String::new(),
            key_passphrase: String::new(),
            known_hosts: "~/.ssh/known_hosts".into(),
        }
    }
}

impl SftpDraft {
    pub fn clear_secrets(&mut self) {
        self.password.clear();
        self.key_passphrase.clear();
    }

    pub fn set(&mut self, field: SftpField, value: String) {
        *match field {
            SftpField::Host => &mut self.host,
            SftpField::Port => &mut self.port,
            SftpField::User => &mut self.user,
            SftpField::Password => &mut self.password,
            SftpField::KeyFile => &mut self.key_file,
            SftpField::KeyPassphrase => &mut self.key_passphrase,
            SftpField::KnownHosts => &mut self.known_hosts,
        } = value;
    }

    fn validate(&self) -> Result<(), String> {
        for (label, value) in [("host", &self.host), ("username", &self.user)] {
            if optional_setup_value(label, value)?.is_none() {
                return Err(format!("SFTP {label} is required."));
            }
        }
        if self.host.contains('/') || self.host.chars().any(char::is_whitespace) {
            return Err(
                "SFTP host must be a DNS name or IP address, without a URL or directory.".into(),
            );
        }
        if self
            .port
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|p| *p > 0)
            .is_none()
        {
            return Err("SFTP port must be between 1 and 65535.".into());
        }
        file_path("known-hosts file", &self.known_hosts)?;
        if self.auth == SftpAuth::Key {
            file_path("private-key file", &self.key_file)?;
            optional_secret_value("key passphrase", &self.key_passphrase)?;
        }
        if self.auth == SftpAuth::Password {
            optional_secret_value("password", &self.password)?;
        }
        Ok(())
    }

    fn from_config(config: &serde_json::Value) -> Self {
        let mut draft = Self {
            host: string_option(config, "host"),
            user: string_option(config, "user"),
            ..Self::default()
        };
        let port = string_option(config, "port");
        if !port.is_empty() {
            draft.port = port;
        }
        draft.key_file = string_option(config, "key_file");
        let known_hosts = string_option(config, "known_hosts_file");
        if !known_hosts.is_empty() {
            draft.known_hosts = known_hosts;
        }
        draft.auth = if bool_option(config, "key_use_agent") {
            SftpAuth::Agent
        } else if !draft.key_file.is_empty() || !string_option(config, "key_pem").is_empty() {
            SftpAuth::Key
        } else if has_password(config) {
            SftpAuth::Password
        } else {
            SftpAuth::Agent
        };
        draft
    }
}

fn string_option(config: &serde_json::Value, key: &str) -> String {
    match &config[key] {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => String::new(),
    }
}

fn has_password(config: &serde_json::Value) -> bool {
    let password = string_option(config, "pass");
    // Rclone --obscure writes an empty password as a 16-byte IV in raw base64
    // (22 characters). Do not mistake that for password authentication.
    !password.is_empty() && password.len() != 22
}

fn bool_option(config: &serde_json::Value, key: &str) -> bool {
    config[key]
        .as_bool()
        .unwrap_or_else(|| string_option(config, key) == "true")
}

fn file_path(label: &str, value: &str) -> Result<String, String> {
    let value =
        optional_setup_value(label, value)?.ok_or_else(|| format!("SFTP {label} is required."))?;
    let path = expand_user_path(&value);
    if !path.is_absolute() {
        return Err(format!(
            "SFTP {label} must be an absolute host path or start with ~/."
        ));
    }
    Ok(path.to_string_lossy().into_owned())
}

fn config_section(output: &str, name: &str) -> Result<Option<serde_json::Value>, String> {
    // Never include config dump text or parser excerpts in an error.
    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|_| "Could not parse rclone configuration.".to_owned())?;
    let object = value.as_object().ok_or("Invalid rclone configuration.")?;
    let Some(section) = object.get(name) else {
        return Ok(None);
    };
    if section["type"].as_str() != Some("sftp") {
        return Err(
            "This remote belongs to another provider. Choose an SFTP remote or a new name.".into(),
        );
    }
    Ok(Some(section.clone()))
}

impl AppModel {
    pub(super) fn mark_sftp_settings_changed(&mut self) {
        self.sftp_settings_revision = self.sftp_settings_revision.wrapping_add(1);
        self.sftp_settings_dirty = true;
    }

    pub(super) fn reset_sftp_settings_tracking(&mut self) {
        self.sftp_settings_revision = self.sftp_settings_revision.wrapping_add(1);
        self.sftp_settings_dirty = false;
        self.sftp_retry_secret_auth = None;
    }

    pub(super) fn finish_sftp_settings_update(
        &mut self,
        name: &str,
        revision: u64,
        succeeded: bool,
    ) {
        if succeeded
            && self.draft.provider == Provider::Sftp
            && self.draft.remote_reference.trim() == name
            && self.sftp_settings_revision == revision
        {
            self.sftp_settings_dirty = false;
        }
    }

    fn sftp_action_highlighted(&self, adding: bool) -> bool {
        !self.sftp_setup_pending
            && (self.sftp_settings_dirty
                || (adding
                    && !self
                        .rclone_remotes
                        .iter()
                        .any(|remote| remote.name == self.draft.remote_reference.trim())))
    }

    pub(super) fn load_sftp_details(&mut self) {
        self.reset_sftp_settings_tracking();
        self.draft.sftp = SftpDraft::default();
        if !self
            .rclone_remotes
            .iter()
            .any(|r| r.name == self.draft.remote_reference && r.backend == "sftp")
            && !self.config.document.connections.iter().any(|c| {
                c.provider == Provider::Sftp && c.remote_reference == self.draft.remote_reference
            })
        {
            return;
        }
        let result = run_sync_host_command("rclone", &["config", "dump"])
            .map_err(|_| "Could not read host rclone configuration.".to_owned())
            .and_then(|output| {
                if !output.status.success() {
                    return Err("Could not read host rclone configuration.".into());
                }
                config_section(
                    &String::from_utf8_lossy(&output.stdout),
                    &self.draft.remote_reference,
                )
            });
        match result {
            Ok(Some(config)) => {
                self.draft.sftp = SftpDraft::from_config(&config);
                self.last_notice = Some("SFTP server settings loaded. Secrets remain blank; Update SFTP Remote explicitly applies changes.".into());
            }
            Ok(None) => self.last_notice = Some("SFTP remote was not found. Select an existing remote or create one in Add Connection.".into()),
            Err(error) => self.last_notice = Some(error),
        }
    }

    pub(super) fn view_sftp_action(&self, adding: bool) -> Element<'static, Message> {
        let label = if self.sftp_setup_pending {
            fl!("sftp-action-applying")
        } else if self.sftp_update_ack.is_some() {
            fl!("sftp-action-confirm")
        } else if adding {
            fl!("sftp-action-create-update")
        } else {
            fl!("sftp-action-update")
        };
        let button = if self.sftp_action_highlighted(adding) {
            widget::button::suggested(label)
        } else {
            widget::button::standard(label)
        };
        field_with_help(
            if self.sftp_setup_pending {
                button
            } else {
                button.on_press(Message::ApplySftpRemote)
            },
            fl!("sftp-action-help"),
        )
    }

    pub(super) fn view_sftp_fields(&self) -> Element<'_, Message> {
        let draft = &self.draft.sftp;
        let input = |field, value, help| {
            field_with_help(
                widget::text_input::text_input(sftp_field_placeholder(field), value)
                    .on_input(move |v| Message::DraftSftp(field, v)),
                help,
            )
        };
        let mut column = widget::Column::new()
            .spacing(8)
            .push(input(SftpField::Host, &draft.host, fl!("sftp-host-help")))
            .push(input(SftpField::Port, &draft.port, fl!("sftp-port-help")))
            .push(input(
                SftpField::User,
                &draft.user,
                fl!("sftp-username-help"),
            ));
        let choices = [
            (
                fl!("sftp-auth-password"),
                SftpAuth::Password,
                fl!("sftp-auth-password-help"),
            ),
            (
                fl!("sftp-auth-key"),
                SftpAuth::Key,
                fl!("sftp-auth-key-help"),
            ),
            (
                fl!("sftp-auth-agent"),
                SftpAuth::Agent,
                fl!("sftp-auth-agent-help"),
            ),
        ]
        .into_iter()
        .map(|(label, auth, help)| {
            let button = if draft.auth == auth {
                widget::button::suggested(label)
            } else {
                widget::button::standard(label)
            };
            field_with_help(button.on_press(Message::DraftSftpAuth(auth)), help)
        })
        .collect();
        column = column.push(choice_row(choices));
        match draft.auth {
            SftpAuth::Password => {
                column = column.push(field_with_help(
                    widget::text_input::text_input(
                        sftp_field_placeholder(SftpField::Password),
                        &draft.password,
                    )
                    .password()
                    .on_input(|v| Message::DraftSftp(SftpField::Password, v)),
                    fl!("sftp-password-help"),
                ));
            }
            SftpAuth::Key => {
                column = column
                    .push(input(
                        SftpField::KeyFile,
                        &draft.key_file,
                        fl!("sftp-key-file-help"),
                    ))
                    .push(field_with_help(
                        widget::text_input::text_input(
                            sftp_field_placeholder(SftpField::KeyPassphrase),
                            &draft.key_passphrase,
                        )
                        .password()
                        .on_input(|v| Message::DraftSftp(SftpField::KeyPassphrase, v)),
                        fl!("sftp-key-passphrase-help"),
                    ));
            }
            SftpAuth::Agent => {
                column = column.push(field_with_help(
                    widget::text::body(fl!("sftp-agent-summary")),
                    fl!("sftp-agent-summary-help"),
                ));
            }
        }
        column
            .push(input(
                SftpField::KnownHosts,
                &draft.known_hosts,
                fl!("sftp-known-hosts-help"),
            ))
            .into()
    }

    pub(super) fn apply_sftp_remote(&mut self) -> Task<cosmic::Action<Message>> {
        if self.draft.provider != Provider::Sftp || self.sftp_setup_pending {
            return Task::none();
        }
        let retry_secret_empty = match self.draft.sftp.auth {
            SftpAuth::Password => self.draft.sftp.password.is_empty(),
            SftpAuth::Key => self.draft.sftp.key_passphrase.is_empty(),
            SftpAuth::Agent => false,
        };
        if self.sftp_retry_secret_auth == Some(self.draft.sftp.auth) && retry_secret_empty {
            self.last_notice = Some(
                "The previous SFTP update failed. Re-enter the password or key passphrase before retrying."
                    .into(),
            );
            return Task::none();
        }
        let name = self.draft.remote_reference.trim().to_owned();
        if let Err(error) =
            validate_rclone_remote_create_name(&name).and_then(|()| self.draft.sftp.validate())
        {
            self.draft.sftp.clear_secrets();
            self.sftp_update_ack = None;
            self.last_notice = Some(error);
            return Task::none();
        }
        let affected = self
            .config
            .document
            .connections
            .iter()
            .filter(|c| {
                c.provider != Provider::OneDrive && c.remote_reference.eq_ignore_ascii_case(&name)
            })
            .map(|c| c.name.clone())
            .collect::<Vec<_>>();
        let current = (name.clone(), self.draft.sftp.clone());
        if !affected.is_empty() && self.sftp_update_ack.as_ref() != Some(&current) {
            self.sftp_update_ack = Some(current);
            self.last_notice = Some(format!(
                "Updating this remote affects: {}. Restart active connections afterward. Click Confirm SFTP Update to apply these settings.",
                affected.join(", ")
            ));
            return Task::none();
        }
        self.sftp_update_ack = None;
        self.sftp_retry_secret_auth = None;
        let draft = self.draft.sftp.clone();
        let attempted_secret = match draft.auth {
            SftpAuth::Password if !draft.password.is_empty() => Some(SftpAuth::Password),
            SftpAuth::Key if !draft.key_passphrase.is_empty() => Some(SftpAuth::Key),
            _ => None,
        };
        self.draft.sftp.clear_secrets();
        self.sftp_settings_dirty = true;
        let revision = self.sftp_settings_revision;
        self.sftp_setup_pending = true;
        self.validated_draft = None;
        let update_only = matches!(self.window_mode, WindowMode::ModifyConnection(_));
        self.last_notice = Some(format!("Applying SFTP remote `{name}`…"));
        Task::perform(
            async move {
                let result = apply_remote(&app_command_runner(), &name, &draft, update_only).await;
                (name, result)
            },
            move |(name, result)| {
                cosmic::Action::App(Message::SftpRemoteApplied(
                    name,
                    revision,
                    attempted_secret,
                    result,
                ))
            },
        )
    }
}

async fn apply_remote(
    runner: &dyn CommandRunner,
    name: &str,
    draft: &SftpDraft,
    update_only: bool,
) -> Result<bool, String> {
    validate_rclone_remote_create_name(name)?;
    draft.validate()?;
    let request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .and_then(|r| r.arg("dump"))
        .map_err(|_| "Invalid configuration request.")?
        .with_timeout(Duration::from_secs(5));
    let dump = runner
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| sftp_setup_error("read configuration", error))?;
    let existing = config_section(&dump.stdout.text, name)?;
    if update_only && existing.is_none() {
        return Err("Remote no longer exists. Create it in Add Connection first.".into());
    }
    let request = setup_request(name, draft, existing.as_ref())?;
    let output = runner
        .run(request, CancellationToken::new())
        .await
        .map_err(|error| sftp_setup_error("save remote", error))?;
    // rclone config update can print a save error but still exit successfully
    // with an empty JSON Error field. Do not report that as an applied update.
    if output
        .stderr
        .text
        .lines()
        .any(|line| line.contains("ERROR :") || line.contains("Failed to save config"))
    {
        return Err("SFTP save remote failed. Check rclone configuration permissions and inspect the remote before retrying; settings may have changed.".into());
    }
    if !output.stdout.text.trim().is_empty() {
        let result: serde_json::Value = serde_json::from_str(&output.stdout.text).map_err(
            |_| "Rclone returned an unexpected setup response; inspect the remote before retrying.",
        )?;
        if result
            .get("State")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty())
            || result
                .get("Error")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty())
        {
            return Err("Rclone setup needs additional configuration. Complete it with rclone config and then select the remote.".into());
        }
    }
    Ok(existing.is_some())
}

fn sftp_setup_error(stage: &str, error: CommandError) -> String {
    match error {
        CommandError::Timeout { timeout, .. } => format!(
            "SFTP {stage} timed out after {} seconds. Inspect the remote before retrying; settings may have changed.",
            timeout.as_secs()
        ),
        CommandError::MissingExecutable(_) => "rclone is missing on the host.".into(),
        CommandError::Cancelled { .. } => format!("SFTP {stage} was cancelled."),
        CommandError::InvalidArgument => "SFTP setup contains an invalid option.".into(),
        CommandError::Spawn { .. } => "Could not start host rclone for SFTP setup.".into(),
        CommandError::NonZero { .. } => format!(
            "SFTP {stage} failed. Check rclone configuration permissions and inspect the remote before retrying; settings may have changed."
        ),
    }
}

fn setup_request(
    name: &str,
    draft: &SftpDraft,
    existing: Option<&serde_json::Value>,
) -> Result<CommandRequest, String> {
    validate_rclone_remote_create_name(name)?;
    draft.validate()?;
    if let Some(config) = existing {
        if config["type"].as_str() != Some("sftp") {
            return Err("Remote backend is not SFTP.".into());
        }
        if !string_option(config, "ssh").is_empty() || !string_option(config, "key_pem").is_empty()
        {
            return Err("This remote uses custom SSH commands or an embedded key. Manage its authentication with rclone config. Applet access testing requires rclone's built-in SSH authentication.".into());
        }
    }
    let previous = existing.map(SftpDraft::from_config);
    let same_auth = previous.as_ref().is_some_and(|d| d.auth == draft.auth);
    let mut pairs = vec![
        ("host", draft.host.trim().to_owned()),
        ("port", draft.port.trim().to_owned()),
        ("user", draft.user.trim().to_owned()),
        (
            "known_hosts_file",
            file_path("known-hosts file", &draft.known_hosts)?,
        ),
        ("ask_password", "false".into()),
        ("key_use_agent", (draft.auth == SftpAuth::Agent).to_string()),
    ];
    match draft.auth {
        SftpAuth::Password => {
            if draft.password.is_empty() && !(same_auth && existing.is_some_and(has_password)) {
                return Err("Enter a password for a new password login.".into());
            }
            if !draft.password.is_empty() {
                pairs.push(("pass", draft.password.clone()));
            }
            pairs.extend([
                ("key_file", String::new()),
                ("key_file_pass", String::new()),
                ("pubkey_file", String::new()),
                ("pubkey", String::new()),
            ]);
        }
        SftpAuth::Key => {
            let key = file_path("private-key file", &draft.key_file)?;
            let same_key = same_auth
                && previous.as_ref().is_some_and(|d| {
                    file_path("private-key file", &d.key_file).ok().as_ref() == Some(&key)
                });
            pairs.extend([("pass", String::new()), ("key_file", key)]);
            if !same_key || !draft.key_passphrase.is_empty() {
                pairs.push(("key_file_pass", draft.key_passphrase.clone()));
            }
            if !same_key {
                pairs.extend([("pubkey_file", String::new()), ("pubkey", String::new())]);
            }
        }
        SftpAuth::Agent => {
            pairs.extend([("pass", String::new()), ("key_file_pass", String::new())]);
            if !same_auth {
                pairs.extend([
                    ("key_file", String::new()),
                    ("pubkey_file", String::new()),
                    ("pubkey", String::new()),
                ]);
            }
        }
    }
    if existing.is_none() {
        pairs.push(("shell_type", "none".into()));
    }
    let mut request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .and_then(|r| {
            r.arg(if existing.is_some() {
                "update"
            } else {
                "create"
            })
        })
        .and_then(|r| r.arg(name))
        .map_err(|_| "Invalid remote name.")?;
    if existing.is_none() {
        request = request.arg("sftp").map_err(|_| "Invalid backend.")?;
    }
    for (key, value) in pairs {
        request = request
            .arg(key)
            .and_then(|r| r.sensitive_arg(value))
            .map_err(|_| "Invalid SFTP option.")?;
    }
    // One config operation avoids partially switching authentication across commands.
    request
        .arg("--obscure")
        .and_then(|r| r.arg("--non-interactive"))
        .map(|r| r.with_timeout(Duration::from_secs(30)))
        .map_err(|_| "Invalid SFTP request.".into())
}

pub(super) async fn verify_host_verification(
    runner: &dyn CommandRunner,
    name: &str,
) -> Result<(), String> {
    let request = CommandRequest::new(Executable::Rclone)
        .arg("config")
        .and_then(|r| r.arg("dump"))
        .map_err(|_| "Invalid configuration request.")?
        .with_timeout(Duration::from_secs(5));
    let dump = runner
        .run(request, CancellationToken::new())
        .await
        .map_err(|_| "Could not read SFTP host-verification settings.".to_owned())?;
    let config = config_section(&dump.stdout.text, name)?.ok_or("SFTP remote was not found.")?;
    if !string_option(&config, "ssh").is_empty() {
        return Err("This remote uses an external SSH command. Select a remote using rclone's built-in SFTP authentication so the applet can verify its host-key policy.".into());
    }
    file_path("known-hosts file", &string_option(&config, "known_hosts_file"))
        .map_err(|_| "SFTP requires a known-hosts file. Set a verified file with Create/Update SFTP Remote before testing or saving.".to_owned())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::Application;
    use cosmic_ext_applet_mounter::process::{CapturedOutput, FakeCommandRunner};
    use serde_json::json;

    fn draft() -> SftpDraft {
        SftpDraft {
            host: "files.example.test".into(),
            user: "alice".into(),
            password: "private-password".into(),
            known_hosts: "/home/alice/.ssh/known_hosts".into(),
            ..SftpDraft::default()
        }
    }

    fn output(text: &str) -> CommandOutput {
        CommandOutput {
            duration: Duration::ZERO,
            command: "rclone".into(),
            stdout: CapturedOutput {
                text: text.into(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
        }
    }

    #[test]
    fn sftp_remote_edits_highlight_update_in_add_and_modify() {
        for (field, value) in [
            (SftpField::Host, "other.example"),
            (SftpField::Port, "2222"),
            (SftpField::User, "other-user"),
            (SftpField::Password, "new-password"),
            (SftpField::KeyFile, "/key"),
            (SftpField::KeyPassphrase, "new-passphrase"),
            (SftpField::KnownHosts, "/known_hosts"),
        ] {
            let mut app = AppModel::default();
            app.draft.provider = Provider::Sftp;
            app.draft.remote_reference = "remote".into();
            app.rclone_remotes.push(RcloneDraftRemote {
                name: "remote".into(),
                backend: "sftp".into(),
            });
            assert!(!app.sftp_action_highlighted(true));
            let _ = app.update(Message::DraftSftp(field, value.into()));
            assert!(app.sftp_action_highlighted(true));
            assert!(app.sftp_action_highlighted(false));
            app.sftp_setup_pending = true;
            assert!(!app.sftp_action_highlighted(false));
        }
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        let _ = app.update(Message::DraftSftpAuth(SftpAuth::Agent));
        assert!(app.sftp_action_highlighted(false));
        app.reset_sftp_settings_tracking();
        let _ = app.update(Message::DraftLocalPath("/tmp/mount".into()));
        assert!(!app.sftp_action_highlighted(false));
    }

    #[test]
    fn sftp_update_completion_preserves_failed_and_newer_edits() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        app.draft.remote_reference = "remote".into();
        app.mark_sftp_settings_changed();
        let revision = app.sftp_settings_revision;
        app.finish_sftp_settings_update("remote", revision, false);
        assert!(app.sftp_settings_dirty);
        app.finish_sftp_settings_update("another", revision, true);
        assert!(app.sftp_settings_dirty);
        app.mark_sftp_settings_changed();
        app.finish_sftp_settings_update("remote", revision, true);
        assert!(app.sftp_settings_dirty);
        app.finish_sftp_settings_update("remote", app.sftp_settings_revision, true);
        assert!(!app.sftp_settings_dirty);
        app.mark_sftp_settings_changed();
        app.reset_sftp_settings_tracking();
        assert!(!app.sftp_settings_dirty);
    }

    #[test]
    fn sftp_setup_validates_fields_and_masks_credentials() {
        let mut draft = draft();
        let command = setup_request("test_sftp", &draft, None)
            .unwrap()
            .sanitized_command();
        assert!(command.starts_with("rclone config create test_sftp sftp"));
        assert!(command.ends_with("--obscure --non-interactive"));
        assert!(!command.contains(&draft.password));
        assert!(!format!("{draft:?}").contains(&draft.password));
        for port in ["0", "65536", "abc", "22\n23"] {
            draft.port = port.into();
            assert!(draft.validate().is_err());
        }
        draft.port = "22".into();
        draft.host = "sftp://host/path".into();
        assert!(draft.validate().is_err());
        draft.host = "host".into();
        draft.known_hosts = "none".into();
        assert!(draft.validate().is_err());
        assert!(setup_request("bad:name", &draft, None).is_err());
    }

    #[test]
    fn sftp_details_never_load_secrets_and_detect_authentication() {
        let mut config = json!({"type":"sftp", "host":"server", "user":"alice", "port":2222, "pass":"obscured-secret", "key_file_pass":"obscured-key"});
        let d = SftpDraft::from_config(&config);
        assert_eq!(d.port, "2222");
        assert_eq!(d.auth, SftpAuth::Password);
        assert!(d.password.is_empty() && d.key_passphrase.is_empty());
        config["key_file"] = json!("/keys/private");
        assert_eq!(SftpDraft::from_config(&config).auth, SftpAuth::Key);
        config["key_use_agent"] = json!(true);
        assert_eq!(SftpDraft::from_config(&config).auth, SftpAuth::Agent);
    }

    #[test]
    fn sftp_updates_preserve_blank_secrets_only_for_unchanged_authentication() {
        let mut d = draft();
        d.password.clear();
        let existing = json!({"type":"sftp", "pass":"stored"});
        let request = setup_request("remote", &d, Some(&existing))
            .unwrap()
            .sanitized_command();
        assert!(request.starts_with("rclone config update remote"));
        assert!(!request.contains(" pass "));
        assert!(setup_request("remote", &d, None).is_err());
        assert!(
            setup_request(
                "remote",
                &d,
                Some(&json!({"type":"sftp", "key_file":"/key"}))
            )
            .is_err()
        );
        assert!(setup_request("remote", &d, Some(&json!({"type":"smb"}))).is_err());
        assert!(
            setup_request(
                "remote",
                &d,
                Some(&json!({"type":"sftp", "ssh":"ssh server"}))
            )
            .is_err()
        );
        d.auth = SftpAuth::Agent;
        let request = setup_request("remote", &d, Some(&existing))
            .unwrap()
            .sanitized_command();
        assert!(request.contains(" pass [REDACTED]"));
        assert!(request.contains(" key_file [REDACTED]"));
        d.auth = SftpAuth::Key;
        d.key_file = "/key".into();
        let old = json!({"type":"sftp", "key_file":"/key", "key_file_pass":"stored"});
        let same = setup_request("remote", &d, Some(&old))
            .unwrap()
            .sanitized_command();
        assert!(!same.contains(" key_file_pass "));
        d.key_file = "/new-key".into();
        let changed = setup_request("remote", &d, Some(&old))
            .unwrap()
            .sanitized_command();
        assert!(changed.contains(" key_file_pass [REDACTED]"));
    }

    #[tokio::test]
    async fn sftp_apply_checks_backend_before_mutating_and_requires_existing_for_modify() {
        for config in [r#"{"remote":{"type":"smb"}}"#, "{}"] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(config)));
            assert!(
                apply_remote(&runner, "remote", &draft(), true)
                    .await
                    .is_err()
            );
            assert_eq!(runner.requests().len(), 1);
        }
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output("{}")));
        assert!(
            !apply_remote(&runner, "remote", &draft(), false)
                .await
                .unwrap()
        );
        assert_eq!(runner.requests().len(), 2);
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"remote":{"type":"sftp","pass":"stored"}}"#)));
        runner.push(Ok(output("{}")));
        assert!(
            apply_remote(&runner, "remote", &draft(), true)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn sftp_pending_config_question_is_not_success() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(
            r#"{"State":"needs-more","Error":"secret-server-detail"}"#,
        )));
        let error = apply_remote(&runner, "remote", &draft(), false)
            .await
            .unwrap_err();
        assert!(!error.contains("secret-server-detail"));
        assert!(error.contains("additional configuration"));
    }

    #[tokio::test]
    async fn sftp_zero_exit_with_save_error_is_not_success() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"remote":{"type":"sftp","pass":"stored"}}"#)));
        let mut misleading = output(r#"{"State":"","Error":"","Result":""}"#);
        misleading.stderr.text = "ERROR : Failed to save config after 10 tries: /private/config password=private-password".into();
        runner.push(Ok(misleading));
        let error = apply_remote(&runner, "remote", &draft(), true)
            .await
            .unwrap_err();
        assert!(error.contains("save remote failed"));
        assert!(!error.contains("private-password"));
        assert!(!error.contains("/private/config"));
    }

    #[tokio::test]
    async fn sftp_setup_timeout_names_stage_without_exposing_credentials() {
        let runner = FakeCommandRunner::default();
        runner.push(Err(CommandError::Timeout {
            command: "rclone config dump password=private-password".into(),
            timeout: Duration::from_secs(5),
        }));
        let error = apply_remote(&runner, "remote", &draft(), false)
            .await
            .unwrap_err();
        assert!(error.contains("read configuration timed out after 5 seconds"));
        assert!(!error.contains("private-password"));

        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"remote":{"type":"sftp","pass":"stored"}}"#)));
        runner.push(Err(CommandError::Timeout {
            command: "rclone config update remote password=private-password".into(),
            timeout: Duration::from_secs(30),
        }));
        let error = apply_remote(&runner, "remote", &draft(), true)
            .await
            .unwrap_err();
        assert!(error.contains("save remote timed out after 30 seconds"));
        assert!(error.contains("settings may have changed"));
        assert!(!error.contains("private-password"));
    }

    #[test]
    fn failed_secret_update_requires_reentry_before_retry() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        app.draft.remote_reference = "remote".into();
        app.draft.sftp = draft();
        app.draft.sftp.password.clear();
        app.sftp_settings_dirty = true;
        app.sftp_retry_secret_auth = Some(SftpAuth::Password);
        let _ = app.apply_sftp_remote();
        assert!(!app.sftp_setup_pending);
        assert!(app.sftp_action_highlighted(false));
        assert!(app.last_notice.as_deref().unwrap().contains("Re-enter"));

        let _ = app.update(Message::DraftSftp(SftpField::Password, "new-secret".into()));
        let _ = app.update(Message::DraftSftp(SftpField::Password, String::new()));
        let _ = app.apply_sftp_remote();
        assert!(!app.sftp_setup_pending);
        assert!(app.last_notice.as_deref().unwrap().contains("Re-enter"));
    }

    #[test]
    fn remote_update_completion_keeps_failed_action_blue_and_clears_success() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        app.draft.remote_reference = "remote".into();
        app.sftp_settings_dirty = true;
        app.sftp_setup_pending = true;
        let revision = app.sftp_settings_revision;
        let _ = app.update(Message::SftpRemoteApplied(
            "remote".into(),
            revision,
            Some(SftpAuth::Password),
            Err("SFTP save remote timed out after 30 seconds".into()),
        ));
        assert!(app.sftp_action_highlighted(false));
        assert_eq!(app.sftp_retry_secret_auth, Some(SftpAuth::Password));
        let notice = app.last_notice.as_deref().unwrap();
        assert!(notice.contains("timed out after 30 seconds"));
        assert!(notice.contains("Re-enter the credential"));

        app.draft.sftp = draft();
        app.draft.sftp.clear_secrets();
        app.sftp_setup_pending = true;
        let _ = app.update(Message::SftpRemoteApplied(
            "remote".into(),
            revision,
            Some(SftpAuth::Password),
            Ok(true),
        ));
        assert!(!app.sftp_action_highlighted(false));
        assert!(app.sftp_retry_secret_auth.is_none());
        assert!(app.draft.sftp.password.is_empty());
        assert!(app.last_notice.as_deref().unwrap().contains("Updated"));
    }

    #[test]
    fn sftp_auth_changes_clear_secrets_and_modify_locks_provider() {
        let mut app = AppModel::default();
        app.draft.provider = Provider::Sftp;
        app.draft.sftp = draft();
        app.draft.sftp.key_passphrase = "key-secret".into();
        let _ = app.update(Message::DraftSftpAuth(SftpAuth::Agent));
        assert!(app.draft.sftp.password.is_empty() && app.draft.sftp.key_passphrase.is_empty());
        app.window_mode = WindowMode::ModifyConnection(ConnectionId::new());
        let _ = app.update(Message::DraftProvider(Provider::Smb));
        assert_eq!(app.draft.provider, Provider::Sftp);
    }

    #[test]
    fn sftp_discovery_filters_backend_and_help_matches_editor_mode() {
        let remotes = parse_rclone_remotes_for_app(
            r#"{"ssh":{"type":"sftp"},"share":{"type":"smb"},"ftp":{"type":"ftp"}}"#,
        )
        .unwrap();
        let app = AppModel {
            rclone_remotes: remotes,
            ..AppModel::default()
        };
        assert_eq!(app.matching_rclone_remotes(Provider::Sftp).len(), 1);
        assert_eq!(app.matching_rclone_remotes(Provider::Sftp)[0].name, "ssh");
        assert!(rclone_remote_help(Provider::Sftp, false).contains("Update SFTP Remote"));
        assert!(rclone_remote_help(Provider::Sftp, true).contains("Create/Update"));
    }

    #[tokio::test]
    async fn sftp_access_rejects_disabled_host_verification() {
        for config in [
            json!({"type":"sftp"}),
            json!({"type":"sftp", "known_hosts_file":"none"}),
            json!({"type":"sftp", "known_hosts_file":"/known_hosts", "ssh":"ssh host"}),
        ] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(&json!({"remote":config}).to_string())));
            assert!(verify_host_verification(&runner, "remote").await.is_err());
        }
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(
            r#"{"remote":{"type":"sftp","known_hosts_file":"/known_hosts"}}"#,
        )));
        assert!(verify_host_verification(&runner, "remote").await.is_ok());
    }

    #[test]
    fn sftp_obscured_empty_password_is_not_a_password_login() {
        let config = json!({"type":"sftp", "pass":"abcdefghijklmnopqrstuv"});
        assert_eq!(SftpDraft::from_config(&config).auth, SftpAuth::Agent);
        let mut draft = draft();
        draft.password.clear();
        assert!(setup_request("remote", &draft, Some(&config)).is_err());
    }

    #[test]
    fn sftp_shared_remote_update_requires_acknowledgement_of_current_fields() {
        let draft = ConnectionDraft {
            provider: Provider::Sftp,
            name: "Server".into(),
            remote_reference: "remote".into(),
            local_path: "/tmp/sftp-editor".into(),
            sftp: draft(),
            ..ConnectionDraft::default()
        };
        let connection = connection_from_draft(&draft).unwrap();
        let mut app = AppModel {
            draft,
            window_mode: WindowMode::ModifyConnection(connection.id),
            ..AppModel::default()
        };
        app.config.document.connections.push(connection);
        let _ = app.apply_sftp_remote();
        assert!(app.sftp_update_ack.is_some());
        assert!(!app.sftp_setup_pending);
        assert!(app.last_notice.as_deref().unwrap().contains("Server"));
        let _ = app.update(Message::DraftSftp(SftpField::Host, "other.test".into()));
        assert!(app.sftp_update_ack.is_none());
        let _ = app.apply_sftp_remote();
        assert!(app.sftp_update_ack.is_some());
        assert!(!app.sftp_setup_pending);
    }

    #[test]
    fn sftp_draft_roundtrip_preserves_absolute_target_and_provider() {
        let draft = ConnectionDraft {
            provider: Provider::Sftp,
            name: "SSH".into(),
            remote_reference: "ssh".into(),
            remote_subpath: "/srv/files".into(),
            local_path: "/tmp/sftp-editor-target".into(),
            ..ConnectionDraft::default()
        };
        let connection = connection_from_draft(&draft).unwrap();
        assert_eq!(rclone_access_target(&connection), "ssh:/srv/files");
        let restored = draft_from_connection(&connection);
        assert_eq!(restored.provider, Provider::Sftp);
        assert_eq!(restored.remote_subpath, "/srv/files");
        let encoded = ron::to_string(&connection).unwrap();
        assert_eq!(ron::from_str::<Connection>(&encoded).unwrap(), connection);
        assert!(
            !ConfigDocument::default()
                .preload_policy_for(&connection)
                .enabled
        );
    }
}
