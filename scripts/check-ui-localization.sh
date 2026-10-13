#!/usr/bin/env bash
set -euo pipefail

# Check the production UI, excluding test fixtures, plus Fluent key integrity.
python3 "$(dirname "$0")/check-ui-translations.py"
exec python3 "$(dirname "$0")/../tests/auth_helper_localization_test.py"
