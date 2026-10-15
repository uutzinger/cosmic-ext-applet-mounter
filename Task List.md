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
  | Personal OneDrive | generic (provisional; round-trip check pending) | October 15, 2026: `target/task-runs/onedrive-guard-check-qmyfk3fx/results.json`; stable before/after web edit, 15 equal files |
  | OneDrive Business | generic (provisional; round-trip check pending) | October 15, 2026: `target/task-runs/onedrive-guard-check-rsx8nj5g/results.json`; stable before/after web edit, 15 equal files |

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
Profiles remain provisional until this follow-up passes, before phase 3. Preserve
the completed checks and their receipts.

October 15: `--round-trip` extension implemented; focused fake-rclone and local-only
rclone checks and the broader gate passed. First operator attempt failed before
cloud setup on unsupported `rclone version --json`; provenance fix and regression
verification in progress. No live round-trip verdict yet.
Implementation checks below do not imply live cloud acceptance.

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
- [ ] If generic settings cause repeated transfers or drift, test compatibility
  settings on independent fresh trees with identical samples. Identify the smallest
  justified settings and any required convergence pass. A size discrepancy or byte
  rewrite alone does not prove a workaround is needed. Ignoring size/hash for sync
  must not bypass downloaded-byte verification; explain rewrites before accepting
  a profile, otherwise leave the result unresolved.
- [x] Add fake-rclone tests for unchanged round trips, reported-size-only differences,
  byte rewrites, redundant transfers despite unchanged snapshots, missing evidence,
  transfer failures, cancellation and partial cleanup. Preserve marker-scoped
  deletion and original samples; retain partial failure evidence. Version results,
  record tool/source identity and effective settings, update help/catalog/inventory,
  then run focused tests and the routine gate.
- [ ] Run separately for personal and business with operator web-edit handoffs.
  Record dated results paths, original/downloaded comparisons, transfer counts,
  cleanup and limitations in completion notes. Finalize both profile tables from
  these results only. No whole-drive sync or installed-app acceptance is implied.

Test script(s) output

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

#### 3. Prototype on disposable accounts (no real data)

- [ ] Personal and business OneDrive via rclone browser OAuth in a native and a
  Flatpak-style run. Verify the signed-in drive differs from other accounts (see
  the earlier wrong-account incident) using the existing in-process Graph
  identity check.
- [ ] Bisync on disposable trees: first sync, edits both sides, deletes, conflicts,
  large files, network loss and pending uploads, cancellation, Repair. Measure
  runtime at a 5-minute interval on a representative tree, with and without
  full-tree checksum verification, to set its cadence.
- [ ] Confirm the minimum rclone version and flags; update the dependency checks.

#### 4. Implementation

- [ ] Routing: `RcloneBackend::try_from(Provider)` (`src/providers.rs:113`) rejects
  OneDrive for all modes. Introduce mode-aware routing so OneDrive Offline uses the
  rclone `onedrive` backend while OneDrive Online keeps `onedriver_mount_plan`.
- [ ] Remote setup and verification: reuse the `teams_oauth.rs` create/continue
  pattern for OneDrive (personal/business, rclone default client),
  verify type, authentication, drive identity and subtree access before saving;
  secrets stay in rclone config only.
- [ ] Planner/coordinator: add OneDrive bisync planning and the guards chosen above
  (`src/bisync_coordinator.rs`); exclude cloud-native documents consistently with
  the existing SharePoint rules; keep recovery, ownership, retention and
  initialization-marker behavior identical.
- [ ] Units and status: generated scheduled service/timer (`--run-bisync scheduled
  <UUID>`), readiness/VPN guards, Start/Stop, Pause, Repair, Preview/Sync Now, and
  per-connection pending counts via bisync preview (replaces the abraunegg
  `--display-sync-status` parsing; supersedes the abraunegg part of the carried
  "Offline mirror counts" item).
- [ ] Add/Modify editor: replace the abraunegg authentication controls (open URL,
  paste response, graphical helper) with rclone remote create/select for OneDrive
  Offline; Fluent keys for every new string and `just localization-check`.
- [ ] Tests: config round trip with old files, routing, plan/unit generation, fake
  runner coordinator tests, and a regression proving onedriver Online is unchanged.

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
- [ ] Commit accepted implementation and generate/check Cargo sources; pin
  immutable 0.5.4 candidate/final source. Keep COSMIC BaseApp/Freedesktop 25.08
  unless a separate reviewed runtime decision explicitly changes the target.
- [ ] Build source-based candidate/final Flatpak with shipping features only;
  verify source/runtime identity, helper/worker payloads and size. Inspect effective
  grants including BaseApp/global overrides; no silent blanket host filesystem/
  bus grants or restoration of obsolete shell wrappers.
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
- [ ] Condense `Requirements and Specifications.md` section by section in the
  planned order, keeping every requirement ID and every safety-critical "shall";
  commit after each step.
  - [x] Step 1: preamble, sections 1-5, 15, 16 (130 KB to 124 KB; 199 IDs unchanged; section 16 reduced to application identity).
  - [x] Step 2: sections 12, 10, 11 (124.1 KB to 123.5 KB; IDs unchanged; contracts already dense, little to cut).
  - [x] Step 3: sections 7, 8 (already concise; two edits only).
  - [x] Step 4: section 13 with 6.1 and 6.2 (123.6 KB to 120.0 KB; layout-only 13, tooltip rules merged into FR-011H).
  - [ ] Step 5 (high effort): sections 6.3-6.5 and 6.7.
  - [ ] Step 6: section 14.
  - [ ] Step 7 (high effort): sections 17 and 18, folding the unnumbered blocks into 6.x.
  - [ ] Step 8 (high effort): sections 9.4 and 9.5.
  - [ ] Step 9: stale-tool grep, ID-list diff, update the line 447 item.

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
