"""Offline hosted selection/download/output/resume tests; never contact GitHub."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

from tooling_fixtures import ToolingFixture


class DownloadDestinationTest(unittest.TestCase):
    def test_default_and_overrides(self):
        for environment, extra, expected in [
            (None, [], "target/artifacts/ci-123"),
            ("custom-env", [], "custom-env"),
            ("custom-env", ["--download-dir", "custom-cli"], "custom-cli"),
        ]:
            with self.subTest(expected=expected), tempfile.TemporaryDirectory() as tmp:
                f = ToolingFixture(tmp)
                if environment:
                    f.env["DOWNLOAD_DIR"] = environment
                result = f.run("run_github_linux_build.sh", "--run-id", "123", *extra)
                self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
                summary = f.summary(result)
                self.assertEqual(summary["status"], "succeeded")
                receipt = Path(summary["artifact_receipt_path"])
                self.assertEqual(receipt.parent, f.root / expected)
                self.assertEqual(len(json.loads(receipt.read_text())["artifacts"]), 6)
                self.assertNotIn("NOISY", result.stdout + result.stderr)
                self.assertIn("NOISY", Path(summary["log_path"]).read_text())
                self.assertLess(len(result.stdout), 2500)
                self.assertEqual(os.stat(summary["log_path"]).st_mode & 0o777, 0o600)
                self.assertEqual(os.stat(summary["summary_path"]).st_mode & 0o777, 0o600)

    def test_correlated_dispatch_ignores_stale_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = ToolingFixture(tmp)
            result = f.run()
            self.assertEqual(result.returncode, 0, result.stdout)
            summary = f.summary(result)
            state = json.loads(Path(summary["summary_path"]).read_text())
            self.assertEqual(summary["run_id"], "123")
            self.assertEqual(len(state["request_id"]), 32)
            self.assertEqual(len([c for c in f.commands() if c[:3] == ["gh", "workflow", "run"]]), 1)
            self.assertFalse(any(c[:3] == ["gh", "run", "watch"] for c in f.commands()))

    def test_ambiguous_wrong_source_and_failed_ci_do_not_download(self):
        for config, args in [
            ({"ambiguous": True}, []),
            ({"run_sha": "b" * 40}, ["--run-id", "123"]),
            ({"workflow_path": ".github/workflows/publish-release.yml"}, ["--run-id", "123"]),
            ({}, ["--run-id", "999"]),
            ({"conclusion": "failure"}, ["--run-id", "123"]),
            ({"remote_source": "b" * 40}, []),
        ]:
            with self.subTest(config=config), tempfile.TemporaryDirectory() as tmp:
                f = ToolingFixture(tmp)
                f.configure(**config)
                result = f.run("run_github_linux_build.sh", *args)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(f.summary(result)["status"], "failed")
                self.assertFalse(any(c[:3] == ["gh", "run", "download"] for c in f.commands()))

    def test_dispatch_failure_preserves_exit_and_resume_request_without_redispatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = ToolingFixture(tmp)
            f.configure(dispatch_exit=23)
            failed = f.run()
            self.assertEqual(failed.returncode, 23)
            receipt = f.summary(failed)["summary_path"]
            self.assertTrue(json.loads(Path(receipt).read_text())["request_id"])
            f.configure(dispatch_exit=0)
            resumed = f.run("run_github_linux_build.sh", "--resume-report", receipt)
            self.assertEqual(resumed.returncode, 0, resumed.stdout)
            self.assertEqual(len([c for c in f.commands() if c[:3] == ["gh", "workflow", "run"]]), 1)

    def test_download_failure_preserves_exit_and_does_not_publish_partial_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = ToolingFixture(tmp)
            f.configure(download_exit=17)
            result = f.run("run_github_linux_build.sh", "--run-id", "123")
            self.assertEqual(result.returncode, 17)
            self.assertFalse((f.root / "target/artifacts/ci-123").exists())
            self.assertIsNone(f.summary(result)["artifact_receipt_path"])

    def test_resume_reuses_cache_and_rejects_tampering(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = ToolingFixture(tmp)
            first = f.run("run_github_linux_build.sh", "--run-id", "123")
            self.assertEqual(first.returncode, 0, first.stdout)
            second = f.run("run_github_linux_build.sh", "--run-id", "123")
            self.assertEqual(second.returncode, 0, second.stdout)
            self.assertEqual(len([c for c in f.commands() if c[:3] == ["gh", "run", "download"]]), 1)
            next((f.root / "target/artifacts/ci-123").glob("deb-*/*.deb")).write_bytes(b"tampered")
            third = f.run("run_github_linux_build.sh", "--run-id", "123")
            self.assertNotEqual(third.returncode, 0)

    def test_discovery_and_ci_timeouts_keep_receipt_without_retrying_build(self):
        for config, args in [
            ({"stale_only": True}, ["--discovery-timeout", "1"]),
            ({"status": "in_progress"}, ["--run-id", "123", "--timeout-seconds", "1"]),
        ]:
            with self.subTest(config=config), tempfile.TemporaryDirectory() as tmp:
                f = ToolingFixture(tmp)
                f.configure(**config)
                result = f.run("run_github_linux_build.sh", *args)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(f.summary(result)["status"], "failed")
                self.assertLessEqual(len([c for c in f.commands() if c[:3] == ["gh", "workflow", "run"]]), 1)

    def test_unrelated_directory_and_symbolic_receipt_are_not_adopted(self):
        for mode in ["unrelated", "symlink"]:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                f = ToolingFixture(tmp)
                destination = f.root / "target/artifacts/ci-123"
                if mode == "unrelated":
                    destination.mkdir(parents=True)
                    (destination / "sentinel").write_text("preserve")
                else:
                    first = f.run("run_github_linux_build.sh", "--run-id", "123")
                    self.assertEqual(first.returncode, 0)
                    receipt = destination / "receipt.json"
                    saved = f.root / "saved-receipt.json"
                    saved.write_bytes(receipt.read_bytes())
                    receipt.unlink()
                    receipt.symlink_to(saved)
                result = f.run("run_github_linux_build.sh", "--run-id", "123")
                self.assertNotEqual(result.returncode, 0)
                if mode == "unrelated":
                    self.assertEqual((destination / "sentinel").read_text(), "preserve")
                    self.assertFalse(any(c[:3] == ["gh", "run", "download"] for c in f.commands()))
                else:
                    self.assertTrue(receipt.is_symlink())

    def test_signal_reaps_local_cli_and_records_cancelled(self):
        with tempfile.TemporaryDirectory() as tmp:
            f = ToolingFixture(tmp)
            f.configure(sleep_command="gh")
            process = subprocess.Popen(
                ["bash", str(f.root / "scripts/run_github_linux_build.sh"), "--run-id", "123"],
                cwd=f.root, env=f.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                deadline = time.monotonic() + 5
                while not (f.root / "child.pid").exists() and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue((f.root / "child.pid").exists())
                child = int((f.root / "child.pid").read_text())
                process.send_signal(signal.SIGTERM)
                stdout, stderr = process.communicate(timeout=7)
                self.assertEqual(process.returncode, 143, stderr)
                self.assertEqual(json.loads(stdout.splitlines()[-1])["status"], "cancelled")
                with self.assertRaises(ProcessLookupError):
                    os.kill(child, 0)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()


if __name__ == "__main__":
    unittest.main()
