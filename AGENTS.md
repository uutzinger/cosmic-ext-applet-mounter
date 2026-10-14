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

- [Part I — General policy](#part-i--general-policy): working rules, verification,
  scripts, Git gates, Rust checks, task and engineering records.
- [Part II — Project bindings](#part-ii--project-bindings): COSMIC/API choices,
  recipes and runtime ownership.
- [Release, packaging and documentation procedures](docs/engineering/release-packaging-procedures-contracts.md):
  read on demand for build/artifact policy, release gates, native/Debian/Flatpak/GitHub
  procedures, release notes, documentation layout and the release script table.

General policy applies to every workflow. Project bindings select concrete tools
without waiving the general safeguards.

## Part I — General policy

### Working rules

- Read a file before editing it. Never revert or reformat changes you did not make.
- Keep patches to one purpose. Update ${TASK_LIST_FILE} when a task starts and when
  it finishes; mark `[x]` only for what was actually verified.
- Never put real credentials, tokens or organization names in source, fixtures,
  logs or docs. Redact them from any output you quote.
- Automated tests use temp directories, synthetic fixtures and fake adapters.
  Live tests run only against approved disposable resources.
- Before reporting a phase done, run the checks in the routine loop below. The
  report lists: files changed, commands run with exit status, failures, open risks,
  next step. Put evidence in ${COMPLETION_NOTES_FILE}, not in chat.
- If approval for the same scope was already given in this conversation, proceed;
  otherwise ask before any action in the approval gates below.

### Routine verification loop

Use `bash scripts/dev.sh <task>`; it logs to `target/task-runs/`, keeps the real
exit status and prints a one-line JSON receipt (check `status` and `source_dirty`).

1. After a Rust edit: `check`, then the focused test (`cargo test --locked <name>`).
2. Before reporting done: `gate` (= `fmt-check`, `lint`, `test`). Add `localization-check`
   if UI strings or `.ftl` files changed, `metadata-check` if desktop/AppStream
   files changed.
3. Run `verify` only when asked or before a release; it repeats step 2.
4. On failure read the stderr excerpt, then at most the failing section of
   `task.log`. Do not paste whole logs into the conversation.
5. Do not re-run a passing check whose inputs have not changed; cite the earlier
   receipt in ${COMPLETION_NOTES_FILE} (source SHA, task, outcome, log path).
6. Tasks expected to exceed a few minutes (release builds, package builds, hosted
   CI waits): give the operator the exact command, working directory, expected
   state changes and log path, and ask only for exit status and the summary line.
   Run short read-only checks yourself. If the user asked you to run it, run it.
7. Hosted builds take about 7 minutes. The release scripts wait and poll
   internally (default 60 seconds); run them once and read the summary line. Do
   not add your own polling loop around them.

### Design rules

- Product intent is in ${DESCRIPTION_FILE}; testable behavior in
  ${SPECIFICATION_FILE}. If they contradict each other or the request, ask before
  building.
- Decide the API for each desktop-control operation before coding it. Use the
  typed D-Bus/portal/library API from Part II; do not wrap a CLI because it is
  quicker. A CLI is acceptable only for engines that need a process (for example
  rclone); then use a fixed executable, separate validated arguments, a timeout,
  a bounded output size, and cancellation that reaps the child. Background workers
  accept fixed operations and validated identities, never shell text.
- Host command execution from the Flatpak sandbox (flatpak-spawn) needs a written
  justification and an entry in `docs/engineering/flatpak-host-command-inventory.md`
  (command, permission, path, owner, removal criteria).
- If an API is missing on the minimum supported host, either gate a fallback on
  capability/version or raise the stated minimum. Never swap runtimes or drop
  behavior silently.
- Put desktop control behind a trait with a fake implementation for tests. Native
  and Flatpak builds share the policy code; only transport, path resolution and
  launch specs differ.
- Check dependency minimums in packaging metadata and report typed errors from
  the actual operation. Do not probe CLI `--version` at startup.
- Derive state from structured data (D-Bus properties, exit codes, typed results).
  Never parse localized text or log lines, and never treat "timer is active" as
  "operation succeeded".
- Prefer D-Bus signals over polling. Any necessary poll has a fixed interval and
  upper bound. No blocking I/O, authentication or process wait on the UI thread.

### What a test result means

State which of these levels you verified; never claim a higher one.

1. Compiles (`check`).
2. Unit/fake-adapter tests pass (`test`).
3. Real engine or command integration (ignored live tests, run explicitly with
   their documented disposable setup).
4. Installed service behavior (user systemd units, D-Bus, after `install-user`).
5. Rendered UI in the running panel applet.
6. Final package (Debian or Flatpak) installed and exercised.

- Run the focused test first, then the broader gate. Repeat a live test only if
  the code under it changed or it failed.
- If a sandbox blocks sockets, D-Bus, GUI or network, record that and rerun with
  authorized access. A skipped, ignored or blocked test is not a pass.
- For anything stateful (mounts, services, mirrors) test rollback, cancellation,
  ownership, malformed state and partial failure. Real user services and data
  never appear in ordinary unit tests.

### Scripts

- Find tools in ${SCRIPT_CATALOG_FILE} (state-change boundaries) and
  ${SCRIPT_INVENTORY_FILE} (callers, archive status). Update both when a script is
  added, changed or retired. Do not archive a runtime helper without evidence it
  is unused.
- Use the maintained script or `just` recipe for builds, installs, inspections,
  releases and publication. Read its `--help` or source first; a script name does
  not prove it is safe.
- If a script lacks a capability or contradicts policy, say so and fix and test
  the script within the task. Do not bypass it with ad hoc commands. Editing a
  script does not authorize running its state-changing operations.
- If a manual exception is unavoidable: state its scope, get approval, record the
  reason, artifact identities and outcome in ${COMPLETION_NOTES_FILE}, then fold
  the fix back into the script.
- Scripts own sequencing, logging, exit status and the summary line; agent and
  operator use the same entry points.

### Git changes and approval gates

- Run `git status` and check the branch before editing or committing. Stage only
  intended source, tests and records; never packages, caches, logs or rollback files.
- These need separate approval for the current scope, unless already granted in
  this conversation: commit, push, tag, build dispatch (hosted CI), package
  installation (`install-user`, APT), service changes, deleting data, publication.
  Editing code or docs authorizes none of them.
- Never force-push, amend accepted history or move a release tag.
- Releases use only the maintained scripts, which enforce source, tag and
  artifact identity. Do not rebuild just to tag, publish or attach receipts.
- Before a build matrix, package build, release preparation or publication, read
  the [release and packaging procedures](docs/engineering/release-packaging-procedures-contracts.md).

### Rust verification procedure

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

## Part II — Project bindings

These instructions bind Part I's policy to the configured
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

Use `just fmt` when formatting changes are needed. Use the recipes directly,
or `bash scripts/dev.sh <task>` for allowlisted checks and
native builds with private logs and concise results. The wrapper runs the same
just recipes; it does not install, clean, launch or publish. See the script catalog.
Inspect constituent recipes and their exit behavior before interpreting `just verify`. Recipes may
make known COSMIC-specific metadata warnings nonfatal; verify this in the
justfile. Where available, use `just metadata-check-strict` for the strict
validator result and record warnings and their rationale.

Use the [Rust verification procedure](#rust-verification-procedure) when direct
Cargo checks are needed instead of a recipe.

### Documentation

Keep ${DESCRIPTION_FILE}, ${SPECIFICATION_FILE}, ${TASK_LIST_FILE} and
${COMPLETION_NOTES_FILE} at the root, user guides in ${USER_DOCS_DIR}, and release
notes in ${RELEASE_NOTES_DIR}. When paths change, update links and indexes
together. Layout and release-note rules are in the procedures record below.

### Packaging and release entry points

Native, Debian, Flatpak and GitHub release bindings (`just stage`, `just deb`,
`scripts/build-flatpak.sh`, `scripts/run_github_linux_build.sh`,
`scripts/build_release.sh`, `scripts/promote_release.py`, publication workflow)
are documented with their approval boundaries in the
[release and packaging procedures](docs/engineering/release-packaging-procedures-contracts.md).
Installation (`just install-user`, APT) and publication need explicit approval.
