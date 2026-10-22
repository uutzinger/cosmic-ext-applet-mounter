# COSMIC submission readiness

Type: Audit

Status: 0.5.5 readiness review recorded; immutable candidate and package acceptance pending; COSMIC submission separate from release

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

## Evidence available for human review

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
- Whether the external engine/plugin/secret-agent installation model and legacy
  transition dependency are acceptable; no engine/runtime swap is assumed.
- Whether retaining Platform25.08/COSMIC BaseApp stable and the supported-host
  Documents-v4 fallback is acceptable for a new submission.

These are scoped external-review questions, not implementation defects or
permission requests to silently broaden the manifest. A COSMIC decision does
not substitute for operator release/publication approval. Prepare/submit only
a human-authored contribution after the operator chooses that action.

## 0.5.5 readiness review — October 22

This section supersedes the historical candidate facts for new release planning;
it does not change the identity or acceptance of any earlier package.

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
