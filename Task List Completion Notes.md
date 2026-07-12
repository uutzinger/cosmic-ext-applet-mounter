# COSMIC Cloud Mounter Applet Task List Completion Notes

This document preserves execution history, completion evidence, manual-test
notes, command summaries, superseded gate approvals, and implementation
commentary originally mixed into `Task List.md`.

`Task List.md` now remains the complete checklist. This file keeps the
supporting notes under the same or parallel phase/gate headings so the task
list stays readable while the history remains available.

**Split completed:** June 20, 2026.

---

**Status:** Current Gate 1 is approved. Current Phase 9 UI/runtime completion,
Current Phase 10 integrated manual testing, and Current Phase 11 documentation
and packaging recheck are complete. Historical Gate 1, Phase 10, Phase 11, and
Gate 2 passes are preserved below but superseded by later UI, provider setup,
and runtime changes. Current Gate 2 remains open pending final
release-candidate checks and user approval.
**Execution environment:** VS Code with Codex  
**Development state:** Tooltip/help, VPN selector, managed rclone runtime,
managed OneDrive runtime vertical slices, compact Add/Modify summaries,
confirmed import replacement creation, applet-owned generated-unit cleanup, and
background sync readiness/metered gating are implemented. Rclone applet-driven
setup is implemented for SMB, Box, and Google Drive; SMB setup is live-verified,
and Box plus Google Drive OAuth remote creation, Online Mount, and Offline
Mirror are live-verified. Open pre-Gate-1 item: manual keyboard/accessibility
verification in COSMIC. Later work remains for deeper Add/Modify UI cleanup and
Task List/Completion Notes separation.
This list implements `Requirements and Specifications.md`. The approved source
description is `Applet Description.md`.

## Gate 0: Review and Approval

**Gate 0 completed:** June 15, 2026. This approval permits application
development but does not authorize changing system configuration, starting
storage services, connecting VPNs, mounting storage, synchronizing data, or
accessing cloud accounts.

## Phase 1: Baseline and Scaffold

Manual COSMIC popup open/close smoke test passed on June 15, 2026. The
executable launched and the user confirmed the popup opened and closed
correctly.

## Phase 4: Systemd and Runtime Infrastructure

### Phase 4 verification

**Phase 4 completed:** June 15, 2026. Verification uses temporary unit
directories, fake command and systemd adapters, and parsed fixture data only.

## Phase 5: Online Mount Engines

### Phase 5 verification

**Phase 5 completed:** June 15, 2026. The implementation builds provider
runtime plans and decisions with fakes and typed commands only; it does not
start real mounts, synchronize data, or modify user services.

## Phase 6: Offline Mirror Engines

### Phase 6 verification

**Phase 6 completed:** June 15, 2026. The implementation builds offline mirror
plans, previews, command requests, safety gates, recovery retention decisions,
and status mappings with fakes and typed commands only; it does not start real
synchronization or modify remote/local mirror data.

## Phase 7: VPN Integration

### Phase 7 verification

**Phase 7 completed:** June 15, 2026. The implementation uses typed
NetworkManager fallback commands, Cisco component detection, readiness probes,
and dependency coordination with fakes; it does not activate or disconnect real
VPNs before the manual-test gate.

## Phase 8: Legacy Service Import

### Phase 8 verification

**Phase 8 completed:** June 15, 2026. The implementation parses copied unit
text as structured data, previews compatible rclone and `jstaf/onedriver` imports, and
plans applet-owned replacements while preserving original external units by
default. It does not execute imported text.

## Phase 9: Operation Controller and UI

### Phase 9 verification

**Phase 9 completed:** June 15, 2026. The implementation restores UI-facing
state and safe operation decisions from existing snapshots, renders popup rows,
shows legacy import previews and sanitized activity, and provides text labels
for keyboard and non-color state understanding. It does not yet execute mount,
sync, VPN, import, or destructive actions.

### Phase 9 addendum verification

**Phase 9 addendum opened:** June 16, 2026. The installed applet currently
displays popup/status information but lacks full GUI settings access and
clickable popup operation controls. Gate 2 requires recheck after this addendum
is implemented and verified.

**Phase 9 UI implementation completed:** June 16, 2026. The popup now has
Settings, Refresh, empty-state Add Connection, clickable operation controls,
Details, disabled-action explanations, and visible request notices. Settings
opens as a separate window and exposes connection management, provider/mode
configuration, Online mount, Offline mirror, VPN, dependency, legacy import, and
safety-confirmation surfaces. Controls dispatch typed non-blocking UI requests;
starting real mount/sync/import/system changes remains behind the managed
operation backend and explicit confirmation policy. Automated verification
passed with `cargo check --all-targets`, `cargo clippy --all-targets
--all-features -- -D warnings`, and `cargo test --all-targets` with 84 tests.
Manual COSMIC panel verification remains part of the current Gate 2 release
candidate recheck.

**Phase 9 manual window verification passed:** June 16, 2026. After reinstalling
the current user binary, the user confirmed that the popup, Settings window, and
add/settings windows open correctly. Keyboard-only behavior remains unverified
because the applet does not yet document expected keys beyond standard COSMIC
focus traversal.

**Keyboard verification method:** Use standard COSMIC focus traversal. `Tab`
should move forward through visible controls, `Shift+Tab` should move backward,
`Enter` or `Space` should activate the focused button, and window-manager close
shortcuts should close the Settings window without interrupting the applet.

**Keyboard/accessibility manual verification passed:** June 20, 2026. The user
confirmed that keyboard traversal and activation work in the current COSMIC UI.

**Phase 9 UI review fixes completed:** June 16, 2026. The popup header was
split into a title and control row to avoid clipping the Refresh label, Help was
added next to Refresh, status/action explanatory text now uses normal body text,
the settings subtitle was removed, Add/Import/Refresh/Help route to their
specific settings modes, and dependency/safety guidance is shown only from Help.
Verification passed with `cargo check --all-targets`, `cargo clippy
--all-targets --all-features -- -D warnings`, and `cargo test --all-targets`
with 84 tests.

### UI Design Completion

**UI Design Completion implementation update:** June 16, 2026. The popup now
uses connection rows followed by bottom Add Connection and Refresh controls,
without a separate global Settings entry point, main-window Import control, Help
control, recent-activity log, or free-form connection detail paragraph. Existing
connection rows are being refined toward a compact single-line model with a
clickable connection name and one primary state control. VPN dependencies are
summarized once in the popup header instead of repeated on every row. Static
provider, mode, local path, and remote details live in Add/Modify instead of
the main popup.
Add/Modify uses a single wizard for provider, access mode, remote/account,
subtree, local target, mode-specific settings, VPN dependency, plan testing, and
validated save. Provider and mode selections are highlighted by button style
instead of changing the label text. Import is available only inside the Add
Connection window; it scans `~/.config/systemd/user/`, shows compatible legacy
service previews, then maps a selected preview into the Add Connection wizard
for review, test, and save while preserving the original service. Remaining UI
Design Completion work is limited to the Add/Modify settings workflow redesign,
richer dependency/disk-estimate/status previews in the wizard, live managed unit
installation/removal from the UI, and final user/UI recheck.

**UI Design Completion opened:** June 16, 2026. User review found the current
Settings-centric UI confusing: the popup exposed both Settings and Add
Connection, Add/Import/Refresh displayed too much explanatory text instead of
creating or modifying connections, import previews appeared like main-window
content without becoming connection status rows, and the app lacked a clear
connection-type/VPN selection plan. The next implementation pass must follow the
single UI Design Completion checklist above.

**Superseded UI note:** Earlier checked Phase 9 items that mention a global
Settings button/window are historical implementation work, not the approved
current design. The current main popup uses Add Connection, Refresh, clickable
connection names for Modify, a compact primary state control, and a header VPN
summary. Import, Preview, Sync Now, detailed status, and detailed help belong
in the Add/Modify or diagnostics workflow.

### Main Applet UI User-Guided Cleanup

Complete this main-popup review before advancing to later recheck phases. Keep
the scope limited to the applet popup; Add/Modify connection cleanup remains a
separate later task.

### Settings (Add/Modify) UI User-Guided Cleanup

Complete this settings-window review before returning to provider/runtime
verification. Keep the scope limited to Add/Modify layout, instructions,
field-help placement, and window identity; provider behavior changes remain in
the runtime sections.



**Add/Modify layout cleanup first pass:** June 19, 2026. The settings rows now
use a custom compact row instead of `widget::settings::item`, reducing the
title/control gap and top-aligning section titles. Add-mode notice text was
updated, main controls moved left as a group, Online mount toggles now use the
COSMIC toggler, and rclone empty/setup guidance for Box and Google Drive moved
from inline body text into tooltips. User-guided visual review remains open.

**Add/Modify action-row policy update:** June 19, 2026. Add-mode rclone
connections now show Detect rclone remotes and the provider-specific Create
Remote action in the top action row. Create Box, Google Drive, and SMB Remote
actions are hidden while modifying existing connections. Test Connection and
Save Connection render as standard buttons for new rclone connections until the
selected remote appears in the detected matching rclone remote list.

**Add/Modify inline guidance cleanup:** June 19, 2026. Extra OneDrive and SMB
setup paragraphs were removed from the Connection form body and folded into the
relevant field/action tooltips. The VPN selector now presents No VPN, detected
NetworkManager profiles, Cisco Secure Client, and Detect VPNs in one compact
selector without visible NetworkManager/Cisco group headers.

**Add/Modify alignment update:** June 19, 2026. Section body containers now
explicitly align left, so Provider, Access mode, text inputs, and setup controls
share the same starting grid instead of short button rows drifting relative to
longer rows.

**Add/Modify install and identity update:** June 19, 2026. The desktop entry now
includes `StartupWMClass=io.github.uutzinger.cosmic-ext-applet-mounter` in
addition to the reverse-DNS desktop file ID and app ID. The release build was
installed with `just install-user`; user visual verification remains open.

**Window title identity follow-up:** June 19, 2026. User review showed the
COSMIC app tray still listed the toolkit default `Cosmic - Iced`. The startup
path now seeds the standalone settings title against the reserved window ID when
no main window ID is assigned yet, and the applet window title uses `COSMIC
Cloud Mounter`.

**Window identity conclusion:** June 20, 2026. User testing showed other COSMIC
applets also report `Cosmic - Iced` when run standalone from `target/release`.
The behavior is therefore treated as COSMIC/direct-binary desktop matching
behavior rather than a project defect. The temporary split settings desktop
identity workaround was removed; the applet keeps the normal app ID and desktop
metadata.

**Add/Modify OneDrive action-row refinement:** June 19, 2026. The standalone
settings window default width was increased to 880 px. OneDrive setup actions
were moved from the Connection section into the top action row, and Test
Connection/Save Connection now render as non-primary buttons until the app-owned
onedriver metadata or `abraunegg/onedrive` refresh token exists.

**Add/Modify Modify-mode action wrapping:** June 19, 2026. OneDrive Offline
Mirror Modify windows now render primary validation/setup actions on the first
action row and saved connection operations/removal actions on a second row,
preventing OneDrive mirror actions from clipping at the default settings window
width. Other connection types keep a single Modify action row.

**Tooltip content review completed:** June 20, 2026. User edits from
`Tooltip Review.md` were incorporated into `src/app.rs`: local target help now
plainly warns not to reuse mountpoints and mirror directories; rclone and
OneDrive subtree help now says to use an existing folder/subtree; OneDrive setup
actions use "Runs" wording; rclone remote help now includes provider-specific
naming examples; and OneDrive account help now includes account-label examples.
Verification passed with `cargo fmt --all -- --check` and
`cargo check --all-targets`.

**Safety tooltip emphasis added:** June 20, 2026. Tooltips related to the README
Data Integrity Warning now render an explicit bold warning line followed by
normal explanatory text. Covered safety areas are mountpoint/mirror-directory
reuse, initial synchronization preview/confirmation, Offline Mirror Sync Now,
OneDrive engine overlap, and recovery-directory placement. `Tooltip Review.md`
was updated to reflect the bold-warning structure. Verification passed with
`cargo fmt --all -- --check` and `cargo check --all-targets`.

### Help/Tooltip User-Guided Cleanup

Tooltip behavior is now tracked under Add/Modify UI implementation and
Add/Modify UI user-guided cleanup. Keep this section as a cross-reference only:
attached delayed tooltips replace visible help buttons, nested tooltip wrappers
are disallowed, and remaining inline setup guidance should move into the
relevant field/action tooltip or top notice area.


**Main popup cleanup implementation update:** June 19, 2026. Add Connection and
Refresh were moved into the fixed header below the title and aggregate status.
The connection rows now live in a dedicated scrollable region, and the popup
maximum height was reduced so it should not extend past the screen. The
connection settings launcher now falls back to the user-installed
`~/.local/bin/cosmic-ext-applet-mounter` executable when the applet session
cannot resolve the current executable path. User-guided visual verification
remains open.

**Main popup scroll correction:** June 19, 2026. User review with seven
configured connections showed the popup still extended past the screen because
the operation notice remained in the fixed header and the connection scroller was
not tightly bounded. The popup now keeps only title, three status lines, Add
Connection, and Refresh fixed at the top. Operation notices and connection rows
share a bounded scrollable region. A later refinement estimates the current
notice/message height and subtracts it from the connection-list scroll height so
long notices reduce the list area instead of increasing the overall popup size.

**Main popup compact-row update:** June 19, 2026. The popup connection row now
uses the COSMIC `widget::toggler` pattern used by the KDE Connect applet. The
row removes the separate wide operation-state chip and wide primary text button:
the connection name remains the Modify entry point, and the toggler sends
Mount/Unmount for Online Mount or Start/Stop for Offline Mirror while exposing
current status and unavailable reasons through tooltip help.

**Offline Mirror secondary action relocation:** June 19, 2026. Preview and Sync
Now are now shown in the Modify Connection action row for saved Offline Mirror
connections. They dispatch the same managed runtime operations as the former
popup buttons and retain the existing preview-before-initial-sync safety checks.

**Main popup and Add/Modify visual review completed:** June 20, 2026. User
review accepted the compact toggler-based main popup, long-name elision, fixed
header with scrollable connection list, and current Add/Modify visuals after the
OneDrive action-row wrapping refinement. Remaining `Cosmic - Iced` task-switcher
labeling is recorded as a COSMIC/direct-binary behavior, not an app workaround
target.

### Add/Modify Settings Workflow Redesign

**Opened:** June 17, 2026. The main popup has converged on a compact
fixed-width applet design. The next UI work is the larger Add/Modify window,
where there is enough room for configuration, validation, import, help, and
destructive actions.

#### Resolved Setup Decisions

**Setup investigation note:** June 17, 2026. rclone appears suitable for
applet-guided setup because `rclone config create --non-interactive` is intended
for applications and returns structured questions plus continuation state when
user input or authentication is required. Google Drive and Box setup can be
guided by the applet with browser/OAuth handoff; SMB can be created from applet
fields such as host, share/path, user, domain, port, and password. `jstaf/onedriver`
supports a GUI account setup flow and command-line mount startup, so the applet
will launch/integrate that flow where practical and still allow an already
configured account. `abraunegg/onedrive` supports OAuth/device/Intune
authentication and isolated config/sync directories are already planned, so the
applet will launch authorization directly where practical and guide an external
authorization step when tenant policy requires it. Every engine retains an
external-setup-plus-verification fallback.

#### Rclone Provider Runtime Verification

Use disposable local targets and the disposable remote subtrees created for
verification. Implement and verify one complete path before broadening to the
other providers and modes.

#### OneDrive Provider Runtime Verification

Keep OneDrive validation separate from the rclone vertical slice because
OneDrive uses two different engines: ``jstaf/onedriver`` for Online Mount and
`abraunegg/onedrive` for Offline Mirror.

#### Completed OneDrive Validation And Cleanup Follow-Ups

These tasks previously blocked applet-driven setup work and are now complete.

1. [x] Verify OneDrive Offline Mirror conflict/deletion behavior where practical
   using the disposable personal-account subtree.
2. [x] Remove the disposable OneDrive remote/local test subtree after explicit
   user approval.
3. [x] Implement OneDrive Online Mount account/setup validation messages for
   missing `jstaf/onedriver`, unauthenticated account, unavailable account,
   overlapping mountpoints, and active conflicting `jstaf/onedriver` processes.
4. [x] Implement OneDrive Offline Mirror account/setup validation messages for
   missing `abraunegg/onedrive`, unauthenticated account, tenant/admin-consent
   failure, Intune/SSO requirements, overlapping `jstaf/onedriver` usage, unsafe
   sync directories, expired/invalid OAuth responses, and resync-required states.
5. [x] Update the shared Add/Modify OneDrive validation UI so Online Mount and
   Offline Mirror validation messages appear in Test Connection, Save
   Connection, and setup guidance without storing provider credentials.

#### Provider Setup Flow Implementation

**OneDrive Offline Mirror authorization update:** June 19, 2026. The installed
`abraunegg/onedrive` help and upstream usage guide show that interactive
browser-based OAuth uses a redirect URI/local listener, while `--auth-files` and
`--auth-response` are non-interactive/manual mechanisms. The applet now makes
the upstream interactive `onedrive --reauth` local-redirect flow the primary
setup action and keeps Manual Auth Handoff as an explicit fallback. Automated
tests cover the primary interactive command shape, fallback auth-files command
shape, transient handoff cleanup, and validation after authorization.

**OneDrive WebKit auth helper prototype:** June 19, 2026. A separate
`cosmic-ext-applet-mounter-onedrive-auth-helper` GTK/WebKit helper was added and
installed with the applet. Manual Auth Handoff now opens this helper when
available; the helper attempts to capture the Microsoft native-client redirect
and write it to the `abraunegg/onedrive --auth-files` response file. If the
helper is missing or fails, the applet falls back to opening the auth URL with
`xdg-open` and keeps the response URL text field. Python syntax checks,
`cargo check`, focused OneDrive tests, and clippy passed.

**OneDrive WebKit auth helper live verification:** June 19, 2026. The helper
captured the Microsoft native-client redirect without manual URL copy/paste for
a disposable/non-critical OneDrive Offline Mirror connection. The applet reported
`abraunegg/onedrive` authentication success, validation preview completed, Test
Connection passed, and Save Connection installed the managed OneDrive mirror
unit without starting synchronization.

**SMB applet-driven rclone setup verification:** June 18, 2026. A temporary
rclone SMB remote `cosmic_mounter_smb_verify` was created from the applet-style
host/user/domain fields against `engr-drive.bluecat.arizona.edu`, then the SMB
password was entered through rclone's own password command so the applet never
stored or handled the secret. Direct rclone access to
`Research/Utzinger/cosmic-mounter-ui-test` passed, including a tiny
write/read/delete check. A disposable applet Online Mount connection using the
created remote was saved through the validated configuration path, installed an
applet-owned disabled systemd user unit, mounted successfully with rclone FUSE,
and passed write/read/delete through the mount before clean stop/detach. A
disposable applet Offline Mirror connection using the created remote was saved,
installed disabled service/timer units, passed initial `--resync --dry-run`,
confirmed initial `--resync`, and passed a normal bidirectional bisync with one
local-only and one remote-only file. During verification, SMB remote recovery
path planning was fixed so backup/recovery directories stay inside the selected
SMB share but outside the mirrored subtree, for example
`Research/Utzinger/.cosmic-mounter-recovery/<connection-id>`, instead of the
invalid remote-root path that rclone interpreted as a nonexistent SMB share.
The temporary rclone remote, test applet config entries, disposable user units,
temporary local mirror/recovery/work directories, and remote test files were
removed after verification; the original `ua_engr` remote remained accessible.

**Box applet-driven rclone setup implementation:** June 18, 2026. The
Add/Modify connection window now exposes a Box-specific `Create Box Remote`
action when the Box provider is selected. The action validates the requested
rclone remote name, rejects duplicates before OAuth starts, and runs
`rclone config create <remote> box config_is_local true --non-interactive`
with a five-minute timeout so rclone owns the local-browser OAuth flow and keeps
tokens in rclone config rather than applet configuration. On success the new
remote is selected and rclone remotes are re-detected; on failure the applet
reports the rclone setup error. Automated tests cover fixed arguments,
duplicate-name blocking, timeout, and invalid remote-name rejection. Live Box
OAuth verification passed with a disposable/non-critical Box remote name.

**Box applet-driven rclone setup verification:** June 19, 2026. A disposable
Box remote `cosmic_mounter_box_live_verify` was created through the Add/Modify
window, authenticated through rclone's browser OAuth flow, and selected by the
applet. A disposable remote subtree `cosmic-mounter-live-verify` was created
and direct rclone access verified. Test Connection, Save Connection, and Online
Mount were verified with the applet-created remote and disposable mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-box-live-verify`. The generated user
service `cosmic-mounter-2777721d-ff56-4faa-a8b1-4f587e31d0c1.service` mounted
successfully. A start-path bug was fixed by treating `systemctl --user
reset-failed` as best-effort because it exits nonzero when a disabled unit is
not loaded. A popup status bug was fixed by building the applet view state from
the runtime mount table and `systemctl --user show`, and by allowing active
mounting services to expose Unmount even when the mount table has not yet
reported Mounted. Verification included `cargo check --all-targets`,
`cargo test runtime_systemd_status_parser_recognizes_active_disabled_units
--all-targets`, `cargo test controller::tests --all-targets`, and
`just install-user`. Box Offline Mirror with an existing Box remote was already
verified; live Offline Mirror verification specifically with the newly created
Box OAuth remote was completed later on June 19, 2026.

**Box applet-created OAuth remote Offline Mirror verification:** June 19, 2026.
A disposable Box Offline Mirror connection named `Box OAuth live verify offline`
was added with ID `80ee88c8-d44b-405d-911a-427dc6cace8f`, remote
`cosmic_mounter_box_live_verify`, subtree
`cosmic-mounter-live-verify/offline-mirror-20260619-codex`, local mirror
`/home/uutzinger/Cloud/cosmic-mounter-box-oauth-offline-verify`, recovery
directory
`/home/uutzinger/Cloud/.cosmic-mounter-recovery/cosmic-mounter-box-oauth-offline-verify-80ee88c8-d44b-405d-911a-427dc6cace8f`,
and work directory
`/home/uutzinger/.local/state/cosmic-ext-applet-mounter/rclone-bisync/80ee88c8-d44b-405d-911a-427dc6cace8f`.
The generated service and timer installed disabled. Initial
`rclone bisync --resync --dry-run` reported the expected two pending transfers:
one local seed file to Box and one Box seed file to local. Confirmed initial
`--resync` completed successfully, and local/remote listings converged. A
follow-up normal sync through the generated applet-owned service copied one new
local file to Box and one new Box file back to local; final listings matched on
both sides. The applet initial-preview and initial-sync marker files were
recorded for this disposable connection, and the generated timer Start/Stop
path was verified by enabling/starting the timer, confirming it became active
and enabled, then stopping/disabling it and confirming it returned inactive and
disabled.

**Google Drive applet-driven rclone setup implementation:** June 18, 2026. The
Add/Modify connection window now exposes a Google Drive-specific
`Create Google Drive Remote` action when the Google Drive provider is selected.
The action validates the requested rclone remote name, rejects duplicates before
OAuth starts, and runs
`rclone config create <remote> drive scope drive config_is_local true --non-interactive`
with a five-minute timeout so rclone owns the local-browser OAuth flow and keeps
tokens in rclone config rather than applet configuration. The full-drive scope
matches the applet's Online Mount and Offline Mirror use cases; Google
Docs/Sheets/Slides are still skipped by Offline Mirror as browser-accessible
files. On success the new remote is selected and rclone remotes are re-detected;
on failure the applet reports the rclone setup error. Automated tests cover
fixed arguments, duplicate-name blocking, timeout, and invalid remote-name
rejection. Live Google Drive OAuth verification passed with a
disposable/non-critical Google Drive remote name.

**Google Drive applet-created OAuth remote verification:** June 19, 2026. A
fresh disposable Google Drive rclone remote `cosmic_mounter_gdrive_live_verify`
was created with the applet's fixed command shape
`rclone config create <remote> drive scope drive config_is_local true
--non-interactive`. The browser OAuth flow completed, `rclone about` succeeded,
and the remote reported a usable Google Drive quota response. Disposable remote
subtrees were created under `cosmic-mounter-gdrive-live-verify/`.

The Online Mount slice used connection ID
`18684a83-01f1-49f8-bb27-232b936c665d`, subtree
`cosmic-mounter-gdrive-live-verify/online-mount-20260619-codex`, and mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-gdrive-oauth-online-verify`. The generated
service mounted as `fuse.rclone`, exposed the remote seed file, wrote a marker
through the mounted filesystem, confirmed the marker through direct rclone
access, removed it through the mount, and stopped cleanly.

The Offline Mirror slice used connection ID
`6e6be0c8-71b7-413e-bd1b-1625406c8e82`, subtree
`cosmic-mounter-gdrive-live-verify/offline-mirror-20260619-codex`, local mirror
`/home/uutzinger/Cloud/cosmic-mounter-gdrive-oauth-offline-verify`, recovery
directory
`/home/uutzinger/Cloud/.cosmic-mounter-recovery/cosmic-mounter-gdrive-oauth-offline-verify-6e6be0c8-71b7-413e-bd1b-1625406c8e82`,
and work directory
`/home/uutzinger/.local/state/cosmic-ext-applet-mounter/rclone-bisync/6e6be0c8-71b7-413e-bd1b-1625406c8e82`.
The generated service and timer installed disabled. Initial
`rclone bisync --resync --dry-run` reported the expected local-to-Drive and
Drive-to-local transfers. Confirmed initial `--resync` completed successfully,
and local/remote listings converged. A normal post-initial sync through the
generated applet-owned service copied one new local file to Google Drive and
one new Google Drive file back to local; final listings matched on both sides.
The applet initial-preview and initial-sync marker files were recorded for this
disposable connection, and the generated timer Start/Stop path was verified by
enabling/starting the timer, confirming it became active and enabled, then
stopping/disabling it and confirming it returned inactive and disabled.

**OneDrive Online Mount applet-driven setup implementation:** June 18, 2026. The
Add/Modify connection window now exposes `Start OneDrive Setup` for OneDrive
Online Mount connections. The action pins an unsaved draft to a stable
connection ID, creates the applet-owned mountpoint, cache directory, and
config directory, runs `onedriver --auth-only --config-file <app-config>
--cache-dir <app-cache> <mountpoint>` with a five-minute timeout, then reuses
the existing onedriver validation path to confirm metadata exists and the
mountpoint does not overlap an active `fuse.onedriver` mount. Credentials
remain in onedriver-owned metadata; the applet stores only connection
configuration. Automated tests cover the fixed auth command shape, runtime
directory creation, validation handoff, and actionable auth-command failures.

**OneDrive Online Mount applet-driven setup verification:** June 18, 2026. A
disposable OneDrive Online Mount connection named `OneDrive onedriver setup
test` was created through the Add Connection window using remote/setup label
`onedriver-corporate-test` and mountpoint
`~/Cloud/cosmic-mounter-onedriver-ui-setup-test`. The first live attempt showed
that `jstaf/onedriver` writes `auth_tokens.json` under the per-connection cache
directory rather than the requested config file; validation was updated to
accept that metadata without reading token contents, and local path handling now
expands leading `~/` to the user's home directory. The second live attempt
passed Test Connection and Save Connection, installed applet-owned service
`cosmic-mounter-f909daa0-12f6-48e8-83d9-fc43037ea964.service`, and generated an
ExecStart using app-owned config/cache paths with expanded mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-onedriver-ui-setup-test`. Read-only disk
verification confirmed matching app-owned `auth_tokens.json` metadata under the
same connection ID. The unit was installed disabled and was not started.

**OneDrive Offline Mirror applet-driven setup implementation:** June 18, 2026.
The Add/Modify connection window now exposes `Start OneDrive Mirror Setup` for
OneDrive Offline Mirror connections. The action pins an unsaved draft to a
stable connection ID, creates the app-owned confdir, syncdir, and recovery
directories, and runs the `onedrive --reauth --auth-files
<auth-url-file>:<response-url-file>` handoff with a ten-minute timeout. The
applet opens the generated Microsoft auth URL with `xdg-open`, keeps a
selectable shell command as backup, and provides a response URL field for the
final Microsoft native-client redirect URL. After onedrive exits, the applet
reuses the existing OneDrive Offline Mirror validation path, including the
bounded dry-run preview. Credentials remain in `abraunegg/onedrive`'s app-owned
confdir; the applet stores only connection configuration. Automated tests cover
the auth-files command, runtime directory creation, validation handoff, stale
handoff cleanup, and actionable auth failures.

**OneDrive Offline Mirror auth handoff UI update:** June 18, 2026. The
Add/Modify connection window originally kept the `abraunegg/onedrive`
auth-files handoff in the form instead of relying on non-selectable notice text.
Live applet testing showed the local-browser redirect flow is not reliable from
the applet's non-interactive launch context, so the auth-files handoff is the
primary applet-driven path. The response URL is not stored in applet
configuration or echoed in notices, and the transient response file is written
with restricted permissions on Unix.

**OneDrive Offline Mirror setup correction:** June 18, 2026. Live setup showed
that `abraunegg/onedrive` refuses to complete authorization while `--dry-run` is
present, and that the applet's non-interactive launch context cannot answer the
terminal paste prompt. Setup therefore uses `--auth-files` without `--sync`,
`--monitor`, or `--dry-run`; if the client writes `refresh_token` and then exits
with its known missing `--sync`/`--monitor` usage complaint, the applet proceeds
to the bounded dry-run validation step. The Offline Mirror recovery directory
field is now optional; leaving it blank auto-generates a sibling recovery path
based on the mirror directory, for example
`~/Cloud/.cosmic-mounter-recovery/<mirror-name>-<connection-id>`, outside the
mirror tree. The UI shows the computed automatic recovery path instead of a
`/home/user/...` placeholder.

**OneDrive Offline Mirror applet-driven setup verification:** June 18, 2026.
The corrected Add/Modify flow was live-verified with the disposable connection
`OneDrive offline mirror test`, account/setup label `onedrive-personal-test`,
remote subtree `cosmic-mounter-test`, and mirror directory
`~/Cloud/cosmic-mounter-onedrive-offline-test`. The applet-driven auth-files
handoff completed, `abraunegg/onedrive` created the app-owned refresh token, and
the follow-up validation preview completed for the app-owned sync directory.
Save Connection wrote the applet configuration and installed the generated
disabled systemd user service
`cosmic-mounter-a92b706d-567e-4d69-9af5-95611ff742f0.service` with the
app-owned confdir/syncdir/recovery paths. The popup now exposes Preview as a
visible secondary Offline Mirror action and displays operation notices. Initial
Preview and Sync Now completed from the applet; Sync Now reported one notable
file-change line and wrote the `initial-sync-complete` marker. The manual step
of copying the final Microsoft redirect URL from browser history is functional
but not elegant; a separate open task now tracks an applet-owned localhost
callback helper to remove that step if feasible.

#### OneDrive Runtime And Validation Status Notes

These notes support the completed OneDrive runtime and validation groups above.

**OneDrive Online Mount runtime update:** June 18, 2026. Save Connection now
installs or updates the applet-owned `jstaf/onedriver` systemd user service for OneDrive
Online Mount connections. The generated unit uses per-connection applet-owned
config and cache paths, creates those directories before install/start, and is
enabled only when Start at login is selected. Popup Mount and Unmount now start
and stop the same managed `jstaf/onedriver` service. This wiring preserves existing
user-managed `jstaf/onedriver` setup; full live app-managed `jstaf/onedriver` verification
remains a separate checklist item because the isolated app-owned config may
need account authorization before the service can mount.

**OneDrive Online Mount app-managed verification:** June 18, 2026. The user
authorized a disposable app-owned `jstaf/onedriver` config using the existing corporate
OneDrive account. A disposable OneDrive Online Mount connection with ID
`4f4f7e18-9d74-4f72-9e4c-0ed1a6f6c101` was saved through the validated
configuration path, and the applet-owned systemd user service
`cosmic-mounter-4f4f7e18-9d74-4f72-9e4c-0ed1a6f6c101.service` was installed
disabled and inactive. Starting the unit mounted OneDrive at
`/home/uutzinger/Cloud/cosmic-mounter-onedriver-test` as `fuse.onedriver`.
A disposable marker file was written, read back, and removed through the mount.
Stopping the unit detached the mountpoint and left the unit inactive and
disabled. The test used per-connection applet config and cache roots under
`~/.config/cosmic-ext-applet-mounter/onedriver/` and
`~/.cache/cosmic-ext-applet-mounter/onedriver/`, preserving the existing
user-managed `jstaf/onedriver` setup.

**OneDrive Offline Mirror runtime update:** June 18, 2026. Save Connection now
installs or updates an applet-owned `abraunegg/onedrive` monitor service for
OneDrive Offline Mirror connections. The generated plan uses isolated
per-connection config directories under
`~/.config/cosmic-ext-applet-mounter/onedrive-sync/`, creates the sync and
recovery directories before install or run, and disables the service after
installation so synchronization is not started by saving. Preview and Sync Now
dispatch to `abraunegg/onedrive` for OneDrive Offline Mirror connections and
are now exposed in the saved connection's Modify action row. Preview uses
`--sync --dry-run --verbose`, preserves the selected
`--single-directory` subtree, and records an app-owned confirmation marker only
after a successful preview. Sync Now refuses initial sync until that marker
exists, then records initial sync completion after a successful sync.

**OneDrive Offline Mirror personal-account verification:** June 18, 2026.
`abraunegg/onedrive` 2.5.10 was authorized against a non-critical personal
Microsoft account using the documented `--auth-files` flow and an isolated
`--confdir` at
`~/.config/cosmic-ext-applet-mounter/onedrive-sync/84f5d5db-3b37-43c4-823f-5c726f6c0c74`.
The browser's final `nativeclient?code=...` response was treated as transient
secret material and was not retained in project files. The disposable remote
subtree `cosmic-mounter-test` was created, previewed with `--sync --dry-run`,
and synchronized once to upload
`cosmic-mounter-onedrive-marker.txt` from the isolated syncdir
`~/Cloud/cosmic-mounter-onedrive-mirror-test`. A follow-up dry-run completed
cleanly. Existing `jstaf/onedriver` configuration and mounts were not used.

**OneDrive Offline Mirror conflict/deletion verification:** June 18, 2026.
Deletion propagation was verified by removing the disposable marker file locally
and syncing the isolated `cosmic-mounter-test` subtree; `abraunegg/onedrive`
reported one local deletion and deleted
`cosmic-mounter-onedrive-marker.txt` from Microsoft OneDrive. A follow-up
`--download-file` check reported that the path no longer existed online.
Conflict preservation was verified by syncing a baseline
`cosmic-mounter-conflict.txt`, editing it locally, then using a temporary second
isolated `onedrive` profile with copied auth state in `/tmp` to upload a
different remote-side edit with `--upload-only --no-remote-delete`. The primary
sync detected the modified local file, renamed it to
`cosmic-mounter-conflict-urslabtop-safeBackup-0001.txt`, downloaded the remote
version as `cosmic-mounter-conflict.txt`, and uploaded the preserved local
safe-backup version. A final dry-run completed cleanly. The temporary actor
profile and sync directory were removed from `/tmp`.

**OneDrive Offline Mirror cleanup:** June 18, 2026. After explicit user
approval, the disposable remote subtree `cosmic-mounter-test` was removed from
Microsoft OneDrive with `onedrive --remove-directory`. A follow-up
`--download-file` check confirmed that a file under that subtree no longer
existed online. The local disposable mirror directory
`~/Cloud/cosmic-mounter-onedrive-mirror-test` was removed. The isolated
`abraunegg/onedrive` config/auth directory was intentionally left in place
because it is account state, not disposable mirror data.

**OneDrive Online Mount validation update:** June 18, 2026. Test Connection for
OneDrive Online Mount now validates `jstaf/onedriver` setup instead of showing a
placeholder follow-up message. The validation is metadata-only and does not read
provider tokens or launch authentication. It reports missing `onedriver`, missing
app-owned authentication state, empty/unavailable app-owned config files,
mountpoints that are files instead of directories, invalid mountpoint parents,
and active `fuse.onedriver` mount overlaps. A passing validation reports the
selected mountpoint and planned cache directory. Verification passed with
`cargo fmt --all -- --check`, `cargo test --all-targets`, and
`cargo clippy --all-targets -- -D warnings`.

**OneDrive Offline Mirror validation update:** June 18, 2026. Test Connection
for OneDrive Offline Mirror now validates `abraunegg/onedrive` setup instead of
showing a placeholder follow-up message. The validation checks that
`abraunegg/onedrive` is installed, the app-owned mirror has authentication state
metadata, sync/config/recovery directories are usable and separated, and the
mirror does not overlap an active `jstaf/onedriver` mount. It then runs a
bounded `onedrive --sync --dry-run --verbose` probe against the selected subtree.
Failure output is mapped into actionable messages for missing authentication,
tenant/admin-consent policy, expired or invalid OAuth responses,
resync/state-rebuild requirements, inaccessible remote subtrees, and
network/VPN readiness. Verification passed with `cargo fmt --all -- --check`,
`cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.

**Shared OneDrive validation UI update:** June 18, 2026. The Add/Modify
Connection editor now shows visible OneDrive setup guidance for the selected
mode, while hover help explains the same validation contract. Test Connection
and Save Connection share the same OneDrive validation paths: `jstaf/onedriver`
for Online Mount and `abraunegg/onedrive` for Offline Mirror. Save Connection
blocks before writing applet configuration when OneDrive validation fails, and
continues to install the managed unit only after validation passes. The UI
continues to avoid storing provider credentials; validation uses metadata and
bounded dry-run probes only. Verification passed with
`cargo fmt --all -- --check`, `cargo check --all-targets`,
`cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.

#### Add/Modify UI And Rclone Runtime Status Notes

**Layout implementation update:** June 17, 2026. Add and Modify now use the
same editor surface: Add starts with defaults, Modify prepopulates the selected
connection. The previous Add/Import/Refresh/Help navigation row was removed
from the editor. The editor now has a top action row with Test Connection and
Save Connection in both modes, Import only in Add mode, and Enable/Disable plus
Remove only in Modify mode. Test Plan and Test Existing were combined into Test
Connection, which validates the current form values. The duplicated Enabled
toggle was removed from mode-specific settings; the top Enable/Disable action
updates the draft `enabled` field and Save Connection commits it.

**Field help implementation update:** June 17, 2026. Add/Modify fields and
primary popup controls now attach COSMIC/libcosmic hover tooltips directly to
the relevant input, button, choice row, status chip, or VPN chip. Visible help
buttons are not shown unless a future control cannot reasonably carry its own
tooltip. Tooltips are positioned above the source control by default so the
control remains visible while the help is displayed. Tooltip display is delayed
by one second to avoid distracting popups during normal pointer movement.
Tooltip wrappers are not nested; each interactive control owns at most one
tooltip so help text does not visually overlay other help text.

**Connection settings window update:** June 17, 2026. Add, Modify, and Import
now launch as a standalone COSMIC settings application window instead of an
embedded applet child window. This matches the pattern used by other COSMIC
applets for larger settings surfaces and allows the real window title to be set
to `Cloud Mounter Connection Settings` instead of inheriting the toolkit default
title.

**Rclone remote selection update:** June 17, 2026. The Add/Modify wizard now
separates rclone remote/account selection by provider. Google Drive shows and
accepts `drive` remotes, Box shows and accepts `box` remotes, and SMB shows and
accepts `smb` remotes. Detect rclone remotes reads `rclone config dump`, parses
the JSON response, stores only remote names and backend types for UI selection,
and ignores secrets or unrelated backend configuration. Provider-specific setup
guidance is shown in the wizard; applet-driven setup actions are now implemented
for SMB, Box, and Google Drive and tracked in the Provider Setup Flow
Implementation group.

**Rclone remote selection manual verification:** June 17, 2026. The user
confirmed Detect rclone remotes shows the expected provider-specific choices for
Google Drive, Box, and SMB. Disposable verification subtrees were created for
the next remote-access validation step: `ua_box:Utzinger/cosmic-mounter-ui-test`,
`uutzinger_gdrive:cosmic-mounter-ui-test`, and
`ua_engr:Research/Utzinger/cosmic-mounter-ui-test`.

**Rclone remote verification update:** June 17, 2026. Test Connection now runs
as an asynchronous task. For Google Drive, Box, and SMB it first validates that
the selected rclone remote exists and has the expected backend, then performs a
bounded read-only `rclone lsf --max-depth 1 remote:subtree` access check. The UI
reports actionable failures for missing remotes, wrong backend type,
inaccessible subtrees, authorization/authentication failures, and network or VPN
readiness problems. OneDrive account/setup validation is handled by the OneDrive
runtime and validation groups.

**Rclone Online Mount save-side-effect update:** June 17, 2026. Save Connection
now writes the validated applet configuration and, for Google Drive, Box, and
SMB Online Mount connections, asynchronously installs or updates the
applet-owned systemd user service through the managed unit controller. The unit
is verified, `systemctl --user daemon-reload` is run, and the unit is enabled
only when Start at login is selected; otherwise it is disabled. Saving does not
start the mount service.

**Rclone Online Mount runtime update:** June 17, 2026. Popup Mount and Unmount
now dispatch to the managed runtime backend for saved Google Drive, Box, and SMB
Online Mount connections. Mount prepares the mountpoint, runtime socket
directory, and rclone cache directory, clears previous failed unit state, then
starts the applet-owned systemd user service. Unmount stops the same service.
Generated service units now include `RuntimeDirectory=cosmic-ext-applet-mounter`
and `SuccessExitStatus=130 143` so direct/start-at-login systemd starts have the
needed rclone RC socket directory and normal rclone SIGTERM shutdown is treated
as a clean stop.

**Rclone Offline Mirror save-side-effect update:** June 18, 2026. Save
Connection now writes the validated applet configuration and, for Google Drive,
Box, and SMB Offline Mirror connections, asynchronously installs or updates the
applet-owned systemd user sync service and timer through the managed unit
controller. The mirror directory, work root, and recovery directory are created
if needed, but the service and timer are explicitly disabled so no destructive
or bidirectional sync starts until preview and initial synchronization are
confirmed.

**Rclone Offline Mirror preview/Sync Now update:** June 18, 2026. Preview and
Sync Now dispatch to the managed rclone Offline Mirror runtime for Google Drive,
Box, and SMB. They were initially exposed in the popup and are now exposed in
the saved connection's Modify action row. Preview runs
`rclone bisync --resync --dry-run`
before the first sync, records a small applet-owned confirmation marker in the
per-connection work directory when it succeeds, and tells the user to press Sync
Now to confirm the initial synchronization. Sync Now refuses an initial sync
until that preview marker exists, then runs the initial `--resync` once and
records initial completion; later Sync Now runs normal bisync without
`--resync`. Applet markers and rclone filter files live in the work directory,
not in the mirror tree. Offline Mirror rows now use Start/Stop as the primary
background-sync control: Start enables/starts the applet-owned timer or monitor
after preview and confirmed initial Sync Now, while Stop disables/stops the
timer or monitor and leaves manual Sync Now available. Additional readiness and
metered-network gating before background start remains an open runtime
follow-up.

**Box Offline Mirror controlled verification:** June 18, 2026. A disposable Box
Offline Mirror verification passed against remote `ua_box` using parent subtree
`Utzinger/cosmic-mounter-ui-test` and unique child subtree
`offline-mirror-20260618-codex`. Disposable local roots were
`/tmp/cosmic-mounter-box-offline-mirror.v2NpoO`,
`/tmp/cosmic-mounter-box-offline-work.Ky63X3`, and
`/tmp/cosmic-mounter-box-offline-recovery.JRJaRB`. The initial
`rclone bisync --resync --dry-run` preview succeeded after creating the empty
remote root and `RCLONE_TEST` access sentinel on both sides. The confirmed
initial `--resync` uploaded the local seed files to Box. A later normal bisync
without `--resync` copied one new local file to Box and one new Box file back to
the local mirror. Final local and remote listings matched. The disposable remote
test folder and local `/tmp` directories were removed; the remote recovery purge
reported `directory not found`, meaning no recovery backup folder was created
for this non-destructive run.

**Google Drive Offline Mirror controlled verification:** June 18, 2026. A
disposable Google Drive Offline Mirror verification passed against remote
`uutzinger_gdrive` using parent subtree `cosmic-mounter-ui-test` and unique
child subtree `offline-mirror-20260618-codex`. Disposable local roots were
`/tmp/cosmic-mounter-gdrive-offline-mirror.UygY0F`,
`/tmp/cosmic-mounter-gdrive-offline-work.vwTgqg`, and
`/tmp/cosmic-mounter-gdrive-offline-recovery.RyA6xu`. The Google Drive filter
file excluded browser-native document sidecar types. The initial
`rclone bisync --resync --dry-run` preview succeeded with an `RCLONE_TEST`
access sentinel on both sides and did not write the seed file. The confirmed
initial `--resync` uploaded the local seed file to Google Drive. A later normal
bisync without `--resync` copied one new local file to Google Drive and one new
Google Drive file back to the local mirror. Final local and remote listings
matched. The disposable remote test folder and local `/tmp` directories were
removed; the remote recovery purge reported `directory not found`, meaning no
recovery backup folder was created for this non-destructive run.

**SMB Offline Mirror controlled verification:** June 18, 2026. With Cisco VPN
readiness available, a disposable SMB Offline Mirror verification passed against
remote `ua_engr` using parent subtree
`Research/Utzinger/cosmic-mounter-ui-test` and unique child subtree
`offline-mirror-20260618-codex`. Disposable local roots were
`/tmp/cosmic-mounter-smb-offline-mirror.fxCiXR`,
`/tmp/cosmic-mounter-smb-offline-work.H5JDST`, and
`/tmp/cosmic-mounter-smb-offline-recovery.VMBkHx`. The first preview attempt
showed that SMB remote recovery must use a path inside the writable share
subtree; `ua_engr:.cosmic-mounter-recovery/...` is not valid for this SMB
remote. Retrying with remote recovery under
`Research/Utzinger/cosmic-mounter-ui-test/.cosmic-mounter-recovery/...`
succeeded. The initial `rclone bisync --resync --dry-run` preview succeeded
with an `RCLONE_TEST` access sentinel on both sides and did not write the seed
file. The confirmed initial `--resync` uploaded the local seed file to SMB. A
later normal bisync without `--resync` copied one new local file to SMB and one
new SMB file back to the local mirror. Final local and remote listings matched.
The disposable remote test folder and local `/tmp` directories were removed;
the remote recovery purge reported `directory not found`, meaning no recovery
backup folder was created for this non-destructive run.

**Box Online Mount controlled verification:** June 17, 2026. A disposable Box
Online Mount connection was added through the validated configuration path with
remote `ua_box`, subtree `Utzinger/cosmic-mounter-ui-test`, and mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-box-test`. The applet-owned unit
`cosmic-mounter-1e31ac32-dcae-4ac7-9546-7a82437d04f4.service` was installed
disabled and inactive, then started successfully. `findmnt` reported a
`fuse.rclone` mount at the disposable mountpoint, and stopping the service
detached the mountpoint and left the unit inactive without failed state. Direct
rclone write/read/delete against the disposable Box subtree succeeded. A
real-session FUSE write/read/delete through the managed mount also succeeded.
Earlier read-only or disconnected FUSE results were caused by attempting to
write to the real FUSE mount from inside the Codex sandbox rather than from the
real user session. The Box Online Mount vertical slice is complete.

**Google Drive Online Mount controlled verification:** June 17, 2026. A
disposable Google Drive Online Mount connection was added through the validated
configuration path with remote `uutzinger_gdrive`, subtree
`cosmic-mounter-ui-test`, and mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-gdrive-test`. The applet-owned unit
`cosmic-mounter-bbd33f7c-f9db-4a5f-b7b3-71e3b0e3d370.service` was installed
disabled and inactive, then started successfully. The first start attempt hit a
transient Google Drive API rate-limit/quota error and systemd restarted the
unit; the retry mounted successfully. `findmnt` reported a writable
`fuse.rclone` mount, real-session write/read/delete through the mount
succeeded, and stopping the service detached the mountpoint and left the unit
inactive without failed state. The Google Drive Online Mount vertical slice is
complete.

**SMB Online Mount controlled verification:** June 17, 2026. With Cisco VPN
readiness available, a disposable SMB Online Mount connection was added through
the validated configuration path with remote `ua_engr`, subtree
`Research/Utzinger/cosmic-mounter-ui-test`, and mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-smb-test`. The applet-owned unit
`cosmic-mounter-3e04d0b2-83be-48ed-813a-fc7e727df0cd.service` was installed
disabled and inactive, then started successfully. `findmnt` reported a writable
`fuse.rclone` mount, real-session write/read/delete through the mount
succeeded, and stopping the service detached the mountpoint and left the unit
inactive without failed state. rclone reported that poll interval is not
supported by this SMB remote; this was informational and did not block mount,
write, or unmount behavior. The SMB Online Mount vertical slice is complete.

**VPN selector implementation update:** June 17, 2026. The Add/Modify VPN
section now presents a grouped selector for no VPN, configured NetworkManager
VPN profiles, and configured Cisco Secure Client dependencies. Profile details
such as kind, external profile/account behavior, readiness checks, timeout, and
activation expectation are shown as tooltip/help text on the profile choice
rather than inline body text. The disconnect-when-unused policy appears only
when a VPN dependency is selected. A Detect VPNs action imports existing
NetworkManager VPN profiles by UUID and Cisco Secure Client availability as
applet VPN references without storing credentials. Repeated detection updates
known references rather than duplicating them. Detection result details do not
remain in the VPN section; success/failure is reported through the normal app
notice path.

**VPN/help cleanup update:** June 17, 2026. The VPN section no longer wraps the
entire selector in a parent tooltip, preventing child tooltips from visually
overlaying another tooltip. No VPN, Detect VPNs, NetworkManager profile choices,
Cisco Secure Client choices, and the disconnect-when-unused control each carry
their own single tooltip. Verbose selected-profile text remains available as
choice help rather than inline form text.

## Historical Gate 1: Isolated Release Candidate (Superseded)

**Gate 1 approved:** June 15, 2026. User approval: "I approve Gate 1 and
authorize Phase 10 controlled manual testing with disposable/non-critical data."
This approval is preserved as historical evidence and is superseded by later
UI/settings, provider setup, and runtime changes.

## Current Gate 1 Recheck: Integrated Release Candidate

**Current Gate 1 status:** Approved on June 20, 2026. Historical approval
remains recorded above but does not represent the current integrated
release-candidate state. Automated verification, Gate 1 review items, and
manual keyboard/accessibility verification are current. User approval: "I
approve Gate 1."

**Current Gate 1 automated verification:** June 19, 2026. `just verify` passed:
format check, `cargo check --all-targets`, clippy with `-D warnings`, all unit
tests, and example target builds completed successfully.

**Current Gate 1 automated verification refresh:** June 20, 2026. `just verify`
passed again after the Add/Modify UI cleanup, main popup cleanup, and removal of
the temporary settings-desktop identity workaround. The run completed format
check, `cargo check --all-targets`, clippy with `-D warnings`, 88 library
tests, 43 app/main tests, and example target builds.

**Current Gate 1 review:** June 20, 2026. Generated unit/timer review is covered
by deterministic service/timer snapshot tests, applet ownership/UUID marker
tests, and no-shell/no-secret command rendering tests. Side-effect review keeps
real changes limited to applet-owned generated units and applet-owned
configuration; removal preserves credentials, local data, caches, recovery
data, and legacy files. Sync deletion/conflict/recovery review matches the
current runtime: previews report deletes/conflicts, initial sync requires
preview confirmation, conflict losers are preserved, deleted/overwritten files
use recovery locations, and recovery retention is bounded. Detach,
background-sync, metered-network, and VPN review matches the current
implementation: background Start now reuses the sync policy gate, checks
network/VPN readiness where available, honors metered-network pause policy, and
does not enable timers/monitors before Preview plus confirmed initial Sync Now.
Lazy unmount remains confirmation-gated and blocked by queued writes, automatic
sync pauses on metered networks unless explicitly overridden, manual Sync Now
remains available, VPN credentials are not stored, and only applet-activated
VPNs may be disconnected after no active connection still needs them.

**Current Gate 1 implementation refresh:** June 20, 2026. Add/Modify now shows a
compact Summary row with the selected engine, generated unit validation preview,
safety confirmation policy, and VPN dependency summary. Legacy import
confirmation now creates the applet-managed replacement unit directly while
preserving the original legacy service. Removing a connection now also
stop/disables and removes only applet-owned generated units with ownership
markers, while preserving credentials, local data, caches, recovery data, and
external services. Verification passed with `cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets` with 88 library tests and 43 app/main tests.

## Historical Phase 10: Controlled Manual Testing Evidence (Superseded)

This section preserves prior controlled manual testing with disposable
connections and non-critical data. It is not the current final integrated
manual test pass for the latest applet UI/runtime.


**Phase 10 progress:** Dependency detection passed on June 15, 2026 with
rclone 1.74.3, `jstaf/onedriver` 0.15.0, `abraunegg/onedrive` 2.5.10, FUSE,
NetworkManager, and optional diagnostics available. A disposable local-to-local
rclone bisync mirror test passed using temp root
`/tmp/cosmic-mounter-bisync.zepiT3`; the test confirmed that initial rclone
bisync preview must use `--resync --dry-run`, initial synchronization must use
`--resync`, and later normal bisync runs complete without `--resync`. Real
service-folder import preview passed against `~/.config/systemd/user/`: four
compatible rclone services were parsed, backup files were ignored, no
unsupported options were found, and no active-service or target conflicts were
reported. No imported service was created, disabled, started, or stopped. A
Google Drive disposable online-mount attempt against
`uutzinger_gdrive:cosmic-mounter-test` was safely cleaned up after Drive API
quota/rate limiting caused directory reads to fail. The disposable Box online
mount test passed on June 16, 2026 against `ua_box:cosmic-mounter-test`: rclone
created the remote test folder, mounted it at
`/tmp/cosmic-mounter-online-box-mount` with full VFS cache, wrote and uploaded a
tiny marker file, read the marker back through direct rclone access, removed the
marker, confirmed the remote test folder was empty, and cleanly unmounted. A
disposable Box offline mirror test passed on June 16, 2026 against
`ua_box:cosmic-mounter-test` using `/tmp/cosmic-mounter-box-mirror`,
`/tmp/cosmic-mounter-box-bisync-work`, and
`/tmp/cosmic-mounter-box-recovery`: initial `--resync --dry-run`, initial
`--resync`, and later normal bisync all succeeded; local offline edits and
remote-only changes converged; a local deletion propagated to Box and moved the
deleted remote file into the configured recovery directory; simultaneous edits
preserved both versions as `.conflict1` and `.conflict2` files on both sides.
The disposable Box test folder, disposable remote recovery folder, and local
`/tmp` mirror/work/recovery directories were removed after verification.
Metered-network policy testing passed on June 16, 2026: the sync decision path
pauses automatic synchronization when a connection is marked metered and the
connection has not opted into metered sync, while explicit manual Sync Now is
allowed. A read-only NetworkManager check showed the currently connected devices
were not metered, so no live NetworkManager profile was modified to force a
metered state. NetworkManager VPN readiness testing passed on June 16, 2026 in
read-only mode: `nmcli` listed configured connections including the WireGuard
VPN profile `SalterLab`, reported active Wi-Fi, loopback, and externally
connected `tailscale0` tunnel state, and exposed gateway/DNS readiness fields;
the applet VPN parser/readiness tests passed. No VPN was activated or
disconnected. Cisco Secure Client component detection initially found CLI, GUI,
and agent binaries under `/opt/cisco/secureclient/bin`, but the expected systemd
unit on this host is `vpnagentd.service`, not
`cisco-secure-client-vpnagentd.service`. After the user started and enabled
`vpnagentd.service`, Cisco CLI status reported `Disconnected` and `Ready to
connect`, the applet dependency inventory reported Cisco Secure Client
available, and the user confirmed a successful manual corporate VPN connect and
disconnect on June 16, 2026. OneDrive Online mount testing with `jstaf/onedriver` passed
on June 16, 2026:
no existing `jstaf/onedriver` process or mount was active, the test reused the existing
authentication configuration without reading its contents, mounted OneDrive at
`/tmp/cosmic-mounter-onedriver-mount` with isolated cache
`/tmp/cosmic-mounter-onedriver-cache`, wrote/read/removed one disposable marker
file through the mount, cleanly unmounted, verified the process exited, and
removed the temporary mount/cache directories. Offline cached read-only behavior
was not forced because that would require deliberately disrupting network
connectivity. Controlled connectivity-loss policy testing passed on June 16,
2026 without disrupting the live network: provider decision tests confirmed
rclone safe detach is allowed only when the mount is healthy enough and no
writes are pending, lazy unmount remains gated behind the approved policy,
pending or active VFS writes block automatic detach, cache errors and write
queues are surfaced, and remount backoff is bounded and resettable. Removal
preservation testing passed on June 16, 2026 with temporary applet-owned units:
removing a managed unit deletes only that unit and preserves separate credential,
data, cache, recovery, and original legacy files; external unmanaged units are
still protected from applet removal. Confirmed import dry-run testing passed on
June 16, 2026 using the real `~/.config/systemd/user/` service folder as input:
four compatible services produced confirmed applet-owned replacement unit plans
in a temporary output directory, originals were marked for preservation and not
disabling, generated units contained applet ownership markers and fixed rclone
argument vectors, and the temporary output was removed after inspection. No
replacement unit was installed into the live user systemd manager.
`abraunegg/onedrive` isolated dry-run preflight was attempted on June 16, 2026
with `/tmp/cosmic-mounter-onedrive-conf`,
`/tmp/cosmic-mounter-onedrive-sync`, `--single-directory
cosmic-mounter-test`, and `--dry-run`; the client reached Microsoft OneDrive
successfully but reported that authorization is required and cannot be completed
with `--dry-run`. Temporary OneDrive test directories were removed. At that
time, the full non-critical OneDrive mirror test was blocked because the
available corporate Microsoft account did not permit this authorization flow,
and an alternate Microsoft account redirected back to the corporate tenant. On
June 18, 2026, this was resolved with a personal Microsoft account authorized
through the documented `--auth-files` flow, and the disposable offline mirror
test passed as recorded in the OneDrive runtime section.
`jstaf/onedriver` offline cached read-only testing passed on June 16, 2026 using the
corporate OneDrive account, a temporary mountpoint
`/tmp/cosmic-mounter-onedriver-offline-mount`, and isolated cache
`/tmp/cosmic-mounter-onedriver-offline-cache`: a disposable
`cosmic-mounter-test/offline-cache-marker.txt` file was created and cached
online, `jstaf/onedriver` was restarted with an invalid per-process proxy to simulate
loss of network access, the process logged that it marked the filesystem
offline, the cached file remained readable, and creation of a new file failed
with `Read-only file system` while `jstaf/onedriver` logged that it refused the write to
avoid data loss. The mount was restored online long enough to delete the
disposable OneDrive test folder, then all temporary local mount/cache
directories were removed.

## Current Phase 10: Integrated Manual Testing Recheck

Use the current applet UI, current generated units/timers, and
disposable/non-critical data. Record whether test artifacts are intentionally
retained for inspection or cleaned up.

**Pre-Phase-10 automated verification:** June 20, 2026. User reported that
`verify` passed before starting the Current Phase 10 integrated manual rechecks.

**Dependency detection and guidance UI recheck:** June 20, 2026. `just verify`
passed on the current code before this manual recheck. The applet diagnostic
inventory reported rclone 1.74.3, `jstaf/onedriver` 0.15.0,
`abraunegg/onedrive` 2.5.10, FUSE3 3.14.0, NetworkManager 1.46.0, Cisco Secure
Client detected, systemd user support, and `fuser` available. The installed
settings UI was opened from `~/.local/bin/cosmic-ext-applet-mounter --settings`;
the user checked provider/mode setup, Test Connection, Save Connection, setup
actions, and account/remote tooltips and confirmed that the UI passed and gave
useful dependency/setup information without obsolete version warnings.

**Applet-created rclone OAuth remote recheck:** June 20, 2026. Existing
applet-created verification remotes `cosmic_mounter_box_live_verify` and
`cosmic_mounter_gdrive_live_verify` were present in rclone config with backend
types `box` and `drive` respectively. Initial read-only `rclone lsf` checks
inside the sandbox failed due to restricted DNS/network access. Re-running the
same read-only checks with approved network access succeeded for both Box and
Google Drive. Google Drive reported one provider-side dangling shortcut notice,
but the remote listed successfully.

**Rclone Online Mount recheck progress:** June 20, 2026. Applet-owned Box
Online Mount unit `cosmic-mounter-2777721d-ff56-4faa-a8b1-4f587e31d0c1.service`
and Google Drive Online Mount unit
`cosmic-mounter-18684a83-01f1-49f8-bb27-232b936c665d.service` were initially
inactive/disabled. Starting both units through `systemctl --user start`
succeeded; both reached `active/running`, appeared as `fuse.rclone` mounts, and
their mountpoint directories listed successfully. Stopping both units through
`systemctl --user stop` succeeded; both returned to `inactive/dead` and the
rclone FUSE mount table was empty. SMB Online Mount was not attempted because
the prerequisite read-only `rclone lsf ua_engr:Research/Utzinger/cosmic-mounter-ui-test`
probe hung until interrupted and reported DNS timeout for
`engr-drive.bluecat.arizona.edu`; revisit SMB with the VPN readiness test.

**Rclone Offline Mirror recheck:** June 20, 2026. Applet-owned Box
Offline Mirror unit/timer
`cosmic-mounter-80ee88c8-d44b-405d-911a-427dc6cace8f.service/.timer` and
Google Drive Offline Mirror unit/timer
`cosmic-mounter-6e6be0c8-71b7-413e-bd1b-1625406c8e82.service/.timer` were
initially inactive/disabled, matching the paused/manual state. Both mirrors had
initial preview and initial sync markers, so the recheck used normal bisync
preview/sync rather than first-run resync confirmation. Box and Google Drive
Preview passed with `rclone bisync --dry-run`; Box reported one local
timestamp-only change queued and Google Drive reported no changes. Box and
Google Drive Sync Now passed with normal `rclone bisync`. Starting both timers
through `systemctl --user enable/start` succeeded; the timers entered
`active/elapsed` and the associated services completed successfully with
`Result=success`. Stopping/disabling both timers succeeded and returned them to
`inactive/dead` and `disabled`.

With Cisco VPN connected, a disposable SMB Offline Mirror connection named
`Disposable SMB Offline Mirror Test` was generated for remote
`ua_engr:Research/Utzinger/cosmic-mounter-ui-test`, local mirror
`/tmp/cosmic-mounter-smb-mirror`, recovery directory
`/tmp/cosmic-mounter-smb-recovery`, and managed unit/timer
`cosmic-mounter-9e6d9640-9c99-48ef-86c1-b3e91d8dc146.service/.timer`. The
first manual preview intentionally exposed two setup requirements: remote
backup recovery must not overlap the mirrored subtree, and rclone bisync
requires an `RCLONE_TEST` access-check file on both sides. Using the generated
non-overlapping recovery path and a tiny disposable `RCLONE_TEST` marker,
SMB initial preview passed with `rclone bisync --resync --dry-run`; confirmed
initial Sync Now passed with `rclone bisync --resync`. Starting the timer
through `systemctl --user enable/start` succeeded; the timer entered
`active/elapsed`, the associated service completed with `Result=success`, and
stopping/disabling the timer returned it to `inactive/dead` and `disabled`.

**OneDrive Online Mount recheck:** June 20, 2026. A disposable applet-owned
OneDrive Online Mount connection named `One Drive Online Mount Test` was
authenticated with the existing corporate account and saved with generated unit
`cosmic-mounter-bdda0155-f69d-4734-b02c-7cc5e9c7c8ac.service`, isolated
config/cache, and mountpoint
`/home/uutzinger/Cloud/cosmic-mounter-onedriver-online-test`. The mount became
active as `fuse.onedriver`. A disposable file
`cosmic-mounter-onedriver-test.txt` was written, synced through onedriver,
listed, and removed. The generated service did not disturb the existing
user-managed onedriver setup.

Unmount was invoked from the popup, but `jstaf/onedriver` failed clean
unmount with `Device or resource busy` and exited with status 128, leaving a
stale FUSE mount attached while the service was failed/dead. Confirmed manual
lazy cleanup with `fusermount3 -uz
/home/uutzinger/Cloud/cosmic-mounter-onedriver-online-test` detached the stale
mount, and `systemctl --user reset-failed
cosmic-mounter-bdda0155-f69d-4734-b02c-7cc5e9c7c8ac.service` returned the
service to `inactive/dead` and `disabled`. A controller regression fix now makes
failed service state win over lingering mount-table presence, so the popup
reports Error rather than misleading Mounted in this state. Gate 2 retains an
open follow-up to implement visible lazy-unmount confirmation/recovery from the
popup after clean onedriver/FUSE unmount fails.

**Lazy-unmount Repair implementation:** June 20, 2026. The popup now treats an
Online Mount Error row as a Repair action instead of a Mount action. Repair is
explicitly confirmed: the first Repair click arms lazy-unmount recovery and
warns that it will run `fusermount3 -uz` on the connection mountpoint and reset
the generated service; the second Repair click performs the recovery. The
runtime stops the generated service if needed, runs the existing
`lazy_unmount_request`, and resets failed systemd state. The change is
installed with `just install-user`. Verification passed with
`cargo fmt --all`, `cargo check --all-targets`,
`cargo test controller::tests:: -- --nocapture`, and
`cargo test app::tests:: -- --nocapture`. Added regression tests verify that a
failed service with a lingering FUSE mount reports Error rather than Mounted,
and that Error online-mount rows use Repair as the compact popup primary
action. Live COSMIC verification of the two-click Repair path remains open.

**OneDrive Offline Mirror recheck:** June 20, 2026. Applet-owned OneDrive
Offline Mirror service
`cosmic-mounter-a92b706d-567e-4d69-9af5-95611ff742f0.service` was tested
against the personal OneDrive account using sync directory
`/home/uutzinger/Cloud/cosmic-mounter-onedrive-offline-test` and
`--single-directory cosmic-mounter-test`. Dry-run preview passed with
`onedrive --sync --dry-run --verbose`; the client reached the Microsoft
OneDrive service, initialized the API, identified a personal account, and
completed without errors. Sync Now passed with `onedrive --sync`. Starting the
managed monitor service through `systemctl --user enable/start` succeeded and
reached `active/running`; stopping/disabling it succeeded and returned it to
`inactive/dead` and `disabled`.

**VPN selection/readiness recheck:** June 20, 2026. Read-only NetworkManager
profile discovery required host D-Bus access outside the sandbox and listed
`SalterLab` as a `wireguard` profile with UUID
`51424a59-495c-4483-ad44-a0bf49327d5e`. The active applet configuration stores
SalterLab only as a NetworkManager external profile UUID with a
`NetworkManagerState` readiness check; no VPN credentials are stored. Cisco
Secure Client read-only status checks reported version `5.1.10.233`, active
`vpnagentd.service`, and a connected Engineering SSL VPN tunnel. The active
applet configuration stores Cisco as a Cisco dependency reference without an
external credential/profile id. No `nmcli connection up/down` or Cisco
connect/disconnect command was run during this recheck, so no unintended
disconnect was introduced.

**SMB Online Mount recheck:** June 20, 2026. With Cisco VPN connected, a
read-only `rclone lsf ua_engr:Research/Utzinger/cosmic-mounter-ui-test`
completed successfully. The disposable applet-owned SMB Online Mount unit
`cosmic-mounter-3e04d0b2-83be-48ed-813a-fc7e727df0cd.service` started
successfully, reached `active/running`, and appeared in the FUSE mount table as
`/home/uutzinger/Cloud/cosmic-mounter-smb-test` from
`ua_engr:Research/Utzinger/cosmic-mounter-ui-test`. Stopping the unit returned
it to `inactive/dead`; the follow-up FUSE mount table check found no remaining
rclone mount.

**Import preview/replacement recheck:** June 20, 2026. The
`legacy_import_preview` example scanned
`/home/uutzinger/.config/systemd/user` and found 12 compatible units, including
applet-owned generated rclone/onedriver units and legacy `rclone-ua-box`,
`rclone-ua-engr`, `rclone-ua-gdrive`, and `rclone-uutzinger-gdrive` units.
Preview reported provider, remote/subpath, local target, cache directory, start
at login state, active conflicts, target conflicts, and unsupported options for
each unit. The `legacy_import_confirm_dry_run` example then wrote 12 managed
replacement plans to `/tmp/cosmic-mounter-phase10-import-dry-run`, preserving
originals and not disabling original units. No live user systemd service was
replaced during this dry-run recheck.

**Removal cleanup policy recheck:** June 20, 2026. Live removal was not run
against the current applet configuration because it would remove real saved
connections. The cleanup policy was rechecked with targeted automated tests:
`cargo test services::tests::removing_managed_unit_preserves_user_data_and_originals`
passed, confirming managed-unit removal deletes only the applet-owned unit in a
temporary store while preserving separate user data/originals; and
`cargo test import::tests::import_replacement_requires_confirmation_and_preserves_original`
passed, confirming replacement import requires confirmation and records
original preservation. This validates the removal policy without changing the
live configuration, credentials, local data, cache, recovery data, or legacy
service files.

**Accessible label recheck:** June 20, 2026. Automated label coverage passed
with `cargo test controller::tests::labels_are_stable_for_popup_rows` and
`cargo test controller::tests::row_accessible_text_does_not_depend_on_color`.
These tests verify stable provider/status/operation labels and row accessible
text that does not depend only on color. User manual verification confirmed
keyboard navigation works in the current slider-based popup and Add/Modify
window.

**Disposable data/generated-unit disposition recheck:** June 20, 2026. After
the Phase 10 runtime tests, `findmnt` reported no active `fuse.rclone` or
onedriver FUSE mounts. `systemctl --user list-units` reported no loaded
`cosmic-mounter-*.service` or `cosmic-mounter-*.timer` units. Installed
applet-owned unit files remain present for ongoing inspection and future
testing. After the later SMB Offline Mirror recheck, all 13 service unit files
and all 3 timer unit files are disabled. Disposable local/remote test data and
generated unit files are intentionally retained at this point; they are not
active.


**Current Phase 10 status:** Completed after current Gate 1 approval on
June 20, 2026. Dependency checks, rclone OAuth remote checks, rclone Online
Mount checks, Box/Google rclone Offline Mirror checks, OneDrive Offline Mirror
checks, OneDrive Online Mount checks, VPN detection/readiness checks, import
checks, removal policy checks, keyboard/accessibility checks, and generated-unit
disposition checks have passed. The OneDrive Online Mount recheck exposed a
Gate 2 follow-up for visible lazy-unmount confirmation/recovery after clean
onedriver/FUSE unmount failure.

## Historical Phase 11: Documentation and Packaging (Superseded)

**Phase 11 completed:** June 16, 2026. README and dependency documentation now
cover runtime installation/version requirements, Online mount versus Offline
mirror tradeoffs, conflict/deletion/recovery behavior, authentication and VPN
behavior, generated units, legacy imports, removal/uninstall behavior, and
known limitations. The MIT license credits Urs Utzinger and OpenAI Codex.
Desktop metadata, AppStream metainfo, and the SVG icon were finalized and
validated with `desktop-file-validate` and `appstreamcli validate --no-net`.
Staged installation to `target/stage/usr` and staged uninstallation with
`just rootdir=target/stage prefix=/usr uninstall` passed without touching the
host install. `just verify` passed with 84 tests.

**Historical Phase 11 status:** Preserved as documentation for the earlier
release candidate; superseded by current UI/runtime, provider setup, import,
and OneDrive authentication changes.

## Current Phase 11: Documentation and Packaging Recheck

**Current Phase 11 status:** Completed on June 20, 2026. README now documents
the compact popup, Add/Modify workflow, Start/Stop versus Preview/Sync Now,
active/VPN summaries, generated units, background sync gating, import,
removal/cleanup behavior, OneDrive client isolation, OAuth/security, data
integrity warnings, source-build workflow, and known limitations. Dependency
documentation is now condensed to installation and version checks for rclone,
`jstaf/onedriver`, `abraunegg/onedrive`, FUSE, NetworkManager, Cisco, and
optional diagnostics.
Validation passed with
`desktop-file-validate resources/app.desktop`,
`appstreamcli validate --no-net resources/app.metainfo.xml`,
`just stage target/stage-phase11`, and
`just rootdir=target/stage-phase11 prefix=/usr uninstall`; staged uninstall left
no staged files behind.

## Historical Gate 2: Version 0.1 Completion (Superseded)

**Gate 2 approved:** June 16, 2026. User approval: "I believe we can pass gate
2 also and check off the codex working rules." At approval time, the
`abraunegg/onedrive` account-backed mirror test was documented as blocked by
external Microsoft tenant/account policy and accepted as a known limitation for
version 0.1. On June 18, 2026, the personal-account offline mirror verification
passed. Final June 16 verification passed with `just verify`: formatting, check,
clippy, and 84 tests succeeded.

**Historical Gate 2 status:** Preserved, but superseded by later UI/settings,
provider setup, import, provider runtime, and OneDrive authentication changes.

## Current Gate 2: Version 0.1 Release Candidate

**Current Gate 2 status:** Release-candidate checks are complete except for
explicit user approval.

**Automated verification:** June 20, 2026. `just verify` passed:
`cargo fmt --all -- --check`, `cargo check --all-targets`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-targets`. The test suite reported 89 library tests and 44
application tests passing. `desktop-file-validate resources/app.desktop` passed.
`appstreamcli validate --no-net resources/app.metainfo.xml` passed with one
pedantic notice and no validation failure. A source-tree scan for common secret
markers found only documentation, tests, and redaction code paths; no live
credential material was identified.

**Live lazy-unmount Repair verification:** June 20, 2026. The disposable
OneDrive Online Mount test unit
`cosmic-mounter-bdda0155-f69d-4734-b02c-7cc5e9c7c8ac.service` was used with
mountpoint `/home/uutzinger/Cloud/cosmic-mounter-onedriver-online-test`. A
temporary process held the mount busy, producing the expected clean-unmount
failure from onedriver: `fusermount3: failed to unmount ... Device or resource
busy`. The applet popup showed the Repair confirmation path; the first click
armed repair and the second click displayed the `fusermount3` recovery message.
After cleanup, `findmnt -rn -t fuse.rclone,fuse.onedriver,fuse3.onedriver`
reported no FUSE mounts and the generated service was `inactive (dead)` rather
than failed.

**Gate 2 remaining item:** user approval of the current release candidate.

## Version 0.3 Planning: Local Path Selection

### Version 0.3 Folder Picker

**Folder picker implementation:** June 23, 2026. The Add/Modify Connection
window now keeps the existing local target text field and adds a `Browse`
button for both Online Mount mountpoints and Offline Mirror directories. The
implementation enables libcosmic's `xdg-portal` feature and uses
`cosmic::dialog::file_chooser` to request a folder from the user's desktop
session. The selected local filesystem path is written back into the same draft
`local_path` field used by manual entry, so existing duplicate-target,
nested-path, unsafe-path, and mount/mirror overlap validation remains the
single enforcement point before Test Connection or Save Connection.

**Folder picker verification:** June 23, 2026. Automated verification passed
with `cargo fmt --all -- --check`, `cargo check --all-targets`, focused test
`cargo test --all-targets selected_folder_path_reuses_existing_local_target_validation`,
and full `cargo test --all-targets`. The full test suite passed with 89 library
tests and 52 application tests. The focused test confirms a folder selected
under an existing configured local target is rejected by the existing overlap
validation path.

**User build/install packaging:** June 23, 2026. User ran `just install-user`
and `just deb` after the folder picker change. This confirms the updated applet
was installed into the user session and the Debian package target was
exercised. Manual visual verification remains open for the folder chooser
itself, specifically whether the session's portal chooser exposes a
create-folder affordance.

**Version 0.3 release package build:** June 23, 2026. Version metadata was
updated to 0.3.0 in Cargo, Debian changelog, AppStream release metadata, and
README package examples. `just metadata-check` passed with
`desktop-file-validate` and `appstreamcli validate --pedantic --no-net`.
`just deb` built `../cosmic-ext-applet-mounter_0.3.0_amd64.deb`; package
metadata reports `Package: cosmic-ext-applet-mounter` and `Version: 0.3.0`.
Debian build scratch paths under `debian/` were added to `.gitignore` so repeat
package builds do not appear as untracked source changes.

### Version 0.3 Popup Runtime Status

**Popup notice and VPN runtime status update:** June 23, 2026. The popup header
no longer reports VPN dependency presence as though the VPN is active. It now
uses runtime readiness for configured VPN dependencies and displays active or
inactive state. Transient popup messages above Add Connection/Refresh now clear
automatically after 10 seconds, and successful Add/Modify settings launches no
longer leave stale "Modify connection selected" style messages in the main
popup.

**Cisco VPN parsing repair:** June 23, 2026. Cisco Secure Client status parsing
now reads the exact `Connection State:` field. This prevents disconnected
states from being misclassified as connected because the word `Disconnected`
contains the substring `connected`. Regression coverage was added for
`Connected`, `Disconnected`, `Not Available`, and `Cannot contact the VPN
service` status output.

**Non-blocking VPN status follow-up:** June 23, 2026. The first runtime VPN
status implementation synchronously probed NetworkManager/Cisco during popup
view construction, which could delay opening the applet. The popup now opens
with `VPN status: checking` when configured VPN dependencies exist and performs
NetworkManager/Cisco readiness checks asynchronously. Refresh also reloads
configuration immediately and updates VPN readiness when the async check
finishes. Manual installed-applet verification remains open for the exact
Cisco-configured-but-disconnected case.

## Version 0.3 COSMIC Flatpak Repository Publication

### Publication Suitability and Architecture Gate

**Architecture decision:** July 8, 2026. Native source and Debian installations
remain the primary execution model. The existing `just deb` packaging path is
kept as the low-friction package for users who need direct host integration.
Flatpak is treated as an additional distribution target for COSMIC repository
publication, not as a replacement for the native package.

**Host-integration requirements documented:** July 8, 2026. The Flatpak
architecture gate now records that the applet depends on host executable
discovery, host user systemd unit creation/control, host-visible FUSE mounts,
host NetworkManager/Cisco VPN state, existing rclone configuration, provider
authentication state, and user-selected mount, mirror, cache, and recovery
directories. These requirements are incompatible with a sandbox-only storage
manager.

**Selected Flatpak execution model:** July 8, 2026. The approved direction is
to add a Flatpak runtime mode behind the existing typed command-runner boundary.
Native installations continue to execute fixed approved commands directly. The
Flatpak mode shall route approved host commands through
`flatpak-spawn --host`, preserving separate validated arguments, no shell
concatenation, bounded output, timeouts, cancellation behavior, and redaction.

**Configuration ownership decision:** July 8, 2026. General applet settings
remain applet configuration, but Flatpak must not make that configuration
sandbox-private for normal operation. Users switching from source or Debian
installs to Flatpak should see the same saved connections instead of recreating
them. Provider-owned state also remains host-owned: existing rclone remotes and
credentials, `jstaf/onedriver` state, `abraunegg/onedrive` state, generated
user systemd units/timers, and selected storage paths must not be silently
copied into a second Flatpak-private credential or connection store. The likely
implementation is a narrow host-side helper/bridge for applet configuration and
unit writes, preserving the existing typed-operation safety model.

**Publication rejection rule:** July 8, 2026. Flatpak publication shall be
rejected or postponed if prototype testing shows that the package cannot expose
mounts to ordinary host applications, cannot manage the intended host user
services, silently uses different rclone or OneDrive credentials than the
native applet, or requires unjustified unrestricted host access.

**Requirements update:** July 8, 2026. `Requirements and Specifications.md`
now includes the selected Flatpak execution architecture and security tradeoffs
in section 9.4. Implementation of the Flatpak command runner and live
`flatpak-spawn --host` verification remain open tasks.

**Flatpak host-runner prototype:** July 8, 2026. Branch
`flatpak-host-runner` adds the initial command-runner support for Flatpak host
execution. `src/process.rs` now has `FlatpakHostCommandRunner`, which transforms
typed requests such as `rclone version` into
`flatpak-spawn --host rclone version` while preserving separate validated
arguments, retry policy, timeout, output limit, cancellation, and redaction.
`RuntimeCommandRunner` can select either the unchanged native
`SystemCommandRunner` path or the Flatpak host-spawn path. The applet is not
yet wired to use this runtime selector, so native `just install-user` and
`just deb` behavior remains unchanged.

**Prototype verification:** July 8, 2026. Added unit tests for Flatpak command
wrapping, sensitive argument redaction, and native command shape preservation.
Verification passed with `cargo fmt --all -- --check`,
`cargo check --all-targets`, and `cargo test --all-targets` on the
`flatpak-host-runner` branch. `flatpak-spawn` is not present on the current host
PATH outside a Flatpak sandbox, so live `flatpak-spawn --host` execution remains
open until a local Flatpak manifest/build exists.

**Probe-only Flatpak manifest:** July 8, 2026. Added
`packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json`
and `packaging/flatpak/README.md`. This is a prototype manifest only; it copies
the locally built `flatpak_host_runner_probe` example into a minimal Flatpak and
is not the final COSMIC repository submission. The probe app ID is
`io.github.uutzinger.cosmic-ext-applet-mounter-probe`; the first attempted ID
with an extra segment after `cosmic-ext-applet-mounter` was rejected by
Flatpak's app ID rules. The final applet ID
`io.github.uutzinger.cosmic-ext-applet-mounter` remains structurally valid
because its hyphens are in the final segment.

**Live Flatpak host-spawn verification:** July 8, 2026. Built and installed the
probe with `flatpak-builder --force-clean --user --install
target/flatpak-host-runner-probe
packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json`.
The manifest grants only `--talk-name=org.freedesktop.Flatpak`. Running
`flatpak run io.github.uutzinger.cosmic-ext-applet-mounter-probe` passed. The
probe verified that `DependencyInventory` can detect host rclone 1.74.3,
onedriver 0.15.0, onedrive 2.5.10, FUSE 3.14.0, NetworkManager 1.46.0, Cisco
Secure Client components, user systemd, and fuser through
`flatpak-spawn --host`.

**Core host command verification:** July 8, 2026. The same Flatpak probe
successfully ran `rclone version`, `nmcli general status`,
`systemctl --user --version`, and `fusermount3 --version` through
`flatpak-spawn --host`. Native probe mode also passed outside the Codex sandbox.
Running native `nmcli` inside the Codex command sandbox failed with
`Operation not permitted`, confirming that NetworkManager checks require the
normal user session rather than the restricted Codex sandbox. Remaining live
host-spawn checks: nonzero exit status, stderr capture, timeout behavior, and
cancellation behavior.

**Stage 1 host-runner behavior verification:** July 8, 2026. Extended the
Flatpak host-runner probe with expected-error cases and reran it inside the
installed probe Flatpak. `flatpak-spawn --host` correctly propagated nonzero
exit status using `false`, stderr capture using `cat` on a nonexistent path,
timeout behavior using `sleep 5` with a short timeout, and cancellation
behavior using `sleep 5` with a cancellation token. Native probe mode passed
the same behavior checks outside the Codex sandbox. This completes the command
behavior portion of the Flatpak host-runner verification; FUSE visibility and
host user-systemd service behavior remain open.

**FUSE and user-systemd host-visibility verification:** July 8, 2026. Extended
the probe with opt-in `--fuse` mode. The probe creates disposable host paths
under `/tmp`, starts a transient user systemd unit named
`cosmic-mounter-flatpak-probe.service`, runs host `rclone mount` against a
local source directory, verifies the mount with host `mountpoint` and `findmnt`,
lists a probe file through the mounted FUSE filesystem, then stops the unit and
removes the disposable paths. Native mode passed first. The installed Flatpak
probe then passed the same test through `flatpak-spawn --host` using only
`--talk-name=org.freedesktop.Flatpak`. The resulting mount was reported as
`fuse.rclone` by host `findmnt`, demonstrating that the mount is created in the
host namespace rather than trapped in the Flatpak sandbox. Post-run cleanup
verification confirmed `mountpoint` returned inactive, the transient unit was
inactive, and the disposable source/mount paths no longer existed.

**GUI prototype Flatpak smoke test:** July 8, 2026. Added
`packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json`
as a copy-based GUI prototype manifest for the real applet binary. This is not
the final reproducible COSMIC submission manifest. The applet runtime command
paths now use `RuntimeCommandRunner::detect_current()` so direct host commands
are routed through `flatpak-spawn --host` when the applet runs inside Flatpak,
while native source and Debian installs keep direct execution.

**GUI prototype permissions:** July 8, 2026. The prototype was built and
installed with `flatpak-builder --force-clean --user --install
target/flatpak-gui-prototype
packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json`.
The installed prototype exposes Wayland, fallback X11, IPC, DRI, and
`org.freedesktop.Flatpak` session-bus access. Flatpak also added read-only
theme/config access for GTK/KDE color settings. The settings window launched
with `flatpak run io.github.uutzinger.cosmic-ext-applet-mounter --settings`
without immediate error and exited cleanly.

**User GUI verification:** July 8, 2026. User confirmed the Flatpak settings
window behaved normally. Detect rclone remotes showed existing rclone
configuration, and Detect VPNs found the existing VPN choices. User observed
that the prototype used a dark theme instead of the system-default light theme;
theme matching remains an open Flatpak polish/minimum-permissions item. User
did not create or save a new connection during this smoke test.

**Flatpak configuration visibility finding:** July 8, 2026. Launching the
prototype with `--modify-connection 260b9a2f-3409-48e6-8120-308b43b9fa04`
opened the Modify workflow, but the app reported "Connection is no longer
available." The native connection exists in
`~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document`,
while the Flatpak runtime currently uses sandboxed app configuration under
`~/.var/app/io.github.uutzinger.cosmic-ext-applet-mounter`. This means host
command execution through `flatpak-spawn --host` is working, but configuration,
generated unit files, and app-owned engine metadata need a separate Flatpak
state/access decision before Modify/Save flows can be considered verified.
The local `../kdeconnect` reference confirms the same class of issue: code that
uses `dirs::config_dir()` or `cosmic_config` inside Flatpak sees sandbox paths
unless it explicitly reads host-visible `~/.config/cosmic` state via `HOME` or a
bridge.

**Host-visible applet state bridge prototype:** July 8, 2026. Added
`AppConfigStorage` and `HostVisibleConfigStorage`. Native/source/Debian mode
continues using `cosmic_config::Config`; Flatpak mode detects `/.flatpak-info`
and reads/writes the native-visible applet document at
`~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document`.
The GUI prototype manifest now grants only that applet-specific COSMIC config
path with `--filesystem=xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter:create`.
Automated tests cover host-visible document loading and atomic writeback. After
rebuilding and reinstalling the GUI prototype, launching
`flatpak run io.github.uutzinger.cosmic-ext-applet-mounter --modify-connection
260b9a2f-3409-48e6-8120-308b43b9fa04` opened the native saved `UA Box`
connection. User ran Test Connection successfully: the app reported that the
managed online mount unit validates structurally, rclone remote `ua_box` exists,
backend `box` matches Box, and `ua_box:` is accessible with one visible item at
depth 1. The theme mismatch remains open; the prototype still appears dark when
the system default is light.

**Flatpak Save-path and host user-unit verification:** July 8, 2026. Extended
Flatpak mode so durable app-owned roots use host-visible paths:
`~/.config/cosmic-ext-applet-mounter`, `~/.cache/cosmic-ext-applet-mounter`,
and `~/.local/state/cosmic-ext-applet-mounter`. The GUI prototype manifest now
also grants narrow access to those app-specific paths plus
`~/.config/systemd/user`. User opened the Flatpak Modify window for native
saved `UA Box` and clicked Save Connection. The app reported: "UA Box saved and
managed mount unit installed. Start at login is disabled; the unit was not
started." Shell verification showed the host unit file
`~/.config/systemd/user/cosmic-mounter-260b9a2f-3409-48e6-8120-308b43b9fa04.service`
was updated, loaded by the host user systemd manager, disabled, and inactive.
The generated unit contains applet ownership markers, host `/usr/bin/rclone`,
host mountpoint `/home/uutzinger/Cloud/UA_Box`, and host cache directory
`/home/uutzinger/.cache/cosmic-ext-applet-mounter/rclone/260b9a2f-3409-48e6-8120-308b43b9fa04`.
This verifies the Flatpak Save path for a non-destructive rclone Online Mount.

**Flatpak host COSMIC theme verification:** July 8, 2026. The prototype
settings window initially used a dark theme because the sandboxed runtime did
not inherit the host COSMIC theme correctly. Diagnostic runs confirmed the
Flatpak could read the host files under `~/.config/cosmic`, but constructing a
theme from the built-in light palette plus accent did not match the native
surface colors. The loader now reads the full host COSMIC theme through
`cosmic_config::Config::with_custom_path` using the host-visible
`~/.config/cosmic/com.system76.CosmicTheme.{Light,Dark}/v1` directories, with a
built-in light/dark plus accent fallback. User confirmed the new Flatpak
settings window uses light mode and is much closer to the native applet theme.
Theme changes are applied when the settings window starts; live theme-change
watching remains a possible later polish item if needed.

**Flatpak rclone Box host-unit and FUSE verification:** July 8, 2026. Verified
the saved `UA Box` connection generated by the Flatpak prototype through the
host-visible configuration bridge. Before the test,
`cosmic-mounter-260b9a2f-3409-48e6-8120-308b43b9fa04.service` was loaded by the
host user systemd manager, disabled, inactive, and the local path
`/home/uutzinger/Cloud/UA_Box` was not a mountpoint. Starting the unit with
host `systemctl --user start` succeeded. Follow-up checks showed
`ActiveState=active`, `SubState=running`, a nonzero `MainPID`, and
`mountpoint -q /home/uutzinger/Cloud/UA_Box` returned success. `findmnt`
reported `/home/uutzinger/Cloud/UA_Box` as source `ua_box:` with filesystem
type `fuse.rclone`, and a normal host `ls` could list the mounted Box content.
Stopping the unit with host `systemctl --user stop` succeeded; the unit returned
to `ActiveState=inactive`, `SubState=dead`, `MainPID=0`, and the path was no
longer a mountpoint. This verifies the Flatpak-generated host user unit and
host-visible FUSE behavior for a Box Online Mount. The applet-popup toggle path
for the same operation remains open for separate UI-driven verification.

**Flatpak rclone Google Drive host-unit and FUSE verification:** July 8, 2026.
Verified the saved `uutzinger Google Drive` connection
`88abbf95-0372-44ab-b2a5-e90847060b2c` through the Flatpak-generated host unit.
Before the test, the unit was loaded, disabled, inactive, and
`/home/uutzinger/Cloud/uutzinger_GoogleDrive` was not a mountpoint. Starting
`cosmic-mounter-88abbf95-0372-44ab-b2a5-e90847060b2c.service` with host
`systemctl --user start` succeeded. The unit entered `ActiveState=active`,
`SubState=running`, and `findmnt` reported
`/home/uutzinger/Cloud/uutzinger_GoogleDrive` as source `uutzinger_gdrive:`
with filesystem type `fuse.rclone`. A normal host `ls` could list the mounted
Google Drive content. Stopping the unit succeeded, returning the service to
`ActiveState=inactive`, `SubState=dead`, `MainPID=0`; the path was no longer a
mountpoint. This verifies host-visible FUSE behavior and clean start/stop for a
Google Drive Online Mount generated from the Flatpak-visible configuration.

**Flatpak popup-toggle Box Online Mount verification:** July 8, 2026. With the
Flatpak applet runtime running, user toggled `UA Box` on from the popup. Host
verification showed
`cosmic-mounter-260b9a2f-3409-48e6-8120-308b43b9fa04.service` entered
`ActiveState=active`, `SubState=running`, `MainPID` was nonzero, and
`/home/uutzinger/Cloud/UA_Box` became a mountpoint. `findmnt` reported the
mount as source `ua_box:` with filesystem type `fuse.rclone`, proving the popup
operation used the host-visible FUSE path. User then toggled `UA Box` off from
the popup. Host verification showed the unit returned to
`ActiveState=inactive`, `SubState=dead`, `MainPID=0`, and the path was no
longer a mountpoint. This completes popup-driven mount/unmount verification for
the Box Online Mount path under the Flatpak prototype.

**Flatpak popup-toggle Google Drive Online Mount verification:** July 8, 2026.
With the Flatpak applet runtime running, user toggled `uutzinger Google Drive`
on from the popup. Host verification showed
`cosmic-mounter-88abbf95-0372-44ab-b2a5-e90847060b2c.service` entered
`ActiveState=active`, `SubState=running`, `MainPID` was nonzero, and
`/home/uutzinger/Cloud/uutzinger_GoogleDrive` became a mountpoint. `findmnt`
reported the mount as source `uutzinger_gdrive:` with filesystem type
`fuse.rclone`. User toggled the connection off from the popup. Host
verification showed the unit returned to `ActiveState=inactive`,
`SubState=dead`, `MainPID=0`, and the path was no longer a mountpoint. This
completes popup-driven mount/unmount verification for the personal Google Drive
Online Mount path under the Flatpak prototype.

**Flatpak popup-toggle SMB Online Mount with Cisco VPN verification:** July 8,
2026. User started Cisco VPN, toggled `UA Engineering Research storage` from
the Flatpak applet popup, checked the mounted content, and confirmed the
expected files were visible. User then closed/disconnected the VPN and
unmounted the drive from the applet. Follow-up host verification showed
`cosmic-mounter-740cb417-10a1-4afd-a46a-8569c4e0d3e1.service` was
`ActiveState=inactive`, `SubState=dead`, `MainPID=0`, and
`/home/uutzinger/Cloud/UA_ENGR` was no longer a mountpoint. This verifies the
VPN-gated SMB Online Mount popup path at the user-observed content level and
confirms clean post-test detachment.
OneDrive-specific metadata paths still need separate live verification.

**Flatpak Box Offline Mirror initial preview correction:** July 8, 2026. Added
a disposable Box Offline Mirror test connection for the Flatpak prototype. The
first selected remote subtree, `ua_box:Utzinger/cosmic-mounter-ui-test`, failed
because the path was ambiguous/not found. The test connection was updated to
the verified disposable subtree
`ua_box:cosmic-mounter-live-verify/offline-mirror-20260619-codex`. Preview then
failed because rclone bisync `--check-access` requires an `RCLONE_TEST`
sentinel file on both Path1 and Path2, which is incompatible with ordinary
first-time empty local mirrors unless the applet creates sentinel files in the
user's cloud storage. A manual bounded dry-run using `--resync --dry-run`
without `--check-access` succeeded and reported five remote files that would be
copied to the local mirror. The applet now relies on its existing read-only
provider setup/access validation and no longer adds rclone bisync
`--check-access` to generated mirror commands. Focused tests
`rclone_bisync_plan_preserves_conflicts_recovery_and_schedule` and
`rclone_preview_request_is_dry_run_and_bounded` passed. The native user install
and local Flatpak GUI prototype were rebuilt so the next UI Preview test uses
the corrected command. User then re-ran Preview from the Flatpak Modify
Connection window. Preview completed successfully for `Disposable Box Offline
Mirror Test` with uploads 0, downloads 1, deletes 0, conflicts 0, skipped 1,
and transfer estimate 208 bytes. This confirms the initial preview path works
for an empty local mirror without requiring a preexisting local sentinel file.
User then clicked Sync Now for the same disposable mirror. The app reported
uploads 0, downloads 1, deletes 0, conflicts 0, skipped 0, and recorded initial
synchronization as complete so future Sync Now runs use normal bisync. Host
verification showed `/tmp/cosmic-mounter-box-mirror` contains the expected
files `RCLONE_TEST`, `local-followup.txt`, `local-seed.txt`,
`remote-followup.txt`, and `remote-seed.txt`. The work directory contains
`initial-sync-complete`, and the host user systemd service
`cosmic-mounter-bb69e234-cf3e-4e63-8592-2601a93d604b.service` is loaded,
disabled, and inactive after the manual sync.

**Flatpak Google Drive Offline Mirror preview verification:** July 8, 2026.
Created disposable Google Drive subtree
`uutzinger_gdrive:cosmic-mounter-ui-test` with `RCLONE_TEST` and
`remote-seed.txt`, then added the `Disposable Google Drive Offline Mirror Test`
connection through the controlled setup helper. The generated service is
`cosmic-mounter-4e30dc23-c887-4704-bb98-41c5dfaf6467.service`, the timer is
`cosmic-mounter-4e30dc23-c887-4704-bb98-41c5dfaf6467.timer`, the local mirror is
`/tmp/cosmic-mounter-gdrive-mirror`, and the work directory is
`/home/uutzinger/.local/state/cosmic-ext-applet-mounter/rclone-bisync/4e30dc23-c887-4704-bb98-41c5dfaf6467`.
User ran Preview from the Flatpak Modify Connection window. Preview completed
successfully with uploads 0, downloads 1, deletes 0, conflicts 0, skipped 1,
and transfer estimate 92 bytes. This confirms the Google Drive initial preview
path works for an empty local mirror without requiring a preexisting local
sentinel file.
User then clicked Sync Now for the same disposable mirror. The app reported
uploads 0, downloads 1, deletes 0, conflicts 0, skipped 0, and recorded initial
synchronization as complete so future Sync Now runs use normal bisync. Host
verification showed `/tmp/cosmic-mounter-gdrive-mirror` contains the expected
files `RCLONE_TEST` and `remote-seed.txt`. The work directory contains
`initial-sync-complete`, and the host user systemd service
`cosmic-mounter-4e30dc23-c887-4704-bb98-41c5dfaf6467.service` is loaded,
disabled, and inactive after the manual sync.

**Flatpak SMB Offline Mirror preview and initial sync verification:** July 8,
2026. With Cisco VPN connected, verified the disposable SMB subtree
`ua_engr:Research/Utzinger/cosmic-mounter-ui-test` was reachable and contained
`RCLONE_TEST`. Added the `Disposable SMB Offline Mirror Test` connection
through the controlled setup helper. The generated service is
`cosmic-mounter-9e6d9640-9c99-48ef-86c1-b3e91d8dc146.service`, the timer is
`cosmic-mounter-9e6d9640-9c99-48ef-86c1-b3e91d8dc146.timer`, the local mirror is
`/tmp/cosmic-mounter-smb-mirror`, and the work directory is
`/home/uutzinger/.local/state/cosmic-ext-applet-mounter/rclone-bisync/9e6d9640-9c99-48ef-86c1-b3e91d8dc146`.
User ran the Flatpak UI preview/sync path and reported Sync Now completed with
uploads 0, downloads 1, deletes 0, conflicts 0, skipped 0, and initial
synchronization recorded as complete. Host verification showed
`/tmp/cosmic-mounter-smb-mirror` contains `RCLONE_TEST`, the work directory
contains `initial-sync-complete`, and the host user systemd service is loaded,
disabled, and inactive after the manual sync.

**Flatpak host user systemd visibility rollup:** July 8, 2026. Verified the
tested Flatpak-generated units are visible to the host user systemd manager and
loaded from host `~/.config/systemd/user`: `UA Box`,
`uutzinger Google Drive`, `UA Engineering Research storage`, and disposable
Box, Google Drive, and SMB Offline Mirror services. The three disposable
offline mirror timers are also loaded from the host user systemd directory. All
checked services and timers were disabled and inactive after verification,
which matches the manual-start/manual-sync test state.

**Flatpak FUSE visibility rollup:** July 8, 2026. Box and personal Google
Drive Online Mounts created from Flatpak-generated host user units were visible
to ordinary host processes at `/home/uutzinger/Cloud/UA_Box` and
`/home/uutzinger/Cloud/uutzinger_GoogleDrive` as `fuse.rclone` mounts. User
also verified the Cisco-VPN-gated SMB Online Mount displayed the expected
content from the host-visible mount location before unmounting. This completes
the current FUSE namespace check for the tested rclone online mount providers:
mounts are created in the host user session and are not trapped inside the
Flatpak sandbox.

**Flatpak provider-owned state check:** July 8, 2026. Inspected
`~/.var/app/io.github.uutzinger.cosmic-ext-applet-mounter` after Flatpak
prototype testing. The sandbox-private area contained runtime/font/theme/cache
artifacts such as `fontconfig` caches and `kdeglobals`, but no rclone config,
onedriver authentication state, abraunegg/onedrive refresh tokens,
NetworkManager/Cisco profiles, or generated user systemd units. App-owned
durable roots are present in host-visible locations under
`~/.config/cosmic-ext-applet-mounter`, `~/.cache/cosmic-ext-applet-mounter`,
and `~/.local/state/cosmic-ext-applet-mounter`, matching the selected
host-visible state model.

**Native/direct code path preservation:** July 8, 2026. Focused tests confirmed
the Flatpak host-runner and host-visible configuration paths are opt-in runtime
modes rather than replacing native behavior. Passing tests included
`runtime_runner_can_be_selected_without_changing_native_default`,
`flatpak_host_runner_wraps_fixed_executable_and_arguments`,
`flatpak_host_runner_preserves_sensitive_redaction`,
`host_visible_storage_loads_native_cosmic_document`,
`host_visible_storage_writes_validated_document_atomically`, and
`flatpak_durable_roots_use_host_visible_home_paths`. This preserves the direct
command/config path expected by source and Debian installs while allowing the
Flatpak runtime to select host-spawn and host-visible state paths.

**Flatpak data-integrity safety recheck:** July 8, 2026. Re-ran focused
automated safety tests for overlapping mount/mirror targets, unsafe local
targets, cache/recovery overlap, unsafe rclone remote values, active onedriver
overlap with OneDrive Offline Mirror, and Add/Modify UI validation. Passing
tests included `duplicate_and_nested_targets_are_rejected_across_modes`,
`unsafe_relative_and_system_targets_are_rejected`,
`cache_and_recovery_directories_cannot_overlap_visible_tree`,
`rclone_plan_rejects_overlap_and_unsafe_remote_values`,
`onedrive_plan_blocks_active_onedriver_overlap`,
`add_validation_rejects_duplicate_name_and_local_overlap`,
`modify_validation_rejects_duplicate_name_and_local_overlap`, and
`onedrive_offline_validation_reports_onedriver_overlap`.

**Flatpak VPN parser/readiness automated recheck:** July 8, 2026. Re-ran
focused VPN tests after Flatpak host-runner changes. Passing tests included
`runtime_cisco_tunnel_state_reads_exact_connection_state`,
`runtime_systemd_status_parser_recognizes_active_disabled_units`,
`vpn_import_dedupes_by_backend_reference`,
`app_nmcli_parser_recovers_flattened_applet_output`,
`network_manager_list_profiles_falls_back_to_name_uuid_type_output`,
`parsers_handle_nmcli_escaping_cisco_connected_and_access_modes`, and
`activation_and_shutdown_decisions_respect_readiness_and_ownership`. Live
Flatpak popup observation of asynchronous VPN status and activation/ownership
behavior remains open.

**Flatpak popup VPN status live check:** July 8, 2026. Host checks showed
NetworkManager had `cscotun0:tun:activated` and Cisco Secure Client reported
`Connected to Engineering SSL VPN`. User opened/refreshed the Flatpak applet
popup and confirmed it displayed the expected active Cisco VPN status. Live
activation-readiness and disconnect-only-if-activated behavior remains open.

**Flatpak OneDrive app-owned metadata path verification:** July 8, 2026.
Verified that OneDrive app-owned metadata is stored in host-visible applet roots
rather than Flatpak sandbox-private state. Existing `jstaf/onedriver` metadata
appears under `~/.config/cosmic-ext-applet-mounter/onedriver` and
`~/.cache/cosmic-ext-applet-mounter/onedriver`; existing
`abraunegg/onedrive` metadata appears under
`~/.config/cosmic-ext-applet-mounter/onedrive-sync` and recovery state under
`~/.local/state/cosmic-ext-applet-mounter/onedrive-recovery`. A scan of
`~/.var/app/io.github.uutzinger.cosmic-ext-applet-mounter` found no copied
onedriver, onedrive, or rclone credential/config stores. Focused tests also
passed for isolated onedriver config/cache planning, auth-only request
construction, authenticated onedriver metadata validation, OneDrive mirror
interactive setup validation, OneDrive manual auth-files setup validation, and
transient auth handoff file behavior. Full live OneDrive Flatpak setup/mount
and mirror authentication flows remain separate open verification items.

**Flatpak metered/network readiness policy recheck:** July 8, 2026. Re-ran
focused automated tests for offline mirror preview confirmation, metered
network pause/override behavior, VPN/network readiness blocking, restored
offline sync and metered states, rclone status composition across readiness,
service, mount, and pending-write states, and common offline status mappings.
Passing tests included `sync_decision_requires_preview_confirmation_and_metered_override`,
`vpn_and_network_readiness_block_operations`,
`restores_offline_sync_and_metered_states`,
`rclone_status_combines_readiness_service_mount_and_writes`, and
`offline_status_maps_common_states`. Live network-loss disruption was not
performed in this pass to avoid interrupting the user's active network/VPN
session.

**Flatpak sandbox limitations and behavior differences captured:** July 8,
2026. Current GUI prototype permissions include Wayland/X11 fallback, IPC, DRI,
`org.freedesktop.Flatpak` session-bus talk access for `flatpak-spawn --host`,
narrow host-visible applet config/cache/state grants, `~/.config/systemd/user`
for generated user units, and read-only host theme/config grants for COSMIC,
GTK, KDE, and color-scheme data. This differs from native/source/Debian
execution in three important ways: host commands are mediated through
`flatpak-spawn --host`; applet-owned state must be explicitly mapped to
host-visible locations to avoid duplicate sandbox-private configuration; and
standalone settings windows need explicit COSMIC theme loading to match host
light/dark mode. Verified behavior so far shows rclone remotes, generated user
systemd units, app-owned metadata, and FUSE mounts stay host-visible. Remaining
known open live checks are OneDrive Flatpak setup/mount/mirror flows and popup
VPN async activation/readiness behavior.

**Flatpak VPN activation live failure and fix:** July 9, 2026. User verified
that VPN disconnect status is reported correctly, but a Cisco-dependent storage
connection only reported that Cisco VPN was not running and did not open Cisco
Secure Client when toggled on. The applet also did not run disconnect ownership
handling after unmount. Code inspection confirmed that the controller disabled
Mount while an online mount was in `WaitingForVpn`, and the managed mount
operation only started/stopped the systemd unit without invoking VPN
activation or shutdown policy. The controller now allows Mount from
`WaitingForVpn`, and managed online mount operations call VPN readiness
activation before starting the mount unit. NetworkManager profiles are
activated with `nmcli connection up uuid`; Cisco profiles attempt to start the
agent if needed, open Cisco Secure Client, and poll readiness until the profile
timeout. Runtime applet-activated VPN IDs are stored under app-owned state so
unmount can disconnect only a VPN the applet activated and only after no other
active connection still needs it. Focused checks passed:
`vpn_and_network_readiness_block_operations`,
`activation_and_shutdown_decisions_respect_readiness_and_ownership`, and
`cargo check --all-targets`. Native and Flatpak GUI prototype builds were
reinstalled for retesting.

**Flatpak VPN activation live verification:** July 9, 2026. User toggled a
Cisco-VPN-dependent SMB Online Mount from the prototype Flatpak popup while
Cisco was disconnected. The applet opened/started Cisco Secure Client, allowed
the user to connect interactively, then completed the SMB mount. Toggling the
connection off unmounted the SMB share and stopped the VPN connection. This
verifies the prototype Flatpak popup path for Cisco activation readiness and
disconnect-only-if-applet-activated behavior.

**Flatpak Add/Modify window tooltip smoke recheck:** July 9, 2026. User found
overlapping tooltips in the Flatpak settings window: Browse showed both the
Browse tooltip and mountpoint safety tooltip, and Modify Connection showed both
the general provider/access-mode tooltip and the locked-control tooltip. The UI
was changed so Modify Connection uses only locked-control tooltips for provider
and access-mode buttons, Add Connection retains the general provider/mode
guidance, and the mountpoint safety warning is attached only to the path text
input while Browse has its own bottom-positioned tooltip. After reinstalling the
native build and prototype Flatpak, the user confirmed the tooltip behavior now
looks correct.

**SMB password handling implementation:** July 11, 2026. Replaced the
shell-copy/paste SMB password workaround with an applet-managed password update
path. Add/Modify Connection now shows SMB host, user, domain, and a masked
transient password field for SMB connections. The password field is intentionally
blank when modifying an existing connection; leaving it blank preserves the
current rclone password, while entering a value runs
`rclone config password <remote> pass <secret>` through the command runner with
the secret marked sensitive and redacted from recorded command strings. On
success, the password is cleared from the draft and is not saved in applet
configuration. Focused tests passed for SMB redacted request construction and
SMB create/update command sequencing.

**SMB Modify hydration fix:** July 11, 2026. User found that reopening an SMB
connection showed stale/default SMB metadata such as blank host, local username,
and WORKGROUP even after the rclone remote had been updated. Modify Connection
now loads SMB host, username, and domain from the selected rclone remote via
`rclone config dump`; the password field remains blank and is never hydrated
from rclone. If the remote cannot be read, Modify mode leaves the SMB fields
blank rather than showing fabricated defaults. Added a parser test for SMB
remote detail hydration that ignores secret fields.

**VPN shutdown state recheck:** July 11, 2026. User reported that toggling an
SMB storage connection off did not disconnect Cisco VPN even though the user
believed the applet initiated it. Inspection showed the primary SMB connection
has `disconnect_vpn_when_unused: true`, while the disposable SMB test connection
has it disabled. The applet activation marker still contained the Cisco VPN id
even though Cisco was currently disconnected, so popup/global VPN status refresh
now clears stale applet-activated markers whenever the referenced VPN is not
ready. This prevents a stale marker from being carried across manual disconnects
or later tests. Native and Flatpak prototype builds were reinstalled.

**OneDrive Offline Mirror auth/preview timeout finding:** July 11, 2026. User
reported that OneDrive Online Mount authentication worked, but OneDrive Offline
Mirror setup gave no clear feedback after browser/WebKit authorization and then
timed out during dry-run preview. Inspection showed two new app-owned
`abraunegg/onedrive` `refresh_token` files were created, so authentication was
likely completing and the failure point was the post-auth
`onedrive --sync --dry-run --verbose` validation. The WebKitGTK helper remains
present and installed; it is used by the Manual Auth Handoff path, while the
primary Start OneDrive Mirror Setup path still uses `onedrive --reauth`
directly. The OneDrive dry-run preview timeout was increased from 120 seconds
to 10 minutes, setup failures now explicitly distinguish completed
authorization from post-auth validation failure, and the helper success dialog
now states that Cloud Mounter will continue with a potentially long dry-run
validation.

**OneDrive Offline Mirror setup/save/sync UX correction:** July 11, 2026. User reported that OneDrive browser authorization completed but the applet gave no intermediate indication that post-auth validation had started, Save Connection repeated the same long dry-run validation, and Sync Now failed with misleading rclone wording plus an abraunegg/onedrive resync-required message. The editor now explains that post-auth validation can take several minutes, recommends Manual Auth Handoff only after setup failure, caches successful setup/Test Connection validation for the unchanged draft so Save can install the unit without repeating the dry-run, and labels OneDrive preview/sync failures as onedrive failures. Initial OneDrive Offline Mirror Sync Now now runs onedrive with --resync after a successful Preview, records initial sync completion, and leaves later Sync Now runs on normal onedrive --sync. Verification passed with cargo fmt --all -- --check after formatting, cargo test onedrive, cargo test onedrive_plan_uses_isolated_config_and_monitor_mode, cargo check --all-targets, and just install-user. Live Flatpak recheck remains open.

**OneDrive initial resync noninteractive confirmation fix:** July 11, 2026. Live OneDrive Offline Mirror initial Sync Now reached abraunegg/onedrive --resync but failed because onedrive prompts for confirmation unless --resync-auth is supplied. Since the applet already requires Preview plus explicit Sync Now before initial sync, the initial OneDrive sync command now includes both --resync and --resync-auth. Added a command-construction assertion for --resync-auth.

**SMB Online Mount timeout correction:** July 11, 2026. User reported that the same corporate SMB path worked through GVFS but applet/rclone mount returned I/O errors when opening the writable folder. Read-only rclone checks against ua_engr showed ua_engr:Research/Utzinger is accessible, but listing it took about 46 seconds. The applet-generated rclone online mount used --timeout 10s and --contimeout 5s for all providers, which is too aggressive for this SMB share. SMB rclone online mounts now use --timeout 90s and --contimeout 15s, while Google Drive and Box keep the shorter cloud-mount defaults. Focused tests rclone_mount_plan_uses_provider_remote_and_bounded_defaults and rclone_mount_plan_uses_longer_smb_timeouts passed, along with cargo fmt --all -- --check and cargo check --all-targets. Native and Flatpak prototype builds were reinstalled; existing SMB unit files need Save Connection or regeneration before they receive the longer timeout.

**Connection and rclone remote removal confirmation UI:** July 11, 2026. User confirmed that rclone remote removal and whole-connection removal completed successfully, but requested clearer parity between the two workflows. Modify Connection removal now changes from Remove to Confirm Remove after the first click and then to a disabled/shaded Removing... state while generated-unit cleanup runs. Rclone remote management now similarly changes from Remove remote to Confirm Remove and then to disabled Removing... while rclone configuration is updated. Verification passed with cargo fmt --all -- --check and cargo check --all-targets; native and Flatpak prototype builds were reinstalled.

**OneDrive initial Sync Now retest status:** July 11, 2026. After adding --resync-auth and reinstalling the native and Flatpak prototype builds, user confirmed that OneDrive Offline Mirror Sync Now now starts instead of failing at the abraunegg/onedrive resync confirmation prompt. Full live recheck remains open until setup/save/preview/initial sync/later normal sync behavior is completed end-to-end.

**Flatpak task-list stale-item audit:** July 11, 2026. Reviewed unchecked Flatpak/publication tasks after the host-runner, state-bridge, rclone remote management, SMB password, VPN activation, theme, and OneDrive sync UX work. Marked the Flatpak runtime command-runner task complete because the applet now detects Flatpak mode and selects the `flatpak-spawn --host` runner. Marked host command verification complete because dependency detection, `rclone version`, `nmcli`, `systemctl --user`, `fusermount3`, nonzero status, stderr capture, timeout, cancellation, and redaction were live/probe tested. Marked the host-visible configuration/state bridge and Flatpak configuration/state model complete because native-visible applet config, generated user units, app-owned engine metadata, and provider-owned host state are routed through host-visible locations. Marked Box/Google/SMB rclone remote setup and unused remote removal complete based on live Flatpak prototype testing. Split OneDrive Online Mount and OneDrive Offline Mirror verification into completed setup/auth pieces and remaining end-to-end live rechecks.

**Flatpak local toolchain prerequisite check:** July 11, 2026. Verified `flatpak` 1.16.6, `flatpak-builder` 1.4.2, and `just` 1.42.4 are installed. Verified the user Flatpak remotes include `flathub` and `cosmic`, plus local prototype remotes. Marked the local Flatpak prerequisite, Flathub remote, and COSMIC remote tasks complete.

**Flatpak OneDrive Online Mount partial live recheck:** July 11, 2026. User mounted `UA OneDrive` from the Flatpak applet popup, confirmed files were visible, then unmounted and confirmed access was removed. Host verification showed the onedriver service had exited with status 128 after `fusermount3 -u` failed with "Device or resource busy"; `findmnt` still reported `fuse.onedriver`, while the mountpoint returned "Transport endpoint is not connected." Manual lazy unmount with `fusermount3 -uz /home/uutzinger/Cloud/UA_OneDrive` detached the stale mount and `systemctl --user reset-failed` restored the service to inactive/success. Marked popup toggle/status/file-access and stale-endpoint finding complete, but left the applet Repair confirmation flow open until it is exercised from the popup.

**Flatpak OneDrive Offline Mirror initial sync live recheck:** July 11, 2026. User created a new OneDrive Offline Mirror for the personal Microsoft account named `uutzinger OneDrive mirror`, completed setup, and completed Sync Now from the applet. The saved connection id is `671e12ea-54a8-4077-971a-f115dc77e82b`, remote reference `onedrive-mirror`, local mirror `/home/uutzinger/Cloud/uutzinger_OneDrive_mirror`, and recovery directory `/home/uutzinger/Cloud/.cosmic-mounter-recovery/uutzinger_OneDrive_mirror-671e12ea-54a8-4077-971a-f115dc77e82b`. Host verification showed the generated monitor service exists and is disabled, its ExecStart uses host `/usr/bin/onedrive` with the app-owned confdir and syncdir, the OneDrive confdir contains `refresh_token`, `config`, `.config.hash`, `.config.backup`, `items.sqlite3`, and `initial-sync-complete`, and the local mirror contains downloaded OneDrive directories/files. Marked initial Sync Now completion verified; later normal Sync Now plus start/stop monitoring remain open.

**Flatpak OneDrive Offline Mirror later normal Sync Now live recheck:** July 11, 2026. User ran Sync Now again for `uutzinger OneDrive mirror` after the `initial-sync-complete` marker existed. The app reported completion. Host verification showed the generated service was inactive/dead with `Result=success` and `ExecMainStatus=0` afterward, the `initial-sync-complete` marker remained present, and `items.sqlite3` had a newer modification time than the initial marker. Marked later normal Sync Now verified.

**Flatpak OneDrive Offline Mirror monitor toggle live recheck:** July 11, 2026. User toggled `uutzinger OneDrive mirror` on from the Flatpak popup. Host verification showed `cosmic-mounter-671e12ea-54a8-4077-971a-f115dc77e82b.service` active/running with `MainPID=1393908`, executing `/usr/bin/onedrive --confdir ... --syncdir ... --monitor --monitor-interval 300 --disable-notifications`; logs showed a completed sync followed by continued monitor operation. User then toggled the mirror off. Host verification showed `ActiveState=inactive`, `SubState=dead`, `MainPID=0`, `Result=success`, `ExecMainStatus=0`; logs showed onedrive received the termination signal, performed database vacuum, and stopped cleanly. Marked OneDrive Offline Mirror start/stop monitoring verified.

**Flatpak OneDrive Offline Mirror local-to-cloud Sync Now live recheck:** July 11, 2026. Created disposable local file `/home/uutzinger/Cloud/uutzinger_OneDrive_mirror/cosmic-mounter-test/local-seed.txt`. User ran Sync Now from the Flatpak popup and confirmed `local-seed.txt` exists in OneDrive. Host verification showed the generated service was inactive/dead with `Result=success` and `ExecMainStatus=0`, and the OneDrive `items.sqlite3` database timestamp updated after the local file was created. Marked local-to-cloud Sync Now verified for the OneDrive mirror.

**Flatpak OneDrive Offline Mirror conflict preservation live recheck:** July 11, 2026. Created disposable `conflict-test.txt`, synced it to OneDrive, user edited the remote copy in OneDrive web, then the local copy was edited differently before Sync Now. The sync completed without applet error and the user confirmed the OneDrive web copy retained the remote edit. Local verification showed `/home/uutzinger/Cloud/uutzinger_OneDrive_mirror/cosmic-mounter-test/conflict-test.txt` contains the remote edit, while `/home/uutzinger/Cloud/uutzinger_OneDrive_mirror/cosmic-mounter-test/conflict-test-urslabtop-safeBackup-0001.txt` preserves the local edit. Marked conflict preservation verified for the OneDrive mirror.

**Flatpak OneDrive Offline Mirror deletion/recovery retention live recheck:** July 11, 2026. Created disposable `delete-retention-test.txt`, user ran Sync Now and confirmed it appeared in OneDrive web, then the local file was deleted and user ran Sync Now again. User confirmed the file was removed from the active OneDrive folder and appeared in the OneDrive web Recycle Bin. Local verification showed the active local file was absent, the generated service was inactive/dead with `Result=success` and `ExecMainStatus=0`, and `items.sqlite3` updated after the deletion sync. For abraunegg/onedrive, this verifies provider recycle-bin recovery for deletion retention rather than applet-local recovery-directory retention.

**Flatpak OneDrive Online Mount lazy repair live recheck:** July 11, 2026. Starting from a clean inactive `UA OneDrive` onedriver mount, user reproduced the stale/busy unmount condition using file-manager access and then the applet reported that lazy repair finished. Host verification showed `/home/uutzinger/Cloud/UA_OneDrive` was no longer present in `findmnt`, `mountpoint` reported it is not a mountpoint, the directory was normally accessible again, and `cosmic-mounter-990cc48f-4e4e-4ed7-a07b-c545ad3d3f9d.service` was `ActiveState=inactive`, `SubState=dead`, `MainPID=0`, `Result=success`, and `ExecMainStatus=0`. Marked OneDrive Online Mount lazy-unmount Repair flow verified from the Flatpak prototype.

**Flatpak packaging scaffold and metadata cleanup:** July 11, 2026. Added clear create-folder guidance to the Browse tooltip: users can create a folder in the portal chooser when supported, or type the desired path manually and let the applet validate it before saving. Added `packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json` as the project-owned final-manifest scaffold using the accepted COSMIC runtime stack (`org.freedesktop.Platform//25.08`, `org.freedesktop.Sdk//25.08`, `org.freedesktop.Sdk.Extension.rust-stable`, and `com.system76.Cosmic.BaseApp//stable`), command `cosmic-ext-applet-mounter`, tested host-runner/state finish-args, and final install paths under `/app/bin`, `/app/share/applications`, `/app/share/metainfo`, and `/app/share/icons/hicolor/scalable/apps`. Updated desktop/AppStream metadata to include COSMIC category/keywords and the official template `<binaries>` wrapper. Local strict validators currently reject the official COSMIC category and report the AppStream `<binaries>` wrapper as an unknown provides item, matching the archived COSMIC applet template behavior; therefore the final metadata-validation task remains open until the target `pop-os/cosmic-flatpak` workflow/template is checked at submission time.

**Flatpak minimum-permission decision:** July 11, 2026. Documented the final-manifest permission rationale in `packaging/flatpak/README.md`. Based on prototype live testing, `--filesystem=host` is not required: provider discovery and control happen through `flatpak-spawn --host`, generated user systemd units run host tools in the host session, and the Flatpak only needs app-specific host-visible configuration/cache/state grants plus host COSMIC theme read access. The final scaffold retains Wayland, fallback X11, IPC, DRI, `org.freedesktop.Flatpak`, app-specific COSMIC config, app-owned engine config/cache/state, and user systemd unit-file permissions. The folder portal remains the user-facing folder selection path.

**Flatpak cargo-sources generation command:** July 11, 2026. Added `just flatpak-cargo-sources`, which uses `flatpak-cargo-generator` if installed or the local sibling COSMIC helper script when available, and writes `packaging/flatpak/cargo-sources.json`. Running the recipe initially failed in the sandbox because `just` could not write its runtime directory, then failed outside the sandbox because the sibling helper requires Python `aiohttp`, which was not installed in the current environment. Created a temporary local generator virtual environment, installed the helper dependencies, generated `packaging/flatpak/cargo-sources.json`, and verified it parses as JSON. Updated the recipe to reuse `.venv-flatpak-generator/bin/python` when present, reran `just flatpak-cargo-sources` successfully, and added `.venv-flatpak-generator/` to `.gitignore` so the local helper environment is not tracked.

**Flatpak README documentation update:** July 11, 2026. Added a Flatpak Packaging Status section to `README.md` covering the local GUI prototype build/run commands, the final manifest scaffold path, host dependency expectations, `flatpak-spawn --host` host-command architecture, non-default permission categories, native-visible configuration sharing, source/Debian/Flatpak coexistence warning, and `just flatpak-cargo-sources` usage. Also clarified that `just metadata-check` is non-fatal because strict freedesktop validators currently reject official COSMIC applet template metadata fields; `just metadata-check-strict` shows the raw validator result.

**Native source and Debian compatibility recheck after Flatpak host-runner work:** July 11, 2026. Verified the Flatpak host-runner branch still supports non-Flatpak installs. `just verify` passed: formatting, `cargo check --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, and all Rust tests passed. `just install-user` rebuilt and installed the applet binary, OneDrive auth helper, desktop file, AppStream metadata, and icon under the user-local paths. Initial `just deb` failed only because Debian packaging still treated strict freedesktop validation of official COSMIC applet metadata as fatal (`Categories=COSMIC` and AppStream provides/binaries). Updated `debian/rules` to keep those metadata checks visible but non-fatal, matching `just verify`; reran `just deb` successfully and produced `../cosmic-ext-applet-mounter_0.3.0_amd64.deb`. After deciding this work should become the next release, version metadata was bumped to 0.4.0 in Cargo, Debian changelog, AppStream, and README release-package examples; 0.4.0 build verification is the next required step before tagging.

**Version 0.4.0 pre-merge verification:** July 11, 2026. After the version bump, `just verify` passed for `cosmic-ext-applet-mounter v0.4.0`, including formatting, `cargo check --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, and all Rust tests. `just install-user` rebuilt and installed the 0.4.0 user-local applet binary, OneDrive auth helper, desktop file, AppStream metadata, and icon. `just deb` completed successfully and produced `../cosmic-ext-applet-mounter_0.4.0_amd64.deb`; strict freedesktop metadata warnings for official COSMIC template fields remained visible but non-fatal.

**Version 0.4.0 merge and release-tag evidence:** July 11, 2026. Merged `flatpak-host-runner` into `master` with merge commit `fb7fceb`, reran `just verify` and `just deb` successfully on merged `master`, pushed `master`, and created/pushed annotated tag `v0.4.0` on the merge commit. Local Debian artifact `../cosmic-ext-applet-mounter_0.4.0_amd64.deb` was produced. The official Flatpak publication tasks remain open because the final Flatpak manifest still uses the placeholder `REPLACE_WITH_FLATPAK_READY_COMMIT`, AppStream screenshot URLs still point at an older raw GitHub commit, and no GitHub release artifact or `pop-os/cosmic-flatpak` submission has been completed yet.
