# Script catalog

Run maintained scripts from the repository root unless their help says otherwise.
This directory separates project setup, installed runtime helpers, and developer/
release tooling by purpose; filenames remain stable for callers and user guides.
No script is authorized to install, alter services, tag or publish merely because
it is listed here. Inspect its help and obtain the applicable approval first.

## Project setup and runtime

| Script | Purpose | State changes / guidance |
|---|---|---|
| [install-dependencies.sh](install-dependencies.sh) | Install selected host storage/base/optional OneDrive GUI dependencies. | Package/repository/network changes, potentially privileged. No options means `--all`, **not** inspection. See [dependencies](../docs/dependency-installation.md). |
| [setup-openconnect.sh](setup-openconnect.sh) | Install OpenConnect packages and the on-demand authentication-agent service; uninstall that service. | Packages, user service and icon setting change. Does not create a VPN profile; uninstall retains packages/profile. See [OpenConnect guide](../docs/openconnect.md). |
| [cosmic-ext-applet-mounter-onedrive-auth-helper](cosmic-ext-applet-mounter-onedrive-auth-helper) | Installed native graphical OneDrive redirect capture. | Opens GUI, reads auth URL file and writes private response file. Runtime payload used by packaging/app, not an obsolete developer experiment. Never paste redirect URLs or credentials into logs. |

## Optional setup logs

Both setup scripts accept `--log-dir NEW_DIRECTORY`, whose parent must exist.
For example, `bash scripts/setup-openconnect.sh install --log-dir /tmp/openconnect-setup-report`
or `bash scripts/install-dependencies.sh --base --log-dir /tmp/dependency-setup-report`.
These are installation commands, not checks; use only when installation is wanted.
The directory must not exist. Output and prompts remain visible; a private
`task.log` and atomic `summary.json` retain the result and actual exit code.
Default invocation is unchanged; standalone downloads/stdin use still work.
Raw logs may contain sensitive diagnostics. Failed setup can leave partial
changes; inspect them before retrying. Help does not create logs.

The small reporting block is embedded identically in both scripts (parity-tested)
so standalone users do not need Python or checkout-relative support files.

## Development and release tools

| Script | Purpose | Current boundary |
|---|---|---|
| [check-ui-localization.sh](check-ui-localization.sh) | Localization gate used by `just localization-check`. | Calls translation checker and helper localization tests; does not build. |
| [dev.sh](dev.sh) | Run a fixed just check/native-build recipe with private logs and concise phase/result output. | `bash scripts/dev.sh check`; `gate` runs fmt-check, lint and test; supports `--log-dir` and bounded `--timeout-seconds`. No arbitrary arguments, install, cleanup, GUI launch or release tasks. |
| [build-flatpak.sh](build-flatpak.sh) | Explicit-manifest candidate build/export, or verified reuse with --reuse-receipt. | Clean source SHA and pinned inputs required. Builds need a new output; reuse needs a schema-2 receipt/successful summary and passes hashes, fsck and metadata checks without builder/runtime calls. No installation, launch or submission. |
| [check-ui-translations.py](check-ui-translations.py) | Catalog/reference and selected hardcoded UI-string checks. | Source checks, not full Fluent compilation or rendered UI review. |
| [check-release-publication.sh](check-release-publication.sh) | Version/metadata/finalized-note preflight (wrapper for `lib/publication_check.py`). | Read-only; does not prove package acceptance or grant approval. |
| [archive-release.sh](archive-release.sh) | After a release, move completed task-list and completion-note history to `docs/archive/` and carry open items forward. | Edits the two Markdown files and `docs/archive/` only; no commit. Needs `--tasks-from` / `--notes-from` heading prefixes; `--dry-run` reports counts. Version may carry a pre-release label (`0.5.4-pre`). Refuses existing archives and uncommitted edits to the two files. |
| [onedrive-mirror-guard-check.sh](onedrive-mirror-guard-check.sh) | Live OneDrive compatibility investigation. | **Changes cloud data:** only fresh `CloudMounter-GuardTest-<token>` folders. `--round-trip` adds immutable originals, independent downloads, measured sync transfers and schema-2 partial evidence; cleanup verifies marker content. `--settle-seconds` is bounded 0..60 (default 10); `--web-edit` pauses for an Office edit. `--plan` only checks configured backend presence. Legacy mode retains schema 1. |
| [run_github_linux_build.sh](run_github_linux_build.sh) | Dispatch or retrieve a hosted Debian build. | Unique dispatch correlation, structured polling and verified downloads/cache. `--run-id` or `--resume-report` does not dispatch. No installation/publication. |
| [build_release.sh](build_release.sh) | Tag/push and start one hosted matrix or reuse an exact-commit run. | Tag/push approval required. `--dry-run` is non-mutating; `--resume --run-id` reuses the original tag/run without implicit rebuilding. Does not publish. |
| [promote_release.py](promote_release.py) | Validate/download an existing accepted build and optionally publish its packages. | Inspection default; publication requires `--publish --accepted` and approval. Persistent inspection receipt, no rebuild. |
| [inspect-artifacts.py](inspect-artifacts.py) | Offline Debian/native inspection and Flatpak repository/bundle ref and complete metadata comparison. | Invoke with `python3`; Flatpak modes require expected metadata and identity. Bundle import uses a private temporary repo, never an installation. No app execution/network; metadata checks do not establish payload/signature/provenance or live acceptance. |

Use [justfile](../justfile) recipes for existing native build/run/stage/install and
verification tasks; do not create duplicate standalone scripts without a gap.
Debian packaging also uses [debian/rules](../debian/rules). Hosted definitions are
[build-deb.yml](../.github/workflows/build-deb.yml) and
[publish-release.yml](../.github/workflows/publish-release.yml). Flatpak commands
and manifests remain in the [packaging guide](../packaging/flatpak/README.md).

## Logs, summaries and operator handoff

The three release/build entry points accept `--log-dir <new-directory>`; by
default each creates a private unique directory under `target/task-runs/`.
Verbose output stays in `task.log`. Phase/status changes and one final compact
JSON summary identify `summary_path`, `log_path`, real `exit_code`, source/run
and any `artifact_receipt_path`. Return the summary path/result, not full logs.
Failure excerpts must be bounded and redacted; they are not printed automatically.
Choose custom log directories outside the source tree or under ignored output;
release clean-worktree checks must not be bypassed for untracked log directories.

Hosted monitoring defaults to 60-second polls; `--poll-seconds`,
`--timeout-seconds` and `--discovery-timeout` are bounded controls. Cancellation
stops local child processes, not remote CI. Resume the same run using its ID or
the original `summary.json` (`--resume-report`); ambiguous dispatches never retry
automatically. Existing artifact directories need a matching verified receipt;
legacy/unrelated directories are not overwritten or silently adopted.

## Internal modules and directory layout

| Location | Role / callers | Usage boundary |
|---|---|---|
| [lib/task_output.py](lib/task_output.py) | Shared `TaskReport`, atomic JSON receipts, private logging, exit-status preservation and local child cancellation/cleanup. Used by the task driver and promotion tool. | Importable support module, not an operator command. Does not cancel remote CI. |
| [lib/release_tasks.py](lib/release_tasks.py) | Fixed hosted-build and release-preparation implementations behind `run_github_linux_build.sh` and `build_release.sh`; reuses verification functions from `promote_release.py`. | Internal task driver, not an additional public entry point. It can dispatch builds or create/push tags through the authorized wrappers. |
| [lib/project.py](lib/project.py) | Project constants (package name, hosted workflow, Debian artifact matrix and count) shared by the release tooling. | Edit this file first when reusing the scripts in another project; `tests/project_constants_test.py` checks it against Cargo.toml and the workflow. |
| [lib/publication_check.py](lib/publication_check.py) | Version, AppStream and release-note preflight using `tomllib`, behind `check-release-publication.sh`. | Read-only; exercised by `tests/release_publication_test.py`. |
| [lib/archive_tasks.py](lib/archive_tasks.py) | Implementation behind `archive-release.sh`: split at a heading, repair links, extract outermost open checkbox items. | Exercised by `tests/archive_release_test.py` on a synthetic repository. |
| [lib/mirror_guard_check.py](lib/mirror_guard_check.py) | Implementation behind `onedrive-mirror-guard-check.sh`: generic versus compatibility bisync profiles compared by stability of a no-edit second sync. | Mirrors the settings in `src/sync.rs`; exercised by `tests/mirror_guard_check_test.py` with a fake rclone only. |
| [lib/mirror_guard_roundtrip.py](lib/mirror_guard_roundtrip.py) | Explicit `--round-trip` implementation called by mirror_guard_check.py. | Schema-2 byte/metadata comparisons, structured transfer counters, opaque `rclone version` text provenance, bounded observations and marker-content cleanup; fixtures and optional real-rclone local-only smoke in `tests/mirror_guard_roundtrip_test.py`. |
| [lib/artifact_inspection.py](lib/artifact_inspection.py) | Shared Debian identity checks and streaming hashes, used by inspection and promotion (also through the hosted task driver). | Importable offline policy; reads metadata using a caller-provided command adapter, never installs or executes artifacts. |
| [lib/dev_tasks.py](lib/dev_tasks.py) | Allowlisted just dispatch behind `dev.sh`, using the shared reporter. | Records recipe, HEAD, dirty-state flag and loose Git object counts (stderr warning if a task creates more than 2000); no independent build/test policy. |
| [lib/flatpak_tasks.py](lib/flatpak_tasks.py) | Validates tracked/pinned inputs, observes required runtimes, builds/exports and records inspected candidate receipts. | Uses shared export/bundle comparison against built metadata; no independent policy approval or shipping acceptance. |
| [lib/flatpak_inspection.py](lib/flatpak_inspection.py) | Complete keyfile comparison, selected OSTree ref/commit checks and temporary bundle import. | Shared by inspector/build driver; requires local ostree (and flatpak for bundles), never installs or launches. |
| [archive/](archive/README.md) | Frozen local historical scripts and their disposition. | Not current tooling, build inputs or installed payloads; historical contents remain Git-ignored. |

Keep operator-facing commands at this directory's top level and shared Python
implementation under `lib/`. Bash entry-point names remain stable; delegation
avoids duplicating JSON/state logic. The internal modules use Python's standard
library and are not installed applet payloads. Their safety/output contract is
documented in [output/resumption contracts](../docs/engineering/script-output-contracts.md).

No additional setup/build/release subdirectories are needed at the current size:
the catalog groups those purposes without breaking existing callers. Add new
P2 operator tools here and shared inspection logic under `lib/`; promotion now
delegates matrix inspection to that shared policy. Tests belong in `../tests/`, while
logs, receipts, packages and generated caches are ignored build output, not
maintained tools. Do not create a separate README for each two-file internal
directory unless its usage grows beyond this catalog.

## Reusing the tooling in another project

The development and release tools are written to be copied to another Rust
project (they use only the Python standard library, `just`, `git`, `gh` and, for
packages, `dpkg-deb`/`ostree`/`flatpak-builder`). Copy `scripts/`, the matching
`tests/*_test.py` and `tests/tooling_fixtures.py`, then adapt the items below.
Run the project's `just test` afterwards: the fixture tests catch most missed edits.

**Usually no change needed**

- `lib/task_output.py` (receipts, logs, cancellation) and the `.sh` wrappers.
- `dev.sh` / `lib/dev_tasks.py`, provided the project has `just` recipes with the
  allowlisted names. Add or remove names in `RECIPES` and in
  `tests/dev_wrapper_test.py`; keep the allowlist fixed, never pass arbitrary args.

**Edit first: [lib/project.py](lib/project.py)**

| Constant | Set it to |
|---|---|
| `PACKAGE` | The Cargo package / Debian package name. `tests/project_constants_test.py` checks it against `Cargo.toml`. |
| `WORKFLOW` | The manual, build-only GitHub workflow file (default `build-deb.yml`). It must not publish and should take a `request_id` input used in the run title (`Debian packages · <request_id>`), which run correlation relies on. |
| `MATRIX` | `(ubuntu version, arch)` pairs the workflow builds. Artifacts are named `deb-ubuntu-<version>-<arch>`; `ARTIFACT_COUNT` follows automatically. |

If your artifact names, package file names (`<PACKAGE>_<version>_ubuntu<ver>_<arch>.deb`)
or distribution differ, also update `lib/artifact_inspection.py`,
`version_from_packages()` in `lib/release_tasks.py` and `tests/tooling_fixtures.py`.

**Project files the tools read** (rename or adjust the paths if yours differ)

| Tool | Reads |
|---|---|
| `lib/publication_check.py` | `Cargo.toml` and `Cargo.lock`, `debian/changelog` (via `dpkg-parsechangelog`), `resources/app.metainfo.xml` (latest `<release>`), `docs/releases/v<version>.md` (must not contain "draft"). |
| `promote_release.py` | `docs/releases/v<version>.md` as the release body; runs `check-release-publication.sh`. |
| `lib/release_tasks.py` | `cargo metadata` for the version and `debian/changelog`; a clean branch on a GitHub remote. |
| `check-ui-translations.py`, `check-ui-localization.sh` | `src/**/*.rs`, `i18n/*/cosmic_ext_applet_mounter.ftl`, `tests/auth_helper_localization_test.py`. COSMIC/Fluent-specific: adapt the catalog name or drop the `localization-check` recipe. |
| `build-flatpak.sh`, `inspect-artifacts.py` | An explicit Flatpak manifest passed on the command line; drop them if you do not ship Flatpak. |

**Project-specific and not reusable as written**

- `install-dependencies.sh`, `setup-openconnect.sh` and the OneDrive auth helper
  belong to this applet.
- `archive/` holds retired runtime helpers; leave it behind.

**Checklist**

1. Copy the files and edit `lib/project.py`.
2. Make the `justfile` provide the recipes `dev_tasks.py` allows (`gate` is
   `fmt-check lint test`; `test` should run `cargo test --locked` and any Python tests).
3. Provide the workflow, `debian/`, AppStream file and `docs/releases/` layout
   above, or adapt the readers.
4. Replace the project-specific `AGENTS.md` variables block and Part II; keep the
   routine verification loop and approval gates, which refer to these scripts.
5. Run `just gate` and the Python tests; update this catalog and the script
   inventory for anything you removed.

## Maintenance

- The [engineering inventory](../docs/engineering/script-tooling-inventory.md) is
  the authoritative classification, caller evidence and tooling history (phases
  P1–P3 are complete for source and fixture verification; live package
  acceptance, installation and publication approvals remain separate). Update it
  and this catalog together whenever a tool is added, changed or retired.
- Operator and logging policy: [AGENTS.md](../AGENTS.md). Rebuild and release
  policy: [release and packaging procedures](../docs/engineering/release-packaging-procedures-contracts.md).
- [archive/](archive/README.md) holds frozen legacy tools, not installation or
  build inputs. Python `__pycache__` is generated output, not an archive
  candidate or an engineering record.
- Keep existing public script names stable. New tools use lowercase hyphenated
  names; importable Python modules in `lib/` use underscores. Renaming an old
  underscore-named tool requires a caller and link migration, not a cosmetic move.
- Every new tool gets a fixture test under `tests/` that runs in `just test`
  (the `tests/*_test.py` glob picks it up).
- `tests/repo_hygiene_test.py` fails if an entry point loses its git executable
  bit, is missing from this catalog, or if a relative Markdown link in the
  maintained docs breaks.

## OneDrive original/upload/download investigation

From the repository directory, first inspect with `--plan`; omit it to run:

```bash
bash scripts/onedrive-mirror-guard-check.sh --remote onedrive_personal --drive personal \
  --files-dir "$HOME/Cloud/OneDrive_Mirrors/test_folder" --round-trip --web-edit --plan
```

Use the corresponding business remote and label for a separate run. The directory
must contain 1..20 regular files (no subdirectories/symlinks), at most 5 MB. The
source is read-only. Baseline, upload and independent download copies remain in
`target/task-runs/onedrive-guard-check-*/work/`; only token-owned cloud folders are
purged. `--keep` deliberately retains them. Logs/results may contain sample names;
keep them private. Each command is bounded; per-transfer JSON logs rotate at 8 MB
with one backup and are read with an 8 MB cap. Cancelling terminates/reaps the
active command, attempts bounded marker-verified cleanup and retains partial JSON.

The new mode compares SHA-256/actual byte lengths separately from remote-reported
sizes, timestamps and QuickXorHash. It measures two no-edit syncs, and with
`--web-edit` verifies an actual local content change followed by a quiet sync.
Diagnostic downloads are labelled separately from bisync transfer totals.
Generic failure triggers an independent compatibility candidate including one
convergence pass; even a passing candidate remains inconclusive until the required
settings are minimized and any original-byte rewrite is explained. It never
silently ignores independent byte differences. Missing evidence, failed transfers,
incomplete cleanup or unresolved changes give a nonzero exit and partial results.
The default 10-second interval is an observation window, not a guarantee about
later service rewrites. Without `--web-edit`, no web-edit acceptance is claimed.
