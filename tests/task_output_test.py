"""Shared receipt/logging tests using disposable local children only."""

import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts/lib"))
from task_output import TaskReport


class TaskOutputTests(unittest.TestCase):
    def test_atomic_running_and_final_success_receipts(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()) as console:
            with TaskReport("fixture", Path(tmp) / "task") as report:
                report.update(source_sha="a" * 40, run_id="123")
                running = json.loads((report.directory / "summary.json").read_text())
                self.assertEqual(running["status"], "running")
                self.assertIsNone(running["exit_code"])
                report.run([sys.executable, "-c", "print('private-output')"])
            final = json.loads(console.getvalue().splitlines()[-1])
            self.assertEqual(final["exit_code"], 0)
            self.assertEqual(final["source_sha"], "a" * 40)
            self.assertNotIn("private-output", console.getvalue())
            self.assertIn("private-output", Path(final["log_path"]).read_text())
            self.assertFalse(list(report.directory.glob(".receipt-*")))

    def test_failure_keeps_real_exit_and_does_not_echo_private_output(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()) as console:
            with self.assertRaises(SystemExit) as caught:
                with TaskReport("fixture", Path(tmp) / "task") as report:
                    report.run([sys.executable, "-c", "import sys; print('private-error',file=sys.stderr); sys.exit(37)"])
            self.assertEqual(caught.exception.code, 37)
            final = json.loads(console.getvalue().splitlines()[-1])
            self.assertEqual(final["status"], "failed")
            self.assertEqual(final["exit_code"], 37)
            self.assertNotIn("private-error", console.getvalue())
            self.assertIn("private-error", Path(final["log_path"]).read_text())

    def test_failure_prints_bounded_excerpt_without_traceback_in_log(self):
        script = "import sys; [print('line %d' % i) for i in range(500)]; print('x' * 20000); sys.exit(3)"
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()), \
                contextlib.redirect_stderr(io.StringIO()) as errors:
            with self.assertRaises(SystemExit):
                with TaskReport("fixture", Path(tmp) / "task") as report:
                    report.run([sys.executable, "-c", script])
            log = report.log_path.read_text()
        self.assertIn("last log lines", errors.getvalue())
        self.assertLess(len(errors.getvalue()), 4500)
        self.assertNotIn("Traceback", log)

    def test_unexpected_error_keeps_traceback_in_log(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()), \
                contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                with TaskReport("fixture", Path(tmp) / "task") as report:
                    raise ValueError("fixture failure")
            self.assertIn("ValueError: fixture failure", report.log_path.read_text())

    def test_existing_log_directory_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "sentinel"
            p.write_text("preserve")
            with self.assertRaises(FileExistsError):
                TaskReport("fixture", tmp)
            self.assertEqual(p.read_text(), "preserve")

    def test_command_timeout_reaps_child_and_preserves_failure_receipt(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()) as console:
            with self.assertRaises(SystemExit) as caught:
                with TaskReport("fixture", Path(tmp) / "task") as report:
                    report.run([sys.executable, "-c", "import time; time.sleep(30)"], timeout=0.05)
            self.assertEqual(caught.exception.code, 1)
            self.assertEqual(json.loads(console.getvalue().splitlines()[-1])["status"], "failed")


if __name__ == "__main__":
    unittest.main()
