// SPDX-License-Identifier: MIT

//! Guided rclone browser authorization for one SharePoint document library.
//! The applet never stores OAuth tokens; rclone owns its remote configuration.

use std::time::Duration;

use serde_json::Value;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::fl;
use crate::microsoft_oauth::{self, NewRemote, OAuthError, Target};
use crate::model::SharePointLibraryIdentity;
use crate::process::CommandRunner;
use crate::sharepoint::{self, VerifiedSharePointLibrary};

const MAX_CONFIG_STEPS: usize = 8;
const MAX_DRIVE_CHOICES: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharePointSetup {
    pub remote_name: String,
    pub library: SharePointLibraryIdentity,
}

impl SharePointSetup {
    pub fn new(remote_name: &str, library_url: &str) -> Result<Self, String> {
        let remote_name = remote_name.trim();
        microsoft_oauth::validate_remote_name(remote_name)
            .map_err(|_| fl!("teams-oauth-invalid-name"))?;
        Ok(Self {
            remote_name: remote_name.to_owned(),
            library: sharepoint::parse_library_url(library_url)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SharePointSetupOutcome {
    pub remote_name: String,
    pub library_url: String,
    pub verified: VerifiedSharePointLibrary,
}

pub async fn create_sharepoint_remote<R: CommandRunner>(
    runner: &R,
    setup: SharePointSetup,
) -> Result<SharePointSetupOutcome, String> {
    let mut remote = NewRemote::prepare(
        runner,
        &setup.remote_name,
        Target::SharePoint {
            site_url: setup.library.site_url.clone(),
        },
        CancellationToken::new(),
    )
    .await
    .map_err(|error| error.to_string())?;
    let result = create_new_remote(runner, &setup, &mut remote).await;
    if let Err(error) = result {
        if remote.rollback(runner).await.is_err() {
            return Err(fl!(
                "teams-oauth-remove-partial",
                error = error,
                name = setup.remote_name
            ));
        }
        return Err(error);
    }
    result
}

async fn create_new_remote<R: CommandRunner>(
    runner: &R,
    setup: &SharePointSetup,
    remote: &mut NewRemote,
) -> Result<SharePointSetupOutcome, String> {
    let mut next = remote.create(runner, CancellationToken::new()).await;
    let mut selected_drive = false;
    for _ in 0..MAX_CONFIG_STEPS {
        let response = match next {
            Ok(response) => response,
            Err(OAuthError::UnstructuredResponse) => {
                // Plain-text completion is accepted only after full identity verification.
                return sharepoint::verify_remote(runner, &setup.remote_name, &setup.library)
                    .await
                    .map(|verified| SharePointSetupOutcome {
                        remote_name: setup.remote_name.clone(),
                        library_url: setup.library.library_url.clone(),
                        verified,
                    })
                    .map_err(|error| fl!("microsoft-oauth-unverified", error = error));
            }
            Err(error) => return Err(error.to_string()),
        };
        let state = response
            .get("State")
            .and_then(Value::as_str)
            .ok_or_else(|| fl!("teams-oauth-missing-state"))?;
        if state.is_empty() {
            let verified =
                sharepoint::verify_remote(runner, &setup.remote_name, &setup.library).await?;
            return Ok(SharePointSetupOutcome {
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
        next = remote
            .continue_with(runner, state, &answer, CancellationToken::new())
            .await;
    }
    Err(fl!("teams-oauth-too-many-steps"))
}

async fn select_library_drive<R: CommandRunner>(
    runner: &R,
    remote_name: &str,
    expected: &SharePointLibraryIdentity,
    option: &Value,
) -> Result<String, String> {
    let examples = option
        .get("Examples")
        .and_then(Value::as_array)
        .ok_or_else(|| fl!("teams-oauth-no-library-list"))?;
    if examples.is_empty() || examples.len() > MAX_DRIVE_CHOICES {
        return Err(fl!("teams-oauth-library-count"));
    }
    let dump = microsoft_oauth::config_dump(runner, CancellationToken::new())
        .await
        .map_err(|error| error.to_string())?;
    let token = microsoft_oauth::oauth_access_token(&dump.to_string(), remote_name)
        .map_err(|error| error.to_string())?;
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
        if sharepoint::verify_graph_drive(&metadata, expected, drive_id).is_ok() {
            matching.push(drive_id.to_owned());
        }
    }
    match matching.as_slice() {
        [drive_id] => Ok(drive_id.clone()),
        [] => Err(fl!("teams-oauth-no-matching-library")),
        _ => Err(fl!("teams-oauth-multiple-libraries")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{
        CapturedOutput, CommandError, CommandOutput, CommandRequest, FakeCommandRunner,
        PrivateConfigOutput,
    };

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
        let setup = SharePointSetup::new(
            "teams_assessment_new",
            "https://example.sharepoint.com/sites/assessment/Shared%20Documents",
        )
        .unwrap();
        assert_eq!(
            setup.library.site_url,
            "https://example.sharepoint.com/sites/assessment"
        );
        assert!(SharePointSetup::new("bad:name", &setup.library.library_url).is_err());
    }

    #[tokio::test]
    async fn oauth_create_uses_shared_transport_and_protects_continuation_state() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(r#"{"State":"private-state","Error":""}"#)));
        runner.push(Ok(output(r#"{"State":"","Error":""}"#)));
        let mut remote = NewRemote::prepare(
            &runner,
            "new_teams",
            Target::SharePoint {
                site_url: "https://example.sharepoint.com/sites/team".into(),
            },
            CancellationToken::new(),
        )
        .await
        .unwrap();
        remote
            .create(&runner, CancellationToken::new())
            .await
            .unwrap();
        remote
            .continue_with(
                &runner,
                "private-state",
                "private-answer",
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let requests = runner.requests();
        let create = requests[1].sanitized_command();
        assert!(create.contains("config create new_teams onedrive"));
        assert!(create.contains("config_type url"));
        let next = requests[2].sanitized_command();
        assert!(next.contains("config update new_teams --continue"));
        assert!(!next.contains("private-state"));
        assert!(!next.contains("private-answer"));
    }

    #[test]
    fn setup_response_parser_ignores_browser_launcher_text_but_requires_rclone_shape() {
        let response = PrivateConfigOutput::test_output(output(
            "browser helper started\n{\"State\":\"driveid_final\",\"Option\":{\"Name\":\"config_driveid\"},\"Error\":\"\"}\nlauncher done",
        )).oauth_json()
        .unwrap();
        assert_eq!(response["State"], "driveid_final");
        assert!(
            PrivateConfigOutput::test_output(output("{\"access_token\":\"synthetic\"}"))
                .oauth_json()
                .is_err()
        );
    }

    #[tokio::test]
    async fn existing_remote_is_rejected_without_create_or_delete() {
        let setup = SharePointSetup::new(
            "existing",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output(r#"{"existing":{"type":"onedrive"}}"#)));
        let error = create_sharepoint_remote(&runner, setup).await.unwrap_err();
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
        let setup = SharePointSetup::new(
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
        let error = create_sharepoint_remote(&runner, setup).await.unwrap_err();
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
    async fn oversized_response_never_falls_back_to_library_verification() {
        let setup = SharePointSetup::new(
            "new_teams",
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        let mut incomplete = output(r#"{"State":"","Error":""}"#);
        incomplete.stdout.truncated = true;
        runner.push(Ok(incomplete));
        runner.push(Ok(output(r#"{"new_teams":{}}"#)));
        runner.push(Ok(output("")));
        assert!(create_sharepoint_remote(&runner, setup).await.is_err());
        let requests = runner.requests();
        assert_eq!(requests.len(), 4);
        assert!(
            !requests
                .iter()
                .any(|request| request.sanitized_command().contains("lsf"))
        );
        assert!(
            requests[3]
                .sanitized_command()
                .contains("config delete new_teams")
        );
    }

    #[tokio::test]
    async fn unreadable_response_cannot_accept_an_unverified_remote() {
        let setup = SharePointSetup::new(
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
        let error = create_sharepoint_remote(&runner, setup).await.unwrap_err();
        assert!(error.contains("did not verify"));
        assert!(
            runner.requests()[4]
                .sanitized_command()
                .contains("config delete new_teams")
        );
    }
}
