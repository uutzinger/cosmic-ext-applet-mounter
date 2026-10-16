// SPDX-License-Identifier: MIT

//! Shared rclone Microsoft authorization transport. Provider wrappers select
//! and verify their drive; this module never accepts arbitrary commands/clients.

use crate::process::{CommandError, CommandRequest, CommandRunner, Executable};
use serde_json::Value;
use std::{fmt, time::Duration};
use tokio_util::sync::CancellationToken;

pub(crate) const PERSONAL_SCOPES: &str = "Files.Read Files.ReadWrite Files.Read.All Files.ReadWrite.All Sites.Read.All offline_access User.Read";

#[derive(Clone)]
pub(crate) enum Target {
    SharePoint { site_url: String },
    Personal,
    Business,
}

impl From<crate::model::OneDriveDriveType> for Target {
    fn from(kind: crate::model::OneDriveDriveType) -> Self {
        match kind {
            crate::model::OneDriveDriveType::Personal => Self::Personal,
            crate::model::OneDriveDriveType::Business => Self::Business,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandFailure {
    MissingExecutable,
    InvalidArgument,
    Spawn,
    Timeout,
    Cancelled,
    NonZero(Option<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OAuthError {
    InvalidRemoteName,
    ExistingRemote,
    InvalidResponse,
    UnstructuredResponse,
    Rejected,
    TooManySteps,
    RemoteNotFound,
    WrongBackend,
    NotAuthenticated,
    InvalidToken,
    ReauthorizationRequired,
    CleanupRequired,
    Command(CommandFailure),
}

impl From<CommandError> for OAuthError {
    fn from(error: CommandError) -> Self {
        Self::Command(match error {
            CommandError::MissingExecutable(_) => CommandFailure::MissingExecutable,
            CommandError::InvalidArgument => CommandFailure::InvalidArgument,
            CommandError::Spawn { .. } => CommandFailure::Spawn,
            CommandError::Timeout { .. } => CommandFailure::Timeout,
            CommandError::Cancelled { .. } => CommandFailure::Cancelled,
            CommandError::NonZero { code, .. } => CommandFailure::NonZero(code),
        })
    }
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidRemoteName => crate::fl!("microsoft-oauth-invalid-name"),
            Self::ExistingRemote => crate::fl!("microsoft-oauth-remote-exists"),
            Self::InvalidResponse | Self::UnstructuredResponse => {
                crate::fl!("microsoft-oauth-invalid-response")
            }
            Self::Rejected => crate::fl!("microsoft-oauth-rejected"),
            Self::TooManySteps => crate::fl!("microsoft-oauth-too-many-steps"),
            Self::RemoteNotFound => crate::fl!("microsoft-oauth-remote-missing"),
            Self::WrongBackend => crate::fl!("microsoft-oauth-wrong-backend"),
            Self::NotAuthenticated => crate::fl!("microsoft-oauth-not-authenticated"),
            Self::InvalidToken => crate::fl!("microsoft-oauth-invalid-token"),
            Self::ReauthorizationRequired => crate::fl!("microsoft-oauth-reauthorize"),
            Self::CleanupRequired => crate::fl!("microsoft-oauth-cleanup-required"),
            Self::Command(CommandFailure::MissingExecutable) => {
                crate::fl!("microsoft-oauth-rclone-missing")
            }
            Self::Command(CommandFailure::Timeout) => crate::fl!("microsoft-oauth-timeout"),
            Self::Command(CommandFailure::Cancelled) => crate::fl!("microsoft-oauth-cancelled"),
            Self::Command(_) => crate::fl!("microsoft-oauth-command-failed"),
        };
        f.write_str(&message)
    }
}

pub(crate) fn validate_remote_name(name: &str) -> Result<(), OAuthError> {
    if name.is_empty()
        || name.len() > 96
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "_-.".contains(ch))
    {
        return Err(OAuthError::InvalidRemoteName);
    }
    Ok(())
}

pub(crate) async fn config_dump<R: CommandRunner>(
    runner: &R,
    cancel: CancellationToken,
) -> Result<Value, OAuthError> {
    if cancel.is_cancelled() {
        return Err(OAuthError::Command(CommandFailure::Cancelled));
    }
    let request = CommandRequest::new(Executable::Rclone)
        .arg("config")?
        .arg("dump")?
        .with_timeout(Duration::from_secs(10))
        .with_output_limit(1024 * 1024);
    let value = runner
        .private_config(request, cancel)
        .await?
        .json()
        .map_err(|_| OAuthError::InvalidResponse)?;
    if !value
        .as_object()
        .is_some_and(|remotes| remotes.values().all(Value::is_object))
    {
        return Err(OAuthError::InvalidResponse);
    }
    Ok(value)
}

/// Ownership capability granted only after a read-only absence check. It is
/// never constructed for an existing remote; rollback needs a started creation.
pub(crate) struct NewRemote {
    name: String,
    target: Target,
    started: bool,
    steps: usize,
}

impl fmt::Debug for NewRemote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NewRemote([REDACTED])")
    }
}

impl NewRemote {
    pub(crate) async fn prepare<R: CommandRunner>(
        runner: &R,
        name: &str,
        target: Target,
        cancel: CancellationToken,
    ) -> Result<Self, OAuthError> {
        validate_remote_name(name)?;
        if config_dump(runner, cancel).await?.get(name).is_some() {
            return Err(OAuthError::ExistingRemote);
        }
        Ok(Self {
            name: name.into(),
            target,
            started: false,
            steps: 0,
        })
    }

    fn request(&self, action: &str) -> Result<CommandRequest, OAuthError> {
        let mut request = CommandRequest::new(Executable::Rclone)
            .arg("config")?
            .arg(action)?
            .arg(&self.name)?;
        if action == "delete" {
            return Ok(request
                .with_timeout(Duration::from_secs(10))
                .with_output_limit(16 * 1024));
        }
        request = if action == "create" {
            request.arg("onedrive")?
        } else {
            request.arg("--continue")?
        };
        request = request
            .arg("config_is_local")?
            .arg("true")?
            .arg("config_type")?;
        request = match &self.target {
            Target::SharePoint { site_url } => {
                request.arg("url")?.arg("config_site_url")?.arg(site_url)?
            }
            Target::Personal | Target::Business => request.arg("onedrive")?,
        };
        if action == "create" && matches!(self.target, Target::Personal) {
            request = request.arg("access_scopes")?.arg(PERSONAL_SCOPES)?;
        }
        Ok(request
            .arg("--non-interactive")?
            .with_timeout(Duration::from_secs(300))
            .with_output_limit(256 * 1024))
    }

    async fn execute<R: CommandRunner>(
        &mut self,
        runner: &R,
        request: CommandRequest,
        cancel: CancellationToken,
    ) -> Result<Value, OAuthError> {
        if cancel.is_cancelled() {
            return Err(OAuthError::Command(CommandFailure::Cancelled));
        }
        let maximum = if matches!(self.target, Target::SharePoint { .. }) {
            8
        } else {
            12
        };
        if self.steps >= maximum {
            return Err(OAuthError::TooManySteps);
        }
        self.steps += 1;
        self.started = true;
        let output = runner.private_config(request, cancel).await?;
        if !output.is_complete() {
            return Err(OAuthError::InvalidResponse);
        }
        let response = output
            .oauth_json()
            .map_err(|_| OAuthError::UnstructuredResponse)?;
        match response.get("Error") {
            Some(Value::String(error)) if !error.is_empty() => return Err(OAuthError::Rejected),
            None | Some(Value::String(_)) => {}
            _ => return Err(OAuthError::InvalidResponse),
        }
        Ok(response)
    }

    pub(crate) async fn create<R: CommandRunner>(
        &mut self,
        runner: &R,
        cancel: CancellationToken,
    ) -> Result<Value, OAuthError> {
        if self.started {
            return Err(OAuthError::InvalidResponse);
        }
        let request = self.request("create")?;
        self.execute(runner, request, cancel).await
    }

    pub(crate) async fn continue_with<R: CommandRunner>(
        &mut self,
        runner: &R,
        state: &str,
        answer: &str,
        cancel: CancellationToken,
    ) -> Result<Value, OAuthError> {
        if !self.started || state.is_empty() || state.len() > 16 * 1024 || answer.len() > 16 * 1024
        {
            return Err(OAuthError::InvalidResponse);
        }
        let request = self
            .request("update")?
            .arg("--state")?
            .sensitive_arg(state)?
            .arg("--result")?
            .sensitive_arg(answer)?;
        self.execute(runner, request, cancel).await
    }

    pub(crate) async fn rollback<R: CommandRunner>(&self, runner: &R) -> Result<(), OAuthError> {
        if !self.started {
            return Ok(());
        }
        let dump = config_dump(runner, CancellationToken::new())
            .await
            .map_err(|_| OAuthError::CleanupRequired)?;
        if dump.get(&self.name).is_some() {
            runner
                .run(self.request("delete")?, CancellationToken::new())
                .await
                .map_err(|_| OAuthError::CleanupRequired)?;
        }
        Ok(())
    }
}

pub(crate) fn oauth_access_token(config_json: &str, name: &str) -> Result<String, OAuthError> {
    let dump: Value = serde_json::from_str(config_json).map_err(|_| OAuthError::InvalidResponse)?;
    let remote = dump.get(name).ok_or(OAuthError::RemoteNotFound)?;
    if remote.get("type").and_then(Value::as_str) != Some("onedrive") {
        return Err(OAuthError::WrongBackend);
    }
    access_token_from_remote(remote)
}

pub(crate) fn access_token_from_remote(remote: &Value) -> Result<String, OAuthError> {
    let token = remote.get("token").ok_or(OAuthError::NotAuthenticated)?;
    let parsed;
    let token = if let Some(text) = token.as_str() {
        parsed = serde_json::from_str::<Value>(text).map_err(|_| OAuthError::InvalidToken)?;
        &parsed
    } else {
        token
    };
    token
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(OAuthError::ReauthorizationRequired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner, PrivateConfigOutput};
    use serde_json::json;

    fn output(text: &str) -> CommandOutput {
        CommandOutput {
            command: "synthetic config".into(),
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
            duration: Duration::ZERO,
        }
    }

    #[tokio::test]
    async fn microsoft_oauth_invalid_names_and_existing_remotes_never_mutate_config() {
        let runner = FakeCommandRunner::default();
        for name in ["", "bad:name", "bad/name", "bad\nname"] {
            assert_eq!(
                NewRemote::prepare(&runner, name, Target::Business, CancellationToken::new())
                    .await
                    .unwrap_err(),
                OAuthError::InvalidRemoteName
            );
        }
        assert!(runner.requests().is_empty());
        runner.push(Ok(output(r#"{"existing":{"type":"onedrive"}}"#)));
        assert_eq!(
            NewRemote::prepare(
                &runner,
                "existing",
                Target::Business,
                CancellationToken::new()
            )
            .await
            .unwrap_err(),
            OAuthError::ExistingRemote
        );
        assert_eq!(runner.requests().len(), 1);
    }

    #[tokio::test]
    async fn microsoft_oauth_malformed_dump_does_not_authorize_creation() {
        for dump in [
            "[]",
            "null",
            r#"{"State":"","Error":""}"#,
            r#"{"broken":null}"#,
        ] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(dump)));
            assert_eq!(
                NewRemote::prepare(&runner, "new", Target::Business, CancellationToken::new())
                    .await
                    .unwrap_err(),
                OAuthError::InvalidResponse
            );
            assert_eq!(runner.requests().len(), 1);
        }
    }

    #[tokio::test]
    async fn microsoft_oauth_targets_preserve_scopes_and_redact_continuations() {
        for target in [
            Target::Personal,
            Target::Business,
            Target::SharePoint {
                site_url: "https://example.sharepoint.com/sites/test".into(),
            },
        ] {
            let personal = matches!(target, Target::Personal);
            let sharepoint = matches!(target, Target::SharePoint { .. });
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output("{}")));
            runner.push(Ok(output(
                r#"{"State":"synthetic-private-state","Error":""}"#,
            )));
            runner.push(Ok(output(r#"{"State":"","Error":""}"#)));
            let mut remote =
                NewRemote::prepare(&runner, "synthetic", target, CancellationToken::new())
                    .await
                    .unwrap();
            remote
                .create(&runner, CancellationToken::new())
                .await
                .unwrap();
            remote
                .continue_with(
                    &runner,
                    "synthetic-private-state",
                    "synthetic-private-answer",
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            let requests = runner.requests();
            let create = requests[1].sanitized_command();
            assert!(create.contains(if sharepoint {
                "config_type url"
            } else {
                "config_type onedrive"
            }));
            assert_eq!(create.contains("User.Read"), personal);
            assert!(!create.contains("client_id"));
            assert!(!create.contains("client_secret"));
            for request in requests {
                assert!(request.timeout <= Duration::from_secs(300));
                assert!(request.output_limit <= 1024 * 1024);
                for text in [request.sanitized_command(), format!("{request:?}")] {
                    assert!(!text.contains("synthetic-private-state"));
                    assert!(!text.contains("synthetic-private-answer"));
                }
            }
        }
    }

    #[tokio::test]
    async fn microsoft_oauth_cancellation_before_creation_does_not_enable_rollback() {
        let runner = FakeCommandRunner::default();
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(matches!(
            NewRemote::prepare(&runner, "new", Target::Personal, cancelled.clone()).await,
            Err(OAuthError::Command(CommandFailure::Cancelled))
        ));
        assert!(runner.requests().is_empty());
        runner.push(Ok(output("{}")));
        let mut remote =
            NewRemote::prepare(&runner, "new", Target::Personal, CancellationToken::new())
                .await
                .unwrap();
        assert_eq!(
            remote.create(&runner, cancelled).await.unwrap_err(),
            OAuthError::Command(CommandFailure::Cancelled)
        );
        remote.rollback(&runner).await.unwrap();
        assert_eq!(runner.requests().len(), 1);
    }

    #[tokio::test]
    async fn microsoft_oauth_failed_creation_rolls_back_only_the_new_name() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"existing":{"type":"onedrive"}}"#)));
        runner.push(Ok(output(
            r#"{"State":"","Error":"synthetic-provider-secret"}"#,
        )));
        runner.push(Ok(output(
            r#"{"existing":{"type":"onedrive"},"new":{"type":"onedrive"}}"#,
        )));
        runner.push(Ok(output("")));
        let mut remote =
            NewRemote::prepare(&runner, "new", Target::Business, CancellationToken::new())
                .await
                .unwrap();
        let error = remote
            .create(&runner, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(error, OAuthError::Rejected);
        assert!(!format!("{error:?} {error}").contains("synthetic-provider-secret"));
        remote.rollback(&runner).await.unwrap();
        let requests = runner.requests();
        assert!(
            requests
                .last()
                .unwrap()
                .sanitized_command()
                .ends_with("config delete new")
        );
        assert!(
            !requests
                .iter()
                .any(|request| request.sanitized_command().contains("delete existing"))
        );
    }

    #[tokio::test]
    async fn microsoft_oauth_partial_cleanup_failure_requires_operator_attention() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(r#"{"State":"","Error":""}"#)));
        runner.push(Err(CommandError::InvalidArgument));
        let mut remote =
            NewRemote::prepare(&runner, "new", Target::Personal, CancellationToken::new())
                .await
                .unwrap();
        remote
            .create(&runner, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            remote.rollback(&runner).await.unwrap_err(),
            OAuthError::CleanupRequired
        );
        assert_eq!(runner.requests().len(), 3);
    }

    #[tokio::test]
    async fn microsoft_oauth_step_limit_is_bounded_for_both_flows() {
        for (target, maximum) in [
            (Target::Business, 12),
            (
                Target::SharePoint {
                    site_url: "https://example.sharepoint.com/sites/test".into(),
                },
                8,
            ),
        ] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output("{}")));
            for _ in 0..maximum {
                runner.push(Ok(output(r#"{"State":"state","Error":""}"#)));
            }
            let mut remote = NewRemote::prepare(&runner, "new", target, CancellationToken::new())
                .await
                .unwrap();
            remote
                .create(&runner, CancellationToken::new())
                .await
                .unwrap();
            for _ in 1..maximum {
                remote
                    .continue_with(&runner, "state", "true", CancellationToken::new())
                    .await
                    .unwrap();
            }
            assert_eq!(
                remote
                    .continue_with(&runner, "state", "true", CancellationToken::new())
                    .await
                    .unwrap_err(),
                OAuthError::TooManySteps
            );
            assert_eq!(runner.requests().len(), maximum + 1);
        }
    }

    #[test]
    fn microsoft_oauth_private_response_rejects_partial_data_and_accepts_browser_prefix() {
        let prefixed =
            output("launcher started\n{\"State\":\"done\",\"Error\":\"\"}\nlauncher done");
        assert_eq!(
            PrivateConfigOutput::test_output(prefixed)
                .oauth_json()
                .unwrap()["State"],
            "done"
        );
        for stream in [0, 1] {
            for invalid_utf8 in [false, true] {
                let mut response = output(r#"{"State":"","Error":""}"#);
                let stream = if stream == 0 {
                    &mut response.stdout
                } else {
                    &mut response.stderr
                };
                stream.truncated = !invalid_utf8;
                stream.invalid_utf8 = invalid_utf8;
                let private = PrivateConfigOutput::test_output(response);
                assert!(private.json().is_err());
                assert!(private.oauth_json().is_err());
            }
        }
        assert!(
            PrivateConfigOutput::test_output(output(r#"{"access_token":"synthetic"}"#))
                .oauth_json()
                .is_err()
        );
    }

    #[test]
    fn microsoft_oauth_command_failures_hide_provider_text_and_keep_exit_status() {
        let private = output("synthetic-private-provider-output");
        let error = OAuthError::from(CommandError::NonZero {
            command: "synthetic-private-command".into(),
            code: Some(2),
            stdout: private.stdout,
            stderr: private.stderr,
            attempts: 1,
        });
        assert_eq!(error, OAuthError::Command(CommandFailure::NonZero(Some(2))));
        assert!(!format!("{error:?} {error}").contains("synthetic-private"));
    }

    #[test]
    fn microsoft_oauth_token_extraction_accepts_encoded_and_object_tokens_only_for_microsoft() {
        for token in [
            json!({"access_token":"synthetic-access"}),
            json!(r#"{"access_token":"synthetic-access"}"#),
        ] {
            let dump = json!({"new":{"type":"onedrive","token":token}});
            assert_eq!(
                oauth_access_token(&dump.to_string(), "new").unwrap(),
                "synthetic-access"
            );
        }
        assert_eq!(
            oauth_access_token(r#"{"new":{"type":"drive"}}"#, "new").unwrap_err(),
            OAuthError::WrongBackend
        );
        assert_eq!(
            oauth_access_token(r#"{}"#, "new").unwrap_err(),
            OAuthError::RemoteNotFound
        );
        assert_eq!(
            access_token_from_remote(&json!({"token":"invalid"})).unwrap_err(),
            OAuthError::InvalidToken
        );
    }
}
