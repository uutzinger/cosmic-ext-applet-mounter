"""Disposable offline inspection fixtures; no builds or installed-data access."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from tooling_fixtures import ToolingFixture

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts/lib"))
from artifact_inspection import digest, regular_file, verify_debian, verify_matrix


class InspectionTests(unittest.TestCase):
    def cli(self, *args):
        return subprocess.run([sys.executable, str(ROOT / "scripts/inspect-artifacts.py"),
                               *map(str, args)], capture_output=True, text=True)

    def test_native_receipt_is_observation_not_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            file = root / "native"
            file.write_bytes(b"not executed")
            result = self.cli("native", file, "--log-dir", root / "report")
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads(result.stdout.splitlines()[-1])
            receipt = json.loads(Path(summary["artifact_receipt_path"]).read_text())
            self.assertIsNone(receipt["source_sha"])
            self.assertIsNone(receipt["run_id"])
            self.assertEqual(receipt["artifacts"][0]["sha256"], hashlib.sha256(file.read_bytes()).hexdigest())
            self.assertEqual(receipt["artifacts"][0]["size_bytes"], 12)

    def test_unsafe_or_inapplicable_arguments_rejected(self):
        for args in [("debian", "absent"), ("debian-matrix", "absent", "--version", "../bad"),
                     ("native", "absent", "--version", "0.5.4"),
                     ("native", "absent", "--architecture", "amd64")]:
            self.assertEqual(self.cli(*args).returncode, 2)

    def test_links_directories_and_missing_files_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            file = root / "file"
            file.touch()
            link = root / "link"
            link.symlink_to(file)
            directory_link = root / "directory-link"
            directory_link.symlink_to(root, target_is_directory=True)
            for path in [link, root, directory_link / "file"]:
                with self.subTest(path=path), self.assertRaises(ValueError):
                    regular_file(path)
            with self.assertRaises(FileNotFoundError):
                digest(root / "absent")

    def test_single_debian_uses_only_metadata_reader(self):
        with tempfile.TemporaryDirectory() as tmp:
            fixture = ToolingFixture(tmp)
            package = fixture.root / "candidate_amd64.deb"
            package.write_bytes(b"fixture")
            result = subprocess.run([sys.executable, str(ROOT / "scripts/inspect-artifacts.py"),
                                    "debian", str(package), "--version", "0.5.4",
                                    "--architecture", "amd64", "--log-dir", str(fixture.root / "receipt")],
                                    env=fixture.env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([c[0] for c in fixture.commands()], ["dpkg-deb"] * 3)
            with self.assertRaises(ValueError):
                verify_debian(package, "0.5.4", "amd64", lambda *args: "wrong")

    def test_invalid_matrix_tag_fails_before_metadata(self):
        with self.assertRaises(ValueError):
            verify_matrix(Path("."), "../../unsafe", lambda *args: self.fail("metadata called"))

    def test_failure_writes_receipt_without_success_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = self.cli("native", root / "missing", "--log-dir", root / "report")
            self.assertEqual(result.returncode, 1)
            summary = json.loads((root / "report/summary.json").read_text())
            self.assertEqual(summary["status"], "failed")
            self.assertIsNone(summary["artifact_receipt_path"])


if __name__ == "__main__":
    unittest.main()
