#!/usr/bin/env bash
# Move completed task-list/notes history to docs/archive/; edits files only (no commit).
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/archive_tasks.py "$@"
