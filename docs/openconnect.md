# OpenConnect

For an AnyConnect-compatible gateway, use NetworkManager's OpenConnect plugin.
COSMIC's network panel does not properly handle multi-field authentication, so
Cloud Mounter activates the VPN through NetworkManager with an on-demand
`nm-applet` authentication agent.

The setup script installs the required packages and creates the authentication
agent on the distributions where COSMIC is packaged: Debian, Ubuntu, Pop!_OS,
Fedora, Arch Linux, openSUSE Tumbleweed, and Gentoo.

## Install the on-demand service

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/setup-openconnect.sh | bash -s -- install
```

Or download and run it locally:

```sh
bash scripts/setup-openconnect.sh install
```

To remove only the on-demand agent service later:

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/setup-openconnect.sh | bash -s -- uninstall
```

Installing `openconnect` alone is not sufficient: NetworkManager also needs the
OpenConnect plugin and its graphical authentication dialog.

## Uninstall the on-demand service

Run the setup script with the `uninstall` command:

```sh
curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/setup-openconnect.sh | bash -s -- uninstall
```

Or do it manually by stopping the service, removing its unit, and reloading the
user manager:

```sh
systemctl --user stop cosmic-mounter-openconnect-agent.service
rm -f ~/.config/systemd/user/cosmic-mounter-openconnect-agent.service
systemctl --user daemon-reload
```

Removing the service does not remove the OpenConnect packages or saved VPN
profile. To remove those separately, first delete the VPN profile in
`nm-connection-editor`, then use your distribution's package manager. The
`uninstall` command prints an example for the detected distribution. Keep
`network-manager-applet` (or `network-manager-gnome`) if you still need
`nm-connection-editor` for other NetworkManager profiles.

## Create the VPN profile with NetworkManager's GUI

Do not use COSMIC's network panel to create an OpenConnect profile that requires
multi-field authentication. Open NetworkManager's connection editor instead:

```sh
nm-connection-editor
```

In the editor:

1. Select **+** to add a connection.
2. Choose **Cisco AnyConnect or OpenConnect**.
3. Give the profile a recognizable name and enter the gateway URL supplied by
   the VPN administrator.
4. Leave the user agent, CA certificate, proxy, and CSD options unchanged unless
   the VPN administrator provided values for them.
5. Save the profile. Do not put a password or second-factor response in the
   profile or in a command.

For example, a University of Arizona Engineering profile may use:

- Name: `UA Engineering OpenConnect`
- Protocol: **Cisco AnyConnect or OpenConnect**
- Gateway: `https://vpn.arizona.edu/Engineering`

Select the saved profile as the connection's VPN dependency in this applet.
When the applet activates it, the OpenConnect dialog should request the fields
defined by the gateway—for example, username, password, and `NetID +Method`—and
then start the second-factor authentication. The exact fields and gateway URL
are organization-specific.
