# Flatpak Packaging Prototypes

This directory contains Flatpak packaging experiments. These files are not the
final COSMIC repository submission.

## Candidate build/export wrapper

`scripts/build-flatpak.sh` provides logged build/export only, with no install or
submission. It requires a selected JSON manifest in a separate clean checkout
at an expected commit, tracked referenced inputs, full Git source commit pins
and SHA-256 for remote archives/files. Tags alone and directory-source prototypes
are rejected. The source manifest now pins reviewed 0.5.5 implementation commit
`13e204ec93049587611978c0cfbad0f9e0a5968c` for package preparation.
The wrapper does not change that pin or choose a runtime. The packaging changes
must also be committed before a clean-checkout candidate build.

**Published 0.5.5:** tag/packaging cd1b465 uses the app pin above. Both Flatpak
architectures built and were inspected; exact x86_64 installed UI/Online/manual
mirror smoke passed. Remaining acceptance limits and immutable identities are in
the [current submission baseline](../../docs/engineering/cosmic-submission-audit.md#current-submission-baseline--published-055).
The earlier dirty-checkout preflight is historical, not a current build blocker.

Example interface (placeholders, not a request to build now):

```sh
bash scripts/build-flatpak.sh \
  --manifest /path/to/clean-checkout/packaging/flatpak/selected.json \
  --source-sha FULL_PACKAGING_CHECKOUT_COMMIT \
  --arch x86_64 --branch candidate \
  --output-dir /path/to/new-candidate
```

The output parent must exist, the destination must be new/outside the source
checkout, and the report directory must be separate. Prefer a new destination
under `target/artifacts/candidates/` in the tooling checkout when building from
a separate clean source checkout. `--log-dir` chooses a new private report
directory; default logs are under `target/task-runs/`. `--offline` forbids source
downloads; it does not populate caches. Required runtimes/SDK/BaseApp must already
be available. Failures leave partial output for inspection, never automatic
cleanup/retry. The resulting repository/bundle, built metadata and hash receipt
are unaccepted candidates, not installed or store-ready packages. Shared metadata
inspection now compares both repository and bundle to built metadata before
success. Existing destinations are refused unless --reuse-receipt is explicitly
selected. The wrapper also requires the local `ostree` CLI.

### Building a locally committed candidate

If the pinned application commit is local and has not been published, add
`--local-source-repo /absolute/path/to/application-git-repository` to the build
command. This must be a Git repository containing the exact full app source commit
selected by the manifest. It never copies an uncommitted working tree or relaxes
clean packaging/pin checks. Only the builder child receives an environment-scoped
Git URL substitution for that one app repository; global Git configuration and
the manifest URL/pin stay unchanged. Registry/dependency Git sources keep their
normal transports. `artifacts.json` records the local repository, logical URL and
commit as `source_transport`. This option is build-only; verified reuse preserves
original provenance without needing the local repository again.

### Reuse instead of rebuilding

Repeat the original manifest/source/architecture/branch/output arguments, adding
`--reuse-receipt /path/to/original-report/artifacts.json`. Use a new `--log-dir`
or let the wrapper allocate one. For example, after a successful candidate build:

```sh
bash scripts/build-flatpak.sh \
  --manifest /path/to/clean-checkout/packaging/flatpak/selected.json \
  --source-sha FULL_PACKAGING_CHECKOUT_COMMIT \
  --arch x86_64 --branch candidate \
  --output-dir /path/to/existing-candidate \
  --reuse-receipt /path/to/original-report/artifacts.json
```

Keep the receipt and its sibling `summary.json`. Reuse requires schema 2, a
successful producer summary, matching source/manifest fingerprints and build
options (including `--offline` if originally used). It verifies exact artifact
paths/hashes, recorded runtime/tool identities, repository integrity via
`ostree fsck --quiet`, and repository/bundle ref/commit/metadata agreement.
It never invokes the builder, export or bundle creation, and does not require
build runtimes still to be installed. Original build provenance is retained;
current inspection tool versions are recorded separately. The new report links
the previous receipt; original reports/candidate files are not overwritten.

Any mismatch, missing output, corrupt repository, partial build or legacy
receipt fails closed. There is no automatic rebuild or repair. Inspect the
private log first; if a new build is actually authorized, choose a new destination.
Legacy candidates remain inspectable with the standalone inspector but are not
silently promoted to reusable schema-2 candidates. Local receipts are comparison
baselines, not signed attestations. No installation/publication is implied.

## Integrated OneDrive acceptance preflight

After an approved candidate build, close settings/editors, pause mirror syncs,
and temporarily remove the native/present applet from the panel. Install the
reviewed existing bundle through the maintained entry point (separate approval):

```sh
bash scripts/install-flatpak.sh --candidate-receipt /absolute/path/to/artifacts.json \
  --activate-panel-entry
```

The installer repeats candidate verification, requires an existing matching user
ref for rollback identity, refuses running Cloud Mounter owners and installs no
dependencies/related packages. It records previous ref/commit and verified new
identity/metadata/user overrides in private `installation.json`. It does not
launch the app or alter services. Optional `--activate-panel-entry` preserves and
parks the overriding user desktop launcher in the private receipt directory, so
COSMIC discovers the real Flatpak export. Do not copy that export into ordinary
user applications: COSMIC classifies Flatpak applets by desktop-file location
and needs that identity to forward the panel socket. The receipt records both
paths/hashes; restoring the saved launcher reverses the selection. With the exact
candidate already installed, `--panel-entry-only --activate-panel-entry` verifies
it and corrects discovery without reinstalling. The native package is retained.
Re-add the Flatpak panel entry afterwards;
confirm the sandboxed process, since a native installation may also be present.

Use the maintained wrapper for read-only host observations before building:

```sh
bash scripts/build-flatpak.sh --preflight \
  --manifest packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json \
  --arch x86_64 --branch master
```

This writes private `summary.json`, `task.log` and `preflight.json` under
`target/task-runs/`. It does not build, install, download, launch, edit overrides,
change services or access cloud accounts. It observes source dirtiness/pins,
Platform/SDK/Rust extension **runtime refs**, COSMIC BaseApp **app ref**, installed
user/system application metadata and global/application overrides. Missing apps
are valid observations; unexpected command failures remain failures. Existing
source-build checks also use these typed ref identities, including SDK extensions.

A successful inspection receipt is not a ready build or acceptance pass: inspect
`ready_for_candidate_build`, `blockers` and `acceptance=not-run`. Uncommitted source
and mutable manifest pins remain build blockers. Actual build still requires a
clean pinned packaging checkout, full source commit, new output directory and
separate logs. Preflight does not relax or bypass those checks. Source manifests
for the acceptance candidate keep the shipping feature set and existing
Freedesktop 25.08/COSMIC BaseApp target; the old v0.5.2 source is not current evidence.

The existing installed application ID is shared with native configuration and
worker identities. Use its reviewed branch explicitly; installing a replacement
requires approval and retained rollback identity. Do not run competing native
and Flatpak owners. The full operator sequence is in the
[OneDrive acceptance procedure](../../docs/engineering/onedrive-rclone-offline-contracts.md#integrated-flatpak-operator-sequence-october-16).
Transport-only probes and local-binary GUI prototypes do not establish final
source-based package acceptance.

## Offline artifact inspection

Select the expected metadata keyfile explicitly after reviewing it. For example:

```sh
python3 scripts/inspect-artifacts.py flatpak-bundle /path/to/candidate.flatpak \
  --app-id org.example.Candidate --flatpak-arch x86_64 --branch candidate \
  --expected-metadata /path/to/reviewed-metadata.ini
```

Use `flatpak-repo /path/to/repo` for an exported local repository with the same
identity/metadata options; `--expected-commit` optionally pins its OSTree commit.
Both require `ostree`; bundles also require `flatpak`. No application/runtime is
installed or launched. A bundle is imported only into a private temporary repo,
which is removed after inspection. All keyfile sections/keys are compared, including
permissions and environment; do not publish potentially sensitive metadata receipts.
The build wrapper uses its built metadata as the consistency baseline, not an
independently approved permission policy. Manifest finish-args are not treated as
proof of exported permissions. Checks cover selected ref/commit/metadata and bundle
bytes—not payload integrity, signatures, installed overrides or final acceptance.

## Project packaging status

October 13 release decision: 0.5.3 ships through native source/Debian channels.
Final Flatpak packaging is deferred to 0.5.4, starting with the engine decision in
[Task List](../../docs/archive/task-list-through-v0.5.4.md#release-054--engine-decision-and-flatpak-completion).
Retain candidates/probes as evidence; do not ship unvalidated WebKit modules or
claim native success completes sandbox acceptance.

The permission review below applies to the 0.5.4 development candidate. Its source
manifest pins implementation commit `72661ce9d705038cdb1878165917f298e0ced8c2`,
including the integrated OneDrive rclone editor/runtime and subsequent tests and
diagnostics. This is a development source pin, not a release tag or installed
acceptance result. Version 0.5.4 metadata is prepared; final artifact acceptance
and release-note finalization remain open.
The GUI prototype copies local artifacts and is not reproducible release proof.
Both manifests use the same application ID: installing either replaces that
user's existing Flatpak installation, not the native/Debian package. Do not run
native and Flatpak runtime owners together against shared configuration.

## Host Runner Probe

`io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json` copies the
locally built `flatpak_host_runner_probe` example into a minimal Flatpak. It is
used only to verify whether `flatpak-spawn --host` can execute the host tools
the applet needs.

Build the probe binary first:

```sh
cargo build --release --features development-probes --example flatpak_host_runner_probe
```

The extra probe executables are available only to tests or this explicit
development feature. Native/Debian and production Flatpak builds use the
default feature set; do not enable `development-probes` for the shipped applet.

Build and install the local Flatpak:

```sh
flatpak-builder --force-clean --user --install \
  target/flatpak-host-runner-probe \
  packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.HostRunnerProbe.json
```

Run the probe:

```sh
flatpak run io.github.uutzinger.cosmic-ext-applet-mounter-probe
```

Expected checks:

- `rclone version`
- `nmcli general status`
- `systemctl --user --version`
- `fusermount3 --version`

These commands belong only to this development transport probe. The production
applet does not run version/help commands for dependency inventory. Native runs
discover executable availability; the Flatpak confirms its host-execution route
and reports a typed failure from a real operation when a host tool is absent or
incompatible. Packaging/install-time checks enforce supported minimums.

This prototype grants only `--talk-name=org.freedesktop.Flatpak`, which is
required for `flatpak-spawn --host`. Add broader filesystem or session
permissions only after a specific failing feature proves they are required.

## GUI Prototype

`io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json` copies the
locally built applet binary, translation catalog, desktop file, AppStream metadata, and icon
into a Flatpak so the real GUI can be smoke-tested before the final
reproducible manifest exists.

Install the Freedesktop 25.08 SDK, Rust extension, and COSMIC BaseApp from
Flathub before building. The COSMIC repository hosts applets but does not
currently publish the BaseApp ref; `com.system76.Cosmic.BaseApp//stable` on
Flathub uses the same Freedesktop 25.08 runtime as this manifest.

```sh
flatpak install --user flathub \
  org.freedesktop.Sdk//25.08 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08 \
  com.system76.Cosmic.BaseApp//stable
```

Build the applet first:

```sh
cargo build --release
```

Build and install the GUI prototype:

```sh
flatpak-builder --force-clean --user --install \
  target/flatpak-gui-prototype \
  packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.GuiPrototype.json
```

Run the settings window:

```sh
flatpak run io.github.uutzinger.cosmic-ext-applet-mounter --settings
```

This prototype uses `com.system76.Cosmic.BaseApp//stable` on Freedesktop 25.08
and shares the source manifest's explicit UI/network/D-Bus/filesystem grants.
The [permission rationale](#permission-rationale) covers every `finish-arg`;
the filesystem grants are:

- `xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter`
- `xdg-config/cosmic/com.system76.CosmicTheme.Mode` (read-only)
- `xdg-config/cosmic/com.system76.CosmicTheme.Light` (read-only)
- `xdg-config/cosmic/com.system76.CosmicTheme.Dark` (read-only)
- `xdg-config/cosmic-ext-applet-mounter`
- `xdg-config/systemd/user`
- `xdg-cache/cosmic-ext-applet-mounter`
- `~/.local/state/cosmic-ext-applet-mounter`

The prototype now includes the same narrow COSMIC settings-daemon grant as the
source manifest. This source change does not update an installed Flatpak; its
effective permissions and behavior require the final rebuild/acceptance gate.

The GUI prototype includes the host-visible state bridge for the applet
configuration document. When the applet detects Flatpak mode, it reads and
writes the same native-visible COSMIC config document used by source and Debian
installs:

```text
~/.config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter/v2/document
```

This avoids forcing users to recreate saved connections when switching package
formats. The prototype also maps app-owned durable config/cache/state roots to
host-visible `~/.config/cosmic-ext-applet-mounter`,
`~/.cache/cosmic-ext-applet-mounter`, and
`~/.local/state/cosmic-ext-applet-mounter` when running in Flatpak mode.
Generated user systemd unit writes, app-owned onedriver/onedrive metadata, and
rclone remote/config paths have been live-tested through the GUI prototype, but
the prototype manifest is still not the final reproducible COSMIC repository
submission manifest.

Flatpak mode also applies the host COSMIC theme explicitly for standalone
settings windows, because toolkit/system theme lookup can otherwise resolve
inside the Flatpak sandbox and choose the wrong light/dark mode.

## Final Manifest Scaffold

`io.github.uutzinger.cosmic-ext-applet-mounter.json` is the project-owned
manifest scaffold intended to be copied into the COSMIC Flatpak repository as:

```text
app/io.github.uutzinger.cosmic-ext-applet-mounter/io.github.uutzinger.cosmic-ext-applet-mounter.json
```

The current manifest pins implementation commit
`13e204ec93049587611978c0cfbad0f9e0a5968c` and uses generated `cargo-sources.json`.
Offline contract tests compare every registry archive/checksum with Cargo.lock,
check vendor checksum metadata/config, and verify Git commits and source selectors.
The checked-in vendor list already matches this revision; regeneration is not
needed. The wrapper now validates and fingerprints generated Cargo `inline` and
`shell` entries as well as Git/archive sources. These checks do not prove remote
availability or a completed offline build.
Rebuild and retest after dependency or permission changes; public COSMIC
repository submission remains pending.

## Permission Rationale

The reviewed manifests do not grant `--filesystem=host`. Retained engine/helper
commands use the typed `flatpak-spawn --host` runner, and generated units run host
tools such as `rclone`, `onedriver`, and `fusermount3`
outside the sandbox. The Flatpak itself needs only enough filesystem access to
share applet state and app-owned engine metadata with the host user session.

Desktop control uses the same NetworkManager, user-systemd, and logind D-Bus
adapters in native/source, Debian, and Flatpak builds. There are no production
`nmcli`, `systemctl`, `journalctl`, `systemd-analyze`, proprietary Cisco, `ip`,
`getent`, or `nc` command routes. Manual and scheduled rclone mirrors share the
Rust coordinator; generated units invoke the installed restricted `--run-bisync
scheduled <UUID>` worker (native/Debian) or its fixed `flatpak run --command`
entry point. No bisync shell wrapper remains. Test/install commands are not
shipping runtime requirements.

Retained command families are reconciled in the
[host-command inventory](../../docs/engineering/flatpak-host-command-inventory.md):

| Host command | Why it remains |
|---|---|
| `rclone` | Remote setup/auth/configuration; scoped access checks; private RC readiness, queue/health/refresh/job operations; coordinator preview/sync, recovery markers, checksum verification, and ownership-checked retention. |
| `onedriver` | Online OneDrive authentication; generated service runs the mount engine directly. |
| `onedrive` (retired in 0.5.5) | No executable route remains. Legacy profiles/data are preserved; owned monitor retirement uses systemd D-Bus. |
| `fusermount3` | Clean FUSE detach and confirmed lazy recovery. |
| `mountpoint`, `find`, `cat` | Host mount readiness, path/emptiness/preload checks, and mount-namespace inspection; native mountinfo instead reads local `/proc`. |
| `flatpak document-info` | Explicit Documents portal-v4 compatibility exception, selected only for the unsupported sandbox interface-version result. |
| `flatpak-spawn` | Transport for the above fixed host requests, not a UI command endpoint. |

The fixed inspection adapter accepts only configuration dump or mountinfo,
with a 10-second deadline, 1 MiB/4 MiB per-stream limits, strict validation,
cancellation, and `--watch-bus` host-child lifetime protection. Other engine
operations retain their own typed deadlines/safety policy. Raw config dumps
are not returned in UI messages/errors. Idle rclone mirrors do not run bisync
to scrape pending counts; active runs publish invocation-matched app-owned state.

Keeping `org.freedesktop.Flatpak` permits host execution at the sandbox boundary;
application-side fixed commands do not turn that grant into an OS-enforced
per-executable allowlist. This retained exception still requires submission review.

The published built metadata also includes an inherited `x11` socket, in addition
to `wayland` and `fallback-x11`. No new grant is proposed. Inspect the full built
policy and installed overrides when reviewing effective access; the test computer
overrides were not independently captured.

Current final-manifest `finish-args`:

- `--socket=wayland`: required for COSMIC/Wayland UI.
- `--socket=fallback-x11`: retained as the tested fallback display path.
- `--share=ipc`: paired with fallback X11 and common GUI toolkit behavior.
- `--share=network`: lets the SharePoint editor verify a document-library
  drive with Microsoft Graph and lets in-process VPN readiness checks perform
  route selection, DNS resolution, and bounded endpoint connections. Host
  rclone still handles file access.
- `--device=dri`: retained from the GUI prototype so libcosmic rendering and
  acceleration match the tested settings-window behavior.
- `--talk-name=com.system76.CosmicSettingsDaemon`: pinned libcosmic's
  `dbus-config` integration obtains its settings proxy and watches COSMIC config/
  theme changes. Direct host theme-file reads remain narrowly read-only below.
- `--talk-name=org.freedesktop.Flatpak`: required for
  `flatpak-spawn --host`.
- Documents portal access uses Flatpak's default allowance for
  `org.freedesktop.portal.*`; no additional permission is needed. Sandboxed
  portal v5+ resolves selected document IDs with `GetHostPaths`, while native
  runs use `Info`. A host `flatpak document-info` fallback remains only for
  sandboxed portal v4, whose host-path methods are not sandbox-accessible.
- `--talk-name=org.freedesktop.systemd1`: control the applet's generated user
  services and timers and start the fixed, user-installed
  `cosmic-mounter-openconnect-agent.service` before OpenConnect activation.
  Daemon reload, enable/disable, start/stop/restart, status, failure reset,
  invocation IDs, job completion, unit load-state validation, and sleep-cleanup
  policy checks no longer execute host `systemctl --user` or
  `systemd-analyze`. Generated unit syntax still passes systemd's parser in
  build/tests.
- `--system-talk-name=org.freedesktop.login1`: subscribe to sleep preparation
  and retain the delay-inhibitor descriptor while opt-in Online cleanup runs.
- `--system-talk-name=org.freedesktop.NetworkManager`: discover and control
  saved VPN profiles and read connectivity, interface, and metered state through
  NetworkManager D-Bus. The applet and generated metered-network guard no
  longer execute host `nmcli`.
- `--own-name` and `--talk-name` for
  `io.github.uutzinger.cosmic-ext-applet-mounter.Runtime` and `.Settings`: the
  typed cross-window/runtime IPC interfaces, single runtime ownership, settings
  coordination, and completion notifications. These do not expose command text.
- `--filesystem=xdg-run/systemd/user:create`: create only app-owned Online-unit
  runtime drop-ins for normal, nonforcing sleep cleanup. These safeguards
  persist for the user runtime session. Offline mirror units are not changed.
  The logind grant and runtime drop-in path require final rebuilt-Flatpak
  verification; older
  installed packages will not acquire them from source changes alone.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Mode:ro`: read the
  host COSMIC light/dark preference for standalone settings windows.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Light:ro`: read the
  host COSMIC light theme palette.
- `--filesystem=xdg-config/cosmic/com.system76.CosmicTheme.Dark:ro`: read the
  host COSMIC dark theme palette.
- `--filesystem=xdg-config/cosmic/io.github.uutzinger.cosmic-ext-applet-mounter:create`:
  share the applet's native-visible COSMIC configuration document with source
  and Debian installations.
- `--filesystem=xdg-config/cosmic-ext-applet-mounter:create`: store app-owned
  engine configuration such as isolated onedriver metadata in host-visible paths
  used by host services; existing legacy metadata is preserved.
- `--filesystem=xdg-config/systemd/user:create`: write generated user systemd
  service/timer files where the host user manager can load them.
- `--filesystem=xdg-cache/cosmic-ext-applet-mounter:create`: store app-owned
  caches used by host-run engines such as onedriver/rclone.
- `--filesystem=~/.local/state/cosmic-ext-applet-mounter:create`: store
  coordinator locks/results, invocation-matched progress, and rclone bisync work files.

The applet still uses the desktop folder portal for user-selected mountpoints
and mirror directories. Document-portal paths are resolved back to host-visible
paths before saving so host systemd services can use them.

Resolving a host path does not itself grant the sandbox access to that original
path. Final worker acceptance must prove local mirror/recovery metadata access
under the reviewed grants and portal authorization. If another grant is needed,
record the specific path/operation and a narrowly scoped solution; do not silently
add `--filesystem=host`. Host engine execution and host-visible FUSE mounts do
not prove that the sandboxed Rust worker can access every user-selected path.

October 17 source correction: choose both the mirror folder and an explicit
recovery folder using their Browse buttons. The app remembers and revalidates
the existing portal grants for local coordinator I/O; rclone keeps host paths.
Typed paths alone are insufficient. This does not add filesystem permissions.
The scheduled worker shares the applet's host-visible state root. Initial sync
now requires both anchors after transfer before it can report initialization.
These changes require a new candidate and installed acceptance; missing markers
in an already initialized profile are not silently recreated or reset.

All remaining grants support current source paths; replacing host CLIs does not
remove the D-Bus grants for their replacement APIs or the shared engine/unit
paths. No blanket session/system bus socket or proprietary VPN-service grant is
present. Final acceptance must inspect effective permissions (including BaseApp
defaults), exact source/runtime identity, generated worker/guard units, and live
Tasks (5)/(6) behavior before release. Do not enable `development-probes` in the
shipping build, advance the tag prematurely, or substitute a different base/runtime.

### 0.5.5 development retirement

The legacy engine and browser/manual handoff are removed from current source.
Owned legacy monitor units are stopped/disabled through systemd D-Bus and removed
only after inactive-state verification. No new host command or permission is
introduced; rclone and onedriver still require existing host access. Installed
native unit retirement/data preservation has passed. Debian/Flatpak upgrade and
retained-profile missing-service regression acceptance remain pending and will
be combined. The following 0.5.4
notes record historical behavior, not the current execution surface.

> **0.5.4 development:** new OneDrive Offline mirrors use host rclone >= 1.74.3 and the shared applet OAuth/coordinator path. The `onedrive` host command remains for legacy profiles for one transition release; its removal is a later phase. The WebKit auth helper is retired in 0.5.4. Integrated Flatpak/browser acceptance is still pending. See the [OneDrive Offline contracts](../../docs/engineering/onedrive-rclone-offline-contracts.md).

Version 0.5.4 no longer packages the WebKit OneDrive authentication helper or
requires its Python GI/GTK/WebKit libraries. Existing legacy mirrors retain
browser/manual response-URL reauthentication: start setup, open sign-in through
the desktop portal, then paste and submit the full final Microsoft native-client
redirect URL. The same manual handoff is available in native installations.
Browser request acceptance is not sign-in completion. Transient auth files live
in the private app-owned shared state directory so sandbox and host engine see
the same files; they are removed after setup. Existing legacy synchronization,
credentials and engine dependencies are retained. No host libraries are removed.

October 13 candidate testing confirmed browser/redirect submission and dry-run
validation. Some Microsoft personal-account redirects leave the native-client
URL quickly; retrieving that preceding URL from browser history worked. Treat
the redirect URL as a credential: paste it only into the applet response field,
not logs or support messages. This is a manual workaround, not automatic capture.


## October 19 permission and acceptance reconciliation

Candidate packaging7069ae5/app72661ce was built and installed without changing
its 23 declared grants, runtime/BaseApp or host dependencies. Installation receipt
records matching metadata/commit, no app-specific override and only read-only
user-global theme paths. Earlier final-worker/portal pending statements above
are historical checkpoints: actual selected mirror/recovery access and shared
scheduled worker evidence are accepted by unchanged-code reuse; the replacement
passed the focused VPN Online cycle. Full source/fake/installed boundaries and
unrun limits are in the [acceptance map](../../docs/engineering/release-054-acceptance-audit.md).

The broad org.freedesktop.Flatpak gateway and external host engines remain
explicit submission-review topics. The app's typed restrictions do not make
that permission an OS allowlist. App-owned state paths are scoped; selected
mirror/recovery local I/O uses revalidated portal grants rather than treating a
resolved host path as permission. Host credentials stay with host engines.

Current [COSMIC repository instructions](https://github.com/pop-os/cosmic-flatpak/blob/master/README.md)
cover the applet repository and build entry point. Its
[PR template](https://github.com/pop-os/cosmic-flatpak/blob/master/.github/PULL_REQUEST_TEMPLATE.md)
requires a personal contribution acknowledgment and excludes AI-generated PR
content except the contributor's own app source and manifest. The operator must
author the submission description and assess the actual proposed diff; do not
paste these AI-prepared internal notes into that PR. No acknowledgment or
maintainer approval has been supplied by this work. Internal facts and open
questions: [submission readiness notes](../../docs/engineering/cosmic-submission-audit.md).


### GitHub bundle distribution decision — October 19

0.5.4 will distribute the accepted x86_64 bundle through GitHub Release assets,
with checksums and [local installation instructions](../../README.md#installation-from-github-release-flatpak).
COSMIC repository submission/maintainer exception review is deferred until the
0.5.5 legacy-engine removal and GUI cleanup. Earlier pending submission gates
above do not block independent GitHub distribution. Release approval is separate.
At the October 19 planning point the promotion script handled Debian only.
The October 20 extension below adds selected Flatpak verification/promotion;
final exact-tag artifact identity and publication approval remain required before
advertising the mixed release as available. Clean-profile documented installation
needs focused acceptance; completed unchanged cloud/VPN tests are reused.

### Standalone GitHub release installer

The README uses `scripts/install-flatpak-release.sh` via curl/stdin. It requires
Flatpak already installed and runs as the desktop user, without sudo. It defaults
to 0.5.4; `--version X.Y.Z` selects another published version. CPU detection maps
x86_64/amd64 to x86_64 and aarch64/arm64 to aarch64. It downloads the corresponding
`io.github.uutzinger.cosmic-ext-applet-mounter-<arch>-master.flatpak` and
SHA256SUMS from that GitHub release, verifies the unique selected checksum, then
adds user Flathub, installs/updates Platform 25.08 for that architecture and
installs/updates the user bundle. Future runtime changes require updating this
installer's runtime selection. Host engines/VPN setup remain separate.

Unsupported CPUs or unavailable/mismatched assets fail before Flatpak changes.
Failure after runtime setup can leave the remote/runtime installed; no rollback
or app/service stop is attempted. The script cleans private temporary downloads,
retains the actual exit status and prints a result line. GitHub bundles do not
provide automatic app updates. Close windows, stop connections and remove the
panel applet before replacement. The existing `install-flatpak.sh` remains the
receipt-based internal candidate installer; neither script builds or dispatches
CI. Both hosted CPU bundles have now passed offline checks; installed ARM
acceptance remains separate. Before release assets are public, `--bundle FILE`
uses an exact CPU/app-named local bundle and its sibling SHA256SUMS with the same
checksum/runtime/install steps. It does not download a release in that mode.

### Manual GitHub build workflow

`.github/workflows/build-flatpak.yml` builds x86_64 and aarch64 on native
Ubuntu24.04 GitHub runners. Dispatch is manual and separately authorized; no
push/tag event builds or publishes automatically. It invokes
`scripts/build-flatpak-ci.sh`, which requires the matching clean packaging SHA
and native architecture, prepares runner tools and exact manifest runtime refs,
then calls the existing `build-flatpak.sh` once. Ubuntu's user-namespace restriction
is relaxed only on the disposable CI runner so bubblewrap can build.

Artifacts `flatpak-x86_64` / `flatpak-aarch64` retain the bundle, metadata,
producer receipts, a portable source/run/ref/commit index and SHA256SUMS for
14 days. Separate summary-only diagnostic artifacts are retained on failure.
Original candidate receipt paths refer to the runner; the portable index lists
the downloaded files. No app is installed and no GitHub/COSMIC release is
published. Both real hosted CPU builds passed by October 20; installed ARM
acceptance remains pending.
Flathub's ARM COSMIC BaseApp was confirmed against Platform25.08 on October19;
that read-only observation is not a successful ARM build.

The October 20 publisher extension accepts repeated `--flatpak-run ARCH=RUN_ID`
and verifies the selected successful job, all file hashes, producer/input/runtime
provenance and offline bundle ref/commit/full metadata. Normal publication still
requires each selected build's packaging SHA to match the exact release tag.
It attaches the selected bundles, portable FLATPAK-PROVENANCE.json and combined
SHA256SUMS alongside Debian packages without rebuilding. The workflow exposes
optional run IDs for each CPU; publication requires explicit approval.

For candidate review before a final tag (inspection only):

```sh
python3 scripts/promote_release.py v0.5.4 --inspect-flatpaks \
  --flatpak-run x86_64=39433010848 --flatpak-run aarch64=39382160325 \
  --repo uutzinger/cosmic-ext-applet-mounter
```

This retains verified files in the new private report directory and explicitly
records that acceptance/publication are not performed. These two candidates
have different packaging SHAs; inspection does not waive the final identity gate.


### October 20 hosted AppStream follow-up

Run39382160325 at ba8a341 exported/uploaded aarch64 successfully; the preserved
bundle/provenance is under target/artifacts/candidates/flatpak-hosted-ba8a341/.
Its checksums/source/run identity passed local verification; installed ARM
acceptance is not claimed. x86_64 reached AppStream composition and failed with
file-read-error/filters-but-no-output. The same errors were reproduced locally
with the SVG loader disabled; normal pinned-source metadata/icon composition
passes. The failed runner's detailed hints were not retained, so missing SVG
support is an evidence-backed diagnosis, not a direct package inventory of that
runner. No icon/metadata edits or validation bypass were needed.

CI now explicitly installs appstream, appstream-compose and librsvg2-common and
composes the pinned app commit's metainfo/desktop/SVG before SDK download/build.
The preflight records versions/package inventory and retains issue hints in the
diagnostic artifact. Select architecture=x86_64 in a new manual run to avoid
rebuilding the successful ARM target; both remains the default for full builds.
A mixed-source artifact set must still satisfy explicit release provenance and
acceptance policy before promotion; preserving ARM does not silently waive that
final identity gate. These are runner-only changes, not an app/runtime update.
