# Cloud Mounter Applet Task List

This document tracks the complete work-item checklist, including historical,
superseded, completed, and current tasks. Detailed execution evidence, manual
test notes, command output summaries, and completion commentary are kept in
`Task List Completion Notes.md` using a parallel phase/gate structure.

**Status:** Current Gate 1 is approved. Current Phase 9 UI/runtime completion,
Current Phase 10 integrated manual testing, and Current Phase 11 documentation
and packaging recheck are complete. Current Gate 2 remains open pending final
release-candidate checks and user approval.

**Execution environment:** VS Code with Codex

This list implements `Requirements and Specifications.md`. The approved source
description is `Applet Description.md`.

## GitHub Debian release automation

**Feasibility verified October 7, 2026.** GitHub-hosted native amd64 and arm64
runners are available for Ubuntu 22.04, 24.04, and 26.04. A COSMIC desktop
session is not required to compile or package the applet; Rust, libcosmic's
native development headers, and Debian packaging tools are sufficient. Ubuntu
22.04 runners entered deprecation on September 17, 2026 and are scheduled for
retirement on April 17, 2027.

### Chronological implementation and cloud validation

- [x] Add a six-entry GitHub Actions matrix for Ubuntu 22.04, 24.04, and 26.04
  on native amd64 and arm64 GitHub-hosted runners.
- [x] Install the pinned Rust 1.95.0 toolchain, libcosmic build headers, and
  Debian packaging tools on each runner.
- [x] Run the package's existing Debian build and test path with
  `dpkg-buildpackage -us -uc -b`.
- [x] Inspect the resulting package architecture, add the Ubuntu version to
  release filenames, generate SHA-256 checksums, and retain workflow artifacts.
- [x] On `v*` tags, create a GitHub release only after every matrix build passes
  and upload all packages plus a combined `SHA256SUMS` file.
- [x] Add a local workflow trigger/download helper and a release script that
  validates versions, a clean/pushed commit, and tag uniqueness before pushing
  the release tag.
- [x] Commit and push the initial workflow to the default branch.
- [x] In GitHub **Settings > Actions > General**, confirm GitHub Actions and
  GitHub-maintained actions are allowed for the repository.
- [x] Confirm the repository workflow-permission setting permits the tagged
  release job's requested `contents: write` access.
- [x] Start the first manual **Build Debian packages** run and confirm all six
  Ubuntu/architecture jobs are admitted by the runner policy.
- [x] Diagnose the first six-job cloud run: every job stopped at
  `dpkg-checkbuilddeps` because the runner did not install the declared
  `cargo`, `rustc`, `desktop-file-utils`, and `appstream` build dependencies.
- [x] Add all four declared packages to the workflow installation step. The
  apt Rust packages satisfy Debian dependency checking; the subsequent rustup
  step still selects the project's pinned Rust 1.95.0 toolchain for the build.
- [x] Rerun **Build Debian packages**; all six jobs passed dependency checking
  and reached the same test-compilation failure described next.
- [x] Diagnose the second six-job run: test compilation referenced ignored
  local files under `archive/services`, which do not exist in a clean checkout.
- [x] Replace the local archive dependency with neutral, tracked legacy-service
  fixtures under `tests/fixtures/legacy-services`; remove account-specific
  names and paths while retaining Box, SMB, and Google Drive import coverage.
- [x] Pass formatting, diff checks, and all 12 focused import tests with the
  tracked fixtures (180 other tests filtered by the focused invocation).
- [x] Commit and push the tracked fixtures and import-test changes.
- [x] Complete the third **Build Debian packages** run successfully on all six
  Ubuntu/architecture jobs after adding the tracked fixtures.
- [x] Download the Ubuntu 24.04 amd64 artifact from successful run
  `37642473346`; verify its SHA-256 checksum, package/version/architecture,
  runtime dependency metadata, and installed-file layout.
- [x] Back up Cloud Mounter configuration, rclone configuration, generated
  services, and local installation files before package testing.
- [x] Install the Ubuntu 24.04 amd64 package on Pop!_OS 24.04 and confirm the
  packaged `/usr/bin/cosmic-ext-applet-mounter` replaces the parked user-local
  executable during the test.
- [x] Live-test the packaged applet UI, preserved connections, General
  Settings, and OneDrive mount, unmount, and remount behavior.
- [x] Review the live journal: no applet crash, panic, failed service exit, or
  mount failure occurred. onedriver continued successfully after its optional
  missing-config warning and one unsupported FUSE `STATX` request.
- [x] Unmount all test connections, remove the Debian package without purging
  user data, restore both parked user-local executables, and confirm command
  resolution returned to `~/.local/bin` with no Cloud Mounter mounts left.
- [x] Re-authenticate GitHub CLI and use it successfully to enumerate run
  `37642473346`, list all six artifacts, and download the 24.04 amd64 package.
- [x] Complete structural validation for all six packages in the workflow;
  each matrix job checked the package architecture and metadata and generated
  a matching SHA-256 checksum. Live installation was completed on the available
  Pop!_OS 24.04 amd64 COSMIC system; other OS/architecture live checks require
  matching external systems and are not part of this release gate.

### Remaining for the next release

- [x] Commit and push `.gitignore`, `scripts/build_release.sh`,
  `scripts/run_github_linux_build.sh`, and this final task-list update.
- [x] Confirm current release metadata agrees at version 0.5.1 in
  `Cargo.toml`, the root package in `Cargo.lock`, `debian/changelog`, AppStream
  releases and screenshot URLs, README download commands, the Flatpak source
  tag and packaging guide, local tag `v0.5.1`, and the published GitHub release.
- [x] Prepare version 0.5.2 and update Cargo, Debian, AppStream, README,
  Flatpak, and release-note metadata together.
- [x] Push the version 0.5.2 release commit, pass
  `scripts/build_release.sh --dry-run`, then run `scripts/build_release.sh` to
  tag and build release `v0.5.2`. All six jobs in run `37649661706` passed and
  the downloaded packages were verified as version 0.5.2 with the expected
  architectures. Recovered the failed final publish step by giving GitHub CLI
  explicit repository context, then published the six packages and combined
  `SHA256SUMS` file to the `v0.5.2` GitHub release.
- [ ] Update GitHub actions that still target the deprecated Node.js 20 runtime;
  GitHub currently forces `actions/checkout@v4` and
  `actions/upload-artifact@v4` to run on Node.js 24.
- [ ] Remove the Ubuntu 22.04 matrix entries no later than GitHub's scheduled
  runner retirement on April 17, 2027.

## Release 0.4.6

- [x] Align Cargo, Debian, AppStream, README installation examples,
  screenshots, and the Flatpak source tag with version 0.4.6.
- [x] Run formatting, compilation, all-target/all-feature Clippy with warnings
  denied, the complete 194-test suite, metadata checks, and diff validation.
- [x] Build and inspect the amd64 Debian installation asset and prepare its
  SHA256SUMS file.
- [x] Record the final release scope and validation in Task List Completion
  Notes.
- User authorized commit, push, tag and GitHub release publication for
  `v0.4.6` with the installation asset.

## Online directory-cache optimization and Google OAuth client work

**Planning recorded September 15–16, 2026. The OAuth client, Google Drive
refresh-level fast-list, asynchronous Google Drive VFS refresh, and bounded
OneDrive directory preload are implemented. Live Box and SMB prototypes reject
recursive VFS refresh and approve bounded directory walks for implementation.**

### Approved Google Drive and OneDrive work

- [x] Add matching Google Drive custom OAuth client ID and client-secret fields
  to the rclone remote create/update workflow. Keep both values out of applet
  configuration, redact them from command/error output, clear the fields after
  use, and preserve existing remotes unless the user explicitly updates one.
  Existing-remote update repeats browser OAuth and advises remounting active
  connections.
- [x] Enable fast-list on Google Drive recursive VFS refresh requests and add
  tests proving mount commands do not receive rclone's ignored `--fast-list`
  argument. Regenerate the applet-owned unit before starting a mount so existing
  saved Google Drive connections receive the private RC endpoint after an
  unmount/remount.
- [x] Measure whole-remote recursive directory-list latency and peak memory on
  the configured `ua_gdrive:` remote. Five trials per mode returned identical
  510-directory results; `--fast-list` showed no meaningful latency or memory
  improvement under the measured conditions.
- [x] After Google Drive mount and RC-socket readiness, run recursive
  `vfs/refresh` as an asynchronous RC job. Retain its job ID and track its
  connection and completion state.
- [x] After onedriver mount readiness, run a directory-only background traversal
  that neither opens file contents nor follows links outside the mount.
- [x] Add an app-wide OneDrive preload-time setting to General Settings. Default
  to 60 seconds, validate finite bounds, stop automatically at the deadline,
  and provide no manual Cancel control.
- [x] Use the existing notice path to report OneDrive preload
  completion or timeout without adding persistent notification machinery.
- [x] Cancel and finish a tracked Google Drive refresh before manual unmount,
  connection removal, repair, and sleep cleanup. Keep cancellation and detach
  bounded.
- [x] Cancel and finish tracked OneDrive preload work before manual unmount,
  connection removal, repair, and sleep cleanup. Keep cancellation and detach
  bounded.
- [x] Add unit/integration coverage for provider selection, command construction,
  background lifecycle, timeout, cancellation, setting defaults, and bounds.
Further Google Drive and cloud cold-cache trials are tracked under Deferred
validation after the SFTP native release.

### Approved provider preload settings and remaining implementation

- [x] Fix clean-unmount completion for all Online providers. Do not prepare a
  mountpoint during unmount; verify mount-table disappearance after service
  stop; classify an inactive/failed service with a lingering FUSE entry as
  Error so the confirmed Repair flow is offered; and skip `reset-failed` when
  the service is already inactive with a successful result.

- [x] Test Box recursive `vfs/refresh` without fast-list. Reject it after the UA
  Box prototype reached HTTP 429 `rate_limit_exceeded` and retained little useful
  metadata.
- [x] Test an SMB recursive VFS refresh through Cisco VPN. Reject it because the
  60-second cancellation did not promptly stop backend work and partial cache
  progress was not retained.
- [x] Test bounded directory walks. Approve Box depth 2 after a 16.67-second cold
  and 0.05-second warm traversal cached 212 directories without a new 429.
  Approve SMB depth 3 after a 15.84-second cold and 0.10-second warm traversal
  cached 491 directories with no file-content reads.
- [x] Replace the single OneDrive preload control with a General Settings
  **Preload** section. Add enabled and 5-to-600-second maximum controls for all
  providers; add depth 2 for Box and depth 3 for SMB. Default all providers to
  enabled and 60 seconds.
- [x] Migrate the existing OneDrive duration into the new provider-policy model
  and initialize missing Google Drive, Box, and SMB values safely.
- [x] Add SMB per-connection **Use global preload settings**, enabled by default,
  with independent enabled, duration, and depth overrides.
- [x] Implement mount/root readiness and tracked directory-only walks for Box
  and SMB. Preserve incremental progress on timeout and cancel before unmount,
  repair, removal, and sleep.
- [x] Harden Google RC completion parsing so any non-`OK` `output.result` value
  is a failure even when rclone returns top-level `success: true`. Enforce the
  configured Google maximum.
- [x] Add configuration, migration, IPC, UI, command-boundary, timeout,
  cancellation, nested-RC-error, and effective-SMB-policy tests.
The remaining UA Box, narrowed `ua_engr:Research/Utzinger` SMB, and LAN/home
SMB live checks are tracked under Deferred validation.

## Rclone mount access gate and multi-panel runtime fix (after 0.4.5)

- [x] Diagnose UA Box `Input/output error` from the managed-service journal:
  Box rejected the expired OAuth refresh token with `invalid_grant`.
- [x] Reauthorize `ua_box` and verify read-only root access.
- [x] Require a bounded remote/subtree listing before every applet-requested
  rclone Online mount, including wake restoration; do not start FUSE when the
  provider, credentials, network, or subtree is unavailable.
- [x] Treat additional Cloud Mounter panel instances as clients of the existing
  runtime owner instead of showing a false Settings communication warning.
- [x] Pass formatting/diff checks and all 178 tests; install the repaired binary.
- [x] Live-verify Box listing, healthy VFS state, clean unmount, and a responsive
  empty mountpoint after unmount.

## Settings background-poll UI fix (after 0.4.5)

- [x] Keep Refresh and sleep toggles visually stable during background polling.
- [x] Discard obsolete poll replies when a newer user action has started.
- [x] Verify background polling, reply ordering and stale error handling with
  regression tests; update design and completion documentation.
The installed desktop button-stability check is tracked under Deferred
validation.

## Release 0.4.5

- [x] Align design documentation and README with the final Settings/popup layout.
- [x] Bump Cargo, Debian, AppStream, README examples and Flatpak tag to 0.4.5.
- [x] Pass release-build validation and all 174 tests; inspect the amd64 Debian
  package and prepare its SHA256SUMS for publication.
- User authorized commit, push and publication of tag/release `v0.4.5`.

## Compact Popup and General Settings Window

**September 15, 2026 — implemented and locally tested; desktop/Flatpak live checks pending.**
This revision supersedes previous popup placement of Add Connection, Refresh
and sleep toggles. Earlier completed layout tasks remain historical evidence.

- [x] Inspect title/button APIs, icon availability, separate-window architecture,
  shared notices, Refresh behavior and runtime ownership.
- [x] Update Applet Description, requirements, Tooltip Review and Completion
  Notes; incorporate the preceding rclone remote-name help correction.
- [x] Add right-aligned Settings gear with keyboard/accessible label, tooltip and
  bundled icon independent of the installed icon theme. Keep title/status/connections and meaningful runtime notices.
- [x] Add/focus one standalone General Settings window with a distinct launch
  mode/title; retain existing connection-editor launch arguments and behavior.
- [x] Implement runtime-owner IPC for settings, Refresh, current sleep status
  and saved connection changes; review native/Flatpak service-name permissions.
- [x] Move sleep toggles into General Settings; keep Restore visible but disabled
  when Unmount when sleep is off. Persist through the applet owner without
  duplicating its listener or changing Online-only cleanup semantics.
- [x] Move Add Connection and Refresh to the top of General Settings, above
  the sleep options. Refresh must reach
  the actual applet; successful editor saves must refresh its connection list.
- [x] Separate settings-action notices from popup runtime notices. Preserve
  mount/unmount/repair errors and sleep warnings when settings is closed, and
  keep provider/authentication/test/save feedback in the connection editor.
- [x] Remove toolbar/footer and their popup height reservation. Update empty
  state to direct users to Settings > Add Connection.
- [x] Make rclone help context-sensitive: new-name/Create guidance in Add;
  existing-name guidance in Modify where creation buttons are hidden.
- [x] Test routing, owner-unavailable errors, persistent setting acknowledgments,
  refresh completion, notice preservation and one-listener lifecycle.
- [x] Match General Settings to the themed list surface; put button notices
  below the top actions and plain sleep feedback below the Online-only sentence.
  Remove the extra Offline sentence and separate cleanup-status heading/box.
- [x] Align the title left and Settings gear right; replace the status/list
  background gap with a thin divider on a continuous themed surface.
Broader native/Flatpak window and editor visual checks are tracked under
Deferred validation.

## Rclone Remote Name Help Clarification

**September 15, 2026 — source and documentation updated.**

- [x] Explain entering a new remote name before creation in the Google Drive,
  Box, and SMB field tooltips; retain detected/existing remote guidance.
- [x] Identify provider-specific Create buttons and SMB Create/Update behavior.
- [x] Update Applet Description, FR-011MA, Tooltip Review and Completion Notes.
- [x] Review the three strings against creation code and run formatting/diff
  checks. This text-only change does not alter provider behavior.

## VPN Authentication Wait and Sleep Cleanup

**Requested September 14, 2026. Status: A/B runtime implemented locally;
live acceptance and additional VPN UI work remain open.**
Requirements: FR-086 through FR-087A (A), FR-097 through FR-102 (B).

- [x] Update Applet Description, Requirements and Specifications, and Completion
  Notes; constrain sleep cleanup to Online connections only.
- [x] A: Fix the Cisco GUI lifecycle without increasing the 90-second default.
  Preserve launch behavior through the Flatpak host runner; report spawn errors.
- [x] A: Preserve customized VPN timeouts/readiness checks on detection.
- [x] A: Bound activation and full readiness by one deadline, retry transient
  probe failures, serialize shared-profile activation and cancel Online starts
  for sleep. Recheck connection changes after authentication.
- [x] A: Test interactive child survival past its command timeout, Flatpak launch
  wrapping, transient readiness failure and hung-probe deadline handling.
- [ ] A: Add dedicated waiting/progress, Cancel and Retry UI (existing mount
  control remains the retry action). Add explicit authentication-rejection UI.
The live NetworkManager/Cisco authentication workflow is tracked under
Deferred validation.
- [x] B: Add persistent, default-off sleep cleanup toggle below the main popup
  connection list and optional wake restoration beneath it.
- [x] B: Implement logind listener/delay inhibitor independent of popup visibility,
  finite cleanup budget, inhibitor reacquisition and visible degraded status.
- [x] B: Select Online connections only, verify unit/filesystem ownership, cancel
  Online operations and wait for them to leave before cleanup. Leave Offline
  mirrors, sync jobs and shared VPNs untouched.
- [x] B: Use nonforcing runtime service drop-ins, verify loaded stop policy, check
  rclone upload queues, and verify host mount disappearance without traversal.
- [x] B: Restore successfully cleaned connections only when explicitly enabled,
  after network/VPN checks; preserve login policy and report incomplete cleanup.
- [x] B: Test Online-only selection, mirror-only no-op, repeated sleep gate,
  old-operation invalidation, deadlines, queue-state parsing, foreign mounts,
  runtime drop-in ownership and effective stop-policy checks.
The broader logind fault-cycle checks and OneDrive sleep-hang investigation
are tracked under Deferred validation. The next native SFTP release gate
includes one normal SFTP sleep/wake cycle.
- [x] Final local formatting, all-target checks, warning-free Clippy, all 167
  tests and debug build passed; results are recorded in Completion Notes.

## Codex Working Rules

- [x] Work one phase at a time and keep this list current.
- [x] Inspect current files and preserve unrelated user changes.
- [x] Use small, reviewable patches.
- [x] Never place real credentials, tokens, or organization-specific secrets in
  source or fixtures.
- [x] Use temporary directories and fake adapters for automated tests.
- [x] Run each phase's verification before reporting completion.
- [x] Report changed files, tests, failures, and remaining risks.
- [x] Stop at each approval gate and wait for explicit approval.

## Gate 0: Review and Approval

- [x] Limit providers to OneDrive, Google Drive, Box, and SMB.
- [x] Include Online mount and Offline mirror in version 0.1.
- [x] Select provider-specific mount and synchronization engines.
- [x] Define bidirectional synchronization, conflict, deletion, recovery,
  metered-network, and initial-preview behavior.
- [x] Approve compatible legacy service import from
  `~/.config/systemd/user/`.
- [x] Approve a configurable 20 GiB default rclone mount cache.
- [x] Select package name, planned app ID, MIT license, and authors.
- [x] Offer lazy unmount only after clean unmount fails and explicit
  confirmation; block it when writes are pending.
- [x] Disconnect only applet-activated VPNs after all dependent connections are
  inactive.
- [x] Review the revised requirements and this task list.
- [x] Record explicit user approval to begin development.

## Phase 1: Baseline and Scaffold

- [x] Inventory the supplied applet template, scripts, service fixtures, and
  available local tool versions.
- [x] Lock a compatible libcosmic revision and Rust toolchain.
- [x] Instantiate the template as `cosmic-ext-applet-mounter`.
- [x] Set app ID `io.github.uutzinger.cosmic-ext-applet-mounter`.
- [x] Add MIT license and authors Urs Utzinger and OpenAI Codex.
- [x] Set the user-created repository URL without creating or publishing the
  repository from Codex.
- [x] Create the approved module layout.
- [x] Add formatting, lint, build, run, test, and staged-install recipes.
- [x] Add a developer README for VS Code and command-line workflows.
- [x] Use the existing user-created Git repository without reinitializing it.
- [x] Run a manual COSMIC popup open/close smoke test.

### Phase 1 verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --all-targets`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-targets`
- [x] Manual popup open/close test.

## Phase 2: Domain Model and Configuration

- [x] Define providers, modes, connections, VPN profiles, operations, conflicts,
  recovery records, and status types.
- [x] Implement stable UUID connection identities.
- [x] Implement versioned COSMIC configuration.
- [x] Implement atomic writes, rollback, migration, and malformed-data recovery.
- [x] Validate local and remote paths.
- [x] Reject mount/mirror reuse, nested mirrors, recursive sync trees, duplicate
  targets, and unsafe system paths.
- [x] Add configuration fixtures without real credentials.

### Phase 2 verification

- [x] Test serialization, migration, defaults, and invalid data.
- [x] Test all path-overlap and mode-separation rules.
- [x] Confirm configuration contains no secret fields.

## Phase 3: Process, Dependency, and Diagnostics Layer

- [x] Define an asynchronous typed command-runner interface.
- [x] Invoke fixed executables with separate validated arguments.
- [x] Bound output, duration, retries, and cancellation.
- [x] Redact credentials, tokens, URLs, and sensitive arguments.
- [x] Detect rclone, `jstaf/onedriver`, `abraunegg/onedrive`, FUSE, NetworkManager, Cisco,
  and optional diagnostics tools.
- [x] Validate required capabilities as well as versions.
- [x] Reject rclone `1.60.1` and provide current-stable upgrade guidance.
- [x] Add fake runners and dependency inventories.

### Phase 3 verification

- [x] Test missing, outdated, and capability-incomplete tools.
- [x] Test timeout, nonzero exit, invalid UTF-8, large output, and cancellation.
- [x] Security-review all external command construction.

## Phase 4: Systemd and Runtime Infrastructure

- [x] Define service and timer manager interfaces.
- [x] Render deterministic applet-owned units with UUID markers.
- [x] Install, validate, update, roll back, and remove units atomically.
- [x] Implement daemon reload, enable, disable, start, stop, reset-failed, and
  status.
- [x] Define mount-table and sync-runtime interfaces.
- [x] Implement per-connection operation serialization.
- [x] Build fake systemd, mount-table, and timer implementations.

### Phase 4 verification

- [x] Snapshot-test generated units and timers.
- [x] Test ownership checks and rollback.
- [x] Confirm tests never change the real user manager.

## Phase 5: Online Mount Engines

### Rclone mount

- [x] Enumerate and validate Google Drive, Box, and SMB remotes.
- [x] Implement provider-specific remote/subtree configuration.
- [x] Render rclone mount services with full VFS caching.
- [x] Apply the configurable 20 GiB default cache limit.
- [x] Apply bounded timeout and retry defaults.
- [x] Enable VFS queue and cache health inspection.
- [x] Detect queued uploads, active uploads, cache errors, and cache exhaustion.

### `jstaf/onedriver`

- [x] Detect current `jstaf/onedriver` capabilities and authentication state.
- [x] Implement OneDrive Online mount setup and service management.
- [x] Detect cached read-only offline behavior and mount health.

### Connectivity recovery

- [x] Monitor network, VPN, provider, service, and actual mount state.
- [x] Safely auto-detach rclone mounts only when no writes are pending.
- [x] Preserve cache and expose recovery when writes are pending.
- [x] Implement automatic remount for enabled connections after readiness returns.
- [x] Add bounded exponential remount backoff.
- [x] Implement clean unmount and busy-mount diagnostics.
- [x] Offer warned, explicit-confirmation lazy unmount after clean unmount fails.
- [x] Block lazy unmount while writes are queued or in progress.

### Phase 5 verification

- [x] Test clean mount/unmount and partial service/mount states.
- [x] Test safe detach, blocked detach, cache error, and reconnect/remount.
- [x] Test file-manager-facing operations return within configured bounds during
  simulated connectivity failure.

## Phase 6: Offline Mirror Engines

### Shared synchronization behavior

- [x] Select whole remote or remote subtree.
- [x] Estimate remote size and local free-space requirements.
- [x] Generate a dry preview of upload, download, delete, conflict, skip, and
  transfer totals.
- [x] Require explicit confirmation before initial synchronization.
- [x] Implement Sync Now, Pause, Resume, and automatic reconnect sync.
- [x] Schedule non-continuous engines every 15 minutes after completion.
- [x] Prevent concurrent runs.
- [x] Pause automatic sync on metered networks by default.
- [x] Implement per-connection metered override.
- [x] Propagate creates, modifications, renames, and deletions bidirectionally.
- [x] Preserve both conflict versions and report them.
- [x] Retain deleted and overwritten files for 30 days.
- [x] Keep cache, work, and recovery directories outside the mirror tree.
- [x] Implement interrupted-run recovery without routine destructive resync.
- [x] Require preview and confirmation for any state rebuild or resync.

### Rclone bisync

- [x] Implement dedicated work state per connection.
- [x] Require supported access checks and resilient/recovery capabilities.
- [x] Configure conflict-loser preservation and recovery directories.
- [x] Implement Google Drive cloud-native document exclusion and reporting.
- [x] Add Google Drive, Box, and SMB adapters.

### OneDrive mirror

- [x] Detect current `abraunegg/onedrive` capabilities.
- [x] Detect and block overlap with configured or active `jstaf/onedriver` accounts and
  remote subtrees.
- [x] Create isolated configuration and sync directories per connection.
- [x] Implement supported browser authentication.
- [x] Use monitor mode for continuous synchronization where supported.
- [x] Map native conflict, recovery, status, and resync-required states into the
  applet model.

### Phase 6 verification

- [x] Test initial empty/local/remote/both-populated cases.
- [x] Test offline local edits followed by reconnect.
- [x] Test remote-only changes and deletions.
- [x] Test simultaneous same-file edits preserving both versions.
- [x] Test interrupted runs and non-destructive recovery.
- [x] Test 30-day recovery cleanup boundaries.
- [x] Test low disk space and metered-network behavior.
- [x] Test Google cloud-native document exclusion.

## Phase 7: VPN Integration

### NetworkManager

- [x] Enumerate visible VPN profiles.
- [x] Implement activation, state, and deactivation through a typed
  NetworkManager adapter.
- [x] Document and test the fixed-argument `nmcli` fallback; direct D-Bus
  remains the preferred future transport.

### Cisco Secure Client

- [x] Detect agent, GUI, interface, and tunnel state.
- [x] Implement authorized agent startup without storing sudo credentials.
- [x] Open the Cisco GUI for interactive authentication.

### Coordination

- [x] Implement interface, route, DNS, endpoint, and NetworkManager readiness
  checks.
- [x] Block mount and sync until readiness succeeds or times out.
- [x] Reference-count shared VPN dependencies.
- [x] Track whether the applet activated each VPN.
- [x] Never disconnect a VPN still used by another connection.
- [x] Disconnect an unused VPN automatically only when the applet activated it.

### Phase 7 verification

- [x] Test shared, pre-existing, applet-activated, failed, and timed-out VPNs with
  fakes.
- [x] Test mount and sync failure after successful VPN activation.
- [x] Do not activate a real VPN before the manual-test gate.

## Phase 8: Legacy Service Import

- [x] Scan `~/.config/systemd/user/` by default.
- [x] Parse the compatible subset of rclone and `jstaf/onedriver` units as structured
  data.
- [x] Never execute imported text.
- [x] Display provider, remote, target, cache, startup, and unsupported options.
- [x] Detect active-service and local-target conflicts.
- [x] Require confirmation before creating an applet-owned replacement.
- [x] Preserve original units by default.
- [x] Offer a separate confirmed action to disable an imported original.
- [x] Test against copies of all files in repository `services/`.

### Phase 8 verification

- [x] Confirm fixture and original units remain unchanged.
- [x] Confirm import cannot inject commands or copy credentials.
- [x] Test malformed, unsupported, duplicate, and conflicting units.

## Phase 9: Operation Controller and UI

- [x] Implement separate Online mount and Offline mirror state machines.
- [x] Restore status from configuration, systemd, mount tables, sync state,
  connectivity, and VPN state.
- [x] Implement popup aggregate state and connection rows.
- [x] Implement UI-facing operation model and labels for mount/unmount, Sync
  Now, Pause/Resume, retry, repair, and details.
- [x] Implement provider- and mode-specific settings data in the domain model.
- [x] Implement disk estimate and initial-sync preview/confirmation models.
- [x] Implement pending-write, conflict, recovery, low-space, and metered states.
- [x] Implement dependency setup and upgrade guidance messages.
- [x] Implement legacy import preview display.
- [x] Add sanitized logs and optional notifications.
- [x] Add accessible labels and localization for implemented popup views.

### Phase 9 verification

- [x] Test UI model updates with fake backends.
- [x] Check narrow and wide panel layouts.
- [x] Check keyboard-only operation.
- [x] Check that every state is understandable without color.

### Phase 9 UI/settings addendum

- [x] Add a popup Settings control that opens the settings window.
- [x] Add an empty-state Add connection control that opens settings in
  add-connection mode.
- [x] Implement settings window lifecycle, close behavior, and navigation.
- [x] Implement settings connection list with Add, Edit, Test, Enable, Disable,
  Import, and Remove actions.
- [x] Implement provider- and mode-specific settings forms for provider, mode,
  account/remote, remote subtree, local mountpoint or mirror directory, display
  name, and enablement.
- [x] Implement Online mount settings for manual startup default, optional
  startup at login, 20 GiB default rclone VFS cache limit, timeouts, retries,
  bandwidth, safe detach behavior, and lazy-unmount confirmation policy.
- [x] Implement Offline mirror settings for whole-drive or subtree selection,
  disk estimate, initial-sync preview, sync interval, Sync Now, Pause/Resume,
  metered-network behavior, recovery location, 30-day retention, conflict
  preservation, and resync/state-rebuild confirmation.
- [x] Implement VPN settings for NetworkManager or Cisco dependency selection,
  readiness checks, startup behavior, and shutdown limited to applet-activated
  VPNs.
- [x] Implement dependency settings showing executable detection, versions,
  unsupported or outdated tools, authentication/setup guidance, and upgrade
  guidance without installing dependencies.
- [x] Implement legacy import settings workflow for scanning
  `~/.config/systemd/user/`, previewing compatible services, showing conflicts,
  confirming applet-owned replacement creation, preserving originals by default,
  and separately confirming original disablement.
- [x] Implement confirmation dialogs for initial synchronization, state rebuild,
  destructive resync, removal, disabling imported originals, lazy unmount, and
  cleanup.
- [x] Replace text-only popup operation labels with clickable row controls for
  Mount, Unmount, Sync Now, Pause, Resume, Retry, Repair, and Details.
- [x] Show disabled or unavailable popup actions with visible reasons.
- [x] Wire popup and settings controls to typed operation requests without
  blocking the COSMIC event loop.
- [x] Add keyboard navigation, accessible names, localization, and non-color
  state cues for settings and popup action controls.

### Phase 9 addendum verification

- [x] Manually test that popup Settings opens settings and closing settings does not
  interrupt active operations.
- [x] Manually test that empty-state Add connection opens settings in add-connection
  mode.
- [x] Test settings add, edit, test, enable, disable, import, and remove flows
  with fake backends.
- [x] Test that settings expose all approved provider, mode, local path, cache,
  sync, metered, VPN, dependency, import, recovery, and confirmation options.
- [x] Test popup action controls for Online mount and Offline mirror rows,
  including disabled-state explanations.
- [x] Manually check keyboard-only operation and accessible labels for settings and popup
  action controls.
- [x] Check that action dispatch remains asynchronous and does not block UI
  rendering.

### UI Design Completion

- [x] Replace fake-backend-only UI notices for Refresh, Import preview, Test,
  Enable, Disable, and Remove with managed backend integration.
- [x] Connect Refresh to configuration reload.
- [x] Connect Import to real `~/.config/systemd/user/` legacy service scan and
  structured import previews.
- [x] Connect Test to managed service/timer plan construction and structural
  validation for Online mount and Offline mirror connections.
- [x] Connect Enable/Disable to validated COSMIC configuration writeback.
- [x] Connect Remove to a two-step validated configuration removal that preserves
  credentials, local data, cache, recovery data, and external services.
- [x] Replace the global Settings primary entry point with one Add Connection
  control, per-row Edit controls, Refresh, and Add-window Import.
- [x] Implement the main popup as an operational dashboard: aggregate status,
  connection rows, and bottom Add Connection plus Refresh controls.
- [x] For each existing connection row, display name, provider, mode, engine,
  local target, remote/subtree, VPN dependency, enabled state, runtime state,
  last sync where applicable, warnings, and operation buttons.
- [x] Keep Import previews out of the main connection status list until the user
  confirms creating applet-managed connections.
- [x] Implement Add/Modify wizard provider selection for OneDrive, Google Drive,
  Box, and SMB.
- [x] Implement Add/Modify wizard mode selection for Online mount versus Offline
  mirror, constrained by the approved provider/mode matrix.
- [x] Implement Add/Modify wizard account or remote selection, including
  `jstaf/onedriver` account, `abraunegg/onedrive` account, rclone remote, and SMB rclone
  remote.
- [x] Implement Add/Modify wizard whole remote versus remote subtree selection.
- [x] Implement Add/Modify wizard local mountpoint or mirror directory selection
  with mount/mirror separation validation.
- [x] Implement Add/Modify wizard mode-specific settings for Online mount and
  Offline mirror.
- [x] Implement Add/Modify wizard per-connection VPN dependency, readiness
  checks, applet activation permission, and applet-owned disconnect policy.
- [x] Implement Add/Modify wizard dependency status, generated unit/command
  preview, disk estimate or mount validation, safety warnings, and confirmation.
- [x] Save Add/Modify wizard results through validated configuration and managed
  unit planning.
- [x] Implement Import as a dedicated workflow that maps each compatible legacy
  service preview into the Add/Modify wizard fields before confirmation.
- [x] Implement confirmed Import replacement creation through the managed unit
  backend.
- [x] Implement confirmed removal of applet-owned generated units after the
  configuration removal confirmation flow is extended for unit cleanup.
- [x] Replace standalone Help controls with attached per-field tooltip help and
  keep longer dependency, safety, and troubleshooting guidance in documentation.
- [x] Update Phase 9 and Gate 2 verification criteria after UI Design Completion
  is approved.

### Main Applet UI User-Guided Cleanup

- [x] Verify the popup behaves with enough configured connections to exceed the
  available panel popup height.
- [x] Add a scrollable connection-list region so every configured connection can
  be reached when the list is longer than the popup.
- [x] Keep the title and aggregate status visible above the scrollable
  connection-list region.
- [x] Keep the main popup aggregate status to three concise lines: active
  connection count, notification state, and VPN summary/state.
- [x] Decide whether Add Connection and Refresh stay at the bottom or move below
  the status text at the top to make the remaining popup area a simpler
  scrollable connection list.
- [x] If Add Connection and Refresh move to the top, update the approved main
  popup layout notes and remove stale references to a bottom action row.
- [x] Replace the current multi-line popup connection rows with a compact
  single-line row model: clickable name opens Modify, one compact state control
  handles Mount/Unmount for Online Mount and Start/Stop for Offline Mirror, and
  detailed state is conveyed by label/color/non-color cue/tooltip/disabled
  reason rather than a separate wide status chip.
- [x] Move Offline Mirror secondary actions Preview and Sync Now out of the
  cramped main popup and into the Add/Modify connection action area or a
  per-connection diagnostics/action surface.
- [x] Re-check compact control sizing, text wrapping/elision, and clipped text
  with long connection names and multiple connection rows.
- [x] Reinstall the applet and complete user-guided visual review of the main
  popup before returning to provider/runtime work.

### Settings (Add/Modify) UI User-Guided Cleanup

- [x] Update the Add Connection instruction/notice text to say: "Choose
  provider, mode, remote/subtree, local target, VPN, and start at login." Reuse
  the same top notice area for later instructions, validation results, and
  operation status messages.
- [x] Reduce the horizontal gap between each section title and its controls.
- [x] Top-align section titles with the first control in their section instead
  of vertically centering the title across tall multi-control sections.
- [x] Align the Provider button row with the Access mode button row. The
  OneDrive provider button should visually line up with Online mount and
  Offline mirror rather than sitting on a different horizontal grid.
- [x] Move Connection-section controls left so remote/account/subtree fields
  align with the Access mode controls.
- [x] Move Mountpoint and Mirror directory text boxes left so they align with
  the other main input fields.
- [x] Move Start at login and cache-size controls left so they align with the
  other main input fields.
- [x] Replace the Start at login Yes/No button with the same COSMIC toggler
  style used in the main popup.
- [x] Move "No detected Box rclone remotes..." and equivalent provider-empty
  guidance into the Detect Rclone Remotes tooltip or notice area instead of
  leaving it as inline body text.
- [x] Move Create Box Remote and Create Google Drive Remote browser/OAuth
  guidance into the Create Remote tooltip or top notice area.
- [x] Hide Create Box Remote, Create Google Drive Remote, and Create SMB Remote
  when modifying an existing connection. Provider remote creation belongs to Add
  Connection only.
- [x] Move Create Remote actions into the top action row alongside Test
  Connection.
- [x] Move Detect rclone remotes into the top action row alongside Test
  Connection, and show it only in Add Connection mode, not Modify mode.
- [x] Make Test Connection and Save Connection visually non-primary until the
  required Create Remote flow succeeds or an existing valid remote is selected.
- [x] Remove inline "Box setup: use existing..." and equivalent random-looking
  setup paragraphs from the Connection section. Keep necessary guidance in the
  top notice area or attached tooltips.
- [x] Supersede the earlier Create SMB Remote bottom-of-section placement:
  Create SMB Remote now follows the unified Add-mode top action row policy.
- [x] Remove or tooltip-integrate extra SMB setup guidance text in the
  Connection section while keeping the SMB host/user/domain fields available in
  Add mode.
- [x] Remove or tooltip-integrate extra OneDrive Online Mount and OneDrive
  Offline Mirror guidance text in the Connection section.
- [x] Remove or tooltip-integrate extra OneDrive Offline Mirror guidance text in
  Offline mirror settings.
- [x] Simplify the VPN selector: remove visible group titles for NetworkManager
  and Cisco, arrange VPN choices horizontally where space allows, and place
  Detect VPNs last on the same selector line.
- [x] Ensure Cisco remains self-labeled as Cisco and NetworkManager profiles are
  understandable from their profile names/tooltips without extra visible group
  headers.
- [x] Widen the standalone Add/Modify settings window enough that the top action
  row, including Import, is not clipped in the default window size.
- [x] Move OneDrive setup actions into the top action row: Start OneDrive Setup,
  Start OneDrive Mirror Setup, and Use Manual Auth Handoff.
- [x] Split Modify-mode action buttons into two rows only for OneDrive Offline
  Mirror connections, so OneDrive setup actions do not crowd Preview, Sync Now,
  Disable, and Remove in the default settings window width while other
  connection types keep a single action row.
- [x] Make Test Connection and Save Connection visually non-primary for OneDrive
  drafts until the app-owned OneDrive setup/authentication artifact exists.
- [x] Add app/window identity metadata so the task switcher and right-click
  taskbar menu can resolve the app name instead of `Cosmic - Iced` or
  `Cosmic, Iced`.
- [x] User-verify app/window identity in the task switcher and right-click
  taskbar menu after reopening the installed applet/settings window.
- [x] Reinstall the applet/settings binary after the Add/Modify UI cleanup.
- [x] Complete user-guided visual review of the Add/Modify window.
- [x] Complete user-guided tooltip content review using `Tooltip Review.md`,
  then incorporate approved wording back into the applet.

### Add/Modify Settings Workflow Redesign

#### Resolved Setup Decisions

- [x] Decide rclone setup policy: prefer applet-driven setup. The applet shall
  create/configure rclone remotes, start the authentication flow, and verify the
  selected provider/subtree before saving. Existing rclone remotes remain
  importable/selectable as a fallback.
- [x] Define rclone verification behavior: the applet verifies that the remote
  exists, uses the expected backend, is authenticated, and can access the
  selected subtree without storing provider credentials.
- [x] Decide OneDrive Online mount setup policy for `jstaf/onedriver`: prefer
  applet-guided setup by launching or integrating with the `jstaf/onedriver` account
  setup flow, then verifying that the selected mount can start. Existing
  `jstaf/onedriver` account/mount configuration remains selectable as a fallback.
- [x] Define `jstaf/onedriver` verification behavior: the applet verifies that the
  selected OneDrive account is authenticated, that the mountpoint can be
  started, and that it does not overlap with an Offline mirror.
- [x] Decide OneDrive Offline mirror setup policy for `abraunegg/onedrive`:
  prefer applet-guided setup using isolated per-connection config and sync
  directories plus the supported authorization flow. External authorization
  remains available when Microsoft tenant policy or authentication requirements
  prevent in-applet completion.
- [x] Define `abraunegg/onedrive` verification behavior: the applet verifies
  authorization, selected subtree/syncdir, monitor support, and no overlap with
  `jstaf/onedriver` before saving/enabling.
- [x] Decide VPN selection UX: list detected NetworkManager and Cisco options,
  allow no VPN, and expose whether the applet may activate the selected VPN and
  later disconnect only VPNs it activated.
- [x] Decide NetworkManager VPN behavior: the applet enumerates existing
  NetworkManager VPN profiles, allows associating one profile with a storage
  connection, and may start/stop that profile through NetworkManager when the
  user permits applet activation. The applet does not create or edit
  NetworkManager VPN profiles or store VPN credentials.
- [x] Decide Cisco VPN behavior: the applet may detect/start the Cisco agent and
  open the Cisco client UI, but the user selects the account/profile and
  completes authentication interactively in Cisco. The applet verifies tunnel
  readiness before mount/sync and may disconnect only Cisco sessions it
  activated, after no dependent connection still needs them.
- [x] Decide per-field help behavior: attach COSMIC/libcosmic hover tooltips to
  the relevant field or button. Do not show visible help buttons in the default
  UI; reserve them only for future controls that cannot reasonably carry their
  own tooltip. Position tooltips above or to the side so they do not obscure the
  field or button being explained. This replaces the separate Help button for
  settings guidance.

#### Add/Modify UI Implementation Tasks

- [x] Use one shared Add/Modify connection window. Add opens the window with
  defaults; Modify opens the same window prepopulated from the selected
  connection.
- [x] Launch Add/Modify/Import as a standalone COSMIC settings application
  window titled `Cloud Mounter Connection Settings`, following the same pattern
  used by other COSMIC applets for larger settings surfaces.
- [x] Remove the top navigation row from the Modify window. Do not show Add
  Connection, Import, Refresh, or Help at the top while editing an existing
  connection.
- [x] Add a top action row for Add/Modify:
  - Add mode baseline: Test Connection, Save Connection, Import.
  - Add mode for rclone providers: also Detect rclone remotes and the
    provider-specific Create Remote action.
  - Modify mode: Test Connection, Save Connection, Disable or Enable, Remove.
  - Saved Offline Mirror Modify mode: also Preview and Sync Now.
- [x] Combine Test Plan and Test Existing into one Test Connection action that
  validates the current form values and reports the result in the window.
- [x] Remove the Enable Yes/No toggle from Online mount settings if the top
  Enable/Disable action controls the same `enabled` field.
- [x] Keep Remove as a Modify-window action with two-step confirmation; do not
  expose Remove on the main popup.
- [x] Make destructive connection removal state visible in Modify mode:
  `Remove` changes to `Confirm Remove`, then to a disabled/shaded
  `Removing...` state while generated-unit cleanup runs.
- [x] Move Import into the Add-mode action row only. Import shall scan legacy
  services, map a selected preview into the same Add/Modify fields, and require
  Save Connection before creating the applet-managed connection.
- [x] Replace the settings Help button with attached per-field tooltip help for
  provider, access mode, remote/account, subtree, local target, mode-specific
  options, VPN dependency, test, save, import, disable, and remove. Prefer
  hover tooltips attached to the field or action itself; reserve visible help
  buttons only for cases where attached help is not practical. Tooltips shall
  appear above or to the side of the source control rather than covering it.
- [x] Remove nested tooltip wrappers so controls such as Detect VPNs,
  NetworkManager VPN profile choices, and Cisco Secure Client choices display
  only one tooltip at a time.
- [x] Write concise tooltip help text for each supported field/action and keep
  longer background material in README/documentation.
- [x] Implement VPN dependency selection as a real field backed by detected or
  configured VPN profiles, including no VPN, profile choice, readiness behavior,
  applet activation permission, and disconnect-at-unmount/unused behavior.
- [x] Implement provider-specific rclone remote/account selection for Google
  Drive, Box, and SMB. Detect existing rclone remotes with `rclone config dump`,
  keep only remote names and backend types, filter choices by provider backend,
  and allow selecting a matching remote from the Add/Modify wizard.
- [x] Add validation messages for missing rclone remotes, wrong backend type,
  inaccessible subtree, authentication/authorization failures, and network/VPN
  readiness failures during rclone access tests.
- [x] Update Add/Modify verification to cover add, modify, import, test,
  enable/disable, remove, VPN selection, rclone verification, and attached
  tooltip help.

#### Rclone Provider Runtime Verification

- [x] Implement Save Connection side effects for rclone Online Mount
  connections: after validated configuration save, create or update the
  applet-owned systemd user service for the saved connection without starting
  it automatically.
- [x] Implement popup Mount for saved rclone Online Mount connections by
  starting the applet-owned systemd user service and refreshing actual service
  and mount state.
- [x] Implement popup Unmount for saved rclone Online Mount connections by
  performing the approved clean stop/unmount path, preserving cache and local
  mountpoint, and refreshing state.
- [x] Verify the first full vertical slice with Box Online Mount:
  remote `ua_box`, subtree `Utzinger/cosmic-mounter-ui-test`, and a disposable
  mountpoint such as `/home/uutzinger/Cloud/cosmic-mounter-box-test`.
  Required checks: Test Connection passes, Save creates an applet-owned unit,
  main popup shows the connection, Mount exposes the remote folder as a
  filesystem, Unmount removes the mount, and Modify reopens with saved values.
- [x] Repeat rclone Online Mount runtime verification with Google Drive using
  remote `uutzinger_gdrive`, subtree `cosmic-mounter-ui-test`, and a disposable
  local mountpoint.
- [x] Repeat rclone Online Mount runtime verification with SMB using remote
  `ua_engr`, subtree `Research/Utzinger/cosmic-mounter-ui-test`, a disposable
  local mountpoint, and Cisco VPN readiness.
- [x] Implement Save Connection side effects for rclone Offline Mirror
  connections: create or update applet-owned sync service and timer without
  starting destructive synchronization automatically.
- [x] Implement Offline Mirror preview/Sync Now using the managed backend,
  preserving dry-run preview and confirmation before initial synchronization.
  These actions were initially exposed in the popup and are now exposed from
  the saved connection's Modify action row.
- [x] Implement Offline Mirror Start/Stop as the primary popup action, using
  the applet-owned timer or monitor for background synchronization after
  successful preview and confirmed initial Sync Now. Keep Sync Now and Preview
  as secondary one-shot actions.
- [x] Complete readiness and policy gating for automatic background
  synchronization start: honor network/VPN readiness, metered-network policy,
  and Pause/Resume state before enabling or starting timers/monitors.
- [x] Verify Box Offline Mirror with remote `ua_box`, subtree
  `Utzinger/cosmic-mounter-ui-test`, and disposable mirror/work/recovery
  directories.
- [x] Verify Google Drive Offline Mirror with remote `uutzinger_gdrive`, subtree
  `cosmic-mounter-ui-test`, and disposable mirror/work/recovery directories.
- [x] Verify SMB Offline Mirror with remote `ua_engr`, subtree
  `Research/Utzinger/cosmic-mounter-ui-test`, disposable mirror/work/recovery
  directories, and Cisco VPN readiness.

#### OneDrive Provider Runtime Verification

- [x] Implement Save Connection side effects for OneDrive Online Mount
  connections: create or update the applet-owned `jstaf/onedriver` service without
  disturbing an existing user-managed `jstaf/onedriver` setup.
- [x] Implement popup Mount for saved OneDrive Online Mount connections by
  starting the applet-owned `jstaf/onedriver` service and refreshing actual service and
  mount state.
- [x] Implement popup Unmount for saved OneDrive Online Mount connections by
  performing the clean unmount/stop path while preserving credentials, cache,
  and local mountpoint.
- [x] Record controlled `jstaf/onedriver` baseline verification using the existing
  corporate OneDrive account: isolated mount/cache, disposable marker
  write/read/remove, clean unmount, and cached read-only offline behavior.
- [x] Verify the full app-managed OneDrive Online Mount vertical slice with
  `jstaf/onedriver` using a disposable mountpoint and non-critical test folder:
  Save creates an applet-owned unit, popup Mount exposes the OneDrive folder,
  Unmount detaches it, and the app-managed service preserves existing
  user-managed `jstaf/onedriver` setup.
- [x] Implement Save Connection side effects for OneDrive Offline Mirror
  connections: create isolated `abraunegg/onedrive` config/sync/recovery
  directories and an applet-owned monitor or sync service without starting
  synchronization automatically.
- [x] Implement OneDrive Offline Mirror preview and Sync Now using
  `abraunegg/onedrive --dry-run` and explicit confirmation before initial
  synchronization; these actions were initially exposed in the popup and are now
  exposed from the saved connection Modify action row.
- [x] Verify `abraunegg/onedrive` Offline Mirror with a non-critical personal
  Microsoft account or disposable subtree: isolated `--confdir`, isolated
  `--syncdir`, dry-run preview, initial sync, normal sync, and no interaction
  with `jstaf/onedriver`.
- [x] Document how to create and authorize a private personal OneDrive test
  account for `abraunegg/onedrive` verification, including account isolation
  from existing corporate browser sessions.

#### Provider Setup Flow Implementation

- [x] Implement applet-driven rclone remote creation/configuration for Google
  Drive, Box, and SMB after the existing-remote vertical slices are stable.
  - [x] Implement SMB rclone remote creation from the Add/Modify window using
    remote name, SMB host, optional username, and optional domain/workgroup.
    The applet creates the remote with fixed `rclone config create ... smb`
    arguments, `--non-interactive`, duplicate-name checks, and redacted
    diagnostics. Password handling remains with rclone rather than applet
    storage.
  - [x] Verify SMB rclone remote creation against a disposable/non-critical
    NetworkManager/Cisco-ready share and confirm Test Connection, Save
    Connection, Online Mount, and Offline Mirror paths still work with the
    created remote.
  - [x] Implement Box OAuth remote creation/configuration through rclone.
  - [x] Implement Google Drive OAuth remote creation/configuration through
    rclone.
  - [x] Live-verify Box OAuth remote creation with a disposable/non-critical
    remote name, then verify Test Connection, Save Connection, and Online Mount
    with the created remote.
  - [x] Live-verify Box Offline Mirror with the applet-created Box OAuth remote.
  - [x] Live-verify Google Drive OAuth remote creation with a disposable/non-critical
    remote name, then verify Test Connection, Save Connection, Online Mount, and
    Offline Mirror with the created remote.
- [x] Implement applet-driven OneDrive Online Mount setup/configuration for
  `jstaf/onedriver`.
  - [x] Live-verify applet-driven OneDrive Online Mount setup by starting
    `jstaf/onedriver` authentication from the Add/Modify window with a
    disposable/non-critical mountpoint, then confirming Test Connection and Save
    Connection use the same app-owned config/cache paths.
- [x] Implement applet-driven OneDrive Offline Mirror setup/configuration for
  `abraunegg/onedrive`.
  - [x] Live-verify applet-driven OneDrive Offline Mirror setup/authentication
    by starting
    `abraunegg/onedrive` authentication from the Add/Modify window with a
    disposable/non-critical local mirror directory, completing the applet
    auth-files handoff, then confirming authentication and dry-run validation
    preview use the same app-owned confdir/syncdir/recovery paths.
  - [x] Live-verify OneDrive Offline Mirror Test Connection, Save Connection,
    initial preview, Sync Now, and generated service/timer paths with the
    app-owned confdir/syncdir/recovery paths.
  - [x] Improve `abraunegg/onedrive` Offline Mirror authorization so the applet
    can capture or receive the Microsoft redirect more gracefully than the
    current browser-history/manual paste flow.
    - [x] Investigate whether `abraunegg/onedrive` can complete authorization
      through a local redirect listener, device-code flow, external browser
      integration, or another supported upstream mechanism.
    - [x] Implement the supported upstream local-redirect path as the primary
      applet action: `Start OneDrive Mirror Setup` now runs
      `onedrive --reauth` against the app-owned confdir and lets
      `abraunegg/onedrive` open the browser and receive the local callback.
    - [x] Prototype a separate WebKitGTK auth helper launched from Manual Auth
      Handoff. The helper reads the generated auth URL file, opens Microsoft
      sign-in in a GTK/WebKit window, watches for the final native-client
      redirect, and writes that URL to the transient response file.
    - [x] Preserve the selectable instruction/command dialog and explicit
      response URL field as a fallback when the WebKitGTK helper is unavailable
      or cannot capture the redirect.
    - [x] Preserve the existing auth-files paste flow as a live-tested fallback for
      tenant/browser environments where automatic redirect capture fails.
    - [x] Live-verify the WebKitGTK helper with a disposable/non-critical
      OneDrive Offline Mirror connection. Confirm whether the helper captures
      the native-client redirect automatically. If it fails, verify that manual
      paste fallback remains usable.

## Historical Gate 1: Isolated Release Candidate (Superseded)

- [x] All formatting, lint, unit, integration, and snapshot tests pass.
- [x] No test has touched real services, mounts, VPNs, credentials, or cloud data.
- [x] Review generated units, timers, logs, and fixtures for secret leakage.
- [x] Review sync deletion, conflict, and recovery behavior for data-loss risks.
- [x] Review detach and VPN policies against the approved specification.
- [x] Obtain user approval for controlled real-system testing.

## Current Gate 1 Recheck: Integrated Release Candidate

- [x] Current formatting, lint, unit, integration, and snapshot tests pass.
- [x] Review generated units/timers after latest Online Mount, Offline Mirror,
  Start/Stop, OAuth, and import changes.
- [x] Review real-system side effects and cleanup plan for disposable test
  connections, remotes, services, timers, mirrors, caches, and recovery paths.
- [x] Review sync deletion, conflict, and recovery behavior against the current
  UI/runtime implementation.
- [x] Review detach, background sync readiness/metered policy, and VPN policies
  against the current specification.
- [x] Obtain user approval for current controlled integrated manual testing.

## Historical Phase 10: Controlled Manual Testing Evidence (Superseded)

- [x] Verify dependency detection and outdated-version guidance.
- [x] Test a disposable rclone Online mount.
- [x] Test safe connectivity-loss detach and automatic remount.
- [x] Test pending-write protection during connectivity loss.
- [x] Test `jstaf/onedriver` Online mount.
- [x] Test `jstaf/onedriver` offline cached read-only behavior.
- [x] Test a local-to-local rclone bisync mirror.
- [x] Test one approved Google Drive or Box Offline mirror.
- [x] Test offline editing followed by reconnect synchronization.
- [x] Test conflict preservation, deletion propagation, and recovery.
- [x] Test metered pause and Sync Now.
- [x] Test `abraunegg/onedrive` with a non-critical account or subtree.
- [x] Test NetworkManager VPN readiness.
- [x] Test Cisco interactive authentication when installed.
- [x] Test import preview from the real user service folder.
- [x] Test confirmed import from the real user service folder.
- [x] Verify removal preserves credentials, data, cache, recovery, and originals.

## Current Phase 10: Integrated Manual Testing Recheck

- [x] Verify dependency detection and upgrade guidance in the current UI.
- [x] Test applet-created rclone OAuth remotes for Box and Google Drive from
  the Add/Modify workflow or documented equivalent.
- [x] Test rclone Online Mount Start/Stop/Unmount flows for Box, Google Drive,
  and SMB as applicable.
  - [x] Box Online Mount Start/Stop/Unmount recheck.
  - [x] Google Drive Online Mount Start/Stop/Unmount recheck.
  - [x] SMB Online Mount Start/Stop/Unmount recheck with VPN readiness.
- [x] Test rclone Offline Mirror Preview, Sync Now, Start, Stop, and
  timer/service state for Box, Google Drive, and SMB.
  - [x] Box Offline Mirror Preview, Sync Now, Start, Stop, and timer/service
    state recheck.
  - [x] Google Drive Offline Mirror Preview, Sync Now, Start, Stop, and
    timer/service state recheck.
  - [x] SMB Offline Mirror recheck with VPN readiness.
- [x] Test OneDrive Online Mount setup/mount/unmount with the existing
  corporate account without disturbing the existing onedriver setup.
- [x] Test OneDrive Offline Mirror setup/preview/sync with the personal account
  and document the manual auth fallback.
- [x] Test NetworkManager and Cisco VPN selection/readiness with no credential
  storage and no unintended disconnect.
- [x] Test import preview and confirmed import replacement creation from the
  real user service folder.
- [x] Test removal cleanup policy for applet-owned generated units while
  preserving data, cache, recovery, credentials, and original legacy files.
- [x] Test keyboard-only navigation and accessible labels in the popup and
  Add/Modify windows.
  - [x] Automated accessible/non-color label tests.
  - [x] Manual keyboard-only traversal and activation check in the current
    slider-based popup and Add/Modify windows.
- [x] Verify disposable local/remote test data and generated units/timers are
  either intentionally retained for inspection or cleaned up.

## Historical Phase 11: Documentation and Packaging (Superseded)

- [x] Document installation and current-version requirements for every engine.
- [x] Document the Online mount versus Offline mirror tradeoff.
- [x] Document synchronization conflicts, deletions, recovery, and backup limits.
- [x] Document authentication and VPN behavior.
- [x] Document generated files, legacy imports, and uninstall behavior.
- [x] Add MIT license and author attribution.
- [x] Finalize desktop entry, metainfo, and icon.
- [x] Test staged installation and uninstallation.
- [x] Prepare release notes and known limitations.

## Current Phase 11: Documentation and Packaging Recheck

- [x] Update README for the current popup layout, Add/Modify workflow,
  Start/Stop versus Sync Now/Preview behavior, and active/VPN summary behavior.
- [x] Update dependency docs for applet-driven rclone, onedriver, and onedrive
  setup flows.
- [x] Document generated services/timers, background sync gating, cleanup/removal
  behavior, and known limitations.
- [x] Document the current import workflow and remaining import limitations.
- [x] Document OAuth/security behavior without storing credentials.
- [x] Update release notes and known limitations.
- [x] Re-run desktop/metainfo validation.
- [x] Re-test staged install/uninstall on current artifacts.
- [x] Prepare final user-facing cleanup instructions for disposable verification
  connections, remotes, services, timers, mirrors, caches, and recovery paths.

## Historical Gate 2: Version 0.1 Completion (Superseded)

- [x] Every acceptance criterion is demonstrated.
- [x] Required automated and approved manual tests pass.
- [x] No unresolved high-severity security, data-loss, or privilege issue remains.
- [x] Known limitations are documented.
- [x] User approves the release candidate.

## Current Gate 2: Version 0.1 Release Candidate

- [x] Phase 9 UI/runtime completion is complete.
- [x] Current Gate 1 recheck is approved.
- [x] Current Phase 10 integrated manual testing passes.
- [x] Current Phase 11 documentation and packaging recheck passes.
- [x] Popup mount, unmount, Start, Stop, Sync Now, Preview, retry/repair/details
  behavior is demonstrated with fake or approved disposable backends.
  - [x] Implement visible lazy-unmount confirmation/recovery after
    clean onedriver/FUSE unmount fails and leaves a stale mount attached.
  - [x] Verify failed service plus lingering FUSE mount reports Error, not
    Mounted.
  - [x] Verify Error rows use Repair as the primary popup action.
  - [x] Live-verify the two-click Repair confirmation/recovery in COSMIC after
    a clean onedriver/FUSE unmount failure.
- [x] Settings workflows cover all approved user-configurable items.
- [x] Required automated and approved manual UI tests pass.
- [x] No unresolved high-severity security, data-loss, cleanup, credential, or
  privilege issue remains.
- [x] Known limitations are documented.
- [x] User approves the current release candidate.

## Version 0.2 Planning: UI Modification

These tasks refine the version 0.1 release-candidate UI. They are intentionally
tracked as new version 0.2 work so the completed version 0.1 gates remain
auditable.

### Version 0.2 Change-Existing-Connection Policy

- [x] Decide and document the Modify Connection scope.
  - Recommended policy: modifying an existing connection may change display
    name, remote/subtree, local mountpoint or mirror directory, cache/sync
    settings, VPN dependency, start-at-login, enable/disable state, and
    provider-specific non-credential settings.
  - Recommended policy: modifying an existing connection shall not change the
    provider or access mode because that can change storage engine, credential
    ownership, generated unit type, cache/sync state, and data-safety rules.
  - Future option: add a separate Duplicate or Convert Connection flow if users
    need to move a connection from one provider or mode to another.
- [x] If provider changes remain disallowed in Modify mode, disable provider
  controls visually and expose a tooltip explaining that provider changes
  require creating or duplicating a connection.
- [x] If access-mode changes remain disallowed in Modify mode, disable Online
  mount and Offline mirror controls visually and expose a tooltip explaining
  that mode changes require creating or duplicating a connection.
- [x] If a provider or access-mode conversion flow is later approved, define
  provider-specific validation, generated-unit replacement, credential reuse,
  cache/mirror migration, and rollback behavior before implementation.
  - Deferred for version 0.2. Version 0.2 Modify mode shall not support provider
    or access-mode conversion. Any future conversion feature must be specified
    as a separate Duplicate or Convert Connection workflow before
    implementation.

### Version 0.2 Main Popup UI

- [x] Move operation result/status text out of the top of the connection list
  into a separate status row immediately above the `Add Connection` and
  `Refresh` button row.
- [x] Ensure the popup height follows the connection-list height until roughly
  75 percent of screen height, then constrains the connection list and enables
  scrolling.
  - Implemented with tunable popup constants: the list uses natural row-count
    height until `POPUP_CONNECTION_LIST_MAX_HEIGHT`, then scrolls.
- [x] Re-check popup height calculation with and without operation result text
  so notices do not push the popup beyond the screen.

### Version 0.2 Modify Connection UI

- [x] Choose and apply a non-grey color treatment for `Disable` that is distinct
  from primary blue actions and destructive red actions.
  - Implemented as a theme-derived soft destructive button that brightens the
    COSMIC destructive button color while keeping `Remove` as full destructive.
- [x] Disable provider selection in Modify mode unless a later conversion flow
  is explicitly approved.
- [x] Disable access-mode selection in Modify mode unless a later conversion
  flow is explicitly approved.
- [x] Preserve original remote/account field values when switching disabled or
  preview-only provider controls is impossible; do not allow provider toggles to
  corrupt the saved rclone remote or OneDrive account fields.
- [x] Validate renamed connections so a connection name cannot duplicate another
  existing connection.
- [x] Validate changed local mountpoint or mirror directory against every other
  connection; reject duplicate targets, nested targets, and paths inside another
  configured mount or mirror tree.
- [x] When a selected account or rclone remote is already used by another saved
  connection, require explicit user acknowledgement before saving if the
  provider/mode combination permits shared credentials safely.

### Version 0.2 Add Connection UI

- [x] Remove the `Import` button from the default Add Connection action row, or
  move legacy import behind a less prominent advanced entry point.
- [x] In OneDrive Online mount mode, label the account/setup field as
  `onedriver` rather than generic `onedrive`.
- [x] In OneDrive Offline mirror mode, label the account/setup field as
  `onedrive` or `abraunegg/onedrive` consistently.
- [x] Update OneDrive Online mount tooltip text to warn: "Do not reuse this
  mountpoint for a OneDrive mirror."
- [x] Update OneDrive Offline mirror tooltip text to warn: "Do not reuse this
  mirror directory for a OneDrive Online mount."
- [x] Validate Add Connection names so a new connection name cannot duplicate an
  existing connection.
- [x] Validate Add Connection local mountpoint or mirror directory against every
  other configured connection; reject duplicate targets, nested targets, and
  paths inside another configured mount or mirror tree.
- [x] When a selected account or rclone remote is already used by another saved
  connection, require explicit user acknowledgement before saving if the
  provider/mode combination permits shared credentials safely.
- [x] Change the Add Connection display-name field from prefilled text to
  placeholder/suggested text, so activating the field starts with an empty input
  rather than requiring the user to remove generated text.

### Version 0.2 Rclone Remote Management UI

- [x] Add a way to remove unused rclone remotes created during setup/testing.
- [x] Prevent removal of any rclone remote currently referenced by a saved
  connection.
- [x] Require explicit confirmation before deleting an rclone remote, and state
  that this affects rclone configuration rather than only applet configuration.
- [x] Make rclone remote removal state visible: `Remove remote` changes to
  `Confirm Remove`, then to a disabled/shaded `Removing...` state while rclone
  configuration is updated.
- [x] Decide whether rclone remote removal is hidden behind an advanced action
  such as Shift-click, a context action, or a dedicated advanced management
  surface.
- [x] Wrap detected rclone remote buttons across multiple rows when they exceed
  the settings window width.
- [x] Estimate remote-button row capacity from available width rather than
  allowing buttons to overflow or clip.

### Version 0.2 OneDrive Setup Help Text

- [x] Rewrite `Start OneDrive Setup` tooltip/help text to say that all required
  fields must be completed before setup and that the user must complete browser
  authentication.
- [x] Rewrite `Start OneDrive Mirror Setup` tooltip/help text to say that all
  required fields must be completed before setup and that the user must complete
  browser authentication.
- [x] Remove storage-engine internals and authorization-storage details from
  OneDrive setup tooltips; keep those details in README or dependency
  documentation.

### Version 0.2 Verification

- [x] Verify main popup status-row placement, dynamic height, and scrolling with
  zero, few, and many configured connections.
- [x] Verify Modify mode cannot accidentally change provider, access mode,
  remote/account field, or generated engine state unless a conversion flow is
  explicitly implemented.
- [x] Verify Add and Modify validation for duplicate names, duplicate local
  targets, nested paths, and shared account/remote acknowledgement.
- [x] Verify rclone remote removal refuses in-use remotes and removes only the
  selected unused remote after confirmation.
- [x] Verify connection and rclone remote removal succeed from the Add/Modify
  window after the confirmation-state UI update.
- [x] Verify detected rclone remotes wrap cleanly at the default settings window
  width.
- [x] Complete user-guided visual review of the Version 0.2 Add/Modify and main
  popup changes.

## Version 0.3 Planning: Local Path Selection

These tasks improve Add/Modify usability without changing provider engines or
authentication behavior.

### Version 0.3 Folder Picker

- [x] Add a folder picker/requestor for the local mountpoint field in Online
  Mount mode.
- [x] Add a folder picker/requestor for the local mirror directory field in
  Offline Mirror mode.
- [x] Prefer a desktop-portal folder chooser when available so the applet uses
  the user session's native file/folder selection workflow.
- [x] Ensure the folder picker can create a new folder or offers a clear
  create-folder path through the underlying chooser.
- [x] Preserve plain text entry as an advanced/manual fallback.
- [x] Validate picked folders with the existing duplicate-target, nested-path,
  unsafe-path, and mount/mirror overlap checks before saving.
- [x] Add focused tests for path normalization and validation after a selected
  folder is applied to a draft connection.

### Version 0.3 Popup Runtime Status

- [x] Replace config-only VPN header text with runtime VPN state reporting.
- [x] Parse Cisco Secure Client `Connection State:` exactly so `Disconnected`
  and `Not Available` are not misclassified as connected.
- [x] Clear transient popup messages automatically after 10 seconds.
- [x] Keep the popup startup/open message empty unless an actual user action or
  error produces a message.
- [x] Move NetworkManager and Cisco VPN status checks out of popup rendering so
  the applet opens immediately and updates VPN state asynchronously.
- [x] Manually verify the installed applet opens promptly with Cisco configured
  but disconnected, then updates the header to inactive after the async status
  check completes.


## Version 0.3 COSMIC Utils Project List Publication

- [x] Fork and clone the
  [COSMIC project collection](https://github.com/cosmic-utils/cosmic-project-collection).
- [x] Add and validate the Cloud Mounter entry in `applets.ron`,
  including its public screenshot URL.
- [x] Commit and push the entry to the `uutzinger/cosmic-project-collection`
  fork.
- [x] Open
  [cosmic-utils/cosmic-project-collection pull request 85](https://github.com/cosmic-utils/cosmic-project-collection/pull/85).
- [x] After the pull request is merged, verify the upstream workflow generates
  the README and website entries and that both display the project correctly.


## Version 0.3 COSMIC Flatpak Repository Publication

The target is the `pop-os/cosmic-flatpak` repository, which hosts COSMIC
applets and software that is not suitable for Flathub. The application ID is
`io.github.uutzinger.cosmic-ext-applet-mounter`. COSMIC Store consumes
AppStream data from configured Flatpak remotes; publication in this repository
makes the applet available to users who have enabled the COSMIC Flatpak remote.

### Verified COSMIC Distribution Conventions

- [x] Confirm `pop-os/cosmic-flatpak` is the intended submission repository for
  COSMIC applets and other COSMIC software unsuitable for Flathub.
- [x] Confirm accepted applets are ordinary sandboxed Flatpak applications
  built with `flatpak-builder`, exported through their desktop/AppStream/icon
  files, and launched by the COSMIC panel through the exported desktop entry.
- [x] Confirm current accepted applets generally use
  `org.freedesktop.Platform` and `org.freedesktop.Sdk` version `25.08`, the
  `org.freedesktop.Sdk.Extension.rust-stable` SDK extension, and
  `com.system76.Cosmic.BaseApp` version `stable`.
- [x] Confirm submissions currently use
  `app/<application-id>/<application-id>.json` plus a generated
  `cargo-sources.json` or equivalent generated Rust source list.
- [x] Confirm the official applet template identifies applets with
  `NoDisplay=true`, `X-CosmicApplet=true`, `X-CosmicHoverPopup=Auto`, the
  `COSMIC` category/keyword, and `com.system76.CosmicApplet` in AppStream
  `<provides>`.
- [x] Confirm repository CI runs `just build-changed`; the equivalent local
  build is `just build io.github.uutzinger.cosmic-ext-applet-mounter`.

### Publication Suitability and Architecture Gate

- [x] Document the current host-integration requirements: executable discovery,
  user systemd unit creation and control, FUSE mount visibility, host
  NetworkManager/Cisco status, existing rclone configuration, provider
  authentication, and access to user-selected mount/mirror directories.
- [x] Add a Flatpak runtime mode that routes approved host commands through
  `flatpak-spawn --host`; preserve the current direct command runner for native
  `.deb` and source installations.
  - [x] Prototype the runtime command-runner layer with native and
    `flatpak-spawn --host` modes without wiring it into the installed applet.
  - [x] Wire the applet to select the Flatpak runtime runner when executing
    inside an installed Flatpak.
- [x] Start from the accepted COSMIC drives-applet precedent of
  `--talk-name=org.freedesktop.Flatpak` and narrowly justify any additional
  permissions this applet requires.
- [x] Verify host command argument passing, exit status, standard output,
  standard error, cancellation, timeouts, and secret redaction through
  `flatpak-spawn --host`.
  - [x] Add a probe-only Flatpak manifest that grants only
    `--talk-name=org.freedesktop.Flatpak`.
  - [x] Live-verify applet dependency detection through `flatpak-spawn --host`.
  - [x] Live-verify `rclone version`, `nmcli general status`,
    `systemctl --user --version`, and `fusermount3 --version` through
    `flatpak-spawn --host`.
  - [x] Live-verify nonzero exit status, stderr capture, timeout, and
    cancellation behavior through `flatpak-spawn --host`.
- [x] Determine which configuration belongs inside the Flatpak and which must
  remain on the host. Existing applet connections, rclone and OneDrive
  credentials, generated user units, and app-owned engine metadata must not be
  silently copied into a second sandbox-private store.
- [x] Implement a Flatpak host-visible applet state bridge.
  - [x] Reject sandbox-private applet connection configuration as the normal
    Flatpak behavior because users switching from source or Debian installs
    would otherwise have to recreate connections.
  - [x] Define the host-visible applet state root for Flatpak mode, matching
    the native COSMIC configuration namespace where possible.
  - [x] Add a typed host-side helper or equivalent host-spawn operation for
    reading the applet configuration document from the native-visible location.
  - [x] Add a typed host-side helper or equivalent host-spawn operation for
    validated atomic applet configuration writes.
  - [x] Route generated user systemd unit/timer writes through the same
    host-visible state path or host-side helper so systemd sees the files in
    the normal user session.
  - [x] Route app-owned onedriver config/cache paths and abraunegg/onedrive
    config/sync/recovery paths through documented host-visible locations.
  - [x] Keep provider-owned state host-owned: rclone config, onedriver auth,
    onedrive refresh tokens, NetworkManager/Cisco profiles, and systemd state.
  - [x] Verify native source and Debian installs continue using the existing
    direct COSMIC configuration and unit-management code paths.
- [x] Determine whether `--filesystem=host` is actually required. Prefer the
  narrowest filesystem permissions that still allow existing remote discovery,
  selected mount/mirror targets, recovery directories, and user-unit files.
- [x] Verify that host-created FUSE mounts are visible to ordinary host file
  managers and are not confined to the Flatpak mount namespace.
- [x] Verify that user systemd units generated by the applet invoke host binary
  paths and remain usable while the Flatpak is not running.
- [x] Reject any design that silently uses a different rclone/OneDrive config,
  cannot expose mounts to host applications, cannot manage the intended user
  services, or requires unjustified unrestricted host access.
- [x] Record the selected architecture and its security tradeoffs in
  `Requirements and Specifications.md` before implementing packaging changes.
- [ ] Ask `pop-os/cosmic-flatpak` maintainers for architecture guidance before
  submission if this applet requires broader host access than accepted applets
  such as `dev.cappsy.CosmicExtAppletDrives`.

### Flatpak Build Inputs

- [x] Add a project-owned Flatpak manifest named
  `io.github.uutzinger.cosmic-ext-applet-mounter.json` under a documented
  packaging directory; keep it structurally ready to copy into the COSMIC
  repository's matching `app/<application-id>/` directory.
- [x] Use the current accepted runtime stack:
  `org.freedesktop.Platform//25.08`, `org.freedesktop.Sdk//25.08`,
  `org.freedesktop.Sdk.Extension.rust-stable`, and
  `com.system76.Cosmic.BaseApp//stable`. Recheck these versions immediately
  before submission because repository conventions can advance.
- [x] Set the manifest `command` to `cosmic-ext-applet-mounter`.
- [x] Build the Rust binary from a tagged source archive or pinned commit rather
  than from an unpinned branch.
- [x] Generate `cargo-sources.json` for an offline, reproducible
  `flatpak-builder --sandbox` build, including the pinned libcosmic Git revision
  and its Git/submodule dependencies.
- [x] Add a repeatable project command or script that regenerates
  `cargo-sources.json` after dependency changes.
- [x] Install the applet binary to `/app/bin/cosmic-ext-applet-mounter` and the
  OneDrive authentication helper only if the approved architecture still uses
  it.
- [x] Install the desktop file as
  `/app/share/applications/io.github.uutzinger.cosmic-ext-applet-mounter.desktop`.
- [x] Install AppStream metadata as
  `/app/share/metainfo/io.github.uutzinger.cosmic-ext-applet-mounter.metainfo.xml`.
- [x] Install the scalable icon as
  `/app/share/icons/hicolor/scalable/apps/io.github.uutzinger.cosmic-ext-applet-mounter.svg`.
- [x] Define the minimum required `finish-args`; justify each filesystem,
  socket, device, D-Bus, and host-command permission in manifest comments or
  packaging documentation.
- [x] Include Wayland and required COSMIC settings-daemon access following
  accepted applet manifests; add fallback X11, IPC, network, session bus, host
  filesystem, or device access only when a tested feature requires it.
- [x] Continue using the desktop portal for user-selected folders where
  possible, even if broader filesystem permissions are ultimately required for
  mount and mirror operation.

### Desktop and AppStream Metadata

- [x] Use the reverse-DNS application ID
  `io.github.uutzinger.cosmic-ext-applet-mounter` consistently in the desktop
  file, AppStream metadata, icon name, and application code.
- [x] Include `<id>com.system76.CosmicApplet</id>` in the AppStream
  `<provides>` section so COSMIC can classify the package as an applet.
- [x] Change the AppStream binary declaration to the official template form:
  `<binaries><binary>cosmic-ext-applet-mounter</binary></binaries>`.
- [x] Add the AppStream `COSMIC` category and `COSMIC` keyword while retaining
  useful storage-related keywords.
- [x] Change the desktop entry to include `Categories=COSMIC;Utility;` and keep
  `Keywords=COSMIC;cloud;mount;mirror;storage;`.
- [x] Include `NoDisplay=true`, `X-CosmicApplet=true`, and
  `X-CosmicHoverPopup=Auto` in the desktop entry.
- [x] Compare the final desktop and AppStream files with the current official
  applet template again immediately before submission.
- [x] Update AppStream release entries to match the source tag submitted to the
  repository; do not label a final publication as a release candidate.
- [x] Ensure AppStream includes stable remote icon and screenshot URLs that are
  reachable without authentication and will remain valid for the submitted
  release.
Metadata validation for the next build is tracked in the final
release-candidate check below.

### Local Build and Installation

- [x] Install the local prerequisites: `flatpak`, `flatpak-builder`, and `just`.
- [x] Add the Flathub remote required by repository builds and runtime/SDK
  dependency installation.
- [x] Add the COSMIC repository for local testing:
  `flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo`.
- [x] Build through the current `pop-os/cosmic-flatpak` workflow using
  `just build io.github.uutzinger.cosmic-ext-applet-mounter` in a local clone of
  that repository; this runs `flatpak-builder` with `--sandbox`,
  `--force-clean`, `--install-deps-from=flathub`, and `--require-changes` and
  retains a log under `log/app/<application-id>/`.
- [x] Run `just build-changed` from a submission branch to reproduce repository
  CI behavior before opening the pull request.
- [x] Install the locally built Flatpak from the generated local OSTree
  repository for the current user and confirm the
  exported desktop entry, icon, AppStream record, and applet classification.
- [x] Confirm the applet can be added to and removed from the COSMIC panel, and
  that logout/login preserves the configured panel entry.
- [x] Confirm COSMIC Store displays the applet from a configured test/local
  Flatpak remote using its AppStream name, summary, icon, screenshots, and
  applet classification.
- [x] Confirm uninstall removes packaged files without deleting user-created
  connection configuration, mirrors, mountpoints, credentials, or recovery
  data.

### Flatpak Functional Verification

- [x] Verify the popup opens promptly and Add/Modify windows have the correct
  title, icon, tooltips, folder chooser, and keyboard navigation.
- [x] Decide and implement the Flatpak configuration/state model.
  - [x] Decide that the Flatpak applet shall share or migrate the native-visible
    COSMIC applet configuration rather than using sandbox-private connection
    configuration for normal operation.
  - [x] Implement host-visible applet configuration read access for Flatpak
    mode.
  - [x] Implement host-visible applet configuration write access for Flatpak
    mode.
  - [x] Verify Modify Connection opens native saved connections from the
    Flatpak runtime through the host-visible configuration bridge.
  - [x] Ensure generated user systemd unit files and app-owned durable
    config/cache/state paths are routed to locations where host services can
    actually use them.
  - [x] Live-verify app-owned onedriver and abraunegg/onedrive metadata paths
    from the Flatpak prototype before marking OneDrive Save/setup flows
    complete.
- [x] Verify the GUI settings window opens from the prototype Flatpak.
- [x] Resolve GUI prototype theme mismatch so the Flatpak settings window uses
  the host COSMIC light/dark mode and full theme configuration at startup.
- [x] Verify dependency detection reports host dependency state accurately from
  inside the packaged applet.
  - [x] Verify host dependency inventory through the packaged Flatpak host
    runner probe.
  - [x] Verify the real GUI reports dependency and safety state correctly from
    inside the prototype Flatpak.
- [x] Verify an existing rclone remote can be detected without copying or
  exposing credentials to applet configuration.
- [x] Verify Box and Google Drive OAuth setup, SMB remote setup, and unused
  rclone remote removal under the approved sandbox architecture.
  - [x] Live-verify applet-created Box OAuth remote setup from the Flatpak
    prototype using a disposable/non-critical connection.
  - [x] Live-verify applet-created Google Drive OAuth remote setup from the
    Flatpak prototype using a disposable/non-critical connection.
  - [x] Live-verify applet-created SMB remote setup from the Flatpak prototype
    after adding applet-managed password entry/update.
  - [x] Implement SMB password entry/update inside the applet using a transient
    masked password field and `rclone config password`, without saving SMB
    passwords in applet configuration.
  - [x] Live-verify unused rclone remote removal succeeds after the
    confirmation-state UI update.
- [x] Verify OneDrive Online Mount setup, mount, status refresh, clean unmount,
  and confirmed lazy-unmount recovery.
  - [x] Live-verify OneDrive Online Mount authentication/setup from the Flatpak
    prototype with the user's corporate OneDrive account.
  - [x] Live-recheck OneDrive Online Mount popup toggle, status refresh, and
    file access from the Flatpak prototype.
  - [x] Confirm failed onedriver clean unmount can leave a stale
    `fuse.onedriver` endpoint that requires lazy-unmount recovery.
  - [x] Live-recheck applet Repair confirmation flow for OneDrive Online Mount
    lazy-unmount recovery from the Flatpak prototype.
- [x] Verify OneDrive Offline Mirror authentication, preview, initial sync,
  start/stop, Sync Now, conflict preservation, and recovery retention.
  - [x] Increase bounded OneDrive post-auth dry-run validation timeout and make
    setup notices explain that authorization can be followed by several minutes
    of validation.
  - [x] Cache successful OneDrive setup/Test Connection validation in the open
    editor so Save Connection does not repeat the same dry-run when the form is
    unchanged.
  - [x] Use OneDrive-specific command error wording for OneDrive preview/sync
    operations instead of rclone wording.
  - [x] Run initial OneDrive Offline Mirror Sync Now with `--resync` after a
    successful Preview, then record initial sync completion so later Sync Now
    runs use normal `onedrive --sync`.
  - [x] Add `--resync-auth` to initial OneDrive Offline Mirror Sync Now so
    abraunegg/onedrive can run non-interactively after the applet-required
    Preview plus explicit Sync Now confirmation.
  - [x] Live-recheck OneDrive Offline Mirror authentication/setup and confirm
    the applet explains the long post-auth validation wait.
  - [x] Live-recheck OneDrive Offline Mirror Save after setup validation and
    confirm it skips duplicate validation when the draft is unchanged.
  - [x] Live-recheck initial OneDrive Offline Mirror Sync Now starts after the
    `--resync-auth` correction.
  - [x] Live-recheck OneDrive Offline Mirror initial Sync Now completion and
    confirm downloaded local mirror content plus `initial-sync-complete` state.
  - [x] Live-recheck OneDrive Offline Mirror later normal Sync Now after
    `initial-sync-complete`.
  - [x] Live-recheck OneDrive Offline Mirror start/stop monitoring through the
    Flatpak popup toggle.
  - [x] Live-recheck OneDrive Offline Mirror local-to-cloud Sync Now with a
    disposable test file in the personal OneDrive mirror.
  - [x] Live-recheck OneDrive Offline Mirror conflict preservation with a
    disposable same-file local/remote edit.
  - [x] Live-recheck OneDrive Offline Mirror recovery retention using provider
    recycle-bin recovery for a disposable deleted file.
- [x] Verify rclone Online Mount and Offline Mirror workflows for Box, Google
  Drive, and SMB using disposable/non-critical data.
  - [x] Verify the Flatpak-generated Box Online Mount host user unit can be
    started and stopped cleanly for the saved `UA Box` connection.
  - [x] Verify the Box Online Mount appears as a host-visible `fuse.rclone`
    mount and can be read by ordinary host processes.
  - [x] Verify the Flatpak-generated personal Google Drive Online Mount host
    user unit can be started/stopped cleanly and appears as host-visible
    `fuse.rclone`.
  - [x] Verify the same Box Online Mount start/stop path through the applet
    popup toggle rather than direct host `systemctl --user` control.
  - [x] Verify the personal Google Drive Online Mount start/stop path through
    the applet popup toggle.
  - [x] Verify the SMB Online Mount popup toggle with Cisco VPN connected,
    confirm expected mounted content, then disconnect VPN and unmount cleanly.
  - [x] Use longer bounded rclone mount timeouts for SMB Online Mounts than for
    Google Drive and Box, because slow corporate SMB directory listings can
    exceed the cloud-mount timeout and surface as file-manager I/O errors.
  - [x] Add a disposable Box Offline Mirror connection for Flatpak preview and
    sync verification.
  - [x] Resolve initial rclone bisync preview failure caused by the
    `--check-access`/`RCLONE_TEST` sentinel requirement on empty first-time
    local mirrors.
  - [x] Re-run Box Offline Mirror Preview from the Flatpak popup and confirm it
    produces a dry-run summary without requiring a preexisting local sentinel
    file.
  - [x] Run Box Offline Mirror initial Sync Now from the Flatpak popup, confirm
    the disposable remote files appear in the local mirror, and confirm future
    syncs use normal bisync behavior.
  - [x] Add a disposable Google Drive Offline Mirror connection for Flatpak
    preview and sync verification.
  - [x] Re-run Google Drive Offline Mirror Preview from the Flatpak popup and
    confirm it produces a dry-run summary without requiring a preexisting local
    sentinel file.
  - [x] Run Google Drive Offline Mirror initial Sync Now from the Flatpak popup,
    confirm the disposable remote files appear in the local mirror, and confirm
    future syncs use normal bisync behavior.
  - [x] Add a disposable SMB Offline Mirror connection for Flatpak preview and
    sync verification.
  - [x] Run SMB Offline Mirror Preview from the Flatpak popup and confirm it
    produces a dry-run summary with Cisco VPN connected.
  - [x] Run SMB Offline Mirror initial Sync Now from the Flatpak popup, confirm
    the disposable remote files appear in the local mirror, and confirm future
    syncs use normal bisync behavior.
- [x] Verify user systemd services and timers are created, controlled, and
  observed in the intended host user session.
  - [x] Verify the Flatpak-generated `UA Box` mount unit is loaded in the host
    user systemd manager and remains disabled after manual start/stop testing.
  - [x] Verify tested Box, Google Drive, and SMB online mount services plus
    disposable offline mirror services/timers are loaded from host
    `~/.config/systemd/user`, disabled, and inactive after verification.
- [x] Verify FUSE mounts created by the applet are visible to ordinary host file
  managers and applications and do not remain trapped in the Flatpak namespace.
  - [x] Verify `UA Box` mounted from the Flatpak-generated unit is visible at
    `/home/uutzinger/Cloud/UA_Box` as `fuse.rclone`.
  - [x] Verify personal Google Drive mounted from the Flatpak-generated unit is
    visible at `/home/uutzinger/Cloud/uutzinger_GoogleDrive` as `fuse.rclone`.
  - [x] Verify SMB Online Mount content was visible through the host file
    manager/process view with Cisco VPN connected.
- [x] Verify NetworkManager and Cisco VPN detection, asynchronous popup status,
  activation readiness, and disconnect-only-if-activated behavior.
  - [x] Verify GUI Detect VPNs imports/detects existing VPN choices from the
    prototype Flatpak.
  - [x] Re-run automated NetworkManager/Cisco VPN parser, readiness, and
    disconnect-ownership tests after Flatpak host-runner changes.
  - [x] Verify popup asynchronous VPN status from the prototype Flatpak.
  - [x] Verify popup VPN activation readiness and disconnect-only-if-activated
    behavior from the prototype Flatpak.
- [x] Verify metered-network pause behavior and network-loss recovery policy
  with automated tests; live network disruption was not performed during this
  Flatpak pass.
- [x] Repeat the data-integrity safety tests that prevent overlapping Online
  Mount and Offline Mirror targets or engines.
- [x] Capture known sandbox limitations and any behavior that differs from the
  Debian/native installation.

### User Documentation and Release Preparation

- [x] Add Flatpak build, local install, update, run, troubleshooting, and
  uninstall instructions to `README.md` without displacing native Debian/source
  installation instructions.
- [x] State clearly whether host dependencies must be installed separately and
  how the Flatpak discovers and invokes them.
- [x] Document every non-default Flatpak permission and why the applet needs it.
- [x] Document whether existing native applet configuration is shared with or
  migrated to the Flatpak installation.
- [x] Add a Flatpak-specific warning if the package cannot safely coexist with
  the native `.deb` or source installation.
- [x] Prepare a tagged release whose version matches Cargo, AppStream, the
  Flatpak manifest source, and GitHub release artifacts.
- [x] Add the completed build and verification evidence to
  `Task List Completion Notes.md`.

### COSMIC Repository Submission

- [x] Fork and clone `https://github.com/pop-os/cosmic-flatpak`.
- [x] Add only
  `app/io.github.uutzinger.cosmic-ext-applet-mounter/io.github.uutzinger.cosmic-ext-applet-mounter.json`
  and its required generated `cargo-sources.json` or equivalent source file to
  the repository submission unless maintainers request additional files.
- [x] Run the repository's complete local validation/build target and resolve
  all errors without weakening the approved permission model.
- [x] Review the submission diff for generated files, credentials, local paths,
  test accounts, and machine-specific configuration before committing.
- [ ] Open a focused pull request containing the manifest and required
  packaging files, with links to the source repository, MIT license, tagged
  source, build instructions, AppStream metadata, and screenshots.
- [ ] In the pull request, call out the host-integration architecture and all
  permissions rather than relying on reviewers to infer them from `finish-args`.
- [ ] Complete the repository pull-request checklist: disclose AI-generated or
  AI-assisted code in commit messages, understand and be able to explain every
  submitted change, accurately describe and test the change, and certify it
  under the Developer Certificate of Origin.
- [ ] Address repository CI and maintainer review, updating the source tag/hash
  when a packaging fix requires a new application release.
The clean-profile public COSMIC remote smoke test is tracked under Deferred
validation; it follows repository merge and publication.

### Community Review

- [ ] Ask in the COSMIC App Developer/Mattermost channel for architecture review
  before submission if repository maintainers have not already answered the
  host-integration questions.
- [ ] Include the applet name, source URL, proposed manifest/PR URL,
  screenshots, `com.system76.CosmicApplet` AppStream classification, and a
  concise explanation of the required host integration.
- [ ] Ask specifically whether the applet belongs in `cosmic-flatpak`, Flathub,
  or native distribution packaging; the repository currently directs ordinary
  COSMIC applications to try Flathub first and reserves this repository for
  applets and software unsuitable for Flathub.

### Publication References

- [COSMIC Flatpak repository](https://github.com/pop-os/cosmic-flatpak)
- [Official COSMIC applet template](https://github.com/pop-os/cosmic-applet-template)
- [libcosmic project and applet documentation](https://github.com/pop-os/libcosmic)
- [Flatpak manifest documentation](https://docs.flatpak.org/en/latest/manifests.html)
- [Flathub submission documentation](https://docs.flathub.org/docs/for-app-authors/submission)


## SFTP Provider Extension — September 29, 2026

SFTP extends the earlier four-provider scope. Historical completed tasks above
retain their original scope and evidence. Requirements are FR-SFTP-001 through
FR-SFTP-016 in Section 17 of `Requirements and Specifications.md`.

### Description and specification

- [x] Add SFTP to `Applet Description.md` with rclone Online mount and Offline
  mirror, an SFTP button immediately to the right of SMB, and an SMB-style form.
- [x] Update `Requirements and Specifications.md` with the provider matrix,
  editor behavior, authentication, host verification, directory semantics,
  runtime integration, and acceptance criteria.
- [x] Record the implementation and verification work below. Documentation
  completion does not establish implemented or tested SFTP support.

### Provider and connection editor

- [x] Add the SFTP provider/backend, configuration compatibility, remote
  detection/filtering, and protected removal.
- [x] Extend compatible legacy import to identify SFTP remotes from their
  configured rclone backend, without guessing from the remote name. Unknown
  backends and non-default legacy config files are not silently imported.
- [x] Place the SFTP provider button immediately to the right of SMB; match
  styling, spacing, selection behavior, and accessible keyboard order.
- [x] Reuse the SMB editor layout and top action row with SFTP host, port 22,
  username, authentication, known-hosts file, and remote-directory fields.
  Omit SMB domain/workgroup inputs; preserve shared mode, local-path, and VPN UI.
- [x] Add conditional password, key-file/passphrase, and SSH-agent controls;
  mask and clear secrets and keep them out of applet configuration and logs.
- [x] Implement explicit Create/Update SFTP Remote in Add mode, existing-remote
  selection, backend validation, shared-remote update feedback, preservation of
  unchanged options/secrets, and authentication-mode transitions.
- [x] Preserve Modify's provider/mode lock, prefill non-secret remote settings,
  and provide explicit Update SFTP Remote with delayed field help matching SMB.
- [x] Move SFTP editor labels and hover help into the English Fluent catalog,
  including remote setup, authentication, directory, and preload text. Other
  language catalogs can add translations when available.
- [x] Make Create/Update SFTP Remote use the blue/theme-accent suggested style
  when adding a new remote. Unchanged existing remotes retain standard styling.
- [x] Add delayed hover help to Password, SSH Key, and SSH Agent choices.

### Runtime and safety

- [x] Preserve blank, relative, and absolute SFTP directories in connection
  validation, access-test targets, mount plans, and bisync plans.
- [x] Verify SFTP blank, relative, and absolute directory semantics through
  legacy import preview and generated replacement-plan tests.
- [x] Require a known-hosts path in setup and before Test/Save access checks;
  leave unknown/changed-key handling to rclone without adding or replacing keys.
- [x] Live-verify missing/unreadable key and known-hosts files and
  unknown/changed host-key errors against a disposable localhost SFTP server.
- [x] Use the bounded read-only rclone access check for SFTP Test Connection and
  before mounting the generated host service; validate the selected directory.
- [x] Classify SFTP host, authentication, host-key, directory, permission,
  key/known-hosts file, and SSH-agent failures in Test Connection and mount
  preflight; return short guidance without echoing raw rclone diagnostics.
  Automated cases cover each category, fallback, timeout, and redaction.
- [x] Verify that generated host services can read host rclone, key, and
  known-hosts paths and use an available unlocked SSH agent in native and Flatpak
  execution. An isolated service rendered by the applet's SFTP mount plan and
  unit generator mounted and read disposable data with a key file and a private
  agent. The Flatpak GUI prototype's host bridge read the same SFTP data with
  both methods and inspected the generated unit. Installed native GUI actions
  are in the next-release gate; public Flatpak acceptance is deferred.
- [x] Wire SFTP into the shared rclone mount/mirror provider branches and
  generated plans. SFTP preload defaults to disabled; retain bounded rclone
  defaults rather than SMB-specific timeout settings.
- [x] Correct SFTP Online lifecycle plumbing exposed by disposable live tests:
  skip redundant lazy unmount after systemd has detached the mount, preserve
  multiline systemd output during redaction, read the host mount table from
  Flatpak, and report disconnected NetworkManager state accurately. Unit and
  disposable localhost SFTP lifecycle tests passed.
- [x] Enforce metered-network policy at each scheduled rclone mirror run with a
  generated service condition. A disposable systemd probe skipped the command
  for a metered response and ran it for an unmetered response without changing
  the real network. The later SFTP generated-unit probe confirmed a metered
  skip in the host journal; installed applet controls remain in the release gate.
- [x] Put new local and remote bisync recovery copies in dated, applet-marked
  directories with unique run suffixes, and prune only marked directories after
  at least 30 days. A disposable local/alias integration test passed creation,
  preservation, and aged cleanup; preexisting unmarked files remain untouched.
- [x] Live-check dated recovery and cleanup through an SFTP-only server. The
  applet's managed runner preserved deletion recovery, pruned old marked local
  and remote folders, and left old unmarked folders untouched.
- [x] Verify an applet-generated native SFTP mirror service/timer uses the
  managed runner. An accelerated disposable timer fired the service and synced
  a file; systemd confirmed the metered condition skipped a later run. The
  Flatpak prototype's host bridge read both generated units.
- [x] On disposable SFTP data, verify initial and repeat dry previews, confirmed
  two-way sync, both conflict versions, deletion propagation into the recovery
  directory, and byte-equal completion after an interrupted transfer.
- [x] Verify initial mirror preview/sync against a disposable SFTP-only server
  that rejects shell commands, with client shell use and hash checks disabled.
- [x] Verify a transient user timer starts an SFTP bisync service and transfers
  a file. A later applet-generated timer and metered-policy probe also passed.
- [x] Add rclone's two-minute renewable `--max-lock` to every generated bisync
  service and preview, and require the capability in dependency checks. A live
  disposable test confirmed active and fresh locks block another run; an
  expired lock allows `--recover` without `--resync` when prior listings exist.
  The applet gives path-free lock guidance. It never deletes an active lock.
Legacy unbounded locks and interrupted runs without prior bisync listings need
a reviewed recovery path. This is deferred implementation work, not a new
SFTP-release test; document the limitation before the next release. Any
destructive state rebuild/resync must still require preview and confirmation.

Installed SFTP notice, Online lifecycle, and Offline mirror acceptance is
tracked only in Remaining tests before the next SFTP release below. Public
Flatpak acceptance is tracked under Deferred validation.

### SFTP preload — approved implementation

User approved optional directory preload with configurable depth/duration and
an SFTP row immediately below SMB in General Settings. Defaults are off,
30 seconds, and depth 2. The user explicitly required excluding `/proc`, `/sys`,
and `/dev`. This supersedes the initial no-preload scope and deferred note.

- [x] Update the source description and requirements with defaults, settings-row
  placement, per-connection overrides, and remote-system-directory exclusions.
- [x] Add the SFTP settings row, persistence, range validation, and migration
  defaults that preserve existing provider settings.
- [x] Add SFTP Online connection global-policy inheritance and independent
  enabled/duration/depth overrides in Add/Modify.
- [x] Connect SFTP to the bounded directory-only preload lifecycle, including
  restored mounts and cancellation before unmount, repair, removal, or sleep.
- [x] Prune `/proc`, `/sys`, and `/dev` for absolute server-root mounts; skip
  preload entirely for targets inside those trees and never follow symlinks.
- [x] Validate migration, overrides, depth limits, cancellation, and real local
  traversal exclusions, including mountpoints containing glob characters.
- [x] Complete the full post-preload regression run on September 30, 2026:
  214 tests passed (128 library, 86 binary), with no failures or ignored tests.
  Formatting and all-target/all-feature Clippy with warnings denied also pass.
- [x] User compiled and installed the updated applet and reported that the SFTP
  preload settings are visible. Saved configuration confirms enabled preload,
  30 seconds, depth 2, and global-policy inheritance for the VPS connection.
- [x] Visually verify the SFTP row below SMB and the saved per-connection
  preload override in native and Flatpak Modify. The VPS connection loaded
  global off, preload off, 30 seconds, and depth 2 in both builds.
- [x] User observed the VPS preload running and then stopping after 36 seconds
  with a saved 30-second limit. This confirms the live start/timeout workflow,
  not completion of the directory scan or strict deadline compliance.
- [x] User repeated VPS preload with 60 seconds and depth 3 and reported a stop
  after 62 seconds. Read-only configuration inspection confirms these settings.
- [x] Clarify acceptance: elapsed time may exceed the requested time limit while
  work stops. A strict wall-clock cutoff is not required (user, September 30).
- [x] Use shared completion/timeout reporting for all preload mechanisms:
  Google Drive recursive refresh and OneDrive, Box, SMB, and SFTP directory walks.
  Initially reported depth, configured limit, and elapsed time; shortened after
  installed testing feedback below. Excluded SFTP targets report skipped.
- [x] Validate shared reporting and lifecycle behavior: 218 tests passed
  (132 library, 86 binary), formatting, and all-target/all-feature Clippy with
  warnings denied. The private D-Bus test passed with approved host access.
- [x] User installed and tested the shared notices; reported that timing details
  made them too long to read before dismissal.
- [x] Shorten notices across all preload mechanisms to completion (with depth),
  time limit reached/incomplete, or skipped; remove both times and routine prose.
- [x] User confirmed the shortened notices are checked in the installed applet.
  This accepts their readability; individual completion/timeout scenarios were
  not separately reported and browsing-speed improvement remains unverified.
- [x] Compare first-time directory browsing on four fresh, temporary, read-only
  VPS SFTP mounts in disabled/enabled/enabled/disabled order. With the current
  20-second, depth-2 preload, both enabled walks timed out; first listings
  nonetheless improved for `/`, `/etc`, and `/home`, while `/var` stayed about
  the same. Two runs per condition were consistent. This supports useful partial
  cache warming for reached directories, not completion of all requested levels.
  The active applet mount and saved settings were left unchanged.
- [x] Superseded optional benchmark: a longer limit to finish all depth-2
  directories is not needed for the approved partial-warming goal. The fresh
  mount comparison above showed benefit even when the preload timed out.
- [x] User live-tested unmount during SFTP preload and observed a busy FUSE
  failure requiring Repair. The test does not pass; journal confirmed rclone's
  clean unmount failed with `Device or resource busy` at 01:14:28 on September 30.
- [x] Implement a shared strict cancellation path for OneDrive, Box, SMB, and
  SFTP directory walks: wait for the child to exit before manual unmount, and
  leave the mount service running if cancellation does not finish. Show
  "Unmounting… stopping background preload first" while applicable; suppress
  a late preload notice from replacing that progress message. Google Drive's
  separate RC refresh still receives `job/stop` before clean detach.
- [x] Attempt clean FUSE detach before stopping generated Online services so a
  busy mount stays usable for retry instead of becoming a stale endpoint. A
  disposable local FUSE test confirmed busy refusal, retained mount, and clean
  retry after its holder exited. Automated tests and Clippy passed.
- [x] Investigate the user's subsequent installed-test failure: progress notice
  appeared, followed by a `fusermount3` failure. The latest service log showed
  a clean stop and the mount was absent on inspection. The exact exit detail
  was not retained, so whether the mount was busy or already detached at the
  instant of failure remains unproven.
- [x] Reconcile `fusermount3` failure with the mount table and retry transient
  busy clean detach for up to two seconds after preload cancellation, keeping
  the service running if the mount remains busy. The user build was installed
  September 30; the later 60-second cancellation build received live validation.
- [x] Final source validation passed: 220 tests (133 library, 87 binary),
  formatting, all-target/all-feature Clippy with warnings denied, and diff
  whitespace checks. `just install-user` completed and the installed binary
  matches the tested release artifact.
- [x] Fix and live-retest SFTP unmount during an active directory traversal.
  A read-only VPS probe reproduced the exact strict-cancellation error after
  ten seconds; the native `find` child can take longer to release FUSE. Raise
  the bounded unmount cancellation wait to 60 seconds. A second live test
  confirmed `find` was running, then exercised the production applet unmount
  path: preload canceled, clean unmount succeeded after 17.6 seconds, mount
  disappeared, and service became inactive/successful. The temporary live-test
  code was removed; 220 regression tests, Clippy, formatting, and diff checks
  pass. The separate VPS browsing comparison above tested partial cache warming.
- [x] Install the tested 60-second cancellation build with `just install-user`;
  the installed binary matches the release artifact.
- [x] User confirmed the installed SFTP mount-cancellation sequence worked as
  expected after the 60-second guard update. This closes the live SFTP unmount
  acceptance item; exact notice wording and timing were not separately recorded.
Other-provider shared cancellation and Google Drive RC refresh live checks are
tracked under Deferred validation. The release gate retains a focused
existing-provider regression after any shared-code fix.

### Verification and delivery

- [x] Visually verify the Create/Update action styling and all three
  authentication-choice tooltips in the installed native app and Flatpak GUI
  build. The user confirmed all three help bubbles were readable and distinct.
- [x] User reopened the existing SFTP connection in Modify, checked it, and
  reported that it worked. This confirms the basic Modify workflow; credential
  rotation and failure handling are not separately verified.
- [x] User tested an invalid SFTP entry: missing local mountpoint was reported
  correctly, and incorrect username/password produced an error. Exact wording
  and the action producing the authentication/setup error remain unrecorded.
- [x] Retain Save Connection's blue suggested style for SFTP and allow saving
  after configuration/plan validation without requiring a successful access
  test, as requested by the user. Save does not apply server credentials.
- [x] Highlight Update SFTP Remote when server/authentication fields change,
  including a newly entered password. Clear after successful application;
  preserve pending changes after failure or edits made during an update.
- [x] Visually verify blue Save and Create/Update in Add, and blue Update after
  an unsaved host edit in Flatpak Modify. No remote settings were applied.
- [x] Preserve the highlighted Update action after failed SFTP setup, require
  re-entry after a failed secret change, and distinguish config-read, save, and
  access timeouts without exposing rclone diagnostics. Focused tests passed;
  a disposable rclone remote accepted a correct password, rejected an updated
  wrong password, and worked again after correction. Installed-app notice
  presentation is in the next-release gate.
- [x] User-confirmed SFTP setup and Online mount workflow on the VPS after the
  Create/Update, Test Connection, Save, and mount instructions. Authentication
  method and package type were not recorded; individual write/delete/unmount
  results were not separately reported. See the live follow-up in completion
  notes. Broader acceptance is in the next-release gate.
- [x] Run focused automated checks for configuration compatibility, discovery,
  validation, setup/update commands, secret handling, authentication changes,
  remote-path preservation, and generated mount/mirror targets. The narrow
  remaining release checks are listed at the end of this document.
- [x] Verify native and Flatpak UI placement beside SMB, similar form layout,
  conditional fields, Add/Modify actions, keyboard navigation, and no clipping
  in the reviewed editor sections. The locally built Flatpak prototype was also
  installed temporarily and launched in Add and Modify, then uninstalled.
  User-guided keyboard checks passed after focusing an editor field; neither
  test draft was saved.
- [x] Superseded duplicate: generated-service key and agent access is covered
  by the completed host-service probe above; startup-at-login behavior belongs
  to the ordinary service policy, not another SFTP-only test.

The focused installed failed-update, Online safety/wake, and Offline control
checks are consolidated in the next-release gate at the end of this list.
Direct password, key, agent, host-key, timeout, and SFTP-only protocol probes
already passed; exhaustive installed error variants are deferred.
- [x] Run all-target compilation, formatting, all-target/all-feature Clippy
  with warnings denied, all-target tests, and diff validation. All 208 tests
  pass, including 14 new SFTP checks and the existing-provider regressions.
- [x] Check installed rclone 1.75.0 setup/update behavior using only a disposable
  configuration and dummy credentials; verify obscuring and empty-secret
  handling without contacting a server.
- [x] Record source/test evidence separately from pending installed-service
  and live-server acceptance in `Task List Completion Notes.md`.
- [x] Add SFTP to the README's supported providers, engine table, authentication,
  and preload defaults with minimal wording.
- [ ] Update dependency/setup guidance and release metadata to describe
  delivered SFTP support and any verified limitations before release.

Editor implementation evidence is recorded under “SFTP Provider and Connection
Editor — September 29, 2026” in `Task List Completion Notes.md`.

### SFTP checklist audit — September 30, 2026

Historical September 30 audit: legacy import and directory semantics were
source-complete; the longer preload benchmark and duplicate host-service
authentication item were superseded. Subsequent host-service, mirror, and
visual checks are recorded above. The current release test gate and deferred
validation lists are at the end of this document; this audit does not add
separate acceptance requirements. Release documentation remains open until
the supported SFTP scope and verified limitations are stated accurately.


## Provider editor localization — September 30, 2026

- [x] Move OneDrive, Google Drive, Box, and SMB provider names, setup actions,
  input placeholders, and field help into the English Fluent catalog. Include
  OneDrive mirror auth handoff, provider preload help, and shared rclone remote
  controls. SFTP editor strings were already cataloged above.
- [x] Compile the catalog, run the complete automated test suite, and check
  formatting and Clippy. Other-language catalogs and installed visual checks
  remain separate from this source change.

## Remaining tests before the next SFTP release — October 1, 2026

This is the remaining test gate for the next native SFTP-enabled release. The
installed Add/Modify UI, authentication methods, successful VPS mount, preload
and unmount sequence, and disposable Online/Offline data paths have already
been checked above and in Completion Notes. Do not repeat those tests unless
a later fix changes them. Use disposable credentials and data for the focused
checks below. Public COSMIC Flatpak publication and its installed-app tests are
deferred until that distribution is available.

- [x] **Failed credential update in the installed native editor:** make one
  disposable rclone configuration temporarily unwritable, attempt a password
  update, and confirm the Update action stays highlighted with a redacted
  failure notice. Restore write access, re-enter the correct password, confirm
  Update clears after saving, and use Test Connection to verify authentication.
  A wrong password by itself is a successful config update followed by an
  access failure; it is not the save-failure trigger for this check.
- [x] **Online safety and wake edge cases:** on a disposable SFTP mount, verify
  a pending upload prevents unsafe cleanup, then complete one normal
  sleep-cleanup/wake-restore cycle if that option is enabled. Existing mount,
  unmount, Repair, cache-health, and localhost server-recovery probes need no
  repeat. Record the known cached-read I/O error during an outage as a release
  limitation or fix it before release. Real VPN transitions remain deferred.
- [x] **Installed native Offline controls:** use disposable data to verify the
  Preview → confirmed initial Sync Now → Start/Stop path and visible status in
  the installed applet. Command-level conflict, deletion, interrupted-transfer,
  SFTP-only, generated-timer, metered, and retention checks already passed and
  need no repeat through the UI.
- [x] **Final release-candidate check:** after the last code change, run the
  automated suite, formatting, Clippy, metadata validators, and native package
  installation check. Perform only a brief SFTP smoke test plus focused tests
  for any changed shared code; record the exact tested build/version.

## Deferred validation after the SFTP native release — October 1, 2026

These are follow-ups, not additional gates for the native SFTP release above.
Move one here into the release gate only if its supported behavior changes or
the final tests expose a related defect.

- [ ] After the public COSMIC Flatpak repository/package is available, test
  installed Flatpak SFTP editor notices, Online and Offline controls, generated
  host services/timers, and a clean-profile mount/mirror/VPN/uninstall smoke
  test. The local Flatpak prototype and host bridge have separate evidence.
- [ ] If later needed, broaden installed SFTP error-notice checks to host-key,
  unreadable-file, permission, agent, and timeout variants, and live-check a
  VPN-bound SFTP connection. Direct protocol and automated classifier tests
  already cover these errors; they are not repeat gates for the native release.
- [ ] Broaden Google Drive `--fast-list` testing to a larger ordinary drive and
  shared drive, including API/rate-limit behavior; repeat cloud cold-cache
  preload trials without reading or changing file contents.
- [ ] Live-check Box and SMB preload on the UA and LAN shares, and shared
  cancellation on OneDrive, Box, SMB, and Google Drive's RC refresh when
  suitable active-preload shares are available.
- [ ] Complete broader installed native/Flatpak Settings title, focus,
  keyboard, scaling, long-list, editor-save, and background-poll visual checks.
- [ ] Exercise the full Cisco/NetworkManager password/MFA, concurrent-request,
  and rejected-credential path with a real VPN profile; add the separately
  planned waiting/Cancel/Retry UI before accepting that extended workflow.
  Include a VPN-bound SFTP connection when that combination is available.
- [ ] Exercise repeated logind/inhibitor and native/Flatpak lid/menu suspend
  fault cases, then investigate the separately reported OneDrive sleep hang
  with journal evidence. The native SFTP release gate above includes one
  normal sleep/wake cycle only.

## SharePoint (Teams Files) Provider — Planning, September 30, 2026

SharePoint library connections are a future provider. Keep Online mount and
Offline mirror as separate modes; implement and validate Online mount first.
No SharePoint applet integration or live acceptance is claimed yet.

- [x] Add the planned **Teams** engine for SharePoint libraries and two-mode strategy
  to `Applet Description.md`.
- [x] Add a requirements placeholder in Section 18 of
  `Requirements and Specifications.md`.
- [x] Record implementation and acceptance placeholders here.
- [x] Define exact site/library URL parsing, persisted library identity,
  preconfigured rclone remote selection, OAuth/tenant-consent errors, preload
  policy, and native/Flatpak acceptance before implementation. The detailed
  plan and unresolved authenticated-metadata probe are recorded in the new
  October 1 planning section below.

The unfinished Online and Offline implementation and acceptance items from
this placeholder are expanded in the October 1 implementation plan below.

Example site:
https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment
Example library URL path segment (not a local mountpoint):
Shared%20Documents

## Teams (SharePoint library) implementation plan — October 1, 2026

This expands the September 30 placeholder above. Online mount is the first
deliverable; Offline mirror is a separate later milestone. The Online implementation
is in progress; its authenticated library identity check and an installed native
read-only mount passed against the university site. Use only a disposable library/folder
and files for write, delete, conflict, or sync tests. Section 18 of
`Requirements and Specifications.md` defines the behavior and release gates.

- [x] Check the upstream engine design and installed tools: rclone 1.75.0
  supports SharePoint document libraries through its `onedrive` backend, but
  `rclone backend help onedrive` exposes no metadata command here; installed
  `abraunegg/onedrive` is 2.5.11. At this planning stage, authenticated Graph
  identity lookup and tenant consent still needed a read-only feasibility probe.
- [x] Start the example-site feasibility probe: the configured rclone remotes
  have no `onedrive` backend, and an unauthenticated HEAD request for the example
  library returns a sign-in-required 403. Neither result establishes the
  library's identity or the work account's access. The user subsequently
  created a dedicated remote; authenticated verification is recorded below.

### Identity and first usable Online mount

- [x] Run a read-only feasibility probe with a preconfigured rclone `onedrive`
  document-library remote and delegated Microsoft access: obtain its `drive_id`,
  `drive_type`, and authenticated library `webUrl` (Graph drive metadata or an
  equivalent authenticated engine response). Prove that the example URL maps
  to the expected site and library without displaying or persisting tokens. If
  tenant policy blocks identity verification, retain a clear blocked state;
  do not silently accept a remote name or URL as proof. Completed with
  `ua_teams_engr_bme_assessment`: configured and Graph drive types were
  `documentLibrary`, drive IDs matched, authenticated `webUrl` exactly matched
  the example library, and a read-only root listing succeeded.
- [x] Add a distinct SharePoint provider and version-tolerant connection
  identity fields: canonical site URL, library URL/name, verified drive ID,
  selected rclone remote, and optional folder. Preserve old configurations and
  existing OneDrive engine selection. Reject stale IDs or remote retargeting
  before mount; avoid credential storage in applet configuration.
- [x] Implement a conservative URL parser for HTTPS SharePoint site/library
  links, including percent-decoding once, path boundaries, case/encoding
  normalization, query/fragment stripping, and `/sites/` or `/teams/` site
  paths. Reject ambiguous sharing/view links and require an explicit library
  choice when the URL cannot identify one. Test the example URL; `Shared%20Documents`
  is the library URL segment, while the local mountpoint is chosen separately.
- [x] Add a **Teams** button beside OneDrive in Add/Modify and collect the
  preconfigured document-library remote, library URL, optional folder, and
  local path. Verify the site, library, drive, and signed-in account when
  available before Save; show `account unavailable` when Microsoft Graph does
  not supply an account. Keep Test Connection read-only. The installed native
  Add editor saved the assessment connection, and an unsaved draft passed Test
  Connection; a nonexistent folder produced a clear error without raw rclone
  output. The saved Modify editor displayed the remote, URL, optional folder,
  target, and verified identity correctly without clipped controls. Focused
  tests pass for consent, expired authorization, denied access, wrong remote
  type, and secret redaction. Additional installed authentication/permission
  variants remain conditional in the Online acceptance item below. In-app
  OAuth and library discovery are separate optional work.
- [x] Make Test Connection usable for an unsaved Teams connection: verify the
  selected document-library drive first, build the managed plan with that
  verified ID, then check the chosen folder. Show the verified site, library,
  drive, and available signed-in account in the result. Discard an asynchronous
  test result if the draft's remote, URL, or other settings changed meanwhile.
  This path remains read-only and does not save the connection.
- [x] Route SharePoint Online through the rclone `onedrive` backend only when
  `drive_type=documentLibrary` and the verified drive ID still match. Reuse
  service, mount, cache, VPN/network, sleep/wake, clean unmount, pending-write,
  Repair, and removal safeguards. Keep SharePoint-specific diagnostics and
  throttle-aware retries; default directory preload to off and bound it to the
  selected library/folder if later enabled.
  Teams uses the shared rclone Online lifecycle and a generated service guard
  that independently rechecks the saved SharePoint identity for manual,
  direct-systemd, and login starts. Start at login is opt-in and the saved
  setting enables or disables the guarded unit; the previous note saying it
  must stay disabled is obsolete. Directory preload is off and unavailable for
  Teams at this stage. Teams mounts limit uploads to one transfer, use bounded
  retries, and rely on rclone's OneDrive backend for server-directed throttling.
  A focused code test covers routing, guarded service generation, the login
  unit target, and preload exclusion. Installed lifecycle behavior remains in
  the separate acceptance task below.
- [x] Classify Teams read-only rclone failures without displaying raw output:
  tenant consent, expired authorization, denied access, missing library/folder,
  throttling, and network failure. Treat Graph 503 as a transient limit.
  Isolated tests cover these messages, token/path redaction, drive retargeting,
  and the verified-ID requirement in a new draft.
- [x] Add focused tests for URL and identity matching, old-config loading,
  remote retargeting, authentication/permission errors, service generation,
  secret redaction, and no regression to existing OneDrive connections.
  URL/identity, old-config, remote filtering, and service-plan tests are in
  place. Isolated permission, retargeting, secret-redaction, Graph status,
  remote-config, and service-routing checks are also complete. Installed
  service behavior remains in the acceptance task below.
- [x] Review the updated Test Connection in the installed native Add editor.
  An unsaved Teams draft passed with the verified site, library, drive, and
  available account detail. An intentionally absent optional folder produced a
  clear missing-folder notice without raw rclone output; the draft was not saved.
- [x] Validate basic installed native Online writes with one disposable folder
  in the assessment library. The installed identity guard passed read-only;
  the saved mount started with zero queued uploads and cache errors. A small
  file was created and edited twice through the mount, and each version was
  read independently from SharePoint. With the file held open, VFS reported a
  queued upload after close, then one upload in progress, then zero queued,
  active, or errored files. Only the disposable file and folder were removed;
  SharePoint confirmed the folder absent. Clean FUSE detach and service stop
  succeeded. The service is inactive, the mountpoint empty, and no cache entry
  is dirty. Earlier installed read and applet-toggle unmount checks also passed.
- [x] Check installed native Repair and applet status transitions. The saved
  assessment connection mounted with an active switch, then cleanly unmounted
  with a grey switch. A temporary Teams-only systemd override made the stopped
  service fail before rclone could start; the override was removed immediately.
  The popup then reported `Current status: Error`, and its two-click switch
  action reported `Repair completed`. The service is now inactive with a
  successful result, the Teams target is unmounted and empty, and the generated
  unit has no test override. Repair is on the popup switch in Error state; it
  is not a button in Modify.
- [x] Check the installed Teams identity guard's network-failure and recovery
  messages without interrupting other mounts. A proxy limited to one read-only
  guard invocation caused the clear `Could not reach SharePoint` notice; the
  same guard passed again with normal network settings. The configured remote's
  `General/` standard-channel folder was also listed successfully. This does
  not claim a live applet mount recovered from a host-wide network outage.
- [ ] Check a full installed mount through a Teams-only lost/recovered network
  event if it can be isolated from other active connections. Check a separate
  private/shared-channel site if one is available and record tenant permission
  limits. Test additional installed authentication/consent diagnostics only
  with disposable credentials or a safe simulated failure. Run packaged-host
  checks in Flatpak when its public distribution path exists; do not repeat
  passed assessment-library checks unless a failure or code change warrants it.

### Optional usability follow-up

- [x] Plan and implement URL-guided Microsoft OAuth before Offline mirror.
  Add **Connect Microsoft Account** in Teams Add/Modify. Require an unused
  rclone remote name and exact library URL, run rclone's browser authorization,
  discover the site's drives, and select only a `documentLibrary` whose
  authenticated Graph URL matches the entered URL. Verify the completed remote
  before offering Test Connection/Save, clean up a partial new remote on failure,
  and keep manual remote selection. Account switching creates a separate remote;
  existing Teams and OneDrive remotes are never updated. Focused library and
  binary tests pass; live browser sign-in is tracked below. OAuth editor labels,
  notices, and setup errors use the Fluent catalog.
- [x] Live-check native Teams browser OAuth with a new disposable remote name
  and the assessment library URL. The first attempt exposed a non-JSON rclone
  completion response; the applet now accepts it only after the new remote
  passes the same read-only identity gate as a manual remote. Browser sign-in
  then created `ua_teams_oauth_probe` with `type=onedrive`,
  `drive_type=documentLibrary`, and the same verified drive ID as the working
  assessment remote. The editor reported the verified library and `account
  unavailable` because Graph did not return `/me`; its unsaved Test Connection
  passed. An independent root listing and a transient user-service listing
  both succeeded. The unsaved editor was closed and only the disposable probe
  remote was removed; the working assessment remote remains. No SharePoint file
  or saved connection was changed.
- [ ] Check Teams OAuth tenant-consent/authorization failure feedback with a
  disposable account or safely simulated provider response. Check Flatpak
  host-browser and credential access when the public package path exists.
- [ ] Optionally add tenant-wide site/library browsing without a pasted library
  URL. Keep URL-guided discovery and manual remote selection as fallbacks.


### Installed Teams write-test interruption — October 1, 2026

- [x] Stop the installed assessment mount after the disposable write-test
  preparation exposed repeated uploads of existing files. The service logged
  SharePoint throttling and file-size mismatch errors; its VFS cache was about
  1.2 GB. The user authorized stopping the service. The service is inactive,
  the mount detached, and the cache was preserved. The empty disposable
  `CloudMounter-Test-20261001-a3d674dd9b09` folder was removed.
  The user confirmed the Box source was mounted during the copy. Of 652 VFS
  metadata entries, 386 Office files (about 697 MB) remain marked dirty;
  266 are clean. A separate local cache snapshot was made at
  `~/.local/state/cosmic-ext-applet-mounter/teams-cache-snapshot-20261001-1750`
  with matching file count and byte total. Do not restart the Teams mount
  until the queued copies are reconciled.
- [x] Determine which cached uploads were intentional and reconcile each
  affected file with SharePoint. The user confirmed the Box-to-Teams copy was
  intentional. The read-only audit identified 375 absent remote paths and 11
  already-present files; the user verified those 11 in Teams Client before the
  controlled retry. The original VFS cache snapshot was preserved.
- [x] Reproduce the SharePoint Office-file size-change behavior with isolated
  disposable data. A 4,997-byte DOCX failed a plain `rclone copyto` after
  SharePoint expanded it; `--ignore-checksum --ignore-size` completed the copy,
  and the downloaded DOCX remained a valid archive with matching document text.
  An isolated rclone VFS mount using the same flags also uploaded the DOCX and
  cleared its dirty state. The disposable SharePoint folder and isolated mount
  were removed. Those flags were added only to generated Teams online mounts;
  focused provider tests and Clippy pass. The installed assessment mount remains
  stopped, and its queued uploads have not been retried. These flags relax
  transfer verification, so recovery still requires independent review.
- [x] Build a read-only recovery inventory from the stopped VFS cache and
  compare the 386 dirty files with the assessment SharePoint library and the
  specified Box source folder. SharePoint reports 375 paths absent and 11
  present with different sizes; no dirty file matched by size. Box provides
  43 exact-path-and-size matches and 300 unique filename-and-size matches in
  another folder. Thirty-one Box matches are ambiguous and 12 need manual
  source review. Three absent SharePoint paths were independently spot-checked.
  The private 386-row audit is saved at
  `~/.local/state/cosmic-ext-applet-mounter/teams-upload-audit-20261001.csv`;
  Teams remains stopped and the cache snapshot remains intact.
- [x] Review the 11 SharePoint files with different sizes before considering
  deletion. The user opened all 11 through the independent Teams client and
  confirmed they are present, open correctly, and contain the expected content.
  Treat these as usable SharePoint copies; a size difference alone is not
  evidence of an incomplete file. All 11 were also downloaded read-only to
  `~/.local/state/cosmic-ext-applet-mounter/teams-present-11-20261001` before
  retry; each is a readable Office archive. One downloaded file differs in size
  from SharePoint's own reported size. No deletion is proposed.
- [x] Build and install the Teams-only rclone flag change with `just install-user`.
  The installed binary matches the release build; the Teams service remained
  inactive and its 1.2 GB VFS cache was not retried by installation. A second
  build added `--transfers 1` for Teams to limit SharePoint retry pressure;
  focused provider tests and Clippy passed, and the panel was reloaded.
- [x] With the preserved cache and SharePoint copies available, reload the
  installed applet and make one monitored Teams mount to retry the 386 queued
  uploads. Confirm the generated service contains `--ignore-checksum` and
  `--ignore-size`, watch upload errors and dirty-file count, then verify remote
  content before allowing normal cache cleanup. Systemd loaded both flags and
  `--transfers 1`. All 375 previously absent paths uploaded and were present
  in a subsequent read-only SharePoint inventory. The 11 already-present files
  alone remained dirty after SharePoint returned 404 upload-session errors on
  replacement attempts. The service was stopped rather than retrying them
  indefinitely; no SharePoint file was deleted.
- [x] Resolve the 31 ambiguous and 12 unmatched Box sources as a conditional
  recovery fallback. No Box source reconstruction was needed: all 375 missing
  SharePoint paths uploaded from the preserved cache. The Box path ambiguity
  does not block this recovery; retain the audit if source mapping is needed
  later.
- [x] Preserve and retire the stale dirty cache safely. Downloaded all 11
  remote files again after retry; all were readable Office archives. Seven
  were byte-identical to the pre-retry downloads; the other four changed only
  `docProps/custom.xml`. With the service stopped, moved the whole post-retry
  cache to `~/.local/state/cosmic-ext-applet-mounter/teams-cache-after-retry-20261001`
  rather than editing rclone metadata or deleting files. Restarted the Teams
  service with a fresh cache: mount active, zero dirty entries, and no new
  warning logs. A newly uploaded DOCX and an existing PPTX opened through the
  mount as valid archives. Four larger recovered Office files also opened as
  valid archives with no missing original entries; their main document, slide,
  or workbook content entries matched the cached originals. Keep the original
  and post-retry cache archives and the pre/post-retry copies of the 11 files
  until the user is satisfied with the mounted library.

### Teams Online mount safety follow-up — October 2, 2026

- [x] Add a service-level Teams library identity guard. Generated Teams Online
  units now verify the saved connection binding and authenticated SharePoint
  drive ID, type, and library URL before rclone starts, including direct systemd
  starts and starts at login. A skipped guard is reported as a blocked mount in
  the applet. Other mount engines retain their existing units.
- [x] Cover changed or disabled connections, unit rendering, absent subfolders,
  and SharePoint URL escaping with focused tests. The library suite passes.
- [x] Install the updated binary and refresh the saved assessment unit while
  stopped. Systemd verified and loaded the guarded unit. The installed guard
  passed with the saved SharePoint identity and rejected a deliberately wrong
  drive ID. A direct `systemctl --user start` ran the guard successfully before
  mounting; the service was then stopped, and the mountpoint is unmounted.
  Preserve the cache archives until the user is satisfied with the library.

### UA OneDrive reset and sleep incidents — October 2, 2026

These are separate incidents. Empty local directories were seen beneath the UA OneDrive
mountpoint after a hard reset several days earlier. The sleep/wake failure and
three-toggle Repair sequence occurred this morning.

- [x] Identify the affected connection: **UA OneDrive**, `onedriver` Online
  mount, service `cosmic-mounter-990cc48f-4e4e-4ed7-a07b-c545ad3d3f9d.service`.
  It is currently active and mounted; its service reports success and no restart
  limit. The current mounted view hides any underlying local files.
- [x] Inventory the earlier obstruction from the user's September 29 repair
  record: while unmounted, the mountpoint held only an empty
  `BME/BME497G/2026` directory chain. onedriver repeatedly reported
  `Mountpoint must be empty`; the record then reports that mounting resumed.
  The omitted portion does not confirm the exact recovery command or which
  process created the directories.
- [ ] Determine how local directories appeared while the share was unmounted.
  Preserve any moved-aside copy if it still exists; do not unmount the currently
  working share merely to recreate this older incident.
- [x] Establish this morning's sleep failure from the journal: at 05:55 on
  October 2, `fusermount3` returned `Device or resource busy`; onedriver exited
  with status 128 while stopping, leaving cleanup incomplete. No current
  process holder proves which process held it open at that earlier moment.
- [x] Change onedriver sleep cleanup to attempt a bounded clean detach before
  stopping its service. If the FUSE mount remains busy, leave the service
  running and report the incomplete cleanup; never force or lazily detach it
  automatically. Focused busy and successful-detach tests pass.
- [ ] Install/reload the updated applet and perform one controlled sleep/wake
  check when the active OneDrive mount can be interrupted safely. Verify that
  a busy mount remains usable after wake, and that a clean mount stops and
  restores according to the saved setting without repeated Repair clicks.

### UA OneDrive earlier repair record reviewed — October 2, 2026

- [x] Separate the September 29 non-empty mountpoint from the later failed
  service state and the October 2 sleep failure. The earlier repair record
  reports an onedriver `Mountpoint must be empty` restart loop, then successful
  mounting after a proposed move of an empty `BME/BME497G/2026` hierarchy; the
  exact command is omitted from the supplied excerpt. A later
  empty-mountpoint attempt recovered with `systemctl --user reset-failed` and
  `start`; that sequence does not prove reset is required after every failure.
- [x] Check the current applet path: a manual OneDrive Mount already issues
  `reset-failed` before `start`, but it ignores errors from that reset. Setup
  validation checks that the mountpoint is a directory; it does not check that
  an unmounted onedriver target is empty. The applet creates only the target
  directory in this path, not the nested `BME/...` hierarchy.
- [x] Add a read-only empty-mountpoint preflight before onedriver starts,
  including generated-service starts, to prevent a non-empty target from
  entering a restart loop. Explain the obstruction and preserve all local
  entries; offer inspection or a separate backup location, never automatic
  deletion. Cover manual Mount, login/direct starts, and a mount that is
  already active. Implemented for all Online engines in the later section.
- [x] Make failed-state handling explicit for all Online engines: read service
  state before starting, reset only failed units, and report a failed reset or
  status read. Classify a systemd start-rate limit separately from a service
  preflight block; the shared mountpoint guard already distinguishes local
  entries from an existing mount before applet starts. Focused fake-service
  tests cover inactive starts, failed resets, start limits, and preflight
  blocks. No live connection was started for this check.
  Native user binary installed; the running panel has not been restarted.

### All Online engines: empty mountpoint preflight — October 2, 2026

- [x] Add a shared read-only host preflight for OneDrive, Teams, Google Drive,
  Box, SMB, and SFTP Online mounts. It blocks an already mounted target, a
  non-directory or symlink target, and any local entry in an unmounted target;
  it never removes or moves user data. Offline mirrors remain unaffected.
- [x] Run the preflight before applet Mount and in each generated user service,
  including login/direct starts. Teams retains its SharePoint identity check.
  An applet startup refreshes saved managed Online units without restarting
  active mounts; OneDrive also refreshes its unit on manual Mount.
- [x] Test empty, non-empty, symlink, and already mounted targets; assert each
  engine generates a guard. Library tests, applet tests excluding the unrelated
  sandbox-only private-DBus test, and Clippy pass. A read-only CLI check rejected
  the currently mounted UA OneDrive target without changing it.
- [x] Install and restart the native applet. All eight saved Online units now
  contain their guards; the one Offline mirror unit has none. Systemd loaded
  UA OneDrive's `ExecCondition`, and the installed guard accepted its currently
  empty, unmounted target. No mount service was started for this check.
- [ ] Verify a disposable non-empty target is blocked by a direct systemd
  start. Repeat the host-command check for the packaged Flatpak when that
  installation is available.

### Per-connection pending-change count — October 2, 2026

- [x] Define the compact row count and its evidence limits in Requirements:
  one number between name and switch, `0` only when currently verified,
  `+` when active with an unknown count, `-` when inactive with an unknown
  count, and no cache-size estimate masquerading as pending files.
- [x] Implement the shared count state: connection ID, access mode, observed
  count, checked time, source, and unknown/error reason. Keep observations
  transient and invalidate them on service stop, sleep, restart, network loss,
  connection edit, or failed status query. Add localized count help text.
- [x] Implement rclone Online counts for Teams, Google Drive, Box, SMB, and
  SFTP from each active mount's existing private RC socket. Read `vfs/queue`
  for queued plus uploading entries and VFS error state for confidence. Poll
  asynchronously at a bounded interval, and never block an operation or scan
  the cache/remote for this badge. Parser tests pass; installed live validation
  remains in the verification item below.
- [ ] Implement Offline mirror counts for supported engines. Use a bounded
  read-only preview/status check while idle, avoid concurrent bisync preview
  during a run, report confirmed per-run progress for manual and scheduled
  sync, and refresh the scoped count after completion. Include uploads,
  downloads, deletions, and conflicts; show age/unknown when the result is
  stale or incomplete. Use `--display-sync-status` for abraunegg/onedrive
  where available; defer Teams mirrors until their release gate passes.
  - [x] Add read-only OneDrive mirror status parsing and bounded idle bisync
    change checks for rclone mirrors. A disposable initial bisync dry run
    confirmed its output shape; status parsers have focused tests.
  - [x] Read the current scheduled bisync invocation's journal without taking
    its lock. Show a decreasing estimate only after both change summaries and
    completed file operations are reported; never infer zero before bisync
    reports success. Unknown or truncated logs use the active/inactive marker.
  - [ ] Add a reliable count during active manual and scheduled mirror runs,
    then verify that it falls to zero only after a fresh completed check.
    Manual Sync Now still has no streaming progress source. A running timer
    suppresses automatic bisync preview to avoid lock interference, so its
    idle baseline remains unknown until a safe check is available.
- [x] Put the count in each main-popup connection row between name and switch.
  Preserve switch reachability, compact layout, hover help, keyboard and
  screen-reader text, and Fluent localization. Show `+` or `-` for onedriver Online
  mounts until a supported count exists; never treat unavailable as synced.
  Source layout is implemented; installed visual/accessibility validation is
  tracked below.
- [ ] Verify with disposable data that rclone Online queue counts decrease to
  zero after upload; mirror counts update through Preview, Sync Now, scheduled
  sync, new changes, interruption, and recovery. Cover errors/stale values,
  slow providers, native and available Flatpak builds, and a user-guided
  installed-popup visual check. Update the displayed-count wording if live
  evidence shows an engine exposes only an estimate. The native build was
  installed with `just install-user`; the panel has not been reloaded for a
  visual check, and no work connection was mounted for this feature test.

### Teams Offline mirror feasibility — October 2, 2026

- [x] Confirm the installed `abraunegg/onedrive` v2.5.11 accepts a separate
  `--confdir`, `drive_id`, and `sync_dir` in an isolated local
  `--display-config` probe. The probe used a disposable ID and directory;
  it did not authenticate or contact SharePoint.
- [x] Attempt separate abraunegg authorization: the University tenant showed
  **Need admin approval** for OneDrive Client for Linux. The existing OneDrive
  mirror is a personal-account connection; it does not authorize this client
  for UA SharePoint. Do not require the user to sign in as an administrator.
- [x] Add a probe-only Teams mirror plan with a separate `teams-sync/<id>`
  configuration path, verified drive ID, path-overlap validation, and bounded
  `--display-config`/`--dry-run` requests. Generate only the isolated
  `drive_id` setting and reject unsafe ID bytes. This probe-only code was
  removed after tenant consent blocked the engine. No Teams mirror Sync Now
  or service was exposed.
- [x] With the user's approval, create the empty
  `CloudMounter-TeamsMirror-Test-20261002` folder at the verified assessment
  library root. An independent `lsjson` returned an empty list. Prepare a
  mode-0700 profile under `/tmp/cosmic-teams-mirror-probe-20261002` with only
  the verified `drive_id`; `--display-config` resolved its isolated sync path.
  No successful abraunegg authentication or dry run occurred; this profile is
  now unused.

### Teams Offline mirror rclone path — October 2, 2026

- [x] Use the already authorized `ua_teams_engr_bme_assessment` rclone remote
  as the candidate Teams Offline engine. A scoped initial `rclone bisync
  --resync --dry-run` against the empty disposable folder and empty local
  directory completed successfully, reporting no transfers. This proves
  access and command compatibility only, not safe bidirectional behavior.
- [x] Scope Teams remote recovery to the selected folder and exclude the
  recovery directory from bisync. A disposable local-to-local initial sync
  and overwrite test verified that the old file lands in the scoped recovery
  directory and that the recovery directory is not copied back into the
  mirror. Teams Offline controls remain disabled.
- [x] Live-verify the saved Teams remote against the assessment site, library,
  and drive. Confirm the approved `CloudMounter-TeamsMirror-Test-20261002`
  folder was empty before writing. A scoped SharePoint bisync dry run proposed
  only four generated text files and no deletions.
- [x] Run scoped SharePoint safety tests: initial sync and read-back,
  local-to-remote overwrite with the old version in folder-local recovery,
  remote-to-local edit, deletion with recovery, and a two-sided conflict with
  both versions preserved. A repeat sync found no changes and did not copy
  recovery into the local mirror. Generated files and recovery remain in the
  disposable test folder for inspection; no work-library files were used.
- [x] Add an automatic check of the saved site/library/drive identity and
  selected folder at the start of the shared Preview, writable sync, and
  scheduled-run script. It also binds the local mirror and recovery targets,
  checks that the remote folder is accessible, and runs before any recovery
  write. Unit tests reject changed targets and library roots. Teams Offline
  Add/Modify and Sync Now remain disabled pending installed checks.
- [x] Test Office-file changes, interruption and resumption only with
  disposable files. The later SharePoint Offline safety sections record the
  Office comparison blocker and the interruption recovery result.
- [x] Expose manual SharePoint Offline controls after the Office comparison
  and generated-runner safety gates passed. Scheduling remains disabled pending
  separate unattended network/VPN, metered, sleep/wake, and failure checks.

### Teams Offline mirror guard follow-up — October 3, 2026

- [x] Verify the source build after adding the guard: 171 library tests passed,
  3 ignored; all-target Clippy and `git diff --check` passed.
- [x] Live-test the read-only guard against the approved assessment folder:
  the saved drive/library/folder passed, while a wrong drive ID and a missing
  folder were rejected. The compiled guard command also rejected the saved
  Teams Online connection as an Offline mirror before remote access. No
  SharePoint files were changed. Use a Teams-specific localized failure notice.
- [x] In an isolated native configuration, the compiled guard accepted the
  unchanged disposable SharePoint folder. Changes to the saved remote,
  library, drive, selected folder, local target, recovery target, or enabled
  state blocked Preview, Sync Now, and scheduled service starts before rclone
  or recovery writes. The real connection and SharePoint files were untouched.
  Teams Offline editor and sync controls remain disabled.
- [x] Require a new preview and bisync state rebuild when the saved remote,
  library, drive, folder, local, or recovery target changes. Target-specific
  state directories isolate the changed target; saving a retarget now removes
  the old Preview and initial-sync markers, so changing away and back cannot
  reuse an earlier confirmation. Installed Add/Modify acceptance remains below.

### SharePoint display name — October 3, 2026

- [x] Label the document-library provider **SharePoint** in the applet,
  current descriptions, requirements, README, and user-facing notices. Keep
  Microsoft Teams files as a use case. Retain the serialized `Teams` provider,
  internal keys, saved connection IDs, rclone remote names, and historical task
  entries so existing configurations continue to load.

### Settings layout — October 3, 2026

- [x] Add a **Connections** heading above Add Connection and Refresh in General
  Settings, matching the existing Sleep and wake heading style and reusing the
  localized Connections label.

### SharePoint Offline safety tests — October 3, 2026

- [x] In a new subfolder of the approved disposable SharePoint test folder,
  reproduce Office-file growth: a 921-byte DOCX became 9,263 bytes after
  upload, causing the current bisync plan to fail initial sync. Transfer
  tolerance alone let the upload finish but bisync still rejected the unequal
  final listing. The remote DOCX remained a valid archive with the intended text.
- [x] Test a disposable interruption after about 60% of a 2 MiB transfer.
  After the two-minute lock expired, `--recover` completed the transfer; the
  downloaded file matched the original SHA-256, and a repeat run had no changes.
  A separate failed DOCX overwrite was preserved as conflict copies during
  recovery. Keep those test files for inspection.
- [x] Resolve the Office-file comparison safety gate for manual sync. Earlier
  modtime-only and size-plus-time workarounds missed or skipped disposable
  changes, while checksum plus `--ignore-times` caused extra transfers. The
  later checksum prototype uses `--compare modtime,checksum` with a one-second
  timestamp window, two bounded passes, and independent `rclone check
  --checksum`; it detected same-time edits and settled SharePoint's rewritten
  DOCX without repeated work on a clean Preview. Installed manual controls and
  separate unattended scheduling checks remain below.
- [x] Isolate SharePoint bisync work directories by a digest of the verified
  remote, library, drive, selected folder, local target, and recovery target.
  A focused test confirms changed targets cannot reuse the prior workdir,
  Preview marker, initial-sync marker, listings, or managed script path.

### SharePoint Offline checksum prototype — October 3, 2026

- [x] In another subfolder of the approved disposable folder, test
  `--compare modtime,checksum --modify-window 1s --ignore-size
  --ignore-checksum --check-sync false`. A same-size, same-timestamp local edit
  was detected by hash, uploaded, read back correctly, and followed by a clean
  Preview. The one-second window removed a false Office-file time change.
- [x] Upload a 929-byte DOCX that SharePoint rewrote to 9,273 bytes. The next
  bisync pass downloaded the valid rewritten archive and preserved the 929-byte
  original in recovery. A later local Office edit with its timestamp preserved
  also uploaded correctly; SharePoint's rewritten revision retained the edit,
  downloaded on the next pass, and left a clean Preview.
- [x] Verify a bounded two-pass strategy: after the first disposable Office
  upload, independent `rclone check --checksum` failed on the size difference;
  after the second pass it found zero differences across all four test files.
  Add SharePoint-only comparison flags and a two-pass checksum postcheck to the
  generated mirror plan. Keep Add/Modify and Sync Now disabled pending the
  generated-runner and remaining safety checks.
- [x] Live-test the generated runner in fresh disposable subfolders. Initial
  Preview/Sync, a one-file edit, dated recovery suffixes, and the independent
  checksum check passed. The generated runner also handled a rewritten DOCX
  in its two passes, retained the original in local recovery, and left a valid
  local archive. A live test caught and fixed the `rclone check` filter flag.
- [x] Keep rclone's all-files-changed safety stop active. A one-file edited
  mirror triggered that stop, so add a stable, hidden, app-owned access marker
  to the scoped local/remote mirror. The generated runner now checks the marker
  before normal runs; the one-file edit and recovery passed without `--force`.
  The applet creates the local marker before initial Preview and rejects a
  missing or altered marker after initial sync.
- [x] With the checksum comparison strategy in the disposable folder, a
  remote-only text edit downloaded and preserved the previous local copy;
  simultaneous local and remote edits became readable `.conflict1` and
  `.conflict2` copies on both sides; deleting one local conflict copy moved its
  remote counterpart into scoped, dated recovery. Independent `rclone check
  --checksum` found zero differences after the conflict and deletion runs.
- [x] Invalidate the prior target's initial Preview and initial-sync markers
  before saving a changed SharePoint Offline remote, drive, folder, local, or
  recovery target. A focused test changes targets and changes back: neither
  target can reuse its old confirmation, while bisync listings stay preserved.
- [x] Expose the installed SharePoint Offline Add form with a manual-only notice,
  recovery target, and no background interval or metered controls. The native
  Add layout was visually checked; a disposable installed connection is next.
- [x] Fix the installed Add test's false "details changed during testing" notice
  when recovery is automatic: assign a stable SharePoint draft ID before the
  test so its derived recovery path remains unchanged. A focused regression
  test covers provider selection, Offline mode, and automatic recovery.
- [x] The installed initial Preview and Sync Now synchronized only the scoped
  probe and access marker; independent checksum verification found two matching
  files and zero differences, and the generated timer stayed disabled.
- [x] Fix misleading follow-up Preview counts: bisync's "0 deleted" summary was
  counted as a deletion, while its dry-run backup move counted as a skipped
  file. Parse queued copies/deletions as actions and ignore repeated detection
  and progress lines. Focused tests cover both overwrite and genuine deletion
  output; an installed notice retest remains below.
- [x] Test installed SharePoint Offline Add/Modify, Preview, and manual Sync Now
  in an isolated disposable connection. The follow-up Preview showed one upload
  and zero deletes; Sync Now completed, two files matched the scoped remote with
  zero differences, and a fresh Preview found no changes. The timer stayed
  disabled; unattended scheduling remains a separate check.

### Clean up Main Applet. - October 3, 2026

- [x] Main applet has 3 line display at the top: x of y conenctions active, notifications enabled, VPN  usage. It appears this information is distracting and not necessary. When information needs to be displayed it usually is in a text line just below that section. That mechanism can remain but I prefer the other 3 lines to be removed, or convince me of its utility.
  - [x] Remove the static "notifications enabled" line from the main popup header.
  - [x] Remove the VPN summary line from the main popup header.
  - [x] Keep the aggregate status line, but auto-hide it after 5 seconds when the
    overall state is `Healthy`.
  - [x] Continue showing the aggregate line immediately for `Empty`, `Attention`,
    `Busy`, and `Error` states.
  - [x] Remove unused `vpn_status_pending` state, `vpn_summary()` helper,
    `connection_vpn_label()` helper, and notification i18n strings.
  - [x] Pass `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`,
    and `cargo test`.
  - [x] Live-verify the installed applet popup shows only the title/gear row after
    the brief healthy-state summary.

### Reorder Connections. - October 3, 2026

- [x] Add dedicated `ReorderConnections` application launch mode and settings
  window mode.
- [x] Add `OpenReorderConnections` message and `--reorder-connections` CLI
  argument wiring so the reorder window can be opened from General Settings.
- [x] Add a "Reorder" button in General Settings next to Add Connection and
  Refresh.
- [x] Build `view_reorder_connections()` showing each saved connection name
  with a dropdown for positions `1..N`.
- [x] Implement `ReorderConnection(connection_id, new_position)` to remove the
  connection and re-insert it at the selected 1-based index, persist through
  validated configuration writeback, and notify the runtime owner.
- [x] Refresh the reorder window immediately because the model is updated in
  place.
- [x] Ignore invalid or unchanged positions and connections that are no longer
  present.
- [x] Add i18n strings `reorder-connections`, `reorder-connections-title`, and
  `reorder-connections-empty`.
- [x] Add unit tests covering a move to a selected position and invalid/
  unchanged position handling.
- [x] Pass `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`,
  and `cargo test`.
- [x] Fix the reorder tests to use isolated temporary storage instead of the
  live runtime configuration path, preventing future test runs from overwriting
  the user's saved connections.
- [x] Add `examples/recover_config_from_units.rs` to rebuild the saved
  connection list from the managed systemd units left in
  `~/.config/systemd/user`.
- [x] Recover the user's connections after the initial reorder test run
  overwrote `~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document`
  with test fixture data.
- [ ] Live-verify the Reorder window opens from General Settings, dropdown
  changes reorder the main applet popup list, and changes persist across applet
  restarts.

- [x] Dependency installation. Converted `Dependency Installation.md` into `scripts/install-dependencies.sh`, which supports `--all`, `--rclone`, `--onedriver`, `--onedrive`, `--base`, and `--cisco-check`. It can be run via curl:
  ```sh
  curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/install-dependencies.sh | bash
  ```

### Reorder recovery audit and account verification — October 3, 2026

- [x] Back up the live v2 document privately before further changes and verify
  its checksum. Preserve the original under the applet's local state in
  `config-backups/20261003-d1Hyo4/document.original`.
- [x] Make `examples/recover_config_from_units.rs` read-only by default. It now
  writes only explicitly named, new candidate files, refuses the live document,
  validates recovered entries, preserves a valid baseline's order/global/VPN/
  per-connection settings, checks actual unit enablement, and strips the
  `Cloud Mounter:` service-description prefix. Unit-only OneDrive labels are
  unique placeholders rather than reused `onedrive` account labels. The audit
  also avoids creating OneDrive recovery directories as a side effect.
- [x] Compare the reviewed recovery output with the saved configuration:
  ten managed units and ten saved connections match by ID. The merged candidate
  preserves every saved value; a unit-only rebuild would lose sleep settings,
  VPN profiles/assignment, SFTP preload override, and connection order. Do not
  install the unit-only candidate over the live config.
- [x] Confirm the `UA OneDrive` online cache is the University business drive.
  The personal-labeled `uutzinger OneDrive` online cache contains the same
  Microsoft user, tenant, and drive, contrary to the intended personal account.
  Disable its stopped service at login and set only that saved startup flag to
  false; preserve the cache and tokens for inspection.
- [x] Confirm the separate `uutzinger OneDrive mirror` reaches a 5 GB OneDrive
  drive using its own client. Its no-sync status check reports about 333 MB of
  remote changes pending; no synchronization was run.
- [x] Restore the VPS SFTP per-connection preload override recorded in the
  earlier installed acceptance check: use global off, preload off, 30 seconds,
  depth 2. Preserve the global SFTP policy and all other settings.
- [x] Confirm saved Box, both Google Drive, SFTP, and SharePoint rclone remotes
  have the expected backend/credential fields and pass bounded read-only root
  listings. The saved SMB remote has the expected backend and credentials, but
  Cisco VPN was not active for a live access test.
- [x] Replace the three reused `onedrive` labels in the saved OneDrive entries
  with distinct work online, pending personal online, and personal mirror
  labels. Keep their connection IDs and separate credential locations. New
  OneDrive drafts now receive a unique provisional label. Preserve a private
  backup of the final corrected document in `document.after-label-fix`. The
  updated native applet was installed and the COSMIC panel reloaded; full tests,
  all-target lint, formatting, and whitespace checks passed.
- [ ] Reauthenticate the personal `uutzinger OneDrive` online connection in an
  isolated fresh cache, verify its drive differs from the University drive,
  then restore startup only if the user wants it. Preserve the old work-account
  cache until the corrected mount is accepted.
- [ ] With Cisco VPN connected, run a read-only access check for the saved
  `ua_engr` SMB remote and confirm the VPN assignment in the installed editor.

### UA Google Drive unmount incident — October 3, 2026

- [x] Diagnose and complete the failed clean unmount. The desktop document
  portal held directory handles under `UA_GoogleDrive/BME310`; rclone reported
  zero pending uploads. Temporarily stop the portal, cleanly unmount, stop the
  generated mount service, and restart the portal. Confirm the mountpoint is
  empty, the mount service inactive, and the portal active.
- [ ] Resolve separate Google Drive 403 request-quota errors. `ua_gdrive` has
  no custom OAuth client ID, so it uses rclone's shared client. Google Drive
  preload is enabled for 60 seconds with an unbounded recursive directory
  refresh, which may add a burst of requests. Temporarily disable Google Drive
  preload and configure a private client ID for the remote; then check whether
  listing errors recur. These errors did not cause the busy unmount.

### SharePoint Offline unattended release gate — October 3, 2026

The installed manual Preview and Sync Now test above is complete. Keep scheduled
sync disabled until this fixed checklist passes with disposable data scoped to
the approved SharePoint test folder. A failed test may require a focused fix and
retest; completed tests do not need to be repeated otherwise.

- [x] Expose the existing sync-interval and metered-network controls for
  SharePoint Offline, and use the normal Start/Stop schedule toggle. Saving a
  connection must still leave its timer disabled; Start must still require a
  successful Preview and confirmed initial Sync Now. The editor and popup now
  expose those controls without enabling a timer on Save.
- [x] Remove the temporary SharePoint manual-only runtime block while retaining
  the verified-target guard, access marker, network/VPN/metered checks,
  target-bound state, and non-overlap protections. Add focused regression tests
  for the initial-sync gate, Start/Stop selection, timer configuration, and
  retarget invalidation. The full locked suite passed (283 tests, 7 ignored), as
  did all-target/all-feature clippy, formatting, build, and whitespace checks.
  Both explicit live SharePoint tests also passed against a new subfolder of
  `CloudMounter-TeamsMirror-Test-20261002`: changed targets were blocked before
  rclone, and Preview/initial sync/later sync/clean repeat preserved recovery
  copies and ended with zero checksum differences.
- [x] Move the disposable mirror to a persistent local directory and confirm
  its scoped remote, recovery area, access marker, and existing files match
  before enabling any unattended run. The installed test uses
  `~/Cloud/Sharepoint_Mirrors/UA_BME_Assessment_Test`, the approved disposable
  SharePoint subfolder, and a connection-specific recovery directory; its
  access marker and initial state were confirmed before scheduling.
- [x] Run repeated scheduled syncs with a local edit and a remote edit between
  runs. Verify the intended changes arrive, a clean repeat makes no changes,
  and recovery stays inside the approved scope. Opposite-direction edits
  propagated, SharePoint's rewritten Office file was reconciled on the second
  pass, the clean repeat transferred nothing, and an independent checksum
  comparison reported zero differences.
- [x] Test a scheduled run across network or required-VPN loss and return, and
  check metered-network policy. Verify a failed or deferred run reports its
  state, preserves both sides, and resumes only when its policy allows. The
  controlled metered-network test passed: scheduling deferred the local file
  and later uploaded it after the policy allowed a run. This test connection
  has no required VPN, so the October 5 installed test used Wi-Fi loss instead:
  the read-only SharePoint access guard failed safely and boundedly, the timer
  remained enabled, and its next run after Wi-Fi returned uploaded the 52-byte
  disposable `network-recovery-test-2026-10-05.txt`. A clean follow-up and an
  independent comparison reported zero differences, and a direct remote read
  returned the expected content.
- [x] Review and test sleep/wake and interruption or failure during a scheduled
  run to the best ability of the available development computer. Per the
  revised Section 18 acceptance boundary, reliable live suspend/resume hardware
  proof is preferred but is not mandatory when the limitation and substitute
  evidence are documented. An observed applet failure still blocks release.
  Confirm Stop/Start, stale-lock handling, subsequent retry, and independent
  file comparison without lost or unreviewed changes. Stop/Start, a forced
  interruption, lock-expiry handling, retry, and zero-difference comparison
  passed. A sleep during the remote access probe exposed an unbounded wait;
  all generated remote operations now use 10-second connect and 30-second I/O
  idle timeouts. Code review confirms that applet sleep cleanup selects Online
  connections only, and the mirror-only cleanup test confirms it issues no
  commands. A reliable installed sleep/wake retry cannot be obtained on the
  current development computer, so live hardware behavior remains explicitly
  unverified; the documented code, automated, interruption, retry, and
  independent-comparison evidence satisfies this SharePoint Offline gate.
- [x] Change the saved remote, folder, or local target in an isolated disposable
  connection. Verify the old scheduled unit is blocked by the identity guard,
  and the new target requires fresh Preview and sync state before writing. The
  live guard blocked a changed target before rclone, and retargeting invalidated
  the saved Preview and initial-sync markers as intended.
- [x] Verify installed native notices and status for those unattended runs,
  then enable scheduling only if every gate passes. During the October 5 Wi-Fi
  test, the native popup showed **Storage operation in progress**, then
  **Storage connection needs attention** after the bounded access failure,
  returned to the in-progress state during the automatic retry, and cleared the
  notice after success. The service journal independently recorded the failed
  access guard, successful retry, clean repeat, and zero-difference comparison.
  Fix the runtime mapper discovered by this test so a failed scheduled service
  remains an error while its timer is active; keep the popup switch tied to the
  persistent timer intent rather than moving off for a transient failure.
  Regression tests and the full locked suite pass (287 tests, 7 explicitly
  ignored live/integration tests), as do all-target/all-feature clippy and
  formatting. The native SharePoint Offline unattended release gate now
  passes. Test Flatpak host-service access when the public Flatpak distribution
  path exists; that package check remains tracked separately under Flatpak
  installation and does not block this native gate.

### Reorder window layout — October 3, 2026

- [x] Make Reorder Connections a compact 480 × 600 window instead of using the
  full 880 × 720 connection-editor size. Present padded rows in the Settings
  list style, keep names flexible, and correct the guidance to say choose a
  position rather than drag. An intermediate 600-pixel width was reduced after
  the user's installed review.
- [x] Fix the installed Reorder layout reported by the user: remove the forced
  96-pixel dropdown width so its arrow stays over the choices menu, and tighten
  number/name spacing. Let the list fill the narrower window's content area so
  its right edge aligns with the content boundary. Focused tests and lint pass.
- [x] Visually check the installed Reorder window with saved connection names.
  The user confirmed the compact layout looks right after the dropdown and
  alignment changes. The functional reorder/persistence check remains in the
  earlier Reorder Connections section.

### Google OAuth client ID for Google Drive — October 3, 2026
- [ ] I created an OAuth client ID and secret, but authentication fails. It
  reported something like a parsing error and an unexpected field.
- [x] Reproduce the current `uutzinger_gdrive` failure with a bounded,
  read-only listing. The saved custom client ID and secret have the expected
  shapes and the remote has a refresh token, but Google rejects token refresh
  with `unauthorized_client`.
- [x] Run `ua_gdrive`, which still uses rclone's shared client, as a read-only
  control. Authentication succeeds.
- [x] Test recovery in a private copy of the rclone configuration. Removing the
  custom client ID and secret while preserving the existing token restores
  read-only access, proving the failed update left a token issued to rclone's
  shared client paired with the new custom client credentials. The live rclone
  configuration was not changed, and the temporary credential-bearing files
  were deleted after the test.
- [x] Restore the live `uutzinger_gdrive` remote to the shared client as a
  temporary recovery step while preserving its existing token. Verify the
  custom fields are empty and a read-only listing of the known `BME` directory
  succeeds. The shared rclone client is being retired during 2026, so this is
  recovery rather than the final configuration.
- [ ] Create or verify a Google Desktop OAuth client with Drive API access and
  the intended account allowed by the consent-screen audience, then complete a
  fresh browser authorization that issues a new token for that client. Do not
  pair new client credentials with the token issued to rclone's shared client.
- [x] Diagnose the editor's `rclone config dump returned an unexpected
  response: expected value` error. The command output redactor removed Google
  client values by inserting an unquoted `[REDACTED]`, corrupting otherwise
  valid JSON; one remote's custom fields could therefore break testing another
  remote. Preserve the surrounding JSON quotes and punctuation while still
  redacting both values, and add a regression test that parses a multi-remote
  redacted dump successfully.
- [x] Reauthorize `uutzinger_gdrive` through a private browser window using the
  personal Google account, replace the prior token, and complete a bounded
  read-only root listing. The user visually confirmed that `BME`, `Personal`,
  and `Pictures` belong to the personal Drive shown in Google Drive; the earlier
  inference that `BME` identified the University account was incorrect. The
  remote currently works with rclone's shared client while private-client setup
  remains pending.
- [x] Fix **Update Google OAuth Client** for an existing remote. Remove
  `--non-interactive` so rclone follows its default replace-token path and opens
  browser authorization; reject a successful-process JSON response when it
  still contains a pending state or option instead of falsely reporting OAuth
  completion. Add a regression test for the unanswered `config_refresh_token`
  prompt, install the corrected build, and restart the panel.
- [x] Complete browser authorization for `uutzinger_gdrive` with its private
  Desktop OAuth client and personal Google account. Verify the saved remote has
  a client ID, client secret, and newly issued token, then complete a bounded
  read-only listing of the personal `Personal` folder without the shared-client
  retirement warning. Confirm the installed applet's **Test Connection** also
  passes afterward.

### Add preload to SharePoint Online. - October 3,4, 2026

- [x] Add a SharePoint Online row to Directory preload settings, with a separate
  default-off policy (30 seconds, depth 2). Existing documents migrate to the
  default-off policy.
- [x] Connect enabled SharePoint Online mounts to the existing bounded,
  directory-only preload walk for the selected mounted library or folder. Wait
  for the rclone mount and its root listing before walking, enforce the saved
  time and depth limits, and keep Offline mirrors unaffected.
- [x] Include SharePoint preload in the shared lifecycle classification so
  unmount, repair, removal, sleep cleanup, and a replacement preload cancel and
  reap the traversal child before the mount service is stopped.
- [x] Add focused regression coverage proving that SharePoint defaults off,
  enabling it selects the bounded directory preload with root readiness, and an
  Offline mirror never enters the preload path. Run formatting, the focused
  tests, the full locked test suite, and a locked build. The focused preload
  tests passed; the full suite passed with 282 tests and seven explicitly
  ignored live/integration tests, and the locked build and formatting checks
  passed.
- [x] After installing the new build, enable SharePoint preload briefly on the
  disposable Online mount and verify bounded completion or timeout, readable
  notice, and clean cancellation on unmount. Restore the policy to off afterward.
  The installed UA Assessment mount used the enabled 30-second/depth-2 policy;
  the user reported fast directory listing, and the bounded preload ended before
  the first clean unmount. The user then remounted and unmounted after about
  three seconds while preload was starting. The unit finished with a successful
  result, with no FUSE mount or traversal child remaining. The saved SharePoint
  policy was restored to disabled with its 30-second/depth-2 limits preserved.
  A final disabled-policy control mounted normally without displaying a preload
  notice or starting a traversal process, then unmounted with a successful unit
  result and no remaining FUSE mount.

### Persistent popup startup state — October 5, 2026

- [x] Define one user-facing startup rule in Requirements: the popup switch is
  the persistent desired-state input. Online mounts left on are remounted at
  login; Online mounts and Offline mirrors switched off remain off. Sleep-only
  cleanup and wake restoration do not change this state.
- [x] Remove the separate **Start at login** control from the Online editor
  while retaining the existing serialized field for backward-compatible
  configuration loading and internal desired-state storage.
- [x] After a successful popup Mount, persist the Online connection as enabled
  at login and enable its managed service. On popup Unmount, persist the off
  intent and disable the managed service so a later login cannot remount it.
  Report partial failures without claiming the persistent state was saved. The
  implementation rolls systemd enablement back if the configuration write
  fails; an off request is persisted before clean detach so a busy mount cannot
  unexpectedly return at login.
- [x] Preserve the existing Offline mirror behavior: Start enables its managed
  timer or monitor across login, Stop disables it, new mirrors remain stopped,
  and Preview plus initial Sync Now remain mandatory before first Start. Sleep
  cleanup still uses temporary stops and does not change persistent intent.
- [ ] Add focused tests for rclone and onedriver Online persistence, Offline
  schedule persistence, new-connection defaults, unit/config reconciliation,
  and sleep/wake not changing the popup intent. Run formatting, locked tests,
  build, clippy, then install and verify the three affected UA connections
  across a controlled logout/login or reboot. Automated persistence/rollback
  tests pass; the full locked suite passes with 285 tests and seven explicitly
  ignored live/integration tests. Locked build, all-target/all-feature clippy,
  formatting, and whitespace checks pass. Installed lifecycle and reboot
  verification remain.

### Release 0.5.1 Activity - October 5, 2026

- [x] Localization audit and migration
  - General Settings and connection editor labels/help
  - Popup status and operation labels
  - Success/error notices
  - Validation messages
  - SFTP setup notices
  - Window titles
  - Added `just localization-check` and included it in `just verify` so new
    directly rendered English labels and hardcoded notice assignments fail the
    release check. Internal logs, command/protocol values, and test fixtures
    remain in Rust.
  - Validation: localization check, locked all-target test suite (290 passed,
    seven explicitly ignored live/integration tests), all-target/all-feature
    Clippy with warnings denied, and formatting check pass.
- [x] Complete the persistent-startup live test
  - One Online mount left on.
  - At least one Online mount left off.
  - SharePoint Offline schedule either deliberately on or off.
  - Confirm the popup switches retain those choices.
  - Confirm only intended connections restart.
  - Confirm temporary failures do not change switch positions.
  - October 5 native reboot/login test passed. Before reboot, **UA Box** was
    saved on, enabled, active, and mounted; **Test_SharePoint Offline UI** had
    its timer enabled and active; **UA OneDrive** and all other managed
    connections were saved off and disabled. After login, the popup retained
    the two intended running switches. Systemd and the mount table independently
    confirmed that UA Box alone restarted as an Online mount, the SharePoint
    timer alone restarted as an Offline schedule, and every other managed
    mount/mirror remained disabled and inactive. The earlier installed Wi-Fi
    loss/recovery test confirms a temporary failure does not change the saved
    SharePoint switch position and that operation resumes after connectivity
    returns.
- [x] Correct release metadata
  - Cargo.toml and Cargo.lock
  - Debian changelog and generated Debian metadata
  - README download commands
  - AppStream release entry
  - Flatpak tag
  - Aligned authoritative source metadata to `0.5.1`/`v0.5.1`, retained the
    historical AppStream entries, added the October 5 release description, and
    updated tag-based screenshot URLs. `cargo metadata --locked` and
    `dpkg-parsechangelog` both report `0.5.1`.
  - Rebuilt the Debian package so generated control and file metadata report
    `0.5.1`; `dpkg-deb` confirms package
    `cosmic-ext-applet-mounter_0.5.1_amd64.deb`, version `0.5.1`, architecture
    `amd64`. The package build passed 290 tests with seven explicitly ignored
    live/integration tests. COSMIC-specific AppStream/Desktop validator findings
    remain the documented non-fatal `COSMIC` category and `binaries` provide
    compatibility notices.
- [x] Final release-candidate validation
  - `cargo fmt --all -- --check`, `cargo check --locked --all-targets`, the
    complete 290-test suite, and Clippy with warnings denied passed. Seven
    explicitly external/live tests remained ignored.
  - `cargo vendor --locked` completed into a disposable directory without the
    earlier duplicate-libcosmic-source error.
  - Desktop/AppStream validation found only the documented COSMIC template
    compatibility notices for the `COSMIC` category and `binaries` provide;
    the AppStream XML is well formed.
  - The localization audit and `git diff --check` passed. A strict repository
    scan found no Google API keys, OAuth client IDs, private-key blocks, or
    assigned access, refresh, or client-secret values.
  - Rebuilt and inspected `cosmic-ext-applet-mounter_0.5.1_amd64.deb`; SHA-256
    is `15ba6b3fb86bb32cab82a1c2bbae4d5106cc1d192a0a98f6528d4da9c4903ca8`.
    The package reports version `0.5.1`, architecture `amd64`, and contains
    both executables plus the desktop entry, AppStream metadata, icon,
    copyright, and changelog.
  - Installed, removed without purge, reinstalled, and finally removed the
    Debian package. The existing `~/.local` applet and COSMIC configuration
    retained identical hashes throughout both package cycles; `/usr/bin` is
    absent afterward and command resolution again selects the custom build.
  - The packaged `/usr/bin` executable ran for the bounded native smoke period
    with all shared libraries resolved. After final package removal, restarting
    `cosmic-panel` successfully relaunched the custom applet. Its subsequent
    normal runtime-state write changed the configuration hash only after the
    package preservation checks had passed.
- [x] Release documentation and publication
  - [x] Create Release Notes 0.5.1.md.
  - [x] Update the completion notes with the final evidence.
  - [x] Generate the .deb (completed above) and SHA256SUMS.
  - [x] Review the complete dirty worktree before committing, including the
        intentional archival removal of the completed Tooltip Review.
  - [x] Commit and push before creating the v0.5.1 tag.
  - [x] Verify screenshot and documentation links against the published tag.
  - Published the annotated `v0.5.1` tag and public GitHub Release with the
    amd64 Debian package and `SHA256SUMS`. The release is neither a draft nor a
    prerelease. Freshly downloaded assets passed `sha256sum -c`, and the public
    tag, README, Google OAuth guide, and four AppStream screenshot URLs each
    returned HTTP 200.

## Flatpak installation. - Started on October 3, 2026

Remaining work to make the applet installable from COSMIC Store:

- [ ] Ask `pop-os/cosmic-flatpak` maintainers for architecture guidance if the applet's
      host-integration requirements need broader access than accepted applets such as
       `dev.cappsy.CosmicExtAppletDrives`. Ask in COSMIC App Developer/Mattermost channel.
- [ ] Open a focused pull request to `pop-os/cosmic-flatpak` containing:
    - `app/io.github.uutzinger.cosmic-ext-applet-mounter/io.github.uutzinger.cosmic-ext-applet-mounter.json`
    - Generated `cargo-sources.json`
    - Links to source repository, MIT license, tagged release, build instructions,
      AppStream metadata, and screenshots.
- [ ] In the pull request, explicitly call out the host-integration architecture and
      justify every non-default Flatpak permission, especially host-command execution,
      systemd unit creation, sleep signals, and shared state.
- [ ] Complete the repository pull-request checklist: disclose AI-generated or AI-assisted
      code in commit messages, understand and be able to explain every submitted change,
      accurately describe and test the change, and certify it under the Developer Certificate of Origin.
- [ ] Address repository CI and maintainer review, updating the source tag/hash when
      a packaging fix requires a new application release.
- [ ] After merge and publication to a configured public remote, install from the
      COSMIC Flatpak remote on a clean profile and run a final mount/mirror/VPN/uninstall smoke test.

Completed preparation work: local Flatpak manifest, cargo-sources generation,
desktop/AppStream metadata, local `just build`/`just build-changed`, Store listing
verification, panel add/remove, and permission rationale in `packaging/flatpak/README.md`.
