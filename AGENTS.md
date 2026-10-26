# LimeShot — contributor and agent guide

LimeShot is an open-source, offline-first, cross-platform DAW in Rust with an FL Studio–style pattern workflow
(Channel Rack, Piano Roll, Playlist, Mixer). It is an independent implementation: no FL Studio code or assets, and no
code copied from ArtCraft/SoundCraft. Binding rules: `docs/development/principles.md`.

## Checkout and build cache
Create and use branches in your existing primary checkout. Run builds and tests from that checkout so Cargo's
build cache is reused. Avoid separate worktrees or clones for routine branch work. Preserve local changes when
switching branches, and include the checkout and cache guidance in handoffs.
All paths and commands below are relative to the repository root unless stated otherwise.

## Start every session here
1. Read `docs/development/next-steps.md` (prioritised work) and the latest entries in `docs/development/progress.md`.
2. Read the relevant records in `docs/roadmap/features.json` before changing a feature. When choosing new work,
   follow P0 → P3 priority and respect `dependencies`.
3. Architecture: `docs/architecture/{audio-engine,project-model,ui}.md`; reasons: `docs/development/decisions.md`.

## Build, run, test
```
python3 native-plugins/build.py   # the twelve Lime VST3 bundles LimeShot plays through (once, and after changing them)
cargo build --workspace
cargo run --release -p limeshot-app [-- --demo | path/to/song.lime] [--show=piano --detach=mixer]
cargo run -p limeshot-app --bin limeshot-cli -- demo out/demo.lime --render out/demo.wav
cargo run -p limeshot-app --bin limeshot-cli -- play out/demo.lime --seconds 5     # realtime through the device
cargo run -p limeshot-app --example ui_shot -- out/ui.png [--piano --mixer --fx --settings --melody --empty --audio --automation --synth --plugin [--plugin-window]]
cargo run -p limeshot-app --bin limeshot-cli -- midi out/demo.lime out/demo.mid      # or import-midi in.mid out.lime
cargo run -p limeshot-app --bin limeshot-cli -- import-flp song.flp out/song.lime    # FL Studio project → .lime
cargo run -p limeshot-project --example make_flp_fixtures                           # regenerate synthetic .flp fixtures
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
python3 scripts/features.py check        # or `generate`; also checks keybindings.json and settings.json
python3 scripts/check_repository.py      # offline local links, evidence and embedded assets
cargo run -p limeshot-dsp --example make_samples   # regenerate bundled samples
scripts/package-macos.sh                           # dist/LimeShot.app, a DMG and a PKG (ad-hoc signed)
scripts/package-linux.sh                           # tarball with install.sh, and a .deb
powershell -File scripts/package-windows.ps1       # Inno Setup installer and portable ZIP (docs/development/release-packaging.md)
```
**Look at the UI** after UI changes. Render with `ui_shot` and inspect the PNG.
For live checks, run `cargo run -p limeshot-app --example live_check -- <temporary-project.lime>` with a disposable
project in your system's temporary directory. This harness disables shared autosave/recovery and does not save
preferences on exit. Verify the permissions required by your platform before live automation. On macOS these
include Screen Recording and Accessibility; `screencapture -l <window id>` can capture the app window.
Use the local keyboard layout when automating input. Headless UI tests in `crates/limeshot-app/tests/ui.rs`
take pointer positions from geometry the editors record.

## Workspace
| Crate | Role |
|---|---|
| `limeshot-core` | time (960 PPQ ticks, exact `Timeline`), `Project` model, `Command` enum, `Document` (undo/redo, gestures) |
| `limeshot-project` | `.lime` I/O: versioned envelope, `migrate`, unknown-field preservation, atomic save, relative sample paths; `midi` (SMF), `flp` (FL Studio import, format notes in `docs/research/flp-format.md`) |
| `limeshot-dsp` | oscillators (PolyBLEP), ADSR, pan law, dB, WAV read/write |
| `limeshot-sequencer` | `compile(project, pattern, samples) → Snapshot` (the only model→sound translation) |
| `limeshot-engine` | `Renderer` (transport, scheduling, voices, mixer), `fx` (effect chains), offline `render_offline`/`export_wav` |
| `limeshot-audio` | `Player` (cpal stream, rtrb queues, `Status` atomics, silent fallback), `SampleCache` loader thread |
| `limeshot-plugins` | VST3 hosting (`vst3` bindings): `Vst3Module` (bundle loading, classes), `scan`, `Vst3Instance` (control thread: params, state) + `Vst3Processor` (audio thread), `Vst3Editor` (the plugin's own view in a host window), built-in `testing` plugins, `examples/vst3_probe`. The only crate allowed `unsafe` (D32) |
| `limeshot-ui` | egui app: `app.rs` state/layout/glue, `toolbar`, `browser`, `channel_rack`, `piano_roll`, `playlist`, `mixer`, `workspace` (window manager), `inspector` (Channel Settings window body), `plugins` + `plugin_window` (plugin database, running plugins, plugin windows), `effects_ui`, `dialogs`, `theme` (tokens + fonts), `icons`, `widgets` |
| `limeshot-app` | `limeshot` (GUI), `limeshot-cli`, `demo.rs` (demo song via commands), `ui_shot` example, MVP and UI tests |

## Coding conventions
* Every project change is a `Command` executed on the `Document`; UI code never mutates `Project` directly. Continuous
  controls use `LimeShotApp::exec_gesture` so a drag is one undo step. Add a `merge_key` for new continuous commands.
* Colors come from `theme::Theme::current()`; icons are painter code in `icons.rs`. Use painter icons and the
  existing bundled fonts; avoid adding assets without a need. Every bundled asset gets a row in `ATTRIBUTION.md`.
* No panics in production code: crate roots deny `unwrap/expect/panic/todo/unimplemented` (allowed in tests). Use
  typed errors (`thiserror`). Treat file input as hostile (`Project::sanitize`). `unsafe` is forbidden workspace-wide, except in `limeshot-plugins` (FFI; every block has a `SAFETY:` comment).
* Visible controls must work. Unimplemented features are shown disabled ("not implemented yet") or left out.
* Match the surrounding style; keep comments for the *why*.

## Real-time audio rules
In `Renderer::render` and the cpal callback: no allocation, no locks, no I/O, no logging, and no dropping of large
objects. Communicate only through the rtrb rings and `Status` atomics, and return replaced snapshots to the UI thread.
Preallocate in constructors. Add a test with several block sizes for any scheduling change. Details:
`docs/architecture/audio-engine.md`.

## Feature tracking
`docs/roadmap/features.json` is canonical; regenerate `features.md` with `scripts/features.py generate`.
FL Studio's shortcuts, settings and main menu items are tracked in `docs/roadmap/keybindings.json`, `settings.json` and
`menus.json` (same command; a menu label LimeShot claims must exist in the UI code).
When adding or changing a shortcut, update `shortcuts.rs`'s help tables and `keybindings.json` together; the check
fails if a binding LimeShot claims is not in the help list, or a claimed setting is not a real field.
IMPLEMENTED ≠ VERIFIED: VERIFIED needs tests (or a recorded manual verification) listed in `tests`. Update the status,
`notes` and `docs/roadmap/known-limitations.md` in the same change as the code. Add newly discovered FL Studio
features as records.

## Project status and validation
Use `docs/development/progress.md` for the latest implementation and test results, and
`docs/roadmap/known-limitations.md` for open behavior and verification gaps. Platform evidence is recorded in
`docs/development/platform-validation.md`; packaging details are in `docs/development/release-packaging.md`.
Build the twelve Lime Native VST3 bundles before running from source. Report the platform and build tested;
a successful build or headless test does not establish live GUI/audio behavior on other platforms.

## Immediate next tasks
Follow `docs/development/next-steps.md`: verify native plugin embedding and the alpha packages on real hardware,
then continue the editor menus and production features by priority and dependencies. Keep unverified behavior and
disabled actions in `docs/roadmap/known-limitations.md`.

## Optional reverse-engineering research tools
REA is a research aid, not a runtime or contributor prerequisite. Tool availability and configuration depend on
local setup; see `docs/research/rea-findings.md` for recorded evidence and continuation steps.
Clean-room rules apply to all research: observe behavior and metadata only, never copy decompiled code or bundle
resources, and never modify the installed FL Studio.
