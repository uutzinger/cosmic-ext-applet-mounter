# Cloud Mounter Applet Task List Completion Notes

Dated execution evidence for the active [Task List](Task%20List.md). Entries through
the native v0.5.3 publication are archived, unchanged, in
[docs/archive/task-list-completion-notes-through-v0.5.3.md](docs/archive/task-list-completion-notes-through-v0.5.3.md).

Entries through v0.5.4-pre: [archive](docs/archive/task-list-completion-notes-through-v0.5.4-pre.md).

### OneDrive Offline engine decision — October 14, 2026

✅ User decided to drop `abraunegg/onedrive`: no clear authentication path covers
both personal and corporate accounts, and the employer does not support its
Microsoft Graph access requirements. OneDrive Offline mirrors will use `rclone
bisync` through the existing SharePoint (Teams) coordinator; `jstaf/onedriver` stays
for OneDrive Online, and no other mount engine changes. Continuous/event-driven
mirroring is not required; abraunegg already polled the remote every 300 seconds
(`--monitor-interval 300`, `src/sync.rs`), so scheduled bisync plus Sync Now keeps
automatic syncing and loses only immediate local-change upload and delta efficiency.

✅ Rewrote the Release 0.5.4 engine section in `Task List.md` as a recorded decision
plus three open decisions (legacy timeline, which SharePoint guards apply, Microsoft
app registration) and six phases: contract/requirements, disposable-account
prototype, implementation, legacy migration, removal, acceptance. Source touchpoints
were identified read-only (`RcloneBackend::try_from(Provider)` rejects OneDrive;
`one_drive_*` in `src/sync.rs`; the auth UI in `src/app.rs`; `debian/control`
Recommends `onedrive`, `python3-gi`, `gir1.2-webkit2-4.1`).

⏳ No source, packaging, requirements or Description change, authentication test,
migration or installation occurred. `Applet Description.md` and `Requirements and
Specifications.md` still describe abraunegg until phase 1 updates them.

✅ October 14, 2026 (same decision): legacy timeline chosen. Existing abraunegg
connections keep running but cannot be created for one release, then the engine is
removed. No conversion tool is planned because the applet has no established user
base; users recreate OneDrive Offline mirrors as rclone connections. Task List phase 4
now covers a deprecation notice, an overlap guard so two engines never share a tree,
and manual replacement documentation; the conversion flow and rehearsal were removed.

✅ October 14, 2026: mirror-guard approach agreed for the rclone OneDrive Offline
mirror. The SharePoint guards (identity guard, UUID-bound access marker, Office-rewrite
flags with convergence pass, checksum verification; see `src/sync.rs` and
`src/bisync_coordinator.rs`) came from SharePoint tests and are not assumed to apply
to OneDrive. Task List phase 2 now has a per-drive-type Office-rewrite test and guard
tests; the open decision lists the baseline to confirm or reject by test. Initial
Preview plus confirmed initial sync, recovery copies and `--max-delete` are already
generic. No code change or test was run.

✅ October 14, 2026: user decided against Microsoft app registration for the rclone
OneDrive Offline mirror. Custom app registration does not work for the corporate
storage, and the SharePoint rclone mirror already works with rclone's default client,
so the OneDrive editor will have no client ID/secret/tenant fields. Accepted risk: a
tenant that blocks rclone's shared client cannot use OneDrive Offline. Task List
open decision closed; the custom-client mentions in phase 3, phase 4 and the
post-0.5.4 backlog were removed. `docs/microsoft-cloud-auth-setup.md` is unchanged.

✅ October 14, 2026: closed the mirror-guard design decision as a rule (safety guards
always on; compatibility workarounds only where a per-drive-type test shows the
instability and the fix; checksum verification by cadence) with an empty
per-drive-type profile table to fill from test results. Added the operator-run live
check `scripts/onedrive-mirror-guard-check.sh` (lib/mirror_guard_check.py): one
uniquely named test folder on a named rclone OneDrive remote, generic versus
compatibility bisync profiles compared by stability of a no-edit second sync,
optional web-edit pause, and purge restricted to that folder after its marker is
confirmed. Verified only against a fake rclone (`tests/mirror_guard_check_test.py`,
5 methods); no live run, authentication or cloud change occurred.

✅ October 14, 2026: Task List engine section reordered. The live mirror guard check
is now its own phase 2, directly after the contract draft; its results complete the
per-drive-type profile table and feed the contracts record. The prototype, implementation,
legacy, removal and acceptance phases are renumbered 3–7. The guard tests that need
implemented guards moved to acceptance, and the release-notes task no longer
mentions Microsoft app registration.

✅ October 14, 2026: Task List phase 1 (contract and requirements) completed as design
documents only. Added `docs/engineering/onedrive-rclone-offline-contracts.md`
(configuration with `engine` and `onedrive_identity`, setup and identity confirmation,
guards and the pending per-drive-type profile table, scheduling/status/Repair, legacy
handling, failure table, acceptance matrix, open items) and indexed it. Updated `Applet
Description.md` and `Requirements and Specifications.md` to the target state: OneDrive
Offline on `rclone bisync`, FR-031/031A/031B/053/060C revised, new FR-031C for legacy
connections, dependency and module tables, setup text and the SharePoint Offline
paragraph; the 0.5.3 delivery paragraph keeps its history with a pointer to the 0.5.4
decision. README, dependency installation guide and Flatpak packaging guide carry
"planned change" notes that do not claim shipped behavior. `docs/index.md` was not
changed. No source, packaging, test or live run occurred; the compatibility profiles
and verification cadence remain pending the live guard check.

✅ October 15, 2026: phase 2 Prepare completed. The operator created
`onedrive_business` and `onedrive_personal` with rclone v1.75.0. A read-only
configuration check confirmed backend `onedrive`, respective `business` and
`personal` drive types, populated drive IDs and default clients (no custom client
ID/secret); exit 0. No credentials or drive IDs were printed by that check.
Read-only `rclone lsjson --stat` checks of the operator-created test directories
initially failed because sandbox network access was blocked; the authorized
network retry returned directory metadata for both, with final exit 0.

The operator subsequently chose the script-owned `CloudMounter-GuardTest-<token>`
folders instead of those manually created directories. Only script-owned test
trees will be synced; neither whole drive will be mirrored. The manually created
directories remain untouched.

Read-only Python inspection of `~/Cloud/OneDrive_Mirrors/test_folder` found 12
regular files totaling 2,797,198 bytes (below 20 files / 5 MB): four DOCX, three
XLSX, two TXT, one PDF, one ZIP and one PPTX. Required OOXML parts and ZIP CRCs
passed for DOCX/XLSX; TXT decoded as UTF-8. Additional ZIP/PPTX integrity and PDF
signature checks passed. Both inspection commands exited 0. The script copies
all supplied regular files, including the extra formats. This establishes sample
format/size readiness, not rendered Office validity or a content privacy audit.

Files changed for this preparation record: Task List.md and Task List Completion
Notes.md. No Rust or script changes, transfers, bisync, service changes or
installation occurred. The live guard phase remains open; next step is the
maintained `--plan` check for each remote, then the operator-run live checks.

✅ October 15, 2026: both operator-run remote plans passed. Commands:
`bash scripts/onedrive-mirror-guard-check.sh --remote onedrive_personal --drive personal --plan`
and `bash scripts/onedrive-mirror-guard-check.sh --remote onedrive_business --drive business --plan`.
Both supplied receipts report `status=succeeded`, `exit_code=0` and
`plan-complete`; `source_sha` and `source_dirty` are null (no source provenance
claim). Personal receipt/log:
`target/task-runs/onedrive-guard-check-5kmv8oh7/{summary.json,task.log}`;
business receipt/log:
`target/task-runs/onedrive-guard-check-axs3wbef/{summary.json,task.log}`.
These plans check configured backend presence only; drive-type configuration and
live directory access were checked separately during Prepare. No cloud mutations
or live bisync acceptance are implied. Updated the task checkbox and these notes;
next step is the personal live run with the prepared files and `--web-edit`,
followed by the business run. Compatibility profiles remain pending.

⏳ October 15, 2026: operator requested agent execution of the business live
guard check with web-edit handoff. First run failed before the prompt, exit 3:
`target/task-runs/onedrive-guard-check-xkb3o5wt/task.log`. Initial bisync could not
read the nonexistent cloud test directory and reported zero transferred bytes;
subsequent observation failed. This is a script setup defect, not a compatibility
verdict. Added explicit bounded `rclone mkdir` for the generated test directory
before initial bisync. Updated fake rclone to reject absent directories and added
a creation-failure test that forbids sync/purge. Updated script catalog and inventory.
`python3 tests/mirror_guard_check_test.py`: six tests passed, exit 0.
`bash scripts/dev.sh gate`: formatting and lint passed, then sandbox runtime-dir
access blocked just before tests, exit 1; receipt/log in
`target/task-runs/dev-gate-fsuj44fj/`, HEAD
`7d78d2ef0c9e95965658cbc15d104588eb6d7650`, source_dirty=true.
Authorized `bash scripts/dev.sh test` rerun is ongoing in
`target/task-runs/dev-test-nyaxxrkx/`; passing formatting/lint were not repeated.
`git diff --check` passed. The restarted business live run reached its generic
web-edit prompt; waiting for operator edit before sending Enter. No profile
acceptance yet. Changes: script module, its fixture tests, script catalog,
tooling inventory, task list and these completion notes.

✅ October 15, 2026: business live retry completed, exit 0. Receipt, log and
`results.json`: `target/task-runs/onedrive-guard-check-rsx8nj5g/`. Generic profile:
initial, no-edit, web-edit and post-edit bisync exits all 0; snapshots stable both
without edits and after the web edit. One spreadsheet changed locally after the
operator saved the web edit. Final check: 15 equal, 0 different, 0 local-only,
0 remote-only; remote hash type quickxor. Cleanup reports purged. Compatibility
retry was not needed. Recorded generic for Business in the task profile table and
contract; evidence applies to this drive and sample set, not all possible files
or installed applet behavior. Personal live check remains pending.

✅ Broader verification completed: `bash scripts/dev.sh test`, exit 0,
`target/task-runs/dev-test-nyaxxrkx/{summary.json,task.log}`, source SHA
`7d78d2ef0c9e95965658cbc15d104588eb6d7650`, source_dirty=true. Combined with earlier
passing formatting/lint, this completes the routine gate sections after the
sandbox interruption. No installation or package acceptance claimed. Next step:
personal live guard check with the same sample directory and web-edit handoff.

⏳ October 15, 2026: authorized personal live check started with
`bash scripts/onedrive-mirror-guard-check.sh --remote onedrive_personal --drive personal --files-dir /home/uutzinger/Cloud/OneDrive_Mirrors/test_folder --web-edit`.
Reached the generic web-edit prompt for `CloudMounter-GuardTest-f403f8097adc`.
Waiting for operator save confirmation before sending Enter; no personal verdict
or completed cleanup yet. Original sample directory is only the copy source.


✅ October 15, 2026: personal live guard check completed after the operator's
save confirmation, exit 0. Receipt, log and results.json are in
`target/task-runs/onedrive-guard-check-qmyfk3fx/`. Initial, no-edit, web-edit and
post-edit bisync exits were all 0. The edited spreadsheet changed locally;
no-edit and post-edit snapshots were stable. Final check: 15 equal, 0 different,
0 local-only, 0 remote-only; quickxor hashes. No issues; cleanup reports purged.
Generic settings are sufficient for the stages and samples tested; no compatibility
retry was needed. Original sample files were only copied as inputs.

Both per-drive-type profiles are now recorded as generic in the task list and
contract. Phase 2 checklist is complete; engineering index/inventory updated.
This does not establish that the initial upload preserved original bytes: the
stability baseline is after initial sync. Nor does it validate implemented
identity/access guards, installed services, UI or Flatpak behavior. Those remain
in later phases. Personal run receipts have null source_sha/source_dirty; the
script fix's source/dirty provenance and passing gate sections are recorded above.
No code changes since those checks; no repeat gate needed for these evidence-only
updates. `git diff --check` passed (exit 0). Next task is phase 3, the disposable
account prototype; no prototype execution or installation was started here.


⏳ October 15, 2026: added phase 2a for the missing initial-upload round-trip
and unnecessary-transfer investigation. Extend the maintained guard script;
no ad hoc cloud workflow. Preserve originals, independently download and compare
bytes, distinguish reported metadata from actual content, and measure transfers
rather than inferring them from stable snapshots. Add fixtures before live runs.
Earlier phase 2 results remain valid within their measured scope; the earlier
completion statement does not close this newly explicit follow-up. Both profiles
are provisional in the task list and contract until phase 2a passes. Phase 3 waits.
Files changed: Task List.md, this completion record and the OneDrive contract.
Planning/documentation only: no script changes or live operations in this update.
`git diff --check` passed, exit 0. Next step: implement the script extension and
focused fixtures, then the routine gate before the new live comparisons.


✅ October 15, 2026: phase 2a script extension implemented and verified locally.
The existing wrapper accepts `--round-trip` and bounded `--settle-seconds` (default
10, range 0..60); legacy invocations retain their prior schema and behavior.
New `scripts/lib/mirror_guard_roundtrip.py` preserves one baseline, records original
SHA-256/byte lengths/timestamps/QuickXorHash, compares immediate and delayed remote
metadata with independent downloads, and uses a separate fresh bisync tree.
Two no-edit syncs record structured transfer counts/bytes; diagnostic downloads
are separate. Requested web edits must produce an observed local byte change and
settle. Initial local changes, byte differences, absent evidence, command failures
and cleanup problems cannot silently pass. A compatibility retry includes one
explicit convergence pass but remains a candidate requiring settings minimization;
no automatic claim that the full workaround bundle is necessary.

Schema-2 results retain checkpoints on failure/cancellation, source HEAD/dirty
state, implementation hashes, structured rclone version, effective profile flags,
observation timing, manifests, counters and cleanup outcomes. Marker content is
verified before deletion; cancellation reaps the active child before bounded
cleanup. Original sample snapshots must remain equal. Sample symlinks/subdirectories
are rejected before cloud writes; generated plain/binary files no longer overwrite
same-named sample copies. No real samples or cloud trees were modified this turn.

Changed files: `scripts/lib/mirror_guard_check.py`, new
`scripts/lib/mirror_guard_roundtrip.py`, `tests/mirror_guard_check_test.py`, new
`tests/mirror_guard_roundtrip_test.py`, `scripts/README.md`,
`docs/engineering/script-tooling-inventory.md`,
`docs/engineering/script-output-contracts.md`, `docs/engineering/README.md`,
Task List.md and this completion record. Existing unrelated edits were preserved.

Verification commands (all exit 0):
- `python3 tests/mirror_guard_check_test.py`: 6 tests.
- `python3 tests/mirror_guard_roundtrip_test.py`: 13 tests, including fake byte
  rewrites, reported-size discrepancies, redundant transfers with unchanged
  snapshots, missing evidence, observed/unobserved web edits, cancellation,
  partial cleanup and unchanged original samples. Installed rclone also exercised
  copy and bisync JSON counters and local QuickXorHash in temporary directories
  with an isolated empty config; no network or user services involved.
- `python3 -m py_compile scripts/lib/mirror_guard_check.py scripts/lib/mirror_guard_roundtrip.py tests/mirror_guard_check_test.py tests/mirror_guard_roundtrip_test.py`;
  wrapper `--help`; `git diff --check`.
- `bash scripts/dev.sh gate`: formatting, lint and full automated tests passed.
  Receipt/log: `target/task-runs/dev-gate-rmvz9p7w/{summary.json,task.log}`;
  source SHA `7d78d2ef0c9e95965658cbc15d104588eb6d7650`, source_dirty=true.
  Gate used authorized host access for just's runtime directory. No live OneDrive
  operations, app installation or final-package acceptance are implied.

⏳ Remaining: extended personal/business live runs and conditional settings
minimization if generic fails. Observations cover only the configured window;
provider rewrites occurring later remain outside measured evidence. Both profile
choices remain provisional. Next command is the maintained guard wrapper with
`--round-trip --web-edit`, the selected remote/drive and the prepared sample path;
use `--plan` first to inspect its scope.


⏳ October 15, 2026: first operator-run extended personal check failed before
cloud setup, exit 2, because the new provenance code invoked unsupported
`rclone version --json`. Receipt/log/results:
`target/task-runs/onedrive-guard-check-3p6md0gs/`. Partial results have empty folders
and commands arrays: no cloud folder creation or transfer occurred. The fake had
incorrectly accepted that option, and the real-rclone smoke exercised transfers
but not version recording; earlier test success did not cover this capability.

Fixed provenance to capture plain `rclone version` output as opaque text in a
`{format: "text", output: ...}` envelope, without parsing text for behavior or
capability decisions. The fake now rejects extra version arguments. The installed
rclone smoke calls the exact production version helper, and the fake integration
asserts the resulting evidence. Updated catalog, inventory and output contract.
Files: mirror_guard_roundtrip.py, both mirror-guard Python test files, scripts
README, script-tooling inventory, output contract, task list and completion notes.
`python3 tests/mirror_guard_roundtrip_test.py`: 13 passed, exit 0, including the
installed-rclone version regression. `git diff --check`: exit 0. Broader
`bash scripts/dev.sh test` in progress at `target/task-runs/dev-test-8odrhl5_/`.
Earlier passing formatting/lint from `dev-gate-rmvz9p7w` retain unchanged Rust
inputs and are not repeated. No cloud retry performed by the agent; operator
will rerun the same maintained command after verification.

### Mid-release archive of completed history — October 15, 2026

✅ Manual exception to the archive-after-publication rule, approved by the
operator, to reduce the context agents read each session. No release was published.

- Scope: task-list sections before "Release 0.5.4" and completion-note entries
  before "OneDrive Offline engine decision" moved unchanged to
  `docs/archive/*-through-v0.5.4-pre.md`. `0.5.4-pre` is a label, not a release.
- Script change: `scripts/lib/archive_tasks.py` now accepts an optional
  `-label` suffix on the version (commit f4b0cb1; five archive tests passed).
  Dry run reported 89 task lines, 455 note lines and 1 open item carried forward;
  the real run (`archive-release` receipt `archive-release-ned27ekd`) succeeded.
- Result: notes 48.6 KB to 19.2 KB; task list 60 KB to 54 KB. Open item carried
  to "Archive and open items carried forward".
- Follow-up: when 0.5.4 is published, archive the remainder under the real
  version `0.5.4`; the `-pre` archives stay as historical records.
