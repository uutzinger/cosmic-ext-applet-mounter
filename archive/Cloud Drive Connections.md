## Cloud Drive Connections

Using rclone

sudo apt install rclone -y

### Corporate Box Account

```
rclone config
```

```
config file [enter]
access token [enter]
type: user
advanced:n
in web browser use SSO sign in
username@organization
if success finish: q
```

```
mkdir -p ~/Cloud/Box
mkdir -p ~/.cache/rclone-vfs
mkdir -p ~/.cache/rclone-vfs/box
mkdir -p ~/.config/systemd/user
```

Test only
```
rclone mount ua_box: ~/Cloud/Box \
  --vfs-cache-mode writes \
  --cache-dir ~/.cache/rclone-vfs \
  --dir-cache-time 5m \
  --log-level INFO
```

Persistance
```
nano ~/.config/systemd/user/rclone-ua-box.service
```

```
[Unit]
Description=Rclone mount for UA Box
Documentation=https://rclone.org/commands/rclone_mount/
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/rclone mount ua_box: %h/Cloud/Box \
  --config %h/.config/rclone/rclone.conf \
  --vfs-cache-mode full \
  --vfs-cache-max-age 168h \
  --vfs-cache-max-size 20G \
  --vfs-cache-poll-interval 5m \
  --cache-dir %h/.cache/rclone-vfs/box \
  --dir-cache-time 5m \
    --timeout 10s \
    --contimeout 5s \
    --low-level-retries 1 \
    --retries 1 \
    --retries-sleep 5s \
  --umask 002 \
  --log-level INFO
ExecStop=/bin/fusermount3 -uz %h/Cloud/Box
ExecStopPost=-/bin/fusermount3 -uz %h/Cloud/Box
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

Make a service

```
systemctl --user enable rclone-ua-box.service
systemctl --user restart rclone-ua-box.service
systemctl --user status rclone-ua-box.service
```

### Corporate Google
```
rclone config
```

```
n: New remote
name> ua_gdrive
Storage> drive
client_id> Press Enter
client_secret> Press Enter
scope> drive
root_folder_id> Press Enter
service_account_file> Press Enter
Edit advanced config? n
Use web browser to automatically authenticate rclone with remote? y
Configure this as a Shared Drive? n
Keep this "ua_gdrive" remote?
y
q
```

```
mkdir -p ~/Cloud/UA_GoogleDrive
mkdir -p ~/.cache/rclone-vfs/ua-gdrive
mkdir -p ~/.config/systemd/user
```

```
nano ~/.config/systemd/user/rclone-ua-gdrive.service
```

```
[Unit]
Description=Rclone mount for UA Google Drive
Documentation=https://rclone.org/drive/
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/rclone mount ua_gdrive: %h/Cloud/UA_GoogleDrive \
  --config %h/.config/rclone/rclone.conf \
  --vfs-cache-mode full \
  --vfs-cache-max-age 168h \
  --vfs-cache-max-size 20G \
  --vfs-cache-poll-interval 5m \
  --cache-dir %h/.cache/rclone-vfs/ua-gdrive \
  --dir-cache-time 5m \
    --timeout 10s \
    --contimeout 5s \
    --low-level-retries 1 \
    --retries 1 \
    --retries-sleep 5s \
  --umask 002 \
  --log-level INFO
ExecStop=/bin/fusermount3 -uz %h/Cloud/UA_GoogleDrive
ExecStopPost=-/bin/fusermount3 -uz %h/Cloud/UA_GoogleDrive
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

```
systemctl --user enable --now rclone-ua-gdrive.service
systemctl --user restart rclone-ua-gdrive.service
systemctl --user status rclone-ua-gdrive.service
systemctl --user is-enabled rclone-ua-gdrive.service

```

### Personal Google

```
rclone config
```

```
n: New remote
name> uutzinger_gdrive
Storage> drive
client_id> Press Enter
client_secret> Press Enter
scope> drive
root_folder_id> Press Enter
service_account_file> Press Enter
Edit advanced config? n
Use web browser to automatically authenticate rclone with remote? y
Configure this as a Shared Drive? n
Keep this "uutzinger_gdrive" remote?
y
q
```

```
mkdir -p ~/Cloud/uutzinger_GoogleDrive
mkdir -p ~/.cache/rclone-vfs/uutzinger-gdrive
mkdir -p ~/.config/systemd/user
```

```
nano ~/.config/systemd/user/rclone-uutzinger-gdrive.service
```

```
[Unit]
Description=Rclone mount for personal Google Drive
Documentation=https://rclone.org/drive/
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/rclone mount uutzinger_gdrive: %h/Cloud/uutzinger_GoogleDrive \
  --config %h/.config/rclone/rclone.conf \
  --vfs-cache-mode full \
  --vfs-cache-max-age 168h \
  --vfs-cache-max-size 20G \
  --vfs-cache-poll-interval 5m \
  --cache-dir %h/.cache/rclone-vfs/uutzinger-gdrive \
  --dir-cache-time 5m \
    --timeout 10s \
    --contimeout 5s \
    --low-level-retries 1 \
    --retries 1 \
    --retries-sleep 5s \
  --umask 002 \
  --log-level INFO
ExecStop=/bin/fusermount3 -uz %h/Cloud/uutzinger_GoogleDrive
ExecStopPost=-/bin/fusermount3 -uz %h/Cloud/uutzinger_GoogleDrive
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

```
systemctl --user daemon-reload
systemctl --user enable rclone-uutzinger-gdrive.service
systemctl --user restart rclone-uutzinger-gdrive.service
systemctl --user status rclone-uutzinger-gdrive.service
systemctl --user is-enabled rclone-uutzinger-gdrive.service
```
### RClone check

echo "=== Enabled services ==="
systemctl --user is-enabled rclone-ua-box.service
systemctl --user is-enabled rclone-ua-gdrive.service
systemctl --user is-enabled rclone-uutzinger-gdrive.service
systemctl --user is-enabled rclone-ua-engr.service

echo
echo "=== Running services ==="
systemctl --user list-units --type=service | grep -i rclone

echo
echo "=== Mounted cloud folders ==="
findmnt | grep "$HOME/Cloud"

echo
echo "=== Rclone processes ==="
pgrep -a rclone

echo
echo "=== Folder access test ==="
ls ~/Cloud/Box | head
ls ~/Cloud/UA_GoogleDrive | head
ls ~/Cloud/uutzinger_GoogleDrive | head

### Microsoft OneDrive

```
echo 'deb http://download.opensuse.org/repositories/home:/jstaf/xUbuntu_24.04/ /' | sudo tee /etc/apt/sources.list.d/home:jstaf.list
curl -fsSL https://download.opensuse.org/repositories/home:jstaf/xUbuntu_24.04/Release.key | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/home_jstaf.gpg > /dev/null
sudo apt update
sudo apt install onedriver

mkdir -p ~/Cloud/UA_OneDrive
```

```
onedriver
```

### College Server

Wireguard

```
sudo apt update
sudo apt install wireguard wireguard-tools
```

Cosmic Settings, Network, VPN, Switch to WireGuard at bottom. Import settings.

```
sudo apt update
sudo apt install smbclient cifs-utils
```

test

```
sudo mount -t cifs //engr-drive.bluecat.arizona.edu/Research /home/uutzinger/Cloud/UA_ENGR \
 -o credentials=/home/uutzinger/Cloud/.smbcredentials-engr,uid=$(id -u),gid=$(id -g),vers=3.0
```


```
rclone config
```

```
n: New remote
name> ua_engr
SMB
y
q
```

```
mkdir -p ~/Cloud/UA_ENGR
mkdir -p ~/.cache/rclone-vfs/ua-engr
mkdir -p ~/.config/systemd/user
```

```
nano ~/.config/systemd/user/rclone-ua-engr.service
```

```
[Unit]
Description=Rclone mount for UA Engineering Research storage
Documentation=https://rclone.org/commands/rclone_mount/
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
ExecStartPre=/bin/mkdir -p /home/uutzinger/Cloud/UA_ENGR
ExecStart=/usr/bin/rclone mount ua_engr:Research /home/uutzinger/Cloud/UA_ENGR \
    --vfs-cache-mode full \
  --vfs-cache-max-size 20G \
    --vfs-cache-max-age 168h \
  --dir-cache-time 5m \
    --timeout 10s \
    --contimeout 5s \
    --low-level-retries 1 \
    --retries 1 \
    --retries-sleep 5s \
  --poll-interval 0 \
  --cache-dir /home/uutzinger/.cache/rclone-ua-engr \
  --log-level INFO \
  --log-file /home/uutzinger/.cache/rclone-ua-engr.log
ExecStop=/bin/fusermount3 -uz /home/uutzinger/Cloud/UA_ENGR
ExecStopPost=-/bin/fusermount3 -uz /home/uutzinger/Cloud/UA_ENGR
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target
```

```
systemctl --user daemon-reload
systemctl --user start rclone-ua-engr.service
systemctl --user disable rclone-ua-engr.service
```


#### Mount unMount

##### Toggle Script CIFS Mounts

```
mkdir -p ~/.local/bin
nano ~/.local/bin/ua-engr-mount
```

```
#!/usr/bin/env bash
set -u

SERVICE="rclone-ua-engr.service"
MOUNTPOINT="$HOME/Cloud/UA_ENGR"

notify() {
    if command -v notify-send >/dev/null 2>&1; then
        notify-send "UA_ENGR mount" "$1"
    fi
    echo "$1"
}

is_mounted() {
    findmnt -rn --target "$MOUNTPOINT" >/dev/null 2>&1
}

is_active() {
    systemctl --user is-active --quiet "$SERVICE"
}

mkdir -p "$MOUNTPOINT"

if is_mounted || is_active; then
    notify "Unmounting UA_ENGR..."
    systemctl --user stop "$SERVICE" >/dev/null 2>&1

    # Extra cleanup in case service stopped but FUSE mount remains.
    fusermount3 -uz "$MOUNTPOINT" >/dev/null 2>&1 || true

    if is_mounted; then
        notify "UA_ENGR still appears mounted. It may be busy."
        exit 1
    else
        notify "UA_ENGR unmounted."
        exit 0
    fi
else
    notify "Mounting UA_ENGR..."

    # Clear previous failed state so systemd can start cleanly.
    systemctl --user reset-failed "$SERVICE" >/dev/null 2>&1 || true

    if systemctl --user start "$SERVICE"; then
        sleep 1
        if is_mounted || is_active; then
            notify "UA_ENGR mounted."
            exit 0
        else
            notify "UA_ENGR start command finished, but mount is not visible."
            exit 1
        fi
    else
        notify "Failed to mount UA_ENGR. Is Cisco VPN connected?"
        exit 1
    fi
fi
```

```
chmod +x ~/.local/bin/ua-engr-mount
```

##### Make Desktop Entry

```
cat > ~/Desktop/UA_ENGR_Mount.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=Toggle UA_ENGR Mount
Comment=Mount or unmount UA Engineering Research storage
Exec=bash -lc '/home/uutzinger/.local/bin/ua-engr-mount toggle; echo; read -p "Press Enter to close..."'
Icon=folder-remote
Terminal=true
Categories=Utility;FileManager;
EOF

chmod +x ~/Desktop/UA_ENGR_Mount.desktop

systemctl --user disable rclone-ua-engr.service
```

##### Toggle Script Allways On Mounts
```
nano ~/.local/bin/cloud-mounts-toggle
```
```
#!/usr/bin/env bash
set -u

RCLONE_SERVICES=(
  "rclone-ua-box.service"
  "rclone-ua-gdrive.service"
  "rclone-uutzinger-gdrive.service"
)

ONEDRIVER_SERVICE="onedriver@home-uutzinger-Cloud-UA_OneDrive.service"

MOUNTPOINTS=(
  "$HOME/Cloud/Box"
  "$HOME/Cloud/UA_GoogleDrive"
  "$HOME/Cloud/uutzinger_GoogleDrive"
  "$HOME/Cloud/UA_OneDrive"
)

notify() {
    if command -v notify-send >/dev/null 2>&1; then
        notify-send "Cloud mounts" "$1"
    fi
    echo "$1"
}

is_any_mounted() {
    for mp in "${MOUNTPOINTS[@]}"; do
        if findmnt -rn --target "$mp" >/dev/null 2>&1; then
            return 0
        fi
    done
    return 1
}

stop_cloud_mounts() {
    notify "Unmounting cloud mounts..."

    for svc in "${RCLONE_SERVICES[@]}"; do
        systemctl --user stop "$svc" >/dev/null 2>&1 || true
    done

    systemctl --user stop "$ONEDRIVER_SERVICE" >/dev/null 2>&1 || true

    # Extra lazy unmount cleanup in case any service stopped but FUSE remains.
    for mp in "${MOUNTPOINTS[@]}"; do
        fusermount3 -uz "$mp" >/dev/null 2>&1 || true
    done

    sleep 1

    if is_any_mounted; then
        notify "Some cloud mounts still appear mounted. They may be busy."
        findmnt | grep -E 'Cloud|rclone|onedriver' || true
        exit 1
    else
        notify "Cloud mounts unmounted."
        exit 0
    fi
}

start_cloud_mounts() {
    notify "Mounting cloud services..."

    for svc in "${RCLONE_SERVICES[@]}"; do
        systemctl --user reset-failed "$svc" >/dev/null 2>&1 || true
        systemctl --user start "$svc" >/dev/null 2>&1 || notify "Failed to start $svc"
    done

    systemctl --user reset-failed "$ONEDRIVER_SERVICE" >/dev/null 2>&1 || true
    systemctl --user start "$ONEDRIVER_SERVICE" >/dev/null 2>&1 || notify "Failed to start OneDrive"

    sleep 2

    notify "Cloud mount status:"
    findmnt | grep -E 'Cloud|rclone|onedriver' || true
}

if is_any_mounted; then
    stop_cloud_mounts
else
    start_cloud_mounts
fi
```

##### Make Desktop Entry

cat > ~/Desktop/Cloud_Mounts_Toggle.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=Toggle Cloud Mounts
Comment=Mount or unmount Box, Google Drive, and OneDrive
Exec=bash -lc '/home/uutzinger/.local/bin/cloud-mounts-toggle toggle; echo; read -p "Press Enter to close..."'
Icon=folder-remote
Terminal=true
Categories=Utility;FileManager;
EOF

chmod +x ~/Desktop/Cloud_Mounts_Toggle.desktop

mkdir -p ~/.local/share/applications

cp ~/Desktop/Cloud_Mounts_Toggle.desktop ~/.local/share/applications/Cloud_Mounts_Toggle.desktop
cp ~/Desktop/UA_ENGR_Mount.desktop ~/.local/share/applications/UA_ENGR_Mount.desktop

chmod +x ~/.local/share/applications/Cloud_Mounts_Toggle.desktop
chmod +x ~/.local/share/applications/UA_ENGR_Mount.desktop

update-desktop-database ~/.local/share/applications 2>/dev/null || true
