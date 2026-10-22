# GUI action layout inventory

Type: Inventory and proposed consistency contract  
Status: Recorded GUI source changes implemented; scoped native acceptance recorded; remaining native and package acceptance open  
Created: October 21, 2026  
Last reviewed: October 22, 2026

## Scope

This record supports the 0.5.5 GUI cleanup in `Task List.md` and interprets the
operator's unchanged `Push Buttons.md`. It inventories the Settings, Add
Connection and Modify Connection actions in `src/app.rs`, including providers
that have not yet received a rendered UI check. It does not claim that
every provider/mode/window or final package is visually accepted. Current scoped
results and remaining gaps are reconciled in the October 22 acceptance matrix in
[completion notes](../../Task%20List%20Completion%20Notes.md#055-acceptance-reconciliation-october-22-2026).

Authoritative implementation inputs are `AppModel::view_general_settings`,
`view_editor_actions`, the provider setup action builders, draft invalidation
handlers and the Fluent catalog. Runtime popup actions are outside this layout
inventory except where Preview and Sync Now are launched from Modify.

## Existing Settings actions

The General Settings connection row currently shows:

| Order | Action | Current style | Current behavior |
|---|---|---|---|
| 1 | Add Connection | Suggested/blue | Opens Add Connection. |
| 2 | Remove Remote | Standard/grey | Opens the dedicated rclone remote manager. |
| 3 | Reorder Connections | Standard/grey | Opens the reorder window. |

The dedicated manager discovers every typed rclone remote, including backends
that Cloud Mounter cannot select for a connection. It lists saved connection
names for referenced remotes and disables their **In use** action. Unreferenced
remotes have a red **Remove** action followed by red confirmation. Add and Modify
retain provider-compatible remote selection without removal controls. The
deletion worker reloads app configuration and rechecks references immediately
before invoking fixed `rclone config delete` arguments. It does not delete a
saved connection, local data or cloud files. Each row uses a fill-width,
left-aligned identity/usage column and a fixed-width action column at the right,
so **In use**, **Remove**, confirmation and progress actions stay aligned.

The former manual **Refresh** action and its dedicated UI/IPC messages were
removed. Settings continues to poll runtime status and network readiness every
two seconds. The running applet observes mounts and units every five seconds,
and editor saves, removals, reordering and startup-state changes send the shared
`ConfigurationChanged` notification, which reloads saved connections and VPN
readiness without requiring a Settings button.

## Add Connection actions after layout cleanup

Online and Offline use the same first-row builder, in workflow order:

| Provider and mode | Current visible action order |
|---|---|
| OneDrive Online | Connect, Test, Save |
| OneDrive Offline | Detect, Connect, Test, Save |
| SharePoint Online or Offline | Detect, Connect, Test, Save |
| Google Online or Offline | Detect, Connect, Test, Save |
| Box Online or Offline | Detect, Connect, Test, Save |
| SMB Online or Offline | Detect, Connect, Test, Save |
| SFTP Online or Offline | Detect, Connect, Test, Save |

Connect invokes each provider's existing setup/create/update operation. OneDrive
retains Cancel during setup; SFTP retains its applying and confirmation labels.
Help describes the provider-specific operation behind the common Connect label.

Current Add styling and enablement is derived by `src/app/editor_actions.rs`.
Detect and new setup are suggested; Test accepts valid manually entered existing
remotes without requiring discovery or creation. Save requires a matching passing
Test. Pending setup, unapplied credentials/server settings and invalid fields
block dependent actions. The shared helper preserves suggested/standard class
when disabling a button, and running labels distinguish work from prerequisites.

## Modify Connection actions after layout cleanup

The October 21 source implementation uses these first-row orders:

| Provider and mode | Current visible action order |
|---|---|
| OneDrive Online | Re-Connect, Re-Test, Save, Disable/Enable, Remove |
| OneDrive Offline | Detect, Re-Connect, Re-Test, Save, Disable/Enable, Remove |
| SharePoint Online or Offline | Detect, Re-Connect, Re-Test, Save, Disable/Enable, Remove |
| Google Online or Offline | Detect, Re-Connect (disabled), Update Google OAuth, Re-Test, Save, Disable/Enable, Remove |
| Box Online or Offline | Detect, Re-Connect (disabled), Re-Test, Save, Disable/Enable, Remove |
| SMB Online or Offline | Detect, Re-Connect, Re-Test, Save, Disable/Enable, Remove |
| SFTP Online or Offline | Detect, Re-Connect, Re-Test, Save, Disable/Enable, Remove |

Every saved Offline Mirror places **Preview, Sync Now** on a second row.
OneDrive setup in progress retains its Cancel action beside Re-Connect. SFTP
retains its progress and explicit update-confirmation labels while those states
are active. Existing backend operations and confirmation safeguards are retained.

Current Modify styling and enablement uses the same policy. Setup edits recommend
Re-Connect, while persisted edits recommend Re-Test; current passing validation
allows changed settings to be saved. Unchanged settings have no save action to
perform. Google OAuth remains standard/grey, including its faded running state.
Google/Box ordinary Re-Connect remain unavailable with explanatory help.
Disable/Remove retain destructive styling, including disabled removal progress.
Preview/Sync Now retain standard styling and use existing backend eligibility;
unsaved settings and conflicting runtime requests block them. Confirmation,
cancellation, provider identity and ownership safeguards are retained.

OneDrive Offline no longer exposes a separate **Verify selected remote** action.
If its selected remote/account/folder has no current verified identity, **Test
Connection** first runs the existing non-destructive verification and then
automatically continues the full plan, dependency and access test. Other rclone
providers already verify their selected remote in Test. The all-remotes inventory
retains unsupported and incomplete remotes, while editor filtering now records
the `documentLibrary` marker explicitly so an incomplete OneDrive entry cannot
appear as a SharePoint choice.

## Original consistency findings (baseline before cleanup)

1. Action order does not match the workflow. Test and Save appear before Detect
   and Connect even when those are prerequisites.
2. Color and availability are conflated. A grey Test or Save frequently remains
   clickable, while blue can mean selected, authenticated, available, or merely
   provider-specific.
3. Successful Test state exists but is not represented by Save styling.
4. Modify has no explicit saved-draft comparison, so “connection changed” is not
   yet a defined state. Most edits only clear validation.
5. Detect is inconsistent across Modify providers even though their remote field
   accepts detected selections.
6. Provider setup vocabulary varies among Start, Connect, Create, Select and
   Update. The common workflow role is not evident from position.
7. Box lacks a Modify reconnect operation; adding a button without a backend
   contract would misrepresent capability.
8. Offline action grouping is inconsistent and can overflow. Preview and Sync
   Now belong on a second row for every supported Offline provider.
9. Remote removal is embedded in Add, filters by the selected provider and uses
   grey for the first destructive request. It therefore does not satisfy the
   Settings-level all-remote layout or the requested red removal cue.
10. Text labels provide a non-color cue and normal buttons have keyboard focus,
    but focus order, scaling and accessible names have not been rendered and
    inspected across the proposed layouts.

## Proposed action and color contract

Color communicates the recommended next step; enabled state communicates whether
an action can run. Text remains the primary non-color cue.

| Presentation | Meaning |
|---|---|
| Suggested/blue | Available and recommended as the next workflow action. |
| Standard/grey, enabled | Available secondary action; another action is normally recommended first. |
| Disabled version of the base style | Unavailable or running; retain the blue/grey/red identity with reduced contrast. |
| Destructive/red | Disables, removes or confirms removal of a saved connection/remote. |

## Shared action states and draft-change contract

Definition completed October 21, 2026 for implementation in the next step.
These rules supersede the earlier generic “connection changed” proposal; they
are the implementation contract. Source policy and synthetic transition tests
are implemented; installed rendering remains unverified.

### Action presentation and operation results

Each action has a base style (Suggested/blue, Standard/grey or Destructive/red)
and one derived interaction state, independent of its result:

| State | Enabled | Presentation and meaning |
|---|---|---|
| Recommended | Yes | Suggested/blue; a useful next workflow step. |
| Available | Yes | Standard/grey; an optional or secondary action. |
| Blocked(reason) | No | Theme-disabled version of the base style; explain the missing prerequisite. |
| Running(operation) | No | Theme-disabled version of the launch style with progress text; prevent duplicate submission. |
| Destructive | Yes | Red; Disable, Remove or their confirmations. |

October 21 operator refinement: blocked/running is an interaction state, not a
new neutral color. Follow the existing locked provider/mode controls built by
`locked_select_button`: retain the button class and remove its press action so
libcosmic renders its disabled appearance. A blue button becomes faded blue,
a standard button becomes faded grey, and a destructive button becomes faded
red. Use theme disabled background/text treatment rather than hard-coded opacity
or RGB values; the requested lighter/less-contrast appearance must be checked
in both light and dark themes.

A running action retains the style it had when launched; editing the draft
must not change its apparent role while it runs. For a blocked action, derive the
base style from the intended workflow role: Test before prerequisites and Save
before a current passing test use faded blue; an unavailable optional Re-Connect
(such as Google/Box) uses faded grey. An unchanged Modify Save has no recommended
work to perform and uses disabled standard/grey. After completion, failure or
cancellation, recalculate the role and availability from current inputs/results.
Keep progress text (for example Testing…) and blocked reasons so both states
remain distinguishable without relying on fading. Cancellation, if supported,
remains a separate enabled control.

Cancellation is a separate available action when the backend supports it.
Running destructive operations use Running, not a clickable red button.
Each operation independently records NotRun, Running, Succeeded, Failed or
Cancelled, associated with its request identity. A failure or cancellation never
counts as success. A completed action may remain available for explicit retry;
its previous success does not dictate its color forever. Several actions can
be recommended at once, such as Detect and Connect in a new editor.

Use libcosmic theme styles for suggested, standard and destructive presentation;
verify actual requested blue/grey/red appearance in the installed theme. Keep
labels, reasons and progress text as non-color cues, localized through Fluent.

### Draft comparisons

Use three explicit comparisons rather than one dirty flag:

1. **Saved settings changed:** compare the current normalized persisted
   connection with the saved connection. This determines whether Modify has
   anything to save. Add starts unsaved. An unchanged Modify editor has disabled
   grey Save with “No changes to save,” even after a successful Re-Test.
2. **Remote update pending:** compare the current provider setup inputs with
   the last successfully loaded/applied inputs for that selected remote. This
   determines whether Connect/Re-Connect or Update Google OAuth is recommended
   and whether Test would otherwise test outdated backend settings.
3. **Current test passed:** success must belong to the current editor session,
   normalized connection settings, backend revision and setup-input revision.
   This determines whether Save can use the test. Persisted changes conservatively
   invalidate the test, even if they do not require remote authorization.

Capture the saved baseline on opening Modify and refresh it only after a
successful save. Capture setup baselines only after successful remote-detail
loading or a matching successful setup. Initial loading and programmatic prefill
are not user edits. While details are loading, dependent actions are blocked.
A failed load is not proof that blank fields equal the actual remote settings.
Typing and reverting non-secret settings restores the original comparison;
it does not resurrect an invalidated test result.

Normalize with the same validated parsing used by the backend: trim identities
where the backend does, parse numeric fields and resolve paths consistently.
Do not lowercase case-sensitive remote names, usernames or paths, and do not
resolve network paths or perform blocking I/O during state calculation. Invalid
input is Blocked with its field reason, not equal to a default value.

| Input group | Remote update pending? | Test invalidated? |
|---|---|---|
| Existing remote selected / remote reference changed | No update solely for selecting an existing remote; load its details and assess capability | Yes; verify the new binding by Test |
| OneDrive drive type or expected account; SharePoint library setup input | Yes, unless existing-remote verification can establish the requested identity | Yes; preserve saved-identity/retarget safeguards |
| SMB host, user, domain; SFTP host, port, user, auth method, key file, known-hosts file | Yes when edited relative to loaded/applied setup | Yes |
| Google OAuth client pair; entered SMB/SFTP secret or passphrase | Yes; Google uses its separate OAuth update action | Yes |
| Remote subpath / folder | No remote authorization solely for a subtree change | Yes; clear/reverify provider identity proof where required |
| Local path, cache, recovery path, VPN, scheduling, metered policy, preload settings | No | Yes; full current connection must be validated |
| Display name, enabled state, start-at-login | No | Yes; conservative exact-connection validation |
| Provider, access mode, engine | Reset setup context in Add; still locked in Modify | Yes; clear provider-specific results |

Credentials are transient intent, not saved configuration values. Blank secret
fields mean “retain existing” where supported. Nonblank input marks a pending
update; track secret changes using revision counters, without logging secrets or
putting them into debug output, persisted snapshots or diagnostic hashes. Clearing
secrets after a matching successful update must acknowledge that update, not mark
the cleared form as another credential change.

### Add and Modify transitions

| Action | Add | Modify |
|---|---|---|
| Detect | Recommended when discovery can run | Recommended when discovery can run |
| Connect / Re-Connect | Recommended for new setup when required fields are valid; Available for optional setup of an existing usable remote | Recommended for pending remote updates or retry after setup failure; otherwise Available where a usable setup flow exists |
| Test / Re-Test | Recommended when a selected existing remote can be verified or Connect succeeds | Available for an unchanged saved binding; Recommended after test-relevant edits, matching setup success or test failure |
| Save | Recommended only with a current passing test and no unapplied remote edits | Recommended only with saved settings changed, a current passing test and no unapplied remote edits |
| Disable / Remove | Not present | Destructive, retaining confirmation and ownership checks |
| Enable | Not present | Recommended when available; enabling remains an unsaved change until saved |
| Update Google OAuth | Uses Connect flow | Available when its required client pair is complete; blocked during conflicts |

Test is blocked for malformed/incomplete connection inputs, conflicting setup,
or unapplied credentials/server edits. A manually entered existing remote does
not have to appear in Detect results before Test can verify it. OneDrive Test
retains its verify-then-test continuation; requiring preverified identity before
allowing Test would deadlock that workflow. A missing/unknown remote may be
verified by Test and produce a typed failure. It is never presumed connected.

After matching Connect succeeds, remote-update intent is acknowledged, the old
test is invalid, and Test is recommended. A failed/cancelled Connect retains retry
intent and leaves Save blocked. A failed Test recommends retry when prerequisites
remain valid. A successful Test makes changed/unsaved settings saveable. Re-Test
can remain Available after success; it does not need to stay blue beside Save.

Disable must remain usable when connectivity tests fail. Permit a narrowly
scoped deactivation save of an existing connection without a new Test only when
its sole persisted change is enabled=true to enabled=false, with no pending
remote update. Keep existing backend validation and stop/rollback safeguards.
If other edits are present, require them to be tested or reverted. This exception
does not permit creating or enabling an untested connection. Remove remains a
separate operation and never requires a passing Test.

### Asynchronous results and backend changes

Every setup/test request carries an editor session ID, unique operation ID and
relevant input/backend revisions. On completion accept it only if all applicable
identities still match and it has not been cancelled or superseded. Apply this
to every provider, including failures: an old failure must not clear a newer
success. Switching editors, closing the window, relevant edits, starting a new
request and cancellation invalidate the corresponding request. Retain backend
cancellation/reaping and cleanup; ignoring a late result is not cancellation.

Selecting another remote reloads setup details. Successful setup or a known
external remote/config change advances backend revision and invalidates dependent
proofs. Discovery alone is not authentication or proof that credentials are
unchanged; failures invalidate the affected capability rather than being treated
as an empty successful inventory. Existing action handlers enforce the same
policy as button enablement so keyboard/IPC paths cannot bypass prerequisites.

### Preview and Sync Now

Both are Available/grey secondary actions when the saved backend reports them
eligible. They act on saved configuration, so block them while persisted draft
changes or unapplied setup inputs would make that target differ from the editor.
Running or unknown runtime state blocks conflicting operations with a reason.
Use typed backend eligibility, initialization and preview state for each engine;
do not introduce a universal new Preview prerequisite merely to change colors.
Where that backend requires a successful initial Preview, Sync Now is disabled
until the requirement is met, then enabled grey. An initialized eligible mirror
can Sync Now without a fresh Preview. Failed/cancelled Preview never grants
permission; a changed saved target invalidates target-specific Preview evidence.

### Verification cases for implementation

Test the pure policy and each provider adapter with synthetic inputs: Add using
new and existing remotes (including a manually entered name), unchanged Modify,
setup edits versus folder/local-policy edits, edit/revert, secret clearing,
setup/test success/failure/cancellation, late/out-of-order completions and editor
switching. Cover deactivation while offline, conflicting requests, Google/Box
capability exceptions and Preview eligibility on both new and initialized saved
mirrors. Shared-state tests precede wiring and installed color/focus checks.

## Provider exceptions requiring implementation decisions

- OneDrive Offline should present one Re-Connect role even though the backend
  distinguishes creating a replacement remote and selecting an existing one.
  Re-Connect currently creates a replacement under a new remote name, using
  saved-identity checks; selecting an existing remote is verified by Re-Test.
- OneDrive Online Re-Connect maps to onedriver setup and remains identity-safe.
- SharePoint Re-Connect maps to Microsoft account/library setup.
- Google displays disabled Re-Connect with explanatory help until ordinary
  remote reauthorization has a backend. The explicit custom-client
  **Update Google OAuth** action remains executable.
- SMB and SFTP Re-Connect map to their existing update flows.
- Box requires a reviewed reauthorization/update backend before Re-Connect can
  be offered. Until then it must be disabled with explanation or the common
  label must be omitted as an explicit exception.

## Implementation and acceptance boundary

Add/Modify layout and shared editor action policy are implemented in source.
`src/app/editor_actions.rs` derives style/availability from normalized saved and
current connections, non-secret setup baselines, validation and in-flight
requests. SFTP retains its existing explicit-update safeguards. Requests carry
operation IDs and scoped setup/test revisions, and completions cannot validate
newer edits. Intermediate discovery does not release a continuing Test/Save.
The same guards serve UI actions and dispatched messages. Running removal keeps
destructive styling; neutral actions keep standard styling. Localized reasons
and progress accompany disabled controls.

Synthetic tests cover core transitions, duplicate requests, failed/cancelled and
late results, Google credential updates and saved-target runtime guards. Existing
provider/remote ownership tests remain relevant. Source checks do not establish
installed light/dark appearance, focus, keyboard or scaling acceptance; native
and Flatpak rendered checks remain open. Broader engine lifecycle evidence is
reused only at its original scope; new provider authorization/live operations
have not been exercised as part of this UI implementation.


## Google Drive creation repair and release-history audit — October 22, 2026

Installed operator testing on October 21–22 found Connect returning a success
notice without opening Google authorization. Source inspection found that Google
creation requested `--non-interactive`, which returns structured pending setup
questions, but accepted any successful process exit without inspecting the
response. The old success test supplied empty fake output, so it never exercised
this pending-response case. Detect inventories configured sections, not valid
credentials; an incomplete entry therefore also appeared in the remote list.

The repair removes question-return mode from Google creation and retains
`config_is_local true`, full-drive scope, redacted optional custom-client fields
and the five-minute process deadline. Rclone's automatic/default-answer setup
runs its browser OAuth flow. Both Create and Update now reject pending/error or
truncated configuration responses and perform a bounded read-only root listing
before reporting success. The listing uses the existing host engine transport,
20-second timeout and 16-KiB output cap; only the typed exit result is inspected.
No filenames, OAuth output or provider error payloads are included in new error
notices. A successful empty listing is valid authorization evidence, while full
connection/subtree validation remains in Test.

New-remote failure triggers cleanup only after the initial absent-name check;
failed rollback is explicitly reported with the unfinished remote name. Existing
remote creation is rejected; Update failure never deletes an existing remote.
The user's already-created incomplete remotes are preserved, so installed retry
should use a fresh unused name or operator-reviewed unused-remote removal.
Blank custom-client fields still attempt rclone's shared client; shared-client
retirement can independently require a personal OAuth client. This repair cannot
promise Google accepts the shared client.

### History evidence and limits

Method: inspect the complete Google request builder at every locally available
release tag, the creation handler at the earliest tag/current release, and Git
function history. Confirm the published 0.5.4 commit with read-only
`git ls-remote --tags origin v0.5.4 'v0.5.4^{}'`.

| Release/source | Relevant evidence |
|---|---|
| Earliest available tag `0.1`, `a9424fe`, June 20, 2026 | Google builder already uses `--non-interactive`; creation handler ignores successful command output and returns the remote name. |
| Current-master earliest implementation `cb45c00`, June 20, 2026 | Same flag and missing completion/access checks. |
| Tags `v0.2.0`, `v0.3.0`, `v0.4.0`–`v0.4.7`, `v0.5.0`–`v0.5.3` | Google creation builder retains question-return mode. No tagged automatic-browser baseline found. |
| Published `v0.5.4`, `aa25846c6558a72a6fb1602aab83fc3997495e91` | Remote tag confirmed; same creation flag and unchecked successful output. |
| Current uncommitted 0.5.5 repair | Automatic browser mode, completion/access verification, safe failure notices and regression tests. |

The faulty handling exists as far back as the available `0.1` source. This does
not establish that every earlier installed version failed to open a browser:
rclone version, returned setup stage and existing authorization can affect the
observed behavior. Historical binaries/accounts were not exercised, and the two
working Google mounts do not reveal which creation path/version originally
provisioned their remotes. There is no evidence here for attributing this to the
recent 0.5.5 button renaming/color work.

### Remaining creation acceptance

Repair evidence is compilation/synthetic tests; installed browser login is still
pending. Retest Google with an unused remote name and complete browser consent,
then Test/Save. Separately schedule disposable installed creation checks for
OneDrive Online, OneDrive rclone Offline, SharePoint, Google, Box, SMB and SFTP,
including failed/cancelled setup and existing-remote selection. Source audit
found the same non-interactive/uninspected-output pattern in Box creation; it is
outside the Google-only repair; the Box follow-up below now addresses it. Microsoft flows
already have dedicated bounded state handling; that source distinction is not
installed acceptance. Track native and Flatpak evidence separately.

References: [rclone config create](https://rclone.org/commands/rclone_config_create/)
(default-answer versus JSON-question modes),
[rclone config update](https://rclone.org/commands/rclone_config_update/), and
[rclone Google Drive](https://rclone.org/drive/#configuration) (shared-client
retirement). These document engine semantics; they do not verify applet rendering.


## Box browser authorization repair — October 22, 2026

The Box Online Mount and Offline Mirror editors share `BoxRemoteSetup` and
`create_box_rclone_remote_with`. Removed `--non-interactive` from that creation
request, retaining `config_is_local true` and the five-minute deadline so rclone
runs its automatic/default-answer browser OAuth flow.

Extracted the existing Google completion/access checks into typed shared helpers
selected by `BrowserOAuthProvider::{GoogleDrive, Box}`. Both providers reject
pending questions, structured configuration errors and truncated/invalid output;
only a successful bounded read-only `lsf <remote>: --max-depth 1` probe permits a
creation-success notice. The probe retains 20-second/16-KiB bounds. Box success
now says access was verified and directs Test of the connection settings.
Nonzero OAuth/access diagnostics expose provider/stage/exit code without raw
credential-bearing output. Timeout/cancellation remain typed failures. Existing
names (including malformed sections) block creation before mutation; only the
newly attempted initially absent remote is rolled back, and failed rollback is
reported. Existing Box remotes are never reauthorized or removed by Create.

Six focused Box tests cover automatic-browser arguments in both editor modes,
empty-root access success and probe limits, existing-name guards, pending/error/
truncated responses, access failure, redaction, cancellation/timeout and rollback
failure. Eight focused Google tests pass after shared-helper extraction. This
is compilation/synthetic evidence, not observed browser authorization. Installed
Box login, Online mount and Offline mirror setup remain in the creation matrix;
use a fresh disposable remote name and record native/Flatpak separately.

The existing disabled Box Re-Connect exception is retained: this task fixes
creation, without adding an existing-remote reauthorization backend.
References: [rclone Box configuration](https://rclone.org/box/#configuration) and
[rclone config create](https://rclone.org/commands/rclone_config_create/).


## SharePoint whole-library Offline mirror — October 22 follow-up

The operator explicitly selected whole-library mirroring after an empty optional
folder failed Re-Test with `invalid remote subpath`. The planner and mirror guard
previously required a subfolder despite the optional field and whole-remote help.
An absent subpath now means the verified library root (`remote:`). Explicit empty,
absolute, traversal and malformed folder components remain invalid in persisted
bindings; the editor normalizes its empty field to an absent subpath.

The existing serialized guard shape is retained: its string subpath is empty for
the library root. Verified site/library/drive identity and saved connection,
local directory and recovery checks remain required. Root and subfolder targets
hash to separate work directories. Changing either way clears old confirmation
markers without deleting listings, requiring a fresh Preview before initial Sync.
Remote recovery stays at `.cosmic-mounter-recovery/<connection-id>` under the
selected target, excluded by the existing SharePoint filter. Access-marker and
post-sync checksum verification behavior is retained. Scheduling stays disabled.

The folder help now explicitly describes whole-library selection. Source/fake
regressions cover root planning and recovery, serialized binding compatibility,
unsafe paths, retarget rejection before remote access, configuration validation,
confirmation invalidation in both directions and Re-Test/Save gating. Compilation
and final gate receipts are in completion notes. Installed whole-library Preview
and synchronization have not been exercised; no live library is touched here.


## Missing runtime owner and Save investigation — October 22 follow-up

Installed observation: the native panel process remained running while its
Runtime D-Bus name had no owner. Settings owned its separate Settings name and
reported `ServiceUnknown` when contacting Runtime. The running panel executable
matched the installed binary. No communication/panic journal evidence identified
the original failure. Read-only configuration inspection found no `archive`
subpath; therefore the reported edit had not reached that saved document. The
original editor Save notice was unavailable. This does not establish that loss
of runtime communication prevented persistence: Save writes before notifying
Runtime, and validation or another pre-write failure remains possible.

Source repair replaces the permanent client/error wait with D-Bus queued runtime
ownership. Waiting panels do not replace the owner and receive NameAcquired
signals when it exits. Signal listeners precede name acquisition, avoiding a
handoff race. Bus setup/disconnection errors get at most five attempts with a
two-second interval; exhaustion leaves the error visible. Settings retains its
separate singleton name. A panel acquiring ownership reloads saved configuration
before restarting shared refresh/preload setup; an ownership error clears its
owner/availability flags. A completed reload after ownership loss is ignored.
The existing enumflags2 dependency is now direct to request explicitly empty
D-Bus flags, avoiding its replacement/nonqueueing defaults.

Private-bus regression covers two queued panels, owner disconnection, ordered
takeover, preserved singleton ownership and ConfigurationChanged routing after
handoff. Synthetic model coverage checks saved connection reload and ignored
late results. The Save persistence boundary accepts injected storage and current
configuration for a temporary-file test: changing a SharePoint mirror name,
folder to `archive`, and local/recovery directories persists and reloads exactly.
Returned service-install tasks are not executed in that test. This verifies the
persistence stage, not live Microsoft Save-time validation or installed repair.

The original Save failure is not reproduced. Installed retry must retain the
Modify notice after Save and separately verify persisted changes and panel
refresh. No real saved settings or services were changed during investigation.


## Explicit shared-remote confirmation — October 22 follow-up

The operator supplied the missing sequence: first Save displayed the shared
remote confirmation, second Save displayed successful mirror-unit installation.
The operator then reported completing Preview and initial Sync for the selected
`archive` folder. This explains the unapplied first attempt as an unconfirmed
Save, rather than evidence of failed persistence. It does not establish recovery
of the separately observed missing Runtime name or whole-library acceptance.

Shared Add/Modify action policy now labels the pending second click **Confirm
Save**, with the existing blue style. The notice names Confirm Save. During
validation/persistence the existing **Saving…** label and disabled/faded styling
have priority. Consume confirmation before proceeding, so failed or cancelled
attempts require fresh confirmation. All draft-edit messages clear pending
confirmation, including policy, recovery, credentials and folder-picker changes;
editor close/reload and successful completion retain their existing reset paths.
The existing Save-time reuse warning and remote matching rules remain shared
across providers; connections without a warning retain **Save**.

Synthetic coverage spans six supported providers, both modes and Add/Modify
(24 combinations), testing label, enabled blue styling, running state,
confirmation consumption, success reset and no-warning behavior. Separate
regressions cover draft edits/cancellation and the real first-Save dispatch
returning before persistence. Installed rendering of the new label is pending.
The operator's already completed Preview/Sync is retained without repetition.


## Google slow-listing diagnosis — October 22 follow-up

Historical preliminary diagnosis: the unsupported-parameter claim below is
superseded by the real-rclone dispatch/VFS correction in the next section.

The test mount's journal contains repeated Google 403 `rateLimitExceeded` /
`RATE_LIMIT_EXCEEDED` errors referencing Requests per minute. In the inspected
recent journal, the 403/reason text appeared 51 times; these are log occurrences,
not a count of independent API requests. No 429, userRateLimitExceeded, daily
limit, storage quota or timeout match appeared in that inspected sample. Reading
only presence booleans from the test remote's config showed no custom OAuth
client ID/secret. Shared OAuth quota is a supported explanation for the observed
throttling, without attributing all quota use to this applet.

Installed rclone reports version 1.75.0. Source `src/rclone_refresh.rs` submits
`vfs/refresh recursive=true fast_list=true _async=true`. Upstream v1.75.0
`vfs/rc.go` accepts recursive and dir-prefixed keys, rejects other remaining
keys, and does not accept `fast_list`. Its documentation says recursive refresh
uses --fast-list when enabled. The mount planner has no --fast-list, and current
Google preload requests a whole-tree refresh, with 60-second policy, rather than
warming the root first. VFS readDirTree builds a complete tree before installing
it in the directory cache. The existing five-minute directory cache and full
file-content cache do not cure failed preload or API quota throttling.

This corrects the interpretation of refresh-level `fast_list=true` in frozen
0.5.3 records: those records are preserved as historical claims, not current
proof of compatibility. Their fake tests asserted command text. A repair needs
an integration test of actual RC parameter handling, root-first behavior,
bounded cancellation, and quota-conscious recursive warming. Longer directory
cache retention with change polling can then be evaluated. A private OAuth
client requires operator-controlled Google setup/reauthorization.

The test service was already inactive (MainPID zero) at this inspection. It was
not restarted, so no new live mount latency or preload trial is claimed. The
first live diagnostic stopped on missing /proc/0 after observing inactivity;
a separate bounded journal inspection succeeded. No files or settings changed.
References: [rclone Drive client ID](https://rclone.org/drive/#making-your-own-client-id),
[RC refresh](https://rclone.org/rc/#vfs-refresh-refresh-the-directory-cache),
[v1.75.0 RC source](https://github.com/rclone/rclone/blob/v1.75.0/vfs/rc.go),
[v1.75.0 directory cache source](https://github.com/rclone/rclone/blob/v1.75.0/vfs/dir.go),
[VFS directory cache](https://rclone.org/commands/rclone_mount/#vfs-directory-cache).
Implementation and installed acceptance remain open in Task List.


## Google root-first refresh and private OAuth recommendation — October 22

**Current source contract; supersedes the preceding API diagnosis.** A real
`rclone rc --loopback options/local fast_list=true` on installed 1.75.0 returned
UseListR=true; `_config={"UseListR":true}` did too. The RC dispatcher consumes
flat global options before invoking the refresh handler. Therefore the previous
claim that 1.75.0 rejects `fast_list=true` was incorrect. Absence from `vfs/rc.go`
was insufficient evidence; earlier journal inspection also found no unknown-key
error. Keep the original diagnosis for chronology with this explicit correction.

The actual VFS issue is its retained context: VFS.New copies configuration at
creation, and readDirTree uses vfs.ctx rather than the individual RC request's
context. Google mount services now receive --fast-list at creation. RC refresh
requests no longer send a misleading per-call fast_list override. Other mount
providers retain their previous arguments. Upstream 1.74.3/1.75.0 source and
current RC docs support this without raising the project's 1.74.3 minimum.

After readiness, the applet starts `recursive=false _async=true`, checks that
root job's execution ID and successful output, and only then submits
`recursive=true _async=true`. A single budget starts at root submission and is
shared through recursive completion; monitoring cannot restart the budget.
Cancellation follows the current child job, while the original root job remains
the stable UI request identity. Failed or malformed root results prevent
recursive work. Already-cached root entries remain browsable while the tree is
built. Root-first listing adds one small listing to the existing preload.

Tests cover ordered dispatch, provider selection, delayed monitoring, nested
provider errors, changed execution identity and cancellation of the recursive
job. A real local-only rclone WebDAV VFS over temporary synthetic directories
verifies the startup UseListR option, root metadata population before recursion,
and complete descendant metadata afterwards. Rclone's metadataCache/files
counter includes directories: two root entries become five total entries.
This is local engine integration, not installed Google performance acceptance.

**Operator decision:** recommend a private Google Desktop OAuth client ID and
matching secret visibly in Add and Modify, Online and Offline. The shared Google
client is being retired. Retain blank-pair compatibility; creating/changing the
operator's Cloud project, credentials and authorization requires their action.
Do not tune pacing/retries or redesign preload around shared-client quota.
Current quota observations remain evidence, not the design target. Directory
cache retention and other mount settings are unchanged in this task.

Depth limits remain applicable as a user-selected browsing scope. Google lacks
a depth field because its existing refresh API offers only recursive/nonrecursive,
not because Drive has no meaningful hierarchy. Recommend considering root-only,
shallow, or whole-tree scope using app-managed bounded traversal; an RC depth
parameter or request-local MaxDepth does not bound readDirTree's -1 traversal.
This task implements root-first/full-tree behavior and records depth as future
work, without presenting an unenforced setting.

Cancellation limit: job/stop is requested and monitoring remains bounded, but
upstream VFS methods use their retained context. A stopped RC job does not prove
all underlying VFS network work has already stopped. No mount is forcibly stopped
to cancel preload. Installed cold-cache Google timings, quota-independent private
client acceptance, and engine-level cancellation latency remain unverified.
References: [RC option overrides](https://rclone.org/rc/#setting-config-flags-with-_config),
[VFS creation](https://github.com/rclone/rclone/blob/v1.75.0/vfs/vfs.go),
[VFS directory refresh](https://github.com/rclone/rclone/blob/v1.75.0/vfs/dir.go),
[private Google client](https://rclone.org/drive/#making-your-own-client-id).


### Google Online mount options after this repair

Authoritative generator: `src/providers.rs::rclone_mount_plan`. The **only new
mount option** in this task is Google-specific `--fast-list`. Generated command:

```text
/usr/bin/rclone mount <remote>:<optional-subpath> <mountpoint>
  --vfs-cache-mode full
  --vfs-cache-max-age 168h
  --vfs-cache-max-size <connection-cache-limit; default 20G>
  --vfs-cache-poll-interval 5m
  --dir-cache-time 5m
  --timeout 10s
  --contimeout 5s
  --low-level-retries 1
  --retries 1
  --retries-sleep 5s
  --umask 002
  --log-level INFO
  --cache-dir <per-connection-cache-directory>
  --rc
  --rc-no-auth
  --rc-addr unix://<private-per-connection-socket>
  --fast-list
  --no-modtime
```

The command is shown as an argument inventory, not a shell recipe to start a
second mount. The managed service owns its lifecycle. Poll interval and VFS
write-back are not explicitly set: installed 1.75.0 defaults are one minute and
five seconds respectively. No rate-limit pacing, cache-retention, depth,
fingerprint, buffer or chunk-size option was added by this repair.
Root-first then recursive RC jobs are separate from mount flags. Credentials
remain in rclone configuration; no OAuth token or secret is put in this command.


## Mirror command failure diagnostics — October 22, 2026

Box Offline installed initial Preview failed; the operator recalls exit code 7.
The coordinator persisted failed state, which blocks Preview until Repair.
Previously `command_failure` discarded captured stderr and showed only stage
and exit code. Nonzero mirror commands now append a localized diagnostic label
and a bounded excerpt: last eight stderr lines, at most 1,600 Unicode characters.
The existing credential redactor runs before additional removal of quoted
values and path/remote-like tokens. Invalid UTF-8 is omitted. Command arguments
and stdout are never appended; no diagnostic file or extra host command is added.
External engine text is display data, not input to state or retry policy.
The original Box failure remains undiagnosed until an installed retry.

The operator also observed the main-applet toggle on after failure. Source
inspection finds `cached_offline_state` prioritizes failed coordinator state
over inactive service/timer state; the paused set then excludes that row, and
`persistent_active` becomes true. This is an indicator issue, not evidence of a
running service. Preview does not activate the timer. Correction and installed
acceptance of that independent observation remain open. Repair stops/disables
owned triggers/services, reconciles the worker and records paused state; it does
not perform an initial sync.


## Box/Google root-mirror recovery exclusion — October 22, 2026

An installed Box retry now displays the diagnostic: bisync rejects overlap
between the destination and its backup directory. The root mirror's remote
recovery path is `<remote>:.cosmic-mounter-recovery/<connection-id>/<date>`,
but the old Box filter contained only the catch-all include. Google root
mirrors have the same omission. Rclone permits nested backup directories only
when excluded from the synchronized set (https://rclone.org/bisync/). Local
recovery overlap is already rejected by plan validation.

`rclone_bisync_filter_file_for_connection` now prepends the anchored recovery
exclusion for Box/Google remote-root selections, including `.` equivalents.
Both managed-unit preparation and the shared runtime coordinator use this
helper. Existing subfolder filter bytes and other providers remain unchanged,
avoiding unrelated initialized-mirror filter-hash changes. Existing OneDrive
and SharePoint recovery exclusions stay intact. No forced resync, safety-limit
change, recovery-data migration or deletion is introduced.

Synthetic coverage checks root/normalized-root selections, exclusion ordering,
Google cloud-native rules and unchanged subfolder/other-provider filters. An
explicitly ignored real-rclone regression uses isolated temporary alias/local
remotes under Box/Google policy: old filters reproduce exit 7; corrected filters
allow initial dry-run Preview and initial sync; recovery files remain excluded,
and an overwrite preserves the original content in the remote recovery folder.
This establishes local engine integration, not native Box API or Flatpak
acceptance. The failed-Preview main-toggle issue remains separately open.


## Failed finite-operation toggle correction — October 22, 2026

Separate cached unit-derived synchronization state from coordinator failure
state in `src/app.rs`. The error/Repair state retains its existing priority,
while membership of the paused set comes directly from cached service/timer
observations. Thus an inactive service and stopped timer keep the main-applet
toggle off after failed/cancelled Preview; the row still shows Error and offers
Repair, with Preview/Resume/Sync blocked. An active timer retains the on
indication even when the last operation failed. Active services and locally
tracked operations retain the running indication and override paused display.
Missing unit observations remain unknown, not an assumed successful pause.

No new I/O, signals, polling, UI strings, engine commands or configuration
changes are introduced. Existing fake-observation tests now cover failed and
cancelled paused mirrors across OneDrive/rclone, SharePoint, Google Drive, Box,
SMB and SFTP, plus active schedules/services, tracked operations and incomplete
observations. Installed native rendering and Flatpak acceptance remain separate.


## Offline synchronization interval field clarity — October 22, 2026

Retain the existing full-width numeric interval field and add the visible
Fluent label “Sync interval in minutes” above it, grouped with four pixels of
spacing in the shared Add/Modify wizard. This applies to all Offline mirror
providers. Tooltip now explains the first background run waits the interval
after background synchronization is started, later runs wait the interval
after the previous background run finishes, and manual Sync Now does not wait
for the interval. This matches timer OnActiveSec/OnUnitInactiveSec semantics.
No configuration, defaults, validation, unit generation or running service is
changed. Installed label layout/tooltip readability remain to be observed.


## Retired-profile expected unit absence — October 22, 2026

Keep batch systemd observations typed as Present, Missing or Unavailable. Only
LoadUnit's `org.freedesktop.systemd1.NoSuchUnit` proves absence; access errors,
load errors, timeouts and cancellation are not absence. A loaded unit whose
file was removed can still be active: GetUnitFileState's typed NoSuchUnitFile
means not enabled, while the observed ActiveState/SubState/Result are retained.
The existing D-Bus transport, per-unit two-second timeout, five-second batch
deadline and cancellation behavior are unchanged.

The UI accepts confirmed service absence as fresh only for unsupported retired
profiles. It removes any cached active service for that confirmed absence;
failed observations instead retain their old cache while blocking controls.
Supported service/timer absence remains a refresh failure. The absence marker
is replaced on each observation and remains subject to the same configuration,
revision, invalidation and freshness gates. No fabricated inactive UnitStatus,
global warning suppression, user-profile deletion or data migration is used.
Retired rows retain their unsupported state. A subsequently observed active
legacy service remains visible with the existing Stop/Pause policy.

Focused synthetic tests cover explicit and absent-engine compatibility profiles,
missing-after-active cache replacement, subsequent genuine read failure, live
service reappearance, all six supported rclone mirror providers, typed D-Bus
error distinctions, cancellation/deadline and empty batches. Installed retained
profile and package-specific acceptance remain separate; the operator already
removed the old profile, so no live configuration is recreated by this task.
