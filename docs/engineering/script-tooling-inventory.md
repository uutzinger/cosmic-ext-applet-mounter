# Script and development-tooling inventory

Type: Inventory

Status: Maintained inventory; P0 and P1 implemented with focused verification, P2/P3 pending

Created: October 13, 2026

Last reviewed: October 13, 2026

## Scope and method

Supports the task-list script review after the agent-guide reorganization.
Authoritative sources: `scripts/`, justfile, Debian rules/control, hosted
workflows, packaging manifests, Rust/Python tests, runtime helper launch code
and current setup/build guides. Reviewed source interfaces and callers; searched
active source/tests/packaging/docs for archive references. Did not run installers,
authentication helpers, builds, service changes, tag/push or publication commands.
This is a tooling inventory, not a full installer security or distro-validation
audit. Installed executables and external user scripts were not inventoried.

General requirements: one build per immutable source/required target, separate
state-changing approvals, bounded logs with real exit status, operator handoff,
preserved data and script-first repeatable workflows. See [AGENTS.md](../../AGENTS.md).

## Active scripts: nine retained

| Script in scripts/ | Category | Current evidence/callers | Disposition |
|---|---|---|---|
| `install-dependencies.sh` | Project host setup | README, dependency guide, dependency-installer tests; component selectors, helper-runtime checks and package-manager operations | Keep. Update logging/summary interface later; engine selectors depend on 0.5.4 decisions. No-options installation is not read-only. |
| `setup-openconnect.sh` | Project host setup | OpenConnect guide; installs packages, changes icon setting and writes on-demand agent unit; uninstall removes unit, not packages/profile | Keep. Add focused service/setting fixtures and bounded reporting later. Do not remove until COSMIC-agent replacement is accepted. |
| `cosmic-ext-applet-mounter-onedrive-auth-helper` | Installed runtime payload | just/debian installation, app helper launcher, localization tests and package contracts | Keep. Not obsolete because Flatpak helper packaging is deferred. Any retirement follows engine/auth decisions and migration. |
| `check-ui-localization.sh` | Development gate | just localization-check; calls translation checker plus helper localization tests | Keep wrapper: it adds test coverage beyond the checker alone. |
| `check-ui-translations.py` | Source verification | localization wrapper and audit; checks catalog identity/literal references and selected UI strings | Keep. Already concise; not a general Fluent parser or rendered UI test. |
| `check-release-publication.sh` | Release preflight | publication fixtures and promotion script; validates versions, stable metadata and finalized notes | Keep. Read-only and already concise; does not replace human acceptance. |
| `run_github_linux_build.sh` | Hosted build/retrieval | build_release.sh, guide, fake-gh download tests | P1 implemented: delegates to fixed Python task; correlated run selection, bounded status/logging, cache verification and resumable receipts. Hosted execution remains separate acceptance. |
| `build_release.sh` | Release preparation | guide, AGENTS and fake-command promotion tests; creates/pushes tag and builds/reuses one run | P1 implemented: common reporting and explicit original-source --resume --run-id. Network/tag mismatches fail closed; no implicit rebuild or retag. |
| `promote_release.py` | Artifact inspection/promotion | promotion workflow, guide, policy/package/fake-command tests | P1 implemented: private logs and persistent name/hash/origin inspection receipts. Default remains no-publication. Shared standalone inspector remains P2. |

These tools contain project bindings even where their underlying task is
reusable. Do not move installers into generic build tooling, rename public
download URLs cosmetically, or archive current runtime payloads. New names
should be lowercase/hyphenated; existing underscore names remain stable.

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
| Project dependencies/OpenConnect setup | Existing two setup scripts | Update reporting and tests; keep separate from build/release defaults. |
| Bounded output, logs, reliable exit status and receipt schema | P1: scripts/lib/task_output.py and output/resumption contract | Implemented. Replaced the proposed Bash-only task-output.sh with a shared standard-library Python reporter; stable Bash entry points delegate rather than duplicating JSON/state handling. No arbitrary shell-text/eval interface. |
| Operator-friendly native checks/builds | just recipes; no durable bounded-output wrapper | Create `scripts/dev.sh` as a fixed task-to-recipe dispatcher with log/summary/exit reporting; no installation/publication by default. |
| Local Debian candidate creation and collection | just deb/debian rules; ad hoc output collection during 0.5.3 | Create `scripts/build-deb.sh` around existing Debian build, with explicit output/log paths, source provenance and inspection handoff. Do not silently skip package tests or rebuild a reusable accepted candidate. |
| Hosted matrix creation/retrieval | P1: stable Bash entry point, lib/release_tasks.py, correlated manual build workflow | Implemented and fixture-tested. Reuse selected runs, exact source and request-ID selection; no newest-run shortcut. |
| Reusable artifact inspection | promote_release.py covers six Debian artifact checks; additional inspection was ad hoc | Create `scripts/inspect-artifacts.py` for offline/read-only inspection and receipt generation, initially Debian/native. Reuse it from promotion/local packaging; add Flatpak support after its acceptance contract is agreed. |
| Release preparation and artifact promotion | P1: existing entry points plus shared reporter/fixed task driver | Implemented logs/receipts/resume, with fixture verification. No competing release.sh or rebuild merely to publish. Next hosted acceptance remains open. |
| Reproducible Flatpak build/inspection | Packaging guide/manifests and just flatpak-cargo-sources; no unified shipping build wrapper | Create `scripts/build-flatpak.sh` after engine/runtime/manifest decisions. Separate build from installation/submission, distinguish prototypes, record source/runtime/grants. |
| Installed native/sandbox smoke | Manual/live receipts and some tests, not one reusable command | Optional later `scripts/check-installed.sh` for read-only identities/services/mount ownership. Never automate repair/unmount/reload as an implicit check. UI acceptance stays operator-assisted. |

Proposed P2/P3 files are a plan, **not existing commands**. Shared logging and
existing-tool improvements are implemented first in P1. The remaining minimal
new-tool plan is a dev task wrapper, local Debian wrapper,
shared artifact inspector, and later Flatpak wrapper. Installed-smoke tooling
is optional; do not build a framework merely to wrap a one-off command.

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
4. **P2 — Add dev/local-Debian wrappers and shared inspection** using existing
   just/Debian recipes and promotion policy, with tests. Keep install approval
   and artifact inspection separate from building.
5. **P2 — Add deployment-script reporting/tests**, preserving their service,
   package, credential and rollback boundaries. Revisit optional engine/runtime
   dependency choices only after the 0.5.4 engine decision.
6. **P3 — Shipping Flatpak wrapper and affected acceptance**, after decisions
   already tracked under 0.5.4. Consider installed-smoke wrapper only if needed.

## Acceptance and maintenance

Maintain [scripts README](../../scripts/README.md), caller/test references and
this inventory together. Proposed archive moves require positive replacement/
non-use evidence, not filename age or lack of direct UI use. A helper invoked by
another helper is still used. Preserve historical receipts and identify any
relocation rather than rewriting history. Source/syntax/link checks establish
documentation/structural correctness only; safe fixtures, installed UI and final
package acceptance remain separate verification boundaries.
