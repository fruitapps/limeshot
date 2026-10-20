# Progress log

## 2026-10-20 — Session 1: research, architecture, MVP vertical slice

**Environment:** macOS 26.6.2 (arm64, MacBook Pro), Rust 1.99.0 stable, Node 26.12, FL Studio 2026 26.1.1.5313 at
`/Applications/FL Studio 2026.app`; default output MacBook Pro Speakers (44.1 kHz system rate; LimeShot opened it at
48 kHz f32).

Done:
1. Cloned and read the ArtCraft repos (commits in `docs/research/artcraft-architecture.md`), especially SoundCraft's
   playback, engine command registry, theme and CI.
2. Installed REA for Claude Code (`npx rea-agents setup --client claude_code`, plan reviewed with `--dry-run`).
   Verified the registration and skill. Used the REA CLI for plist/Mach-O evidence. Live UI observation was blocked by
   macOS Screen Recording permission. Evidence: `docs/research/evidence/rea/`.
3. Read the FL Studio manual pages for the core windows → `fl-studio-analysis.md`, `workflow-comparisons.md`.
4. Built the workspace: core model and commands, `.lime` format, DSP, sequencer, engine, cpal output, egui UI, desktop
   app, CLI, bundled synthesized drum samples.
5. Feature inventory: 217 features in `docs/roadmap/features.json` (validator and generator: `scripts/features.py`).

Verification (all on this machine):
* `cargo test --workspace`: all tests pass (core 21, project 4, dsp 13, sequencer 5, engine 15, audio 4,
  ui settings 1, app MVP workflow 2, app UI 7).
* `cargo clippy --workspace --all-targets -D warnings` and `cargo fmt --check`: clean.
* `limeshot-cli demo out/demo.lime --render out/demo.wav`, then `render` of the saved file: bit-identical WAVs
  (15.70 s, peak −2.4 dBFS).
* x86_64-apple-darwin release build under Rosetta: identical demo render.
* `cargo check` for x86_64-pc-windows-msvc and aarch64-pc-windows-msvc: passes. Linux cross-check stops at `alsa-sys`
  (needs target ALSA via pkg-config), which is expected.
* GUI launched (`limeshot --demo`): opens the audio device. Offscreen renders reviewed (`docs/screenshots/`).
* Realtime: `limeshot-cli play out/demo.lime --seconds 6` gave 563 callbacks, peak −3.7 dBFS, 1.1% load, and position
  4:01:394 after 6 s (expected 4:01 at 124 BPM).

Not verified: interactive mouse use of the GUI window by a person; Windows and Linux builds (CI not run yet; no remote
repository configured); multi-device audio switching.

### Manual checks (to repeat after UI changes)
1. `cargo run --release -p limeshot-app -- --demo`, press Space: drums play; L switches to Song mode and the Playlist
   playhead moves; the mixer meters move (F9).
2. Channel Rack: click/drag steps on and off; right-drag erases; change tempo while playing.
3. F7: draw, move and resize notes; drag in the velocity lane; click keys to audition.
4. Playlist: place, move, resize and delete clips; ▾ › Make unique; double-click opens the pattern.
5. Browser: click a sample (preview); double-click (loads into the selected sampler); drag onto the Channel Rack (new
   channel).
6. Cmd+S, quit, reopen; File › Export WAV; play the file in another player.

## 2026-10-20 — Session 2: live observation, ArtCraft-style UI, editing depth, insert effects

Permissions: Screen Recording granted; Accessibility not granted.

Done:
1. **FL Studio live capture.** `screencapture -l` of the main window and welcome dialog (`docs/research/evidence/
   screenshots/`). Recorded the observed layout (toolbar rows, PAT/SONG pills, 140 BPM default, B:S:T display, Playlist
   maximised with a pattern picker, LED track headers) in `fl-studio-analysis.md` §1a. REA `observe-native-ui` still
   refuses without Accessibility (`evidence/rea/observe-ui-965.json`).
2. **UI redesign** after studying the ArtCraft apps' screenshots and theme code (FilmCraft tokens, Inter/JetBrains Mono,
   SoundCraft clips and mixer): docked layout, Inspector, dock tabs, pattern picker, flat toolbar, new tokens and fonts.
   Verified with offscreen renders and a live capture of the running app (`docs/screenshots/live-macos.png`).
3. **Bug found by a new UI test and fixed:** drag-painting steps switched to erasing after the first step.
4. **Editing:** swing (sequencer), quantize (command + Alt+Q + button), copy/cut/paste for notes, rack steps and clips.
5. **Insert effects** (model, commands, original DSP, real-time-safe processor hand-off, export tails, Inspector
   editor, strip slots). The demo song uses a reverb, a delay and a master limiter.
6. Window title now uses the project title for unsaved projects.

Verification on this machine:
* `cargo test --workspace`: 92 tests pass. Clippy (`-D warnings`) and `cargo fmt --check` are clean.
* `limeshot-cli demo … --render`: 16.81 s with effect tails, peak −3.7 dBFS.
* `limeshot-cli play out/demo.lime --seconds 4`: realtime with effects at 5.9% audio load, position correct.
* Live app launched (`limeshot --demo`); window captured and reviewed.

Not verified: Windows/Linux (no CI run; no remote), operating the live GUI with real input, FL Studio interaction
details.

## 2026-10-20 — Session 3: back to FL Studio's window model

The user clarified that FL Studio is the UI/UX source and ArtCraft only the broad style. The docked layout from session 2
was replaced by FL's model (`workspace.rs`, decision D24): editor windows in a workspace, maximise, detach into OS
windows, FL's default layout, Channel Settings as a window, FX slots in the Mixer's right panel.

Found and fixed: viewing the Mixer committed a "Mixer volume" edit (egui slider rounding) — guarded by
`widgets::user_changed`, with a regression test (D25).

Verification: 95 tests pass; Clippy and fmt clean. Offscreen renders reviewed. The live app was started with
`--demo --detach=mixer --show=piano`: a second OS window "Mixer — LimeShot" appeared, and both windows were captured
(`docs/screenshots/live-macos.png`, `live-mixer-detached.png`).

Then: **mixer routing** (MIX-008/013) — `Route`s on mixer tracks (inserts → master by default), `SetRoute` with feedback
refusal, topological mixing order in the snapshot, FL-style route buttons and send knobs on strips, an Output section
in the track panel. 99 tests pass.

## 2026-10-20 — Session 4: live FL Studio observation and a real-input pass

Permissions: Screen Recording and Accessibility granted. Synthetic mouse and keyboard input (CoreGraphics events from a
small throwaway helper outside the repository) reaches both FL Studio and LimeShot. REA's MCP tools are loaded; FL
Studio's UI is custom-drawn, so input was driven by coordinates read from screenshots rather than the accessibility tree.

FL Studio 2026.1.1, observed live (`fl-studio-analysis.md` §2–§4, screenshots in `docs/research/evidence/screenshots/`):
1. **Steps:** both mouse buttons toggle; a drag paints the state of the first step. LimeShot now does the same (D26).
2. **Clip looping:** a lengthened pattern clip is silent after its pattern ends — confirms D14.
3. **Default note length:** the first drawn note is two steps; later notes take the last drawn/resized length.
   LimeShot's initial length changed from one step to two.

Real-input pass over the live LimeShot window (`limeshot --demo`): moving a workspace window, painting steps,
right-click toggles, drag-erase, Cmd+Z (one undo step for a drag), Space in Pattern and Song mode (playhead, meters,
voices), F6/F7/F9, drawing, resizing and remembered note length in the Piano Roll, placing and right-click deleting a
Playlist clip, dragging a Mixer fader. All behaved as expected except:
* **Bug found and fixed:** the Mixer's body (and a floating Playlist's) was transparent — the windows behind showed
  through. The bodies are laid out in child UIs that did not advance the parent cursor, so the window frame only
  covered the title bar. Regression test `floating_window_frames_cover_their_body` (fails without the fix).

Notes for the next real-input pass: this Mac uses a German (QWERTZ) layout, so the Z key is virtual keycode 16; and
driving apps through AppleScript "System Events" triggers a macOS Automation prompt — use CoreGraphics events instead.

Verification: `cargo test --workspace` (101 tests) passes; Clippy (`-D warnings`) and `cargo fmt --check` are clean.

Not verified: Windows/Linux (CI results not readable from this machine: `gh` is not installed and the repository is
private).
