"""Release archiving of the task list and completion notes, on a synthetic repository."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "scripts/lib/archive_tasks.py"

TASKS = """# Task List

Intro paragraph.

## Release 1.0 work

- [x] Done thing, see [guide](Guide.md).
- [ ] Open thing
  continued on a second line.
  - [ ] Nested open child
- [x] Another

### Sub area

- [ ] Open in subsection

## Release 1.1 work

- [ ] Active item
"""
NOTES = """# Notes

Intro.

### Entry one

Evidence for 1.0.

### Entry two (release 1.1)

Evidence for 1.1.
"""
TASKS_2 = TASKS + """
## Release 1.2 work

- [ ] Later item
"""


class ArchiveRelease(unittest.TestCase):
    def run_tool(self, root, version, tasks_from, notes_from, *extra):
        env = dict(os.environ, ARCHIVE_ROOT=str(root))
        return subprocess.run([sys.executable, str(TOOL), version, "--tasks-from", tasks_from,
                               "--notes-from", notes_from, "--allow-dirty",
                               "--log-dir", str(root / f"log-{version}-{len(extra)}"), *extra],
                              cwd=root, env=env, capture_output=True, text=True)

    def fixture(self, tmp):
        root = Path(tmp)
        (root / "Task List.md").write_text(TASKS)
        (root / "Task List Completion Notes.md").write_text(NOTES)
        (root / "Guide.md").write_text("guide")
        return root

    def test_archive_moves_history_and_carries_open_items(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = self.fixture(tmp)
            result = self.run_tool(root, "1.0.0", "## Release 1.1 work", "### Entry two")
            self.assertEqual(result.returncode, 0, result.stderr)
            tasks = (root / "Task List.md").read_text()
            self.assertIn("## Archive and open items carried forward", tasks)
            self.assertIn("- [ ] Open thing\n  continued on a second line.\n  - [ ] Nested open child", tasks)
            self.assertIn("_Subsection: Sub area_", tasks)
            self.assertIn("- [ ] Active item", tasks)
            self.assertEqual(tasks.count("Nested open child"), 1)
            archive = (root / "docs/archive/task-list-through-v1.0.0.md").read_text()
            self.assertIn("- [x] Done thing, see [guide](../../Guide.md).", archive)
            self.assertIn("- [ ] Open thing", archive)
            notes = (root / "Task List Completion Notes.md").read_text()
            self.assertIn("### Entry two", notes)
            self.assertNotIn("Evidence for 1.0.", notes)
            self.assertIn("Evidence for 1.0.", (root / "docs/archive/task-list-completion-notes-through-v1.0.0.md").read_text())
            self.assertIn("task-list-completion-notes-through-v1.0.0.md", notes)

    def test_pre_release_label_is_accepted_and_bad_version_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = self.fixture(tmp)
            bad = self.run_tool(root, "1.0", "## Release 1.1 work", "### Entry two")
            self.assertNotEqual(bad.returncode, 0)
            self.assertFalse((root / "docs").exists())
            result = self.run_tool(root, "1.0.0-pre", "## Release 1.1 work", "### Entry two")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / "docs/archive/task-list-through-v1.0.0-pre.md").exists())

    def test_dry_run_writes_nothing_and_reports_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = self.fixture(tmp)
            result = self.run_tool(root, "1.0.0", "## Release 1.1 work", "### Entry two", "--dry-run")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("carry forward 2 open items", result.stderr)
            self.assertEqual((root / "Task List.md").read_text(), TASKS)
            self.assertFalse((root / "docs").exists())

    def test_second_archive_keeps_earlier_carry_forward_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = self.fixture(tmp)
            self.assertEqual(self.run_tool(root, "1.0.0", "## Release 1.1 work", "### Entry two").returncode, 0)
            tasks = (root / "Task List.md").read_text()
            (root / "Task List.md").write_text(tasks + "\n## Release 1.2 work\n\n- [ ] Later item\n")
            notes = root / "Task List Completion Notes.md"
            notes.write_text(notes.read_text() + "\n### Entry three (release 1.2)\n\nEvidence for 1.2.\n")
            repeat = self.run_tool(root, "1.0.0", "## Release 1.2 work", "### Entry three")
            self.assertNotEqual(repeat.returncode, 0)
            second = self.run_tool(root, "1.1.0", "## Release 1.2 work", "### Entry three")
            self.assertEqual(second.returncode, 0, second.stderr)
            result = (root / "Task List.md").read_text()
            self.assertIn("- [ ] Open thing", result)
            self.assertIn("- v1.0.0:", result)
            self.assertIn("- v1.1.0:", result)
            self.assertIn("- [ ] Later item", result)
            self.assertEqual(result.count("Archive and open items carried forward"), 1)

    def test_unknown_or_ambiguous_break_is_rejected_without_changes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = self.fixture(tmp)
            for marker in ["## Missing", "## Release 1."]:
                result = self.run_tool(root, "1.0.0", marker, "### Entry two", marker)
                self.assertNotEqual(result.returncode, 0)
            self.assertEqual((root / "Task List.md").read_text(), TASKS)
            self.assertFalse((root / "docs").exists())


if __name__ == "__main__":
    unittest.main()
