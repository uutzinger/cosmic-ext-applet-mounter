# 0.5.4 Release Acceptance Map

Type: Audit

Status: Scoped steps 2–6 accepted and step-7 internal reconciliation prepared; GitHub bundle promotion/final release acceptance open, COSMIC submission deferred to 0.5.5

Created: October 18, 2026

Last reviewed: October 20, 2026

Scope: Step 1 of [Consolidated Flatpak acceptance and release](../../Task%20List.md#consolidated-flatpak-acceptance-and-release).
This map links retained evidence to missing work. It does not authorize builds,
installation, service changes, tags or publication. The task list owns status;
[completion notes](../../Task%20List%20Completion%20Notes.md) retain chronology.

October 19 distribution decision: 0.5.4 ships through GitHub Releases, including
the x86_64 Flatpak bundle. COSMIC repository submission follows the planned
0.5.5 legacy-engine removal and GUI cleanup; earlier references to pending
COSMIC approval below are historical, not 0.5.4 blockers. Remaining work is
maintained mixed-package promotion/provenance/checksums, clean-profile bundle
installation/documentation smoke, final verification and approved publication.
Existing scoped provider/VPN/mirror acceptance is reused where unchanged.

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
a focused installed native/Debian smoke of changed OneDrive behavior and release
identity/checks. Legacy live continuity was waived by operator October 19;
retained compatibility is not claimed newly live-tested. Review earlier
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
| OneDrive personal / installed Flatpak | Setup, Preview/initial/manual sync with checksum, restart, scheduled edit, Pause, recovery creation; phase 4 | Step-4 worker/progress/cancellation/retention review reused unchanged source and accepted installed behavior; remaining limitations below. Exact previous recovery bytes were not captured. |
| OneDrive work/school / installed Flatpak | Setup, selected folder, Test/Save, Preview/initial transfer/checksum; phase 4 | Reuse shared personal scheduling/fault policy evidence at its level; map any remaining account-specific lifecycle gap in step 4. Removed disposable profile must be recreated if new live work needs it. |
| SharePoint mirror / installed Flatpak | Existing auth/library/subtree verification, portal folders, initial and ordinary sync after applet restart; phase 4 | Step-4 review reused shared coordinator tests and installed access/transfer evidence; no additional SharePoint-specific worker change identified. Missing transfer counts are not zero transfers. |
| OneDrive guards / installed Flatpak plus fake adapters | Drive-type/account refusals, local/remote marker loss, saved retarget and remote-folder loss/refusal/restoration; phase 6 | Reuse accepted cases. Malformed/resource/corporate variants retain source/fake status; no blanket per-account live claim. |
| Repair / personal installed Flatpak | Failed state visible; Repair preserved all 17 baseline files; units disabled; subsequent normal checksum sync passed | Reuse phase-6 result for unchanged code. Legacy live testing waived in step 4; no unrun engine/mode case is claimed passed. |
| Idle sleep/wake / personal installed Flatpak | Same-boot return, 17 unchanged files before sync, post-wake checksum sync, schedule paused | Reused in step 5; active-transfer sleep remains an unrun live limitation. Do not repeat idle smoke solely for a newer receipt. |
| Manual OpenConnect / personal installed Flatpak | Save-time cancellation/no persistence, positive auth/Save, bound checksum sync, retain/release option, original settings restored | October 19 source-built Flatpak Online negative/positive UI and post-unmount VPN-off check accepted separately below; reuse all mirror results. Shared/pre-existing scenarios retain their stated limitations. |
| Scheduled VPN unavailable / installed service | Guard source/fake policy evidence only; manual auth cancellation is separate | October 19 review confirms unchanged fail-closed scheduled guard and no interactive activation path. Unrun live limitation; not an automatic additional test or installed pass. |
| Pre-existing VPN / installed ownership | Source ownership policy tests passed; applet-owned release tested live | Earlier native ownership acceptance and unchanged policy tests retained. Final Flatpak pre-existing-tunnel scenario unrun live; no mandatory duplicate matrix identified. |
| Shared VPN / installed ownership | Reference-count/shutdown fake tests passed | Earlier native/shared-policy coverage retained. Final Flatpak two-consumer scenario unrun live; no mandatory duplicate matrix identified. |
| Active-transfer sleep / installed lifecycle | Coordinator cancellation and sleep token tests passed; idle live wake accepted | October 19 review found no changed sleep policy. Unrun live limitation; no additional laptop suspend required. Any future focused check needs a checkpoint and suitable host. |
| Active-transfer VPN loss / installed lifecycle | Cancellation/readiness/failure policy tests passed | Unchanged readiness/cancellation policy coverage retained. Unrun live limitation; no new fault test required absent a concrete changed behavior or blocker. |
| Native/Debian OneDrive migration | October 19 Pop!_OS 24.04 amd64 VM: personal/business Test/Save/Preview and upload/remote-edit download; personal Start/Pause; onedriver Online Test/Save/mount/list/unmount accepted by operator observation | Focused native smoke complete; reuse shared policy/Flatpak lifecycle evidence. VM checksum/binary parity not independently captured; final artifact sign-off separate. Disabled-profile Save wording accepted in replacement Debian VM October 19; legacy live testing waived October 19. |
| Legacy abraunegg mirror / installed continuity | Rendered notice, authenticated dry-run Preview, retained legacy route/unit declaration | Operator skipped browser/manual reauthentication and monitor/transfer live checks October 19; no longer a 0.5.4 gate. Retained compatibility/source evidence unchanged; unrun live cases are not passes. Real pending user changes remain untouched. |
| Online onedriver / installed Flatpak | Test/setup, mount/list/unmount; phase 6 | Step-6 review reuses native clean-profile/Online and earlier installed busy-Repair evidence plus unchanged policy tests. Latest Flatpak busy-Repair is unrun; no repeat suspend or full suite required. |
| Other rclone providers / final Flatpak | Earlier Google Drive/Box prototype mounts and native SFTP lifecycle; latest source-Flatpak SMB cycle; unchanged shared engine routing tests | Step-6 review reuses evidence at those levels, without claiming fresh final-package authentication for every provider. Existing carried-forward OAuth issues remain open; no expanded provider/scenario matrix. |
| Portal/local access / installed Flatpak | Separate mirror/recovery Browse, retained cross-launch access, host anchor/state fix accepted | Final payload/Platform capability and effective-grant audit, including overrides; steps 2-3. Host path alone is not a grant. |
| Desktop/preservation / installed Flatpak | Panel embedding, popup, recovery text, legacy notice, disposable connection removal with local files retained | Step-6 review reuses fresh VM setup, native settings/editor and notice observations, profile continuity after Debian/Flatpak upgrade, prior removal and catalog guards. No full state-hash, new locale or exhaustive accessibility pass claimed. |
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


## Step-2 artifact results — October 18

Selected replacement packaging `196f7b11cecc7c382943af23b4ebc68579b00b17` pins
app `7ccc9f7e5a635d59dd0db9a3bb6ed3b1c85f4aef`. Flatpak x86_64/master built
and exported once; five payloads/source assets/dependencies and helper absence
verified. Export/bundle commit
`80afd14af567164388f30e8b1632f18dd56c294190890da36b983a4d5ffe0bc5`. Metadata,
runtime commits and permissions match the accepted baseline; current overrides
add only read-only theme paths. Producer/preflight/payload evidence under
`target/task-runs/flatpak-candidate-196f7b1/` and `flatpak-preflight-196f7b1/`.

[Debian run 39176426152](https://github.com/uutzinger/cosmic-ext-applet-mounter/actions/runs/39176426152)
succeeded across Ubuntu22.04/24.04/26.04 amd64/arm64, including actual Python3.10
preflight coverage on both22.04 runners. Maintained producer verified exact source,
six expected artifacts, package identities and hashes; payload audit verified
five required files/source assets/ELF architecture in every package, no helper
or helper-only GUI recommendations, retained rclone/onedriver/onedrive engine
recommendations. Native payload is the Debian binary; no redundant standalone
native build. Download receipt `target/artifacts/candidates/debian-196f7b1/receipt.json`;
review `target/task-runs/debian-candidate-196f7b1/payload-review.json`.

Step 2 complete for build/offline artifact review. None of these replacements
was installed. Existing rclone behavior evidence is reused; native browser/manual
legacy reauthentication, focused native/legacy continuity and remaining release
checks retain their stated live acceptance boundaries. Earlier incomplete run
39166940625 and pre-cleanup Flatpak receipts remain historical evidence.


## Step-3 authentication and sandbox access review — October 18

Closed by scoped reuse of phase-4/6 installed evidence, with candidate inspection
from step 2. Personal and work/school OneDrive authentication, saved drive/folder
verification and transfer passed. SharePoint reused an authenticated remote;
its library/subtree verification and transfer passed, without a new consent claim.
Separate mirror/recovery Browse authorization was necessary: missing recovery
selection blocked Test, then selecting it allowed Test. Personal and SharePoint
ordinary sync after applet relaunch without Browse proved retained access.
Personal scheduled transfer and recovery-copy creation exercised worker/state
and selected recovery access. Exact prior recovery bytes were not captured.

Source comparison `67b3aec..7ccc9f7 -- src` changes only `src/app.rs`: legacy
manual sign-in labels and browser launch. Microsoft OAuth, OneDrive verification,
mirror-access and coordinator modules are unchanged. The existing Flatpak
OpenURI request is retained; native now uses it too, replacing helper/direct
launch. Native browser/manual legacy reauthentication remains step 4 live work.
No repeat OAuth or grant manipulation is required for the unchanged rclone path.

Step-2 receipts establish unchanged complete Flatpak metadata/runtime/policy,
reviewed overrides and all main ELF requirements present in Platform 25.08.
Host path resolution alone confers no grant; SDK/host packages are not used as
proof of Platform availability. This review does not install packaging 196f7b1
or turn earlier installed tests into an installed-replacement claim. Other
providers and unrun lifecycle cases retain their existing acceptance boundaries.


## Legacy live acceptance disposition — October 19

Operator declined further abraunegg/onedrive testing after native rclone mirrors
and onedriver Online smoke passed. This supersedes earlier step-4 live legacy
handoff/monitor/transfer gates in this record; mark deliberately skipped, not
passed or deferred as a new mandatory release gate. No legacy test profile was
created and no real pending legacy changes were synchronized.

The VM version screenshot shows onedrive v2.5.11-1+np1+1.1. Current dependency
installer --all includes --onedrive legacy engine installation; new Cloud Mounter
Offline connections still use rclone. This decision waives live testing only: it
does not remove compatibility code, dependencies or credentials. New native
browser/manual reauthentication remains unrun live, explicitly retained as an
acceptance limitation. Earlier dated notes retain their original scope.


## Step-4 worker and recovery evidence review — October 19

Reviewed the retained Task (5) record in Task List Carry Forward, the historical
bisync coordinator inventory, current worker/coordinator/progress code, phase-4/6
observations and passing gate dev-gate-zl7m77q1. Source diff from 7ccc9f7 shows no
change to bisync_worker, bisync_coordinator, mirror_script, mirror_access or
pending_count. Current app edit only selects a disabled-mirror Save notice.

| Area | Evidence reused | Acceptance boundary |
|---|---|---|
| Restricted worker | Retained native installed manual/initial/scheduled/Pause/Repair; Flatpak scheduled personal edit arrived remotely and Pause reported stopped; parser/launch and shared-state tests | No new native scheduled-transfer claim from VM slider alone; no full repeated suite justified by notice-only code. |
| Progress | Invocation-bound atomic progress state and stale/partial/foreign rejection tests; structured current-invocation count test passed | Fake/source coverage, not final-package rendered streaming-count acceptance. Manual streaming progress remains a separate carried-forward feature limitation. |
| Cancellation/overlap | Required-phase timeout/cancellation, lock exclusion, failed result and Repair preservation tests; installed Repair and normal sync recovery | Active-transfer sleep/VPN loss stays under step 5 review; not claimed live-passed. |
| Recovery/retention | Installed recovery-copy creation; owned dated cleanup, symlink/unowned/incomplete-listing rejection, success-only cleanup and bounded nonfatal cleanup-failure tests | Age-expiry deletion not newly live-tested; no destructive live purge required for unchanged code. Exact prior backup bytes were not captured. |

No changed engine path or unresolved defect requiring a new step-4 live test was
identified. Close the scoped evidence review, preserving these limitations and
later final-package/network/desktop gates. Legacy live tests remain deliberately
skipped per operator. The disabled-mirror notice is a source correction; candidate
196f7b1/7ccc9f7 predates it. Freeze refreshed source/manifest inputs for the next
approved affected package build and accept the changed notice there. Do not rerun
unchanged OAuth or mirror suites just to replace the artifact identity.


Step-4 closure validation: check dev-check-pmjdf7i3, two focused Save-notice tests,
localization dev-localization-check-uor7nshy and full gate dev-gate-nh0kz3wx all
exit 0 at d066a00 dirty source. Documentation links and git diff --check passed.
Step 4 closed at the stated source/reused-installed scope; the new notice has
not been built into or exercised in an installed replacement.


## Replacement package checkpoint — October 19

Source fix/records committed as 72661ce; exact Flatpak pin/packaging 7069ae5.
Both pushed after fetch confirmed no incoming changes. Existing full gate and
pin-specific four packaging tests passed; no source changes since verification.

Flatpak x86_64/master built once from clean 7069ae5 checkout and local exact-source
Git transport. Producer export/bundle inspection passed. Additional offline check
found all five payload files match export and all four assets match source; new
disabled Save notice present, helper absent. Full metadata, declared permissions,
runtime commits and ELF dependencies equal prior 196f7b1. Bundle SHA256
`a11c6d2a0451b00e27f56fdc78591223779c93acec9d280d5867107204a0c5ac`,
13,523,784 bytes; export commit
`b2a8a6e68f9f64687304a482849271ed5aab53d8039b97abb4099e62f58e6408`.
Receipts/review: target/task-runs/flatpak-candidate-7069ae5/ at repository root.

Debian run 39282971942 selects exact packaging 7069ae5. All six jobs remain
queued with no executed steps at this checkpoint. GitHub reports an active
[Actions availability incident](https://www.githubstatus.com/incidents/8tzbpcnl75jp)
with workflow-processing delays; this plausibly explains the queue, rather than
a demonstrated source/package failure. Hosted wrapper remains waiting with its
bounded timeout, logs/receipt in target/task-runs/debian-candidate-7069ae5-master/.
Recover that existing run; do not dispatch another build. New Debian packages
are not yet available or accepted. No new package was installed or published.


## Debian completion after queue delay — October 19

The original hosted wrapper completed exit 0, phase artifacts-ready: existing run
39282971942 succeeded at exact 7069ae5. All six Ubuntu 22.04/24.04/26.04 amd64/arm64
packages downloaded to target/artifacts/candidates/debian-7069ae5/; producer
verified run/source, complete artifact set, package identity and SHA256 checksums.
No redispatch or rebuild. Receipt: receipt.json in that candidate directory.

Additional offline notice audit exit 0: each package's Fluent catalog equals
clean source and contains the disabled-mirror notice; retired helper absent.
Review: target/task-runs/debian-candidate-7069ae5-master/payload-review.json.
Earlier queue checkpoint remains historical. The new Debian/Flatpak candidates
are ready for affected installed-notice checks; no installation or release claimed.


## Installed Debian notice acceptance — October 19

Operator reloaded the VM panel after the replacement-package instructions, then
provided the business Modify screenshot after Disable/Test/Save. The corrected
notice is fully rendered, including explicit Enable/Test/Save and stopped
background synchronization guidance. Enable is available; Preview/Sync Now are
unavailable for the disabled saved profile. This accepts the changed native
notice at operator-observed installed UI level; VM binary/hash parity was not
independently captured. No new Flatpak replacement installation is inferred.
Existing unchanged personal/business/Online lifecycle evidence reused. Operator
subsequently confirmed Enable, Test Connection and Save Connection restoration;
no additional sync, Start or independent timer-property check claimed.


## Step-5 scope and evidence review — October 19

Active numeric Task (6) referred to the earlier NetworkManager/OpenConnect
migration, not consolidated release step 6. Task (5) similarly meant the older
bisync coordinator migration. Replace ambiguous numbers in the current checklist
with named historical references; preserve archived/carry-forward numbering and
chronology. Source: the carried-forward October 7 migration section and the
v0.5.3 task archive's OpenConnect acceptance section.

Earlier installed native and GUI-prototype Flatpak evidence includes successful
OpenConnect authentication, Online share access, unmount/conditional release,
cancelled authentication and rejected second factor. Phase-6 source-built Flatpak
adds manual mirror VPN cancellation/no-persistence, positive auth/checksum sync,
retention without release permission and release with permission, then restored
options. Idle wake and preserved-checkpoint sync also passed. These are distinct
package/evidence levels; prototype Online success does not alone close the final
source-built Flatpak Online acceptance gate.

Diff 67b3aec..7069ae5 has no changes to vpn.rs, sleep.rs, services.rs,
mirror_access.rs or bisync_coordinator.rs. App edits concern legacy browser
handoff and disabled Save notice, not VPN/sleep control. Retain passing gate
nh0kz3wx tests for activation/shutdown ownership, shared/finite consumers,
readiness deadlines and sleep invalidation. Scheduled guard source checks
NetworkManager/profile readiness and returns an error without invoking activation.
No extra suspend or full shared/pre-existing/VPN-loss matrix is warranted by
these changes. Those unrun live cases remain explicit limitations.

The remaining focused check is one Online VPN mount/list/unmount/conditional
release and one canceled-authentication attempt on the built source Flatpak.
Prefer the laptop's existing host VPN/share setup; a fresh VM would add redundant
VPN configuration. Candidate 7069ae5 and payload/permission review are ready;
installation still requires operator approval and applet quiescence. Keep any
installed proprietary client as the manual fallback pending acceptance; no
client uninstall, VPN profile edit or service change has been performed.


## Laptop Flatpak replacement for VPN acceptance — October 19

Operator approved quiescence and installation. Restored missing temporary
packaging checkout at the recorded exact7069ae5 commit, then maintained installer
reused verified candidate and installed expected commit
`b2a8a6e68f9f64687304a482849271ed5aab53d8039b97abb4099e62f58e6408` successfully.
Receipt: target/task-runs/flatpak-install-7069ae5-retry/installation.json. Original
missing-checkout failure retained separately; no rebuild, data/service cleanup
or runtime launch. Installer acceptance remains not-run; pending panel reload
and scoped positive/negative VPN Online observations are separate evidence.


## Step-5 focused VPN completion — October 19

On installed source Flatpak 7069ae5, operator began with no VPN connected. Following
canceled-authentication instructions, operator reported VPN connection failed and
connection row off. After retry instructions to complete authentication, operator
confirmed mounted share files visible, then closed the file manager/turned row
off and reported unmount completed. Operator was unsure whether VPN disconnected;
a read-only typed NetworkManager D-Bus check returned zero active VPN connections.
The agent did not activate or deactivate any VPN. Final observation saved at
 target/task-runs/phase5-flatpak-vpn-7069ae5/final-networkmanager.json.

Accept the focused installed UI negative/positive cycle and final VPN-off outcome.
No independent negative-case FUSE/service trace, release-option dump or live
shared/pre-existing-tunnel protection claim. Earlier ownership/source evidence
and mirror release tests remain scoped separately. Idle wake is reused; no new
suspend or active-transfer fault test was performed. Step 5 is complete at this
scope. Keep any installed proprietary client as an optional manual fallback;
no OS removal is a release gate or implied. User guide clarified accordingly.
Final package/release, desktop and submission acceptance remain open.


## Step-6 focused Online/desktop/preservation review — October 19

Reviewed scope before requesting any live operation. Existing evidence covers
these unchanged paths; no additional live test was selected:

| Area | Evidence reused | Limit retained |
|---|---|---|
| Online engines | VM native onedriver setup/mount/list/unmount; latest source Flatpak SMB VPN cycle; earlier prototype Google Drive/Box mounts; native disposable SFTP lifecycle; shared routing tests | Earlier provider results keep their native/prototype identities. No fresh per-provider final-Flatpak auth claim or closure of carried-forward OAuth/quota issues. |
| Busy/Repair | v0.5.3 completion archive: deliberate busy native clean-unmount refusal, separate two-step confirmed Repair, inactive/disabled service and detached mount; current gate eligibility, ownership, failed-unmount latch and confirmation tests | Latest source Flatpak busy-Repair not repeated. Ignored live SFTP test remains ignored, not passed. |
| Clean profile/preservation | Fresh VM Add/Test/Save and mirror round trips; saved profiles retained through Debian reinstall; existing SMB profile used successfully after verified Flatpak replacement; phase-4 removal preserved local/recovery files and removed owned units | No bitwise complete config/state/cache comparison across upgrade; no user reset, repeated removal or real-data transfer needed. |
| Desktop/localization | VM settings/editor, native corrected Save notice, latest Flatpak panel/popup operation, prior recovery and legacy text; localization receipt uor7nshy (1 catalog, 1,363 messages, 1,576 references); candidate catalogs match source | English-only shipping catalog. No untranslated-language acceptance, exhaustive screenshot/keyboard/screen-reader matrix or next-release GUI cleanup implied. |

Source comparison 67b3aec..7069ae5 has no changes to controller, config,
runtime_ipc, providers, services, mount_guard, mounts, preload, model or theme.
App changes only retire legacy browser helper and choose the disabled Save
notice. Current gate nh0kz3wx covers shared Online provider ownership, failed
unmount recovery, timed Repair confirmation and managed-engine routing. Prior
installed/prototype evidence is reused; its age/package level remains explicit.

Step 6 closes as this scoped evidence review, not as fresh execution of every
historical live scenario on every package. Existing carried-forward mount/sleep,
pending-count, provider OAuth and startup observations retain their own open
status. No additional sleep, busy mount, destructive cleanup, new account setup,
code edit or package rebuild was performed. Engineering/submission and final
release verification remain open.


## Step-7 internal reconciliation and external boundary — October 19

Reconciled the host-command inventory and packaging permission rationale with
source plus existing candidate/installation receipts. Nine default executable
identities and23 finish-args confirmed; installed metadata/commit equal candidate
export, no app-specific override and only four read-only global theme paths.
Static summary: target/task-runs/step7-reconciliation/summary.json. Text search
CLI hits were reviewed as cfg-gated tests/development probes, not shipping routes.
Each retained family now has permission/path, owner and removal criteria.

Transferred bisync-worker and NetworkManager/OpenConnect sandbox gates close at
steps3–6's scoped/reused evidence levels. No fresh blanket provider/fault pass or
release-publication approval follows. No runtime code, permission or dependency
change; prior gate/localization and packaging checks reused.

Public COSMIC repository README/build recipe and PR template were inspected.
The template excludes AI-generated PR descriptions outside its own-app-source/
manifest exception and requires personal acknowledgment of contribution terms.
Therefore preparation is an [internal factual review brief](cosmic-submission-audit.md),
not a generated submission description. Operator authorship/acknowledgment and
maintainer acceptance of retained host execution, dependencies and runtime/portal
compatibility remain open. No third-party message, PR, consent or approval was
created. This is an external submission boundary; internal release verification
can be reviewed independently in step8, with publication separately authorized.


## Hosted Flatpak build closure — October 20

Both CPU bundles are preserved and offline-inspected: x86_64 run39433010848 at
095c180, ARM job run39382160325 atba8a341 (overall earlier run failed only x86).
App Git source remains72661ce; manifest/dependency fingerprints and complete
source pins match between targets. Their packaging/workflow commits differ and
remain individually recorded; final tag/promotion identity is not waived.
Checksums and producer/source/run identities passed. Maintained offline bundle
inspections matched each ref/OSTree commit and full uploaded producer metadata;
no app execution/installation. Reports under target/task-runs/hosted-flatpak-
inspect-{x86,arm}-2026-10-20-retry/. Source/fingerprint report under
target/task-runs/hosted-flatpak-acceptance-2026-10-20/verified-artifacts.json.

Compared hosted x86 metadata with accepted local7069ae5: all sections equal
except Context list ordering and Flatpak1.14.6's explicit x11 alongside fallback-
x11 (local1.18.1 omits redundant x11). Official Flatpak command documentation
states fallback-x11 takes precedence when both are supported. With that specific
semantic equivalence and unordered Context lists, permissions match; exact
per-bundle checks were not weakened or normalized in the maintained inspector.
No new host filesystem/bus grants. Installed clean-profile/ARM acceptance,
Flatpak release promotion support and final tag/publication checks remain open.


## Promotion implementation and VM smoke follow-up — October 20

Flatpak-aware publisher implementation/focused tests and gate dev-gate-85flvcye
passed; real candidate-only inspection passed for both retained hosted bundles
under target/task-runs/flatpak-promotion-candidates-2026-10-20/. Exact-tag
publication policy remains intact; these mixed packaging SHAs are not an
approved final release set. No actual publication performed.

Operator installed the retained x86 bundle on the existing Pop!_OS24.04 VM
using install-flatpak-release.sh --bundle: checksum OK, exit0. Installed commit
readback equals 2ab220a037806b795523f9b6d345f95a1b5b998578ccf15386a050883df3d7c8.
Screenshot of flatpak ps identifies the running application; operator confirms
popup, Settings and Modify Connection open. Focused VM installation/panel/UI
smoke accepted. Native profiles already exist, so clean-profile installation is
not established. Public-download installation and ARM installed acceptance
remain unverified; unchanged provider/VPN/sync evidence is reused. See completion
notes for chronological observations; task list retains remaining release gates.


## Final source identity decision — October 20

The reviewed Debian run39282971942 used7069ae5, x86 Flatpak run39433010848
used095c180 and ARM job run39382160325 usedba8a341. They retain their real
acceptance/provenance but cannot be combined as one exact-tag source set.
The pending standalone installers and Flatpak-aware publisher are absent from
those historical sources. Commit the preparation, finalize notes/remaining
acceptance, then freeze one packaging source for both package modes. Keep the
accepted app pin72661ce unchanged. Once separately authorized, build each
required target once at that common SHA; inspect resulting identities and
affected inputs while reusing unchanged behavior tests. No current candidate
is relabeled, tag chosen/created, or replacement build dispatched by this
decision. Final source freeze remains a separate outstanding checklist item.


## Acceptance reuse and release-note freeze preparation — October 20

Operator explicitly chose to reuse completed fresh-VM Debian setup and the
existing-VM Flatpak installation/panel/settings/editor checks. A separate
clean-profile Flatpak check is deliberately skipped and remains untested; it is
no longer a release blocker. No profile reset, repeat sync/VPN/sleep suite or
additional installation occurred. ARM installed behavior and public-download
checks remain explicit limitations. Release notes now cover all six Debian
targets and x86_64/aarch64 Flatpak bundles, with publication still pending.
Publication metadata/notes preflight passed. Full pre-release verify runs before
committing/fixing the common source identity; package artifact acceptance still
requires exact-source replacement build verification.

Pre-release verification completed successfully (dev-verify-gjyhru76, exit0),
alongside publication metadata/notes preflight. The "Finalize 0.5.4 release notes
and build source" commit fixes the intended common build source; later receipt
updates do not change that identity. New exact-source package builds, artifact
inspection and explicit tag/publication approval remain outstanding. No push,
tag, hosted dispatch or publication occurred during this preparation.
