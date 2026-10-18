# 0.5.4 Release Acceptance Map

Type: Audit

Status: Evidence reuse plan reconciled; post-helper candidate source 7ccc9f7 pinned, replacement builds/final acceptance open

Created: October 18, 2026

Last reviewed: October 18, 2026

Scope: Step 1 of [Consolidated Flatpak acceptance and release](../../Task%20List.md#consolidated-flatpak-acceptance-and-release).
This map links retained evidence to missing work. It does not authorize builds,
installation, service changes, tags or publication. The task list owns status;
[completion notes](../../Task%20List%20Completion%20Notes.md) retain chronology.

## Evidence baseline

- Installed development Flatpak: packaging `82c1d7a`, app source `67b3aec`,
  x86_64/master, installed OSTree
  `942e1805dd8e02156290d2b05bed0d47c5fda61053eb6ddd8099b0d731a6b1b3`.
  Build/install receipts: `target/task-runs/flatpak-candidate-82c1d7a/artifacts.json`
  and `target/task-runs/flatpak-install-82c1d7a/installation.json` at repository root.
- Earlier phase-4 installed evidence uses the source/package identities recorded
  with each observation; metadata/runtimes/grants remained unchanged in the
  replacements. Reuse applies only where the relevant behavior is unchanged.
- Source/fake coverage: retained `dev-gate-s4od_kr5` and guard evidence
  `dev-gate-6_eqf2pd`; source fixes subsequently committed. Passing source tests
  do not establish installed/native/package acceptance.
- Real engine prototypes, native installed checks, development Flatpak checks
  and final artifact checks are separate evidence levels. Unrun live rows
  below remain unverified at that level even when shared policy tests passed;
  evidence review determines whether extra live execution is a release blocker.

## Evidence reuse decision — October 18

Operator directed reuse of completed Flatpak tests and removal of redundant work.
The table is an evidence inventory, not a requirement to execute every row live.
Reuse accepted phase-4/6 setup/auth, sync/schedule/Pause/recovery, guards/Repair,
portal/restart, idle wake, manual VPN, panel and connection-removal evidence where
relevant inputs are unchanged. Shared policy coverage remains source/fake evidence.

Required remaining work is final package/version/permission/dependency review,
a focused installed native/Debian smoke of changed OneDrive behavior, disposable
legacy monitor/transfer continuity, and release identity/checks. Review earlier
provider, worker/retention, Online/UI and VPN/sleep evidence first; request a new
live test only for an identified release blocker or changed behavior. No full
provider/account/scenario cross-product, new architecture matrix, repeated OAuth
or additional laptop suspend is mandated by this plan. Unrun cases below remain
explicit limitations, never installed passes. Real defects remain release blockers.

## Coverage and remaining work

“Accepted” below means accepted at the stated development-evidence level, not
final release sign-off. Step numbers refer to the consolidated task sequence. Described live procedures
are conditional options for unresolved gaps, not a newly required full matrix.

| Area / mode | Retained evidence | Missing work and destination |
|---|---|---|
| OneDrive personal / installed Flatpak | Setup, Preview/initial/manual sync with checksum, restart, scheduled edit, Pause, recovery creation; phase 4 | Reuse unchanged behavior; close remaining worker/progress/cancellation/retention observations and final artifact differences in step 4. Exact previous recovery bytes were not captured. |
| OneDrive work/school / installed Flatpak | Setup, selected folder, Test/Save, Preview/initial transfer/checksum; phase 4 | Reuse shared personal scheduling/fault policy evidence at its level; map any remaining account-specific lifecycle gap in step 4. Removed disposable profile must be recreated if new live work needs it. |
| SharePoint mirror / installed Flatpak | Existing auth/library/subtree verification, portal folders, initial and ordinary sync after applet restart; phase 4 | Reuse shared coordinator tests; identify missing final Task (5) scenarios in step 4. Missing transfer counts are not zero transfers. |
| OneDrive guards / installed Flatpak plus fake adapters | Drive-type/account refusals, local/remote marker loss, saved retarget and remote-folder loss/refusal/restoration; phase 6 | Reuse accepted cases. Malformed/resource/corporate variants retain source/fake status; no blanket per-account live claim. |
| Repair / personal installed Flatpak | Failed state visible; Repair preserved all 17 baseline files; units disabled; subsequent normal checksum sync passed | Reuse phase-6 result for unchanged code. Legacy Repair or other missing engine/mode coverage remains step 4. |
| Idle sleep/wake / personal installed Flatpak | Same-boot return, 17 unchanged files before sync, post-wake checksum sync, schedule paused | Active-transfer sleep case remains step 5; do not repeat idle smoke solely for a newer receipt. |
| Manual OpenConnect / personal installed Flatpak | Save-time cancellation/no persistence, positive auth/Save, bound checksum sync, retain/release option, original settings restored | Mount/unmount VPN integration and other Task (6) gaps remain step 5. |
| Scheduled VPN unavailable / installed service | Guard source/fake policy evidence only; manual auth cancellation is separate | With disposable initialized mirror and unavailable assigned VPN, prove scheduled invocation performs no sync and no interactive auth; preserve files/init, restore and show normal operation; step 5. |
| Pre-existing VPN / installed ownership | Source ownership policy tests passed; applet-owned release tested live | Activate tunnel independently, run disposable operation with release option, prove tunnel remains connected; operator restores state; step 5. |
| Shared VPN / installed ownership | Reference-count/shutdown fake tests passed | Use two approved disposable applet consumers; after first stops, shared tunnel remains; release only after last consumer and only if applet-owned; step 5. |
| Active-transfer sleep / installed lifecycle | Coordinator cancellation and sleep token tests passed; idle live wake accepted | One bounded disposable transfer with checkpoint/cancellation/recovery observations. Laptop display-driver failure or hard reboot is inconclusive for wake; use a suitable test host if needed; step 5. |
| Active-transfer VPN loss / installed lifecycle | Cancellation/readiness/failure policy tests passed | Disposable transfer; observe failure/cancellation and preserved state, restore VPN and explicit recovery without unsafe resync; step 5. |
| Native/Debian OneDrive migration | Earlier engine/prototype/fake evidence retained; current integrated native lifecycle not accepted | Focused installed personal/business OneDrive smoke and onedriver continuity; reuse shared policy/Flatpak lifecycle evidence, without competing runtime owners or a duplicate full suite. |
| Legacy abraunegg mirror / installed continuity | Rendered notice, authenticated dry-run Preview, retained legacy route/unit declaration | Actual monitor/transfer on separate approved disposable legacy mirror, including stop/preservation; step 4. Real pending user changes remain untouched. |
| Online onedriver / installed Flatpak | Test/setup, mount/list/unmount; phase 6 | Missing final busy unmount/Repair, clean-profile and sleep/desktop scenarios; step 6. |
| Other rclone providers / final Flatpak | Shared coordinator/policy tests and earlier engine evidence exist; per-provider release audit not complete | Review retained Google Drive/Box/SMB/SFTP evidence, select missing engine/auth-specific smoke for claimed support; steps 3-6. No full repeated provider/scenario cross-product is assumed. |
| Portal/local access / installed Flatpak | Separate mirror/recovery Browse, retained cross-launch access, host anchor/state fix accepted | Final payload/Platform capability and effective-grant audit, including overrides; steps 2-3. Host path alone is not a grant. |
| Desktop/preservation / installed Flatpak | Panel embedding, popup, recovery text, legacy notice, disposable connection removal with local files retained | Final settings/localization, clean-profile, package upgrade/removal and other Online coverage; step 6. |
| Artifact/release identity | Development export/bundle/install ref/commit/metadata match | Final frozen-version artifact matrix, payloads, effective grants, checksums and release verification; steps 1-2 and 8. |

## Release inputs to freeze

1. **Scope:** 0.5.4 uses rclone for new OneDrive mirrors, retains/deprecates existing
   abraunegg mirrors and preserves onedriver Online. Legacy removal and GUI button
   cleanup are later work. The draft notes already describe this boundary.
2. **Metadata:** Cargo manifest/root lockfile, Debian changelog and latest
   AppStream entry committed as 0.5.4 in `ef5c4d8` on October 18. Dependency
   package pins are unchanged; full gate and focused tests passed. Notes remain
   Draft. Flatpak manifest now pins that exact source; the installed candidate
   still contains older `67b3aec`, and no new package was built or installed.
3. **Targets:** existing Debian workflow declares Ubuntu 22.04/24.04/26.04 for
   amd64/arm64. Current installed Flatpak evidence is x86_64/master only. Confirm
   final advertised artifact/platform coverage before dispatch; a build matrix
   does not imply live acceptance on every target. Additional Flatpak architecture
   coverage is not established by current x86_64 evidence.
4. **Fixed package inputs:** retain reviewed runtime/BaseApp, Cargo lock/vendor
   sources, permissions and shipping features unless an explicit change is needed.
   Inspect final metadata/effective grants rather than infer them from this map.
5. **Source identity:** commit approved application metadata/source and the exact
   manifest pin/packaging inputs. Record both application and packaging revisions;
   final tag/artifact identity must satisfy maintained release scripts. The
   current development candidate is a reuse baseline, not a 0.5.4 release tag.
6. **Freeze boundary:** fixes found by missing acceptance must be resolved before
   freezing affected final inputs, or explicitly invalidate that freeze. Build
   each immutable input set once per required target; record later acceptance
   separately. No rebuild merely for documentation receipts or publication.

## Step-1 exit criteria

- Evidence rows have concrete accepted levels and a destination for missing work.
- Reuse policy reconciled: retain existing advertised provider/target scope;
  no added architecture or full live scenario matrix. Select disposable resources
  only for the focused native/legacy checks and any evidence-backed blocker.
- 0.5.4 metadata/source/manifest inputs are reviewed and pinned for required
  builds; no outstanding implementation fix is silently excluded from the freeze.
- Documentation-only mapping can finish before those preparation decisions;
  step 1 as a whole stays open until its scope and input freeze are complete.


October 18 source/pin checkpoint: application metadata/evidence committed as
`ef5c4d8f165544c6909bc43c17ed84c4ff5aae77`; packaging manifest pins that source.
This completes step-1 candidate input preparation, not final artifact acceptance.
Use the separate packaging commit containing this pin as the clean build identity.
Any later source/package fix requires a new immutable candidate and only affected
checks; documentation receipts alone do not invalidate accepted packages.


## October 18 package-review correction: remove WebKit helper now

Operator confirmed the rclone migration was chosen to avoid WebKit/Ruby build
requirements. Legacy compatibility does not require keeping the WebKit helper.
Retire helper/GUI recommendations/installer options in 0.5.4, while retaining
existing legacy synchronization and browser/manual reauthentication. Native and
Flatpak manual sign-in use the desktop OpenURI portal; no old installed helper
is discovered or launched. Python remains only where build/tooling uses it.
No host libraries, credentials or user files are automatically removed.

28b0abd/ef5c4d8 build results are pre-cleanup artifacts and remain retained
evidence, not selected post-cleanup shipping inputs. Source/pin preparation
reopens only for this change; reviewed runtime/permissions and accepted rclone
behavior remain reusable. Replacement payload audit must prove helper absence
and Debian recommendations without helper-only GUI dependencies.


Post-cleanup source checkpoint: `7ccc9f7e5a635d59dd0db9a3bb6ed3b1c85f4aef`
commits helper retirement and the Python3.10 tomli fallback/build dependency.
Combined gate `dev-gate-5jl1l4eq` passed. Manifest now pins this exact source;
replacement artifact builds/inspection are pending. No installation authorized.
