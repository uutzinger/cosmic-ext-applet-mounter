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
- [x] Ask `pop-os/cosmic-flatpak` maintainers for architecture guidance before
  submission. The October 7 response established the D-Bus-first direction
  recorded in the dated completion note below.

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

## Version 0.5.1 release candidate — October 5, 2026

Version 0.5.1 adds guarded unattended SharePoint Offline scheduling, makes the
popup switch the persistent login/startup choice for Online mounts and Offline
mirror timers, adds bounded SharePoint Online preload, and moves the audited
user-facing UI boundary into the Fluent catalog. Preload defaults off for every
provider. The README and the new Google Cloud OAuth guide also clarify that
authentication belongs to the selected mount or mirror engine.

The unattended SharePoint gate covered Preview and confirmed initial Sync Now,
repeated scheduled operation, pause/resume, network-loss recovery, target-change
protection, and installed status feedback. Persistent-startup testing after a
real login restored only the saved UA Box Online mount and SharePoint Offline
timer while keeping all saved-off connections inactive. Sleep/wake behavior was
reviewed and tested to the available hardware limit under the documented
best-effort gate.

Final release-candidate validation passed formatting, locked all-target checks,
Clippy with warnings denied, the localization audit, `git diff --check`, strict
credential-pattern scans, and 290 automated tests; seven explicitly external or
live tests remained ignored. `cargo vendor --locked` succeeded without the
previous duplicate-libcosmic-source error. Desktop and AppStream validation
reported only the documented COSMIC category and `binaries` extension
compatibility notices.

The final Debian package reports version 0.5.1 and architecture amd64 and
contains the applet, OneDrive authentication helper, desktop entry, AppStream
metadata, icon, copyright, and changelog. Its SHA-256 is
`15ba6b3fb86bb32cab82a1c2bbae4d5106cc1d192a0a98f6528d4da9c4903ca8`.
Install/remove/reinstall/remove testing preserved the existing custom applet and
COSMIC configuration. The package was left uninstalled, and restarting the
COSMIC panel successfully relaunched the user-local candidate.

The reviewed release commit `8aeeb68` was pushed to `origin/master` before the
annotated `v0.5.1` tag was created and pushed. GitHub Release **Cloud Mounter
0.5.1** was published as a normal release with the amd64 Debian package and
`SHA256SUMS`. A clean download of both published assets passed `sha256sum -c`.
The public tag, README, Google Cloud OAuth guide, and all four tag-based
AppStream screenshot URLs returned HTTP 200. The completed Tooltip Review was
intentionally archived and removed from the repository as part of the release.

## GitHub Debian release automation — October 7, 2026

GitHub Actions now builds native Debian packages on Ubuntu 22.04, 24.04, and
26.04 for amd64 and arm64. The workflow installs the declared Debian build
dependencies and pinned Rust 1.95.0 toolchain, runs `dpkg-buildpackage`, checks
package metadata and architecture, generates checksums, uploads artifacts, and
publishes tagged releases. Local helpers validate release metadata, commit and
tag state, monitor the workflow, and download its artifacts.

The first cloud run exposed missing declared build dependencies. The second
exposed test fixtures that existed only in an ignored local archive. Both were
corrected, and neutral legacy-service fixtures were committed under
`tests/fixtures/legacy-services`. Run `37642473346` then passed all six matrix
jobs. Its Ubuntu 24.04 amd64 package passed metadata, layout, installation, UI,
settings, OneDrive mount/unmount/remount, and journal checks on Pop!_OS 24.04;
other matrix packages received structural rather than live-platform testing.
The test package was removed without purging user data and the user-local build
was restored.

Release metadata was advanced consistently to 0.5.2. Running
`scripts/build_release.sh` created and pushed tag `v0.5.2`; all six package jobs
in run `37649661706` passed. The initial publishing job failed because GitHub
CLI lacked explicit repository context in a job without a checkout. The six
artifacts were downloaded and verified, the public release was recovered with
all packages plus combined `SHA256SUMS`, and the workflow was corrected with
`GH_REPO: ${{ github.repository }}` for subsequent releases. Obsolete local
v0.5.1 artifacts were removed while the complete v0.5.2 artifact set was kept.

## COSMIC Flatpak architecture guidance — October 7, 2026

The locally built Flatpak had already passed the repository's `just build` and
`just build-changed` paths and had been installed for panel, Store metadata,
settings, host-dependency, and representative mount/mirror testing. The open
question was repository acceptance of its broader host integration rather than
manifest syntax.

The architecture and permission question was posted in the official COSMIC
Flatpak Mattermost channel. Michael Murphy (`@mmstick`), a System76 engineer
and Pop!_OS/COSMIC maintainer, advised using D-Bus APIs wherever they are
available and noted that spawning commands on the host requires additional
review scrutiny. He specifically recommended the `freedesktop-rs/nmrs` Rust
library for NetworkManager. The resulting direction is to replace `nmcli` with
`nmrs`/NetworkManager D-Bus, replace `systemctl --user` with the systemd user
manager D-Bus API, minimize remaining host execution, and explicitly justify
the storage-engine commands that cannot be replaced. Implementation and live
validation are tracked in the new D-Bus migration category at the end of
`Task List.md`.

## Flatpak host-command inventory — October 7, 2026

✅ `docs/engineering/flatpak-host-command-inventory.md` inventories the production commands routed through
`flatpak-spawn --host`, with dynamic command patterns and source locations. It
classifies NetworkManager, systemd, and document-portal operations as D-Bus
replacement candidates; identifies `rclone`, `onedriver`, and `onedrive` as
unavoidable storage-engine invocations; and marks runtime diagnostics, shell
orchestration, log scraping, and replaceable network probes as obsolete host
commands. Required FUSE and host mount-namespace inspection are called out
separately as unavoidable non-engine host helpers. Development-only tests and
probes are separated from the shipped applet surface. No D-Bus migration code
or Flatpak permission change was implemented by this inventory step.

## NetworkManager D-Bus migration — October 7, 2026

✅ Production NetworkManager integration now uses a typed `zbus` system-bus
adapter instead of host `nmcli`. It discovers saved VPN and WireGuard profiles,
activates and deactivates connections by UUID, reads active-connection state,
checks global and per-interface readiness, reports dependency availability,
and reads metered state. Existing desktop NetworkManager secret agents remain
responsible for authentication prompts initiated by `ActivateConnection`.

✅ Scheduled mirrors no longer embed the `nmcli`/shell/`grep` metered condition.
Their generated service calls an app-owned `--require-unmetered-network` guard,
which performs the same NetworkManager D-Bus query and fails closed if state
cannot be determined. The Flatpak manifest now grants only
`--system-talk-name=org.freedesktop.NetworkManager` for this integration, and
the packaging rationale documents that permission.

✅ `cargo test --all-targets` passed with 289 tests passed and 8 ignored
live/destructive tests; the focused saved-profile/settings test also passed.
A read-only live native NetworkManager smoke test passed against the host system
bus, covering global state, saved-profile discovery, and saved-profile state.
Activation/deactivation was not exercised automatically because it would alter
the user's live VPN state. Rebuilt-Flatpak and interactive VPN validation remain
tracked by the final migration validation item.

The production boundary excludes test programs and the standalone host-runner
probe. Their legacy parser fixtures and intentional `nmcli general status`
probe remain available for regression and transport testing, but the shipped
applet contains no reachable `nmcli` execution path or fallback.

## systemd user-manager D-Bus migration — October 7, 2026

✅ Production `systemctl --user` execution was replaced with the user systemd
D-Bus API. The shared manager now performs daemon reload, enable/disable,
start/stop/restart, status reads, and failure reset through typed `zbus` calls.
Start, stop, and restart subscribe before queuing work and wait for the matching
`JobRemoved` signal so failed jobs are not reported as successful.

✅ The less-visible call sites were migrated too: synchronous popup status,
sleep-cleanup active-state and stop-policy checks, deliberately nonblocking
sleep stops, runtime dependency detection, and mirror `InvocationID` lookup.
The Flatpak manifest grants session-bus access only to
`org.freedesktop.systemd1`, and the packaging rationale records why. The
system-manager Cisco agent start, obsolete `journalctl` pending-count scrape,
and runtime `systemd-analyze verify` are separate inventory items and are not
`systemctl --user` migration gaps.

Development-only probes and integration-test cleanup retain direct
`systemctl --user` calls by policy. They are not linked into normal applet
execution and remain documented separately in `docs/engineering/flatpak-host-command-inventory.md`.

## Documents portal D-Bus migration — October 8, 2026

⏳ Selected document-portal folders now resolve through a validated D-Bus
adapter whenever the portal supports a sandbox-safe method. Sandboxed portal
v5+ uses `GetHostPaths`; native runs use `Info`. The adapter obtains the actual
portal mount point, extracts direct or app-scoped document IDs, handles the
portal's NUL-terminated byte paths, and rejects malformed or non-absolute
results. Flatpak already permits `org.freedesktop.portal.*` by default, so no
new manifest permission was added.

The installed Pop!_OS Documents portal reports interface version 4. Its
host-path-returning `Info` and `Lookup` methods are explicitly unavailable to
sandboxed callers, while sandbox-safe `GetHostPaths` begins with version 5.
The old host `flatpak document-info` invocation therefore remains only as a
version-4 compatibility fallback; this migration item cannot be marked fully
complete without either raising the minimum portal version or accepting a
folder-selection regression.

Focused portal parsing tests and `cargo check --all-targets` passed. The full
all-target suite passed with 291 tests and 10 explicitly ignored live or
destructive tests. A separately enabled read-only live test resolved an
existing exported document through the native D-Bus `Info` path and matched
its recorded host origin.

## Mirror progress journal removal — October 8, 2026

✅ The applet no longer invokes or scrapes `journalctl` to estimate progress for
an active rclone bisync. Scheduled runs enable rclone's JSON-line log format in
the app-owned mirror work directory and atomically publish a small versioned
state file containing systemd's 32-character `INVOCATION_ID`. The reader gets
the active unit invocation through the existing user-systemd D-Bus adapter and
rejects stale state before deriving the Path1/Path2 total and completed
copy/delete count.

The reader rejects malformed and oversized state, treats a partially written
last JSON record as still in progress, and preserves rclone failure and success
handling. The production `Journalctl` executable variant was removed. Focused
pending-count tests passed, and the ignored disposable live-bisync test was run
explicitly and passed with the generated scheduled runner, its invocation state
file, and real rclone JSON output. `cargo check --all-targets` passed, and the
full suite passed with 290 tests passed and 10 explicitly ignored live or
destructive tests.

## In-process network readiness probes — October 8, 2026

✅ Production readiness checks no longer invoke host `ip`, `getent`, or `nc`.
Route checks use an IPv4/IPv6 UDP connect to ask the kernel for target-specific
route selection without transmitting a packet. DNS checks use Tokio's
asynchronous system resolver, and endpoint checks use a Tokio TCP connection.
The existing Flatpak `--share=network` permission makes these operations
available inside the sandbox without adding another permission.

Route and DNS checks retain a five-second bound; endpoint connections retain
the prior three-second bound. All three respond to cancellation, expose only
the existing redacted `ready`/`not ready` details, and require no host command
or newly added permission. The obsolete `Ip`, `Getent`, and `Nc` executable
variants were removed. Focused loopback tests cover route selection, localhost
resolution, open and closed TCP endpoints, invalid endpoint handling, and
cancellation. Separate read-only lookups inside the Freedesktop 25.08 Flatpak
runtime with `--share=network` resolved IPv4 and IPv6 addresses for both
`localhost` and public `example.com`, confirming sandbox resolver access
without installing a replacement applet.
`cargo check --all-targets`, strict all-target Clippy, and the full suite passed;
the suite reported 291 tests passed and 10 explicitly ignored live or
destructive tests.

## Runtime systemd unit verifier removal — October 8, 2026

✅ Unit installation no longer invokes host `systemd-analyze --user verify`.
Before writing, the app now strictly validates its generated service/timer
schema, ownership markers, required sections, allowed directives, fixed policy
values, timer/service pairing, and command shape. After the atomic write it
calls the user manager's D-Bus `Reload` and `LoadUnit`, then requires a loaded
unit with no `LoadError`, the exact app-managed `FragmentPath`, and no pending
daemon reload. If reload or load validation fails, installation restores the
previous unit (or removes the new one) and reloads that rollback.

Systemd's parser remains an authoritative Linux build/test check over freshly
generated paired service and timer files. The Debian package and Debian CI now
declare `systemd` as a build dependency, while the production
`SystemdAnalyze` executable variant and runtime request are removed. This
preserves generated-unit verification without requiring another
`flatpak-spawn --host` command or weakening rollback behavior.

`cargo check --locked --all-targets`, strict all-target Clippy, and the full
suite passed; the suite reported 294 tests passed and 10 explicitly ignored
live or destructive tests. The separately enabled read-only live user-systemd
D-Bus smoke test also passed, including exact managed-fragment load validation.

## OpenConnect D-Bus lifecycle verification — October 10, 2026

✅ The saved `UA Engineering OpenConnect` profile was activated directly with
NetworkManager D-Bus. NetworkManager reported active-connection state `2` and
VPN state `5` (activated). Its IPv4 configuration installed the private
`10.138.0.0/17` route and UA DNS servers `128.196.11.233` and
`128.196.11.234`, providing route and DNS readiness evidence without probing an
unknown internal service.

✅ D-Bus deactivation removed the VPN active-connection object while preserving
the unrelated active connections. A second D-Bus activation completed the
OpenConnect authentication flow, returned to the activated states, and restored
the same private route and DNS configuration. Task (6) lifecycle steps 1–4 are
complete. Logout/login or reboot autostart and post-session reconnection remain
pending and must be verified from the resumed desktop session.

⏳ After reboot, `nm-applet` autostarted as expected, owned
`org.freedesktop.network-manager-applet`, and had the correct display, Wayland,
desktop, and session-bus environment. Nevertheless, two direct NetworkManager
D-Bus activations remained in preparation and opened no authentication dialog;
each later failed and its active-connection object was removed. Restarting only
`nm-applet` after COSMIC was fully running changed its registration order. The
next D-Bus activation immediately presented authentication and reached active
connection state `2` and VPN state `5`, restoring `10.138.0.0/17` and the two UA
DNS servers. This demonstrates a post-login secret-agent ordering defect: mere
process or well-known-bus-name detection is insufficient. Task (6) step 5 stays
open until the session setup is corrected and succeeds without a manual agent
restart.

✅ The replacement on-demand service design passed its first live test. The old
login autostart entry was absent and the static
`cosmic-mounter-openconnect-agent.service` began inactive. Starting it through
the systemd user-manager D-Bus completed the service's registration-stabilizing
`ExecStartPost`; NetworkManager then logged the `nm-applet` secret-agent
registration. Subsequent NetworkManager D-Bus activation presented the full
three-field OpenConnect window, the user completed phone authentication, and
the VPN reached active-connection state `2` and VPN state `5`. D-Bus inspection
again found route `10.138.0.0/17` and UA DNS servers `128.196.11.233` and
`128.196.11.234`.

✅ The applet implementation now reads `vpn.service-type` from saved
NetworkManager settings and starts the fixed authentication-agent unit through
the existing job-waiting systemd user-manager D-Bus adapter only for
`org.freedesktop.NetworkManager.openconnect`. Missing or failed service startup
stops before VPN activation with installation guidance. Non-OpenConnect
profiles do not acquire this dependency. The applet deliberately leaves the
agent running for possible reauthentication; `PartOf=graphical-session.target`
owns end-of-session cleanup, so the applet never stops a shared agent.

Focused profile-selection and unit-name tests passed, as did the live
application helper test against the installed static service. Formatting,
strict all-target Clippy, and the complete suite passed; the suite reported 295
tests passed and 11 explicitly ignored live or destructive tests. Task (6)
step 5 remains open only for installation of this updated build followed by a
logout/login or reboot reconnection with no terminal intervention.

### Installed OpenConnect plus SMB lifecycle — October 10, 2026

✅ After login and installation of the updated applet, the final acceptance run
began with `cosmic-mounter-openconnect-agent.service`, the selected VPN, and the
SMB mount inactive. Enabling `UA ENGR Research Storage VPN` in Cloud Mounter
started the authentication agent on demand through the systemd user-manager
D-Bus adapter. NetworkManager D-Bus activation presented the complete
three-field OpenConnect authentication window; after phone authentication, the
VPN reached its active state, restored route `10.138.0.0/17` and UA DNS servers
`128.196.11.233` and `128.196.11.234`, and the applet mounted
`ua_engr:Research/Utzinger` at `/home/uutzinger/Cloud/UA_ENGR`. The user
confirmed that the mounted content was accessible.

✅ Disabling the same row in the installed applet cleanly stopped its managed
mount service, removed the FUSE mount, saved `start_at_login: false`, and
deactivated `UA Engineering OpenConnect` because no configured mount still
needed it and disconnect-when-unused was enabled. The OpenConnect
authentication agent remained active intentionally: it is a shared secret
agent retained for possible reauthentication and owned by the graphical
session rather than by one mount. This completes Task (6) lifecycle step 5 and
the successful installed-native activation, readiness, SMB mount/access,
unmount, and conditional-deactivation path.

⏳ Authentication cancellation and failure paths, confirmation that COSMIC's
generic single-field prompt never remains open, the rebuilt Flatpak lifecycle,
and final removal of proprietary Cisco handling remain separate Task (6)
acceptance work. This result applies to the NetworkManager OpenConnect profile
for the Cisco-compatible gateway, not to direct Cisco Secure Client control.

### Installed Flatpak OpenConnect plus SMB positive lifecycle — October 10, 2026

✅ The rebuilt Freedesktop 25.08 GUI prototype at user-installed Flatpak
commit `b95fbbcccaa1` passed the positive OpenConnect lifecycle. The run began
with the ENGR mount and VPN inactive. Enabling `UA ENGR Research Storage VPN`
opened the three-field OpenConnect authentication flow, completed phone
authentication, activated the NetworkManager VPN, and mounted
`ua_engr:Research/Utzinger` through the managed user service. Live inspection
confirmed the `fuse.rclone` mount, private route `10.138.0.0/17`, and UA DNS
servers `128.196.11.233` and `128.196.11.234`; the user confirmed that mounted
data was accessible.

✅ Turning the connection off reported `Unmount completed`. The managed service
then reported inactive/dead with `Result=success`, the FUSE mount was absent,
the ENGR VPN was no longer active, and the persisted connection state was
`start_at_login: false`. The tested Flatpak process remained running throughout,
which distinguishes this evidence from the earlier installed-native lifecycle.
This completes ordered Flatpak acceptance step 2; cancellation is the next
acceptance case.

✅ The installed prototype also passed ordered Flatpak acceptance step 3.
Closing the three-field OpenConnect dialog produced the bounded notification
`Mount failed: authentication was canceled or failed`; the connection row
returned to off and no competing COSMIC single-field prompt appeared. Live
inspection found the mount service inactive/dead, no ENGR FUSE mount, no stale
UA VPN active object, and persisted `start_at_login: false`. The shared
OpenConnect authentication agent remained active intentionally.

✅ Ordered Flatpak acceptance step 4 passed with a deliberately rejected phone
approval using valid primary credentials. OpenConnect displayed `Login failed`;
after its dialog was closed, Cloud Mounter reported that VPN authentication
failed or was canceled. No COSMIC single-field prompt appeared. The row returned
to off, and live inspection confirmed no UA VPN active object, ENGR FUSE mount,
or running managed mount service, with persisted `start_at_login: false`.

✅ The COSMIC BaseApp dependency gate was resolved before bisync work. Contrary
to the earlier unavailable-ref observation, Flathub now publishes
`com.system76.Cosmic.BaseApp//stable`; the configured COSMIC remote does not.
The installed BaseApp at commit `b3f1b274d540` declares Freedesktop Platform
and SDK 25.08, exactly matching the project manifest. The GUI prototype was
updated to declare that base, rebuilt successfully, installed as commit
`20bd5aea1b91`, and launched on Freedesktop Platform 25.08.

✅ Comparing the effective permissions found no new application D-Bus or writable
filesystem access. NetworkManager, login1, the user systemd manager, the
Flatpak host interface, the app-owned buses, and the existing scoped paths are
unchanged. The effective report also contains the runtime/base defaults for
read-only GTK/KDE color-scheme configuration. The final reproducible tagged
manifest build and repeated lifecycle acceptance remain the later final-package
gate; they are not implied by this prototype dependency resolution.

### Bisync coordinator inventory and shared-engine decision — October 10, 2026

✅ Task (5) began with a source-backed inventory of all five generated-shell
modes, their ordered checks, mutations, recovery ownership, SharePoint second
pass, structured scheduled progress, success-only retention, initial markers,
and failure boundaries. The detailed equivalence contract is recorded in
`docs/engineering/2026-10-10-bisync-coordinator-inventory.md`; no execution path changed during inventory.

✅ Manual and scheduled rclone Offline Mirror operations will use one typed Rust
coordinator. The UI and user-systemd service will call the same fixed-mode
transaction implementation so backup, ownership, SharePoint, progress, and
retention behavior cannot drift. The restricted native/Flatpak host entry point
remains the next design gate.

✅ The worker entry-point architecture was accepted. Native/source and Debian
services will execute the absolute installed Cloud Mounter binary; a
Flatpak-generated service will use `/usr/bin/flatpak run
--command=cosmic-ext-applet-mounter` with the application ID. Both enter the
same restricted Rust worker before GUI or applet-bus initialization. The caller
may supply only a fixed mode and connection UUID; all paths and rclone arguments
come from revalidated saved state. A non-mutating disposable user-systemd probe
remains required before this accepted design is treated as runtime-confirmed.

### Provider-wide online lifecycle and NetworkManager-only cleanup — October 10, 2026

✅ Online operation dispatch is now exhaustive across every provider. OneDrive
selects the onedriver engine; SharePoint, Google Drive, Box, SMB, and SFTP
select the rclone engine. Both engine paths converge on the same managed online
start and stop helpers for systemd user-manager D-Bus activation, bounded
transitional-state reconciliation, slider persistence, clean mount
disappearance verification, and conditional NetworkManager VPN release. A
regression test covers all six providers and proves that Offline Mirror modes
cannot enter this path. The successful installed SMB/OpenConnect lifecycle is
the live example; no offline-mirror or bisync functionality is inferred from
it.

✅ Direct proprietary Cisco support was removed from production. The applet no
longer discovers or invokes Cisco CLI, GUI, agent, status, startup, or disconnect
operations, and dependency documentation and the host-command inventory now
describe Cisco-compatible access only through NetworkManager OpenConnect. An
old serialized `Cisco` profile kind loads as an inert `LegacyCisco`
compatibility marker and is rejected before D-Bus access with instructions to
select a saved NetworkManager profile; no UUID is guessed or rewritten.

✅ Formatting, all-target check, strict Clippy, installer shell parsing, diff
checks, and a production Cisco reachability scan passed. The full test suite
passed serially outside the restricted network sandbox: 294 tests passed and 12
live or destructive tests were explicitly ignored. The release binary was
installed to the user location and exactly matches the built artifact.

⏳ Task (6) still requires user-visible cancellation/failure testing,
confirmation that COSMIC leaves no competing generic single-field prompt, and
a complete installed-Flatpak lifecycle. The final manifest intentionally
fetches tagged `v0.5.2`, so it cannot exercise these uncommitted changes; the
configured COSMIC remote also does not currently offer its requested
`com.system76.Cosmic.BaseApp//stable` ref. Those gates must not be reported as
complete using an old tagged build.

✅ As an intermediate packaging check, the current release binary was built and
installed with the repository's Freedesktop 25.08 GUI prototype manifest after
bringing its D-Bus permissions in sync with the final manifest. The installed
Flatpak commit is
`c8f461f6a408b77cfa8074b9c69227b52a4ac10718607de21fbe441d1c5970fe`.
Read-only calls from inside that sandbox successfully reached the user systemd
manager and NetworkManager; NetworkManager returned state `70`. This proves the
required bus access, not OpenConnect authentication or mount lifecycle behavior.

### Installed OpenConnect cancellation — October 10, 2026

✅ The user requested the VPN-dependent SMB mount and closed the three-field
OpenConnect authentication requester. This requester supplied no labeled Cancel
button, so its window close control was the available cancellation action. The
Cloud Mounter slider returned to off. Read-only live inspection confirmed the
managed SMB unit was `inactive/dead` with `Result=success` and exit status zero,
the SMB path had no FUSE mount, NetworkManager had no active VPN connection, and
the saved connection retained `start_at_login: false`. The shared OpenConnect
authentication agent remained active as designed for later authentication.

⏳ The user clarified that the resulting notification was generated but had
already disappeared by the time the applet UI was reopened. The timeout had
been measured from background notice creation rather than from the next popup
opening. `popup_notice_expired` now starts the 10-second visible interval at the
later of those two events. Focused unit coverage, formatting, strict Clippy,
and diff checks passed. Installed user verification remains pending.

⏳ Confirmation that COSMIC did not leave its separate generic single-field
prompt also remains pending. Rejected or expired Duo, timeout,
network-interruption, and reconnect cases are still open.

⚠ A subsequent post-login cancellation retest reproduced the competing-agent
race. The OpenConnect service was already active from 12:02, then COSMIC
registered its generic NetworkManager secret agent at 12:07:22. Cloud Mounter's
OpenConnect activation at 12:07:27 reached NetworkManager, but `StartUnit` was a
no-op and no three-field dialog process appeared; NetworkManager recorded the
secret request as user-canceled. A second request remained in `activating` with
`nm-openconnect-service` running but no authentication dialog.

✅ The OpenConnect-only preparation now calls systemd user-manager
`RestartUnit` and waits for the complete job, including the service's
one-second `ExecStartPost`, before NetworkManager activation. This deliberately
re-registers `nm-applet` after COSMIC while preserving the D-Bus-only applet
boundary. `docs/openconnect.md` documents why restart rather than start is required.
Installed live verification remains pending.

✅ After logout/login with the installed `RestartUnit` build, COSMIC registered
its generic secret agent at 12:13:13 and Cloud Mounter restarted `nm-applet`,
which registered afterward at 12:13:26. NetworkManager accepted the applet's
activation at 12:13:27 and the three-field OpenConnect authentication window
appeared. This live result closes the registration-order portion of the fix.

✅ The user closed the authentication window. NetworkManager recorded the final
secret request as insufficient at 12:13:33, six seconds after activation. Live
inspection found no active VPN, no managed mount unit, no FUSE mount, and the
saved SMB connection still had `start_at_login: false`. The user was not certain
of every transient notification message, so no exact wording or readability
claim is recorded.

✅ Readiness monitoring now treats an app-requested NetworkManager activation
that returns to `Disconnected` or `Failed` as terminal. It will replace the
waiting state with an explicit authentication/activation failure instead of
waiting until the 90-second OpenConnect deadline. Focused unit coverage was
added; installed live verification of the final notice remains pending.

✅ The installed terminal-state build was retested. After closing the
three-field authentication window, the user could read and report the explicit
mount-failed-or-canceled and VPN-activation-canceled-or-failed result. Live
inspection found no active VPN and no managed mount unit; only the shared
OpenConnect authentication agent remained active as designed. Cancellation
rollback and cancellation-notice visibility are now verified. Exact punctuation
is not treated as a contractual message assertion.

✅ A deliberate rejected second-factor attempt displayed `Login failed` inside
the three-field OpenConnect dialog while NetworkManager appropriately remained
in `activating` state to permit retry. The user then closed the dialog and read
the applet's failure result. Live inspection found no OpenConnect service or
authentication-dialog process, no active VPN, no managed mount unit, and no
mount at the ENGR target. Rejected-authentication cleanup is verified.

⚠ The first untouched-dialog timeout test exposed incomplete cleanup. The share
remained unmounted, its saved state stayed off, and NetworkManager no longer
listed the VPN activation, but `nm-openconnect-auth-dialog` remained alive in
the authentication-agent service control group well beyond the applet's
90-second deadline. Restarting that unit closed the orphan, but the stop phase
then consumed nearly systemd's default service-stop allowance.

✅ The timeout path now deactivates any remaining NetworkManager active object
over D-Bus and restarts the OpenConnect agent through systemd user-manager
D-Bus. The documented unit now bounds shutdown with `TimeoutStopSec=5` and
explicitly cleans its control group. This preserves the D-Bus-only applet
boundary and leaves a fresh authentication agent after timeout. Focused tests,
formatting, strict Clippy, and the code diff check passed. The release binary
was installed and matched the built artifact. The installed service contains
the five-second stop and control-group settings, and systemd reported those as
its effective properties. Reloading the applet and repeating the live timeout
test remain pending.

✅ The installed timeout cleanup passed its repeated live test. NetworkManager
started the OpenConnect activation at 18:28:58 and the untouched three-field
dialog remained pending. At 18:30:27, the applet deactivated that exact VPN
object over D-Bus. Systemd stopped the complete authentication-agent control
group, removing both the service and dialog, then registered a fresh
`nm-applet`; the unit was active again at 18:30:28. No VPN or managed mount unit
remained, the ENGR target was not a mountpoint, and `start_at_login` remained
false. The installed five-second stop bound and control-group policy were
effective. User confirmation of the visible timeout notice is tracked
separately from this lifecycle result.

✅ The user confirmed the applet displayed the mount-failure notice explaining
that the VPN did not become ready within its timeout. The timeout test is now
verified at both the lifecycle and user-visible result boundaries.

✅ The user confirmed that no separate COSMIC single-field prompt appeared
during the corrected installed-app activation flow. Together with the live
agent-registration timestamps and the three-field OpenConnect dialog, this
verifies the competing-agent UI portion of Task (6).

✅ A successful recovery followed the cancellation, rejected-authentication,
and timeout cases. NetworkManager activated OpenConnect and created `vpn0` with
address `10.138.88.8/17`, private route `10.138.0.0/17`, and UA DNS servers
`128.196.11.233` and `128.196.11.234`. The managed ENGR rclone service was
active, the FUSE mount was present, and a bounded directory listing succeeded.

⚠️ A controlled network-interruption test scheduled exact-profile Wi-Fi
recovery before disconnecting Wi-Fi for 15 seconds. Wi-Fi returned, but
NetworkManager stopped the OpenConnect VPN and did not reauthenticate it. The
managed rclone service and FUSE mount remained present, while bounded remote
access did not recover. The installed applet consequently showed a misleading
healthy blue switch and cached zero pending-file count. This evidence does not
satisfy the broader automatic safe-detach and remount requirements.

✅ The applet now models a still-mounted Online connection whose required
network or VPN is unavailable as degraded. It retains Unmount, hides the cached
pending count behind an attention marker, and tells the user to turn the
connection off and on to reauthenticate. Focused controller coverage includes
both network and VPN dependency loss. The installed UI displayed the expected
blue mounted switch, `Mounted, connection unavailable` status, `!` instead of
the stale zero count, retained Unmount, and off/on recovery guidance. Clean
unmount and authenticated remount validation remain pending.

⚠️ The first degraded-state unmount attempt saved `start_at_login: false`
but clean FUSE detach failed while the private SMB endpoint was unreachable.
The service and mount correctly remained active rather than being forcefully
detached. Inspection then found that Repair was allowed by the controller but
not rendered in the popup, so the documented confirmed lazy-detach recovery was
unreachable. A degraded-row Repair control and off-state persistence were added;
installed confirmation remains pending.

✅ Logging out after the failed clean-unmount attempt ended the stale user-session
rclone process and mount. After login, the managed unit was inactive and
disabled, the ENGR target was no longer a FUSE mount, and the saved
`start_at_login` value remained false. Therefore the row correctly appeared off
rather than degraded. This verifies safe session cleanup, but it does not count
as installed confirmation of the newly exposed Repair button.

✅ The subsequent applet-driven recovery succeeded. The three-field
OpenConnect and phone-authentication flow restored `vpn0` at
`10.138.88.25/17`, route `10.138.0.0/17`, and UA DNS servers
`128.196.11.233` and `128.196.11.234`. The managed rclone service and FUSE mount
were active, a bounded ENGR directory listing succeeded, preload started, and
`start_at_login` was saved true. Live Repair-button validation remains the only
open substep of this controlled interruption case.

⚠️ The first installed Repair-button test exposed an ambiguous inline hit
target, not execution of Repair: clicking the visible Repair area produced the
exact `Unmount completed` notice, and lifecycle inspection confirmed the clean
primary-unmount path stopped the service and detached the mount. Repair is now
a separate full-width action below the warning. It arms for 1 to 30 seconds,
visibly changes to `Confirm Repair`, and rejects repeated first-click delivery
during the minimum interval. Installed retesting remains pending.

✅ The separated full-width Repair action passed the installed safety test
against a deliberately busy mount. Clean Unmount first failed without forcing
detach. The first Repair click displayed the confirmation warning; the second
started Repair, which stopped the service, lazily detached the busy FUSE mount,
and retained `start_at_login: false`. The temporary shell holding the mount was
closed afterward. The user observed one remaining presentation defect: the
Repair button stayed visible while asynchronous cleanup ran. It is now hidden
as soon as confirmation starts; installed confirmation of that timing remains
pending.

✅ Final installed confirmation passed. With the VPN disconnected and a
temporary shell deliberately holding the mounted directory busy, the separate
Repair action displayed its warning on the first click and changed to Confirm
Repair. After the required delay, confirmation immediately removed the action
while cleanup ran; `Repair completed` appeared afterward. Lifecycle inspection
found the service inactive and disabled, no ENGR FUSE mount, and
`start_at_login: false`; the temporary holder was then closed. An attempted
no-VPN shortcut was also cleaned up after rclone correctly refused to create the
SMB filesystem without private DNS. The native cancellation, rejected-factor,
timeout, interruption, degraded-state, Repair, and reconnect checklist is now
complete. Rebuilt-Flatpak live authentication remains a separate Task (6)
acceptance gate.

✅ Final regression validation passed: formatting, strict all-target Clippy,
the scoped diff check, and 298 tests passed with 12 explicitly ignored live or
destructive tests. The first sandboxed full-suite attempt could not bind the
loopback listener used by an in-process readiness test; rerunning outside that
network sandbox passed, so this was an execution-environment restriction rather
than a product failure.
The requested on-state was persisted. Successful reconnect is verified.

✅ Task (5)'s restricted bisync worker transport probe is implemented and
verified without touching mirror data. The native disposable user-systemd
service completed in 6 ms and the Flatpak service in 172 ms; both emitted the
single bounded success record and were collected. Missing, extra, malformed,
and disabled production-mode requests are rejected, including an installed
Flatpak extra-argument service that exited with status 2. Dispatch precedes
localization, async-runtime, GUI, and applet initialization, and rebuilding the
development Flatpak added no permission. This proves only safe worker
invocation; production bisync remains on the existing shell pending the shared
Rust coordinator and equivalence tests.

✅ Task (5)'s engine-specific Offline Mirror Repair contracts are accepted and
recorded. Shared rules make Repair a confirmed, bounded recovery-and-pause
operation that preserves all mirror, recovery, engine, lock, ownership, and
initialization state and never synchronizes or uses Online FUSE recovery.
Rclone Repair disables its timer before stopping the finite service and retains
the two-minute bisync safety lock; `abraunegg/onedrive` Repair stops and
disables only its continuous monitor and preserves its token, database, and
engine state. Requirements FR-060A through FR-060C and the separate live-test
boundaries now capture these decisions. No Repair implementation changed in
this design step.

✅ The locally testable Offline Mirror Repair contract is now implemented.
Offline and Online Repair share the protected one-to-30-second confirmation and
pending-action hiding behavior. Typed dispatch reloads the saved connection by
UUID, verifies applet-owned structurally valid units, and separates rclone's
timer-first recovery from `abraunegg/onedrive`'s monitor-only recovery. Both
paths use the systemd user-manager D-Bus adapter, disable their background
service, reset only observed failed state, and reject success unless final
service/timer state is inactive and disabled. No engine command, FUSE detach,
resync, retention, authentication, or filesystem mutation is part of Repair.
The complete automated suite passed with 315 tests and 12 intentionally ignored
tests, and strict all-target Clippy passed. Coordinator-owned cancellation and
separate installed rclone/OneDrive interruption tests remain open, so the Task
(5) implementation item is intentionally not checked complete.

### Task (5) focused failure and native integration closure — October 12, 2026

✅ Completed the remaining coordinator failure matrix and native engine
integration gaps. Required transaction phases stop on failure, timeout, or
cancellation; failed initial work preserves initialization markers. Added
coverage for foreign ownership, missing/changed access anchors, linked recovery
paths and dangling marker links, malformed calendar dates, incomplete/invalid
output, and bounded error redaction. Fixed remote owner inspection to stop on
timeout/cancellation and require complete valid output before ownership or
cleanup decisions. Incomplete ownership data cannot authorize a purge.

✅ The new disposable alias/local and loopback-SFTP tests call the production
Rust coordinator with isolated configuration. Both passed initial dry preview,
initial sync, ordinary preview, two-way changes, deletion recovery, unique
repeated backups, removal of marked old directories, and preservation of
unmarked or invalid-date directories. Their temporary data and SFTP server are
removed by test cleanup. The obsolete shell-era SFTP test and its unused helper
were removed; its checklist disposition is `[o]` with replacement evidence.

✅ Existing authenticated SharePoint identity/folder guards and the compiled
changed-target guard passed. A writable test used only a new UUID child of
`CloudMounter-TeamsMirror-Test-20261002`; initial and ordinary sync independently
passed checksum comparison, Office archive validation, overwrite recovery,
and ordinary preview. Test cleanup purged that child, without touching the
approved parent or other mirrors. Corrected the stale test expectation that
the original bisync outcome contained the later checksum-command log.

✅ Both previously sandbox-blocked socket tests passed with host access. The
full nonignored all-target host suite passed 357 tests, and six ignored engine/
SharePoint tests were explicitly run successfully. Formatting, locked all-target
checking, the 21 coordinator tests, and strict all-target Clippy passed.

⏳ These results close the 0.5.3 focused-test source item, not final Flatpak
acceptance. The accepted release artifact still needs the restricted-worker,
offline-control, and OpenConnect lifecycle tests recorded in Tasks (5) and (6).

### 0.5.3 UI translation audit — October 12, 2026

✅ Extracted the remaining app-owned UI labels, validation messages, operation
summaries, and diagnostics into Fluent, including messages returned by storage,
VPN, systemd, recovery, configuration, import, preload, and sleep modules.
Configuration errors now display localized explanations rather than enum debug
names. Preserved executable/protocol values, user data, machine records, external
error classifiers, and interpolated third-party diagnostics. Coverage and these
intentional exceptions are recorded in `docs/engineering/2026-10-12-ui-translation-audit.md`.

✅ Sleep attention is a typed flag carried over runtime IPC, so translated
messages cannot hide the warning action. Retention singular/plural wording uses
Fluent numeric selectors. Added regression coverage for non-English sleep text,
plural selection, and older IPC payloads lacking the new flag.

✅ `scripts/check-ui-translations.py` checks catalog key uniqueness and literal
Rust references: 1,291 English messages and 1,443 references have no missing or
duplicate keys. Formatting, locked all-target checks, strict Clippy, and all 358
nonignored native host tests passed, including both socket-opening tests.

✅ The separate OneDrive GTK authentication helper also uses Fluent-owned text.
The applet supplies four localized labels in a JSON environment handoff; direct
helper launches read the packaged English catalog. Native, Debian, and Flatpak
install recipes include the fallback catalog. Three helper tests passed without
a GUI or authentication. `just localization-check` also verifies key integrity
and excludes test-module notice fixtures from its production scan.

✅ Installed the audited native release build, helper, and English fallback
catalog with `just install-user`. Byte comparisons confirm all three installed
files match the validated workspace artifacts. Existing saved connection
configuration was not changed. Reload the panel/session to use the new build.

⏳ English is the only available catalog. Live UI layout/wrapping and the final
Flatpak artifact remain separate release acceptance checks.

### Production host-command source reconciliation — October 12, 2026

✅ Reviewed the default Linux applet/settings/IPC, guards, restricted worker,
command factories, generated services, and local child launches. A Rust syntax
scan excluded test/development-feature branches; manual review followed callers,
macros, and package-specific launchers. This is current-working-tree source
evidence, not acceptance of the older v0.5.2 tag or the final rebuilt Flatpak.

✅ Reconciled every row in
`docs/engineering/flatpak-host-command-inventory.md` to Replaced by D-Bus,
Removed, or a justified Unavoidable invocation. Added the Rust coordinator's
recovery mkdir/cat/list/copy/check/purge operations, corrected stale shell and
version-probe descriptions, removed obsolete idle-count/core-stats claims, and
documented host service payloads and sandbox-local launches separately. The
supported portal-v4 compatibility exception remains version-gated and explicit.

✅ The default typed runner now has nine executable kinds. Ten development-only
variants require tests or the explicit `development-probes` feature, and the
unused Printf variant is removed. Cargo requires that feature for the standalone
host-runner probe; its documented build/help commands are updated. Raw legacy
bisync request factories are test-only. An initial default binary-test build
exposed that library cfg(test) does not propagate to its binary-test consumer;
the unused legacy NM detector fixture was therefore guarded by both test and
development-feature conditions. Existing parser/probe coverage is retained.

✅ Restricted the synchronous host bridge to exactly RcloneConfigDump and
HostMountInfo, replacing its arbitrary executable/argument parameters at all
five callers. The displayed OneDrive manual-auth shell instruction remains
display-only; it is not the removed bisync wrapper or an applet shell invocation.

✅ Default and all-feature all-target checks, strict all-feature Clippy,
formatting, localization validation, the three helper tests, and all 358
nonignored host regression tests in both default and all-feature suites passed.
Local link targets and diff whitespace checks also passed. Existing native disposable engine
acceptance is unchanged; no live cloud, mount, VPN, or user-service operation
was initiated by this audit. Source inventory items are complete; Tasks (5)
and (6) retain their final release-artifact acceptance gates.

⏳ The audit also identified existing unbounded synchronous `.output()` waits
in the fixed bridge. A new 0.5.3 pre-build item requires bounded/output-limited
execution off the GUI loop, focused failure tests, and installed responsiveness
acceptance before release. Manifest/permission/documentation reconciliation
remains the following gate. This audit does not certify the bridge's timing
behavior or install a final release package.

### Fixed host-inspection bridge hardening — October 12, 2026

✅ Replaced the blocking bridge with the fixed `HostInspection` adapter in
`src/host_inspection.rs`. Configuration dumps have a 1 MiB per-stream capture
limit; host mountinfo has a 4 MiB limit. Both use a 10-second execution deadline,
no retries, cancellation, strict output validation, and sanitized failures that
do not echo configuration or captured diagnostics. Native mountinfo remains an
asynchronous bounded local `/proc` read, not a polling subprocess. Flatpak uses
the same typed runner with scoped `--watch-bus`; the installed runtime's launcher
help confirms support. Unrelated interactive launches retain their lifetime.

✅ Removed all production app-module `.output()` waits. Remote discovery,
SMB/SFTP prefill, and import backend inspection run as asynchronous UI tasks;
completions carry parsed metadata rather than raw config dumps. Superseded
discovery/prefill requests cancel older work and generation/context/revision
guards preserve newer lists, user edits, identity, and operation notices.

✅ UI mount consumers use a last-successful observation; periodic refresh is
limited to open UI windows, with immediate requests on popup opening and relevant
operation completions. Failures retain the last observation and report a
sanitized warning; obsolete in-flight results are discarded and refreshed.
Setup, unmount, repair, and shared-VPN release take fresh asynchronous observations.
VPN release now fails closed if inspection fails. OneDrive setup also rechecks
mount isolation after potentially long authentication; no mount/sync/VPN action
was initiated to verify this source change.

✅ Fixed a shared-runner timeout gap: output-pipe draining is now inside the
command deadline, even if a descendant holds a pipe after the parent exits.
Cleanup retains the original isolated process-group ID after reaping the parent.
Added regression tests for that hang, scoped Flatpak lifetime flags, fixed bounds,
invalid/truncated output, private diagnostics, pre-cancelled requests, stale
discovery/prefill, edit revisions, and failed/obsolete mount observations.
The final all-feature suite passed 369 nonignored Rust tests; the default suite
also passed 369 before the final post-auth isolation-only wrapper adjustment.
Final default all-target checking, strict all-feature Clippy, formatting, local
link checks, and diff checks passed. Localization validation passed with 1,293
English messages, 1,441 Rust references, and three helper tests. No ignored live
engine test or final Flatpak artifact is claimed by this gate.

✅ Installed the validated native build and English catalog with
`just install-user`; byte comparisons match the workspace release binary and
catalog. The saved connection document SHA-256 is unchanged before/after install.
No saved configuration, generated user service, authentication, or cloud data
was modified. The installed binary still carries the pending source version
0.5.2; release metadata advancement remains a separate 0.5.3 gate.

⏳ Reload the native panel/session and confirm Detect rclone remotes and
SMB/SFTP editor prefill remain responsive and preserve expected values/mount
state, without saving changes or initiating storage/VPN operations. The parent
pre-build task remains unchecked until that installed UI receipt is obtained;
source/tests/install subtasks are complete. Final rebuilt-Flatpak transport/UI
acceptance and subsequent manifest/documentation reconciliation remain open.

### Installed host-inspection UI receipts — October 12, 2026

✅ User screenshots show successful discovery of eight rclone remotes, including
two matching Google Drive remotes. The existing SFTP editor reports settings
loaded with secrets remaining blank; the existing SMB editor reports host/user/
domain loaded with password remaining blank. These establish visible completion
of discovery and both editor prefill paths; private endpoint/account values are
not copied into this record.

⏳ Screenshots alone do not establish responsiveness during execution or popup
mount-state accuracy. Those explicit user confirmations remain before closing
the native bridge-hardening acceptance item; final Flatpak acceptance is separate.

✅ The user subsequently confirmed that the windows are responsive and the main
applet lists all expected connections. Discovery, SMB/SFTP prefill, responsiveness,
and connection-list coverage now have installed UI receipts.

⏳ Current mounted/unmounted state accuracy was not explicitly confirmed by that
reply. Only this remaining native observation is needed to close the parent
bridge-hardening item; final rebuilt-Flatpak acceptance remains separate.

✅ The user confirmed that the mount sliders correctly reflect the current
no-active-mounts state. Together with remote discovery, SMB/SFTP prefill,
responsiveness, and expected connection-list receipts, this completes the native
bridge-hardening acceptance item. Marked its parent and remaining UI subtask
`[x]` in Task List.md. No new active-mount lifecycle or final rebuilt-Flatpak
acceptance is claimed. Manifest/packaging/documentation reconciliation is next.

### Packaging and documentation source reconciliation — October 12, 2026

✅ Compared the final command inventory with both applet manifests, packaging
guide, README, product description, requirements, dependency guide, Fluent text,
native just/Cargo recipes, Debian recipes, shared adapters, and generated worker
launch specifications. Replaced stale scheduled-shell/idle-bisync-count promises
with the shared restricted Rust coordinator and invocation-matched progress
contract. Added bounded asynchronous inspection requirements, explicit engine
minimums, helper/namespace exceptions, and the preserved portal-v4 fallback.
README changes are limited to one concise Flatpak paragraph.

✅ Reviewed all 23 explicit grants. No obsolete command-specific/proprietary-VPN
grant exists to remove: the replacements need their D-Bus service grants, and
storage engines still need the documented host runner and shared paths. Retained
the source manifest's COSMIC settings-daemon grant after checking pinned
libcosmic a991d7b's `dbus-config` proxy/watch code; added the same grant to the GUI
prototype for parity. Documented both Runtime/Settings IPC names and the fact
that host-execution access is not an OS-enforced per-executable allowlist.
No blanket host filesystem or session/system bus socket grant was added.

✅ Preserved the accepted COSMIC BaseApp stable/Freedesktop 25.08 target. Local
ref inspection confirmed the BaseApp's runtime and installed Platform/SDK 25.08.
Kept the production scaffold pinned to v0.5.2 until the separate accepted 0.5.3
tag gate, and documented that this cannot build the pending migrated source yet.
Clarified that prototype/source manifests share the app ID, prototype installation
replaces the user's existing Flatpak, and native/Flatpak owners must not compete.

✅ Added two source packaging contract tests for exact grant parity, accepted
base/runtime, release/prototype identity, default shipping build features, and
packaged authentication helper/Fluent catalog. Both pass. Flatpak-builder parsed
both manifests with 23 grants after correcting the `--show-manifest` invocation
to take only the manifest path. Strict all-feature Clippy, formatting, diff and
local Markdown link checks passed; the full default host suite passed all 371
nonignored Rust tests, including the socket tests. Localization validation remains
1,293 messages/1,441 references with no missing/duplicate keys; all three helper
tests pass. Intentional unsupported-legacy-Cisco errors and display-only manual
auth instructions remain; no obsolete runtime CLI guidance was found in Fluent.

✅ Marked the 0.5.3 pre-build reconciliation item and matching migration-source
documentation item `[x]`; preserved transferred `[o]` records and final-package
gates. Native/source and Debian build recipes compile the same default Rust
adapters; this is source/build evidence, not a fresh Debian package acceptance.
No applet runtime implementation changed, and no package was rebuilt, installed,
tagged, published, or submitted by this review. Saved user configuration,
services, mounts, VPNs, credentials, and cloud data were not modified.

⏳ Final worker acceptance must prove local mirror/recovery metadata access:
resolving an original host path does not authorize sandbox access to it. Recorded
this explicitly in the packaging guide and final Flatpak gate. Any missing path
permission requires a documented approved narrow solution, not blanket host
access or shell rollback. Installed effective grants (including BaseApp defaults),
final source/runtime identity, Debian acceptance, Tasks (5)/(6) lifecycles, and
release/publication remain separate gates. The next release-list item is CI action
runtime cleanup, followed by final verification/native packaging and release prep.

### GitHub Actions runtime cleanup — October 12, 2026

✅ Reviewed the repository's single workflow and upgraded all three old v4 action
references to official releases declaring Node.js 24. Resolved release tags with
read-only `git ls-remote` and pinned immutable commits:

- [checkout v7.0.1](https://github.com/actions/checkout/blob/v7.0.1/action.yml):
  `3d3c42e5aac5ba805825da76410c181273ba90b1`.
- [upload-artifact v7.0.2](https://github.com/actions/upload-artifact/blob/v7.0.2/action.yml):
  `cf430e030ddbb5b0abf93d22962f4752f3646cd9`.
- [download-artifact v8.0.1](https://github.com/actions/download-artifact/blob/v8.0.1/action.yml):
  `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`.

✅ Checked official tagged action metadata for runtime and compatibility with
all existing inputs. Archive upload remains the default and download still
extracts/merges artifacts; the newer download action defaults to failing a digest
mismatch rather than weakening integrity checks. The checkout documentation
specifies runner 2.327.1 or newer for Node.js 24; this workflow uses GitHub-hosted
runners, not self-hosted/container action runners. No unsafe Node opt-out was added.

✅ Local PyYAML validation checked triggers, three immutable action refs/inputs,
all six Ubuntu/architecture targets, read-only build permissions, tag-only release
publication, and release write permissions. `bash -n` passed all six run blocks;
diff whitespace checks passed. `actionlint` is not installed, so these are local
YAML/policy/syntax checks, not an actionlint or hosted-run result. Preserved the
pre-existing addition of systemd to CI build dependencies and all other workflow
behavior. No Rust/app code or README change was needed.

✅ Marked action-runtime cleanup `[x]`. No commit, push, workflow dispatch, tag,
release, or publication was performed. Recorded the fresh hosted verification
step in the following final-verification gate: after an authorized push, dispatch
from the updated branch and inspect all six jobs/artifacts. An old-job rerun
cannot validate the newly edited workflow. Final verification and native Debian
package acceptance are next; final Flatpak acceptance remains separate.

### Local Debian candidate build and inspection — October 12, 2026

✅ Built from an explicit temporary source snapshot rather than cleaning the
dirty workspace or overwriting parent-directory release artifacts. Build
dependencies were satisfied. Reused the declared Cargo cache with locked/offline
resolution and host access for socket tests. The first build compiled and passed
all 371 nonignored tests, then failed because install used a hard-coded target
path despite the supported `CARGO_TARGET_DIR` override. Fixed that one install
line in workspace and snapshot `debian/rules`, preserving the existing catalog
addition, and rebuilt successfully with `dpkg-buildpackage -us -uc -b`.

✅ Saved the main `.deb`, debug `.ddeb`, `.buildinfo`, `.changes`, and an explicit
candidate README in `target/local-deb-2026-10-12.LBoZhW/`. This is an unsigned
Pop!_OS 24.04 amd64 working-tree candidate with current metadata 0.5.2, not the
published v0.5.2 package or accepted v0.5.3 release. The main package is 9,170,244
bytes; SHA-256 is
`c3840524ca93e4028df97633060db000d3891b958bcf8d288f199dca258b6d9e`.
The extracted ELF is x86-64 PIE/stripped with build ID
`6ce5272b6b0d9d3c023becda4dfc6e71cdf9fbb5`. Its generated runtime dependencies
are libc6 >=2.39, libgcc-s1 >=4.2, and libxkbcommon0 >=0.5.0, so this host build
is not an Ubuntu 22.04 compatibility artifact.

✅ Verified all eight payload md5sums and exact source matches for helper,
Fluent catalog, desktop entry, AppStream metadata, and icon. Executables have
0755 modes without setuid/setgid bits; package payload has no user config,
generated services/timers, conffiles, or maintainer scripts. These support a
non-destructive packaging contract but do not prove installed upgrade/removal.
The extracted binary's fixed probe succeeded and malformed identity returned
exit 2, without loading real connection config or starting storage/VPN work.
The helper's extracted label-loading function found its packaged catalog without
GUI/authentication. Saved connection document SHA-256 is unchanged throughout.

⏳ Dependency inspection found an undeclared optional GUI-helper runtime:
the script imports Python GI, Gtk 3.0, and WebKit2 4.1, but package Depends/
Recommends omit those components. They are installed on this host, which can
mask the issue on a fresh installation. Recorded an explicit follow-up before
release rather than claiming dependency closure. No helper-dependency metadata
change or installation was performed in this inspection.

⏳ Known COSMIC desktop/AppStream validation issues (COSMIC category and
provides/binaries) remained nonfatal by existing recipe policy. dh_dwz reported
no debug_info, and dpkg-shlibdeps reported merged-/usr libc diversions; packaging
completed and declared dependencies were inspected. lintian is unavailable, so
no lintian pass is claimed. Temporary snapshot/control/extracted artifacts remain
under `/tmp/cosmic-mounter-deb.YBPy3u/`; no user files or existing releases were
deleted. The parent package gate remains open for dependency remediation,
explicitly approved installed smoke/lifecycle tests, six-platform hosted builds,
and final 0.5.3 version/release acceptance. No tag/push/publication occurred.

### OneDrive helper dependency remediation and installer — October 12, 2026

✅ Added `python3`, `python3-gi`, `gir1.2-gtk-3.0`, and `gir1.2-webkit2-4.1` to
Debian Recommends, keeping the optional GUI helper out of mandatory storage-mode
dependencies. Added Python 3 to declared build/CI prerequisites because package
tests now run installer/helper checks. Two Debian source contracts guard the
recommendations, default shipping build, target-directory override, helper, and
Fluent catalog. Updated the dependency guide and requirements; README unchanged.

✅ Extended `scripts/install-dependencies.sh` with GUI import capability checks,
`--onedrive-auth-helper` and `--skip-onedrive-auth-helper`. Default/`--all` and
`--onedrive` include the check even if the onedrive engine is already current;
unrelated selections do not install the GUI stack. Successful imports avoid
package-manager calls; missing imports install verified Debian/Fedora/Arch
runtime packages, then recheck. Package-manager failures propagate, and failed
post-install imports report an actionable error. openSUSE/Gentoo give explicit
manual variant/USE-flag guidance without guessed package changes. No GTK window,
authentication, or user-state write is part of the capability probe.

✅ Verified package naming against the official
[Ubuntu 22.04](https://packages.ubuntu.com/jammy/gir1.2-webkit2-4.1),
[Ubuntu 24.04](https://packages.ubuntu.com/noble/gir1.2-webkit2-4.1),
[Fedora](https://packages.fedoraproject.org/pkgs/webkitgtk/webkit2gtk4.1/), and
[Arch](https://archlinux.org/packages/extra/x86_64/webkit2gtk-4.1/) catalogs.
This is package-name/API evidence, not cross-distribution installed acceptance.
The current host passes the GI import probe; an APT simulation resolves the four
packages without installs/upgrades/removals. No real installer or package-manager
installation was run.

✅ Nine installer tests use fake package/engine adapters for default/all,
already-current OneDrive, helper-only/skip/base selections, three package maps,
existing runtime, post-install/import failure, package-manager failure, manual
distributions, and stdin help. Tests caught and fixed the source-guard edge case
for `curl ... | bash`; sourcing remains non-executing for tests. `just test` runs
installer tests, and Debian builds run them plus the three helper tests.
Shell syntax, help, strict Clippy, formatting, localization, YAML/Python CI
dependency, documentation links, and diff checks passed.

✅ Rebuilt the unsigned candidate in `/tmp/cosmic-mounter-deb-helper.roKGLa/`,
then saved artifacts to `target/local-deb-helper-2026-10-12.5hQiO7/`. Packaging
passed all 373 nonignored Rust tests plus nine installer and three helper tests.
The main package's actual Recommends includes all four runtimes, its eight
payload md5sums pass, helper/catalog match source bytes, and the extracted worker
safe probe succeeds. Main SHA-256:
`2e8b8ed3190a2eb559ff5f4d2724cbb6ef4f5f87ba3d2fd39a7bdf61be67b975`.
Marked the inspection-finding subtask `[x]`; retained the earlier candidate as
historical evidence rather than overwriting it. Known COSMIC/merged-/usr metadata
warnings remain unchanged; lintian is unavailable.

⏳ No package was installed/published, no authentication/storage/VPN operation
was requested, and saved connection checksum is unchanged. Metadata remains
0.5.2 for this Pop!_OS 24.04 amd64 test candidate. Installed smoke/upgrade/removal,
six-platform CI, and final 0.5.3 acceptance remain open. Recorded a final Flatpak
helper-runtime import/handoff check: host packages do not supply sandbox libraries.

### Installed Debian smoke preflight — October 12, 2026

✅ User approved the installed-package smoke check. Preflight found no installed
Debian Cloud Mounter package or `/usr/bin` binary; command lookup resolves the
existing user-local native binary instead. Recorded the unchanged saved connection
checksum before installation. APT simulation selects the candidate package;
actual installation has not occurred. No autoremove is authorized or required.

⏳ Noninteractive sudo requires a password. The user must run the scoped APT
installation in their terminal; no password is requested in chat. After it
completes, verify dpkg status, installed assets and explicit `/usr/bin` worker/UI
smoke so the user-local executable cannot mask the result. Keep installed smoke,
upgrade/removal, and final-release gates open until their own receipts exist.

### Targeted failed-upload cache quarantine — October 12, 2026

✅ After the user completed unmounting, verified the BME Assessment mount was
absent, its rclone process stopped, and its user service inactive/dead. Moved only
the previously identified recovered/precomment DOCX cache files and their two
dirty VFS metadata records into the private recovery directory
`/home/uutzinger/.local/state/cosmic-ext-applet-mounter/cache-quarantine/2026-10-12-bme.OHRuWI/`.
The pending uploads are intentionally discarded; copies remain recoverable.

✅ Verified all four original cache paths are absent, the two preserved DOCX
sizes match, and no other dirty metadata entries remain in this mount's cache.
No cloud deletion, general cache clearing, or mount restart was performed.
The user must remount before confirming the live pending count clears. This
cleanup does not diagnose or resolve the separate reported document regressions.

✅ After remount, live read-only RC checks confirmed an empty upload queue,
zero uploads queued/in progress, zero errored cache files, and no out-of-space
condition. The two failed pending entries are cleared. This confirms current
upload-queue health, not whole-library content/version correctness.

### Final native and packaging gate resumed — October 12, 2026

✅ Re-ran final local formatting, locked all-target checking, strict all-feature
Clippy, localization, all 373 nonignored all-feature Rust tests (including socket
tests), nine installer and three helper tests. All passed. The two ignored
coordinator lifecycle integrations passed explicitly with isolated temporary
configuration/data and a loopback SFTP server, including initialization, ordinary
two-way work, unique backups, deletion recovery, and ownership-safe retention.
These exercise the current shared process/coordinator path without production
remotes or credentials. Earlier accepted SharePoint/native UI receipts are not
relabelled as freshly repeated tests.

✅ All 22 project/documentation Markdown files' local link targets resolve.
Removed one trailing space from the user-added future runtime task without
changing its text or runtime decision; diff checks now pass. Added a completed
local-checkpoint subtask while leaving the combined hosted/native/package gate
open. No live storage, VPN, user-service, mount, or production-data operation
was initiated by these tests.

⏳ The Debian package is still not installed according to dpkg preflight; its
smoke/upgrade/removal acceptance remains open. Installation approval already
exists, but noninteractive sudo requires the user's terminal password. Continue
with the saved unsigned candidate and verify explicit `/usr/bin` execution so
the user-local installation cannot mask the test. No installation/tag/push/
workflow dispatch/release publication occurred in this resumed checkpoint.

⏳ Confirmed a release-ordering conflict: `build_release.sh` pushes a `v*` tag,
whose workflow publishes immediately after Debian builds, while the recorded
gate requires final Flatpak acceptance before publication. Added a pre-tag
planning/implementation item to separate source/tag builds from explicitly
approved publication. Workflow behavior was not changed without resolving that
decision; do not push the release tag until publication gating is corrected.

### Installed Debian editor smoke — October 12, 2026

✅ User installed the candidate successfully through APT: one new package, no
upgrades/removals. The `_apt` local-file accessibility notice did not prevent
installation. dpkg reports 0.5.2 installed; dpkg integrity verification passes.
Fresh extraction of the saved candidate matches all six installed application
assets byte-for-byte. The earlier temporary extraction was no longer present;
verification was repeated from the retained artifact, not stale temporary state.
Installed helper GI imports and `/usr/bin` restricted worker probe pass without
authentication/storage work.

✅ Explicit `/usr/bin/cosmic-ext-applet-mounter --settings` opened Add Connection
(not General Settings); the user confirmed detected remotes appeared. Process
identity verifies the Debian executable, not the shadowing user-local binary.
Then opened `/usr/bin/cosmic-ext-applet-mounter --app-settings` for General
Settings. Its D-Bus Settings owner is the packaged process, while the original
user-local applet remains the sole Runtime owner. No competing runtime was started.

✅ Configuration checksum remained identical across both installed-window
launches and remote discovery. It differs from the much earlier package-build
baseline after intervening user mount/config actions; no claim is made that it
was unchanged across that entire interval or that installation caused a change.
No mount/VPN/sync/preferences were modified by this smoke.

⏳ Await the user's General Settings responsiveness/saved-preference/runtime
receipt. Packaged panel launch and upgrade/removal preservation remain distinct
acceptance steps; the current runtime owner is still the user-local build.

✅ User confirmed the Debian General Settings window stayed responsive and
showed expected behavior. The installed editor/General Settings/IPC smoke is
accepted and its UI subtask marked `[x]`. Actual packaged panel identity and
upgrade/removal preservation remain open; those are not inferred from editor
success. Next is a reversible panel launcher switch with the user-local build
preserved and existing storage services untouched.

### Reversible Debian panel switch — October 12, 2026

✅ User authorized a reversible panel switch. Preserved the exact existing
user-local desktop launcher in
`~/.local/state/cosmic-ext-applet-mounter/package-smoke/2026-10-12-panel.YuEmTu/launcher.desktop.before`.
Staged its sole Exec-line change through apply_patch, then deployed
`Exec=/usr/bin/cosmic-ext-applet-mounter`, refreshed the desktop database,
terminated only the verified old user-local applet PID, and reloaded COSMIC panel.
The user-local executable and installed package assets were not changed.

✅ Verified new panel PID 83283 resolves to `/usr/bin/cosmic-ext-applet-mounter`
and owns the sole Runtime D-Bus name. Saved configuration checksum is identical
before/after. The BME Assessment service was already inactive/dead before the
switch and remains inactive (MainPID zero); no mount service was stopped,
restarted, or activated by the switch. Earlier expectations that it was still
active were corrected against the fresh baseline rather than assumed.

⏳ Await the user's popup/connection-state/gear responsiveness receipt before
accepting actual packaged-panel smoke. The launcher override remains in place
for this test; rollback is to restore the saved launcher, refresh desktop
metadata, and reload the panel. Restore it before any future package-removal
test if returning to the preserved user-local installation. Upgrade/removal,
hosted matrix, and final 0.5.3/Flatpak release gates remain open.

### Packaged panel popup latency diagnosis — October 12, 2026

⏳ User reports responsive popup content but a 0.5–0.75-second opening delay.
Kept packaged-panel acceptance open. Source tracing shows `view_popup` builds a
controller snapshot, which reads Online unit and Offline service/timer status
synchronously. `systemd_user_status_blocking` spawns a thread and immediately
joins it, so rendering still waits; each unit establishes its own blocking
D-Bus connection and performs multiple property requests.

✅ A temporary read-only Rust probe used that exact public status function with
the current saved unit set: all 13 serial queries succeeded in 382, 315, and
395 ms across three passes. This establishes a substantial blocking component,
not a complete click-to-first-frame profile or proof that no rendering/config
cost contributes. The probe started/stopped no service and requested no cloud,
mount, sync, or VPN operation. Temporary probe sources are under
`/tmp/cosmic-mounter-popup-timing.IC2hP8/`; no production implementation was edited.

⏳ Recorded the proposed correction under the panel smoke gate: asynchronous
bounded cached unit-state observation, no service-status I/O during rendering,
freshness/unknown-state/operation regression coverage, and rebuilt installed
panel timing acceptance. This retains D-Bus, not a CLI fallback. The prior fixed
host-inspection bridge hardening is separate; it did not remove these remaining
synchronous systemd UI reads. Await implementation direction before changing code.

### Nonblocking popup unit-observation implementation — October 12, 2026

✅ User approved implementation. Popup/controller rendering and pending-count
UI handlers now read cached Online service and Offline service/timer/warning
observations, not blocking systemd queries or coordinator files. Added one
asynchronous D-Bus connection per batch, two-second per-unit/five-second total
deadlines, cancellation, and a bounded off-UI warning-file worker. Startup
prefetch, popup opening, visible five-second refresh, and operation/configuration
reconciliation keep the cache current without closing-popup polling.

✅ Failed/unknown observations retain previous successful state, never fake
inactive; pending/unavailable UI text and disabled primary controls identify
untrusted observations. Fifteen-second freshness, exact saved configuration,
generation and cancellation guards reject obsolete callbacks. Routed/control
requests wait for a fresh observation and reject changed connection identity
instead of applying stale intent. Existing backend safety checks and manual
running-operation overlays remain; synchronous status helpers are retained only
in off-UI backend/safety/test paths. No command fallback or permission was added.

✅ Added eight cached render/failure/timer/manual-overlay/freshness/generation/
identity/cancellation/unknown-not-paused tests and three asynchronous budget tests.
The final Debian package suite passed all 384 nonignored Rust tests; the earlier
all-feature suite passed 382 before the final timer-overlay-only test addition.
Strict all-feature Clippy, formatting, locked all-target checking, diff/link
checks, nine installer and three helper tests pass. Fluent validation reports
1,296 messages and 1,451 references with no missing/duplicate keys. Requirements
and the maintained inventory now state the UI-observation contract; README
and shipping permissions are unchanged.

✅ The same read-only 13-unit probe measured the new asynchronous single-
connection batch at 241, 208, and 226 ms, compared with earlier 382/315/395 ms
serial blocking passes. These reads no longer lie on the rendering path. This
does not certify the installed click-to-first-frame latency; user confirmation
is still required. No service start/stop or production data action was requested
by either status probe.

✅ Built an unsigned current-source candidate in
`/tmp/cosmic-mounter-deb-popup.kjwEvL/` and saved artifacts to
`target/local-deb-popup-2026-10-12.vc4ich/`. All eight payload checksums,
helper/catalog byte comparisons and the extracted fixed worker probe pass.
Main SHA-256:
`0dd5cfc9d2c43f53e81a1e92d6e644ceb4ceff056dd6c4ced684346442835a36`.
Build ID is `7bd3d2952ee843c5880855a48f8660140dcac997`. The new candidate retains
helper runtime recommendations and current 0.5.2 metadata; known packaging
warnings remain unchanged. No installed package was replaced or panel reloaded
by this implementation turn.

⏳ Reinstall the new candidate and recheck actual packaged panel opening/steady
state. The saved configuration's latest checksum differs from the earlier
panel-switch baseline during intervening user activity; no claim of an unchanged
whole-session checksum is made. Source edits/probes initiated no configuration,
storage or VPN changes. Capture a fresh baseline before reinstall/reload.
Installed lifecycle, six-platform CI, final metadata/tag/publication ordering,
and final Flatpak gates remain open; progress is saved.

### Updated popup candidate installed and reloaded — October 12, 2026

✅ User reinstalled the updated local candidate successfully over 0.5.2. dpkg
integrity verification passes and installed build ID matches
`7bd3d2952ee843c5880855a48f8660140dcac997`. The local-file `_apt` notice did not
prevent installation. Verified the old runtime still used the deleted prior
`/usr/bin` image and no stale editor window remained.

✅ Terminated only that verified old Cloud Mounter runtime and reloaded COSMIC
panel under the previously approved reversible launcher override. New PID 131404
resolves to `/usr/bin/cosmic-ext-applet-mounter` and owns the sole Runtime bus
name. Saved configuration checksum is identical before/after reload. The BME
Assessment service remains inactive/MainPID zero, its fresh pre-reload baseline;
no mount service was stopped/restarted by this step. The launcher backup remains
available for rollback to the preserved user-local installation.

⏳ Await updated panel opening-speed and normal popup/gear/slider confirmation.
Technical reinstall/reload is marked `[x]`; visual timing acceptance stays open.
No real sync, authentication, VPN, cloud edit, tag, or publication was initiated.

✅ User confirmed that opening is noticeably faster on the updated packaged
panel. Installed qualitative latency acceptance passes; no exact first-frame
measurement is claimed. Marked that receipt `[x]`. Normal connection-list/slider/
gear confirmation remains before closing the overall packaged-panel smoke item.

### Packaged panel behavior accepted — October 12, 2026

✅ User confirmed the updated panel works as expected after confirming noticeably
faster opening. Connection-list/slider/gear behavior and the installed panel smoke
are marked `[x]`, along with the popup latency fix. This is qualitative UI
acceptance, not a new storage/VPN lifecycle test.

⏳ Kept the overall installed package gate open. Finite offline control
reconciliation on the updated package and installed upgrade/removal preservation
remain explicit separate acceptance items. No configuration, service, storage,
credential, installation, or launcher change was made while recording this receipt.

### Installed Debian offline Start/Pause acceptance — October 12, 2026

✅ User saved the approved disposable `Rust Coordinator Initial Test` connection.
Its generated scheduled worker now resolves to `/usr/bin/cosmic-ext-applet-mounter`
instead of the preserved user-local executable. Before Start, service and timer
were inactive/disabled and MainPID was zero.

✅ User started the connection without errors. Read-only unit inspection confirmed
the timer was active/waiting and enabled; atomic coordinator state recorded
`resume/succeeded` without warnings. No sync worker was running at observation.
User then paused it and confirmed the slider turned off and stayed off. Service
and timer were inactive/dead and disabled, MainPID zero, Result success; atomic
state recorded `pause/succeeded` without warnings. Installed finite Start/Pause
reconciliation is marked `[x]`; this does not claim a new file-transfer test.

⏳ Installed upgrade/removal preservation remains next and requires an explicitly
approved reversible lifecycle test. The disposable mirror is left paused.

### Installed package preservation preflight — October 12, 2026

✅ User approved reversible package removal/reinstallation. Confirmed the rollback
candidate exists with SHA-256
`0dd5cfc9d2c43f53e81a1e92d6e644ceb4ceff056dd6c4ced684346442835a36`.
Package control archive contains only control/md5sums, with no maintainer removal
scripts. Read-only content/mode fingerprint covers 5,328 files across app
configuration/authentication, rclone configuration, generated units, app caches
and state, saved offline mirror/recovery paths, and launcher. Private baseline is
`/tmp/cloud-mounter-preservation-before-20261012.json`; the comparison helper is
`/tmp/cloud-mounter-preservation-check.py`. Online mount paths were not traversed.

⏳ No package removal has occurred. Sudo requires an interactive password, so the
user must run the removal command in a terminal. Use remove, not purge or
autoremove; inspect preservation before reinstalling the retained candidate.

✅ User completed `apt remove cosmic-ext-applet-mounter` (not purge); dpkg confirms
the package is absent. Post-removal fingerprint comparison passed: all 5,328
tracked files retain identical content and modes, with zero added/removed/changed
files and no root-existence changes. Configuration, authentication, generated
units, caches, state, and offline mirror/recovery data covered by the baseline
are preserved. Private result:
`/tmp/cloud-mounter-preservation-after-remove-20261012.json`.

⏳ Reinstall the retained popup candidate, repeat preservation/integrity checks,
then reload the installed panel before closing lifecycle acceptance.

✅ User reinstalled the retained popup candidate successfully. The `_apt` local-
file access notice did not prevent installation. `dpkg -V` passes; installed
build ID is `7bd3d2952ee843c5880855a48f8660140dcac997`. Post-reinstall fingerprint
also passes: all 5,328 files are unchanged against the pre-removal baseline,
including content/modes and root existence. Private result:
`/tmp/cloud-mounter-preservation-after-reinstall-20261012.json`.

✅ Verified old Runtime PID 131404 still used the deleted prior executable;
terminated only that instance and reloaded COSMIC panel under the approved
workflow. New Runtime owner is PID 137244. No storage/VPN action was initiated.

⏳ Await user confirmation of the reinstalled panel's expected connections,
off-state disposable mirror, and responsive popup/settings before marking
installed lifecycle acceptance complete.

### Installed candidate lifecycle accepted — October 12, 2026

✅ User confirmed expected behavior after package reinstall and panel reload.
New Runtime PID 137244 resolves to `/usr/bin/cosmic-ext-applet-mounter`.
Local Debian candidate inspection, installed editor/panel smoke, offline
Start/Pause reconciliation, and approved removal/reinstall preservation are
marked `[x]`. All 5,328 baseline files retained identical contents and modes
after removal and reinstall; package integrity and build identity passed.

⏳ This accepts the local amd64 current-source candidate with 0.5.2 metadata,
including same-version replacement, not an actual cross-version release upgrade.
Six-platform hosted CI, publication-ordering repair, consistent 0.5.3 metadata,
final versioned artifacts, and accepted-source final Flatpak tests remain open.
The reversible panel launcher override and user-local rollback build are retained.

### Release build/publication separation and 0.5.3 preparation — October 12, 2026

✅ Tag pushes and ordinary manual Debian dispatches now build only. Publication
requires a separately requested manual accepted-tag dispatch with
`publish_after_acceptance=true`; branch publication requests fail. Publication
waits for all six matrix builds, serializes by tag, verifies matching Cargo/lock/
Debian/AppStream versions, rejects development AppStream metadata and missing/
draft notes, and uses reviewed notes plus `--verify-tag`. The release orchestrator
downloads to `target/artifacts` and explicitly reports no publication.

✅ Prepared Cargo/lock and Debian 0.5.3 metadata, development AppStream entry,
draft versioned release notes/index, and concise build-guide instructions. Kept
published README download URLs/historical screenshots and the existing final
Flatpak v0.5.2 source pin unchanged. No dependency version changed.

✅ Bash syntax, formatting, diff check, three Debian source contracts, and six
offline fixture preflight tests pass. Actual current-source publication preflight
correctly rejects development metadata. Tests cover finalized success, invalid
tag, lock mismatch, development status, and draft/missing notes without network,
tags, or publication. New tests are wired into native/packaging test recipes.

⏳ Hosted six-platform build/publication paths are not live-tested; acceptance is
a manual authorization gate, not an automatic claim by CI. Final note/AppStream
status and immutable candidate-to-final source identity need reconciliation
before tag creation/final Flatpak acceptance. No installed artifacts, tags,
commits, remote branches, releases, storage, or VPNs were changed by this turn.

✅ Built the local versioned 0.5.3 candidate from a temporary current-source
snapshot. First attempt correctly failed because the snapshot omitted `.github`,
required by the new include_str workflow contract. Added that source directory
and reran successfully; no product-code repair was needed. Package tests pass:
385 nonignored Rust and 18 Python checks. Locked all-target compilation, strict
all-feature Clippy, formatting, localization and workflow YAML parsing pass.
Known nonfatal COSMIC metadata, missing debug-info and merged-/usr warnings remain.

✅ Saved Debian/debug/buildinfo/changes artifacts and provenance README under
`target/local-deb-053-2026-10-12.QlFmb9/`. Main SHA-256:
`4821cc6c176406456b36298dca8a91a78d330ec08998e613c58789a872f9886b`.
Build ID: `a6d63c5f420db4ca17526b999d1284dc87d6b8e0`.
All eight extracted payload checksums pass, helper/catalog match source, and
the fixed synthetic-UUID worker probe succeeds without configuration/engine work.
No installation or publication occurred. Actual versioned upgrade acceptance
remains next; prior same-version preservation remains separate evidence.

### Versioned Debian upgrade preflight — October 12, 2026

✅ User approved the actual local 0.5.2-to-0.5.3 upgrade test. Confirmed installed
version is 0.5.2 and retained candidate is 0.5.3/amd64 with the recorded SHA-256.
Fresh preservation baseline covers 5,328 files with no absent roots:
`/tmp/cloud-mounter-upgrade-053-before-20261012.json`. No installation has occurred
at this preflight; user will run APT in a terminal for password authentication.

✅ User completed the actual 0.5.2-to-0.5.3 APT upgrade. dpkg reports installed
0.5.3; integrity verification passes and build ID matches
`a6d63c5f420db4ca17526b999d1284dc87d6b8e0`. The local-file `_apt` notice was
nonfatal. Post-upgrade preservation comparison passes: all 5,328 files retain
identical content/modes, with no added/removed files or root-existence changes.
Private result: `/tmp/cloud-mounter-upgrade-053-after-20261012.json`.

✅ Verified old Runtime PID 137244 used the deleted prior image; stopped only
that applet and reloaded COSMIC panel through the approved workflow. New Runtime
owner is PID 147124. No mounting, synchronization, saving, or VPN action was
initiated. Await user confirmation of expected responsive panel/settings behavior
before completing the versioned upgrade item.

✅ User confirmed expected behavior after the installed 0.5.3 panel reload.
New Runtime PID 147124 resolves to `/usr/bin/cosmic-ext-applet-mounter`.
Actual local amd64 0.5.2-to-0.5.3 upgrade preservation and installed UI smoke
are accepted and marked `[x]`. Configuration, credentials, units, cache/state,
and offline mirror/recovery files in the 5,328-file baseline were unchanged by
the upgrade. This does not close hosted six-platform or final Flatpak acceptance.

### Source/worktree finalization review started — October 12, 2026

✅ Inspected release gates, workflow/publication guard, Cargo/lock/version delta,
source fixture references and worktree inclusion. Initial scope is 55 changed/
deleted tracked paths and 28 untracked files. New source/test modules and moved
guides must be included, not omitted by a tracked-only commit. Filename-based
private-artifact scan found no matches; this is not a complete secret audit.
Diff check, release shell syntax and six offline preflight tests pass.

✅ Corrected draft notes that still described the actual native upgrade as
pending. Created the maintained release-source audit/index entry and recorded
the proposed immutable candidate/prerelease-to-final sequence, resolving the
planning conflict without pretending draft metadata can pass final publication.

⏳ User can run the latest all-feature test gate in their terminal and retain the
output under target to reduce tool/chat overhead. Full source/content/reference
review and immutable source approval remain open. No commit, stage, tag, push,
publication, installed-artifact change or storage operation was performed.

✅ User ran `cargo test --locked --all-targets --all-features` in the host terminal
and reported exit status zero. Reviewed `target/0.5.3-all-features.log`: 385 tests
passed, zero failed, 12 ignored, with no error/warning lines. Latest all-feature
source gate is marked `[x]`; ignored integration tests remain separate evidence
and are not converted to passing status. Source/content/reference review and
immutable candidate/final approval remain outstanding.

✅ Completed focused content/reference and architectural-boundary review across
114 existing tracked/nonignored-untracked source files. Signature scan found two
synthetic test-password literals and no confirmed credentials. Binary contents,
remote history and unknown secret formats were not certified. Local inline
Markdown file targets and heading anchors resolve; moved 0.5.0/0.5.1 notes are
byte-identical to their historical HEAD originals. No external URL availability
or complete line-by-line/security review is claimed.

✅ Inspected production worker/host-inspection entry points and cfg boundaries
around legacy CLI test adapters. Required untracked modules/tests/guides remain
intended source; package/log outputs remain ignored. Retained organization-specific
test/history context is recorded as a privacy consideration, not silently erased.
Private report: `/tmp/cloud-mounter-source-review-20261012.json`. No runtime code,
installed artifacts or storage changed; diff check passes.

⏳ Next: intended staged-diff/source-inclusion review and explicit immutable
candidate route approval, then separately authorized commit/tag/push. No source
has been staged or committed by this review.

### Staged source inclusion/candidate comparison — October 12, 2026

✅ User staged the repo. Reviewed staged path status/renames: required new source,
tests, guides and engineering records are included; no ignored package/cache/log
artifacts are staged. Compared 88 runtime/build/test/packaging files from the
index to the tested local 0.5.3 source snapshot: zero byte differences. Before
review-receipt edits, index and working tree had no unstaged tracked differences.

✅ Cached diff check found three trailing-space Markdown hard-break lines in
the newly staged host inventory. Replaced them with blank-line separation;
documentation-only correction does not invalidate native runtime acceptance.
Staged inclusion/source-consistency item is marked `[x]`, not full code/security
review or immutable release-source approval.

⏳ Restage the inventory fix and Task List/completion/audit receipts, then run
`git diff --cached --check`. Commit, candidate route/tag and push still require
separate approval; none was performed here.

### Local candidate source committed — October 12, 2026

✅ User restaged documentation and approved the next local commit stage. Final
cached whitespace check passed; there were no unstaged tracked changes. Created
`efbf73e` (`Prepare 0.5.3 D-Bus integration and synchronization candidate`):
80 intended changed/new/renamed paths, with required source/tests/docs included
and no package outputs. Runtime/build source remains the tested native candidate.
This freezes candidate source, not final Flatpak/release acceptance.

⏳ No tag, push or publication was performed. Draft notes/development AppStream
and final Flatpak source-pin/acceptance gates remain open. Record this receipt
in a documentation-only follow-up commit; the implementation candidate stays
immutable rather than being amended.

### Immutable local candidate Flatpak build prepared — October 12, 2026

✅ Continued the task-list candidate-to-final gate. Prepared a build-only manifest
under `target/flatpak-candidate-053-efbf73e.hxxZ6G/` using the local Git repository
and full committed source identity `efbf73eae0e596d56f2aa950dca37afb91337947`.
Only app source URL/tag-to-commit selector differ from the reviewed source
manifest; generated Cargo sources, permissions/runtime/base are unchanged.
This does not use copied native binaries or require a push/tag. The final manifest
remains pinned to v0.5.2 pending final acceptance/tag authorization.

✅ Verified installed BaseApp stable uses Freedesktop 25.08, with installed SDK
and Rust extension 25.08. Exact commits/tool version are saved in the build
directory README. No runtime substitution, installation or applet stop occurred.

⏳ User can run the candidate build in their terminal with output saved under
target. Build excludes --install; installed native/Flatpak artifacts stay intact.
Candidate source/permission/helper/worker inspection and controlled live tests
precede final-tag build/acceptance; no final package gate is marked complete.

### Candidate Flatpak offline vendor identity repair — October 12, 2026

✅ User build returned exit status 1. Log identifies failure before compilation:
generated Cargo configuration qualified libcosmic with rev a991d7bb while the
committed manifest/lock identify the unqualified Git source. Cargo did not match
the vendored replacement and attempted a forbidden offline checkout. The correct
commit was downloaded; the failure was source-selector configuration, not a
missing runtime or a reason to enable online Cargo fetching.

✅ Regenerated sources from unchanged Cargo.lock through the existing recipe.
Managed runtime-directory access blocked the first invocation; authorized host
rerun succeeded. Git dependency revisions and registry/download URLs are unchanged.
Updated the build-only candidate's external generated sources. Added a regression
contract comparing every locked Git source selector and pinned checkout with the
generated replacements. Three Flatpak contract tests, formatting and diff check
pass. No app runtime code, runtime/base selection or permissions changed.

⏳ Retry build to a separate build-retry.log, retaining original failure evidence.
The regenerated vendor input/test are currently uncommitted packaging changes;
candidate app source stays efbf73e, so the overlay must be included in the final
immutable source before final acceptance. No install, tag, push or publication.

✅ User retry returned exit status zero. Reviewed saved log: exact efbf73e checkout,
offline locked 0.5.3 compilation, successful export. Candidate OSTree commit:
`0b6b66e778a5d62365e3b52b30f39b6a5fe4c3b6fa2412d710d81ac0996da88d`.
Binary SHA-256:
`5e820735d6799c6dbecd66ec631ab262a6fcb5b6f53bac529172963aff1072bd`;
build ID `84338a68c3df76f11a10bfb1e1d9b6380f1da79c`.
Refreshed vendor-input SHA-256:
`0c098520c9e8933773ddd935afaa3cb109a78f137593ab9d7ac0ac1693b8815f`.
Metadata retains BaseApp stable/Platform 25.08 and reviewed narrow grants.
Known COSMIC desktop-category warning and BaseApp non-app-ID icon export skips
remain; neither prevented export. No install or runtime switch occurred.

⏳ Next pre-install checks must use Platform, not SDK: fixed synthetic worker
probe and Python GI/Gtk 3/WebKit2 4.1 import capabilities. These checks do not
establish live mirror path access, authentication or final installed permissions.

✅ User ran the fixed synthetic worker probe with Platform/readonly build flags;
output matches `bisync-worker-probe-ok` and the supplied synthetic UUID. Worker
entry point passes without storage/configuration work or competing Runtime owner.

⏳ User Platform Python import check fails immediately with
`ModuleNotFoundError: No module named 'gi'`. GTK/WebKit imports and actual helper
sign-in are not verified. Host GI packages do not supply sandbox libraries.
Source inspection finds `open_onedrive_auth_url` selects a helper by file presence
and reports success after spawn, without validating helper capability/exit; a
packaged but unusable helper therefore cannot count as a working fallback.
Existing manual auth-files/browser/response-URL UI is available, but a deliberate
Flatpak policy and launcher handling still need implementation/acceptance.
Proposed minimal route: explicit manual browser handoff for this Flatpak runtime,
retaining optional graphical helper for native installations. Await user choice
before adding sandbox dependencies or changing authentication policy.

### Explicit Flatpak OneDrive browser fallback implemented — October 12, 2026

✅ User approved manual browser/response-URL fallback. Flatpak default setup now
selects manual auth-files; browser opening uses an asynchronous 10-second bounded
OpenURI portal request and explicitly reports request acceptance, not sign-in
success. It never selects the installed-but-unusable GI helper. Native setup
retains interactive authorization and its optional graphical helper. Late launch
completion does not replace another connection draft's notice.

✅ Transient handoff paths are connection-scoped under private app-owned shared
state, not sandbox /tmp that the host engine cannot see. Preparation and response
writing enforce private directory permissions; normal setup cleanup removes URL/
response files. Added four policy/path/privacy/invalid-URL/stale-draft regressions.
The initial regression compilation exposed a missing trait qualification and
the privacy test caught directory chmod missing at preparation; corrected both.
All 28 focused OneDrive tests, three helper localization tests, compilation,
strict all-feature Clippy, formatting and localization now pass. Catalog has
1,301 messages and 1,455 references, with no missing/duplicate keys.

✅ Requirements, draft notes and packaging guidance describe the runtime-specific
manual fallback. No host-spawn exception, runtime switch, extra permission,
installed-package replacement, real authentication, storage or remote write.

⏳ Latest broad tests and an approved new immutable candidate commit/rebuild are
required: the existing efbf73e Flatpak does not contain these edits. Sandbox
browser/redirect/cleanup/cancellation acceptance remains open; source tests do
not establish that actual sign-in succeeded.

### Browser-fallback rebuild prepared — October 12, 2026

✅ User recalled that browser redirect capture was difficult during prior manual
tests and may require history; recorded this as an unresolved usability limitation,
not something the portal request or source tests solve. User requested rebuilding.
Prepared a frozen current-source archive including the uncommitted fallback/vendor
delta, without implicitly authorizing a new commit/tag. Candidate build directory:
`target/flatpak-candidate-053-browser.pzKAYd/`; archive SHA-256:
`e24dabffac3f3a16635e25d27bddde157a7bd6f88b201d4109a8702e4a59b259`.
Manifest only substitutes this checksummed source; runtime/base/grants unchanged.

✅ Latest all-feature tests initially hit the managed loopback socket restriction.
Authorized host rerun passed 390 tests, zero failed, 12 ignored. Saved full logs:
`target/0.5.3-auth-fallback-tests.log` and
`target/0.5.3-auth-fallback-tests-host.log`. Prior focused/lint/localization receipts
remain; ignored live tests are not passing tests.

⏳ Build frozen development snapshot without installation, then assess actual
browser/redirect usability. This snapshot is not a final immutable-tag acceptance
artifact; approved commit/repin and final package identity checks still remain.
No installed applet/storage/authentication, tag or remote was changed.

✅ User browser-fallback build returned exit status zero. Reviewed locked offline
0.5.3 compilation/export log; archive checksum matches frozen snapshot. Packaged
Fluent catalog contains the browser-request/manual-handoff labels. Exported
metadata is byte-identical to the preceding candidate: accepted BaseApp/Platform
and reviewed permissions are unchanged. Export commit:
`d0f101c60a1e4fb78700184e3dac11c2544904989c3e171ec12d73e07d7138d7`.
Binary SHA-256:
`d28460f3683a23348c369f54eb73500b6d96939644a84a580c18ce5a0bdf27d0`.
Known COSMIC desktop-category export warning remains nonfatal.

⏳ Candidate is not installed. Current installed Flatpak commit remains
`9896da0ac99a235dd21675ed170d11810c06c1c5c795cf369b72ce6cfab555bf`.
Next needs explicit controlled installation/runtime-switch approval and rollback
preparation; do not run native/Flatpak owners together against shared state.
Browser/redirect usability and real authentication remain unverified. No automatic
redirect capture is claimed; no package/runtime/service change was made here.

### Candidate Flatpak installed and runtime switched — October 12, 2026

✅ User approved rollback preparation, candidate installation and temporary runtime
switch. Saved prior Flatpak as
`target/flatpak-candidate-053-browser.pzKAYd/rollback-previous.flatpak`; validated
bundle by offline delta application into a temporary repository and confirmed exact
prior commit 9896da0. An unavailable `flatpak build-import` command was replaced by
OSTree apply-offline; managed O_TMPFILE restriction required authorized host rerun.
No existing repository/object was deleted or pruned.

✅ Installed bundle candidate d0f101c under the existing user Flatpak ID; Debian
installation is untouched. Fresh preservation comparison confirms all 5,328
baseline files retain contents/modes after installation. Effective grants match
candidate metadata plus four existing global read-only theme grants (GTK 3/4,
kdeglobals and color-schemes). No app override or blanket host grant was added.

✅ Saved exact native launcher as
`target/flatpak-candidate-053-browser.pzKAYd/native-launcher-before.desktop`.
Deployed reversible Exec=`/usr/bin/flatpak run --branch=master ...`, stopped only
verified native PID 147124 and reloaded panel. Sole applet process is sandboxed
PID 195586; host Runtime owner PID 195561 is its xdg-dbus-proxy, whose sandbox
identity matches d0f101c. No native applet remains. Saved configuration hash is
unchanged after switch; Cloud Mounter storage units remain inactive.

⏳ Await panel/settings UI receipt before disposable OneDrive browser handoff.
No authentication/mount/sync started. For rollback, stop the app-specific Flatpak,
restore the saved native launcher and reload panel; reinstall the retained prior
Flatpak bundle if needed. User-local and Debian rollback builds remain intact.
Candidate tests do not establish final immutable-tag/package acceptance.

### Flatpak OneDrive positive browser handoff — October 13, 2026

✅ User created disposable `Flatpak OneDrive Auth Test` draft, started setup and
opened browser from the separate sign-in action. Personal-account sign-in briefly
showed the redirect and then navigated away. User recovered the preceding redirect
URL from browser history, submitted it only to the applet and obtained completed
authorization plus dry-run validation. This demonstrates the sandbox/host shared
auth-file path and browser portal route; it does not show automatic redirect
capture or initial synchronization. No credential/redirect value was recorded.

✅ Read-only verification found the shared auth directory mode 0700 and no
remaining transient files. Documented history workaround and credential handling.
Corrected stale helper-specific start-notice wording in source; installed candidate
still has the old notice until rebuilt. In Flatpak, Start Setup selects the same
manual flow as Use Manual Auth Handoff; native default remains interactive.

⏳ Negative authentication/cancellation and disposable saved-unit checks remain.
No Preview/Sync Now/background Start is required to accept sign-in. Commit/repin
and final-source artifact gates remain open, as does fast-redirect usability.

### Graphical helper packaging follow-up planned — October 13, 2026

✅ Added the requested subtask to the existing OneDrive authentication block:
package/validate Python GI, GTK 3 and WebKitGTK 4.1 inside the accepted Flatpak;
prove automatic redirect capture and negative/cleanup behavior; restore a
preferred graphical path with one explicit manual fallback; reconcile UI/docs/
specifications and affected final-artifact checks. Manual browser-history success
remains fallback evidence and does not complete graphical authentication. Final
Flatpak gate now references that subtask. No implementation/install change here.

### Graphical helper dependency inventory started — October 13, 2026

✅ Inspected accepted Platform (not host): Python 3.13.15, GTK 3.24.52 runtime/
typelibs, GObject typelibs and GI repository 1.0/2.0 are present. PyGObject gi and
WebKitGTK/JavaScriptCore libraries/typelibs are absent. SDK 25.08 provides GLib/GI
2.84.4, introspection 1.86, libsoup 3.6.6, libseccomp 2.6.1, GStreamer 1.26.11,
libsecret 0.21.7 and cmake/ninja/meson. WPE/backend and enchant are not detected.
Python imports search /app/lib/python3.13/site-packages, so app-local GI packaging
is possible without exposing host Python libraries.

⏳ Packaging investigation is in progress: reuse runtime GTK/GI, add pinned
PyGObject and WebKitGTK with only required supporting modules, review security/
licenses/size/maintenance and test actual Platform imports. No SDK/runtime/base
upgrade is justified by this missing-library result. Compatibility of the chosen
WebKit source/options still needs build verification; no module/version acceptance
or installed-app change is claimed. Upstream PyGObject packaging guide and
WebKitGTK release/checksum listings are being consulted, not an old distro binary.

### WebKitGTK SDK configure probe — October 13, 2026

✅ Downloaded WebKitGTK 2.54.1 and verified the upstream published SHA-256;
GTK3/4.1 configure probe uses the accepted SDK, two-job temporary tooling and
readonly candidate app directory. Managed network access required host rerun;
no SDK/runtime/base upgrade or installed app changes occurred. Logs/sources are
under `target/webkitgtk-2541-compat-20261013/`.

✅ First configure stopped at missing Ruby. Actual Flathub extension lookup
found no Ruby 25.08 ref. Downloaded checksum-verified official Ruby 3.4.11 source
and built miniruby in the temporary SDK probe directory. Resumed configuration
passes compiler/core-library checks; optional libbacktrace was missing and its
diagnostic feature was disabled for the next probe. Browser sandbox stayed ON.

⏳ Next configure stops at missing bwrap. SDK inspection confirms both bwrap and
xdg-dbus-proxy absent, while libseccomp/libcap are available. Need reviewed actual
runtime sandbox-tool modules and build-only Ruby before complete configure/full
WebKit build. Minimal interpreter success does not establish all Ruby generators.
No newer SDK requirement has been found; full compatibility remains unverified.
Current logs: configure.log, ruby-probe.log, configure-ruby.log and
configure-no-backtrace.log. Do not disable WebKit sandboxing or bind host tools
as a shortcut to final packaging acceptance.

### WebKit configure succeeds on SDK 25.08 — October 13, 2026

✅ Added candidate bubblewrap 0.13.0 and xdg-dbus-proxy 0.1.9 Meson modules with
official release asset digests and upstream license payloads. Actual source-built
tools compile in SDK 25.08 and load for version probes in Platform 25.08. Built
only into the temporary tools prefix, not installed app/host; about 178 KiB combined
executables. Module source contract plus existing Flatpak contracts pass (four).
Upstream tool tests were disabled, not counted as passing integration tests.

✅ WebKit configure passed sandbox checks with these tools, then found missing
system unifdef and sysprof-capture. Used the source's bundled alternatives as its
CMake branches recommend. Final configure/generation succeeds with sandbox and
introspection ON, GTK3 API, libbacktrace OFF and the accepted SDK unchanged.
Latest log: `target/webkitgtk-2541-compat-20261013/configure-bundled-sysprof.log`.
There is no configure evidence requiring WPE modules or a newer SDK for this path.

⏳ Prepared `build-webkit-probe.sh` under the probe output directory: finish the
temporary full Ruby generator installation, then compile WebKit with two jobs.
It does not install WebKit into /app. Full compiler/runtime/helper/nested-sandbox
and automatic redirect tests remain; final module integration uses /app paths
and strips build-only Ruby. Candidate installed app and user data were untouched.

### OneDrive engine/auth alternatives investigated — October 13, 2026

✅ At user's request, read prior Task List/completion OneDrive auth/lifecycle
sections before recommending alternatives. Local callback/device-flow research,
--reauth integration, WebKit helper and manual-history workaround were already
recorded; this is not a proposal to blindly repeat those tests. Versioned upstream
2.5.11 docs/source show --auth-files bypasses automatic desktop loopback auth;
current Flatpak default deliberately uses that branch. Host desktop script could
invoke existing complete authorization instead of merely emitting a URL, but
this distinct path has not received a fresh live receipt.

✅ Confirmed installed host versions (onedrive 2.5.11 package, rclone 1.75.0) and
source integration gaps: Provider::OneDrive is explicitly rejected by existing
rclone backend/bisync/coordinator, with no persisted offline-engine selector.
Rclone OneDrive support is an achievable adaptation, not a configuration-only
change. Device-code/personal-account limits and tenant admin consent remain
independent of browser UI; no approval bypass is proposed.

⏳ Recorded proposed decision in release-source audit: defer WebKit packaging,
reuse rclone coordinator for OneDrive offline mirrors while preserving optional
abraunegg monitoring/external host auth and existing configs. Await user decision
before altering requirements, active tasks or engine routing. No cloud/auth/data
operation, build stop, install, source implementation or module deletion occurred.

### Three-engine architecture reconsidered — October 13, 2026

✅ At user's request, assessed whether to retain abraunegg and jstaf before any
terminal-auth test. Read current provider/planner/config routing and earlier native/
Flatpak receipts; consulted primary upstream capability documentation. Onedriver
has on-demand/cached read-only offline behavior and existing corporate acceptance;
rclone VFS overlaps but equivalence is not yet tested. Abraunegg monitor mode's
local event/remote signal behavior is a distinct capability that finite scheduled
bisync does not replace. Rclone's OneDrive mount polling is not offline monitoring.

⏳ Proposed rclone-only target if scheduled mirrors meet product requirements,
not perpetual optional-engine support merely to postpone the decision. Requires
disposable personal/business Online/Offline acceptance and explicit migration
preserving legacy units/data/credentials/pending uploads. Documented maintenance,
permission/review and CPU/power boundaries in source audit. No terminal auth,
implementation, migration, installed change or deletion took place.

### Native 0.5.3 / deferred Flatpak 0.5.4 scope adopted — October 13, 2026

✅ User directed native source/Debian 0.5.3 completion and Git push, deferring
final Flatpak packaging and engine-retention decision to 0.5.4. Verified source:
native default is engine-interactive --reauth; manual handoff can launch installed
WebKit helper. Only Flatpak defaults to manual copy/paste. Host helper imports
pass. No native auth reversion or engine/data conversion is needed for this scope.

✅ Consolidated remaining sandbox tasks under new chronological 0.5.4 release
section before future tasks; prior unfinished steps use [o] with transfer links,
while [x] evidence remains intact. New list starts with engine decision, preserves
legacy migration safety, and covers conditional terminal/WebKit options, final
source/runtime/permission/auth/worker/repair/VPN/clean-profile/submission gates.
Updated Description/Requirements, concise README, draft notes and build/publication
guidance to native-only 0.5.3. Final Flatpak no longer blocks native publication.

⏳ Latest native source/package rebuild and installed helper smoke, restoration
of native panel, source review and fresh six-platform Debian CI remain. User
authorized completed-source commit/push; tag/publication is still separate. No
installed package, launcher, storage, credentials, remote or engine changed while
recording this decision. AppStream/notes remain provisional until native gates.

✅ Latest native source gate passes 391 Rust tests (12 ignored remain separate),
18 Python checks, strict all-feature Clippy, formatting, diff and local inline
link/anchor checks. Source release build completed and just install-user installed
the matching user-local bytes. Restricted synthetic worker probe passes; native
Runtime PID 256850 resolves to that source binary, not Flatpak. User confirmed
expected responsive popup/settings. Build ID: 4a16966949ae6389e13d9d70b41a84dc02d4b043.

✅ Source-install rollback copies retain six prior user-local assets under
target/native-053-source-backup-20261013.FqvmDS/. Fresh 5,331-file preservation
comparison found only authorized launcher replacement and four regenerated
Online service files with native ExecCondition helper paths. Config/credentials/
cache/mirror data were unchanged and services remained inactive. The old Flatpak
was already not running at the switch; no unrelated processes were stopped.

✅ Executed installed native graphical WebKit helper with a synthetic callback,
not a real login. It captured the expected synthetic URI, wrote mode 0600 and
exited cleanly; removed both synthetic fixture files afterward. This proves the
native helper path remains functional without reauthorizing a user account.
No actual Microsoft sign-in or synchronization is claimed for that probe.

⏳ Building latest native Debian candidate from complete snapshot
/tmp/cloud-mounter-native053-deb.1QENj3/source; log is build.log there. Source/UI
closure progressed; package inspection/installed acceptance, final metadata,
commit/push and hosted CI remain. No release tag/publication has occurred.

✅ Latest native Debian build succeeded with 391 passing Rust tests and 18 Python
checks. Saved unsigned Debian/debug/buildinfo/changes artifacts under
target/local-deb-native053-20261013.N16OSk/. Main package SHA-256:
c869497bb3d63a7aeddd83daf0d1da2206c93ab4b490ca3cc847e1b1a012f762.
Build ID: 6ce0047816ce44c8642be90b1f11314f8bcfb561. All eight payload checksums,
source helper/catalog comparisons and extracted fixed worker probe pass. Known
nonfatal packaging warnings remain. AppStream/notes remain provisional until
native installed package acceptance/final metadata closure; no release published.

⏳ Prepared fresh preservation baseline for latest Debian installation. User
terminal APT is needed for sudo password. User-local source Runtime is currently
active and preserved for rollback. Package installation itself will not replace
that running source instance; controlled packaged-panel verification follows.

### Latest native Debian candidate installed — October 13, 2026

✅ User APT reinstall completed. Installed 0.5.3 integrity passes and build ID is
6ce0047816ce44c8642be90b1f11314f8bcfb561. The local-file _apt permission notice
is nonfatal. All 5,331 baseline files retained identical content/modes after APT;
no config/credential/cache/mirror data was changed by installation.

✅ No Settings bus owner remained. Saved native source launcher as
target/local-deb-native053-20261013.N16OSk/source-launcher-before.desktop, deployed
reversible Exec=/usr/bin override, verified/stopped only source Runtime PID 256850
and reloaded COSMIC panel. New Runtime PID 264223 is native /usr/bin. Configuration
hash remains unchanged after reload and storage units remain inactive. User-local
source build, Flatpak rollback and all existing data are retained.

⏳ Await packaged popup/settings receipt, then finalize native metadata/source
review and authorized Git push/hosted Debian gate. No tag/publication occurred.

✅ User confirmed final packaged popup/settings behavior. Installed native latest
source/Debian acceptance is complete; marked [x] separately from hosted CI and
release publication. Prepared native-only notes and stable 0.5.3 AppStream entry
dated October 13. Runtime code did not change in metadata closure. Tested local
package predates that metadata-only edit; final hosted/tag identities remain
separate, not silently equated to the local candidate.

⏳ Proceed with authorized reviewed-source commit/push and fresh publication-
disabled six-platform Debian workflow. Cleanup item added by user remains open:
exact stale units/folders need inventory and approval before any material removal.
No cleanup, tag or release publication is implicit in this push.

✅ Native metadata/notes preflight passes; six publication fixture tests pass.
Fresh intended-source scan finds only the same two synthetic test-password
matches, no confirmed credential, and no unresolved local inline links/anchors.
Retained organization-specific history remains explicitly acknowledged. No final
hosted/tag checksum is equated to the earlier local candidate. Preparing the
authorized commit/push and publication-disabled matrix; cleanup remains open.

✅ Committed reviewed source as 95fa290 and pushed master successfully, including
earlier candidate commits. No release tag or GitHub release was created. Existing
Git GC/unreachable-object warning was left untouched; no prune/destructive cleanup.

✅ Dispatched fresh Build Debian packages workflow with
publish_after_acceptance=false. Run 38438465174 builds exact source
95fa290ebd6bc3fcdad83870e503d67104bf6537 and is in progress. Six-job completion/
artifact inspection remain pending; notes/publication status must not claim these
passed. Versioned docs/releases/v0.5.3.md supplies future GitHub release body;
debian/changelog separately records package changes.

✅ Hosted run 38438465174 completed successfully: Ubuntu 22.04/24.04/26.04,
amd64/arm64 build/test/upload jobs all passed for exact source 95fa290; publication
job was skipped. Downloaded all six artifacts and verified per-job SHA-256,
package/version/architecture, eight extracted payload checksums each, source-
matching helper/catalog and stable 0.5.3 AppStream metadata. No artifact was
installed; six builds are not six-platform live UI/storage acceptance.

✅ Organized verified release candidates at target/artifacts/v0.5.3/ with combined
SHA256SUMS and provenance README; checksum verification passes for all six there.
Original run extraction/inspection remains under
target/artifacts/0.5.3-ci-38438465174.kHHWVi/ pending deliberate artifact cleanup.
Root artifacts/v0.5.2 contains the six older workflow packages, not copies of
current 0.5.3. Other local test candidates have distinct build identities and
must not be treated as interchangeable duplicates. No package removal occurred.

⏳ Standardize old release/candidate/rollback artifact locations and fix the
standalone download helper's legacy artifacts/deb default if requested. Preserve
the historical v0.5.2 set before removing its old folder. Stale services/folders,
receipt commit/push and separately authorized tag/publication remain outstanding.

### Candidate Flatpak UI receipt — October 13, 2026

✅ User confirmed candidate panel/settings responsiveness. Marked the UI receipt
`[x]`; browser sign-in, redirect capture, shared auth-file access and cancellation
are separate pending acceptance checks. The former `OneDrive Mirror Repair Test`
name is absent from the current saved configuration, so do not silently reuse
another production OneDrive profile for reauthentication.

### Debian artifact layout cleanup — October 13, 2026

✅ Verified the user-moved target/artifacts/v0.5.2/ set: all six SHA-256
checks pass. Canonical v0.5.3 checks also pass. Moved each directory below
with full file-by-file SHA-256 parity, retaining metadata and rollback files:

- target/local-deb-2026-10-12.LBoZhW/
- target/local-deb-helper-2026-10-12.5hQiO7/
- target/local-deb-popup-2026-10-12.vc4ich/
- target/local-deb-053-2026-10-12.QlFmb9/
- target/local-deb-native053-20261013.N16OSk/

Their new parent is target/artifacts/candidates/; the directory names remain
unchanged. Historical commands above retain their original execution paths.
Moved target/release-files/v0.5.0/ to target/artifacts/v0.5.0/ and
target/release-assets/0.4.5/ to target/artifacts/v0.4.5/ with byte parity.

✅ Removed exactly six checksum-identical .deb downloads from
target/artifacts/0.5.3-ci-38438465174.kHHWVi/deb-*/; canonical copies remain
in target/artifacts/v0.5.3/ and can also be downloaded again from that workflow.
Retained per-job sums and extracted inspection evidence, with a relocation
README. Removed the empty root artifacts/ directory. No unique build, Flatpak
rollback bundle, installed artifact, service, configuration or user data changed.

✅ The workflow helper defaults to target/artifacts/ci-<run-id>/, retaining
environment and CLI destination overrides. Bash syntax, three offline fake-gh
destination cases and git diff --check pass; no real workflow dispatched.
AGENTS.md and the build guide now document the layout. All .deb/.ddeb files
inside this repository are under target/artifacts/.

⏳ Three older .deb files in the parent ~/Build directory (0.4.7, 0.5.0,
0.5.1) remain outside this workspace and were not moved or deleted. Release
tag/publication and stale mount-service/data cleanup remain separate gates.

### Accepted native identity and cleanup review — October 13, 2026

✅ Rechecked installed Debian version 0.5.3. Installed binary SHA-256 is
`0fbf8b77eb8a7556564f07e1c7e16d51ca2a79ee2f25950c52942f5edfea73bb`,
matching the extracted accepted local package; ELF build ID is
`6ce0047816ce44c8642be90b1f11314f8bcfb561`. Accepted package at
target/artifacts/candidates/local-deb-native053-20261013.N16OSk/ has SHA-256
`c869497bb3d63a7aeddd83daf0d1da2206c93ab4b490ca3cc847e1b1a012f762`.
Source-installed rollback binary SHA-256 is
`a86554dafbc2268710fae4721b1e1d52ac5e89b34e5cf2fac26d77b65d3ca6ab`,
build ID `4a16966949ae6389e13d9d70b41a84dc02d4b043`. Different build modes
have distinct identities; the source and packaged UI receipts remain separate.

✅ Reverified all six canonical branch-CI checksums for source
95fa290ebd6bc3fcdad83870e503d67104bf6537, workflow 38438465174:

| Package suffix (all version 0.5.3) | SHA-256 |
|---|---|
| ubuntu22.04_amd64.deb | `588b7a63428601186fbe6a41c6c2fa42fd189dd8e2883e7a5ae4e92951615e86` |
| ubuntu22.04_arm64.deb | `6f96b031ea053712749981d16fdd1bdb71197d46ea19a8c13feda8da9e9eb4af` |
| ubuntu24.04_amd64.deb | `5d6a1178822c9bf4c30cd881fd06b19f97aa36e24b035515120f04a5cfbc5da2` |
| ubuntu24.04_arm64.deb | `2edc4a921da2c3c37a43c46e7d7139b2c96eebea8305a794833585e85aaae693` |
| ubuntu26.04_amd64.deb | `277ecdc048d8b3b8639757bde6e6796507d6fababfb26e68d29c4d6c1b735e3c` |
| ubuntu26.04_arm64.deb | `15a827e52cde0a1e0b9f5c9799bb32393e498dbb53848b9b9d505483c7722196` |

All filenames begin cosmic-ext-applet-mounter_0.5.3_. Native live acceptance
remains Pop!_OS 24.04 amd64; no six-platform live UI claim. These are retained
branch-CI acceptance artifacts, not yet tagged/public release identities.
No final Flatpak acceptance or COSMIC Store submission claim for 0.5.3.

⏳ Requested separate approval for v0.5.3 tag/push and native GitHub publication
after final-record commit and fresh tagged matrix verification. No approval
reply, tag creation or publication occurred during this inventory.

✅ Completed read-only cleanup review; see
docs/engineering/development-cleanup-inventory.md. Managed-sandbox D-Bus access
was denied; reran read-only systemd queries with authorized host access.
UA OneDrive is currently enabled/running and mounted; leave it untouched.
All 11 managed service identities have saved owners, including the two disabled
SharePoint test profiles. Four disabled legacy rclone units are possible archive
candidates, not proven orphans. Two prior OneDrive test folders contain no files;
older repair authentication/database state remains and must not be pruned by
folder-name inference. Shared recovery contains saved files and is retained.

⏳ Cleanup execution awaits explicit test-profile retirement/legacy-unit archive
scope. No service stopped, profile altered, folder deleted, authentication state
removed, or remote data modified. Recommend retaining Rust Coordinator Initial
Test as the 0.5.4 regression fixture. Generic Cloud/test ownership is unknown.

### Approved narrow cleanup — October 13, 2026

✅ User explicitly retained every applet-listed folder/mount. Kept all 11 saved
connections, including both SharePoint tests and production offline mirrors.
The reply was cleanup direction, not approval to tag/publish v0.5.3.

✅ Archived four unlisted legacy rclone unit files (ua-box, ua-engr, ua-gdrive,
uutzinger-gdrive) after confirming inactive/disabled state, no systemd reverse
dependencies/triggers, no local unit/autostart references and no user crontab.
Their managed replacements remain. Archived the two unlisted file-empty
cosmic-mounter-{flatpak-auth-test-20261013,onedrive-repair-test-20261011}
folders, retaining the latter's empty subdirectory. No files permanently deleted.
Archive: target/artifacts/rollback/native-cleanup-20261013/ with original-path
and rollback README. Existing unit backups remain in place.

✅ Rechecked archived unit byte parity and unchanged saved connection document
(SHA-256 ac1b9635462d4032a639b0e0ebd0cab2f665d99747cf5716a10ac9c350d7c23b).
After user-manager reload, all 11 managed services, both test timers and the
OpenConnect agent still exist. UA OneDrive remains active/running and mounted.
No managed service stopped or restarted, no profile altered, and no remote
data operation performed. Credentials/databases/recovery and unknown Cloud/test
are retained. Cleanup task is complete within the approved narrow scope.

⏳ Native v0.5.3 tag/publication still awaits explicit approval; final Flatpak
and COSMIC Store submission remain excluded. Records/checksum receipts are saved
in the working tree; no tag or publication occurred.

### Native release follow-up source closure — October 13, 2026

✅ Reviewed the checksum/CI receipts, reversible cleanup inventory, artifact
layout instructions and run-specific download helper/tests. Applet and engine
code are unchanged from accepted source 95fa290. Reconciled stale native gate
and branch-CI parent checkboxes without closing immutable-tag/publication or
0.5.4 sandbox acceptance. Fixed the release note's stale cleanup-pending wording.

✅ Focused verification passes: Rust formatting, 9 dependency-installer tests,
6 publication fixture tests, 3 download-destination subcases, shell syntax and
release metadata/notes preflight. Source review covers intended changed files
only; prior unchanged native/full-suite receipts remain separate. No mount or
provider authentication test was repeated. Prepared the authorized follow-up
commit/push with generated artifacts/rollback files excluded.

⏳ Next gate: explicit native v0.5.3 tag/publication approval, fresh tagged
six-platform builds and distinct final checksums. No tag, release publication
or final Flatpak/COSMIC Store claim is authorized by this source follow-up.

### Native v0.5.3 publication authorized — October 13, 2026

✅ Follow-up source/records committed and pushed as 61f2798; worktree clean.
User subsequently explicitly authorized publishing v0.5.3. Finalized native
release-note status and recorded tag/publication approval. This approval
excludes Flatpak delivery and COSMIC Store submission. Applet/engine code
remains unchanged from native accepted source; download helper/tests and
documentation changes were independently checked.

⏳ Commit/push the final approval record, create the immutable v0.5.3 tag,
verify the fresh six-platform tag build and package checksums, then use the
explicit accepted-tag publication workflow. Preserve branch-CI identities
and record tagged/public artifacts separately. No publication success yet.
