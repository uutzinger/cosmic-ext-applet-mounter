// SPDX-License-Identifier: MIT

//! Source packaging contracts, not rebuilt Flatpak acceptance.

use std::collections::{BTreeMap, BTreeSet};

use cosmic_ext_applet_mounter::config::APP_ID;
use serde_json::Value;

#[test]
fn webkit_sandbox_modules_are_pinned_source_builds_with_license_payloads() {
    for (text, name, checksum) in [
        (
            include_str!("../packaging/flatpak/modules/bubblewrap.json"),
            "bubblewrap",
            "4734237473c0e5d695e4e9034a34e43b2dbf5164655bd13fa59ae376b2b7a765",
        ),
        (
            include_str!("../packaging/flatpak/modules/xdg-dbus-proxy.json"),
            "xdg-dbus-proxy",
            "5450dda586ec3bb3ca709d311e845487883faa3b09cf562608d7e84f4311dced",
        ),
    ] {
        let module: Value = serde_json::from_str(text).unwrap();
        assert_eq!(module["name"], name);
        assert_eq!(module["buildsystem"], "meson");
        assert_eq!(module["sources"][0]["type"], "archive");
        assert_eq!(module["sources"][0]["sha256"], checksum);
        assert!(
            module["sources"][0]["url"]
                .as_str()
                .unwrap()
                .starts_with("https://github.com/")
        );
        assert!(
            module["post-install"]
                .as_array()
                .unwrap()
                .iter()
                .any(|command| command.as_str().unwrap().contains("/app/share/licenses/"))
        );
        assert!(module.get("finish-args").is_none());
        assert!(!text.contains("chmod"));
        assert!(!text.contains("--filesystem=host"));
    }
}

#[test]
fn generated_git_replacements_match_locked_source_selectors() {
    let sources: Vec<Value> =
        serde_json::from_str(include_str!("../packaging/flatpak/cargo-sources.json")).unwrap();
    let config = sources
        .iter()
        .find(|source| source["dest"] == "cargo" && source["dest-filename"] == "config")
        .unwrap()["contents"]
        .as_str()
        .unwrap();
    let normalized = |value: &str| value.trim_end_matches(".git").to_owned();
    for line in include_str!("../Cargo.lock").lines() {
        let Some(source) = line.strip_prefix("source = \"git+") else {
            continue;
        };
        let source = source.trim_end_matches('"');
        let (location, commit) = source.rsplit_once('#').unwrap();
        let mut url = url::Url::parse(location).unwrap();
        let expected: BTreeMap<String, String> = url.query_pairs().into_owned().collect();
        url.set_query(None);
        let repository = normalized(url.as_str());
        let replacement = config
            .split("[source.")
            .map(|block| {
                block
                    .lines()
                    .filter_map(|line| line.split_once(" = "))
                    .map(|(key, value)| (key.to_owned(), value.trim_matches('"').to_owned()))
                    .collect::<BTreeMap<_, _>>()
            })
            .find(|block| {
                block
                    .get("git")
                    .is_some_and(|git| normalized(git) == repository)
            })
            .unwrap_or_else(|| panic!("Missing replacement for {repository}"));
        let actual: BTreeMap<_, _> = replacement
            .into_iter()
            .filter(|(key, _)| matches!(key.as_str(), "rev" | "tag" | "branch"))
            .collect();
        assert_eq!(actual, expected, "Source selector mismatch: {repository}");
        assert!(
            sources.iter().any(|entry| {
                entry["type"] == "git"
                    && entry["url"]
                        .as_str()
                        .is_some_and(|git| normalized(git) == repository)
                    && entry["commit"] == commit
            }),
            "Missing pinned checkout: {repository}"
        );
    }
}

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
    let commit = source["commit"].as_str().expect("immutable source commit");
    assert_eq!(commit.len(), 40);
    assert!(commit.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert!(source.get("tag").is_none());
    assert!(source.get("branch").is_none());
    assert_eq!(manifests[1]["modules"][0]["sources"][1]["type"], "file");
}
