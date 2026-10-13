// SPDX-License-Identifier: MIT

//! Scheduled bisync service and progress artifact definitions.
//!
//! Bisync execution is implemented by `bisync_coordinator`; this module no
//! longer renders or launches a host shell script.

use std::path::PathBuf;

use crate::bisync_coordinator::BisyncMode;
use crate::bisync_worker;
use crate::services::ServiceSpec;
use crate::sync::RcloneBisyncPlan;

const LIVE_PROGRESS_LOG: &str = "live-progress.jsonl";
const LIVE_PROGRESS_STATE: &str = "live-progress-state.json";

#[must_use]
pub fn live_progress_log_path(work_directory: &std::path::Path) -> PathBuf {
    work_directory.join(LIVE_PROGRESS_LOG)
}

#[must_use]
pub fn live_progress_state_path(work_directory: &std::path::Path) -> PathBuf {
    work_directory.join(LIVE_PROGRESS_STATE)
}

pub fn service_spec(plan: &RcloneBisyncPlan) -> Result<ServiceSpec, String> {
    let mut service = plan.service.clone();
    let command = bisync_worker::service_command(BisyncMode::Scheduled, plan.connection_id)?;
    service.executable = command.executable;
    service.arguments = command.arguments;
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TuningProfile,
    };
    use crate::sync::rclone_bisync_plan;

    #[test]
    fn scheduled_service_uses_only_the_restricted_worker_boundary() {
        let root = tempfile::tempdir().unwrap();
        let connection = Connection {
            id: ConnectionId::new(),
            name: "Scheduled mirror".into(),
            provider: Provider::Sftp,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory: root.path().join("recovery"),
                sync_interval_minutes: 15,
                sync_on_metered: false,
            }),
            remote_reference: "test_sftp".into(),
            remote_subpath: Some("mirror".into()),
            local_path: root.path().join("local"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            teams_identity: None,
        };
        let plan = rclone_bisync_plan(&connection, &root.path().join("state")).unwrap();
        let service = service_spec(&plan).unwrap();
        assert_eq!(
            service.arguments,
            vec![
                bisync_worker::WORKER_FLAG.to_owned(),
                "scheduled".to_owned(),
                connection.id.to_string(),
            ]
        );
        let condition = service
            .pre_start_condition
            .as_ref()
            .expect("scheduled readiness guard");
        assert_eq!(
            condition.arguments,
            vec![
                crate::vpn::OFFLINE_READINESS_GUARD_FLAG.to_owned(),
                connection.id.to_string(),
            ]
        );
        let document = crate::services::UnitDocument::service(&service).unwrap();
        assert!(document.content.find("ExecCondition=") < document.content.find("ExecStart="));
        assert!(
            !service
                .arguments
                .iter()
                .any(|argument| argument.contains("sh"))
        );
        assert!(
            !service
                .arguments
                .iter()
                .any(|argument| argument.contains("rclone"))
        );
    }
}
