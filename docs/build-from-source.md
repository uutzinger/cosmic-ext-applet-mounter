## Build from Source

Run the commands below from the repository root.

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

```sh
cargo run --example dependency_inventory
```
checks dependencies.

```sh
just install-user
```

installs the development build under `~/.local` and
updates desktop metadata and icons for the current user. After updating,
close any Cloud Mounter settings/editor windows and restart the COSMIC panel
without logging out:

```sh
killall cosmic-panel
```
restarts the panel and loads the updated applet.

Release preparation: `scripts/build_release.sh` creates/pushes the version tag
and downloads Debian build artifacts only; it does not publish. After final
native/Flatpak acceptance and finalized notes/AppStream metadata, separately
authorize a manual **Build Debian packages** dispatch on the accepted tag with
`publish_after_acceptance=true`. This rebuilds all six packages before publishing.
Do not run tag/publish commands without the corresponding approval.

`just stage` installs into `target/stage/usr` and does not modify the host
system. `just metadata-check` reports known COSMIC-specific freedesktop validation
warnings without failing. Use `just metadata-check-strict` for the raw result
and `just metadata-check-net` to check published URLs. `just deb` builds an
unsigned Debian package in the parent directory.

## Flatpak Packaging and Publication

The Flatpak has been tested locally but is not yet published through a public
remote. The manifest, build steps, and permission rationale are in the
[Flatpak packaging guide](../packaging/flatpak/README.md). It runs storage tools on
the host, so those dependencies must be installed there.

Native and Flatpak installations share connection configuration and generated
services. Stop one before starting the other to avoid competing operations.
