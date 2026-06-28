#!/bin/bash
set -euo pipefail
NAME=raskladka
VERSION="${VERSION:-0.2.0}"

echo "=== Building release binary ==="
cargo build --release

echo "=== Creating AppDir ==="
APPDIR="AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$APPDIR/usr/share/applications"

cp "target/release/$NAME" "$APPDIR/usr/bin/"

cp on.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/${NAME}-on.svg"
cp off.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/${NAME}-off.svg"
cp on.svg "$APPDIR/${NAME}.svg"

cat > "$APPDIR/usr/share/applications/${NAME}.desktop" <<DESKTOP
[Desktop Entry]
Version=1.0
Type=Application
Name=raskladka
Comment=Keyboard layout switcher (QWERTY ↔ ЙЦУКЕН)
Exec=$NAME
Icon=$NAME
Categories=Utility;
Terminal=false
DESKTOP

cat > "$APPDIR/AppRun" <<APPRUN
#!/bin/bash
HERE="\$(dirname "\$(readlink -f "\$0")")"
export PATH="\$HERE/usr/bin:\$PATH"
exec "\$HERE/usr/bin/$NAME" "\$@"
APPRUN
chmod +x "$APPDIR/AppRun"

echo "=== Downloading appimagetool ==="
if ! command -v appimagetool &>/dev/null; then
    ARCH=x86_64
    TOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    curl -L -o appimagetool "$TOOL_URL"
    chmod +x appimagetool
    APPIMAGETOOL="./appimagetool"
else
    APPIMAGETOOL="appimagetool"
fi

echo "=== Building AppImage ==="
mkdir -p appimage-pkg
$APPIMAGETOOL "$APPDIR" "appimage-pkg/${NAME}-${VERSION}-${ARCH}.AppImage"

echo "=== Done ==="
ls -lh "appimage-pkg/${NAME}-${VERSION}-${ARCH}.AppImage"
