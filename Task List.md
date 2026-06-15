# COSMIC Cloud Mounter Applet Task List

**Status:** Draft for user review  
**Execution environment:** VS Code with Codex  
**Development state:** On hold pending final specification approval

This list implements `Requirements and Specifications.md`. The approved source
description is `Applet Description.md`.

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
- [ ] Review the revised requirements and this task list.
- [ ] Record explicit user approval to begin development.

**HOLD:** Codex shall not scaffold the application, change system configuration,
start services, connect VPNs, mount storage, synchronize data, or access cloud
accounts before Gate 0 is approved.

## Phase 1: Baseline and Scaffold

- [ ] Inventory the supplied applet template, scripts, service fixtures, and
  available local tool versions.
- [ ] Lock a compatible libcosmic revision and Rust toolchain.
- [ ] Instantiate the template as `cosmic-ext-applet-mounter`.
- [ ] Set app ID `io.github.uutzinger.cosmic-ext-applet-mounter`.
- [ ] Add MIT license and authors Urs Utzinger and OpenAI Codex.
- [ ] Set the planned repository URL without creating or publishing the
  repository.
- [ ] Create the approved module layout.
- [ ] Add formatting, lint, build, run, test, and staged-install recipes.
- [ ] Add a developer README for VS Code and command-line workflows.
- [ ] Initialize Git only after separate user approval.
- [ ] Run a COSMIC popup smoke test.

### Phase 1 verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --all-targets`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] Manual popup open/close test.

## Phase 2: Domain Model and Configuration

- [ ] Define providers, modes, connections, VPN profiles, operations, conflicts,
  recovery records, and status types.
- [ ] Implement stable UUID connection identities.
- [ ] Implement versioned COSMIC configuration.
- [ ] Implement atomic writes, rollback, migration, and malformed-data recovery.
- [ ] Validate local and remote paths.
- [ ] Reject mount/mirror reuse, nested mirrors, recursive sync trees, duplicate
  targets, and unsafe system paths.
- [ ] Add configuration fixtures without real credentials.

### Phase 2 verification

- [ ] Test serialization, migration, defaults, and invalid data.
- [ ] Test all path-overlap and mode-separation rules.
- [ ] Confirm configuration contains no secret fields.

## Phase 3: Process, Dependency, and Diagnostics Layer

- [ ] Define an asynchronous typed command-runner interface.
- [ ] Invoke fixed executables with separate validated arguments.
- [ ] Bound output, duration, retries, and cancellation.
- [ ] Redact credentials, tokens, URLs, and sensitive arguments.
- [ ] Detect rclone, onedriver, `abraunegg/onedrive`, FUSE, NetworkManager, Cisco,
  and optional diagnostics tools.
- [ ] Validate required capabilities as well as versions.
- [ ] Reject rclone `1.60.1` and provide current-stable upgrade guidance.
- [ ] Add fake runners and dependency inventories.

### Phase 3 verification

- [ ] Test missing, outdated, and capability-incomplete tools.
- [ ] Test timeout, nonzero exit, invalid UTF-8, large output, and cancellation.
- [ ] Security-review all external command construction.

## Phase 4: Systemd and Runtime Infrastructure

- [ ] Define service and timer manager interfaces.
- [ ] Render deterministic applet-owned units with UUID markers.
- [ ] Install, validate, update, roll back, and remove units atomically.
- [ ] Implement daemon reload, enable, disable, start, stop, reset-failed, and
  status.
- [ ] Define mount-table and sync-runtime interfaces.
- [ ] Implement per-connection operation serialization.
- [ ] Build fake systemd, mount-table, and timer implementations.

### Phase 4 verification

- [ ] Snapshot-test generated units and timers.
- [ ] Test ownership checks and rollback.
- [ ] Confirm tests never change the real user manager.

## Phase 5: Online Mount Engines

### Rclone mount

- [ ] Enumerate and validate Google Drive, Box, and SMB remotes.
- [ ] Implement provider-specific remote/subtree configuration.
- [ ] Render rclone mount services with full VFS caching.
- [ ] Apply the configurable 20 GiB default cache limit.
- [ ] Apply bounded timeout and retry defaults.
- [ ] Enable VFS queue and cache health inspection.
- [ ] Detect queued uploads, active uploads, cache errors, and cache exhaustion.

### Onedriver

- [ ] Detect current onedriver capabilities and authentication state.
- [ ] Implement OneDrive Online mount setup and service management.
- [ ] Detect cached read-only offline behavior and mount health.

### Connectivity recovery

- [ ] Monitor network, VPN, provider, service, and actual mount state.
- [ ] Safely auto-detach rclone mounts only when no writes are pending.
- [ ] Preserve cache and expose recovery when writes are pending.
- [ ] Implement automatic remount for enabled connections after readiness returns.
- [ ] Add bounded exponential remount backoff.
- [ ] Implement clean unmount and busy-mount diagnostics.
- [ ] Offer warned, explicit-confirmation lazy unmount after clean unmount fails.
- [ ] Block lazy unmount while writes are queued or in progress.

### Phase 5 verification

- [ ] Test clean mount/unmount and partial service/mount states.
- [ ] Test safe detach, blocked detach, cache error, and reconnect/remount.
- [ ] Test file-manager-facing operations return within configured bounds during
  simulated connectivity failure.

## Phase 6: Offline Mirror Engines

### Shared synchronization behavior

- [ ] Select whole remote or remote subtree.
- [ ] Estimate remote size and local free-space requirements.
- [ ] Generate a dry preview of upload, download, delete, conflict, skip, and
  transfer totals.
- [ ] Require explicit confirmation before initial synchronization.
- [ ] Implement Sync Now, Pause, Resume, and automatic reconnect sync.
- [ ] Schedule non-continuous engines every 15 minutes after completion.
- [ ] Prevent concurrent runs.
- [ ] Pause automatic sync on metered networks by default.
- [ ] Implement per-connection metered override.
- [ ] Propagate creates, modifications, renames, and deletions bidirectionally.
- [ ] Preserve both conflict versions and report them.
- [ ] Retain deleted and overwritten files for 30 days.
- [ ] Keep cache, work, and recovery directories outside the mirror tree.
- [ ] Implement interrupted-run recovery without routine destructive resync.
- [ ] Require preview and confirmation for any state rebuild or resync.

### Rclone bisync

- [ ] Implement dedicated work state per connection.
- [ ] Require supported access checks and resilient/recovery capabilities.
- [ ] Configure conflict-loser preservation and recovery directories.
- [ ] Implement Google Drive cloud-native document exclusion and reporting.
- [ ] Add Google Drive, Box, and SMB adapters.

### OneDrive mirror

- [ ] Detect current `abraunegg/onedrive` capabilities.
- [ ] Create isolated configuration and sync directories per connection.
- [ ] Implement supported browser authentication.
- [ ] Use monitor mode for continuous synchronization where supported.
- [ ] Map native conflict, recovery, status, and resync-required states into the
  applet model.

### Phase 6 verification

- [ ] Test initial empty/local/remote/both-populated cases.
- [ ] Test offline local edits followed by reconnect.
- [ ] Test remote-only changes and deletions.
- [ ] Test simultaneous same-file edits preserving both versions.
- [ ] Test interrupted runs and non-destructive recovery.
- [ ] Test 30-day recovery cleanup boundaries.
- [ ] Test low disk space and metered-network behavior.
- [ ] Test Google cloud-native document exclusion.

## Phase 7: VPN Integration

### NetworkManager

- [ ] Enumerate visible VPN profiles.
- [ ] Implement activation, state, and deactivation through D-Bus.
- [ ] Document and test any fixed-argument fallback.

### Cisco Secure Client

- [ ] Detect agent, GUI, interface, and tunnel state.
- [ ] Implement authorized agent startup without storing sudo credentials.
- [ ] Open the Cisco GUI for interactive authentication.

### Coordination

- [ ] Implement interface, route, DNS, endpoint, and NetworkManager readiness
  checks.
- [ ] Block mount and sync until readiness succeeds or times out.
- [ ] Reference-count shared VPN dependencies.
- [ ] Track whether the applet activated each VPN.
- [ ] Never disconnect a VPN still used by another connection.
- [ ] Disconnect an unused VPN automatically only when the applet activated it.

### Phase 7 verification

- [ ] Test shared, pre-existing, applet-activated, failed, and timed-out VPNs with
  fakes.
- [ ] Test mount and sync failure after successful VPN activation.
- [ ] Do not activate a real VPN before the manual-test gate.

## Phase 8: Legacy Service Import

- [ ] Scan `~/.config/systemd/user/` by default.
- [ ] Parse the compatible subset of rclone and onedriver units as structured
  data.
- [ ] Never execute imported text.
- [ ] Display provider, remote, target, cache, startup, and unsupported options.
- [ ] Detect active-service and local-target conflicts.
- [ ] Require confirmation before creating an applet-owned replacement.
- [ ] Preserve original units by default.
- [ ] Offer a separate confirmed action to disable an imported original.
- [ ] Test against copies of all files in repository `services/`.

### Phase 8 verification

- [ ] Confirm fixture and original units remain unchanged.
- [ ] Confirm import cannot inject commands or copy credentials.
- [ ] Test malformed, unsupported, duplicate, and conflicting units.

## Phase 9: Operation Controller and UI

- [ ] Implement separate Online mount and Offline mirror state machines.
- [ ] Restore status from configuration, systemd, mount tables, sync state,
  connectivity, and VPN state.
- [ ] Implement popup aggregate state and connection rows.
- [ ] Implement mount/unmount, Sync Now, Pause/Resume, retry, repair, and details.
- [ ] Implement provider- and mode-specific settings.
- [ ] Implement disk estimate and initial-sync preview/confirmation.
- [ ] Implement pending-write, conflict, recovery, low-space, and metered states.
- [ ] Implement dependency setup and upgrade guidance.
- [ ] Implement legacy import UI.
- [ ] Add sanitized logs and optional notifications.
- [ ] Add keyboard navigation, accessible names, and localization.

### Phase 9 verification

- [ ] Test UI model updates with fake backends.
- [ ] Check narrow and wide panel layouts.
- [ ] Check keyboard-only operation.
- [ ] Check that every state is understandable without color.

## Gate 1: Isolated Release Candidate

- [ ] All formatting, lint, unit, integration, and snapshot tests pass.
- [ ] No test has touched real services, mounts, VPNs, credentials, or cloud data.
- [ ] Review generated units, timers, logs, and fixtures for secret leakage.
- [ ] Review sync deletion, conflict, and recovery behavior for data-loss risks.
- [ ] Review detach and VPN policies against the approved specification.
- [ ] Obtain user approval for controlled real-system testing.

## Phase 10: Controlled Manual Testing

Use disposable connections and non-critical data. Record results and cleanup.

- [ ] Verify dependency detection and outdated-version guidance.
- [ ] Test a disposable rclone Online mount.
- [ ] Test safe connectivity-loss detach and automatic remount.
- [ ] Test pending-write protection during connectivity loss.
- [ ] Test onedriver Online mount and offline cached read-only behavior.
- [ ] Test a local-to-local rclone bisync mirror.
- [ ] Test one approved Google Drive or Box Offline mirror.
- [ ] Test offline editing followed by reconnect synchronization.
- [ ] Test conflict preservation, deletion propagation, and recovery.
- [ ] Test metered pause and Sync Now.
- [ ] Test `abraunegg/onedrive` with a non-critical account or subtree.
- [ ] Test NetworkManager VPN readiness.
- [ ] Test Cisco interactive authentication when installed.
- [ ] Test import preview and confirmed import from the real user service folder.
- [ ] Verify removal preserves credentials, data, cache, recovery, and originals.

## Phase 11: Documentation and Packaging

- [ ] Document installation and current-version requirements for every engine.
- [ ] Document the Online mount versus Offline mirror tradeoff.
- [ ] Document synchronization conflicts, deletions, recovery, and backup limits.
- [ ] Document authentication and VPN behavior.
- [ ] Document generated files, legacy imports, and uninstall behavior.
- [ ] Add MIT license and author attribution.
- [ ] Finalize desktop entry, metainfo, and icon.
- [ ] Test staged installation and uninstallation.
- [ ] Prepare release notes and known limitations.

## Gate 2: Version 0.1 Completion

- [ ] Every acceptance criterion is demonstrated.
- [ ] Required automated and approved manual tests pass.
- [ ] No unresolved high-severity security, data-loss, or privilege issue remains.
- [ ] Known limitations are documented.
- [ ] User approves the release candidate.

## Codex Working Rules

- [ ] Work one phase at a time and keep this list current.
- [ ] Inspect current files and preserve unrelated user changes.
- [ ] Use small, reviewable patches.
- [ ] Never place real credentials, tokens, or organization-specific secrets in
  source or fixtures.
- [ ] Use temporary directories and fake adapters for automated tests.
- [ ] Run each phase's verification before reporting completion.
- [ ] Report changed files, tests, failures, and remaining risks.
- [ ] Stop at each approval gate and wait for explicit approval.
