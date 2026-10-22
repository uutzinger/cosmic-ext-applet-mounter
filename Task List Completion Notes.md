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
