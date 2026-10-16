#!/usr/bin/env bash
# Read-only preflight or pinned candidate build/export; never installs or publishes.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/flatpak_tasks.py "$@"
