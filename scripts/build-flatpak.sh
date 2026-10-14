#!/usr/bin/env bash
# Explicit-manifest candidate build/export only; never installs or publishes.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/flatpak_tasks.py "$@"
