#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

DRY_RUN=0
REMOTE="origin"

usage() {
  cat <<'EOF'
Usage: scripts/build_release.sh [--dry-run] [--remote <name>]

Validate a release, create and push its v<version> tag, then wait for the
GitHub Actions Debian builds and download their artifacts. The tag workflow
creates the GitHub release after all six Ubuntu/architecture builds succeed.

Before running, update Cargo.toml, Cargo.lock, debian/changelog, release notes,
and commit and push the release commit.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --remote) REMOTE="${2:?--remote requires a value}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

for command in cargo dpkg-parsechangelog gh git; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing command: $command" >&2; exit 2; }
done

VERSION="$(cargo metadata --no-deps --format-version 1 | sed -n 's/.*"version":"\([^"]*\)".*/\1/p')"
DEBIAN_VERSION="$(dpkg-parsechangelog -S Version)"
TAG="v${VERSION}"
BRANCH="$(git branch --show-current)"

[[ -n "$VERSION" && "$VERSION" == "$DEBIAN_VERSION" ]] || {
  echo "Version mismatch: Cargo=${VERSION:-unknown}, Debian=${DEBIAN_VERSION:-unknown}" >&2
  exit 1
}
[[ -n "$BRANCH" ]] || { echo "Release must be made from a branch, not detached HEAD" >&2; exit 1; }
[[ -z "$(git status --porcelain)" ]] || { echo "Working tree is not clean; commit the release changes first" >&2; exit 1; }
git rev-parse --verify "${REMOTE}/${BRANCH}" >/dev/null 2>&1 || { echo "Missing ${REMOTE}/${BRANCH}; fetch or push the branch first" >&2; exit 1; }
[[ "$(git rev-parse HEAD)" == "$(git rev-parse "${REMOTE}/${BRANCH}")" ]] || {
  echo "HEAD is not the same commit as ${REMOTE}/${BRANCH}; push the release commit first" >&2
  exit 1
}
! git rev-parse --verify "refs/tags/${TAG}" >/dev/null 2>&1 || { echo "Tag ${TAG} already exists locally" >&2; exit 1; }
! git ls-remote --exit-code --tags "$REMOTE" "refs/tags/${TAG}" >/dev/null 2>&1 || { echo "Tag ${TAG} already exists on ${REMOTE}" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "GitHub CLI is not authenticated; run: gh auth login" >&2; exit 2; }

echo "Release preflight passed: ${TAG} from ${BRANCH} at $(git rev-parse --short HEAD)"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "Dry run: no tag was created or pushed"
  exit 0
fi

git tag -a "$TAG" -m "Release ${TAG}"
git push "$REMOTE" "refs/tags/${TAG}"
bash scripts/run_github_linux_build.sh --ref "$TAG" --wait-for-existing \
  --download-dir "artifacts/${TAG}"

echo "Release ${TAG} completed and artifacts were downloaded to artifacts/${TAG}"
