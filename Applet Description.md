# Cloud Mounter Applet for COSMIC™

## Purpose

The Cloud Mounter Applet manages cloud and network storage connections from the
COSMIC™ desktop panel. It allows users to choose between direct online
access and a local offline copy that synchronizes with the remote service.

The applet may also start a required VPN connection before accessing storage and
disconnect it when it is no longer needed.

No network-backed filesystem can guarantee that a file manager will never block
when a connection becomes slow or unavailable. Users who require uninterrupted
file access shall use Offline mirror mode, where the file manager works only
with ordinary local files.

This document is the source description for subsequent requirements,
specifications, and development tasks.

## Supported Storage Providers and Tools

Each storage connection uses exactly one access mode.

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | `jstaf/onedriver` | `abraunegg/onedrive` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |
| SFTP | `rclone mount` | `rclone bisync` |

SharePoint is a provider for document libraries, including Teams files, with separate Online mount and
Offline mirror milestones, described below; it is not in the supported matrix
until implementation and acceptance are complete.

### Tool selection

- `jstaf/onedriver` provides an on-demand OneDrive filesystem, local caching, and
  read-only access to previously opened files while offline.
- `abraunegg/onedrive` provides a complete local OneDrive mirror, continuous
  monitoring, bidirectional synchronization, conflict handling, and recovery
  safeguards.
- A current stable release of `rclone` provides the common mount and
  bidirectional synchronization engine for Google Drive, Box, SMB, and SFTP.
- The applet requires current supported releases of its external tools. It
  detects missing or outdated dependencies and provides installation or upgrade
  guidance, but does not install or update software.

## Connection Modes

Online mount and Offline mirror are mutually exclusive for each connection. A
directory configured for one mode shall not be reused by the other mode.

### Online mount

Online mount creates a network-backed FUSE filesystem. It provides on-demand
access without storing a complete local copy, but applications may wait when
the network, VPN, or provider is slow.

- New connections start manually by default.
- The user may enable mounting at login.
- The applet uses bounded connection and operation timeouts.
- Before starting an rclone Online mount, the applet verifies that the selected
  remote and subtree can be listed. Authentication and access failures stop the
  mount before a FUSE filesystem can block applications.
- Google Drive Online mount services expose a private per-user Unix RC socket.
  The recursive refresh request enables rclone's fast-list mode to reduce
  listing transactions at the cost of additional memory. The mount command
  does not receive `--fast-list`, which rclone ignores for mounts. Box and SMB
  do not receive this refresh behavior. Before starting an Online rclone
  connection, the applet regenerates its managed unit from the saved connection
  so updated provider options apply to existing connections on their next
  mount.
- After a Google Drive filesystem is mounted and its rclone RC/VFS endpoint is
  ready, the applet submits a recursive `vfs/refresh` as an asynchronous rclone
  job. Mount completion does not wait for this directory-metadata warm-up. The
  applet retains the job and rclone execution identifiers, tracks completion,
  and reports the result through its existing status notice.
- Manual unmount, repair, connection removal, and pre-sleep cleanup cancel an
  active Google Drive refresh before continuing the bounded detach sequence.
- The applet monitors network, VPN, service, mount, and provider health.
- For rclone mounts, the applet monitors the VFS upload queue and cache state.
- When required connectivity is lost, the applet automatically detaches a mount
  only when no uploads or other writes are pending.
- If writes are pending, the applet warns the user, preserves the cache, and
  provides recovery and manual detach controls.
- When connectivity returns, the applet automatically remounts connections that
  remain enabled, after all network and VPN readiness checks pass.
- The applet provides explicit mount, unmount, retry, and repair actions.

An online mount's cache improves compatibility and reliability, but it is not a
complete offline copy and shall not be described as one.

### Offline mirror

Offline mirror maintains a complete local copy of a selected remote folder or
the whole remote. Applications and the file manager access only the local
directory, so cloud connectivity cannot block normal browsing and editing.

- The user selects a remote subtree or the complete remote.
- Before the first synchronization, the applet estimates remote size and local
  disk requirements.
- The applet performs a dry preview and shows the expected uploads, downloads,
  deletions, and conflicts.
- The initial synchronization requires explicit user confirmation.
- Synchronization starts when required network and VPN connectivity becomes
  ready, runs periodically while online, and can be started manually.
- A provider client may use continuous local and remote change monitoring when
  it supports that behavior.
- Automatic synchronization pauses on metered networks by default. The user may
  run Sync Now or override the policy per connection.
- Local files remain readable and writable while offline.
- Local changes made while offline are synchronized after connectivity returns.
- If the same file changes on both sides, both versions are preserved and the
  user is notified.
- Deletions are propagated in both directions.
- Deleted or overwritten files are retained in recovery storage for 30 days.
- Interrupted synchronization resumes or recovers without silently discarding
  local work.

For Google Drive, cloud-native Google Docs, Sheets, and Slides are excluded from
Offline mirror mode because exported representations cannot be safely edited
and round-tripped. The applet lists skipped documents and directs the user to
open them in Google Drive through a web browser.

### Google Drive custom OAuth client

Google Drive remote setup accepts a user-provided OAuth client ID and its
matching client secret. New remotes may leave both fields blank for
compatibility with rclone's shared client, while the UI explains that a private
client is required to avoid interruption during the shared-client retirement.
An existing Drive remote changes only when the user presses
**Update Google OAuth Client**, which runs rclone's update and browser OAuth
flow. Active connections using an updated remote must be remounted.

The two OAuth values remain transient applet input: the secret is masked, both
values are redacted from displayed command and error text, and both fields are
cleared after success or failure. The applet does not copy them into its own
configuration. Rclone owns the persistent remote configuration and token.

### Online directory-cache preload

General Settings contains a **Preload** section with one provider row per
Online mount engine. Provider defaults are:

| Provider | Enabled | Maximum | Method |
|---|---:|---:|---|
| Google Drive | Yes | 60 seconds | Recursive asynchronous `vfs/refresh` with fast-list |
| OneDrive | Yes | 60 seconds | Directory-only walk of the mounted tree |
| Box | Yes | 60 seconds | Directory-only walk, maximum depth 2 |
| SharePoint | No | 30 seconds | Directory-only walk within the selected library or folder, maximum depth 2 |
| SMB | Yes | 60 seconds | Directory-only walk, maximum depth 3 |
| SFTP | No | 30 seconds | Directory-only walk, maximum depth 2; excludes server `/proc`, `/sys`, `/dev` |

Durations accept 5 to 600 seconds. Box, SharePoint, SMB, and SFTP depths accept a finite validated
range from 1 to 10. Google Drive and OneDrive use their provider-specific full
tree mechanisms and do not expose a depth value.

Each provider toggle has provider-specific hover help and visible spacing
between its label and switch. A provider's switch, seconds field, and optional
levels field remain on one horizontal row. Duration and recursive-depth details
and their accepted ranges appear only as delayed hover help. Duration help
clarifies that browsing remains available throughout preload; depth help
explains the additional server requests, rate-limit risk, and preload-time cost
of deeper scans. No standalone preload-description or range line is shown.

Each SMB or SFTP Online connection defaults to **Use global preload settings**. The
connection editor can instead override preload enablement, maximum duration,
and depth for that connection. This supports independent home, LAN, corporate,
and VPN shares without requiring one compromise setting. Google Drive,
OneDrive, Box, and SharePoint use their provider-wide values.

- Google Drive waits for the mount and private RC socket, then submits its
  recursive refresh. Job completion must inspect both the top-level RC status
  and every `output.result` entry; only `OK` result values are successful.
- OneDrive, Box, and SMB use app-managed walks that list directories without
  intentionally opening file contents, following symbolic links, or leaving
  the mounted filesystem. Box and SMB retain cache progress if their deadline
  expires.
- Box and SMB wait for both the mountpoint and a usable root listing. They do
  not use recursive `vfs/refresh` or fast-list. Live testing showed that Box
  recursive refresh reached HTTP 429 rate limiting and that SMB refresh did not
  stop promptly or retain partial cache work.
- Active preloads are tracked per connection. Unmount, repair, removal, and
  pre-sleep cleanup cancel and finish preload work before continuing their
  bounded detach sequence.
- Manual unmount displays progress while a running preload stops. Directory
  walkers must exit before clean detach. The applet attempts clean FUSE detach
  while the generated service is still running; if another process holds the
  mount busy, it reports failure and leaves the service and mount available for
  retry. Brief retries cover a handle released just after preload cancellation;
  the mount table decides whether detach succeeded even if the helper reports
  failure. It then stops the service and verifies that the mount-table entry has
  disappeared. A stopped service with a lingering endpoint uses the existing
  confirmed Repair flow. This applies to rclone mounts and online OneDrive.
  Repair resets the generated service only when it is actually failed; an
  inactive successful service needs no reset.
- All preload mechanisms share brief notices stating completed (with depth where
  applicable), time limit reached with an incomplete scan, or skipped. Omit timing
  figures and routine explanatory text so notices can be read before dismissal. Elapsed time may exceed the limit
  while work stops. SFTP targets inside excluded system directories are reported
  as skipped. Completion demonstrates traversal of the requested scope, not a
  measured browsing-speed improvement.
- Completion, timeout, and sanitized failure results use the existing applet
  notice path. There is no manual Cancel control because ordinary browsing can
  continue after the bounded preload ends.

## Supported VPN Connections

The applet supports storage dependencies on:

- Cisco Secure Client.
- VPN connections configured through the COSMIC desktop network settings and
  managed by NetworkManager.

VPN profiles and credentials are configured outside the applet. The applet
enumerates available profiles and allows a user to associate one profile with a
storage connection.

A connection may define readiness checks such as active NetworkManager state,
interface presence, route presence, DNS resolution, or endpoint reachability.
Storage mounting or synchronization starts only after the checks pass.

Starting the Cisco VPN agent does not necessarily establish an authenticated
tunnel. The applet may start the agent and open Cisco Secure Client, then waits
for the configured readiness checks while the user completes authentication.

The applet disconnects a VPN only when it activated that VPN and no other active
storage connection still requires it.

### Interactive VPN connection wait (A)

Cisco Secure Client is launched as an interactive application; the applet does
not kill its window after a short command timeout. The saved Cisco timeout
remains 90 seconds. Activation and readiness share one deadline, transient
readiness failures are retried, and a mount starts only after the tunnel is
connected and every configured readiness check passes. Timeout fails the mount
attempt and the user can retry with the existing mount control. VPN detection
preserves customized timeouts and readiness checks. Dedicated progress and
Cancel/Retry controls remain a planned UI improvement.

## Unmount Before Sleep (B)

An app-wide **Unmount when sleep** toggle is in the General
Settings window and defaults to off. Its help specifies Online connections
only. Version 0.4.5 moves this control out of the popup connection list. When enabled,
the applet listens for system sleep preparation even while the popup is closed,
cancels pending Online mount operations, and attempts clean unmount of its
Online connections, including OneDrive and SMB. Offline mirrors and their
running synchronization jobs are unaffected. Sleep cleanup does not disconnect
shared VPNs.

The applet holds a bounded sleep-delay handle and releases it on completion or
deadline. Busy mounts or pending uploads can prevent cleanup. Caches are
preserved, forced/lazy unmount is not automatic, and a persistent summary reports
incomplete cleanup. This cannot guarantee that a stuck filesystem releases
before sleep. App-owned runtime service drop-ins prevent forced killing during
cleanup; these protections last for the user runtime session.

A separate **Restore after wake up** toggle in General Settings defaults to
off. Show it below Unmount when sleep, disabled while that option is off; retain
its saved value and explain that it restores only previously active Online
connections successfully unmounted for sleep. When enabled, connections successfully unmounted for sleep are
restored only if still enabled, after network and VPN readiness checks pass.
Saved login policies are preserved. Implementation is locally tested; live
native/Flatpak acceptance remains tracked in `Task List.md` under “VPN
Authentication Wait and Sleep Cleanup”.

## User Interface

### Compact popup — September 15, 2026

Selecting the panel icon opens a compact popup containing the title **Cloud
Mounter** aligned left, a clickable gear aligned to the right edge of the
title row, runtime status, and the
scrollable connection list. The gear has the accessible name **Settings** and a
one-second delayed tooltip, matching connection-list help, explaining what
it opens. Add Connection, Refresh, and both sleep settings move into a
separate **Cloud Mounter Settings** window. The implementation is locally
verified; desktop and Flatpak visual acceptance remains pending.

Each connection row retains its clickable name and primary state control.
The name opens its existing Add/Modify editor. Online mounts expose Mount or
Unmount; Offline mirrors expose Start or Stop background synchronization.
Static configuration and secondary operations remain in the connection editor.

The popup's status area must continue to show relevant operation feedback,
errors, and sleep-cleanup warnings. The existing notice area is shared by more
than Add Connection and Refresh, so moving those buttons must not remove it.
Show notices only when needed; do not reserve an empty toolbar or settings
footer. Use one continuous themed surface with a thin horizontal divider
between status/notices and connections, without an empty background band. In the empty state, direct the user to Settings to add a connection.

### General Settings window

The gear opens or focuses one standalone window titled **Cloud Mounter
Settings**, following the existing standalone connection-editor architecture.
Its top row contains **Add Connection** and **Refresh**, with Preload and Sleep
sections below. The content surface uses the same COSMIC themed list background
as the popup and Add/Modify windows. Button action messages appear directly
below the top button row. The sleep section ends with “Applies only to Online connections.”
and plain-text sleep feedback immediately below it, without a separate status
heading or fixed-height status box. Long messages wrap within the window's
scrollable content. Content padding sits inside the scrollable region so the
vertical scrollbar reaches the window's right edge. It offers:

- **Unmount when sleep** (Online connections only, default off).
- **Restore after wake up** (default off, enabled only when unmount-on-sleep is
  enabled, with its saved preference retained).
- **Add Connection**, opening the existing standalone connection wizard.
- **Refresh**, reloading the running applet's configuration and refreshing its
  runtime status, with completion/failure feedback in General Settings.
- **Preload**, with enabled and maximum-duration settings for Google Drive,
  OneDrive, Box, SharePoint, SMB, and SFTP plus directory-depth defaults for
  Box, SharePoint, SMB, and SFTP. The SFTP row appears immediately below SMB.
- Sleep-listener status and the latest cleanup details.

Settings changes and Refresh must reach the running applet through an explicit
runtime communication interface; changing only the settings process is
insufficient. Periodic background status checks leave controls visually stable;
controls become busy only for user actions. Sleep controls also require an
available runtime owner. When Cloud Mounter appears on multiple panels, one
instance owns the runtime service and the other instances use it as clients
without displaying a communication warning. The runtime owner remains the sole
owner of the sleep listener. Closing either settings window leaves it running. Background
mount/sync errors stay visible in popup status; settings-action feedback stays
in General Settings, and connection-editor feedback stays in its editor.

```text
Main popup                      Cloud Mounter Settings
Cloud Mounter          [gear]    [Add Connection] [Refresh]
Status / relevant notice        Button action feedback
                                Sleep and wake
Connection A             [on]   [ ] Unmount when sleep
Connection B            [off]   [ ] Restore after wake up
                                Applies only to Online connections.
                                Sleep feedback / cleanup details
```

## Add, Modify, Import, and Information

Add Connection opens a standalone connection editor. Modify opens the same
editor prefilled for an existing connection. Import opens a dedicated legacy
service import workflow.

For each connection, the user specifies:

- Display name.
- Storage provider.
- Online mount or Offline mirror mode.
- Remote account or rclone remote and optional remote subtree.
- Mount point or local mirror directory.
- Optional start-at-login behavior for Online mount.
- Cache size and safe detach behavior for Online mount.
- Preview, synchronization interval, metered-network behavior, and recovery
  location for Offline mirror.
- Optional VPN dependency and readiness checks.

The editor provides Test Connection and Save Connection actions. Add mode also
provides Import and provider setup actions where applicable. Modify mode also
provides Preview and Sync Now for Offline mirrors, Disable or Enable, and
Remove. A compact Information section at the bottom summarizes the selected
engine, generated unit validation, and safety or confirmation policy.

Per-field help is attached to the relevant input, button, choice, or chip as a
tooltip where the toolkit supports it. Longer dependency, safety, and
troubleshooting guidance belongs in documentation.

The rclone remote-name field accepts a new name for the provider's Create
Remote action, a detected existing remote, or the exact name of a remote
configured with `rclone config`. Its tooltip explains both paths. SMB uses
Create/Update SMB Remote to create a remote or update an existing one.

For Google Drive, Box, SMB, and SFTP, the applet can detect existing rclone remotes
and can start applet-driven remote creation. Google Drive and Box setup delegate
browser OAuth to rclone. SMB credentials remain in rclone's credential
mechanism, not in applet configuration.

Google Drive creation also accepts a matching custom OAuth client ID and secret.
Modify mode exposes a separate **Update Google OAuth Client** action for an
existing Drive remote. That action reauthorizes the remote in the browser;
active connections using it must be remounted afterward. The transient fields
are cleared after the operation, and the applet never stores their values in
its own configuration.

For OneDrive Online mount, setup uses `jstaf/onedriver` with applet-owned
configuration and cache paths. For OneDrive Offline mirror, setup uses
`abraunegg/onedrive` with applet-owned configuration, sync, and recovery paths.
The applet first attempts the interactive browser authorization flow and offers
a manual auth handoff/helper fallback when the redirect cannot be captured
cleanly.

Credentials remain in the selected provider tool or operating-system secret
store and are not copied into the applet's configuration.

### SFTP connections — September 29, 2026

SFTP extends the supported provider scope using rclone's `sftp` backend for
Online mount and Offline mirror. The Add/Modify editor and provider wiring are
implemented; installed-service and live-server acceptance remain pending in
`Task List.md`.

In Add Connection, place the **SFTP** provider button immediately to the right
of **SMB**, with matching styling, spacing, and selection behavior. Keep the
buttons adjacent at the default window width. The SFTP editor follows the SMB
layout, section order, top action row, field help, and Test Connection / Save
Connection workflow. Use SFTP-specific inputs rather than SMB domain or
workgroup fields:

- Remote name, detected existing SFTP remotes, and **Create/Update SFTP Remote**.
  For a new remote in Add mode, emphasize this action with the blue/theme-accent
  suggested-button style. Server/authentication edits also highlight Update in
  Add and Modify until successfully applied; failures and newer edits retain
  the highlight. Save Connection stays blue and can save the applet's settings
  without a successful access test; it does not apply remote credentials.
  Password, SSH Key, and SSH Agent choices each provide
  delayed hover help explaining when and how to use that method.
- Server host, port (default 22), and username.
- Authentication choice: password, SSH private key, or SSH agent. Password and
  optional key passphrase inputs are masked and transient; key authentication
  uses a private-key file path, not pasted key contents.
- A known-hosts file for server identity verification, defaulting to the host
  user's `~/.ssh/known_hosts`. Unknown or changed host keys stop the operation
  with actionable guidance; the applet does not silently trust them.
- Remote directory, local mountpoint or mirror directory, access-mode options,
  and optional VPN dependency, following the existing SMB layout.

An empty remote directory selects the server's login directory. Relative paths
are relative to that directory; a leading `/` selects an absolute server path
within the account's permitted filesystem. Preserve that distinction when
saving, testing, mounting, and synchronizing.

As with SMB, Add provides **Create/Update SFTP Remote**, while Modify provides
**Update SFTP Remote** for an existing remote. Modify retains the locked
provider/mode policy and prefills non-secret server settings from rclone. Creating or updating a remote is explicit; Test and Save do
not silently rewrite rclone credentials. Updating a shared remote identifies
affected saved connections and explains that active connections need restarting.
Secrets stay with rclone or the SSH agent, are redacted from diagnostics, and
are cleared from the editor after setup succeeds or fails.

SFTP reuses the existing bounded access test, mount/cache health, safe unmount,
repair, sleep/wake, VPN, and Offline mirror safeguards. Generated host user
services must access the same rclone configuration, key/known-hosts files, and
agent socket used during setup, including from Flatpak. Missing agent access
must produce actionable failure rather than an interactive service prompt.
SFTP directory preload is optional and defaults to off, 30 seconds, and depth 2.
General Settings places the SFTP preload row directly below SMB; each SFTP Online
connection can inherit the global policy or override enabled state, duration,
and depth. Preload uses a bounded directory-only walk, never follows symbolic
links, and cancels before unmount, repair, removal, or sleep. A server-root mount
prunes `/proc`, `/sys`, and `/dev` before descending; a mount of one of those
directories or its descendants is not preloaded. These are remote absolute-path
exclusions, not blanket exclusions of ordinary project folders named `sys`.
Paths relative to the login directory cannot be mapped to server absolute paths
without server information. Ordinary on-demand browsing remains available.

### SharePoint connections — September 30, 2026

SharePoint document libraries, including files presented in Microsoft Teams,
are a planned provider distinct from personal or business OneDrive connections.
The editor and provider button label it **SharePoint**, beside OneDrive. A connection selects a
SharePoint site, one document library on that site, and optionally a folder
within the library. For example,
`https://emailarizona.sharepoint.com/sites/ENGR-BME-Assessment/Shared%20Documents`
identifies the `ENGR-BME-Assessment` site and its `Shared Documents` library;
the URL is not itself a remote folder path. The applet must verify the selected
library rather than infer its identity solely from a URL or remote name.

Keep **Online mount** and **Offline mirror** as distinct, mutually exclusive
access modes for each SharePoint connection. Online mount is the first planned
milestone: use rclone's Microsoft OneDrive backend configured for the selected
SharePoint document library, with the applet's existing managed mount, access
test, cache, unmount, and recovery controls. Setup may select a preconfigured
rclone remote or use **Connect Microsoft Account** to create a separate remote
through rclone's browser OAuth. The applet resolves the site's document libraries
and accepts only the one whose verified URL matches the entered library URL.
The applet shows **SharePoint** for document-library storage, including
Microsoft Teams files; rclone calls its backend `onedrive`.

Offline mirror uses the verified library's rclone remote with a separate local
directory and bisync state. The University's tenant requires administrator
approval for the separate `abraunegg/onedrive` app, so that client cannot
currently authenticate to this work library. SharePoint Offline provides
manual Preview and Sync Now after scoped disposable tests of conflicts,
deletion recovery, Office-file rewrites, and interrupted sync. Background
scheduling remains disabled pending unattended safety checks.

The Add/Modify workflow accepts the library URL as a locator, then lets the
user choose a preconfigured rclone `onedrive` remote or create a new one with
browser sign-in. A new remote name must be unused; switching accounts creates
a separate remote rather than changing an existing OneDrive remote. The applet
checks authenticated document-library identity and access before Save or Mount.
It stores the verified library ID,
site/library URLs, selected remote, and optional folder separately; it does not
turn `Shared%20Documents` into a local mountpoint or assume a Teams channel is
a library. Standard Teams channels are folders in a site's library, while
private or shared channels may use another SharePoint site. Browsing all sites
without a known URL remains a later option; manual remote selection remains
available.

Offline mirror reuses the selected rclone library remote but has its own bisync
state, local directory, preview, and recovery data. It starts with manual, confirmed
sync on disposable data. Background scheduling becomes available only after
SharePoint-specific overwrite, conflict, deletion, and interruption checks
pass; indexing or applications that replace files during save can otherwise
trigger unwanted uploads. Credentials stay with the external engines.

## Legacy Import and Removal

The applet scans `~/.config/systemd/user/` by default for compatible legacy
rclone mount and `jstaf/onedriver` service files. Import parsing is structural:
the applet reads unit text, tokenizes supported `ExecStart` forms, and never
executes imported commands.

Import previews show the parsed provider, remote or account placeholder, remote
subtree, local target, cache directory, startup behavior, unsupported options,
active-service conflicts, and local-target conflicts. Confirming an import
creates the applet-managed replacement connection and applet-owned generated
unit directly. Original legacy units are preserved by default; disabling an
original is a separate confirmed action.

Generated units contain applet ownership markers and the connection UUID.
Removing an applet-owned connection removes only applet-owned generated unit
files. Credentials, cloud data, local mirror data, caches, recovery directories,
and original legacy service files are preserved unless a separate explicitly
confirmed cleanup action says otherwise.

## Connection and Failure Handling

- Long-running storage and VPN operations do not block the applet interface.
- Offline or unavailable providers are reported as connection states rather than
  application failures.
- Mount and synchronization operations use retries with bounded backoff.
- The applet never force-unmounts a connection with pending writes without
  explicit user confirmation and a clear data-loss warning.
- A failed synchronization preserves its state and reports whether user action,
  authentication, storage space, or a recovery operation is required.
- Logs and notifications do not expose passwords, tokens, or other credentials.

## Verification References

- [rclone mount](https://rclone.org/commands/rclone_mount/)
- [rclone bisync](https://rclone.org/bisync/)
- [rclone downloads](https://rclone.org/downloads/)
- [jstaf/onedriver](https://github.com/jstaf/onedriver)
- [OneDrive Client for Linux](https://github.com/abraunegg/onedrive)
