# Task List Completion Notes — archive through v0.5.4-pre

**Archived** after the v0.5.4-pre release. Frozen historical record; checkboxes here are
not maintained. Open items were carried forward to the active
[Task List](../../Task%20List.md) under "Open items carried forward"; that copy is
authoritative. Tasks: [task list](task-list-through-v0.5.4-pre.md).

---

### Permanent single-build release path — October 13, 2026

User's task-list note retained as historical request:
"ATTEMPT TO BUILD DEB 3 TIMES / Complete changes to build workflow where rate
limits stopped you. / What you mean by build worklow? did you decide on your
own to build 3 tiems?" Converted that discussion into an actionable chronological
follow-up in Task List.md without losing its purpose.

✅ Root cause: AGENTS.md never required three builds. The old sequence used a
branch matrix, automatic tag matrix, and manual publication workflow depending
on a fresh matrix. The third dispatch during 0.5.3 was denied/not started; native
0.5.3 was published by reusing verified tag artifacts. Historical tag unchanged.

✅ Permanent source changes: build-deb.yml is manual-only and read-only, with
no publication job/input or tag trigger. build_release.sh tags/pushes, then
explicitly starts one matrix; --run-id checks and reuses an exact-source
successful existing build before creating the tag. Publication uses separate
manual publish-release.yml, requiring a tag, accepted=true and build_run_id.
It invokes shared promote_release.py without compilation or build dispatch.

✅ Promotion checks same trusted repository/head repository, build workflow,
successful completed run, exact source SHA, remote lightweight/annotated tag,
six unexpired/nonduplicate matrix artifacts, safe exact files, SHA-256 and Debian
package/version/architecture. Requires clean tagged checkout and finalized
metadata/notes. Only --publish --accepted creates a release; inspection and
pre-tag run-only validation do not publish. Missing/expired artifacts fail closed
with an explicit need for a new accepted build, never silent substitution.

✅ Added tests to just test and Debian test recipes. Eleven promotion test
methods cover positive/negative policy, remote tags, tampering/unsafe paths,
inspection versus approved publication, and fake local release paths showing
one default build dispatch or zero with reused run. Existing publication,
download, installer and helper suites pass. Both workflows parse as manual-only
single-job workflows; Bash syntax and diff checks pass. No Rust runtime changes
or new package build needed for this focused tooling change.

✅ Read-only live check of existing v0.5.3 run 38447948939 confirms reusable
source 346c97bc090991d8f419caa1ebd74875e3516496 and all six artifacts. No
download, build, tag, asset mutation or publication was performed by that check.
AGENTS.md and build guide now require verified artifact reuse. Former 0.5.4
tooling item is [o], transferred to the chronological follow-up.

⏳ New hosted promotion execution remains for the next approved immutable
release, not a repeat 0.5.3 build. Existing v0.5.3 tag predates these workflow
files and will not be moved. Offline tests/live API validation do not establish
the new workflow's hosted publication acceptance.

### Agent guide efficiency rules — October 13, 2026

✅ Added Efficient verification and operator handoff immediately after Working
rules in AGENTS.md: bounded log reading, status-only monitoring, preserved exit
status, receipt reuse, long-command handoff with working directory/log/result
instructions, direct short checks and automation when explicitly requested.
Added rebuild justification at the start of Build, installation, and packaging.
Operator delegation never substitutes for approval or verification responsibility.
Documentation-only diff checks pass; no rebuild or external mutation performed.

⏳ Broader guide reorganization is proposed for discussion, not implemented:
general policy separated from project/API rules; explicit source, Debian and
Flatpak build/install workflows; Git/release authorization and maintained-script
ownership. Policy alone does not change existing scripts' verbose watch output;
bounded-output/script-summary support remains a potential implementation step.

### Agent guide reorganization approved and completed — October 13, 2026

✅ User approved the proposed layout and explicitly deferred script work.
Reorganized AGENTS.md into Part I General policy and Part II Project-specific
workflows, with a guide map. Preserved safety/approval, single-build/evidence,
operator handoff, design, records, engineering-document and release-note rules.
Moved concrete NetworkManager/systemd/portal choices to the COSMIC/API chapter
and language-specific commands to Rust verification; general verification
boundaries remain technology-independent.

✅ Added maintained-script ownership and explicit Git/immutable release gates.
Separated native source, Debian and Flatpak build/install/acceptance instructions
and documented current release scripts/workflows in a task/entry-point/boundary
table. Scripts are preferred over parallel ad hoc procedures; gaps should be
fixed/tested, with approved documented exceptions only when necessary.
Distinguished policy from tools that do not yet mechanically enforce bounded
output. Added ARTIFACTS_DIR to project variables instead of embedding its path
throughout the general policy.

✅ Documentation structure/reference and whitespace checks pass. No rebuild,
script/workflow edit, installation, service or publication change performed.
Next task is script output/summary/operator-handoff improvements, as requested.

### Reusable procedures versus project bindings — October 13, 2026

✅ User requested a more accurate general/project split. Part I now contains
general policy plus technology-specific procedures that apply across projects
when that technology is used: native source, Debian, Flatpak, GitHub Actions
artifact promotion and generic Rust verification. These are not requirements
for every project to use every technology. Part II contains concrete project
bindings: COSMIC/API choices, just recipes, package paths/manifests, shared runtime
behavior and release script/workflow names. Updated guide map/anchors and removed
duplicated procedural text from the bindings while retaining safeguards.

✅ Documentation-only structure/reference and whitespace checks pass. No scripts,
workflows, package builds, installs or releases changed. Script log/summary and
operator-handoff improvements remain the next task.

### Script catalog, inventory and archive review — October 13, 2026

✅ Reviewed nine active scripts and callers across source, tests, recipes,
packaging/workflows and guides. Created scripts/README.md and maintained
docs/engineering/script-tooling-inventory.md with active/update/create categories,
capability gaps, evidence boundaries and prioritized implementation sequence.
All nine active scripts remain necessary, including the installed OneDrive
helper and localization wrapper. Catalog/inventory linked from AGENTS and
engineering index; task list has a dedicated chronological implementation plan.

✅ Nine already archived scripts moved from archive/scripts/ to scripts/archive/
with before/after SHA-256 parity. No current caller references were found. Names
and old/new parents are recorded in archive README/inventory for rollback;
old empty archive/scripts directory removed. They were ignored previously and
remain ignored at the new location; no silent publication of local historical
content. Narrow literal-secret scan negative, not a complete security review.
No active script archived or dependency/runtime behavior changed. Python cache
is generated output, not an archive candidate. No external configuration/data,
installed executable, service or remote resource changed.

⚠ Source review found stale Rust test
tests/debian_packaging_contract.rs::publication_requires_explicit_manual_acceptance
still requiring the removed rebuild-on-publication architecture. This was missed
by the prior Python-focused workflow checks. Recorded as P0 before broader
development/package gates; did not compile merely to reconfirm static assertions.
This inventory task does not implement the proposed tool changes or test fix.

✅ Local documentation references/anchors, inventory coverage and whitespace
checks pass. No installer/authentication/build/release command executed. Next
recommended action is the focused Rust packaging-contract correction, followed
by the agreed logging/receipt interface and existing-tool improvements.

### P0 Rust packaging contract repaired — October 13, 2026

✅ Replaced the stale publication source contract in
tests/debian_packaging_contract.rs with checks for manual build-only execution,
explicit approved tag/run promotion without compilation, shared artifact/metadata
verification, and release preparation that reuses a run without publishing.
Kept the two existing optional-GUI dependency and helper/catalog payload tests.
No production script, workflow, application code or package metadata changed.

✅ cargo test --locked --test debian_packaging_contract passed: 6 tests,
0 failures/ignored. The focused test target compiled in 1.57 seconds; no Debian,
Flatpak or hosted release build ran. Captured full output under
target/p0-packaging-contract.AzPD82/test.log and read only its bounded summary.
Applied just fmt after its check identified wrapping differences; formatting
and whitespace validation complete the source-only repair. Inventory/task P0
marked [x]; hosted promotion acceptance still belongs to the next approved
release. Existing unrelated guide/catalog/archive edits remain preserved.

### P1 output contract and tooling implemented — October 13, 2026

✅ User agreed the output contract and asked to start implementing "P0";
clarified that P0 was already complete and treated the continuation as P1.
Created docs/engineering/script-output-contracts.md before implementation.
Implemented a Python standard-library shared TaskReport under scripts/lib/ for
private unique task directories, logs, atomic versioned summaries, source/run/
artifact identity, real command exit codes and bounded phase/final console output.
Signal handling terminates/reaps local child groups and records cancellation;
it does not cancel remote CI. Raw failures stay private, not automatically
printed. This replaces the initially proposed Bash-only logging helper to avoid
duplicated JSON/state code across Bash and Python entry points.

✅ Stable Bash build/release entry points now delegate fixed tasks to
scripts/lib/release_tasks.py. New dispatch intent gets a durable unique request
ID before any network mutation; build workflow exposes that ID in its title.
Run selection matches request/source/repository/workflow, not newest-run order.
Ambiguous/failed dispatches do not retry; selected-run/receipt resume is explicit.
Structured CI polling defaults to 30 seconds with timeouts/status changes only.
Success precedes download; artifacts/checksums/identities verify in staging,
then publish locally with provenance. Matching cached downloads are reverified
and reused; missing/mismatched legacy receipts are not silently adopted.

✅ Release preparation verifies clean original source, matching GitHub/Git
remote and branch/tag identities; transport failure is not treated as absent
tag. Existing tags require --resume --run-id, never move tags or implicitly
dispatch. Partial tag-push recovery and existing tag/run/cache reuse are tested.
Promotion remains inspection-only unless --publish --accepted; logs/summary and
artifact name/hash/server-origin receipts persist without deleted temp paths.
Hosted promotion retains summary/checksum receipts only, not private raw logs.
No implicit publication retry or asset replacement was introduced.

✅ Focused Python suites passed: 4 output, 8 hosted-build, 15 promotion,
6 publication, 9 installer and 3 helper tests (45 methods, additional subcases).
They cover noisy output, real exit codes, cancelled child cleanup, timeout,
stale/ambiguous/wrong-source runs, dispatch uncertainty, failed downloads,
verified cache reuse/tampering, dry-run and partial/existing-tag resume. All
commands are disposable fake Git/GitHub/package CLIs; no real network mutation.
Logs: target/p1-final-checks.drgroB/. Further receipt-symlink/unrelated-directory
coverage adds a ninth hosted test method, with its focused final result below.
The six Rust Debian packaging source contracts also passed; log:
target/p1-tooling-tests.UXXgx3/rust.log. They now follow the stable entry points
into the shared task driver, rather than requiring removed inline Bash logic.
Formatting, Bash syntax, workflow YAML/manual-only structure and diff checks pass.

⏳ New correlated dispatch, hosted promotion and runner receipt-upload acceptance
remain for the next explicitly approved immutable release. No real CI dispatch,
Git mutation, package build, installation or publication was performed for P1.
Local tests do not establish hosted acceptance. Maintained catalogs/inventory,
guide and task statuses are updated; P2/P3 wrappers/installers remain untouched.

✅ Final focused additions also pass: hosted 9 methods (including refusing
unrelated directories/symbolic receipts), promotion 15 methods (including
receipt-only upload contract), six Rust contracts and shared reporting/other
Python suites above. Aggregate applicable Python coverage is now 46 methods;
no ignored/live/provider tests counted. Final hosted/promotion logs are
target/p1-tooling-tests.UXXgx3/hosted-final.log and promotion-final.log;
42 local links/anchors, manual-only workflow YAML, safe receipt upload paths,
Python syntax, Bash syntax, Rust formatting and git diff --check pass.
Changes remain local/uncommitted along with the preserved earlier guide/catalog
work. Published v0.5.3 and running applet configuration remain untouched.

### P0/P1 source commit and push authorization — October 13, 2026

✅ User authorized committing and pushing the accumulated P0/P1 tooling,
guide, catalog and inventory changes. Reviewed the worktree and manual-only
workflow triggers; reused the focused verification above and confirmed
git diff --check passes. Archived local script contents remain excluded from
version control. This source update does not authorize or require another
build, tag, installation or release publication. P2 remains the next
implementation phase; hosted P1 acceptance remains separately pending.

### Script internal-module catalog clarification — October 13, 2026

✅ Expanded scripts/README.md's brief internal-module paragraph into a dedicated
catalog of both scripts/lib modules, their callers and state-change boundaries.
Updated the engineering inventory with the same layout decision: retain public
entry points at scripts/, shared implementation in lib/, and historical material
in archive/. No further directory migration is warranted at the current size.
P2 retains the planned extraction of shared artifact inspection policy from
promotion. Documentation-only change; local links and git diff --check verified,
with no build, installation, service change or publication.

### P2 shared artifact inspection — October 13, 2026

✅ Began P2 with the shared inspector, preserving the earlier uncommitted script
catalog clarification. Extended the output contract before implementation.
Added scripts/inspect-artifacts.py and scripts/lib/artifact_inspection.py;
promotion's stable verify_packages interface now delegates to the shared policy.
Modes cover the existing six-target Debian matrix, a single Debian package and
native byte identity. Streamed SHA-256, expected package metadata, nonsymbolic
regular inputs and bounded checksum metadata are checked without execution.
Private atomic receipts describe observed paths/bytes, not trusted provenance,
runtime/ABI compatibility, dependencies/payload or installed acceptance.

✅ Six new inspector tests pass, including CLI validation, fake Debian metadata,
native receipts, symbolic/nonregular/missing inputs and failure receipts.
The 15 promotion and nine hosted-download regression tests also pass using fake
CLIs only. Logs: /tmp/mounter-p2-inspection.log, /tmp/mounter-p2-promotion.log,
/tmp/mounter-p2-hosted.log (temporary local logs, not durable build artifacts).
An initial new-test fixture constructor mismatch was fixed before the passing
run. Registered the inspector suite in just test and Debian rules; updated
catalog, inventory, build guide and task statuses. No real package build, GitHub
dispatch, installation, tag or publication. P2 is partial: local Debian wrapper,
fixed dev wrapper and deployment reporting/tests remain outstanding.

### Script-only P2/P3 scope clarified — October 13, 2026

✅ At user request, removed release-number and engine-decision dependencies
from the script inventory. Product architecture and dependency-policy choices
remain in their existing project tasks; no product work was deleted or marked
complete. Script phases now have concrete implementation and exit criteria:
P2 local Debian candidate/provenance/reuse, fixed dev recipes, standalone-safe
opt-in setup reporting, fixtures and handoff; P3 explicit-manifest Flatpak
orchestration, shared metadata inspection and fake-command coverage. Existing
dependencies and service policy stay unchanged during reporting improvements.
Tooling implementation does not claim live build or final package acceptance.
Updated Task List and script catalog consistently. Documentation-only change;
whitespace, removed release-specific inventory references and new section links
checked. No builds, installers, engine changes or release operations performed.

### P2 logged development wrapper — October 13, 2026

✅ Implemented bash scripts/dev.sh with fixed task dispatch in
scripts/lib/dev_tasks.py, reusing TaskReport and existing just recipes. Full
stdout/stderr stay private; console output is phases and a final result/log path.
Receipts identify recipe, HEAD and dirty-state flag without claiming dirty inputs
equal HEAD. Bounded timeout and signal cleanup preserve failure/cancellation;
arbitrary recipe arguments and install/cleanup/launch/release tasks are rejected.
Defined the interface in the output contract before implementation.

✅ Five fixture methods pass, exercising every allowed recipe, noisy output,
real exit codes, invalid tasks/arguments, log-directory preservation, timeout
and SIGTERM child cleanup. Shared reporting (four methods) and inspection (six)
regressions pass. Logs: /tmp/mounter-p2-dev.log, /tmp/mounter-p2-output.log,
/tmp/mounter-p2-inspection.log. Bash/Python syntax and diff checks pass.
Actual bash scripts/dev.sh fmt-check succeeded without compiling the app;
receipt: target/task-runs/dev-fmt-check-gdkhxn7q/summary.json.

✅ Marked the local Debian wrapper [o] per user decision: local work builds
source, Debian packaging uses hosted matrix artifacts; just deb remains an
exceptional diagnostic, not a required release gate. Updated catalog, inventory,
guide and test recipes. No package build, installer, service, Git mutation or
publication performed. Remaining P2: setup reporting/fixtures and final tooling
gate; changes remain uncommitted.

### P2 standalone setup reporting and tooling closure — October 13, 2026

✅ Added optional --log-dir NEW_DIRECTORY to both standalone setup scripts.
Default package/component/service behavior is retained. The embedded, parity-tested
Bash reporter mirrors existing output so prompts remain visible; it logs no
input or command tracing. New directories/logs/reports are private, JSON path
escaping is covered, initial/final receipts are atomic and exit codes preserved.
Existing/symbolic destinations, duplicate/missing arguments and control-character
paths fail before setup. Help stays non-mutating. Signal status is reported,
without claiming transactional rollback or the dev reporter's child-group cleanup.
Logging failures do not turn a failed operation into success. Reporting uses only
ordinary Bash/coreutils, with no Python or repository-relative bootstrap imports.

✅ Thirteen new setup-report fixture methods pass: standalone stdin help and
logged failure, parity, permissions/escaping, default behavior, failed log sinks,
failure/signals/no retry, path rejection, visible input prompts without logging
input, all five OpenConnect package branches through fake adapters, generated
unit/icon/no-start policy, package/reload partial failures, uninstall preservation,
and dependency component/default selection. Existing nine installer tests pass.
No real package manager, gsettings, systemctl, sudo or authentication operation ran.

✅ Final applicable Python gate: 70 methods passed (4 reporting, 6 inspection,
5 dev, 13 setup reporting, 9 dependency installer, 6 publication preflight,
3 helper localization, 9 hosted-download and 15 promotion). Logs are local
/tmp/mounter-p2-final-<suite>.log; the last setup rerun after diagnostic newline
cleanup is /tmp/mounter-p2-setup.log. Bash/Python syntax and git diff --check pass.
ShellCheck is not installed; no ShellCheck claim. Test recipes, help, setup guides,
catalog, contracts and inventory are updated. P2 marked complete for source and
fixtures only; local Debian wrapper remains [o], P3 remains pending. Separate
live/hosted package acceptance is not inferred. Changes remain uncommitted;
no builds, installation, service mutation, tag or release publication performed.

### P3.1 explicit-manifest candidate build/export — October 13, 2026

✅ Defined the build/export contract before implementation. Added
scripts/build-flatpak.sh and scripts/lib/flatpak_tasks.py with explicit JSON
manifest, expected clean packaging-checkout SHA, architecture, branch and new
output destination. Referenced inputs must be tracked, within that checkout and
nonsymbolic; Git sources require full commit pins and remote archive/file sources
require SHA-256. Includes/local files are fingerprinted and rechecked after build.
No mutable-tag substitution, source generation, force-clean, installation, app
launch or publication. Packaging-checkout identity is not confused with separately
pinned application/dependency source commits.

✅ Required runtime/SDK/BaseApp identities and installed commits are observed
read-only. The fixed sequence builds, exports a local repository and creates a
bundle; tool versions, declared finish-args, source pins, built metadata snapshot
and bundle hashes go into candidate receipts. Missing tools/runtimes or mismatched
built identities fail closed. Partial output is retained on failure, never silently
removed/retried. Output must be new/outside the source checkout and separate from
the report directory. Existing-output reuse is deliberately not yet supported;
P3.2 shared export inspection and P3.3 verified reuse remain outstanding.

✅ Eight fixture methods pass, covering successful build/export/bundle receipts,
immutable pins and JSON includes, dirty/wrong/untracked/escaping/symbolic inputs,
branch/runtime/app identity mismatches, missing runtime, build/export/bundle and
input-mutation failures, existing-output preservation, rejected unsafe flags,
timeout and cancellation/child cleanup. An initial lexical containment bug was
caught by the path-escape fixture and fixed using canonical paths before the
passing run. Log: /tmp/mounter-p3-flatpak.log. Four shared reporter regressions
also pass (/tmp/mounter-p3-output.log). Bash/Python syntax, wrapper help and
git diff --check pass. Registered fixtures in just/Debian tests and updated
catalog, inventory, contracts, guide and task statuses.

Only real Flatpak/builder help was inspected; no real build/export, network,
runtime installation, app install or Git/release operation ran. The existing
source manifest still has a mutable tag and must not be claimed compatible
without an accepted commit pin. SDK-extension resolution and final exported
permissions/repository integrity, live sandbox behavior and shipping acceptance
are not established by these fixtures. Changes remain uncommitted.

### P3.2 shared Flatpak artifact inspection — October 13, 2026

✅ Defined the metadata comparison contract, then added
scripts/lib/flatpak_inspection.py and flatpak-repo/flatpak-bundle modes to the
existing inspector. Require explicit app/architecture/branch and expected
metadata, with optional OSTree commit pin. Compare all metadata sections and
keys, not a permission subset: runtime/SDK, Context, bus policy, Environment and
extension settings. Reject malformed/duplicate/default/continued keyfiles,
unsafe refs, symbolic inputs, ref drift, changed baselines or changed bundles.
Record canonical metadata fingerprints separately from raw input hashes.
Bundle import occurs only in a private temporary repository and is cleaned up;
no app/runtime installation, configured remote or app launch is used.

✅ Build/export now invokes the shared policy for both exported repository and
bundle, requiring matching commits and built/exported metadata before success.
Its built metadata is a consistency baseline, not independent permission
approval. The wrapper records ostree tool identity and private inspection
details. Updated fake export fixtures cover exported policy mismatch; test
recipes, catalog, inventory, packaging guide and task status are updated.

✅ Eight new inspection methods pass (including policy/runtime/identity/ref,
bundle/import/input failure, malformed keyfile, privacy and path cases); eight
updated build/export and six existing Debian/native inspection tests pass.
Logs: /tmp/mounter-p3-inspection.log,
/tmp/mounter-p3-inspection-wrapper.log, /tmp/mounter-p3-inspection-existing.log.
Python syntax and git diff --check pass.

✅ Also verified the real OSTree read and Flatpak bundle-import interfaces with
a tiny synthetic metadata-only repository/bundle under
/tmp/mounter-p3-metadata.9Io4Fm/ (not a Cloud Mounter build). Repository inspection
passed in the managed sandbox. Bundle import first failed with read-only
O_TMPFILE access; the approved unrestricted rerun passed. Receipts:
repo-report/summary.json and bundle-report-unrestricted/summary.json there.
No installed application/runtime or remote was altered. Payload integrity,
signatures, installed/dynamic overrides, trusted provenance and store acceptance
are outside this metadata check. P3.2 is complete at the stated boundary;
P3.3 verified reuse/final handoff remains pending. Changes remain uncommitted.

### P3.3 verified reuse and tooling closure — October 13, 2026

✅ Defined explicit --reuse-receipt behavior before implementation. Candidate
receipts now use schema 2 (task summaries remain schema 1), recording manifest
relative path, canonical output and build options for exact comparison. Reuse
requires the prior successful producer summary and checks selected clean source,
manifest/include/local-file hashes, source pins, app/branch/architecture, exact
bundle/metadata paths and hashes, recorded runtime/tool provenance and build
options. It rechecks source, artifacts, baseline and exported ref before success.
Legacy/incomplete/failed receipts, corruption and mismatches fail closed.

✅ Reuse performs read-only ostree fsck without repair/delete flags and shared
repository/bundle commit/full-metadata inspection. Symbolic repository roots are
rejected before fsck. No builder invocation (including --version), runtime info,
build-export or build-bundle occurs on reuse. Original build tool/runtime records
are retained; current verification tool versions and prior receipt path/hash are
recorded separately in a new private report. Original receipts/candidates are not
overwritten. Local receipts remain trusted comparison records, not signatures.
No automatic retry, rebuild, cleanup, installation or publication was introduced.

✅ Seven reuse fixture methods pass: successful/chained reuse without builder
or runtimes; bundle/metadata/repository/commit tampering, missing artifacts and
fsck failure; malformed/legacy/incomplete/failed/symbolic producer records;
source/options mismatch; mid-verification mutation and symbolic repository;
partial-build refusal with a separately selected new-output build; cancellation
with child cleanup and original preservation. Final gate: 93 Python methods pass
across all 12 applicable tooling suites. Logs:
/tmp/mounter-p3-final-<suite>.log, including flatpak_reuse, flatpak_wrapper,
flatpak_inspection and the existing P1/P2 regressions. Bash/Python syntax,
wrapper help and git diff --check pass. Real ostree fsck also passed on the
earlier synthetic metadata-only repository /tmp/mounter-p3-metadata.9Io4Fm/repo.
No Cloud Mounter compilation, package matrix, install or runtime/service change
was performed. Previous synthetic bundle-import evidence remains separate.

Updated setup/build recipes, agent binding, script catalog, inventory, contracts,
packaging handoff/recovery instructions and task markers. P0–P3 script source/
fixture work is complete; local Debian wrapper remains [o]. Actual pinned-manifest
build, installed sandbox acceptance, next hosted release acceptance and any store
submission remain distinct project gates. Accumulated changes remain uncommitted.

### Script-tooling source commit/push authorized — October 13, 2026

✅ User approved committing and pushing the accumulated P2/P3 source, tests and
documentation. Reviewed the intended worktree and reused the recorded 93-test
gate for unchanged code; whitespace checks pass. Package workflows remain
manual-only. Stage only maintained files, not private reports, synthetic fixtures
under /tmp, packages, caches or ignored archive contents. This source update
does not dispatch a build, move a release tag, install or publish a package.
