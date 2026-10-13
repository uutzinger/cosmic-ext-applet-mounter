# Offline Mirror Repair Contracts

Accepted October 10, 2026 for Task (5). These contracts define recovery from
an interrupted or failed Offline Mirror operation. They are implementation and
acceptance requirements; documenting them does not mean the current code meets
them.

## Shared Repair contract

Repair is a recovery and pause operation, not a synchronization operation.
It must never infer that a resync is safe.

1. Offer Repair for an Offline Mirror only when observed coordinator or systemd
   state is failed, interrupted, inconsistent, or unavailable enough to require
   recovery. Do not route an Offline Mirror through Online-mount FUSE Repair.
2. Use the existing separate two-step confirmation window: the first activation
   arms Repair, a repeated activation before one second does nothing, and a
   second activation from one through 30 seconds confirms it. Expired or
   different-connection confirmation must arm again.
3. Immediately after confirmation, mark Repair pending, hide its action, and
   show recovery progress. Prevent Start, Sync Now, Preview, and a second Repair
   for that connection until reconciliation completes.
4. Reload the authoritative saved connection by UUID. Require the expected
   Offline Mirror engine and applet-owned generated unit identity. Saved
   `enabled = false` must not block Repair when observed runtime state still
   needs cleanup. Do not accept a unit name, executable, path, engine argument,
   or serialized connection from the UI or worker caller.
5. Use the engine-specific systemd D-Bus sequence to prevent a new background
   invocation, then request cancellation of any app-owned manual coordinator
   operation and wait a bounded interval for it to finish. If it cannot be
   stopped safely, report failure: the background trigger may remain paused,
   but do not reset engine state or modify recovery data. Stop and reconcile
   the managed service only through the shared systemd user-manager D-Bus
   adapter.
6. Preserve the local mirror, remote data, recovery directories, engine state,
   authentication state, initial-preview and initial-sync markers, filters,
   logs, progress history, ownership markers, and safety locks. Repair must not
   delete, truncate, rename, rebuild, or take ownership of them.
7. Never run synchronization, preview, retention, purge, cleanup, resync,
   state-database rebuild, or authentication as a side effect of Repair.
8. Reset failed state only for a unit observed as `failed`. Missing, external,
   unknown, still-activating, or still-running state is not success. Re-read
   unit and coordinator state after every required stop/reset sequence.
9. Finish in an observed Paused state. Resume requires an explicit Start or
   Resume action and all normal readiness, initialization, metered-network, and
   concurrency checks. A failed persistence write must not hide a running
   engine or override observed state.
10. Release a VPN only after no operation or service for this connection is
    active, and only under the existing applet-started, unshared-tunnel
    ownership rule. Repair failure must not cause speculative VPN shutdown.
11. Report a bounded, sanitized result that identifies the failed recovery
    stage and gives engine-specific next steps without exposing local paths,
    remotes, credentials, lock paths, or provider output.

## Rclone bisync Repair

This contract applies to the rclone-backed Google Drive, Box, SMB, SFTP, and
SharePoint Offline Mirror engines. Each connection has a periodic timer and a
finite oneshot service.

1. Stop and disable the timer before stopping the service so no new scheduled
   invocation can begin during Repair. Failure to disable the timer is a Repair
   failure even when the current service has already stopped.
2. Stop the oneshot service and wait boundedly until it is inactive or failed.
   Reset failed state only if it is then observed as failed. Ensure the service
   is not enabled directly; the timer is the only supported background trigger.
3. Re-read both units. Success requires an inactive, disabled timer and an
   inactive, non-failed service. An `inactive (success)` oneshot is normal and
   must not be reported as a failure.
4. Preserve rclone's bisync lock, listings, work directory, recovery ownership
   markers, access anchors, request/result state, and initial-sync markers.
   Repair must never remove a lock or add `--resync`.
5. If a run was interrupted, explain that the renewable bisync lock may remain
   valid for up to the configured two-minute maximum. Start remains explicit;
   an older or persistent lock requires the reviewed manual recovery workflow,
   not automatic deletion or resync.
6. Do not run recovery retention. Cleanup remains successful-sync-only.

## `abraunegg/onedrive` Repair

This contract applies only to OneDrive Offline Mirror. It has one continuously
running monitor service and no periodic timer.

1. Stop and disable the monitor service, then wait boundedly until it is
   inactive or failed. Reset failed state only when it is observed as failed.
2. Re-read the service. Success requires it to be inactive, disabled, and not
   failed. Do not look for, create, stop, or disable an rclone-style timer.
3. Preserve the app-owned OneDrive configuration directory, refresh token,
   sync database, engine lock/state, local mirror, recovery directory,
   single-directory selection, and initial-preview/initial-sync markers.
4. Never invoke `--sync`, `--monitor`, `--resync`, `--resync-auth`, `--reauth`,
   or database cleanup as part of Repair. A later explicit Resume starts the
   normal monitor only after ordinary validation.
5. Do not inspect, detach, reset, or remove any `jstaf/onedriver` Online mount,
   mount unit, cache, or FUSE endpoint. Existing overlap validation remains a
   start-time rule, not a Repair side effect.
6. If the OneDrive client leaves a lock or engine recovery condition after a
   stopped process, retain it and present tool-specific reviewed guidance.
   Never delete or rebuild it automatically.

## Failure and UI reconciliation

Repair may partially pause an engine. The final UI must derive from observed
coordinator and unit state rather than the intended action:

| Observed result | Required UI behavior |
| --- | --- |
| Engine stopped and background trigger disabled | Paused; offer explicit Resume and keep Sync Now subject to normal safety checks. |
| Timer disabled but rclone service still active | Show Repair failed/running; retain Stop or Repair recovery access and block new work. |
| OneDrive service stopped but disable failed | Show Repair failed/inconsistent; do not claim Paused persistently. |
| Unit remains failed after reset | Show Error and offer Repair again after the current attempt completes. |
| Unit state missing, external, or unknown | Show Unavailable/Error with an ownership or inspection message; do not modify external units. |
| App-owned manual coordinator does not stop | Show Repair failed/interrupted-run guidance; perform no subsequent unit or state cleanup. |

The Repair action disappears when confirmed, not when cleanup finishes. It may
return only after reconciliation shows a recoverable state. Notices remain
visible through the existing operation-result path and never report success
solely because a D-Bus method call returned successfully.

## Acceptance boundary

Focused fake-manager tests must verify exact action ordering, conditional
failed-state reset, bounded cancellation, reinspection, partial failures,
missing/external units, and that no filesystem or engine mutation is requested.
Installed acceptance must separately interrupt and repair one disposable
rclone bisync mirror and one disposable OneDrive Offline Mirror. Success for
one engine is not evidence for the other.
