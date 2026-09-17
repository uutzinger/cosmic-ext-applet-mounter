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

### Tool selection

- `jstaf/onedriver` provides an on-demand OneDrive filesystem, local caching, and
  read-only access to previously opened files while offline.
- `abraunegg/onedrive` provides a complete local OneDrive mirror, continuous
  monitoring, bidirectional synchronization, conflict handling, and recovery
  safeguards.
- A current stable release of `rclone` provides the common mount and
  bidirectional synchronization engine for Google Drive, Box, and SMB.
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
| SMB | Yes | 60 seconds | Directory-only walk, maximum depth 3 |

Durations accept 5 to 600 seconds. Box and SMB depths accept a finite validated
range from 1 to 10. Google Drive and OneDrive use their provider-specific full
tree mechanisms and do not expose a depth value.

Each provider toggle has provider-specific hover help and visible spacing
between its label and switch. A provider's switch, seconds field, and optional
levels field remain on one horizontal row. Duration and recursive-depth details
and their accepted ranges appear only as delayed hover help. Duration help
clarifies that browsing remains available throughout preload; depth help
explains the additional server requests, rate-limit risk, and preload-time cost
of deeper scans. No standalone preload-description or range line is shown.

Each SMB Online connection defaults to **Use global preload settings**. The
connection editor can instead override preload enablement, maximum duration,
and depth for that connection. This supports independent home, LAN, corporate,
and VPN shares without requiring one compromise setting. Google Drive,
OneDrive, and Box use their provider-wide values.

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
- Manual unmount verifies that the mount-table entry disappears after the
  generated service stops. If a busy FUSE endpoint remains, the applet reports
  Error and exposes the existing two-step Repair confirmation instead of
  reporting completion or attempting to recreate the mountpoint. This applies
  to rclone mounts for Google Drive, Box, and SMB and to online OneDrive.
  Repair resets the generated service only when it is actually failed; an
  inactive successful service needs no reset.
- Completion, timeout, and sanitized failure results use the existing applet
  notice path. There is no manual Cancel control because ordinary browsing can
  continue after the bounded preload ends.

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
  OneDrive, Box, and SMB plus directory-depth defaults for Box and SMB.
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

For Google Drive, Box, and SMB, the applet can detect existing rclone remotes
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
