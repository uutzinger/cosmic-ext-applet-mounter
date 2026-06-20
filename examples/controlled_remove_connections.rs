// SPDX-License-Identifier: MIT

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic_ext_applet_mounter::config::{APP_ID, Config};
use cosmic_ext_applet_mounter::model::ConnectionId;
use uuid::Uuid;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let ids = std::env::args()
        .skip(1)
        .map(|value| {
            Uuid::parse_str(&value)
                .map(ConnectionId::from_uuid)
                .map_err(|error| format!("invalid connection id `{value}`: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() {
        return Err("provide at least one connection UUID to remove".into());
    }

    let storage = cosmic::cosmic_config::Config::new(APP_ID, Config::VERSION)
        .map_err(|error| format!("failed to open applet config: {error}"))?;
    let mut config = Config::load().config;
    let before = config.document.connections.len();
    config
        .update_validated(&storage, |document| {
            document
                .connections
                .retain(|connection| !ids.contains(&connection.id));
        })
        .map_err(|error| format!("failed to update applet config: {error}"))?;
    let removed = before.saturating_sub(config.document.connections.len());
    println!("Removed {removed} connection(s).");
    Ok(())
}
