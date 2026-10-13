# Agent Working Guide

## Project variables

    PROJECT_NAME = "Cloud Mounter"
    BINARY_NAME = "cosmic-ext-applet-mounter"
    APPLICATION_ID = "io.github.uutzinger.cosmic-ext-applet-mounter"
    LANGUAGE = "Rust"
    DESKTOP = "COSMIC"
    UI_FRAMEWORK = "libcosmic"
    TASK_LIST_FILE = "Task List.md"
    COMPLETION_NOTES_FILE = "Task List Completion Notes.md"
    DESCRIPTION_FILE = "Applet Description.md"
    SPECIFICATION_FILE = "Requirements and Specifications.md"
    BUILD_GUIDE_FILE = "docs/build-from-source.md"
    PACKAGING_GUIDE_FILE = "packaging/flatpak/README.md"
    ENGINEERING_RECORDS_DIR = "docs/engineering"
    USER_DOCS_DIR = "docs"
    RELEASE_NOTES_DIR = "docs/releases"
    BUILD_OUTPUT_DIR = "target"
    ARTIFACTS_DIR = "target/artifacts"
    SCRIPT_CATALOG_FILE = "scripts/README.md"
    SCRIPT_INVENTORY_FILE = "docs/engineering/script-tooling-inventory.md"

This guide governs ${PROJECT_NAME}.

${VARIABLE} denotes a documentation placeholder referring to the values above;
these declarations do not export shell environment variables. Change this block
when reusing the guide. General rules apply to any project; the Rust and COSMIC
instructions apply when those technologies are configured above.

## Guide map

- [Part I — General policy and technology procedures](#part-i--general-policy-and-technology-procedures):
  reusable policy plus applicable Rust, Debian, Flatpak and GitHub procedures.
- [Part II — Project bindings](#part-ii--project-bindings): concrete COSMIC/API
  choices, recipes, manifests, paths and release entry points for this project.

General policy applies to every workflow. Technology procedures apply only when
that technology is in use; they are reusable across projects, not universal
requirements to use a particular package system. Project bindings select the
concrete tools and configuration without waiving the general safeguards.

## Part I — General policy and technology procedures

### Working rules

- Work through the authorized task or phase and keep ${TASK_LIST_FILE} current.
- Inspect the current files before editing and preserve unrelated user changes.
- Use small, reviewable patches with a clear purpose.
- Keep real credentials, tokens, and organization-specific secrets out of source,
  fixtures, logs, and documentation. Redact diagnostic output.
- Use temporary directories, synthetic fixtures, and fake adapters for automated
  tests. Use approved disposable resources for live integration tests.
- Complete the applicable verification before reporting a phase as complete.
- Report changed files, checks performed, failures, unresolved risks, and the
  next relevant step. Save evidence in ${COMPLETION_NOTES_FILE}.
- Honor explicit approval gates. Obtain approval when it is still required;
  recognize approval already given for the same scope instead of asking again.

### Efficient verification and operator handoff

- Choose checks from the changed behavior and acceptance boundary. Reuse recorded
  successful evidence for unchanged behavior; do not rerun a broad gate merely
  because the task moved to a different phase.
- Keep verbose build/test output in task-specific logs under ${BUILD_OUTPUT_DIR}
  or a private temporary directory, not in the conversation or tracked source.
  Preserve the real command exit status; output truncation must not hide failure.
- Monitor long operations with status-only queries at bounded intervals. Do not
  repeatedly stream compile logs or refreshing watch output into model context.
  Read a short completion summary on success. On failure, inspect the failing
  step and a bounded relevant excerpt; expand only when needed for diagnosis.
- Record source SHA, run ID, artifact/checksum identity, check scope, outcome and
  log location once in ${COMPLETION_NOTES_FILE}; reference that receipt on later
  steps instead of rereading full logs. Redact secrets before sharing excerpts.
- For long builds, package creation or verbose verification, prefer offering
  an operator-run shell command unless automation was explicitly requested.
  Give the working directory, maintained script/recipe, expected state changes,
  log destination and exact result to return. Ask for the exit status and concise
  summary, not pasted compiler output. Retain responsibility for interpreting
  failures, tracking progress and completing verification.
- Do short read-only checks and focused tests directly when useful. Do not ask
  the operator to perform every small step. When automation is requested, run
  the scoped workflow with bounded output rather than delegating it back.
- Operator execution does not replace approval gates. Distinguish commands that
  only inspect/build from installation, service changes, Git push/tag/publication
  and data removal. Obtain or recognize the applicable approval before either
  the agent or operator performs those changes.

### Design before implementation

- Use ${DESCRIPTION_FILE} for product intent and ${SPECIFICATION_FILE} for
  testable behavior and safety boundaries. Clarify material contradictions
  before building an incompatible implementation.
- Identify supported desktops, operating systems, native packages, and sandboxed
  packages at design time. Prototype their integration boundaries early.
- Choose an API for every desktop-control operation before implementing it.
  Prefer supported structured APIs and typed libraries over CLI wrappers. See
  Part II for the project's required desktop APIs and transport choices.
- A host-spawn mechanism is a transport exception, not the default architecture.
  Justify each production host invocation by the capability it supplies that an
  available API cannot provide. Preserve an inventory of commands, permissions,
  paths, ownership, and fallback-removal criteria when host execution is needed.
- Verify API availability and permissions on the minimum supported host. If an
  API is unavailable, specify a capability/version-gated fallback or an explicit
  minimum requirement. Do not silently substitute a runtime or drop behavior.
- Put desktop control behind typed, testable adapters. Native and sandboxed
  packages must share operation policy and safety contracts; isolate differences
  in transport, path resolution, and package launch specifications.
- Retain process execution for engines whose work requires it. Use fixed
  executables, separate validated arguments, bounded output/timeouts, controlled
  retries, and cancellation that finishes child cleanup. Background workers
  accept fixed operations and validated identities, not arbitrary shell text or
  executable/command payloads.
- Enforce dependency minimums through packaging and installation checks, then
  provide typed failures from actual operations. Avoid startup-wide CLI
  version/help probes used only to inventory dependencies.
- Keep observations and outcomes structured. Do not determine control state by
  parsing localized display text, scraping human-readable logs, or assuming
  that an active timer means its operation succeeded.
- Prefer notifications/signals over repeated external commands. Bound necessary
  polling and keep long-running I/O, authentication, and process waits off the
  graphical event loop.

### Verification boundaries

- Start with focused tests for the affected behavior, then run the applicable
  broader gate. Do not repeat completed live tests unless changes or failures
  invalidate their evidence.
- Distinguish compilation, fake-adapter tests, command/engine integration,
  installed-service tests, rendered UI review, and final-package acceptance.
  State exactly which boundary was verified.
- If a managed sandbox blocks sockets, D-Bus, a GUI, or network access, record
  that result and rerun with the required authorized access. An excluded,
  ignored, or blocked test is not a passing test. Run applicable ignored live
  tests explicitly with their documented disposable setup.
- Include rollback, cancellation, ownership, malformed state, and partial-failure
  behavior in verification for stateful integrations. Keep real user services
  and data outside ordinary unit tests.

### Maintained scripts and execution ownership

- Use ${SCRIPT_CATALOG_FILE} to find current tools and their state-change
  boundaries; ${SCRIPT_INVENTORY_FILE} records callers, archive disposition and
  planned gaps. Update both when tools are added, changed or retired. Archive
  only with replacement/non-use evidence, preserving provenance and existing
  privacy/tracking boundaries; old-looking runtime helpers are not obsolete.
- Use maintained scripts/recipes for routine build, installation, inspection,
  Git release and publication operations. Inspect their current behavior and
  approval boundary before running them; a script name is not proof of safety.
- If a script lacks a required capability or contradicts policy, explain the
  gap and improve/test it within the authorized task instead of silently
  bypassing it with ad hoc commands. Changes to scripts are separate from
  permission to execute their state-changing operations.
- If an exceptional manual path is necessary, explain its scope, obtain any
  required approval and record the reason, exact artifact identities and outcome.
  Do not silently bypass a guard. Feed durable improvements back into the script
  so the same exception is not needed on each release.
- Scripts should own command sequencing, log files, preserved exit status and
  concise machine-readable/summary output. Agent and operator should use the
  same entry points; avoid maintaining a parallel conversational procedure.

### Build and artifact policy

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

### Git changes and release gates

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

### Technology-specific procedures

Select the applicable package mode and verification scope before building.
Acceptance in one mode does not establish acceptance in another; do not rebuild
all modes for every edit. These procedures do not prescribe project script names.

#### Native source builds and installation

- Distinguish compilation, staged installation, live installation and launching.
  Inspect staged payloads before replacing installed files where appropriate.
- Verify the installed binary matches the accepted artifact and reload/restart
  the intended running instance through the supported desktop workflow. A debug
  launch or file replacement does not replace an already-running process.
- Preserve configuration and user data. Avoid competing runtime owners during
  installed checks and reuse accepted artifacts for unchanged behavior.

#### Debian packaging procedure

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

#### Flatpak packaging procedure

- Use the selected source-based manifest and accepted runtime/BaseApp. Verify
  immutable source, offline dependencies, payload and effective permissions.
  Do not silently substitute runtimes, broaden grants or assume a dependency
  installed on the host is available inside the sandbox.
- Separate build, installation and store submission/publication. Confirm app
  identity/branch and replacement effect before installing; preserve rollback.
- Prototype builds that copy local binaries are not reproducible release proof.
  Verify affected installed sandbox, authentication, portal and lifecycle behavior;
  native acceptance alone does not establish final Flatpak acceptance.

#### GitHub Actions build and promotion procedure

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

#### Rust verification procedure

Use the pinned toolchain and project recipes when available. For projects with
a committed lockfile, direct Cargo equivalents include:

    cargo fmt --all -- --check
    cargo check --locked --all-targets
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all-targets

These are alternatives to applicable recipes, not an extra duplicate gate.
Record lockfile/features/targets with results; do not assume a recipe supplies
flags that its current definition does not contain.

### Task and evidence records

- Keep ${TASK_LIST_FILE} chronological: oldest work first, newest work last,
  with future tasks in the final sections. Use the original request/planning
  date; later verification dates do not change a task's historical position.
- Add new active work at the end of the chronological record, before the future
  sections. Append new future work to the appropriate final section.
- Preserve historical scope, decisions, failures, and acceptance boundaries.
  Keep active checklists concise and append dated execution evidence to
  ${COMPLETION_NOTES_FILE}.
- Status markers: `[ ]` means outstanding, `[x]` means completed, and `[o]`
  means deliberately skipped or handled elsewhere. Give transferred work a reference to its
  authoritative task. Do not mark source implementation as installed/native or
  sandboxed acceptance when those checks are still pending.

### Engineering records

Engineering records are maintained design and verification documents: inventories,
contracts, audits, designs, and decisions. Keep them under
${ENGINEERING_RECORDS_DIR}, separate from the repository's primary product,
requirements, task, and deployment documents.

- Use lowercase ASCII filenames with hyphens: subject-kind.md. Prefer the kinds
  inventory, contracts, audit, design, and decision. Choose a descriptive subject
  and retain the same filename while maintaining the same record.
- Keep dates in document metadata for maintained records. For a separate frozen
  baseline or historical snapshot, use YYYY-MM-DD-subject-kind.md and state which
  current record or task it supports. Do not create a new dated copy for every
  small update or silently overwrite an accepted historical baseline.
- Create the directory and its README.md index with the first record. List each
  record's purpose, type, status, and relative link in the index. Avoid deeper
  subdirectories until the number of records makes them useful.
- Track reviewed engineering records in version control. Compiled binaries,
  packages, caches, and transient raw reports/logs belong in the build tool's
  output area, such as ${BUILD_OUTPUT_DIR}, or a task-specific temporary directory.
  They are not engineering records and must not be mixed into that directory.

#### Creation and maintenance process

1. Create a record when work needs a durable baseline inventory, interface or
   safety contract, architecture decision, or audit beyond a checklist entry.
   Use existing records for routine follow-ups instead of creating redundant
   documents. Establish a required baseline or contract before its implementation.
2. Begin with a title, type, status, creation date, and last-reviewed date. Identify
   the scope, associated task, applicable requirements, and authoritative source
   files or interfaces. State whether the text describes an observed baseline,
   a proposed design, an accepted contract, or verified behavior.
3. Record assumptions, relevant versions/capabilities, ownership, dependencies,
   decisions, and unresolved issues. For inventories, classify each item and its
   disposition. For contracts, define permitted operations, invariants, failure
   handling, and acceptance criteria. For audits, state the method, coverage,
   intentional exceptions, findings, and corrective actions.
4. Separate documentation status from implementation and acceptance status.
   Identify what was checked in source, automated tests, native installations,
   live systems, and final sandboxed packages. Include dates and build identities
   when relevant; do not turn an accepted design into a claim of tested behavior.
5. Link the record from the appropriate task and the directory index. Put durable
   product requirements into ${SPECIFICATION_FILE}, or reference an accepted
   contract there. Keep detailed execution chronology in ${COMPLETION_NOTES_FILE}
   and avoid copying the same checklist or transcript into multiple records.
6. Update maintained records when related implementation or decisions change.
   Preserve frozen baselines and mark superseded records with links to their
   successors. Maintain references if a record is renamed and verify relative
   links from its actual directory. Review for secrets and private diagnostic
   data before recording evidence.

### Documentation layout and release notes

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

## Part II — Project bindings

These instructions bind Part I's policy and procedures to the configured
language, desktop, package modes and repository tooling. Commands below identify maintained entry
points, not instructions to run every command on every task.
The script catalog and inventory distinguish existing commands from proposed
tools; a planned filename is not an available or approved command.

### COSMIC desktop and API integration

#### Required API choices

- Use NetworkManager's system D-Bus API for network/VPN control and the systemd
  user-manager session D-Bus API for user services/timers. Use appropriate desktop
  portals for sandboxed requests. Do not replace these with CLI wrappers merely
  because a CLI is convenient.
- Use the correct session or system D-Bus for each service. Declare narrowly
  scoped permissions for required bus names and filesystem locations. Prefer
  portals and specific grants to blanket host-filesystem access.
- Resolve portal-selected paths through the supported portal API. Specify which
  configuration and engine state belongs on the host and which belongs in the
  sandbox; preserve that ownership across installation methods.

#### Desktop, UI and runtime ownership

- Follow supported ${UI_FRAMEWORK} application/applet patterns and the pinned
  toolchain. Read the existing entry points and package metadata before adding
  new windows or launch modes.
- Keep ${APPLICATION_ID} consistent across desktop/AppStream/icon metadata
  and window identity. For applets, retain the COSMIC applet metadata and test
  launch from the actual panel, including standalone settings windows.
- Define one owner for shared runtime state and route other windows/instances
  through typed IPC. Keep long operations cancellable without blocking that IPC
  or the UI; reconcile displayed state from observed backend state.
- Use Fluent for app-owned labels, tooltips, notices, validation, and diagnostics,
  including separate graphical helpers. Keep protocol strings and user data
  distinct from translated messages. Use locale-aware plural rules and provide
  keyboard, accessible, and non-color state cues.
- For installed UI verification, close project settings/editor windows and use
  the authorized panel reload or logout/login workflow described in
  ${BUILD_GUIDE_FILE}. A debug launch does not replace an already-running
  installed applet. Preserve user configuration and unrelated processes.

### Rust recipes and metadata checks

Inspect `just --list`, the justfile, and ${BUILD_GUIDE_FILE} first. Prefer the
repository's recipes, selecting only the applicable checks:

    just fmt-check
    just localization-check
    just check
    just lint
    just test
    just verify

Use `just fmt` when formatting changes are needed. Inspect the constituent
recipes and their exit behavior before interpreting `just verify`. Recipes may
make known COSMIC-specific metadata warnings nonfatal; verify this in the
justfile. Where available, use `just metadata-check-strict` for the strict
validator result and record warnings and their rationale.

Use the [Rust verification procedure](#rust-verification-procedure) when direct
Cargo checks are needed instead of a recipe.

### Package build and installation bindings

Apply the relevant [technology-specific procedure](#technology-specific-procedures).
The following are this repository's entry points and installation behavior.

#### Native source

- Follow ${BUILD_GUIDE_FILE}. `just build` and `just build-release` create native
  artifacts. `just run` launches a development instance; avoid a competing runtime
  owner during installed tests.
- `just stage` builds/stages an installation under the configured staging
  directory for inspection, not into the live host installation.
- `just install-user` builds/replaces user-local artifacts. Use it only when
  installation is authorized, verify installed ${BINARY_NAME} identity, then
  use this project's COSMIC reload workflow for live checks.

#### Debian packages

- `just deb` invokes `dpkg-buildpackage`; `debian/rules` defines its build, test
  and payload steps, and `debian/control` defines dependencies for ${BINARY_NAME}.
- The current local command writes packages to the repository's parent directory.
  Collect them with provenance/checksums under ${ARTIFACTS_DIR}; keep distinct
  candidates separate. Use APT for approved installation, then reload the native
  COSMIC panel instance and obtain the applicable UI/lifecycle receipt. Preserve
  this applet's mirror/cache/recovery state and generated user services.

#### Flatpak packages

- Follow ${PACKAGING_GUIDE_FILE} for the selected source-based manifest and its
  build/install commands; that directory also contains explicitly labeled probes
  and prototypes. Verify ${APPLICATION_ID}, selected branch and helper/worker
  payloads. Native and Flatpak modes share saved connections/host service state;
  prevent competing runtime owners. Use ${TASK_LIST_FILE} for accepted runtime,
  manifests and supported release scope, not an assumption that all modes ship.

### GitHub release scripts and workflow bindings

Apply the general Git/release gates first. Local Git review/staging/commit/push
uses the normal non-destructive Git workflow; release sequencing belongs to the
maintained entry points below. Inspect their help/current implementation before
execution. Do not bypass them with a parallel ad hoc publication procedure.

| Task | Entry point | Execution boundary |
|---|---|---|
| Build or retrieve hosted Debian packages | `scripts/run_github_linux_build.sh` | Starts a manual matrix unless an existing run is selected; downloads artifacts, does not install/publish. |
| Prepare release tag and build/reuse artifacts | `scripts/build_release.sh` | Creates/pushes a new tag, then starts one matrix or reuses `--run-id`; requires tag/push approval, never publishes. |
| Validate release metadata/notes | `scripts/check-release-publication.sh` | Read-only preflight, not proof of package acceptance or publication approval. |
| Validate/download existing release artifacts | `python3 scripts/promote_release.py <tag> <run-id>` | Default is inspection only from a clean tagged checkout; no compilation/build dispatch. |
| Publish accepted existing artifacts | `.github/workflows/publish-release.yml` | Manual tag dispatch with selected `build_run_id` and `accepted=true`; requires publication approval. |
| Local approved promotion alternative | `python3 scripts/promote_release.py <tag> <run-id> --publish --accepted` | Same verification policy; publishes existing artifacts without rebuilding. |

- `.github/workflows/build-deb.yml` defines the hosted Debian matrix and is
  manual-only, with no publication job. Tag pushes do not automatically build.
- `scripts/promote_release.py` implements the general promotion checks for this
  repository's Debian matrix. Changes to supported targets must update its
  expected artifact coverage, the workflow and their tests together.
- Build/release tools use the shared `scripts/lib/task_output.py` receipt/logger;
  the stable Bash entry points delegate fixed tasks to `scripts/lib/release_tasks.py`.
  Use `--log-dir` for a new private log/summary directory and selected-run/receipt
  options for resumption; see ${SCRIPT_CATALOG_FILE} and the output contract.
- Use ${BUILD_GUIDE_FILE} for deployment commands and ${RELEASE_NOTES_DIR} for
  release content. Keep operational receipts in ${COMPLETION_NOTES_FILE}; do not
  change a frozen source/tag merely to attach those receipts.
