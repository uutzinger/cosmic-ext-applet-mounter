// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::sync::Arc;

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic_ext_applet_mounter::config::{APP_ID, Config};
use cosmic_ext_applet_mounter::model::{
    Connection, ConnectionId, ConnectionMode, OnlineMountConfig, Provider, TuningProfile,
};
use cosmic_ext_applet_mounter::process::SystemCommandRunner;
use cosmic_ext_applet_mounter::providers::onedriver_mount_plan;
use cosmic_ext_applet_mounter::services::{
    ActiveState, CommandSystemdManager, FileUnitStore, StructuralUnitValidator, SystemdAction,
    SystemdManager, UnitController, UnitDocument, UnitStatus,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const CONNECTION_ID: &str = "4f4f7e18-9d74-4f72-9e4c-0ed1a6f6c101";
const CONNECTION_NAME: &str = "Disposable OneDrive Online Mount Test";

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is unavailable".to_owned())?;
    let connection = Connection {
        id: ConnectionId::from_uuid(
            Uuid::parse_str(CONNECTION_ID).map_err(|error| error.to_string())?,
        ),
        name: CONNECTION_NAME.into(),
        provider: Provider::OneDrive,
        mode: ConnectionMode::OnlineMount(OnlineMountConfig {
            cache_directory: None,
            cache_limit_bytes: 20 * 1024 * 1024 * 1024,
            start_at_login: false,
        }),
        remote_reference: "onedriver-corporate-test".into(),
        remote_subpath: None,
        local_path: home.join("Cloud/cosmic-mounter-onedriver-test"),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
    };

    let config_root = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("cosmic-ext-applet-mounter");
    let cache_root = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cache"))
        .join("cosmic-ext-applet-mounter");
    let plan = onedriver_mount_plan(&connection, &cache_root, &config_root)
        .map_err(|error| format!("failed to build onedriver plan: {error}"))?;
    std::fs::create_dir_all(&plan.mountpoint).map_err(|error| {
        format!(
            "failed to create mountpoint {}: {error}",
            plan.mountpoint.display()
        )
    })?;
    std::fs::create_dir_all(&plan.cache_directory).map_err(|error| {
        format!(
            "failed to create cache directory {}: {error}",
            plan.cache_directory.display()
        )
    })?;
    if let Some(config_dir) = plan.config_file.parent() {
        std::fs::create_dir_all(config_dir).map_err(|error| {
            format!(
                "failed to create config directory {}: {error}",
                config_dir.display()
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
        .map_err(|error| format!("failed to read unit status: {error}"))?
        .unwrap_or_else(|| UnitStatus {
            active: ActiveState::Unknown,
            enabled: false,
            detail: "unknown".into(),
        });

    println!("Connection: {}", connection.name);
    println!("Unit: {}", document.name.file_name());
    println!("Mountpoint: {}", connection.local_path.display());
    println!("Config: {}", plan.config_file.display());
    println!("Cache: {}", plan.cache_directory.display());
    println!("Status: {status:?}");
    println!("Installed managed onedriver unit without starting it.");
    Ok(())
}
