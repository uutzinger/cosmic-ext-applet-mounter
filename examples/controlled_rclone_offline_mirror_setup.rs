// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic_ext_applet_mounter::config::{APP_ID, Config};
use cosmic_ext_applet_mounter::model::{
    Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TuningProfile,
};
use cosmic_ext_applet_mounter::process::SystemCommandRunner;
use cosmic_ext_applet_mounter::services::{
    CommandSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction, SystemdManager,
    UnitController, UnitDocument,
};
use cosmic_ext_applet_mounter::sync::rclone_bisync_plan;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct TestConnectionSpec {
    id: &'static str,
    name: &'static str,
    provider: Provider,
    remote: &'static str,
    subtree: &'static str,
    mirror: &'static str,
    recovery: &'static str,
}

const BOX_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "bb69e234-cf3e-4e63-8592-2601a93d604b",
    name: "Disposable Box Offline Mirror Test",
    provider: Provider::Box,
    remote: "ua_box",
    subtree: "Utzinger/cosmic-mounter-ui-test",
    mirror: "/tmp/cosmic-mounter-box-mirror",
    recovery: "/tmp/cosmic-mounter-box-recovery",
};

const GOOGLE_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "4e30dc23-c887-4704-bb98-41c5dfaf6467",
    name: "Disposable Google Drive Offline Mirror Test",
    provider: Provider::GoogleDrive,
    remote: "uutzinger_gdrive",
    subtree: "cosmic-mounter-ui-test",
    mirror: "/tmp/cosmic-mounter-gdrive-mirror",
    recovery: "/tmp/cosmic-mounter-gdrive-recovery",
};

const SMB_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "9e6d9640-9c99-48ef-86c1-b3e91d8dc146",
    name: "Disposable SMB Offline Mirror Test",
    provider: Provider::Smb,
    remote: "ua_engr",
    subtree: "Research/Utzinger/cosmic-mounter-ui-test",
    mirror: "/tmp/cosmic-mounter-smb-mirror",
    recovery: "/tmp/cosmic-mounter-smb-recovery",
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let spec = match std::env::args().nth(1).as_deref() {
        None | Some("box") => &BOX_SPEC,
        Some("google") | Some("gdrive") | Some("google-drive") => &GOOGLE_SPEC,
        Some("smb") => &SMB_SPEC,
        Some(other) => {
            return Err(format!(
                "unknown provider `{other}`; use one of: box, google, smb"
            ));
        }
    };
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is unavailable".to_owned())?;
    let id = std::env::var("COSMIC_MOUNTER_TEST_ID").unwrap_or_else(|_| spec.id.into());
    let name = std::env::var("COSMIC_MOUNTER_TEST_NAME").unwrap_or_else(|_| spec.name.into());
    let remote = std::env::var("COSMIC_MOUNTER_TEST_REMOTE").unwrap_or_else(|_| spec.remote.into());
    let subtree =
        std::env::var("COSMIC_MOUNTER_TEST_SUBTREE").unwrap_or_else(|_| spec.subtree.into());
    let mirror = std::env::var("COSMIC_MOUNTER_TEST_MIRROR").unwrap_or_else(|_| spec.mirror.into());
    let recovery =
        std::env::var("COSMIC_MOUNTER_TEST_RECOVERY").unwrap_or_else(|_| spec.recovery.into());
    let connection = Connection {
        id: ConnectionId::from_uuid(Uuid::parse_str(&id).map_err(|error| error.to_string())?),
        name,
        provider: spec.provider,
        mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: PathBuf::from(recovery),
            sync_interval_minutes: 15,
            sync_on_metered: false,
        }),
        remote_reference: remote,
        remote_subpath: Some(subtree),
        local_path: PathBuf::from(mirror),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
    };

    std::fs::create_dir_all(&connection.local_path).map_err(|error| {
        format!(
            "failed to create mirror directory {}: {error}",
            connection.local_path.display()
        )
    })?;
    if let ConnectionMode::OfflineMirror(options) = &connection.mode {
        std::fs::create_dir_all(&options.recovery_directory).map_err(|error| {
            format!(
                "failed to create recovery directory {}: {error}",
                options.recovery_directory.display()
            )
        })?;
    }

    let storage = cosmic::cosmic_config::Config::new(APP_ID, Config::VERSION)
        .map_err(|error| format!("failed to open applet config: {error}"))?;
    let mut config = Config::load().config;
    config
        .update_validated(&storage, |document| {
            if let Some(existing) = document
                .connections
                .iter_mut()
                .find(|existing| existing.id == connection.id)
            {
                *existing = connection.clone();
            } else {
                document.connections.push(connection.clone());
            }
        })
        .map_err(|error| format!("failed to save test connection: {error}"))?;

    let work_root = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"))
        .join("cosmic-ext-applet-mounter");
    std::fs::create_dir_all(&work_root).map_err(|error| {
        format!(
            "failed to create work root {}: {error}",
            work_root.display()
        )
    })?;
    let plan = rclone_bisync_plan(&connection, &work_root)
        .map_err(|error| format!("failed to build bisync plan: {error}"))?;
    std::fs::create_dir_all(&plan.work_directory).map_err(|error| {
        format!(
            "failed to create bisync work directory {}: {error}",
            plan.work_directory.display()
        )
    })?;
    std::fs::write(
        &plan.filters_file,
        "# Google cloud-native documents remain browser-accessible.\n- *.gdoc\n- *.gsheet\n- *.gslides\n",
    )
    .map_err(|error| {
        format!(
            "failed to write filters file {}: {error}",
            plan.filters_file.display()
        )
    })?;
    let service = UnitDocument::service(&plan.service)
        .map_err(|error| format!("failed to render service unit: {error}"))?;
    let timer = UnitDocument::timer(&plan.timer)
        .map_err(|error| format!("failed to render timer unit: {error}"))?;

    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| format!("failed to open user unit store: {error}"))?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&service, cancellation.child_token())
        .await
        .map_err(|error| format!("failed to install service unit: {error}"))?;
    controller
        .install(&timer, cancellation.child_token())
        .await
        .map_err(|error| format!("failed to install timer unit: {error}"))?;

    let manager = CommandSystemdManager::new(SystemCommandRunner);
    manager
        .action(
            SystemdAction::Disable,
            Some(&service.name),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| format!("failed to keep service disabled: {error}"))?;
    manager
        .action(SystemdAction::Disable, Some(&timer.name), cancellation)
        .await
        .map_err(|error| format!("failed to keep timer disabled: {error}"))?;

    println!("Connection: {}", connection.name);
    println!("Service: {}", service.name.file_name());
    println!("Timer: {}", timer.name.file_name());
    println!("Remote: {}", plan.path1_remote);
    println!("Mirror: {}", plan.path2_local.display());
    println!("Recovery: {}", plan.recovery_directory.display());
    println!("Work: {}", plan.work_directory.display());
    println!("Interval: {:?}", Duration::from_secs(15 * 60));
    println!("Installed managed mirror service/timer without starting them.");
    Ok(())
}
