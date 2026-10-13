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
acceptance of the advertised package modes and finalized notes/AppStream, separately
authorize a manual **Build Debian packages** dispatch on the accepted tag with
`publish_after_acceptance=true`. This rebuilds all six packages before publishing.
Do not run tag/publish commands without the corresponding approval.
Release 0.5.3 targets native source/Debian delivery; final Flatpak is a 0.5.4 gate.
For 0.5.3, the user approved direct publication of the already verified tagged
artifacts rather than rebuilding them. Future promotion tooling should preserve
that single-build identity and the separate approval gate.

Generated packages belong in `target/artifacts/`: versioned release sets in
`v<version>/`, distinct local builds in `candidates/`, and workflow downloads
in run-specific folders. `scripts/run_github_linux_build.sh` defaults to
`target/artifacts/ci-<run-id>/`; `--download-dir` overrides it.

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
