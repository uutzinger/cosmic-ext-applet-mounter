# COSMIC submission readiness

Type: Audit

Status: Published 0.5.5 manifest/dependency/permission identity refreshed; scoped x86_64 installed acceptance recorded; COSMIC submission and maintainer decisions pending

Created: October 19, 2026

Last reviewed: October 22, 2026

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
| Built permissions | Inspected built-metadata.ini includes sockets x11, wayland and fallback-x11, shared network/ipc, dri, nine scoped filesystem entries and the documented session/system bus policies. Inherited x11 is part of the packaged baseline; 23 declarations alone are not a full effective-policy description. User/global overrides on the remote test computer were not independently captured. |
| Installed evidence | Debian UI/Online/manual mirror smoke; exact x86_64 Flatpak UI/Online smoke and manual business OneDrive synchronization, cloud deletion and local recovery-copy presence. Reuse earlier native/0.5.4 evidence only at recorded scope. |
| Limits | Final Flatpak scheduled execution deliberately skipped; live Google/Box whole-root mirrors skipped. Package-specific legacy/absent-unit regression, SharePoint whole-library and fresh Google/Box browser consent in the final sandbox remain unrun. These are disclosed limits, not new passes. |

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
