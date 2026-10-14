"""Release-tooling constants must match the repository they describe."""

from pathlib import Path
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts/lib"))
import project  # noqa: E402


class ProjectConstants(unittest.TestCase):
    def test_package_matches_cargo(self):
        cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(project.PACKAGE, cargo["package"]["name"])

    def test_workflow_exists_and_matrix_counted(self):
        self.assertTrue((ROOT / project.WORKFLOW_PATH).is_file())
        self.assertEqual(project.ARTIFACT_COUNT, len(project.MATRIX))


if __name__ == "__main__":
    unittest.main()
