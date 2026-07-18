#!/bin/bash

# Exit immediately if any command fails
set -e

APP_NAME="luma"

# Read target from the workflow argument, fallback to arm64 (Apple Silicon)
TARGET=${1:-${TARGET:-aarch64-apple-darwin}}

# Extract version from tauri.conf.json using jq
VERSION=$(jq -r '.version' tauri.conf.json)

# Determine the architecture suffix to match your other files
if [[ "$TARGET" == *"aarch64"* ]]; then
    ARCH="aarch64"
else
    ARCH="x64"
fi

echo "Compiling Tauri app bundle for $TARGET..."
cargo tauri build --target "$TARGET"

# Check for target directory
if [ -d "target/$TARGET/release/bundle/macos" ]; then
    TARGET_DIR="target/$TARGET/release/bundle/macos"
elif [ -d "src-tauri/target/$TARGET/release/bundle/macos" ]; then
    TARGET_DIR="src-tauri/target/$TARGET/release/bundle/macos"
else
    echo "Error: Release bundle directory not found for target $TARGET."
    exit 1
fi

# Check to ensure the bundle actually built
if [ ! -d "$TARGET_DIR/$APP_NAME.app" ]; then
    echo "Error: App bundle not found at $TARGET_DIR/$APP_NAME.app"
    exit 1
fi

echo "Archiving .app bundle into compressed tarball..."
tar -czf "${APP_NAME}_${VERSION}_${ARCH}.app.tar.gz" -C "$TARGET_DIR" "$APP_NAME.app"

echo "Temporary payload root folder..."
mkdir -p ./pkg_payload
cp -r "$TARGET_DIR/$APP_NAME.app" ./pkg_payload/

echo "Generating component config..."
pkgbuild --analyze --root ./pkg_payload components.plist

# Disable bundle relocation behavior so it forces install into /Applications
plutil -replace BundleIsRelocatable -bool false components.plist

echo "Setting up postinstall script..."
mkdir -p installer/scripts
cat << 'EOF' > installer/scripts/postinstall
#!/bin/bash
BINARY_SOURCE="/Applications/luma.app/Contents/MacOS/luma"
SYMLINK_TARGET="/usr/local/bin/luma"

if [ -f "$BINARY_SOURCE" ]; then
    mkdir -p /usr/local/bin
    ln -sf "$BINARY_SOURCE" "$SYMLINK_TARGET"
fi
exit 0
EOF

chmod +x installer/scripts/postinstall

PKG_NAME="${APP_NAME}_${VERSION}_${ARCH}_Installer.pkg"

echo "Building non-relocatable installer: $PKG_NAME"
pkgbuild \
  --root ./pkg_payload \
  --install-location "/Applications" \
  --component-plist components.plist \
  --scripts "installer/scripts" \
  "$PKG_NAME"

echo "Cleaning up..."
rm -rf ./pkg_payload
rm components.plist

echo "Fixed package built: ${APP_NAME}."