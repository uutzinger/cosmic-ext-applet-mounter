// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::sync::Arc;

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic_ext_applet_mounter::config::{APP_ID, Config};
use cosmic_ext_applet_mounter::model::{
    Connection, ConnectionId, ConnectionMode, OnlineMountConfig, Provider, TuningProfile,
};
use cosmic_ext_applet_mounter::process::SystemCommandRunner;
use cosmic_ext_applet_mounter::providers::rclone_mount_plan;
use cosmic_ext_applet_mounter::services::{
    CommandSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction, SystemdManager,
    UnitController, UnitDocument,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct TestConnectionSpec {
    id: &'static str,
    name: &'static str,
    provider: Provider,
    remote: &'static str,
    subtree: &'static str,
    mountpoint: &'static str,
}

const BOX_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "1e31ac32-dcae-4ac7-9546-7a82437d04f4",
    name: "Disposable Box Online Mount Test",
    provider: Provider::Box,
    remote: "ua_box",
    subtree: "Utzinger/cosmic-mounter-ui-test",
    mountpoint: "Cloud/cosmic-mounter-box-test",
};

const GOOGLE_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "bbd33f7c-f9db-4a5f-b7b3-71e3b0e3d370",
    name: "Disposable Google Drive Online Mount Test",
    provider: Provider::GoogleDrive,
    remote: "uutzinger_gdrive",
    subtree: "cosmic-mounter-ui-test",
    mountpoint: "Cloud/cosmic-mounter-gdrive-test",
};

const SMB_SPEC: TestConnectionSpec = TestConnectionSpec {
    id: "3e04d0b2-83be-48ed-813a-fc7e727df0cd",
    name: "Disposable SMB Online Mount Test",
    provider: Provider::Smb,
    remote: "ua_engr",
    subtree: "Research/Utzinger/cosmic-mounter-ui-test",
    mountpoint: "Cloud/cosmic-mounter-smb-test",
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
    let mountpoint =
        std::env::var("COSMIC_MOUNTER_TEST_MOUNTPOINT").unwrap_or_else(|_| spec.mountpoint.into());
    let connection = Connection {
        id: ConnectionId::from_uuid(Uuid::parse_str(&id).map_err(|error| error.to_string())?),
        name,
        provider: spec.provider,
        mode: ConnectionMode::OnlineMount(OnlineMountConfig {
            cache_directory: None,
            cache_limit_bytes: 20 * 1024 * 1024 * 1024,
            start_at_login: false,
        }),
        remote_reference: remote,
        remote_subpath: Some(subtree),
        local_path: home.join(mountpoint),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
    };

    std::fs::create_dir_all(&connection.local_path).map_err(|error| {
        format!(
            "failed to create mountpoint {}: {error}",
            connection.local_path.display()
        )
    })?;

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

    let runtime_root = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-ext-applet-mounter");
    let cache_root = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cache"))
        .join("cosmic-ext-applet-mounter");
    std::fs::create_dir_all(&runtime_root).map_err(|error| {
        format!(
            "failed to create runtime directory {}: {error}",
            runtime_root.display()
        )
    })?;
    std::fs::create_dir_all(cache_root.join("rclone").join(connection.id.to_string()))
        .map_err(|error| format!("failed to create rclone cache directory: {error}"))?;
    let plan = rclone_mount_plan(&connection, &runtime_root, &cache_root)
        .map_err(|error| format!("failed to build mount plan: {error}"))?;
    let document = UnitDocument::service(&plan.service)
        .map_err(|error| format!("failed to render unit: {error}"))?;

    let store = FileUnitStore::user(Arc::new(StructuralUnitValidator))
        .map_err(|error| format!("failed to open user unit store: {error}"))?;
    let manager = CommandSystemdManager::new(SystemCommandRunner);
    let controller = UnitController::new(store, manager);
    let cancellation = CancellationToken::new();
    controller
        .install(&document, cancellation.child_token())
        .await
        .map_err(|error| format!("failed to install unit: {error}"))?;

    let manager = CommandSystemdManager::new(SystemCommandRunner);
    manager
        .action(
            SystemdAction::Disable,
            Some(&document.name),
            cancellation.child_token(),
        )
        .await
        .map_err(|error| format!("failed to keep unit disabled: {error}"))?;
    let status = manager
        .action(SystemdAction::Status, Some(&document.name), cancellation)
        .await
        .map_err(|error| format!("failed to read unit status: {error}"))?;

    println!("Connection: {}", connection.name);
    println!("Unit: {}", document.name.file_name());
    println!("Mountpoint: {}", connection.local_path.display());
    println!(
        "Status: {:?}",
        status.unwrap_or_else(|| cosmic_ext_applet_mounter::services::UnitStatus {
            active: cosmic_ext_applet_mounter::services::ActiveState::Unknown,
            enabled: false,
            detail: "unknown".into(),
        })
    );
    println!("Installed managed unit without starting it.");
    Ok(())
}
