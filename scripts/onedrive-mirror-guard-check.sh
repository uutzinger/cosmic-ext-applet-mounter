#!/usr/bin/env bash
# Live OneDrive bisync guard check; creates and purges one test folder on the named remote.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/mirror_guard_check.py "$@"
