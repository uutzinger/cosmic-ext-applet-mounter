// SPDX-License-Identifier: MIT

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use cosmic_ext_applet_mounter::config::{APP_ID, Config, ConfigDocument};
use cosmic_ext_applet_mounter::mirror_script;
use cosmic_ext_applet_mounter::model::{
    Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TuningProfile,
};
use cosmic_ext_applet_mounter::sync::rclone_bisync_plan;
use cosmic_ext_applet_mounter::teams::{MIRROR_GUARD_FLAG, MirrorGuardBinding, parse_library_url};
use tempfile::TempDir;
use uuid::Uuid;

fn save_isolated_config(config_home: &Path, connection: Connection) {
    let config = Config {
        document: ConfigDocument {
            connections: vec![connection],
            ..ConfigDocument::default()
        },
    };
    config.validate().expect("isolated Teams mirror config");
    let path = config_home
        .join("cosmic")
        .join(APP_ID)
        .join("v2")
        .join("document");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, ron::to_string(&config.document).unwrap()).unwrap();
}

fn guard(binary: &Path, config_home: &Path, binding: &MirrorGuardBinding) -> std::process::Output {
    Command::new(binary)
        .env("XDG_CONFIG_HOME", config_home)
        .arg(MIRROR_GUARD_FLAG)
        .arg(serde_json::to_string(binding).unwrap())
        .output()
        .expect("run compiled Teams mirror guard")
}

#[test]
#[ignore = "requires authenticated rclone access to the approved disposable SharePoint folder"]
fn compiled_guard_passes_approved_folder_and_blocks_changed_targets_before_sync() {
    let remote = std::env::var("TEAMS_TEST_REMOTE").expect("TEAMS_TEST_REMOTE");
    let url = std::env::var("TEAMS_TEST_LIBRARY_URL").expect("TEAMS_TEST_LIBRARY_URL");
    let drive_id = std::env::var("TEAMS_TEST_DRIVE_ID").expect("TEAMS_TEST_DRIVE_ID");
    let folder = std::env::var("TEAMS_TEST_FOLDER").expect("TEAMS_TEST_FOLDER");
    let temp = TempDir::new().unwrap();
    let config_home = temp.path().join("config");
    let binary = Path::new(env!("CARGO_BIN_EXE_cosmic-ext-applet-mounter"));
    let mut identity = parse_library_url(&url).unwrap();
    identity.drive_id = drive_id;
    let original = Connection {
        id: ConnectionId::new(),
        name: "Isolated Teams mirror guard test".into(),
        provider: Provider::Teams,
        mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: temp.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }),
        remote_reference: remote,
        remote_subpath: Some(folder),
        local_path: temp.path().join("local"),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
        smb_preload_override: None,
        sftp_preload_override: None,
        teams_identity: Some(identity),
    };
    save_isolated_config(&config_home, original.clone());
    let binding = MirrorGuardBinding::from_connection(&original).unwrap();
    let pass = guard(binary, &config_home, &binding);
    assert!(
        pass.status.success(),
        "approved folder failed: {}",
        String::from_utf8_lossy(&pass.stderr)
    );

    // A fake rclone records if the generated script ever gets past its guard.
    let marker = temp.path().join("rclone-was-called");
    let fake_rclone = temp.path().join("rclone-never-run.sh");
    fs::write(
        &fake_rclone,
        format!(
            "#!/bin/sh\nprintf called > '{}'\nexit 87\n",
            marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake_rclone, fs::Permissions::from_mode(0o700)).unwrap();
    let mut plan = rclone_bisync_plan(&original, &temp.path().join("work")).unwrap();
    plan.preflight_command.as_mut().unwrap().executable = binary.to_path_buf();
    plan.service.executable = fake_rclone;
    let script = temp.path().join("managed-bisync.sh");
    fs::write(&script, mirror_script::render(&plan).unwrap()).unwrap();

    let mut changed = original.clone();
    for case in [
        "remote", "drive", "library", "folder", "local", "recovery", "disabled",
    ] {
        changed.clone_from(&original);
        match case {
            "remote" => changed.remote_reference = "other_remote".into(),
            "drive" => changed.teams_identity.as_mut().unwrap().drive_id = "b!other".into(),
            "library" => {
                changed.teams_identity = Some(
                    parse_library_url("https://emailarizona.sharepoint.com/sites/Other/Documents")
                        .unwrap(),
                );
                changed.teams_identity.as_mut().unwrap().drive_id = "b!other".into();
            }
            "folder" => changed.remote_subpath = Some("Other-Disposable-Folder".into()),
            "local" => changed.local_path = temp.path().join("other-local"),
            "recovery" => {
                let ConnectionMode::OfflineMirror(options) = &mut changed.mode else {
                    unreachable!()
                };
                options.recovery_directory = temp.path().join("other-recovery");
            }
            "disabled" => changed.enabled = false,
            _ => unreachable!(),
        }
        save_isolated_config(&config_home, changed.clone());
        let blocked = guard(binary, &config_home, &binding);
        assert!(!blocked.status.success(), "{case} change passed guard");
        assert!(
            String::from_utf8_lossy(&blocked.stderr).contains("changed or was disabled"),
            "unexpected {case} error: {}",
            String::from_utf8_lossy(&blocked.stderr)
        );
        for mode in ["initial-preview", "initial-sync", "scheduled"] {
            let output = Command::new("/usr/bin/sh")
                .env("XDG_CONFIG_HOME", &config_home)
                .arg(&script)
                .arg(mode)
                .output()
                .unwrap();
            assert!(!output.status.success(), "{case}/{mode} passed guard");
            assert!(!marker.exists(), "{case}/{mode} reached rclone");
        }
    }
    assert!(
        !original.local_path.exists(),
        "guard created local mirror data"
    );
    let ConnectionMode::OfflineMirror(options) = &original.mode else {
        unreachable!()
    };
    assert!(
        !options.recovery_directory.exists(),
        "guard created recovery data"
    );
}

#[test]
#[ignore = "writes only inside a new subfolder of the approved disposable SharePoint folder"]
fn managed_sharepoint_runner_initial_sync_rechecks_both_sides() {
    let remote = std::env::var("TEAMS_TEST_REMOTE").expect("TEAMS_TEST_REMOTE");
    let url = std::env::var("TEAMS_TEST_LIBRARY_URL").expect("TEAMS_TEST_LIBRARY_URL");
    let drive_id = std::env::var("TEAMS_TEST_DRIVE_ID").expect("TEAMS_TEST_DRIVE_ID");
    let approved_root = std::env::var("TEAMS_TEST_FOLDER").expect("TEAMS_TEST_FOLDER");
    assert_eq!(approved_root, "CloudMounter-TeamsMirror-Test-20261002");
    let folder = format!("{approved_root}/cm-managed-runner-{}", Uuid::new_v4());
    let remote_path = format!("{remote}:{folder}");
    let temp = TempDir::new().unwrap();
    let config_home = temp.path().join("config");
    let local = temp.path().join("local");
    fs::create_dir_all(&local).unwrap();
    fs::write(
        local.join("marker.txt"),
        "managed SharePoint runner probe\n",
    )
    .unwrap();
    fs::write(
        local.join("office.docx"),
        include_bytes!("fixtures/sharepoint-office-probe.docx"),
    )
    .unwrap();
    let mkdir = Command::new("rclone")
        .arg("mkdir")
        .arg(&remote_path)
        .output()
        .expect("create isolated remote test folder");
    assert!(
        mkdir.status.success(),
        "remote test folder: {}",
        String::from_utf8_lossy(&mkdir.stderr)
    );

    let mut identity = parse_library_url(&url).unwrap();
    identity.drive_id = drive_id;
    let connection = Connection {
        id: ConnectionId::new(),
        name: "Isolated SharePoint runner test".into(),
        provider: Provider::Teams,
        mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: temp.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }),
        remote_reference: remote,
        remote_subpath: Some(folder),
        local_path: local.clone(),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
        smb_preload_override: None,
        sftp_preload_override: None,
        teams_identity: Some(identity),
    };
    save_isolated_config(&config_home, connection.clone());
    let mut plan = rclone_bisync_plan(&connection, &temp.path().join("work")).unwrap();
    fs::write(
        local.join(plan.access_marker_name.as_ref().unwrap()),
        format!("{}\n", connection.id),
    )
    .unwrap();
    plan.preflight_command.as_mut().unwrap().executable =
        Path::new(env!("CARGO_BIN_EXE_cosmic-ext-applet-mounter")).to_path_buf();
    fs::create_dir_all(&plan.work_directory).unwrap();
    fs::write(&plan.filters_file, "- /.cosmic-mounter-recovery/\n+ **\n").unwrap();
    let script = mirror_script::script_path(&plan);
    fs::write(&script, mirror_script::render(&plan).unwrap()).unwrap();
    for mode in ["initial-preview", "initial-sync"] {
        let output = Command::new("/usr/bin/sh")
            .env("XDG_CONFIG_HOME", &config_home)
            .arg(&script)
            .arg(mode)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mode == "initial-sync" {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("0 differences found"),
                "missing final checksum verification"
            );
        }
    }
    let archive = Command::new("unzip")
        .arg("-t")
        .arg(local.join("office.docx"))
        .output()
        .unwrap();
    assert!(
        archive.status.success(),
        "SharePoint Office copy is not a valid archive"
    );
    let dated_recovery = plan.recovery_directory.clone();
    let recovered_office = fs::read_dir(dated_recovery)
        .unwrap()
        .flat_map(|date| fs::read_dir(date.unwrap().path()).unwrap())
        .any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("office.docx-")
        });
    assert!(
        recovered_office,
        "original Office file missing from recovery"
    );
    fs::write(
        local.join("marker.txt"),
        "managed SharePoint runner update\n",
    )
    .unwrap();
    let updated = Command::new("/usr/bin/sh")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg(&script)
        .arg("sync")
        .output()
        .unwrap();
    assert!(
        updated.status.success(),
        "updated sync: {}",
        String::from_utf8_lossy(&updated.stderr)
    );
    assert!(String::from_utf8_lossy(&updated.stderr).contains("0 differences found"));
    let recovery = Command::new("rclone")
        .arg("lsf")
        .arg("--recursive")
        .arg(&plan.remote_recovery_path)
        .output()
        .unwrap();
    assert!(recovery.status.success());
    assert!(
        String::from_utf8_lossy(&recovery.stdout).contains("marker.txt-"),
        "previous marker missing from dated recovery"
    );
    let output = Command::new("/usr/bin/sh")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg(&script)
        .arg("preview")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No changes found"));
}
