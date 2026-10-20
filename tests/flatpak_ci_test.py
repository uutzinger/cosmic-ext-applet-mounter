#!/usr/bin/env python3
"""Offline hosted-build collection/runner policy tests; no CI or package changes."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts/lib'))
import flatpak_ci as ci
from artifact_inspection import digest
from flatpak_inspection import expected_policy


class FlatpakCITests(unittest.TestCase):
    def setUp(self):
        self.sha = 'a' * 40
        self.environment = {'GITHUB_ACTIONS': 'true', 'GITHUB_EVENT_NAME': 'workflow_dispatch',
                            'GITHUB_SHA': self.sha, 'GITHUB_REPOSITORY': 'example/project',
                            'GITHUB_RUN_ID': '123', 'GITHUB_RUN_ATTEMPT': '1',
                            'GITHUB_WORKFLOW_REF': 'example/project/.github/workflows/build-flatpak.yml@refs/heads/master'}

    def capture_responses(self):
        return [str(ROOT), self.sha, '', 'eu-strip fixture', 'eu-elfcompress fixture',
                *[(ROOT / path).read_text() for path in
                  ('resources/app.metainfo.xml', 'resources/app.desktop', 'resources/icon.svg')],
                'AppStream fixture', 'appstream 1.0.2\nlibrsvg2-common fixture']

    def fixture(self, work, arch):
        candidate, producer = work / 'candidate', work / 'producer'
        candidate.mkdir(parents=True)
        producer.mkdir()
        bundle = candidate / f'{ci.APP_ID}-{arch}-master.flatpak'
        bundle.write_bytes(b'fake-bundle')
        metadata = candidate / 'built-metadata.ini'
        metadata.write_text(f'[Application]\nname={ci.APP_ID}\nruntime=org.freedesktop.Platform/{arch}/25.08\nsdk=org.freedesktop.Sdk/{arch}/25.08\n')
        inspection = {'ref': f'app/{ci.APP_ID}/{arch}/master', 'commit': 'b' * 64,
                      'metadata': expected_policy(metadata.read_text(), ci.APP_ID, arch)}
        receipt = {'schema_version': 2, 'kind': 'unaccepted-flatpak-candidate',
                   'source_sha': self.sha, 'architecture': arch, 'application_id': ci.APP_ID,
                   'branch': 'master', 'export_inspection': inspection,
                   'bundle_inspection': {**inspection, 'bundle_sha256': digest(bundle)},
                   'source_pins': [{'type': 'git', 'commit': 'c' * 40}],
                   'runtimes': [{'ref': f'runtime/org.freedesktop.Platform/{arch}/25.08', 'commit': 'd' * 64}],
                   'artifacts': [{'path': str(p), 'sha256': digest(p)} for p in (bundle, metadata)]}
        summary = {'task': 'flatpak-build', 'status': 'succeeded', 'exit_code': 0,
                   'phase': 'candidate-exported', 'source_sha': self.sha, 'source_dirty': False,
                   'artifact_receipt_path': str(producer / 'artifacts.json')}
        (producer / 'artifacts.json').write_text(json.dumps(receipt))
        (producer / 'summary.json').write_text(json.dumps(summary))
        return receipt, summary

    def test_runner_identity_rejects_wrong_or_nonmanual_context(self):
        for arch in ('x86_64', 'aarch64'):
            self.assertEqual(ci.runner_context(self.sha, arch, self.environment, arch), ('example/project', '123', '1'))
        for key, value in (('GITHUB_ACTIONS', 'false'), ('GITHUB_EVENT_NAME', 'pull_request'),
                           ('GITHUB_SHA', 'e' * 40), ('GITHUB_WORKFLOW_REF', 'other/build.yml@master'),
                           ('GITHUB_RUN_ID', 'bad')):
            with self.subTest(key=key), self.assertRaises(ValueError):
                ci.runner_context(self.sha, 'x86_64', {**self.environment, key: value}, 'x86_64')
        with self.assertRaises(ValueError):
            ci.runner_context(self.sha, 'aarch64', self.environment, 'x86_64')

    def test_both_architectures_collect_portable_checksums_and_provenance(self):
        for arch in ('x86_64', 'aarch64'):
            with self.subTest(arch=arch), tempfile.TemporaryDirectory() as directory:
                work = Path(directory)
                self.fixture(work, arch)
                index = ci.collect_artifacts(work, self.sha, arch, 'example/project', '123', '1')
                data = json.loads(index.read_text())
                self.assertEqual(data['acceptance'], 'not-run')
                self.assertEqual(data['source_sha'], self.sha)
                self.assertEqual(data['run_id'], '123')
                self.assertEqual(data['architecture'], arch)
                self.assertEqual(len(data['artifacts']), 4)
                for item in data['artifacts']:
                    self.assertEqual(digest(index.parent / item['name']), item['sha256'])
                for line in (index.parent / 'SHA256SUMS').read_text().splitlines():
                    checksum, name = line.split('  ')
                    self.assertEqual(digest(index.parent / name), checksum)

    def test_mismatch_or_partial_build_never_creates_upload(self):
        for failure in ('status', 'source_sha', 'architecture', 'commit', 'metadata', 'bundle'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                work = Path(directory)
                receipt, summary = self.fixture(work, 'x86_64')
                if failure == 'status':
                    summary['status'] = 'failed'
                elif failure in ('source_sha', 'architecture'):
                    receipt[failure] = 'wrong'
                elif failure == 'commit':
                    receipt['bundle_inspection']['commit'] = 'e' * 64
                elif failure == 'metadata':
                    receipt['bundle_inspection']['metadata'] = {}
                else:
                    (work / 'candidate' / f'{ci.APP_ID}-x86_64-master.flatpak').write_bytes(b'corrupted')
                (work / 'producer/artifacts.json').write_text(json.dumps(receipt))
                (work / 'producer/summary.json').write_text(json.dumps(summary))
                with self.assertRaises(ValueError):
                    ci.collect_artifacts(work, self.sha, 'x86_64', 'example/project', '123', '1')
                self.assertFalse((work / 'upload').exists())

    def test_dirty_checkout_stops_before_runner_changes(self):
        report = Mock()
        report.capture.side_effect = [str(ROOT), self.sha, ' M README.md']
        args = Mock(source_sha=self.sha, arch='x86_64')
        with patch.dict(ci.os.environ, self.environment, clear=True), patch.object(ci.platform, 'machine', return_value='x86_64'):
            with self.assertRaises(ValueError):
                ci.operation(args, report)
        report.run.assert_not_called()

    def test_build_failure_stops_before_collection(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Mock(directory=Path(directory) / 'logs')
            report.capture.side_effect = self.capture_responses()
            def fail_build(argv, **kwargs):
                if 'scripts/build-flatpak.sh' in argv:
                    raise RuntimeError('fake builder failure')
            report.run.side_effect = fail_build
            args = Mock(source_sha=self.sha, arch='x86_64', work_dir=Path(directory) / 'work')
            with patch.dict(ci.os.environ, self.environment, clear=True), patch.object(ci.platform, 'machine', return_value='x86_64'):
                with self.assertRaisesRegex(RuntimeError, 'fake builder'):
                    ci.operation(args, report)
            self.assertFalse((args.work_dir / 'upload').exists())
            commands = [call.args[0] for call in report.run.call_args_list]
            self.assertTrue(any('--arch=x86_64' in cmd and 'flatpak' in cmd for cmd in commands))
            packages = next(cmd for cmd in commands if cmd[:3] == ['sudo', 'apt-get', 'install'])
            self.assertIn('elfutils', packages)
            self.assertIn('appstream-compose', packages)
            self.assertIn('librsvg2-common', packages)
            self.assertIn(('eu-strip', '--version'), [call.args for call in report.capture.call_args_list])
            self.assertIn(('eu-elfcompress', '--version'), [call.args for call in report.capture.call_args_list])
            self.assertFalse(any('release' in cmd or 'publish' in cmd for cmd in commands))

    def test_missing_elfutils_tool_stops_before_runtime_download_and_build(self):
        for tool in ('eu-strip', 'eu-elfcompress'):
            with self.subTest(tool=tool), tempfile.TemporaryDirectory() as directory:
                report = Mock(directory=Path(directory) / 'logs')
                responses = [str(ROOT), self.sha, '']
                if tool == 'eu-elfcompress':
                    responses.append('eu-strip fixture')
                report.capture.side_effect = [*responses, subprocess.CalledProcessError(127, [tool, '--version'])]
                args = Mock(source_sha=self.sha, arch='x86_64', work_dir=Path(directory) / 'work')
                with patch.dict(ci.os.environ, self.environment, clear=True), patch.object(ci.platform, 'machine', return_value='x86_64'):
                    with self.assertRaises(subprocess.CalledProcessError):
                        ci.operation(args, report)
                commands = [call.args[0] for call in report.run.call_args_list]
                self.assertFalse(any('flatpak' == cmd[0] or 'scripts/build-flatpak.sh' in cmd for cmd in commands))

    def test_appstream_failure_stops_before_runtime_install_and_build(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Mock(directory=Path(directory) / 'logs')
            report.capture.side_effect = self.capture_responses()
            def fail_compose(argv, **kwargs):
                if argv[:2] == ['appstreamcli', 'compose']:
                    raise subprocess.CalledProcessError(1, argv)
            report.run.side_effect = fail_compose
            args = Mock(source_sha=self.sha, arch='x86_64', work_dir=Path(directory) / 'work')
            with patch.dict(ci.os.environ, self.environment, clear=True), patch.object(ci.platform, 'machine', return_value='x86_64'):
                with self.assertRaises(subprocess.CalledProcessError):
                    ci.operation(args, report)
            commands = [call.args[0] for call in report.run.call_args_list]
            compose = next(cmd for cmd in commands if cmd[:2] == ['appstreamcli', 'compose'])
            self.assertIn('--no-net', compose)
            self.assertIn('--print-report=full', compose)
            self.assertFalse(any('flatpak' == cmd[0] or 'scripts/build-flatpak.sh' in cmd for cmd in commands))
            data = json.loads((ROOT / ci.MANIFEST).read_text())
            commit = data['modules'][0]['sources'][0]['commit']
            git_shows = [call.args[-1] for call in report.capture.call_args_list if 'show' in call.args]
            self.assertTrue(all(value.startswith(commit + ':resources/') for value in git_shows))
            self.assertEqual(len(git_shows), 3)

    def test_appstream_rejects_unpinned_source_before_reading_resources(self):
        data = {'modules': [{'name': 'cosmic-ext-applet-mounter', 'sources': [{
            'type': 'git', 'url': 'https://github.com/uutzinger/cosmic-ext-applet-mounter.git', 'commit': 'master'}]}]}
        report = Mock()
        with self.assertRaises(ValueError):
            ci.check_appstream(ROOT, data, Path('/unused'), report)
        report.capture.assert_not_called()


if __name__ == '__main__':
    unittest.main()
