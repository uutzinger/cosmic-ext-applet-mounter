name := 'cosmic-ext-applet-mounter'
appid := 'io.github.uutzinger.cosmic-ext-applet-mounter'
rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')
appdata-dst := base-dir / 'share' / 'metainfo' / appid + '.metainfo.xml'
bin-dst := base-dir / 'bin' / name
desktop-dst := base-dir / 'share' / 'applications' / appid + '.desktop'
icon-dst := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / appid + '.svg'
user-data-dir := env('XDG_DATA_HOME', env('HOME') / '.local' / 'share')
user-bin-dir := env('HOME') / '.local' / 'bin'

default: build

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --all-targets

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets

metadata-check:
    # Strict freedesktop validators currently reject official COSMIC applet
    # template fields such as Categories=COSMIC and AppStream provides/binaries.
    # Keep this recipe non-fatal so `just verify` remains useful; run
    # `metadata-check-strict` when checking freedesktop-only metadata.
    -desktop-file-validate resources/app.desktop
    -appstreamcli validate --pedantic --no-net resources/app.metainfo.xml

metadata-check-net:
    -desktop-file-validate resources/app.desktop
    -appstreamcli validate --pedantic resources/app.metainfo.xml

metadata-check-strict:
    desktop-file-validate resources/app.desktop
    appstreamcli validate --pedantic --no-net resources/app.metainfo.xml

flatpak-cargo-sources:
    #!/usr/bin/env bash
    set -euo pipefail
    generator_python="python3"
    if [ -x .venv-flatpak-generator/bin/python ]; then
        generator_python=".venv-flatpak-generator/bin/python"
    fi
    if command -v flatpak-cargo-generator >/dev/null 2>&1; then
        flatpak-cargo-generator Cargo.lock -o packaging/flatpak/cargo-sources.json
    elif [ -f ../cosmic-ext-applet-sysinfo/flatpak/flatpak-cargo-generator.py ]; then
        "$generator_python" ../cosmic-ext-applet-sysinfo/flatpak/flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json
    elif [ -f ../cosmic-ext-applet-weather/flatpak/flatpak-cargo-generator.py ]; then
        "$generator_python" ../cosmic-ext-applet-weather/flatpak/flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json
    else
        printf '%s\n' 'flatpak-cargo-generator is required to generate packaging/flatpak/cargo-sources.json.' >&2
        printf '%s\n' 'Install/provide flatpak-cargo-generator, or place the COSMIC helper script at ../cosmic-ext-applet-sysinfo/flatpak/flatpak-cargo-generator.py.' >&2
        exit 127
    fi

build *args:
    cargo build {{args}}

build-release *args:
    cargo build --release {{args}}

run *args:
    env RUST_BACKTRACE=1 cargo run {{args}}

verify: fmt-check check lint test metadata-check

deb:
    dpkg-buildpackage -us -uc -b

stage destination='target/stage':
    just rootdir={{absolute_path(destination)}} prefix=/usr install

install-user: build-release
    install -Dm0755 {{cargo-target-dir / 'release' / name}} {{user-bin-dir / name}}
    install -Dm0755 scripts/cosmic-ext-applet-mounter-onedrive-auth-helper {{user-bin-dir / 'cosmic-ext-applet-mounter-onedrive-auth-helper'}}
    install -Dm0644 resources/app.desktop {{user-data-dir / 'applications' / appid + '.desktop'}}
    install -Dm0644 resources/app.metainfo.xml {{user-data-dir / 'metainfo' / appid + '.metainfo.xml'}}
    install -Dm0644 resources/icon.svg {{user-data-dir / 'icons' / 'hicolor' / 'scalable' / 'apps' / appid + '.svg'}}
    -update-desktop-database {{user-data-dir / 'applications'}}
    -gtk-update-icon-cache -f -t {{user-data-dir / 'icons' / 'hicolor'}}

install: build-release
    install -Dm0755 {{cargo-target-dir / 'release' / name}} {{bin-dst}}
    install -Dm0755 scripts/cosmic-ext-applet-mounter-onedrive-auth-helper {{base-dir / 'bin' / 'cosmic-ext-applet-mounter-onedrive-auth-helper'}}
    install -Dm0644 resources/app.desktop {{desktop-dst}}
    install -Dm0644 resources/app.metainfo.xml {{appdata-dst}}
    install -Dm0644 resources/icon.svg {{icon-dst}}

uninstall:
    rm -f {{bin-dst}} {{base-dir / 'bin' / 'cosmic-ext-applet-mounter-onedrive-auth-helper'}} {{desktop-dst}} {{appdata-dst}} {{icon-dst}}

clean:
    cargo clean
