# Cloud Mounter Applet Task List

This document tracks the current work-item checklist. Work through the
v0.5.4 release is archived in `docs/archive/` (see "Archive and open items carried
forward" below). Detailed execution evidence, manual test notes, command output
summaries, and completion commentary are kept in `Task List Completion Notes.md`.

**Execution environment:** VS Code with Codex

This list implements `Requirements and Specifications.md`. The approved source
description is `Applet Description.md`.

Repository-wide working guidance is maintained in [AGENTS.md](AGENTS.md).

## Archive and open items carried forward

Completed history is archived, unchanged, under `docs/archive/`:

- v0.5.3: [task list](docs/archive/task-list-through-v0.5.3.md), [completion notes](docs/archive/task-list-completion-notes-through-v0.5.3.md)
- v0.5.4-pre: [task list](docs/archive/task-list-through-v0.5.4-pre.md), [completion notes](docs/archive/task-list-completion-notes-through-v0.5.4-pre.md)
- v0.5.4: [task list](docs/archive/task-list-through-v0.5.4.md), [completion notes](docs/archive/task-list-completion-notes-through-v0.5.4.md)

Open items below are condensed; this list is authoritative for their status.
Full historical wording is in [Task List Carry Forward.md](Task%20List%20Carry%20Forward.md).
The v0.5.4 archive introduced no additional open items. Existing carried-forward
work and the future sections below remain active.

### From: SFTP Provider Extension — September 29, 2026

- [ ] Update dependency/setup guidance and release metadata for delivered SFTP support and verified limitations before release.

### From: Teams (SharePoint library) implementation plan — October 1, 2026

- [ ] Installed Teams checks: isolated network loss/recovery, private/shared-channel site, consent/auth failure diagnostics, Flatpak host checks.
- [ ] Teams OAuth tenant-consent failure feedback and Flatpak host-browser/credential access.
- [ ] Optional: tenant-wide site/library browsing without a pasted library URL.

### From: Sleep, mountpoint and pending-count work — October 2, 2026

- [ ] UA OneDrive: determine how local directories appeared while the share was unmounted; keep any moved-aside copy.
- [ ] Controlled sleep/wake check on the installed applet (busy mount stays usable; clean mount restores without repeated Repair).
- [ ] Verify a non-empty mountpoint is blocked by direct systemd start; repeat in the packaged Flatpak.
- [ ] Offline mirror pending-change counts: reliable count during active manual/scheduled runs, falling to zero only after a fresh completed check.
- [ ] Verify Online queue and mirror counts on disposable data (Preview, Sync Now, scheduled, interruption, errors, Flatpak) plus an installed-popup visual check.

### From: Reorder audit, Google Drive and popup state — October 3–5, 2026

- [ ] Reauthenticate the personal OneDrive Online connection in a fresh cache and confirm its drive differs from the University drive.
- [ ] With Cisco VPN connected, read-only check the saved `ua_engr` SMB remote and its VPN assignment.
- [ ] Google Drive 403 quota errors: disable preload and use a private OAuth client ID, then check for recurrence.
- [ ] Google OAuth client ID authentication fails with a parsing/unexpected-field error; diagnose.
- [ ] Create or verify a Google Desktop OAuth client and issue a fresh token for it; never reuse the shared-client token.
- [ ] Persistent popup startup state: installed lifecycle and reboot verification across logout/login for the three UA connections.

### From: Release build workflow follow-up — October 13, 2026

- [ ] Verify the new hosted promotion workflow with the next approved immutable release; do not rebuild 0.5.3 to test it.

# 0.5.5 Release

### Scope review — October 20, 2026

- [x] Review AGENTS.md, the 0.5.5 sections and `Push Buttons.md`; preserve
  operator edits and establish the execution order below. Evidence is in
  Task List Completion Notes.md, “0.5.5 scope review”.
- [x] Phase 1: complete the legacy abraunegg removal checklist below, including
  safe retirement of proven app-owned units and explicit unsupported profiles.
- [x] Phase 2: complete GUI source cleanup and scoped native acceptance from
  `Push Buttons.md`. October 22: remaining retired-profile regression transfers
  to package-upgrade acceptance; whole-root cloud limits are recorded below.
  Broader validation remains in its backlog; final-package checks are separate.
- [ ] Phase 3: prepare COSMIC Flatpak submission from the accepted 0.5.5 source,
  refresh candidate/permission evidence and hand off operator-authored submission.
  October 22 clarification: the COSMIC repository PR comes later; prioritize
  the 0.5.5 GitHub release and Debian/Flatpak artifacts first.

Execution order is removal, GUI cleanup, then submission; section placement
retains the existing planning record. The validation/expansion backlog remains
deferred unless changed behavior or a related defect makes an item necessary.
Published 0.5.4 history remains unchanged. Reuse passing checks only where their
inputs and acceptance scope remain applicable; new removal/GUI behavior needs
focused tests and the final gate. Installed/package acceptance stays separate.

## 0.5.5: legacy abraunegg engine removal

Planned October 14, 2026; separated from 0.5.4 release gates October 18, 2026.
Target: release after the 0.5.4 deprecation release, tentatively 0.5.5. This
future section has its own numbering. Existing abraunegg connections remain
supported in 0.5.4; this work is not a condition of publishing 0.5.4.
OneDrive Online using onedriver remains supported throughout.

Checklist refreshed October 18 against current source. Release boundary above
remains authoritative until an explicit policy change; preparation may proceed
separately without removing legacy support from the 0.5.4 source/package.

- [x] Retain legacy profile deserialization/identity and replacement guidance, but
  block retired-engine authentication, Test/Preview/Sync/Start and scheduling with
  typed unsupported-engine results. Never reinterpret absent-engine legacy
  profiles as rclone. Preserve local mirror/recovery files, databases and
  credentials; verify existing profiles remain visible and removable.
  - [x] Completed October 21: typed engine-support policy, inert restored rows,
    blocked execution paths and safe profile removal preserve the legacy identity.
- [x] Define and test upgrade handling for old app-owned monitor units so an
  already installed/enabled legacy service cannot continue launching the retired
  engine. Use typed systemd control and ownership checks; stop/disable only
  proven app-owned units. Preserve unrelated user services and all file data.
- [x] Remove only legacy runtime/authentication code: one_drive_* plans/requests
  and legacy-only status/parser helpers in src/sync.rs; legacy setup/auth-files,
  pending-count, monitor and dispatch branches in src/app.rs; OneDriveSync
  diagnostics/controller requirements and Executable::OneDrive process route.
  Preserve rclone OneDrive backend/identity/guards, shared Microsoft OAuth and
  onedriver Online. Replace removed behavior with explicit unsupported state.
- [o] WebKit helper, helper-only dependency recommendations/install options and
  unused localization keys moved into 0.5.4 step-2 cleanup by operator October 18.
  Browser/manual legacy reauthentication retained; this does not retire engine
  support or delete existing credentials. See consolidated acceptance above.
  The 0.5.5 removal now supersedes that retained runtime without changing the
  published 0.5.4 source or history.
- [x] Remove remaining onedrive engine recommendation and --onedrive installer
  option after checking consumers. Retain Python for maintained build/developer
  tools; do not uninstall packages from user machines. Preserve --onedriver,
  --rclone and rclone's onedrive backend.
- [x] Retire current legacy-executable entries in host-command inventory and
  permission rationale; preserve historical removed-command evidence and review
  consumers before dropping shared permissions. Update script catalog/inventory,
  AppStream/README/dependency guides and removal-release notes without rewriting
  published 0.5.4 deprecation history. Keep prototype evidence explicitly labeled.
- [x] Focused synthetic/fake tests: legacy profile upgrade and inert controls,
  no retired executable/dependency/helper dispatch, app-owned unit retirement
  with rollback/partial failure, data preservation and rclone/onedriver parity.
  Run check and focused tests after Rust edits, final gate once complete, plus
  localization-check and metadata-check when their inputs change.
- [x] Accept an installed native upgrade with an app-owned legacy profile/unit.
  October 21: the exact installed 0.5.5 candidate retired the unit, retained the
  unsupported row and preserved its mirror, recovery, database and credential
  locations. The operator used an existing backed-up profile rather than a
  disposable profile; profile removal was not exercised.
- [ ] Repeat package-specific upgrade acceptance before claiming Debian or Flatpak
  removal-release support. No Debian or Flatpak package was built or installed in
  this phase.
  Include the retained retired-profile missing-service regression in this same
  disposable upgrade check: absent retired unit must not cause recurring refresh
  warnings, while the unsupported row remains visible and its data preserved.

## 0.5.5 GUI Clean Up

Read and work on Push buttons.md. Create task list here.

Source: [Push Buttons.md](Push%20Buttons.md) (exact filename; retained unchanged).

- [x] Inventory existing Add/Modify controls for all listed Online/Offline
  providers, including the untested Google, Box, SMB and SFTP Offline cases.
  Record shared actions and provider-specific exceptions before editing. Source
  baseline and proposed state/color contract: [GUI action layout inventory](docs/engineering/gui-action-layout-inventory.md).
- [x] Add “Remove Remote” after Add Connection in Settings. List all rclone
  remotes, with disabled “In Use” or red “Remove” actions. Define references
  that prevent removal, recheck usage before deletion, and handle failure without
  deleting connection data or cloud/local files.
  - [x] Implemented October 21, 2026: remote selection remains in
    Add/Modify; removal moved into a dedicated Settings window that identifies
    the saved connections using each remote and rechecks current configuration
    at deletion.
  - [x] Layout follow-up implemented October 21, 2026: each remote's identity
    and usage description are left-aligned, while every remote action occupies
    one fixed-width column at the right edge. Installed rendering remains in the
    shared GUI acceptance item below.
- [x] Remove the Settings Refresh button and its dedicated action; verify
  automatic updates and shared refresh mechanisms still serve their callers.
  - [x] Implemented October 21, 2026: removed the button-only message, IPC
    command, notices and tests while retaining configuration-change reloads,
    periodic status/network polling, mount inspection and unit observation.
- [x] Merge selected-remote verification into Test. Add editors use
  Detect, Connect, Test, Save; OneDrive Online omits Detect.
  - [x] Implemented October 21, 2026: removed OneDrive Offline's separate
    Verify selected remote action. Test now verifies an unverified selected
    remote and automatically continues the full connection test.
- [x] Modify editors use Detect, Re-Connect, Re-Test, Save, Disable, Remove.
  OneDrive Online omits Detect; Google Online retains Update Google OAuth.
  Offline editors put Preview and Sync Now on a second row.
  - [x] Source implemented October 21, 2026: common Modify labels/order and
    Offline second row; Google and Box show disabled Re-Connect with explanatory
    help. Existing backend operations and confirmations retained. Shared action
    colors/states and installed rendering remain in the acceptance items below.
- [x] Modify Action buttons in New Connection Window by renaming and reordering
  to Detect, Connect Test, Save. For OneDrive Online we use Connect, Test, Save. 
  - [x] Source implemented October 21, 2026: common Add labels and workflow
    order across providers; existing pending and confirmation states retained.
    Installed rendering remains in the shared acceptance item.
- [x] Implement the requested blue/grey/red presentation with explicit action
  states, accessible labels and keyboard paths. Invalidate successful tests
  when relevant inputs change; distinguish disabled actions from neutral styling.
  - [x] Definition completed October 21, 2026: shared action states, saved/setup/
    test comparisons, provider field mapping and stale-result rules recorded in
    the GUI action layout inventory. Subsequent source implementation is below.
  - [x] October 21 refinement: Blocked/Running retain the base button color
    using the theme-disabled appearance, matching locked provider/mode controls;
    running blue actions become faded blue with progress text. Wiring completed below.
  - [x] Source implemented October 21: shared color/availability policy, scoped
    request tracking and style-preserving disabled controls; sixteen synthetic
    transition tests pass. Installed rendering remains in the acceptance task.
- [x] Resolve the source's ambiguous Sync Now grey-after-success rule and
  Re-Test “changed” versus “passed” wording before implementing those states.
  Define behavior for existing valid remotes and failed/cancelled operations.
  Definition October 21: Re-Test is recommended after relevant edits or setup
  success; Sync Now stays grey and follows backend eligibility. No new universal
  Preview prerequisite. Implementation and tests remain in the adjacent tasks.
- [x] Localize new/changed text; test action transitions, remote-in-use guards,
  stale results and partial failures with synthetic fixtures/fake adapters.
  Run focused checks and localization-check, then include changes in final gate.
  Source/synthetic acceptance only; existing provider/remote guards are reused
  through the gate. Installed UI and live provider acceptance remain separate.
- [x] Accept the conducted installed native panel/Settings/editor checks below:
  observed button layouts, keyboard/focus and light/dark appearance at usual
  scaling. Reuse these results; final-package rendering and broader window/
  scaling coverage remain separate in the acceptance matrix/backlog.
  - [x] October 22 native Google test: deliberately missing draft folder shows
    disabled Testing…, then failure with Save blocked and blue Re-Test available;
    operator reports the guided sequence worked as expected.
  - [x] October 22 native SharePoint mirror: an unsaved name edit blocks Preview
    and Sync Now; restoring the saved name restores prior action availability.
  - [x] October 22 native Google draft-only Disable/Enable: red Disable changes
    to blue Enable with Save available; restoring enabled returns red Disable
    and unchanged-draft Save blocking. No persisted disable was tested.
  - [x] October 22 native Google Remove: first click shows red Confirm Remove;
    closing/reopening cancels confirmation and retains the connection unchanged.
    Confirmed deletion was not exercised.
  - [x] October 22 native remote manager: remaining entries show aligned,
    non-clickable In use actions, confirmed by operator. Later Box cleanup below
    supplies confirmed unused-remote removal; no repeat removal-label test is needed.
  - [x] October 22 subsequent native Box cleanup: operator removed the test
    connection and its rclone remote; both removals worked as expected.
  - [x] October 22 native keyboard: Google Re-Test activates with Return;
    Tab/Shift+Tab navigation and visible focus work as expected. Pinned libcosmic
    uses Enter, not Space; guided expectation corrected.
  - [x] October 22 operator native light/dark check: editor button roles,
    disabled-label readability, focus and layout work at usual scaling. Broader
    multi-scale/Flatpak coverage remains separate; Box Add acceptance is below.


- [x] Google Drive browser authorization follow-up — October 21: installed
  testing reported no browser while Connect claimed successful OAuth. Correct
  automatic browser setup, reject unfinished responses and verify access before
  reporting success; preserve existing remotes and reuse unchanged GUI checks.
  - [x] Source repaired and eight focused tests passed October 22: automatic
    browser setup, completion/access checks and reported rollback failures.
    History audit finds the defect in the earliest available 0.1 source through
    published 0.5.4; no new 0.5.5 introduction identified. Installed acceptance
    remains in the shared acceptance task. See GUI action layout inventory.
- [x] Installed Google private OAuth update and Online Re-Test — October 22:
  operator entered a private client ID/secret; test remote now has both fields
  configured, and screenshot confirms successful Google backend/access testing
  with 121 root entries. Credentials were not displayed. Fresh browser consent,
  token freshness and grey OAuth running state are not inferred; Google Offline
  subfolder acceptance is below, and Flatpak coverage remains separate.
- [x] October 22 native Google Drive Offline subfolder: operator reports Preview
  proposed one file, then Sync Now copied it into the local mirror directory.
  Accept Preview/initial-sync/local arrival; fresh browser authentication,
  whole-root, broader two-way lifecycle and Flatpak acceptance remain separate.
  - [x] October 22 operator uploaded two new local files with Sync Now and
    confirmed both in Google Drive. Operator then edited the uploaded text file
    online and ran Preview/Sync; the cloud edit appeared locally. Accept local
    file uploads and cloud-to-local modification syncing for this test folder.
- [x] Box browser authorization repair — October 22: repaired the shared Online/
  Offline creation path with automatic browser setup, completion/access checks,
  safe diagnostics and reported rollback failures. Six Box and eight Google
  focused tests pass; native Online acceptance recorded below.
  - [x] October 22 operator completed new Box Online Connect, browser
    authentication, Test and Save as expected. Read-only inspection confirms a
    saved test connection referencing a configured Box remote.
  - [x] October 22 operator mounted the new Box Online connection and observed
    directory entries populating. Online file read/write and Flatpak acceptance
    remain separate; Offline subfolder acceptance is recorded below.
  - [x] October 22 operator saved a new Box Offline mirror successfully.
  - [ ] October 22 Box Offline initial Preview failure: operator reopened the
    saved connection, Preview failed (recalled exit status 7), and retry was
    unavailable. Diagnosis and source repair are complete: the recovery directory
    lacked an exclusion. Only installed whole-root retry remains open; the
    completed subfolder Preview/Sync and recovery-copy checks are recorded below.
    - [x] Expose bounded, redacted engine diagnostics on failed mirror commands:
      focused tests, check, localization and gate pass. October 22 installed
      screenshot confirms diagnostic rendering and backup-directory overlap.
    - [x] Correct the Offline main-applet toggle after a failed Preview: paused
      indication now derives independently from cached unit observations while
      retaining failure/Repair state. Check, four focused tests and gate pass;
      October 22 native Google failure/recovery check passes: toggle stayed off,
      Repair available, then Preview/Sync passed after repair and folder restore.
      Flatpak rendering acceptance remains pending.
    - [x] Fix Box/Google root-mirror recovery exclusion: installed diagnostic
      identifies backup-directory overlap. Established subfolder filters preserved;
      focused/check/gate and disposable real-engine root Preview/recovery pass.
      Installed whole-root Box acceptance remains pending.
  - [x] October 22 native Box Offline subfolder: operator reports Preview and
    Sync Now passed and files arrived in the local mirror directory. Cloud deletion,
    recovery-copy presence and scheduled downloads also passed as recorded below.
    Upload/edit, whole-root and Flatpak acceptance are not inferred.
  - [x] October 22 native Box Offline cloud-to-local deletion: operator deleted
    one file on Box, synchronized, and confirmed its local mirror copy disappeared.
    Operator also confirmed the deleted files are present in the recovery folder.
    Recovery content/restoration and the proposed upload/edit checks remain separate.
- [x] SharePoint whole-library Offline mirror source support — October 22:
  empty folder selects the verified library root; target-bound state, recovery
  exclusions and fresh Preview confirmation on retargeting retained. Focused
  tests, localization and final gate pass; installed acceptance remains separate.
  - [x] Reuse installed Modify subfolder acceptance: name/path changes, Re-Test,
    shared-remote Confirm Save, Preview and initial Sync passed. This is the
    archive-subfolder case, not installed empty-folder/whole-library acceptance.
- [x] Communication recovery source repair — October 22: queued D-Bus ownership,
  bounded reconnect and saved-config reload implemented. Private-bus takeover,
  temporary SharePoint Save persistence, localization and final gate pass.
- [x] Verify installed communication recovery separately; Runtime name was absent
  during the original report. SharePoint Save was subsequently completed after
  shared-remote confirmation; operator also completed Preview and initial Sync.
  See completion notes for the acceptance scope.
  October 22 operator kept Settings open across panel restart: brief communication
  notice cleared automatically and Settings returned to normal. Runtime D-Bus
  owner observed present. Main-applet refresh warning separately traced to the
  retired legacy profile; operator removed it and the warning disappeared.
- [x] Handle expected service absence for retained retired legacy profiles without
  a global status-refresh warning. Preserve genuine observation failures and
  the ability to stop an unexpectedly active legacy service. October 22 source
  fix separates typed unit absence from failed D-Bus observations and replaces
  stale active cache on confirmed absence. Check, six focused tests and gate
  pass; installed retained-profile/package acceptance remains separate.
- [x] Confirm Save label source — October 22: shared Add/Modify label and notice
  for remote reuse; resets on edits, consumption and cancellation. All 24 provider/
  mode/editor combinations, focused tests, localization and gate pass.
  - [x] October 22 operator native SharePoint Modify test: name edit, Re-Test,
    Save → Confirm Save → disabled Saving observed. Other controls/themes remain
    in shared GUI acceptance; completed live tests reused.
  - [x] October 22 operator follow-up: SharePoint confirmation clears after
    another name edit; reverting the draft to saved values and Re-Testing keeps
    Save unavailable. Google test and private OneDrive name edits save without
    a reuse confirmation, consistent with the conditional shared-remote guard.
- [x] Google test Online mount inspection — October 22: native service active,
  FUSE mount present and 109 root entries match a fresh remote listing; no name,
  size or type discrepancies. Write/refresh-event acceptance remains separate.
- [x] Google slow-listing diagnosis — October 22: test service logs show Google
  per-minute rate limiting; shared OAuth client and ineffective VFS fast-list
  selection identified (earlier invalid-parameter diagnosis corrected below).
  See GUI inventory and completion notes; no runtime settings changed.
- [x] Google preload source repair — October 22: enable fast-list at VFS creation;
  populate root before recursive warming under one budget. Real local rclone
  integration, focused tests, localization and final gate pass. Private OAuth
  recommendation visible in Add/Modify for both modes; no shared-quota tuning.
  Installed Google acceptance is recorded immediately below.
- [x] Installed Google preload visibility — October 22, operator-reported: root
  visible within 10–20 seconds and directories populated through three levels.
  This accepts observed browsing improvement, not a depth cap, full-drive cache,
  private-OAuth migration, quota resolution or Flatpak behavior.
- [o] Consider Google root-only/shallow/full-tree preload scope. A depth limit
  applies to Google too, but requires app-managed traversal rather than an
  unsupported depth argument to recursive VFS refresh. Scope is browsing
  usability, not a shared-client quota workaround; custom OAuth setup remains
  operator-owned. Keep rate-limit pacing unchanged.
  October 22 operator decision: do not implement these selectable scopes;
  no implementation is planned. Retain existing root-first recursive preload.
- [x] Close the scoped native changed-path creation acceptance using the reconciled
  matrix in the completion notes. Box Online Add/auth/Test/Save and Google/Box
  Offline subfolder setup/initial sync are accepted at their recorded scope.
  OneDrive/rclone Offline Add/Test verification merge and fresh Google browser
  consent now pass; do not repeat unchanged backend/provider lifecycles
  solely for shared button labels. Keep native and Flatpak acceptance distinct.
  - [x] October 22 OneDrive/rclone Offline new-remote route: Detect found no
    existing remotes; operator created one, Test passed after one click and Save
    completed. Reuse this new-remote Add/Test/Save acceptance.
  - [x] October 22 replacement work/school OneDrive Offline mirror: operator
    created another remote with Connect, saved, Preview reported two local
    files, and Sync Now uploaded both to the remote. Native new-remote setup
    and manual upload are accepted. A file added through the cloud website also
    arrived locally after Sync Now, accepting manual download. This does not
    cover existing-remote Test.
  - [x] October 22 existing-remote Add/Test continuation: after choosing Work
    or school, Detect listed two matching remotes. Operator selected one and
    supplied the required account email and mirror paths. Test verified drive
    identity and selected-folder access, completed structural validation and
    enabled Save. No Connect or additional saved connection was required.
    Earlier discovery/type and missing-email blockers are explained in the
    completion notes. The removed profile's disabled flag remains unexplained;
    replacement setup does not establish a fix for that historical state.
  - [x] October 22 fresh Google browser-consent check: operator reports the
    guided new-remote Google Online browser authentication and Test passed,
    then saved successfully. Accept native fresh Connect/auth/Test/Save;
    existing OAuth-update and mirror-transfer acceptance remain separate.
- [x] October 22 Offline interval field clarity: keep the existing width, add
  visible “Sync interval in minutes” above the shared Add/Modify field and clarify
  interval timing in its hover help. Check, focused test, localization and gate
  pass (gate rerun after an unreproduced process-test failure). Operator installed
  the update and confirms the label is visible; hover-help and Flatpak checks remain separate.
- [x] October 22 native Google/Box Offline scheduled downloads: both small test
  mirrors used 15-minute intervals; operator confirms cloud test files arrived
  locally automatically, without Sync Now, then turned both test toggles off.
  Scheduled uploads remain separate; the interval label change was not installed
  during this test.
- [x] October 22 acceptance reconciliation: reuse the conducted native checks
  and retain source/local-engine/package boundaries. See the current
  [acceptance matrix and residual gaps](Task%20List%20Completion%20Notes.md#055-acceptance-reconciliation-october-22-2026).
  Broader window/scaling and extended lifecycle cases remain in the validation/
  expansion backlog; selectable root-only/shallow preload remains skipped.
- [o] Installed regression acceptance reconciled: failed-Preview toggle passes
  natively below; retained retired-profile missing-service acceptance transfers
  to the legacy removal package-specific upgrade item above by operator agreement
  October 22. Source/fake checks already pass; package acceptance remains open.
  - [x] October 22 native failed-Preview toggle: temporarily renamed Google
    test subfolder caused Preview exit 7; operator confirms toggle off and
    Repair available. After Repair and restoring the original folder name,
    Preview and Sync Now passed. Retired-profile and Flatpak checks remain open.
- [o] Google/Box installed whole-root cloud mirroring deliberately untested:
  October 22 operator cannot mirror these large accounts. Retain the explicit
  live-validation limit; reuse disposable real-engine root/recovery tests and
  installed cloud subfolder results at their original scope. No whole-account
  transfer is required to proceed; do not claim cloud-root acceptance.
- [ ] SharePoint whole-library installed acceptance remains separate: use a
  suitable small library or record an explicit limit. Archive-subfolder success
  does not establish whole-library acceptance.
- [ ] October 22 0.5.5 release preparation started: finalize source review and
  release metadata, freeze approved application/packaging commits and update the
  old Flatpak pin before the single release build. Use the combined package
  checklist in `docs/engineering/cosmic-submission-audit.md`; install/upgrade
  acceptance, tag and publication remain separate. COSMIC repository PR later.
  - [x] October 22 README refresh: align current controls, engine retirement,
    setup/authentication, remote management and scheduling with 0.5.5. Preserve
    operator screenshot/layout edits; distinguish published downloads from candidates.
  - [x] Read-only Flatpak preflight: all four required x86_64 refs available;
    dirty packaging checkout blocks build. Old 0.5.4 app pin identified separately.
    Cargo vendor parity tests pass; 23 declared grants and retained host-engine
    rationale reviewed. No build, install or dispatch performed.
  - [x] Approved source freeze: b43f316 records 0.5.5 implementation/tests/docs;
    Flatpak app pin updated to its full SHA in a separate packaging commit.
    Push Buttons remains untracked. Final release metadata/package acceptance
    are pending; no push, tag, build or publication implied.
  - [x] October 22 release metadata finalized for package preparation: stable
    AppStream 0.5.5, Debian unstable entry/date and versioned release notes/index.
    Native/package evidence and whole-root limits remain explicit. Operator
    removed Push Buttons after its inventory moved to engineering documentation.
    Finalized app inputs committed as 13e204e; Flatpak pin refreshed to its full
    SHA before building. Package acceptance, builds and publication remain separate.
  - [x] October 22 pushed cd1b465 after upstream fetch/reconciliation (0 behind,
    4 ahead), verified remote identity and dispatched Debian run 39842941118
    (six targets) and Flatpak run 39842934431 (both architectures). Both runs
    were in progress at confirmation; artifacts/acceptance are not yet passed.
  - [x] October 22 Flatpak run 39842934431: both architectures built
    successfully. Maintained candidate inspector downloaded and verified source/
    provenance/checksums/metadata for both bundles; receipt release-promotion-4fdzna92.
    Installed package acceptance remains pending. Debian last observed 3/6 passed
    (all arm64), amd64 still running; existing tracker continues, no redispatch.
  - [x] October 22 Ubuntu 26.04 amd64 retry: operator approved canceling
    the slow dependency-download job after about 38 minutes. Same run 39842941118
    attempt 2 retries only that target at cd1b465 (job 120068845122). Five other
    targets retained success. Retry and six-package download/verification passed;
    receipt target/artifacts/ci-39842941118/receipt.json. Installed acceptance pending.
  - [x] October 22 test-computer Debian smoke: operator confirms installed
    package version 0.5.5 and panel applet loads/looks as expected after guided
    Ubuntu 24.04 amd64 package installation. Native UI smoke only; independent
    running-process identity, Settings, connection lifecycle and retained-profile
    upgrade checks remain separate. Duplicate Flatpak removal not confirmed.
  - [x] October 22 test-computer Debian Settings/editor smoke: following
    gear/editor guidance, operator reports all looks as expected. Accept reported
    Settings/editor appearance and absence of communication warning; backend
    connection lifecycle and retained-profile upgrade acceptance remain separate.
  - [x] October 22 test-computer Debian Online smoke: operator reports
    Test and Mount succeeded, then Unmount returned the toggle to off. Provider
    and file-list/content observations were not separately specified; accept this
    reported package lifecycle only. Mirror/legacy upgrade/Flatpak remain separate.
  - [x] October 22 test-computer Debian Offline smoke: operator reports
    the guided small-subfolder Preview/Sync Now/download test passed. Accept
    reported manual mirror package smoke; scheduled worker, retained-profile
    upgrade and Flatpak acceptance remain separate.
  - [x] October 22 test-computer Flatpak runtime/UI smoke: screenshot of
    flatpak ps identifies Cloud Mounter running in the sandbox; operator confirms
    popup/Settings/editor look okay after guided checks. Exact installed OSTree
    commit and connection/portal/worker/upgrade checks remain separate.
  - [x] October 22 test-computer Flatpak Online smoke: operator reports
    the guided Test/Mount/folder listing/Unmount check passed, with toggle off.
    This accepts the scoped installed sandbox Online lifecycle; mirror, exact
    installed commit and retained-profile/upgrade checks remain separate.
  - [x] October 22 test-computer Flatpak business OneDrive Offline: operator
    reports Sync Now completed, then a cloud-deleted file also disappeared
    locally. Accept reported sync completion and cloud-to-local deletion;
    recovery-copy presence, initial download and scheduling remain separate.
  - [x] October 22 test-computer Flatpak business OneDrive recovery: operator
    confirms the cloud-deleted file is preserved in the configured local recovery
    folder. Presence accepted; restore/content integrity and scheduled execution
    were not separately tested.
  - [x] October 22 test-computer Flatpak exact identity: operator confirms
    installed OSTree commit b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098,
    matching the inspected x86_64 candidate from run 39842934431. Scoped sandbox
    UI/Online/manual business-mirror deletion/recovery results apply to this artifact.
  - [o] October 22 operator skips the 15-minute final Flatpak scheduled-worker
    check. Record exact-candidate scheduled execution as untested; reuse earlier
    timer evidence only at its original scope. Manual Flatpak sync/recovery passed.
  - [x] October 22 tag preparation: approved isolated release/0.5.5 branch
    and v0.5.5 tag point to built packaging cd1b465. Maintained release script
    reused verified Debian run 39842941118; no rebuild/publication. Receipt:
    target/task-runs/release-prepare-v055/summary.json. README/evidence remain
    later master commits; package upgrade limitations remain explicit.

## Release installer follow-up — October 23, 2026

- [x] Update Debian and Flatpak download installer defaults/help to published
  0.5.5, retaining explicit version overrides and fail-closed verification.
  Run focused fake-adapter tests; no live installation or release rebuild.

## 0.5.5 Flatpak Pull Request, AI consideration

Design documentation follow-up requested October 22: create root `Design.md`
for engineers preparing host-integration review; inventory every Rust source
file, explain Online/Offline execution and distinguish source design from tests.

- [x] Write and check the architecture/source guide, linking it from README and
  the engineering index. Root location is explicitly requested by the operator.

Deferred from 0.5.4 on October 19, 2026. Refresh the submission against the
accepted 0.5.5 source after legacy removal and GUI cleanup; reuse unchanged
evidence. GitHub bundle distribution in 0.5.4 does not imply COSMIC approval.

- [x] October 22 refresh against published 0.5.5: manifest/app/package identities,
  host-command inventory, dependencies and permission rationale reconciled in
  docs/engineering/cosmic-submission-audit.md. Existing manifest is correct;
  no source/grant change or rebuild. Scoped installed evidence and limits explicit.
- [ ] Recheck current COSMIC contribution rules before submission; preserve the
  operator-authored PR/reply boundary below. Read release/packaging procedures
  before candidate preparation; link reused receipts and identify changed-input
  checks and remaining installed/package acceptance explicitly.
- [ ] Prepare the actual COSMIC repository submission after the planned 0.5.5
  changes; operator authors and acknowledges the contribution. Obtain maintainer
  decisions on host execution/dependencies and runtime/portal compatibility.

### PR Submissions Rules

**Your app code and its Flatpak manifest:** AI assistance or generation is permitted under that exception.

**The submission itself:** 
- [x] You should personally write the PR description and review replies. 
- [x] AI-generated changes to COSMIC Flatpak repository’s shared machinery, are outside the stated exception. For Cloud Mounter, "No shared build-system changes are currently identified as necessary." 

**Your responsibility:** You must understand the submitted changes, explain them during review, and accurately describe what was tested. Using AI does not transfer that responsibility.

**Licensing and provenance:** You must be able to make the required certification about your right to contribute the material. The Developer Certificate of Origin concerns contribution rights and licensing; it does not require that you manually typed every line.

### PR description layout:

Title: Identify the applet being added.

Purpose
- What does Cloud Mounter let users do?
- Why does it belong in the COSMIC applet repository?

Package
- Which version and source commit does the manifest build?
- Which runtime does it use?
- Which dependencies must users install on the host?

Permissions
- Why does it need NetworkManager and systemd access?
- Why does it need host execution?
- How are local directory permissions granted?

Testing
- Identify the installed Flatpak candidate.
- Summarize the operations actually exercised.
- State relevant limitations and skipped checks.

## 0.5.5 validation and expansion backlog

Originally recorded October 1, 2026 as follow-ups to the native SFTP release.
Reconciled October 22 after publication of 0.5.4 and 0.5.5; remaining work carries
forward without changing its planning date. These are not release gates unless
changed behavior or an observed defect makes a focused check necessary.

Completed scoped native keyboard/focus, light/dark, editor actions, OneDrive
setup and Debian/Flatpak smoke checks remain in the preceding acceptance record
and completion notes. Reuse them rather than repeat the whole matrix. Ordinary
panel/Settings/editor rendering does not establish exhaustive accessibility or
all-provider sandbox acceptance.

- [ ] Extend installed Flatpak coverage beyond the accepted 0.5.5 UI/Online and
  manual business OneDrive mirror/deletion/recovery smoke: provider-specific
  setup/browser consent, portal-grant persistence, generated host service/timer
  behavior, and clean-profile mount/mirror/VPN/uninstall combinations. The final
  0.5.5 scheduled-worker run was deliberately skipped, not passed. Reuse earlier
  tests at their recorded scope; local prototypes are separate evidence.
- [ ] After COSMIC repository acceptance, check Store discovery, installation,
  updates and removal from that channel. GitHub bundle availability and its
  installed smoke checks do not establish COSMIC Store acceptance.
- [ ] If later needed, broaden installed SFTP error-notice checks to host-key,
  unreadable-file, permission, agent, and timeout variants, and live-check a
  VPN-bound SFTP connection. Direct protocol and automated classifier tests
  already cover these errors; they are not repeat gates for the native release.
- [ ] Broaden Google Drive `--fast-list` testing to a larger ordinary drive and
  shared drive, including API/rate-limit behavior; repeat cloud cold-cache
  preload trials without reading or changing file contents. Root-first browsing
  improvement already passed natively; selectable shallow-depth scopes remain
  deliberately out of scope.
- [ ] Live-check Box and SMB preload on suitable cloud/LAN shares, and shared
  cancellation on OneDrive (onedriver), Box, SMB, and Google Drive's RC refresh
  when suitable active-preload shares are available.
- [ ] Extend native/Flatpak Settings/editor visual coverage to alternate scaling,
  long lists, long or localized titles/labels, all-window keyboard/focus behavior,
  editor-save edge cases and background status updates. Native Return/Tab/focus,
  light/dark appearance at usual scaling, shared Save confirmations and scoped
  packaged Settings/editor smoke already passed. OneDrive rclone Offline setup
  exists; new/existing-remote verification has native acceptance.
- [ ] Exercise the full Cisco/NetworkManager password/MFA, concurrent-request,
  and rejected-credential path with a real VPN profile; add the separately
  planned waiting/Cancel/Retry UI before accepting that extended workflow.
  Include a VPN-bound SFTP connection when that combination is available.
- [ ] Broaden the shipped rclone OneDrive Offline mirror's testing: very large
  trees, shared-with-me and SharePoint-backed libraries, personal vault, Graph
  throttling (HTTP 429) and retry behavior, Office-document metadata changes,
  and tenant conditional-access/consent policies. These scenarios do not imply
  additional supported target types; review support boundaries before testing.
- [ ] Evaluate whether a debounced local-change trigger (starting one scheduled
  bisync after local edits) is worth a separate watcher, since scheduled polling
  replaces abraunegg's immediate local upload. Measure CPU/power first; do not add
  a second writer to a mirror.
- [ ] Exercise repeated logind/inhibitor and native/Flatpak lid/menu suspend
  fault cases, then investigate the separately reported onedriver (OneDrive
  Online) sleep hang with journal evidence. Reuse normal/idle sleep observations
  only at their recorded scope; see the [historical acceptance map](docs/engineering/release-054-acceptance-audit.md)
  and [archived execution notes](docs/archive/task-list-completion-notes-through-v0.5.4.md).
  Those observations do not establish active-transfer or repeated fault handling.

# Cloud Mounter Applet Future Task List

Items that will be activated at some time in the future.

## Future Tasks depending on Pop!_OS or COSMIC desktop upgrades

- [ ] 25.08 is still maintained, but it is no longer the newest Freedesktop branch. Freedesktop 26.08 was released September 1, 2026;
- [ ] Remove every `flatpak document-info` invocation after the minimum
  supported Pop!_OS release provides Documents portal v5 or newer.
  - Verify the supported host exposes sandbox-enabled `GetHostPaths` and repeat
    the native and installed-Flatpak folder-selection tests before removing the
    portal-v4 fallback.
  - Remove the fallback executable path, parser, permission rationale, tests,
    translations, and inventory entry; retain focused portal D-Bus coverage.
- [ ] Remove the OpenConnect-only `nm-applet` dependency after COSMIC provides a
  complete NetworkManager OpenConnect authentication experience.
  - Do not use a COSMIC or Pop!_OS version number alone as the gate. Verify that
    applet-initiated NetworkManager D-Bus activation opens every gateway-defined
    field, including username, password, and second-factor method, completes
    phone authentication, displays gateway messages, and leaves no stale generic
    single-field prompt.
  - Exercise cancellation, rejected or expired second factor, timeout,
    reconnect, logout/login, and competing-agent behavior from both the native
    package and installed Flatpak.
  - Keep the host NetworkManager OpenConnect plugin and authentication dialog;
    remove only the `nm-applet` workaround when COSMIC's agent reliably handles
    NetworkManager's secret request and authentication UI.
  - Remove OpenConnect-specific `nm-applet` session-bus detection, missing-agent
    errors, package/startup/autostart guidance, tests, translations, and README
    instructions. Do not remove or modify a user's autostart file automatically.
  - Retest non-OpenConnect VPN profiles and prove they remain independent of
    either authentication-agent implementation.

- [ ] Remove the Ubuntu 22.04 matrix entries no later than GitHub's scheduled
  runner retirement on April 17, 2027.

## Future Tasks for project cleanup

Start this section only after the Flatpak D-Bus host-integration migration,
including Tasks (5) and (6), inventory closure, rebuilt-Flatpak installation,
and live acceptance, is complete. Until then, continue using `Task List.md` as
the active implementation and verification record.

- [ ] Make `Applet Description.md` and `Requirements and Specifications.md`
  self-contained descriptions of the accepted product and final architecture.
  - Move every durable behavior, safety boundary, migration rule, and acceptance
    criterion that exists only in the task list into the appropriate source
    document.
  - Replace references that delegate required behavior to `Task List.md` with
    the actual requirement or a stable supporting document.
- [ ] Separate current supported scope from future and acceptance-pending scope.
  - October 15 progress: the status matrix now lives only in `Applet Description.md`;
    the specification carries no status, date, release or "planned" wording.
  - Add a concise status matrix for implemented, pending live validation, and
    future providers and features, including SFTP, SharePoint, Flatpak, and VPN
    authentication.
  - Remove obsolete `planned`, version-specific, or conditional wording after
    the corresponding decision and acceptance gate has closed.
- [ ] Reconcile the documentation with the final implementation.
  - Audit the applet description, requirements, README, Flatpak packaging guide,
    manifest rationale, `docs/engineering/flatpak-host-command-inventory.md`, configuration model, translations,
    and tests for contradictions or stale command paths.
  - Ensure the final host-command and permission descriptions agree with the
    closed inventory and contain no removed Cisco, `nmcli`, `systemctl`, journal,
    shell-wrapper, `abraunegg/onedrive` or WebKit authentication-helper behavior.
  - Resolve every remaining permitted architecture alternative, or clearly mark
    it as an intentional supported variation with equivalent acceptance tests.
- [ ] Restore clear document roles without discarding history.
  - Keep `Applet Description.md` as the product source of truth and
    `Requirements and Specifications.md` as its testable technical translation.
  - Keep the active task list concise and actionable; move completed chronology,
    failures, decisions, and validation evidence to
    `Task List Completion Notes.md` rather than deleting it.
  - Add requirement-to-test traceability for safety-critical mount, mirror,
    systemd, portal, VPN, and Flatpak behaviors.
- [ ] Validate the cleanup with link/anchor checks, Markdown and diff checks, a
  stale-term scan, and a final human review confirming that the applet could be
  rebuilt with equivalent user-visible behavior and security boundaries without
  treating the task list as a design specification.
