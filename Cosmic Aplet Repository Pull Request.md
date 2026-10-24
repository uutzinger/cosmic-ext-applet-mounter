# Cloud Mounter

Written by Urs Utzinger October 23, 2026 for a request to include Cloud Mounter in the COSMIC™ applet repository.

Cloud Mounter is an applet for the COSMIC™ desktop for managing storage
connections to **OneDrive**, **SharePoint**, **Google Drive**, **Box**, 
**SMB**, and **SFTP**. It supports direct **Online mount** and **Offline mirror** mode.

## Purpose
- **What does Cloud Mounter let users do?**

    The applet simplifies mounting cloud storage. It authenticates to user's 
    online storage services and creates systemd units that manage the mount points or synchronize mirrors. The user can turn storage connections on or off to reduce file manager stalls when the network is slow or unavailable. 

- **Why does it belong in the COSMIC applet repository?**

    Cloud Mounter is a COSMIC desktop extension. The COSMIC applet repository is the ideal flatpak host.

## Package
- **Which version and source commit does the manifest build?**

    The manifest points to version [0.5.5](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/v0.5.5) and application commit [13e204ec93049587611978c0cfbad0f9e0a5968c](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/13e204ec93049587611978c0cfbad0f9e0a5968c)

- **Which runtime does it use?**
    - **`Freedesktop Platform`**, version 25.08 provides the Runtime libraries to run the app  
    - COSMIC BaseApp provides the build base
    - Freedesktop SDK, version 25.08 provides the build tools
    - Rust SDK extension

- **Which dependencies must users install on the host, which are already present through the OS?**
    - [**`rclone`**](https://rclone.org/) manages most Online mounts and Offline mirrors, 1.74.3 or newer.
    - [**`onedriver`**](https://github.com/jstaf/onedriver) manages OneDrive Online mount, 0.15.0 or newer.
    - [`fusermount3`](https://github.com/libfuse/libfuse) / FUSE 3 is used for Online mounts and already part of the operating system.
    - [NetworkManager](https://networkmanager.dev/) checks network and VPN readiness over D-Bus and is already part of the operating system.
    - systemd user manager & systemd logind is used for services and timer control and optional sleep cleanup over D-Bus. It is already part fo the operating system services.

    The dependencies  can be installed with an on the applets Github repository [install script](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/scripts/install-dependencies.sh) It supports Debian/Ubuntu/Pop!_OS, Fedora, Arch Linux, openSUSE and Gentoo.

    Dependency installation is not automatic with Flatpak installation of the applet.

## Permissions

The applets reaches outside its isolated environment in a restricted and limited manner. The types of **unavoidable invocations** of external host commands are listed below:

| Executable | Role and Restrictions |
|---|---|
| `flatpak-spawn -host` |  Is used on helpers for storage engines and filesystem running in the user session. It implements the reviewed and fixed host requests listed below. It is not an arbitrary UI command endpoint. |
| `flatpak document-info <portal-path` | Host-path lookup is needed by host mounts and mirrors and services. It is a  `Documents portal` compatibility lookup for Documents portal v4. If v5 is available, this fallback is not used. |
| `rclone` |  Is used for most storage setups and access. The applet creates rclone configuration, authentication, storage operations, and a private remote control client. It uses validated rclone connection names and remote/local paths. It redacts credentials. It assures ownership of mounts, folders and processes before altering them. It checks connections before copying data. It applies checks when recovering from interrupted or failed operations. Systemd D-Bus starts, stops and monitors rclone user services. |
| `onedriver` |  Is used for Online OneDrive access and makes mounts available to user services. It keeps track of mount connections. It validates mounting location. Systemd D-Bus is used to manage its background services for starting, stopping, checking if its running, or failure. |
| `fusermount3` | Is used to cleanly detach mounts or start a confirmed lazy recovery. It checks ownership of the mount. It executes only when an eligible recovery state is present. |
| `cat /proc/self/mountinfo` |  Is used to read host mount information. It uses a fixed read-only request. It reads the information in the background and limits the amount of output it collects. |
| `mountpoint --quiet <path>` |  It checks host mount readiness before starting or stopping connections. It only accesses validated mount points and uses fixed commands. |
| `find` |  The applet checks directories and prevents mounting over existing files. The applet also provides optional pre loading of file metadata into the directory cache of Online mounts. |

- **Why does it need NetworkManager and systemd access?**

    The applets interacts with Network Manager to start and stop VPNs associated with a connection over D-Bus. It discovers existing VPN profiles. It also discovers metered connections and stops syncing mirrors that have such setting enabled. 
    
    Systemd user manager manages the applets background services and schedules and maintains timers. 
    
    Systemd logind detects preparation for sleep and starts optional connection unmounting before the host computer goes to sleep and restarts them after wake up.

- **Why does it need host execution?**

    The applets uses rclone and onedriver for cloud storage access. These programs run on the host and create mounts and mirrors accessible to other applications. The applet uses flatpak spawn to run these commands as listed below and above in Permissions.

- **How are local directory permissions granted?**

    This programs `rclone` and `onedriver` run with normal user permission. They receive the host paths through their configuration which was set by the applets configuration editor. The user can use a browser to select storage paths which then are stored in the host programs configuration.

    The applet receives folder path by a user selected folder picker. It does not have blanket access to users' home directory.

## Testing

- **Identify the installed Flatpak candidate.**

  Version [0.5.5](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/v0.5.5)

- **Summarize the test operations actually exercised.**

    Testing begins on the developer's computer using a source built version with `just install-user`. Validation covers the user interface, connection settings and supported storage providers with each configuration tested and results recorded.

    The developer tests
    - Google Drive and Box storage on corporate account both for mirroring and mounting.
    - Google Drive on personal account for mirroring and mounting
    - OneDrive mirror and online on personal and corporate accounts.
    - Sharepoint mount and mirror on corporate account
    - SMB with VPN on corporate storage server
    - SFTP mount of an externally hosted machine

    The developer also tests other workflows such as mounting and unmounting and starting and stopping mirrors.

    Debian packages are built through Github Actions and tested separately in a PopOS virtual machine.

    The Flatpak candidate is also built with Github Actions and installed on the same virtual machine. Tests cover the user interface, rclone mirror and mount and the onedriver online mount.

    Detailed test results and outstanding checks are recorded in [`Task List'](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/Task%20List.md) and ['Task List Completion Notes'](https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/master/Task%20List%20Completion%20Notes.md). These lists also contain the development steps and decisions and are AI generated recordings of the developers instructions and interaction with the agent. The agents behavior is defined in [AGENTS.md](AGENTS.md)
    
    Previous release Task Lists and Completion notes are available in [archived documentation](https://github.com/uutzinger/cosmic-ext-applet-mounter/tree/master/docs/archive).

- **State relevant limitations and skipped checks.**

    - The applet does not a provide a multi user file access and locking mechanism.
    - ARM64 support is not tested
    - Sleep and Wake behavior for unmounting and remounting is not thoroughly tested.
    - Testing of whole root mirrors was not conducted. Only subfolders.
    - File restoration in the recovery process of mirrors was not tested.

## Appendix: List of commands interacting with systems outside the Flatpak isolation

The following external commands are used in version 0.5.5. They include direct host-command requests and commands launched by generated user services.

For completeness, the list below is generated with help from AI.

### Host execution and folder-path compatibility

| Command pattern | Purpose |
|---|---|
| `flatpak-spawn --host <fixed-command> ...` | Runs the host rclone and onedriver and helper commands listed in this appendix. |
| `flatpak document-info <portal-path>` | Resolves a selected folder's host path on Documents portal v4. Portal v5+ uses D-Bus; this fallback is not used for general portal errors. |

### Storage engines

| Command pattern | Purpose |
|---|---|
| `rclone config dump` |  Host remote discovery and provider/authentication inspection. |
| `rclone config create <remote> <backend> ...` |  Separate Google Drive, Box, SMB, SFTP, SharePoint, or OneDrive mirror remote setup. |
| `rclone config update <remote> ...` |  Provider updates and Microsoft OAuth continuation. |
| `rclone config password <remote> <field> <secret>` |  Engine-owned obscured credential storage; arguments are redacted. |
| `rclone config delete <remote>` |  Confirmed unused-remote removal or setup rollback. |
| `rclone lsjson <remote:path> --stat` |  Bounded OneDrive token refresh and selected-directory access; metadata only. |
| `rclone lsjson <remote-or-local> --recursive --files-only --no-mimetype --filter-from <filters>` |  Explicit OneDrive Preview estimate only; two read-only inventories, each 30 seconds/4 MiB, under coordinator lock. Same scoped host gateway and owned paths; no new permission. Remove when an equivalent reviewed structured engine API replaces spawning. |
| `rclone lsf <remote:path> --max-depth 1` |  Read-only remote/library access checks, including Google/Box post-OAuth verification (20 seconds, 16 KiB capture). Owner: shared app engine runner; same native/host-gateway path and permissions. |
| `rclone rc --unix-socket <socket> rc/noop` |  Private RC readiness. |
| `rclone rc --unix-socket <socket> vfs/stats` |  Cache/upload health and sleep safety. |
| `rclone rc --unix-socket <socket> vfs/queue` |  Structured queued/in-progress counts. |
| `rclone rc --unix-socket <socket> vfs/refresh recursive=<false/true> _async=true` |  Tracked Google root-first then recursive directory refresh, sharing one deadline; fast-list belongs to VFS creation. |
| `rclone rc --unix-socket <socket> job/status jobid=<id>` / `job/stop jobid=<id>` |  Refresh observation/cancellation. |
| `rclone bisync <remote> <local> ... [--resync] [--dry-run]` |  The coordinator's fixed preview/initial/manual/scheduled transactions. |
| `rclone mkdir <remote-recovery-day>` |  Prepare the selected engine's recovery area. |
| `rclone cat <remote-anchor-or-owner>` |  Access-anchor and ownership verification. |
| `rclone lsf <remote-recovery-day>` |  Prove an unmarked recovery day is empty before claiming it. |
| `rclone copyto <private-owner-file> <remote-owner-marker>` |  Publish applet ownership. |
| `rclone check <remote> <local> --checksum --filter-from <filters>` |  SharePoint post-convergence verification; OneDrive initial/manual verification without convergence workaround. |
| `rclone lsf --dirs-only --max-depth 1 <remote-recovery-root>` |  Inspect retention candidates after successful synchronization. |
| `rclone purge <verified-owned-expired-day>` |  Remove only validated, applet-owned expired remote recovery data. |
| `onedriver --auth-only --config-file <file> --cache-dir <dir> <mountpoint>` |  Engine-owned Online OneDrive authentication; validation otherwise inspects metadata. |

### FUSE and host inspection

| Command pattern | Purpose |
|---|---|
| `fusermount3 -u <mountpoint>` | Clean low-level FUSE detach before stopping the owning unit. |
| `fusermount3 -uz <mountpoint>` | Confirmed lazy recovery only in an eligible Online state. |
| `cat /proc/self/mountinfo` | Inspect the host namespace visible to ordinary host applications. |
| `mountpoint --quiet <path>` | Host mount/root readiness. |
| `find <path> -maxdepth 0 -type d -print` | Validate an ordinary host directory. |
| `find <path> -mindepth 1 -maxdepth 1 -print -quit` | Prevent covering nonempty local data with a mount. |
| `find -P <mountpoint> -xdev ... -type d ...` | Bounded, directory-only preload with depth/exclusion policy. |

### VPN and OpenConnect authentication

VPN control uses D-Bus. If OpenConnect authentication is desired the user can install it with an optional script. Then the applet uses nm-agent and OpenConnects user service as described below.

| Service or component | Purpose |
|---|---|
| NetworkManager D-Bus | Discovers saved VPN profiles, activates and deactivates connections, and monitors their state. |
| systemd user-manager D-Bus | Starts or restarts the user-installed `cosmic-mounter-openconnect-agent.service` when needed for OpenConnect authentication. |
| `nm-applet` secret agent and NetworkManager OpenConnect plugin/dialog | Handle VPN credentials and authentication prompts. These are external authentication dependencies; the applet does not launch them through direct host-command requests. |

### Commands launched by host services

These commands run through the applets generated systemd user services rather than direct
host-command requests from the applet. They are controlled through D-Bus.

| Command pattern | Purpose |
|---|---|
| `rclone mount ...` | Provides Online cloud mounts accessible to host applications. |
| `onedriver --config-file <file> --cache-dir <dir> <mountpoint>` | Provides Online OneDrive mounts. |
| `/usr/bin/flatpak run --command=<worker> <app-id> <fixed-operation> <connection-id>` | Starts the app's scheduled synchronization worker or readiness/mount guards. |


