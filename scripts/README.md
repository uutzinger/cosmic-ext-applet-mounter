# Script catalog

Run maintained scripts from the repository root unless their help says otherwise.
This directory separates project setup, installed runtime helpers, and developer/
release tooling by purpose; filenames remain stable for callers and user guides.
No script is authorized to install, alter services, tag or publish merely because
it is listed here. Inspect its help and obtain the applicable approval first.

## Project setup and runtime

| Script | Purpose | State changes / guidance |
|---|---|---|
| [install-dependencies.sh](install-dependencies.sh) | Install selected host storage/base/optional legacy OneDrive GUI dependencies; `--rclone` covers new OneDrive mirrors. | Package/repository/network changes, potentially privileged. No options means `--all`, **not** inspection. See [dependencies](../docs/dependency-installation.md). |
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
| [build-flatpak.sh](build-flatpak.sh) | Read-only `--preflight` source/runtime/installed-state inspection, or explicit-manifest candidate build/export, or verified reuse with --reuse-receipt. | Preflight only observes current/installed state and records blockers. Builds require clean pinned source and a new output; reuse needs a schema-2 receipt/successful summary and passes hashes, fsck and metadata checks without builder/runtime calls. No installation, launch or submission. |
| [check-ui-translations.py](check-ui-translations.py) | Catalog/reference and selected hardcoded UI-string checks. | Source checks, not full Fluent compilation or rendered UI review. |
| [check-release-publication.sh](check-release-publication.sh) | Version/metadata/finalized-note preflight (wrapper for `lib/publication_check.py`). | Read-only; does not prove package acceptance or grant approval. |
| [archive-release.sh](archive-release.sh) | After a release, move completed task-list and completion-note history to `docs/archive/` and carry open items forward. | Edits the two Markdown files and `docs/archive/` only; no commit. Needs `--tasks-from` / `--notes-from` heading prefixes; `--dry-run` reports counts. Version may carry a pre-release label (`0.5.4-pre`). Refuses existing archives and uncommitted edits to the two files. |
| [onedrive-mirror-prototype.sh](onedrive-mirror-prototype.sh) | Development-only OneDrive OAuth, identity and shared-coordinator lifecycle prototype. | Uses `src/microsoft_oauth.rs` for shared protected config/create/continue handling. Explicit fixed actions; `inspect` reads identity, `auth` creates only a fresh `cm_proto_` remote, `prepare` creates an owned disposable cloud tree and initial Preview. Later actions require its retained session. No applet configuration, installation, services or mount changes. See the operator sequence below. |
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
| [lib/onedrive_prototype_tasks.py](lib/onedrive_prototype_tasks.py) | Fixed-action reporter behind the prototype wrapper; runs the `development-probes` Rust example. | Private receipts and real exit status; Cargo quiet mode prevents account arguments entering logs. Identity/setup policy lives in production `src/onedrive.rs`; disposable scenario orchestration lives in `src/onedrive_prototype.rs`; driver fixtures in `tests/onedrive_prototype_tasks_test.py`. |
| [lib/mirror_guard_check.py](lib/mirror_guard_check.py) | Implementation behind `onedrive-mirror-guard-check.sh`: generic versus compatibility bisync profiles compared by stability of a no-edit second sync. | Mirrors the settings in `src/sync.rs`, including the 50% bisync deletion limit; exercised by `tests/mirror_guard_check_test.py` with a fake rclone only. |
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

## OneDrive development prototype (phase 3)

This harness extends the existing SharePoint design: it uses the same bisync
planner, coordinator, initial Preview/confirmed-sync markers, operation lock,
UUID access marker checks, dated recovery copies, retention and cancellation
runner. OneDrive selects `--compare size,modtime`; it does not enable SharePoint's
ignore-size/checksum flags, modify-window override or automatic convergence/check
pass. Initial and manual full verification are explicit harness operations.
Production provider routing and saved applet connections stay unchanged for phase 4.

Run from the repository root. The wrapper builds/runs a development-only Rust
example, logs under `target/task-runs/onedrive-prototype-*/`, and prints a shared
JSON receipt. `artifact_receipt_path` points to `prototype.json`; a successful
`prepare` result gives its retained absolute `session` path. All paths in a session
must be visible at the same absolute locations to the selected runner.

Begin with an identity inspection of an already configured remote:

```bash
bash scripts/onedrive-mirror-prototype.sh --action inspect \
  --remote onedrive_personal --drive personal --expected-account 'YOUR_PERSONAL_SIGN_IN'
```

Use the actual sign-in identity in place of the placeholder. Personal accepts
UPN or mail and checks Graph `/me`. Business requires the sign-in UPN and compares
`/me/drive` with `/users/{expected-UPN}/drive`: both must return the same business
drive ID and nonempty owner user ID. The configured drive must equal that primary
drive. Business uses file permissions and never calls the profile `/me` endpoint.
Missing metadata or a denied expected-account lookup fails the check. This rejects
another signed-in account, cache libraries, SharePoint document libraries,
custom clients, alternate roots and drive changes. Tokens stay in rclone config;
only identity fingerprints enter receipts. Credential-bearing config JSON uses a
restricted in-memory capture type with opaque Debug output; ordinary command
output remains redacted, and private command failures discard both output bodies.
Commands and Graph response bodies
are bounded; errors are protocol codes and never include provider output.

To test browser OAuth itself, use `--action auth --remote cm_proto_personal`
with the same other options. The name must be new and begin `cm_proto_`; existing
remotes are never overwritten. Browser sign-in uses rclone's default client.
Business uses rclone's default scopes, matching the working SharePoint setup;
it does not request `User.Read` or change existing grants. Personal requests the
default file scopes plus `User.Read`, required for its `/me` identity check.
Existing personal file-only remotes can pass file operations yet fail the
profile lookup. A 401/403 at profile `/me` emits `account_profile_unauthorized` with the
HTTP status; it does not establish that the drive or expected account is wrong.
For personal profile failures, use auth with a new name to obtain consent; inspect
never changes an existing remote's scopes or bypasses identity verification.
Policy or token issues can also deny profile access; new consent is not a
guaranteed fix for every 401/403. Business drive requests retain typed Graph
HTTP failures; file-scope auth can still be denied by tenant policy.
Partial failure rolls back only the name found absent before this attempt;
failed cleanup requires operator inspection of that named remote. Do not run
concurrent authentication attempts for the same development remote name.

Use disposable accounts for phase-3 acceptance. Each step below is a separate
explicit operator action with the same `--remote`, `--drive`, `--expected-account`
and `--transport` options as inspect:

| Action | What it does |
|---|---|
| `prepare` | Creates only `CloudMounter-Prototype-<UUID>/tree` plus an ownership marker; seeds synthetic local/remote files and runs the shared initial dry-run Preview. No initial sync until the next action. |
| `prepare --files-dir /ABSOLUTE/SAMPLES` | Copies the source into the new session's local/sample directory, verifies its SHA-256 tree inventory against the unchanged source, adds a synthetic timing file, then creates the owned cloud tree and Preview. No source write or initial sync. |
| `initial-sync` | With `--session /ABSOLUTE/PATH/FROM/PREPARE`, requires successful shared Preview, syncs and independently verifies downloaded bytes. |
| `verify` | With the same retained `--session`, rechecks identity/ownership/access and independently verifies downloaded bytes. Requires an initialized, nonfailed session; does not run bisync or repeat initial sync. |
| `scenario` | Uses that session for edits on both sides, deletes in both directions, preservation of both conflict versions, a 16 MiB upload and independent verification; finishes with timing measurements. |
| `benchmark` | Measures a quiet shared sync, then a quiet sync plus full-tree `check --checksum`, reporting both durations against a 300-second budget. It does not sleep five minutes or pretend this synthetic tree is representative. |
| `timing` | For an initialized sample session, measures quiet sync and sync plus checksum verification, changes only a synthetic timing file, measures that sync and independently verifies downloaded bytes. Records the initial-sync engine duration from retained stages; no full scenario or fault replay. |
| `fault-network` | Deliberately gives only the bisync child an unavailable source address, with bounded retries/timeouts; records observed failed coordinator state and leaves pending uploads. This controlled transport fault does not prove a physical network outage. |
| `fault-cancel` | Limits upload speed, disables server-side copying for the fault probe and cancels the active bisync after one second; waits for the shared runner to terminate/reap it and checks cancelled coordinator state. |
| `repair` | Verifies identity/access and inspects retained pending data and safety locks. Preserves them; never deletes locks, resyncs or changes real services. Production Repair/service acceptance stays pending. |
| `cleanup` | Verifies account, drive fingerprint and root ownership, then purges only the session's UUID cloud root, including its recovery copies. Local evidence/pending files remain. |

For safety, use a fresh prepared/initialized session for each fault case. A failed
session cannot resume ordinary sync automatically. Inspect/retain it with `repair`,
then explicitly `cleanup`; a new session requires its own Preview and initial-sync
confirmation. No implicit state rebuild, force-unlock or resync workflow is added.
A setup failure leaves private session.json under the prepare report's `session/`
directory, even if prototype.json only reports a failure code. Keep that evidence.
An initial-sync action can fail after bisync succeeds: inspect the retained
initialized flag and stages before retrying. Use `verify` for an initialized,
nonfailed session whose post-sync checks failed. A `rclone_failed` error contains
only fixed operation/failure categories and an optional exit code, never command
arguments or provider output. These diagnostics identify the failed read/check;
they do not establish its underlying network/provider cause.

Sample timing uses prepare with `--files-dir`, then separate initial-sync, timing
and explicit cleanup actions on that retained session. Use an existing absolute
source directory that does not overlap the session. Includes hidden files such
as .git and preserves empty directories. Rejects symbolic links, special files,
non-UTF-8 names, depth over 64, more than 20,000 entries or over 2 GiB. Source
and destination inventories compare relative names, types, byte counts and
SHA-256 contents; only aggregate counts/digest enter session state. A changed or
incomplete source/copy fails before cloud setup. Keep the source stable during
prepare. Later actions use only the copied files and a separate synthetic edit.
`timing` reports an operator-sample baseline with representative_tree=false and
cadence_exercised=false. The 300-second value is a comparison budget, not a
five-minute periodic-service test. Initial-sync engine time excludes the entry
guard; quiet/small-change and sync-plus-checksum measurements include their
guards. Larger real workloads and interval/cadence acceptance remain separate.

October 15 operator acceptance plan: normal native scenarios passed separately
for personal/business; cancellation, injected network failure and Repair passed
on personal using the shared coordinator/runner. Do not repeat those fault tests
on business or repeat completed scenarios unless relevant code changes or a
drive-specific failure warrants it. Business live fault behavior is not thereby
claimed verified. A scenario receipt's suggested next action is not a requirement
to repeat already accepted shared coverage.

October 15 sequencing decision: Flatpak acceptance is part of Task List phase 4
after applet integration, using the existing applet manifests/runtime and relevant
probe setup. Do not build a separate prototype environment as an implementation
prerequisite. The checks remain outstanding and are not inferred from native runs.
Use `--transport flatpak-host` when useful for focused checks of account-specific identity/
OAuth transport differences and one disposable initial-sync/verification/cleanup
smoke. Reuse unchanged native scenario/fault coverage. This exercises the existing
`flatpak-spawn --host rclone` transport; it is a Flatpak-style check, not final
installed-sandbox or portal acceptance. No permissions are added and no user systemd units are touched.
The host owns rclone credentials and cloud engine; the selected shared paths hold
local mirror/work/recovery state. Do not substitute transports after a failure.

The declared project minimum remains rclone 1.74.3. October 15 pinned-source
assessment confirms generic profile flag support at that minimum; live tests used
1.75.0, not a 1.74.3 binary. The measured sample timing is accepted; larger-tree
and repeated interval tests are deliberately skipped by operator decision, reusing
unchanged SharePoint scheduling coverage. OneNote/shortcut behavior remains open.
Full lifecycle source/fixture completion is not installed native/Flatpak
acceptance. `--help` is read-only.
