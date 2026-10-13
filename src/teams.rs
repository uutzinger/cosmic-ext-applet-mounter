// SPDX-License-Identifier: MIT

//! Authenticated identity checks for SharePoint document libraries.

use std::path::PathBuf;
use std::time::Duration;

use percent_encoding::percent_decode_str;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::config::{Config, LoadSource};
use crate::model::{Connection, ConnectionId, ConnectionMode, Provider, TeamsLibraryIdentity};
use crate::mount_guard;
use crate::process::{
    CommandError, CommandRequest, CommandRunner, Executable, RuntimeCommandRunner,
};
use crate::services::ServiceCommand;

pub const SERVICE_GUARD_FLAG: &str = "--verify-teams-mount";
pub const MIRROR_GUARD_FLAG: &str = "--verify-teams-mirror";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceGuardBinding {
    pub connection_id: ConnectionId,
    pub remote_name: String,
    pub remote_subpath: Option<String>,
    pub mountpoint: PathBuf,
    pub drive_id: String,
    pub library_url: String,
}

impl ServiceGuardBinding {
    pub fn from_connection(connection: &Connection) -> Result<Self, String> {
        if connection.provider != Provider::Teams
            || !matches!(connection.mode, ConnectionMode::OnlineMount(_))
        {
            return Err(crate::fl!(
                "ui-teams-the-service-guard-requires-a-sharepoint-online"
            ));
        }
        let identity = connection.teams_identity.as_ref().ok_or_else(|| {
            crate::fl!("ui-teams-the-sharepoint-connection-has-no-verified-library")
        })?;
        if identity.drive_id.trim().is_empty()
            || parse_library_url(&identity.library_url)
                .map_or(true, |parsed| parsed.site_url != identity.site_url)
        {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-connection-has-an-invalid-verified"
            ));
        }
        Ok(Self {
            connection_id: connection.id,
            remote_name: connection.remote_reference.clone(),
            remote_subpath: connection.remote_subpath.clone(),
            mountpoint: connection.local_path.clone(),
            drive_id: identity.drive_id.clone(),
            library_url: identity.library_url.clone(),
        })
    }

    fn arguments(&self) -> Vec<String> {
        vec![
            SERVICE_GUARD_FLAG.into(),
            self.connection_id.to_string(),
            self.remote_name.clone(),
            serde_json::to_string(&self.remote_subpath)
                .expect("an optional remote path can always be serialized"),
            self.mountpoint.display().to_string(),
            self.drive_id.clone(),
            self.library_url.clone(),
        ]
    }

    pub fn parse_arguments(arguments: &[String]) -> Result<Self, String> {
        let [
            id,
            remote_name,
            encoded_subpath,
            mountpoint,
            drive_id,
            library_url,
        ] = arguments
        else {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-service-guard-received-invalid-arguments"
            ));
        };
        let connection_id = uuid::Uuid::parse_str(id)
            .map(ConnectionId::from_uuid)
            .map_err(|_| "The SharePoint service guard received an invalid connection ID.")?;
        if remote_name.is_empty() || mountpoint.is_empty() || drive_id.is_empty() {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-service-guard-received-incomplete-identity"
            ));
        }
        Ok(Self {
            connection_id,
            remote_name: remote_name.clone(),
            remote_subpath: serde_json::from_str(encoded_subpath)
                .map_err(|_| "The SharePoint service guard received an invalid remote folder.")?,
            mountpoint: PathBuf::from(mountpoint),
            drive_id: drive_id.clone(),
            library_url: library_url.clone(),
        })
    }

    pub fn matches_saved_connection(&self, connection: &Connection) -> Result<(), String> {
        if !connection.enabled || Self::from_connection(connection)? != *self {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-connection-changed-or-was-disabled"
            ));
        }
        Ok(())
    }
}

pub fn service_guard_command(connection: &Connection) -> Result<ServiceCommand, String> {
    let binding = ServiceGuardBinding::from_connection(connection)?;
    mount_guard::service_command(binding.arguments())
}

pub async fn run_service_guard(arguments: &[String]) -> Result<(), String> {
    let binding = ServiceGuardBinding::parse_arguments(arguments)?;
    let report = Config::load_runtime();
    if report.source != LoadSource::Current {
        return Err(crate::fl!(
            "ui-teams-could-not-load-the-saved-sharepoint-connection"
        ));
    }
    let connection = report
        .config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == binding.connection_id)
        .ok_or_else(|| crate::fl!("ui-teams-the-sharepoint-connection-was-removed-mount-was"))?;
    binding.matches_saved_connection(connection)?;
    mount_guard::check_empty_mountpoint(
        &RuntimeCommandRunner::detect_current(),
        &binding.mountpoint,
    )
    .await?;
    let expected = connection
        .teams_identity
        .as_ref()
        .ok_or_else(|| crate::fl!("ui-teams-the-sharepoint-connection-has-no-verified-library"))?;
    verify_remote(
        &RuntimeCommandRunner::detect_current(),
        &binding.remote_name,
        expected,
    )
    .await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorGuardBinding {
    pub connection_id: ConnectionId,
    pub remote_name: String,
    pub remote_subpath: String,
    pub local_path: PathBuf,
    pub recovery_directory: PathBuf,
    pub drive_id: String,
    pub site_url: String,
    pub library_url: String,
}

impl MirrorGuardBinding {
    pub fn from_connection(connection: &Connection) -> Result<Self, String> {
        let ConnectionMode::OfflineMirror(options) = &connection.mode else {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-mirror-guard-requires-an-offline"
            ));
        };
        if connection.provider != Provider::Teams {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-mirror-guard-requires-a-sharepoint"
            ));
        }
        let subpath = connection
            .remote_subpath
            .as_deref()
            .ok_or(crate::fl!("ui-teams-select-a-folder-before-mirroring"))?;
        crate::sync::validate_remote_name(&connection.remote_reference)
            .map_err(|error| error.to_string())?;
        crate::sync::validate_remote_subpath(subpath).map_err(|error| error.to_string())?;
        if subpath
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return Err(crate::fl!(
                "ui-teams-select-a-folder-within-the-sharepoint-library"
            ));
        }
        let identity = connection.teams_identity.as_ref().ok_or(crate::fl!(
            "ui-teams-the-sharepoint-connection-has-no-verified-library"
        ))?;
        if identity.drive_id.trim().is_empty()
            || parse_library_url(&identity.library_url).map_or(true, |parsed| {
                parsed.site_url != identity.site_url || parsed.library_url != identity.library_url
            })
        {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-connection-has-an-invalid-verified"
            ));
        }
        Ok(Self {
            connection_id: connection.id,
            remote_name: connection.remote_reference.clone(),
            remote_subpath: subpath.to_owned(),
            local_path: connection.local_path.clone(),
            recovery_directory: options.recovery_directory.clone(),
            drive_id: identity.drive_id.clone(),
            site_url: identity.site_url.clone(),
            library_url: identity.library_url.clone(),
        })
    }

    pub fn matches_saved_connection(&self, connection: &Connection) -> Result<(), String> {
        if !connection.enabled || Self::from_connection(connection)? != *self {
            return Err(crate::fl!(
                "ui-teams-the-sharepoint-mirror-target-changed-or-was"
            ));
        }
        Ok(())
    }
}

pub fn mirror_guard_command(connection: &Connection) -> Result<ServiceCommand, String> {
    let binding = MirrorGuardBinding::from_connection(connection)?;
    mount_guard::service_command(vec![
        MIRROR_GUARD_FLAG.into(),
        serde_json::to_string(&binding)
            .map_err(|_| crate::fl!("ui-teams-could-not-encode-the-sharepoint-mirror-identity"))?,
    ])
}

pub async fn run_mirror_service_guard(arguments: &[String]) -> Result<(), String> {
    let [encoded] = arguments else {
        return Err(crate::fl!(
            "ui-teams-the-sharepoint-mirror-guard-received-invalid-arguments"
        ));
    };
    let binding: MirrorGuardBinding = serde_json::from_str(encoded)
        .map_err(|_| crate::fl!("ui-teams-the-sharepoint-mirror-guard-received-an-invalid"))?;
    let report = Config::load_runtime();
    if report.source != LoadSource::Current {
        return Err(crate::fl!(
            "ui-teams-could-not-load-the-saved-sharepoint-mirror"
        ));
    }
    let connection = report
        .config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == binding.connection_id)
        .ok_or(crate::fl!(
            "ui-teams-the-sharepoint-mirror-was-removed-synchronization-was"
        ))?;
    verify_mirror_binding(
        &binding,
        connection,
        &RuntimeCommandRunner::detect_current(),
    )
    .await
}

pub async fn verify_mirror_binding<R: CommandRunner>(
    binding: &MirrorGuardBinding,
    connection: &Connection,
    runner: &R,
) -> Result<(), String> {
    binding.matches_saved_connection(connection)?;
    let expected = connection.teams_identity.as_ref().ok_or(crate::fl!(
        "ui-teams-the-sharepoint-connection-has-no-verified-library"
    ))?;
    verify_remote(runner, &binding.remote_name, expected).await?;
    let target = format!("{}:{}", binding.remote_name, binding.remote_subpath);
    runner
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("lsf")
                .and_then(|request| request.arg(&target))
                .and_then(|request| request.arg("--max-depth"))
                .and_then(|request| request.arg("1"))
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(20))
                .with_output_limit(16 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_read_error(&error, true))?;
    Ok(())
}

fn decoded_segment(segment: &str) -> Result<String, String> {
    let bytes = segment.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (bytes
                .get(index + 1)
                .is_none_or(|value| !value.is_ascii_hexdigit())
                || bytes
                    .get(index + 2)
                    .is_none_or(|value| !value.is_ascii_hexdigit()))
        {
            return Err(crate::fl!(
                "ui-teams-sharepoint-url-has-invalid-percent-encoding"
            ));
        }
    }
    let decoded = percent_decode_str(segment)
        .decode_utf8()
        .map_err(|_| crate::fl!("ui-teams-sharepoint-url-has-invalid-percent-encoding"))?;
    if decoded.is_empty()
        || decoded == "."
        || decoded == ".."
        || decoded.contains(['/', '\\', '\0'])
    {
        return Err(crate::fl!(
            "ui-teams-sharepoint-url-contains-an-unsafe-path-segment"
        ));
    }
    Ok(decoded.into_owned())
}

pub fn parse_library_url(input: &str) -> Result<TeamsLibraryIdentity, String> {
    let url = Url::parse(input.trim())
        .map_err(|_| crate::fl!("ui-teams-enter-a-sharepoint-library-url"))?;
    let host = url
        .host_str()
        .ok_or_else(|| crate::fl!("ui-teams-sharepoint-url-needs-a-hostname"))?;
    if url.scheme() != "https"
        || !(host.ends_with(".sharepoint.com") || host == "sharepoint.com")
        || url.username() != ""
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err(crate::fl!(
            "ui-teams-use-an-https-sharepoint-library-url-without"
        ));
    }
    let mut raw_segments = url
        .path_segments()
        .ok_or_else(|| crate::fl!("ui-teams-sharepoint-url-needs-a-site-and-library"))?
        .collect::<Vec<_>>();
    if raw_segments.last() == Some(&"") {
        raw_segments.pop();
    }
    let segments = raw_segments
        .into_iter()
        .map(decoded_segment)
        .collect::<Result<Vec<_>, _>>()?;
    if segments.len() != 3
        || !matches!(segments[0].to_ascii_lowercase().as_str(), "sites" | "teams")
    {
        return Err(crate::fl!("ui-teams-enter-a-library-url-under-sites-site"));
    }
    let mut canonical = Url::parse(&format!("https://{host}"))
        .map_err(|_| crate::fl!("ui-teams-sharepoint-url-has-an-invalid-host"))?;
    canonical
        .path_segments_mut()
        .map_err(|_| crate::fl!("ui-teams-sharepoint-url-has-an-invalid-path"))?
        .extend(segments.iter().take(2).map(String::as_str));
    let site_url = canonical.as_str().trim_end_matches('/').to_owned();
    canonical
        .path_segments_mut()
        .map_err(|_| crate::fl!("ui-teams-sharepoint-url-has-an-invalid-path"))?
        .push(&segments[2]);
    let library_url = canonical.as_str().trim_end_matches('/').to_owned();
    Ok(TeamsLibraryIdentity {
        site_url,
        library_url,
        drive_id: String::new(),
    })
}

pub fn same_library_url(left: &str, right: &str) -> bool {
    let Ok(left) = parse_library_url(left) else {
        return false;
    };
    let Ok(right) = parse_library_url(right) else {
        return false;
    };
    left.library_url.eq_ignore_ascii_case(&right.library_url)
}

struct RemoteCredentials {
    drive_id: String,
    access_token: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedTeamsLibrary {
    pub identity: TeamsLibraryIdentity,
    pub account: Option<String>,
}

fn remote_credentials(config_json: &str, remote_name: &str) -> Result<RemoteCredentials, String> {
    let dump: Value = serde_json::from_str(config_json)
        .map_err(|_| crate::fl!("ui-teams-rclone-returned-invalid-configuration-data"))?;
    let remote = dump.get(remote_name).ok_or_else(|| {
        crate::fl!(
            "ui-teams-rclone-remote-remote-name-was-not-found",
            remote_name = (remote_name).to_string()
        )
    })?;
    if remote.get("type").and_then(Value::as_str) != Some("onedrive")
        || remote.get("drive_type").and_then(Value::as_str) != Some("documentLibrary")
    {
        return Err(crate::fl!(
            "ui-teams-choose-a-rclone-onedrive-remote-configured-for"
        ));
    }
    let drive_id = remote
        .get("drive_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| crate::fl!("ui-teams-the-sharepoint-remote-has-no-drive-id"))?;
    let access_token = access_token_from_remote(remote)?;
    Ok(RemoteCredentials {
        drive_id: drive_id.to_owned(),
        access_token,
    })
}

/// Read a transient token from rclone's configuration response while a new
/// Teams remote is still in its drive-selection step. Never persist or log it.
pub fn oauth_access_token(config_json: &str, remote_name: &str) -> Result<String, String> {
    let dump: Value = serde_json::from_str(config_json)
        .map_err(|_| crate::fl!("ui-teams-rclone-returned-invalid-configuration-data"))?;
    let remote = dump.get(remote_name).ok_or_else(|| {
        crate::fl!(
            "ui-teams-rclone-remote-remote-name-was-not-found",
            remote_name = (remote_name).to_string()
        )
    })?;
    if remote.get("type").and_then(Value::as_str) != Some("onedrive") {
        return Err(crate::fl!("ui-teams-the-new-remote-is-not-a-microsoft"));
    }
    access_token_from_remote(remote)
}

fn access_token_from_remote(remote: &Value) -> Result<String, String> {
    let token = remote
        .get("token")
        .ok_or_else(|| crate::fl!("ui-teams-the-sharepoint-remote-is-not-authenticated"))?;
    let token = if let Some(text) = token.as_str() {
        serde_json::from_str::<Value>(text)
            .map_err(|_| crate::fl!("ui-teams-the-sharepoint-remote-has-an-invalid-token"))?
    } else {
        token.clone()
    };
    let access_token = token
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            crate::fl!("ui-teams-the-sharepoint-remote-needs-browser-reauthorization")
        })?;
    Ok(access_token.to_owned())
}

pub fn verify_graph_drive(
    graph_response: &Value,
    expected: &TeamsLibraryIdentity,
    configured_drive_id: &str,
) -> Result<TeamsLibraryIdentity, String> {
    if graph_response.get("id").and_then(Value::as_str) != Some(configured_drive_id)
        || graph_response.get("driveType").and_then(Value::as_str) != Some("documentLibrary")
    {
        return Err(crate::fl!(
            "ui-teams-microsoft-returned-a-different-drive-or-a"
        ));
    }
    let actual_url = graph_response
        .get("webUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| crate::fl!("ui-teams-microsoft-did-not-return-the-document-library"))?;
    if !same_library_url(&expected.library_url, actual_url) {
        return Err(crate::fl!(
            "ui-teams-the-rclone-remote-points-to-a-different"
        ));
    }
    if !expected.drive_id.is_empty() && expected.drive_id != configured_drive_id {
        return Err(crate::fl!("ui-teams-the-rclone-remote-s-drive-id-changed"));
    }
    let mut verified = expected.clone();
    verified.drive_id = configured_drive_id.to_owned();
    Ok(verified)
}

/// Classify rclone's read-only SharePoint failures without echoing its output.
/// It can contain remote paths, account details, or credential-bearing options.
pub fn rclone_read_error(error: &CommandError, folder_selected: bool) -> String {
    match error {
        CommandError::MissingExecutable(_) => {
            crate::fl!("ui-teams-rclone-is-missing-install-it-before-testing")
        }
        CommandError::InvalidArgument => {
            crate::fl!("ui-teams-sharepoint-remote-or-folder-contains-an-invalid")
        }
        CommandError::Timeout { .. } => {
            crate::fl!("ui-teams-the-sharepoint-read-only-access-check-timed")
        }
        CommandError::Cancelled { .. } => {
            crate::fl!("ui-teams-the-sharepoint-access-check-was-cancelled")
        }
        CommandError::Spawn { .. } => {
            crate::fl!("ui-teams-could-not-start-rclone-for-the-sharepoint")
        }
        CommandError::NonZero { stderr, stdout, .. } => {
            let detail = if stderr.text.trim().is_empty() {
                &stdout.text
            } else {
                &stderr.text
            }
            .to_ascii_lowercase();
            if detail.contains("aadsts65001")
                || detail.contains("consent_required")
                || detail.contains("admin consent")
                || detail.contains("consent is required")
            {
                crate::fl!("ui-teams-microsoft-tenant-consent-is-required-for-this")
            } else if detail.contains("invalid_grant")
                || detail.contains("token expired")
                || detail.contains("expired token")
                || detail.contains("couldn't fetch token")
                || detail.contains("unauthorized")
                || detail.contains("401")
            {
                crate::fl!("ui-teams-sharepoint-rclone-authorization-expired-or-was-revoked")
            } else if detail.contains("429")
                || detail.contains("503")
                || detail.contains("too many requests")
                || detail.contains("throttl")
                || detail.contains("server too busy")
            {
                crate::fl!("ui-teams-sharepoint-is-limiting-requests-wait-before-retrying")
            } else if detail.contains("403")
                || detail.contains("accessdenied")
                || detail.contains("access denied")
                || detail.contains("forbidden")
                || detail.contains("permission denied")
            {
                crate::fl!("ui-teams-the-signed-in-account-cannot-read-this")
            } else if detail.contains("404")
                || detail.contains("itemnotfound")
                || detail.contains("directory not found")
                || detail.contains("object not found")
                || detail.contains("not found")
            {
                if folder_selected {
                    crate::fl!("ui-teams-the-selected-sharepoint-folder-was-not-found")
                } else {
                    crate::fl!("ui-teams-the-sharepoint-library-is-unavailable-to-this")
                }
            } else if detail.contains("network")
                || detail.contains("connection")
                || detail.contains("timeout")
                || detail.contains("no route")
                || detail.contains("dns")
            {
                crate::fl!("ui-teams-could-not-reach-sharepoint-check-network-or")
            } else {
                crate::fl!("ui-teams-the-sharepoint-read-only-access-check-failed")
            }
        }
    }
}

pub async fn verify_remote<R: CommandRunner>(
    runner: &R,
    remote_name: &str,
    expected: &TeamsLibraryIdentity,
) -> Result<VerifiedTeamsLibrary, String> {
    // A root listing lets rclone refresh expired OAuth credentials before we
    // consume its current token. No file data is read or changed.
    runner
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("lsf")
                .and_then(|request| request.arg(format!("{remote_name}:")))
                .and_then(|request| request.arg("--max-depth"))
                .and_then(|request| request.arg("1"))
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(25))
                .with_output_limit(16 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|error| rclone_read_error(&error, false))?;
    let output = runner
        .run(
            CommandRequest::new(Executable::Rclone)
                .arg("config")
                .and_then(|request| request.arg("dump"))
                .map_err(|error| error.to_string())?
                .with_timeout(Duration::from_secs(10))
                .with_output_limit(1024 * 1024),
            CancellationToken::new(),
        )
        .await
        .map_err(|_| crate::fl!("ui-teams-could-not-inspect-the-sharepoint-rclone-remote"))?;
    let credentials = remote_credentials(&output.stdout.text, remote_name)?;
    let url = format!(
        "https://graph.microsoft.com/v1.0/drives/{}",
        credentials.drive_id
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| crate::fl!("ui-teams-could-not-prepare-microsoft-library-verification"))?;
    let response = client
        .get(url)
        .bearer_auth(&credentials.access_token)
        .send()
        .await
        .map_err(|_| crate::fl!("ui-teams-could-not-contact-microsoft-to-verify-the"))?;
    if !response.status().is_success() {
        return Err(graph_identity_error(response.status()));
    }
    let graph_response: Value = response
        .json()
        .await
        .map_err(|_| crate::fl!("ui-teams-microsoft-returned-invalid-library-metadata"))?;
    let identity = verify_graph_drive(&graph_response, expected, &credentials.drive_id)?;
    let account = match client
        .get("https://graph.microsoft.com/v1.0/me?$select=userPrincipalName,mail")
        .bearer_auth(&credentials.access_token)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            response.json::<Value>().await.ok().and_then(|value| {
                value
                    .get("userPrincipalName")
                    .or_else(|| value.get("mail"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
        }
        _ => None,
    };
    Ok(VerifiedTeamsLibrary { identity, account })
}

fn graph_identity_error(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 => crate::fl!("ui-teams-microsoft-authorization-expired-reconnect-this-rclone-remote"),
        403 => {
            crate::fl!("ui-teams-microsoft-denied-library-metadata-access-check-account")
        }
        404 => crate::fl!("ui-teams-microsoft-could-not-find-this-library-drive"),
        429 | 503 => {
            crate::fl!("ui-teams-microsoft-limited-library-verification-wait-before-retrying")
        }
        code => crate::fl!(
            "ui-teams-microsoft-library-verification-failed-http-code",
            code = (code).to_string()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{OfflineMirrorConfig, OnlineMountConfig, TuningProfile};
    use crate::process::{CapturedOutput, FakeCommandRunner};

    fn failed_rclone(detail: &str) -> CommandError {
        CommandError::NonZero {
            command: "rclone lsf Teams:".into(),
            code: Some(1),
            stdout: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: detail.into(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
        }
    }

    fn saved_connection() -> Connection {
        Connection {
            id: ConnectionId::from_uuid(uuid::Uuid::nil()),
            name: "Assessment".into(),
            provider: Provider::Teams,
            mode: ConnectionMode::OnlineMount(OnlineMountConfig::default()),
            remote_reference: "assessment".into(),
            remote_subpath: Some("Reports".into()),
            local_path: PathBuf::from("/home/example/Cloud/Assessment"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            teams_identity: Some(TeamsLibraryIdentity {
                site_url: "https://example.sharepoint.com/sites/assessment".into(),
                library_url: "https://example.sharepoint.com/sites/assessment/Documents".into(),
                drive_id: "b!assessment".into(),
            }),
        }
    }

    #[test]
    fn service_guard_rejects_changed_or_disabled_mount_before_remote_access() {
        let connection = saved_connection();
        let binding = ServiceGuardBinding::from_connection(&connection).unwrap();
        let parsed = ServiceGuardBinding::parse_arguments(&binding.arguments()[1..]).unwrap();
        assert_eq!(parsed, binding);
        assert!(parsed.matches_saved_connection(&connection).is_ok());

        let mut changed = connection.clone();
        changed.enabled = false;
        assert!(parsed.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.remote_reference = "other".into();
        assert!(parsed.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.remote_subpath = Some("Elsewhere".into());
        assert!(parsed.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.local_path = PathBuf::from("/home/example/Cloud/Other");
        assert!(parsed.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.teams_identity.as_mut().unwrap().drive_id = "b!other".into();
        assert!(parsed.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.teams_identity.as_mut().unwrap().library_url =
            "https://example.sharepoint.com/sites/other/Documents".into();
        assert!(parsed.matches_saved_connection(&changed).is_err());
    }

    #[test]
    fn mirror_guard_binds_saved_library_folder_and_local_recovery() {
        let mut connection = saved_connection();
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: PathBuf::from("/home/example/Recovery/Assessment"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
        let encoded = serde_json::to_string(&binding).unwrap();
        let decoded: MirrorGuardBinding = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, binding);
        assert!(decoded.matches_saved_connection(&connection).is_ok());

        let mut changed = connection.clone();
        changed.remote_subpath = Some("Other".into());
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.teams_identity.as_mut().unwrap().drive_id = "b!other".into();
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.teams_identity.as_mut().unwrap().library_url =
            "https://example.sharepoint.com/sites/assessment/Other".into();
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.teams_identity.as_mut().unwrap().site_url =
            "https://example.sharepoint.com/sites/other".into();
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.local_path = PathBuf::from("/home/example/Cloud/Other");
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        if let ConnectionMode::OfflineMirror(options) = &mut changed.mode {
            options.recovery_directory = PathBuf::from("/home/example/Recovery/Other");
        }
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.enabled = false;
        assert!(decoded.matches_saved_connection(&changed).is_err());
        changed = connection.clone();
        changed.remote_subpath = Some(".".into());
        assert!(MirrorGuardBinding::from_connection(&changed).is_err());
        changed.remote_subpath = None;
        assert!(MirrorGuardBinding::from_connection(&changed).is_err());
    }

    #[tokio::test]
    async fn changed_mirror_target_is_blocked_before_remote_access() {
        let mut connection = saved_connection();
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: PathBuf::from("/home/example/Recovery/Assessment"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
        connection.remote_subpath = Some("Other".into());
        let runner = FakeCommandRunner::default();
        assert!(
            verify_mirror_binding(&binding, &connection, &runner)
                .await
                .is_err()
        );
        assert!(runner.requests().is_empty());
    }

    #[test]
    fn parses_example_library_and_rejects_ambiguous_paths() {
        let parsed = parse_library_url(
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents",
        )
        .unwrap();
        assert_eq!(
            parsed.site_url,
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment"
        );
        assert_eq!(
            parsed.library_url,
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents"
        );
        assert!(
            parse_library_url("https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment")
                .is_err()
        );
        assert!(
            parse_library_url(
                "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%2FDocuments"
            )
            .is_err()
        );
        assert!(
            parse_library_url(
                "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%ZZDocuments"
            )
            .is_err()
        );
        assert!(
            parse_library_url(
                "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment//Shared%20Documents"
            )
            .is_err()
        );
        assert!(parse_library_url("https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents/Forms/AllItems.aspx").is_err());
        assert!(same_library_url(
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents?web=1#view",
            &parsed.library_url
        ));
        assert!(same_library_url(
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents/",
            &parsed.library_url
        ));
    }

    #[test]
    fn verifies_drive_id_type_and_url() {
        let expected = parse_library_url(
            "https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents",
        )
        .unwrap();
        let drive = serde_json::json!({"id":"b!example", "driveType":"documentLibrary", "webUrl":"https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents"});
        assert_eq!(
            verify_graph_drive(&drive, &expected, "b!example")
                .unwrap()
                .drive_id,
            "b!example"
        );
        assert!(verify_graph_drive(&drive, &expected, "b!other").is_err());
        let other = serde_json::json!({"id":"b!example", "driveType":"documentLibrary", "webUrl":"https://emailarizona.sharepoint.com/sites/OTHER/Shared%20Documents"});
        assert!(verify_graph_drive(&other, &expected, "b!example").is_err());
        let mut saved = expected.clone();
        saved.drive_id = "b!previous".into();
        assert!(
            verify_graph_drive(&drive, &saved, "b!example")
                .unwrap_err()
                .contains("drive ID changed")
        );
        let personal = serde_json::json!({"id":"b!example", "driveType":"business", "webUrl":expected.library_url});
        assert!(verify_graph_drive(&personal, &expected, "b!example").is_err());
    }

    #[test]
    fn teams_read_errors_identify_cause_without_echoing_remote_output() {
        for (detail, folder, expected) in [
            (
                "AADSTS65001 consent_required secret=abc",
                false,
                "tenant consent",
            ),
            ("invalid_grant token=abc", false, "authorization expired"),
            (
                "403 accessDenied /private/file token=abc",
                false,
                "cannot read",
            ),
            (
                "429 Too Many Requests token=abc",
                false,
                "limiting requests",
            ),
            ("503 Server Too Busy token=abc", false, "limiting requests"),
            (
                "404 itemNotFound /private/file token=abc",
                true,
                "selected SharePoint folder",
            ),
            (
                "404 itemNotFound token=abc",
                false,
                "SharePoint library is unavailable",
            ),
            (
                "dial tcp: network timeout token=abc",
                false,
                "Could not reach SharePoint",
            ),
            ("unexpected secret=abc", false, "inspect the rclone log"),
        ] {
            let message = rclone_read_error(&failed_rclone(detail), folder);
            assert!(message.contains(expected), "{message}");
            assert!(!message.contains("abc"), "{message}");
            assert!(!message.contains("/private/file"), "{message}");
        }
    }

    #[test]
    fn teams_graph_status_and_remote_config_failures_are_specific() {
        for (status, expected) in [
            (401, "authorization expired"),
            (403, "permission or tenant consent"),
            (404, "could not find this library drive"),
            (429, "wait before retrying"),
            (503, "wait before retrying"),
        ] {
            let message = graph_identity_error(reqwest::StatusCode::from_u16(status).unwrap());
            assert!(message.contains(expected), "{message}");
        }
        let wrong_type = r#"{"assessment":{"type":"onedrive","drive_type":"business","drive_id":"b!x","token":{"access_token":"secret"}}}"#;
        assert!(
            remote_credentials(wrong_type, "assessment")
                .err()
                .expect("wrong type")
                .contains("document library")
        );
        let no_token =
            r#"{"assessment":{"type":"onedrive","drive_type":"documentLibrary","drive_id":"b!x"}}"#;
        assert!(
            remote_credentials(no_token, "assessment")
                .err()
                .expect("missing token")
                .contains("not authenticated")
        );
    }

    #[tokio::test]
    async fn failed_teams_root_listing_reports_specific_safe_error() {
        let runner = FakeCommandRunner::default();
        runner.push(Err(failed_rclone("AADSTS65001 consent_required token=abc")));
        let expected =
            parse_library_url("https://example.sharepoint.com/sites/assessment/Documents").unwrap();
        let error = verify_remote(&runner, "assessment", &expected)
            .await
            .expect_err("consent failure");
        assert!(error.contains("tenant consent"));
        assert!(!error.contains("abc"));
        assert_eq!(runner.requests().len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires an authenticated disposable SharePoint library remote"]
    async fn live_remote_identity() {
        let remote = std::env::var("TEAMS_TEST_REMOTE").expect("TEAMS_TEST_REMOTE");
        let url = std::env::var("TEAMS_TEST_LIBRARY_URL").expect("TEAMS_TEST_LIBRARY_URL");
        let mut expected = parse_library_url(&url).expect("library URL");
        if let Ok(drive_id) = std::env::var("TEAMS_TEST_DRIVE_ID") {
            expected.drive_id = drive_id;
        }
        let verified = verify_remote(&crate::process::SystemCommandRunner, &remote, &expected)
            .await
            .expect("matching authenticated library");
        assert!(!verified.identity.drive_id.is_empty());
        assert_eq!(verified.identity.library_url, expected.library_url);
        if !expected.drive_id.is_empty() {
            assert_eq!(verified.identity.drive_id, expected.drive_id);
        }
    }

    #[tokio::test]
    #[ignore = "requires an authenticated disposable SharePoint library folder"]
    async fn live_mirror_guard_reads_only_selected_folder() {
        let remote = std::env::var("TEAMS_TEST_REMOTE").expect("TEAMS_TEST_REMOTE");
        let url = std::env::var("TEAMS_TEST_LIBRARY_URL").expect("TEAMS_TEST_LIBRARY_URL");
        let drive_id = std::env::var("TEAMS_TEST_DRIVE_ID").expect("TEAMS_TEST_DRIVE_ID");
        let folder = std::env::var("TEAMS_TEST_FOLDER").expect("TEAMS_TEST_FOLDER");
        let mut connection = saved_connection();
        connection.mode = ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: PathBuf::from("/tmp/cosmic-teams-mirror-guard-recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        });
        connection.remote_reference = remote;
        connection.remote_subpath = Some(folder);
        connection.local_path = PathBuf::from("/tmp/cosmic-teams-mirror-guard-local");
        let mut identity = parse_library_url(&url).expect("library URL");
        identity.drive_id = drive_id;
        connection.teams_identity = Some(identity);
        let binding = MirrorGuardBinding::from_connection(&connection).expect("scoped binding");
        verify_mirror_binding(&binding, &connection, &crate::process::SystemCommandRunner)
            .await
            .expect("unchanged disposable SharePoint folder must pass the read-only guard");

        let mut changed_drive = connection.clone();
        changed_drive.teams_identity.as_mut().unwrap().drive_id = "b!wrong-drive".into();
        let changed_binding =
            MirrorGuardBinding::from_connection(&changed_drive).expect("changed binding");
        assert!(
            verify_mirror_binding(
                &changed_binding,
                &changed_drive,
                &crate::process::SystemCommandRunner
            )
            .await
            .is_err()
        );

        let mut missing_folder = connection.clone();
        missing_folder.remote_subpath =
            Some("CloudMounter-Definitely-Missing-Guard-20261003".into());
        let missing_binding =
            MirrorGuardBinding::from_connection(&missing_folder).expect("missing folder binding");
        assert!(
            verify_mirror_binding(
                &missing_binding,
                &missing_folder,
                &crate::process::SystemCommandRunner
            )
            .await
            .is_err()
        );
    }
}
