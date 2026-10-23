# Flatpak host-command inventory

Type: Inventory

Status: Published 0.5.5 host surface refreshed; scoped installed package smoke recorded; COSMIC host-execution review pending

Created: October 7, 2026

Last reviewed: October 22, 2026

## Published 0.5.5 surface refresh — October 22, 2026

Baseline: packaging/tag cd1b465, app source 13e204e, published Flatpak run
39842934431. The manifest and cargo sources match those published build inputs;
no executable or permission change is made by this documentation refresh.
Default runner has eight shipping variants: flatpak-spawn, flatpak, rclone,
onedriver, fusermount3, mountpoint, find and cat. Test/development-only variants
remain excluded. Legacy onedrive is absent; retirement uses typed systemd D-Bus.
Generated host units additionally use fixed flatpak run worker launch and the
conditional OpenConnect agent, as documented in the payload inventory below.

The reviewed 23 declared grants still serve current supported engines and shared
host state. org.freedesktop.Flatpak permits broad host execution; fixed validated
requests, bounded output/deadlines/cancellation and ownership checks constrain
application behavior, not the user's OS authority. NetworkManager/systemd/logind
remain typed D-Bus operations. Per-family owners, paths and removal criteria in
the October 19 matrix below remain applicable to retained families.

Published built metadata also includes inherited x11 alongside wayland and
fallback-x11; reviewers must consider complete packaged policy and any installed
overrides, not only manifest declarations. Remote test overrides were not captured.
Exact x86_64 Flatpak UI/Online/manual business mirror/deletion/recovery smoke
passed; final scheduled-worker and package legacy regression are not claimed.
Earlier native retirement passed; full per-provider sandbox/browser, portal and
active-transfer coverage retain their recorded limits. See the
[current submission baseline](cosmic-submission-audit.md#current-submission-baseline--published-055).

## Current 0.5.5 exemption summary

Cloud Mounter manages storage mounts and synchronization in the user's host
session. File managers and scheduled user services must be able to use those
mounts and local folders outside the applets sandbox. It requires permission to interact with the host through the following operations. 

| Required host interaction | Why it must reach outside the sandbox | Access required | Application safeguards |
|---|---|---|---|
| Launch host commands through `flatpak-spawn --host` | Storage engines and filesystem helpers run in the host user session and mount namespace. | Session-bus access to `org.freedesktop.Flatpak`, which enables broad host command execution. | Fixed executable identities and separately validated arguments; bounded output, deadlines and cancellation. These checks are application policy, not an OS-enforced executable allowlist. |
| Run `rclone` for storage setup and access | Host engines authenticate accounts, mount cloud storage and synchronize host-visible mirror folders. Desktop portals do not provide these provider-specific storage operations. | Host-installed rclone and its configuration, selected storage paths and app-owned working state. | Validated remote/path identities, credential redaction, ownership checks and guarded synchronization/recovery operations. |
| Run `onedriver` for OneDrive Online access | The OneDrive FUSE mount must be available to host applications and managed by a host user service. | Host-installed onedriver, app-owned configuration/cache and the selected mountpoint. | Separate app-owned engine metadata and validated mount targets; service lifecycle controlled through systemd D-Bus. |
| Detach mounts with `fusermount3 -u` or `-uz` | The FUSE mount belongs to the host mount namespace. | Host FUSE helper and the connection's mountpoint. | Ownership/target checks; lazy detach only after separate confirmation in an eligible recovery state. |
| Read host mount information with `cat /proc/self/mountinfo` | Mounts visible inside the sandbox do not fully describe mounts seen by ordinary host applications. | Fixed host mountinfo path through the host command gateway. | Fixed read-only request, asynchronous execution and bounded output. |
| Check host mount readiness with `mountpoint --quiet <path>` | Starting or stopping a connection requires observing its actual host mount state. | Validated host mountpoint through the host command gateway. | Fixed operation and validated target; no caller-supplied command text. |
| Inspect host directories with bounded `find` operations | The app checks directories, prevents mounting over existing files and optionally preloads directory metadata visible on the host. | Selected host directory or mountpoint through the host command gateway. | Validated paths; bounded readiness/preload operations, depth/exclusion rules and directory-only preload without symlink traversal. |
| Resolve selected host paths with `flatpak document-info <portal-path>` on Documents portal v4 | The supported v4 portal lacks a sandbox-accessible host-path lookup needed by host engines and services. | Host Flatpak helper and a validated portal-selected path. | Typed portal lookup first; compatibility helper only for unsupported interface versions, with a five-second/64-KiB bound. General v5 errors do not activate this fallback. |

Host services are also part of this execution model:

- Online services run host `rclone mount` or `onedriver`. Cloud Mounter controls
  their lifecycle through the systemd user-manager D-Bus API.
- Mirror and guard services start fixed app worker entry points using
  `/usr/bin/flatpak run --command=...` with a validated operation and connection
  identifier. The sandboxed Rust worker coordinates host-engine synchronization.
- OpenConnect VPN authentication can require a user-installed nm-applet secret
  agent and NetworkManager plugin/dialog. The app starts the fixed agent service
  through systemd D-Bus; NetworkManager handles VPN activation and deactivation.

The host command gateway grants substantial authority. App-side validation does
not restrict that authority at the operating-system boundary. The manifest does
not request blanket host filesystem access or blanket bus sockets; app state
uses named filesystem grants, and user-selected directories use portal grants.
Host execution and direct filesystem grants are separate permissions, and both
need review. Per-operation source references and command bounds are documented
below. The [submission baseline](cosmic-submission-audit.md#current-submission-baseline--published-055)
identifies the published source, packaged permissions, tests and unrun limits.

## 0.5.5 removal update — October 20, 2026

Source implementation: the legacy `onedrive` executable identity, authentication,
preview/sync/status factories and monitor startup are removed. Native and Flatpak
share typed retirement through systemd D-Bus, with ownership, loaded-file and
inactive-state checks. Unit-file rollback never enables or restarts the engine.
Existing profile identity and user files/credentials remain intact. Automated
verification is recorded in completion notes; native unit-retirement/data
preservation acceptance passed October 21. Package upgrade and retained-profile
missing-service regression acceptance remain pending, combined for 0.5.5.

The dated 0.5.4 observations below remain historical evidence. Every legacy
`onedrive` invocation in those observations is retired in 0.5.5. Supported rclone
and onedriver still require the host gateway; no shared permission is removed.
Prototype rclone evidence remains development-only and does not imply installed
upgrade acceptance or COSMIC submission approval.

## Scope and method

This records the default shipping surface for the 0.5.4 candidate: app source
72661ce, packaging7069ae5, with later documentation-only acceptance updates.
The October 7/12 migration observations below retain their historical scope;
this inventory does not assert final publication or COSMIC maintainer approval. The review follows the
shipping applet, settings/IPC paths, guards, restricted bisync worker, generated
service payloads, and their command factories.

Rust syntax was inspected with test/feature conditions evaluated for a default
Linux build, followed by manual caller and macro/launcher review. Default and
development-feature builds are checked separately. This is source reachability
evidence, not a formal whole-program proof or a new live-system acceptance test.

Each command has one source disposition:

- **Replaced by D-Bus**: desktop control now uses the relevant D-Bus API.
- **Removed**: the invocation is absent from shipping runtime paths. A retained
  test, development probe, installation check, or display-only instruction does
  not change that disposition.
- **Unavoidable invocation**: a retained engine/host-helper operation or supported
  host compatibility exception has no usable D-Bus replacement for its role.

Current acceptance is in [consolidated release steps 7 and 8](../../Task%20List.md)
and the [0.5.4 evidence map](release-054-acceptance-audit.md). Historical Task (5)
and Task (6) meant the bisync and NetworkManager/OpenConnect migrations.
The inventory is maintained; the
[bisync baseline](2026-10-10-bisync-coordinator-inventory.md) is historical.

## Default executable surface

The shipping typed runner contains eight variants in 0.5.5; the retired ninth
entry is retained below to explain the historical removal:

| Executable | Classification | Role |
|---|---|---|
| `flatpak-spawn` | **Unavoidable invocation** | Gateway implementing the reviewed host requests below; not an arbitrary UI command endpoint. |
| `flatpak` | **Unavoidable invocation** | Version-gated Documents portal compatibility lookup. |
| `rclone` | **Unavoidable invocation** | Provider configuration, authentication, storage operations, and private RC client. |
| `onedriver` | **Unavoidable invocation** | Online OneDrive authentication. |
| `onedrive` | **Removed in 0.5.5** | Legacy Offline OneDrive authentication, preview, sync, and status only; retained for the transition release. |
| `fusermount3` | **Unavoidable invocation** | Clean detach and confirmed lazy recovery. |
| `cat` | **Unavoidable invocation** | Host mount-namespace inspection. |
| `mountpoint` | **Unavoidable invocation** | Host mount readiness. |
| `find` | **Unavoidable invocation** | Host path/emptiness checks and bounded directory-only preload. |

`Findmnt`, `Nmcli`, `Systemctl`, `False`, `Ls`, `Mkdir`, `Rm`,
`Sleep`, `SystemdRun`, and `Touch` exist only for unit tests or the explicit
`development-probes` feature. The unused `Printf` variant is removed.
No `Sh`, `Journalctl`, `SystemdAnalyze`, `Fuser`, `Ip`, `Getent`,
`Nc`, or proprietary Cisco executable variant remains.

The former synchronous bridge is now `src/host_inspection.rs`: it accepts only
`RcloneConfigDump` and `HostMountInfo`, never caller-supplied commands/arguments.
Both inspections are asynchronous with a 10-second execution deadline, no
retries, and capture limits of 1 MiB for configuration and 4 MiB for mountinfo
(per output stream). Oversized/invalid output is rejected, not partially used.
Errors omit captured configuration/diagnostics; GUI completions carry parsed
metadata only. Native mountinfo uses bounded asynchronous `/proc` reading;
Flatpak inspections use the typed host runner with `--watch-bus` so transport
termination also terminates the host child. The shared runner's deadline covers
pipe draining, including descendants holding pipes after parent exit; bounded
child cleanup can add time after the execution deadline.

GUI views read the last successfully refreshed mount observation, not a process
result obtained during rendering. Periodic refresh is limited to open UI
windows; popup opening and operation completions also request refresh. A failed
or superseded inspection does not mean mounts disappeared. Safety-critical
setup, unmount, repair, and shared-VPN checks obtain fresh observations and fail
closed on inspection failure. OneDrive setup rechecks isolation after potentially
long browser authentication. Superseded discovery/prefill jobs are cancelled;
generation/context guards prevent late results replacing newer lists or edits.
Automated source/transport/failure checks are separate from installed UI and
final rebuilt-Flatpak acceptance.

## Replaced desktop control

### NetworkManager

| Former command | Classification | Replacement |
|---|---|---|
| `nmcli -t -f NAME,TYPE,UUID connection show` | **Replaced by D-Bus** | Saved settings discovery via `Settings.ListConnections` and connection settings. |
| `nmcli -g NAME,UUID,TYPE connection show` | **Replaced by D-Bus** | The same settings API; no command fallback. |
| `nmcli connection up uuid <uuid>` | **Replaced by D-Bus** | `ActivateConnection`, observed state, and readiness. |
| `nmcli connection down uuid <uuid>` | **Replaced by D-Bus** | `DeactivateConnection` on the active object. |
| `nmcli -t -f GENERAL.STATE connection show <uuid>` | **Replaced by D-Bus** | Active/VPN connection properties. |
| `nmcli -t -f STATE general` | **Replaced by D-Bus** | NetworkManager state/connectivity. |
| `nmcli -t -f DEVICE,STATE device status` | **Replaced by D-Bus** | Device/interface properties. |
| `nmcli -t -f GENERAL.STATE,GENERAL.CONNECTIVITY general status` | **Replaced by D-Bus** | Connectivity properties; metered policy also uses D-Bus. |
| `nmcli --version` | **Removed** | Dependency inspection connects to NetworkManager D-Bus. |

Sources: `src/vpn.rs`, `src/app.rs`, `src/diagnostics.rs`.
OpenConnect activation/release uses this same adapter. Authentication-agent
readiness/restart uses the fixed user-service D-Bus path; the applet does not
execute `nm-applet` or the OpenConnect authentication dialog itself.

### systemd and progress

| Former command | Classification | Replacement/reason |
|---|---|---|
| `systemctl --user daemon-reload` | **Replaced by D-Bus** | `Manager.Reload`. |
| `systemctl --user start <unit>` | **Replaced by D-Bus** | `StartUnit` and matching job completion. |
| `systemctl --user stop <unit>` | **Replaced by D-Bus** | `StopUnit`; sleep deliberately uses nonblocking job submission. |
| `systemctl --user restart <unit>` | **Replaced by D-Bus** | `RestartUnit` and matching job completion. |
| `systemctl --user enable <unit>` | **Replaced by D-Bus** | `EnableUnitFiles` followed by reload. |
| `systemctl --user disable <unit>` | **Replaced by D-Bus** | `DisableUnitFiles` followed by reload. |
| `systemctl --user reset-failed <unit>` | **Replaced by D-Bus** | `ResetFailedUnit`. |
| `systemctl --user show <unit> ...` state/result/enablement variants | **Replaced by D-Bus** | Unit/service properties and manager unit-file state. |
| `systemctl --user show --property=InvocationID --value <unit>` | **Replaced by D-Bus** | Unit invocation ID property. |
| Sleep `is-active`, `stop --no-block`, and stop-policy `show` | **Replaced by D-Bus** | Active/stop-policy properties and `StopUnit`. |
| `systemctl --user --version` | **Removed** | D-Bus availability inspection. |
| `systemd-analyze --user verify <unit>` | **Removed** | Runtime structural/load validation and rollback. Parser verification remains in build/tests. |
| `journalctl --user _SYSTEMD_INVOCATION_ID=<id> --output=cat --no-pager -n 1000` | **Removed** | Invocation-matched JSON progress and atomic app-owned coordinator state. |
| `systemctl start vpnagentd.service` | **Removed** | Proprietary Cisco handling is outside supported applet scope. |

Sources: `src/services.rs`, `src/sleep.rs`, `src/pending_count.rs`,
`src/vpn.rs`, `src/app.rs`, `src/diagnostics.rs`.

Popup service/timer observations use a bounded asynchronous one-connection batch,
not synchronous D-Bus calls during rendering. UI snapshots retain last-known
states/warnings, with freshness/configuration/generation guards before controls.
Periodic observation runs only while the popup is visible. This is D-Bus
execution hardening, not an additional host-command exception; synchronous
status helpers remain only in off-UI backend/safety and test paths.

### Readiness and proprietary Cisco

| Former command | Classification | Replacement/reason |
|---|---|---|
| `ip route get <target>` | **Removed** | Bounded in-process kernel route selection using UDP connect without transmitting data. |
| `getent hosts <name>` | **Removed** | Bounded asynchronous system DNS resolution. |
| `nc -z -w 3 <host> <port>` | **Removed** | Bounded TCP connection with cancellation. |
| Cisco `vpn -s stats` | **Removed** | Supported profile state comes from NetworkManager. |
| Cisco `vpn connect <host>` / `vpn disconnect` | **Removed** | Supported profiles use NetworkManager activation/deactivation. |
| Cisco `vpnui` and `vpnagentd` discovery/control | **Removed** | NetworkManager OpenConnect and its external secret agent own compatible authentication. |

Sources: `src/vpn.rs`, `src/app.rs`, `src/config.rs`.
The inert legacy Cisco configuration marker and explanatory UI errors remain
necessary for existing saved configurations; they launch no proprietary client.

## Retained production host requests

### Documents portal compatibility

| Command | Classification | Justification |
|---|---|---|
| `flatpak document-info <portal-path>` | **Unavoidable invocation** | Supported sandboxed portal-v4 host compatibility exception. |

`document_portal_origin_path` attempts typed D-Bus resolution first. Native
execution uses `Info`; sandboxed v5+ uses `GetHostPaths`. The legacy lookup is
selected only for the unsupported sandbox interface-version result (reported
version below 5), not for a general D-Bus failure, timeout, or invalid path.
Portal v4 is the supported-host case that requires this exception. Remove it
when the minimum supported host provides sandbox-enabled v5+ host-path lookup.

Source: `src/app.rs` and `src/mirror_access.rs`. October 17: mirror/recovery access
revalidates remembered directory grants through the same version-gated policy;
the v4 host lookup has a fixed operation, watch-bus lifetime, five-second timeout
and 64 KiB output bound. There is no fallback on a general v5 failure. This adds
no host executable, shell text, blanket grant or filesystem permission. Native
and sandbox local I/O are separated from rclone's host engine paths; installed
access/recovery/worker acceptance remains pending.

This is an explicit exception, not an unresolved D-Bus
conversion or a reason to postpone the source inventory scan.

### Storage engines

October 22 Google setup follow-up: Google Create uses the existing
`config create` host-engine route with automatic/default-answer browser OAuth
(`config_is_local true`, no `--non-interactive`). Create/Update now require a
successful existing `lsf` access probe before success. Both stages keep fixed
validated arguments and bounded deadlines/output; no new host executable,
transport or permission. Rclone owns browser OAuth and credential storage.
Creation rollback only targets the newly attempted, initially absent remote;
existing remotes are never deleted on Update failure. Retire these engine calls
when an equivalent reviewed structured engine API replaces process execution.
Source/synthetic verification does not establish installed Flatpak browser/portal
acceptance, which remains pending.

October 22 Box follow-up: Box creation uses the same automatic-browser
`config create` engine route and the shared completion/access checks, with the
same five-minute OAuth and 20-second/16-KiB read-only probe bounds. Error payloads
are not exposed. Initially absent new-remote rollback is retained, with failed
rollback reported. Both Online/Offline editors use this path. No additional
host command, desktop-control CLI, permission or credential location is added;
owner and removal criteria remain the shared engine runner/API replacement above.
Installed native/Flatpak Box browser authorization is not yet verified.

Every row below is an engine-specific operation; neither desktop D-Bus nor a
portal implements its storage/configuration semantics.

| Command pattern | Classification | Purpose |
|---|---|---|
| `rclone config dump` | **Unavoidable invocation** | Host remote discovery and provider/authentication inspection. |
| `rclone config create <remote> <backend> ...` | **Unavoidable invocation** | Separate Google Drive, Box, SMB, SFTP, SharePoint, or OneDrive mirror remote setup. |
| `rclone config update <remote> ...` | **Unavoidable invocation** | Provider updates and Microsoft OAuth continuation. |
| `rclone config password <remote> <field> <secret>` | **Unavoidable invocation** | Engine-owned obscured credential storage; arguments are redacted. |
| `rclone config delete <remote>` | **Unavoidable invocation** | Confirmed unused-remote removal or setup rollback. |
| `rclone lsjson <remote:path> --stat` | **Unavoidable invocation** | Bounded OneDrive token refresh and selected-directory access; metadata only. |
| `rclone lsjson <remote-or-local> --recursive --files-only --no-mimetype --filter-from <filters>` | **Unavoidable invocation** | Explicit OneDrive Preview estimate only; two read-only inventories, each 30 seconds/4 MiB, under coordinator lock. Same scoped host gateway and owned paths; no new permission. Remove when an equivalent reviewed structured engine API replaces spawning. |
| `rclone lsf <remote:path> --max-depth 1` | **Unavoidable invocation** | Read-only remote/library access checks, including Google/Box post-OAuth verification (20 seconds, 16 KiB capture). Owner: shared app engine runner; same native/host-gateway path and permissions. |
| `rclone rc --unix-socket <socket> rc/noop` | **Unavoidable invocation** | Private RC readiness. |
| `rclone rc --unix-socket <socket> vfs/stats` | **Unavoidable invocation** | Cache/upload health and sleep safety. |
| `rclone rc --unix-socket <socket> vfs/queue` | **Unavoidable invocation** | Structured queued/in-progress counts. |
| `rclone rc --unix-socket <socket> vfs/refresh recursive=<false/true> _async=true` | **Unavoidable invocation** | Tracked Google root-first then recursive directory refresh, sharing one deadline; fast-list belongs to VFS creation. |
| `rclone rc --unix-socket <socket> job/status jobid=<id>` / `job/stop jobid=<id>` | **Unavoidable invocation** | Refresh observation/cancellation. |
| `rclone bisync <remote> <local> ... [--resync] [--dry-run]` | **Unavoidable invocation** | The coordinator's fixed preview/initial/manual/scheduled transactions. |
| `rclone mkdir <remote-recovery-day>` | **Unavoidable invocation** | Prepare the selected engine's recovery area. |
| `rclone cat <remote-anchor-or-owner>` | **Unavoidable invocation** | Access-anchor and ownership verification. |
| `rclone lsf <remote-recovery-day>` | **Unavoidable invocation** | Prove an unmarked recovery day is empty before claiming it. |
| `rclone copyto <private-owner-file> <remote-owner-marker>` | **Unavoidable invocation** | Publish applet ownership. |
| `rclone check <remote> <local> --checksum --filter-from <filters>` | **Unavoidable invocation** | SharePoint post-convergence verification; OneDrive initial/manual verification without convergence workaround. |
| `rclone lsf --dirs-only --max-depth 1 <remote-recovery-root>` | **Unavoidable invocation** | Inspect retention candidates after successful synchronization. |
| `rclone purge <verified-owned-expired-day>` | **Unavoidable invocation** | Remove only validated, applet-owned expired remote recovery data. |
| `onedriver --auth-only --config-file <file> --cache-dir <dir> <mountpoint>` | **Unavoidable invocation** | Engine-owned Online OneDrive authentication; validation otherwise inspects metadata. |
| `onedrive --confdir <dir> --reauth [--auth-files <url-file:response-file>]` | **Removed in 0.5.5** | Historical Legacy Offline OneDrive authentication. |
| `onedrive --confdir <dir> --syncdir <dir> --sync --dry-run --verbose [--single-directory <path>]` | **Removed in 0.5.5** | Historical Legacy Offline OneDrive preview/validation. |
| `onedrive --confdir <dir> --syncdir <dir> --sync [--resync --resync-auth] [--single-directory <path>]` | **Removed in 0.5.5** | Historical Legacy Offline OneDrive initial/ordinary synchronization. |
| `onedrive --confdir <dir> --syncdir <dir> --display-sync-status [--single-directory <path>]` | **Removed in 0.5.5** | Historical Legacy Offline OneDrive pending/status inspection. |

RC CLI calls already use rclone's structured private API. A future direct
Unix-socket client could reduce spawns; that is an engine-transport optimization,
not an outstanding D-Bus replacement. No production `core/stats` request remains.
Idle rclone mirrors do not launch uncoordinated dry-run bisync for inferred counts.

Sources: `src/app.rs`, `src/app/sftp.rs`, `src/providers.rs`, `src/sync.rs`,
`src/sharepoint.rs`, `src/sharepoint_setup.rs`, `src/microsoft_oauth.rs`, `src/onedrive.rs`, `src/rclone_refresh.rs`,
`src/pending_count.rs`, `src/sleep.rs`, `src/bisync_coordinator.rs`.

October 15 Microsoft OAuth extraction: `src/microsoft_oauth.rs` now owns shared
fixed config dump/create/update/rollback requests for the SharePoint wrapper and
production OneDrive setup, also reused by the development prototype. The engine owns its host rclone config and
credentials; only bounded, opaque, in-memory capture carries config responses
back to policy code. The production capture API allows dump/create/update only;
rollback delete uses ordinary redacted execution. rclone has no D-Bus/portal API
for these configuration operations. Transport still uses the existing scoped
`org.freedesktop.Flatpak` gateway permission, with no filesystem/bus grant added.
Removal criterion: replace spawning only if an equivalent reviewed engine API
can preserve OAuth, credential ownership and cancellation. OneDrive backend setup/guard ownership is now `src/onedrive.rs`; Add/Modify editor integration is implemented;
installed portal/host-transport acceptance remains pending. The production
OneDrive guard reuses the applet helper/worker launch transport and fixed rclone
stat/config/anchor operations. Graph verification stays in-process HTTPS, not a
host command. Host credentials remain in rclone configuration; local/work/recovery
paths retain the shared native/Flatpak mapping. No new filesystem or bus grant.
The command removal criterion above applies to these OneDrive operations too.

### FUSE and host inspection

| Command | Classification | Justification |
|---|---|---|
| `fusermount3 -u <mountpoint>` | **Unavoidable invocation** | Clean low-level FUSE detach before stopping the owning unit. |
| `fusermount3 -uz <mountpoint>` | **Unavoidable invocation** | Confirmed lazy recovery only in an eligible Online state. |
| `cat /proc/self/mountinfo` | **Unavoidable invocation** | Inspect the host namespace visible to ordinary host applications. |
| `mountpoint --quiet <path>` | **Unavoidable invocation** | Host mount/root readiness. |
| `find <path> -maxdepth 0 -type d -print` | **Unavoidable invocation** | Validate an ordinary host directory. |
| `find <path> -mindepth 1 -maxdepth 1 -print -quit` | **Unavoidable invocation** | Prevent covering nonempty local data with a mount. |
| `find -P <mountpoint> -xdev ... -type d ...` | **Unavoidable invocation** | Bounded, directory-only preload with depth/exclusion policy. |

Sources: `src/providers.rs`, `src/mount_guard.rs`, `src/directory_preload.rs`,
`src/rclone_refresh.rs`, `src/sleep.rs`, `src/app.rs`.

## Removed runtime probes and shell

| Former invocation | Classification | Source result |
|---|---|---|
| `rclone version`, `rclone help`, `rclone mount --help`, `rclone help flags rc`, `rclone bisync --help` | **Removed** | Packaging/install minimums, focused capability tests, and real operation failures replace startup probes. |
| `onedriver --version` / `onedriver --help` | **Removed** | Executable/route discovery without running probes. |
| `onedrive --version` | **Removed** | The same policy. |
| `fusermount3 --version` | **Removed** | Actual detach remains; no inventory command runs. |
| `fuser --version` | **Removed** | No variant, invocation, or optional psmisc dependency remains. |
| `/usr/bin/sh <managed-bisync.sh> <mode>` (manual/scheduled) | **Removed** | One typed Rust coordinator, restricted worker, locking, atomic state, verification, and retention replace the complete transaction. |

Sources: `src/diagnostics.rs`, `src/process.rs`, `src/bisync_coordinator.rs`,
`src/bisync_worker.rs`, `src/mirror_script.rs`, `src/app.rs`.
Raw legacy bisync request factories remain only in unit tests. A displayed
OneDrive manual-auth `xdg-open "$(cat ...)"` instruction is not executed by the
applet and is not the removed bisync shell path.

## Host service payloads and sandbox-local launches

These are separate from an app-issued `flatpak-spawn --host` request, but were
reviewed to avoid overlooking an indirect host executable.

| Payload/launch | Classification | Boundary |
|---|---|---|
| `rclone mount ...` | **Unavoidable invocation** | Host systemd Online service; lifecycle control is D-Bus. |
| `onedriver --config-file ... --cache-dir ... <mountpoint>` | **Unavoidable invocation** | Host systemd Online OneDrive service. |
| `onedrive --confdir ... --syncdir ... --monitor ...` | **Removed in 0.5.5** | Historical legacy Offline OneDrive monitor; new mirrors use the shared rclone worker/timer. |
| Installed applet `--run-bisync scheduled <UUID>` | **Unavoidable invocation** | Native/Debian service runs the installed binary; Flatpak service uses fixed `/usr/bin/flatpak run --command=... <app-id> ...`. Both enter the same coordinator before GUI/IPC ownership. |
| App-owned readiness/mount guards | **Unavoidable invocation** | Fixed flags and validated saved identities through that package-specific launcher. |
| User-installed OpenConnect secret-agent service | **Replaced by D-Bus** | Applet starts/restarts the fixed service through the user manager. Its external nm-applet/plugin process is required authentication infrastructure, not an applet host CLI control path. |

Settings/editor self-launch remains a local child launch, not a host-runner
request. October 18: WebKit helper and direct native `xdg-open` auth launch
retired in 0.5.4; validated legacy sign-in URLs open through the typed desktop
OpenURI portal for native and Flatpak. Manual redirect submission remains.
Graph identity verification and VPN route/DNS/TCP checks are in-process APIs.

## Development and installation exclusions

- The standalone transport probe requires the explicit `development-probes`
  feature and is not enabled by the default native/Flatpak build:
  `cargo build --release --features development-probes --example flatpak_host_runner_probe`.
  It retains generic `nmcli`, `systemctl`, version, error, and disposable FUSE
  transport coverage without converting those test programs.
- Unit/ignored integration tests and examples retain their isolated CLI setup,
  parser, systemd-analyze, systemd, FUSE, and socket fixtures. They do not create
  additional production permissions.
- Installer scripts and deployment instructions may run host package managers,
  version checks, and user `systemctl` commands; they are not applet runtime
  calls and are not launched by the applet.

### OneDrive phase-3 development harness (October 15, 2026)

The `development-probes`-gated `onedrive_mirror_prototype` example uses the existing
`RuntimeCommandRunner`/`FlatpakHostCommandRunner` for fixed rclone engine operations:
config create/update/delete for fresh `cm_proto_` remotes, metadata and ownership
reads, disposable fixture transfers, shared coordinator bisync/recovery, checksum
verification and marker-guarded purge. Host execution is required because the host
owns rclone credentials and engine; Graph identity is an in-process HTTPS API.
No new executable, CLI desktop control or Flatpak grant is introduced.

Sample timing additionally uses fixed `rclone copy` from an explicitly supplied
absolute source directory to the generated session's local/sample directory.
The source is read-only; source/copy SHA-256 inventories must match before cloud
setup. This uses the same engine runner, bounds and cancellation. Forced-host
testing requires those paths to be visible at the same absolute locations;
no additional filesystem permission is implicitly granted by the sample option.

| Command | Permission/path | Owner | Removal/acceptance boundary |
|---|---|---|---|
| `flatpak-spawn --host rclone <fixed prototype operation>` | Existing scoped `org.freedesktop.Flatpak` session-bus permission; shared explicit absolute mirror/work/recovery paths and read-only sample source; mutations only in owned session directories, generated UUID cloud trees or a proven-new auth remote | `src/onedrive_prototype.rs`, gated example and `scripts/onedrive-mirror-prototype.sh` | Development-only; retire the harness after phase-3/4 acceptance or retain as an explicitly gated regression probe. Not a new installed runtime helper. |

Native and forced-host runs share identity and safety policy. A forced-host run
from the host tests transport only; actual sandbox path visibility, browser portal
behavior and installed effective permissions remain separate acceptance gates.

## Remaining acceptance and follow-ups

### October 12 source checkpoint (historical)

Source permission/documentation reconciliation completed October 12. The source
and GUI manifests have the same 23 explicit, justified grants; the GUI prototype
now includes pinned libcosmic's settings-daemon grant. No obsolete command-specific
or proprietary-VPN grant is present to remove. The
[packaging guide](../../packaging/flatpak/README.md#permission-rationale) explains
every grant and retained command family. Two source contract tests guard parity,
runtime/base, build features, packaged helper/catalog, and source/prototype identity.
This is not validation of installed effective permissions or sandboxed worker
access to arbitrary user-selected mirror/recovery metadata; final acceptance
must prove that access or document an approved narrow solution.

All inventory source rows are reconciled; there is no unresolved mixed or
obsolete runtime classification. Keep the portal compatibility exception
explicit. The manifest/permission/documentation source review is complete.
Installed native discovery, SMB/SFTP prefill, responsiveness, expected connection
list, and correct no-active-mounts display were confirmed on October 12.
Final source identity,
native packaging, and rebuilt-Flatpak worker/VPN/mount acceptance still gate
release publication and closure of Tasks (5) and (6).


October 16 dependency/docs reconciliation: new OneDrive Offline mirrors require
host rclone >= 1.74.3; Online uses onedriver >= 0.15.0, and retained legacy
Offline profiles use onedrive >= 2.5.10. The legacy host command/helper remain
until the separate removal phase. Installer --all selections and package
recommendations are unchanged. Configured engine/mode selects missing-tool
warnings; no startup version/help probe, new host executable, filesystem grant
or bus permission was added. Integrated Flatpak acceptance remains pending.


### October 19 reconciliation and transferred-gate closure

Default source review still finds nine shipping executable identities listed
above. Nmcli/Systemctl references found by text search are cfg-gated tests or
development probes; sleep's systemctl helper is test-only. No shipping shell,
proprietary Cisco or removed GUI-helper launch route was found. This is reviewed
source reachability, not a new formal whole-program proof.

Historical October 19 matrix (0.5.4); superseded for current exemption selection
by the 0.5.5 summary above. In particular, its legacy onedrive row is retired.

| Retained family | Permission and host path boundary | Source owner | Removal criterion |
|---|---|---|---|
| rclone, onedriver, legacy onedrive | org.freedesktop.Flatpak host gateway; fixed executable identities and separately validated arguments; host owns engine config and selected storage paths | process.rs, app.rs, providers.rs, sync.rs, microsoft_oauth.rs, bisync_coordinator.rs | Equivalent reviewed engine API; legacy onedrive only after the separate post-0.5.4 retirement/preservation work. |
| fusermount3 | Same host gateway; owned/validated mountpoint, clean detach or separately confirmed eligible recovery | app.rs, providers.rs, sleep.rs | Equivalent host FUSE API preserving namespace and ownership safety. |
| cat/mountpoint/find | Same host gateway; fixed mountinfo read or validated directory/readiness/preload operations, bounded output and cancellation | host_inspection.rs, mount_guard.rs, directory_preload.rs, sleep.rs | Equivalent host namespace/metadata APIs on the supported minimum host. |
| flatpak document-info | Same host gateway; validated portal-selected path, portal-v4 capability exception only | app.rs, mirror_access.rs | Minimum supported host provides sandbox-accessible Documents v5+ resolution; no fallback on general v5 errors. |
| Restricted app worker/guards | App-owned user service; native binary or fixed /usr/bin/flatpak run entry with mode and UUID | bisync_worker.rs, mirror_script.rs, services.rs, app.rs | Retire the corresponding feature/guard; never replace with a generic command endpoint. |

The host gateway is broad OS authority; typed requests are application policy,
not an OS-enforced executable allowlist. No --filesystem=host or blanket bus
socket grant is present. All 23 explicit finish-args remain justified in the
packaging guide. Installed7069ae5 metadata/commit match the verified export;
installation receipt has no app-specific override and only read-only global
theme paths. SDK availability is not used as proof of Platform capabilities.

Close the transferred bisync-worker and OpenConnect sandbox gates at the scoped
levels in the evidence map: actual portal-selected mirror/recovery access,
restart/scheduled worker and recovery observations from unchanged code; latest
source Flatpak negative/positive VPN Online cycle and final zero-active-VPN
D-Bus check. Native/prototype and final-package results remain distinct. Unrun
fault/ownership/legacy cases and full final release checks are not promoted to
passes. No source/permission change or repeat live suite needed for this review.

The WebKit helper and helper-only recommendations were retired October18;
legacy engine compatibility remains, its additional live test waived. Host
engines may have their own dependencies; no WebKit/Ruby build pipeline is bundled
in the app manifest. October16 pending/helper statements above are historical.

External COSMIC acceptance of the host gateway, supported host dependencies,
portal-v4 exception and runtime baseline remains unconfirmed. Current public
submission instructions were checked; they do not grant app-specific exception
approval. See [internal submission readiness notes](cosmic-submission-audit.md).
No maintainer contact, external PR or publication performed.
