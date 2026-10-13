# Bisync Coordinator Inventory

Inventory completed October 10, 2026 for Task (5). This document describes the
current rclone Offline Mirror transaction before replacing the generated host
shell. It is an equivalence contract, not approval of the current architecture.

## Scope and owners

The current path is split among four owners:

1. `src/app.rs` prepares directories and app-owned files, selects the operation,
   enforces the initial Preview/Sync Now sequence, applies manual timeouts, and
   records successful initial operations.
2. `src/sync.rs` validates the connection and constructs the fixed bisync plan,
   provider filters, recovery roots, service, and timer.
3. `src/mirror_script.rs` renders `managed-bisync.sh`, which coordinates the
   host-side transaction for manual and scheduled operations.
4. rclone owns the actual two-way comparison and transfer, its work listings and
   lock, conflict copies, and backup moves.

The systemd user manager starts only `scheduled` mode. Manual `initial-preview`,
`preview`, `initial-sync`, and `sync` run `/usr/bin/sh <managed-bisync.sh>
<mode>` through the app command runner. Native runs execute this directly;
Flatpak runs reach the host through `flatpak-spawn --host`.

## Fixed plan and data locations

Every rclone plan fixes both endpoints, a connection-scoped work directory,
local and remote recovery roots, and a provider filter file. Its common bisync
arguments include:

- `--workdir` and dated `--backup-dir1`/`--backup-dir2` targets;
- `--resilient --recover --max-lock 2m`;
- conflict preservation with no automatic winner, pathname losers, and the
  `conflict` suffix;
- empty-directory creation, a maximum of 1000 deletes, provider comparison
  rules, the generated filter file, and INFO logging;
- a unique UTC nanosecond suffix for backup/conflict names; and
- 10-second connection and 30-second I/O timeouts on every wrapper-issued
  rclone command.

Google Drive excludes cloud-native documents according to its generated filter.
SharePoint excludes `.cosmic-mounter-recovery`, binds its work directory to a
hash of the verified target, compares by modification time plus checksum, and
uses its rewrite-tolerant bisync options.

The transaction may affect these locations:

| Location | Current owner and purpose |
| --- | --- |
| Local mirror | rclone creates, updates, and deletes synchronized content. For SharePoint it also contains the connection-owned access anchor. |
| Remote subtree | rclone creates, updates, and deletes synchronized content. Initial SharePoint sync copies the access anchor to this side. |
| Work directory | App writes the filter, generated script, initial markers, and scheduled progress files; rclone writes listings, lock, and other bisync state. |
| Local recovery root | Wrapper creates a UTC-date directory, validates/writes its owner marker, receives rclone backups, and may remove old owned date directories. |
| Remote recovery root | Wrapper creates a UTC-date directory through rclone, validates/writes its owner marker, receives backups, and may purge old owned date directories. |
| Saved configuration | The SharePoint guard reads the authoritative connection and rejects a removed, disabled, or retargeted mirror; the wrapper does not modify configuration. |

Before every manual operation, the app creates the local mirror, work root, and
local recovery root if needed. It rewrites the provider filter and atomically
replaces the owner-marked script with mode `0600`. It refuses to overwrite an
existing script that lacks the exact connection-owner header. For SharePoint it
also creates the local access anchor only before the first successful initial
sync, then requires a regular non-symlink file with the expected connection ID.

## Common wrapper transaction

All five modes currently perform these stages in order:

1. Run the optional SharePoint preflight helper. It reloads the saved
   configuration, compares the complete bound target, verifies the selected
   document-library identity, and performs a bounded remote listing. Failure
   occurs before any wrapper recovery or sync mutation.
2. Generate a UTC day and unique timestamp, replace the plan's static recovery
   paths with the dated paths, append the unique suffix, and validate the mode.
3. For SharePoint, validate the local access anchor. Normal preview/sync and
   scheduled modes also read and validate the remote anchor; initial modes skip
   the remote check because the first sync has not created it yet.
4. Preview modes run rclone and exit immediately. They do not create recovery
   date directories, write recovery owner markers, run post-sync verification,
   or prune retention data.
5. Mutating modes reject a symlinked local date directory, create it, and accept
   it only when empty or marked for this connection. They then write its owner
   marker.
6. Mutating modes create the remote date directory. An existing marker must
   match this connection; an unmarked nonempty directory is rejected. The
   wrapper then writes the remote owner marker.
7. Run bisync. SharePoint then runs a second convergence bisync and performs a
   checksum `rclone check` using the same filter. Any failure stops the
   transaction before retention.
8. Only after successful synchronization and SharePoint verification, calculate
   a UTC cutoff of 32 days ago. Remove only strictly dated, older, marked,
   connection-owned, non-symlink local directories. List the remote recovery
   root and purge only strictly dated, older, connection-owned directories.
   Cleanup failures are warnings and do not turn an otherwise successful sync
   into failure.

The shell uses `set -eu`, so unhandled failures stop the transaction. Expected
absence checks and retention failures are the only deliberate nonfatal cases.

## Mode and mutation matrix

| Mode | Selection and rclone flags | Data and state mutations |
| --- | --- | --- |
| `initial-preview` | Selected while `initial-sync-complete` is absent; runs `--resync --dry-run` with a 120-second app timeout. | App preparation may create local/work/recovery roots, the SharePoint local anchor, filter, and script. Rclone may update connection-scoped work/listing/lock state but must not transfer or delete mirror data. On success the app writes `initial-preview-confirmable` with the parsed summary. No recovery date directory, remote owner marker, verification, or retention cleanup occurs. |
| `preview` | Selected after initialization; runs `--dry-run` with a 120-second app timeout. | App preparation rewrites the filter/script; rclone may update work/listing/lock state but must not transfer or delete mirror data. SharePoint reads both anchors. No confirmation marker, recovery ownership mutation, verification, or retention cleanup occurs. |
| `initial-sync` | Requires a successful initial-preview marker; runs `--resync` with a 15-minute app timeout and no automatic retry. | Creates/validates both dated recovery directories and markers, mutates both mirror sides through bisync, writes rclone work/listing state, performs the SharePoint second pass/check, and then runs retention. Only after complete success the app writes `initial-sync-complete` and removes `initial-preview-confirmable`. |
| `sync` | Selected when `initial-sync-complete` exists; normal bisync with a 15-minute app timeout and no automatic retry. | Validates both SharePoint anchors, creates/validates dated recovery ownership, mutates both sides and rclone state, performs the SharePoint second pass/check, and then runs success-only retention. Initial markers are unchanged. |
| `scheduled` | Installed service always calls this mode. It performs normal bisync; the timer is enabled only after initial sync by the app. | Requires a 32-hex systemd `INVOCATION_ID`, truncates `live-progress.jsonl`, atomically publishes `live-progress-state.json`, and adds rclone JSON logging. It then has the same recovery, mirror, SharePoint verification, and retention mutations as `sync`. Initial markers are not changed. |

## Scheduled-service boundaries

The generated service invokes `/usr/bin/sh <script> scheduled` and is controlled
through the user-systemd D-Bus adapter. When metered sync is disallowed, its
`ExecCondition` runs the fixed applet guard at every timer firing. The timer is
persistent and uses the configured interval from both activation and last unit
inactivity. The service treats exits 130 and 143 as successful stops.

Scheduled progress is correlated with systemd's current invocation ID. The
reader rejects a malformed, oversized, stale, or mismatched state file and
parses only the app-owned JSON log. A partially written final JSON record means
the run is still in progress rather than failed.

## Behavior outside the shell that the replacement must retain

- Initial Sync Now cannot proceed until the successful initial-preview marker
  exists. Background scheduling cannot start until `initial-sync-complete`
  exists.
- Unit installation is ownership checked and structurally validated. Service
  and timer are initially disabled; Start enables only the timer, while Stop
  stops both units and disables the timer.
- Offline Repair currently stops/disables the timer, stops the service, resets
  a failed service, preserves the rclone lock, and advises waiting for its
  two-minute expiry. It never resyncs or deletes recovery/work state.
- Retargeting a SharePoint mirror changes its target-bound work directory and
  invalidates the old initial-preview and initial-sync markers.
- Manual requests preserve the typed preview/sync timeout, one-attempt policy,
  output limit, cancellation, and redacted error handling even though the
  actual executable is replaced by the shell.

## Gaps exposed by the inventory

These are migration requirements, not behavior to copy blindly:

- Manual preview/sync currently passes hard-coded network/VPN-ready and
  unmetered values into its decision object. Only background Start evaluates
  live NetworkManager readiness, and scheduled firings enforce only the
  metered condition. Task (5) must apply the accepted NetworkManager/VPN
  ownership rules before every offline operation.
- The scheduled script itself does not verify `initial-sync-complete`; it relies
  on the app enabling the timer only after initialization. The typed
  coordinator should enforce this invariant at execution time.
- Manual and scheduled overlap is prevented principally by rclone's renewable
  lock rather than an app-owned request arbiter. The new coordinator must
  publish atomic request/result state and reject overlapping operations before
  starting a second transaction.
- Only scheduled mode publishes structured live progress. The typed coordinator
  needs one authoritative state contract for manual and scheduled runs.
- Filter creation is a direct write while script installation is atomic. The
  replacement should atomically publish all app-owned transaction inputs.
- The current wrapper depends on general shell utilities (`date`, `cat`,
  `mkdir`, `find`, `mv`, `rm`) and rclone subcommands (`mkdir`, `cat`, `lsf`,
  `rcat`, `check`, `purge`) in addition to `bisync`. The Rust coordinator must
  replace the general shell/filesystem work with typed operations while keeping
  rclone only for operations against the storage backend.

## Equivalence gate for implementation

The shell path cannot be removed until focused tests prove all five modes retain
the operation ordering, fixed arguments, target binding, access anchors,
ownership rules, symlink rejection, unique suffixes, SharePoint second pass and
verification, progress correlation, success-only retention, nonfatal cleanup,
initial markers, timeout/cancellation behavior, and failure boundaries described
above.

## Accepted coordinator decision

Manual and scheduled rclone Offline Mirror operations will share one typed Rust
coordinator. It will accept only a validated connection identity and one of the
five fixed modes: `initial-preview`, `preview`, `initial-sync`, `sync`, or
`scheduled`. The mode selects policy within the same ordered transaction; it
does not select a separate implementation.

The shared coordinator owns preflight, readiness, request arbitration, progress
and result state, recovery ownership, rclone request construction, SharePoint
convergence and verification, retention, initial markers, and final status. The
UI and user-systemd service are callers of that same API. Neither caller may
reimplement or omit transaction stages.

## Accepted worker entry point

The coordinator has one restricted worker interface and two package-specific
launch specifications:

- native/source and Debian user services execute the absolute installed Cloud
  Mounter binary; and
- Flatpak-generated user services execute `/usr/bin/flatpak run
  --command=cosmic-ext-applet-mounter
  io.github.uutzinger.cosmic-ext-applet-mounter ...`.

This is a transport difference, not a coordinator fork. Both enter worker mode
before graphical initialization or applet runtime/settings D-Bus ownership and
then call the same Rust transaction implementation. Manual UI operations call
that coordinator library in process. A scheduled systemd service calls it
through the worker entry point.

The worker accepts only a fixed operation mode and connection UUID. It reloads
the authoritative saved configuration, requires an enabled rclone Offline
Mirror, and reconstructs every endpoint and argument from typed state. It must
reject extra or malformed arguments and cannot accept an executable, shell
fragment, raw rclone argument, path, recovery location, or serialized
connection.

The entry-point architecture is accepted, but it is not yet runtime evidence.
A non-mutating disposable user-systemd probe must prove both launch forms,
bounded completion, absence of UI and applet-bus conflicts, unchanged Flatpak
permissions, and strict argument rejection before production bisync is routed
through it.

## Worker probe evidence

The restricted `--run-bisync probe <connection-UUID>` entry point now runs
before localization, async-runtime, graphical, and applet initialization. The
probe parses exactly one fixed mode and one UUID, emits one bounded success
line, and performs no configuration, filesystem, network, rclone, or D-Bus
operation. All production modes, including `scheduled`, remain disabled.

Focused tests cover valid parsing, missing, extra, malformed, and disabled-mode
arguments, bounded output, and both package-specific launch specifications. A
disposable native user-systemd service completed successfully in 6 ms with a
256 KiB memory peak. A disposable Flatpak service completed successfully in
172 ms with a 10.5 MiB memory peak. A Flatpak service carrying one extra
argument was rejected with exit status 2 in 112 ms. Both successful units and
the rejection unit were collected after execution. Rebuilding the development
Flatpak changed no effective permissions.

This proves the restricted worker transport, not the bisync transaction. The
shell path remains authoritative until the shared production coordinator is
implemented and passes the full equivalence gate.

## Accepted Offline Mirror Repair boundary

The engine-specific recovery contract is recorded in
[Offline Mirror Repair Contracts](offline-mirror-repair-contracts.md).
Rclone bisync Repair pauses its timer before stopping the finite service and
preserves the renewable bisync lock. `abraunegg/onedrive` Repair stops and
disables only its continuous monitor and preserves its client database and
engine state. Both are confirmed recovery-and-pause operations; neither may
sync, resync, authenticate, prune recovery data, or use Online FUSE Repair.
