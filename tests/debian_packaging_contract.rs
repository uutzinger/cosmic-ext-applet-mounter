// SPDX-License-Identifier: MIT

//! Debian source contracts; not installed package acceptance.

#[test]
fn package_build_is_manual_and_cannot_publish() {
    let workflow = include_str!("../.github/workflows/build-deb.yml");
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("request_id:"));
    assert!(workflow.contains("run-name: Debian packages"));
    assert!(workflow.contains("contents: read"));
    assert!(workflow.contains("dpkg-buildpackage -us -uc -b"));
    assert!(!workflow.contains("  push:"));
    assert!(!workflow.contains("publish_after_acceptance"));
    assert!(!workflow.contains("gh release create"));
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("scripts/promote_release.py"));
}

#[test]
fn publication_requires_explicit_manual_acceptance_without_rebuilding() {
    let workflow = include_str!("../.github/workflows/publish-release.yml");
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("build_run_id:"));
    assert!(workflow.contains("accepted:"));
    assert!(workflow.contains("default: false"));
    assert!(workflow.contains("actions: read"));
    assert!(workflow.contains("contents: write"));
    assert!(workflow.contains("$SOURCE_REF\" == refs/tags/v* && \"$ACCEPTED\" == true"));
    assert!(workflow.contains("BUILD_RUN_ID: ${{ inputs.build_run_id }}"));
    assert!(workflow.contains("--publish --accepted"));
    let guard = workflow
        .find("Require approved stable-tag promotion")
        .unwrap();
    let promotion = workflow.find("python3 scripts/promote_release.py").unwrap();
    assert!(guard < promotion, "approval guard must precede promotion");
    for forbidden in [
        "needs: build",
        "dpkg-buildpackage",
        "cargo build",
        "  push:",
    ] {
        assert!(!workflow.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn promotion_verifies_existing_artifacts_and_finalized_notes() {
    let promotion = include_str!("../scripts/promote_release.py");
    assert!(promotion.contains("if args.publish and (not args.accepted"));
    assert!(promotion.contains("validate_run(run,"));
    assert!(promotion.contains("verify_remote_tag(repository, args.tag, sha, capture)"));
    assert!(promotion.contains("verify_packages(root, args.tag, capture)"));
    assert!(promotion.contains("scripts/check-release-publication.sh"));
    assert!(promotion.contains("\"gh\", \"run\", \"download\""));
    assert!(promotion.contains("\"--verify-tag\""));
    assert!(promotion.contains("\"--notes-file\""));
    assert!(!promotion.contains("--generate-notes"));
    assert!(!promotion.contains("dpkg-buildpackage"));
    assert!(!promotion.contains("\"gh\", \"workflow\", \"run\""));
}

#[test]
fn release_preparation_supports_reuse_and_does_not_publish() {
    let orchestrator = include_str!("../scripts/build_release.sh");
    assert!(!orchestrator.contains("gh release create"));
    assert!(orchestrator.contains("lib/release_tasks.py\" release-prepare"));
    assert!(orchestrator.contains("--run-id --resume"));
    assert!(!orchestrator.contains("--wait-for-existing"));
    assert!(orchestrator.contains("No release was published."));
    let tasks = include_str!("../scripts/lib/release_tasks.py");
    assert!(tasks.contains("def preparation(report, args):"));
    assert!(tasks.contains("validate_selected(run, report)"));
    assert!(tasks.contains("args.resume and not args.run_id"));
    assert!(!tasks.contains("\"gh\", \"release\", \"create\""));
}

#[test]
fn debian_keeps_supported_engines_without_retired_onedrive_runtime() {
    let control = include_str!("../debian/control");
    let binary = control
        .split("\n\n")
        .find(|block| block.starts_with("Package:"))
        .unwrap();
    for retired in [
        "python3-gi",
        "gir1.2-gtk-3.0",
        "gir1.2-webkit2-4.1",
        "onedrive (>=",
    ] {
        assert!(!binary.contains(retired), "{retired}");
    }
    for retained in ["rclone (>= 1.74.3)", "onedriver (>= 0.15.0)"] {
        assert!(binary.contains(retained), "{retained}");
    }
    assert!(
        control.contains(" python3,"),
        "build tooling retains Python"
    );
}

#[test]
fn debian_install_respects_cargo_target_and_packages_catalog_without_helper() {
    let rules = include_str!("../debian/rules");
    assert!(rules.contains("\"$(CARGO_TARGET_DIR)/release/cosmic-ext-applet-mounter\""));
    assert!(!rules.contains("scripts/cosmic-ext-applet-mounter-onedrive-auth-helper"));
    assert!(rules.contains("i18n/en/cosmic_ext_applet_mounter.ftl"));
    assert!(!rules.contains("--all-features"));
    assert!(!rules.contains("development-probes"));
}
