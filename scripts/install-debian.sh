#!/usr/bin/env bash
# Standalone GitHub package installer; also works when piped into bash.
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: install-debian.sh [--version X.Y.Z] [--help]
Install Cloud Mounter from GitHub Releases (default: 0.5.4).
Automatically selects Ubuntu/Pop!_OS 22.04, 24.04 or 26.04 and amd64/arm64.
Verifies SHA256SUMS and package identity before APT installation/reinstallation.
APT installs declared dependencies; optional storage engines need separate setup.
Close Cloud Mounter windows and remove the applet from the panel first.
Downloads are temporary. APT changes require root or sudo; no release is built.
EOF
}

fail() { printf 'install-debian: %s\n' "$*" >&2; return 1; }

detect_platform() {
    local ID='' VERSION_ID=''
    # /etc/os-release is distribution-owned configuration, not network input.
    source "${1:-/etc/os-release}"
    case "$ID" in
        ubuntu|pop) ;;
        *) fail "Unsupported distribution: $ID (Ubuntu or Pop!_OS required)."; return 1 ;;
    esac
    case "$VERSION_ID" in
        22.04|24.04|26.04) ubuntu_version="$VERSION_ID" ;;
        *) fail "No package for $ID $VERSION_ID."; return 1 ;;
    esac
    architecture=$(dpkg --print-architecture)
    case "$architecture" in
        amd64|arm64) ;;
        *) fail "No package for architecture $architecture."; return 1 ;;
    esac
}

download() {
    curl --fail --silent --show-error --location --proto '=https' \
        --proto-redir '=https' --connect-timeout 20 --max-time 300 \
        --output "$2" "$1"
}

install_package() {
    # --yes works with curl | bash; sudo obtains passwords from the terminal.
    if (( EUID == 0 )); then
        apt-get install --yes --reinstall "$1"
    else
        sudo apt-get install --yes --reinstall "$1"
    fi
}

main() (
    local version=0.5.4 ubuntu_version architecture tool workdir='' status
    while (( $# )); do
        case "$1" in
            --help|-h) usage; return 0 ;;
            --version)
                (( $# >= 2 )) || { fail '--version needs X.Y.Z'; return 1; }
                version="${2#v}"; shift 2 ;;
            *) fail "Unknown option: $1"; return 1 ;;
        esac
    done
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { fail 'Invalid version.'; return 1; }
    for tool in curl dpkg dpkg-deb sha256sum awk mktemp chmod apt-get; do
        command -v "$tool" >/dev/null || { fail "Required command missing: $tool"; return 1; }
    done
    if (( EUID != 0 )); then
        command -v sudo >/dev/null || { fail 'sudo is required.'; return 1; }
    fi
    detect_platform
    local filename="cosmic-ext-applet-mounter_${version}_ubuntu${ubuntu_version}_${architecture}.deb"
    local base="https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v${version}"
    printf 'Installing Cloud Mounter %s for Ubuntu %s / %s\n' "$version" "$ubuntu_version" "$architecture"
    workdir=$(mktemp -d /tmp/cloud-mounter-debian.XXXXXXXX)
    trap 'status=$?; trap - EXIT; rm -rf -- "$workdir"; printf '\''{"task":"debian-install","exit_code":%s}\n'\'' "$status"; exit "$status"' EXIT
    download "$base/SHA256SUMS" "$workdir/SHA256SUMS"
    download "$base/$filename" "$workdir/$filename"
    local expected
    expected=$(awk -v name="$filename" '$2 == name || $2 == "*" name {print $1}' "$workdir/SHA256SUMS")
    [[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || { fail 'Missing or ambiguous package checksum.'; return 1; }
    printf '%s  %s\n' "$expected" "$filename" > "$workdir/selected.sha256"
    (cd "$workdir" && sha256sum --check selected.sha256)
    [[ $(dpkg-deb --field "$workdir/$filename" Package) == cosmic-ext-applet-mounter \
        && $(dpkg-deb --field "$workdir/$filename" Version) == "$version" \
        && $(dpkg-deb --field "$workdir/$filename" Architecture) == "$architecture" ]] \
        || { fail 'Downloaded package identity does not match the selected release.'; return 1; }
    # Let APT's unprivileged downloader read the verified local package.
    chmod 755 "$workdir"
    chmod 644 "$workdir/$filename"
    install_package "$workdir/$filename"
    printf 'Installed. Add Cloud Mounter in COSMIC Settings > Desktop > Panel > Applets.\n'
)

if [[ "${BASH_SOURCE[0]:-$0}" == "$0" ]]; then
    main "$@"
fi
