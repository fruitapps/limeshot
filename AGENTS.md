# LimeShot — instructions for agents

LimeShot is an open-source, offline-first, cross-platform DAW in Rust with an FL Studio–style pattern workflow
(Channel Rack, Piano Roll, Playlist, Mixer). It is an independent implementation: no FL Studio code or assets, and no
code copied from ArtCraft/SoundCraft. The full brief is `START.md`.

## Start every session here
1. Read `docs/development/next-steps.md` (prioritised work) and the latest entries in `docs/development/progress.md`.
2. Pick work from `docs/roadmap/features.json` (P0 → P3, respecting `dependencies`). Read the record first.
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
**Look at the UI** after UI changes. Render with `ui_shot` and read the PNG. Permissions depend on the execution host;
verify Screen Recording and Accessibility before live automation.
For live checks, use `cargo run -p limeshot-app --example live_check -- /tmp/check.lime` with a disposable project;
this harness disables shared autosave/recovery and does not save preferences on exit.
When granted, `screencapture -l <window id>` captures the live app and CoreGraphics events can drive it
(keyboard layout is QWERTZ). The
headless UI tests in `crates/limeshot-app/tests/ui.rs` take pointer positions from geometry the editors record.

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

## Current state (v0.1.0-alpha.1, 2026-10-25)
* The first prerelease ships installers for Windows x64, macOS Apple Silicon/Intel and Linux x86-64/ARM64
  (`docs/development/release-packaging.md`). CI passes on all platforms; hands-on GUI/audio checks exist for macOS
  Apple Silicon and partly for Windows (`docs/development/platform-validation.md`).
* Instruments and effects are the twelve Lime Native VST3 bundles (D39), hosted like any VST3. Build them with
  `python3 native-plugins/build.py` before running from source. Lime Test Gain / Tone are compiled-in test fixtures.
* Plugin editors live in internal Plugin / Settings wrappers. **PLG-006 is IMPLEMENTED, not VERIFIED**: keyboard,
  OS drops, pin/occlusion and host-edge resizing are open (`docs/development/live-plugin-checks-2026-10-25.md`).
  Windows embeds editors in owned popups; Linux shows parameters. VST3 class ids are portable hex (`util::tuid_to_hex`).
* Samples (WAV/MP3/FLAC/Ogg/AIFF) are bounded to 6 million stereo frames (about 125 s at 48 kHz) and decode once on
  the loader thread; Lime Sampler embeds them in its state and replaces them through bounded hand-off queues.
* Feature, shortcut, setting and menu counts live in the generated inventories; the latest test counts are in
  `docs/development/progress.md`.
* UI/UX follows FL Studio; ArtCraft informs the broad visual style. See `docs/architecture/ui.md` and D24/D35.

## Immediate next tasks
Follow `docs/development/next-steps.md`: verify native plugin embedding and the alpha packages on real hardware,
then continue the editor menus and production features by priority and dependencies. Keep unverified behavior and
disabled actions in `docs/roadmap/known-limitations.md`.

## REA (reverse-engineering research tool; not a runtime dependency)
Registered for Claude Code in `~/.claude.json` (`rea-agents@6.7.0`), skill in `~/.claude/skills/reverse-engineer-anything`.
The MCP tools appear after a Claude Code restart. Without them, use `npx -y rea-agents@6.7.0 <command>`. Evidence and
continuation steps: `docs/research/rea-findings.md`. Clean-room rules: observe behaviour and metadata only, never copy
decompiled code or bundle resources, and never modify the installed FL Studio.
