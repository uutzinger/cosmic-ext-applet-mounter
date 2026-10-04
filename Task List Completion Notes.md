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
- [ ] Complete broader Google Drive `--fast-list` testing with a larger ordinary
  drive and a shared drive, including API-call/rate-limit behavior.
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
- [ ] Perform cold-cache live tests without reading or modifying cloud file
  contents.

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
- [ ] Live-test the implementation on UA Box and the narrowed
  `ua_engr:Research/Utzinger` SMB connection, then test a separate LAN/home SMB
  connection with its own override.

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
- [ ] Verify the button remains stable in the installed desktop applet.

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
- [ ] Live-check native/Flatpak title/focus, keyboard access, scaling, empty/long
  lists, editor-save propagation and closing either window during runtime work.

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
- [ ] A: Live-check native and Flatpak NetworkManager/Cisco password/MFA followed
  by SMB mount; verify concurrent requests and credentials rejected by client.
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
- [ ] B: Exercise live logind signal/inhibitor lifecycle, busy mounts, failed
  service stops, denied bus permissions, canceled sleep, connection edits during
  wake and native/Flatpak lid/menu suspend over repeated sleep/wake cycles.
- [ ] B: Capture sleep journal/service evidence to isolate the reported OneDrive
  hang; source changes do not establish its live root cause.
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
- [ ] Validate metadata with `desktop-file-validate` and
  `appstreamcli validate --pedantic`.

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
- [ ] After merge and publication, install from the public COSMIC remote on a
  clean user profile and repeat a minimal mount, mirror, VPN, and uninstall
  smoke test.

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
- [ ] Extend compatible legacy import to identify SFTP remotes correctly.
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
- [ ] Localize SFTP labels alongside the existing connection-editor strings.
- [x] Make Create/Update SFTP Remote use the blue/theme-accent suggested style
  when adding a new remote. Unchanged existing remotes retain standard styling.
- [x] Add delayed hover help to Password, SSH Key, and SSH Agent choices.

### Runtime and safety

- [x] Preserve blank, relative, and absolute SFTP directories in connection
  validation, access-test targets, mount plans, and bisync plans.
- [ ] Verify SFTP directory semantics through legacy import.
- [x] Require a known-hosts path in setup and before Test/Save access checks;
  leave unknown/changed-key handling to rclone without adding or replacing keys.
- [ ] Live-verify missing/unreadable files and unknown/changed host-key errors.
- [ ] Add bounded Test Connection and mount preflight with sanitized host,
  authentication, host-key, directory, and permission failure messages.
- [ ] Integrate generated host services, including host rclone/key/known-hosts
  paths and SSH-agent availability for native and Flatpak execution.
- [x] Wire SFTP into the shared rclone mount/mirror provider branches and
  generated plans. SFTP preload defaults to disabled; retain bounded rclone
  defaults rather than SMB-specific timeout settings.
- [ ] Live-verify Online status/cache health, safe unmount/repair, network/VPN
  recovery, and sleep/wake handling for SFTP.
- [ ] Integrate Offline mirror preview/confirmation, scheduling, metered policy,
  conflict preservation, recovery retention, and interrupted-sync recovery;
  support SFTP-only servers without requiring shell/hash commands.

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
- [ ] Visually verify row placement and per-connection override persistence.
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
- [ ] Repeat with a longer preload limit if full depth-2 completion matters;
  compare the first listing of directories reached later in the traversal.
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
  September 30; applet restart and live validation remain open.
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
  pass. VPS exclusions and ordinary browsing remain separate open checks.
- [ ] Install/restart the applet with this fix and visually confirm the progress
  and successful completion notices during an SFTP unmount. Test the same
  cancellation behavior for other providers when suitable shares are available.

### Verification and delivery

- [ ] Visually verify the new Create/Update action styling and all three
  authentication-choice tooltips in the installed applet.
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
- [ ] Visually verify blue Save and changed-server Update behavior in Add and
  Modify, including a failed update and a successful credential update.
- [x] User-confirmed SFTP setup and Online mount workflow on the VPS after the
  Create/Update, Test Connection, Save, and mount instructions. Authentication
  method and package type were not recorded; individual write/delete/unmount
  results were not separately reported. See the live follow-up in completion
  notes. Broader acceptance below remains pending.
- [x] Run focused automated checks for configuration compatibility, discovery,
  validation, setup/update commands, secret handling, authentication changes,
  remote-path preservation, and generated mount/mirror targets. Runtime and
  data-integrity acceptance remain separate tasks below.
- [ ] Run isolated SFTP integration checks for password/key/agent authentication,
  host-key validation, permission errors, timeouts, and servers without a shell.
- [ ] Verify native and Flatpak UI placement beside SMB, similar form layout,
  conditional fields, Add/Modify actions, keyboard navigation, and no clipping.
- [ ] Verify generated-service authentication independently of terminal tests,
  including unavailable/locked SSH-agent behavior and startup at login.
- [ ] With disposable server data, verify read/write/mount/unmount, pending-write
  protection, network/VPN recovery, sleep/wake, mirror preview/initial/repeated
  sync, conflict preservation, deletion recovery, and interruption recovery.
- [x] Run all-target compilation, formatting, all-target/all-feature Clippy
  with warnings denied, all-target tests, and diff validation. All 208 tests
  pass, including 14 new SFTP checks and the existing-provider regressions.
- [x] Check installed rclone 1.75.0 setup/update behavior using only a disposable
  configuration and dummy credentials; verify obscuring and empty-secret
  handling without contacting a server.
- [x] Record source/test evidence separately from pending installed-service
  and live-server acceptance in `Task List Completion Notes.md`.
- [ ] Update README, dependency/setup guidance, and release metadata to describe
  delivered SFTP support and any verified limitations before release.

Editor implementation evidence is recorded under “SFTP Provider and Connection
Editor — September 29, 2026” in `Task List Completion Notes.md`.


## OneDrive Mount Failure — Deferred Investigation

User report: OneDrive shares had mounting problems. The end-of-list reminder
suggested clearing the generated systemd user service's failed state before
starting it again. Keep this as an investigation note for a later discussion;
the cause and appropriate applet behavior have not yet been established.

Commands retained from the note for reference (the abbreviated service ID is a
placeholder, not a runnable connection identifier):

```sh
systemctl --user reset-failed cosmic-mounter-990cc48f-...-c545ad3d3f9d.service
systemctl --user start cosmic-mounter-990cc48f-...-c545ad3d3f9d.service
```

- [ ] Identify the affected OneDrive connection, access mode, engine, and exact
  generated service; capture its status and journal around a failed mount.
- [ ] Determine whether the failed state, a start-rate limit, authentication,
  connectivity, or a lingering mount prevented recovery. Treat the original
  claim that every failed service requires `reset-failed` as a hypothesis to
  check, not a confirmed rule.
- [ ] Discuss the expected Mount/Retry/Repair behavior and whether targeted
  failed-state recovery is needed; then define any implementation and regression
  tests based on the observed failure.

No runtime fix or successful recovery is claimed by this note.


### Installed SFTP preload cancellation accepted — September 30, 2026

✅ User reported that the sequenced mount cancellation worked as expected after
installation of the 60-second guard. This confirms the installed SFTP unmount
workflow passed the user's live retest, in addition to the earlier production
path test with an observed active `find` traversal. Updated the task-list
checkmark. Exact notice text and elapsed time were not separately recorded.

⏳ The shared cancellation path still needs live checks for OneDrive, Box, and
SMB when suitable shares are available. Google Drive's separate RC refresh
stop is also not covered by this SFTP result.


### SFTP checklist audit and legacy import — September 30, 2026

✅ Legacy import now reads supported provider types from the host rclone
configuration. An arbitrary-name SFTP remote imports as SFTP; missing or
unsupported remotes are not guessed from their names. A legacy unit naming a
different rclone config file is rejected because the managed replacement would
otherwise use a different configuration. Import previews and generated plans
retain blank, relative, and absolute SFTP directory targets.

✅ Focused import tests and the full 224-test suite passed (137 library, 87
binary tests), including the private D-Bus test with approved host access.
Formatting and diff checks passed. The first full-suite attempt reached the
known sandbox D-Bus socket restriction; it passed on the host rerun.

✅ The optional longer depth-2 benchmark was superseded by the approved
partial-warming goal and earlier fresh-mount comparison. Remaining unchecked
items are actual localization, diagnostics, host-service, mirror, visual, and
live integration work; the other-provider cancellation check is a shared
regression task. No new installed-service or live SFTP import test was run.


### SFTP access diagnostics — September 30, 2026

✅ Test Connection and Online mount preflight now route SFTP rclone listing
errors through one provider-specific classifier. It distinguishes host reachability,
authentication, host-key verification, missing/unreadable key and known-hosts
files, SSH-agent access, missing directories, and directory permissions.
Messages provide corrective guidance without repeating rclone stderr, stdout,
command text, paths from diagnostics, or credential-like values. Other rclone
providers retain their previous messages.

✅ Automated cases cover each failure category, unknown-error fallback,
timeout, and spawn-error redaction. The full 225-test suite passed (137 library,
88 binary tests), as did formatting, all-target/all-feature Clippy with warnings
denied, and diff whitespace checks. The private D-Bus test used approved host
access.

✅ A read-only probe of an intentionally absent path on the configured VPS
returned rclone's `directory not found` diagnostic; the classifier covers that
wording. The first attempt was blocked by the sandbox's network restriction;
the host-access rerun reached the VPS. No remote files were changed.

⏳ Installed-app notice wording and the remaining live failure categories have
not been verified. Live acceptance checkboxes remain open.

### SFTP native and Flatpak editor visual review — September 30, 2026

✅ The installed native binary exactly matched the current release build, which
was built after the editor source changes. That release binary was packaged with
the local Flatpak GUI prototype and first launched using `flatpak-builder --run`
with the Wayland socket. The prototype was then temporarily installed as a user
Flatpak and launched with `flatpak run` in Add and Modify modes. Both installed
windows opened with the expected SFTP layout and the saved connection loaded in
Modify. The test Flatpak was uninstalled afterward. Native and Flatpak editor
processes were run one at a time.

✅ The Flatpak Add editor showed SFTP immediately after SMB, blue Save and
Create/Update actions, the expected host/port/user/authentication/known-hosts/
remote-directory fields, and a scrollable layout without visible clipping.
Turning off global SFTP preload exposed enabled, 60-second, depth-2 draft
controls. The user confirmed readable, distinct hover help for Password, SSH
Key, and SSH Agent. An initial keyboard check happened while VS Code had focus;
after focusing an editor field, Tab and Space/Enter activation worked.

✅ Flatpak Modify loaded the saved VPS SFTP connection, kept provider and mode
locked, left secrets blank, and displayed the saved per-connection preload
override: global off, preload off, 30 seconds, depth 2. The user confirmed
that an unsaved host edit turned Update SFTP Remote blue and that SSH Key and
SSH Agent showed their conditional controls without clipping or keyboard
trouble. The native Modify view matched the Flatpak layout and saved preload
values; the user confirmed its tooltips and keyboard controls worked. Native
Add showed blue Save and Create/Update actions and the SFTP preload override
controls. No draft was saved or remote updated during this review.

⏳ Failed-update styling and a successful credential-update outcome still need
disposable credentials and a separate live check. This visual review does not
verify authentication or mount behavior in the Flatpak GUI prototype.

### SFTP disposable live connection probes — September 30, 2026

✅ Used a temporary localhost `rclone serve sftp` endpoint, generated host and
client keys, a known-hosts file, and a separate disposable SSH agent under
`/tmp`. The existing VPS connection, saved applet settings, and user SSH agent
were not changed. A second temporary endpoint tested password authentication.
Correct password, private-key, and unlocked-agent authentication succeeded;
wrong password, unauthorized key, and missing agent failed as expected. A
client configured with `shell_type = none` connected, though the test server
itself was not proved to lack a shell.

✅ Direct rclone probes produced real errors for missing and unreadable private
keys and known-hosts files, unknown and changed host keys, failed
authentication, missing agent, absent directory, and a refused localhost
connection. The errors match the applet classifier's automated categories.
The disposable server returned `directory not found` for both a denied and an
absent directory, so a distinct permission error was not observed. Installed
applet notice wording and deliberate timeout behavior were not live-tested.

✅ A transient host user service read the temporary rclone configuration,
private key, and known-hosts file. Another transient user service authenticated
through the disposable agent socket when that socket was supplied explicitly.
The user service manager already had an `SSH_AUTH_SOCK` value, but the test key
was not loaded into that existing agent. These probes do not prove the native
or Flatpak applet-generated service inherits and can use the user's agent.

✅ A disposable rclone FUSE mount under a transient user service became active,
listed and read the test file, and populated the VFS cache. Stopping only the
localhost server left the mount service active but a read of that cached file
returned I/O error; the direct SFTP probe reported connection refused. After
restarting the server, the same mount read the file successfully. Restarting
the transient mount service remounted and read it successfully. `fusermount3 -u`
cleanly removed the mount; systemd collected the transient unit. Both local
servers and the disposable agent were stopped afterward.

⏳ Applet Online status and cache indicators, its Repair action, generated-unit
agent inheritance, installed native/Flatpak notices, VPN transitions, and a
real sleep/wake cycle remain open. The localhost interruption is an isolated
server-loss test, not proof of system network or VPN recovery. The temporary
test directory, keys, agent socket, and cache were deleted after unmount.

### SFTP Offline mirror disposable acceptance — September 30, 2026

✅ Used isolated rclone configuration, keys, localhost servers, mirror, work,
and recovery directories under `/tmp`; no saved applet connection or user
remote was changed. The test repeated the SFTP bisync plan's workdir, separate
remote/local backup directories, `--resilient --recover`, conflict preservation,
`size,modtime` comparison, delete bound, and filter-file arguments. Initial
`--resync --dry-run` left both sides untouched. Confirmed initial sync copied
remote-only and local-only files in both directions. A later dry preview also
left new files untouched; normal bisync then propagated them both ways.

✅ Simultaneous edits to the same file produced two distinct conflict copies on
both sides (`conflict1` and `conflict2`). Deleting a remote file propagated its
deletion to the local mirror, while its prior local copy appeared in the local
recovery directory. This verifies a recovery copy was made, not the configured
30-day retention period.

✅ A deliberately interrupted throttled 8 MiB upload left a remote partial
file and a bisync lock. A normal retry with `--recover` refused the still-valid
lock. After verifying the recorded process had exited and removing only the
disposable stale lock, `--recover` completed without `--resync`; remote and
local SHA-256 hashes matched. The applet has no verified automatic stale-lock
path, so unattended recovery remains open.

✅ A separate Paramiko SFTP-only endpoint explicitly rejected shell and exec
requests. With rclone `shell_type = none` and `disable_hashcheck = true`, the
same initial preview and confirmed two-way sync completed. A transient user
timer started one bisync service and transferred a new local file to that
server; its service exited successfully. This checks the SFTP-only protocol
path and transient scheduling, not the installed applet-generated timer.

⏳ Installed applet Preview/Sync Now and marker behavior, metered scheduling,
recovery retention aging, and safe stale-lock repair are unverified. The first
repeat attempt edited the rclone server's backing directory directly and its
directory cache did not notice a new file; the valid rerun made all remote edits
through SFTP. Both disposable servers were stopped, the transient timer was
collected, and the generated keys and test data were deleted afterward.

### Bisync stale-lock recovery — September 30, 2026

✅ Generated rclone bisync plans for Google Drive, Box, SMB, and SFTP now pass
`--max-lock 2m` alongside `--resilient --recover`. Rclone renews locks held by
active runs and lets locks left by interrupted runs expire. Dependency checks
require `--max-lock`; the applet reports a concise, path-free message when a
lock blocks sync. No applet code deletes a lock or starts a routine resync.
Rclone's documentation states that a lock must have been created with
`--max-lock` to gain an expiry, so locks from older generated units remain a
separate manual/reviewed recovery case.

✅ A disposable local bisync test confirmed a second run was blocked while the
first was active and immediately after its hard stop. After the real two-minute
expiry, rclone attempted recovery without clearing the lock manually. An
especially early interruption had no usable prior listings and stopped with a
critical error instead of resyncing; that state needs reviewed recovery.
In a separate established mirror with a completed repeat run, interrupting an
8 MiB transfer and simulating expiry of only the disposable lock allowed
`--recover` to complete without `--resync`. The recovered file had identical
SHA-256 hashes on both sides. The test also confirmed the next normal bisync
run succeeds and removes an interrupted partial file into recovery storage.

✅ The focused plan and lock-message tests passed. The full all-target suite
passed (137 library and 89 binary tests) with host socket access, as did
all-target/all-feature Clippy with warnings denied and the formatting check.
The initial sandbox run of the full suite failed only at the private D-Bus
socket test; the host rerun passed. Live installed-applet and generated-timer
acceptance remain open. Disposable test files and scripts were deleted.

### SFTP Offline mirror remaining-check audit — September 30, 2026

⏳ Installed applet Preview, initial Sync Now, Start/Stop, status, and generated
service/timer behavior still need a disposable end-to-end check after a build
is installed. The earlier command-line and transient-timer probes did not test
these applet controls.

⏳ Metered scheduling needs a code fix before live acceptance. The applet checks
metered state when enabling the background timer, but each timer firing starts
the generated `rclone bisync` service directly. A later change to a metered
network therefore bypasses that start-time guard.

⏳ Thirty-day recovery retention also needs a code fix. `recovery_record` and
`expired_recovery_records` model the deadline and have unit tests, but no
runtime caller prunes actual files in the rclone bisync backup directories.
The disposable mirror test proved creation of a recovery copy, not its aging
or cleanup. Do not mark retention accepted until a bounded, safe cleanup path
and aged-file tests exist.

### Metered scheduled runs and recovery retention — September 30, 2026

✅ Generated rclone mirror services now include an `ExecCondition` when automatic
sync on metered networks is disabled. It checks NetworkManager at every timer
firing, so a network that becomes metered after Start skips the bisync command.
The condition is omitted when the connection permits metered sync. A
runtime-only disposable user service with a fake `nmcli` response skipped its
command for `yes` and ran it for `no`; the real network was not changed. The
temporary service link and files were removed afterward.

✅ A managed host-side bisync runner now serves scheduled and manual Preview/
Sync Now operations. New recovery copies go into per-day local and remote
folders with an applet ownership marker, and a unique run suffix prevents
repeated backups of the same path from overwriting an earlier copy. Cleanup
after a successful sync removes only marked date folders older than the
retention boundary. It uses a calendar-day safety margin so files stay at
least 30 days, and leaves preexisting unmarked recovery data untouched. The
script is stored privately in the connection's work directory, with an
ownership check before replacement.

✅ An isolated rclone alias/local integration check passed dry preview,
confirmed two-way sync, deletion into a dated recovery folder, removal of
marked old folders on both sides, and preservation of an unmarked old folder.
The full all-target test suite passed with host socket access (one separate
external-rclone integration test is ignored by default), as did Clippy with
warnings denied. The first sandbox suite run hit the known private D-Bus
socket restriction; its host rerun passed.

⏳ A real SFTP-only recovery/cleanup run and installed native/Flatpak service
and applet-control checks remain. Units generated before this change still
contain their previous direct rclone command until the mirror is saved again
or Start regenerates its service. Legacy unmarked recovery files are preserved
for a separately reviewed cleanup decision.

### SFTP credential-update failures and timeout wording — October 1, 2026

✅ SFTP setup now distinguishes a five-second rclone configuration-read timeout
from a 30-second remote-save timeout. Both notices say the remote should be
inspected before retry because a timed-out update may have changed settings.
The messages do not include command output or credentials. A deliberately
unresponsive disposable localhost SSH listener caused rclone to report an
`i/o timeout` during handshake; the SFTP access classifier now reports a
connection timeout rather than a generic unreachable-server error.

✅ A failed SFTP remote update keeps the Update action highlighted. If a password
or key passphrase was submitted, the cleared credential must be re-entered
before retrying, preventing an apparently successful retry from silently
preserving the old stored secret. A successful update clears the highlight.
Focused state and error-message tests, the all-target suite, formatting,
Clippy with warnings denied, and diff validation passed.

✅ A disposable localhost `rclone serve sftp` endpoint with a temporary host
key, known-hosts file, data tree, and rclone config accepted the initial
password, rejected an updated wrong password, and worked again after the
correct password was applied. The test left saved applet remotes untouched.

⏳ The installed native and Flatpak editor notices, an applet-driven failed
then successful credential rotation, and distinct permission-denied wording
still need live acceptance with disposable credentials. No existing saved
remote was changed for this check.

### SFTP host-service key and agent access — October 1, 2026

✅ An isolated probe used the applet's SFTP mount-plan builder and systemd unit
renderer, adding only a temporary rclone config path. Host user services
mounted and read disposable localhost SFTP data first with a private key and
known-hosts file, then through an unlocked private SSH agent. The host service
used `/usr/bin/rclone`, and `systemd-analyze verify` accepted both units.

✅ The locally built Flatpak GUI prototype ran `flatpak-spawn --host` to read the
same remote with a private key and with an SSH agent, and inspected the
generated user unit through host `systemctl`. Flatpak host commands used the
desktop session agent rather than the test agent set only in the user manager;
a disposable key was loaded into that session agent for the probe and removed
afterward. The original user-manager `SSH_AUTH_SOCK` was restored, no disposable
unit remained active, and temporary keys/configuration were deleted.

⏳ This verifies the native generated-unit path and the Flatpak prototype's
host-command bridge. It does not claim that a connection was saved or mounted
through the installed Flatpak GUI; that remains in Online-mount acceptance.

### SFTP Online lifecycle probe — October 1, 2026

✅ A disposable localhost SFTP connection was mounted through the applet's
generated user service. Its file was readable, the service and controller
reported Mounted, and the rclone VFS endpoint reported a healthy cache with no
pending writes. The applet's unmount and repair operations each detached it
cleanly. Its sleep-cleanup function stopped the active service; a manual
restart mounted it again and restored file access. The probe removed its unit,
temporary credentials, server, mount, and sleep drop-in afterward.

✅ The live probe found that a Flatpak sandbox's mount table omits the host FUSE
mount while the host bridge can see it. Applet mount-table reads now use that
bridge in Flatpak. Repair now avoids a redundant lazy unmount after service
stop has already detached the mount. Command-output redaction preserves line
breaks, allowing sleep cleanup to read systemd properties. The applet now
passes polled NetworkManager readiness to its controller and correctly treats
`disconnected` as offline. Focused state tests, the all-target test suite,
formatting, Clippy with warnings denied, and the disposable live probe passed.

⏳ The earlier server-loss probe showed recovery after the localhost SFTP
server returned, but a cached file produced I/O error during the outage;
acceptable offline-cache behavior still needs a decision. Real network/VPN
transitions, an actual suspend/wake cycle with automatic remount, and installed
native/Flatpak applet notices were not exercised by this disposable probe.

### SFTP Offline mirror generated-unit acceptance — October 1, 2026

✅ An isolated SFTP-only Paramiko server explicitly rejected SSH shell and exec
requests. With client shell use and hash checks disabled, the applet's Offline
mirror operation functions required Preview before initial Sync Now, left data
untouched during previews, recorded initial-sync markers, and transferred new
files on repeated syncs. Deleting a remote file put a copy in dated local
recovery. A later sync pruned old local and remote folders only when their
ownership marker matched this connection; old unmarked folders survived.

✅ The applet installed a unique generated user service and timer using its
managed bisync script. A test-only timer drop-in shortened the 15-minute
interval so the timer could fire during this probe; it transferred a new file.
With a scoped fake NetworkManager response of `yes`, systemd's journal
explicitly recorded `Skipped due to 'exec-condition'`, and the next file stayed
local. The Flatpak GUI prototype's host bridge read both generated units. The
test stopped and removed its units and drop-ins, and used only disposable
credentials and data.

⏳ This tests the applet's operation functions and generated host units, not
clicks through an installed native or Flatpak editor. Installed Preview, Sync
Now, Start/Stop, status, and applet-driven interruption recovery remain open.
The earlier command-level SFTP test covered conflicts and interrupted sync;
those results do not by themselves establish installed-applet behavior.

### Remaining-test checklist consolidation — October 1, 2026

The next native SFTP release now has one fixed test gate at the end of
`Task List.md`: installed editor/error feedback, Online lifecycle, Offline
mirror controls, and a final release-candidate check. Duplicate open SFTP test
entries were removed from the historical implementation sections. Public
COSMIC Flatpak tests wait for that distribution path, while broader provider,
VPN, sleep fault-injection, and UI checks are listed as deferred validation.
Future SharePoint implementation and publication tasks remain in their own
sections and do not expand the native SFTP test gate.

### Release-test scope correction — October 1, 2026

The first consolidated checklist overstated what remained by repeating basic
SFTP authentication, installed Add/Modify UI, mount/unmount, and disposable
Online/Offline workflows already recorded as checked. The current checklist
limits the next native release to one failed credential-update flow, Online
pending-write and normal wake behavior, the installed Offline control path,
and final package/regression checks. Exhaustive installed error variants,
VPN-bound SFTP, and public Flatpak acceptance are deferred. Prior completed
evidence remains valid unless a later fix changes that behavior.

### Installed SFTP credential-update check — October 1, 2026

✅ A disposable localhost SFTP server and isolated rclone config exposed a
save-error bug: `rclone config update` could report "Failed to save config" on
stderr while returning exit status 0 and an empty JSON Error. The editor now
recognizes that failure and gives redacted permission/retry guidance. A focused
regression test covers the misleading response and confirms private values do
not enter the notice. The full automated suite, formatting, Clippy, and the
installed native build passed after the fix.

✅ With the isolated config unwritable, the installed Add editor timed out on
the failed password update, cleared the password for re-entry, and kept the
Create/Update action blue. The separate zero-exit save-error case was verified
by the focused regression test. After restoring write access, the corrected
password update succeeded. The remote listed `probe.txt`, and the editor's
Test Connection passed using a disposable name and mountpoint. No connection
was saved to the user's configuration.

### SFTP Online sleep safety and wake restoration — October 1, 2026

✅ The existing disposable localhost SFTP lifecycle probe was extended to queue
an actual VFS upload. While it was pending, the app's sleep cleanup reported
"pending uploads" and kept the mount active. After the upload reached the
server, cleanup stopped the service and detached the mount. With isolated
settings enabling both sleep cleanup and restore, the wake restoration path
restarted the managed service, remounted the share, and read its test file.
The focused live test passed; its temporary service, drop-ins, credentials,
cache, and server were removed. This exercised the live cleanup/restore code
path without suspending the workstation or sending a logind sleep signal;
repeated lid/menu suspend and inhibitor fault cases remain deferred.

⚠️ Release limitation: in the earlier server-loss probe, reading a file that
had been populated into the SFTP VFS cache returned I/O error while the server
was unavailable. The same mount read it again after the server returned.
Offline reads during an SFTP outage are therefore not guaranteed; no cache
recovery claim is made. Real VPN transitions remain deferred.

### Installed native SFTP Offline controls — October 1, 2026

✅ The installed applet displayed a disposable localhost SFTP Offline Mirror.
Its Modify editor completed Preview and asked for Sync Now; the local mirror
remained empty. Confirmed initial Sync Now copied the remote test file to the
local mirror and wrote the initial-sync marker. The applet's Start switch
enabled the generated background timer, and its Stop switch disabled it;
systemd confirmed both states. Offline Mirror's blue switch indicates an
active sync schedule, not a mounted filesystem. The applet's confirmed Remove
action removed the connection and generated units. The temporary rclone
remote, server, mirror, recovery data, and state were removed afterward; all
pre-existing rclone remote settings matched the pre-test backup.

✅ Preview initially overcounted one proposed download as eight downloads and
two skips because its parser treated rclone progress and dry-run housekeeping
as file actions. The parser now counts directional dry-run copy lines and
ignores filters-file hash housekeeping. A focused regression test, the full
automated suite, formatting, and Clippy passed. In the reinstalled native
editor, an isolated one-file Preview then reported one download, zero skips,
and a four-byte estimate without copying the file. The corrected installed
binary matched the release build checksum.

### SFTP 0.4.7 final release-candidate check — October 1, 2026

✅ Bumped the source, lockfile, Debian changelog, AppStream release entry,
Flatpak manifest tag, and README package example from 0.4.6 to 0.4.7 before
the final build. Debian and AppStream provider descriptions now include SFTP.
The AppStream screenshot URLs still point to the existing published v0.4.6
assets; a v0.4.7 tag has not been published.

✅ `just deb` built `cosmic-ext-applet-mounter_0.4.7_amd64.deb` and passed its
locked release build and automated tests (141 library tests, one ignored; 97
binary tests, two ignored). Formatting, all-target/all-feature Clippy with
warnings denied, and `git diff --check` passed. The disposable live SFTP
Online mount, unmount, repair, pending-upload sleep cleanup, and wake-restore
smoke test passed on the 0.4.7 source build.

✅ The Debian package reports version 0.4.7, contains the expected binary,
helper, desktop entry, AppStream metadata, and icon, and passed an
`apt-get -s install` dependency simulation. Extracted desktop, metadata, and icon files
matched the source. `just install-user` installed a binary whose SHA-256
matched `target/release/cosmic-ext-applet-mounter`. Package SHA-256:
`6ff7c3e47704612aad174e0ef3dffe8db549026bce7c670a93fb796095671d0f`.
No system-wide Debian installation or publication was performed; the running
panel applet may need a restart to load the new user-installed binary.

⚠️ `just metadata-check` completed with the existing COSMIC-template validator
exceptions: `desktop-file-validate` does not recognize the `COSMIC` category,
and pedantic AppStream validation reports `category-invalid COSMIC` plus an
informational `unknown-provides-item-type binaries`. These findings predate
this version bump and are documented in the packaging recipe.

### Teams (SharePoint library) planning — October 1, 2026

✅ Expanded the earlier SharePoint placeholder into one chronological Teams
implementation checklist. The applet label is **Teams**; Online mount uses a
verified rclone `onedrive` document-library remote first, while Offline mirror
is a separate `abraunegg/onedrive` milestone. The example
`ENGR-BME-Assessment/Shared%20Documents` is a site/library URL, not a local
mountpoint. Authenticated drive ID and library URL verification is a required
read-only feasibility probe before the editor can enable a mount.

✅ Official [Microsoft Teams/SharePoint documentation](https://learn.microsoft.com/en-us/sharepoint/teams-connected-sites)
distinguishes standard-channel folders from private/shared-channel sites.
[rclone](https://rclone.org/onedrive/) documents its `onedrive` backend's
document-library drive type and ID. [abraunegg/onedrive](https://github.com/abraunegg/onedrive/blob/master/docs/sharepoint-libraries.md)
documents a separate SharePoint `drive_id`/configuration and cautions about
overwrite reports with background services, indexing, and replace-on-save
editors. The Offline milestone therefore gates unattended scheduling on
disposable-data safety tests rather than assuming personal OneDrive results
apply. Installed rclone is 1.75.0 and `onedrive` is 2.5.11;
`rclone backend help onedrive` reports no backend metadata commands. No university site access or
live SharePoint test was performed during planning.

### Teams example-site feasibility probe — October 1, 2026

✅ The current rclone configuration has seven remotes, none using the
`onedrive` backend. A read-only unauthenticated HEAD request to the example
`ENGR-BME-Assessment/Shared%20Documents` URL returned 403 with a SharePoint
forms-sign-in-required header. This shows authentication is needed, but proves
neither the exact library ID nor account permissions. The interactive rclone
session was closed before the user began setting up a dedicated work-account
remote. At that stage, authenticated drive ID, `drive_type`, and library URL
checks remained open; no mount, remote change, or file operation was performed.

✅ After the user created `ua_teams_engr_bme_assessment` with work-account
browser authorization, a read-only rclone root listing succeeded. The remote
configuration reports `type=onedrive` and `drive_type=documentLibrary`.
Microsoft Graph's authenticated drive response returned the same drive ID,
`driveType=documentLibrary`, library name `Documents`, and the exact example
`https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents`
web URL. The root contained three directories and six files. The access token
was used transiently in memory for the Graph lookup, never printed or stored by
the probe. No mount, upload, delete, or other library mutation was performed.

## Main Popup Header Cleanup

**October 3, 2026 — implemented and tested.**

The user observed that the three-line popup header (active connection count,
notification state, VPN status) was distracting. The connection count is already
visible through the row toggle colors; notifications state never changes because
there is no UI toggle for it; and VPN status does not need to be visible at all
times.

Changes:
- `src/app.rs`: removed `notification_status` and `vpn_summary()` from the popup
  header. The header now shows only the title row and Settings gear when the
  aggregate state is healthy and 5 seconds have passed since the popup opened.
- Added `popup_opened_at: Option<Instant>` to drive the auto-hide timer; it
  resets when the popup closes.
- Added `POPUP_HEALTHY_AGGREGATE_HIDE_AFTER = Duration::from_secs(5)`.
- Removed the now-unused `vpn_status_pending` field, `vpn_summary()` helper,
  and `connection_vpn_label()` helper.
- Removed unused `notifications-enabled` / `notifications-disabled` strings from
  `i18n/en/cosmic_ext_applet_mounter.ftl`.
- Cleaned up the `NoticeTick` subscription so it runs whenever the popup is
  open, which is needed for the timed hide, and collapsed its nested `if`
  block per Clippy.

Verification:
- `cargo check` passed.
- `cargo clippy -- -D warnings` passed.
- `cargo fmt --check` passed.
- `cargo test` passed: 175 lib tests, 101 bin tests, ignored live tests.
- No manual live UI verification performed yet.

## Reorder Connections

**October 3, 2026 — implemented and tested.**

The user wanted to change the display order of connections in the main applet
popup. Drag-and-drop was discussed, but a dropdown-based reorder window was
chosen because the connection editor does not show the full list.

Changes:
- `src/app.rs`:
  - Added `AppLaunchMode::ReorderConnections` and
    `WindowMode::ReorderConnections`.
  - Added `Message::OpenReorderConnections` and
    `Message::ReorderConnection(ConnectionId, usize)`.
  - Wired the `--reorder-connections` CLI argument through
    `launch_mode_from_args()` and `launch_settings_process()`.
  - Added a "Reorder" button in `view_general_settings()` next to Add
    Connection and Refresh.
  - Implemented `view_reorder_connections()` as a settings window that lists
    every saved connection name with a position dropdown showing `1..N`.
  - Implemented `reorder_connection()` to remove the selected connection and
    re-insert it at the chosen 1-based index, persist via
    `config.update_validated_with()`, and notify the runtime owner.
- `i18n/en/cosmic_ext_applet_mounter.ftl`: added `reorder-connections`,
  `reorder-connections-title`, and `reorder-connections-empty`.
- `src/app.rs` tests: added
  `reorder_connection_moves_item_to_selected_position` and
  `reorder_connection_ignores_invalid_and_unchanged_positions`.

Verification:
- `cargo check` passed.
- `cargo clippy -- -D warnings` passed.
- `cargo fmt --check` passed.
- `cargo test` passed: 175 lib tests, 103 bin tests, ignored live tests.
- No manual live UI verification performed yet.

## Reorder Connections — Recovery from Test Data Loss

**October 3, 2026 — recovery completed and root cause fixed.**

While verifying the new Reorder Connections feature, the unit tests for
`reorder_connection` called `AppConfigStorage::runtime()`, which pointed at the
live applet configuration path. The tests wrote a fixture document containing
three test connections to
`~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document`,
overwriting the user's saved connections and settings.

Immediate recovery:
- Removed the three test systemd service files created by the fixture data
  (`2657cdf6…`, `cc5714ac…`, `fda7f8d6…`) and reloaded the systemd user daemon.
- Wrote `examples/recover_config_from_units.rs` to rebuild the saved connection
  list from the surviving applet-managed systemd units in
  `~/.config/systemd/user`. It handles:
  - rclone Online mounts (Box, Google Drive, SMB, SFTP, SharePoint/Teams),
  - onedriver Online mounts,
  - `onedrive` Offline mirrors,
  - SharePoint Offline mirrors via the generated `managed-bisync.sh` script.
- Ran the recovery example, which backed up the corrupted config and restored
  10 connections with their original IDs, names, targets, remotes, cache
  directories, and Teams identities.
- Deleted the leftover corrupted backup files after confirming the recovered
  config loads and validates.

Root-cause fix:
- Split `reorder_connection` into a runtime wrapper that opens live storage and
  a testable `reorder_connection_with_storage` helper that accepts any
  `AppConfigStorage`.
- Rewrote the two reorder tests to use a temporary `HostVisibleConfigStorage`
  path, so they no longer touch the user's real configuration.

Verification after the fix:
- `cargo check` passed.
- `cargo clippy -- -D warnings` passed (including the new recovery example).
- `cargo fmt --check` passed.
- `cargo test` passed: 175 lib tests, 103 bin tests, ignored live tests.

Remaining open item: live UI verification of the Reorder window from General
Settings.

What was lost and not recovered:
- Any global settings changes the user had made (sleep toggles, preload policy,
  VPN profiles) were overwritten by the fixture defaults. These are not stored
  in the systemd units and had to be reset to defaults. The user may need to
  re-enable or adjust sleep/preload/VPN preferences.
- The recovered connection names retain the "Cloud Mounter: " prefix from the
  unit descriptions; the user can rename them in the connection editor if
  desired.

## Reorder recovery follow-up audit

**October 3, 2026 — current configuration protected and compared.**

The live v2 document was copied to a private 0600 backup at
`~/.local/state/cosmic-ext-applet-mounter/config-backups/20261003-d1Hyo4/document.original`.
The copy matched the original checksum. The recovery example was changed so it
never writes the live document: it reads managed units, can produce only new
unit-only or baseline-merged candidate files, validates both, and compares
matching IDs. It strips `Cloud Mounter:` from recovered online names, uses
unique provisional OneDrive labels, recognizes SharePoint only from its managed
guard, and checks service enablement rather than treating `WantedBy` as enabled.
Focused tests prove candidate writes cannot replace an existing/live file.
The follow-up removed an unintended recovery-directory creation during audit
and added a test for all three service-description prefixes.

The reviewed run found ten units and ten saved connections. The merged candidate
retained the saved document's content; the only serialization difference after
the targeted safety edits was a trailing newline. The unit-only candidate
would reset sleep settings and VPN profiles, lose the SMB VPN assignment and
SFTP override, and reorder connections. It was not installed. The original
corrupted-recovery note above describes the earlier state; current saved display
names no longer have the `Cloud Mounter:` prefix.

Account checks found both onedriver Online caches identify the same University
business drive, while `uutzinger OneDrive` is intended to be personal. The
personal-labeled service was stopped and its login enablement disabled; its
saved `start_at_login` value was changed from true to false through a validated,
one-field atomic update. Its old cache and tokens remain untouched pending
personal browser reauthorization. The separate abraunegg personal mirror used a
different 5 GB drive; `--display-sync-status` reported about 333 MB out of sync
without synchronizing. Earlier UI evidence also supported restoring the VPS
SFTP per-connection preload override to off, 30 seconds, depth 2; a second
one-field validated update did so. Box, both Google Drives, SFTP, and SharePoint
passed read-only rclone root listings. SMB remains unprobed because Cisco VPN
was not active.

All three recovered OneDrive entries still had the generic account label
`onedrive`, which triggered the editor's shared-account warning. A validated
three-field update changed only those labels to `ua-work-onedriver`,
`personal-onedriver-pending-auth`, and `personal-onedrive-mirror`; the labels do
not select credentials. The new OneDrive draft default now includes its
connection ID to avoid a repeat. The live document was backed up again as
`document.after-label-fix`. The pending personal online label deliberately
reflects that its cached authentication still belongs to the University drive.
The native applet was rebuilt and installed with `just install-user`, and the
COSMIC panel restarted. The full Rust test suite, all-target Clippy, formatting,
and diff whitespace checks passed. The corrected document and final private
backup have matching SHA-256 checksums.

## UA Google Drive clean unmount — October 3, 2026

The applet's `fusermount3 -u` failed because `xdg-document-portal` held open
directory handles within `UA_GoogleDrive/BME310`. The rclone VFS log reported
zero files to upload, and its cache contained only two clean cached files.
Restarting the portal alone did not help because it reopened the handles. We
briefly stopped the portal, ran a normal clean unmount, stopped the generated
mount service, then restarted the portal. The mount was absent, its mountpoint
empty, the mount service inactive, and the portal active afterward. No lazy
unmount or cache deletion was used. Separately, this mount's log reported Google
Drive 403 request-quota errors and warned that `ua_gdrive` uses rclone's shared
Google client ID; that needs its own follow-up.

Follow-up inspection found Google Drive preload enabled for 60 seconds with
unbounded recursive `vfs/refresh` and no custom `client_id` or `client_secret`
on `ua_gdrive`. The first quota 403 appeared 25 seconds after mount startup and
errors continued for about eight minutes. The refresh could have contributed
to the initial request burst, but timing does not prove it caused every error;
the shared client quota and ongoing directory access remain separate factors.

## Version 0.5.0 release preparation — October 3, 2026

The README, Debian changelog, AppStream metadata, Flatpak source tag, Cargo
package version, and release notes now describe SharePoint Online and the
manual-only SharePoint Offline mirror without claiming unattended scheduling.
The user accepted the installed compact Reorder window. `just verify` passed
formatting, all-target compilation, Clippy, and the required test suite; its
known COSMIC-specific desktop/AppStream warnings remained nonfatal. `just deb`
produced the 0.5.0 amd64 package, and package inspection found the applet,
OneDrive helper, desktop entry, icon, and AppStream metadata.

## SharePoint Offline installed manual sync — October 3, 2026

After the temporary local test directory disappeared, the approved disposable
SharePoint subfolder was copied back into the local test directory and checked
read-only: the probe and access marker matched. A local-only edit to `probe.txt`
then produced an installed Modify Preview of one upload and zero deletes. The
installed Sync Now reported completion. An independent `rclone check --download`
found two matching files and zero differences; the generated bisync Preview
found no remaining changes. The connection's background timer remained disabled.
This completes the installed manual-control test; unattended scheduling remains
a separate task.
