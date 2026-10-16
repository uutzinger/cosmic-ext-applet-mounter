// SPDX-License-Identifier: MIT
//! Production OneDrive primary-drive identity, remote setup and mirror guards.
//! Errors and results are machine protocol codes; provider data/tokens are not diagnostics.
use crate::process::{CommandError, CommandOutput, CommandRequest, CommandRunner, Executable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{future::Future, pin::Pin, time::Duration};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OneDriveError {
    InvalidInput,
    CommandFailed,
    RcloneFailed {
        operation: RcloneOperation,
        failure: RcloneFailure,
    },
    IncompleteOutput,
    InvalidMetadata,
    InvalidConfigResponse,
    WrongAccount,
    WrongDrive,
    UnsupportedItem,
    CustomClient,
    ExistingRemote,
    CleanupRequired,
    UnsupportedQuestion,
    Cancelled,
    GraphFailed,
    GraphHttp(u16),
    AccountProfileUnauthorized(u16),
    Io,
    Ownership,
    InvalidState,
    VerificationFailed,
}
pub type OneDriveResult<T> = Result<T, OneDriveError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcloneOperation {
    IdentityAccess,
    Config,
    OwnerRead,
    AnchorRead,
    DownloadVerification,
    Other,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcloneFailure {
    MissingExecutable,
    InvalidArgument,
    Spawn,
    Timeout,
    NonZero(Option<i32>),
}
fn rclone_error(operation: RcloneOperation, error: CommandError) -> OneDriveError {
    let failure = match error {
        CommandError::Cancelled { .. } => return OneDriveError::Cancelled,
        CommandError::MissingExecutable(_) => RcloneFailure::MissingExecutable,
        CommandError::InvalidArgument => RcloneFailure::InvalidArgument,
        CommandError::Spawn { .. } => RcloneFailure::Spawn,
        CommandError::Timeout { .. } => RcloneFailure::Timeout,
        CommandError::NonZero { code, .. } => RcloneFailure::NonZero(code),
    };
    OneDriveError::RcloneFailed { operation, failure }
}
fn command_operation(args: &[&str]) -> RcloneOperation {
    match args.first().copied() {
        Some("lsjson") => RcloneOperation::IdentityAccess,
        Some("config") => RcloneOperation::Config,
        Some("cat")
            if args
                .get(1)
                .is_some_and(|p| p.ends_with("/.prototype-owner")) =>
        {
            RcloneOperation::OwnerRead
        }
        Some("cat")
            if args
                .get(1)
                .is_some_and(|p| p.contains("/.cloud-mounter-onedrive-anchor-")) =>
        {
            RcloneOperation::AnchorRead
        }
        Some("check") => RcloneOperation::DownloadVerification,
        _ => RcloneOperation::Other,
    }
}
pub use crate::model::OneDriveDriveType as DriveType;
impl DriveType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Business => "business",
        }
    }
    pub fn parse(s: &str) -> OneDriveResult<Self> {
        match s {
            "personal" => Ok(Self::Personal),
            "business" => Ok(Self::Business),
            _ => Err(OneDriveError::InvalidInput),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub drive_fingerprint: String,
    pub account_fingerprint: String,
    pub drive_type: DriveType,
}
pub fn fingerprint(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
pub fn validate_remote(name: &str) -> OneDriveResult<()> {
    if name.is_empty()
        || name.len() > 96
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Err(OneDriveError::InvalidInput);
    }
    Ok(())
}
pub fn request(args: &[&str], timeout: u64) -> OneDriveResult<CommandRequest> {
    args.iter()
        .try_fold(CommandRequest::new(Executable::Rclone), |r, a| r.arg(*a))
        .map(|r| {
            r.with_timeout(Duration::from_secs(timeout))
                .with_output_limit(1024 * 1024)
        })
        .map_err(|_| OneDriveError::InvalidInput)
}
pub fn complete(output: &CommandOutput) -> OneDriveResult<&str> {
    if output.stdout.truncated || output.stdout.invalid_utf8 {
        return Err(OneDriveError::IncompleteOutput);
    }
    Ok(&output.stdout.text)
}
pub async fn command<R: CommandRunner>(
    runner: &R,
    args: &[&str],
    timeout: u64,
    cancel: CancellationToken,
) -> OneDriveResult<String> {
    if cancel.is_cancelled() {
        return Err(OneDriveError::Cancelled);
    }
    let out = runner
        .run(request(args, timeout)?, cancel.clone())
        .await
        .map_err(|error| {
            if cancel.is_cancelled() {
                OneDriveError::Cancelled
            } else {
                rclone_error(command_operation(args), error)
            }
        })?;
    if cancel.is_cancelled() {
        return Err(OneDriveError::Cancelled);
    }
    Ok(complete(&out)?.to_owned())
}
pub type GraphFuture<'a> = Pin<Box<dyn Future<Output = OneDriveResult<Value>> + Send + 'a>>;
pub enum GraphTarget<'a> {
    Profile,
    PrimaryDrive,
    ExpectedBusinessDrive(&'a str),
    Folder { drive_id: &'a str, path: &'a str },
}
fn business_upn(account: &str) -> OneDriveResult<&str> {
    let account = account.trim();
    let (name, domain) = account.split_once('@').ok_or(OneDriveError::InvalidInput)?;
    if account.len() > 320
        || name.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || account.chars().any(|c| c.is_control() || c.is_whitespace())
        || account.contains(['/', '\\', '?'])
    {
        return Err(OneDriveError::InvalidInput);
    }
    Ok(account)
}
pub(crate) fn graph_url(target: GraphTarget<'_>) -> OneDriveResult<url::Url> {
    let mut url =
        url::Url::parse("https://graph.microsoft.com/v1.0/").expect("fixed Microsoft Graph URL");
    match target {
        GraphTarget::Profile => {
            url.set_path("/v1.0/me");
            url.set_query(Some("$select=id,userPrincipalName,mail"));
        }
        GraphTarget::PrimaryDrive => {
            url.set_path("/v1.0/me/drive");
            url.set_query(Some("$select=id,driveType,owner"));
        }
        GraphTarget::Folder { drive_id, path } => {
            validate_subpath(path)?;
            let identity = crate::model::OneDriveIdentity {
                drive_id: drive_id.into(),
                drive_type: DriveType::Personal,
            };
            if !identity.is_valid() {
                return Err(OneDriveError::InvalidInput);
            }
            url.path_segments_mut()
                .expect("fixed base URL")
                .pop_if_empty()
                .push("drives")
                .push(drive_id)
                .push("root:");
            for part in path.split('/') {
                url.path_segments_mut().expect("fixed path").push(part);
            }
            url.set_query(Some("$select=id,folder,remoteItem,package,parentReference"));
        }
        GraphTarget::ExpectedBusinessDrive(account) => {
            url.path_segments_mut()
                .expect("fixed base URL")
                .pop_if_empty()
                .push("users")
                .push(business_upn(account)?)
                .push("drive");
            url.set_query(Some("$select=id,driveType,owner"));
        }
    }
    Ok(url)
}
pub trait Graph: Send + Sync {
    fn get<'a>(
        &'a self,
        target: GraphTarget<'a>,
        token: &'a str,
        cancel: CancellationToken,
    ) -> GraphFuture<'a>;
}
pub struct MicrosoftGraph(reqwest::Client);
impl MicrosoftGraph {
    pub fn new() -> OneDriveResult<Self> {
        Ok(Self(
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(25))
                .build()
                .map_err(|_| OneDriveError::GraphFailed)?,
        ))
    }
}
impl Graph for MicrosoftGraph {
    fn get<'a>(
        &'a self,
        target: GraphTarget<'a>,
        token: &'a str,
        cancel: CancellationToken,
    ) -> GraphFuture<'a> {
        Box::pin(async move {
            let url = graph_url(target)?;
            tokio::select! { _=cancel.cancelled()=>Err(OneDriveError::Cancelled), result=async {
                let mut response=self.0.get(url).bearer_auth(token).send().await.map_err(|_|OneDriveError::GraphFailed)?;
                if !response.status().is_success() {return Err(OneDriveError::GraphHttp(response.status().as_u16()));}
                let mut bytes=Vec::new();
                while let Some(chunk)=response.chunk().await.map_err(|_|OneDriveError::GraphFailed)? {
                    if bytes.len()+chunk.len()>64*1024 { return Err(OneDriveError::IncompleteOutput); }
                    bytes.extend_from_slice(&chunk);
                }
                serde_json::from_slice(&bytes).map_err(|_|OneDriveError::InvalidMetadata)
            }=>result }
        })
    }
}
pub(crate) async fn primary<G: Graph>(
    graph: &G,
    token: &str,
    drive_type: DriveType,
    account: &str,
    cancel: CancellationToken,
) -> OneDriveResult<(String, Identity)> {
    if account.trim().is_empty() || account.contains(['\n', '\r', '\0']) {
        return Err(OneDriveError::InvalidInput);
    }
    if drive_type == DriveType::Business {
        let account = business_upn(account)?;
        let signed_in = graph
            .get(GraphTarget::PrimaryDrive, token, cancel.child_token())
            .await?;
        let (drive, owner) = business_drive(&signed_in)?;
        let expected = graph
            .get(GraphTarget::ExpectedBusinessDrive(account), token, cancel)
            .await?;
        let (expected_drive, expected_owner) = business_drive(&expected)?;
        if drive != expected_drive || owner != expected_owner {
            return Err(OneDriveError::WrongAccount);
        }
        return Ok((
            drive.to_owned(),
            Identity {
                drive_fingerprint: fingerprint(drive),
                account_fingerprint: fingerprint(owner),
                drive_type,
            },
        ));
    }
    let me = graph
        .get(GraphTarget::Profile, token, cancel.child_token())
        .await
        .map_err(|error| match error {
            OneDriveError::GraphHttp(status @ (401 | 403)) => {
                OneDriveError::AccountProfileUnauthorized(status)
            }
            other => other,
        })?;
    let id = me
        .get("id")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or(OneDriveError::InvalidMetadata)?;
    if !["userPrincipalName", "mail"].iter().any(|k| {
        me.get(k)
            .and_then(Value::as_str)
            .is_some_and(|v| v.eq_ignore_ascii_case(account.trim()))
    }) {
        return Err(OneDriveError::WrongAccount);
    }
    let drive = graph.get(GraphTarget::PrimaryDrive, token, cancel).await?;
    let drive_id = drive
        .get("id")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or(OneDriveError::InvalidMetadata)?;
    if drive.get("driveType").and_then(Value::as_str) != Some(drive_type.as_str()) {
        return Err(OneDriveError::WrongDrive);
    }
    Ok((
        drive_id.to_owned(),
        Identity {
            drive_fingerprint: fingerprint(drive_id),
            account_fingerprint: fingerprint(id),
            drive_type,
        },
    ))
}
fn business_drive(value: &Value) -> OneDriveResult<(&str, &str)> {
    if value.get("driveType").and_then(Value::as_str) != Some("business") {
        return Err(OneDriveError::WrongDrive);
    }
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or(OneDriveError::InvalidMetadata)?;
    let owner = value
        .pointer("/owner/user/id")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or(OneDriveError::InvalidMetadata)?;
    Ok((id, owner))
}
pub async fn verify<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    remote: &str,
    drive_type: DriveType,
    account: &str,
    cancel: CancellationToken,
) -> OneDriveResult<VerifiedRemote> {
    business_upn(account)?;
    let (configured, token) = credentials(runner, remote, drive_type, cancel.child_token()).await?;
    let (id, identity) = primary(graph, &token, drive_type, account, cancel).await?;
    if configured.drive_id != id {
        return Err(OneDriveError::WrongDrive);
    }
    let drive = crate::model::OneDriveIdentity {
        drive_id: id,
        drive_type,
    };
    if !drive.is_valid() {
        return Err(OneDriveError::InvalidMetadata);
    }
    Ok(VerifiedRemote {
        identity: drive,
        fingerprints: identity,
    })
}
async fn credentials<R: CommandRunner>(
    runner: &R,
    remote: &str,
    drive_type: DriveType,
    cancel: CancellationToken,
) -> OneDriveResult<(crate::model::OneDriveIdentity, String)> {
    validate_remote(remote)?;
    // Refresh through rclone without enumerating names or downloading user files.
    command(
        runner,
        &["lsjson", &format!("{remote}:"), "--stat"],
        30,
        cancel.child_token(),
    )
    .await?;
    let json = dump(runner, cancel.child_token()).await?;
    let dump = json.to_string();
    let r = json.get(remote).ok_or(OneDriveError::InvalidMetadata)?;
    if r.get("type").and_then(Value::as_str) != Some("onedrive")
        || r.get("drive_type").and_then(Value::as_str) != Some(drive_type.as_str())
    {
        return Err(OneDriveError::WrongDrive);
    }
    if [
        "client_id",
        "client_secret",
        "tenant",
        "tenant_url",
        "root_folder_id",
        "auth_url",
        "token_url",
    ]
    .iter()
    .any(|k| r.get(k).is_some_and(|v| v.as_str() != Some("")))
    {
        return Err(OneDriveError::CustomClient);
    }
    // Decode credentials privately; never echo configuration.
    let token = crate::microsoft_oauth::oauth_access_token(&dump, remote)
        .map_err(|_| OneDriveError::InvalidMetadata)?;
    if ["client_credentials", "expose_onenote_files"]
        .iter()
        .any(|key| {
            r.get(key).is_some_and(|value| {
                !matches!(value, Value::Null) && value != false && value != "" && value != "false"
            })
        })
    {
        return Err(OneDriveError::CustomClient);
    }
    let identity = crate::model::OneDriveIdentity {
        drive_id: r
            .get("drive_id")
            .and_then(Value::as_str)
            .ok_or(OneDriveError::InvalidMetadata)?
            .into(),
        drive_type,
    };
    if !identity.is_valid() {
        return Err(OneDriveError::InvalidMetadata);
    }
    Ok((identity, token))
}
fn microsoft_oauth_error(error: crate::microsoft_oauth::OAuthError) -> OneDriveError {
    use crate::microsoft_oauth::{CommandFailure, OAuthError};
    match error {
        OAuthError::ExistingRemote => OneDriveError::ExistingRemote,
        OAuthError::InvalidRemoteName => OneDriveError::InvalidInput,
        OAuthError::InvalidResponse | OAuthError::UnstructuredResponse => {
            OneDriveError::InvalidConfigResponse
        }
        OAuthError::TooManySteps => OneDriveError::UnsupportedQuestion,
        OAuthError::Command(CommandFailure::Cancelled) => OneDriveError::Cancelled,
        OAuthError::Command(failure) => OneDriveError::RcloneFailed {
            operation: RcloneOperation::Config,
            failure: match failure {
                CommandFailure::MissingExecutable => RcloneFailure::MissingExecutable,
                CommandFailure::InvalidArgument => RcloneFailure::InvalidArgument,
                CommandFailure::Spawn => RcloneFailure::Spawn,
                CommandFailure::Timeout => RcloneFailure::Timeout,
                CommandFailure::NonZero(code) => RcloneFailure::NonZero(code),
                CommandFailure::Cancelled => unreachable!(),
            },
        },
        OAuthError::CleanupRequired => OneDriveError::CleanupRequired,
        _ => OneDriveError::CommandFailed,
    }
}
async fn dump<R: CommandRunner>(runner: &R, cancel: CancellationToken) -> OneDriveResult<Value> {
    crate::microsoft_oauth::config_dump(runner, cancel)
        .await
        .map_err(microsoft_oauth_error)
}
pub async fn authenticate<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    setup: &RemoteSetup,
    cancel: CancellationToken,
) -> OneDriveResult<VerifiedRemote> {
    let remote = &setup.remote_name;
    let drive_type = setup.drive_type;
    let account = &setup.expected_account;
    setup.validate()?;
    if drive_type == DriveType::Business {
        business_upn(account)?;
    }
    use crate::microsoft_oauth::{NewRemote, Target};
    let target = match drive_type {
        DriveType::Personal => Target::from(crate::model::OneDriveDriveType::Personal),
        DriveType::Business => Target::from(crate::model::OneDriveDriveType::Business),
    };
    let mut new_remote = NewRemote::prepare(runner, remote, target, cancel.child_token())
        .await
        .map_err(microsoft_oauth_error)?;
    let result = async {
        let mut response = new_remote
            .create(runner, cancel.child_token())
            .await
            .map_err(microsoft_oauth_error)?;
        let mut selected = false;
        for _ in 0..12 {
            if response
                .get("Error")
                .and_then(Value::as_str)
                .is_some_and(|v| !v.is_empty())
            {
                return Err(OneDriveError::CommandFailed);
            }
            let state = response
                .get("State")
                .and_then(Value::as_str)
                .ok_or(OneDriveError::InvalidMetadata)?;
            if state.is_empty() {
                let verified = verify(
                    runner,
                    graph,
                    remote,
                    drive_type,
                    account,
                    cancel.child_token(),
                )
                .await?;
                setup.accept(&verified)?;
                verify_setup_subtree(
                    runner,
                    graph,
                    setup,
                    &verified.identity,
                    cancel.child_token(),
                )
                .await?;
                return Ok(verified);
            }
            let option = &response["Option"];
            let answer = match option["Name"].as_str() {
                Some("config_is_local") => "true".to_owned(),
                Some("config_type") => "onedrive".to_owned(),
                Some("config_driveid") => {
                    let text = dump(runner, cancel.child_token()).await?.to_string();
                    let token = crate::microsoft_oauth::oauth_access_token(&text, remote)
                        .map_err(|_| OneDriveError::InvalidMetadata)?;
                    let (id, _) =
                        primary(graph, &token, drive_type, account, cancel.child_token()).await?;
                    let examples = option["Examples"]
                        .as_array()
                        .ok_or(OneDriveError::InvalidMetadata)?;
                    if examples.len() > 40
                        || !examples
                            .iter()
                            .any(|v| v["Value"].as_str() == Some(id.as_str()))
                    {
                        return Err(OneDriveError::WrongDrive);
                    }
                    selected = true;
                    id
                }
                Some("config_drive_ok") if selected => "true".to_owned(),
                _ => return Err(OneDriveError::UnsupportedQuestion),
            };
            response = new_remote
                .continue_with(runner, state, &answer, cancel.child_token())
                .await
                .map_err(microsoft_oauth_error)?;
        }
        Err(OneDriveError::UnsupportedQuestion)
    }
    .await;
    if result.is_err() {
        new_remote
            .rollback(runner)
            .await
            .map_err(microsoft_oauth_error)?;
    }
    result
}

/// Setup result contains stable drive metadata, never tokens or raw Graph bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRemote {
    pub identity: crate::model::OneDriveIdentity,
    pub fingerprints: Identity,
}

/// Entered account is used only during explicit setup/test, never saved as a token.
#[derive(Clone)]
pub struct RemoteSetup {
    pub remote_name: String,
    pub drive_type: DriveType,
    pub expected_account: String,
    pub remote_subpath: Option<String>,
    /// A replacement remote must verify this saved identity before it is accepted.
    pub previous_identity: Option<crate::model::OneDriveIdentity>,
}

impl std::fmt::Debug for RemoteSetup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RemoteSetup([REDACTED])")
    }
}

impl RemoteSetup {
    pub fn new(remote: &str, drive_type: DriveType, account: &str) -> OneDriveResult<Self> {
        let setup = Self {
            remote_name: remote.trim().into(),
            drive_type,
            expected_account: account.trim().into(),
            remote_subpath: None,
            previous_identity: None,
        };
        setup.validate()?;
        Ok(setup)
    }
    fn validate(&self) -> OneDriveResult<()> {
        validate_remote(&self.remote_name)?;
        business_upn(&self.expected_account)?;
        if let Some(path) = &self.remote_subpath {
            validate_subpath(path)?;
        }
        if self
            .previous_identity
            .as_ref()
            .is_some_and(|identity| !identity.is_valid() || identity.drive_type != self.drive_type)
        {
            return Err(OneDriveError::WrongDrive);
        }
        Ok(())
    }
    fn accept(&self, verified: &VerifiedRemote) -> OneDriveResult<()> {
        if self
            .previous_identity
            .as_ref()
            .is_some_and(|saved| saved != &verified.identity)
        {
            return Err(OneDriveError::WrongDrive);
        }
        Ok(())
    }
}

/// Inspect/select an existing remote without changing its configuration.
pub async fn verify_existing<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    setup: &RemoteSetup,
    cancel: CancellationToken,
) -> OneDriveResult<VerifiedRemote> {
    setup.validate()?;
    let verified = verify(
        runner,
        graph,
        &setup.remote_name,
        setup.drive_type,
        &setup.expected_account,
        cancel.child_token(),
    )
    .await?;
    setup.accept(&verified)?;
    verify_setup_subtree(runner, graph, setup, &verified.identity, cancel).await?;
    Ok(verified)
}

fn validate_subpath(path: &str) -> OneDriveResult<()> {
    crate::sync::validate_remote_subpath(path).map_err(|_| OneDriveError::InvalidInput)?;
    if path
        .split('/')
        .any(|part| part.is_empty() || matches!(part, "." | ".."))
        || path.len() > 2048
    {
        return Err(OneDriveError::InvalidInput);
    }
    Ok(())
}

async fn verify_setup_subtree<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    setup: &RemoteSetup,
    identity: &crate::model::OneDriveIdentity,
    cancel: CancellationToken,
) -> OneDriveResult<()> {
    if let Some(path) = &setup.remote_subpath {
        let dump = dump(runner, cancel.child_token()).await?;
        let token =
            crate::microsoft_oauth::oauth_access_token(&dump.to_string(), &setup.remote_name)
                .map_err(microsoft_oauth_error)?;
        verify_subtree(
            runner,
            graph,
            &setup.remote_name,
            path,
            identity,
            &token,
            cancel,
        )
        .await?;
    }
    Ok(())
}

async fn verify_subtree<R: CommandRunner, G: Graph + ?Sized>(
    runner: &R,
    graph: &G,
    remote: &str,
    subpath: &str,
    identity: &crate::model::OneDriveIdentity,
    token: &str,
    cancel: CancellationToken,
) -> OneDriveResult<()> {
    validate_remote(remote)?;
    validate_subpath(subpath)?;
    let folder = graph
        .get(
            GraphTarget::Folder {
                drive_id: &identity.drive_id,
                path: subpath,
            },
            token,
            cancel.child_token(),
        )
        .await?;
    if ["remoteItem", "package"]
        .iter()
        .any(|key| folder.get(key).is_some_and(|value| !value.is_null()))
    {
        return Err(OneDriveError::UnsupportedItem);
    }
    if folder.get("folder").and_then(Value::as_object).is_none()
        || folder
            .get("id")
            .and_then(Value::as_str)
            .is_none_or(|id| id.is_empty())
        || folder
            .pointer("/parentReference/driveId")
            .and_then(Value::as_str)
            != Some(identity.drive_id.as_str())
    {
        return Err(OneDriveError::InvalidMetadata);
    }
    let text = command(
        runner,
        &["lsjson", &format!("{remote}:{subpath}"), "--stat"],
        30,
        cancel,
    )
    .await?;
    let value: Value = serde_json::from_str(&text).map_err(|_| OneDriveError::InvalidMetadata)?;
    if value.get("IsDir").and_then(Value::as_bool) != Some(true) {
        return Err(OneDriveError::InvalidMetadata);
    }
    Ok(())
}

pub const MIRROR_GUARD_FLAG: &str = "--verify-onedrive-mirror";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MirrorGuardBinding {
    pub connection_id: crate::model::ConnectionId,
    pub remote_name: String,
    pub remote_subpath: Option<String>,
    pub local_path: std::path::PathBuf,
    pub recovery_directory: std::path::PathBuf,
    pub identity: crate::model::OneDriveIdentity,
}

impl MirrorGuardBinding {
    pub fn from_connection(connection: &crate::model::Connection) -> OneDriveResult<Self> {
        use crate::model::{ConnectionMode, OfflineEngine, Provider};
        let ConnectionMode::OfflineMirror(options) = &connection.mode else {
            return Err(OneDriveError::InvalidState);
        };
        if connection.provider != Provider::OneDrive
            || connection.offline_engine() != Some(OfflineEngine::Rclone)
        {
            return Err(OneDriveError::InvalidState);
        }
        validate_remote(&connection.remote_reference)?;
        if let Some(path) = &connection.remote_subpath {
            validate_subpath(path)?;
        }
        let identity = connection
            .onedrive_identity
            .as_ref()
            .filter(|identity| identity.is_valid())
            .ok_or(OneDriveError::WrongDrive)?;
        Ok(Self {
            connection_id: connection.id,
            remote_name: connection.remote_reference.clone(),
            remote_subpath: connection.remote_subpath.clone(),
            local_path: connection.local_path.clone(),
            recovery_directory: options.recovery_directory.clone(),
            identity: identity.clone(),
        })
    }
    pub fn matches_saved_connection(
        &self,
        connection: &crate::model::Connection,
    ) -> OneDriveResult<()> {
        if !connection.enabled || Self::from_connection(connection)? != *self {
            return Err(OneDriveError::InvalidState);
        }
        Ok(())
    }
}

pub fn mirror_guard_command(
    connection: &crate::model::Connection,
) -> Result<crate::services::ServiceCommand, String> {
    let binding =
        MirrorGuardBinding::from_connection(connection).map_err(|error| error.to_string())?;
    crate::mount_guard::service_command(vec![
        MIRROR_GUARD_FLAG.into(),
        serde_json::to_string(&binding).map_err(|_| OneDriveError::InvalidState.to_string())?,
    ])
}

/// Recurring guard verifies the saved resource; no profile request or extra consent.
pub async fn verify_mirror_binding<R: CommandRunner, G: Graph + ?Sized>(
    binding: &MirrorGuardBinding,
    connection: &crate::model::Connection,
    runner: &R,
    graph: &G,
    cancel: CancellationToken,
) -> OneDriveResult<()> {
    binding.matches_saved_connection(connection)?;
    let (configured, token) = credentials(
        runner,
        &binding.remote_name,
        binding.identity.drive_type,
        cancel.child_token(),
    )
    .await?;
    if configured != binding.identity {
        return Err(OneDriveError::WrongDrive);
    }
    let primary = graph
        .get(GraphTarget::PrimaryDrive, &token, cancel.child_token())
        .await?;
    if primary.get("id").and_then(Value::as_str) != Some(binding.identity.drive_id.as_str())
        || primary.get("driveType").and_then(Value::as_str)
            != Some(binding.identity.drive_type.as_str())
    {
        return Err(OneDriveError::WrongDrive);
    }
    if let Some(path) = &binding.remote_subpath {
        verify_subtree(
            runner,
            graph,
            &binding.remote_name,
            path,
            &binding.identity,
            &token,
            cancel,
        )
        .await?;
    }
    Ok(())
}

pub async fn run_mirror_service_guard(arguments: &[String]) -> Result<(), String> {
    let [encoded] = arguments else {
        return Err(OneDriveError::InvalidInput.to_string());
    };
    let binding: MirrorGuardBinding =
        serde_json::from_str(encoded).map_err(|_| OneDriveError::InvalidInput.to_string())?;
    let report = crate::config::Config::load_runtime();
    if report.source != crate::config::LoadSource::Current {
        return Err(OneDriveError::InvalidState.to_string());
    }
    let connection = report
        .config
        .document
        .connections
        .iter()
        .find(|connection| connection.id == binding.connection_id)
        .ok_or_else(|| OneDriveError::InvalidState.to_string())?;
    verify_mirror_binding(
        &binding,
        connection,
        &crate::process::RuntimeCommandRunner::detect_current(),
        &MicrosoftGraph::new().map_err(|error| error.to_string())?,
        CancellationToken::new(),
    )
    .await
    .map_err(|error| error.to_string())
}

impl std::fmt::Display for OneDriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidInput => crate::fl!("onedrive-invalid-setup"),
            Self::UnsupportedItem => crate::fl!("onedrive-unsupported-item"),
            Self::WrongAccount => crate::fl!("onedrive-wrong-account"),
            Self::WrongDrive => crate::fl!("onedrive-wrong-drive"),
            Self::CustomClient => crate::fl!("onedrive-unsupported-config"),
            Self::ExistingRemote => crate::fl!("microsoft-oauth-remote-exists"),
            Self::CleanupRequired => crate::fl!("microsoft-oauth-cleanup-required"),
            Self::Cancelled => crate::fl!("microsoft-oauth-cancelled"),
            Self::AccountProfileUnauthorized(_) => crate::fl!("onedrive-profile-required"),
            Self::GraphHttp(code) => crate::fl!("onedrive-graph-http", code = code.to_string()),
            Self::GraphFailed => crate::fl!("onedrive-graph-failed"),
            Self::InvalidMetadata | Self::IncompleteOutput | Self::InvalidConfigResponse => {
                crate::fl!("onedrive-invalid-metadata")
            }
            Self::UnsupportedQuestion => crate::fl!("microsoft-oauth-invalid-response"),
            Self::InvalidState => crate::fl!("onedrive-binding-changed"),
            Self::Ownership => crate::fl!("onedrive-anchor-invalid"),
            Self::Io
            | Self::VerificationFailed
            | Self::CommandFailed
            | Self::RcloneFailed { .. } => crate::fl!("onedrive-command-failed"),
        };
        f.write_str(&message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onedrive_folder_url_encodes_each_segment_and_preserves_fixed_host() {
        let url = graph_url(GraphTarget::Folder {
            drive_id: "b!primary",
            path: "Test #1/space ?",
        })
        .unwrap();
        assert_eq!(url.host_str(), Some("graph.microsoft.com"));
        assert_eq!(
            url.path(),
            "/v1.0/drives/b!primary/root:/Test%20%231/space%20%3F"
        );
        assert_eq!(
            url.query(),
            Some("$select=id,folder,remoteItem,package,parentReference")
        );
        for path in ["../other", "/root", "A/./B", "A//B", "A/../B"] {
            assert!(
                graph_url(GraphTarget::Folder {
                    drive_id: "b!primary",
                    path
                })
                .is_err()
            );
        }
    }
}
