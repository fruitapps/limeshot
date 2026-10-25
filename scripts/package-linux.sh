#!/usr/bin/env bash
# Build LimeShot for Linux (PF-006):
#   scripts/package-linux.sh [--skip-build]
# Output (dist/):
#   linux/limeshot-<version>-linux-<arch>/          staged, runnable folder (Lime VST3s in plugins/ beside the binary)
#   limeshot-<version>-linux-<arch>.tar.gz          that folder; ./install.sh installs it (system or --user)
#   limeshot_<version>_<debarch>.deb             when dpkg-deb is available: /opt/limeshot, VST3s in /usr/lib/vst3
# The version defaults to $LIMESHOT_VERSION, then to the workspace version in Cargo.toml.
# Before packaging, the staged limeshot-cli renders the demo through the staged Lime VST3s; a silent render fails.
set -euo pipefail
cd "$(dirname "$0")/.."

skip_build=0
[[ "${1:-}" == "--skip-build" ]] && skip_build=1

version="${LIMESHOT_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
version="${version#v}"
arch="$(uname -m)"
case "$arch" in
  x86_64) debarch=amd64 ;;
  aarch64|arm64) arch=aarch64; debarch=arm64 ;;
  *) debarch="$arch" ;;
esac
target="${CARGO_TARGET_DIR:-target}"
dist=dist
name="limeshot-$version-linux-$arch"
stage="$dist/linux/$name"

if [[ $skip_build == 0 ]]; then
  cargo build --release --locked -p limeshot-app --bins
  (cd native-plugins && python3 build.py)
fi

rm -rf "$stage"
mkdir -p "$stage/plugins" "$stage/share" "$stage/licenses/fonts" "$stage/licenses/plugins"
cp "$target/release/limeshot" "$target/release/limeshot-cli" "$stage/"
cp -R assets/samples "$stage/samples"
# LimeShot's instruments and effects are the Lime VST3s (D39); the app finds them in plugins/ beside the
# binary or in ~/.vst3, /usr/lib/vst3 and /usr/local/lib/vst3.
count=0
for bundle in native-plugins/dist/*.vst3; do
  cp -R "$bundle" "$stage/plugins/"
  count=$((count + 1))
done
[[ $count == 12 ]] || { echo "expected 12 Lime VST3 bundles in native-plugins/dist, found $count" >&2; exit 1; }
cp LICENSE LICENSE-MIT LICENSE-APACHE ATTRIBUTION.md "$stage/"
cp assets/fonts/*.txt "$stage/licenses/fonts/"
cp -R native-plugins/dist/licenses/. "$stage/licenses/plugins/"
cp packaging/linux/limeshot.desktop packaging/linux/limeshot-mime.xml "$stage/share/"
cp packaging/linux/install.sh packaging/linux/uninstall.sh packaging/linux/README.txt "$stage/"
chmod +x "$stage/install.sh" "$stage/uninstall.sh" "$stage/limeshot" "$stage/limeshot-cli"
cargo run --release --locked -q -p limeshot-app --example make_icon -- "$stage/share/limeshot.png"

# Smoke test: the staged CLI renders the demo through the staged bundles.
check="$(mktemp -d)"
out="$(LIMESHOT_LIME_PLUGINS="$PWD/$stage/plugins" "$stage/limeshot-cli" demo "$check/demo.lime" --render "$check/demo.wav")"
echo "$out"
rm -rf "$check"
if grep -q 'peak -inf' <<<"$out"; then echo "silent render: the staged Lime VST3s did not load" >&2; exit 1; fi

tarball="$dist/$name.tar.gz"
tar -C "$dist/linux" -czf "$tarball" "$name"
echo "built $tarball"

if ! command -v dpkg-deb >/dev/null; then
  echo "dpkg-deb not found: skipping the .deb"
  exit 0
fi
# Debian versions sort "~" before the release: 0.1.0-alpha.1 -> 0.1.0~alpha.1.
debversion="${version//-/\~}"
deb="$dist/linux/deb"
rm -rf "$deb"
mkdir -p "$deb/DEBIAN" "$deb/opt/limeshot" "$deb/usr/bin" "$deb/usr/lib/vst3" \
  "$deb/usr/share/applications" "$deb/usr/share/icons/hicolor/256x256/apps" "$deb/usr/share/mime/packages" \
  "$deb/usr/share/doc/limeshot"
cp -R "$stage/limeshot" "$stage/limeshot-cli" "$stage/samples" "$stage/licenses" \
  "$stage/LICENSE" "$stage/LICENSE-MIT" "$stage/LICENSE-APACHE" "$stage/ATTRIBUTION.md" "$deb/opt/limeshot/"
cp -R "$stage/plugins/." "$deb/usr/lib/vst3/"
ln -s /opt/limeshot/limeshot "$deb/usr/bin/limeshot"
ln -s /opt/limeshot/limeshot-cli "$deb/usr/bin/limeshot-cli"
sed 's|^Exec=.*|Exec=/opt/limeshot/limeshot %f|' packaging/linux/limeshot.desktop > "$deb/usr/share/applications/limeshot.desktop"
cp "$stage/share/limeshot.png" "$deb/usr/share/icons/hicolor/256x256/apps/limeshot.png"
cp packaging/linux/limeshot-mime.xml "$deb/usr/share/mime/packages/limeshot.xml"
cp LICENSE "$deb/usr/share/doc/limeshot/copyright"
size="$(du -sk "$deb" | cut -f1)"
cat > "$deb/DEBIAN/control" <<CONTROL
Package: limeshot
Version: $debversion
Section: sound
Priority: optional
Architecture: $debarch
Installed-Size: $size
Depends: libc6, libasound2 | libasound2t64, libgl1, libxkbcommon0
Recommends: xdg-desktop-portal, libxkbcommon-x11-0, libwayland-client0, libxcursor1, libxrandr2, libxi6
Maintainer: LimeShot contributors <https://github.com/fruitapps/limeshot/issues>
Homepage: https://github.com/fruitapps/limeshot
Description: Pattern-based digital audio workstation (alpha)
 LimeShot is an open-source, offline-first DAW with a Channel Rack, Piano Roll,
 Playlist and Mixer. This package also installs the twelve Lime Native VST3
 plugins (LimeShot's instruments and effects) into /usr/lib/vst3, where other
 VST3 hosts can load them.
CONTROL
find "$deb" -type d -exec chmod 755 {} +
# Keep Debian's prerelease tilde inside control metadata only: GitHub rewrites tildes in asset
# names to dots, which would make the uploaded filenames disagree with SHA256SUMS.txt.
debfile="$dist/limeshot_${version}_${debarch}.deb"
dpkg-deb --root-owner-group --build "$deb" "$debfile"
rm -rf "$deb"
echo "built $debfile"
