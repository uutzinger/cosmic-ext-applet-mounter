# Cloud Mounter Tooltip Review

Use this file to review tooltip wording without editing Rust code.

Edit the **Reviewed text** blocks only. Leave the IDs stable so the approved
wording can be mapped back into `src/app.rs`.

## Review Rules

- Keep tooltips short enough to read quickly.
- Prefer user-facing behavior over implementation detail.
- Mention destructive or data-risk behavior clearly.
- Data-safety warnings from README's "Do not" list should use a bold warning
  line in the applet tooltip, followed by normal explanatory text.
- Mention where credentials are stored when authentication is involved.
- Avoid repeating visible button or field labels unless needed for clarity.
- Keep dynamic placeholders such as `{provider}`, `{connection}`, `{remote}`,
  `{backend}`, `{status}`, `{action}`, and `{reason}` intact when present.

## General Settings Layout and Preload Policies — September 15–16, 2026

The popup keeps title/gear, status/notices and connections. Add Connection,
Refresh occupy the top row of a standalone General Settings window, with
Preload and Sleep settings below.
Existing `popup.add_connection` and `popup.refresh` IDs below remain stable for
review history; their controls are now hosted in General Settings. Remote-name guidance from the preceding review stays
in the connection editor, not General Settings. In Modify, where Create is
hidden, the implemented help describes existing remotes without instructing users to press Create.

### popup.settings

Reviewed text:

```text
Open Settings to add connections, refresh the applet, and configure sleep and directory preload behavior.
```

Accessible label: `Settings`. Align the clickable gear to the right edge of the
title row; keep the title left-aligned. Use the shared one-second tooltip delay
used by connection-list help instead of the icon button's immediate tooltip.

### settings.unmount_before_sleep

Visible label: `Unmount when sleep`.

Reviewed text:

```text
Cleanly unmount applet-managed Online connections before system sleep. Offline mirrors and synchronization remain untouched. Busy mounts may outlast the system’s sleep delay; caches are preserved.
```

### settings.restore_after_wake

Visible label: `Restore after wake up`.

Reviewed text:

```text
Restore only previously active, still-enabled Online connections after network and VPN readiness checks pass. Enable Unmount when sleep first; the saved restore preference is retained while disabled.
```

Keep this toggle visible but disabled when Unmount when sleep is off, retaining
its saved preference. Show listener status and cleanup details as plain text
below “Applies only to Online connections.”, without a separate heading;
keep a concise unresolved-warning indication in popup status.

### settings.preload

Visible heading: `Directory preload`.

No standalone descriptive sentence is shown below the heading. Provider and
field details are available from their delayed hover help.

### settings.google_preload

Visible label: `Google Drive`.

Reviewed text:

```text
Enables a recursive Google Drive VFS directory-cache refresh in the background after mounting. File contents are not downloaded.
```

### settings.onedrive_preload

Visible label: `OneDrive`.

Reviewed text:

```text
Enables a directory-only OneDrive walk in the background after mounting so folders open faster. File contents are not downloaded.
```

The legacy ID `settings.onedriver_preload_seconds` maps to this provider row
during implementation and configuration migration.

### settings.box_preload

Visible label: `Box`.

Reviewed text:

```text
Enables a depth-limited Box directory walk in the background after mounting. The depth limit reduces API requests and the risk of Box rate limiting.
```

### settings.smb_preload

Visible label: `SMB`.

Reviewed text:

```text
Enables a depth-limited SMB directory walk in the background after network and VPN readiness. Individual SMB connections may override this global policy.
```

Provider duration fields accept 5 to 600 seconds. Box and SMB depth fields
accept 1 to 10. All provider preload toggles default on. Save one complete
provider row atomically so enabled, duration, and depth cannot be mixed with
stale values.

Each provider's switch, seconds field, and optional levels field remain on one
horizontal row. Duration and depth explanations appear only in delayed hover
help. Provider toggles use the reviewed provider-specific descriptions above
and keep a small gap between label and switch.

### settings.preload_duration

Control: unlabeled seconds field followed by visible text `seconds`.

Reviewed text:

```text
Maximum duration of the background preload task: 5–600 seconds. Browsing remains available while the preload runs and after it stops.
```

This field is present in every provider row and uses seconds. Accepted values
are 5 through 600.

### settings.preload_depth

Control: unlabeled depth field followed by visible text `levels`.

Reviewed text:

```text
Maximum recursive directory depth: 1–10 levels. Higher values create more storage-server requests, may trigger provider rate limits, and can consume the entire preload time.
```

This field appears only for Box and SMB. Accepted values are 1 through 10.

### connection.smb_preload_use_global

Visible label: `Use global preload settings`.

Reviewed text:

```text
Use the SMB preload policy from Cloud Mounter Settings for this connection.
```

The per-connection override controls are available only for SMB Online mounts.
They default to global inheritance and are useful when home, LAN, corporate,
and VPN shares have different performance. When global inheritance is off, the
connection editor shows `Preload directories after mount`, duration in seconds,
and depth in levels. These use the same bounds and field meanings as the global
SMB row.

## Popup

### popup.add_connection

Current text:

```text
Open the Add Connection workflow to create a new storage connection.
```

Reviewed text:

```text
Open the Add Connection workflow to create a new storage connection.
```

### popup.refresh

Current text:

```text
Reload saved connections and refresh VPN status in the running applet. Existing operations continue.
```

Reviewed text:

```text
Reload saved connections and refresh VPN status in the running applet. Existing operations continue.
```

### popup.connection_name

Current text:

```text
Open `{connection}` in the Modify workflow.
```

Reviewed text:

```text
Open `{connection}` in the Modify workflow.
```

### popup.primary_toggle

Current text:

```text
Current status: {status}. Toggle to {action}.
```

Unavailable variant:

```text
Current status: {status}. {action} is unavailable: {reason}.
```

No-operation variant:

```text
Current status: {status}.
```

Reviewed text:

```text
Current status: {status}. Toggle to {action}.
```

Reviewed unavailable variant:

```text
Current status: {status}. {action} is unavailable: {reason}.
```

Reviewed no-operation variant:

```text
Current status: {status}.
```

## Add/Modify Sections

### settings.provider

Current text:

```text
Choose the storage provider. OneDrive uses OneDrive-specific engines; Google Drive, Box, and SMB use rclone.
```

Reviewed text:

```text
Choose the storage provider. OneDrive uses OneDrive-specific engines; Google Drive, Box, and SMB use rclone.
```

### settings.access_mode

Current text:

```text
Online mount gives on-demand network-backed access. Offline mirror keeps a local copy and synchronizes later.
```

Reviewed text:

```text
Online mount gives on-demand network-backed access. Offline mirror keeps a local copy and synchronizes later.
```

### settings.connection_name

Current text:

```text
Display name shown in the applet popup.
```

Reviewed text:

```text
Display name shown in the applet popup.
```

### settings.local_target

Current text:

```text
Bold warning: Do not reuse mountpoints and mirror directories.
Body: Online mounts use a mountpoint. Offline mirrors use an ordinary local directory.
```

Reviewed text:

```text
Bold warning: Do not reuse mountpoints and mirror directories.
Body: Online mounts use a mountpoint. Offline mirrors use an ordinary local directory.
```

## Add/Modify Action Row

### action.test_connection

Current text:

```text
Validate the current form values, dependencies, remote/account access, and generated plan before saving.
```

Reviewed text:

```text
Validate the current form values, dependencies, remote/account access, and generated plan before saving.
```

### action.save_connection

Current text:

```text
Bold warning: Preview and confirm before initial synchronization.
Body: Save this connection after validation. Potentially destructive sync setup still requires preview and confirmation.
```

Reviewed text:

```text
Bold warning: Preview and confirm before initial synchronization.
Body: Save this connection after validation. Potentially destructive sync setup still requires preview and confirmation.
```

### action.detect_rclone_remotes

Current text:

```text
Read rclone config dump, filter remotes by provider backend, and offer matching remotes as selectable account choices. If no {provider} remotes are detected, enter an existing remote name or create one here.
```

Reviewed text:

```text
Read rclone config dump, filter remotes by provider backend, and offer matching remotes as selectable account choices. If no {provider} remotes are detected, enter an existing remote name or create one here.
```

### action.import

Current text:

```text
Scan existing user services, preview compatible rclone or onedriver mounts, then map one into this wizard.
```

Reviewed text:

```text
Scan existing user services, preview compatible rclone or onedriver mounts, then map one into this wizard.
```

### action.preview

Current text:

```text
Run a dry-run preview for the saved Offline Mirror connection. Save pending form changes first if they should be included.
```

Reviewed text:

```text
Run a dry-run preview for the saved Offline Mirror connection. Save pending form changes first if they should be included.
```

### action.sync_now

Current text:

```text
Bold warning: Initial synchronization requires a successful preview first.
Body: Run synchronization now for the saved Offline Mirror connection.
```

Reviewed text:

```text
Bold warning: Initial synchronization requires a successful preview first.
Body: Run synchronization now for the saved Offline Mirror connection.
```

### action.enable_disable

Current text:

```text
Disable prevents automatic use without deleting credentials, data, cache, recovery, or imported originals.
```

Reviewed text:

```text
Disable prevents automatic use without deleting credentials, data, cache, recovery, or imported originals.
```

### action.remove

Current text:

```text
Remove this applet-managed connection after confirmation. User data and external credentials are preserved.
```

Reviewed text:

```text
Remove this applet-managed connection after confirmation. User data and external credentials are preserved.
```

## Rclone Setup

### rclone.create_google_drive_remote

Current text:

```text
Create the rclone Google Drive remote with full-drive scope and local browser OAuth. A custom client ID requires its matching client secret. Complete browser authorization, then run Test Connection. OAuth values stay in rclone config, not applet configuration.
```

Reviewed text:

```text
Create the rclone Google Drive remote with full-drive scope and local browser OAuth. A custom client ID requires its matching client secret. Complete browser authorization, then run Test Connection. OAuth values stay in rclone config, not applet configuration.
```

### rclone.google_client_id

Reviewed text:

```text
Use the OAuth client ID from a Google Cloud Desktop app with the Google Drive API enabled. Rclone says a private client is required to avoid interruption during its 2026 shared-client retirement. Leaving both OAuth fields blank attempts the shared client only for compatibility.
```

### rclone.google_client_secret

Reviewed safety text:

```text
The applet does not save this value in its configuration or logs.
```

Reviewed detail text:

```text
Enter the secret issued with the client ID. Client ID and secret must be supplied together. The value is passed directly to rclone and cleared from this form after the operation.
```

### rclone.update_google_oauth_client

Reviewed text:

```text
Explicitly replace this rclone remote's Google OAuth client ID and secret and authorize it again in the browser. Both fields are required. Remount active connections afterward.
```

### rclone.create_box_remote

Current text:

```text
Create the rclone Box remote with local browser OAuth. Complete the browser authorization window that rclone opens, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.
```

Reviewed text:

```text
Create the rclone Box remote with local browser OAuth. Complete the browser authorization window that rclone opens, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.
```

### rclone.create_smb_remote

Current text:

```text
Create the rclone SMB remote with host/user/domain only, then detect and select it. Passwords stay in rclone, not applet config.
```

Reviewed text:

```text
Create the rclone SMB remote with host/user/domain only, then detect and select it. Passwords stay in rclone, not applet config.
```

### rclone.remote_name

Current text:

```text
Enter the rclone remote name. The applet verifies that the backend matches the selected provider before saving.
```

Provider-specific variants mention Google Drive, Box, or SMB.

Reviewed text:

```text
Enter a new rclone remote name, then click the provider-specific Create Remote button. To use an existing remote, select a detected remote or enter its exact name from `rclone config`. For SMB, fill in the SMB settings and use Create/Update SMB Remote, which can also update an existing remote.
```

Resolution (September 15, 2026): corrected all three provider-specific tooltips to explain that a new name is entered before creation. Retained existing-remote selection, exact-name entry, naming examples and provider validation details; clarified SMB create/update behavior.

### rclone.remote_choice

Current text:

```text
Use rclone remote `{remote}` with backend `{backend}` for this connection.
```

Reviewed text:

```text
Use rclone remote `{remote}` with backend `{backend}` for this connection.
```

### rclone.remote_subtree

Current text:

```text
Leave empty for the whole rclone remote, or enter a folder/subtree to limit this connection.
```

Reviewed text:

```text
Leave empty for the whole rclone remote, or enter an existing folder/subtree to limit this connection.
```

### smb.host

Current text:

```text
Server DNS name or IP address used for rclone's SMB `host` option. Create SMB Remote uses these fields and leaves passwords in rclone, not applet configuration.
```

Reviewed text:

```text
Server DNS name or IP address used for rclone's SMB `host` option. Create SMB Remote uses these fields and leaves passwords in rclone, not applet configuration.
```

### smb.username

Current text:

```text
Optional SMB username. Leave blank for guest or rclone defaults.
```

Reviewed text:

```text
Optional SMB username. Leave blank for guest or rclone defaults.
```

### smb.domain

Current text:

```text
Optional NTLM domain. WORKGROUP is rclone's default.
```

Reviewed text:

```text
Optional NTLM domain. WORKGROUP is rclone's default.
```

## OneDrive Setup

### onedrive.start_onedriver_setup

Current text:

```text
{mode_guidance} Run `onedriver --auth-only` with this connection's app-owned config file and cache directory. Complete authorization in the browser, then run Test Connection.
```

Reviewed text:

```text
{mode_guidance} Runs `onedriver --auth-only` with this connection's app-owned config file and cache directory. Complete authorization in the browser, then run Test Connection.
```

### onedrive.start_mirror_setup

Current text:

```text
{mode_guidance} Run `onedrive --reauth` with this connection's app-owned config directory. Complete authorization in the browser; onedrive should receive the local redirect itself.
```

Reviewed text:

```text
{mode_guidance} Runs `onedrive --reauth` with this connection's app-owned config directory. Complete authorization in the browser; onedrive should receive the local redirect itself.
```

### onedrive.manual_auth_handoff

Current text:

```text
Fallback for browser or tenant cases where onedrive cannot capture the redirect automatically. The applet prepares auth-files and a response URL field.
```

Reviewed text:

```text
Fallback for browser or tenant cases where onedrive cannot capture the redirect automatically. The applet prepares auth-files and a response URL field.
```

### onedrive.account

Current text:

```text
Bold warning: Do not reuse OneDrive mountpoints as sync directories, and do not run onedriver and abraunegg/onedrive against overlapping OneDrive trees.
Body: OneDrive account label used by this applet. Credentials remain with the selected OneDrive engine.
```

Mode-specific variants mention onedriver Online Mount or abraunegg/onedrive Offline Mirror.

Reviewed text:

```text
Bold warning: Do not reuse OneDrive mountpoints as sync directories, and do not run onedriver and abraunegg/onedrive against overlapping OneDrive trees.
Body: Label this OneDrive account so you can recognize it later, for example `onedriver-work` or `onedrive-personal`. Test Connection and Save validate the selected OneDrive engine without reading provider tokens.
```

Resolution: added account-label examples and clarified that the label is for recognizing the account later.

### onedrive.subtree

Current text:

```text
Leave empty for the whole OneDrive account, or enter a folder/subtree to limit this connection.
```

Reviewed text:

```text
Leave empty for the whole OneDrive account, or enter an existing folder/subtree to limit this connection.
```

### onedrive.open_auth_helper

Current text:

```text
Open the generated onedrive auth URL in the WebKitGTK helper when available. The helper attempts to capture the final Microsoft redirect automatically; otherwise the applet falls back to xdg-open.
```

Reviewed text:

```text
Open the generated onedrive auth URL in the WebKitGTK helper when available. The helper attempts to capture the final Microsoft redirect automatically; otherwise the applet falls back to xdg-open.
```

### onedrive.shell_fallback

Current text:

```text
This selectable command opens the same auth URL in your normal browser if the WebKitGTK helper cannot be used.
```

Reviewed text:

```text
This selectable command opens the same auth URL in your normal browser if the WebKitGTK helper cannot be used.
```

### onedrive.response_url

Current text:

```text
Paste the full URL beginning with https://login.microsoftonline.com/... and containing code=.
```

Reviewed text:

```text
Paste the full URL beginning with https://login.microsoftonline.com/... and containing code=.
```

### onedrive.submit_response_url

Current text:

```text
Write the pasted response URL to the transient response file expected by the running onedrive authentication process.
```

Reviewed text:

```text
Write the pasted response URL to the transient response file expected by the running onedrive authentication process.
```

## Online Mount Settings

### online.start_at_login

Current text:

```text
Manual startup is the default. Enable this only for connections that should start when you log in.
```

Reviewed text:

```text
Manual startup is the default. Enable this only for connections that should start when you log in.
```

### online.cache_limit

Current text:

```text
Maximum rclone VFS cache size. The approved default is 20 GiB.
```

Reviewed text:

```text
Maximum rclone VFS cache size. The approved default is 20 GiB.
```

## Offline Mirror Settings

### mirror.sync_interval

Current text:

```text
How often to run background synchronization while connected. Manual Sync Now remains available.
```

Reviewed text:

```text
How often to run background synchronization while connected. Manual Sync Now remains available.
```

### mirror.sync_on_metered

Current text:

```text
Disabled by default so automatic sync pauses on metered networks. Manual Sync Now can still be used.
```

Reviewed text:

```text
Disabled by default so automatic sync pauses on metered networks. Manual Sync Now can still be used.
```

### mirror.recovery_directory

Current text:

```text
Bold warning: Keep recovery data outside the mirror tree.
Body: Optional. Leave blank to auto-generate a sibling recovery directory based on the mirror directory.
```

Reviewed text:

```text
Bold warning: Keep recovery data outside the mirror tree.
Body: Optional. Leave blank to auto-generate a sibling recovery directory based on the mirror directory.
```

## VPN

### vpn.no_vpn

Current text:

```text
No VPN will be started or checked before this connection runs.
```

Reviewed text:

```text
No VPN will be started or checked before this connection runs.
```

### vpn.profile_choice

Current text:

```text
{vpn_kind}: {profile}. External profile: {external_profile}. Readiness: {readiness}. Timeout: {timeout_seconds} seconds. The applet may request activation before mount/sync; authentication remains with the VPN client.
```

Reviewed text:

```text
{vpn_kind}: {profile}. External profile: {external_profile}. Readiness: {readiness}. Timeout: {timeout_seconds} seconds. The applet may request activation before mount/sync; authentication remains with the VPN client.
```

### vpn.detect

Current text:

```text
Detect existing NetworkManager VPN profiles and Cisco Secure Client availability, then import them as applet VPN references without storing credentials.
```

Reviewed text:

```text
Detect existing NetworkManager VPN profiles and Cisco Secure Client availability, then import them as applet VPN references without storing credentials.
```

### vpn.disconnect_when_unused

Current text:

```text
The applet may disconnect only a VPN it activated, and only after no active connection still needs it.
```

Reviewed text:

```text
The applet may disconnect only a VPN it activated, and only after no active connection still needs it.
```
