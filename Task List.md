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

## 0.5.5 Flatpak Pull Request, AI consideration

Deferred from 0.5.4 on October 19, 2026. Refresh the submission against the
accepted 0.5.5 source after legacy removal and GUI cleanup; reuse unchanged
evidence. GitHub bundle distribution in 0.5.4 does not imply COSMIC approval.

- [ ] Prepare the actual COSMIC repository submission after the planned 0.5.5
  changes; operator authors and acknowledges the contribution. Obtain maintainer
  decisions on host execution/dependencies and runtime/portal compatibility.

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
