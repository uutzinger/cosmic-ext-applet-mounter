"""Prototype driver boundaries: fake cargo/git, no remote or real credentials."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / 'scripts/lib/onedrive_prototype_tasks.py'


class PrototypeDriver(unittest.TestCase):
    def test_fixed_action_receipt_and_account_not_logged(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / 'bin'
            binary.mkdir()
            git = binary / 'git'
            git.write_text('#!/usr/bin/env python3\nimport sys\nif sys.argv[1] == "rev-parse": print("a" * 40)\n')
            git.chmod(0o755)
            cargo = binary / 'cargo'
            cargo.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
args=sys.argv[1:]
pathlib.Path(os.environ['FAKE_CALLS']).write_text(json.dumps(args))
report=pathlib.Path(args[args.index('--report-dir')+1])
(report/'prototype.json').write_text(json.dumps({'status':'succeeded'}))
print(json.dumps({'status':'succeeded'}))
''')
            cargo.chmod(0o755)
            env = dict(os.environ, PATH=f'{binary}:{os.environ["PATH"]}', FAKE_CALLS=str(root / 'calls'))
            report = root / 'report'
            result = subprocess.run([sys.executable, str(TOOL), '--action', 'inspect', '--remote',
                'probe', '--drive', 'personal', '--expected-account', 'private@example.invalid',
                '--log-dir', str(report)], cwd=root, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            data = json.loads((report / 'summary.json').read_text())
            self.assertEqual(data['status'], 'succeeded')
            self.assertEqual(data['operation'], 'inspect')
            self.assertNotIn('private@example.invalid', (report / 'task.log').read_text())
            self.assertNotIn('private@example.invalid', (report / 'summary.json').read_text())
            args = json.loads((root / 'calls').read_text())
            self.assertEqual(args[:8], ['run', '--quiet', '--locked', '--features', 'development-probes', '--example', 'onedrive_mirror_prototype', '--'])
            self.assertTrue((report / 'prototype.json').is_file())
            verify_report = root / 'verify-report'
            session = root / 'session'
            result = subprocess.run([sys.executable, str(TOOL), '--action', 'verify', '--remote',
                'probe', '--drive', 'personal', '--expected-account', 'private@example.invalid',
                '--session', str(session), '--log-dir', str(verify_report)],
                cwd=root, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            args = json.loads((root / 'calls').read_text())
            self.assertEqual(args[args.index('--action')+1], 'verify')
            self.assertEqual(args[args.index('--session')+1], str(session))
            self.assertEqual(json.loads((verify_report / 'summary.json').read_text())['operation'], 'verify')
            samples = root / 'samples'
            samples.mkdir()
            for action, extra in [('prepare', ['--files-dir', str(samples)]), ('timing', ['--session', str(session)])]:
                result = subprocess.run([sys.executable, str(TOOL), '--action', action, '--remote',
                    'probe', '--drive', 'personal', '--expected-account', 'private@example.invalid',
                    '--log-dir', str(root / f'{action}-report'), *extra],
                    cwd=root, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                args = json.loads((root / 'calls').read_text())
                self.assertEqual(args[args.index('--action')+1], action)
                self.assertEqual(args[args.index(extra[0])+1], extra[1])

    def test_invalid_action_and_session_fail_before_starting_task(self):
        cases = [('--action', 'unknown'), ('--action', 'cleanup'),
                 ('--action', 'verify'), ('--action', 'verify', '--session', 'relative'),
                 ('--action', 'cleanup', '--session', 'relative'),
                 ('--action', 'inspect', '--session', '/tmp/session')]
        cases += [('--action', 'timing'), ('--action', 'prepare', '--files-dir', 'relative'),
                  ('--action', 'prepare', '--files-dir', '/nonexistent-synthetic-samples'),
                  ('--action', 'inspect', '--files-dir', '/tmp')]
        with tempfile.TemporaryDirectory() as tmp:
            for extra in cases:
                result = subprocess.run([sys.executable, str(TOOL), '--remote', 'probe', '--drive',
                    'personal', '--expected-account', 'person@example.invalid', *extra],
                    cwd=tmp, capture_output=True, text=True)
                self.assertEqual(result.returncode, 2, result.stderr)
            self.assertFalse((Path(tmp) / 'target').exists())


if __name__ == '__main__':
    unittest.main()
