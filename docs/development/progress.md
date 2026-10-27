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


## 2026-10-27 — Windows CI event-loop registration race

CI `40463705283` at main merge `586443f` aborted in Windows UI tests at nice-plug's
`RegisterClassExA` assertion (`windows.rs:94`), while the other six jobs passed. The framework used
`QueryPerformanceCounter` alone to name window classes, so concurrent plugin creation could register
the same class twice. Names now combine a DLL-local atomic sequence with that static's address,
separating concurrent instances and framework copies in different DLLs. No test serialization is needed.

Added `concurrent_event_loops_register_distinct_window_classes`: eight threads create/destroy 512
hidden event-loop windows and check name uniqueness and GUI task dispatch. Native-plugin CI now runs
the patched framework's unit suite on Windows, Linux and macOS. Feature statuses remain IMPLEMENTED;
headless checks do not verify live GUI/audio behavior.

Local validation on macOS Apple Silicon: rebuilt all twelve debug VST3 bundles; native-plugin suite
22 passed, vendor suite 8 passed, workspace suite 443 passed; Windows x86-64 vendor test cross-compilation passed.
Workspace tests ran with local-socket permission for the plugin bridge after the sandbox denied binding. Workspace and
native strict Clippy, both formatting checks and inventory/repository checks passed.

Work stays in the existing primary checkout (`~/Developer/limeshot`) on `codex/fix-merge-ci`, using
its Cargo caches. Continue branch work and validation there; do not create a routine worktree or clone.


## 2026-10-27 — Corrupt audio import regression coverage

Added `crates/limeshot-dsp/tests/hostile_audio.rs`: 2,400 deterministic byte mutations,
truncations, 0xFF runs and header-focused edits across 16/24-bit PCM WAV, float WAV, FLAC, AIFF
and generated silent MP3. Every original must decode; successful mutations must stay within
the requested frame limit and produce finite, complete stereo frames with a positive sample rate.
A separate float-WAV test checks that NaN and infinities become silence.

The corpus exposed a Symphonia 0.5.5 panic in format probing when a corrupt AIFF sample rate
produces a zero TimeBase. The loader now contains an unwinding third-party probe panic and
returns `SampleError::Decode`; a direct zero-rate AIFF regression covers the failure. The
containment surrounds format probing only and does not change audio-thread processing or
skipped-frame behavior. It does not establish exhaustive robustness or bound allocations
inside third-party codecs. INS-001/INS-008 evidence and remaining coverage gaps are updated.

Local validation on macOS Apple Silicon: the three new decoder tests pass in debug and release
profiles; the native unit suite passes all 22 tests. `cargo test --workspace` passes, including
159 UI tests and 10 hosted VST3 tests. The workspace run required local-socket permission for
the plugin process bridge after the sandbox refused its listener. `cargo build --workspace`,
strict workspace Clippy, formatting and inventory/repository checks pass. All twelve release
VST3 bundles rebuilt successfully; the existing hosted integration-test executable was then
rerun against those bundles and all 10 tests passed. Live GUI/audio and other platforms were
not exercised by this change.

Work uses branch `codex/hostile-audio-decoders` in the existing primary checkout
(`~/Developer/limeshot`) with its Cargo caches. Continue there; do not create a routine
worktree or clone.


## 2026-10-27 — Plugin-state corruption and decoded size boundaries

Continued the hostile-input audit on `codex/hostile-audio-decoders` with plugin-state tests.
The base64 guard previously treated padding as decoded bytes, rejecting a valid state exactly
at the documented 64 MiB limit. It now counts non-whitespace quartets before allocating,
allows the final padded quartet, and checks the exact decoded size before retaining the state.
Small-limit tests cover every padding alignment, whitespace and one/two-byte overruns without
allocating a 64 MiB fixture. Invalid base64 still restores defaults while retaining plugin
identity. Host state-container tests reject every truncation and extreme component/controller
lengths, while preserving opaque payload bytes for the plugin reader.

These checks cover the host container and serialized text; they do not establish robustness
of arbitrary third-party plugin state readers. PLG-005 evidence and remaining test gaps are
updated. Work remains in the primary checkout (`~/Developer/limeshot`) with its Cargo caches;
continue branch work there without a routine worktree or clone.

Local validation on macOS Apple Silicon: rebuilt all twelve debug VST3 bundles; full workspace
tests pass with `LIMESHOT_LIME_PLUGINS` pointing at `native-plugins/target/debug-bundled`, including
159 UI tests and 10 hosted VST3 tests. Local-socket permission was enabled for the plugin bridge.
Native unit suite: 22 passed. Workspace build, strict Clippy, formatting, inventory and repository
checks pass. Live GUI/audio and other-platform runtime verification remain open.


## 2026-10-27 — Separate agent development from human verification

`next-steps.md` now holds the agent work queue and a Development entry directing agents to
pursue FL Studio feature/workflow parity and implement new features by priority and dependencies.
Checks needing human input, hardware, permissions, credentials or owner decisions are recorded
in the new [human verification queue](human-verification.md), with prerequisites and partial
evidence, while independent development continues. Implementation and verification statuses
remain distinct. The existing native/plugin/platform/recording checks, FL observations and
Piano Roll owner decision are retained in that queue.

Updated AGENTS.md, CONTRIBUTING.md, README.md, the project principles and known limitations
to distinguish agent work from hands-on verification. Branch/PR delivery remains in the primary
checkout (`~/Developer/limeshot`) using its Cargo caches, without routine worktrees/clones,
direct pushes to main or automatic merges.

Documentation-only validation: inventory checks, repository references (195 links, zero errors),
and whitespace checks pass. No Rust behavior changed; no rebuild or runtime tests were needed.


## 2026-10-27 — PR #20 CI hang-test synchronization and review wording

Linux ARM64 CI job `122120738692` (run `40483780318`, head `fcc77f7`) failed
`a_hung_bridge_times_out` because the simulated hung bridge returned nonzero audio. Sending
SIGSTOP is asynchronous: the audio thread can process a request before the process finishes
stopping. The test now waits for the child stop event using bounded `waitpid(WUNTRACED |
WNOHANG)` polling before starting the timeout measurement. It also refills the later block
with nonzero input and verifies that it stays silent after the timeout. `libc` is a Unix-only
test dependency; production bridge processing and timeout behavior are unchanged.

Addressed CodeRabbit's next-steps wording: agents must explicitly run the available automated
tests and local checks, in addition to implementing complete workflows.

Work continues on `codex/hostile-audio-decoders` in the primary checkout
(`~/Developer/limeshot`), reusing its Cargo caches; no worktree/clone or direct main push.

Local validation on macOS Apple Silicon: all five bridge integration tests pass against the
debug VST3 bundles; the hang regression passes 100/100 runs with eight concurrent workers.
Tests used local-socket/child-process permission. Workspace strict Clippy, formatting, inventory
and repository checks pass. This is a test/documentation change; native bundles and production
code were unchanged. Linux ARM64 validation awaits the updated CI run.


## 2026-10-27 — Input monitoring and recording count-in

Implemented REC-003 monitoring in Audio settings and the Record menu: Off / When armed / On,
with an explicit Mixer destination by position. When armed requires Record + Audio and includes
stopped transport. One input stream feeds raw-WAV recording and a bounded stereo monitor ring;
output performs linear rate conversion into preallocated scratch and mixes before the destination's
inserts/EQ/fader/routes. Muting/rerouting invalidates queued input; missing frames are silent and
overflow recovery discards backlog. Replaced rings are destroyed on the control thread. Capture
errors report once and need explicit retry/configuration change. Starting/ending a take reopens
an idle monitoring stream; monitoring policy changes during a take preserve its writer.

REC-007 adds toolbar 3.2.1, Options > Recording precount and Cmd/Ctrl+P, with a one/two-bar context
menu. The renderer schedules accented clicks at the starting tempo/signature, freezes the program
position and splits a block at the exact lead-in boundary. Stop/pause/locate cancel it; loops do not
repeat it. MIDI/typing notes during the lead-in are auditioned without recording. Raw input is
prepared during the lead-in and gated by the output-transport atomic. Independent device callbacks
and UI input startup still impose alignment uncertainty; this is not REC-006 latency compensation.
Monitoring and count-in are session-local, default Off, and do not edit the project. Physical-input
routing remains the existing single device/first stereo pair, rather than per-track hardware I/O.
[Research, behavior and limits](../research/recording-monitoring-and-count-in.md).

Local macOS Apple Silicon validation: rebuilt all twelve debug VST3 bundles; workspace suite
457 passed, including 161 UI and 10 hosted VST3 tests; native suite 22 passed. New tests cover
monitor rate conversion across block sizes, overflow/underflow/stale frames, raw capture gating,
non-finite/mono/multichannel input, Mixer inserts/fader/pan/mute, sample-exact count-in at fractional
BPM in 4/4 and 7/8, cancellation, and actual settings/menu/shortcut/bar controls. Workspace build,
strict Clippy, formatting and inventory/repository checks pass. The final readout adjustment was
also checked by the count-in UI test and an inspected offscreen render. Offscreen renders required
host GPU access; plugin bridge tests used local-socket/child-process permission.

Live `live_check` on macOS 26.6.2 arm64: a temporary app bundle supplied the microphone usage
string; preferences/autosave remained disabled. The main app was not used. Output opened AirPods
Pro at 48 kHz; the system-default input produced 24 kHz stereo float WAV. With insert 1 muted,
monitoring On worked through the idle/record/stop lifecycle. Cmd+P enabled count-in; the live
readout showed remaining beats. Stop produced a 15.14 s take (363,360 frames, finite/nonzero PCM,
peak 0.08263), placed as one clip at tick 0 in a saved disposable `/tmp` project. Monitoring was
then set Off and both test processes were closed. No microphone prompt appeared. Accessibility
reads temporarily timed out on settings, but a process sample and screenshots showed the app
still processing frames, and subsequent settings interactions succeeded. This does not establish
audible monitoring quality, measured latency, long-run clock drift/device-loss behavior, every
mode transition, or other-platform runtime behavior. Feature statuses stay IMPLEMENTED.

Work uses `codex/input-monitoring` in the primary checkout (`~/Developer/limeshot`), preserving
its Cargo caches. Continue there without routine worktrees/clones; push only the feature branch
and its PR, never main, and do not merge automatically.


## 2026-10-27 — PR #21 count-in UI synchronization

macOS Intel CI job `122159946662` (run `40495547564`, head `ba1e83a`) failed the
count-in UI test's `COUNT IN` accessibility-label assertion. The audio thread can publish
count-in status after the polling helper's last UI frame, so observing the atomic does not
establish that the corresponding label has been drawn. The test now draws a fresh frame
after observing startup and uses 60 BPM to keep the lead-in active on busy runners; it still
stops immediately rather than waiting for the lead-in. Production playback is unchanged.
CodeRabbit's preceding documentation correction is retained.

Local macOS Apple Silicon validation: regression 100/100 runs with eight concurrent workers;
all 161 UI tests pass with debug Lime bundles and local-socket permission for the plugin bridge.
The initial sandboxed suite could not open the bridge listener; the permitted rerun passes.
Strict Clippy for the UI test target, formatting, inventory/reference and whitespace checks pass.
Intel CI verification awaits the pushed commit. Feature/verification statuses are unchanged.
Work remains on `codex/input-monitoring` in the primary checkout (`~/Developer/limeshot`),
reusing Cargo caches without a routine worktree/clone, direct main push or automatic merge.


## 2026-10-27 — MP3 export and sampler parity/integration audit

Implemented EXP-005: File > Export MP3… and Cmd/Ctrl+Shift+R open the shared Export audio dialog
with MP3 selected. Stereo MPEG-1 CBR supports 32/44.1/48 kHz and all 14 standard bitrates from
32 to 320 kbps (default 320). GUI and CLI share the existing hosted render, range, PDC and tail
path; CLI selects `.wav`/`.mp3` by suffix and accepts `--bitrate`. Pinned `rusty_mp3` 1.0.5 is
Apache-2.0, pure Rust, with no runtime dependencies/C encoder installation. MP3 validates settings
before render, sanitizes/clips PCM, finishes the gapless tag, and writes through a synced temporary
file with atomic replacement. The GUI rejects a mismatched filename suffix rather than rewriting
a name after the save panel confirmed overwriting a different path. WAV entry points remain.
[Design, licensing, tests and limitations](../research/mp3-export.md).

Automated coverage: independent Symphonia decode at all 42 rate/bitrate combinations, stereo tone
separation, header parameters/CBR size, partial-frame/short/silent/non-finite inputs, invalid-setting
preservation and file-error cleanup. Engine tests compare WAV/MP3 Song/Pattern/selection reports
and release tails; CLI renders the hosted demo and checks invalid rates/bitrates/extensions; UI
tests exercise the MP3 shortcut and format/rate switching. The
[dialog screenshot](../screenshots/current/export-mp3.png) was generated with `ui_shot --dialog=mp3`
and visually inspected. Offscreen rendering required host GPU access after the sandbox found no adapter.

A separate system-decoder check exported the demo at 48 kHz/192 kbps: `afinfo` recognized stereo,
191,999 bits/s, and 806,202 valid frames plus priming/remainder. Permitted AudioToolbox `afconvert`
decoded exactly 806,202 stereo PCM frames (matching a fresh WAV export), peak 0.64877; sample-difference
SNR against 16-bit WAV was 26.50 dB. The first sandboxed conversion failed; the permitted conversion
succeeded. This one fixture is not listening-quality certification. LimeShot's own loader retains
codec padding with default decoder options; WAV remains the aligned-loop format. CBR only, no MP3
metadata/VBR/multiple simultaneous formats; offline PCM/encoder memory and other-platform listening/runtime
checks remain open. EXP-005 is IMPLEMENTED, not promoted to full verification.

The owner also requested a sampler feature and integration comparison. The
[sampler parity audit](../research/sampler-parity-audit.md) compares documented FL controls against
Lime code and tests, and proposes staged fixed-duration preparation then realtime stretching.
New records INS-017…INS-024 track precomputed processing, metadata/navigation, declick, modulation,
host/native synchronization, cut groups, interpolation quality and event arp/echo. Existing records
now link the audit; INS-014 corrects the old claim about a pan envelope (FL documents pan LFOs).
Source-confirmed integration gaps include root-key state/display divergence, native edits without
document undo, shared sampler/audio-clip channel state and defaulted advanced FLP sampler settings.
Host source replacement/clear metadata needs live checking. The sampler-loss helper's comment now
clarifies that preserving every former LimeShot field does not mean complete FL sampler parity.

No FL source/assets were used. Current official manuals and the existing observed menu inventory
supply the reference. Two attempts to select installed FL Studio through computer use timed out;
the owner authorized Accessibility/Screen Recording, so no missing permission is asserted. No fresh
live FL A/B or standalone sampler-in-FL result is claimed. The exact outstanding checks, plus MP3
listening/player/platform checks, are in the human verification queue; limitations and next steps
are updated. Time stretching is research/design scope here, not a new implemented DSP mode.

Local validation on macOS 26.6.2 Apple Silicon: all twelve debug VST3 bundles rebuilt; native suite
22 passed. Workspace suite 463 passed, including 162 UI tests and 10 hosted VST3 tests; workspace
build, strict workspace Clippy, formatting and inventory/repository/whitespace checks pass. The first
sandboxed workspace run failed the existing plugin-bridge listener test; the run with local-socket
permission passes. Live GUI save-panel interaction, listening and other-platform runtime are unverified.

Work uses `codex/mp3-export-sampler-audit` in the existing primary checkout
(`~/Developer/limeshot`), reusing Cargo caches. Continue there without routine worktrees/clones;
push only the feature branch and its PR, never main, and do not merge automatically.


## 2026-10-27 — Sampler host and native control synchronization (INS-021)

Implemented INS-021 to keep host and native Lime Sampler controls, metadata, presets and undo
history synchronized:

* `limeshot_core::lime::sync_sampler_metadata` updates `Channel.root_key` from `value_8` and
  synchronizes `lime-sample-path`, `lime-sample-pending`, `lime-disk-unapproved` and
  `lime-external-source` with `sample-v1` across `AddChannel`, `SetInstrument`, `SetPluginState`,
  `Document::store_plugin_state`, `Project::sanitize` and `limeshot_project::load`. Explicit native
  `Clear` removes stale host sample path hints so waveform/drag/preview fallbacks do not resurrect
  cleared audio, while older embedded states without a `source` field retain their host path across
  parameter-only edits.
* Host sample imports (`browser::sampler_with_audio`, `lime::convert_project`) and `.lime`
  save/load/recovery (`sample_source`, `set_sample_source`, `absolutize_samples`) preserve and
  relocate `sample-v1.source` relative to the project folder without turning embedded PCM into disk
  mode.
* `ComponentHandler` tracks active VST3 `beginEdit`/`endEdit` parameter gestures
  (`editor_gesture_active`), and `PluginHost::poll` / `store_states` captures completed native
  Lime Sampler parameter gestures, `restartComponent` state changes (Load/Clear/Keep on disk) and
  wrapper preset loads into single undoable `Command::SetPluginState` steps without serializing PCM
  on every drag frame.
* Applying a Channel Settings sampler preset (`presets_ui::apply`) updates controls through
  `edited_values_state` and `Command::SetPluginState`, preserving embedded or disk-backed `sample-v1`
  audio and `Channel.root_key` even if the original source file was moved or removed.
* Fixed Playlist `clip_menu` so the opening pointer press does not dismiss the menu on the same pass
  (matching `point_menu`).
* Addressed PR #22 review feedback: MP3 atomic export preserves existing destination file permissions
  and respects `umask` on Unix; `mp3-export.md` labels `VERIFIED`/`OBSERVED`/`UNKNOWN` confidence;
  `audio-engine.md` and `features.json` (`PLG-022`) formatting notes are cleaned up.

Local validation on macOS Apple Silicon: workspace test suite passes (471 passed, including 163 UI
tests and 11 hosted VST3 tests across `limeshot-core`, `limeshot-plugins`, `limeshot-project`,
`crates/limeshot-ui/tests/lime_vst3.rs` and `crates/limeshot-app/tests/ui.rs`), along with strict
workspace Clippy, `cargo fmt`, `scripts/features.py check` and `scripts/check_repository.py`.
Live GUI A/B verification with FL Studio and native editor interaction remains in
`docs/development/human-verification.md`; INS-021 is marked `IMPLEMENTED`, not `VERIFIED`.

Work remains on `codex/mp3-export-sampler-audit` in the primary checkout (`~/Developer/limeshot`),
reusing Cargo caches without routine worktrees/clones, direct main pushes or automatic merges.


## 2026-10-27 — Alpha.1 Rev 1 continuation

Continued the inherited uncommitted work in `codex/alpha1-rev1-issues` in the existing primary
checkout (`~/Developer/limeshot`), reusing Cargo caches. No repository lock named Argon was found;
application-managed thread locks were left untouched. No worktree/clone was created.

Implemented mouse-held Piano Roll auditions, Playlist edge-drag scrolling and pattern-source
selection, Sample/Automation/Patterns focus controls, clip-preview scrollbar, arrangements and
track selection/activity/height/collapse/groups, plus release-to-play Browser waveform scrubbing.
The continuation fixes next-frame audition cancellation, group-parent selection, the arrangement
limit, geometry above 64 tracks and pattern selection interfering with held Channel Rack strokes.
It adds core arrangement/height/undo/persistence tests, six UI regressions, held-audition coverage
and a preview-offset/replacement audio test at block sizes 1/64/333/1024. Roadmap, menu/shortcut
inventories, architecture and limitations are updated. The requested Windows native-plugin
responsiveness investigation and FL-style wrapper/Settings research are in next steps.

macOS 26.6.2 (25G83), Apple Silicon: all 480 workspace tests pass across workspace/package runs,
including 169 UI and 62 engine tests. Workspace build, strict workspace all-target Clippy, rustfmt,
feature inventories, repository references and whitespace checks pass. All twelve native release
VST3 bundles rebuilt. The initial sandboxed plugin bridge test could not listen on its local socket;
permitted tests pass. The new preview test was corrected to account for the existing attack/release
fades. [Playlist screenshot](../screenshots/current/alpha1-rev1.png) rendered with `ui_shot --rev1`
and inspected after replacing a missing arrow glyph; offscreen rendering required host GPU access.

[Design, inherited coverage and remaining gaps](../research/alpha1-rev1.md). Stretch is shared-sampler
resampling on the right edge, changes pitch and is bounded by sampler pitch range; pitch-preserving
and independent clip stretch remain open. Track lights infer musical activity rather than metering
actual audio. STEP pulse/tension parity, disk zero crossings, full arrangement merge policies and
additional FLP arrangements remain open. Live FL/macOS comparisons and Windows/Linux runtime checks
are recorded in human verification; new features remain IMPLEMENTED rather than VERIFIED.

Push only the feature branch and its PR; do not push main or merge automatically. Continue using the
primary checkout and shared Cargo cache in handoffs.

## 2026-10-27 — Sampler ping-pong, loop gestures and filter outputs

Expanded the [FL Sampler control inventory](../research/sampler-controls-2026-10-27.md) and refreshed
the [parity audit](../research/sampler-parity-audit.md). FL 26.1.1 build 5313 was available after retry:
screenshots and keyboard input worked, but pointer input repeatedly returned `noWindowsAvailable`.
The owner opened Sample and Envelope/Instrument pages for observation. The control presence and
selected Fast LP label were observed; the filter popup was closed in the captured view. Exact audio,
endpoint, envelope and filter equivalence remains unverified. No FL code/resources were copied.

Implemented ping-pong loops, native waveform loop handles with Shift-pair movement, and independent
TPT low-/band-/high-pass outputs. Sampler controls 16/17 append stable parameters; IDs 0–15/defaults
are unchanged. Legacy sampler conversion, component states, factory/user presets, host automation,
reset and undo retain the fields. The editor has additional vertical space for the controls and
footer. The Shift gesture is in shortcut help and the inventory. INS-007 remains IN_PROGRESS for
stretch/crossfades; INS-020 is IN_PROGRESS for expanded modulation, without claiming FL filter DSP.

DSP reflects arbitrarily large loop steps in constant time without duplicate turn endpoints, and
retains bounce direction across parameter polls. Loop seek/chase folds through the same loop;
near-integer frame offsets avoid floating-point seam-side errors. Reverse wrapping stays inside
the half-open loop rather than occasionally selecting its exclusive end. Tests cover expected
turn sequences, reverse, high pitch, release, seeks, stereo coherence, old state, presets, waveform
gesture routing and geometry, filter rejection and host reset/undo. Native disk/embedded comparison
at 44.1/48/96 kHz and blocks 1/64/333/1024 runs with allocation assertions. Hosted output is compared
with the reference across those rates/blocks for all three filter modes.

macOS 26.6.2 (25G83), arm64, debug: rebuilt all twelve VST3 bundles with `build.py --debug`;
485 workspace tests and 26 native tests pass. The twelve exported bundles pass ABI/editor creation,
state, MIDI/audio and automation checks at three rates/four buffer sizes (`examples/verify`).
Workspace build, strict all-target Clippy in both workspaces, rustfmt, feature/shortcut inventories,
repository references and whitespace checks pass. Workspace tests use `LIMESHOT_LIME_PLUGINS` pointing
at `native-plugins/target/debug-bundled` and local-socket permission for bridge tests.
The [native sampler screenshot](../screenshots/current/lime-sampler-loops.png) was rendered and inspected;
offscreen rendering required host GPU access. No live Lime GUI/audio or Windows/Linux pass is claimed.

Remaining: pitch-preserving/independent stretching, seam crossfades, extended envelopes/LFOs, further
filter families/tracking, reversible processing, metadata/navigation, declick and channel voice/event
tools. Live endpoint/DSP comparisons and standalone-in-FL testing remain in human verification.

Work uses `codex/sampler-loop-filter-parity` atop `codex/alpha1-rev1-issues` (PR #23 still open at the
start of delivery), in the primary checkout `~/Developer/limeshot`, reusing Cargo caches. No routine
worktree/clone; push only the feature branch/PR, never main, and do not merge automatically.

## 2026-10-27 — Prepared sampling, arrangement crossfades and UI consistency

Implemented the fixed-duration sampling/arrangement workflow on
`codex/prepared-sampling-arrangement`. [Design, comparison and limits](../research/prepared-sampling.md).
Compared pinned independent Rust `wsola` 0.1.0 and `timestretch` 0.15.0 against synthetic
stereo tones, impulses, percussion and polyphonic fixtures before integration. Their source,
licenses, allocation, reset/flush and output-length behavior were inspected. Neither became
a product dependency: their length/priming and DJ low-bass semantics respectively do not
fit the uniform sampler contract. An independent bounded, stereo-linked Rust WSOLA path
with worker-only sinc resampling supplies exact-length preparation. Numeric evidence is
not listening-quality certification; reproducible comparison tooling and its pinned lockfile
are under `tools/stretch-eval`.

Host/native Time stretching controls offer Resample / Stretch, duration multiplier and
independent prepared pitch; the host also fits beats at the current tempo. Originals and
recipe remain in versioned portable state, reprepare starts from the original, and Restore
original works without its source file. Existing parameter IDs/defaults and old-song
resampling behavior remain. Clip preparation gives only the selected clip a separate sampler
channel, preserving shared notes/clips, controls, root and routing. Trim/loops use normalized
prepared positions; reverse, output-time slip/chase, duration, waveform, audition, fit and export
share the prepared result. Workers cancel/discard stale jobs. Embedded source/result/intermediate
and state limits, bounded preview/weak PCM caches and control-thread retirement preserve the
render-thread contract. Long disk preparation and realtime tempo following remain open.

Arrangement adds saved cubic smoothstep fades and automatic adjacent two-clip, same-track
crossfades. A persisted toggle recalculates overlaps in the edit's undo step; moving clips
apart restores manual base fades. Nested, coincident-start, muted and triple overlaps are
skipped. Clip protocol v3 retains v1/v2 semantics and the 96-byte bounded event size.

At the owner's request, reviewed UI consistency and replaced the initial ad-hoc layout:
the host now reuses the centered dialog helper, aligned two-column grid, secondary typography,
units and primary action style. Native controls sit in a sampler panel with existing flat
buttons/accent; Keep on disk is beside source information. Mode labels are Resample / Stretch,
invalid prerequisites have actionable guidance, and Restore original is disabled until useful.
[Host dialog](../screenshots/current/sampler-time-stretch.png) and
[native sampler](../screenshots/current/native/lime-sampler.png) were rendered and visually
inspected after the review. No live GUI/listening result is inferred from these PNGs.

Local evidence on macOS 26.6.2 arm64:

* Full workspace suite: **479 passed**, including 163 app UI and 12 hosted-native tests.
  The first sandboxed pass hit the existing bridge listener restriction; the permitted
  local-socket run passes. After the UI polish, the three preparation/arrangement worker
  tests were rerun successfully.
* Native workspace suite: **23 passed**, including shared immutable prepared PCM,
  native persistence, reverse/trim/loops/seek and callback allocation assertions.
* Hosted prepared/crossfaded clips save/reopen/undo, sum to the expected stereo gain and
  chase inside overlaps identically at blocks 1/64/333/1024. Existing linear/tension fade
  coverage also exercises smooth v3. An early run against an outdated native bundle failed;
  the rebuilt-bundle run passes.
* DSP duration/pitch tests pass at 0.5/1/2× and −12/0/+12 st for 13 source rates from
  8–384 kHz; invalid/extreme ratios, cancellation, identity, tiny and silent input are covered.
  Portable recipe tests prove reprepare uses the original and refuses implicit disk reads.
* All twelve release bundles were built, then final debug bundles rebuilt with allocation
  assertions. Exported-binary verification passed every bundle at 44.1/48/96 kHz ×
  1/64/333/1024, covering load, native editor creation, MIDI offsets, automation and state.
* Workspace build, strict workspace/native Clippy, both workspace formatting checks,
  feature inventories, local links/evidence and whitespace checks pass.

INS-007 / PL-023 retain IN_PROGRESS for remaining parity/verification work. Listening on
real music, FL A/B, standalone recall and physical/cross-platform runtime checks are queued
in [human verification](human-verification.md). Generic VST3 state/preset/parameter/editor
paths remain; preparation helpers require explicit Lime Sampler identity. Pre-existing
hosting gaps from the sanitation report were not promoted or broadened into this change.

Work uses the existing primary checkout `/Users/sebastian/Developer/limeshot` and its Cargo
caches. Continue there without routine worktrees/clones; push only the feature branch and
its PR, never main, and do not merge automatically.

### PR #25 — integration with current main

Main advanced to `c38615c` during implementation (Rev 1 Playlist/arrangements and sampler
ping-pong/filter work). Integrated it on the feature branch and retained both implementations.
The sampler panel now sits beneath the newer loop/filter controls; its combined 760×780
layout and the current Playlist/dialog PNGs were rendered and visually inspected. Private
channel reuse checks inactive arrangements as well as active clips/notes; its worker test
proves a clip edit leaves an inactive arrangement's shared sampler unchanged. Automatic fade
cleanup restores manual bases across arrangements before edits. The prepared hosted test also
runs with ping-pong enabled. Legacy right-edge stretch retains its existing shared-channel
resampling behavior; independent fixed-duration preparation is the Time stretching action.

Combined local macOS 26.6.2 arm64 validation: **493 workspace tests passed**, including 169
app UI and 13 hosted-native tests; **27 native tests passed**. Rebuilt all twelve debug bundles;
exported-binary verification passed all twelve at three rates/four block sizes. Workspace build,
strict Clippy in both workspaces, formatting, 301-feature inventories, repository references
(247 links, 6 embedded assets, 251 evidence paths) and whitespace checks pass. Original
source/restore bounds and generic VST3 identity gates remain enforced. The previous 479/23
counts above describe the pre-integration stage. Listening/live/platform gaps remain unchanged.

[PR #25](https://github.com/fruitapps/limeshot/pull/25) carries the DSP comparison, integrated
workflow, UI consistency review and main integration. Only the feature branch is pushed;
main is not pushed or the PR merged automatically. Continue using the primary checkout and
its Cargo caches without routine worktrees/clones.

## 2026-10-27 — PR #25 preview ownership and fade review follow-up

Continued from PR #25 on `codex/sampler-preview-fade-fixes` in the existing primary checkout
`/Users/sebastian/Developer/limeshot`, reusing Cargo caches. The follow-up PR targets
`codex/prepared-sampling-arrangement`; no worktree/clone, main push or automatic merge.

Fixed the preview ownership issue identified independently by Codex and CodeRabbit: channel
source mappings now retain their decoded PCM, so cache eviction cannot leave published virtual
paths without audio. Changed/removed source mappings release pins before new previews are
inserted. Raised the entry ceiling from 32 to 4096 while retaining the 256 MiB PCM budget;
large projects can show more than 32 small embedded sources. A full pinned budget refuses
additional previews, and embedded state never substitutes an external reload hint. Normal
file-cache requests remain available to existing asynchronous Playlist import helpers.

Smooth S fades is disabled with an explanatory hover hint for active automatic crossfade
pairs. The control reads current document state after the automatic toggle, so disabling
crossfades immediately restores the manual curve and enables editing; nonoverlapping clips
remain editable. The [menu PNG](../screenshots/current/automatic-crossfade-menu.png) was
rendered with `ui_shot --audio --crossfade-menu` and visually inspected. Research confidence
labels and the Time stretching… action name are corrected; INS-007 no longer incorrectly
calls ping-pong/fixed preparation unimplemented. Feature statuses remain unchanged.

Local validation: macOS 26.6.2 (25G83), Apple Silicon, debug. Rebuilt all twelve debug Lime
VST3 bundles. **495 workspace tests passed**, including 171 app UI tests. Added a final
cache-exhaustion/source-hint recovery regression afterward: the three embedded-preview UI
cases pass together (496 distinct tests covered overall). The 40-source regression forces
cache collection, checks PCM identity after unrelated edits and verifies source replacement/
removal releases pins. The cache unit test fills/pins all 4096 entries, rejects overflow,
then reclaims released buffers. Workspace build, strict all-target Clippy, formatting,
feature inventories, repository references and whitespace checks pass. The first full pass
caught two import-helper regressions from skipping raw-file cache requests; restored those
requests and the complete rerun passed. Live GUI/listening and Windows/Linux runtime checks
remain in human verification; offscreen tests do not establish those results.

PR #25 CI was green when reviewed. All five remaining inline findings are addressed in this
stacked follow-up: the two duplicate preview findings, curve checkbox, research labels/action
name and stale sampler notes. Its advisory docstring-coverage warning does not describe a
runtime failure; existing broad documentation coverage was not rewritten in this bug fix.
Continue implementation in the primary checkout/cache, push feature branches/PRs only, and
pass the owner's issue → implementation → previous-PR review/CI fixes → stacked PR → new
session workflow forward until the current usage window is exhausted; do not wait for a reset.

## 2026-10-27 — Custom VST3 search folders (PLG-004)

Continued from #26 in the primary checkout and existing Cargo caches on
`codex/custom-vst3-search-folders`. Settings › File now adds/removes persistent extra VST3
folders. Find plugins searches those plus platform defaults; discovery runs on the scan
worker before the existing per-bundle child processes, with separate discovery/progress text.
Overlapping roots, repeated bundles and symlink aliases scan once; canonical directory
identities stop cycles, and a shallower root can extend a previous depth-limited search.
Missing/offline roots remain in settings and are skipped until a later rescan. Removing a
root does not unload project instances. See [scope and evidence](../research/custom-vst3-folders.md).

Local validation: macOS 26.6.2 (25G83), arm64, debug app/tests. Rebuilt the twelve release
Lime VST3 bundles; workspace tests use the existing matching debug bundles via
`LIMESHOT_LIME_PLUGINS=$PWD/native-plugins/target/debug-bundled`. **501 workspace tests
passed**, including 173 app UI tests. Coverage adds duplicate/bundle roots, depth-limit
recovery, Unix symlink cycles/aliases, worker failure/progress and folder removal without a
project edit; settings round-trip covers defaults and custom/missing paths. A real app scan
child successfully found rebuilt Lime Utility through a temporary custom path containing
spaces. Workspace build, strict all-target Clippy, formatting, feature inventory, repository
references and whitespace checks pass. The first full test pass hit the sandbox's local
socket restriction in the existing bridge UI test; the full rerun with socket access passed.
The [settings screenshot](../screenshots/current/vst3-search-folders.png), rendered with
`ui_shot --prefs=File` and GPU access, was visually inspected. Live folder picker/rescan/
restart and Windows/Linux runtime checks remain in human verification; PLG-004 stays
IMPLEMENTED and the broader Plugin Manager setting remains partial.

Reviewed #26's implementation and review discussion: Codex review completed without
findings, no inline review threads existed, and CodeRabbit skipped automatic review because
its base is a feature branch. CI run 40590120668 was still running without a reported failure
at the pre-delivery check; no green cross-platform result is claimed. The owner's continuation
workflow remains: implement eligible roadmap work, inspect the preceding PR/CI/reviews, push
a stacked feature PR, and hand off to a new local session until the current usage window is
exhausted. Never wait for its reset or spend reset credits; keep using the primary checkout
and Cargo caches, and never push main or merge automatically.

## 2026-10-27 — Navigable undo history and PR #27 review fix

Continued from #27 in the primary checkout `/Users/sebastian/Developer/limeshot`, reusing
Cargo caches, on `codex/navigable-undo-history`. Edit › History now shows all retained
undo/redo edits in a scrollable list with the current state selected. Clicking restores the
state after an edit; Oldest retained state restores the first retained snapshot. Forward
navigation preserves redo, saved-state identity and revision updates; a new edit branches
normally. The existing 200-step cap and new/open reset remain. See
[reference, semantics and evidence](../research/undo-history.md). APP-006 stays IMPLEMENTED;
headless evidence does not establish live audio/plugin or cross-platform runtime behavior.

Addressed the valid Codex review finding on #27: canonical bundle paths now represent
symlink aliases during discovery. Renamed aliases no longer cause Linux/Windows binary
resolution to use the wrong bundle stem. The Unix test visits `Renamed.vst3` before the
real bundle and verifies only the canonical bundle is retained. Discovery tests use
canonical temporary roots, and the worker test compares canonical failure paths. The first
full suite caught that worker test's `/var` versus `/private/var` expectation after the
intentional canonical-path change; corrected the expectation and reran the full suite.
CodeRabbit skipped #27 review because its target is a feature branch; that is not a pass.

Local validation: macOS 26.6.2 (25G83), arm64, debug, using existing matching debug Lime
VST3 bundles via `LIMESHOT_LIME_PLUGINS=$PWD/native-plugins/target/debug-bundled`; no native/DSP
production changes. Core navigation and real-menu UI tests pass. Workspace build, strict
all-target Clippy, formatting, feature inventories, repository references and whitespace
checks pass. The [History menu PNG](../screenshots/current/undo-history.png) was rendered
with `ui_shot --history --menu=Edit/History` and visually inspected. Full-suite final result
is recorded below. Live/plugin/audio and other-platform checks remain in human verification.

At review time, #26 CI run 40590120668 still had Linux x64/ARM64 and Intel macOS unfinished;
other jobs passed. #27 CI run 40591035650 was still running without a reported failure.
Continue checking these and the new PR during the next session. Keep using the primary
checkout/cache, push feature branches only, never push main or merge automatically. The
owner's continuing chain remains authorized until the current 300-minute usage window
is exhausted; do not wait for reset or consume reset credits.

Final rerun: **504 workspace tests passed**, including **174 app UI tests** and 13
hosted-native tests. Bridge tests ran with local-socket access. No test failures remained.
Usage was 46% in the current primary window at the delivery check; ordinary usage remains
allowed and no reset credit was used.

## 2026-10-27 — Preserve Playlist clipboard properties (PL-026)

Continued from #28 in the existing primary checkout/Cargo cache on
`codex/preserve-clip-clipboard-properties`. The P1 clipboard workflow previously reconstructed
clips from source/track/timing only, silently losing mute, manual fades, curves and extra metadata.
The internal clipboard now retains full clip snapshots. `PasteClips` assigns fresh IDs through
validated placement, retains properties and relative timing, and restores cached automatic fade
bases before destination overlap recalculation. Document execution makes a mixed paste atomic
and undoable as one action. A failed paste preserves selection/clipboard; an entirely stale copy
selection leaves the clipboard intact. Sources remain linked current-project entities.

Validation: macOS 26.6.2 (25G83), arm64, debug, matching existing debug Lime bundles via
`LIMESHOT_LIME_PLUGINS=$PWD/native-plugins/target/debug-bundled`. **511 workspace tests passed**
(including **177 UI tests**, four new core clipboard tests and 13 hosted-native tests), with bridge
local-socket access. Targeted tests were rerun after strengthening selection count and unordered
clipboard-origin assertions. Workspace build, strict all-target Clippy, formatting, feature inventories,
repository references and whitespace checks pass. Rendered and inspected the Playlist and Edit
menu with `ui_shot --audio --menu=Edit` (temporary `/tmp/limeshot-clipboard-playlist.png`). No layout,
audio-thread or native/DSP production changes. Core coverage includes every clip kind, serialization,
saved-state undo/redo, invalid sources/tracks, empty/extreme placement and automatic crossfades.
Real Playlist shortcuts cover cut/paste properties and single undo; Song placement and failed-paste
selection are also tested. PL-026 remains IMPLEMENTED: headless tests do not establish live audio,
FL equivalence or other-platform runtime behavior. See human-verification and known limitations.

Inspected #28: Codex review completed at `75c1ff0` with no findings; inline review threads empty.
CodeRabbit skipped the feature-branch base, not a pass. #26 CI run 40590120668 at `18952da` is now
all green (five native OS/architecture jobs, Windows ARM64 cross-build and rustfmt). #27 run
40591035650 still has Intel macOS and Linux x64/ARM64 running; other jobs green. #28 CI
40591729991 and native-plugin run 40591729889 remain running without reported failures at this
check. Continue inspecting them and the new PR in the next local session. The #27 renamed-symlink
review finding is already fixed in #28; its original thread remains unresolved on #27's branch.

The owner's local-session chain remains authorized until this current 300-minute usage window
is exhausted (reset Unix 1793153182); do not wait for reset or consume reset credits. At the latest
check usage was 54%, ordinary usage allowed. Keep using the primary checkout and Cargo caches,
push feature branches/stacked PRs only, never push main or merge automatically.

## 2026-10-27 — Embedded Windows executable icons (PF-006)

Continued from #29 in the existing primary checkout `/Users/sebastian/Developer/limeshot`,
reusing Cargo caches, on `codex/windows-executable-icons`. Both Windows app/CLI binaries now
embed the existing LimeShot icon at six sizes and Cargo version metadata. A build script uses
unmodified build-only `winresource` with target-specific links, leaving libraries/tests/examples
without resources. The checked-in ICO is generated by the existing Rust icon painter; a
regeneration test guards artwork drift. Packaging copies that same ICO rather than regenerating
an unrelated staged file. Attribution and packaging prerequisites are documented.

The Windows resource verifier maps each executable as data, checks application group icon 1
and compares each referenced image byte-for-byte to the ICO. Windows x64 and ARM64 cross-build
CI check both debug binaries; packaging checks the staged release binaries, including SkipBuild,
so stale or mismatched resources fail before an archive/installer is produced. Resource checks
on Windows are pending until CI runs; no Windows desktop appearance is claimed. Explorer,
taskbar, Alt+Tab and installed/portable checks remain in human verification. PF-006 stays
IMPLEMENTED, with Finder document opening and signing/notarisation still open.

Local validation: macOS 26.6.2 (25G83), arm64/debug. **511 workspace tests passed** (177 UI,
13 hosted-native), using the existing matching debug Lime bundles via
`LIMESHOT_LIME_PLUGINS=$PWD/native-plugins/target/debug-bundled` and local-socket access.
The additional icon regeneration test passes (512 total including this example test).
Workspace build, strict all-target Clippy, formatting, feature inventory, repository links
and whitespace checks pass. The verifier's C# helper compiles with the local compiler;
PowerShell/Windows APIs are unavailable here. No native/DSP/audio-thread or UI layout changes;
the icon artwork itself is unchanged.

Inspected #29: Codex review completed at `065ed21` with no findings, inline review threads empty;
CodeRabbit skipped its feature-branch base, not a pass. CI 40592530958 and native run 40592530952
were still running without reported failures. #27/#28 Windows, ARM64 cross-build and Apple Silicon
jobs pass; Intel macOS has completed validation and is saving cache, while Linux jobs remain
running. These partial results do not establish a green overall run. Keep future work in the
primary checkout/cache, push feature branches only and never merge automatically.

The owner explicitly requested stopping after this session, superseding the earlier ongoing
local-session chain. Do not create another session or automation. Usage at the latest check was
63% in the same authorized window; no reset credit was used.
