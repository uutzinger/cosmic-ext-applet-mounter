#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}/.."
# Tag/push approval is required; --run-id --resume never retags an existing source.
# This entry point never publishes. No release was published.
exec python3 "${SCRIPT_DIR}/lib/release_tasks.py" release-prepare "$@"
