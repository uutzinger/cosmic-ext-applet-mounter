"""Prepare native GitHub runners and collect verified, unpublished Flatpak bundles."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil

from artifact_inspection import digest, regular_file
from flatpak_inspection import expected_policy
from flatpak_tasks import load_receipt, required_refs, within
from task_output import TaskReport, atomic_json

APP_ID = 'io.github.uutzinger.cosmic-ext-applet-mounter'
MANIFEST = Path('packaging/flatpak') / f'{APP_ID}.json'
WORKFLOW = '.github/workflows/build-flatpak.yml'


def runner_context(sha, arch, environment, machine):
    repository = environment.get('GITHUB_REPOSITORY', '')
    run_id, attempt = (environment.get(k, '') for k in ('GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT'))
    if (environment.get('GITHUB_ACTIONS') != 'true'
            or environment.get('GITHUB_EVENT_NAME') != 'workflow_dispatch'
            or environment.get('GITHUB_SHA') != sha or machine != arch
            or not re.fullmatch(r'[0-9a-f]{40}', sha)
            or not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository)
            or not run_id.isdigit() or not attempt.isdigit()
            or not environment.get('GITHUB_WORKFLOW_REF', '').startswith(f'{repository}/{WORKFLOW}@')):
        raise ValueError('Requires the manual Flatpak workflow on the matching native GitHub runner/source')
    return repository, run_id, attempt


def collect_artifacts(work, sha, arch, repository, run_id, attempt):
    candidate, producer = work / 'candidate', work / 'producer'
    receipt_path, receipt = load_receipt(producer / 'artifacts.json')
    summary_path, summary = load_receipt(producer / 'summary.json')
    ref = f'app/{APP_ID}/{arch}/master'
    if (receipt.get('schema_version') != 2 or receipt.get('kind') != 'unaccepted-flatpak-candidate'
            or receipt.get('source_sha') != sha or receipt.get('architecture') != arch
            or receipt.get('application_id') != APP_ID or receipt.get('branch') != 'master'
            or summary.get('task') != 'flatpak-build' or summary.get('status') != 'succeeded'
            or summary.get('exit_code') != 0 or summary.get('phase') != 'candidate-exported'
            or summary.get('source_sha') != sha or summary.get('source_dirty') is True
            or summary.get('artifact_receipt_path') != str(receipt_path)):
        raise ValueError('Successful exact-source candidate producer required')
    bundle = regular_file(candidate / f'{APP_ID}-{arch}-master.flatpak')
    metadata = regular_file(candidate / 'built-metadata.ini')
    artifacts = receipt.get('artifacts', [])
    if (len(artifacts) != 2 or {item['path']: item['sha256'] for item in artifacts}
            != {str(p): digest(p) for p in (bundle, metadata)}):
        raise ValueError('Candidate artifacts differ from producer checksums')
    exported, bundled = (receipt.get(key, {}) for key in ('export_inspection', 'bundle_inspection'))
    commit = exported.get('commit', '')
    policy = expected_policy(metadata.read_text(), APP_ID, arch)
    if (not re.fullmatch(r'[0-9a-f]{64}', commit)
            or any(item.get('ref') != ref or item.get('commit') != commit
                   or item.get('metadata') != policy for item in (exported, bundled))
            or bundled.get('bundle_sha256') != digest(bundle)):
        raise ValueError('Bundle/export ref, commit or metadata mismatch')
    upload = work / 'upload'
    upload.mkdir()
    files = []
    for source, name in ((bundle, bundle.name), (metadata, metadata.name),
                         (receipt_path, 'producer-artifacts.json'), (summary_path, 'producer-summary.json')):
        target = upload / name
        shutil.copyfile(source, target)
        if digest(target) != digest(source):
            raise ValueError('Artifact changed during collection')
        files.append({'name': name, 'sha256': digest(target)})
    index = upload / 'hosted-artifacts.json'
    atomic_json(index, {'schema_version': 1, 'kind': 'unaccepted-hosted-flatpak',
                       'repository': repository, 'workflow': WORKFLOW, 'source_sha': sha,
                       'run_id': run_id, 'run_attempt': attempt, 'architecture': arch,
                       'application_id': APP_ID, 'ref': ref, 'commit': commit,
                       'source_pins': receipt['source_pins'], 'runtimes': receipt['runtimes'],
                       'artifacts': files, 'acceptance': 'not-run'})
    sums = upload / 'SHA256SUMS'
    sums.write_text(''.join(f'{digest(p)}  {p.name}\n' for p in sorted(upload.iterdir())))
    return index


def operation(args, report):
    repository, run_id, attempt = runner_context(args.source_sha, args.arch, os.environ, platform.machine())
    checkout = Path(report.capture('git', 'rev-parse', '--show-toplevel')).resolve()
    if (report.capture('git', 'rev-parse', 'HEAD') != args.source_sha
            or report.capture('git', 'status', '--porcelain', '--untracked-files=all')):
        raise ValueError('Requires a clean exact-source checkout before runner changes')
    work = args.work_dir.absolute()
    if (any(p.is_symlink() for p in (work, *work.parents))
            or within(work, checkout) or within(checkout, work)
            or within(work, report.directory) or within(report.directory, work)):
        raise ValueError('New work directory must be outside checkout and separate from logs')
    work.mkdir(mode=0o700)
    report.update(source_sha=args.source_sha, source_dirty=False, repository=repository,
                  run_id=run_id, ref='master')
    report.phase('preparing-runner')
    report.run(['sudo', 'apt-get', 'update'], timeout=600)
    report.run(['sudo', 'apt-get', 'install', '--no-install-recommends', '--yes',
                'flatpak', 'flatpak-builder', 'ostree', 'dbus-daemon', 'git', 'ca-certificates'], timeout=600)
    # Ubuntu 24.04 AppArmor otherwise blocks bubblewrap's user namespace.
    # This changes only the ephemeral CI VM, never the operator's host.
    if Path('/proc/sys/kernel/apparmor_restrict_unprivileged_userns').exists():
        report.run(['sudo', 'sysctl', '-w', 'kernel.apparmor_restrict_unprivileged_userns=0'])
    data = json.loads((checkout / MANIFEST).read_text())
    if data.get('id') != APP_ID:
        raise ValueError('Unexpected manifest application')
    report.run(['flatpak', 'remote-add', '--user', '--if-not-exists', 'flathub',
                'https://flathub.org/repo/flathub.flatpakrepo'])
    report.run(['dbus-run-session', '--', 'flatpak', 'install', '--user', '--noninteractive',
                '--assumeyes', '--arch=' + args.arch, 'flathub', *required_refs(data, args.arch)], timeout=1800)
    report.phase('building-candidate')
    report.run(['dbus-run-session', '--', 'bash', 'scripts/build-flatpak.sh',
                '--manifest', str(checkout / MANIFEST), '--source-sha', args.source_sha,
                '--arch', args.arch, '--branch', 'master', '--output-dir', str(work / 'candidate'),
                '--log-dir', str(work / 'producer'), '--timeout-seconds', '5400'], timeout=5700)
    index = collect_artifacts(work, args.source_sha, args.arch, repository, run_id, attempt)
    report.phase('artifacts-collected', artifact_receipt_path=str(index),
                 next_action='Download and review the existing bundles; installed acceptance/publication remain separate. No rebuild required.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--arch', choices=('x86_64', 'aarch64'), required=True)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--work-dir', type=Path, required=True)
    parser.add_argument('--log-dir', required=True)
    args = parser.parse_args()
    with TaskReport('flatpak-ci-build', args.log_dir) as report:
        operation(args, report)


if __name__ == '__main__':
    main()
