#!/usr/bin/env bash
set -euo pipefail

# Installs the OpenConnect packages and the on-demand nm-applet authentication
# agent required by cosmic-ext-applet-mounter.
#
# Supported distributions (where COSMIC itself is packaged):
#   Debian/Ubuntu/Pop!_OS  apt
#   Fedora                 dnf
#   Arch Linux             pacman
#   openSUSE Tumbleweed    zypper
#   Gentoo                 emerge
#
# This script can be fetched and run directly:
#   curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/setup-openconnect.sh | bash -s -- install
# Or to uninstall:
#   curl -fsSL https://raw.githubusercontent.com/uutzinger/cosmic-ext-applet-mounter/main/scripts/setup-openconnect.sh | bash -s -- uninstall
#
# The VPN profile itself must still be created with nm-connection-editor.

readonly SERVICE_NAME="cosmic-mounter-openconnect-agent.service"
readonly SERVICE_PATH="${HOME}/.config/systemd/user/${SERVICE_NAME}"

info() { printf '\033[34m[INFO]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[WARN]\033[0m %s\n' "$*" >&2; }
error() { printf '\033[31m[ERROR]\033[0m %s\n' "$*" >&2; }

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
            error "Add the COSMIC modules and required packages in your configuration.nix instead:"
            error "  services.desktopManager.cosmic.enable = true;"
            error "  environment.systemPackages = with pkgs; [ openconnect networkmanager-openconnect network-manager-applet ];"
            exit 1
            ;;
        *)
            error "Unsupported distribution: $(read_os_release ID) (ID_LIKE: $(read_os_release ID_LIKE))"
            error "This script supports Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux, openSUSE Tumbleweed, and Gentoo."
            exit 1
            ;;
    esac
}

service_unit() {
    cat <<'EOF'
[Unit]
Description=NetworkManager OpenConnect authentication agent for Cloud Mounter
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=dbus
BusName=org.freedesktop.network-manager-applet
ExecStart=/usr/bin/nm-applet
# Bus-name ownership precedes NetworkManager secret-agent registration. Keep
# RestartUnit pending briefly so Cloud Mounter cannot activate the VPN too early.
ExecStartPost=/usr/bin/sleep 1
Restart=on-failure
TimeoutStartSec=10
TimeoutStopSec=5
KillMode=control-group
EOF
}

hide_nm_applet_icon() {
    if command -v gsettings >/dev/null 2>&1; then
        info "Hiding nm-applet status icon when the agent runs..."
        gsettings set org.gnome.nm-applet show-applet false
    else
        warn "gsettings not found; skipping show-applet configuration."
    fi
}

install_packages() {
    local distro
    distro="$(detect_distro)"

    info "Detected distribution family: ${distro}"
    require_sudo

    case "$distro" in
        debian)
            info "Installing OpenConnect packages with apt..."
            run_sudo apt update
            run_sudo apt install -y \
                openconnect \
                network-manager-openconnect \
                network-manager-openconnect-gnome \
                network-manager-gnome
            ;;
        fedora)
            info "Installing OpenConnect packages with dnf..."
            # NetworkManager-openconnect provides the OpenConnect plugin and auth
            # dialog; network-manager-applet provides nm-applet and nm-connection-editor.
            run_sudo dnf install -y \
                openconnect \
                NetworkManager-openconnect \
                network-manager-applet
            ;;
        arch)
            info "Installing OpenConnect packages with pacman..."
            run_sudo pacman -Sy --noconfirm \
                openconnect \
                networkmanager-openconnect \
                network-manager-applet
            ;;
        opensuse)
            info "Installing OpenConnect packages with zypper..."
            run_sudo zypper --non-interactive install \
                openconnect \
                NetworkManager-openconnect \
                network-manager-applet
            ;;
        gentoo)
            warn "Gentoo packages are built from source and may take a long time."
            info "Installing OpenConnect packages with emerge..."
            run_sudo emerge --ask=n -v \
                net-vpn/openconnect \
                net-misc/networkmanager-openconnect \
                net-misc/network-manager-applet
            ;;
        *)
            error "Unsupported distribution: ${distro}"
            exit 1
            ;;
    esac
}

install_openconnect() {
    require_supported_distro
    install_packages
    hide_nm_applet_icon

    info "Creating on-demand authentication agent service..."
    mkdir -p "${HOME}/.config/systemd/user"
    service_unit > "$SERVICE_PATH"

    info "Reloading systemd user manager..."
    systemctl --user daemon-reload

    info "Installation complete."
    echo
    info "Next steps:"
    info "  1. Create the VPN profile by running:"
    info ""
    info "       nm-connection-editor"
    info ""
    info "  2. Select the saved profile as the VPN dependency in Cloud Mounter."
    echo
    info "Do not start or enable ${SERVICE_NAME} manually;"
    info "Cloud Mounter will start it on demand before each OpenConnect activation."
}

uninstall_openconnect() {
    info "Stopping authentication agent service..."
    systemctl --user stop "$SERVICE_NAME" 2>/dev/null || true

    if [ -f "$SERVICE_PATH" ]; then
        info "Removing service unit..."
        rm -f "$SERVICE_PATH"
    else
        warn "Service unit not found at ${SERVICE_PATH}"
    fi

    info "Reloading systemd user manager..."
    systemctl --user daemon-reload

    local distro
    distro="$(detect_distro)"

    echo
    warn "The OpenConnect packages and saved VPN profile were not removed."
    warn "To remove packages, first delete the VPN profile in nm-connection-editor,"
    warn "then use your distribution's package manager, for example:"
    case "$distro" in
        debian)
            warn "  sudo apt remove network-manager-openconnect-gnome network-manager-openconnect openconnect" ;;
        fedora)
            warn "  sudo dnf remove NetworkManager-openconnect openconnect" ;;
        arch)
            warn "  sudo pacman -R networkmanager-openconnect openconnect" ;;
        opensuse)
            warn "  sudo zypper remove NetworkManager-openconnect openconnect" ;;
        gentoo)
            warn "  sudo emerge -c net-misc/networkmanager-openconnect net-vpn/openconnect" ;;
    esac
    warn "Keep network-manager-applet (or network-manager-gnome) if you still need"
    warn "nm-connection-editor for other NetworkManager profiles."
}

usage() {
    cat <<EOF
Usage: $0 {install|uninstall|help}
       $0 {install|uninstall} --log-dir NEW_DIRECTORY

Commands:
  install     Install OpenConnect packages and the on-demand nm-applet agent.
  uninstall   Remove the on-demand nm-applet agent service.
  help        Show this help message.

Supported distributions: Debian, Ubuntu, Pop!_OS, Fedora, Arch Linux,
openSUSE Tumbleweed, and Gentoo.

The VPN profile must still be created with nm-connection-editor after install.
EOF
}

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
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi

    case "$1" in
        install)
            install_openconnect
            ;;
        uninstall)
            uninstall_openconnect
            ;;
        help|--help|-h)
            usage
            exit 0
            ;;
        *)
            error "Unknown command: $1"
            usage
            exit 1
            ;;
    esac
}

if [[ "${BASH_SOURCE[0]:-$0}" == "$0" ]]; then
    setup_report_task=openconnect-setup
    setup_entry "$@"
fi
