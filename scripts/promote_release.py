#!/usr/bin/env python3
"""Verify/promote existing Debian and selected Flatpak artifacts; never build or dispatch."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

sys.path.insert(0, str(Path(__file__).resolve().parent / "lib"))
from task_output import TaskReport, atomic_json
from artifact_inspection import MATRIX, verify_matrix
from project import ARTIFACT_COUNT, WORKFLOW_PATH
from artifact_inspection import digest
from flatpak_tasks import inspect_inputs
from flatpak_promotion import selections, validate_hosted_run, verify_download, APP_ID


def validate_run(run, artifacts, repository, sha):
    if (run.get("repository", {}).get("full_name") != repository
            or run.get("head_repository", {}).get("full_name") != repository
            or run.get("head_sha") != sha
            or run.get("path") != WORKFLOW_PATH
            or run.get("event") not in ("workflow_dispatch", "push")
            or run.get("status") != "completed" or run.get("conclusion") != "success"):
        raise ValueError("Build must be a successful trusted same-repository workflow at the exact accepted commit")
    expected = {f"deb-ubuntu-{version}-{arch}" for version, arch in MATRIX}
    names = [item["name"] for item in artifacts]
    if (len(names) != len(expected) or set(names) != expected
            or any(item.get("expired", True) for item in artifacts)):
        raise ValueError(f"Exactly {ARTIFACT_COUNT} unexpired matrix artifacts are required; missing/expired artifacts need a new accepted build")


def verify_remote_tag(repository, tag, sha, capture):
    ref = json.loads(capture("gh", "api", f"repos/{repository}/git/ref/tags/{tag}"))["object"]
    for _ in range(8):
        if ref["type"] == "commit":
            if ref["sha"] != sha:
                raise ValueError("Remote tag does not match the accepted local commit")
            return
        if ref["type"] != "tag" or not re.fullmatch(r"[0-9a-f]{40}", ref["sha"]):
            raise ValueError("Invalid remote tag object")
        ref = json.loads(capture("gh", "api", f"repos/{repository}/git/tags/{ref['sha']}"))["object"]
    raise ValueError("Remote tag chain is too deep")


def verify_packages(root, tag, capture):
    """Check exact filenames/checksums/identities without executing package code."""
    return verify_matrix(root, tag, capture)


def flatpak_downloads(args, report, repository, destination, expected_sha=None):
    selected = selections(args.flatpak_run)
    checkout = Path(report.capture('git', 'rev-parse', '--show-toplevel')).resolve()
    manifest = checkout / 'packaging/flatpak' / f'{APP_ID}.json'
    data, inputs, pins = inspect_inputs(manifest, checkout, report)
    app_sources = [source for module in data.get('modules', [])
                   if module.get('name') == 'cosmic-ext-applet-mounter'
                   for source in module.get('sources', []) if isinstance(source, dict)
                   and source.get('type') == 'git']
    if len(app_sources) != 1:
        raise ValueError('Expected one pinned application Git source')
    package = tomllib.loads(report.capture('git', '-C', str(checkout), 'show',
                                         app_sources[0]['commit'] + ':Cargo.toml'))['package']
    if package.get('name') != 'cosmic-ext-applet-mounter' or package.get('version') != args.tag[1:]:
        raise ValueError('Pinned Flatpak app version does not match the proposed release')
    contexts, packages, provenance = {}, [], []
    for arch, run_id in selected.items():
        if run_id not in contexts:
            prefix = f'repos/{repository}/actions/runs/{run_id}'
            contexts[run_id] = tuple(json.loads(report.capture('gh', 'api', prefix + suffix))
                                     for suffix in ('', '/artifacts?per_page=100', '/jobs?filter=latest&per_page=100'))
        run, artifacts, jobs = contexts[run_id]
        if str(run.get('id')) != run_id:
            raise ValueError('Unexpected Flatpak run identity')
        sha = expected_sha or run.get('head_sha', '')
        attempt = validate_hosted_run(run, artifacts, jobs, repository, sha, arch)
        folder = destination / f'flatpak-{arch}'
        report.run(['gh', 'run', 'download', run_id, '--repo', repository,
                    '--name', f'flatpak-{arch}', '--dir', str(folder)], timeout=900)
        bundle, details = verify_download(folder, arch, repository, sha, run_id, attempt,
                                         inputs, pins, data, report.capture, report.directory)
        packages.append(bundle)
        provenance.append(details)
    return packages, provenance


def inspect_flatpak_candidates(args, report, repository):
    destination = report.directory / 'flatpak-artifacts'
    destination.mkdir()
    report.update(acceptance='not-run', publication='not-performed')
    report.phase('inspecting-flatpak-candidates')
    bundles, provenance = flatpak_downloads(args, report, repository, destination)
    evidence = report.directory / 'FLATPAK-PROVENANCE.json'
    atomic_json(evidence, {'schema_version': 1, 'repository': repository,
                          'proposed_tag': args.tag, 'publication_ready': False, 'flatpaks': provenance})
    receipt = report.directory / 'artifacts.json'
    atomic_json(receipt, {'schema_version': 1, 'kind': 'flatpak-candidate-inspection',
                         'repository': repository, 'publication_ready': False,
                         'artifact_directory': str(destination),
                         'artifacts': [{'name': p.name, 'sha256': digest(p)} for p in [*bundles, evidence]]})
    report.phase('inspection-complete', artifact_receipt_path=str(receipt),
                 next_action='Candidates retained for focused installation/review; exact-tag publication and acceptance remain separate. No build or installation performed.')


def operation(args, report):
    capture = report.capture

    def command(argv):
        return report.run(argv, timeout=900)

    repository = args.repo or capture("gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository identity")
    if args.inspect_flatpaks:
        report.update(repository=repository, ref=args.tag)
        inspect_flatpak_candidates(args, report, repository)
        return
    sha = args.source_sha or capture("git", "rev-parse", f"refs/tags/{args.tag}^{{commit}}")
    report.update(repository=repository, ref=args.tag, source_sha=sha, run_id=args.run_id,
                  run_url=f"https://github.com/{repository}/actions/runs/{args.run_id}",
                  next_action="Inspect the private log and remote release state before retrying publication; never replace assets automatically.")
    if not args.source_sha:
        verify_remote_tag(repository, args.tag, sha, capture)
    run = json.loads(capture("gh", "api", f"repos/{repository}/actions/runs/{args.run_id}"))
    artifact_data = json.loads(capture("gh", "api", f"repos/{repository}/actions/runs/{args.run_id}/artifacts?per_page=100"))
    if artifact_data.get("total_count") != ARTIFACT_COUNT:
        raise ValueError(f"Build must supply exactly the {ARTIFACT_COUNT} expected artifacts")
    validate_run(run, artifact_data["artifacts"], repository, sha)
    if args.check_run_only:
        report.phase("run-validated", next_action="Run metadata verified only; no build, download or publication.")
        return
    if capture("git", "rev-parse", "HEAD") != sha or capture("git", "status", "--porcelain"):
        raise ValueError("Use a clean checkout of the accepted tag for promotion")
    command(["bash", "scripts/check-release-publication.sh", args.tag])
    report.phase("downloading")
    with tempfile.TemporaryDirectory(prefix="release-promotion-") as temporary:
        root = Path(temporary)
        for version, arch in MATRIX:
            name = f"deb-ubuntu-{version}-{arch}"
            command(["gh", "run", "download", args.run_id, "--repo", repository,
                     "--name", name, "--dir", str(root / name)])
        packages = verify_packages(root, args.tag, capture)
        flatpaks = []
        if args.flatpak_run:
            bundles, flatpaks = flatpak_downloads(args, report, repository, root, sha)
            packages.extend(bundles)
            evidence = root / 'FLATPAK-PROVENANCE.json'
            atomic_json(evidence, {'schema_version': 1, 'repository': repository,
                                  'tag': args.tag, 'release_source_sha': sha, 'flatpaks': flatpaks})
            packages.append(evidence)
        sums = root / "SHA256SUMS"
        sums.write_text("".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n" for p in packages))
        receipt = report.directory / "artifacts.json"
        origins = {item['bundle']: item['run_url'] for item in flatpaks}
        if flatpaks:
            origins['FLATPAK-PROVENANCE.json'] = 'verified-selected-flatpak-runs'
        origins['SHA256SUMS'] = 'verified-release-assets'
        atomic_json(receipt, {"schema_version": 1, "source_sha": sha, "run_id": args.run_id,
                    "repository": repository, "flatpaks": flatpaks, "artifacts": [{"name": p.name,
                    "sha256": hashlib.sha256(p.read_bytes()).hexdigest(),
                    "origin": origins.get(p.name, report.state["run_url"])} for p in [*packages, sums]]})
        report.phase("artifacts-verified", artifact_receipt_path=str(receipt))
        if args.publish:
            report.phase("publishing")
            command(["gh", "release", "create", args.tag, *map(str, packages), str(sums),
                     "--repo", repository, "--verify-tag", "--title", args.tag,
                     "--notes-file", f"docs/releases/{args.tag}.md"])
            report.phase("published", next_action="Verify public asset digests against the artifact receipt and record acceptance; no rebuild performed.")
        else:
            report.phase("inspection-complete", next_action="Inspection only; publication requires separate approval and --publish --accepted.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    parser.add_argument("run_id", nargs='?', help='Trusted Debian build run (required except --inspect-flatpaks)')
    parser.add_argument("--repo")
    parser.add_argument("--log-dir", help="New private log/summary directory (default: target/task-runs/...) ")
    parser.add_argument("--publish", action="store_true")
    parser.add_argument("--accepted", action="store_true")
    parser.add_argument("--check-run-only", action="store_true")
    parser.add_argument("--source-sha", help="Pre-tag reuse validation only")
    parser.add_argument('--flatpak-run', action='append', default=[], metavar='ARCH=RUN_ID',
                        help='Explicit accepted hosted Flatpak job; repeat for x86_64/aarch64')
    parser.add_argument('--inspect-flatpaks', action='store_true',
                        help='Download/inspect candidates without a tag or Debian run; cannot publish/install')
    args = parser.parse_args()
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.tag) or (args.run_id is not None and not re.fullmatch(r"[1-9][0-9]*", args.run_id)):
        parser.error("Stable version tag and positive numeric run ID required")
    try:
        selections(args.flatpak_run)
    except ValueError as error:
        parser.error(str(error))
    if args.inspect_flatpaks:
        if args.run_id or not args.flatpak_run or args.publish or args.accepted or args.check_run_only or args.source_sha:
            parser.error('--inspect-flatpaks requires explicit Flatpak runs, no Debian run/tag validation/publication flags')
    elif args.run_id is None:
        parser.error('Debian run ID is required for release promotion')
    if args.flatpak_run and args.check_run_only:
        parser.error('--check-run-only is Debian metadata only; use --inspect-flatpaks for candidate inspection')
    if args.publish and (not args.accepted or args.check_run_only or args.source_sha):
        parser.error("Publication requires --accepted and an existing tag, not pre-tag validation")
    if args.source_sha and (not args.check_run_only or not re.fullmatch(r"[0-9a-f]{40}", args.source_sha)):
        parser.error("--source-sha is only valid for pre-tag --check-run-only")
    with TaskReport("release-promotion", args.log_dir) as report:
        operation(args, report)


if __name__ == "__main__":
    main()
