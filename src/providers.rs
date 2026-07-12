// SPDX-License-Identifier: MIT

//! Adapters for rclone and onedriver online mount engines.

use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::model::{Connection, ConnectionId, ConnectionMode, OnlineMountConfig, Provider};
use crate::process::{CommandError, CommandRequest, CommandRunner, Executable, RetryPolicy};
use crate::services::{ActiveState, ServiceSpec, UnitStatus};

const DEFAULT_RCLONE: &str = "/usr/bin/rclone";
const DEFAULT_ONEDRIVER: &str = "/usr/bin/onedriver";

pub type ProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    UnsupportedProvider(Provider),
    InvalidMode,
    InvalidRemoteReference,
    InvalidRemoteSubpath,
    MissingExecutable(Executable),
    Command(CommandError),
    InvalidResponse(String),
    Unauthenticated,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProvider(provider) => {
                write!(formatter, "unsupported provider {provider:?}")
            }
            Self::InvalidMode => write!(formatter, "connection is not an online mount"),
            Self::InvalidRemoteReference => write!(formatter, "invalid remote reference"),
            Self::InvalidRemoteSubpath => write!(formatter, "invalid remote subpath"),
            Self::MissingExecutable(executable) => {
                write!(formatter, "{} was not found", executable.display_name())
            }
            Self::Command(error) => error.fmt(formatter),
            Self::InvalidResponse(message) => {
                write!(formatter, "invalid provider response: {message}")
            }
            Self::Unauthenticated => write!(formatter, "provider is not authenticated"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<CommandError> for ProviderError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcloneBackend {
    GoogleDrive,
    Box,
    Smb,
}

impl RcloneBackend {
    const fn expected_type(self) -> &'static str {
        match self {
            Self::GoogleDrive => "drive",
            Self::Box => "box",
            Self::Smb => "smb",
        }
    }
}

impl TryFrom<Provider> for RcloneBackend {
    type Error = ProviderError;

    fn try_from(provider: Provider) -> Result<Self, Self::Error> {
        match provider {
            Provider::GoogleDrive => Ok(Self::GoogleDrive),
            Provider::Box => Ok(Self::Box),
            Provider::Smb => Ok(Self::Smb),
            Provider::OneDrive => Err(ProviderError::UnsupportedProvider(provider)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcloneRemote {
    pub name: String,
    pub backend: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcloneMountPlan {
    pub connection_id: ConnectionId,
    pub remote: String,
    pub mountpoint: PathBuf,
    pub cache_directory: PathBuf,
    pub rc_socket: PathBuf,
    pub service: ServiceSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnedriverMountPlan {
    pub connection_id: ConnectionId,
    pub mountpoint: PathBuf,
    pub cache_directory: PathBuf,
    pub config_file: PathBuf,
    pub service: ServiceSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RcloneVfsHealth {
    pub queued_uploads: u64,
    pub active_uploads: u64,
    pub cache_errors: u64,
    pub cache_exhausted: bool,
}

impl RcloneVfsHealth {
    #[must_use]
    pub const fn pending_or_active_writes(&self) -> bool {
        self.queued_uploads > 0 || self.active_uploads > 0
    }

    #[must_use]
    pub const fn healthy_cache(&self) -> bool {
        self.cache_errors == 0 && !self.cache_exhausted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetachDecision {
    CleanUnmount,
    AutoDetachSafe,
    BlockedByPendingWrites,
    BlockedByCacheError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LazyUnmountDecision {
    OfferAfterConfirmation,
    BlockedByPendingWrites,
    BlockedByCacheError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadinessSnapshot {
    pub network_ready: bool,
    pub vpn_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnlineRuntimeSnapshot {
    pub readiness: ReadinessSnapshot,
    pub service: Option<UnitStatus>,
    pub mount_present: bool,
    pub rclone_health: Option<RcloneVfsHealth>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnedriverCacheState {
    NoCache,
    CachedReadOnlyCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnedriverRuntimeSnapshot {
    pub readiness: ReadinessSnapshot,
    pub authenticated: bool,
    pub service: Option<UnitStatus>,
    pub mount_present: bool,
    pub cache_state: OnedriverCacheState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemountBackoff {
    current: Duration,
    max: Duration,
}

impl RemountBackoff {
    #[must_use]
    pub const fn new(initial: Duration, max: Duration) -> Self {
        Self {
            current: initial,
            max,
        }
    }

    #[must_use]
    pub const fn current(self) -> Duration {
        self.current
    }

    #[must_use]
    pub fn after_failure(self) -> Self {
        Self {
            current: self.current.saturating_mul(2).min(self.max),
            max: self.max,
        }
    }

    #[must_use]
    pub const fn after_success(self) -> Self {
        Self {
            current: Duration::from_secs(5),
            max: self.max,
        }
    }
}

impl Default for RemountBackoff {
    fn default() -> Self {
        Self::new(Duration::from_secs(5), Duration::from_secs(300))
    }
}

pub trait RcloneRemoteInventory: Send + Sync {
    fn list_remotes<'a>(
        &'a self,
        cancellation: CancellationToken,
    ) -> ProviderFuture<'a, Vec<RcloneRemote>>;
}

pub struct CommandRcloneProvider<R> {
    runner: R,
}

impl<R> CommandRcloneProvider<R> {
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: CommandRunner> RcloneRemoteInventory for CommandRcloneProvider<R> {
    fn list_remotes<'a>(
        &'a self,
        cancellation: CancellationToken,
    ) -> ProviderFuture<'a, Vec<RcloneRemote>> {
        Box::pin(async move {
            let output = self
                .runner
                .run(
                    CommandRequest::new(Executable::Rclone)
                        .arg("config")?
                        .arg("dump")?
                        .with_timeout(Duration::from_secs(5)),
                    cancellation,
                )
                .await?;
            parse_rclone_config_dump(&output.stdout.text)
        })
    }
}

impl<R: CommandRunner> CommandRcloneProvider<R> {
    pub fn validate_remote<'a>(
        &'a self,
        connection: &'a Connection,
        cancellation: CancellationToken,
    ) -> ProviderFuture<'a, ()> {
        Box::pin(async move {
            let expected = RcloneBackend::try_from(connection.provider)?.expected_type();
            let remotes = self.list_remotes(cancellation).await?;
            let Some(remote) = remotes
                .iter()
                .find(|remote| remote.name == connection.remote_reference)
            else {
                return Err(ProviderError::InvalidRemoteReference);
            };
            if remote.backend.as_deref() == Some(expected) {
                Ok(())
            } else {
                Err(ProviderError::UnsupportedProvider(connection.provider))
            }
        })
    }

    pub fn vfs_health<'a>(
        &'a self,
        rc_socket: &'a Path,
        cancellation: CancellationToken,
    ) -> ProviderFuture<'a, RcloneVfsHealth> {
        Box::pin(async move {
            let output = self
                .runner
                .run(
                    CommandRequest::new(Executable::Rclone)
                        .arg("rc")?
                        .arg("--unix-socket")?
                        .arg(rc_socket.as_os_str())?
                        .arg("vfs/stats")?
                        .with_timeout(Duration::from_secs(5)),
                    cancellation,
                )
                .await?;
            parse_vfs_stats(&output.stdout.text)
        })
    }
}

pub fn rclone_mount_plan(
    connection: &Connection,
    runtime_directory: &Path,
    default_cache_root: &Path,
) -> Result<RcloneMountPlan, ProviderError> {
    RcloneBackend::try_from(connection.provider)?;
    let options = online_options(connection)?;
    let remote = rclone_remote_path(
        &connection.remote_reference,
        connection.remote_subpath.as_deref(),
    )?;
    let cache_directory = options.cache_directory.clone().unwrap_or_else(|| {
        default_cache_root
            .join("rclone")
            .join(connection.id.to_string())
    });
    let rc_socket = runtime_directory.join(format!("rclone-{}.sock", connection.id));
    let cache_limit = format_cache_limit(options.cache_limit_bytes);
    let (timeout, contimeout) = rclone_mount_timeouts(connection.provider);
    let mut arguments = vec![
        "mount".to_owned(),
        remote.clone(),
        connection.local_path.display().to_string(),
        "--vfs-cache-mode".to_owned(),
        "full".to_owned(),
        "--vfs-cache-max-age".to_owned(),
        "168h".to_owned(),
        "--vfs-cache-max-size".to_owned(),
        cache_limit,
        "--vfs-cache-poll-interval".to_owned(),
        "5m".to_owned(),
        "--dir-cache-time".to_owned(),
        "5m".to_owned(),
        "--timeout".to_owned(),
        timeout.to_owned(),
        "--contimeout".to_owned(),
        contimeout.to_owned(),
        "--low-level-retries".to_owned(),
        "1".to_owned(),
        "--retries".to_owned(),
        "1".to_owned(),
        "--retries-sleep".to_owned(),
        "5s".to_owned(),
        "--umask".to_owned(),
        "002".to_owned(),
        "--log-level".to_owned(),
        "INFO".to_owned(),
        "--cache-dir".to_owned(),
        cache_directory.display().to_string(),
        "--rc".to_owned(),
        "--rc-addr".to_owned(),
        format!("unix://{}", rc_socket.display()),
    ];
    arguments.push("--no-modtime".to_owned());

    Ok(RcloneMountPlan {
        connection_id: connection.id,
        remote,
        mountpoint: connection.local_path.clone(),
        cache_directory,
        rc_socket,
        service: ServiceSpec {
            connection_id: connection.id,
            description: format!("COSMIC Cloud Mounter: {}", connection.name),
            executable: PathBuf::from(DEFAULT_RCLONE),
            arguments,
            restart_on_failure: true,
        },
    })
}

fn rclone_mount_timeouts(provider: Provider) -> (&'static str, &'static str) {
    match provider {
        Provider::Smb => ("90s", "15s"),
        Provider::GoogleDrive | Provider::Box | Provider::OneDrive => ("10s", "5s"),
    }
}

pub fn onedriver_mount_plan(
    connection: &Connection,
    default_cache_root: &Path,
    default_config_root: &Path,
) -> Result<OnedriverMountPlan, ProviderError> {
    if connection.provider != Provider::OneDrive {
        return Err(ProviderError::UnsupportedProvider(connection.provider));
    }
    let options = online_options(connection)?;
    let cache_directory = options.cache_directory.clone().unwrap_or_else(|| {
        default_cache_root
            .join("onedriver")
            .join(connection.id.to_string())
    });
    let config_file = default_config_root
        .join("onedriver")
        .join(connection.id.to_string())
        .join("config.json");
    let arguments = vec![
        "--config-file".to_owned(),
        config_file.display().to_string(),
        "--cache-dir".to_owned(),
        cache_directory.display().to_string(),
        connection.local_path.display().to_string(),
    ];
    Ok(OnedriverMountPlan {
        connection_id: connection.id,
        mountpoint: connection.local_path.clone(),
        cache_directory,
        config_file,
        service: ServiceSpec {
            connection_id: connection.id,
            description: format!("COSMIC Cloud Mounter: {}", connection.name),
            executable: PathBuf::from(DEFAULT_ONEDRIVER),
            arguments,
            restart_on_failure: true,
        },
    })
}

pub fn clean_unmount_request(mountpoint: &Path) -> Result<CommandRequest, CommandError> {
    CommandRequest::new(Executable::Fusermount3)
        .arg("-u")?
        .arg(mountpoint.as_os_str())?
        .with_timeout(Duration::from_secs(10))
        .with_retry(RetryPolicy {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        })
        .pipe(Ok)
}

pub fn lazy_unmount_request(mountpoint: &Path) -> Result<CommandRequest, CommandError> {
    CommandRequest::new(Executable::Fusermount3)
        .arg("-uz")?
        .arg(mountpoint.as_os_str())?
        .with_timeout(Duration::from_secs(10))
        .pipe(Ok)
}

#[must_use]
pub const fn detach_decision(health: &RcloneVfsHealth, connectivity_lost: bool) -> DetachDecision {
    if health.pending_or_active_writes() {
        DetachDecision::BlockedByPendingWrites
    } else if !health.healthy_cache() {
        DetachDecision::BlockedByCacheError
    } else if connectivity_lost {
        DetachDecision::AutoDetachSafe
    } else {
        DetachDecision::CleanUnmount
    }
}

#[must_use]
pub const fn lazy_unmount_decision(health: &RcloneVfsHealth) -> LazyUnmountDecision {
    if health.pending_or_active_writes() {
        LazyUnmountDecision::BlockedByPendingWrites
    } else if !health.healthy_cache() {
        LazyUnmountDecision::BlockedByCacheError
    } else {
        LazyUnmountDecision::OfferAfterConfirmation
    }
}

#[must_use]
pub fn rclone_online_status(snapshot: &OnlineRuntimeSnapshot) -> crate::model::OnlineMountStatus {
    use crate::model::OnlineMountStatus;

    if !snapshot.readiness.network_ready {
        return OnlineMountStatus::WaitingForNetwork;
    }
    if !snapshot.readiness.vpn_ready {
        return OnlineMountStatus::WaitingForVpn;
    }
    if let Some(status) = &snapshot.service
        && status.active == ActiveState::Failed
    {
        return OnlineMountStatus::Error;
    }
    if let Some(health) = &snapshot.rclone_health
        && health.pending_or_active_writes()
    {
        return OnlineMountStatus::PendingWrites;
    }
    if snapshot.mount_present {
        OnlineMountStatus::Mounted
    } else if snapshot.service.as_ref().is_some_and(|status| {
        matches!(status.active, ActiveState::Active | ActiveState::Activating)
    }) {
        OnlineMountStatus::Mounting
    } else {
        OnlineMountStatus::Unmounted
    }
}

#[must_use]
pub fn onedriver_online_status(
    snapshot: &OnedriverRuntimeSnapshot,
) -> crate::model::OnlineMountStatus {
    use crate::model::OnlineMountStatus;

    if !snapshot.authenticated {
        return OnlineMountStatus::Unavailable;
    }
    if !snapshot.readiness.network_ready
        && matches!(
            snapshot.cache_state,
            OnedriverCacheState::CachedReadOnlyCandidate
        )
        && snapshot.mount_present
    {
        return OnlineMountStatus::Mounted;
    }
    if !snapshot.readiness.network_ready {
        return OnlineMountStatus::WaitingForNetwork;
    }
    if !snapshot.readiness.vpn_ready {
        return OnlineMountStatus::WaitingForVpn;
    }
    if let Some(status) = &snapshot.service
        && status.active == ActiveState::Failed
    {
        return OnlineMountStatus::Error;
    }
    if snapshot.mount_present {
        OnlineMountStatus::Mounted
    } else if snapshot.service.as_ref().is_some_and(|status| {
        matches!(status.active, ActiveState::Active | ActiveState::Activating)
    }) {
        OnlineMountStatus::Mounting
    } else {
        OnlineMountStatus::Unmounted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnedriverAuthState {
    Unauthenticated,
    Authenticated { config_file: PathBuf },
}

#[must_use]
pub fn onedriver_auth_state(config_file: &Path) -> OnedriverAuthState {
    if config_file.is_file() {
        OnedriverAuthState::Authenticated {
            config_file: config_file.to_path_buf(),
        }
    } else {
        OnedriverAuthState::Unauthenticated
    }
}

#[must_use]
pub fn onedriver_auth_state_for_plan(plan: &OnedriverMountPlan) -> OnedriverAuthState {
    let config_state = onedriver_auth_state(&plan.config_file);
    if matches!(config_state, OnedriverAuthState::Authenticated { .. }) {
        return config_state;
    }
    let direct_tokens = plan.cache_directory.join("auth_tokens.json");
    if direct_tokens.is_file() {
        return OnedriverAuthState::Authenticated {
            config_file: direct_tokens,
        };
    }
    let Ok(entries) = std::fs::read_dir(&plan.cache_directory) else {
        return OnedriverAuthState::Unauthenticated;
    };
    for entry in entries.flatten() {
        let token_file = entry.path().join("auth_tokens.json");
        if token_file.is_file() {
            return OnedriverAuthState::Authenticated {
                config_file: token_file,
            };
        }
    }
    OnedriverAuthState::Unauthenticated
}

#[must_use]
pub fn onedriver_cache_state(cache_directory: &Path) -> OnedriverCacheState {
    if cache_directory.is_dir() {
        OnedriverCacheState::CachedReadOnlyCandidate
    } else {
        OnedriverCacheState::NoCache
    }
}

fn online_options(connection: &Connection) -> Result<&OnlineMountConfig, ProviderError> {
    match &connection.mode {
        ConnectionMode::OnlineMount(options) => Ok(options),
        ConnectionMode::OfflineMirror(_) => Err(ProviderError::InvalidMode),
    }
}

fn rclone_remote_path(reference: &str, subpath: Option<&str>) -> Result<String, ProviderError> {
    validate_remote_name(reference)?;
    match subpath {
        Some(subpath) => {
            validate_remote_subpath(subpath)?;
            Ok(format!("{reference}:{subpath}"))
        }
        None => Ok(format!("{reference}:")),
    }
}

fn validate_remote_name(value: &str) -> Result<(), ProviderError> {
    let valid = !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ' ')
        });
    if valid {
        Ok(())
    } else {
        Err(ProviderError::InvalidRemoteReference)
    }
}

fn validate_remote_subpath(value: &str) -> Result<(), ProviderError> {
    let path = Path::new(value);
    let valid = !value.trim().is_empty()
        && !path.is_absolute()
        && !value.contains('\\')
        && !value.chars().any(char::is_control)
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir));
    if valid {
        Ok(())
    } else {
        Err(ProviderError::InvalidRemoteSubpath)
    }
}

fn parse_rclone_config_dump(output: &str) -> Result<Vec<RcloneRemote>, ProviderError> {
    let value: Value = serde_json::from_str(output)
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        ProviderError::InvalidResponse("rclone config dump is not an object".into())
    })?;
    let mut remotes = Vec::new();
    for (name, settings) in object {
        let backend = settings
            .get("type")
            .and_then(Value::as_str)
            .map(str::to_owned);
        remotes.push(RcloneRemote {
            name: name.to_owned(),
            backend,
        });
    }
    remotes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(remotes)
}

fn parse_vfs_stats(output: &str) -> Result<RcloneVfsHealth, ProviderError> {
    let value: Value = serde_json::from_str(output)
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
    let mut paths = Vec::new();
    collect_json_paths(&value, String::new(), &mut paths);
    let mut queued_uploads = 0;
    let mut active_uploads = 0;
    let mut cache_errors = 0;
    let mut cache_exhausted = false;
    for (path, value) in paths {
        let key = path.to_ascii_lowercase();
        if key.contains("queued") && key.contains("upload") {
            queued_uploads = queued_uploads.max(json_u64(&value));
        } else if (key.contains("active") || key.contains("inprogress"))
            && (key.contains("upload") || key.contains("transfer"))
        {
            active_uploads = active_uploads.max(json_u64(&value));
        } else if key.contains("error") {
            cache_errors = cache_errors.max(json_u64(&value));
        } else if key.contains("outofspace")
            || key.contains("out_of_space")
            || key.contains("exhaust")
        {
            cache_exhausted |= value.as_bool().unwrap_or_else(|| json_u64(&value) > 0);
        }
    }
    Ok(RcloneVfsHealth {
        queued_uploads,
        active_uploads,
        cache_errors,
        cache_exhausted,
    })
}

fn collect_json_paths(value: &Value, prefix: String, output: &mut Vec<(String, Value)>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_json_paths(value, next, output);
            }
        }
        Value::Array(values) => {
            let count = values.iter().filter(|value| !value.is_null()).count() as u64;
            output.push((format!("{prefix}.count"), Value::from(count)));
            for (index, value) in values.iter().enumerate() {
                collect_json_paths(value, format!("{prefix}.{index}"), output);
            }
        }
        _ => output.push((prefix, value.clone())),
    }
}

fn json_u64(value: &Value) -> u64 {
    match value {
        Value::Number(number) => number.as_u64().unwrap_or(0),
        Value::Array(values) => values.len() as u64,
        _ => 0,
    }
}

fn format_cache_limit(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    if bytes.is_multiple_of(GIB) {
        format!("{}G", bytes / GIB)
    } else {
        bytes.to_string()
    }
}

#[cfg(test)]
fn output(stdout: &str) -> crate::process::CommandOutput {
    use crate::process::CapturedOutput;

    crate::process::CommandOutput {
        command: "fake".into(),
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

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    use super::*;
    use crate::model::{ConnectionMode, OnlineMountConfig, TuningProfile};
    use crate::process::FakeCommandRunner;
    use crate::services::{ActiveState, UnitDocument, UnitStatus};

    fn id() -> ConnectionId {
        ConnectionId::from_uuid(
            Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
        )
    }

    fn connection(provider: Provider) -> Connection {
        Connection {
            id: id(),
            name: "Engineering Drive".into(),
            provider,
            mode: ConnectionMode::OnlineMount(OnlineMountConfig::default()),
            remote_reference: match provider {
                Provider::GoogleDrive => "ua_gdrive",
                Provider::Box => "ua_box",
                Provider::Smb => "ua_engr",
                Provider::OneDrive => "unused",
            }
            .into(),
            remote_subpath: Some("Projects/2026".into()),
            local_path: PathBuf::from("/home/example/Cloud/Engineering"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
        }
    }

    #[test]
    fn rclone_mount_plan_uses_provider_remote_and_bounded_defaults() {
        let plan = rclone_mount_plan(
            &connection(Provider::GoogleDrive),
            Path::new("/run/user/1000/cosmic-mounter"),
            Path::new("/home/example/.cache/cosmic-mounter"),
        )
        .expect("plan");
        assert_eq!(plan.remote, "ua_gdrive:Projects/2026");
        assert_eq!(
            plan.cache_directory,
            PathBuf::from(
                "/home/example/.cache/cosmic-mounter/rclone/2a3f5d45-e867-47e7-943f-66cf60e777ad",
            )
        );
        assert_eq!(plan.service.executable, PathBuf::from(DEFAULT_RCLONE));
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--vfs-cache-mode", "full"])
        );
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--vfs-cache-max-size", "20G"])
        );
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--timeout", "10s"])
        );
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--retries", "1"])
        );
        assert!(plan.service.arguments.contains(&"--rc".to_owned()));
    }

    #[test]
    fn rclone_mount_plan_uses_longer_smb_timeouts() {
        let plan = rclone_mount_plan(
            &connection(Provider::Smb),
            Path::new("/run/user/1000/cosmic-mounter"),
            Path::new("/home/example/.cache/cosmic-mounter"),
        )
        .expect("plan");
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--timeout", "90s"])
        );
        assert!(
            plan.service
                .arguments
                .windows(2)
                .any(|pair| pair == ["--contimeout", "15s"])
        );
    }

    #[test]
    fn rclone_service_snapshot_contains_no_shell_or_secrets() {
        let plan = rclone_mount_plan(
            &connection(Provider::Smb),
            Path::new("/run/user/1000/cosmic-mounter"),
            Path::new("/home/example/.cache/cosmic-mounter"),
        )
        .expect("plan");
        let document = UnitDocument::service(&plan.service).expect("unit");
        assert!(
            document
                .content
                .contains("ExecStart=\"/usr/bin/rclone\" \"mount\"")
        );
        assert!(
            document
                .content
                .contains("\"--vfs-cache-max-size\" \"20G\"")
        );
        assert!(!document.content.contains("password"));
        assert!(!document.content.contains("token"));
        assert!(!document.content.contains("sh -c"));
    }

    #[test]
    fn rclone_rejects_unsafe_remote_values() {
        let mut connection = connection(Provider::Box);
        connection.remote_reference = "box; rm".into();
        assert!(matches!(
            rclone_mount_plan(&connection, Path::new("/run/user/1000"), Path::new("/tmp")),
            Err(ProviderError::InvalidRemoteReference)
        ));
        connection.remote_reference = "ua_box".into();
        connection.remote_subpath = Some("../outside".into());
        assert!(matches!(
            rclone_mount_plan(&connection, Path::new("/run/user/1000"), Path::new("/tmp")),
            Err(ProviderError::InvalidRemoteSubpath)
        ));
    }

    #[tokio::test]
    async fn rclone_remote_inventory_validates_expected_backend() {
        let runner = FakeCommandRunner::default().with_resolved([Executable::Rclone]);
        runner.push(Ok(output(
            r#"{
                "ua_box": {"type": "box"},
                "ua_engr": {"type": "smb"},
                "ua_gdrive": {"type": "drive"}
            }"#,
        )));
        let provider = CommandRcloneProvider::new(runner.clone());
        provider
            .validate_remote(&connection(Provider::GoogleDrive), CancellationToken::new())
            .await
            .expect("valid remote");
        assert_eq!(
            runner.requests()[0].sanitized_command(),
            "rclone config dump"
        );
    }

    #[tokio::test]
    async fn vfs_health_detects_pending_writes_and_cache_errors() {
        let runner = FakeCommandRunner::default().with_resolved([Executable::Rclone]);
        runner.push(Ok(output(
            r#"{
                "uploads": {"queued": 2, "inProgress": 1},
                "cache": {"errors": 3, "outOfSpace": true}
            }"#,
        )));
        let provider = CommandRcloneProvider::new(runner);
        let health = provider
            .vfs_health(
                Path::new("/run/user/1000/cosmic.sock"),
                CancellationToken::new(),
            )
            .await
            .expect("health");
        assert_eq!(
            health,
            RcloneVfsHealth {
                queued_uploads: 2,
                active_uploads: 1,
                cache_errors: 3,
                cache_exhausted: true,
            }
        );
        assert_eq!(
            detach_decision(&health, true),
            DetachDecision::BlockedByPendingWrites
        );
        assert_eq!(
            lazy_unmount_decision(&health),
            LazyUnmountDecision::BlockedByPendingWrites
        );
    }

    #[test]
    fn detach_and_lazy_unmount_require_clean_health() {
        let clean = RcloneVfsHealth {
            queued_uploads: 0,
            active_uploads: 0,
            cache_errors: 0,
            cache_exhausted: false,
        };
        assert_eq!(
            detach_decision(&clean, true),
            DetachDecision::AutoDetachSafe
        );
        assert_eq!(
            lazy_unmount_decision(&clean),
            LazyUnmountDecision::OfferAfterConfirmation
        );
        let bad_cache = RcloneVfsHealth {
            cache_errors: 1,
            ..clean
        };
        assert_eq!(
            detach_decision(&bad_cache, true),
            DetachDecision::BlockedByCacheError
        );
    }

    #[test]
    fn rclone_status_combines_readiness_service_mount_and_writes() {
        use crate::model::OnlineMountStatus;

        let clean = RcloneVfsHealth {
            queued_uploads: 0,
            active_uploads: 0,
            cache_errors: 0,
            cache_exhausted: false,
        };
        let service = UnitStatus {
            active: ActiveState::Active,
            enabled: true,
            detail: "running".into(),
        };
        let mut snapshot = OnlineRuntimeSnapshot {
            readiness: ReadinessSnapshot {
                network_ready: false,
                vpn_ready: true,
            },
            service: Some(service.clone()),
            mount_present: true,
            rclone_health: Some(clean),
        };
        assert_eq!(
            rclone_online_status(&snapshot),
            OnlineMountStatus::WaitingForNetwork
        );
        snapshot.readiness.network_ready = true;
        snapshot.readiness.vpn_ready = false;
        assert_eq!(
            rclone_online_status(&snapshot),
            OnlineMountStatus::WaitingForVpn
        );
        snapshot.readiness.vpn_ready = true;
        snapshot.rclone_health = Some(RcloneVfsHealth {
            queued_uploads: 1,
            ..clean
        });
        assert_eq!(
            rclone_online_status(&snapshot),
            OnlineMountStatus::PendingWrites
        );
        snapshot.rclone_health = Some(clean);
        snapshot.mount_present = false;
        assert_eq!(rclone_online_status(&snapshot), OnlineMountStatus::Mounting);
        snapshot.service = Some(UnitStatus {
            active: ActiveState::Failed,
            ..service
        });
        assert_eq!(rclone_online_status(&snapshot), OnlineMountStatus::Error);
    }

    #[test]
    fn onedriver_plan_uses_isolated_config_and_cache() {
        let plan = onedriver_mount_plan(
            &connection(Provider::OneDrive),
            Path::new("/home/example/.cache/cosmic-mounter"),
            Path::new("/home/example/.config/cosmic-mounter"),
        )
        .expect("plan");
        assert_eq!(plan.service.executable, PathBuf::from(DEFAULT_ONEDRIVER));
        assert!(plan.service.arguments.contains(&"--config-file".to_owned()));
        assert!(plan.service.arguments.contains(&"--cache-dir".to_owned()));
        assert!(
            plan.config_file
                .starts_with("/home/example/.config/cosmic-mounter/onedriver")
        );
        assert!(
            plan.cache_directory
                .starts_with("/home/example/.cache/cosmic-mounter/onedriver")
        );
    }

    #[test]
    fn onedriver_auth_state_uses_metadata_only() {
        let temp = TempDir::new().expect("temp");
        let config = temp.path().join("config.json");
        assert_eq!(
            onedriver_auth_state(&config),
            OnedriverAuthState::Unauthenticated
        );
        std::fs::write(&config, "{}").expect("config");
        assert_eq!(
            onedriver_auth_state(&config),
            OnedriverAuthState::Authenticated {
                config_file: config
            }
        );
    }

    #[test]
    fn onedriver_auth_state_for_plan_accepts_cache_tokens_without_reading_them() {
        let temp = TempDir::new().expect("temp");
        let plan = onedriver_mount_plan(
            &connection(Provider::OneDrive),
            &temp.path().join("cache"),
            &temp.path().join("config"),
        )
        .expect("plan");
        let token_directory = plan.cache_directory.join("mount-specific-cache");
        std::fs::create_dir_all(&token_directory).expect("token dir");
        let token_file = token_directory.join("auth_tokens.json");
        std::fs::write(&token_file, "opaque-token-metadata").expect("token metadata");

        assert_eq!(
            onedriver_auth_state_for_plan(&plan),
            OnedriverAuthState::Authenticated {
                config_file: token_file
            }
        );
    }

    #[test]
    fn onedriver_status_detects_cached_read_only_offline_candidate() {
        use crate::model::OnlineMountStatus;

        let snapshot = OnedriverRuntimeSnapshot {
            readiness: ReadinessSnapshot {
                network_ready: false,
                vpn_ready: true,
            },
            authenticated: true,
            service: Some(UnitStatus {
                active: ActiveState::Active,
                enabled: true,
                detail: "running".into(),
            }),
            mount_present: true,
            cache_state: OnedriverCacheState::CachedReadOnlyCandidate,
        };
        assert_eq!(
            onedriver_online_status(&snapshot),
            OnlineMountStatus::Mounted
        );
        assert_eq!(
            onedriver_online_status(&OnedriverRuntimeSnapshot {
                authenticated: false,
                ..snapshot
            }),
            OnlineMountStatus::Unavailable
        );
    }

    #[test]
    fn onedriver_cache_state_uses_directory_metadata_only() {
        let temp = TempDir::new().expect("temp");
        let cache = temp.path().join("cache");
        assert_eq!(onedriver_cache_state(&cache), OnedriverCacheState::NoCache);
        std::fs::create_dir(&cache).expect("cache");
        assert_eq!(
            onedriver_cache_state(&cache),
            OnedriverCacheState::CachedReadOnlyCandidate
        );
    }

    #[test]
    fn unmount_requests_are_bounded_and_argument_safe() {
        assert_eq!(
            clean_unmount_request(Path::new("/home/example/Cloud Drive"))
                .expect("request")
                .sanitized_command(),
            "fusermount3 -u /home/example/Cloud Drive"
        );
        assert_eq!(
            lazy_unmount_request(Path::new("/home/example/Cloud Drive"))
                .expect("request")
                .sanitized_command(),
            "fusermount3 -uz /home/example/Cloud Drive"
        );
    }

    #[test]
    fn remount_backoff_is_bounded_and_resets() {
        let backoff = RemountBackoff::default();
        assert_eq!(backoff.current(), Duration::from_secs(5));
        let backed_off = backoff
            .after_failure()
            .after_failure()
            .after_failure()
            .after_failure()
            .after_failure()
            .after_failure()
            .after_failure();
        assert_eq!(backed_off.current(), Duration::from_secs(300));
        assert_eq!(backed_off.after_success().current(), Duration::from_secs(5));
    }
}
