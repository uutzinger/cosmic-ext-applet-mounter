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

// Microsoft authorization needs credential-bearing config JSON in memory.
// This response deliberately has no Display/Serialize or public raw output fields.
pub struct PrivateConfigOutput(CommandOutput);
impl fmt::Debug for PrivateConfigOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateConfigOutput([REDACTED])")
    }
}
impl PrivateConfigOutput {
    pub(crate) fn is_complete(&self) -> bool {
        !self.0.stdout.truncated
            && !self.0.stdout.invalid_utf8
            && !self.0.stderr.truncated
            && !self.0.stderr.invalid_utf8
    }
    /// A browser launcher may surround the OAuth JSON with diagnostics. Keep
    /// parsing inside the opaque capture; never expose either raw stream.
    pub(crate) fn oauth_json(&self) -> Result<serde_json::Value, CommandError> {
        if !self.is_complete() {
            return Err(CommandError::InvalidArgument);
        }
        for text in [&self.0.stdout.text, &self.0.stderr.text] {
            for (start, _) in text.match_indices('{').take(32) {
                let mut values = serde_json::Deserializer::from_str(&text[start..])
                    .into_iter::<serde_json::Value>();
                if let Some(Ok(value)) = values.next()
                    && value
                        .get("State")
                        .and_then(serde_json::Value::as_str)
                        .is_some()
                {
                    return Ok(value);
                }
            }
        }
        Err(CommandError::InvalidArgument)
    }

    pub fn json(&self) -> Result<serde_json::Value, CommandError> {
        if !self.is_complete() {
            return Err(CommandError::InvalidArgument);
        }
        serde_json::from_str(&self.0.stdout.text).map_err(|_| CommandError::InvalidArgument)
    }
    #[cfg(test)]
    pub(crate) fn test_output(output: CommandOutput) -> Self {
        Self(output)
    }
}
pub type PrivateConfigFuture<'a> =
    Pin<Box<dyn Future<Output = Result<PrivateConfigOutput, CommandError>> + Send + 'a>>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum CapturePolicy {
    Redacted,
    PrivateConfig,
}
fn validate_private_config(request: &CommandRequest) -> Result<(), CommandError> {
    if request.executable != Executable::Rclone
        || request.launch_only
        || request.output_limit > 1024 * 1024
        || request.timeout > Duration::from_secs(300)
        || request.args.first().and_then(|a| a.value.to_str()) != Some("config")
        || !matches!(
            request.args.get(1).and_then(|a| a.value.to_str()),
            Some("dump" | "create" | "update")
        )
    {
        return Err(CommandError::InvalidArgument);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Executable {
    FlatpakSpawn,
    Flatpak,
    Rclone,
    Onedriver,
    OneDrive,
    Fusermount3,
    Mountpoint,
    #[cfg(any(test, feature = "development-probes"))]
    Findmnt,
    Find,
    #[cfg(any(test, feature = "development-probes"))]
    Nmcli,
    #[cfg(any(test, feature = "development-probes"))]
    Systemctl,
    Cat,
    #[cfg(any(test, feature = "development-probes"))]
    False,
    #[cfg(any(test, feature = "development-probes"))]
    Ls,
    #[cfg(any(test, feature = "development-probes"))]
    Mkdir,
    #[cfg(any(test, feature = "development-probes"))]
    Rm,
    #[cfg(any(test, feature = "development-probes"))]
    Sleep,
    #[cfg(any(test, feature = "development-probes"))]
    SystemdRun,
    #[cfg(any(test, feature = "development-probes"))]
    Touch,
    #[cfg(test)]
    TestOnly(&'static str),
}

impl Executable {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::FlatpakSpawn => "flatpak-spawn",
            Self::Flatpak => "flatpak",
            Self::Rclone => "rclone",
            Self::Onedriver => "onedriver",
            Self::OneDrive => "onedrive",
            Self::Fusermount3 => "fusermount3",
            Self::Mountpoint => "mountpoint",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Findmnt => "findmnt",
            Self::Find => "find",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Nmcli => "nmcli",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Systemctl => "systemctl",
            Self::Cat => "cat",
            #[cfg(any(test, feature = "development-probes"))]
            Self::False => "false",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Ls => "ls",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Mkdir => "mkdir",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Rm => "rm",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Sleep => "sleep",
            #[cfg(any(test, feature = "development-probes"))]
            Self::SystemdRun => "systemd-run",
            #[cfg(any(test, feature = "development-probes"))]
            Self::Touch => "touch",
            #[cfg(test)]
            Self::TestOnly(path) => path,
        }
    }

    fn path_candidates(self) -> Vec<&'static str> {
        match self {
            Self::FlatpakSpawn => vec!["flatpak-spawn"],
            Self::Flatpak => vec!["flatpak", "/usr/bin/flatpak"],
            Self::Rclone => vec!["rclone"],
            Self::Onedriver => vec!["onedriver"],
            Self::OneDrive => vec!["onedrive"],
            Self::Fusermount3 => vec!["fusermount3"],
            Self::Mountpoint => vec!["mountpoint"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Findmnt => vec!["findmnt", "/usr/bin/findmnt"],
            Self::Find => vec!["find", "/usr/bin/find"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Nmcli => vec!["nmcli", "/usr/bin/nmcli", "/bin/nmcli"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Systemctl => vec!["systemctl"],
            Self::Cat => vec!["cat", "/usr/bin/cat"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::False => vec!["false", "/usr/bin/false"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Ls => vec!["ls", "/usr/bin/ls"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Mkdir => vec!["mkdir", "/usr/bin/mkdir"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Rm => vec!["rm", "/usr/bin/rm"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Sleep => vec!["sleep", "/usr/bin/sleep"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::SystemdRun => vec!["systemd-run", "/usr/bin/systemd-run"],
            #[cfg(any(test, feature = "development-probes"))]
            Self::Touch => vec!["touch", "/usr/bin/touch"],
            #[cfg(test)]
            Self::TestOnly(path) => vec![path],
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CommandArg {
    value: OsString,
    sensitive: bool,
}

impl fmt::Debug for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandArg")
            .field("value", &self.sanitized())
            .field("sensitive", &self.sensitive)
            .finish()
    }
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
    /// Return once spawned; the interactive application owns its lifetime.
    pub launch_only: bool,
    /// Terminate a read-only host child if its Flatpak transport disappears.
    pub watch_host_bus: bool,
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
            launch_only: false,
            watch_host_bus: false,
        }
    }

    #[must_use]
    pub fn launch_only(mut self) -> Self {
        self.launch_only = true;
        self
    }

    #[must_use]
    pub fn watch_host_bus(mut self) -> Self {
        self.watch_host_bus = true;
        self
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
            Self::MissingExecutable(executable) => formatter.write_str(&crate::fl!(
                "ui-process-value1-was-not-found",
                value1 = (executable.display_name()).to_string()
            )),
            Self::InvalidArgument => formatter.write_str(&crate::fl!(
                "ui-process-command-argument-contains-unsafe-bytes"
            )),
            Self::Spawn { command, message } => write!(formatter, "{command}: {message}"),
            Self::Timeout { command, timeout } => formatter.write_str(&crate::fl!(
                "ui-process-command-timed-out-after-timeout",
                command = (command).to_string(),
                timeout = format!("{:?}", timeout)
            )),
            Self::Cancelled { command } => formatter.write_str(&crate::fl!(
                "ui-process-command-was-cancelled",
                command = (command).to_string()
            )),
            Self::NonZero { command, code, .. } => formatter.write_str(&crate::fl!(
                "ui-process-command-exited-with-status-code",
                command = (command).to_string(),
                code = format!("{:?}", code)
            )),
        }
    }
}

impl std::error::Error for CommandError {}

pub trait CommandRunner: Send + Sync {
    fn resolve(&self, executable: Executable) -> Option<PathBuf>;
    /// Explicit, restricted in-memory config capture. Ordinary command output
    /// remains redacted; adapters must opt in rather than silently return a dump.
    fn private_config<'a>(
        &'a self,
        _request: CommandRequest,
        _cancellation: CancellationToken,
    ) -> PrivateConfigFuture<'a> {
        Box::pin(async { Err(CommandError::InvalidArgument) })
    }

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
            run_with_retries(path, request, cancellation, CapturePolicy::Redacted).await
        })
    }
    fn private_config<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> PrivateConfigFuture<'a> {
        Box::pin(async move {
            validate_private_config(&request)?;
            let path = self
                .resolve(Executable::Rclone)
                .ok_or(CommandError::MissingExecutable(Executable::Rclone))?;
            run_with_retries(path, request, cancellation, CapturePolicy::PrivateConfig)
                .await
                .map(PrivateConfigOutput)
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
            .with_timeout(request.timeout)
            .with_output_limit(request.output_limit)
            .with_retry(request.retry);
        if request.watch_host_bus {
            host_request = host_request.arg("--watch-bus")?;
        }
        host_request = host_request.arg(host_executable)?;
        host_request.launch_only = request.launch_only;
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
            run_with_retries(path, request, cancellation, CapturePolicy::Redacted).await
        })
    }
    fn private_config<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> PrivateConfigFuture<'a> {
        Box::pin(async move {
            validate_private_config(&request)?;
            let path = SystemCommandRunner
                .resolve(Executable::FlatpakSpawn)
                .ok_or(CommandError::MissingExecutable(Executable::FlatpakSpawn))?;
            let request = Self::host_request(request)?;
            run_with_retries(path, request, cancellation, CapturePolicy::PrivateConfig)
                .await
                .map(PrivateConfigOutput)
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
    fn private_config<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> PrivateConfigFuture<'a> {
        match self {
            Self::Native(runner) => runner.private_config(request, cancellation),
            Self::FlatpakSpawnHost(runner) => runner.private_config(request, cancellation),
        }
    }
}

async fn run_with_retries(
    path: PathBuf,
    request: CommandRequest,
    cancellation: CancellationToken,
    capture: CapturePolicy,
) -> Result<CommandOutput, CommandError> {
    let started = Instant::now();
    let command = request.sanitized_command();
    if request.launch_only {
        if cancellation.is_cancelled() {
            return Err(CommandError::Cancelled { command });
        }
        let mut child = Command::new(path)
            .args(request.args.iter().map(|argument| &argument.value))
            .kill_on_drop(false)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| CommandError::Spawn {
                command: command.clone(),
                message: redact_text(&error.to_string()),
            })?;
        // Reap on exit without imposing the command timeout on authentication.
        // A successful spawn is not evidence that the VPN is connected.
        tokio::spawn(async move {
            let _ = child.wait().await;
        });
        return Ok(CommandOutput {
            command,
            stdout: empty_output(),
            stderr: empty_output(),
            attempts: 1,
            duration: started.elapsed(),
        });
    }
    let max_attempts = request.retry.max_attempts.max(1);

    for attempt in 1..=max_attempts {
        match run_once(&path, &request, &cancellation, capture).await {
            Ok((status, stdout, stderr)) if status.success() => {
                return Ok(CommandOutput {
                    command,
                    stdout,
                    stderr,
                    attempts: attempt,
                    duration: started.elapsed(),
                });
            }
            Ok((status, mut stdout, mut stderr)) => {
                if capture != CapturePolicy::Redacted {
                    // Even failed private commands return only opaque output.
                    stdout.text = "[REDACTED]".into();
                    stderr.text = "[REDACTED]".into();
                }
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
    capture: CapturePolicy,
) -> Result<(std::process::ExitStatus, CapturedOutput, CapturedOutput), CommandError> {
    let command = request.sanitized_command();
    let mut command_builder = Command::new(path);
    command_builder
        .args(request.args.iter().map(|argument| &argument.value))
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(unix)]
    command_builder.process_group(0);
    let mut child = command_builder
        .spawn()
        .map_err(|error| CommandError::Spawn {
            command: command.clone(),
            message: redact_text(&error.to_string()),
        })?;

    let process_group = child.id();

    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let mut stdout_task = tokio::spawn(read_bounded(stdout, request.output_limit, capture));
    let mut stderr_task = tokio::spawn(read_bounded(stderr, request.output_limit, capture));

    // The deadline covers pipe draining too: a descendant can hold a pipe open
    // after the direct child exits. Save its process group before wait reaps it.
    tokio::select! {
        result = async {
            let status = child.wait().await.map_err(|error| CommandError::Spawn {
            command: command.clone(),
            message: redact_text(&error.to_string()),
            })?;
            let stdout = (&mut stdout_task).await.unwrap_or_else(|_| empty_output());
            let stderr = (&mut stderr_task).await.unwrap_or_else(|_| empty_output());
            Ok((status, stdout, stderr))
        } => result,
        () = cancellation.cancelled() => {
            terminate_command_tree(&mut child, process_group).await;
            stdout_task.abort();
            stderr_task.abort();
            Err(CommandError::Cancelled { command })
        }
        () = time::sleep(request.timeout) => {
            terminate_command_tree(&mut child, process_group).await;
            stdout_task.abort();
            stderr_task.abort();
            Err(CommandError::Timeout {
                command,
                timeout: request.timeout,
            })
        }
    }
}

#[cfg(unix)]
fn signal_process_group(process_group: u32, signal: i32) -> bool {
    let Ok(process_group) = i32::try_from(process_group) else {
        return false;
    };
    // SAFETY: the child was placed in a new process group whose positive ID is
    // returned by Child::id. A negative PID addresses only that group. The
    // signal values are libc constants and no pointers cross the FFI boundary.
    let result = unsafe { libc::kill(-process_group, signal) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
}

#[cfg(unix)]
fn process_group_exists(process_group: u32) -> bool {
    let Ok(process_group) = i32::try_from(process_group) else {
        return false;
    };
    // SAFETY: signal 0 performs an existence/permission check and the negative
    // PID is the isolated child process group created above.
    let result = unsafe { libc::kill(-process_group, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

async fn terminate_command_tree(child: &mut tokio::process::Child, process_group: Option<u32>) {
    #[cfg(unix)]
    {
        if let Some(process_group) = process_group {
            let _ = signal_process_group(process_group, libc::SIGTERM);
            let _ = time::timeout(Duration::from_millis(500), child.wait()).await;
            let graceful_deadline = time::Instant::now() + Duration::from_millis(500);
            while process_group_exists(process_group) && time::Instant::now() < graceful_deadline {
                time::sleep(Duration::from_millis(20)).await;
            }
            if process_group_exists(process_group) {
                let _ = signal_process_group(process_group, libc::SIGKILL);
            }
            let kill_deadline = time::Instant::now() + Duration::from_secs(1);
            while process_group_exists(process_group) && time::Instant::now() < kill_deadline {
                time::sleep(Duration::from_millis(20)).await;
            }
            let _ = time::timeout(Duration::from_millis(500), child.wait()).await;
            return;
        }
    }
    let _ = child.kill().await;
    let _ = child.wait().await;
}

async fn read_bounded(
    mut reader: impl AsyncRead + Unpin,
    limit: usize,
    capture: CapturePolicy,
) -> CapturedOutput {
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
        text: if capture == CapturePolicy::Redacted {
            redact_text(&String::from_utf8_lossy(&retained))
        } else {
            String::from_utf8_lossy(&retained).into_owned()
        },
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
    // Structured command output is consumed by policy code as JSON. Redacting
    // whitespace-delimited fragments can break escaped OAuth token strings or
    // expose their remainder. Remove sensitive fields before serializing again.
    if let Ok(mut structured) = serde_json::from_str::<serde_json::Value>(value) {
        redact_json_value(&mut structured);
        return serde_json::to_string(&structured).expect("JSON values serialize");
    }
    redact_plain_text(value)
}

fn redact_json_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                let key = key.to_ascii_lowercase();
                if matches!(
                    key.as_str(),
                    "pass"
                        | "password"
                        | "passwd"
                        | "token"
                        | "secret"
                        | "credential"
                        | "credentials"
                        | "authorization"
                        | "client_id"
                        | "client_secret"
                        | "key_pem"
                        | "key_file_pass"
                        | "private_key"
                ) || key.ends_with("_token")
                    || key.ends_with("_password")
                    || key.ends_with("_secret")
                    || key.ends_with("_credentials")
                {
                    *value = serde_json::Value::String("[REDACTED]".into());
                } else {
                    redact_json_value(value);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(redact_json_value),
        serde_json::Value::String(text) => *text = redact_plain_text(text),
        _ => {}
    }
}

fn redact_plain_text(value: &str) -> String {
    let mut redact_remaining = 0_u8;
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while !rest.is_empty() {
        let whitespace = rest
            .find(|character: char| !character.is_whitespace())
            .unwrap_or(rest.len());
        result.push_str(&rest[..whitespace]);
        rest = &rest[whitespace..];
        if rest.is_empty() {
            break;
        }
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..end];
        let lower = token.to_ascii_lowercase();
        let normalized_key =
            lower.trim_matches(|character| matches!(character, '"' | '\'' | ':' | ',' | '{' | '}'));
        if redact_remaining > 0 {
            redact_remaining -= 1;
            result.push_str(&redact_value_token(token));
        } else if lower == "authorization:" {
            redact_remaining = 2;
            result.push_str(token);
        } else if matches!(
            lower.as_str(),
            "bearer" | "--password" | "--passwd" | "--token" | "--client-id" | "--client-secret"
        ) || matches!(normalized_key, "client_id" | "client_secret")
        {
            redact_remaining = 1;
            result.push_str(token);
        } else {
            result.push_str(&redact_token(token));
        }
        rest = &rest[end..];
    }
    result
}

fn redact_value_token(token: &str) -> String {
    let Some(quote @ ('"' | '\'')) = token.chars().next() else {
        return "[REDACTED]".into();
    };
    let Some(closing_quote) = token.rfind(quote).filter(|index| *index > 0) else {
        return "[REDACTED]".into();
    };
    format!("{quote}[REDACTED]{quote}{}", &token[closing_quote + 1..])
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
        "client_id",
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
    fn private_config<'a>(
        &'a self,
        request: CommandRequest,
        cancellation: CancellationToken,
    ) -> PrivateConfigFuture<'a> {
        Box::pin(async move {
            validate_private_config(&request)?;
            self.run(request, cancellation)
                .await
                .map(PrivateConfigOutput)
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

    #[test]
    fn host_inspection_transport_owns_child_lifetime_without_changing_other_launches() {
        let request = CommandRequest::new(Executable::Cat)
            .arg("/proc/self/mountinfo")
            .unwrap()
            .watch_host_bus()
            .with_timeout(Duration::from_secs(10))
            .with_output_limit(4 * 1024 * 1024);
        let wrapped = FlatpakHostCommandRunner::host_request(request).unwrap();
        assert!(wrapped.sanitized_command().contains("--host --watch-bus"));
        assert_eq!(wrapped.timeout, Duration::from_secs(10));
        assert_eq!(wrapped.output_limit, 4 * 1024 * 1024);
        let unrelated = FlatpakHostCommandRunner::host_request(
            CommandRequest::new(Executable::Onedriver).launch_only(),
        )
        .unwrap();
        assert!(!unrelated.sanitized_command().contains("--watch-bus"));
        assert!(unrelated.launch_only);
    }

    #[tokio::test]
    async fn interactive_launch_outlives_command_timeout() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let marker = temp.path().join("finished");
        // A disposable child writes a marker only after the short request
        // timeout. Normal bounded execution would kill it before that point.
        let request = request("/bin/sh")
            .arg("-c")
            .unwrap()
            .arg("sleep 0.1; printf done > \"$1\"")
            .unwrap()
            .arg("test-child")
            .unwrap()
            .arg(marker.as_os_str())
            .unwrap()
            .with_timeout(Duration::from_millis(5))
            .launch_only();
        SystemCommandRunner
            .run(request, CancellationToken::new())
            .await
            .expect("launch");
        let deadline = time::Instant::now() + Duration::from_secs(3);
        while !marker.exists() && time::Instant::now() < deadline {
            time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(std::fs::read_to_string(marker).unwrap(), "done");
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

    #[cfg(unix)]
    #[tokio::test]
    async fn deadline_covers_pipes_held_by_descendant_after_parent_exit() {
        let request = request("/usr/bin/sh")
            .arg("-c")
            .unwrap()
            .arg("sleep 30 & exit 0")
            .unwrap()
            .with_timeout(Duration::from_millis(50));
        let started = Instant::now();
        let result = SystemCommandRunner
            .run(request, CancellationToken::new())
            .await;
        assert!(matches!(result, Err(CommandError::Timeout { .. })));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_kills_shell_descendants_before_returning() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let marker = temp.path().join("descendant-pid");
        let cancellation = CancellationToken::new();
        let operation = cancellation.clone();
        let request = request("/usr/bin/sh")
            .arg("-c")
            .unwrap()
            .arg("sleep 30 & child=$!; printf '%s' \"$child\" > \"$1\"; wait")
            .unwrap()
            .arg("test-shell")
            .unwrap()
            .arg(marker.as_os_str())
            .unwrap();
        let running =
            tokio::spawn(async move { SystemCommandRunner.run(request, operation).await });
        let marker_deadline = time::Instant::now() + Duration::from_secs(2);
        while !marker.exists() && time::Instant::now() < marker_deadline {
            time::sleep(Duration::from_millis(10)).await;
        }
        let descendant = std::fs::read_to_string(&marker)
            .expect("shell recorded descendant")
            .trim()
            .parse::<u32>()
            .expect("descendant PID");
        assert!(Path::new(&format!("/proc/{descendant}")).exists());

        cancellation.cancel();
        let error = running
            .await
            .expect("runner task")
            .expect_err("shell command must be cancelled");
        assert!(matches!(error, CommandError::Cancelled { .. }));
        assert!(
            !Path::new(&format!("/proc/{descendant}")).exists(),
            "descendant survived command cancellation"
        );
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
        assert_eq!(
            redact_text("SendSIGKILL=no\npassword=hunter2\nTimeoutStopUSec=2s\n"),
            "SendSIGKILL=no\npassword=[REDACTED]\nTimeoutStopUSec=2s\n"
        );
        assert_eq!(
            redact_text("client_id private-id client_secret=private-secret"),
            "client_id [REDACTED] client_secret=[REDACTED]"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&redact_text(
                r#"{"client_id": "private-id", "client_secret": "private-secret"}"#
            ))
            .unwrap(),
            serde_json::json!({"client_id": "[REDACTED]", "client_secret": "[REDACTED]"})
        );
        let redacted_dump = redact_text(
            r#"{
                "ua_gdrive": {"type": "drive"},
                "uutzinger_gdrive": {
                    "type": "drive",
                    "client_id": "private-client.apps.googleusercontent.com",
                    "client_secret": "GOCSPX-private-secret"
                }
            }"#,
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&redacted_dump).expect("redacted config dump stays valid JSON");
        assert_eq!(parsed["uutzinger_gdrive"]["client_id"], "[REDACTED]");
        assert_eq!(parsed["uutzinger_gdrive"]["client_secret"], "[REDACTED]");
    }

    #[test]
    fn structured_config_redaction_preserves_escaped_tokens_and_metadata() {
        let original = serde_json::json!({
            "personal_fixture": {
                "type": "onedrive", "drive_type": "personal", "drive_id": "synthetic-drive",
                "token": "{\"access_token\":\"synthetic-access==\",\"refresh_token\":\"synthetic-refresh==\"}",
                "client_secret": "synthetic secret with spaces = punctuation",
                "client_id": "synthetic-client",
                "nested": {"password": "synthetic-password", "key_pem": "synthetic-private-key"}
            }
        });
        let text = redact_text(&serde_json::to_string_pretty(&original).unwrap());
        let parsed: serde_json::Value =
            serde_json::from_str(&text).expect("redaction preserves JSON");
        assert_eq!(parsed["personal_fixture"]["type"], "onedrive");
        assert_eq!(parsed["personal_fixture"]["drive_type"], "personal");
        assert_eq!(parsed["personal_fixture"]["drive_id"], "synthetic-drive");
        for secret in [
            "synthetic-access",
            "synthetic-refresh",
            "synthetic secret",
            "synthetic-client",
            "synthetic-password",
            "synthetic-private-key",
        ] {
            assert!(!text.contains(secret));
        }
    }

    #[test]
    fn newline_arguments_are_rejected() {
        assert!(matches!(
            CommandArg::plain("unsafe\nargument"),
            Err(CommandError::InvalidArgument)
        ));
    }
}

#[cfg(test)]
mod private_config_tests {
    use super::*;
    use serde_json::json;
    fn config_request() -> CommandRequest {
        CommandRequest::new(Executable::Rclone)
            .arg("config")
            .unwrap()
            .arg("dump")
            .unwrap()
            .with_timeout(Duration::from_secs(10))
            .with_output_limit(64 * 1024)
    }
    #[test]
    fn private_config_is_restricted_and_debug_is_opaque() {
        for req in [
            CommandRequest::new(Executable::Rclone).arg("cat").unwrap(),
            CommandRequest::new(Executable::Cat)
                .arg("config")
                .unwrap()
                .arg("dump")
                .unwrap(),
            config_request().launch_only(),
            config_request().with_output_limit(2 * 1024 * 1024),
        ] {
            assert!(validate_private_config(&req).is_err());
        }
        let out = PrivateConfigOutput(CommandOutput {
            command: "private".into(),
            stdout: CapturedOutput {
                text: json!({"secret":"synthetic-opaque"}).to_string(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: empty_output(),
            attempts: 1,
            duration: Duration::ZERO,
        });
        assert_eq!(out.json().unwrap()["secret"], "synthetic-opaque");
        assert!(!format!("{out:?}").contains("synthetic-opaque"));
    }
    #[tokio::test]
    async fn private_config_failure_hides_both_output_streams() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("fake-rclone");
        std::fs::write(&file,"#!/bin/sh\nprintf 'synthetic-stdout-secret'\nprintf 'synthetic-stderr-secret' >&2\nexit 7\n").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = run_with_retries(
            file,
            config_request(),
            CancellationToken::new(),
            CapturePolicy::PrivateConfig,
        )
        .await
        .unwrap_err();
        let message = format!("{error:?}");
        assert!(!message.contains("synthetic-stdout-secret"));
        assert!(!message.contains("synthetic-stderr-secret"));
        assert!(matches!(error, CommandError::NonZero { code: Some(7), .. }));
    }
    #[tokio::test]
    #[ignore = "requires installed rclone; isolated synthetic config only, no network or user configuration"]
    async fn private_config_real_encoded_token_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let config = tmp.path().join("rclone.conf");
        // '=' inside an encoded token demonstrates why the display sanitizer
        // must never be used as the input to a credential JSON decoder.
        let token=json!({"access_token":"synthetic-private-access=encoded","refresh_token":"synthetic-private-refresh=encoded"}).to_string();
        std::fs::write(&config,format!("[probe]\ntype = onedrive\ndrive_type = personal\ndrive_id = synthetic-drive\ntoken = {token}\n")).unwrap();
        let request = config_request()
            .arg("--config")
            .unwrap()
            .arg(config.as_os_str())
            .unwrap();
        let ordinary = SystemCommandRunner
            .run(request.clone(), CancellationToken::new())
            .await
            .unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&ordinary.stdout.text).is_err());
        let private = SystemCommandRunner
            .private_config(request, CancellationToken::new())
            .await
            .unwrap();
        assert!(!format!("{private:?}").contains("synthetic-private-access"));
        let value = private.json().unwrap();
        assert_eq!(
            crate::sharepoint::oauth_access_token(&value.to_string(), "probe").unwrap(),
            "synthetic-private-access=encoded"
        );
    }
}
