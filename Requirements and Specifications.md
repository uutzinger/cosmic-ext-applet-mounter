# Cloud Mounter Applet for COSMIC™

## Requirements and Specifications

This document translates `Applet Description.md` into testable product
requirements and a technical design. `Applet Description.md` is the source of
truth, including product status and acceptance-pending scope; if the two
disagree, the description wins.

## 1. Purpose

The applet manages cloud and network storage connections from the COSMIC™ panel.
Each connection uses one of two mutually exclusive access modes: **Online mount**
(a network-backed filesystem with on-demand access) or **Offline mirror** (an
ordinary local directory synchronized bidirectionally with a remote location). A
connection may depend on a VPN that the applet prepares and verifies before
mounting or synchronizing. The applet replaces manually maintained shortcuts,
scripts and service files while keeping systemd user services as the durable
runtime mechanism.

## 2. Goals

- Show every applet-managed connection and its state, and let the user mount,
  unmount, synchronize, pause, resume and inspect each one.
- Prevent cloud connectivity from blocking the file manager in Offline mirror mode,
  and reduce stalls for Online mounts through health monitoring, bounded
  timeouts, safe detachment and automatic recovery.
- Treat the popup switch as the persistent desired state: Online mounts that are
  on return at the next login; stopped mounts and mirrors stay stopped.
- Configure optional NetworkManager VPN dependencies, including Cisco-compatible
  gateways through the OpenConnect plugin.
- Detect, create, select and safely remove unused rclone remotes for Google Drive,
  Box, SMB and SFTP.
- Generate and manage applet-owned systemd user services and timers, and import
  compatible existing rclone and `jstaf/onedriver` services.
- Preserve both sides of synchronization conflicts and provide recovery copies.
- Detect missing or outdated dependencies without installing software.
- Keep credentials out of configuration, generated units, logs and notifications.

## 3. Current Scope

### 3.1 Supported provider and mode matrix

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | `jstaf/onedriver` | `rclone bisync` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |
| SFTP | `rclone mount` | `rclone bisync` |

Only the providers and engines in this matrix are included. SharePoint document
libraries are specified in Section 18. Implementation and acceptance status is
kept in `Applet Description.md` and `Task List.md`.

### 3.2 Included

- COSMIC panel icon, popup, and standalone settings windows.
- Live connection, mount, synchronization, network and VPN status.
- Add, edit, validate, test, enable, disable and remove connections.
- Applet-driven rclone remote detection, creation, selection and confirmed
  removal of unused remotes.
- Online mount lifecycle and health management.
- Offline mirror initialization, scheduling, synchronization, conflict
  preservation, deletion propagation and recovery retention.
- NetworkManager VPN enumeration, activation, deactivation and state monitoring
  over D-Bus, including OpenConnect authentication through a host `nm-applet`
  secret agent when the profile requires it.
- Dependency and version checks for the external tools.
- Import of compatible legacy services from `~/.config/systemd/user/`.
- Localized UI strings and accessible controls.

### 3.3 Non-goals

- Implementing storage, synchronization or VPN protocols in the applet.
- Supporting providers outside the approved matrix.
- Creating or editing VPN profiles, handling VPN credentials, or implementing a
  VPN authentication client.
- Treating an Online mount cache as a complete offline copy.
- Automatically installing packages, adding repositories or updating tools.
- Storing provider or VPN secrets.
- Running arbitrary user-entered shell commands.
- Presenting synchronization as a replacement for backups.

## 4. Technical Clarifications

### 4.1 Online mount limitations

A FUSE filesystem can still make an application wait while the provider, network
or VPN is unavailable. Bounded timeouts and automatic detachment reduce but cannot
eliminate this. The applet shall recommend Offline mirror mode when uninterrupted
local browsing and editing are required.

### 4.2 Offline mirror behavior

Offline mirror is a full local working tree, not a mount cache. Applications
interact only with local files; a background engine compares local and remote
trees when required connectivity is available. All Offline mirrors use
`rclone bisync` (SharePoint document libraries included), as scheduled
synchronization plus Sync Now, not continuous monitoring. Existing
`abraunegg/onedrive` mirrors are a transition case; see FR-031C.

### 4.3 Google cloud-native documents

Google Docs, Sheets and Slides do not round-trip safely through exported formats.
Offline mirror mode shall skip them, report them to the user, and offer to open
them in the browser.

### 4.4 NetworkManager VPN control

NetworkManager state is authoritative for saved VPN profile discovery and
lifecycle, and the applet uses D-Bus only (Section 6.9). Cisco-compatible gateways
use a NetworkManager OpenConnect profile; interactive secrets come from a host
`nm-applet` agent, an OpenConnect-specific dependency that other VPN plugins shall
not require unless separately specified.

### 4.5 Settings windows

General Settings, Add, Modify and Import run in standalone COSMIC application
windows launched from the panel applet, not as embedded applet child windows,
which may inherit toolkit titles such as `Cosmic - Iced` and cannot reliably
update titles. Add/Modify/Import shall be titled `Cloud Mounter Connection
Settings`; the app-wide window shall be titled `Cloud Mounter Settings`.

## 5. User Workflows

1. **Configure:** the user selects a provider and one access mode; selects,
   detects or creates a remote and optional subtree; chooses a mountpoint or
   mirror directory; optionally selects a VPN and readiness checks. The applet
   validates paths, disk space, dependencies and versions, runs the supported
   authentication flow or shows exact setup instructions, tests the connection,
   shows a preview, and after confirmation writes configuration and generated
   units atomically.
2. **Online mount:** turning the popup switch on records the intent to mount now
   and at future logins. The applet prepares and verifies any VPN, starts the
   service, verifies the actual mount and provider health, and monitors pending
   writes and connectivity. On a safe connectivity failure it detaches, and when
   readiness returns it remounts while the connection remains enabled. Turning
   the switch off unmounts and keeps it off at future logins.
3. **Offline mirror:** the applet estimates remote size and local space, runs a dry
   preview, and shows expected uploads, downloads, deletions and conflicts. The
   user explicitly confirms the initial synchronization. Afterwards it
   synchronizes when connectivity returns, periodically while online, and on Sync
   Now; conflicts preserve both versions for review.
4. **Import:** the applet scans `~/.config/systemd/user/`, parses only a documented
   safe subset of unit syntax, and shows a preview with unsupported options and
   target conflicts. After confirmation it creates an applet-managed replacement;
   the original is preserved unless the user separately confirms disabling it.

## 6. Functional Requirements

### 6.1 Panel and status

- **FR-001:** The applet shall provide a COSMIC panel icon and popup.
- **FR-001A:** The popup shall place a clickable Settings gear at the right edge
  of the title row, with Cloud Mounter aligned to the left. Add Connection,
  Refresh and sleep options shall be in General Settings, with no toolbar or
  sleep-settings footer in the popup.
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
  semantics. The popup control is the persistent desired-state input: an
  Online mount left on shall be remounted at the next login, while an Online
  mount or Offline mirror turned off shall remain off. A slider/toggle-style
  control is acceptable when it preserves keyboard accessibility and non-color
  state indication.
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
- **FR-006A:** If an Online filesystem remains mounted after its required
  network or VPN dependency becomes unavailable, the UI shall show a degraded
  mounted state rather than healthy/connected. It shall retain safe Unmount and
  expose the separately confirmed Repair/lazy-detach action if clean unmount
  cannot detach the mount. Repair confirmation shall require a separately
  labeled second activation after a minimum arming interval so duplicate UI
  delivery cannot execute a lazy detach. The Repair action shall be spatially
  separate from the primary mount switch so its hit target is unambiguous, and
  it shall disappear immediately after confirmation while Repair is running. It
  shall suppress any cached pending-upload number as unverified and provide
  explicit off/on reauthentication guidance. This state does not satisfy the
  separate automatic safe-detach and remount requirements.
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
  control per connection (FR-003). Secondary actions such as Preview, Sync Now,
  Retry, Details, and Remove shall be available from the Add/Modify or
  diagnostics workflow, not the main popup; Repair appears only as FR-006A
  specifies.
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
- **FR-011B:** The Add/Modify wizard shall expose Online mount options including
  cache size, bounded timeouts, retries, bandwidth limits, and safe detach
  policy. It shall not expose a second Start at login switch that can conflict
  with the persistent popup control.
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
  choice, action, status chip, or VPN chip as a COSMIC/libcosmic hover tooltip
  when the toolkit supports it, appearing after a short delay of about one
  second and positioned above or beside the source so it does not obscure it.
  Tooltip wrappers shall not be nested; each interactive control shall own at
  most one tooltip so help text does not overlay other help text. Visible help
  buttons are reserved for future controls that cannot carry a tooltip. Longer
  dependency, safety, setup, and troubleshooting guidance belongs in
  documentation, not a main-popup Help control.
- **FR-011I:** Add/Modify shall keep verbose selected-item explanations out of
  the normal form body when the same information can be provided as help text.
  In particular, selected VPN profile details (external profile behavior,
  readiness checks, timeout, activation expectations) shall be available from the
  VPN profile choice tooltip instead of inline paragraphs.
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
- **FR-016:** New Online mounts shall remain stopped by default until the user
  turns them on from the popup.
- **FR-017:** A successful popup Mount shall persist the Online mount as enabled
  for future logins. A popup Unmount request shall persist it as disabled for
  future logins and disable its user service, without requiring an editor save.
  Offline mirror Start/Stop shall continue to enable or disable its schedule or
  monitor across logins. Sleep-only cleanup shall not modify this persistent
  state.
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

- **FR-021:** Packaging shall declare required executables. The applet shall
  report a missing dependency or an operation-time capability failure without
  running startup-wide version/help probes solely for inventory.
- **FR-022:** The applet shall require a current supported release of each
  selected storage engine.
- **FR-023:** The applet shall reject the installed rclone `1.60.1` for managed
  connections and direct the user to upgrade.
- **FR-024:** Dependency guidance shall identify the required upstream project
  and shall not perform installation or repository changes.
- **FR-025:** A missing dependency for one provider or mode shall not prevent use
  of other available providers or modes.
- **FR-026:** Required capabilities shall be verified in packaging/build tests
  or by typed operation-time results rather than relying only on a version
  string or repeated runtime help probes.
- **FR-026A:** Production startup shall not run `rclone version`, rclone help,
  `onedriver --version`, `onedriver --help`, `onedrive --version`,
  `fusermount3 --version`, or `fuser --version` solely as dependency probes.
  Packaging shall declare minimum dependencies, focused diagnostic/build tests
  shall retain capability coverage, and operation-time failures shall remain
  actionable. Actual clean/lazy `fusermount3` unmount operations remain allowed;
  `fuser` shall not remain when it has no production operation.
- **FR-026B:** NetworkManager and the systemd user manager shall be detected by
  connecting to their D-Bus APIs, not by running `nmcli --version` or
  `systemctl --user --version`.

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
  to rclone. The applet shall store no OneDrive tokens, client IDs or secrets.
- **FR-031A:** For OneDrive Offline mirror setup the applet shall run rclone's
  browser OAuth for backend `onedrive` with rclone's default client, using the
  same create-and-continue flow as SharePoint setup and the desktop portal for
  browser opening in Flatpak. It shall verify the remote type and drive type
  (personal or business), show the signed-in account or drive name from Microsoft
  Graph for user confirmation, and record the drive identity in the connection.
  Later runs shall be blocked when the identity differs, and reauthentication
  shall yield the same identity. The applet shall not offer client ID, secret or
  tenant entry. Authorization shall not use `--dry-run`; bounded preview occurs
  only after authentication.
- **FR-031B:** The applet shall prevent concurrent `jstaf/onedriver` Online mount and
  OneDrive Offline mirror operation against the same OneDrive account or an
  overlapping remote subtree, and shall prevent an rclone OneDrive mirror from
  sharing or overlapping a local tree with a legacy `abraunegg/onedrive` mirror.
- **FR-031C:** Existing `abraunegg/onedrive` OneDrive Offline connections shall keep
  running unchanged for one transition release with a deprecation notice, shall not
  be newly creatable, and shall load as unsupported after the removal release with
  their units not regenerated and local data untouched. No conversion tool is
  provided; the abraunegg database, cache and token are never reused.
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
  remain sanitized. No manual Cancel control is required. Notices shall be
  brief and readable before automatic dismissal: completed (with requested depth
  where applicable), time limit reached with an incomplete scan, or skipped; an
  excluded SFTP target is reported as skipped, never completed. They omit
  configured and elapsed times and routine explanations, and take their settings
  from the running job, not from settings reread at completion. Elapsed time may
  exceed the limit while work stops; strict wall-clock termination is not an
  acceptance requirement. Successful traversal is not proof of faster browsing;
  validate usefulness with repeatable fresh-mount comparisons with preload
  disabled and enabled.
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
- **FR-053:** OneDrive Offline mirrors shall use scheduled rclone bisync plus Sync
  Now; continuous monitoring is not required. Legacy `abraunegg/onedrive`
  connections keep their monitor service during the transition release.
- **FR-054:** Only one synchronization operation may run per connection.
- **FR-055:** Automatic synchronization shall pause on metered networks by
  default, with Sync Now and a per-connection override available.
- **FR-055A:** Metered-state detection shall use NetworkManager D-Bus in both
  native and Flatpak operation. Generated scheduled-mirror behavior shall not
  invoke `nmcli`; manual and scheduled synchronization shall apply the same
  metered-network policy.
- **FR-056:** Changes and deletions shall propagate in both directions.
- **FR-057:** When the same file changes on both sides, neither version shall be
  silently overwritten; both versions shall be preserved and reported.
- **FR-058:** Deleted and overwritten files shall be retained in recovery
  storage for 30 days.
- **FR-059:** Recovery cleanup shall never run while a synchronization for that
  connection is active.
- **FR-060:** Interrupted synchronization shall retain engine state and offer
  automatic recovery or a reviewed recovery workflow.
- **FR-060A:** Offline Mirror Repair shall be a confirmed recovery-and-pause
  operation, not a synchronization operation. It shall cancel and boundedly
  await app-owned manual work, use the systemd user-manager D-Bus adapter,
  reconcile observed final state, and never run preview, sync, resync,
  authentication, retention, or cleanup. It shall preserve mirror data,
  recovery data, engine state, locks, ownership and initialization markers, and
  shall never use Online-mount FUSE Repair.
- **FR-060B:** Rclone bisync Repair shall stop and disable the periodic timer
  before stopping its finite service, reset only an observed failed service,
  and finish only when the timer is inactive/disabled and the service is
  inactive/non-failed. It shall preserve the renewable bisync lock and direct
  the user to wait through its two-minute maximum or use reviewed manual
  recovery; it shall never delete the lock or trigger implicit resync.
- **FR-060C:** During the transition release, `abraunegg/onedrive` Repair for a
  legacy connection shall stop and disable only its
  continuous monitor service, reset only observed failed state, and preserve
  its configuration, token, sync database, lock/state, local mirror, recovery
  data, selection, and initialization markers. It shall not create an rclone
  timer, invoke sync/resync/reauthentication, or touch a `jstaf/onedriver`
  Online mount or cache.
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
- **FR-077:** Unit writes shall use strict in-process structural validation, a
  temporary file, and atomic rename. Runtime validation shall not invoke
  `systemd-analyze`.
- **FR-078:** Managed user-unit lifecycle shall use the systemd user-manager
  D-Bus API for reload, enable/disable, start/stop/restart, job completion,
  status, failure reset, and timer management. Production code shall not invoke
  `systemctl --user`.
- **FR-078A:** After a write and manager reload, `LoadUnit` results shall verify
  load state, load error, the exact applet-managed fragment path, and daemon
  reload state. Failure shall restore the previous file and reload the rollback.
- **FR-078B:** Start, stop, and restart operations shall correlate the returned
  job path with the matching `JobRemoved` signal and report its result. A local
  timeout or cancellation shall not be reported as cancellation of an already
  queued systemd job. Pre-sleep stop requests may be deliberately nonblocking
  when the shared sleep deadline owns completion monitoring.
- **FR-079:** A failed update shall preserve or restore the last valid
  configuration and unit.
- **FR-080:** Commands shall use fixed executables and separate validated
  arguments, never shell interpolation.

### 6.9 VPN dependencies

- **FR-081:** A connection may reference zero or one VPN profile.
- **FR-082:** The applet shall enumerate NetworkManager VPN profiles visible to
  the current user through NetworkManager D-Bus.
- **FR-082A:** NetworkManager VPN detection shall import existing profile
  references by UUID, display name, and VPN service type without storing VPN
  credentials. Repeated detection shall update existing references rather than
  creating duplicates.
- **FR-082B:** The applet shall identify OpenConnect profiles by service type
  `org.freedesktop.NetworkManager.openconnect`; it shall not infer plugin type
  from a display name.
- **FR-083:** NetworkManager profile enumeration, activation, active-state
  observation, and deactivation shall use typed D-Bus calls. No production
  `nmcli` fallback is permitted.
- **FR-084:** Before activating an OpenConnect profile, the applet shall start
  `cosmic-mounter-openconnect-agent.service` through the systemd user-manager
  D-Bus and wait for the start job, including its registration-stabilization
  step, to complete. This operation shall not run for other VPN service types.
- **FR-084A:** If the required OpenConnect authentication-agent user service is
  absent or fails to start, the applet shall stop before activation and provide
  actionable package and service-installation guidance. It shall not execute
  `nm-applet`, `systemctl`, or `nm-openconnect-auth-dialog` from the Flatpak.
- **FR-084B:** NetworkManager shall request OpenConnect secrets over D-Bus from
  the registered host agent. The applet shall not request, receive, store,
  display, log, or forward usernames, passwords, cookies, or second-factor
  responses.
- **FR-084C:** If more than one NetworkManager secret agent is registered, the
  installed native and Flatpak applet shall prove that OpenConnect activation
  opens the complete multi-field dialog without leaving COSMIC's generic
  single-field prompt active. A reliable session or agent-selection rule is
  required before OpenConnect support is accepted.
- **FR-084D:** OpenConnect dependency diagnostics shall distinguish a missing
  saved profile, NetworkManager plugin, graphical authentication dialog, and
  authentication-agent user service. The applet and Flatpak shall not install
  packages, execute host agents directly, or bundle OpenConnect or `nm-applet`.
- **FR-084E:** After it is started, the OpenConnect authentication-agent service
  shall remain available for possible reauthentication until the COSMIC
  graphical session ends. The applet shall not stop the service and therefore
  cannot stop an agent started by another application.
- **FR-085:** Each dependency shall support readiness checks using one or more of
  NetworkManager state, interface, route, DNS, or endpoint reachability.
- **FR-085A:** Route, DNS, and TCP readiness shall use bounded native socket and
  asynchronous resolver APIs. Production readiness checks shall not invoke
  `ip`, `getent`, or `nc`.
- **FR-086:** Mounting or synchronization shall start only after the tunnel is
  connected and all configured readiness checks pass. Timeout shall fail the
  attempt without starting storage.
- **FR-087:** The UI shall report when external interactive VPN authentication
  is required and shall keep the applet responsive while it is pending.
- **FR-086A (A):** Use one monotonic overall activation/readiness deadline from
  the saved profile. Include interactive password and second-factor time in that
  deadline. NetworkManager D-Bus activation monitoring shall not impose a
  shorter process timeout. Short readiness-probe timeouts shall remain separate.
- **FR-086B (A):** Existing saved positive timeouts shall be preserved;
  Detect VPNs shall preserve user timeout and readiness settings when refreshing
  references.
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
- **FR-090A:** The Add/Modify VPN section shall offer No VPN and detected or
  configured NetworkManager profiles as mutually exclusive choices. Direct
  proprietary Cisco dependencies shall not be offered. Cisco-compatible access
  is represented by a saved NetworkManager OpenConnect profile.
- **FR-090B:** The Add/Modify VPN section shall show the
  disconnect-when-unused control only when a VPN dependency is selected.
- **FR-090C:** Detect VPNs shall report success or failure through the app's
  normal notice path. It shall not leave a detection transcript, detected-profile
  list, or debug log text in the VPN section body.
- **FR-090D:** A saved proprietary-Cisco dependency shall not be silently mapped
  to a NetworkManager profile by name. Migration shall require the user to
  select a saved NetworkManager profile UUID; obsolete Cisco-specific
  configuration shall be removed only after that explicit replacement.

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
  Bound and track the actual systemd D-Bus stop jobs; a local wait timeout shall
  not be treated as cancellation of an already queued manager job.
- **FR-101:** On wake or canceled sleep, reconcile mount/service state, report
  incomplete cleanup once, and reacquire the delay inhibitor. A separate
  “Restore after wake up” setting in General Settings defaults false. Show it
  disabled while Unmount when sleep is off, retain its saved value, and explain
  that only successfully cleaned Online connections are restored.
  Keep cleaned-up connections suspended until manual restart unless restoration
  is enabled; do not alter their saved popup desired state. Restoration shall use the
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
- **NFR-007 Least privilege:** Storage engines shall run as the user. Host
  integration shall use narrowly scoped D-Bus access where available; no system
  authorization is required for proprietary VPN-agent control.
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
| Google Drive, Box, SMB, SFTP, SharePoint Online mount | `rclone` 1.74.3 or newer and FUSE 3 |
| Google Drive, Box, SMB, SFTP, SharePoint Offline mirror | `rclone` 1.74.3 or newer with required bisync safety features |
| OneDrive Online mount | `jstaf/onedriver` 0.15.0 or newer |
| OneDrive Offline mirror | `rclone` 1.74.3 or newer with bisync safety features (a OneDrive-specific minimum is an open item in the [OneDrive Offline contracts](docs/engineering/onedrive-rclone-offline-contracts.md)) |
| Legacy OneDrive Offline mirror (transition release only) | `abraunegg/onedrive` 2.5.10 or newer |
| User service/timer control and optional sleep cleanup | Host systemd user-manager and logind D-Bus services |
| NetworkManager VPN | NetworkManager D-Bus service |
| OpenConnect VPN | Host `network-manager-openconnect` plugin and authentication dialog |
| Interactive OpenConnect authentication | Host `nm-applet` secret agent |
| Legacy OneDrive sign-in helper (transition release only) | Python 3/PyGObject, GTK 3 and WebKitGTK 4.1 runtime/introspection libraries |

Dependencies shall be checked independently. An unavailable mode shall not
disable unrelated modes.

The OneDrive sign-in helper and the manual browser/response-URL handoff exist only
for legacy `abraunegg/onedrive` connections. They are unchanged during the
transition release and removed, with their Debian recommendations and installer
options, in the removal release. rclone OneDrive setup needs neither.

### 8.3 Permissions

- Mountpoints, mirror directories, caches, recovery directories, and units shall
  be user-owned.
- Storage services and sync jobs shall run without root privileges.
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
| `providers` | rclone and `jstaf/onedriver` adapters (and the legacy `abraunegg/onedrive` adapter until removal) |
| `mounts` | Mount lifecycle, mount-table inspection, and VFS health |
| `sync` | Preview, scheduling, conflict/recovery state, and sync lifecycle |
| `services` | systemd unit/timer rendering and user-manager D-Bus control |
| `vpn` | NetworkManager D-Bus, OpenConnect agent checks, readiness, and dependency references |
| `process` | Typed asynchronous command execution and sanitized results |
| `host_inspection` | Fixed, bounded asynchronous configuration and mount observations |
| `bisync_coordinator` / `bisync_worker` | Shared Rust transactions and restricted scheduled entry point |
| `import` | Structured legacy unit discovery, parsing, and preview |
| `diagnostics` | Dependency checks, structured app-owned state, and error mapping |
| `i18n` | Localization resources and initialization |

External interactions shall be represented by traits so tests can use fake
process, provider, mount, synchronization, service, and VPN implementations.

### 9.2 Runtime boundaries

- The UI emits typed intents such as `Mount`, `Unmount`, `SyncNow`, `PauseSync`,
  and `ConfirmInitialSync`.
- Operations are serialized per connection.
- Provider and VPN work runs asynchronously.
- Popup rendering shall read cached service/timer status and coordinator warnings,
  never synchronously query systemd or read coordinator files. Prefetch unit state
  off the UI thread at startup, refresh on opening/configuration/operation changes,
  and bound periodic refresh to visible UI. Use one asynchronous D-Bus connection
  per batch with a five-second batch deadline and two-second per-unit deadlines.
  Keep last-known data on failed reads, visibly identify pending/unavailable
  observations, and reject stale generations/configuration results. Control
  actions shall wait for observations no older than 15 seconds, with unchanged
  connection identity, and retain the backend's fresh safety checks. Expired or
  failed observations shall not authorize controls or imply inactive/zero state.
- Applet exit does not terminate enabled systemd services.
- Shell scripts are references only and are not the applet execution API.

### 9.3 Command execution

Host integration shall use this order:

1. Stable native Rust or D-Bus API.
2. A required external storage engine with a fixed executable identity and
   separate validated arguments.
3. A narrowly scoped, documented compatibility fallback when the required API
   is unavailable on a supported host version.

An executable shall not remain merely because it predates an available D-Bus or
native API. Obsolete invocations shall be removed from production reachability.
There shall be no `sh -c`, command concatenation, or execution of imported
configuration. Manual and generated scheduled bisync execution shall use the
shared Rust coordinator, not a host shell wrapper. Display-only manual-auth
instructions and test/deployment scripts are not applet runtime execution.

Output shall be bounded, decoded safely, and redacted before logging.

### 9.4 Flatpak execution architecture

Native, Debian, and Flatpak installations shall share the same typed D-Bus and
native integration interfaces. Flatpak is an additional packaging target, not a
separate implementation. Because the applet coordinates host storage, FUSE
mounts, host user systemd units, existing provider configuration, and host VPN
state, the sandbox shall receive only the D-Bus and filesystem access required
for those operations.

The following host operations shall use D-Bus rather than
`flatpak-spawn --host`:

- NetworkManager profile discovery, activation, state, and deactivation;
- systemd user-manager reload, unit enable/disable, start/stop/restart, job
  completion, failure reset, and timer lifecycle;
- logind sleep preparation; and
- Documents portal path resolution when the installed portal supplies the
  required API.

`flatpak-spawn --host` is not the default host-integration mechanism. It may be
used only for an invocation classified in `docs/engineering/flatpak-host-command-inventory.md` as an unavoidable
storage-engine operation, required FUSE/host-namespace helper, or documented, version-gated compatibility
fallback. Production invocations classified as obsolete shall be removed.
Tests and standalone transport probes shall not be treated as production
requirements.

Every remaining host command shall preserve the safety model:

- fixed executable identities rather than arbitrary shell commands;
- separate validated arguments;
- no `sh -c` or command concatenation;
- bounded stdout and stderr capture;
- bounded lifetime and cancellation behavior equivalent to the native runner;
- redaction of secrets, OAuth URLs, tokens, and passwords before display or
  logging.

Read-only host inspection shall accept only `RcloneConfigDump` and `HostMountInfo`.
Configuration capture shall be limited to 1 MiB per stream and mountinfo capture
to 4 MiB, with a 10-second execution deadline, no retries, cancellation, and
bounded child cleanup. Deadlines shall include output-pipe draining; Flatpak
inspection children shall exit when their transport disappears (`--watch-bus`).
Reject truncated, invalid-UTF-8, or malformed results instead of applying partial
data. Native mountinfo shall use a bounded asynchronous local `/proc` read.
GUI completions shall carry parsed non-secret metadata, not raw configuration.
Superseded discovery/prefill shall cancel old work and reject late results after
identity/revision/user-edit changes. UI rendering shall read a refreshed mount
snapshot without executing host commands; failed/obsolete reads shall not imply
mount disappearance. Safety-critical setup, unmount, repair, and shared-VPN
release shall obtain fresh observations and fail closed on inspection failure,
including OneDrive isolation after potentially long authentication.

The following resources are host resources and shall not be silently copied into
a Flatpak-private credential or configuration store:

- the applet's own saved connection list and VPN profile references;
- existing rclone remotes and credentials;
- `jstaf/onedriver` authentication state (and legacy `abraunegg/onedrive` state until removal);
- generated user systemd services and timers under the host user session;
- user-selected mountpoints, mirror directories, cache directories, and
  recovery directories;
- NetworkManager VPN state.

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
general-purpose shell execution. The helper shall also make clear which state is
applet-owned and which remains provider-owned by rclone, onedriver, onedrive,
NetworkManager, OpenConnect, or systemd.

Flatpak permissions shall start from the accepted COSMIC applet pattern with
Wayland/session access. Add narrowly scoped access to NetworkManager, the
systemd user manager, logind, portals, and the applet runtime interface as each
feature requires. Retain `--talk-name=org.freedesktop.Flatpak` only while a
justified `flatpak-spawn --host` invocation remains. Broader filesystem access, including
`--filesystem=host`, is permitted only if prototype testing proves narrower
permissions cannot support user-selected storage targets, generated host
services, or host-visible FUSE mounts. Any broader permission must be documented
in the Flatpak manifest notes, README, and submission pull request.

Flatpak publication shall be rejected or postponed if testing shows that the
package cannot expose mounts to ordinary host applications, cannot manage the
intended host user services, silently uses different rclone or OneDrive
credentials or applet connection configuration than the native applet, or
requires unjustified unrestricted host access.

The Documents portal implementation shall use sandbox-capable `GetHostPaths`
when portal version 5 or newer is available. On supported systems exposing only
portal version 4, a narrowly version-gated `flatpak document-info` host fallback
may remain because `Info` and `Lookup` are forbidden to the sandbox and
`GetHostPaths` is unavailable. The fallback shall be removed when portal v5 can
be required and shall not establish a general host-command exception.

Portal results shall support direct and app-scoped document IDs, decode
NUL-terminated byte paths safely, and reject non-absolute host paths. Native
execution may use portal `Info`; sandbox execution shall not rely on portal
methods forbidden to sandbox callers. Standard portal access shall not acquire
an unnecessary additional Flatpak permission.

For an OpenConnect profile only, the applet shall start the installed static
`cosmic-mounter-openconnect-agent.service` through the systemd user-manager
D-Bus and wait for successful job completion before asking NetworkManager to
activate the profile. NetworkManager, not the applet, owns the subsequent D-Bus
secret exchange and launches the host authentication dialog. The Flatpak shall
not execute `nm-applet`, `systemctl`, or the dialog and shall not require
`nm-applet` for other VPN service types.

Migration completion requires reconciliation against `docs/engineering/flatpak-host-command-inventory.md`. Every
production row shall end as **Replaced by D-Bus**, **Removed**, or an explicitly
justified **Unavoidable invocation**; no unresolved replaceable, obsolete, or
mixed disposition may remain. A production-reachability scan shall exclude only
test code, examples, and the standalone transport probe and shall confirm that
obsolete executable variants, parsers, permissions, translations, and
documentation are removed. A nested or ungated `flatpak document-info` path is
not permitted.

Native source and Debian packages shall use the same typed NetworkManager and
systemd adapters as the Flatpak. Flatpak-specific behavior shall be confined to
required bus permissions, portal/sandbox setup, host-visible paths, and the
narrow command-runner boundary. Packaging documentation and the submission
rationale shall identify every remaining host command and host-visible path and
justify its permission.

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
  unmount_before_sleep: bool = false
  restore_after_wake: bool = false
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
  start_at_login  # compatibility storage for the Online popup desired state
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
  kind: NetworkManager
  external_profile_id: NetworkManager UUID
  service_type
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
- be disabled at login until the popup switch is turned on; successful Mount
  enables it for later logins and Unmount disables it.

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

The managed rclone service enables unauthenticated RC commands only on its
connection-specific Unix socket below the user's private runtime directory. The
applet retains the RC job ID and rclone execution ID, polls `job/status`, and
cancels with `job/stop` before unmount, repair, removal or sleep cleanup. Any
non-`OK` value below `output.result` is a provider failure even if rclone marks
the job successful. Mount success is independent of this background job.

OneDrive, Box, and SMB post-mount directory preload:

```text
find -P <mountpoint> -xdev [-maxdepth <Box or SMB depth>] -type d -printf ""
```

The applet shall construct this as a typed command, not shell text, enforce the
effective provider or connection deadline, and cancel the child before detach
(FR-044C, FR-044EA, FR-044G). OneDrive traverses without a depth limit; Box and
SMB use their configured depths and shall not use recursive `vfs/refresh`.

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

Manual and scheduled bisync preview/synchronization shall use one app-owned
Rust coordinator instead of a generated shell wrapper. The coordinator
shall expose only fixed modes—`initial-preview`, `preview`, `initial-sync`,
`sync`, and `scheduled`—with validated typed inputs; it shall not expose an
arbitrary executable or shell-fragment parameter. Host `rclone` remains the
external storage engine.

The coordinator shall preserve the complete storage-safety transaction:

- preflight validation, recovery preparation, operation ordering, post-sync
  verification, progress publication, and successful-run-only retention;
- UTC recovery stamps, unique backup suffixes, local and remote ownership
  markers, strict dated-directory parsing, and preservation of unowned or
  symlinked recovery data;
- SharePoint saved-target verification, local and remote access anchors, and
  checksum verification for rewritten Office files;
- initial-preview and initial-sync markers, explicit `--resync` selection,
  per-connection work-directory locking, bounded/redacted output, existing
  deadlines, cancellation, and no automatic destructive retry;
- atomic request/result metadata, structured invocation-ID progress state, and
  prevention of overlap between manual and timer-triggered runs; and
- retention only after bisync and required post-sync verification succeed.
  Cleanup failure shall be actionable but shall not invalidate or destructively
  roll back a successful synchronization.

Native/Debian services shall invoke the installed restricted `--run-bisync`
worker with a fixed mode and saved connection UUID. Flatpak services shall use
the same worker through the fixed `flatpak run --command=cosmic-ext-applet-mounter`
entry point. Paths and engine arguments shall be reconstructed from authoritative
saved configuration, not supplied as arbitrary command payloads. Neither route
shall retain a scheduled shell wrapper or acquire a general-command interface.
Failure of the final Flatpak worker acceptance gate shall block publication,
not silently restore shell execution.

### 11.3 OneDrive services

- Online mount shall use `jstaf/onedriver`'s supported user-service behavior.
- Offline mirror shall use an rclone OneDrive remote, a dedicated local directory
  and bisync state per connection, and the restricted scheduled worker.
- Legacy `abraunegg/onedrive` connections keep a dedicated configuration, sync
  directory and monitor service during the transition release only.
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
- Manual Online unmount shall show progress and, when preload is active, say that
  preload is stopping first. Directory-walk subprocesses shall be cancelled and
  reaped before clean detach; if they do not exit within the bounded wait, leave
  the mount service running and report that unmount can be retried. Attempt clean
  FUSE detach while the service is still available, keeping a mount usable if
  another process holds it busy. Retry a transient busy result briefly after
  preload cancellation, and use the mount table to resolve a helper error after
  the mount has already detached.
- A successful service-stop command is not evidence of unmount success. The
  mount-table entry shall disappear before the applet reports completion or
  disconnects an applet-started VPN. A stopped or failed service with a lingering
  mount entry shall report Error and offer the confirmed Repair flow. Unmount
  shall not recreate or prepare the mountpoint. Repair shall reset systemd failed
  state only when the service is actually Failed, so an already inactive service
  never makes a successful lazy detach appear to fail.
- If clean unmount fails, the applet may offer lazy unmount only after explicit
  user confirmation and a clear warning.
- Queued or in-progress writes shall prevent lazy unmount.
- The applet shall disconnect only a VPN it activated, and only after no active
  connection still depends on it.

## 13. UI Specification

Behavior is specified by the functional requirements in Section 6. This section
fixes layout and presentation; its statements are normative layout requirements.

### 13.1 Panel popup

Behavior: FR-001A to FR-001D, FR-002, FR-003 to FR-003B, FR-010A to FR-010C.

- Title row with Cloud Mounter aligned left and a clickable Settings gear at the
  right edge, followed by active connection count, notification state and VPN
  summary.
- A notice/result area below the header only when an operation has something to
  report.
- A continuous themed background for header and connections, with a thin
  horizontal line between status/notices and the connection list and no empty
  background band. The space the removed toolbar and sleep footer used belongs to
  the connection list.
- Rows are compact cards sized for the fixed-width popup: one line with the name
  on the left and the compact primary state control on the right. A long name may
  wrap or elide, but the control stays reachable and consistently sized.
- The popup grows to fit configured rows up to a bounded maximum height; scrolling
  engages only when rows exceed it.
- Import previews do not replace the connection list; candidates stay previews
  until the user confirms creating applet-managed connections.
- Popup actions dispatch typed operation requests and return immediately.

### 13.1A General Settings

Behavior: FR-001C, FR-044D and the sleep requirements of Section 6.11.

- Opened and focused from the popup gear in a standalone window with an ordinary
  close control, titled `Cloud Mounter Settings`, on the same COSMIC themed list
  surface as the popup and Add/Modify content, including behind hover help; no
  hardcoded background color.
- Top row: Add Connection and Refresh. Add Connection launches the existing
  editor rather than embedding connection fields. Refresh updates the runtime
  owner. Refresh results, launch errors and connection-to-applet notices appear
  directly below this row, before the sleep options.
- Sleep: both toggles together, labeled "Unmount when sleep" and "Restore after
  wake up". Help explains the Online-only scope and restoration eligibility. The
  second toggle stays visible but disabled while the first is off, so the window
  does not resize.
- End the sleep options with "Applies only to Online connections." Show sleep
  preference feedback and cleanup status as ordinary wrapping text immediately
  below it, with no "Sleep cleanup status" heading or fixed-height box, separate
  from top-row notices. Surface unresolved sleep problems briefly in popup status
  even when Settings is closed.
- **Preload** section: one row per Online provider (Google Drive, OneDrive, Box,
  SharePoint, SMB, SFTP, with SFTP immediately below SMB), each with an enabled
  control and maximum seconds; Box, SharePoint, SMB and SFTP also have maximum
  depth. Defaults and bounds are in FR-044D, FR-SFTP-015 and Section 10. Populate
  from saved values; keep controls visible when disabled so the window does not
  resize; keep each provider's switch, duration and optional depth on one
  horizontal row. Explain duration and depth, with accepted ranges, through
  delayed hover help instead of captions or a range line. Duration help states
  that browsing remains available while preload runs; depth help explains that
  deeper scans add server requests, can trigger rate limits and may use up the
  preload time.
- Provider and sleep toggles have visible spacing between label and switch.
- Persist toggles immediately with validated writes acknowledged by the owner;
  show save failures without presenting an unsaved setting as effective.
- All controls remain reachable with keyboard navigation, long translated
  labels, display scaling and small windows. Long status details wrap in the
  window's scrollable content, with content padding inside the scrollable region
  so the vertical scrollbar stays at the window's right edge; do not reserve a
  separate status box or reduce the popup list height.

### 13.2 Add, Modify, Import, and Field Help

Behavior: FR-011 to FR-011N, FR-SFTP-002 to FR-SFTP-004.

- Add Connection (from General Settings) opens the wizard; Modify opens it
  prefilled. App-wide General Settings and the per-connection editor are distinct.
- Add mode instruction text names the required choices: provider, mode,
  remote/subtree, local target, VPN and startup policy. The same notice area
  shows validation results and operation feedback.
- Modify mode shows connection actions at the top, in one row where possible and
  two rows for action-heavy modes such as OneDrive Offline to avoid clipping.
- An SMB or SFTP Online connection shows **Use global preload settings**, enabled
  by default. Disabling it enables that connection's toggle, maximum seconds and
  maximum depth; help explains the override suits home, LAN, corporate and VPN
  shares of differing size and latency (FR-044H).
- Wizard step order: (1) provider: OneDrive, Google Drive, Box, SMB or SFTP, with
  SFTP immediately right of SMB in matching style; (2) access mode, which with the
  provider determines the engine; (3) account or remote (OneDrive Online uses
  `jstaf/onedriver`; every other case uses rclone remotes); (4) whole remote or
  subtree; (5) local target, a mountpoint for Online or a mirror directory for
  Offline (not interchangeable); (6) mode-specific behavior; (7) optional VPN;
  (8) review of dependency status, generated units, preview and safety warnings;
  (9) confirm creation or update.
- Online fields: display name, enabled state, VFS cache limit (20 GiB default),
  timeout and retry bounds, bandwidth limits, safe-detach behavior and
  lazy-unmount confirmation policy. Persistent startup is controlled only by the
  popup switch.
- Offline fields: display name, enabled state, whole-drive or subtree selection,
  disk-space estimate, initial-sync preview, sync interval, Sync Now, metered
  pause default and per-connection override, recovery location, 30-day retention,
  conflict preservation, and resync/state-rebuild confirmation.
- VPN is per connection: No VPN, a NetworkManager VPN profile, or Detect VPNs,
  presented compactly. Cisco-compatible gateways are NetworkManager OpenConnect
  profiles; direct Cisco Secure Client dependencies are not offered. Detect VPNs
  imports or updates references and reports through the normal notice path, with
  no detection transcript or profile list in the section body. Fields are
  readiness checks, timeout, whether the applet may activate the VPN, and the
  rule that it disconnects only VPNs it activated; the disconnect-when-unused
  control appears only after a VPN is selected.
- Import opens a dedicated workflow (FR-011F, FR-011N, Section 6.7) whose previews
  map into the wizard fields.
- Rclone-backed providers in Add mode show Detect rclone remotes, the
  provider-specific Create Remote, detected-remote choices wrapped into bounded
  rows, and the unused-remote management area (FR-011K to FR-011MA). Modify mode
  hides these; its remote-name help describes existing-name use without pointing
  to a hidden Create button, and SMB help explains Create/Update.
- OneDrive setup uses user-facing guidance: complete required fields, finish
  browser authentication, then Test Connection and Save Connection. Credential
  file locations belong in documentation and diagnostics, not form text.
- Per-field help follows FR-011H.
- Destructive or data-affecting actions use confirmation dialogs stating the
  action, target paths, preserved data and expected consequences.
- Controls are keyboard accessible, localized and understandable without color.

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
- NetworkManager and systemd D-Bus availability, method/error mapping,
  `JobRemoved` correlation, unit-load verification, rollback, timeout, and
  cancellation behavior without production command fallbacks.
- Documents portal v5 host-path resolution, portal-v4 fallback gating, direct
  and app-scoped IDs, byte/NUL decoding, and rejection of relative paths.
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
- Typed bisync-coordinator request ordering for all five modes, ordinary and
  SharePoint-specific guards, ownership and symlink rejection, failure/timeout/
  cancellation at every phase, one-run-per-connection serialization, and proof
  that failed required phases cannot run later mutation or retention steps.
- Google cloud-native document exclusion.
- Legacy import parsing, preview, ownership, and conflicts.
- VPN D-Bus discovery and lifecycle, readiness, shared dependencies,
  applet-activation tracking, and approved shutdown behavior.
- OpenConnect-only authentication-agent service startup and job completion,
  missing-service guidance, external multi-field authentication, and proof that
  other VPN service types do not acquire the agent dependency.
- Secret redaction and error mapping.

### 14.2 Isolated integration tests

Tests shall use temporary directories and fakes for systemd, mount tables,
NetworkManager, the OpenConnect agent-availability boundary, and cloud providers.
Local-to-local rclone test remotes may exercise mount and bisync behavior without
real accounts.

Tests shall not write to the real `~/.config/systemd/user/`, activate real VPNs,
or access real cloud data.

Disposable local/alias integration tests shall cover initial preview, initial
sync, ordinary two-way sync, deletion recovery, unique repeated backups,
marked-old cleanup, and preservation of unmarked data. Credentialed SFTP,
SharePoint, and VPN tests remain opt-in and shall state their external-system
evidence boundary.

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
- OneDrive Offline mirror with `rclone bisync`.
- NetworkManager VPN readiness, OpenConnect multi-field authentication through
  the host agent, missing-agent guidance, and non-OpenConnect independence from
  `nm-applet`.
- Native and rebuilt-Flatpak parity for VPN-dependent mount/sync, generated
  service and timer lifecycle, sleep cleanup, failure recovery, manual bisync,
  and a timer-fired bisync run. Verify that a scheduled Flatpak coordinator, if
  selected, exposes only its restricted operation interface and can reach host
  rclone without a broader command permission.
- Import from `~/.config/systemd/user/`.
- Removal without credential, data, cache, or recovery deletion.
- Unused rclone remote removal without deleting applet connections or local/cloud
  data.

Real accounts, VPNs, and remote writes require explicit user authorization for
each manual test session.

## 15. Current Acceptance Criteria

The implemented scope is acceptable when:

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
12. Logs, units, configuration and notifications contain no secrets.
13. Automated tests and approved manual acceptance tests pass.
14. The popup and Add/Modify UI pass user-guided visual review, including compact
    popup rows, bounded scrolling, action-row layout, help tooltip placement and
    rclone remote management.
15. The host-command inventory has no unresolved replaceable or obsolete
    production invocation, and each retained invocation and permission has a
    documented justification.
16. Focused adapter tests, formatting, all-target compilation, strict Clippy, the
    full test suite, `just build` and `just build-changed` pass before the Flatpak
    packaging pull request.
17. The installed rebuilt Flatpak passes live NetworkManager/OpenConnect and
    generated mount/mirror service lifecycle tests before submission.

## 16. Application Identity

Behavioral decisions formerly listed here live in the functional requirements,
Sections 3 and 9; this section keeps only the fixed identity.

- Package and binary: `cosmic-ext-applet-mounter`
- App ID: `io.github.uutzinger.cosmic-ext-applet-mounter`
- Repository: `https://github.com/uutzinger/cosmic-ext-applet-mounter`
- License: MIT
- Authors: Urs Utzinger and OpenAI Codex

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

- Reuse the verified SharePoint rclone `onedrive` document-library remote. Keep
  SharePoint bisync listings, lock, local directory, and recovery
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
  separate unattended network/VPN, metered, sleep/wake, and failure checks meet
  the acceptance boundaries below.
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
- Review the SharePoint Offline timer, service, and managed runner for suspend
  and resume safety. Automated or isolated tests shall prove that applet sleep
  cleanup does not stop Offline mirrors, remote operations are bounded, and an
  interrupted scheduled run can stop, clear or outlive its lock safely, retry,
  and finish with an independent file comparison. Attempt a live sleep/wake
  test when the development computer provides reliable suspend and resume.
  Live hardware proof is preferred but is not an unconditional release gate:
  if the available computer cannot suspend or resume reliably, document the
  limitation and the substitute evidence. An observed failure attributable to
  the applet remains release-blocking; inability to obtain reliable hardware
  evidence by itself does not.

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
- Record sleep/wake evidence by layer: code review, automated or isolated
  tests, installed-service observations, and live suspend/resume hardware.
  When reliable live suspend/resume is unavailable, the SharePoint Offline
  gate may pass on the first three layers if they satisfy Section 18.3, with
  live hardware behavior clearly marked unverified rather than passed.

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
- For Offline mirrors, show only counts supported by the active engine's scoped
  observations. Rclone mirrors shall use invocation-matched structured progress
  during a run; idle mirrors shall show an unknown count rather than launch an
  uncoordinated dry-run bisync or infer zero. Preview is an explicit coordinator
  operation, not a background pending-count scrape. OneDrive may obtain a bounded
  read-only client status with its check time. A failed/interrupted run must not
  imply zero, and a new local/remote change can increase the count.
- Scheduled rclone bisync runs shall publish structured app-owned progress state
  tied to the current systemd invocation ID. The applet shall compare that state
  with the unit's D-Bus `InvocationID`, tolerate a partially written final
  record, and reject malformed or oversized state. Production code shall not
  scrape `journalctl` output for pending counts.
- OneDrive and SharePoint rclone mirrors shall use the same coordinator progress
  path and a bounded bisync Preview for pending counts, and keep their saved-target
  and access-anchor safety guards. Legacy `abraunegg/onedrive` connections keep
  their read-only `--display-sync-status` reading during the transition release.
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
