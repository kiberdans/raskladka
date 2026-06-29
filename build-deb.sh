#!/bin/bash
set -euo pipefail
NAME=raskladka
VERSION="${VERSION:-0.2.1}"
ARCH=amd64
PKG_DIR="debian-pkg/${NAME}_${VERSION}_${ARCH}"

echo "=== Building release binary ==="
cargo build --release

echo "=== Creating package structure ==="
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/48x48/apps"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$PKG_DIR/usr/lib/systemd/user"

echo "=== Copying files ==="
cp "target/release/$NAME" "$PKG_DIR/usr/bin/"
cp "$NAME.desktop" "$PKG_DIR/usr/share/applications/"
cp "$NAME.service" "$PKG_DIR/usr/lib/systemd/user/"
cp on.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/${NAME}-on.svg"
cp off.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/${NAME}-off.svg"

if command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w 48 -h 48 on.svg > "$PKG_DIR/usr/share/icons/hicolor/48x48/apps/${NAME}-on.png"
    rsvg-convert -w 48 -h 48 off.svg > "$PKG_DIR/usr/share/icons/hicolor/48x48/apps/${NAME}-off.png"
elif command -v convert &>/dev/null; then
    convert -background none -resize 48x48 on.svg "$PKG_DIR/usr/share/icons/hicolor/48x48/apps/${NAME}-on.png"
    convert -background none -resize 48x48 off.svg "$PKG_DIR/usr/share/icons/hicolor/48x48/apps/${NAME}-off.png"
fi

echo "=== Generating control file ==="
cat > "$PKG_DIR/DEBIAN/control" <<CONTROL
Package: $NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: kiberdans <kiberdans@yandex.ru>
Depends: xdotool, xclip, curl
Recommends: wl-clipboard, ydotool
Description: Keyboard layout switcher via double-press
 Переключает раскладку клавиатуры (QWERTY ↔ ЙЦУКЕН)
 по двойному нажатию настраиваемой клавиши. Работает в трее.
 .
 Поддержка X11 и Wayland.
CONTROL

echo "=== Building .deb ==="
mkdir -p debian-pkg
dpkg-deb --build "$PKG_DIR" "debian-pkg/${NAME}_${VERSION}_${ARCH}.deb"

echo "=== Done ==="
ls -lh "debian-pkg/${NAME}_${VERSION}_${ARCH}.deb"
