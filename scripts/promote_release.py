#!/usr/bin/env python3
"""Verify/promote one existing immutable-source build; never build or dispatch."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile


MATRIX = [(os_version, arch) for os_version in ("22.04", "24.04", "26.04")
          for arch in ("amd64", "arm64")]


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def validate_run(run, artifacts, repository, sha):
    if (run.get("repository", {}).get("full_name") != repository
            or run.get("head_repository", {}).get("full_name") != repository
            or run.get("head_sha") != sha
            or run.get("path") != ".github/workflows/build-deb.yml"
            or run.get("event") not in ("workflow_dispatch", "push")
            or run.get("status") != "completed" or run.get("conclusion") != "success"):
        raise ValueError("Build must be a successful trusted same-repository workflow at the exact accepted commit")
    expected = {f"deb-ubuntu-{version}-{arch}" for version, arch in MATRIX}
    names = [item["name"] for item in artifacts]
    if (len(names) != len(expected) or set(names) != expected
            or any(item.get("expired", True) for item in artifacts)):
        raise ValueError("Exactly six unexpired matrix artifacts are required; missing/expired artifacts need a new accepted build")


def verify_remote_tag(repository, tag, sha):
    ref = json.loads(output("gh", "api", f"repos/{repository}/git/ref/tags/{tag}"))["object"]
    for _ in range(8):
        if ref["type"] == "commit":
            if ref["sha"] != sha:
                raise ValueError("Remote tag does not match the accepted local commit")
            return
        if ref["type"] != "tag" or not re.fullmatch(r"[0-9a-f]{40}", ref["sha"]):
            raise ValueError("Invalid remote tag object")
        ref = json.loads(output("gh", "api", f"repos/{repository}/git/tags/{ref['sha']}"))["object"]
    raise ValueError("Remote tag chain is too deep")


def verify_packages(root, tag):
    """Check exact filenames/checksums/identities without executing package code."""
    packages = []
    for version, arch in MATRIX:
        folder = root / f"deb-ubuntu-{version}-{arch}"
        name = f"cosmic-ext-applet-mounter_{tag[1:]}_ubuntu{version}_{arch}.deb"
        package = folder / name
        sums = folder / "SHA256SUMS"
        if (folder.is_symlink() or not folder.is_dir()
                or {p.name for p in folder.iterdir()} != {name, "SHA256SUMS"}
                or package.is_symlink() or sums.is_symlink()):
            raise ValueError("Unexpected or symbolic artifact contents")
        match = re.fullmatch(r"([0-9a-f]{64})  " + re.escape(name) + r"\n?", sums.read_text())
        if not match or hashlib.sha256(package.read_bytes()).hexdigest() != match[1]:
            raise ValueError("Artifact checksum mismatch or unsafe checksum filename")
        for field, expected in [("Package", "cosmic-ext-applet-mounter"),
                                ("Version", tag[1:]), ("Architecture", arch)]:
            if output("dpkg-deb", "-f", str(package), field) != expected:
                raise ValueError("Debian package identity mismatch")
        packages.append(package)
    return packages


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    parser.add_argument("run_id")
    parser.add_argument("--repo")
    parser.add_argument("--publish", action="store_true")
    parser.add_argument("--accepted", action="store_true")
    parser.add_argument("--check-run-only", action="store_true")
    parser.add_argument("--source-sha", help="Pre-tag reuse validation only")
    args = parser.parse_args()
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.tag) or not re.fullmatch(r"[1-9][0-9]*", args.run_id):
        parser.error("Stable version tag and positive numeric run ID required")
    if args.publish and (not args.accepted or args.check_run_only or args.source_sha):
        parser.error("Publication requires --accepted and an existing tag, not pre-tag validation")
    if args.source_sha and (not args.check_run_only or not re.fullmatch(r"[0-9a-f]{40}", args.source_sha)):
        parser.error("--source-sha is only valid for pre-tag --check-run-only")
    repository = args.repo or output("gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        parser.error("Invalid repository identity")
    sha = args.source_sha or output("git", "rev-parse", f"refs/tags/{args.tag}^{{commit}}")
    if not args.source_sha:
        verify_remote_tag(repository, args.tag, sha)
    run = json.loads(output("gh", "api", f"repos/{repository}/actions/runs/{args.run_id}"))
    artifact_data = json.loads(output("gh", "api", f"repos/{repository}/actions/runs/{args.run_id}/artifacts?per_page=100"))
    if artifact_data.get("total_count") != 6:
        raise ValueError("Build must supply exactly the six expected artifacts")
    validate_run(run, artifact_data["artifacts"], repository, sha)
    if args.check_run_only:
        print(f"Verified reusable run {args.run_id} at {sha}; no build/download/publication")
        return
    if output("git", "rev-parse", "HEAD") != sha or output("git", "status", "--porcelain"):
        raise ValueError("Use a clean checkout of the accepted tag for promotion")
    subprocess.run(["bash", "scripts/check-release-publication.sh", args.tag], check=True)
    with tempfile.TemporaryDirectory(prefix="release-promotion-") as temporary:
        root = Path(temporary)
        for version, arch in MATRIX:
            name = f"deb-ubuntu-{version}-{arch}"
            subprocess.run(["gh", "run", "download", args.run_id, "--repo", repository,
                            "--name", name, "--dir", str(root / name)], check=True)
        packages = verify_packages(root, args.tag)
        sums = root / "SHA256SUMS"
        sums.write_text("".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n" for p in packages))
        print(f"Verified {len(packages)} existing artifacts from run {args.run_id}; no build performed")
        if args.publish:
            subprocess.run(["gh", "release", "create", args.tag, *map(str, packages), str(sums),
                            "--repo", repository, "--verify-tag", "--title", args.tag,
                            "--notes-file", f"docs/releases/{args.tag}.md"], check=True)
        else:
            print("Inspection only; --publish --accepted is required to publish")


if __name__ == "__main__":
    main()
