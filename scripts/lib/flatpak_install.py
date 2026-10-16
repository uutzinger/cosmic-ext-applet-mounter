"""Install a verified existing user Flatpak candidate; never build, launch or stop apps."""

import argparse
import os
from pathlib import Path
import re

from artifact_inspection import digest, regular_file
from flatpak_tasks import load_receipt, run
from flatpak_inspection import expected_policy
from task_output import TaskReport, atomic_json

APP = "io.github.uutzinger.cosmic-ext-applet-mounter"


def native_running():
    for process in Path("/proc").iterdir():
        if process.name.isdigit():
            try:
                if (process / "exe").readlink().name.removesuffix(" (deleted)") == "cosmic-ext-applet-mounter":
                    return True
            except (FileNotFoundError, PermissionError, ProcessLookupError):
                pass
    return False


def activate_panel_entry(source, destination, directory, result, receipt):
    exported_path = source
    source = regular_file(source.resolve(strict=True))
    text = source.read_text()
    if f"X-Flatpak={APP}" not in text.splitlines() or "Exec=" not in text:
        raise ValueError("Installed exported launcher is not a Flatpak entry")
    if any(path.is_symlink() for path in [destination, *destination.parents]):
        raise ValueError("Refusing a symbolic panel launcher path")
    previous = None
    if destination.exists():
        previous = directory / "previous-panel.desktop"
        with previous.open("xb") as stream:
            stream.write(regular_file(destination).read_bytes())
        previous.chmod(0o600)
    result["panel_entry"] = {"path": str(destination), "previous_copy": str(previous) if previous else None,
                             "export_path": str(exported_path), "export_sha256": digest(source), "status": "pending"}
    atomic_json(receipt, result)
    # COSMIC classifies Flatpak by desktop-file location. A copied launcher in
    # ordinary applications loses socket forwarding even with X-Flatpak set.
    if previous:
        if digest(destination) != digest(previous):
            raise ValueError("User launcher changed before parking")
        destination.unlink()
    result["panel_entry"]["status"] = "verified"
    atomic_json(receipt, result)


def install(args, report):
    receipt_path, candidate = load_receipt(args.candidate_receipt)
    if candidate.get("application_id") != APP:
        raise ValueError("Only the Cloud Mounter application may be installed")
    # Reuse the maintained complete provenance/hash/repository/bundle checks.
    run(argparse.Namespace(
        manifest=Path(candidate["manifest_path"]), source_sha=candidate["source_sha"],
        arch=candidate["architecture"], branch=candidate["branch"],
        output_dir=candidate["output_dir"], reuse_receipt=receipt_path,
        offline=candidate["offline"], local_source_repo=None, timeout_seconds=300), report)
    output = Path(candidate["output_dir"])
    bundle = output / f"{APP}-{candidate['architecture']}-{candidate['branch']}.flatpak"
    expected = candidate["export_inspection"]
    running = report.capture("flatpak", "ps", "--columns=application")
    if APP in running.splitlines():
        raise ValueError("Close the running Cloud Mounter Flatpak before installation")
    if native_running():
        raise ValueError("Close/remove the native Cloud Mounter applet before installation")
    old_ref = report.capture("flatpak", "info", "--user", "--show-ref", APP)
    old_commit = report.capture("flatpak", "info", "--user", "--show-commit", APP)
    if old_ref != expected["ref"] or not re.fullmatch(r"[0-9a-f]{64}", old_commit):
        raise ValueError("Existing installation must match the candidate ref; rollback identity required")
    result = {"schema_version": 1, "kind": "flatpak-installation", "candidate_receipt": str(receipt_path),
              "ref": old_ref, "previous_commit": old_commit, "expected_commit": expected["commit"],
              "installation": "pending", "acceptance": "not-run"}
    installed_receipt = report.directory / "installation.json"
    atomic_json(installed_receipt, result)
    report.phase("configuring-panel-entry" if args.panel_entry_only else "installing", artifact_receipt_path=str(installed_receipt),
                 next_action="On failure inspect retained installation state; no automatic rollback or retry.")
    if args.panel_entry_only:
        if old_commit != expected["commit"]:
            raise ValueError("Panel-entry-only requires this exact candidate already installed")
    else:
        report.run(["flatpak", "install", "--user", "--noninteractive", "--no-deps", "--no-related",
                    "--bundle", str(bundle)], timeout=300)
    actual_ref = report.capture("flatpak", "info", "--user", "--show-ref", APP)
    actual_commit = report.capture("flatpak", "info", "--user", "--show-commit", APP)
    metadata = report.capture("flatpak", "info", "--user", "--show-metadata", APP)
    if (actual_ref != expected["ref"] or actual_commit != expected["commit"]
            or expected_policy(metadata, APP, candidate["architecture"]) != expected["metadata"]):
        raise ValueError("Installed identity/complete metadata differs from inspected candidate")
    result.update(installation="verified", installed_commit=actual_commit,
                  metadata=expected["metadata"], overrides={
                      "global": report.capture("flatpak", "override", "--user", "--show"),
                      "application": report.capture("flatpak", "override", "--user", "--show", APP)})
    atomic_json(installed_receipt, result)
    if args.activate_panel_entry:
        destination = Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
        activate_panel_entry(Path.home() / ".local/share/flatpak/exports/share/applications" / f"{APP}.desktop",
                             destination / "applications" / f"{APP}.desktop",
                             report.directory, result, installed_receipt)
    report.phase("installed", next_action="Installed identity verified; panel/UI and disposable mirror acceptance remain outstanding.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-receipt", type=Path, required=True)
    parser.add_argument("--log-dir")
    parser.add_argument("--activate-panel-entry", action="store_true",
                        help="Back up and park the user launcher so COSMIC discovers the real Flatpak export")
    parser.add_argument("--panel-entry-only", action="store_true",
                        help="Require candidate already installed; only park the overriding launcher, never reinstall")
    args = parser.parse_args()
    if args.panel_entry_only and not args.activate_panel_entry:
        parser.error("--panel-entry-only requires --activate-panel-entry")
    with TaskReport("flatpak-install", args.log_dir) as report:
        install(args, report)


if __name__ == "__main__":
    main()
