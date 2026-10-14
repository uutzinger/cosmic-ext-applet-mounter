#!/usr/bin/env python3
"""Inspect local artifact bytes/identities offline; never build, install or publish."""

import argparse
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / "lib"))
from artifact_inspection import digest, regular_file, verify_debian, verify_matrix
from task_output import TaskReport, atomic_json
from flatpak_inspection import inspect_repository, inspect_bundle, validate_identity


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["debian-matrix", "debian", "native", "flatpak-repo", "flatpak-bundle"])
    parser.add_argument("path", type=Path)
    parser.add_argument("--version", help="Expected numeric stable version, without v")
    parser.add_argument("--architecture", choices=["amd64", "arm64"])
    parser.add_argument("--log-dir")
    parser.add_argument("--app-id")
    parser.add_argument("--flatpak-arch", choices=["x86_64", "aarch64"])
    parser.add_argument("--branch")
    parser.add_argument("--expected-metadata", type=Path)
    parser.add_argument("--expected-commit")
    args = parser.parse_args()
    is_flatpak = args.mode.startswith("flatpak-")
    if args.mode.startswith("debian") and (not args.version or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", args.version)):
        parser.error("Debian inspection requires --version X.Y.Z")
    if (args.mode == "debian") != bool(args.architecture):
        parser.error("--architecture is required only for single Debian packages")
    if (args.mode == "native" or is_flatpak) and args.version:
        parser.error("--version applies only to Debian inspection")
    flatpak_options = [args.app_id, args.flatpak_arch, args.branch, args.expected_metadata, args.expected_commit]
    if is_flatpak:
        if not all(flatpak_options[:4]):
            parser.error("Flatpak inspection requires --app-id, --flatpak-arch, --branch and --expected-metadata")
        try:
            validate_identity(args.app_id, args.flatpak_arch, args.branch, args.expected_commit)
        except ValueError as error:
            parser.error(str(error))
    elif any(flatpak_options):
        parser.error("Flatpak options require a Flatpak inspection mode")
    with TaskReport("artifact-inspection", args.log_dir) as report:
        report.phase("inspecting", next_action="Inspection only; provenance, payload and installed acceptance remain separate.")
        if is_flatpak:
            baseline = regular_file(args.expected_metadata)
            if baseline.stat().st_size > 1024 * 1024:
                raise ValueError("Expected metadata exceeds size budget")
            baseline_hash = digest(baseline)
            expected_text = baseline.read_text()
            options = (args.path, args.app_id, args.flatpak_arch, args.branch, expected_text, report.capture)
            if args.mode == "flatpak-bundle":
                inspection = inspect_bundle(*options, report.directory, args.expected_commit)
            else:
                inspection = inspect_repository(*options, args.expected_commit)
            if digest(baseline) != baseline_hash:
                raise ValueError("Comparison baseline changed during inspection")
            receipt = report.directory / "artifacts.json"
            atomic_json(receipt, {"schema_version": 1, "mode": args.mode, "source_sha": None,
                        "run_id": None, "input_path": str(args.path.absolute()),
                        "expected_metadata_sha256": baseline_hash, "inspection": inspection,
                        "artifacts": [{"path": str(args.path.absolute()),
                                       "sha256": inspection["bundle_sha256"]}] if args.mode == "flatpak-bundle" else []})
            report.phase("inspection-complete", artifact_receipt_path=str(receipt),
                         next_action="Metadata comparison only; no payload/signature, installed override or store acceptance claim.")
            return
        if args.mode == "debian-matrix":
            files = verify_matrix(args.path, "v" + args.version, report.capture)
        elif args.mode == "debian":
            files = [verify_debian(args.path, args.version, args.architecture, report.capture)]
        else:
            files = [regular_file(args.path)]
        receipt = report.directory / "artifacts.json"
        atomic_json(receipt, {"schema_version": 1, "mode": args.mode,
                    "source_sha": None, "run_id": None, "version": args.version,
                    "architecture": args.architecture,
                    "artifacts": [{"path": str(p), "sha256": digest(p),
                                   "size_bytes": p.stat().st_size} for p in files]})
        report.phase("inspection-complete", artifact_receipt_path=str(receipt))


if __name__ == "__main__":
    main()
