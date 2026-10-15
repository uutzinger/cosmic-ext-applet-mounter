"""New evidence mode against the shared fake; optional installed-rclone local smoke."""
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

import mirror_guard_check_test as fixtures

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts/lib'))
from mirror_guard_roundtrip import Experiment, counters, metadata, version_evidence
from task_output import TaskReport


class RoundTrip(unittest.TestCase):
    def run_tool(self, tmp, **env):
        return fixtures.GuardCheck().run_tool(tmp, '--remote', 'od', '--round-trip', '--settle-seconds', '0', **env)

    def test_stable_roundtrip_and_zero_no_edit_transfers(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, data = self.run_tool(tmp)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(data['schema_version'], 2)
            self.assertEqual(data['rclone_version'], {'format': 'text', 'output': 'rclone vFAKE'})
            self.assertEqual(len(data['roundtrip']['observations']), 2)
            self.assertEqual(len(data['folders']), 2)
            self.assertTrue(all(f['cleanup'] == 'purged' for f in data['folders']))
            for observation in data['roundtrip']['observations']:
                self.assertEqual(observation['download']['byte_differences'], [])
            stages = data['profiles'][0]['stages']
            self.assertGreater(stages[0]['transfer']['stats']['transfers'], 0)
            self.assertTrue(all(s['no_transfers'] for s in stages[1:]))
            self.assertEqual(len([c for c in calls if c[0] == 'bisync']), 3)

    def test_reported_size_difference_is_not_a_byte_rewrite(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, _, data = self.run_tool(tmp, FAKE_SIZE='1')
            self.assertEqual(result.returncode, 0, result.stderr)
            obs = data['roundtrip']['observations'][0]
            self.assertTrue(obs['reported_size_differences'])
            self.assertFalse(obs['download']['byte_differences'])

    def test_byte_rewrites_do_not_pass_even_if_sync_settles(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, _, data = self.run_tool(tmp, FAKE_REWRITE='1')
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue(data['roundtrip']['observations'][0]['download']['byte_differences'])
            self.assertIn('inconclusive', data['verdict'])

    def test_redundant_transfers_detected_despite_stable_snapshots(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, _, data = self.run_tool(tmp, FAKE_REDUNDANT='1')
            self.assertNotEqual(result.returncode, 0)
            stage = data['profiles'][0]['stages'][1]
            self.assertTrue(stage['stable'])
            self.assertFalse(stage['no_transfers'])

    def test_missing_evidence_never_passes(self):
        for key in ('FAKE_NO_STATS', 'FAKE_NO_HASH', 'FAKE_MISSING'):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                result, _, data = self.run_tool(tmp, **{key: '1'})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('inconclusive', data['verdict'])

    def test_failure_and_cancellation_preserve_partial_evidence_and_cleanup(self):
        for key in ('FAKE_COPY_FAIL', 'FAKE_CANCEL'):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                result, _, data = self.run_tool(tmp, **{key: '1'})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(data['status'], 'interrupted')
                self.assertEqual(data['folders'][0]['cleanup'], 'purged')
                self.assertTrue(data['commands'])

    def test_wrong_marker_does_not_purge(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, data = self.run_tool(tmp, FAKE_DROP_MARKER='1')
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse([c for c in calls if c[0] == 'purge'])
            self.assertTrue(all('not purged' in f['cleanup'] for f in data['folders']))

    def test_real_source_files_unchanged(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'source'
            source.mkdir()
            (source / 'plain.txt').write_bytes(b'unchanged\n')
            result, _, data = fixtures.GuardCheck().run_tool(tmp, '--remote', 'od', '--round-trip',
                '--settle-seconds', '0', '--files-dir', str(source))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(data['source_unchanged'])
            self.assertEqual(data['baseline']['plain.txt'][2], data['source_before']['plain.txt'][2])
            self.assertEqual((source / 'plain.txt').read_bytes(), b'unchanged\n')

    def test_web_edit_is_observed_and_settles(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, _, data = fixtures.GuardCheck().run_tool(tmp, '--remote', 'od', '--round-trip',
                '--settle-seconds', '0', '--web-edit', FAKE_WEB_EDIT='1')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(data['profiles'][0]['edited_files'])
            self.assertTrue(data['profiles'][0]['stages'][-1]['no_transfers'])
            self.assertEqual(data['profiles'][0]['after_edit_download']['byte_differences'], [])

    def test_unobserved_web_edit_is_inconclusive(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, _, data = fixtures.GuardCheck().run_tool(tmp, '--remote', 'od', '--round-trip',
                '--settle-seconds', '0', '--web-edit')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('web edit was not observed in local bytes', data['profiles'][0]['issues'])

    def test_invalid_sample_tree_has_no_cloud_writes(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'source'
            source.mkdir()
            (source / 'link').symlink_to('/etc/hosts')
            result, calls, data = fixtures.GuardCheck().run_tool(tmp, '--remote', 'od', '--round-trip',
                '--settle-seconds', '0', '--files-dir', str(source))
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse([c for c in calls if c[0] in ('mkdir', 'copy', 'copyto', 'bisync', 'purge')])
            self.assertEqual(data['status'], 'interrupted')

    def test_malformed_counters_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'stats'
            for record in ({}, {'stats': {'bytes': 0, 'transfers': True, 'errors': 0}},
                           {'stats': {'bytes': -1, 'transfers': 0, 'errors': 0}}):
                path.write_text(json.dumps(record) + '\n')
                with self.assertRaises(ValueError):
                    counters(path)

    @unittest.skipUnless(shutil.which('rclone'), 'installed rclone unavailable')
    def test_real_rclone_local_copy_stats(self):
        # No network, remote config, service or real user directory; validates actual JSON format.
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source, target = root / 'source', root / 'target'
            source.mkdir()
            (source / 'a.txt').write_text('sample')
            report = TaskReport('local-stats', str(root / 'log'))
            try:
                evidence = version_evidence(report)
                self.assertEqual(evidence['format'], 'text')
                self.assertTrue(evidence['output'].startswith('rclone v'))
                experiment = Experiment(report, SimpleNamespace(drive='personal', remote='unused',
                    settle_seconds=0, web_edit=False))
                args = ['rclone', 'copy', str(source), str(target), '--config', str(root / 'empty.conf')]
                first = experiment.transfer('first', args)
                second = experiment.transfer('second', args)
                self.assertEqual(first['stats']['transfers'], 1)
                self.assertEqual(second['stats']['transfers'], 0)
                self.assertEqual(second['stats']['bytes'], 0)
                self.assertTrue(metadata(report, source)['a.txt']['quickxor'])
                sync_args = ['rclone', 'bisync', str(source), str(target), '--workdir', str(root / 'bisync'),
                             '--config', str(root / 'empty.conf')]
                experiment.transfer('initial-sync', [*sync_args, '--resync'])
                final = experiment.transfer('no-edit-sync', sync_args)
                self.assertEqual(final['stats']['transfers'], 0)
                self.assertEqual(final['stats']['bytes'], 0)
            finally:
                report.log.close()


if __name__ == '__main__':
    unittest.main()
