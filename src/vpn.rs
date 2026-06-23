// SPDX-License-Identifier: MIT

//! NetworkManager and Cisco VPN integration and readiness checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::model::{Connection, ConnectionId, ReadinessCheck, VpnKind, VpnProfile, VpnProfileId};
use crate::process::{CommandError, CommandRequest, CommandRunner, Executable};

pub type VpnFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, VpnError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpnError {
    InvalidProfileKind,
    MissingExternalProfileId,
    InvalidEndpoint,
    MissingExecutable(Executable),
    Command(CommandError),
}

impl fmt::Display for VpnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileKind => write!(formatter, "invalid VPN profile kind"),
            Self::MissingExternalProfileId => write!(formatter, "missing external VPN profile id"),
            Self::InvalidEndpoint => write!(formatter, "invalid endpoint readiness check"),
            Self::MissingExecutable(executable) => {
                write!(formatter, "{} was not found", executable.display_name())
            }
            Self::Command(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for VpnError {}

impl From<CommandError> for VpnError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkManagerVpnProfile {
    pub name: String,
    pub uuid: String,
    pub vpn_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnConnectionState {
    Unknown,
    Disconnected,
    Activating,
    Activated,
    Failed,
}

impl VpnConnectionState {
    #[must_use]
    pub const fn ready(self) -> bool {
        matches!(self, Self::Activated)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiscoTunnelState {
    NotInstalled,
    ServiceUnavailable,
    Disconnected,
    Connecting,
    Connected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiscoComponents {
    pub cli: bool,
    pub gui: bool,
    pub agent: bool,
    pub tunnel: CiscoTunnelState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessReport {
    pub ready: bool,
    pub checks: Vec<ReadinessCheckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessCheckResult {
    pub check: ReadinessCheck,
    pub ready: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnActionDecision {
    NoVpnRequired,
    AlreadyReady,
    Activate,
    WaitForReadiness,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpnUsageSnapshot {
    pub profile_id: VpnProfileId,
    pub applet_activated: bool,
    pub active_connection_ids: BTreeSet<ConnectionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnShutdownDecision {
    NoAction,
    KeepAliveShared,
    KeepAlivePreExisting,
    Disconnect,
}

pub trait NetworkManagerVpn: Send + Sync {
    fn list_profiles<'a>(
        &'a self,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, Vec<NetworkManagerVpnProfile>>;

    fn activate<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ()>;

    fn deactivate<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ()>;

    fn state<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, VpnConnectionState>;
}

pub struct CommandNetworkManagerVpn<R> {
    runner: R,
}

impl<R> CommandNetworkManagerVpn<R> {
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: CommandRunner> NetworkManagerVpn for CommandNetworkManagerVpn<R> {
    fn list_profiles<'a>(
        &'a self,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, Vec<NetworkManagerVpnProfile>> {
        Box::pin(async move {
            let output = self
                .runner
                .run(
                    CommandRequest::new(Executable::Nmcli)
                        .arg("-t")?
                        .arg("-f")?
                        .arg("NAME,TYPE,UUID")?
                        .arg("connection")?
                        .arg("show")?
                        .with_timeout(Duration::from_secs(5)),
                    cancellation.clone(),
                )
                .await?;
            let profiles = parse_nmcli_profiles(&output.stdout.text);
            if !profiles.is_empty() {
                return Ok(profiles);
            }

            let fallback = self
                .runner
                .run(
                    CommandRequest::new(Executable::Nmcli)
                        .arg("-g")?
                        .arg("NAME,UUID,TYPE")?
                        .arg("connection")?
                        .arg("show")?
                        .with_timeout(Duration::from_secs(5)),
                    cancellation,
                )
                .await?;
            Ok(parse_nmcli_profiles(&fallback.stdout.text))
        })
    }

    fn activate<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ()> {
        Box::pin(async move {
            ensure_kind(profile, VpnKind::NetworkManager)?;
            let id = external_profile_id(profile)?;
            self.runner
                .run(
                    CommandRequest::new(Executable::Nmcli)
                        .arg("connection")?
                        .arg("up")?
                        .arg("uuid")?
                        .arg(id)?
                        .with_timeout(Duration::from_secs(u64::from(profile.timeout_seconds))),
                    cancellation,
                )
                .await?;
            Ok(())
        })
    }

    fn deactivate<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ()> {
        Box::pin(async move {
            ensure_kind(profile, VpnKind::NetworkManager)?;
            let id = external_profile_id(profile)?;
            self.runner
                .run(
                    CommandRequest::new(Executable::Nmcli)
                        .arg("connection")?
                        .arg("down")?
                        .arg("uuid")?
                        .arg(id)?
                        .with_timeout(Duration::from_secs(20)),
                    cancellation,
                )
                .await?;
            Ok(())
        })
    }

    fn state<'a>(
        &'a self,
        profile: &'a VpnProfile,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, VpnConnectionState> {
        Box::pin(async move {
            ensure_kind(profile, VpnKind::NetworkManager)?;
            let id = external_profile_id(profile)?;
            let output = self
                .runner
                .run(
                    CommandRequest::new(Executable::Nmcli)
                        .arg("-t")?
                        .arg("-f")?
                        .arg("GENERAL.STATE")?
                        .arg("connection")?
                        .arg("show")?
                        .arg(id)?
                        .with_timeout(Duration::from_secs(5)),
                    cancellation,
                )
                .await?;
            Ok(parse_nmcli_state(&output.stdout.text))
        })
    }
}

pub trait CiscoVpn: Send + Sync {
    fn components<'a>(&'a self, cancellation: CancellationToken) -> VpnFuture<'a, CiscoComponents>;
    fn open_gui_request(&self) -> Result<CommandRequest, VpnError>;
    fn start_agent_request(&self) -> Result<CommandRequest, VpnError>;
}

pub struct CommandCiscoVpn<R> {
    runner: R,
}

impl<R> CommandCiscoVpn<R> {
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: CommandRunner> CiscoVpn for CommandCiscoVpn<R> {
    fn components<'a>(&'a self, cancellation: CancellationToken) -> VpnFuture<'a, CiscoComponents> {
        Box::pin(async move {
            let cli = self.runner.resolve(Executable::CiscoVpn).is_some();
            let gui = self.runner.resolve(Executable::CiscoVpnUi).is_some();
            let agent = self.runner.resolve(Executable::CiscoAgent).is_some();
            let tunnel = if cli {
                match self
                    .runner
                    .run(
                        CommandRequest::new(Executable::CiscoVpn)
                            .arg("-s")?
                            .arg("stats")?
                            .with_timeout(Duration::from_secs(5)),
                        cancellation,
                    )
                    .await
                {
                    Ok(output) => {
                        parse_cisco_stats(&combined(&output.stdout.text, &output.stderr.text))
                    }
                    Err(_) if agent => CiscoTunnelState::ServiceUnavailable,
                    Err(_) => CiscoTunnelState::NotInstalled,
                }
            } else {
                CiscoTunnelState::NotInstalled
            };
            Ok(CiscoComponents {
                cli,
                gui,
                agent,
                tunnel,
            })
        })
    }

    fn open_gui_request(&self) -> Result<CommandRequest, VpnError> {
        Ok(CommandRequest::new(Executable::CiscoVpnUi).with_timeout(Duration::from_secs(5)))
    }

    fn start_agent_request(&self) -> Result<CommandRequest, VpnError> {
        CommandRequest::new(Executable::Systemctl)
            .arg("start")?
            .arg("vpnagentd.service")?
            .with_timeout(Duration::from_secs(20))
            .pipe(Ok)
    }
}

pub trait ReadinessProbe: Send + Sync {
    fn check<'a>(
        &'a self,
        check: &'a ReadinessCheck,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ReadinessCheckResult>;
}

pub struct CommandReadinessProbe<R> {
    runner: R,
}

impl<R> CommandReadinessProbe<R> {
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: CommandRunner> ReadinessProbe for CommandReadinessProbe<R> {
    fn check<'a>(
        &'a self,
        check: &'a ReadinessCheck,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ReadinessCheckResult> {
        Box::pin(async move {
            let ready = match check {
                ReadinessCheck::NetworkManagerState => {
                    let output = self
                        .runner
                        .run(
                            CommandRequest::new(Executable::Nmcli)
                                .arg("-t")?
                                .arg("-f")?
                                .arg("STATE")?
                                .arg("general")?
                                .with_timeout(Duration::from_secs(5)),
                            cancellation,
                        )
                        .await?;
                    output
                        .stdout
                        .text
                        .to_ascii_lowercase()
                        .contains("connected")
                }
                ReadinessCheck::Interface(name) => {
                    let output = self
                        .runner
                        .run(
                            CommandRequest::new(Executable::Nmcli)
                                .arg("-t")?
                                .arg("-f")?
                                .arg("DEVICE,STATE")?
                                .arg("device")?
                                .arg("status")?
                                .with_timeout(Duration::from_secs(5)),
                            cancellation,
                        )
                        .await?;
                    interface_ready(&output.stdout.text, name)
                }
                ReadinessCheck::Route(target) => self
                    .runner
                    .run(
                        CommandRequest::new(Executable::Ip)
                            .arg("route")?
                            .arg("get")?
                            .arg(target)?
                            .with_timeout(Duration::from_secs(5)),
                        cancellation,
                    )
                    .await
                    .is_ok(),
                ReadinessCheck::DnsName(name) => self
                    .runner
                    .run(
                        CommandRequest::new(Executable::Getent)
                            .arg("hosts")?
                            .arg(name)?
                            .with_timeout(Duration::from_secs(5)),
                        cancellation,
                    )
                    .await
                    .is_ok(),
                ReadinessCheck::Endpoint(endpoint) => {
                    let (host, port) = parse_endpoint(endpoint)?;
                    self.runner
                        .run(
                            CommandRequest::new(Executable::Nc)
                                .arg("-z")?
                                .arg("-w")?
                                .arg("3")?
                                .arg(host)?
                                .arg(port.to_string())?
                                .with_timeout(Duration::from_secs(5)),
                            cancellation,
                        )
                        .await
                        .is_ok()
                }
            };
            Ok(ReadinessCheckResult {
                check: check.clone(),
                ready,
                detail: if ready { "ready" } else { "not ready" }.into(),
            })
        })
    }
}

pub async fn readiness_report<P: ReadinessProbe>(
    probe: &P,
    checks: &[ReadinessCheck],
    cancellation: CancellationToken,
) -> Result<ReadinessReport, VpnError> {
    let mut results = Vec::new();
    for check in checks {
        results.push(probe.check(check, cancellation.child_token()).await?);
    }
    Ok(ReadinessReport {
        ready: results.iter().all(|result| result.ready),
        checks: results,
    })
}

#[must_use]
pub fn activation_decision(
    profile: Option<&VpnProfile>,
    readiness: &ReadinessReport,
    activation_timed_out: bool,
) -> VpnActionDecision {
    if profile.is_none() {
        VpnActionDecision::NoVpnRequired
    } else if readiness.ready {
        VpnActionDecision::AlreadyReady
    } else if activation_timed_out {
        VpnActionDecision::TimedOut
    } else if readiness.checks.is_empty() {
        VpnActionDecision::Activate
    } else {
        VpnActionDecision::WaitForReadiness
    }
}

#[must_use]
pub fn usage_snapshots<'a>(
    profiles: impl IntoIterator<Item = &'a VpnProfile>,
    connections: impl IntoIterator<Item = &'a Connection>,
    applet_activated: &BTreeSet<VpnProfileId>,
) -> Vec<VpnUsageSnapshot> {
    let mut usage = profiles
        .into_iter()
        .map(|profile| {
            (
                profile.id,
                VpnUsageSnapshot {
                    profile_id: profile.id,
                    applet_activated: applet_activated.contains(&profile.id),
                    active_connection_ids: BTreeSet::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for connection in connections {
        if connection.enabled
            && let Some(profile_id) = connection.vpn_profile_id
            && let Some(snapshot) = usage.get_mut(&profile_id)
        {
            snapshot.active_connection_ids.insert(connection.id);
        }
    }
    usage.into_values().collect()
}

#[must_use]
pub fn shutdown_decision(snapshot: &VpnUsageSnapshot) -> VpnShutdownDecision {
    if !snapshot.active_connection_ids.is_empty() {
        VpnShutdownDecision::KeepAliveShared
    } else if snapshot.applet_activated {
        VpnShutdownDecision::Disconnect
    } else {
        VpnShutdownDecision::KeepAlivePreExisting
    }
}

fn parse_nmcli_profiles(output: &str) -> Vec<NetworkManagerVpnProfile> {
    nmcli_profile_records(output)
        .into_iter()
        .filter_map(|line| {
            let parts = split_nmcli(&line);
            let (name, vpn_type, uuid) = match parts.as_slice() {
                [name, vpn_type, uuid] => (name, vpn_type, uuid),
                [name, uuid, vpn_type, ..] if looks_like_uuid(uuid) => (name, vpn_type, uuid),
                _ => return None,
            };
            valid_nmcli_profile_fields(name, vpn_type, uuid).then(|| NetworkManagerVpnProfile {
                name: name.trim().to_owned(),
                vpn_type: vpn_type.trim().to_owned(),
                uuid: uuid.trim().to_owned(),
            })
        })
        .collect()
}

fn valid_nmcli_profile_fields(name: &str, vpn_type: &str, uuid: &str) -> bool {
    !name.trim().is_empty()
        && looks_like_uuid(uuid.trim())
        && !vpn_type.chars().any(char::is_whitespace)
        && is_vpn_type(vpn_type)
}

fn nmcli_profile_records(output: &str) -> Vec<String> {
    let line_records: Vec<_> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if line_records.len() > 1 {
        line_records
    } else {
        line_records
            .into_iter()
            .chain(flattened_nmcli_records(output))
            .collect()
    }
}

fn flattened_nmcli_records(output: &str) -> impl Iterator<Item = String> + '_ {
    let mut records = Vec::new();
    let mut start = 0;
    while let Some((uuid_start, uuid_end)) = find_uuid_range(&output[start..]) {
        let record_end = start + uuid_end;
        let record = output[start..record_end].trim();
        if record.contains(':') {
            records.push(record.to_owned());
        }
        start += uuid_start + 36;
        while output[start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            start += output[start..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
        }
    }
    records.into_iter()
}

fn find_uuid_range(value: &str) -> Option<(usize, usize)> {
    for (index, _) in value.char_indices() {
        let Some(candidate) = value.get(index..index + 36) else {
            continue;
        };
        if looks_like_uuid(candidate) {
            return Some((index, index + 36));
        }
    }
    None
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

fn parse_nmcli_state(output: &str) -> VpnConnectionState {
    let lower = output.to_ascii_lowercase();
    if lower.contains("activated") || lower.contains(":100") {
        VpnConnectionState::Activated
    } else if lower.contains("activating") || lower.contains(":50") {
        VpnConnectionState::Activating
    } else if lower.contains("failed") {
        VpnConnectionState::Failed
    } else if lower.contains("deactivated") || lower.contains("disconnected") {
        VpnConnectionState::Disconnected
    } else {
        VpnConnectionState::Unknown
    }
}

fn parse_cisco_stats(output: &str) -> CiscoTunnelState {
    let lower = output.to_ascii_lowercase();
    if lower.contains("cannot contact the vpn service") {
        return CiscoTunnelState::ServiceUnavailable;
    }
    for line in lower.lines() {
        let Some((_, state)) = line.split_once("connection state:") else {
            continue;
        };
        let state = state.trim();
        if state.starts_with("connected") {
            return CiscoTunnelState::Connected;
        }
        if state.starts_with("connecting") {
            return CiscoTunnelState::Connecting;
        }
        if state.starts_with("disconnected") || state.starts_with("not available") {
            return CiscoTunnelState::Disconnected;
        }
    }
    CiscoTunnelState::Unknown
}

fn split_nmcli(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == ':' {
            values.push(current);
            current = String::new();
        } else {
            current.push(character);
        }
    }
    values.push(current);
    values
}

fn is_vpn_type(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("vpn")
        || lower.contains("wireguard")
        || lower.contains("openvpn")
        || lower.contains("anyconnect")
}

fn interface_ready(output: &str, name: &str) -> bool {
    output.lines().any(|line| {
        let parts = split_nmcli(line);
        let [device, state] = parts.as_slice() else {
            return false;
        };
        device == name && state.eq_ignore_ascii_case("connected")
    })
}

fn parse_endpoint(endpoint: &str) -> Result<(String, u16), VpnError> {
    let Some((host, port)) = endpoint.rsplit_once(':') else {
        return Err(VpnError::InvalidEndpoint);
    };
    let port = port.parse().map_err(|_| VpnError::InvalidEndpoint)?;
    let host = host.trim_matches(['[', ']']).to_owned();
    if host.is_empty() || (host.as_str(), port).to_socket_addrs().is_err() {
        return Err(VpnError::InvalidEndpoint);
    }
    Ok((host, port))
}

fn ensure_kind(profile: &VpnProfile, expected: VpnKind) -> Result<(), VpnError> {
    if profile.kind == expected {
        Ok(())
    } else {
        Err(VpnError::InvalidProfileKind)
    }
}

fn external_profile_id(profile: &VpnProfile) -> Result<&str, VpnError> {
    profile
        .external_profile_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or(VpnError::MissingExternalProfileId)
}

fn combined(stdout: &str, stderr: &str) -> String {
    format!("{stdout}\n{stderr}")
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use crate::model::{AccessMode, ConnectionMode, OfflineMirrorConfig, TuningProfile};
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};
    use uuid::Uuid;

    use super::*;

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

    fn vpn_id() -> VpnProfileId {
        VpnProfileId::from_uuid(
            Uuid::parse_str("17ea4cc5-f4f0-405b-b112-dad6f855bb77").expect("UUID"),
        )
    }

    fn connection_id(value: &str) -> ConnectionId {
        ConnectionId::from_uuid(Uuid::parse_str(value).expect("UUID"))
    }

    fn nm_profile() -> VpnProfile {
        VpnProfile {
            id: vpn_id(),
            name: "Work VPN".into(),
            kind: VpnKind::NetworkManager,
            external_profile_id: Some("91a601dd-2df4-4b32-bc66-25a16a7612fe".into()),
            readiness_checks: vec![ReadinessCheck::NetworkManagerState],
            timeout_seconds: 30,
        }
    }

    fn connection(
        id: ConnectionId,
        vpn_profile_id: Option<VpnProfileId>,
        enabled: bool,
    ) -> Connection {
        Connection {
            id,
            name: "Mirror".into(),
            provider: crate::model::Provider::GoogleDrive,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig::default()),
            remote_reference: "remote".into(),
            remote_subpath: None,
            local_path: "/home/example/Cloud".into(),
            enabled,
            vpn_profile_id,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
        }
    }

    #[tokio::test]
    async fn network_manager_lists_activates_deactivates_and_reads_state() {
        let runner = FakeCommandRunner::default().with_resolved([Executable::Nmcli]);
        runner.push(Ok(output(
            "Home:802-11-wireless:abc\nWork VPN:vpn:91a601dd-2df4-4b32-bc66-25a16a7612fe\n",
        )));
        runner.push(Ok(output("")));
        runner.push(Ok(output("GENERAL.STATE:activated\n")));
        runner.push(Ok(output("")));
        let manager = CommandNetworkManagerVpn::new(runner.clone());
        let profile = nm_profile();

        let profiles = manager
            .list_profiles(CancellationToken::new())
            .await
            .expect("profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Work VPN");
        manager
            .activate(&profile, CancellationToken::new())
            .await
            .expect("activate");
        assert_eq!(
            manager
                .state(&profile, CancellationToken::new())
                .await
                .expect("state"),
            VpnConnectionState::Activated
        );
        manager
            .deactivate(&profile, CancellationToken::new())
            .await
            .expect("deactivate");

        assert_eq!(
            runner
                .requests()
                .iter()
                .map(CommandRequest::sanitized_command)
                .collect::<Vec<_>>(),
            vec![
                "nmcli -t -f NAME,TYPE,UUID connection show",
                "nmcli connection up uuid 91a601dd-2df4-4b32-bc66-25a16a7612fe",
                "nmcli -t -f GENERAL.STATE connection show 91a601dd-2df4-4b32-bc66-25a16a7612fe",
                "nmcli connection down uuid 91a601dd-2df4-4b32-bc66-25a16a7612fe",
            ]
        );
    }

    #[tokio::test]
    async fn network_manager_list_profiles_falls_back_to_name_uuid_type_output() {
        let runner = FakeCommandRunner::default().with_resolved([Executable::Nmcli]);
        runner.push(Ok(output("")));
        runner.push(Ok(output(
            "SalterLab:51424a59-495c-4483-ad44-a0bf49327d5e:wireguard:\n",
        )));
        let manager = CommandNetworkManagerVpn::new(runner.clone());

        let profiles = manager
            .list_profiles(CancellationToken::new())
            .await
            .expect("profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "SalterLab");
        assert_eq!(profiles[0].vpn_type, "wireguard");
        assert_eq!(profiles[0].uuid, "51424a59-495c-4483-ad44-a0bf49327d5e");
        assert_eq!(
            runner
                .requests()
                .iter()
                .map(CommandRequest::sanitized_command)
                .collect::<Vec<_>>(),
            vec![
                "nmcli -t -f NAME,TYPE,UUID connection show",
                "nmcli -g NAME,UUID,TYPE connection show",
            ]
        );
    }

    #[tokio::test]
    async fn readiness_checks_use_fixed_arguments() {
        let runner = FakeCommandRunner::default().with_resolved([
            Executable::Nmcli,
            Executable::Ip,
            Executable::Getent,
            Executable::Nc,
        ]);
        runner.push(Ok(output("STATE:connected\n")));
        runner.push(Ok(output("tun0:connected\nwlan0:connected\n")));
        runner.push(Ok(output("10.0.0.1 via 192.168.1.1 dev tun0\n")));
        runner.push(Ok(output("10.0.0.10 storage.example\n")));
        runner.push(Ok(output("")));
        let probe = CommandReadinessProbe::new(runner.clone());
        let checks = vec![
            ReadinessCheck::NetworkManagerState,
            ReadinessCheck::Interface("tun0".into()),
            ReadinessCheck::Route("10.0.0.10".into()),
            ReadinessCheck::DnsName("storage.example".into()),
            ReadinessCheck::Endpoint("127.0.0.1:443".into()),
        ];
        let report = readiness_report(&probe, &checks, CancellationToken::new())
            .await
            .expect("readiness");
        assert!(report.ready);
        assert_eq!(
            runner
                .requests()
                .iter()
                .map(CommandRequest::sanitized_command)
                .collect::<Vec<_>>(),
            vec![
                "nmcli -t -f STATE general",
                "nmcli -t -f DEVICE,STATE device status",
                "ip route get 10.0.0.10",
                "getent hosts storage.example",
                "nc -z -w 3 127.0.0.1 443",
            ]
        );
    }

    #[tokio::test]
    async fn cisco_components_report_service_unavailable_and_gui_commands_are_typed() {
        let runner = FakeCommandRunner::default().with_resolved([
            Executable::CiscoVpn,
            Executable::CiscoVpnUi,
            Executable::CiscoAgent,
        ]);
        runner.push(Ok(output("error: Cannot contact the VPN service.")));
        let cisco = CommandCiscoVpn::new(runner);
        let components = cisco
            .components(CancellationToken::new())
            .await
            .expect("components");
        assert_eq!(
            components,
            CiscoComponents {
                cli: true,
                gui: true,
                agent: true,
                tunnel: CiscoTunnelState::ServiceUnavailable,
            }
        );
        assert_eq!(
            cisco.open_gui_request().expect("gui").sanitized_command(),
            "Cisco Secure Client VPN UI"
        );
        assert_eq!(
            cisco
                .start_agent_request()
                .expect("agent")
                .sanitized_command(),
            "systemctl start vpnagentd.service"
        );
    }

    #[test]
    fn activation_and_shutdown_decisions_respect_readiness_and_ownership() {
        let profile = nm_profile();
        assert_eq!(
            activation_decision(
                None,
                &ReadinessReport {
                    ready: false,
                    checks: vec![],
                },
                false,
            ),
            VpnActionDecision::NoVpnRequired
        );
        assert_eq!(
            activation_decision(
                Some(&profile),
                &ReadinessReport {
                    ready: false,
                    checks: vec![],
                },
                false,
            ),
            VpnActionDecision::Activate
        );
        assert_eq!(
            activation_decision(
                Some(&profile),
                &ReadinessReport {
                    ready: false,
                    checks: vec![ReadinessCheckResult {
                        check: ReadinessCheck::NetworkManagerState,
                        ready: false,
                        detail: "not ready".into(),
                    }],
                },
                true,
            ),
            VpnActionDecision::TimedOut
        );

        let active = VpnUsageSnapshot {
            profile_id: profile.id,
            applet_activated: true,
            active_connection_ids: [connection_id("2a3f5d45-e867-47e7-943f-66cf60e777ad")]
                .into_iter()
                .collect(),
        };
        assert_eq!(
            shutdown_decision(&active),
            VpnShutdownDecision::KeepAliveShared
        );
        assert_eq!(
            shutdown_decision(&VpnUsageSnapshot {
                active_connection_ids: BTreeSet::new(),
                ..active
            }),
            VpnShutdownDecision::Disconnect
        );
        assert_eq!(
            shutdown_decision(&VpnUsageSnapshot {
                applet_activated: false,
                active_connection_ids: BTreeSet::new(),
                ..active
            }),
            VpnShutdownDecision::KeepAlivePreExisting
        );
    }

    #[test]
    fn usage_snapshots_reference_count_shared_dependencies() {
        let profile = nm_profile();
        let applet_activated = [profile.id].into_iter().collect();
        let snapshots = usage_snapshots(
            [&profile],
            [
                connection(
                    connection_id("2a3f5d45-e867-47e7-943f-66cf60e777ad"),
                    Some(profile.id),
                    true,
                ),
                connection(
                    connection_id("3815551b-93f0-4731-ac61-b303bbff3260"),
                    Some(profile.id),
                    true,
                ),
                connection(
                    connection_id("3c5f7365-19c3-4464-90dc-92e3910ed4be"),
                    Some(profile.id),
                    false,
                ),
            ]
            .iter(),
            &applet_activated,
        );
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].active_connection_ids.len(), 2);
        assert!(snapshots[0].applet_activated);
    }

    #[test]
    fn parsers_handle_nmcli_escaping_cisco_connected_and_access_modes() {
        assert_eq!(
            split_nmcli(r"Work\: VPN:vpn:uuid"),
            vec!["Work: VPN", "vpn", "uuid"]
        );
        let profiles = parse_nmcli_profiles(
            "SalterLab:wireguard:51424a59-495c-4483-ad44-a0bf49327d5e\n\
             SalterLab:51424a59-495c-4483-ad44-a0bf49327d5e:wireguard:\n",
        );
        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().all(|profile| profile.name == "SalterLab"));
        let flattened_profiles = parse_nmcli_profiles(
            "Jarvis-5G:802-11-wireless:9000c4f8-da8d-4cf5-baea-9d747e3161ee \
             Pixel 6 Network:bluetooth:01637915-76fb-44ee-a7d8-50519b92176f \
             SalterLab:wireguard:51424a59-495c-4483-ad44-a0bf49327d5e \
             Wired connection 1:802-3-ethernet:7097f8f6-a5d9-3425-8e2c-e7d7ef12b8a0",
        );
        assert_eq!(
            flattened_profiles,
            vec![NetworkManagerVpnProfile {
                name: "SalterLab".into(),
                uuid: "51424a59-495c-4483-ad44-a0bf49327d5e".into(),
                vpn_type: "wireguard".into(),
            }]
        );
        let flattened_name_uuid_type_profiles = parse_nmcli_profiles(
            "Jarvis-5G:9000c4f8-da8d-4cf5-baea-9d747e3161ee:802-11-wireless \
             Pixel 6 Network:01637915-76fb-44ee-a7d8-50519b92176f:bluetooth \
             SalterLab:51424a59-495c-4483-ad44-a0bf49327d5e:wireguard \
             Wired connection 1:7097f8f6-a5d9-3425-8e2c-e7d7ef12b8a0:802-3-ethernet",
        );
        assert!(
            flattened_name_uuid_type_profiles
                .iter()
                .all(|profile| !profile.name.is_empty())
        );
        assert_eq!(
            parse_cisco_stats("Connection State:            Connected\n"),
            CiscoTunnelState::Connected
        );
        assert_eq!(
            parse_cisco_stats("Connection State:            Disconnected\n"),
            CiscoTunnelState::Disconnected
        );
        assert_eq!(
            parse_cisco_stats("Connection State:            Not Available\n"),
            CiscoTunnelState::Disconnected
        );
        assert_eq!(
            parse_cisco_stats(
                "Cannot contact the VPN service.\nConnection State:            Not Available\n"
            ),
            CiscoTunnelState::ServiceUnavailable
        );
        assert_eq!(
            AccessMode::OfflineMirror,
            ConnectionMode::OfflineMirror(OfflineMirrorConfig::default()).kind()
        );
    }
}
