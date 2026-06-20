// SPDX-License-Identifier: MIT

//! Applet-owned systemd user service and timer management.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tokio_util::sync::CancellationToken;

use crate::model::ConnectionId;
use crate::process::{CommandError, CommandRequest, CommandRunner, Executable};

const MANAGED_MARKER: &str = "# X-Cosmic-Mounter-Managed=true";
const UUID_MARKER: &str = "# X-Cosmic-Mounter-Connection=";

pub type ServiceFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ServiceError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Service,
    Timer,
}

impl UnitKind {
    const fn extension(self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Timer => "timer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitName {
    connection_id: ConnectionId,
    kind: UnitKind,
}

impl UnitName {
    #[must_use]
    pub const fn new(connection_id: ConnectionId, kind: UnitKind) -> Self {
        Self {
            connection_id,
            kind,
        }
    }

    #[must_use]
    pub fn file_name(&self) -> String {
        format!(
            "cosmic-mounter-{}.{}",
            self.connection_id,
            self.kind.extension()
        )
    }

    #[must_use]
    pub const fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    pub connection_id: ConnectionId,
    pub description: String,
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub restart_on_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerSpec {
    pub connection_id: ConnectionId,
    pub description: String,
    pub interval: Duration,
    pub persistent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDocument {
    pub name: UnitName,
    pub content: String,
}

impl UnitDocument {
    pub fn service(spec: &ServiceSpec) -> Result<Self, ServiceError> {
        validate_text(&spec.description)?;
        let executable = validate_absolute_executable(&spec.executable)?;
        let arguments = spec
            .arguments
            .iter()
            .map(|argument| escape_systemd_argument(argument))
            .collect::<Result<Vec<_>, _>>()?;
        let mut exec = escape_systemd_argument(executable)?;
        if !arguments.is_empty() {
            exec.push(' ');
            exec.push_str(&arguments.join(" "));
        }

        let restart = if spec.restart_on_failure {
            "Restart=on-failure\nRestartSec=5s\n"
        } else {
            ""
        };
        let content = format!(
            "{MANAGED_MARKER}\n{UUID_MARKER}{}\n\n[Unit]\nDescription={}\n\n[Service]\nType=simple\nRuntimeDirectory=cosmic-ext-applet-mounter\nSuccessExitStatus=130 143\nExecStart={exec}\n{restart}\n[Install]\nWantedBy=default.target\n",
            spec.connection_id, spec.description
        );
        Ok(Self {
            name: UnitName::new(spec.connection_id, UnitKind::Service),
            content,
        })
    }

    pub fn timer(spec: &TimerSpec) -> Result<Self, ServiceError> {
        validate_text(&spec.description)?;
        if spec.interval.is_zero() {
            return Err(ServiceError::InvalidSpec(
                "timer interval must be greater than zero".into(),
            ));
        }
        let service = UnitName::new(spec.connection_id, UnitKind::Service).file_name();
        let persistent = if spec.persistent { "true" } else { "false" };
        let content = format!(
            "{MANAGED_MARKER}\n{UUID_MARKER}{}\n\n[Unit]\nDescription={}\n\n[Timer]\nOnUnitInactiveSec={}s\nPersistent={persistent}\nUnit={service}\n\n[Install]\nWantedBy=timers.target\n",
            spec.connection_id,
            spec.description,
            spec.interval.as_secs()
        );
        Ok(Self {
            name: UnitName::new(spec.connection_id, UnitKind::Timer),
            content,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitFileState {
    Missing,
    External,
    Managed(ConnectionId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveState {
    Active,
    Activating,
    Inactive,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitStatus {
    pub active: ActiveState,
    pub enabled: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemdAction {
    Verify,
    DaemonReload,
    Enable,
    Disable,
    Start,
    Stop,
    ResetFailed,
    Status,
}

#[derive(Debug)]
pub enum ServiceError {
    InvalidSpec(String),
    Io(std::io::Error),
    OwnershipMismatch { path: PathBuf },
    Validation(String),
    Runtime(String),
    Command(CommandError),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpec(message) | Self::Validation(message) | Self::Runtime(message) => {
                formatter.write_str(message)
            }
            Self::Io(error) => error.fmt(formatter),
            Self::OwnershipMismatch { path } => {
                write!(
                    formatter,
                    "unit is not owned by this applet: {}",
                    path.display()
                )
            }
            Self::Command(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<CommandError> for ServiceError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

pub trait UnitValidator: Send + Sync {
    fn validate(&self, document: &UnitDocument) -> Result<(), ServiceError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StructuralUnitValidator;

impl UnitValidator for StructuralUnitValidator {
    fn validate(&self, document: &UnitDocument) -> Result<(), ServiceError> {
        if !document.content.starts_with(MANAGED_MARKER)
            || !document
                .content
                .contains(&format!("{UUID_MARKER}{}", document.name.connection_id()))
            || !document.content.contains("[Unit]")
        {
            return Err(ServiceError::Validation(
                "generated unit is missing required structure or ownership markers".into(),
            ));
        }
        match document.name.kind {
            UnitKind::Service if !document.content.contains("[Service]") => Err(
                ServiceError::Validation("service document lacks [Service]".into()),
            ),
            UnitKind::Timer if !document.content.contains("[Timer]") => Err(
                ServiceError::Validation("timer document lacks [Timer]".into()),
            ),
            _ => Ok(()),
        }
    }
}

pub trait UnitStore: Send + Sync {
    fn state(&self, name: &UnitName) -> Result<UnitFileState, ServiceError>;
    fn read(&self, name: &UnitName) -> Result<Option<String>, ServiceError>;
    fn write(&self, document: &UnitDocument) -> Result<(), ServiceError>;
    fn remove(&self, name: &UnitName) -> Result<(), ServiceError>;
}

#[derive(Clone)]
pub struct FileUnitStore {
    root: PathBuf,
    validator: Arc<dyn UnitValidator>,
}

impl FileUnitStore {
    #[must_use]
    pub fn new(root: PathBuf, validator: Arc<dyn UnitValidator>) -> Self {
        Self { root, validator }
    }

    pub fn user(validator: Arc<dyn UnitValidator>) -> Result<Self, ServiceError> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| ServiceError::Runtime("HOME is unavailable".into()))?;
        Ok(Self::new(
            PathBuf::from(home).join(".config/systemd/user"),
            validator,
        ))
    }

    fn path(&self, name: &UnitName) -> PathBuf {
        self.root.join(name.file_name())
    }

    fn write_raw(&self, path: &Path, content: &str) -> Result<(), ServiceError> {
        fs::create_dir_all(&self.root)?;
        let temporary = self.root.join(format!(
            ".{}.tmp-{}",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unit"),
            std::process::id()
        ));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, path)?;
            if let Ok(directory) = fs::File::open(&self.root) {
                let _ = directory.sync_all();
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

impl UnitStore for FileUnitStore {
    fn state(&self, name: &UnitName) -> Result<UnitFileState, ServiceError> {
        let path = self.path(name);
        let Some(content) = fs::read_to_string(&path).map(Some).or_else(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(error)
            }
        })?
        else {
            return Ok(UnitFileState::Missing);
        };
        Ok(parse_owned_connection(&content)
            .map(UnitFileState::Managed)
            .unwrap_or(UnitFileState::External))
    }

    fn read(&self, name: &UnitName) -> Result<Option<String>, ServiceError> {
        let path = self.path(name);
        match fs::read_to_string(path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn write(&self, document: &UnitDocument) -> Result<(), ServiceError> {
        self.validator.validate(document)?;
        let path = self.path(&document.name);
        match self.state(&document.name)? {
            UnitFileState::External => {
                return Err(ServiceError::OwnershipMismatch { path });
            }
            UnitFileState::Managed(id) if id != document.name.connection_id => {
                return Err(ServiceError::OwnershipMismatch { path });
            }
            UnitFileState::Missing | UnitFileState::Managed(_) => {}
        }
        self.write_raw(&path, &document.content)
    }

    fn remove(&self, name: &UnitName) -> Result<(), ServiceError> {
        let path = self.path(name);
        match self.state(name)? {
            UnitFileState::Missing => Ok(()),
            UnitFileState::Managed(id) if id == name.connection_id => {
                fs::remove_file(path)?;
                Ok(())
            }
            UnitFileState::External | UnitFileState::Managed(_) => {
                Err(ServiceError::OwnershipMismatch { path })
            }
        }
    }
}

pub trait SystemdManager: Send + Sync {
    fn action<'a>(
        &'a self,
        action: SystemdAction,
        unit: Option<&'a UnitName>,
        cancellation: CancellationToken,
    ) -> ServiceFuture<'a, Option<UnitStatus>>;
}

pub struct CommandSystemdManager<R> {
    runner: R,
}

impl<R> CommandSystemdManager<R> {
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: CommandRunner> SystemdManager for CommandSystemdManager<R> {
    fn action<'a>(
        &'a self,
        action: SystemdAction,
        unit: Option<&'a UnitName>,
        cancellation: CancellationToken,
    ) -> ServiceFuture<'a, Option<UnitStatus>> {
        Box::pin(async move {
            let command = match action {
                SystemdAction::Verify => {
                    let unit = unit.ok_or_else(|| {
                        ServiceError::InvalidSpec("verify requires a unit".into())
                    })?;
                    CommandRequest::new(Executable::SystemdAnalyze)
                        .arg("--user")?
                        .arg("verify")?
                        .arg(unit.file_name())?
                }
                SystemdAction::DaemonReload => CommandRequest::new(Executable::Systemctl)
                    .arg("--user")?
                    .arg("daemon-reload")?,
                SystemdAction::Status => {
                    let unit = unit.ok_or_else(|| {
                        ServiceError::InvalidSpec("status requires a unit".into())
                    })?;
                    CommandRequest::new(Executable::Systemctl)
                        .arg("--user")?
                        .arg("show")?
                        .arg("--property=ActiveState")?
                        .arg("--property=UnitFileState")?
                        .arg("--property=SubState")?
                        .arg(unit.file_name())?
                }
                _ => {
                    let unit = unit.ok_or_else(|| {
                        ServiceError::InvalidSpec("systemd action requires a unit".into())
                    })?;
                    CommandRequest::new(Executable::Systemctl)
                        .arg("--user")?
                        .arg(match action {
                            SystemdAction::Enable => "enable",
                            SystemdAction::Disable => "disable",
                            SystemdAction::Start => "start",
                            SystemdAction::Stop => "stop",
                            SystemdAction::ResetFailed => "reset-failed",
                            SystemdAction::Verify
                            | SystemdAction::DaemonReload
                            | SystemdAction::Status => unreachable!(),
                        })?
                        .arg(unit.file_name())?
                }
            }
            .with_timeout(Duration::from_secs(15));

            let output = self
                .runner
                .run(command, cancellation)
                .await
                .map_err(ServiceError::Command)?;
            if action == SystemdAction::Status {
                Ok(Some(parse_systemd_status(&output.stdout.text)))
            } else {
                Ok(None)
            }
        })
    }
}

pub struct UnitController<S, M> {
    store: S,
    manager: M,
}

impl<S, M> UnitController<S, M> {
    #[must_use]
    pub const fn new(store: S, manager: M) -> Self {
        Self { store, manager }
    }
}

impl<S: UnitStore, M: SystemdManager> UnitController<S, M> {
    pub async fn install(
        &self,
        document: &UnitDocument,
        cancellation: CancellationToken,
    ) -> Result<(), ServiceError> {
        let previous = self.store.read(&document.name)?;
        self.store.write(document)?;
        if let Err(error) = self
            .manager
            .action(
                SystemdAction::Verify,
                Some(&document.name),
                cancellation.child_token(),
            )
            .await
        {
            self.restore(&document.name, previous)?;
            return Err(error);
        }
        if let Err(error) = self
            .manager
            .action(SystemdAction::DaemonReload, None, cancellation)
            .await
        {
            self.restore(&document.name, previous)?;
            return Err(error);
        }
        Ok(())
    }

    pub async fn remove(
        &self,
        name: &UnitName,
        cancellation: CancellationToken,
    ) -> Result<(), ServiceError> {
        let previous = self.store.read(name)?;
        self.store.remove(name)?;
        if let Err(error) = self
            .manager
            .action(SystemdAction::DaemonReload, None, cancellation)
            .await
        {
            if let Some(content) = previous {
                self.restore(name, Some(content))?;
            }
            return Err(error);
        }
        Ok(())
    }

    fn restore(&self, name: &UnitName, previous: Option<String>) -> Result<(), ServiceError> {
        match previous {
            Some(content) => self.store.write(&UnitDocument {
                name: name.clone(),
                content,
            }),
            None => self.store.remove(name),
        }
    }
}

#[derive(Default)]
pub struct ConnectionOperationLocks {
    locks: Mutex<HashMap<ConnectionId, Arc<AsyncMutex<()>>>>,
}

impl ConnectionOperationLocks {
    pub async fn acquire(&self, connection_id: ConnectionId) -> OwnedMutexGuard<()> {
        let lock = self
            .locks
            .lock()
            .expect("operation lock map")
            .entry(connection_id)
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone();
        lock.lock_owned().await
    }
}

#[derive(Clone, Default)]
pub struct FakeSystemdManager {
    actions: Arc<Mutex<SystemdActionLog>>,
    results: Arc<Mutex<SystemdResultQueue>>,
}

type SystemdActionLog = Vec<(SystemdAction, Option<String>)>;
type SystemdResultQueue = VecDeque<Result<Option<UnitStatus>, ServiceError>>;

impl FakeSystemdManager {
    pub fn push(&self, result: Result<Option<UnitStatus>, ServiceError>) {
        self.results
            .lock()
            .expect("fake systemd results")
            .push_back(result);
    }

    #[must_use]
    pub fn actions(&self) -> Vec<(SystemdAction, Option<String>)> {
        self.actions.lock().expect("fake systemd actions").clone()
    }
}

impl SystemdManager for FakeSystemdManager {
    fn action<'a>(
        &'a self,
        action: SystemdAction,
        unit: Option<&'a UnitName>,
        _cancellation: CancellationToken,
    ) -> ServiceFuture<'a, Option<UnitStatus>> {
        Box::pin(async move {
            self.actions
                .lock()
                .expect("fake systemd actions")
                .push((action, unit.map(UnitName::file_name)));
            self.results
                .lock()
                .expect("fake systemd results")
                .pop_front()
                .unwrap_or(Ok(None))
        })
    }
}

fn validate_text(value: &str) -> Result<(), ServiceError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(ServiceError::InvalidSpec(
            "unit text is empty or contains control characters".into(),
        ))
    } else {
        Ok(())
    }
}

fn validate_absolute_executable(path: &Path) -> Result<&str, ServiceError> {
    if !path.is_absolute() {
        return Err(ServiceError::InvalidSpec(
            "unit executable must be absolute".into(),
        ));
    }
    path.to_str()
        .ok_or_else(|| ServiceError::InvalidSpec("unit executable is not UTF-8".into()))
}

fn escape_systemd_argument(value: &str) -> Result<String, ServiceError> {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character == '\0' || character == '\n')
    {
        return Err(ServiceError::InvalidSpec(
            "unit argument is empty or contains unsafe characters".into(),
        ));
    }
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("\"{escaped}\""))
}

fn parse_owned_connection(content: &str) -> Option<ConnectionId> {
    if !content.lines().any(|line| line == MANAGED_MARKER) {
        return None;
    }
    content
        .lines()
        .find_map(|line| line.strip_prefix(UUID_MARKER))
        .and_then(|uuid| uuid::Uuid::parse_str(uuid).ok())
        .map(ConnectionId::from_uuid)
}

fn parse_systemd_status(output: &str) -> UnitStatus {
    let property = |name: &str| {
        output
            .lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix('='))
            .unwrap_or_default()
    };
    let active = match property("ActiveState") {
        "active" => ActiveState::Active,
        "activating" => ActiveState::Activating,
        "inactive" => ActiveState::Inactive,
        "failed" => ActiveState::Failed,
        _ => ActiveState::Unknown,
    };
    let enabled = matches!(
        property("UnitFileState"),
        "enabled" | "enabled-runtime" | "linked" | "linked-runtime"
    );
    UnitStatus {
        active,
        enabled,
        detail: property("SubState").to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;

    fn id() -> ConnectionId {
        ConnectionId::from_uuid(
            Uuid::parse_str("2a3f5d45-e867-47e7-943f-66cf60e777ad").expect("UUID"),
        )
    }

    fn service() -> UnitDocument {
        UnitDocument::service(&ServiceSpec {
            connection_id: id(),
            description: "Example storage connection".into(),
            executable: PathBuf::from("/usr/bin/example"),
            arguments: vec!["--path".into(), "/home/example/Cloud Drive".into()],
            restart_on_failure: true,
        })
        .expect("service")
    }

    fn command_output(stdout: &str) -> CommandOutput {
        CommandOutput {
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

    #[test]
    fn service_snapshot_is_deterministic() {
        assert_eq!(
            service().content,
            "# X-Cosmic-Mounter-Managed=true\n\
# X-Cosmic-Mounter-Connection=2a3f5d45-e867-47e7-943f-66cf60e777ad\n\n\
[Unit]\nDescription=Example storage connection\n\n\
[Service]\nType=simple\n\
RuntimeDirectory=cosmic-ext-applet-mounter\n\
SuccessExitStatus=130 143\n\
ExecStart=\"/usr/bin/example\" \"--path\" \"/home/example/Cloud Drive\"\n\
Restart=on-failure\nRestartSec=5s\n\
\n\
[Install]\nWantedBy=default.target\n"
        );
    }

    #[test]
    fn timer_snapshot_is_deterministic() {
        let timer = UnitDocument::timer(&TimerSpec {
            connection_id: id(),
            description: "Example storage schedule".into(),
            interval: Duration::from_secs(900),
            persistent: true,
        })
        .expect("timer");
        assert_eq!(
            timer.content,
            "# X-Cosmic-Mounter-Managed=true\n\
# X-Cosmic-Mounter-Connection=2a3f5d45-e867-47e7-943f-66cf60e777ad\n\n\
[Unit]\nDescription=Example storage schedule\n\n\
[Timer]\nOnUnitInactiveSec=900s\nPersistent=true\n\
Unit=cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service\n\n\
[Install]\nWantedBy=timers.target\n"
        );
    }

    #[test]
    fn external_unit_cannot_be_replaced_or_removed() {
        let temp = TempDir::new().expect("temp");
        let store = FileUnitStore::new(temp.path().into(), Arc::new(StructuralUnitValidator));
        let document = service();
        fs::write(
            temp.path().join(document.name.file_name()),
            "[Unit]\nDescription=External\n",
        )
        .expect("external unit");
        assert!(matches!(
            store.write(&document),
            Err(ServiceError::OwnershipMismatch { .. })
        ));
        assert!(matches!(
            store.remove(&document.name),
            Err(ServiceError::OwnershipMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn reload_failure_restores_previous_unit() {
        let temp = TempDir::new().expect("temp");
        let store = FileUnitStore::new(temp.path().into(), Arc::new(StructuralUnitValidator));
        let first = service();
        store.write(&first).expect("initial unit");
        let mut second = first.clone();
        second.content = second.content.replace("Example storage", "Updated storage");
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Err(ServiceError::Runtime("reload failed".into())));
        let controller = UnitController::new(store.clone(), manager);

        assert!(
            controller
                .install(&second, CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(
            store.read(&first.name).expect("read restored"),
            Some(first.content)
        );
    }

    #[tokio::test]
    async fn verification_failure_removes_new_unit() {
        let temp = TempDir::new().expect("temp");
        let store = FileUnitStore::new(temp.path().into(), Arc::new(StructuralUnitValidator));
        let document = service();
        let manager = FakeSystemdManager::default();
        manager.push(Err(ServiceError::Runtime("verification failed".into())));
        let controller = UnitController::new(store.clone(), manager);

        assert!(
            controller
                .install(&document, CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(store.read(&document.name).expect("read restored"), None);
    }

    #[tokio::test]
    async fn removing_managed_unit_preserves_user_data_and_originals() {
        let temp = TempDir::new().expect("temp");
        let units = temp.path().join("units");
        let data = temp.path().join("data");
        fs::create_dir_all(&units).expect("units");
        fs::create_dir_all(&data).expect("data");
        let credential_file = data.join("rclone.conf");
        let cache_file = data.join("cache.bin");
        let recovery_file = data.join("recovery.txt");
        let original_file = data.join("legacy.service");
        fs::write(&credential_file, "secret").expect("credential");
        fs::write(&cache_file, "cache").expect("cache");
        fs::write(&recovery_file, "recovery").expect("recovery");
        fs::write(&original_file, "original").expect("original");

        let store = FileUnitStore::new(units, Arc::new(StructuralUnitValidator));
        let document = service();
        store.write(&document).expect("managed unit");
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        let controller = UnitController::new(store.clone(), manager);

        controller
            .remove(&document.name, CancellationToken::new())
            .await
            .expect("remove managed unit");

        assert_eq!(store.read(&document.name).expect("managed removed"), None);
        assert_eq!(
            fs::read_to_string(credential_file).expect("credential"),
            "secret"
        );
        assert_eq!(fs::read_to_string(cache_file).expect("cache"), "cache");
        assert_eq!(
            fs::read_to_string(recovery_file).expect("recovery"),
            "recovery"
        );
        assert_eq!(
            fs::read_to_string(original_file).expect("original"),
            "original"
        );
    }

    #[tokio::test]
    async fn command_manager_constructs_typed_actions_and_parses_status() {
        let runner = FakeCommandRunner::default();
        for _ in 0..7 {
            runner.push(Ok(command_output("")));
        }
        runner.push(Ok(command_output(
            "SubState=running\nUnitFileState=enabled\nActiveState=active\n",
        )));
        let manager = CommandSystemdManager::new(runner.clone());
        let unit = service().name;

        manager
            .action(SystemdAction::Verify, Some(&unit), CancellationToken::new())
            .await
            .expect("verify");
        manager
            .action(SystemdAction::DaemonReload, None, CancellationToken::new())
            .await
            .expect("reload");
        for action in [
            SystemdAction::Enable,
            SystemdAction::Disable,
            SystemdAction::Start,
            SystemdAction::Stop,
            SystemdAction::ResetFailed,
        ] {
            manager
                .action(action, Some(&unit), CancellationToken::new())
                .await
                .expect("typed action");
        }
        let status = manager
            .action(SystemdAction::Status, Some(&unit), CancellationToken::new())
            .await
            .expect("status")
            .expect("status value");

        assert_eq!(
            status,
            UnitStatus {
                active: ActiveState::Active,
                enabled: true,
                detail: "running".into(),
            }
        );
        assert_eq!(
            runner
                .requests()
                .iter()
                .map(CommandRequest::sanitized_command)
                .collect::<Vec<_>>(),
            vec![
                "systemd-analyze --user verify cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user daemon-reload",
                "systemctl --user enable cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user disable cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user start cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user stop cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user reset-failed cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
                "systemctl --user show --property=ActiveState --property=UnitFileState --property=SubState cosmic-mounter-2a3f5d45-e867-47e7-943f-66cf60e777ad.service",
            ]
        );
    }

    #[tokio::test]
    async fn same_connection_operations_serialize() {
        let locks = Arc::new(ConnectionOperationLocks::default());
        let first = locks.acquire(id()).await;
        let second_locks = Arc::clone(&locks);
        let task = tokio::spawn(async move {
            let _guard = second_locks.acquire(id()).await;
        });
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        drop(first);
        task.await.expect("serialized task");
    }
}
