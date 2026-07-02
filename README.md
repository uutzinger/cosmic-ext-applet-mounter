# COSMIC Cloud Mounter

<img src="./resources/icon.svg" alt="COSMIC Mounter" style="float: left; margin-right: 15px; width: 100px;">

COSMIC Cloud Mounter is a COSMIC desktop applet for managing storage
connections to *OneDrive, Google Drive, Box,* and *SMB*. It supports direct <u>Online
mount</u> and <u>Offline mirror</u> with background synchronization.

The applet was developed to simplify mounting cloud storage. Users can easily
turn storage connections on and off to reduce file manager stalls when the
network is slow or unavailable.

## Modes and Providers

**Online mount** uses a network-backed FUSE filesystem. It is useful for browsing
large remote trees without keeping a full local copy.

**Offline mirror** uses an ordinary local directory plus bidirectional sync. Initial
sync requires a dry Preview and confirmed Sync Now. Automatic background sync
pauses on metered networks by default.

The following connection engines are used to connect to the providers:

| Provider | Online mount | Offline mirror |
|---|---|---|
| OneDrive | [`jstaf/onedriver`](https://github.com/jstaf/onedriver) | [`abraunegg/onedrive`](https://github.com/abraunegg/onedrive) |
| Google Drive | [`rclone mount`](https://rclone.org/) | [`rclone bisync`](https://rclone.org/) |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |

Example screen shots of the applet:
<table>
  <tr>
    <td valign="top"><img src="./resources/Popup.png" alt="COSMIC Cloud Mounter popup" width="200"></td>
    <td valign="top"><img src="./resources/Add_Connection.png" alt="Add Connection window" width="275"></td>
    <td valign="top"><img src="./resources/Change_Connection.png" alt="Modify Connection window" width="275"></td>
  </tr>
</table>

## Installation and Removal

Until installation through COSMIC Store is available, install the Debian
package from the [latest GitHub release](https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/latest).
Version 0.3.0 currently provides an `amd64` package:

```sh
wget https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v0.3.0/cosmic-ext-applet-mounter_0.3.0_amd64.deb
sudo apt install ./cosmic-ext-applet-mounter_0.3.0_amd64.deb
```

The package installs the applet binary, OneDrive authentication helper, desktop
entry, AppStream metadata, and icon.

After installation, open **COSMIC Settings > Desktop > Panel > Applets** and add
**COSMIC Cloud Mounter** to the desired panel or dock. Install the external
storage engines needed for your providers as described under
[Dependencies](#dependencies).

Before uninstalling, use the applet to stop active mounts and mirrors. If you
also want to remove its generated user services and timers, remove each
connection from the applet first. Removing a connection does not delete cloud
data, local mirror data, provider credentials, caches, or recovery directories.

Remove COSMIC Cloud Mounter from the panel, then uninstall the package:

```sh
sudo apt remove cosmic-ext-applet-mounter
```

Package removal does not delete configuration and data in the user's home
directory. Any connection records or generated user services not removed
before uninstalling remain in place for a later reinstall or manual cleanup.

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

Install and upgrade guidance is provided here:
[Dependency Installation.md](Dependency%20Installation.md).

## Data Integrity Warning

Cloud sync and mounts can delete, overwrite, duplicate, or hide files when they
are configured incorrectly. **Before testing with important data, make an
independent backup**.

To reduce data integrity risks, **do not**:

- configure Online mount and Offline mirror simultaneously for the same
  provider account and overlapping remote subtree;
- use an Online mount point as an Offline mirror directory;
- use an Offline mirror directory as an Online mount point;
- run OneDrive Online mount and OneDrive Offline mirror concurrently against the same
  OneDrive account or overlapping subtree unless the applet has explicitly
  isolated that setup;
- run `onedrive --resync` casually. State rebuilds require preview and
  confirmation when managed by the applet.

Offline **mirror** mode is the reliable option for uninterrupted local file access.

Online mounts can block or fail when the provider, VPN, FUSE layer, or
network stalls.

## Applet Workflow

The panel popup shows active connection count, notification
state, VPN summary, `Add Connection`, `Refresh`, and a scrollable list of
connections.

Each connection row has the connection name and one primary state control:

- Online mount toggle button uses Mount or Unmount.
- Offline mirror toggle button uses Start or Stop for background synchronization.

Clicking the connection name opens `Modify`.

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

## Recovery and Limitations

Offline mirrors preserve both versions of same-file conflicts. Deletions
propagate bidirectionally after preview and confirmation policy has been
satisfied. Deleted and overwritten files are moved into recovery locations and
retained by applet policy for 30 days.

Recovery retention is not a backup system.

Google Docs, Sheets, Slides, and related browser-native Google document types
are excluded from rclone Offline mirrors and remain browser-accessible.

Known limitations:

- `abraunegg/onedrive` requires a GTK webview because it does not yet provide a
  method that hands the authorization redirect to this applet.
- Google Drive Online mount testing can hit Google Drive API quota/rate limiting.
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
- Rust 1.95.0 or later through rustup
- `just`
- native development packages required by libcosmic

The repository pins the Rust toolchain and libcosmic Git revision.

Common commands:

```sh
just fmt
just check
just lint
just test
just metadata-check
just verify
just run
just stage
just install-user
just deb
```

Useful read-only examples:

```sh
cargo run --example dependency_inventory
```

`just stage` installs into `target/stage/usr` and does not modify the host
system. `just install-user` installs the development build under `~/.local` and
updates desktop metadata and icons for the current user.

`just metadata-check` validates the desktop entry and AppStream metadata without
network access. `just metadata-check-net` additionally verifies published URLs
and screenshots. `just deb` builds a local unsigned Debian binary package in the
parent directory.

## Project Development

This applet was developed with agent-assisted programming. The project starts
from [Applet Description.md](Applet%20Description.md), which is translated into
[Requirements and Specifications.md](Requirements%20and%20Specifications.md) including the Functional Requirements. They are reviewed by the author.
The requirements drive [Task List.md](Task%20List.md), and its execution history is
documented in [Task List Completion Notes.md](Task%20List%20Completion%20Notes.md). The author supervises each task and its verification.

## Contributing & Feature Requests

### Feature Requests

You can implement additional features to this project using agentic programming;

- Download the github repository.
- Update the Applet Description to include your request or create a document describing your reuqest and ask you AI agent to include it in the Applet Description.
- Have your AI agent check and update the Applet Description.
- Have your AI agent update the Requirements and Specifications based on the Applet Description.
- Verify the modifications to the Requirements and Specifications.
- Ask your AI agent to update the Functional Requirements based on your reviewed Requirements and Specifications.
- Have your AI agent add Tasks to the Task list based on the Specifications.
- Have your AI agent execute the additions to the Task list.
- Complete the verifications and test your implementation.
- Submit pull request.

### Bug Reports

- Submit report on Github,

## License

MIT, copyright Urs Utzinger and OpenAI Codex.
