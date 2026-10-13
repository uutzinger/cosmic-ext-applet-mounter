# Cloud Mounter 0.5.0

This release adds SharePoint document-library connections and strengthens mount,
sync, and configuration recovery.

- **SharePoint Online:** Connect an existing verified rclone document-library
  remote or set one up through Microsoft sign-in. The applet checks the saved
  site, library, drive, and selected folder before mounting. Directory preload
  is optional and off by default.
- **SharePoint Offline:** Preview and Sync Now operate on a selected folder with
  scoped recovery. Disposable tests covered uploads, downloads, conflicts,
  deletion recovery, interrupted transfers, and SharePoint Office-file rewrites.
  Background scheduling remains disabled pending unattended safety checks.
- **Connection management:** Add a compact Reorder Connections window, pending
  work indicators, mountpoint preflight checks, clearer failed-service repair,
  and a guarded configuration recovery utility.
- **Setup:** Add a host-dependency installer and update the provider and Flatpak
  documentation. The Flatpak manifest remains a local packaging prototype; no
  public COSMIC Flatpak repository release is claimed.

The native `amd64` Debian package was built with locked dependencies. Formatting,
compilation, Clippy, and the required test suite passed. The official COSMIC
desktop category and AppStream binaries extension still produce the documented
nonfatal freedesktop validator warnings.
