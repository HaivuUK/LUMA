#!/bin/bash

# Exit immediately if any command fails
set -e

APP_NAME="luma"

# Read target from argument, default to Apple Silicon (aarch64-apple-darwin)
TARGET=${1:-aarch64-apple-darwin}

# Extract version from Cargo.toml
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

if [ -z "$VERSION" ]; then
    echo "Error: Could not extract version from Cargo.toml"
    exit 1
fi

# Determine architecture identifier
if [[ "$TARGET" == *"aarch64"* ]]; then
    ARCH="arm64"
else
    ARCH="x86_64"
fi

echo "Compiling native egui binary for $TARGET (v$VERSION)..."
cargo build --release --target "$TARGET"

BINARY_PATH="target/$TARGET/release/$APP_NAME"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Release binary not found at $BINARY_PATH"
    exit 1
fi

echo "Creating macOS .app bundle structure..."
APP_BUNDLE="${APP_NAME}.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary into bundle
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

if [ -d "assets" ]; then
    cp -r "assets" "$APP_BUNDLE/Contents/Resources/assets"
fi

# Copy icon file if present
if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "$APP_BUNDLE/Contents/Resources/icon.icns"
fi

# Generate Info.plist required for macOS app bundle
cat << EOF > "$APP_BUNDLE/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.luma.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Archiving .app bundle into compressed tarball..."
tar -czf "${APP_NAME}_${VERSION}_${ARCH}.app.tar.gz" "$APP_BUNDLE"

echo "Setting up temporary payload root for PKG..."
rm -rf ./pkg_payload
mkdir -p ./pkg_payload
cp -r "$APP_BUNDLE" ./pkg_payload/

echo "Generating component configuration..."
pkgbuild --analyze --root ./pkg_payload components.plist

# Force install strictly into /Applications (disable relocatable behavior)
plutil -replace BundleIsRelocatable -bool false components.plist

echo "Setting up postinstall script for terminal PATH symlink..."
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

echo "Building installer package: $PKG_NAME"
pkgbuild \
  --root ./pkg_payload \
  --install-location "/Applications" \
  --component-plist components.plist \
  --scripts "installer/scripts" \
  "$PKG_NAME"

echo "Cleaning up temporary build files..."
rm -rf ./pkg_payload components.plist installer/ "$APP_BUNDLE"

echo "Package build complete: $PKG_NAME"