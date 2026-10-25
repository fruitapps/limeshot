# Release packaging (PF-006)

How the alpha packages are built, what they install and how they were checked. Decision: D40 in
[decisions.md](decisions.md).

## Packages

| Platform | Command | Output in `dist/` | Installs |
| --- | --- | --- | --- |
| Windows x64 | `powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1` | `LimeShot-<v>-windows-x64-setup.exe`, `LimeShot-<v>-windows-x64-portable.zip` | App in `Program Files\LimeShot` (or `%LOCALAPPDATA%\Programs\LimeShot` for "me only"); VST3s in `Common Files\VST3` (or `%LOCALAPPDATA%\Programs\Common\VST3`), or privately in `<app>\plugins` when the VST3 component is unticked |
| macOS (per arch) | `scripts/package-macos.sh` | `LimeShot.app`, `LimeShot-<v>-macos-<arch>.dmg`, `LimeShot-<v>-macos-<arch>.pkg` | PKG: `/Applications/LimeShot.app` and `/Library/Audio/Plug-Ins/VST3`. DMG: a self-contained app with `Contents/PlugIns` |
| Linux (per arch) | `scripts/package-linux.sh` | `limeshot-<v>-linux-<arch>.tar.gz`, `limeshot_<v>_<debarch>.deb` | `.deb`: `/opt/limeshot`, `/usr/bin` links, VST3s in `/usr/lib/vst3`, desktop entry, icon and `.lime` MIME type. Tarball: runs in place; `install.sh` installs to `/opt/limeshot` + `/usr/local/lib/vst3`, or with `--user` to `~/.local/opt/limeshot` + `~/.vst3` |

Every package contains `limeshot`, `limeshot-cli`, the bundled samples, the twelve Lime Native VST3 bundles, the
icon (drawn by `make_icon` from the same code as the window icon) and the licence texts (LimeShot, fonts and the
plugins' dependencies). LimeShot finds the Lime VST3s through `limeshot_plugins::lime::search_dirs`:
`LIMESHOT_LIME_PLUGINS`, `../PlugIns` and `plugins/` beside the executable, then the standard VST3 folders.

The version comes from `LIMESHOT_VERSION` (a leading `v` is dropped), else from the workspace `Cargo.toml`.
Numeric-only fields use the part before `-`: Windows file versions and macOS bundle versions become `0.1.0` for
`0.1.0-alpha.1`, and the `.deb` version becomes `0.1.0~alpha.1` so the final release sorts after it. The Debian filename keeps the original `0.1.0-alpha.1` version;
using a tilde in the filename lets GitHub rename the asset and breaks checksum-manifest lookup.

Each script builds the release binaries and the release VST3 bundles (`native-plugins/build.py`) and then renders
the demo song with the staged `limeshot-cli` through the staged bundles; a silent render stops the build.

## Release workflow

[`.github/workflows/release.yml`](../../.github/workflows/release.yml) runs the three scripts on Windows x86-64,
macOS Apple Silicon and Intel, and Ubuntu 22.04 x86-64 and ARM64 (the oldest runners, for a low glibc
requirement). On Linux it also installs the `.deb` and the tarball (`--user`), renders the demo with the installed
CLI while the checkout's `native-plugins/dist` is hidden, and uninstalls both. Pushing a tag such as
`v0.1.0-alpha.1` attaches every package and `SHA256SUMS.txt` to a **draft** prerelease; publishing is manual.
`workflow_dispatch` builds the packages as workflow artifacts.

To cut a release: set the workspace version if it changes, add its section to [CHANGELOG.md](../../CHANGELOG.md),
make sure CI is green on `main`, then push a tag such as `git tag v0.1.0-alpha.2 && git push origin v0.1.0-alpha.2`.
Review the draft release (its assets and `SHA256SUMS.txt`) and publish it as a prerelease.

## Signing

Nothing is signed with a real identity yet. Windows SmartScreen warns about the unsigned installer and
executables ("More info" → "Run anyway"). macOS packages are signed ad hoc, so Gatekeeper blocks a downloaded DMG
app or PKG until the user allows it (right-click → Open, or System Settings → Privacy & Security). Adding an
Authenticode certificate (`signtool` before ISCC and `SignTool=` in the `.iss`) and a Developer ID with
notarisation (`codesign --options runtime`, `productsign`, `notarytool`) are the next packaging steps.

## Verification record

* Windows 11 x64 (2026-10-25): installer in per-user and private-plugin modes, the installed CLI renders the demo
  with the checkout's bundles hidden, and the uninstaller removes everything (PF-006 record in
  [features.json](../roadmap/features.json)).
* v0.1.0-alpha.1 was built by release run
  [40219554203](https://github.com/fruitapps/limeshot/actions/runs/40219554203) at `28d3a41`; the Linux jobs install,
  test and uninstall the `.deb` and the tarball. See [platform validation](platform-validation.md).
* The macOS PKG/DMG and the Linux packages have not been installed on hardware yet.
