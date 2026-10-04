# Cloud Mounter Applet for COSMIC™

## Requirements and Specifications

This document translates `Applet Description.md`, existing scripts, existing
systemd user services, and archived cloud-drive connection examples into
testable product requirements and a proposed technical design.

`Applet Description.md` is the source of truth when this document and the source description disagree.

## 1. Purpose

The Cloud Mounter Applet manages cloud and network storage connections from the
COSMIC™ desktop panel. Each connection uses one of two mutually exclusive
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
- Detect, create, select, and safely remove unused rclone remotes for Google
  Drive, Box, SMB, and SFTP workflows.
- Generate and manage applet-owned systemd user services and timers.
- Import compatible existing rclone and `jstaf/onedriver` user services.
- Preserve both sides of synchronization conflicts and provide recovery copies.
- Detect missing or outdated dependencies without installing software.
- Keep credentials out of applet configuration, generated units, logs, and
  notifications.

## 3. Current Scope

### 3.1 Supported provider and mode matrix

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | ``jstaf/onedriver`` | `abraunegg/onedrive` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |
| SFTP | `rclone mount` | `rclone bisync` |

Only the providers and engines in this matrix are included. SFTP editor support is implemented; remaining implementation and acceptance
are tracked in Section 17 and
`Task List.md`.

### 3.2 Included

- COSMIC panel icon, popup, and standalone connection settings window.
- Live connection, mount, synchronization, network, and VPN status.
- Add, edit, validate, test, enable, disable, and remove connections.
- Applet-driven rclone remote detection, creation, selection, and confirmed
  removal of unused remotes.
- Online mount lifecycle and health management.
- Offline mirror initialization, scheduling, synchronization, conflict
  preservation, deletion propagation, and recovery retention.
- NetworkManager VPN enumeration and activation.
- Cisco Secure Client detection, agent startup, GUI launch, and readiness
  monitoring.
- Dependency and version checks for rclone, `jstaf/onedriver`,
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
- `rclone bisync` for Google Drive, Box, SMB, and SFTP.

### 4.3 Google cloud-native documents

Google Docs, Sheets, and Slides do not round-trip safely through exported file
formats. Offline mirror mode shall skip these cloud-native documents, report
them to the user, and offer to open them in the browser.

### 4.4 Cisco VPN control

An active Cisco agent does not prove that an authenticated tunnel exists. The
applet may start the agent and open the Cisco UI, but storage operations shall
wait for configured readiness checks.

### 4.5 Settings windows

General Settings, Add, Modify, and Import run in standalone COSMIC
application windows launched from the panel applet. They shall not be implemented as embedded applet child
windows, because embedded child windows may inherit toolkit window titles such
as `Cosmic - Iced` and may not support reliable title updates in the active
libcosmic revision. Add/Modify/Import shall retain the title `Cloud Mounter
Connection Settings`; the new app-wide window shall use `Cloud Mounter
Settings`. This revision does not reintroduce embedded applet child windows.

## 5. User Workflows

### 5.1 Configure a connection

1. The user selects a provider and one access mode.
2. The user selects, detects, or creates a remote account and optional remote
   subtree.
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
   `jstaf/onedriver` services.
2. It parses only a documented safe subset of unit syntax and arguments.
3. It shows an import preview, unsupported options, and target conflicts.
4. The user confirms creation of an applet-managed replacement.
5. The original service is preserved unless the user separately confirms
   disabling it.

## 6. Functional Requirements

### 6.1 Panel and status

- **FR-001:** The applet shall provide a COSMIC panel icon and popup.
- **FR-001A (layout revision, September 15, 2026):** The popup shall place a
  clickable Settings gear at the right edge of the title row, with Cloud
  Mounter aligned to the left. Add Connection, Refresh and sleep options shall be in General
  Settings, with no toolbar or sleep-settings footer in the popup.
- **FR-001B:** The popup empty state shall explain that a connection can be
  added through Settings; the title-row gear shall remain available.
- **FR-001C:** The gear shall open/focus one standalone General Settings window
  titled `Cloud Mounter Settings`. It shall support keyboard focus/activation,
  an accessible Settings label, and a Settings tooltip; provide an icon fallback
  when the theme lacks the gear. Opening it may close the transient popup but
  shall not stop the applet or any runtime subscriptions.
- **FR-001D:** Moving controls shall preserve the popup's shared operation/error
  status area. Settings commands shall report in General Settings; mount/sync
  feedback shall remain visible in the popup, and provider setup/validation
  feedback in the connection editor. A Refresh success message shall not erase
  an unresolved background error or sleep-cleanup warning.
- **FR-002:** The popup shall show every configured connection.
- **FR-003:** Each row shall be optimized for the fixed-width COSMIC applet
  popup. It shall prefer a single-line layout with the connection name as the
  edit entry point and one right-aligned compact state control. The primary
  control shall indicate and change the connection state: Online mounts use
  Mount/Unmount semantics and Offline mirrors use Start/Stop background-sync
  semantics. A slider/toggle-style control is acceptable when it preserves
  keyboard accessibility and non-color state indication.
- **FR-003A:** The main popup shall not spend row space on a separate text
  status chip when the same state can be conveyed by the primary control label,
  color, non-color cue, tooltip, and disabled reason.
- **FR-003B:** Provider, mode, local path, remote, and other static connection
  settings shall not be repeated as main-popup chips; they shall be visible in
  the Add/Modify editor opened from the connection name.
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
- **FR-010A:** Connection operations in the popup shall be interactive controls,
  not text-only labels.
- **FR-010B:** The popup shall expose one currently valid primary operation
  control per connection. Online mounts use Mount or Unmount. Offline mirrors
  use Start or Stop for automatic/background synchronization. Less common or
  secondary actions such as Preview, Sync Now, Retry, Repair, Details, and
  Remove shall be available from the Add/Modify or diagnostics workflow rather
  than cluttering the main popup.
- **FR-010C:** Unavailable primary operations shall be disabled with a visible
  reason.

### 6.2 Connection management

- **FR-011:** The configuration UI shall allow add, edit, test, enable,
  disable, and remove.
- **FR-011A:** The Add/Modify wizard shall expose provider and access-mode selection,
  remote/account selection, optional remote subtree selection, and local
  mountpoint or mirror directory selection.
- **FR-011AA:** Add mode shall start with an empty display-name field using
  placeholder or suggested text. Modify mode shall prepopulate fields from the
  existing connection.
- **FR-011AB:** Modify mode shall not allow provider or access-mode changes
  unless a future explicit conversion workflow is implemented. Disabled provider
  and mode controls shall explain this restriction through field help.
- **FR-011B:** The Add/Modify wizard shall expose Online mount options, including manual
  startup by default, optional startup at login, cache size, bounded timeouts,
  retries, bandwidth limits, and safe detach policy.
- **FR-011C:** The Add/Modify wizard shall expose Offline mirror options, including initial
  preview, sync interval, Sync Now, pause/resume policy, metered-network
  behavior, recovery location, recovery retention, conflict behavior, and
  destructive resync confirmation.
- **FR-011D:** The Add/Modify wizard shall expose per-connection VPN dependency
  selection, readiness checks, and applet-activated VPN shutdown behavior.
- **FR-011E:** Per-field help and Add/Modify review shall expose dependency
  inventory, version status, authentication/setup guidance, and upgrade guidance
  without installing or updating dependencies.
- **FR-011F:** Import shall expose legacy service import from
  `~/.config/systemd/user/`, including scan, preview, conflict display,
  replacement confirmation, and optional confirmed disablement of the original.
- **FR-011G:** The configuration UI shall expose confirmation dialogs for destructive or
  data-affecting actions, including initial synchronization, state rebuild,
  resync, removal, disabling imported originals, lazy unmount, and cleanup.
- **FR-011H:** Add/Modify help shall be attached to the relevant input, button,
  choice, action, or chip as a delayed tooltip when the toolkit supports it.
  Tooltip wrappers shall not be nested; each interactive control shall own at
  most one tooltip so help text does not overlay other help text.
- **FR-011I:** Add/Modify shall keep verbose selected-item explanations out of
  the normal form body when the same information can be provided as help text.
  In particular, selected VPN profile details shall be available from the VPN
  profile choice tooltip instead of inline paragraphs.
- **FR-011J:** Add, Modify, and Import shall open in a standalone COSMIC
  settings application window titled `Cloud Mounter Connection Settings`, not as
  embedded applet child windows.
- **FR-011K:** In Add mode for Google Drive, Box, SMB, and SFTP, Detect rclone
  remotes and the provider-specific Create Remote action shall appear in the
  top action row. These remote-creation actions shall not appear while modifying
  an existing connection. Test Connection and Save Connection shall use
  non-primary visual styling until a selected existing remote is detected or a
  remote has been created and selected.
- **FR-011L:** In Add mode for Google Drive, Box, SMB, and SFTP, detected rclone
  remote choices shall wrap across bounded rows at the default settings-window
  width rather than clipping horizontally.
- **FR-011M:** In Add mode for Google Drive, Box, SMB, and SFTP, the UI shall provide
  an advanced rclone management area for unused remotes. It shall prevent
  removal of remotes referenced by saved connections, require explicit
  confirmation before deletion, and state that deletion changes rclone
  configuration rather than only applet configuration.
- **FR-011MA:** Google Drive, Box, SMB, and SFTP remote-name help shall explain both
  entering a new name before using the provider's Create Remote action and
  selecting/entering an existing remote name from `rclone config`. SMB help
  shall identify its Create/Update action and distinguish updating an existing
  remote from creating a new one.
- **FR-011N:** Import shall not be part of the default Add Connection action
  row. Legacy import shall remain a dedicated workflow or advanced entry point.
- **FR-012:** Every connection shall have a stable generated UUID.
- **FR-013:** A connection shall use exactly one access mode.
- **FR-014:** A path used as an Online mountpoint shall not be used as an
  Offline mirror directory, and vice versa.
- **FR-015:** The applet shall reject duplicate, nested, unsafe, or unsupported
  local targets where overlap could cause recursion or data loss.
- **FR-016:** New Online mounts shall start manually by default.
- **FR-017:** The user may enable an Online mount at login.
- **FR-018:** Removing a connection shall remove the applet configuration record
  and matching applet-owned generated units only. It shall preserve provider
  credentials, cloud data, local mirror data, caches, recovery data, and
  external services unless separate cleanup is explicitly confirmed.
- **FR-018A:** Unit removal shall verify the applet ownership marker and matching
  connection UUID before deleting generated files. If user-manager reload fails,
  the applet shall restore the unit file where possible to avoid an inconsistent
  service state.
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

- **FR-027:** Google Drive, Box, SMB, and SFTP shall use an existing or newly
  configured rclone remote.
- **FR-027A:** Applet-driven rclone remote creation shall use fixed validated
  command arguments for the selected provider. Google Drive and Box setup shall
  delegate OAuth to rclone's browser flow. SMB setup shall create the remote
  without storing SMB passwords in applet configuration.
- **FR-027B:** Google Drive remote creation and update shall accept a
  user-provided OAuth client ID and its matching client secret. Both values
  shall be supplied together or both left blank when creating a remote. An
  existing Drive remote shall require both values and shall change only through
  an explicit update action that repeats browser OAuth. The applet shall mask
  the secret, redact both values from command and error text, clear the form
  fields when the operation finishes, and retain neither value in applet
  configuration. Existing remotes without a custom client ID shall remain
  usable. The UI shall tell the user to remount active connections after an
  update.
- **FR-028:** The applet shall enumerate rclone remote names without loading
  credentials into applet state.
- **FR-028A:** The applet may remove an unused rclone remote only after
  confirmation. It shall refuse to remove any rclone remote referenced by a
  saved applet connection.
- **FR-029:** The applet shall verify that a selected remote and subtree exist
  before activation.
- **FR-030:** Rclone and `jstaf/onedriver` authentication shall be delegated to their
  supported setup flows.
- **FR-031:** OneDrive Offline mirror authentication and state shall be delegated
  to `abraunegg/onedrive`.
- **FR-031A:** For OneDrive Offline mirror setup launched by the applet, the
  applet shall first use `abraunegg/onedrive`'s interactive browser OAuth flow
  with its upstream local redirect listener by running against the app-owned
  configuration directory. If automatic redirect capture fails or is unsuitable
  for the user's browser or tenant, the applet shall offer the supported
  non-interactive auth-file fallback: create transient auth URL and response URL
  files, open the generated Microsoft authorization URL for the user, accept the
  final redirect URL in the UI, and write it to the response file. Authorization
  shall not use `--dry-run`; bounded dry-run preview occurs only after
  authentication.
- **FR-031B:** The applet shall prevent concurrent `jstaf/onedriver` Online mount and
  `abraunegg/onedrive` Offline mirror operation against the same OneDrive
  account or an overlapping remote subtree.
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
- **FR-037A:** Before starting an rclone Online mount, the applet shall perform
  a bounded read-only listing of the configured remote and subtree. If
  authentication, authorization, network, or subtree access fails, it shall
  report an actionable error and shall not start the FUSE service.
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
- **FR-044A:** Google Drive recursive VFS refresh requests shall enable
  rclone's fast-list mode. Online mount service commands shall not include the
  ignored `--fast-list` argument. Box and SMB shall not inherit the Google Drive
  refresh behavior. Automated tests shall cover provider selection; live tests
  shall compare ordinary and shared drives, directory-list latency, memory use,
  and API/rate-limit behavior. Before starting an rclone Online mount, the
  applet shall regenerate its owned unit from the current saved connection so
  updated provider options apply without requiring the user to resave the
  connection.
- **FR-044B:** After a Google Drive mount, the applet shall wait a bounded time
  for both the filesystem and its private per-user Unix RC endpoint and submit
  recursive `vfs/refresh` with `fast_list=true` and `_async=true`. The service
  shall explicitly allow unauthenticated commands on that private Unix socket.
  It shall retain the returned job ID and rclone execution ID by connection,
  poll `job/status` through completion, and enforce the configured provider
  maximum. Completion parsing shall inspect the top-level status and every
  string value in `output.result`; a value other than `OK` is a provider error
  even when rclone returns top-level `success: true`. The applet shall report a
  sanitized error. The refresh shall not delay the successful Mount result once
  the filesystem itself is ready.
- **FR-044C:** After a `jstaf/onedriver` Online mount is ready, the
  applet shall start a background traversal that reads directory entries only.
  It shall not intentionally read file contents, follow symbolic links, or
  leave the selected mount tree.
- **FR-044D:** General Settings shall contain a **Preload** section with saved
  enabled and maximum-duration values for Google Drive, OneDrive, Box, and SMB.
  All four providers shall default enabled with a 60-second maximum. Duration
  bounds shall be 5 to 600 seconds. Box shall have a default maximum directory
  depth of 2 and SMB a default maximum directory depth of 3, with validated
  depth bounds from 1 to 10. Google Drive and OneDrive shall not expose depth.
- **FR-044E:** Before a Google Drive connection is unmounted, repaired, removed,
  or stopped for sleep, the applet shall cancel its active rclone refresh with
  `job/stop`. Cancellation and detach shall remain bounded so the refresh cannot
  delay system sleep indefinitely.
- **FR-044EA:** Before a OneDrive, Box, or SMB connection is unmounted,
  repaired, removed, or stopped for sleep, the applet shall cancel its active
  directory walk and wait a short bounded interval for it to finish.
  Cancellation and detach shall remain bounded so preload work cannot delay
  system sleep indefinitely.
- **FR-044F:** The existing notice mechanism shall report whether any provider
  preload completed, stopped at its deadline, or failed. Failure details shall
  remain sanitized. No manual Cancel control is required.
- **FR-044G:** After a Box or SMB Online mount, the applet shall wait for both
  the mountpoint and a usable root directory listing before starting an
  app-managed, directory-only walk. Box shall stop at the configured depth-2
  default and SMB at depth 3. The walk shall preserve directory-cache progress
  when its deadline expires and shall not use recursive `vfs/refresh`, fast-list,
  or intentionally read file contents.
- **FR-044H:** Each SMB Online connection shall default to **Use global preload
  settings**. Its editor may instead save connection-specific overrides for
  enabled state, maximum duration, and maximum depth. Overrides shall apply only
  to that connection and shall not mutate the global SMB defaults. Other
  providers with preload support shall use their provider-wide settings.
  SFTP has equivalent per-connection overrides, as specified in FR-SFTP-015.
- **FR-044I:** Configuration migration shall preserve the existing OneDrive
  60-second value, initialize missing Google Drive, Box, and SMB settings to the
  documented defaults, and initialize every existing SMB connection to use the
  global policy.

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
- **FR-064:** Import shall support compatible rclone mount and `jstaf/onedriver` units.
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
- **FR-082A:** NetworkManager VPN detection shall import existing profile
  references by UUID and display name without storing VPN credentials. Detection
  shall tolerate normal newline-separated `nmcli` output and applet-session
  output where records may be flattened into a single line.
- **FR-082B:** Repeated VPN detection shall update known NetworkManager and Cisco
  references rather than creating duplicates.
- **FR-083:** NetworkManager activation and status shall use D-Bus when
  practical, with a documented fixed-argument fallback.
- **FR-084:** Cisco support shall detect its agent, GUI, interface, and tunnel
  state separately.
- **FR-084A:** Cisco support shall import an applet reference to Cisco Secure
  Client availability without storing Cisco credentials, account names, or
  authentication material.
- **FR-085:** Each dependency shall support readiness checks using one or more of
  NetworkManager state, interface, route, DNS, or endpoint reachability.
- **FR-086:** Mounting or synchronization shall start only after the tunnel is
  connected and all configured readiness checks pass. Timeout shall fail the
  attempt without starting storage.
- **FR-087:** The UI shall report when interactive Cisco authentication is
  required.
- **FR-086A (A):** Use one monotonic overall activation/readiness
  deadline using the saved profile timeout (Cisco default 90 seconds,
  NetworkManager default 30 seconds). Include interactive password/MFA time.
  Launch the Cisco GUI with lifecycle handling suitable for an interactive
  application; the short command-runner timeout shall not kill its window. Coordinate the
  NetworkManager activation wait with this deadline so a shorter CLI timeout
  does not end authentication early. Short probe timeouts shall remain separate.
- **FR-086B (A):** Existing saved positive timeouts shall be preserved;
  do not increase Cisco’s default to mask premature GUI termination. Detect VPNs shall
  preserve user timeout and readiness settings when refreshing references.
- **FR-087A (planned A):** Show waiting, connected/checking, timeout and failed
  states, elapsed/remaining time, Cancel and Retry. Cancel shall prevent a late
  activation result from starting storage. Transient probe failures shall retry
  within the deadline; explicit authentication rejection shall be actionable.
  Passwords and MFA remain in the external VPN client. Concurrent requests for
  one profile shall share activation and avoid duplicate authentication prompts.
- **FR-088:** The applet shall reference-count connections using the same VPN.
- **FR-089:** No VPN still required by a mounting, mounted, previewing, or syncing
  connection shall be disconnected.
- **FR-090:** The applet shall automatically disconnect a VPN only when the
  applet activated it and no mounting, mounted, previewing, or syncing connection
  still depends on it.
- **FR-090A:** The Add/Modify VPN section shall offer No VPN, detected or
  configured NetworkManager profiles, and detected Cisco Secure Client
  dependencies as mutually exclusive choices. Visible group headers are not
  required; NetworkManager profiles and Cisco choices may be arranged in one
  compact selector when space allows.
- **FR-090B:** The Add/Modify VPN section shall show the
  disconnect-when-unused control only when a VPN dependency is selected.
- **FR-090C:** Detect VPNs shall report success or failure through the app's
  normal notice path. It shall not leave a detection transcript, detected-profile
  list, or debug log text in the VPN section body.

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

### 6.11 Sleep preparation and wake (B; live acceptance pending)

- **FR-097:** Provide an app-wide “Unmount when sleep” setting in General
  Settings, default false and persisted across restarts. Its help shall state
  that only Online connections are affected. The September 15 layout revision
  supersedes placement below the popup connection list. Keep this control out
  of both the main popup and Add/Modify connection settings. Scope includes
  applet-owned Online mounts, including disabled connections with a remaining
  live mount;
  unrelated system/removable mounts are outside its scope.
- **FR-098:** Subscribe to host logind `PrepareForSleep`. While enabled, acquire
  a `sleep` inhibitor in `delay` mode before sleep is requested, and keep its
  file descriptor open. Read `InhibitDelayMaxUSec`; signal handling alone is
  insufficient. Use one cleanup deadline below that host limit, with a release
  margin. Do not change host sleep policy or use an indefinite block inhibitor.
- **FR-099:** On preparation, atomically suspend new mounts, automatic recovery,
  Online-mount VPN activation, cancel queued Online-mount starts, and snapshot
  active Online connections. Offline mirrors and their sync jobs are unaffected.
  Coordinate in-flight operations so late completion cannot mount during
  cleanup. Unmount managed Online mounts with bounded concurrency under the
  shared deadline. Keep dependent VPN connectivity available during cleanup; retain the existing ownership and
  shared-use rules for any subsequent VPN shutdown.
- **FR-100:** Preserve pending-write caches and mirror data. Do not force/lazily
  detach automatically or claim data is flushed without provider evidence.
  Verify service state and mount-table disappearance without traversing a
  potentially hung mount. Record busy mounts, unknown flush state and failures;
  release the delay inhibitor on completion or deadline even if cleanup fails.
  Bound the actual service stop jobs as well as command waits: timing out a
  `systemctl` client does not cancel its already queued systemd job.
- **FR-101:** On wake or canceled sleep, reconcile mount/service state, report
  incomplete cleanup once, and reacquire the delay inhibitor. A separate
  “Restore after wake up” setting in General Settings defaults false. Show it
  disabled while Unmount when sleep is off, retain its saved value, and explain
  that only successfully cleaned Online connections are restored.
  Keep cleaned-up connections suspended until manual restart unless restoration
  is enabled; do not alter their saved login policy. Restoration shall use the
  pre-sleep snapshot of successfully cleaned connections, respect later user
  disables/removals, and wait for network and the full VPN readiness gate. Repeated sleep signals shall be idempotent.
- **FR-102:** Implement the listener in the applet runtime for native and Flatpak
  installs. Grant Flatpak access only to the logind system-bus name for this
  feature; retain the inhibitor descriptor in the applet throughout cleanup.
  Continue storage commands through the existing host-runner boundary. Use
  app-owned Online-service drop-ins under the host user runtime systemd
  directory, with narrowly scoped Flatpak filesystem access. Verify effective
  normal-stop settings before enqueueing a stop; refuse overridden unsafe
  settings. Runtime stop protection remains for the user runtime session.
  Avoid a root sleep hook controlling user services. Surface
  unavailable logind, denied inhibitor access, listener failure, or an insufficient
  cleanup window as degraded protection, and verify listener lifecycle when the
  applet exits or the setting changes.

Implementation reference: systemd's [inhibitor lock documentation](https://github.com/systemd/systemd/blob/main/docs/INHIBITOR_LOCKS.md).
The host's delay is finite; this is best-effort cleanup, not a guarantee that
OneDrive caused the reported suspend hang or that every busy mount will detach.

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
| Google Drive, Box, SMB, SFTP Online mount | Current stable `rclone` and FUSE 3 |
| Google Drive, Box, SMB, SFTP Offline mirror | Current stable `rclone` with required bisync safety features |
| OneDrive Online mount | Current supported `jstaf/onedriver` |
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
| `providers` | rclone, `jstaf/onedriver`, and `abraunegg/onedrive` adapters |
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

### 9.4 Flatpak execution architecture

The native source and Debian installations remain the primary execution model
for version 0.3. Native installations run approved external commands directly
through the typed command-runner boundary.

Flatpak installation shall be treated as an additional packaging target, not as
a replacement for the native package. Because the applet manages host storage,
host FUSE mounts, host user systemd units, existing host rclone configuration,
and host VPN state, a sandbox-only design is not acceptable. The Flatpak build
shall add an explicit Flatpak runtime mode that routes approved host operations
through `flatpak-spawn --host`.

The Flatpak command-runner mode shall preserve the current safety model:

- fixed executable identities rather than arbitrary shell commands;
- separate validated arguments;
- no `sh -c` or command concatenation;
- bounded stdout and stderr capture;
- timeout and cancellation behavior equivalent to the native runner;
- redaction of secrets, OAuth URLs, tokens, and passwords before display or
  logging.

The following resources are host resources and shall not be silently copied into
a Flatpak-private credential or configuration store:

- the applet's own saved connection list and VPN profile references;
- existing rclone remotes and credentials;
- `jstaf/onedriver` and `abraunegg/onedrive` authentication state;
- generated user systemd services and timers under the host user session;
- user-selected mountpoints, mirror directories, cache directories, and
  recovery directories;
- NetworkManager and Cisco VPN state.

Flatpak installations shall use the same user-visible applet configuration as
native source and Debian installations, or shall migrate to that configuration
with explicit user-visible behavior. A user who switches between native and
Flatpak packaging shall not be forced to recreate saved connections solely
because Flatpak changed `XDG_CONFIG_HOME`. The implementation may accomplish
this with a narrow host-side helper/bridge for configuration and unit writes,
or with a documented host-visible configuration path, but it shall not silently
use an isolated sandbox configuration for normal operation.

Any host-side helper or bridge shall keep the same safety constraints as the
native code path: fixed operations, validated connection IDs and paths, applet
ownership markers for unit changes, atomic writes, no secret disclosure, and no
general-purpose shell execution. The helper shall also make it clear which
state is applet-owned and which state remains provider-owned by rclone,
onedriver, onedrive, NetworkManager, Cisco Secure Client, or systemd.

Flatpak permissions shall start from the accepted COSMIC applet pattern with
Wayland/session access and `--talk-name=org.freedesktop.Flatpak` for
`flatpak-spawn --host`. Broader filesystem access, including
`--filesystem=host`, is permitted only if prototype testing proves narrower
permissions cannot support user-selected storage targets, generated host
services, or host-visible FUSE mounts. Any broader permission must be documented
in the Flatpak manifest notes, README, and submission pull request.

Flatpak publication shall be rejected or postponed if testing shows that the
package cannot expose mounts to ordinary host applications, cannot manage the
intended host user services, silently uses different rclone or OneDrive
credentials or applet connection configuration than the native applet, or
requires unjustified unrestricted host access.

### 9.5 General Settings runtime coordination

- Add a distinct General Settings launch mode and `--app-settings` argument.
  Preserve existing `--settings`/`--add-connection` aliases for the connection
  wizard, avoiding a silent change to scripts or existing launch paths.
- Use one standalone General Settings process, with repeat gear activation
  focusing that window. Reuse existing native/Flatpak executable resolution.
- Add a session-bus runtime interface owned by the panel applet (bus name
  `io.github.uutzinger.cosmic-ext-applet-mounter.Runtime`). It shall support
  applying sleep preferences, Refresh, reading runtime/cleanup status, and
  notification of saved connection changes. Use request/reply completion and
  status updates so General Settings never reports success for a request that
  did not reach the owner. Poll status every two seconds while Settings is open;
  background polls shall not disable or visually blink Refresh or the sleep
  controls. Disable controls for user-requested actions and unavailable runtime
  state as appropriate. Discard replies from polls predating a user action,
  including stale errors, even if the action finishes first.
  Serialize preference writes and acknowledge saved preferences separately
  from the listener's readiness status. Reserve both Runtime and Settings bus
  names without replacement to prevent duplicate owners/listeners. If multiple
  panel instances contain the applet, non-owning instances shall use the
  existing runtime service as clients and shall not report the expected name
  ownership collision as a communication failure. Check the corresponding Flatpak bus grants during
  implementation; do not assume a separately launched process shares memory.
- The runtime owner shall reload the latest document, validate and persist only
  the requested settings, update its in-memory config and subscriptions, then
  acknowledge the change. Preserve unrelated fields, defaults and current keys
  `unmount_before_sleep` and `restore_after_wake`. Preload updates shall apply
  one complete validated provider policy at a time so concurrent settings
  replies cannot combine stale enabled, duration, or depth values.
- Refresh shall retain configuration reload and asynchronous VPN/status checks
  in the running applet and return completion/failure to General Settings. It
  shall not mount, unmount, sync, or replace an in-flight operation as a side
  effect. Preserve unresolved errors separately from the Refresh result.
- Add Connection shall reuse the existing wizard launch path. On a successful
  save, notify the runtime owner to reload the connection list; retain popup
  reopen/reload as a recovery path if the owner is temporarily unavailable.
- General Settings shall not start another sleep listener or execute duplicate
  mount operations. If the runtime owner is unavailable, show that explicitly
  and do not silently claim that runtime settings/Refresh were applied.
- Closing General Settings or the editor shall not exit the applet, cancel
  unrelated work, or release the applet-owned inhibitor. Keep settings-action
  notices distinct from popup runtime notices and editor-local notices.

## 10. Configuration Model

Configuration shall use app ID
`io.github.uutzinger.cosmic-ext-applet-mounter` and a versioned COSMIC
configuration namespace.

```text
AppConfig
  version
  notifications_enabled
  unmount_before_sleep: bool = false  # planned B
  restore_after_wake: bool = false    # planned B
  preload:
    google_drive: { enabled: true, maximum_seconds: 60 }
    onedrive: { enabled: true, maximum_seconds: 60 }
    box: { enabled: true, maximum_seconds: 60, maximum_depth: 2 }
    smb: { enabled: true, maximum_seconds: 60, maximum_depth: 3 }
    sftp: { enabled: false, maximum_seconds: 30, maximum_depth: 2 }
  connections[]
  vpn_profiles[]

Connection
  id: UUID
  name
  provider: OneDrive | GoogleDrive | Box | Smb | Sftp
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
  smb_preload_override?:
    enabled
    maximum_seconds: integer 5..600
    maximum_depth: integer 1..10
  sftp_preload_override?:
    enabled
    maximum_seconds: integer 5..600
    maximum_depth: integer 1..10

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

Google Drive post-mount directory warm-up:

```text
mountpoint --quiet <local mountpoint>
rclone rc --unix-socket <connection socket> rc/noop
rclone rc --unix-socket <connection socket> vfs/refresh recursive=true fast_list=true _async=true
```

The managed rclone service enables RC commands without authentication only on
its connection-specific Unix socket below the user's private runtime directory.
The applet retains the returned RC job ID and rclone execution ID, polls
`job/status`, and uses `job/stop` to cancel it before unmount, repair, removal,
or sleep cleanup. Status parsing treats non-`OK` values below `output.result` as
provider failures even if rclone marks the RC job successful. The mount
operation succeeds independently of this background job.

OneDrive, Box, and SMB post-mount directory preload:

```text
find -P <mountpoint> -xdev [-maxdepth <Box or SMB depth>] -type d -printf ""
```

The applet shall construct this as a typed command rather than shell text. It
shall wait for the mountpoint and root listing to become usable, enforce the
effective provider or SMB-connection deadline, retain incremental VFS metadata
on timeout, and cancel the child before detach. OneDrive traverses without a
depth limit. Box defaults to depth 2 and SMB to depth 3. Box and SMB shall not
use recursive `vfs/refresh`.

All preload mechanisms (Google Drive recursive refresh and OneDrive, Box, SMB,
and SFTP directory walks) shall use consistent, brief outcome notices readable
before automatic dismissal. Show completion (including requested depth where
applicable), time limit reached with an incomplete scan, or skipped. Omit configured
and elapsed times and routine browsing/exclusion explanations from completion and
timeout notices. Elapsed time may exceed the
limit while work stops; strict wall-clock termination is not an acceptance
requirement. Report excluded SFTP targets as skipped, never completed. Capture
report settings from the running job rather than rereading settings on completion.
Successful traversal is not proof of faster browsing; validate usefulness through
repeatable fresh-mount comparisons with preload disabled and enabled.

### 11.2 Rclone Offline mirror

The generated service and timer shall:

- use one dedicated bisync work directory per connection;
- use read-only setup access validation plus supported resilient/recovery
  features;
- prevent concurrent runs;
- preserve conflict losers rather than deleting them;
- use recovery directories for deleted or overwritten files;
- date and mark new local and remote recovery batches, preserve each version,
  and prune only applet-owned batches after at least 30 days; legacy unmarked
  recovery data shall remain untouched until reviewed;
- skip cloud-native Google documents for Google Drive;
- run every 15 minutes while readiness permits;
- preserve state after interruption;
- renew locks held by active bisync runs and allow locks left by interrupted
  runs to expire after a bounded interval, without removing a live run's lock;
- never add routine `--resync` behavior.

### 11.3 OneDrive services

- Online mount shall use `jstaf/onedriver`'s supported user-service behavior.
- Offline mirror shall use a dedicated `abraunegg/onedrive` configuration and
  sync directory per connection.
- OneDrive monitor mode shall be used when supported.
- Destructive resync/state rebuild options require explicit reviewed recovery.

### 11.4 Ownership

Every generated file shall include an applet-managed marker and connection UUID.
Existing unmarked units remain external until explicitly imported.

### 11.5 Rclone remote management

Rclone remotes are owned by rclone configuration, not by the applet
configuration. The applet may create and remove remotes only through fixed
`rclone config` commands with validated remote names.

Remote deletion shall:

- require a detected remote name;
- reject names referenced by saved applet connections;
- require explicit confirmation;
- run `rclone config delete <remote>`;
- preserve applet connections, local data, cloud data, caches, and recovery
  directories.

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
- Manual Online unmount shall show in-progress status and, when preload is
  active, indicate that background preload is stopping first. Directory-walk
  subprocesses shall be cancelled and reaped before clean detach. If they do
  not exit within the bounded wait, leave the mount service running and report
  that unmount can be retried. Attempt clean FUSE detach while the service is
  still available, preserving a usable mount if another process holds it busy.
  Retry a transient busy result briefly after preload cancellation, and use the
  mount table to resolve a helper error after the mount has already detached.
- A successful service-stop command is not sufficient evidence of unmount
  success. The mount-table entry shall disappear before the applet reports
  completion or disconnects an applet-started VPN. A stopped or failed service
  with a lingering mount entry shall report Error and offer the confirmed
  Repair flow. Unmount shall not recreate or prepare the mountpoint. Repair
  shall reset systemd failed state only when the service is actually Failed;
  an already inactive/successful service shall not make a successful lazy
  detach appear to fail.
- If clean unmount fails, the applet may offer lazy unmount only after explicit
  user confirmation and a clear warning.
- Queued or in-progress writes shall prevent lazy unmount.
- The applet shall disconnect only a VPN it activated, and only after no active
  connection still depends on it.

## 13. UI Specification

### 13.1 Panel popup

- Title row with Cloud Mounter aligned left and a clickable Settings gear
  aligned to the right edge,
  followed by active connection count, notification state and VPN summary.
- A current notice/result area below the header when an operation has something
  useful to report.
- No Add Connection/Refresh toolbar or sleep-settings footer. Put these in
  General Settings and restore the connection list space they previously used.
- Use a continuous themed background for header and connections. Separate
  status/notices from the connection list with a thin horizontal line and no
  empty background band.
- Scrollable connection rows sized for the fixed-width COSMIC applet popup.
- Each row is a compact card:
  - preferred layout: one line with the connection name on the left and a
    compact primary state control on the right;
  - if the connection name is too long, it may wrap or elide, but the primary
    state control remains reachable and consistently sized.
- The connection name is clickable and opens the Add/Modify editor for that
  connection.
- The main popup shall not repeat static connection settings such as provider,
  mode, local path, or remote; those fields are shown in Add/Modify.
- Online mounts expose Mount or Unmount as the primary operation.
- Offline mirrors expose Start or Stop as the primary operation for
  background synchronization.
- Preview, Sync Now, Retry, Repair, Details, and Remove are kept out of the
  main popup and belong in the Add/Modify or diagnostics workflow.
- Disabled or unavailable primary controls explain why the action cannot
  currently run.
- Empty state directs the user to Settings > Add Connection. The gear remains
  visible without duplicating Add Connection in the popup.
- Import previews shall not replace the connection list. Imported candidates
  remain previews until the user confirms creating applet-managed connections.
- Popup actions dispatch typed operation requests and return immediately without
  blocking the COSMIC event loop.
- The popup shall grow to fit configured rows up to a bounded maximum height.
  Scrolling shall engage only when the configured rows exceed that bound.

### 13.1A General Settings

- Open/focus using the popup title-row gear. Use a standalone window with an
  ordinary close control and title `Cloud Mounter Settings`.
- Present both sleep toggles together under Sleep. Use short labels “Unmount
  when sleep” and “Restore after wake up”; help shall explain Online-only scope
  and restoration eligibility. The second toggle remains visible but disabled
  while the first is off, so changing the first does not resize the window.
- Place Add Connection and Refresh together in the top row of the window,
  above the Preload and Sleep settings and status/feedback areas.
  Add Connection launches the existing editor rather than replacing General
  Settings with connection-specific fields. Refresh updates the runtime owner.
- Add a **Preload** section with separate Google Drive, OneDrive, Box, SMB, and
  SFTP rows, placing SFTP immediately below SMB. Each row contains an enabled control and maximum seconds. Box and SMB
  also contain maximum depth. Populate controls from saved values and show
  defaults of 60 seconds for all providers, depth 2 for Box, and depth 3 for
  SMB. Keep controls visible when disabled so toggling a provider does not
  resize the window. Keep each provider's switch, duration field, and optional
  depth field on one horizontal row. Explain maximum duration and recursive
  depth, including their accepted ranges, through delayed hover help instead of
  persistent captions or a standalone range line. Duration help shall state
  that browsing remains available while preload runs. Depth help shall explain
  that deeper scans increase server requests, can trigger provider rate limits,
  and may consume the available preload time. Provider and sleep toggles shall
  have visible spacing between their text labels and switches.
- Use the same COSMIC themed list surface as the popup and Add/Modify content,
  including the area behind hover help; do not hardcode a background color.
- Place Refresh results, Add Connection launch errors and connection-to-applet
  notices directly below the top button row, before Sleep and wake.
- End the sleep options with “Applies only to Online connections.” Display sleep
  preference feedback and cleanup status as ordinary wrapping text immediately
  below it, without a “Sleep cleanup status” heading or fixed-height status box.
  Keep these messages separate from top-row action notices. Surface unresolved
  sleep problems briefly in popup status even when Settings is closed.
- Persist toggles immediately with validated writes acknowledged by the owner;
  display save failures without presenting an unsaved setting as effective.
- All controls shall remain reachable with keyboard navigation, long translated
  labels, display scaling and small windows. Long status details shall wrap in
  the window’s overall scrollable content; do not reserve a separate status box
  or reduce the main popup list height for hidden settings. Pad content inside
  the scrollable region so its vertical scrollbar remains at the right edge of
  the window.

### 13.2 Add, Modify, Import, and Field Help

- Add Connection, reached from General Settings, opens the existing connection
  wizard. App-wide General Settings and the per-connection editor are distinct.
- Modify opens the same wizard prefilled for an existing connection.
- Add mode instruction text shall describe the required choices: provider, mode,
  remote/subtree, local target, VPN, and startup policy. The same notice area
  shall be reused for validation results and operation feedback.
- Modify mode shall show connection actions at the top. Most connection types
  may use one action row. OneDrive Offline mirror and other action-heavy modes
  may split actions into two rows to avoid clipping.
- An SMB Online connection shall show **Use global preload settings**, enabled
  by default. Disabling it reveals or enables that connection's preload toggle,
  maximum seconds, and maximum depth. Help text shall explain that the override
  is useful when home, LAN, corporate, and VPN shares have different sizes and
  latency.
- The wizard step order is:
  1. Choose provider: OneDrive, Google Drive, Box, SMB, or SFTP.
     Place SFTP immediately to the right of SMB with matching button styling.
  2. Choose access mode: Online mount or Offline mirror. The provider/mode
     matrix determines the engine.
  3. Choose account or remote. OneDrive Online uses `jstaf/onedriver`; OneDrive Offline
     uses `abraunegg/onedrive`; Google Drive, Box, SMB, and SFTP use rclone remotes.
  4. Choose whole remote or remote subtree.
  5. Choose local target: mountpoint for Online mount or mirror directory for
     Offline mirror. Mountpoints and mirror directories are not interchangeable.
  6. Configure mode-specific behavior.
  7. Configure optional VPN dependency.
  8. Review dependency status, generated units, sync/mount preview, and safety
     warnings.
  9. Confirm creation or update.
- Online mount fields include display name, enabled state, manual startup by
  default, optional startup at login, rclone VFS cache limit with 20 GiB default,
  timeout and retry bounds, bandwidth limits, safe detach behavior, and
  lazy-unmount confirmation policy.
- Offline mirror fields include display name, enabled state, whole-drive or
  subtree selection, disk-space estimate, initial-sync preview, sync interval,
  manual Sync Now availability, metered-network default pause, per-connection
  metered override, recovery location, 30-day retention, conflict preservation,
  and resync/state-rebuild confirmation.
- VPN configuration is per connection. The user can choose no VPN, a detected or
  configured NetworkManager VPN profile, or a detected Cisco Secure Client
  dependency. The applet may import these as applet VPN profile references, but
  it does not create or edit VPN profiles and does not store VPN credentials.
  The VPN selector presents No VPN, NetworkManager VPN profiles, Cisco Secure
  Client dependencies, and Detect VPNs compactly; visible NetworkManager/Cisco
  group headers should be avoided when the choices are self-explanatory. The
  Detect VPNs action imports or updates references and reports completion
  through the normal app notice path; it shall not leave a detection transcript
  or detected-profile list in the VPN section body. VPN fields include readiness
  checks, timeout, whether the applet may activate the VPN, and the rule that
  the applet may only disconnect VPNs it activated. The
  disconnect-when-unused control appears only after a VPN dependency is
  selected.
- Import opens a dedicated import workflow. It scans `~/.config/systemd/user/`
  by default, previews compatible rclone and `jstaf/onedriver` units, maps each preview
  into the Add/Modify wizard fields, shows unsupported options and conflicts,
  and requires explicit confirmation before creating an applet-managed
  connection. Originals are preserved by default; disabling an original is a
  separate confirmed action.
- In Add mode for rclone-backed providers, remote controls include:
  Detect rclone remotes, provider-specific Create Remote, detected remote
  choices, and a management area for removing unused remotes. Remote creation
  and removal controls shall not appear while modifying an existing connection.
  Detected remote choice buttons shall wrap into bounded rows at the default
  settings-window width. Remote-name help shall explain new-name entry before
  Create and detected/exact existing-name selection (FR-011MA). SMB help shall
  explain Create/Update. Since Modify hides creation actions, its help shall
  describe existing-name use without directing users to a hidden Create button.
- OneDrive setup controls shall use user-facing guidance: complete required
  fields, finish browser authentication, then run Test Connection and Save
  Connection. Implementation details about credential file locations belong in
  documentation and diagnostics, not normal form text.
- Per-field help opens concise guidance near the relevant setting. The preferred
  behavior is COSMIC/libcosmic hover tooltips attached directly to the relevant
  field, button, choice row, status chip, or VPN chip. Visible help buttons are
  not part of the default design and shall be reserved only for future controls
  that cannot reasonably carry their own tooltip. Tooltips shall be positioned
  above or to the side of the source control so they do not obscure the field or
  button being explained. Tooltips shall appear after a short delay,
  approximately one second, so ordinary pointer movement does not constantly
  open help. Tooltip wrappers shall not be nested around controls that already
  have their own tooltip. Selected-item details, such as NetworkManager or Cisco
  VPN external profile behavior, readiness checks, timeout, and activation
  expectations, belong in the relevant choice tooltip rather than as inline body
  text. Longer dependency, safety, setup, and troubleshooting guidance belongs
  in documentation rather than a standalone main-popup Help control.
- Destructive or data-affecting actions use explicit confirmation dialogs with
  the action, target paths, preserved data, and expected consequences.
- Controls shall be keyboard accessible, localized, and understandable without
  color.

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

Planned A/B acceptance coverage is tracked in `Task List.md`, section “VPN
Authentication Wait and Sleep Cleanup”: fake-clock VPN deadlines/cancellation,
sleep lifecycle and service-stop races, plus native/Flatpak live acceptance.


- Configuration serialization, migration, and invalid-data recovery.
- Mode/path overlap and recursive-sync validation.
- Dependency version and capability checks.
- Deterministic service and timer rendering.
- Provider argument construction without shell interpretation.
- Rclone remote detection, creation request construction, duplicate-name
  rejection, confirmed removal, and in-use removal blocking.
- Google Drive custom OAuth client pair validation, redacted create/update
  command construction, explicit existing-remote update, and form clearing.
- VFS queue states and safe/unsafe auto-detach decisions.
- Automatic remount readiness and backoff.
- Google Drive refresh-level fast-list selection, private Unix RC readiness,
  background VFS refresh lifecycle, nested-result error handling, deadline, and
  cancellation.
- OneDrive, Box, and SMB directory-only traversal boundaries, root readiness,
  provider defaults, depth limits, configurable deadlines, incremental timeout
  behavior, and cancellation before unmount or sleep.
- SMB global-policy inheritance and independent per-connection override
  serialization, validation, migration, and effective-value selection.
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

- Open/focus General Settings using the title-row gear, including an empty
  popup, and open the existing Add Connection wizard from that window.
- Verify that the popup contains only title/gear, status/notices and connections;
  enabling sleep options does not add popup controls or shrink the list.
- Verify Refresh from General Settings updates the actual applet, including
  asynchronous VPN status, without clearing unrelated unresolved errors.
- Change sleep settings with the popup closed and confirm one runtime listener
  updates immediately; closing settings shall leave that listener active.
- Verify gear focus, icon fallback, standalone title, duplicate-window handling,
  runtime-owner unavailability, native/Flatpak IPC, and save/reload propagation.
- Verify mount/unmount/repair failures remain visible in popup status, while
  remote creation, authentication, detection and save feedback remain in the
  connection editor. Check sleep warnings with General Settings closed.
- Verify Google Drive, Box and SMB remote-name help covers creation and existing
  remote selection in Add mode and avoids hidden-action instructions in Modify.
- Expire or invalidate a disposable rclone OAuth credential and verify Mount
  fails before FUSE starts, reports reauthorization guidance, and leaves the
  local mountpoint responsive.
- Place Cloud Mounter on two panels and verify one runtime owner serves both
  instances without a Settings communication warning or duplicate sleep listener.
- For Google Drive, compare cold first-level and nested browsing with a
  recursive fast-list VFS refresh; verify the refresh remains in the background
  and is cancelled by unmount and sleep. Inject a top-level successful RC job
  containing a provider error in `output.result` and verify it is reported as a
  failure.
- For OneDrive, verify the directory-only preload stops at the configured
  60-second default, does not read file contents, and is cancelled before
  unmount and sleep. Check completion/timeout notices only if implemented.
- For Box, verify depth 2 warms first browsing without HTTP 429, preserves
  partial progress at timeout, and cancels before unmount and sleep.
- For SMB, test at least one LAN and one VPN connection. Verify depth 3 warms
  first browsing, global defaults and per-connection overrides remain
  independent, timeout preserves partial progress, and cancellation precedes
  unmount and sleep.
- Verify each popup row shows a clickable connection name, one primary
  compact state control, and no separate static provider/local/VPN chips.
- Verify popup primary operation controls dispatch Mount, Unmount, Start, or
  Stop requests as applicable.
- Verify Preview, Sync Now, Retry, Repair, Details, Remove, provider, mode,
  local path, remote details, and detailed status are available from Add/Modify
  or diagnostics workflows rather than repeated in the main popup.
- Verify Add/Modify/Import workflows can add, edit, test, enable, disable,
  import, and remove connections without editing files by hand.
- Verify the Add/Modify/Import window title is `Cloud Mounter Connection
  Settings`.
- Verify Add/Modify workflows expose all approved provider, mode, local path, cache, sync,
  metered, VPN, dependency, import, recovery, and confirmation options.
- Verify Add mode starts with an empty display-name field and suggested
  placeholder text.
- Verify Modify mode locks provider and access-mode changes unless a future
  conversion flow is implemented.
- Verify rclone-backed Add mode can detect remotes, create provider-specific
  remotes, wrap detected remote choices, refuse removal of in-use remotes, and
  remove only a selected unused remote after confirmation.
- Online mount with a disposable rclone remote.
- Safe connectivity loss with an empty write queue.
- Connectivity loss while writes are pending.
- Automatic remount after readiness returns.
- OneDrive Online mount with `jstaf/onedriver`.
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
- Unused rclone remote removal without deleting applet connections or local/cloud
  data.

Real accounts, VPNs, and remote writes require explicit user authorization for
each manual test session.

## 15. Current Acceptance Criteria

The current implemented scope is acceptable when:

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
14. The v0.2 popup and Add/Modify UI refinements pass user-guided visual review,
    including compact popup rows, bounded scrolling, action-row layout, help
    tooltip placement, and rclone remote management.

## 16. Decisions Required Before Development

### 16.1 Resolved decisions

The following decisions are approved and are no longer implementation choices:

1. **Provider scope:** Support only Microsoft OneDrive, Google Drive, Box, SMB, and
   SFTP using the matrix in Section 3.1.
2. **Access modes:** Include both Online mount and Offline mirror in version 0.1.
3. **Offline synchronization:** Use bidirectional synchronization; propagate
   deletions; preserve both conflict versions; retain deleted/overwritten files
   for 30 days; pause automatic sync on metered networks; preview and confirm
   initial synchronization; skip Google cloud-native documents.
4. **Tool selection:** Use `jstaf/onedriver` for OneDrive Online mount,
   `abraunegg/onedrive` for OneDrive Offline mirror, and current stable rclone
   for Google Drive, Box, SMB, and SFTP.
5. **Legacy imports:** Scan `~/.config/systemd/user/` and allow confirmed import
   of compatible existing rclone and `jstaf/onedriver` services while preserving
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
10. **Applet-driven rclone setup:** The applet may create Google Drive, Box, SMB, and
    SFTP rclone remotes through fixed validated `rclone config create` commands.
    Provider authentication and SMB/SFTP password storage remain with rclone
    or the selected SSH agent.
11. **Rclone remote removal:** The applet may remove unused rclone remotes from
    rclone configuration only after explicit confirmation and only when no saved
    applet connection references the remote.
12. **Version 0.2 popup UI:** The main popup uses compact connection rows with a
    clickable name and one primary slider/toggle-style state control. Static
    provider, mode, local path, and remote details belong in Add/Modify.
13. **Version 0.2 Add/Modify UI:** Add and Modify share one editor. Modify locks
    provider and access-mode changes, moves secondary actions such as Preview
    and Sync Now into the editor, and uses tooltips for field-specific help.

All current design decisions required for the implemented scope are resolved.
Future feature work shall update this specification and `Task List.md` before
implementation.

## 17. SFTP Provider Extension — September 29, 2026

This section specifies the SFTP extension to the provider matrix. The provider
and Add/Modify editor are implemented; installed-service and live-server
acceptance remain pending. Existing completion evidence for other providers
does not establish SFTP acceptance.

### 17.1 Provider and editor requirements

- **FR-SFTP-001:** Add `Provider::Sftp` and an rclone backend mapping to `sftp`.
  Support Online mount through `rclone mount` and Offline mirror through
  `rclone bisync`, reusing existing lifecycle and data-protection rules.
  Existing configurations shall load without rewriting other providers.
- **FR-SFTP-002:** Add Connection shall place the **SFTP** button immediately
  to the right of **SMB** in provider order. Match SMB's button styling,
  selection feedback, section order, field spacing, tooltips, and action
  placement. Both buttons shall remain adjacent without clipping at the default
  editor width; narrower layouts shall preserve their order and accessibility.
- **FR-SFTP-003:** Show remote name, server host, port defaulting to 22,
  username, authentication choice, known-hosts file, and remote directory.
  Require a host, username, and integer port from 1 to 65535 for applet-created
  remotes. Do not show SMB domain/workgroup inputs. Retain the shared mode,
  local-target, VPN, startup, cache, and mirror controls as applicable.
- **FR-SFTP-004:** Detect and select existing remotes whose backend is `sftp`,
  including remotes configured outside the applet. In Add mode, place
  **Create/Update SFTP Remote** beside the existing detection action in the top
  action row. Follow FR-011K through FR-011MA for selection, wrapped remote
  choices, name help, and protected remote removal. Modify shall preserve the
  provider/mode lock and expose **Update SFTP Remote** for existing remotes,
  matching the current SMB editor. It shall not create missing remotes.
  In Add mode, a new remote's Create/Update action shall use the blue/theme-accent
  suggested style. Changed server/authentication settings shall also highlight
  Update in Add and Modify. Successful application clears the highlight unless
  newer edits exist; failure retains it. Unchanged existing remotes use standard
  styling. This
  explicit update action is an exception to the generic Modify setup-control
  restriction in FR-011K and Section 13.2.
- **FR-SFTP-005:** Support password, private-key file with optional passphrase,
  and SSH-agent authentication. Show only applicable authentication fields.
  Each authentication choice shall expose delayed hover help describing the
  credential source, required server setup, and relevant key/agent limitations.
  Validate key-format support in the installed rclone; encrypted OpenSSH keys
  shall use an SSH agent when direct passphrase loading is unsupported.
  Secrets shall be masked, transient, redacted from command/error output, and
  cleared after setup success or failure. Never persist passwords, passphrases,
  or private-key contents in applet configuration or generated services.
- **FR-SFTP-006:** Require server identity verification through rclone's
  `known_hosts_file`, defaulting to the host user's `~/.ssh/known_hosts`.
  Missing/unreadable files and unknown or changed host keys shall fail with
  actionable guidance. Do not disable verification or silently add/replace keys.
- **FR-SFTP-007:** Preserve SFTP directory semantics: blank means the login
  directory, relative paths remain relative, and leading `/` remains absolute
  within the server account's permitted filesystem. Do not strip the leading
  slash or apply SMB share-name rules. Test, preview, mount, bisync, and import
  shall resolve the same target.

### 17.2 Setup and runtime specification

- **FR-SFTP-008:** Use validated argument-based `rclone config create/update`
  with backend `sftp` and provider options `host`, `port`, `user`,
  `known_hosts_file`, and applicable authentication options. Use the existing
  protected rclone password-handling mechanism for `pass` and `key_file_pass`;
  retain key paths as `key_file`. No shell interpolation or pasted private keys.
  Reject updates to an existing remote of another backend. Preserve unrelated
  remote options and existing secrets when no replacement is requested; an
  authentication-mode change shall explicitly reconcile incompatible options.
- **FR-SFTP-009:** Remote creation/update shall require the explicit setup
  action. Before an update, identify saved connections sharing that remote and
  explain restart implications. Test Connection and Save Connection shall not
  implicitly change remote credentials. The applet connection record shall
  reference the rclone remote and directory; rclone owns persistent setup data.
  Save Connection shall remain blue for SFTP and may persist a connection after
  configuration/plan validation even when access testing has failed or has not
  been performed. This does not imply successful authentication and does not
  apply edited remote credentials.
- **FR-SFTP-010:** Test Connection shall perform a bounded listing of the exact
  selected directory using the configured network/VPN readiness path. Distinguish
  unreachable host/port, authentication failure, host-key failure, missing
  directory, and permission errors without exposing secrets. Mount access
  preflight shall use the same target and authentication context.
- **FR-SFTP-011:** Generated host user services shall use the same host rclone
  configuration, readable key and known-hosts paths, and selected authentication
  as setup. For agent authentication, validate service access to `SSH_AUTH_SOCK`
  and the required unlocked identity. Missing or expired agent access shall fail
  without a hanging prompt. Verify this for native and Flatpak operation; an
  interactive terminal success alone is insufficient.
- **FR-SFTP-012:** Reuse mount status, VFS upload/cache monitoring, network/VPN
  recovery, safe detach, repair, sleep cleanup, and wake restoration. SFTP shall
  not inherit Google Drive recursive refresh or SMB-specific timeout assumptions.
  SFTP preload is optional as specified in FR-SFTP-015; existing provider
  preload defaults remain unchanged.
- **FR-SFTP-013:** Offline mirrors shall retain the existing size estimate,
  preview, explicit initial-sync confirmation, scheduling, metered policy,
  conflict preservation, deletion recovery, and interrupted-sync safeguards.
  Do not require remote shell commands or checksums unavailable on an SFTP-only
  account; validate the selected bisync comparison strategy on that server.
- **FR-SFTP-014:** Extend compatible legacy import, remote-reference protection,
  configuration validation, localized labels/tooltips, and provider-specific
  diagnostics to SFTP without changing existing providers' saved behavior.

- **FR-SFTP-015:** General Settings shall show an SFTP preload row immediately
  below SMB, with an enabled switch, maximum seconds, and maximum depth.
  Defaults shall be off, 30 seconds, and depth 2; accepted bounds shall be
  5–600 seconds and 1–10 levels. Each SFTP Online connection shall inherit the
  global policy by default and may override all three values independently.
  Older configurations shall initialize missing SFTP settings to these defaults
  and missing overrides to inheritance without changing existing providers.
- **FR-SFTP-016:** Enabled SFTP preload shall use the existing cancellable,
  bounded directory-only walk after mount readiness, including restored mounts.
  It shall not read file contents or follow symbolic links. For an absolute
  server-root target, prune `/proc`, `/sys`, and `/dev` before descending. If
  the remote target is itself inside one of those trees, skip preload entirely,
  including readiness probes. Apply exclusions using remote absolute paths;
  ordinary project directories named `sys`, `proc`, or `dev` remain eligible.
  Login-relative paths cannot be classified as server absolute paths from the
  saved configuration alone. Exclusion patterns shall match mountpoint names
  literally, including spaces and glob characters. Cancel through the shared
  lifecycle before unmount, repair, removal, or sleep; report bounded outcomes.

### 17.3 Verification and acceptance

- Automated checks shall cover provider serialization and old-config loading,
  backend filtering, field validation, remote setup/update argument handling,
  secret redaction and clearing, authentication-mode changes, and exact relative
  versus absolute target preservation throughout mount and mirror planning.
- Isolated integration checks shall exercise password/key/agent authentication,
  trusted/unknown/changed host keys, permission failures, bounded timeouts,
  service environment access, and SFTP-only servers without shell/hash commands.
- Desktop acceptance shall verify the SFTP button immediately beside SMB,
  comparable editor layout, conditional fields, keyboard navigation, and no
  clipping in native and Flatpak Add/Modify windows.
- Use disposable server data to verify mount/read/write/unmount, pending-write
  protection, network/VPN recovery, sleep/wake, and Offline mirror preview,
  initial/repeated sync, conflicts, deletion recovery, and interruption recovery.
- Report automated, packaged-service, and live-server evidence separately.
  SFTP support is complete only after the corresponding implementation and
  acceptance tasks have passed; existing SMB results cannot substitute.

Technical reference: [rclone SFTP documentation](https://rclone.org/sftp/)
(authentication, host verification, path semantics, and shell-access limitations).

## 18. SharePoint Engine (including Teams Files) — Implementation Plan, October 1, 2026

SharePoint is a distinct applet engine labelled **SharePoint**, with Online mount implemented
and separate Offline mirror acceptance pending. Its storage is a
SharePoint document library; it does not use the personal OneDrive workflow.
It is not yet a release-supported connection. A Teams standard-channel Files tab points
to a folder within the parent site's document library; private and shared
channels can have their own sites. A connection addresses exactly one verified
site, one document library, and optionally one folder within that library. The
user chooses a separate local mountpoint or mirror directory. For example,
`https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents`
is a proposed library URL, not a local mountpoint or a rclone folder path. The
applet must confirm which library it denotes using authenticated metadata.

### 18.1 Identity, URL input, and account isolation

- Accept HTTPS SharePoint site or library URLs as **locators**, not proof of
  identity. Normalize hostname and supported `/sites/` or `/teams/` paths,
  decode path segments once without treating encoded separators as path
  boundaries, remove query and fragment data, and reject
  malformed, unsupported, or ambiguous sharing/view links. Never infer a drive
  ID solely from `Shared Documents`, a Teams channel name, or a remote name.
- Persist a separate SharePoint identity record with canonical site URL,
  canonical library URL/name, verified document-library drive ID, selected
  rclone remote, and optional folder path. Store no access or refresh token in
  the applet connection. Migrate older configurations without changing existing
  OneDrive, Google Drive, Box, SMB, or SFTP records.
- Resolve the selected drive ID against an authenticated Microsoft drive
  metadata response, checking its library `webUrl` and site association against
  the entered URL. A read-only Graph drive lookup using the selected remote's
  delegated token is the initial candidate; prove that it works with the
  installed rclone version and tenant permissions before implementing the
  editor. Keep the token transient and redacted. If no authenticated identity
  lookup is available, block setup with a useful explanation instead of
  accepting an unverified library. Recheck identity on Add/Modify test and
  before Start when a remote's drive ID or account changes.
- Distinguish missing tenant consent, expired/revoked authorization, wrong
  account or drive, inaccessible site, read-only access, missing subfolder, and
  rate limiting. Access tests may list but shall not write or delete. Only an
  explicit disposable-data test may determine write capability.

### 18.2 First deliverable: Online mount with rclone

- Show a **SharePoint** button beside OneDrive in Add/Modify, with Online mount
  available first. The manual setup path selects an existing rclone remote with `type=onedrive`,
  `drive_type=documentLibrary`, and a drive ID that passes Section 18.1. The
  user selects an optional folder beneath the library root. A separate guided
  Microsoft browser-OAuth path creates a new SharePoint-specific rclone remote from
  the entered library URL, discovers the site's document libraries, and selects
  only an authenticated drive whose type and URL exactly match. A duplicate
  remote name must be rejected; an incomplete new remote must be cleaned up.
  Never rewrite an existing SharePoint or personal OneDrive remote. The preconfigured
  remote path remains usable. Tenant-wide browsing and in-app account/consent
  administration remain separate work.
- Use the verified rclone remote and optional folder with the existing Online
  mount lifecycle: generated user service, bounded access test, cache/status,
  pending-write protection, connectivity and optional VPN readiness, sleep/wake,
  clean unmount, Repair, and removal. Validate target overlap and avoid
  modifying an existing personal OneDrive remote. Changing the selected remote,
  drive ID, or folder requires a new identity and access test before mount.
- Default SharePoint directory preload to **off** because library enumeration
  can be expensive or throttled. If enabled, keep traversal bounded to the
  chosen library/folder, cancellable before unmount, and report partial work
  honestly. SharePoint uses its own policy; Google Drive fast-list and OneDrive
  `onedriver` preload assumptions do not apply automatically.
- Detect rclone, FUSE, a usable document-library remote, and host-service access
  to that remote's credentials. Give actionable errors for missing dependencies,
  token refresh/consent failures, and throttling. Do not import legacy
  `onedriver` or ordinary rclone `onedrive` units as SharePoint merely because
  their URLs or names contain a site; an explicit verified migration is needed.

### 18.3 Second deliverable: Offline mirror with rclone bisync

- Reuse the verified SharePoint rclone `onedrive` document-library remote. The
  separate `abraunegg/onedrive` Microsoft app requires tenant administrator
  approval for the University work account, so it is not a usable default
  engine here. Keep SharePoint bisync listings, lock, local directory, and recovery
  data separate from every Online mount and other mirror. Never copy or expose
  rclone tokens in applet configuration.
- Bind every Preview, Sync Now, and eventual scheduled run to the saved site,
  library URL, document-library type, drive ID, and selected remote/folder.
  Recheck identity before writable operations; a retargeted remote or changed
  folder requires a new preview and explicit state rebuild. Validate local
  target overlap and available space.
- Require a scoped `--resync --dry-run` Preview and explicit confirmation before
  the first Sync Now. The first release of SharePoint Offline shall expose
  manual Preview and Sync Now only; keep its background timer **off** until
  separate unattended network/VPN, metered, sleep/wake, and failure checks pass.
  Preview must
  report uploads, downloads, deletions, conflicts, and recovery location.
  When testing inside an existing work library, limit both remote and local
  paths to a disposable folder; prove that no proposed operation or recovery
  write escapes the approved scope before any test sync.
- Preserve overwritten/deleted files in applet-owned recovery areas. If remote
  recovery cannot be kept within an approved scope without bisync reading its
  own backups, do not run a writable mirror until an explicit safe location is
  chosen and verified. Never create a recovery folder elsewhere in the work
  library as a side effect of Preview.
- On disposable data, demonstrate conflict preservation, deletion and
  overwrite recovery, Office-file post-upload behavior, interrupted sync,
  resync after identity/filter changes, and repeated scheduled runs. If these
  checks cannot establish safe unattended operation, keep scheduling disabled
  and do not claim supported bidirectional Offline mirror behavior.
- For SharePoint's Office-file rewrites, track changes with checksums as well
  as modification times. A writable sync may require one bounded follow-up
  pass to download SharePoint's normalized copy. Report success only after an
  independent checksum comparison finds no differences; preserve prior file
  versions in recovery and report a failed or incomplete comparison honestly.
- Include one stable, hidden, app-owned access marker in the selected local and
  remote folder so a one-file mirror can change without disabling bisync's
  all-files-changed safety stop. Create it locally before initial Preview, show
  its upload in that Preview, and make no remote marker write until confirmed
  initial Sync Now. Reject missing or altered markers on subsequent runs.
- Once the safety gate passes, apply existing network/VPN, metered, sleep/wake,
  status, Stop/Start, and generated-service safeguards. Deleting an applet
  connection removes only applet-owned units; preserve library data and
  credentials unless separately confirmed.

### 18.4 Acceptance boundaries

- Unit and isolated tests cover URL parsing, identity drift, serialization,
  service targeting, error classification, token redaction, and old-provider
  regression. Installed native Online acceptance uses a disposable SharePoint
  library/folder for read, write, pending upload, unmount, repair, and network
  recovery. Verify standard-channel folders and a private/shared channel site
  when access to such a site is available. Record tenant restrictions explicitly.
- Offline acceptance separately covers initial preview and sync, repeated
  bidirectional sync, conflicts, deletion recovery, interrupted transfers,
  selective-folder scope, and the unattended safety gate above. Native and
  Flatpak host-service access must each be tested when their distribution path
  exists; a prototype Flatpak result is not public-package acceptance. In-app
  OAuth/discovery is optional follow-up, not a prerequisite for the first
  preconfigured-remote Online milestone.

References: [Microsoft Teams/SharePoint site relationships](https://learn.microsoft.com/en-us/sharepoint/teams-connected-sites),
[Microsoft Graph site lookup](https://learn.microsoft.com/en-us/graph/api/site-get?view=graph-rest-1.0),
[Graph drive lookup](https://learn.microsoft.com/en-us/graph/api/drive-get?view=graph-rest-1.0),
[rclone Microsoft OneDrive backend](https://rclone.org/onedrive/), and
[rclone bisync safety guidance](https://rclone.org/bisync/).

### SharePoint Online service identity gate — October 2, 2026

Every generated SharePoint Online service shall check the saved connection and
authenticated SharePoint document-library identity before starting rclone. The
check shall compare connection ID, enabled state, remote, optional folder,
mountpoint, verified drive ID, and library URL with the unit's bound values,
then verify the remote's current drive ID, type, and URL through authenticated
drive metadata. Failure shall prevent the mount for direct systemd starts,
restarts, and start at login as well as applet-initiated starts. The applet shall
report a skipped start as blocked rather than successful. Other providers are
unaffected by the SharePoint identity gate. Installed validation shall use a saved unit and a deliberately
mismatched binding without writing to SharePoint.

### Online mountpoint preflight for all engines — October 2, 2026

Before starting any Online mount, including direct service starts and start at
login, check the host mount table and local target. OneDrive, SharePoint, Google
Drive, Box, SMB, and SFTP shall start only when the target is an ordinary empty
directory and is not already mounted. A missing, unreadable, non-directory,
mounted, or non-empty target shall block startup with a useful reason. Preserve
all local entries; never delete, move, or hide them as automatic recovery.
Offline mirrors are ordinary populated local directories and are outside this
mountpoint rule. Refresh existing managed Online service definitions after an
applet update without restarting active mounts.

### Per-connection pending-change count — October 2, 2026

- Each connection row in the main applet popup shall reserve one compact count
  between the clickable name and the Mount/Unmount or Start/Stop switch. The
  count shall not shift or hide the switch when names are long or text scaling
  changes. It shall be plain text with a hover and accessible description,
  using the Fluent catalog for labels and explanations.
- When a reliable count is unavailable, show `+` for an active connection and
  `-` for an inactive connection. These markers are not numeric counts; the
  hover/accessibility explanation shall say why the number is unavailable.
- The number means **pending file changes**, not cache occupancy or bytes. For
  rclone Online mounts (SharePoint, Google Drive, Box, SMB, SFTP), obtain the number
  from that connection's existing private RC socket: count the entries in
  `vfs/queue`, including queued and currently uploading files. Poll at a
  bounded interval while the mount is active, without reading file contents
  or traversing the remote. Cross-check VFS error state. Show `0` only after a
  successful current query with no unresolved upload/cache error; a stopped
  service, unavailable socket, unsupported response, or query failure is
  unknown, not zero. Never sum `diskCache.files` or `bytesUsed` as pending work.
- For Offline mirrors, count pending file operations in the configured sync
  scope: uploads, downloads, deletions, and unresolved conflicts. Derive an
  idle baseline from a bounded read-only preview/status operation, with its
  check time; do not launch a second bisync preview while sync or recovery is
  running. During a run, update the count only from that run's confirmed
  progress, then perform a fresh scoped check after completion. Scheduled and
  manual runs shall use the same reporting path. A new local or remote change
  can increase the count; a failed or interrupted run must not imply zero.
- Use rclone bisync's existing preview path for Google Drive, Box, SMB, and
  SFTP mirrors. Use the configured abraunegg/onedrive client's read-only
  `--display-sync-status` for OneDrive mirrors when supported, interpreting its
  textual status and pending directions rather than its exit code alone.
  SharePoint mirror counting begins only if that engine passes its
  separate Offline mirror safety gate and becomes supported.
- onedriver Online mounts expose no supported pending-upload count. Display
  `+` while mounted or `-` while inactive, with an explanation until a reliable
  engine-supported source exists;
  do not inspect its private cache database or infer pending uploads from
  cached bytes. Use the same active/inactive marker when a mirror count is not yet
  checked, becomes stale, or cannot be determined safely. A previously shown
  number shall not silently persist as current after unmount, sleep, restart,
  disconnection, or a configuration change.
- Keep count collection asynchronous and bounded so popup opening, toggles,
  unmount, and sync are never delayed. An error in counting shall affect only
  the count and its explanation, not the underlying mount or mirror operation.
  Test zero, growing/shrinking queues, upload errors, stale/unavailable
  sockets, active and interrupted mirror runs, scheduled runs, new changes
  after a check, small popup widths, accessibility, and native/Flatpak host
  command paths.
