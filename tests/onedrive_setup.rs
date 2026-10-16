// SPDX-License-Identifier: MIT
//! Production setup and guards with synthetic remote configuration and Graph data.
use cosmic_ext_applet_mounter::{model::*, onedrive::*, process::*};
use serde_json::{Value, json};
use std::{collections::VecDeque, sync::Mutex, time::Duration};
use tokio_util::sync::CancellationToken;

fn output(text: &str) -> CommandOutput {
    CommandOutput {
        command: "synthetic".into(),
        stdout: CapturedOutput {
            text: text.into(),
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
fn config(kind: DriveType, drive: &str) -> String {
    json!({"example_remote":{"type":"onedrive","drive_type":kind.as_str(),"drive_id":drive,
        "token":{"access_token":"synthetic-token"}}})
    .to_string()
}
fn runner_with_config(kind: DriveType, drive: &str) -> FakeCommandRunner {
    let runner = FakeCommandRunner::default();
    runner.push(Ok(output("{}")));
    runner.push(Ok(output(&config(kind, drive))));
    runner
}
fn primary_drive(kind: DriveType, id: &str) -> Value {
    json!({"id":id,"driveType":kind.as_str(),"owner":{"user":{"id":"synthetic-owner"}}})
}
struct FakeGraph {
    responses: Mutex<VecDeque<OneDriveResult<Value>>>,
    calls: Mutex<Vec<&'static str>>,
}
impl FakeGraph {
    fn new(responses: impl IntoIterator<Item = OneDriveResult<Value>>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Default::default(),
        }
    }
    fn account(kind: DriveType) -> Self {
        Self::new(if kind == DriveType::Personal {
            vec![
                Ok(json!({"id":"synthetic-user","mail":"person@example.invalid"})),
                Ok(primary_drive(kind, "primary")),
            ]
        } else {
            vec![
                Ok(primary_drive(kind, "primary")),
                Ok(primary_drive(kind, "primary")),
            ]
        })
    }
}
impl Graph for FakeGraph {
    fn get<'a>(
        &'a self,
        target: GraphTarget<'a>,
        _token: &'a str,
        cancel: CancellationToken,
    ) -> GraphFuture<'a> {
        Box::pin(async move {
            if cancel.is_cancelled() {
                return Err(OneDriveError::Cancelled);
            }
            self.calls.lock().unwrap().push(match target {
                GraphTarget::Profile => "profile",
                GraphTarget::PrimaryDrive => "primary",
                GraphTarget::ExpectedBusinessDrive(_) => "expected",
                GraphTarget::Folder { .. } => "folder",
            });
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected Graph request")
        })
    }
}
fn setup(kind: DriveType) -> RemoteSetup {
    RemoteSetup::new("example_remote", kind, "person@example.invalid").unwrap()
}
fn connection(kind: DriveType, root: &std::path::Path) -> Connection {
    Connection {
        id: ConnectionId::new(),
        name: "Synthetic mirror".into(),
        provider: Provider::OneDrive,
        mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            engine: Some(OfflineEngine::Rclone),
            recovery_directory: root.join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }),
        remote_reference: "example_remote".into(),
        remote_subpath: Some("Test folder".into()),
        local_path: root.join("local"),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
        smb_preload_override: None,
        sftp_preload_override: None,
        onedrive_identity: Some(OneDriveIdentity {
            drive_id: "primary".into(),
            drive_type: kind,
        }),
        teams_identity: None,
    }
}
fn folder() -> Value {
    json!({"id":"folder-id","folder":{},"parentReference":{"driveId":"primary"}})
}

#[tokio::test]
async fn onedrive_setup_creates_production_names_with_default_client_and_required_account_checks() {
    for kind in [DriveType::Personal, DriveType::Business] {
        let runner = FakeCommandRunner::default();
        for text in [
            "{}".to_owned(),
            r#"{"State":""}"#.into(),
            "{}".into(),
            config(kind, "primary"),
        ] {
            runner.push(Ok(output(&text)));
        }
        let graph = FakeGraph::account(kind);
        let verified = authenticate(&runner, &graph, &setup(kind), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            verified.identity,
            OneDriveIdentity {
                drive_id: "primary".into(),
                drive_type: kind
            }
        );
        let requests = runner.requests();
        let create = requests[1].sanitized_command();
        let args = create.split_whitespace().collect::<Vec<_>>();
        assert!(
            args.windows(2)
                .any(|pair| pair == ["config_type", "onedrive"])
        );
        assert_eq!(
            args.iter().any(|arg| arg.contains("User.Read")),
            kind == DriveType::Personal
        );
        assert!(!args.contains(&"client_id"));
        assert_eq!(
            *graph.calls.lock().unwrap(),
            if kind == DriveType::Personal {
                vec!["profile", "primary"]
            } else {
                vec!["primary", "expected"]
            }
        );
    }
}

#[tokio::test]
async fn onedrive_setup_existing_remote_and_precancelled_setup_do_not_mutate() {
    let runner = FakeCommandRunner::default();
    runner.push(Ok(output(&config(DriveType::Personal, "primary"))));
    assert_eq!(
        authenticate(
            &runner,
            &FakeGraph::new([]),
            &setup(DriveType::Personal),
            CancellationToken::new()
        )
        .await
        .unwrap_err(),
        OneDriveError::ExistingRemote
    );
    assert_eq!(runner.requests().len(), 1);
    let runner = FakeCommandRunner::default();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert_eq!(
        authenticate(
            &runner,
            &FakeGraph::new([]),
            &setup(DriveType::Business),
            cancel
        )
        .await
        .unwrap_err(),
        OneDriveError::Cancelled
    );
    assert!(runner.requests().is_empty());
}

#[tokio::test]
async fn onedrive_setup_replacement_drive_mismatch_rolls_back_only_new_remote() {
    let kind = DriveType::Business;
    let runner = FakeCommandRunner::default();
    for text in [
        "{}".into(),
        r#"{"State":""}"#.into(),
        "{}".into(),
        config(kind, "primary"),
        json!({"example_remote":{},"retained_old_remote":{}}).to_string(),
        "".into(),
    ] {
        runner.push(Ok(output(&text)));
    }
    let mut setup = setup(kind);
    setup.previous_identity = Some(OneDriveIdentity {
        drive_id: "saved-different-primary".into(),
        drive_type: kind,
    });
    assert_eq!(
        authenticate(
            &runner,
            &FakeGraph::account(kind),
            &setup,
            CancellationToken::new()
        )
        .await
        .unwrap_err(),
        OneDriveError::WrongDrive
    );
    let requests = runner.requests();
    assert!(
        requests
            .last()
            .unwrap()
            .sanitized_command()
            .ends_with("config delete example_remote")
    );
    assert!(
        requests
            .iter()
            .all(|request| !request.sanitized_command().contains("retained_old_remote"))
    );
}

#[tokio::test]
async fn onedrive_setup_existing_inspection_is_read_only_and_rejects_saved_drive_change() {
    let mut setup = setup(DriveType::Personal);
    setup.previous_identity = Some(OneDriveIdentity {
        drive_id: "other".into(),
        drive_type: DriveType::Personal,
    });
    let runner = runner_with_config(DriveType::Personal, "primary");
    assert_eq!(
        verify_existing(
            &runner,
            &FakeGraph::account(DriveType::Personal),
            &setup,
            CancellationToken::new()
        )
        .await
        .unwrap_err(),
        OneDriveError::WrongDrive
    );
    assert_eq!(runner.requests().len(), 2);
}

#[tokio::test]
async fn onedrive_guard_checks_saved_resource_and_folder_without_profile() {
    let temp = tempfile::tempdir().unwrap();
    for kind in [DriveType::Personal, DriveType::Business] {
        let connection = connection(kind, temp.path());
        let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
        let runner = runner_with_config(kind, "primary");
        runner.push(Ok(output(r#"{"IsDir":true}"#)));
        let graph = FakeGraph::new([Ok(primary_drive(kind, "primary")), Ok(folder())]);
        verify_mirror_binding(
            &binding,
            &connection,
            &runner,
            &graph,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(*graph.calls.lock().unwrap(), ["primary", "folder"]);
        assert_eq!(runner.requests().len(), 3);
    }
}

#[tokio::test]
async fn onedrive_guard_saved_target_edits_stop_before_remote_access() {
    let temp = tempfile::tempdir().unwrap();
    let connection = connection(DriveType::Business, temp.path());
    let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
    for change in 0..8 {
        let mut changed = connection.clone();
        match change {
            0 => changed.id = ConnectionId::new(),
            1 => changed.enabled = false,
            2 => changed.remote_reference = "other".into(),
            3 => changed.remote_subpath = Some("Other".into()),
            4 => changed.local_path = temp.path().join("other"),
            5 => {
                let ConnectionMode::OfflineMirror(ref mut options) = changed.mode else {
                    unreachable!()
                };
                options.recovery_directory = temp.path().join("other-recovery");
            }
            6 => changed.onedrive_identity.as_mut().unwrap().drive_id = "other".into(),
            _ => {
                let ConnectionMode::OfflineMirror(ref mut options) = changed.mode else {
                    unreachable!()
                };
                options.engine = None;
            }
        }
        let runner = FakeCommandRunner::default();
        assert!(
            verify_mirror_binding(
                &binding,
                &changed,
                &runner,
                &FakeGraph::new([]),
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert!(runner.requests().is_empty());
    }
}

#[tokio::test]
async fn onedrive_guard_rejects_remapped_config_and_denied_or_malformed_graph_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let connection = connection(DriveType::Business, temp.path());
    let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
    let runner = runner_with_config(DriveType::Business, "other");
    assert_eq!(
        verify_mirror_binding(
            &binding,
            &connection,
            &runner,
            &FakeGraph::new([]),
            CancellationToken::new()
        )
        .await,
        Err(OneDriveError::WrongDrive)
    );
    for value in [
        Err(OneDriveError::GraphHttp(403)),
        Ok(json!({})),
        Ok(primary_drive(DriveType::Personal, "primary")),
        Ok(primary_drive(DriveType::Business, "other")),
    ] {
        let runner = runner_with_config(DriveType::Business, "primary");
        let graph = FakeGraph::new([value]);
        assert!(
            verify_mirror_binding(
                &binding,
                &connection,
                &runner,
                &graph,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert_eq!(*graph.calls.lock().unwrap(), ["primary"]);
        assert_eq!(runner.requests().len(), 2);
    }
}

#[tokio::test]
async fn onedrive_guard_rejects_shortcut_notebook_foreign_folder_and_incomplete_stat() {
    let temp = tempfile::tempdir().unwrap();
    let connection = connection(DriveType::Personal, temp.path());
    let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
    for field in ["remoteItem", "package", "parentReference", "folder"] {
        let mut value = folder();
        value[field] = if field == "folder" {
            Value::Null
        } else {
            json!({})
        };
        let runner = runner_with_config(DriveType::Personal, "primary");
        let graph = FakeGraph::new([Ok(primary_drive(DriveType::Personal, "primary")), Ok(value)]);
        assert!(
            verify_mirror_binding(
                &binding,
                &connection,
                &runner,
                &graph,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert_eq!(runner.requests().len(), 2);
    }
    for stat in [r#"{"IsDir":false}"#, r#"{}"#, "broken"] {
        let runner = runner_with_config(DriveType::Personal, "primary");
        runner.push(Ok(output(stat)));
        let graph = FakeGraph::new([
            Ok(primary_drive(DriveType::Personal, "primary")),
            Ok(folder()),
        ]);
        assert!(
            verify_mirror_binding(
                &binding,
                &connection,
                &runner,
                &graph,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
    }
    let runner = runner_with_config(DriveType::Personal, "primary");
    let mut stat = output(r#"{"IsDir":true}"#);
    stat.stdout.truncated = true;
    runner.push(Ok(stat));
    let graph = FakeGraph::new([
        Ok(primary_drive(DriveType::Personal, "primary")),
        Ok(folder()),
    ]);
    assert_eq!(
        verify_mirror_binding(
            &binding,
            &connection,
            &runner,
            &graph,
            CancellationToken::new()
        )
        .await,
        Err(OneDriveError::IncompleteOutput)
    );
}

#[test]
fn onedrive_plan_binds_initialization_to_target_and_excludes_browser_items() {
    use cosmic_ext_applet_mounter::sync::{rclone_bisync_filter_file, rclone_bisync_plan};
    let temp = tempfile::tempdir().unwrap();
    let connection = connection(DriveType::Business, temp.path());
    let plan = rclone_bisync_plan(&connection, &temp.path().join("work")).unwrap();
    assert!(plan.preflight_command.is_some());
    assert!(!plan.verify_after_sync);
    assert!(
        plan.remote_recovery_path
            .contains("Test folder/.cosmic-mounter-recovery/")
    );
    let args = &plan.service.arguments;
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--compare", "size,modtime"])
    );
    assert!(args.windows(2).any(|pair| pair == ["--max-delete", "50"]));
    assert!(!args.iter().any(|arg| matches!(
        arg.as_str(),
        "--ignore-size" | "--ignore-checksum" | "--force"
    )));
    for change in 0..4 {
        let mut changed = connection.clone();
        match change {
            0 => changed.remote_subpath = Some("Other".into()),
            1 => changed.onedrive_identity.as_mut().unwrap().drive_id = "other".into(),
            2 => changed.local_path = temp.path().join("other"),
            _ => changed.onedrive_identity.as_mut().unwrap().drive_type = DriveType::Personal,
        }
        let other = rclone_bisync_plan(&changed, &temp.path().join("work")).unwrap();
        assert_ne!(plan.work_directory, other.work_directory);
    }
    let filters = rclone_bisync_filter_file(Provider::OneDrive);
    for pattern in [
        "- /.cosmic-mounter-recovery/",
        "- **.one",
        "- **.onepkg",
        "- **.lnk",
        "- **.url",
    ] {
        assert!(filters.contains(pattern));
    }
}

#[tokio::test]
async fn onedrive_setup_failed_partial_cleanup_is_explicit_and_safe() {
    let runner = FakeCommandRunner::default();
    runner.push(Ok(output("{}")));
    runner.push(Err(CommandError::Cancelled {
        command: "synthetic-secret".into(),
    }));
    runner.push(Ok(output(&config(DriveType::Business, "primary"))));
    runner.push(Err(CommandError::Timeout {
        command: "synthetic-secret".into(),
        timeout: Duration::from_secs(10),
    }));
    let error = authenticate(
        &runner,
        &FakeGraph::new([]),
        &setup(DriveType::Business),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert_eq!(error, OneDriveError::CleanupRequired);
    assert!(!error.to_string().contains("synthetic-secret"));
    assert_eq!(runner.requests().len(), 4);
}

#[tokio::test]
async fn onedrive_guard_rejects_unsupported_configuration_before_graph() {
    let temp = tempfile::tempdir().unwrap();
    let connection = connection(DriveType::Personal, temp.path());
    let binding = MirrorGuardBinding::from_connection(&connection).unwrap();
    for (key, value) in [
        ("root_folder_id", json!("alternate")),
        ("client_id", json!("custom")),
        ("auth_url", json!("https://example.invalid/authorize")),
        ("expose_onenote_files", json!(true)),
        ("client_credentials", json!(true)),
    ] {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        let mut config: Value =
            serde_json::from_str(&config(DriveType::Personal, "primary")).unwrap();
        config["example_remote"][key] = value;
        runner.push(Ok(output(&config.to_string())));
        assert_eq!(
            verify_mirror_binding(
                &binding,
                &connection,
                &runner,
                &FakeGraph::new([]),
                CancellationToken::new()
            )
            .await,
            Err(OneDriveError::CustomClient)
        );
    }
}
