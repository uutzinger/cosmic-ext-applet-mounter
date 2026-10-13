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

## Development and release tools

| Script | Purpose | Current boundary |
|---|---|---|
| [check-ui-localization.sh](check-ui-localization.sh) | Localization gate used by `just localization-check`. | Calls translation checker and helper localization tests; does not build. |
| [check-ui-translations.py](check-ui-translations.py) | Catalog/reference and selected hardcoded UI-string checks. | Source checks, not full Fluent compilation or rendered UI review. |
| [check-release-publication.sh](check-release-publication.sh) | Version/metadata/finalized-note preflight. | Read-only; does not prove package acceptance or grant approval. |
| [run_github_linux_build.sh](run_github_linux_build.sh) | Dispatch or retrieve a hosted Debian build. | Unique dispatch correlation, structured polling and verified downloads/cache. `--run-id` or `--resume-report` does not dispatch. No installation/publication. |
| [build_release.sh](build_release.sh) | Tag/push and start one hosted matrix or reuse an exact-commit run. | Tag/push approval required. `--dry-run` is non-mutating; `--resume --run-id` reuses the original tag/run without implicit rebuilding. Does not publish. |
| [promote_release.py](promote_release.py) | Validate/download an existing accepted build and optionally publish its packages. | Inspection default; publication requires `--publish --accepted` and approval. Persistent inspection receipt, no rebuild. |

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

Shared internals are [lib/task_output.py](lib/task_output.py) (receipts/logging/
local child cleanup) and [lib/release_tasks.py](lib/release_tasks.py) (fixed
build/preparation tasks). Bash entry-point names stay stable and delegate to this
Python standard-library implementation, avoiding duplicate Bash JSON/state logic.
See [output/resumption contracts](../docs/engineering/script-output-contracts.md).
These are build/development helpers, not installed applet payloads.

## Status and next work

The [engineering inventory](../docs/engineering/script-tooling-inventory.md) is
the authoritative current classification, reference coverage, gaps and proposed
implementation order. Proposed tools are **not implemented** by this catalog.
General operator/rebuild/logging policy lives in [AGENTS.md](../AGENTS.md).

[archive/](archive/README.md) contains local frozen legacy tools moved from the
older archive location, not installation or build inputs. Nine active scripts
were reviewed; all remain needed. Python `__pycache__` is generated output, not
an archive candidate or an engineering record.

Keep existing public script names stable. For new tools prefer lowercase,
hyphenated names; internal importable Python modules use underscores. Renaming
old underscore-named tools requires a caller/link migration, not a cosmetic move.
Update this catalog, tests, packaging and the
inventory together when tools are added, retired or materially changed.
