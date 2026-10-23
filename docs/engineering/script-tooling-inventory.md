# Script and development-tooling inventory

Type: Inventory

Status: P0–P3 source/fixture gates complete; separate live/hosted package acceptance pending

Created: October 13, 2026

Last reviewed: October 20, 2026

## Scope and method

Supports the task-list script review after the agent-guide reorganization.
Authoritative sources: `scripts/`, justfile, Debian rules/control, hosted
workflows, packaging manifests, Rust/Python tests, runtime helper launch code
and current setup/build guides. Reviewed source interfaces and callers; searched
active source/tests/packaging/docs for archive references. Did not run installers,
authentication helpers, builds, service changes, tag/push or publication commands.
This is a tooling inventory, not a full installer security or distro-validation
audit. Installed executables and external user scripts were not inventoried.

This record owns script organization, interfaces, implementation and verification
only. Product engine selection, dependency policy, release scheduling and package
runtime approval belong to their project plans. Scripts consume the selected
configuration; improving them does not require making those product decisions.

General requirements: one build per immutable source/required target, separate
state-changing approvals, bounded logs with real exit status, operator handoff,
preserved data and script-first repeatable workflows. See [AGENTS.md](../../AGENTS.md).

## Active scripts and callers

October 16 installed acceptance: `scripts/install-flatpak.sh` and
`scripts/lib/flatpak_install.py` provide the missing approved candidate-install
entry point. Packaging guide/operator calls it with the existing candidate receipt.
It reuses full candidate verification, refuses competing native/Flatpak owners,
retains prior ref/commit, installs only the user bundle without dependency/related
updates, verifies installed identity/metadata and records user overrides.
Fake-adapter tests cover invalid candidates, running owners, partial failure,
post-install mismatch and reversible panel launcher selection. Optional
`--activate-panel-entry` backs up and parks the user override so COSMIC discovers
the real Flatpak export (location determines socket forwarding); symbolic
destination paths are refused. `--panel-entry-only` verifies an already-installed
candidate and only corrects discovery. No automatic
rollback, app launch or service changes.

| Script in scripts/ | Category | Current evidence/callers | Disposition |
|---|---|---|---|
| `install-debian.sh` | Standalone release-package setup | README curl command; `tests/debian_installer_test.py` fake network/package adapters | October 23: default advanced to published 0.5.5. Added October 19: fixed 0.5.4 default with explicit version override; distro/native architecture selection, bounded HTTPS downloads, checksum and package identity verification, APT install/reinstall and private temporary-download cleanup. Fails closed on unsupported targets, missing releases or bad artifacts; no real install acceptance yet. No optional engine setup or automatic app/service stop. |
| `install-flatpak-release.sh` | Standalone GitHub bundle setup | README curl command; `tests/flatpak_release_installer_test.py` fake adapters | October 23: default advanced to published 0.5.5. Added October 19: x86_64/aarch64 CPU mapping, fixed 0.5.4 default/explicit version, selected checksum verification before user Flathub/runtime/bundle changes. Requires preinstalled Flatpak; no root, build/CI, engine setup or rollback. Fake tests include partial failures and local --bundle/sibling checksum mode. Both hosted CPU bundles verified October 20; installed ARM acceptance not established. Separate from receipt-driven install-flatpak.sh. |
| `build-flatpak-ci.sh`, `lib/flatpak_ci.py` | Manual hosted Flatpak build | `.github/workflows/build-flatpak.yml`, `tests/flatpak_ci_test.py` | Added October 19: native x86_64/aarch64 runner matrix, ephemeral runner-only APT/Flatpak/user-namespace setup, existing build/export driver and portable verified upload set. Manual dispatch/clean-source guards; no dispatch by the script, app install or publication. First x86_64 hosted build failed during stripping (missing eu-strip); elfutils/tool preflight fixed in ba8a341. Run39382160325 ARM succeeded and was preserved/checksummed; x86_64 AppStream failure reproduced without SVG loader. Explicit librsvg/AppStream deps and pinned-source compose preflight added; architecture-only retry and diagnostic hints retained. October20 x86_64 run39433010848 succeeded; both CPU artifacts preserved and offline-inspected. Installed ARM/final publication acceptance remain pending. |
| `install-dependencies.sh` | Project host setup | Dependency guide, installer/reporting fixtures, component selections | P2 optional --log-dir implemented; visible prompts, private logs/atomic result. October 18: helper-only GUI install/probe/options retired. October 20, 0.5.5: legacy engine installer and --onedrive removed; --all selects base, rclone and onedriver. Retired options fail before privileged work; --rclone-only isolation retained. OBS mapping now uses actual distro ID so Pop/Ubuntu are not mistaken for Debian; VM retry pending. No-options installation is not read-only. |
| `setup-openconnect.sh` | Project host setup | OpenConnect guide and fake package/service/setting/report fixtures | P2 optional --log-dir implemented; installs/changes service and icon as before. Uninstall retains packages/profiles. No live setup execution counted. |
| `cosmic-ext-applet-mounter-onedrive-auth-helper` | Retired runtime payload | October 18 operator moved helper-only retirement into 0.5.4 | Source, install payloads, dispatcher, GUI dependency options and helper-localization tests removed. Existing legacy browser/manual handoff retained. No longer a runtime consumer. |
| `check-ui-localization.sh` | Development gate | just localization-check; calls translation checker | Keep maintained entry point. October 18 retired helper-localization tests removed with helper; catalog/source checks remain. |
| `check-ui-translations.py` | Source verification | localization wrapper and audit; checks catalog identity/literal references and selected UI strings | Keep. Already concise; not a general Fluent parser or rendered UI test. |
| `check-release-publication.sh` | Release preflight | publication fixtures and promotion script; validates versions, stable metadata and finalized notes | Keep. Thin wrapper over `lib/publication_check.py` (October 13: moved from an inline heredoc, TOML parsed with `tomllib` or `tomli` on Python 3.10); read-only, does not replace human acceptance. |
| `onedrive-mirror-prototype.sh` | OneDrive phase-3 development prototype | `lib/onedrive_prototype_tasks.py`, gated Rust example/module; Rust identity/coordinator fixtures and `tests/onedrive_prototype_tasks_test.py` | Fixed explicit auth/inspect/prepare/verify/timing/lifecycle/fault/repair/cleanup actions; shares SharePoint planner/coordinator and runner, uses generic OneDrive profile. Native and forced host transport; source/fake/local-engine checks and full gate passed October 15; first inspect exposed parsing of redacted credential JSON, fixed with restricted private capture; 13 focused tests and full gate passed. Retry reached Graph HTTP 401; added User.Read to new prototype auth for the /me profile check and typed profile-denial errors. Eleven focused prototype tests and the full gate passed; native personal OAuth/identity and preparation passed. Personal initial bisync succeeded but a post-sync guard failed; safe typed command diagnostics and verification-only retry passed 12 focused Rust tests, 2 driver tests and the full gate; personal native scenario/verification and cancellation/network-fault inspection passed. Business admin-consent failure exposed extra profile scope; revised business auth uses default scopes and server drive/owner binding; 15 focused tests and the full gate passed. Native OAuth/identity and normal scenarios passed for both drive types October 15; shared faults/Repair passed on personal. Operator explicitly omits redundant business fault runs; all native test cloud roots cleaned up. Focused Flatpak acceptance transfers to phase-4 applet integration using existing manifests/probes; sample-folder copy/inventory and dedicated timing action added for a small-project baseline; 17 focused Rust tests, 2 driver tests and the full gate passed; representative performance remains open. Not installed applet payload or service management. October 15: OAuth/config capture now delegates to shared microsoft_oauth.rs with the production SharePoint wrapper; provider identity policies remain separate. No new live authorization result is claimed for the extraction. Phase-4 identity/setup policy is now production src/onedrive.rs; the harness delegates to it while preserving disposable ownership and its cm_proto_ creation restriction. Script actions/receipts unchanged. |
| `onedrive-mirror-guard-check.sh` | Live OneDrive bisync guard check | fake-rclone fixtures (`tests/mirror_guard_check_test.py`) | Added October 14: operator-run, changes only its own uniquely named test folder on a OneDrive remote; settles per-drive-type guard handling for the rclone OneDrive Offline mirror. October 15: explicitly create the remote directory before initial bisync; fake backend now rejects missing directories and tests creation failure. Business and personal legacy checks passed October 15. New opt-in schema-2 module `lib/mirror_guard_roundtrip.py` adds original/download comparison, structured counters and marker-content cleanup; `tests/mirror_guard_roundtrip_test.py` owns fixtures/local-only smoke. First extended attempt exposed unsupported version JSON flag; provenance now captures plain version text, with strict fake and installed-rclone regression coverage. Extended personal/business live tests passed October 15: original/download bytes and reported sizes match, idle transfers zero, cleanup purged; receipts in completion notes. October 15: both profiles now use the shared 50% bisync deletion limit; strict fake commands reject the former value and force overrides. Earlier cloud receipts retain their historical settings. |
| `run_github_linux_build.sh` | Hosted build/retrieval | build_release.sh, guide, fake-gh download tests | P1 implemented: delegates to fixed Python task; correlated run selection, bounded status/logging, cache verification and resumable receipts. Hosted execution remains separate acceptance. |
| `build_release.sh` | Release preparation | guide, AGENTS and fake-command promotion tests; creates/pushes tag and builds/reuses one run | P1 implemented: common reporting and explicit original-source --resume --run-id. Network/tag mismatches fail closed; no implicit rebuild or retag. |
| `promote_release.py` | Artifact inspection/promotion | promotion workflow, guide, policy/package/fake-command tests | P1/P2 Debian verification retained. October 20 adds explicit per-CPU hosted Flatpak selection, portable provenance/combined checksums and exact-tag publication enforcement; --inspect-flatpaks retains candidates without publishing. Fake policy/publication tests and real offline inspection are separate from installed acceptance. |
| `inspect-artifacts.py` | Offline artifact inspection | Shared Debian/native and Flatpak metadata verifiers; fixtures registered in just/Debian tests | Implemented: package/byte identity and explicit-baseline repository/bundle metadata comparison. No builds/network/install, provenance or installed-acceptance claim. |
| `dev.sh` | Logged native development | Fixed just recipes through lib/dev_tasks.py; fixture tests in just/Debian gates | Implemented: private logs, concise results (including `source_dirty` and a bounded failure excerpt on stderr), recipe/HEAD/dirty identity, real exit status and cancellation; `gate` = fmt-check, lint, test. No install/cleanup/release tasks. |
| `build-flatpak.sh` | Read-only preflight and candidate build/export/reuse | Explicit clean manifest checkout, pinned sources and fake-command fixtures | Implemented: read-only --preflight source/installed metadata and override receipts; typed runtime/SDK/extension refs and BaseApp app ref; typed/fingerprinted generated Cargo inline/shell sources; build-only exact-pin local Git transport and child-only environment recorded in receipts; build/export, shared inspection and explicit schema-2 reuse with fsck/hashes/successful producer summary. No implicit rebuild or install/launch/submission; mutable-tag manifest is not silently fixed. |

These tools contain project bindings even where their underlying task is
reusable. Do not move installers into generic build tooling, rename public
download URLs cosmetically, or archive current runtime payloads. New names
should be lowercase/hyphenated; existing underscore names remain stable.

## Internal modules and layout

The twelve public commands above are distinct from six maintained support modules:
`scripts/lib/task_output.py` supplies private logging, atomic receipts and child
cleanup to promotion and the task driver; `scripts/lib/release_tasks.py` supplies
the fixed hosted-build/release-preparation tasks behind the two Bash wrappers.
The latter reuses promotion's compatibility wrapper around the extracted
`scripts/lib/artifact_inspection.py` policy. The standalone inspector uses that
same module; checksums are streamed and symbolic/nonregular inputs are rejected.
`scripts/lib/dev_tasks.py` supplies the fixed just-recipe dispatcher behind dev.sh; `scripts/lib/project.py` holds the project constants (package, workflow, artifact matrix) shared by release tooling.
`scripts/lib/flatpak_tasks.py` supplies tracked/pinned input checks, runtime
observations and the candidate build/export/receipt driver behind build-flatpak.sh.
`scripts/lib/flatpak_inspection.py` supplies selected-ref/commit and complete
metadata comparison, including temporary bundle import, for both that driver
and inspect-artifacts.py. No app or runtime installation is involved.

`lib/flatpak_promotion.py` supplies run/job/source/checksum/input/runtime policy
to promote_release.py and delegates bundle import to flatpak_inspection.py.
Focused tests use synthetic artifacts and fake GitHub/OSTree adapters; no package
installation or publication is implied.

Keep public entry points at `scripts/`, internal implementation at `scripts/lib/`,
and frozen historical material at `scripts/archive/`. The current size does not
justify further setup/build/release subdirectories or public-name migrations.
The [script catalog](../../scripts/README.md#internal-modules-and-directory-layout)
documents each module's callers and execution boundary. Tests stay in `tests/`;
generated logs, receipts, packages and caches are not maintained scripts.

## Archive disposition

No active script was shown to be obsolete. Nine **already archived**, locally
ignored tools moved from `archive/scripts/` to `scripts/archive/`, preserving
all bytes by before/after SHA-256 comparison:

- `cisco-vpn-start`, `cisco-vpn-stop`, `cisco-vpn-status`
- `cloud-box-onedrive-toggle`, `cloud-box-onedrive-toggle-terminal`
- `cloud-google-toggle`, `cloud-google-toggle-terminal`
- `ua-engr-mount`, `ua-engr-mount-terminal-stop-all`

Their old proprietary Cisco/shell-toggle/service coordination paths have been
superseded by applet-managed connections. No current source, test, recipe,
packaging or current guide reference to the old directory/names was found.
Frozen material is not executed or installed. Existing Git exclusion is
preserved with scripts/archive/.gitignore; only its README/policy are tracked.
A narrow literal-secret scan was negative, not a full privacy review. Do not
silently publish ignored historical files. Original parent/name and disposition
are documented in [archive README](../../scripts/archive/README.md).

## Build/development capabilities and gaps

Statuses below distinguish existing capability, an update to existing tooling,
and a proposed new file. They do not authorize executing any state change.

| Needed capability | Already available | Action / proposed owner |
|---|---|---|
| Native compilation and development launch | just build/build-release/run | Keep recipes. No duplicate build-source.sh needed. |
| Focused and aggregate verification | just fmt-check/check/lint/test/localization-check/verify; direct focused tests | Keep recipes; add logged task wrapper, not duplicate test policy. |
| Staged and user-local source installation | just stage/install-user/uninstall; build guide and live receipts | Keep recipes. A dedicated safe reload/install wrapper is optional only if repeated handoffs justify it; no new mandatory installer now. |
| Project dependencies/OpenConnect setup | Two standalone Bash scripts with opt-in private logs/JSON summaries | Implemented and fixture-tested. Prompt output remains visible; identical embedded reporter is parity-tested. Keep separate from build/release defaults. |
| Bounded output, logs, reliable exit status and receipt schema | P1: scripts/lib/task_output.py and output/resumption contract | Implemented. Replaced the proposed Bash-only task-output.sh with a shared standard-library Python reporter; stable Bash entry points delegate rather than duplicating JSON/state handling. No arbitrary shell-text/eval interface. |
| Operator-friendly native checks/builds | P2: dev.sh plus lib/dev_tasks.py over just recipes | Implemented and fixture-tested; private logs, real exit codes and fixed tasks only. |
| Local Debian candidate creation and collection | just deb/debian rules | Deliberately dropped from P2. Native development uses source builds; Debian packages use hosted matrix artifacts. Keep just deb only for exceptional diagnostics; no build-deb.sh needed. |
| Hosted matrix creation/retrieval | P1: stable Bash entry point, lib/release_tasks.py, correlated manual build workflow | Implemented and fixture-tested. Reuse selected runs, exact source and request-ID selection; no newest-run shortcut. |
| Reusable artifact inspection | P2: inspect-artifacts.py and lib/artifact_inspection.py, reused by promotion | Implemented/fixture-verified for offline Debian/native identity/bytes. P3 adds explicitly scoped Flatpak inspection. |
| Release preparation and artifact promotion | P1: existing entry points plus shared reporter/fixed task driver | Implemented logs/receipts/resume, with fixture verification. No competing release.sh or rebuild merely to publish. Next hosted acceptance remains open. |
| Flatpak build/inspection orchestration | build-flatpak.sh, lib/flatpak_tasks.py and shared lib/flatpak_inspection.py | P3 implemented/fixture-tested: immutable inputs, build/export, metadata comparison and explicit verified reuse without builder calls. Reuse adds fsck; original runtime/tool evidence retained. No shipping acceptance implied. |
| Installed native/sandbox smoke | Manual/live receipts and some tests, not one reusable command | Optional later `scripts/check-installed.sh` for read-only identities/services/mount ownership. Never automate repair/unmount/reload as an implicit check. UI acceptance stays operator-assisted. |

The minimal P0–P3 tooling plan is implemented and fixture-verified: shared
reporting, existing hosted/setup improvements, inspector, dev and Flatpak wrappers,
and verified reuse. The local Debian wrapper was deliberately dropped.
Installed-smoke tooling remains optional; do not build a framework merely to
wrap a one-off command. Source/fixture completion is not final-package acceptance.

### Verification gap found during this inventory (P0, resolved October 13)

`tests/debian_packaging_contract.rs::publication_requires_explicit_manual_acceptance`
initially asserted that build-deb.yml contained `needs: build`, the removed
publish_after_acceptance input and the former publication job. Source inspection
showed those assertions conflicted with the new split workflows. The earlier
single-build change checked Python suites but missed this Rust source contract.
This was a regression-test gap, not evidence that the production publication
path should rebuild again. The initial inventory did not compile merely to
reconfirm an explicit static mismatch.

P0 resolution: replaced the stale test with four source contracts for manual
build-only execution, explicitly approved promotion without rebuilding, delegated
artifact/metadata verification, and release preparation with existing-run reuse.
Both existing dependency/payload contracts remain. All six focused Rust tests
pass; log: `target/p0-packaging-contract.AzPD82/test.log`. Test-target compilation
took about two seconds, not a package/release build. Formatting and diff checks
pass. This verifies source contracts, not hosted publication or installed UI.

## Proposed implementation order

1. **P0 — Completed:** repaired the stale Rust packaging contract and ran its
   six focused tests with bounded logging. It now forbids rebuild-on-publish.
   No full package matrix was needed; proceed to P1.
2. **P1 — Completed, source/fixtures:** agreed the output contract: selected task/source/run identity,
   artifact path/hash when applicable, log path, exit code, concise result and
   operator next step. Keep raw sensitive output private; retain true exit
   status through redirection/pipelines and bound failure excerpts.
3. **P1 — Completed, source/fixtures:** implemented reporting helper and hosted/release updates, including
   precise run selection, polling intervals and restart/resume boundaries.
   Use fake commands/fixtures for failures and cancellation; no live publishing
   or redundant builds to test orchestration.
   New dispatch uses a workflow request_id/run title. Existing old-source runs
   remain retrievable by explicit ID; new correlated dispatch requires the
   updated workflow in the selected source. Resume/cache and cancellation are
   fixture-verified, not a claim of new hosted publication acceptance.
4. **P2 — Completed, source/fixtures:** local development and deployment tooling, using the
   concrete work and exit criteria below. Shared inspection is already done.

5. **P3 — Completed, source/fixtures:** Flatpak orchestration and inspection, using an explicit supplied
   manifest and fixture verification. Script implementation is not gated on a
   release number or engine selection; final-package acceptance is separate.

## P2 remaining implementation and exit criteria

Implement in this order; extend the output contract before each interface change.

1. **[o] Local Debian wrapper dropped.** Use source builds locally and hosted
   matrix artifacts for Debian packaging; a local package cannot validate six
   hosted targets. Keep just deb for exceptional diagnostics only.
2. **[x] Development wrapper — `scripts/dev.sh`.** Fixed task-to-just-recipe
   allowlist for checks and native builds, using the common private log/summary
   interface and real exit status. Reject arbitrary shell text, recipe injection
   and unapproved passthrough flags. Do not include installation, uninstallation,
   cleanup, panel reload or publication as implicit tasks. Keep justfile as the
   authority for command sequencing; record which recipe ran and its boundary.
3. **[x] Existing setup-script reporting and fixtures.** Improve
   `install-dependencies.sh` and `setup-openconnect.sh` without changing selected
   packages, versions, service policy or existing defaults. Preserve documented
   standalone download/pipe-to-Bash use: no mandatory checkout-relative Python
   imports or new bootstrap dependencies. Define an opt-in logged/receipt mode
   that preserves interactive privilege prompts and keeps secrets out of public
   output. Separate inspection/help from mutations; never rerun an installer
   automatically after failure. Cover package-manager failure, component options,
   generated unit paths, icon setting, uninstall preservation and partial failure
   using fake commands and temporary user directories.
4. **[x] Close the tooling gate.** Register tests in maintained recipes; update
   help, catalog, module callers, inventory and operator handoff examples. Test
   noisy output, cancellation, real exit codes, conflicting destinations, dirty
   inputs, failed inspection and reuse without rebuilding. Preserve earlier P1
   regression coverage. Use fixtures first; a real candidate build or setup run
   is separately authorized and does not replace fixture verification.

P2 is complete when these interfaces and applicable fixtures pass and their
boundaries are documented. No package matrix, release, engine migration or live
installer execution is required merely to close the script implementation phase.

## P3 implementation and exit criteria

1. **[x] Flatpak wrapper — `scripts/build-flatpak.sh` (source/fixtures).** Require an explicit
   manifest and immutable source/input identity, with unique build/output paths
   and the common log/summary interface. Record application ID/branch,
   runtime/SDK/BaseApp, manifest/dependency-input hashes, tool identity and
   effective permissions. Reject missing dependencies with a typed/clear failure;
   never substitute a runtime, broaden grants or regenerate dependency inputs
   silently. No default `--force-clean`, installation, launch or submission.
   Existing outputs are refused by default; explicit verified reuse is implemented below.
2. **[x] Flatpak artifact inspection (source/fixtures and synthetic local CLI check).** Extend the shared inspector with an
   agreed bundle/repository metadata contract: application/ref identity,
   checksums, runtime and permissions. Clearly distinguish a prototype,
   successful build and accepted shipping artifact. Do not execute the app to
   inspect metadata or equate a supplied manifest with built effective grants.
3. **[x] Fixtures, verified reuse and handoff.** Build/export/inspection and
   explicit schema-2 reuse are implemented with producer-summary validation,
   source/artifact fingerprints and read-only fsck. Reuse does not invoke the
   builder or require installed build runtimes. Fake commands cover wrong ref,
   missing runtime, permission mismatch, build/export failure, cancellation,
   destination conflicts and explicit reuse without rebuilding. Document exact
   operator commands and separate build/export from installation/publication.
   Final gate: 93 applicable Python methods passed, plus Bash/Python syntax,
   help and whitespace checks. Real fsck passed on the earlier metadata-only
   synthetic repository; no Cloud Mounter rebuild or installed acceptance claimed.

P3 script completion requires the contract, implementation and fixtures, not a
particular product release. A live build against the chosen project manifest and
final sandboxed-package acceptance remain distinct project verification gates.
An installed-smoke script remains optional, not a P2/P3 completion requirement;
add it only for a demonstrated recurring read-only inspection gap.

## Acceptance and maintenance

Maintain [scripts README](../../scripts/README.md), caller/test references and
this inventory together. Proposed archive moves require positive replacement/
non-use evidence, not filename age or lack of direct UI use. A helper invoked by
another helper is still used. Preserve historical receipts and identify any
relocation rather than rewriting history. Source/syntax/link checks establish
documentation/structural correctness only; safe fixtures, installed UI and final
package acceptance remain separate verification boundaries.
