#!/usr/bin/env bash
# Remove a LimeShot installation made by install.sh.
#   sudo /opt/limeshot/uninstall.sh            system-wide install
#   ~/.local/opt/limeshot/uninstall.sh         --user install
# Removes exactly the paths in install-manifest.txt (next to this script, or --manifest FILE). Projects,
# settings and recovery files in your home folder are kept.
set -euo pipefail
manifest="$(cd "$(dirname "$0")" && pwd)/install-manifest.txt"
quiet=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest) manifest="$2"; shift 2 ;;
    --quiet) quiet=1; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done
[[ -f "$manifest" ]] || { echo "no install manifest at $manifest" >&2; exit 1; }
app="$(dirname "$manifest")"

while IFS= read -r path; do
  # Only absolute paths written by install.sh; never the filesystem root or a home folder itself.
  case "$path" in
    /*) ;;
    *) continue ;;
  esac
  [[ "$path" == "/" || "$path" == "$HOME" ]] && continue
  rm -rf -- "$path"
done < "$manifest"
rm -f -- "$manifest"
rmdir -- "$app" 2>/dev/null || true
[[ $quiet == 1 ]] || echo "LimeShot removed from $app."
