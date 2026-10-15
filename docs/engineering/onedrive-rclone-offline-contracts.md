# OneDrive Offline mirror on rclone — contracts

Type: Contracts

Status: Accepted design (decisions of October 14, 2026); implementation not started;
generic profiles passed post-initial-sync checks on October 15, 2026;
profiles provisional pending initial-upload round-trip and transfer-count checks

Created: October 14, 2026

Last reviewed: October 15, 2026

Scope: OneDrive Offline mirror connections after `abraunegg/onedrive` is dropped.
Associated task: Task List "Engine decision and OneDrive Offline migration to
rclone". Requirements: FR-031 and related OneDrive Offline requirements, updated by
the same task. Authoritative sources today: `src/sync.rs`, `src/bisync_coordinator.rs`,
`src/teams.rs`, `src/teams_oauth.rs`, `src/model.rs`, `src/config.rs`. This record
describes the target behavior; it is not a claim of tested behavior.

## Decisions

- `abraunegg/onedrive` is dropped: it has no authentication path that works for
  both personal and corporate accounts, and the employer does not support its Graph
  access requirements.
- OneDrive Offline mirrors use `rclone bisync` through the existing coordinator and
  generated scheduled units, the same engine path as SharePoint (Teams), Google
  Drive, Box, SMB and SFTP.
- `jstaf/onedriver` remains the OneDrive Online engine. No other mount engine is
  added or changed. SharePoint document libraries stay under the Teams provider.
- Scheduled bisync plus Sync Now replaces continuous monitoring.
- rclone's default OneDrive client only; no client ID, secret or tenant is stored or
  entered in the applet.
- Existing abraunegg connections keep running for one transition release, cannot be
  created, and are removed in the following release. There is no conversion tool.

## Configuration contract

- A OneDrive Offline connection that uses rclone stores `remote_reference` (the
  rclone remote name), optional `remote_subpath` (relative path inside the drive),
  `local_path`, and the existing `OfflineMirrorConfig` fields (`recovery_directory`,
  `sync_interval_minutes`, `sync_on_metered`). Tokens and OAuth secrets stay in the
  rclone configuration only.
- New field `engine: Option<OfflineEngine>` on `OfflineMirrorConfig`
  (`Rclone`, `LegacyOnedrive`), serialized only when present. Absent means the
  provider default: legacy abraunegg for OneDrive during the transition release,
  rclone for every other provider. New OneDrive mirrors always write `Rclone`.
- New field `onedrive_identity: Option<OneDriveIdentity>` on `Connection`
  (`drive_id`, `drive_type` = `personal` | `business`), modeled on
  `teams_identity` and serialized only when present. It is required when
  `provider = OneDrive`, mode is Offline and `engine = Rclone`; forbidden otherwise.
- Old configuration files load unchanged. Validation rejects a OneDrive rclone mirror
  without an identity, and rejects an identity on any other connection.
- `remote_subpath` follows the existing safe-subpath rules. The local path,
  recovery directory and work directory follow the existing rclone mirror rules and
  must not overlap any other connection, including legacy ones.

## Setup and authentication

1. The user chooses account type (personal or business) and a remote name. The
   applet runs rclone's browser OAuth for backend `onedrive` with rclone's default
   client, using the same create-and-continue pattern as the SharePoint flow
   (`teams_oauth.rs`). The browser opens through the desktop portal in Flatpak.
2. After authorization the applet selects the drive, reads `drive_id` and
   `drive_type` from the remote (in memory, never logged), and checks that the
   remote type is `onedrive` and `drive_type` matches the user's choice.
3. The applet shows the signed-in account or drive name from Microsoft Graph and
   requires confirmation before saving, so a different account in the browser is
   caught before any data moves.
4. Reauthentication in Modify mode must produce the same `drive_id`; otherwise the
   connection is left disabled with a clear message.

Failure handling: a tenant or policy that blocks rclone's default client produces a
clear authentication error; there is no in-app fallback.

## Safety guards

Always on:

- **Identity guard.** Before every scheduled or manual run, verify the saved
  `drive_id` and `drive_type` against the remote and list the target, as the
  SharePoint mirror guard does (`teams::verify_mirror_binding`). A mismatch stops
  the run before rclone transfers anything and invalidates Preview and
  initial-sync state when the target changes.
- **Access marker.** A file `.cloud-mounter-onedrive-anchor-<connection id>` holding
  the connection UUID must exist locally and remotely and match before each sync;
  otherwise the run stops. Created only by the initial sync.
- **Generic gates** (already shared by all rclone mirrors): Preview then confirmed
  initial sync before Start, Save never enables the timer, recovery copies on both
  sides, `--max-delete`, VPN and metered-network checks, bounded timeouts.

Compatibility profile per drive type (chosen from the live check, never assumed):

| Drive type | Profile | Evidence |
|---|---|---|
| Personal | generic (provisional) | October 15, 2026: `target/task-runs/onedrive-guard-check-qmyfk3fx/results.json`; no-edit and post-web-edit syncs stable; 15 files equal, none missing/different; cleanup purged |
| Business | generic (provisional) | October 15, 2026: `target/task-runs/onedrive-guard-check-rsx8nj5g/results.json`; no-edit and post-web-edit syncs stable; 15 files equal, none missing/different; cleanup purged |

Task List phase 2a must compare untouched originals with separately downloaded
files and measure redundant transfers before these profiles are finalized. The
existing receipts establish post-initial-sync stability only.

The generic profile is the rclone bisync settings used by non-SharePoint mirrors.
The compatibility profile is the SharePoint set (`--compare modtime,checksum`,
`--modify-window 1s`, `--ignore-size`, `--ignore-checksum`, `--check-sync false`,
plus a convergence pass). A drive type uses it only if the guard check shows an
unstable second sync with generic settings and a stable one with it.

Verification cadence: full-tree checksum verification runs after the initial sync
and on a manual Verify action, not on every scheduled run, unless the measured
runtime shows it is affordable.

## Scheduling, status and repair

- The scheduled service runs the restricted worker `--run-bisync scheduled <UUID>`
  (native and Flatpak launch forms unchanged). Modes, locking, Pause/Start/Stop,
  Preview, Sync Now, and successful-run retention behave as for other rclone mirrors.
- Default sync interval stays at the existing default (15 minutes) unless the
  runtime measurement supports a shorter one; the existing bounds apply.
- Pending-change counts come from a bounded bisync Preview, never from parsing
  localized text; they replace the abraunegg `--display-sync-status` reading.
- Repair follows the rclone bisync contract (FR-060B and the offline Repair
  contracts record): stop and disable the timer first, reset only observed failed
  state, preserve the bisync safety lock and recovery guidance, never trigger an
  implicit resync or delete data.
- Cloud-native items that cannot be mirrored safely follow the existing rclone
  filter rules. Behavior for OneNote notebooks and shortcuts is pending the
  disposable-account prototype.

## Legacy abraunegg connections

- `engine` absent on an Offline OneDrive connection means legacy. During the
  transition release the connection keeps running unchanged, shows a deprecation
  notice, and cannot be newly created.
- Creating an rclone OneDrive mirror whose local path overlaps a legacy mirror is
  blocked; the abraunegg database, cache and token are never reused; two engines
  never write one tree.
- In the removal release a legacy connection loads as unsupported with a
  "recreate this connection" notice. Its units are not regenerated and its local
  data is untouched.

## Non-goals

Continuous or event-driven monitoring, custom Microsoft app registration, a
conversion tool, SharePoint libraries under the OneDrive provider, and any change to
the onedriver Online path.

## Failure handling

| Condition | Required behavior |
|---|---|
| Wrong account or drive | Identity guard blocks before any transfer; state invalidated. |
| Missing or changed access marker, empty folder | Run stops; nothing is deleted. |
| Expired or revoked token | rclone refresh first; if it fails, a reauthenticate action; run stops. |
| Throttling (HTTP 429) | rclone retry behavior; surface the delay, no data loss. |
| Network or VPN loss | Existing readiness guards; timer stays enabled and retries. |
| Tenant blocks rclone's client | Clear error; no fallback engine. |
| Cancellation | Child processes finish cleanup; lock and recovery state preserved. |

## Acceptance criteria and verification status

| Area | Source/design | Fake-adapter tests | Live disposable | Native install | Flatpak |
|---|---|---|---|---|---|
| Configuration and validation | contract only | pending | n/a | pending | pending |
| Setup and identity confirmation | contract only | pending | pending | pending | pending |
| Guards and profiles | contract only | pending | compatibility profiles verified on sample trees; implemented safety guards pending | pending | pending |
| Scheduling, status, Repair | contract only | pending | pending | pending | pending |
| Legacy handling | contract only | pending | n/a | pending | pending |

Acceptance requires the Task List phases 2–7 to pass: the live guard check, the
disposable-account prototype, tests for each row above, and native then Flatpak
lifecycle runs. Fixtures and synthetic checks are not installed or final-package
acceptance.

## Open items

- Verification cadence and default interval from the runtime measurement.
- OneNote notebook and shortcut handling.
- Minimum rclone version for the OneDrive backend features used.
