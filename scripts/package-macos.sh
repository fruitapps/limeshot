#!/usr/bin/env bash
# Build LimeShot.app and a DMG for macOS (PF-006).
#   scripts/package-macos.sh            → dist/LimeShot.app, dist/LimeShot-<version>-macos-<arch>.dmg
# The app is signed ad hoc (codesign -s -): it runs on this Mac; other Macs need a Developer ID
# signature and notarisation, which this script does not do.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
arch=$(uname -m)
dist=dist
app="$dist/LimeShot.app"

cargo build --release -p limeshot-app --bins
rm -rf "$app" "$dist/LimeShot.iconset" "$dist/dmg"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/limeshot target/release/limeshot-cli "$app/Contents/MacOS/"
# Bundled samples: the app looks for them in ../Resources/samples next to its binary.
cp -R assets/samples "$app/Contents/Resources/samples"
cp LICENSE* ATTRIBUTION.md "$app/Contents/Resources/" 2>/dev/null || true

cargo run --release -q -p limeshot-app --example make_icon -- "$dist/LimeShot.iconset"
iconutil -c icns "$dist/LimeShot.iconset" -o "$app/Contents/Resources/LimeShot.icns"
rm -rf "$dist/LimeShot.iconset"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>LimeShot</string>
  <key>CFBundleDisplayName</key><string>LimeShot</string>
  <key>CFBundleIdentifier</key><string>org.limeshot.LimeShot</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleExecutable</key><string>limeshot</string>
  <key>CFBundleIconFile</key><string>LimeShot</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key><string>LimeShot project</string>
      <key>CFBundleTypeExtensions</key><array><string>lime</string></array>
      <key>CFBundleTypeRole</key><string>Editor</string>
    </dict>
  </array>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

codesign --force --deep --sign - "$app"

# The DMG holds the app and a link to /Applications.
mkdir -p "$dist/dmg"
cp -R "$app" "$dist/dmg/"
ln -s /Applications "$dist/dmg/Applications"
dmg="$dist/LimeShot-$version-macos-$arch.dmg"
rm -f "$dmg"
hdiutil create -quiet -volname LimeShot -srcfolder "$dist/dmg" -ov -format UDZO "$dmg"
rm -rf "$dist/dmg"
echo "built $app and $dmg"
