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


### OneDrive round-trip and measured-sync acceptance — October 15, 2026

✅ Operator completed both extended live runs, each with exit 0, and the agent
read their saved schema-2 results and summary receipts. Command for each:
`bash scripts/onedrive-mirror-guard-check.sh --remote NAME --drive TYPE --files-dir "$HOME/Cloud/OneDrive_Mirrors/test_folder" --round-trip --web-edit`.
Personal used `onedrive_personal`/`personal`; business used
`onedrive_business`/`business`. Evidence directories (each contains results.json,
summary.json, task.log and per-stage structured transfer logs):

| Drive | Evidence directory | No-edit 1 | No-edit 2 | Web edit | Post-edit |
|---|---|---|---|---|---|
| Personal | `target/task-runs/onedrive-guard-check-8zm9owl_/` | 0 files / 0 bytes | 0 files / 0 bytes | 1 file / 15,677 bytes | 0 files / 0 bytes |
| Business | `target/task-runs/onedrive-guard-check-xpn4ex9d/` | 0 files / 0 bytes | 0 files / 0 bytes | 1 file / 15,646 bytes | 0 files / 0 bytes |

Both results: status completed, generic profile clean, no issues. Two independent
round-trip downloads of all 15 files (12 samples, two generated controls and an
ownership marker) had no missing/extra files, no actual byte-length/SHA-256
changes, no QuickXorHash differences and no reported-size discrepancies against
either originals or downloaded bytes. Initial bisync made no local byte changes.
Before/after-edit verification downloads matched their corresponding local
mirrors. Exactly one spreadsheet edit was observed per drive; both subsequent
syncs were stable. Diagnostic download counters are separate from bisync totals.

Timestamp comparison records differences for all 15 entries because it compares
strings; inspection of parsed instants shows only timezone representation and
subsecond precision differences (remote whole-second times, less than one second
from local values). Remote metadata stayed stable over both observations, and
all idle syncs reported zero transfers. These observations do not justify adding
the SharePoint timestamp/size/checksum workarounds. Source samples remained
unchanged, both owned cloud folders per run were purged, and no compatibility
candidate was needed. Generic `--compare size,modtime` is confirmed for both drive
types within these samples and the configured 10-second observation interval.
Phase 2a compatibility minimization is deliberately skipped as its trigger did
not occur; the live-run checklist is completed.

Provenance: both receipts record source SHA
`7d78d2ef0c9e95965658cbc15d104588eb6d7650`, source_dirty=true; each results file
also retains implementation hashes and rclone v1.75.0 text provenance. This is
live-engine/sample-tree acceptance, not proof of a clean committed revision or
installed applet, systemd, UI or Flatpak behavior. Delayed service rewrites beyond
the measured window remain outside the evidence. Prior phase 2 receipts and the
failed first schema-2 attempt remain historical records.

✅ Version-fix broader tests also completed: `bash scripts/dev.sh test`, exit 0,
`target/task-runs/dev-test-8odrhl5_/{summary.json,task.log}`, same source SHA and
dirty flag. This closes the previously pending verification entry above.

Updated Task List.md, this completion record, the OneDrive profile contract,
engineering index and script-tooling inventory. Moved the operator's pasted
console output verbatim from the active task list to the evidence below.
Verification this turn: read-only receipt assertions and `git diff --check`,
both exit 0. No code changed or live test repeated; earlier applicable checks
were retained. Next step is phase 3 disposable-account prototype and minimum
rclone/feature confirmation; safety guards and installed acceptance remain open.

Operator console output (preserved from the task list):

```text
uutzinger@urslabtop:~/Build/cosmic-ext-applet-mounter$ bash scripts/onedrive-mirror-guard-check.sh   --remote onedrive_personal   --drive personal   --files-dir "$HOME/Cloud/OneDrive_Mirrors/test_folder"   --round-trip   --web-edit
onedrive-guard-check: roundtrip-upload
onedrive-guard-check: roundtrip-download-0
onedrive-guard-check: roundtrip-download-1
onedrive-guard-check: generic-initial
onedrive-guard-check: generic-no-edit-1
onedrive-guard-check: generic-no-edit-2
onedrive-guard-check: generic-verification

Open CloudMounter-GuardTest-5ed5484e249fb4d9b65974043e2ecb54/ on the drive in the web editor, change and save one Office file, then press Enter.

onedrive-guard-check: generic-web-edit
onedrive-guard-check: generic-post-edit
onedrive-guard-check: generic-post-edit-verification
verdict (personal): generic passed round-trip and measured sync stages
results: /home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-8zm9owl_/results.json
onedrive-guard-check: check-complete
{"task": "onedrive-guard-check", "status": "succeeded", "exit_code": 0, "source_sha": "7d78d2ef0c9e95965658cbc15d104588eb6d7650", "source_dirty": true, "run_id": null, "summary_path": "/home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-8zm9owl_/summary.json", "log_path": "/home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-8zm9owl_/task.log", "artifact_receipt_path": null, "next_action": "Inspect the task log if this operation does not complete."}
uutzinger@urslabtop:~/Build/cosmic-ext-applet-mounter$


uutzinger@urslabtop:~/Build/cosmic-ext-applet-mounter$ bash scripts/onedrive-mirror-guard-check.sh   --remote onedrive_business   --drive business   --files-dir "$HOME/Cloud/OneDrive_Mirrors/test_folder"   --round-trip   --web-edit
onedrive-guard-check: roundtrip-upload
onedrive-guard-check: roundtrip-download-0
onedrive-guard-check: roundtrip-download-1
onedrive-guard-check: generic-initial
onedrive-guard-check: generic-no-edit-1
onedrive-guard-check: generic-no-edit-2
onedrive-guard-check: generic-verification

Open CloudMounter-GuardTest-5887ffe0a0abbcaf9d742291943f3169/ on the drive in the web editor, change and save one Office file, then press Enter.

onedrive-guard-check: generic-web-edit
onedrive-guard-check: generic-post-edit
onedrive-guard-check: generic-post-edit-verification
verdict (business): generic passed round-trip and measured sync stages
results: /home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-xpn4ex9d/results.json
onedrive-guard-check: check-complete
{"task": "onedrive-guard-check", "status": "succeeded", "exit_code": 0, "source_sha": "7d78d2ef0c9e95965658cbc15d104588eb6d7650", "source_dirty": true, "run_id": null, "summary_path": "/home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-xpn4ex9d/summary.json", "log_path": "/home/uutzinger/Build/cosmic-ext-applet-mounter/target/task-runs/onedrive-guard-check-xpn4ex9d/task.log", "artifact_receipt_path": null, "next_action": "Inspect the task log if this operation does not complete."}
```


### OneDrive phase-3 prototype implementation — October 15, 2026

⏳ Implemented an isolated development harness, following operator steering to
model OneDrive after the existing SharePoint mirror. Refactored the existing
planner and coordinator transaction bodies into shared private helpers. Default
production routing/validation is unchanged; a development-probes-gated entry
point builds the OneDrive generic plan and invokes the same Preview/initial-sync
markers, operation lock, UUID access marker, recovery, retention and cancellation
transaction as SharePoint. SharePoint retains its existing compatibility flags,
convergence and verification; tests explicitly check that separation.

New `src/onedrive_prototype.rs` uses the existing bounded CommandRunner/native
and FlatpakHostCommandRunner transports plus the existing Microsoft token
extractor. In-process bounded Graph requests verify the expected sign-in account,
primary drive and personal/business type, rejecting cache/document libraries,
custom clients/roots and changed identity. Only identity fingerprints enter
receipts. Browser OAuth creates only proven-new cm_proto_ remotes with the default
client and bounded create/continue; ordinary failure rolls back that new remote.
Existing remotes are not overwritten. Credential dumps, tokens, raw provider
errors and account names are not logged. Cargo is quiet to suppress echoed account
arguments; Graph HTTP failures retain only their numeric status.

The fixed-action shell/Python driver invokes the gated Rust example. Prepare
creates only its CloudMounter-Prototype-UUID tree, root ownership marker and
synthetic fixtures, and stops after shared initial Preview. Separate initial-sync
is explicit confirmation. Scenario covers changes both sides, deletes, both
conflict versions, 16 MiB transfer, independent download verification and runtime
measurement. Benchmark compares sync with sync plus full-tree check --checksum
against a 300-second budget, labelled synthetic/non-representative. Fault actions
inject an unavailable network bind or throttle/cancel an active bisync; results
require both the expected typed child failure and the observed coordinator state.
Repair inspects and preserves pending data and locks. Explicit cleanup verifies
identity/root ownership and purges only the UUID cloud root, retaining local data.
Each CLI session action also holds a shared operation guard. No timers, services,
mounts, applet configuration, installation or production routing were changed.

Failure/fix chronology: initial local engine scenarios passed but cancellation
was not injected because the shared coordinator prepends global timeout arguments.
The first and second local lifecycle attempts failed with VerificationFailed at
the cancellation assertion (exit 101); neither was treated as acceptance. Fixed
matching of the typed bisync argument, forced buffered copy during the throttled
fault probe, and added a regression covering the shared option prefix. The next
local lifecycle passed. Tightened fault acceptance to require the expected child
error itself, preventing unrelated preflight failure from passing a fault test.

Final focused verification (exit 0):
- `bash scripts/dev.sh check`: `target/task-runs/dev-check-4jnm5urv/` receipt/log;
  source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
- `cargo test --locked --lib onedrive_prototype -- --include-ignored`: 10 passed,
  including the explicitly run installed-rclone local alias lifecycle. Log:
  `target/onedrive-prototype-focused.log`. The alias test uses synthetic Graph
  and isolated temporary config/data, not real OneDrive accounts or network.
  It verified shared initial Preview/sync, both-side changes/deletes, conflicts,
  large copy, verification, cancellation, Repair/pending-data preservation and
  marker-guarded cleanup. This is local engine integration only.
- `python3 tests/onedrive_prototype_tasks_test.py`: 2 passed using fake cargo/git;
  fixed-action/session validation, receipts and account exclusion from logs.
- Wrapper --help, bash -n, Python py_compile and git diff --check passed.

First final gate: `bash scripts/dev.sh gate`, exit 101, receipt/log
`target/task-runs/dev-gate-bok_92ud/`. Formatting passed; Clippy found one
collapsible if in the example's receipt writer. Fixed it; feature-specific
`cargo check --locked --features development-probes --example onedrive_mirror_prototype`
passed. Focused Rust units rerun: 9 passed, 1 ignored (the unchanged local engine
integration was not repeated), exit 0; log
`target/onedrive-prototype-focused-unit.log`. Gate retry is in progress at
`target/task-runs/dev-gate-5t4id0ep/` with authorized runtime/local-test access.

Updated source/tooling: Cargo.toml, src/lib.rs, src/sync.rs,
src/bisync_coordinator.rs, new src/onedrive_prototype.rs, new gated example,
new scripts/onedrive-mirror-prototype.sh and lib/onedrive_prototype_tasks.py,
new tests/onedrive_prototype_tasks_test.py. Documentation: task list/these notes,
script catalog/inventory, output contract, host-command inventory, OneDrive
contract and engineering index. Existing documentation edits from earlier work
were preserved. The host-command inventory gives the prototype's justification,
permission/path, owner and removal criteria; no new Flatpak grant is added.

Live acceptance remains pending: new native/Flatpak-style browser OAuth and
primary-account checks, real OneDrive lifecycle/fault/Repair runs, representative
five-minute/cost measurements, OneNote/shortcut behavior and minimum-version
confirmation. The declared minimum stays 1.74.3; a local test on installed 1.75.0
does not establish that lower bound. Forced host transport is not final installed
sandbox/browser-portal acceptance. No cloud test was executed this turn. Operator
entry point and fixed action sequence are in scripts/README.md; start with the
read-only inspect action and the expected sign-in identity, then an explicitly
approved disposable prepare/initial-sync sequence.


✅ October 15, 2026: final prototype gate retry passed, exit 0:
`bash scripts/dev.sh gate`, receipt/log
`target/task-runs/dev-gate-5t4id0ep/{summary.json,task.log}`;
source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Formatting, all-feature Clippy and all ordinary automated tests passed. This
closes the pending gate retry above. The explicitly run local-engine integration
retains its earlier passing focused receipt; it was not repeated during the gate.
No localization/desktop metadata inputs changed, so those extra gates were not
required. New example diagnostics are protocol codes rather than UI strings.

A final real wrapper/Cargo/example/rclone smoke used an empty temporary config
and a nonexistent synthetic remote: inspect correctly returned command_failed
and process exit 1; the verification harness exited 0 after asserting that
expected failure, both receipt files and absence of the synthetic account from
logs/summary. Evidence: `target/onedrive-prototype-cli-smoke.json`. No Microsoft
request, actual account or cloud mutation was involved. This verifies the real
operator command path beyond the fake cargo fixture.

Marked the phase-3 harness source task complete, updated contract/index/inventory
status, and retained all live native/Flatpak-style, representative timing,
minimum-version and installed acceptance items as open. Final git diff --check
passed, exit 0. Operator sequence starts with inspect, then auth if testing fresh
browser OAuth, prepare, separate confirmed initial-sync and scenario. Real cloud
fault cases use separate disposable sessions; Repair and cleanup are explicit.

### October 15, 2026 — prototype inspect config-response repair

The operator's first personal inspect failed with invalid_metadata, exit 1;
receipt/log: `target/task-runs/onedrive-prototype-2tinurlz/`. This did not
complete identity acceptance or mutate cloud data. A local structural inspection
confirmed the expected backend/type and presence of the configured drive and
encoded token; no credential values were printed or copied into records.

Cause: the prototype decoded credential JSON after the shared runner's display
redaction. Redaction can change encoded token JSON, so display output cannot be
used as a configuration protocol response. The fix adds development-only private
config capture restricted to rclone config dump/create/update, with bounded
output, timeout and the existing cancellation/reaping machinery. Its opaque
wrapper hides Debug output, and nonzero errors hide both output streams. Normal
command capture still uses the existing redaction. The prototype consumes private
JSON only in memory; account and primary-drive checks remain unchanged. Config
decode errors now have a separate invalid_config_response protocol code.

Changed source: src/process.rs and src/onedrive_prototype.rs. Updated the task
list, these notes, script catalog/inventory and script output contract. No remote
recreation, installation, applet reload or new permission grant is required.

Verification, all exit 0:
- `just fmt` applied formatting.
- `bash scripts/dev.sh check`: `target/task-runs/dev-check-co_qu3j9/`.
- `cargo test --locked --lib private_config -- --include-ignored`: 3 passed;
  `target/onedrive-private-config-tests.log`. Includes real installed rclone
  reading an isolated synthetic config with encoded tokens; no cloud request or
  user configuration. Tests also cover restricted commands and opaque failures.
- `cargo test --locked --lib onedrive_prototype -- --include-ignored`: 10 passed;
  `target/onedrive-prototype-private-config-regression.log`. Includes the local
  alias engine lifecycle; no live OneDrive acceptance.

Source SHA: 217640a80d0591a79490cc446c5fc9d160bbca42; source_dirty=true.
Final `bash scripts/dev.sh gate` passed, exit 0, receipt/log:
`target/task-runs/dev-gate-fei84v1e/{summary.json,task.log}`. Formatting,
all-feature Clippy and the ordinary automated suite passed. Live identity
inspection remains pending an operator retry of the same personal inspect command.
No UI/localization or desktop metadata inputs changed. No live cloud tests were
run; applet, installed-service and package acceptance remain open.

### October 15, 2026 — prototype profile authorization prerequisite

After the private config-response fix, the operator's second personal inspect
reached Graph and failed with graph_http 401, exit 1. Receipt/log:
`target/task-runs/onedrive-prototype-ce0g2fxn/`. No identity acceptance or cloud
data mutation occurred. Local inspection printed only token-presence, token-type,
redaction-marker and expiry booleans: the stored token was current and included
a refresh token. No token, account, drive ID or provider message was recorded.

The prototype's account lookup calls `/me`, for which Microsoft documents
delegated User.Read as the least privilege. rclone 1.74.3's default OneDrive
scope lists omit that permission. Sources:
[Microsoft Get user permissions](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0#permissions),
[rclone OneDrive backend](https://github.com/rclone/rclone/blob/v1.74.3/backend/onedrive/onedrive.go).
This establishes a missing authentication prerequisite, not proof of the exact
provider-side cause of this particular 401. Existing root stat already performs
a backend request; no assumption that it skipped refresh was used in the fix.

New prototype auth now explicitly requests rclone's default file scopes plus
User.Read, using the same default client. Existing remotes are unchanged. `/me`
401/403 results are classified as account_profile_unauthorized with HTTP status;
other Graph errors preserve their prior code. Identity checks still fail closed;
no drive-owner/name fallback, automatic reauthorization or identity bypass was
introduced. The operator should next use auth with a new cm_proto_ remote name
to test browser profile consent, retaining the two existing remotes. Tenant,
client or token restrictions can still deny access after consent.

Changed src/onedrive_prototype.rs, task list/these notes, script README/inventory,
script output contract and OneDrive contract. Verification, exit 0:
- `just fmt`.
- `bash scripts/dev.sh check`: `target/task-runs/dev-check-3uyg9d_a/`.
- `cargo test --locked --lib onedrive_prototype -- --include-ignored`: 11 passed,
  including synthetic profile-denial/no-mutation coverage and explicitly run
  local-rclone alias lifecycle. Log:
  `target/onedrive-prototype-profile-consent-tests.log`. Auth fixture verifies
  the explicit scope argument and retained secret-safe continuation handling.
- `git diff --check`.

Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Final `bash scripts/dev.sh gate` passed, exit 0; receipt/log:
`target/task-runs/dev-gate-qv4qrwti/{summary.json,task.log}`. Formatting,
all-feature Clippy and ordinary automated tests passed. No UI/localization or
desktop metadata inputs changed. Live new browser consent, personal/business
identity checks and lifecycle acceptance remain open; no cloud tests ran this turn.

### October 15, 2026 — native personal prototype OAuth accepted

The operator ran the maintained prototype auth action for a new personal
cm_proto_ remote. Both summary.json and prototype.json confirm succeeded,
exit 0, native transport, remote_created=true and drive_type=personal.
Receipt/log: `target/task-runs/onedrive-prototype-20ox97ks/`;
source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Only hashed identity fingerprints appear in the prototype receipt.

This verifies real native browser OAuth with the added User.Read request,
expected signed-in account and configured primary personal drive. The new
authentication path succeeded after the earlier config-decode and Graph 401
failures; no claim is made that the old file-only remote was reauthorized.
It does not verify business or Flatpak transport, mirror sync, fault handling,
representative timing, installed services, rendered UI or final packages.

Updated only the task list and these notes, separating the completed native
personal identity check from remaining business/Flatpak checks. No source inputs
changed; retain the passing gate `target/task-runs/dev-gate-qv4qrwti/` rather than
repeat it. Next operator step: prepare a disposable personal session using the
new remote. Prepare creates its own UUID cloud tree, local synthetic files and
initial Preview; local/work/recovery state resides under that new prepare run's
session directory. Initial sync remains a separate explicit action. The existing
sample test_folder and existing whole-drive contents are not mirror inputs.

### October 15, 2026 — native personal preparation and Preview accepted

Operator prepare succeeded, exit 0, native transport. Verified summary.json,
prototype.json and retained session.json under
`target/task-runs/onedrive-prototype-gzpgo0w6/`; source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true. Session state:
owned=true, previewed=true, initialized=false, failed=false, cleaned=false.
mkdir, ownership, remote-fixture and initial-preview stages all succeeded;
the shared initial-preview-confirmable marker is retained in session/work.
Account/drive fingerprints match the preceding accepted personal auth receipt.

The disposable cloud root is
`cm_proto_personal:CloudMounter-Prototype-ccb9485c-9079-4c58-a784-c1e66432ba72/`;
the mirrored content is its tree subdirectory. Local synthetic mirror, work,
recovery and session state are under this prepare run's session directory.
Preparation seeded disposable fixtures and ran Preview; it has not performed
initial bisync. Next operator action is initial-sync with this exact retained
absolute session path, followed by independent byte verification within that
action. Full lifecycle, business and Flatpak-style checks remain open.

Updated only task list and completion notes. git diff --check passed, exit 0.
No code/test inputs changed; the passing gate at
`target/task-runs/dev-gate-qv4qrwti/` remains applicable and was not repeated.

### October 15, 2026 — personal initial bisync succeeded; post-sync guard failed

Operator initial-sync action failed with command_failed, exit 1; receipt/log:
`target/task-runs/onedrive-prototype-il8kdm12/`. The retained preparation session
at `target/task-runs/onedrive-prototype-gzpgo0w6/session/` shows a successful
initial-sync stage (28,783 ms), initialized=true, previewed=false, failed=false.
The shared initial-sync-complete marker exists, coordinator state is succeeded
and both resulting listings include the access anchor. Local synthetic local.txt,
remote.txt and anchor files are present. There is no verify-download stage:
the error occurred in the rclone-backed guard before byte verification started.
The original generic error does not identify which remote read failed or why;
no remote-data or flag change is justified by that evidence.

Added a fixed verify action using the same retained session, operation lock,
identity/ownership/access guards and independent downloaded-byte check. It
requires an initialized, nonfailed session and never repeats initial bisync.
Read-command failures now expose only fixed operation/failure categories and
an optional numeric exit code; cancellation remains typed. Config bodies,
provider output, command arguments and identity values remain absent from these
diagnostics. Shared coordinator failures retain their existing generic handling.
The operator can retry verification without resetting markers, rebuilding state
or mutating cloud data. Exact original provider/transport cause remains unknown.

Changed src/onedrive_prototype.rs, development example, Python action dispatcher
and driver tests; updated task list, these notes, script catalog/inventory and
output contract. An intermediate check failed because a new test accessed a
private CommandArg field; replaced that access with public argument equality.
Failure receipt: `target/task-runs/dev-check-bx3e0e9e/`, exit 101. Following checks:
- `just fmt`, exit 0.
- `bash scripts/dev.sh check`, exit 0:
  `target/task-runs/dev-check-wdzdt50o/`.
- `python3 tests/onedrive_prototype_tasks_test.py`: 2 passed, exit 0; includes
  verify dispatch with absolute session, secret-safe logs and rejection of
  missing/relative sessions before any task starts.

Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
`cargo test --locked --lib onedrive_prototype -- --include-ignored` passed,
exit 0: 12 tests including the isolated real-rclone lifecycle and verification
of a reloaded initialized session. The new fake-adapter regression forces failures
at identity access, config read, owner read, anchor read and downloaded-byte check,
then verifies successful retries issue only read/check commands and retain valid
initialization. It also asserts command/provider sentinel secrets do not enter
serialized diagnostics. Log: `target/onedrive-prototype-verify-retry-tests.log`.
Final `bash scripts/dev.sh gate` passed, exit 0; receipt/log:
`target/task-runs/dev-gate-lskm21lj/{summary.json,task.log}`. Formatting,
all-feature Clippy and ordinary automated tests passed. git diff --check passed,
exit 0. No UI/localization or desktop metadata inputs changed. No cloud tests
were rerun; live byte verification, full lifecycle, business and Flatpak acceptance
remain open. Next operator command uses the new verify action with the existing
initialized session; do not rerun initial-sync or reset its markers.

### October 15, 2026 — personal verification retry accepted

Operator verify succeeded, exit 0, native transport. Verified summary.json and
prototype.json in `target/task-runs/onedrive-prototype-6e_dw5__/` and session.json
in the original `target/task-runs/onedrive-prototype-gzpgo0w6/session/`.
Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Identity, ownership and access guards passed; verify-download succeeded in
5,484 ms. Session remains initialized=true, failed=false, owned=true,
cleaned=false. No additional bisync stage was recorded during this retry.

Together with the earlier successful initial-sync stage, this establishes native
personal initial bisync plus independent downloaded-byte verification of the
disposable fixtures. The earlier post-sync guard failure remains unexplained;
successful retry does not prove its cause or warrant weakening any guard.
Next operator action: scenario on the same retained session, exercising edits
and deletes on both sides, conflict preservation, a 16 MiB upload, repeated byte
checks and synthetic timing comparisons. Fault/Repair, business, Flatpak-style
and representative-tree/cadence acceptance remain open.

Updated only task list and these notes; git diff --check passed, exit 0. No code
inputs changed; retain the passing `target/task-runs/dev-gate-lskm21lj/` gate
receipt rather than repeat it. No additional live actions were run by the agent.

### October 15, 2026 — personal scenario passed; timing reviewed

Operator scenario succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-xeppy7dg/` and retained
session.json at `target/task-runs/onedrive-prototype-gzpgo0w6/session/`.
Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
All added stages succeeded; initialized=true, failed=false, owned=true,
cleaned=false. The scenario verified edits/deletes on both sides, preservation
of both conflict versions, a 16 MiB upload and downloaded-byte comparisons.

The operator reported the long runtime. task.log birth time was
19:05:28.409595 UTC and summary updated_at 19:12:05.755336 UTC, approximately
397.3 seconds (6 min 37 s); this is a filesystem-derived wall-time estimate,
not an instrumented action-duration field. Scenario-only recorded stages:
- Seven bisync transactions: 176.383 s total, individually 21.946–28.420 s.
- Five downloaded-byte checks: 15.841 s total.
- Two remote fixture writes: 6.954 s; one remote delete: 3.688 s.
- Final checksum check subprocess: 2.261 s.
These stages total 205.127 s. The remaining approximately 192.2 s include
repeated identity/ownership/access guards, Cargo startup, the intentional 1.1 s
conflict timestamp delay and other untimed overhead. Guards run outside the
individual stage timers; this receipt cannot attribute that remainder exactly.
No five-minute sleep occurs in scenario/benchmark.

The benchmark separately measures guard-inclusive action durations: quiet shared
sync 31.152 s, quiet sync plus full checksum verification 46.955 s, against a
300 s interval budget. The 15.803 s difference includes a second guard and
variable sync latency, not merely checksum work (the checksum command itself
was 2.261 s). representative_tree=false, scope=synthetic_16mib_tree and
cadence_decision=operator_measurement_required. These are synthetic measurements;
no production five-minute cadence or representative-tree acceptance is inferred.

Updated only task list and these notes. git diff --check passed, exit 0; unchanged
source inputs retain gate `target/task-runs/dev-gate-lskm21lj/` without repeating
tests. The original post-initial-sync guard failure remains unexplained. Next:
fault-network and fault-cancel, each on its own freshly prepared/initialized
disposable session, followed by Repair inspection and explicit cleanup.
Business, Flatpak-style and representative timing acceptance remain open.

### October 15, 2026 — personal cancellation test started

Operator authorized the next test. Selected fault-cancel, with a fresh disposable
session as required by the prototype catalog. Next operator action is prepare
on the existing personal prototype remote; it creates a new UUID cloud test tree
and local run/session, seeds fixtures and performs Preview. It does not reuse
the completed scenario session. After its receipt is checked, initial-sync and
fault-cancel are separate explicit actions, followed by Repair inspection.
The fault action throttles the bisync child, cancels it after one second and
checks cancellation state/reaping while retaining synthetic pending data; the
overall command also performs guards and setup. No manual Ctrl+C is required.

Updated only task list and these notes. No live operation was run by the agent;
preparation and cancellation acceptance are pending. Source inputs are unchanged;
retain passing gate `target/task-runs/dev-gate-lskm21lj/`. git diff --check
passed, exit 0.

### October 15, 2026 — cancellation session preparation accepted

Operator prepare succeeded, exit 0, native transport. Verified summary.json,
prototype.json and session/session.json under
`target/task-runs/onedrive-prototype-zz9bn84r/`. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true. New session UUID
a0f5c308-cc5b-422d-99b4-a986fd47c198 is distinct from the completed scenario.
Its identity fingerprints match the accepted personal account/primary drive.
mkdir, ownership, remote-fixture and initial-preview stages succeeded;
previewed=true, initialized=false, owned=true, failed=false, cleaned=false.

Next operator action: initial-sync using this run's absolute session path;
fault-cancel remains pending until initialization and byte verification pass.
Updated task list and these notes only; no code inputs changed, so the passing
gate `target/task-runs/dev-gate-lskm21lj/` was not repeated. git diff --check
passed, exit 0. No live action was run by the agent.

### October 15, 2026 — cancellation session initialization accepted

Operator initial-sync succeeded, exit 0, native transport. Verified summary.json
and prototype.json at `target/task-runs/onedrive-prototype-w4ae9cq1/`, plus the
retained session at `target/task-runs/onedrive-prototype-zz9bn84r/session/`.
Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Initial-sync stage succeeded in 28,738 ms; verify-download succeeded in 4,882 ms.
State: initialized=true, previewed=false, failed=false, owned=true, cleaned=false.

The fresh session is ready for fault-cancel. Its generic next_action=scenario
receipt describes the normal lifecycle; this dedicated cancellation session
does not need the previously accepted scenario repeated. Next action injects
automatic cancellation into only its own throttled bisync, retains pending
synthetic data and requires the expected cancelled coordinator state before
reporting test success. Repair inspection follows; ordinary sync is not resumed
on that failed session. Cancellation acceptance itself remains pending.

Updated only task list and these notes; git diff --check passed, exit 0. Unchanged
source/test inputs retain the passing `target/task-runs/dev-gate-lskm21lj/` gate;
no checks or cloud actions were repeated by the agent.

### October 15, 2026 — personal cancellation observed; Repair pending

Operator fault-cancel succeeded, exit 0, native transport. Verified summary.json
and prototype.json at `target/task-runs/onedrive-prototype-1ov9i7nn/`, retained
session.json and shared coordinator state in the zz9bn84r session. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
cancel-upload stage succeeded in 17,856 ms; the shared sync state is cancelled.
The harness requires an observed typed cancellation from its bisync runner and
awaits that operation before reporting test success. This is expected injected
cancellation, not successful completion of the pending upload.

Session initialized=true, failed=true, owned=true, cleaned=false. Synthetic
local/pending.bin remains exactly 16,777,216 bytes. The failed flag intentionally
blocks ordinary sync; no automatic resume, resync or lock deletion was performed.
Next operator action is repair on this same retained session to inspect locks
and pending-data preservation. The completed scenario session remains separate.
Repair, explicit disposable cleanup, network-fault, business/Flatpak and
representative-performance acceptance remain pending.

Updated task list and these notes only; git diff --check passed, exit 0. Source
and test inputs are unchanged; retain passing gate
`target/task-runs/dev-gate-lskm21lj/`. No live action was repeated by the agent.

### October 15, 2026 — personal cancellation Repair inspection accepted

Operator repair succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-0t_xxwq4/`; retained
session remains `target/task-runs/onedrive-prototype-zz9bn84r/session/`.
Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Details: failed=true, locks_preserved=0, pending_bytes_preserved=true,
implicit_resync=false. Identity/access/ownership guards passed. The pending-file
flag checks existence, not a byte hash; the earlier cancellation inspection
separately confirmed its 16 MiB size. No bisync .lck files were present to preserve;
this run does not establish live handling of a preexisting stale lock.

Marked native personal cancellation plus nonmutating Repair inspection complete.
Repair here only inspects and preserves state; it does not implement recovery
or resume sync. Next operator action is explicit cleanup on this session, which
rechecks account/drive/root ownership and purges only its UUID cloud test root,
including disposable recovery copies. Local session evidence and pending data
remain. The completed scenario session and configured rclone remotes stay intact.
Cleanup, separate network-fault testing, business/Flatpak and representative
performance acceptance remain open.

Updated only task list and these notes; git diff --check passed, exit 0.
Unchanged source/test inputs retain gate `target/task-runs/dev-gate-lskm21lj/`;
no tests or live operations were repeated by the agent.

### October 15, 2026 — cancellation cleanup accepted; network-fault test next

Operator cleanup succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-k8bk7g8m/`, plus the retained
zz9bn84r session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Receipt reports cloud_cleanup=purged, local_evidence=retained. The cleanup stage
succeeded in 2,090 ms; session cleaned=true, failed=true. Local pending.bin still
exists at 16,777,216 bytes. This confirms the ownership-guarded purge command
succeeded for the cancellation session's UUID cloud root; no separate cloud
absence probe was run. It is not a reset/resume of the failed session.

Native personal cancellation, Repair inspection and explicit disposable cloud
cleanup are now recorded as complete. Next is network-fault testing on a fresh
prepared/initialized session, using the same configured personal remote. The
fault action affects only the bisync child's source address, not host networking.
Network failure/Repair/cleanup, business/Flatpak, representative timing and
installed acceptance remain outstanding. The original completed scenario cloud
tree remains separate and has not been cleaned up.

Updated only task list and these notes; git diff --check passed, exit 0. No code
or test inputs changed; retain passing gate `target/task-runs/dev-gate-lskm21lj/`
without repeating it. No cloud operations were run by the agent.

### October 15, 2026 — network-fault session preparation accepted

Operator prepare succeeded, exit 0, native transport. Verified summary.json,
prototype.json and session/session.json at
`target/task-runs/onedrive-prototype-i7ulanzh/`. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Session UUID 4fbcd105-bfa1-443e-885a-238ba94005ac is distinct from both earlier
personal sessions. Identity fingerprints match the accepted personal drive.
mkdir, ownership, remote-fixture and initial-preview stages all succeeded;
previewed=true, initialized=false, owned=true, failed=false, cleaned=false.

Next operator action is initial-sync with this run's retained absolute session
path. Network-fault acceptance awaits initialization, fault-network, Repair and
explicit cleanup. Updated only task list and these notes; git diff --check passed,
exit 0. Source/test inputs are unchanged; retain the passing gate at
`target/task-runs/dev-gate-lskm21lj/`. No cloud action was run by the agent.

### October 15, 2026 — network-fault session initialization accepted

Operator reported initial-sync success. Located and verified its receipt at
`target/task-runs/onedrive-prototype-tpeynjmf/summary.json` and the linked
prototype.json, which selects the i7ulanzh session. Native transport, succeeded,
exit 0; source SHA 217640a80d0591a79490cc446c5fc9d160bbca42,
source_dirty=true. Retained session stages confirm initial-sync succeeded in
31,926 ms and verify-download in 2,889 ms. State initialized=true,
previewed=false, failed=false, owned=true, cleaned=false.

Next operator action is fault-network on this same session. It gives only the
bisync child an unavailable bind address and bounded connection/operation
timeouts/retries; the test requires an observed typed nonzero/timeout failure
and failed coordinator state. This is an injected transport failure, not a
physical host-network outage. It retains synthetic pending data for subsequent
Repair inspection and explicit cleanup. Network-fault acceptance remains open.

Updated only task list and these notes; git diff --check passed, exit 0. No code
or test inputs changed; retain passing gate `target/task-runs/dev-gate-lskm21lj/`.
No live operation was repeated by the agent.

### October 15, 2026 — personal injected network failure observed

Operator fault-network succeeded, exit 0, native transport. Verified summary.json
and prototype.json at `target/task-runs/onedrive-prototype-hlnj6h29/`, retained
i7ulanzh session.json and shared coordinator state. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
network-unavailable stage succeeded in 21,461 ms; shared sync state is failed.
The harness requires an observed typed nonzero/timeout failure from the injected
bisync request as well as that failed state; success means expected fault
handling, not successful synchronization. Local pending.bin remains
16,777,216 bytes. Session initialized=true, failed=true, owned=true, cleaned=false.

This verifies controlled transport-failure handling against the disposable
personal session. It does not simulate every physical network outage or prove
resumption of an interrupted transfer. Next operator action: repair on this same
session to inspect preserved pending data/locks without sync or implicit resync;
then explicit cleanup. Network-fault Repair/cleanup acceptance remains pending.

Updated task list and these notes only; git diff --check passed, exit 0. No source
or test inputs changed; retain passing gate `target/task-runs/dev-gate-lskm21lj/`.
No live operation was repeated by the agent.

### October 15, 2026 — personal network-fault Repair inspection accepted

Operator repair succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-g2ln84d0/`; retained
session is `target/task-runs/onedrive-prototype-i7ulanzh/session/`. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Details: failed=true, locks_preserved=0, pending_bytes_preserved=true,
implicit_resync=false. Identity/access/ownership guards passed. Local pending.bin
still exists at 16,777,216 bytes. The receipt's preservation flag checks existence;
no hash comparison was performed. No bisync .lck files were present, so stale-lock
preservation is not live-verified by this run.

The session remains failed as expected; Repair inspection performed no sync,
implicit resync or lock deletion. Next operator action is explicit cleanup on
the same session: ownership-guarded purge of only this UUID cloud test root,
including its disposable recovery copies, while retaining local pending data and
evidence. Network-fault task remains open until that cleanup succeeds. Business,
Flatpak-style, representative timing and installed acceptance remain outstanding.

Updated only task list and these notes; git diff --check passed, exit 0. Unchanged
source/test inputs retain passing gate `target/task-runs/dev-gate-lskm21lj/`.
No tests or live operations were repeated by the agent.

### October 15, 2026 — personal network-fault cleanup accepted

Operator cleanup succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-1ix636x6/`, plus retained
i7ulanzh session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Receipt reports cloud_cleanup=purged and local_evidence=retained. Cleanup stage
succeeded in 1,022 ms; cleaned=true, failed=true. Local pending.bin still exists
at 16,777,216 bytes. This records successful ownership-guarded purge of the
network-fault session's UUID cloud root; no separate cloud absence probe ran.

Marked native personal network-fault testing, Repair inspection and disposable
cloud cleanup complete. Native personal normal scenario and cancellation tests
also passed earlier. The original normal-scenario cloud tree remains separate
and retained. Business OAuth/lifecycle/fault checks, Flatpak-style acceptance,
representative-tree timing/cadence, minimum-version and installed acceptance
remain open. Next operator action: new cm_proto_business browser authentication
with the expected business sign-in identity, using the same profile-consent
flow proven for personal. Existing onedrive_business is not replaced.

Updated task list and these notes only, correcting the order of the two preceding
agent-authored network-failure/Repair entries without changing their contents.
git diff --check passed, exit 0. Unchanged source/test inputs retain passing gate
`target/task-runs/dev-gate-lskm21lj/`; no tests or cloud operations were repeated
by the agent.

### October 15, 2026 — business auth consent failure compared with SharePoint

Operator reported browser admin approval during new business prototype auth.
Verified failed receipt/prototype.json at
`target/task-runs/onedrive-prototype-rh1atd7h/`: action exit 1, config child
nonzero 2, native transport; source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true. No successful
business identity verification or mirror setup is implied. Local configuration
inspection found no remaining cm_proto_business section after failure; existing
business and SharePoint remote sections remain present.

Compared src/teams_oauth.rs rclone_config_request/select_library_drive and
src/teams.rs verify_graph_drive with prototype AUTH_SCOPES/primary. SharePoint
uses rclone default browser scopes and config_type=url for a selected library;
it verifies that library's ID/type/canonical URL through Graph drive metadata.
It does not require /me profile access. Prototype config_type=onedrive requests
those default scopes plus User.Read, then calls /me and /me/drive to bind an
expected account to its primary drive. User.Read was added for the personal
profile refusal earlier; it is not required for file synchronization itself.

Read-only local configuration comparison printed only role labels, fixed
permission names and booleans. Both existing business and SharePoint remotes
use no custom client/secret, explicit scope override, auth/token URL override or
alternate tenant_url endpoint. Their stored JWT scope claims list the standard
file/site permissions and omit User.Read. Decoded claims were diagnostic only,
not cryptographically validated identity evidence. No credentials, account names,
organization names, raw JWTs or Graph bodies were printed or retained.

The extra scope/new consent is the leading explanation for the browser policy
prompt, not a proven tenant-policy diagnosis: the receipt retains only a safe
config failure category. User.Read delegated permissions do not intrinsically
require admin consent, but tenant policy may restrict new user consent while
existing grants remain usable. Microsoft documents the distinction:
[User.Read permissions](https://learn.microsoft.com/en-us/graph/permissions-reference#userread),
[tenant consent policy](https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/configure-user-consent).
No tenant policy was queried or changed.

Candidate next business fix: keep the existing file-scope authorization and
compare the configured drive, /me/drive and
/users/{expected-user-principal-name}/drive with server-returned IDs/types.
[Get Drive](https://learn.microsoft.com/en-us/graph/api/drive-get?view=graph-rest-1.0)
documents these endpoints under delegated Files.Read permissions. This must be
implemented and tested with wrong-account/primary-drive rejection before use;
it is not a fallback that skips identity verification. It may still encounter
provider/tenant restrictions; existing remotes should be preserved. Personal
profile-based behavior and accepted tests need not change for this investigation.

Updated task list and these notes only. No Rust/scripts, credentials, consent,
services or cloud data changed. Existing source inputs retain passing gate
`target/task-runs/dev-gate-lskm21lj/`; git diff --check passed, exit 0. Business
OAuth and native/Flatpak lifecycle acceptance remain open.

### October 15, 2026 — business file-scope identity fix implemented

Operator requested the fix after the admin-consent comparison. Revised only the
development prototype's business path: browser auth uses rclone default scopes,
without access_scopes override or User.Read. Personal keeps its already accepted
profile-consent flow and identity fingerprints. Existing remotes/grants are not
rewritten. The new business flow remains subject to actual tenant policy; removal
of the extra permission is not a guarantee that every consent failure disappears.

Business requires a validated expected sign-in UPN. Typed Graph targets build
only the fixed Microsoft Graph host; the UPN is encoded as one path segment.
The policy compares server responses from /me/drive and
/users/{expected-UPN}/drive, requiring equal drive IDs, business types and matching
nonempty owner user IDs. The configured drive must equal that primary drive.
The account fingerprint hashes the returned owner ID. No profile /me call,
JWT-claim shortcut, display-name fallback or skipped wrong-account check is used.
Denied or missing metadata fails closed. The default-scope requests match the
working SharePoint auth pattern while preserving the OneDrive primary-drive
requirement. API reference:
[Get Drive permissions and user-drive endpoints](https://learn.microsoft.com/en-us/graph/api/drive-get?view=graph-rest-1.0).

Changed src/onedrive_prototype.rs (typed targets, business binding, conditional
scope request and regressions), task list/these notes, scripts/README.md and
engineering OneDrive/output contracts and tooling inventory. No applet routing,
UI strings, system services, grants or real remote configuration changed.
Verification, exit 0:
- just fmt.
- bash scripts/dev.sh check: target/task-runs/dev-check-li_ci5ra/.
- cargo test --locked --lib onedrive_prototype -- --include-ignored: 15 passed,
  including isolated real-rclone lifecycle and existing personal tests. Log:
  target/onedrive-prototype-business-file-scopes-tests.log.
  New tests cover fixed-host/encoded-UPN URLs, malformed inputs, business auth
  without a scope override, wrong signed-in/expected drive or owner, library
  rejection, missing owner/drive fields, denied/not-found account lookups and
  configured cache-drive rejection. They use synthetic data and Graph fakes.

Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Final bash scripts/dev.sh gate passed, exit 0; receipt/log:
`target/task-runs/dev-gate-eul7_c_n/{summary.json,task.log}`. Formatting,
all-feature Clippy and ordinary automated tests passed; git diff --check passed,
exit 0. No UI/localization or desktop metadata inputs changed. Native business
OAuth and both server drive lookups await operator acceptance; no live cloud
operation was run during this fix. Next: retry the same new business auth action
with the business sign-in UPN; its prior failed partial remote was confirmed
absent during the preceding investigation.

### October 15, 2026 — revised business native OAuth and identity accepted

Operator reran business auth successfully. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-nyvqpbet/`: succeeded,
exit 0, native transport, remote_created=true, drive_type=business. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true. Receipt contains
only hashed identity fingerprints, distinct from the accepted personal identity.

This accepts the revised default-scope browser flow and server-side business
binding: signed-in primary drive and expected-UPN primary drive agree on drive
and owner IDs, and the configured drive matches. The retry succeeded after the
prior admin-consent failure with the extra User.Read scope; no tenant-policy
change, custom client or skipped identity check was introduced. Existing remotes
were not replaced. Business mirror lifecycle, faults/Repair, Flatpak-style,
representative-performance and installed acceptance remain open.

Updated task list, these notes, OneDrive contract and script inventory to reflect
native business authentication acceptance separately from the remaining checks.
No code/test inputs changed; retain passing gate
`target/task-runs/dev-gate-eul7_c_n/` rather than repeat it. git diff --check
passed, exit 0. Next operator step is prepare on the new business prototype
remote: create a disposable UUID cloud tree and synthetic local fixtures, then
Preview. Initial sync remains separate. No cloud action was run by the agent.

### October 15, 2026 — business session preparation accepted

Operator prepare succeeded, exit 0, native transport. Verified summary.json,
prototype.json and session/session.json at
`target/task-runs/onedrive-prototype-8v4nz4pi/`. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Session UUID 54e2391a-562f-4a2c-8fe2-d824cfa92809 is distinct from the personal
sessions; its identity fingerprints match accepted business authentication.
mkdir, ownership, remote-fixture and initial-preview stages succeeded;
previewed=true, initialized=false, owned=true, failed=false, cleaned=false.

Next operator action is initial-sync with this run's retained absolute session
path, followed by independent downloaded-byte verification within that action.
Only the disposable UUID tree and synthetic fixtures are in scope; no whole-drive
mirror acceptance is implied. Business initialization/scenario/fault/Repair,
Flatpak-style and representative timing acceptance remain open.

Updated only task list and these notes; git diff --check passed, exit 0.
Unchanged source/test inputs retain passing gate
`target/task-runs/dev-gate-eul7_c_n/`; no tests or cloud operations were repeated
by the agent.

### October 15, 2026 — business initialization and byte verification accepted

Operator initial-sync succeeded, exit 0, native transport. Verified summary.json
and prototype.json at `target/task-runs/onedrive-prototype-4b7jieoi/`, plus the
retained `target/task-runs/onedrive-prototype-8v4nz4pi/session/session.json`.
Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Initial-sync succeeded in 18,834 ms; verify-download succeeded in 1,841 ms.
Session initialized=true, previewed=false, failed=false, owned=true, cleaned=false.

Marked native business initial bisync and independent downloaded-byte verification
complete for the disposable fixtures. Next operator action is scenario on this
same session: edits/deletes both sides, conflict preservation, a 16 MiB upload,
independent byte checks and synthetic timing comparison. It runs several syncs
and may take several minutes, as observed in the personal scenario. Business
scenario/fault/Repair, Flatpak-style and representative timing acceptance remain
open; no whole-drive or installed-app acceptance is implied.

Updated only task list and these notes; git diff --check passed, exit 0.
Unchanged source/test inputs retain passing gate
`target/task-runs/dev-gate-eul7_c_n/`; no tests or live operations were repeated
by the agent.

### October 15, 2026 — business scenario accepted; remaining tests narrowed

Operator scenario succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-6dlwwed1/` and retained
8v4nz4pi session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true. All added stages
succeeded; initialized=true, failed=false, owned=true, cleaned=false. This accepts
native business edits/deletes both sides, preservation of both conflict versions,
16 MiB upload and independent downloaded-byte checks. Benchmark: quiet sync
22,744 ms, sync plus full checksum verification 31,868 ms; representative_tree=false,
synthetic_16mib_tree, cadence_decision=operator_measurement_required. These do not
establish representative-drive performance or the production cadence.

Operator explicitly requested no duplication of personal coverage. Updated the
remaining plan accordingly: native shared cancellation, injected transport failure,
runner reaping/state and nonmutating Repair coverage reuse the personal sessions
and their receipts already recorded above. Redundant business fault repetitions
are deliberately skipped (Task List marker o), not marked as business live passes.
A different account-auth path justified business OAuth/identity and normal-scenario
checks; those now passed. Rerun shared tests only when changed code, an unresolved
failure or drive-specific evidence warrants it. The earlier unexplained personal
post-initial guard failure remains recorded; successful later verification does
not prove its cause.

Remaining prototype work targets genuinely uncovered behavior: cleanup of the
two retained normal-scenario cloud trees; focused Flatpak-style identity/OAuth
transport differences and one disposable initial-sync/verification/cleanup smoke;
representative-tree timing/cadence, minimum-version confirmation and cloud-native
item handling. Full installed sandbox/portal/service/UI acceptance remains
separate. No additional full native scenarios or business fault sessions are
required under this operator-approved test plan. The scenario receipt's generic
fault next_action does not override this plan.

Updated Task List.md, these notes, script catalog/inventory and the existing
OneDrive contract to make scope and evidence reuse explicit. No runtime/script
implementation or test inputs changed; retain passing gate
`target/task-runs/dev-gate-eul7_c_n/` rather than repeat it. git diff --check
passed, exit 0. No cloud operation was run by the agent. Next operator action:
explicit cleanup of the business scenario session, then the retained personal
scenario session, preserving local evidence and both configured remotes.

### October 15, 2026 — business normal-scenario cleanup accepted

Operator cleanup succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-m9t0u83x/`, plus retained
8v4nz4pi session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Receipt reports cloud_cleanup=purged and local_evidence=retained; cleaned=true,
failed=false. Cleanup stage succeeded in 1,052 ms. This records successful
ownership-guarded purge of the business UUID cloud root and its recovery copies;
no separate cloud absence probe was run. Local session evidence remains.

Marked business scenario cleanup complete. The retained personal normal-scenario
session gzpgo0w6 remains initialized=true, failed=false, cleaned=false and is next
for explicit cleanup. This removes a different disposable resource, not a repeat
of the accepted fault tests. Existing remotes and local evidence remain in scope
for preservation. Reduced remaining test plan is unchanged; no redundant business
fault or normal-scenario rerun is introduced.

Updated only task list and these notes; git diff --check passed, exit 0.
Unchanged source/test inputs retain passing gate
`target/task-runs/dev-gate-eul7_c_n/`; no tests or cloud operations were repeated
by the agent.

### October 15, 2026 — all native prototype session cloud cleanup recorded

Operator personal scenario cleanup succeeded, exit 0, native transport. Verified
summary.json and prototype.json at `target/task-runs/onedrive-prototype-7zgdotbm/`
and retained gzpgo0w6 session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Receipt reports cloud_cleanup=purged and local_evidence=retained; cleanup stage
succeeded in 1,434 ms. Checked all four retained native session records:
gzpgo0w6 (personal scenario), zz9bn84r (cancellation), i7ulanzh (network fault)
and 8v4nz4pi (business scenario) report cleaned=true and successful cleanup
stages. These are command/session receipts, not additional cloud absence probes.
Local evidence and configured remotes remain retained.

Marked the remaining personal scenario cleanup complete. The operator-approved
plan still reuses native shared-fault results and omits redundant business fault
or normal-scenario reruns. Next distinct transport work needs a Flatpak environment:
read-only host command lookup found no flatpak-spawn in PATH, and the existing
FlatpakHostCommandRunner requires that executable. No failing cloud smoke was
launched, no fallback transport substituted, and no package/service was changed.
Flatpak transport/browser-portal acceptance remains outstanding, together with
representative performance/cadence, minimum-version and cloud-native item checks.

Updated only task list and these notes; git diff --check passed, exit 0.
Unchanged source/test inputs retain passing gate
`target/task-runs/dev-gate-eul7_c_n/`; no tests or cloud operations were repeated
by the agent.

### October 15, 2026 — Flatpak checks transferred to implementation acceptance

Operator selected moving remaining Flatpak work to the appropriate Implementation
step. Replaced the phase-3 open transport task with an o transfer reference and
added an unchecked integrated Flatpak acceptance task after routing, remote setup,
planner/coordinator, units/status, editor and source regression work in phase 4.
No Flatpak result is marked passed or waived. Native prototype receipts remain
accepted within their recorded scope; the earlier host-shell executable absence
is not a prerequisite to source implementation.

The transferred task reuses existing applet manifests/runtime and relevant
host-runner/GUI probe setup. It covers personal/business identity/OAuth,
browser-portal and selected-path behavior, host/sandbox boundaries, integrated
service/status behavior and one disposable initial-sync/verification/cleanup
smoke. Unchanged native scenarios and shared fault/Repair tests are reused;
no separate prototype sandbox or duplicate business fault suite is scheduled.
Transport probes remain distinct from installed applet/portal/service acceptance.
No build, installation or launch was performed or newly authorized by this edit.

Updated Task List.md, these notes, script catalog/inventory and the OneDrive
contract to agree on sequencing. No runtime/script or test inputs changed;
retain passing gate `target/task-runs/dev-gate-eul7_c_n/`. git diff --check
passed, exit 0. Next source step remains phase-4 configuration/routing, followed
by setup/coordinator/editor integration; Flatpak acceptance follows that work.

### October 15, 2026 — small-project sample timing harness implemented

Operator authorized a first timing baseline using the supplied project/sample
folder. Before interruption, source edits, bash scripts/dev.sh check and the
Python driver tests had passed. Resumed from those saved changes without repeating
passing unchanged checks or touching the original samples. The earlier read-only
inventory found 317 files and 39,779,429 bytes including .git; this is a small
project workload, not acceptance of representative large-drive performance.

Added prepare --files-dir and a dedicated timing action to the maintained
prototype wrapper/example. Preparation reads an absolute source, rejects overlap
with the session, symbolic links, special files, non-UTF-8 names and bounded tree
limits (20,000 entries, 2 GiB, depth 64). Fixed rclone copy creates local/sample
under the new private session, including .git and empty directories. Source-before,
source-after and copied relative-name/type/content SHA-256 inventories must agree
before cloud setup. Aggregate sample counts/digest enter optional sample_tree
state; source paths and individual filenames/contents do not. Existing schema-1
sessions still load without that optional field. The source must remain stable
during preparation; originals are never a mirror destination.

After separate initial-sync and downloaded-byte verification, timing measures
quiet sync, sync plus full checksum verification, and a small change made only
to a synthetic timing file outside the sample copy. It independently verifies
downloaded bytes afterward and retains the initial-sync engine duration from
stage evidence. Normal scenario rejects sample sessions to prevent accidental
functional-test duplication. Scope=operator_sample_tree, representative_tree=false,
cadence_exercised=false; 300 seconds is a budget comparison, not an exercised
periodic-service interval or a finalized verification cadence. Initial engine
stage time excludes its entry guards; action timings include guards. Explicit
cleanup purges only the UUID cloud root and retains local copies/evidence.

Changed src/onedrive_prototype.rs, gated example, Python action dispatcher and
driver tests; task list/these notes, script catalog/inventory, output/OneDrive
contracts and host-command inventory. Host sample copy is documented under the
existing fixed rclone engine exception; no new Flatpak permission is granted.
No production applet routing, UI strings, services or credentials changed.
Verification, all exit 0:
- just fmt.
- bash scripts/dev.sh check: target/task-runs/dev-check-v4bnu_qt/.
- python3 tests/onedrive_prototype_tasks_test.py: 2 passed, including sample option
  and timing dispatch, absolute paths/action boundaries and secret-safe logging.
- cargo test --locked --lib onedrive_prototype -- --include-ignored: 17 passed;
  target/onedrive-prototype-sample-timing-tests.log. Includes installed-rclone
  local alias lifecycle/sample timing with synthetic fixtures and Graph fakes,
  source/copy hashes unchanged, empty/.git directories and read-only source files,
  overlap rejection, symlink/oversize/cancellation failure and old personal/
  business behavior. No network, actual account or original sample is involved.
- bash -n scripts/onedrive-mirror-prototype.sh and git diff --check.

Source SHA 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Final bash scripts/dev.sh gate passed, exit 0; receipt/log:
`target/task-runs/dev-gate-3gy_byw9/{summary.json,task.log}`. Formatting,
all-feature Clippy and ordinary automated tests passed. No UI/localization or
desktop metadata inputs changed. Final git diff --check passed, exit 0.
Next live operator step: prepare a new business disposable sample session with
--files-dir, followed by separate initial-sync, timing and cleanup actions.
Original samples remain untouched. Live timings and full representative-workload/
cadence acceptance are still pending.

### October 15, 2026 — business project-sample preparation accepted

Operator prepare --files-dir succeeded, exit 0, native transport. Verified
summary.json/prototype.json and session/session.json at
`target/task-runs/onedrive-prototype-v02yqke_/`. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Copied workload: 317 files, 39,779,429 bytes, 170 directories, maximum entry depth
10 (includes file entries). Source-before/source-after/copy inventories matched
at preparation; only aggregate counts/digest are recorded. sample-copy succeeded
in 201 ms, cloud mkdir in 4,296 ms, ownership in 4,492 ms and Preview in 5,583 ms.
State initialized=false, previewed=true, failed=false, owned=true, cleaned=false.
Identity fingerprints match the accepted business account/primary drive.

Next operator action is initial-sync on this retained session. This uploads the
copied workload plus synthetic timing/access files into only its owned UUID cloud
tree and independently verifies downloaded bytes. It may take several minutes
because of the file count; logs remain under the new action run in target/task-runs.
Timing and explicit cleanup follow separately; original samples are not sync
inputs after preparation. No completed functional scenario/fault test is repeated,
no periodic five-minute cadence is inferred and full representative acceptance
remains open.

Updated only task list and these notes; unrelated README.md changes are preserved.
No source/test inputs changed; retain passing gate
`target/task-runs/dev-gate-3gy_byw9/`. Global git diff --check returned exit 2
for preexisting/unrelated README.md trailing whitespace at line 291; it was not
modified. Scoped git diff --check on the two changed task/evidence files passed,
exit 0. No cloud operation was run by the agent.

### October 15, 2026 — business project-sample initial upload accepted

Operator initial-sync succeeded, exit 0, native transport. Verified summary.json
and prototype.json at `target/task-runs/onedrive-prototype-7ygyrdn1/`, plus retained
v02yqke_ session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Initial-sync stage succeeded in 327,349 ms (5 min 27 s); independent
verify-download succeeded in 32,957 ms (33 s). task.log birth time
23:19:38.131500 UTC to summary updated_at 23:25:55.098298 UTC gives an approximate
wall time of 376.967 s (6 min 17 s), including guards/startup/other overhead.
These filesystem-derived wall times are not an instrumented action duration.

Session initialized=true, previewed=false, failed=false, owned=true, cleaned=false;
sample inventory remains 317 files, 39,779,429 bytes, 170 directories, same digest.
This is successful initial population plus downloaded-byte verification of the
copied sample tree. It does not establish recurring unchanged-sync latency; the
initial duration exceeding 300 s alone does not determine periodic-sync cadence.
Earlier network-rate estimates were provisional and are superseded by these
measured durations.

Next operator action is timing on this same session: quiet sync, sync plus full
checksum verification, a separate synthetic small edit and independent downloaded
verification. No initial upload, original-file edit, full scenario or fault test
is repeated. Cleanup follows after timing evidence is inspected. Small-project
baseline, representative workload and exercised periodic cadence remain distinct.

Updated only task list and these notes; unrelated README.md changes remain intact.
Scoped git diff --check on the changed task/evidence files passed, exit 0. No
source/test inputs changed; retain passing gate
`target/task-runs/dev-gate-3gy_byw9/`. No live operation was run by the agent.

### October 15, 2026 — business small-project timing baseline accepted

Operator timing succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-9so2o6hi/` and retained
v02yqke_ session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Workload remains 317 sample files, 39,779,429 bytes and 170 directories, with the
same prepared digest. Session initialized=true, failed=false, owned=true,
cleaned=false. No completed functional scenario/fault suite was repeated.

Recorded benchmark timings:
- Initial-sync engine duration from the prior action: 327,349 ms; excludes its
  entry guards and later verification.
- Quiet sync including guards: 36,009 ms; engine stage 25,566 ms.
- Quiet sync plus full checksum verification including guards: 51,952 ms;
  second engine stage 25,556 ms and checksum subprocess 8,315 ms.
- Small synthetic change sync including guards: 38,184 ms; engine stage 28,853 ms.
- Independent downloaded-byte verification after the change: 30,526 ms,
  separate from the small-change sync duration above.
The 15,943 ms difference between the two quiet benchmark actions includes extra
identity/access guard work and variable latency, not only the 8,315 ms checksum
subprocess. Quiet means no intentional edits; these receipts do not record or
assert zero transferred bytes/files for those stages.

All measured recurring actions are below the 300-second comparison budget on
this workload. Scope=operator_sample_tree, representative_tree=false,
cadence_exercised=false, cadence_decision=operator_measurement_required. Marked
only the small-project baseline complete, not larger/target-workload scalability,
five-minute scheduled behavior or a final checksum cadence. Source policy stays
read_only_copy_verified_at_prepare: timing edits a separate synthetic file in
session/local, not original samples. No source paths or contents enter the receipt.

Next operator action is explicit cleanup on v02yqke_; its owned UUID cloud root
and disposable recovery copies are purged, while local copied samples/evidence
and configured remotes remain. Cleanup remains pending. Updated only task list
and these notes; unrelated README.md changes remain intact. Scoped git diff
--check on changed task/evidence files passed, exit 0. No source/test inputs
changed; retain passing gate `target/task-runs/dev-gate-3gy_byw9/`. No live
operation was run by the agent.

### October 15, 2026 — project-sample cleanup accepted; throughput interpretation

Operator cleanup succeeded, exit 0, native transport. Verified summary.json and
prototype.json at `target/task-runs/onedrive-prototype-09174atk/` and retained
v02yqke_ session.json. Source SHA
217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true.
Receipt reports cloud_cleanup=purged and local_evidence=retained; cleaned=true,
failed=false, cleanup stage succeeded in 2,000 ms. This is successful command/
session evidence, not an additional cloud absence probe. Local sample copies,
original source and configured remotes remain retained.

Operator asked whether timings imply about 600 kB/s. They do not measure wire
throughput. The 39,779,429-byte initial workload divided by its 327.349-second
engine transaction gives approximately 121.5 decimal kB/s effective workload
throughput including per-file/API/metadata overhead. Quiet-sync and sync-plus-
checksum durations must not be treated as transfers of all sample bytes;
their receipts contain no network-byte counters. Network bursts observed by the
operator can coexist with lower end-to-end workload throughput. No bandwidth or
cadence setting was changed based on this derived ratio.

Updated only task list and these notes; unrelated README.md changes remain
preserved. Scoped git diff --check passed, exit 0. No source/test inputs changed;
retain passing gate `target/task-runs/dev-gate-3gy_byw9/`. The small-project timing
baseline and its cleanup are complete; representative scaling/periodic cadence,
minimum version and implementation/Flatpak acceptance remain open. No tests or
cloud operations were repeated by the agent.


### October 15, 2026 — timing preparation scope accepted; duplicate scheduling tests skipped

Operator decided that a larger tree and repeated interval runs are unnecessary
for this prototype, given the completed sample timing and existing SharePoint
scheduling coverage. This supersedes the earlier open larger-tree/periodic timing
requirement. Marked that additional work [o], deliberately skipped, and retained
the completed 317-file baseline [x]. Timing preparation is complete within this
accepted scope; receipts still correctly report representative_tree=false and
cadence_exercised=false. No new large-tree or scheduled live result is claimed.

Implementation will reuse the shared SharePoint scheduling machinery. Existing
implementation tasks for OneDrive unit generation, guards and integrated status
remain open; changes to those paths still require focused verification. No extra
prototype interval or scaling run is required. Minimum rclone version/flags and
dependency checks remain a separate open preparation item.

Changed only Task List.md and these notes. Scoped git diff --check passed, exit 0.
No source/test inputs changed; retain the earlier passing gate receipt at
`target/task-runs/dev-gate-3gy_byw9/`. No cloud operations or tests were repeated.


### October 15, 2026 — rclone minimum and generic OneDrive flag assessment

Confirmed the existing supported floor, rclone >= 1.74.3, without raising it.
Compared shared sync planner and coordinator arguments against pinned upstream
v1.74.3 cmd/bisync/cmd.go and backend/onedrive/onedrive.go. Generic compare,
recovery, locking, conflict, per-side backup, filters, empty-directory and initial
resync flags exist at that floor. Recorded references and assessment boundaries
in docs/engineering/onedrive-rclone-offline-contracts.md. This determines the
project minimum; it does not assert those features first appeared in 1.74.3.

Read-only consistency check passed, exit 0: installer RCLONE_MIN_VERSION,
diagnostics MIN_RCLONE_VERSION, Debian versioned dependency and dependency guide
all agree on 1.74.3. Existing installer checks versions before/after updates;
no dependency check code change is necessary. Runtime executable discovery and
typed operation failures remain unchanged, with no startup version/help probes.
Existing guard logs confirm rclone v1.75.0 for both cloud runs at
`target/task-runs/onedrive-guard-check-8zm9owl_/task.log` and
`target/task-runs/onedrive-guard-check-xpn4ex9d/task.log`. No 1.74.3 binary was
installed or tested live. Marked preparation minimum/flags complete on this
source compatibility basis, retaining the live-test version distinction.

Flag audit finding: upstream bisync treats --max-delete as a percentage and
clamps values >100 to 100. The shared planner's 1000 is not a 1,000-file cap.
Added a phase-4 corrective task to choose/verify the intended shared deletion
policy before production integration, with fake-adapter safety coverage for
SharePoint and OneDrive. No deletion or policy change was performed now.

Updated task/evidence records, dependency guide, OneDrive contract and script
catalog wording. Reconciled the latter records with the operator's earlier
accepted timing scope; no larger-tree/repeated-interval test is required.
Scoped git diff --check passed, exit 0. No Rust, script or test inputs changed;
no test/gate rerun is required; retain passing gate
`target/task-runs/dev-gate-3gy_byw9/`. Unrelated worktree changes are preserved.
No install, service, cloud, commit or publication operation was run. Next step
is phase-4 source implementation, including the recorded deletion-policy fix;
installed native/Flatpak acceptance remains separate.


### October 15, 2026 — phase-4 checklist reconciled with prototype findings

Reviewed the implementation checklist against the accepted OneDrive contract and
completed prototype findings. Added explicit configuration-first work for engine
selection, saved drive identity, backward compatibility and path overlap;
account-specific personal/business OAuth/identity behavior; identity/anchor guards
at every run entry point; private credential capture and typed failures; existing
15-minute scheduling and initial/manual checksum defaults; initialization gates;
focused partial-failure/rollback tests; OneNote/shortcut disposition; and mode-aware
dependency/host-command documentation while retaining transition dependencies.
The deletion-policy fix remains the shared planner prerequisite already recorded.

These clarify existing accepted scope rather than requiring more prototype tree
or interval tests. OneNote/shortcut live observation, if necessary, stays in phase-7
provider acceptance. Installed native/Flatpak acceptance remains outstanding.
Recommended first implementation slice: configuration/validation and mode-aware
routing with focused compatibility tests, then setup and shared coordinator work.

Changed only Task List.md and these notes. Scoped git diff --check passed, exit 0.
No source/test changes; retained passing gate receipt
`target/task-runs/dev-gate-3gy_byw9/` without rerunning it. No cloud, installation,
service or Git publication action was performed.


### October 15, 2026 — phase-4 simplification without changing acceptance scope

Condensed phase 4 from eleven checklist entries to eight implementation slices:
configuration/routing, shared planner safety, setup/guards, runtime integration,
editor, focused tests, dependency/docs reconciliation and integrated Flatpak.
Linked the accepted contract for detailed invariants instead of repeating them.
Both drive types use one generic sync profile; authentication/identity differences
remain explicit. Reuse shared scheduling, Repair, recovery and UI components;
no second engine, scheduler or per-drive compatibility framework is needed.

Integrated Flatpak results feed phase-7/consolidated acceptance rather than
creating a duplicate suite. Existing scheduling and accepted live evidence are
reused where behavior is unchanged. OneNote/shortcut disposition is part of
setup/filter work. No required guards or acceptance levels were removed.
Clarified that the editor replaces abraunegg controls for new connections while
legacy controls remain available through the already accepted transition release;
legacy dependency/code removal remains phase 6. The deletion-percentage correction
remains explicit shared-planner work before production integration.

Changed only Task List.md and these notes. Scoped git diff --check passed, exit 0.
No source/test inputs changed; retain passing gate
`target/task-runs/dev-gate-3gy_byw9/` without rerun. No cloud, install, service or
publication operations were performed. Next step remains configuration/routing.


### October 15, 2026 — phase-4 configuration and mode-aware engine selection

Implemented the first source slice. `OfflineMirrorConfig.engine` is optional
(`Rclone` or `LegacyOnedrive`); `Connection.onedrive_identity` is optional and holds
only a bounded drive ID plus typed personal/business drive kind, never tokens.
Absent fields are omitted when serialized, preserving old documents and their
engine defaults: OneDrive Offline stays legacy, other providers stay rclone.
Schema version remains unchanged. Validation requires an identity only for rclone
OneDrive Offline, rejects misplaced identities/legacy engines, and checks new
rclone OneDrive local/recovery paths against all connections, including legacy.
Previously valid legacy-only recovery-sharing semantics remain unchanged.

Added mode-aware `RcloneBackend::try_from(&Connection)`; provider-only OneDrive
routing still rejects it, so Online remains onedriver. Legacy planner rejects an
explicit rclone mirror. Editor drafts retain engine/identity during modification;
new OneDrive Offline drafts explicitly select rclone and cannot be saved until
setup supplies a verified identity. Newness is not inferred from an allocated
UUID: drafts can receive IDs before saving; a regression covers this case.
Editor validation checks the proposed full configuration and caller-resolved
native/host-visible work root. Production OneDrive bisync activation remains
blocked until the shared planner/deletion policy and runtime guards are integrated.

Changed core files: src/model.rs, src/config.rs, src/providers.rs, src/app.rs,
src/sync.rs; added tests/onedrive_configuration.rs and three editor regression
cases, plus three English Fluent validation messages. Updated constructor fields
mechanically in src/{sleep,import,mirror_script,teams,controller,directory_preload,
vpn,bisync_coordinator,onedrive_prototype}.rs, the three controlled setup examples,
examples/recover_config_from_units.rs and existing bisync/Teams live-test fixtures.
These constructor updates preserve their previous engine behavior. Updated task
and engineering records; unrelated pre-existing changes remain preserved.

Verification chronology (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42,
source_dirty=true):
- `just fmt`: exit 0. `bash scripts/dev.sh check`: exit 0, final receipt/log at
  `target/task-runs/dev-check-rbhhf7vb/` (earlier incremental checks also passed).
- `cargo test --locked onedrive_configuration`: exit 0, ten focused synthetic
  tests (seven integration tests and three editor tests); both drive types,
  old-format RON compatibility, RON/JSON round trips, invalid/misplaced identities,
  engine routing, legacy/new path overlap, work-root conflicts and Online planning.
- `bash scripts/dev.sh localization-check`: exit 0, receipt/log at
  `target/task-runs/dev-localization-check-xmaqpejm/`. New Fluent inputs unchanged
  since that passing check; no duplicate rerun required.
- First `bash scripts/dev.sh gate`: exit 1 at
  `target/task-runs/dev-gate-ge4fvjkv/`. Formatting and lint passed; sandbox blocked
  just's temporary test script under /run/user/1000. Test recipe did not start.
- Authorized outside-sandbox gate: exit 0 at
  `target/task-runs/dev-gate-m2d3bkia/`. A final editor-ID review then added the
  new-draft regression/fix; final check and focused tests above cover that change.
- Final gate rerun after the editor-ID fix: exit 0, receipt/log at
  `target/task-runs/dev-gate-_94j9dv2/`. Formatting, all-feature lint, Rust
  all-target tests and the maintained Python fixture suite passed.
- Scoped `git diff --check` on source/tests/examples/locale and changed records:
  exit 0; unrelated README.md edits excluded and preserved.

Marked only phase-4 configuration/mode-aware engine selection complete.

Verification level: compilation and automated synthetic/unit coverage only.
No live cloud test, installation, service change, applet reload or publication
was performed. Installed native/Flatpak acceptance remains open. New mirror setup
and runtime activation are subsequent slices; next source task is correcting the
shared bisync deletion-percentage policy before production integration.


### October 15, 2026 — shared bisync deletion-percentage correction

Replaced the ineffective --max-delete 1000 with --max-delete 50 in the shared
Rust planner and both generic/compatibility development guard profiles. Bisync
uses a percentage of each side's prior listing; values above 100 are clamped to
100, unlike one-way sync's count interpretation. Chose the pinned upstream
v1.74.3 default of 50 rather than inventing a new threshold. Exactly 50% remains
allowed; more than 50% aborts. No force override, implicit resync, higher-limit
retry, new UI option or absolute file-count claim was introduced. Backup dirs,
readiness, identity/anchor handling, recovery/retention, locking and existing
SharePoint compatibility/convergence settings are unchanged.

Source references: pinned upstream cmd/bisync/cmd.go defines DefaultMaxDelete=50
and clamps the configured value to 0..100; the accepted OneDrive contract records
that reference and the corrected policy. No rclone minimum change is needed.
No existing cloud evidence was rewritten to claim the new flag was used there.

Changed src/sync.rs (named percentage constant and all-profile planner coverage),
src/bisync_coordinator.rs (extended existing fake failure test),
scripts/lib/mirror_guard_check.py and tests/mirror_guard_check_test.py (matching
profiles and strict fake-command expectations). Added the isolated deletion-limit
case to tests/bisync_coordinator_live.rs, sharing its existing synthetic connection
constructor. Updated script catalog/inventory and the OneDrive contract; task
status is tracked in Task List.md. Pre-existing unrelated edits remain preserved.

Verification (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42, dirty worktree):
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0; receipt/log at
  `target/task-runs/dev-check-ogktlvli/`.
- `cargo test --locked bisync_deletion_limit`: exit 0; planner regression passes
  across all six providers with both backup dirs and no force/resync. The new
  real-engine integration is ignored by this ordinary invocation, not passed.
- `cargo test --locked failed_bisync_records_failure_and_never_starts_retention`:
  exit 0; fake coordinator failure preserves initialization, records failure,
  skips retention and submits the 50% limit without force or implicit resync.
- `python3 tests/mirror_guard_check_test.py`: exit 0, six fake-adapter tests.
- `python3 tests/mirror_guard_roundtrip_test.py`: exit 0, thirteen tests, including
  its existing local-only rclone statistics smoke; no cloud resources used.
- `cargo test --locked --test bisync_coordinator_live bisync_deletion_limit -- --ignored`:
  exit 0, one explicitly run real-engine test, 2.26 s. Temporary alias config and
  local directories only, no network or saved credentials. On both sides, deletes
  of five of ten entries succeed and create five recoverable copies; then three
  of five (60%) abort before deleting peer files. Peer bytes, previous backup
  contents and initialization marker remain; coordinator state is Failed.
- `bash scripts/dev.sh gate`: exit 0; receipt/log at
  `target/task-runs/dev-gate-vjyap8w7/`. Formatting, all-feature lint, Rust
  all-target tests and the maintained Python suite passed. Used already authorized
  outside-sandbox access for just's temporary test script.
- Scoped `git diff --check` on this task's source/tests/scripts/records: exit 0;
  unrelated README.md edits remain excluded and preserved.

Marked the phase-4 shared planner safety item complete.

Evidence levels: compilation and unit/fake-adapter checks plus explicitly run
local real-rclone integration. This is not a cloud deletion test, installed
service update or native/Flatpak acceptance. No applet configuration, live cloud,
installation, service, commit or publication operation was performed. New units
and the installed worker pick up this policy only after the later approved
build/install/service application. Next implementation slice is remote setup
and guards, reusing the tested account-specific prototype behavior.


### October 15, 2026 — clarify Microsoft OAuth reuse and module naming

Operator questioned whether phase-4 reuse of teams_oauth.rs meant direct calls
or using it only as an example. Source inspection confirms that the current
module is SharePoint-specific: TeamsOAuthSetup requires a library URL, config
selects a SharePoint site/library, and completion verifies that library. It cannot
serve OneDrive primary-drive setup unchanged. The prototype already uses its own
account-specific flow rather than importing teams_oauth.

Clarified the implementation plan: extract reusable bounded rclone Microsoft
OAuth create/continue machinery from the existing module and tested prototype
into microsoft_oauth.rs. Reuse that code for both providers. Retain teams_oauth.rs
as a correctly named SharePoint wrapper for its URL/library selection and
verification; OneDrive retains its personal/business primary-drive/identity
policy. Keep credential privacy, remote ownership, cancellation/rollback and
consent behavior explicit during extraction. Avoid a generic OAuth framework or
copying the whole SharePoint flow. The common module is planned, not implemented.

Changed only Task List.md, the accepted OneDrive contract and these notes.
Scoped git diff --check passed, exit 0. No source/test inputs changed; retain
passing gate `target/task-runs/dev-gate-vjyap8w7/` without rerun. No runtime,
cloud, installation, service or publication operation was performed.


### October 15, 2026 — shared microsoft_oauth.rs extraction

Operator confirmed microsoft_oauth.rs (correcting the initial microsoft_auth.rs
wording). Added the shared module and moved fixed rclone Microsoft config
create/continue request construction, remote-name validation, bounded step counts,
new-name absence checks, owned partial-setup rollback, private config dumps and
Microsoft token decoding into it. Target policy stays narrow: SharePoint library,
personal primary drive or business primary drive; no arbitrary OAuth framework,
custom client, shell input or fallback engine. SharePoint retains eight response
steps, OneDrive twelve; requests remain bounded to 300 s and <=1 MiB capture.

Production teams_oauth.rs now delegates transport/rollback to the shared module
while retaining URL/library selection and final verification. OneDrive's gated
prototype delegates the same machinery while retaining its tested primary-drive,
expected-account and personal/business scope rules. Business and SharePoint keep
default scopes; personal alone adds User.Read. teams.rs forwards its compatibility
token helper to the shared decoder and reads verification credentials through
protected capture. No OneDrive production editor/setup/guard activation is claimed.

Promoted the restricted CommandRunner::private_config API from development-only
capture to the shipping native/Flatpak adapters. Its allowlist remains only config
dump/create/update, no launch-only operation, no unbounded output or timeout.
Ordinary output remains redacted. Both streams must be complete and valid UTF-8;
malformed dump shapes cannot authorize creation. Browser-prefix JSON parsing
remains inside the opaque response. SharePoint plain-text completion still needs
full identity verification; truncated responses cannot use that fallback.
Continuation state/answers are sensitive arguments and CommandArg Debug now
redacts them as well. Shared command errors retain typed failure/exit status,
never raw provider output or localized-log classification. Failed cleanup is
explicit. Existing remotes are rejected before creation or rollback.

Changed source: src/microsoft_oauth.rs (new), src/lib.rs, src/process.rs,
src/teams_oauth.rs, src/teams.rs, src/onedrive_prototype.rs and English Fluent
messages. Updated contract, host-command inventory, script output/tooling records,
script catalog and task/evidence records. The SharePoint OAuth test URL now uses
a synthetic example domain. No new Flatpak grants, scripts or runtime helpers.
The engine still owns host rclone config; the host-command inventory records
owner, fixed operations, permission/path split, justification and removal criteria.

Verification (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true):
- `just fmt`: exit 0. Incremental `bash scripts/dev.sh check` runs passed;
  final receipt/log at `target/task-runs/dev-check-l8xv09sx/`.
- `cargo test --locked --lib oauth`: seventeen focused shared/SharePoint setup
  tests passed before final source ordering fix; final rerun recorded below.
  Coverage includes all three targets, scopes, existing names, malformed state,
  truncated captures, private token representations, request Debug redaction,
  cancellation, step bounds and partial rollback/cleanup failure.
- `cargo test --locked --lib teams::tests`: exit 0, eight passed; two live cloud
  tests ignored, not claimed passed. `cargo test --locked --lib onedrive_prototype`:
  exit 0, fifteen passed; two local real-engine lifecycle/timing tests ignored.
  No functional/fault cloud scenarios or tree timing were repeated.
- `cargo test --locked --lib private_config`: exit 0, two passed, one ignored.
  `cargo test --locked --lib private_config_real_encoded_token_roundtrip -- --ignored`:
  exit 0, one explicitly run local-rclone integration, synthetic temporary config,
  no network or saved user config. Final execution took 0.11 s. Confirms encoded
  tokens survive protected capture while ordinary capture and Debug stay opaque.
- `bash scripts/dev.sh localization-check`: exit 0, receipt/log at
  `target/task-runs/dev-localization-check-k8mfm5s6/`. Fluent inputs and production
  message references unchanged since that pass.
- First final gate: exit 101 at `target/task-runs/dev-gate-ov8b1clc/`; formatting
  passed, lint rejected items placed after the new test module. Moved token
  helpers before the test module, then reran check and focused OAuth tests.
- Final `cargo test --locked --lib oauth`: exit 0, all seventeen passed after
  the ordering fix.
- Final `bash scripts/dev.sh gate`: exit 0, receipt/log at
  `target/task-runs/dev-gate-7qckwnmk/`; formatting, all-feature lint, Rust all-target
  tests and the maintained Python suite passed. Used the already authorized
  outside-sandbox access for just's temporary test script.
- Scoped `git diff --check` on changed source/locale/scripts documentation and
  task/evidence records: exit 0; unrelated README.md edits excluded and preserved.

Marked only the shared Microsoft OAuth extraction complete; production OneDrive
setup/identity and runtime guards remain open. Engineering index/status and script
records reflect source reuse without elevating it to live or installed acceptance.

Verification is source/unit/fake-adapter plus local real-engine credential
capture, not a new browser/Microsoft OAuth or installed native/Flatpak acceptance
run. No live remote creation, credential/config mutation, installation, service,
commit or publication operation was performed. Remaining work is production
OneDrive setup/identity and runtime guards, then editor and installed acceptance.


### October 15, 2026 — Common Microsoft identity/profile policy

Accepted the shared policy: mandatory resource/account verification stays separate
from optional display-profile enrichment. SharePoint Online and Offline verify
library ID/type/URL and saved bindings; setup/connection tests can attempt `/me`
with the existing token, without added consent. OneDrive business retains the
accepted primary-drive/expected-UPN/owner checks and default file scopes.
Personal OneDrive retains required profile/account verification and `User.Read`;
removing that check requires an equivalent identity mechanism or a deliberate
requirement change. Optional profile denial cannot authorize skipping any required
identity check. A successful SharePoint label is not an expected-account guard.

Implemented the SharePoint split in src/teams.rs: both Online service and Offline
mirror guards use identity-only verification, while OAuth/setup/connection tests
retain optional labels. Selected-subtree access and saved-binding checks remain.
A small fakeable Graph adapter exercises the real verification sequence without
network or credentials. Microsoft responses are bounded to 64 KiB and redirects
are disabled. Label extraction falls back to mail when UPN is absent/null/empty.

Changed files for this follow-up: src/teams.rs, Task List.md, this completion
record, docs/engineering/onedrive-rclone-offline-contracts.md and its README index.
No script, OAuth scopes, OneDrive prototype logic or Fluent strings changed.
Production OneDrive setup/guards and native/Flatpak acceptance remain pending.

Verification (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true):
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0, final receipt/log at
  `target/task-runs/dev-check-szo875is/` (earlier incremental pass:
  `target/task-runs/dev-check-i63_ycx9/`).
- `cargo test --locked --lib teams::tests`: exit 0, eleven passed, two live
  cloud tests ignored. Three added fake-adapter tests cover optional HTTP
  401/403, transport/parse failure, missing labels, successful label fallback,
  skipped guard profile calls, and required denied/malformed/wrong-ID/type/URL
  metadata stopping before optional enrichment.
- `cargo test --locked --lib onedrive_prototype`: exit 0, fifteen passed,
  two local real-engine tests ignored. Includes business default scopes and
  primary-drive/owner checks without profile access, and required personal
  profile denial stopping identity verification without mutation.
- `bash scripts/dev.sh gate`: exit 0, receipt/log at
  `target/task-runs/dev-gate-ayqx5y8e/`; formatting, all-feature lint, Rust
  all-target tests and maintained Python tests passed. Used authorized
  outside-sandbox access for just temporary scripts. No check failure occurred.
- Scoped `git diff --check` on the five changed files: exit 0. Existing
  unrelated work, including README.md edits, was preserved.

Marked the common-policy follow-up complete. Verification is compilation and
unit/fake-adapter coverage; no live Microsoft sign-in, installed service, panel
or final native/Flatpak acceptance was performed. No remote/config mutation,
installation, service changes, commit or publication. Next: production OneDrive
setup and required identity/subtree guards using the shared OAuth module and
these accepted personal/business verification rules.


### October 15, 2026 — Production OneDrive remote setup and guards

Started the phase-4 remote setup/guards task after approval of the common
Microsoft identity policy. Promoted prototype identity/authentication code into
src/onedrive.rs; the development harness now delegates to that production module
while keeping scenario ownership, protocol receipts and its cm_proto_ restriction.
The production module uses the model drive-type enum, shared protected Microsoft
OAuth transport, required personal account/profile matching and business
primary-drive/expected-UPN/owner matching. Production names are no longer limited
to the prototype prefix. Tokens/configuration remain private and errors are typed
protocol values with localized safe Display messages.

Added RemoteSetup/VerifiedRemote, owned-new-remote creation, read-only existing
selection and saved previous_identity comparison for replacement/reconnection.
Wrong-account/drive or denied/incomplete metadata prevents acceptance; a failed
new setup rolls back only its proven-new remote, reporting cleanup failure
explicitly. Existing remotes and configuration are not edited by this API.
Editor wiring and the disabled/unaccepted draft presentation remain in the next
editor/lifecycle tasks.

Added serialized MirrorGuardBinding and restricted --verify-onedrive-mirror
entry point. Bindings check authoritative enabled saved UUID, remote/subpath,
local/recovery paths and drive/type before any remote request. Scheduled/manual
coordinator modes run this guard before input preparation. It refreshes via
bounded rclone root stat, reads private configuration, requires the saved primary
Graph drive/type and checks a selected folder via Graph plus rclone directory
stat. No recurring profile request or added business User.Read scope. Selected
notebook/shortcut roots and foreign/malformed folder metadata are rejected.
Alternate roots/custom clients/tenant endpoints and exposed notebooks are refused.

Enabled the shared production planner/coordinator for explicit rclone OneDrive
mirrors with valid identity. Legacy and Online routing stay unchanged. Target
binding hashes isolate Preview/initialization state; subtree recovery stays
inside that subtree and outside filters. Reuse UUID anchors, recovery, locking,
retention and 50-percent deletion limits. Initial Preview creates no OneDrive
anchor; Initial Sync creates it and must pass one checksum check before marking
initialization complete. Normal sync has no SharePoint convergence/full checksum
pass. App lifecycle/status/Repair dispatch and installed acceptance remain pending.

Resolved the cloud-item source policy in the contract: backend-hidden notebooks,
.one/.onepkg/.lnk/.url filter exclusions, and rejection of package/remoteItem mirror
roots. Nested cloud shortcuts retain rclone resolution; no full descendant
metadata inventory or preservation of cloud shortcut relationships is claimed.
Ordinary source folders are the supported setup; separate live cloud-item
acceptance is pending. Verified the file-permission folder API and shared facet
against Microsoft docs and the OneNote/shared-item behavior against rclone docs
and pinned v1.74.3 backend source. Used the documented root:/path item endpoint
without a content/children suffix; path components are URL-encoded independently.

Changed source: src/onedrive.rs (new), src/onedrive_prototype.rs, src/lib.rs,
src/main.rs, src/sync.rs, src/bisync_coordinator.rs, English Fluent messages,
tests/onedrive_setup.rs (new) and tests/onedrive_configuration.rs. Updated task,
contract/index, host-command inventory and script caller/receipt documentation;
script actions and receipts did not change. Existing unrelated edits preserved.

Verification chronology (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42,
source_dirty=true):
- Initial check failed at target/task-runs/dev-check-f2bkugf5/: moved prototype
  helpers needed CommandRequest import; corrected the import and removed the
  redundant CommandOutput import. Check then passed at dev-check-52ehbuxf/.
- New integration-test compilation failed at dev-check-rcgl0jld/: tests assumed
  public raw command argument accessors. Replaced these with the runner's
  sanitized command representation; no new credential-exposing accessor added.
- Check passed at dev-check-ft4z9ex8/; focused integration tests passed seven
  configuration tests plus nine setup/guard tests at that intermediate point.
- First cargo test --locked --lib onedrive: 31 passed, one failed, two ignored.
  The new checksum test incorrectly assumed the subcommand preceded global
  rclone timeout flags. Corrected the assertion to allow the maintained prefix.
  This was a test assertion failure, not failed checksum enforcement.
- Added cleanup-failure/configuration-rejection checks and folder URL encoding
  coverage. Incremental check passed at dev-check-1twaqy12/; final verification
  receipts follow below.

- Final `bash scripts/dev.sh check`: exit 0, receipt/log at
  `target/task-runs/dev-check-_otkp0ut/`.
- `cargo test --locked --lib --test onedrive_setup --test onedrive_configuration onedrive`:
  exit 0; 33 library tests, seven configuration tests and eleven production
  setup/guard tests passed (51 total). Two explicitly ignored local-engine
  library tests were not counted as passes by this invocation.
- `cargo test --locked --lib onedrive_prototype_local_shared_coordinator_lifecycle -- --ignored`:
  exit 0, one explicitly run local real-rclone lifecycle passed in 15.92 seconds;
  synthetic temporary alias filesystem and Graph fake, no live account or user data.
- `bash scripts/dev.sh localization-check`: exit 0, receipt/log at
  `target/task-runs/dev-localization-check-nelsp1gr/`.
- The first final-gate tool call was interrupted before creating a task run.
  After the operator asked to check, confirmed source changes and the passing
  receipts were intact and started the missing gate. No passing focused check
  was rerun solely because of the interruption.

- Final `bash scripts/dev.sh gate`: exit 0, receipt/log at
  `target/task-runs/dev-gate-jnvev19m/`. Formatting, all-feature lint, Rust
  all-target tests and maintained Python tests passed. Used the previously
  authorized outside-sandbox access required by just temporary scripts.
- Final scoped `git diff --check` on changed source, tests, localization and
  task/engineering/script records: exit 0. Unrelated README.md edits excluded.

Marked only the remote setup/guards backend task complete. This verifies source,
unit/fake adapters and one temporary local real-engine lifecycle. The newly added
Graph selected-folder metadata path and production guard launch have not been
accepted against live accounts or an installed native/Flatpak package. Nested
cloud shortcuts are not exhaustively inventoried; the documented support boundary
remains explicit. All original live sample/scenario/fault evidence is retained,
not reclassified as new production acceptance.

No actual user remote, credential/configuration, cloud file, service, installed
applet, commit or publication was changed. Next: shared app runtime/lifecycle
integration and Add/Modify editor wiring, then the planned installed acceptance.


### October 15, 2026 — SharePoint source module naming

Clarification: the operator expected the agreed rename to be performed; the agent
had misread the previous message as a report that it was already done. Inspection
confirmed the old module files and callers were still present. Per the existing
approval, renamed src/teams.rs to src/sharepoint.rs and src/teams_oauth.rs to
src/sharepoint_setup.rs. The shared Microsoft transport remains microsoft_oauth.rs.

Updated all Rust module imports/callers, SharePoint library/setup/result types,
identity error variants and internal editor setup fields/messages/functions.
SharePointLibraryIdentity retains serde name TeamsLibraryIdentity; Provider::Teams
and Connection.teams_identity remain unchanged. Service guard flags, unit and
remote identities, Fluent keys and user profile settings are unchanged. Updated
maintained contract/source inventory paths, preserving historical completion
notes. Extended the existing compatibility test to load the legacy named RON
identity and assert unchanged JSON provider/field names. No migration or aliases
to retired source modules are needed for saved settings.

Changed files: renamed source modules; src/app.rs, src/lib.rs, src/main.rs,
src/config.rs, src/model.rs, src/providers.rs, src/sync.rs,
src/bisync_coordinator.rs, src/process.rs, src/onedrive_prototype.rs,
examples/recover_config_from_units.rs, tests/teams_mirror_guard_live.rs,
Task List.md, this record and the engineering contract/index/host-command inventory.
Verification receipts follow after the required focused checks and final gate.

- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0, receipt/log at
  `target/task-runs/dev-check-iu9t3han/` (HEAD
  217640a80d0591a79490cc446c5fc9d160bbca42, source_dirty=true).
- `cargo test --locked --lib sharepoint`: exit 0, twenty passed; two live
  SharePoint tests ignored, not claimed passed.
- `cargo test --locked --lib teams_identity_is_required_and_old_connections_still_load`:
  exit 0, one compatibility test passed, including old named RON identity and
  unchanged JSON Provider::Teams/teams_identity fields.
- `cargo test --locked --bin cosmic-ext-applet-mounter teams_`: exit 0,
  three editor/verification/service-routing tests passed.
- Repository source/current-record scan found no retired module callers or old
  source paths; deliberate history/archive references were excluded. Scoped
  diff whitespace check passed. No Fluent content or keys changed, so the
  message IDs are unchanged; no localization content was added by the refactor.

- Final `bash scripts/dev.sh gate`: exit 0, receipt/log at
  `target/task-runs/dev-gate-o9tuv3xj/`; formatting, all-feature lint, Rust
  all-target tests and maintained Python tests passed. Used authorized
  outside-sandbox access for just temporary scripts. No verification failure.
- Final scoped `git diff --check`: exit 0; unrelated README.md edits preserved.

Marked the source rename complete. Verification is compilation and existing
unit/fake-adapter compatibility coverage, not a new cloud sign-in, installed
service, rendered applet or final-package acceptance run. No profile rewrite,
remote/config mutation, installation, service change, commit or publication.
The next implementation task remains shared runtime integration and editor wiring.


### October 15, 2026 — OneDrive shared runtime integration

Connected explicit rclone OneDrive Offline mirrors to the existing shared manual
coordinator, readiness/VPN policy, units, scheduling status and Repair lifecycle.
Legacy OneDrive monitor profiles and Online onedriver retain their existing routes.
Save installs units paused, refuses external ownership, pauses existing timers
before replacement and restores the previous service after partial timer failure.
Start requires initialization and resource/readiness guards, observes the enabled
active timer and rolls back a failed start. Pause observes both units disabled and
stopped without clearing failures; Repair retains conditional failed-state reset.
The default remains 15 minutes. Initial and manual sync perform one checksum
verification; scheduled sync does not acquire SharePoint convergence workarounds.

OneDrive explicit Preview uses two bounded structured filtered lsjson inventories
for estimated differing file paths, not exact planned transfers or conflicts.
Requests are read-only, limited to 30 seconds/4 MiB each, inside the coordinator
lock. Metadata errors produce unknown counts. Atomic UUID-bound reports expire
three minutes after Preview, survive bounded async reads without resetting age,
and are invalidated by subsequent operations. No background dry-run was added.
The existing other-provider/legacy live-log counters were not rewritten.
Jiff (already locked transitively) is now a direct dependency for RFC3339 offset
and fractional-time parsing; no locked dependency versions changed.

Files changed for this task: src/app.rs, src/bisync_coordinator.rs,
src/pending_count.rs, Cargo.toml/Cargo.lock, the English Fluent catalog,
Task List.md, this record, and the engineering OneDrive, offline Repair, host
command inventory and index records. Earlier uncommitted work is preserved.
No maintained scripts were added or changed by this runtime task.

Verification chronology (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42,
source_dirty=true throughout):

- Initial check failed: Fluent count interpolation needed an explicit string;
  corrected it. Receipt: target/task-runs/dev-check-u1hpjx4e/.
- Intermediate check passed: target/task-runs/dev-check-f21gmpxd/.
- First new app test run: seven passed, one failed because the fake omitted
  UnitController's ValidateLoaded action. Corrected the fake queues, including
  the partial-failure injection phase; no production workaround was added.
- Subsequent check passed: target/task-runs/dev-check-1ofp2cza/.
- Broader `cargo test --locked --lib onedrive`: exit 0, 37 passed, three ignored.
- Final freshness handling preserves the original Preview timestamp. `just fmt`:
  exit 0; `bash scripts/dev.sh check`: exit 0, receipt
  target/task-runs/dev-check-kcllqfeh/.
- `cargo test --locked --bin cosmic-ext-applet-mounter onedrive_runtime`: exit 0,
  seven lifecycle/status/ownership/rollback/structured-summary tests passed.
- `cargo test --locked --lib onedrive_runtime`: exit 0, four passed, one local
  engine test ignored in this ordinary run (explicit execution below).
- `cargo test --locked --bin cosmic-ext-applet-mounter offline`: exit 0,
  19 shared and legacy offline regression tests passed.
- `cargo test --locked --lib onedrive_runtime_local_preview_metadata_uses_real_rclone -- --ignored`:
  exit 0, one real rclone local metadata test passed with temporary directories,
  including filtered items. No cloud, user configuration or services were used.
- `bash scripts/dev.sh localization-check`: exit 0, receipt
  target/task-runs/dev-localization-check-oy6ommh6/.

- Final `bash scripts/dev.sh gate`: exit 0; receipt/log at
  target/task-runs/dev-gate-qo0ofhym/. Formatting, all-feature lint, all-target Rust
  tests and maintained Python tests passed. Used the authorized outside-sandbox
  execution needed by just's temporary scripts. Final scoped `git diff --check`
  passed, excluding unrelated README.md edits.

Marked shared runtime source integration complete. Verification covers compilation, synthetic/fake-adapter
behavior and one temporary local-engine metadata probe. Production cloud guards,
installed services, rendered applet and final Flatpak acceptance remain pending.
No remote, cloud data, user service, installed applet, commit or publication changed.
Next: Add/Modify editor remote/auth controls and the planned installed acceptance.


### October 15, 2026 — OneDrive Add/Modify editor integration

New OneDrive Offline drafts now use personal/work-or-school choice, an ephemeral
expected-account field, mode-aware rclone discovery, Connect new remote and Verify
selected remote. Both actions delegate to the production RemoteSetup backend and
shared Microsoft OAuth; there is no second OAuth implementation. Existing remotes
are inspected without config replacement. Reconnect takes previous_identity from
saved configuration, rejects a different drive/type and preserves saved settings
on failure. Changed remote, folder, type or account clears draft verification.
A failed setup clears draft identity and disables the draft; a successful retry
never silently enables a disabled draft. Save still installs runtime units paused.
Legacy Offline authentication/manual handoff and Online onedriver remain separate.

Added cancellable setup state, generation and editor/saved-binding checks on
completion, blocked Test/Save while setup is pending, and checks rejecting stale
OneDrive Test/Save results. Cancelled or edited setup results are not accepted;
cleanup errors remain visible. Account input is not serialized into connections.
Discovery now distinguishes personal/business primary drives from SharePoint
libraries. Saved rclone OneDrive remote references are protected from deletion,
while Online/legacy account labels are not treated as rclone references.
No setup or test synchronizes data, starts services or installs the applet.

Files changed for this task: src/app.rs, one synthetic remote fixture in
src/app/sftp.rs, i18n/en/cosmic_ext_applet_mounter.ftl, Task List.md, this record,
and the OneDrive engineering contract/index. No maintained scripts changed.

Verification (HEAD 217640a80d0591a79490cc446c5fc9d160bbca42,
source_dirty=true):

- Early `bash scripts/dev.sh check` failures were test-fixture integration issues:
  the SFTP synthetic remote needed the new drive-type field, then new app tests
  needed Application and FakeCommandRunner imports. Corrected all three; receipts
  target/task-runs/dev-check-tme7gpv_/, dev-check-0vrynoer/ and dev-check-bpz6p1p1/.
- `just fmt`: exit 0. Final `bash scripts/dev.sh check`: exit 0; receipt/log
  target/task-runs/dev-check-yirxkys0/.
- `cargo test --locked --bin cosmic-ext-applet-mounter onedrive_editor`: exit 0,
  six tests passed: both-drive create/select through production backend with fake
  rclone/Graph and pre-cancellation; discovery/routing; draft invalidation and
  account serialization; authoritative reconnect and failed-draft behavior;
  cancellation/late completion/blocked Save; referenced-remote protection.
- `cargo test --locked --bin cosmic-ext-applet-mounter rclone_remote_`: exit 0,
  six discovery/matching/removal regressions passed (one overlaps the new tests).
- `bash scripts/dev.sh localization-check`: exit 0; receipt/log
  target/task-runs/dev-localization-check-c59y3wla/.
- Scoped `git diff --check`: exit 0, excluding unrelated README.md edits.

- Final `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-0ipai5h1/. Formatting, all-feature lint, all-target Rust
  tests and maintained Python tests passed. Used authorized outside-sandbox access
  for just temporary scripts. Marked the Add/Modify editor source task complete.

These are compilation and synthetic/fake-adapter tests,
not browser sign-in, rendered controls, installed native services or final-package
acceptance. No cloud remote/configuration, user data, service, installation, commit
or publication was changed. Installed native/Flatpak acceptance remains pending.


### October 15, 2026 — Phase-4 focused-test consolidation

Reviewed the focused-test checklist against the committed implementation rather
than duplicating accepted live scenarios. Existing configuration, routing, units,
identity/anchor, malformed/partial-state, setup rollback, item-filter, Preview,
checksum and unchanged Online tests are mapped in the existing OneDrive contract.
Retained earlier native cloud/prototype and temporary local-engine evidence with
its original acceptance boundaries; no live scenario was repeated.

Added four tests for remaining integration cases:

- Successful replacement/reconnect for personal and business accepts the saved
  primary-drive identity and never addresses or deletes the retained old remote.
- Cancellation of the actual token after remote creation makes both setup flows
  stop before Graph access and delete only the newly-created remote. The adapter
  asserts that rollback requests use fresh uncancelled tokens.
- Late successful Test/Save validation after a remote edit cannot accept identity,
  validate the changed draft or save any configuration.
- Authentication completion is rejected when authoritative saved identity changed
  during the operation; the newer saved identity remains intact.

Files changed: tests/onedrive_setup.rs, the test module in src/app.rs,
Task List.md, this record, docs/engineering/onedrive-rclone-offline-contracts.md
and docs/engineering/README.md. No production behavior, scripts, Fluent strings,
dependencies or installed state changed. Unrelated README.md edits were preserved.

Verification (HEAD 5d312f5ca329af23619eefb86f599316a04988ef,
source_dirty=true):

- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0; receipt/log
  target/task-runs/dev-check-njz2nbpl/.
- `cargo test --locked --test onedrive_setup onedrive_focused_`: exit 0,
  two tests passed, exercising both drive types.
- `cargo test --locked --bin cosmic-ext-applet-mounter onedrive_focused_`:
  exit 0, two editor race tests passed.
- Existing unchanged checks are retained from the previous implementation receipt
  target/task-runs/dev-gate-0ipai5h1/ and its preceding completion entry; the
  final gate below also covers the complete current suite. No localization check
  repeated because Fluent/UI content is unchanged.

- Final `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-aayjw5vv/. Formatting, all-feature lint, all-target Rust
  tests and maintained Python tests passed. Used the authorized outside-sandbox
  access needed for just temporary scripts. No check or test failure in this task.
- Final scoped `git diff --check`: exit 0; unrelated README.md edits excluded.

Marked the automated focused-test checklist complete. These tests use synthetic
accounts, fake adapters and local state only. Installed native lifecycle, browser/consent, rendered applet and
integrated Flatpak acceptance remain outstanding. No cloud remote, user service,
installation, commit, push or publication was performed by this test task.


### October 16, 2026 — Dependency and documentation reconciliation

Kept the rclone minimum at 1.74.3 in diagnostics, the maintained installer and
Debian recommendations. Kept onedriver >= 0.15.0 for Online OneDrive, onedrive
>= 2.5.10 and optional GUI dependencies for legacy Offline profiles. No package
recommendation, manifest permission or shipped helper was removed; removal stays
in its later task. Installer defaults/--all selections are unchanged; help now
explains new rclone mirrors versus retained legacy engine/helper selections.

Fixed dependency warnings to use configured provider, mode and resolved engine.
The inventory still lists supported tools; only configured engine requirements
become missing-tool warnings, including disabled saved profiles. New OneDrive
Offline needs rclone, Online needs onedriver/FUSE, legacy Offline needs onedrive;
other providers retain rclone and Online FUSE requirements. NetworkManager/systemd
remain common requirements. No startup version/help process was added. The actual
OneDrive missing-rclone error now gives localized minimum-version guidance from
the typed failure, without inspecting raw error text.

Updated dependency installation, README OneDrive engine/setup paragraphs, docs
index, Microsoft auth guide and Flatpak guide. Custom-client instructions are now
explicitly scoped to SharePoint/legacy profiles: new OneDrive Offline rejects
custom client/tenant/root settings. Documented personal required profile access,
business drive-ownership policy and optional SharePoint labels. Checked the
upstream rclone OneDrive configuration/custom-client page while reconciling scope:
https://rclone.org/onedrive/ . The applet-specific restrictions come from current
src/onedrive.rs and src/microsoft_oauth.rs, not from upstream rclone restrictions.
Source/test implementation is distinguished from pending installed acceptance.

Updated the host-command inventory and script catalog/inventory, including the
legacy command/helper ownership and removal boundary. Earlier unrelated README
edits and all uncommitted focused-test work were preserved.

Changed files for this task: src/diagnostics.rs, src/controller.rs,
src/onedrive.rs, English Fluent catalog, scripts/install-dependencies.sh,
tests/dependency_installer_test.py, README.md, docs/dependency-installation.md,
docs/index.md, docs/microsoft-cloud-auth-setup.md, packaging/flatpak/README.md,
scripts/README.md, engineering README/OneDrive contract/host-command inventory/
script-tooling inventory, Task List.md and this record.

Verification (HEAD 5d312f5ca329af23619eefb86f599316a04988ef,
source_dirty=true):

- Initial `bash scripts/dev.sh check`: exit 101; used a nonexistent Connection
  access_mode accessor in the new diagnostic policy. Corrected this to typed
  ConnectionMode matching. Receipt: target/task-runs/dev-check-uyepopsd/.
- `just fmt`: exit 0. Subsequent check: exit 0; receipt/log
  target/task-runs/dev-check-roy4cf93/.
- `cargo test --locked --lib dependency_warnings`: exit 0, two passed, including
  empty/mixed configuration, explicit rclone, explicit/implicit legacy and Online
  routing. No host tools or services are invoked by these synthetic checks.
- `cargo test --locked --lib onedrive_missing_rclone_guidance`: exit 0, one typed
  diagnostic/minimum-version test passed.
- Initial pytest invocation could not run because pytest is absent (exit 1).
  Used the file's standard-library unittest runner, not a package installation:
  `python3 -m unittest discover -s tests -p dependency_installer_test.py -v`:
  exit 0, ten tests passed, including --rclone-only isolation and retained --all.
- `bash -n scripts/install-dependencies.sh` and its `--help`: exit 0 each;
  help/syntax checks only. No installing option was executed.
- `bash scripts/dev.sh localization-check`: exit 0; receipt/log
  target/task-runs/dev-localization-check-esg5puot/.
- Scoped `git diff --check`: exit 0; excluded preexisting unrelated README
  whitespace. No unrelated prose was reformatted or reverted.

- Final `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-2wve0nue/. Formatting, all-feature lint, all-target Rust
  tests and maintained Python tests passed, with authorized outside-sandbox access
  for just temporary scripts. Final scoped `git diff --check`: exit 0.

Marked dependency/docs reconciliation complete. This is source/fake-adapter and
documentation evidence, not installation, system-package upgrade, browser consent, native service/UI or
final Flatpak acceptance. No cloud, credentials, services, commit, push or release
publication changed. Next: planned installed integration acceptance, with remaining
legacy deprecation work tracked separately in phase 5.


### October 16, 2026 — Integrated Flatpak acceptance preparation

Started the integrated acceptance task without re-running the accepted native
cloud scenarios. Reviewed package procedure, existing source/prototype manifests,
build/reuse guards and installed-state boundaries. The source scaffold still
selects v0.5.2, while new implementation tests/diagnostics/docs are uncommitted;
building that old source or accepting a transport probe would not test the current
integration. The source wrapper correctly requires clean immutable inputs. No
such gate was bypassed and no build, installation or launch was performed.

Added --preflight to the maintained build-flatpak entry point, with bounded
read-only observations and private schema-1 summary/preflight.json: source dirtiness
and pin blockers, tools, runtime/SDK/extension refs, BaseApp, installed user/system
application metadata and overrides. Success means inspection completed, not that
build or acceptance passed. Fixed an existing candidate/reuse ref bug exposed by
the live inspection: BaseApp is app/com.system76.Cosmic.BaseApp, not runtime/... .
Required SDK extension refs now participate in build/reuse provenance too; no
runtime, BaseApp or permission was changed.

Chronology/evidence:

- Initial live preflight target/task-runs/flatpak-preflight-ufc21brf/ completed
  successfully but falsely classified installed BaseApp as unavailable because of
  the old runtime-ref assumption. Retained this report as failure/fix evidence.
- Corrected preflight target/task-runs/flatpak-preflight-tzwup1c8/; final branch-
  explicit preflight target/task-runs/flatpak-preflight-f28r8mmf/: exit 0 each.
  Final receipt observes all four refs available: Freedesktop Platform/SDK and
  rust-stable extension 25.08, COSMIC BaseApp stable. Build readiness is false only
  because checkout is dirty and Git manifest source lacks a full immutable pin.
- Existing user app ref: app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master,
  installed OSTree commit d0f101c60a1e4fb78700184e3dac11c2544904989c3e171ec12d73e07d7138d7.
  No system installation. User global overrides exist; no application-specific
  override was observed. Full metadata/overrides remain in the private receipt;
  candidate effective-grant review is still required before installed acceptance.
- `python3 -m unittest discover -s tests -p flatpak_wrapper_test.py -q`: exit 0,
  ten tests (including two new preflight tests) passed before the typed-ref fix.
- `python3 -m unittest discover -s tests -p 'flatpak_*_test.py' -q`: exit 0,
  25 wrapper/reuse/inspection tests passed with typed BaseApp/extension checks.
- After adding explicit preflight branch/architecture receipt fields,
  `python3 -m unittest discover -s tests -p flatpak_wrapper_test.py -k preflight -q`:
  exit 0, two passed. Help and scoped whitespace checks passed.

Prepared the concrete operator sequence in the existing OneDrive contract:
source commit/Cargo-input check, immutable candidate source/packaging pin, one
shipping-feature build, payload/effective-grant review, separately approved install
with rollback identity and one runtime owner, both disposable account flows and
one installed initial/checksum/scheduling/Pause/cleanup smoke. No whole user drives
or duplicate business fault suite. Longer build commands will be supplied against
actual pinned inputs; no placeholder was executed as a build.

Files changed for this preparation: scripts/build-flatpak.sh,
scripts/lib/flatpak_tasks.py, tests/flatpak_wrapper_test.py, packaging/flatpak/README.md,
scripts/README.md, script-tooling inventory, script-output contracts, OneDrive
contracts, Task List.md and this record. Earlier uncommitted work is preserved.
Final `bash scripts/dev.sh gate`: exit 0, receipt/log
`target/task-runs/dev-gate-qjnhjpw2/` (HEAD
5d312f5ca329af23619eefb86f599316a04988ef, source_dirty=true). Formatting,
all-feature lint, all-target Rust and maintained Python tests passed. Used
authorized outside-sandbox access for just temporary scripts. Final scoped
`git diff --check` passed, excluding preexisting unrelated README whitespace.

Acceptance remains open: commit/pinning, build and installed/operator steps have
not run. AGENTS.md requires separate approval for
commits and package installation; the current commit request applied to earlier
work, not these subsequent changes. No cloud/credentials/services/data changed.


### October 16, 2026 — Flatpak candidate source pin and Cargo validation

Pinned the source manifest to accepted implementation commit
cf5da32f622ac7b03efb42facba9e9102da365ca, replacing v0.5.2. This is the committed
OneDrive integration plus focused tests, dependency/docs and preflight tooling;
no release tag or version metadata was advanced. Runtime/BaseApp, grants,
shipping build features and payload paths are unchanged. Source and packaging
revisions remain distinct: the packaging update still needs its own commit.

Validated the existing Cargo source list against Cargo.lock instead of needlessly
regenerating it. All 620 registry archives match package versions, canonical
archive URLs and SHA-256 values; their vendor checksum metadata and registry
replacement config match. Existing Rust tests cover all 12 dependency Git pins
and locked source selectors. Cargo.toml, Cargo.lock and cargo-sources.json match
the cf5da32 source snapshot exactly (`git diff --exit-code cf5da32 -- ...`: exit 0).

Added two Python source-contract checks and strengthened the Rust manifest
contract to require a full immutable commit and reject tag/branch source refs.
The first Python run passed registry coverage but failed the checksum comparison:
Git vendor metadata has package=null, not a registry archive checksum. Corrected
the test to distinguish those supported cases; both checks then passed. No change
to generated Cargo data was needed.

Pinned-source preflight exposed another pre-existing wrapper gap: the tracked
Cargo source JSON includes 725 inline and 52 shell entries, previously rejected
as unsupported. Added typed validation for inline contents/filenames and native
shell command arrays, matching the existing script-source policy. The entire
tracked JSON (including commands/contents) remains fingerprinted. Added synthetic
success/fingerprint and malformed-source rejection tests. This is build-time
manifest support, not runtime UI shell execution or a new permission grant.

Verification (HEAD cf5da32f622ac7b03efb42facba9e9102da365ca, source_dirty=true):

- `just fmt`: exit 0; `bash scripts/dev.sh check`: exit 0, receipt/log
  target/task-runs/dev-check-wbr3cwd6/.
- Initial Python source check: one passed, one failed as described above;
  corrected `python3 -m unittest discover -s tests -p flatpak_cargo_sources_test.py -v`:
  exit 0, two passed.
- `cargo test --locked --test flatpak_packaging_contract`: exit 0, four passed:
  immutable source/payload/features/runtime identity, grants, dependency Git
  source selectors and existing sandbox module source pins.
- `python3 -m unittest discover -s tests -p flatpak_wrapper_test.py -k sources -q`:
  exit 0, three passed, including generated inline/shell metadata fingerprints
  and invalid types rejected before the builder.
- Pinned preflight target/task-runs/flatpak-preflight-mnbwt9fv/: exit 0,
  recorded unsupported generated sources as a blocker (not an acceptance pass).
- Corrected preflight target/task-runs/flatpak-preflight-nxc0skxl/: exit 0,
  633 immutable source pins inspected (app plus 12 dependency Git pins and
  620 archives), all four runtime/BaseApp refs available. Its only build blocker
  is dirty packaging checkout. This does not verify remote source availability.
- A gate was started before that preflight exposed the wrapper gap; a final gate
  after the source-validation fix is required below rather than treating an
  in-flight run as validation of later edits.

Changed files: source manifest, tests/flatpak_packaging_contract.rs,
tests/flatpak_cargo_sources_test.py, scripts/lib/flatpak_tasks.py,
tests/flatpak_wrapper_test.py, packaging guide, script catalog/inventory/output
contract, OneDrive contract/index, Task List.md and this record. No desktop or
Fluent strings changed. No package build/install, cloud/service/data mutation,
commit, push or publication occurred in this pin/validation task.
- The earlier in-flight gate completed successfully at
  target/task-runs/dev-gate-792f4fvm/; not used as the final check of later edits.
- Final `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-xl5ldun_/. Formatting, all-feature lint, all-target Rust
  tests and maintained Python tests passed. Used authorized outside-sandbox access
  for just temporary scripts. Final `git diff --check`: exit 0.

Marked source pin/Cargo-input validation complete. Packaging edits are uncommitted;
clean packaging source and pinned-source availability remain build prerequisites.
Integrated installed acceptance remains outstanding.


### October 16, 2026 — Commit and source-based Flatpak build

Operator explicitly approved committing the packaging update and building the
candidate. Read-only GitHub master lookup returned
4b5bb2824fdc566f0e1d5e7d9153a7b63f68cf21, older than the selected local app source
cf5da32f622ac7b03efb42facba9e9102da365ca. No push is authorized or needed for a
local candidate: added build-only --local-source-repo to the maintained wrapper.
It requires that exact commit in a real Git repository and gives only the builder
child an environment-scoped URL rewrite for the selected app source. The manifest
and source pin remain unchanged; working-tree files are not used as app source.
This transport is recorded separately in the candidate receipt. Source packaging
still must be clean/pinned, and reuse keeps original provenance without needing
the local repository. No global Git configuration was changed.

Changed scripts/lib/flatpak_tasks.py, the shared task runner's optional child-env
argument, focused wrapper fixtures and related catalog/inventory/contracts/guide.
These necessary build-transport changes are included with the approved pending
packaging commit. No runtime app source or dependency version changed.

Focused checks before commit/build:

- `python3 -m unittest discover -s tests -p flatpak_wrapper_test.py -k local_source_transport -q`:
  exit 0, two tests passed: exact-pin/provenance and builder-only environment;
  mismatched source commit prevents builder/output creation.
- `python3 -m unittest discover -s tests -p task_output_test.py -q`: exit 0,
  six task logging/timeout/failure tests passed (expected negative-case excerpts).
- Final gate, commit and build receipts follow. Previous source-pin/full-gate
  evidence remains in the preceding entry. No build failure is hidden by reuse.

- Final transport/packaging `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-hnmyltem/ (source cf5da32f622ac7b03efb42facba9e9102da365ca,
  source_dirty=true). Formatting, all-feature lint, all-target Rust and maintained
  Python tests passed. Authorized outside-sandbox access used for just scripts.
  Source pin/Cargo checks are reused from the preceding accepted receipt.

- Commit: de8d7796047656cc088d70a96047b62448c09cff,
  `Pin Flatpak candidate source and validate Cargo build inputs`; root master
  clean after commit. Created clean detached packaging worktree
  /tmp/cloud-mounter-flatpak-packaging-de8d779 at that commit.
- Ran maintained `bash scripts/build-flatpak.sh` with that worktree's manifest,
  `--source-sha de8d7796047656cc088d70a96047b62448c09cff --arch x86_64 --branch master`,
  root repository as `--local-source-repo`, output
  target/artifacts/candidates/flatpak-de8d779-x86_64-master and log directory
  target/task-runs/flatpak-candidate-de8d779. Exit 0, status succeeded;
  summary.json, task.log and artifacts.json retained in that log directory.
- Builder fetched the exact app commit cf5da32f622ac7b03efb42facba9e9102da365ca
  through receipt-recorded local Git transport. Shipping release features,
  locked/offline Cargo build, Freedesktop 25.08 and COSMIC BaseApp stable retained.
  Four installed build/runtime refs and immutable source/input hashes recorded.
- Export and private bundle-import inspection agree on
  app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master, OSTree commit
  cfa15ec80e68374319cda2749291939880badb88bcdc86b64190bc4f37a28e57,
  and complete metadata. Bundle size 13,511,896 bytes; SHA256
  85ad69c87594157d4f7736e9e23e76c738d5ea3572e234df4e7d53b627e324ad.
  Built metadata contains scoped filesystem paths and named D-Bus policies;
  no blanket home/host filesystem or wildcard bus grant is declared.

Verification level: final candidate built/exported and inspected, not installed
or exercised. No build failure occurred. No installation, app launch, cloud data,
service, push or publication action performed. Payload and effective installed
grants, then integrated disposable mirror/UI acceptance, remain outstanding.
Task/evidence records updated after the build; these result-only edits are not
part of the committed packaging source or candidate and do not require rebuilding.

### October 16, 2026 — Approved Flatpak installation and acceptance start

Operator approved installing and testing the exact candidate. Initial observations
found the native /usr/bin applet running, with a higher-precedence user desktop
launcher selecting it despite an older user Flatpak installation. Operator
confirmed mirror syncs paused, windows closed and applet removed from the panel.

The build wrapper intentionally has no install operation. Added maintained
scripts/install-flatpak.sh and scripts/lib/flatpak_install.py with focused fake
adapters, catalog/inventory/output contract and packaging instructions. Installer
reuses the existing full schema-2 candidate/source/input/hash/repository/bundle
verification, rejects running owners, requires matching existing user ref, retains
previous commit and installs only the bundle without dependency/related updates.
It compares installed identity and complete metadata; no build/launch/stop/service
or automatic rollback. Optional panel-entry activation retains the original
launcher privately before atomic replacement with the installed Flatpak export.

Focused `python3 -m unittest discover -s tests -p flatpak_install_test.py -q`:
exit 0, five tests passed, covering candidate rejection, both running owners,
partial failure/post-install mismatch, rollback records and launcher preservation/
symlink refusal. First gate target/task-runs/dev-gate-3df9t36h/ passed before the
additional launcher capability; final gate target/task-runs/dev-gate-c03vn42f/
failed exit 101 on existing Rust process test
private_config_failure_hides_both_output_streams (expected exit 7). No Rust source
changed. Isolated `cargo test --locked private_config_failure_hides_both_output_streams`
passed exit 0; full gate rerun follows, keeping the failure chronology.

Ran `bash scripts/install-flatpak.sh --candidate-receipt
target/task-runs/flatpak-candidate-de8d779/artifacts.json --activate-panel-entry
--log-dir target/task-runs/flatpak-install-de8d779` (absolute paths supplied).
Exit 0, installation verified. Private summary.json/task.log/installation.json
retained in that log directory. Installed ref
app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master,
commit cfa15ec80e68374319cda2749291939880badb88bcdc86b64190bc4f37a28e57
matches candidate; previous commit
d0f101c60a1e4fb78700184e3dac11c2544904989c3e171ec12d73e07d7138d7
retained as rollback identity. No native package removal or dependency updates.

Previous user launcher preserved at
target/task-runs/flatpak-install-de8d779/previous-panel.desktop; receipt records
destination and matching installed-export SHA256. Restore that saved copy to the
recorded destination to reverse launcher selection. Prior Flatpak deployment
rollback uses the recorded previous commit, not a rebuild. No rollback performed.
User global overrides only add read-only GTK/KDE theme/config paths; application
override empty. Actual mirror/recovery access is still unverified.

Read-only candidate payload inspection confirmed applet binary (33,231,048 bytes),
retained auth helper (6,598 bytes), English Fluent catalog and desktop entry.
Worker/guard are integrated binary entry points, not separate shell payloads.
An initial source search included two nonexistent historical filenames (exit 2);
corrected search found worker/guard definitions in src/bisync_worker.rs and
src/onedrive.rs. No product source changed or candidate rebuild needed.

Installation acceptance boundary: identity/metadata and launcher replacement
verified; panel rendering, both OAuth account flows, selected-folder access and
disposable integrated mirror/service lifecycle remain outstanding. Operator
asked to re-add Cloud Mounter and open the popup; no cloud or service mutation
performed by the installer. New installer sources/records remain uncommitted.

Subsequent execution, preserving failure/fix order:

- Full gate rerun target/task-runs/dev-gate-z8u6es5g/: exit 0, all checks passed.
  Earlier Rust process-error failure did not reproduce; no Rust edits made.
- Operator reported floating desktop applets instead of panel embedding, with
  another instance when changing panel position. This is a failed rendered-panel
  acceptance check, despite successful package installation. Read-only observation
  then found no remaining Cloud Mounter process; no unrelated app was stopped.
- Diagnosis: the installer copied the Flatpak desktop export into ordinary user
  applications. Locally available COSMIC panel source classifies Flatpak by
  desktop-file PathSource, not X-Flatpak alone (panel_space.rs is_flatpak).
  wrapper_space.rs adds inherited Wayland socket and env forwarding only for that
  class. The copy hid the Flatpak path identity and therefore panel connection.
  This was an installer defect, not accepted OneDrive behavior.
- Corrected --activate-panel-entry to back up and park the overriding user
  launcher, leaving the actual Flatpak export discoverable. Added
  --panel-entry-only to require the exact candidate already installed, verify it
  and fix launcher discovery without reinstalling. Focused installer suite now
  six tests, exit 0, including no-copy/no-reinstall boundaries.
- Ran maintained installer with the same candidate receipt,
  --activate-panel-entry --panel-entry-only and new log directory
  target/task-runs/flatpak-panel-entry-de8d779/: exit 0. Retained previous copied
  launcher and actual export path/hash in installation.json. The original native
  launcher backup in flatpak-install-de8d779 remains unchanged. App package,
  native package, panel configuration and cloud data unchanged by this correction.
- Operator asked to remove/re-add the panel entry and retest embedding. Final gate
  after this installer correction is recorded next. OAuth and mirror/service
  acceptance still have not run.

- Final corrected-installer `bash scripts/dev.sh gate`: exit 0, receipt/log
  target/task-runs/dev-gate-oj9_r083/ (source de8d779, dirty=true). All formatting,
  all-feature lint, all-target Rust tests and maintained Python tests passed.
  `git diff --check`: exit 0. No installed app inputs changed; no rebuild needed.
- Operator confirmed re-added Cloud Mounter embeds in the panel and opens there.
  Read-only `flatpak ps --columns=instance,application,pid` confirms application
  io.github.uutzinger.cosmic-ext-applet-mounter, instance 3932016410, host PID
  115373. Panel rendering/installed sandbox launch now verified after the fix;
  no earlier floating-window result counted as acceptance.
- Next operator step: create an empty disposable personal cloud folder, configure
  a fresh personal remote in installed Add Connection with that exact subtree and
  a separate local directory selected through the portal, then browser sign-in.
  No existing real mirror tree or whole-drive root is part of acceptance.

### October 16, 2026 — Personal Flatpak consent and configuration inventory defect

Operator completed browser sign-in/consent and observed the success page. Editor
showed verified OneDrive setup together with an invalid-data notice for rclone
config dump. Source inspection shows these are distinct operations: successful
OneDriveRcloneCompleted stores verified identity and starts detect_rclone_remotes;
that later inventory failure overwrites the global notice. Authentication itself
was not reported as failed. Installed post-setup inventory acceptance failed.

Screenshot also showed an empty remote-subtree field (whole-drive selection).
Operator instructed not to Save/Sync and to fill the exact disposable personal
folder. Operator subsequently confirmed that folder exists remotely. No sync,
Save, service setup or cloud deletion was executed by the agent.

Diagnosis/reproduction: process::redact_text previously applied whitespace-token
redaction to JSON before HostInspection parsed it. An escaped encoded OAuth token
with padding or a spaced client secret could lose JSON string boundaries. Added
synthetic structured_config_redaction_preserves_escaped_tokens_and_metadata:
failed before fix, exit 101, reproducing invalid JSON without reading real config.
Changed src/process.rs to parse complete JSON first, recursively replace secret
fields, sanitize remaining text values, and serialize structurally valid JSON;
non-JSON diagnostics retain existing text redaction. Private OAuth capture is
unchanged. Metadata needed by remote detection remains available; no raw
credentials are logged or used in tests. Existing whitespace-sensitive assertion
now checks JSON values rather than serialized spacing.

Checks at source de8d779, dirty=true:

- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0; target/task-runs/dev-check-ci6gzl88/.
- `cargo test --locked --lib process::`: exit 0, 20 passed, one explicit live test
  ignored (not counted as live acceptance).
- `cargo test --locked --lib host_inspection::`: exit 0, four passed; bounded
  requests, malformed/truncated data rejection, cancellation and safe errors.
- Final gate receipt follows. This source fix is not yet in the installed
  cf5da32-based candidate; updated source/package and installed retest are needed.

- Final `bash scripts/dev.sh gate`: exit 0, receipt/log
  target/task-runs/dev-gate-tzawhxc8/ (source de8d779, dirty=true). Formatting,
  all-feature lint, all-target Rust tests and maintained Python tests passed.
  `git diff --check`: exit 0. No Fluent or desktop metadata source changed in
  this redaction fix. Source changes remain uncommitted; no rebuilt/installed
  fix is claimed. Next: approved source commit/pin and replacement candidate,
  then Verify selected remote with the explicit disposable subtree. Preserve the
  already-authenticated remote rather than repeating OAuth or deleting it.

### October 16, 2026 — Approved replacement Flatpak commit and build

Operator approved committing the corrections and building the replacement.
Advised closing Add Connection without saving; existing authenticated remote
remains in rclone. Installed applet may run during the build; replacement
installation still requires closing/removing the running applet beforehand.

Committed JSON-safe redaction, corrected receipt-verified installer and the
preceding acceptance records as f5c384126460fc4c912e62be0c4ed75d8068dfac
(`Fix JSON credential redaction and Flatpak applet discovery`). Reused passing
check/focused/full-gate receipts from the preceding entry; no application code
changed after those checks. Updated source manifest/documentation to pin that
full app commit for the replacement. No dependency/runtime/permission change or
vendor regeneration. Pin-focused packaging check and build receipt follow.

- Replacement-pin `cargo test --locked --test flatpak_packaging_contract`: exit 0,
  four passed. Source f5c3841, packaging docs/manifest dirty. Unchanged app source
  and Cargo inputs retain full gate target/task-runs/dev-gate-tzawhxc8/; only
  affected source-pin packaging contracts rerun. `git diff --check`: exit 0.

- Packaging pin/records committed as cf00f2a76a26adfd96636dc064043d14021bbaa2
  (`Pin corrected Flatpak acceptance candidate source`). Clean detached worktree
  /tmp/cloud-mounter-flatpak-packaging-cf00f2a created at that exact commit.
- Ran maintained `bash scripts/build-flatpak.sh` from the root tooling checkout:
  --manifest /tmp/cloud-mounter-flatpak-packaging-cf00f2a/packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json,
  --source-sha cf00f2a76a26adfd96636dc064043d14021bbaa2,
  --arch x86_64 --branch master, root repository as --local-source-repo,
  --output-dir target/artifacts/candidates/flatpak-cf00f2a-x86_64-master,
  --log-dir target/task-runs/flatpak-candidate-cf00f2a (absolute output/log paths
  supplied). Exit 0, status succeeded. summary.json, task.log and artifacts.json
  retained in the log directory; no build errors or retries.
- App source fetched through exact-pin local Git transport:
  f5c384126460fc4c912e62be0c4ed75d8068dfac. Shipping release feature set,
  locked/offline Cargo build and all four runtime/SDK/BaseApp identities recorded.
- Repository and private bundle-import verification agree on ref
  app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master and commit
  48e7cabc6280678b999521499f475efd82395d2487da114ce60d4a887c97f8fd.
  Bundle 13,396,600 bytes; SHA256
  98d7aae7c7c11d96ab9686aa922953adf9313745e034975de1953490b1ddc521.
  Complete metadata and four runtime-provenance entries exactly match the prior
  de8d779 candidate. Built metadata SHA256 remains
  f791b92fc3866e73e401ea29f45367f7f7834fa7c6458171d71d1dfa3ae82cc3.

Verification boundary: replacement built/exported and inspected, not installed
or exercised. Current installed applet, authenticated remote, services and cloud
data unchanged during this build; no push or publication. Next: close/remove the
running applet for replacement installation, then select the existing remote,
fill the disposable subtree and Verify selected remote before saving/syncing.
These post-build task/evidence updates are a separate records-only commit and
do not alter candidate source/packaging identity or require another build.
