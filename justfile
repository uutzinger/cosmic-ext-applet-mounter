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

build *args:
    cargo build {{args}}

build-release *args:
    cargo build --release {{args}}

run *args:
    env RUST_BACKTRACE=1 cargo run {{args}}

verify: fmt-check check lint test

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
