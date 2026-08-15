#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-x86_64-unknown-linux-musl}"

case "$TARGET" in
  x86_64*)
    ARCH="x86_64"
    ;;
  aarch64*)
    ARCH="aarch64"
    ;;
  *)
    echo "Unsupported architecture for AppImage: $TARGET"
    exit 1
    ;;
esac

APPDIR="AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/512x512/apps"

# Copy binaries
if [ -f "target/$TARGET/release/che" ]; then
  cp "target/$TARGET/release/che" "$APPDIR/usr/bin/che"
  cp "target/$TARGET/release/ch" "$APPDIR/usr/bin/ch"
elif [ -f "target/release/che" ]; then
  cp "target/release/che" "$APPDIR/usr/bin/che"
  cp "target/release/ch" "$APPDIR/usr/bin/ch"
else
  echo "Binaries not found for target $TARGET"
  exit 1
fi
chmod +x "$APPDIR/usr/bin/che" "$APPDIR/usr/bin/ch"

# Copy desktop and icon assets
cp assets/che.desktop "$APPDIR/che.desktop"
cp assets/che.desktop "$APPDIR/usr/share/applications/che.desktop"
cp assets/logo.png "$APPDIR/che.png"
cp assets/logo.png "$APPDIR/usr/share/icons/hicolor/512x512/apps/che.png"

# Create smart multi-call AppRun entrypoint
cat << 'APPRUN' > "$APPDIR/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"

# Check invoked binary name (support symlink: ln -s che.AppImage ch)
CALL_NAME="$(basename "${ARGV0:-$0}")"
if [ "$CALL_NAME" = "ch" ]; then
  exec "${HERE}/usr/bin/ch" "$@"
fi

# Support invoking ch as first subcommand: ./che.AppImage ch ...
if [ "$#" -gt 0 ] && [ "$1" = "ch" ]; then
  shift
  exec "${HERE}/usr/bin/ch" "$@"
fi

exec "${HERE}/usr/bin/che" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

# Download and run appimagetool
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
curl -fsSL "$APPIMAGETOOL_URL" -o appimagetool
chmod +x appimagetool

./appimagetool --appimage-extract
ARCH="$ARCH" ./squashfs-root/AppRun "$APPDIR" "che-${ARCH}.AppImage"

rm -rf "$APPDIR" squashfs-root appimagetool
echo "Successfully created che-${ARCH}.AppImage"
