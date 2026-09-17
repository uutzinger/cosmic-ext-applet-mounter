# Cloud Mounter for COSMIC™

<img src="./resources/icon.svg" alt="Cloud Mounter" style="float: left; margin-right: 15px; width: 100px;">

Cloud Mounter is an applet for the COSMIC™ desktop for managing storage
connections to **OneDrive**, **Google Drive**, **Box**, and **SMB**. It supports
direct **Online mount** and **Offline mirror** modes with background
synchronization.

The applet simplifies mounting cloud storage. Users can turn storage
connections on or off to reduce file manager stalls when the network is slow or
unavailable. The applet attempts to pre-cache directory metadata when available.

## Table of Contents

- [Applet and Settings](#applet-and-settings)
- [Modes and Providers](#modes-and-providers)
- [Installation and Removal](#installation-and-removal)
  - [Installation from Source](#installation-from-source)
  - [Installation from Debian Package](#installation-from-debian-package)
  - [Installation from Flatpak](#installation-from-flatpak)
  - [Post Installation](#post-installation)
  - [Uninstallation](#uninstallation)
- [Data Integrity Warning](#data-integrity-warning)
- [Applet Workflow](#applet-workflow)
- [Settings](#settings)
- [Authentication](#authentication)
- [Conflict Recovery and Limitations](#conflict-recovery-and-limitations)
- [VPN Integration](#vpn-integration)
- [Connection Removal](#connection-removal)
- [Project Development](#project-development)
- [Contributing & Feature Requests](#contributing--feature-requests)
  - [Feature Requests](#feature-requests)
  - [Bug Reports](#bug-reports)
- [License](#license)
- [Appendix](#appendix)
  - [Build from Source](#build-from-source)
  - [Flatpak Packaging and Publication](#flatpak-packaging-and-publication)

## Applet and Settings

The popup shows connection status and connections. Click a connection name in the
popup to open its Modify window.
Use the gear at the top right
of **Cloud Mounter** to open **Settings**. **Add Connection** and **Refresh** are
at the top. Settings also offers **Unmount when sleep** and **Restore after wake up**.
Both default to off and apply only to applet-managed Online connections.
Settings also alows to activate **directory preload** of the connection entries when they are mounted as a background task.

## Modes and Providers

**Online mount** uses a network-backed FUSE filesystem. It is useful for browsing
large remote trees without keeping a full local copy.

**Offline mirror** uses an ordinary local directory plus bidirectional sync.
Automatic background sync pauses on metered networks by default.

The following connection engines are used to connect to the providers ([external dependencies](Dependency%20Installation.md)):

| Provider | Online mount | Offline mirror |
|---|---|---|
| OneDrive | [`jstaf/onedriver`](https://github.com/jstaf/onedriver) | [`abraunegg/onedrive`](https://github.com/abraunegg/onedrive) |
| Google Drive | [`rclone mount`](https://rclone.org/) | [`rclone bisync`](https://rclone.org/) |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |

Example screenshots of the applet and its separate windows:
<table>
  <tr>
    <td valign="top"><img src="./resources/Popup.png" alt="Cloud Mounter popup" width="200"></td>
    <td valign="top"><img src="./resources/Settings.png" alt="Cloud Mounter popup" width="200"></td>
    <td valign="top"><img src="./resources/Add_Connection.png" alt="Add Connection window" width="275"></td>
    <td valign="top"><img src="./resources/Change_Connection.png" alt="Modify Connection window" width="275"></td>
  </tr>
</table>

## Installation and Removal

Before installing, verify dependencies in
[Dependency Installation.md](Dependency%20Installation.md). Follow instructions to install the
external storage engines you plan to use. The applet does not install them for you.

### Installation from Source

See [Build from Source](#build-from-source) below.

### Installation from Debian Package

The [latest GitHub release](https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/latest)
provides an `amd64` Debian package:

```sh
wget https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v0.4.6/cosmic-ext-applet-mounter_0.4.6_amd64.deb
sudo apt install ./cosmic-ext-applet-mounter_0.4.6_amd64.deb
```

The package installs the applet binary, OneDrive authentication helper, desktop
entry, AppStream metadata, and icon.

### Installation from Flatpak

Flatpak packaging is being prepared for COSMIC repository submission. Until it is published through a public Flatpak remote, use the Debian package or build from source unless you are testing the Flatpak manifest locally.

### Post Installation

After installation, open **COSMIC Settings > Desktop > Panel > Applets** and add
**Cloud Mounter** to the desired panel or dock.

### Uninstallation

Before uninstalling, use the applet to stop active mounts and mirrors. If you
also want to remove its generated user services and timers, remove each
connection from the applet first. Removing a connection does not delete cloud
data, local mirror data, provider credentials, caches, or recovery directories.

Remove Cloud Mounter from the panel, then uninstall the package:

When installed from Debian package:

```sh
sudo apt remove cosmic-ext-applet-mounter
```

When installed from Flatpak:

```sh
flatpak uninstall io.github.uutzinger.cosmic-ext-applet-mounter
```

When installed from source with `just install-user`, remove the installed user
files:

```sh
rm -f ~/.local/bin/cosmic-ext-applet-mounter
rm -f ~/.local/bin/cosmic-ext-applet-mounter-onedrive-auth-helper
rm -f ~/.local/share/applications/io.github.uutzinger.cosmic-ext-applet-mounter.desktop
rm -f ~/.local/share/metainfo/io.github.uutzinger.cosmic-ext-applet-mounter.metainfo.xml
rm -f ~/.local/share/icons/hicolor/scalable/apps/io.github.uutzinger.cosmic-ext-applet-mounter.svg
update-desktop-database ~/.local/share/applications 2>/dev/null || true
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
```

Package removal does not delete configuration and data in the user's home
directory. Any connection records or generated user services not removed
before uninstalling remain in place for a later reinstall or manual cleanup.

## Data Integrity Warning

Cloud sync and mounts can delete, overwrite, duplicate, or hide files when they
are configured incorrectly. **Before testing with important data, make an
independent backup**.

To reduce data integrity risks, **do not**:

- configure Online mount and Offline mirror simultaneously for the same
  provider account and overlapping remote subtree;
- use an Online mount point as an Offline mirror directory;
- use an Offline mirror directory as an Online mount point;
- run OneDrive Online mount and OneDrive Offline mirror concurrently against the
  same OneDrive account or overlapping subtree unless the applet has explicitly
  isolated that setup;
- run `onedrive --resync` casually. State rebuilds require preview and
  confirmation when managed by the applet.

Offline **mirror** mode is the reliable option for uninterrupted local file
access. Online mounts can block or fail when the provider, VPN, FUSE layer, or
network stalls.

## Applet Workflow

The panel popup shows the active connection count, notification state, VPN
summary, and a scrollable list of connections.

Each connection row has the connection name and one primary state control:

- Online mount toggle button uses Mount or Unmount.
- Offline mirror toggle button uses Start or Stop for background synchronization.

Clicking the connection name opens `Modify`.

## Settings

Open the Settings window by clicking the gear icon. `Add Connection` and
`Refresh` are available at the top. Add and Modify connections share the same
editor.

Add mode exposes `Test Connection`, `Save Connection`, `Import`, and the
provider-specific setup and rclone remote-management actions needed to create
or select a storage remote.

Modify mode exposes
`Test Connection`, `Save Connection`, `Preview` and `Sync Now` for Offline
mirrors, `Disable` or `Enable`, and `Remove`. The Information section
summarizes the selected engine, generated unit validation, and confirmation
policy.

Settings can `unmount` Online connections before the computer `sleeps` and restore
them after wake. A mounted share can block or delay the sleep process.

Settings can also enable `directory preloading`. Preload tasks do not open file
contents.

**Google Drive**: After its private Unix RC/VFS endpoint is ready, Cloud Mounter
starts a recursive directory-cache refresh as an asynchronous rclone job using
fast-list mode. Cloud Mounter tracks the job through completion.
Unmount, repair, connection removal, and sleep cleanup cancel an active refresh
before detaching the filesystem.

**OneDrive** starts a directory-only metadata preload after onedriver is ready.
The traversal does not follow symbolic links, stops at the configured deadline,
and is cancelled before unmount, repair, removal, or sleep.

**Box** waits for both the mountpoint and a usable root listing, then starts a
bounded directory-only metadata walk. It does not use recursive rclone RC
refresh or fast-list, which helps limit Box API requests and rate-limit risk.

**SMB** also waits for the mountpoint and root listing before starting its
bounded directory-only metadata walk. Each SMB connection can inherit the
global preload policy or override its enabled state, duration, and depth.
All provider preloads default to enabled with a 60-second maximum. Box uses a
bounded directory-only walk with depth 2 to avoid API rate limits; SMB uses
depth 3.
SMB connections may override enablement, duration, and depth
individually so local and VPN shares can use different policies.

## Authentication

The applet does not store provider credentials. Credentials stay with `rclone`,
`jstaf/onedriver`, `abraunegg/onedrive`, or the operating system.

For Google Drive and Box, applet-driven setup delegates browser OAuth to
`rclone`. For SMB, the password remains in rclone's credential mechanism, not
in applet configuration.

Google Drive setup accepts the client ID and matching client secret from a
Google Cloud Desktop OAuth application. Supplying both values creates the
remote with that private client; leaving both blank retains rclone's shared
client for compatibility. In Modify mode, **Update Google OAuth Client**
changes an existing Drive remote and opens browser authorization again.
The form masks the secret and clears it when the operation finishes.

For OneDrive Online mount, the applet uses `jstaf/onedriver` with applet-owned
configuration and cache paths. For OneDrive Offline mirror, it uses
`abraunegg/onedrive` with applet-owned configuration, sync, and recovery paths.

## Conflict Recovery and Limitations

Offline mirrors preserve both versions of same-file conflicts. Deletions
propagate bidirectionally after preview and confirmation policy has been
satisfied. Deleted and overwritten files are moved into recovery locations and
retained by applet policy for 30 days.

Recovery retention is not a backup system.

Google Docs, Sheets, Slides, and related browser-native Google document types
are excluded from rclone Offline mirrors and remain browser-accessible.

Known limitations:

- OneDrive Offline mirror setup uses `abraunegg/onedrive` authentication. The
  applet provides a helper flow for the Microsoft redirect and retains a manual
  handoff fallback because redirect handling can vary by browser and account
  type.
- Google Drive Online mount testing can hit Google Drive API quota/rate
  limiting.
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

Press **Remove** once to request confirmation, then press **Confirm Remove**
again to remove the connection.

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

## Project Development

This applet was developed with agent-assisted programming. The project starts
from [Applet Description.md](Applet%20Description.md), which is translated into
[Requirements and Specifications.md](Requirements%20and%20Specifications.md),
including the Functional Requirements. The author reviews these documents before
implementation. The requirements drive [Task List.md](Task%20List.md), and its
execution history is documented in
[Task List Completion Notes.md](Task%20List%20Completion%20Notes.md). The author
supervises and approves each task and its verification.

## Contributing & Feature Requests

### Feature Requests

You can implement additional features using agent-assisted programming. OpenAI Codex was used for the current version:

- Clone the GitHub repository.
- Ask your AI agent to read "Applet Description.md", "Requirements and Specifications.md".
- Ask the AI agent to update the Description and Requirements with the feature you want.
- Verify the modifications to the two files.
- Have your AI agent add Tasks to the Task list based on the updated Specifications.
- Have your AI agent execute the additions to the Task list.
- Make sure your AI agent updates Task List Completion Notes.
- Complete the verifications and test your implementation as instructed by your AI agent. Do not skip the testing.
- Submit a pull request to this repo.

### Bug Reports

- Submit a report on GitHub.

## License

MIT, copyright Urs Utzinger and OpenAI Codex.

# Appendix

## Build from Source

Requirements:

- Linux with the COSMIC™ desktop
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

`cargo run --example dependency_inventory` checks dependencies.

`just install-user` installs the development build under `~/.local` and
updates desktop metadata and icons for the current user. After updating,
close any Cloud Mounter settings/editor windows and restart the COSMIC panel
without logging out:

```sh
killall cosmic-panel
```

The panel should automatically restart and load the updated applet.

`just stage` installs into `target/stage/usr` and does not modify the host
system.

`just metadata-check` validates the desktop entry and AppStream metadata without
network access. Because the official COSMIC applet template currently uses
COSMIC-specific metadata fields that strict freedesktop validators report as
invalid or unknown, `just metadata-check` is non-fatal; use
`just metadata-check-strict` to see the raw validator result. `just
metadata-check-net` additionally checks published URLs and screenshots. `just
deb` builds a local unsigned Debian binary package in the parent directory.

## Flatpak Packaging and Publication

Flatpak packaging is intended for the COSMIC Flatpak repository because this is
an applet for the COSMIC™ desktop, not a general desktop application. The local
Flatpak has been built, installed, tested from a local repository, and verified
for AppStream discovery and uninstall behavior. It is not yet published through
a public Flatpak remote.

The project-owned manifest is:

```text
packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json
```

The manifest builds from the tagged source release and generated Cargo source
list. The COSMIC repository submission uses the `pop-os/cosmic-flatpak`
workflow:

```sh
cd ../cosmic-flatpak
just build io.github.uutzinger.cosmic-ext-applet-mounter
just build-changed
flatpak build-update-repo --generate-static-deltas --prune repo
```

The final manifest uses `flatpak-spawn --host` for approved host commands, so
host dependencies still must be installed separately: `rclone`, `onedriver`,
`onedrive`, `fusermount3`, `nmcli`, and any VPN clients used by configured
connections.

The tested Flatpak design does **not** require `--filesystem=host`. It uses
narrow app-specific grants for:

- native-visible applet configuration for the COSMIC™ desktop;
- app-owned engine configuration/cache/state;
- generated user systemd units;
- host COSMIC™ theme files for standalone settings windows.

Existing native/source/Debian applet configuration is shared with the Flatpak
prototype through:

```text
~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document
```

**Do not** run native and Flatpak instances at the same time. Both can see the same
connection configuration and manage the same generated user services, so
concurrent instances can race on mount, sync, and service state. Switching
package formats should be done by stopping the running applet first, then
starting the other package format.

To regenerate the reproducible Flatpak source list after dependency changes:

```sh
just flatpak-cargo-sources
```
