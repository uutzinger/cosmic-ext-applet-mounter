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

## GUI Prototype

`io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json` copies the
locally built applet binary, helper, desktop file, AppStream metadata, and icon
into a Flatpak so the real GUI can be smoke-tested before the final
reproducible manifest exists.

Build the applet first:

```sh
cargo build --release
```

Build and install the GUI prototype:

```sh
flatpak-builder --force-clean --user --install \
  target/flatpak-gui-prototype \
  packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json
```

Run the settings window:

```sh
flatpak run io.github.uutzinger.cosmic-ext-applet-mounter --settings
```

This prototype currently grants Wayland, fallback X11, IPC, DRI,
`org.freedesktop.Flatpak` access, and narrow filesystem grants for:

- `xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter`
- `xdg-config/cosmic/com.system76.CosmicTheme.Mode` (read-only)
- `xdg-config/cosmic/com.system76.CosmicTheme.Light` (read-only)
- `xdg-config/cosmic/com.system76.CosmicTheme.Dark` (read-only)
- `xdg-config/cosmic-ext-applet-mounter`
- `xdg-config/systemd/user`
- `xdg-cache/cosmic-ext-applet-mounter`
- `~/.local/state/cosmic-ext-applet-mounter`

These permissions are for GUI smoke testing; they still need to be minimized
against the final applet behavior.

The GUI prototype now includes a first host-visible state bridge for the applet
configuration document. When the applet detects Flatpak mode, it reads and
writes the same native-visible COSMIC config document used by source and Debian
installs:

```text
~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document
```

This avoids forcing users to recreate saved connections when switching package
formats. The prototype also maps app-owned durable config/cache/state roots to
host-visible `~/.config/cosmic-ext-applet-mounter`,
`~/.cache/cosmic-ext-applet-mounter`, and
`~/.local/state/cosmic-ext-applet-mounter` when running in Flatpak mode.
Generated user systemd unit writes, app-owned onedriver/onedrive metadata, and
rclone remote/config paths have been live-tested through the GUI prototype, but
the prototype manifest is still not the final reproducible COSMIC repository
submission manifest.

Flatpak mode also applies the host COSMIC theme explicitly for standalone
settings windows, because toolkit/system theme lookup can otherwise resolve
inside the Flatpak sandbox and choose the wrong light/dark mode.

## Final Manifest Scaffold

`io.github.uutzinger.cosmic-ext-applet-mounter.json` is the project-owned
manifest scaffold intended to be copied into the COSMIC Flatpak repository as:

```text
app/io.github.uutzinger.cosmic-ext-applet-mounter/io.github.uutzinger.cosmic-ext-applet-mounter.json
```

It is not fully buildable until `REPLACE_WITH_FLATPAK_READY_COMMIT` is replaced
with the tagged or pinned release commit and `cargo-sources.json` is generated.

## Permission Rationale

The tested architecture does not require `--filesystem=host`. Host commands are
executed with `flatpak-spawn --host`, and generated systemd units run host
tools such as `rclone`, `onedriver`, `onedrive`, `nmcli`, and `fusermount3`
outside the sandbox. The Flatpak itself needs only enough filesystem access to
share applet state and app-owned engine metadata with the host user session.

Current final-manifest `finish-args`:

- `--socket=wayland`: required for COSMIC/Wayland UI.
- `--socket=fallback-x11`: retained as the tested fallback display path.
- `--share=ipc`: paired with fallback X11 and common GUI toolkit behavior.
- `--device=dri`: retained from the GUI prototype so libcosmic rendering and
  acceleration match the tested settings-window behavior.
- `--talk-name=org.freedesktop.Flatpak`: required for
  `flatpak-spawn --host`.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Mode:ro`: read the
  host COSMIC light/dark preference for standalone settings windows.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Light:ro`: read the
  host COSMIC light theme palette.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Dark:ro`: read the
  host COSMIC dark theme palette.
- `--filesystem=xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter:create`:
  share the applet's native-visible COSMIC configuration document with source
  and Debian installations.
- `--filesystem=xdg-config/cosmic-ext-applet-mounter:create`: store app-owned
  engine configuration such as isolated onedriver and abraunegg/onedrive
  metadata in host-visible paths used by host services.
- `--filesystem=xdg-config/systemd/user:create`: write generated user systemd
  service/timer files where the host user manager can load them.
- `--filesystem=xdg-cache/cosmic-ext-applet-mounter:create`: store app-owned
  caches used by host-run engines such as onedriver/rclone.
- `--filesystem=~/.local/state/cosmic-ext-applet-mounter:create`: store
  app-owned runtime state such as rclone bisync work files.

The applet still uses the desktop folder portal for user-selected mountpoints
and mirror directories. Document-portal paths are resolved back to host-visible
paths before saving so host systemd services can use them.
