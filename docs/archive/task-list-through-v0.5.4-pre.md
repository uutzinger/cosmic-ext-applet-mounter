# Task List — archive through v0.5.4-pre

**Archived** after the v0.5.4-pre release. Frozen historical record; checkboxes here are
not maintained. Open items were carried forward to the active
[Task List](../../Task%20List.md) under "Open items carried forward"; that copy is
authoritative. Evidence: [completion notes](task-list-completion-notes-through-v0.5.4-pre.md).

---

## Release build workflow follow-up — October 13, 2026

User requested permanent reduction of repeated Debian builds after questioning
the attempted third matrix run during 0.5.3 publication. That run was not started.

- [x] Enforce one build per immutable source: build workflow is manual-only and
  cannot publish; tag pushes no longer start a redundant build. Local release
  script starts one matrix or reuses an exact-commit successful run via --run-id.
- [x] Separate publication into promotion-only workflow/script with explicit
  approval, same-repository/workflow/commit and remote-tag checks, complete
  unexpired matrix artifacts, checksums and package identity validation. No
  compilation or build dispatch occurs during promotion.
- [x] Record the rule in AGENTS.md/build guide; add offline policy, package and
  fake-command tests. Read-only reuse validation passed against existing tag
  run 38447948939. No new build/publication or change to v0.5.3 was performed.
- [ ] Verify the new hosted promotion workflow with the next approved immutable
  release. Source/offline tests and live metadata validation are not hosted
  publication acceptance; do not rebuild 0.5.3 merely to test this workflow.

## Agent guide efficiency and organization — October 13, 2026

- [x] Add rebuild justification, bounded output/status monitoring, reusable
  acceptance receipts and operator shell-handoff rules to AGENTS.md. Documentation
  diff checks pass; no build, installation or publication needed for these rules.
- [x] Agree on clearer general versus project/API-specific chapters, separate
  source/Debian/Flatpak workflows and Git/release script ownership. Discuss the
  layout before restructuring; retain approval and artifact-reuse boundaries.
  User approved the proposed layout. AGENTS.md now separates general policy
  from project-specific workflows, with a guide map and script ownership table.
  Refined the boundary after user review: reusable Rust/source/Debian/Flatpak/
  GitHub procedures are in Part I; concrete recipes, manifests, COSMIC/API
  choices and scripts are Part II project bindings.
- [o] Improve maintained scripts' log/summary controls and operator handoff
  support after the guide reorganization; no script implementation in this step.
  Handled by the authoritative
  [script tooling plan](#script-tooling-inventory-and-implementation-plan--october-13-2026).

## Script tooling inventory and implementation plan — October 13, 2026

- [x] Create scripts/README.md and the maintained
  [script/tooling inventory](../engineering/script-tooling-inventory.md),
  separating project deployment, runtime helpers and development/release tools.
  All nine active scripts retained after caller review.
- [x] Consolidate nine already archived local tools from archive/scripts/ into
  scripts/archive/ with byte parity, README and preserved Git exclusion. No
  active runtime payload retired; no install/service/data operation performed.
- [x] P0: Update the stale Rust Debian publication source contract for split
  build/promotion workflows; earlier Python-only checks missed this assertion.
  Use the focused test, not a repeat package matrix.
  Six focused contract tests pass, covering build-only/manual execution,
  explicit approved promotion without rebuilding, artifact verification and
  existing-run reuse. No package build or release operation performed.
- [x] Agree/implement the inventory's bounded logging/receipt contract and
  improve existing hosted/release scripts, precise run selection and safe resume.
  P1: shared private logs/atomic summaries, request-ID correlation, structured
  status polling, verified download cache, explicit tag/run resumption and
  persistent promotion receipts. Tests use fake CLIs only; next approved hosted
  acceptance remains separate. See
  [output/resumption contracts](../engineering/script-output-contracts.md).
- [x] Add the justified dev and shared-inspection wrappers; improve
  deployment-script reporting/tests. Keep existing just recipes as underlying
  implementation rather than creating competing procedures.
  - [x] P2 shared offline artifact inspector and extracted promotion verification
    policy: Debian matrix/single package and native byte receipts. Focused
    inspection, promotion and hosted-download fixtures pass; no real build.
  - [o] P2 local-Debian wrapper dropped by user: native source development is
    local; Debian packages use the six-target hosted workflow. Retain just deb
    for exceptional diagnostics, not a mandatory release gate.
  - [x] P2 fixed dev-task allowlist around existing just recipes, with private
    reporting; fixture checks cover output, rejection, exit status and cleanup.
  - [x] P2 opt-in setup reporting and focused fixtures; preserve standalone Bash
    use, existing dependency selections and service/data safety boundaries.
  - [x] P2 close fixture/regression, help/catalog and operator-handoff gate:
    70 applicable Python tests pass; source/fixture scope, not live deployment.
- [x] P3 explicit-manifest Flatpak build/export wrapper, shared metadata
  inspection and fake-command failure/reuse tests. Tooling implementation is
  independent of release/engine decisions; live build and final package acceptance
  stay in the project packaging plan. Installed-smoke wrapper remains optional.
  See the inventory's P2/P3 exit criteria for authoritative tooling scope.
  - [x] P3.1 explicit-manifest immutable-input build/export/bundle wrapper,
    private receipts and fake-command failure/cancellation checks. No live build.
  - [x] P3.2 shared Flatpak repository/bundle identity and complete metadata/
    permissions comparison, reused by build/export; fake tests and synthetic
    local OSTree/Flatpak checks pass. No installed/package acceptance claimed.
  - [x] P3.3 explicit schema-2 candidate reuse with successful producer summary,
    input/artifact identity, metadata and read-only fsck verification. No builder/
    export/runtime calls on reuse; mismatches fail without rebuild or cleanup.
    Final tooling gate: 93 Python methods pass; source/fixture acceptance only.
