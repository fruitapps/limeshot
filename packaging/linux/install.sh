#!/usr/bin/env bash
# Install LimeShot from this extracted folder.
#   sudo ./install.sh                 system-wide: /opt/limeshot, /usr/local/bin, VST3s in /usr/local/lib/vst3
#   ./install.sh --user               for you only: ~/.local/opt/limeshot, ~/.local/bin, VST3s in ~/.vst3
#   ./install.sh [--user] --private-plugins
#                                     keep the Lime VST3s private to LimeShot (in its plugins/ folder) instead of
#                                     the shared VST3 folder where other hosts find them
# Every installed path is listed in <app>/install-manifest.txt; uninstall.sh removes exactly those.
set -euo pipefail
src="$(cd "$(dirname "$0")" && pwd)"

user=0
private=0
for arg in "$@"; do
  case "$arg" in
    --user) user=1 ;;
    --private-plugins) private=1 ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

if [[ $user == 1 ]]; then
  data="${XDG_DATA_HOME:-$HOME/.local/share}"
  app="$HOME/.local/opt/limeshot"
  bin="$HOME/.local/bin"
  vst3="$HOME/.vst3"
else
  if [[ $(id -u) != 0 ]]; then
    echo "A system-wide install needs root: run 'sudo ./install.sh', or './install.sh --user'." >&2
    exit 1
  fi
  data=/usr/local/share
  app=/opt/limeshot
  bin=/usr/local/bin
  vst3=/usr/local/lib/vst3
fi

if [[ -f "$app/install-manifest.txt" ]]; then
  echo "Removing the previous installation in $app"
  "$src/uninstall.sh" --manifest "$app/install-manifest.txt" --quiet
fi

manifest="$(mktemp)"
record() { printf '%s\n' "$1" >> "$manifest"; }

mkdir -p "$app" "$bin" "$data/applications" "$data/icons/hicolor/256x256/apps" "$data/mime/packages"
for item in limeshot limeshot-cli samples licenses LICENSE LICENSE-MIT LICENSE-APACHE ATTRIBUTION.md README.txt uninstall.sh; do
  cp -R "$src/$item" "$app/"
  record "$app/$item"
done

if [[ $private == 1 ]]; then
  cp -R "$src/plugins" "$app/"
  record "$app/plugins"
else
  mkdir -p "$vst3"
  for bundle in "$src"/plugins/*.vst3; do
    dest="$vst3/$(basename "$bundle")"
    rm -rf "$dest"
    cp -R "$bundle" "$dest"
    record "$dest"
  done
fi

ln -sf "$app/limeshot" "$bin/limeshot"
ln -sf "$app/limeshot-cli" "$bin/limeshot-cli"
record "$bin/limeshot"
record "$bin/limeshot-cli"
sed "s|^Exec=.*|Exec=\"$app/limeshot\" %f|" "$src/share/limeshot.desktop" > "$data/applications/limeshot.desktop"
record "$data/applications/limeshot.desktop"
cp "$src/share/limeshot.png" "$data/icons/hicolor/256x256/apps/limeshot.png"
record "$data/icons/hicolor/256x256/apps/limeshot.png"
cp "$src/share/limeshot-mime.xml" "$data/mime/packages/limeshot.xml"
record "$data/mime/packages/limeshot.xml"

# Desktop caches are optional: missing tools only delay the menu entry and file association.
command -v update-mime-database >/dev/null && update-mime-database "$data/mime" >/dev/null 2>&1 || true
command -v update-desktop-database >/dev/null && update-desktop-database "$data/applications" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$data/icons/hicolor" >/dev/null 2>&1 || true

mv "$manifest" "$app/install-manifest.txt"
chmod 644 "$app/install-manifest.txt"
echo "LimeShot installed in $app (start it with 'limeshot' or from the application menu)."
if [[ $private == 0 ]]; then echo "Lime VST3 suite installed in $vst3."; fi
case ":$PATH:" in *":$bin:"*) ;; *) echo "Note: $bin is not on your PATH." ;; esac
