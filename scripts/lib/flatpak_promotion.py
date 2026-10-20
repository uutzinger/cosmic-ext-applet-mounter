"""Verify selected hosted Flatpak jobs/downloads; never build, install or publish."""
from pathlib import Path
import re

from artifact_inspection import digest, regular_file
from flatpak_inspection import inspect_bundle, expected_policy
from flatpak_tasks import load_receipt, required_refs

APP_ID = 'io.github.uutzinger.cosmic-ext-applet-mounter'
WORKFLOW_PATH = '.github/workflows/build-flatpak.yml'
ARCHITECTURES = ('x86_64', 'aarch64')


def selections(values):
    result = {}
    for value in values:
        match = re.fullmatch(r'(x86_64|aarch64)=([1-9][0-9]*)', value)
        if not match or match[1] in result:
            raise ValueError('Select each Flatpak CPU once with --flatpak-run ARCH=RUN_ID')
        result[match[1]] = match[2]
    return result


def validate_hosted_run(run, artifact_data, job_data, repository, sha, arch):
    if (arch not in ARCHITECTURES or run.get('head_sha') != sha
            or not re.fullmatch(r'[0-9a-f]{40}', sha)
            or run.get('repository', {}).get('full_name') != repository
            or run.get('head_repository', {}).get('full_name') != repository
            or run.get('path') != WORKFLOW_PATH or run.get('event') != 'workflow_dispatch'
            or run.get('status') != 'completed' or run.get('conclusion') not in ('success', 'failure')
            or not isinstance(run.get('run_attempt'), int) or run['run_attempt'] < 1):
        raise ValueError('Flatpak run must be a trusted completed manual build at the expected source')
    artifacts, jobs = artifact_data.get('artifacts', []), job_data.get('jobs', [])
    if (artifact_data.get('total_count') != len(artifacts) or len(artifacts) > 100
            or job_data.get('total_count') != len(jobs) or len(jobs) > 100):
        raise ValueError('Incomplete Flatpak artifact/job listing')
    selected = [item for item in artifacts if item.get('name') == f'flatpak-{arch}']
    selected_jobs = [item for item in jobs if item.get('name') == f'Flatpak {arch}']
    if (len(selected) != 1 or selected[0].get('expired', True)
            or len(selected_jobs) != 1 or selected_jobs[0].get('status') != 'completed'
            or selected_jobs[0].get('conclusion') != 'success'):
        raise ValueError('Selected Flatpak job must succeed and have exactly one unexpired bundle artifact')
    return run['run_attempt']


def checksum_files(folder, arch):
    folder = Path(folder).absolute()
    name = f'{APP_ID}-{arch}-master.flatpak'
    expected = {name, 'built-metadata.ini', 'hosted-artifacts.json',
                'producer-artifacts.json', 'producer-summary.json'}
    if folder.is_symlink() or {p.name for p in folder.iterdir()} != expected | {'SHA256SUMS'}:
        raise ValueError('Unexpected Flatpak download contents')
    for filename in expected | {'SHA256SUMS'}:
        regular_file(folder / filename)
    sums = folder / 'SHA256SUMS'
    if sums.stat().st_size > 16384:
        raise ValueError('Oversized Flatpak checksum file')
    rows = []
    for line in sums.read_text().splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9._-]+)', line)
        if not match or match[2] not in expected:
            raise ValueError('Unsafe or unexpected Flatpak checksum entry')
        rows.append((match[2], match[1]))
    if len(rows) != len(expected) or {name for name, _ in rows} != expected:
        raise ValueError('Missing or duplicate Flatpak checksum entries')
    hashes = dict(rows)
    if any(digest(folder / name) != checksum for name, checksum in rows):
        raise ValueError('Flatpak download checksum mismatch')
    return folder, name, hashes


def verify_download(folder, arch, repository, sha, run_id, attempt, inputs, pins,
                    manifest, capture, scratch):
    folder, name, hashes = checksum_files(folder, arch)
    _, index = load_receipt(folder / 'hosted-artifacts.json')
    _, receipt = load_receipt(folder / 'producer-artifacts.json')
    _, summary = load_receipt(folder / 'producer-summary.json')
    if (index.get('schema_version') != 1 or index.get('kind') != 'unaccepted-hosted-flatpak'
            or index.get('repository') != repository or index.get('workflow') != WORKFLOW_PATH
            or index.get('source_sha') != sha or index.get('run_id') != str(run_id)
            or index.get('run_attempt') != str(attempt) or index.get('architecture') != arch
            or index.get('application_id') != APP_ID
            or receipt.get('schema_version') != 2 or receipt.get('kind') != 'unaccepted-flatpak-candidate'
            or receipt.get('source_sha') != sha or receipt.get('architecture') != arch
            or receipt.get('application_id') != APP_ID or receipt.get('branch') != 'master'
            or receipt.get('manifest_relative_path') != f'packaging/flatpak/{APP_ID}.json'
            or receipt.get('input_sha256') != inputs or receipt.get('source_pins') != pins
            or index.get('source_pins') != pins or index.get('runtimes') != receipt.get('runtimes')
            or receipt.get('finish_args_declared') != manifest['finish-args']
            or receipt.get('sdk_extensions_declared') != manifest.get('sdk-extensions', [])
            or receipt.get('build_options') != {'disable_updates': True, 'offline': False}
            or receipt.get('source_transport') != {'kind': 'manifest-url'}
            or summary.get('schema_version') != 1 or summary.get('task') != 'flatpak-build'
            or summary.get('status') != 'succeeded' or summary.get('exit_code') != 0
            or summary.get('phase') != 'candidate-exported' or summary.get('source_sha') != sha
            or summary.get('source_dirty') is True):
        raise ValueError('Flatpak producer/source/run/input identity mismatch')
    refs = required_refs(manifest, arch)
    runtimes = receipt.get('runtimes', [])
    tools = receipt.get('tools', {})
    if (len(runtimes) != len(refs)
            or any(item.get('ref') != ref or not re.fullmatch(r'[0-9a-f]{64}', item.get('commit', ''))
                   for item, ref in zip(runtimes, refs))
            or set(tools) != {'flatpak', 'flatpak-builder', 'ostree'}
            or any(not isinstance(value, str) or not value for value in tools.values())):
        raise ValueError('Incomplete Flatpak runtime/tool provenance')
    indexed = index.get('artifacts', [])
    expected_index = {key: value for key, value in hashes.items() if key != 'hosted-artifacts.json'}
    if (len(indexed) != 4 or {item['name']: item['sha256'] for item in indexed} != expected_index):
        raise ValueError('Hosted index differs from verified download bytes')
    original = receipt.get('artifacts', [])
    if (len(original) != 2 or {Path(item['path']).name: item['sha256'] for item in original}
            != {name: hashes[name], 'built-metadata.ini': hashes['built-metadata.ini']}):
        raise ValueError('Producer artifact digests differ')
    metadata = regular_file(folder / 'built-metadata.ini')
    if metadata.stat().st_size > 1024 * 1024:
        raise ValueError('Oversized Flatpak metadata')
    text = metadata.read_text()
    policy = expected_policy(text, APP_ID, arch)
    app = policy['Application']
    if (app['runtime'] != f'{manifest["runtime"]}/{arch}/{manifest["runtime-version"]}'
            or app['sdk'] != f'{manifest["sdk"]}/{arch}/{manifest["runtime-version"]}'):
        raise ValueError('Flatpak runtime does not match selected manifest')
    ref = f'app/{APP_ID}/{arch}/master'
    commit = index.get('commit', '')
    if (index.get('ref') != ref or not re.fullmatch(r'[0-9a-f]{64}', commit)
            or any(item.get('ref') != ref or item.get('commit') != commit or item.get('metadata') != policy
                   for item in (receipt.get('export_inspection', {}), receipt.get('bundle_inspection', {})))
            or receipt['bundle_inspection'].get('bundle_sha256') != hashes[name]):
        raise ValueError('Flatpak ref/commit or complete producer metadata mismatch')
    inspect_bundle(folder / name, APP_ID, arch, 'master', text, capture, scratch, commit)
    provenance = {'architecture': arch, 'run_id': str(run_id), 'run_attempt': attempt,
                  'workflow': WORKFLOW_PATH, 'build_source_sha': sha,
                  'run_url': f'https://github.com/{repository}/actions/runs/{run_id}',
                  'bundle': name, 'sha256': hashes[name], 'ref': ref, 'commit': commit,
                  'input_sha256': inputs, 'source_pins': pins, 'runtimes': receipt['runtimes'],
                  'tools': receipt['tools'], 'finish_args': receipt['finish_args_declared'],
                  'metadata_sha256': hashes['built-metadata.ini']}
    return folder / name, provenance
