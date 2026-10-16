"""Inspect Flatpak prerequisites or build/export a fixed-input candidate; never install."""

import argparse
import json
from pathlib import Path
import re

from artifact_inspection import digest, regular_file
from task_output import TaskReport, atomic_json
from flatpak_inspection import expected_policy, inspect_repository, inspect_bundle, repository_directory


def required_refs(data, arch):
    version = data["runtime-version"]
    refs = [f"runtime/{data[key]}/{arch}/{version}" for key in ["runtime", "sdk"]]
    if data.get("base"):
        refs.append(f"app/{data['base']}/{arch}/{data.get('base-version', version)}")
    for extension in data.get("sdk-extensions", []):
        if not isinstance(extension, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+", extension):
            raise ValueError("Invalid SDK extension identity")
        refs.append(f"runtime/{extension}/{arch}/{version}")
    return refs


def within(path, parent):
    return path == parent or parent in path.parents


def load_receipt(path):
    path = regular_file(path).resolve()
    if path.stat().st_size > 8 * 1024 * 1024:
        raise ValueError("Receipt exceeds size budget")
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError("Receipt must be a JSON object")
    return path, data


def unchanged_inputs(manifest, checkout, sha, inputs, pins, report):
    _, final_inputs, final_pins = inspect_inputs(manifest, checkout, report)
    if inputs != final_inputs or pins != final_pins or report.capture("git", "-C", str(checkout), "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("Source inputs changed during verification/build")
    if report.capture("git", "-C", str(checkout), "rev-parse", "HEAD^{commit}") != sha:
        raise ValueError("Checkout revision changed during verification/build")


def reuse_candidate(args, report, output, manifest, checkout, sha, data, inputs, pins, required):
    report.phase("verifying-reuse", next_action="Reuse verification failed: inspect the private log and candidate; no rebuild, overwrite or cleanup will occur.")
    prior_path, prior = load_receipt(args.reuse_receipt)
    prior_hash = digest(prior_path)
    summary_path, summary = load_receipt(prior_path.parent / "summary.json")
    summary_hash = digest(summary_path)
    if (prior.get("schema_version") != 2 or prior.get("kind") != "unaccepted-flatpak-candidate"
            or summary.get("schema_version") != 1 or summary.get("ref") != args.branch
            or summary.get("phase") not in ("candidate-exported", "candidate-reused")
            or summary.get("task") != "flatpak-build" or summary.get("status") != "succeeded"
            or summary.get("exit_code") != 0 or summary.get("source_sha") != sha
            or summary.get("artifact_receipt_path") != str(prior_path)):
        raise ValueError("Reuse requires a schema-2 receipt and its successful producer summary")
    expected = {"source_sha": sha, "manifest_relative_path": str(manifest.relative_to(checkout)),
                "input_sha256": inputs, "source_pins": pins,
                "application_id": data.get("id", data.get("app-id")), "branch": args.branch,
                "architecture": args.arch, "output_dir": str(output),
                "repository_path": str(output / "repo"), "offline": args.offline,
                "sdk_extensions_declared": data.get("sdk-extensions", []),
                "finish_args_declared": data.get("finish-args", []),
                "build_options": {"disable_updates": True, "offline": args.offline}}
    if any(prior.get(key) != value for key, value in expected.items()):
        raise ValueError("Candidate receipt source/input/options/location mismatch")
    if not isinstance(prior.get("tools"), dict) or set(prior["tools"]) != {"flatpak", "flatpak-builder", "ostree"} or not all(isinstance(v, str) and v for v in prior["tools"].values()):
        raise ValueError("Original build tool provenance is incomplete")
    runtimes = prior.get("runtimes")
    if (not isinstance(runtimes, list) or len(runtimes) != len(required)
            or any(not isinstance(item, dict) or item.get("ref") != ref
                   or not re.fullmatch(r"[0-9a-f]{64}", str(item.get("commit", "")))
                   for item, ref in zip(runtimes, required))):
        raise ValueError("Original runtime provenance is incomplete or mismatched")
    appid = expected["application_id"]
    bundle = output / f"{appid}-{args.arch}-{args.branch}.flatpak"
    metadata = regular_file(output / "built-metadata.ini")
    if metadata.stat().st_size > 1024 * 1024:
        raise ValueError("Metadata exceeds size budget")
    artifacts = [{"path": str(p), "sha256": digest(p)} for p in [bundle, metadata]]
    if artifacts != prior.get("artifacts"):
        raise ValueError("Candidate bundle/metadata hashes or file locations differ")
    captured = lambda *argv: report.capture(*argv, timeout=args.timeout_seconds)
    inspection_tools = {tool: captured(tool, "--version") for tool in ["flatpak", "ostree"]}
    repo = repository_directory(output / "repo")
    # No repair/delete flags: corrupt objects fail reuse instead of being repaired.
    captured("ostree", f"--repo={repo}", "fsck", "--quiet")
    old_export = prior.get("export_inspection", {})
    if not isinstance(old_export, dict) or not re.fullmatch(r"[0-9a-f]{64}", str(old_export.get("commit", ""))):
        raise ValueError("Original exported commit is missing")
    raw_metadata = metadata.read_text()
    exported = inspect_repository(output / "repo", appid, args.arch, args.branch, raw_metadata,
                                  captured, old_export["commit"])
    bundled = inspect_bundle(bundle, appid, args.arch, args.branch, raw_metadata,
                             captured, report.directory, exported["commit"])
    if exported != old_export or bundled != prior.get("bundle_inspection"):
        raise ValueError("Current repository/bundle inspection differs from recorded result")
    unchanged_inputs(manifest, checkout, sha, inputs, pins, report)
    if (digest(prior_path) != prior_hash or digest(summary_path) != summary_hash
            or artifacts != [{"path": str(p), "sha256": digest(p)} for p in [bundle, metadata]]):
        raise ValueError("Reuse baseline or artifacts changed during verification")
    if captured("ostree", f"--repo={repo}", "rev-parse", exported["ref"]) != exported["commit"]:
        raise ValueError("Exported ref changed before reuse completed")
    receipt = report.directory / "artifacts.json"
    atomic_json(receipt, {**prior, "reuse_inspection_tools": inspection_tools,
                "reused_from": {"path": str(prior_path), "sha256": prior_hash},
                "verification_manifest_path": str(manifest),
                "repository_integrity": "ostree-fsck-passed"})
    report.phase("candidate-reused", artifact_receipt_path=str(receipt),
                 next_action="Verified existing candidate reused without building; installation/publication and final acceptance remain separate.")


def inspect_inputs(manifest, checkout, report):
    inputs, pins, visiting = {}, [], set()

    def local(path):
        path = regular_file(path).resolve()
        if not within(path, checkout):
            raise ValueError("Manifest inputs must stay inside the source checkout")
        report.capture("git", "-C", str(checkout), "ls-files", "--error-unmatch", "--", str(path.relative_to(checkout)))
        inputs[str(path.relative_to(checkout))] = digest(path)
        return path

    def load(path, kind):
        path = local(path)
        if path in visiting:
            raise ValueError("Cyclic manifest includes")
        visiting.add(path)
        if path.stat().st_size > 16 * 1024 * 1024:
            raise ValueError("Manifest input exceeds the JSON size budget")
        value = json.loads(path.read_text())
        if kind == "manifest":
            walk_module(value, path.parent)
        else:
            items = value if isinstance(value, list) else [value]
            for item in items:
                (walk_module if kind == "module" else source)(item, path.parent)
        visiting.remove(path)
        return value

    def source(value, base):
        if isinstance(value, str):
            load(base / value, "source")
            return
        if not isinstance(value, dict):
            raise ValueError("Malformed source entry")
        kind = value.get("type")
        if kind == "git":
            if (not re.fullmatch(r"[0-9a-f]{40}", value.get("commit", ""))
                    or not isinstance(value.get("url"), str)):
                raise ValueError("Git inputs require full immutable commit pins and URLs")
            pins.append({"type": kind, "commit": value["commit"]})
        elif kind in ("archive", "file", "extra-data") and "url" in value:
            if not re.fullmatch(r"[0-9a-f]{64}", value.get("sha256", "")):
                raise ValueError("Remote inputs require SHA-256 pins")
            pins.append({"type": kind, "sha256": value["sha256"]})
        elif kind in ("file", "patch"):
            paths = value.get("paths", [value["path"]] if "path" in value else [])
            if not isinstance(paths, list) or not paths or not all(isinstance(p, str) for p in paths):
                raise ValueError("Local file/patch paths required")
            for path in paths:
                local(base / path)
        elif kind == "script" and isinstance(value.get("commands"), list):
            if not all(isinstance(command, str) for command in value["commands"]):
                raise ValueError("Malformed script source")
        else:
            raise ValueError("Unsupported or mutable source type; directory/prototype inputs are not accepted")

    def walk_module(value, base):
        if isinstance(value, str):
            load(base / value, "module")
            return
        if not isinstance(value, dict):
            raise ValueError("Malformed module")
        for item in value.get("sources", []):
            source(item, base)
        for item in value.get("modules", []):
            walk_module(item, base)

    data = load(manifest, "manifest")
    return data, inputs, pins


def run(args, report):
    manifest = regular_file(args.manifest).resolve()
    checkout = Path(report.capture("git", "-C", str(manifest.parent), "rev-parse", "--show-toplevel")).resolve()
    sha = report.capture("git", "-C", str(checkout), "rev-parse", "--verify", "HEAD^{commit}")
    report.update(source_sha=sha, ref=args.branch)
    if sha != args.source_sha or report.capture("git", "-C", str(checkout), "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("A clean checkout at --source-sha is required")
    manifest_data, inputs, pins = inspect_inputs(manifest, checkout, report)
    appid = manifest_data.get("id", manifest_data.get("app-id", ""))
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*(?:\.[A-Za-z_][A-Za-z0-9_-]*){2,}", appid):
        raise ValueError("Invalid application ID")
    if manifest_data.get("branch", args.branch) != args.branch:
        raise ValueError("Requested and manifest branch differ")
    runtime, sdk, version = (manifest_data.get(key, "") for key in ("runtime", "sdk", "runtime-version"))
    component = r"[A-Za-z0-9_.-]+"
    for value in [runtime, sdk, version, manifest_data.get("base", ""), manifest_data.get("base-version", "")]:
        if value and not re.fullmatch(component, value):
            raise ValueError("Invalid runtime identity component")
    if not all([runtime, sdk, version]):
        raise ValueError("Explicit runtime, SDK and runtime-version are required")
    required = required_refs(manifest_data, args.arch)
    output = Path(args.output_dir).absolute()
    if any(p.is_symlink() for p in [output, *output.parents]):
        raise ValueError("Symbolic output paths are not accepted")
    if not output.parent.is_dir():
        raise ValueError("Output parent must exist")
    output = output.parent.resolve() / output.name
    if within(output, checkout) or within(checkout, output) or within(report.directory, output) or within(output, report.directory):
        raise ValueError("Candidate output must be outside checkout and separate from the report directory")
    if args.reuse_receipt:
        if not output.is_dir():
            raise ValueError("Reuse requires an existing candidate directory")
        reuse_candidate(args, report, output, manifest, checkout, sha, manifest_data, inputs, pins, required)
        return
    if output.exists():
        raise ValueError("Output must be new unless --reuse-receipt is explicitly selected")
    tools = {tool: report.capture(tool, "--version") for tool in ["flatpak", "flatpak-builder", "ostree"]}
    runtimes = []
    for ref in required:
        observed = report.capture("flatpak", "info", "--show-ref", ref)
        commit = report.capture("flatpak", "info", "--show-commit", ref)
        if observed != ref or not re.fullmatch(r"[0-9a-f]{64}", commit):
            raise ValueError("Required installed runtime identity mismatch")
        runtimes.append({"ref": observed, "commit": commit})
    output.mkdir(mode=0o700)
    build, repo = output / "build", output / "repo"
    report.phase("building", next_action="On failure inspect partial candidate output; never retry or delete it implicitly.")
    command = ["flatpak-builder", "--disable-updates", f"--arch={args.arch}",
               f"--default-branch={args.branch}", f"--state-dir={output / 'state'}"]
    if args.offline:
        command.append("--disable-download")
    # A manifest may intentionally run build commands; the operator authorizes
    # that selected manifest, not arbitrary shell payloads passed to this wrapper.
    report.run([*command, str(build), str(manifest)], timeout=args.timeout_seconds)
    metadata = regular_file(build / "metadata")
    if metadata.stat().st_size > 1024 * 1024:
        raise ValueError("Built metadata exceeds size budget")
    raw_metadata = metadata.read_text()
    parsed = expected_policy(raw_metadata, appid, args.arch)["Application"]
    if (parsed["runtime"] != f"{runtime}/{args.arch}/{version}"
            or parsed["sdk"] != f"{sdk}/{args.arch}/{version}"):
        raise ValueError("Built application/runtime/SDK metadata differs from requested identity")
    report.phase("exporting")
    report.run(["flatpak", "build-export", f"--arch={args.arch}", str(repo), str(build), args.branch], timeout=args.timeout_seconds)
    bundle = output / f"{appid}-{args.arch}-{args.branch}.flatpak"
    report.run(["flatpak", "build-bundle", f"--arch={args.arch}", str(repo), str(bundle), appid, args.branch], timeout=args.timeout_seconds)
    report.phase("inspecting-export")
    exported = inspect_repository(repo, appid, args.arch, args.branch, raw_metadata, report.capture)
    bundled = inspect_bundle(bundle, appid, args.arch, args.branch, raw_metadata,
                             report.capture, report.directory, exported["commit"])
    unchanged_inputs(manifest, checkout, sha, inputs, pins, report)
    snapshot = output / "built-metadata.ini"
    with snapshot.open("x") as stream:
        stream.write(raw_metadata)
    receipt = report.directory / "artifacts.json"
    atomic_json(receipt, {"schema_version": 2, "kind": "unaccepted-flatpak-candidate",
                "source_sha": sha, "manifest_path": str(manifest), "input_sha256": inputs,
                "manifest_relative_path": str(manifest.relative_to(checkout)), "output_dir": str(output),
                "build_options": {"disable_updates": True, "offline": args.offline},
                "source_pins": pins, "tools": tools, "runtimes": runtimes,
                "application_id": appid, "branch": args.branch, "architecture": args.arch,
                "sdk_extensions_declared": manifest_data.get("sdk-extensions", []),
                "finish_args_declared": manifest_data.get("finish-args", []),
                "repository_path": str(repo), "offline": args.offline,
                "export_inspection": exported, "bundle_inspection": bundled,
                "artifacts": [{"path": str(p), "sha256": digest(p)} for p in [bundle, snapshot]]})
    report.phase("candidate-exported", artifact_receipt_path=str(receipt),
                 next_action="Unaccepted candidate only: inspect exported metadata and obtain separate installed acceptance; no installation or submission performed.")


def preflight(args, report):
    """Read-only observations, including blockers; success is not acceptance."""
    manifest = regular_file(args.manifest).resolve()
    data = json.loads(manifest.read_text())
    appid = data.get("id", data.get("app-id", ""))
    component = r"[A-Za-z0-9_.-]+"
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*(?:\.[A-Za-z_][A-Za-z0-9_-]*){2,}", appid):
        raise ValueError("Invalid application ID")
    for key in ["runtime", "sdk", "runtime-version", "base", "base-version"]:
        if data.get(key) and not re.fullmatch(component, data[key]):
            raise ValueError("Invalid runtime identity component")
    if not all(data.get(key) for key in ["runtime", "sdk", "runtime-version"]):
        raise ValueError("Explicit runtime and SDK required")
    checkout = Path(report.capture("git", "-C", str(manifest.parent), "rev-parse", "--show-toplevel")).resolve()
    sha = report.capture("git", "-C", str(checkout), "rev-parse", "HEAD^{commit}")
    dirty = bool(report.capture("git", "-C", str(checkout), "status", "--porcelain", "--untracked-files=all"))
    report.update(source_sha=sha, source_dirty=dirty, ref=args.branch)
    blockers = ["Packaging checkout is dirty"] if dirty else []
    if args.source_sha and args.source_sha != sha:
        blockers.append("Expected source SHA differs from checkout")
    try:
        _, inputs, pins = inspect_inputs(manifest, checkout, report)
    except ValueError as error:
        inputs, pins = {}, []
        blockers.append(str(error))
    tools = {name: report.capture(name, "--version", timeout=20) for name in ["flatpak", "flatpak-builder", "ostree"]}
    refs = required_refs(data, args.arch)
    runtimes = []
    for ref in refs:
        observed = report.capture("flatpak", "info", "--show-ref", ref, allowed=(0, 1), timeout=20)
        available = observed == ref
        commit = report.capture("flatpak", "info", "--show-commit", ref, timeout=20) if available else None
        if available and not re.fullmatch(r"[0-9a-f]{64}", commit):
            raise ValueError("Invalid installed runtime commit")
        if not available:
            blockers.append(f"Required runtime unavailable: {ref}")
        runtimes.append({"requested": ref, "observed": observed or None, "available": available, "commit": commit})
    installations = []
    for scope in ["--user", "--system"]:
        ref = report.capture("flatpak", "info", "--show-ref", scope, appid, allowed=(0, 1), timeout=20)
        item = {"scope": scope, "ref": ref or None}
        if ref:
            if not ref.startswith(f"app/{appid}/{args.arch}/"):
                raise ValueError("Unexpected installed application identity")
            item["commit"] = report.capture("flatpak", "info", "--show-commit", scope, appid, timeout=20)
            item["metadata"] = report.capture("flatpak", "info", "--show-metadata", scope, appid, timeout=20)
        item["global_overrides"] = report.capture("flatpak", "override", scope, "--show", timeout=20)
        item["app_overrides"] = report.capture("flatpak", "override", scope, "--show", appid, timeout=20)
        installations.append(item)
    receipt = report.directory / "preflight.json"
    atomic_json(receipt, {"schema_version": 1, "kind": "flatpak-read-only-preflight",
        "application_id": appid, "branch": args.branch, "architecture": args.arch,
        "source_sha": sha, "source_dirty": dirty,
        "manifest_path": str(manifest), "input_sha256": inputs, "source_pins": pins,
        "tools": tools, "runtimes": runtimes, "installations": installations,
        "ready_for_candidate_build": not blockers, "blockers": blockers,
        "acceptance": "not-run", "installation": "not-performed"})
    report.phase("preflight-complete", artifact_receipt_path=str(receipt),
        next_action="Read preflight.json blockers; no build, install, launch, service or cloud change performed.")


def main():
    parser = argparse.ArgumentParser(description=__doc__, prog="scripts/build-flatpak.sh")
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--source-sha", help="Expected clean packaging checkout commit (required for builds)")
    parser.add_argument("--arch", required=True, choices=["x86_64", "aarch64"])
    parser.add_argument("--branch", required=True)
    parser.add_argument("--output-dir", help="New candidate output (required for builds)")
    parser.add_argument("--preflight", action="store_true", help="Read-only source/runtime/installed metadata and overrides; never build or install")
    parser.add_argument("--log-dir")
    parser.add_argument("--reuse-receipt", type=Path, help="Verify/reuse existing candidate using its successful schema-2 receipt; never rebuild")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--timeout-seconds", type=int, default=7200)
    args = parser.parse_args()
    if args.preflight and (args.output_dir or args.reuse_receipt):
        parser.error("Preflight does not accept candidate output/reuse arguments")
    if not args.preflight and (not args.source_sha or not args.output_dir):
        parser.error("Build requires --source-sha and --output-dir")
    if (args.source_sha and not re.fullmatch(r"[0-9a-f]{40}", args.source_sha)) or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", args.branch):
        parser.error("Full source SHA and a safe explicit branch are required")
    if not 1 <= args.timeout_seconds <= 86400:
        parser.error("--timeout-seconds must be between 1 and 86400")
    with TaskReport("flatpak-preflight" if args.preflight else "flatpak-build", args.log_dir) as report:
        (preflight if args.preflight else run)(args, report)


if __name__ == "__main__":
    main()
