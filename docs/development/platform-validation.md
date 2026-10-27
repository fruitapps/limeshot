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

## 2026-10-27 — Recording monitoring/count-in local pass

`codex/input-monitoring`, macOS 26.6.2 arm64, primary checkout debug build and twelve debug Lime
VST3 bundles. Full workspace: 457 tests (161 UI, 10 hosted VST3); native suite: 22. Build/strict
Clippy/fmt/inventory/reference checks pass. Audio settings and active-count-in renders inspected.

Disposable `live_check`: AirPods Pro output at 48 kHz, system-default input capture at 24 kHz;
input monitoring On into muted insert 1, Record + Audio with Cmd+P count-in, Stop and successful
Playlist take placement at tick 0. The raw float WAV has 363,360 stereo frames (15.14 s), finite
nonzero PCM and no writer drop warning. Preferences/autosave were disabled; monitoring was turned
Off before exit. See [progress](progress.md) and [research](../research/recording-monitoring-and-count-in.md).
Audible monitoring, latency alignment, sustained drift/device loss, hardware MIDI and Windows/
Linux/Intel macOS behavior remain unverified; REC-003 and REC-007 remain IMPLEMENTED.
