# COSMIC Cloud Mounter Tooltip Review

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
Reload saved configuration and refresh the applet view.
```

Reviewed text:

```text
Reload saved configuration and refresh the applet view.
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
Create the rclone Google Drive remote with full-drive scope and local browser OAuth. Complete the browser authorization window, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.
```

Reviewed text:

```text
Create the rclone Google Drive remote with full-drive scope and local browser OAuth. Complete the browser authorization window, then run Test Connection. Credentials and refresh tokens stay in rclone config, not applet configuration.
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
Select a detected rclone remote, or enter the exact remote name from `rclone config`. Use a clear provider-specific name; the applet verifies the backend, authentication, and subtree access before saving. Passwords stay in rclone, not applet configuration.
```

Resolution: added provider-specific naming examples and clarified that the user may select a detected remote or enter the exact rclone remote name.

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
