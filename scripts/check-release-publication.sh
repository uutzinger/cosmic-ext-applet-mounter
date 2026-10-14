#!/usr/bin/env bash
# Read-only publication preflight. Run from the accepted source checkout.
set -euo pipefail
exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/lib/publication_check.py" "${1:?Supply the accepted v-version tag}"
