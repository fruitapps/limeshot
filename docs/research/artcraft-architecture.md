# ArtCraft architecture research

> Investigated 2026-10-20 by reading source, manifests, CI workflows and agent docs (not only READMEs).
> No code was copied from any ArtCraft repository. LimeShot is an independent implementation.

## Repositories inspected

All cloned with `git clone --depth 1` into a scratch directory outside the LimeShot tree.

| Repository | Commit | Language / UI | Notes |
|---|---|---|---|
| storytold/soundcraft | `7b339fe921dfc1231051cd1263d582f6da589e75` | Rust, egui 0.36 / eframe 0.36, cpal 0.16 | Pro Tools–style DAW. Primary reference. |
| storytold/photocraft | `872748fb5b2a53e09dd4e385e31968d985c0515b` | Rust, egui 0.36 | Photo editor. Has the most complete `ci.yml`. |
| storytold/filmcraft | `70f0fc7a75f2681bc18302e67eaf0c9d41c68873` | Rust, egui 0.36 | Video editor. |
| storytold/effectcraft | `9db7f3b21700fd821f215782ed1c82afe077880f` | Rust, egui 0.36, wgpu 30, cpal 0.16 | Compositing app. |
| storytold/craft-launcher | `046d48fb2a44e74c19630c3f91fc9d36fc3ce2e5` | Rust, egui 0.36, egui_extras | Launcher for the suite. |
| storytold/artcraft | `40e04793b504147de97ba1f0659d3975167524e5` | Rust backend + **Tauri 2** + TypeScript frontend | The original ArtCraft app; its UI stack is excluded by LimeShot's constraints. |

The `storytold` org listing (GitHub API, same date) also has older Unreal/Bevy experiments; none were relevant.

## SoundCraft: relevant paths

| Path | What it shows |
|---|---|
| `Cargo.toml` | Workspace with `resolver = "3"`, edition 2024, centralized `[workspace.dependencies]`, `[workspace.lints]` with `unsafe_code = "forbid"`, `dev` profile with `opt-level = 2` for dependencies (keeps DSP usable in debug builds). |
| `AGENTS.md` | Layering table (L0–L6), "everything is a command", "never crash" rules (`#![deny(clippy::unwrap_used, …)]` per crate root), clean-room asset rules. |
| `docs/architecture.md` | Document model, command/undo, audio flow, agent control. |
| `crates/playback/src/lib.rs` | `Player` on the UI thread; the cpal callback owns the mix engine and an `Arc<Session>` snapshot sent through a channel. Position and meters come back via atomics and a `try_lock`. Replaced snapshots are sent **back** so the UI thread frees them. A silent clock thread replaces the device when none exists. |
| `crates/engine/src/lib.rs`, `crates/engine/src/cmd/mod.rs` | `CommandSpec { id, label, menu, shortcut, params, enabled, run }`; `Engine::execute(id, json)`; undo is a stack of previous `Arc<Session>` snapshots; `execute_merged` coalesces drags. |
| `crates/ui-egui/src/theme.rs` | Central `Tokens` struct of semantic colors, palette switch via an atomic. |
| `crates/ui-egui/src/icons.rs` | All icons drawn in code (no image assets). |
| `crates/time/` | Dedicated timebase crate (samples, grid, tempo map) at the bottom of the stack. |
| `crates/mix/src/lib.rs` | One mix engine shared by realtime playback and offline bounce. |
| `.github/workflows/*.yml` | Release, FreeBSD, Nix, Windows ARM64 (macOS jobs reserved for release). |
| `apps/soundcraft/src/main.rs` | eframe 0.36 bootstrap: `App::logic` + `App::ui`, icon from PNG bytes, close-request handling with "ask to save". |

## Useful patterns adopted (re-implemented, not copied)

1. **Layered workspace, UI at the top.** Nothing below `limeshot-ui` depends on egui. This lets tests and a CLI drive the whole product headlessly.
2. **One renderer for realtime and offline.** The same `Renderer::render` produces device audio and WAV export, so export tests also verify playback logic.
3. **Snapshots to the audio thread, garbage back to the UI thread.** The audio thread never frees large allocations; replaced snapshots go back through a bounded queue.
4. **Typed commands with snapshot undo.** Every project mutation is a command; undo stores the previous project state. Continuous gestures merge into one undo step.
5. **Semantic theme tokens** in one module, and **icons drawn in code**, so no third-party artwork is needed.
6. **No-panic discipline** in production crates (`clippy::unwrap_used` etc. denied at crate roots), typed errors via `thiserror`.
7. **Centralized dependency versions and lints** in the workspace manifest; `opt-level` bump for dependencies in dev builds.
8. **Repository-local progress tracking** (roadmap, gaps, parity docs) as the source of truth, with an `AGENTS.md` that tells the next session where to start.
9. **Clean-room rule set:** observe the reference app; never copy its resources, presets, artwork or wording beyond feature names.

## Patterns rejected or changed

| SoundCraft pattern | LimeShot decision | Reason |
|---|---|---|
| Commands dispatched by string id with JSON params (`engine.execute("track.new", json)`) | A Rust `enum Command` with typed fields, which is also `serde`-serializable | Compile-time checking for a smaller codebase; serialization still allows scripting from the CLI. |
| TCP JSON-lines control channel + MCP server for agent control | Headless `limeshot-cli` that runs a JSON command script and renders | START.md forbids adding a network service. The CLI covers headless tests and agent control. |
| WASM/web target (`soundcraft-web`) | Not built | START.md forbids browser/WASM targets. |
| Pro Tools sample-based timeline (`Samples` everywhere) | Musical ticks (960 PPQ) as the primary timebase; samples derived exactly with integer math | FL Studio's model is pattern/step based; musical positions must not drift when tempo changes. |
| `xtask` with layer/asset/parity checks | Deferred; CI runs fmt, clippy, tests directly | Fewer moving parts for an MVP. Worth adding once the crate graph grows. |
| `craft-ui` shared git dependency | Not used | LimeShot must stay independently maintained; it has its own widgets. |
| Session saved as JSON + `Audio Files/` folder | `.lime` JSON with a versioned envelope, relative sample paths, unknown-field preservation | Same idea, simpler bundle. A zipped bundle format is tracked as a later feature. |
| artcraft (Tauri + TS frontend) | Rejected entirely | Violates "no webview / no JS frontend". |

## Differences between SoundCraft and LimeShot

- **Workflow model.** SoundCraft is linear track/clip (Pro Tools). LimeShot is pattern-centric (FL Studio): a Channel Rack shared by all patterns, patterns holding per-channel notes, and a Playlist of clips that reference patterns by id (linked instances).
- **Instruments.** SoundCraft hosts plugins (CLAP/VST3/AU, isolated `unsafe` crates). LimeShot's MVP ships a built-in sampler and synth only; plugin hosting is tracked (P2) and would follow the same "unsafe isolated in one crate" rule.
- **Mixer routing.** LimeShot routes *channels* to *mixer inserts* (FL style) rather than one strip per track.
- **Scheduling.** SoundCraft renders clips in sample time; LimeShot schedules note events from ticks and splits audio blocks at event boundaries for sample-accurate onsets.

## Cross-platform implications

- egui/eframe (winit + wgpu/glow) and cpal (CoreAudio, WASAPI, ALSA/JACK) are what every egui ArtCraft app ships on macOS, Windows and Linux. That is evidence they work on all three.
- Linux CI needs `libasound2-dev` (cpal/ALSA) and X11/Wayland dev headers for winit; Windows and macOS need no extra system packages.
- File dialogs use `rfd` (as SoundCraft does): native panels on macOS/Windows, the XDG portal or GTK on Linux.
- Configuration directories come from the `directories` crate, not hard-coded paths.
- ArtCraft saves macOS CI minutes for release builds only. LimeShot's CI runs macOS, Windows and Linux on every push because the project is small. Verification is still recorded per platform in `docs/development/progress.md`.
