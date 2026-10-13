"""Offline checks for workflow artifact destinations; never dispatch real jobs."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class DownloadDestinationTest(unittest.TestCase):
    def test_default_and_overrides(self):
        source = Path(__file__).resolve().parents[1] / "scripts/run_github_linux_build.sh"
        for override, arguments, expected in [
            (None, [], "target/artifacts/ci-123"),
            ("custom-env", [], "custom-env"),
            ("custom-env", ["--download-dir", "custom-cli"], "custom-cli"),
        ]:
            with self.subTest(expected=expected), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / "scripts").mkdir()
                script = root / "scripts/run_github_linux_build.sh"
                script.write_bytes(source.read_bytes())
                fake = root / "gh"
                fake.write_text(
                    '#!/bin/sh\n'
                    'case "$1 $2" in\n'
                    '  "auth status"|"run watch") exit 0 ;;\n'
                    '  "run download") test "$3" = 123 && test "$4" = --dir '
                    '&& test "$5" = "$EXPECTED" ;;\n'
                    '  *) exit 99 ;;\n'
                    'esac\n'
                )
                fake.chmod(0o755)
                env = os.environ.copy()
                env.pop("DOWNLOAD_DIR", None)
                env.update(PATH=f"{root}:{env['PATH']}", REF="fixture", EXPECTED=expected)
                if override:
                    env["DOWNLOAD_DIR"] = override
                result = subprocess.run(
                    ["bash", str(script), "--run-id", "123", *arguments],
                    env=env, capture_output=True, text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue((root / expected).is_dir())
                self.assertFalse((root / "artifacts").exists())


if __name__ == "__main__":
    unittest.main()
