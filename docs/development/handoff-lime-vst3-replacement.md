# Handoff: replace LimeShot's built-in instruments and effects with the Lime VST3s

Written 2026-10-22 by a Claude Code session that ran out of usage before starting the implementation.
**Owner decision (2026-10-22): full replacement.** Delete the built-in (in-process, non-VST) LimeSynth, Sampler
and mixer effects. LimeShot must use the Lime Native VST3 plugins (PLG-012, `native-plugins/`) by default. Old
projects are converted to VST3 references. Every capability lost in the rewrite is documented as a feature record
with the **highest priority** (P0) to be implemented next.

Keep **Lime Test Gain / Lime Test Tone** (`crates/limeshot-plugins/src/testing.rs`, `builtin:limeshot-test`).
They are compiled-in VST3 test fixtures that users never see. The owner chose to keep them so CI can test hosting
without installed plugins.

## Status (2026-10-23, Codex continuation on `claude/lime-vst3-default`)

PLG-013 remains **IN_PROGRESS**. The owner explicitly authorized the supplied takeover prompt: continue the
existing branch and draft PR #8 instead of making a new branch/PR for this continuation. Do not merge.

Completed:
* Portable Lime references, legacy conversion, embedded sample state and bundle resolution remain in place.
* GUI and headless constructors now share the converter. Rack add actions and Browser loads create Lime
  references. Browser/Playlist previews and audio-source reuse recognize Lime Sampler; source hints save relative
  and reopen absolute. Root-key edits reach sampler state.
* Deleted the former synth/sampler and insert parameter panels. Hosted controls retain factory/user `.limepreset`
  interchange, source-file loading and waveform audition. Lime parameter edits write undoable state directly;
  native and host parameter definitions have one source in `limeshot_core::lime`. `SetPluginState` merges per drag.
* Existing instances follow changed document state, including undo/redo and sample replacement. Loading failures
  show app error toasts; failed plugin loading/state recall aborts export. CLI play and every CLI save/import path
  now convert to Lime references too.
* Fixed the live macOS black-host-window regression when native editors render: restore the main viewport's
  OpenGL context before/after its UI. The plugin renderer clears the current context, but glutin's macOS surface
  check only compares bound views. Live Lime Sampler rendering and Plugin/Settings tab switching now retain the
  surrounding host UI. PLG-006 is still IMPLEMENTED: broader native interaction checks remain.
* `python3 build.py` built all nine release bundles. The real-binary verifier passed all nine at 44.1/48/96 kHz
  and 1/64/333/1024-frame buffers. `scripts/package-macos.sh` built the app/DMG with `Contents/PlugIns`; deep
  signature verification passed. The packaged CLI rendered the demo at peak −3.7 dBFS.
* Realtime CLI playback through converted Lime instruments: MacBook Pro Speakers, 44.1 kHz, 324 callbacks in
  2 seconds, peak −5.3 dBFS. This is measured output, not an independent listening assessment.
* All 134 headless UI tests pass, plus the DAW workspace suite and 13 native tests. Non-automated equivalence now
  asserts identical sample bits (only trailing silent frames may differ in length), and passes with RMS difference 0.
  Both workspaces' fmt/Clippy, inventories and repository references pass.

Next, in order:
1. **Delete the built-in playback paths.** `InstrumentState::{Synth, Sampler}` and `fx::build`/`build_all` still
   exist in the engine/sequencer and are exercised by legacy DSP/engine tests. Keep shared Voice/DSP for native
   plugins and preserve legacy enum deserialization. Move DSP assertions to native/shared tests and retain exact
   conversion-equivalence evidence when replacing the reference renderer. Core legacy creation commands/importers
   still accept old enums; the app/CLI convert before playback/save.
2. **Preserve and address P0 losses.** PLG-017…020 and PLG-022 remain. Newly confirmed PLG-024…028 cover silent
   hosted Playlist audio clips, note pan, full host typed-value menus, host dynamics meters and voice statistics.
   The old audio-clip engine tests use built-ins; do not mistake them for VST3 playback coverage.
3. Continue native embedding validation: resize, occlusion, pinning, keyboard forwarding, reopening and third-party
   editors (Vital), with special attention to detached viewports. See [lime-vst3-host-validation.md](lime-vst3-host-validation.md).
4. **External validation remains unavailable.** GitHub Actions is blocked by account billing. Keep relying on local
   checks and state this in PR #8. Windows/Linux/Intel runtime checks remain open. `pluginval` was unavailable in
   this continuation and was not rerun; older recorded passes are not evidence for the latest binaries.

## Session policy

The owner's explicit takeover instruction overrides the new-session branch/PR rule **for this continuation only**:
continue `claude/lime-vst3-default`, push to it and update draft PR #8. Never merge automatically. For every future
session, repeat and follow AGENTS.md: a NEW `codex/` branch in a separate managed worktree, a NEW draft PR stacked
on the latest unmerged continuation, and no new chat/session unless the owner asks.

## State of the repository at handoff

* `main` = `b045db8` (PRs #3 and #6 merged). PR #7 (`claude/s16-synth-tempo`) is open, targets `main`, has `main`
  merged in, and has no conflicts. It also fixes the native-plugin build that is red on `main`, where Lime Sampler
  lacked the sampler envelope fields.
* Cleanup is done: the merged worktrees and the local and remote branches are deleted, and the build outputs
  (`target/`, `native-plugins/{target,dist}`) are removed.
* The owner's main checkout (`~/Developer/limeshot`) is on `main`. Its earlier uncommitted state, an older draft
  of the native suite superseded by #6 plus three docs, is kept in `git stash` as "Backup before cleanup
  2026-10-22". The three docs were moved into PR #7: `docs/development/test-coverage-audit.md`,
  `docs/roadmap/first-release.md` and `docs/roadmap/first-release-features.md`. Their baseline is pinned to
  `dd53fb3` / 272 features. Update them after the replacement, because their scope changes.

## What exists today (verified by reading the code)

* **Built-ins.** `Instrument::{Sampler, Synth, Plugin, Unsupported}` (`limeshot-core/src/model.rs`). `EffectKind`
  (`limeshot-core/src/effects.rs`) has Utility, Eq3, Filter, Delay, Reverb, Compressor, Limiter, Gate, Drive,
  Chorus and Plugin. The sequencer compiles these to `InstrumentState::{Synth, Sampler, Plugin}`, and
  `limeshot-engine::fx::build` builds the effects.
* **Lime VST3s.** There are nine: LimeSynth (ID 0), Lime Sampler (1), Utility, EQ, Filter, Delay, Room, Comp and
  Limiter (2–8). They are built from the separate `native-plugins/` workspace with nice-plug 0.4.2
  (`python build.py`, which writes to `native-plugins/dist/`). Class IDs are stable (`VST3_CLASS_ID` in
  `native-plugins/shared/src/processor.rs`). Parameters are a nested array of `value` params in the order of
  `native-plugins/shared/src/parameters.rs`. The sample is a persisted field, `sample-v1`. The DSP is the DAW's
  own (`limeshot-dsp`, `limeshot-engine::voice::Voice`).
* **The DAW does not know the Lime VST3s exist.** Nothing in `crates/` references `native-plugins` or the
  bundles, and the app does not ship them.
* **Hosting.** `limeshot-plugins`: `load_module(path)` (a path, or `builtin:` for the test fixtures), the
  `Vst3Instance` / `Vst3Processor` split, an optional out-of-process bridge (`PluginRef::separate_process`), and
  `PluginRef.state`, which holds the opaque component state base64-encoded. `limeshot-engine/tests/plugins.rs`
  covers hosted instruments and effects, including latency.

## Proposed implementation plan

1. **Make the bundles available to the DAW.** Build `native-plugins` as part of the app build and packaging:
   `scripts/package-macos.sh` copies them into `LimeShot.app/Contents/PlugIns/`, and CI builds them before the
   DAW tests that need them. Add a resolver that checks, in order: an env override, the app bundle's `PlugIns/`,
   the dev fallback `native-plugins/dist/`, then the standard VST3 folders. Identify each plugin by class ID, not
   by path, so moved installs still resolve.
2. **Defaults.** New channels (`Add channel`, sample drops in `limeshot-core/src/command.rs` around line 1393,
   templates, the demo project) and new mixer slots create `Instrument::Plugin` / `EffectKind::Plugin` with Lime
   class IDs. Add-menu entries list the Lime VST3s first.
3. **State from parameters.** Write a helper that builds the nice-plug component state from plain values, so the
   DAW can create a configured Lime Synth or Sampler without opening it. Sample drops need the embedded WAV in
   `sample-v1`. Check nice-plug's state JSON format and its nested-array param ID scheme first. If that is not
   practical, set parameters through `Vst3Instance` after instantiation and store the resulting state.
4. **Convert old projects on load.** `Instrument::Synth` becomes LimeSynth. `Instrument::Sampler` becomes Lime
   Sampler, embedding the referenced sample. The 7 exported effect kinds become their VST. Map values; settings
   with no VST equivalent are dropped with a load warning (list below). Gate, Drive and Chorus have no VST: keep
   them as `Unsupported` / bypassed slots so saving does not lose them, and warn. Bump the project format version.
   Keep `.limepreset` reading working, because the VSTs share that format.
5. **Delete the built-ins.** Remove the built-in synth, sampler and effect paths: `InstrumentState::{Synth,
   Sampler}` compile, Channel Settings synth and sampler panels (`limeshot-ui/src/inspector.rs`), built-in
   automation targets (replace them with plugin-parameter automation), and built-in effect editors. The shared
   DSP crates stay, because the VSTs link them.
6. **Tests.** Rewrite engine, UI and app tests that use built-in instruments. Either build the bundles in CI and
   use them, or keep the DSP-level tests in `limeshot-dsp` / `limeshot-engine::voice` and use the test fixtures
   for hosting. Re-run the native verifier and pluginval.
7. **Records.** Update `features.json` (statuses of instrument, effect and automation records whose behaviour
   changes), `known-limitations.md`, `decisions.md` (new decision: built-ins replaced by Lime VST3s), README,
   `progress.md`, and the first-release docs.

## Capabilities lost by the replacement → new P0 feature records

Add these to `features.json` as P0 (NOT_STARTED), link them from `next-steps.md`, and verify each against the code
before writing it down:

* **Lime Sampler VST:** sampler attack/decay/sustain envelope (it is a gate today); sample loop with loop
  start/end; the sampler low-pass filter (cutoff/resonance); and any other `SamplerParams` field that has no VST
  control.
* **Effects without a VST:** Gate, Drive (soft clip, hard clip, wave folding) and Chorus (chorus, flanger,
  phaser), each as a new Lime VST3.
* **Lime Comp:** the DAW's mixer-track sidechain selector. The VST uses a host key bus instead, and LimeShot's
  hosting must route a mixer track into that bus.
* **Channel Settings integration:** inline synth and sampler panels, sample drag-and-drop onto an existing
  channel, sampler waveform preview in the Channel Rack, and per-parameter automation clips with names. Check
  that plugin-parameter automation covers each of them.
* **Other gaps to verify:** project load without bundles available (graceful fallback), offline export
  determinism with hosted plugins, plugin latency and CPU with many sampler instances (bridge off by default),
  and Windows/Linux bundle locations.

## Checks to run before opening the PR

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` in
both the repo root and `native-plugins/`; `python3 scripts/features.py check`; `python3 scripts/check_repository.py`;
`python build.py` plus the `verify` example on `dist`. Open the demo project in the running app and confirm it plays
through the Lime VST3s.
