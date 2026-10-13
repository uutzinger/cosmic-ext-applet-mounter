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

This guide governs ${PROJECT_NAME}.

${VARIABLE} denotes a documentation placeholder referring to the values above;
these declarations do not export shell environment variables. Change this block
when reusing the guide. General rules apply to any project; the Rust and COSMIC
instructions apply when those technologies are configured above.

## Working rules

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

## Design before implementation

- Use ${DESCRIPTION_FILE} for product intent and ${SPECIFICATION_FILE} for
  testable behavior and safety boundaries. Clarify material contradictions
  before building an incompatible implementation.
- Identify supported desktops, operating systems, native packages, and sandboxed
  packages at design time. Prototype their integration boundaries early.
- Choose an API for every desktop-control operation before implementing it.
  Prefer standard D-Bus interfaces, desktop portals, and typed libraries over
  CLI wrappers. In particular, use NetworkManager D-Bus for network/VPN control,
  the systemd user-manager D-Bus API for user services/timers, and appropriate
  portals for sandboxed desktop requests.
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

## COSMIC desktop integration

- Follow supported ${UI_FRAMEWORK} application/applet patterns and the pinned
  toolchain. Read the existing entry points and package metadata before adding
  new windows or launch modes.
- Keep ${APPLICATION_ID} consistent across desktop/AppStream/icon metadata
  and window identity. For applets, retain the COSMIC applet metadata and test
  launch from the actual panel, including standalone settings windows.
- Use the correct session or system D-Bus for each service. Declare narrowly
  scoped permissions for required bus names and filesystem locations. Prefer
  portals and specific grants to blanket host-filesystem access.
- Define one owner for shared runtime state and route other windows/instances
  through typed IPC. Keep long operations cancellable without blocking that IPC
  or the UI; reconcile displayed state from observed backend state.
- Resolve portal-selected paths through the supported portal API. Specify which
  configuration and engine state belongs on the host and which belongs in the
  sandbox; preserve that ownership across installation methods.
- Use Fluent for app-owned labels, tooltips, notices, validation, and diagnostics,
  including separate graphical helpers. Keep protocol strings and user data
  distinct from translated messages. Use locale-aware plural rules and provide
  keyboard, accessible, and non-color state cues.
- For installed UI verification, close project settings/editor windows and use
  the authorized panel reload or logout/login workflow described in
  ${BUILD_GUIDE_FILE}. A debug launch does not replace an already-running
  installed applet. Preserve user configuration and unrelated processes.

## Rust build and verification

Inspect just --list, the justfile, and ${BUILD_GUIDE_FILE} first. Prefer the
repository's recipes where available:

    just fmt-check
    just localization-check
    just check
    just lint
    just test
    just verify

Use just fmt when formatting changes are needed. Inspect the constituent
recipes and their exit behavior before interpreting just verify. Recipes may
make known COSMIC-specific metadata warnings nonfatal; verify this in the
justfile. Where available, use just metadata-check-strict for the strict
validator result and record warnings and their rationale.

Direct Cargo equivalents for a repository with a committed lockfile are:

    cargo fmt --all -- --check
    cargo check --locked --all-targets
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all-targets

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

## Build, installation, and packaging

- just build and just build-release build native artifacts. just run launches
  a development instance; use the configured launch mode and avoid a competing
  runtime owner during installed tests.
- just stage stages installation for inspection. just install-user replaces
  user-local installed artifacts; use it when installation is part of the
  authorized workflow, verify the installed ${BINARY_NAME} matches the tested
  artifact, then reload the desktop instance for live checks.
- Follow ${PACKAGING_GUIDE_FILE} for sandboxed builds. Verify the accepted
  runtime, source revision, offline dependencies, permissions, and installed
  package behavior. Prototype success does not establish final-artifact
  acceptance.
- Keep release metadata and package contents consistent. Preserve configuration,
  credentials, and user data across upgrades/removal. Commit, tag, publish, and
  external submission actions follow the user's authorization and release gates.

## Task and evidence records

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

## Engineering records

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

### Creation and maintenance process

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
