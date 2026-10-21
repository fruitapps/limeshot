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

## 2026-10-20 — Session 5: expanding the MVP's core

Done (each a commit with tests and feature records):
1. **Audio clips** (PL-014, BR-010, SEQ-019): samples dropped on the Playlist become an audio-clip channel and a clip;
   sample-accurate playback from the clip offset, part-way start when playback begins inside a clip, waveform overview,
   trimming from the left edge for every clip kind, copy/paste of every clip kind.
2. **Automation clips** (AUT-001…003, PL-015): FL's automation UX was observed live first (§11 of
   `fl-studio-analysis.md`). Created from control context menus, edited in the clip, applied in Song mode to channel
   and mixer volume/pan and effect parameters (also in exports).
3. **MIDI files** (PR-020, EXP-012): own SMF reader/writer, File menu import/export, `limeshot-cli midi` and
   `import-midi`.
4. **LimeSynth** filter, filter envelope and LFO (INS-006); Channel Settings label column widened.
5. **Autosave and crash recovery** (APP-016, APP-024). A test found that a recovered copy was never removed after saving;
   fixed with explicit ownership (D30).
6. **Inventories:** 16 new feature records (236 total), plus `keybindings.json` (76 FL shortcuts) and `settings.json`
   (27 FL settings) checked by `scripts/inventories.py`. The check immediately found Cmd+Q and Home missing from
   Help › Keyboard shortcuts.

Real-input pass over the live app (`limeshot --demo`, synthetic CoreGraphics input): dragged "Lime Snare" from the
Browser onto Track 6 (audio clip and channel created, song grew to 9 bars), played it in Song mode (1 voice, master
meter), right-clicked the Kick fader › Create automation clip (clip on Track 4, listed in the picker), right-clicked
the curve to add a point and dragged it. The Mixer was opaque (session 4's fix).

Verification: `cargo test --workspace` (123 tests) passes; Clippy (`-D warnings`), `cargo fmt --check` and
`scripts/features.py check` are clean. `limeshot-cli midi` → `import-midi` round-trips the demo (130 notes).

Not verified: Windows/Linux (CI results still unread), exported MIDI in a third-party program, automation editing by a
person (the curve area is small at the default track height).

## 2026-10-21 — Session 6: FL Studio project import

Done (each a commit, pushed to `origin/main`):
1. **`.flp` import** (APP-022, now VERIFIED): `limeshot-project::flp` reads the event stream and converts tempo, time
   signature, channels, samples, levels, inserts, patterns, notes and the first arrangement's pattern and audio clips.
   File › Open accepts `.flp` and opens it as a new unsaved project (Save asks for the `.lime`), as does
   `limeshot song.flp`; the toast shows the import report. CLI: `limeshot-cli import-flp in.flp [out.lime]`.
2. **Test files saved by FL Studio 26.1.1** (`crates/limeshot-project/tests/fixtures/flp/fl26/`, how each was made is
   in its README): drum steps, piano-roll notes with lengths and velocities on a 3x Osc channel plus a named pattern,
   and an arrangement with pattern, audio, automation and muted clips at 128 BPM in 3/4. Plus six synthetic files
   from `make_flp_fixtures` and an import of all 164 projects FL ships.
3. **Findings from the FL-saved files** (all in `docs/research/flp-format.md`): playlist item records are 80 bytes in
   FL 24–25 and 88 in FL 26 — the earlier "240 bytes" was a multiple of 80 by chance, so e.g. `NewStuff.flp` lost two
   thirds of its clips; mute is bit 0x2000 of the word at offset 18 (the code had guessed 0x1000); FL shows key 60 as
   C5 and velocity as a percentage of 128; FL 26 writes pattern names after the channel list under a second event 65;
   changing the time signature rewrites pattern clip lengths to whole bars.

Driving FL Studio for fixtures (synthetic CoreGraphics input): FL ignores quick clicks inside its editors — hold the
button ~0.3 s (mouse move, wait, down, wait, up). The native menu bar does not stay open; use FL's own menus in the
window. FL crashed once (its own access violation) when a sample was dragged from the browser before its preview had
loaded; clicking the sample first and waiting a few seconds avoided it. Save often while building a fixture. The
welcome dialog's close button at first-start sometimes does nothing until FL finishes loading the template.

Verification: `cargo test --workspace` (136 tests) passes; Clippy (`-D warnings`), `cargo fmt --check` and
`scripts/features.py check` are clean.

Not verified: `.flp` files from FL versions other than 26.1.1 with known content (older ones only import without
errors); opening an `.flp` through the real file dialog by a person (the headless test calls the same `open_path`).

## 2026-10-21 — Session 7: closing the MVP one FL feature area at a time

Plan agreed with the user: verify the remaining MVP features area by area (each observed in FL Studio first), get
Windows/Linux CI running, then expand, starting with the Piano Roll tools. Each area is one commit.

1. **Channel Rack + Browser.** Observed live in FL 2026.1.1 (`fl-studio-analysis.md` §2, five new screenshots):
   selector LEDs (click selects one; Ctrl/Shift/right-click toggles, no Shift range), Alt+Up/Down moves the selected
   channels, Alt+Del and the menu's Delete… ask first ("Remember my choice"), F2 renames the *pattern* (KB-041 had
   guessed "rename channel"), clicking a step or right-clicking a channel button selects that channel only.
   LimeShot now has the same channel selection (SEQ-020, new), move/delete/Cmd+L on the selection (one undo step
   each), the confirmation dialog (setting `confirm_channel_delete`, SET-028) and F2. SEQ-007, SEQ-008, SEQ-020 and
   BR-001…004 are VERIFIED by new headless UI tests (LEDs with modifiers, delete dialog, Cmd+L, F2, the rack's note
   preview, Browser preview/double-click/drag/folders/filter). Real-input check in the live app: Cmd-click on LEDs,
   Option+Up and Option+Delete behave as in the tests.
   Test note: the egui_kittest harness advances 0.25 s per frame, so double-clicks set the frame time explicitly
   (`double_click` in `tests/ui.rs`).
2. **Piano Roll basics.** FL 2026.1.1's Piano Roll menu lists its macOS shortcuts (two new screenshots): transpose
   and nudge are Shift+arrows, plain arrows scroll, and with nothing selected the edit applies to all of the
   channel's notes. LimeShot used plain arrows (KB-057/058 "DIFFERENT"); it now matches, and Shift+I inverts the
   selection. **Bug found by the new tests:** Cmd+wheel zoom never worked in the Piano Roll or the Playlist (egui
   reports Cmd+wheel and pinch as a zoom delta, not as scrolling); both now use `piano_roll::wheel_zoom`. PR-002,
   PR-004…007 and PR-009…011 are VERIFIED by five new UI tests (selection gestures, velocity lane, wheel/arrow
   scrolling and zoom, Shift/Cmd arrows, keyboard audition, ruler locate, ghosts, channel selector). Live check:
   Cmd+wheel zooms both ways and the playhead draws during playback.
3. **Playlist, Mixer and app.** FL 2026.1.1's Playlist menus (two new screenshots) list Shift+arrows to shift clips,
   Cmd+A/Cmd+D, Shift+I invert, Opt+M mute, Cmd+G merge. LimeShot moved clips with plain arrows (KB-069 INFERRED);
   it now uses Shift+arrows, plain arrows scroll (inferred from the Piano Roll: FL could not be brought to the front
   for a live check this time) and Shift+I inverts. The toolbar's position readout now exposes its text to
   accessibility. New UI tests verify PL-003/006/008/009, MIX-005 (Song playback reaches the master meter),
   APP-001/014 (New asks to save; Cancel / Don't save) and TR-005/006; APP-012 is VERIFIED by its recorded manual
   checks. All MVP features except the platform ones (PF-002…004, waiting for CI) are now VERIFIED.

Not done in this session: Windows/Linux CI. Every run of `.github/workflows/ci.yml` on `fruitapps/limeshot` stays
"pending" with no jobs, and each new push cancels the pending one (the workflow's concurrency group). The account used
here is not an org admin, so the org's Actions and billing settings could not be read; the owner has to check them.

5. **Piano Roll tools (expansion).** Observed live in FL 2026.1.1: the Tools menu (Glue Cmd+G, Quick chop Cmd+U,
   Quick legato Cmd+L, Quick quantize Cmd+Q, Quantize… Opt+Q …), Paint (end-to-end notes of the last length, all
   selected) and Slice (cut at the line, right piece selected). LimeShot now has FL's eight tools with their keys and
   painted icons (Draw, Paint, Delete, Mute, Slice, Select, Zoom, Playback), left-edge resizing in Draw/Paint, note
   mute (`Note::muted`, silent in both modes, dimmed; saved only when set), Glue, Quick chop and Opt+M / Shift+Opt+M.
   New commands `SliceNotes`, `GlueNotes`, `SetNotesMuted`, `ResizeNotesStart`; a paint stroke, a mute drag and a
   left-edge drag are each one undo step (merge keys). PR-012, PR-023 and PR-024 are VERIFIED; the Stamp tool is a new
   record (PR-027). FL's macOS menu binds quick quantize to Cmd+Q; LimeShot keeps Cmd+Q for Quit (KB-059 DIFFERENT).
6. **Playlist time selection and markers (expansion).** Observed live in FL 2026.1.1: a plain ruler drag scrubs,
   Ctrl+drag makes a time selection (red band, song position to its start), the Time markers menu (Add one… Opt+T,
   Add auto Cmd+T …), and plain Up/Down scrolling the Playlist a track (confirming task 3's inference). LimeShot now
   has the Cmd+drag time selection, looped by the engine in Song mode (`Renderer::set_loop_range`, tested at block
   sizes 1…1024), and time markers in the model (`Playlist::markers`, `AddMarker`/`RemoveMarker`/`RenameMarker`/
   `MoveMarker`) with Cmd+T / Alt+T and click-to-jump, drag-to-move and right-click menus on the ruler. PL-016 is
   VERIFIED; special marker kinds and Place loop are recorded as missing. `ui_shot --markers` shows them.

## 2026-10-21 — Session 8: the rest of the session 7 plan, then VST3

CI first: the newest run on `fruitapps/limeshot` is still "pending" with no jobs, and older ones were cancelled with
no jobs. Windows/Linux builds stay blocked on the org's GitHub Actions settings (owner action needed).

1. **Playlist tools, slip and slice (PL-017, KB-064).** Observed live in FL 2026.1.1 (three new screenshots): the
   tool strip (Draw, Paint, Delete, Mute, Slip, Slice, Select, Zoom, Playback), Slip moving the content under a
   shortened clip (snapped, silence past the sample end, stopping at the sample start), and Slice cutting a clip in
   two at the snapped line with the content running on across the cut. LimeShot's Playlist now has the same tool
   strip on the left of its header, FL's single-key shortcuts, and the shared `piano_roll::Tool` (new `Slip`
   variant, `Tool::PLAYLIST`, a painted slip icon). New commands `SlipClip` and `SliceClips`; a paint stroke, a
   mute drag and a slip drag are each one undo step (new merge keys for clip placing and muting). PL-017 is VERIFIED
   by a core test and four UI tests; `ui_shot --audio --tools` shows a sliced and a slipped audio clip. Missing:
   Shift+S ripple edit, diagonal and Shift+click slicing, zero-cross snapping.
2. **Mixer track EQ (MIX-010).** Observed live in FL 2026.1.1 (`fl-mixer-eq.png`): the Equalizer at the bottom of
   the Mixer's track panel, its band names, defaults (90 / 1500 / 8000 Hz, 0 dB, width 0.27) and ±18 dB range read
   from the hint bar. Every LimeShot mixer track now has `MixerTrack::eq` (three `EqBand`s, saved only when changed,
   sanitised on load) and `Command::SetMixerEq`. The renderer runs a preallocated `Eq3` per track after the insert
   effects and before the fader, redesigning the bands only when a snapshot arrives (no allocation on the audio
   thread). The Mixer's track panel shows the response curve with a draggable handle per band (left-drag frequency
   and gain, right-drag width, as in FL), gain/frequency/width knobs and Reset. Tested in the DSP, core, engine and
   UI; `ui_shot --eq` shows it.
3. **Sidechain and gain reduction (MIX-009).** Observed live in FL 2026.1.1: the route button's right-click menu and
   what "Sidechain to this track" does (a send at level 0, two new screenshots). LimeShot's route buttons now have the
   same four entries (the "only" variants are one undo step). `Effect` gained `process_sidechain`,
   `sidechain_source` and `gain_reduction_db`; Lime Comp has a sixth parameter, Sidechain (the key track, which must
   route to the compressor's track). The renderer hands a sidechained effect the key track's post-fader bus (the
   bus is moved out and back, so nothing allocates) and publishes each track's gain reduction through `Status`. The
   compressor card shows a Sidechain picker and a reduction meter; the track panel lists incoming sidechains.
   Tested in the engine (the key ducks the synth by more than 10 dB, an unrouted key is ignored, the reduction is
   reported block by block) and the UI (route menu, picker, live meter during playback). `ui_shot --sidechain`.
4. **Channel Rack fill and rotate (SEQ-014).** Observed live in FL 2026.1.1: Fill each 4 steps replaces the
   channel's steps, Shift+Cmd+Left/Right rotate by one step and wrap (two new screenshots). New commands `FillSteps`
   and `RotateNotes`; the channel button's menu has Fill each 2/4/8 steps and Rotate left/right, Shift+Cmd+Left/Right
   rotate the selected channels in the Channel Rack and, by the snap, the Piano Roll's selection (KB-040, KB-081).
   Tested in core and UI. Advanced fill… is not done.
5. **Graph editor (SEQ-011).** Observed live in FL 2026.1.1: the panel under the step row, its lanes, and drags that
   draw a line (new screenshot). Notes gained `pan` (added to the channel's pan in the engine, saved only when set,
   sanitised on load) and `Command::SetNoteValues`, whose merge key makes a whole drag one undo step. The Channel
   Rack's header button or the channel menu's Graph editor opens the panel under the rack for the selected channel,
   aligned with its steps, with Velocity and Pan lanes. Tested in core, engine (note pan adds to channel pan) and UI.
   `ui_shot --graph`.
6. **Channel filter groups (SEQ-012).** Observed live in FL 2026.1.1 (three new screenshots): the filter dropdown,
   Group selected… (Opt+G) and its name field. Channels gained an optional `group` (saved only when set) and
   `Command::SetChannelGroup`; audio-clip channels start in "Audio". The Channel Rack header has the All / group /
   Unsorted dropdown, Alt+G and the channel menu name the selected channels' group (KB-093). Tested in core and UI.
   This completes the session 7 expansion plan. VST3 hosting is postponed at the owner's request (2026-10-21).

## 2026-10-21 — Session 9: fixes from the owner's hands-on test

The owner tested session 8's build by hand and listed 16 issues (`next-steps.md`, Mandatory 2). Each is fixed and
covered by a headless UI test unless noted.

1. **Windows and the Mixer (issues 1, 4–9, 11, 12).** Restoring a maximised window kept the maximised size: egui
   stores a window's size, and `fixed_rect` while maximised overwrote it. The workspace now remembers each window's
   floating rectangle and puts it back for one frame on restore (unconstrained that frame, or egui clamps it with
   the old size). Mixer names are typed in place (double-click a strip's name, click the track panel's; the old
   name starts selected; Enter or clicking elsewhere keeps it, Esc cancels) instead of the rename popup. The track
   panel's Output menu lists each target with the route button's four entries (route / sidechain, "only"
   variants). The send knob used 0–1 linear gain while sends go to +6 dB (2.0), so 0 dB sat at the end and higher
   levels overshot the arc; it now shares the faders' cube-root travel (0 dB at about 80 %). The strips' scroll bar
   is a thin bar in its own gutter (the theme's scroll bars no longer use egui's bright foreground colour).
   Double-click: LimeShot's knobs and faders already reset, but egui's sliders only sense drags, so their response
   never reports a double-click; `widgets::reset_on_double_click` now resets the Mixer track panel's volume, pan and
   send sliders, the effect cards and every Channel Settings slider to their defaults. **Insert default (issue 9):**
   the −1.9 dB was `MixerTrack::new`'s 0.8 linear gain, a confusion with FL Studio's fader *position* (0 dB sits at
   about 80 % of an FL fader, which goes to +5.6 dB). FL Studio, Ableton Live, Logic and Reaper all start a new
   mixer track at 0 dB, so new inserts are now 0 dB (and that is the fader's double-click default, where its tick
   already was). The Channel Rack reports the size its content needs, so its window never hides steps or shows
   fewer than five channels; the Mixer's minimum is the master and two strips next to a track panel kept at its
   full width (its EQ knobs were cut off when the panel shrank to 45 % of a narrow window), and a whole strip high.
2. **Piano Roll and Playlist editing (issues 3, 13, 14).** Note resizing: dragging a note's right edge past its
   start let the core clamp the note to one tick, while the drag kept a running total of what it had applied, so
   the note collapsed to a sliver and then no longer followed the pointer. The drag now clamps the change itself:
   no note gets shorter than one snap step (or its own length, if shorter) and the left edge cannot pass the start
   of the timeline. **Snap and grid (issue 13, confirmed and fixed):** the Piano Roll's grid did follow the snap,
   but only once the lines were more than 6 px apart, and quarter steps are exactly 6 px at the default zoom, so
   1/4-step snap showed no finer lines; the threshold is now 5 px. The Playlist's grid ignored the snap (fixed to
   bars) and now follows it too (`piano_roll::grid_step`, shared). Vertical zoom (issue 14) did not exist: Opt+wheel
   and Opt+pinch now zoom the Piano Roll's key height and the Playlist's track height around the pointer (FL
   Studio's Alt+wheel, KB-094, from FL knowledge: a live check was not possible this session because FL could not
   be brought to the front).
3. **Playback scrubbing and tool cursors (issues 2, 15).** The Playback tool (Y) started the transport from the
   pointer (FL Studio's press-to-play, observed in session 7). At the owner's request it now scrubs: over the grid a
   faint line follows the pointer to show what a press would play; while the button is held the line brightens and
   the notes it crosses sound (`piano_roll::scrub` diffs the sounding set each frame, so each note stops when the
   line leaves it), and the transport never starts. In the Playlist the line sounds the notes of the unmuted pattern
   clips under it (audio clips are not scrubbed). Every tool now paints its toolbar icon next to the pointer over
   the Piano Roll and Playlist grids, on top of the system cursor egui sets for edges, slicing and zooming (egui has
   no pencil, eraser or speaker cursors). `ui_shot --scrub` shows the line and the cursor.
4. **Pattern colours and the CPU readout (issues 10, 16).** New `Command::SetPatternColor` (one undo step) and a
   shared palette swatch picker (`widgets::swatch_picker`, the current colour outlined, each swatch labelled for
   accessibility) under Color in the Patterns menu, the Playlist's pattern picker and its pattern clip menus; the
   channel menu's colour row uses the same picker. The toolbar's CPU readout used to repaint the instantaneous load
   every frame; `toolbar::CpuReadout` now changes it once per second and shows the highest load of that second, and
   the toolbar asks for a repaint after a second so the value settles after a stop.

All 16 issues are fixed. Not done: a live FL Studio check for Alt+wheel vertical zoom (FL could not be brought to
the front over the Claude window, and live input was stopped when focus moved back to it), and a real-input pass
in the live LimeShot window for the new gestures (the headless tests drive them with synthetic egui events).

## 2026-10-21 — Session 10: VST3 hosting

CI still shows every run as "pending" with no jobs (Mandatory 1 stays blocked on the org's Actions settings), so
work moved to Mandatory 3, VST3 hosting.

1. **The host crate (`limeshot-plugins`, decision D32).** The VST 3.8 SDK is MIT-licensed, and the `vst3` crate ships
   bindings generated from its headers (no SDK download). `Vst3Module` loads a bundle (CFBundle on macOS, the
   `Contents/<arch>-win|linux` binary elsewhere, entry and exit functions called), shares a module between
   instances and lists its audio classes from `IPluginFactory2`. `Vst3Instance::new` initialises the component and
   its controller (single-object or separate, connected through `IConnectionPoint`; the host context creates
   messages and attribute lists), negotiates stereo main buses with auxiliary buses off, and splits the plugin into
   `Vst3Instance` (parameters, display text, editor edits, latency, state as one blob) and `Vst3Processor`, whose
   `process` uses preallocated buses, a fixed event list and fixed parameter queues fed from a ring. The processor
   implements `Effect` and the new `limeshot_dsp::Instrument` trait (notes at frame offsets plus a transport).
2. **Tests without installed plugins.** `limeshot_plugins::testing` compiles two small VST3 plugins into the crate
   (Lime Test Gain with a separate controller, Lime Test Tone as one object), and `tests/host.rs` runs them
   through the full path: class listing, in-place processing at several block sizes, parameter changes, the
   `Effect` trait, state into a new instance, and notes starting and stopping at their sample offsets.
3. **Installed plugins.** `examples/vst3_probe` loads one bundle on the main thread and processes 40 blocks. Run
   once per bundle in its own process, it loaded all 36 VST3 bundles on the dev Mac (40 classes, including Serum 2,
   Vital, Surge XT, M1, Analog Lab V, Guitar Rig 7 and Waves through WaveShell) with no crash or timeout. In the
   test harness (off the main thread) SINE Player trapped, so plugins are created on the main thread.

4. **Plugins in projects.** `PluginRef` (core) names a plugin and carries its state (base64 in `.lime`, shared
   between undo copies; a bad blob loads as empty). Channels gained `Instrument::Plugin`, insert slots
   `EffectKind::Plugin`, and `Command::AddPluginEffect`; `Document::store_plugin_state` copies a running plugin's
   state in before saving without making an undo step.
5. **Engine.** `InstrumentRack` gives a plugin channel its notes as events at exact frame offsets and holds each
   note until its note-off sample, so the output does not depend on the block size (tested at 1, 64, 333 and 1024
   frames with a fake instrument); stop, locate and loops release held notes; auditions work while stopped.
   `Msg::Instrument` hands instruments to the audio thread, which returns retired ones as garbage. Offline renders
   take their own processors (`render_offline_with`, `export_wav_with`).
6. **App.** `PluginHost` (`limeshot-ui/src/plugins.rs`): Find plugins… scans in child processes
   (`limeshot --scan-vst3`) into `plugins.json`; the Channel Rack's and the insert chain's Add menus list it under
   Plugins; plugins are created and destroyed on the main thread as the project changes (undo included); Channel
   Settings and the insert card list the parameters as sliders with a filter, and a disabled Editor button. Saving,
   autosave, WAV export and `limeshot-cli render`/`play` include plugins. Headless UI tests drive the menus with the
   built-in test plugins, save, reopen and render. `ui_shot --plugin` shows the panels.
7. **Real plugins.** The demo song with its Lead on Surge XT (through an OTT insert) and its Bass on Vital renders
   (each part alone peaks at −15 and −14 dBFS) and plays in realtime through the speakers (−17.7 and −14.1 dBFS,
   about 25 % callback load).

Next: plugin windows (PLG-006, native views in their own windows), automation of plugin parameters (PLG-007),
delay compensation (PLG-009), and a live pass of Find plugins… in the window.

## 2026-10-21 — Session 11: the owner's list before plugin windows

The owner listed 19 items (10 issues, 9 features) to finish before plugin editor windows (PLG-006). Each was
checked in the live FL Studio 2026.1.1 first (synthetic CoreGraphics input); differences are noted per item.

1. **Samples in the Playlist like patterns (item 1).** FL's picker has Patterns, Audio and Automation tabs; a sample
   dropped on the Playlist selects the Audio tab and the new clip, and clicking empty space draws it. LimeShot's
   picker now has the three tabs; the Audio tab lists the audio-clip channels (samplers with a sample, filed under
   "Audio"), + or a drop on the picker adds one, and the Draw/Paint tools place audio clips like patterns. Differs:
   FL's audio clip channels are a special generator; LimeShot's are Sampler channels.
2. **Playlist gestures (items 2–4, 12).** Shift+drag moves copies (FL); right-click on empty space clears the
   selection (FL deletes only the clicked clip and keeps the rest selected, as LimeShot already did); the Draw tool
   gives a new clip the length of the clip of that source last placed, resized or clicked (FL). `Document` gained
   compound gestures, so placing and dragging a clip, or duplicating and moving, is one undo step. A sample dragged
   over the grid shows a ghost clip (FL labels the pointer and highlights the target cell instead).
3. **1/3-step snap (item 7)**, as in FL's snap menu.
4. **Plugin channel settings (items 5, 6).** Vital's and Serum's Channel Settings were stuck at the workspace width:
   a parameter value wider than the space its row reserved ("1.95312ms") overflowed the row, egui grew the window
   to fit, the slider grew with the window, and so on until the window filled the workspace. Every workspace window
   now lays its body out in exactly its own area (overwide rows are clipped, the window stays resizable), and the
   parameter rows reserve the value's measured width. Checked live with Vital. Plugin channels no longer show the
   Sampler/LimeSynth switch.
5. **Auditions always end (item 8).** When an audition's note-off reached the audio thread in the block its note-on
   started in (a quick click, a drag across keys, large buffers), both events had offset 0 and note-offs sort first,
   so a plugin such as Serum got the off before the on and held the note until Play/Pause. A note-off now goes out
   at least one frame after its own note-on (engine test reproduces it at 1, 64 and 512-frame blocks), and the app
   releases any audition still sounding once no mouse button is down.
6. **Graph editor overlay (item 9).** As in FL (Cmd+K), the Channel Rack's graph editor is now an overlay right
   under the selected channel's steps, aligned with them, over the channels below and past the window's edge (its
   own layer, kept just above the rack's window).
7. **Stop and Play silence (item 10).** Stop while stopped now silences everything: voices (notes, auditions,
   sample previews) fade in 5 ms, plugins get their note-offs in a one-frame block and are reset, and effect tails
   are cleared once the fade is over. Play starts from the same silence, with the effects cleared at once.
8. **Sample preview (item 11).** Clicking a sample in the Browser already played it; the Browser now also has FL's
   preview panel (the selected sample's waveform, the preview's position, rate and length; click to replay).
   Previews start from the app's frame logic, so the picker's audio clips play when clicked with the Browser
   hidden, and pressing the waveform in a sampler's Channel Settings plays the channel.
9. **Tool cursors (item 13).** The icon that followed the pointer is now the cursor itself: over the Piano Roll and
   Playlist grids the system cursor is hidden and the tool is drawn at the pointer with its point on the hotspot;
   edges, moving and menus keep the standard cursors.
10. **Windows (items 14, 15).** A press anywhere in a window brings it to the front; the maximised Playlist is no
    longer a fixed backdrop (in FL a click on it raised it over the Channel Rack). Double-clicking a title bar
    maximises or restores the window, as in FL. A pin button keeps a window above the others (LimeShot's own; FL's
    editor windows have none).
11. **Pattern selector (item 16).** The toolbar's − and + buttons are gone; the selector's list ends with Create
    new. (FL keeps a + button and makes new patterns with Find first empty… in the same menu.)
12. **Playback tool (item 17).** The Piano Roll's notes under the held Playback line light up while they sound.
13. **Hint panel (item 18).** FL's top-left hint panel: in LimeShot's header after the menus, a first line with what
    LimeShot is doing (playing/stopped, mode, last edit, exporting, scanning, loading samples) and a second with
    what the control under the pointer does, its shortcut on the right. Every tooltip feeds it (`widgets::Hint`),
    the editors add their own descriptions, and knob and fader values show while dragged. The status bar no
    longer carries hints. Icon buttons gained accessible labels.
14. **Detents (item 19).** In FL a drag that crosses a knob's default holds it ("Centered") for several pixels,
    while a drag starting there moves at once (observed live). Knobs and faders (`drag_value`) and the egui sliders
    (through `reset_on_double_click`) now do the same with a 6 px hold.

All 19 items are done (see the per-item notes above and in `features.json`). Not done: plugin windows (PLG-006),
which wait for the owner's go-ahead. Verification: `cargo test --workspace` passes; Clippy (`-D warnings`),
`cargo fmt --check` and `scripts/features.py check` are clean. Live checks this session: Vital's Channel Settings
width (item 5), the Browser's preview panel (item 11) and the tool cursor (item 13) in the running app; the other
items are covered by headless UI tests and engine tests, not by a live pass with real plugins (Serum's stuck
notes, item 8, were reproduced and fixed in an engine test, not re-checked with Serum itself).

## 2026-10-21 — Session 12: plugin windows

The owner asked for plugin windows (PLG-006), checking first how FL Studio's plugin windows integrate with its own
channel settings.

1. **FL observed live** (FL Studio 2026.1.1, synthetic input; `fl-studio-analysis.md` §17, `fl-plugin-window-*.png`).
   FL had no plugins scanned, so its Plugin Manager scanned the standard folders first (FL's own database; the
   install is untouched). One floating window per plugin, titled "Vital (Master)" / "Surge XT Effects (Insert 1)",
   with the plugin's view in a child window. The gear shows a page strip: Plugin editor, Plugin options (FL's wrapper
   settings) and, for channels, Miscellaneous functions (the channel settings shared by every generator), plus
   ON/PAN/VOL/PITCH/TRACK knobs (effects: ON/MIX LEVEL). The window takes each page's size and follows the plugin's
   own resizing (it has no resize handle); a click on the channel button or the insert slot toggles it; it reopens
   at the same place, size and page. Title bar: ▸ menu (presets, parameters, CPU, colour, channel state), keyboard
   focus toggle, Presets ◀ ▶, captionize, close.
2. **Editor hosting** (`limeshot-plugins/src/editor.rs`, D33). `Vst3Instance::create_editor` asks the controller for
   its `IPlugView` and gives it the host's `IPlugFrame`; `Vst3Editor::attach` puts a container `NSView` (objc2) in
   a window found with `NativeWindow::find`, below the host's tabs, and `resizeView` resizes container and window at
   once before `onSize`. The editor keeps its plugin alive, and is detached (`removed`, container out of the window)
   before the window goes. `Vst3Instance::take_restart` reports `restartComponent`. Windows and Linux compile with a
   stub platform module. Lime Test Gain gained a minimal editor (keys stand in for the mouse: an edit, a resize
   request), so `tests/host.rs` covers create, attach, edits, resizing, detach and drop without a window.
3. **Plugin windows** (`limeshot-ui/src/plugin_window.rs`). One egui viewport per open plugin, titled like FL's;
   Plugin tab (the native editor) and Settings tab (Channel Settings for a channel, the insert card for an effect).
   The window is the editor's size plus the tabs on the Plugin tab and resizable on Settings; it remembers position,
   tab and Settings size for the session, floats above the main window and hides while LimeShot is in the
   background. The channel button of a plugin channel and the insert's name now toggle the window; the Editor
   buttons work. Editor edits mark the project modified (performEdit, restartComponent, and a state comparison when
   the window closes). Plugins without an editor get the parameter list on the Plugin tab. Headless UI tests drive
   the channel button, the tabs, reopening, the slot and the Editor button.
4. **Live check** (release build with a temporary HOME): Vital, Serum 2 and Surge XT channels and an OTT insert, as
   listed in PLG-006's notes (tab switching and resizing, Vital's own resize corner, a knob edit marking the project
   modified, close/reopen, toggling, quitting with a window open). Surge XT reading its Documents folder made macOS
   show a privacy prompt for the dev `.app` wrapper; it was not answered, and the build was run from the shell
   instead.

Differences from FL: two tabs instead of three pages (no wrapper options page), no title-bar presets, menu,
keyboard toggle or captionize, no ON/PAN/VOL strip, positions not saved in the project. Verification:
`cargo test --workspace` passes; Clippy (`-D warnings`, also for `x86_64-pc-windows-msvc`), `cargo fmt --check` and
`scripts/features.py check` are clean.
