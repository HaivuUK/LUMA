#!/bin/bash

# Exit immediately if any command fails
set -e

APP_NAME="luma"
TARGET_DIR="src-tauri/target/universal-apple-darwin/release/bundle/macos"

echo "Compiling Tauri universal app bundle..."
cargo tauri build --target universal-apple-darwin

# Check to ensure the bundle actually built
if [ ! -d "$TARGET_DIR/$APP_NAME.app" ]; then
    echo "Error: App bundle not found at $TARGET_DIR/$APP_NAME.app"
    exit 1
fi

echo "Temporary payload root folder..."
# Create a staging directory. Everything inside here gets dropped into --install-location
mkdir -p ./pkg_payload
cp -r "$TARGET_DIR/$APP_NAME.app" ./pkg_payload/

echo "Generating component config using the --root layout..."
pkgbuild --analyze --root ./pkg_payload components.plist

# Disable the bundle relocation behavior so it is FORCED into /Applications
plutil -replace BundleIsRelocatable -bool false components.plist

echo "Setting up the postinstall PATH injection script..."
mkdir -p installer/scripts
cat << 'EOF' > installer/scripts/postinstall
#!/bin/bash

# Define paths safely
BINARY_SOURCE="/Applications/luma.app/Contents/MacOS/luma"
SYMLINK_TARGET="/usr/local/bin/luma"

if [ -f "$BINARY_SOURCE" ]; then
    # Ensure local binary path exists
    mkdir -p /usr/local/bin
    # Force rewrite the symlink to point to the new binary install location
    ln -sf "$BINARY_SOURCE" "$SYMLINK_TARGET"
fi
exit 0
EOF

# Make the script executable
chmod +x installer/scripts/postinstall

echo "Building the final non-relocatable .pkg installer..."
pkgbuild \
  --root ./pkg_payload \
  --install-location "/Applications" \
  --component-plist components.plist \
  --scripts "installer/scripts" \
  "${APP_NAME}_Installer.pkg"

echo "Cleaning up staging layout files..."
rm -rf ./pkg_payload
rm components.plist

echo "Fixed package built: ${APP_NAME}_Installer.pkg"