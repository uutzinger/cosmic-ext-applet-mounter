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
