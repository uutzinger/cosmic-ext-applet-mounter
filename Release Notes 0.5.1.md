# Cloud Mounter 0.5.1

This release adds guarded unattended SharePoint Offline synchronization,
persists popup switch choices across login, and completes the English Fluent
localization boundary for user-facing interface text.

- **SharePoint Offline scheduling:** After Preview and the confirmed initial
  Sync Now, Start enables a managed background timer. Scheduled synchronization
  checks the approved SharePoint library and folder, local target identity,
  network readiness, metered-network policy, and existing bisync state before
  it runs. Stop disables the schedule without deleting mirror data.
- **Persistent popup switches:** The popup switch is now the saved startup
  choice. Online mounts left on are remounted at the next login; Offline mirrors
  left running restart their schedules. Turning either switch off persists that
  choice. Temporary network or VPN failures do not silently change it.
- **SharePoint Online preload:** SharePoint mounts can use the same bounded,
  cancellable directory-metadata preload lifecycle as other rclone mounts.
  Preload remains disabled by default for every provider.
- **Localization and feedback:** Settings, popup, validation, operation,
  recovery, and SFTP messages now use the Fluent catalog. A release check guards
  against adding new hardcoded user-facing strings in the audited UI paths.
- **Authentication documentation:** The README explains engine-owned
  authentication and links to a new Google Cloud OAuth setup guide for private
  Desktop clients.
- **Build reliability:** The libcosmic dependency source is unified so
  `cargo vendor --locked` no longer fails with duplicate `cosmic-config`
  sources.

The native `amd64` Debian package was built with locked dependencies. Formatting,
locked all-target compilation, Clippy with warnings denied, localization and
credential audits, and the complete automated suite passed: 290 tests passed
and seven explicitly external/live tests were ignored. The package was also
installed, removed, reinstalled, and removed again without altering the
existing user-local applet or its saved configuration.

The official COSMIC desktop category and AppStream `binaries` extension still
produce the documented nonfatal freedesktop validator compatibility notices.

Package SHA-256:

```text
15ba6b3fb86bb32cab82a1c2bbae4d5106cc1d192a0a98f6528d4da9c4903ca8  cosmic-ext-applet-mounter_0.5.1_amd64.deb
```
