#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

DRY_RUN=0
REMOTE="origin"
BUILD_RUN_ID=""

usage() {
  cat <<'EOF'
Usage: scripts/build_release.sh [--dry-run] [--remote <name>] [--run-id <id>]

Validate a release, create and push its v<version> tag, then wait for the
GitHub Actions Debian build and download its artifacts. Tag pushes do not
automatically build or publish. --run-id reuses a successful exact-commit build
instead of dispatching a new one. After acceptance and separate approval,
dispatch publish-release.yml on the tag with build_run_id and accepted=true.
Promotion verifies/downloads that run's artifacts; it never rebuilds them.

Before running, update Cargo.toml, Cargo.lock, debian/changelog, release notes,
and commit and push the release commit.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --remote) REMOTE="${2:?--remote requires a value}"; shift 2 ;;
    --run-id) BUILD_RUN_ID="${2:?--run-id requires a value}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

for command in cargo dpkg-parsechangelog gh git python3; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing command: $command" >&2; exit 2; }
done

VERSION="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "cosmic-ext-applet-mounter"))')"
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
if [[ -n "$BUILD_RUN_ID" ]]; then
  python3 scripts/promote_release.py "$TAG" "$BUILD_RUN_ID" --source-sha "$(git rev-parse HEAD)" --check-run-only
fi

echo "Release preflight passed: ${TAG} from ${BRANCH} at $(git rev-parse --short HEAD)"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "Dry run: no tag was created or pushed"
  exit 0
fi

git tag -a "$TAG" -m "Release ${TAG}"
git push "$REMOTE" "refs/tags/${TAG}"
if [[ -n "$BUILD_RUN_ID" ]]; then
  bash scripts/run_github_linux_build.sh --ref "$TAG" --run-id "$BUILD_RUN_ID"
else
  bash scripts/run_github_linux_build.sh --ref "$TAG"
fi

echo "Build for ${TAG} completed; artifacts are in target/artifacts/ci-<run-id>. No release was published."
