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
  keep running but cannot be created for one release; the engine is removed in
  the following release. No conversion tool: the applet has no established user
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
  rclone >= 1.74.3 and legacy dependencies until phase 6; no new startup probes.
  October 16: check, focused diagnostic/installer tests, localization and full gate
  passed. Installed native/Flatpak acceptance remains below.
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
  Updated installation and Save retest remain outstanding.
  Paused Save and mirror/service acceptance remain pending.

#### 5. Legacy abraunegg connections (no conversion tool)

- [ ] Stop offering abraunegg for new OneDrive Offline connections; existing ones
  keep working unchanged for one release. Show a deprecation notice on legacy
  connections that explains how to create the rclone mirror and delete the old one.
- [ ] Guard against two engines on one tree: block creating an rclone OneDrive
  mirror whose local path overlaps an existing legacy abraunegg mirror, and never
  reuse the abraunegg database, cache or token.
- [ ] Document the manual replacement steps in the release notes.

#### 6. Removal (the release after the deprecation release)

- [ ] Remove abraunegg code (`one_drive_*` in `src/sync.rs`, `OneDriveSync`
  dependency kind, auth-files UI in `src/app.rs`), the
  `scripts/cosmic-ext-applet-mounter-onedrive-auth-helper` and its install/desktop
  entries, and the Fluent keys for the helper.
- [ ] Packaging: drop `onedrive`, `python3-gi` and `gir1.2-webkit2-4.1` from
  `debian/control`; remove OneDrive options from `scripts/install-dependencies.sh`
  and its tests; remove the `onedrive` rows from the Flatpak host-command
  inventory and permission rationale; update AppStream/README text.
- [ ] Update the script catalog and inventory; run `bash scripts/dev.sh gate`.

#### 7. Acceptance

- [ ] Implemented guards, per drive type: wrong account or drive (identity guard
  blocks), empty or missing local and remote folder (access marker blocks), and a
  changed target invalidating Preview and initial-sync state.
- [ ] Native/Debian: personal and business OneDrive mirror lifecycle (setup, sync,
  Pause/Resume, Repair, sleep/wake, VPN), plus onedriver Online unaffected.
- [ ] Flatpak: the same lifecycle in the rebuilt Flatpak (feeds the Consolidated
  Flatpak acceptance items below).
- [ ] Release notes: engine removal and the manual steps to recreate a OneDrive
  Offline mirror as an rclone connection.

### Consolidated Flatpak acceptance and release

- [o] Adjust release promotion tooling to publish the exact verified artifact
  set from one successful immutable-source build without rebuilding it. Retain
  explicit approval, source/run/checksum validation and native/Flatpak gates;
  ordinary tag builds must still never publish automatically.
  Source implementation and focused checks handled by
  [Release build workflow follow-up](#release-build-workflow-follow-up--october-13-2026);
  next hosted promotion acceptance remains explicitly open there.
- [x] Commit accepted implementation and check Cargo sources; manifest pins
  cf5da32f622ac7b03efb42facba9e9102da365ca for the development candidate. October 16:
  all 620 registry archives/checksums and 12 dependency Git pins/selectors checked;
  generated inline/shell source support fixed, focused tests and full gate passed.
  COSMIC BaseApp/Freedesktop 25.08 unchanged. Packaging committed as de8d779;
  builder used validated local Git transport for the exact pinned app commit.
- [ ] Build source-based candidate/final Flatpak with shipping features only (candidate built October 16);
  verify source/runtime identity, helper/worker payloads and size. Inspect effective
  grants including BaseApp/global overrides; no silent blanket host filesystem/
  bus grants or restoration of obsolete shell wrappers.
  Candidate export/bundle ref, commit and complete metadata agree; bundle is
  13,511,896 bytes. Payload review and installed effective-grant acceptance remain
  open; build receipt: target/task-runs/flatpak-candidate-de8d779/artifacts.json.
- [ ] Prove selected engine auth and sandbox mirror/recovery/state access through
  reviewed grants/portal authorization. Host-path resolution is not an access
  grant. Verify Platform capabilities, not merely SDK imports or host packages.
- [ ] Complete Task (5) final sandbox Preview/initial/ordinary/scheduled sync,
  worker/progress, pause/resume, cancellation, failure/Repair, lock/initialization
  preservation and successful-only retention; expand provider matrix as applicable.
- [ ] Complete Task (6) final OpenConnect positive VPN/mount/unmount/ownership and
  one negative authentication lifecycle. Keep prior native/prototype receipts
  separate. Decide Cisco rollback disposition only after relevant acceptance;
  no OS client removal is implicit.
- [ ] Verify final Online engines, busy unmount/Repair, clean-profile setup,
  upgrade/removal preservation, panel/settings integration and translations.
- [ ] Reconcile host-command inventory/permission rationale, close transferred
  Tasks (5)/(6) sandbox gates and prepare concise COSMIC review description.
  Confirm maintainer expectations for retained host execution/dependencies.
- [ ] Finalize 0.5.4 metadata/notes/checksums, inspect native/final Flatpak artifacts
  and obtain release publication/COSMIC packaging submission approval.

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
