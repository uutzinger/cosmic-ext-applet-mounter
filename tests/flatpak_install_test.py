"""Installation boundaries with synthetic receipts and command adapters only."""

import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts/lib"))
import flatpak_install as installer
from flatpak_inspection import expected_policy


class FakeReport:
    def __init__(self, directory, candidate, running=False, mismatch=False, fail=False):
        self.directory = directory
        self.candidate = candidate
        self.running, self.mismatch, self.fail = running, mismatch, fail
        self.calls = []
        self.installed = False

    def capture(self, *args):
        self.calls.append(args)
        if args[1] == "ps":
            return installer.APP if self.running else ""
        if args[1] == "override":
            return ""
        if "--show-ref" in args:
            return self.candidate["export_inspection"]["ref"]
        if "--show-commit" in args:
            return ("e" if self.mismatch else "d") * 64 if self.installed else "b" * 64
        return METADATA

    def run(self, args, **_kwargs):
        self.calls.append(tuple(args))
        if self.fail:
            raise subprocess.CalledProcessError(7, ["flatpak"])
        self.installed = True

    def phase(self, *_args, **_kwargs):
        pass


METADATA = (f"[Application]\nname={installer.APP}\n"
            "runtime=org.example.Platform/x86_64/1\nsdk=org.example.Sdk/x86_64/1\n")


class InstallTests(unittest.TestCase):
    def exercise(self, *, running=False, native=False, mismatch=False, fail=False, invalid=False, panel_only=False):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            candidate = {"application_id": installer.APP, "manifest_path": str(directory / "manifest.json"),
                         "source_sha": "a" * 40, "architecture": "x86_64", "branch": "master",
                         "output_dir": str(directory), "offline": False,
                         "export_inspection": {"ref": f"app/{installer.APP}/x86_64/master", "commit": "d" * 64,
                                               "metadata": expected_policy(METADATA, installer.APP, "x86_64")}}
            receipt = directory / "artifacts.json"
            receipt.write_text(json.dumps(candidate))
            report = FakeReport(directory, candidate, running, mismatch, fail)
            report.installed = panel_only
            with patch.object(installer, "run", side_effect=ValueError("tampered") if invalid else None) as verify:
                with patch.object(installer, "native_running", return_value=native):
                    if running or native or mismatch or invalid:
                        with self.assertRaises(ValueError):
                            installer.install(argparse.Namespace(candidate_receipt=receipt, activate_panel_entry=False, panel_entry_only=False), report)
                    elif fail:
                        with self.assertRaises(subprocess.CalledProcessError):
                            installer.install(argparse.Namespace(candidate_receipt=receipt, activate_panel_entry=False, panel_entry_only=False), report)
                    else:
                        installer.install(argparse.Namespace(candidate_receipt=receipt, activate_panel_entry=False, panel_entry_only=panel_only), report)
                verify.assert_called_once()
            installs = [call for call in report.calls if call[1] == "install"]
            self.assertEqual(len(installs), 0 if running or native or invalid or panel_only else 1)
            path = directory / "installation.json"
            if installs:
                result = json.loads(path.read_text())
                self.assertEqual(result["previous_commit"], "b" * 64)
                self.assertEqual(result["installation"], "pending" if mismatch or fail else "verified")
                self.assertEqual(result["acceptance"], "not-run")
                self.assertIn("--no-deps", installs[0])
                self.assertIn("--no-related", installs[0])
            self.assertFalse(any(call[1] in ("run", "kill", "uninstall", "update") for call in report.calls))

    def test_success_preserves_rollback_and_installs_only_verified_bundle(self):
        self.exercise()

    def test_panel_launcher_preserves_native_copy_and_rejects_symlink(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source, target = root / "export.desktop", root / "user.desktop"
            source.write_text(f"[Desktop Entry]\nExec=flatpak run {installer.APP}\nX-Flatpak={installer.APP}\n")
            target.write_text("Exec=/synthetic/native\n")
            result, receipt = {}, root / "installation.json"
            installer.activate_panel_entry(source, target, root, result, receipt)
            self.assertEqual((root / "previous-panel.desktop").read_text(), "Exec=/synthetic/native\n")
            self.assertFalse(target.exists())
            self.assertEqual(result["panel_entry"]["status"], "verified")
            link = root / "link.desktop"
            link.symlink_to(target)
            with self.assertRaises(ValueError):
                installer.activate_panel_entry(source, link, root, {}, receipt)

    def test_invalid_candidate_never_installs(self):
        self.exercise(invalid=True)

    def test_panel_only_does_not_reinstall_verified_candidate(self):
        self.exercise(panel_only=True)

    def test_running_flatpak_or_native_refuses_install(self):
        self.exercise(running=True)
        self.exercise(native=True)

    def test_partial_failure_and_identity_mismatch_retain_pending_receipt(self):
        self.exercise(fail=True)
        self.exercise(mismatch=True)


if __name__ == "__main__":
    unittest.main()
