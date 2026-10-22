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

use cosmic::iced::futures::StreamExt;
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tokio_util::sync::CancellationToken;
use zbus::zvariant::OwnedObjectPath;

use crate::fl;
use crate::model::ConnectionId;

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
    pub pre_start_condition: Option<ServiceCommand>,
    pub restart_on_failure: bool,
    pub skip_on_metered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCommand {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
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

        let pre_start_condition = spec
            .pre_start_condition
            .as_ref()
            .map(|command| {
                let executable = validate_absolute_executable(&command.executable)?;
                let mut rendered = escape_systemd_argument(executable)?;
                for argument in &command.arguments {
                    rendered.push(' ');
                    rendered.push_str(&escape_systemd_argument(argument)?);
                }
                Ok::<_, ServiceError>(format!("ExecCondition={rendered}\n"))
            })
            .transpose()?
            .unwrap_or_default();
        let restart = if spec.restart_on_failure {
            "Restart=on-failure\nRestartSec=5s\n"
        } else {
            ""
        };
        let metered_condition = if spec.skip_on_metered {
            let command =
                crate::mount_guard::service_command(vec![crate::vpn::METERED_GUARD_FLAG.into()])
                    .map_err(ServiceError::InvalidSpec)?;
            let executable = validate_absolute_executable(&command.executable)?;
            let mut rendered = escape_systemd_argument(executable)?;
            for argument in &command.arguments {
                rendered.push(' ');
                rendered.push_str(&escape_systemd_argument(argument)?);
            }
            format!("ExecCondition={rendered}\n")
        } else {
            String::new()
        };
        let content = format!(
            "{MANAGED_MARKER}\n{UUID_MARKER}{}\n\n[Unit]\nDescription={}\n\n[Service]\nType=simple\nRuntimeDirectory=cosmic-ext-applet-mounter\nSuccessExitStatus=130 143\n{metered_condition}{pre_start_condition}ExecStart={exec}\n{restart}\n[Install]\nWantedBy=default.target\n",
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
            return Err(ServiceError::InvalidSpec(crate::fl!(
                "ui-services-timer-interval-must-be-greater-than-zero"
            )));
        }
        let service = UnitName::new(spec.connection_id, UnitKind::Service).file_name();
        let persistent = if spec.persistent { "true" } else { "false" };
        let content = format!(
            "{MANAGED_MARKER}\n{UUID_MARKER}{}\n\n[Unit]\nDescription={}\n\n[Timer]\nOnActiveSec={}s\nOnUnitInactiveSec={}s\nPersistent={persistent}\nUnit={service}\n\n[Install]\nWantedBy=timers.target\n",
            spec.connection_id,
            spec.description,
            spec.interval.as_secs(),
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
    pub result: String,
}

/// Absence is structured backend evidence, distinct from a failed observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitStatusObservation {
    Present(UnitStatus),
    Missing,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemdAction {
    DaemonReload,
    ValidateLoaded,
    ValidateRetirementLoaded,
    Enable,
    Disable,
    Start,
    Stop,
    Restart,
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
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpec(message) | Self::Validation(message) | Self::Runtime(message) => {
                formatter.write_str(message)
            }
            Self::Io(error) => error.fmt(formatter),
            Self::OwnershipMismatch { path } => formatter.write_str(&crate::fl!(
                "ui-services-unit-is-not-owned-by-this-applet",
                value1 = (path.display()).to_string()
            )),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait UnitValidator: Send + Sync {
    fn validate(&self, document: &UnitDocument) -> Result<(), ServiceError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StructuralUnitValidator;

impl UnitValidator for StructuralUnitValidator {
    fn validate(&self, document: &UnitDocument) -> Result<(), ServiceError> {
        let expected_uuid = format!("{UUID_MARKER}{}", document.name.connection_id());
        let mut lines = document.content.lines();
        if lines.next() != Some(MANAGED_MARKER) || lines.next() != Some(expected_uuid.as_str()) {
            return Err(ServiceError::Validation(crate::fl!(
                "ui-services-generated-unit-has-invalid-ownership-markers"
            )));
        }
        if document.content.contains(['\0', '\r']) {
            return Err(ServiceError::Validation(crate::fl!(
                "ui-services-generated-unit-contains-unsafe-control-characters"
            )));
        }

        let allowed_sections: &[&str] = match document.name.kind {
            UnitKind::Service => &["Unit", "Service", "Install"],
            UnitKind::Timer => &["Unit", "Timer", "Install"],
        };
        let mut section = None;
        let mut sections = HashMap::<String, usize>::new();
        let mut directives = HashMap::<(String, String), Vec<String>>::new();
        for line in lines {
            if line == MANAGED_MARKER || line.starts_with(UUID_MARKER) {
                return Err(ServiceError::Validation(crate::fl!(
                    "ui-services-generated-unit-repeats-ownership-markers"
                )));
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line
                .strip_prefix('[')
                .and_then(|line| line.strip_suffix(']'))
            {
                if !allowed_sections.contains(&name) {
                    return Err(ServiceError::Validation(crate::fl!(
                        "ui-services-generated-unit-contains-unsupported-name-section",
                        name = (name).to_string()
                    )));
                }
                if *sections.entry(name.to_owned()).or_default() != 0 {
                    return Err(ServiceError::Validation(crate::fl!(
                        "ui-services-generated-unit-repeats-name-section",
                        name = (name).to_string()
                    )));
                }
                sections.insert(name.to_owned(), 1);
                section = Some(name.to_owned());
                continue;
            }
            let current = section.as_deref().ok_or_else(|| {
                ServiceError::Validation(crate::fl!(
                    "ui-services-unit-directive-appears-outside-a-section"
                ))
            })?;
            let (key, value) = line.split_once('=').ok_or_else(|| {
                ServiceError::Validation(crate::fl!(
                    "ui-services-generated-unit-contains-a-malformed-directive"
                ))
            })?;
            let allowed = match current {
                "Unit" => matches!(key, "Description"),
                "Service" => matches!(
                    key,
                    "Type"
                        | "RuntimeDirectory"
                        | "SuccessExitStatus"
                        | "ExecCondition"
                        | "ExecStart"
                        | "Restart"
                        | "RestartSec"
                ),
                "Timer" => matches!(
                    key,
                    "OnActiveSec" | "OnUnitInactiveSec" | "Persistent" | "Unit"
                ),
                "Install" => key == "WantedBy",
                _ => false,
            };
            if !allowed || value.is_empty() {
                return Err(ServiceError::Validation(crate::fl!(
                    "ui-services-generated-unit-contains-unsupported-or-empty-current",
                    current = (current).to_string(),
                    key = (key).to_string()
                )));
            }
            directives
                .entry((current.to_owned(), key.to_owned()))
                .or_default()
                .push(value.to_owned());
        }
        for expected in allowed_sections {
            if sections.get(*expected) != Some(&1) {
                return Err(ServiceError::Validation(crate::fl!(
                    "ui-services-generated-unit-lacks-expected-section",
                    expected = (expected).to_string()
                )));
            }
        }

        let one = |section: &str, key: &str| -> Result<&str, ServiceError> {
            match directives.get(&(section.to_owned(), key.to_owned())) {
                Some(values) if values.len() == 1 => Ok(values[0].as_str()),
                _ => Err(ServiceError::Validation(crate::fl!(
                    "ui-services-generated-unit-requires-exactly-one-section-key",
                    section = (section).to_string(),
                    key = (key).to_string()
                ))),
            }
        };
        if one("Unit", "Description")?.chars().any(char::is_control) {
            return Err(ServiceError::Validation(crate::fl!(
                "ui-services-generated-unit-description-contains-control-characters"
            )));
        }
        match document.name.kind {
            UnitKind::Service => {
                if one("Service", "Type")? != "simple"
                    || one("Service", "RuntimeDirectory")? != "cosmic-ext-applet-mounter"
                    || one("Service", "SuccessExitStatus")? != "130 143"
                    || !one("Service", "ExecStart")?.starts_with("\"/")
                    || one("Install", "WantedBy")? != "default.target"
                {
                    return Err(ServiceError::Validation(crate::fl!(
                        "ui-services-generated-service-has-invalid-fixed-directives"
                    )));
                }
                let restart = directives.get(&("Service".into(), "Restart".into()));
                let restart_sec = directives.get(&("Service".into(), "RestartSec".into()));
                if !matches!((restart, restart_sec), (None, None))
                    && !matches!((restart, restart_sec), (Some(a), Some(b)) if a.as_slice() == ["on-failure"] && b.as_slice() == ["5s"])
                {
                    return Err(ServiceError::Validation(crate::fl!(
                        "ui-services-generated-service-has-inconsistent-restart-policy"
                    )));
                }
            }
            UnitKind::Timer => {
                for key in ["OnActiveSec", "OnUnitInactiveSec"] {
                    one("Timer", key)?
                        .strip_suffix('s')
                        .and_then(|value| value.parse::<u64>().ok())
                        .filter(|value| *value > 0)
                        .ok_or_else(|| {
                            ServiceError::Validation(crate::fl!(
                                "ui-services-generated-timer-has-invalid-key",
                                key = (key).to_string()
                            ))
                        })?;
                }
                let expected_service =
                    UnitName::new(document.name.connection_id(), UnitKind::Service).file_name();
                if one("Timer", "Unit")? != expected_service
                    || !matches!(one("Timer", "Persistent")?, "true" | "false")
                    || one("Install", "WantedBy")? != "timers.target"
                {
                    return Err(ServiceError::Validation(crate::fl!(
                        "ui-services-generated-timer-has-invalid-fixed-directives"
                    )));
                }
            }
        }
        Ok(())
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
            .ok_or_else(|| ServiceError::Runtime(crate::fl!("ui-services-home-is-unavailable")))?;
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
        match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err(ServiceError::OwnershipMismatch { path });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
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

#[derive(Debug, Default, Clone, Copy)]
pub struct DbusSystemdManager;

const SYSTEMD_DESTINATION: &str = "org.freedesktop.systemd1";
const SYSTEMD_PATH: &str = "/org/freedesktop/systemd1";
const SYSTEMD_MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
const SYSTEMD_UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";
const SYSTEMD_SERVICE_INTERFACE: &str = "org.freedesktop.systemd1.Service";

async fn systemd_user_connection() -> Result<zbus::Connection, ServiceError> {
    zbus::Connection::session().await.map_err(|error| {
        ServiceError::Runtime(crate::fl!(
            "ui-services-systemd-user-d-bus-error",
            error = (error).to_string()
        ))
    })
}

async fn systemd_user_manager(
    connection: &zbus::Connection,
) -> Result<zbus::Proxy<'_>, ServiceError> {
    zbus::Proxy::new(
        connection,
        SYSTEMD_DESTINATION,
        SYSTEMD_PATH,
        SYSTEMD_MANAGER_INTERFACE,
    )
    .await
    .map_err(|error| {
        ServiceError::Runtime(crate::fl!(
            "ui-services-systemd-user-d-bus-error",
            error = (error).to_string()
        ))
    })
}

async fn systemd_user_unit_path(
    connection: &zbus::Connection,
    unit: &str,
) -> Result<OwnedObjectPath, ServiceError> {
    systemd_user_manager(connection)
        .await?
        .call("LoadUnit", &(unit,))
        .await
        .map_err(|error| {
            ServiceError::Runtime(crate::fl!(
                "ui-services-systemd-unit-unit-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            ))
        })
}

async fn systemd_user_unit_proxy<'a>(
    connection: &'a zbus::Connection,
    path: &'a OwnedObjectPath,
    interface: &'static str,
) -> Result<zbus::Proxy<'a>, ServiceError> {
    zbus::Proxy::new(connection, SYSTEMD_DESTINATION, path.as_str(), interface)
        .await
        .map_err(|error| {
            ServiceError::Runtime(crate::fl!(
                "ui-services-systemd-user-d-bus-error",
                error = (error).to_string()
            ))
        })
}

async fn systemd_user_run_job(
    manager: &zbus::Proxy<'_>,
    method: &str,
    unit: &str,
) -> Result<(), ServiceError> {
    manager
        .call::<_, _, ()>("Subscribe", &())
        .await
        .map_err(|error| ServiceError::Runtime(error.to_string()))?;
    let mut jobs = manager
        .receive_signal("JobRemoved")
        .await
        .map_err(|error| ServiceError::Runtime(error.to_string()))?;
    let job: OwnedObjectPath = manager
        .call(method, &(unit, "replace"))
        .await
        .map_err(|error| ServiceError::Runtime(error.to_string()))?;

    while let Some(signal) = jobs.next().await {
        let (_, finished_job, finished_unit, result): (u32, OwnedObjectPath, String, String) =
            signal
                .body()
                .deserialize()
                .map_err(|error| ServiceError::Runtime(error.to_string()))?;
        if finished_job == job {
            return if result == "done" {
                Ok(())
            } else {
                Err(ServiceError::Runtime(crate::fl!(
                    "ui-services-systemd-method-job-for-finished-unit-finished",
                    method = (method).to_string(),
                    finished_unit = (finished_unit).to_string(),
                    result = (result).to_string()
                )))
            };
        }
    }
    Err(ServiceError::Runtime(crate::fl!(
        "ui-services-systemd-method-job-for-unit-ended-without",
        method = (method).to_string(),
        unit = (unit).to_string()
    )))
}

pub async fn systemd_user_reload() -> Result<(), String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_manager(&connection)
        .await
        .map_err(|error| error.to_string())?
        .call("Reload", &())
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-user-reload-error",
                error = (error).to_string()
            )
        })
}

pub async fn systemd_user_available() -> Result<(), String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_manager(&connection)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn valid_systemd_service_name(unit: &str) -> bool {
    !unit.is_empty()
        && unit.ends_with(".service")
        && !unit
            .chars()
            .any(|character| character.is_whitespace() || character == '/')
}

pub async fn systemd_user_start_named_unit(unit: &str) -> Result<(), String> {
    if !valid_systemd_service_name(unit) {
        return Err(crate::fl!(
            "ui-services-invalid-systemd-user-service-name-unit",
            unit = (unit).to_string()
        ));
    }
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    let manager = systemd_user_manager(&connection)
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_run_job(&manager, "StartUnit", unit)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-start-unit-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            )
        })
}

pub async fn systemd_user_restart_named_unit(unit: &str) -> Result<(), String> {
    if !valid_systemd_service_name(unit) {
        return Err(crate::fl!(
            "ui-services-invalid-systemd-user-service-name-unit",
            unit = (unit).to_string()
        ));
    }
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    let manager = systemd_user_manager(&connection)
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_run_job(&manager, "RestartUnit", unit)
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-restart-unit-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            )
        })
}

pub async fn systemd_user_active_state(unit: &str) -> Result<String, String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    let path = systemd_user_unit_path(&connection, unit)
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_unit_proxy(&connection, &path, SYSTEMD_UNIT_INTERFACE)
        .await
        .map_err(|error| error.to_string())?
        .get_property("ActiveState")
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-unit-unit-state-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            )
        })
}

pub async fn systemd_user_stop_no_wait(unit: &str) -> Result<(), String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    systemd_user_manager(&connection)
        .await
        .map_err(|error| error.to_string())?
        .call::<_, _, OwnedObjectPath>("StopUnit", &(unit, "replace"))
        .await
        .map(|_| ())
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-stop-unit-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            )
        })
}

pub async fn systemd_user_stop_policy(unit: &str) -> Result<(bool, u64, String, i32), String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    let path = systemd_user_unit_path(&connection, unit)
        .await
        .map_err(|error| error.to_string())?;
    let proxy = systemd_user_unit_proxy(&connection, &path, SYSTEMD_SERVICE_INTERFACE)
        .await
        .map_err(|error| error.to_string())?;
    let send_sigkill = proxy
        .get_property("SendSIGKILL")
        .await
        .map_err(|error| error.to_string())?;
    let timeout_stop_usec = proxy
        .get_property("TimeoutStopUSec")
        .await
        .map_err(|error| error.to_string())?;
    let timeout_stop_failure_mode = proxy
        .get_property("TimeoutStopFailureMode")
        .await
        .map_err(|error| error.to_string())?;
    let kill_signal = proxy
        .get_property("KillSignal")
        .await
        .map_err(|error| error.to_string())?;
    Ok((
        send_sigkill,
        timeout_stop_usec,
        timeout_stop_failure_mode,
        kill_signal,
    ))
}

pub async fn systemd_user_invocation_id(unit: &str) -> Result<String, String> {
    let connection = systemd_user_connection()
        .await
        .map_err(|error| error.to_string())?;
    let path = systemd_user_unit_path(&connection, unit)
        .await
        .map_err(|error| error.to_string())?;
    let bytes: Vec<u8> = systemd_user_unit_proxy(&connection, &path, SYSTEMD_UNIT_INTERFACE)
        .await
        .map_err(|error| error.to_string())?
        .get_property("InvocationID")
        .await
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-unit-unit-invocation-error",
                unit = (unit).to_string(),
                error = (error).to_string()
            )
        })?;
    if bytes.len() != 16 {
        return Err(crate::fl!(
            "ui-services-systemd-unit-unit-has-no-current-invocation",
            unit = (unit).to_string()
        ));
    }
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn systemd_user_status_blocking(unit: &UnitName) -> Result<UnitStatus, String> {
    let unit = unit.clone();
    std::thread::Builder::new()
        .name("systemd-user-status".into())
        .spawn(move || systemd_user_status_on_blocking_thread(&unit))
        .map_err(|error| {
            crate::fl!(
                "ui-services-could-not-start-systemd-status-worker-error",
                error = (error).to_string()
            )
        })?
        .join()
        .map_err(|_| crate::fl!("ui-services-systemd-status-worker-panicked"))?
}

/// Observe app-owned units asynchronously over one connection. Failed reads
/// remain unknown, never synthesized inactive; the caller retains last-known data.
pub async fn systemd_user_status_batch(
    units: &[UnitName],
    cancellation: CancellationToken,
) -> Result<Vec<(UnitName, UnitStatusObservation)>, String> {
    if cancellation.is_cancelled() {
        return Err(crate::fl!("unit-status-refresh-unavailable"));
    }
    if units.is_empty() {
        return Ok(Vec::new());
    }
    let operation = async {
        let connection = systemd_user_connection()
            .await
            .map_err(|error| error.to_string())?;
        let manager = systemd_user_manager(&connection)
            .await
            .map_err(|error| error.to_string())?;
        let mut observations = Vec::with_capacity(units.len());
        for unit in units {
            let read = async {
                let name = unit.file_name();
                let path: OwnedObjectPath = match manager.call("LoadUnit", &(name.as_str(),)).await
                {
                    Ok(path) => path,
                    Err(error) if unit_load_reports_missing(&error) => {
                        return Ok(UnitStatusObservation::Missing);
                    }
                    Err(error) => return Err(ServiceError::Runtime(error.to_string())),
                };
                let proxy =
                    systemd_user_unit_proxy(&connection, &path, SYSTEMD_UNIT_INTERFACE).await?;
                let active: String = proxy
                    .get_property("ActiveState")
                    .await
                    .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                let detail: String = proxy
                    .get_property("SubState")
                    .await
                    .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                let enabled = observed_unit_file_enabled(
                    manager.call("GetUnitFileState", &(name.as_str(),)).await,
                )?;
                let result = if unit.kind == UnitKind::Service {
                    systemd_user_unit_proxy(&connection, &path, SYSTEMD_SERVICE_INTERFACE)
                        .await?
                        .get_property("Result")
                        .await
                        .map_err(|error| ServiceError::Runtime(error.to_string()))?
                } else {
                    String::new()
                };
                let active = parse_active_state(&active);
                if active == ActiveState::Unknown {
                    return Err(ServiceError::Runtime(crate::fl!(
                        "unit-status-refresh-unavailable"
                    )));
                }
                Ok::<_, ServiceError>(UnitStatusObservation::Present(UnitStatus {
                    active,
                    enabled,
                    detail,
                    result,
                }))
            };
            let status = tokio::time::timeout(Duration::from_secs(2), read)
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or(UnitStatusObservation::Unavailable);
            observations.push((unit.clone(), status));
        }
        Ok(observations)
    };
    bounded_unit_observation(operation, Duration::from_secs(5), cancellation).await
}

fn unit_load_reports_missing(error: &zbus::Error) -> bool {
    matches!(error, zbus::Error::MethodError(name, _, _)
        if name.as_str() == "org.freedesktop.systemd1.NoSuchUnit")
}

fn observed_unit_file_enabled(result: Result<String, zbus::Error>) -> Result<bool, ServiceError> {
    match result {
        Ok(state) => Ok(matches!(
            state.as_str(),
            "enabled" | "enabled-runtime" | "linked" | "linked-runtime"
        )),
        // A loaded service can remain active after its file is removed. Keep
        // its observed activity visible instead of reporting it as absent.
        Err(zbus::Error::MethodError(name, _, _))
            if name.as_str() == "org.freedesktop.systemd1.NoSuchUnitFile" =>
        {
            Ok(false)
        }
        Err(error) => Err(ServiceError::Runtime(error.to_string())),
    }
}

async fn bounded_unit_observation<T>(
    operation: impl Future<Output = Result<T, String>>,
    timeout: Duration,
    cancellation: CancellationToken,
) -> Result<T, String> {
    tokio::select! {
        () = cancellation.cancelled() => Err(crate::fl!("unit-status-refresh-unavailable")),
        result = tokio::time::timeout(timeout, operation) => {
            result.unwrap_or_else(|_| Err(crate::fl!("unit-status-refresh-unavailable")))
        }
    }
}

fn systemd_user_status_on_blocking_thread(unit: &UnitName) -> Result<UnitStatus, String> {
    let connection = zbus::blocking::Connection::session().map_err(|error| {
        crate::fl!(
            "ui-services-systemd-user-d-bus-error",
            error = (error).to_string()
        )
    })?;
    let manager = zbus::blocking::Proxy::new(
        &connection,
        SYSTEMD_DESTINATION,
        SYSTEMD_PATH,
        SYSTEMD_MANAGER_INTERFACE,
    )
    .map_err(|error| {
        crate::fl!(
            "ui-services-systemd-user-d-bus-error",
            error = (error).to_string()
        )
    })?;
    let name = unit.file_name();
    let path: OwnedObjectPath = manager
        .call("LoadUnit", &(name.as_str(),))
        .map_err(|error| {
            crate::fl!(
                "ui-services-systemd-unit-name-error",
                name = (name).to_string(),
                error = (error).to_string()
            )
        })?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        SYSTEMD_DESTINATION,
        path.as_str(),
        SYSTEMD_UNIT_INTERFACE,
    )
    .map_err(|error| {
        crate::fl!(
            "ui-services-systemd-user-d-bus-error",
            error = (error).to_string()
        )
    })?;
    let active: String = proxy
        .get_property("ActiveState")
        .map_err(|error| error.to_string())?;
    let detail: String = proxy
        .get_property("SubState")
        .map_err(|error| error.to_string())?;
    let enabled: String = manager
        .call("GetUnitFileState", &(name.as_str(),))
        .map_err(|error| error.to_string())?;
    let result = if unit.kind == UnitKind::Service {
        zbus::blocking::Proxy::new(
            &connection,
            SYSTEMD_DESTINATION,
            path.as_str(),
            SYSTEMD_SERVICE_INTERFACE,
        )
        .map_err(|error| error.to_string())?
        .get_property("Result")
        .map_err(|error| error.to_string())?
    } else {
        String::new()
    };
    Ok(UnitStatus {
        active: parse_active_state(&active),
        enabled: matches!(
            enabled.as_str(),
            "enabled" | "enabled-runtime" | "linked" | "linked-runtime"
        ),
        detail,
        result,
    })
}

impl SystemdManager for DbusSystemdManager {
    fn action<'a>(
        &'a self,
        action: SystemdAction,
        unit: Option<&'a UnitName>,
        cancellation: CancellationToken,
    ) -> ServiceFuture<'a, Option<UnitStatus>> {
        Box::pin(async move {
            let operation = async {
                let connection = systemd_user_connection().await?;
                let manager = systemd_user_manager(&connection).await?;
                match action {
                    SystemdAction::DaemonReload => {
                        manager
                            .call::<_, _, ()>("Reload", &())
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        Ok(None)
                    }
                    SystemdAction::ValidateLoaded | SystemdAction::ValidateRetirementLoaded => {
                        let name = unit
                            .ok_or_else(|| {
                                ServiceError::InvalidSpec(crate::fl!(
                                    "ui-services-load-validation-requires-a-unit"
                                ))
                            })?
                            .file_name();
                        let path: OwnedObjectPath = manager
                            .call("LoadUnit", &(name.as_str(),))
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let proxy =
                            systemd_user_unit_proxy(&connection, &path, SYSTEMD_UNIT_INTERFACE)
                                .await?;
                        let load_state: String = proxy
                            .get_property("LoadState")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let load_error: (String, String) = proxy
                            .get_property("LoadError")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let fragment_path: String = proxy
                            .get_property("FragmentPath")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let needs_reload: bool = proxy
                            .get_property("NeedDaemonReload")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        if action == SystemdAction::ValidateRetirementLoaded {
                            let dropins: Vec<String> = proxy
                                .get_property("DropInPaths")
                                .await
                                .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                            if !dropins.is_empty() {
                                return Err(ServiceError::Validation(fl!(
                                    "legacy-retirement-ownership"
                                )));
                            }
                        }
                        let expected_fragment = std::env::var_os("HOME")
                            .map(PathBuf::from)
                            .ok_or_else(|| {
                                ServiceError::Runtime(crate::fl!(
                                    "ui-services-home-is-unavailable-for-systemd-unit-validation"
                                ))
                            })?
                            .join(".config/systemd/user")
                            .join(&name);
                        let fragment_matches = Path::new(&fragment_path) == expected_fragment;
                        if load_state != "loaded"
                            || !load_error.0.is_empty()
                            || !load_error.1.is_empty()
                            || !fragment_matches
                            || needs_reload
                        {
                            let error_name = if load_error.0.is_empty() {
                                "none"
                            } else {
                                load_error.0.as_str()
                            };
                            return Err(ServiceError::Validation(crate::fl!(
                                "ui-services-systemd-rejected-name-load-state-load-state",
                                name = (name).to_string(),
                                load_state = (load_state).to_string(),
                                error_name = (error_name).to_string()
                            )));
                        }
                        Ok(None)
                    }
                    SystemdAction::Status => {
                        let unit = unit.ok_or_else(|| {
                            ServiceError::InvalidSpec(crate::fl!(
                                "ui-services-status-requires-a-unit"
                            ))
                        })?;
                        let name = unit.file_name();
                        let path = systemd_user_unit_path(&connection, &name).await?;
                        let proxy =
                            systemd_user_unit_proxy(&connection, &path, SYSTEMD_UNIT_INTERFACE)
                                .await?;
                        let active: String = proxy
                            .get_property("ActiveState")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let detail: String = proxy
                            .get_property("SubState")
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let enabled: String = manager
                            .call("GetUnitFileState", &(name.as_str(),))
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        let result = if unit.kind == UnitKind::Service {
                            systemd_user_unit_proxy(&connection, &path, SYSTEMD_SERVICE_INTERFACE)
                                .await?
                                .get_property("Result")
                                .await
                                .map_err(|error| ServiceError::Runtime(error.to_string()))?
                        } else {
                            String::new()
                        };
                        Ok(Some(UnitStatus {
                            active: parse_active_state(&active),
                            enabled: matches!(
                                enabled.as_str(),
                                "enabled" | "enabled-runtime" | "linked" | "linked-runtime"
                            ),
                            detail,
                            result,
                        }))
                    }
                    SystemdAction::Enable | SystemdAction::Disable => {
                        let name = unit
                            .ok_or_else(|| {
                                ServiceError::InvalidSpec(crate::fl!(
                                    "ui-services-systemd-action-requires-a-unit"
                                ))
                            })?
                            .file_name();
                        let files = vec![name.as_str()];
                        if action == SystemdAction::Enable {
                            let _: (bool, Vec<(String, String, String)>) = manager
                                .call("EnableUnitFiles", &(files, false, false))
                                .await
                                .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        } else {
                            let _: Vec<(String, String, String)> = manager
                                .call("DisableUnitFiles", &(files, false))
                                .await
                                .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        }
                        // EnableUnitFiles/DisableUnitFiles only update the unit-file
                        // links. Reload the manager so GetUnitFileState and
                        // NeedDaemonReload immediately reflect the accepted change,
                        // matching systemctl enable/disable behavior.
                        manager
                            .call::<_, _, ()>("Reload", &())
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        Ok(None)
                    }
                    SystemdAction::Start | SystemdAction::Stop | SystemdAction::Restart => {
                        let name = unit
                            .ok_or_else(|| {
                                ServiceError::InvalidSpec(crate::fl!(
                                    "ui-services-systemd-action-requires-a-unit"
                                ))
                            })?
                            .file_name();
                        let method = match action {
                            SystemdAction::Start => "StartUnit",
                            SystemdAction::Stop => "StopUnit",
                            SystemdAction::Restart => "RestartUnit",
                            _ => unreachable!(),
                        };
                        systemd_user_run_job(&manager, method, &name).await?;
                        Ok(None)
                    }
                    SystemdAction::ResetFailed => {
                        let name = unit
                            .ok_or_else(|| {
                                ServiceError::InvalidSpec(crate::fl!(
                                    "ui-services-systemd-action-requires-a-unit"
                                ))
                            })?
                            .file_name();
                        manager
                            .call::<_, _, ()>("ResetFailedUnit", &(name.as_str(),))
                            .await
                            .map_err(|error| ServiceError::Runtime(error.to_string()))?;
                        Ok(None)
                    }
                }
            };
            tokio::select! {
                () = cancellation.cancelled() => Err(ServiceError::Runtime("systemd operation cancelled".into())),
                result = tokio::time::timeout(Duration::from_secs(15), operation) => {
                    result.map_err(|_| ServiceError::Runtime("systemd operation timed out".into()))?
                },
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
    /// Retire only a structurally valid, app-owned legacy monitor. A failure
    /// never re-enables or restarts it; the retained file permits a safe retry.
    pub async fn retire_legacy_mirror(
        &self,
        id: ConnectionId,
        cancellation: CancellationToken,
    ) -> Result<(), ServiceError> {
        let name = UnitName::new(id, UnitKind::Service);
        let Some(content) = self.store.read(&name)? else {
            return Ok(());
        };
        let document = UnitDocument {
            name: name.clone(),
            content,
        };
        StructuralUnitValidator.validate(&document)?;
        if self.store.state(&name)? != UnitFileState::Managed(id)
            || !document
                .content
                .lines()
                .any(|line| line.starts_with("ExecStart=\"/usr/bin/onedrive\" "))
        {
            return Err(ServiceError::Validation(fl!("legacy-retirement-ownership")));
        }
        for action in [
            SystemdAction::ValidateRetirementLoaded,
            SystemdAction::Disable,
            SystemdAction::Stop,
        ] {
            if cancellation.is_cancelled() {
                return Err(ServiceError::Runtime(fl!("legacy-retirement-incomplete")));
            }
            if self.store.read(&name)?.as_ref() != Some(&document.content) {
                return Err(ServiceError::Validation(fl!("legacy-retirement-ownership")));
            }
            self.manager
                .action(action, Some(&name), cancellation.child_token())
                .await?;
        }
        let status = self
            .manager
            .action(
                SystemdAction::Status,
                Some(&name),
                cancellation.child_token(),
            )
            .await?;
        if !status.is_some_and(|status| {
            !status.enabled && matches!(status.active, ActiveState::Inactive | ActiveState::Failed)
        }) || cancellation.is_cancelled()
        {
            return Err(ServiceError::Runtime(fl!("legacy-retirement-incomplete")));
        }
        if self.store.read(&name)?.as_ref() != Some(&document.content) {
            return Err(ServiceError::Validation(fl!("legacy-retirement-ownership")));
        }
        self.remove(&name, cancellation).await
    }

    pub async fn install(
        &self,
        document: &UnitDocument,
        cancellation: CancellationToken,
    ) -> Result<(), ServiceError> {
        let previous = self.store.read(&document.name)?;
        self.store.write(document)?;
        let result = async {
            self.manager
                .action(
                    SystemdAction::DaemonReload,
                    None,
                    cancellation.child_token(),
                )
                .await?;
            self.manager
                .action(
                    SystemdAction::ValidateLoaded,
                    Some(&document.name),
                    cancellation,
                )
                .await?;
            Ok::<(), ServiceError>(())
        }
        .await;
        if let Err(error) = result {
            self.restore(&document.name, previous)?;
            if let Err(rollback_error) = self
                .manager
                .action(SystemdAction::DaemonReload, None, CancellationToken::new())
                .await
            {
                return Err(ServiceError::Runtime(crate::fl!(
                    "ui-services-error-unit-file-was-restored-but-systemd",
                    error = (error).to_string(),
                    rollback_error = (rollback_error).to_string()
                )));
            }
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
        Err(ServiceError::InvalidSpec(crate::fl!(
            "ui-services-unit-text-is-empty-or-contains-control"
        )))
    } else {
        Ok(())
    }
}

fn validate_absolute_executable(path: &Path) -> Result<&str, ServiceError> {
    if !path.is_absolute() {
        return Err(ServiceError::InvalidSpec(crate::fl!(
            "ui-services-unit-executable-must-be-absolute"
        )));
    }
    path.to_str().ok_or_else(|| {
        ServiceError::InvalidSpec(crate::fl!("ui-services-unit-executable-is-not-utf-8"))
    })
}

fn escape_systemd_argument(value: &str) -> Result<String, ServiceError> {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character == '\0' || character == '\n')
    {
        return Err(ServiceError::InvalidSpec(crate::fl!(
            "ui-services-unit-argument-is-empty-or-contains-unsafe"
        )));
    }
    let escaped = value
        .replace('%', "%%")
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
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

fn parse_active_state(active: &str) -> ActiveState {
    match active {
        "active" => ActiveState::Active,
        "activating" => ActiveState::Activating,
        "inactive" => ActiveState::Inactive,
        "failed" => ActiveState::Failed,
        _ => ActiveState::Unknown,
    }
}

/// Start a managed Online mount, clearing only a genuinely failed service.
/// A non-empty or already mounted target is rejected by the mountpoint guard
/// before this function is called (and by ExecCondition for direct starts).
pub async fn start_managed_online_service<M: SystemdManager>(
    manager: &M,
    unit: &UnitName,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let status = manager
        .action(
            SystemdAction::Status,
            Some(unit),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| fl!("online-service-read-state", error = error.to_string()))?
        .ok_or_else(|| fl!("online-service-unknown-state"))?;
    if status.active == ActiveState::Unknown {
        return Err(fl!("online-service-state-unavailable"));
    }
    if status.active == ActiveState::Failed {
        manager
            .action(
                SystemdAction::ResetFailed,
                Some(unit),
                cancellation.child_token(),
            )
            .await
            .map_err(|error| fl!("online-service-reset-failed", error = error.to_string()))?;
    }

    let start_result = manager
        .action(SystemdAction::Start, Some(unit), cancellation.child_token())
        .await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let after = loop {
        let status = manager
            .action(
                SystemdAction::Status,
                Some(unit),
                cancellation.child_token(),
            )
            .await;
        let transitional = matches!(status, Ok(Some(ref status)) if
            status.active == ActiveState::Activating
                || (status.active == ActiveState::Inactive
                    && (status.result.is_empty() || status.result == "success")));
        if !transitional || tokio::time::Instant::now() >= deadline {
            break status;
        }
        tokio::select! {
            () = cancellation.cancelled() => {
                break Err(ServiceError::Runtime("systemd operation cancelled".into()));
            }
            () = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    };
    match after {
        Ok(Some(status))
            if matches!(status.active, ActiveState::Active | ActiveState::Activating) =>
        {
            start_result
                .map_err(|error| fl!("online-service-start-failed", error = error.to_string()))?;
            Ok(())
        }
        Ok(Some(status)) => {
            let cause = if status.result.contains("start-limit") {
                fl!("online-service-rate-limit")
            } else if status.result == "exec-condition" || status.detail == "condition" {
                fl!("online-service-preflight-blocked")
            } else {
                fl!("online-service-not-active")
            };
            let detail = if status.result.is_empty() {
                status.detail.as_str()
            } else {
                status.result.as_str()
            };
            match start_result {
                Ok(_) => Err(format!("{cause} ({detail})")),
                Err(_)
                    if status.result.contains("start-limit")
                        || status.result == "exec-condition" =>
                {
                    Err(format!("{cause} ({detail})"))
                }
                Err(error) => Err(format!("{cause} ({detail}; {error})")),
            }
        }
        Ok(None) => Err(fl!("online-service-unknown-after-start")),
        Err(error) => match start_result {
            Err(start_error) => Err(fl!(
                "online-service-start-and-read-failed",
                start_error = start_error.to_string(),
                error = error.to_string()
            )),
            Ok(_) => Err(fl!(
                "online-service-read-after-start",
                error = error.to_string()
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn retired_engine_retirement_requires_observed_inactive_disabled_state() {
        for state in [
            None,
            Some((ActiveState::Active, false)),
            Some((ActiveState::Unknown, false)),
            Some((ActiveState::Inactive, true)),
        ] {
            let temp = TempDir::new().unwrap();
            let store =
                FileUnitStore::new(temp.path().join("units"), Arc::new(StructuralUnitValidator));
            let mut document = service();
            document.content = document
                .content
                .replace("/usr/bin/example", "/usr/bin/onedrive");
            store.write(&document).unwrap();
            let manager = FakeSystemdManager::default();
            for _ in 0..3 {
                manager.push(Ok(None));
            }
            manager.push(Ok(state.map(|(active, enabled)| UnitStatus {
                active,
                enabled,
                detail: String::new(),
                result: String::new(),
            })));
            assert!(
                UnitController::new(store.clone(), manager.clone())
                    .retire_legacy_mirror(id(), CancellationToken::new())
                    .await
                    .is_err()
            );
            assert_eq!(store.read(&document.name).unwrap(), Some(document.content));
            assert_eq!(manager.actions().len(), 4);
        }
        let temp = TempDir::new().unwrap();
        let store =
            FileUnitStore::new(temp.path().join("units"), Arc::new(StructuralUnitValidator));
        let manager = FakeSystemdManager::default();
        UnitController::new(store, manager.clone())
            .retire_legacy_mirror(id(), CancellationToken::new())
            .await
            .unwrap();
        assert!(manager.actions().is_empty());
    }

    #[tokio::test]
    async fn retired_engine_unit_retirement_preserves_data_and_never_restarts_on_failure() {
        for fail_at in 0..=5 {
            let temp = TempDir::new().unwrap();
            let store =
                FileUnitStore::new(temp.path().join("units"), Arc::new(StructuralUnitValidator));
            let mut document = service();
            document.content = document
                .content
                .replace("/usr/bin/example", "/usr/bin/onedrive");
            store.write(&document).unwrap();
            let data = temp.path().join("mirror-file");
            fs::write(&data, "synthetic unsynchronized content").unwrap();
            let manager = FakeSystemdManager::default();
            for index in 0..5 {
                manager.push(if index == fail_at {
                    Err(ServiceError::Runtime("synthetic failure".into()))
                } else if index == 3 {
                    Ok(Some(UnitStatus {
                        active: ActiveState::Inactive,
                        enabled: false,
                        detail: "dead".into(),
                        result: "success".into(),
                    }))
                } else {
                    Ok(None)
                });
            }
            let result = UnitController::new(store.clone(), manager.clone())
                .retire_legacy_mirror(id(), CancellationToken::new())
                .await;
            assert_eq!(result.is_ok(), fail_at == 5);
            assert_eq!(
                fs::read_to_string(data).unwrap(),
                "synthetic unsynchronized content"
            );
            assert_eq!(
                store.read(&document.name).unwrap(),
                (fail_at != 5).then_some(document.content)
            );
            assert!(manager.actions().iter().all(|(action, _)| !matches!(
                action,
                SystemdAction::Start | SystemdAction::Restart | SystemdAction::Enable
            )));
        }
    }

    #[tokio::test]
    async fn retired_engine_unit_retirement_refuses_foreign_malformed_and_cancelled_units() {
        for case in 0..5 {
            let temp = TempDir::new().unwrap();
            let root = temp.path().join("units");
            let store = FileUnitStore::new(root.clone(), Arc::new(StructuralUnitValidator));
            let mut document = service();
            if case != 0 {
                document.content = document
                    .content
                    .replace("/usr/bin/example", "/usr/bin/onedrive");
            }
            store.write(&document).unwrap();
            let path = root.join(document.name.file_name());
            if case == 1 {
                fs::write(&path, "[Service]\nExecStart=/usr/bin/onedrive\n").unwrap();
            }
            if case == 2 {
                fs::write(
                    &path,
                    format!(
                        "{}\n[Service]\nExecStart=/usr/bin/other\n",
                        document.content
                    ),
                )
                .unwrap();
            }
            if case == 3 {
                let original = temp.path().join("unrelated.service");
                fs::rename(&path, &original).unwrap();
                std::os::unix::fs::symlink(original, &path).unwrap();
            }
            let cancellation = CancellationToken::new();
            if case == 4 {
                cancellation.cancel();
            }
            let manager = FakeSystemdManager::default();
            assert!(
                UnitController::new(store, manager.clone())
                    .retire_legacy_mirror(id(), cancellation)
                    .await
                    .is_err()
            );
            assert!(manager.actions().is_empty());
            assert!(path.exists());
        }
    }

    #[tokio::test]
    async fn unit_observation_deadline_does_not_block_executor() {
        let operation = async { std::future::pending::<Result<(), String>>().await };
        let result = bounded_unit_observation(
            operation,
            Duration::from_millis(10),
            CancellationToken::new(),
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn unit_observation_absence_uses_only_typed_systemd_errors() {
        let method_error = |name: &str| {
            zbus::Error::MethodError(
                name.try_into().unwrap(),
                Some("synthetic missing unit diagnostic".into()),
                zbus::Message::method_call("/org/freedesktop/systemd1", "LoadUnit")
                    .unwrap()
                    .build(&())
                    .unwrap(),
            )
        };
        assert!(unit_load_reports_missing(&method_error(
            "org.freedesktop.systemd1.NoSuchUnit"
        )));
        for name in [
            "org.freedesktop.DBus.Error.AccessDenied",
            "org.freedesktop.DBus.Error.Timeout",
            "org.freedesktop.systemd1.LoadFailed",
            "org.freedesktop.systemd1.NoSuchUnitFile",
        ] {
            assert!(!unit_load_reports_missing(&method_error(name)));
        }
        assert!(
            !observed_unit_file_enabled(Err(method_error(
                "org.freedesktop.systemd1.NoSuchUnitFile"
            )))
            .unwrap()
        );
        assert!(
            observed_unit_file_enabled(Err(method_error(
                "org.freedesktop.DBus.Error.AccessDenied"
            )))
            .is_err()
        );
        assert!(observed_unit_file_enabled(Ok("enabled".into())).unwrap());
        assert!(!observed_unit_file_enabled(Ok("disabled".into())).unwrap());
    }

    #[tokio::test]
    async fn unit_observation_cancellation_and_empty_batch_need_no_bus() {
        assert!(
            systemd_user_status_batch(&[], CancellationToken::new())
                .await
                .unwrap()
                .is_empty()
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(
            systemd_user_status_batch(
                &[UnitName::new(ConnectionId::new(), UnitKind::Service)],
                cancellation
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn unit_observation_success_is_preserved() {
        let result = bounded_unit_observation(
            async { Ok(42) },
            Duration::from_secs(1),
            CancellationToken::new(),
        )
        .await;
        assert_eq!(result, Ok(42));
    }

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
            pre_start_condition: None,
            restart_on_failure: true,
            skip_on_metered: false,
        })
        .expect("service")
    }

    #[test]
    fn named_systemd_start_accepts_only_plain_service_units() {
        assert!(valid_systemd_service_name(
            "cosmic-mounter-openconnect-agent.service"
        ));
        assert!(!valid_systemd_service_name(""));
        assert!(!valid_systemd_service_name("openconnect.timer"));
        assert!(!valid_systemd_service_name("../openconnect.service"));
        assert!(!valid_systemd_service_name("open connect.service"));
    }

    #[test]
    fn scheduled_mirror_condition_is_rendered_only_when_metered_sync_is_disabled() {
        let mut spec = ServiceSpec {
            connection_id: id(),
            description: "Metered mirror".into(),
            executable: PathBuf::from("/usr/bin/rclone"),
            arguments: vec!["bisync".into()],
            pre_start_condition: None,
            restart_on_failure: false,
            skip_on_metered: true,
        };
        let guarded = UnitDocument::service(&spec).unwrap();
        assert!(guarded.content.contains("ExecCondition="));
        assert!(guarded.content.contains(crate::vpn::METERED_GUARD_FLAG));
        assert!(!guarded.content.contains("nmcli"));
        spec.skip_on_metered = false;
        assert!(
            !UnitDocument::service(&spec)
                .unwrap()
                .content
                .contains("ExecCondition=")
        );
    }

    #[test]
    fn teams_guard_runs_before_mount_and_requires_safe_arguments() {
        let mut spec = ServiceSpec {
            connection_id: id(),
            description: "Teams assessment".into(),
            executable: PathBuf::from("/usr/bin/rclone"),
            arguments: vec!["mount".into()],
            pre_start_condition: Some(ServiceCommand {
                executable: PathBuf::from("/usr/bin/cloud-mounter"),
                arguments: vec![
                    "--verify-teams-mount".into(),
                    "Shared Documents".into(),
                    "https://example.sharepoint.com/Shared%20Documents".into(),
                ],
            }),
            restart_on_failure: true,
            skip_on_metered: false,
        };
        let unit = UnitDocument::service(&spec).unwrap();
        let condition = unit.content.find("ExecCondition=").unwrap();
        let start = unit.content.find("ExecStart=").unwrap();
        assert!(condition < start);
        assert!(unit.content.contains("\"Shared Documents\""));
        assert!(unit.content.contains("Shared%%20Documents"));
        spec.pre_start_condition.as_mut().unwrap().arguments[1] = "bad\nargument".into();
        assert!(UnitDocument::service(&spec).is_err());
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
[Timer]\nOnActiveSec=900s\nOnUnitInactiveSec=900s\nPersistent=true\n\
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
        manager.push(Err(ServiceError::Runtime("reload failed".into())));
        manager.push(Ok(None));
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
    async fn load_validation_failure_removes_new_unit_and_reloads_rollback() {
        let temp = TempDir::new().expect("temp");
        let store = FileUnitStore::new(temp.path().into(), Arc::new(StructuralUnitValidator));
        let document = service();
        let manager = FakeSystemdManager::default();
        manager.push(Ok(None));
        manager.push(Err(ServiceError::Runtime("load validation failed".into())));
        manager.push(Ok(None));
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
    async fn installation_reloads_then_validates_systemd_load_state() {
        let temp = TempDir::new().expect("temp");
        let store = FileUnitStore::new(temp.path().into(), Arc::new(StructuralUnitValidator));
        let document = service();
        let manager = FakeSystemdManager::default();
        let controller = UnitController::new(store, manager.clone());

        controller
            .install(&document, CancellationToken::new())
            .await
            .expect("install");
        assert_eq!(
            manager.actions(),
            vec![
                (SystemdAction::DaemonReload, None),
                (
                    SystemdAction::ValidateLoaded,
                    Some(document.name.file_name())
                )
            ]
        );
    }

    #[test]
    fn structural_validator_rejects_unknown_duplicate_and_mismatched_directives() {
        let validator = StructuralUnitValidator;
        let mut unknown = service();
        unknown.content = unknown
            .content
            .replace("Type=simple", "Mystery=true\nType=simple");
        assert!(validator.validate(&unknown).is_err());

        let mut duplicate = service();
        duplicate.content = duplicate
            .content
            .replace("ExecStart=", "ExecStart=\"/usr/bin/true\"\nExecStart=");
        assert!(validator.validate(&duplicate).is_err());

        let timer = UnitDocument::timer(&TimerSpec {
            connection_id: id(),
            description: "Example timer".into(),
            interval: Duration::from_secs(60),
            persistent: true,
        })
        .unwrap();
        let mut mismatched = timer.clone();
        mismatched.content = mismatched
            .content
            .replace(&service().name.file_name(), "cosmic-mounter-other.service");
        assert!(validator.validate(&mismatched).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_analyze_accepts_generated_templates_in_test_tooling() {
        let available = std::process::Command::new("systemd-analyze")
            .arg("--version")
            .output()
            .expect("systemd-analyze is a declared Linux build dependency");
        assert!(
            available.status.success(),
            "systemd-analyze --version failed"
        );
        let temp = TempDir::new().expect("temp");
        let service = UnitDocument::service(&ServiceSpec {
            connection_id: id(),
            description: "Generated verification service".into(),
            executable: PathBuf::from("/usr/bin/true"),
            arguments: vec!["--version".into()],
            pre_start_condition: Some(ServiceCommand {
                executable: PathBuf::from("/usr/bin/true"),
                arguments: vec!["condition".into()],
            }),
            restart_on_failure: true,
            skip_on_metered: false,
        })
        .unwrap();
        let timer = UnitDocument::timer(&TimerSpec {
            connection_id: id(),
            description: "Generated verification timer".into(),
            interval: Duration::from_secs(900),
            persistent: true,
        })
        .unwrap();
        let service_path = temp.path().join(service.name.file_name());
        let timer_path = temp.path().join(timer.name.file_name());
        fs::write(&service_path, service.content).unwrap();
        fs::write(&timer_path, timer.content).unwrap();

        let output = std::process::Command::new("systemd-analyze")
            .args(["--user", "verify"])
            .arg(&service_path)
            .arg(&timer_path)
            .output()
            .expect("run systemd-analyze test verifier");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
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

    #[test]
    fn dbus_active_states_map_to_runtime_states() {
        assert_eq!(parse_active_state("active"), ActiveState::Active);
        assert_eq!(parse_active_state("activating"), ActiveState::Activating);
        assert_eq!(parse_active_state("inactive"), ActiveState::Inactive);
        assert_eq!(parse_active_state("failed"), ActiveState::Failed);
        assert_eq!(parse_active_state("deactivating"), ActiveState::Unknown);
    }

    #[tokio::test]
    #[ignore = "requires the live systemd user bus"]
    async fn live_systemd_user_dbus_smoke_test() {
        systemd_user_available()
            .await
            .expect("connect to systemd user manager");
        assert_eq!(
            systemd_user_active_state("default.target")
                .await
                .expect("read default target state"),
            "active"
        );
        assert_eq!(
            systemd_user_invocation_id("default.target")
                .await
                .expect("read default target invocation")
                .len(),
            32
        );
        let (_, timeout, mode, signal) = systemd_user_stop_policy("dbus.service")
            .await
            .expect("read a user service stop policy");
        assert!(timeout > 0);
        assert!(!mode.is_empty());
        assert!(signal > 0);

        if let Ok(store) = FileUnitStore::user(Arc::new(StructuralUnitValidator))
            && let Ok(entries) = fs::read_dir(&store.root)
            && let Some((name, content)) = entries.filter_map(Result::ok).find_map(|entry| {
                let name = entry.file_name().to_str()?.to_owned();
                let kind = if name.ends_with(".service") {
                    UnitKind::Service
                } else if name.ends_with(".timer") {
                    UnitKind::Timer
                } else {
                    return None;
                };
                let content = fs::read_to_string(entry.path()).ok()?;
                let id = parse_owned_connection(&content)?;
                (name.starts_with("cosmic-mounter-")).then_some((UnitName::new(id, kind), content))
            })
        {
            StructuralUnitValidator
                .validate(&UnitDocument {
                    name: name.clone(),
                    content,
                })
                .expect("validate an installed managed unit structurally");
            DbusSystemdManager
                .action(
                    SystemdAction::ValidateLoaded,
                    Some(&name),
                    CancellationToken::new(),
                )
                .await
                .expect("validate an installed managed unit through systemd D-Bus");
        }
    }

    #[tokio::test]
    #[ignore = "requires the optional OpenConnect authentication-agent user service"]
    async fn live_openconnect_agent_start_waits_for_active_service() {
        let unit = "cosmic-mounter-openconnect-agent.service";
        systemd_user_start_named_unit(unit)
            .await
            .expect("start OpenConnect authentication agent");
        assert_eq!(
            systemd_user_active_state(unit)
                .await
                .expect("read OpenConnect authentication-agent state"),
            "active"
        );
    }

    #[tokio::test]
    #[ignore = "requires COSMIC_MOUNTER_LIVE_CONNECTION_ID and its disposable managed service"]
    async fn live_managed_online_service_start_uses_systemd_dbus() {
        let id = std::env::var("COSMIC_MOUNTER_LIVE_CONNECTION_ID")
            .expect("set COSMIC_MOUNTER_LIVE_CONNECTION_ID");
        let id = ConnectionId::from_uuid(Uuid::parse_str(&id).expect("connection UUID"));
        let unit = UnitName::new(id, UnitKind::Service);
        start_managed_online_service(&DbusSystemdManager, &unit, CancellationToken::new())
            .await
            .expect("start managed service through systemd D-Bus");
    }

    fn online_status(active: ActiveState, detail: &str, result: &str) -> UnitStatus {
        UnitStatus {
            active,
            enabled: false,
            detail: detail.into(),
            result: result.into(),
        }
    }

    #[tokio::test]
    async fn online_start_does_not_reset_an_inactive_service() {
        let manager = FakeSystemdManager::default();
        manager.push(Ok(Some(online_status(
            ActiveState::Inactive,
            "dead",
            "success",
        ))));
        manager.push(Ok(None));
        manager.push(Ok(Some(online_status(
            ActiveState::Active,
            "running",
            "success",
        ))));
        let unit = service().name;

        start_managed_online_service(&manager, &unit, CancellationToken::new())
            .await
            .expect("online start");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Status,
                SystemdAction::Start,
                SystemdAction::Status
            ]
        );
    }

    #[tokio::test]
    async fn online_start_waits_for_transitional_unit_state_to_become_active() {
        let manager = FakeSystemdManager::default();
        manager.push(Ok(Some(online_status(
            ActiveState::Inactive,
            "dead",
            "success",
        ))));
        manager.push(Ok(None));
        manager.push(Ok(Some(online_status(
            ActiveState::Inactive,
            "dead",
            "success",
        ))));
        manager.push(Ok(Some(online_status(
            ActiveState::Activating,
            "start",
            "success",
        ))));
        manager.push(Ok(Some(online_status(
            ActiveState::Active,
            "running",
            "success",
        ))));
        let unit = service().name;

        start_managed_online_service(&manager, &unit, CancellationToken::new())
            .await
            .expect("transitional online start");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Status,
                SystemdAction::Start,
                SystemdAction::Status,
                SystemdAction::Status,
                SystemdAction::Status,
            ]
        );
    }

    #[tokio::test]
    async fn online_start_reports_failed_reset_and_does_not_start() {
        let manager = FakeSystemdManager::default();
        manager.push(Ok(Some(online_status(
            ActiveState::Failed,
            "failed",
            "exit-code",
        ))));
        manager.push(Err(ServiceError::Runtime("reset rejected".into())));
        let unit = service().name;

        let error = start_managed_online_service(&manager, &unit, CancellationToken::new())
            .await
            .expect_err("reset failure");
        assert!(error.contains("Could not reset failed Online mount service:"));
        assert!(error.contains("reset rejected"));
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [SystemdAction::Status, SystemdAction::ResetFailed]
        );
    }

    #[tokio::test]
    async fn online_start_resets_failed_service_before_start() {
        let manager = FakeSystemdManager::default();
        manager.push(Ok(Some(online_status(
            ActiveState::Failed,
            "failed",
            "exit-code",
        ))));
        manager.push(Ok(None));
        manager.push(Ok(None));
        manager.push(Ok(Some(online_status(
            ActiveState::Active,
            "running",
            "success",
        ))));
        let unit = service().name;

        start_managed_online_service(&manager, &unit, CancellationToken::new())
            .await
            .expect("recovered start");
        assert_eq!(
            manager
                .actions()
                .iter()
                .map(|(action, _)| *action)
                .collect::<Vec<_>>(),
            [
                SystemdAction::Status,
                SystemdAction::ResetFailed,
                SystemdAction::Start,
                SystemdAction::Status
            ]
        );
    }

    #[tokio::test]
    async fn online_start_distinguishes_rate_limit_and_mountpoint_guard() {
        for (result, detail, expected) in [
            ("start-limit-hit", "failed", "start rate limit"),
            ("exec-condition", "dead", "blocked by its preflight"),
        ] {
            let manager = FakeSystemdManager::default();
            manager.push(Ok(Some(online_status(
                ActiveState::Inactive,
                "dead",
                "success",
            ))));
            manager.push(Ok(None));
            manager.push(Ok(Some(online_status(ActiveState::Failed, detail, result))));
            let unit = service().name;
            let error = start_managed_online_service(&manager, &unit, CancellationToken::new())
                .await
                .expect_err("start blocked");
            assert!(error.contains(expected), "{error}");
            assert!(
                !manager
                    .actions()
                    .iter()
                    .any(|(action, _)| *action == SystemdAction::ResetFailed)
            );
        }
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
