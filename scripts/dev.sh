#!/usr/bin/env bash
# Logged fixed just tasks; no installation or release operations.
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/lib/dev_tasks.py "$@"
