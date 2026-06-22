# COSMIC Cloud Mounter

<img src="./resources/icon.svg" alt="COSMIC Mounter" style="float: left; margin-right: 15px; width: 100px;">

COSMIC Cloud Mounter is a COSMIC desktop applet for managing storage
connections to *OneDrive, Google Drive, Box,* and *SMB*. It supports direct Online
mounts and Offline mirrors with background synchronization.

The applet was developed so users can turn cloud storage connections on and off
to reduce file manager stalls when the network is slow or unavailable.

## Modes and Providers

**Online mount** uses a network-backed FUSE filesystem. It is useful for browsing
large remote trees without keeping a full local copy.

**Offline mirror** uses an ordinary local directory plus bidirectional sync. Initial
sync requires a dry Preview and confirmed Sync Now. Automatic background sync
pauses on metered networks by default.

| Provider | Online mount | Offline mirror |
|---|---|---|
| OneDrive | `jstaf/onedriver` | `abraunegg/onedrive` |
| Google Drive | `rclone mount` | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |

<table>
  <tr>
    <td><img src="./resources/Popup.png" alt="COSMIC Cloud Mounter popup" width="150"></td>
    <td><img src="./resources/Add_Connection.png" alt="Add Connection window" width="200"></td>
    <td><img src="./resources/Change_Connection.png" alt="Modify Connection window" width="200"></td>
  </tr>
</table>

## Dependencies

The applet detects dependencies and provides guidance, but does not install or
upgrade them.

| Dependency | Purpose | Minimum for this project |
|---|---|---|
| `rclone` | Google Drive, Box, and SMB mounts and mirrors | 1.74.3 |
| `onedriver` by jstaf | OneDrive Online mount | 0.15.0 |
| `onedrive` by abraunegg | OneDrive Offline mirror | 2.5.10 |
| FUSE 3 / `fusermount3` | Online mounts | Required |
| NetworkManager / `nmcli` | Network and VPN readiness | Required |
| Cisco Secure Client | Optional Cisco VPN support | 5.1.10 tested |
| `fuser` from `psmisc` | Optional busy-mount diagnostics | Recommended |

Install and upgrade guidance is in
[Dependency Installation.md](Dependency%20Installation.md).

## Data Integrity Warning

Cloud sync and mounts can delete, overwrite, duplicate, or hide files when they
are configured incorrectly. **Before testing with important data, make an
independent backup**.

**Do not**:

- configure Online mount and Offline mirror simultaneously for the same
  provider account and overlapping remote subtree;
- use an Online mountpoint as an Offline mirror directory;
- use an Offline mirror directory as an Online mountpoint;
- use a `jstaf/onedriver` mountpoint as an `abraunegg/onedrive` sync directory;
- run `jstaf/onedriver` and `abraunegg/onedrive` concurrently against the same
  OneDrive account or overlapping subtree unless the applet has explicitly
  isolated that setup;
- enable a generic `onedrive.service` for an applet-managed OneDrive mirror;
- run `onedrive --resync` casually. State rebuilds require preview and
  confirmation when managed by the applet.

Offline mirror mode is the reliable option for uninterrupted local file access.
Online mounts can still block or fail when the provider, VPN, FUSE layer, or
network stalls.

## Applet Workflow

The panel popup shows active connection count, notification
state, VPN summary, `Add Connection`, `Refresh`, and a scrollable list of
connections.

Each connection row has the connection name and one primary state control:

- Online mounts use Mount or Unmount.
- Offline mirrors use Start or Stop for background synchronization.

Clicking the connection name opens `Modify`. `Preview` and `Sync Now` for Offline
mirrors are available from Modify.

`Add` and `Modify` share the same editor. Modify mode exposes `Test Connection`, `Save Connection`, `Preview` and `Sync Now` for Offline mirrors, `Disable` or `Enable`, and
`Remove`. The Information section summarizes the selected engine, generated unit
validation, and confirmation policy.

## Authentication

The applet does not store provider credentials. Credentials stay with `rclone`,
`jstaf/onedriver`, `abraunegg/onedrive`, or the operating system.

For Google Drive and Box, applet-driven setup delegates browser OAuth to
`rclone`. For SMB, the password remains in rclone's credential mechanism, not
in applet configuration.

For OneDrive Online mount, the applet uses `jstaf/onedriver` with applet-owned
configuration and cache paths. For OneDrive Offline mirror, it uses
`abraunegg/onedrive` with applet-owned configuration, sync, and recovery paths.
The OneDrive mirror setup first attempts `onedrive --reauth` interactive
browser authorization. If Microsoft/browser redirect capture fails, the applet
offers a manual auth handoff/helper fallback.

## Recovery and Limitations

Offline mirrors preserve both versions of same-file conflicts. Deletions
propagate bidirectionally after preview and confirmation policy has been
satisfied. Deleted and overwritten files are moved into recovery locations and
retained by applet policy for 30 days.

Recovery retention is not a backup system.

Google Docs, Sheets, Slides, and related browser-native Google document types
are excluded from rclone Offline mirrors and remain browser-accessible.

Known limitations:

- `abraunegg/onedrive` personal-account Offline mirror testing has covered
  authorization, disposable subtree creation, dry-run preview, and one-shot
  upload sync. Conflict, deletion-retention, and long-running monitor behavior
  need broader manual coverage.
- Google Drive Online mount testing previously hit Google Drive API quota/rate
  limiting on the development machine.
- NetworkManager support currently uses fixed `nmcli` commands; direct D-Bus
  integration remains future work.

## VPN Integration

The applet can associate a connection with a NetworkManager VPN profile or
Cisco Secure Client dependency. VPN profiles and credentials are configured
outside the applet.

The applet may start a VPN dependency and wait for readiness checks before
mounting or syncing. It disconnects only a VPN it activated, and only after no
active connection still requires it.

## Connection Removal

Press **Remove** once to request confirmation, then press **Remove** again to
remove the connection.

Removal deletes the applet-managed connection record and any matching
applet-owned systemd user units. Units that do not carry this applet's ownership
marker for the selected connection are treated as external and are left
untouched. If `systemctl --user daemon-reload` fails, the unit file is restored
to avoid an inconsistent service state.

Connection removal does **not** delete provider credentials, cloud data, local
mirror data, caches, recovery directories, or original imported legacy service
files.

Unused rclone remotes can be removed separately from the Add Connection rclone
management area. That action requires confirmation and changes rclone
configuration, not only applet configuration.

## Build from Source

Requirements:

- Linux with the COSMIC desktop
- Rust 1.95.0 through rustup
- `just`
- native development packages required by libcosmic

The repository pins the Rust toolchain and libcosmic Git revision.

Common commands:

```sh
just fmt
just check
just lint
just test
just verify
just run
just stage
just install-user
```

Useful read-only examples:

```sh
cargo run --example dependency_inventory
cargo run --example legacy_import_preview
cargo run --example legacy_import_confirm_dry_run
```

`just stage` installs into `target/stage/usr` and does not modify the host
system. `just install-user` installs the development build under `~/.local` and
updates desktop metadata and icons for the current user.

To test staged uninstall:

```sh
just stage
just rootdir=target/stage prefix=/usr uninstall
```

## Project Development

This applet was developed with agent-assisted programming. The project starts
from [Applet Description.md](Applet%20Description.md), which is translated into
[Requirements and Specifications.md](Requirements%20and%20Specifications.md) including the Functional Requirements. They are reviewed by the author.
The requirements drive [Task List.md](Task%20List.md), and its execution history is
documented in [Task List Completion Notes.md](Task%20List%20Completion%20Notes.md). The author supervises each task and its verification.

## Contributing & Feature Requests

### Feature Requests

- Download the github repository.
- Update the Applet Description to include your request or create a document describing your reuqest and ask you AI agent to include it in the Applet Description.
- Have your AI agent check and update the Applet Description.
- Have your AI agent update the Requirements and Specifications based on the Applet Description.
- Verify the modifications to the Requirements and Specifications.
- Ask your AI agent to update the Functional Requirements based on your reviewed Requirements and Specifications.
- Have your AI agent add Tasks to the Task list based on the Specifications.
- Have your AI agent execute the addition to the Task list.
- Complete the verifications and test your implementation.
- Submit pull request.

### Bug Reports

- Submit report on Github,

## License

MIT, copyright Urs Utzinger and OpenAI Codex.
