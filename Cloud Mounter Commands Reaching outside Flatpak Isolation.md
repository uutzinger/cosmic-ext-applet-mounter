## List of commands interacting with systems outside the Flatpak isolation

The following external commands are used in version 0.5.5. They include direct host-command requests and commands launched by generated user services.

This appendix covers the Flatpak app's runtime commands. Development tests, installer commands and native-package launch variants are outside this list. The VPN table also identifies the services used for authentication.

For completeness, the list below was generated with help from AI.

### Host execution and folder-path compatibility

| Command pattern | Purpose |
|---|---|
| `flatpak-spawn --host <fixed-command> ...` | Runs the host rclone and onedriver and helper commands listed in this appendix. |
| `flatpak document-info <portal-path>` | Resolves a selected folder's host path on Documents portal v4. Portal v5+ uses D-Bus; this fallback is not used for general portal errors. |

### Storage engines

| Command pattern | Purpose |
|---|---|
| `rclone config dump` |  Read saved rclone connections to identify their storage providers and authentication settings. |
| `rclone config create <remote> <backend> ...` |  Create a saved connection for Google Drive, Box, SMB, SFTP, SharePoint, or a OneDrive Offline Mirror. |
| `rclone config update <remote> ...` |  Update a saved connection’s settings, including completing Microsoft browser sign-in. |
| `rclone config password <remote> <field> <secret>` |  Save a password using rclone’s obscured format. Hide the secret argument from diagnostic output. |
| `rclone config delete <remote>` |  Delete a saved connection after confirming it is unused, or undo its creation if setup fails. |
| `rclone lsjson <remote:path> --stat` |  Check access to the selected OneDrive folder and allow rclone to refresh its sign-in token if needed. Read folder information only. |
| `rclone lsjson <remote-or-local> --recursive --files-only --no-mimetype --filter-from <filters>` |  Estimate how many files differ between OneDrive and its local mirror without changing either. List files on each side, limiting each listing to 30 seconds and 4 MiB of captured output. Prevent overlap with synchronization. Use the existing host-command route and connection-owned paths; no additional permission. |
| `rclone lsf <remote:path> --max-depth 1` |  Check that the selected cloud folder or document library can be listed, including after Google Drive or Box sign-in. Limit the check to 20 seconds and 16 KiB of captured output. Use the applet's shared storage-command runner, with the existing native or Flatpak host-command route and permissions. |
| `rclone rc --unix-socket <socket> rc/noop` |  Check that the running rclone mount responds through its private local control connection. |
| `rclone rc --unix-socket <socket> vfs/stats` |  Read cache and upload status to detect problems and assess whether the mount can safely stop before sleep. |
| `rclone rc --unix-socket <socket> vfs/queue` |  Read the number of files waiting to upload or currently uploading. |
| `rclone rc --unix-socket <socket> vfs/refresh recursive=<false/true> _async=true` |  Refresh Google Drive's top-level folder listing first, then its subfolders, in the background. Both stages share one time limit. |
| `rclone rc --unix-socket <socket> job/status jobid=<id>` / `job/stop jobid=<id>` |  Check whether a Google Drive folder-listing refresh is running, completed, or failed; request cancellation when needed. |
| `rclone bisync <remote> <local> ... [--resync] [--dry-run]` |  Two way synchronization between cloud folder and local offline mirror. |
| `rclone mkdir <remote-recovery-day>` |  Create the dated cloud recovery folder used to retain files replaced or deleted during synchronization. |
| `rclone cat <remote-anchor-or-owner>` |  Read marker files to verify remote-folder access and recovery-folder ownership. |
| `rclone lsf <remote-recovery-day>` |  Check that a dated recovery folder is empty before assigning it to this connection. |
| `rclone copyto <private-owner-file> <remote-owner-marker>` |  Upload a marker identifying which Cloud Mounter connection owns the remote recovery folder. |
| `rclone check <remote> <local> --checksum --filter-from <filters>` |  Verify that cloud and local mirror files match after synchronization, using checksums and the configured exclusion filters. SharePoint runs an additional synchronization pass before this check; OneDrive checks after initial and manual synchronization. |
| `rclone lsf --dirs-only --max-depth 1 <remote-recovery-root>` |  List dated recovery folders to identify those eligible for cleanup under the configured retention period after successful synchronization. |
| `rclone purge <verified-owned-expired-day>` |  Delete a dated recovery folder and its contents only after verifying that it belongs to this connection and its retention period has expired. |
| `onedriver --auth-only --config-file <file> --cache-dir <dir> <mountpoint>` |  Sign in to OneDrive through onedriver without mounting the drive. Routine validation checks saved connection metadata. |

### FUSE and host inspection

| Command pattern | Purpose |
|---|---|
| `fusermount3 -u <mountpoint>` | Unmount the connection's filesystem normally before stopping its background service. |
| `fusermount3 -uz <mountpoint>` | Detach a mount immediately, allowing cleanup to finish after existing references are released. Use only after separate user confirmation and only when the Online Mount meets the recovery conditions. |
| `cat /proc/self/mountinfo` | Read the host's list of mounted filesystems to check what ordinary applications can access outside the Flatpak sandbox. |
| `mountpoint --quiet <path>` | Check whether a host folder is a mountpoint, as part of determining whether the mounted filesystem is ready. |
| `find <path> -maxdepth 0 -type d -print` | Check that the selected path is a directory on the host. |
| `find <path> -mindepth 1 -maxdepth 1 -print -quit` | Check whether the selected folder contains anything, so mounting a drive there will not hide existing local files or folders. |
| `find -P <mountpoint> -xdev ... -type d ...` | Scan folders in advance to prepare directory listings, without reading file contents. Apply time, depth and exclusion limits; do not follow symbolic links or cross into other mounted filesystems. |

### VPN and OpenConnect authentication

VPN control uses D-Bus. For OpenConnect authentication, the user can install the optional authentication components and agent service using the setup script. The applet starts that service when needed; nm-applet and the NetworkManager OpenConnect plugin/dialog handle sign-in.

| Service or component | Purpose |
|---|---|
| NetworkManager D-Bus | Discovers saved VPN profiles, activates and deactivates the selected VPN connections, and monitors their state. |
| systemd user-manager D-Bus | Starts or restarts the user-installed `cosmic-mounter-openconnect-agent.service` when needed for OpenConnect authentication. |
| `nm-applet` secret agent and NetworkManager OpenConnect plugin/dialog | Handle VPN credentials and authentication prompts. These are external authentication dependencies; the applet does not launch them through direct host-command requests. |

### Commands launched by host services

These commands run through the applet's generated systemd user services rather than direct
host-command requests from the applet. They are controlled through D-Bus.

| Command pattern | Purpose |
|---|---|
| `rclone mount ...` | Provides Online cloud mounts accessible to host applications. |
| `onedriver --config-file <file> --cache-dir <dir> <mountpoint>` | Provides Online OneDrive mounts. |
| `/usr/bin/flatpak run --command=cosmic-ext-applet-mounter <app-id> <fixed-operation> [<validated-arguments> ...]` | Starts an app worker to run scheduled synchronization or check conditions before proceeding: network/VPN availability, whether the network is metered, or whether the mount folder is suitable. Arguments depend on the operation and can include the saved connection's identifier and mount path. |
