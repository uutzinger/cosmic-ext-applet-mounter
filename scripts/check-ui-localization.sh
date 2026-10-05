#!/usr/bin/env bash
set -euo pipefail

status=0

check_pattern() {
    local description="$1"
    local pattern="$2"
    local matches
    matches="$(rg -n -U "$pattern" src/app.rs src/app/sftp.rs || true)"
    if [[ -n "$matches" ]]; then
        printf 'Hardcoded user-facing text found in %s:\n%s\n' "$description" "$matches" >&2
        status=1
    fi
}

# UI labels/help and notices must enter the Fluent catalog through fl!(...).
check_pattern 'rendered widgets' 'widget::(text::(body|caption|title[0-9]?)|button::(standard|suggested|destructive|text))\(\s*"[A-Za-z]'
check_pattern 'toggler labels' '\.label\(\s*"[A-Za-z]'
check_pattern 'connection notices' 'last_notice\s*=\s*Some\(\s*(format!\s*\(|"[A-Za-z])'
check_pattern 'settings notices' '(sleep_notice|preload_notice)\s*=\s*Some\(\s*(format!\s*\(|"[A-Za-z])'

exit "$status"
