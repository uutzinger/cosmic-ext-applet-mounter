"""Offline publication-preflight checks; never creates tags or releases."""
from pathlib import Path
import subprocess
import tempfile
import runpy
import sys
from unittest.mock import patch
import unittest

ROOT = Path(__file__).resolve().parents[1]


class PublicationPreflight(unittest.TestCase):
    def check_fixture(self, *, tag='v0.5.3', lock='0.5.3', kind='stable', notes='Accepted release notes.'):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'debian').mkdir()
            (root / 'resources').mkdir()
            (root / 'docs/releases').mkdir(parents=True)
            (root / 'Cargo.toml').write_text('[package]\nversion = "0.5.3"\n')
            (root / 'Cargo.lock').write_text(
                '[[package]]\nname = "cosmic-ext-applet-mounter"\nversion = "' + lock + '"\n')
            (root / 'debian/changelog').write_text(
                'cosmic-ext-applet-mounter (0.5.3) unstable; urgency=medium\n\n'
                '  * Synthetic publication fixture.\n\n'
                ' -- Test Maintainer <test@example.invalid>  Mon, 12 Oct 2026 12:00:00 -0700\n')
            (root / 'resources/app.metainfo.xml').write_text(
                '<component><releases><release version="0.5.3" type="' + kind +
                '"/></releases></component>')
            if notes is not None:
                (root / 'docs/releases/v0.5.3.md').write_text(notes)
            return subprocess.run(['bash', str(ROOT / 'scripts/check-release-publication.sh'), tag],
                                  cwd=root, capture_output=True, text=True)

    def test_python310_fallback_uses_full_toml_parser(self):
        try:
            import tomllib as parser
        except ModuleNotFoundError:
            import tomli as parser
        with patch.dict(sys.modules, {"tomllib": None, "tomli": parser}):
            namespace = runpy.run_path(str(ROOT / "scripts/lib/publication_check.py"))
        document = '[package]\nversion = "0.5.4" # fixture\n[[dependency]]\nname = "sample"\n'
        self.assertEqual(namespace["tomllib"].loads(document), {
            "package": {"version": "0.5.4"}, "dependency": [{"name": "sample"}],
        })
        with self.assertRaises(parser.TOMLDecodeError):
            namespace["tomllib"].loads('[package]\nversion = [')

    def test_finalized_metadata_passes(self):
        result = self.check_fixture()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_draft_notes_fail(self):
        self.assertNotEqual(self.check_fixture(notes='Status: Draft').returncode, 0)

    def test_missing_notes_fail(self):
        self.assertNotEqual(self.check_fixture(notes=None).returncode, 0)

    def test_development_metadata_fails(self):
        self.assertNotEqual(self.check_fixture(kind='development').returncode, 0)

    def test_lock_mismatch_fails(self):
        self.assertNotEqual(self.check_fixture(lock='0.5.2').returncode, 0)

    def test_invalid_tag_fails(self):
        self.assertNotEqual(self.check_fixture(tag='../v0.5.3').returncode, 0)


if __name__ == '__main__':
    unittest.main()
