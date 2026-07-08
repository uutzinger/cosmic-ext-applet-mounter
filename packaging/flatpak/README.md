# Flatpak Packaging Prototypes

This directory contains Flatpak packaging experiments. These files are not the
final COSMIC repository submission.

## Host Runner Probe

`io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json` copies the
locally built `flatpak_host_runner_probe` example into a minimal Flatpak. It is
used only to verify whether `flatpak-spawn --host` can execute the host tools
the applet needs.

Build the probe binary first:

```sh
cargo build --release --example flatpak_host_runner_probe
```

Build and install the local Flatpak:

```sh
flatpak-builder --force-clean --user --install \
  target/flatpak-host-runner-probe \
  packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json
```

Run the probe:

```sh
flatpak run io.github.uutzinger.cosmic-ext-applet-mounter-probe
```

Expected checks:

- `rclone version`
- `nmcli general status`
- `systemctl --user --version`
- `fusermount3 --version`

This prototype grants only `--talk-name=org.freedesktop.Flatpak`, which is
required for `flatpak-spawn --host`. Add broader filesystem or session
permissions only after a specific failing feature proves they are required.
