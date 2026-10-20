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
