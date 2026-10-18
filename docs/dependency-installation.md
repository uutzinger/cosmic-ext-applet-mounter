# Dependencies

Native execution discovers whether required executables are available. A
Flatpak confirms that its host-execution route is available, then reports a
typed failure if the requested host operation cannot find or use its tool. The
applet does not run startup version/help probes or install or upgrade
dependencies. The installer and packaging requirements below enforce the
supported minimum versions.

For the 0.5.4 development source, **rclone 1.74.3 or newer** handles new
OneDrive Offline mirrors. OneDrive Online mounts still need `onedriver`;
`onedrive` is needed only for retained legacy Offline profiles; sign-in uses
the browser/manual response-URL handoff, without a WebKit helper. Installed native/Flatpak acceptance of the new integration is
pending; an older installed applet keeps its existing engine behavior.

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

No options and `--all` retain all engines for compatibility; they do not install
the retired WebKit helper runtime.
Use `--onedriver` for OneDrive Online or `--onedrive` for existing legacy mirrors.
These commands install/update packages; the applet never installs them itself.

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
and `onedrive` packages. On Gentoo and openSUSE, those packages must be built or
installed manually; the script will report the upstream instructions.

Manual instructions are retained below for reference and for other distributions.

## Required Versions

| Tool | Used For | Required/Tested Version |
|---|---|---|
| [`rclone`](https://rclone.org/) | Google Drive, Box, SMB, SFTP and SharePoint mounts/mirrors; new OneDrive Offline mirrors | 1.74.3 or newer |
| [`onedriver`](https://github.com/jstaf/onedriver) | OneDrive Online mount | 0.15.0 or newer |
| [`onedrive`](https://github.com/abraunegg/onedrive) | Legacy OneDrive Offline mirror only | 2.5.10 or newer |
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

### Legacy OneDrive reauthentication

Existing legacy mirrors keep their onedrive engine and stored authentication.
For reauthentication, use the editor's browser/manual response-URL handoff:
prepare the handoff, open sign-in through the desktop portal, then paste and
submit the final Microsoft native-client redirect URL in the editor. Treat the
URL as a credential; never paste it in logs or support messages.

Version 0.5.4 retires the applet's WebKit helper and its PyGObject/GTK/WebKit-only
recommendations and installer options. It does not uninstall libraries from the
host; other applications may need them. Python remains a build/developer-tool
prerequisite where those tools use it. New rclone mirrors use their own supported
Microsoft browser flow and do not consume legacy authentication files.

### Dependency diagnostics

The inventory lists supported executables. Missing-tool warnings follow saved
connections: new OneDrive mirrors use rclone, Online OneDrive uses onedriver,
and legacy Offline OneDrive uses onedrive. FUSE is needed for Online mounts;
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

## abraunegg/onedrive (legacy OneDrive Mirror)

> **Transition release:** the 0.5.4 source creates OneDrive Offline mirrors with `rclone bisync`. Retain `onedrive` for existing legacy profiles for one release; engine removal is a later phase. The WebKit helper is retired in 0.5.4. There is no automatic conversion. See the [OneDrive Offline contracts](engineering/onedrive-rclone-offline-contracts.md).

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
