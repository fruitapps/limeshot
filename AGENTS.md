# LimeShot — instructions for agents

LimeShot is an open-source, offline-first, cross-platform DAW in Rust with an FL Studio–style pattern workflow
(Channel Rack, Piano Roll, Playlist, Mixer). It is an independent implementation: no FL Studio code or assets, and no
code copied from ArtCraft/SoundCraft. The full brief is `START.md`.

## Start every session here
1. Read `docs/development/next-steps.md` (prioritised work) and the latest entry in `docs/development/progress.md`.
2. Pick work from `docs/roadmap/features.json` (P0 → P3, respecting `dependencies`). Read the record first.
3. Architecture: `docs/architecture/{audio-engine,project-model,ui}.md`; reasons: `docs/development/decisions.md`.

## Build, run, test
```
cargo build --workspace
cargo run --release -p limeshot-app [-- --demo | path/to/song.lime]
cargo run -p limeshot-app --bin limeshot-cli -- demo out/demo.lime --render out/demo.wav
cargo run -p limeshot-app --bin limeshot-cli -- play out/demo.lime --seconds 5     # realtime through the device
cargo run -p limeshot-app --example ui_shot -- out/ui.png [--piano --mixer --settings --melody --empty]
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
python3 scripts/features.py check        # or `generate` after editing features.json
cargo run -p limeshot-dsp --example make_samples   # regenerate bundled samples
```
**Look at the UI** after UI changes. Render with `ui_shot` and read the PNG (screen capture of a live window may be
blocked by macOS permissions). The headless UI tests in `crates/limeshot-app/tests/ui.rs` use pointer positions from
the default 1440×900 layout. Update them if the layout moves.

## Workspace
| Crate | Role |
|---|---|
| `limeshot-core` | time (960 PPQ ticks, exact `Timeline`), `Project` model, `Command` enum, `Document` (undo/redo, gestures) |
| `limeshot-project` | `.lime` I/O: versioned envelope, `migrate`, unknown-field preservation, atomic save, relative sample paths |
| `limeshot-dsp` | oscillators (PolyBLEP), ADSR, pan law, dB, WAV read/write |
| `limeshot-sequencer` | `compile(project, pattern, samples) → Snapshot` (the only model→sound translation) |
| `limeshot-engine` | `Renderer` (transport, scheduling, voices, mixer), offline `render_offline`/`export_wav` |
| `limeshot-audio` | `Player` (cpal stream, rtrb queues, `Status` atomics, silent fallback), `SampleCache` loader thread |
| `limeshot-ui` | egui app: `app.rs` state/glue, `toolbar`, `browser`, `channel_rack`, `piano_roll`, `playlist`, `mixer`, `dialogs`, `theme`, `icons`, `widgets` |
| `limeshot-app` | `limeshot` (GUI), `limeshot-cli`, `demo.rs` (demo song via commands), `ui_shot` example, MVP and UI tests |

## Coding conventions
* Every project change is a `Command` executed on the `Document`; UI code never mutates `Project` directly. Continuous
  controls use `LimeShotApp::exec_gesture` so a drag is one undo step. Add a `merge_key` for new continuous commands.
* Colors come from `theme::Theme::current()`; icons are painter code in `icons.rs`. No image or font assets beyond
  egui's defaults. Every bundled asset gets a row in `ATTRIBUTION.md`.
* No panics in production code: crate roots deny `unwrap/expect/panic/todo/unimplemented` (allowed in tests). Use
  typed errors (`thiserror`). Treat file input as hostile (`Project::sanitize`). `unsafe` is forbidden workspace-wide.
* Visible controls must work. Unimplemented features are shown disabled ("not implemented yet") or left out.
* Match the surrounding style; keep comments for the *why*.

## Real-time audio rules
In `Renderer::render` and the cpal callback: no allocation, no locks, no I/O, no logging, and no dropping of large
objects. Communicate only through the rtrb rings and `Status` atomics, and return replaced snapshots to the UI thread.
Preallocate in constructors. Add a test with several block sizes for any scheduling change. Details:
`docs/architecture/audio-engine.md`.

## Feature tracking
`docs/roadmap/features.json` is canonical; regenerate `features.md` with `scripts/features.py generate`.
IMPLEMENTED ≠ VERIFIED: VERIFIED needs tests (or a recorded manual verification) listed in `tests`. Update the status,
`notes` and `docs/roadmap/known-limitations.md` in the same change as the code. Add newly discovered FL Studio
features as records.

## Current state (2026-10-20)
* Milestones 0–5 done; milestone 6 mostly done (MVP workflow passes as an automated test on macOS; Windows and Linux
  CI not yet run).
* 217 features tracked: 51 verified, 37 implemented, 6 in progress, 120 not started, 1 researched, 2 deferred.
* Known problems: see `docs/roadmap/known-limitations.md`.

## Immediate next tasks
See `docs/development/next-steps.md`: run CI on Windows and Linux, a manual GUI pass, live FL observation with REA,
then copy/paste, quantize, swing and the effects framework.

## REA (reverse-engineering research tool; not a runtime dependency)
Registered for Claude Code in `~/.claude.json` (`rea-agents@6.7.0`), skill in `~/.claude/skills/reverse-engineer-anything`.
The MCP tools appear after a Claude Code restart. Without them, use `npx -y rea-agents@6.7.0 <command>`. Evidence and
continuation steps: `docs/research/rea-findings.md`. Clean-room rules: observe behaviour and metadata only, never copy
decompiled code or bundle resources, and never modify the installed FL Studio.
