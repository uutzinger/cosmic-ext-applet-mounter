// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use cosmic_ext_applet_mounter::import::{preview_import, replacement_plan, scan_legacy_units};

fn main() {
    let home = env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    let directory = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config/systemd/user"));
    let output_directory = env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cosmic-mounter-import-confirm-dry-run"));

    let units = match scan_legacy_units(&directory) {
        Ok(units) => units,
        Err(error) => {
            eprintln!("Failed to scan {}: {error}", directory.display());
            std::process::exit(1);
        }
    };
    if let Err(error) = fs::create_dir_all(&output_directory) {
        eprintln!(
            "Failed to create output directory {}: {error}",
            output_directory.display()
        );
        std::process::exit(1);
    }

    println!("Legacy import source directory: {}", directory.display());
    println!(
        "Dry-run managed-unit output: {}",
        output_directory.display()
    );

    let active_units = active_unit_names(units.iter().map(|unit| unit.name.as_str()));
    let mut planned = 0usize;
    for unit in units {
        let preview = match preview_import(&unit, &[], &active_units, &home) {
            Ok(preview) => preview,
            Err(error) => {
                println!("Skipping {}: {error}", unit.name);
                continue;
            }
        };
        if preview.active_conflict || preview.local_target_conflict {
            println!("Skipping {}: conflict detected", unit.name);
            continue;
        }
        let plan = match replacement_plan(
            preview,
            true,
            true,
            false,
            &env::temp_dir().join("cosmic-mounter-runtime"),
            &home.join(".cache/cosmic-mounter"),
            &home.join(".config/cosmic-mounter"),
        ) {
            Ok(plan) => plan,
            Err(error) => {
                println!("Skipping {}: {error}", unit.name);
                continue;
            }
        };
        let path = output_directory.join(plan.managed_service.name.file_name());
        if let Err(error) = fs::write(&path, &plan.managed_service.content) {
            eprintln!("Failed to write {}: {error}", path.display());
            std::process::exit(1);
        }
        planned += 1;
        println!(
            "Planned {} -> {} (preserve_original={}, disable_original={})",
            plan.preview.original_unit_name,
            path.display(),
            plan.preserve_original,
            plan.disable_original
        );
    }
    println!("Confirmed import dry-run plans: {planned}");
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
