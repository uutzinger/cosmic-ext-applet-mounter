# COSMIC submission readiness

Type: Audit

Status: Internal technical preparation complete; external submission deferred to 0.5.5, outside the 0.5.4 GitHub release gates

Created: October 19, 2026

Last reviewed: October 19, 2026

Scope: Consolidated release step7 in [Task List](../../Task%20List.md).
These are internal facts and questions, not a ready-to-submit PR description.
No external message, PR, acknowledgment or approval has been sent.

October 19 operator decision: publish the 0.5.4 Flatpak as a GitHub Release
bundle, with local installation instructions. Complete legacy-engine removal
and GUI cleanup before pursuing COSMIC submission in 0.5.5. The questions below
remain preparation for that later review; maintainer approval is not a condition
of the independently distributed 0.5.4 bundle. Refresh candidate facts for 0.5.5.

## Candidate facts

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
