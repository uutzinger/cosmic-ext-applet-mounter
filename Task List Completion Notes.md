# Cloud Mounter Applet Task List Completion Notes

Dated execution evidence for the active [Task List](Task%20List.md). Entries through
the native v0.5.3 publication are archived, unchanged, in
[docs/archive/task-list-completion-notes-through-v0.5.3.md](docs/archive/task-list-completion-notes-through-v0.5.3.md).

Entries through v0.5.4-pre: [archive](docs/archive/task-list-completion-notes-through-v0.5.4-pre.md).

Entries through v0.5.4: [archive](docs/archive/task-list-completion-notes-through-v0.5.4.md).

### Post-release closure — October 20, 2026

GitHub v0.5.4 published and all ten public asset downloads verified. The
maintained archive script will run in dry-run mode first, keeping the operator's
0.5.5 and later future sections active. --allow-dirty is the script's explicit
capability for preserving current uncommitted release receipts and user edits;
no commit/stash or history change is performed. Archive outcomes follow.

Maintained archive dry-run and actual --allow-dirty execution both passed,
exit0. Reports: target/task-runs/release-054-archive-preview/ and
release-054-archive/. Archived716 task lines and5976 note lines to
docs/archive/task-list-through-v0.5.4.md and its completion-notes counterpart.
Zero new open items, so no additional verbatim carry-forward blocks needed.
Existing condensed carried-forward entries preserved; operator 0.5.5/future
sections verified byte-for-byte equal before/after archival. Original snapshots
retained privately under /tmp/cloud-mounter-pre-archive-054-{task-list,notes}.md.
Root intro/carry guidance and two historical task-anchor links updated to the
archive. Local README and release index/notes now mark publication; engineering
map/index record final public asset checks. Frozen tag and artifacts unchanged.
No additional Git commit/push or package installation performed. Focused
documentation-link/whitespace verification follows; reuse prior dev-verify-
gjyhru76 because no application/script code changed.

Post-archive validation: python3 tests/repo_hygiene_test.py passed four tests,
exit0, including all relative Markdown file links. Both archived bodies equal
the original captured sections exactly apart from the maintained script's
relative-link rewrites and terminal newline handling; future suffix equality
and carried-forward retention assertions passed. No open items lost.
Targeted whitespace checks passed; existing operator future prose remains
untouched. Public release is complete. README/release-status/audit/archive
cleanup is prepared locally and uncommitted, requiring separate commit/push
authorization before it appears on master. Tag/source and release assets stay
immutable.


### Publication cleanup commit/push — October 20, 2026

Operator explicitly approved committing and pushing README publication status
and release-record/archive cleanup. Reviewed branch/status, intended diffs and
archive boundaries. Commit includes only release documentation, acceptance
map/index, archive files and the release-only active task-list transformation.
Operator future GUI/AI prose and Push Buttons.md remain local and excluded.
No package/source input changed, no tag/asset/release-body mutation. Prior
repo_hygiene_test.py four-test pass and dev-verify-gjyhru76 are reused because
code/links are unchanged. Staged whitespace/mode checks and fresh upstream
reconciliation precede push; no force or history rewrite permitted.

Staged review: ten intended documentation/archive files; git diff --cached
--check passed, exit0. Remaining unstaged Task List diff contains only the
pre-existing future GUI/AI changes (56 insertions/6 deletions); Push Buttons.md
remains untracked. Archive preservation and link checks retain their passing
results; no application, script, runtime, package or tag change.
Prepared commit title: Record v0.5.4 publication and archive release history.

### 0.5.5 scope review — October 20, 2026

Scope review completed; implementation and installed acceptance remain open.
Read AGENTS.md, the 0.5.5 task sections and Push Buttons.md. Confirmed master
at 057b6474ce206768645c841bc4f0a1c9818640d0 with the operator's Task List.md
edits and untracked Push Buttons.md. Added scope/checklists only to Task List.md
and this evidence entry; all pre-existing task prose remains unchanged.
Push Buttons.md is byte-for-byte unchanged. No application/script/package
changes, commit, installation, service operation or submission performed.

Execution order: complete abraunegg retirement, then GUI cleanup, then COSMIC
Flatpak submission preparation. Removal retains inert legacy identities and
recovery data, protects unrelated services and preserves rclone/onedriver.
GUI checklist covers remote removal, Add/Modify action layouts and state
transitions, localization and separate installed visual acceptance.
Existing expansion backlog remains deferred. Submission prose about retaining
onedrive is historical 0.5.4 guidance and must be refreshed for the final source.
Current external contribution policy was not verified in this scope review;
that check remains explicitly pending before submission.

Verification commands, all exit 0:
- git status --short; git branch --show-current; git log -3 --oneline;
  git rev-parse HEAD: baseline and operator edits confirmed.
- git diff a2fb318 057b647 -- src resources i18n Cargo.toml Cargo.lock scripts
  tests justfile: empty; same paths compared against current HEAD also empty.
- Python difflib additive-only assertion on Task List.md, byte equality for
  Push Buttons.md, and relative-link checks for Task List.md: passed.
  Original snapshots: /tmp/cloud-mounter-055-scope-original.md and
  /tmp/cloud-mounter-055-buttons-original.md.

Reused existing dev-verify receipt:
source a2fb318b7d624470430dead01f2878f7e03590b6, task dev-verify,
status succeeded, exit_code 0, source_dirty true;
target/task-runs/dev-verify-gjyhru76/summary.json and task.log.
The archive already records its reuse through the 0.5.4 release, and the
checked code/tool inputs remain unchanged. The dirty-source receipt is prior
baseline evidence, not a clean-tree 0.5.5 acceptance result. No repeat gate
was run for this additive planning-only edit. No compilation, live-engine,
installed-service, rendered-UI or final-package acceptance is newly claimed.

Failures: none. Open decisions: Sync Now styling versus enablement after
Preview, Re-Test changed/passed semantics, and precise remote-reference guards.
Next step: inspect legacy profile dispatch and owned-unit retirement paths
against the phase-1 checklist, then implement and run focused verification.

### 0.5.5 abraunegg source removal — October 20-21, 2026

Source removal and fake-adapter acceptance are complete. Installed upgrade and
package acceptance remain open, so Phase 1 is not yet marked complete. The
project development version is 0.5.5; Debian changelog status is UNRELEASED and
the AppStream release list still identifies 0.5.4 as the published release.

Old OneDrive Offline profiles retain their serialized identity and remain
visible and removable. Absent engine values remain LegacyOnedrive and are never
converted to rclone. Authentication, Test, Preview, Sync, Start, scheduling,
pending-count and Repair paths return a typed unsupported result. Pause remains
available only for an observed running legacy monitor so the app can retire it.
Profile removal first retires the unit and reloads authoritative configuration;
a retirement failure or concurrent profile change leaves the profile intact.
Mirror, recovery, database, cache and credential files are never removed.

Upgrade startup uses the typed systemd user-manager adapter to inspect old units.
Only a regular, unchanged app-owned unit with the matching connection UUID,
expected user-unit path, no drop-ins and exact `/usr/bin/onedrive` ExecStart is
eligible. Retirement disables and stops it, verifies inactive/disabled state,
removes the unchanged unit file and reloads systemd. Missing units are a no-op;
foreign, malformed, linked, changed, active, enabled, cancelled and partial-failure
cases fail closed. Rollback may restore the unit file after a removal/reload
failure but never enables, starts or restarts the retired engine.

Removed runtime surface: legacy sync/auth request planners, status and pending
parsers, executable and diagnostic routes, app authentication/manual handoff and
operation dispatch, dependency installer option/implementation, and Debian
recommendation. Rclone OneDrive identity/guards/OAuth, onedriver Online, and
development-only rclone OneDrive probes remain. The installer now rejects the
retired options before privileged work; it does not uninstall existing packages.

Changed files:
- Runtime and tests: Cargo.toml, Cargo.lock, src/app.rs, src/controller.rs,
  src/diagnostics.rs, src/model.rs, src/pending_count.rs, src/process.rs,
  src/services.rs, src/sync.rs, tests/debian_packaging_contract.rs,
  tests/dependency_installer_test.py, tests/onedrive_configuration.rs and
  tests/setup_reporting_test.py.
- Packaging/tooling: debian/changelog, debian/control,
  scripts/install-dependencies.sh and scripts/README.md.
- Product/release records: Applet Description.md, Requirements and
  Specifications.md, README.md, Task List.md, resources/app.metainfo.xml,
  i18n/en/cosmic_ext_applet_mounter.ftl, docs/dependency-installation.md,
  docs/microsoft-cloud-auth-setup.md, docs/engineering/README.md,
  docs/engineering/flatpak-host-command-inventory.md,
  docs/engineering/onedrive-rclone-offline-contracts.md,
  docs/engineering/script-tooling-inventory.md, packaging/flatpak/README.md,
  docs/releases/README.md and the new draft docs/releases/v0.5.5.md.

Verification passed:
- `bash scripts/dev.sh check`: final receipt
  target/task-runs/dev-check-0v4hh7fu/summary.json, exit 0.
- `cargo test --locked retired_engine`: 8 passed, 0 failed; retirement,
  unsupported dispatch, inert controls, ownership, cancellation, rollback and
  data-preservation cases.
- `cargo test --locked onedrive`: 34 library passed/3 installed-rclone tests
  ignored, 30 app passed, Debian contract passed, 10 configuration and 13 setup
  integration tests passed; all failures zero.
- `python3 tests/dependency_installer_test.py`: 6 passed; `python3
  tests/setup_reporting_test.py`: 13 passed.
- `bash scripts/dev.sh localization-check`: receipt
  target/task-runs/dev-localization-check-c3tex7vh/summary.json, exit 0.
- `bash scripts/dev.sh metadata-check`: receipt
  target/task-runs/dev-metadata-check-46od10qx/summary.json, exit 0.
- `bash scripts/dev.sh gate`: final receipt
  target/task-runs/dev-gate-s1un16bc/summary.json, exit 0. Formatting, Clippy
  with warnings denied, the full non-live Rust/Python suite and repository checks
  passed against dirty source at HEAD 057b6474ce206768645c841bc4f0a1c9818640d0.
- `python3 tests/repo_hygiene_test.py`: 4 passed after the final task/evidence
  edits; relative Markdown links and tracked-file hygiene remain valid.

Failure/fix chronology: first gate receipt dev-gate-3qn4u8f9 stopped at format
differences; `just fmt` fixed only edited Rust. The next receipt
dev-gate-65v2tzgi found an orphaned removed-parser comment and redundant
must-use attribute; both were removed and focused checks passed. Receipt
dev-gate-1823dg4j passed formatting/Clippy but the sandbox blocked
`/run/user/1000/just`; the same maintained gate was rerun with authorized normal
runtime access. Receipt dev-gate-0hcjugal then exposed a stale diagnostic count
of six instead of five supported reports; the assertion and focused diagnostic
test were corrected before the final passing gate. No live or ignored tests were
reclassified as passes.

Open risk and next step: source tests cannot prove behavior against the installed
user systemd manager or a real pre-upgrade unit. Build/install a reviewable 0.5.5
candidate only after separate approval, then exercise a disposable app-owned
legacy profile/unit and verify the profile/data preservation and unit retirement
from the installed applet. Native, Debian and Flatpak package acceptance remain
unclaimed. No service, package, credential, cloud data, commit, push or COSMIC
submission was changed. Push Buttons.md remains untouched for the GUI phase.

### 0.5.5 native legacy-upgrade acceptance — October 21, 2026

The operator installed the current 0.5.5 development candidate with the
maintained `just install-user` recipe and reloaded the COSMIC panel. The installed
binary is byte-identical to `target/release/cosmic-ext-applet-mounter`, SHA-256
`327071a07c4229671f8a379b653852696abab2d9f08852563a542f66603b33ad`.
This was an existing app-owned legacy profile and unit, not a disposable profile;
the configuration, unit and prior installed binary were backed up before testing.

Rendered installed UI evidence from the operator: the legacy row remains visible,
shows its retired status, and Sync and Start are unavailable. Read-only local
inspection confirms that the saved profile remains present with its absent-engine
legacy identity, its mirror and recovery directories still exist, and its
legacy database/credential directory remains present with six entries. Contents
and credentials were not read.

Installed service evidence from the operator after panel reload:

```text
PASS: legacy unit removed
LoadState=not-found
ActiveState=inactive
FragmentPath=
UnitFileState=
```

This accepts native installed startup retirement and rendered retired-row behavior
at verification levels 4 and 5. Profile removal was deliberately not exercised
against the real profile; synthetic tests cover successful removal and failure
preservation. Debian and Flatpak package-specific upgrade acceptance remain open
and are not implied by this native installation. No cloud operation, credential
change, mirror/recovery deletion, commit, push or publication occurred. Phase 1
is complete; GUI cleanup is next.

### 0.5.5 GUI action inventory — October 21, 2026

Started Phase 2 by reading the operator's unchanged Push Buttons.md, the current
GUI task section and the editor/settings implementation. Added
docs/engineering/gui-action-layout-inventory.md and indexed it from the engineering
README. No UI code, localization or runtime behavior changed.

The source inventory covers Settings and Add/Modify for OneDrive Online and
Offline, SharePoint, Google Drive, Box, SMB and SFTP in both modes. Existing code
puts Test/Save before prerequisites, gives only OneDrive Offline two rows, omits
Detect from other Modify editors, lacks Box reconnect, and does not style Save
from its existing exact-draft validation result. Grey Test/Save/Preview/Sync
buttons are generally executable, so current color does not reliably communicate
availability. Settings-level all-remote removal is absent; a narrower matching-
provider removal list already exists inside Add and will be moved/reworked rather
than duplicated.

The proposed contract defines blue as the available recommended next action,
enabled grey as secondary, disabled grey as unavailable, and red as
disable/remove. It defines binding changes separately from ordinary edits,
invalidates Re-Test/Save after relevant changes, and resolves the ambiguous grey
Sync Now rule through enablement: initial Sync Now stays disabled until Preview
passes, then remains an enabled secondary grey action. Offline runtime actions use
a second row for every provider. Box reconnect remains an explicit backend
decision rather than an invented button.

Verification: `python3 tests/repo_hygiene_test.py` is run after this record and
link update. Source inspection only; no compilation, rendered UI, native/Flatpak
layout or accessibility acceptance is claimed. Next step is to define the typed
editor action state and move all-remote management into Settings before changing
button layout.

### 0.5.5 Settings rclone remote manager — October 21, 2026

Implemented the Settings-level **Remove Remote** workflow in `src/app.rs` and
localized its labels and notices. The button appears immediately after **Add
Connection** and opens a dedicated window. Discovery now retains all typed
rclone remotes for this inventory while Add/Modify selection continues to filter
for provider-compatible remotes. Each manager row shows its backend and either
the saved connection names that use it or that it is unused. Used remotes expose
a disabled grey **In use** action; unused remotes expose red **Remove** and red
confirmation actions. The former provider-limited removal controls were removed
from Add, while its remote selection controls remain.

The existing app-owned reference rule remains authoritative: every non-OneDrive
connection reference counts, while OneDrive counts only rclone Offline mirrors
and excludes onedriver Online mounts. The asynchronous deletion path now reloads
the current app configuration and repeats this reference check immediately
before issuing the fixed `rclone config delete <name>` request. A failed check or
command leaves saved connections, local data and cloud files untouched and is
reported in the manager.

Files changed for this step: `src/app.rs`,
`i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`, and
`docs/engineering/gui-action-layout-inventory.md`, plus this completion note.
`Push Buttons.md` remained unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

### 0.5.5 Test and selected-remote verification merge — October 21, 2026

Removed OneDrive Offline's separate **Verify selected remote** button and its
Fluent labels. **Connect new remote** still creates and authorizes a new rclone
remote. When **Test Connection** sees an existing OneDrive remote whose current
remote, drive type, expected account or folder has not been verified, it now
runs the existing non-destructive remote verification and automatically
continues the full connection plan/dependency/access test after accepting the
identity. Cancellation, stale-result rejection and exact-draft validation remain
in force. Other providers already verify selected remote access in Test.

The broader OneDrive editor test exposed an inventory regression from retaining
all rclone backends for the new manager: an incomplete `onedrive` entry and a
SharePoint `documentLibrary` entry both previously had no parsed personal/work
drive type. Added an explicit document-library marker so all remotes remain
visible in the manager but only actual document libraries appear in SharePoint
selection.

Files changed: `src/app.rs`, `src/app/sftp.rs`,
`i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`, and
`docs/engineering/gui-action-layout-inventory.md`, plus this completion note.
Verification:

- Initial `bash scripts/dev.sh check` failed because the new inventory field was
  missing from one SFTP test fixture; the fixture was updated.
- `just fmt` and the subsequent `bash scripts/dev.sh check` exited 0; receipt
  `target/task-runs/dev-check-hn9gd2av/summary.json`.
- `cargo test --locked onedrive_editor` initially exposed the incomplete-remote
  SharePoint filtering defect; after the explicit marker fix, all six focused
  editor tests passed.
- `cargo test --locked test_connection_continues_after_selected_onedrive_remote_verification`
  exited 0 with the merged workflow test passing.
- `bash scripts/dev.sh localization-check` exited 0; receipt
  `target/task-runs/dev-localization-check-_qz80vld/summary.json`.

Documentation hygiene and scoped whitespace checks follow this record. The final
gate and installed/rendered acceptance remain pending for the broader GUI phase.

Follow-up verification: `cargo test --locked rclone_remote` exited 0 with eight
focused tests passing, `python3 tests/repo_hygiene_test.py` exited 0 with four
tests passing, and the scoped `git diff --check` exited 0. Source and localization
contain no remaining Verify-selected action, message or Fluent key.
`Push Buttons.md` remains unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification:

- `just fmt` — exit 0.
- `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-pn64w44e/summary.json`.
- `cargo test --locked rclone_remote` — exit 0; 8 focused tests passed (one
  library test and seven app tests), including all-backend inventory,
  provider-specific selection, usage guards, the deletion-time recheck, and
  fixed delete arguments.
- `bash scripts/dev.sh localization-check` — exit 0; receipt
  `target/task-runs/dev-localization-check-n9ccpfuv/summary.json`.
- `python3 tests/repo_hygiene_test.py` — exit 0; 4 tests passed.
- Scoped `git diff --check` for this step's source, localization, engineering
  record and completion note — exit 0.
- `git diff --check` for the whole worktree remains blocked by trailing spaces
  in the operator's pre-existing uncommitted `Task List.md` submission notes at
  lines 236–238; those lines were preserved.

No final `gate` was run because the broader GUI cleanup phase remains in
progress. Rendered installed UI, focus/keyboard behavior and scaling are still
pending. The next source task is removal of the Settings Refresh action and
verification of its remaining automatic/shared refresh callers.

### 0.5.5 remote-manager row alignment follow-up — October 21, 2026

Rendered operator feedback found that the original `settings::item` layout
right-aligned the usage description and allowed each action to begin at a
different horizontal position. Replaced that row with two explicit columns: a
fill-width, left-aligned remote identity and usage description, followed by a
160-pixel action column at the right edge. **In use**, **Remove**, **Confirm
Remove**, and **Removing** buttons now share that width and horizontal position.

Files changed: `src/app.rs`, `Task List.md`, and
`docs/engineering/gui-action-layout-inventory.md`, plus this completion note.
`just fmt` and `bash scripts/dev.sh check` exited 0; check receipt:
`target/task-runs/dev-check-asmfnw2f/summary.json`. The focused
`cargo test --locked rclone_remote` exited 0 with the same 8 focused tests
passing. No final gate was run while the GUI cleanup phase remains active.
Installed rendering of the revised alignment remains pending.

### 0.5.5 Settings Refresh removal — October 21, 2026

Removed the manual **Refresh** button from General Settings along with its
dedicated `Message::Refresh`, `RuntimeCommand::Refresh`, success notice, help
text and test branches. The Settings connection actions are now **Add
Connection**, **Remove Remote**, and **Reorder Connections**.

Automatic/shared updates remain separate and intact. A standalone Settings
window polls runtime status and network readiness every two seconds. The running
applet observes mounts and user units every five seconds. Saves, removals,
reordering and popup startup-state changes still send
`RuntimeCommand::ConfigurationChanged`; the runtime owner reloads saved
configuration, clears stale pending counts, refreshes VPN readiness and returns
current status. The completion message was renamed to
`RuntimeConfigurationReloaded` to describe that remaining shared path.

Files changed: `src/app.rs`, `src/runtime_ipc.rs`,
`i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`, and
`docs/engineering/gui-action-layout-inventory.md`, plus this completion note.
Verification before the final documentation update:

- `just fmt` — exit 0.
- `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-h2k2v2sv/summary.json`.
- `cargo test --locked configuration_reload_preserves_operation_notice_and_pending_action`
  — exit 0; one focused test passed.
- `cargo test --locked background_poll_does_not_start_a_user_action` — exit 0;
  one focused test passed.
- `cargo test --locked poll` — exit 0; six focused polling tests passed.
- `cargo test --locked private_bus_routes_requests_errors_and_singleton_activation`
  initially could not bind its private D-Bus socket inside the filesystem
  sandbox; the authorized rerun outside that restriction exited 0 with the
  focused IPC test passing.

Localization and documentation checks follow this record. No final gate was run
while the broader GUI cleanup remains active. Installed/rendered verification is
still pending in the shared GUI acceptance item.

Follow-up verification: `bash scripts/dev.sh localization-check` exited 0
(`target/task-runs/dev-localization-check-7_bmfwn4/summary.json`),
`python3 tests/repo_hygiene_test.py` exited 0 with four tests passing, and the
scoped `git diff --check` for source, localization and records exited 0. Source
search found no remaining manual Refresh message, IPC command, notice or help
label. `Push Buttons.md` remains unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.


### 0.5.5 Modify editor button names and placement — October 21, 2026

Modify first-row actions now read **Detect, Re-Connect, Re-Test, Save,
Disable/Enable, Remove**. OneDrive Online omits Detect. Google retains
**Update Google OAuth** beside Re-Connect. Every saved Offline Mirror puts
**Preview, Sync Now** on a second row. Existing setup cancellation, SFTP update
confirmation, saved-identity checks and removal confirmation remain intact.
Google and Box show disabled Re-Connect with explanatory help because a separate
reauthorization backend is absent. OneDrive Offline and SharePoint use their
existing replacement-remote setup; SMB and SFTP use their existing update flows.

Files changed for this step: `src/app.rs`, `src/app/sftp.rs`,
`i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`,
`docs/engineering/gui-action-layout-inventory.md`, `docs/engineering/README.md`,
and this note. User-authored task-list content outside the scoped entry is
preserved. `Push Buttons.md` remains unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification chronology (source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`,
dirty worktree):

- Initial formatting/check passed before interruption: `just fmt`, then
  `bash scripts/dev.sh check`, exit 0; reused receipt
  `target/task-runs/dev-check-rci3b_02/summary.json`.
- `cargo test --locked sftp_remote_edits_highlight_update_in_add_and_modify`
  exited 0, one focused test passed.
- `bash scripts/dev.sh localization-check` exited 0; receipt
  `target/task-runs/dev-localization-check-xrr1tv7e/summary.json`. Later edits
  changed no catalog content, so this passing check is reused.
- Initial `gate` exited 101 on Clippy's `unnecessary_sort_by` in the earlier
  remote-manager work (`dev-gate-u9vf9dnf`). Replaced it with equivalent
  `sort_by_key`. `check` exited 0 (`dev-check-z2cf_3j6`) and
  `cargo test --locked rclone_remote` exited 0, eight focused tests passed.
- Next `gate` passed formatting/lint but exited 1 because the sandbox made
  `/run/user/1000/just` read-only (`dev-gate-g2mxaluc`). Authorized rerun outside
  the sandbox reached tests and exited 101 (`dev-gate-cc4h9fdh`): one SFTP test
  still asserted the old help wording. Updated that assertion to Re-Connect.
- `bash scripts/dev.sh check` then exited 0; receipt
  `target/task-runs/dev-check-bxjd67c0/summary.json`.
- `cargo test --locked sftp_discovery_filters_backend_and_help_matches_editor_mode`
  exited 0, one focused test passed.

Add-window naming/order and the shared action color/enablement model remain
separate outstanding tasks. Source compilation and synthetic tests do not verify
installed rendering. Native/Flatpak layout, scaling, focus and keyboard acceptance
remain open. No installation, service change, commit or publication was performed.

Final verification: authorized `bash scripts/dev.sh gate` exited 0; receipt
`target/task-runs/dev-gate-oo4lqdhl/summary.json`, log
`target/task-runs/dev-gate-oo4lqdhl/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and the repository Python suites passed. Ignored live tests
remain unrun and are not acceptance evidence. Scoped `git diff --check` exited 0
for source, localization, engineering index and completion notes. The new GUI
inventory also passed its whitespace check (intentional Markdown metadata hard
breaks retained). The operator's pre-existing task-list whitespace is preserved.
Next: Add-window names/order, then shared action-state/color implementation and
installed layout acceptance.


### 0.5.5 New Connection button names and placement — October 21, 2026

Add/New Connection now uses **Detect, Connect, Test, Save** for all rclone-backed
providers in both Online and Offline modes. OneDrive Online uses **Connect,
Test, Save**. Connect invokes the existing provider setup/create/update backend;
OneDrive retains setup cancellation and SFTP retains applying/confirmation states.
Changed Add help to reference the common button names and updated the existing
SFTP mode-help assertion. Shared colors and availability states remain separate
work; no backend or installed-rendering acceptance is claimed.

Files changed: `src/app.rs`, `src/app/sftp.rs`,
`i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`,
`docs/engineering/gui-action-layout-inventory.md`, `docs/engineering/README.md`,
and these notes. Existing operator task-list text is retained, with only the
scoped task status/progress added. `Push Buttons.md` remains unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification (source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`, dirty):

- `just fmt` — exit 0.
- `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-jy1cft5a/summary.json`.
- `cargo test --locked sftp_discovery_filters_backend_and_help_matches_editor_mode`
  — exit 0, one focused test passed.
- `bash scripts/dev.sh localization-check` — exit 0; receipt
  `target/task-runs/dev-localization-check-h3as5l3j/summary.json`.
- Scoped `git diff --check` for Rust, Fluent and the engineering index — exit 0.

Next: shared action-state colors/enablement and installed Add/Modify rendering,
keyboard/focus and scaling checks. No install, service change, commit or push.

The first gate attempt exited 101 (`dev-gate-tw47a3mp`) because
`remote_help_only_suggests_creation_in_add_mode` still expected the old Create
button wording. Updated its Add assertion to `click Connect`, preserving the
new-name and Modify-existing-remote checks. Also corrected SharePoint mode help
to Connect/Re-Connect. Follow-up `check` exited 0 (`dev-check-yf3tr0of`), focused
`cargo test --locked remote_help` exited 0 (one test), and localization-check
exited 0 (`dev-localization-check-tcbcyjed`).

Final `bash scripts/dev.sh gate` exited 0; receipt
`target/task-runs/dev-gate-tl1y230f/summary.json`, log
`target/task-runs/dev-gate-tl1y230f/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. Ignored live tests
remain unrun. Final scoped whitespace check exited 0 for changed source,
localization, engineering index and completion notes; operator-authored task-list
whitespace is retained. Installed UI acceptance remains open.


### 0.5.5 shared action-state and draft-change definition — October 21, 2026

Step 1 definition completed in the existing GUI action layout inventory. This
is a design contract; no Rust policy code, button behavior or color changed.
Defines Recommended, Available, Blocked(reason), Running(operation) and
Destructive presentation separately from operation results. Separates saved
settings changes, unapplied remote setup changes and current-test validity.
Maps actual `ConnectionDraft`/`SftpDraft` fields to those comparisons, with
consistent normalization, transient-secret revision handling, setup baselines,
provider capability exceptions and per-request stale-result rejection.

Clarifies earlier broad proposals: existing remote selection and subtree changes
lead to Test rather than automatic reauthorization; Test can verify manually
entered remotes and retains OneDrive verify-then-test. Unchanged Modify has no
settings to save. Persisted edits conservatively invalidate the test; deactivation
has a narrow enabled-only exception so connectivity failure cannot prevent
turning a saved connection off. Preview and Sync Now remain grey and use actual
saved-backend eligibility, without a new universal Preview prerequisite.

Files changed: `docs/engineering/gui-action-layout-inventory.md`,
`docs/engineering/README.md`, `Task List.md` and these notes. Source inspection
covered draft fields, SFTP loaded/applied state, validation handlers and existing
provider identity checks. Definition/status checked against the active task.
No new Rust, localization or runtime inputs: reuse the preceding successful gate
at source SHA `057b6474ce206768645c841bc4f0a1c9818640d0` (dirty worktree),
`target/task-runs/dev-gate-tl1y230f/summary.json`, and localization receipt
`target/task-runs/dev-localization-check-tcbcyjed/summary.json`; these validate
unchanged implementation, not the newly defined policy. No new policy tests are
claimed. Documentation checks are recorded below. Next: implement and test the
pure shared policy, then wire button styles/availability and inspect installed UI.

Documentation verification: scoped `git diff --check` for the engineering index
and completion notes exited 0. A focused Python structure/whitespace check of
the new contract exited 0 (all five presentations, three comparisons and async
section present; intentional metadata hard breaks retained). `sha256sum` exited
0 and confirmed unchanged `Push Buttons.md` digest
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.
Existing operator task-list content/whitespace was preserved. No installation,
service mutation, commit or publication occurred.


### 0.5.5 disabled/running color refinement — October 21, 2026

Operator clarified that Blocked/Running should retain the original color with
the reduced-contrast disabled appearance, matching locked OneDrive/Online Mount
choices. Updated the shared contract: base style and interaction state are
separate; Running retains the launch style, while Blocked uses the intended
workflow style. Blue becomes faded blue; grey/red retain their corresponding
disabled styles. Progress text and blocked reasons remain required. Use theme
rendering without hard-coded alpha/RGB values; light/dark appearance remains
installed acceptance work. No Rust or Fluent edits and no behavior changed.

Inspected `locked_select_button` in `src/app.rs`: it preserves suggested/standard
class and removes the press handler when locked. Updated files:
`docs/engineering/gui-action-layout-inventory.md`, `Task List.md`, and these
notes. Reuse unchanged source checks from `dev-gate-tl1y230f` and localization
`dev-localization-check-tcbcyjed` at source SHA
`057b6474ce206768645c841bc4f0a1c9818640d0` (dirty); these do not validate the
new design. Next: implement shared policy and style-preserving disabled controls.

Verification: scoped `git diff --check` for completion notes exited 0; focused
contract consistency/whitespace check exited 0; `sha256sum` exited 0 and confirmed
unchanged `Push Buttons.md` at
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.
Operator task-list content preserved. No code tests rerun for documentation only.


### 0.5.5 shared button colors, blocked and running states — October 21, 2026

Implemented shared editor policy in `src/app/editor_actions.rs` and wired Add,
Modify, Google OAuth, Detect and provider setup buttons through it. Suggested
blue actions and standard grey actions retain their button class while disabled;
localized progress labels replace the normal label while running. Remove retains
destructive red during removal, including remote-manager removal. Existing
softened-red Disable styling is retained and disabled during conflicting editor
work. Buttons and dispatched messages use the same guards against duplicate
operations. The remote manager's discovery action uses the same progress helper.

Policy compares normalized persisted connections and non-secret loaded/applied
setup values, keeping transient credentials out of retained setup baselines.
Persisted edits invalidate Test; unapplied server/credential edits block testing
against obsolete remote settings. Existing remote selection can proceed to Test
without Detect/Connect, including OneDrive's verify-then-test continuation. Save
requires changed/unsaved settings and a current successful Test; an enabled-only
deactivation of a saved connection retains the agreed offline exception.
Preview/Sync remain grey, use typed controller eligibility and reject unsaved
editor targets/conflicting requests. No new Preview prerequisite was added.

Async request envelopes carry operation IDs plus scoped revisions: setup changes
invalidate setup callbacks; persisted edits invalidate tests/saves. A display-name
edit during OAuth does not discard the remote authorization result, while a new
credential edit does. Matching setup success clears transient intent and
recommends Test. Stale success/failure/cancelled results cannot overwrite newer
inputs or validation. Intermediate discovery is not a Test/Save completion.
SFTP existing dirty/retry/confirmation safeguards remain, while reverting loaded
non-secret setup fields removes the pending-update recommendation.

Files changed for this step: new `src/app/editor_actions.rs`, `src/app.rs`,
`src/app/sftp.rs`, `i18n/en/cosmic_ext_applet_mounter.ftl`, `Task List.md`,
`docs/engineering/gui-action-layout-inventory.md`, `docs/engineering/README.md`,
and these notes. Operator-authored task-list sections and `Push Buttons.md` are
preserved. No installation, real provider operation, service mutation, commit or
publication was performed. Installed light/dark rendering, focus, keyboard and
scaling remain open; source/fake tests are not installed UI acceptance.

Verification chronology, source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`
with dirty worktree:

- Initial `check` failed on three private helper methods shared by the parent
  module (`dev-check-wu6jcili`); corrected visibility to `pub(super)`.
- `check` then passed (`dev-check-ceqkmwa8`); `cargo test --locked onedrive_editor`
  exited 0 with six focused tests passing.
- Shared-policy `check` passed (`dev-check-9ed14orj`), followed by ten focused
  `editor_actions` tests, exit 0. After scoped setup revisions/cancellation
  coverage, `check` passed (`dev-check-nvi0c8y_`) and thirteen focused tests passed.
- `bash scripts/dev.sh localization-check` exited 0; receipt
  `target/task-runs/dev-localization-check-fovqdmtl/summary.json`. Subsequent
  edits added no Fluent keys or app-owned text, so reuse that passing result.
- First `gate` exited 101 on three Clippy style issues: collapsible deactivation
  guard, test-module placement and test-fixture initialization (`dev-gate-0a266gl5`).
  Corrected them without suppressions. `check` passed (`dev-check-3xoeh50f`).
- Further SFTP comparison/remote-manager progress wiring: `check` passed
  (`dev-check-j_e039gk`). `cargo test --locked sftp` exited 0 with 28 tests passed
  and two live tests ignored; ignored tests are not acceptance evidence.
- Provider-switch/revert regression: `check` passed (`dev-check-qk1doaob`);
  `cargo test --locked editor_actions` exited 0 with fourteen focused tests passed.

- Second `bash scripts/dev.sh gate` exited 0; receipt
  `target/task-runs/dev-gate-c8pb43tg/summary.json`. Formatting, all-feature Clippy,
  ordinary Rust tests and repository Python suites passed.
- Final review then added two focused safeguards: retired legacy Re-Connect is
  explicitly blocked, and a stale completed Save still notifies the runtime
  owner without overwriting newer editor input. `check` exited 0; receipt
  `target/task-runs/dev-check-mglvmewh/summary.json`.

- Final focused `cargo test --locked editor_actions` exited 0 with sixteen tests
  passed, including both final-review safeguards. No ignored live tests run.

Source implementation is complete for the shared color/disabled-state task.
Verification levels established: compilation and ordinary synthetic/unit tests.
Native and Flatpak installed UI, actual theme contrast, focus/keyboard and scaling
are explicitly pending. Existing engine/service live evidence has not been
repeated for presentation-only changes. The next step is installed Add/Modify
and remote-manager inspection, including a grey Google OAuth button during work
and blue Detect/Test/Save progress, without changing real provider resources as
an ordinary automated test.

Final verification: `bash scripts/dev.sh gate` exited 0 for the final source;
receipt `target/task-runs/dev-gate-w_ou4ge_/summary.json`, log
`target/task-runs/dev-gate-w_ou4ge_/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. Reused the unchanged
passing localization receipt `dev-localization-check-fovqdmtl`. Scoped
`git diff --check` for source, Fluent, engineering index and completion notes
exited 0; explicit whitespace check for the new policy file and maintained
inventory exited 0. Operator-authored task-list whitespace remains unchanged.
`Push Buttons.md` is unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.
No remaining automated failures. Open acceptance: installed native/Flatpak
rendering and interaction; next step is installation and operator UI testing
under the repository's explicit installation approval boundary.


### 0.5.5 Google Drive browser creation repair and history — October 21–22, 2026

Operator screenshots showed Connect claiming successful Google OAuth without
opening a browser, including after logout/login. Resumed the interrupted repair
October 22. The creation command requested `--non-interactive` but discarded a
successful process's structured pending question. Removed question-return mode
from Google Create while preserving `config_is_local true`, full-drive scope,
optional redacted custom-client fields and five-minute bound. Existing Update
already uses automatic browser mode. Both Create/Update now reject incomplete,
error-bearing or truncated config output, then require a real read-only API
listing (`rclone lsf <remote>: --max-depth 1`, 20 seconds/16 KiB capture) before
success. A root listing is used rather than potentially synthesized directory
metadata. No listing names or raw OAuth/provider payloads enter new error notices.
The success notice now says creation/access were verified and directs the user
to Test the full connection settings, rather than assuming browser authorization
from process exit alone.

Failed new creation/verification cleans up only the initially absent remote;
failed cleanup is explicitly reported. Update failure does not delete an existing
remote. Already-existing user remotes, including incomplete entries from the old
flow, are preserved. Installed retest must use an unused name or operator-reviewed
unused-remote removal. Blank client fields still request rclone's shared client,
whose retirement may independently require custom OAuth credentials.

History findings are in the maintained GUI action layout inventory. Inspected
all local release-tag Google builders, earliest creation handlers, function Git
history and the published 0.5.4 remote tag. Earliest available tag `0.1` points to
`a9424fe5ea726cedfa9049881471993ac2ee4143` (June 20); it already has this flag and
unchecked output. Current-master earliest implementation `cb45c00` has the same
pattern. Every available `v0.2.0` through `v0.5.3` tag retains it. Read-only
`git ls-remote --tags origin v0.5.4 'v0.5.4^{}'` exited 0 and confirmed published
source `aa25846c6558a72a6fb1602aab83fc3997495e91`; that source retains the defect.
No later introducing release or tagged known-good creation baseline was found.
This dates the flawed handling, not the first failing installed/rclone-version
combination. Existing working Google mounts may use earlier stored credentials;
no user credential files or historical binaries were inspected/exercised.

Source audit also found the same non-interactive/unchecked-output pattern in Box
Create. Recorded a separate follow-up and disposable installed creation matrix
for all supported provider/mode families; no Box repair or installed acceptance
is claimed in this Google task. Native and Flatpak browser checks remain separate.
Engine behavior was checked against official rclone config create/update docs;
references are retained in the maintained inventory.

Files changed: `src/app.rs`, `i18n/en/cosmic_ext_applet_mounter.ftl`,
`docs/engineering/gui-action-layout-inventory.md`,
`docs/engineering/flatpak-host-command-inventory.md`,
`docs/engineering/README.md`, `Task List.md`, and these notes. Existing engine
transport/permissions are reused; host-command inventory documents the call's
owner and bounds. No scripts or package permissions changed. User-authored task
text outside scoped status/progress and new follow-ups was preserved.

Verification, source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`, dirty worktree:

- Initial interrupted repair: `check` exited 101 (`dev-check-wpsk9ar2`) on a
  builder-chain Result typo; corrected it. `check` then exited 0
  (`dev-check-nik4n8q8`), and four focused Google tests passed, exit 0.
- Completed regression/rollback/access repair: `just fmt` exited 0;
  `bash scripts/dev.sh check` exited 0, receipt
  `target/task-runs/dev-check-qzkz1xxv/summary.json`.
- `cargo test --locked google_drive` exited 0, eight focused tests passed.
  Covers shared/custom client argument construction, duplicate/missing inputs,
  unfinished OAuth, structured errors, access failure, secret-safe diagnostics,
  cancellation, failed rollback, and preservation of existing remotes.
- `bash scripts/dev.sh localization-check` exited 0; receipt
  `target/task-runs/dev-localization-check-pctda053/summary.json`.
- `bash scripts/dev.sh gate` exited 0; receipt
  `target/task-runs/dev-gate-fneitvyt/summary.json`, log
  `target/task-runs/dev-gate-fneitvyt/task.log`. Formatting, all-feature Clippy,
  ordinary Rust tests and repository Python suites passed. Ignored live tests
  remain unrun. Authorized socket/runtime-directory access used for the gate.
- Scoped `git diff --check` for source, Fluent and engineering index/host-command
  record exited 0. Additional record checks follow this entry. `Push Buttons.md`
  remains unchanged at SHA-256
  `0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Levels established: compilation and synthetic/fake-engine tests, not real Google
browser authorization. No installation, credential/data deletion, service changes,
commit, tag or publication was executed. Next: install the repaired source with
`just install-user`, close old editors/settings and reload the applet; operator
then creates a fresh disposable Google remote and completes browser login before
Test/Save. Actual browser opening/authentication remains pending until observed.

Final record verification: scoped `git diff --check` including completion notes
exited 0. Explicit maintained-inventory structure/whitespace assertions exited 0.
Read `install-user` recipe and build guide for the operator handoff: it rebuilds
release source and replaces the user binary/catalog/desktop metadata/icons; no
installation was run in this task. No remaining automated failures.


### 0.5.5 Box browser authentication repair — October 22, 2026

Repaired the shared Box Online Mount/Offline Mirror creation flow. Removed
`--non-interactive` while retaining `config_is_local true` and the five-minute
OAuth deadline. Extracted the existing Google completion/error/access logic into
shared typed `BrowserOAuthProvider::{GoogleDrive, Box}` helpers. Both providers
now reject pending/error/truncated config responses and require successful
bounded read-only API access (`lsf <remote>: --max-depth 1`, 20 seconds, 16 KiB)
before a success notice. Box notices now say creation/access were verified;
full connection/folder validation remains in Test. Nonzero error payloads are
not exposed, preserving provider/stage/exit-code diagnostics without credentials.

Existing remote names, including malformed sections, are protected before
mutation. Failure/cancellation/timeout rolls back only the newly attempted,
initially absent name, and failed rollback is reported. Existing Box selection
still proceeds to Test, with ordinary Modify Re-Connect deliberately unavailable
until its separate backend exists. This task repairs creation in both modes.
No real provider authentication, installation, service or user credential/data
mutation was performed. Native/Flatpak browser login and subsequent mount/mirror
acceptance remain pending in the installed creation matrix.

Files changed: `src/app.rs`, `i18n/en/cosmic_ext_applet_mounter.ftl`,
`Task List.md`, `docs/engineering/gui-action-layout-inventory.md`,
`docs/engineering/flatpak-host-command-inventory.md`, `docs/engineering/README.md`,
and these notes. Reused existing engine transport/permissions; the host-command
inventory records Box's use of the shared flow. Preserved the operator's latest
Test New Connections section and other uncommitted work. `Push Buttons.md` is
unchanged at SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification at source SHA `057b6474ce206768645c841bc4f0a1c9818640d0` (dirty):

- `just fmt` — exit 0.
- Initial shared-helper `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-iyoykwgq/summary.json`. Existing three focused Box
  tests passed, exit 0.
- Completed regressions: `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-_azwhbks/summary.json`.
- `cargo test --locked box_remote` — exit 0, six focused tests passed. Includes
  both mode builders, duplicate/malformed existing-section guards, access-probe
  bounds/empty-root success, pending/error/truncated responses, raw-output
  redaction, access failure, timeout/cancellation and failed rollback.
- `cargo test --locked google_drive` — exit 0, eight focused tests passed after
  the shared-helper extraction, preserving the preceding Google repair.
- `bash scripts/dev.sh localization-check` — exit 0; receipt
  `target/task-runs/dev-localization-check-_kcky_4d/summary.json`.
- Scoped `git diff --check` for Rust, Fluent, engineering index and host-command
  inventory — exit 0. Final gate/record checks follow below.

Evidence levels: compilation and synthetic/fake-engine tests. Installed browser
opening and consent are not claimed. Official rclone Box/config-create semantics
were checked; references are retained in the maintained GUI inventory. Next:
install the updated source, close old Settings/editors, reload the applet, and
use a fresh disposable Box remote for browser authorization followed by Test/Save.

Final `bash scripts/dev.sh gate` exited 0; receipt
`target/task-runs/dev-gate-n01zz3ts/summary.json`, log
`target/task-runs/dev-gate-n01zz3ts/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. Ignored live tests
remain unrun. Gate used authorized runtime-directory/disposable socket access.
Final scoped `git diff --check` including completion notes exited 0, and explicit
maintained inventory/task structure/whitespace checks exited 0. No remaining
automated failures. `Push Buttons.md` remains unchanged at the recorded digest;
operator's Test New Connections section was retained. No installation, live Box
account operation, commit or publication occurred. Next is installed native/
Flatpak browser and connection creation acceptance.


## SharePoint whole-library Offline mirror — October 22, 2026

Request: accept the operator's explicit whole-library choice when the optional
folder is empty. Earlier Re-Test failed because both planner and guard required
a subfolder. Implemented absent subpath as the verified library root; explicit
malformed paths remain rejected. Kept serialized bindings compatible, with an
empty string denoting root. Existing identity, saved-target, local/recovery,
access-marker and post-sync checksum guards are retained. Root recovery is
inside the selected library and excluded by the existing filter. Retargeting
between root and subfolder gives separate state and invalidates old confirmation
markers while retaining listings. Scheduling remains unchanged and disabled.

Files changed for this scope: `src/sharepoint.rs`, `src/sync.rs`, `src/config.rs`,
`src/app.rs`, `src/app/editor_actions.rs`, the English Fluent catalog,
`Requirements and Specifications.md`, GUI action layout inventory and engineering
index, task list and these completion notes. Existing operator notes and other
uncommitted work were retained. `Push Buttons.md` remains unchanged, SHA-256
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification at source SHA `057b6474ce206768645c841bc4f0a1c9818640d0` (dirty):

- `just fmt` — exit 0.
- `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-4l4ar03h/summary.json`.
- `cargo test --locked sharepoint` — exit 0; 25 passed, three live tests ignored.
  Covers root/subfolder state binding, serialized guards, rejection of stale
  bindings before remote I/O, two-way confirmation invalidation and editor
  Re-Test/Save gating. Existing identity/setup tests were reused.
- `cargo test --locked teams` — exit 0; 14 passed. Includes whole-library
  configuration, recovery path/filter and guarded planner regressions.
- `bash scripts/dev.sh localization-check` — exit 0; receipt
  `target/task-runs/dev-localization-check-c9_z2d_c/summary.json`.
- Initial final `bash scripts/dev.sh gate` — exit 101; receipt
  `target/task-runs/dev-gate-7ed84wzw/summary.json`. Formatting and lint passed;
  Rust suite hit `private_config_failure_hides_both_output_streams`, expecting
  exit 7. No SharePoint failure. The exact process-error variant was not printed
  by its existing assertion, so the transient cause is unconfirmed.
- `cargo test --locked private_config_failure_hides_both_output_streams` —
  exit 0 immediately afterwards, one passed, no source changes to that test.
  Full gate rerun follows to resolve the failed broad-check result.
- Scoped `git diff --check` on tracked source, Fluent, specification and
  engineering index — exit 0. Including `Task List.md` reports four pre-existing
  operator whitespace lines (178, 307–309); preserved rather than reformatted.

Evidence level: compilation and synthetic/unit regressions only. No installed
whole-library Preview, real SharePoint data operation, installation, service
mutation, commit or publication performed. Native/Flatpak installed acceptance
remains open. Next: install/reload, leave the folder empty, Re-Test then Save;
review a fresh Preview of the new target before confirming its initial Sync.

Final rerun `bash scripts/dev.sh gate` — exit 0; receipt
`target/task-runs/dev-gate-rq4b4bvr/summary.json`, log
`target/task-runs/dev-gate-rq4b4bvr/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. The earlier process
test failure did not recur; its cause remains unconfirmed. Gate used authorized
runtime-directory and disposable D-Bus socket access. Live tests remain unrun.
Final scoped whitespace checks passed; preserved operator Task List whitespace
is the only recorded diff-check exception. No remaining automated failures.


## Missing runtime owner and unapplied SharePoint edit — October 22, 2026

Read-only installed observations: Runtime D-Bus name absent despite a running
panel process; Settings name present. Exact panel/Settings process filtering
confirmed their roles. Running panel SHA-256 matched the installed executable:
`3084981f635fcb44c11d898c4d92731461b607c23024e96fd82ac65479aa91a9`.
No matching communication/panic journal records explained the initial loss.
Saved native configuration had no `archive` text/subpath; modification time was
October 22 at 04:08:27 America/Phoenix. Original Save notice was unavailable;
operator later confirmed the old entry remained and offered a retry. Do not
claim that IPC loss prevented persistence: notification follows the write.

Implemented queued D-Bus ownership with acquisition/loss signals, no owner
replacement, and bounded bus-error retries (five attempts, two-second interval).
Owner acquisition asynchronously reloads saved settings; owner/error state and
late reload guards prevent stale ownership. Extracted the persistence boundary
to inject temporary storage/configuration into a regression without executing
service tasks. See maintained GUI inventory for the contract and limits.

Files: `src/runtime_ipc.rs`, `src/app.rs`, `Cargo.toml`, `Cargo.lock` (existing
transitive enumflags2 now direct), English Fluent catalog, GUI inventory/index,
task list and these notes. Existing user edits and Push Buttons preserved.

Verification at dirty source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`:

- `just fmt` — exit 0 after each Rust edit group.
- Initial `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-t5mzhuj9/summary.json`.
- Initial `cargo test --locked runtime_ipc` — exit 0, two existing tests passed.
- Handoff `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-3qtu4z7w/summary.json`.
- Handoff `cargo test --locked runtime_ipc` — exit 0, three tests passed,
  including production queued-ownership handling on a private disposable bus.
- Completed `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-b1f57r3_/summary.json`.
- `cargo test --locked --bin cosmic-ext-applet-mounter runtime_` — exit 0,
  sixteen focused tests passed, including ownership-loss/reload and existing
  runtime, service-rollback and stale-Save-notification regressions.
- `cargo test --locked --bin cosmic-ext-applet-mounter sharepoint_save_persists_retargeted_folder_name_and_directories`
  — exit 0, one passed. Renamed folder/local/recovery settings roundtrip exactly
  through temporary storage; returned installation tasks are not executed.
- `bash scripts/dev.sh localization-check` — exit 0;
  `target/task-runs/dev-localization-check-lkrze59q/summary.json`.

Compilation, synthetic storage/model tests and private-bus integration are
verified; installed recovery and real SharePoint Save validation are pending.
No installation, process termination, user config write, service change, commit
or publication performed. Original Save failure remains unconfirmed until the
operator repeats it and retains the editor notice. Final gate follows.

Final `bash scripts/dev.sh gate` — exit 0; receipt
`target/task-runs/dev-gate-drchpgre/summary.json`, log
`target/task-runs/dev-gate-drchpgre/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. No automated failure
in this task. Scoped source/lock/catalog/record `git diff --check` passed; existing
operator Task List whitespace remains preserved. Push Buttons SHA-256 remains
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.
Next: operator retry retaining the Save notice, install/reload the communication
repair and verify the Runtime name, Settings status and persisted connection
changes separately. Initial installed owner-loss cause and original failed Save
remain undetermined; neither is presented as reproduced by the isolated tests.


## Confirm Save for remote reuse — October 22, 2026

Operator clarification resolved the earlier unapplied SharePoint edit: the first
Save requested shared-remote confirmation; the second displayed successful Save
and managed mirror units. Operator then reported completing Preview and initial
Sync for the selected `archive` folder. Retain this installed/user-reported
acceptance; no whole-library, independent checksum, or communication-recovery
claim is inferred. The missing Runtime name remains a separate acceptance item.

Implemented explicit **Confirm Save** in the shared Add/Modify button-label
policy when the Save-time remote-reuse guard awaits its second click. Existing
blue styling and enabled policy remain. Running Save still shows **Saving…**
with the existing disabled/faded styling. Confirmation notice now names the
button exactly. Consume acknowledgement before validation/persistence, and clear
it for every draft-edit message and cancellation; successful completion and
editor reload retain existing clearing. Unshared connections keep **Save**.
No provider-specific label logic or new engine/service operation was added.

Files changed for this task: `src/app.rs`, `src/app/editor_actions.rs`, English
Fluent catalog, task list, GUI action layout inventory/index and these notes.
Preserved all unrelated work and the operator's task-list section. Push Buttons
SHA-256 unchanged:
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification at dirty source SHA `057b6474ce206768645c841bc4f0a1c9818640d0`:

- `just fmt` — exit 0 for both source-edit groups.
- Initial `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-fupflm9x/summary.json`.
- Initial focused `cargo test --locked --bin cosmic-ext-applet-mounter editor_actions_`
  — exit 101, 19 passed and one failed at SMB/Online/Modify: the synthetic editor
  lacked the already-loaded remote setup baseline. Corrected that fixture to
  represent loaded SMB/SFTP settings; no product change was needed for this failure.
- Completed `bash scripts/dev.sh check` — exit 0; receipt
  `target/task-runs/dev-check-mlwat1ic/summary.json`.
- Focused `cargo test --locked --bin cosmic-ext-applet-mounter editor_actions_`
  — exit 0, all 20 passed. New coverage includes 24 provider/mode/editor
  combinations (OneDrive, SharePoint, Google, Box, SMB, SFTP; Online/Offline;
  Add/Modify), blue/enabled confirmation, running precedence, confirmation
  consumption, successful reset, no-reuse behavior, draft-edit/cancellation reset
  and first-Save dispatch stopping before persistence. Existing retired-engine,
  stale-result and unsaved-runtime guards were reused.
- `bash scripts/dev.sh localization-check` — exit 0; receipt
  `target/task-runs/dev-localization-check-pyzb816n/summary.json`. Catalog inputs
  unchanged after this receipt; fixture-only correction did not change UI text.
- Scoped tracked-source/catalog/index `git diff --check` — exit 0.

No installation, service/data operation, commit or publication. Compilation and
synthetic state transitions are verified; installed Confirm Save rendering is
pending. Existing completed Preview/Sync acceptance is reused. Final gate follows.

Final `bash scripts/dev.sh gate` — exit 0; receipt
`target/task-runs/dev-gate-eb_x7tgj/summary.json`, log
`target/task-runs/dev-gate-eb_x7tgj/task.log`. Formatting, all-feature Clippy,
ordinary Rust tests and repository Python suites passed. No remaining automated
failure. Final scoped tracked-source/catalog/index/notes whitespace checks and
explicit added-file whitespace checks passed. Existing operator Task List
whitespace was preserved. No native or Flatpak installation performed; next is
install/reload and visually confirm Save → Confirm Save → Saving… behavior.
Unchanged completed Preview/Sync tests do not need repetition for this label task.


## Google test mount read-only inspection — October 22, 2026

Operator asked whether the Google test Online mount was active and updating its
mountpoint. Selected only the saved test entry. Read-only service inspection
reported loaded/active/running, successful result, zero restarts, active since
05:04:39 America/Phoenix. `findmnt --mountpoint <home>/Cloud/test1 --json` showed
an rw `fuse.rclone` mount of the configured test remote at that exact target.
These installed observations are not inferred from the configuration's enabled
flag. Service/mount inspection command exited 0 with authorized session access.

A fresh root `rclone lsjson` used max depth 1, engine max duration 20 seconds,
25-second outer deadline, 256-KiB output cap and kill/reap on failure. Mount root
inspection used a separate 15-second child. Both listings succeeded: 109 remote
and 109 mount entries; zero names missing in either direction; zero type/size
mismatches among common entries. Native Google-document size comparisons were
excluded where applicable, and nested content/byte checksums were not compared.
No actual filenames, tokens or account organization names were recorded.

The initial combined command exited 1 only because `rclone help flags` did not
include mount-specific poll flags; its preceding comparison succeeded. Corrected
read-only `rclone mount --help` inspection exited 0 and reports default polling
at one minute and write-back five seconds. Running process flags explicitly use
five-minute directory cache and full VFS cache, without overriding poll/write-back.
No new remote change was introduced, so root parity is verified, while propagation
latency, upload/write-back completion and nested-file freshness are not proven.

No source changes, writes/uploads, mount/service mutations or installation.
Changed only task status and these notes. Reused unchanged source check/gate
receipt `target/task-runs/dev-gate-eb_x7tgj/summary.json` at dirty source SHA
`057b6474ce206768645c841bc4f0a1c9818640d0`; no reason to rerun compilation/tests
for this inspection. Full creation/Flatpak and change-event acceptance remain
open. Next if needed: operator makes a disposable remote edit to observe its
appearance without manually refreshing the mount.


## Google Drive slow-listing investigation — October 22, 2026

Operator reports root not populating after 60-second preload. Read only the
current Google/preload task sections, mount planner and refresh lifecycle,
installed Google preload policy, and matching prior frozen 0.5.3 evidence.
The policy is enabled, maximum_seconds 60 and unlimited depth. Installed rclone
reports v1.75.0. Planner uses full VFS file cache, five-minute directory cache,
10-second I/O timeout, five-second connection timeout and no --fast-list.
Current refresh requests the entire tree with unsupported `fast_list=true`.
Upstream tagged RC/VFS source confirms the incompatibility and all-tree cache
installation behavior. References/interpretation correction are in GUI inventory.

Read-only native service inspection found inactive/MainPID zero, unlike the
preceding active-mount observation. Combined diagnostic exited 1 when its process
inspection tried /proc/0; no mount probe, cache change or timing was performed.
A separate authorized bounded journal read exited 0. The inspected two-hour
window/sample had repeated Google 403 rateLimitExceeded/RATE_LIMIT_EXCEEDED,
with 51 occurrences of each and Requests per minute text. Counts can duplicate
one API failure across log messages. No raw filenames/account/project identifiers
or credential values were printed or retained. A subsequent narrow reason-code
classification exited 0; no daily-limit, storage-quota, 429 or timeout signal was
found in the sample. Test remote custom client ID and secret presence booleans
were both false. Do not treat this as proof of quota allocation or unique calls.

Diagnosis: observed API throttling plus a source preload API bug; root-first
warming and cache retention also warrant repair/evaluation. A longer preload
limit alone is not an adequate fix. Historical 0.5.3 fast-list claims remain
unchanged, with the new compatibility correction in the maintained record.
No code, credentials, user config, services or mounts changed. Current task list
tracks the actual RC/root-first repair and quota-conscious live follow-up.
Only task/evidence/engineering record docs changed; unchanged source checks reuse
`target/task-runs/dev-gate-eb_x7tgj/summary.json` at dirty SHA
`057b6474ce206768645c841bc4f0a1c9818640d0`. No additional compile/test gate warranted.


## Google fast-list/VFS correction and root-first preload — October 22, 2026

Scope authorized: correct fast-list selection, populate root first, assess
Google depth limits. Later operator steering: visibly recommend a private Google
OAuth client rather than designing around shared-client rate limits; report all
Google mount options once implemented. No private client or account was created
or changed. Rate/pacer/retry/directory-cache settings remain unchanged.

**Correction to earlier diagnosis:** real local `rclone rc --loopback
options/local fast_list=true` returned UseListR=true, as did an explicit
_config UseListR override. Both commands and the default false comparison exited
0. The 1.75.0 RC dispatcher accepts/consumes flat global options; looking only at
vfs/rc.go caused the earlier incorrect rejection claim. The actual VFS keeps its
creation context, so the call-local override does not change readDirTree's config.
Tagged 1.74.3/1.75.0 VFS creation and directory code and RC docs were checked.
Chronology and corrected authoritative contract are retained in GUI inventory.

Implemented Google-only --fast-list in generated mount services. Replaced the
single refresh with root-only asynchronous refresh followed by recursive refresh
only on verified success. One deadline starts at root dispatch; root/recursive
jobs and delayed monitoring share that budget. UI identity remains the initial
job while cancellation targets the current root/recursive job. Updated requirements
FR-044A/B, their command example and automated acceptance, product description,
Google preload help and Flatpak host-command inventory (no new permission).

Visible private-client recommendation uses the shared Google Add/Modify view for
Online and Offline. Updated creation help recommends a matching client pair;
blank compatibility and existing credentials remain unchanged. No extra test
was invented for static wording; compilation/localization checks cover it, and
eight existing OAuth regressions were reused. Real provider/UI acceptance remains
separate. All explicit Google mount options are listed in GUI inventory for the
operator's requested report.

Depth assessment: applicable and worth considering for browsing scope. Existing
Google policy intentionally disallows Some(depth) and offers only a duration,
because its recursive RC path has no depth control. A proper root-only/shallow/
whole-tree choice requires app-managed bounded traversal, not an ignored RC
parameter or request-local MaxDepth. Recorded as follow-up without silently
adding a nonfunctional field or changing current full-tree behavior. Operator
specifically rejects redesign around shared-client quota; no pacing tuning added.

Files changed in this task: `src/providers.rs`, `src/rclone_refresh.rs`, shared
Google view in `src/app.rs`, English Fluent catalog, product description,
requirements, task list, GUI inventory/index, Flatpak host-command inventory and
these notes. Existing uncommitted work/user Task List content retained.
Push Buttons SHA-256 unchanged:
`0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f`.

Verification at dirty SHA `057b6474ce206768645c841bc4f0a1c9818640d0`:

- `just fmt` — exit 0 after each Rust edit group.
- Initial `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-aqvlt8nd/summary.json`.
- Initial `cargo test --locked rclone_refresh` — exit 0, six passed.
- Extended `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-ry0glgmo/summary.json`.
- Extended focused refresh tests — exit 0, nine passed including the existing
  app stale-result test; the real-engine test is ignored in ordinary runs.
- First explicit real-rclone local integration — exit 101: expected one root
  metadata file, observed two. Upstream VFS stats counts directory entries in
  metadataCache/files; corrected fixture expectation to two root entries and
  five full-tree entries. Product root-first behavior was not the failure.
- First provider suite — exit 101: an old test asserted the obsolete no-fast-list
  contract. Replaced that expectation and reused its provider matrix, removing
  the redundant newly added test.
- Corrected `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-0dgn7bub/summary.json`.
- `cargo test --locked --lib rclone_refresh` — exit 0, eight passed/one ignored.
- `cargo test --locked --lib providers::tests` — exit 0, eighteen passed.
- `cargo test --locked --lib rclone_refresh::tests::real_rclone_root_first_refresh_uses_mount_config -- --ignored --exact`
  — exit 0, one passed. Real installed rclone 1.75.0 served only temporary local
  synthetic directories, read-only, using ephemeral localhost/private Unix
  sockets, an empty config and isolated cache. Verified main.UseListR=true, root
  cache entries before recursive launch and all descendants afterward. Child
  is killed/reaped by a test guard; no FUSE mount, cloud token or user service used.
- Pre-steering `bash scripts/dev.sh gate` — exit 0;
  `target/task-runs/dev-gate-jib50uv9/summary.json`. Later visible-OAuth view/text
  addition requires a final new gate; earlier unchanged engine tests reused.
- Pre-steering localization — exit 0;
  `target/task-runs/dev-localization-check-8hpiobod/summary.json`.
- Visible-OAuth `bash scripts/dev.sh check` — exit 0;
  `target/task-runs/dev-check-11yfvpvy/summary.json`.
- `cargo test --locked --bin cosmic-ext-applet-mounter google_drive` — exit 0,
  eight existing Google creation/update/cancellation/redaction tests passed.
- Final text `bash scripts/dev.sh localization-check` — exit 0;
  `target/task-runs/dev-localization-check-7jpc5ryt/summary.json`.
- Scoped source/catalog/specification/description/index/host-inventory
  `git diff --check` — exit 0. Final gate/record check follow below.

Evidence: source compiles, fake transitions pass, actual local-engine integration
passes. Installed Google cold-cache improvement and final Flatpak are untested.
Existing observed quota errors remain independent of these repairs. Upstream
job/stop cannot prove immediate cancellation of VFS methods using retained VFS
context; monitor deadlines/stop requests are verified, not termination of every
underlying network request. No forced mount stop introduced. No installation,
real data writes, service change, commit, push or publication. Next: install/reload
and remount Google to regenerate the service with --fast-list; use a private
OAuth client for intended ongoing acceptance. Final gate receipt follows.

Final `bash scripts/dev.sh gate` — exit 0; receipt
`target/task-runs/dev-gate-d4fz0g1o/summary.json`, log
`target/task-runs/dev-gate-d4fz0g1o/task.log`. Includes the later operator-requested
visible private-OAuth recommendation. Formatting, all-feature Clippy, ordinary
Rust tests and repository Python suites passed. No remaining automated failure.
Final scoped diff/record whitespace checks passed; existing user Task List
whitespace remained untouched. Source scope complete; installed native Google
and Flatpak behavior still pending, with upstream stop-request limit explicit.
The exact generated Google command is retained in the maintained GUI inventory;
only --fast-list was newly enabled. Prior quota findings are not used to justify
new rate-limit workarounds, per the operator's steering.


## Installed Google browsing acceptance and remaining-scope review — October 22

Operator reported after installing the repair that root entries became visible
within 10–20 seconds, whereas the earlier mount appeared empty even after preload
finished. Operator subsequently reported populated directories through three
levels. Marked this scoped installed/native browsing improvement complete in the
active GUI section. Do not infer a configured depth-three limit, whole-drive
completion, nested content checksums, quota resolution, private-client setup or
Flatpak acceptance. No new cloud write, service operation or test was run here.

Reviewed active 0.5.5 removal, GUI and submission sections and their explicitly
deferred validation boundary. Legacy source/native retirement is accepted; package
upgrade acceptance remains open. GUI source changes and automated checks are
complete; installed layouts/focus/keyboard/scaling, explicit communication-owner
recovery and provider creation matrix remain open. SharePoint archive-subfolder
Save/Preview/initial Sync is accepted by preceding operator evidence; whole-library
root installed acceptance is not inferred. Private Google OAuth setup, Box real
browser authorization and remaining provider/mode creation coverage need their
own evidence. Existing Connections is the operator's inventory, not proof of
new-creation acceptance.

Next work is focused installed GUI/provider checks, reconcile final release docs
with accepted 0.5.5 behavior, package-specific upgrade/native/Flatpak acceptance
and final manifest/source/permissions review, then operator-authored COSMIC
submission after current contribution-rule review. Google depth/scope expansion
and the large-drive/extended VPN/sleep backlog stay deferred unless promoted or
a relevant defect makes them necessary. No source tests repeated; reuse passing
`target/task-runs/dev-gate-d4fz0g1o/summary.json` at dirty SHA
`057b6474ce206768645c841bc4f0a1c9818640d0` within its original source scope.
Only task list and these evidence notes were updated; user inventory and unrelated
changes were preserved. Scoped completion-note `git diff --check` exited 0.
Push Buttons digest is unchanged at its previously recorded SHA-256.


## Installed SharePoint Save-button transition — October 22, 2026

Operator changed a SharePoint connection name, ran Re-Test and Save, and observed
Save → Confirm Save → greyed/disabled Saving. Record that native Modify label and
disabled-state sequence as accepted; no independent screenshot/color rendering
inspection was performed in this turn. Do not infer base-color appearance across
themes, Add behavior, every provider, confirmation invalidation after a further
edit, failure/retry, destructive actions or keyboard/scaling acceptance.

Remaining focused button checks: Add creation/Test/Save ordering; relevant edits
invalidating Test/Save and pending confirmation; blue and grey running actions
rejecting repeated presses; failed Test keeping Save blocked and retry available;
Preview/Sync blocked on unsaved target changes; red Disable/Remove confirmations
and used-remote protection on disposable resources; keyboard/light-dark/scaling.
Reuse shared policy's 24 provider/mode/editor combinations and existing completed
SharePoint Preview/Sync and Google browsing tests; no reason for repetitive full
matrix live tests solely for this label. Prefer the pending Box creation task for
the next Add/browser button check, and private Google OAuth for a neutral-running
check when performing that real setup, not merely to exercise a color.

Only task/evidence docs changed. No installation, service or data mutation,
source edit or repeated source tests. Latest source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json` remains applicable. Scoped notes
whitespace check is recorded by the command accompanying this update.


## Installed Google private OAuth update and Re-Test — October 22, 2026

Operator supplied the private Google OAuth client ID/secret and provided an
installed Modify screenshot: Google Drive Online, Draft test passed, managed
unit structural validation passed, backend `drive` matched and root access
reported 121 visible entries at depth one. Read-only local config inspection
confirmed the test remote's drive backend and nonempty private client-ID and
secret fields. Only presence booleans were printed; no credential/token value,
raw configuration, organization name or cloud filename was retained. Relative
to the earlier inspection, both private-client presence flags changed from
false to true. No fresh authentication operation or additional remote read was
performed by the agent.

Mark the scoped native existing-remote private OAuth update plus successful
Online Re-Test accepted. The visible private-client recommendation is rendered
in the screenshot. Save appears neutral after a passing test of unchanged applet
connection settings; credential update belongs in rclone config and is not
itself an unsaved connection-field change. Do not infer observed browser-consent
UI, token freshness inspection, upload/write-back, removal of quota errors,
grey running-button appearance, Offline creation or Flatpak acceptance. The
screenshot shows the completed test state, not an in-flight OAuth operation.

Task list and these evidence notes updated only. No source/service/config/data
mutation, credential change, installation, commit or repeated automated tests.
Reuse source gate `target/task-runs/dev-gate-d4fz0g1o/summary.json` at dirty SHA
`057b6474ce206768645c841bc4f0a1c9818640d0`; no changed source inputs. Scoped
completion-note whitespace validation accompanies this update. Other carried
forward authentication items retain their historical scope; this new acceptance
is explicitly limited to the tested native existing remote.


## Installed button checks: confirmation reset and unchanged draft — October 22

Operator reports Google test and personal OneDrive name edits save without a
second confirmation. The guard is conditional on another saved connection using
the same provider/remote; ordinary Save is expected where no reuse is detected.
Operator reports SharePoint name edit, Re-Test and first Save require Confirm
Save, and another name edit clears that confirmation and restores Re-Test.
After reverting to the original saved name and passing Re-Test, Save remains
unavailable. This is the expected unchanged-draft gate, provided the draft again
matches the current saved connection. Mark these native transition observations
accepted under the shared confirmation task; no all-provider/theme claim.

Next guided check is a deliberately missing subfolder on the Google test
connection: test only, observe running/disabled state and failed-test Save gate,
then restore the original folder field without saving. No actual failure, retry
or running appearance is marked passed before operator evidence arrives.
Only task/evidence docs changed. Reuse unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; no source tests, installation,
service/config/remote mutation or credentials inspection performed this turn.
Scoped completion-note whitespace check accompanies this update.


## Installed Google failed-test button sequence — October 22, 2026

After the guided missing-subfolder Re-Test, operator reported “Worked as expected.”
Accept the scoped native sequence: running Testing… is faded/disabled, failure
is reported, Save remains blocked, Re-Test returns blue/available, and the
original folder can be restored without saving the invalid draft. The guided
sequence did not include clicking Re-Test a second time after failure, so repeat
execution itself is not claimed; retry availability is observed. No remote write,
file-content test, credentials change or package acceptance is inferred.

Next guided check: temporary name edit in SharePoint mirror blocks Preview and
Sync Now; reverting the draft restores their prior backend eligibility. Pending
until operator reports it. Only task/notes changed; unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json` reused. No tests or service actions
run by the agent. Scoped notes whitespace validation accompanies this update.


## Installed unsaved-mirror action guard — October 22, 2026

Operator reported “Worked as expected” for the guided SharePoint mirror name
edit: unsaved changes fade/disable Preview and Sync Now, blocked clicks do not
start them, and restoring the saved name returns prior availability. Mark this
native saved-target action guard accepted. No new Preview or Sync execution,
file transfer or mutation is inferred or required.

Next guided check is reversible draft-only Disable/Enable on the Google test
connection. Source confirms red Disable sets DraftEnabled(false), shows blue
Enable and makes disable-only Save available without a new Test. Clicking Enable
again restores the unchanged saved draft and disables Save. Do not Save during
this first presentation check. No persistence/unmount is claimed from a draft
toggle. Agent changed task/evidence docs only; source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json` reused without rerunning tests.
Scoped completion-note whitespace validation accompanies this update.


## Installed draft Disable/Enable buttons — October 22, 2026

Operator reported “worked as expected” for Google test draft-only Disable/Enable:
initial unchanged Save disabled; red Disable changes to blue Enable and Save
becomes available without Re-Test; clicking Enable restores red Disable and
unchanged Save blocking. No Save was requested in the guided sequence, so this
accepts labels/colors and draft state only, not persisted disabling, unmount or
subsequent re-enabling of services.

Next guided check is first-click Remove confirmation followed by editor close/
reopen without confirming deletion. Source remove_connection_with_confirmation
returns immediately on first click after setting pending_remove/notice. Actual
removal and file/service preservation remain outside that first-click check.
Only task/notes edited; no service/data operation or tests run. Reuse source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; scoped notes whitespace check
accompanies this update.


## Installed Remove confirmation cancelled on close — October 22, 2026

Operator reported “Worked as expected” for first-click Remove on the Google test
connection: confirmation notice and red Confirm Remove appeared; connection
remained; closing without confirmation and reopening restored Remove with the
connection unchanged. Accept native confirmation presentation and cancel-on-close.
Actual deletion, in-flight Removing appearance and backend preservation on
confirmed removal are not claimed by this check.

Next guided check is Settings Remove Remote window: a saved reference blocks
removal with disabled In use, unused actions are red, descriptions/actions align
in their fixed columns. No remote deletion requested. Only task/evidence docs
changed; no source tests or service/data operations performed. Reuse source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; scoped notes whitespace check
accompanies this update.


## Installed remote manager protection and alignment — October 22, 2026

Operator reports all remaining entries show In use; the controls align and are
not clickable. Mark native used-remote presentation, fixed action-column
alignment and disabled interaction accepted. Do not infer a backend deletion
attempt or independent remote-reference audit from this UI check. Operator also
reported earlier first-click Confirm Remove and disappearance after confirmation
for an unused entry. Whether that earlier removal occurred in remote manager or
the Connections list remains unspecified; preserve the report without marking
remote-specific deletion acceptance or data-preservation proof. No new unused
remote is needed solely to repeat a removal-label test.

Next guided check is keyboard traversal and activation of an available Re-Test
on the Google test connection, without saving/removing anything. Pending until
reported. Only task/evidence docs changed. Reuse unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; no source tests, data/service
operations or credential access. Scoped notes whitespace validation accompanies
this update.


## Installed keyboard activation: Return, not Space — October 22, 2026

Operator reports Space did not activate focused Re-Test, but Return did. Verified
pinned libcosmic a991d7bb433f67016c1473e8e4a03691ededd75a in Cargo.lock and its
local source: src/widget/button/widget.rs around 839 checks focused state and
Named::Enter when an on_press callback exists. The pinned iced button handler
also checks Enter. Space activation is not implemented in those handlers.
Therefore the previous guided Space expectation was incorrect for this framework;
this observation is not evidence of a Cloud Mounter regression. Mark native
Return activation accepted; Tab/Shift+Tab ordering, visible focus, disabled-key
handling and themes/scaling are not inferred from the narrower response.

No app/framework code changed, dependency update or new key binding introduced.
Only task/evidence docs edited. Reuse unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; no tests, service/data operation
or installation. Scoped notes whitespace validation accompanies this update.


## Installed keyboard traversal and focus — October 22, 2026

In response to the guided Tab/Shift+Tab ordering and visible-focus question,
operator answered that Tab and Shift+Tab moved as expected. Accept the native
forward/backward navigation and visible-focus check within this guided editor
scope, alongside the preceding Return activation result. This is operator
acceptance, not independent screen-reader or exhaustive accessibility proof.

Next guided observation: button roles, readable disabled labels and focus in
light and dark themes at usual scaling; no backend action needed. Theme/scaling
remain open pending report. Only task/evidence docs changed; source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json` reused. No source tests or
service/data operations performed. Scoped notes whitespace check accompanies
this update.


## Installed light/dark button appearance — October 22, 2026

Operator reports “Dark and Light theme works” after the guided editor check of
blue/grey/red roles, disabled-label readability, visible keyboard focus and
unclipped layout at usual scaling. Accept that native light/dark observation;
no independent screenshot, contrast measurement, alternate scaling/monitor,
all-window or Flatpak acceptance is inferred. Existing Modify/remote-manager
state checks remain accepted within their preceding scope.

Next pending flow is Add Connection, using a new disposable Box remote/name and
local test mountpoint. Observe Connect running state, browser authorization and
completion; then Test/Save can be checked separately. This also exercises the
Box creation repair, whose installed browser acceptance remains open. Agent
performed no authentication, config, service or data operation. Only task/notes
edited; source gate `target/task-runs/dev-gate-d4fz0g1o/summary.json` reused.
Scoped notes whitespace validation accompanies this update.

## Installed Box Add/authentication/Test/Save acceptance — October 22, 2026

Operator reports completing the guided Box Online Add process, including browser
authentication and testing, with all expected behavior. Read-only native config
inspection found a saved Box test connection whose remote resolves to a Box
backend. The operator used a different test name than the suggested literals:
initial literal-name checks were false, so they were not misreported as failed
persistence. A narrow count/type check found two Box remotes and two saved Box
connections, including one test-style remote/connection; saved references resolve
to Box remotes. No actual account names, tokens, client values or config dump
were printed or retained.

Accept native Add Connect/browser authorization/Test/Save and persisted test-entry
presence, closing the guided native Add button flow and Box creation-repair smoke.
No independent browser screenshot, real mounted Box read/write behavior,
Offline mirror, failure/cancellation, or Flatpak acceptance is inferred. Prior
synthetic failure/rollback coverage remains reusable. Native guided button checks
now cover selected Add/Modify transitions, confirmation/reset, failed Test gates,
saved-target guards, draft-only Disable/Enable, Remove confirmation/cancel,
remote In use/alignment, Return/Tab navigation and light/dark appearance. Broader
window/scaling and package acceptance remain in the open shared GUI item.

Task list and these notes changed only. Agent performed no authentication,
configuration write, mount/service/data mutation or installation. Reuse unchanged
source gate `target/task-runs/dev-gate-d4fz0g1o/summary.json`; no tests rerun.
Scoped notes whitespace validation accompanies this update.


## Installed Box mount and directory visibility acceptance — October 22, 2026

Following successful Box Add/authentication/Test/Save, the operator reports
mounting the new Box connection and observing directory entries populate.
Accept native Online mounting and directory visibility for this connection.
This adds to the preceding setup acceptance; file-content read/write, complete
tree/cache population, Offline mirror and Flatpak behavior remain unverified.

Only task records changed. No agent mount, service, configuration or data
operation was performed. Reuse unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; no tests rerun. Scoped notes
whitespace check passed (exit 0).


## Installed Box connection and remote removal acceptance — October 22, 2026

After the new Box Online setup and mount checks, the operator reports removing
both the Box test connection and its rclone remote successfully. Accept confirmed
connection and remote removal for this disposable native Box test. This supplies
the previously missing unused-remote removal observation; the earlier Google
Remove cancellation and remote In use checks retain their separate scope.
No independent backend inspection, local-file preservation check or Flatpak
acceptance is inferred from this report.

Only the task list and completion notes changed. The agent performed no deletion
or runtime operation. Reuse unchanged source gate
`target/task-runs/dev-gate-d4fz0g1o/summary.json`; no source tests rerun.


## Box Offline saved setup and failed initial Preview — October 22, 2026

Operator saved the new Box Offline connection, closed Add Connection, reopened
the saved connection and clicked Preview. Preview failed; the operator recalls
error 7. Another click reported unavailable in the current state. Accept saved
Offline setup only; initial Preview and synchronization remain outstanding.

Read-only inspection found the most recent coordinator state records
`initial-preview` / `failed`. Its bisync work directory contains only the filter
file, with no retained engine diagnostics or successful-preview marker found.
Source inspection explains retry blocking: `cached_offline_state` maps failed
coordinator state to Failed, controller maps it to Error, and Error offers Repair
while blocking Preview. `command_failure` exposes a nonzero exit code but discards
the captured engine diagnostic. The recalled code alone does not identify the
original failure. No root cause or repair is claimed.

Task records updated; read-only inspections exited 0. No source changes,
installation, Repair, sync, or service/data operations were performed. Existing
source gate remains reusable. Next: obtain the underlying engine diagnostic and
verify a safe recovery path before continuing initial-sync acceptance.


## Mirror failure diagnostic repair and Box retry plan — October 22, 2026

The operator proposes Repair followed by another Preview, and reports the Box
main-applet toggle remains on. Source inspection confirms Repair is the existing
recovery path: it stops/disables owned timer/service, reconciles workers, retains
mirror/recovery/engine data, and records paused state. No synchronization occurs
as part of Repair. The on indicator is independently misleading: failed
coordinator state masks paused timer observations before computing the paused
set. Its correction is tracked separately; no running sync is inferred.

Changed `src/bisync_coordinator.rs` to append a bounded redacted stderr excerpt
to nonzero command failures. Existing credential redaction precedes removal of
quoted values and path/remote-like tokens. At most eight lines and 1,600 Unicode
characters are shown; invalid UTF-8 is omitted. Arguments and stdout are not
shown, and no diagnostic log file is created. The English Fluent resource adds
the diagnostic wrapper; GUI engineering inventory documents the boundary.
Tests now actually populate stderr in the path-redaction fixture (the old test
had inadvertently supplied an empty stream). Additional synthetic coverage
checks useful error text, secret/path/quoted-value removal, stdout exclusion,
Unicode size limits and invalid output. State decisions still use typed exits.

Verification on dirty HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`:
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0, final receipt
  `target/task-runs/dev-check-sou2kj03/summary.json`.
- `cargo test --locked bisync_coordinator::tests::command_failure`: exit 0,
  two focused tests passed after the final fixture change.
- `bash scripts/dev.sh localization-check`: exit 0,
  `target/task-runs/dev-localization-check-yv4mdqfi/summary.json`;
  localization inputs unchanged by subsequent test-only edits.
- First `gate` attempt: exit 1, sandbox prevented just creating its temporary
  directory under the runtime directory; fmt and lint passed before that block.
  Receipt `target/task-runs/dev-gate-9371kfea/summary.json`.

No installation, Repair, service change, cloud access or synchronization was
performed by the agent. Operator must install/reload the changed applet before
Repair/Preview; if Preview fails, its new diagnostic notice is the next evidence.
The original Box error and installed diagnostic rendering remain unverified.

Full gate rerun with authorized runtime access passed (exit 0):
`target/task-runs/dev-gate-2qgvo7fv/summary.json`, with full output at
`target/task-runs/dev-gate-2qgvo7fv/task.log`. This establishes compilation and
synthetic/local test acceptance, not installed Box acceptance. Scoped whitespace
validation of the changed Rust and evidence records also passed (exit 0).


## Box Offline Preview backup-directory overlap repair — October 22, 2026

Operator installed the diagnostic change, retried Preview and supplied a
screenshot showing exit status 7 and rclone's backup-directory overlap error.
This accepts native diagnostic rendering and identifies the original failure.
Source corroboration: Box root remote recovery is inside the selected root,
while its filter had only a catch-all include. Google root filters likewise
lacked the recovery exclusion. Official rule checked at
https://rclone.org/bisync/: overlapping backup directories require exclusion.

Changed `src/sync.rs` to select Box/Google root recovery exclusions per
connection, retaining byte-identical established subfolder filters and all other
provider filters. `src/app.rs` managed-unit preparation and
`src/bisync_coordinator.rs` runtime preparation share the helper. Added a focused
synthetic test in sync and an ignored local-only engine regression in
`tests/bisync_coordinator_live.rs`; the GUI engineering inventory records scope.
No remote paths, credentials, account data or cloud calls appear in these tests.

The real-engine regression reproduces old-filter exit 7, then verifies fixed
Preview does not transfer content or set initialization, initial sync copies
ordinary files, recovery folders never appear on the opposite side, and an
overwrite preserves the old content in recovery. An initial fixture with just
one synchronized file hit rclone's all-files-changed guard during the overwrite
phase (exit 1). Added two unchanged synthetic files; no production safeguards
were relaxed. The corrected test passes for both Box and Google policies, using
local alias backends rather than their cloud APIs.

Verification on dirty HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`:
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0, final receipt
  `target/task-runs/dev-check-p0t_g31m/summary.json`.
- `cargo test --locked root_recovery_filters_exclude_backups_without_changing_subfolder_filters`:
  exit 0, one focused test passed; inputs unchanged by fixture-only correction.
- `cargo test --locked --test bisync_coordinator_live root_mirror_recovery_exclusion -- --ignored --nocapture`:
  first run exit 101 on the all-files-changed fixture guard; corrected run exit 0,
  one explicitly enabled integration test passed with both provider policies.
- English/localization inputs unchanged; reuse localization receipt
  `target/task-runs/dev-localization-check-yv4mdqfi/summary.json`.

No agent installation, user configuration write, real service operation, Repair
or cloud synchronization occurred. Installed Box Repair/Preview/initial-sync
acceptance remains pending. The main-applet toggle issue is separate and open.

Narrow read-only native configuration inspection confirms the saved Box Offline
connection selects the remote root (`remote_subpath: None`), matching the
reproduction. Only provider/mode/root-selection booleans were printed; no
connection names, remote references, paths or credentials were exposed.

Final `bash scripts/dev.sh gate` passed with authorized runtime access (exit 0):
`target/task-runs/dev-gate-tckkmd9d/summary.json`, full log
`target/task-runs/dev-gate-tckkmd9d/task.log`. Scoped Rust/test/record whitespace
check passed (exit 0). The gate's release-publication messages come from fake
adapter fixtures; no actual publication was requested or performed. Next
operator step: install-user, close editor/settings and reload the panel, then
Repair/confirm and Preview the saved Box test mirror. Do not infer cloud
initial-sync acceptance from the local alias integration.


## Installed Box Offline subfolder Preview and initial sync — October 22, 2026

After guidance to select a small existing Box subfolder rather than the very
large account root, the operator reports Preview and Sync Now passed and files
were synchronized into the local folder. Accept native Box Offline subfolder
Preview, initial sync and observed local file arrival. This follows the earlier
saved Offline setup and diagnosed root backup-directory overlap failure; it
does not establish installed whole-root recovery-exclusion acceptance.

No independent file-count/checksum verification, ongoing two-way sync, timer
scheduling, failure/recovery or Flatpak behavior is inferred. The misleading
main-applet toggle after a failed Preview remains open. Only task/evidence
records changed; no engine, service or data operation was performed by the
agent. Reuse unchanged source gate
`target/task-runs/dev-gate-tckkmd9d/summary.json`; no tests rerun. Scoped notes
whitespace validation passed (exit 0).


## Failed-Preview main-applet toggle source correction — October 22, 2026

Operator authorized correcting the toggle that remained on after failed Box
Preview despite paused background synchronization. Changed `src/app.rs` to
separate cached unit-derived state from the coordinator error overlay. Paused
set membership now uses cached service/timer observations independently of a
failed/cancelled operation. Failure still produces Error and keeps Repair
available while Preview/Resume/Sync remain blocked. A live timer retains the on
indication despite a last-run error; live services and tracked manual operations
retain the running/on indication. Missing observations retain unknown behavior.
No new backend I/O, command, polling, configuration write or UI text is added.

Expanded existing fake-observation regression coverage across six rclone mirror
providers (OneDrive, SharePoint, Google Drive, Box, SMB, SFTP), checking failed/
cancelled stopped schedules, active timer/service, manual-operation overrides,
Repair visibility and action guards. Incomplete-observation coverage also
checks a coordinator failure does not invent an observed paused state. Updated
the task list and GUI engineering inventory alongside these completion notes.

Verification on dirty HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`:
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0,
  `target/task-runs/dev-check-j_1pqst8/summary.json`.
- `cargo test --locked --bin cosmic-ext-applet-mounter popup_coordinator_`:
  exit 0, three focused tests passed.
- `cargo test --locked --bin cosmic-ext-applet-mounter unobserved_offline_units_are_unknown_not_paused`:
  exit 0, one focused test passed.

Earlier source/UI-string/engine results retain their recorded scope. No
localization inputs or engine/coordinator code changed, so localization and
Box/Google local-engine integration were not repeated. No installation, service
change, cloud operation or induced live failure was performed by the agent.
Installed native toggle rendering and Flatpak acceptance remain pending.

Final `bash scripts/dev.sh gate` passed with authorized runtime access (exit 0):
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; full output at
`target/task-runs/dev-gate-8fnjtzwp/task.log`. Scoped source/record whitespace
check passed (exit 0). No verification failures occurred in this task. Next:
operator install-user/reload; confirm stopped failed mirrors show an off toggle
while retaining Error and Repair. Installed confirmation is still pending.


## Installed Google Offline subfolder Preview and initial sync — October 22, 2026

Following the guided Google Drive Offline subfolder setup, the operator reports
Preview showed one file to synchronize, then Sync Now copied that file into the
local folder. Accept native Google Offline subfolder Preview, initial sync and
observed local file arrival. Remote reuse versus newly authenticated remote was
not specified; no additional browser/OAuth acceptance is inferred.

Whole-root recovery exclusion, ongoing two-way synchronization, scheduled runs,
independent content/checksum validation and Flatpak behavior remain separate.
Only the task list and these notes changed. No engine, service, authentication
or data operation was performed by the agent. Source inputs are unchanged;
reuse gate `target/task-runs/dev-gate-8fnjtzwp/summary.json`. No tests repeated.
Scoped notes whitespace validation passed (exit 0).


## Installed Google Offline uploads and text-update follow-up — October 22, 2026

Operator added two local files and ran Sync Now directly; both uploaded and were
visible in Google Drive. Accept native ongoing local-to-cloud creation for
those files. The requested pre-upload Preview was skipped, so no upload Preview
count is inferred. Operator then edited a text file, ran Preview and Sync, and
reports the changed text is now also on the local drive. Record that update
observation; the edit's origin was not explicitly specified, so cloud-to-local
modification acceptance awaits clarification. No conflict, deletion, scheduler,
whole-root or Flatpak behavior is inferred.

Task list and completion notes only changed. No agent runtime or data operation
was performed. Reuse unchanged gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests repeated. Scoped notes
whitespace check passed (exit 0).


## Google Offline cloud-origin edit clarified — October 22, 2026

Operator clarifies that the text file was uploaded first, edited online in
Google Drive, then synchronized again; the online edit appeared in the local
file. This resolves the preceding direction ambiguity. Accept native Google
Offline cloud-to-local modification syncing, alongside the already accepted
local-to-cloud file creation in the same test folder. Broader lifecycle cases
(conflicts, deletion/recovery, scheduling), whole-root and Flatpak acceptance
remain separate.

Only task/evidence records changed. No agent runtime or data operation occurred.
Reuse unchanged gate `target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests
repeated. Scoped notes whitespace validation passed (exit 0).


## Selectable preload scope declined — October 22, 2026

Operator explicitly declines implementing root-only/shallow preload scope.
Mark the selectable Google preload-scope proposal deliberately skipped (`[o]`),
with no implementation planned. Retain the implemented root-first then
recursive preload behavior and its existing acceptance evidence. This decision
does not concern Offline mirror remote-folder selection or its recovery guards.

Next proposed installed check is Box Offline ongoing local-to-cloud creation
and cloud-to-local modification in the existing disposable subfolder, matching
the recently accepted Google flow. Box subfolder Preview/initial download are
already accepted and need not be repeated. Only task/evidence records changed;
no source, runtime or data operation occurred. Reuse unchanged gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests repeated. Scoped notes
whitespace validation passed (exit 0).


## Installed Box Offline cloud-to-local deletion — October 22, 2026

Operator reports deleting one file on Box, synchronizing again and observing
that file disappear from the local mirror directory. Accept native Box Offline
cloud-to-local deletion propagation for that test file. The previously proposed
upload/edit flow was not reported; do not infer it passed. Preservation of the
deleted local copy in the configured recovery directory remains unverified and
is the next read-only observation. No restoration, conflict, scheduling,
whole-root or Flatpak acceptance is inferred.

Only task/evidence records changed. The agent performed no deletion, sync or
runtime operation. Reuse unchanged source gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests repeated. Scoped notes
whitespace validation passed (exit 0).


## Installed Box deleted-file recovery presence — October 22, 2026

Operator confirms the deleted files are present in the recovery folder after
the Box Offline cloud-to-local deletion test. Accept observed recovery-copy
presence, extending the preceding deletion-propagation acceptance. Opening or
comparing recovered contents, restoration and retention expiry were not
reported and remain unverified.

Only task/evidence records changed; the agent performed no runtime or data
operation. Reuse unchanged gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests repeated. Scoped notes
whitespace validation passed (exit 0).


## Installed Settings communication recovery check started — October 22, 2026

Operator explicitly requests the proposed installed recovery test across panel
restart. Reviewed native build guide and runtime implementation: standalone
windows request Status every two seconds, with a five-second request timeout;
a successful reply marks runtime available and clears the current runtime error.
Runtime ownership uses queued D-Bus ownership and signal-based acquisition.

The normal installation/reload procedure closes all app windows first. For this
authorized recovery test, intentionally keep standalone Settings open across
`killall cosmic-panel` to observe reconnection; close editors and avoid running
operations or configuration edits during the observation. This is a scoped test
of an existing window losing/reacquiring the runtime owner, not a new install
or an agent command/service action. Observe both initial health and recovery
within roughly 10–15 seconds after the panel returns; brief restart downtime
need not produce a visible warning if it falls between status requests.

No acceptance result yet. Task/evidence records only changed. Prior source gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json` reused; no tests repeated.
Scoped notes whitespace validation passed (exit 0).


## Installed Settings reconnection passed; retired-profile refresh warning diagnosed — October 22, 2026

Operator confirms standalone Settings remained open across the requested panel
restart, briefly showed a communication notice, then returned to normal
automatically. Accept native Settings reconnection to the restarted applet.
This does not establish owner-loss recovery within the same running applet
process or Flatpak behavior.

A separate recurring main-applet status-refresh warning remained. Source trace
shows any failed per-unit observation triggers it; batch reads currently
collapse absent-unit/error/timeout into unavailable status. Read-only live
D-Bus inspection found the Runtime owner present, 13 configured connection IDs
and one missing service file. That entry was a OneDrive Offline compatibility
profile with no explicit engine; the model resolves that legacy form to retired
LegacyOnedrive. Other units had disabled files; their absence from ListUnits
means unloaded, not a failed unit or missing file. A narrow state-file scan found
no malformed coordinator JSON. No names, paths, credentials or account values
were printed.

Operator then removed the legacy profile and reports the recurring warning
disappeared, confirming the missing retired unit as its cause. Record source
follow-up: distinguish expected absence of a retired service from observation
failure without suppressing real errors or hiding unexpectedly active legacy
services. No source implementation is claimed.

Initial read-only D-Bus probe was blocked by the sandbox (exit 1, AccessDenied);
rerun with operator-authorized runtime access succeeded (exit 0). Only read-only
bus-name/unit/file listings were requested; the agent did not install, stop,
start, delete or alter configuration/services/data. Task list and these notes
changed. Reuse unchanged source gate
`target/task-runs/dev-gate-8fnjtzwp/summary.json`; no tests repeated. Scoped notes
whitespace validation passed (exit 0).


## Installed Google/Box automatic scheduled downloads — October 22, 2026

Operator started both disposable Offline test mirrors and waited for scheduled
synchronization. Read-only saved interval inspection showed 15 minutes for both
Box and Google Offline mirrors. Operator then reports the scheduled test files
arrived in both local folders and explicitly clarifies the direction was
cloud-to-local automatically, without manual Sync Now. Accept native scheduled
download behavior for these two small test folders. This supersedes the initially
proposed local-to-cloud direction for this particular timer test.

Stopping/pausing both schedules was requested as the remaining cleanup/control
check; its result is not yet reported. No scheduled-upload, interruption,
conflict, whole-root or Flatpak acceptance is inferred. No agent runtime or data
operation occurred. Source change in progress concerns only the interval
field label/help; it was not installed during the timer test.

## Offline interval field label and help — October 22, 2026

Operator requested a narrower input with a left label or a visible label above
the existing full-width input. Implemented the latter in `src/app.rs` shared
Add/Modify wizard: visible “Sync interval in minutes” label, grouped above the
input. English Fluent adds the label and replaces “How often” help with explicit
interval semantics: initial delay after background Start, delay after the prior
background run finishes, and manual Sync Now independence. No backend, default,
validation, configuration or live schedule changes. GUI engineering inventory
and task list updated.

Verification on dirty HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`:
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0,
  `target/task-runs/dev-check-hps97dj9/summary.json`.
- `cargo test --locked --bin cosmic-ext-applet-mounter editor_actions_confirm_save_resets_on_edits_and_cancellation`:
  exit 0, one existing focused test covers interval-edit confirmation invalidation.
  No new tests mirroring this label-only change were added.
- `bash scripts/dev.sh localization-check`: exit 0,
  `target/task-runs/dev-localization-check-yoi8ivp9/summary.json`.

Installed Add/Modify label and hover-help rendering remain pending. No agent
installation or service operation occurred. Earlier engine/IPC evidence remains
reusable; no additional cloud tests were run by the agent.


## Scheduled Google/Box test toggles off; interval verification follow-up — October 22, 2026

Operator confirms both test mirror toggles are now off after the accepted
automatic cloud-to-local downloads. Accept operator-observed pause controls for
this test cleanup; no independent post-action timer inspection or extra-interval
no-transfer observation was performed.

First gate for the interval label/help change failed (exit 101):
`target/task-runs/dev-gate-rez5fzgy/summary.json`. Failure was the existing
`process::private_config_tests::private_config_failure_hides_both_output_streams`
assertion expecting NonZero exit 7; 285 library tests passed, one failed and
11 were ignored. The assertion did not expose the actual unexpected error
variant, so its cause is not established. The process source was unchanged
by this UI task. Isolated rerun
`cargo test --locked --lib process::private_config_tests::private_config_failure_hides_both_output_streams -- --exact --nocapture`
passed (exit 0). A final gate rerun is required; do not treat that isolated pass
as full-gate acceptance. No production change or fixture relaxation was made
to hide the initial failure.

Final interval-change gate rerun passed (exit 0):
`target/task-runs/dev-gate-s6ixc3td/summary.json`; full log
`target/task-runs/dev-gate-s6ixc3td/task.log`. The initial process-test failure
was not reproduced either in isolation or this full rerun; its underlying cause
remains unknown and its first failure receipt is retained above. Scoped
source/Fluent/record whitespace checks passed (exit 0). The interval-label source
task is complete; installed native and Flatpak visual checks remain pending.


## Installed Offline interval label visible — October 22, 2026

Operator reports installing the updated version and confirms the interval text
is present. Accept native visibility of the new interval label in the observed
editor. Hover-help wording, both Add and Modify windows individually, alternate
scaling and Flatpak rendering were not explicitly reported.

Only task/evidence records changed; no agent runtime or data operation occurred.
Reuse unchanged gate `target/task-runs/dev-gate-s6ixc3td/summary.json`; no tests
repeated. Scoped notes whitespace validation passed (exit 0).


## Retained retired-profile refresh warning source fix — October 22, 2026

Operator authorized fixing the global refresh warning caused by a retained
retired profile whose service had been removed. Changed `src/services.rs` to
return typed Present/Missing/Unavailable batch observations. Only LoadUnit
NoSuchUnit is absence; genuine errors/timeouts remain unavailable. Loaded
services with no unit file retain their live properties; GetUnitFileState
NoSuchUnitFile is recorded as not enabled rather than suppressing the service.

Changed `src/app.rs` to accept a missing service as fresh only for a retired
profile, replace cached active status on confirmed absence, and replace the
absence set per observation. Genuine failures keep the old cache and fail
freshness; supported missing services/timers still warn. Retired-state and
operation safety policies remain intact. New fake-observation tests exercise
implicit/explicit legacy engines, missing-after-active cache removal, subsequent
read failure, reappearing active service and all six supported rclone mirror
providers. Typed error tests reject access/load/time-out errors as absence.
Task list and GUI engineering inventory updated. No new UI text, transport,
polling, timeout, permissions or persistent format was introduced.

Verification on dirty HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`:
- `just fmt`: exit 0.
- `bash scripts/dev.sh check`: exit 0,
  `target/task-runs/dev-check-doqj0aed/summary.json`.
- `cargo test --locked --lib unit_observation_`: exit 0, four focused tests pass.
- `cargo test --locked --bin cosmic-ext-applet-mounter missing_service`:
  exit 0, two focused tests pass.

No installation, user configuration recreation, service operation or data
mutation was performed by the agent. Installed retained-profile and Flatpak
acceptance remain pending. Localization inputs unchanged; reuse
`target/task-runs/dev-localization-check-yoi8ivp9/summary.json`.

## 0.5.5 closure before validation/expansion selection — October 22, 2026

Operator asks what remains before moving toward the validation/expansion
backlog. Review confirms that backlog explicitly remains deferred unless a
changed supported behavior or observed defect promotes an item. It is not a
mandatory exhaustive pre-release retest. Preserve the original priority order:
legacy removal, GUI cleanup, then COSMIC submission preparation.

Remaining closure groups:
- Complete the current defect's source gate and installed smoke; reconcile
  which changed-input installed checks remain (notably the new OneDrive/rclone
  Offline Add/Test workflow, remaining GUI/scaling observations and separately
  unaccepted whole-root mirror cases). Reuse the accepted Google/Box native
  setup, subfolder sync, timer, recovery and Settings reconnection observations.
- Reconcile active checklist, release notes, user/specification documentation
  and engineering evidence so native source installations, package acceptance
  and deliberately skipped/deferred cases are explicit. The draft release notes'
  blanket no-installed-acceptance wording needs this distinction.
- Before claiming Debian/Flatpak removal-release support, complete their
  package-specific upgrade acceptance. For COSMIC preparation, refresh source/
  manifest identity and dependency/permission rationale, recheck current rules,
  and retain operator-authored submission/replies. No build, install or
  publication is authorized merely by this planning review.

Then select backlog items by priority and available disposable resources,
without repeating already accepted unchanged paths or restoring the declined
root-only/shallow preload proposal. No additional backlog implementation began.

Final retired-profile fix gate passed with authorized runtime access (exit 0):
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; full log
`target/task-runs/dev-gate-sbk8qk3q/task.log`. No compilation or test failures
occurred in this source task. A trailing space in the new planning note was
removed; final scoped Rust/record whitespace check passed (exit 0). Source and
synthetic acceptance are complete. Next practical closure work is the 0.5.5
acceptance/documentation reconciliation, then only the remaining changed-input
live checks (starting with the OneDrive/rclone Offline Add/Test workflow),
followed by package/submission preparation in the original priority order.


## 0.5.5 acceptance reconciliation (October 22, 2026)

Reconciled against the dated operator reports and execution records above.
This is the current acceptance snapshot; earlier “pending” statements describe
their historical stage and are superseded only within the scope shown here.
Native development installs, synthetic tests, local-engine integration and
final-package acceptance remain separate. Current dirty source is based on
HEAD `057b6474ce206768645c841bc4f0a1c9818640d0`; there is no immutable final
0.5.5 package candidate accepted by this reconciliation.

### Completed cases to reuse

| Area | Accepted evidence | Boundary retained |
|---|---|---|
| Legacy retirement | Native installed startup stopped/disabled/removed the owned monitor unit; retired row and preserved mirror/recovery/database/credential locations observed. Operator later removed the retired profile. | Original installed check used the operator's backed-up profile. Debian/Flatpak upgrade acceptance is separate; no cloud/local data deletion is inferred from profile removal. |
| Native GUI and buttons | Guided Add/Modify layouts, blue/grey/red roles, faded running Test/Saving states, failed-Test Save blocking, draft invalidation, unsaved Preview/Sync guards, draft Disable/Enable, Remove confirmation/cancel. | Accept the observed editor paths and ordinary scaling, not every provider/window or persisted disable. Source tests cover the shared provider/mode policy. |
| Confirmation and keyboard | SharePoint Save → Confirm Save → Saving; confirmation clears after edits; reverted unchanged draft blocks Save. Unique Google/OneDrive remotes save without unnecessary confirmation. Return and Tab/Shift+Tab/focus passed. | Space activation was an incorrect guided expectation for pinned libcosmic, not a remaining app defect. |
| Remote inventory and cleanup | In use actions aligned and non-clickable; later Box test connection and rclone remote removed successfully. | No repeated unused-remote deletion is needed to resolve the earlier ambiguous removal report. Data-preservation/content checks were not independently reported. |
| Themes and interval label | Native light/dark appearance at usual scaling; installed “Sync interval in minutes” label visible. | Hover-help text, each Add/Modify window individually, alternate scaling and Flatpak rendering are not blanket accepted. |
| Google Online | Mount/root listing inspection; root visible in 10–20 seconds and directories through three levels after root-first/fast-list repair. Private OAuth fields updated and Online Re-Test passed. | No full-drive/depth-cap/quota-resolution claim. Existing-remote OAuth update does not prove fresh browser consent or token freshness. |
| Google Offline subfolder | Preview proposed one file; initial Sync copied it locally. Two local files uploaded; an online text edit subsequently arrived locally after Preview/Sync. | Accept the reported file creation and cloud-origin update, not conflicts/deletions or exhaustive content verification. Fresh remote creation/authentication was not specified. |
| Box Online | New Connect/browser authorization/Test/Save, saved remote reference, mount and visible directory entries. Later test connection/remote cleanup passed. | Mounted file-content read/write and final-package behavior remain separate. |
| Box Offline subfolder | Saved setup, successful Preview/initial Sync/local arrival; cloud deletion propagated locally and deleted copies were present in recovery. | Local uploads/cloud edits were proposed but not reported as passed. Recovery content comparison/restoration/expiry are not inferred from copy presence. |
| Google and Box timers | With 15-minute intervals, cloud test files arrived locally automatically without Sync Now; operator then turned both test toggles off. | Scheduled uploads, prolonged no-transfer-after-pause observation and independent post-action timer inspection were not reported. |
| SharePoint Modify subfolder | Name/path changes, Re-Test, shared-remote Confirm Save, successful Save, Preview and initial Sync; unsaved edits block runtime actions. | This is the archive-subfolder case, not empty-folder/whole-library acceptance or fresh Add authentication. |
| Settings communication | Settings stayed open across panel restart, briefly showed disconnection, then returned to normal automatically. Runtime owner observed present. | Same-process owner-loss recovery and Flatpak acceptance remain separate. The recurring refresh warning was independently diagnosed as the removed legacy unit. |
| Root recovery exclusion | Synthetic and real-rclone temporary local-alias tests reproduce old exit 7, then pass root Preview/initial sync and overwrite recovery for Box/Google policy. | Engine integration is local-only, not installed Box/Google whole-root cloud acceptance. Existing subfolder filters remain unchanged. |
| Failed-Preview toggle | Source/fake tests across all six rclone mirror providers retain Error/Repair, show stopped schedules off and live work/schedules on. | A normal successful-sync toggle-off report does not establish installed failed-Preview rendering. |
| Retired missing-unit warning | Typed absence/read failure separation, stale active-cache replacement, supported-unit failures and live legacy visibility pass six focused tests and full gate. | Operator removed the old profile before this source fix. That workaround is not an installed retained-profile regression test. |

The native GUI aggregate item is now marked complete only for the conducted
checks above. Its original scope also included broader window/scaling and
package coverage; those unperformed cases remain explicit below and in their
existing authoritative backlog/package tasks. Phase 2 is not blanket signed
off while changed-path installed gaps remain.

### Remaining acceptance, without repeating passed cases

1. **Changed setup paths:** installed OneDrive/rclone Offline Add/Test selected-
   remote verification merge; fresh Google browser consent after the repair.
   Reuse prior backend lifecycle evidence and the accepted Google/Box flows;
   do not create a new full provider/account matrix solely for button labels.
2. **New fix rendering/upgrade:** installed failed-Preview toggle behavior and
   retained-retired-profile absence handling. Use disposable fixtures; the latter
   can be combined with package-upgrade acceptance instead of restoring the
   removed real user profile. Source/fake coverage already passes.
3. **New root scope:** installed Box/Google whole-root and SharePoint empty-
   folder/whole-library cases remain unaccepted. A very large real account is
   not a reason to repeat subfolder tests or to claim root acceptance. Choose
   suitable small resources or explicitly retain the live-validation limit.
4. **Final packages:** 0.5.5 Debian/Flatpak removal upgrade and changed GUI/
   host-integration smoke remain separate. Native installs and 0.5.4 package
   acceptance do not establish these results. No candidate is built/installed here.
5. **Release/submission closure:** finalize documentation and source/manifest/
   permission rationale; recheck current COSMIC rules at submission time and
   preserve operator-authored PR/replies. These are preparation tasks, not
   additional live provider tests.

Broader window/scaling, grey OAuth in-flight observation, upload/edit variants,
recovery restoration, fault/lifecycle breadth, large/shared drives and extended
VPN/sleep cases retain their original limitations. They are not silently marked
passed or promoted into new mandatory release gates. Broader validation belongs
to the existing validation/expansion backlog unless changed behavior or a defect
requires focused work. Selectable root-only/shallow preload remains deliberately
skipped; existing root-first recursive preload remains accepted within its scope.

### Earlier evidence and verification reuse

[0.5.4 acceptance map](docs/engineering/release-054-acceptance-audit.md) retains
OneDrive personal/work-school mirror setup/sync/guards, SharePoint subfolder,
Repair, schedule/Pause, ownership/VPN and idle sleep evidence at its stated
development/native/Flatpak levels. Reuse unchanged policy/backend cases only;
it does not cover the changed 0.5.5 verification merge, root scope, retirement,
rendering or final package identity. No prior live limitation was converted to a
pass. The latest source gate is
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; root-recovery local integration
and earlier localized UI checks retain their recorded unchanged-input scope.

Changed records: Task List, these completion notes, GUI inventory/status and
engineering index, and draft 0.5.5 release-note acceptance wording. The draft
remains unpublished; its former blanket “no installed artifact accepted” wording
is replaced with scoped native development acceptance and pending package
acceptance. No Rust, Fluent, test, script, configuration, service or cloud input
is changed. No full gate, installation, build, commit or publication is repeated.

Reconciliation verification:
- `python3 tests/repo_hygiene_test.py`: exit 0, four checks passed, including
  repository documentation links. No changed input to that check followed.
- Scoped `git diff --check` for completion notes, GUI inventory/index and draft
  release notes: exit 0. Existing operator Task List whitespace is preserved.
- SHA-256 preservation assertions: exit 0. Task List sections before GUI cleanup
  and from validation/expansion backlog onward are byte-identical; all prior
  completion notes are an unchanged prefix; Push Buttons is unchanged; Rust,
  Fluent and Cargo inputs checked are unchanged.

No additional test or live acceptance was invented by this reconciliation.
Next focused installed case is the OneDrive/rclone Offline Add/Test selected-
remote verification merge, reusing previously accepted backend lifecycle tests.


## OneDrive Offline selected-remote Add/Test check started — October 22, 2026

Operator requests the next guided test. Use Add Connection, OneDrive Offline,
Detect/select an existing primary-drive rclone remote, matching account type
and expected account, a small existing remote subfolder, and separate empty
local mirror/recovery directories. One Test should verify identity/folder and
automatically continue full draft validation, enabling Save on success without
a separate Verify action. Stop at the Test result before confirming Save. If
no suitable remote is detected, report that rather than silently substituting
new browser setup for this selected-remote verification check.

This is a pending native editor test, not new backend lifecycle acceptance.
No agent runtime/configuration/data operation or source change occurred. Source
gate `target/task-runs/dev-gate-sbk8qk3q/summary.json` remains reusable; no tests
repeated. Scoped notes whitespace validation passed (exit 0).


## Installed OneDrive Offline new-remote Add/Test/Save — October 22, 2026

Operator saved the new OneDrive Offline connection, confirms Test passed after
one click, and clarifies Detect found no existing remotes so a new remote was
created. Accept native new-remote creation, one-click Test and successful Save.
The run followed a different route than the planned detected-existing-remote
check. Connect can populate verified drive identity; therefore passing Test
after creation does not prove Test's automatic existing-remote verification
continuation. Keep that narrower branch pending rather than repeating new
authentication or a full backend lifecycle. Browser consent details were not
separately described in this report.

Next lightweight observation: a fresh OneDrive Offline Add editor's Detect should
now list the newly created remote. It can be reused to test the existing-remote
branch without another remote creation or another saved connection. No result
for this follow-up is yet reported. Task list and these notes changed only; no
agent runtime/data operation or source edit occurred. Reuse unchanged gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; no tests repeated. Scoped notes
whitespace check passed (exit 0).


## OneDrive discovery filtering and returning Repair action — October 22, 2026

Operator reports Detect found nine total remotes but none matching OneDrive,
although the remote inventory includes the saved mirror remote. Narrow read-only
inspection finds one primary OneDrive work/school backend and one document-
library backend. The saved rclone Offline test uses the work/school remote.
Source filtering requires the selected drive type; fresh Add defaults Personal.
Therefore choose Work or school before Detect for this remote. Actual current
Add-window type was not independently observed; do not claim an underlying
discovery defect or an existing-remote verification pass yet. Correct the earlier
guide's ordering: type selection must precede filtered remote selection.

The operator also reports a returning Repair action and clarifies the screenshot
notice was from an earlier Repair. Current saved test profile has enabled=false;
both managed service/timer files exist and coordinator state is repair/paused.
Controller policy labels a disabled profile Unavailable and offers Repair in
that state, explaining availability despite completed Repair. Repair pauses
units but does not enable the saved profile. No current failed sync is inferred
from the older completion notice. Await operator clarification whether Disable
was intentional or an earlier setup attempt failed. Source authentication-error
handling can disable the draft and successful verification preserves that flag;
this is a possible route, not an established event in this test.

Only provider/type/engine/enabled/state/file-presence facts were printed, not
remote names, account values, credentials or directory contents. Task list and
these notes changed; no agent config, service or cloud operation occurred.
Source unchanged; reuse gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`. No tests repeated. Scoped notes
whitespace validation passed (exit 0).

## Replacement work/school OneDrive mirror transfers — October 22, 2026

Operator removed the earlier test connection and created a replacement using
a work/school account. They explicitly confirm Connect created another remote,
rather than selecting an existing remote. After saving, two files were placed
in the local mirror directory; Preview reported two items needing update and
Sync Now copied them to the remote. Accept installed native new-remote setup,
Preview of two local additions and manual local-to-cloud upload. Do not infer
scheduled upload, existing-remote Test continuation, or a fix for the removed
profile's unexplained disabled flag.

In the subsequent reverse-direction check, the operator added a file through
the OneDrive website, clicked Sync Now and confirmed it arrived in the local
mirror directory. Accept manual cloud-to-local download as well. These two
directions establish basic native manual transfers for this replacement;
scheduled transfers, conflicts and recovery behavior were not exercised here.

Remaining focused check: in a fresh Add editor choose OneDrive, Offline mirror
and Work or school before Detect; select the existing remote and test a small
subfolder using separate empty local/recovery directories. Stop after a single
successful Test; another saved connection or browser authentication is not
needed for that check.

Only Task List and completion notes changed; no source, runtime, cloud or user
configuration operation was performed by the agent. Reuse unchanged source gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; no source tests repeated.
Scoped patch whitespace validation passed (exit 0).


## Existing OneDrive remote Add/Test accepted — October 22, 2026

With Work or school selected before Detect, the operator reports two matching
remotes. They selected an existing remote and configured mirror paths. Test was
initially blocked because the required account email was blank; source review
confirmed RemoteSetup validates that input before enabling Test. The earlier
walkthrough omitted this required field. After supplying it, the operator
reports success and provides a rendered Add-window screenshot: drive identity
and selected-folder access verified, managed service/timer structural validation
passed, and Save is enabled. Accept installed native existing-remote Test
verification continuing into full draft validation, without Connect. Saving a
second connection is unnecessary for this scoped acceptance and is not claimed.

Only task/evidence records changed. No source edit or agent service/config/cloud
operation occurred; unrelated edits and Push Buttons are preserved. Reuse gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json` for unchanged source inputs;
no source tests repeated. Completion-notes whitespace check exited 0. Fresh
Google browser consent remains a separate changed-path creation acceptance item.


## Fresh Google browser-consent check started — October 22, 2026

Operator agrees to run the remaining native fresh-remote authentication check.
Use Add Connection, Google Drive, Online mount, a distinct connection name and
unused remote name, the operator's own Google OAuth client ID/secret pair and
a separate empty mountpoint. Connect should open the browser and complete
consent; then Test should validate access and enable Save. Stop at the Test
result. This creates a disposable remote through the UI but does not require
saving or mounting another connection to prove this authentication path.
Await actual browser and Test observations; nothing is accepted yet.

Only task/evidence records changed. No agent runtime/configuration/cloud
operation or source edit occurred. Reuse unchanged source gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; no tests repeated.
Completion-notes whitespace check exited 0.


## Fresh Google browser-consent accepted — October 22, 2026

Following the guided fresh Google Online setup, the operator reports "Passed
and Saved." Accept the reported installed native new-remote browser
authentication, Test and successful Save. This closes the remaining scoped
native creation checks alongside the accepted OneDrive existing-remote Test
continuation. Earlier existing-remote OAuth updates and Google/Box mirror
transfers remain separately evidenced; no fresh Google Offline consent,
new mount observation or final Flatpak acceptance is inferred from this report.

Updated Task List and appended these completion notes only; no source edit or
agent configuration/service/cloud operation. No failures in this documentation
update; completion-notes whitespace check exited 0. Reuse unchanged source gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; no source tests repeated.
Remaining acceptance boundaries include installed failed-Preview toggle and
retained retired-profile behavior, small whole-root cloud cases or explicit
limits, and final package/Flatpak integration. These are not closed by the
successful native creation checks.


## Installed failed-Preview toggle and recovery accepted — October 22, 2026

During the guided disposable Google Offline subfolder check, temporarily
renaming the cloud test folder caused Preview to fail with exit status 7. The
rendered diagnostic reported bisync aborted and retryable without resync in
resilient mode. Operator explicitly confirms the main-applet toggle was off
before Repair, and Repair was available in the main applet. This accepts the
installed native correction separating a paused schedule from operation failure.

Operator clicked Repair, restored the cloud folder's original name, then reports
Preview and Sync Now both passed. Accept this observed native failure/recovery
sequence. No forced resync, data deletion, final Flatpak behavior or retained
legacy-profile missing-service acceptance is inferred. The latter two checks
remain open; the historical disabled OneDrive profile was not reproduced here.

Updated Task List and appended completion notes only. Agent performed no source,
service, configuration or cloud mutation. Reuse unchanged source gate
`target/task-runs/dev-gate-sbk8qk3q/summary.json`; no source tests repeated.
Completion-notes whitespace validation exited 0. No documentation-check failure.


## GUI acceptance handoff and cloud-root limits — October 22, 2026

Operator agrees to combine the retained retired-profile missing-service check
with package-upgrade acceptance and proceed. The package check remains open:
a disposable retained unsupported profile with its retired unit absent must not
cause recurring refresh warnings; data preservation and unsupported controls
remain part of upgrade acceptance. Do not repeat a separate native legacy setup
merely to duplicate that package observation. Failed-Preview native acceptance
is already complete and remains distinct from sandbox rendering.

Operator cannot mirror the full Google/Box accounts because of data volume.
Deliberately omit those live whole-root runs and retain this explicit limitation
in the active task list and draft release notes. Reuse the real-engine local
root/recovery fixture and installed cloud subfolder tests at their recorded
levels. This is an accepted validation limit, not a cloud-root pass. SharePoint
whole-library acceptance is separate and remains open; no decision for it is
inferred from the Google/Box constraint.

Phase 2 source/scoped native GUI work is complete with the above explicit handoff.
Proceed to Phase 3 source/manifest, dependencies and permissions review before
freezing a package candidate. Reviewed release/packaging procedures; Cargo
version is 0.5.5 and Debian changelog is 0.5.5 UNRELEASED. Dirty source still needs
an immutable candidate identity before final release/package claims. No commit,
build dispatch, installation, service change or publication was authorized by
this documentation handoff. An initial manifest search glob matched no files;
manifest discovery is still needed, not evidence of a missing manifest.

Changed Task List, completion notes and draft v0.5.5 release notes only. Reuse
unchanged source gate `target/task-runs/dev-gate-sbk8qk3q/summary.json`; no source
tests repeated. Scoped notes/release whitespace validation exited 0. Next:
review the maintained Flatpak source manifest and host-integration rationale,
then prepare the exact combined package-acceptance checklist.


## 0.5.5 release readiness preparation — October 22, 2026

Operator directs moving toward the 0.5.5 release and clarifies COSMIC submission
means a later PR. Prioritize GitHub Debian/Flatpak release artifacts; do not make
store acceptance or a submitted PR a release prerequisite. Existing COSMIC audit
now has a current readiness section and combined package-validation checklist,
while preserving dated 0.5.4 candidate facts. Current official README and PR
template were fetched: applets still belong in cosmic-flatpak; own app/manifest
exception and operator-authored submission boundary remain. Repository justfile
and BaseApp-source URL fetches failed; no current upstream build-recipe/runtime
compatibility approval is inferred from them.

Reviewed manifest: app pin remains historical 72661ce, runtime target unchanged,
23 declared grants, eight shipping typed executable variants after onedrive
removal. Host execution remains required by supported engine/helper routes;
no permission reduction or expansion is proposed. Legacy retirement is native-
accepted; package upgrade and retained-profile missing-service regression are
combined and pending. Cargo sources match the current lock: enumflags2 was
already vendored. Required four x86_64 runtime/BaseApp refs are installed.
Preflight syntactic-pin checks cannot tell the historical app pin is semantically
wrong for 0.5.5; review identified that additional blocker.

Changes in this preparation: Task List; completion notes; COSMIC submission audit
and engineering index; host-command inventory; Flatpak packaging guide; expanded
v0.5.5 draft release notes; Debian changelog change summary; AppStream development
0.5.5 entry. Earlier user changes remain; Push Buttons SHA-256 is unchanged at
0c15d440ede70c679a634655efa8b1607022d70744a45b399e1e9f9594ebde3f.

Verification:
- bash scripts/build-flatpak.sh --preflight --manifest packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json --arch x86_64 --branch master:
  exit 0; target/task-runs/flatpak-preflight-haf64edw/summary.json and preflight.json.
  ready_for_candidate_build=false, dirty packaging blocker; acceptance not run.
- python3 tests/flatpak_cargo_sources_test.py: exit 0, two tests pass.
- bash scripts/dev.sh metadata-check: exit 0;
  target/task-runs/dev-metadata-check-w7ssoop6/summary.json.
- python3 tests/repo_hygiene_test.py: exit 0, four tests pass.
- Scoped git diff --check: exit 0. Existing Task List whitespace is preserved.
- bash scripts/check-release-publication.sh v0.5.5: refused publication as
  expected because AppStream remains development (script exit 1; the combined
  inspection shell later returned 0 from sha256sum). Draft release notes also
  remain intentionally unfinalized pending package acceptance.
- Unchanged Rust/source verification reuses
  target/task-runs/dev-gate-sbk8qk3q/summary.json. No Rust edits or repeated gate.

No build, installation, service operation, commit, push, tag, dispatch or
publication occurred. Next requires explicit commit approval under AGENTS.md:
freeze current intended 0.5.5 source/tests/docs (exclude operator Push Buttons),
then pin that app SHA in the manifest and freeze packaging separately. Review
exact source/package identities and use one approved build set; later acceptance
receipts must not move tags or silently rebuild. Release notes/AppStream must
be finalized before the final immutable build revision, with package outcomes
reported separately rather than editing accepted artifact identity afterward.


## 0.5.5 commit authorization — October 22, 2026

Operator explicitly approves committing the prepared 0.5.5 source/tests/records
and then the Flatpak application pin in a separate packaging commit. Preserve
Task List edits and leave Push Buttons untracked. Pre-commit tracked scope and
empty index inspected; scoped whitespace check passes outside preserved operator
Task List whitespace. Source gate, metadata and vendor/hygiene results above
retain their original scope. This authorization includes no push, tag, hosted
build dispatch, installation or publication. Development AppStream/draft notes
remain explicit; these commits do not constitute a final release tag.


## Final 0.5.5 release metadata — October 22, 2026

Operator requests finalization after approving the 0.5.5 commit sequence. Updated
AppStream entry to stable 0.5.5 dated October 22, Debian changelog to unstable
with the current Phoenix timestamp, and release notes/index to prepared status.
Notes distinguish finalized change descriptions from unperformed final package
acceptance/publication; they include host minimums, planned target matrix, large
Google/Box root limits and open SharePoint whole-library validation. No package
acceptance has been invented by removing the development labels.

Operator confirms personally removing Push Buttons after its inventory was
incorporated into engineering documentation; its absence is intentional. Earlier
failed final hash inspection therefore does not indicate agent deletion.
Source commit b43f316 and packaging commit ef9868f both succeeded (exit 0).

Checks after finalization:
- bash scripts/dev.sh metadata-check: exit 0;
  target/task-runs/dev-metadata-check-jabc3e8s/summary.json.
- bash scripts/check-release-publication.sh v0.5.5: exit 0; version/notes only,
  explicitly reports acceptance is a manual gate.
- python3 tests/repo_hygiene_test.py: exit 0, four tests pass.
- Scoped git diff --check: exit 0. Preserved Task List whitespace unchanged.
No Rust changes; reuse source gate dev-gate-sbk8qk3q at its recorded scope.

Commit finalized app metadata under the approved 0.5.5 scope, then update the
manifest to that exact full app SHA in a separate packaging commit before any
build. Build/installation/tag/push/publication approvals remain separate.


## Final metadata pin refreshed — October 22, 2026

Metadata commit 13e204ec93049587611978c0cfbad0f9e0a5968c succeeded (exit 0). Main
Flatpak manifest pins that exact app commit; packaging guide, readiness audit
and task status reflect this source identity. Inspection of the pinned Git
objects confirms stable AppStream 0.5.5 and finalized versioned notes (exit 0).
Vendor parity (two tests), repository hygiene (four tests) and scoped whitespace
checks pass (all exit 0). Existing metadata check jabc3e8s remains valid for
unchanged AppStream input; no repeated Rust gate. Record packaging in a separate
approved commit. Final worktree/source identity does not establish package
acceptance. No push, tag, hosted dispatch, build, install or publication occurred.
Next: approved single Debian/Flatpak build set, followed by package inspection
and the combined installed upgrade/regression checklist.


## 0.5.5 push and hosted dispatch — October 22, 2026

Operator explicitly authorizes push and dispatch. Fetched origin/master (exit 0),
verified no upstream divergence (0 behind, 4 ahead), pushed the four commits
(exit 0), and verified remote refs/heads/master equals packaging revision
cd1b465160d692052f32dbb53010583c8ed11a19. Main manifest pins application source
13e204ec93049587611978c0cfbad0f9e0a5968c. No tag or release publication occurred.

Debian dispatch/tracking uses the maintained run_github_linux_build.sh with
--ref master --repo uutzinger/cosmic-ext-applet-mounter. Its request ID is
6cbcf7a09341473dbcf096cb96494205; run 39842941118, six Ubuntu/CPU targets. The
wrapper continues its own bounded polling/download sequence; do not redispatch.
Resume by explicit run ID/receipt if needed, rather than starting another matrix.

Dispatched the maintained build-flatpak.yml workflow once through gh workflow
run with architecture=both and request_id=release-055-cd1b465-20261022 (exit 0).
Selected run 39842934431, x86_64/aarch64. A single gh run list inspection (exit 0)
confirms both runs are in progress at exactly cd1b465. No build success, final
artifact identity or installed acceptance is claimed yet. GitHub Pages also
completed automatically on push; it is not the application release publication.

Only task/evidence notes changed after the frozen build revision. Keep these
later observations separate from that immutable source; no rebuild merely to
include them. Source tests were not repeated for push/dispatch. Next: inspect
producer summaries/artifact coverage when the runs finish, then approve/install
the selected bytes for the combined package-upgrade/regression checks.


## Hosted Flatpak candidates verified; Debian progressing — October 22, 2026

Read run 39842934431 and its artifact inventory: both x86_64 and aarch64 jobs
succeeded at packaging cd1b465; both bundle artifacts are unexpired. Maintained
python3 scripts/promote_release.py v0.5.5 --inspect-flatpaks --repo
uutzinger/cosmic-ext-applet-mounter --flatpak-run x86_64=39842934431
--flatpak-run aarch64=39842934431 completed successfully (exit 0).

Receipt target/task-runs/release-promotion-4fdzna92/summary.json; artifacts.json
and FLATPAK-PROVENANCE.json retain hashes/source/runtime/producer evidence. Both
bundles are retained in its flatpak-artifacts/flatpak-<arch>/ directories.
Verified bundle SHA256:
- x86_64: a6110a0a33f516caf97ffbc971e54f723e35fec49adb2a3bf2e92893011e52b1
- aarch64: 191ec80e73a69d26617a6a7e627e1335e87afa3b3bf026dc4d56409c10cb80d9
These are inspected existing candidates, not installed or publication-accepted.
No build was repeated, no tag created, and nothing installed or published.

Debian run 39842941118 observed at the same packaging SHA: Ubuntu 22.04/24.04/
26.04 arm64 all passed; three amd64 jobs still in progress. Maintained tracker
session remains active, receipt target/task-runs/hosted-build-5k1nfh5i/summary.json.
Do not redispatch or wrap its polling in another CI polling loop. Next: finish
Debian artifact verification and select an inspected package for approved
installation/combined upgrade checks. Task/evidence notes only changed; scoped
completion-notes whitespace check exited 0. No source tests repeated.


## Laptop Flatpak removal; native installation preserved — October 22, 2026

Operator explicitly requests removing the laptop Flatpak app so it is absent
from panel selection and Store updates, retaining the working just install-user
installation. Future Flatpak acceptance will use another computer. Read-only
inspection found only user Flatpak 0.5.4, not running, and zero managed service
files invoking this Flatpak. Native launcher uses the native executable.

Reviewed flatpak uninstall help, then executed the approved package-manager
operation: flatpak uninstall --user --app --no-related --noninteractive
app/io.github.uutzinger.cosmic-ext-applet-mounter/x86_64/master (exit 0).
No --delete-data, --unused, --all or forced removal. No runtime/service/panel
restart was performed. This normal package-manager operation is the explicit
operator-requested uninstall, not a candidate installer or release publication.

Post-operation checks pass (exit 0): app absent from both user/system installed
Flatpak app lists, exported Flatpak desktop entry removed, app data directory
retained, and native executable/desktop launcher SHA256 unchanged. No cloud
data or native configuration was changed. Store/panel rendered cache refresh
was not observed; their next refresh should remove the stale choice/update.
Only this evidence record changed in the repository; no source tests required.
Completion-notes whitespace validation exited 0.


## Ubuntu 26.04 amd64 dependency-install stall — October 22, 2026

Operator reports unchanged Azure mirror cmake download line (14.1 MB). Read-only
GitHub job inspection confirms Debian run 39842941118 now has five successful
targets; only Ubuntu 26.04 amd64 job 120048575388 remains in progress. It entered
Install native and Debian build dependencies at 19:46:07 UTC and was still there
at approximately 20:08 UTC. Rust toolchain and compilation have not started.
This suggests an external APT mirror/download stall, not an observed application
build failure. Running-job logs are unavailable via gh run view --log (exit 1:
job still in progress); API job/run inspections succeeded (exit 0). No precise
network cause is established and no CLI-log progress comparison is possible yet.
No run cancellation or retry occurred. Preserve all five successful targets if
operator authorizes recovery of only the stuck job. Existing tracker remains
active; do not start a second full matrix. Only evidence notes changed; no
source tests repeated. Scoped whitespace validation exited 0.


## Approved single-target Debian retry — October 22, 2026

Operator explicitly authorizes stopping and retrying the slow target. Read
current run/job identity and CLI help, requested cancellation of run 39842941118
(exit 0), and waited through gh run watch until cancelled (exit 1, expected
non-success cancellation). Only Ubuntu 26.04 amd64 was unfinished; it ended
after 38m44s during APT dependencies, before Rust installation or compilation.
Five completed jobs remained successful.

Initial gh run rerun supplied both run ID and --job and was rejected locally
(exit 1: specify only one); no dispatch occurred. Corrected command gh run rerun
--repo uutzinger/cosmic-ext-applet-mounter --job 120048575388 succeeded (exit 0).
Read-only verification confirms same run, attempt 2, unchanged packaging SHA
cd1b465160d692052f32dbb53010583c8ed11a19. Only Ubuntu 26.04 amd64 is in progress
(new job 120068845122); all five other latest jobs are completed/success, not a
new full matrix. Prior Flatpak builds are untouched. The existing maintained
Debian tracker observed queued retry and continues; no second tracking/build
wrapper started.

Task/evidence records only changed. No source/package inputs altered, no tests
repeated, no installation/tag/publication. Scoped notes whitespace check exited
0. Next: wait for this retry, inspect its result and verify retained six-target
artifacts, not another automatic retry or full matrix.


## README aligned with 0.5.5 — October 22, 2026

Operator requests updating the main README. Read its current worktree and diff
before editing: operator had simplified the engine table and screenshot layout,
removed older transition paragraphs, and replaced/added screenshot assets.
Preserved those choices and all screenshot file bytes. Updated popup HTML link
to the operator's new case-sensitive PopUp.png and removed an unmatched td close.
New image remains operator-provided/untracked; no screenshot was generated or
visually accepted by this text review.

README now describes current Settings/Add/Modify action labels, shared-remote
Confirm Save, required OneDrive Offline account type/email, paused initial mirror
workflow and sync interval, Google root-first preload, browser authentication,
private OAuth recommendation, Settings remote inventory/removal, retired legacy
profiles and replacement data preservation. Host minimums match the dependency
guide. Published 0.5.4 downloads are distinguished from unpublished 0.5.5
candidates; COSMIC repository PR remains later. Flatpak uninstall example is
user-scoped, app-only and retains related runtimes/data. No uninstall executed.

Checks: python3 tests/repo_hygiene_test.py exits 0 (four tests). Focused read-only
assertions exit 0 for local HTML image/link targets, balanced three-cell table,
preserved operator three-image layout/widths, absent obsolete labels, and
unchanged hashes of four screenshot files. README and completion-notes whitespace
checks exit 0. No Rust, scripts, translation, manifest or package inputs edited;
no source gate, build or provider test repeated. No rendered README review claimed.

Files edited: README.md, Task List and completion notes. Earlier dirty task/notes
and screenshot edits preserved. No commit/push/tag/publication occurred; hosted
artifacts remain tied to cd1b465, not this later documentation working tree.
Next: review/commit documentation at the appropriate release-record boundary
without silently changing the accepted build/tag source identity.


## Debian six-target artifacts ready — October 22, 2026

Operator reports Debian completion. The original tracker process is no longer
attached (write_stdin reports unknown process); its durable receipt confirms
success, exit 0, phase artifacts-ready, run 39842941118, packaging source
cd1b465160d692052f32dbb53010583c8ed11a19. No second tracker or build started.

Maintained producer summary: target/task-runs/hosted-build-5k1nfh5i/summary.json.
Downloaded six-package set and checksums: target/artifacts/ci-39842941118/receipt.json.
Wrapper completed run/source/artifact coverage, package version/architecture
and checksum verification before recording success. All Ubuntu 22.04/24.04/
26.04 amd64/arm64 packages are retained. Single-target retry succeeded without
rebuilding the five earlier successful targets. Reuse these passing checks; no
new source/test input warrants rerunning them.

Together with release-promotion-4fdzna92, both Flatpak bundles and all six Debian
packages are built/downloaded/inspected. Installed final-package acceptance is
still pending; native just install-user results do not establish this level.
No package installed, tag created or release published. Task/evidence records
updated only; README/screenshot worktree edits preserved. Scoped completion-notes
whitespace check exited 0. Next: transfer the selected exact artifacts to the
test machine for the combined installed upgrade/retired-profile and sandbox
checks; keep the laptop native installation intact.


## README-only commit — October 22, 2026

Operator explicitly requests committing README changes. Source/index/worktree
review confirms the index initially empty. Commit 5ad29ab succeeded (exit 0),
containing README.md only. Kept existing tracked Popup.png reference so the
commit has no missing untracked image dependency. User screenshot edits and
PopUp.png remain unchanged and unstaged; rendered inspection found organization/
VPN-profile labels, so those images were not added under the agent-guide
documentation rule. No image edit or sanitization was performed.

After the popup-link adjustment, repository hygiene passes (four tests, exit 0)
and README whitespace check passes (exit 0). Pending Task List/completion notes
remain outside the commit. No push, tag, build or publication. Existing release
artifacts still identify cd1b465; this later documentation commit must not be
silently substituted as their source/tag identity. Scoped notes whitespace
validation exited 0.


## Test-computer Debian panel smoke — October 22, 2026

Operator transferred the Ubuntu 24.04 amd64 package from the verified run set,
followed target installation/version guidance, confirmed version 0.5.5 and now
reports the panel applet loaded and looks as expected. Accept reported installed
Debian version and rendered panel smoke at that scope. The target also had a
Flatpak entry; removal/native-entry guidance was supplied, but removal and
independent runtime-process identity have not been reported. Do not infer them
or full package/upgrade acceptance merely from appearance.

Next focused observation: open Settings from that applet and inspect a saved
connection editor; confirm communication is available and current action labels
render. Then run an appropriate small disposable connection lifecycle, reusing
unchanged native provider tests. Retained legacy-profile/missing-unit acceptance
and final Flatpak testing remain separate. No agent target-machine operation,
source edit or repeated automated test occurred. Task/evidence notes updated;
scoped completion-notes whitespace check exited 0.


## Test-computer Debian Settings/editor smoke — October 22, 2026

After the guided gear-icon/connection-editor check, operator reports all looks
as expected. Accept reported installed Debian Settings/editor rendering and no
communication warning in this check. This extends the panel smoke; it is not
independent process identity, engine lifecycle, legacy upgrade or Flatpak proof.
No new automated check is needed for this live operator observation.

Next: exercise one small disposable Online connection through Test, Mount,
folder listing and Unmount using this installed package. Reuse previous provider
lifecycles rather than repeat every provider. Keep saved-data and legacy-profile
upgrade checks distinct. Task/evidence records only updated, no agent target
operation/source edit. Scoped completion-notes whitespace check exited 0.


## Test-computer Debian Online lifecycle — October 22, 2026

Following the installed-package Online smoke guide, operator reports Test and
Mount succeeded. After the explicit Unmount prompt, operator confirms the toggle
is off. Accept reported Debian Test/Mount/Unmount and final off-toggle behavior.
Provider and file listing/read-write content observations were not separately
reported; do not infer them. No repeat across all providers is required for this
scoped packaging smoke. Installed mirror worker/legacy retirement and Flatpak
acceptance remain distinct. Task/evidence notes only changed, no agent runtime
operation/source edit or repeated automated test. Scoped completion-notes
whitespace validation exited 0. Next: one small installed Debian mirror
Preview/Sync Now check, then final Flatpak acceptance on the test computer.


## Test-computer Debian Offline smoke — October 22, 2026

After the guided small disposable cloud-subfolder file -> Preview -> Sync Now
-> local arrival check with scheduling off, operator reports "test passed".
Accept reported installed Debian manual mirror smoke at that guided scope.
Provider, counts, checksums and scheduled-worker observations were not separately
reported; none are inferred. Panel, Settings/editor and Online/manual Offline
package smoke now have scoped operator acceptance. Retained retired-profile
upgrade/missing-unit handling and final installed Flatpak behavior remain open.

Only task/evidence notes changed; no agent target/runtime/source operation or
repeated automated test. Scoped completion-notes whitespace check exited 0.
Next: transfer the verified existing x86_64 Flatpak bundle to the test computer
and follow the maintained installer/identity/rollback procedure, keeping the
laptop native and preventing competing runtime owners on the test machine.


## Final Flatpak test-computer installation guided — October 22, 2026

Operator agrees to proceed with final Flatpak installed checks on the separate
test computer. Reviewed maintained install-flatpak-release.sh local --bundle
mode: verifies sibling SHA256SUMS, installs user Flathub/Freedesktop 25.08 as
needed, then installs/updates only the selected bundle. Native Debian remains
installed; first stop connections, close Settings and remove the native panel
instance. No competing owners. Guidance transfers the already inspected x86_64
bundle/checksums plus the standalone installer; no rebuild or unpublished-asset
download. Installer needs no repository checkout on the target.

Expected inspected Flatpak OSTree commit:
b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098.
Use flatpak info --user to verify installed identity, then add the Flatpak panel
entry and check flatpak ps rather than infer sandbox identity from appearance.
No target installation or acceptance reported yet. Laptop stays native. No agent
remote communication, install, source edit, test repeat or publication occurred.
Only evidence record appended; scoped whitespace check exited 0.


## Final Flatpak running process and UI smoke — October 22, 2026

Operator supplies target terminal screenshot: the second flatpak ps listing
contains io.github.uutzinger.cosmic-ext-applet-mounter (PID 73240), confirming a
sandbox process rather than inferring transport from identical native UI. After
the guided popup/gear/connection-editor check, operator reports looks okay.
Accept reported Flatpak popup/Settings/editor rendering and no communication
warning in this check. Full installed info/OSTree commit was not visible in the
provided screenshot; retain exact installed identity as separate confirmation.

Next: one disposable Online connection Test/Mount/folder listing/Unmount in this
sandbox, then mirror directories selected via separate Browse grants and a
small Preview/Sync check. Browser/worker/upgrade acceptance remains separate; no
blanket filesystem grants or native fallback inferred. Task/evidence records
only changed. No agent target/source operation or repeated automated tests;
scoped completion-notes whitespace check exited 0.


## Final Flatpak Online lifecycle smoke — October 22, 2026

Following the explicit Test/Re-Test -> Mount -> folder listing -> Unmount/off
toggle guide, operator reports "Passed". Accept reported installed Flatpak
Online access test, mount, visible file listing and unmount/off state at that
scope. Provider/content checksums or writes were not separately reported; no
extra acceptance inferred. Reuse prior native provider tests rather than repeat
all providers. Next: small disposable Offline mirror with local and recovery
directories selected separately via Browse, then Preview/Sync/local file
arrival. Keep background scheduling off for this manual check. Task/evidence
records only updated; no agent target/source operation or repeated automated
test. Scoped completion-notes whitespace check exited 0.


## Final Flatpak business OneDrive mirror deletion — October 22, 2026

Operator identifies the sandbox mirror as business OneDrive and reports Sync
Now completed. In the subsequent observation, a file deleted through the cloud
folder also disappeared locally. Accept reported installed Flatpak mirror sync
completion and cloud-to-local deletion propagation. The report did not separately
confirm initial test-file download, Preview, Browse-grant selection, invocation
timing for deletion, recovery copy, or schedule; do not infer those results.

Next ask the operator to inspect the configured recovery directory for the
deleted test file. This checks preservation independently of propagated deletion
and command completion. Task/evidence records only updated; no agent cloud/data
operation, source edit or repeated automated test. Scoped notes whitespace
validation exited 0. Exact installed OSTree identity remains pending.


## Final Flatpak business OneDrive recovery copy — October 22, 2026

After the explicit recovery-directory inspection prompt, operator confirms the
cloud-deleted file is preserved in the local recovery folder. Accept installed
Flatpak business OneDrive recovery-copy presence alongside the previously
observed propagated deletion. File contents, hashes and restoration were not
reported, and no scheduled execution is inferred. Task/evidence notes only
updated; no agent data/runtime/source operation or repeated automated test.
Scoped notes whitespace validation exited 0. Next: confirm exact installed
Flatpak commit before extending acceptance to this inspected release candidate.


## Final Flatpak installed identity confirmed — October 22, 2026

Operator confirms flatpak info --user --show-commit matches the expected
b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098. This binds
reported sandbox UI/Online and business OneDrive manual sync/deletion/recovery
observations to the inspected x86_64 candidate from run 39842934431, packaging
cd1b465 and pinned app source 13e204e. It is not ARM installed acceptance.

Next focused sandbox check is one automatic scheduled business OneDrive download:
place a distinct disposable file in the cloud test subfolder, Start background
scheduling, wait its configured interval plus operation time without Sync Now,
confirm local arrival, then Stop. This checks installed scheduled-worker launch
independently of manual sync. Retained retired-profile upgrade/missing-service
regression remains separate. Task/evidence records only updated; no source or
agent runtime/data operation or repeated automated test. Scoped notes whitespace
check exited 0. No publication or tag.


## Remaining release acceptance reconciled — October 22, 2026

Operator requests skipping the 15-minute Flatpak scheduler check; explicitly
waived, not passed. No observation of the final candidate's scheduled worker is
inferred from manual Sync Now or earlier native timer tests. The request does
not waive other open checks. Mirror off-state after this instruction is not yet
reported; operator was asked to stop it if started.

Built/inspected: six Debian targets and both Flatpak architectures. Installed
reported smoke: Debian panel/Settings/editor, Online Test/Mount/Unmount and manual
Offline flow; exact x86_64 Flatpak identity, UI/Online lifecycle, manual business
OneDrive sync, cloud deletion/local propagation and recovery-copy presence.

Outstanding release review:
- Combined package legacy-profile upgrade/absent-retired-unit regression remains
  open, despite earlier native retirement/data-preservation and fake tests. Test
  a suitable retained/disposable profile or obtain an explicit recorded waiver.
- SharePoint whole-library live acceptance remains open (test small library or
  explicitly retain limit). Google/Box whole-root and final Flatpak timer runs are
  already deliberately skipped. Fresh repaired Google/Box browser consent in this
  exact sandbox package and installed ARM acceptance are unrun; reuse only prior
  evidence at original scope and disclose those limits in final acceptance.
- Final acceptance receipt/source/tag selection must retain artifact identity:
  packages built at cd1b465, Flatpak app source 13e204e. Later README commit
  5ad29ab is not those artifacts' packaging SHA. Tagging that later commit would
  fail exact-source promotion; do not rebuild merely for README/receipt inclusion.
  Keep documentation updates separate from the accepted built revision unless
  operator changes release source and explicitly approves new affected builds.
- After acceptance/limits resolved, obtain tag/publication approval, promote the
  existing bytes and verify public asset checksums/coverage. COSMIC PR remains
  later, broader expansion backlog deferred.

Task/evidence records updated only; source/build/install/publish untouched.
Scoped completion-notes whitespace check exited 0. No tests repeated for this
reconciliation. Nothing additionally marked installed-passed or published.


## Approved push and v0.5.5 tag preparation — October 22, 2026

Operator explicitly requests push and tag after the remaining acceptance review.
This does not confer publication approval or silently waive pending package
legacy/SharePoint/other limits. Fetched origin/master: no new upstream commits,
local README one ahead. Retain built packaging cd1b465 as tag identity, with
later README/evidence on master rather than rebuilding for documentation.

The maintained release helper requires a clean branch whose remote points to
the build SHA. Created isolated worktree /tmp/cloud-mounter-release-v0.5.5 with
branch release/0.5.5 at cd1b465160d692052f32dbb53010583c8ed11a19 and pushed that
branch (exit 0), preserving main-worktree screenshot edits. Ran approved
scripts/build_release.sh --repo uutzinger/cosmic-ext-applet-mounter
--run-id 39842941118 --download-dir <original verified ci-39842941118 directory>
--log-dir <target/task-runs/release-prepare-v055> from the clean checkout.

Helper succeeded (exit 0), created/pushed annotated v0.5.5 at cd1b465, validated
the existing successful run, and reused the six checksum/identity-verified Debian
packages. Receipt target/task-runs/release-prepare-v055/summary.json; phase
artifacts-ready. No new matrix, compilation, install or publication occurred.
Flatpak run 39842934431 also has packaging cd1b465 and app pin 13e204e.

Commit only pending Task List/completion notes for master under the approved
release-record scope; README 5ad29ab is already committed. Leave all screenshot
edits/untracked PopUp.png outside the commit. Fetch immediately before master
push and verify remote branch/tag identities. Source checks are reused; scoped
notes whitespace check exits 0. GitHub release publication remains unapproved.


## v0.5.5 published and public bytes verified — October 22, 2026

Operator requests completing the last publication step after push/tag. Ran the
maintained promote_release.py with v0.5.5, Debian run 39842941118 and both
Flatpak architectures from run 39842934431, --publish --accepted, in the clean
tagged checkout. Exit 0; source cd1b465. Existing verified artifacts reused,
no build. Producer receipt /tmp/cloud-mounter-release-v0.5.5/target/task-runs/
release-promotion-e8zpajq5/summary.json and artifacts.json.

Public GitHub release v0.5.5 is neither draft nor prerelease. Six Debian packages,
two Flatpak bundles, FLATPAK-PROVENANCE.json and SHA256SUMS present. GitHub returned
no asset digests, so downloaded all ten public assets to
target/artifacts/v0.5.5-public-verification and compared every SHA256 against the
publication receipt: all match (exit 0). No installed coverage beyond recorded
operator observations is inferred; previously disclosed limits remain. COSMIC
repository PR remains later. Source tag was not moved. Later master README and
acceptance records remain distinct from the tagged package source.


## Validation/expansion backlog reconciled — October 22, 2026

Operator requests reconciliation following stale-item review. Updated only the
backlog section in Task List, preserving original October 1 planning chronology
and the existing 0.5.5 heading. Removed shipped/not-yet-existing prerequisites;
separated future COSMIC Store channel checks from already available GitHub
Flatpak testing. Narrowed UI checks to residual coverage, citing scoped native
keyboard/theme/Save and packaged UI acceptance rather than closing broad items.
Kept larger/shared-drive, provider errors, cancellation, MFA, advanced OneDrive,
watcher evaluation and suspend-fault work open. Replaced the stale sleep gate
reference with historical audit/archive links. Final scheduled-worker skip stays
a limitation, not a pass. Advanced target testing does not expand supported
OneDrive semantics. No original open scenario was silently marked completed.

Pre-reconciliation backlog retained below as a chronological wording snapshot;
the active Task List above remains authoritative for current status.

```markdown
## 0.5.5 validation and expansion backlog

Deferred validation and functional expansion planned after the 0.5.4 release.
Originally recorded October 1, 2026 as follow-ups to the native SFTP release; they
are not gates for any current release. Move one into a release gate only if its
supported behavior changes or testing exposes a related defect.

- [ ] After the public COSMIC Flatpak repository/package is available, test
  installed Flatpak SFTP editor notices, Online and Offline controls, generated
  host services/timers, and a clean-profile mount/mirror/VPN/uninstall smoke
  test, including OneDrive (onedriver Online, rclone Offline). The local Flatpak
  prototype and host bridge have separate evidence.
- [ ] If later needed, broaden installed SFTP error-notice checks to host-key,
  unreadable-file, permission, agent, and timeout variants, and live-check a
  VPN-bound SFTP connection. Direct protocol and automated classifier tests
  already cover these errors; they are not repeat gates for the native release.
- [ ] Broaden Google Drive `--fast-list` testing to a larger ordinary drive and
  shared drive, including API/rate-limit behavior; repeat cloud cold-cache
  preload trials without reading or changing file contents.
- [ ] Live-check Box and SMB preload on the UA and LAN shares, and shared
  cancellation on OneDrive (onedriver), Box, SMB, and Google Drive's RC refresh when
  suitable active-preload shares are available.
- [ ] Complete broader installed native/Flatpak Settings title, focus,
  keyboard, scaling, long-list, editor-save, and background-poll visual checks,
  including the rclone-based OneDrive Offline setup editor once it exists.
- [ ] Exercise the full Cisco/NetworkManager password/MFA, concurrent-request,
  and rejected-credential path with a real VPN profile; add the separately
  planned waiting/Cancel/Retry UI before accepting that extended workflow.
  Include a VPN-bound SFTP connection when that combination is available.
- [ ] After the rclone OneDrive Offline mirror ships, broaden its testing: very
  large trees, shared-with-me and SharePoint-backed libraries, personal vault,
  Graph throttling (HTTP 429) and retry behavior, Office-document metadata changes,
  and tenant conditional-access/consent policies.
- [ ] Evaluate whether a debounced local-change trigger (starting one scheduled
  bisync after local edits) is worth a separate watcher, since scheduled polling
  replaces abraunegg's immediate local upload. Measure CPU/power first; do not add
  a second writer to a mirror.
- [ ] Exercise repeated logind/inhibitor and native/Flatpak lid/menu suspend
  fault cases, then investigate the separately reported onedriver (OneDrive
  Online) sleep hang with journal evidence. The native SFTP release gate above includes one
  normal sleep/wake cycle only.
```

Only Task List and completion notes edited. Existing publication receipt and
operator screenshot edits preserved. No source/runtime action, test repetition,
commit or push. Repository hygiene and scoped whitespace checks reported below.

Validation: python3 tests/repo_hygiene_test.py exits 0 (four checks);
scoped git diff --check for Task List and completion notes exits 0. No broader
source gate repeated for this documentation-only reconciliation.


## Backlog moved after Flatpak PR — October 22, 2026

Operator explicitly requests prioritizing Flatpak PR before validation/expansion
in Task List. Moved the entire reconciled backlog after the full Flatpak PR
section and before the Future Task List. Both sections retain their exact text,
status markers and original planning dates; prefix and future sections remain
unchanged. Section/content preservation assertions pass (exit 0). User-requested
priority order supersedes the default chronological placement for this move.
No scope, acceptance or implementation changed. Only task/evidence records edited;
no commit/push/runtime action or source test repeated.


## Published 0.5.5 COSMIC baseline refresh — October 22, 2026

Operator requests refreshing manifest identity, host-command inventory, dependency
and permission rationale. Current manifest already pins published app 13e204e;
no manifest/source/permission change needed. Added current published baseline to
COSMIC audit ahead of dated historical candidate facts, updated host inventory,
packaging guide and engineering index, and marked the scoped refresh complete
in Task List. Replaced obsolete retained-legacy dependency answer with retirement
behavior. No PR description or external submission was authored.

Current tag/packaging cd1b465, app pin full 13e204e, published Flatpak run
39842934431 and installed x86_64 OSTree commit are distinguished. Manifest/vendor
bytes equal tagged inputs and verified publication hashes (assertions exit 0);
23 grants unchanged. Typed runner inspection confirms eight shipping variants,
with legacy onedrive absent. Per-family retained ownership/removal criteria
remain applicable. Dependency minimums match Debian and dependency guide; Python
is build/development tooling, not new runtime payload. No rebuild is warranted.

Built metadata inspection includes inherited x11 in addition to wayland/fallback
x11. This is documented without pretending 23 manifest declarations fully
describe effective policy or OS-restrict host execution. Remote installed
overrides were not independently captured. Exact x86_64 smoke and manual mirror
delete/recovery pass; ARM installed, final timer, legacy-package regression,
SharePoint whole-library and fresh final-sandbox Google/Box consent retain their
explicit limits. Published release is not COSMIC maintainer acceptance.

Checks: source-pin/hash/tag-input assertions exit 0; python3
tests/repo_hygiene_test.py exits 0 (four checks); scoped whitespace exits 0.
Earlier vendor/source tests reused because inputs unchanged. No source/runtime/
package operation, commit, push, tag or submission occurred in this refresh.
Operator screenshot edits and earlier records preserved. Next: current COSMIC
rules recheck and review proposed manifest placement/maintainer questions before
the operator authors a contribution.


## Current host-command exemption summary clarified — October 22, 2026

Operator needs the current exception inventory for independently authored PR,
not the D-Bus migration checklist. Existing authoritative file remains
docs/engineering/flatpak-host-command-inventory.md. Added a prominently named
Current 0.5.5 exemption summary with eight retained runner identities, purpose,
source owner/boundary and replacement/removal condition; separately disclosed
fixed worker/guard and conditional OpenConnect agent service payloads. Clearly
marked historical October 19 engine table as superseded for exemption selection,
including retired legacy onedrive. Removed/D-Bus-replaced commands are not new
exception requests. Proposed justification is not maintainer approval.

No manifest/command/runtime change, PR prose, submission or package operation.
Four repository-hygiene checks and inventory whitespace pass (exit 0). Current
source/published identity checks from preceding refresh reused; no repeated
source gate. User edits and prior records preserved. Not committed or pushed.


## Root architecture/source design guide — October 22, 2026

Operator explicitly requests root Design.md explaining structure, mount/mirror
solutions, host mechanisms and a grouped source inventory for software engineers.
That explicit location overrides the default engineering-record directory rule;
linked from the engineering index and README without duplicating the guide.
Read Cargo/main/lib, per-module interfaces, key provider/coordinator/worker, IPC,
configuration, portal, service and command paths before writing. No full product
specification was loaded or changed.

Created Design.md with purpose/mode comparison, process/runtime ownership,
configuration versus observed state, Online lifecycle, shared manual/scheduled
Offline transaction and guards, host D-Bus/portal/process mechanisms, distinct
host/authorized local paths, persistence ownership, authentication/VPN/sleep,
retired profiles, design tradeoffs and change guidance. Two Mermaid diagrams
show component boundaries and mirror execution. Source map groups all 36 Rust
files, including development-only orchestration, without moving source files.
Describes actual IPC JSON-over-D-Bus rather than claiming generated typed wire
methods, and broad host gateway authority rather than an OS executable allowlist.
Preview app-state mutation, private RC socket auth boundary and unrun acceptance
limits are explicit; architecture description is not proof of every live path.

Files edited: new Design.md, README, engineering index, Task List and completion
notes. Existing documentation/screenshot changes preserved; no runtime/source
code or package input changed and no new build/gate required. Verification:
- Full source-set equality and uniqueness: all 36 src/**/*.rs entries represented
  once (exit 0), with Design local links and fenced diagrams structurally valid.
- python3 tests/repo_hygiene_test.py: exit 0, four checks pass.
- Scoped tracked-document git diff --check: exit 0. New-file no-index check
  emitted no whitespace diagnostics (exit 1 is the ordinary new-file difference).
- Final scoped notes/README/index whitespace validation: exit 0.
Diagrams/Markdown were not independently rendered; no new engine, service, UI or
package acceptance claimed. No commit, push or PR submission performed.


## Release installer default 0.5.5 — October 23, 2026

Operator requests updating installer version. Changed both standalone Debian and
Flatpak GitHub-release installers' default and help from 0.5.4 to published
0.5.5. Explicit --version and Flatpak local --bundle paths remain unchanged.
Updated script catalog/current inventory while preserving dated 0.5.4 history.
Existing Debian fixtures now supply 0.5.5 package identity; supported-target tests
assert the actual default checksum/package download URLs select v0.5.5. Existing
Flatpak CPU tests assert default v0.5.5 URLs; explicit override test now selects
v0.5.4 to prove a different version is still honored.

Checks: python3 tests/debian_installer_test.py exit 0 (5 tests, all supported
OS/CPU combinations and verification/APT-failure cases); python3
tests/flatpak_release_installer_test.py exit 0 (6 tests, both CPU mappings,
verification failures, partial changes, explicit override and local-bundle mode).
bash -n for both scripts exit 0; scoped diff whitespace exit 0. All adapters fake
network/package commands; no live install, runtime/repository change, app build
or publication. No Rust changes or broader source gate warranted. Existing user
documents, PR file and screenshots preserved. Not committed or pushed; online
master download defaults do not change until these scripts are pushed.


## README published 0.5.5 wording corrected — October 23, 2026

Operator asks that README refer to 0.5.5. Preserved their current v0.5.5 release
link and removed contradictory preparation/unpublished-candidate wording. README
now states publication October 22 and that the updated installers default to
0.5.5 with explicit version overrides. COSMIC PR remains planned, not accepted.
Existing Design link/layout/user edits preserved. Repository hygiene passes
(four checks, exit 0); README whitespace passes (exit 0), and targeted search
finds no remaining 0.5.4/unpublished reference. No source/package/runtime action,
commit or push. Changes remain local alongside the installer updates.


## Documentation and 0.5.5 installer commit/push authorized — October 23, 2026

Operator explicitly asks to add, commit and push pending updates. Scope: README,
root Design, task/evidence records, engineering index/readiness/host-interaction
justification, Flatpak packaging guide, script catalog/inventory, both download
installer defaults and focused tests. Exclude operator-authored PR draft and
all screenshot modifications/untracked image. Preserve operator README/inventory
edits without reformatting; one trailing-space diagnostic in operator inventory
prose is intentionally retained.

Upstream fetch finds local and origin/master equal at 1b630d6. No staged files
preceded this operation. Existing 11 focused installer tests and shell syntax
checks pass at unchanged script/test inputs; no repeat warranted. Current four
repository hygiene tests and source-inventory/default-version assertions pass
(exit 0). Source manifest/pins/runtime unchanged; no app rebuild or tag movement.
Commit approved intended files, fetch again before push, verify remote identity.
This does not publish another release, submit a COSMIC PR or add screenshot data.

## Latest README push preparation — October 23, 2026

Operator requests pushing the latest README. Reviewed its screenshot width,
Google Drive engine-link simplification and separated backup warning; removed
one trailing space. Scope is README and required task/evidence records only.
Screenshot changes and the PR draft remain local.

Verification at source a600e562ae2e25defbcda33ebed583453ee3fc8d (dirty):
- `git fetch origin`: initial exit 255 because Git metadata was read-only;
  authorized retry exit 0. `git rev-list --left-right --count HEAD...origin/master`
  exit 0, result 0/0.
- `git diff --check -- README.md 'Task List.md'`: exit 0.
- `bash scripts/dev.sh gate`: initial exit 1 because the sandbox blocked the
  just runtime directory; authorized retry exit 0, status succeeded,
  source_dirty true. Receipt `target/task-runs/dev-gate-1izh04qa/summary.json`,
  log `target/task-runs/dev-gate-1izh04qa/task.log`; blocked attempt recorded in
  `target/task-runs/dev-gate-ht2dyvgy/summary.json`.

Source checks and unit/fake-adapter tests only; no installed or rendered UI
acceptance is claimed. Next: commit the three intended documents, fetch again
immediately before push, inspect divergence and verify the remote commit.

## Settings screenshot update — October 23, 2026

Operator explicitly authorizes adding and pushing the updated Settings picture.
Scope: `resources/Settings.png` and required task/evidence records. Viewed the
472 x 657 image; no credentials or organization names are visible. PNG signature,
all chunk CRCs and IDAT decompression pass (Python validation, exit 0). SHA-256:
`cb44851fb29fc6d8ecb1353e69a9590ab4e189b0ebd109550dfe3d91ccb59b02`.

Reused the successful `dev-gate` receipt from source
`a600e562ae2e25defbcda33ebed583453ee3fc8d`, source_dirty true, exit 0:
`target/task-runs/dev-gate-1izh04qa/summary.json`, log
`target/task-runs/dev-gate-1izh04qa/task.log`. Since that check only README and
task records were committed; Rust/script/test inputs remain unchanged, and no
test or build recipe references this screenshot. No repeat of passing checks.
This verifies the image artifact, not installed applet behavior.

Upstream fetch and divergence check exit 0, result 0/0. Next: commit intended
files, fetch again immediately before push and verify remote identity. Leave
Add Connection/PopUp image changes and the PR draft local.

## Minor documentation and screenshot push preparation — October 23, 2026

Operator authorizes documentation, screenshots and the PR draft together.
Included the operator's build guide/OpenConnect deletions, Add Connection image,
new PopUp image and PR draft. Corrected the task-list reference to the draft's
actual filename and pointed README at the new PopUp image. Preserved PR draft
wording and existing whitespace; removed one new build-guide blank line at EOF.
This commits a draft only; it does not submit a repository PR.

Checks (exit 0): PNG signature/chunk CRC/decompression validation for both
screenshots; visual image inspection; `python3 tests/repo_hygiene_test.py`
(four passing tests); `git diff --check` for tracked edits; upstream fetch and
divergence check (0/0). The initial whitespace check found the extra build-guide
EOF line; it was removed and the check passed. Draft whitespace is retained as
operator-authored text. No new runtime or installed behavior is claimed.

Reuse gate receipt `target/task-runs/dev-gate-1izh04qa/summary.json`, source
`a600e562ae2e25defbcda33ebed583453ee3fc8d`, source_dirty true, succeeded, exit 0,
log `target/task-runs/dev-gate-1izh04qa/task.log`: Rust/script/test inputs remain
unchanged. Rechecked the affected documentation-link tests separately above.
Next: commit this scope, fetch immediately before push, reconcile if necessary
and verify remote identity.

## COSMIC contribution-rule recheck — October 23, 2026

Completed the specifically requested rules/evidence review. Read release and
packaging procedures and engineering-record rules. Updated the existing
submission audit, engineering index and task status; operator PR draft unchanged
(SHA256 `4113a6835cc849472238d73b2641a0afb5b1bde926b27b29366ad57b02df49b0`).

Browser results included cached pages and raw recipe/API fetch failures. Initial
direct curl exited 6 (sandbox DNS); authorized direct retry exited 0 and resolved
cosmic-flatpak master to f702d691ce9060f50c2908d36a7d83b6676703a8. Downloaded
README/template/recipe at that immutable revision plus linked contributor,
trademark and DCO sources (exit 0). URL/hash index and snapshots:
`target/cosmic-contribution-review-2026-10-23/sources.json`. The current recipe
still reads app/<id>/<id>.json; no destination checkout build was run.

Template retains the own-app source/manifest AI exception; general guide still
excludes AI contributions. Operator authors PR/replies and personally certifies
the actual diff. The existing draft's AI-assisted appendix needs operator review
against that exception. No rewrite, acknowledgment or external message performed.

Reused successful Flatpak inspection and release preparation receipts and scoped
October 22 installed observations, linked in the audit. Reread local published
provenance; current manifest/cargo-source hashes match both architectures and
v0.5.5. No source/runtime/grant change or rebuild justified. Preserved installed
limits: skipped final scheduler and whole-root cases; unrun sandbox browser,
package retirement regression, whole-library SharePoint and installed ARM;
recovery-copy presence does not establish restoration. Maintainer host/runtime/
dependency acceptance and destination repository integration remain open.

Validation: `python3 tests/repo_hygiene_test.py`, four tests, exit 0;
`git diff --check`, exit 0; provenance/hash/draft/code-gate-input assertions,
exit 0, `target/cosmic-contribution-review-2026-10-23/checks.json`.
Reuse `target/task-runs/dev-gate-1izh04qa/summary.json` (source a600e562,
source_dirty true, succeeded, exit 0; sibling task.log): no Rust/script/test/
justfile inputs changed since that run. Only affected documentation checks ran.
This is a source/document review, not new installed or package acceptance.
Next: operator reviews the actual submission and acknowledgments; prepare the
destination candidate separately, retaining these acceptance limits. No commit,
push, build dispatch, install or submission in this task.

## Pre-submission acceptance priorities added — October 23, 2026

Operator requests placing the discussed acceptance priorities immediately before
the 0.5.5 Flatpak PR/AI section. Added scheduled sync, fresh Google/Box consent
and effective overrides as highest priorities; retained package-upgrade,
SharePoint and recovery/fault checks or explicit disclosure. ARM64 installed
testing is deliberately skipped because the operator has no ARM64 computer;
built/inspected status is not an installed pass. Historical skipped checks remain
unchanged. These are project priorities, not a claimed mandatory COSMIC test list.

Scoped whitespace and placement/status assertions pass (exit 0). No code or
test inputs changed; reuse the passing gate receipt documented in the preceding
review. No live tests, commit or push. Next: execute the selected installed
checks and record their actual results and limits.

## VM personal OneDrive scheduled-sync check started — October 24, 2026

Operator offers the existing personal OneDrive mirror on the virtual machine for
scheduled synchronization testing. Begin by confirming the installed Flatpak
commit, running sandbox process and configured interval. Use only a newly
created disposable text file in the mirrored scope; do not infer scheduled
execution from Sync Now, an initial Start-triggered sync or an active timer.
No test result or changed VM state is reported yet. Earlier skipped-test history
remains unchanged; current acceptance item stays open. No source, installation,
build, commit or push action. Next: collect identity/interval, then observe a
file created after any immediate sync has finished arriving on a later schedule.

## VM Flatpak identity reconfirmed — October 24, 2026

Operator supplies terminal screenshot: `flatpak info --user --show-commit`
reports b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098,
matching the inspected 0.5.5 x86_64 candidate. `flatpak ps` lists the app ID
io.github.uutzinger.cosmic-ext-applet-mounter as a running sandbox process.
Accept installed identity/running-process evidence only; no scheduled transfer
is shown. The personal OneDrive mirror's interval and Started/Stopped state
remain to be supplied before timing the disposable-file check. No VM operation,
source change, build or repeated automated test performed by the agent.

## VM interval edit blocked before scheduler test — October 24, 2026

Operator reports interval was 15 minutes, edited to 5; only Detect, Disable and
Remove are clickable. A saved five-minute configuration and running schedule
are not established. Source action rules can block Test/Save for a retired
legacy profile or invalid/incomplete connection/setup inputs; button availability
alone does not identify the cause. Request the editor's visible blocking reason
and popup toggle state before changing the connection or timing a scheduled run.
No runtime operation or acceptance pass; no source edit or repeated code tests.

## VM interval edit requires identity and folder grants — October 24, 2026

Operator reports entering the expected account email enabled Re-Connect/Re-Test.
Re-Test then required choosing both mirror and recovery folders through Browse;
it passed after those selections. Operator believes selected paths are unchanged;
path equality and saved five-minute interval are not independently confirmed.
Accept reported installed personal OneDrive Re-Test success only, not scheduled
execution or persistent folder-grant reuse.

Source inspection: draft_from_connection deliberately clears the expected email;
an existing test asserts it is not serialized and is blank on reopening. Interval
edits clear validated_draft, and Save requires a matching validated connection.
mirror_access persists folder-picker grant records and revalidates them; a typed
host path alone is not a grant. Missing grant metadata requests Browse selection.
This explains first-time/recovery selection, but does not establish why the VM's
previous folder grants were unavailable. Requiring fresh Browse selection on
every unchanged-path interval edit would warrant investigation; persistence is
intended. No code changes or repeated code tests. Next: Save and reopen to verify
the interval, then continue the scheduled-file test and assess grant persistence.

## VM personal OneDrive folder grants persist — October 24, 2026

Operator reports interval changes to 3 and back to 5 minutes, each allowing
Re-Test/Save after entering the account email without reselecting folders. After
removing the applet from the panel and adding it again, another interval edit
also worked without Browse. Accept reported saved-folder grant reuse across
these edits and panel removal/re-addition on the identified x86_64 Flatpak.
This does not establish persistence across VM reboot or logout.

Operator suspects an earlier folder-path error. The original paths/grant state
were not captured; record the cause as unresolved rather than a confirmed path
typo or recurring permission defect. No source fix is established as necessary.
Scheduled worker execution remains unverified. Next: use a saved five-minute
interval, Start scheduling, allow any current run to finish, create a new
disposable text file through OneDrive web in the mirrored scope, and observe
automatic local arrival/content without Sync Now while the VM stays awake.
No source change, code-test repeat or VM operation by the agent.

## VM personal OneDrive scheduled download and upload accepted — October 24, 2026

Following the scheduled-file procedure, operator reports the schedule completed
in both directions: a new remote file downloaded into the local mirror and a
new local file uploaded to OneDrive. Accept reported scheduled bidirectional
new-file transfer at the five-minute test interval on the VM's identified
0.5.5 x86_64 Flatpak b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098.
This closes the reopened scheduled-worker check for personal OneDrive; retain
the October 22 skipped test as historical evidence. Exact completion timing,
content/hash equality, other providers, deletion/conflict/restore behavior and
reboot persistence were not reported and are not inferred.

Updated active task, future coverage summary, submission audit and engineering
index; preserved earlier reports and operator-authored PR draft. No code,
installation or build changed. Documentation checks: four repository-hygiene
tests and scoped whitespace check, exit 0. Reuse existing passing code gate at
unchanged inputs; this is operator-reported installed acceptance, not an agent
runtime test. Next: capture test-machine effective Flatpak overrides, then fresh
Google/Box sandbox browser-consent checks. No commit or push.

## VM override output observed; scope confirmation pending — October 24, 2026

Operator screenshot shows all four requested user/system, global/app-specific
override commands, followed by one Context block containing only read-only
xdg-config/gtk-3.0, xdg-config/gtk-4.0, xdg-config/kdeglobals and
xdg-data/color-schemes filesystem entries. No broad host/home filesystem or
mirror-directory override is shown. Because the pasted commands are echoed
together without output labels, the block's scope is not independently clear.
Request the system-global command alone to attribute the block before closing
the permission-baseline item. No overrides changed and no new runtime test.

## VM override baseline accepted — October 24, 2026

Operator ran scopes separately to resolve the initial unlabeled output. Reports
system-global output empty; supplies screenshot confirming user-global Context
contains only read-only xdg-config/gtk-3.0, xdg-config/gtk-4.0,
xdg-config/kdeglobals and xdg-data/color-schemes. Then reports no output for both
user and system app-specific commands for the Cloud Mounter app ID. Accept the
four-scope override baseline on the identified VM Flatpak: no broad host/home
or mirror-directory override observed. Portal folder grants are separate; these
commands do not exhaustively enumerate those grants or all effective permissions.

Marked override-capture task complete; updated submission audit and engineering
index. No permissions changed or services touched. Four repository-hygiene tests
and scoped whitespace validation pass (exit 0); reuse the passing code gate at
unchanged code/test inputs. No source, package, build, commit or push action.
Next: fresh Google Drive and Box browser-consent checks in the same Flatpak.

## VM Google authorization and native-document limitation — October 24, 2026

Operator reports fresh Google authorization and Save succeeded, Test passed,
mount succeeded and folders/files were visible. In one folder, an apparent DOCX
was listed at zero bytes and opened empty in LibreOffice, while the web version
had content and no filename extension. After downloading via browser, renaming
and uploading again, the new file showed a DOCX extension on the web and correct
size/content when opened through the mount. Accept reported fresh Google setup,
mount/listing and ordinary uploaded-DOCX access on the identified VM Flatpak.
Record native-file opening as failed/limited, not a successful content test.
Native MIME type is inferred; no Slides/Sheets live result, mount writeback,
unmount or checksum comparison was reported.

Checked official rclone Drive export/limitations documentation and 1.74.3 backend
update implementation; links and scoped findings are in the submission audit.
Source mount plan uses full VFS caching and no import/export or read-only flag.
Do not describe the entire mount as read-only or all native exports as always
unreadable. No remote configuration dump or live engine operation performed.

Split Google consent (passed) from pending Box consent in Task List; retained
Google unmount confirmation as open. Updated README limitations, audit and index.
Four repository-hygiene tests and whitespace validation pass (exit 0). Unchanged
code gate reused; no source/build/install/commit/push or PR-draft changes.

## 0.5.6 Google Drive configuration future task — October 24, 2026

Added the requested 0.5.6 entry in the Future Task List immediately before the
desktop-upgrade-dependent tasks. It plans Microsoft Office, OpenDocument and
browser-link representations of Google-native files. Online mount versus
Offline mirror applicability remains an explicit future discussion; no new
import/editing behavior or mirror support is approved or implemented. Scoped
whitespace and heading-placement checks pass (exit 0). Code/test inputs remain
unchanged; no code-gate repeat, commit or push. Next: agree mode scope before
design and implementation.

## 0.5.5 VM acceptance resumed — October 24, 2026

Operator requests continuing installed/package acceptance. Reuse the recorded
VM Flatpak identity, personal OneDrive scheduled transfers, override baseline
and Google authorization/mount/file observations; do not repeat passed checks.
Next guided checks: confirm Google unmount/off, then fresh Box Online connection
authorization, Test/Save, mount/listing and unmount. No new pass reported yet;
other acceptance and disclosure items remain open. Scoped whitespace check
passes (exit 0); no code/test inputs changed, code gate not repeated. No source,
build, installation, commit, push or PR-draft changes.

## VM Google unmount and Box mounted-file content — October 24, 2026

Operator reports Google unmounted without problems. Mark Google unmount complete.
Following the Box procedure, operator reports opening a test file on the VM and
newly created text matching between the mounted folder and provider web browser.
Accept reported Box mounted/web text agreement; no hash, write direction or
explicit fresh authorization/Test/Save/Box-unmount result was supplied. Request
confirmation of those steps without repeating the completed content check.

Updated task/evidence records, submission audit and index. Four documentation
hygiene tests and whitespace check pass (exit 0); code gate reused at unchanged
inputs. No code, runtime operation by the agent, build, installation, commit,
push or operator PR-draft change. Next: close Box lifecycle evidence, then select
the next remaining acceptance/disclosure item.

## VM Box Re-Test, Save and unmount accepted — October 24, 2026

Operator explicitly confirms Re-Test passed, Save passed and unmount completed
cleanly. Accept these installed Box lifecycle observations alongside the prior
mounted/web text agreement. Fresh Connect/browser consent was not explicitly
confirmed in this response, so retain that one confirmation as pending without
repeating completed tests. Updated task/audit/index and evidence notes; scoped
whitespace validation passes (exit 0). Code/test inputs unchanged; code gate and
documentation link tests reused from the preceding checks. No code, runtime,
build, install, commit, push or PR-draft action. Next: confirm whether a fresh
Box browser authorization was completed during this setup.

## VM fresh Box browser authorization accepted — October 24, 2026

Operator explicitly confirms creating a new connection and completing browser
authorization. Close the fresh Box consent task alongside previously reported
Re-Test, Save, mounted/web text agreement and clean unmount. All items under the
pre-submission highest-priority checklist are now complete at their stated
scope. No extra provider, checksum, reboot or mirror acceptance is inferred.

Updated Task List, submission audit/index and these evidence notes. Four
repository-hygiene tests and whitespace check pass (exit 0); unchanged code
gate reused. No code, VM operation by the agent, build, installation, commit,
push or PR-draft change. Remaining work: package legacy/absent-unit regression,
small SharePoint whole-library mirror or explicit scope limitation, and
recovery-restoration/sleep/VPN fault checks or retained limits. ARM64 installed
testing remains skipped because no test computer is available.

## VM small SharePoint whole-library test started — October 24, 2026

Operator confirms a small SharePoint test library is available. Begin a new
SharePoint Offline mirror on the same identified VM Flatpak, leaving the remote
subfolder empty to select the whole document library. Use new empty local mirror
and separate recovery directories selected through Browse. Connect/verify the
library remote, Test and Save first; keep scheduling paused. Preview and the
confirmed initial Sync Now follow only after setup results are reported. No
setup result or whole-library acceptance claimed yet. Scoped whitespace passes
(exit 0); no source/code-test inputs changed, no code gate repeated. No build,
installation, commit, push or agent operation on the VM or cloud data.

## VM SharePoint whole-library setup accepted — October 24, 2026

Following the guided new SharePoint Offline connection with a blank library
subfolder, separate Browse-selected mirror/recovery directories and
Connect/Test/Save, operator reports setup passed. Accept reported setup at that
scope on the identified VM Flatpak. No Preview, initial transfer, file-content
comparison or scheduling result yet; whole-library mirror acceptance stays open.
Next: run Preview and inspect its expected scope/change summary before initial
Sync Now. Scoped whitespace passes (exit 0); no source/test inputs changed and
no code gate repeated. No agent VM/cloud action, build, install, commit or push.

## VM SharePoint initial Preview observed — October 24, 2026

Operator screenshot reports Preview completed: uploads 1, downloads 2, deletes 0,
conflicts 0, skipped 6; initial sync has not run and requires Sync Now confirmation.
No library/connection name retained here. Accept Preview completion and displayed
counts only; no transfer, content or exact skipped-file coverage inferred.

Source inspection: SharePoint initial-preview preparation creates a local
`.cloud-mounter-sharepoint-anchor-<connection-id>` access marker, so an upload
from a new mirror directory is plausible. Actual queued filename was not shown.
The preview parser counts skip/excluded/filtered log lines, so six skipped does
not establish six distinct excluded user files. Next: verify the local directory
contains only the expected marker and confirm the two remote files are expected
before initial sync. Screenshot also shows grey Preview/Sync Now controls; actual
clickability/reason has not been established. No source change or repeated code
test; scoped whitespace check passes (exit 0). No agent VM/cloud operation.

## VM SharePoint Preview scope confirmed — October 24, 2026

Operator confirms the selected library has three directories and two files,
and the local mirror contains the anchor file. This is consistent with the
Preview's two downloads and one anchor upload; directory creation is distinct
from file-copy totals. No other local file is reported. Initial transfer and
directory/content verification remain pending. Next: Sync Now with its initial
confirmation, keep scheduling paused, then check both files and three directories
locally and compare file contents with the web copies. No agent VM/cloud action,
source change or repeated code tests. Scoped whitespace passes (exit 0).

## VM SharePoint whole-library initial mirroring accepted — October 24, 2026

After initial Sync Now and expected local-file/directory checks were requested,
operator reports local files are as expected. Accept reported initial local-file
arrival for the small whole-library SharePoint mirror on the identified 0.5.5
x86_64 VM Flatpak, following setup and Preview with blank remote subfolder.
No explicit command-completion text, content/hash comparison, ongoing upload/
download cycle, schedule or recovery result is inferred. Mark the small-library
initial-mirroring task complete with those limits; larger libraries and broader
lifecycle coverage remain separate.

Updated task, submission audit/index and evidence notes. Four repository-hygiene
tests and whitespace check pass (exit 0); unchanged code gate reused. No source,
agent VM/cloud operation, build, installation, commit, push or PR-draft change.
Next: establish a new disposable text-file baseline for content-verified recovery
restoration, keeping background scheduling paused. Legacy-profile regression
and sleep/VPN fault coverage remain open or require explicit retained limits.

## VM SharePoint disposable recovery-file baseline accepted — October 24, 2026

Following the new local recovery-test text-file upload procedure, operator
confirms the file uploaded and has expected content. Accept reported local-to-
SharePoint text transfer/content at this scope; no byte/hash comparison inferred.
The file is the disposable resource for the next deletion/recovery/restoration
steps. Keep scheduling paused; delete only that cloud test file, inspect Preview
for expected local deletion, then confirm Sync Now and inspect the local recovery
directory's dated subfolder for preserved content. Restoration remains separate.
Source backup-dir2 resolves under local recovery/<UTC-day>; no agent VM/cloud
action or source change. Scoped whitespace passes (exit 0); code inputs unchanged,
no gate repeat. No build, installation, commit, push or PR-draft change.

## VM SharePoint deletion and recovery-copy presence accepted — October 24, 2026

Operator reports the disposable test file was deleted locally and a copy exists
in the recovery folder after the guided cloud deletion/Preview/Sync Now steps.
Accept reported deletion propagation and recovery-copy presence. Recovered-file
content has not yet been confirmed; restoration is not a pass. Next: open the
recovery copy and verify the original test text, copy it back to the mirror under
the original filename while retaining the recovery copy, Preview/Sync Now, then
confirm restored cloud content. Limit operations to this disposable test file.
No source change or agent VM/cloud operation. Scoped whitespace passes (exit 0);
unchanged code gate not repeated. No build, install, commit or push.

## VM SharePoint restored file reappears remotely — October 24, 2026

Following the recovery-copy restoration procedure, operator reports the file is
back in the remote. Accept reported restored-file upload/presence in SharePoint.
Original text in the recovery copy and restored web file has not been explicitly
confirmed; content-verified restoration remains open pending that confirmation.
No further sync is needed merely to confirm the text. Scoped whitespace passes
(exit 0); no code/test inputs changed and no gate repeat. No agent VM/cloud
action, build, installation, commit, push or PR-draft change.

## VM SharePoint content-verified recovery restoration accepted — October 24, 2026

Operator explicitly confirms original test text in both the recovery copy and
restored SharePoint web file. Accept content-verified manual recovery restoration
for the disposable text file on the identified 0.5.5 x86_64 VM Flatpak: upload,
cloud deletion/local propagation, preserved recovery copy, manual copy-back and
remote reupload with original text. No byte/hash equality, other file type,
automatic restoration, schedule or sleep/VPN fault behavior is inferred.

Split the combined active recovery/fault task to mark restoration complete while
leaving active-transfer sleep/VPN checks or explicit limitations open. Updated
submission audit/index and evidence notes. Four repository-hygiene tests and
whitespace check pass (exit 0); unchanged code gate reused. No source, agent
VM/cloud operation, build, installation, commit, push or PR-draft change.
Next: determine availability of a retained legacy OneDrive profile for package
upgrade/absent-retired-unit regression; fault cases remain separate.

## VM legacy-profile package regression unavailable — October 24, 2026

Operator confirms no retained settings from the retired abraunegg/onedrive
engine. Record legacy-profile upgrade/absent-retired-unit package regression as
not run because that VM setup is unavailable; no compatibility pass or blanket
waiver is inferred. Mark the active item deliberately untested with its reason
and retain the disclosure in the submission audit. Earlier native/fake tests
retain their original scope; no synthetic old production profile is created.

Scoped whitespace check passes (exit 0). No code or documentation-link inputs
changed; existing code gate and link tests reused. No agent VM/cloud operation,
source change, build, installation, commit, push or PR-draft change. Remaining
active acceptance decision: exercise active-transfer sleep/VPN faults using
disposable data, or explicitly retain that untested coverage limit.

## VM active-transfer fault-test feasibility started — October 24, 2026

Operator requests attempting the remaining sleep/VPN tests. Source inspection:
src/sleep.rs scopes opt-in logind preparation and wake restoration to Online
mounts, so an Offline mirror interrupted by suspension does not independently
verify Online cleanup/remount behavior. VM-manager Pause/Resume is not evidence
of guest logind PrepareForSleep handling. VPN disconnection from a public cloud
transfer may leave routing intact; distinguish a declared VPN dependency and
real VPN loss from a generic network outage.

Asked whether guest-desktop Suspend works and whether the VM has a working VPN.
Do not begin dependent interruption steps until setup is established. Plan one
disposable transfer per fault, confirm it is active at interruption and compare
original/downloaded SHA256 after recovery; preserve failed state and report
typed/UI results before any repair. Existing native injected-fault/prototype
coverage retains its scope and is not a new installed-package pass.

No fault executed, fixture generated, service/network state changed or source
edited. Scoped whitespace check passes (exit 0); unchanged code gate reused.
No build, install, commit, push or PR-draft change. Next: select the first
feasible VM fault case from the operator's environment answers.

## VM guest-suspend feasibility needs wake diagnosis — October 24, 2026

Operator reports no working VPN yet, but can attempt setup. Guest-desktop Suspend
left the screen apparently frozen; Proxmox reports the VM running. Guest suspend
state and responsiveness have not been independently established. This was a
feasibility attempt, not an observed active-transfer interruption; no fault-test
pass or applet-caused failure is inferred. Prior transfer/recovery passes remain.

Checked official QEMU monitor documentation: `info status` inspects state and
`system_wakeup` wakes a suspended guest. Proxmox qm manual web retrieval returned
403; QEMU command semantics verified from its official monitor reference:
https://www.qemu.org/docs/master/system/monitor.html . Guide operator through
the selected VM's Monitor, then inspect Console responsiveness; do not reset,
stop or power-cycle while wake diagnosis is pending. No agent hypervisor/VM
operation or VPN installation. Scoped whitespace passes (exit 0); no code/test
inputs changed, no gate repeated. Next: obtain status/wake output and whether
the desktop resumes before preparing an active-transfer test.

## VM responsive guest agent; display recovery pending — October 24, 2026

Operator reports QEMU status running; system_wakeup returns unable to wake
because the system is not in suspend state. Console reports display output
inactive. Screenshot of the Proxmox host shell shows two `qm agent 105 ping`
commands returning without output/error. Accept responsive guest-agent evidence;
this does not establish a healthy graphical session or successful sleep/wake.
No active-transfer fault result or applet-caused failure is inferred.

Verified qm guest exec syntax in official Proxmox command reference and chvt
behavior in the packaged kbd manual. Next guided recovery step: use guest agent
to switch only VM 105 to text VT3, then inspect Console before any desktop or
service restart. No agent hypervisor/guest operation, reset or configuration
change. Scoped whitespace passes (exit 0); no code/test inputs changed, code gate
not repeated. Acceptance remains pending while VM display is diagnosed.

## VM graphical VT switch remains blocked — October 24, 2026

Operator uses direct SPICE; restarting the viewer still reports display output
inactive. Guest-agent `chvt 3` exited 0, but SPICE did not recover. Guest
`loginctl list-sessions` shows graphical user session 3 on seat0/tty1, state
online. Switching back with `chvt 1` exceeded the synchronous wait and returned
guest PID 145933. Subsequent exec-status returned exited=0: process still running,
not successful exit code 0. Do not queue repeated VT switches or infer a complete
guest hang; guest-agent queries still respond.

No reboot, reset, logout or display-manager restart has been requested. Next:
inspect recent guest kernel messages through the agent for DRM/suspend errors,
then choose recovery from that evidence. No active-transfer fault was exercised
or accepted; no applet-caused failure inferred. Scoped whitespace passes (exit 0);
no source/test inputs changed, no gate repeat, no agent hypervisor/VM operation.

## VM kernel graphics workers blocked — October 24, 2026

Guest journal read through the agent exited 0. Operator-provided excerpt contains
kernel hung-task reports in drm_modeset_lock, drm_atomic_helper_swap_state,
framebuffer removal and fbcon_register_existing_fbs; one worker waits on a mutex
likely held by another worker. This supports blocked kernel graphics/modesetting
as the immediate display/VT-switch problem. The original suspend failure's root
cause and any applet involvement are not established from this excerpt. Terminal
switch attempts may also contribute framebuffer/console work to the captured
trace; do not equate all trace activity with the original suspend event.

Record installed active-transfer sleep testing as blocked at VM feasibility;
no transfer was deliberately arranged or observed at the interruption. Preserve
earlier package passes. Recommended recovery is a graceful guest reboot after
preserving available diagnostic output; it ends the current session and can
lose unsaved application edits. No restart/reset executed or successful recovery
claimed. Scoped whitespace passes (exit 0); no source/test inputs changed, code
gate not repeated. Next: operator chooses graceful guest reboot, then confirms
display and applet recovery before any further fault test.

## VM force-stop and available sleep mode observed — October 24, 2026

Operator reports plain qm stop timed out acquiring the VM lock. Supported
`qm stop 105 --overrule-shutdown 1 --timeout 30` returned successfully and
`qm status 105` reported stopped. No successful subsequent desktop recovery is
reported yet. A later successful guest exec returned `/sys/power/mem_sleep` as
`[s2idle]`; the transcript therefore spans VM-state changes whose timing is not
fully shown. Recheck current VM status before further action.

Reported configuration: q35, OVMF, manual virtio-gpu-pci and SPICE devices in args,
with vga:none. The custom args supply a GPU despite vga:none; no missing-GPU
diagnosis or virtual-hardware change is justified by that field alone. Omit
host addresses, MACs and UUIDs from these records. Only suspend-to-idle is exposed
to this guest; no deep suspend mode is listed. Linux documents in-band wakeup
for s2idle, so QEMU suspended-state wake is not a general recovery mechanism for
it. This does not establish whether the original transition entered or exited
s2idle before the graphics lockup.

No sleep-mode/firmware/display change or new fault test. Next: confirm running
desktop recovery and kernel version, then assess a controlled wake source before
another idle feasibility test. Active-transfer acceptance remains blocked, not
passed. Scoped whitespace passes (exit 0); no code/test change or gate repeat.

## VM desktop baseline recovered; RTC inspection next — October 24, 2026

Operator reports VM 105 running; guest uname returns 7.2.6-76070206-generic,
and explicitly confirms SPICE desktop is responsive again. Accept post-recovery
guest/display baseline only; no successful suspend/wake or applet-specific
post-reboot acceptance inferred. No repeat suspend yet.

Verified rtcwake manual: mode show prints alarm state without setting an alarm;
mode no sets an alarm without suspending. First inspect alarm/wakeup capability
through guest agent before any timed sleep. Later applet sleep tests must use
the logind/systemd suspend path; direct rtcwake sleep is not equivalent evidence
for applet PrepareForSleep handling. No alarm or VM configuration changed by the
agent. Scoped whitespace passes (exit 0); code/test inputs unchanged, no gate
repeat. Next: obtain read-only RTC alarm and wakeup-state output.

## VM RTC alarm readable and wakeup enabled — October 24, 2026

Operator supplies successful guest-agent command results: rtcwake mode show
reports RTC assumed UTC and alarm off; rtc0 device power/wakeup reports enabled.
Accept readable RTC and configured wakeup state, not proven wake capability.
Next controlled feasibility step: save work, pause mirror schedules and unmount
Online connections, arm a 60-second RTC alarm using mode no, then promptly issue
systemctl suspend only after alarm setup returns guest exitcode 0. Observe SPICE
and guest responsiveness after the alarm; do not treat this idle trial as an
active-transfer or Online cleanup pass. A repeated graphics hang remains possible.
No alarm/suspend command has been executed by the agent. Scoped whitespace
passes (exit 0); no code/test input changes or gate repeat. No build, install,
commit, push or PR-draft action.

## VM timed-suspend wake not observed; guest agent unavailable — October 24, 2026

Operator reports completing RTC-arm and systemctl-suspend commands but no wake
observed so far. Subsequent `qm agent 105 ping` reports guest agent not running.
Alarm-programming/suspend output and exact elapsed time were not supplied;
successful RTC delivery or failed guest resume cannot yet be established.
An unavailable agent is compatible with suspended userspace and is not alone
evidence of a crashed guest. No active transfer was deliberately tested.

Next: inspect current QEMU status through Monitor. For a running QEMU guest,
try a single injected Shift key via supported sendkey, independent of SPICE;
for an actual QEMU suspended state, use system_wakeup. Recheck guest-agent and
display responsiveness, avoiding repeated terminal switches or suspend requests.
No new pass or applet-specific failure inferred. Scoped whitespace passes
(exit 0); no source/test inputs changed, code gate not repeated. No agent
hypervisor/VM action or new force-stop.

## VM timed-suspend trial still lacks observable wake — October 24, 2026

Operator reports QEMU running and no return after the injected-key guidance;
interpret as display not returning unless operator clarifies the command itself
is blocked. RTC/key wake success is not observed; guest agent was unavailable
at the preceding query. Stop repeating sleep trials on this unchanged setup.
Retain active-transfer sleep acceptance as blocked at idle-VM feasibility, not
a completed fault test or applet failure. VPN setup remains separate.

Recommend previously successful targeted force-stop with overrule-shutdown,
verify stopped, then start VM 105; this discards unsaved guest state. No recovery
executed by the agent or successful second restart claimed. Updated concise
active task and audit limitation; kept earlier detailed failure/recovery chronology.
Scoped whitespace passes (exit 0); unchanged code/link checks reused. Next:
operator recovers the VM and confirms responsive SPICE; investigate virtual
sleep/graphics support separately before another applet sleep test.

## VM back online after second recovery — October 24, 2026

Operator confirms the VM is back online following the targeted recovery guidance.
Accept reported availability only; no successful suspend/wake, active-transfer
recovery or post-reboot applet identity/health pass inferred. Prior acceptance
results retain their scope. Leave sleep testing blocked on this VM configuration
and avoid another unchanged suspend trial. VPN remains unconfigured; next determine
which VPN protocol/profile is available before selecting its typed setup path.
Scoped whitespace passes (exit 0); no code/test change or gate repeat. No agent
VM/cloud operation, package build/install, commit, push or PR-draft change.

## VM OpenConnect guide URL correction — October 24, 2026

Operator screenshot shows HTTP 404 using main/scripts/setup-openconnect.sh with
uninstall. Branch is master. During review the operator corrected the install
URL locally; preserved it and corrected the remaining two uninstall guide URLs.
Local setup script reviewed: install changes host packages, on-demand user
authentication-agent unit and icon setting; uninstall retains packages/profiles.
Neither creates a VPN profile. Script behavior and historical header examples
are unchanged; no install/uninstall performed by the agent.

Raw web lookup failed with cache miss; direct download failed sandbox DNS
(exit 6). Escalated download verification was declined by user and did not run;
remote response/script equality remain unverified. No bypass attempted.
Four repository-hygiene tests and scoped whitespace check pass (exit 0);
unchanged code gate reused. No package/service action, commit, push or PR-draft
change. Next: operator uses master URL with install to set up OpenConnect on VM,
then creates its VPN profile with nm-connection-editor.

## VM independent VPN baseline accepted — October 24, 2026

Operator requests testing outside Cloud Mounter first, followed by disconnect
and applet activation. Following the manual NetworkManager procedure, operator
reports it worked. Accept reported manual VPN authentication/connection,
protected-resource access and disconnection at that scope. Do not infer the
setup script installation outcome, applet-owned activation/release or loss of
an active transfer. No private gateway, profile name or credentials recorded.

Updated active checklist to separate passed manual baseline from pending applet
activation and VPN-loss tests. If an nm-applet was launched manually, exit only
that instance before testing the applet's on-demand authentication-agent service;
avoid competing bus-name owners. Next: select the saved VPN on one test Online
connection, Re-Test/Save if required, activate from a disconnected VPN baseline,
check mounted access, unmount and observe VPN release. Scoped whitespace passes
(exit 0); code/test/link inputs unchanged, earlier checks reused. No agent VM,
service, cloud, build, install, commit, push or PR-draft action.

## VM applet VPN activation and mounted listing accepted — October 24, 2026

Following the applet VPN-dependency procedure from the disconnected baseline,
operator reports it connected, the mount is accessible and files show up. Accept
reported applet-driven VPN connection and mounted-directory listing on the VM
Flatpak. File-open/content equality, exact prompt details, unmount and VPN release
are not separately reported. No VPN-loss or active-transfer recovery result.

Updated task/audit and these notes. Scoped whitespace passes (exit 0); unchanged
code/test/link inputs retain earlier checks. No agent VM/service/cloud operation,
source change, build, install, commit, push or PR-draft action. Next: switch the
test mount off with other VPN-dependent connections off, confirm unmount and
VPN release; retain file-content check as pending until reported.

## VM applet-owned VPN release accepted — October 24, 2026

After toggle-off/unmount/VPN-release guidance, operator reports VPN no longer
connected. Accept reported VPN release alongside prior applet-driven activation,
mount accessibility and file listing. Separate file-open/content and explicit
unmount-completion observations were not supplied; no additional pass inferred.
Mark the scoped VPN activation/release task complete with these limits.

Next: identify the mounted provider and a disposable resource actually reachable
only through the VPN before testing loss during transfer. Disconnecting a tunnel
while public-cloud traffic uses an unaffected route is not evidence of interrupted
data transport. No loss injected or active transfer result yet. Task/audit/notes
updated; scoped whitespace passes (exit 0). Unchanged code/link checks reused;
no source, agent VM/service/cloud operation, build, install, commit, push or
PR-draft action.

## VM VPN-loss test resource identified as VPN-only SMB — October 24, 2026

Operator confirms the tested mount is SMB and reachable only through VPN. This
supports a real network-dependency interruption test rather than assuming public
cloud traffic stops on tunnel loss. Requested a disposable writable directory
and its VM mount path before creating test data. No private remote identity,
server address or credentials needed in these records.

Source mount plan uses full VFS caching: a completed file-manager copy can mean
local cache completion while upload is pending. A loss test must establish
pending/active remote upload independently and verify the final server copy via
an independent read; reading the same mount can return cached data. Preserve
the local source and its SHA256, and avoid unmounting/deleting cache during an
interrupted upload. No file created or fault injected yet. Scoped whitespace
passes (exit 0); code/test inputs unchanged, existing gate reused. Next: select
the approved disposable directory and prepare the bounded test file.

## VM SMB disposable test directory confirmed — October 24, 2026

Operator confirms a writable test directory, remounts the SMB connection and
creates tmp on the share. File-manager screenshot supplies its full local path;
do not reproduce organization-specific path text in these records. Target is
only that approved tmp directory. VPN remains connected; no fault injected.

Next create 128 MiB random disposable source data and a SHA256 baseline in a
unique Downloads directory outside the mount. Keep the source/hash throughout
the test. No remote copy yet; after preparation, confirm upload activity before
VPN disconnection, preserve the mount/cache on failure and independently verify
server bytes after recovery. No fixture created or VM/cloud operation by the
agent. Scoped whitespace passes (exit 0); unchanged code/link checks reused.

## VM SMB VPN-loss source/hash baseline created — October 24, 2026

Operator screenshot confirms creation of 134217728 bytes in the unique Downloads
test directory, outside the SMB mount; sha256sum completed and tee saved
source.sha256. Synthetic source SHA256:
`3f32cc4309c5a4e6ce59f26e6bed5c483717b46141c3d21b956769c7f838c226`.
The displayed pre-test listing of the approved tmp directory is empty. Do not
retain organization-specific mount path or username in these records.

Next: copy source into that directory without overwriting, observe pending/
active upload, manually disconnect VPN through NetworkManager while mount stays
on, then observe status. Full VFS caching means cp completion alone is not proof
of remote completion or in-flight transfer. If upload completes before loss,
record that the interruption was not exercised. Keep source/hash and cache;
no cleanup or unmount until recovery and independent server verification.
No copy/fault/recovery pass yet. Scoped whitespace passes (exit 0); no source/
test inputs changed, code gate reused. No agent VM/cloud operation or install.

## VM SMB copy visible with partial upload artifact — October 24, 2026

Operator reports no pending uploads observed and VPN remained connected, so no
VPN-loss attempt has occurred. cp -n emitted a portability warning, not a
reported copy error; avoid repeating it blindly. Screenshot shows mounted
vpn-loss-test.bin at 128 MiB plus vpn-loss-test.bin.4fb71853.partial. Mounted
size may come from VFS cache; neither this size nor partial-file presence proves
completed server bytes or an active transfer. No server SHA256 result yet.

Official rclone docs describe temporary .partial uploads renamed to final names
on completion: https://rclone.org/docs/#inplace . A stale/retrying artifact is
also possible here; do not infer current upload progress or delete it. Next:
keep VPN and mount connected, inspect bounded matching user-service journal
entries for upload completion/errors before choosing the interruption window.
Source unit names match cosmic-mounter-*.service. No agent VM/cloud operation,
new fixture or code change. Scoped whitespace passes (exit 0); existing code/
link checks reused. Fault acceptance remains untested.

## VM SMB first upload succeeded without interruption — October 24, 2026

Operator journal screenshot shows synthetic tmp/vpn-loss-test.bin queued at
13:42:41 for upload in 5 seconds, then Copied (new) and upload succeeded try #1
at 13:43:29. Times are as displayed in the guest journal; no timezone conversion
assumed. About 48 seconds elapsed from queue to completion. VPN stayed connected;
accept engine-reported upload completion only, not independent server hashes or
VPN-loss recovery. The earlier partial-file snapshot is consistent with a
temporary upload artifact; current presence was not rechecked. Pending UI work
was not observed by the operator, without enough evidence to establish a UI bug.

Next reuse the unchanged 128 MiB source/hash with a unique new remote filename,
copy with update=none, then disconnect VPN via NetworkManager roughly 10 seconds
after cp returns while preserving the mount/cache. This timing targets the
observed upload window but actual interruption still requires engine/status
evidence. No automatic retry/recovery success inferred. Scoped whitespace passes
(exit 0); no source/test inputs changed, code gate reused. No agent VM/cloud
operation, build, install, commit, push or PR-draft action. Private host/remote
identity and addresses from the raw screenshot are not copied into these notes.

## VM SMB VPN loss detected; Repair offered — October 24, 2026

Following the unique-filename copy and timed VPN-disconnect guidance, operator
reports the applet says VPN is off and offers Repair. Accept reported loss notice
and Repair availability only. Exact disconnect timing, upload-in-flight evidence,
child/mount state and server integrity are not yet confirmed. No transfer-loss
recovery or Repair pass inferred.

Next reconnect VPN through NetworkManager while keeping the existing mount on,
source/hash intact and cache untouched; do not invoke Repair first. Observe for
up to 60 seconds whether state clears and upload retries or Repair remains
required, then inspect bounded engine evidence. An observation window is not a
guarantee of retry completion. Scoped whitespace passes (exit 0); code/test inputs
unchanged, existing gate reused. No source or agent VM/service/cloud operation,
build, install, commit, push or PR-draft action.

## VM SMB state recovers after manual VPN reconnect — October 24, 2026

Operator manually reconnects VPN and reports applet summary 1/7 connections
active, Repair no longer displayed, mount running and row indicator 1. Accept
reported connection-state recovery without invoking Repair. The row indicator
is not a completed-transfer count; pending/queued upload completion is not yet
established. No engine retry log or independent server hash result supplied.

Next inspect recent bounded user-service journal entries for the unique second
test filename, upload failure/retry and success timestamps; retain mount/cache
and source/hash. Then verify server bytes independently of the cached mount.
Scoped whitespace passes (exit 0); no source/test changes or gate repeat. No
agent VM/service/cloud action, build, install, commit, push or PR-draft change.

## VM interrupted SMB upload retries and succeeds — October 24, 2026

Operator journal screenshot establishes the second synthetic file's transfer
timeline (guest journal times as displayed): tmp/vpn-loss-1792875102.bin queued
13:51:43; first write/copy failed 13:53:26 with TCP I/O timeout; VFS reported
failed upload try 1 at 13:53:36, retry delay 10 seconds. Subsequent SMB connection/
DNS errors led to failed tries 2 and 3 with 20/40-second backoff. Failed partial
copy removal initially failed, later logged removal at 13:54:47. Copied (new)
and upload succeeded try 4 both appear at 13:56:10. Raw host/share/account and
network addresses are omitted from these records. Authentication-agent GLib
assertion messages are also visible; no credentials, causal diagnosis or new
authentication failure is inferred from those lines alone.

Together with reported VPN disconnect/reconnect and automatic Repair clearing,
accept actual interrupted SMB write and engine upload recovery without Repair
or remount. Independent server-byte/hash validation remains pending; same-mount
hash can read cached bytes. Next determine availability of smbclient (or another
independent SMB reader) to download only the synthetic file to local Downloads
and compare SHA256 with preserved source. No new upload, VPN interruption or
cache removal needed. Scoped whitespace passes (exit 0); unchanged code/link
checks reused. No source, agent VM/cloud action, build, install, commit or push.

## VM independent SMB hash-reader selection — October 24, 2026

Operator reports command -v smbclient printed nothing. Do not install a package
solely for this check without approval. Existing host rclone can read the SMB
backend directly outside the mount's VFS cache. Verified official hashsum
documentation for SHA256 with --download; this can compute the server-file hash
by reading its bytes without copying into the mounted path or changing the remote.
Request only the saved remote name and remote subdirectory from the editor to
construct a scoped read of the synthetic file. No secrets/config dump requested.
No server hash yet; no engine command executed by the agent. Scoped whitespace
passes (exit 0); code/test inputs unchanged, existing gate reused. Next compare
direct remote SHA256 with the preserved baseline.

## VM SMB VPN-loss recovery byte integrity accepted — October 24, 2026

Operator screenshot shows successful direct rclone hashsum SHA256 of the unique
second test file with --download, --retries 1, --timeout 60s and --max-duration 3m.
Hash returned for vpn-loss-1792875102.bin:
`3f32cc4309c5a4e6ce59f26e6bed5c483717b46141c3d21b956769c7f838c226`.
It exactly matches the preserved 128 MiB local source baseline. Source is the
remote SMB backend, not the mounted filesystem; this bypasses VFS cached reads.
Private remote/share/account details are intentionally omitted here.

Accept scoped active-upload VPN-loss handling on the identified VM Flatpak:
VPN-off/Repair notice, actual interrupted SMB write, three failed attempts,
manual VPN restoration, automatic upload success on try 4, Repair clearing
without invocation/remount, and matching server bytes. This does not claim
offset-based resume, automatic VPN reconnection, pending-count correctness,
other providers or successful sleep/wake. Test source/hash and remote copies
remain; no deletion/cleanup performed or authorized by this result.

Marked VPN-loss check complete and separated still-blocked sleep acceptance.
Condensed active checklist while preserving detailed failure/recovery chronology
in these notes; updated audit/index. Four repository-hygiene tests and whitespace
check pass (exit 0); unchanged code gate reused. No source, agent VM/cloud action,
build, install, commit, push or PR-draft change. Remaining acceptance decision:
active-transfer sleep testing is blocked by idle VM suspend/wake reliability;
legacy-profile and installed ARM tests remain explicitly unavailable/unrun.

### October 24, 2026 — requested documentation commit preparation

Operator requested checking for new files, staging and committing. git status
--short --branch --untracked-files=all found no untracked files on master.
Reviewed the seven pending documentation files: PR draft, README, task list,
completion notes, engineering index, COSMIC submission audit and OpenConnect
guide. Changes retain scoped installed-test evidence and unrun limitations,
future Google-native configuration work, corrected guide URLs, operator draft
edits and the requested plain-language appendix. No packages, caches or raw
logs selected. Reuse the just-passed four documentation hygiene tests and
appendix assertions; runtime source remains unchanged from published 13e204e.
Scoped task/evidence whitespace check passes (exit 0); repository-wide draft
trailing whitespace was previously reported and is deliberately preserved.
Commit authorized; push is not requested. Commit identity/outcome is recorded
in Git history and the operator response, not predicted in this record.
