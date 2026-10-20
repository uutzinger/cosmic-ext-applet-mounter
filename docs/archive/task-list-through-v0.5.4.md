# Task List — archive through v0.5.4

**Archived** after the v0.5.4 release. Frozen historical record; checkboxes here are
not maintained. Open items were carried forward to the active
[Task List](../../Task%20List.md) under "Open items carried forward"; that copy is
authoritative. Evidence: [completion notes](task-list-completion-notes-through-v0.5.4.md).

---

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

Follow the [accepted contract](../engineering/onedrive-rclone-offline-contracts.md);
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
  [0.5.4 release notes](../releases/v0.5.4.md), including unsynchronized files,
  fresh directories/auth, Preview/initial sync and verified old-profile removal.
  Final release-note reconciliation remains phase 6.

#### 6. OneDrive migration acceptance — 0.5.4

Started and closed October 18 for the documented Flatpak migration smoke scope.
Operator transferred native/Debian and actual legacy monitor/transfer acceptance
into final release acceptance; neither is claimed as passed. Earlier numbering
was phase 7. Detailed guard/failure/fix chronology and receipts remain in
[completion notes](../../Task%20List%20Completion%20Notes.md).

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

- [x] Reconcile the [0.5.4 acceptance map](../engineering/release-054-acceptance-audit.md).
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
28b0abd packages are pre-cleanup evidence. Post-cleanup 196f7b1 replacements
built/inspected successfully; reuse unchanged Flatpak behavior tests.

- [x] Build source-based Flatpak replacement with shipping features: packaging
  196f7b1 pins app 7ccc9f7, x86_64/master. Build/export and payload/runtime/policy
  inspection passed; helper absent. Receipt: target/task-runs/flatpak-candidate-196f7b1/.
- [x] Obtain and inspect native/Debian six-target matrix at 196f7b1. Run
  39176426152 passed all six targets, including both 22.04 architectures. Producer
  verified exact source, package identity and SHA256; five-file payload/source/
  ELF/dependency audit passed. Receipts: target/artifacts/candidates/debian-196f7b1/
  and target/task-runs/debian-candidate-196f7b1/payload-review.json.
- [x] Complete combined artifact check: exact identities, payload/helper absence,
  dependencies, checksums and effective grants. Flatpak metadata/runtime/policy
  unchanged; only readonly theme global overrides, no app override. WebKit/Ruby
  build pipeline/helper absent. Build-only tomli supports Python3.10; no GUI-only
  recommendation. No candidate installation or package lifecycle claim.

#### 3. Verify authentication and sandbox access

- [x] Reconcile selected rclone OneDrive personal/work-school and SharePoint
  authentication and portal mirror/recovery/state access using phase-4/6 installed
  evidence. Relevant code and candidate permissions/runtime are unchanged;
  Platform library availability passed step 2. Host-path resolution grants no
  access: separate Browse authorization and retained cross-launch/worker access
  were exercised. See acceptance map and October 18 completion notes. This reuses
  earlier installed evidence; replacement installation remains open. Changed
  native browser/manual legacy handoff belongs to step 4 continuity acceptance.

#### 4. Complete mirror and legacy continuity acceptance

- [x] Fix native VM dependency installer OBS mapping: use Pop!_OS/Ubuntu distro
  identity rather than Debian package-manager family. Six installer tests and
  dev-gate-zl7m77q1 passed; operator retry screenshot shows Done.
- [x] Focused installed native/Debian smoke on Pop!_OS 24.04 amd64 VM:
  personal/business Test/Save/Preview, initial transfer and remote-edit download;
  personal Start/Pause; onedriver Online Test/Save/mount/list/unmount passed by
  operator observation October 19. Business required explicit Enable then Save
  before Preview. Reuse shared source/engine/Flatpak lifecycle evidence; no
  duplicate full sleep/VPN/Repair suite. Exact VM package hash not independently
  captured; this is scoped lifecycle acceptance, not final artifact sign-off.
- [x] Correct disabled-mirror Save notice: require Enable, Test Connection and
  Save Connection before Preview/Sync Now; enabling/saving leaves scheduling
  stopped. Two focused tests, check, localization and gate dev-gate-nh0kz3wx
  passed October 19. Source correction; replacement-package UI check in step 8.
- [o] Legacy abraunegg browser/manual reauthentication and monitor/actual-transfer
  live acceptance skipped by operator October 19; no longer a 0.5.4 release gate.
  Retain earlier deprecation/Preview and source-test evidence at its scope;
  these unrun live cases are not passes. Legacy compatibility/dependency policy
  unchanged; no disposable legacy profile or real pending transfer started.
- [x] Reuse phase-4/6 Flatpak personal/corporate/SharePoint Preview/initial/ordinary/
  scheduled/Pause/recovery, guards and Repair evidence at its recorded scope.
  No repeated setup/auth/sync/Repair suite for unchanged behavior.
- [x] Review carried-forward bisync worker/progress/cancellation/retention evidence and
  final package differences. Unchanged worker/coordinator paths and retained
  source/installed evidence justify reuse; no additional step-4 live blocker
  identified. Limits recorded in acceptance map/completion notes; unrun cases
  remain unverified. Final package identity and later network/UI gates stay open.

#### 5. Complete VPN and sleep/wake acceptance

Historical scope: the [carried-forward NetworkManager/OpenConnect migration](../../Task%20List%20Carry%20Forward.md)
was numbered Task (6) before consolidation; it is not current release step 6.

- [x] Installed Flatpak 7069ae5 focused VPN/Online check October 19: following
  canceled-auth instructions, operator reports failure/row off; retry mounted
  share with files visible; unmount completed. Read-only NetworkManager D-Bus
  check confirms zero active VPNs afterward. Scoped UI/outcome acceptance; no
  independent negative-case mount trace or shared-tunnel policy pass inferred.
  Reuse native/prototype and phase-6 mirror VPN evidence; no duplicate matrix.
- [x] Reuse phase-6 idle wake and manual VPN cancellation/auth/sync/optional-release
  smoke with existing source/fake readiness, ownership and cancellation coverage.
  No additional laptop suspend or full VPN fault matrix required by default.
- [x] October 19 review: VPN/sleep/services/coordinator code unchanged since
  accepted phase-6 source; gate nh0kz3wx readiness/ownership/sleep tests passed.
  No additional suspend or full fault matrix justified. Scheduled VPN-unavailable,
  Flatpak shared/pre-existing ownership and active-transfer sleep/VPN-loss remain
  explicitly unrun live limitations. Focused Online package check accepted
  separately above; no source-only case is labeled an installed pass.
- [x] Rollback disposition: keep any installed proprietary Cisco client as an
  optional manual fallback; Cloud Mounter uses NetworkManager/OpenConnect only.
  Client removal is not a 0.5.4 gate and requires a separate user decision; no OS
  package, profile or service removal performed. Guidance in docs/openconnect.md.

#### 6. Complete Online, desktop and preservation checks

October 19 focused evidence review: no additional live test justified for
unchanged code. Reuse at recorded package/observation levels, not as a claim that
every scenario ran on the latest Flatpak. Historical open items remain open in
the carried-forward list; GUI button cleanup stays post-0.5.4.

- [x] Online engines: reuse VM native onedriver setup/mount/list/unmount and latest
  Flatpak SMB/VPN cycle; earlier Google Drive/Box prototype and native SFTP
  evidence plus unchanged shared engine routing. No repeated provider matrix or
  claim of fresh per-provider authentication; carried-forward OAuth issues remain.
- [x] Busy unmount/Repair: reuse earlier installed native deliberate-busy refusal
  and confirmed Repair, plus unchanged ownership/confirmation/failure tests in
  gate nh0kz3wx. Latest Flatpak busy Repair and ignored SFTP live test are not
  newly passed; no forced busy mount or real-data write needed for this review.
- [x] Clean profile and preservation: reuse fresh VM setup, saved profiles across
  Debian replacement, existing SMB profile after Flatpak upgrade, and accepted
  disposable removal with local mirror/recovery files retained. No profile reset,
  repeat removal or independent byte-for-byte upgrade-state claim.
- [x] Desktop/localization: reuse panel/popup/editor/settings observations, native
  corrected notice and prior recovery/legacy wording. English-only catalog guard
  uor7nshy passed (1,363 messages/1,576 references), packaged catalogs verified;
  no new locale or exhaustive rendered/keyboard/accessibility claim.

#### 7. Reconcile engineering and submission records

October 19 scope decision: 0.5.4 distributes Debian packages and a standalone
x86_64 Flatpak bundle through GitHub Releases. COSMIC repository submission is
deferred until the 0.5.5 legacy-engine removal and GUI cleanup; it is not a
0.5.4 release gate. Preserve completed technical preparation and acceptance.

- [x] Reconcile host-command inventory/permission rationale against source and
  installed7069ae5 receipts: nine shipping executable identities, 23 explicit
  grants, no app override, only read-only global theme overrides. Typed host
  restrictions are not an OS allowlist; owners/removal criteria documented.
- [x] Close transferred bisync worker and NetworkManager/OpenConnect sandbox
  gates using scoped steps3–6 evidence/reuse. Unrun cases remain explicit; final
  release checks and COSMIC approval are separate.
- [x] Prepare [internal COSMIC review facts/questions](../engineering/cosmic-submission-audit.md)
  and check public repository instructions/template. No tests/builds or external
  communications repeated. No app-specific permission acceptance inferred.
- [o] COSMIC submission authorship/acknowledgment and maintainer decisions moved
  to [0.5.5 Flatpak submission](#055-flatpak-pull-request-ai-consideration).
  Nothing posted or acknowledged; no COSMIC approval claimed for GitHub bundles.
- [o] Release promotion tooling implementation/focused checks are handled by
  [Release build workflow follow-up](#release-build-workflow-follow-up--october-13-2026).
  It must publish the exact accepted immutable-source artifact set without
  rebuilding; ordinary tag builds must never publish automatically. Hosted
  promotion acceptance remains open in that authoritative task.

#### 8. Final release verification and approval

- [x] Commit refreshed source 72661ce and Flatpak pin 7069ae5 for the October 19
  disabled-mirror Save notice. Flatpak x86_64/master build/export/payload checks
  passed; prior 196f7b1 artifacts retained. Flatpak replacement installed October 19;
  installer verified identity/metadata; focused VPN accepted in step 5 and
  desktop/UI evidence reviewed in step 6. Final release checks remain open.
- [x] Finish Debian run 39282971942 at exact 7069ae5: all six Ubuntu/architecture
  candidates downloaded and producer-verified; corrected catalog present in all
  packages. Original wrapper completed after queue delay, without redispatch.
- [x] Review corrected disabled-mirror Save notice in installed Debian replacement:
  October 19 VM screenshot confirms Enable/Test/Save instructions and background
  sync stopped. Reuse unchanged lifecycle evidence; no Flatpak replacement UI or
  independent VM binary-hash verification inferred.
- [x] Extend maintained release promotion to include the accepted x86_64 Flatpak
  bundle and its producer provenance alongside Debian assets and SHA256SUMS.
  Validate exact app/packaging source identities, bundle digest/ref/commit and
  acceptance; refuse mismatches and never rebuild during promotion. Include
  focused fake/offline tests and update script catalog/inventory and procedures.
  October20: explicit per-CPU selection, candidate-only inspection and strict
  exact-tag publication implemented; focused tests, real offline inspection and
  gate dev-gate-85flvcye pass. Actual publication/installed acceptance separate.
- [x] Draft README download/checksum/runtime/local-install/update instructions;
  explicitly state GitHub distribution, external host dependencies and deferred
  COSMIC submission. Download links remain prospective until publication.
- [x] Review the operator's revised README for concise wording and clear
  installation steps; preserve release scope and distinguish written instructions
  from the pending clean-profile installation check.
- [x] Add a standalone Debian release installer with automatic Ubuntu/Pop!_OS
  version and native CPU selection, verified downloads and APT installation;
  replace README Debian instructions with one curl command. Test with fake
  downloads/package-manager adapters; actual installation is separate.
- [x] Add a standalone GitHub Flatpak installer with CPU detection, verified
  bundle download and user runtime/install setup; reduce README Flatpak steps
  to one curl command. Test with fake adapters; hosted workflow comes next.
- [x] Implement manual GitHub Flatpak builds on native x86_64/aarch64 runners
  using maintained build/export checks; retain bundles, checksums and provenance
  without publishing. Local fake/policy validation first; commit/push/dispatch
  and real hosted/installed acceptance remain separate approvals.
- [x] Verify hosted Flatpak bundles/provenance for both CPUs: x86_64 run39433010848
  at095c180 and preserved ARM job run39382160325 atba8a341. Downloaded checksums,
  app/source-input equality and offline ref/commit/metadata checks passed October20.
  Retain existing local acceptance; no installed ARM or final publication claim.
  - [x] Fix first hosted x86_64 build failure: install runner elfutils for
    eu-strip/eu-elfcompress; verify the setup before retrying a new-source run.
  - [x] Preserve successful ARM run39382160325; fix x86_64 AppStream SVG-loader
    dependency and add an early compose check plus architecture-only retry.
- [x] Accept focused documented bundle installation, panel discovery and UI
  smoke on the existing VM, reusing completed provider/VPN/mirror evidence.
  October20 operator decision: reuse fresh-VM Debian setup and existing-VM
  Flatpak checks; record the separate clean-profile Flatpak case as untested.
  October20: existing Pop!_OS VM selected; prior native profiles are not a clean
  profile. Operator screenshot confirms local x86 bundle checksum OK and
  user app/x86_64/master installation exit0. Installed commit readback matches
  verified bundle 2ab220a0 exactly.
  - [x] Focused existing-VM install/panel smoke: flatpak ps identifies running
    application; operator confirms popup, Settings and Modify Connection open.
  - [o] Separate clean-profile Flatpak test deliberately omitted by operator
    decision October20; existing native profiles retained. No installed ARM or
    public-download acceptance inferred.
- [x] Complete pre-publication release checks and final artifact/source
  inspection: dev-verify-gjyhru76, finalized-note preflight and eight aa25846
  build artifacts verified. Reuse unchanged installed evidence at its recorded
  scope; no new ARM live, clean-profile or public-download pass inferred.
  Tag/promotion/public-asset verification remains separate below.
- [x] Review final tag/source identity and choose the release path: Debian7069ae5,
  Flatpak x86_64095c180 and ARMba8a341 are distinct packaging revisions. Retain
  their acceptance evidence; they cannot be promoted together under one exact
  tag. Keep the app pin72661ce; new installers/publisher require a new packaging
  source. No tag chosen or build authorized by this decision.
- [x] Finalize release notes and freeze one release source containing the new
  installers/publisher. October20: notes finalized with explicit test limits;
  dev-verify-gjyhru76 and publication preflight pass. Common source is the
  commit titled "Finalize 0.5.4 release notes and build source"; full SHA
  reported after commit. No push/tag/build/publication performed.
- [x] October20 operator authorized pushing the two prepared commits and
  starting one Debian matrix and one Flatpak CPU matrix at frozen source
  aa25846c6558a72a6fb1602aab83fc3997495e91. No tag/publication authorized.
- [x] Push frozen aa25846 and start both exact-source matrices October20:
  Debian run39450879826 and Flatpak run39450919904, both observed in progress.
- [x] Verify/download the successful exact-source builds: Debian39450879826
  (six packages) and Flatpak39450919904 (both CPUs), all at aa25846. Maintained
  producer/identity/checksum/offline bundle checks pass. All Debian bytes equal
  prior checked candidates; Flatpak pins/runtimes/options/full metadata unchanged.
  Reuse installed checks at their documented scope; ARM installed unverified.
- [x] Create/push v0.5.4 at frozen aa25846. Operator approved October20;
  maintained script succeeded, reusing Debian39450879826 and verified artifacts.
  Remote annotated tag/peeled commit confirmed; no rebuild or release published.
- [x] Exact-tag mixed promotion inspection passed October20 for six Debian
  packages, two Flatpaks, FLATPAK-PROVENANCE.json and combined SHA256SUMS.
  Receipt release-054-exact-tag-inspection/artifacts.json binds all ten assets
  to v0.5.4/aa25846; no rebuild or publication. Remote release absent.
- [x] October20 operator approved GitHub publication of the verified v0.5.4
  Debian/Flatpak assets and finalized notes. No COSMIC PR in 0.5.4.
- [x] Publish v0.5.4 through the maintained exact-tag publisher, October20.
  Ten verified assets uploaded using existing Debian39450879826 and
  Flatpak39450919904; publication receipt release-054-publication/artifacts.json.
- [x] Verify all ten public asset downloads against the publication receipt
  and combined SHA256SUMS; both README installer URLs match source and pass
  bash -n. No extra installation or rebuild.
- [o] Post-release archival proceeds under the active completion-note entry
  "Post-release closure — October 20, 2026"; preserve all open items and user
  future edits. No tag or artifact changes during record cleanup.

## Agent context reduction — October 15, 2026

Plan, decisions and section order:
[requirements condensation plan](../engineering/requirements-specification-condensing-design.md).

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
