#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

WORKFLOW="${WORKFLOW:-build-deb.yml}"
REF="${REF:-$(git branch --show-current)}"
DOWNLOAD_DIR="${DOWNLOAD_DIR:-artifacts/deb}"
RUN_ID=""
WAIT_FOR_EXISTING=0

usage() {
  cat <<EOF
Usage: scripts/run_github_linux_build.sh [options]

Trigger the Debian-package workflow, wait for it, and download its artifacts.

Options:
  --ref <branch-or-tag>      Ref already pushed to GitHub (default: current branch)
  --workflow <file-or-name>  Workflow identifier (default: ${WORKFLOW})
  --download-dir <path>      Artifact destination (default: ${DOWNLOAD_DIR})
  --run-id <id>              Use an existing workflow run
  --wait-for-existing        Find the newest run for --ref instead of dispatching
  -h, --help                 Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref) REF="${2:?--ref requires a value}"; shift 2 ;;
    --workflow) WORKFLOW="${2:?--workflow requires a value}"; shift 2 ;;
    --download-dir) DOWNLOAD_DIR="${2:?--download-dir requires a value}"; shift 2 ;;
    --run-id) RUN_ID="${2:?--run-id requires a value}"; shift 2 ;;
    --wait-for-existing) WAIT_FOR_EXISTING=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

for command in gh git; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing command: $command" >&2; exit 2; }
done
gh auth status >/dev/null 2>&1 || { echo "GitHub CLI is not authenticated; run: gh auth login" >&2; exit 2; }
[[ -n "$REF" ]] || { echo "A branch or tag ref is required" >&2; exit 2; }

if [[ -z "$RUN_ID" && "$WAIT_FOR_EXISTING" -eq 0 ]]; then
  echo "Dispatching ${WORKFLOW} for ${REF}"
  gh workflow run "$WORKFLOW" --ref "$REF"
fi

if [[ -z "$RUN_ID" ]]; then
  echo "Waiting for a workflow run for ${REF}"
  for _ in {1..40}; do
    RUN_ID="$(gh run list --workflow "$WORKFLOW" --branch "$REF" --limit 1 --json databaseId --jq '.[0].databaseId // empty')"
    [[ -n "$RUN_ID" ]] && break
    sleep 3
  done
  [[ -n "$RUN_ID" ]] || { echo "Could not find a workflow run for ${REF}" >&2; exit 1; }
fi

echo "Watching run ${RUN_ID}"
gh run watch "$RUN_ID" --exit-status
mkdir -p "$DOWNLOAD_DIR"
gh run download "$RUN_ID" --dir "$DOWNLOAD_DIR"
echo "Downloaded artifacts to ${DOWNLOAD_DIR}:"
find "$DOWNLOAD_DIR" -type f -print | sort
