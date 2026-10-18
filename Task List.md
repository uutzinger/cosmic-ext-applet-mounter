# Cloud Mounter Applet Task List

This document tracks the current work-item checklist. Work through the native
v0.5.3 release is archived in `docs/archive/` (see "Archive and open items carried
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

Only the open items are listed here, condensed to one line each; this list is
authoritative for their status. Full wording, plus the items already triaged `[x]`
or `[o]`, is in [Task List Carry Forward.md](Task%20List%20Carry%20Forward.md) as of
October 15, 2026 (read it only when an item's detail is needed). Triage each item as
`[x]`, `[o]` (with a reference to its authoritative task) or keep it open.

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

## Release 0.5.4 — Engine decision and Flatpak completion

Planned October 13, 2026. Authoritative home for deferred Flatpak work from
Tasks (5)/(6) and 0.5.3. Prior [x] receipts remain historical; transferred [o]
steps are not completed. Start with architecture, not more WebKit/auth code.

### Engine decision and OneDrive Offline migration to rclone — decided October 14, 2026

Decision (user): drop `abraunegg/onedrive`. It has no clear authentication path
for both personal and corporate accounts, and the employer does not support its
Microsoft Graph access requirements. OneDrive Offline mirrors move to `rclone`,
reusing the SharePoint (Teams) bisync path. `jstaf/onedriver` stays for OneDrive
Online. No other mount engine is added or changed.

Release boundary: **0.5.4 introduces rclone for new OneDrive Offline mirrors and
retains existing abraunegg mirrors for one deprecation release.** Phase 5 and
phase 6 below, plus Consolidated Flatpak acceptance and release, are 0.5.4 work.
Actual engine/helper/dependency removal is
[post-0.5.4 work](#post-054-legacy-abraunegg-engine-removal), targeted at the next
release (tentatively 0.5.5), not a 0.5.4 gate. No automatic conversion is planned.

| Provider | Online mount | Offline mirror |
|---|---|---|
| OneDrive | `jstaf/onedriver` | `rclone bisync` (new; replaces `abraunegg/onedrive`) |
| SharePoint (Teams), Google Drive, Box, SMB, SFTP | `rclone mount` | `rclone bisync` |

- [x] Continuous/event-driven mirroring is not required. Scheduled bisync plus
  Sync Now meets product intent (abraunegg already polled the remote every 300 s
  in `sync.rs`; the loss is immediate local-change upload and delta efficiency).
- [x] Drop abraunegg/onedrive; keep jstaf/onedriver for Online; rclone for every
  other Online and Offline path.
- [o] Retain abraunegg with host-script authorization: not pursued. Dropped by the
  decision above.
- [o] Decide whether WebKit packaging is justified: no. The WebKit auth helper
  leaves with abraunegg (see removal below). Probe evidence stays historical.

#### Open decisions

- [x] Legacy timeline (decided October 14, 2026): existing abraunegg connections
  keep running in 0.5.4 but cannot be newly created; the engine is removed in
  the following release (tentatively 0.5.5), under the post-0.5.4 section. No conversion tool: the applet has no established user
  base, so users recreate the connection as an rclone mirror manually.
- [x] Mirror guard rule (decided October 14, 2026). The SharePoint guards came from
  SharePoint tests and do not transfer to OneDrive, so:
  - Safety guards are always on: the identity guard (verified drive ID checked
    before every run) and the UUID-bound access marker (local and remote). The
    initial Preview plus confirmed initial sync, recovery copies and `--max-delete`
    are already generic.
  - Compatibility workarounds (`--ignore-size`, `--ignore-checksum`,
    `--modify-window 1s`, `--check-sync false`, `--compare modtime,checksum`, the
    convergence pass) are enabled for a drive type only if its test shows an
    unstable second sync with generic settings and a stable one with them.
  - Full-tree checksum verification runs after the initial sync and on manual
    Verify, not on every scheduled run, unless the runtime measurement shows it is
    affordable.
  - Implementation uses one small per-drive-type profile, defaulting to the
    generic settings. The test results fill in the table below.

  | Drive type | Compatibility profile | Evidence |
  |---|---|---|
  | Personal OneDrive | generic (`--compare size,modtime`) | October 15, 2026: `target/task-runs/onedrive-guard-check-8zm9owl_/results.json`; original/download bytes and sizes match; no-edit/post-edit transfers 0 |
  | OneDrive Business | generic (`--compare size,modtime`) | October 15, 2026: `target/task-runs/onedrive-guard-check-xpn4ex9d/results.json`; original/download bytes and sizes match; no-edit/post-edit transfers 0 |

- [x] Microsoft app registration (decided October 14, 2026): use rclone's default
  client only; no client ID, secret or tenant fields in the OneDrive Offline
  editor. Custom app registration does not work for the corporate storage, and the
  SharePoint rclone mirror already works with the default client. Accepted risk: a
  tenant that blocks rclone's shared client cannot use OneDrive Offline, and there
  is no in-app fallback.

#### 1. Contract and requirements (before implementation)

- [x] Write `docs/engineering/onedrive-rclone-offline-contracts.md`: persisted
  configuration (rclone remote reference, drive type, subpath, legacy-engine
  marker with a serde default for old files), setup/auth flow, safety guards from
  the open decision (the per-drive-type profile table is completed after phase 2),
  repair semantics, and the legacy handling rules. Link it from
  the engineering README and the requirements. Drafted October 14, 2026; the
  compatibility profile table is completed after phase 2.
- [x] Update `Applet Description.md` (engine matrix, Offline sections, lines
  referring to `abraunegg/onedrive`) and `Requirements and Specifications.md`
  (engine matrix, 4.2 Offline behavior, dependency list, FR-060A–C Repair), plus
  `README.md`, `docs/dependency-installation.md` and `docs/index.md`. Done October
  14, 2026 as target-state requirements (new FR-031C; FR-031/031A/031B/053/060C
  revised) with planned-change notes in the user docs and Flatpak guide; the
  shipped-behavior text changes with the implementation and removal phases.
  `docs/index.md` needed no engine change.

#### 2. Mirror guard check (live, operator-run; after the contract draft)

Settles the per-drive-type compatibility profile in the open decision above. The
script creates and purges only its own `CloudMounter-GuardTest-<token>` folder.

- [x] Prepare: one rclone remote per drive type using the default client
  (`rclone config`, type `onedrive`, personal and business), a dedicated test area,
  and a directory of real `.docx`, `.xlsx` and `.txt` files (at most 20 files and
  5 MB; real Office files reproduce service rewrites better than the synthetic
  ones the script generates when `--files-dir` is omitted). Verified October 15,
  2026; sample counts, format checks and remote evidence are in the completion notes.
- [x] Validate each remote without changes:
  `bash scripts/onedrive-mirror-guard-check.sh --remote NAME --drive personal --plan`
  (and `--drive business`). Both operator-run plans passed October 15, 2026;
  receipts are recorded in the completion notes.
- [x] Run once per drive type:
  `bash scripts/onedrive-mirror-guard-check.sh --remote NAME --drive TYPE --files-dir DIR --web-edit`.
  Business and personal passed October 15, 2026 with generic settings.
  Generic stable: that drive type uses the generic profile. Generic unstable and
  compat stable: it needs the compatibility profile. Neither stable: investigate
  before choosing guards. Do not reuse the SharePoint results.
- [x] Record each verdict, date and `results.json` path in the profile table of the
  open decision, then copy the confirmed profiles into the contracts record.
- [x] If the run exposes a script gap, fix and test the script instead of working
  around it, and record the change in the completion notes.
  October 15: missing remote-folder creation before initial bisync fixed;
  six focused tests, formatting, lint and broader tests passed. Business live
  retry and personal live run both passed.

#### 2a. Initial-upload round-trip and unnecessary-transfer check (added October 15, 2026)

Extend `scripts/onedrive-mirror-guard-check.sh` and its Python module, rather than
using ad hoc commands. Phase 2 established stability after initial sync, not
original-byte preservation during upload or absence of redundant transfers.
Follow-up passed for both drive types October 15, 2026; generic profiles are
confirmed for the tested sample set and observation window. Preserve the earlier
completed checks and their receipts.

October 15: extension, version-provenance fix and verification completed. Both
operator-run extended live tests passed (exit 0); dated receipts, comparisons and
transfer counts are in the completion notes. This closes phase 2a; phase 3 remains
outstanding.

- [x] Add a separately recorded round-trip stage. Before upload, create an isolated
  baseline from the samples; record relative paths, byte lengths, SHA-256, timestamps
  and a common remote-supported checksum. Never write to the sample source; verify
  its hashes again at the end.
- [x] Upload copies to a fresh script-owned subtree; capture remote-reported sizes,
  timestamps and available checksums. Download every file into a separate empty
  directory without size/checksum-based skipping. Compare actual downloaded sizes
  and SHA-256 with the untouched baseline, and the common checksum separately.
  Distinguish reported-size discrepancies, timestamp drift and actual byte rewrites.
  Missing files, unavailable evidence or command failures cannot produce a pass.
- [x] Use a separate fresh pair for initial bisync, preserving its pre-sync baseline.
  Measure initial changes, then at least two no-edit syncs and the existing web-edit
  and settling stages. Record structured transfer counts/bytes and snapshots per
  stage: unchanged snapshots alone do not prove zero transfers. Exclude deliberate
  diagnostic downloads from sync transfer totals. Define bounded service-settling
  observations and record timing; unresolved drift is inconclusive.
- [o] If generic settings cause repeated transfers or drift, test compatibility
  settings on independent fresh trees with identical samples. Identify the smallest
  justified settings and any required convergence pass. A size discrepancy or byte
  rewrite alone does not prove a workaround is needed. Ignoring size/hash for sync
  must not bypass downloaded-byte verification; explain rewrites before accepting
  a profile, otherwise leave the result unresolved. Not triggered: both extended
  live runs passed with generic settings and zero idle transfers; no SharePoint
  workaround or convergence pass is justified by these results.
- [x] Add fake-rclone tests for unchanged round trips, reported-size-only differences,
  byte rewrites, redundant transfers despite unchanged snapshots, missing evidence,
  transfer failures, cancellation and partial cleanup. Preserve marker-scoped
  deletion and original samples; retain partial failure evidence. Version results,
  record tool/source identity and effective settings, update help/catalog/inventory,
  then run focused tests and the routine gate.
- [x] Run separately for personal and business with operator web-edit handoffs.
  Record dated results paths, original/downloaded comparisons, transfer counts,
  cleanup and limitations in completion notes. Finalize both profile tables from
  these results only. No whole-drive sync or installed-app acceptance is implied.

#### 3. Prototype on disposable accounts (no real data)

October 15: isolated development harness implemented and verified. Per operator
steering, reuse the existing SharePoint planner/coordinator transaction path,
initial Preview/confirmation, anchors, recovery, retention and cancellation.
OneDrive uses its verified identity and generic profile, without SharePoint
workarounds/convergence. Applet routing and saved formats remain phase 4.
Prototype source/fixture checks do not complete live native/Flatpak-style acceptance.

October 15 follow-up: first personal inspect returned invalid_metadata before
identity verification. Private in-memory config-response capture is fixed;
13 focused tests and the full gate passed. The operator retry reached Graph,
as recorded below; no cloud mutation occurred during diagnosis.

The retry reached Graph but returned HTTP 401. October 15 follow-up implemented:
the prototype's /me account lookup needs User.Read, absent from rclone's default
file scopes. New prototype OAuth remotes request explicit profile consent;
existing remotes stay unchanged. Eleven focused prototype tests and the full gate
passed, including refusal without mutation. New native personal OAuth and identity
verification passed October 15; business and Flatpak-style acceptance remain open.
HTTP 401 alone does not establish its exact provider-side cause.

- [x] Implement and verify the isolated `onedrive-mirror-prototype.sh` harness;
  10 focused Rust tests (including explicit local engine integration), 2 driver
  fixtures and the full gate passed. Receipts are in completion notes; the following
  cloud checks remain outstanding.
- [x] Native personal OneDrive browser OAuth and signed-in account/primary-drive
  identity verification on the new prototype remote; October 15 receipt in
  completion notes. No mirror lifecycle acceptance is implied.
- [x] Native business OneDrive browser OAuth and signed-in account/primary-drive
  identity verification on a separate prototype remote. October 15 attempt
  encountered admin approval; prototype added User.Read while working SharePoint
  and business remotes use default file scopes. Business auth now uses default
  scopes and server drive/owner binding without profile access; 15 focused tests
  and the full gate passed. Operator business auth retry passed October 15;
  account/primary-drive binding accepted; native scenario results follow below.
- [x] Native business disposable-session preparation and shared initial Preview;
  October 15 passed on session 8v4nz4pi.
- [x] Native business initial bisync and independent downloaded-byte verification;
  October 15 passed on session 8v4nz4pi.
- [x] Native business disposable scenario: edits/deletes both sides, conflicts,
  16 MiB upload and byte verification; October 15 passed. Synthetic quiet sync
  22.7 s, sync plus checksum verification 31.9 s; cadence remains undecided.
- [o] Repeat shared native cancellation/network-failure/Repair tests for business:
  deliberately omitted per October 15 operator instruction; use the accepted
  personal fault tests below for shared coordinator/runner coverage. This is
  not business-specific live fault acceptance; repeat only for changed behavior
  or evidence of a drive-specific failure.
- [o] Flatpak transport/OAuth checks transferred October 15 to
  [Implementation](#4-implementation), after routing, setup, coordinator, units
  and editor integration. No separate prototype environment is required before
  implementation; sandbox acceptance remains outstanding there.
- [x] Clean up business normal-scenario cloud tree, session 8v4nz4pi;
  October 15 purge succeeded, local evidence retained.
- [x] Clean up personal normal-scenario cloud tree, session gzpgo0w6;
  October 15 purge succeeded. All four native prototype sessions now report
  cleaned=true; local evidence and configured remotes retained.
- [x] Native personal disposable-session preparation and shared initial Preview;
  October 15 receipt in completion notes. Initial sync completed, but its action
  failed in a subsequent guard before byte verification. Safe command diagnostics
  and a verification-only action passed 12 focused Rust tests, 2 driver tests and
  the full gate. Operator verification retry passed October 15 on this session.
- [x] Native personal initial bisync and independent downloaded-byte verification;
  successful verify retry recorded in completion notes. The earlier post-sync
  guard failure remains in history with its cause undetermined.
- [x] Native personal disposable scenario: edits/deletes both sides, conflict
  preservation, 16 MiB upload and independent verification; October 15 passed.
  Synthetic quiet sync 31.2 s, sync plus checksum verification 47.0 s; these do
  not establish representative-tree performance or a verification cadence.
- [x] Native personal cancellation test: prepare and initialize a fresh session,
  run fault-cancel, then inspect Repair evidence. Started October 15;
  fresh preparation/Preview, initial-sync and downloaded-byte verification passed
  on session zz9bn84r. fault-cancel passed with cancelled coordinator state and
  16 MiB pending data retained. Repair inspection passed: pending file present,
  no bisync locks remained, no implicit resync. Disposable cloud cleanup passed;
  local evidence and pending data retained.
  Preserve the completed scenario session.
- [x] Native personal network-failure test: fresh preparation/initial-sync,
  fault-network, Repair inspection and explicit cleanup. Preparation/Preview
  and initial-sync/downloaded-byte verification passed on session i7ulanzh;
  fault-network passed with failed coordinator state and 16 MiB pending file
  retained. Repair inspection passed: pending data retained, no bisync locks,
  no implicit resync. Explicit cloud cleanup passed; local pending data and
  evidence retained. Failure injected only in the bisync child; host networking
  unchanged.
- [o] Combined native disposable-tree lifecycle checklist: normal behavior covered
  by both scenarios above; shared faults/Repair covered by personal sessions.
  Remaining transport, cleanup and representative measurements are listed
  separately; do not rerun the completed scenarios.
- [o] Larger-tree timing and repeated five-minute runs deliberately skipped by
  operator decision October 15: accept the measured small-project baseline below
  and reuse existing SharePoint scheduling coverage for unchanged shared machinery.
  October 15: sample-folder preparation and dedicated timing action implemented;
  17 focused Rust tests, 2 driver tests and the full gate passed. Business sample
  preparation/Preview passed on session v02yqke_ (317 files, 39,779,429 bytes).
  Initial-sync (327.3 s) and downloaded-byte verification (33.0 s) passed.
  Timing passed: quiet sync 36.0 s, sync plus checksum 52.0 s, small-change sync
  38.2 s; cloud cleanup passed, local evidence retained. Original-copy hashes
  matched at preparation.
  Copy originals read-only; do not repeat functional scenarios.
  No claim of larger-tree measurements or live five-minute cadence validation.
- [x] Business small-project timing baseline on 317 copied files completed
  October 15; quiet/small-change/checksum measurements recorded in completion
  notes. Timing preparation is complete under the operator's accepted scope above.
- [x] Confirm minimum rclone version and flags (October 15): retain >= 1.74.3;
  pinned upstream source supports the generic OneDrive profile. Installer,
  diagnostics policy constant and Debian dependency already agree; no minimum
  change needed. Live cloud evidence is from 1.75.0, not a 1.74.3 runtime test.

#### 4. Implementation

Follow the [accepted contract](docs/engineering/onedrive-rclone-offline-contracts.md);
reuse existing SharePoint setup/coordinator/services/UI components. Both OneDrive
drive types use the same tested generic profile; only identity/auth differs.

- [x] Configuration and mode-aware engine selection: add optional
  `OfflineEngine`/`OneDriveIdentity`,
  preserve absent-engine legacy behavior, explicitly select rclone for new Offline
  connections and keep onedriver Online unchanged. Validate identity/engine/mode
  combinations and local/work/recovery overlap, including legacy connections.
  October 15: source implemented; ten focused tests, check, localization-check
  and full gate passed. Legacy serialization preserved. Production planner/worker
  activation remains in the setup/guards and runtime integration tasks below.
- [x] Shared planner safety: correct `--max-delete 1000` (bisync clamps it to
  100%, not 1,000 files); choose/test the intended percentage for SharePoint and
  OneDrive. Reuse shared recovery, ownership, retention and initialization logic;
  keep SharePoint compatibility flags/convergence conditional to SharePoint.
  October 15: all shared planner and guard profiles now use 50%, retaining
  recovery without an automatic override. All-provider planner/fake failure tests,
  guard fixtures, explicit local real-rclone 50%/60% checks on both sides, check
  and full gate passed. No installed services or cloud data were changed.
- [x] Shared Microsoft OAuth: extracted protected config/create/continue/token
  handling and new-remote rollback into `microsoft_oauth.rs`; production SharePoint
  wrapper and OneDrive prototype share it. Provider drive selection stays separate.
  October 15: seventeen OAuth tests, provider regressions, local-rclone credential
  capture, check, localization-check and full gate passed; no live sign-in repeated.
- [x] Common Microsoft identity policy: require resource/account binding, keep
  display profiles optional, and omit optional profile requests from recurring
  SharePoint guards. Retain personal OneDrive profile verification. October 15:
  contract updated; eleven SharePoint and fifteen OneDrive focused tests, check
  and full gate passed. Production OneDrive integration remains below.
- [x] Remote setup and guards backend: production `onedrive.rs` reuses shared
  OAuth and tested personal/business account checks; supports new/read-only
  existing remotes and rejects changed-drive replacements. Drive/type/subtree
  guards, UUID anchors and target-bound Preview/initialization state are wired
  into the shared planner/coordinator. OneNote/shortcut support limits and filters
  documented. October 15: 51 focused tests, local real-rclone lifecycle, check,
  localization-check and full gate passed. Editor/lifecycle integration and new
  production guard live/installed acceptance remain in the following tasks.
- [x] SharePoint source naming: renamed `teams.rs`/`teams_oauth.rs` to
  `sharepoint.rs`/`sharepoint_setup.rs` and updated Rust callers/types. Stored
  profile names, service flags and Fluent keys preserved. October 15: check,
  24 focused tests (including legacy named RON/JSON) and full gate passed.
- [x] Shared runtime integration: rclone OneDrive uses shared units, readiness/VPN
  guards, Start/Stop, Pause, Repair and Preview/Sync Now; legacy/Online routes
  preserved. Explicit Preview supplies bounded structured difference estimates
  (supersedes the abraunegg part of carried "Offline mirror counts"). Retained
  15-minute default, initial/manual checksum verification and paused Save; Start
  requires initialized state. October 15: check, focused lifecycle/metadata and
  offline regression tests, temporary local-rclone metadata test, localization
  check and full gate passed. Editor and installed acceptance remain below.
- [x] Add/Modify editor: OneDrive Offline uses personal/business selection,
  production rclone create/select and saved previous_identity for reconnect.
  Failed drafts remain unaccepted/disabled; cancellation and stale setup/Test/Save
  results are guarded. Legacy Offline and Online controls preserved. October 15:
  check, six editor tests, discovery/removal regressions, localization-check and
  full gate passed. Browser/rendered/installed acceptance remains pending.
- [x] Focused tests: mapped existing config/routing, units, identity/anchor,
  malformed/partial-state, rollback, filter and Online coverage; added both-drive
  reconnect, actual mid-setup cancellation/cleanup and late editor validation/
  saved-identity race cases. October 15: check, four added focused tests and full
  gate passed. Reused accepted live scenarios; installed acceptance remains below.
- [x] Dependency/docs reconciliation: guidance, diagnostics and host-command/script
  inventories distinguish new rclone mirrors, Online and legacy engines. Retained
  rclone >= 1.74.3 and legacy dependencies until post-0.5.4 engine removal; no new startup probes.
  October 16: check, focused diagnostic/installer tests, localization and full gate
  passed. Installed native/Flatpak acceptance remains below.
- [x] Integrated Flatpak implementation smoke (started October 16; transferred
  from phase 3 October 15; closed October 18): shipping-feature candidate built,
  installed and identity/metadata verified. Both OneDrive account flows passed;
  personal initial/ordinary/restart/scheduled/Pause/recovery checks passed,
  corporate initial transfer passed, and SharePoint initial/ordinary sync after
  restart passed. Shared native scenario/fault evidence reused without repeating
  unchanged suites. Portal host/local I/O correction and ownership anchors
  verified; paused Save and disposable connection removal verified, including
  absent systemd units and preserved local data. Candidate 848d48b pins source
  cf2d13b; recovery Browse placeholder observed in installed editor. Full
  chronology, failures/fixes, verification receipts and limitations are retained
  in completion notes. Remaining visual details and broader lifecycle/release
  acceptance belong to phase 6 below, not this implementation smoke.

Phase 4 closed October 18, 2026 under the operator's accepted smoke scope.
This is not full phase-6/native/Debian/Flatpak release acceptance. Retained test
resources and original failed profile are preserved; no cloud cleanup claimed.

#### 5. Legacy abraunegg deprecation — 0.5.4 (no conversion tool)

Source work completed October 18; check, focused compatibility/routing tests,
localization and final gate passed. Phase-5 source/tests and draft guidance are
included in this commit; no phase-5 package has been built or installed. Rendered
notice and installed legacy continuity remain phase-6 acceptance. No engine/
dependency removal or profile conversion.

- [x] Stop offering abraunegg for new OneDrive Offline connections; existing ones
  retain their engine for 0.5.4. Existing rclone-only new-draft routing verified;
  persistent Fluent deprecation/replacement notice added to legacy Modify editor.
  No new legacy selector, automatic conversion or changed legacy/Online route.
- [x] Guard against two engines on one tree: existing overlap guards verified for
  equal/parent/child mirror and recovery paths, both drive types and connection
  orders, including disabled legacy profiles. Separate engine plan namespaces
  and unchanged legacy serialization tested; no database/cache/token copying.
- [x] Draft manual replacement steps and two-release policy in
  [0.5.4 release notes](docs/releases/v0.5.4.md), including unsynchronized files,
  fresh directories/auth, Preview/initial sync and verified old-profile removal.
  Final release-note reconciliation remains phase 6.

#### 6. OneDrive migration acceptance — 0.5.4

Started and closed October 18 for the documented Flatpak migration smoke scope.
Operator transferred native/Debian and actual legacy monitor/transfer acceptance
into final release acceptance; neither is claimed as passed. Earlier numbering
was phase 7. Detailed guard/failure/fix chronology and receipts remain in
[completion notes](Task%20List%20Completion%20Notes.md).

- [x] Rendered legacy deprecation notice and replacement instructions accepted
  on installed a0cf060; legacy Preview passed (19 notable change lines), and
  existing onedriver Online setup/mount/list/unmount passed. Real legacy pending
  changes untouched; actual monitor/transfer remains a release gate below.
- [x] Final recovery guidance accepted on installed 848d48b: Browse placeholder,
  separate folder authorization help, no automatic recovery-directory promise.
- [x] Installed guard smoke: reciprocal personal/business drive-type mismatch,
  wrong expected personal account, local/remote ownership-marker loss, saved
  local retarget without Preview and missing remote folder all blocked as
  expected; original target restored and normal checksum-verified sync passed.
  Remaining malformed/resource/corporate variants reuse passing source/fake
  evidence from dev-gate-6_eqf2pd; not every case was live-tested per account.
- [x] Reuse phase-4 installed personal setup/initial/manual/restart/scheduled/
  Pause/recovery, corporate setup/initial transfer, SharePoint restart and safe
  disposable connection-removal evidence. No duplicate unchanged suites.
- [x] Repair/recovery-to-normal on installed 82c1d7a/source 67b3aec: hidden Repair
  integration defect fixed, compilation/three regressions/full gate passed;
  popup Repair completed, 17-file preservation baseline unchanged, units paused;
  subsequent normal sync/checksum passed without reinitialization/resync.
- [x] Focused idle sleep/wake: popup returned in same boot, all 17 checkpoint files
  unchanged before manual sync, post-wake sync/checksum passed, units disabled.
  Checkpoint: target/task-runs/phase6-sleep-wake-checkpoint/checkpoint.json.
  No active-transfer or scheduled-resume wake claim.
- [x] Focused manual VPN: cancellation blocked Save without persistence; positive
  OpenConnect authentication/Save and bound sync/checksum passed. VPN retained
  with disconnect permission off and released with permission on. Original No
  VPN/false options restored; initialization unchanged, units disabled/unloaded.
- [x] Reconcile OneDrive migration wording in draft 0.5.4 notes: new rclone mirrors,
  legacy support/deprecation retained in 0.5.4, manual replacement instructions,
  later removal and verified scope/limits. Publication metadata remains separate.
- [o] Native/Debian lifecycle and actual legacy monitor/transfer transferred by
  operator to [consolidated final acceptance](#consolidated-flatpak-acceptance-and-release).
- [o] Broader scheduled VPN/ownership/active-transfer lifecycle remains at those
  final gates. Retained dev-gate-s4od_kr5 covers VPN ownership/reference counting,
  bounded readiness, cancellation and engine dispatch with fake adapters;
  this is source evidence, not installed acceptance of unrun scenarios.

### Consolidated Flatpak acceptance and release

Ordered October 18 to combine final migration and package gates. Native/Debian
and actual legacy transfer checks were transferred here by operator decision;
they remain open. Reuse accepted phase-4/6 evidence and test only missing scope
or behavior affected by later source/package changes. Source/fake, native and
installed Flatpak evidence remain distinct.

#### 1. Reconcile evidence and freeze release inputs

- [x] Reconcile the [0.5.4 acceptance map](docs/engineering/release-054-acceptance-audit.md).
  October 18 operator chose reuse of completed Flatpak tests and elimination of
  redundant runs. Preserve existing target scope; no added architecture/provider
  test matrix. Missing evidence is reviewed first, with new live tests only for
  a concrete unresolved release gap or changed behavior.
- [x] Refresh 0.5.4 candidate source/manifest pins after helper cleanup and the
  Python3.10 CI fix: source 7ccc9f7, exact Flatpak pin updated. Combined gate
  dev-gate-5jl1l4eq, focused auth/packaging/installer/publication tests and
  localization passed; metadata retains known COSMIC diagnostics. Notes remain
  draft; build and final acceptance stay separate. Older ef5c4d8/28b0abd
  artifacts are retained pre-cleanup evidence, not selected shipping inputs.
- [x] Source/dependency baseline retained: all 620 Cargo registry archives/checksums
  and 12 dependency Git pins/selectors checked; generated inline/shell support
  fixed and verified. Latest installed development packaging 82c1d7a pins source
  67b3aec; build/install receipts are under target/task-runs/flatpak-candidate-82c1d7a/
  and flatpak-install-82c1d7a/. Earlier de8d779/cf5da32 evidence is historical.
  COSMIC BaseApp/Freedesktop 25.08 retained with unchanged metadata/grants.

#### 2. Build or reuse packages and inspect their contents

October 18 scope update: remove WebKit helper and helper-only dependencies now;
retain browser/manual legacy reauthentication and all legacy sync support. Current
28b0abd packages are pre-cleanup evidence; affected package inputs need replacement
after source checks. Do not repeat unchanged Flatpak behavior tests.

- [ ] Obtain required native/Debian and source-based Flatpak artifacts at the
  frozen revisions with shipping features only. Started October 18: Flatpak
  28b0abd/ef5c4d8 x86_64/master build/export passed. Authorized master push
  verified; hosted run 39166940625 failed on both 22.04 targets (missing tomllib),
  with four newer-target jobs passed. Python3.10 fallback/build dependency fixed
  alongside helper retirement; replacement source verification in progress. Reuse exact-source accepted
  artifacts; build only changed inputs or missing artifacts, once per matrix.
- [ ] Verify source/runtime identity, helper/worker payloads, size, checksums and
  matching Flatpak export/bundle identities. Inspect installed effective grants,
  including BaseApp/global overrides; no silent blanket host filesystem/bus
  grants or restoration of obsolete shell wrappers. Latest development candidate
  export/bundle/install identity already matches; payload/effective-grant review
  and final-release identities remain open.

  October 18 Flatpak payload/dependency review passed: six source-matched export
  files, correct 0.5.4 metadata, Platform ELF libraries available, unchanged
  runtime/policy and only read-only theme global overrides. Receipt:
  target/task-runs/flatpak-candidate-28b0abd/payload-review.json. Candidate not
  installed; this pre-cleanup payload is not the replacement shipping candidate.

#### 3. Verify authentication and sandbox access

- [ ] Prove selected engine authentication and sandbox mirror/recovery/state
  access through reviewed grants/portal authorization. Reuse phase-4/6 personal,
  corporate and SharePoint evidence where inputs are unchanged. Host-path
  resolution is not an access grant; verify Platform capabilities rather than
  SDK imports or host-installed packages alone.

#### 4. Complete mirror and legacy continuity acceptance

- [ ] Native/Debian: personal and business OneDrive mirror lifecycle (setup, sync,
  Pause/Resume, Repair, sleep/wake, VPN), plus onedriver Online unaffected.
  Run a focused installed personal/business smoke of changed OneDrive behavior
  and onedriver continuity. Reuse shared source/engine/Flatpak lifecycle evidence;
  no duplicate full sleep/VPN/Repair suite merely for a different package mode.
- [ ] Legacy background monitor/actual transfer acceptance; dry-run is accepted
  separately, and pending real-user changes have not been applied. Use a separate
  approved disposable legacy mirror; do not start the real mirror with pending
  user changes for this test.
- [x] Reuse phase-4/6 Flatpak personal/corporate/SharePoint Preview/initial/ordinary/
  scheduled/Pause/recovery, guards and Repair evidence at its recorded scope.
  No repeated setup/auth/sync/Repair suite for unchanged behavior.
- [ ] Review retained Task (5) worker/progress/cancellation/retention evidence and
  final package differences; perform a narrow live test only for an unresolved
  release gap. Do not label unrun scenarios as installed passes.

#### 5. Complete VPN and sleep/wake acceptance

- [ ] Complete Task (6) final OpenConnect positive VPN/mount/unmount/ownership and
  one negative authentication lifecycle. Reuse phase-6 manual mirror auth cancel,
  positive auth/sync and optional applet-owned release; keep native/prototype
  receipts separate from installed Flatpak evidence.
- [x] Reuse phase-6 idle wake and manual VPN cancellation/auth/sync/optional-release
  smoke with existing source/fake readiness, ownership and cancellation coverage.
  No additional laptop suspend or full VPN fault matrix required by default.
- [ ] Review whether package/runtime changes leave a concrete network/sleep gap.
  Scheduled VPN-unavailable, live shared/pre-existing ownership and active-transfer
  sleep/VPN-loss remain unrun live cases, not automatic mandatory reruns or passes.
  Add a focused live check only when the evidence review identifies a blocker.
- [ ] Decide Cisco rollback disposition only after relevant acceptance; no OS
  client removal is implicit.

#### 6. Complete Online, desktop and preservation checks

- [ ] Verify final Online engines, busy unmount/Repair, clean-profile setup,
  upgrade/removal preservation, panel/settings integration and translations.
  Reuse installed onedriver mount/list/unmount, panel embedding, recovery wording,
  legacy notice and disposable connection-removal evidence where applicable.

#### 7. Reconcile engineering and submission records

- [ ] Reconcile host-command inventory/permission rationale, close transferred
  Tasks (5)/(6) sandbox gates and prepare concise COSMIC review description.
  Confirm maintainer expectations for retained host execution/dependencies.
- [o] Release promotion tooling implementation/focused checks are handled by
  [Release build workflow follow-up](#release-build-workflow-follow-up--october-13-2026).
  It must publish the exact accepted immutable-source artifact set without
  rebuilding; ordinary tag builds must never publish automatically. Hosted
  promotion acceptance remains open in that authoritative task.

#### 8. Final release verification and approval

- [ ] Complete required release checks, reconcile final 0.5.4 notes/checksums and
  inspect the accepted native/final Flatpak artifacts against frozen identities.
  Keep earlier development smoke separate from final artifact acceptance.
- [ ] Obtain release publication/COSMIC packaging submission approval for the
  concrete verified artifacts and review description. Publication/submission and
  any package installation remain separately authorized actions.

## Agent context reduction — October 15, 2026

Plan, decisions and section order:
[requirements condensation plan](docs/engineering/requirements-specification-condensing-design.md).

- [x] Archive completed history through `0.5.4-pre`, condense the carried-forward
  items into `Task List Carry Forward.md`, and condense `Applet Description.md`.
- [x] Condense `Requirements and Specifications.md` section by section in the
  planned order, keeping every requirement ID and every safety-critical "shall";
  commit after each step.
  - [x] Step 1: preamble, sections 1-5, 15, 16 (130 KB to 124 KB; 199 IDs unchanged; section 16 reduced to application identity).
  - [x] Step 2: sections 12, 10, 11 (124.1 KB to 123.5 KB; IDs unchanged; contracts already dense, little to cut).
  - [x] Step 3: sections 7, 8 (already concise; two edits only).
  - [x] Step 4: section 13 with 6.1 and 6.2 (123.6 KB to 120.0 KB; layout-only 13, tooltip rules merged into FR-011H).
  - [x] Step 5 (high effort): sections 6.3-6.5 and 6.7 (already compact; FR-023 and FR-044D corrected, no size change).
  - [x] Step 6: section 14 (120.2 KB to 119.6 KB; duplicate UI checks merged, status pointer removed; section 15 kept as is).
  - [x] Step 7 (high effort): sections 17 and 18, folding the unnumbered blocks into 6.x (119.6 KB to 118.2 KB; new IDs FR-008A and FR-037B; real organization URL replaced).
  - [x] Step 8 (high effort): sections 9.4 and 9.5 (118.2 KB to 117.5 KB; command-runner rules unified in 9.3, OpenConnect text points to FR-084 to FR-084D).
  - [x] Step 9: stale-tool grep, ID-list diff, FR-086A/B and FR-087A tags, 6.11 heading, FR-097 date and SharePoint preload config entry fixed (117.5 KB final; 201 IDs, FR-008A and FR-037B added). The line 447 item below stays open: its audit starts after the Flatpak migration.

# Cloud Mounter Applet Future Task List

Items that will be activated at some time in the future.

## Post-0.5.4: legacy abraunegg engine removal

Planned October 14, 2026; separated from 0.5.4 release gates October 18, 2026.
Target: release after the 0.5.4 deprecation release, tentatively 0.5.5. This
future section has its own numbering. Existing abraunegg connections remain
supported in 0.5.4; this work is not a condition of publishing 0.5.4.
OneDrive Online using onedriver remains supported throughout.

#### 1. Legacy engine removal (the release after the deprecation release)

Checklist refreshed October 18 against current source. Release boundary above
remains authoritative until an explicit policy change; preparation may proceed
separately without removing legacy support from the 0.5.4 source/package.

- [ ] Retain legacy profile deserialization/identity and replacement guidance, but
  block retired-engine authentication, Test/Preview/Sync/Start and scheduling with
  typed unsupported-engine results. Never reinterpret absent-engine legacy
  profiles as rclone. Preserve local mirror/recovery files, databases and
  credentials; verify existing profiles remain visible and removable.
- [ ] Define and test upgrade handling for old app-owned monitor units so an
  already installed/enabled legacy service cannot continue launching the retired
  engine. Use typed systemd control and ownership checks; stop/disable only
  proven app-owned units. Preserve unrelated user services and all file data.
- [ ] Remove only legacy runtime/authentication code: one_drive_* plans/requests
  and legacy-only status/parser helpers in src/sync.rs; legacy setup/auth-files,
  pending-count, monitor and dispatch branches in src/app.rs; OneDriveSync
  diagnostics/controller requirements and Executable::OneDrive process route.
  Preserve rclone OneDrive backend/identity/guards, shared Microsoft OAuth and
  onedriver Online. Replace removed behavior with explicit unsupported state.
- [o] WebKit helper, helper-only dependency recommendations/install options and
  unused localization keys moved into 0.5.4 step-2 cleanup by operator October 18.
  Browser/manual legacy reauthentication retained; this does not retire engine
  support or delete existing credentials. See consolidated acceptance above.
- [ ] Remove remaining onedrive engine recommendation and --onedrive installer
  option after checking consumers. Retain Python for maintained build/developer
  tools; do not uninstall packages from user machines. Preserve --onedriver,
  --rclone and rclone's onedrive backend.
- [ ] Retire current legacy-executable entries in host-command inventory and
  permission rationale; preserve historical removed-command evidence and review
  consumers before dropping shared permissions. Update script catalog/inventory,
  AppStream/README/dependency guides and removal-release notes without rewriting
  published 0.5.4 deprecation history. Keep prototype evidence explicitly labeled.
- [ ] Focused synthetic/fake tests: legacy profile upgrade and inert controls,
  no retired executable/dependency/helper dispatch, app-owned unit retirement
  with rollback/partial failure, data preservation and rclone/onedriver parity.
  Run check and focused tests after Rust edits, final gate once complete, plus
  localization-check and metadata-check when their inputs change. Separately
  accept installed upgrade with an owned disposable legacy profile/unit before
  claiming native/Debian/Flatpak removal-release support.

## Post-0.5.4 validation and expansion backlog

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
