#!/bin/bash
set -euo pipefail
NAME=raskladka
VERSION="${VERSION:-0.2.1}"
ARCH=x86_64
RPM_DIR="rpm-pkg"

echo "=== Building release binary ==="
cargo build --release

echo "=== Creating RPM structure ==="
rm -rf "$RPM_DIR"
mkdir -p "$RPM_DIR/BUILD"
mkdir -p "$RPM_DIR/RPMS"
mkdir -p "$RPM_DIR/SOURCES"
mkdir -p "$RPM_DIR/SPECS"
mkdir -p "$RPM_DIR/SRPMS"

INSTALL_ROOT="$RPM_DIR/BUILD/${NAME}-${VERSION}"
mkdir -p "$INSTALL_ROOT/usr/bin"
mkdir -p "$INSTALL_ROOT/usr/share/applications"
mkdir -p "$INSTALL_ROOT/usr/share/icons/hicolor/48x48/apps"
mkdir -p "$INSTALL_ROOT/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$INSTALL_ROOT/usr/lib/systemd/user"

cp "target/release/$NAME" "$INSTALL_ROOT/usr/bin/"
cp "$NAME.desktop" "$INSTALL_ROOT/usr/share/applications/"
cp "$NAME.service" "$INSTALL_ROOT/usr/lib/systemd/user/"
cp on.svg "$INSTALL_ROOT/usr/share/icons/hicolor/scalable/apps/${NAME}-on.svg"
cp off.svg "$INSTALL_ROOT/usr/share/icons/hicolor/scalable/apps/${NAME}-off.svg"

if command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w 48 -h 48 on.svg > "$INSTALL_ROOT/usr/share/icons/hicolor/48x48/apps/${NAME}-on.png"
    rsvg-convert -w 48 -h 48 off.svg > "$INSTALL_ROOT/usr/share/icons/hicolor/48x48/apps/${NAME}-off.png"
fi

cat > "$RPM_DIR/SPECS/${NAME}.spec" <<SPEC
Name:           $NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        Keyboard layout switcher (QWERTY ↔ ЙЦУКЕН)

License:        MIT
URL:            https://github.com/kiberdans/raskladka

Requires:       xdotool, xclip, curl
Recommends:     wl-clipboard, ydotool

%description
Переключает раскладку клавиатуры (QWERTY ↔ ЙЦУКЕН)
по двойному нажатию настраиваемой клавиши. Работает в трее.
Поддержка X11 и Wayland.

%files
/usr/bin/$NAME
/usr/share/applications/$NAME.desktop
/usr/lib/systemd/user/$NAME.service
/usr/share/icons/hicolor/scalable/apps/${NAME}-on.svg
/usr/share/icons/hicolor/scalable/apps/${NAME}-off.svg
/usr/share/icons/hicolor/48x48/apps/${NAME}-on.png
/usr/share/icons/hicolor/48x48/apps/${NAME}-off.png

%changelog
* $(date +"%a %b %d %Y") kiberdans <kiberdans@yandex.ru> - $VERSION-1
- Release $VERSION
SPEC

echo "=== Building .rpm ==="
rpmbuild -bb \
    --define "_topdir $(pwd)/$RPM_DIR" \
    --buildroot "$(pwd)/$INSTALL_ROOT" \
    "$RPM_DIR/SPECS/${NAME}.spec"

mkdir -p rpm-pkg
cp "$RPM_DIR/RPMS/$ARCH/${NAME}-${VERSION}-1"*.rpm "rpm-pkg/${NAME}-${VERSION}-1.${ARCH}.rpm" 2>/dev/null || true

echo "=== Done ==="
ls -lh rpm-pkg/*.rpm 2>/dev/null || echo "rpm not found, check rpmbuild output"
