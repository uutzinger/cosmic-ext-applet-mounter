// SPDX-License-Identifier: MIT

//! Dependency inventories, capability checks, and setup guidance.

use semver::Version;
use std::collections::BTreeSet;
use tokio_util::sync::CancellationToken;

use crate::process::{CommandRunner, Executable};
use crate::services::systemd_user_available;
use crate::vpn::DbusNetworkManagerVpn;

pub const MIN_RCLONE_VERSION: &str = "1.74.3";
pub const MIN_ONEDRIVER_VERSION: &str = "0.15.0";
pub const MIN_ONEDRIVE_VERSION: &str = "2.5.10";
pub const RCLONE_REQUIRED_CAPABILITIES: &[(&str, &str)] = &[
    ("command:mount", "mount"),
    ("command:bisync", "bisync"),
    ("command:listremotes", "listremotes"),
    ("command:rc", "rc"),
    ("mount:vfs-cache-mode", "--vfs-cache-mode"),
    ("mount:vfs-cache-max-size", "--vfs-cache-max-size"),
    ("mount:rc", "--rc"),
    ("bisync:resilient", "--resilient"),
    ("bisync:recover", "--recover"),
    ("bisync:max-lock", "--max-lock"),
    ("bisync:conflict-resolve", "--conflict-resolve"),
    ("bisync:backup-dir", "--backup-dir"),
];
pub const ONEDRIVER_REQUIRED_CAPABILITIES: &[(&str, &str)] = &[
    ("auth-only", "--auth-only"),
    ("cache-directory", "--cache-dir"),
    ("config-file", "--config-file"),
    ("read-only-offline", "read-only"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    Rclone,
    Onedriver,
    OneDriveSync,
    Fuse3,
    NetworkManager,
    SystemdUser,
}

impl DependencyKind {
    /// Inventory lists all supported tools; warnings follow configured engines.
    #[must_use]
    pub fn required_for(self, connections: &[crate::model::Connection]) -> bool {
        use crate::model::{ConnectionMode, OfflineEngine, Provider};
        match self {
            Self::NetworkManager | Self::SystemdUser => true,
            Self::Rclone => connections.iter().any(|connection| {
                connection.provider != Provider::OneDrive
                    || connection.offline_engine() == Some(OfflineEngine::Rclone)
            }),
            Self::Onedriver => connections.iter().any(|connection| {
                connection.provider == Provider::OneDrive
                    && matches!(connection.mode, ConnectionMode::OnlineMount(_))
            }),
            Self::OneDriveSync => connections.iter().any(|connection| {
                connection.provider == Provider::OneDrive
                    && connection.offline_engine() == Some(OfflineEngine::LegacyOnedrive)
            }),
            Self::Fuse3 => connections
                .iter()
                .any(|connection| matches!(connection.mode, ConnectionMode::OnlineMount(_))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyState {
    Available,
    Missing,
    Outdated { minimum: Version },
    MissingCapabilities,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyReport {
    pub kind: DependencyKind,
    pub required: bool,
    pub state: DependencyState,
    pub version: Option<Version>,
    pub path: Option<String>,
    pub capabilities: BTreeSet<String>,
    pub missing_capabilities: BTreeSet<String>,
    pub summary: String,
    pub guidance_url: &'static str,
}

impl DependencyReport {
    #[must_use]
    pub fn usable(&self) -> bool {
        self.state == DependencyState::Available
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyInventory {
    pub reports: Vec<DependencyReport>,
}

impl DependencyInventory {
    #[must_use]
    pub fn get(&self, kind: DependencyKind) -> Option<&DependencyReport> {
        self.reports.iter().find(|report| report.kind == kind)
    }

    pub async fn inspect(runner: &dyn CommandRunner, cancellation: CancellationToken) -> Self {
        let reports = vec![
            inspect_executable(
                runner,
                DependencyKind::Rclone,
                Executable::Rclone,
                true,
                "https://rclone.org/downloads/",
            ),
            inspect_executable(
                runner,
                DependencyKind::Onedriver,
                Executable::Onedriver,
                true,
                "https://github.com/jstaf/onedriver",
            ),
            inspect_executable(
                runner,
                DependencyKind::OneDriveSync,
                Executable::OneDrive,
                true,
                "https://github.com/abraunegg/onedrive",
            ),
            inspect_executable(
                runner,
                DependencyKind::Fuse3,
                Executable::Fusermount3,
                true,
                "https://github.com/libfuse/libfuse",
            ),
            inspect_network_manager(cancellation.child_token()).await,
            inspect_systemd_user(cancellation).await,
        ];
        Self { reports }
    }
}

fn inspect_executable(
    runner: &dyn CommandRunner,
    kind: DependencyKind,
    executable: Executable,
    required: bool,
    guidance_url: &'static str,
) -> DependencyReport {
    let Some(path) = runner.resolve(executable) else {
        return missing_report(kind, required, guidance_url);
    };
    DependencyReport {
        kind,
        required,
        state: DependencyState::Available,
        version: None,
        path: Some(path.display().to_string()),
        capabilities: BTreeSet::new(),
        missing_capabilities: BTreeSet::new(),
        summary: if path.to_string_lossy().starts_with("host:") {
            crate::fl!(
                "ui-diagnostics-host-execution-route-available-installation-and-compatibility"
            )
        } else {
            crate::fl!("ui-diagnostics-executable-available-compatibility-is-enforced-by-packaging")
        },
        guidance_url,
    }
}

async fn inspect_network_manager(cancellation: CancellationToken) -> DependencyReport {
    let result = tokio::select! {
        () = cancellation.cancelled() => Err(crate::fl!("dependency-inspection-cancelled")),
        result = DbusNetworkManagerVpn::connected() => result.map(|_| ()).map_err(|error| error.to_string()),
    };
    match result {
        Ok(()) => DependencyReport {
            kind: DependencyKind::NetworkManager,
            required: true,
            state: DependencyState::Available,
            version: None,
            path: Some("org.freedesktop.NetworkManager (system bus)".into()),
            capabilities: ["dbus".to_owned()].into_iter().collect(),
            missing_capabilities: BTreeSet::new(),
            summary: crate::fl!("ui-diagnostics-available-through-d-bus"),
            guidance_url: "https://networkmanager.dev/",
        },
        Err(error) => DependencyReport {
            kind: DependencyKind::NetworkManager,
            required: true,
            state: DependencyState::Error,
            version: None,
            path: None,
            capabilities: BTreeSet::new(),
            missing_capabilities: ["dbus".to_owned()].into_iter().collect(),
            summary: error,
            guidance_url: "https://networkmanager.dev/",
        },
    }
}

async fn inspect_systemd_user(cancellation: CancellationToken) -> DependencyReport {
    let result = tokio::select! {
        () = cancellation.cancelled() => Err(crate::fl!("dependency-inspection-cancelled")),
        result = systemd_user_available() => result,
    };
    match result {
        Ok(()) => DependencyReport {
            kind: DependencyKind::SystemdUser,
            required: true,
            state: DependencyState::Available,
            version: None,
            path: Some("org.freedesktop.systemd1 (user bus)".into()),
            capabilities: ["dbus".to_owned()].into_iter().collect(),
            missing_capabilities: BTreeSet::new(),
            summary: crate::fl!("ui-diagnostics-available-through-d-bus"),
            guidance_url: "https://systemd.io/",
        },
        Err(error) => DependencyReport {
            kind: DependencyKind::SystemdUser,
            required: true,
            state: DependencyState::Error,
            version: None,
            path: None,
            capabilities: BTreeSet::new(),
            missing_capabilities: ["dbus".to_owned()].into_iter().collect(),
            summary: error,
            guidance_url: "https://systemd.io/",
        },
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
fn assess_versioned(
    kind: DependencyKind,
    required: bool,
    path: String,
    version: Option<Version>,
    minimum: &str,
    required_capabilities: &[(&str, &str)],
    help: &str,
    guidance_url: &'static str,
) -> DependencyReport {
    let minimum = Version::parse(minimum).expect("minimum versions are valid");
    let capabilities: BTreeSet<_> = required_capabilities
        .iter()
        .filter(|(_, marker)| help.contains(marker))
        .map(|(name, _)| (*name).to_owned())
        .collect();
    let missing_capabilities: BTreeSet<_> = required_capabilities
        .iter()
        .filter(|(name, _)| !capabilities.contains(*name))
        .map(|(name, _)| (*name).to_owned())
        .collect();

    let state = if version.as_ref().is_none_or(|version| version < &minimum) {
        DependencyState::Outdated {
            minimum: minimum.clone(),
        }
    } else if missing_capabilities.is_empty() {
        DependencyState::Available
    } else {
        DependencyState::MissingCapabilities
    };
    let summary = match &state {
        DependencyState::Available => "Available".into(),
        DependencyState::Outdated { minimum } => {
            format!("Upgrade required: version {minimum} or newer")
        }
        DependencyState::MissingCapabilities => format!(
            "Required capabilities are unavailable: {}",
            missing_capabilities
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ),
        DependencyState::Missing | DependencyState::Error => unreachable!(),
    };

    DependencyReport {
        kind,
        required,
        state,
        version,
        path: Some(path),
        capabilities,
        missing_capabilities,
        summary,
        guidance_url,
    }
}

fn missing_report(
    kind: DependencyKind,
    required: bool,
    guidance_url: &'static str,
) -> DependencyReport {
    DependencyReport {
        kind,
        required,
        state: DependencyState::Missing,
        version: None,
        path: None,
        capabilities: BTreeSet::new(),
        missing_capabilities: BTreeSet::new(),
        summary: crate::fl!("ui-diagnostics-not-installed-or-not-discoverable"),
        guidance_url,
    }
}

#[cfg(test)]
fn parse_version(output: &str) -> Option<Version> {
    output
        .split(|character: char| {
            character.is_whitespace() || matches!(character, ',' | '(' | ')' | ':')
        })
        .map(|token| token.trim_start_matches('v'))
        .find_map(|token| {
            Version::parse(token)
                .ok()
                .or_else(|| parse_packaged_version(token))
        })
}

#[cfg(test)]
fn parse_packaged_version(token: &str) -> Option<Version> {
    let mut parts = token.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch_text = parts.next()?;
    let patch_digits: String = patch_text
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let patch = patch_digits.parse().ok()?;
    Some(Version::new(major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeCommandRunner;

    #[test]
    fn version_parser_handles_supported_formats() {
        assert_eq!(
            parse_version("rclone v1.74.3\n"),
            Some(Version::new(1, 74, 3))
        );
        assert_eq!(
            parse_version("onedriver v0.15.0 abc"),
            Some(Version::new(0, 15, 0))
        );
        assert_eq!(
            parse_version("onedrive v2.5.10"),
            Some(Version::new(2, 5, 10))
        );
        assert_eq!(
            parse_version("onedrive v2.5.10-1+np1+1.1"),
            Some(Version::new(2, 5, 10))
        );
    }

    #[test]
    fn outdated_version_wins_over_capabilities() {
        let report = assess_versioned(
            DependencyKind::Rclone,
            true,
            "/usr/bin/rclone".into(),
            Some(Version::new(1, 60, 1)),
            MIN_RCLONE_VERSION,
            &[("mount", "mount")],
            "mount",
            "https://rclone.org/downloads/",
        );
        assert!(matches!(
            report.state,
            DependencyState::Outdated { minimum } if minimum == Version::new(1, 74, 3)
        ));
        assert!(report.summary.contains("1.74.3"));
    }

    #[test]
    fn current_but_incomplete_tool_is_rejected() {
        let report = assess_versioned(
            DependencyKind::Rclone,
            true,
            "/usr/bin/rclone".into(),
            Some(Version::new(1, 74, 3)),
            MIN_RCLONE_VERSION,
            &[("mount", "mount"), ("bisync", "bisync")],
            "mount",
            "https://rclone.org/downloads/",
        );
        assert_eq!(report.state, DependencyState::MissingCapabilities);
        assert!(report.missing_capabilities.contains("bisync"));
    }

    #[test]
    fn packaging_policy_retains_all_storage_engine_minimums_and_capabilities() {
        let rclone_help = RCLONE_REQUIRED_CAPABILITIES
            .iter()
            .map(|(_, marker)| *marker)
            .collect::<Vec<_>>()
            .join(" ");
        let rclone = assess_versioned(
            DependencyKind::Rclone,
            true,
            "/usr/bin/rclone".into(),
            parse_version("rclone v1.74.3"),
            MIN_RCLONE_VERSION,
            RCLONE_REQUIRED_CAPABILITIES,
            &rclone_help,
            "https://rclone.org/downloads/",
        );
        assert_eq!(rclone.state, DependencyState::Available);
        assert_eq!(
            rclone.capabilities.len(),
            RCLONE_REQUIRED_CAPABILITIES.len()
        );

        let onedriver_help = ONEDRIVER_REQUIRED_CAPABILITIES
            .iter()
            .map(|(_, marker)| *marker)
            .collect::<Vec<_>>()
            .join(" ");
        let onedriver = assess_versioned(
            DependencyKind::Onedriver,
            true,
            "/usr/bin/onedriver".into(),
            parse_version("onedriver v0.15.0"),
            MIN_ONEDRIVER_VERSION,
            ONEDRIVER_REQUIRED_CAPABILITIES,
            &onedriver_help,
            "https://github.com/jstaf/onedriver",
        );
        assert_eq!(onedriver.state, DependencyState::Available);

        let onedrive = assess_versioned(
            DependencyKind::OneDriveSync,
            true,
            "/usr/bin/onedrive".into(),
            parse_version("onedrive v2.5.10"),
            MIN_ONEDRIVE_VERSION,
            &[],
            "",
            "https://github.com/abraunegg/onedrive",
        );
        assert_eq!(onedrive.state, DependencyState::Available);
    }

    #[tokio::test]
    async fn missing_tools_are_reported_independently() {
        let inventory =
            DependencyInventory::inspect(&FakeCommandRunner::default(), CancellationToken::new())
                .await;
        assert_eq!(inventory.reports.len(), 6);
        assert!(inventory.reports.iter().all(|report| {
            matches!(
                report.kind,
                DependencyKind::NetworkManager | DependencyKind::SystemdUser
            ) || report.state == DependencyState::Missing
        }));
        assert_ne!(
            inventory
                .get(DependencyKind::NetworkManager)
                .expect("NetworkManager report")
                .state,
            DependencyState::Missing
        );
    }

    #[tokio::test]
    async fn runtime_inventory_discovers_tools_without_executing_probes() {
        let runner = FakeCommandRunner::default().with_resolved([
            Executable::Rclone,
            Executable::Onedriver,
            Executable::OneDrive,
            Executable::Fusermount3,
        ]);

        let inventory = DependencyInventory::inspect(&runner, CancellationToken::new()).await;
        let report = inventory
            .get(DependencyKind::Onedriver)
            .expect("onedriver report");
        assert_eq!(report.state, DependencyState::Available);
        assert!(report.version.is_none());
        assert!(report.capabilities.is_empty());
        assert!(runner.requests().is_empty());
    }
}
