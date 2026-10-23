# Roadmap to the first public LimeShot release

**2026-10-23 replacement update:** this checklist/estimate retains its 272-record historical baseline. The current
inventory has 290 records. Active instruments/effects now require bundled Lime VST3s; the in-process implementations
are deleted. Release gates must include PLG-017…020, PLG-022 and PLG-024…029: missing effects, key-bus routing,
source integration, silent Playlist audio clips, note pan, host value menus/meters/statistics and inactive track EQ.
Prior baseline VERIFIED labels do not establish these replacement behaviors. See
[known limitations](known-limitations.md) and the [current inventory](features.md).

Planning baseline: **22 October 2026**, remote `main` at
[`dd53fb317085b58816c9c8bafb0071b92aa94dde`](https://github.com/fruitapps/limeshot/commit/dd53fb317085b58816c9c8bafb0071b92aa94dde).
This is a proposed release scope and effort forecast, not an announced launch date. “First release” here means
**v0.1: a downloadable, supported desktop app that someone can use to finish a song**, beyond the source-built MVP.
The version number is a planning label. Full FL Studio feature parity is a longer-term goal.

**Planning estimate: 12–20 calendar weeks with one experienced Rust/audio developer working full time with AI assistance.**
Plan around 16 weeks; allow 20–26 weeks if native plugin integration, recording/device behavior or platform testing
requires substantial rework. An earlier 10–12-week release requires few surprises and hardware, testers and signing
credentials available immediately. Do not turn these ranges into a date until the initial validation phase is complete.

The [complete release feature checklist](first-release-features.md) assigns every one of the 272 baseline feature
records to included, bounded, new work or later. It is the detailed scope companion to this readable plan.

## What is already there

LimeShot already supports the central workflow: load sounds, program drums, write melodies, arrange patterns and
audio, automate controls, mix with effects, save/reopen a project and export a WAV. The MVP does not need to be
rebuilt. Main contains a native Rust/egui app, an offline renderer and CLI, a command-based undo system and a
versioned project format.

| Baseline inventory status | Features | What that means for release planning |
| --- | ---: | --- |
| VERIFIED | 145 | Evidence is recorded, often automated; this does not establish every platform or hardware scenario. |
| IMPLEMENTED | 21 | Code exists; release acceptance and hands-on verification still matter. |
| IN_PROGRESS | 16 | Some behavior exists; the full feature name can overstate what is usable. |
| NOT_STARTED | 85 | Mostly later production depth and workflow options. |
| RESEARCHED | 1 | Automation STEP/SLIDE display research. |
| DEFERRED | 4 | Cloud content, stem separation and control creator. |
| **Total** | **272** | These are records, not equal-sized units of effort or a percentage of release readiness. |

Recent main-branch additions include MIDI input and note recording, an audio-recording implementation, MP3/FLAC/
Ogg/AIFF sample loading, clip fades, sampler loops/filter, automation point modes and moving controls, export
ranges, gate, drive and chorus/flanger/phaser. Some README, next-step and limitation text predates these additions.
For this plan, the individual feature records, latest progress entries and relevant code take precedence over
those older summaries.

The difficult remaining work is proving that these features cooperate in real sessions. Audio recording has not
had a real-input pass. MIDI has virtual-port evidence rather than hardware-keyboard validation. Native macOS plugin
embedding still needs a reliable live pass. Other platforms need real GUI/audio/device testing, beyond compilation.
The macOS packaging script exists, but its package is ad-hoc signed; Windows/Linux packages remain to be built.

## The first-release product

The target is an offline-first pattern DAW for beatmaking, instrument composition, basic audio recording and
mixing. No account or online service is required to compose, save or render. Each section below is part of v0.1;
limits stated here are intentional release boundaries.

| Area | What the first release includes |
| --- | --- |
| Projects and safety | New/open/save/save-as, atomic versioned `.lime` files, migrations and unknown-field preservation, relative sample references, dirty-state prompts, undo/redo and history, recent projects, metadata, settings, autosave/recovery, and a portable project bundle containing samples. |
| Channel Rack | Sampler/synth/plugin channels, steps and pattern lengths, multiple patterns, mute/solo and routing, channel volume/pan, selection and channel organization, groups/filters, copy/paste, global swing, fill/rotate and velocity/pan graphs. |
| Piano Roll | Draw/paint/delete/mute/slice/select/zoom/playback tools; note move/resize, velocity, snapping, audition, playhead, clipboard, duplicate, transpose/nudge, ghost notes, quantize, glue/slice, stamps, scale highlighting and the existing simplified note-tool dialogs. MIDI file import/export and note recording are included. |
| Playlist | Linked pattern clips and make-unique; audio and automation clips; move/trim/offset/slip/slice/duplicate/mute and clipboard; picker, snapping/zoom, markers/time selection/looping, track organization/mute locks and audio fade handles. Existing pattern merging is included. |
| Instruments and samples | LimeSynth oscillators, ADSR, low-pass filter, envelope/LFO modulation; sampler trim, pitch/root key, reverse, release, loop points and low-pass filter; polyphony/voice stealing; bundled sounds and factory/user presets. WAV, MP3, FLAC, Ogg Vorbis and AIFF sample input. |
| Mixer | Master/inserts, channel routing, track management, faders/pan, mute/solo/meters/clipping, insert chains, built-in track EQ, insert-to-insert routing, pre/post-fader sends, compressor sidechain and bounded plugin delay compensation. Utility provides width/polarity processing. |
| Built-in effects | Lime EQ, Filter, Delay with tempo sync, Room reverb, Comp, Limiter, Gate, Drive, Chorus with flanger/phaser modes, and Utility; effect bypass, parameter editing and factory/user presets. |
| Automation | Song-mode clips for supported channel/mixer controls, built-in effect parameters and VST3 parameters; point editing, tension and the implemented curve/hold/stair/pulse/wave modes; supported controls visibly follow playback. |
| Playing and recording | Realtime output, Pattern/Song transport, tempo/time signature, metronome, looping, position/load/voice display and panic; typing keyboard and one MIDI input; MIDI notes recorded into patterns; audio input selection and WAV takes placed in the Playlist. Add basic monitoring, latency alignment and count-in before release. |
| VST3 plugins | Instruments and inserts, scanner/database, saved state, parameter automation, factory/`.vstpreset` presets and latency compensation. Verified native editors on macOS; a documented generic parameter editor on Windows/Linux. The Unix process bridge remains experimental unless its callback behavior is repaired and validated. |
| Browser and workspace | Bundled/user folders, search, sample preview/load/drag, current-project tree; toolbar, editor windows, Channel Settings, hints/help, scaling, detach/maximize/arrange/custom layouts, core shortcuts and shortcut remapping. |
| Export and interchange | Stereo WAV at selectable sample rate and 16/24-bit PCM or 32-bit float, song/pattern/time-selection export, MIDI export, new optional integer-output dither and per-mixer-track WAV stems. Existing lossy `.flp` import ships as experimental, with a conversion warning. |
| Distribution and onboarding | Tested packages for macOS Apple Silicon/Intel, Windows x86-64 and Linux x86-64; installation/open-file handling, quick-start manual, release notes, bundled demo, license notices and an explicit platform/plugin compatibility list. ARM64 Windows/Linux remain experimental build targets. |

“Supported” means a tested platform/version/device combination published with the release. The exact minimum OS
versions, Linux package/distribution coverage and tested plugin list must be established in phase 1 and frozen
before beta. A CI matrix alone does not define a support promise. Choose and document one primary Windows package
and one Linux package format during packaging; multiple installer formats are not a separate v0.1 requirement.

The portable bundle, recording monitoring/alignment/count-in, remappable shortcuts, dither and mixer stems are
**proposed additions**, not claims about current main. Clip-offset editing and control-value entry also need finishing.
Everything already included still has to pass release gates; a VERIFIED inventory label is not a waiver.

## Work plan and AI-assisted estimates

An engineering day is one full developer workday, including AI prompting/runs, reading generated changes,
integration, tests and review. The estimates already assume AI assistance; do not apply another “AI speedup” factor.
Phases are sequential for a single developer. Small fixes can overlap testing, but the same person's time is counted once.

| Phase | Work and dependency order | AI-assisted engineering days | Exit condition |
| --- | --- | ---: | --- |
| 1. Establish a reliable baseline | Reconcile stale inventories/docs; run platform CI; reproduce native-plugin and recording gaps; confirm target OS/device/plugin matrix; inspect callback ownership and queue saturation; triage blockers. | 5–8 | A reproducible release checklist, named testers/devices and a bounded blocker list. Reforecast at this point. |
| 2. Make playback and projects dependable | Fix confirmed realtime, queue/teardown and save/recovery failures; stress transport/edits/plugin replacement; verify real VST3 scan/presets/state/automation/PDC and macOS editor resize/focus/occlusion. Keep unsafe or blocking bridge behavior outside supported realtime use. | 7–12 | No callback allocation/large destruction/blocking in supported paths; projects recover and reopen; plugin acceptance matrix passes. |
| 3. Finish recording | Validate hardware MIDI and microphone/interface input; handle permissions, device changes and dropped frames; finish input selection/monitoring, latency compensation and count-in, then undo/save/reopen recorded takes. | 6–10 | Repeatable MIDI/audio recording on each supported OS, with measured alignment and visible overrun/error reporting. |
| 4. Finish daily workflows | Portable project bundles; remappable shortcuts; clip-offset/control-value editing; optional dither and mixer stems. Confirm packaging of recorded assets and missing-sample/plugin handling. | 5–8 | A user can move a project to a clean machine, reopen it, edit and export the intended mix/stems. |
| 5. Package and validate platforms | Production signing/notarization where credentials permit; Windows/Linux packages; file associations/open-file events; bundled resources/licenses; installation and GUI/audio/plugin checks on target hardware. | 6–10 | Clean-machine install, launch, save, reopen, record and export work for every supported target. |
| 6. Beta and release candidate | External users complete songs; fix high-impact failures; long-session/performance checks; review UI images after changes; complete quick start, compatibility notes and release notes; freeze and retest packages. | 10–15 | Two-week beta observation window completed and the final release gates below pass. |
| **Total active work** | Includes implementation, review, tests, packaging and beta fixes. | **39–63 days** | About **8–13 full-time working weeks**, before contingency and external waiting. |

Use **12–20 elapsed weeks** as the working forecast: 8–13 active weeks plus roughly 25–40% uncertainty and calendar
slack for testers, devices, signing and release coordination. These allowances overlap rather than all adding at
maximum. The two-week beta window is included in phase 6, not an additional two weeks. The slower 20–26-week
scenario covers larger architectural or platform rework; it is not a guaranteed upper bound.

At half-time developer availability, expect approximately 20–34 elapsed weeks, with external waits still variable.
More AI sessions can accelerate independent implementation and test generation, but do not divide the schedule
linearly: native device tests, listening, review, integration and beta feedback remain shared bottlenecks.

AI is useful for command/model plumbing, isolated DSP changes, test fixtures, repetitive platform packaging,
documentation and regression fixes. A developer must review audio-thread ownership/FFI, listen for artifacts,
judge note-entry and recording latency, and evaluate the app in actual music sessions. Testers, target machines,
audio/MIDI hardware, permissions and signing credentials must be available alongside the AI work.

## Release gates

The app is ready when these are true, rather than when a feature count reaches a target:

1. **Finish a song:** on each supported OS, a tester starts from an empty project, programs drums, writes a melody,
   uses a VST3 instrument/effect, records MIDI and audio, arranges and automates, mixes, saves, quits, reopens and
   exports a matching WAV. Record machine, OS, device, plugin version and result.
2. **Protect work:** test cancelled/failed saves, missing samples/plugins, corrupt/newer files, interrupted
   autosave, recovery and multiple instances. A portable bundle opens on a machine without the original sample paths.
3. **Keep audio stable:** exercise multiple sample rates and small/irregular/large block sizes, device changes,
   queue saturation, plugin removal and long sessions. Prove supported callbacks avoid allocation, blocking and
   destruction of large objects. Define a reference project and measured CPU/dropout budget in phase 1.
4. **Verify recording:** use a hardware keyboard and real input on each OS; check timing/latency, monitoring,
   count-in, long takes, buffer overruns, stopping/cancelling and saved media references. Compare alignment against
   a measured loopback, not just visual clip placement.
5. **Verify plugins:** scan, load, play, automate, save/reopen, export, resize and use keyboard focus with a small
   representative real-plugin set. Exercise the documented parameter fallback on Windows/Linux. No blanket
   compatibility or crash-isolation claim; disclose in-process crash risk and bridge restrictions.
6. **Verify installation and export:** fresh-machine installs include sounds/resources; `.lime` opening works;
   signatures/notarization meet the published installation policy. Read WAV/stems and MIDI in an independent
   application; compare renders and effect tails. All workspace build/test/Clippy/fmt/inventory checks pass.
7. **Complete beta:** no known reproducible data-loss/crash blocker or failure of the central compose/record/export
   workflow remains. Lesser issues have a workaround and a release-note entry. The manual, limits, support matrix,
   licenses and feature evidence match the actual packaged release.

Two concrete engineering risks deserve early attention: main's snapshot swap ignores failure when returning an
old snapshot to a full garbage ring (`crates/limeshot-audio/src/lib.rs`), which can destroy it in the callback;
and the Unix bridge uses synchronous socket I/O with a 500 ms audio timeout (D34). These are inspection findings,
not claims of reproduced audible failures. Investigate and fix or explicitly exclude the affected mode from
supported realtime operation before release.

## After v0.1

These are dependency-ordered planning bands for the same developer/AI setup, **not extra work hidden in the v0.1
estimate**. Re-estimate after beta; advanced DSP and platform hosting have high uncertainty.

| Follow-up | Likely scope | Additional active effort |
| --- | --- | ---: |
| Production depth | Tempo map/automation, larger automation editing, track groups, consolidation, fuller recording/take management, controller mapping and additional export formats. | 6–10 engineering weeks |
| Native plugin coverage | Native editors on Windows/Linux; repair/extend bridge transport and editor support; CLAP, then macOS AU hosting. Platform/format tasks can ship separately. | 8–14 engineering weeks |
| Advanced composition and audio | Time stretching/crossfades, richer sampler/synth features, per-note expression, multi-arrangements, chord/Riff tools, analysers and broader accessibility. | 12–24+ engineering weeks |
| Separate Lime Native plugins | Package shared instruments/effects as standalone plugins; independent host validation, installers/signing and compatibility releases. | 3–6 engineering weeks from this main baseline, subject to reviewing off-main work |

The standalone Lime Native suite is not present at the pinned main commit, even if work exists on another branch
or in the current checkout. It is a separate deliverable, not a prerequisite for the DAW release. Review and
re-estimate it after merge; do not assume unpublished work is either absent progress or validated release content.

Cloud services, stem separation, video export, surround and full FL Studio parity have no date commitment.
The remaining inventory is a backlog, not a promise to implement all 272 records before release.

## Keeping the plan honest

This document sets proposed product boundaries; [features.json](features.json) remains canonical for implementation
status. The companion checklist deliberately preserves baseline statuses and spells out narrower behavior where a
feature title is broader than its implementation. Do not promote statuses simply because a feature is scheduled.

After phase 1, replace ranges with estimates based on reproduced defects, target machines and actual throughput.
At scope freeze, resolve the cross-record inconsistencies (for example AUT-009 versus PLG-007, and the CLAP
prerequisites on generic manager/browser records), update acceptance criteria, and regenerate the inventories.
Any added v0.1 feature must identify what work or schedule allowance it consumes. Keep release limitations and
verification evidence current as code lands.

Sources: the pinned main versions of [feature inventory](features.json), [progress](../development/progress.md),
[next steps](../development/next-steps.md), [limitations](known-limitations.md), [MVP](mvp.md),
[engine architecture](../architecture/audio-engine.md), [project model](../architecture/project-model.md),
[UI architecture](../architecture/ui.md) and [decisions](../development/decisions.md). Relative links open the current
checkout; use the pinned commit above when reproducing this baseline. No fresh build, CI run, hardware test or
benchmark was performed to produce this documentation forecast.
