#!/usr/bin/env bash
set -euo pipefail

# Check the production UI, excluding test fixtures, plus Fluent key integrity.
exec python3 "$(dirname "$0")/check-ui-translations.py"
