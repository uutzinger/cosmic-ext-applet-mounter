# 0.5.3 release source review

Type: Audit
Status: In progress — not final source approval
Created: October 12, 2026
Last reviewed: October 13, 2026

## Scope and sources

Supports the final native/packaging gate in [Task List](../../Task%20List.md).
Authoritative inputs are the complete current worktree, Cargo/lock metadata,
Debian packaging, Flatpak manifests, release workflows/scripts, and
[draft notes](../releases/v0.5.3.md). Requirements and the maintained host-command
inventory retain their own implementation and package-acceptance boundaries.

## Current findings

- The migration is a large multi-session change: 55 tracked changed/deleted
  paths plus 28 untracked files at initial review. Untracked source modules,
  tests, agent guidance and moved documents are required source, not disposable
  build output. Do not finalize with only `git commit -am`.
- Cargo/lock/Debian versions match 0.5.3; AppStream is development and notes
  are Draft. The lock delta adds direct libc ownership without changing its
  resolved version. Existing published download/screenshot URLs remain 0.5.2.
- Tag pushes and ordinary manual builds cannot publish through the updated
  workflow. Explicit publication requires the six builds and the stable-version/
  finalized-notes preflight. This is source/fixture verification, not hosted CI.
- Native 0.5.3 amd64 candidate compilation/package tests and actual installed
  upgrade/UI/preservation receipts passed. Broad all-feature tests for the latest
  source passed October 12: 385 passed, zero failed, 12 ignored. Default package
  tests and all-feature coverage remain distinct from ignored live tests.
- The final Flatpak still pins v0.5.2 intentionally. It is not migration evidence.
  Remaining effective-permission, coordinator-path, helper and VPN gates cannot
  be waived by native success.
- Updated stale draft-note text to reflect the actual versioned upgrade receipt.
- Filename-based credential-file checks found no matching private key/token/
  environment/rclone-config artifacts outside ignored build output. This is not
  a complete content-level secret audit or proof that all documentation is public.

## October 12 content/reference review result

- Enumerated 114 existing tracked/nonignored-untracked source files, excluding
  deleted old paths and ignored package/cache/log output. Text scan covered
  private-key, GitHub-token, JWT, AWS-key and literal credential signatures.
  Two literal matches are synthetic passwords in test-only SMB/SFTP fixtures;
  no confirmed credential was found. Binary fixture contents, remote history and
  novel secret formats are outside this signature scan.
- All inspected inline local Markdown file destinations and heading anchors
  resolve. External URLs, reference-style links, public-site rendering and
  remote availability are not established by this check.
- Moved 0.5.0 and 0.5.1 notes are byte-identical to their historical HEAD versions.
  New documentation/modules/tests are required intended source. Ignored build
  output is not part of source finalization.
- Production entry-point spot checks confirm the restricted bisync request
  parser; fixed inspection operations/budgets; default executable feature gates;
  and test-only legacy nmcli/systemctl/systemd-analyze/private-bus paths. Settings
  and authentication helper launches remain separate documented UI actions.
  This checks these architectural boundaries, not every changed line.
- Organization-specific URLs/home paths remain in project history and some
  test examples. They are not authentication secrets, but are a public-document
  privacy consideration. Retain historical receipts rather than silently
  rewriting them; publication review must acknowledge this retained context.

Private diagnostic report: `/tmp/cloud-mounter-source-review-20261012.json`.
No production code or accepted runtime behavior changed during this review.

## Immutable candidate-to-final sequence (proposed)

1. Complete source review and checks, then obtain approval to commit all intended
   source and moved/new documents. Keep package outputs/logs out of that commit.
2. With separate push/tag approval, use an immutable candidate commit or
   prerelease tag such as `v0.5.3-rc1` for candidate Flatpak/hosted acceptance.
   Do not use `scripts/build_release.sh` to create the final stable tag while
   notes and AppStream metadata are still provisional.
3. After candidate acceptance, finalize notes/AppStream and disposition decisions
   in a new commit, then create the final immutable `v0.5.3` tag with approval.
   Never move the candidate or final tag to conceal a later change.
4. Build the final tag and verify identity plus any behavior invalidated by the
   candidate-to-final delta. Implementation/permission changes require affected
   lifecycle tests again. Only separately approved manual publication follows.

The candidate is not the final artifact. Final Flatpak identity/acceptance stays
open even if candidate tests pass. This sequence is a proposed resolution of the
current draft-notes/immutable-tag ordering conflict; no tag or push is authorized.

## Remaining review

October 12 staged inclusion review: all intended new modules/tests/moved documents
are present; no target/artifacts/Flatpak build-output paths are staged. The 88
runtime/build/test/packaging source files compared against the tested native
0.5.3 snapshot are byte-identical. Remaining differences are documentation review
receipts. The staged whitespace check exposed three Markdown hard-break spaces
in the newly included inventory; replaced them with blank-line separation.
Restage that fix and the review receipts before final staged check/commit.

Final cached whitespace check passed after user restaging. Candidate source was
committed locally as `efbf73e` on October 12 under explicit approval. No tag/push
was made. Implementation identity is immutable; any receipt follow-up is a
separate documentation commit, not an amendment or claim of final acceptance.

- Latest all-feature test gate is recorded as passed; ignored live tests are not
  counted as passes and retain their separate prior/remaining acceptance gates.
- Focused content/reference and architecture-boundary checks above passed with
  stated coverage limits; they are not a full security or line-by-line audit.
- Verify a complete clean source snapshot includes compile-time fixtures and
  `.github` workflow input required by the new Debian contract test.
- Staged inclusion/candidate-source comparison is complete; final restaging and
  whitespace check are still required. Commit/tag/push approval remains separate;
  the present
  worktree is not an immutable release identity and has not received full line-by-
  line/security review merely because tests pass.

Detailed receipts belong in [completion notes](../../Task%20List%20Completion%20Notes.md).

## October 13 WebKitGTK compatibility probe

Configure-only SDK 25.08 probe uses WebKitGTK 2.54.1 from the
[upstream release directory/checksum](https://webkitgtk.org/releases/webkitgtk-2.54.1.tar.xz.sums).
Source SHA-256: ea0bbb02dbdbc596874a4e7ad35b66645b3e0a232bd0e4081de5ed92eb0a397d.
GTK3/4.1 API is selected with USE_GTK4=OFF; introspection and bubblewrap
sandboxing remain enabled. No source/app installation or runtime upgrade.

SDK GCC 15.2, GLib/GTK/libsoup/ICU and subsequent core library checks succeed.
The first probe lacked Ruby; official Ruby 3.4.11 source matches its published
SHA-256 5c22be44524312b3d433d68739bcc530633b1da5ef8ba0afa0a37680da17d3de.
Its minimal interpreter built in the temporary SDK-bound probe directory, not
on the host or in the shipped app. Production build must provide the complete
Ruby tooling needed by generators and remove it from the final payload.

Second probe stopped at optional libbacktrace; with USE_LIBBACKTRACE=OFF the
next blocker is missing bwrap. SDK inspection also confirms xdg-dbus-proxy is
missing; libseccomp 2.6.1 and libcap 2.78 are available. Package the actual sandbox
tools as runtime dependencies; do not disable sandboxing to produce a misleading
configure success. No evidence requires a newer SDK yet. Configure completion,
full build, Platform imports and actual rendering/authentication are unverified.

Artifacts/logs: target/webkitgtk-2541-compat-20261013/. These are local generated
probe outputs, not committed dependencies or accepted packaging modules.

### Configure completed with source-built sandbox tools

October 13: upstream release metadata/digests pin bubblewrap 0.13.0 and
xdg-dbus-proxy 0.1.9. Both built with SDK 25.08 and load for version probes in
Platform 25.08. Candidate [module definitions](../../packaging/flatpak/modules/README.md)
preserve license payloads and add no permissions/setuid steps; they are not yet
wired into the final app manifest. Probe executables total about 178 KiB,
excluding libraries; this is not the final helper/package footprint. Upstream
tool tests were disabled for the minimal probe and are not claimed as passes.

WebKit configure/generation now succeeds with ENABLE_BUBBLEWRAP_SANDBOX and
ENABLE_INTROSPECTION ON, USE_GTK4 OFF (4.1 API), optional libbacktrace OFF, and
bundled unifdef/sysprof-capture. No WPE module was required by this GTK configure
result. Sandbox tools use actual temporary SDK-built paths, not host binaries.
Four packaging source contracts pass. No newer SDK requirement was encountered.

Next compile proof must complete Ruby generator tooling and build WebKit with
bounded parallelism. Full sandboxed helper acceptance still requires Platform
imports, nested process behavior, rendering, redirect capture and negative tests.
Final packaging must use /app executable paths, include PyGObject, retain source/
license/security maintenance, and remove build-only Ruby. Installed app unchanged.

## October 13 authentication strategy reconsideration (proposed, not implemented)

User requested comparison of lighter alternatives before committing to bundled
WebKit. Reviewed Task List lines 746–778/1294–1327 and completion notes 928–960/
1523–1558, plus current October 13 manual handoff receipts. These already record
local-redirect investigation/implementation, Python helper implementation/live
checks, personal-mirror lifecycle tests and browser-history fallback. They do not
record the outcome/reason for every rejected authentication mechanism; do not
claim that loopback or device-code support is newly discovered or never tried.

Current installed host engines: onedrive v2.5.11-1+np1+1.1; rclone v1.75.0.
[Versioned upstream usage](https://github.com/abraunegg/onedrive/blob/v2.5.11/docs/usage.md)
documents automatic host-browser loopback auth and device auth. Versioned source
confirms --auth-files bypasses the loopback branch; our current Flatpak default
explicitly selects that file handoff. Desktop detection uses display/session,
runtime directory, uid and home, not a TTY requirement. Thus a host setup script
could invoke the existing engine's complete browser/callback authorization in
an isolated app-owned confdir, without WebKit or app-owned OAuth implementation.
Generating only another URL would not solve the fast-redirect capture problem.
This specific host-script route is a proposal, not a live success claim; any
new test must explain its environment/argument difference from previous trials.

Device auth is not a universal substitute: upstream documents personal-account
limitations for its registration. Organizational admin consent is separate from
UI technology; neither a helper, script nor device code grants missing tenant
approval. rclone also remains subject to account/tenant policy.

[Rclone's OneDrive backend](https://rclone.org/onedrive/) supports personal and
business drives, and [bisync](https://rclone.org/bisync/) provides the same engine
family as our SharePoint mirrors. Current app code rejects Provider::OneDrive in
rclone backend/planner/coordinator routing, and OfflineMirrorConfig has no engine
selector. Supporting it is feasible but requires an explicit persisted offline-
engine choice, auth/remote setup, unit/status/repair/controller routing and tests;
not simply a backend-name substitution or automatic migration of existing data.
Online onedriver behavior must remain separate.

Recommendation for decision: defer bundled WebKit; prioritize rclone OneDrive
offline support using the existing Rust coordinator, retain abraunegg monitoring
as an optional engine with external host authorization, and evaluate that minimal
host setup before restoring an embedded-browser dependency. Existing configurations,
tokens and data stay unchanged. Rclone remains scheduled/one-shot; adding a local
watcher would be separate future work and would not eliminate remote polling.
Never let two engines write the same mirror. No engine conversion or new auth,
script implementation, build cancellation, module removal, commit/tag/push was
performed by this read-only investigation.

## October 13 three-engine consolidation assessment (proposed)

User requested an architecture decision before terminal authentication tests.
No engine conversion, authentication test or installed change is authorized by
this assessment. Earlier optional-engine proposal above remains historical,
not an adopted requirement.

| Engine | Strongest justification for retention | Consolidation trade-off |
|---|---|---|
| rclone | Common provider/auth/mount/RC/recovery pipeline and Rust bisync coordinator already implemented. | OneDrive routing/setup/acceptance still must be added; existing SharePoint success does not prove personal/business OneDrive equivalence. |
| abraunegg/onedrive | True local mirror with inotify-driven changes and supported remote notification/delta behavior in monitor mode. | Scheduled bisync is not continuous monitoring; external host authorization removes WebKit burden but retains engine/database/state/repair/test maintenance. |
| jstaf/onedriver | On-demand filesystem with documented cached read-only offline access and our recorded successful corporate setup. | Rclone provides on-demand VFS caching, but exact offline/reconnect/conflict behavior needs acceptance rather than assumed parity. |

Primary references: [rclone mount](https://rclone.org/commands/rclone_mount/),
[OneDrive backend](https://rclone.org/onedrive/),
[onedriver features](https://github.com/jstaf/onedriver), and
[onedrive monitor architecture](https://github.com/abraunegg/onedrive/blob/v2.5.11/docs/client-architecture.md).
Rclone 1.75 source implements OneDrive delta-based change polling for mounts;
this does not turn finite bisync into an event-driven offline daemon.

Assessment: a rclone-only target is defensible if scheduled offline sync plus
explicit Sync Now meets product intent. Optional engine selection does not
eliminate engine-specific lifecycle/auth/version/repair branches or test matrices.
Onedriver has pragmatic existing acceptance value, but no demonstrated mandatory
requirement currently justifies maintaining it forever alongside a validated
rclone replacement. Abraunegg has a stronger distinct feature case; retain it
only if continuous/event-driven mirroring is a hard requirement, not solely to
avoid making an engine decision.

Simplification would remove two engine runtime recommendations, their auth/helper
paths and WebKit bundling, and eventually consolidate service/status/repair policy.
It does not remove FUSE, justified host engine execution, systemd/NetworkManager
D-Bus or optional VPN dependencies; it cannot promise store acceptance or tenant
consent. Fewer engines also do not prove lower CPU/power: sync cadence, tree size
and metadata/hash work determine that and need measurement.

Recommended decision sequence: agree whether continuous mirroring is required;
prototype personal/business OneDrive rclone Online and Offline on disposable
trees; verify auth, editing/uploads, remote changes, network-loss/pending-cache
handling, conflicts/deletions/recovery and repair; then adopt requirements and
plan explicit migration. Do not silently reinterpret old account labels as rclone
remotes, reuse engine databases/caches/tokens, or run two engines on one tree.
Protect active legacy units and pending data until a controlled migration succeeds.
Legacy compatibility may be transitional without offering permanent engine choices
for new connections. Keep Online/Offline modes distinct; no engine removal yet.

## October 13 adopted release-channel decision

User deferred final Flatpak packaging and engine decisions to 0.5.4. Release
0.5.3 proceeds with current engines through source/Debian channels; no engine
conversion or WebKit bundling is part of that release closure. Native source
still selects interactive engine auth, and explicit manual handoff retains the
installed graphical helper. Host Python GI/Gtk 3/WebKit2 4.1 imports pass.
Flatpak's forced manual fallback does not apply to native installations.

Task List retains completed receipts and marks transferred unfinished sandbox
steps [o] with an authoritative 0.5.4 reference. Full latest native rebuild/
installed helper checks and six-platform CI remain before final metadata/source
acceptance. Completed-development commit/push is authorized; tag/publication is
separate. Existing Flatpak runtime switch/rollback must be restored to native
for those checks, with configuration/data preserved. Prior candidate-to-final
Flatpak ordering proposals remain history, not 0.5.3 prerequisites.
