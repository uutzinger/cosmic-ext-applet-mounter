# COSMIC submission readiness

Type: Audit

Status: Published 0.5.5 manifest/dependency/permission identity refreshed; scoped x86_64 installed acceptance recorded; COSMIC submission and maintainer decisions pending

Created: October 19, 2026

Last reviewed: October 24, 2026

Scope: Consolidated release step7 in [Task List](../../Task%20List.md).
These are internal facts and questions, not a ready-to-submit PR description.
No external message, PR, acknowledgment or approval has been sent.

October 19 operator decision: publish the 0.5.4 Flatpak as a GitHub Release
bundle, with local installation instructions. Complete legacy-engine removal
and GUI cleanup before pursuing COSMIC submission in 0.5.5. The questions below
remain preparation for that later review; maintainer approval is not a condition
of the independently distributed 0.5.4 bundle. Refresh candidate facts for 0.5.5.

## Current submission baseline — published 0.5.5

The manifest remains unchanged: it already selects the published implementation.
Use these identities for COSMIC preparation; dated preparation sections below
retain history and are superseded by this baseline for current status.

| Item | Verified identity or boundary |
|---|---|
| Release | v0.5.5, published October 22, 2026; annotated tag resolves to packaging cd1b465160d692052f32dbb53010583c8ed11a19. Later master documentation is not the built source. |
| Application pin | 13e204ec93049587611978c0cfbad0f9e0a5968c, fixed full Git commit in the main manifest; not a mutable branch or tag. |
| Manifest/dependencies | packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json and relative cargo-sources.json match the tagged build inputs. Keep both together when preparing app/<app-id>/<app-id>.json. |
| Manifest SHA256 | b9d1094ae1909d297e42c4fba81b0fcc0d3d6a82106d1ffc85bffd350419c9e8 |
| Cargo sources SHA256 | 0c098520c9e8933773ddd935afaa3cb109a78f137593ab9d7ac0ac1693b8815f |
| Published Flatpak build | Run 39842934431, both architectures succeeded; app/io.github.uutzinger.cosmic-ext-applet-mounter/<arch>/master. Runtime commits and producer identities retained in published FLATPAK-PROVENANCE.json. |
| Installed x86_64 identity | b8f91a7bb2c6a0b321ee887718f488131bbe42798cd3944e1f5d07473b929098; operator confirmed exact installed commit and sandbox process. ARM built/inspected; no installed ARM acceptance. |
| Build environment | Freedesktop Platform/SDK 25.08, rust-stable SDK extension, COSMIC BaseApp stable. Successful source builds on both architectures; no runtime substitution. BaseApp/SDK refs are versioned channels, not promises of immutable future runtime contents. |
| Host dependencies | rclone >=1.74.3; onedriver >=0.15.0 for OneDrive Online; FUSE3 for mounts; NetworkManager for network/VPN integration. OpenConnect plugin/dialog and on-demand nm-applet agent apply only to that VPN workflow. No legacy onedrive engine required. |
| Build-only tools | Rust/Cargo/SDK and Python dependency-source tooling belong to builds, not app runtime. External engines/plugins are not bundled in /app. |
| Declared permissions | 23 finish-args, unchanged from the published manifest; narrowly named filesystem/bus entries, plus broad host execution through org.freedesktop.Flatpak. No filesystem=host or blanket bus socket. |
| Built permissions | Inspected built-metadata.ini includes sockets x11, wayland and fallback-x11, shared network/ipc, dri, nine scoped filesystem entries and the documented session/system bus policies. Inherited x11 is part of the packaged baseline; 23 declarations alone are not a full effective-policy description. October 24 VM override check: user-global read-only GTK/KDE theme entries only; system-global and both app-specific scopes empty (details below). |
| Installed evidence | Debian UI/Online/manual mirror smoke; exact x86_64 Flatpak UI/Online smoke and manual business OneDrive synchronization, cloud deletion and local recovery-copy presence. October 24: personal OneDrive scheduled download/upload, saved folder-grant reuse, and fresh Google authorization/Test/Save/mount/listing reported. Separately uploaded DOCX opens with expected content/size; suspected native Google document opens empty at zero bytes. Reuse earlier evidence only at recorded scope. |
| Limits | October 22 scheduled skip superseded by October 24 personal OneDrive transfers; content hashes and other-provider scheduling unverified. Google native-document mount limitation recorded; presentations not live-tested. Google and Box browser authorization and unmount passed; Box Re-Test/Save and mounted/web text agreement reported. Package legacy/absent-unit regression remains open. Google/Box whole-root mirrors remain skipped. |
| SharePoint whole-library initial mirror | October 24: small test library, blank remote subfolder, new mirror/recovery folders; setup and Preview passed, then expected local files reported after initial Sync Now guidance. No explicit content/hash comparison, ongoing two-way sync or recovery pass inferred. |
| SharePoint recovery restoration | October 24 follow-up: disposable text-file upload, cloud deletion/local propagation, recovery-copy preservation and manual copy-back/upload accepted. Operator confirms original text in recovery and restored web copies. No byte/hash verification or fault handling inferred. |

All ten public release asset hashes matched the verified publisher receipt.
See [completion notes](../../Task%20List%20Completion%20Notes.md) for publication
and installed observations, and the [host-command inventory](flatpak-host-command-inventory.md)
for per-family owners/removal criteria. Publication does not establish COSMIC
maintainer approval. Review host gateway, external dependency model, runtime
compatibility and portal-v4 fallback with maintainers; do not change shared
COSMIC machinery or prepare AI-authored PR prose from this internal record.

## Historical 0.5.4 candidate facts

| Item | Prepared identity or boundary |
|---|---|
| App | Cloud Mounter 0.5.4, MIT; io.github.uutzinger.cosmic-ext-applet-mounter |
| Sources | App 72661ce9d705038cdb1878165917f298e0ced8c2; packaging 7069ae5b2182379ebd4f32ba968e4692dbe6c155. Candidates, not an approved release tag. |
| Manifest | packaging/flatpak/io.github.uutzinger.cosmic-ext-applet-mounter.json plus generated cargo-sources.json; proposed repository layout app/<app-id>/<app-id>.json with its relative cargo source file. |
| Runtime | Freedesktop Platform/SDK 25.08, COSMIC BaseApp stable, build-time Rust SDK extension; x86_64/master Flatpak validated. |
| Payload | Rust app, Fluent catalog, desktop file, AppStream and icon; no WebKit helper or WebKit/Ruby build module. |
| Desktop control | NetworkManager/system bus; systemd user/session bus; logind/system bus; Documents/OpenURI portals; typed runtime/settings IPC. |
| Host engines | rclone>=1.74.3 for new OneDrive/shared mirrors and other providers; onedriver>=0.15.0 for OneDrive Online; onedrive>=2.5.10 for retained legacy profiles only. External engines may have their own transitive dependencies. |
| Host VPN setup | NetworkManager OpenConnect plugin/dialog, on-demand nm-applet secret agent and app-owned user agent service; installed separately using the maintained setup guide. |
| Permission boundary | 23 reviewed finish-args; org.freedesktop.Flatpak permits broad host execution. Typed fixed operations constrain the app, not OS authority. Scoped state/unit/cache paths and explicit portal-selected directory grants; no filesystem=host or blanket bus socket. |

## Historical 0.5.4 evidence available for human review

The [acceptance map](release-054-acceptance-audit.md) distinguishes current
source/fake tests, earlier native/prototype tests and installed source Flatpak
observations. Both OneDrive account flows and SharePoint mirrors, portal grant
persistence, worker/recovery access, safeguards/Repair, idle wake and manual VPN
were accepted at recorded scopes. The replacement passed the focused VPN Online
negative/positive/unmount cycle; NetworkManager then reported zero active VPNs.
All six Debian candidates and the Flatpak export/payload were verified.

[Host-command inventory](flatpak-host-command-inventory.md) and
[permission rationale](../../packaging/flatpak/README.md#permission-rationale)
cover retained commands, owners and removal conditions. Legacy live reauthentication/
monitor transfer was waived; active-transfer sleep/VPN-loss, final shared/
pre-existing ownership variants and latest Flatpak busy Repair are not claimed
newly live-passed. Final release verification and publication remain separate.

## Public submission requirements checked October 19

The [official README](https://github.com/pop-os/cosmic-flatpak/blob/master/README.md)
places COSMIC applets in this repository and documents building by app ID. The
[repository build recipe](https://github.com/pop-os/cosmic-flatpak/blob/master/justfile)
loads app/<app-id>/<app-id>.json. These facts support the proposed layout; they
do not certify that our external-host design or chosen runtime will be accepted.

The [PR template](https://github.com/pop-os/cosmic-flatpak/blob/master/.github/PULL_REQUEST_TEMPLATE.md)
requires a personal contribution acknowledgment, including trademark/DCO and
understanding/testing attestations. It excludes AI-generated PR material except
one's own app source and manifest, explicitly covering descriptions. The operator
must independently author the submission description and assess every included
file; these AI-prepared notes must not be copied into the PR. No checkbox is
pre-acknowledged. No claim of current maintainer approval is made.

## Maintainer decisions still needed

- Whether the org.freedesktop.Flatpak gateway and documented fixed host command
  surface are acceptable for this applet, including host configuration/unit paths.
- Whether the external engine/plugin/secret-agent installation model is acceptable; no engine/runtime swap is assumed.
- Whether retaining Platform25.08/COSMIC BaseApp stable and the supported-host
  Documents-v4 fallback is acceptable for a new submission.

These are scoped external-review questions, not implementation defects or
permission requests to silently broaden the manifest. A COSMIC decision does
not substitute for operator release/publication approval. Prepare/submit only
a human-authored contribution after the operator chooses that action.

## 0.5.5 readiness review — October 22

This pre-build checkpoint is historical; the current published baseline above
supersedes its dirty-tree/old-pin/pending-build status. It does not change earlier
package identities or acceptance.

| Item | Current source review and remaining boundary |
|---|---|
| Identity | Cargo/lock and Debian declare 0.5.5; worktree on master at 057b647 is dirty. No immutable 0.5.5 app/package commit selected yet. |
| Manifest pin | Still 72661ce9d705038cdb1878165917f298e0ced8c2, a historical 0.5.4 development source. Must be replaced with the reviewed full 0.5.5 app commit before building; committing packaging produces a separate packaging identity. |
| Dependencies | cargo-sources registry archives/checksums and generated vendor metadata match current Cargo.lock (two Python contract tests pass). enumflags2 was already vendored; no regeneration needed for current lock contents. This does not establish remote availability/offline build success. |
| Runtime | Keep Freedesktop Platform/SDK 25.08, Rust extension and COSMIC BaseApp stable. All four x86_64 refs available in read-only preflight; their exact observed commits are in the private receipt. No runtime change proposed. |
| Permissions | Manifest still declares 23 finish-args. Retiring onedrive does not remove shared engine/host state grants. No blanket filesystem or bus socket added. Effective candidate/BaseApp/override permissions must be inspected after build/install. |
| Engines | Host rclone >=1.74.3 and onedriver >=0.15.0 remain; legacy onedrive is no longer invoked or recommended by Debian. FUSE3 and NetworkManager remain host integration dependencies; OpenConnect plugin/agent applies to that VPN workflow. |
| Host gateway | Default typed runner has eight shipping executable variants; onedrive is removed. org.freedesktop.Flatpak still permits broad host execution, constrained by app policy rather than an OS executable allowlist. Maintainer acceptance remains unresolved. |
| Native acceptance | Retired-unit/data preservation, scoped GUI, OneDrive existing-remote Test, fresh Google/Box setup, selected mirror transfers and failed-Preview recovery are recorded in completion notes. These do not establish final sandbox acceptance. |
| Limits | Google/Box live whole-root mirrors deliberately unrun due account volume; retain local-engine root/recovery and cloud-subfolder evidence separately. SharePoint whole-library live acceptance remains open. |

Read-only preflight succeeded with ready_for_candidate_build=false: packaging
checkout is dirty. Its syntactically valid old app pin is an additional semantic
release blocker found by this review, not a readiness check the wrapper inferred.
Receipt: `target/task-runs/flatpak-preflight-haf64edw/preflight.json`.

### Combined final-package validation checklist

1. Review intended changes, finalize release metadata, select an approved immutable
   app commit, update the manifest pin, then select a clean packaging commit.
   Build the required Debian/Flatpak targets once from that packaging revision;
   retain checksums, source/runtime identities and producer receipts. Reuse those
   bytes for acceptance/tag/publication when source identity matches.
2. Inspect Debian dependency/payload/version and Flatpak app ID, branch, source
   pin, metadata and effective permissions. Do not enable development-probes.
3. With installation approval, preserve rollback identity, close editors and
   ensure only one runtime owner. Use maintained package installers; record which
   exact package/ref was installed. Do not use a debug launch as panel acceptance.
4. Combine upgrade and missing-retired-unit regression on a disposable retained
   legacy profile: proven app-owned unit retires, profile stays unsupported,
   Start/Sync remain unavailable, and data/credential/recovery locations survive.
   Reopen the popup repeatedly with the retired unit absent: no recurring refresh
   warning. Genuine supported-unit failures must still be reported. Ordinary tests
   must not alter live user configuration to construct this fixture.
5. Check real panel/standalone Settings handoff, button states/confirmations and
   Google/Box browser opening in the sandbox. Exercise a small mount and mirror,
   scheduled worker access, portal grants and scoped VPN behavior affected by the
   final package; reuse unchanged earlier checks only at their original level.
6. Record passed, blocked, waived and unrun cases separately. Preserve the root
   cloud limit; do not mirror large accounts for acceptance. Finalize notes after
   applicable acceptance, then obtain tag/push/publication approval and promote
   the accepted artifacts. COSMIC repository submission remains a later action.

### Current public submission rules rechecked

On October 22 the official README still directs COSMIC applets to this repository.
The official PR template still restricts generated submission material, with the
exception for the contributor's own app source/manifest, and requires personal
understanding, testing, trademark and DCO acknowledgment. The operator must author
the PR description/replies independently; this internal audit is not PR text.
Sources: [README](https://github.com/pop-os/cosmic-flatpak/blob/master/README.md),
[PR template](https://github.com/pop-os/cosmic-flatpak/blob/master/.github/PULL_REQUEST_TEMPLATE.md).
The repository justfile could not be fetched in this review; layout evidence above
retains its October 19 scope rather than claiming a new current recipe check.


### Approved development source freeze — October 22

Application source is now b43f31617b8c147f80dd4aff924cb30c2c2676d7. The manifest
was updated from the historical 0.5.4 pin to that exact commit, with packaging
recorded separately. The earlier dirty-tree/old-pin observations above retain
their pre-freeze chronology. AppStream development status and draft notes remain;
no final release tag, package build or acceptance is implied. Finalizing embedded
metadata later changes application inputs and requires updating the app pin
before the final single build set.


### Finalized metadata source — October 22

Application commit 13e204ec93049587611978c0cfbad0f9e0a5968c supersedes the earlier
development source pin. It includes finalized stable AppStream/Debian metadata
and prepared release notes with explicit package acceptance limits. The main
manifest now pins that exact app commit; packaging is committed separately.
Metadata/publication preflight passes, which does not confer installed package
acceptance or publication approval. No build bytes exist for this identity yet.

## Contribution-rule recheck — October 23, 2026

Documentation review complete; submission and maintainer acceptance remain open.
Read the [release/packaging procedures](release-packaging-procedures-contracts.md)
before this review. No candidate build, installation or submission was performed.

Direct GitHub API lookup resolves cosmic-flatpak master to
`f702d691ce9060f50c2908d36a7d83b6676703a8`. Fetched the following official files
at that revision, rather than relying on cached browser snapshots:

- [README](https://github.com/pop-os/cosmic-flatpak/blob/f702d691ce9060f50c2908d36a7d83b6676703a8/README.md):
  repository covers COSMIC applets and other packages unsuitable for Flathub;
  ordinary COSMIC applications should first try Flathub.
- [PR template](https://github.com/pop-os/cosmic-flatpak/blob/f702d691ce9060f50c2908d36a7d83b6676703a8/.github/PULL_REQUEST_TEMPLATE.md):
  personal acknowledgment covers trademark policy, contribution rights, accurate
  commits, understanding/review responses and tested behavior. Its AI exception
  covers the contributor's own app source and manifest. It does not extend to
  descriptions, replies or shared repository machinery.
- [Build recipe](https://github.com/pop-os/cosmic-flatpak/blob/f702d691ce9060f50c2908d36a7d83b6676703a8/justfile):
  `just build <app-id>` reads `app/<app-id>/<app-id>.json`, builds for the host
  architecture using sandboxed flatpak-builder with dependencies from Flathub,
  and writes `build/app/<app-id>/<arch>` and `log/app/<app-id>/<arch>.txt`.
  This closes the October 22 recipe-fetch gap; the recipe was inspected, not run.

Also fetched the linked [general contributor guide](https://github.com/pop-os/pop/blob/master/CONTRIBUTING.md),
[trademark policy](https://github.com/pop-os/cosmic-epoch/blob/master/TRADEMARK.md)
and [DCO 1.1](https://developercertificate.org/). The general guide excludes AI
contributions; the repository-specific template supplies the narrow app/manifest
exception. Keep this internal audit outside the submitted diff. The operator
must personally assess provenance, licensing and every acknowledgment; none is
checked on their behalf. Third-party branding must not imply official status.
These sources do not approve this app's host-command/dependency/runtime model.

The operator's [PR draft](../../Cosmic%20Aplet%20Repository%20Pull%20Request.md)
is preserved byte-for-byte. Its existing appendix identifies AI assistance;
the operator must assess the actual submission against the exception before
copying any material. This review neither rewrites nor certifies the draft.
Raw source snapshots and URL/SHA256 index are local evidence at
`target/cosmic-contribution-review-2026-10-23/sources.json` (not PR material).

### Reused evidence and changed-input checks

| Evidence | Reuse and limit |
|---|---|
| [Flatpak inspection receipt](../../target/task-runs/release-promotion-4fdzna92/summary.json) and [hosted run 39842934431](https://github.com/uutzinger/cosmic-ext-applet-mounter/actions/runs/39842934431) | Existing successful inspection of both architectures, packaging cd1b465, app pin 13e204e. Receipt success was reread; no new build or full artifact inspection. |
| [Release preparation receipt](../../target/task-runs/release-prepare-v055/summary.json) | Existing successful six-target Debian reuse at cd1b465; no native/Flatpak installed acceptance implied. |
| [Published provenance](https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v0.5.5/FLATPAK-PROVENANCE.json) and [checksums](https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v0.5.5/SHA256SUMS) | Previously verified public artifacts; local provenance reread for source, pins and manifest/dependency hashes. No repeat download or hash of every package. Publication observations remain in the [completion record](../../Task%20List%20Completion%20Notes.md#v055-published-and-public-bytes-verified--october-22-2026). |
| [Exact installed identity](../../Task%20List%20Completion%20Notes.md#final-flatpak-installed-identity-confirmed--october-22-2026) | Reuse x86_64 b8f91a7 recorded UI/Online/manual business OneDrive sync, deletion propagation and recovery-copy presence only. No new installed test. |

Current manifest and cargo-sources hashes still equal the published baseline
above and local published provenance. Compared them against v0.5.5: no changes.
Post-release edits affect documentation, installers and screenshots, not the
manifest's pinned app or vendored sources. Rebuilding the published bundles for
this rules/doc review would not add acceptance evidence. Changed documentation
gets link/hygiene and whitespace checks; unchanged code gates are reused.

When the operator prepares the destination app directory, compare both copied
JSON files to these hashes and check relative dependency resolution. Moving the
files has not yet been tested in the COSMIC checkout. A destination repository
build validates its integration and is separate from reuse of these release
bundles. Any source, dependency, runtime/BaseApp, grant or payload change needs
affected build/inspection and installed checks; do not substitute acceptance
from the old bundle. No shared build-system change is currently identified.

### Remaining installed/package acceptance

- October 24 update: final x86_64 personal OneDrive scheduled download and upload
  are now accepted by operator report. This supersedes the October 22 skip for
  this scoped check; other providers, exact timing and content hashes are not
  established. See the [scheduled-sync evidence](../../Task%20List%20Completion%20Notes.md#vm-personal-onedrive-scheduled-download-and-upload-accepted--october-24-2026).
- Package legacy-profile upgrade/absent-retired-unit regression was not run:
  October 24 operator confirms no retained abraunegg/onedrive settings on the VM.
  Retain an explicit untested limitation; native/fake tests and current-profile
  package smoke do not establish legacy upgrade compatibility. Revisit if a
  suitable disposable legacy fixture/test environment becomes available.
- October 24: fresh Google and Box browser authorization, Test/Re-Test, Save,
  mount/file access and unmount now accepted at their recorded scopes.
  Small SharePoint whole-library setup/Preview/initial local-file arrival now
  accepted by operator report; explicit content/hash checks and ongoing two-way
  behavior remain outside that result. Google/Box whole-root mirrors remain
  skipped; use small approved disposable resources for any new live test.
- ARM packages were built/inspected but have no installed acceptance; operator
  has no ARM64 test computer.
- October 24 follow-up: SharePoint disposable-file recovery restoration and
  original-text agreement are now accepted; no checksum or automatic-restore
  claim for that text-file restoration test.
  October 24 VM feasibility attempts did not yield a working idle suspend/wake
  baseline: initial DRM/framebuffer workers blocked; a later timed s2idle trial
  had no observed RTC/key wake and guest agent became unavailable. Active-transfer
  sleep testing remains blocked by VM sleep/wake reliability. The VM was recovered;
  independent VPN authentication, protected-resource access and disconnection
  subsequently passed by operator report. Applet-driven VPN connection, mount
  accessibility and file listing then passed by operator report. Following
  toggle-off guidance, operator confirms VPN no longer connected; accept VPN
  release at that scope. Separate file-open/content and explicit unmount-completion
  results are not supplied. Subsequent VPN-only SMB upload test shows write timeout,
  retries and success on try 4 after reconnection; direct server-read SHA256 now
  matches the preserved 128 MiB source. Accept scoped SMB VPN-loss recovery and
  byte integrity. Other-provider fault coverage remains untested; no applet cause
  is established for the VM sleep/display failure.

Before submission the operator must describe these limits accurately and obtain
maintainer decisions on host execution, externally installed engines/plugins and
runtime/portal compatibility. No new pass or blanket waiver is inferred.

### Test VM override baseline — October 24, 2026

Operator supplied a screenshot of user-global override output and separately
confirmed empty output for system-global and both app-specific commands:

| Scope (`flatpak override ... --show`) | Observed overrides |
|---|---|
| User global (`--user`) | `xdg-config/gtk-3.0:ro`, `xdg-config/gtk-4.0:ro`, `xdg-config/kdeglobals:ro`, `xdg-data/color-schemes:ro` |
| System global (`--system`) | None reported |
| User app-specific (`--user <app-id>`) | None reported |
| System app-specific (`--system <app-id>`) | None reported |

App ID: `io.github.uutzinger.cosmic-ext-applet-mounter`. This captures the four
override scopes on the test VM; no broad host/home or mirror-folder override is
shown. Read-only theme overrides do not explain mirror-directory access. Folder
picker grants are separate and were exercised by the reported personal OneDrive
checks. No permission was changed, no overrides reset, and no exhaustive audit
of portal grants or all effective permissions is claimed by these commands.

### Google-native document mount limitation — October 24, 2026

Operator observed an original without a web filename extension appear as a
zero-byte DOCX and open empty in LibreOffice. A browser-downloaded, renamed and
reuploaded DOCX opened with correct content and size through the same mount.
Native Google MIME type is inferred, not independently captured. Live Slides/
Sheets behavior, editing/upload through the mount and content hashes were not
tested. Google authorization and regular-file access pass at the reported scope;
native-document opening does not pass.

[rclone documentation](https://rclone.org/drive/#importexport-of-google-documents)
confirms export suffixes; its [limitations](https://rclone.org/drive/#limitations-of-google-docs)
describe unknown sizes and application-dependent empty mount reads. The
[1.74.3 backend](https://github.com/rclone/rclone/blob/v1.74.3/backend/drive/drive.go#L4286-L4295)
requires import formats to update native documents. `src/providers.rs` enables
full VFS caching but no import/export override or read-only enforcement. Existing
remote configuration could override defaults; it was not inspected on the VM.
See [user-facing limitations](../../README.md#conflict-recovery-and-limitations).

### Google unmount and Box file observation — October 24, 2026

Operator reports Google unmounted without problems; accept Google Online
unmount at that scope. In response to the Box test procedure, operator reports
opening a VM test file and matching newly created text between the mount and
web browser. Accept reported Box mounted/web text agreement; no checksum or
specific write-direction assertion. Operator subsequently confirms Re-Test and
Save passed and Box unmounted cleanly, then explicitly confirms a new connection
completed browser authorization. Fresh Box authorization and the reported Online
lifecycle are accepted. No repeated live test or new artifact is implied.

### SharePoint content-verified recovery restoration — October 24, 2026

After a disposable local text file uploaded with expected web content, operator
deleted that file in SharePoint, observed local deletion and found the recovery
copy. Following manual copy-back to the mirror and Sync Now, the file reappeared
remotely. Operator explicitly confirms both recovery and restored web files
contain the original test text. Accept this content-verified manual restoration
cycle on the identified x86_64 Flatpak. No byte/hash comparison, other-filetype
coverage, automatic restoration, scheduled recovery or sleep/VPN fault pass is
inferred. Earlier presence-only observations retain their historical scope.

### SMB active-upload VPN loss and byte integrity accepted — October 24, 2026

On the identified 0.5.5 x86_64 VM Flatpak, operator copied a new 128 MiB file to
a disposable folder on VPN-only SMB, then disconnected the tunnel. The applet
reported VPN off and offered Repair. Manual VPN reconnection cleared Repair
without invoking it or remounting. Engine journal shows an interrupted write,
three failed attempts with retry backoff, then upload success on try 4.

Operator subsequently ran `rclone hashsum SHA256 <remote-test-file> --download`
with bounded retries/timeouts outside the mount's VFS. Screenshot confirms the
server bytes hash to the preserved source SHA256:
`3f32cc4309c5a4e6ce59f26e6bed5c483717b46141c3d21b956769c7f838c226`.
Accept this upload-interruption/recovery and integrity case. It establishes
neither byte-offset resumption nor automatic VPN reconnection, pending-count UI
correctness, fault handling on other providers or successful sleep/wake.
Synthetic source and test copies are retained; no cleanup has been performed.
