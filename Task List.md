# COSMIC Cloud Mounter Applet Task List

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
- [ ] Ensure the folder picker can create a new folder or offers a clear
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
- [x] Add and validate the COSMIC Cloud Mounter entry in `applets.ron`,
  including its public screenshot URL.
- [x] Commit and push the entry to the `uutzinger/cosmic-project-collection`
  fork.
- [x] Open
  [cosmic-utils/cosmic-project-collection pull request 85](https://github.com/cosmic-utils/cosmic-project-collection/pull/85).
- [ ] After the pull request is merged, verify the upstream workflow generates
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
- [ ] Add a Flatpak runtime mode that routes approved host commands through
  `flatpak-spawn --host`; preserve the current direct command runner for native
  `.deb` and source installations.
  - [x] Prototype the runtime command-runner layer with native and
    `flatpak-spawn --host` modes without wiring it into the installed applet.
  - [ ] Wire the applet to select the Flatpak runtime runner when executing
    inside an installed Flatpak.
- [x] Start from the accepted COSMIC drives-applet precedent of
  `--talk-name=org.freedesktop.Flatpak` and narrowly justify any additional
  permissions this applet requires.
- [ ] Verify host command argument passing, exit status, standard output,
  standard error, cancellation, timeouts, and secret redaction through
  `flatpak-spawn --host`.
  - [x] Add a probe-only Flatpak manifest that grants only
    `--talk-name=org.freedesktop.Flatpak`.
  - [x] Live-verify applet dependency detection through `flatpak-spawn --host`.
  - [x] Live-verify `rclone version`, `nmcli general status`,
    `systemctl --user --version`, and `fusermount3 --version` through
    `flatpak-spawn --host`.
  - [ ] Live-verify nonzero exit status, stderr capture, timeout, and
    cancellation behavior through `flatpak-spawn --host`.
- [x] Determine which configuration belongs inside the Flatpak and which must
  remain on the host. Existing rclone and OneDrive credentials must not be
  silently copied into a second credential store.
- [ ] Determine whether `--filesystem=host` is actually required. Prefer the
  narrowest filesystem permissions that still allow existing remote discovery,
  selected mount/mirror targets, recovery directories, and user-unit files.
- [ ] Verify that host-created FUSE mounts are visible to ordinary host file
  managers and are not confined to the Flatpak mount namespace.
- [ ] Verify that user systemd units generated by the applet invoke host binary
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

- [ ] Add a project-owned Flatpak manifest named
  `io.github.uutzinger.cosmic-ext-applet-mounter.json` under a documented
  packaging directory; keep it structurally ready to copy into the COSMIC
  repository's matching `app/<application-id>/` directory.
- [ ] Use the current accepted runtime stack:
  `org.freedesktop.Platform//25.08`, `org.freedesktop.Sdk//25.08`,
  `org.freedesktop.Sdk.Extension.rust-stable`, and
  `com.system76.Cosmic.BaseApp//stable`. Recheck these versions immediately
  before submission because repository conventions can advance.
- [ ] Set the manifest `command` to `cosmic-ext-applet-mounter`.
- [ ] Build the Rust binary from a tagged source archive or pinned commit rather
  than from an unpinned branch.
- [ ] Generate `cargo-sources.json` for an offline, reproducible
  `flatpak-builder --sandbox` build, including the pinned libcosmic Git revision
  and its Git/submodule dependencies.
- [ ] Add a repeatable project command or script that regenerates
  `cargo-sources.json` after dependency changes.
- [ ] Install the applet binary to `/app/bin/cosmic-ext-applet-mounter` and the
  OneDrive authentication helper only if the approved architecture still uses
  it.
- [ ] Install the desktop file as
  `/app/share/applications/io.github.uutzinger.cosmic-ext-applet-mounter.desktop`.
- [ ] Install AppStream metadata as
  `/app/share/metainfo/io.github.uutzinger.cosmic-ext-applet-mounter.metainfo.xml`.
- [ ] Install the scalable icon as
  `/app/share/icons/hicolor/scalable/apps/io.github.uutzinger.cosmic-ext-applet-mounter.svg`.
- [ ] Define the minimum required `finish-args`; justify each filesystem,
  socket, device, D-Bus, and host-command permission in manifest comments or
  packaging documentation.
- [ ] Include Wayland and required COSMIC settings-daemon access following
  accepted applet manifests; add fallback X11, IPC, network, session bus, host
  filesystem, or device access only when a tested feature requires it.
- [ ] Continue using the desktop portal for user-selected folders where
  possible, even if broader filesystem permissions are ultimately required for
  mount and mirror operation.

### Desktop and AppStream Metadata

- [x] Use the reverse-DNS application ID
  `io.github.uutzinger.cosmic-ext-applet-mounter` consistently in the desktop
  file, AppStream metadata, icon name, and application code.
- [x] Include `<id>com.system76.CosmicApplet</id>` in the AppStream
  `<provides>` section so COSMIC can classify the package as an applet.
- [ ] Change the AppStream binary declaration to the official template form:
  `<binaries><binary>cosmic-ext-applet-mounter</binary></binaries>`.
- [ ] Add the AppStream `COSMIC` category and `COSMIC` keyword while retaining
  useful storage-related keywords.
- [ ] Change the desktop entry to include `Categories=COSMIC;Utility;` and keep
  `Keywords=COSMIC;cloud;mount;mirror;storage;`.
- [x] Include `NoDisplay=true`, `X-CosmicApplet=true`, and
  `X-CosmicHoverPopup=Auto` in the desktop entry.
- [ ] Compare the final desktop and AppStream files with the current official
  applet template again immediately before submission.
- [ ] Update AppStream release entries to match the source tag submitted to the
  repository; do not label a final publication as a release candidate.
- [ ] Ensure AppStream includes stable remote icon and screenshot URLs that are
  reachable without authentication and will remain valid for the submitted
  release.
- [ ] Validate metadata with `desktop-file-validate` and
  `appstreamcli validate --pedantic`.

### Local Build and Installation

- [ ] Install the local prerequisites: `flatpak`, `flatpak-builder`, and `just`.
- [ ] Add the Flathub remote required by repository builds and runtime/SDK
  dependency installation.
- [ ] Add the COSMIC repository for local testing:
  `flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo`.
- [ ] Build through the current `pop-os/cosmic-flatpak` workflow using
  `just build io.github.uutzinger.cosmic-ext-applet-mounter` in a local clone of
  that repository; this runs `flatpak-builder` with `--sandbox`,
  `--force-clean`, `--install-deps-from=flathub`, and `--require-changes` and
  retains a log under `log/app/<application-id>/`.
- [ ] Run `just build-changed` from a submission branch to reproduce repository
  CI behavior before opening the pull request.
- [ ] Install the locally built Flatpak from the generated local OSTree
  repository for the current user and confirm the
  exported desktop entry, icon, AppStream record, and applet classification.
- [ ] Confirm the applet can be added to and removed from the COSMIC panel, and
  that logout/login preserves the configured panel entry.
- [ ] Confirm COSMIC Store displays the applet from a configured test/local
  Flatpak remote using its AppStream name, summary, icon, screenshots, and
  applet classification.
- [ ] Confirm uninstall removes packaged files without deleting user-created
  connection configuration, mirrors, mountpoints, credentials, or recovery
  data.

### Flatpak Functional Verification

- [ ] Verify the popup opens promptly and Add/Modify windows have the correct
  title, icon, tooltips, folder chooser, and keyboard navigation.
- [ ] Verify dependency detection reports host dependency state accurately from
  inside the packaged applet.
- [ ] Verify an existing rclone remote can be detected without copying or
  exposing credentials to applet configuration.
- [ ] Verify Box and Google Drive OAuth setup, SMB remote setup, and unused
  rclone remote removal under the approved sandbox architecture.
- [ ] Verify OneDrive Online Mount setup, mount, status refresh, clean unmount,
  and confirmed lazy-unmount recovery.
- [ ] Verify OneDrive Offline Mirror authentication, preview, initial sync,
  start/stop, Sync Now, conflict preservation, and recovery retention.
- [ ] Verify rclone Online Mount and Offline Mirror workflows for Box, Google
  Drive, and SMB using disposable/non-critical data.
- [ ] Verify user systemd services and timers are created, controlled, and
  observed in the intended host user session.
- [ ] Verify FUSE mounts created by the applet are visible to ordinary host file
  managers and applications and do not remain trapped in the Flatpak namespace.
- [ ] Verify NetworkManager and Cisco VPN detection, asynchronous popup status,
  activation readiness, and disconnect-only-if-activated behavior.
- [ ] Verify metered-network pause behavior and network-loss recovery.
- [ ] Repeat the data-integrity safety tests that prevent overlapping Online
  Mount and Offline Mirror targets or engines.
- [ ] Capture known sandbox limitations and any behavior that differs from the
  Debian/native installation.

### User Documentation and Release Preparation

- [ ] Add Flatpak build, local install, update, run, troubleshooting, and
  uninstall instructions to `README.md` without displacing native Debian/source
  installation instructions.
- [ ] State clearly whether host dependencies must be installed separately and
  how the Flatpak discovers and invokes them.
- [ ] Document every non-default Flatpak permission and why the applet needs it.
- [ ] Document whether existing native applet configuration is shared with or
  migrated to the Flatpak installation.
- [ ] Add a Flatpak-specific warning if the package cannot safely coexist with
  the native `.deb` or source installation.
- [ ] Prepare a tagged release whose version matches Cargo, AppStream, the
  Flatpak manifest source, and GitHub release artifacts.
- [ ] Add the completed build and verification evidence to
  `Task List Completion Notes.md`.

### COSMIC Repository Submission

- [ ] Fork and clone `https://github.com/pop-os/cosmic-flatpak`.
- [ ] Add only
  `app/io.github.uutzinger.cosmic-ext-applet-mounter/io.github.uutzinger.cosmic-ext-applet-mounter.json`
  and its required generated `cargo-sources.json` or equivalent source file to
  the repository submission unless maintainers request additional files.
- [ ] Run the repository's complete local validation/build target and resolve
  all errors without weakening the approved permission model.
- [ ] Review the submission diff for generated files, credentials, local paths,
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
