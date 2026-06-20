// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use cosmic_ext_applet_mounter::import::{preview_import, scan_legacy_units};

fn main() {
    let home = env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    let directory = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config/systemd/user"));

    let units = match scan_legacy_units(&directory) {
        Ok(units) => units,
        Err(error) => {
            eprintln!("Failed to scan {}: {error}", directory.display());
            std::process::exit(1);
        }
    };

    println!("Legacy import preview directory: {}", directory.display());
    println!("Compatible units: {}", units.len());

    let active_units = active_unit_names(units.iter().map(|unit| unit.name.as_str()));
    for unit in units {
        match preview_import(&unit, &[], &active_units, &home) {
            Ok(preview) => {
                println!();
                println!("Unit: {}", preview.original_unit_name);
                println!("  Engine: {:?}", preview.engine);
                println!("  Provider: {:?}", preview.provider);
                println!("  Remote: {}", preview.remote_reference);
                println!(
                    "  Subpath: {}",
                    preview
                        .remote_subpath
                        .as_deref()
                        .unwrap_or("(whole remote)")
                );
                println!("  Target: {}", preview.local_target.display());
                println!(
                    "  Cache: {}",
                    preview
                        .cache_directory
                        .as_ref()
                        .map_or_else(|| "(default)".into(), |path| path.display().to_string())
                );
                println!("  Start at login: {}", preview.start_at_login);
                println!("  Active conflict: {}", preview.active_conflict);
                println!("  Target conflict: {}", preview.local_target_conflict);
                println!(
                    "  Unsupported options: {}",
                    if preview.unsupported_options.is_empty() {
                        "(none)".into()
                    } else {
                        preview.unsupported_options.join(", ")
                    }
                );
            }
            Err(error) => {
                println!();
                println!("Unit: {}", unit.name);
                println!("  Preview failed: {error}");
            }
        }
    }
}

fn active_unit_names<'a>(unit_names: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
    unit_names
        .filter(|name| {
            Command::new("systemctl")
                .args(["--user", "is-active", name])
                .output()
                .is_ok_and(|output| {
                    output.status.success()
                        && String::from_utf8_lossy(&output.stdout).trim() == "active"
                })
        })
        .map(str::to_owned)
        .collect()
}
