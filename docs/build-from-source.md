## Build from Source

Run the commands below from the repository root.

Requirements:

- Linux with the COSMIC™ desktop
- Rust 1.95.0 or later through rustup
- `just`
- Python 3 for tooling; Python 3.10 also needs `python3-tomli` for release checks
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

For concise output with full private logs, run a fixed recipe through the wrapper:

```sh
bash scripts/dev.sh check
bash scripts/dev.sh test
```

Use `--help` for the task list. It preserves just's exit status and prints a
result with log/summary paths under `target/task-runs/`; it does not install or
reload the applet. `build` and `build-release` explicitly compile native source.
Debian release packages use the hosted workflow, not a required local build.

Inspect an existing local Debian candidate without rebuilding or installing:

```sh
python3 scripts/inspect-artifacts.py debian /absolute/path/to/package.deb --version 0.5.3 --architecture amd64
```

Use `debian-matrix /path/to/download-root --version 0.5.3` for six hosted
targets, or `native /path/to/binary` for byte identity only. The tool writes a
private receipt under `target/task-runs/`; this is not installed acceptance or
proof of build provenance. See the [script catalog](../scripts/README.md).

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
and explicitly starts one Debian matrix build; tag pushes do not auto-build.
Use `--run-id <id>` to reuse a successful build at the exact same commit instead.
It downloads artifacts without publishing. Finalize metadata before that build;
acceptance receipts alone do not require recompiling an unchanged source.
For interrupted preparation, use `--resume --run-id <id>` from the original
source; an existing matching tag is not recreated or pushed again. Without a
selected run, resumption refuses to start another build.

After package acceptance and separate publication approval, dispatch
**Promote verified release artifacts** (`publish-release.yml`) on the accepted
tag with `build_run_id=<id>` and `accepted=true`. This verifies source/run identity,
six unexpired artifacts, checksums and package identities, then publishes those
existing packages without building again. Inspection-only local equivalent:
`python3 scripts/promote_release.py v<version> <run-id>` from a clean checkout
of that tag. Adding `--publish --accepted` requires publication approval.
Expired/missing artifacts or changed source require a new accepted build, never
a silent substitution. Do not run tag/publish commands without approval.
Release 0.5.3 targets native source/Debian delivery; final Flatpak is a 0.5.4 gate.

Generated packages belong in `target/artifacts/`: versioned release sets in
`v<version>/`, distinct local builds in `candidates/`, and workflow downloads
in run-specific folders. `scripts/run_github_linux_build.sh` defaults to
`target/artifacts/ci-<run-id>/`; `--download-dir` overrides it.

Build/release tools save private `task.log` and `summary.json` under
`target/task-runs/` (or a new `--log-dir`). Return the concise result/summary path
for operator handoffs. Hosted retrieval resumes via `--run-id <id>` or
`--resume-report <summary.json>` without another dispatch; existing downloads
are reused only after receipt/checksum verification. Full output is not streamed.
See [script catalog](../scripts/README.md#logs-summaries-and-operator-handoff).
Hosted promotion retains summary/checksum receipts as workflow artifacts; private
raw task logs are not uploaded automatically.

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
