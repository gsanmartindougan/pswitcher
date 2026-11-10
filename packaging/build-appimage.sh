#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$(dirname "$(realpath "$0")")")"

APP_VERSION="$(awk -F '"' '/^version = / { print $2; exit }' Cargo.toml)"
test -n "$APP_VERSION"

docker build -t pswitcher-builder:bionic -f packaging/Dockerfile .
docker run --rm \
    -v pswitcher-cargo-registry:/root/.cargo/registry \
    -v "$PWD:/src" -w /src pswitcher-builder:bionic \
    bash -lc 'CARGO_TARGET_DIR=target/bionic cargo build --locked --release'

install -Dm755 target/bionic/release/pswitcher AppDir/usr/bin/pswitcher
install -Dm755 packaging/AppRun AppDir/AppRun
install -Dm644 packaging/org.pswitcher.Pswitcher.desktop AppDir/org.pswitcher.Pswitcher.desktop
install -Dm644 packaging/org.pswitcher.Pswitcher.desktop AppDir/usr/share/applications/org.pswitcher.Pswitcher.desktop
install -Dm644 packaging/pswitcher.svg AppDir/pswitcher.svg
install -Dm644 packaging/pswitcher.svg AppDir/usr/share/icons/hicolor/scalable/apps/pswitcher.svg
magick -background none packaging/pswitcher.svg -resize 256x256 AppDir/pswitcher.png
install -Dm644 AppDir/pswitcher.png AppDir/usr/share/icons/hicolor/256x256/apps/pswitcher.png
ln -sfn pswitcher.png AppDir/.DirIcon

OUTPUT="Pswitcher-${APP_VERSION}-x86_64.AppImage"
rm -f "$OUTPUT"
docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp -e ARCH=x86_64 \
    -v "$PWD:/src" -w /src pswitcher-builder:bionic \
    appimagetool --no-appstream /src/AppDir "/src/$OUTPUT"
