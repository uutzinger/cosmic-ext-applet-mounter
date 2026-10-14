"""Logged just dispatch fixtures; never compile or operate on real user state."""

import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DevTests(unittest.TestCase):
    def setup_fixture(self, tmp):
        root = Path(tmp)
        (root / "scripts").mkdir()
        shutil.copy2(ROOT / "scripts/dev.sh", root / "scripts/dev.sh")
        shutil.copytree(ROOT / "scripts/lib", root / "scripts/lib",
                        ignore=shutil.ignore_patterns("__pycache__"))
        (root / "justfile").touch()
        tools = root / "bin"
        tools.mkdir()
        fake = tools / "fake"
        fake.write_text('''#!/usr/bin/env python3
import json,os,pathlib,sys,time
root=pathlib.Path(os.environ['FIXTURE_ROOT'])
name=pathlib.Path(sys.argv[0]).name
with (root/'calls').open('a') as log: log.write(json.dumps([name,*sys.argv[1:]])+'\\n')
if name=='git':
 print('a'*40 if sys.argv[1]=='rev-parse' else os.environ.get('DIRTY',''),end='')
else:
 print('private-stdout'*10000)
 print('private-stderr',file=sys.stderr)
 if os.environ.get('SLEEP'):
  (root/'pid').write_text(str(os.getpid()))
  time.sleep(60)
 sys.exit(int(os.environ.get('RESULT','0')))
''')
        fake.chmod(0o755)
        for name in ["git", "just"]:
            (tools / name).symlink_to(fake)
        env = dict(os.environ, PATH=f"{tools}:{os.environ['PATH']}", FIXTURE_ROOT=str(root))
        for key in ["DIRTY", "SLEEP", "RESULT"]:
            env.pop(key, None)
        return root, env

    def run_task(self, root, env, *args):
        return subprocess.run(["bash", str(root / "scripts/dev.sh"), *args],
                              cwd="/tmp", env=env, capture_output=True, text=True, timeout=10)

    def test_allowed_recipes_run_once_with_bounded_output_and_dirty_receipt(self):
        tasks = ["fmt-check", "localization-check", "check", "lint", "test", "verify",
                 "metadata-check", "metadata-check-strict", "build", "build-release"]
        for task in tasks:
            with self.subTest(task=task), tempfile.TemporaryDirectory() as tmp:
                root, env = self.setup_fixture(tmp)
                env["DIRTY"] = " M fixture"
                result = self.run_task(root, env, task)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertLess(len(result.stdout), 2500)
                self.assertNotIn("private-stdout", result.stdout)
                summary = json.loads(result.stdout.splitlines()[-1])
                receipt = json.loads(Path(summary["summary_path"]).read_text())
                self.assertTrue(receipt["source_dirty"])
                self.assertEqual(receipt["recipe"], task)
                self.assertIn("private-stderr", Path(summary["log_path"]).read_text())
                calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
                self.assertEqual([c for c in calls if c[0] == "just"],
                                 [["just", "--justfile", "justfile", task]])

    def test_invalid_tasks_and_arguments_do_not_run_commands(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env = self.setup_fixture(tmp)
            for args in [("install-user",), ("clean",), ("run",), ("check;touch bad",),
                         ("check", "--release"), ("check", "--timeout-seconds", "0")]:
                self.assertEqual(self.run_task(root, env, *args).returncode, 2)
            self.assertFalse((root / "calls").exists())

    def test_real_exit_status_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env = self.setup_fixture(tmp)
            env["RESULT"] = "37"
            result = self.run_task(root, env, "check")
            self.assertEqual(result.returncode, 37)
            self.assertEqual(json.loads(result.stdout.splitlines()[-1])["status"], "failed")

    def test_existing_log_directory_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env = self.setup_fixture(tmp)
            logs = root / "logs"
            logs.mkdir()
            self.assertNotEqual(self.run_task(root, env, "test", "--log-dir", str(logs)).returncode, 0)
            self.assertFalse((root / "calls").exists())

    def test_timeout_and_cancellation_reap_recipe(self):
        for cancelled in [False, True]:
            with self.subTest(cancelled=cancelled), tempfile.TemporaryDirectory() as tmp:
                root, env = self.setup_fixture(tmp)
                env["SLEEP"] = "1"
                process = subprocess.Popen(["bash", str(root / "scripts/dev.sh"), "check",
                                            "--timeout-seconds", "5" if cancelled else "1"],
                                           env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    deadline = time.monotonic() + 4
                    while not (root / "pid").exists() and time.monotonic() < deadline:
                        time.sleep(0.02)
                    self.assertTrue((root / "pid").exists())
                    if cancelled:
                        process.send_signal(signal.SIGTERM)
                    stdout, stderr = process.communicate(timeout=8)
                    self.assertEqual(process.returncode, 143 if cancelled else 1, stderr)
                    self.assertEqual(json.loads(stdout.splitlines()[-1])["status"],
                                     "cancelled" if cancelled else "failed")
                    with self.assertRaises(ProcessLookupError):
                        os.kill(int((root / "pid").read_text()), 0)
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.communicate()


if __name__ == "__main__":
    unittest.main()
