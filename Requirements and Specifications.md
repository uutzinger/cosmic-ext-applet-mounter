# COSMIC Cloud Mounter Applet

## Requirements and Specifications

**Document status:** Draft for user review  
**Revision:** 0.2  
**Implementation status:** Not started  

This document translates `Applet Description.md`, `Cloud Drive Connections.md`, the existing scripts, and the existing systemd user services into testable product requirements and a proposed technical design.

`Applet Description.md` is the source of truth when this document and the source description disagree.

## 1. Purpose

The COSMIC Cloud Mounter Applet manages cloud and network storage connections
from the COSMIC desktop panel. Each connection uses one of two mutually exclusive
access modes:

- **Online mount:** A network-backed filesystem that provides on-demand access.
- **Offline mirror:** An ordinary local directory synchronized bidirectionally
  with a remote location.

A connection may depend on a VPN. The applet prepares and verifies that VPN
before mounting or synchronizing.

The applet replaces manually maintained desktop shortcuts, shell scripts, and
service files while retaining systemd user services as the durable runtime
mechanism.

## 2. Goals

- Show every applet-managed storage connection and its current state.
- Let the user choose Online mount or Offline mirror for each connection.
- Prevent cloud connectivity from blocking the file manager when Offline mirror mode is selected.
- Reduce file-manager stalls for Online mounts through health monitoring,
  bounded timeouts, safe detachment, and automatic recovery.
- Mount, unmount, synchronize, pause, resume, and inspect individual
  connections.
- Configure optional NetworkManager or Cisco VPN dependencies.
- Generate and manage applet-owned systemd user services and timers.
- Import compatible existing rclone and onedriver user services.
- Preserve both sides of synchronization conflicts and provide recovery copies.
- Detect missing or outdated dependencies without installing software.
- Keep credentials out of applet configuration, generated units, logs, and
  notifications.

## 3. Version 0.1 Scope

### 3.1 Supported provider and mode matrix

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | `onedriver` | `abraunegg/onedrive` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |

Only the providers and engines in this matrix are included.

### 3.2 Included

- COSMIC panel icon, popup, and settings window.
- Live connection, mount, synchronization, network, and VPN status.
- Add, edit, validate, test, enable, disable, and remove connections.
- Online mount lifecycle and health management.
- Offline mirror initialization, scheduling, synchronization, conflict
  preservation, deletion propagation, and recovery retention.
- NetworkManager VPN enumeration and activation.
- Cisco Secure Client detection, agent startup, GUI launch, and readiness
  monitoring.
- Dependency and version checks for rclone, onedriver,
  `abraunegg/onedrive`, FUSE, and supporting utilities.
- Import of compatible legacy services found in
  `~/.config/systemd/user/`.
- Localized UI strings and accessible controls.

### 3.3 Non-goals

- Implementing storage, synchronization, or VPN protocols in the applet.
- Supporting providers outside the approved matrix.
- Managing VPN profiles, VPN credentials, or Cisco authentication.
- Treating an Online mount cache as a complete offline copy.
- Automatically installing packages, adding repositories, or updating tools.
- Storing provider or VPN secrets.
- Running arbitrary user-entered shell commands.
- Presenting synchronization as a replacement for backups.

## 4. Technical Clarifications

### 4.1 Online mount limitations

A FUSE filesystem can still cause an application to wait while the remote
provider, network, or VPN is unavailable. Bounded timeouts and automatic
detachment reduce this risk but cannot eliminate it.

The applet shall recommend Offline mirror mode when uninterrupted local browsing
and editing are required.

### 4.2 Offline mirror behavior

Offline mirror is a full local working tree, not a mount cache. Applications
interact only with local files. A background engine compares the local and remote
trees when required connectivity is available.

Offline mirror mode uses:

- `abraunegg/onedrive` for Microsoft OneDrive.
- `rclone bisync` for Google Drive, Box, and SMB.

### 4.3 Google cloud-native documents

Google Docs, Sheets, and Slides do not round-trip safely through exported file
formats. Offline mirror mode shall skip these cloud-native documents, report
them to the user, and offer to open them in the browser.

### 4.4 Cisco VPN control

An active Cisco agent does not prove that an authenticated tunnel exists. The
applet may start the agent and open the Cisco UI, but storage operations shall
wait for configured readiness checks.

## 5. User Workflows

### 5.1 Configure a connection

1. The user selects a provider and one access mode.
2. The user selects a remote account and optional remote subtree.
3. The user selects a mountpoint or local mirror directory.
4. The applet validates paths, disk space, dependencies, and tool versions.
5. The applet launches the supported authentication flow or provides exact
   setup instructions.
6. The user optionally selects a VPN dependency and readiness checks.
7. The applet tests the connection and shows a preview.
8. The user confirms creation.
9. The applet writes configuration and generated units atomically.

### 5.2 Use an Online mount

1. The user enables the connection.
2. The applet prepares and verifies any required VPN.
3. The applet starts the mount service.
4. The applet verifies the actual mount and provider health.
5. The applet monitors pending writes and connectivity.
6. On a safe connectivity failure, the applet detaches the mount.
7. When readiness returns, the applet remounts the connection if it remains
   enabled.

### 5.3 Use an Offline mirror

1. The applet estimates remote size and available local space.
2. The applet runs a dry preview of the initial synchronization.
3. The UI shows expected uploads, downloads, deletions, and conflicts.
4. The user explicitly confirms the initial synchronization.
5. Applications use the local directory whether online or offline.
6. The applet synchronizes after connectivity returns, periodically while
   online, and when the user selects Sync Now.
7. Conflicts preserve both versions and are presented for user review.

### 5.4 Import a legacy connection

1. The applet scans `~/.config/systemd/user/` for compatible rclone and
   onedriver services.
2. It parses only a documented safe subset of unit syntax and arguments.
3. It shows an import preview, unsupported options, and target conflicts.
4. The user confirms creation of an applet-managed replacement.
5. The original service is preserved unless the user separately confirms
   disabling it.

## 6. Functional Requirements

### 6.1 Panel and status

- **FR-001:** The applet shall provide a COSMIC panel icon and popup.
- **FR-002:** The popup shall show every configured connection.
- **FR-003:** Each row shall show provider, mode, local path, VPN state, and
  current operation state.
- **FR-004:** Online mount states shall include `unmounted`,
  `waiting-for-network`, `waiting-for-vpn`, `mounting`, `mounted`,
  `pending-writes`, `detaching`, `error`, and `unavailable`.
- **FR-005:** Offline mirror states shall include `idle`, `offline`,
  `waiting-for-vpn`, `previewing`, `syncing`, `paused`, `metered-paused`,
  `conflict`, `error`, and `unavailable`.
- **FR-006:** The UI shall distinguish an active service from an actual
  filesystem mount.
- **FR-007:** The UI shall show the last successful synchronization time for
  Offline mirrors.
- **FR-008:** The UI shall show pending uploads, conflicts, warnings, and
  actionable errors.
- **FR-009:** Status shall refresh at startup, popup open, operation completion,
  systemd state changes, and connectivity changes.
- **FR-010:** Status polling shall not block the COSMIC event loop.

### 6.2 Connection management

- **FR-011:** Settings shall allow add, edit, test, enable, disable, and remove.
- **FR-012:** Every connection shall have a stable generated UUID.
- **FR-013:** A connection shall use exactly one access mode.
- **FR-014:** A path used as an Online mountpoint shall not be used as an
  Offline mirror directory, and vice versa.
- **FR-015:** The applet shall reject duplicate, nested, unsafe, or unsupported
  local targets where overlap could cause recursion or data loss.
- **FR-016:** New Online mounts shall start manually by default.
- **FR-017:** The user may enable an Online mount at login.
- **FR-018:** Removing a connection shall preserve credentials, user data,
  caches, recovery data, and external services unless separate cleanup is
  explicitly confirmed.
- **FR-019:** Destructive or data-affecting actions shall require confirmation.
- **FR-020:** Configuration and managed unit changes shall be atomic and
  recoverable.

### 6.3 Dependency management

- **FR-021:** The applet shall detect each required executable and report its
  version.
- **FR-022:** The applet shall require a current supported release of each
  selected storage engine.
- **FR-023:** The applet shall reject the installed rclone `1.60.1` for managed
  connections and direct the user to upgrade.
- **FR-024:** Dependency guidance shall identify the required upstream project
  and shall not perform installation or repository changes.
- **FR-025:** A missing dependency for one provider or mode shall not prevent use
  of other available providers or modes.
- **FR-026:** Capability checks shall verify required commands and flags rather
  than relying only on a version string.

### 6.4 Provider behavior

- **FR-027:** Google Drive, Box, and SMB shall use an existing or newly
  configured rclone remote.
- **FR-028:** The applet shall enumerate rclone remote names without loading
  credentials into applet state.
- **FR-029:** The applet shall verify that a selected remote and subtree exist
  before activation.
- **FR-030:** Rclone and onedriver authentication shall be delegated to their
  supported setup flows.
- **FR-031:** OneDrive Offline mirror authentication and state shall be delegated
  to `abraunegg/onedrive`.
- **FR-032:** SMB passwords shall remain in rclone's credential mechanism.
- **FR-033:** Google cloud-native documents shall be excluded from Offline
  mirrors and reported in the UI.

### 6.5 Online mount management

- **FR-034:** Online mounts shall run as systemd user services.
- **FR-035:** Rclone mounts shall use full VFS caching and a default maximum
  cache size of 20 GiB per connection.
- **FR-036:** The user may override the cache limit in advanced settings.
- **FR-037:** Mount operations shall use bounded connection, operation, retry,
  and backoff values.
- **FR-038:** The applet shall monitor network, VPN, service, mount-table, and
  provider health.
- **FR-039:** For rclone mounts, the applet shall inspect VFS queue and cache
  status, including queued uploads, uploads in progress, cache errors, and
  cache exhaustion.
- **FR-040:** After connectivity loss, the applet shall automatically detach an
  rclone mount only when no write is queued or in progress.
- **FR-041:** When writes are pending, the applet shall preserve the cache, warn
  the user, and expose retry, wait, and manual recovery actions.
- **FR-042:** Enabled connections safely detached because of connectivity shall
  automatically remount after network and VPN readiness checks pass.
- **FR-043:** Automatic remount retries shall use bounded exponential backoff.
- **FR-044:** An Online mount cache shall never be presented as complete offline
  availability.

### 6.6 Offline mirror management

- **FR-045:** The user shall be able to mirror an entire remote or a selected
  remote subtree.
- **FR-046:** The applet shall estimate remote size and validate local free space
  before initial synchronization.
- **FR-047:** Initial synchronization shall run a dry preview and require
  explicit confirmation.
- **FR-048:** The preview shall summarize uploads, downloads, deletions,
  conflicts, skipped items, and estimated transfer size.
- **FR-049:** Local files shall remain readable and writable without network or
  VPN connectivity.
- **FR-050:** Automatic synchronization shall run when readiness returns and
  periodically while connected.
- **FR-051:** The default periodic interval for engines without continuous
  monitoring shall be 15 minutes after the previous run completes.
- **FR-052:** The user shall have Sync Now, Pause, and Resume actions.
- **FR-053:** `abraunegg/onedrive` continuous monitoring shall be used for
  OneDrive when supported by the installed release.
- **FR-054:** Only one synchronization operation may run per connection.
- **FR-055:** Automatic synchronization shall pause on metered networks by
  default, with Sync Now and a per-connection override available.
- **FR-056:** Changes and deletions shall propagate in both directions.
- **FR-057:** When the same file changes on both sides, neither version shall be
  silently overwritten; both versions shall be preserved and reported.
- **FR-058:** Deleted and overwritten files shall be retained in recovery
  storage for 30 days.
- **FR-059:** Recovery cleanup shall never run while a synchronization for that
  connection is active.
- **FR-060:** Interrupted synchronization shall retain engine state and offer
  automatic recovery or a reviewed recovery workflow.
- **FR-061:** A resync or state-database rebuild shall not run as routine startup
  behavior and shall require a dry preview and explicit confirmation.
- **FR-062:** Synchronization errors shall preserve local user work.

### 6.7 Legacy service import

- **FR-063:** The default legacy scan location shall be
  `~/.config/systemd/user/`.
- **FR-064:** Import shall support compatible rclone mount and onedriver units.
- **FR-065:** Import shall parse unit files as structured data and shall never
  execute imported text.
- **FR-066:** Import shall display parsed provider, remote, local target, cache,
  startup, and unsupported options before confirmation.
- **FR-067:** Import shall create a new applet-owned connection and managed unit.
- **FR-068:** Import shall preserve the original external unit by default.
- **FR-069:** Conflicting active services or local targets shall block activation
  until resolved.
- **FR-070:** The repository `services/` directory shall serve as import test
  fixtures.

### 6.8 Systemd service management

- **FR-071:** Managed units shall be stored in
  `~/.config/systemd/user/`.
- **FR-072:** Managed service names shall use
  `cosmic-mounter-<connection-uuid>.service`.
- **FR-073:** Scheduled sync timers shall use
  `cosmic-mounter-<connection-uuid>.timer`.
- **FR-074:** Generated files shall include an applet ownership marker and UUID.
- **FR-075:** The applet shall update or remove only files whose ownership marker
  and UUID match its configuration.
- **FR-076:** Generated unit content shall be deterministic.
- **FR-077:** Unit writes shall use a temporary file, validation, and atomic
  rename.
- **FR-078:** The user manager shall be reloaded after managed unit changes.
- **FR-079:** A failed update shall preserve or restore the last valid
  configuration and unit.
- **FR-080:** Commands shall use fixed executables and separate validated
  arguments, never shell interpolation.

### 6.9 VPN dependencies

- **FR-081:** A connection may reference zero or one VPN profile.
- **FR-082:** The applet shall enumerate NetworkManager VPN profiles visible to
  the current user.
- **FR-083:** NetworkManager activation and status shall use D-Bus when practical,
  with a documented fixed-argument fallback.
- **FR-084:** Cisco support shall detect its agent, GUI, interface, and tunnel
  state separately.
- **FR-085:** Each dependency shall support readiness checks using one or more of
  NetworkManager state, interface, route, DNS, or endpoint reachability.
- **FR-086:** Mounting or synchronization shall not start until readiness passes
  or a bounded timeout expires.
- **FR-087:** The UI shall report when interactive Cisco authentication is
  required.
- **FR-088:** The applet shall reference-count connections using the same VPN.
- **FR-089:** No VPN still required by a mounting, mounted, previewing, or syncing
  connection shall be disconnected.
- **FR-090:** The applet shall automatically disconnect a VPN only when the
  applet activated it and no mounting, mounted, previewing, or syncing connection
  still depends on it.

### 6.10 Errors, logs, and notifications

- **FR-091:** Expected operational failures shall not crash the applet.
- **FR-092:** Errors shall identify the failed stage and a practical next action.
- **FR-093:** Logs shall redact credentials, tokens, and sensitive command
  arguments.
- **FR-094:** The UI shall provide sanitized recent logs and details.
- **FR-095:** Notifications shall be optional and shall not repeat on status
  polling.
- **FR-096:** Conflict, pending-write, low-space, and recovery-required states
  shall produce persistent UI indicators until resolved.

## 7. Non-functional Requirements

- **NFR-001 Reliability:** Applet restart shall reconstruct state from
  configuration, systemd, mount tables, sync-engine state, and connectivity.
- **NFR-002 Responsiveness:** No external command, network check, mount, or sync
  shall block the UI event loop.
- **NFR-003 File-manager isolation:** Offline mirror paths shall remain ordinary
  local filesystem paths independent of provider availability.
- **NFR-004 Performance:** With 20 configured connections, the popup should
  render cached state within 250 ms.
- **NFR-005 Resource use:** Event-driven monitoring shall be preferred to rapid
  polling.
- **NFR-006 Security:** Secrets shall not appear in applet configuration, unit
  files, process logs, notifications, or test fixtures.
- **NFR-007 Least privilege:** Storage engines shall run as the user. System
  authorization shall be limited to starting or stopping the Cisco agent when
  required.
- **NFR-008 Accessibility:** Controls shall have accessible names, keyboard
  focus, and non-color-only state indicators.
- **NFR-009 Localization:** All user-visible strings shall be translatable.
- **NFR-010 Compatibility:** The target is Linux with COSMIC, systemd user
  sessions, NetworkManager, and FUSE 3.
- **NFR-011 Maintainability:** UI, provider, synchronization, service, VPN, and
  process logic shall use typed interfaces and test fakes.
- **NFR-012 Testability:** Automated tests shall not use real credentials,
  services, mounts, VPNs, or cloud data.
- **NFR-013 Recoverability:** Interrupted writes or operations shall not leave
  partial applet configuration or silently discard local work.

## 8. System Requirements

### 8.1 Platform

- Linux distribution running COSMIC.
- systemd with a working user manager.
- Rust toolchain compatible with the selected libcosmic revision.
- FUSE 3 and `fusermount3` for Online mounts.
- NetworkManager for COSMIC-managed VPN integration.
- A graphical browser for provider authentication.

### 8.2 External tools

| Capability | Required software |
|---|---|
| Google Drive, Box, SMB Online mount | Current stable `rclone` and FUSE 3 |
| Google Drive, Box, SMB Offline mirror | Current stable `rclone` with required bisync safety features |
| OneDrive Online mount | Current supported `onedriver` |
| OneDrive Offline mirror | Current supported `abraunegg/onedrive` |
| NetworkManager VPN | NetworkManager D-Bus service |
| Cisco VPN | Cisco Secure Client agent and optional GUI |
| Busy-mount diagnostics | Optional `fuser` from `psmisc` |

Dependencies shall be checked independently. An unavailable mode shall not
disable unrelated modes.

### 8.3 Permissions

- Mountpoints, mirror directories, caches, recovery directories, and units shall
  be user-owned.
- Storage services and sync jobs shall run without root privileges.
- Cisco system-service control may request system authorization.
- The applet shall not embed or request a reusable sudo password.

## 9. Proposed Architecture

The applet shall use Rust edition 2024, libcosmic, asynchronous tasks, versioned
COSMIC configuration, and systemd user services.

### 9.1 Modules

| Module | Responsibility |
|---|---|
| `app` | COSMIC model, messages, popup, and settings |
| `model` | Connection, mode, provider, VPN, operation, and status types |
| `config` | Versioning, validation, migration, and atomic persistence |
| `providers` | rclone, onedriver, and `abraunegg/onedrive` adapters |
| `mounts` | Mount lifecycle, mount-table inspection, and VFS health |
| `sync` | Preview, scheduling, conflict/recovery state, and sync lifecycle |
| `services` | systemd unit/timer rendering and management |
| `vpn` | NetworkManager, Cisco, readiness checks, and dependency references |
| `process` | Typed asynchronous command execution and sanitized results |
| `import` | Structured legacy unit discovery, parsing, and preview |
| `diagnostics` | Dependency checks, journal access, and error mapping |
| `i18n` | Localization resources and initialization |

External interactions shall be represented by traits so tests can use fake
process, provider, mount, synchronization, service, and VPN implementations.

### 9.2 Runtime boundaries

- The UI emits typed intents such as `Mount`, `Unmount`, `SyncNow`, `PauseSync`,
  and `ConfirmInitialSync`.
- Operations are serialized per connection.
- Provider and VPN work runs asynchronously.
- Applet exit does not terminate enabled systemd services.
- Shell scripts are references only and are not the applet execution API.

### 9.3 Command execution

The preferred order is:

1. Stable native Rust or D-Bus API.
2. A known executable with separate validated arguments.
3. No `sh -c`, command concatenation, or execution of imported configuration.

Output shall be bounded, decoded safely, and redacted before logging.

## 10. Configuration Model

Configuration shall use app ID
`io.github.uutzinger.cosmic-ext-applet-mounter` and a versioned COSMIC
configuration namespace.

```text
AppConfig
  version
  notifications_enabled
  connections[]
  vpn_profiles[]

Connection
  id: UUID
  name
  provider: OneDrive | GoogleDrive | Box | Smb
  mode: OnlineMount | OfflineMirror
  remote_reference
  remote_subpath?
  local_path
  cache_directory?
  recovery_directory?
  start_at_login
  sync_interval_minutes
  sync_on_metered
  vpn_profile_id?
  disconnect_vpn_when_unused
  tuning_profile

VpnProfile
  id: UUID
  name
  kind: NetworkManager | Cisco
  external_profile_id?
  readiness_checks[]
  timeout_seconds
```

### 10.1 Validation

- Names shall be non-empty and contain no control characters.
- UUIDs are generated and not user editable.
- Local paths shall be absolute, user-writable or safely creatable, and not a
  system directory.
- No configured local path may equal, contain, or be contained by another path
  when that overlap could cause recursive synchronization or mount shadowing.
- A mountpoint and mirror directory shall never be shared.
- Cache and recovery directories shall use user-writable locations outside the
  visible mirror tree.
- Remote references shall be passed as arguments, not shell syntax.
- VPN references shall resolve to configured profiles.

## 11. Generated Service Specifications

### 11.1 Rclone Online mount

The generated service shall:

- wait for applet-managed network and VPN readiness;
- create validated mount and cache directories;
- invoke rclone directly;
- use the user's rclone configuration;
- enable VFS status inspection;
- use FUSE 3 clean unmount;
- restart unexpected failures with bounded backoff;
- contain no credentials;
- be disabled at login by default.

Initial mount tuning:

```text
--vfs-cache-mode full
--vfs-cache-max-age 168h
--vfs-cache-max-size 20G
--vfs-cache-poll-interval 5m
--dir-cache-time 5m
--timeout 10s
--contimeout 5s
--low-level-retries 1
--retries 1
--retries-sleep 5s
--umask 002
--log-level INFO
```

Provider-specific changes require tests and shall preserve bounded failure
behavior.

### 11.2 Rclone Offline mirror

The generated service and timer shall:

- use one dedicated bisync work directory per connection;
- use access checks and supported resilient/recovery features;
- prevent concurrent runs;
- preserve conflict losers rather than deleting them;
- use recovery directories for deleted or overwritten files;
- skip cloud-native Google documents for Google Drive;
- run every 15 minutes while readiness permits;
- preserve state after interruption;
- never add routine `--resync` behavior.

### 11.3 OneDrive services

- Online mount shall use onedriver's supported user-service behavior.
- Offline mirror shall use a dedicated `abraunegg/onedrive` configuration and
  sync directory per connection.
- OneDrive monitor mode shall be used when supported.
- Destructive resync/state rebuild options require explicit reviewed recovery.

### 11.4 Ownership

Every generated file shall include an applet-managed marker and connection UUID.
Existing unmarked units remain external until explicitly imported.

## 12. State and Operation Rules

- Repeated identical requests shall be idempotent.
- Closing the popup shall not cancel an operation.
- Mount success requires an actual mount, not only an active service.
- Sync success requires a successful engine result and post-sync validation.
- A safe auto-detach requires no queued or in-progress writes.
- Automatic remount applies only to connections that remain enabled.
- Offline mirror files remain available while synchronization is paused or
  offline.
- A failed sync shall not trigger automatic destructive resync.
- Clean unmount is always attempted before any alternative.
- If clean unmount fails, the applet may offer lazy unmount only after explicit
  user confirmation and a clear warning.
- Queued or in-progress writes shall prevent lazy unmount.
- The applet shall disconnect only a VPN it activated, and only after no active
  connection still depends on it.

## 13. UI Specification

### 13.1 Panel popup

- Header with aggregate status, refresh, and Settings.
- Scrollable connection rows.
- Each row shows provider, mode, local path, VPN, state, and warnings.
- Online mounts provide mount/unmount and repair actions.
- Offline mirrors provide Sync Now, Pause/Resume, last-sync time, progress,
  conflicts, and recovery status.
- Empty state provides Add connection.

### 13.2 Settings

- Connection list with Add, Edit, Test, Import, and Remove.
- Provider and mode selection followed by mode-specific fields.
- Remote/subtree selection and local path picker.
- Disk-space estimate and initial-sync preview for Offline mirrors.
- Start-at-login setting for Online mounts.
- Sync interval and metered-network setting for Offline mirrors.
- VPN dependency and readiness checks.
- Dependency versions and setup/authentication actions.
- Advanced cache, timeout, bandwidth, recovery, and schedule settings.

### 13.3 Error and conflict details

Details shall show:

- failed stage and sanitized summary;
- current service, mount, sync, network, and VPN state;
- pending writes or transfers;
- conflict and recovery file locations;
- suggested corrective action;
- optional sanitized recent logs.

## 14. Testing Requirements

### 14.1 Automated tests

- Configuration serialization, migration, and invalid-data recovery.
- Mode/path overlap and recursive-sync validation.
- Dependency version and capability checks.
- Deterministic service and timer rendering.
- Provider argument construction without shell interpretation.
- VFS queue states and safe/unsafe auto-detach decisions.
- Automatic remount readiness and backoff.
- Initial dry preview and confirmation gate.
- Bidirectional create, modify, rename, delete, and conflict behavior.
- Both-version conflict preservation.
- Thirty-day recovery retention and cleanup exclusions.
- Metered-network pause and manual override.
- Interrupted sync recovery without routine resync.
- Google cloud-native document exclusion.
- Legacy import parsing, preview, ownership, and conflicts.
- VPN readiness, shared dependencies, applet-activation tracking, and approved
  shutdown behavior.
- Secret redaction and error mapping.

### 14.2 Isolated integration tests

Tests shall use temporary directories and fakes for systemd, mount tables,
NetworkManager, Cisco, and cloud providers. Local-to-local rclone test remotes may
exercise mount and bisync behavior without real accounts.

Tests shall not write to the real `~/.config/systemd/user/`, activate real VPNs,
or access real cloud data.

### 14.3 Manual acceptance tests

- Online mount with a disposable rclone remote.
- Safe connectivity loss with an empty write queue.
- Connectivity loss while writes are pending.
- Automatic remount after readiness returns.
- OneDrive Online mount with onedriver.
- Offline mirror first-run estimate, preview, and confirmation.
- Offline local editing followed by reconnect synchronization.
- Same-file conflict preserving both versions.
- Deletion propagation and recovery retention.
- Metered-network pause and Sync Now.
- Google cloud-native document exclusion.
- OneDrive Offline mirror with `abraunegg/onedrive`.
- NetworkManager and Cisco VPN readiness.
- Import from `~/.config/systemd/user/`.
- Removal without credential, data, cache, or recovery deletion.

Real accounts, VPNs, and remote writes require explicit user authorization for
each manual test session.

## 15. Version 0.1 Acceptance Criteria

Version 0.1 is acceptable when:

1. The applet builds and runs in the target COSMIC environment.
2. Every provider in the approved matrix supports its specified modes.
3. Offline mirror paths remain responsive without connectivity.
4. Initial synchronization cannot modify data before preview and confirmation.
5. Conflicts preserve both file versions.
6. Deletions propagate and remain recoverable for 30 days.
7. An rclone mount with no pending writes safely detaches after connectivity
   failure and automatically remounts after readiness returns.
8. Pending writes prevent automatic detachment and remain recoverable.
9. Required VPN readiness is verified before mount or sync.
10. Compatible legacy services can be previewed and imported without modifying
    the originals by default.
11. Missing or outdated dependencies produce actionable guidance.
12. Logs, units, configuration, and notifications contain no secrets.
13. Automated tests and approved manual acceptance tests pass.

## 16. Decisions Required Before Development

### 16.1 Resolved decisions

The following decisions are approved and are no longer implementation choices:

1. **Provider scope:** Support only Microsoft OneDrive, Google Drive, Box, and
   SMB using the matrix in Section 3.1.
2. **Access modes:** Include both Online mount and Offline mirror in version 0.1.
3. **Offline synchronization:** Use bidirectional synchronization; propagate
   deletions; preserve both conflict versions; retain deleted/overwritten files
   for 30 days; pause automatic sync on metered networks; preview and confirm
   initial synchronization; skip Google cloud-native documents.
4. **Tool selection:** Use onedriver for OneDrive Online mount,
   `abraunegg/onedrive` for OneDrive Offline mirror, and current stable rclone
   for Google Drive, Box, and SMB.
5. **Legacy imports:** Scan `~/.config/systemd/user/` and allow confirmed import
   of compatible existing rclone and onedriver services while preserving
   originals by default.
6. **Default rclone cache:** Use 20 GiB per Online mount, configurable in advanced
   settings.
7. **Application identity:**
   - Package and binary: `cosmic-ext-applet-mounter`
   - App ID: `io.github.uutzinger.cosmic-ext-applet-mounter`
   - Planned repository:
     `https://github.com/uutzinger/cosmic-ext-applet-mounter`
   - License: MIT
   - Authors: Urs Utzinger and OpenAI Codex
   - Repository creation: performed manually by Urs Utzinger later
8. **Lazy unmount:** Offer lazy unmount only after clean unmount fails and the
   user explicitly confirms the warned action. Queued or in-progress writes
   shall prevent lazy unmount.
9. **VPN shutdown:** Automatically disconnect only a VPN the applet activated,
   and only when no active connection still depends on it.

All design decisions required before development are resolved. No application
code or system configuration shall be created until this specification and
`Task List.md` receive explicit final approval.
