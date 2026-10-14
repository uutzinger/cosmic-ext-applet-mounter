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

Prefer `bash scripts/dev.sh <task>`. When no recipe fits, run the Cargo equivalent
with `--locked` and record the features and targets used:

    cargo fmt --all -- --check
    cargo check --locked --all-targets
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all-targets

Do not run both a recipe and its Cargo equivalent.

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

Inventories, contracts, audits, designs and decisions live in `docs/engineering/`
(${ENGINEERING_RECORDS_DIR}). Read its [README](docs/engineering/README.md) before
creating or changing a record: it has the naming, header, status and indexing rules.

- Name files `subject-kind.md`; reuse an existing record for follow-ups.
- State documentation status separately from implementation and acceptance
  status; never present a design as tested behavior.
- Keep logs, packages and raw reports in ${BUILD_OUTPUT_DIR}, not in that directory.

## Part II — Project bindings

Commands below are the maintained entry points, not a list to run on every task.
A script named only in the catalog or inventory as "planned" does not exist.

### COSMIC desktop and API integration

#### Required APIs

- Network and VPN: NetworkManager on the system D-Bus. User services and timers:
  the systemd user manager on the session D-Bus. Sandboxed requests: desktop
  portals. Do not substitute CLI wrappers for these.
- Pick the correct bus per service. Request narrow Flatpak permissions (specific bus
  names and paths), never blanket host filesystem access.
- Resolve portal-selected paths through the portal API. Keep a written split of
  what state lives on the host and what lives in the sandbox, and keep it the
  same for native and Flatpak installs.

#### Applet, UI and runtime rules

- Follow existing ${UI_FRAMEWORK} applet patterns and the pinned toolchain. Read the
  entry points and package metadata before adding a window or launch mode.
- ${APPLICATION_ID} must be identical in the desktop file, AppStream metadata, icon
  and window app id. Keep the COSMIC applet metadata and test from the real panel,
  including the standalone settings window.
- One owner holds shared runtime state; other windows and instances talk to it
  through typed IPC. Long operations are cancellable and never block the IPC or
  UI thread. Show state read back from the backend, not state you assumed.
- All app-owned text (labels, tooltips, notices, validation, diagnostics, and the
  separate graphical helpers) goes through Fluent. Protocol strings and user data
  are not translated. Use locale plural rules, and give every state a keyboard
  path, an accessible name and a non-color cue.
- Installed UI checks: close settings/editor windows, then reload the panel or
  log out and in as described in ${BUILD_GUIDE_FILE}. A debug launch does not
  replace the running installed applet. Do not touch user configuration or
  unrelated processes.

### Recipes

Run checks through `bash scripts/dev.sh <task>` (see the routine loop above).
Tasks: `fmt-check`, `localization-check`, `check`, `lint`, `test`, `gate`,
`verify`, `metadata-check`, `metadata-check-strict`, `build`, `build-release`.
`just fmt` applies formatting. The wrapper never installs, cleans, launches or
publishes.

- `metadata-check` ignores known COSMIC-specific validator warnings so `verify`
  stays usable. Run `metadata-check-strict` for the freedesktop-only result and
  record any warnings with their rationale.
- Without a recipe, use the direct Cargo commands in the Rust verification
  procedure above.

### Documentation

Keep ${DESCRIPTION_FILE}, ${SPECIFICATION_FILE}, ${TASK_LIST_FILE} and
${COMPLETION_NOTES_FILE} at the root, user guides in ${USER_DOCS_DIR}, and release
notes in ${RELEASE_NOTES_DIR}. When paths change, update links and indexes
together. Layout and release-note rules are in the procedures record linked below.

### Packaging and release entry points

Native, Debian, Flatpak and GitHub release bindings (`just stage`, `just deb`,
`scripts/build-flatpak.sh`, `scripts/run_github_linux_build.sh`,
`scripts/build_release.sh`, `scripts/promote_release.py`, publication workflow)
are documented with their approval boundaries in the
[release and packaging procedures](docs/engineering/release-packaging-procedures-contracts.md).
Installation (`just install-user`, APT) and publication need explicit approval.
