// SPDX-License-Identifier: MIT

//! Guided rclone browser authorization for one SharePoint document library.
//! The applet never stores OAuth tokens; rclone owns its remote configuration.

use std::time::Duration;

use serde_json::Value;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::fl;
use crate::model::TeamsLibraryIdentity;
use crate::process::{CommandError, CommandRequest, CommandRunner, Executable};
use crate::teams::{self, VerifiedTeamsLibrary};

const MAX_CONFIG_STEPS: usize = 8;
const MAX_DRIVE_CHOICES: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamsOAuthSetup {
    pub remote_name: String,
    pub library: TeamsLibraryIdentity,
}

impl TeamsOAuthSetup {
    pub fn new(remote_name: &str, library_url: &str) -> Result<Self, String> {
        let remote_name = remote_name.trim();
        if remote_name.is_empty()
            || remote_name.len() > 96
            || !remote_name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "_-.".contains(character))
        {
            return Err(fl!("teams-oauth-invalid-name"));
        }
        Ok(Self {
            remote_name: remote_name.to_owned(),
            library: teams::parse_library_url(library_url)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TeamsOAuthOutcome {
    pub remote_name: String,
    pub library_url: String,
    pub verified: VerifiedTeamsLibrary,
}

pub async fn create_teams_remote<R: CommandRunner>(
    runner: &R,
    setup: TeamsOAuthSetup,
) -> Result<TeamsOAuthOutcome, String> {
    if remote_exists(runner, &setup.remote_name).await? {
        return Err(fl!("teams-oauth-remote-exists", name = setup.remote_name));
    }
    let result = create_new_remote(runner, &setup).await;
    if let Err(error) = result {
        // This name was absent before creation. Remove a partial OAuth remote,
        // which could otherwise retain credentials without a verified drive.
        let cleanup = match remote_exists(runner, &setup.remote_name).await {
            Ok(false) => Ok(()),
            Ok(true) => runner
                .run(
                    rclone_config_request("delete", &setup, None, None)
                        .expect("validated remote name")
                        .with_timeout(Duration::from_secs(10)),
                    CancellationToken::new(),
                )
                .await
                .map(|_| ()),
            Err(_) => {
                return Err(fl!(
                    "teams-oauth-check-partial",
                    error = error,
                    name = setup.remote_name
                ));
            }
        };
        return match cleanup {
            Ok(()) => Err(error),
            Err(_) => Err(fl!(
                "teams-oauth-remove-partial",
                error = error,
                name = setup.remote_name
            )),
        };
    }
    result
}

async fn create_new_remote<R: CommandRunner>(
    runner: &R,
    setup: &TeamsOAuthSetup,
) -> Result<TeamsOAuthOutcome, String> {
    let mut request = rclone_config_request("create", setup, None, None)?;
    let mut selected_drive = false;
    for _ in 0..MAX_CONFIG_STEPS {
        let output = runner
            .run(request, CancellationToken::new())
            .await
            .map_err(oauth_command_error)?;
        let response = match parse_config_response(&output.stdout.text)
            .or_else(|_| parse_config_response(&output.stderr.text))
        {
            Ok(response) => response,
            Err(_) => {
                // Some rclone/browser combinations print a completion notice
                // instead of the final JSON state. Accept that only if the
                // newly created remote passes the full read-only identity gate.
                return teams::verify_remote(runner, &setup.remote_name, &setup.library)
                    .await
                    .map(|verified| TeamsOAuthOutcome {
                        remote_name: setup.remote_name.clone(),
                        library_url: setup.library.library_url.clone(),
                        verified,
                    })
                    .map_err(|verification_error| {
                        fl!(
                            "teams-oauth-unreadable-unverified",
                            stdout_bytes = output.stdout.text.len(),
                            stderr_bytes = output.stderr.text.len(),
                            error = verification_error
                        )
                    });
            }
        };
        if response
            .get("Error")
            .and_then(Value::as_str)
            .is_some_and(|error| !error.is_empty())
        {
            return Err(fl!("teams-oauth-setup-access"));
        }
        let state = response
            .get("State")
            .and_then(Value::as_str)
            .ok_or_else(|| fl!("teams-oauth-missing-state"))?;
        if state.is_empty() {
            let verified = teams::verify_remote(runner, &setup.remote_name, &setup.library).await?;
            return Ok(TeamsOAuthOutcome {
                remote_name: setup.remote_name.clone(),
                library_url: setup.library.library_url.clone(),
                verified,
            });
        }
        let option = response
            .get("Option")
            .ok_or_else(|| fl!("teams-oauth-missing-question"))?;
        let name = option
            .get("Name")
            .and_then(Value::as_str)
            .ok_or_else(|| fl!("teams-oauth-unnamed-question"))?;
        let answer = match name {
            "config_is_local" => "true".to_owned(),
            "config_type" => "url".to_owned(),
            "config_site_url" => setup.library.site_url.clone(),
            "config_driveid" => {
                let drive =
                    select_library_drive(runner, &setup.remote_name, &setup.library, option)
                        .await?;
                selected_drive = true;
                drive
            }
            "config_drive_ok" if selected_drive => "true".to_owned(),
            _ => return Err(fl!("teams-oauth-unsupported-choice")),
        };
        request = rclone_config_request("update", setup, Some(state), Some(&answer))?;
    }
    Err(fl!("teams-oauth-too-many-steps"))
}

fn parse_config_response(stdout: &str) -> Result<Value, String> {
    // A browser launcher can write a short diagnostic around rclone's JSON.
    // Accept only an object with rclone's state-machine shape and never echo
    // stdout, which can contain OAuth details.
    for (start, _) in stdout.match_indices('{').take(32) {
        let mut values = serde_json::Deserializer::from_str(&stdout[start..]).into_iter::<Value>();
        if let Some(Ok(value)) = values.next()
            && value.get("State").and_then(Value::as_str).is_some()
            && value.get("Error").and_then(Value::as_str).is_some()
        {
            return Ok(value);
        }
    }
    Err(fl!("teams-oauth-unreadable-response"))
}

async fn remote_exists<R: CommandRunner>(runner: &R, name: &str) -> Result<bool, String> {
    let output = runner
        .run(config_dump_request()?, CancellationToken::new())
        .await
        .map_err(oauth_command_error)?;
    let dump: Value = serde_json::from_str(&output.stdout.text)
        .map_err(|_| fl!("teams-oauth-inspect-remotes"))?;
    Ok(dump.get(name).is_some())
}

fn config_dump_request() -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("config")
        .and_then(|request| request.arg("dump"))
        .map(|request| {
            request
                .with_timeout(Duration::from_secs(10))
                .with_output_limit(1024 * 1024)
        })
        .map_err(|error| error.to_string())
}

fn rclone_config_request(
    action: &str,
    setup: &TeamsOAuthSetup,
    state: Option<&str>,
    result: Option<&str>,
) -> Result<CommandRequest, String> {
    let mut request = CommandRequest::new(Executable::Rclone);
    let mut arguments = vec!["config", action, setup.remote_name.as_str()];
    if action != "delete" {
        if action == "create" {
            arguments.push("onedrive");
        } else {
            arguments.push("--continue");
        }
        arguments.extend([
            "config_is_local",
            "true",
            "config_type",
            "url",
            "config_site_url",
            setup.library.site_url.as_str(),
        ]);
    }
    for argument in arguments {
        request = request.arg(argument).map_err(|error| error.to_string())?;
    }
    if let (Some(state), Some(result)) = (state, result) {
        request = request
            .arg("--state")
            .and_then(|request| request.arg(state))
            .and_then(|request| request.arg("--result"))
            .and_then(|request| request.arg(result))
            .map_err(|error| error.to_string())?;
    }
    if action != "delete" {
        request = request
            .arg("--non-interactive")
            .map_err(|error| error.to_string())?;
    }
    Ok(request
        .with_timeout(Duration::from_secs(5 * 60))
        .with_output_limit(256 * 1024))
}

async fn select_library_drive<R: CommandRunner>(
    runner: &R,
    remote_name: &str,
    expected: &TeamsLibraryIdentity,
    option: &Value,
) -> Result<String, String> {
    let examples = option
        .get("Examples")
        .and_then(Value::as_array)
        .ok_or_else(|| fl!("teams-oauth-no-library-list"))?;
    if examples.is_empty() || examples.len() > MAX_DRIVE_CHOICES {
        return Err(fl!("teams-oauth-library-count"));
    }
    let output = runner
        .run(config_dump_request()?, CancellationToken::new())
        .await
        .map_err(oauth_command_error)?;
    let token = teams::oauth_access_token(&output.stdout.text, remote_name)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| fl!("teams-oauth-discovery-prepare"))?;
    let mut matching = Vec::new();
    for example in examples {
        let Some(drive_id) = example.get("Value").and_then(Value::as_str) else {
            continue;
        };
        let mut url =
            Url::parse("https://graph.microsoft.com/v1.0/drives/").expect("fixed Graph URL");
        url.path_segments_mut()
            .expect("Graph URL has path")
            .push(drive_id);
        let response = client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|_| fl!("teams-oauth-discovery-contact"))?;
        if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
            return Err(fl!("teams-oauth-discovery-denied"));
        }
        if !response.status().is_success() {
            continue;
        }
        let Ok(metadata) = response.json::<Value>().await else {
            continue;
        };
        if teams::verify_graph_drive(&metadata, expected, drive_id).is_ok() {
            matching.push(drive_id.to_owned());
        }
    }
    match matching.as_slice() {
        [drive_id] => Ok(drive_id.clone()),
        [] => Err(fl!("teams-oauth-no-matching-library")),
        _ => Err(fl!("teams-oauth-multiple-libraries")),
    }
}

fn oauth_command_error(error: CommandError) -> String {
    match error {
        CommandError::MissingExecutable(_) => {
            fl!("teams-oauth-rclone-missing")
        }
        CommandError::Timeout { .. } => {
            fl!("teams-oauth-timeout")
        }
        CommandError::Cancelled { .. } => fl!("teams-oauth-cancelled"),
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.is_empty() {
                &stdout.text
            } else {
                &stderr.text
            };
            let lower = detail.to_ascii_lowercase();
            if lower.contains("consent") || lower.contains("aadsts65001") {
                fl!("teams-oauth-consent-required")
            } else if lower.contains("invalid_grant") || lower.contains("unauthorized") {
                fl!("teams-oauth-authorization-failed")
            } else {
                fl!("teams-oauth-rclone-failed")
            }
        }
        _ => fl!("teams-oauth-start-failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};

    fn output(stdout: &str) -> CommandOutput {
        CommandOutput {
            command: "rclone config".into(),
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
    fn setup_uses_a_library_url_and_new_remote_name() {
        let setup = TeamsOAuthSetup::new(
            "teams_assessment_new",
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents",
        )
        .unwrap();
        assert_eq!(
            setup.library.site_url,
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment"
        );
        assert!(TeamsOAuthSetup::new("bad:name", &setup.library.library_url).is_err());
    }

    #[test]
    fn oauth_create_never_updates_an_existing_remote() {
        let setup = TeamsOAuthSetup::new(
            "new_teams",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let request = rclone_config_request("create", &setup, None, None).unwrap();
        assert!(
            request
                .sanitized_command()
                .contains("config create new_teams onedrive")
        );
        assert!(request.sanitized_command().contains("config_type url"));
        let next = rclone_config_request("update", &setup, Some("state"), Some("true")).unwrap();
        assert!(
            next.sanitized_command()
                .contains("config update new_teams --continue")
        );
        assert!(
            next.sanitized_command()
                .contains("--state state --result true")
        );
    }

    #[test]
    fn setup_response_parser_ignores_browser_launcher_text_but_requires_rclone_shape() {
        let response = parse_config_response(
            "browser helper started\n{\"State\":\"driveid_final\",\"Option\":{\"Name\":\"config_driveid\"},\"Error\":\"\"}\nlauncher done",
        )
        .unwrap();
        assert_eq!(response["State"], "driveid_final");
        assert!(parse_config_response("{\"access_token\":\"secret\"}").is_err());
    }

    #[tokio::test]
    async fn existing_remote_is_rejected_without_create_or_delete() {
        let setup = TeamsOAuthSetup::new(
            "existing",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"existing":{"type":"onedrive"}}"#)));
        let error = create_teams_remote(&runner, setup).await.unwrap_err();
        assert!(error.contains("already exists"));
        assert_eq!(runner.requests().len(), 1);
        assert!(
            runner.requests()[0]
                .sanitized_command()
                .contains("config dump")
        );
    }

    #[tokio::test]
    async fn unsupported_question_removes_only_new_partial_remote() {
        let setup = TeamsOAuthSetup::new(
            "new_teams",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(
            r#"{"State":"unexpected","Option":{"Name":"secret_prompt"},"Error":""}"#,
        )));
        runner.push(Ok(output(r#"{"new_teams":{"type":"onedrive"}}"#)));
        runner.push(Ok(output("")));
        let error = create_teams_remote(&runner, setup).await.unwrap_err();
        assert!(error.contains("unsupported"));
        let commands: Vec<_> = runner
            .requests()
            .iter()
            .map(CommandRequest::sanitized_command)
            .collect();
        assert!(commands[1].contains("config create new_teams"));
        assert!(commands[2].contains("config dump"));
        assert!(commands[3].contains("config delete new_teams"));
    }

    #[tokio::test]
    async fn unreadable_response_cannot_accept_an_unverified_remote() {
        let setup = TeamsOAuthSetup::new(
            "new_teams",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output("browser completed without a setup state")));
        runner.push(Err(CommandError::InvalidArgument));
        runner.push(Ok(output(r#"{"new_teams":{"type":"onedrive"}}"#)));
        runner.push(Ok(output("")));
        let error = create_teams_remote(&runner, setup).await.unwrap_err();
        assert!(error.contains("did not verify"));
        assert!(
            runner.requests()[4]
                .sanitized_command()
                .contains("config delete new_teams")
        );
    }
}
