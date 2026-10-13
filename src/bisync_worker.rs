// SPDX-License-Identifier: MIT

//! Restricted entry point for scheduled rclone Offline Mirror work.
//!
//! The caller supplies only a fixed mode and connection UUID. All paths and
//! rclone arguments are reconstructed from authoritative saved configuration.

use crate::bisync_coordinator::{self, BisyncMode};
use crate::model::ConnectionId;
use crate::mount_guard;
use crate::process::RuntimeCommandRunner;
use crate::services::ServiceCommand;
use tokio_util::sync::CancellationToken;

pub const WORKER_FLAG: &str = "--run-bisync";
pub const PROBE_MODE: &str = "probe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerRequest {
    Probe(ConnectionId),
    Run(BisyncMode, ConnectionId),
}

pub fn parse_request(arguments: &[String]) -> Result<WorkerRequest, String> {
    let [mode, id] = arguments else {
        return Err(crate::fl!(
            "ui-bisync-worker-bisync-worker-requires-exactly-one-mode-and"
        ));
    };
    let id = uuid::Uuid::parse_str(id)
        .map(ConnectionId::from_uuid)
        .map_err(|_| {
            crate::fl!("ui-bisync-worker-bisync-worker-received-an-invalid-connection-uuid")
        })?;
    if mode == PROBE_MODE {
        Ok(WorkerRequest::Probe(id))
    } else {
        BisyncMode::parse(mode)
            .map(|mode| WorkerRequest::Run(mode, id))
            .ok_or_else(|| {
                crate::fl!("ui-bisync-worker-bisync-worker-received-an-unsupported-mode")
            })
    }
}

pub fn run(arguments: &[String]) -> Result<String, String> {
    match parse_request(arguments)? {
        WorkerRequest::Probe(id) => Ok(format!("bisync-worker-probe-ok {id}")),
        WorkerRequest::Run(mode, id) => {
            let runtime = tokio::runtime::Runtime::new().map_err(|error| {
                crate::fl!(
                    "ui-bisync-worker-could-not-start-bisync-worker-runtime-error",
                    error = (error).to_string()
                )
            })?;
            runtime.block_on(async move {
                bisync_coordinator::run_saved(
                    id,
                    mode,
                    &bisync_coordinator::default_work_root(),
                    &RuntimeCommandRunner::detect_current(),
                    CancellationToken::new(),
                )
                .await
                .map(|outcome| {
                    let mut summary = format!("bisync-worker-completed {} {id}", mode.as_str());
                    if !outcome.warnings.is_empty() {
                        summary.push_str(" warning: ");
                        summary.push_str(&outcome.warnings.join(" "));
                    }
                    summary
                })
            })
        }
    }
}

pub fn probe_service_command(id: ConnectionId) -> Result<ServiceCommand, String> {
    mount_guard::service_command(vec![WORKER_FLAG.into(), PROBE_MODE.into(), id.to_string()])
}

pub fn service_command(mode: BisyncMode, id: ConnectionId) -> Result<ServiceCommand, String> {
    mount_guard::service_command(vec![
        WORKER_FLAG.into(),
        mode.as_str().into(),
        id.to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "2a3f5d45-e867-47e7-943f-66cf60e777ad";

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn probe_accepts_only_a_mode_and_uuid() {
        let request = parse_request(&arguments(&[PROBE_MODE, ID])).unwrap();
        assert_eq!(
            request,
            WorkerRequest::Probe(ConnectionId::from_uuid(uuid::Uuid::parse_str(ID).unwrap()))
        );
    }

    #[test]
    fn probe_rejects_missing_extra_and_malformed_arguments() {
        assert!(parse_request(&arguments(&[])).is_err());
        assert!(parse_request(&arguments(&[PROBE_MODE])).is_err());
        assert!(parse_request(&arguments(&[PROBE_MODE, ID, "extra"])).is_err());
        assert_eq!(
            parse_request(&arguments(&["scheduled", ID])).unwrap(),
            WorkerRequest::Run(
                BisyncMode::Scheduled,
                ConnectionId::from_uuid(uuid::Uuid::parse_str(ID).unwrap())
            )
        );
        assert!(parse_request(&arguments(&["command", ID])).is_err());
        assert!(parse_request(&arguments(&[PROBE_MODE, "not-a-uuid"])).is_err());
    }

    #[test]
    fn probe_result_is_bounded_and_contains_no_caller_supplied_path() {
        let output = run(&arguments(&[PROBE_MODE, ID])).unwrap();
        assert_eq!(output, format!("bisync-worker-probe-ok {ID}"));
        assert!(output.len() < 128);
    }

    #[test]
    fn probe_service_command_exposes_only_the_fixed_request() {
        let id = ConnectionId::from_uuid(uuid::Uuid::parse_str(ID).unwrap());
        let command = probe_service_command(id).unwrap();
        assert_eq!(command.arguments, arguments(&[WORKER_FLAG, PROBE_MODE, ID]));
    }

    #[test]
    fn production_service_command_exposes_only_mode_and_identity() {
        let id = ConnectionId::from_uuid(uuid::Uuid::parse_str(ID).unwrap());
        let command = service_command(BisyncMode::Scheduled, id).unwrap();
        assert_eq!(
            command.arguments,
            arguments(&[WORKER_FLAG, "scheduled", ID])
        );
    }
}
