# Dependency Installation

This document covers installing and checking external dependencies.

## Required Versions

| Tool | Used For | Required/Tested Version |
|---|---|---|
| `rclone` | Google Drive, Box, and SMB Online mount and Offline mirror | 1.74.3 or newer |
| `onedriver` | OneDrive Online mount | 0.15.0 or newer |
| `onedrive` | OneDrive Offline mirror | 2.5.10 or newer |
| `fusermount3` / FUSE 3 | Online mounts | installed |
| `nmcli` / NetworkManager | network and VPN readiness | installed |
| Cisco Secure Client | optional Cisco VPN dependency | 5.1.10 tested |
| `fuser` from `psmisc` | optional busy-mount diagnostics | recommended |

If you build from source you can check dependencies with:

```sh
cargo run --example dependency_inventory
```

## rclone

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

## jstaf/onedriver

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

## abraunegg/onedrive

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

Before installing, confirm `apt-cache policy onedrive` selects the OBS package
and version 2.5.10 or newer.

References:

- <https://github.com/abraunegg/onedrive/blob/master/docs/ubuntu-package-install.md>
- <https://github.com/abraunegg/onedrive/blob/master/docs/usage.md>
- <https://github.com/abraunegg/onedrive/blob/master/docs/advanced-usage.md>

## Cisco Secure Client

Cisco Secure Client support requires the CLI/GUI binaries and the VPN agent
service. On the tested machine the service was named `vpnagentd.service`.

Cisco Secure Client installation instructions are usually provided by the
organization providing VPN services, such as your employer.

Check:

```sh
systemctl status vpnagentd.service
/opt/cisco/secureclient/bin/vpn stats
```

If the CLI cannot contact the VPN service:

```sh
sudo systemctl start vpnagentd.service
sudo systemctl enable vpnagentd.service
```

This starts the background agent, not the VPN tunnel itself.
