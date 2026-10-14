"""Fixed just recipe dispatch with private logs and concise task receipts."""

import argparse
import os
from pathlib import Path
import re

from task_output import TaskReport

RECIPES = ("fmt-check", "localization-check", "check", "lint", "test", "verify",
           "metadata-check", "metadata-check-strict", "build", "build-release")


def main():
    parser = argparse.ArgumentParser(description=__doc__, prog="scripts/dev.sh")
    parser.add_argument("task", choices=RECIPES)
    parser.add_argument("--log-dir", help="New private receipt/log directory")
    parser.add_argument("--timeout-seconds", type=int, default=7200)
    args = parser.parse_args()
    if not 1 <= args.timeout_seconds <= 86400:
        parser.error("--timeout-seconds must be between 1 and 86400")
    os.chdir(Path(__file__).resolve().parents[2])
    with TaskReport("dev-" + args.task, args.log_dir) as report:
        report.update(recipe=args.task)
        sha = report.capture("git", "rev-parse", "--verify", "HEAD^{commit}")
        if not re.fullmatch(r"[0-9a-f]{40}", sha):
            raise ValueError("Invalid checkout source identity")
        dirty = bool(report.capture("git", "status", "--porcelain", "--untracked-files=normal"))
        report.update(source_sha=sha, source_dirty=dirty)
        report.phase("running-recipe", next_action="Inspect the private log on failure; no automatic retry or installation.")
        report.run(["just", "--justfile", "justfile", args.task], timeout=args.timeout_seconds)
        report.phase("recipe-complete", next_action="Recipe completed; consult its scope, not an installed/package acceptance claim.")


if __name__ == "__main__":
    main()
