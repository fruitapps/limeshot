# Progress log

Dated verification evidence: what was built, what was checked and how, and what stayed unverified. Feature
statuses live in [features.json](../roadmap/features.json); `notes` there refer to the session numbers below.
Newest entries are at the end. Commit-specific CI evidence is in [platform-validation.md](platform-validation.md).

## Manual checks (repeat after UI changes)

1. `cargo run --release -p limeshot-app -- --demo`, press Space: drums play; L switches to Song mode and the Playlist
   playhead moves; the Mixer meters move (F9).
2. Channel Rack: click/drag steps on and off; right-drag erases; change tempo while playing.
3. F7: draw, move and resize notes; drag in the velocity lane; click keys to audition.
4. Playlist: place, move, resize and delete clips; ▾ › Make unique; double-click opens the pattern.
5. Browser: click a sample (preview); drag it onto the Channel Rack (new channel) or the Playlist (audio clip).
6. Cmd+S, quit, reopen; File › Export WAV; play the file in another player.

Use `cargo run -p limeshot-app --example live_check -- /tmp/check.lime` for live checks: it disables shared
autosave/recovery and does not save preferences. On the development Mac (QWERTZ layout, Z is keycode 16), drive
apps with CoreGraphics events, not AppleScript "System Events". FL Studio ignores quick clicks inside its editors:
hold the button about 0.3 s.

## 2026-10-20

**Session 1 — research, architecture, MVP slice.** Workspace crates, `.lime` format, DSP, sequencer, engine, cpal
output, egui UI, CLI and bundled samples; 217-feature inventory. Checked: all tests, Clippy, fmt; demo render and
re-render bit-identical (15.70 s, −2.4 dBFS); Rosetta x86-64 render identical; Windows x64/ARM64 `cargo check`;
realtime `limeshot-cli play` (563 callbacks, 1.1 % load, correct position at 124 BPM).

**Session 2 — FL observation, editing depth, insert effects.** Swing, quantize, copy/paste, insert effects with
real-time-safe hand-off and export tails. A new UI test found and fixed drag-painting switching to erase. 92 tests;
realtime with effects at 5.9 % load.

**Session 3 — FL's window model (D24).** Workspace windows, maximise, detach to OS windows; mixer routing with
feedback refusal (MIX-008/013). Fixed: viewing the Mixer committed an edit through slider rounding (D25). Live
capture of detached Mixer. 99 tests.

**Session 4 — real-input pass.** Observed live in FL: steps toggle with both buttons, drag paints the first step's
state (D26); lengthened pattern clips are silent past the pattern end (D14); first note is two steps. Real-input pass
over windows, steps, undo, transport, Piano Roll, Playlist and Mixer. Fixed: transparent Mixer/Playlist bodies
(`floating_window_frames_cover_their_body`). 101 tests.

**Session 5 — MVP core.** Audio clips (PL-014, BR-010, SEQ-019), automation clips (AUT-001…003, PL-015), MIDI files
(PR-020, EXP-012), LimeSynth filter/LFO (INS-006), autosave and recovery (APP-016, APP-024, D30), keybinding and
settings inventories. Live: Browser drop onto the Playlist, Song playback, automation clip from a fader. MIDI round
trip of the demo (130 notes). 123 tests.

## 2026-10-21

**Session 6 — `.flp` import (APP-022).** Tempo, signature, channels, samples, levels, inserts, patterns, notes and
the first arrangement. Fixtures saved by FL 26.1.1 plus six synthetic files; all 164 shipped FL projects import.
Format findings in `docs/research/flp-format.md`. 136 tests.

**Session 7 — MVP area by area.** Each area observed in FL first. Channel selection, move/delete with confirmation,
F2 (SEQ-007/008/020, BR-001…004); Piano Roll Shift+arrow transpose/nudge, Cmd+wheel zoom fixed (PR-002, PR-004…007,
PR-009…011); Playlist keys, Mixer and app (PL-003/006/008/009, MIX-005, APP-001/014, TR-005/006; APP-012 by manual
checks). All MVP features except PF-002…004 VERIFIED. Then Piano Roll tools (PR-012/023/024) and Playlist time
selection and markers (PL-016).

**Session 8 — Playlist tools, EQ, sidechain, fill, graph editor, groups.** PL-017 (slip, slice), MIX-010 (track EQ),
MIX-009 (sidechain, gain reduction), SEQ-014 (fill, rotate), SEQ-011 (graph editor, note pan), SEQ-012 (filter
groups). Engine sidechain test ducks by more than 10 dB.

**Session 9 — owner's 16 hands-on issues.** Window restore size, in-place Mixer names, send knob travel, double-click
resets, inserts default to 0 dB, rack/Mixer minimum sizes, note-resize clamping, grid follows snap, Opt+wheel
vertical zoom, Playback-tool scrubbing, tool cursors, pattern colours, CPU readout. All covered by UI tests; Alt+wheel
in FL not observed live.

**Session 10 — VST3 hosting (D32).** `limeshot-plugins`: module loading, class listing, component/controller split,
real-time processor, compiled-in Lime Test Gain/Tone fixtures. Plugins in projects, sample-accurate plugin notes
(block sizes 1/64/333/1024), scanning in child processes. `vst3_probe` loaded all 36 bundles on the dev Mac. Demo with
Surge XT, OTT and Vital renders and plays in realtime (about 25 % load).

**Session 11 — owner's 19 items.** Audio tab in the clip picker, Shift+drag copies, compound undo gestures, 1/3-step
snap, plugin Channel Settings width (checked live with Vital), auditions always end (note-off after note-on, tested at
1/64/512), graph editor overlay, Stop silences everything, Browser preview panel, tool cursors, window raising and
pinning, hint panel, knob detents.

**Session 12 — plugin windows (PLG-006, D33).** FL's plugin windows observed. Native editor hosting in a container
`NSView`, Plugin/Settings tabs, toggling from channel button and insert name. Manual verification (release build):
Vital, Serum 2, Surge XT and OTT editors open, switch tabs, resize, mark the project modified and close/reopen.

## 2026-10-22

**Sessions 13–14 — FL menus and dialogs documented.** `menus.json` (main menus), 31 new feature records,
`settings.json` grown to 188 records from FL's settings window and menu dialogs. No code changes.

**Session 15 — plugin extras and menus.** Factory programs and `.vstpreset` (PLG-008), keyboard forwarding, window
places saved in the project; plugin parameter automation (PLG-007; engine test at 64/333/1024 and through Lime Test
Gain); delay compensation (PLG-009, MIX-012; four latent paths align at frame 24 000 at three block sizes); crash
isolation through a bridge process (PLG-010, D34; tested with Vital, Surge XT, OTT, Elephant). Patterns, Edit, Tools
and View › Layout menus; FL's settings window (APP-025); toolbar and editor menus documented (725 menu records).

**Session 16 — editor menus and production features.** Piano Roll menu and tool dialogs (PR-015/014/023), stamps and
scale highlighting (PR-027/013), Playlist menu (PL-006/008/013/016/022), Mixer track menu (MIX-006/007), Channel Rack
options (SEQ-007/003), delay tempo sync (FX-008), pre-fader sends (MIX-008), presets (FX-014, INS-010), automated
controls follow the song (AUT-013), point modes (AUT-012), MIDI input (WF-003, APP-020; virtual CoreMIDI port test),
note recording (REC-002, PR-021), audio recording (REC-001/003; not tried with a real input), macOS packaging, clip
fades (PL-023), MP3/FLAC/Ogg/AIFF (INS-008, D37), Gate, Drive, Chorus (FX-006/009/010), sampler loop/filter/envelope
(INS-007/014; envelope at 1/64/333/1024), export of a time selection (EXP-010), tempo automation (AUT-008), LimeSynth
filter modes and key tracking (INS-013).

**Session 17 — owner interaction fixes.** Audio-clock note deadlines for drawn notes (1/64/333/1024), deselect and
delete gestures, rack drag selection, Mixer strip selection, waveform ghosts on drops, Shift-click mute locks, track
header menus, window raising, typing keyboard, automation point cursors, internal plugin wrappers (D35). 111 UI tests.
Native embedding not verified live: the host lacked Accessibility. CI fixes: a wall-clock assumption in
`playlist_time_selection_loops_song_playback` and a path-separator comparison on Windows; jobs run on CI again.

**Further 2026-10-22 work.** Control value menus (UI-010), auto scrolling (UI-016), numbered saves (APP-018), Channel
Rack Zip/Unzip and auto zip (SEQ-013, SET-089). Full CI run `39826217710` at `38d9a7b` passed all seven jobs, including
`macos-15-intel`. 342 tests (132 UI).

**Lime Native VST3 suite (PLG-012, D38).** `native-plugins/`: nine standalone bundles, then twelve, with original
editors (`docs/research/lime-native-ui.md`). Nine bundles passed the exported-binary verifier at 44.1/48/96 kHz and
buffers 1/64/333/1024 and pluginval 1.0.4 strictness 5. Details: [VALIDATION.md](../../native-plugins/VALIDATION.md).

## 2026-10-23 — LimeShot hosts the Lime VST3s (PLG-013, D39)

LimeShot's instruments and effects run as hosted Lime VST3s; the in-process versions are gone. Older projects open
with portable `lime:` references. Checked:

* A project with LimeSynth (band-pass, key tracking), Sampler (WAV, pitch, trim) and EQ renders with exact sample
  bits through the real bundles against the test-only reference processors.
* Hosted parity (PLG-017…020, PLG-022, PLG-024…029): Playlist clips in Lime Sampler, note pan, exact track EQ,
  Gate/Drive/Chorus, Comp key buses, source waveforms, unit-value menus, dynamics meters, voice statistics. Real
  hosted acceptance tests, including local versus bridged audio.
* Synced Delay at 90/180 BPM and under tempo automation at block sizes 1/64/333/1024; sidechain indices follow mixer
  edits; 3,840-note ramp onsets at four block sizes.
* Live, macOS: the host went black after a native editor attached. Fixed with a host graphics-context guard; Lime
  Sampler and the host then render together, including a switch to Settings (PLG-006, PLG-023).
* Packaged demo render: 741 207 frames at 44.1 kHz, −3.7 dBFS. Realtime: 324 callbacks in 2 s, −5.3 dBFS.
* Twelve release and debug bundles pass the binary verifier at three rates and four buffer sizes; signatures verify.

Piano Roll Target control (PR-016) and keyboard/note label views (PR-030) also landed:
[research](../research/piano-roll-view-controls.md). 399 tests (141 UI).

## 2026-10-25

**Plugin windows, imports, mute locks, zoom (PR #11).** Wrapper sizing, workspace title controls, AppKit preset
menus, Browser/OS sample drops, compressed decoding in the native sampler, Mixer mute locks, 0.25–16 384 px/beat zoom.
egui-baseview teardown patch (`native-plugins/vendor/egui-baseview/LIMESHOT-PATCH.md`) with an offscreen GL
regression. Live pass with permissions granted: [live-plugin-checks-2026-10-25.md](live-plugin-checks-2026-10-25.md).
A modal re-entrancy panic in egui-baseview was found and fixed. Recovery copies are no longer overwritten or deleted
on normal quit.

**Sampler, mixer and editor workflow (PR #12).** Native drag passthrough, Browser double-click previews, ruler range
selection and Cmd+B duplication, plugin buttons toggle/raise, audio clip scrubbing, OS drops at the pointer, sampler
bound raised to 6 million stereo frames, On + Mix insert slots with drag reordering, plugin window opens on add.
Pattern mode ignores loop ranges (`pattern_ignores_time_selection_at_any_block_size`). 420 host tests, 19 native.

**Windows (PR #13).** Portable class ids (`util::tuid_to_hex`), editors in owned popups clipped around covering
windows, editor timers serviced each frame, Explorer drop positions, no console window. Confirmed by the owner on
Windows: Lime and third-party editors appear and respond.

**Installers (PR #14, PF-006, D40).** Windows installer and portable ZIP verified on Windows 11 x64 in shared and
private plugin modes; macOS app, DMG and PKG; Linux tarball and `.deb`. 428 root tests (156 UI), 19 native tests.

**CI and release audit (PR #15).** Bounded waits for the asynchronous rack-drop test, strong VST3 module retention
(D41), Wayland headers for native-plugin CI, committed slider values through the parameter queue (100/100 stress runs).
Full CI passes on Linux x86-64/ARM64, Windows and Intel macOS at `d6f3d17`; native-plugin CI passes on all three OSes
(`40215397199`). Release run `40211718081`: all five packaging jobs pass; downloaded assets match `SHA256SUMS.txt`,
both DMGs pass `hdiutil verify`, ZIP CRCs pass, Linux tarballs contain twelve VST3s and their installers.

**v0.1.0-alpha.1 published** (prerelease, 2026-10-25) from `28d3a41`: CI `40219529679`, native-plugin CI
`40219529749` and release run `40219554203` pass. Windows x64, macOS Apple Silicon and Intel, Linux x86-64 and ARM64
packages with checksums.


## 2026-10-26

**Alpha.1 interaction and long-audio follow-up.** Native controls retain local drag precision and accept direct
value entry; plugin resize echoes are coalesced and self-resizing editors retain their current natural size.
Browser trees constrain labels, recursive search has a temporary Search tab, sample roots live under Files,
and preview has a Stop control. Playlist scrollbars, triangular fade length/circular tension handles and
selected-clip fade gestures are implemented. Untouched pattern clips follow note-end length; explicit resize
preserves arrangement length and undo groups the automatic update. Pin uses a painter pushpin.

Keep on disk and Auto keep long audio on disk follow the documented FL sampler behavior with an independent
implementation: 2,000,000,000 decoded stereo-float bytes, 1 MiB page storage per reader, worker decoding and
waveform extrema, small linked project state and relative source paths. Compressed sources prepare to seekable
WAV in the temporary cache. Realtime page misses return silence and are counted; offline exports wait. A hosted
seek regression caught nice-plug 0.4.2 storing its previous activation mode; a documented ISC vendor patch fixes
that setup ordering. Song-end plugin releases now retain their absolute frame rather than cutting the final
buffer short; the hosted seek test checks its final sample across four block sizes. See [research](../research/alpha1-sampler-and-fades.md) for sources and limitations.

Performance review removed per-block editor wakeups, limits idle host polling to 5 Hz, bounds Browser repaint
rates, virtualizes search results and processes only the occupied native voice prefix while preserving all 128
voice slots. The CPU meter measures audio render time / buffer deadline, not total process CPU.
A ten-minute 230.4 MB float-WAV probe measured 8265 ms cold preparation, 1.70 ms warm open, 1 MiB pages and
225 KB waveform extrema; warm standalone process RSS was 3.38 MB. These are reader measurements, not a whole-DAW
CPU benchmark.

Manual macOS arm64 verification with granted Screen Recording/Accessibility: the disposable live_check app
rendered LimeSynth, accepted a clicked octave value of -2, and Serum 2 shrank and grew from its own corner with
its wrapper aligned and top-left position stable. A ten-minute disk-backed Browser preview loaded, displayed
600 seconds, retained Browser width with a truncated filename, and stopped to zero voices. Offscreen Playlist render inspected for fades/scrollbars/Files.
Workspace suite: 440 passed (159 UI, 10 hosted VST3); native unit suite: 22 passed; `nice-plug` unit suite: 8 passed.
Workspace/native strict Clippy, builds, formatting and inventory/reference checks passed. Windows/Linux runtime and broad third-party editor coverage remain unverified.

Owner update (2026-10-26): create and use branches in the main checkout at `~/Developer/limeshot`, and build/test
there to retain Cargo caches. Do not use separate worktrees or clones for routine branch work. Preserve local
changes when switching branches. This supersedes the earlier separate-worktree session policy; repeat it in handoffs.


## Hostile-input audit (2026-10-26)

New `crates/limeshot-app/tests/hostile_input.rs`: every numeric field of the demo project is replaced with extreme
values (0, -1, u32::MAX, u64::MAX, i64::MIN, ±1e30) and loaded; a random quarter are also compiled and rendered.
FL Studio and MIDI fixtures are byte-corrupted, truncated and given 0xFF runs (deterministic PRNG). It found real
bugs: `u64::MAX` note/clip/marker ticks overflowed tick arithmetic (panic in debug, silent wrap in release).
`Project::sanitize` now clamps all positions and lengths to `MAX_TICK` (about a million 4/4 bars), with a unit test.
Result: `cargo test --workspace`, clippy and fmt pass on macOS. Not yet covered: audio file decoders (WAV/MP3/FLAC)
on corrupt input, plugin state blobs, automation point ticks beyond clamping, and UI-level edge cases.
