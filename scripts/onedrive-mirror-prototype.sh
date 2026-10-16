#!/usr/bin/env bash
# Development-only OneDrive harness. Mutations require explicit fixed actions.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/onedrive_prototype_tasks.py "$@"
