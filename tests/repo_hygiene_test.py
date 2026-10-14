"""Repository hygiene: executable entry points, catalog coverage and doc links."""

from pathlib import Path
import re
import subprocess
import unittest
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
DOCS = [ROOT / "AGENTS.md", ROOT / "README.md", ROOT / "scripts/README.md",
        *sorted((ROOT / "docs").rglob("*.md"))]
LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")


def entry_points():
    return sorted(p for p in (ROOT / "scripts").iterdir()
                  if p.is_file() and p.suffix in ("", ".sh", ".py"))


class RepositoryHygiene(unittest.TestCase):
    def test_entry_points_are_executable_in_git(self):
        listing = subprocess.run(["git", "ls-files", "-s", "scripts"], cwd=ROOT,
                                 capture_output=True, text=True)
        if listing.returncode:
            self.skipTest("not a git checkout")
        modes = {line.split("\t")[1]: line.split()[0] for line in listing.stdout.splitlines()}
        for path in entry_points():
            rel = path.relative_to(ROOT).as_posix()
            if rel in modes:
                self.assertEqual(modes[rel], "100755", f"{rel} must be executable")

    def test_every_entry_point_is_in_the_script_catalog(self):
        catalog = (ROOT / "scripts/README.md").read_text()
        for path in entry_points():
            if path.name != "README.md":
                self.assertIn(path.name, catalog, f"{path.name} missing from scripts/README.md")

    def test_relative_markdown_links_resolve(self):
        for doc in DOCS:
            if not doc.is_file():
                continue
            for target in LINK.findall(doc.read_text()):
                if re.match(r"[a-z][a-z0-9+.-]*:", target) or target.startswith("#"):
                    continue
                path = (doc.parent / unquote(target.split("#")[0])).resolve()
                self.assertTrue(path.exists(), f"{doc.relative_to(ROOT)}: broken link {target}")


if __name__ == "__main__":
    unittest.main()
