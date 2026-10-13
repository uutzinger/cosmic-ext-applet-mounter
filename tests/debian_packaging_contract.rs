// SPDX-License-Identifier: MIT

//! Debian source contracts; not installed package acceptance.

#[test]
fn publication_requires_explicit_manual_acceptance() {
    let workflow = include_str!("../.github/workflows/build-deb.yml");
    assert!(workflow.contains("default: false"));
    assert!(workflow.contains("if: github.event_name == 'workflow_dispatch' && inputs.publish_after_acceptance && startsWith(github.ref, 'refs/tags/v')"));
    assert!(workflow.contains("needs: build"));
    assert!(workflow.contains("bash scripts/check-release-publication.sh"));
    assert!(workflow.contains("--verify-tag --notes-file"));
    assert!(!workflow.contains("--generate-notes"));
    let orchestrator = include_str!("../scripts/build_release.sh");
    assert!(!orchestrator.contains("gh release create"));
    assert!(orchestrator.contains("No release was published."));
}

#[test]
fn debian_recommends_optional_onedrive_gui_runtime() {
    let control = include_str!("../debian/control");
    let binary = control
        .split("\n\n")
        .find(|block| block.starts_with("Package:"))
        .unwrap();
    let recommends = binary
        .split("Recommends:\n")
        .nth(1)
        .unwrap()
        .split("\nDescription:")
        .next()
        .unwrap();
    for package in [
        "python3",
        "python3-gi",
        "gir1.2-gtk-3.0",
        "gir1.2-webkit2-4.1",
    ] {
        assert!(
            recommends
                .lines()
                .any(|line| line.trim().trim_end_matches(',') == package),
            "{package}"
        );
    }
    let depends = binary
        .split("Depends:\n")
        .nth(1)
        .unwrap()
        .split("\nRecommends:")
        .next()
        .unwrap();
    assert!(!depends.contains("gir1.2-webkit"), "helper stays optional");
}

#[test]
fn debian_install_respects_cargo_target_and_packages_helper_catalog() {
    let rules = include_str!("../debian/rules");
    assert!(rules.contains("\"$(CARGO_TARGET_DIR)/release/cosmic-ext-applet-mounter\""));
    assert!(rules.contains("scripts/cosmic-ext-applet-mounter-onedrive-auth-helper"));
    assert!(rules.contains("i18n/en/cosmic_ext_applet_mounter.ftl"));
    assert!(!rules.contains("--all-features"));
    assert!(!rules.contains("development-probes"));
}
