#!/usr/bin/env bash
# Build LimeShot.app, a DMG and a PKG installer for macOS (PF-006).
#   scripts/package-macos.sh [--skip-build]
# Output (dist/):
#   LimeShot.app                                  self-contained: the Lime VST3s are in Contents/PlugIns
#   LimeShot-<version>-macos-<arch>.dmg           the app and a link to /Applications (drag to install)
#   LimeShot-<version>-macos-<arch>.pkg           installer: LimeShot.app in /Applications and the Lime VST3 suite
#                                                 in /Library/Audio/Plug-Ins/VST3, where other hosts find it too
# The version defaults to $LIMESHOT_VERSION, then to the workspace version in Cargo.toml.
# Everything is signed ad hoc (codesign -s -): other Macs need a Developer ID signature and notarisation,
# which this script does not do (Gatekeeper asks the user to confirm the first start).
set -euo pipefail
cd "$(dirname "$0")/.."

skip_build=0
[[ "${1:-}" == "--skip-build" ]] && skip_build=1

version="${LIMESHOT_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
version="${version#v}"
# Bundle versions are numeric: 0.1.0-alpha.1 -> 0.1.0.
numeric="${version%%[-+]*}"
arch=$(uname -m)
target="${CARGO_TARGET_DIR:-target}"
dist=dist
app="$dist/LimeShot.app"

if [[ $skip_build == 0 ]]; then
  cargo build --release --locked -p limeshot-app --bins
  # LimeShot's instruments and effects are the Lime VST3s (D39).
  (cd native-plugins && python3 build.py)
fi
rm -rf "$app" "$dist/LimeShot.iconset" "$dist/dmg" "$dist/pkg"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target/release/limeshot" "$target/release/limeshot-cli" "$app/Contents/MacOS/"
# Bundled samples: the app looks for them in ../Resources/samples next to its binary.
cp -R assets/samples "$app/Contents/Resources/samples"
# The app finds the Lime VST3s in ../PlugIns, then in the standard VST3 folders.
mkdir -p "$app/Contents/PlugIns"
count=0
for bundle in native-plugins/dist/*.vst3; do
  cp -R "$bundle" "$app/Contents/PlugIns/"
  count=$((count + 1))
done
[[ $count == 12 ]] || { echo "expected 12 Lime VST3 bundles in native-plugins/dist, found $count" >&2; exit 1; }
cp LICENSE LICENSE-MIT LICENSE-APACHE ATTRIBUTION.md "$app/Contents/Resources/"
# Fonts are embedded in the executable; their licence texts must travel with the app, like the plugins'.
mkdir -p "$app/Contents/Resources/licenses/fonts" "$app/Contents/Resources/licenses/plugins"
cp assets/fonts/*.txt "$app/Contents/Resources/licenses/fonts/"
cp -R native-plugins/dist/licenses/. "$app/Contents/Resources/licenses/plugins/"

cargo run --release --locked -q -p limeshot-app --example make_icon -- "$dist/LimeShot.iconset"
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
  <key>CFBundleVersion</key><string>$numeric</string>
  <key>CFBundleShortVersionString</key><string>$numeric</string>
  <key>CFBundleGetInfoString</key><string>LimeShot $version</string>
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
  <key>NSMicrophoneUsageDescription</key><string>LimeShot records audio from your input when you arm Record with the Audio filter.</string>
</dict>
</plist>
PLIST

# Sign the plugins first, then the app around them (--deep alone is deprecated for nested bundles).
for bundle in "$app/Contents/PlugIns/"*.vst3; do codesign --force --sign - "$bundle"; done
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"

# Smoke test: the bundled CLI renders the demo through the bundled plugins.
check="$(mktemp -d)"
out="$(LIMESHOT_LIME_PLUGINS="$PWD/$app/Contents/PlugIns" "$app/Contents/MacOS/limeshot-cli" demo "$check/demo.lime" --render "$check/demo.wav")"
echo "$out"
rm -rf "$check"
if grep -q 'peak -inf' <<<"$out"; then echo "silent render: the bundled Lime VST3s did not load" >&2; exit 1; fi

# The DMG holds the self-contained app and a link to /Applications.
mkdir -p "$dist/dmg"
cp -R "$app" "$dist/dmg/"
ln -s /Applications "$dist/dmg/Applications"
dmg="$dist/LimeShot-$version-macos-$arch.dmg"
rm -f "$dmg"
hdiutil create -quiet -volname LimeShot -srcfolder "$dist/dmg" -ov -format UDZO "$dmg"
rm -rf "$dist/dmg"
echo "built $dmg"

# The PKG installs the app without its private PlugIns copy and the plugins once, in the shared VST3 folder
# (LimeShot searches it). Both components are required: the app has no instruments or effects without them.
pkg="$dist/pkg"
mkdir -p "$pkg/app-root" "$pkg/vst3-root"
cp -R "$app" "$pkg/app-root/"
rm -rf "$pkg/app-root/LimeShot.app/Contents/PlugIns"
codesign --force --deep --sign - "$pkg/app-root/LimeShot.app"
cp -R "$app/Contents/PlugIns/." "$pkg/vst3-root/"
# Install exactly where stated: do not follow an app the user moved elsewhere.
pkgbuild --analyze --root "$pkg/app-root" "$pkg/app.plist"
plutil -replace 0.BundleIsRelocatable -bool NO "$pkg/app.plist"
pkgbuild --quiet --root "$pkg/app-root" --component-plist "$pkg/app.plist" --identifier org.limeshot.LimeShot.app \
  --version "$numeric" --install-location /Applications "$pkg/LimeShot-app.pkg"
pkgbuild --quiet --root "$pkg/vst3-root" --identifier org.limeshot.LimeShot.vst3 \
  --version "$numeric" --install-location /Library/Audio/Plug-Ins/VST3 "$pkg/LimeShot-vst3.pkg"
mkdir -p "$pkg/resources"
cp LICENSE "$pkg/resources/LICENSE.txt"
cat > "$pkg/resources/welcome.txt" <<WELCOME
LimeShot $version (alpha)

This installs LimeShot.app into /Applications and the twelve Lime Native VST3 plugins (LimeShot's
instruments and effects) into /Library/Audio/Plug-Ins/VST3, where other VST3 hosts can load them too.
WELCOME
cat > "$pkg/distribution.xml" <<DIST
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
  <title>LimeShot $version</title>
  <welcome file="welcome.txt"/>
  <license file="LICENSE.txt"/>
  <options customize="never" require-scripts="false" hostArchitectures="$arch"/>
  <domains enable_localSystem="true"/>
  <os-version min="11.0"/>
  <choices-outline>
    <line choice="app"/>
    <line choice="vst3"/>
  </choices-outline>
  <choice id="app" title="LimeShot" enabled="false"><pkg-ref id="org.limeshot.LimeShot.app"/></choice>
  <choice id="vst3" title="Lime Native VST3 suite" enabled="false"><pkg-ref id="org.limeshot.LimeShot.vst3"/></choice>
  <pkg-ref id="org.limeshot.LimeShot.app" version="$numeric">LimeShot-app.pkg</pkg-ref>
  <pkg-ref id="org.limeshot.LimeShot.vst3" version="$numeric">LimeShot-vst3.pkg</pkg-ref>
</installer-gui-script>
DIST
installer_pkg="$dist/LimeShot-$version-macos-$arch.pkg"
rm -f "$installer_pkg"
productbuild --quiet --distribution "$pkg/distribution.xml" --resources "$pkg/resources" --package-path "$pkg" \
  "$installer_pkg"
rm -rf "$pkg"
echo "built $app, $dmg and $installer_pkg"
