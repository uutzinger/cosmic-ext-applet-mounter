// SPDX-License-Identifier: MIT

//! Offline engine compatibility and routing; no user data, commands or services.

use std::path::{Path, PathBuf};

use cosmic_ext_applet_mounter::config::{Config, ValidationError};
use cosmic_ext_applet_mounter::model::*;
use cosmic_ext_applet_mounter::providers::{
    RcloneBackend, onedriver_mount_plan, rclone_mount_plan,
};
use cosmic_ext_applet_mounter::sync::rclone_bisync_plan;

const LEGACY: &str = r#"(
    id:"2a3f5d45-e867-47e7-943f-66cf60e777ad",name:"Legacy mirror",provider:OneDrive,
    mode:OfflineMirror((recovery_directory:"/home/example/Recovery/legacy",
        sync_interval_minutes:15,sync_on_metered:false)),
    remote_reference:"legacy",remote_subpath:None,local_path:"/home/example/Cloud/legacy",
    enabled:false,vpn_profile_id:None,disconnect_vpn_when_unused:false,tuning_profile:Balanced
)"#;

fn legacy() -> Connection {
    ron::from_str(LEGACY).expect("pre-engine configuration must still load")
}

fn mirror(drive_type: OneDriveDriveType) -> Connection {
    let mut connection = legacy();
    connection.id = ConnectionId::new();
    connection.name = "Rclone mirror".into();
    connection.remote_reference = "example-remote".into();
    connection.remote_subpath = Some("Test".into());
    connection.local_path = "/home/example/Cloud/rclone".into();
    let ConnectionMode::OfflineMirror(options) = &mut connection.mode else {
        unreachable!()
    };
    options.engine = Some(OfflineEngine::Rclone);
    options.recovery_directory = "/home/example/Recovery/rclone".into();
    connection.onedrive_identity = Some(OneDriveIdentity {
        drive_id: "synthetic-drive-id".into(),
        drive_type,
    });
    connection
}

fn config(connections: Vec<Connection>) -> Config {
    let mut config = Config::default();
    config.document.connections = connections;
    config
}

#[test]
fn onedrive_configuration_legacy_round_trip_keeps_engine_and_wire_format() {
    let old = legacy();
    assert_eq!(old.offline_engine(), Some(OfflineEngine::LegacyOnedrive));
    assert!(config(vec![old.clone()]).validate().is_ok());
    let serialized = ron::to_string(&old).unwrap();
    assert!(!serialized.contains("engine:"));
    assert!(!serialized.contains("onedrive_identity:"));
    assert_eq!(ron::from_str::<Connection>(&serialized).unwrap(), old);
    assert!(RcloneBackend::try_from(&old).is_err());
    assert!(old.ensure_supported_engine().is_err());
}

#[test]
fn onedrive_configuration_preserves_historic_legacy_recovery_on_unrelated_save() {
    let work = Path::new("/home/example/State");
    let mut old = legacy();
    if let ConnectionMode::OfflineMirror(options) = &mut old.mode {
        options.recovery_directory = work.join("onedrive-recovery").join(old.id.to_string());
    }
    let new = mirror(OneDriveDriveType::Personal);
    assert!(
        config(vec![old.clone(), new.clone()])
            .validate_with_work_root(work)
            .is_ok()
    );
    let mut invalid = new.clone();
    if let ConnectionMode::OfflineMirror(options) = &mut invalid.mode {
        options.recovery_directory = work.join("onedrive-recovery").join(invalid.id.to_string());
    }
    assert!(
        config(vec![old.clone(), invalid])
            .validate_with_work_root(work)
            .is_err()
    );
    for recovery in [
        work.to_path_buf(),
        work.join("rclone-bisync"),
        work.join("onedrive-recovery/other-owner"),
    ] {
        let mut invalid = old.clone();
        if let ConnectionMode::OfflineMirror(options) = &mut invalid.mode {
            options.recovery_directory = recovery;
        }
        assert!(
            config(vec![invalid, new.clone()])
                .validate_with_work_root(work)
                .is_err()
        );
    }
    let mut invalid = old;
    invalid.local_path = work.join("onedrive-recovery/mirror");
    assert!(
        config(vec![invalid, new])
            .validate_with_work_root(work)
            .is_err()
    );
}

#[test]
fn onedrive_configuration_legacy_only_shared_recovery_still_loads() {
    let first = legacy();
    let mut second = first.clone();
    second.id = ConnectionId::new();
    second.name = "Second legacy".into();
    second.local_path = "/home/example/Cloud/second-legacy".into();
    assert!(config(vec![first, second]).validate().is_ok());
}

#[test]
fn onedrive_configuration_both_drive_types_round_trip_and_select_rclone() {
    for drive_type in [OneDriveDriveType::Personal, OneDriveDriveType::Business] {
        let new = mirror(drive_type);
        assert!(config(vec![new.clone(), legacy()]).validate().is_ok());
        assert_eq!(new.offline_engine(), Some(OfflineEngine::Rclone));
        assert_eq!(
            RcloneBackend::try_from(&new).unwrap(),
            RcloneBackend::OneDrive
        );
        assert_eq!(
            ron::from_str::<Connection>(&ron::to_string(&new).unwrap()).unwrap(),
            new
        );
        assert_eq!(
            serde_json::from_str::<Connection>(&serde_json::to_string(&new).unwrap()).unwrap(),
            new
        );
        assert!(new.ensure_supported_engine().is_ok());
        let plan = rclone_bisync_plan(&new, Path::new("/home/example/State")).unwrap();
        assert!(plan.preflight_command.is_some());
        assert!(plan.work_directory.to_string_lossy().contains("target-"));
        assert!(
            plan.access_marker_name
                .as_deref()
                .unwrap()
                .starts_with(".cloud-mounter-onedrive-anchor-")
        );
    }
}

#[test]
fn onedrive_configuration_rejects_missing_malformed_or_misplaced_identity() {
    let new = mirror(OneDriveDriveType::Personal);
    let mut cases = Vec::new();
    let mut missing = new.clone();
    missing.onedrive_identity = None;
    assert!(RcloneBackend::try_from(&missing).is_err());
    cases.push(missing);
    for id in ["", " ", "bad\nid", "bad/id", "bad\\id"] {
        let mut invalid = new.clone();
        invalid.onedrive_identity.as_mut().unwrap().drive_id = id.into();
        cases.push(invalid);
    }
    let mut misplaced = new.clone();
    misplaced.provider = Provider::Box;
    cases.push(misplaced);
    let mut misplaced = new.clone();
    misplaced.mode = ConnectionMode::OnlineMount(OnlineMountConfig::default());
    cases.push(misplaced);
    let mut misplaced = new;
    if let ConnectionMode::OfflineMirror(options) = &mut misplaced.mode {
        options.engine = None;
    }
    cases.push(misplaced);
    for invalid in cases {
        assert!(
            config(vec![invalid])
                .validate()
                .unwrap_err()
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidOneDriveIdentity(_)))
        );
    }
    assert!(
        serde_json::from_str::<OneDriveIdentity>(
            r#"{"drive_id":"example","drive_type":"documentLibrary"}"#
        )
        .is_err()
    );
}

#[test]
fn onedrive_configuration_other_providers_keep_rclone_and_reject_legacy_engine() {
    for provider in [
        Provider::GoogleDrive,
        Provider::Box,
        Provider::Smb,
        Provider::Sftp,
    ] {
        let mut connection = legacy();
        connection.provider = provider;
        assert_eq!(connection.offline_engine(), Some(OfflineEngine::Rclone));
        assert!(config(vec![connection.clone()]).validate().is_ok());
        assert!(RcloneBackend::try_from(&connection).is_ok());
        if let ConnectionMode::OfflineMirror(options) = &mut connection.mode {
            options.engine = Some(OfflineEngine::LegacyOnedrive);
        }
        assert!(
            config(vec![connection.clone()])
                .validate()
                .unwrap_err()
                .iter()
                .any(|error| matches!(error, ValidationError::InvalidOfflineEngine(_)))
        );
        assert!(RcloneBackend::try_from(&connection).is_err());
    }
}

#[test]
fn onedrive_configuration_online_stays_on_onedriver() {
    let mut online = legacy();
    online.mode = ConnectionMode::OnlineMount(OnlineMountConfig::default());
    assert_eq!(online.offline_engine(), None);
    assert!(RcloneBackend::try_from(&online).is_err());
    assert!(
        rclone_mount_plan(&online, Path::new("/tmp/runtime"), Path::new("/tmp/cache")).is_err()
    );
    let plan =
        onedriver_mount_plan(&online, Path::new("/tmp/config"), Path::new("/tmp/cache")).unwrap();
    assert_eq!(plan.service.executable, PathBuf::from("/usr/bin/onedriver"));
}

#[test]
fn onedrive_configuration_blocks_legacy_recovery_and_runtime_work_overlap() {
    let old = legacy();
    let new = mirror(OneDriveDriveType::Business);
    for overlap in [
        &old.local_path,
        match &old.mode {
            ConnectionMode::OfflineMirror(options) => &options.recovery_directory,
            _ => unreachable!(),
        },
    ] {
        let mut invalid = new.clone();
        invalid.local_path = overlap.join("nested");
        assert!(
            config(vec![old.clone(), invalid])
                .validate()
                .unwrap_err()
                .iter()
                .any(|error| matches!(error, ValidationError::ConflictingTargets { .. }))
        );
        let mut invalid = new.clone();
        if let ConnectionMode::OfflineMirror(options) = &mut invalid.mode {
            options.recovery_directory = overlap.join("nested");
        }
        assert!(config(vec![old.clone(), invalid]).validate().is_err());
    }
    let config = config(vec![old.clone(), new]);
    assert!(
        config
            .validate_with_work_root(Path::new("/home/example/State"))
            .is_ok()
    );
    assert!(
        config
            .validate_with_work_root(&old.local_path.join("work"))
            .is_err()
    );
    assert!(
        config
            .validate_with_work_root(Path::new("/home/example/Recovery"))
            .is_err()
    );
    assert!(
        config
            .validate_with_work_root(Path::new("relative"))
            .is_err()
    );
}

#[test]
fn onedrive_configuration_legacy_overlap_blocks_equal_parent_child_in_either_order() {
    // Retired profiles still reserve their paths so replacement cannot overwrite data.
    let old = legacy();
    for kind in [OneDriveDriveType::Personal, OneDriveDriveType::Business] {
        for root in [
            old.local_path.clone(),
            match &old.mode {
                ConnectionMode::OfflineMirror(options) => options.recovery_directory.clone(),
                _ => unreachable!(),
            },
        ] {
            for path in [
                root.clone(),
                root.join("child"),
                root.parent().unwrap().to_path_buf(),
            ] {
                for recovery in [false, true] {
                    let mut candidate = mirror(kind);
                    if recovery {
                        let ConnectionMode::OfflineMirror(options) = &mut candidate.mode else {
                            unreachable!()
                        };
                        options.recovery_directory = path.clone();
                    } else {
                        candidate.local_path = path.clone();
                    }
                    for connections in [
                        vec![old.clone(), candidate.clone()],
                        vec![candidate.clone(), old.clone()],
                    ] {
                        assert!(
                            config(connections).validate().unwrap_err().iter().any(|e| {
                                matches!(e, ValidationError::ConflictingTargets { .. })
                            })
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn onedrive_configuration_replacement_keeps_legacy_state_out_of_rclone_plan() {
    let old = legacy();
    let before = ron::to_string(&old).unwrap();
    let legacy_directory = Path::new("/home/example/Config/onedrive-sync").join(old.id.to_string());
    for kind in [OneDriveDriveType::Personal, OneDriveDriveType::Business] {
        let new = mirror(kind);
        assert!(config(vec![old.clone(), new.clone()]).validate().is_ok());
        let plan = rclone_bisync_plan(&new, Path::new("/home/example/State")).unwrap();
        assert_ne!(new.id, old.id);
        assert!(!plan.work_directory.starts_with(&legacy_directory));
        assert!(!plan.service.arguments.iter().any(|argument| {
            argument == "--confdir"
                || argument.contains(&old.id.to_string())
                || argument.contains("onedrive-sync")
        }));
        assert_eq!(ron::to_string(&old).unwrap(), before);
    }
}
