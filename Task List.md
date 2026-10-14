# Cloud Mounter Applet Task List

This document tracks the current work-item checklist. Work through the native
v0.5.3 release is archived in `docs/archive/` (see "Archive and open items carried
forward" below). Detailed execution evidence, manual test notes, command output
summaries, and completion commentary are kept in `Task List Completion Notes.md`.

**Execution environment:** VS Code with Codex

This list implements `Requirements and Specifications.md`. The approved source
description is `Applet Description.md`.

Repository-wide working guidance is maintained in [AGENTS.md](AGENTS.md).

## Archive and open items carried forward

Work through the native v0.5.3 release is archived, unchanged, in
[docs/archive/task-list-through-v0.5.3.md](docs/archive/task-list-through-v0.5.3.md) and its evidence in
[docs/archive/task-list-completion-notes-through-v0.5.3.md](docs/archive/task-list-completion-notes-through-v0.5.3.md).
The items below were still `[ ]` there. They are copied verbatim, grouped by their
original section; the archived copy is historical and no longer updated. Triage
status (October 13, 2026): items now marked `[o]` are handled by the Release 0.5.4 or
Future sections; one is `[x]` with its evidence; the remaining `[ ]` items had no
evidence of completion or replacement and stay open.

### From: Version 0.3 COSMIC Flatpak Repository Publication

_Subsection: COSMIC Repository Submission_

- [o] Open a focused pull request containing the manifest and required
  packaging files, with links to the source repository, MIT license, tagged
  source, build instructions, AppStream metadata, and screenshots.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": prepare the COSMIC review description and obtain the packaging submission approval); no submission has been made.

- [o] In the pull request, call out the host-integration architecture and all
  permissions rather than relying on reviewers to infer them from `finish-args`.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": prepare the COSMIC review description and obtain the packaging submission approval); no submission has been made.

- [o] Complete the repository pull-request checklist: disclose AI-generated or
  AI-assisted code in commit messages, understand and be able to explain every
  submitted change, accurately describe and test the change, and certify it
  under the Developer Certificate of Origin.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": prepare the COSMIC review description and obtain the packaging submission approval); no submission has been made.

- [o] Address repository CI and maintainer review, updating the source tag/hash
  when a packaging fix requires a new application release.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": prepare the COSMIC review description and obtain the packaging submission approval); no submission has been made.

_Subsection: Community Review_

- [o] Ask in the COSMIC App Developer/Mattermost channel for architecture review
  before submission if repository maintainers have not already answered the
  host-integration questions.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": confirm maintainer expectations and prepare the COSMIC review description).

- [o] Include the applet name, source URL, proposed manifest/PR URL,
  screenshots, `com.system76.CosmicApplet` AppStream classification, and a
  concise explanation of the required host integration.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": confirm maintainer expectations and prepare the COSMIC review description).

- [o] Ask specifically whether the applet belongs in `cosmic-flatpak`, Flathub,
  or native distribution packaging; the repository currently directs ordinary
  COSMIC applications to try Flathub first and reserves this repository for
  applets and software unsuitable for Flathub.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Consolidated Flatpak acceptance and release": confirm maintainer expectations and prepare the COSMIC review description).

### From: VPN Authentication Wait and Sleep Cleanup

- [o] A: Add dedicated waiting/progress, Cancel and Retry UI (existing mount
  control remains the retry action). Add explicit authentication-rejection UI.
  Duplicate of the Future item in "Deferred validation after the SFTP native release" ("add the separately planned waiting/Cancel/Retry UI"); tracked there.

### From: SFTP Provider Extension — September 29, 2026

_Subsection: Verification and delivery_

- [ ] Update dependency/setup guidance and release metadata to describe
  delivered SFTP support and any verified limitations before release.

### From: Teams (SharePoint library) implementation plan — October 1, 2026

_Subsection: Identity and first usable Online mount_

- [ ] Check a full installed mount through a Teams-only lost/recovered network
  event if it can be isolated from other active connections. Check a separate
  private/shared-channel site if one is available and record tenant permission
  limits. Test additional installed authentication/consent diagnostics only
  with disposable credentials or a safe simulated failure. Run packaged-host
  checks in Flatpak when its public distribution path exists; do not repeat
  passed assessment-library checks unless a failure or code change warrants it.

_Subsection: Optional usability follow-up_

- [ ] Check Teams OAuth tenant-consent/authorization failure feedback with a
  disposable account or safely simulated provider response. Check Flatpak
  host-browser and credential access when the public package path exists.

- [ ] Optionally add tenant-wide site/library browsing without a pasted library
  URL. Keep URL-guided discovery and manual remote selection as fallbacks.

_Subsection: UA OneDrive reset and sleep incidents — October 2, 2026_

- [ ] Determine how local directories appeared while the share was unmounted.
  Preserve any moved-aside copy if it still exists; do not unmount the currently
  working share merely to recreate this older incident.

- [ ] Install/reload the updated applet and perform one controlled sleep/wake
  check when the active OneDrive mount can be interrupted safely. Verify that
  a busy mount remains usable after wake, and that a clean mount stops and
  restores according to the saved setting without repeated Repair clicks.

_Subsection: All Online engines: empty mountpoint preflight — October 2, 2026_

- [ ] Verify a disposable non-empty target is blocked by a direct systemd
  start. Repeat the host-command check for the packaged Flatpak when that
  installation is available.

_Subsection: Per-connection pending-change count — October 2, 2026_

- [ ] Implement Offline mirror counts for supported engines. Use a bounded
  read-only preview/status check while idle, avoid concurrent bisync preview
  during a run, report confirmed per-run progress for manual and scheduled
  sync, and refresh the scoped count after completion. Include uploads,
  downloads, deletions, and conflicts; show age/unknown when the result is
  stale or incomplete. Use `--display-sync-status` for abraunegg/onedrive
  where available; defer Teams mirrors until their release gate passes.
  - [x] Add read-only OneDrive mirror status parsing and bounded idle bisync
    change checks for rclone mirrors. A disposable initial bisync dry run
    confirmed its output shape; status parsers have focused tests.
  - [x] Read the current scheduled bisync invocation's journal without taking
    its lock. Show a decreasing estimate only after both change summaries and
    completed file operations are reported; never infer zero before bisync
    reports success. Unknown or truncated logs use the active/inactive marker.
  - [ ] Add a reliable count during active manual and scheduled mirror runs,
    then verify that it falls to zero only after a fresh completed check.
    Manual Sync Now still has no streaming progress source. A running timer
    suppresses automatic bisync preview to avoid lock interference, so its
    idle baseline remains unknown until a safe check is available.

- [ ] Verify with disposable data that rclone Online queue counts decrease to
  zero after upload; mirror counts update through Preview, Sync Now, scheduled
  sync, new changes, interruption, and recovery. Cover errors/stale values,
  slow providers, native and available Flatpak builds, and a user-guided
  installed-popup visual check. Update the displayed-count wording if live
  evidence shows an engine exposes only an estimate. The native build was
  installed with `just install-user`; the panel has not been reloaded for a
  visual check, and no work connection was mounted for this feature test.

_Subsection: Reorder Connections. - October 3, 2026_

- [x] Live-verify the Reorder window opens from General Settings, dropdown
  changes reorder the main applet popup list, and changes persist across applet
  restarts.
  Evidence: the user accepted the installed compact Reorder window (completion notes archive, v0.5.0 preparation, "The user accepted the installed compact Reorder window"); storage-backed reorder persistence tests exist. Marked complete by triage; reopen if restart persistence was never checked live.

_Subsection: Reorder recovery audit and account verification — October 3, 2026_

- [ ] Reauthenticate the personal `uutzinger OneDrive` online connection in an
  isolated fresh cache, verify its drive differs from the University drive,
  then restore startup only if the user wants it. Preserve the old work-account
  cache until the corrected mount is accepted.

- [ ] With Cisco VPN connected, run a read-only access check for the saved
  `ua_engr` SMB remote and confirm the VPN assignment in the installed editor.

_Subsection: UA Google Drive unmount incident — October 3, 2026_

- [ ] Resolve separate Google Drive 403 request-quota errors. `ua_gdrive` has
  no custom OAuth client ID, so it uses rclone's shared client. Google Drive
  preload is enabled for 60 seconds with an unbounded recursive directory
  refresh, which may add a burst of requests. Temporarily disable Google Drive
  preload and configure a private client ID for the remote; then check whether
  listing errors recur. These errors did not cause the busy unmount.

_Subsection: Google OAuth client ID for Google Drive — October 3, 2026_

- [ ] I created an OAuth client ID and secret, but authentication fails. It
  reported something like a parsing error and an unexpected field.

- [ ] Create or verify a Google Desktop OAuth client with Drive API access and
  the intended account allowed by the consent-screen audience, then complete a
  fresh browser authorization that issues a new token for that client. Do not
  pair new client credentials with the token issued to rclone's shared client.

_Subsection: Persistent popup startup state — October 5, 2026_

- [ ] Add focused tests for rclone and onedriver Online persistence, Offline
  schedule persistence, new-connection defaults, unit/config reconciliation,
  and sleep/wake not changing the popup intent. Run formatting, locked tests,
  build, clippy, then install and verify the three affected UA connections
  across a controlled logout/login or reboot. Automated persistence/rollback
  tests pass; the full locked suite passes with 285 tests and seven explicitly
  ignored live/integration tests. Locked build, all-target/all-feature clippy,
  formatting, and whitespace checks pass. Installed lifecycle and reboot
  verification remain.

### From: Flatpak D-Bus host-integration migration — Started October 7, 2026

- [o] Complete every remaining D-Bus-replaceable inventory item.
  Only child (6) remained open; transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) (Task (6) final OpenConnect acceptance). Child (1) is deferred to the Future list item on `flatpak document-info`.
  - [o] (1) Retain the version-gated host `flatpak document-info <path>`
    fallback until the supported Documents portal provides the required D-Bus
    API.
    - [x] Add validated D-Bus resolution: use sandbox-enabled `GetHostPaths` on
      Documents portal v5+, and `Info` for native execution. Direct and
      app-scoped portal IDs, NUL-terminated byte paths, absolute-path checks,
      and a read-only live portal lookup have focused coverage. Standard
      `org.freedesktop.portal.*` access requires no extra Flatpak permission.
    - [o] Remove the narrowly version-gated host-command fallback after the
      Flatpak can require Documents portal v5. Pop!_OS currently exposes v4;
      `Info` and `Lookup` are forbidden inside sandboxes and `GetHostPaths` is
      unavailable, so removing the fallback now would break folder selection.
  - [x] Route NetworkManager-managed Cisco-compatible profiles through the
    NetworkManager D-Bus adapter. Direct proprietary Cisco profiles remain a
    separate product-scope decision below.
  - [o] (6) Complete the packaged `network-manager-openconnect` migration for
    the UA VPN, using `nm-applet` as the host NetworkManager secret agent only
    for OpenConnect profiles that require interactive authentication. Follow the
    separate
    [Task 6 NetworkManager OpenConnect migration](#task-6-networkmanager-openconnect-migration)
    below. Retain the installed Cisco client only as a rollback option until the
    installed native and Flatpak acceptance tests pass.
    Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Complete Task (6) final OpenConnect positive VPN/mount/unmount/ownership…").

- [o] Remove every production invocation classified as obsolete at runtime.
  Test fixtures and standalone transport probes remain outside this boundary.
  Only child (5) remained open; transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) (Task (5) final sandbox acceptance).
  - [x] Remove `nmcli --version` and `systemctl --user --version` dependency
    probes; the typed NetworkManager and user-systemd adapters now test D-Bus
    availability directly.
  - [x] (2) Replace the `journalctl --user _SYSTEMD_INVOCATION_ID=...` mirror
    pending-count scrape with structured rclone or app-owned state. Preserve
    current-invocation isolation, truncation/error handling, and accurate
    queued/in-progress/completed counts, then remove the production
    `Journalctl` executable path. Scheduled bisync now writes an app-owned
    JSON-lines rclone log and atomically publishes its systemd invocation ID;
    the reader accepts only state matching the unit's D-Bus `InvocationID`,
    ignores a partially written final record, rejects malformed/oversized
    state, and no production `Journalctl` executable remains.
  - [x] (4) Remove runtime `systemd-analyze --user verify` from unit installation.
    Generated units now pass a strict in-process structural validator before an
    atomic write, then `Manager.Reload` and `Manager.LoadUnit` verify systemd's
    `LoadState`, `LoadError`, exact managed `FragmentPath`, and
    `NeedDaemonReload`. A failed reload/load check restores the prior file and
    reloads the rollback. Systemd's parser verification remains in build/tests,
    and the production `SystemdAnalyze` executable path is gone.
  - [x] Replace startup-wide `rclone version` and help/capability probes with
    packaging requirements plus typed, operation-time capability failures.
    Retain equivalent minimum-version/capability coverage in build or focused
    diagnostic tests rather than silently weakening compatibility checks.
    Production inventory now performs path/route discovery only. The exact
    minimum and command/flag matrix remains asserted by diagnostics tests, while
    real rclone operations surface typed runner failures.
  - [x] Remove runtime `onedriver --version`, `onedriver --help`, and
    `onedrive --version` probes using the same packaging and operation-time
    validation policy.
    Debian recommendations, the dependency installer, and focused tests retain
    both engine minimums; real setup, test, mount, and sync operations diagnose
    failures at their operation boundary.
  - [x] Remove runtime `fusermount3 --version` and `fuser --version` probes.
    Keep actual `fusermount3 -u/-uz` failures actionable; `fuser` has no other
    production use and should leave the executable inventory entirely.
    `fuser` and `psmisc` are gone from source and packaging. Production retains
    only the clean and user-confirmed lazy `fusermount3` detach operations.
  - [x] (3) Replace `ip route get <target>` with an in-process kernel route
    selection. An IPv4/IPv6 UDP connect validates the target-specific route
    without sending a packet and is bounded to five seconds with cancellation.
  - [x] (3) Replace `getent hosts <name>` with Tokio's asynchronous system
    resolver. Native loopback resolution, timeout, and cancellation have
    focused coverage. Read-only `localhost` and public `example.com` lookups
    also succeeded over IPv4 and IPv6 inside the Freedesktop 25.08 Flatpak
    runtime with the manifest's `--share=network` permission.
  - [x] (3) Replace `nc -z -w 3 <host> <port>` with a bounded Tokio TCP
    connection. Endpoint parsing and redacted ready/not-ready results remain,
    the connection is limited to three seconds, and cancellation is immediate.
  - [o] (5, high risk) Accept removal of `/usr/bin/sh <mirror-script> ...` from
    manual and scheduled rclone bisync in the final rebuilt Flatpak. Follow the
    separate
    [Task 5 bisync coordinator migration](#task-5-bisync-coordinator-migration)
    below for the remaining packaging and lifecycle gates.
    Source removal is done; final rebuilt-Flatpak acceptance transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Complete Task (5) final sandbox…").
    - Source implementation is complete for both manual and scheduled modes:
      the shell path and executable variant are removed. The source inventory
      now records **Removed**. Keep this final-artifact acceptance item open
      until Task (5)'s rebuilt-Flatpak lifecycle confirms equivalent behavior.

- [o] Keep native source and Debian installations functional through the same
  typed NetworkManager and systemd adapters; isolate Flatpak-specific D-Bus
  permissions and connection setup.
  Native/Debian accepted in v0.5.3 (archive); the Flatpak side transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) (Task (5)/(6) acceptance).

- [o] Add focused tests for the new D-Bus adapters and regression tests proving
  native and Flatpak behavior remain equivalent for VPN-dependent mounts,
  generated services, timers, sleep cleanup, and failure recovery.
  Adapter tests exist and pass in the locked suite; the native-versus-Flatpak equivalence proof is transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) (Task (5)/(6) acceptance).

- [o] Rebuild with `just build` and `just build-changed`, install the resulting
  Flatpak, and live-test NetworkManager/VPN handling plus generated mount and
  mirror service lifecycles before opening the repository pull request.
  Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Build source-based candidate/final Flatpak…").

_Subsection: Task (5): Bisync coordinator migration_

- [o] Complete the restricted coordinator gate across both packaging modes.
  Only the rebuilt-Flatpak lifecycle remained; transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion) ("Complete Task (5) final sandbox…").
  - [x] Inventory the wrapper's behavior for `initial-preview`, `preview`,
    `initial-sync`, `sync`, and `scheduled`, including which steps may mutate
    local, remote, recovery, work-state, and initial-sync marker data.
    - The reviewed mode/mutation matrix, common transaction ordering, scheduled
      progress contract, out-of-wrapper responsibilities, and migration gaps are
      recorded in [Bisync Coordinator Inventory](docs/engineering/2026-10-10-bisync-coordinator-inventory.md).
      This inventory changes no execution path and is the equivalence contract
      for the remaining design and implementation stages.
  - [x] Use one Rust coordinator for both manual and scheduled runs so their
    backup, ownership, SharePoint, progress, and retention behavior cannot
    drift. The coordinator exposes only the five fixed operation modes and a
    validated connection identity; mode-specific policy selects preview,
    initial, manual, or scheduled behavior without duplicating the transaction.
  - [o] Finish packaging acceptance for the accepted restricted coordinator
    entry point.
    Native/Debian parts accepted; the Flatpak lifecycle step is already transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion).
    - [x] Use two package-specific launch specifications for one Rust worker:
      native/source and Debian units execute the absolute installed Cloud
      Mounter binary, while Flatpak-generated units execute `/usr/bin/flatpak
      run --command=cosmic-ext-applet-mounter
      io.github.uutzinger.cosmic-ext-applet-mounter ...`.
    - [x] Define the worker boundary as a fixed bisync mode plus connection UUID.
      It loads and validates the saved connection and reconstructs the typed
      plan; it accepts no executable, shell fragment, rclone argument, local or
      remote path, recovery path, or serialized connection from the caller.
    - [x] Enter worker mode before graphical startup and before acquiring the
      applet runtime/settings D-Bus names so it can coexist with the panel
      instance. Manual UI operations call the same coordinator library; the
      scheduled service enters it through this restricted worker mode.
    - [x] Add a non-mutating worker probe and confirm both native-direct and
      Flatpak `flatpak run` forms from disposable user-systemd services. Verify
      bounded completion, no window or applet-bus ownership, no new Flatpak
      permission, and rejection of extra or malformed arguments.
      - `--run-bisync probe <connection-UUID>` dispatches before localization,
        async-runtime, GUI, and applet initialization. It performs no config,
        filesystem, network, rclone, or D-Bus operation. The native disposable
        service completed successfully in 6 ms; the Flatpak form completed in
        172 ms. An extra-argument Flatpak service was rejected with status 2,
        focused parser/launch tests passed, and the installed Flatpak permission
        set remained unchanged.
    - [x] Replace the probe with the production fixed-mode entry point after the
      coordinator exists and remove the shell service from production source.
      - [x] Implement the production entry point. It accepts only one of the
        five fixed modes plus a connection UUID, reloads the authoritative
        saved connection, reconstructs the typed plan, and enters before GUI
        or applet-bus startup. Generated scheduled services now use
        `--run-bisync scheduled <UUID>` for both native and Flatpak package
        launch forms.
      - [x] Remove the shell service implementation from source. The legacy
        renderer and `Executable::Sh` variant are gone; scheduled unit tests
        reject a service payload containing either a shell or raw rclone
        arguments.
      - [x] Repeat the installed native user-manager test with disposable
        mirrors. The October 11 native release build exactly matched the
        installed user-local binary. Regenerated units invoked only the
        restricted Rust worker; manual, initial, scheduled, Pause, and Repair
        lifecycles passed without a shell or surviving storage process.
      - [o] Repeat the production worker lifecycle from the final rebuilt
        Flatpak before checking the packaging-mode parent item.
        Transferred to [Release 0.5.4](#release-054--engine-decision-and-flatpak-completion).
  - [x] Confirm scheduled execution can safely use the accepted Rust
    coordinator. The installed timer invoked the restricted worker, published
    matching systemd invocation state, completed successfully, and left no
    shell or storage process; no scheduled-shell exception is retained.
  - Reject the shortcut of invoking the existing raw typed bisync request: it
    omits required recovery preparation, ownership checks, SharePoint guards,
    post-sync verification, and successful-run-only retention cleanup.
  - [x] Define engine-specific offline Repair contracts before implementation and
    keep them separate from online FUSE repair.
    - The rclone bisync contract stops and disables the timer before stopping
      the service, resets only observed failed state, preserves the bisync
      safety lock and recovery guidance, and never triggers an implicit resync
      or deletes data.
    - The OneDrive Offline Mirror (`abraunegg/onedrive`) contract stops and
      disables its continuous monitor without looking for an rclone timer and
      preserves its client database and engine state. It never applies the
      online onedriver lazy-detach behavior.
    - The accepted shared, rclone bisync, and `abraunegg/onedrive` contracts,
      partial-failure UI matrix, and separate installed-acceptance boundaries
      are recorded in [Offline Mirror Repair Contracts](docs/engineering/offline-mirror-repair-contracts.md)
      and requirements FR-060A through FR-060C.

## Release build workflow follow-up — October 13, 2026

User requested permanent reduction of repeated Debian builds after questioning
the attempted third matrix run during 0.5.3 publication. That run was not started.

- [x] Enforce one build per immutable source: build workflow is manual-only and
  cannot publish; tag pushes no longer start a redundant build. Local release
  script starts one matrix or reuses an exact-commit successful run via --run-id.
- [x] Separate publication into promotion-only workflow/script with explicit
  approval, same-repository/workflow/commit and remote-tag checks, complete
  unexpired matrix artifacts, checksums and package identity validation. No
  compilation or build dispatch occurs during promotion.
- [x] Record the rule in AGENTS.md/build guide; add offline policy, package and
  fake-command tests. Read-only reuse validation passed against existing tag
  run 38447948939. No new build/publication or change to v0.5.3 was performed.
- [ ] Verify the new hosted promotion workflow with the next approved immutable
  release. Source/offline tests and live metadata validation are not hosted
  publication acceptance; do not rebuild 0.5.3 merely to test this workflow.

## Agent guide efficiency and organization — October 13, 2026

- [x] Add rebuild justification, bounded output/status monitoring, reusable
  acceptance receipts and operator shell-handoff rules to AGENTS.md. Documentation
  diff checks pass; no build, installation or publication needed for these rules.
- [x] Agree on clearer general versus project/API-specific chapters, separate
  source/Debian/Flatpak workflows and Git/release script ownership. Discuss the
  layout before restructuring; retain approval and artifact-reuse boundaries.
  User approved the proposed layout. AGENTS.md now separates general policy
  from project-specific workflows, with a guide map and script ownership table.
  Refined the boundary after user review: reusable Rust/source/Debian/Flatpak/
  GitHub procedures are in Part I; concrete recipes, manifests, COSMIC/API
  choices and scripts are Part II project bindings.
- [o] Improve maintained scripts' log/summary controls and operator handoff
  support after the guide reorganization; no script implementation in this step.
  Handled by the authoritative
  [script tooling plan](#script-tooling-inventory-and-implementation-plan--october-13-2026).

## Script tooling inventory and implementation plan — October 13, 2026

- [x] Create scripts/README.md and the maintained
  [script/tooling inventory](docs/engineering/script-tooling-inventory.md),
  separating project deployment, runtime helpers and development/release tools.
  All nine active scripts retained after caller review.
- [x] Consolidate nine already archived local tools from archive/scripts/ into
  scripts/archive/ with byte parity, README and preserved Git exclusion. No
  active runtime payload retired; no install/service/data operation performed.
- [x] P0: Update the stale Rust Debian publication source contract for split
  build/promotion workflows; earlier Python-only checks missed this assertion.
  Use the focused test, not a repeat package matrix.
  Six focused contract tests pass, covering build-only/manual execution,
  explicit approved promotion without rebuilding, artifact verification and
  existing-run reuse. No package build or release operation performed.
- [x] Agree/implement the inventory's bounded logging/receipt contract and
  improve existing hosted/release scripts, precise run selection and safe resume.
  P1: shared private logs/atomic summaries, request-ID correlation, structured
  status polling, verified download cache, explicit tag/run resumption and
  persistent promotion receipts. Tests use fake CLIs only; next approved hosted
  acceptance remains separate. See
  [output/resumption contracts](docs/engineering/script-output-contracts.md).
- [x] Add the justified dev and shared-inspection wrappers; improve
  deployment-script reporting/tests. Keep existing just recipes as underlying
  implementation rather than creating competing procedures.
  - [x] P2 shared offline artifact inspector and extracted promotion verification
    policy: Debian matrix/single package and native byte receipts. Focused
    inspection, promotion and hosted-download fixtures pass; no real build.
  - [o] P2 local-Debian wrapper dropped by user: native source development is
    local; Debian packages use the six-target hosted workflow. Retain just deb
    for exceptional diagnostics, not a mandatory release gate.
  - [x] P2 fixed dev-task allowlist around existing just recipes, with private
    reporting; fixture checks cover output, rejection, exit status and cleanup.
  - [x] P2 opt-in setup reporting and focused fixtures; preserve standalone Bash
    use, existing dependency selections and service/data safety boundaries.
  - [x] P2 close fixture/regression, help/catalog and operator-handoff gate:
    70 applicable Python tests pass; source/fixture scope, not live deployment.
- [x] P3 explicit-manifest Flatpak build/export wrapper, shared metadata
  inspection and fake-command failure/reuse tests. Tooling implementation is
  independent of release/engine decisions; live build and final package acceptance
  stay in the project packaging plan. Installed-smoke wrapper remains optional.
  See the inventory's P2/P3 exit criteria for authoritative tooling scope.
  - [x] P3.1 explicit-manifest immutable-input build/export/bundle wrapper,
    private receipts and fake-command failure/cancellation checks. No live build.
  - [x] P3.2 shared Flatpak repository/bundle identity and complete metadata/
    permissions comparison, reused by build/export; fake tests and synthetic
    local OSTree/Flatpak checks pass. No installed/package acceptance claimed.
  - [x] P3.3 explicit schema-2 candidate reuse with successful producer summary,
    input/artifact identity, metadata and read-only fsck verification. No builder/
    export/runtime calls on reuse; mismatches fail without rebuild or cleanup.
    Final tooling gate: 93 Python methods pass; source/fixture acceptance only.

## Release 0.5.4 — Engine decision and Flatpak completion

Planned October 13, 2026. Authoritative home for deferred Flatpak work from
Tasks (5)/(6) and 0.5.3. Prior [x] receipts remain historical; transferred [o]
steps are not completed. Start with architecture, not more WebKit/auth code.

### Decide supported engines before implementation

- [ ] Decide whether continuous/event-driven offline mirroring is required or
  scheduled bisync plus Sync Now meets product intent. Compare rclone-only with
  retaining abraunegg/onedrive; justify continued jstaf/onedriver support and the
  maintenance cost of any permanent user-selectable engine choices.
- [ ] Define native/Flatpak engine matrix and explicit legacy migration policy.
  Preserve existing connections, credentials, engine databases/caches and pending
  writes. Never silently reinterpret account labels as rclone remotes or run two
  engines on one mirror. Update Description/Requirements after deciding.
- [ ] If consolidating, implement/test personal/business OneDrive rclone Online
  and Offline routing, auth/drive identity, generated units, status/RC, repair,
  initialization, retention and safety contracts on disposable trees. Cover
  network loss, cached reads, reconnect/pending uploads, remote edits, conflicts/
  deletions, cancellation and recovery before migrating real data.
- [ ] If retaining abraunegg, review prior callback/device/helper trials and
  validate the distinct host-terminal/script authorization route with isolated
  confdir. Use upstream complete callback flow rather than just generating a URL
  or reinventing OAuth. Organization consent remains independent of browser UI.
- [ ] Decide whether WebKit packaging is still justified. Retain probe evidence
  without claiming shipped support. Only if selected, finish Ruby/WebKit/PyGObject
  builds, preserve sandboxing/licenses/security maintenance, remove build-only
  Ruby, and prove automatic capture/failure/cancel/cleanup. Keep one explicit
  manual fallback, not duplicate manual entry approaches.

### Consolidated Flatpak acceptance and release

- [o] Adjust release promotion tooling to publish the exact verified artifact
  set from one successful immutable-source build without rebuilding it. Retain
  explicit approval, source/run/checksum validation and native/Flatpak gates;
  ordinary tag builds must still never publish automatically.
  Source implementation and focused checks handled by
  [Release build workflow follow-up](#release-build-workflow-follow-up--october-13-2026);
  next hosted promotion acceptance remains explicitly open there.
- [ ] Commit accepted implementation and generate/check Cargo sources; pin
  immutable 0.5.4 candidate/final source. Keep COSMIC BaseApp/Freedesktop 25.08
  unless a separate reviewed runtime decision explicitly changes the target.
- [ ] Build source-based candidate/final Flatpak with shipping features only;
  verify source/runtime identity, helper/worker payloads and size. Inspect effective
  grants including BaseApp/global overrides; no silent blanket host filesystem/
  bus grants or restoration of obsolete shell wrappers.
- [ ] Prove selected engine auth and sandbox mirror/recovery/state access through
  reviewed grants/portal authorization. Host-path resolution is not an access
  grant. Verify Platform capabilities, not merely SDK imports or host packages.
- [ ] Complete Task (5) final sandbox Preview/initial/ordinary/scheduled sync,
  worker/progress, pause/resume, cancellation, failure/Repair, lock/initialization
  preservation and successful-only retention; expand provider matrix as applicable.
- [ ] Complete Task (6) final OpenConnect positive VPN/mount/unmount/ownership and
  one negative authentication lifecycle. Keep prior native/prototype receipts
  separate. Decide Cisco rollback disposition only after relevant acceptance;
  no OS client removal is implicit.
- [ ] Verify final Online engines, busy unmount/Repair, clean-profile setup,
  upgrade/removal preservation, panel/settings integration and translations.
- [ ] Reconcile host-command inventory/permission rationale, close transferred
  Tasks (5)/(6) sandbox gates and prepare concise COSMIC review description.
  Confirm maintainer expectations for retained host execution/dependencies.
- [ ] Finalize 0.5.4 metadata/notes/checksums, inspect native/final Flatpak artifacts
  and obtain release publication/COSMIC packaging submission approval.

# Cloud Mounter Applet Future Task List

Items that will be activated at some time in the future.

## Deferred validation after the SFTP native release — October 1, 2026

These are follow-ups, not additional gates for the native SFTP release above.
Move one here into the release gate only if its supported behavior changes or
the final tests expose a related defect.

- [ ] After the public COSMIC Flatpak repository/package is available, test
  installed Flatpak SFTP editor notices, Online and Offline controls, generated
  host services/timers, and a clean-profile mount/mirror/VPN/uninstall smoke
  test. The local Flatpak prototype and host bridge have separate evidence.
- [ ] If later needed, broaden installed SFTP error-notice checks to host-key,
  unreadable-file, permission, agent, and timeout variants, and live-check a
  VPN-bound SFTP connection. Direct protocol and automated classifier tests
  already cover these errors; they are not repeat gates for the native release.
- [ ] Broaden Google Drive `--fast-list` testing to a larger ordinary drive and
  shared drive, including API/rate-limit behavior; repeat cloud cold-cache
  preload trials without reading or changing file contents.
- [ ] Live-check Box and SMB preload on the UA and LAN shares, and shared
  cancellation on OneDrive, Box, SMB, and Google Drive's RC refresh when
  suitable active-preload shares are available.
- [ ] Complete broader installed native/Flatpak Settings title, focus,
  keyboard, scaling, long-list, editor-save, and background-poll visual checks.
- [ ] Exercise the full Cisco/NetworkManager password/MFA, concurrent-request,
  and rejected-credential path with a real VPN profile; add the separately
  planned waiting/Cancel/Retry UI before accepting that extended workflow.
  Include a VPN-bound SFTP connection when that combination is available.
- [ ] Exercise repeated logind/inhibitor and native/Flatpak lid/menu suspend
  fault cases, then investigate the separately reported OneDrive sleep hang
  with journal evidence. The native SFTP release gate above includes one
  normal sleep/wake cycle only.

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
    or shell-wrapper behavior.
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
