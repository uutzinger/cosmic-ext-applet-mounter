"""Schema-2 guard evidence: immutable originals, independent downloads and transfer counters.

Called only by mirror_guard_check's explicit --round-trip mode. All cloud mutations
are confined to fresh token-named folders; no services or mounts are involved.
"""
import hashlib
import json
from pathlib import Path
import secrets
import shutil
import sys
import time

import mirror_guard_check as core
from task_output import atomic_json

MAX_LOG = 8 * 1024 * 1024


def version_evidence(report):
    # version has no JSON mode. Preserve opaque provenance, never parse it to
    # select behavior or infer capability; actual operations supply errors.
    return {'format': 'text', 'output': report.capture('rclone', 'version', timeout=60)}


def content(snapshot):
    return {name: [item[0], item[2]] for name, item in snapshot.items()}


def validate_source(source):
    files = list(source.iterdir())
    if not files or len(files) > core.MAX_COPIED_FILES:
        raise ValueError('Sample directory must contain 1..20 regular files')
    if any(p.is_symlink() or not p.is_file() for p in files):
        raise ValueError('Sample directory must contain regular files only, no symlinks or subdirectories')
    if sum(p.stat().st_size for p in files) > core.MAX_COPIED_BYTES:
        raise ValueError('Sample directory exceeds 5 MB')


def changes(before, after):
    return sorted(name for name in before.keys() | after.keys() if before.get(name) != after.get(name))


def metadata(report, location):
    entries = json.loads(report.capture('rclone', 'lsjson', '-R', '--files-only',
                                       '--hash-type', 'quickxor', str(location),
                                       timeout=core.RCLONE_TIMEOUT))
    result = {}
    for entry in entries:
        name = entry['Path']
        if name in result or Path(name).is_absolute() or '..' in Path(name).parts:
            raise ValueError('Invalid or duplicate listing path')
        result[name] = {'size': entry.get('Size'), 'mtime': entry.get('ModTime'),
                        'quickxor': (entry.get('Hashes') or {}).get('quickxor')}
    return result


def counters(path):
    if not path.is_file() or path.stat().st_size > MAX_LOG:
        raise ValueError('Missing or oversized structured transfer log')
    last = None
    with path.open() as stream:
        for line in stream:
            entry = json.loads(line)
            if 'stats' in entry:
                last = entry['stats']
    if last is None:
        raise ValueError('No structured final transfer statistics')
    for key in ('bytes', 'transfers', 'errors'):
        if type(last.get(key)) is not int or last[key] < 0:
            raise ValueError('Invalid structured transfer counters')
    return {key: last.get(key) for key in
            ('bytes', 'transfers', 'errors', 'checks', 'elapsedTime', 'serverSideCopies', 'serverSideCopyBytes')}


class Experiment:
    def __init__(self, report, args):
        self.report, self.args = report, args
        self.work = report.directory / 'work'
        self.work.mkdir()
        self.path = report.directory / 'results.json'
        self.data = {'schema_version': 2, 'status': 'running', 'drive': args.drive,
                     'remote': args.remote, 'settle_seconds': args.settle_seconds,
                     'no_edit_runs': 2, 'web_edit_requested': args.web_edit,
                     'folders': [], 'commands': [], 'profiles': [], 'issues': [],
                     'verdict': 'incomplete', 'note': 'Evidence applies only to these samples and observation times.'}
        self.save()

    def save(self):
        atomic_json(self.path, self.data)

    def wait(self):
        # Explicit bounded observation interval, never an unbounded convergence loop.
        time.sleep(self.args.settle_seconds)

    def transfer(self, label, argv, diagnostic=False):
        self.report.phase(label)
        path = self.report.directory / f'{len(self.data["commands"]):03d}-{label}.jsonl'
        record = {'stage': label, 'diagnostic': diagnostic, 'log': str(path),
                  'started_at_unix': time.time(), 'operation': argv[1]}
        self.data['commands'].append(record)
        self.save()
        command = [*argv, '--use-json-log', '--log-file', str(path), '--log-level', 'INFO',
                   '--stats', '1s', '--stats-log-level', 'NOTICE',
                   '--log-file-max-size', '8M', '--log-file-max-backups', '1']
        try:
            record['exit_code'] = self.report.run(command, timeout=core.RCLONE_TIMEOUT,
                                                   allowed=tuple(range(256)))
            record['stats'] = counters(path)
            if record['exit_code'] or record['stats']['errors']:
                raise ValueError(f'{label}: transfer failed; inspect its structured log')
            return record
        finally:
            record['finished_at_unix'] = time.time()
            self.save()

    def folder(self, label):
        token = secrets.token_hex(16)
        folder = f'{core.PREFIX}{token}'
        record = {'label': label, 'location': f'{self.args.remote}:{folder}',
                  'token': token, 'marker': f'.cloud-mounter-guard-test-{token}',
                  'cleanup': 'pending'}
        self.data['folders'].append(record)
        self.save()
        self.report.run(['rclone', 'mkdir', record['location']], timeout=core.RCLONE_TIMEOUT)
        # Establish ownership before bulk transfer, so partial uploads can be cleaned.
        marker = self.work / record['marker']
        marker.write_text(token + '\n')
        self.report.run(['rclone', 'copyto', str(marker),
                         record['location'] + '/' + record['marker']], timeout=core.RCLONE_TIMEOUT)
        record['ownership_established'] = True
        self.save()
        return record

    def snapshot(self, local, location):
        return {'local': core.local_snapshot(local), 'remote': metadata(self.report, location),
                'observed_at_unix': time.time()}

    def download(self, label, folder, expected):
        destination = self.work / label
        destination.mkdir()  # Must be new and empty; never overwrite earlier evidence.
        self.transfer(label, ['rclone', 'copy', folder['location'], str(destination),
                              '--ignore-times', '--ignore-size', '--ignore-checksum'], diagnostic=True)
        actual = core.local_snapshot(destination)
        downloaded_metadata = metadata(self.report, destination)
        if any(not downloaded_metadata.get(n, {}).get('quickxor') for n in actual):
            raise ValueError('Downloaded-file checksum evidence is missing')
        expected_content = content(expected)
        actual_content = content(actual)
        return {'directory': str(destination), 'files': actual, 'metadata': downloaded_metadata,
                'missing': sorted(expected.keys() - actual.keys()),
                'extra': sorted(actual.keys() - expected.keys()),
                'byte_differences': changes(expected_content, actual_content)}

    def roundtrip(self, baseline):
        folder = self.folder('roundtrip')
        upload = self.work / 'roundtrip-upload'
        shutil.copytree(baseline, upload)
        shutil.copy2(self.work / folder['marker'], upload / folder['marker'])
        before = core.local_snapshot(upload)
        result = {'original': before, 'original_metadata': metadata(self.report, upload),
                  'observations': [], 'location': folder['location']}
        self.data['roundtrip'] = result
        self.save()
        self.transfer('roundtrip-upload', ['rclone', 'copy', str(upload), folder['location']], diagnostic=True)
        # Immediate and one delayed observation expose changes within a stated window.
        for index in range(2):
            if index:
                self.wait()
            remote = metadata(self.report, folder['location'])
            observed = {'remote': remote, 'observed_at_unix': time.time()}
            result['observations'].append(observed)
            self.save()
            downloaded = self.download(f'roundtrip-download-{index}', folder, before)
            observed['download'] = downloaded
            observed['reported_size_differences'] = sorted(
                n for n in before if n in remote and remote[n]['size'] != before[n][0])
            observed['timestamp_differences'] = sorted(
                n for n in before if n in remote and remote[n]['mtime'] != result['original_metadata'][n]['mtime'])
            observed['checksum_differences'] = sorted(
                n for n in before if n in remote and remote[n]['quickxor'] != result['original_metadata'][n]['quickxor'])
            observed['reported_vs_download_size_differences'] = sorted(
                n for n in downloaded['files'] if n in remote and remote[n]['size'] != downloaded['files'][n][0])
            observed['reported_vs_download_checksum_differences'] = sorted(
                n for n in downloaded['metadata'] if n in remote and
                remote[n]['quickxor'] != downloaded['metadata'][n]['quickxor'])
            observed['missing_metadata'] = sorted(n for n in before if n not in remote or
                not remote[n]['quickxor'] or not result['original_metadata'][n]['quickxor'] or
                type(remote[n]['size']) is not int or not remote[n]['mtime'])
            if downloaded['byte_differences'] or observed['missing_metadata']:
                self.data['issues'].append('round-trip byte rewrite, missing file or missing metadata; investigate')
            if observed['reported_vs_download_checksum_differences']:
                self.data['issues'].append('remote checksum disagrees with downloaded bytes; investigate')
            self.save()
        result['metadata_stable'] = result['observations'][0]['remote'] == result['observations'][1]['remote']
        if not result['metadata_stable']:
            self.data['issues'].append('round-trip remote metadata changed during observation window')

    def profile(self, name, flags, baseline):
        folder = self.folder(name)
        local, workdir = self.work / f'local-{name}', self.work / f'workdir-{name}'
        shutil.copytree(baseline, local)
        shutil.copy2(self.work / folder['marker'], local / folder['marker'])
        workdir.mkdir()
        result = {'profile': name, 'flags': flags, 'location': folder['location'],
                  'before_initial': core.local_snapshot(local), 'stages': [], 'issues': []}
        self.data['profiles'].append(result)
        self.save()

        def sync(stage, resync=False):
            command = ['rclone', 'bisync', str(local), folder['location'], '--workdir', str(workdir), *flags]
            if resync:
                command.append('--resync')
            transfer = self.transfer(f'{name}-{stage}', command)
            state = self.snapshot(local, folder['location'])
            record = {'stage': stage, 'transfer': transfer, 'snapshot': state}
            result['stages'].append(record)
            self.save()
            return record

        def quiet(stage, previous):
            self.wait()
            current = sync(stage)
            prior, state = previous['snapshot'], current['snapshot']
            current['stable'] = prior['local'] == state['local'] and prior['remote'] == state['remote']
            stats = current['transfer']['stats']
            current['no_transfers'] = stats['bytes'] == 0 and stats['transfers'] == 0
            if not current['stable'] or not current['no_transfers']:
                result['issues'].append(f'{stage}: drift or unnecessary transfers')
            return current

        first = sync('initial', True)
        result['initial_byte_changes'] = changes(content(result['before_initial']), content(first['snapshot']['local']))
        if result['initial_byte_changes']:
            result['issues'].append('initial sync changed local bytes')
        if name == 'compat':
            # The compatibility bundle includes one explicit convergence pass.
            # Its transfers remain visible; only later quiet stages must be idle.
            first = sync('convergence')
        second = quiet('no-edit-1', first)
        last = quiet('no-edit-2', second)
        # Independent byte verification is mandatory even when sync ignores size/hash.
        result['before_edit_download'] = self.download(f'{name}-verification', folder, last['snapshot']['local'])
        if result['before_edit_download']['byte_differences']:
            result['issues'].append('remote downloaded bytes differ from local mirror')
        if self.args.web_edit:
            print(f"\nOpen {folder['location'].split(':', 1)[1]}/ on the drive in the web editor, "
                  'change and save one Office file, then press Enter.', file=sys.stderr, flush=True)
            input()
            edited = sync('web-edit')
            result['edited_files'] = changes(content(last['snapshot']['local']), content(edited['snapshot']['local']))
            if not result['edited_files']:
                result['issues'].append('web edit was not observed in local bytes')
            settled = quiet('post-edit', edited)
            result['after_edit_download'] = self.download(f'{name}-post-edit-verification', folder,
                                                         settled['snapshot']['local'])
            if result['after_edit_download']['byte_differences']:
                result['issues'].append('post-edit downloaded bytes differ from local mirror')
        result['clean'] = not result['issues']
        self.save()
        return result

    def cleanup(self):
        for folder in self.data['folders']:
            if self.args.keep:
                folder['cleanup'] = 'kept by request'
            else:
                try:
                    # Check marker CONTENT, not merely presence; fail closed on partial setup.
                    value = self.report.capture('rclone', 'cat', folder['location'] + '/' + folder['marker'],
                                                timeout=60)
                    if value != folder['token']:
                        raise ValueError('Ownership marker mismatch')
                    self.report.run(['rclone', 'purge', folder['location']], timeout=60)
                    folder['cleanup'] = 'purged'
                except (Exception, KeyboardInterrupt) as error:
                    folder['cleanup'] = 'not purged: ' + type(error).__name__
                    self.data['issues'].append('cleanup incomplete; retain folder identity for operator')
            self.save()

    def execute(self):
        source = Path(self.args.files_dir).resolve() if self.args.files_dir else None
        source_before = None
        try:
            if source:
                validate_source(source)
                source_before = core.local_snapshot(source)
            self.data['source_before'] = source_before
            sha = self.report.capture('git', 'rev-parse', 'HEAD', allowed=(0, 128))
            dirty = self.report.capture('git', 'status', '--porcelain', allowed=(0, 128))
            self.report.update(source_sha=sha if len(sha) == 40 else None,
                               source_dirty=bool(dirty) if len(sha) == 40 else None)
            self.data['source_sha'] = self.report.state['source_sha']
            self.data['source_dirty'] = self.report.state['source_dirty']
            self.data['implementation_sha256'] = {
                p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                for p in (Path(__file__), Path(core.__file__))}
            self.data['rclone_version'] = version_evidence(self.report)
            baseline = self.work / 'baseline'
            self.data['synthetic_office'] = core.seed_local(baseline, source)
            self.data['baseline'] = core.local_snapshot(baseline)
            self.save()
            self.roundtrip(baseline)
            generic = self.profile('generic', core.GENERIC, baseline)
            if not generic['clean']:
                candidate = self.profile('compat', core.COMPAT, baseline)
                # A bundle passing does not establish that every workaround is needed.
                self.data['issues'].append('generic failed; compatibility candidate requires settings minimization'
                                           if candidate['clean'] else 'neither profile passed')
            self.data['status'] = 'completed'
        except BaseException as error:
            self.data['status'] = 'interrupted'
            self.data['error_type'] = type(error).__name__
            raise
        finally:
            try:
                self.cleanup()
            finally:
                if source and source_before is not None:
                    try:
                        self.data['source_after'] = core.local_snapshot(source)
                        self.data['source_unchanged'] = source_before == self.data['source_after']
                    except OSError:
                        self.data['source_unchanged'] = False
                    if not self.data['source_unchanged']:
                        self.data['issues'].append('source samples changed during experiment')
                complete = self.data['status'] == 'completed'
                self.data['verdict'] = ('generic passed round-trip and measured sync stages' if complete and
                    not self.data['issues'] else 'inconclusive; inspect recorded issues and partial evidence')
                self.save()
        print(f"verdict ({self.args.drive}): {self.data['verdict']}\nresults: {self.path}", file=sys.stderr)
        if self.data['issues']:
            raise ValueError('Round-trip investigation is inconclusive; see results.json')


def run(report, args):
    Experiment(report, args).execute()
