# Cloud Mounter Applet for COSMIC™

## Purpose

The Cloud Mounter Applet manages cloud and network storage connections from the
COSMIC™ desktop panel. Each connection is either a direct **Online mount** or a
local **Offline mirror** that synchronizes with the remote service. The applet
can also bring up a required VPN before accessing storage and disconnect it when
it is no longer needed.

No network-backed filesystem can guarantee that a file manager never blocks when
a connection is slow or unavailable. Users who need uninterrupted file access
shall use Offline mirror, where applications work only with ordinary local files.

This document is the product source of truth. `Requirements and Specifications.md`
is its testable translation; if they disagree, this document wins. Detailed UI
behavior, generated-service contents and acceptance tests belong there.

## Status

Release 0.5.3 targets native source and Debian installation with the current
engines. Flatpak is experimental; final packaging and engine-consolidation
decisions are deferred to 0.5.4. Existing connection and data semantics are kept
until an explicitly accepted migration changes them.

| Area | Status |
|---|---|
| OneDrive, Google Drive, Box, SMB | Supported |
| SFTP | Editor and provider wiring implemented; installed-service and live-server acceptance pending |
| SharePoint libraries (including Teams files) | Not in the supported matrix until implementation and acceptance complete |
| Unmount before sleep / restore after wake | Locally tested; live native and Flatpak acceptance pending |
| Compact popup and General Settings window | Locally verified; desktop and Flatpak visual acceptance pending |

Acceptance tracking is in `Task List.md`.

## Providers and Tools

Each connection uses exactly one access mode.

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | `jstaf/onedriver` | `rclone bisync` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |
| SFTP | `rclone mount` | `rclone bisync` |

- `jstaf/onedriver` gives an on-demand OneDrive filesystem with local caching and
  read-only access to previously opened files while offline.
- A current stable `rclone` is the common engine for every other mount and for
  all Offline mirrors, including OneDrive and SharePoint. Offline is scheduled
  synchronization plus Sync Now, not continuous monitoring.
- OneDrive Offline mirrors created by earlier versions with `abraunegg/onedrive`
  keep working for one transition release, cannot be newly created, and are then
  removed; users recreate them as rclone connections. See the
  [OneDrive Offline contracts](docs/engineering/onedrive-rclone-offline-contracts.md).
- The applet requires current supported releases of its external tools. It
  detects missing or outdated ones and gives installation guidance, but never
  installs or updates software.
- Credentials stay in the provider tool or the operating-system secret store;
  the applet never copies them into its configuration.

## Connection Modes

Online mount and Offline mirror are mutually exclusive per connection, and a
directory configured for one mode is never reused by the other.

### Online mount

A network-backed FUSE filesystem giving on-demand access without a complete local
copy. Applications may wait when the network, VPN or provider is slow. Its cache
improves compatibility but is not an offline copy and is never described as one.

- New connections start manually; the user may enable mounting at login.
- Connection and operation timeouts are bounded.
- Before an rclone mount starts, the selected remote and subtree must list
  successfully, so authentication and access failures stop the mount before a
  FUSE filesystem can block applications.
- The applet monitors network, VPN, service, mount and provider health, and for
  rclone mounts the upload queue and cache state.
- When required connectivity is lost, it detaches automatically only if no
  uploads or writes are pending. Otherwise it warns, preserves the cache and
  offers recovery and manual detach.
- When connectivity returns and all network and VPN readiness checks pass, it
  remounts connections that remain enabled. It never opens an interactive
  password or second-factor dialog merely because connectivity returned.
- A mount that is present while a required dependency is unavailable shows as
  degraded, not healthy. Unmount stays available, plus a separately confirmed
  Repair (lazy detach) if clean unmount fails.
- Explicit mount, unmount, retry and repair actions are always available.
- Manual unmount, repair, removal and pre-sleep cleanup first cancel any active
  preload or refresh, then detach cleanly while the service is still running.
  If a process holds the mount busy, the applet reports failure and leaves the
  mount available for retry. The mount table, not the helper's exit status,
  decides whether detach succeeded.

### Offline mirror

A complete local copy of a chosen remote folder or the whole remote. Applications
and the file manager touch only the local directory.

- Before the first sync the applet estimates remote size and local disk needs and
  shows a dry preview of uploads, downloads, deletions and conflicts. The initial
  sync needs explicit confirmation.
- Sync runs when network and VPN are ready, periodically while online, and on
  demand. Automatic sync pauses on metered networks unless overridden per
  connection; Sync Now always works.
- Local files stay readable and writable offline; offline changes sync when
  connectivity returns.
- If a file changes on both sides, both versions are kept and the user is
  notified. Deletions propagate both ways. Deleted or overwritten files are kept
  in recovery storage for 30 days.
- Interrupted syncs resume or recover without silently discarding local work.
- Offline Repair is a separately confirmed recover-and-pause operation: it
  disables the schedule, stops the finite service and keeps the bisync lock and
  recovery state. It never syncs, resyncs, authenticates, deletes recovery state
  or uses Online FUSE Repair. For a legacy `abraunegg/onedrive` connection it
  stops only the continuous monitor and preserves token, database, lock, mirror
  and recovery data. The resulting row state is read back from the coordinator
  and systemd, not assumed from a successful stop.
- Google Docs, Sheets and Slides are excluded because their exports cannot be
  safely edited and round-tripped. The applet lists skipped documents and points
  the user to Google Drive in a browser.

### Google Drive custom OAuth client

Remote setup accepts an optional OAuth client ID and secret. New remotes may leave
them blank to use rclone's shared client, but the UI explains that a private
client avoids interruption when the shared client is retired. An existing remote
changes only when the user presses **Update Google OAuth Client**, which reruns
rclone's browser OAuth; active connections must then be remounted. The values are
transient: masked, redacted from command and error text, cleared after success or
failure, and never stored by the applet. Rclone owns the remote and token.

### Online directory-cache preload

Preload warms directory metadata after an Online mount so browsing is faster. It
is configured per provider in General Settings; SMB and SFTP connections can
instead override it per connection (**Use global preload settings** is the
default) so home, LAN, corporate and VPN shares need not share one compromise.

| Provider | Default | Maximum | Method |
|---|---|---:|---|
| Google Drive | On | 60 s | Asynchronous recursive `vfs/refresh` with fast-list |
| OneDrive | On | 60 s | Directory-only walk of the mounted tree |
| Box | On | 60 s | Directory-only walk, depth 2 |
| SharePoint | Off | 30 s | Directory-only walk, depth 2 |
| SMB | On | 60 s | Directory-only walk, depth 3 |
| SFTP | Off | 30 s | Directory-only walk, depth 2; skips server `/proc`, `/sys`, `/dev` |

Durations accept 5–600 seconds; depth, where offered, accepts 1–10.

- Browsing stays available throughout preload and there is no manual Cancel.
- App-managed walks list directories only: no file contents opened, no symlinks
  followed, no leaving the mounted filesystem.
- Google Drive waits for the mount and its private per-user RC socket, then
  submits the refresh as an asynchronous job that does not delay mount
  completion. Success requires both the top-level status and every
  `output.result` entry to be `OK`.
- Box and SMB do not use recursive `vfs/refresh`: live tests showed Box hitting
  HTTP 429 and SMB failing to stop promptly or keep partial cache work.
- Results appear as brief notices: completed, time limit reached, or skipped.
  Completion shows the requested scope was traversed, not that browsing got
  faster.

## VPN Connections

The applet supports storage dependencies on NetworkManager VPN profiles. Profiles
and credentials are configured outside the applet (for example with
`nm-connection-editor`). The applet lists saved profiles, and activates, observes
and deactivates the chosen one through NetworkManager D-Bus, never through `nmcli`
or a proprietary client. It never receives, stores, displays or forwards VPN
credentials.

- Cisco-compatible gateways use the NetworkManager OpenConnect plugin. For
  interactive authentication, NetworkManager asks a host `nm-applet` secret agent
  over D-Bus to show the gateway's full dialog, including multi-field password
  and second-factor prompts. Other VPN plugins use their own paths.
- Before activating an OpenConnect profile, the applet restarts the static
  `cosmic-mounter-openconnect-agent.service` through the systemd user-manager
  D-Bus and waits for the complete start job. If the unit is missing or fails,
  it stops and gives package and service-installation guidance. Non-OpenConnect
  profiles skip this step.
- A connection may define readiness checks (NetworkManager state, interface,
  route, DNS, endpoint reachability). Mounting or syncing starts only after they
  pass.
- Activation and readiness share one bounded deadline that includes interactive
  authentication. Transient failures are retried. Timeout or cancellation fails
  the attempt, and a late activation result cannot start storage. Repeated
  requests for one profile share one attempt and one prompt.
- The applet disconnects a VPN only if it activated it and no other active
  connection still needs it.

## Sleep and Wake

Two app-wide toggles in General Settings, both off by default:

- **Unmount when sleep** applies to Online connections only. The applet listens
  for system sleep preparation even with the popup closed, cancels pending Online
  operations and cleanly unmounts its Online connections. Offline mirrors and
  running syncs are unaffected, and shared VPNs are not disconnected.
- **Restore after wake up** is enabled only while the first toggle is on (its
  value is retained otherwise). It remounts only connections that were
  successfully unmounted for sleep, still enabled, after readiness checks pass.

The applet holds a bounded sleep-delay handle and releases it on completion or
deadline. Busy mounts or pending uploads can prevent cleanup; caches are kept,
forced or lazy unmount is never automatic, and a persistent summary reports
incomplete cleanup. Delivery before sleep cannot be guaranteed for a stuck
filesystem.

## User Interface

### Popup

The panel icon opens a compact popup: the title, a gear button (accessible name
**Settings**, delayed tooltip), runtime status, and the scrollable connection
list. Each row has a clickable name that opens the connection editor and a primary
control: Mount/Unmount for Online, Start/Stop background sync for Offline.
Configuration and secondary actions live in the editor. The status area keeps
showing operation feedback, errors and sleep-cleanup warnings, only when needed,
on one themed surface; an empty list directs the user to Settings.

### General Settings window

The gear opens or focuses one standalone **Cloud Mounter Settings** window with:
**Add Connection**, **Refresh**, the Preload settings, the two sleep toggles, and
sleep-listener status with the latest cleanup details.

- Settings and Refresh reach the running applet through an explicit runtime
  interface; changing only the settings process is not enough.
- With several panel instances, one owns the runtime service and the sleep
  listener and the others are clients without communication warnings.
- Background status polling leaves controls stable; controls go busy only for user
  actions. Closing a settings window leaves the applet running.
- Feedback stays where it belongs: background errors in the popup status,
  settings actions in General Settings, editor feedback in the editor.

### Connection editor

Add opens a standalone editor and Modify opens it prefilled; Import opens the
legacy-service workflow. For each connection the user sets: display name,
provider, mode, remote or rclone remote and optional subtree, mount point or
mirror directory, and optional VPN dependency and readiness checks. Online adds
start-at-login, cache size and safe-detach behavior; Offline adds preview,
interval, metered-network policy and recovery location.

- Test Connection and Save Connection are always offered. Add mode also offers
  Import and provider setup; Modify also offers Preview and Sync Now (Offline),
  Disable/Enable, and Remove. An Information section summarizes the engine,
  generated-unit validation and safety policy.
- Per-field help is a delayed tooltip on the relevant control; longer guidance
  belongs in documentation.
- Test and Save never silently rewrite rclone credentials; creating or updating a
  remote is an explicit action. Updating a shared remote names the affected saved
  connections and says active ones need restarting.
- The rclone remote-name field accepts a new name, a detected remote, or the exact
  name of a remote made with `rclone config`.
- Google Drive, Box, SMB and SFTP detect existing rclone remotes and can create
  them. Google Drive and Box delegate browser OAuth to rclone; SMB and SFTP
  secrets stay in rclone's mechanism or the SSH agent.
- OneDrive Online uses `jstaf/onedriver` with applet-owned config and cache
  paths. OneDrive Offline uses rclone's browser OAuth with its default client and
  applet-owned sync and recovery paths. The applet shows the signed-in account or
  drive for confirmation, binds the connection to that drive, and refuses to sync
  if a later run finds a different one. It stores no OneDrive tokens, client IDs
  or secrets.

## Provider Details

### SFTP

SFTP uses rclone's `sftp` backend for Online and Offline. Its editor follows the
SMB layout, with the **SFTP** provider button immediately right of **SMB**. Inputs:

- Remote name, detected remotes, and **Create/Update SFTP Remote** (the suggested
  action for a new remote; Update is highlighted until server or authentication
  edits are applied).
- Host, port (default 22) and username.
- Authentication: password, SSH private key (a file path, never pasted contents,
  with an optional passphrase) or SSH agent. Secret inputs are masked, transient,
  redacted from diagnostics and cleared after setup succeeds or fails.
- A known-hosts file, defaulting to the host user's `~/.ssh/known_hosts`. Unknown
  or changed host keys stop the operation with guidance; they are never silently
  trusted.
- Remote directory: empty means the server's login directory, relative paths are
  relative to it, and a leading `/` is an absolute server path. This distinction
  is preserved when saving, testing, mounting and syncing.

Generated host services must reach the same rclone config, key and known-hosts
files and agent socket used during setup, including from Flatpak. Missing agent
access gives an actionable failure, never an interactive service prompt. SFTP
reuses the standard access test, health, unmount, repair, sleep, VPN and mirror
safeguards. A mount of the server root prunes `/proc`, `/sys` and `/dev` from
preload; these are absolute remote-path exclusions, not exclusions of ordinary
folders named `sys`.

### SharePoint document libraries

SharePoint libraries, including Teams files, are a provider distinct from
personal and business OneDrive, labeled **SharePoint** beside OneDrive. rclone
calls its backend `onedrive`. A connection selects a site, one document library
on it, and optionally a folder in the library. A library URL such as
`https://example.sharepoint.com/sites/Team/Shared%20Documents` is only a locator:
the applet resolves the site's libraries, accepts only the one whose verified URL
matches, and stores the verified library ID, site and library URLs, remote and
folder separately. It never turns a URL segment into a mountpoint or assumes a
Teams channel is a library. Standard channels are folders in a site's library;
private or shared channels may use another site. Browsing all sites without a
known URL remains a later option; manual remote selection stays available.

Setup picks a preconfigured rclone `onedrive` remote or creates a new one through
browser sign-in (**Connect Microsoft Account**). A new remote name must be unused;
switching accounts creates a separate remote and never alters an existing
OneDrive one. Identity and access are verified before Save or Mount.

- **Online mount** is the first milestone, using the standard managed mount,
  access test, cache, unmount and recovery controls.
- **Offline mirror** has its own local directory, bisync state, preview and
  recovery data. It starts with manual, confirmed Preview and Sync Now, available
  after scoped disposable tests of conflicts, deletion recovery, Office-file
  rewrites and interrupted sync. Background scheduling stays disabled until
  unattended-safety checks pass, because indexing or applications that replace
  files on save can trigger unwanted uploads.

## Native and Flatpak Host Integration

Native and Flatpak installs share the same typed integration boundaries. Where
the host offers an adequate D-Bus API the applet uses it:

- NetworkManager for VPN discovery, state, activation and deactivation;
- the systemd user manager for applet-owned services and timers and the fixed
  OpenConnect agent service;
- logind for sleep preparation;
- the Documents portal for host-path resolution when the portal supports it.

The Flatpak does not route a host operation through `flatpak-spawn --host` when
such an API exists. Host process execution is limited to the external storage
engines, required FUSE helpers and documented, version-gated fallbacks. Each has
a fixed executable, separately validated arguments, bounded output and lifetime,
cancellation and secret redaction, and is justified in
`docs/engineering/flatpak-host-command-inventory.md`. On hosts with a Documents
portal older than version 5, a narrowly gated `flatpak document-info` fallback may
remain until the minimum supported portal provides `GetHostPaths`.

Manual and scheduled rclone mirrors share one Rust coordinator. Native and Debian
services run the installed restricted worker; Flatpak services use a fixed
`flatpak run --command` entry point. No bisync shell wrapper runs. Read-only
remote and mount inspection is asynchronous and bounded; stale editor results are
rejected and fresh observations are used for safety-critical decisions.

## Legacy Import and Removal

By default the applet scans `~/.config/systemd/user/` for compatible legacy rclone
mount and `jstaf/onedriver` unit files. Parsing is structural: it reads and
tokenizes `ExecStart` text and never executes it.

An import preview shows the parsed provider, remote or account placeholder,
subtree, local target, cache directory, startup behavior, unsupported options,
active-service conflicts and local-target conflicts. Confirming creates an
applet-managed connection and generated unit. Original legacy units are preserved;
disabling one is a separate confirmed action.

Generated units carry an applet ownership marker and the connection UUID.
Removing a connection removes only applet-owned unit files. Credentials, cloud
data, mirror data, caches, recovery directories and legacy units are preserved
unless a separate, explicitly confirmed cleanup says otherwise.

## Failure Handling

- Long-running storage and VPN operations never block the interface.
- An offline or unavailable provider is a connection state, not an application
  failure.
- Mount and sync retries use bounded backoff.
- A mount with pending writes is never force-unmounted without explicit
  confirmation and a clear data-loss warning.
- A failed sync preserves its state and says whether user action, authentication,
  storage space or recovery is needed.
- Logs and notifications never expose passwords, tokens or other credentials.

## References

- [rclone mount](https://rclone.org/commands/rclone_mount/)
- [rclone bisync](https://rclone.org/bisync/)
- [rclone downloads](https://rclone.org/downloads/)
- [jstaf/onedriver](https://github.com/jstaf/onedriver)
