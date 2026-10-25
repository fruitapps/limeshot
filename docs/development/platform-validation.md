# Platform validation

Headless CI and hands-on GUI/audio-device checks are separate. A passing run applies to its commit only; later
commits need their own runs ([live CI](https://github.com/fruitapps/limeshot/actions/workflows/ci.yml)).

## v0.1.0-alpha.1 (`28d3a41`)

| Workflow | Run | Result |
|---|---|---|
| CI | [40219529679](https://github.com/fruitapps/limeshot/actions/runs/40219529679) | rustfmt; Clippy, build, workspace tests with the Lime bundles, headless demo render and inventories on Linux x86-64, Linux ARM64, macOS Apple Silicon, macOS Intel (`macos-15-intel`) and Windows x86-64; Windows ARM64 cross-build |
| Lime Native VST3 | [40219529749](https://github.com/fruitapps/limeshot/actions/runs/40219529749) | Native workspace build, tests and bundle verification on macOS, Windows and Linux |
| Release packages | [40219554203](https://github.com/fruitapps/limeshot/actions/runs/40219554203) | The published assets: Windows x64 installer and ZIP, macOS arm64/x86-64 DMG and PKG, Linux x86-64/ARM64 tarball and `.deb`, `SHA256SUMS.txt`. The Linux jobs install the `.deb` and the tarball (`--user`), render the demo with the installed CLI while the checkout's bundles are hidden, and uninstall both |

The earlier packaging run [40211718081](https://github.com/fruitapps/limeshot/actions/runs/40211718081) was checked
by hand after download: every asset matched its checksum, both DMGs passed `hdiutil verify`, the ZIP's CRCs passed
and both tarballs contained the twelve VST3 bundles and their install scripts. It also showed that GitHub renames
`~` in asset names, so Debian filenames now keep the `0.1.0-alpha.1` spelling.

## Hands-on

| Platform | State |
|---|---|
| macOS Apple Silicon | GUI, realtime audio, VST3 processing and native editors exercised; parts of native embedding remain open ([live checks](live-plugin-checks-2026-10-25.md)) |
| Windows x86-64 | The owner's runs: Lime and third-party editors appear and respond, Explorer drops work; no systematic pass |
| macOS Intel | An early Rosetta render matched arm64; GUI and audio devices not exercised |
| Linux x86-64 / ARM64 | GUI and audio devices not exercised |
| Windows ARM64 | Cross-build only; no package |
