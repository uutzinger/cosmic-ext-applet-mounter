"""Verified candidate reuse with fake CLIs: never run a real project build."""

import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

import flatpak_wrapper_test as fixtures


class ReuseTests(unittest.TestCase):
    fixture = fixtures.FlatpakTests.fixture
    invoke = fixtures.FlatpakTests.invoke
    calls = fixtures.FlatpakTests.calls

    def prepared(self, tmp):
        root, env, args, manifest = self.fixture(tmp)
        result = self.invoke(env, args)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        reuse = list(args)
        reuse[reuse.index("--log-dir") + 1] = str(root / "reuse")
        reuse.extend(["--reuse-receipt", str(root / "logs/artifacts.json")])
        return root, env, reuse, manifest

    def configure(self, root, **settings):
        (root / "config.json").write_text(json.dumps(settings))

    def assert_no_build(self, calls):
        self.assertFalse(any(c[0] == "flatpak-builder" for c in calls))
        self.assertFalse(any(c[0] == "flatpak" and c[1] in ["info", "build-export", "build-bundle", "install", "run"] for c in calls))

    def test_reuse_without_builder_or_runtimes_preserves_original_and_chains(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            original = (root / "logs/artifacts.json").read_bytes()
            before = len(self.calls(root))
            self.configure(root, missing_runtime=True, build_failure=True)
            (root / "bin/flatpak-builder").unlink()
            result = self.invoke(env, args)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            self.assertIn("candidate-reused", result.stdout)
            calls = self.calls(root)[before:]
            self.assert_no_build(calls)
            self.assertTrue(any(c[0] == "ostree" and c[2:] == ["fsck", "--quiet"] for c in calls))
            receipt = json.loads((root / "reuse/artifacts.json").read_text())
            old = json.loads(original)
            self.assertEqual(receipt["tools"], old["tools"])
            self.assertEqual(receipt["runtimes"], old["runtimes"])
            self.assertEqual(receipt["repository_integrity"], "ostree-fsck-passed")
            self.assertEqual(receipt["artifacts"], old["artifacts"])
            self.assertEqual((root / "logs/artifacts.json").read_bytes(), original)
            args[args.index("--log-dir") + 1] = str(root / "reuse-again")
            args[args.index("--reuse-receipt") + 1] = str(root / "reuse/artifacts.json")
            self.assertEqual(self.invoke(env, args).returncode, 0)
            self.assert_no_build(self.calls(root)[before:])

    def test_tampered_and_missing_artifacts_or_corrupt_repository_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            paths = [root / "candidate/org.example.Candidate-x86_64-candidate.flatpak",
                     root / "candidate/built-metadata.ini", root / "candidate/repo/metadata"]
            for index, kind in enumerate(["bundle", "metadata", "policy", "commit", "missing", "fsck"]):
                with self.subTest(kind=kind):
                    saved = {p: p.read_bytes() for p in paths}
                    if kind in ["bundle", "metadata", "policy"]:
                        paths[["bundle", "metadata", "policy"].index(kind)].write_bytes(b"tampered")
                    if kind == "missing":
                        paths[0].unlink()
                    self.configure(root, **({"ostree_commit": "e" * 64} if kind == "commit" else {"fsck_failure": True} if kind == "fsck" else {}))
                    attempt = list(args)
                    attempt[attempt.index("--log-dir") + 1] = str(root / f"reject-{index}")
                    before = len(self.calls(root))
                    result = self.invoke(env, attempt)
                    self.assertNotEqual(result.returncode, 0)
                    self.assert_no_build(self.calls(root)[before:])
                    self.assertFalse((root / f"reject-{index}/artifacts.json").exists())
                    for path, contents in saved.items():
                        path.write_bytes(contents)

    def test_bad_or_incomplete_receipt_and_failed_summary_never_adopt_candidate(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            receipt_path, summary_path = root / "logs/artifacts.json", root / "logs/summary.json"
            original, summary = json.loads(receipt_path.read_text()), json.loads(summary_path.read_text())
            for index, kind in enumerate(["legacy", "options", "runtime", "tools", "path", "failed", "missing-summary", "symbolic"]):
                with self.subTest(kind=kind):
                    prior = json.loads(json.dumps(original))
                    old_summary = dict(summary)
                    if kind == "legacy": prior["schema_version"] = 1
                    if kind == "options": prior["build_options"]["disable_updates"] = False
                    if kind == "runtime": prior["runtimes"] = []
                    if kind == "tools": prior["tools"] = {}
                    if kind == "path": prior["artifacts"][0]["path"] = "/outside/candidate.flatpak"
                    if kind == "failed": old_summary["status"] = "failed"
                    receipt_path.write_text(json.dumps(prior))
                    summary_path.write_text(json.dumps(old_summary))
                    if kind == "missing-summary": summary_path.unlink()
                    if kind == "symbolic":
                        moved = root / "saved-receipt"
                        receipt_path.rename(moved)
                        receipt_path.symlink_to(moved)
                    attempt = list(args)
                    attempt[attempt.index("--log-dir") + 1] = str(root / f"baseline-{index}")
                    before = len(self.calls(root))
                    self.assertNotEqual(self.invoke(env, attempt).returncode, 0)
                    self.assert_no_build(self.calls(root)[before:])
                    if receipt_path.is_symlink(): receipt_path.unlink()
                    receipt_path.write_text(json.dumps(original))
                    summary_path.write_text(json.dumps(summary))

    def test_source_and_option_mismatch_fail_without_build(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            source = root / "source/input.txt"
            original = source.read_bytes()
            for index, kind in enumerate(["input", "dirty", "sha", "branch", "offline", "arch"]):
                attempt = list(args)
                attempt[attempt.index("--log-dir") + 1] = str(root / f"identity-{index}")
                self.configure(root, **({"dirty": True} if kind == "dirty" else {"sha": "b" * 40} if kind == "sha" else {}))
                if kind == "input": source.write_bytes(b"changed")
                if kind == "branch": attempt[attempt.index("--branch") + 1] = "other"
                if kind == "offline": attempt.append("--offline")
                if kind == "arch": attempt[attempt.index("--arch") + 1] = "aarch64"
                before = len(self.calls(root))
                self.assertNotEqual(self.invoke(env, attempt).returncode, 0, kind)
                self.assert_no_build(self.calls(root)[before:])
                source.write_bytes(original)

    def test_mid_verification_changes_and_symbolic_repository_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            targets = ["logs/artifacts.json", "logs/summary.json", "source/input.txt"]
            for index, target in enumerate(targets):
                saved = (root / target).read_bytes()
                self.configure(root, mutate_on_fsck=target)
                attempt = list(args)
                attempt[attempt.index("--log-dir") + 1] = str(root / f"mutation-{index}")
                before = len(self.calls(root))
                self.assertNotEqual(self.invoke(env, attempt).returncode, 0)
                self.assert_no_build(self.calls(root)[before:])
                (root / target).write_bytes(saved)
            self.configure(root)
            (root / "candidate/repo").rename(root / "saved-repo")
            (root / "candidate/repo").symlink_to(root / "saved-repo")
            before = len(self.calls(root))
            self.assertNotEqual(self.invoke(env, args).returncode, 0)
            self.assertFalse(any("fsck" in c for c in self.calls(root)[before:]))

    def test_partial_build_cannot_resume_or_rebuild_but_new_output_is_explicit(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp, build_failure=True)
            self.assertEqual(self.invoke(env, args).returncode, 25)
            args[args.index("--log-dir") + 1] = str(root / "reuse")
            before = len(self.calls(root))
            self.assertNotEqual(self.invoke(env, [*args, "--reuse-receipt", str(root / "logs/artifacts.json")]).returncode, 0)
            self.assert_no_build(self.calls(root)[before:])
            self.configure(root)
            args[args.index("--log-dir") + 1] = str(root / "new-build")
            args[args.index("--output-dir") + 1] = str(root / "new-candidate")
            self.assertEqual(self.invoke(env, args).returncode, 0)
            self.assertTrue((root / "candidate").exists())

    def test_cancellation_during_reuse_keeps_original_and_does_not_build(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.prepared(tmp)
            original = (root / "logs/artifacts.json").read_bytes()
            self.configure(root, sleep_fsck=True)
            before = len(self.calls(root))
            process = subprocess.Popen(args, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                deadline = time.monotonic() + 6
                while not (root / "pid").exists() and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue((root / "pid").exists())
                process.send_signal(signal.SIGTERM)
                process.communicate(timeout=8)
                self.assertEqual(process.returncode, 143)
                self.assert_no_build(self.calls(root)[before:])
                self.assertEqual((root / "logs/artifacts.json").read_bytes(), original)
                self.assertFalse((root / "reuse/artifacts.json").exists())
                with self.assertRaises(ProcessLookupError):
                    os.kill(int((root / "pid").read_text()), 0)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()


if __name__ == "__main__":
    unittest.main()
