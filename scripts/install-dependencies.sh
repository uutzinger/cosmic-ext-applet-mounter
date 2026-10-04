#!/usr/bin/env bash
set -euo pipefail

# Installs host dependencies for cosmic-ext-applet-mounter on Debian/Ubuntu/Pop!_OS.
# This script can be fetched and run directly:
#   curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/install-dependencies.sh | bash
# Or run locally:
#   bash scripts/install-dependencies.sh --all

readonly RCLONE_MIN_VERSION="1.74.3"
readonly ONEDRIVER_MIN_VERSION="0.15.0"
readonly ONEDRIVE_MIN_VERSION="2.5.10"

# Set to true with --rclone-selfupdate to try rclone's package-aware selfupdate
# before falling back to the official installer.
PREFER_RCLONE_SELFUPDATE=false

# ---------------------------------------------------------------------------
# Output helpers
# ---------------------------------------------------------------------------

info() { printf '\033[34m[INFO]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[WARN]\033[0m %s\n' "$*" >&2; }
error() { printf '\033[31m[ERROR]\033[0m %s\n' "$*" >&2; }

# ---------------------------------------------------------------------------
# Version comparison: returns 0 if $1 >= $2
# ---------------------------------------------------------------------------

version_ge() {
    local v1="$1"
    local v2="$2"
    if [ "$(printf '%s\n%s\n' "$v1" "$v2" | sort -V | head -n1)" = "$v2" ]; then
        return 0
    fi
    return 1
}

# ---------------------------------------------------------------------------
# Distro detection
# ---------------------------------------------------------------------------

OS_ID=""
OS_VERSION=""
read_os_release() {
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        OS_ID="${ID:-unknown}"
        OS_VERSION="${VERSION_ID:-unknown}"
    else
        OS_ID="unknown"
        OS_VERSION="unknown"
    fi
}
read_os_release

obs_repo_suffix() {
    case "$OS_ID" in
        ubuntu|pop)
            case "$OS_VERSION" in
                22.04) printf '%s' "xUbuntu_22.04" ;;
                24.04) printf '%s' "xUbuntu_24.04" ;;
                24.10) printf '%s' "xUbuntu_24.10" ;;
                25.04) printf '%s' "xUbuntu_25.04" ;;
            esac
            ;;
        debian)
            case "$OS_VERSION" in
                11) printf '%s' "Debian_11" ;;
                12) printf '%s' "Debian_12" ;;
            esac
            ;;
    esac
}

require_debian_family() {
    case "$OS_ID" in
        ubuntu|pop|debian) return 0 ;;
    esac
    error "This script currently supports Debian, Ubuntu, and Pop!_OS."
    error "Detected OS: ${OS_ID} ${OS_VERSION}"
    error "For other distributions, follow Dependency Installation.md manually."
    exit 1
}

# ---------------------------------------------------------------------------
# Command availability
# ---------------------------------------------------------------------------

require_sudo() {
    if [ "$(id -u)" -eq 0 ]; then
        return 0
    fi
    if command -v sudo >/dev/null 2>&1; then
        return 0
    fi
    error "This script needs root privileges. Please run as root or install sudo."
    exit 1
}

run_sudo() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    else
        sudo "$@"
    fi
}

# ---------------------------------------------------------------------------
# Base system dependencies
# ---------------------------------------------------------------------------

install_base() {
    info "Checking base system dependencies..."
    local packages=""

    if ! command -v fusermount3 >/dev/null 2>&1; then
        packages="${packages} fuse3"
    fi

    if ! command -v fuser >/dev/null 2>&1; then
        packages="${packages} psmisc"
    fi

    if ! command -v nmcli >/dev/null 2>&1; then
        packages="${packages} network-manager"
    fi

    if [ -n "$packages" ]; then
        info "Installing:${packages}"
        run_sudo apt update
        # shellcheck disable=SC2086
        run_sudo apt install --no-install-recommends --no-install-suggests -y ${packages}
    else
        info "Base dependencies already present."
    fi
}

# ---------------------------------------------------------------------------
# rclone
# ---------------------------------------------------------------------------

get_rclone_version() {
    if command -v rclone >/dev/null 2>&1; then
        rclone version | awk 'NR==1 {print $2}' | sed 's/^v//'
    else
        printf '%s' "0.0.0"
    fi
}

backup_rclone_config() {
    local cfg="${HOME}/.config/rclone/rclone.conf"
    if [ -f "$cfg" ]; then
        local backup="${cfg}.backup.$(date +%Y%m%d-%H%M%S)"
        info "Backing up rclone config to ${backup}"
        cp --preserve=all "$cfg" "$backup"
    fi
}

install_rclone() {
    info "Checking rclone..."
    local current_version
    current_version="$(get_rclone_version)"

    if version_ge "$current_version" "$RCLONE_MIN_VERSION"; then
        info "rclone ${current_version} already meets the minimum (${RCLONE_MIN_VERSION})."
        return 0
    fi

    warn "rclone ${current_version} is below the recommended minimum ${RCLONE_MIN_VERSION}."
    backup_rclone_config

    if [ "$PREFER_RCLONE_SELFUPDATE" = true ] && command -v rclone >/dev/null 2>&1; then
        info "Attempting package-aware self-update (--rclone-selfupdate was given)..."
        if run_sudo rclone selfupdate --stable --package deb; then
            hash -r
            current_version="$(get_rclone_version)"
            if version_ge "$current_version" "$RCLONE_MIN_VERSION"; then
                info "rclone updated to ${current_version}."
                return 0
            fi
        fi
        warn "rclone selfupdate did not produce a usable version; falling back to the official installer."
    fi

    info "Downloading and running the official rclone installer..."
    local installer="/tmp/rclone-install-$$.sh"
    curl --fail --show-error --silent https://rclone.org/install.sh --output "$installer"
    run_sudo bash "$installer"
    rm -f "$installer"
    hash -r

    current_version="$(get_rclone_version)"
    if ! version_ge "$current_version" "$RCLONE_MIN_VERSION"; then
        error "rclone installation failed or version is still too old: ${current_version}"
        exit 1
    fi
    info "rclone ${current_version} installed."
}

# ---------------------------------------------------------------------------
# jstaf/onedriver
# ---------------------------------------------------------------------------

get_onedriver_version() {
    if command -v onedriver >/dev/null 2>&1; then
        onedriver --version 2>/dev/null | awk '{print $1}' | sed 's/^v//'
    else
        printf '%s' "0.0.0"
    fi
}

install_onedriver() {
    info "Checking jstaf/onedriver..."
    local current_version
    current_version="$(get_onedriver_version)"

    if version_ge "$current_version" "$ONEDRIVER_MIN_VERSION"; then
        info "onedriver ${current_version} already meets the minimum (${ONEDRIVER_MIN_VERSION})."
        return 0
    fi

    local suffix
    suffix="$(obs_repo_suffix)"
    if [ -z "$suffix" ]; then
        error "No Open Build Service repository mapping for ${OS_ID} ${OS_VERSION}."
        error "Install onedriver manually from: https://software.opensuse.org/download.html?project=home%3Ajstaf&package=onedriver"
        exit 1
    fi

    warn "onedriver ${current_version} is below the recommended minimum ${ONEDRIVER_MIN_VERSION}."
    info "Adding home:jstaf OBS repository for ${suffix}..."

    local key_url="https://download.opensuse.org/repositories/home:/jstaf/${suffix}/Release.key"
    local list_file="/etc/apt/sources.list.d/onedriver.list"
    local keyring="/usr/share/keyrings/obs-onedriver.gpg"

    curl --fail --show-error --silent "$key_url" \
        | gpg --dearmor \
        | run_sudo tee "$keyring" >/dev/null

    printf 'deb [arch=%s signed-by=%s] https://download.opensuse.org/repositories/home:/jstaf/%s/ /\n' \
        "$(dpkg --print-architecture)" "$keyring" "$suffix" \
        | run_sudo tee "$list_file" >/dev/null

    run_sudo apt update
    run_sudo apt install --no-install-recommends --no-install-suggests -y onedriver

    current_version="$(get_onedriver_version)"
    if ! version_ge "$current_version" "$ONEDRIVER_MIN_VERSION"; then
        error "onedriver installation failed or version is still too old: ${current_version}"
        exit 1
    fi
    info "onedriver ${current_version} installed."
}

# ---------------------------------------------------------------------------
# abraunegg/onedrive
# ---------------------------------------------------------------------------

get_onedrive_version() {
    if command -v onedrive >/dev/null 2>&1; then
        onedrive --version 2>/dev/null | head -n1 | awk '{print $1}' | sed 's/^v//'
    else
        printf '%s' "0.0.0"
    fi
}

install_onedrive() {
    info "Checking abraunegg/onedrive..."
    local current_version
    current_version="$(get_onedrive_version)"

    if version_ge "$current_version" "$ONEDRIVE_MIN_VERSION"; then
        info "onedrive ${current_version} already meets the minimum (${ONEDRIVE_MIN_VERSION})."
        return 0
    fi

    local suffix
    suffix="$(obs_repo_suffix)"
    if [ -z "$suffix" ]; then
        error "No Open Build Service repository mapping for ${OS_ID} ${OS_VERSION}."
        error "Install abraunegg/onedrive manually from: https://github.com/abraunegg/onedrive/blob/master/docs/ubuntu-package-install.md"
        exit 1
    fi

    warn "onedrive ${current_version} is below the recommended minimum ${ONEDRIVE_MIN_VERSION}."
    info "Adding npreining OBS repository for ${suffix}..."

    local key_url="https://download.opensuse.org/repositories/home:/npreining:/debian-ubuntu-onedrive/${suffix}/Release.key"
    local list_file="/etc/apt/sources.list.d/onedrive.list"
    local keyring="/usr/share/keyrings/obs-onedrive-mirror.gpg"

    curl --fail --show-error --silent "$key_url" \
        | gpg --dearmor \
        | run_sudo tee "$keyring" >/dev/null

    printf 'deb [arch=%s signed-by=%s] https://download.opensuse.org/repositories/home:/npreining:/debian-ubuntu-onedrive/%s/ /\n' \
        "$(dpkg --print-architecture)" "$keyring" "$suffix" \
        | run_sudo tee "$list_file" >/dev/null

    run_sudo apt update
    run_sudo apt-cache policy onedrive
    run_sudo apt install --no-install-recommends --no-install-suggests -y onedrive

    current_version="$(get_onedrive_version)"
    if ! version_ge "$current_version" "$ONEDRIVE_MIN_VERSION"; then
        error "onedrive installation failed or version is still too old: ${current_version}"
        exit 1
    fi
    info "onedrive ${current_version} installed."
}

# ---------------------------------------------------------------------------
# Cisco Secure Client check only
# ---------------------------------------------------------------------------

check_cisco() {
    info "Checking Cisco Secure Client (optional, organization-provided)..."
    local cli="/opt/cisco/secureclient/bin/vpn"
    local agent_service="vpnagentd.service"

    if [ ! -x "$cli" ]; then
        warn "Cisco Secure Client CLI not found at ${cli}."
        warn "Install it using your organization's provided package."
        return 0
    fi

    info "Cisco CLI found."
    if systemctl is-active --quiet "$agent_service" 2>/dev/null; then
        info "${agent_service} is active."
    else
        info "${agent_service} is not active."
        info "To enable it:"
        info "  sudo systemctl enable ${agent_service}"
        info "To start it:"
        info "  sudo systemctl start ${agent_service}"

    fi
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

print_summary() {
    info "Dependency status:"
    printf '  %-12s %s\n' "rclone" "$(get_rclone_version) (min ${RCLONE_MIN_VERSION})"
    if command -v onedriver >/dev/null 2>&1; then
        printf '  %-12s %s\n' "onedriver" "$(get_onedriver_version) (min ${ONEDRIVER_MIN_VERSION})"
    else
        printf '  %-12s %s\n' "onedriver" "not installed (min ${ONEDRIVER_MIN_VERSION})"
    fi
    if command -v onedrive >/dev/null 2>&1; then
        printf '  %-12s %s\n' "onedrive" "$(get_onedrive_version) (min ${ONEDRIVE_MIN_VERSION})"
    else
        printf '  %-12s %s\n' "onedrive" "not installed (min ${ONEDRIVE_MIN_VERSION})"
    fi
    printf '  %-12s %s\n' "fusermount3" "$(command -v fusermount3 >/dev/null 2>&1 && echo present || echo missing)"
    printf '  %-12s %s\n' "nmcli" "$(command -v nmcli >/dev/null 2>&1 && echo present || echo missing)"
    printf '  %-12s %s\n' "fuser" "$(command -v fuser >/dev/null 2>&1 && echo present || echo missing)"
}

# ---------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------

usage() {
    cat <<EOF
Usage: $0 [OPTION]...

Install or update host dependencies for cosmic-ext-applet-mounter.

Options:
  --all                Install/update base tools, rclone, onedriver, and onedrive.
  --base               Install base tools only (fuse3, psmisc, network-manager).
  --rclone             Install/update rclone using the official installer.
  --rclone-selfupdate  Try rclone's package-aware selfupdate first (opt-in).
  --onedriver          Install/update jstaf/onedriver.
  --onedrive           Install/update abraunegg/onedrive.
  --cisco-check        Check Cisco Secure Client status without installing.
  --help               Show this help message.

With no options, --all is assumed.
EOF
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

main() {
    local do_all=false
    local do_base=false
    local do_rclone=false
    local do_onedriver=false
    local do_onedrive=false
    local do_cisco=false

    if [ $# -eq 0 ]; then
        do_all=true
    fi

    while [ $# -gt 0 ]; do
        case "$1" in
            --all) do_all=true ;;
            --base) do_base=true ;;
            --rclone) do_rclone=true ;;
            --rclone-selfupdate) PREFER_RCLONE_SELFUPDATE=true ;;
            --onedriver) do_onedriver=true ;;
            --onedrive) do_onedrive=true ;;
            --cisco-check) do_cisco=true ;;
            --help) usage; exit 0 ;;
            *) error "Unknown option: $1"; usage; exit 1 ;;
        esac
        shift
    done

    if [ "$do_all" = true ]; then
        do_base=true
        do_rclone=true
        do_onedriver=true
        do_onedrive=true
        do_cisco=true
    fi

    require_debian_family
    require_sudo

    if [ "$do_base" = true ]; then
        install_base
    fi
    if [ "$do_rclone" = true ]; then
        install_rclone
    fi
    if [ "$do_onedriver" = true ]; then
        install_onedriver
    fi
    if [ "$do_onedrive" = true ]; then
        install_onedrive
    fi
    if [ "$do_cisco" = true ]; then
        check_cisco
    fi

    print_summary
    info "Done."
}

main "$@"
