# Release, packaging and documentation procedures

Type: Contracts

Status: Accepted procedures, relocated unchanged from the agent guide; verified behavior is recorded in task and completion notes

Created: October 13, 2026

Last reviewed: October 20, 2026

Scope: Build and artifact policy, Git release gates, native/Debian/Flatpak/GitHub
procedures, documentation layout and release notes, and this repository's package
and release script bindings. Associated task: agent-guide reduction.

The [agent guide](../../AGENTS.md) loads every session and defines the
`${VARIABLE}` placeholders used below. Read this record when building packages,
preparing or publishing a release, or changing release documentation. Maintained
scripts enforce many of these rules in code; see the
[script catalog](../../scripts/README.md) and
[output contracts](script-output-contracts.md).

## Build and artifact policy

- Before rebuilding, state what changed, which accepted evidence is invalidated,
  and why the existing build/artifacts cannot be reused. Identify the package
  mode and affected targets; prefer a focused check or artifact inspection when
  recompilation is unnecessary. Do not dispatch a redundant matrix simply to
  obtain a newer receipt or attach a release tag.
- Build each immutable release source once per required package/architecture
  matrix, then inspect and publish those same artifacts. Reuse a successful
  branch build when its source SHA exactly matches the release tag; otherwise
  run one explicit release build. Do not rebuild merely to tag, publish, add
  acceptance receipts, or repeat completed unchanged checks. Rebuild only when
  source/package inputs change, a build fails, or artifacts are unavailable.
  Publication remains separately approved and must verify source/run identity,
  complete artifact coverage, checksums and advertised package acceptance.
- Keep generated release packages under `${ARTIFACTS_DIR}/v<version>/` and
  distinct local builds under `${ARTIFACTS_DIR}/candidates/`. Keep workflow
  downloads in run-specific directories under `${ARTIFACTS_DIR}/`. Preserve
  checksums, provenance and rollback files; equal version numbers do not make
  packages duplicates. Remove duplicate downloads only after checksum comparison.

- Keep release metadata and package contents consistent. Preserve configuration,
  credentials, and user data across upgrades/removal. Commit, tag, publish, and
  external submission actions follow the user's authorization and release gates.

## Git changes and release gates

- Inspect branch/worktree/staged state before editing or committing. Stage only
  intended source, tests and records; exclude packages, caches, logs and rollback
  material. Preserve unrelated changes and do not include them implicitly.
- Commit, push, tag and publication are distinct actions. Perform only those
  authorized for the current scope; recognize approval already granted. A request
  to edit code/documentation alone does not authorize publishing a release.
- Never force-push, amend accepted history or move an existing release tag to
  conceal later changes. Diagnose Git warnings without destructive housekeeping
  unless separately authorized.
- Finalize version/package metadata and release notes before freezing the build
  revision. Use that same revision for the single build, acceptance and tag,
  then promote its verified artifacts. A branch build is reusable only at the
  exact tag SHA; tagging before the single build is also valid.
- Record later acceptance/publication receipts separately without moving the
  immutable tag or rebuilding just to incorporate those records. Changed package
  inputs require renewed affected acceptance; unavailable/expired artifacts need
  an explicit replacement build, not silent substitution.
- Verify remote source/tag identity and published asset coverage/checksums.
  Report commit, run, tag and public release identifiers separately; source
  approval, build success, installed acceptance and publication are not synonyms.

## Technology-specific procedures

Select the applicable package mode and verification scope before building.
Acceptance in one mode does not establish acceptance in another; do not rebuild
all modes for every edit. These procedures do not prescribe project script names.

### Native source builds and installation

- Distinguish compilation, staged installation, live installation and launching.
  Inspect staged payloads before replacing installed files where appropriate.
- Verify the installed binary matches the accepted artifact and reload/restart
  the intended running instance through the supported desktop workflow. A debug
  launch or file replacement does not replace an already-running process.
- Preserve configuration and user data. Avoid competing runtime owners during
  installed checks and reuse accepted artifacts for unchanged behavior.

### Debian packaging procedure

- Use the project's maintained package recipe. Debian's `debian/rules` defines
  build/test/install steps; `debian/control` defines package dependencies.
  A package build may compile and test rather than merely wrap an existing binary.
- Inspect package name/version/architecture, dependencies, payload and checksums.
  A .deb file's existence is not package acceptance. Reuse compatible accepted
  hosted packages for local checks instead of automatically rebuilding them.
- Install the selected absolute package path through the package manager only
  with installation approval. Verify installed identity and affected lifecycle;
  preserve configuration, credentials, services, caches and data across upgrade
  and removal. Keep distinct candidate packages and provenance separate.

### Flatpak packaging procedure

- Use the selected source-based manifest and accepted runtime/BaseApp. Verify
  immutable source, offline dependencies, payload and effective permissions.
  Do not silently substitute runtimes, broaden grants or assume a dependency
  installed on the host is available inside the sandbox.
- Separate build, installation and store submission/publication. Confirm app
  identity/branch and replacement effect before installing; preserve rollback.
- Prototype builds that copy local binaries are not reproducible release proof.
  Verify affected installed sandbox, authentication, portal and lifecycle behavior;
  native acceptance alone does not establish final Flatpak acceptance.

### GitHub Actions build and promotion procedure

- Freeze release metadata/source before the build. Select a successful trusted
  repository/workflow run at the exact tag SHA; verify required target coverage,
  unexpired artifacts, checksums and package identities before promotion.
- Separate building from publication approval. Publishing should download and
  upload the accepted artifacts, not trigger recompilation. Do not automatically
  build again just because a tag was pushed or a release is being published.
- Treat source acceptance, hosted build/test success, installed acceptance and
  public asset verification as distinct evidence. Reject wrong-source, failed,
  incomplete, expired or mismatched artifacts; obtain an explicit replacement
  build when needed. Keep the chosen run/artifact identities visible throughout.


## Documentation layout and release notes

- Keep the repository README and agent guide at the root for discovery. Keep
  ${DESCRIPTION_FILE}, ${SPECIFICATION_FILE}, ${TASK_LIST_FILE}, and
  ${COMPLETION_NOTES_FILE} as the central project documents at their configured
  root locations. Engineering records support them rather than replacing them.
- Put README-linked setup, authentication, dependency, and source-build guides
  in ${USER_DOCS_DIR}, using lowercase hyphenated filenames. If that directory
  also supplies a public website, maintain index.md as its homepage and primary
  documentation catalog; keep README.md as the GitHub directory entry linking
  to that catalog and repository-specific guidance. Otherwise use README.md as
  the directory index. Keep public homepage/privacy locations stable;
  changing their URLs is a separate publication decision.
- Keep package-specific build guidance beside its packaging configuration when
  that makes commands and paths clearer, and link it from the documentation index.
- Store release notes in ${RELEASE_NOTES_DIR} as v<version>.md. Release notes are
  user-facing version records; inventories, design contracts, and technical audits
  remain in ${ENGINEERING_RECORDS_DIR}. Keep a release-note index with relative
  links and versions in ascending order.
- Create a release-note draft when the release scope is agreed and preparation
  begins. Summarize user-visible changes, fixes, upgrade/dependency requirements,
  compatibility, known limitations, and supported installation/package modes.
  Link relevant user guides. Keep raw command output and detailed execution
  history in ${COMPLETION_NOTES_FILE}.
- Mark drafts explicitly. Finalize notes after the applicable acceptance checks
  pass and before tagging/publication; describe only verified scope and identify
  remaining limitations. Record the release version and tested artifact identity
  when relevant. Creating notes does not authorize publishing a release.
- Preserve published notes as historical records. Make any necessary correction
  explicit and dated; do not silently claim later fixes existed in an old release.
- When document paths change, update task/completion references, specifications,
  README/user-guide links, tooling/help references, and directory indexes together.
  Resolve relative links from each file's actual location and check for stale
  filenames, broken anchors, accidental content loss, and secret leakage.


## Package build and installation bindings

Apply the relevant [technology-specific procedure](#technology-specific-procedures).
The following are this repository's entry points and installation behavior.

### Native source

- Follow ${BUILD_GUIDE_FILE}. `just build` and `just build-release` create native
  artifacts. `just run` launches a development instance; avoid a competing runtime
  owner during installed tests.
- `just stage` builds/stages an installation under the configured staging
  directory for inspection, not into the live host installation.
- `just install-user` builds/replaces user-local artifacts. Use it only when
  installation is authorized, verify installed ${BINARY_NAME} identity, then
  use this project's COSMIC reload workflow for live checks.

### Debian packages

- `just deb` invokes `dpkg-buildpackage`; `debian/rules` defines its build, test
  and payload steps, and `debian/control` defines dependencies for ${BINARY_NAME}.
- The current local command writes packages to the repository's parent directory.
  Collect them with provenance/checksums under ${ARTIFACTS_DIR}; keep distinct
  candidates separate. Use APT for approved installation, then reload the native
  COSMIC panel instance and obtain the applicable UI/lifecycle receipt. Preserve
  this applet's mirror/cache/recovery state and generated user services.

### Flatpak packages

- `bash scripts/build-flatpak.sh --help` describes explicit-manifest candidate
  build/export with private logs. It requires a clean pinned source checkout,
  refuses existing output by default, and never installs or submits. Explicit
  `--reuse-receipt` verifies an existing schema-2 candidate without invoking the
  builder; mismatches fail rather than triggering rebuild/repair. Shared repository/bundle
  metadata inspection is available through `scripts/inspect-artifacts.py`; it
  requires the ostree CLI and an explicit comparison baseline. Preserve producer
  summaries and receipts alongside artifact provenance; do not bypass reuse guards.
- Follow ${PACKAGING_GUIDE_FILE} for the selected source-based manifest and its
  build/install commands; that directory also contains explicitly labeled probes
  and prototypes. Verify ${APPLICATION_ID}, selected branch and helper/worker
  payloads. Native and Flatpak modes share saved connections/host service state;
  prevent competing runtime owners. Use ${TASK_LIST_FILE} for accepted runtime,
  manifests and supported release scope, not an assumption that all modes ship.

## GitHub release scripts and workflow bindings

Apply the general Git/release gates first. Local Git review/staging/commit/push
uses the normal non-destructive Git workflow; release sequencing belongs to the
maintained entry points below. Inspect their help/current implementation before
execution. Do not bypass them with a parallel ad hoc publication procedure.

| Task | Entry point | Execution boundary |
|---|---|---|
| Build or retrieve hosted Debian packages | `scripts/run_github_linux_build.sh` | Starts a manual matrix unless an existing run is selected; downloads artifacts, does not install/publish. |
| Build hosted Flatpak bundles | `.github/workflows/build-flatpak.yml`, `scripts/build-flatpak-ci.sh` | Manual dispatch on approved source; native x86_64/aarch64 runners prepare tools/runtime and call the existing candidate builder once each. No app installation/publication. Dispatch requires separate approval. |
| Prepare release tag and build/reuse artifacts | `scripts/build_release.sh` | Creates/pushes a new tag, then starts one matrix or reuses `--run-id`; requires tag/push approval, never publishes. |
| Validate release metadata/notes | `scripts/check-release-publication.sh` | Read-only preflight, not proof of package acceptance or publication approval. |
| Validate/download existing release artifacts | `python3 scripts/promote_release.py <tag> <run-id>` | Default is inspection only from a clean tagged checkout; no compilation/build dispatch. |
| Inspect hosted Flatpak candidates before tagging | `python3 scripts/promote_release.py <proposed-tag> --inspect-flatpaks --flatpak-run x86_64=<run-id> [--flatpak-run aarch64=<run-id>]` | Retains verified candidate bytes/provenance under the private report directory. No tag required; cannot publish, install or confer acceptance. |
| Publish accepted existing artifacts | `.github/workflows/publish-release.yml` | Manual tag dispatch with selected `build_run_id` and `accepted=true`; requires publication approval. |
| Local approved promotion alternative | `python3 scripts/promote_release.py <tag> <run-id> --publish --accepted` | Same verification policy; publishes existing artifacts without rebuilding. |

- `.github/workflows/build-deb.yml` defines the hosted Debian matrix and is
  manual-only, with no publication job. Tag pushes do not automatically build.
- `.github/workflows/build-flatpak.yml` is likewise manual-only. Ubuntu24.04
  x86_64/ARM runners build distribution-independent Flatpak targets. Runner-only
  setup installs manifest refs from Flathub and permits bubblewrap user namespaces
  if the ephemeral VM's AppArmor restriction is enabled. This is not a host setup
  recipe. Source manifest/app pins and runtime commit observations are retained;
  no runtime substitution or pin update is performed by the workflow.
- Flatpak workflow artifacts `flatpak-x86_64` and `flatpak-aarch64` contain the
  bundle, built metadata, producer summary/schema-2 receipt, portable hosted index
  and SHA256SUMS. Separate `flatpak-receipts-<arch>` diagnostics retain summaries
  even on failure; no raw private logs uploaded. Retention is 14 days. These
  remain unaccepted candidates. Original receipts contain runner-local absolute
  paths; the portable index names uploaded files and records source/run/ref/commit.
  Downloaded files do not automatically become local reuse receipts.
- `scripts/promote_release.py` verifies the six Debian targets and optional
  Flatpaks selected by repeated `--flatpak-run ARCH=RUN_ID`; the publication
  workflow exposes optional per-CPU run inputs. Each selected CPU job must have
  succeeded with exactly one unexpired artifact; another CPU failing the same
  matrix does not invalidate the successful job. Full checksums, pinned inputs,
  producer/runtime provenance and offline ref/commit/metadata must agree.
  Publication retains the exact release-tag packaging SHA requirement for every
  selected run, even when application pins agree. Current mixed packaging SHAs
  are candidates, not an approved release set. Changes to supported targets must
  update expected coverage, workflow and tests together.
- Mixed promotion attaches selected bundles, portable FLATPAK-PROVENANCE.json
  and combined SHA256SUMS alongside Debian packages. Acceptance/publication
  approval remains explicit; --inspect-flatpaks cannot satisfy or bypass the
  final tag/clean-checkout gate. Promotion never compiles or dispatches builds.
- Build/release tools use the shared `scripts/lib/task_output.py` receipt/logger;
  the stable Bash entry points delegate fixed tasks to `scripts/lib/release_tasks.py`.
  Use `--log-dir` for a new private log/summary directory and selected-run/receipt
  options for resumption; see ${SCRIPT_CATALOG_FILE} and the output contract.
- Use ${BUILD_GUIDE_FILE} for deployment commands and ${RELEASE_NOTES_DIR} for
  release content. Keep operational receipts in ${COMPLETION_NOTES_FILE}; do not
  change a frozen source/tag merely to attach those receipts.
