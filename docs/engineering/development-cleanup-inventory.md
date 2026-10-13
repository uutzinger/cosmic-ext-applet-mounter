# Development cleanup inventory

Type: Inventory

Status: Approved narrow cleanup completed; listed profiles retained

Created: October 13, 2026

Last reviewed: October 13, 2026

Scope: Native 0.5.3 cleanup task before 0.5.4 development. Source of truth is
the host COSMIC v2 saved connection document, user-unit files, observed systemd
state, mount table and local folder metadata. Safety: preserve credentials,
pending uploads, mirror/recovery data and retained connections. This review
does not authorize deleting remote files or pruning engine state.

## Services

- All 11 UUID-named managed connection services have matching saved profiles.
  None is an orphan solely because it is disabled.
- UA OneDrive (`990cc48f-4e4e-4ed7-a07b-c545ad3d3f9d`) is enabled/running and
  mounted via onedriver. Preserve it and all production connections.
- The two SharePoint test services/timers below are disabled and not running.
- `cosmic-mounter-openconnect-agent.service` is required authentication support,
  not a stale mount; retain it even when no VPN is active.
- Four legacy `rclone-{ua-box,ua-engr,ua-gdrive,uutzinger-gdrive}.service`
  units are disabled and duplicate managed providers. Potential archive
  candidates, not proven safe to delete: confirm no external scripts depend on
  them. Preserve their May 29 backups in an approved rollback archive.

## Disposable resources

| Resource | Observed ownership/data | Proposed disposition |
|---|---|---|
| `Cloud/Sharepoint_Mirrors/UA_BME_Assessment_Test` | Saved Test_SharePoint Offline UI, ID `ccae0833-6fdf-4aca-bd60-578af692df76`; 9 files, 77,856,561 bytes | Retire only with profile approval and data archive; preserve recovery state. |
| `Cloud/Sharepoint_Mirrors/Rust_Coordinator_Initial_Test_20261011` | Saved Rust Coordinator Initial Test, ID `64db4a04-31ae-44ab-b412-0208c1a85341`; 1 file, 37 bytes | Keep as the 0.5.4 rclone regression fixture unless explicitly retired. |
| `Cloud/cosmic-mounter-onedrive-repair-test-20261011` | No saved profile; no files, one empty subdirectory; old engine state ID `77e49b04-9e9b-45f0-ba22-bf642e526aab` remains | Folder is a retirement candidate; preserve authentication/database state unless separately approved. |
| `Cloud/cosmic-mounter-flatpak-auth-test-20261013` | No saved profile; empty; earlier auth-only draft receipt | Retire empty folder if approved; retain packaging/auth evidence and engine credentials. |
| `Cloud/test` | Empty, unknown ownership | Ask owner; do not infer project ownership from its name. |

Counts are local read-only metadata observations, not proof that remote data is
synced or disposable. Remote library contents were not changed or compared.
Do not activate these mirrors to determine ownership.

The shared recovery roots are not cleanup targets. The top-level recovery root
has no files but 13 directories; SharePoint recovery has 4 files (1,040 bytes)
and 5 directories. Recovery files may protect prior versions. Production
`uutzinger_OneDrive_mirror` is a saved profile and must be retained.

## Approval and execution boundary

October 13 user direction: retain all applet-listed folders/mounts; remove only
unlisted obsolete entries/folders. This supersedes the proposed retirement of
Test_SharePoint Offline UI: both saved SharePoint tests and their data stay.
Archived the four disabled legacy units and the two unlisted file-empty OneDrive
test folders under `target/artifacts/rollback/native-cleanup-20261013/`.
No systemd dependents, activation triggers, autostart references or user cron
entries were found for those legacy units. Rechecked inactive/disabled state
before archiving; retained their older backup files. User manager reloaded;
saved COSMIC connection document remained byte-identical. Credentials,
databases, recovery state, Cloud/test and all listed mount folders remain.

The table above preserves pre-cleanup observations/proposals, not current path
existence. No permanent deletion or remote modification occurred. See archive
README for original locations and rollback instructions.

Confirm retirement scope first. Prefer the app's supported profile removal for
saved tests; do not hand-edit the live COSMIC document while its runtime owns
it. Archive exact approved units/local folders with a relocation manifest and
rollback before deleting anything. Recheck mounts/activity, symlinks, pending
work and ownership immediately before execution. Refresh the user manager only
after approved unit changes, and confirm retained production profiles still
resolve. Remote deletion, token removal and engine-state pruning require
separate explicit scope.
