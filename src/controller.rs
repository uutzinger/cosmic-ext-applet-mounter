// SPDX-License-Identifier: MIT

//! Operation state restoration and UI-facing controller decisions.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::config::ConfigDocument;
use crate::diagnostics::{DependencyInventory, DependencyState};
use crate::import::{ImportPreview, LegacyEngine};
use crate::model::{
    AccessMode, ConflictRecord, Connection, ConnectionId, ConnectionMode, ConnectionStatus,
    OfflineMirrorStatus, OnlineMountStatus, Operation, Provider, RecoveryRecord, VpnProfileId,
};
use crate::mounts::{MountEntry, SyncRuntimeState};
use crate::services::{ActiveState, UnitStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerSnapshot {
    pub config: ConfigDocument,
    pub network_ready: bool,
    pub metered_network: bool,
    pub vpn_ready: BTreeMap<VpnProfileId, bool>,
    pub service_status: BTreeMap<ConnectionId, UnitStatus>,
    pub mount_entries: Vec<MountEntry>,
    pub sync_state: BTreeMap<ConnectionId, SyncRuntimeState>,
    pub paused_syncs: BTreeSet<ConnectionId>,
    pub conflicts: Vec<ConflictRecord>,
    pub recoveries: Vec<RecoveryRecord>,
    pub dependencies: Option<DependencyInventory>,
    pub import_previews: Vec<ImportPreview>,
    pub logs: Vec<OperationLogEntry>,
}

impl Default for ControllerSnapshot {
    fn default() -> Self {
        Self {
            config: ConfigDocument::default(),
            network_ready: true,
            metered_network: false,
            vpn_ready: BTreeMap::new(),
            service_status: BTreeMap::new(),
            mount_entries: Vec::new(),
            sync_state: BTreeMap::new(),
            paused_syncs: BTreeSet::new(),
            conflicts: Vec::new(),
            recoveries: Vec::new(),
            dependencies: None,
            import_previews: Vec::new(),
            logs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateKind {
    Empty,
    Healthy,
    Attention,
    Busy,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateStatus {
    pub kind: AggregateKind,
    pub total_connections: usize,
    pub active_connections: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionRowState {
    pub id: ConnectionId,
    pub name: String,
    pub provider: Provider,
    pub mode: AccessMode,
    pub local_path: PathBuf,
    pub vpn_profile_id: Option<VpnProfileId>,
    pub status: ConnectionStatus,
    pub warnings: Vec<String>,
    pub actions: Vec<OperationAction>,
    pub settings: SettingsSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSummary {
    pub remote: String,
    pub remote_subpath: Option<String>,
    pub start_at_login: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
    pub sync_on_metered: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationAction {
    pub operation: Operation,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerViewState {
    pub aggregate: AggregateStatus,
    pub rows: Vec<ConnectionRowState>,
    pub dependency_warnings: Vec<String>,
    pub import_previews: Vec<ImportPreviewRow>,
    pub logs: Vec<OperationLogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecision {
    pub operation: Operation,
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPreviewRow {
    pub unit_name: String,
    pub engine: LegacyEngine,
    pub provider: Provider,
    pub remote: String,
    pub remote_subpath: Option<String>,
    pub local_target: PathBuf,
    pub cache_directory: Option<PathBuf>,
    pub start_at_login: bool,
    pub unsupported_options: Vec<String>,
    pub blocked: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationLogEntry {
    pub stage: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupLayout {
    Narrow,
    Wide,
}

#[must_use]
pub fn restore(snapshot: &ControllerSnapshot) -> ControllerViewState {
    let dependency_warnings = dependency_warnings(snapshot.dependencies.as_ref());
    let rows = snapshot
        .config
        .connections
        .iter()
        .map(|connection| restore_connection(snapshot, connection))
        .collect::<Vec<_>>();
    let aggregate = aggregate(&rows, dependency_warnings.len());
    ControllerViewState {
        aggregate,
        rows,
        dependency_warnings,
        import_previews: snapshot
            .import_previews
            .iter()
            .map(import_preview_row)
            .collect(),
        logs: snapshot.logs.iter().map(sanitize_log).collect(),
    }
}

#[must_use]
pub fn decide_operation(row: &ConnectionRowState, operation: Operation) -> OperationDecision {
    let allowed = row
        .actions
        .iter()
        .find(|action| action.operation == operation)
        .is_some_and(|action| action.enabled);
    let reason = if allowed {
        None
    } else {
        Some(
            match &row.status {
                ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForNetwork)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Offline) => {
                    "waiting for network readiness"
                }
                ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForVpn)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::WaitingForVpn) => {
                    "waiting for VPN readiness"
                }
                ConnectionStatus::OnlineMount(OnlineMountStatus::PendingWrites) => {
                    "pending writes must finish first"
                }
                ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Syncing) => {
                    "synchronization is already running"
                }
                ConnectionStatus::OnlineMount(OnlineMountStatus::Unavailable)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Unavailable) => {
                    "connection is disabled or unavailable"
                }
                _ => "operation is not available in the current state",
            }
            .to_owned(),
        )
    };
    OperationDecision {
        operation,
        allowed,
        reason,
    }
}

#[must_use]
pub fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::OneDrive => "OneDrive",
        Provider::GoogleDrive => "Google Drive",
        Provider::Box => "Box",
        Provider::Smb => "SMB",
    }
}

#[must_use]
pub fn status_label(status: &ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::OnlineMount(OnlineMountStatus::Unmounted) => "Unmounted",
        ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForNetwork) => {
            "Waiting for network"
        }
        ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForVpn) => "Waiting for VPN",
        ConnectionStatus::OnlineMount(OnlineMountStatus::Mounting) => "Mounting",
        ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted) => "Mounted",
        ConnectionStatus::OnlineMount(OnlineMountStatus::PendingWrites) => "Pending writes",
        ConnectionStatus::OnlineMount(OnlineMountStatus::Detaching) => "Detaching",
        ConnectionStatus::OnlineMount(OnlineMountStatus::Error) => "Error",
        ConnectionStatus::OnlineMount(OnlineMountStatus::Unavailable) => "Unavailable",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Idle) => "Idle",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Offline) => "Offline",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::WaitingForVpn) => "Waiting for VPN",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Previewing) => "Previewing",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Syncing) => "Syncing",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Paused) => "Paused",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::MeteredPaused) => {
            "Paused on metered network"
        }
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Conflict) => "Conflict",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Error) => "Error",
        ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Unavailable) => "Unavailable",
    }
}

#[must_use]
pub fn aggregate_label(aggregate: &AggregateStatus) -> String {
    match aggregate.kind {
        AggregateKind::Empty => "No storage connections configured".into(),
        AggregateKind::Healthy => {
            format!(
                "{} of {} connection(s) active",
                aggregate.active_connections, aggregate.total_connections
            )
        }
        AggregateKind::Attention => format!("{} warning(s)", aggregate.warning_count),
        AggregateKind::Busy => "Storage operation in progress".into(),
        AggregateKind::Error => "Storage connection needs attention".into(),
    }
}

#[must_use]
pub fn operation_label(operation: Operation) -> &'static str {
    match operation {
        Operation::Mount => "Mount",
        Operation::Unmount => "Unmount",
        Operation::SyncNow => "Sync Now",
        Operation::PauseSync => "Pause",
        Operation::ResumeSync => "Resume",
        Operation::PreviewInitialSync => "Preview",
        Operation::Repair => "Repair",
    }
}

#[must_use]
pub fn keyboard_label(action: OperationAction) -> String {
    if action.enabled {
        operation_label(action.operation).into()
    } else {
        format!("{} unavailable", operation_label(action.operation))
    }
}

#[must_use]
pub fn row_accessible_text(row: &ConnectionRowState) -> String {
    let actions = row
        .actions
        .iter()
        .map(|action| keyboard_label(*action))
        .collect::<Vec<_>>()
        .join(", ");
    let warnings = if row.warnings.is_empty() {
        "No warnings".into()
    } else {
        format!("Warnings: {}", row.warnings.join("; "))
    };
    format!(
        "{}. {}. {}. Local path {}. Actions: {}. {}.",
        row.name,
        provider_label(row.provider),
        status_label(&row.status),
        row.local_path.display(),
        actions,
        warnings
    )
}

#[must_use]
pub fn popup_layout(width: u16) -> PopupLayout {
    if width < 360 {
        PopupLayout::Narrow
    } else {
        PopupLayout::Wide
    }
}

#[must_use]
pub fn sanitize_log(entry: &OperationLogEntry) -> OperationLogEntry {
    OperationLogEntry {
        stage: redact(&entry.stage),
        summary: redact(&entry.summary),
    }
}

#[must_use]
pub fn import_preview_label(row: &ImportPreviewRow) -> String {
    let state = if row.blocked { "Blocked" } else { "Ready" };
    format!(
        "{} import from {} to {} ({state})",
        provider_label(row.provider),
        row.remote,
        row.local_target.display()
    )
}

fn restore_connection(
    snapshot: &ControllerSnapshot,
    connection: &Connection,
) -> ConnectionRowState {
    let status = match &connection.mode {
        ConnectionMode::OnlineMount(_) => {
            ConnectionStatus::OnlineMount(online_status(snapshot, connection))
        }
        ConnectionMode::OfflineMirror(_) => {
            ConnectionStatus::OfflineMirror(offline_status(snapshot, connection))
        }
    };
    let warnings = warnings(snapshot, connection, &status);
    ConnectionRowState {
        id: connection.id,
        name: connection.name.clone(),
        provider: connection.provider,
        mode: connection.mode.kind(),
        local_path: connection.local_path.clone(),
        vpn_profile_id: connection.vpn_profile_id,
        actions: actions(&status),
        settings: settings(connection),
        status,
        warnings,
    }
}

fn online_status(snapshot: &ControllerSnapshot, connection: &Connection) -> OnlineMountStatus {
    if !connection.enabled {
        return OnlineMountStatus::Unavailable;
    }
    if !snapshot.network_ready {
        return OnlineMountStatus::WaitingForNetwork;
    }
    if !vpn_ready(snapshot, connection.vpn_profile_id) {
        return OnlineMountStatus::WaitingForVpn;
    }
    match snapshot.service_status.get(&connection.id) {
        Some(status) if status.active == ActiveState::Failed => OnlineMountStatus::Error,
        _ if snapshot
            .mount_entries
            .iter()
            .any(|entry| entry.target == connection.local_path) =>
        {
            OnlineMountStatus::Mounted
        }
        Some(status) if matches!(status.active, ActiveState::Active | ActiveState::Activating) => {
            OnlineMountStatus::Mounting
        }
        _ => OnlineMountStatus::Unmounted,
    }
}

fn offline_status(snapshot: &ControllerSnapshot, connection: &Connection) -> OfflineMirrorStatus {
    if !connection.enabled {
        return OfflineMirrorStatus::Unavailable;
    }
    if snapshot
        .conflicts
        .iter()
        .any(|conflict| conflict.connection_id == connection.id)
    {
        return OfflineMirrorStatus::Conflict;
    }
    if snapshot.paused_syncs.contains(&connection.id) {
        return OfflineMirrorStatus::Paused;
    }
    match snapshot.sync_state.get(&connection.id).copied() {
        Some(SyncRuntimeState::Running) => return OfflineMirrorStatus::Syncing,
        Some(SyncRuntimeState::Failed) => return OfflineMirrorStatus::Error,
        _ => {}
    }
    if !snapshot.network_ready {
        return OfflineMirrorStatus::Offline;
    }
    if !vpn_ready(snapshot, connection.vpn_profile_id) {
        return OfflineMirrorStatus::WaitingForVpn;
    }
    if let ConnectionMode::OfflineMirror(options) = &connection.mode
        && snapshot.metered_network
        && !options.sync_on_metered
    {
        return OfflineMirrorStatus::MeteredPaused;
    }
    OfflineMirrorStatus::Idle
}

fn vpn_ready(snapshot: &ControllerSnapshot, profile_id: Option<VpnProfileId>) -> bool {
    profile_id
        .and_then(|id| snapshot.vpn_ready.get(&id).copied())
        .unwrap_or(true)
}

fn actions(status: &ConnectionStatus) -> Vec<OperationAction> {
    use OfflineMirrorStatus as Offline;
    use OnlineMountStatus as Online;

    match status {
        ConnectionStatus::OnlineMount(Online::Mounted) => vec![
            action(Operation::Unmount, true),
            action(Operation::Repair, true),
        ],
        ConnectionStatus::OnlineMount(Online::Unmounted | Online::Error) => vec![
            action(Operation::Mount, true),
            action(Operation::Repair, true),
        ],
        ConnectionStatus::OnlineMount(Online::Mounting) => vec![
            action(Operation::Unmount, true),
            action(Operation::Repair, false),
        ],
        ConnectionStatus::OnlineMount(Online::PendingWrites | Online::Detaching) => {
            vec![action(Operation::Repair, false)]
        }
        ConnectionStatus::OnlineMount(Online::WaitingForVpn) => vec![
            action(Operation::Mount, true),
            action(Operation::Repair, true),
        ],
        ConnectionStatus::OnlineMount(_) => {
            vec![
                action(Operation::Mount, false),
                action(Operation::Repair, true),
            ]
        }
        ConnectionStatus::OfflineMirror(
            Offline::Idle | Offline::Offline | Offline::MeteredPaused,
        ) => {
            vec![
                action(Operation::SyncNow, true),
                action(Operation::PauseSync, true),
                action(Operation::PreviewInitialSync, true),
            ]
        }
        ConnectionStatus::OfflineMirror(Offline::Paused) => vec![
            action(Operation::ResumeSync, true),
            action(Operation::SyncNow, true),
            action(Operation::PreviewInitialSync, true),
        ],
        ConnectionStatus::OfflineMirror(Offline::Syncing) => vec![
            action(Operation::PauseSync, true),
            action(Operation::SyncNow, false),
            action(Operation::Repair, false),
        ],
        ConnectionStatus::OfflineMirror(Offline::Previewing) => vec![
            action(Operation::PauseSync, false),
            action(Operation::Repair, false),
        ],
        ConnectionStatus::OfflineMirror(Offline::Conflict | Offline::Error) => vec![
            action(Operation::SyncNow, false),
            action(Operation::Repair, true),
        ],
        ConnectionStatus::OfflineMirror(Offline::WaitingForVpn | Offline::Unavailable) => vec![
            action(Operation::SyncNow, false),
            action(Operation::Repair, true),
        ],
    }
}

const fn action(operation: Operation, enabled: bool) -> OperationAction {
    OperationAction { operation, enabled }
}

fn settings(connection: &Connection) -> SettingsSummary {
    match &connection.mode {
        ConnectionMode::OnlineMount(options) => SettingsSummary {
            remote: connection.remote_reference.clone(),
            remote_subpath: connection.remote_subpath.clone(),
            start_at_login: Some(options.start_at_login),
            sync_interval_minutes: None,
            sync_on_metered: None,
        },
        ConnectionMode::OfflineMirror(options) => SettingsSummary {
            remote: connection.remote_reference.clone(),
            remote_subpath: connection.remote_subpath.clone(),
            start_at_login: None,
            sync_interval_minutes: Some(options.sync_interval_minutes),
            sync_on_metered: Some(options.sync_on_metered),
        },
    }
}

fn warnings(
    snapshot: &ControllerSnapshot,
    connection: &Connection,
    status: &ConnectionStatus,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if matches!(
        status,
        ConnectionStatus::OnlineMount(OnlineMountStatus::Error)
            | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Error)
    ) {
        warnings.push("service or runtime reported an error".into());
    }
    if snapshot
        .recoveries
        .iter()
        .any(|record| record.connection_id == connection.id)
    {
        warnings.push("recovery files are retained for this connection".into());
    }
    if snapshot
        .conflicts
        .iter()
        .any(|conflict| conflict.connection_id == connection.id)
    {
        warnings.push("conflicts require review".into());
    }
    warnings
}

fn aggregate(rows: &[ConnectionRowState], dependency_warning_count: usize) -> AggregateStatus {
    let total_connections = rows.len();
    let active_connections = rows
        .iter()
        .filter(|row| {
            matches!(
                row.status,
                ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Idle)
            )
        })
        .count();
    let warning_count =
        rows.iter().map(|row| row.warnings.len()).sum::<usize>() + dependency_warning_count;
    let kind = if rows.is_empty() {
        AggregateKind::Empty
    } else if rows.iter().any(|row| {
        matches!(
            row.status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Error)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Error)
        )
    }) {
        AggregateKind::Error
    } else if rows.iter().any(|row| {
        matches!(
            row.status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Mounting)
                | ConnectionStatus::OnlineMount(OnlineMountStatus::Detaching)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Previewing)
                | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Syncing)
        )
    }) {
        AggregateKind::Busy
    } else if warning_count > 0
        || rows.iter().any(|row| {
            matches!(
                row.status,
                ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForNetwork)
                    | ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForVpn)
                    | ConnectionStatus::OnlineMount(OnlineMountStatus::Unavailable)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Offline)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::WaitingForVpn)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::MeteredPaused)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Conflict)
                    | ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Unavailable)
            )
        })
    {
        AggregateKind::Attention
    } else {
        AggregateKind::Healthy
    };
    AggregateStatus {
        kind,
        total_connections,
        active_connections,
        warning_count,
    }
}

fn dependency_warnings(inventory: Option<&DependencyInventory>) -> Vec<String> {
    inventory
        .into_iter()
        .flat_map(|inventory| &inventory.reports)
        .filter(|report| report.required && report.state != DependencyState::Available)
        .map(|report| format!("{:?}: {}", report.kind, report.summary))
        .collect()
}

fn import_preview_row(preview: &ImportPreview) -> ImportPreviewRow {
    let mut warnings = Vec::new();
    if preview.active_conflict {
        warnings.push("original service is active".into());
    }
    if preview.local_target_conflict {
        warnings.push("local target conflicts with an existing connection".into());
    }
    if !preview.unsupported_options.is_empty() {
        warnings.push(format!(
            "unsupported options: {}",
            preview.unsupported_options.join(", ")
        ));
    }
    ImportPreviewRow {
        unit_name: preview.original_unit_name.clone(),
        engine: preview.engine,
        provider: preview.provider,
        remote: preview.remote_reference.clone(),
        remote_subpath: preview.remote_subpath.clone(),
        local_target: preview.local_target.clone(),
        cache_directory: preview.cache_directory.clone(),
        start_at_login: preview.start_at_login,
        unsupported_options: preview.unsupported_options.clone(),
        blocked: preview.active_conflict || preview.local_target_conflict,
        warnings,
    }
}

fn redact(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            if lower.starts_with("http://") || lower.starts_with("https://") {
                "[REDACTED_URL]".into()
            } else if lower.contains("token")
                || lower.contains("secret")
                || lower.contains("password")
                || lower.contains("credential")
            {
                if let Some((key, _)) = token.split_once('=') {
                    format!("{key}=[REDACTED]")
                } else {
                    "[REDACTED]".into()
                }
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::diagnostics::{DependencyKind, DependencyReport};
    use crate::import::{ImportPreview, LegacyEngine};
    use crate::model::{OfflineMirrorConfig, OnlineMountConfig, TuningProfile};
    use semver::Version;

    fn id(value: &str) -> ConnectionId {
        ConnectionId::from_uuid(Uuid::parse_str(value).expect("UUID"))
    }

    fn online(connection_id: ConnectionId) -> Connection {
        Connection {
            id: connection_id,
            name: "Online".into(),
            provider: Provider::GoogleDrive,
            mode: ConnectionMode::OnlineMount(OnlineMountConfig::default()),
            remote_reference: "remote".into(),
            remote_subpath: None,
            local_path: PathBuf::from("/home/example/Cloud/Online"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
        }
    }

    fn offline(connection_id: ConnectionId) -> Connection {
        Connection {
            id: connection_id,
            name: "Offline".into(),
            provider: Provider::Box,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory: PathBuf::from("/home/example/.local/share/recovery"),
                sync_interval_minutes: 15,
                sync_on_metered: false,
            }),
            remote_reference: "box".into(),
            remote_subpath: Some("Projects".into()),
            local_path: PathBuf::from("/home/example/Cloud/Offline"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
        }
    }

    #[test]
    fn restores_online_mount_from_actual_mount_table() {
        let connection = online(id("2a3f5d45-e867-47e7-943f-66cf60e777ad"));
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            mount_entries: vec![MountEntry {
                target: connection.local_path.clone(),
                source: "remote:".into(),
                filesystem: "fuse.rclone".into(),
                options: vec![],
            }],
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(state.aggregate.kind, AggregateKind::Healthy);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted)
        );
        assert!(decide_operation(&state.rows[0], Operation::Unmount).allowed);
    }

    #[test]
    fn service_active_without_mount_is_mounting() {
        let connection = online(id("2a3f5d45-e867-47e7-943f-66cf60e777ad"));
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            service_status: [(
                connection.id,
                UnitStatus {
                    active: ActiveState::Active,
                    enabled: true,
                    detail: "running".into(),
                },
            )]
            .into_iter()
            .collect(),
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(state.aggregate.kind, AggregateKind::Busy);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Mounting)
        );
    }

    #[test]
    fn failed_service_with_lingering_mount_is_error() {
        let connection = online(id("2a3f5d45-e867-47e7-943f-66cf60e777ad"));
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            service_status: [(
                connection.id,
                UnitStatus {
                    active: ActiveState::Failed,
                    enabled: false,
                    detail: "clean unmount failed".into(),
                },
            )]
            .into_iter()
            .collect(),
            mount_entries: vec![MountEntry {
                target: connection.local_path.clone(),
                source: "onedriver".into(),
                filesystem: "fuse.onedriver".into(),
                options: vec![],
            }],
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::Error)
        );
        assert!(decide_operation(&state.rows[0], Operation::Mount).allowed);
    }

    #[test]
    fn restores_offline_sync_and_metered_states() {
        let connection = offline(id("3815551b-93f0-4731-ac61-b303bbff3260"));
        let mut snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            sync_state: [(connection.id, SyncRuntimeState::Running)]
                .into_iter()
                .collect(),
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Syncing)
        );
        assert!(!decide_operation(&state.rows[0], Operation::SyncNow).allowed);

        snapshot.sync_state.clear();
        snapshot.metered_network = true;
        let state = restore(&snapshot);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OfflineMirror(OfflineMirrorStatus::MeteredPaused)
        );
        assert!(decide_operation(&state.rows[0], Operation::SyncNow).allowed);
    }

    #[test]
    fn vpn_and_network_readiness_block_operations() {
        let mut connection = online(id("2a3f5d45-e867-47e7-943f-66cf60e777ad"));
        let vpn_id = VpnProfileId::from_uuid(
            Uuid::parse_str("17ea4cc5-f4f0-405b-b112-dad6f855bb77").expect("UUID"),
        );
        connection.vpn_profile_id = Some(vpn_id);
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
            vpn_ready: [(vpn_id, false)].into_iter().collect(),
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OnlineMount(OnlineMountStatus::WaitingForVpn)
        );
        let decision = decide_operation(&state.rows[0], Operation::Mount);
        assert!(decision.allowed);
        assert_eq!(decision.reason, None);
    }

    #[test]
    fn conflicts_recovery_and_dependency_warnings_are_exposed() {
        let connection = offline(id("3815551b-93f0-4731-ac61-b303bbff3260"));
        let snapshot = ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection.clone()],
                ..ConfigDocument::default()
            },
            conflicts: vec![ConflictRecord {
                connection_id: connection.id,
                relative_path: "file.txt".into(),
                preserved_local_path: "/tmp/local".into(),
                preserved_remote_path: "/tmp/remote".into(),
            }],
            dependencies: Some(DependencyInventory {
                reports: vec![DependencyReport {
                    kind: DependencyKind::Rclone,
                    required: true,
                    state: DependencyState::Outdated {
                        minimum: Version::new(1, 74, 3),
                    },
                    version: Some(Version::new(1, 60, 1)),
                    path: Some("/usr/bin/rclone".into()),
                    capabilities: BTreeSet::new(),
                    missing_capabilities: BTreeSet::new(),
                    summary: "Upgrade required".into(),
                    guidance_url: "https://rclone.org/downloads/",
                }],
            }),
            ..ControllerSnapshot::default()
        };
        let state = restore(&snapshot);
        assert_eq!(state.aggregate.kind, AggregateKind::Attention);
        assert_eq!(
            state.rows[0].status,
            ConnectionStatus::OfflineMirror(OfflineMirrorStatus::Conflict)
        );
        assert_eq!(state.dependency_warnings.len(), 1);
    }

    #[test]
    fn labels_are_stable_for_popup_rows() {
        assert_eq!(provider_label(Provider::Smb), "SMB");
        assert_eq!(
            status_label(&ConnectionStatus::OnlineMount(OnlineMountStatus::Mounted)),
            "Mounted"
        );
        assert_eq!(
            aggregate_label(&AggregateStatus {
                kind: AggregateKind::Empty,
                total_connections: 0,
                active_connections: 0,
                warning_count: 0,
            }),
            "No storage connections configured"
        );
        assert_eq!(operation_label(Operation::SyncNow), "Sync Now");
        assert_eq!(
            keyboard_label(OperationAction {
                operation: Operation::Mount,
                enabled: false,
            }),
            "Mount unavailable"
        );
        assert_eq!(popup_layout(320), PopupLayout::Narrow);
        assert_eq!(popup_layout(420), PopupLayout::Wide);
    }

    #[test]
    fn import_previews_and_sanitized_logs_are_ui_visible() {
        let preview = ImportPreview {
            original_path: "/home/example/.config/systemd/user/rclone.service".into(),
            original_unit_name: "rclone.service".into(),
            engine: LegacyEngine::RcloneMount,
            provider: Provider::GoogleDrive,
            remote_reference: "gdrive".into(),
            remote_subpath: Some("Projects".into()),
            local_target: "/home/example/Cloud/GDrive".into(),
            cache_directory: Some("/home/example/.cache/rclone".into()),
            start_at_login: true,
            unsupported_options: vec!["--daemon".into()],
            active_conflict: true,
            local_target_conflict: false,
            connection: online(id("2a3f5d45-e867-47e7-943f-66cf60e777ad")),
        };
        let state = restore(&ControllerSnapshot {
            import_previews: vec![preview],
            logs: vec![OperationLogEntry {
                stage: "auth token=abc".into(),
                summary: "open https://example.test/callback password=hunter2".into(),
            }],
            ..ControllerSnapshot::default()
        });
        assert_eq!(state.import_previews.len(), 1);
        assert!(state.import_previews[0].blocked);
        assert!(import_preview_label(&state.import_previews[0]).contains("Blocked"));
        assert_eq!(state.logs[0].stage, "auth token=[REDACTED]");
        assert_eq!(
            state.logs[0].summary,
            "open [REDACTED_URL] password=[REDACTED]"
        );
    }

    #[test]
    fn row_accessible_text_does_not_depend_on_color() {
        let connection = offline(id("3815551b-93f0-4731-ac61-b303bbff3260"));
        let state = restore(&ControllerSnapshot {
            config: ConfigDocument {
                connections: vec![connection],
                ..ConfigDocument::default()
            },
            metered_network: true,
            ..ControllerSnapshot::default()
        });
        let text = row_accessible_text(&state.rows[0]);
        assert!(text.contains("Paused on metered network"));
        assert!(text.contains("Sync Now"));
        assert!(text.contains("No warnings"));
    }
}
