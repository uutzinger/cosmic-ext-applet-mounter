#!/usr/bin/env python3
"""Flatpak release policy tests with synthetic files/fake GitHub and OSTree adapters."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts/lib'))
import flatpak_promotion as fp
from artifact_inspection import digest
from flatpak_inspection import expected_policy
from task_output import TaskReport

spec = importlib.util.spec_from_file_location('flatpak_release_driver', ROOT / 'scripts/promote_release.py')
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)


class FlatpakPromotionTests(unittest.TestCase):
    def setUp(self):
        self.sha, self.repo = 'a' * 40, 'example/project'
        self.inputs = {f'packaging/flatpak/{fp.APP_ID}.json': 'e' * 64}
        self.pins = [{'type': 'git', 'commit': 'c' * 40}]
        self.manifest = {'runtime': 'org.freedesktop.Platform', 'sdk': 'org.freedesktop.Sdk',
                         'runtime-version': '25.08', 'finish-args': ['--socket=wayland'],
                         'modules': [{'name': 'cosmic-ext-applet-mounter',
                                      'sources': [{'type': 'git', 'commit': 'c' * 40}]}]}

    def context(self, arch='x86_64', sha=None, run_id='123'):
        run = {'id': int(run_id), 'head_sha': sha or self.sha, 'run_attempt': 1,
               'repository': {'full_name': self.repo}, 'head_repository': {'full_name': self.repo},
               'path': fp.WORKFLOW_PATH, 'event': 'workflow_dispatch', 'status': 'completed', 'conclusion': 'success'}
        artifacts = {'total_count': 2, 'artifacts': [
            {'name': f'flatpak-{arch}', 'expired': False}, {'name': f'flatpak-receipts-{arch}', 'expired': False}]}
        jobs = {'total_count': 1, 'jobs': [{'name': f'Flatpak {arch}', 'status': 'completed', 'conclusion': 'success'}]}
        return run, artifacts, jobs

    def fixture(self, folder, arch='x86_64', sha=None, run_id='123'):
        sha = sha or self.sha
        folder.mkdir(parents=True)
        name = f'{fp.APP_ID}-{arch}-master.flatpak'
        (folder / name).write_bytes(b'synthetic-flatpak')
        text = f'[Application]\nname={fp.APP_ID}\nruntime=org.freedesktop.Platform/{arch}/25.08\nsdk=org.freedesktop.Sdk/{arch}/25.08\n[Context]\nsockets=wayland;\n'
        (folder / 'built-metadata.ini').write_text(text)
        ref = f'app/{fp.APP_ID}/{arch}/master'
        inspection = {'ref': ref, 'commit': 'b' * 64, 'metadata': expected_policy(text, fp.APP_ID, arch)}
        runtimes = [{'ref': f'runtime/{name}/{arch}/25.08', 'commit': 'd' * 64}
                    for name in ('org.freedesktop.Platform', 'org.freedesktop.Sdk')]
        receipt = {'schema_version': 2, 'kind': 'unaccepted-flatpak-candidate', 'source_sha': sha,
                   'architecture': arch, 'application_id': fp.APP_ID, 'branch': 'master',
                   'manifest_relative_path': f'packaging/flatpak/{fp.APP_ID}.json',
                   'input_sha256': self.inputs, 'source_pins': self.pins, 'runtimes': runtimes,
                   'finish_args_declared': self.manifest['finish-args'], 'sdk_extensions_declared': [],
                   'build_options': {'disable_updates': True, 'offline': False}, 'source_transport': {'kind': 'manifest-url'},
                   'tools': {tool: 'fixture' for tool in ('flatpak', 'flatpak-builder', 'ostree')},
                   'artifacts': [{'path': '/runner/' + n, 'sha256': digest(folder / n)} for n in (name, 'built-metadata.ini')],
                   'export_inspection': inspection,
                   'bundle_inspection': {**inspection, 'bundle_sha256': digest(folder / name)}}
        summary = {'schema_version': 1, 'task': 'flatpak-build', 'status': 'succeeded', 'exit_code': 0,
                   'phase': 'candidate-exported', 'source_sha': sha, 'source_dirty': None}
        index = {'schema_version': 1, 'kind': 'unaccepted-hosted-flatpak', 'repository': self.repo,
                 'workflow': fp.WORKFLOW_PATH, 'source_sha': sha, 'run_id': run_id, 'run_attempt': '1',
                 'architecture': arch, 'application_id': fp.APP_ID, 'ref': ref, 'commit': 'b' * 64,
                 'source_pins': self.pins, 'runtimes': runtimes}
        for filename, data in [('producer-artifacts.json', receipt), ('producer-summary.json', summary), ('hosted-artifacts.json', index)]:
            (folder / filename).write_text(json.dumps(data))
        self.rehash(folder, name)

    def rehash(self, folder, name):
        index_file = folder / 'hosted-artifacts.json'
        index = json.loads(index_file.read_text())
        names = [name, 'built-metadata.ini', 'producer-artifacts.json', 'producer-summary.json']
        index['artifacts'] = [{'name': n, 'sha256': digest(folder / n)} for n in names]
        index_file.write_text(json.dumps(index))
        (folder / 'SHA256SUMS').write_text(''.join(f'{digest(folder / n)}  {n}\n' for n in [*names, index_file.name]))

    def verify(self, folder, arch='x86_64'):
        return fp.verify_download(folder, arch, self.repo, self.sha, '123', 1,
                                  self.inputs, self.pins, self.manifest, Mock(), folder.parent)

    def test_successful_selected_job_in_failed_matrix_is_usable_but_other_failures_rejected(self):
        run, artifacts, jobs = self.context()
        run['conclusion'] = 'failure'
        self.assertEqual(fp.validate_hosted_run(run, artifacts, jobs, self.repo, self.sha, 'x86_64'), 1)
        for key, value in [('head_sha', 'f' * 40), ('event', 'pull_request'), ('conclusion', 'cancelled'),
                           ('path', '.github/workflows/other.yml'), ('head_repository', {'full_name': 'fork/project'})]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                fp.validate_hosted_run({**run, key: value}, artifacts, jobs, self.repo, self.sha, 'x86_64')
        for kind in ('expired', 'duplicate', 'failed-job', 'incomplete'):
            a, j = copy.deepcopy(artifacts), copy.deepcopy(jobs)
            if kind == 'expired': a['artifacts'][0]['expired'] = True
            elif kind == 'duplicate':
                a['artifacts'].append(a['artifacts'][0]); a['total_count'] += 1
            elif kind == 'failed-job': j['jobs'][0]['conclusion'] = 'failure'
            else: a['total_count'] += 1
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                fp.validate_hosted_run(run, a, j, self.repo, self.sha, 'x86_64')

    def test_both_architectures_verify_bytes_before_offline_bundle_check(self):
        for arch in fp.ARCHITECTURES:
            with self.subTest(arch=arch), tempfile.TemporaryDirectory() as tmp:
                folder = Path(tmp) / 'artifact'
                self.fixture(folder, arch)
                with patch.object(fp, 'inspect_bundle') as inspect:
                    bundle, evidence = self.verify(folder, arch)
                self.assertEqual(evidence['sha256'], digest(bundle))
                self.assertEqual(evidence['build_source_sha'], self.sha)
                self.assertNotIn('path', evidence)
                inspect.assert_called_once()

    def test_identity_input_and_producer_mismatches_rejected_even_with_recomputed_checksums(self):
        cases = [('hosted-artifacts.json', 'source_sha', 'f' * 40),
                 ('hosted-artifacts.json', 'run_attempt', '2'),
                 ('hosted-artifacts.json', 'repository', 'fork/project'),
                 ('producer-artifacts.json', 'input_sha256', {}),
                 ('producer-artifacts.json', 'source_pins', []),
                 ('producer-artifacts.json', 'finish_args_declared', ['--filesystem=host']),
                 ('producer-summary.json', 'status', 'failed')]
        for filename, key, value in cases:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                folder = Path(tmp) / 'artifact'
                self.fixture(folder)
                data = json.loads((folder / filename).read_text()); data[key] = value
                (folder / filename).write_text(json.dumps(data))
                self.rehash(folder, f'{fp.APP_ID}-x86_64-master.flatpak')
                with patch.object(fp, 'inspect_bundle') as inspect, self.assertRaises(ValueError):
                    self.verify(folder)
                inspect.assert_not_called()

    def test_corruption_extra_symlinks_and_unsafe_checksums_rejected(self):
        for kind in ('corruption', 'extra', 'symlink', 'unsafe', 'duplicate'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                folder = Path(tmp) / 'artifact'; self.fixture(folder)
                bundle = folder / f'{fp.APP_ID}-x86_64-master.flatpak'
                if kind == 'corruption': bundle.write_bytes(b'corrupted')
                elif kind == 'extra': (folder / 'extra').touch()
                elif kind == 'symlink': bundle.unlink(); bundle.symlink_to('/dev/null')
                elif kind == 'unsafe': (folder / 'SHA256SUMS').write_text('0' * 64 + '  ../../outside\n')
                else:
                    path = folder / 'SHA256SUMS'; path.write_text(path.read_text() + path.read_text().splitlines()[0] + '\n')
                with patch.object(fp, 'inspect_bundle') as inspect, self.assertRaises(ValueError):
                    self.verify(folder)
                inspect.assert_not_called()

    def test_selections_and_cli_reject_unapproved_or_ambiguous_modes(self):
        self.assertEqual(fp.selections(['x86_64=123', 'aarch64=456']), {'x86_64': '123', 'aarch64': '456'})
        for values in (['riscv64=123'], ['x86_64=123', 'x86_64=456'], ['aarch64=../bad']):
            with self.assertRaises(ValueError): fp.selections(values)
        for arguments in (['v0.5.4', '--inspect-flatpaks'],
                          ['v0.5.4', '--inspect-flatpaks', '--flatpak-run', 'x86_64=123', '--publish', '--accepted'],
                          ['v0.5.4', '123', '--check-run-only', '--flatpak-run', 'x86_64=123']):
            result = subprocess.run(['python3', str(ROOT / 'scripts/promote_release.py'), *arguments], capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 2)

    def test_candidate_inspection_can_compare_different_commits_without_weakening_exact_tag_guard(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures, contexts = {}, {}
            for arch, sha, run in [('x86_64', self.sha, '123'), ('aarch64', 'f' * 40, '456')]:
                fixtures[run] = root / ('source-' + arch)
                self.fixture(fixtures[run], arch, sha, run)
                contexts[run] = self.context(arch, sha, run)
            def capture(*argv):
                if argv[:3] == ('git', 'rev-parse', '--show-toplevel'): return str(ROOT)
                if argv[0] == 'git': return '[package]\nname="cosmic-ext-applet-mounter"\nversion="0.5.4"\n'
                path = argv[-1]
                run = path.split('/actions/runs/')[1].split('/')[0]
                index = 1 if '/artifacts?' in path else 2 if '/jobs?' in path else 0
                return json.dumps(contexts[run][index])
            def command(argv, **kwargs):
                self.assertEqual(argv[:3], ['gh', 'run', 'download'])
                shutil.copytree(fixtures[argv[3]], Path(argv[-1]))
            report = Mock(directory=root, capture=Mock(side_effect=capture), run=Mock(side_effect=command))
            args = Mock(tag='v0.5.4', flatpak_run=['x86_64=123', 'aarch64=456'])
            for expected, target in [(None, root / 'inspection'), (self.sha, root / 'strict')]:
                with patch.object(driver, 'inspect_inputs', return_value=(self.manifest, self.inputs, self.pins)), patch.object(fp, 'inspect_bundle'):
                    if expected:
                        with self.assertRaisesRegex(ValueError, 'expected source'):
                            driver.flatpak_downloads(args, report, self.repo, target, expected)
                    else:
                        bundles, evidence = driver.flatpak_downloads(args, report, self.repo, target)
                        self.assertEqual(len(bundles), 2)
                        self.assertEqual({item['build_source_sha'] for item in evidence}, {self.sha, 'f' * 40})

    def test_mixed_publication_aggregates_eight_packages_and_provenance_without_building(self):
        artifacts = [{'name': f'deb-ubuntu-{v}-{a}', 'expired': False} for v, a in driver.MATRIX]
        run = {'repository': {'full_name': self.repo}, 'head_repository': {'full_name': self.repo},
               'head_sha': self.sha, 'path': '.github/workflows/build-deb.yml',
               'event': 'workflow_dispatch', 'status': 'completed', 'conclusion': 'success'}
        commands = []
        def capture(*argv):
            if argv[:2] == ('git', 'status'): return ''
            if argv[0] == 'git': return self.sha
            if argv[0] == 'dpkg-deb':
                return {'Package': 'cosmic-ext-applet-mounter', 'Version': '0.5.4',
                        'Architecture': 'arm64' if 'arm64' in argv[-2] else 'amd64'}[argv[-1]]
            if '/git/ref/tags/' in argv[-1]: return json.dumps({'object': {'type': 'commit', 'sha': self.sha}})
            return json.dumps({'total_count': 6, 'artifacts': artifacts} if '/artifacts?' in argv[-1] else run)
        def command(argv, **kwargs):
            commands.append(argv)
            if argv[:3] == ['gh', 'run', 'download']:
                folder = Path(argv[-1]); folder.mkdir()
                v, a = folder.name.removeprefix('deb-ubuntu-').rsplit('-', 1)
                p = folder / f'cosmic-ext-applet-mounter_0.5.4_ubuntu{v}_{a}.deb'; p.write_bytes(b'fake-deb')
                (folder / 'SHA256SUMS').write_text(f'{digest(p)}  {p.name}\n')
            if argv[:3] == ['gh', 'release', 'create']:
                self.assertEqual(len([p for p in argv if p.endswith('.deb')]), 6)
                self.assertEqual(len([p for p in argv if p.endswith('.flatpak')]), 2)
                sums = next(Path(p) for p in argv if p.endswith('/SHA256SUMS'))
                self.assertEqual(len(sums.read_text().splitlines()), 9)
                self.assertTrue(any(p.endswith('/FLATPAK-PROVENANCE.json') for p in argv))
        def flatpaks(args, report, repo, target, expected_sha):
            self.assertEqual(expected_sha, self.sha)
            bundles = [target / f'{fp.APP_ID}-{a}-master.flatpak' for a in fp.ARCHITECTURES]
            for bundle in bundles: bundle.write_bytes(b'fake-bundle')
            return bundles, [{'architecture': a, 'build_source_sha': self.sha,
                              'bundle': bundle.name, 'run_url': f'https://github.com/{self.repo}/actions/runs/{run_id}'}
                             for a, bundle, run_id in zip(fp.ARCHITECTURES, bundles, ('456', '789'))]
        with tempfile.TemporaryDirectory() as tmp:
            argv = ['promote_release.py', 'v0.5.4', '123', '--repo', self.repo, '--publish', '--accepted',
                    '--flatpak-run', 'x86_64=456', '--flatpak-run', 'aarch64=789', '--log-dir', str(Path(tmp) / 'report')]
            with patch('sys.argv', argv), patch.object(TaskReport, 'capture', lambda _self, *a, **k: capture(*a)), \
                    patch.object(TaskReport, 'run', lambda _self, a, **k: command(a)), patch.object(driver, 'flatpak_downloads', side_effect=flatpaks):
                driver.main()
            receipt = json.loads((Path(tmp) / 'report/artifacts.json').read_text())
            self.assertEqual(len(receipt['artifacts']), 10)
            self.assertEqual(len(receipt['flatpaks']), 2)
            for item in receipt['flatpaks']:
                artifact = next(a for a in receipt['artifacts'] if a['name'] == item['bundle'])
                self.assertEqual(artifact['origin'], item['run_url'])
        self.assertEqual(len([c for c in commands if c[:3] == ['gh', 'release', 'create']]), 1)
        self.assertFalse(any('cargo' in c or 'workflow' in c or 'flatpak-builder' in c for c in commands))


if __name__ == '__main__':
    unittest.main()
