#!/usr/bin/env bash
# Standalone GitHub release installer; candidate installs use install-flatpak.sh.
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: install-flatpak-release.sh [--version X.Y.Z] [--bundle FILE] [--help]
Install Cloud Mounter from GitHub Releases (default: 0.5.5).
Detects x86_64 or aarch64 and verifies the matching bundle against SHA256SUMS.
Requires Flatpak, curl and sha256sum. Run as your desktop user, without sudo.
Adds user Flathub, installs Freedesktop Platform 25.08, then installs/updates
the user bundle. Missing assets fail before any Flatpak installation changes.
Close Cloud Mounter windows and remove the applet from the panel first.
Host storage engines/VPN components need separate setup. No workflow or build
is started; GitHub bundles do not provide automatic app updates.
For pre-release checks, --bundle FILE uses a local matching bundle and its
sibling SHA256SUMS instead of downloading unpublished GitHub release assets.
EOF
}

fail() { printf 'install-flatpak-release: %s\n' "$*" >&2; return 1; }

require_desktop_user() {
    (( EUID != 0 )) || { fail 'Run as your desktop user, without sudo.'; return 1; }
}

detect_architecture() {
    case "$(uname -m)" in
        x86_64|amd64) architecture=x86_64 ;;
        aarch64|arm64) architecture=aarch64 ;;
        *) fail 'No release bundle for this CPU architecture.'; return 1 ;;
    esac
}

download() {
    curl --fail --silent --show-error --location --proto '=https' \
        --proto-redir '=https' --connect-timeout 20 --max-time 300 \
        --output "$2" "$1"
}

main() (
    local version=0.5.5 architecture tool workdir='' status local_bundle=''
    while (( $# )); do
        case "$1" in
            --help|-h) usage; return 0 ;;
            --version)
                (( $# >= 2 )) || { fail '--version needs X.Y.Z'; return 1; }
                version="${2#v}"; shift 2 ;;
            --bundle)
                (( $# >= 2 )) || { fail '--bundle needs a filename'; return 1; }
                local_bundle="$2"; shift 2 ;;
            *) fail "Unknown option: $1"; return 1 ;;
        esac
    done
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { fail 'Invalid version.'; return 1; }
    require_desktop_user
    for tool in flatpak sha256sum awk uname mktemp; do
        command -v "$tool" >/dev/null || { fail "Required command missing: $tool"; return 1; }
    done
    detect_architecture
    local filename="io.github.uutzinger.cosmic-ext-applet-mounter-${architecture}-master.flatpak"
    if [[ -n "$local_bundle" ]]; then
        [[ "${local_bundle##*/}" == "$filename" && -f "$local_bundle" && ! -L "$local_bundle" ]] \
            || { fail 'Local bundle must be a regular file matching this CPU and app filename.'; return 1; }
        local local_sums="$(dirname -- "$local_bundle")/SHA256SUMS"
        [[ -f "$local_sums" && ! -L "$local_sums" ]] \
            || { fail 'Local bundle needs a regular sibling SHA256SUMS.'; return 1; }
        command -v cp >/dev/null || { fail 'Required command missing: cp'; return 1; }
    else
        command -v curl >/dev/null || { fail 'Required command missing: curl'; return 1; }
    fi
    local base="https://github.com/uutzinger/cosmic-ext-applet-mounter/releases/download/v${version}"
    printf 'Installing Cloud Mounter %s for %s\n' "$version" "$architecture"
    workdir=$(mktemp -d /tmp/cloud-mounter-flatpak-release.XXXXXXXX)
    trap 'status=$?; trap - EXIT; rm -rf -- "$workdir"; printf '\''{"task":"flatpak-release-install","exit_code":%s}\n'\'' "$status"; exit "$status"' EXIT
    if [[ -n "$local_bundle" ]]; then
        cp -- "$local_sums" "$workdir/SHA256SUMS"
        cp -- "$local_bundle" "$workdir/$filename"
    else
        download "$base/SHA256SUMS" "$workdir/SHA256SUMS"
        download "$base/$filename" "$workdir/$filename"
    fi
    local expected
    expected=$(awk -v name="$filename" '$2 == name || $2 == "*" name {print $1}' "$workdir/SHA256SUMS")
    [[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || { fail 'Missing or ambiguous bundle checksum.'; return 1; }
    printf '%s  %s\n' "$expected" "$filename" > "$workdir/selected.sha256"
    (cd "$workdir" && sha256sum --check selected.sha256)
    flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    flatpak install --user --noninteractive --assumeyes --or-update \
        --arch="$architecture" flathub org.freedesktop.Platform//25.08
    flatpak install --user --noninteractive --assumeyes --or-update --bundle "$workdir/$filename"
    printf 'Installed. Add Cloud Mounter in COSMIC Settings > Desktop > Panel > Applets.\n'
)

if [[ "${BASH_SOURCE[0]:-$0}" == "$0" ]]; then
    main "$@"
fi
