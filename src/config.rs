// SPDX-License-Identifier: MIT

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use cosmic::cosmic_config::{
    self, ConfigGet, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry,
};
use serde::{Deserialize, Serialize};

use crate::model::{
    Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, OnlineMountConfig, VpnProfile,
    VpnProfileId,
};

pub const APP_ID: &str = "io.github.uutzinger.cosmic-ext-applet-mounter";
pub const CONFIG_SCHEMA_VERSION: u32 = 2;
const DOCUMENT_KEY: &str = "document";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDocument {
    pub schema_version: u32,
    pub notifications_enabled: bool,
    pub connections: Vec<Connection>,
    pub vpn_profiles: Vec<VpnProfile>,
}

impl Default for ConfigDocument {
    fn default() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            notifications_enabled: true,
            connections: Vec::new(),
            vpn_profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    pub document: ConfigDocument,
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
struct LegacyConfigV1 {
    notifications_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSource {
    Current,
    Fresh,
    MigratedV1,
    RecoveredDefaults,
}

#[derive(Debug)]
pub struct LoadReport {
    pub config: Config,
    pub source: LoadSource,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidSchemaVersion(u32),
    InvalidName {
        field: &'static str,
        value: String,
    },
    DuplicateConnectionId(ConnectionId),
    DuplicateVpnProfileId(VpnProfileId),
    MissingVpnProfile(VpnProfileId),
    InvalidRemoteReference(ConnectionId),
    InvalidRemoteSubpath(ConnectionId),
    InvalidLocalPath {
        connection: ConnectionId,
        path: PathBuf,
    },
    UnsafeSystemPath {
        connection: ConnectionId,
        path: PathBuf,
    },
    ConflictingTargets {
        first: ConnectionId,
        second: ConnectionId,
        first_path: PathBuf,
        second_path: PathBuf,
    },
    InvalidCacheDirectory {
        connection: ConnectionId,
        path: PathBuf,
    },
    InvalidRecoveryDirectory {
        connection: ConnectionId,
        path: PathBuf,
    },
    InvalidSyncInterval(ConnectionId),
    InvalidVpnTimeout(VpnProfileId),
    InvalidReadinessValue(VpnProfileId),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug)]
pub enum SaveError {
    Validation(Vec<ValidationError>),
    Storage(cosmic_config::Error),
    Io(String),
    Serialize(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(errors) => write!(formatter, "configuration is invalid: {errors:?}"),
            Self::Storage(error) => error.fmt(formatter),
            Self::Io(error) | Self::Serialize(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SaveError {}

#[derive(Debug, Clone)]
pub struct HostVisibleConfigStorage {
    document_path: PathBuf,
}

#[derive(Debug)]
pub enum AppConfigStorage {
    Cosmic(cosmic_config::Config),
    HostVisible(HostVisibleConfigStorage),
}

impl HostVisibleConfigStorage {
    #[must_use]
    pub fn new(document_path: PathBuf) -> Self {
        Self { document_path }
    }

    #[must_use]
    pub fn document_path(&self) -> &Path {
        &self.document_path
    }

    fn load(&self) -> LoadReport {
        match fs::read_to_string(&self.document_path) {
            Ok(contents) => match ron::from_str::<ConfigDocument>(&contents) {
                Ok(document) => {
                    let config = Config { document };
                    match config.validate() {
                        Ok(()) => LoadReport {
                            config,
                            source: LoadSource::Current,
                            warnings: Vec::new(),
                        },
                        Err(errors) => LoadReport {
                            config: Config::default(),
                            source: LoadSource::RecoveredDefaults,
                            warnings: errors.into_iter().map(|error| error.to_string()).collect(),
                        },
                    }
                }
                Err(error) => LoadReport {
                    config: Config::default(),
                    source: LoadSource::RecoveredDefaults,
                    warnings: vec![format!(
                        "failed to parse host-visible applet configuration `{}`: {error}",
                        self.document_path.display()
                    )],
                },
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => LoadReport {
                config: Config::default(),
                source: LoadSource::Fresh,
                warnings: Vec::new(),
            },
            Err(error) => LoadReport {
                config: Config::default(),
                source: LoadSource::RecoveredDefaults,
                warnings: vec![format!(
                    "failed to read host-visible applet configuration `{}`: {error}",
                    self.document_path.display()
                )],
            },
        }
    }

    fn write(&self, document: &ConfigDocument) -> Result<(), SaveError> {
        let parent = self.document_path.parent().ok_or_else(|| {
            SaveError::Io(format!(
                "configuration path `{}` has no parent directory",
                self.document_path.display()
            ))
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            SaveError::Io(format!(
                "failed to create configuration directory `{}`: {error}",
                parent.display()
            ))
        })?;
        let content = ron::ser::to_string_pretty(document, ron::ser::PrettyConfig::new())
            .map_err(|error| SaveError::Serialize(error.to_string()))?;
        let tmp_path = self.document_path.with_extension("tmp");
        fs::write(&tmp_path, content).map_err(|error| {
            SaveError::Io(format!(
                "failed to write temporary configuration `{}`: {error}",
                tmp_path.display()
            ))
        })?;
        fs::rename(&tmp_path, &self.document_path).map_err(|error| {
            let _ = fs::remove_file(&tmp_path);
            SaveError::Io(format!(
                "failed to install configuration `{}`: {error}",
                self.document_path.display()
            ))
        })
    }
}

impl AppConfigStorage {
    pub fn runtime() -> Result<Self, String> {
        if running_in_flatpak() {
            Ok(Self::HostVisible(HostVisibleConfigStorage::new(
                host_visible_document_path()?,
            )))
        } else {
            cosmic_config::Config::new(APP_ID, Config::VERSION)
                .map(Self::Cosmic)
                .map_err(|error| format!("Failed to open applet configuration storage: {error}"))
        }
    }

    fn load(&self) -> LoadReport {
        match self {
            Self::Cosmic(current) => {
                let legacy = cosmic_config::Config::new(APP_ID, LegacyConfigV1::VERSION);
                match legacy {
                    Ok(legacy) => Config::load_from(current, &legacy),
                    Err(error) => LoadReport {
                        config: Config::default(),
                        source: LoadSource::RecoveredDefaults,
                        warnings: vec![error.to_string()],
                    },
                }
            }
            Self::HostVisible(storage) => storage.load(),
        }
    }

    fn write(&self, config: &Config) -> Result<(), SaveError> {
        match self {
            Self::Cosmic(storage) => config.write_validated(storage),
            Self::HostVisible(storage) => {
                config.validate().map_err(SaveError::Validation)?;
                storage.write(&config.document)
            }
        }
    }
}

fn running_in_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

fn host_visible_document_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        "HOME is not set; cannot locate host-visible applet configuration".to_owned()
    })?;
    Ok(home
        .join(".config")
        .join("cosmic")
        .join(APP_ID)
        .join(format!("v{}", Config::VERSION))
        .join(DOCUMENT_KEY))
}

impl Config {
    #[must_use]
    pub fn notifications_enabled(&self) -> bool {
        self.document.notifications_enabled
    }

    pub fn load() -> LoadReport {
        let current = cosmic_config::Config::new(APP_ID, Self::VERSION);
        let legacy = cosmic_config::Config::new(APP_ID, LegacyConfigV1::VERSION);

        match (current, legacy) {
            (Ok(current), Ok(legacy)) => Self::load_from(&current, &legacy),
            (current, legacy) => {
                let warnings = current
                    .err()
                    .into_iter()
                    .chain(legacy.err())
                    .map(|error| error.to_string())
                    .collect();
                LoadReport {
                    config: Self::default(),
                    source: LoadSource::RecoveredDefaults,
                    warnings,
                }
            }
        }
    }

    pub fn load_runtime() -> LoadReport {
        match AppConfigStorage::runtime() {
            Ok(storage) => storage.load(),
            Err(error) => LoadReport {
                config: Self::default(),
                source: LoadSource::RecoveredDefaults,
                warnings: vec![error],
            },
        }
    }

    pub fn load_from(
        current: &cosmic_config::Config,
        legacy: &cosmic_config::Config,
    ) -> LoadReport {
        match current.get_local::<ConfigDocument>("document") {
            Ok(document) => {
                let config = Self { document };
                match config.validate() {
                    Ok(()) => LoadReport {
                        config,
                        source: LoadSource::Current,
                        warnings: Vec::new(),
                    },
                    Err(errors) => LoadReport {
                        config: Self::default(),
                        source: LoadSource::RecoveredDefaults,
                        warnings: errors.into_iter().map(|error| error.to_string()).collect(),
                    },
                }
            }
            Err(error) if !error.is_err() => {
                match legacy.get_local::<bool>("notifications_enabled") {
                    Ok(notifications_enabled) => {
                        let migrated = Self {
                            document: ConfigDocument {
                                notifications_enabled,
                                ..ConfigDocument::default()
                            },
                        };
                        let mut warnings = Vec::new();
                        if let Err(error) = migrated.write_validated(current) {
                            warnings
                                .push(format!("failed to persist migrated configuration: {error}"));
                        }
                        LoadReport {
                            config: migrated,
                            source: LoadSource::MigratedV1,
                            warnings,
                        }
                    }
                    Err(error) if !error.is_err() => LoadReport {
                        config: Self::default(),
                        source: LoadSource::Fresh,
                        warnings: Vec::new(),
                    },
                    Err(error) => LoadReport {
                        config: Self::default(),
                        source: LoadSource::RecoveredDefaults,
                        warnings: vec![error.to_string()],
                    },
                }
            }
            Err(error) => LoadReport {
                config: Self::default(),
                source: LoadSource::RecoveredDefaults,
                warnings: vec![error.to_string()],
            },
        }
    }

    pub fn write_validated(&self, storage: &cosmic_config::Config) -> Result<(), SaveError> {
        self.validate().map_err(SaveError::Validation)?;
        self.write_entry(storage).map_err(SaveError::Storage)
    }

    pub fn update_validated<F>(
        &mut self,
        storage: &cosmic_config::Config,
        update: F,
    ) -> Result<bool, SaveError>
    where
        F: FnOnce(&mut ConfigDocument),
    {
        let previous = self.clone();
        update(&mut self.document);
        if *self == previous {
            return Ok(false);
        }
        if let Err(error) = self.write_validated(storage) {
            *self = previous;
            return Err(error);
        }
        Ok(true)
    }

    pub fn update_validated_runtime<F>(&mut self, update: F) -> Result<bool, SaveError>
    where
        F: FnOnce(&mut ConfigDocument),
    {
        let storage = AppConfigStorage::runtime().map_err(SaveError::Io)?;
        self.update_validated_with(&storage, update)
    }

    pub fn update_validated_with<F>(
        &mut self,
        storage: &AppConfigStorage,
        update: F,
    ) -> Result<bool, SaveError>
    where
        F: FnOnce(&mut ConfigDocument),
    {
        let previous = self.clone();
        update(&mut self.document);
        if *self == previous {
            return Ok(false);
        }
        if let Err(error) = storage.write(self) {
            *self = previous;
            return Err(error);
        }
        Ok(true)
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if self.document.schema_version != CONFIG_SCHEMA_VERSION {
            errors.push(ValidationError::InvalidSchemaVersion(
                self.document.schema_version,
            ));
        }

        let mut vpn_ids = HashSet::new();
        for vpn in &self.document.vpn_profiles {
            if !vpn_ids.insert(vpn.id) {
                errors.push(ValidationError::DuplicateVpnProfileId(vpn.id));
            }
            validate_name("vpn.name", &vpn.name, &mut errors);
            if vpn.timeout_seconds == 0 {
                errors.push(ValidationError::InvalidVpnTimeout(vpn.id));
            }
            if vpn
                .external_profile_id
                .as_deref()
                .is_some_and(has_control_char)
                || vpn.readiness_checks.iter().any(|check| {
                    use crate::model::ReadinessCheck;
                    match check {
                        ReadinessCheck::NetworkManagerState => false,
                        ReadinessCheck::Interface(value)
                        | ReadinessCheck::Route(value)
                        | ReadinessCheck::DnsName(value)
                        | ReadinessCheck::Endpoint(value) => {
                            value.trim().is_empty() || has_control_char(value)
                        }
                    }
                })
            {
                errors.push(ValidationError::InvalidReadinessValue(vpn.id));
            }
        }

        let mut connection_ids = HashSet::new();
        let mut targets = Vec::new();
        for connection in &self.document.connections {
            if !connection_ids.insert(connection.id) {
                errors.push(ValidationError::DuplicateConnectionId(connection.id));
            }
            validate_name("connection.name", &connection.name, &mut errors);
            if connection.remote_reference.trim().is_empty()
                || has_control_char(&connection.remote_reference)
            {
                errors.push(ValidationError::InvalidRemoteReference(connection.id));
            }
            if connection
                .remote_subpath
                .as_deref()
                .is_some_and(|path| has_control_char(path) || is_unsafe_remote_subpath(path))
            {
                errors.push(ValidationError::InvalidRemoteSubpath(connection.id));
            }
            if connection
                .vpn_profile_id
                .is_some_and(|id| !vpn_ids.contains(&id))
            {
                errors.push(ValidationError::MissingVpnProfile(
                    connection.vpn_profile_id.expect("checked as some"),
                ));
            }

            let Some(local_path) = normalize_absolute(&connection.local_path) else {
                errors.push(ValidationError::InvalidLocalPath {
                    connection: connection.id,
                    path: connection.local_path.clone(),
                });
                continue;
            };
            if is_unsafe_system_path(&local_path) {
                errors.push(ValidationError::UnsafeSystemPath {
                    connection: connection.id,
                    path: local_path.clone(),
                });
            }

            match &connection.mode {
                ConnectionMode::OnlineMount(options) => {
                    validate_online_mount(connection.id, &local_path, options, &mut errors);
                }
                ConnectionMode::OfflineMirror(options) => {
                    validate_offline_mirror(connection.id, &local_path, options, &mut errors);
                }
            }
            targets.push((connection.id, local_path));
        }

        for (index, (first_id, first_path)) in targets.iter().enumerate() {
            for (second_id, second_path) in &targets[index + 1..] {
                if paths_overlap(first_path, second_path) {
                    errors.push(ValidationError::ConflictingTargets {
                        first: *first_id,
                        second: *second_id,
                        first_path: first_path.clone(),
                        second_path: second_path.clone(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_name(field: &'static str, value: &str, errors: &mut Vec<ValidationError>) {
    if value.trim().is_empty() || has_control_char(value) {
        errors.push(ValidationError::InvalidName {
            field,
            value: value.to_owned(),
        });
    }
}

fn validate_online_mount(
    id: ConnectionId,
    local_path: &Path,
    options: &OnlineMountConfig,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(cache) = &options.cache_directory {
        match normalize_absolute(cache) {
            Some(cache) if !paths_overlap(local_path, &cache) && !is_unsafe_system_path(&cache) => {
            }
            _ => errors.push(ValidationError::InvalidCacheDirectory {
                connection: id,
                path: cache.clone(),
            }),
        }
    }
}

fn validate_offline_mirror(
    id: ConnectionId,
    local_path: &Path,
    options: &OfflineMirrorConfig,
    errors: &mut Vec<ValidationError>,
) {
    match normalize_absolute(&options.recovery_directory) {
        Some(recovery)
            if !paths_overlap(local_path, &recovery) && !is_unsafe_system_path(&recovery) => {}
        _ => errors.push(ValidationError::InvalidRecoveryDirectory {
            connection: id,
            path: options.recovery_directory.clone(),
        }),
    }
    if options.sync_interval_minutes == 0 {
        errors.push(ValidationError::InvalidSyncInterval(id));
    }
}

fn has_control_char(value: &str) -> bool {
    value.chars().any(char::is_control)
}

fn is_unsafe_remote_subpath(value: &str) -> bool {
    let path = Path::new(value);
    value.trim().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
}

fn normalize_absolute(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Prefix(_) => return None,
        }
    }
    Some(normalized)
}

fn paths_overlap(first: &Path, second: &Path) -> bool {
    first == second || first.starts_with(second) || second.starts_with(first)
}

fn is_unsafe_system_path(path: &Path) -> bool {
    const UNSAFE: &[&str] = &[
        "/", "/bin", "/boot", "/dev", "/etc", "/lib", "/lib64", "/proc", "/root", "/run", "/sbin",
        "/sys", "/usr", "/var",
    ];
    UNSAFE.iter().any(|unsafe_path| {
        let unsafe_path = Path::new(unsafe_path);
        path == unsafe_path || (unsafe_path != Path::new("/") && path.starts_with(unsafe_path))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use cosmic::cosmic_config::CosmicConfigEntry;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::model::{
        ConnectionMode, OfflineMirrorConfig, OnlineMountConfig, Provider, ReadinessCheck,
        TuningProfile, VpnKind,
    };

    const CONNECTION_UUID: &str = "2a3f5d45-e867-47e7-943f-66cf60e777ad";
    const VPN_UUID: &str = "17ea4cc5-f4f0-405b-b112-dad6f855bb77";

    fn connection_id() -> ConnectionId {
        ConnectionId::from_uuid(Uuid::parse_str(CONNECTION_UUID).expect("valid UUID"))
    }

    fn vpn_id() -> VpnProfileId {
        VpnProfileId::from_uuid(Uuid::parse_str(VPN_UUID).expect("valid UUID"))
    }

    fn online_connection(path: &str) -> Connection {
        Connection {
            id: connection_id(),
            name: "Example mount".into(),
            provider: Provider::GoogleDrive,
            mode: ConnectionMode::OnlineMount(OnlineMountConfig {
                cache_directory: Some(PathBuf::from("/home/example/.cache/cosmic-mounter/google")),
                ..OnlineMountConfig::default()
            }),
            remote_reference: "example-gdrive".into(),
            remote_subpath: Some("Projects".into()),
            local_path: PathBuf::from(path),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
        }
    }

    fn offline_connection(path: &str) -> Connection {
        Connection {
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory: PathBuf::from(
                    "/home/example/.local/share/cosmic-mounter/recovery/google",
                ),
                ..OfflineMirrorConfig::default()
            }),
            ..online_connection(path)
        }
    }

    fn storage(temp: &TempDir, version: u64) -> cosmic_config::Config {
        cosmic_config::Config::with_custom_path(APP_ID, version, temp.path().to_path_buf())
            .expect("create isolated config")
    }

    #[test]
    fn uuid_identity_round_trips_without_changing() {
        let id = ConnectionId::new();
        let encoded = ron::to_string(&id).expect("serialize UUID");
        let decoded: ConnectionId = ron::from_str(&encoded).expect("deserialize UUID");
        assert_eq!(decoded, id);
        assert_eq!(decoded.as_uuid(), id.as_uuid());
    }

    #[test]
    fn valid_fixture_deserializes_and_validates_without_secrets() {
        let fixture = include_str!("../tests/fixtures/config-v2-valid.ron");
        let document: ConfigDocument = ron::from_str(fixture).expect("valid fixture");
        let config = Config { document };
        assert_eq!(config.document.connections[0].id, connection_id());
        assert!(config.validate().is_ok());

        let lower = fixture.to_ascii_lowercase();
        for secret_name in ["password", "token", "secret", "credential"] {
            assert!(!lower.contains(secret_name));
        }
    }

    #[test]
    fn fresh_storage_loads_defaults_without_warning() {
        let temp = TempDir::new().expect("temporary directory");
        let report = Config::load_from(
            &storage(&temp, Config::VERSION),
            &storage(&temp, LegacyConfigV1::VERSION),
        );
        assert_eq!(report.source, LoadSource::Fresh);
        assert!(report.warnings.is_empty());
        assert_eq!(report.config, Config::default());
    }

    #[test]
    fn version_one_configuration_is_migrated_and_persisted() {
        let temp = TempDir::new().expect("temporary directory");
        let current = storage(&temp, Config::VERSION);
        let legacy_storage = storage(&temp, LegacyConfigV1::VERSION);
        LegacyConfigV1 {
            notifications_enabled: false,
        }
        .write_entry(&legacy_storage)
        .expect("write legacy configuration");

        let report = Config::load_from(&current, &legacy_storage);
        assert_eq!(report.source, LoadSource::MigratedV1);
        assert!(!report.config.notifications_enabled());
        assert_eq!(
            Config::get_entry(&current).expect("persisted migration"),
            report.config
        );
    }

    #[test]
    fn malformed_current_configuration_recovers_defaults() {
        let temp = TempDir::new().expect("temporary directory");
        let current = storage(&temp, Config::VERSION);
        let legacy = storage(&temp, LegacyConfigV1::VERSION);
        let path = temp
            .path()
            .join("cosmic")
            .join(APP_ID)
            .join(format!("v{}", Config::VERSION))
            .join("document");
        fs::write(path, "this is not ron").expect("write malformed fixture");

        let report = Config::load_from(&current, &legacy);
        assert_eq!(report.source, LoadSource::RecoveredDefaults);
        assert_eq!(report.config, Config::default());
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn failed_update_restores_last_valid_configuration() {
        let temp = TempDir::new().expect("temporary directory");
        let storage = storage(&temp, Config::VERSION);
        let mut config = Config::default();
        config.write_validated(&storage).expect("write defaults");

        let result = config.update_validated(&storage, |document| {
            document.schema_version = 99;
        });
        assert!(matches!(result, Err(SaveError::Validation(_))));
        assert_eq!(config, Config::default());
        assert_eq!(
            Config::get_entry(&storage).expect("read stored defaults"),
            Config::default()
        );
    }

    #[test]
    fn host_visible_storage_loads_native_cosmic_document() {
        let temp = TempDir::new().expect("temporary directory");
        let document_path = temp
            .path()
            .join(".config")
            .join("cosmic")
            .join(APP_ID)
            .join(format!("v{}", Config::VERSION))
            .join("document");
        fs::create_dir_all(document_path.parent().expect("parent")).expect("create parent");
        let expected = ConfigDocument {
            notifications_enabled: false,
            ..ConfigDocument::default()
        };
        fs::write(
            &document_path,
            ron::ser::to_string_pretty(&expected, ron::ser::PrettyConfig::new())
                .expect("serialize"),
        )
        .expect("write document");

        let storage = HostVisibleConfigStorage::new(document_path);
        let report = storage.load();

        assert_eq!(report.source, LoadSource::Current);
        assert_eq!(report.config.document, expected);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn host_visible_storage_writes_validated_document_atomically() {
        let temp = TempDir::new().expect("temporary directory");
        let document_path = temp
            .path()
            .join(".config")
            .join("cosmic")
            .join(APP_ID)
            .join(format!("v{}", Config::VERSION))
            .join("document");
        let storage =
            AppConfigStorage::HostVisible(HostVisibleConfigStorage::new(document_path.clone()));
        let mut config = Config::default();

        let changed = config
            .update_validated_with(&storage, |document| {
                document.notifications_enabled = false;
            })
            .expect("write host-visible config");

        assert!(changed);
        assert!(!document_path.with_extension("tmp").exists());
        let persisted: ConfigDocument =
            ron::from_str(&fs::read_to_string(document_path).expect("read document"))
                .expect("parse document");
        assert!(!persisted.notifications_enabled);
    }

    #[test]
    fn duplicate_and_nested_targets_are_rejected_across_modes() {
        let mut first = online_connection("/home/example/Cloud");
        let mut second = offline_connection("/home/example/Cloud/Google");
        second.id = ConnectionId::new();
        first.name = "Online".into();
        second.name = "Offline".into();
        let config = Config {
            document: ConfigDocument {
                connections: vec![first, second],
                ..ConfigDocument::default()
            },
        };

        let errors = config.validate().expect_err("nested paths must fail");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::ConflictingTargets { .. }))
        );
    }

    #[test]
    fn duplicate_connection_and_vpn_ids_are_rejected() {
        let first_connection = online_connection("/home/example/Cloud/One");
        let mut second_connection = online_connection("/home/example/Cloud/Two");
        second_connection.name = "Second mount".into();

        let first_vpn = VpnProfile {
            id: vpn_id(),
            name: "First VPN".into(),
            kind: VpnKind::NetworkManager,
            external_profile_id: Some("first".into()),
            readiness_checks: vec![ReadinessCheck::NetworkManagerState],
            timeout_seconds: 30,
        };
        let mut second_vpn = first_vpn.clone();
        second_vpn.name = "Second VPN".into();

        let config = Config {
            document: ConfigDocument {
                connections: vec![first_connection, second_connection],
                vpn_profiles: vec![first_vpn, second_vpn],
                ..ConfigDocument::default()
            },
        };
        let errors = config.validate().expect_err("duplicate IDs must fail");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::DuplicateConnectionId(_)))
        );
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::DuplicateVpnProfileId(_)))
        );
    }

    #[test]
    fn unsafe_relative_and_system_targets_are_rejected() {
        for path in [
            "relative/path",
            "/etc/cloud",
            "/home/example/../../usr/data",
        ] {
            let config = Config {
                document: ConfigDocument {
                    connections: vec![online_connection(path)],
                    ..ConfigDocument::default()
                },
            };
            assert!(config.validate().is_err(), "{path} should be rejected");
        }
    }

    #[test]
    fn cache_and_recovery_directories_cannot_overlap_visible_tree() {
        let mut online = online_connection("/home/example/Cloud/Online");
        if let ConnectionMode::OnlineMount(options) = &mut online.mode {
            options.cache_directory = Some(PathBuf::from("/home/example/Cloud/Online/.cache"));
        }
        let mut offline = offline_connection("/home/example/Cloud/Offline");
        offline.id = ConnectionId::new();
        if let ConnectionMode::OfflineMirror(options) = &mut offline.mode {
            options.recovery_directory = PathBuf::from("/home/example/Cloud/Offline/.recovery");
        }
        let config = Config {
            document: ConfigDocument {
                connections: vec![online, offline],
                ..ConfigDocument::default()
            },
        };
        let errors = config
            .validate()
            .expect_err("work directories must be outside");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidCacheDirectory { .. }))
        );
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidRecoveryDirectory { .. }))
        );
    }

    #[test]
    fn vpn_references_must_resolve_and_values_must_be_bounded() {
        let mut connection = online_connection("/home/example/Cloud/Online");
        connection.vpn_profile_id = Some(vpn_id());
        let config = Config {
            document: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
        };
        assert!(config.validate().expect_err("missing VPN").iter().any(
            |error| matches!(error, ValidationError::MissingVpnProfile(id) if *id == vpn_id())
        ));

        let profile = VpnProfile {
            id: vpn_id(),
            name: "Work VPN".into(),
            kind: VpnKind::NetworkManager,
            external_profile_id: Some("work-vpn".into()),
            readiness_checks: vec![ReadinessCheck::Interface("tun0".into())],
            timeout_seconds: 30,
        };
        let mut valid = config;
        valid.document.vpn_profiles.push(profile);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn invalid_remote_and_sync_values_are_rejected() {
        let mut connection = offline_connection("/home/example/Cloud/Offline");
        connection.remote_reference = "\n".into();
        connection.remote_subpath = Some("../outside".into());
        if let ConnectionMode::OfflineMirror(options) = &mut connection.mode {
            options.sync_interval_minutes = 0;
        }
        let config = Config {
            document: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
        };
        let errors = config.validate().expect_err("invalid remote values");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidRemoteReference(_)))
        );
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidRemoteSubpath(_)))
        );
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidSyncInterval(_)))
        );
    }
}
