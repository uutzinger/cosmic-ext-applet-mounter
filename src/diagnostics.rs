// SPDX-License-Identifier: MIT

//! Dependency inventories, capability checks, and setup guidance.

use std::collections::BTreeSet;
use std::time::Duration;

use semver::Version;
use tokio_util::sync::CancellationToken;

use crate::process::{CommandError, CommandOutput, CommandRequest, CommandRunner, Executable};

pub const MIN_RCLONE_VERSION: &str = "1.74.3";
pub const MIN_ONEDRIVER_VERSION: &str = "0.15.0";
pub const MIN_ONEDRIVE_VERSION: &str = "2.5.10";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    Rclone,
    Onedriver,
    OneDriveSync,
    Fuse3,
    NetworkManager,
    CiscoSecureClient,
    SystemdUser,
    Fuser,
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
            inspect_rclone(runner, cancellation.child_token()).await,
            inspect_onedriver(runner, cancellation.child_token()).await,
            inspect_onedrive(runner, cancellation.child_token()).await,
            inspect_simple(
                runner,
                cancellation.child_token(),
                SimpleDependency {
                    kind: DependencyKind::Fuse3,
                    executable: Executable::Fusermount3,
                    args: &["--version"],
                    required: true,
                    guidance_url: "https://github.com/libfuse/libfuse",
                },
            )
            .await,
            inspect_simple(
                runner,
                cancellation.child_token(),
                SimpleDependency {
                    kind: DependencyKind::NetworkManager,
                    executable: Executable::Nmcli,
                    args: &["--version"],
                    required: true,
                    guidance_url: "https://networkmanager.dev/",
                },
            )
            .await,
            inspect_cisco(runner),
            inspect_simple(
                runner,
                cancellation.child_token(),
                SimpleDependency {
                    kind: DependencyKind::SystemdUser,
                    executable: Executable::Systemctl,
                    args: &["--version"],
                    required: true,
                    guidance_url: "https://systemd.io/",
                },
            )
            .await,
            inspect_simple(
                runner,
                cancellation,
                SimpleDependency {
                    kind: DependencyKind::Fuser,
                    executable: Executable::Fuser,
                    args: &["--version"],
                    required: false,
                    guidance_url: "https://gitlab.com/psmisc/psmisc",
                },
            )
            .await,
        ];
        Self { reports }
    }
}

struct SimpleDependency {
    kind: DependencyKind,
    executable: Executable,
    args: &'static [&'static str],
    required: bool,
    guidance_url: &'static str,
}

async fn inspect_simple(
    runner: &dyn CommandRunner,
    cancellation: CancellationToken,
    dependency: SimpleDependency,
) -> DependencyReport {
    let Some(path) = runner.resolve(dependency.executable) else {
        return missing_report(
            dependency.kind,
            dependency.required,
            dependency.guidance_url,
        );
    };
    let request = request(dependency.executable, dependency.args);
    match runner.run(request, cancellation).await {
        Ok(output) => DependencyReport {
            kind: dependency.kind,
            required: dependency.required,
            state: DependencyState::Available,
            version: parse_version(&combined_output(&output)),
            path: Some(path.display().to_string()),
            capabilities: BTreeSet::new(),
            missing_capabilities: BTreeSet::new(),
            summary: "Available".into(),
            guidance_url: dependency.guidance_url,
        },
        Err(error) => error_report(
            dependency.kind,
            dependency.required,
            Some(path.display().to_string()),
            dependency.guidance_url,
            error,
        ),
    }
}

async fn inspect_rclone(
    runner: &dyn CommandRunner,
    cancellation: CancellationToken,
) -> DependencyReport {
    const REQUIRED: &[(&str, &str)] = &[
        ("command:mount", "mount"),
        ("command:bisync", "bisync"),
        ("command:listremotes", "listremotes"),
        ("command:rc", "rc"),
        ("mount:vfs-cache-mode", "--vfs-cache-mode"),
        ("mount:vfs-cache-max-size", "--vfs-cache-max-size"),
        ("mount:rc", "--rc"),
        ("bisync:resilient", "--resilient"),
        ("bisync:recover", "--recover"),
        ("bisync:conflict-resolve", "--conflict-resolve"),
        ("bisync:backup-dir", "--backup-dir"),
    ];

    let Some(path) = runner.resolve(Executable::Rclone) else {
        return missing_report(
            DependencyKind::Rclone,
            true,
            "https://rclone.org/downloads/",
        );
    };

    let version_output = match runner
        .run(
            request(Executable::Rclone, &["version"]),
            cancellation.child_token(),
        )
        .await
    {
        Ok(output) => output,
        Err(error) => {
            return error_report(
                DependencyKind::Rclone,
                true,
                Some(path.display().to_string()),
                "https://rclone.org/downloads/",
                error,
            );
        }
    };

    let mut help = String::new();
    for args in [
        &["help"][..],
        &["mount", "--help"][..],
        &["help", "flags", "rc"][..],
        &["bisync", "--help"][..],
    ] {
        match runner
            .run(
                request(Executable::Rclone, args),
                cancellation.child_token(),
            )
            .await
        {
            Ok(output) => {
                help.push_str(&combined_output(&output));
                help.push('\n');
            }
            Err(error) => {
                return error_report(
                    DependencyKind::Rclone,
                    true,
                    Some(path.display().to_string()),
                    "https://rclone.org/downloads/",
                    error,
                );
            }
        }
    }

    assess_versioned(
        DependencyKind::Rclone,
        true,
        path.display().to_string(),
        parse_version(&combined_output(&version_output)),
        MIN_RCLONE_VERSION,
        REQUIRED,
        &help,
        "https://rclone.org/downloads/",
    )
}

async fn inspect_onedriver(
    runner: &dyn CommandRunner,
    cancellation: CancellationToken,
) -> DependencyReport {
    const REQUIRED: &[(&str, &str)] = &[
        ("auth-only", "--auth-only"),
        ("cache-directory", "--cache-dir"),
        ("config-file", "--config-file"),
        ("read-only-offline", "read-only"),
    ];
    inspect_versioned_tool(
        runner,
        cancellation,
        VersionedDependency {
            kind: DependencyKind::Onedriver,
            executable: Executable::Onedriver,
            version_args: &["--version"],
            help_args: &["--help"],
            minimum: MIN_ONEDRIVER_VERSION,
            required_capabilities: REQUIRED,
            required: true,
            guidance_url: "https://github.com/jstaf/onedriver",
        },
    )
    .await
}

async fn inspect_onedrive(
    runner: &dyn CommandRunner,
    cancellation: CancellationToken,
) -> DependencyReport {
    const GUIDANCE_URL: &str = "https://github.com/abraunegg/onedrive";
    let Some(path) = runner.resolve(Executable::OneDrive) else {
        return missing_report(DependencyKind::OneDriveSync, true, GUIDANCE_URL);
    };
    match runner
        .run(request(Executable::OneDrive, &["--version"]), cancellation)
        .await
    {
        Ok(output) => assess_versioned(
            DependencyKind::OneDriveSync,
            true,
            path.display().to_string(),
            parse_version(&combined_output(&output)),
            MIN_ONEDRIVE_VERSION,
            &[],
            "",
            GUIDANCE_URL,
        ),
        Err(error) => error_report(
            DependencyKind::OneDriveSync,
            true,
            Some(path.display().to_string()),
            GUIDANCE_URL,
            error,
        ),
    }
}

struct VersionedDependency {
    kind: DependencyKind,
    executable: Executable,
    version_args: &'static [&'static str],
    help_args: &'static [&'static str],
    minimum: &'static str,
    required_capabilities: &'static [(&'static str, &'static str)],
    required: bool,
    guidance_url: &'static str,
}

async fn inspect_versioned_tool(
    runner: &dyn CommandRunner,
    cancellation: CancellationToken,
    dependency: VersionedDependency,
) -> DependencyReport {
    let Some(path) = runner.resolve(dependency.executable) else {
        return missing_report(
            dependency.kind,
            dependency.required,
            dependency.guidance_url,
        );
    };

    let version = runner
        .run(
            request(dependency.executable, dependency.version_args),
            cancellation.child_token(),
        )
        .await;
    let help = runner
        .run(
            request(dependency.executable, dependency.help_args),
            cancellation,
        )
        .await;

    match (version, help) {
        (Ok(version), Ok(help)) => assess_versioned(
            dependency.kind,
            dependency.required,
            path.display().to_string(),
            parse_version(&combined_output(&version)),
            dependency.minimum,
            dependency.required_capabilities,
            &combined_output(&help),
            dependency.guidance_url,
        ),
        (Err(error), _) | (_, Err(error)) => error_report(
            dependency.kind,
            dependency.required,
            Some(path.display().to_string()),
            dependency.guidance_url,
            error,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
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

fn inspect_cisco(runner: &dyn CommandRunner) -> DependencyReport {
    let cli = runner.resolve(Executable::CiscoVpn);
    let ui = runner.resolve(Executable::CiscoVpnUi);
    let agent = runner.resolve(Executable::CiscoAgent);
    let capabilities: BTreeSet<_> = [
        cli.as_ref().map(|_| "cli".to_owned()),
        ui.as_ref().map(|_| "gui".to_owned()),
        agent.as_ref().map(|_| "agent".to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect();
    let missing_capabilities: BTreeSet<_> = ["cli", "gui", "agent"]
        .into_iter()
        .filter(|capability| !capabilities.contains(*capability))
        .map(str::to_owned)
        .collect();
    let state = if capabilities.is_empty() {
        DependencyState::Missing
    } else if missing_capabilities.is_empty() {
        DependencyState::Available
    } else {
        DependencyState::MissingCapabilities
    };

    DependencyReport {
        kind: DependencyKind::CiscoSecureClient,
        required: false,
        state,
        version: None,
        path: cli.or(ui).or(agent).map(|path| path.display().to_string()),
        capabilities,
        missing_capabilities,
        summary: "Cisco Secure Client components are detected independently".into(),
        guidance_url: "https://www.cisco.com/site/us/en/products/security/secure-client/",
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
        summary: "Not installed or not discoverable".into(),
        guidance_url,
    }
}

fn error_report(
    kind: DependencyKind,
    required: bool,
    path: Option<String>,
    guidance_url: &'static str,
    error: CommandError,
) -> DependencyReport {
    DependencyReport {
        kind,
        required,
        state: DependencyState::Error,
        version: None,
        path,
        capabilities: BTreeSet::new(),
        missing_capabilities: BTreeSet::new(),
        summary: error.to_string(),
        guidance_url,
    }
}

fn request(executable: Executable, args: &[&str]) -> CommandRequest {
    args.iter().fold(
        CommandRequest::new(executable)
            .with_timeout(Duration::from_secs(5))
            .with_output_limit(128 * 1024),
        |request, argument| request.arg(*argument).expect("fixed arguments are safe"),
    )
}

fn combined_output(output: &CommandOutput) -> String {
    format!("{}\n{}", output.stdout.text, output.stderr.text)
}

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
    use crate::process::{CapturedOutput, FakeCommandRunner};

    fn output(stdout: &str) -> CommandOutput {
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

    #[tokio::test]
    async fn missing_tools_are_reported_independently() {
        let inventory =
            DependencyInventory::inspect(&FakeCommandRunner::default(), CancellationToken::new())
                .await;
        assert_eq!(inventory.reports.len(), 8);
        assert!(
            inventory
                .reports
                .iter()
                .all(|report| report.state == DependencyState::Missing)
        );
    }

    #[tokio::test]
    async fn fake_runner_records_fixed_arguments() {
        let runner = FakeCommandRunner::default().with_resolved([Executable::Onedriver]);
        runner.push(Ok(output("onedriver v0.15.0")));
        runner.push(Ok(output(
            "--auth-only --cache-dir --config-file read-only",
        )));

        let report = inspect_onedriver(&runner, CancellationToken::new()).await;
        assert_eq!(report.state, DependencyState::Available);
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sanitized_command(), "onedriver --version");
        assert_eq!(requests[1].sanitized_command(), "onedriver --help");
    }
}
