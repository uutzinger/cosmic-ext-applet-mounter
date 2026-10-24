# Cloud Mounter

This document was written by 

Urs Utzinger October 23, 2026 

for a request to include Cloud Mounter in the COSMIC™ applet repository.

Cloud Mounter is an applet for the COSMIC™ desktop for managing storage connections to
 **OneDrive**, **SharePoint**, **Google Drive**, **Box**,  **SMB**, and **SFTP**. 
 It supports direct **Online mount** and **Offline mirror** mode.

## Purpose
- **What does Cloud Mounter let users do?**

    The applet simplifies mounting cloud storage. It authenticates to user's 
    online storage services and creates systemd units that manage the mount points or synchronize mirrors. The user can turn storage connections on or off to reduce file manager stalls when the network is slow or unavailable. 

- **Why does it belong in the COSMIC™ applet repository?**

    Cloud Mounter is a COSMIC™ desktop extension. The COSMIC applet repository is the ideal flatpak host.

## Package
- **Which version and source commit does the manifest build?**

    The manifest points to version [0.5.5](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/v0.5.5) and application commit [13e204ec93049587611978c0cfbad0f9e0a5968c](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/13e204ec93049587611978c0cfbad0f9e0a5968c)

- **Which runtime does it use?**
    - `Freedesktop Platform`, version 25.08 provides the Runtime libraries to run the app  
    - COSMIC BaseApp provides the build base
    - Freedesktop SDK, version 25.08 provides the build tools
    - Rust SDK extension

- **Which dependencies must users install on the host, which are already present through the OS?**
    The dependencies  can be installed with an on the applets Github repository [install script](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/scripts/install-dependencies.sh) It supports Debian/Ubuntu/Pop!_OS, Fedora, Arch Linux, openSUSE and Gentoo.

    Dependency installation is not automatic with Flatpak installation of the applet.

    The applet requires these programs:

    - [**`rclone`**](https://rclone.org/) manages most Online mounts and Offline mirrors, 1.74.3 or newer.
    - [**`onedriver`**](https://github.com/jstaf/onedriver) manages OneDrive Online mount, 0.15.0 or newer.
    - [`fusermount3`](https://github.com/libfuse/libfuse) / FUSE 3 is used for Online mounts and already part of the operating system.
    - [NetworkManager](https://networkmanager.dev/) checks network and VPN readiness over D-Bus and is already part of the operating system.
    - systemd user manager & systemd logind is used for services and timer control and optional sleep cleanup over D-Bus. It is already part fo the operating system services.


## Permissions

The applets reaches outside its isolated environment in a restricted and limited manner. The unavoidable types of external host commands are listed below:

| Executable | Role and Restrictions |
|---|---|
| `flatpak-spawn -host` |  This command is used in helpers for storage engines and filesystem running in the user session. The applet implements the reviewed and fixed host requests listed below and the Appendix. `flatpk-spawn` is not used as an arbitrary UI command endpoint. |
| `flatpak document-info <portal-path` | A host-path lookup is needed by the mounts and mirrors and services. It is a `Documents portal` compatibility lookup for Documents portal v4. When v5 is available, this fallback is not used and the host-path is looked up over D-Bus. |
| `rclone` |  The command is used for most storage setups and access. The applet creates rclone configuration, authentication, storage operations, and a private remote control client. It uses validated rclone connection names and remote/local paths. It redacts credentials. It assures ownership of mounts, folders and processes before altering them. It checks connections before copying data. It applies checks when recovering from interrupted or failed operations. Systemd D-Bus starts, stops and monitors rclone user services. |
| `onedriver` |  The command is used for Online OneDrive access and makes mounts available to user services. The applet keeps track of mount connections. It validates mounting location. Systemd D-Bus is used to manage its background services for starting, stopping, checking if its running, or failure. |
| `fusermount3` | Is used to cleanly detach mounts or start a confirmed lazy recovery. The applet checks ownership of the mount. It executes only when an eligible recovery state is present. |
| `cat /proc/self/mountinfo` |  This command is used to read host mount information. The applet uses a fixed read-only request. It reads the information in the background and limits the amount of output it collects. |
| `mountpoint --quiet <path>` |  The applet checks host mount readiness before starting or stopping connections. It only accesses validated mount points and uses fixed commands. |
| `find` |  The applet checks directories and prevents mounting over existing files. The applet also provides optional pre loading of file metadata into the directory cache of Online mounts. |

- **Why does it need NetworkManager and systemd access?**

    The applets interacts with the `Network Manager` to start and stop VPNs associated with a connection over D-Bus. It discovers existing VPN profiles. It also discovers metered connections and stops syncing mirrors when such a connection is active and the setting is enabled.
    
    `Systemd user manager` manages the applets background services and schedules and maintains timers. 
    
    `Systemd logind` detects preparation for sleep and starts optional connection unmounting before the host computer goes to sleep and restarts them after wake up.

- **Why does it need host execution?**

    The applets uses rclone and onedriver for cloud storage access. These programs run on the host and create mounts and mirrors accessible to other applications. The applet uses flatpak spawn to run these commands as listed below and above in Permissions.

- **How are local directory permissions granted?**

    `rclone` and `onedriver` run with normal user permission. They receive the host paths through their configuration which was set by the applets configuration editor. The user can use a browser to select storage paths, which then are stored in the host programs' configuration.

    The applet receives a folder path when the user starts the folder picker and selects a path. It does not have blanket access to users' home directory.

## Testing

- **Identify the installed Flatpak candidate.**

  Version [0.5.5](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/v0.5.5)

- **Summarize the test operations actually exercised.**

    Testing begins on the developer's computer using a source built version with `just install-user`. Validation covers the user interface, connection settings and supported storage providers with each configuration tested and results recorded.

    I tested
    - Google Drive and Box storage on corporate account both for mirroring and mounting
    - Google Drive on personal account for mirroring and mounting
    - OneDrive mirror and online on personal and corporate accounts
    - Sharepoint mount and mirror on corporate account
    - SMB with VPN on corporate storage server
    - SFTP mount of an externally hosted machine

    I also tests other workflows such as mounting and unmounting and starting and stopping mirrors and enabling VPN.

    Debian packages are built through Github Actions and tested separately in a PopOS virtual machine.

    The Flatpak candidate is also built with Github Actions and installed on the same virtual machine. Tests cover the user interface, rclone mirror, rclone mount and the onedriver online mount. I also test VPN activation.

    Detailed test results and outstanding checks are recorded in [`Task List'](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/Task%20List.md) and ['Task List Completion Notes'](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/Task%20List%20Completion%20Notes.md). These lists also contain the development steps and decisions and are AI generated recordings of my instructions and interactions with the agent. The agent's behavior is defined in [AGENTS.md](AGENTS.md)
    
    Previous release Task Lists and Completion notes are available in [archived documentation](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/master/docs/archive).

- **State relevant limitations and skipped checks.**

    - The applet does not a provide a multi user file access and locking mechanism.
    - ARM64 support is not tested
    - Sleep and Wake behavior for unmounting and remounting is not thoroughly tested.

## Appendix: List of commands interacting with systems outside the Flatpak isolation

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


