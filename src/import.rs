// SPDX-License-Identifier: MIT

//! Structured discovery, parsing, and preview of compatible legacy units.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Connection, ConnectionId, ConnectionMode, OnlineMountConfig, Provider};
use crate::providers::{onedriver_mount_plan, rclone_mount_plan};
use crate::services::UnitDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    ReadDirectory(String),
    ReadUnit(PathBuf, String),
    Malformed(String),
    Unsupported(String),
    Conflict(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory(error) => write!(formatter, "failed to scan units: {error}"),
            Self::ReadUnit(path, error) => {
                write!(formatter, "failed to read {}: {error}", path.display())
            }
            Self::Malformed(error) => write!(formatter, "malformed unit: {error}"),
            Self::Unsupported(error) => write!(formatter, "unsupported unit: {error}"),
            Self::Conflict(error) => write!(formatter, "conflicting import: {error}"),
        }
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyEngine {
    RcloneMount,
    Onedriver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyUnit {
    pub path: PathBuf,
    pub name: String,
    pub description: Option<String>,
    pub install_wanted_by: Vec<String>,
    pub exec_start: Vec<String>,
    pub unsupported_options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPreview {
    pub original_path: PathBuf,
    pub original_unit_name: String,
    pub engine: LegacyEngine,
    pub provider: Provider,
    pub remote_reference: String,
    pub remote_subpath: Option<String>,
    pub local_target: PathBuf,
    pub cache_directory: Option<PathBuf>,
    pub start_at_login: bool,
    pub unsupported_options: Vec<String>,
    pub active_conflict: bool,
    pub local_target_conflict: bool,
    pub connection: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReplacementPlan {
    pub preview: ImportPreview,
    pub managed_service: UnitDocument,
    pub preserve_original: bool,
    pub disable_original: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedLegacyConnection {
    provider: Provider,
    remote_reference: String,
    remote_subpath: Option<String>,
    local_target: PathBuf,
    cache_directory: Option<PathBuf>,
    unsupported_options: Vec<String>,
}

#[must_use]
pub fn default_scan_directory(home: &Path) -> PathBuf {
    home.join(".config/systemd/user")
}

pub fn scan_legacy_units(directory: &Path) -> Result<Vec<LegacyUnit>, ImportError> {
    let mut units = Vec::new();
    let entries =
        fs::read_dir(directory).map_err(|error| ImportError::ReadDirectory(error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| ImportError::ReadDirectory(error.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("service") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| ImportError::ReadUnit(path.clone(), error.to_string()))?;
        if let Ok(unit) = parse_unit(&path, &content)
            && classify_unit(&unit).is_ok()
        {
            units.push(unit);
        }
    }
    units.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(units)
}

pub fn parse_unit(path: &Path, content: &str) -> Result<LegacyUnit, ImportError> {
    let mut section = String::new();
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for logical in logical_lines(content) {
        let line = logical.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_matches(['[', ']']).to_owned();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(ImportError::Malformed(format!("line lacks '=': {line}")));
        };
        values
            .entry(format!("{section}.{key}"))
            .or_default()
            .push(value.trim().to_owned());
    }
    let exec_start = values
        .get("Service.ExecStart")
        .and_then(|values| values.first())
        .ok_or_else(|| ImportError::Unsupported("missing ExecStart".into()))
        .and_then(|value| tokenize_systemd_exec(value))?;
    Ok(LegacyUnit {
        path: path.to_path_buf(),
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned(),
        description: values
            .get("Unit.Description")
            .and_then(|values| values.first())
            .cloned(),
        install_wanted_by: values.get("Install.WantedBy").cloned().unwrap_or_default(),
        exec_start,
        unsupported_options: unsupported_service_options(&values),
    })
}

pub fn preview_import(
    unit: &LegacyUnit,
    existing_connections: &[Connection],
    active_units: &BTreeSet<String>,
    home: &Path,
) -> Result<ImportPreview, ImportError> {
    let engine = classify_unit(unit)?;
    let start_at_login = unit
        .install_wanted_by
        .iter()
        .any(|value| value == "default.target");
    let parsed = match engine {
        LegacyEngine::RcloneMount => parse_rclone_mount(unit, home)?,
        LegacyEngine::Onedriver => parse_onedriver(unit, home)?,
    };
    let local_target_conflict = existing_connections
        .iter()
        .any(|connection| paths_overlap(&connection.local_path, &parsed.local_target));
    let active_conflict = active_units.contains(&unit.name);
    let connection = Connection {
        id: ConnectionId::new(),
        name: unit
            .description
            .clone()
            .unwrap_or_else(|| unit.name.trim_end_matches(".service").replace('-', " ")),
        provider: parsed.provider,
        mode: ConnectionMode::OnlineMount(OnlineMountConfig {
            cache_directory: parsed.cache_directory.clone(),
            start_at_login,
            ..OnlineMountConfig::default()
        }),
        remote_reference: parsed.remote_reference,
        remote_subpath: parsed.remote_subpath,
        local_path: parsed.local_target,
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: crate::model::TuningProfile::Balanced,
    };
    Ok(ImportPreview {
        original_path: unit.path.clone(),
        original_unit_name: unit.name.clone(),
        engine,
        provider: connection.provider,
        remote_reference: connection.remote_reference.clone(),
        remote_subpath: connection.remote_subpath.clone(),
        local_target: connection.local_path.clone(),
        cache_directory: parsed.cache_directory,
        start_at_login,
        unsupported_options: parsed.unsupported_options,
        active_conflict,
        local_target_conflict,
        connection,
    })
}

pub fn replacement_plan(
    preview: ImportPreview,
    confirmed: bool,
    preserve_original: bool,
    disable_original: bool,
    runtime_directory: &Path,
    default_cache_root: &Path,
    default_config_root: &Path,
) -> Result<ImportReplacementPlan, ImportError> {
    if !confirmed {
        return Err(ImportError::Conflict("import requires confirmation".into()));
    }
    if preview.active_conflict || preview.local_target_conflict {
        return Err(ImportError::Conflict(
            "active service or local target conflict must be resolved first".into(),
        ));
    }
    let service = match preview.engine {
        LegacyEngine::RcloneMount => {
            rclone_mount_plan(&preview.connection, runtime_directory, default_cache_root)
                .map_err(|error| ImportError::Unsupported(error.to_string()))?
                .service
        }
        LegacyEngine::Onedriver => {
            onedriver_mount_plan(&preview.connection, default_cache_root, default_config_root)
                .map_err(|error| ImportError::Unsupported(error.to_string()))?
                .service
        }
    };
    Ok(ImportReplacementPlan {
        preview,
        managed_service: UnitDocument::service(&service)
            .map_err(|error| ImportError::Malformed(error.to_string()))?,
        preserve_original,
        disable_original,
    })
}

fn classify_unit(unit: &LegacyUnit) -> Result<LegacyEngine, ImportError> {
    let Some(executable) = unit.exec_start.first() else {
        return Err(ImportError::Unsupported("empty ExecStart".into()));
    };
    let binary = Path::new(executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable);
    match binary {
        "rclone" if unit.exec_start.get(1).is_some_and(|value| value == "mount") => {
            Ok(LegacyEngine::RcloneMount)
        }
        "onedriver" => Ok(LegacyEngine::Onedriver),
        _ => Err(ImportError::Unsupported(format!(
            "unsupported executable {executable}"
        ))),
    }
}

fn parse_rclone_mount(
    unit: &LegacyUnit,
    home: &Path,
) -> Result<ParsedLegacyConnection, ImportError> {
    if unit.exec_start.len() < 4 {
        return Err(ImportError::Malformed(
            "rclone mount requires remote and target".into(),
        ));
    }
    let remote = &unit.exec_start[2];
    let target = expand_home(&unit.exec_start[3], home);
    let (remote_reference, remote_subpath) = parse_remote(remote)?;
    let provider = infer_provider(&remote_reference);
    let mut cache_directory = None;
    let mut unsupported = unit.unsupported_options.clone();
    let mut index = 4;
    while index < unit.exec_start.len() {
        let option = &unit.exec_start[index];
        match option.as_str() {
            "--cache-dir" => {
                if let Some(value) = unit.exec_start.get(index + 1) {
                    cache_directory = Some(expand_home(value, home));
                    index += 2;
                } else {
                    unsupported.push(option.clone());
                    index += 1;
                }
            }
            "--config"
            | "--vfs-cache-mode"
            | "--vfs-cache-max-age"
            | "--vfs-cache-max-size"
            | "--vfs-cache-poll-interval"
            | "--dir-cache-time"
            | "--timeout"
            | "--contimeout"
            | "--low-level-retries"
            | "--retries"
            | "--retries-sleep"
            | "--umask"
            | "--log-level"
            | "--poll-interval"
            | "--log-file" => {
                index += if unit
                    .exec_start
                    .get(index + 1)
                    .is_some_and(|value| !value.starts_with("--"))
                {
                    2
                } else {
                    1
                };
            }
            value if value.starts_with("--") => {
                unsupported.push(value.to_owned());
                index += if unit
                    .exec_start
                    .get(index + 1)
                    .is_some_and(|next| !next.starts_with("--"))
                {
                    2
                } else {
                    1
                };
            }
            value => {
                unsupported.push(value.to_owned());
                index += 1;
            }
        }
    }
    Ok(ParsedLegacyConnection {
        provider,
        remote_reference,
        remote_subpath,
        local_target: target,
        cache_directory,
        unsupported_options: unsupported,
    })
}

fn parse_onedriver(unit: &LegacyUnit, home: &Path) -> Result<ParsedLegacyConnection, ImportError> {
    let mut cache_directory = None;
    let mut target = None;
    let mut unsupported = unit.unsupported_options.clone();
    let mut index = 1;
    while index < unit.exec_start.len() {
        match unit.exec_start[index].as_str() {
            "--cache-dir" => {
                if let Some(value) = unit.exec_start.get(index + 1) {
                    cache_directory = Some(expand_home(value, home));
                    index += 2;
                } else {
                    unsupported.push("--cache-dir".into());
                    index += 1;
                }
            }
            "--config-file" | "--auth-only" => {
                index += if unit
                    .exec_start
                    .get(index + 1)
                    .is_some_and(|value| !value.starts_with("--"))
                {
                    2
                } else {
                    1
                };
            }
            value if value.starts_with("--") => {
                unsupported.push(value.to_owned());
                index += if unit
                    .exec_start
                    .get(index + 1)
                    .is_some_and(|next| !next.starts_with("--"))
                {
                    2
                } else {
                    1
                };
            }
            value => {
                target = Some(expand_home(value, home));
                index += 1;
            }
        }
    }
    let target = target.ok_or_else(|| ImportError::Malformed("onedriver target missing".into()))?;
    Ok(ParsedLegacyConnection {
        provider: Provider::OneDrive,
        remote_reference: "onedrive".into(),
        remote_subpath: None,
        local_target: target,
        cache_directory,
        unsupported_options: unsupported,
    })
}

fn parse_remote(value: &str) -> Result<(String, Option<String>), ImportError> {
    let Some((reference, subpath)) = value.split_once(':') else {
        return Err(ImportError::Malformed("rclone remote lacks ':'".into()));
    };
    if reference.is_empty() || reference.chars().any(char::is_control) {
        return Err(ImportError::Malformed("invalid remote reference".into()));
    }
    let subpath = (!subpath.is_empty()).then(|| subpath.to_owned());
    Ok((reference.to_owned(), subpath))
}

fn infer_provider(remote_reference: &str) -> Provider {
    let lower = remote_reference.to_ascii_lowercase();
    if lower.contains("box") {
        Provider::Box
    } else if lower.contains("smb") || lower.contains("engr") || lower.contains("share") {
        Provider::Smb
    } else {
        Provider::GoogleDrive
    }
}

fn logical_lines(content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for raw in content.lines() {
        let line = raw.trim_end();
        let continued = line.ends_with('\\');
        let part = line.trim_end_matches('\\').trim_end();
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(part);
        if !continued {
            lines.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn tokenize_systemd_exec(value: &str) -> Result<Vec<String>, ImportError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            character if character.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            character => current.push(character),
        }
    }
    if quote.is_some() || escaped {
        return Err(ImportError::Malformed(
            "unterminated quoting or escape".into(),
        ));
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    if tokens
        .iter()
        .any(|token| token.contains('\0') || token.contains('\n') || token.contains('\r'))
    {
        return Err(ImportError::Malformed("unsafe control character".into()));
    }
    Ok(tokens)
}

fn unsupported_service_options(values: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let supported = [
        "Unit.Description",
        "Unit.Documentation",
        "Unit.After",
        "Unit.Wants",
        "Service.Type",
        "Service.ExecStart",
        "Service.ExecStartPre",
        "Service.ExecStop",
        "Service.ExecStopPost",
        "Service.Restart",
        "Service.RestartSec",
        "Install.WantedBy",
    ];
    values
        .keys()
        .filter(|key| !supported.contains(&key.as_str()))
        .cloned()
        .collect()
}

fn expand_home(value: &str, home: &Path) -> PathBuf {
    if value == "%h" {
        home.to_path_buf()
    } else if let Some(rest) = value.strip_prefix("%h/") {
        home.join(rest)
    } else {
        PathBuf::from(value)
    }
}

fn paths_overlap(first: &Path, second: &Path) -> bool {
    first == second || first.starts_with(second) || second.starts_with(first)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    const HOME: &str = "/home/example";
    const FIXTURES: &[(&str, &str)] = &[
        (
            "rclone-ua-box.service",
            include_str!("../archive/services/rclone-ua-box.service"),
        ),
        (
            "rclone-ua-engr.service",
            include_str!("../archive/services/rclone-ua-engr.service"),
        ),
        (
            "rclone-ua-gdrive.service",
            include_str!("../archive/services/rclone-ua-gdrive.service"),
        ),
        (
            "rclone-uutzinger-gdrive.service",
            include_str!("../archive/services/rclone-uutzinger-gdrive.service"),
        ),
    ];

    #[test]
    fn parses_all_archived_rclone_services_without_mutating_fixtures() {
        for (name, content) in FIXTURES {
            let before = *content;
            let unit = parse_unit(Path::new(name), content).expect("unit");
            let preview =
                preview_import(&unit, &[], &BTreeSet::new(), Path::new(HOME)).expect("preview");
            assert_eq!(before, *content);
            assert_eq!(preview.engine, LegacyEngine::RcloneMount);
            assert!(preview.local_target.is_absolute());
            assert!(
                preview
                    .cache_directory
                    .as_ref()
                    .is_some_and(|path| path.is_absolute())
            );
            assert!(preview.start_at_login);
        }
    }

    #[test]
    fn rclone_fixture_preview_contains_provider_remote_target_and_cache() {
        let unit = parse_unit(
            Path::new("rclone-ua-engr.service"),
            include_str!("../archive/services/rclone-ua-engr.service"),
        )
        .expect("unit");
        let preview = preview_import(&unit, &[], &BTreeSet::new(), Path::new("/home/uutzinger"))
            .expect("preview");
        assert_eq!(preview.provider, Provider::Smb);
        assert_eq!(preview.remote_reference, "ua_engr");
        assert_eq!(preview.remote_subpath, Some("Research".into()));
        assert_eq!(
            preview.local_target,
            PathBuf::from("/home/uutzinger/Cloud/UA_ENGR")
        );
        assert_eq!(
            preview.cache_directory,
            Some(PathBuf::from("/home/uutzinger/.cache/rclone-ua-engr"))
        );
        assert!(preview.unsupported_options.is_empty());
    }

    #[test]
    fn onedriver_preview_is_supported() {
        let content = "[Unit]\nDescription=OneDrive legacy\n\n[Service]\nExecStart=/usr/bin/onedriver --config-file %h/.config/onedriver/config.json --cache-dir %h/.cache/onedriver %h/Cloud/OneDrive\n\n[Install]\nWantedBy=default.target\n";
        let unit = parse_unit(Path::new("onedriver.service"), content).expect("unit");
        let preview =
            preview_import(&unit, &[], &BTreeSet::new(), Path::new(HOME)).expect("preview");
        assert_eq!(preview.engine, LegacyEngine::Onedriver);
        assert_eq!(preview.provider, Provider::OneDrive);
        assert_eq!(
            preview.local_target,
            PathBuf::from("/home/example/Cloud/OneDrive")
        );
        assert_eq!(
            preview.cache_directory,
            Some(PathBuf::from("/home/example/.cache/onedriver"))
        );
    }

    #[test]
    fn import_replacement_requires_confirmation_and_preserves_original() {
        let unit = parse_unit(
            Path::new("rclone-ua-box.service"),
            include_str!("../archive/services/rclone-ua-box.service"),
        )
        .expect("unit");
        let preview =
            preview_import(&unit, &[], &BTreeSet::new(), Path::new(HOME)).expect("preview");
        assert!(
            replacement_plan(
                preview.clone(),
                false,
                true,
                false,
                Path::new("/run/user/1000/cosmic-mounter"),
                Path::new("/home/example/.cache/cosmic-mounter"),
                Path::new("/home/example/.config/cosmic-mounter"),
            )
            .is_err()
        );
        let plan = replacement_plan(
            preview,
            true,
            true,
            false,
            Path::new("/run/user/1000/cosmic-mounter"),
            Path::new("/home/example/.cache/cosmic-mounter"),
            Path::new("/home/example/.config/cosmic-mounter"),
        )
        .expect("plan");
        assert!(plan.preserve_original);
        assert!(!plan.disable_original);
        assert!(
            plan.managed_service
                .content
                .contains("# X-Cosmic-Mounter-Managed=true")
        );
    }

    #[test]
    fn active_and_target_conflicts_are_reported_and_block_replacement() {
        let unit = parse_unit(
            Path::new("rclone-ua-gdrive.service"),
            include_str!("../archive/services/rclone-ua-gdrive.service"),
        )
        .expect("unit");
        let preview = preview_import(
            &unit,
            &[Connection {
                id: ConnectionId::new(),
                name: "Existing".into(),
                provider: Provider::GoogleDrive,
                mode: ConnectionMode::OnlineMount(OnlineMountConfig::default()),
                remote_reference: "ua_gdrive".into(),
                remote_subpath: None,
                local_path: PathBuf::from("/home/example/Cloud/UA_GoogleDrive"),
                enabled: true,
                vpn_profile_id: None,
                disconnect_vpn_when_unused: false,
                tuning_profile: crate::model::TuningProfile::Balanced,
            }],
            &["rclone-ua-gdrive.service".to_owned()]
                .into_iter()
                .collect(),
            Path::new(HOME),
        )
        .expect("preview");
        assert!(preview.active_conflict);
        assert!(preview.local_target_conflict);
        assert!(
            replacement_plan(
                preview,
                true,
                true,
                false,
                Path::new("/run/user/1000/cosmic-mounter"),
                Path::new("/home/example/.cache/cosmic-mounter"),
                Path::new("/home/example/.config/cosmic-mounter"),
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_and_injection_units_do_not_execute_or_import() {
        let content = "[Service]\nExecStart=/bin/sh -c 'touch /tmp/owned'\n";
        let unit = parse_unit(Path::new("bad.service"), content).expect("parse only");
        assert!(matches!(
            classify_unit(&unit),
            Err(ImportError::Unsupported(_))
        ));
        assert!(
            parse_unit(
                Path::new("bad.service"),
                "[Service]\nExecStart=\"/usr/bin/rclone mount\n"
            )
            .is_err()
        );
    }

    #[test]
    fn unsupported_options_are_reported() {
        let content = "[Service]\nExecStart=/usr/bin/rclone mount remote: %h/Cloud --evil value --cache-dir %h/.cache/rclone\nEnvironment=SECRET=hidden\n";
        let unit = parse_unit(Path::new("unsupported.service"), content).expect("unit");
        let preview =
            preview_import(&unit, &[], &BTreeSet::new(), Path::new(HOME)).expect("preview");
        assert!(preview.unsupported_options.contains(&"--evil".to_owned()));
        assert!(
            preview
                .unsupported_options
                .contains(&"Service.Environment".to_owned())
        );
    }

    #[test]
    fn scans_service_directory_and_ignores_non_importable_files() {
        let temp = TempDir::new().expect("temp");
        fs::write(
            temp.path().join("rclone.service"),
            include_str!("../archive/services/rclone-ua-box.service"),
        )
        .expect("fixture");
        fs::write(temp.path().join("notes.txt"), "ignore").expect("notes");
        fs::write(
            temp.path().join("other.service"),
            "[Service]\nExecStart=/bin/true\n",
        )
        .expect("other");
        let units = scan_legacy_units(temp.path()).expect("scan");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].name, "rclone.service");
    }
}
