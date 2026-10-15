"""Live check of which rclone bisync compatibility handling a OneDrive drive type needs.

Creates fresh token-named test folders on an existing rclone OneDrive remote.
Legacy mode checks post-initial-sync stability. --round-trip additionally compares
untouched originals with independent downloads and measures sync transfers.
Deletes only its test folders after ownership-marker confirmation. Local evidence
is retained. Run only with disposable sample data.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import sys
import time
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from task_output import TaskReport, atomic_json

PREFIX = "CloudMounter-GuardTest-"
# Mirrors the generic rclone bisync settings in src/sync.rs (without recovery dirs).
GENERIC = ["--resilient", "--recover", "--conflict-resolve", "none", "--conflict-loser", "pathname",
           "--conflict-suffix", "conflict", "--create-empty-src-dirs", "--compare", "size,modtime",
           "--max-delete", "1000"]
# Mirrors the SharePoint-specific additions in src/sync.rs.
COMPAT = ["--resilient", "--recover", "--conflict-resolve", "none", "--conflict-loser", "pathname",
          "--conflict-suffix", "conflict", "--create-empty-src-dirs", "--compare", "modtime,checksum",
          "--max-delete", "1000", "--modify-window", "1s", "--ignore-size", "--ignore-checksum",
          "--check-sync", "false"]
PROFILES = {"generic": GENERIC, "compat": COMPAT}
RCLONE_TIMEOUT = 900
MAX_COPIED_FILES = 20
MAX_COPIED_BYTES = 5 * 1024 * 1024


def synthetic_office(directory):
    """Minimal OOXML files; a real office suite file reproduces service rewrites better."""
    for name, body in [("synthetic.docx", "word/document.xml"), ("synthetic.xlsx", "xl/workbook.xml")]:
        with zipfile.ZipFile(directory / name, "w") as archive:
            archive.writestr("[Content_Types].xml", "<Types/>")
            archive.writestr(body, "<root>Cloud Mounter guard test</root>")


def seed_local(directory, files_dir):
    directory.mkdir(parents=True)
    copied, total = 0, 0
    for source in sorted(Path(files_dir).iterdir()) if files_dir else []:
        size = source.stat().st_size
        if not source.is_file() or source.is_symlink():
            continue
        if copied >= MAX_COPIED_FILES or total + size > MAX_COPIED_BYTES:
            raise ValueError("Test files directory exceeds the file-count or size limit")
        shutil.copy2(source, directory / source.name)
        copied, total = copied + 1, total + size
    if not files_dir:
        synthetic_office(directory)
    if not (directory / "plain.txt").exists():
        (directory / "plain.txt").write_text("Cloud Mounter guard test\n")
    if not (directory / "binary.bin").exists():
        (directory / "binary.bin").write_bytes(os.urandom(4096))
    return files_dir is None


def local_snapshot(directory):
    result = {}
    for path in sorted(p for p in directory.rglob("*") if p.is_file()):
        stat = path.stat()
        result[path.relative_to(directory).as_posix()] = [
            stat.st_size, stat.st_mtime_ns, hashlib.sha256(path.read_bytes()).hexdigest()]
    return result


def remote_snapshot(report, location):
    entries = json.loads(report.capture("rclone", "lsjson", "-R", "--files-only", "--hash", location,
                                        timeout=RCLONE_TIMEOUT))
    return {e["Path"]: [e.get("Size"), e.get("ModTime"), e.get("Hashes")] for e in entries}


def check_counts(report, local, location):
    """Counts from rclone's combined check report (symbols are structured, not prose)."""
    text = report.capture("rclone", "check", str(local), location, "--combined", "-",
                          timeout=RCLONE_TIMEOUT, allowed=(0, 1))
    counts = {"equal": 0, "differ": 0, "only_local": 0, "only_remote": 0}
    names = {"=": "equal", "*": "differ", "+": "only_local", "-": "only_remote"}
    differing = []
    for line in text.splitlines():
        symbol, _, name = line.partition(" ")
        if symbol in names:
            counts[names[symbol]] += 1
            if symbol != "=":
                differing.append(f"{symbol} {name}")
    return counts, differing


def observe(report, local, location):
    counts, differing = check_counts(report, local, location)
    return {"local": local_snapshot(local), "remote": remote_snapshot(report, location),
            "check": counts, "check_differences": differing}


def bisync(report, local, location, workdir, flags, resync=False):
    argv = ["rclone", "bisync", str(local), location, "--workdir", str(workdir), *flags, "-v"]
    if resync:
        argv.append("--resync")
    return report.run(argv, timeout=RCLONE_TIMEOUT, allowed=(0, 1, 2, 7))


def run_profile(report, args, name, files_dir, run_dir):
    token = secrets.token_hex(6)
    folder = f"{PREFIX}{token}"
    location = f"{args.remote}:{folder}"
    marker = f".cloud-mounter-guard-test-{token}"
    local, workdir = run_dir / f"local-{name}", run_dir / f"workdir-{name}"
    workdir.mkdir()
    synthetic = seed_local(local, files_dir)
    (local / marker).write_text(token + "\n")
    flags, stages, issues = PROFILES[name], {}, []
    result = {"profile": name, "folder": folder, "synthetic_office": synthetic, "stages": stages,
              "issues": issues}
    try:
        # OneDrive bisync --resync reads Path2 before uploading Path1.
        # It cannot bootstrap a missing remote directory.
        report.phase(f"{name}-create-test-folder")
        report.run(["rclone", "mkdir", location], timeout=RCLONE_TIMEOUT)
        report.phase(f"{name}-initial-sync")
        stages["initial_exit"] = bisync(report, local, location, workdir, flags, resync=True)
        first = observe(report, local, location)
        report.phase(f"{name}-sync-without-edits")
        stages["second_exit"] = bisync(report, local, location, workdir, flags)
        second = observe(report, local, location)
        stable = first["local"] == second["local"] and first["remote"] == second["remote"]
        stages["stable_without_edits"] = stable
        stages["hash_types"] = sorted({h for v in second["remote"].values() for h in (v[2] or {})})
        stages["check"] = second["check"]
        if not stable:
            issues.append("second sync with no edits changed files (service rewrite or metadata drift)")
        if stages["initial_exit"] or stages["second_exit"]:
            issues.append("bisync reported a non-zero exit")
        last = second
        if args.web_edit:
            print(f"\nOpen {folder}/ on the drive in the web editor, change and save one Office file, "
                  f"then press Enter.", file=sys.stderr, flush=True)
            input()
            report.phase(f"{name}-sync-after-web-edit")
            stages["edit_exit"] = bisync(report, local, location, workdir, flags)
            edited = observe(report, local, location)
            stages["edit_changed_files"] = sorted(
                {k for k in edited["local"] if edited["local"][k] != second["local"].get(k)})
            stages["post_edit_exit"] = bisync(report, local, location, workdir, flags)
            settled = observe(report, local, location)
            stages["stable_after_web_edit"] = (edited["local"] == settled["local"]
                                               and edited["remote"] == settled["remote"])
            if not stages["stable_after_web_edit"]:
                issues.append("sync after a web edit did not settle on the next run")
            if stages["edit_exit"] or stages["post_edit_exit"]:
                issues.append("bisync reported a non-zero exit after the web edit")
            last = settled
            stages["check"] = settled["check"]
        # Content differences are informational for compat (its flags ignore size/hash on purpose).
        if name == "generic" and last["check"]["differ"]:
            issues.append("remote and local differ in size or hash after sync")
        result["check_differences"] = last["check_differences"]
    finally:
        result["cleanup"] = cleanup(report, args, location, marker)
    result["clean"] = not issues
    return result


def cleanup(report, args, location, marker):
    if args.keep:
        return "kept"
    folder = location.split(":", 1)[1]
    names = report.capture("rclone", "lsf", location, timeout=RCLONE_TIMEOUT, allowed=(0, 3, 4)).splitlines()
    if not folder.startswith(PREFIX) or marker not in names:
        return "skipped: marker not confirmed, remove the test folder by hand"
    report.run(["rclone", "purge", location], timeout=RCLONE_TIMEOUT)
    return "purged"


def verdict(results):
    by = {r["profile"]: r for r in results}
    if by["generic"]["clean"]:
        return "generic settings are sufficient for the stages run"
    if "compat" in by and by["compat"]["clean"]:
        return "needs the compatibility profile (Office-rewrite handling)"
    if "compat" in by:
        return "neither profile was stable; investigate before choosing guards"
    return "generic settings were not stable"


def parse(argv):
    parser = argparse.ArgumentParser(prog="scripts/onedrive-mirror-guard-check.sh", description=__doc__)
    parser.add_argument("--remote", required=True, help="Existing rclone remote of type onedrive (name only)")
    parser.add_argument("--drive", required=True, choices=["personal", "business"],
                        help="Drive type label recorded in the results")
    parser.add_argument("--files-dir", help="Directory with real .docx/.xlsx test files (max 20 files, 5 MB)")
    parser.add_argument("--web-edit", action="store_true",
                        help="Pause so you can edit a file in the web editor, then sync twice")
    parser.add_argument("--keep", action="store_true", help="Keep the remote and local test folders")
    parser.add_argument("--plan", action="store_true", help="Validate the remote and print the plan only")
    parser.add_argument("--log-dir")
    parser.add_argument("--round-trip", action="store_true",
                        help="Schema-2 original/upload/download comparison and measured repeated syncs")
    parser.add_argument("--settle-seconds", type=int, default=10,
                        help="Round-trip observation interval (0..60 seconds; default 10)")
    args = parser.parse_args(argv)
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", args.remote):
        parser.error("--remote must be a plain rclone remote name without ':'")
    if args.files_dir and not Path(args.files_dir).is_dir():
        parser.error("--files-dir must be an existing directory")
    if not 0 <= args.settle_seconds <= 60:
        parser.error("--settle-seconds must be between 0 and 60")
    return args


def main(argv=None):
    args = parse(sys.argv[1:] if argv is None else argv)
    os.chdir(Path(os.environ.get("GUARD_CHECK_ROOT") or Path(__file__).resolve().parents[2]))
    with TaskReport("onedrive-guard-check", args.log_dir) as report:
        report.update(ref=f"{args.drive}:{args.remote}")
        listing = report.capture("rclone", "listremotes", "--long", timeout=60)
        types = {}
        for line in listing.splitlines():
            name, _, kind = line.partition(":")
            types[name.strip()] = kind.strip()
        if types.get(args.remote) != "onedrive":
            raise ValueError("Remote not found or not of type onedrive")
        if args.plan:
            if args.round_trip:
                print("round-trip plan: separate original/upload/download trees, two observations, "
                      "two measured no-edit syncs; separate token-scoped cloud folders; "
                      "compatibility candidate only if generic fails; marker-verified cleanup. "
                      f"Observation interval: {args.settle_seconds}s; web edit: {args.web_edit}.", file=sys.stderr)
            else:
                print(f"plan: create {PREFIX}<token> on {args.remote}:, run generic profile"
                      f"{' with a web-edit pause' if args.web_edit else ''}, then the compat profile only if "
                      f"generic is unstable, then purge only that folder.", file=sys.stderr)
            report.phase("plan-complete")
            return
        if args.round_trip:
            from mirror_guard_roundtrip import run
            run(report, args)
            report.phase("check-complete")
            return
        run_dir = report.directory / "work"
        run_dir.mkdir()
        results = [run_profile(report, args, "generic", args.files_dir, run_dir)]
        if not results[0]["clean"]:
            results.append(run_profile(report, args, "compat", args.files_dir, run_dir))
        summary = {"schema_version": 1, "drive": args.drive, "remote": args.remote,
                   "profiles": results, "verdict": verdict(results),
                   "note": "Results apply to this drive type and these files only."}
        atomic_json(report.directory / "results.json", summary)
        print(f"verdict ({args.drive}): {summary['verdict']}\nresults: {report.directory / 'results.json'}",
              file=sys.stderr)
        report.phase("check-complete")


if __name__ == "__main__":
    main()
