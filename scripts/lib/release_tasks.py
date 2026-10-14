#!/usr/bin/env python3
"""Fixed hosted-build/release-preparation tasks behind the maintained entry points."""

import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile
import time
import uuid
from urllib.parse import quote

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import promote_release as promotion
from project import ARTIFACT_COUNT, PACKAGE, WORKFLOW, WORKFLOW_PATH
from task_output import TaskReport, atomic_json


def metadata(report, endpoint):
    return json.loads(report.capture("gh", "api", endpoint))


def identity(report, ref, repository=None):
    if ref.startswith("-") or not re.fullmatch(r"[A-Za-z0-9_./-]+", ref):
        raise ValueError("Invalid source ref")
    repository = repository or report.capture("gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository identity")
    sha = report.capture("git", "rev-parse", "--verify", f"{ref}^{{commit}}")
    remote = metadata(report, f"repos/{repository}/commits/{quote(ref, safe='')}")
    if not re.fullmatch(r"[0-9a-f]{40}", sha) or remote.get("sha") != sha:
        raise ValueError("Local and remote source revisions differ")
    report.update(repository=repository, ref=ref, source_sha=sha)
    return repository, sha


def validate_selected(run, report):
    state = report.state
    if (run.get("repository", {}).get("full_name") != state["repository"]
            or run.get("head_repository", {}).get("full_name") != state["repository"]
            or run.get("head_sha") != state["source_sha"]
            or run.get("path") != WORKFLOW_PATH
            or run.get("event") not in ("workflow_dispatch", "push")
            or (state["run_id"] and str(run.get("id")) != state["run_id"])):
        raise ValueError("Selected run has wrong repository, workflow or source")


def version_from_packages(root):
    names = [p.name for p in root.glob("deb-*/*.deb")]
    pattern = re.escape(PACKAGE) + r"_([0-9]+\.[0-9]+\.[0-9]+)_ubuntu[0-9.]+_(?:amd64|arm64)\.deb"
    matches = [re.fullmatch(pattern, n) for n in names]
    if len(names) != ARTIFACT_COUNT or not all(matches) or len({m[1] for m in matches}) != 1:
        raise ValueError("Unexpected package set/version")
    return "v" + matches[0][1]


def hosted(report, args):
    repository, sha = identity(report, args.ref, args.repo)
    prefix = f"repos/{repository}/actions/runs"
    run_id, request_id = args.run_id, None
    if args.resume_report:
        previous = json.loads(Path(args.resume_report).read_text())
        if previous.get("source_sha") != sha or previous.get("repository") != repository:
            raise ValueError("Resume receipt belongs to a different source/repository")
        run_id, request_id = previous.get("run_id"), previous.get("request_id")
        if not run_id and not request_id:
            raise ValueError("Receipt has no selected run or dispatch identity; refusing a new dispatch")
    if run_id and not re.fullmatch(r"[1-9][0-9]*", str(run_id)):
        raise ValueError("Positive numeric run ID required")
    report.update(run_id=str(run_id) if run_id else None, request_id=request_id,
                  next_action="Resume with --run-id or --resume-report; do not redispatch after an ambiguous response.")
    if not run_id:
        if not args.wait_for_existing and not args.resume_report:
            request_id = uuid.uuid4().hex
            report.phase("dispatch-intent", request_id=request_id)
            report.run(["gh", "workflow", "run", WORKFLOW, "--repo", repository,
                        "--ref", args.ref, "-f", f"request_id={request_id}"])
        if request_id and not re.fullmatch(r"[0-9a-f]{32}", request_id):
            raise ValueError("Invalid dispatch identity")
        report.phase("selecting-run")
        deadline = time.monotonic() + args.discovery_timeout
        while True:
            runs = json.loads(report.capture("gh", "run", "list", "--repo", repository,
                              "--workflow", WORKFLOW,
                              "--commit", sha, "--limit", "100", "--json",
                              "databaseId,headSha,event,displayTitle"))
            matches = [r for r in runs if r.get("headSha") == sha
                       and r.get("event") in ("workflow_dispatch", "push")
                       and (not request_id or r.get("displayTitle") == f"Debian packages · {request_id}")]
            if len(matches) > 1:
                raise ValueError("Ambiguous run selection; choose an explicit --run-id")
            if matches:
                run_id = str(matches[0]["databaseId"])
                break
            if time.monotonic() >= deadline:
                raise TimeoutError("Run discovery timed out; resume the recorded request, do not redispatch")
            time.sleep(min(args.poll_seconds, 5))
    if not re.fullmatch(r"[1-9][0-9]*", str(run_id)):
        raise ValueError("Selected run has invalid identity")
    report.update(run_id=str(run_id), run_url=f"https://github.com/{repository}/actions/runs/{run_id}",
                  next_action=f"Resume using --run-id {run_id} at source {sha}; remote CI is not implicitly cancelled.")
    deadline = time.monotonic() + args.timeout_seconds
    while True:
        run = metadata(report, f"{prefix}/{run_id}")
        validate_selected(run, report)
        if run.get("status") not in ("queued", "waiting", "pending", "requested", "in_progress", "completed"):
            raise ValueError("Unknown CI run status")
        report.phase(f"ci-{run.get('status')}")
        if run.get("status") == "completed":
            if run.get("conclusion") != "success":
                raise ValueError("Selected CI run did not succeed; inspect that run's failed step")
            break
        if time.monotonic() >= deadline:
            raise TimeoutError("CI wait timed out; resume the recorded run")
        time.sleep(args.poll_seconds)
    destination = Path(args.download_dir or f"target/artifacts/ci-{run_id}").absolute()
    if destination.is_symlink():
        raise ValueError("Artifact destination must not be a symlink")
    if destination.exists():
        if (destination / "receipt.json").is_symlink():
            raise ValueError("Artifact receipt must not be a symlink")
        receipt = json.loads((destination / "receipt.json").read_text())
        if receipt.get("run_id") != str(run_id) or receipt.get("source_sha") != sha:
            raise ValueError("Existing artifact directory has different provenance; refusing overwrite")
        packages = promotion.verify_packages(destination, args.version or version_from_packages(destination), report.capture)
        actual = {str(p.absolute()): promotion.hashlib.sha256(p.read_bytes()).hexdigest() for p in packages}
        if len(receipt["artifacts"]) != ARTIFACT_COUNT or {p["path"]: p["sha256"] for p in receipt["artifacts"]} != actual:
            raise ValueError("Cached artifact receipt does not match verified files")
        report.phase("reused-verified-artifacts", artifact_receipt_path=str(destination / "receipt.json"))
    else:
        artifacts = metadata(report, f"{prefix}/{run_id}/artifacts?per_page=100")
        if artifacts.get("total_count") != ARTIFACT_COUNT:
            raise ValueError(f"Expected {ARTIFACT_COUNT} build artifacts")
        promotion.validate_run(run, artifacts["artifacts"], repository, sha)
        report.phase("downloading")
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix=".download-", dir=destination.parent) as tmp:
            stage = Path(tmp)
            report.run(["gh", "run", "download", str(run_id), "--repo", repository, "--dir", str(stage)], timeout=900)
            packages = promotion.verify_packages(stage, args.version or version_from_packages(stage), report.capture)
            files = [{"path": str(destination / p.relative_to(stage)),
                      "sha256": promotion.hashlib.sha256(p.read_bytes()).hexdigest()} for p in packages]
            atomic_json(stage / "receipt.json", {"schema_version": 1, "source_sha": sha,
                        "run_id": str(run_id), "artifacts": files})
            if destination.exists():
                raise ValueError("Artifact destination appeared concurrently; refusing replacement")
            os.rename(stage, destination)
        report.update(artifact_receipt_path=str(destination / "receipt.json"))
    report.phase("artifacts-ready", next_action="Review artifact receipt and required acceptance; no installation/publication performed.")


def preparation(report, args):
    packages = json.loads(report.capture("cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"))["packages"]
    version = next(p["version"] for p in packages if p["name"] == PACKAGE)
    tag = "v" + version
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag) or report.capture("dpkg-parsechangelog", "-S", "Version") != version:
        raise ValueError("Release versions do not match")
    branch = report.capture("git", "branch", "--show-current")
    if not branch or report.capture("git", "status", "--porcelain"):
        raise ValueError("Use a clean release branch")
    if args.remote.startswith("-") or not re.fullmatch(r"[A-Za-z0-9_.-]+", args.remote):
        raise ValueError("Invalid remote")
    repository, sha = identity(report, branch, args.repo)
    if report.capture("git", "rev-parse", "HEAD") != sha:
        raise ValueError("Release preparation/resume must use the original source commit")
    remote_url = report.capture("git", "remote", "get-url", args.remote)
    match = re.fullmatch(r"(?:git@github\.com:|https://github\.com/|ssh://git@github\.com/)([^/]+/[^/]+?)(?:\.git)?", remote_url)
    if not match or match[1] != repository:
        raise ValueError("Selected Git remote and GitHub repository differ")
    branch_refs = report.capture("git", "ls-remote", args.remote, f"refs/heads/{branch}").splitlines()
    if len(branch_refs) != 1 or branch_refs[0].split()[0] != sha:
        raise ValueError("Push the original release commit before preparing/resuming")
    # A failing transport is not an absent tag. ls-remote without --exit-code
    # succeeds with empty output only when no matching remote tag exists.
    exists = report.run(["git", "show-ref", "--verify", "--quiet", f"refs/tags/{tag}"], allowed=(0, 1))
    local = report.capture("git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}") if exists == 0 else None
    remote = report.capture("git", "ls-remote", args.remote, f"refs/tags/{tag}", f"refs/tags/{tag}^{{}}")
    lines = [line.split() for line in remote.splitlines()]
    remote_sha = next((line[0] for line in lines if line[1].endswith("^{}")), lines[0][0] if lines else None)
    if (local or remote_sha) and not args.resume:
        raise ValueError("Tag already exists; explicit --resume with --run-id is required")
    if local and local != sha or remote_sha and remote_sha != sha:
        raise ValueError("Existing tag points to a different commit; never move it")
    if args.resume and not args.run_id:
        raise ValueError("Resume requires --run-id; refusing an implicit new build")
    if args.run_id:
        if not re.fullmatch(r"[1-9][0-9]*", args.run_id):
            raise ValueError("Positive numeric run ID required")
        report.update(run_id=args.run_id)
        run = metadata(report, f"repos/{repository}/actions/runs/{args.run_id}")
        validate_selected(run, report)
        if not args.resume and (run.get("status") != "completed" or run.get("conclusion") != "success"):
            raise ValueError("Pre-tag reuse requires a successful completed run")
        if not args.resume:
            artifacts = metadata(report, f"repos/{repository}/actions/runs/{args.run_id}/artifacts?per_page=100")
            if artifacts.get("total_count") != ARTIFACT_COUNT:
                raise ValueError("Pre-tag reuse requires the complete artifact set")
            promotion.validate_run(run, artifacts["artifacts"], repository, sha)
    if args.dry_run:
        report.phase("dry-run-complete", next_action="Preflight only; no tag, build or release created.")
        return
    report.phase("preparing-tag", ref=tag, next_action="If interrupted, inspect this receipt and resume the original tag/run; do not retag.")
    if not local and not remote_sha:
        report.run(["git", "tag", "-a", tag, "-m", f"Release {tag}"])
    if not remote_sha:
        report.run(["git", "push", args.remote, f"refs/tags/{tag}"])
    elif not local:
        report.run(["git", "fetch", args.remote, f"refs/tags/{tag}:refs/tags/{tag}"])
    report.phase("tag-ready")
    args.ref, args.version, args.wait_for_existing, args.resume_report = tag, tag, False, None
    hosted(report, args)


def main():
    task = sys.argv[1] if len(sys.argv) > 1 else None
    if task not in ("hosted-build", "release-prepare"):
        raise SystemExit("Use the maintained Bash entry point")
    parser = argparse.ArgumentParser(prog="scripts/build_release.sh" if task == "release-prepare" else "scripts/run_github_linux_build.sh", description=__doc__)
    parser.add_argument("--ref", default=os.environ.get("REF") if task == "hosted-build" else None,
                        help="Hosted source ref (default: current branch; original SHA when resuming a receipt)")
    parser.add_argument("--workflow", choices=[WORKFLOW], default=os.environ.get("WORKFLOW", WORKFLOW),
                        help="Build-only workflow; publication is never dispatched here")
    parser.add_argument("--repo", help="GitHub owner/repository (default: current repository)")
    parser.add_argument("--run-id", help="Select an existing exact-source run; do not dispatch")
    parser.add_argument("--resume-report", help="Hosted task summary.json to resume, including ambiguous dispatch intent")
    parser.add_argument("--wait-for-existing", action="store_true", help="Require one unambiguous exact-source run; never dispatch")
    parser.add_argument("--download-dir", default=os.environ.get("DOWNLOAD_DIR"), help="New or receipt-verified artifact directory")
    parser.add_argument("--log-dir", help="New private task directory (default: target/task-runs/<unique>)")
    parser.add_argument("--poll-seconds", type=int, default=60, help="CI polling interval, 1..300 (default: 60)")
    parser.add_argument("--timeout-seconds", type=int, default=7200, help="Overall CI wait budget (default: 7200)")
    parser.add_argument("--discovery-timeout", type=int, default=120, help="Correlated run discovery budget (default: 120)")
    parser.add_argument("--remote", default="origin", help="Release Git remote (default: origin)")
    parser.add_argument("--dry-run", action="store_true", help="Release preflight only; no tag/push/build/publication")
    parser.add_argument("--resume", action="store_true", help="Release recovery with --run-id; reuse matching tag without retagging")
    args = parser.parse_args(sys.argv[2:])
    args.task = task
    args.version = None
    if not 1 <= args.poll_seconds <= 300 or args.timeout_seconds < 1 or args.discovery_timeout < 1:
        parser.error("Invalid polling/timeout bounds")
    if sum(bool(x) for x in [args.run_id, args.resume_report, args.wait_for_existing]) > 1:
        parser.error("Choose one run-selection/resumption mode")
    if args.workflow != WORKFLOW:
        parser.error("This helper only dispatches the build-only workflow")
    if task == "hosted-build" and (args.resume or args.dry_run or args.remote != "origin"):
        parser.error("Use --resume-report or --run-id for hosted retrieval; tag options belong to release preparation")
    if task == "release-prepare" and (args.resume_report or args.wait_for_existing or args.ref):
        parser.error("Release preparation uses its versioned source and --resume --run-id, not hosted-only options")
    with TaskReport(args.task, args.log_dir) as report:
        if args.task == "hosted-build":
            if args.resume_report and not args.ref:
                previous = json.loads(Path(args.resume_report).read_text())
                args.ref = previous.get("source_sha")
            args.ref = args.ref or report.capture("git", "branch", "--show-current")
            hosted(report, args)
        else:
            preparation(report, args)


if __name__ == "__main__":
    main()
