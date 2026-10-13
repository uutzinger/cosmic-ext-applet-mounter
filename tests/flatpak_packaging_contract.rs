// SPDX-License-Identifier: MIT

//! Source packaging contracts, not rebuilt Flatpak acceptance.

use std::collections::BTreeSet;

use cosmic_ext_applet_mounter::config::APP_ID;
use serde_json::Value;

fn manifests() -> [Value; 2] {
    [
        serde_json::from_str(include_str!(
            "../packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json"
        ))
        .unwrap(),
    ]
}

#[test]
fn flatpak_manifest_grants_match_reviewed_source_contract() {
    let reviewed = [
        "--socket=wayland",
        "--socket=fallback-x11",
        "--share=ipc",
        "--share=network",
        "--device=dri",
        "--talk-name=com.system76.CosmicSettingsDaemon",
        "--talk-name=org.freedesktop.Flatpak",
        "--talk-name=org.freedesktop.systemd1",
        "--own-name=io.github.uutzinger.cosmic-ext-applet-mounter.Runtime",
        "--talk-name=io.github.uutzinger.cosmic-ext-applet-mounter.Runtime",
        "--own-name=io.github.uutzinger.cosmic-ext-applet-mounter.Settings",
        "--talk-name=io.github.uutzinger.cosmic-ext-applet-mounter.Settings",
        "--system-talk-name=org.freedesktop.login1",
        "--system-talk-name=org.freedesktop.NetworkManager",
        "--filesystem=xdg-run/systemd/user:create",
        "--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Mode:ro",
        "--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Light:ro",
        "--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Dark:ro",
        "--filesystem=xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter:create",
        "--filesystem=xdg-config/cosmic-ext-applet-mounter:create",
        "--filesystem=xdg-config/systemd/user:create",
        "--filesystem=xdg-cache/cosmic-ext-applet-mounter:create",
        "--filesystem=~/.local/state/cosmic-ext-applet-mounter:create",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    for manifest in manifests() {
        let flags = manifest["finish-args"].as_array().unwrap();
        let actual = flags
            .iter()
            .map(|flag| flag.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(flags.len(), actual.len(), "no duplicate grants");
        assert_eq!(actual, reviewed, "permission changes need explicit review");
    }
}

#[test]
fn flatpak_build_identity_and_assets_preserve_release_boundaries() {
    let manifests = manifests();
    for manifest in &manifests {
        let id = manifest
            .get("id")
            .or_else(|| manifest.get("app-id"))
            .unwrap();
        assert_eq!(id, APP_ID);
        assert_eq!(manifest["runtime"], "org.freedesktop.Platform");
        assert_eq!(manifest["runtime-version"], "25.08");
        assert_eq!(manifest["base"], "com.system76.Cosmic.BaseApp");
        assert_eq!(manifest["base-version"], "stable");
        let commands = manifest["modules"][0]["build-commands"].as_array().unwrap();
        let commands = commands
            .iter()
            .map(|item| item.as_str().unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(commands.contains("cosmic-ext-applet-mounter-onedrive-auth-helper"));
        assert!(commands.contains("cosmic_ext_applet_mounter.ftl"));
        assert!(!commands.contains("development-probes"));
        assert!(!commands.contains("--all-features"));
    }
    let source = &manifests[0]["modules"][0]["sources"][0];
    assert_eq!(source["type"], "git");
    assert!(source.get("tag").or_else(|| source.get("commit")).is_some());
    assert_eq!(manifests[1]["modules"][0]["sources"][1]["type"], "file");
}
