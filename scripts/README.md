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
| [dev.sh](dev.sh) | Run a fixed just check/native-build recipe with private logs and concise phase/result output. | `bash scripts/dev.sh check`; supports `--log-dir` and bounded `--timeout-seconds`. No arbitrary arguments, install, cleanup, GUI launch or release tasks. |
| [build-flatpak.sh](build-flatpak.sh) | Explicit-manifest candidate build/export, or verified reuse with --reuse-receipt. | Clean source SHA and pinned inputs required. Builds need a new output; reuse needs a schema-2 receipt/successful summary and passes hashes, fsck and metadata checks without builder/runtime calls. No installation, launch or submission. |
| [check-ui-translations.py](check-ui-translations.py) | Catalog/reference and selected hardcoded UI-string checks. | Source checks, not full Fluent compilation or rendered UI review. |
| [check-release-publication.sh](check-release-publication.sh) | Version/metadata/finalized-note preflight. | Read-only; does not prove package acceptance or grant approval. |
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

Hosted monitoring defaults to 30-second polls; `--poll-seconds`,
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
| [lib/artifact_inspection.py](lib/artifact_inspection.py) | Shared Debian identity checks and streaming hashes, used by inspection and promotion (also through the hosted task driver). | Importable offline policy; reads metadata using a caller-provided command adapter, never installs or executes artifacts. |
| [lib/dev_tasks.py](lib/dev_tasks.py) | Allowlisted just dispatch behind `dev.sh`, using the shared reporter. | Records recipe, HEAD and dirty-state flag; no independent build/test policy. |
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

## Status and next work

The [engineering inventory](../docs/engineering/script-tooling-inventory.md) is
the authoritative current classification, reference coverage, gaps and proposed
implementation order. Proposed tools are **not implemented** by this catalog.
General operator/rebuild/logging policy lives in [AGENTS.md](../AGENTS.md).

[P2](../docs/engineering/script-tooling-inventory.md#p2-remaining-implementation-and-exit-criteria)
is complete for source/fixture verification: setup reporting, dev wrapper and
inspector are implemented. Live deployment remains separate. The local Debian wrapper
was dropped: packages come from the hosted matrix; `just deb` remains diagnostic.
[P3](../docs/engineering/script-tooling-inventory.md#p3-implementation-and-exit-criteria)
has implemented candidate build/export, shared metadata inspection and explicit
verified reuse (see the packaging guide for commands and recovery). These are
tooling phases, not release or engine-selection plans; live package acceptance
and installation/publication approvals remain separate.

[archive/](archive/README.md) contains local frozen legacy tools moved from the
older archive location, not installation or build inputs. The original nine active
scripts remain needed; P2 adds the inspector/dev wrapper and P3 adds Flatpak
orchestration. Python `__pycache__` is generated output, not
an archive candidate or an engineering record.

Keep existing public script names stable. For new tools prefer lowercase,
hyphenated names; internal importable Python modules use underscores. Renaming
old underscore-named tools requires a caller/link migration, not a cosmetic move.
Update this catalog, tests, packaging and the
inventory together when tools are added, retired or materially changed.
