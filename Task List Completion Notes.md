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

### October 16, 2026 — Replacement installed for guided inventory retest

Operator requested guided acceptance and confirmed applet removed. Ran maintained
`bash scripts/install-flatpak.sh --candidate-receipt
target/task-runs/flatpak-candidate-cf00f2a/artifacts.json --log-dir
target/task-runs/flatpak-install-cf00f2a` (absolute paths supplied). Exit 0,
installation verified after full candidate-reuse checks. Prior installed commit
cfa15ec80e68374319cda2749291939880badb88bcdc86b64190bc4f37a28e57 recorded;
replacement 48e7cabc6280678b999521499f475efd82395d2487da114ce60d4a887c97f8fd
and complete metadata match the inspected candidate on the same master/x86_64
user ref. Retained summary.json/task.log/installation.json in that log directory.
No launcher change needed: original shadowing entry already parked. No build,
dependency update, remote creation/deletion, user-service change or app launch
performed by the installer. Existing authenticated remote retained.

Next operator check: re-add applet, open Add Connection, detect/select the existing
personal remote, enter expected account and explicit disposable remote subtree,
select the empty local acceptance directory through Browse and Verify selected
remote. Do not Save or Sync until verification/inventory result is reviewed.
Installed identity is verified; corrected inventory, folder access and integrated
mirror/service acceptance remain pending. Records-only edits do not require gate
or rebuild; prior code verification receipts remain applicable.

Guided personal verification retest: operator screenshot now shows
"OneDrive drive and selected folder verified. Save keeps background
synchronization paused." No preceding invalid-configuration notice is visible.
This verifies the installed editor's successful selected-remote/folder setup
result after the replacement; no new consent or remote creation was requested.
Next is Test Connection (plan/dependency/readiness/guard/access validation),
followed by reviewed Save in Paused state. No synchronization, generated-unit
state or local mirror-write acceptance is inferred from this screenshot.

### October 16, 2026 — Draft test passed; Save exposed legacy recovery compatibility

Operator screenshot: Test Connection passed, managed bisync service/timer
definitions validated structurally, and primary drive/selected-folder access
verified with the saved identity. This does not prove installed units or sync.
Subsequent Save failed with InvalidWorkDirectory referring to a different,
already-saved connection, not the new tested draft.

Read-only configuration inspection printed only engine/provider and overlap
classification. The rejected existing connection is legacy OneDrive (engine
absent), with its historical app-owned recovery directory at
<work-root>/onedrive-recovery/<its-own-connection-id>. Local mirror is outside
the work root. The tested draft ID is absent from the saved document: Save did
not persist it. No user configuration, path, legacy state, unit or cloud data
was edited to bypass validation.

Diagnosis: app Save builds a candidate document containing all saved profiles,
then Config::validate_with_work_root applies the new blanket work-root overlap
guard to all recovery paths. This rejects the historical legacy default even
when adding an unrelated rclone profile. Test Connection validates only its own
draft, so it passed before that document-wide check.

Changed src/config.rs: exempt only legacy OneDrive's exact, normalized per-ID
historical recovery directory. Mirror/cache paths remain protected; new rclone
profiles, other engines/providers, arbitrary state directories, the shared state
root, rclone-bisync and another owner's recovery directory receive no exemption.
No migration, engine conversion, state deletion or public UI string change.
tests/onedrive_configuration.rs adds a synthetic legacy-plus-new-profile Save
validation regression with positive compatibility and negative overlap cases.

Validation (source 4fdd537, dirty=true):

- New regression failed before fix, exit 101, reproducing Save's configuration
  rejection with synthetic paths only.
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0; target/task-runs/dev-check-qzyv08qs/.
- `cargo test --locked --test onedrive_configuration`: exit 0, eight passed,
  including unchanged legacy/engine routing, malformed identity and overlap tests.
- Final gate follows. Installed package is unchanged; Save retest and subsequent
  paused units/Preview/initial/scheduled/Pause/cleanup acceptance remain pending.

- First final gate target/task-runs/dev-gate-ghv7ttex/: failed exit 1 in
  repo_hygiene_test.py because the newly committed scripts/install-flatpak.sh
  had Git mode 100644. Earlier pre-commit gates did not enforce its mode while
  untracked; invocation through bash worked, but repository entry-point policy
  requires 100755. Corrected/staged only its executable bit (no content change).
  `python3 tests/repo_hygiene_test.py`: exit 0, four passed. Final gate rerun
  follows; original failure is retained, not counted as a pass.

- Additional read-only path classification examined 11 saved connection blocks:
  one historical per-ID legacy recovery under the state root, no other overlap
  found by that inspection. This is diagnostic classification, not a substitute
  for the app's full document validation. No saved configuration was modified.
- Final rerun `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-xqjx8r62/ (source 4fdd5379f7549a8214e8d608fe3cd3e4310673da,
  dirty=true). Formatting, all-feature lint, all-target Rust and maintained Python
  tests passed. `git diff --check` and `git diff --cached --check`: exit 0.
  Changed files: src/config.rs, tests/onedrive_configuration.rs, installer Git
  executable mode and task/evidence records. Installer mode is staged; source
  fix/records remain uncommitted. No installed Save success is claimed. Next:
  approved commit/source pin/replacement build, then installed Save/Paused retest.

### October 16, 2026 — Approved legacy-compatible replacement candidate

Operator approved committing and replacing the candidate. Committed the source
fix, synthetic regression, installer executable mode and chronological acceptance
records as 5e9f5b8c8bd3ca9bb426dec584b7cedd1954f37d
(`Preserve legacy OneDrive recovery paths during profile saves`). Reused final
gate target/task-runs/dev-gate-xqjx8r62/ and focused evidence above: application
code unchanged since validation. Updated manifest/current packaging docs to that
full app-source pin; runtime, Cargo inputs, features and permissions unchanged.
Pinned packaging contracts/build evidence follow. Installed applet and existing
profile data remain unchanged while the replacement candidate is prepared.

- `cargo test --locked --test flatpak_packaging_contract`: exit 0, four passed
  for the updated pin. Source 5e9f5b8, packaging edits dirty; no dependency/runtime
  changes. Unchanged application checks reuse dev-gate-xqjx8r62 as above.
  `git diff --check`: exit 0.

- Packaging source pin/records committed as db66196560bb42e461425fa7f8a9f4a3c1f78d01
  (`Pin legacy-compatible Flatpak acceptance candidate`). Clean detached checkout
  /tmp/cloud-mounter-flatpak-packaging-db66196 created at that revision.
- Ran maintained `bash scripts/build-flatpak.sh` from the root repository with
  --manifest /tmp/cloud-mounter-flatpak-packaging-db66196/packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json,
  --source-sha db66196560bb42e461425fa7f8a9f4a3c1f78d01,
  --arch x86_64 --branch master, root Git repository as --local-source-repo,
  --output-dir target/artifacts/candidates/flatpak-db66196-x86_64-master,
  --log-dir target/task-runs/flatpak-candidate-db66196 (absolute output/log paths
  supplied). Exit 0, status succeeded; summary.json/task.log/artifacts.json in
  that log directory. No build failure or retry. Exact app source
  5e9f5b8c8bd3ca9bb426dec584b7cedd1954f37d fetched through recorded local Git
  transport; shipping locked/offline release Cargo build retained.
- Repository and private bundle-import inspection agree on ref
  app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master, commit
  968e0ac9c1acfa722db89e43d6452d31dd73c9e6dacb26b08043fadf2308252c.
  Bundle 13,509,600 bytes, SHA256
  ee7b385dcd4c96d40fbaf85a2bcadc396a6167763f666143aa8b592be8a8125c.
  Complete metadata and all four runtime-provenance entries exactly match the
  prior cf00f2a candidate. Metadata SHA256 unchanged:
  f791b92fc3866e73e401ea29f45367f7f7834fa7c6458171d71d1dfa3ae82cc3.

Boundary: source/fake tests and replacement build/export/metadata verified; new
package not installed or exercised. Existing installed applet, saved profiles,
legacy recovery state, remote and services unchanged during build. No push or
publication. Next operator step: close unsaved editor and remove panel applet for
replacement installation, then existing-remote/subtree verification and Save
retest. Post-build evidence is a separate records-only commit; no rebuild needed.

### October 16, 2026 — Legacy-compatible replacement installed for Save retest

Operator confirmed panel applet removed for replacement. Ran maintained
`bash scripts/install-flatpak.sh --candidate-receipt
target/task-runs/flatpak-candidate-db66196/artifacts.json --log-dir
target/task-runs/flatpak-install-db66196` (absolute paths supplied). Exit 0;
candidate reuse/integrity checks passed and installed identity/complete metadata
match the candidate. Same user master/x86_64 ref; new commit
968e0ac9c1acfa722db89e43d6452d31dd73c9e6dacb26b08043fadf2308252c.
Previous installed commit
48e7cabc6280678b999521499f475efd82395d2487da114ce60d4a887c97f8fd
retained in installation.json for rollback identity. summary.json/task.log and
installation.json retained under target/task-runs/flatpak-install-db66196/.

No launcher modification needed, since native override remains parked. No
dependency updates, profile edits, cloud changes, service changes or app launch
performed by the installer. Next operator sequence: re-add panel applet, recreate
the unsaved personal draft using its existing remote and explicit disposable
subtree, Browse to empty local acceptance directory, Verify selected remote,
Test Connection, then Save only after preceding checks pass. Review resulting
notice/Paused state before any sync. Installed Save/Paused and subsequent
mirror/service lifecycle acceptance remain open. Documentation-only updates;
unchanged code checks reuse the preceding receipts, without another gate/build.

Legacy-compatible installed Save retest: operator screenshot reports the test
connection saved with managed mirror units and schedule disabled, requiring
Preview and confirmed initial Sync Now before Start. No legacy recovery-path
error. Read-only saved-document inspection confirms the disposable test profile
exists with rclone engine, OfflineMirror mode and the exact approved personal
test subtree. No raw configuration or credentials printed. This proves installed
Save/UI result and persistence; timer scheduling, actual transfers and service
lifecycle remain unverified. Next: disposable distinct local/cloud sample files,
then Preview before confirmed initial sync. No Start/Sync issued by the agent.

Installed personal Preview/initial acceptance, same db66196 candidate:

- Operator screenshot: Preview completed, estimated differing paths 2, initial
  sync not yet run; UI requested explicit Sync Now confirmation. This is an
  estimate, not a checksum or transfer result.
- Operator then confirmed initial Sync Now. Completion screenshot reports
  OneDrive synchronization and checksum verification completed successfully.
  Operator noted it took some time; no duration measurement supplied, so no
  performance claim or timing regression inferred.
- Initial sync/checksum success is recorded as observed installed UI/backend
  result. Explicit operator confirmation of both local/cloud test-file arrivals
  is still pending; no independent raw engine receipt inspection claimed here.
  Next: verify arrivals, then one unchanged manual Sync Now to exercise ordinary
  synchronization/checksum. Keep scheduling paused until that result is reviewed.
  Scheduled Start/Pause, business account flow and cleanup remain outstanding.
  No source edits/check repetition, service changes or cloud operations executed
  by the agent in this record-only update.

### October 17, 2026 — Ordinary sync blocked: sandbox/host mirror access

Operator confirmed both test files arrived after initial sync, then reported the
unchanged manual run blocked on missing/changed ownership marker. Read-only
inspection: real host mirror anchor absent, one initialized target, four retained
bisync listing files and none containing the access anchor. Current applet's
namespace does not expose the saved host mirror path. No marker, listing, lock,
initialization state, test file or user configuration changed during diagnosis.

Source establishes a namespace mismatch: folder selection resolves portal URI
to a host path and discards the portal access path; coordinator direct local I/O
uses that host path inside the sandbox, while rclone uses the actual host path.
Sandbox home paths can be temporary shadow directories. The initial engine
checksum checks the host tree, so it did not prove that coordinator anchor writes
reached that tree. This explains the false initialization boundary; current
missing-anchor guards correctly block further work and must not be bypassed.

Selected correction: retain per-directory existing portal grants in private
app-owned config, revalidate origins with typed Documents D-Bus (v5 GetHostPaths,
existing capability-gated portal-v4 document-info exception), and use the granted
portal paths only for sandbox mirror/recovery I/O. Engine arguments, saved
identities and target hashes keep host paths. Missing/revoked/malformed grants
fail closed, never fall back to writable sandbox-home shadows. Add recovery
Browse so its authorization is explicit; do not broaden filesystem grants.
Require local and remote anchor postconditions after initial/ordinary engine
success before checksum and initial completion publication. Also align scheduled
worker's Flatpak state root with the applet's host-shared root, retaining native
XDG_STATE_HOME policy. Source/fake tests and installed acceptance are separate.
The already-initialized test profile's missing anchor is not silently recreated.

Implemented files: new src/mirror_access.rs with DirectoryOrigin trait and fake,
private per-directory grant records, strict portal-path/origin matching, bounded
16 KiB reads, writable-directory checks and cancellation/8-second origin bound;
src/lib.rs export; src/app.rs remembers folder selections, adds recovery Browse
and checks access during validation/runtime preparation; shared coordinator uses
mapped local marker/recovery/retention paths while engine arguments stay host
paths, checks OneDrive anchors after engine success and shares host state with
scheduled workers. New app-owned messages/help use Fluent. Packaging manifest
permissions and rclone flags unchanged. Contracts/inventory/guide updated.

Validation at source 1bf37a053a8155f715ddf97392fb56717d982cd3, dirty=true:

- `just fmt`: exit 0.
- Iterative `bash scripts/dev.sh check`: exit 0, logs
  target/task-runs/dev-check-n29b9g7z/, dev-check-1ewz111q/,
  dev-check-r_p05wcy/ and final dev-check-gztawdhh/; bounded reader/permission
  refinements were included in the final check. Early checks are not substituted
  for final validation of later edits.
- `cargo test --locked --lib bisync_coordinator::`: exit 0, 29 passed, including
  host-vs-portal argument separation, missing post-engine local/remote anchors
  cannot initialize, worker state-root equivalence and existing lock/cancellation/
  partial-failure/retention protections.
- `cargo test --locked --lib mirror_access::`: initial two tests passed; final
  four passed with malformed/oversized/linked record and read-only/absent/linked
  directory cases. Revoked/changed origin and cancellation never select host
  shadow paths. Tests use synthetic paths, temp files and fake origin adapters.
- `bash scripts/dev.sh localization-check`: exit 0, receipt/log
  target/task-runs/dev-localization-check-3w68t2h_/.
- Final gate follows. No live sync, marker recreation, permission change, profile
  conversion, deletion, commit or candidate build performed in this correction.

- First gate target/task-runs/dev-gate-sfapxthr/: exit 101 on Clippy's
  unnecessary_literal_unwrap in the new state-root test fixture. Replaced the
  known Some/unwrap expected value with its literal PathBuf; production behavior
  unchanged. Focused state-root test and final gate rerun follow. No failed gate
  is counted as acceptance.

- After lint-only test correction, `bash scripts/dev.sh check`: exit 0,
  target/task-runs/dev-check-msnv2u9t/; `just fmt`: exit 0;
  `cargo test --locked --lib flatpak_worker_uses_the_same_host_shared_state_root_as_the_applet`:
  exit 0, one passed. Production/Fluent inputs unchanged since focused/localization
  checks; those passing receipts are reused.
- Bounded read-only host portal-directory metadata inspection examined ten
  document entries and found one alias for the selected test directory in
  doc-ID/basename form, matching the implementation's path grammar. No directory
  contents, permissions, grant records or test files modified; this is format
  evidence, not sandbox write/worker acceptance. Final gate rerun follows.

- Final `bash scripts/dev.sh gate`: exit 0; receipt/log
  target/task-runs/dev-gate-j7fe93iz/ (source 1bf37a053a8155f715ddf97392fb56717d982cd3,
  dirty=true). Formatting, all-feature lint, all-target Rust and maintained Python
  tests passed. The state-root fixture lint issue did not recur.
- `git diff --check`: exit 0. No desktop/AppStream source change, so no new
  metadata validation needed. Fluent validation receipt above remains current.
  All runtime/source edits and new mirror_access module are uncommitted. No
  rebuild/installation or renewed live sync performed. Original failed profile,
  test files, marker absence and initialization state retained unchanged.

Next: approve source commit/pinned replacement build, then installed portal Browse
for both mirror and recovery directories in a fresh disposable profile. Retest
initial anchor publication, ordinary sync, cross-launch access and scheduled/Pause
before claiming integrated acceptance. Existing initialized missing-anchor state
is deliberately not repaired by creating markers or deleting bisync state.


### October 17, 2026 — Authorized replacement candidate for portal mirror access

User authorized commit, build and retest. Source correction committed as
91dc5cef71531a64dca17a94fba7b864a4526420. Manifest and current packaging guide now
pin that immutable source; previous candidate remains rollback evidence.
Packaging checkout will be committed separately. Cargo sources, shipping
features, runtimes and permissions are unchanged. Prior passing development gate
and localization receipts above remain applicable to the same runtime inputs;
only affected packaging-contract tests are repeated for this pin.

Replacement x86_64/master build is necessary because installed db66196 predates
the portal-backed local I/O and anchor postconditions. Preserve failed profile
and data; fresh disposable mirror and recovery selections are required for retest.
Build/install success will be recorded separately from operator UI acceptance.

`cargo test --locked --test flatpak_packaging_contract`: exit 0, four passed
for the replacement source pin. `git diff --check`: exit 0.


### October 17, 2026 — Portal-access replacement built and installed

- Source 91dc5cef71531a64dca17a94fba7b864a4526420; packaging
  f685237adb9565da8ac926952b70c8cbc30d9e99, clean detached worktree
  /tmp/cloud-mounter-flatpak-packaging-f685237. No push/tag/publication.
- `bash scripts/build-flatpak.sh --manifest
  /tmp/cloud-mounter-flatpak-packaging-f685237/packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json
  --source-sha f685237adb9565da8ac926952b70c8cbc30d9e99 --arch x86_64
  --branch master --local-source-repo "$PWD" --output-dir
  "$PWD/target/artifacts/candidates/flatpak-f685237-x86_64-master"
  --log-dir "$PWD/target/task-runs/flatpak-candidate-f685237"`: exit 0.
  Receipt target/task-runs/flatpak-candidate-f685237/artifacts.json;
  task.log/summary.json alongside. Bundle 13,512,216 bytes, SHA256
  87f1612006bd0c351f2046194cdf740a9a4fdde49f8cf2a2911162de26d5e380.
- Export and imported bundle agree on ref
  app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master and OSTree
  7b5189aa6322a6ac6ef6ded343173dc70236be959478ca666e0b90eaf3c52f80.
  Complete metadata and runtime provenance equal the prior db66196 receipt;
  metadata file SHA256 f791b92fc3866e73e401ea29f45367f7f7834fa7c6458171d71d1dfa3ae82cc3.
- Operator confirmed windows closed, syncs paused and panel entry removed;
  failed profile/data retained. `bash scripts/install-flatpak.sh
  --candidate-receipt "$PWD/target/task-runs/flatpak-candidate-f685237/artifacts.json"
  --log-dir "$PWD/target/task-runs/flatpak-install-f685237"`: exit 0.
  Installer repeated complete candidate reuse verification, replaced user app,
  and verified installed ref/commit/complete metadata. Receipt
  target/task-runs/flatpak-install-f685237/installation.json, logs alongside.
  Previous installed rollback commit
  968e0ac9c1acfa722db89e43d6452d31dd73c9e6dacb26b08043fadf2308252c.
  No application override; existing global read-only theme/config grants unchanged.
- No build/install failure. App source/Fluent unchanged since the passing final
  gate/localization checks; reused those receipts. No extra rebuild for records.

Installed package identity is verified; rendered panel/editor and fresh disposable
portal authorization, initial/ordinary sync, cross-launch and scheduled/Pause
acceptance remain pending operator observations. Requested panel re-add first.
Old missing-anchor state is preserved and deliberately blocked; no repair/reset,
manual marker recreation, cloud deletion or broader filesystem permission.

October 17 operator observation, installed f685237 candidate: re-added applet
embeds normally in the COSMIC panel and popup opens. Rendered panel launch passed;
fresh disposable directory authorization and mirror synchronization remain pending.
No runtime changes or repeated build/check required for this observation.


October 17 fresh draft authorization failure: operator screenshot shows personal
remote/subtree verified, but Test Connection requests both local Browse grants.
Read-only inspection of private grant metadata found one record, for disposable
Acceptance-Personal-2, and none for Recovery-Personal-2. No credential/config dump
or grants modified. This is missing-record evidence, not proof of a portal
implementation failure after both Browse operations. Asked operator which
selectors were used; recovery authorization retest pending.

Scope review: verify_rclone_access_with_verified, prepare_offline_mirror_runtime
(non-legacy), and the shared bisync coordinator use LocalMirrorPaths; therefore
SharePoint and other rclone mirrors share the Flatpak access requirement. Native
resolution retains original paths. LegacyOnedrive runtime preparation retains
its separate path. No recorded installed Flatpak SharePoint mirror portal test
was found in current active task/evidence records; no such pass claimed. Added
one disposable shared-access smoke to integrated acceptance without repeating
unchanged native/provider fault suites. Source/package inputs unchanged.


### October 17, 2026 — Recovery authorization confirmed; editor wording correction

Operator confirmed only mirror Browse had been used. After recovery Browse,
installed draft Test Connection passed: managed service/timer structural checks
and OneDrive identity/subtree access verified. This resolves the missing-grant
observation; initial/ordinary sync and worker persistence are still unverified.

Operator found misleading automatic recovery text. src/app.rs now chooses
Fluent Browse placeholder/help for Flatpak non-legacy rclone mirrors and omits
the automatic-path notice. Native/legacy automatic recovery remains available;
explicit recovery selections no longer show an automatic-path notice.
i18n/en/cosmic_ext_applet_mounter.ftl adds the two applicable messages. No
runtime, path-resolution, service, credential or permission behavior changed.
Task List.md records source correction and installed acceptance boundaries.

Validation, base 605d865de73e53ee3b7a5c2becb747b27d183d06, dirty=true:
- just fmt: exit 0.
- bash scripts/dev.sh check: exit 0, target/task-runs/dev-check-383zm643/.
- cargo test --locked offline_mirror_blank_recovery_defaults_next_to_mirror_directory:
  exit 0, one focused test passed; other filtered targets are not live acceptance.
- bash scripts/dev.sh localization-check: exit 0,
  target/task-runs/dev-localization-check-o5oj20bq/.
- Initial gate target/task-runs/dev-gate-4sedy4f6/: exit 1 after formatting/lint
  passed; sandbox denied just temporary-directory creation under /run/user.
  Reran with required access, no source correction for this environment failure.
- Final bash scripts/dev.sh gate: exit 0, target/task-runs/dev-gate-4ti9gk3s/.
- git diff --check: exit 0.

Source wording correction is not yet committed/built/installed. Continue current
installed candidate's Save/paused and sync acceptance without rebuilding for
text alone; old failed profile remains unchanged. Rendered corrected wording
requires a later replacement candidate. No new engine behavior is claimed.


October 17 installed f685237 fresh personal profile progression: operator saved
with acknowledgment of existing remote authentication reuse; Preview completed
with two estimated differing file paths and explicit uninitialized status.
Confirmed initial Sync Now then reported synchronization and checksum success.
Read-only host filesystem inspection: Acceptance-Personal-2 exists; one regular,
non-symlink ownership anchor (37 bytes) for new connection
8e646732-5734-44c8-8033-d8d9d06b85ae, four retained connection bisync listings,
two containing the anchor name, and one initial-sync-complete marker. Both
operator-named test files test.docx and test.txt are present in the host mirror.
No contents printed or files modified. Unlike the prior failed profile, the
anchor now exists in the host engine tree. This establishes initial host-path
and retained-listing evidence only; ordinary no-edit sync, portal cross-launch,
real recovery writes, schedule/Pause and SharePoint sandbox smoke remain open.
No source/runtime inputs changed; prior verification receipts reused.


October 17 installed f685237 ordinary no-edit sync retest: after successful
initial sync and read-only host-anchor inspection, operator ran Sync Now again
without editing files. Screenshot reports OneDrive synchronization and checksum
verification completed successfully. Previous initialized missing-anchor failure
did not recur in this fresh disposable profile. This is installed ordinary-sync
success, not measured zero-transfer evidence or background worker acceptance.
Next: close editor, remove/re-add panel applet, reopen saved connection and repeat
manual sync without Browse to verify retained portal authorization across fresh
launches. Schedule/Pause, recovery writes and SharePoint sandbox smoke remain
outstanding. No source/package inputs changed or checks repeated.


October 17 installed f685237 cross-launch manual sync: following instructions to
close the Modify window, remove/re-add the panel applet, reopen the saved profile
and run Sync Now without Browse, operator screenshot reports successful OneDrive
synchronization and checksum verification. Retained portal authorization across
applet relaunch passed at the operator-observation level; separate scheduled
worker access is not inferred. No files, grants or services changed by agent.
Next scheduled Start and Pause with one disposable local edit; wait for the
configured timer, do not substitute manual Sync Now. Source/check receipts
unchanged. Recovery write and SharePoint sandbox checks remain outstanding.


October 17 installed f685237 scheduled worker/Pause: after instructions to edit
local test.txt, Start the saved test connection and wait for the timer without
manual Sync Now, operator confirmed the edit arrived remotely. Popup screenshot
reports Pause completed and background sync timer stopped; manual Sync Now
remains available. This is operator-observed scheduled transfer and Pause success;
no independent systemd signal/property trace or post-Pause interval observation
is claimed. Agent did not modify services.

Read-only recovery inspection: suggested Recovery-Personal-2 directory is absent;
actual Browse grant points to Acceptance-Personal-2-Recovery. That selected host
directory exists and contains one dated .cosmic-mounter-owner file (37 bytes),
with no retained user-file backup yet. Directory date is recorded by runtime;
execution evidence is dated October 17 in operator timezone. A local-to-remote
edit does not establish local overwritten-file recovery. Next: edit only remote
test.txt, keep local original, run paused manual sync, inspect local recovery
copy. No source/check/package inputs changed. SharePoint sandbox smoke and other
integrated acceptance boundaries remain open.


October 17 installed f685237 remote-to-local/recovery smoke: operator confirmed
remote edit arrived locally after instructions to leave local unchanged and run
manual Sync Now while paused. Read-only selected host recovery inspection found
two files: dated owner marker and timestamp-suffixed test.txt backup (10 bytes).
Backup is a regular non-symlink file; byte-hash comparison confirms it differs
from current local test.txt. Contents were not printed. Backup bytes were not
captured before the overwrite, so exact previous-version equality is not claimed;
observed overwrite plus retained distinct timestamped copy establishes recovery
creation for this disposable transfer. Runtime directory timestamps use UTC date
October 18; operator evidence date remains October 17 America/Phoenix.

Personal installed smoke now covers draft Test/Save, Preview, initial sync and
checksum, host ownership anchor, ordinary sync, retained portal access after
applet relaunch, scheduled local-to-remote edit, Pause, remote-to-local edit and
local recovery creation. This does not close integrated acceptance overall:
business account flow, disposable SharePoint shared-access smoke and corrected
recovery wording rendered in a later candidate remain outstanding. No source,
permission, service, credential, or file mutations by agent; prior source checks
remain applicable. Old failed profile/data preserved; current test stays paused.


October 17 installed f685237 corporate setup progression: after instructions to
Connect new remote using work/school account, operator screenshot shows generated
remote selected, Work or school OneDrive selected and drive/selected-folder
verified notice. Actual remote subtree is Acceptance-Business, differing from
suggested longer disposable name; subsequent checks must stay within this
verified test subtree. No account address, organization, tokens or drive IDs
copied into records. This is setup/verification observation, not Test Connection,
Save, managed-unit or transfer acceptance. Next Test Connection and Save paused,
then one minimal business transfer smoke; reuse passed shared scheduling/fault/
recovery tests rather than repeat personal suite. No agent runtime mutation.


October 17 installed f685237 corporate initial transfer smoke passed. Operator
screenshot confirms Test Connection passed (managed service/timer structural
validation and primary-drive/subtree identity/access); operator separately
confirmed Save. Preview screenshot estimated two differing file paths and
explicitly stated initial sync had not run. Subsequent confirmed initial Sync
Now screenshot reports synchronization/checksum verification completed
successfully; operator confirmed test files on remote.

Read-only host inspection via selected Acceptance-Business grant found both
regular test files test.docx/test.txt, one regular non-symlink ownership anchor,
four retained bisync listings (two with anchor) and one initial-sync-complete
marker. No content/config/credential dump or agent mutation. Corporate installed
account setup, portal-selected paths and initial transfer are now observed;
shared personal ordinary/relaunch/scheduled/Pause/recovery and earlier native
fault evidence are reused, not repeated. This does not imply corporate-specific
fault/scheduling testing. Current test stays paused. Shared SharePoint sandbox
smoke, installed corrected wording and final acceptance reconciliation remain
open. No source/check/package input change; no redundant gate/build.


October 17 installed f685237 SharePoint sandbox smoke started: operator created
Acceptance-SharePoint in the existing Documents library, reused existing
SharePoint authenticated remote/library without new authentication, and selected
Offline mirror. Draft Test Connection screenshot reports structural managed
SharePoint service validation, background timer disabled, backend match,
selected subtree accessible with zero visible items, and library/site/drive
matching saved identity. Optional signed-in account label unavailable from Graph;
required identity verification succeeds independently. Organization, library URL,
remote alias, drive ID and account details omitted from records. Draft Test is
not installed-unit execution or sync acceptance. Next Save, disposable bidirectional
initial transfer and retained portal authorization check. No agent config/grant/
service/cloud mutation and no repeated unchanged source checks.


October 17 installed f685237 SharePoint initial sync: saved profile Modify screen
reports Sync Now completed, transfer counts not reported by rclone, and initial
synchronization recorded complete; future runs use normal bisync. Operator
confirmed files on remote. Read-only host inspection of portal-selected
Acceptance-SharePoint found test.docx, test.txt and SharePoint ownership anchor;
connection df45c039-e4a3-4af9-9f25-e9f3da0e1c10 has one initial-sync-complete marker
and six retained listing files. No file contents/config/credentials printed or
mutated. Missing counts are not treated as zero transfers; UI completion and
host/remote observations support initial transfer success. No new standalone
checksum result inferred from SharePoint notice. Next close editor, remove/re-add
panel applet and run ordinary Sync Now without Browse to verify SharePoint
retained portal access. Reuse passed personal shared scheduling/recovery tests;
no duplicate suite. No source/package/check inputs changed.


October 17 installed f685237 SharePoint cross-launch ordinary sync passed:
following requested editor close/panel remove-readd/reopen sequence without
Browse, operator screenshot reports Sync Now completed. No initial-sync message
on this run. Rclone transfer counts unavailable; no measured zero-transfer claim.
Marked the specific installed SharePoint portal-access smoke complete in Task
List.md. Existing SharePoint connection/authentication was reused without engine
migration or modification of its original profile; test is separate and paused.

Installed affected-path coverage now includes personal full smoke (manual,
restart, scheduled transfer/Pause and recovery creation), business account setup
and initial transfer, and SharePoint shared folder-access/initial/ordinary sync
after restart. Unchanged shared scenarios are not repeated per provider. Overall
integrated acceptance remains open for final reconciliation and installed
recovery-placeholder/help correction; that source-only change passed check,
focused recovery test, localization and gate as previously recorded. Current
uncommitted scope: src/app.rs, English Fluent file, Task List.md and completion
notes. No package build/install/commit, data deletion or permission change this
observation; old failed profile and disposable resources preserved. git diff
--check: exit 0; no runtime inputs changed since earlier passing checks.


### October 18, 2026 — Finish installed recovery wording correction

User requested finishing wording fix. Source/UI and acceptance records committed
as cf2d13b53460079b2bb198a605605d3bd1911f3b; manifest and current packaging guide
now pin this immutable source. Replacement x86_64/master package is necessary:
installed f685237 source 91dc5ce predates recovery placeholder/help correction.
Only recovery presentation changes; accepted engine, scheduling, recovery and
account tests remain applicable. Prior check, focused recovery test, Fluent and
final gate receipts remain valid for unchanged source; rerun only packaging
contract for new manifest pin. No permission/runtime/feature/Cargo input change.
Untracked docs/engineering/Push Buttons.md is unrelated user work and preserved
outside staging. Build, receipt-verified installation and rendered blank recovery
placeholder/help/no-automatic-notice review remain next. Preserve all paused
profiles, credentials and failed-profile evidence; no cloud/file cleanup here.

Replacement pin validation: cargo test --locked --test flatpak_packaging_contract,
exit 0 (four passed); git diff --check, exit 0. Prior passing full gate and
localization receipts reused for unchanged application inputs.


October 18 operator replacement build succeeded: maintained build-flatpak.sh
command supplied in chat, packaging source 848d48bcdeefa7b03a914f0d14e5c7930229d8d3,
app pin cf2d13b53460079b2bb198a605605d3bd1911f3b, x86_64/master. Producer receipt
and summary inspected at target/task-runs/flatpak-candidate-848d48b/; succeeded,
exit 0. Export and bundle match ref
app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master and commit
757aae349f1017ea47eb93c88e6f1522b8ed9cef657a480e6f1b18826156e8a0.
Bundle 13,554,792 bytes; SHA256
3cce782f1ca22bb0c60826f3a6d3e991329ddd8d7b5ae4e16c42402f3ea93d39.
Complete metadata and runtime provenance equal previous f685237 candidate;
no runtime/grant change. Receipt inspection is not installed acceptance; maintained
installer will repeat complete reuse/hash checks before replacing app. Asked
operator to close windows/pause/remove panel entry before installation. Preserve
original failed and current paused profiles/data. Untracked Push Buttons.md
(currently at repository root) remains outside scope. No repeated sync suite or
build; installed wording check still pending.


October 18 operator confirmed windows closed, syncs paused and panel entry
removed. bash scripts/install-flatpak.sh --candidate-receipt
"$PWD/target/task-runs/flatpak-candidate-848d48b/artifacts.json" --log-dir
"$PWD/target/task-runs/flatpak-install-848d48b": exit 0. Installer repeated complete
candidate reuse validation, installed user app and verified installed identity/
complete metadata. Receipt target/task-runs/flatpak-install-848d48b/installation.json,
summary/log alongside. Installed commit
757aae349f1017ea47eb93c88e6f1522b8ed9cef657a480e6f1b18826156e8a0;
previous rollback commit
7b5189aa6322a6ac6ef6ded343173dc70236be959478ca666e0b90eaf3c52f80.
No failure or extra grant/runtime/service/data change. Next re-add panel applet,
open blank Add OneDrive Offline draft and inspect empty recovery field/help;
do not Save, authenticate, or repeat accepted sync tests for text-only change.
Rendered wording remains unverified until operator observation.


October 18 installed 848d48b rendered wording observation: operator reports
"Choose a recovery..." appeared after instructions to re-add panel/open blank
OneDrive Offline draft. Correct Browse placeholder confirmed. This response
does not explicitly confirm help text or absence of automatic-directory notice;
those two observations remain pending. No draft Save, runtime/source changes,
checks or rebuild needed for this observation. Untracked Push Buttons.md retained.


### October 18, 2026 — Installed connection removal acceptance

Operator removed paused corporate disposable profile and confirmed disappearance
from main window. Installed candidate 848d48b. Read-only saved document inspection
finds neither corporate test name nor UUID 93ce64e3-5855-4c1b-a486-51024802cfe2.
Managed service/timer files absent from user unit directory; zero remaining
unit/enablement links for that UUID. Host Acceptance-Business directory remains,
with test.docx/test.txt and anchor (three regular files); selected
Acceptance-Business-Recovery remains with owner marker (one regular file).
No file contents or credentials dumped; cloud folder not deleted or inspected.

Read-only typed systemd user D-Bus inspection using dbus-python Manager.GetUnit:
initial sandbox attempt exit 1, AccessDenied connecting session bus; repeated
with authorized bus access exit 0. Both named service and timer return
org.freedesktop.systemd1.NoSuchUnit. Confirms no loaded managed units, rather
than merely absence of unit files. No CLI substitution or service mutation.
Connection-removal preservation smoke marked complete. No new source/runtime
change or repeated gate needed. git diff --check: exit 0. Recovery wording help/
automatic-notice visual confirmation and final phase-4 reconciliation remain.


### October 18, 2026 — Phase 4 reconciled and closed

Operator requested reconciling records and closing phase 4. Closed implementation
and integrated installed smoke under the accepted scope, with no escalation to
full release/lifecycle acceptance. Source cf2d13b53460079b2bb198a605605d3bd1911f3b;
packaging 848d48bcdeefa7b03a914f0d14e5c7930229d8d3; installed OSTree
757aae349f1017ea47eb93c88e6f1522b8ed9cef657a480e6f1b18826156e8a0.
Prior producer/installer and source gate/localization receipts remain authoritative.

Accepted evidence: personal and corporate account/setup; personal initial/manual,
restart, scheduled edit/Pause and recovery creation; corporate initial transfer;
SharePoint initial/ordinary sync after restart; host ownership/state; candidate
identity/unchanged metadata/runtimes; rendered recovery Browse placeholder;
disposable corporate connection removal with absent saved entry/unit files/links
and D-Bus NoSuchUnit, local files preserved. Reused unchanged native/fake faults
and shared scheduling rather than duplicate provider suites. Missing transfer
counts never treated as zero transfers; exact recovery previous-byte equality
not inferred. No cloud deletion or comprehensive phase-7 lifecycle claim.

Two unobserved wording details (help and absence of automatic notice) explicitly
carried to phase-7 visual review, not checked off. Existing phase-7 implemented
negative guards, Native/Debian and broader Flatpak lifecycle/release acceptance
remain open. Phases 5/6 legacy deprecation/removal unchanged. Proposed Test/Verify
and Settings Refresh simplifications are separate UI work, not closing criteria.

Records changed: Task List.md condensed phase-4 execution detail into completed
smoke summary; engineering OneDrive contract status/index reconciled; these notes
preserve full removed checklist chronology below. User Push Buttons.md untouched.
Documentation-only reconciliation: git diff --check exit 0; local linked record
and phase status checks follow. No changed runtime inputs, repeated gate/build,
installation, service operation, commit, push or cleanup needed for this step.

#### Preserved phase-4 installed execution checklist at closure

The following is the pre-reconciliation checklist verbatim, retained as historical
progression; later entries above and the authoritative active task summary
supersede its intermediate pending statuses.

- [ ] Integrated Flatpak acceptance (started October 16; transferred from phase 3 October 15): reuse
  applet manifests/runtime/probes after integration. Check both account flows,
  OAuth/browser portal, selected paths, host/sandbox boundaries and integrated
  units/status; run one disposable initial-sync/verification/cleanup smoke.
  Reuse unchanged native scenario/fault evidence; no separate prototype sandbox
  or repeated business faults. Record results toward phase-7/consolidated Flatpak
  acceptance, not as another duplicate suite; probe-only evidence is insufficient.
  October 16: read-only installed/runtime preflight added and exercised; all four
  accepted runtime/SDK/BaseApp/extension refs present. Corrected BaseApp app-ref
  validation in build/reuse tooling; 25 focused script tests and final gate passed.
  Accepted implementation is now cf5da32 and its full manifest pin/Cargo inputs
  are checked. Packaging committed as de8d779; source-based candidate built and
  export/bundle identity verified. Approved installation and integrated acceptance
  remain next; see the October 16 completion evidence. Installation/acceptance
  authorized October 16 and started; maintained receipt-verified installer added.
  Candidate installed and ref/commit/metadata verified; native launcher backed up
  and initial launcher copy caused floating applets (failed panel acceptance).
  Corrected discovery by parking the user override so COSMIC recognizes the
  real Flatpak export; package unchanged. Panel retest passed (operator observed
  embedding/popup; running Flatpak identity confirmed), final installer gate
  passed. Disposable account/mirror acceptance remains outstanding.
  Personal browser consent/setup succeeded; automatic remote inventory then
  reported invalid data. JSON-safe credential redaction fixed in source;
  regression reproduced before fix, then check, focused tests and final gate
  passed. Updated candidate build/installed retest remains outstanding.
  Fixes/installer/acceptance records committed as f5c3841; replacement manifest
  pins that exact source. Replacement candidate built October 16 from packaging
  cf00f2a: export/bundle identity verified, metadata/runtime provenance unchanged.
  Replacement installed October 16; installed identity/complete metadata match
  candidate. Installed personal selected-folder verification retest passed;
  editor shows verified status without the previous parsing notice. Test
  Connection passed. Save rejected an existing legacy OneDrive recovery path
  under the historic app-owned state namespace; narrow compatibility fix passes
  check, eight focused configuration tests and final gate. Installer executable
  Git mode corrected after gate exposed it. Replacement build/Save retest pending.
  Compatibility fix committed as 5e9f5b8; candidate source pin updated to that
  exact revision. Replacement built from packaging db66196, with export/bundle
  identity verified and unchanged complete metadata/runtime provenance.
  Updated installation verified October 16 (candidate identity/metadata match);
  installed Save retest passed: managed mirror units saved, schedule reported
  disabled. Persisted rclone engine and explicit disposable subtree confirmed.
  Installed Preview passed (two estimated differing paths); confirmed initial
  Sync Now reports synchronization and checksum verification successful.
  Operator confirmed both test files arrived. Quiet manual run was blocked by
  missing ownership anchor; October 17 investigation found no anchor in host
  mirror or retained bisync listings despite initialized state. Portal-backed
  mirror/recovery I/O, scheduled shared state root and initial-sync anchor
  postconditions corrected in source; four access tests, 29 coordinator tests,
  localization check and final gate passed. Correction committed as 91dc5ce;
  replacement manifest pinned to that revision. Packaging f685237 built and
  installed October 17; export/bundle/installed identities match, complete metadata
  and runtime provenance unchanged. Fresh disposable portal/anchor retest is in
  progress; operator confirmed replacement embeds in panel and popup opens.
  No marker/state reset. Old failed profile/data retained. Quiet manual run,
  scheduled/Pause and remaining acceptance stay pending. Fresh draft Test
  Connection blocked on local authorization; read-only inspection found mirror
  Browse grant only, recovery grant absent. Operator confirmed recovery was not
  selected; Test Connection passed after recovery Browse. Correct misleading
  automatic-recovery placeholder/help for Flatpak rclone mirrors corrected in
  source; check, focused recovery test, localization and final gate passed.
  Installed wording retest awaits a later candidate; sync behavior unchanged.
  Fresh profile Save (shared-auth acknowledgment), Preview (two paths) and
  confirmed initial Sync Now/checksum passed. Host mirror now contains one
  ownership marker and both test files; two of four retained listings contain
  the marker, with one initial-complete marker. Operator's second, no-edit Sync
  Now passed synchronization/checksum verification; prior ownership-marker
  failure did not recur. Operator's sync after closing editor and removing/
  re-adding panel applet also passed without Browse: retained portal access
  across applet launches verified. Scheduled local edit arrived remotely;
  operator's Pause notice confirms background timer stopped. Selected recovery
  directory contains a dated ownership marker. Remote-only edit arrived locally;
  timestamped test.txt backup exists in selected host recovery and differs from
  current local file. Personal installed sync/recovery smoke passed; remaining
  account/provider checks below stay open. Corporate installed setup screenshot
  confirms new remote selected, work/school drive and disposable Acceptance-Business
  subtree verified. Test Connection passed; operator confirmed Save. Preview
  estimated two paths; initial Sync Now/checksum passed and operator confirmed
  remote files. Host mirror contains both test files, one regular anchor and
  initial-complete marker; two of four retained listings contain anchor.
  Corporate installed setup/initial transfer smoke passed; unchanged personal
  scheduling/recovery/fault coverage reused.
  - [x] Verify shared portal folder access with one disposable SharePoint rclone
    mirror in the installed Flatpak after the OneDrive access retest; existing
    native/prototype evidence does not establish sandbox access. Reuse unchanged
    provider scenarios; cover mirror/recovery Browse and cross-launch access.
    Installed draft Test Connection passed with existing authenticated remote:
    library identity matched and disposable Acceptance-SharePoint accessible
    (zero visible items). Optional Graph account label unavailable, nonblocking.
    Saved profile initial sync completed; operator confirmed remote files. Host
    mirror contains both test files and SharePoint anchor, with retained listings
    and initial-complete state. Ordinary sync after closing editor/removing and
    re-adding panel applet passed without Browse: retained portal access verified.
    Transfer counts unavailable, not claimed zero; shared personal worker/recovery
    evidence reused. Corrected recovery wording committed as cf2d13b; October 18
    replacement manifest pinned to that revision. Packaging 848d48b built by
    operator October 18; source/export/bundle identity verified, complete metadata
    and runtime provenance unchanged. Receipt-verified replacement installed
    October 18; operator confirmed rendered Browse recovery placeholder.
    Help text and absence of automatic-directory notice await confirmation.
  - [x] Installed disposable connection removal: operator removed corporate test
    profile; saved entry, service/timer files and enablement links absent. Typed
    systemd D-Bus returns NoSuchUnit for both units. Local mirror/test files and
    recovery directory preserved. No cloud deletion performed.


### October 18, 2026 — Separate 0.5.4 deprecation from later engine removal

Operator accepted two-release transition and requested moving phase 6 out of
0.5.4 tasks. Task List.md now states 0.5.4 scope explicitly: new OneDrive Offline
uses rclone; existing abraunegg mirrors remain supported for that deprecation
release. Phase 5 is labeled 0.5.4 deprecation; phase 7 is labeled 0.5.4 migration
acceptance, with release notes describing deprecation rather than completed
removal. Consolidated Flatpak/package release gates remain in 0.5.4.

Moved phase-6 removal checklist verbatim to dedicated future section
Post-0.5.4: legacy abraunegg engine removal, retaining number 6 for historical
references. Next release tentatively 0.5.5; no fixed publication commitment.
Added explicit preservation/recognizable-profile upgrade task: retiring support
must not delete local files/recovery/credentials, silently drop profiles or
reinterpret them as rclone. onedriver Online unchanged; no conversion tool.
Historical implementation/acceptance records and all existing checklist statuses
preserved. Untracked Push Buttons.md untouched. Documentation-only change;
no source, profile, engine, dependency, service or package mutations. Validate
moved checklist equality, release placement, local anchor and diff whitespace;
no repeat runtime gate/build required for unchanged inputs.

Reorganization verification: original phase-6 checklist equals moved text
verbatim; one phase-6 heading exists in future section, outside 0.5.4; release
labels and local link target checked; git diff --check exit 0.


### October 18, 2026 — Phase 5 legacy deprecation implementation

Started at operator request. Audit found new editor drafts already choose rclone,
legacy edits preserve engine/wire format and runtime routing, and existing
configuration guards reject path overlap including disabled profiles. No engine
or dependency removal or automatic conversion needed/authorized in phase 5.

Changes: src/app.rs adds persistent Fluent deprecation section for legacy
OneDrive Offline Modify only. English Fluent notice explains one-release support,
separate replacement/authentication, Preview/initial sync, verification before
old-profile removal and no sync-state/token copying. tests/onedrive_configuration.rs
adds equal/parent/child overlap regression for mirror/recovery against both
legacy roots, both drive types and both connection orders (disabled legacy), plus
separate engine-plan namespace and unchanged legacy-serialization coverage.
Existing guards reused; no new config format or runtime routing change.

Draft docs/releases/v0.5.4.md and index document two-release policy, preserving
unsynchronized local work, separate authorized folders and rclone authentication,
Preview/initial sync, verification and user-controlled old-profile removal.
Version remains draft; no release/package support claim. Engineering contract/
index updated; phase-7 retains installed legacy continuity/rendered notice gate.
Untracked Push Buttons.md untouched. No profiles, databases, caches, tokens,
services, cloud/local files or installed package modified.

Verification so far (source 0a1f951dec41c96966c32baeb9ad8f5872592bba, dirty=true):
- just fmt: exit 0.
- bash scripts/dev.sh check: exit 0, target/task-runs/dev-check-cv13tkte/.
- cargo test --locked --test onedrive_configuration: exit 0, ten passed.
- cargo test --locked --bin cosmic-ext-applet-mounter onedrive_configuration_:
  exit 0, three passed (new routing and legacy round trip).
- cargo test --locked --bin cosmic-ext-applet-mounter
  onedrive_runtime_dispatch_uses_engine_and_preserves_legacy_and_online:
  exit 0, one passed.
- bash scripts/dev.sh localization-check: exit 0,
  target/task-runs/dev-localization-check-89uxoe9f/.
- Final gate follows; no installed notice/legacy lifecycle pass claimed.


Final phase-5 source verification: bash scripts/dev.sh gate, exit 0, receipt/log
at target/task-runs/dev-gate-6_eqf2pd/ (same base SHA above, dirty=true). Formatting,
all-feature lint and all-target Rust/maintained Python tests passed. No failure
or skipped-as-pass claim. Release-note local links and git diff --check passed.
Phase-5 source tasks marked complete; installed notice/legacy continuity retained
explicitly in phase 7. No new release build/installation or live legacy sync run.

Files changed: src/app.rs, English Fluent file, tests/onedrive_configuration.rs,
docs/releases/v0.5.4.md (new draft), release-note index, OneDrive engineering
contract/index, Task List.md and completion notes. Existing cross-engine guards
were already implemented, so only regression coverage changed there. Remaining
risk/limitation: rendered layout of longer legacy notice and actual installed
legacy continuity not revalidated; legacy auth/engine dependencies deliberately
retained for 0.5.4. Next: commit reviewed source/docs, then include notice in the
next accepted package for phase-7 rendered/live review. No push, tag, publication,
profile mutation, dependency removal or data deletion performed.


### October 18, 2026 — Refresh phase-6 removal checklist before execution

Operator requested task-list update first, then work on removal. Refreshed
post-0.5.4 phase 6 against current code/packaging: no separate helper desktop
file; helper still installed via justfile/Flatpak payloads; retire only legacy
OneDrive executable/options, retaining onedriver Online, rclone onedrive backend
and shared OAuth. Added missing GTK3 helper recommendation, consumer-checked
Python/package handling, status/pending/diagnostic/process branches, inert legacy
profile controls, owned installed-monitor-unit retirement/upgrade acceptance and
preservation. Historical inventory/published deprecation evidence stays intact.
No installed package uninstall is implied by dropping recommendations.

Existing release policy still keeps legacy support through 0.5.4. Asked operator
whether work means separate future-release preparation or explicitly bringing
engine removal into 0.5.4. No removal code, service operations or release-policy
change performed while this answer is pending. Completed phase-5 uncommitted
source/docs and user Push Buttons.md preserved. Documentation update only;
phase-6 execution/acceptance checklist remains entirely open.


### October 18, 2026 — Numbering clarification; start migration acceptance

Operator clarified "6" meant the 0.5.4 acceptance section renamed from 7 to close
the gap after future engine removal was moved, not a request to implement engine
removal now. Earlier interpretation in the checklist-refresh entry is superseded;
0.5.4 legacy support policy is unchanged. Task List.md current acceptance heading/
references now say phase 6; future engine-removal section starts at item 1 in its
own numbering. Historical notes/engineering chronology retain original phase
numbers, with this mapping authoritative for current work. Removal checklist
refresh retained as future planning; no retirement code/services modified.

Started actual phase-6 acceptance. Existing installed cf2d13b/packaging 848d48b
supports recovery help/no-automatic-notice visual check; operator prompt sent.
New phase-5 legacy notice is still uncommitted/uninstalled, so its rendered review
requires later committed candidate, not a claim from current screenshot/source.
Reuse accepted phase-4 personal/corporate/SharePoint smoke and focused guard
receipts for unchanged behavior. No automatic branch creation, removal,
repeated engine tests/build, profile mutation or dependency change. Push Buttons.md
and all earlier phase-5 work preserved.


October 18 phase-6 visual acceptance: operator explicitly confirmed correct
recovery Browse/separate-authorization help and absence of Automatic recovery
directory notice on installed 848d48b, following earlier placeholder confirmation.
Operator noted the question had already been asked; do not repeat the accepted
check. Marked final rendered recovery guidance complete, superseding pending
visual-detail statements in historical records. No runtime/source change,
profile Save, test rerun/build or live sync needed. Remaining installed legacy
notice awaits a candidate containing phase-5 source changes; existing broader
acceptance items retain their current status. git diff --check exit 0.


### October 18, 2026 — Commit phase 5 and clarify source/package state

Operator requested commit and updated records after confusion between completed
recovery-wording installation and subsequent legacy deprecation work. This commit
includes phase-5 notice/source, two added compatibility regressions, draft 0.5.4
release notes/index and reconciled task/engineering records. Current phase 6 is
migration acceptance; future removal has independent item 1. Legacy engine
support remains in 0.5.4. Push Buttons.md excluded as user work.

Current package remains 848d48b pinned to cf2d13b, installed/verified at OSTree
757aae349f1017ea47eb93c88e6f1522b8ed9cef657a480e6f1b18826156e8a0. That candidate
contains the completed recovery wording fix, not this new legacy notice. Its
rendered recovery placeholder/help/no-automatic-notice check is complete and
must not be repeated. Phase 5 is source-complete, with no phase-5 build or install
yet; phase-6 installed legacy notice/continuity and other open acceptance gates
remain pending. No package build, install, service/profile mutation or publication
is performed by this commit.

Reused unchanged source verification: check dev-check-cv13tkte, ten configuration
integration tests, three editor engine/round-trip tests, one engine-dispatch test,
localization dev-localization-check-89uxoe9f, final gate dev-gate-6_eqf2pd: all
exit 0 (base 0a1f951, dirty=true). Only records changed since those receipts;
no repeated gate needed. Release-note links and final git diff --check exit 0.
Next: pin a replacement candidate to this committed source, build once and
inspect installed legacy notice; reuse previously accepted sync/recovery smoke.


### October 18, 2026 — Phase-6 deprecation-notice candidate build preparation

Operator clarified Push Buttons.md is GUI cleanup for after 0.5.4; leave that
user file untracked/unchanged and outside current staging. Requested phase-6
build. Source 2f137491e32c459129f0d77befea4f26faedcf19 is committed phase-5
notice/compatibility/guide work. Updated manifest/current packaging guide to pin
that immutable source. Replacement x86_64/master build is necessary because
installed 848d48b/cf2d13b lacks the notice; existing artifact cannot prove rendering
of new text. Runtimes, finish grants, Cargo inputs/features and engine behavior
unchanged. Prior check/focused/localization/gate receipts remain applicable to
unchanged source, with affected packaging-contract test repeated for pin only.
No engine removal; 0.5.4 legacy support retained. Build creates new artifact/
receipts, not an installation/publication; operator requested running this build.

Replacement pin checks: cargo test --locked --test flatpak_packaging_contract,
exit 0, four passed; git diff --check exit 0. No broad source gate repeated.


October 18 phase-6 build completed: packaging a0cf06024cdcf279f1ec226f70d26672fa310e7e
from clean /tmp/cloud-mounter-flatpak-packaging-a0cf060, source pin
2f137491e32c459129f0d77befea4f26faedcf19. Maintained build-flatpak.sh command
shown in chat, --arch x86_64 --branch master --local-source-repo "$PWD",
--output-dir "$PWD/target/artifacts/candidates/flatpak-a0cf060-x86_64-master",
--log-dir "$PWD/target/task-runs/flatpak-candidate-a0cf060": exit 0, succeeded.
Producer artifacts.json/summary.json/task.log retained in that log directory.
Export/bundle ref app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master,
OSTree 31ca8f2a612f9b26c788e1330830918daf7b02c378f7b3a131a7bff68333dba0.
Bundle 13,508,224 bytes, SHA256
65c9f8d1639ceae05b2c9229ee33a90addb4aea862d8b6133ec0157ccafe4756.
Complete metadata and runtime provenance equal prior 848d48b candidate. No build
failure; no changed grants/runtime or publication. Installed app remains 848d48b;
new candidate is not installed or rendered yet. Requested operator window close,
paused syncs and panel removal before receipt-verified replacement install.
GUI cleanup Push Buttons.md stays for post-0.5.4 and is untouched/untracked.
Prior sync/recovery/guard source receipts reused; no repeat tests needed for
this unchanged built input. Records-only change; git diff --check exit 0.


October 18 phase-6 installation: operator confirmed windows closed, syncs paused
and applet removed from panel. bash scripts/install-flatpak.sh
--candidate-receipt "$PWD/target/task-runs/flatpak-candidate-a0cf060/artifacts.json"
--log-dir "$PWD/target/task-runs/flatpak-install-a0cf060": exit 0, succeeded.
Maintained installer revalidated candidate reuse/hash/metadata and installed ref/
commit. Receipt target/task-runs/flatpak-install-a0cf060/installation.json;
summary/task.log alongside. Installed OSTree
31ca8f2a612f9b26c788e1330830918daf7b02c378f7b3a131a7bff68333dba0;
previous rollback commit
757aae349f1017ea47eb93c88e6f1522b8ed9cef657a480e6f1b18826156e8a0.
No installation failure or runtime/grant change. Legacy notice is now packaged,
but rendering and legacy/Online continuity still unverified. Next panel re-add,
open an existing legacy OneDrive Offline Modify screen, inspect notice without
Save/auth/sync; keep original profiles and test data intact. Do not repeat accepted
recovery wording/sync tests. Push Buttons.md unchanged; no publication.


October 18 installed a0cf060 legacy notice acceptance: operator opened identified
existing legacy OneDrive Offline Modify and confirmed Legacy label and replacement
instructions. Marked rendered notice separately complete; no new auth, Save,
conversion or sync performed. Read-only selected-profile unit-file inspection
also confirms existing legacy service file retained, ExecStart still uses
onedrive and --monitor. No arguments/paths, organization/account data or tokens
printed. This verifies preserved installed launch declaration, not successful
engine execution. Actual legacy/Online runtime acceptance remains open.

Split combined notice/continuity task so visual success is not confused with a
runtime pass. Recovery visual and current legacy-notice visual observations need
no repeat. Source/package inputs unchanged; no gate/build needed for records.
User Push Buttons.md retained for post-0.5.4 cleanup. git diff --check exit 0.


October 18 installed legacy continuity started at operator request for named
existing legacy mirror. Read-only saved-profile metadata: absent engine resolves
legacy; configured local directory exists (not symlink); legacy config,
refresh_token, items.sqlite3 and initial-sync-complete files present. Presence is
not proof credentials remain valid. Managed legacy service file retained.
Read-only systemd D-Bus Manager.GetUnit returned NoSuchUnit; service currently
not loaded, so no running-monitor success inferred from enabled config flag.
No service start, sync or user configuration/file change performed. Only a
temporary UUID selector under /tmp used for bounded D-Bus inspection; no token
or account content printed.

Source inspected for next operator action: legacy Test Connection invokes fixed
onedrive --sync --dry-run --verbose through maintained app runner with timeout,
not ordinary transfer. Next operator performs that installed UI check and
reports result; do not Start or Sync Now yet. Engine authentication/preview and
later live continuity remain pending. Records-only, no source/gate/build change.


October 18 installed a0cf060 legacy Preview passed. Operator screenshot explicitly
shows Preview completed for existing legacy mirror, onedrive reported 19 notable
file-change lines. This is actual installed legacy dry-run command success and
remote access, not transfer, exact changed-file count or background-monitor
acceptance. Operator used Preview (screenshot), not inferred Test Connection.
No pending changes applied; existing monitor was not started by agent. Marked
specific legacy Preview subcheck complete, kept monitor/transfer and onedriver
Online open. Next existing OneDrive Online validation; preserve legacy pending
changes and do not request a real sync simply to prove engine launch. Source/
package inputs unchanged, prior tests reused; git diff --check exit 0.


October 18 installed a0cf060 OneDrive Online validation passed. Operator screenshot
shows existing personal Online connection Test Connection succeeded: managed
online mount unit validates structurally, onedriver setup present, cache created
if needed. Account/mountpoint/cache values not copied into records. This is
installed setup/plan validation, not actual engine mount/read/unmount success.
Added separate completed subcheck and kept remaining operation checks open.
Next operator closes editor, activates only this Online connection, observes
existing folder listing without editing and deactivates; no legacy sync/auth
or saved-configuration change needed. Source/package inputs unchanged; no
repeat source checks/build, git diff --check exit 0. Push Buttons.md untouched.


October 18 installed a0cf060 onedriver Online operation passed at operator
observation level: following instructions to activate existing Online connection,
view files without edits, close file manager and deactivate, operator reports
files appeared, file manager closed and Online mount stopped. Marked actual
mount/list/unmount subcheck complete in addition to prior setup/structural Test.
No transfer/edit claim, independent D-Bus/mount trace or restart/sleep/VPN scope
inferred. Legacy Preview's 19 notable change lines remain unapplied; legacy
monitor/transfer acceptance stays open. Source/package inputs unchanged; no
redundant gate/build. Next acceptance gaps are implemented rclone safety guards
on approved disposable fixtures/profiles, reusing existing automated evidence
without mutating real legacy mirror. git diff --check exit 0. User GUI cleanup
file unchanged; records not yet committed.


### October 18, 2026 — Phase-6 safety guard acceptance started

Operator requested guard step. Read retained dev-gate-6_eqf2pd/task.log and mapped
passing tests: onedrive_guard_saved_target_edits_stop_before_remote_access,
onedrive_guard_rejects_remapped_config_and_denied_or_malformed_graph_metadata,
onedrive_guard_rejects_shortcut_notebook_foreign_folder_and_incomplete_stat,
onedrive_plan_binds_initialization_to_target_and_excludes_browser_items;
coordinator local_anchors_and_owners_reject_missing_changed_and_linked_data,
onedrive_anchor_guards_reject_missing_changed_and_linked_markers,
missing_post_sync_anchor_never_initializes_or_runs_checksum. These are
synthetic/fake-adapter source tests, not installed refusal evidence. Runtime
inputs under them unchanged since passing gate; no repeat requested/run.

Installed plan: first unsaved personal draft pointing at existing business remote
should refuse wrong drive type, then reciprocal direction as needed. Read-only
verify-existing action, not Connect new remote or Save; no auth/token replacement.
Later missing-marker/retarget checks only against approved disposable rclone
resources. Legacy mirror, its pending changes, credentials and real data stay
unchanged. Marker perturbations must restore original bytes/ownership without
resync or recreating absent production markers. No mutation yet; task subchecks
track actual installed refusal separately. git diff --check exit 0.


October 18 installed a0cf060 first wrong-drive negative check passed: following
instructions for unsaved personal OneDrive Offline draft selecting existing
business remote, operator screenshot reports primary drive does not match saved
drive/account type and synchronization blocked. No Connect new remote or Save
requested; no sync execution/result inferred beyond UI refusal. This tests
personal-vs-business type mismatch, not account-only mismatch or every-run saved
binding checks (those have separate fake-adapter evidence). Next reciprocal
business draft selecting existing personal remote, still unsaved/read-only.
Source/package inputs unchanged; no repeated gate/build. git diff --check exit 0.


October 18 installed a0cf060 reciprocal wrong-drive negative check passed:
operator screenshot after business-type draft selecting existing personal remote
reports same primary drive/account-type mismatch refusal. Both drive-type
selection directions now observed without Save/new remote/authentication. Next
separate personal account-only mismatch using matching personal remote/type and
synthetic expected email; existing credential is used read-only and no sign-in
as synthetic account requested. Type mismatch and account mismatch are not
conflated. Business expected-account resolution uses different Graph resource
mapping and will be recorded separately if exercised. No source/package change
or repeated automated tests/build; git diff --check exit 0.


October 18 installed a0cf060 account-only mismatch passed: unsaved personal draft
with matching personal remote/type but synthetic expected email yielded explicit
Microsoft returned different account refusal. No new sign-in, remote creation,
Save or sync requested. Wrong-drive both types and personal wrong-account checks
complete under the checklist's wrong-account-or-drive scope. Business-specific
expected-account resource mapping remains covered by retained fake evidence,
not a new live account-only claim.

Prepared missing-local-marker check on already approved disposable personal
Acceptance-Personal-2 profile (paused): one regular non-symlink anchor found,
contents match its connection UUID. Baseline captured under
 target/task-runs/phase6-personal-marker-baseline.json (private mode 0600): three
local-file hashes, seven initialization/listing hashes. No contents or credentials
printed; no marker/profile/cloud mutation performed. Next operator temporarily
renames that original marker, tries existing profile Sync Now (expected refusal),
then restores same filename without editing bytes. Never recreate marker, reset
initialization, Repair/resync or alter real legacy mirror. Refusal/restoration/
unchanged-state evidence remains pending. git diff --check exit 0.


October 18 installed a0cf060 local-marker refusal observed: operator screenshot
shows disposable personal Sync Now failed because ownership marker missing or
changed, synchronization blocked. Read-only baseline comparison: original marker
still absent, .held original present with unchanged byte hash; both user test
files unchanged, all seven retained initialization/listing hashes unchanged.
No agent restoration/mutation or cloud read needed. Refusal did not reset local
initialization or rewrite retained listings; no remote-mutation proof inferred
from local hashes alone. Restore original file name before further operations;
normal-sync recovery and restoration verification remain pending. No Repair,
resync, new marker or source/build/test run. git diff --check exit 0.


October 18 installed a0cf060 personal missing-marker recovery passed: operator
screenshot after restore reports Sync Now synchronization/checksum completed.
Read-only baseline comparison confirms original regular marker restored with
same byte hash, .held absent, both user-file hashes unchanged and original
initial-sync-complete hash preserved. Listings may legitimately advance after
successful normal sync; their unchanged hashes were verified specifically during
the refused operation, not claimed unchanged after this success. No new initial
confirmation, Repair or resync requested. Marked personal installed marker smoke
complete; shared malformed/remote-marker cases retain prior fake evidence.

Next changed-target check: temporarily change only local directory on this
paused disposable profile to a new empty Browse-authorized directory, Save, then
request Sync Now without Preview. Expect fresh initialization/Preview requirement;
never confirm or execute a new initial sync for this negative check. Afterwards
restore original Browse-selected local target and Save. Original files, marker
and target-bound initialization baseline remain retained; no real legacy
profile or remote folder mutation intended. No new target change performed yet.
Source/package inputs unchanged; git diff --check exit 0.


October 18 installed a0cf060 changed-local-target refusal passed: operator
screenshot after saving disposable profile local retarget reports Sync Now
failed, initial synchronization requires a successful Preview. Old initialized
target approval did not authorize new target. Read-only baseline check confirms
original three local file/anchor hashes unchanged, original initialization hash
retained, only one initial-complete marker for this connection, and newly selected
Acceptance-Personal-Retarget directory exists and is empty. No successful Preview
or new initial sync requested or inferred. No cloud-mutation claim from local
inspection alone; refusal is before initial execution by source guard.

Target restoration/ordinary-sync recovery pending: Browse back to original
Acceptance-Personal-2, Save and normal Sync Now, no new initial confirmation.
Remote and recovery targets remain unchanged by test instructions. Do not reset
or copy initialization state, recreate anchors, Repair or delete test folders.
No source/package change; git diff --check exit 0.


October 18 installed a0cf060 retarget recovery passed: after restoring original
local mirror with Browse/Save, operator screenshot reports normal Sync Now and
checksum verification completed successfully. Read-only check confirms original
three local file/anchor hashes unchanged, original initial-complete marker/hash
retained, retarget directory still empty. No Preview/initial confirmation,
Repair, reset or copied state needed to return to original initialized target.
Marked changed-local-target subcheck complete. Guard parent remains open for
explicit remote-folder-loss smoke; corporate marker/retarget not claimed live.

Next personal disposable remote-folder-loss check: operator temporarily renames
only CloudMounter-Flatpak-Acceptance-Personal-2 in OneDrive, keeping contents;
existing profile Sync Now must reject missing configured subtree, then operator
restores original folder name. No deletion or saved target/credential change.
No folder rename/cloud mutation by agent yet. Original legacy unaffected.
Runtime/package inputs unchanged; git diff --check exit 0.


October 18 remote-folder-loss action clarification pending: operator reports sync
completed because connection was not saved. Source routes Sync Now to the saved
profile, so an unsaved editor-path edit is not a missing-remote-folder test.
Read-only saved metadata confirms original personal remote subtree and local
mirror remain configured. Asked whether actual folder was renamed in OneDrive
website, only draft field changed, or statement refers to earlier local test.
No remote-loss pass or guard defect claimed from ambiguous action. Actual remote
folder rename needs no connection Save; keep saved expected path unchanged.
Existing accepted earlier refusal/restoration evidence unchanged pending exact
clarification. No source/config/cloud mutation or repeated tests/build.


October 18 clarification received: operator changed only remote-folder field in
Modify without Save. Successful Sync Now used unchanged saved target, as designed;
this was not the intended actual remote-folder rename. Remote-folder-loss check
remains pending, not failed/passed. Close draft without Save, then rename actual
disposable folder in OneDrive website while keeping saved target fixed. No
configuration or cloud changes performed by agent; prior guard evidence retained.


October 18 installed a0cf060 actual missing-remote-folder refusal observed:
operator screenshot after instructed OneDrive website rename reports Microsoft
verification HTTP 404 and synchronization blocked. This is the expected missing
resource refusal, separate from prior unsaved-editor confusion. Read-only baseline
check confirms all three local file/anchor hashes unchanged and original
initial-sync-complete hash retained. No reset, initialization, cloud recreation
or deletion performed by agent. Restore actual remote folder's original name,
then normal Sync Now/checksum recovery remains pending. No connection Save or
re-authentication needed for restoration. Source/package inputs unchanged;
git diff --check exit 0.


October 18 installed a0cf060 remote-folder restoration passed: operator screenshot
following original-name restoration reports normal Sync Now/checksum completed.
Read-only saved configuration confirms original expected remote subtree (no -held)
and original local mirror; baseline confirms original local file/marker hashes
and initialization hash preserved. Screenshot still shows -held in unsaved editor
field with verification required; it did not change the saved runtime target.
Close editor without Save; no synthetic identity/draft-path changes should persist.

Closed scoped guard-smoke item with explicit verification levels: reciprocal
personal/business drive-type refusals and personal account mismatch; personal
missing local anchor, changed local target, missing remote folder, each restored
and followed by successful ordinary sync/checksum. Retained source/fake tests
cover remaining malformed/shared/corporate cases; no claim of every negative on
every live account. No remote contents deleted, original legacy untouched, no
new auth, Repair, resync or recreated/reset state. Full native/Debian/Flatpak
lifecycle acceptance remains separate/open; legacy real background transfer also
still untested. No source/package change or repeated gate/build. git diff --check
exit 0. Records remain uncommitted; Push Buttons.md untouched.


### October 18, 2026 — Phase-6 remaining lifecycle acceptance: Repair preparation

Operator requested next phase-6 items. Mapped completed installed setup/manual/
restart/scheduled/Pause/recovery/guard/Online smoke; no duplicate suite needed.
Outstanding package-specific lifecycle includes installed Repair, controlled
sleep/wake and VPN; Native/Debian remains separate from installed Flatpak.
Retained gate dev-gate-6_eqf2pd includes passing coordinated cancellation,
unowned-running rejection, scheduled-stop and failed-reset Repair tests; these
fake-adapter passes reused, not promoted to installed acceptance.

Read existing Offline Repair contract/source: two-step confirmation (second
activation after >=1s, within 30s), authoritative owned units, stop/disable timer
and service, preserve bisync state/data, no preview/sync/resync side effect,
finish Paused. Repair UI requires failed/interrupted/inconsistent operation.
Selected approved disposable personal mirror only. Baseline at
 target/task-runs/phase6-personal-repair-baseline.json records mirror, recovery
and connection-specific rclone-bisync file hashes, including original anchors/
initial state. No token contents or file contents logged; private baseline.
No engine/marker/service mutation performed by agent. Next operator briefly
renames local marker, requests Sync Now to obtain controlled failure, restores
same marker, then uses popup Repair two-step confirmation without another Sync
Now. This repeat refusal sets Repair precondition; it is not a repeat guard
acceptance run. Legacy real mirror and all pending user changes untouched.


October 18 Repair setup correction: operator obtained expected local-marker
refusal; read-only inspection finds original still .held and all other 16 Repair
baseline files unchanged. Coordinator state remains succeeded/sync from previous
operation, because local marker validation returns from prepare_inputs before
Running/Failed state publication. Therefore this preflight guard refusal is not
a sufficient Repair eligibility setup; previous preparation instruction corrected,
not counted as Repair failure or successful Repair acceptance.

Source review: remote anchor cat/validation occurs inside run_transaction after
Running publication, before bisync request execution; refusal is persisted as
Failed. Corrected operator plan: restore local original marker, rename only
remote anchor in approved disposable subtree (not whole folder), request Sync
Now once, restore remote original marker without another sync, then popup Repair
with two-step confirmation. This creates genuine coordinator failure without
file-transfer/resync and without synthetic state-file edits. No agent marker/
cloud mutation, no legacy change, no runtime source fix implied. Repair baseline
retained. git diff --check exit 0.


October 18 corrected Repair precondition observed: operator restored local
marker, then renamed actual remote marker by prepending .held (website did not
append suffix). Expected remote name is absent while original bytes remain in
renamed file. Installed Sync Now reports missing/changed ownership and blocks.
Read-only coordinator state confirms failed/sync; all 17 baseline mirror,
recovery and rclone-bisync file hashes remain present/unchanged, including local
anchor/initialization/listings. Genuine recoverable coordinator failure now
established; no transfer/resync needed. Remote restoration still pending.
Next restore remote name exactly, no extra Sync Now, then popup two-step Repair.
No source/package mutation or repeated gate/build. git diff --check exit 0.


### October 18, 2026 — Missing Repair after coordinator failure: source fix

Operator reported no popup Repair after remote-marker refusal/restoration.
Read-only backend inspection still failed/sync. Source confirms cached_offline_state
used only service/timer activity; observe_ui_units read coordinator warnings but
ignored state. Thus stopped timer/inactive unit hid a failed manual sync as
Paused. Original seventeen mirror/recovery/bisync files remain baseline evidence;
no live state reset/Repair/service change performed by agent.

src/app.rs now carries matching typed coordinator states in UnitObservation,
collected through existing bounded background path. Matching Failed/Cancelled
state exposes failure/Repair despite paused units; active service and pending
manual work retain priority. Fresh observation replaces prior coordinator map,
so repaired Paused state clears failure; existing generation/config guards remain.
Malformed/foreign state marks that connection's observation failed and cannot
authorize controls. Missing state retains service/timer behavior; timed-out read
marks offline observations unavailable rather than authorizing cached success.
Warnings for valid connections remain available even if another state is bad.
No new UI strings, host commands, permissions, schema or engine behavior.

Three focused regressions cover six rclone providers, failed/cancelled paused
units, active/pending work priority, repaired-state clearing vs stale generation,
and temp-file missing/malformed/foreign typed state isolation. Existing Fake
systemd Repair sequencing evidence remains applicable; no destructive real test.
Updated existing Repair engineering contract/index, active task and these notes.

Verification (base a0cf06024cdcf279f1ec226f70d26672fa310e7e, dirty=true):
- just fmt: exit 0.
- bash scripts/dev.sh check: exit 0, target/task-runs/dev-check-bu_3z1za/.
- cargo test --locked --bin cosmic-ext-applet-mounter popup_coordinator_:
  exit 0, three passed. No failed focused run.
- git diff --check: exit 0; no new Fluent/desktop/AppStream input, so no duplicate
  localization/metadata check required. Final gate follows.
Next committed source/replacement candidate and installed Repair retest against
retained Failed state; no additional fault injection or manual state editing.


Final Repair-visibility source gate: bash scripts/dev.sh gate exit 0, receipt/log
 target/task-runs/dev-gate-s4od_kr5/ (base a0cf060, dirty=true). Formatting,
all-feature lint, all-target Rust and maintained Python passed. No gate failure.
Focused compilation/three regressions above remain valid; no UI string/metadata
inputs changed. git diff --check exit 0. Source correction complete but uncommitted
and not installed. Installed applet still a0cf060 with hidden Repair defect;
no claimed Repair execution. Current coordinator Failed state is preserved for
replacement, so no repeat marker disturbance is necessary. Files changed for
fix: src/app.rs, existing Repair engineering record/index and task/evidence notes.
Earlier pending build/install/guard/lifecycle observations retained chronologically.
Next approved commit and pinned candidate build/install, then Repair against same
saved failure, compare 17-file baseline and only then normal sync. No state/file/
cloud reset, new permission, engine removal or Push Buttons.md changes.


### October 18, 2026 — Repair replacement source committed and pinned

Operator authorized commit, build and installation. Source commit
67b3aec06007bddd987f87b79e45dccc8c62f376 includes Repair visibility fix and
preceding phase-6 acceptance records. Existing gate dev-gate-s4od_kr5 and three
focused regressions apply unchanged. Manifest and packaging guide now pin this
source; dependencies, shipping features, runtimes and permissions unchanged.
Replacement is necessary because installed a0cf060 omits the popup correction.
Flatpak x86_64/master only; no redundant native/Debian build. Build/install and
real Repair remain pending. Preserve retained Failed state and 17-file baseline.
User's untracked Push Buttons.md remains excluded as post-0.5.4 GUI work.

Packaging verification: cargo test --locked --test flatpak_packaging_contract
exit 0, four passed; git diff --check exit 0. Application source unchanged since
67b3aec, so the passing source gate is reused. Packaging pin commit follows;
installation readiness requested while building.


### October 18, 2026 — Repair visibility replacement built and installed

Source 67b3aec06007bddd987f87b79e45dccc8c62f376; packaging
82c1d7a29906723dda12d5eb30eb2ada8e2e65c5, retained clean packaging checkout
/tmp/cloud-mounter-flatpak-packaging-82c1d7a. User confirmed editor/settings
closed, syncs paused and applet removed from panel before replacement.
Maintained build-flatpak.sh exit 0, receipt/log
target/task-runs/flatpak-candidate-82c1d7a/. Single build; no retry/rebuild.
Complete export/bundle metadata, runtime identities, finish args and SDK
extensions equal preceding a0cf060 candidate. App source pin changed only.
Bundle SHA256 db8569d85c2532ac7a4d6b992f0f2ebc5e9105a6b813411841c164ee541ca80f;
metadata file SHA256 f791b92fc3866e73e401ea29f45367f7f7834fa7c6458171d71d1dfa3ae82cc3.
Maintained install-flatpak.sh exit 0, receipt/log
target/task-runs/flatpak-install-82c1d7a/. Complete candidate reuse verified
before install; installed ref app/io.github.uutzinger.cosmic-ext-applet-mounter/
x86_64/master and OSTree identity match inspected bundle/export:
942e1805dd8e02156290d2b05bed0d47c5fda61053eb6ddd8099b0d731a6b1b3.
Rollback identity retained:
31ca8f2a612f9b26c788e1330830918daf7b02c378f7b3a131a7bff68333dba0.
Read-only baseline recheck after install: all 17 original files unchanged, zero
missing. Before install coordinator remained failed/sync; no engine, cloud or
coordinator reset performed. No permissions/runtime change, push/publication or
user GUI cleanup file inclusion. Source check/focused tests/gate and four
packaging contracts reused as above; record-only follow-up needs no rebuild.
Remaining risk: installed rendered Repair visibility/execution not yet accepted.
Next: re-add panel applet, open popup and Repair retained failed personal test
with existing two-step confirmation. Inspect Paused units and unchanged 17-file
baseline before subsequent explicit normal sync. Phase 6 remains open.


### October 18, 2026 — Installed Repair completed and preservation verified

Operator screenshot confirms popup Repair now visible on installed 82c1d7a.
Operator ran Repair for disposable personal access test; rendered completion
reports timer/service paused and bisync safety lock/recovery preserved. Read-only
state confirms coordinator paused, operation repair, matching connection UUID.
All 17 baseline mirror/recovery/bisync files still present with identical SHA256.
Typed session D-Bus GetUnit initially exited 1 with NoSuchUnit: inactive repaired
units already unloaded. Follow-up read-only ListUnits/GetUnitFileState exited 0:
service and timer both absent from loaded units and disabled on disk. This is
consistent with stopped/disabled runtime; no extra service loading/start/reset
performed. Filesystem/coordinator inspection exit 0. No source change or repeated
source tests needed; remaining recovery acceptance is explicit subsequent normal
Sync Now, without initialization/resync and without enabling background timer.
Phase 6 stays open for that and sleep/wake/VPN and other recorded scope.


### October 18, 2026 — Post-Repair normal sync accepted

Operator screenshot confirms Sync Now completed for the disposable personal
access test, including OneDrive checksum verification on installed packaging
82c1d7a/source 67b3aec. Read-only coordinator inspection: succeeded/sync. The
existing initialization file remains byte-identical to pre-Repair baseline;
no Preview, initial synchronization or resync requested. Typed systemd session
D-Bus ListUnits/GetUnitFileState: service and timer disabled and unloaded after
manual sync, so background scheduling remains paused. Both inspections exit 0.
All 17 baseline files had already been verified unchanged immediately after
Repair, before this normal sync; bisync listings may update during normal sync.
Marked only disposable personal installed Repair/recovery-to-normal smoke done.
Phase 6 remains open for sleep/wake, VPN policy and other listed acceptance.
Task/evidence and existing Repair contract/index updated; git diff --check exit
0. Documentation-only acceptance records reuse previous passing source gate;
no build, service mutation, cloud mutation or repeated source tests performed.


### October 18, 2026 — Focused sleep/wake scope and reboot checkpoint

Operator reports existing laptop sleep/wake display-driver failures, potentially
requiring hard reboot. Save progress before any suspend attempt. Installed Repair
and subsequent checksum-verified normal sync accepted as above; no repeat needed.
Created private checkpoint at
target/task-runs/phase6-sleep-wake-checkpoint/checkpoint.json: current boot ID,
source/package identity, succeeded/sync coordinator snapshot and fresh SHA256
for 17 mirror/recovery/bisync files after the completed normal sync. Original
Repair baseline remains unchanged as historical preservation evidence.

First smoke is one idle manual suspend/resume with the disposable personal mirror
paused, no transfer running, no scheduled trigger and no legacy/profile changes.
Before suspend save other work. After normal wake inspect applet/backend and
checkpoint before one explicit Sync Now; successful checksum result with background
still paused is the scoped acceptance. Active-transfer cancellation and scheduled
wake behavior are not proven by this smoke. If display fails and reboot is needed,
stop further suspend attempts; reopen this repository/chat, inspect preserved
checkpoint/state before sync, and record wake as inconclusive plus separate reboot
recovery. Do not Repair, delete locks, resync or reset markers speculatively.
No suspend/reboot/service mutation performed by agent. VPN remains a separate
open item. Documentation/checkpoint creation exit 0; diff-check follows; no source
changes or build/tests needed. Acceptance records committed before proceeding.


### October 18, 2026 — Idle wake returned; pre-sync checkpoint intact

Operator reports popup opens with the same notification after sleep/wake.
Read-only checkpoint comparison exit 0: same boot ID (no reboot since checkpoint),
all 17 files present and byte-identical, coordinator still succeeded/sync. Typed
systemd session D-Bus inspection exit 0: test service and timer remain disabled
and unloaded. No unexpected background resume or file mutation observed. Existing
notification alone is retained UI result, not evidence of a new sync or failure.
No Repair, resume, marker/state modification or service change performed.
Scoped wake acceptance still awaits one explicit post-wake Sync Now and result.
No claim about active-transfer interruption or scheduled wake behavior.


### October 18, 2026 — Focused idle sleep/wake smoke accepted

Operator screenshot confirms post-wake Sync Now completed with OneDrive
checksum verification on disposable personal test, installed packaging 82c1d7a.
Read-only coordinator inspection succeeded/sync; typed systemd D-Bus confirms
service and timer both disabled/unloaded after manual sync. Inspection exit 0.
Pre-sync same-boot check and all 17 checkpoint hashes matched as recorded above.
Marked scoped idle/paused mirror sleep/wake smoke complete. No hard reboot or
Repair needed. This does not establish active-transfer interruption, scheduled
resume, Online busy-mount wake, other providers or native/Debian acceptance.
No further suspend requested on the display-driver-unstable laptop for this smoke.
Task/evidence records updated; git diff --check exit 0; no source/build inputs
changed and no redundant tests/build. Next open phase-6 mirror item is VPN-bound
readiness/lifecycle; other existing acceptance boundaries remain in the task list.


### October 18, 2026 — VPN-bound mirror acceptance started

Read current VPN activation/readiness code and OpenConnect guide. Manual sync
activates configured NetworkManager VPN if absent, waits boundedly for tunnel
and profile readiness before engine work; scheduled readiness checks never
initiate interactive authentication. VPN ownership cleanup is separate evidence.
Read-only saved configuration: disposable personal mirror has vpn_profile_id None;
existing NetworkManager OpenConnect profile available, alongside other profiles.
Typed NetworkManager D-Bus active-connection inspection exit 0: no active VPN.
No tokens/settings secrets read or exposed; no VPN activated/disconnected by agent.
First operator step: assign existing OpenConnect profile to personal test through
Modify/Save, retain paused timer. Then inspect saved mapping before activation
and readiness/cancellation/normal-sync checks. Restore original None assignment
after tests. No unrelated mounts, real legacy data or VPN profiles changed.
Task started; no VPN acceptance claimed. Documentation-only diff-check exit 0.


October 18 VPN test sequencing correction: operator selected VPN and Save opened
OpenConnect authentication before planned Sync Now. Source inspection confirms
this is the existing OneDrive save validation path: save_draft invokes
validate_onedrive_connection_for_save, which requires configured VPN readiness
before remote verification and persistence. Prior instruction implying Save
would not activate VPN was incorrect. No source defect established. Use this
prompt for cancellation negative test first: cancel authentication, inspect
Save-blocked outcome and unchanged saved VPN assignment/data before positive
authentication retry. No credentials requested in chat or entered by agent.


### October 18, 2026 — VPN authentication cancellation blocked Save

Operator screenshot: Save blocked for disposable personal test because VPN
authentication was cancelled or activation failed. Read-only saved configuration
confirms vpn_profile_id remains None; coordinator still succeeded/sync from prior
manual operation. Typed NetworkManager D-Bus reports no active VPN, and systemd
D-Bus confirms test service/timer disabled and unloaded. Inspections exit 0.
This accepts installed Save-time authentication cancellation/no persistence smoke,
not a scheduled VPN failure or mid-transfer interruption test. No source change.
Next retry Save, authenticate in local OpenConnect dialog, inspect successful
saved assignment and tunnel readiness before one manual sync. Keep timer paused.


### October 18, 2026 — VPN authentication and bound Save passed

Operator completed local OpenConnect authentication and supplied successful Save
screenshot: managed mirror schedule disabled, initialization retained, subsequent
Preview/Sync Now use normal bisync. Read-only config confirms personal test now
has a VPN assignment. Typed NetworkManager D-Bus reports one activated VPN
(ActiveConnection State 2, VPN State 5); systemd D-Bus reports test service and
timer disabled/unloaded. Existing initialization file SHA256 unchanged. Both
inspections exit 0; no agent activation or service mutation. Next one manual
Sync Now while tunnel is active; then inspect ownership/lifecycle and restore
original No VPN assignment. No scheduled or interrupted-transfer claim.


### October 18, 2026 — VPN-bound manual sync accepted

Operator screenshot reports OneDrive synchronization and checksum verification
completed. Read-only coordinator succeeded/sync; typed systemd D-Bus service/
timer remain disabled/unloaded. NetworkManager D-Bus VPN still activated
(ActiveConnection 2, VPN 5). Saved disconnect_vpn_when_unused is false, so
retaining VPN matches policy; no automatic disconnect failure inferred.
Inspections exit 0. Positive tunnel readiness/manual sync accepted, with no
scheduled, mid-transfer or ownership-release claim. Next operator-controlled
check enables explicit applet-owned VPN disconnect permission, saves and runs
one normal sync; shared-user protection must remain in force. Restore original
No VPN and disconnect false afterward. No source/build changes.


### October 18, 2026 — Applet-activated VPN released after manual sync

Operator enabled disconnect permission, saved and ran Sync Now; screenshot
reports successful synchronization/checksum. Read-only configuration confirms
VPN assigned and disconnect_vpn_when_unused true. NetworkManager D-Bus now has
no active VPN; prior authenticated tunnel was activated by this applet during
Save. Coordinator succeeded/sync; test service/timer remain disabled/unloaded.
Both inspections exit 0. Scoped applet-activated, unshared VPN release smoke
passed. This does not prove pre-existing/shared-tunnel protection live, scheduled
readiness or mid-transfer VPN loss. No agent disconnect/service mutation.
Next restore original No VPN assignment and disconnect permission false through
Modify/Save; inspect persistence, initialization and paused units before closing
this scoped VPN test. No new build/source tests required.


### October 18, 2026 — Scoped VPN test restored and accepted

Operator restored No VPN, disconnect permission off and saved; screenshot reports
managed mirror schedule disabled and existing initialization preserved. Read-only
config verifies vpn_profile_id None and disconnect_vpn_when_unused false. Existing
initialization marker SHA256 unchanged. Typed NetworkManager D-Bus reports zero
active VPNs; systemd D-Bus test service/timer disabled and unloaded. Inspections
exit 0. Completed focused installed manual VPN acceptance: authentication cancel
blocks Save without persistence, positive auth/readiness Save, checksum-verified
bound sync, VPN retained without release permission, applet-activated VPN released
with permission, and original connection options restored.

Updated active checklist to distinguish this completed smoke from still-unproven
scheduled VPN-unavailable behavior, live pre-existing/shared-tunnel protection
and active-transfer sleep/VPN-loss cases. Parent Flatpak lifecycle/phase 6 remain
open pending scope/evidence reconciliation and other already listed work. No
source change, rebuild, service mutation, credentials or unrelated GUI work.
Documentation diff-check exit 0; passing source gate reused, not rerun.


### October 18, 2026 — Phase-6 evidence reconciliation and scoped closure

Operator explicitly chose to transfer native/Debian lifecycle and actual legacy
monitor/transfer to final release acceptance. Kept those items open there with
original checklist wording, plus disposable-legacy-only instruction; real legacy
pending user changes remain untouched. Broader scheduled VPN-unavailable, live
pre-existing/shared VPN protection and active-transfer sleep/VPN-loss are likewise
explicit final gates, not implicitly passed by idle/manual smoke. The parent
Flatpak lifecycle item remains open at the final gates for missing scenarios.

Read retained passing dev-gate-s4od_kr5 output (a0cf060 dirty source subsequently
committed as 67b3aec). Existing passing tests cover VPN activation/shutdown
ownership decisions, shared reference counts, bounded/cancellable readiness,
finite-user release, sleep token invalidation, coordinator cancellation and
legacy/Online/rclone engine dispatch. Relevant exact tests include
activation_and_shutdown_decisions_respect_readiness_and_ownership,
usage_snapshots_reference_count_shared_dependencies,
finite_offline_vpn_users_are_reference_counted_and_release_is_non_destructive,
deadline_bounds_a_hung_probe_and_never_permits_storage,
sleep_invalidates_old_operations_even_after_wake,
required_phases_stop_on_failure_timeout_and_cancellation and
onedrive_runtime_dispatch_uses_engine_and_preserves_legacy_and_online. Reused
these source/fake receipts unchanged; they do not prove unrun live scenarios.

Condensed phase-6 checklist around accepted levels, transferred open work and
receipt identities. Detailed chronological failures/fixes/acceptance above and
Git history retained. Corrected installed identity to 82c1d7a/source 67b3aec.
Reconciled draft release notes: rendered legacy notice now accepted, Repair fix,
Browse requirements, Save-time VPN authentication and limited installed scope.
Manual replacement/0.5.4 legacy support and post-0.5.4 removal policy unchanged.
Updated existing OneDrive engineering record/index; no new engineering artifact.

Phase 6 closed for documented Flatpak migration smoke only. Final migration
gates, consolidated sandbox/package release acceptance, version metadata and
publication remain outstanding. No new source change, package rebuild, test
rerun, installation or service/cloud mutation is necessary for record-only
closure. Checks and final worktree disposition follow. Push Buttons.md untouched.


Closure validation: git diff --check exit 0. Focused Python documentation check
exit 0: all local file links in changed task/release/engineering records resolve,
phase-6 section has no outstanding [ ] item, transferred final gates remain
open and release notes retain Draft status. Existing source check/focused tests/
gate receipts apply because application, scripts, packaging and Fluent inputs
are unchanged. No check failures. Changed files: Task List.md, this record,
docs/engineering/README.md, onedrive-rclone-offline-contracts.md and draft
docs/releases/v0.5.4.md. Remaining risks are the explicit final migration/package
gates. Next work is consolidated final acceptance, not another phase-6 build.


### October 18, 2026 — Final acceptance reordered into one sequence

Operator requested merging the transferred phase-6 migration gates and package
acceptance/release checklist. Replaced the two parallel sections with eight
dependency-ordered steps: evidence/matrix and release-input freeze; build/reuse
and artifact/grant inspection; auth/portal access; mirror/legacy continuity;
VPN/sleep; Online/desktop/preservation; engineering/submission records; final
verification and approval. Native/Debian, disposable legacy actual transfer and
broader lifecycle obligations remain open, integrated with corresponding package
checks rather than duplicated umbrella lifecycle items. Existing source/fake
evidence and phase-4/6 installed smoke retain explicit reuse boundaries.

Updated stale development baseline from de8d779/cf5da32 to installed packaging
82c1d7a/source 67b3aec, retaining earlier identities as history. Version/metadata
freeze precedes final builds; notes/checksum/acceptance reconciliation follows
without needless rebuild. Publication and installation approval boundaries remain.
Phase-6 transfer link now targets the unified acceptance heading. No acceptance
item marked passed by this reordering. No source/script/package changes or tests/
rebuild required; user's untracked Push Buttons.md untouched. Checks follow.

Reordering validation: git diff --check exit 0; focused Python checklist check
exit 0 verifies steps 1-8 in order, obsolete anchor removed, retained native/
legacy/network/retention/rollback/grant/UI/host-command/approval obligations and
exactly one existing completed source-baseline item in the unified section.
Only Task List.md and this record changed. No failures; existing passing source
checks reused because inputs unchanged. Next actionable item is step 1: map
missing acceptance coverage and freeze required release inputs before builds.


### October 18, 2026 — Step-1 acceptance map drafted

Operator asked what step 1 involves and to work on the map. Added maintained
release-054-acceptance-audit.md under engineering, indexed and linked from active
step 1. It separates accepted development Flatpak observations, retained source/
fake/prototype evidence, missing live scenarios and final artifact acceptance;
no new live test/build/installation performed. Lists concrete scheduled VPN,
pre-existing/shared ownership, active-transfer sleep/VPN-loss, native and legacy
checks with destinations. Earlier provider evidence still needs a focused audit
for final advertised support; no blanket untested provider pass or full repeated
provider/scenario cross-product imposed.

Verified current Cargo.toml/Debian version 0.5.3, retained 82c1d7a/67b3aec
development receipts, and declared six Ubuntu/architecture Debian build targets.
A broad file search initially used a nonexistent data/metainfo glob; corrected
to resources/app.metainfo.xml, no state change. Map identifies metadata/version,
platform/architecture coverage, immutable source/manifest and freeze boundaries
without silently bumping version or choosing extra targets. Step 1 remains open
until scope is reconciled and inputs finalized; source defects found by later
tests invalidate affected frozen inputs rather than bypass acceptance.
Existing engineering-guide naming/status rules followed; no credentials or
organization names added. Only documentation changes; validation follows.

Step-1 map validation: git diff --check exit 0; focused Python local-link/table
check exit 0, 19 coverage rows and pending freeze status verified. No failed
validation; prior application checks reused because runtime inputs unchanged.
Records remain uncommitted for review. Next: reconcile advertised provider/
platform matrix, then prepare consistent 0.5.4 metadata and immutable source pins.


### October 18, 2026 — Redundant live matrix removed; Flatpak evidence reused

Operator explicitly directed eliminating redundant work and reusing completed
Flatpak tests. Updated step-1 map/status and task list accordingly: accepted
phase-4/6 setup/auth/sync/scheduling/Pause/recovery/guard/Repair/portal/restart/
idle-wake/manual-VPN/panel/removal observations are reused for unchanged inputs.
No default full provider/account/scenario or additional architecture matrix,
repeated OAuth or further laptop suspend. Initial map's unrun live procedures
are conditional choices for concrete unresolved release gaps, not blanket
mandatory execution. Source/fake and installed evidence labels remain distinct.

Retain focused native/Debian changed-OneDrive smoke and disposable legacy actual
monitor/transfer as missing acceptance, plus final package/permission/dependency/
identity checks. Review retained worker/retention, provider, Online/UI and network
evidence before requesting extra live work. Unrun live cases remain explicit
limitations, never passed by bookkeeping; real defects remain blockers. Marked
map/reuse-policy reconciliation complete, input metadata/freeze still pending.
Next concrete step: prepare/review consistent 0.5.4 metadata and final package
inputs, using existing development package as acceptance baseline. No build,
installation, service/cloud changes or new test run required for this plan edit.

Reuse-plan validation: git diff --check exit 0; focused Python documentation
check exit 0 (links resolve, mapping complete and metadata freeze outstanding).
No runtime inputs changed; prior source checks reused. Plan changes remain
uncommitted; Push Buttons.md untouched.


### October 18, 2026 — Prepare 0.5.4 metadata before release-input freeze

Operator requested next step after evidence reuse plan. Updated application
Cargo.toml and only the root package version in Cargo.lock to 0.5.4, prepended
Debian 0.5.4 changelog and AppStream 0.5.4 summary dated October 18. Historic
entries unchanged. No dependency pins/runtime/grant/engine behavior changes;
legacy support and dependencies retained, new OneDrive rclone mirrors and
Repair/portal/recovery guidance improvements summarized. Release notes remain
Draft; prepared metadata is not publication or final package acceptance.

Found release_publication_test.py fixture copied the live Debian changelog while
assuming synthetic Cargo/AppStream 0.5.3. Replaced that one fixture input with a
neutral synthetic changelog to keep tests independent of real version bumps;
no production script change. Focused unittest discovery: six passed, exit 0.
Version/dependency Python consistency inspection exit 0: Cargo/lock/Debian/latest
AppStream all 0.5.4, all dependency package entries exactly unchanged.
cargo metadata --locked --offline --no-deps --format-version 1 exit 0.
Metadata recipe exit 0: target/task-runs/dev-metadata-check-rhpyci3r/. Raw validators
still report known COSMIC category and AppStream provides/binaries diagnostics,
per nonfatal maintained recipe; no new 0.5.4 release-entry diagnostic. This is
not a clean strict freedesktop validator pass. Final development gate follows.

Updated map/task status: metadata prepared, not committed or frozen; Flatpak
manifest still pins accepted older 67b3aec until new source commit is available.
No package build/install/tag/push/publication/service change. Prior accepted
Flatpak behavior remains reusable; a versioned final artifact needs correct
source/pin/identity rather than relabeling the installed 0.5.3 candidate.


0.5.4 metadata final verification: first gate dev-gate-jm4w0709 exited 1 after
fmt/Clippy passed because sandbox blocked just temporary directory creation
under /run/user/1000/just (read-only filesystem). This was an infrastructure
block, not a passing test or source failure. Required access rerun
bash scripts/dev.sh gate exit 0, target/task-runs/dev-gate-dqkwy923/, base
a00730e6573b25704998cbb451005d81416e1eba dirty=true. Full formatting/lint/Rust/
maintained Python gate passed; synthetic promotion test output does not imply
real publication. Focused six publication tests already passed; no source fix
needed between gates. Metadata recipe known COSMIC diagnostics remain above.

Files for metadata preparation: Cargo.toml, Cargo.lock, debian/changelog,
resources/app.metainfo.xml, tests/release_publication_test.py and task/evidence/
acceptance-map records. Earlier uncommitted evidence/reuse-plan edits retained.
Final diff-check exit 0. No production Rust/Fluent string changes, new dependency,
package build/install, service change or publication. Changes remain uncommitted;
next metadata commit and exact app-source manifest pin before freezing inputs.
Installed candidate unchanged; completed Flatpak behavior tests are reused.


### October 18, 2026 — 0.5.4 metadata committed; candidate source pin prepared

Operator authorized commit. Source/evidence commit
ef5c4d8f165544c6909bc43c17ed84c4ff5aae77 contains verified 0.5.4 metadata,
publication fixture isolation and consolidated evidence-reuse plan. User's
Push Buttons.md excluded. Passing dev-gate-dqkwy923, focused six publication
tests, dependency/version consistency and known-diagnostic metadata receipt
apply unchanged. Updated Flatpak manifest and three packaging-guide pins from
67b3aec to ef5c4d8; generated Cargo dependencies, runtimes, grants/features
unchanged. Step-1 candidate input preparation complete; installed package remains
82c1d7a/67b3aec. No new build/install, tag, push or publication performed.
Separate packaging pin commit follows focused packaging contract check.

Pin validation: cargo test --locked --test flatpak_packaging_contract exit 0,
four passed. git diff --check exit 0; source gate reused since app inputs did not
change after ef5c4d8. Documentation link/pin checks exit 0. Separate packaging
commit contains manifest, packaging guide and step-1/audit/index/evidence status.
Candidate inputs ready for a clean pinned build; no package acceptance inferred.


### October 18, 2026 — Step-2 package build and inspection started

Operator authorized step 2. Reviewed maintained build/inspection entry points,
release procedures and script inventory. Existing installed bundle contains
0.5.3 metadata and cannot be reused as 0.5.4 artifact; runtime/behavior unchanged
so completed Flatpak behavior evidence remains reusable. Created clean packaging
worktree /tmp/cloud-mounter-flatpak-packaging-28b0abd at
28b0abd8587f0c2167415eff1f2f5a1e88ac6a73, app source
ef5c4d8f165544c6909bc43c17ed84c4ff5aae77. Started maintained build-flatpak.sh
once for x86_64/master; output target/artifacts/candidates/flatpak-28b0abd-x86_64-master,
receipt/log target/task-runs/flatpak-candidate-28b0abd/. No installation.

Debian inventory says hosted matrix artifacts, not redundant local Debian build.
Read-only git ls-remote matches tracked origin/master 4b5bb28; 44 local commits
through 28b0abd are not remotely available. Requested separate push authorization
for exact committed scope before hosted build; no push/tag/publication performed.
No duplicate standalone native build requested while hosted package path is
being selected. Read attempt for nonexistent internal inspect_artifacts.py was
corrected to public scripts/inspect-artifacts.py; no state change from that read.


October 18 hosted path authorized: operator explicitly approved pushing master
and running build-only Debian workflow. Fetched origin/master immediately before
push, no incoming commits (0 behind/44 ahead). Reviewed committed series and
diff summary; historical series-wide diff-check noted pre-existing whitespace
in documentation, left untouched rather than editing accepted/user text. Push
exit 0, remote master moved 4b5bb28 to 28b0abd. No tag or publication.
Started run_github_linux_build.sh once from clean pinned /tmp checkout, --ref
master, source identity verified 28b0abd; selected run 39166940625. Maintained
script owns CI polling/download/verification, no external CI polling loop.
Logs target/task-runs/debian-candidate-28b0abd/; downloads
target/artifacts/candidates/debian-28b0abd. Hosted and local Flatpak builds run
independently; no separate redundant standalone native build.

Read-only maintained Flatpak preflight exit 0, no blockers, receipt
target/task-runs/flatpak-preflight-28b0abd/preflight.json. Required Platform25.08,
SDK25.08, COSMIC BaseApp stable and Rust SDK extension available. User app override
empty; system app absent and system overrides empty. User global overrides add
only read-only GTK3/GTK4/KDE theme/color configuration paths; no blanket host
filesystem or bus grant. This observes effective grants of installed older
candidate; new export policy comparison and payload inspection still follow.


### October 18, 2026 — 0.5.4 Flatpak build/export and offline payload review passed

Single build-flatpak.sh exit 0, target/task-runs/flatpak-candidate-28b0abd/.
Packaging 28b0abd/app ef5c4d8; export/bundle commit
1b2d100791bcbb76fb0ff940a811a374c48e2858793a838c165338639e5a3530.
Bundle 13,451,400 bytes, SHA256
062834a7dcbfd42a3abc13504941699208dfc395be70b00d2567a6d603451ddd.
Producer checked repository/bundle ref and complete metadata against built
policy; direct comparison with accepted 82c1d7a receipt confirms unchanged
complete metadata, runtime commits, finish args and SDK-extension declarations.

Short read-only payload audit complements maintained metadata inspection, which
explicitly excludes payload: main ELF executable and optional auth helper
executable present; Fluent catalog, desktop file, AppStream and icon present.
Five source payloads byte-identical to clean pinned checkout; all six exported
OSTree payload bytes SHA256 match built files. Main binary 33,545,672 bytes;
helper 6,598 bytes. Latest AppStream version 0.5.4. readelf identifies five ELF
NEEDED libraries, all found in accepted Platform25.08 deployment (not SDK).
Review retained privately in payload-review.json beside build receipt.

Legacy helper's GI/GTK/WebKit imports are not claimed available in Platform: read
production launch branch and existing packaging guide, confirming Flatpak uses
portal/manual redirect handoff while native can use packaged helper. Worker and
readiness guard are modes of shared Rust binary, not missing separate payloads.
No engine/auth/browser/UI/service execution in this audit. All payload, library
and metadata comparison checks exit 0. No install or installed 0.5.4 acceptance.
Hosted Debian matrix still pending; step 2 not marked complete.


October 18 operator flagged WebKit/helper retention during package inspection.
New rclone setup does not use it; current native existing-legacy setup still
offers helper-based reauthentication, while Flatpak production launch explicitly
uses portal/manual browser handoff and bypasses helper. Existing post-0.5.4
checklist had deferred helper/native dependencies along with legacy removal.
Asked scope choice: move helper/WebKit retirement forward everywhere with
browser/manual legacy reauth retained, or remove only unused Flatpak payload.
No source/package input changed yet. Running Debian receipt remains meaningful
for original inputs; affected artifacts require replacement if cleanup changes
the selected package inputs. No helper process, auth dialog or cloud operation
launched by inspection.


### October 18, 2026 — WebKit helper retirement implemented for 0.5.4

Operator selected removal everywhere with browser/manual legacy reauthentication
retained, and reiterated rclone migration avoids WebKit/Ruby compilation. Removed
helper source and helper-localization tests, helper installation from native/
Debian/source/GUI-prototype Flatpak recipes, and GUI-only Debian recommendations.
Installer no longer probes/installs GI/GTK/WebKit or exposes helper-only options;
all engines/--onedrive retained. No host package uninstall. Native manual legacy
sign-in now uses same validated bounded OpenURI portal as Flatpak; helper discovery
and launch and direct native xdg-open fallback removed. Native interactive engine
auth and existing manual response-file handoff remain; no new legacy creation.

Updated browser UI/help/error wording and removed only unused helper keys.
Preserved profile/credential/sync/runtime state, legacy monitor/Preview/Repair,
new rclone OAuth and Online onedriver. Source/dependency/packaging guides,
specification external-tools paragraph, catalog/inventory, host-launch inventory
and release notes reconciled; future engine-removal checklist refers helper-only
work to this 0.5.4 step. Historical configure/prototype records remain history;
shipping manifests do not build WebKit/Ruby. Earlier 28b0abd artifacts retained
as pre-cleanup evidence, not the selected next shipping package.

Verification so far: just fmt exit 0; dev-check-zbigv0pe exit 0; five focused
onedrive_auth_ tests exit 0 (private handoff, stale result, invalid URL validation
and engine default routing preserved); four flatpak_packaging_contract tests
exit 0; five dependency installer fake tests and thirteen reporting tests exit 0.
Helper-only installer flags reject before privileged operations. Translation
checker passed; final maintained localization/metadata/gate follow after docs.
No live auth/browser/helper/service/cloud change. Source/pin commit, replacement
build/inspection and narrow native/browser handoff acceptance remain pending.


### October 18, 2026 — Ubuntu 22.04 CI failure diagnosed and corrected

Hosted run 39166940625 ended failed: both Ubuntu22.04 architectures reached
Build and test Debian package; other four Ubuntu24.04/26.04 jobs succeeded.
Private gh --log-failed produced empty output; direct job-log API downloads
retained ubuntu2204-{amd64,arm64}.log. Both fail publication fixture with
ModuleNotFoundError tomllib, not runner unavailability. Python standard-library
tomllib starts at 3.11 (official docs https://docs.python.org/3/library/tomllib.html);
Ubuntu jammy provides python3-tomli1.2.2-2 in universe (official package download
https://packages.ubuntu.com/jammy/all/python3-tomli/download).

publication_check.py now falls back to tomli on Python3.10. Declared python3-tomli
as Build-Depends and in hosted dependency installation; no app runtime GUI
dependency restored. Added functional simulated-no-tomllib parser test using
real TOML parsing, including invalid document rejection. Build guide/catalog/
inventory updated. Actual 22.04 acceptance awaits new-source hosted matrix.
Do not rerun failed historical source or publish incomplete matrix artifacts.

Helper cleanup first final gate dev-gate-jjswt4q6 failed two Debian contract tests
that still required retired helper/GUI recommendations. Updated those contracts
to reject helper payload/dependencies while requiring retained engines/Python
build tooling; six focused Debian packaging tests then passed. No production
helper restored to satisfy stale tests. Final gate rerun follows combined fixes.


Combined cleanup/CI-fix verification: publication fixture now seven tests passed
(includes Python3.10 fallback simulation); Debian source packaging six passed.
Rechecked all-target compilation after Debian Rust contract changes: maintained
dev-check-zs798q61 exit 0. Localization receipt dev-localization-check-7i9tf6xc
exit 0; metadata receipt dev-metadata-check-fv8yjwyg exit 0 with known COSMIC
validator diagnostics only. Direct manifest/dependency/file/link audit exit 0:
no active manifest helper/WebKit/Ruby module, no helper source or GUI-only
recommendations; edited catalog/docs local links resolve. Maintained script
catalog and inventory updated for both helper retirement and TOML fallback.
Final combined gate running; no replacement dispatch yet.


Final combined source gate: bash scripts/dev.sh gate exit 0, retained receipt
target/task-runs/dev-gate-5jl1l4eq/, base 28b0abd dirty=true. Formatting, all-feature
Clippy, all-target Rust and maintained Python passed. Previous stale Debian
contract failure remains recorded; no broad failing run concealed. Compilation,
auth handoff, packaging, installer/reporting, publication fallback and localization
focused evidence above applies. Metadata recipe known COSMIC diagnostics remain.
Final git diff --check exit 0. Source/browser behavior and packaging cleanup are
implemented/tested but not committed, installed or accepted on 22.04 CI yet.
Requested explicit commit/push and replacement-build authorization for new fixes;
original approved push covered 44 commits only through 28b0abd. Old failed matrix
not redispatched, no tag/publication or source/helper/library deletion on host.
Remaining artifacts: successful pre-cleanup Flatpak retained, incomplete historical
Debian matrix retained remotely; next new-input builds after authorization.


### October 18, 2026 — Replacement source committed and pinned

Operator explicitly approved commit, push and replacement Flatpak/Debian builds.
Source commit 7ccc9f7e5a635d59dd0db9a3bb6ed3b1c85f4aef contains helper retirement,
Python3.10 CI compatibility fix, focused tests and chronology. Push Buttons.md
excluded. Manifest/guide now pin this exact source; no Cargo dependency/runtime/
permission changes. Build-only python3-tomli added for 22.04, no GUI dependency.
Separate packaging commit follows focused contract validation; then fetch,
review divergence, push and dispatch one matrix at new pinned packaging source.
Old successful Flatpak and incomplete CI records retained without relabel/retry.


### October 18, 2026 — Replacement artifacts started at approved post-helper source

Source 7ccc9f7e5a635d59dd0db9a3bb6ed3b1c85f4aef, packaging
196f7b11cecc7c382943af23b4ebc68579b00b17. Four focused Flatpak contracts passed
after pin update. Fetched immediately before approved push: zero incoming/two
outgoing, diff-check clean. Push exit 0; git ls-remote confirms master exactly
196f7b1. Clean packaging checkout /tmp/cloud-mounter-flatpak-packaging-196f7b1.

Started build-flatpak.sh once, x86_64/master, output
target/artifacts/candidates/flatpak-196f7b1-x86_64-master and receipt/log
target/task-runs/flatpak-candidate-196f7b1/. Started maintained hosted wrapper once
from clean checkout, selected run 39176426152 at exact 196f7b1, logs
target/task-runs/debian-candidate-196f7b1/, download destination
target/artifacts/candidates/debian-196f7b1. Tool owns polling and verification.
No standalone native duplicate, no installation/tag/publication. Shipping
manifest builds only Rust app, no WebKit/Ruby build module.


### October 18, 2026 — Post-helper Flatpak replacement passed offline inspection

Maintained single replacement build/export exit 0, source packaging 196f7b1/app
7ccc9f7. Receipt target/task-runs/flatpak-candidate-196f7b1/artifacts.json.
Export/bundle commit 80afd14af567164388f30e8b1632f18dd56c294190890da36b983a4d5ffe0bc5;
bundle 13,555,104 bytes, SHA256
bf86f6305e151f99d43f7abb2b59aa27f9e38186d6049e8dcbfea4d8d9615bbe.
Complete metadata, runtime commits, finish args and SDK extension declarations
match accepted 82c1d7a baseline. Producer ref/commit/bundle comparison passed.

Read-only payload audit exit 0: five expected app files; executable ELF, four
source assets byte-identical to clean checkout; latest AppStream0.5.4. All five
exported OSTree payload hashes match build files; retired helper absent in both
build and export bin listing. Main ELF library requirements present in accepted
Platform25.08, no host/SDK assumption. Review in payload-review.json beside
producer receipt. No WebKit/Ruby module added; browser/manual legacy route kept.

Maintained preflight for new pinned manifest exit 0, clean source/no blockers,
target/task-runs/flatpak-preflight-196f7b1/. Installed older app identity/overrides
match preceding preflight exactly: no app-specific/system override; user-global
adds only read-only theme paths. This plus equal new metadata documents candidate
policy/effective override composition, not installed replacement acceptance.
No installation or runtime-owner change. Debian replacement still pending.


### October 18, 2026 — Step-2 replacement matrix and artifact inspection completed

Maintained hosted-build wrapper exit 0, run 39176426152 succeeded all six Ubuntu
22.04/24.04/26.04 amd64/arm64 jobs. Both22.04 actual preflight tests now pass with
tomli fallback; runner availability was not the earlier issue. Download/producer
verified exact 196f7b1 source, six unexpired expected artifacts, package names/
0.5.4 versions/architectures and SHA256 sums. Receipt
target/artifacts/candidates/debian-196f7b1/receipt.json; summary/log
target/task-runs/debian-candidate-196f7b1/. No duplicate inspector identity/hash
run needed: those checks already passed in producer.

Additional short offline payload/dependency audit exit 0, all six packages:
five required app files present, four assets byte-identical to clean pinned
source, binary executable ELF with correct62/183 machine architecture, AppStream
latest0.5.4, retired helper absent. Binary Depends/Recommends have no helper-only
GI/GTK/WebKit packages; rclone>=1.74.3, onedriver>=0.15.0 and legacyonedrive>=2.5.10
recommendations retained. Build-only python3-tomli does not become app runtime
recommendation. Native binary is supplied by Debian packages, no redundant native
source build. Package bytes, hashes, payload hashes and dependency fields retained
privately in target/task-runs/debian-candidate-196f7b1/payload-review.json.

Step2 build/offline artifact scope complete. Updated active task, acceptance map
and engineering index; source/packaging immutable identities remain196f7b1/7ccc9f7.
No install, live engine/auth/service mutation, tag or publication. Existing
source gate and accepted Flatpak behavior tests reused; only affected source/
package checks repeated after cleanup/CI fix. Remaining risk: no installed
replacement/native/legacy live acceptance claimed. Next step3 evidence reuse/
auth-sandbox review, then focused remaining native/legacy checks. Docs validation
and record-only commit follow; no rebuild for acceptance receipts.

Step-2 closure validation: git diff --check exit 0; focused documentation/receipt
check exit 0 (local links resolve, checklist complete, both producer summaries
succeeded at exact196f7b1). Only task/evidence/map/index records changed after
artifact production; no repeat source gate or build justified. Record-only
commit is outside immutable artifact source; future reuse selects196f7b1, not
new documentation HEAD. Push Buttons.md remains untracked and untouched.


### October 18, 2026 — Step-3 authentication and sandbox access reconciled

Reviewed retained phase-4/6 observations and source comparison against installed
67b3aec, with selected candidate source7ccc9f7/packaging196f7b1. Personal and
work/school rclone OneDrive authentication/drive/folder checks passed previously;
SharePoint reused existing authentication and verified library/subtree access.
Personal missing recovery grant blocked Test until separate recovery Browse.
Personal and SharePoint relaunch sync without Browse, personal scheduled edit
transfer, persisted initialization/listings and recovery-copy creation establish
retained mirror/recovery/state access at their recorded installed scope. Exact
prior backup-byte equality remains unverified; no new SharePoint consent claimed.

Read-only git diff --stat and detailed src/app.rs comparison exit 0: app.rs is the
only changed src file between67b3aec and7ccc9f7. Its changes retire legacy helper
launch and unify browser/manual labels/OpenURI; Flatpak portal request retained.
Mirror access, Microsoft OAuth, OneDrive identity and coordinator sources have no
diff. Read step-2 payload-review.json: required Platform libraries present and
exported payload matches. Reuse prior metadata/runtime/policy/override comparison
and preflight receipts; no repeat build, gate, OAuth, portal grant or service
operation. Native browser/manual legacy handoff is changed live behavior and
remains under step4 continuity, alongside focused native/legacy checks.

Updated task, acceptance map and engineering index. Step3 evidence-review scope
complete; packaging196f7b1 has not been installed. No final replacement, other
provider, native handoff or unrun lifecycle pass inferred. Retained source gate
is dev-gate-5jl1l4eq (exit0); documentation-only edits do not invalidate it.
Existing user edits removing a future-section subheading and untracked
Push Buttons.md preserved. No commit/push/install/publication performed.

Step-3 documentation validation: git diff --check exit0; focused relative-link
and completed-checklist assertions exit0. Four record files changed; source,
artifact inputs and user-owned Push Buttons.md untouched. Next: step4 focused
native/Debian and disposable legacy continuity acceptance.

### October 18, 2026 — Native Pop!_OS VM dependency installer mapping failure

Operator reports 24.04 LTS/amd64 Proxmox VM; screenshot shows Debian candidate
0.5.4 configured successfully. APT local-file _apt permission notice is not an
installation failure; screenshot alone does not establish checksum/binary parity.
Dependency installer installed rclone1.75.2, then stopped before onedriver with
no OBS repository mapping for pop24.04; legacy installation was not reached.

Root cause: obs_repo_suffix used detect_distro (package-manager family), which
normalizes Ubuntu/Pop to debian, making its Ubuntu/Pop repository branches
unreachable. Changed OBS selection to read actual os-release ID; retained
openSUSE wildcard matching. Package-manager routing unchanged. Added synthetic
Pop/Ubuntu/Debian/openSUSE and unsupported distro/version regression cases;
python3 tests/dependency_installer_test.py exit0 (6 tests). No privileged package
installation or network repository availability verified by these fake tests.
Catalog/inventory and active step4 updated; existing source/package artifacts
remain immutable. Fix is standalone installer tooling, no app binary rebuild.

First gate dev-gate-mdze5o5_ exit1: fmt/lint passed, just could not create test
temporary directory under sandbox read-only /run/user path. Rerun requested with
normal authorized host access; outcome below. No commit/push or VM changes by agent.

Host-access gate dev-gate-zl7m77q1 exit0, source d066a00 dirty=true; full
fmt/lint/test passed. git diff --check exit0. Installer source fix complete;
VM retry/network package installation still pending. Operator should copy the
corrected local script to the VM (the immutable published source still has the
old bug), rerun --all with a fresh log directory, and report final status.


### October 19, 2026 — Pop!_OS VM native personal mirror round trip

After corrected installer transfer/retry, operator screenshot shows INFO Done
and native settings/Add Connection windows rendered in VM. No final structured
installer summary or full engine-version inventory captured; this is an operator
completion observation, not independent verification of every installed engine.

For matching 24.04/amd64 Debian candidate0.5.4 selected from run39176426152,
operator reports personal Test Connection passed, then Preview completed after
Save/local test-file instructions. Initial Sync Now completed and operator
confirmed test file arrived remotely. Following explicit remote-edit instructions,
operator confirmed subsequent sync completed and updated text arrived locally.
This establishes operator-observed personal native upload/download smoke on the
disposable test path. No independent VM hash/installed binary parity, checksum
notice, recovery-copy equality or scheduling claim inferred. Start/Pause,
work/school, onedriver and disposable legacy continuity remain open.

Task/evidence records updated without source changes; prior source gate retained.
Next native Start/Pause UI check. No laptop services, files or credentials touched.

October 19 native VM personal Start/Pause: operator observed slider on after
Start, then explicitly switched it off and reports Pause completed/sync stopped.
Accepted as rendered UI start/pause smoke. No timer-property trace, scheduled
transfer or post-Pause interval observation inferred. Personal test left paused.
Next focused work/school OneDrive smoke using a separate disposable subtree;
existing Flatpak lifecycle evidence reused. Documentation diff check exit0.

October 19 native VM business setup: Test passed. Operator Save screenshot for
Acceptance-Business reports managed mirror units saved/schedule disabled and
directs Preview/initial Sync. Preview twice refused: connection disabled or
unavailable. Source inspection: controller offline_status treats enabled=false
without observed active work as Unavailable; Test validates draft separately.
DraftEnabled changes only the draft flag, and Save installs paused units.
Operator explicitly clicked Enable and reports Preview passed afterward. Thus
disabled profile blocked Preview; reason it became disabled was not observed.
Do not infer account/auth failure or auto-enable on successful verification.

Record misleading Save notice as outstanding wording fix: disabled profiles
need explicit Enable/Test/Save before Preview; schedule remains paused. Business
transfer not yet accepted. Personal Start/Pause result unchanged. No runtime
source changes, commands against VM or repeat source checks. git diff --check
exit0 for record edits.

October 19 native VM business transfer: operator clarified Enable needed a
subsequent Save to take effect, matching draft-only DraftEnabled handling.
Initial Sync Now reported completed. After instructions to confirm upload and
edit the disposable remote file, operator reports completed and arrived; accepted
as operator-observed business transfer/remote-edit download smoke. No independent
VM hash, transfer counts or explicit checksum notice captured. New business
profile has not been started for scheduled operation by these instructions.
Next native onedriver Online setup/mount/list/unmount; legacy compatibility and
wording fix remain open. Documentation-only diff check exit0, prior gate reused.

October 19 native VM onedriver Online: operator reports Test and Save passed,
then enabled the Online connection, saw files in mount directory, closed the
file manager and reports unmount completed. Accepted as rendered/operator
observed native onedriver setup/mount/list/unmount continuity. No file mutation
or independent mount-table/property trace inferred. Focused native/Debian smoke
now complete at operator-observation scope with personal/business round trips
and personal Start/Pause. Shared accepted Flatpak lifecycle evidence reused;
no duplicate sleep/VPN/Repair suite. Final VM package checksum/binary parity
not captured; final artifact acceptance remains separate. Legacy monitor/actual
transfer and disabled-profile Save wording remain open in step4. Updated task
and acceptance map; prior passing source gate reused for documentation edits.


### October 19, 2026 — Operator waived legacy live acceptance

Operator questioned further abraunegg/onedrive testing and declined its need.
Removed browser/manual legacy reauthentication and monitor/actual-transfer checks
from 0.5.4 mandatory acceptance, marking [o] deliberately skipped, never [x].
This supersedes the earlier transferred step4 gates. Native rclone/onedriver
smoke retained as completed; legacy rendered deprecation/Preview and source/fake
evidence retain original scope. Changed native browser handoff remains unrun
live and an explicit limitation. No test profile, auth flow, service start or
cloud transfer was performed for this proposed legacy test.

Clarified installer facts: --all includes legacy onedrive and operator screenshot
reports v2.5.11-1+np1+1.1 installed. Waiving this test changes no supported engine,
installer selection, dependencies or existing profile/credential state. Updated
active task, acceptance map and index. Remaining step4 work: disabled-profile
Save wording and retained worker/progress/cancellation/retention evidence review.
Documentation-only validation git diff --check exit0; prior source gate reused.


### October 19, 2026 — Step-4 notice correction and evidence review

Implemented state-sensitive offline_mirror_saved_notice early return for disabled
profiles in src/app.rs, shared by rclone providers and retained legacy engine.
New Fluent notice explicitly instructs Enable, Test Connection, Save Connection
before Preview/Sync Now and says enabling/saving does not start background sync.
Enabled initial/initialized notices and all runtime/engine policy are unchanged;
no auto-enabling after verification. Two focused notice tests passed, including
four provider/engine cases with both initialization states and unchanged enabled
flag. bash scripts/dev.sh check exit0 dev-check-pmjdf7i3; cargo test --locked
--bin cosmic-ext-applet-mounter offline_mirror_save_notice exit0 (2 tests).
Localization-check exit0 dev-localization-check-uor7nshy. Final gate running.

Reviewed retained Task(5) carry-forward chronology and coordinator inventory,
current worker/coordinator/progress implementation, retained gate
 dev-gate-zl7m77q1 and phase4/6 installed results. Diff7ccc9f7 against worker,
coordinator, mirror script/access and pending-count modules is empty. Existing
fixed-mode/UUID worker and shared state-root evidence, installed scheduled
personal transfer/Pause, invocation-correlated progress tests, failure/timeout/
cancellation/overlap and owned success-only retention protections remain valid.
No changed path or unresolved step4 blocker requires another live transfer.

Limits preserved: no final-package rendered streaming-count pass or manual
streaming feature completion; no new live age-expiry purge or exact prior-byte
backup equality; VM switch does not prove native scheduled transfer; active
sleep/VPN-loss is step5 evidence review; legacy live waived. Updated acceptance
map and draft release notes, plus explicit final release task for refreshed
source/pin/packages: currently built196f7b1/7ccc9f7 predates notice correction.
No repeat setup/auth/full lifecycle suite required for notice-only app change.
No commit, build dispatch, install or publication performed in this task.


Step-4 closure: full bash scripts/dev.sh gate exit0,
 target/task-runs/dev-gate-nh0kz3wx/summary.json and task.log; source d066a00,
dirty=true. No validation failures for this notice change. Check/focused notice/
localization receipts above all exit0. Changed implementation files src/app.rs
and i18n/en/cosmic_ext_applet_mounter.ftl; task/completion records, acceptance map,
engineering index and draft v0.5.4 notes reconciled. Existing installer fix and
all prior uncommitted/user edits preserved; Push Buttons.md untouched.
Documentation relative-link assertions and final git diff --check exit0.

All step4 items now completed at recorded scope or deliberately skipped by
operator. New notice is not installed: immutable source/pin refresh, affected
package build and changed-notice UI acceptance remain explicit step8 work.
No code/runtime gap justifies repeating unchanged auth/sync/fault suites. Other
unrun cases retain source-only or limitation status in acceptance map. Next
logical task is step5 VPN/sleep evidence review, reusing accepted manual VPN and
idle wake results; this closure does not authorize another suspend, commit,
push, build dispatch, installation, publication or tag.


### October 19, 2026 — Approved step-4 commit and replacement-package preparation

Operator requested commit and package creation. Rebuild justified by new disabled
Save notice absent from196f7b1/7ccc9f7; standalone installer distro fix and scoped
acceptance records included. Existing runtime/permissions and accepted lifecycle
evidence unchanged. Reuse gate dev-gate-nh0kz3wx and localization
dev-localization-check-uor7nshy; no repeated source gate for unchanged inputs.
Prepare one immutable app-source commit, then exact Flatpak pin/packaging commit
and clean checkout. Local Flatpak build permitted without pushing. Hosted Debian
requires pushing these new commits; separate push approval is not inferred from
commit/package request. Installation, tags and publication remain unauthorized.
Preserve untracked Push Buttons.md and user-owned future-section heading edit.

Step-4 source committed as72661ce9d705038cdb1878165917f298e0ced8c2.
Prepared Flatpak manifest exact pin to that source and updated packaging guide.
No runtime, finish-args, SDK/Platform, Cargo lock or vendor source changes.
Pin-only packaging validation follows; existing final gate reused.
Pin-only cargo test --locked --test flatpak_packaging_contract exit0 (4 tests);
git diff --check exit0. Historical optional-module contract test is not evidence
that WebKit is shipped; selected source manifest continues to exclude it.

Replacement packaging commit7069ae5b2182379ebd4f32ba968e4692dbe6c155 pins app
72661ce9d705038cdb1878165917f298e0ced8c2. Operator explicitly approved push and
Debian candidates. Fetch immediately before push found0incoming/2outgoing;
normal origin/master push and ls-remote exact7069ae5 verification exit0.

Local Flatpak wrapper started using clean detached worktree
/tmp/cloud-mounter-flatpak-packaging-7069ae5, exact packaging SHA, x86_64/master
and --local-source-repo. Logs target/task-runs/flatpak-candidate-7069ae5/;
output target/artifacts/candidates/flatpak-7069ae5-x86_64-master/.

Initial hosted invocation with --ref full SHA received definite GitHub HTTP422
No ref found before any workflow run (summary run_id null); no build occurred.
Preserved failure target/task-runs/debian-candidate-7069ae5/. Retried maintained
wrapper with verified master (branch dispatch required), new log directory
 target/task-runs/debian-candidate-7069ae5-master/. Correlated run39282971942
selected at exact7069ae5; downloads target/artifacts/candidates/debian-7069ae5/.
This is one actual Debian matrix, not a duplicate source build. Wrapper owns
polling/download/identity verification. Both package outcomes pending.

Flatpak7069ae5 build/export/inspection exit0. Export commit
b2a8a6e68f9f64687304a482849271ed5aab53d8039b97abb4099e62f58e6408;
bundle13,523,784bytes SHA256
a11c6d2a0451b00e27f56fdc78591223779c93acec9d280d5867107204a0c5ac.
Short offline payload review exit0: all5 required files match exported bytes;
4 source assets byte-identical to clean7069ae5 checkout, new disabled Save notice
present, helper absent. Complete exported metadata, declared finish-args/SDK
extensions, runtime commits and ELF NEEDED list match prior196f7b1 candidate;
reuse prior actual Platform library-capability evidence. No current installed
override/profile mutation or replacement-installation claim. Review saved in
 target/task-runs/flatpak-candidate-7069ae5/payload-review.json. Debian pending.


October 19 hosted delay diagnosis: one gh run view (exit0) shows all six jobs
queued with no steps executed. No CI build failure or runner-version-specific
failure observed. Official GitHub status summary checked: active incident
8tzbpcnl75jp, Actions degraded availability/workflow-processing delays, opened
15:49 UTC before this16:07 UTC dispatch. Source:
https://www.githubstatus.com/incidents/8tzbpcnl75jp . This plausibly explains the
queue; no claim of a repository-specific root cause. Wrapper continues bounded
waiting (7200-second default), with no remote cancellation or duplicate build.

Checkpoint: requested source commits pushed; Flatpak replacement produced and
verified, Debian matrix pending external service recovery. New Debian artifacts
not claimed. Existing local wrapper session94504 may finish/download later;
inspect its summary/receipt before resuming. If it has ended, use the maintained
wrapper --resume-report target/task-runs/debian-candidate-7069ae5-master/summary.json
with the same download destination and a fresh log directory; this selects the
recorded run/source and never redispatches. Do not run a competing downloader
while the original wrapper is active. Refreshed package task split to distinguish
completed Flatpak/source work, queued Debian work and installed-notice acceptance.
User future-heading edit and Push Buttons.md excluded from commits; preserved.
No installation, tag or publication. Record-only changes do not require rebuild.


### October 19, 2026 — Replacement Debian candidates completed

Operator reports workflow complete. Existing wrapper session94504 finished
exit0 without restart/redispatch, phase artifacts-ready, run39282971942,
source7069ae5b2182379ebd4f32ba968e4692dbe6c155. Maintained producer downloaded and
verified six expected Ubuntu22.04/24.04/26.04 amd64/arm64 artifacts, package
identities and SHA256 checksums. Summary/log:
 target/task-runs/debian-candidate-7069ae5-master/; artifact receipt:
 target/artifacts/candidates/debian-7069ae5/receipt.json. No new network/run poll
or duplicate identity/hash inspector needed; retained producer result reused.

Short offline payload audit exit0: all six Fluent catalogs byte-identical to
clean7069ae5 source, new disabled Save notice present, retired helper absent.
No package execution, extraction into system paths, installation or publication.
Review saved to target/task-runs/debian-candidate-7069ae5-master/payload-review.json.
Original definite raw-SHA dispatch422 and later queue observations remain
historical; they are not CI compiler/package failures. Actual matrix succeeded.

Updated active task, acceptance map and index; all replacement candidate build/
offline inspection work complete. Appsource72661ce/packaging7069ae5 immutable;
record-only updates require no rebuild or repeat source gate. Installed changed
notice and final release checks remain open. Prior user heading edit and untracked
Push Buttons.md preserved. Documentation/diff validation follows.
Documentation relative-link validation and git diff --check exit0. Commit only
package-completion records; retain immutable artifact source and user edits.


### October 19, 2026 — Replacement Debian disabled-mirror notice accepted

Following instructions to copy7069ae5 Ubuntu24.04/amd64 candidate, reinstall the
same0.5.4 version and reload panel, operator confirmed applet added back to VM.
Operator followed Disable, Test Connection, Save Connection on disposable business
profile and supplied rendered Modify screenshot. New notice explicitly says
connection disabled/background sync stopped; choose Enable, Test Connection,
Save Connection before Preview/Sync Now; enabling/saving does not start background
sync. Full notice visible; Enable button present and Preview/Sync unavailable.
Accepted as installed native rendered notice observation. No independent VM
package/binary hash, systemd property trace, Flatpak replacement install or release
claim. Previous native personal/business/Online lifecycle results reused.

Updated task, acceptance map/index and draft release-note acceptance boundary.
No runtime source changes; prior gate/localization retained without repeat. Next
operator restoration: Enable, Test Connection, Save Connection, leave panel switch
off. This restoration is not yet claimed performed. git diff --check exit0.

October 19 restoration confirmed: operator reports enabled, tested and saved
after installed disabled-notice check. Accepted as completion of explicit draft
Enable/Test/Save restoration. No additional sync, schedule activation or backend
property observation claimed. Instructions leave panel switch off; Save's normal
paused-schedule policy unchanged. Installed notice acceptance complete, prior
lifecycle evidence retained. Documentation-only git diff --check exit0.


### October 19, 2026 — Step-5 numbering clarification and evidence review

Operator starts consolidated step5 and questions Task(6). Traced it to the older
October7 D-Bus host-integration migration child6/NetworkManager OpenConnect
migration, retained in Task List Carry Forward and archived v0.5.3 task lines
3995–4088. It is historical numbering, not current consolidated step6. Clarified
current step5 with a named carry-forward link; similarly named bisync and
OpenConnect work in current step4/7. Archived numbering/chronology unchanged.

Reviewed prior native/prototype Online positive mount/unmount/ownership and
canceled/rejected authentication; do not quote private profile/share/route data.
Reused phase6 installed source-built Flatpak mirror cancellation/auth/checksum
sync/conditional release/restoration and idle wake/checkpoint evidence. Existing
candidate metadata/runtime/permission audit valid. Source diff67b3aec..7069ae5 is
empty for vpn, sleep, services, mirror_access and bisync_coordinator. App diff
contains only browser helper retirement and disabled Save notice/tests. Retained
gate dev-gate-nh0kz3wx includes passing sleep invalidation, activation/shutdown,
shared/finite consumers and deadline guards; no unchanged check rerun.

Review complete: no additional laptop suspend or full VPN fault matrix needed.
Scheduled unavailable, final Flatpak shared/pre-existing tunnel protection and
active-transfer sleep/VPN loss remain unrun live limitations. One source-built
Flatpak Online positive mount/list/unmount/conditional release and canceled-auth
case remains explicit from original final-package gate. Candidate7069ae5 already
built/verified; recommend existing laptop VPN/share over duplicating VM setup.
Read installer --help only; no installation, mount, authentication, profile/
service mutation or client removal. Keep existing Cisco fallback pending final
acceptance; optional OS removal not authorized. Updated task/map; next operator
approval/quiescence for candidate installation before focused test.
Step-5 documentation validation: git diff --check exit0; changed-document local
links and historical/current-step boundary assertions exit0. No runtime edits,
so prior passing gate reused. Installation request pending operator readiness;
no package or service change performed. Existing user edits preserved.


### October 19, 2026 — Approved laptop Flatpak replacement for step-5 VPN test

Operator explicitly confirmed Ready/install on laptop, with editor/settings
closed, mirrors paused and panel applet removed. Maintained install wrapper first
failed before installation: recorded /tmp/cloud-mounter-flatpak-packaging-7069ae5
checkout absent (missing file); log target/task-runs/flatpak-install-7069ae5/.
Worktree metadata showed path prunable/nonexistent. Recreated only that exact
missing checkout at7069ae5 using git worktree add --force --detach; no other old
worktree cleanup or user file removal. No candidate rebuild.

Retry maintained install wrapper exit0 after reuse verification and user bundle
installation. Logs/receipt target/task-runs/flatpak-install-7069ae5-retry/.
Installed commit b2a8a6e68f9f64687304a482849271ed5aab53d8039b97abb4099e62f58e6408
matches expected7069ae5 candidate; metadata/identity checks passed, previous
commit and overrides retained in installation.json. Receipt acceptance not-run:
no applet launch, sync, VPN activation, Online mount or sleep performed by agent.
Next operator re-add applet and locate existing VPN-bound Online share; keep
connections off until scoped canceled-authentication instructions. This installed
package result does not close VPN functional acceptance or release approval.
Documentation-only diff check exit0; source gate not repeated.


### October 19, 2026 — Step-5 focused installed Flatpak VPN accepted

Installed packaging7069ae5/app72661ce, OSTree b2a8a6e... (full installation identity
above). Operator reports initially no VPN connected. Following instructions to
start VPN-bound SMB and cancel authentication, reports VPN connection failed and
row off. Retry instructions complete auth and list mounted share; operator
reports files visible. Following close-file-manager/turn-off instructions,
operator reports unmount completed. Private profile/share names omitted.

Operator unsure of final VPN state; agent used typed Gio NetworkManager D-Bus
proxies (system bus) to inspect ActiveConnections and each Vpn property. Command
exit0: active_vpn_connections0, vpn_states empty, read_only true. No agent VPN
activation/deactivation, service or profile change. Observed result transcribed
privately in target/task-runs/phase5-flatpak-vpn-7069ae5/final-networkmanager.json.

Accepted operator-guided installed negative/positive Online cycle and final
VPN-off outcome. No independent negative-case mount/unit trace, release-option
configuration dump, shared/pre-existing-tunnel live proof or mid-transfer fault
inferred. Earlier source ownership/fake and phase6 mirror release evidence reused;
no repeat gate/source check or suspend. Cisco disposition keeps any installed
client as optional manual fallback; no removal release gate, uninstall, profile
or service deletion. Updated docs/openconnect.md, release notes, task/map/index.

Step5 scoped acceptance complete. Remaining final package/desktop/submission
work stays open. Changes are documentation only; prior gate nh0kz3wx retained.
Next consolidated step6 Online/desktop/preservation evidence review. Validation
follows; no commit/push/build/publication performed in this acceptance update.
Step-5 closure validation: changed-document relative links and checklist assertions
exit0; git diff --check exit0. No runtime source inputs changed; no repeated
passing gate or candidate build. Prior user edits remain preserved/uncommitted.


### October 19, 2026 — Step-6 focused evidence reuse and closure

Operator requested focused tests without repeating completed work. Reviewed
active step6, acceptance map, October12 translation audit, current package/UI
observations and exact archived evidence. Archive v0.5.3 completion notes contain
Google Drive/Box prototype host-visible mounts and native SFTP lifecycle, plus
installed deliberate-busy unmount refusal and separately confirmed Repair with
inactive/disabled unit and detached FUSE mount. Private resource names omitted.
Latest source Flatpak SMB/VPN and VM native onedriver normal lifecycles accepted
in steps4/5. Shared routing remains unchanged; no fresh provider auth matrix.

Clean-profile evidence: fresh Pop!_OS VM Add/Test/Save and personal/business round
trips. Upgrade continuity: existing VM profiles/notice after same-version Debian
replacement and existing SMB profile after Flatpak7069ae5 replacement. Removal:
October18 paused disposable profile removed, owned units gone, mirror/recovery
files retained. No whole-state hash equality or repeat removal/reset claimed.
Desktop: native settings/editor screenshots and notice, current Flatpak panel/
popup operation and earlier recovery/legacy wording. English-only current guard
uor7nshy passed1catalog/1363messages/1576references; package catalogs equal source.
Older translation helper/count claims preserved as historical, with current
retirement/acceptance follow-up appended.

Diff67b3aec..7069ae5 empty for controller, config, runtime_ipc, providers, services,
mount_guard, mounts, preload, model and theme; app diff concerns browser helper
and disabled Save notice. Existing gate nh0kz3wx confirms failed-unmount latch,
expected-provider FUSE ownership, timed Repair confirmation and managed-engine
routing. live_sftp_online_unmount_repair_and_sleep_cleanup remains explicitly
ignored and is not accepted as a fresh run. No new source or validation inputs.

Step6 closes at scoped review/reuse level. Latest Flatpak busy Repair not newly
run; no exhaustive accessibility/locales/provider auth or bytewise upgrade-state
claim. Historical carry-forward sleep/mountpoint/pending-count/OAuth/startup items
remain open at their own tasks. No additional sleep, forced busy mount, transfer,
account setup, cleanup, build, service or config change performed. Future GUI
Push Buttons.md untouched. Updated concise task checkboxes, acceptance map/index
and translation follow-up. Next step7 engineering/submission reconciliation;
final release verification remains open. Validation follows, no repeat gate.
Step-6 closure validation: git diff --check exit0; changed-document relative-link
and four-completed-checklist assertions exit0. Prior gate nh0kz3wx/localization
uor7nshy reused unchanged. No runtime source diff or repeat test/build needed.


### October 19, 2026 — Step-7 engineering reconciliation and COSMIC readiness

Operator requested step7. Read existing inventory, packaging rationale, engineering
index and candidate/installation receipts; reviewed default process enum and
cfg-gated CLI hits. Shipping nine identities unchanged. Nmcli/Systemctl in app
and sleep snippets are tests/development only; no source route added or removed.
Static source/receipt reconciliation exit0:23declared grants, no filesystem=host,
installed commit/metadata exactly match candidate, app overrides empty, only four
read-only theme global paths. Saved summary under
 target/task-runs/step7-reconciliation/summary.json. Not a new formal reachability
proof, live test, current remote policy approval or override mutation.

Updated inventory scope from historical0.5.3 wording to candidate0.5.4; retained
dated observations, added explicit current owners/removal/permission boundaries
and scoped sandbox-gate closure. Packaging guide current follow-up explains reused
portal/worker/VPN evidence, actual host authority, external dependencies and
candidate acceptance limits. No new grant or runtime/dependency change. Closed
only transferred worker/VPN sandbox observations at steps3–6 levels; final release
checks and unrun variants remain separate.

Checked public upstream README, justfile and PR template (official pop-os/
cosmic-flatpak repository). README supports applet repository route; build recipe
uses app/<id>/<id>.json. PR template requires personal contribution acknowledgment
and excludes AI-generated PR descriptions/other content outside own app source/
manifest exception. No user attestation, maintainer permission exception or
submission can be inferred. Prepared local cosmic-submission-audit.md as internal
facts/questions only, not text to paste into that external PR. Operator must
author the submission and assess actual included files; maintainer decisions on
host gateway/dependencies/runtime/portal exception remain open. No third-party
contact, PR, checkbox acknowledgment or publication performed. Web raw justfile
and guessed example path fetches unavailable; readable official justfile page and
template/README supplied sufficient direct evidence, no guessed example relied on.

Task7 split into internal completed items and explicit external pending item.
Engineering index and acceptance map updated. No code/source input changed, so
prior gate nh0kz3wx/localization and pin-packaging checks reused without rerun.
Existing uncommitted acceptance records, user future-heading change and
Push Buttons.md preserved. Documentation validation follows; no commit/push or
build/install/service state change performed. Next internal step8 release checks
are separate from the pending COSMIC author/maintainer review.

Step-7 validation: static inventory/receipt reconciliation exit0; changed-document
relative-link and internal/external checklist assertions exit0; git diff --check
exit0. No repeated gate/build or external submission. New readiness audit indexed.


### October 19, 2026 — 0.5.4 GitHub Flatpak distribution scope

Operator clarified that 0.5.4 completes Flatpak release preparation and independent
GitHub bundle distribution, with no COSMIC repository PR. COSMIC submission moves
to 0.5.5 after legacy-engine removal and GUI cleanup. Preserved prior preparation,
open maintainer questions and existing user-authored future sections; those
external decisions are no longer 0.5.4 release blockers. No legacy runtime removed.

Read release procedures, current promotion script/workflow and current acceptance
records. scripts/promote_release.py and hosted publication currently validate and
attach only the six Debian artifacts and SHA256SUMS. Mixed Debian/Flatpak
promotion remains an explicit implementation task; no ad hoc asset upload or
publication performed. Preserve current candidates and acceptance, resolve final
source/tag identity under maintained guards, and rebuild only if actually needed.

README now drafts prospective 0.5.4 GitHub bundle download, checksum verification,
user runtime setup, local installation/replacement, host-engine requirements,
separate directory Browse grants and manual updates. Corrected dependency-script
branch main to master to match this repository. Official Flatpak single-file
bundle documentation confirms direct bundle installation and separate runtime
dependencies; local flatpak install --help confirms bundle/arch/or-update flags.
No clean-profile install or network download of unpublished assets attempted.

Updated task list, acceptance map, submission audit/index, packaging guide and
draft release notes. Remaining: mixed-package promotion with focused tests and
script records, focused clean-profile instructions/panel smoke, final release
checks/notes/checksums/identity review and explicit tag/push/publication approval.
Completed unchanged provider/VPN/mirror tests are not scheduled for repetition.
Documentation-only validation recorded below; no source tests/builds repeated,
no commit, push, install, service change or external submission performed.

Scope-update validation: Python local-link/runtime/candidate-filename and scope
assertions exit0; documentation diff whitespace check excluding Task List.md
exit0. Full git diff --check exits2 for three pre-existing trailing spaces in
the operator's 0.5.5 AI-consideration section; preserved those user edits.
No executable inputs changed and no full gate rerun.

### October 19, 2026 — Operator README wording review

Read the operator's revised README and retained its separate Debian/Flatpak
numbered instructions. Explained user-level Flathub/runtime setup, download
directory, checksum OK condition, architecture, external engines and manual
updates. Added the pending-publication boundary and replacement preparation;
retained separate Browse grants. Corrected shell fence and obvious prose typos,
condensed the stale OneDrive development notice into release-note guidance.
No source, scripts, manifest or release publication changed.

Changed README.md and this task/completion record only for this follow-up.
git diff --check -- README.md exit0; Python local-link, bash -n code-block syntax
and installation wording assertions exit0. Commands in the README were parsed,
not executed; clean-profile installation remains open. No build/test gate repeated
for documentation-only wording. No commit or publication. Next release task:
maintained Flatpak promotion support, as listed in consolidated step8.

### October 19, 2026 — Automatic Debian release installer

Operator requested a standalone installer and one-command README instructions.
Added scripts/install-debian.sh: defaults to 0.5.4, accepts --version X.Y.Z,
selects Ubuntu/Pop!_OS VERSION_ID 22.04/24.04/26.04 and native dpkg architecture
amd64/arm64. Unsupported targets fail before downloading. Fixed-repository HTTPS
downloads use bounded connection/transfer timeouts. Verifies a unique selected
SHA256SUMS entry, actual digest and dpkg-deb Package/Version/Architecture before
APT install --yes --reinstall (sudo when needed). Temporary downloads are removed
on exit, actual installation status propagated and summarized. No OS repositories,
cloud credentials, services or app configuration changed by the installer itself;
APT package installation has normal package-maintainer-script effects. Optional
storage engines remain separate. No real installation was performed in this task.

README Debian section now contains only its curl-to-bash command. Updated script
catalog and engineering script inventory. Added tests/debian_installer_test.py:
12 supported distro/version/architecture cases, unsupported targets, download,
checksum/duplicate and all three package-identity failures, APT exit propagation,
temporary cleanup and stdin help/invalid arguments. All commands use synthetic
OS metadata and fake network/package adapters; no actual downloads or APT calls.

Validation: bash -n scripts/install-debian.sh exit0; focused Python suite exit0
(five tests, multiple subcases); read-only script --help exit0; targeted document
diff whitespace check exit0. First gate dev-gate-huz32vmh failed because sandbox
could not write /run/user/1000/just. Retrying with authorized access under
dev-gate-h6f8jd93; final outcome recorded below. Worktree is dirty at source
1aa9122a2140eab299cb5ba6429e08d046dd7c49. No commit/push or publication;
the public curl URL needs the new script pushed, and package use needs published
0.5.4 assets/checksums. Installed acceptance remains separate.

Final gate dev-gate-h6f8jd93 succeeded, exit0, source_dirty=true; receipt/log:
target/task-runs/dev-gate-h6f8jd93/{summary.json,task.log}. This covers formatting,
Clippy and Rust/Python test suites, including the new fake installer tests.
No further executable inputs changed. Installer implementation complete;
publication and a future approved actual installation remain separate.

### October 19, 2026 — Standalone GitHub Flatpak installer

Operator requested architecture detection and a README curl command, assuming
release assets will be present; hosted workflow explicitly follows this task.
Added scripts/install-flatpak-release.sh, distinct from receipt-driven candidate
install-flatpak.sh. Default version 0.5.4, optional --version X.Y.Z. Maps native
x86_64/amd64 and aarch64/arm64 to release asset names; refuses unsupported CPUs.
Requires existing Flatpak and a desktop user, never root. Downloads selected
bundle/SHA256SUMS over bounded HTTPS and verifies the unique selected digest
before any Flatpak mutation. Adds user Flathub, installs/updates architecture-
matched Platform 25.08, then the user bundle with noninteractive/update flags.
No workflow/build, host-engine installation, service stop or automatic rollback.
Partial setup failure can retain the remote/runtime; actual exit status is
preserved and temporary downloads cleaned. No actual installation performed.

README Flatpak section now contains only curl-to-bash. Prerequisite Flatpak is
noted in Dependencies; detailed setup/update boundaries moved to packaging guide.
Updated script catalog/inventory; corrected two README installation anchors after
the operator's heading rename. Architecture-selection implementation does not
establish ARM assets, runtime availability, hosted builds or installed acceptance.

Validation: bash -n exit0; five focused Python fake-adapter tests exit0 (CPU
aliases, unsupported CPU, missing/download/checksum/duplicate failures, ordered
remote/runtime/bundle failures, cleanup, explicit version and stdin arguments).
Read-only --help exit0; selected-document diff check and single-command/local-
link assertions exit0. First final gate dev-gate-9ht218t8 blocked on sandbox
/run/user/1000/just writes; authorized retry running, outcome recorded below.
No source/manifest change, commit/push, package install or publication. Public
command needs the script pushed and matching release assets/checksums published.
Next task is the separately requested GitHub Flatpak build workflow.

Final gate dev-gate-vj99vh5b succeeded, exit0, source_dirty=true at source
1aa9122a2140eab299cb5ba6429e08d046dd7c49. Receipt/log under
target/task-runs/dev-gate-vj99vh5b/{summary.json,task.log}; formatting, Clippy and
Rust/Python suites passed. No executable changes after that gate. Installer and
README task complete at source/fake-test level; actual installation/hosted build
and published assets remain separate. No commit/push or publication performed.

### October 19, 2026 — Manual native-architecture GitHub Flatpak workflow

Operator requested the hosted Flatpak workflow after both standalone installers.
Added manual-only build-flatpak.yml, native Ubuntu24.04 x86_64/aarch64 jobs,
read-only repository permissions, full-SHA pinned checkout/upload actions and
14-day artifacts. No push/tag/PR trigger or release-publication job. Runner
labels confirmed in GitHub's official runner reference. Flathub read-only ARM
BaseApp query initially needed explicit user installation (two remotes), then
sandbox DNS blocked it. Authorized retry succeeded: app/com.system76.Cosmic.
BaseApp/aarch64/stable, Platform/SDK25.08, commit
5a5f5541eb0fe2bc11cdf2f56044deff76414780274a8b7875d44ad78b50de74.
Official web BaseApp page fetch was unavailable; actual remote metadata supplies
the prerequisite evidence. No ARM package build or installed behavior inferred.

Added build-flatpak-ci.sh/lib/flatpak_ci.py. Requires matching manual workflow,
native CPU, exact SHA and clean checkout before runner mutations; work/log/output
locations stay outside source and separate. Ephemeral runner only: install build
tools, relax Ubuntu AppArmor user-namespace restriction when present, install
manifest runtime/SDK/BaseApp refs, then call maintained build-flatpak.sh once.
No runtime swap, source-pin rewrite, app installation or hosted dispatch by script.
No operator-host APT, kernel setting or runtime mutation performed locally.

Collection rejects failed/wrong-source/architecture producers, corrupt artifacts,
bundle/export ref/commit or metadata differences. Successful uploads include
bundle, built metadata, original summary/schema2 receipt, portable hosted index
with SHA/run/repository/ref/commit/runtime/app-source pins, and SHA256SUMS.
Original receipt paths remain runner-local; portable index names downloaded
files, not a claim of local candidate-reuse compatibility. Separate diagnostic
artifacts retain summaries on failure, not raw private logs. Installed acceptance
remains not-run and current Debian-only publisher is unchanged. Flatpak promotion
support remains an open step8 task; publication will reuse accepted artifacts.

Updated script catalog/inventory, packaging guide and release procedures. New
tests/flatpak_ci_test.py covers both architectures, nonmanual/wrong-native/source
contexts, dirty checkout refusal before runner changes, partial builder failure,
producer/hash/ref/metadata mismatch, portable file checksums/provenance. All use
fake reports and synthetic files; no actual APT, sysctl, SDK install, builder or
GitHub dispatch. Five focused tests exit0. Workflow PyYAML parse, native matrix,
manual/read-only/pinned-action assertions and bash -n run-block checks exit0;
driver bash -n/--help and documentation link/diff checks exit0.

Final gate running with authorized runtime-directory access (earlier tasks
established sandbox /run/user/1000/just limitation); outcome follows. Existing
local x86_64 candidates not rebuilt or invalidated by these tooling changes;
new hosted/ARM provenance and installed acceptance need separately approved
execution after commit/push. User Push Buttons.md and unrelated edits preserved.
No commit/push/tag, hosted dispatch, package install or publication in this task.

Final gate dev-gate-2qaal4fe succeeded, exit0, source_dirty=true at source
1aa9122a2140eab299cb5ba6429e08d046dd7c49. Receipt/log:
target/task-runs/dev-gate-2qaal4fe/{summary.json,task.log}. Formatting, Clippy
and Rust/Python suites passed, including new hosted-policy fixtures. No subsequent
executable changes. Workflow implementation complete locally; first approved
hosted run/download verification retained as a separate outstanding task.


### October 19, 2026 — Push only hosted Flatpak build prerequisites

Operator approved commit/push, then narrowed scope to what is necessary to run
the Flatpak packages. Committed only build-flatpak.yml, build-flatpak-ci.sh,
lib/flatpak_ci.py and tests/flatpak_ci_test.py as
d7e94029f3aab1a2c4e4175b2d365b0d7c03edff. Installers, README, release/acceptance
records and operator future-GUI notes remain local. Existing shared build helpers
and pinned manifest are already tracked/published; no runtime source input changed.
Staged diff check exit0; unchanged-input gate dev-gate-2qaal4fe reused.
Fetched origin before commit and again immediately before push; no upstream
changes (one local commit, zero remote). git commit, fetch and push exited0;
remote master verified by git ls-remote at exact d7e94029f3aab1a2c4e4175b2d365b0d7c03edff.
No hosted dispatch, package installation, tag or publication authorized/performed.
This execution note remains local under the operator's narrow push scope.

### October 19, 2026 — First hosted Flatpak failure: missing elfutils

Operator supplied x86_64 failure output at source d7e9402: Rust build/install
reached flatpak-builder finishing, then eu-elfcompress was unavailable and the
mandatory eu-strip invocation failed with missing executable. No exported bundle
or hosted success inferred. Runner APT setup uses --no-install-recommends and
did not explicitly request elfutils. Local dpkg-query confirmed /usr/bin/eu-strip
and /usr/bin/eu-elfcompress both belong to elfutils. Ubuntu package website
queries were unavailable; no web evidence or package installation claimed.

Added elfutils to fixed runner build dependencies, then explicit bounded
eu-strip/eu-elfcompress --version checks before runtime downloads/build. These
are CI tool prerequisites, not application startup engine probes. Updated fake
orchestration test to verify package/tool setup; added missing-tool failure cases
that stop before runtime downloads or builder. Six focused tests pass, exit0;
selected source/test/catalog/inventory diff check exit0. Final gate pending below.
Catalog/inventory/task records updated locally; only the two runtime-tooling/test
files are intended for commit/push under the operator's existing narrow approval.
No app source/pin/runtime choice or permissions changed; prior installed behavior
does not need repeating for this runner dependency correction. No local install,
hosted dispatch or publication performed. A retry must be a new workflow dispatch
on the corrected commit; rerunning the old run retains its old dependency setup.

Gate dev-gate-0b5sn6tp failed at repository hygiene after the source/fake tests:
newly tracked build-flatpak-ci.sh had mode100644. Earlier pre-commit gate ignored
untracked script modes, so that pass did not establish clean-checkout hygiene.
Corrected executable mode100755 and staged it. Focused repo_hygiene_test.py
(four tests) and flatpak_ci_test.py (six tests) pass, exit0. Added only the CI
wrapper/module catalog entries and the CI inventory entry/review date to the
staged correction so a clean checkout has required documentation; unrelated
installer entries/docs remain local. Staged diff check and narrow-catalog
assertions exit0. Final gate retry pending below; no further code changes.

Final gate dev-gate-f6ynq1tg succeeded, exit0, source_dirty=true at d7e9402;
receipt/log target/task-runs/dev-gate-f6ynq1tg/{summary.json,task.log}. Committed
only five correction/prerequisite files as ba8a34158f86a168b260af2e6b7829c89393ed14.
Fresh fetch before push: one local commit, zero remote-only commits. Commit,
fetch, push and remote ls-remote verification exit0; origin/master equals that
SHA. No amended history, tag, dispatch, app install or publication. Installer,
README, broader release records and user future-GUI changes remain local; these
execution notes/task status remain local under the narrow push scope. New hosted
run on corrected master still needed; source/fake gate is not hosted acceptance.

### October 20, 2026 — ARM hosted success and x86_64 AppStream failure

Operator reported run39382160325 at ba8a341: ARM passed, x86_64 failed after
release compilation/stripping during AppStream compose (file-read-error,
filters-but-no-output). Read-only GitHub job/artifact inspection confirmed ARM
success/upload and x86_64 failure; no retry dispatched. Sandbox GitHub networking
failed; authorized read-only retry worked. Archived existing flatpak-aarch64
artifact under target/artifacts/candidates/flatpak-hosted-ba8a341/flatpak-aarch64.
All uploaded SHA256SUMS entries and source/run/arch identity verified. OSTree
commit2e35268ef8769c645c30334a10ebb9fc34aa36cd809b32dd9d711533646917bc.
No app installation or ARM runtime exercise claimed.

Locally reproduced exact AppStream failure using the unchanged app files and an
empty GDK_PIXBUF_MODULE_FILE: file-read-error identifies the SVG icon with
Unrecognized image file format; filters-but-no-output is consequential. Normal
loader cache composes successfully with only existing nonfatal COSMIC hints.
Real tool AppStream1.0.2 matches Noble package family; reproduction output under
target/task-runs/appstream-ci-repro-b2zw12p1/. This isolates SVG-loader support;
failed runner package inventory/hints were not available, so do not claim those
were directly observed. Upstream issue619 describes similar errors but different
icon installation misuse; not used as the diagnosis of our correctly installed
SVG. No AppStream/metadata validation disabled and no resources changed.

CI explicitly requests appstream/appstream-compose/librsvg2-common rather than
depending on runner recommendations/preinstalls. New preflight extracts actual
metainfo/desktop/SVG from the app manifest's immutable Git pin, composes with
--no-net/--print-report=full before runtime download/build, records AppStream and
package versions, retains issue hints with summary diagnostics. Workflow choice
both/x86_64/aarch64 allows retrying only the failed target. Existing app source,
runtime, permission grants and pinned source remain unchanged. Source identity
policy at final promotion remains explicit even when preserved ARM is reused.

Eight fake CI-policy tests pass, exit0, including compose failure before any SDK
or builder call and refusal of unpinned preflight inputs. Workflow YAML/choice/
run-block syntax assertions pass, exit0. Actual check_appstream function using
pinned app source passed (not just fake adapter) under
target/task-runs/appstream-pinned-preflight-vlr5pvem/report/{summary.json,task.log}.
No native package install, app build or cloud/provider retest performed locally.
Catalog/inventory/packaging/task records updated; final gate outcome follows.

Final gate dev-gate-ooqx5x1_ succeeded, exit0, source_dirty=true at ba8a341;
receipt/log target/task-runs/dev-gate-ooqx5x1_/{summary.json,task.log}.
Committed only five CI workflow/driver/test/catalog/inventory files as
095c18043a36745d99f536b8e43fc3fa4866c1f6. Fresh fetch showed one local and zero
remote-only commits; commit/fetch/push/ls-remote all exit0. Remote master equals
095c18043a36745d99f536b8e43fc3fa4866c1f6. No hosted dispatch, app install or
publication. Operator can dispatch new workflow on master, architecture=x86_64;
successful ARM remains preserved, with overall source/provenance release gate
still separate. Other installers/README/release records and these execution
notes remain local under the existing narrow push scope.


### October 20, 2026 — Hosted Flatpak build artifacts verified for both CPUs

Operator reported retry complete. Read-only GitHub verification confirms
run39433010848 at095c18043a36745d99f536b8e43fc3fa4866c1f6 completed successfully
with only Flatpak x86_64 job; bundle/provenance and diagnostic artifacts uploaded,
unexpired. Existing ARM job run39382160325 atba8a341 remains the preserved
successful ARM build (its whole matrix run failed x86 before correction). No
new build dispatched and ARM not rebuilt. Initial sandbox network request failed;
authorized read-only run/job/artifact queries and download succeeded, exit0.

Downloaded x86 assets under target/artifacts/candidates/flatpak-hosted-095c180/
flatpak-x86_64; ARM retained under flatpak-hosted-ba8a341/flatpak-aarch64. All
SHA256SUMS/index file hashes, producer success and source/run/architecture identities
passed. Complete source pins and manifest/cargo-source fingerprints match across
architectures. App source72661ce9d705038cdb1878165917f298e0ced8c2 unchanged.
x86 bundle:13,442,696 bytes, SHA256
64dd3652bb265f2275d5c1252b8f4e6858234f61cf1df376c428c9ec45a16150, OSTree
2ab220a037806b795523f9b6d345f95a1b5b998578ccf15386a050883df3d7c8.
ARM bundle:12,644,456 bytes, SHA256
da95586ee661c6cf8389023c0b744f42adf6efe73e0cbb0a6b33ed2c3d6532c6, OSTree
2e35268ef8769c645c30334a10ebb9fc34aa36cd809b32dd9d711533646917bc.
Machine-readable comparison retained under
target/task-runs/hosted-flatpak-acceptance-2026-10-20/verified-artifacts.json.

Ran maintained scripts/inspect-artifacts.py flatpak-bundle for each CPU with
explicit app/ref/arch, expected producer metadata and recorded OSTree commit.
Both first attempts blocked on sandbox read-only O_TMPFILE, not corrupt bundles.
Authorized retries both passed, exit0: target/task-runs/hosted-flatpak-inspect-
x86-2026-10-20-retry/ and hosted-flatpak-inspect-arm-2026-10-20-retry/ contain
summary/artifact/log receipts. Temporary import only, no installation/execution.

Full hosted x86 metadata differs from prior accepted local7069ae5 only in Context:
shared list order and sockets explicit x11+wayland+fallback-x11 (host Flatpak1.14.6)
versus wayland+fallback-x11 (local1.18.1). Reviewed official Flatpak command
reference: fallback-x11 takes precedence over x11 when supported. Specific
list-order/fallback equivalence assertion passed; all other full metadata agrees.
No inspector implementation or strict per-bundle baseline was changed. No blanket
claim of installed effective permissions; actual installed upgrade remains separate.

Updated task list/acceptance map/tooling inventory; no source inputs changed.
Reused gate dev-gate-ooqx5x1_ and source/fake checks without repetition. New source
packaging identities differ across CPUs and remain explicit; final immutable tag,
Flatpak-aware promotion/checksum assets and focused clean-profile install still
open. Local documentation checks follow; no commit/push, tag, publication, package
installation, provider/cloud operation or additional CI dispatch in this turn.


### October 20, 2026 — Flatpak-aware promotion and focused VM check preparation

Operator authorized the publisher follow-up and selected the existing Pop!_OS VM
for the focused installation check. No publication/tag/push/build dispatch is
part of this implementation. scripts/promote_release.py and
scripts/lib/flatpak_promotion.py now verify explicit per-CPU hosted runs, selected
successful jobs (including a successful CPU in a failed matrix), complete
checksums, producer/source/input/runtime provenance and offline ref/commit/full
metadata. Normal promotion retains exact release-tag packaging source checks.
Optional publication workflow inputs select each CPU; combined SHA256SUMS and
portable FLATPAK-PROVENANCE.json accompany Debian/bundle assets. Receipt origins
identify each bundle's actual run; no deleted runner paths are published.

The new --inspect-flatpaks mode cannot publish/install and retains verified
candidate bytes/provenance without a final tag or Debian run. Real command:
python3 scripts/promote_release.py v0.5.4 --inspect-flatpaks
--flatpak-run x86_64=39433010848 --flatpak-run aarch64=39382160325
--repo uutzinger/cosmic-ext-applet-mounter
--log-dir target/task-runs/flatpak-promotion-candidates-2026-10-20
passed exit0. summary.json/artifacts.json/FLATPAK-PROVENANCE.json retained in
that directory, with bundles under flatpak-artifacts/flatpak-<arch>. acceptance
is not-run, publication not-performed. Reuses the same successful hosted bytes;
no rebuild, cloud operation or installation. This real verifier result remains
valid after receipt-origin-only changes. Existing Debian7069ae5, x86_64095c180
and ARMba8a341 packaging SHAs remain distinct; matching application pins do not
waive the final exact-tag gate.

Added --bundle FILE to the standalone Flatpak release installer to test its
checksum/runtime/install steps before release assets are public. Local mode
requires the exact CPU/app filename and regular sibling SHA256SUMS, copies
verified inputs to a private temporary directory, and uses the same user
Flathub/Platform25.08/bundle installation path. No provider/dependency setup,
service stop, app launch or automatic rollback added. README curl command stays
unchanged. Catalog, inventory, procedures/output contracts and packaging guide
updated. Focused tests: flatpak_promotion_test.py 7 passed;
release_promotion_test.py 15 passed; flatpak_release_installer_test.py 6 passed.
All exit0. Workflow manual inputs and every Bash run block validated with
PyYAML BaseLoader/bash -n; installer bash -n and targeted git diff --check pass.
Fake publication test messages refer only to fake adapters, not a real release.

Read-only SSH to the VM's previously supplied address timed out (exit255); no
remote changes occurred. Existing VM already has native test profiles, so merely
installing there will establish a focused replacement/first-Flatpak smoke, not
a clean-profile result unless a clean profile is actually used. Operator
readiness/address requested before replacement. VM installation/panel discovery
and final source/tag/publication decisions remain open. Final gate result follows.

Operator supplied the VM Tailscale hostname after the private-LAN timeout.
Read-only SSH reached it but BatchMode authentication was denied (exit255);
no credentials requested or remote state changed. Operator copy/local-bundle
installation commands supplied using the verified retained x86 artifact and
sibling SHA256SUMS. Installation result and panel smoke remain pending.

Final verification: bash scripts/dev.sh gate succeeded, exit0, at source
095c18043a36745d99f536b8e43fc3fa4866c1f6 with source_dirty=true. Receipt/log:
target/task-runs/dev-gate-85flvcye/{summary.json,task.log}. Includes formatting,
Clippy, Rust and Python suites. No Rust/UI/AppStream source changed; no separate
localization/metadata or installed cloud test repetition required. Publisher
implementation checkbox closed; hosted real publication, VM install/panel smoke,
clean-profile coverage and final immutable source/tag remain open. No commit,
push, tag, build dispatch, publication or app installation performed.


### October 20, 2026 — Operator VM Flatpak installation succeeded

Operator screenshot shows the maintained install-flatpak-release.sh --bundle
command on the existing Pop!_OS VM, version0.5.4/x86_64. Selected bundle
SHA256SUMS verification reports OK; output identifies installation of
app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master and the
flatpak-release-install summary reports exit_code0. This establishes successful
operator-run checksum/install execution; independent installed commit readback,
panel discovery and rendered settings/editor smoke remain pending. Existing
native profiles are retained; no clean-profile or ARM installed acceptance
inferred. Documentation-only progress update reuses passing gate
dev-gate-85flvcye; no source changes or additional tests/builds/installation by
the agent. Next: read back installed commit, add panel applet and check settings
and an unsaved Add Connection window without starting sync.

October 20 follow-up: operator screenshot of flatpak info --user --show-commit
io.github.uutzinger.cosmic-ext-applet-mounter returns
2ab220a037806b795523f9b6d345f95a1b5b998578ccf15386a050883df3d7c8, exactly
matching the verified hosted x86 bundle. Installed commit identity accepted by
operator readback. Panel launch/discovery and rendered windows remain unverified;
installation identity alone does not identify which native/Flatpak launcher the
panel selects. No source edits or repeated tests.


### October 20, 2026 — Hosted x86 Flatpak panel/UI smoke accepted on VM

Operator screenshot of flatpak ps --columns=application lists
io.github.uutzinger.cosmic-ext-applet-mounter, confirming the sandboxed applet
is running after panel selection. Operator confirms it opens, Settings display
and Modify Connection is accessible. Together with the earlier successful
installer checksum/exit0 and exact installed commit readback, this accepts the
focused existing-VM install/panel/settings/editor smoke for hosted x86 bundle
2ab220a037806b795523f9b6d345f95a1b5b998578ccf15386a050883df3d7c8.
Native and Flatpak both appear as Cloud Mounter in the panel picker; ordering
was not used as identity evidence. No claim of saving a modified connection,
new cloud synchronization, public GitHub download, clean-profile setup or ARM
installed behavior. Earlier unchanged provider/VPN/mirror results reused.

Updated Task List.md, this chronology and the acceptance map/index. No source
edits; prior gate dev-gate-85flvcye reused without repetition. Final source/tag
identity, uncommitted tooling/docs, clean-profile coverage and publication remain
open; no additional build, install, commit, push or publication performed.


### October 20, 2026 — Release identity decision and commit preparation

Operator authorized resolving source/tag identity and committing the prepared
tooling/records. Reviewed master/status and intended changes. Existing Debian
7069ae5, x86 Flatpak095c180 and ARMba8a341 remain preserved accepted candidates
at their recorded scopes, not one exact-tag release set. New public installers
and Flatpak-aware publisher are not present in those old packaging revisions.
Decision: commit this preparation, finalize remaining notes/acceptance, freeze
one packaging SHA and request a single Debian/Flatpak build set at that SHA.
The app pin72661ce stays unchanged. Release tag must reference that same source;
acceptance receipt-only follow-ups do not move it or trigger rebuilds. No tag
selected, new build dispatched or policy mismatch waived. Tests for unchanged
provider/VPN/sleep behavior remain reusable.

Commit scope: standalone Debian/Flatpak installers and fake tests; Flatpak
promotion library/driver/workflow and tests; concise README, package/setup
guidance, acceptance/submission audits and chronological release records.
Unrelated operator-authored future GUI/AI prose and Push Buttons.md remain
local. Gate dev-gate-85flvcye passed for these code inputs and is reused; only
documentation and installer executable metadata changed afterward. Focused
staged repository hygiene/whitespace verification and commit result follow.

Staged review: 23 intended files; git diff --cached --check exit0.
python3 tests/repo_hygiene_test.py: four tests passed, exit0, including newly
tracked installers'100755 modes and relative links. git diff against72661ce
for src/i18n/Cargo.toml/Cargo.lock/resources is empty, confirming unchanged
application payload inputs. Source freeze remains outstanding because release
notes are still explicitly draft and acceptance limits remain recorded.
Prepared commit title: Prepare verified GitHub package installation and promotion.
Only the release task-list patch staged; operator future GUI/AI prose and
Push Buttons.md excluded. No new build, service or publication action authorized.
