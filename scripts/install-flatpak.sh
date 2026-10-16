#!/usr/bin/env bash
# Explicit approved candidate installation; no launch or service changes.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/flatpak_install.py "$@"
