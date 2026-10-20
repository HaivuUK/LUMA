#!/usr/bin/env bash
# Usage: build-macos.sh <version> <universal-binary> <out-dir>
# Env:   APP_NAME APP_DISPLAY APP_ID  (required)
#        SIGN_APP_ID SIGN_PKG_ID NOTARY_APPLE_ID NOTARY_TEAM_ID NOTARY_PASSWORD (optional)
set -euo pipefail

VERSION="$1"; BIN="$2"; OUT="$3"
: "${APP_NAME:?}" "${APP_DISPLAY:?}" "${APP_ID:?}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="$ROOT/target/macos"
rm -rf "$WORK"; mkdir -p "$WORK" "$OUT"
OUT="$(cd "$OUT" && pwd)"
APP="$WORK/$APP_DISPLAY.app"

notarize() {   # notarize <file> [staple]
  [ -n "${NOTARY_APPLE_ID:-}" ] || return 0
  xcrun notarytool submit "$1" --apple-id "$NOTARY_APPLE_ID" --team-id "$NOTARY_TEAM_ID" \
    --password "$NOTARY_PASSWORD" --wait
  [ "${2:-}" = "staple" ] && xcrun stapler staple "${3:-$1}"
  return 0
}

# ---------------------------------------------------------------- .app bundle
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$APP_NAME"
chmod +x "$APP/Contents/MacOS/$APP_NAME"
cp "$ROOT/icons/icon.icns" "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$APP_DISPLAY</string>
  <key>CFBundleDisplayName</key><string>$APP_DISPLAY</string>
  <key>CFBundleIdentifier</key><string>$APP_ID</string>
  <key>CFBundleExecutable</key><string>$APP_NAME</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST

# lipo invalidates the linker's ad-hoc signature, so always (re)sign.
if [ -n "${SIGN_APP_ID:-}" ]; then
  codesign --force --options runtime --timestamp --sign "$SIGN_APP_ID" "$APP"
else
  codesign --force --sign - "$APP"
fi

if [ -n "${NOTARY_APPLE_ID:-}" ] && [ -n "${SIGN_APP_ID:-}" ]; then
  ditto -c -k --keepParent "$APP" "$WORK/notarize.zip"
  notarize "$WORK/notarize.zip" staple "$APP"
fi

ditto -c -k --keepParent "$APP" "$OUT/$APP_NAME-$VERSION-macos-universal.app.zip"

# ----------------------------------------------------------------------- .dmg
STAGE="$WORK/dmg"; mkdir -p "$STAGE"; cp -R "$APP" "$STAGE/"
DMG="$OUT/$APP_NAME-$VERSION-macos-universal.dmg"
DMG_ARGS=(--volname "$APP_DISPLAY" --volicon "$ROOT/icons/icon.icns"
  --eula "$ROOT/LICENSE.txt"
  --window-pos 200 120 --window-size 600 400 --icon-size 110
  --icon "$APP_DISPLAY.app" 150 190 --hide-extension "$APP_DISPLAY.app"
  --app-drop-link 450 190)
create-dmg "${DMG_ARGS[@]}" "$DMG" "$STAGE" \
  || { rm -f "$DMG"; create-dmg --skip-jenkins "${DMG_ARGS[@]}" "$DMG" "$STAGE"; }
if [ -n "${SIGN_APP_ID:-}" ]; then
  codesign --force --timestamp --sign "$SIGN_APP_ID" "$DMG"
  notarize "$DMG" staple
fi

# ----------------------------------------------------------------------- .pkg
PKGROOT="$WORK/pkgroot"; mkdir -p "$PKGROOT/Applications"
cp -R "$APP" "$PKGROOT/Applications/"

SCRIPTS="$WORK/scripts"; mkdir -p "$SCRIPTS"
sed -e "s|@APP_NAME@|$APP_NAME|g" -e "s|@APP_DISPLAY@|$APP_DISPLAY|g" \
  "$ROOT/packaging/macos/postinstall" > "$SCRIPTS/postinstall"
chmod +x "$SCRIPTS/postinstall"

# Don't let macOS "relocate" the app to some other copy it finds on disk.
pkgbuild --analyze --root "$PKGROOT" "$WORK/components.plist" >/dev/null
/usr/libexec/PlistBuddy -c "Set :0:BundleIsRelocatable false" "$WORK/components.plist"
pkgbuild --root "$PKGROOT" --component-plist "$WORK/components.plist" \
  --identifier "$APP_ID" --version "$VERSION" --install-location / \
  --scripts "$SCRIPTS" "$WORK/component.pkg"

RES="$WORK/resources"; mkdir -p "$RES"; cp "$ROOT/LICENSE.txt" "$RES/LICENSE.txt"
sed -e "s|@APP_DISPLAY@|$APP_DISPLAY|g" -e "s|@APP_ID@|$APP_ID|g" -e "s|@VERSION@|$VERSION|g" \
  "$ROOT/packaging/macos/distribution.xml" > "$WORK/distribution.xml"

PB_SIGN=()
if [ -n "${SIGN_PKG_ID:-}" ]; then PB_SIGN=(--sign "$SIGN_PKG_ID"); fi
PKG="$OUT/$APP_NAME-$VERSION-macos-universal.pkg"
productbuild --distribution "$WORK/distribution.xml" --resources "$RES" \
  --package-path "$WORK" ${PB_SIGN[@]+"${PB_SIGN[@]}"} "$PKG"
if [ -n "${SIGN_PKG_ID:-}" ]; then notarize "$PKG" staple; fi

ls -l "$OUT"
