// SPDX-License-Identifier: MIT

//! NetworkManager VPN integration and readiness checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::time::Duration;

use tokio::net::{TcpStream, UdpSocket};
use tokio_util::sync::CancellationToken;
#[cfg(test)]
use zbus::zvariant::Str;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::model::{Connection, ConnectionId, ReadinessCheck, VpnKind, VpnProfile, VpnProfileId};
use crate::process::{CommandError, Executable};

pub type VpnFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, VpnError>> + Send + 'a>>;
pub const METERED_GUARD_FLAG: &str = "--require-unmetered-network";
pub const OFFLINE_READINESS_GUARD_FLAG: &str = "--require-offline-readiness";

pub fn offline_readiness_guard_command(
    connection_id: ConnectionId,
) -> Result<crate::services::ServiceCommand, String> {
    crate::mount_guard::service_command(vec![
        OFFLINE_READINESS_GUARD_FLAG.into(),
        connection_id.to_string(),
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpnError {
    InvalidProfileKind,
    MissingExternalProfileId,
    InvalidEndpoint,
    ReadinessCancelled,
    MissingExecutable(Executable),
    Command(CommandError),
    Dbus(String),
    AuthenticationAgent(String),
}

impl fmt::Display for VpnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileKind => {
                formatter.write_str(&crate::fl!("ui-vpn-invalid-vpn-profile-kind"))
            }
            Self::MissingExternalProfileId => {
                formatter.write_str(&crate::fl!("ui-vpn-missing-external-vpn-profile-id"))
            }
            Self::InvalidEndpoint => {
                formatter.write_str(&crate::fl!("ui-vpn-invalid-endpoint-readiness-check"))
            }
            Self::ReadinessCancelled => {
                formatter.write_str(&crate::fl!("ui-vpn-readiness-check-was-cancelled"))
            }
            Self::MissingExecutable(executable) => formatter.write_str(&crate::fl!(
                "ui-vpn-value1-was-not-found",
                value1 = (executable.display_name()).to_string()
            )),
            Self::Command(error) => error.fmt(formatter),
            Self::Dbus(error) => formatter.write_str(&crate::fl!(
                "ui-vpn-networkmanager-d-bus-error-error",
                error = (error).to_string()
            )),
            Self::AuthenticationAgent(error) => formatter.write_str(&crate::fl!(
                "ui-vpn-openconnect-authentication-agent-error",
                error = (error).to_string()
            )),
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
    pub service_type: Option<String>,
}

impl NetworkManagerVpnProfile {
    #[must_use]
    pub fn is_openconnect(&self) -> bool {
        self.service_type.as_deref() == Some(OPENCONNECT_SERVICE_TYPE)
    }
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

#[derive(Debug, Clone, Copy, Default)]
pub struct DbusNetworkManagerVpn;

const NM_DESTINATION: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";
const NM_INTERFACE: &str = "org.freedesktop.NetworkManager";
const NM_SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";
const NM_SETTINGS_INTERFACE: &str = "org.freedesktop.NetworkManager.Settings";
const OPENCONNECT_SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.openconnect";
const OPENCONNECT_AGENT_UNIT: &str = "cosmic-mounter-openconnect-agent.service";

type NmSettings = BTreeMap<String, BTreeMap<String, OwnedValue>>;

fn profile_from_settings(values: &NmSettings) -> Option<NetworkManagerVpnProfile> {
    let connection_values = values.get("connection")?;
    let string = |name: &str| {
        connection_values
            .get(name)
            .and_then(|value| String::try_from(value.clone()).ok())
    };
    let (name, uuid, connection_type) = (string("id")?, string("uuid")?, string("type")?);
    let service_type = values
        .get("vpn")
        .and_then(|vpn| vpn.get("service-type"))
        .and_then(|value| String::try_from(value.clone()).ok());
    is_vpn_type(&connection_type).then_some(NetworkManagerVpnProfile {
        name,
        uuid,
        vpn_type: connection_type,
        service_type,
    })
}

fn requires_openconnect_agent(profile: &NetworkManagerVpnProfile) -> bool {
    profile.is_openconnect()
}

impl DbusNetworkManagerVpn {
    async fn connection() -> Result<zbus::Connection, VpnError> {
        zbus::Connection::system()
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))
    }

    async fn profiles() -> Result<Vec<(NetworkManagerVpnProfile, OwnedObjectPath)>, VpnError> {
        let connection = Self::connection().await?;
        let settings = zbus::Proxy::new(
            &connection,
            NM_DESTINATION,
            NM_SETTINGS_PATH,
            NM_SETTINGS_INTERFACE,
        )
        .await
        .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let paths: Vec<OwnedObjectPath> = settings
            .call("ListConnections", &())
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let mut profiles = Vec::new();
        for path in paths {
            let proxy = zbus::Proxy::new(
                &connection,
                NM_DESTINATION,
                path.as_str(),
                "org.freedesktop.NetworkManager.Settings.Connection",
            )
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
            let values: NmSettings = proxy
                .call("GetSettings", &())
                .await
                .map_err(|error| VpnError::Dbus(error.to_string()))?;
            if let Some(profile) = profile_from_settings(&values) {
                profiles.push((profile, path));
            }
        }
        Ok(profiles)
    }

    async fn profile(uuid: &str) -> Result<(NetworkManagerVpnProfile, OwnedObjectPath), VpnError> {
        Self::profiles()
            .await?
            .into_iter()
            .find(|(profile, _)| profile.uuid == uuid)
            .ok_or_else(|| VpnError::Dbus(crate::fl!("ui-vpn-saved-vpn-profile-was-not-found")))
    }

    pub async fn profile_is_openconnect(uuid: &str) -> Result<bool, VpnError> {
        Self::profile(uuid)
            .await
            .map(|(profile, _)| profile.is_openconnect())
    }

    pub async fn restart_openconnect_agent() -> Result<(), VpnError> {
        crate::services::systemd_user_restart_named_unit(OPENCONNECT_AGENT_UNIT)
            .await
            .map_err(|error| {
                VpnError::AuthenticationAgent(crate::fl!(
                    "ui-vpn-the-user-service-could-not-be-restarted",
                    error = (error).to_string()
                ))
            })
    }

    async fn active_path(uuid: &str) -> Result<Option<OwnedObjectPath>, VpnError> {
        let connection = Self::connection().await?;
        let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let paths: Vec<OwnedObjectPath> = manager
            .get_property("ActiveConnections")
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        for path in paths {
            let active = zbus::Proxy::new(
                &connection,
                NM_DESTINATION,
                path.as_str(),
                "org.freedesktop.NetworkManager.Connection.Active",
            )
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
            let active_uuid: String = active
                .get_property("Uuid")
                .await
                .map_err(|error| VpnError::Dbus(error.to_string()))?;
            if active_uuid == uuid {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub async fn connected() -> Result<bool, VpnError> {
        let connection = Self::connection().await?;
        let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let state: u32 = manager
            .get_property("State")
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        Ok(matches!(state, 60 | 70))
    }

    pub async fn metered() -> Result<bool, VpnError> {
        let connection = Self::connection().await?;
        let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let metered: u32 = manager
            .get_property("Metered")
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        Ok(matches!(metered, 1 | 3))
    }

    pub async fn interface_ready(name: &str) -> Result<bool, VpnError> {
        let connection = Self::connection().await?;
        let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        let devices: Vec<OwnedObjectPath> = manager
            .call("GetDevices", &())
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
        for path in devices {
            let device = zbus::Proxy::new(
                &connection,
                NM_DESTINATION,
                path.as_str(),
                "org.freedesktop.NetworkManager.Device",
            )
            .await
            .map_err(|error| VpnError::Dbus(error.to_string()))?;
            let interface: String = device
                .get_property("Interface")
                .await
                .map_err(|error| VpnError::Dbus(error.to_string()))?;
            if interface == name {
                let state: u32 = device
                    .get_property("State")
                    .await
                    .map_err(|error| VpnError::Dbus(error.to_string()))?;
                return Ok(state == 100);
            }
        }
        Ok(false)
    }
}

pub async fn run_unmetered_guard() -> Result<(), String> {
    match DbusNetworkManagerVpn::metered().await {
        Ok(false) => Ok(()),
        Ok(true) => Err(crate::fl!(
            "ui-vpn-automatic-synchronization-is-paused-on-a-metered"
        )),
        Err(error) => Err(crate::fl!(
            "ui-vpn-could-not-inspect-networkmanager-metered-state-error",
            error = (error).to_string()
        )),
    }
}

impl NetworkManagerVpn for DbusNetworkManagerVpn {
    fn list_profiles<'a>(
        &'a self,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, Vec<NetworkManagerVpnProfile>> {
        Box::pin(async move {
            tokio::select! {
                () = cancellation.cancelled() => Err(VpnError::Dbus("operation cancelled".into())),
                result = tokio::time::timeout(Duration::from_secs(5), Self::profiles()) => {
                    result
                        .map_err(|_| VpnError::Dbus("saved-profile discovery timed out".into()))?
                        .map(|profiles| profiles.into_iter().map(|(profile, _)| profile).collect())
                },
            }
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
            let operation = async {
                let (saved_profile, path) = Self::profile(id).await?;
                if requires_openconnect_agent(&saved_profile) {
                    Self::restart_openconnect_agent().await?;
                }
                let connection = Self::connection().await?;
                let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
                    .await
                    .map_err(|error| VpnError::Dbus(error.to_string()))?;
                let root = OwnedObjectPath::try_from("/")
                    .map_err(|error| VpnError::Dbus(error.to_string()))?;
                let arguments = (path, &root, &root);
                manager
                    .call::<_, _, OwnedObjectPath>("ActivateConnection", &arguments)
                    .await
                    .map(|_| ())
                    .map_err(|error| VpnError::Dbus(error.to_string()))
            };
            tokio::select! {
                () = cancellation.cancelled() => Err(VpnError::Dbus("operation cancelled".into())),
                result = tokio::time::timeout(Duration::from_secs(u64::from(profile.timeout_seconds)), operation) => {
                    result.map_err(|_| VpnError::Dbus("activation timed out".into()))?
                },
            }
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
            let operation = async {
                let Some(path) = Self::active_path(id).await? else {
                    return Ok(());
                };
                let connection = Self::connection().await?;
                let manager = zbus::Proxy::new(&connection, NM_DESTINATION, NM_PATH, NM_INTERFACE)
                    .await
                    .map_err(|error| VpnError::Dbus(error.to_string()))?;
                let arguments = (path,);
                manager
                    .call::<_, _, ()>("DeactivateConnection", &arguments)
                    .await
                    .map_err(|error| VpnError::Dbus(error.to_string()))
            };
            tokio::select! {
                () = cancellation.cancelled() => Err(VpnError::Dbus("operation cancelled".into())),
                result = tokio::time::timeout(Duration::from_secs(20), operation) => {
                    result.map_err(|_| VpnError::Dbus("deactivation timed out".into()))?
                },
            }
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
            let operation = async {
                let Some(path) = Self::active_path(id).await? else {
                    return Ok(VpnConnectionState::Disconnected);
                };
                let connection = Self::connection().await?;
                let active = zbus::Proxy::new(
                    &connection,
                    NM_DESTINATION,
                    path.as_str(),
                    "org.freedesktop.NetworkManager.Connection.Active",
                )
                .await
                .map_err(|error| VpnError::Dbus(error.to_string()))?;
                let state: u32 = active
                    .get_property("State")
                    .await
                    .map_err(|error| VpnError::Dbus(error.to_string()))?;
                Ok(match state {
                    1 => VpnConnectionState::Activating,
                    2 => VpnConnectionState::Activated,
                    3 => VpnConnectionState::Disconnected,
                    4 => VpnConnectionState::Failed,
                    _ => VpnConnectionState::Unknown,
                })
            };
            tokio::select! {
                () = cancellation.cancelled() => Err(VpnError::Dbus("operation cancelled".into())),
                result = tokio::time::timeout(Duration::from_secs(5), operation) => {
                    result.map_err(|_| VpnError::Dbus("state query timed out".into()))?
                },
            }
        })
    }
}

pub trait ReadinessProbe: Send + Sync {
    fn check<'a>(
        &'a self,
        check: &'a ReadinessCheck,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ReadinessCheckResult>;
}

pub struct InProcessReadinessProbe;

impl ReadinessProbe for InProcessReadinessProbe {
    fn check<'a>(
        &'a self,
        check: &'a ReadinessCheck,
        cancellation: CancellationToken,
    ) -> VpnFuture<'a, ReadinessCheckResult> {
        Box::pin(async move {
            let ready = match check {
                ReadinessCheck::NetworkManagerState => {
                    tokio::select! {
                        () = cancellation.cancelled() => return Err(VpnError::Dbus("operation cancelled".into())),
                        result = DbusNetworkManagerVpn::connected() => result?,
                    }
                }
                ReadinessCheck::Interface(name) => {
                    tokio::select! {
                        () = cancellation.cancelled() => return Err(VpnError::Dbus("operation cancelled".into())),
                        result = DbusNetworkManagerVpn::interface_ready(name) => result?,
                    }
                }
                ReadinessCheck::Route(target) => {
                    bounded_readiness(route_ready(target), Duration::from_secs(5), cancellation)
                        .await?
                }
                ReadinessCheck::DnsName(name) => {
                    bounded_readiness(dns_ready(name), Duration::from_secs(5), cancellation).await?
                }
                ReadinessCheck::Endpoint(endpoint) => {
                    let (host, port) = parse_endpoint(endpoint)?;
                    bounded_readiness(
                        endpoint_ready(&host, port),
                        Duration::from_secs(3),
                        cancellation,
                    )
                    .await?
                }
            };
            Ok(ReadinessCheckResult {
                check: check.clone(),
                ready,
                detail: if ready {
                    crate::fl!("readiness-ready")
                } else {
                    crate::fl!("readiness-not-ready")
                },
            })
        })
    }
}

async fn bounded_readiness<F>(
    check: F,
    timeout: Duration,
    cancellation: CancellationToken,
) -> Result<bool, VpnError>
where
    F: Future<Output = bool>,
{
    tokio::select! {
        () = cancellation.cancelled() => Err(VpnError::ReadinessCancelled),
        result = tokio::time::timeout(timeout, check) => Ok(result.unwrap_or(false)),
    }
}

async fn route_ready(target: &str) -> bool {
    let addresses = match target.parse::<IpAddr>() {
        Ok(address) => vec![address],
        Err(_) => match tokio::net::lookup_host((target, 9)).await {
            Ok(addresses) => addresses.map(|address| address.ip()).collect(),
            Err(_) => return false,
        },
    };
    for address in addresses {
        let bind_address = match address {
            IpAddr::V4(_) => "0.0.0.0:0",
            IpAddr::V6(_) => "[::]:0",
        };
        let Ok(socket) = UdpSocket::bind(bind_address).await else {
            continue;
        };
        if socket.connect(SocketAddr::new(address, 9)).await.is_ok() {
            return true;
        }
    }
    false
}

async fn dns_ready(name: &str) -> bool {
    tokio::net::lookup_host((name, 0))
        .await
        .is_ok_and(|mut addresses| addresses.next().is_some())
}

async fn endpoint_ready(host: &str, port: u16) -> bool {
    TcpStream::connect((host, port)).await.is_ok()
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

#[cfg(test)]
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
                service_type: None,
            })
        })
        .collect()
}

#[cfg(test)]
fn valid_nmcli_profile_fields(name: &str, vpn_type: &str, uuid: &str) -> bool {
    !name.trim().is_empty()
        && looks_like_uuid(uuid.trim())
        && !vpn_type.chars().any(char::is_whitespace)
        && is_vpn_type(vpn_type)
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

#[cfg(test)]
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

fn parse_endpoint(endpoint: &str) -> Result<(String, u16), VpnError> {
    let Some((host, port)) = endpoint.rsplit_once(':') else {
        return Err(VpnError::InvalidEndpoint);
    };
    let port = port.parse().map_err(|_| VpnError::InvalidEndpoint)?;
    let host = host.trim_matches(['[', ']']).to_owned();
    if host.is_empty() {
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

#[cfg(test)]
mod tests {
    use crate::model::{AccessMode, ConnectionMode, OfflineMirrorConfig, TuningProfile};
    use uuid::Uuid;

    use super::*;

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

    #[test]
    fn network_manager_settings_select_only_vpn_profiles() {
        let settings =
            |name: &str, uuid: &str, connection_type: &str, service_type: Option<&str>| {
                let mut settings = BTreeMap::from([(
                    "connection".to_owned(),
                    BTreeMap::from([
                        (
                            "id".to_owned(),
                            OwnedValue::from(Str::from(name.to_owned())),
                        ),
                        (
                            "uuid".to_owned(),
                            OwnedValue::from(Str::from(uuid.to_owned())),
                        ),
                        (
                            "type".to_owned(),
                            OwnedValue::from(Str::from(connection_type.to_owned())),
                        ),
                    ]),
                )]);
                if let Some(service_type) = service_type {
                    settings.insert(
                        "vpn".to_owned(),
                        BTreeMap::from([(
                            "service-type".to_owned(),
                            OwnedValue::from(Str::from(service_type.to_owned())),
                        )]),
                    );
                }
                settings
            };
        let profile = profile_from_settings(&settings(
            "Work VPN",
            "91a601dd-2df4-4b32-bc66-25a16a7612fe",
            "vpn",
            Some(OPENCONNECT_SERVICE_TYPE),
        ))
        .expect("VPN profile");
        assert_eq!(profile.name, "Work VPN");
        assert_eq!(profile.vpn_type, "vpn");
        assert_eq!(
            profile.service_type.as_deref(),
            Some(OPENCONNECT_SERVICE_TYPE)
        );
        assert!(requires_openconnect_agent(&profile));
        assert!(
            profile_from_settings(&settings(
                "Wi-Fi",
                "9000c4f8-da8d-4cf5-baea-9d747e3161ee",
                "802-11-wireless",
                None,
            ))
            .is_none()
        );

        let wireguard = profile_from_settings(&settings(
            "WireGuard",
            "51424a59-495c-4483-ad44-a0bf49327d5e",
            "wireguard",
            None,
        ))
        .expect("WireGuard profile");
        assert!(!requires_openconnect_agent(&wireguard));
    }

    #[tokio::test]
    #[ignore = "requires the live NetworkManager system bus"]
    async fn live_network_manager_dbus_smoke_test() {
        DbusNetworkManagerVpn::connected()
            .await
            .expect("read NetworkManager state");
        let profiles = DbusNetworkManagerVpn
            .list_profiles(CancellationToken::new())
            .await
            .expect("list saved VPN profiles");
        if let Some(saved) = profiles.first() {
            let profile = VpnProfile {
                id: vpn_id(),
                name: saved.name.clone(),
                kind: VpnKind::NetworkManager,
                external_profile_id: Some(saved.uuid.clone()),
                readiness_checks: Vec::new(),
                timeout_seconds: 30,
            };
            DbusNetworkManagerVpn
                .state(&profile, CancellationToken::new())
                .await
                .expect("read saved VPN state");
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
            smb_preload_override: None,
            sftp_preload_override: None,
            onedrive_identity: None,
            teams_identity: None,
        }
    }

    #[tokio::test]
    async fn readiness_checks_use_in_process_network_apis() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let port = listener.local_addr().unwrap().port();
        let probe = InProcessReadinessProbe;
        let checks = vec![
            ReadinessCheck::Route("127.0.0.1".into()),
            ReadinessCheck::DnsName("localhost".into()),
            ReadinessCheck::Endpoint(format!("127.0.0.1:{port}")),
        ];
        let report = readiness_report(&probe, &checks, CancellationToken::new())
            .await
            .expect("readiness");
        assert!(report.ready);
        assert!(report.checks.iter().all(|result| result.detail == "ready"));

        drop(listener);
        // Port zero is reserved and cannot have a listening TCP endpoint.
        // Reusing the just-released ephemeral port is racy under a parallel
        // test suite because another process may bind it immediately.
        let closed = InProcessReadinessProbe
            .check(
                &ReadinessCheck::Endpoint("127.0.0.1:0".into()),
                CancellationToken::new(),
            )
            .await
            .expect("closed endpoint result");
        assert!(!closed.ready);
        assert_eq!(closed.detail, "not ready");
    }

    #[tokio::test]
    async fn readiness_checks_are_cancellable_and_reject_invalid_endpoints() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result = InProcessReadinessProbe
            .check(&ReadinessCheck::DnsName("localhost".into()), cancellation)
            .await;
        assert_eq!(result, Err(VpnError::ReadinessCancelled));
        assert_eq!(
            InProcessReadinessProbe
                .check(
                    &ReadinessCheck::Endpoint("missing-port".into()),
                    CancellationToken::new(),
                )
                .await,
            Err(VpnError::InvalidEndpoint)
        );
    }

    #[tokio::test]
    async fn network_manager_adapter_rejects_legacy_and_missing_profile_ids_before_dbus() {
        let mut profile = nm_profile();
        profile.kind = VpnKind::LegacyCisco;
        assert_eq!(
            DbusNetworkManagerVpn
                .activate(&profile, CancellationToken::new())
                .await,
            Err(VpnError::InvalidProfileKind)
        );

        profile.kind = VpnKind::NetworkManager;
        profile.external_profile_id = None;
        assert_eq!(
            DbusNetworkManagerVpn
                .activate(&profile, CancellationToken::new())
                .await,
            Err(VpnError::MissingExternalProfileId)
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
    fn legacy_nmcli_parsers_handle_escaping_and_access_modes() {
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
                service_type: None,
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
            AccessMode::OfflineMirror,
            ConnectionMode::OfflineMirror(OfflineMirrorConfig::default()).kind()
        );
    }
}

/// Poll transient readiness failures within one overall authentication deadline.
/// The check itself is bounded by that deadline, not only the delay between checks.
pub async fn poll_readiness_until<F, Fut>(
    deadline: tokio::time::Instant,
    interval: Duration,
    mut check: F,
) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool, String>>,
{
    loop {
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        if tokio::time::timeout_at(deadline, check())
            .await
            .is_ok_and(|result| result.unwrap_or(false))
        {
            return true;
        }
        tokio::time::sleep_until((tokio::time::Instant::now() + interval).min(deadline)).await;
    }
}

#[cfg(test)]
mod readiness_wait_tests {
    use super::*;
    #[tokio::test]
    async fn transient_failure_then_readiness_succeeds() {
        let mut attempts = 0;
        let ready = poll_readiness_until(
            tokio::time::Instant::now() + Duration::from_secs(1),
            Duration::from_millis(1),
            || {
                attempts += 1;
                std::future::ready(match attempts {
                    1 => Err("temporary probe failure".into()),
                    2 => Ok(false),
                    _ => Ok(true),
                })
            },
        )
        .await;
        assert!(ready);
        assert_eq!(attempts, 3);
    }
    #[tokio::test]
    async fn deadline_bounds_a_hung_probe_and_never_permits_storage() {
        let ready = poll_readiness_until(
            tokio::time::Instant::now() + Duration::from_millis(10),
            Duration::from_millis(1),
            std::future::pending::<Result<bool, String>>,
        )
        .await;
        assert!(!ready);
    }
}
