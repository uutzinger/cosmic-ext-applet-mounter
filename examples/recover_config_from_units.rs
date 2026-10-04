// SPDX-License-Identifier: MIT
//
// Audit/recovery helper. It never writes the live applet configuration.
// Example: cargo run --example recover_config_from_units -- \
//   --baseline /path/to/document.backup --output /tmp/document.candidate

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic_ext_applet_mounter::config::{APP_ID, Config, ConfigDocument};
use cosmic_ext_applet_mounter::import::{LegacyUnit, parse_rclone_backends, preview_import};
use cosmic_ext_applet_mounter::model::{
    Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TeamsLibraryIdentity,
    TuningProfile,
};

#[derive(Default)]
struct Options {
    units_dir: Option<PathBuf>,
    baseline: Option<PathBuf>,
    no_baseline: bool,
    output: Option<PathBuf>,
    raw_output: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Recovery audit failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_options()?;
    let home = env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    let units_dir = options
        .units_dir
        .unwrap_or_else(|| home.join(".config/systemd/user"));
    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let live_document = config_home
        .join("cosmic")
        .join(APP_ID)
        .join(format!("v{}", Config::VERSION))
        .join("document");
    let baseline_path = if options.no_baseline {
        None
    } else {
        options
            .baseline
            .or_else(|| live_document.exists().then_some(live_document.clone()))
    };
    let baseline = baseline_path
        .as_ref()
        .map(|path| load_document(path))
        .transpose()?;

    let rclone_output = Command::new("rclone")
        .args(["config", "dump"])
        .output()
        .map_err(|error| format!("cannot inspect rclone remote types: {error}"))?;
    if !rclone_output.status.success() {
        return Err("rclone config dump failed; no candidate was written".into());
    }
    let rclone_dump = String::from_utf8_lossy(&rclone_output.stdout);
    let backends = parse_rclone_backends(&rclone_dump).map_err(|error| error.to_string())?;
    let rclone_types = serde_json::from_str::<serde_json::Value>(&rclone_dump)
        .map_err(|_| "rclone config dump returned invalid JSON".to_owned())?;

    let mut connections = Vec::new();
    let mut warnings = Vec::new();

    let mut paths = fs::read_dir(&units_dir)
        .map_err(|error| format!("cannot read {}: {error}", units_dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot enumerate {}: {error}", units_dir.display()))?;
    paths.sort();
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("service") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        if !name.starts_with("cosmic-mounter-") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if !content
            .lines()
            .any(|line| line.trim() == "# X-Cosmic-Mounter-Managed=true")
        {
            continue;
        }
        let unit = match cosmic_ext_applet_mounter::import::parse_unit(&path, &content) {
            Ok(unit) => unit,
            Err(error) => {
                warnings.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        let Some(original_id) = connection_id_from_service_name(&unit.name) else {
            warnings.push(format!("{} has no valid connection ID", unit.name));
            continue;
        };
        let mut unit_backends = backends.clone();
        if content.contains("--verify-teams-mount")
            && let Some(remote) = unit
                .exec_start
                .get(2)
                .and_then(|value| value.split_once(':'))
            && rclone_types[remote.0]["type"].as_str() == Some("onedrive")
        {
            unit_backends.insert(remote.0.to_owned(), Provider::Teams);
        }

        if let Ok(preview) = preview_import(&unit, &[], &BTreeSet::new(), &home, &unit_backends) {
            let mut connection = preview.connection;
            connection.id = original_id;
            connection.name = clean_description(&connection.name, "Cloud Mounter:");
            // WantedBy describes eligibility, not whether the unit is enabled.
            if let ConnectionMode::OnlineMount(options) = &mut connection.mode {
                options.start_at_login = unit_is_enabled(&units_dir, &unit.name);
            }
            if connection.provider == Provider::OneDrive {
                connection.remote_reference = format!("onedriver-{}", short_id(connection.id));
            }
            if connection.provider == Provider::Teams
                && let Some(identity) = read_teams_mount_identity(&unit)
            {
                connection.teams_identity = Some(identity);
            }
            if let Err(error) = validate_recovered_connection(&connection) {
                warnings.push(format!("{}: {error}", unit.name));
                continue;
            }
            connections.push(connection);
            continue;
        }

        if let Some(connection) = recover_managed_unit(&unit, &home) {
            if let Err(error) = validate_recovered_connection(&connection) {
                warnings.push(format!("{}: {error}", unit.name));
            } else {
                connections.push(connection);
            }
        } else {
            warnings.push(format!("Could not recover managed unit: {name}"));
        }
    }
    let mut seen = BTreeSet::new();
    connections.retain(|connection| {
        if seen.insert(connection.id) {
            true
        } else {
            warnings.push(format!("Duplicate unit ID {} was ignored", connection.id));
            false
        }
    });

    let raw = ConfigDocument {
        connections: connections.clone(),
        ..ConfigDocument::default()
    };
    validate_document(&raw, "unit-only candidate")?;

    let mut merged = baseline.clone().unwrap_or_default();
    let baseline_ids = merged
        .connections
        .iter()
        .map(|connection| connection.id)
        .collect::<BTreeSet<_>>();
    for connection in connections
        .iter()
        .filter(|item| !baseline_ids.contains(&item.id))
    {
        warnings.push(format!(
            "Unit-only connection {} ({}) added to candidate",
            connection.name, connection.id
        ));
        merged.connections.push(connection.clone());
    }
    validate_document(&merged, "merged candidate")?;
    if let Some(path) = options.raw_output.as_ref() {
        write_candidate(path, &live_document, &raw)?;
    }
    if let Some(path) = options.output.as_ref() {
        write_candidate(path, &live_document, &merged)?;
    }

    println!("Managed units recovered: {}", connections.len());
    println!("Merged candidate connections: {}", merged.connections.len());
    println!(
        "Baseline: {}",
        baseline_path
            .as_ref()
            .map_or_else(|| "none".into(), |path| path.display().to_string())
    );
    print_comparison(baseline.as_ref(), &connections);
    for warning in warnings {
        eprintln!("Warning: {warning}");
    }
    if options.output.is_none() && options.raw_output.is_none() {
        println!(
            "Dry run only. Use --output and/or --raw-output to write new candidate files; live config is never changed."
        );
    }
    Ok(())
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        let value = match argument.as_str() {
            "--units-dir" | "--baseline" | "--output" | "--raw-output" => Some(PathBuf::from(
                args.next()
                    .ok_or_else(|| format!("{argument} requires a path"))?,
            )),
            "--no-baseline" => {
                options.no_baseline = true;
                None
            }
            "--help" => {
                println!(
                    "Usage: recover_config_from_units [--units-dir PATH] [--baseline PATH | --no-baseline] [--output NEW_PATH] [--raw-output NEW_PATH]"
                );
                println!(
                    "All writes use create-new files. The live configuration is never modified."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown option: {argument}")),
        };
        match argument.as_str() {
            "--units-dir" => options.units_dir = value,
            "--baseline" => options.baseline = value,
            "--output" => options.output = value,
            "--raw-output" => options.raw_output = value,
            _ => {}
        }
    }
    if options.no_baseline && options.baseline.is_some() {
        return Err("--baseline and --no-baseline cannot be combined".into());
    }
    if options.output.is_some() && options.output == options.raw_output {
        return Err("--output and --raw-output must be different paths".into());
    }
    Ok(options)
}

fn load_document(path: &Path) -> Result<ConfigDocument, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read baseline {}: {error}", path.display()))?;
    let document: ConfigDocument = ron::from_str(&text)
        .map_err(|error| format!("cannot parse baseline {}: {error}", path.display()))?;
    validate_document(&document, "baseline")?;
    Ok(document)
}

fn validate_document(document: &ConfigDocument, label: &str) -> Result<(), String> {
    Config {
        document: document.clone(),
    }
    .validate()
    .map_err(|errors| format!("{label} failed validation: {errors:?}"))
}

fn validate_recovered_connection(connection: &Connection) -> Result<(), String> {
    let document = ConfigDocument {
        connections: vec![connection.clone()],
        ..ConfigDocument::default()
    };
    validate_document(&document, "recovered connection")
}

fn write_candidate(
    path: &Path,
    live_document: &Path,
    document: &ConfigDocument,
) -> Result<(), String> {
    if path == live_document
        || path
            .canonicalize()
            .is_ok_and(|actual| actual == live_document)
    {
        return Err("refusing to write the live applet document".into());
    }
    let text = ron::ser::to_string_pretty(document, ron::ser::PrettyConfig::new())
        .map_err(|error| format!("cannot serialize candidate: {error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("cannot create candidate {}: {error}", path.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("cannot write candidate {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("cannot sync candidate {}: {error}", path.display()))?;
    println!("Wrote candidate: {}", path.display());
    Ok(())
}

fn unit_is_enabled(units_dir: &Path, unit_name: &str) -> bool {
    units_dir
        .join("default.target.wants")
        .join(unit_name)
        .exists()
}

fn short_id(id: ConnectionId) -> String {
    id.to_string()
        .split('-')
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn print_comparison(baseline: Option<&ConfigDocument>, recovered: &[Connection]) {
    let Some(baseline) = baseline else {
        println!(
            "No baseline: global settings, VPN profiles, names, and per-connection preferences cannot be recovered from units."
        );
        return;
    };
    let by_id = recovered
        .iter()
        .map(|connection| (connection.id, connection))
        .collect::<BTreeMap<_, _>>();
    for current in &baseline.connections {
        let Some(unit) = by_id.get(&current.id) else {
            println!(
                "Baseline-only connection: {} ({})",
                current.name, current.id
            );
            continue;
        };
        let mut differences = Vec::new();
        if current.name != unit.name {
            differences.push("name");
        }
        if current.provider != unit.provider || current.mode.kind() != unit.mode.kind() {
            differences.push("provider/mode");
        }
        if current.local_path != unit.local_path {
            differences.push("local target");
        }
        if current.remote_reference != unit.remote_reference {
            differences.push("remote/account label");
        }
        if current.remote_subpath != unit.remote_subpath {
            differences.push("remote subpath");
        }
        if current.teams_identity != unit.teams_identity {
            differences.push("SharePoint identity");
        }
        if current.vpn_profile_id != unit.vpn_profile_id {
            differences.push("VPN assignment");
        }
        if let (ConnectionMode::OnlineMount(current_mode), ConnectionMode::OnlineMount(unit_mode)) =
            (&current.mode, &unit.mode)
            && current_mode.start_at_login != unit_mode.start_at_login
        {
            differences.push("start at login");
        }
        if !differences.is_empty() {
            println!(
                "Unit differs from saved {} ({}): {}",
                current.name,
                current.id,
                differences.join(", ")
            );
        }
    }
}

fn connection_id_from_service_name(name: &str) -> Option<ConnectionId> {
    let stripped = name
        .strip_prefix("cosmic-mounter-")?
        .strip_suffix(".service")?;
    let uuid = stripped.parse::<uuid::Uuid>().ok()?;
    Some(ConnectionId::from_uuid(uuid))
}

fn expand_home(value: &str, home: &Path) -> PathBuf {
    if let Some(rest) = value.strip_prefix("~/") {
        home.join(rest)
    } else if value == "~" {
        home.to_path_buf()
    } else {
        PathBuf::from(value)
    }
}

fn clean_description(description: &str, prefix: &str) -> String {
    description
        .strip_prefix(prefix)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| description.to_owned())
}

fn recover_managed_unit(unit: &LegacyUnit, home: &Path) -> Option<Connection> {
    let id = connection_id_from_service_name(&unit.name)?;
    let description = unit.description.as_deref().unwrap_or(&unit.name);
    let exec = &unit.exec_start;

    // OneDrive offline mirror managed by `onedrive`.
    if exec.iter().any(|token| {
        Path::new(token).file_name().and_then(|name| name.to_str()) == Some("onedrive")
    }) {
        let mut syncdir: Option<PathBuf> = None;
        let mut interval_seconds: u64 = 15 * 60;
        let mut index = 0;
        while index < exec.len() {
            match exec[index].as_str() {
                "--confdir" => {
                    index += 2;
                }
                "--syncdir" => {
                    syncdir = exec.get(index + 1).map(|s| expand_home(s, home));
                    index += 2;
                }
                "--monitor-interval" => {
                    if let Some(value) = exec.get(index + 1) {
                        interval_seconds = value.parse::<u64>().unwrap_or(interval_seconds);
                    }
                    index += 2;
                }
                _ => index += 1,
            }
        }
        let local_path = syncdir?;
        let recovery_directory = home
            .join(".local/state/cosmic-ext-applet-mounter/onedrive-recovery")
            .join(id.to_string());
        let name = clean_description(description, "COSMIC Cloud Mounter OneDrive mirror:");
        return Some(Connection {
            id,
            name,
            provider: Provider::OneDrive,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory,
                sync_interval_minutes: ((interval_seconds / 60).max(1)) as u32,
                sync_on_metered: false,
            }),
            remote_reference: format!("onedrive-mirror-{}", short_id(id)),
            remote_subpath: None,
            local_path,
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            teams_identity: None,
        });
    }

    // SharePoint/Teams offline mirror via managed-bisync.sh.
    if exec
        .first()
        .is_some_and(|token| Path::new(token).file_name() == Some(std::ffi::OsStr::new("sh")))
    {
        let script_path = exec.get(1).map(|s| expand_home(s, home))?;
        let script = match fs::read_to_string(&script_path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("Could not read {}: {}", script_path.display(), error);
                return None;
            }
        };
        let json_text = extract_quoted_argument(&script, "--verify-teams-mirror")?;
        let parsed: serde_json::Value = serde_json::from_str(&json_text).ok()?;
        let local_path = PathBuf::from(parsed["local_path"].as_str()?);
        let recovery_directory = PathBuf::from(parsed["recovery_directory"].as_str()?);
        let remote_reference = parsed["remote_name"].as_str()?.to_owned();
        let remote_subpath = parsed["remote_subpath"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);
        let teams_identity = TeamsLibraryIdentity {
            site_url: parsed["site_url"].as_str()?.to_owned(),
            library_url: parsed["library_url"].as_str()?.to_owned(),
            drive_id: parsed["drive_id"].as_str()?.to_owned(),
        };
        let timer_path = unit.path.with_extension("timer");
        let timer_minutes = timer_path
            .exists()
            .then(|| fs::read_to_string(&timer_path).ok())
            .flatten()
            .and_then(parse_timer_minutes)
            .unwrap_or(15);
        let name = clean_description(description, "Cloud Mounter sync:");
        return Some(Connection {
            id,
            name,
            provider: Provider::Teams,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory,
                sync_interval_minutes: timer_minutes,
                sync_on_metered: false,
            }),
            remote_reference,
            remote_subpath,
            local_path,
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            teams_identity: Some(teams_identity),
        });
    }

    None
}

fn read_teams_mount_identity(unit: &LegacyUnit) -> Option<TeamsLibraryIdentity> {
    let content = fs::read_to_string(&unit.path).ok()?;
    let args = extract_double_quoted_arguments(&content, "--verify-teams-mount")?;
    // Older units: connection_id, remote_name, subpath, local_path, drive_id, library_url
    // Newer units add a separate site_url argument before library_url.
    if args.len() < 6 {
        return None;
    }
    let drive_id = args[4].clone();
    let (site_url, library_url) = if args.len() >= 7 {
        (args[5].clone(), args[6].clone())
    } else {
        let library_url = decode_systemd_percent(&args[5]);
        let identity = cosmic_ext_applet_mounter::teams::parse_library_url(&library_url).ok()?;
        (identity.site_url, identity.library_url)
    };
    Some(TeamsLibraryIdentity {
        drive_id,
        site_url,
        library_url,
    })
}

fn decode_systemd_percent(value: &str) -> String {
    value.replace("%%", "%")
}

fn extract_quoted_argument(text: &str, marker: &str) -> Option<String> {
    let pos = text.find(marker)?;
    let start = pos + marker.len();
    let rest = &text[start..];
    // The marker is followed by its closing quote, whitespace, and then a
    // single-quoted JSON object. Split on single quotes and return the first
    // non-empty segment that starts with '{'.
    rest.split('\'')
        .map(str::trim)
        .find(|part| part.starts_with('{'))
        .map(String::from)
}

fn extract_double_quoted_arguments(text: &str, marker: &str) -> Option<Vec<String>> {
    let start = text.find(marker)?;
    let line = text[start..].lines().next()?;
    let after_marker = &line[marker.len()..];
    // The marker itself is quoted; skip past its closing quote.
    let first_close = after_marker.find('"')? + 1;
    let mut rest = &after_marker[first_close..];
    let mut args = Vec::new();
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let end = after.find('"')?;
        args.push(after[..end].to_owned());
        rest = &after[end + 1..];
    }
    Some(args)
}

fn parse_timer_minutes(content: String) -> Option<u32> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("OnUnitInactiveSec=") {
            let seconds = value
                .trim_end_matches('s')
                .parse::<u64>()
                .unwrap_or_default();
            return Some((seconds / 60).max(1) as u32);
        }
        if let Some(value) = line.strip_prefix("OnCalendar=") {
            // Cron-style timers are not recovered here.
            let _ = value;
            return Some(15);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_writer_never_replaces_a_live_or_existing_document() {
        let temporary = tempfile::tempdir().unwrap();
        let live = temporary.path().join("document");
        fs::write(&live, "existing live settings").unwrap();
        let document = ConfigDocument::default();

        assert!(write_candidate(&live, &live, &document).is_err());
        assert_eq!(fs::read_to_string(&live).unwrap(), "existing live settings");

        let candidate = temporary.path().join("candidate");
        write_candidate(&candidate, &live, &document).unwrap();
        let first = fs::read(&candidate).unwrap();
        assert!(write_candidate(&candidate, &live, &document).is_err());
        assert_eq!(fs::read(&candidate).unwrap(), first);
        assert_eq!(fs::read_to_string(&live).unwrap(), "existing live settings");
    }

    #[test]
    fn one_drive_recovery_labels_are_unique_per_connection() {
        let first =
            ConnectionId::from_uuid("990cc48f-4e4e-4ed7-a07b-c545ad3d3f9d".parse().unwrap());
        let second =
            ConnectionId::from_uuid("5ffc5d9b-6721-49f6-81d2-5f39c15061b7".parse().unwrap());
        assert_ne!(short_id(first), short_id(second));
    }

    #[test]
    fn recovered_names_exclude_service_description_prefixes() {
        assert_eq!(
            clean_description("Cloud Mounter: UA OneDrive", "Cloud Mounter:"),
            "UA OneDrive"
        );
        assert_eq!(
            clean_description(
                "COSMIC Cloud Mounter OneDrive mirror: uutzinger OneDrive mirror",
                "COSMIC Cloud Mounter OneDrive mirror:"
            ),
            "uutzinger OneDrive mirror"
        );
        assert_eq!(
            clean_description(
                "Cloud Mounter sync: Test_SharePoint Offline UI",
                "Cloud Mounter sync:"
            ),
            "Test_SharePoint Offline UI"
        );
    }
}
