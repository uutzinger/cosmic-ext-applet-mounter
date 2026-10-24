# Cloud Mounter for COSMIC™

<img src="./resources/icon.svg" alt="Cloud Mounter" style="float: left; margin-right: 15px; width: 100px;">

Cloud Mounter is an applet for the COSMIC™ desktop for managing storage
connections to **OneDrive**, **SharePoint**, **Google Drive**, **Box**, **SMB**, and **SFTP**. It supports
direct **Online mount** and **Offline mirror** mode.

**0.5.5** updates connection controls, repairs Google Drive and Box browser
authentication, improves Google directory preload, and retires the legacy
abraunegg/onedrive mirror engine. See the [0.5.5 release notes](docs/releases/v0.5.5.md)
for upgrade guidance and validation limits. Version 0.5.5 was published on
October 22, 2026, with Debian packages and Flatpak bundles.

The applet simplifies mounting cloud storage. Users can turn storage
connections on or off to reduce file manager stalls when the network is slow or
unavailable.

Example screenshots of the applet and its separate windows:

<table>
  <tr>
    <td><a href="./resources/PopUp.png" target="_blank" rel="noopener noreferrer"><img src="./resources/PopUp.png" alt="Cloud Mounter popup" width="275"></a></td>
    <td>
    <a href="./resources/Settings.png" target="_blank" rel="noopener noreferrer"><img src="./resources/Settings.png" alt="Settings window" width="275"></a>
    </td>
    <td><a href="./resources/Add_Connection.png" target="_blank" rel="noopener noreferrer"><img src="./resources/Add_Connection.png" alt="Add Connection window" width="350"></a></td>
  </tr>
</table>

## Modes and Providers

**Online mount** uses a network-backed FUSE filesystem. It is useful for browsing
large remote trees without keeping a full local copy.

**Offline mirror** uses an ordinary local directory plus bidirectional background sync.

The following connection engines are used to connect to the providers:

| Provider | Online mount | Offline mirror |
|---|---|---|
| OneDrive | [`jstaf/onedriver`](https://github.com/jstaf/onedriver) | [`rclone bisync`](https://rclone.org/) |
| Google Drive | [`rclone mount`](https://rclone.org/) | `rclone bisync` |
| Box | `rclone mount` | `rclone bisync` |
| SMB | `rclone mount` | `rclone bisync` |
| SFTP | `rclone mount` | `rclone bisync` |
| SharePoint | `rclone mount` | `rclone bisync`  |

`rclone` handles all Offline mirrors and all Online mounts except OneDrive.
`jstaf/onedriver` provides on-demand OneDrive Online access.

## Data Integrity Warning

Cloud sync and mounts can delete, overwrite, duplicate, or hide files when they
are configured incorrectly.

**Before testing with important data, make an independent backup**.

To reduce data integrity risks, **do not**:

- use an Online mount point as an Offline mirror directory;
- use an Offline mirror directory as an Online mount point;
- run Online mount and Offline mirror concurrently against the
  same account or overlapping subtree unless the applet has explicitly
  isolated that setup;
- run `rclone mount`, `rclone bisync`, `onedriver`, `onedrive --sync`, or
  `onedrive --resync` directly against an applet-managed connection. Use the
  applet's Mount, Unmount, Preview, Sync Now, Start, Stop, and repair controls so
  its safety checks and synchronization state remain consistent.

## Installation

The [documentation index](docs/index.md) links to setup guides, release notes,
and engineering records.

[Release 0.5.5](https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/tag/v0.5.5)
is the current published release and provides Debian packages and Flatpak bundles.
The download installers below default to **0.5.5** and accept an explicit
`--version X.Y.Z` override. A PR to the COSMIC Flatpak repository is planned;
GitHub bundles do not imply COSMIC repository approval.

When replacing an existing installation, stop its connections, close its windows
and remove it from the panel first. Run only one native or Flatpak edition at a
time; they share saved connections and host services.

### Dependencies

Both Debian and Flatpak installations require storage engines on the host.
The supported engines are **rclone >= 1.74.3** and **onedriver >= 0.15.0**;
Online mounts also require FUSE3. The dependency installer sets up these engines:

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/scripts/install-dependencies.sh | bash
```

For individual engines and VPN setup, see [Dependency installation](docs/dependency-installation.md).
The applet does not install these dependencies itself.
For a Flatpak installation, install Flatpak through your distribution first.

### Installation from Source

See [Build from Source](docs/build-from-source.md).

### Installation from GitHub Release: Debian Package

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/scripts/install-debian.sh | bash
```

### Installation from GitHub Release: Flatpak

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/scripts/install-flatpak-release.sh | bash
```

### Post Installation

After installation, open **COSMIC Settings > Desktop > Panel > Applets** and add
**Cloud Mounter** to the desired panel or dock.

In Flatpak, select mirror and recovery directories separately using their
**Browse** buttons.

## Applet Workflow

The panel popup shows a scrollable list of connections and a brief status line
when attention is needed.

Each connection row has the connection name and one primary state control:

- Online mount toggle button uses Mount or Unmount.
- Offline mirror toggle button uses Start or Stop for background synchronization.

Clicking the connection name opens `Modify` window.

The connection mounting state is persistent over login: An Online mount left
on is remounted at the next login; switching it off unmounts it and keeps it off
after reboot. Offline mirror Start and Stop likewise enable or disable the
background schedule across logins.

A small count beside the name shows pending work when an engine can estimate it;
`+` means active without a count and `-` means unknown or unavailable.

## Settings

Open the Settings window by clicking the gear icon. **Add Connection**,
**Remove Remote**, and **Reorder** are available under **Connections**.
Add and Modify connections share the same editor.

In Add mode, use **Detect** to find existing remotes or **Connect** to set up a
new one. Enter the connection details, run **Test**, then **Save**. An existing
OneDrive Offline remote is verified as part of Test. Choose its matching
Personal or Work or school account type and enter the expected account email.

Modify mode uses **Re-Connect**, **Re-Test**, **Save**, **Disable** or **Enable**,
and **Remove**, according to provider and connection state. Offline mirrors also
expose **Preview** and **Sync Now**. Relevant edits require a successful Re-Test
before Save. When remote sharing needs acknowledgment, Save changes to
**Confirm Save** for the second click; editing the details clears that confirmation.

Blue actions indicate recommended next steps, grey actions are secondary, and
red actions disable or remove a connection. Blocked and running actions keep
their base color with reduced contrast; progress labels indicate work in flight.

New Offline mirrors are saved with background synchronization paused. Run Preview
and the confirmed initial Sync Now before Start enables their schedule.
**Sync interval in minutes** controls the first scheduled delay after Start and
the delay after each background sync finishes. **Sync Now** runs immediately.

**Unmount for Sleep**: Because a busy mount may delay sleep, settings can
unmount Online connections before sleep and restore them after wake.

**Directory preload** is optional and reads directory metadata (not file contents).
Enabled preloads run after mounting, stop at their configured time limit, and
are cancelled before unmount, repair, removal, or sleep cleanup.
The preload approach is optimized for each provider and connection type.
Preload is disabled by default and should be tested before leaving it enabled.
*Warning: Preloading can trigger provider rate limits or exhaust API quotas.*

**Google Drive:** mounts enable fast-list at creation. Preload refreshes the root
directory first, then recursively fills the directory cache within its time limit.

**SMB:** connections may individually override the global preload setting.

**SFTP:** skips server `/proc`, `/sys`, and `/dev` trees and does not
follow symbolic links. Each connection may individually override the global preload setting.

## Authentication

The applet does not store provider credentials. They stay with `rclone`,
`jstaf/onedriver`, or the operating system. Retired legacy credentials remain untouched.

__SFTP__ supports a password, SSH key, or SSH agent and requires a known-hosts file.

__SharePoint__ can use a verified existing rclone document-library remote or the
editor's Microsoft sign-in setup.

__Google Drive__ and __Box__ remote setup open the browser for authorization.
Selecting an existing remote allows access testing without creating another one.

__Google Drive__ setup accepts the client ID and matching client secret from a
Google Cloud Desktop OAuth application managed through the
[Google Auth Platform](https://console.cloud.google.com/auth/overview).
Use your own client for independent API quota and reliable access. Leaving both
fields blank uses rclone's shared client for compatibility; do not rely on its
continued availability.

To configure a private __Google Drive OAuth__ client, follow
[Google Cloud authentication setup](docs/google-cloud-auth-setup.md).
This applet does not provide a Google Cloud project or client ID/secret for you.

In Modify mode, **Update Google OAuth** changes an existing Drive remote
and opens browser authorization to issue a new token for the selected Google
account. Remount active connections using that remote after updating it.
Your personal Google client can also authorize Google Workspace
accounts when the Workspace policy permits it.

## VPN Integration

The applet can associate a connection with a saved NetworkManager VPN profile.

The applet may start a VPN dependency and wait for readiness checks before
mounting or syncing. It disconnects only a VPN it activated, and only after no
active connection still requires it.

For a Cisco AnyConnect-compatible gateway, follow the OpenConnect setup and
authentication-agent instructions in [OpenConnect setup](docs/openconnect.md).

## Connection Removal

Press **Remove** once to request confirmation, then press **Confirm Remove**
again to remove the connection.

Removal deletes the applet-managed connection record and any matching
applet-owned systemd user units. Connection removal does **not** delete
provider credentials, cloud data, local mirror data, caches, recovery directories,
or original imported legacy service files.

Open **Settings → Remove Remote** to list rclone remotes and whether saved
connections use them. **In use** entries cannot be removed. Unused entries use
**Remove → Confirm Remove**. This changes rclone configuration, not only applet
configuration, and does not delete cloud files.

## Conflict Recovery and Limitations

The applet does not coordinate simultaneous document editing by multiple users.

Offline mirrors preserve both versions of same-file conflicts. Deletions
propagate bidirectionally after preview and confirmation policy has been
satisfied. Deleted and overwritten files are moved into recovery locations and
retained by applet policy for 30 days.

Google Docs, Sheets, Slides, and related browser-native Google document types
are excluded from rclone Offline mirrors.

In Online mounts, rclone presents Google-native files with export suffixes
such as `.docx`, `.xlsx` and `.pptx`. They can appear as zero bytes and open
empty in desktop applications; see [rclone's Google Docs limitations](https://rclone.org/drive/#limitations-of-google-docs).
Edit native originals in the browser. Native-document updates require explicit
[rclone import configuration](https://github.com/rclone/rclone/blob/v1.74.3/backend/drive/drive.go#L4286-L4295),
which the applet does not enable. This is not an enforced read-only guarantee.
An exported, separately uploaded Office file is distinct from its native original.

## Uninstallation

Package removal does not delete configuration and data in the user's home
directory. The configurations can be reused later.

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
flatpak uninstall --user --app --no-related io.github.uutzinger.cosmic-ext-applet-mounter
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

## Project Development

This applet was developed with agent-assisted programming.

See [Design.md](Design.md) for the architecture, host integration, mount/mirror
lifecycles and a grouped source-file inventory.

The developer describes applet functionality in [Applet Description.md](Applet%20Description.md).
The description is translated into [Requirements and Specifications.md](Requirements%20and%20Specifications.md).
The author reviews these documents before implementation.
The requirements drive [Task List.md](Task%20List.md), and its execution history
is documented in [Task List Completion Notes.md](Task%20List%20Completion%20Notes.md).
The author supervises and approves each task and its verification.
After a release a Task List and its Completion history is archived in docs folder.

## Contributing & Feature Requests

### Feature Requests

For a feature contribution, update the [description](Applet%20Description.md)
and use your AI agent to update [requirements](Requirements%20and%20Specifications.md) and append its
implementation and verification steps to the [task list](Task%20List.md).
Instruct your AI agent to implement and test each step in the task list. It will record the completion in
[Task List Completion Notes.md](Task%20List%20Completion%20Notes.md). You will need to approve each task individually. When your feature works submit a pull request. 

### Bug Reports

- Submit a report on GitHub.

## License

MIT, copyright Urs Utzinger and OpenAI Codex.
