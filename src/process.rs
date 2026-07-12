// SPDX-License-Identifier: MIT

//! Typed, bounded, asynchronous command execution.

use std::collections::VecDeque;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::time;
use tokio_util::sync::CancellationToken;

pub type CommandFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CommandOutput, CommandError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Executable {
    FlatpakSpawn,
    Rclone,
    Onedriver,
    OneDrive,
    Fusermount3,
    Mountpoint,
    Findmnt,
    Nmcli,
    Ip,
    Getent,
    Nc,
    Systemctl,
    SystemdAnalyze,
    Journalctl,
    Fuser,
    Cat,
    False,
    Ls,
    Mkdir,
    Printf,
    Rm,
    Sleep,
    SystemdRun,
    Touch,
    CiscoVpn,
    CiscoVpnUi,
    CiscoAgent,
    #[cfg(test)]
    TestOnly(&'static str),
}

impl Executable {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::FlatpakSpawn => "flatpak-spawn",
            Self::Rclone => "rclone",
            Self::Onedriver => "onedriver",
            Self::OneDrive => "onedrive",
            Self::Fusermount3 => "fusermount3",
            Self::Mountpoint => "mountpoint",
            Self::Findmnt => "findmnt",
            Self::Nmcli => "nmcli",
            Self::Ip => "ip",
            Self::Getent => "getent",
            Self::Nc => "nc",
            Self::Systemctl => "systemctl",
            Self::SystemdAnalyze => "systemd-analyze",
            Self::Journalctl => "journalctl",
            Self::Fuser => "fuser",
            Self::Cat => "cat",
            Self::False => "false",
            Self::Ls => "ls",
            Self::Mkdir => "mkdir",
            Self::Printf => "printf",
            Self::Rm => "rm",
            Self::Sleep => "sleep",
            Self::SystemdRun => "systemd-run",
            Self::Touch => "touch",
            Self::CiscoVpn => "Cisco Secure Client VPN CLI",
            Self::CiscoVpnUi => "Cisco Secure Client VPN UI",
            Self::CiscoAgent => "Cisco Secure Client agent",
            #[cfg(test)]
            Self::TestOnly(path) => path,
        }
    }

    fn path_candidates(self) -> Vec<&'static str> {
        match self {
            Self::FlatpakSpawn => vec!["flatpak-spawn"],
            Self::Rclone => vec!["rclone"],
            Self::Onedriver => vec!["onedriver"],
            Self::OneDrive => vec!["onedrive"],
            Self::Fusermount3 => vec!["fusermount3"],
            Self::Mountpoint => vec!["mountpoint"],
            Self::Findmnt => vec!["findmnt", "/usr/bin/findmnt"],
            Self::Nmcli => vec!["nmcli", "/usr/bin/nmcli", "/bin/nmcli"],
            Self::Ip => vec!["ip"],
            Self::Getent => vec!["getent"],
            Self::Nc => vec!["nc"],
            Self::Systemctl => vec!["systemctl"],
            Self::SystemdAnalyze => vec!["systemd-analyze"],
            Self::Journalctl => vec!["journalctl"],
            Self::Fuser => vec!["fuser"],
            Self::Cat => vec!["cat", "/usr/bin/cat"],
            Self::False => vec!["false", "/usr/bin/false"],
            Self::Ls => vec!["ls", "/usr/bin/ls"],
            Self::Mkdir => vec!["mkdir", "/usr/bin/mkdir"],
            Self::Printf => vec!["printf", "/usr/bin/printf"],
            Self::Rm => vec!["rm", "/usr/bin/rm"],
            Self::Sleep => vec!["sleep", "/usr/bin/sleep"],
            Self::SystemdRun => vec!["systemd-run", "/usr/bin/systemd-run"],
            Self::Touch => vec!["touch", "/usr/bin/touch"],
            Self::CiscoVpn => vec![
                "/opt/cisco/secureclient/bin/vpn",
                "/opt/cisco/anyconnect/bin/vpn",
            ],
            Self::CiscoVpnUi => vec![
                "/opt/cisco/secureclient/bin/vpnui",
                "/opt/cisco/anyconnect/bin/vpnui",
            ],
            Self::CiscoAgent => vec![
                "/opt/cisco/secureclient/bin/vpnagentd",
                "/opt/cisco/anyconnect/bin/vpnagentd",
            ],
            #[cfg(test)]
            Self::TestOnly(path) => vec![path],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandArg {
    value: OsString,
    sensitive: bool,
}

impl CommandArg {
    pub fn plain(value: impl Into<OsString>) -> Result<Self, CommandError> {
        Self::new(value.into(), false)
    }

    pub fn sensitive(value: impl Into<OsString>) -> Result<Self, CommandError> {
        Self::new(value.into(), true)
    }

    fn new(value: OsString, sensitive: bool) -> Result<Self, CommandError> {
        if value
            .to_string_lossy()
            .chars()
            .any(|character| character == '\0' || character == '\n' || character == '\r')
        {
            return Err(CommandError::InvalidArgument);
        }
        Ok(Self { value, sensitive })
    }

    fn sanitized(&self) -> String {
        if self.sensitive {
            "[REDACTED]".into()
        } else {
            redact_text(&self.value.to_string_lossy())
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub delay: Duration,
    pub retry_nonzero: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            delay: Duration::ZERO,
            retry_nonzero: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandRequest {
    pub executable: Executable,
    pub args: Vec<CommandArg>,
    pub timeout: Duration,
    pub output_limit: usize,
    pub retry: RetryPolicy,
}

impl CommandRequest {
    #[must_use]
    pub fn new(executable: Executable) -> Self {
        Self {
            executable,
            args: Vec::new(),
            timeout: Duration::from_secs(10),
            output_limit: 64 * 1024,
            retry: RetryPolicy::default(),
        }
    }

    pub fn arg(mut self, value: impl Into<OsString>) -> Result<Self, CommandError> {
        self.args.push(CommandArg::plain(value)?);
        Ok(self)
    }

    pub fn sensitive_arg(mut self, value: impl Into<OsString>) -> Result<Self, CommandError> {
        self.args.push(CommandArg::sensitive(value)?);
        Ok(self)
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_output_limit(mut self, output_limit: usize) -> Self {
        self.output_limit = output_limit;
        self
    }

    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    #[must_use]
    pub fn sanitized_command(&self) -> String {
        std::iter::once(self.executable.display_name().to_owned())
            .chain(self.args.iter().map(CommandArg::sanitized))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedOutput {
    pub text: String,
    pub truncated: bool,
    pub invalid_utf8: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub command: String,
    pub stdout: CapturedOutput,
    pub stderr: CapturedOutput,
    pub attempts: u8,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    MissingExecutable(Executable),
    InvalidArgument,
    Spawn {
        command: String,
        message: String,
    },
    Timeout {
        command: String,
        timeout: Duration,
    },
    Cancelled {
        command: String,
    },
    NonZero {
        command: String,
        code: Option<i32>,
        stdout: CapturedOutput,
        stderr: CapturedOutput,
        attempts: u8,
    },
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExecutable(executable) => {
                write!(formatter, "{} was not found", executable.display_name())
            }
            Self::InvalidArgument => write!(formatter, "command argument contains unsafe bytes"),
            Self::Spawn { command, message } => write!(formatter, "{command}: {message}"),
            Self::Timeout { command, timeout } => {
                write!(formatter, "{command} timed out after {timeout:?}")
            }
            Self::Cancelled { command } => write!(formatter, "{command} was cancelled"),
            Self::NonZero { command, code, .. } => {
                write!(formatter, "{command} exited with status {code:?}")
            }
        }
    }
}

impl std::error::Error for CommandError {}

pub trait CommandRunner: Send + Sync {
    fn resolve(&self, executable: Executable) -> Option<PathBuf>;

    fn run<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> CommandFuture<'a>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        executable
            .path_candidates()
            .iter()
            .find_map(|candidate| resolve_candidate(candidate))
    }

    fn run<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> CommandFuture<'a> {
        Box::pin(async move {
            let path = self
                .resolve(request.executable)
                .ok_or(CommandError::MissingExecutable(request.executable))?;
            run_with_retries(path, request, cancellation).await
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlatpakHostCommandRunner;

impl FlatpakHostCommandRunner {
    fn host_executable_arg(executable: Executable) -> Result<&'static str, CommandError> {
        executable
            .path_candidates()
            .first()
            .copied()
            .ok_or(CommandError::MissingExecutable(executable))
    }

    fn host_request(request: CommandRequest) -> Result<CommandRequest, CommandError> {
        let host_executable = Self::host_executable_arg(request.executable)?;
        let mut host_request = CommandRequest::new(Executable::FlatpakSpawn)
            .arg("--host")?
            .arg(host_executable)?
            .with_timeout(request.timeout)
            .with_output_limit(request.output_limit)
            .with_retry(request.retry);
        host_request.args.extend(request.args);
        Ok(host_request)
    }
}

impl CommandRunner for FlatpakHostCommandRunner {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        SystemCommandRunner.resolve(Executable::FlatpakSpawn)?;
        Self::host_executable_arg(executable)
            .ok()
            .map(|candidate| PathBuf::from(format!("host:{candidate}")))
    }

    fn run<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> CommandFuture<'a> {
        Box::pin(async move {
            let path = SystemCommandRunner
                .resolve(Executable::FlatpakSpawn)
                .ok_or(CommandError::MissingExecutable(Executable::FlatpakSpawn))?;
            let request = Self::host_request(request)?;
            run_with_retries(path, request, cancellation).await
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecutionMode {
    Native,
    FlatpakSpawnHost,
}

impl CommandExecutionMode {
    #[must_use]
    pub fn detect_current() -> Self {
        if Path::new("/.flatpak-info").is_file() {
            Self::FlatpakSpawnHost
        } else {
            Self::Native
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RuntimeCommandRunner {
    Native(SystemCommandRunner),
    FlatpakSpawnHost(FlatpakHostCommandRunner),
}

impl RuntimeCommandRunner {
    #[must_use]
    pub fn for_mode(mode: CommandExecutionMode) -> Self {
        match mode {
            CommandExecutionMode::Native => Self::Native(SystemCommandRunner),
            CommandExecutionMode::FlatpakSpawnHost => {
                Self::FlatpakSpawnHost(FlatpakHostCommandRunner)
            }
        }
    }

    #[must_use]
    pub fn detect_current() -> Self {
        Self::for_mode(CommandExecutionMode::detect_current())
    }
}

impl CommandRunner for RuntimeCommandRunner {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        match self {
            Self::Native(runner) => runner.resolve(executable),
            Self::FlatpakSpawnHost(runner) => runner.resolve(executable),
        }
    }

    fn run<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> CommandFuture<'a> {
        match self {
            Self::Native(runner) => runner.run(request, cancellation),
            Self::FlatpakSpawnHost(runner) => runner.run(request, cancellation),
        }
    }
}

async fn run_with_retries(
    path: PathBuf,
    request: CommandRequest,
    cancellation: CancellationToken,
) -> Result<CommandOutput, CommandError> {
    let started = Instant::now();
    let command = request.sanitized_command();
    let max_attempts = request.retry.max_attempts.max(1);

    for attempt in 1..=max_attempts {
        match run_once(&path, &request, &cancellation).await {
            Ok((status, stdout, stderr)) if status.success() => {
                return Ok(CommandOutput {
                    command,
                    stdout,
                    stderr,
                    attempts: attempt,
                    duration: started.elapsed(),
                });
            }
            Ok((status, stdout, stderr)) => {
                if attempt == max_attempts || !request.retry.retry_nonzero {
                    return Err(CommandError::NonZero {
                        command,
                        code: status.code(),
                        stdout,
                        stderr,
                        attempts: attempt,
                    });
                }
            }
            Err(error) => return Err(error),
        }

        tokio::select! {
            () = cancellation.cancelled() => {
                return Err(CommandError::Cancelled { command });
            }
            () = time::sleep(request.retry.delay) => {}
        }
    }

    unreachable!("attempt loop always returns")
}

async fn run_once(
    path: &Path,
    request: &CommandRequest,
    cancellation: &CancellationToken,
) -> Result<(std::process::ExitStatus, CapturedOutput, CapturedOutput), CommandError> {
    let command = request.sanitized_command();
    let mut child = Command::new(path)
        .args(request.args.iter().map(|argument| &argument.value))
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| CommandError::Spawn {
            command: command.clone(),
            message: redact_text(&error.to_string()),
        })?;

    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_task = tokio::spawn(read_bounded(stdout, request.output_limit));
    let stderr_task = tokio::spawn(read_bounded(stderr, request.output_limit));

    let status = tokio::select! {
        result = child.wait() => result.map_err(|error| CommandError::Spawn {
            command: command.clone(),
            message: redact_text(&error.to_string()),
        })?,
        () = cancellation.cancelled() => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            stdout_task.abort();
            stderr_task.abort();
            return Err(CommandError::Cancelled { command });
        }
        () = time::sleep(request.timeout) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            stdout_task.abort();
            stderr_task.abort();
            return Err(CommandError::Timeout {
                command,
                timeout: request.timeout,
            });
        }
    };

    let stdout = stdout_task.await.unwrap_or_else(|_| empty_output());
    let stderr = stderr_task.await.unwrap_or_else(|_| empty_output());
    Ok((status, stdout, stderr))
}

async fn read_bounded(mut reader: impl AsyncRead + Unpin, limit: usize) -> CapturedOutput {
    let mut retained = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    let mut total = 0_usize;

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                total = total.saturating_add(read);
                let available = limit.saturating_sub(retained.len());
                retained.extend_from_slice(&buffer[..read.min(available)]);
            }
        }
    }

    let invalid_utf8 = std::str::from_utf8(&retained).is_err();
    CapturedOutput {
        text: redact_text(&String::from_utf8_lossy(&retained)),
        truncated: total > retained.len(),
        invalid_utf8,
    }
}

fn empty_output() -> CapturedOutput {
    CapturedOutput {
        text: String::new(),
        truncated: false,
        invalid_utf8: false,
    }
}

fn resolve_candidate(candidate: &str) -> Option<PathBuf> {
    let candidate_path = Path::new(candidate);
    if candidate_path.is_absolute() {
        return candidate_path
            .is_file()
            .then(|| candidate_path.to_path_buf());
    }

    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|directory| directory.join(candidate))
        .find(|path| path.is_file())
}

#[must_use]
pub fn redact_text(value: &str) -> String {
    let mut redact_remaining = 0_u8;
    value
        .split_whitespace()
        .map(|token| {
            if redact_remaining > 0 {
                redact_remaining -= 1;
                return "[REDACTED]".into();
            }

            let lower = token.to_ascii_lowercase();
            if lower == "authorization:" {
                redact_remaining = 2;
                return token.to_owned();
            }
            if matches!(
                lower.as_str(),
                "bearer" | "--password" | "--passwd" | "--token" | "--client-secret"
            ) {
                redact_remaining = 1;
                return token.to_owned();
            }

            redact_token(token)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    const SENSITIVE_KEYS: &[&str] = &[
        "password",
        "passwd",
        "token",
        "secret",
        "credential",
        "authorization",
        "client_secret",
        "refresh_token",
        "access_token",
    ];

    if lower.starts_with("http://") || lower.starts_with("https://") {
        return "[REDACTED_URL]".into();
    }
    if lower.starts_with("bearer") {
        return "[REDACTED]".into();
    }
    if let Some((key, _)) = token.split_once('=')
        && SENSITIVE_KEYS
            .iter()
            .any(|sensitive| key.to_ascii_lowercase().contains(sensitive))
    {
        return format!("{key}=[REDACTED]");
    }
    token.to_owned()
}

#[derive(Clone, Default)]
pub struct FakeCommandRunner {
    resolved: Arc<Mutex<Vec<Executable>>>,
    responses: Arc<Mutex<VecDeque<Result<CommandOutput, CommandError>>>>,
    requests: Arc<Mutex<Vec<CommandRequest>>>,
}

impl FakeCommandRunner {
    #[must_use]
    pub fn with_resolved(self, executables: impl IntoIterator<Item = Executable>) -> Self {
        self.resolved
            .lock()
            .expect("fake runner lock")
            .extend(executables);
        self
    }

    pub fn push(&self, response: Result<CommandOutput, CommandError>) {
        self.responses
            .lock()
            .expect("fake runner lock")
            .push_back(response);
    }

    #[must_use]
    pub fn requests(&self) -> Vec<CommandRequest> {
        self.requests.lock().expect("fake runner lock").clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        self.resolved
            .lock()
            .expect("fake runner lock")
            .contains(&executable)
            .then(|| PathBuf::from(executable.display_name()))
    }

    fn run<'a>(
        &'a self,
        request: CommandRequest,
        _cancellation: CancellationToken,
    ) -> CommandFuture<'a> {
        Box::pin(async move {
            self.requests
                .lock()
                .expect("fake runner lock")
                .push(request);
            self.responses
                .lock()
                .expect("fake runner lock")
                .pop_front()
                .unwrap_or_else(|| {
                    Err(CommandError::Spawn {
                        command: "fake".into(),
                        message: "no fake response queued".into(),
                    })
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(path: &'static str) -> CommandRequest {
        CommandRequest::new(Executable::TestOnly(path))
    }

    #[test]
    fn nmcli_resolution_has_absolute_fallbacks_for_applet_sessions() {
        let candidates = Executable::Nmcli.path_candidates();
        assert!(candidates.contains(&"nmcli"));
        assert!(candidates.contains(&"/usr/bin/nmcli"));
    }

    #[test]
    fn flatpak_host_runner_wraps_fixed_executable_and_arguments() {
        let request = CommandRequest::new(Executable::Rclone)
            .arg("version")
            .expect("safe argument")
            .with_timeout(Duration::from_secs(22))
            .with_output_limit(256)
            .with_retry(RetryPolicy {
                max_attempts: 2,
                delay: Duration::from_millis(5),
                retry_nonzero: true,
            });
        let wrapped = FlatpakHostCommandRunner::host_request(request).expect("host request");

        assert_eq!(
            wrapped.sanitized_command(),
            "flatpak-spawn --host rclone version"
        );
        assert_eq!(wrapped.timeout, Duration::from_secs(22));
        assert_eq!(wrapped.output_limit, 256);
        assert_eq!(wrapped.retry.max_attempts, 2);
        assert_eq!(wrapped.retry.delay, Duration::from_millis(5));
        assert!(wrapped.retry.retry_nonzero);
    }

    #[test]
    fn flatpak_host_runner_preserves_sensitive_redaction() {
        let request = CommandRequest::new(Executable::Rclone)
            .arg("config")
            .expect("safe argument")
            .arg("password")
            .expect("safe argument")
            .arg("remote")
            .expect("safe argument")
            .arg("pass")
            .expect("safe argument")
            .sensitive_arg("top-secret")
            .expect("safe sensitive argument");
        let wrapped = FlatpakHostCommandRunner::host_request(request).expect("host request");

        assert_eq!(
            wrapped.sanitized_command(),
            "flatpak-spawn --host rclone config password remote pass [REDACTED]"
        );
    }

    #[test]
    fn system_runner_keeps_native_command_shape() {
        let request = CommandRequest::new(Executable::Rclone)
            .arg("version")
            .expect("safe argument");

        assert_eq!(request.sanitized_command(), "rclone version");
    }

    #[test]
    fn runtime_runner_can_be_selected_without_changing_native_default() {
        assert!(matches!(
            RuntimeCommandRunner::for_mode(CommandExecutionMode::Native),
            RuntimeCommandRunner::Native(_)
        ));
        assert!(matches!(
            RuntimeCommandRunner::for_mode(CommandExecutionMode::FlatpakSpawnHost),
            RuntimeCommandRunner::FlatpakSpawnHost(_)
        ));
    }

    #[tokio::test]
    async fn nonzero_exit_is_typed_and_sanitized() {
        let error = SystemCommandRunner
            .run(request("/usr/bin/false"), CancellationToken::new())
            .await
            .expect_err("false must fail");
        assert!(matches!(error, CommandError::NonZero { code: Some(1), .. }));
    }

    #[tokio::test]
    async fn retry_count_is_bounded() {
        let request = request("/usr/bin/false").with_retry(RetryPolicy {
            max_attempts: 3,
            delay: Duration::from_millis(1),
            retry_nonzero: true,
        });
        let error = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .expect_err("false must fail after retries");
        assert!(matches!(error, CommandError::NonZero { attempts: 3, .. }));
    }

    #[tokio::test]
    async fn timeout_kills_the_child() {
        let request = request("/usr/bin/sleep")
            .arg("5")
            .expect("safe argument")
            .with_timeout(Duration::from_millis(20));
        let error = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .expect_err("sleep must time out");
        assert!(matches!(error, CommandError::Timeout { .. }));
    }

    #[tokio::test]
    async fn cancellation_kills_the_child() {
        let cancellation = CancellationToken::new();
        let trigger = cancellation.clone();
        tokio::spawn(async move {
            time::sleep(Duration::from_millis(20)).await;
            trigger.cancel();
        });
        let request = request("/usr/bin/sleep").arg("5").expect("safe argument");
        let error = SystemCommandRunner
            .run(request, cancellation)
            .await
            .expect_err("sleep must be cancelled");
        assert!(matches!(error, CommandError::Cancelled { .. }));
    }

    #[tokio::test]
    async fn output_is_bounded_and_drained() {
        let content = "x".repeat(16 * 1024);
        let request = request("/usr/bin/printf")
            .arg(content)
            .expect("safe argument")
            .with_output_limit(128);
        let output = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .expect("printf succeeds");
        assert_eq!(output.stdout.text.len(), 128);
        assert!(output.stdout.truncated);
    }

    #[tokio::test]
    async fn invalid_utf8_is_lossy_and_reported() {
        let request = request("/usr/bin/printf")
            .arg("\\377")
            .expect("safe argument");
        let output = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .expect("printf succeeds");
        assert!(output.stdout.invalid_utf8);
        assert!(output.stdout.text.contains('\u{fffd}'));
    }

    #[test]
    fn arguments_and_output_are_redacted() {
        let request = CommandRequest::new(Executable::Rclone)
            .sensitive_arg("top-secret")
            .expect("safe sensitive argument")
            .arg("https://user:pass@example.com/path?token=secret")
            .expect("safe URL argument");
        assert_eq!(
            request.sanitized_command(),
            "rclone [REDACTED] [REDACTED_URL]"
        );
        assert_eq!(
            redact_text("access_token=abc password=hunter2"),
            "access_token=[REDACTED] password=[REDACTED]"
        );
        assert_eq!(
            redact_text("Authorization: Bearer abc123"),
            "Authorization: [REDACTED] [REDACTED]"
        );
    }

    #[test]
    fn newline_arguments_are_rejected() {
        assert!(matches!(
            CommandArg::plain("unsafe\nargument"),
            Err(CommandError::InvalidArgument)
        ));
    }
}
