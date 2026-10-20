#!/usr/bin/env bash
# Ephemeral GitHub runner preparation/build/export; never publish or install app.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/flatpak_ci.py "$@"
