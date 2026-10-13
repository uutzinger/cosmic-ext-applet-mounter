# Dependencies

Native execution discovers whether required executables are available. A
Flatpak confirms that its host-execution route is available, then reports a
typed failure if the requested host operation cannot find or use its tool. The
applet does not run startup version/help probes or install or upgrade
dependencies. The installer and packaging requirements below enforce the
supported minimum versions.

You likely will need to **update rclone**. If you want to connect to OneDrive storage, you
need to install onedriver and/or onedrive. All other components should be
preinstalled by the operating system.

The automated installer supports Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux,
openSUSE Tumbleweed, and Gentoo:

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/install-dependencies.sh | bash
```

Or run it locally after cloning the repository:

```sh
bash scripts/install-dependencies.sh --all
```

The script installs rclone using the official installer by default. If you prefer
to try rclone's package-aware selfupdate first, add `--rclone-selfupdate`:

```sh
bash scripts/install-dependencies.sh --all --rclone-selfupdate
```

On Arch Linux, an AUR helper (`yay` or `paru`) is required for the `onedriver`
and `onedrive` packages. On Gentoo and openSUSE, those packages must be built or
installed manually; the script will report the upstream instructions.

Manual instructions are retained below for reference and for other distributions.

## Required Versions

| Tool | Used For | Required/Tested Version |
|---|---|---|
| [`rclone`](https://rclone.org/) | Google Drive, Box, SMB, SFTP, and SharePoint Online mount and Offline mirror | 1.74.3 or newer |
| [`onedriver`](https://github.com/jstaf/onedriver) | OneDrive Online mount | 0.15.0 or newer |
| [`onedrive`](https://github.com/abraunegg/onedrive) | OneDrive Offline mirror | 2.5.10 or newer |
| [`fusermount3`](https://github.com/libfuse/libfuse) / FUSE 3 | Online mounts | installed |
| [NetworkManager](https://networkmanager.dev/) | network and VPN readiness over D-Bus | installed |
| systemd user manager and logind | service/timer control and optional sleep cleanup over D-Bus | host services available |
| OpenConnect NetworkManager plugin | Cisco-compatible VPN authentication | see [OpenConnect setup](openconnect.md) |
| Python 3 / PyGObject, GTK 3, WebKitGTK 4.1 | Optional OneDrive Offline Mirror sign-in window | Debian packages below |

### OneDrive sign-in helper

The supported OneDrive Offline Mirror GUI helper needs these runtime packages
on Debian/Ubuntu/Pop!_OS (not compiler/development packages):

```sh
sudo apt install python3 python3-gi gir1.2-gtk-3.0 gir1.2-webkit2-4.1
```

The Debian package recommends them, so normal APT installation includes them.
Source installations need the command above if using this helper. If installing
with `--no-install-recommends`, install them explicitly or use the editor's manual
browser/response-URL handoff instead. They are not required for ordinary mounts,
already-authorized synchronization, or OpenConnect VPN authentication. Flatpak
must supply these helper libraries inside its runtime/base; host packages alone
do not make them available in the sandbox.

The installer includes this runtime with `--all` or `--onedrive`, even when
onedrive is already current. To install/check only the GUI dependencies:

```sh
bash scripts/install-dependencies.sh --onedrive-auth-helper
```

Use `--skip-onedrive-auth-helper` with other selections to keep manual URL
handoff instead. The helper path checks imports without opening a GUI or
authenticating. Debian/Ubuntu/Pop!_OS, Fedora, and Arch package mappings are
automated; openSUSE/Gentoo report explicit manual runtime guidance when missing.

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
cargo run --example dependency_inventory
```

## rclone (Google Drive, Box, SMB)

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

## abraunegg/onedrive (OneDrive Mirror)

Ubuntu 24.04 repositories may provide an obsolete 2.4.x package. Use an
upstream-supported package source that provides version 2.5.10 or newer.

For Pop!_OS/Ubuntu 24.04 using the Open Build Service repository:

```sh
sudo apt install wget gnupg

wget -qO - \
  https://download.opensuse.org/repositories/home:/npreining:/debian-ubuntu-onedrive/xUbuntu_24.04/Release.key \
  | gpg --dearmor \
  | sudo tee /usr/share/keyrings/obs-onedrive.gpg >/dev/null

echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/obs-onedrive.gpg] https://download.opensuse.org/repositories/home:/npreining:/debian-ubuntu-onedrive/xUbuntu_24.04/ ./" \
  | sudo tee /etc/apt/sources.list.d/onedrive.list

sudo apt update
apt-cache policy onedrive
sudo apt install --no-install-recommends --no-install-suggests onedrive
onedrive --version
```

For an operating system other than Ubuntu 24.04, check the onedrive installation instructions in the reference below.

Before installing, confirm `apt-cache policy onedrive` selects the OBS package
and version 2.5.10 or newer.

References:

- <https://github.com/abraunegg/onedrive/blob/master/docs/ubuntu-package-install.md>
- <https://github.com/abraunegg/onedrive/blob/master/docs/usage.md>
- <https://github.com/abraunegg/onedrive/blob/master/docs/advanced-usage.md>

## Cisco-compatible VPN through NetworkManager OpenConnect

Cloud Mounter supports Cisco AnyConnect-compatible gateways only through a
saved NetworkManager OpenConnect profile. It does not inspect, launch, or stop
the proprietary Cisco Secure Client. See [OpenConnect setup](openconnect.md) for
the required packages, authentication-agent service, and profile setup.
