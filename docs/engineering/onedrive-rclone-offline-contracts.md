# OneDrive Offline mirror on rclone — contracts

Type: Contracts

Status: Accepted design (decisions of October 14, 2026); phase-3 prototype source,
fake-adapter and local-engine checks passed; native cloud prototype account/scenario
flows accepted for both drive types; new production folder guards await live acceptance;
production configuration, shared Microsoft OAuth, OneDrive setup/guards and guarded
coordinator/planner, shared app runtime and Add/Modify editor paths implemented;
production sync activation and installed acceptance pending;
generic profiles confirmed by original/download and measured-sync checks for
personal and business sample trees on October 15, 2026

Created: October 14, 2026

Last reviewed: October 15, 2026

Scope: OneDrive Offline mirror connections after `abraunegg/onedrive` is dropped.
Associated task: Task List "Engine decision and OneDrive Offline migration to
rclone". Requirements: FR-031 and related OneDrive Offline requirements, updated by
the same task. Authoritative sources today: `src/sync.rs`, `src/bisync_coordinator.rs`,
`src/sharepoint.rs`, `src/sharepoint_setup.rs`, `src/microsoft_oauth.rs`, `src/model.rs`, `src/config.rs`. This record
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
   client. Phase 4 extracted the shared rclone create/continue machinery from
   `sharepoint_setup.rs` and the tested prototype into `microsoft_oauth.rs`. Both the
   production SharePoint wrapper and gated OneDrive prototype use that module;
   production OneDrive backend setup now uses `onedrive.rs`; editor integration remains pending. `sharepoint_setup.rs` remains
   the SharePoint-specific wrapper for library URL/selection/verification, while
   OneDrive keeps primary-drive and account-type identity handling. The browser opens
   through the desktop portal in Flatpak.
   The phase-3 personal prototype requests `User.Read` alongside rclone's default
   file scopes for its `/me` signed-in-profile check. Business uses the default
   scopes without requesting additional profile consent. It binds the configured
   drive to `/me/drive` and `/users/{expected-UPN}/drive`; all drive IDs must agree,
   both responses must be business drives with matching nonempty owner user IDs.
   It requires the business sign-in UPN, not an arbitrary mail alias. Missing or
   denied metadata fails closed; no unchecked token claims or display-name
   fallback. Existing remotes are not automatically reauthorized. Personal and
   revised business native OAuth/identity and normal-scenario checks passed
   October 15; Flatpak-style consent/transport still require live acceptance.
   Microsoft documents file-permission drive lookups in
   [Get Drive](https://learn.microsoft.com/en-us/graph/api/drive-get?view=graph-rest-1.0)
   and the personal profile prerequisite in
   [Get user permissions](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0#permissions);
   [rclone 1.74.3's defaults](https://github.com/rclone/rclone/blob/v1.74.3/backend/onedrive/onedrive.go)
   omit `User.Read`.
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

October 15 shared transport boundary: `microsoft_oauth.rs` owns remote-name
validation, proof of absence before new setup, bounded create/continue requests,
in-memory config/token decoding, typed command failures and explicit partial-setup
rollback. Provider wrappers still choose and verify drives; no generic OAuth
framework or SharePoint library flow is imposed on OneDrive. Step limits remain
eight for SharePoint and twelve for primary-drive setup. Failed cleanup is an
explicit error; existing names are never accepted for new creation.

The restricted `CommandRunner::private_config` capture is now available in
production as well as development: only rclone config dump/create/update,
at most 1 MiB per stream and 300 seconds, no launch-only request, opaque Debug
and failure output. Ordinary runner output stays redacted. Continuation state
and answers are sensitive arguments, including in request Debug output.
Truncated/invalid UTF-8 captures fail closed. SharePoint's plain-text completion
fallback still requires full identity verification; incomplete capture cannot
take that fallback. This is source behavior, not a new live OAuth acceptance run.

### Common Microsoft identity policy (accepted October 15)

Required resource/account binding and optional profile enrichment are separate:

| Connection | Required verification | Profile policy |
|---|---|---|
| SharePoint Online and Offline | Authenticated library ID/type/URL; saved connection binding and selected-subtree access remain mandatory | Setup/test may read `/me` for a label using the existing token; missing permission, transport failure or malformed profile yields no label. No added `User.Read` scope. Service guards omit the profile request. |
| OneDrive business Offline | Configured primary business drive matches `/me/drive` and `/users/{expected-UPN}/drive`, including nonempty matching owner IDs | No extra profile consent and no profile request needed for verification. Future optional setup labels must not replace this check. |
| OneDrive personal Offline | `/me` matches the entered account, then primary personal drive matches the configured drive | Retain explicit `User.Read` and required profile verification. Denial is an account-verification failure, not an optional-label failure. |

A `/me` request does not acquire permissions or start consent. Attempting it with
an existing token is different from requesting `User.Read` during authorization;
the latter can block authorization before any token exists. Do not add that
consent requirement to business or SharePoint merely for a display label. A
successful SharePoint label is not proof of an expected OneDrive account: shared
library access is valid independently of that label.

All modes fail closed on required resource/account mismatch, missing metadata or
denied access. Reauthentication must preserve the saved drive binding. Personal
profile removal requires an equivalent expected-account check or an explicitly
approved requirement change. The remaining production OneDrive setup/guards task
must preserve these tested personal/business checks; optional enrichment belongs
to setup, connection tests or an explicit user refresh, not recurring sync guards.

Source status: SharePoint setup/tests retain optional enrichment; Online and
Offline service guards now use identity-only verification. Fake-adapter checks
cover unavailable profiles, skipped profile calls, required failures and label
success. Existing OneDrive prototype verification/scopes are unchanged; production
OneDrive activation and installed native/Flatpak acceptance remain pending.

## Safety guards

Always on:

- **Identity guard.** Before every scheduled or manual run, verify the saved
  `drive_id` and `drive_type` against the remote and list the target, as the
  SharePoint mirror guard does (`sharepoint::verify_mirror_binding`). A mismatch stops
  the run before rclone transfers anything and invalidates Preview and
  initial-sync state when the target changes.
- **Access marker.** A file `.cloud-mounter-onedrive-anchor-<connection id>` holding
  the connection UUID must exist locally and remotely and match before each sync;
  otherwise the run stops. Created only by the initial sync.
- **Generic gates** (already shared by all rclone mirrors): Preview then confirmed
  initial sync before Start, Save never enables the timer, recovery copies on both
  sides, `--max-delete 50` (bisync percentage, not a file count), VPN and
  metered-network checks, bounded timeouts. Excess deletion stops the run;
  the coordinator never adds `--force` or an implicit resync to bypass it.

Compatibility profile per drive type (chosen from the live check, never assumed):

| Drive type | Profile | Evidence |
|---|---|---|
| Personal | generic (`--compare size,modtime`) | October 15, 2026: `target/task-runs/onedrive-guard-check-8zm9owl_/results.json`; both independent downloads match originals; two no-edit and post-edit syncs transfer 0 files/bytes; cleanup purged |
| Business | generic (`--compare size,modtime`) | October 15, 2026: `target/task-runs/onedrive-guard-check-xpn4ex9d/results.json`; both independent downloads match originals; two no-edit and post-edit syncs transfer 0 files/bytes; cleanup purged |

Task List phase 2a completed the original/upload/download and transfer-count
comparison. All 15 test files matched original SHA-256, actual byte lengths and
QuickXorHash at both observations, with no reported-size discrepancies. Remote
timestamps differed in representation/subsecond precision; neither this nor the
web edits caused repeated transfers. The sources remained unchanged and all
script-owned cloud folders were purged. These results justify the generic profile
for the tested samples and 10-second observation interval, with no SharePoint
compatibility additions or extra convergence pass. Longer-term service behavior,
implemented safety guards and installed/native/Flatpak acceptance remain outside
this check. The earlier phase 2 receipts retain their post-initial-sync scope.

The generic profile is the rclone bisync settings used by non-SharePoint mirrors.
The compatibility profile is the SharePoint set (`--compare modtime,checksum`,
`--modify-window 1s`, `--ignore-size`, `--ignore-checksum`, `--check-sync false`,
plus a convergence pass). A drive type uses it only if the guard check shows an
unstable second sync with generic settings and a stable one with it.

Verification cadence: full-tree checksum verification runs after the initial sync
and after manual Sync Now, not on every scheduled run. SharePoint convergence
workarounds are not enabled for OneDrive.

## Scheduling, status and repair

- The scheduled service runs the restricted worker `--run-bisync scheduled <UUID>`
  (native and Flatpak launch forms unchanged). Modes, locking, Pause/Start/Stop,
  Preview, Sync Now, and successful-run retention behave as for other rclone mirrors.
- Default sync interval stays at the existing default (15 minutes) unless the
  runtime measurement supports a shorter one; the existing bounds apply.
- Explicit successful bisync Preview publishes an estimate from two filtered,
  read-only `lsjson --recursive --files-only --no-mimetype` inventories (remote
  and local), each bounded to 30 seconds and 4 MiB. Structured Path/Size/ModTime/
  IsDir metadata counts differing file paths with one-second time tolerance;
  Jiff parses RFC3339 offsets and fractional seconds. This is an estimate, not an
  exact transfer/deletion/conflict count. No localized log parsing or automatic
  background dry runs are used for rclone OneDrive counts.
- The owned workdir's atomic `preview-pending.json` records schema version, UUID,
  timestamp and optional count. Async reads are bounded to two seconds/4 KiB;
  foreign, stale (three minutes), future, malformed or linked reports are unknown.
  Inventory failure keeps Preview successful but its estimate unknown. Runtime
  work invalidates prior estimates; re-reading does not refresh their age.
- Save installs owned units paused and disables scheduling. Existing timers are
  paused before replacement, and a failed timer installation restores the prior
  service document. Start requires initialized state and readiness/resource
  guards, observes an active enabled timer, and stops/disables it on failure.
  Pause observes timer and service stopped/disabled without clearing failed state;
  Repair retains its separate conditional reset and cancellation policy.
- Repair follows the rclone bisync contract (FR-060B and the offline Repair
  contracts record): stop and disable the timer first, reset only observed failed
  state, preserve the bisync safety lock and recovery guidance, never trigger an
  implicit resync or delete data.
- Cloud-native items follow the policy below; an Offline mirror is not a complete
  backup of Microsoft cloud-only objects.

### OneNote and shortcuts (October 15 source policy)

Keep rclone's `expose_onenote_files=false`; remotes enabling it are rejected.
The OneDrive filter excludes `.one`, `.onepkg`, `.lnk` and `.url` files and the
selected subtree's recovery directory. Ordinary file content uses the generic
size/modtime profile; no size/hash bypass is added for unsupported cloud items.
OneNote notebooks remain accessible through the browser rather than being
presented as fully mirrored notebooks.

For a selected folder, Graph must report a real folder on the saved drive, with
no `package` or `remoteItem` facet. A notebook package, shared shortcut root,
foreign-drive folder or incomplete metadata blocks setup and every guarded run.
The guard also requires successful rclone `lsjson --stat` with `IsDir=true`.
The [Graph folder metadata endpoint](https://learn.microsoft.com/en-us/graph/api/driveitem-get?view=graph-rest-1.0)
uses file permissions; the [remoteItem facet](https://learn.microsoft.com/en-us/graph/api/resources/remoteitem?view=graph-rest-1.0)
identifies cross-drive/shared items without adding profile consent.
Nested cloud shortcuts retain rclone's resolution behavior: they are not preserved
as cloud shortcut relationships, and this guard does not inventory or certify all
descendant objects. Use ordinary source folders for supported mirrors; unreadable
or unsupported descendant listings fail the engine operation and do not establish
successful initialization. Do not silently skip a failed listing or relax the
comparison rules. Separate live cloud-item acceptance remains pending.

Sources: rclone's [OneNote option](https://rclone.org/onedrive/#onedrive-expose-onenote-files)
and [shared-item limitations](https://rclone.org/onedrive/#can-not-access-shared-with-me-files).
No backend switch exists here to universally exclude Graph shortcut folders by
filename; `.lnk`/`.url` filters cover file shortcuts only. This is an explicit
support boundary, not a claim that glob filters detect every cloud shortcut.

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
| Configuration and validation | implemented; engine/identity, mode-aware selection and overlap checks | ten focused compatibility/routing/editor tests passed October 15 | n/a | pending | pending |
| Setup and identity confirmation | production backend and Add/Modify editor implemented | focused synthetic setup, consent, replacement binding, rollback and editor cancellation/stale-result checks | prior prototype account flows accepted; new folder metadata path pending | pending | pending |
| Guards and profiles | production drive/subtree guards and target-bound planner implemented | required/optional identity, edited binding, anchors and initial checksum checks | generic profiles verified; new production guard/cloud-item acceptance pending | pending | pending |
| Scheduling, status, Repair | shared app routing implemented | lifecycle ordering, ownership, rollback, status, Preview metadata and manual checksum tests | new production lifecycle pending | pending | pending |
| Legacy handling | engine dispatch preserves legacy monitor and Online routes | focused routing/status compatibility tests | n/a | pending | pending |

Acceptance requires the Task List phases 2–7 to pass: the live guard check, the
disposable-account prototype, tests for each row above, and native then Flatpak
lifecycle runs. Fixtures and synthetic checks are not installed or final-package
acceptance.

October 15 phase-4 configuration slice: optional fields serialize only when
present; old OneDrive documents still select the legacy engine and old documents
for other providers still select rclone. New OneDrive Offline editor drafts
explicitly select rclone and cannot pass configuration validation until setup
supplies an identity. Editing preserves the stored engine and identity.
`RcloneBackend::try_from(&Connection)` selects OneDrive only for an explicit
rclone Offline mirror with a syntactically valid identity; provider-only routing
continues to reject OneDrive, preserving the Online onedriver path. The legacy
planner rejects explicit rclone mirrors. October 15 follow-up enabled the production planner/coordinator only for explicit
rclone OneDrive mirrors with a valid identity; legacy and Online routing remain
unchanged. Editor and lifecycle dispatch integration are still pending.

New rclone OneDrive mirrors cannot overlap another connection's local or recovery
paths, including legacy paths; existing legacy-only document semantics are
preserved. Editor validation also checks all configured local/cache/recovery
paths against the caller-resolved native/host-visible work root. Runtime guard
integration remains subsequent work. Automated coverage uses synthetic documents
and paths; no installed configuration, services or cloud state was changed.

## Open items

- Verification cadence and default interval from the runtime measurement.
- Installed/live acceptance of the documented OneNote/shortcut support boundary.

## Rclone version and flag assessment

October 15: retain the existing project minimum **rclone >= 1.74.3**. This is the
supported project floor, not a claim that every older release lacks these flags.
The installer (`RCLONE_MIN_VERSION`), diagnostics policy (`MIN_RCLONE_VERSION`)
and `debian/control` already specify 1.74.3; no dependency minimum change is needed.
Runtime diagnostics resolve executables/host transport and report operation
failures; do not add startup version/help probes.

Checked the generic planner/coordinator against the pinned upstream
[1.74.3 bisync flag definitions](https://github.com/rclone/rclone/blob/v1.74.3/cmd/bisync/cmd.go)
and [OneDrive backend](https://github.com/rclone/rclone/blob/v1.74.3/backend/onedrive/onedrive.go).
The backend supports the personal/business drive configuration used by the
prototype. Bisync supports `--compare size,modtime`, `--workdir`, `--filters-file`,
`--backup-dir1`, `--backup-dir2`, `--resilient`, `--recover`, `--max-lock 2m`,
`--conflict-resolve none`, `--conflict-loser pathname`, `--conflict-suffix conflict`,
`--create-empty-src-dirs` and initial `--resync`. No OneDrive requirement for a
newer flag was found. Shared global timeout, dry-run, suffix and logging options
remain unchanged. OneDrive does not add SharePoint's ignore-size/checksum or
timestamp/convergence workarounds.

Personal/business cloud acceptance used rclone 1.75.0. The 1.74.3 conclusion is a
source/API compatibility assessment; that binary was not installed or exercised
against cloud accounts. Native and integrated Flatpak acceptance remain distinct.

The initial flag audit found that bisync interprets `--max-delete` as a percentage,
clamping values above 100 to 100. The previous shared value 1000 therefore did
not enforce a 1,000-file limit. October 15 implementation corrects all shared
planner profiles and both development guard profiles to **50%**, matching
upstream bisync's default. The limit applies to each side's previous listing;
exactly 50% is allowed, while more than 50% aborts before propagating deletes.
This is a percentage policy, not an absolute file-count cap. Recovery copies on
both sides and failure/retention behavior remain unchanged. No automatic force,
resync or retry with a higher limit is added.

Planner tests cover generic providers, OneDrive's shared prototype path and
SharePoint without dropping backup directories. Fake-runner failure coverage
checks the actual coordinator request, initialization-marker preservation and no
retention after failure. An explicitly run real-rclone integration test uses
temporary local alias data/config only: 50% succeeds with recovery copies, then
60% fails on either side while retaining peer bytes and existing backups. This
is local engine evidence; no cloud deletion test, installed service update or
Flatpak acceptance is claimed. No minimum-version increase is needed.


## Phase-3 prototype implementation boundary

The gated `src/onedrive_prototype.rs` module and `onedrive_mirror_prototype` example
reuse SharePoint's planner and coordinator transaction functions. The development
entry point accepts only the prototype OneDrive connection; production routing
continues to reject legacy OneDrive connections from the rclone path until phase 4.
The generic plan keeps shared preview/initialization markers, access anchor,
locking, dated recovery/retention, structured coordinator state and cancellation.
OneDrive-specific code verifies default-client config, expected Graph account,
primary drive/type and the saved identity fingerprints before operations. The
owned root also keeps remote recovery inside the disposable tree.

This separation is not permission to fork the production mirror implementation:
phase 4 extends the shared SharePoint path with a per-drive-type profile and
OneDrive identity binding. The tested generic profile excludes SharePoint
workarounds/convergence; full verification remains initial/manual unless measured
cost supports another cadence. Prototype auth reuses the existing Microsoft token
extractor and bounded rclone create/continue pattern; tokens never enter receipts.

Operator actions and failure/cleanup boundaries are documented in
[scripts README](../../scripts/README.md#onedrive-development-prototype-phase-3).
No applet configuration, installed units, mount or timer is created by the harness.
Native OAuth/identity and normal scenarios passed for both types October 15;
shared cancellation/network-fault/Repair checks passed on personal. Per operator
instruction, redundant business fault runs are deliberately omitted, not recorded
as business-specific live passes. Remaining Flatpak-style work targets transport
and account-flow differences plus one disposable sync/verification/cleanup smoke;
unchanged scenario/fault code reuses native evidence. Installed sandbox/portal,
cloud-native item acceptance remain open. Minimum-version source assessment is
recorded above. The synthetic 16 MiB benchmark is non-representative;
it must not finalize a five-minute schedule or checksum-verification cadence.

Operator sample timing is a separate baseline: prepare --files-dir copies a
bounded source tree into a new session and verifies source/copy SHA-256 inventories
before cloud setup. Initial-sync and timing are separate actions; timing measures
quiet sync, sync plus checksum verification and a synthetic small edit without
replaying scenarios/faults or modifying original samples. Counts and digest identify
the copied workload; no source paths or file contents are stored in receipts.
The result stays non-representative with cadence_exercised=false. October 15
operator decision accepts the completed 317-file sample baseline and deliberately
skips larger-tree/repeated-interval prototype tests, reusing existing SharePoint
scheduling coverage for unchanged shared machinery. Integrated OneDrive unit and
status checks remain implementation work; no periodic prototype run is claimed.

October 15 operator sequencing decision: remaining Flatpak checks transfer to
Task List phase 4 after routing, setup, planner/coordinator, units/status and editor
integration. Reuse the existing applet Flatpak manifests/runtime and relevant
probe setup; a separate prototype sandbox is not a prerequisite to implementation.
Check integrated account flows, portal paths, host/sandbox boundaries and service
status with one disposable sync/verification/cleanup smoke. Native results do not
complete that acceptance; unchanged shared scenarios/faults are not repeated.

Primary API references: [Microsoft Graph primary drive](https://learn.microsoft.com/en-us/graph/api/drive-get?view=graph-rest-1.0)
and [rclone OneDrive backend](https://rclone.org/onedrive/). These describe APIs;
the run receipts establish tested behavior.


## Phase-4 production remote setup and guards (October 15)

`src/onedrive.rs` owns the promoted prototype account/primary-drive policy, typed
safe errors, bounded/cancellable Graph and rclone requests, production remote
setup and read-only selection. The development harness delegates to this module;
it retains its `cm_proto_` name restriction, scenario ownership and protocol
receipts. Drive types use the configuration model enum rather than a second enum.

`RemoteSetup` validates the entered account before creation. `authenticate` uses
shared `microsoft_oauth::NewRemote` and its default-client scopes, drive-choice
allowlist, bounded continuation and explicit owned-new-remote rollback.
`verify_existing` reads an existing remote without editing it. For reconnecting,
pass the saved `previous_identity`: only an identical drive/type is accepted.
A rejected replacement is rolled back; existing remotes and saved configuration
are never modified by setup. The editor requires this result before accepting a new or edited binding. Failed
setup clears draft identity and disables the draft; saved configuration is unchanged.

Recurring checks verify the configured backend/type/default-client settings and
saved drive ID against `/me/drive`, then selected-folder access, without `/me`
profile access or a persisted account email. Remote inspection rejects alternate
roots, custom clients/tenant endpoints and exposed notebooks. The restricted
`--verify-onedrive-mirror` entry point reloads authoritative configuration and
compares UUID, drive/type, remote/path, local and recovery binding before access.
The coordinator invokes the same guard before preparing sync inputs for every
mode, then checks UUID access anchors for normal manual/scheduled operations.

The shared plan uses a digest of that binding for its work directory: a changed
target cannot reuse another target's Preview/initialization markers. Recovery
stays inside the selected subtree when one is chosen and is filtered from sync.
Initial Preview does not create a OneDrive access anchor. Confirmed Initial Sync
creates it and requires a checksum check before publishing initialization; normal
runs use UUID anchors and ordinary bisync without SharePoint convergence or a
recurring full checksum check. Missing, changed, linked or incomplete markers
stop the operation. Shared locking/recovery/deletion policies remain in force.

This is source/backend integration and synthetic coverage, not installed native
or Flatpak acceptance. The app editor and Start/Stop/Pause/Repair/status dispatch
are still tracked by their following implementation tasks. No installation,
services, user configuration or live remote was changed for this work.


### SharePoint source naming (October 15)

`teams.rs` is now `sharepoint.rs`; the provider-specific `teams_oauth.rs` wrapper
is now `sharepoint_setup.rs`. Shared transport remains in `microsoft_oauth.rs`.
Rust callers, library/setup result types and internal UI setup messages use
SharePoint names. The configuration enum `Provider::Teams`, `teams_identity`
field and serialized `TeamsLibraryIdentity` struct name are preserved, as are
existing guard flags, unit identifiers, remote names and Fluent keys. This is a
source refactor without a profile migration or new live/installed acceptance.


## Add/Modify editor integration (October 15)

New OneDrive Offline drafts use personal/work-or-school selection, an ephemeral
expected-account field, rclone remote discovery, Connect new remote and Verify
selected remote. Discovery separates primary drive types from SharePoint document
libraries; selecting a remote is not authentication or proof of identity. Existing
Online onedriver and legacy Offline monitor profiles retain their controls.

The async adapter calls production RemoteSetup/authenticate or verify_existing.
Reconnect reads previous_identity from saved configuration, not draft identity;
a replacement must match the saved drive/type. Existing remotes are inspected
without config replacement; Connect requires an unused name and retains backend
owned-new-remote rollback on failure. Changed account, type, remote or folder
clears verification. No setup operation synchronizes files or changes services.

Setup has a cancellation token and explicit Cancel action. Test/Save are blocked
until it finishes; generation, editor binding and saved identity are rechecked
before applying the result. Changed/cancelled results never become accepted
configuration; cleanup failures remain visible, and discovery can review a remote
created before a late cancellation. Late Test/Save validations are also checked
against the current draft. Dropping pending state cancels its work. Account input
is not serialized into connections. Save retains the paused runtime policy.

Focused synthetic tests exercise both account types through the real backend
adapter with fake rclone/Graph, plus discovery, old/new routing, saved-identity
reconnect rejection, cancellation, stale results, serialization and referenced
remote protection. Compilation and the final gate are recorded in completion
notes. Rendered controls, browser consent, installed services and final Flatpak
acceptance still require the planned disposable operator tests.
