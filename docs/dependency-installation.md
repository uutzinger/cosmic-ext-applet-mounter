# Dependencies

Native execution discovers whether required executables are available. A
Flatpak confirms that its host-execution route is available, then reports a
typed failure if the requested host operation cannot find or use its tool. The
applet does not run startup version/help probes or install or upgrade
dependencies. The installer and packaging requirements below enforce the
supported minimum versions.

For 0.5.5 development, **rclone 1.74.3 or newer** handles OneDrive Offline
mirrors and `onedriver` handles OneDrive Online. The abraunegg `onedrive` engine
is retired; its package is neither installed nor uninstalled by this change.
Existing profiles and recovery data are preserved without conversion.
Installed upgrade acceptance remains pending.

The automated installer supports Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux,
openSUSE Tumbleweed, and Gentoo:

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/install-dependencies.sh | bash
```

Or run it locally after cloning the repository:

```sh
bash scripts/install-dependencies.sh --all
```

To install only the engine for new OneDrive mirrors, select `--rclone`:

```sh
bash scripts/install-dependencies.sh --rclone
```

No options and `--all` install base dependencies, rclone and onedriver.
Use `--onedriver` for OneDrive Online. The retired `--onedrive` option is rejected
before privileged work. These commands install/update packages; the applet never
installs them itself.

Optionally add `--log-dir /tmp/dependency-setup-report` (a new directory with
an existing parent) to retain private logs and a JSON result. Prompts/output
remain visible. Logs can contain sensitive diagnostics; do not publish them.
Failure may leave partial changes—inspect before retrying.

The script installs rclone using the official installer by default. If you prefer
to try rclone's package-aware selfupdate first, add `--rclone-selfupdate`:

```sh
bash scripts/install-dependencies.sh --all --rclone-selfupdate
```

On Arch Linux, an AUR helper (`yay` or `paru`) is required for the `onedriver`
package. On Gentoo and openSUSE, those packages must be built or
installed manually; the script will report the upstream instructions.

Manual instructions are retained below for reference and for other distributions.

## Required Versions

| Tool | Used For | Required/Tested Version |
|---|---|---|
| [`rclone`](https://rclone.org/) | Google Drive, Box, SMB, SFTP and SharePoint mounts/mirrors; new OneDrive Offline mirrors | 1.74.3 or newer |
| [`onedriver`](https://github.com/jstaf/onedriver) | OneDrive Online mount | 0.15.0 or newer |
| [`fusermount3`](https://github.com/libfuse/libfuse) / FUSE 3 | Online mounts | installed |
| [NetworkManager](https://networkmanager.dev/) | network and VPN readiness over D-Bus | installed |
| systemd user manager and logind | service/timer control and optional sleep cleanup over D-Bus | host services available |
| OpenConnect NetworkManager plugin | Cisco-compatible VPN authentication | see [OpenConnect setup](openconnect.md) |

The implemented 0.5.4 development rclone OneDrive Offline mirror uses the same **1.74.3 minimum**;
it does not require another upgrade beyond this existing requirement. Its generic
bisync flags are present in the pinned 1.74.3 source; personal/business live
prototype tests used 1.75.0. This is source compatibility evidence for 1.74.3,
not a live test of that release. See the
[version and flag assessment](engineering/onedrive-rclone-offline-contracts.md#rclone-version-and-flag-assessment).

### Retired legacy OneDrive profiles

Version 0.5.5 removes legacy authentication, preview, sync and scheduling.
Existing profiles remain readable and removable. Preserve unsynchronized files;
create a replacement rclone connection with separate mirror/recovery folders and
new authentication. Do not copy legacy databases or credentials into rclone.
The runtime owner retires only verified app-owned monitor units through systemd;
failures are reported without restarting the engine. No user files or packages
are removed. See [draft release notes](releases/v0.5.5.md).

### Dependency diagnostics

The inventory lists supported executables. Missing-tool warnings follow saved
connections: new OneDrive mirrors use rclone, Online OneDrive uses onedriver,
and retired legacy profiles no longer request an engine installation. FUSE is needed for Online mounts;
NetworkManager and the systemd user manager remain common host prerequisites.
Saved disabled profiles still retain their engine requirements. Version/capability
probes are not run at applet startup; actual operations report typed failures.

### Host integration prerequisites

Flatpak does not bundle or upgrade these host engines. OpenConnect additionally
needs the host authentication-agent service described in its setup guide; other
NetworkManager VPN types do not require `nm-applet`. Native, Debian, and Flatpak
use the same NetworkManager/systemd D-Bus adapters, not runtime `nmcli` or
`systemctl` commands. Installer and manual setup commands are separate from
applet runtime execution.

Documents portal v5+ enables sandboxed host-path lookup. Supported portal-v4
hosts instead need the host `flatpak` command for the gated `document-info`
fallback; the applet does not upgrade the portal. See the
[packaging permission rationale](../packaging/flatpak/README.md#permission-rationale).

If you build from source you can check dependencies with:

```sh
cargo run --locked --example dependency_inventory
```

## rclone (mounts and Offline mirrors)

Check the installed version and configuration location:

```sh
command -v rclone
rclone version
rclone config file
```

If you need to upgrade, first stop active rclone mounts or transfers and back up the
configuration:

```sh
cp --preserve=all ~/.config/rclone/rclone.conf \
  ~/.config/rclone/rclone.conf.backup
pgrep -af '(^|/)rclone( |$)'
findmnt -rn -t fuse.rclone -o TARGET,SOURCE,FSTYPE,OPTIONS
```

Preferred update is the package-aware update:

```sh
rclone selfupdate --stable --check
sudo rclone selfupdate --stable --package deb
hash -r
rclone version
```

If the packaged build cannot self-update, use the official installer:

```sh
curl --fail --show-error --silent \
  https://rclone.org/install.sh \
  --output /tmp/rclone-install.sh
less /tmp/rclone-install.sh
sudo bash /tmp/rclone-install.sh
hash -r
rclone version
```

Avoid the Snap package for this applet because strict confinement does not
support `rclone mount`.

References:

- <https://rclone.org/install/>
- <https://rclone.org/commands/rclone_selfupdate/>
- <https://rclone.org/downloads/>

## jstaf/onedriver (OneDrive Online)

Install `onedriver` from the
[OpenSUSE Build Service](https://software.opensuse.org/download.html?project=home%3Ajstaf&package=onedriver)
or a trusted package source that provides version 0.15.0 or newer.

Check:

```sh
onedriver --version
onedriver --help
pgrep -af '(^|/)onedriver( |$)'
findmnt -rn -t fuse.onedriver -o TARGET,SOURCE,FSTYPE,OPTIONS
```

Reference:

- <https://github.com/jstaf/onedriver>

## abraunegg/onedrive (retired)

The 0.5.5 installer no longer installs this engine. The published
[0.5.4 release notes](releases/v0.5.4.md) retain the transition-release history.
Python remains available for maintained build and developer tools.

## Cisco-compatible VPN through NetworkManager OpenConnect

Cloud Mounter supports Cisco AnyConnect-compatible gateways only through a
saved NetworkManager OpenConnect profile. It does not inspect, launch, or stop
the proprietary Cisco Secure Client. See [OpenConnect setup](openconnect.md) for
the required packages, authentication-agent service, and profile setup.
