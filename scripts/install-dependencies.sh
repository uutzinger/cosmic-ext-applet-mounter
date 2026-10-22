#!/usr/bin/env bash
set -euo pipefail

# Installs host dependencies for cosmic-ext-applet-mounter.
#
# Supported distributions (where COSMIC itself is packaged):
#   Debian/Ubuntu/Pop!_OS  apt
#   Fedora                 dnf
#   Arch Linux             pacman (+ AUR helper for onedriver)
#   openSUSE Tumbleweed    zypper
#   Gentoo                 emerge
#
# This script can be fetched and run directly:
#   curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/install-dependencies.sh | bash
# Or run locally:
#   bash scripts/install-dependencies.sh --all

readonly RCLONE_MIN_VERSION="1.74.3"
readonly ONEDRIVER_MIN_VERSION="0.15.0"

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

read_os_release() {
    local key="${1:-ID}"
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        printf '%s' "${!key:-unknown}"
    else
        printf '%s' "unknown"
    fi
}

detect_distro() {
    local id_like
    id_like="$(read_os_release ID_LIKE)"
    case "$(read_os_release ID)" in
        ubuntu|pop|debian)
            printf '%s' "debian" ;;
        fedora|rhel|centos|rocky|almalinux)
            printf '%s' "fedora" ;;
        arch|manjaro|endeavouros)
            printf '%s' "arch" ;;
        opensuse*)
            printf '%s' "opensuse" ;;
        gentoo)
            printf '%s' "gentoo" ;;
        nixos)
            printf '%s' "nixos" ;;
        *)
            # Fallback to ID_LIKE for derivatives.
            case "$id_like" in
                *debian*) printf '%s' "debian" ;;
                *fedora*|*rhel*) printf '%s' "fedora" ;;
                *arch*) printf '%s' "arch" ;;
                *opensuse*|*suse*) printf '%s' "opensuse" ;;
                *) printf '%s' "unknown" ;;
            esac
            ;;
    esac
}

require_supported_distro() {
    local distro
    distro="$(detect_distro)"
    case "$distro" in
        debian|fedora|arch|opensuse|gentoo) return 0 ;;
        nixos)
            error "NixOS is not supported by this script."
            error "Add the required packages in your configuration.nix instead."
            exit 1
            ;;
        *)
            error "Unsupported distribution: $(read_os_release ID) (ID_LIKE: $(read_os_release ID_LIKE))"
            error "This script supports Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux, openSUSE Tumbleweed, and Gentoo."
            error "For other distributions, follow docs/dependency-installation.md manually."
            exit 1
            ;;
    esac
}

obs_repo_suffix() {
    # Package-manager families merge Ubuntu/Pop into Debian; OBS repository
    # names require the actual distribution identity instead.
    case "$(read_os_release ID)" in
        debian)
            case "$(read_os_release VERSION_ID)" in
                11) printf '%s' "Debian_11" ;;
                12) printf '%s' "Debian_12" ;;
            esac
            ;;
        ubuntu|pop)
            case "$(read_os_release VERSION_ID)" in
                22.04) printf '%s' "xUbuntu_22.04" ;;
                24.04) printf '%s' "xUbuntu_24.04" ;;
                24.10) printf '%s' "xUbuntu_24.10" ;;
                25.04) printf '%s' "xUbuntu_25.04" ;;
            esac
            ;;
        opensuse*)
            printf '%s' "openSUSE_Tumbleweed" ;;
    esac
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
# Package manager abstraction
# ---------------------------------------------------------------------------

pkg_update_index() {
    case "$(detect_distro)" in
        debian)
            run_sudo apt update ;;
        fedora)
            run_sudo dnf makecache ;;
        arch)
            run_sudo pacman -Sy ;;
        opensuse)
            run_sudo zypper refresh ;;
        gentoo)
            run_sudo emerge --sync ;;
    esac
}

pkg_install() {
    case "$(detect_distro)" in
        debian)
            run_sudo apt install --no-install-recommends --no-install-suggests -y "$@" ;;
        fedora)
            run_sudo dnf install -y "$@" ;;
        arch)
            run_sudo pacman -Sy --noconfirm "$@" ;;
        opensuse)
            run_sudo zypper --non-interactive install "$@" ;;
        gentoo)
            warn "Gentoo packages are built from source and may take a long time."
            run_sudo emerge --ask=n -v "$@" ;;
    esac
}

# ---------------------------------------------------------------------------
# Base system dependencies
# ---------------------------------------------------------------------------

install_base() {
    info "Checking base system dependencies..."

    local distro
    distro="$(detect_distro)"

    local packages=""
    case "$distro" in
        debian)
            if ! command -v fusermount3 >/dev/null 2>&1; then packages="${packages} fuse3"; fi
            if ! command -v nmcli >/dev/null 2>&1; then packages="${packages} network-manager"; fi
            ;;
        fedora)
            if ! command -v fusermount3 >/dev/null 2>&1; then packages="${packages} fuse3"; fi
            if ! command -v nmcli >/dev/null 2>&1; then packages="${packages} NetworkManager"; fi
            ;;
        arch)
            if ! command -v fusermount3 >/dev/null 2>&1; then packages="${packages} fuse3"; fi
            if ! command -v nmcli >/dev/null 2>&1; then packages="${packages} networkmanager"; fi
            ;;
        opensuse)
            if ! command -v fusermount3 >/dev/null 2>&1; then packages="${packages} fuse3"; fi
            if ! command -v nmcli >/dev/null 2>&1; then packages="${packages} NetworkManager"; fi
            ;;
        gentoo)
            if ! command -v fusermount3 >/dev/null 2>&1; then packages="${packages} sys-fs/fuse"; fi
            if ! command -v nmcli >/dev/null 2>&1; then packages="${packages} net-misc/networkmanager"; fi
            ;;
    esac

    if [ -n "$packages" ]; then
        info "Installing:${packages}"
        pkg_update_index
        # shellcheck disable=SC2086
        pkg_install ${packages}
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

rclone_selfupdate_package() {
    case "$(detect_distro)" in
        debian) printf '%s' "deb" ;;
        fedora) printf '%s' "rpm" ;;
        *) printf '%s' "" ;;
    esac
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
        local pkg
        pkg="$(rclone_selfupdate_package)"
        info "Attempting package-aware self-update (--rclone-selfupdate was given)..."
        if [ -n "$pkg" ]; then
            if run_sudo rclone selfupdate --stable --package "$pkg"; then
                hash -r
                current_version="$(get_rclone_version)"
                if version_ge "$current_version" "$RCLONE_MIN_VERSION"; then
                    info "rclone updated to ${current_version}."
                    return 0
                fi
            fi
        else
            if run_sudo rclone selfupdate --stable; then
                hash -r
                current_version="$(get_rclone_version)"
                if version_ge "$current_version" "$RCLONE_MIN_VERSION"; then
                    info "rclone updated to ${current_version}."
                    return 0
                fi
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

detect_aur_helper() {
    if command -v paru >/dev/null 2>&1; then
        printf '%s' "paru"
    elif command -v yay >/dev/null 2>&1; then
        printf '%s' "yay"
    else
        printf '%s' ""
    fi
}

install_onedriver_debian() {
    local suffix
    suffix="$(obs_repo_suffix)"
    if [ -z "$suffix" ]; then
        error "No Open Build Service repository mapping for $(read_os_release ID) $(read_os_release VERSION_ID)."
        error "Install onedriver manually from: https://software.opensuse.org/download.html?project=home%3Ajstaf&package=onedriver"
        exit 1
    fi

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

    pkg_update_index
    pkg_install onedriver
}

install_onedriver_fedora() {
    info "Enabling jstaf/onedriver COPR..."
    run_sudo dnf copr enable -y jstaf/onedriver
    pkg_update_index
    pkg_install onedriver
}

install_onedriver_arch() {
    local helper
    helper="$(detect_aur_helper)"
    if [ -z "$helper" ]; then
        error "An AUR helper (yay or paru) is required to install onedriver on Arch Linux."
        error "Install one manually, then re-run this script, or install onedriver from the AUR yourself:"
        error "  https://aur.archlinux.org/packages/onedriver"
        exit 1
    fi
    info "Installing onedriver from AUR using ${helper}..."
    "$helper" -Sy --noconfirm onedriver
}

install_onedriver_opensuse() {
    local suffix
    suffix="$(obs_repo_suffix)"
    if [ -z "$suffix" ]; then
        error "No Open Build Service repository mapping for openSUSE."
        error "Install onedriver manually from: https://software.opensuse.org/download.html?project=home%3Ajstaf&package=onedriver"
        exit 1
    fi

    info "Adding home:jstaf OBS repository for ${suffix}..."
    run_sudo zypper --non-interactive addrepo \
        "https://download.opensuse.org/repositories/home:/jstaf/${suffix}/home:jstaf.repo"
    run_sudo zypper --gpg-auto-import-keys refresh
    pkg_install onedriver
}

install_onedriver_gentoo() {
    error "Gentoo does not have an official onedriver package."
    error "Build and install onedriver manually from: https://github.com/jstaf/onedriver"
    exit 1
}

install_onedriver() {
    info "Checking jstaf/onedriver..."
    local current_version
    current_version="$(get_onedriver_version)"

    if version_ge "$current_version" "$ONEDRIVER_MIN_VERSION"; then
        info "onedriver ${current_version} already meets the minimum (${ONEDRIVER_MIN_VERSION})."
        return 0
    fi

    warn "onedriver ${current_version} is below the recommended minimum ${ONEDRIVER_MIN_VERSION}."

    case "$(detect_distro)" in
        debian) install_onedriver_debian ;;
        fedora) install_onedriver_fedora ;;
        arch) install_onedriver_arch ;;
        opensuse) install_onedriver_opensuse ;;
        gentoo) install_onedriver_gentoo ;;
    esac

    current_version="$(get_onedriver_version)"
    if ! version_ge "$current_version" "$ONEDRIVER_MIN_VERSION"; then
        error "onedriver installation failed or version is still too old: ${current_version}"
        exit 1
    fi
    info "onedriver ${current_version} installed."
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
    printf '  %-12s %s\n' "fusermount3" "$(command -v fusermount3 >/dev/null 2>&1 && echo present || echo missing)"
    printf '  %-12s %s\n' "nmcli" "$(command -v nmcli >/dev/null 2>&1 && echo present || echo missing)"
}

# ---------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------

usage() {
    cat <<EOF
Usage: $0 [OPTION]...

Install or update host dependencies for cosmic-ext-applet-mounter.

Options:
  --all                Install/update supported engines (rclone and onedriver).
  --base               Install base tools only (fuse3, network-manager).
  --rclone             Install/update rclone (new OneDrive mirrors and other providers).
  --rclone-selfupdate  Try rclone's package-aware selfupdate first (opt-in).
  --onedriver          Install/update jstaf/onedriver (OneDrive Online mounts).
  --help               Show this help message.
  --log-dir DIR        Mirror visible output to a new private log/report directory.

With no options, --all is assumed.

Supported distributions: Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux,
openSUSE Tumbleweed, and Gentoo.
EOF
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

# BEGIN standalone setup reporting (kept identical in both setup scripts)
setup_json_string() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    printf '"%s"' "$value"
}

setup_write_report() {
    local status="$1" code="$2" temporary
    temporary="$(mktemp "${setup_report_dir}/.summary.XXXXXX")" || return
    {
        printf '{"schema_version":1,"task":'
        setup_json_string "$setup_report_task"
        printf ',"status":"%s","exit_code":%s,"log_path":' "$status" "$code"
        setup_json_string "${setup_report_dir}/task.log"
        printf ',"summary_path":'
        setup_json_string "${setup_report_dir}/summary.json"
        printf ',"next_action":"Inspect partial changes before retrying; do not publish raw logs."}\n'
    } > "$temporary"
    mv -- "$temporary" "${setup_report_dir}/summary.json"
}

setup_finish_report() {
    local code="$?" status=failed
    trap - EXIT INT TERM
    set +e
    exec 1>&3 2>&4
    wait "$setup_stdout_pid"
    local stdout_code="$?"
    wait "$setup_stderr_pid"
    local stderr_code="$?"
    if (( code == 0 && (stdout_code != 0 || stderr_code != 0) )); then code=1; fi
    if (( code == 0 )); then status=succeeded; fi
    if (( code == 130 || code == 143 )); then status=cancelled; fi
    setup_write_report "$status" "$code"
    if (( $? != 0 )); then
        printf 'Could not finalize setup report.\n' >&2
        if (( code == 0 )); then code=1; fi
    fi
    printf 'Setup %s (exit %s). Report: %s/summary.json\n' "$status" "$code" "$setup_report_dir"
    exit "$code"
}

setup_entry() {
    local requested_log_dir="" option
    local -a setup_arguments=()
    for option in "$@"; do
        case "$option" in --help|-h|help) usage; return 0 ;; esac
    done
    while (( $# )); do
        case "$1" in
            --log-dir)
                if (( $# < 2 )) || [[ -z "$2" || -n "$requested_log_dir" ]]; then
                    printf 'One nonempty --log-dir NEW_DIRECTORY is required.\n' >&2
                    return 2
                fi
                requested_log_dir="$2"
                shift 2 ;;
            *) setup_arguments+=("$1"); shift ;;
        esac
    done
    if [[ -n "$requested_log_dir" ]]; then
        for option in tee mktemp mv; do
            command -v "$option" >/dev/null || {
                printf 'Reporting requires %s.\n' "$option" >&2
                return 127
            }
        done
        if [[ "$requested_log_dir" =~ [[:cntrl:]] ]]; then
            printf 'Control characters are not allowed in report paths.\n' >&2
            return 2
        fi
        # mkdir (without -p) refuses existing files/directories and symbolic links.
        (umask 077; mkdir -m 700 -- "$requested_log_dir") || return
        setup_report_dir="$(cd -- "$requested_log_dir" && pwd -P)"
        if [[ "$setup_report_dir" =~ [[:cntrl:]] ]]; then
            printf 'Control characters are not allowed in resolved report paths.\n' >&2
            return 2
        fi
        (umask 077; : > "${setup_report_dir}/task.log")
        setup_write_report running null
        exec 3>&1 4>&2
        exec > >(tee -a "${setup_report_dir}/task.log" >&3)
        setup_stdout_pid=$!
        exec 2> >(tee -a "${setup_report_dir}/task.log" >&4)
        setup_stderr_pid=$!
        trap setup_finish_report EXIT
        trap 'exit 130' INT
        trap 'exit 143' TERM
    fi
    main "${setup_arguments[@]}"
}
# END standalone setup reporting

main() {
    local do_all=false
    local do_base=false
    local do_rclone=false
    local do_onedriver=false

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
            --help) usage; exit 0 ;;
            *) error "Unknown option: $1"; usage; exit 1 ;;
        esac
        shift
    done

    if [ "$do_all" = true ]; then
        do_base=true
        do_rclone=true
        do_onedriver=true
    fi

    require_supported_distro
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
    print_summary
    info "Done."
}

if [[ "${BASH_SOURCE[0]:-$0}" == "$0" ]]; then
    setup_report_task=dependency-setup
    setup_entry "$@"
fi
