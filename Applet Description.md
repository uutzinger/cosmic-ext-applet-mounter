# COSMIC Cloud Mounter Applet

## Purpose

The COSMIC Cloud Mounter Applet manages cloud and network storage connections
from the COSMIC desktop panel. It allows users to choose between direct online
access and a local offline copy that synchronizes with the remote service.

The applet may also start a required VPN connection before accessing storage and disconnect it when it is no longer needed.

No network-backed filesystem can guarantee that a file manager will never block when a connection becomes slow or unavailable. Users who require uninterrupted file access shall use Offline mirror mode, where the file manager works only with ordinary local files.

This document is the source description for subsequent requirements,
specifications, and development tasks.

## Supported Storage Providers and Tools

Each storage connection uses exactly one access mode.

| Provider | Online mount | Offline mirror |
|---|---|---|
| Microsoft OneDrive | `onedriver` | `abraunegg/onedrive` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |

### Tool selection

- `onedriver` provides an on-demand OneDrive filesystem, local caching, and
  read-only access to previously opened files while offline.
- `abraunegg/onedrive` provides a complete local OneDrive mirror, continuous
  monitoring, bidirectional synchronization, conflict handling, and recovery
  safeguards.
- A current stable release of `rclone` provides the common mount and
  bidirectional synchronization engine for Google Drive, Box, and SMB.
- The applet requires current supported releases of its external tools. It
  detects missing or outdated dependencies and provides installation or upgrade
  guidance, but does not install or update software.

At the time of this revision:

- The installed rclone version is `1.60.1` and must be upgraded. The current
  stable release is `1.74.3`.
- The installed onedriver version is `0.15.0`, which is current.
- The current upstream `abraunegg/onedrive` release is `2.5.10`. The Ubuntu
  `2.4.25` package available on this machine is outdated.

## Connection Modes

Online mount and Offline mirror are mutually exclusive for each connection. A
directory configured for one mode shall not be reused by the other mode.

### Online mount

Online mount creates a network-backed FUSE filesystem. It provides on-demand
access without storing a complete local copy, but applications may wait when the network, VPN, or provider is slow.

- New connections start manually by default.
- The user may enable mounting at login.
- The applet uses bounded connection and operation timeouts.
- The applet monitors network, VPN, service, mount, and provider health.
- For rclone mounts, the applet monitors the VFS upload queue and cache state.
- When required connectivity is lost, the applet automatically detaches a mount only when no uploads or other writes are pending.
- If writes are pending, the applet warns the user, preserves the cache, and
  provides recovery and manual detach controls.
- When connectivity returns, the applet automatically remounts connections that remain enabled, after all network and VPN readiness checks pass.
- The applet provides explicit mount, unmount, retry, and repair actions.

An online mount's cache improves compatibility and reliability, but it is not a complete offline copy and shall not be described as one.

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
- Automatic synchronization pauses on metered networks by default. The user may run Sync Now or override the policy per connection.
- Local files remain readable and writable while offline.
- Local changes made while offline are synchronized after connectivity returns.
- If the same file changes on both sides, both versions are preserved and the
  user is notified.
- Deletions are propagated in both directions.
- Deleted or overwritten files are retained in recovery storage for 30 days,
  subject to a configurable storage limit.
- Interrupted synchronization resumes or recovers without silently discarding
  local work.

For Google Drive, cloud-native Google Docs, Sheets, and Slides are excluded from Offline mirror mode because exported representations cannot be safely edited and round-tripped. The applet lists skipped documents and directs the user to open them in Google Drive through a web browser.

## Supported VPN Connections

The applet supports storage dependencies on:

- Cisco Secure Client.
- VPN connections configured through the COSMIC desktop network settings and
  managed by NetworkManager.

VPN profiles and credentials are configured outside the applet. The applet
enumerates available profiles and allows a user to associate one profile with a storage connection.

A connection may define readiness checks such as active NetworkManager state,
interface presence, route presence, DNS resolution, or endpoint reachability.
Storage mounting or synchronization starts only after the checks pass.

Starting the Cisco VPN agent does not necessarily establish an authenticated
tunnel. The applet may start the agent and open Cisco Secure Client, then waits for the configured readiness checks while the user completes authentication.

The applet disconnects a VPN only when it activated that VPN and no other active storage connection still requires it.

## User Interface

Selecting the panel icon opens a popup showing every configured storage
connection and its status.

Each connection displays:

- Name and provider.
- Online mount or Offline mirror mode.
- Mount point or local mirror directory.
- Network and VPN readiness.
- Current mount or synchronization state.
- Pending uploads, conflicts, warnings, or errors.
- The last successful synchronization time for Offline mirror connections.

Online mount connections provide a mount/unmount toggle. Offline mirror
connections provide synchronization status, pause/resume, and Sync Now actions.

The popup provides access to connection settings and detailed diagnostics.

## Settings

The settings interface allows the user to add, edit, test, enable, disable, and remove storage connections.

For each connection, the user specifies:

- Display name.
- Storage provider.
- Online mount or Offline mirror mode.
- Remote account and optional remote subtree.
- Mount point or local mirror directory.
- Optional start-at-login behavior for Online mount.
- Automatic synchronization and metered-network behavior for Offline mirror.
- Optional VPN dependency and readiness checks.

The applet launches a tool's supported authentication interface when practical.
Otherwise, it presents provider-specific setup instructions and validates the
result before enabling the connection. Credentials remain in the selected
provider tool or operating-system secret store and are not copied into the
applet's configuration.

Advanced settings expose provider-specific cache, timeout, retry, bandwidth,
schedule, recovery, and storage limits. Safe defaults are selected
automatically.

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

## Starting Point

The repository contains manually created scripts and systemd user services that demonstrate the desired mount, unmount, VPN, and status workflows:

- `Cloud Drive Connections.md` records the existing setup process.
- `scripts/` contains the current shell-script workflows.
- `services/` contains the current rclone systemd user services.
- `applet template/` contains the COSMIC applet template.

These files are implementation references. The completed applet replaces the
desktop shortcuts and manual scripts, and creates or manages the required
per-user services automatically.

`Requirements and Specifications.md` and `Task List.md` shall be revised from
this description before application development begins.

## Verification References

- [rclone mount](https://rclone.org/commands/rclone_mount/)
- [rclone bisync](https://rclone.org/bisync/)
- [rclone downloads](https://rclone.org/downloads/)
- [onedriver](https://github.com/jstaf/onedriver)
- [OneDrive Client for Linux](https://github.com/abraunegg/onedrive)
