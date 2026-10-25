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

## 2026-10-22 — Session 13: FL Studio's main menu bar (outside-in documentation)

The owner asked for FL Studio to be documented from the outside in, starting with the main menu bar, and for the
missing menus to be tracked in the roadmap in a form both people and scripts can read. Scope agreed up front: FL's
in-window menu bar, every submenu, no dialogs yet; static lists (templates, plugins) recorded in full.

1. **FL observed live** (FL Studio 2026.1.1, synthetic input; `fl-studio-analysis.md` §18, `fl-menu-*.png`). Each of
   the eight menus and their submenus was opened and captured. FL mirrors its menus into the native macOS menu bar, so
   an Accessibility dump (`docs/research/evidence/menus/`) gave exact labels and shortcuts; the in-window screenshots
   added section headers, enabled states and check marks. Template names were read from the bundle's
   `Data/Templates` folder. An accidental clip painted into the Playlist was undone through the Edit menu (which
   showed how FL names the undo step). Nothing was saved in FL.
2. **`docs/roadmap/menus.json`** (new inventory, 277 items): one record per menu item with FL's path, section, kind,
   keys, default state, where LimeShot has it, a status (IMPLEMENTED, DIFFERENT, LIMESHOT_ONLY, NOT_STARTED,
   NOT_PLANNED) and the feature that implements it; `lists` holds templates, generators, effects, toolbar presets,
   stretch modes and declicking modes. `scripts/inventories.py` validates it (a claimed LimeShot label must exist in
   `crates/limeshot-ui/src`) and generates `menus.md` with a per-menu summary and an "open work by feature" table.
3. **Roadmap:** 31 new feature records for menu items that had none (APP-026 main menu bar, APP-027…032, EXP-014…017,
   SEQ-021/022, UI-013…018, PLG-011, WF-009…013, REC-010/011, PR-028/029, PL-030, BR-011). `keybindings.json` gained 24
   menu shortcuts (KB-095…118) and six INFERRED bindings became OBSERVED.

Result: of FL's 268 menu items LimeShot has 12 as in FL and 35 in another form; 194 are not started and 27 are not
planned (Image-Line services). No code changed. Verification: `scripts/features.py check` is clean.

## 2026-10-22 — Session 14: dialogs the main menus open

Second layer of the outside-in pass (owner's request): the windows and dialogs behind the main menu items.

1. **FL observed live** (synthetic input; `fl-studio-analysis.md` §19, `fl-dlg-*.png`): the settings window with all
   ten tabs (General scrolled through, collapsed sections expanded), the render dialog and its drop-downs (after the
   macOS save panel; closed without rendering), the Plugin Manager app, Set time signature…, Transpose… and the Welcome
   window. Nothing in FL was changed or saved.
2. **`settings.json`** grew from 28 to 188 records and gained `section`, `control` and `fl_default`; the 28 old
   records were refined in place (most INFERRED → OBSERVED, with FL's exact labels). A `dialog.*` prefix lets a record
   name an option of a LimeShot dialog (export range, bit depth); the check reads it from `dialogs.rs`.
3. **Links:** 18 menu records now say which settings page they open. New features EXP-018 (several formats / mono in
   one render) and APP-033 (project statistics and working time).

Result: LimeShot has 15 of the 188 settings, 2 partly; 156 are not started and 15 not planned. No code changed.
`scripts/features.py check` is clean.

## 2026-10-22 — Session 15: plugin window extras (presets, keyboard toggle, saved places)

Continuing the mandatory VST3 work (`next-steps.md` 4), FL's wrapper extras from the session 12 observation
(`fl-studio-analysis.md` §17).

1. **Presets (PLG-008).** `limeshot-plugins` reads a plugin's factory programs (the `kIsProgramChange` parameter;
   names from `IUnitInfo` program lists, else the parameter's text) and reads and writes `.vstpreset` files
   (`preset.rs`, Steinberg's documented chunk format) found under `<vendor>/<plugin>` in the platform's preset
   folders. The plugin window's tab row now ends like FL's title bar: a presets menu (factory programs, then files,
   then Save preset as…), Presets ◀ ▶ for channels (wrapping) and the keyboard toggle. Lime Test Tone gained three
   factory programs for tests. `vst3_probe` prints the programs: Elephant, Quiet Piano 2 Lite, M1, OTT, Guitar Rig
   7 and the Arturia plugins have them (Arturia's only as numbered slots).
2. **Keyboard toggle.** When on, characters typed in the plugin window go to the editor first
   (`IPlugView::onKeyDown`); LimeShot's shortcuts act only when the plugin did not take them.
3. **Window places in the project.** `PluginRef.window` (position, tab, Settings size, keyboard toggle, open) is
   written when saving, like the plugin state (not an edit), sanitised on load, and windows that were open reopen
   with the project.

Verification: `cargo test --workspace` passes (new: preset parsing and folder search, factory programs, `.vstpreset`
round trip through a plugin, the window record's JSON and sanitising, and a UI test that steps presets with ◀ ▶,
picks one from the menu, flips the keyboard toggle, saves, reopens and finds the window open on its tab); Clippy
(`-D warnings`), `cargo fmt --check` and `scripts/features.py check` are clean. Looked at with
`ui_shot --plugin --plugin-window`. Not done: a live pass with a real plugin's window (PLG-008 stays IMPLEMENTED).

### Session 15, part 2: plugin parameter automation (PLG-007)

1. **Model.** `AutomationTarget::PluginParam { plugin, id }` (a channel's or insert's plugin and its VST3 parameter
   id, normalised values); `AddAutomationClip` gained an optional starting value and name for targets whose value the
   project does not hold. `Project::plugin(target)` finds the plugin a channel or slot runs.
2. **Engine.** New lane targets for channel and insert plugins. `apply_controls` (every 64 frames in Song mode) hands
   each value to the plugin processor with its frame offset in the block (`Instrument`/`Effect::set_plugin_param`);
   `Vst3Processor::set_param_at` queues them without allocating and adds them to the plugin's parameter queues in the
   chunk they fall in. Offline renders use the same path.
3. **UI.** Right-click a plugin parameter → Create automation clip; FL's Add › Automation for last tweaked parameter
   (plugin parameters changed in the editor or the list). While playing in Song mode the plugin's own controls follow
   the curve (`Vst3Instance::show_param`, controller only).
4. **Bug fixed on the way.** A right-click on any egui slider (effect cards, plugin parameters) never opened the
   automation menu, because sliders sense only drags, and it moved the value. `widgets::clickable` adds click sensing
   for the menu and `user_changed` ignores right-button changes.

Verification: engine test with logging processors at block sizes 64, 333 and 1024 (values at their frames, a point
at least every 64 frames, channel and insert plugins), a host test through the real VST3 path (Lime Test Gain), UI
tests for the menu, the last-tweaked item, the plugin's control following the curve, and the effect slider's
right-click. `cargo test --workspace`, Clippy (`-D warnings`, also `x86_64-pc-windows-msvc`), `cargo fmt --check` and
`scripts/features.py check` pass. Not done: listening to automation on a commercial plugin in the live app.

### Session 15, part 3: plugin delay compensation (PLG-009, MIX-012)

Processors report their latency through the DSP traits (`Effect`/`Instrument::latency`; a VST3 processor returns
what the control thread last read, refreshed on kLatencyChanged). The renderer plans compensation itself
(`engine/src/pdc.rs`) whenever the snapshot or a latency changes: each track's arrival latency is the latest of its
inputs, its voices and earlier plugin channels are delayed to match, and routes read the source track's output ring
at their own tap. 64 rings of 16 384 frames are allocated with the renderer; planning keeps rings already in use, so
edits do not interrupt delayed audio. Exports drop the leading delay (FL's Trim PDC silence) and render that much
longer; the Audio settings show the total. Verification: an engine test where a 1000-frame look-ahead insert, a
300-frame instrument latency, a plain track and a track routed through the latent one all land on frame 24 000 at
block sizes 64, 333 and 1024 (without compensation they land on 24 000, 24 300 and 25 000); the full suite, Clippy
(also for Windows) and fmt pass. Not done: listening with a commercial latent plugin.

### Session 15, part 4: plugin crash isolation (PLG-010, D34)

A plugin can now run in a process of its own ("Run in a separate process" under its name in Channel Settings or the
insert card; FL Studio's bridged plugins). `limeshot-plugins/src/bridge.rs` starts LimeShot with
`--plugin-bridge <socket>`; the child serves control calls on one Unix-socket connection (main thread) and audio
blocks on another (second thread). `PluginInstance`/`PluginProcessor` (`hosted.rs`) put in-process and bridged
plugins behind one interface for the UI and engine. A broken connection or a block that takes over 500 ms marks the
plugin crashed: silence, a one-time toast, "The plugin crashed" with Reload in its settings. Switching processes keeps
the state and reopens the window; Reload re-syncs the engine although the project did not change.
Verification: bridge tests (processing, parameters, state, automation points, programs, a killed and a stopped child),
a UI test (checkbox, kill, toast, notice, Reload, undo), and `examples/bridge_probe` with Vital, Surge XT, OTT and
Elephant (sound, latency, state; about 0.1 ms per 512-frame block). The full suite, Clippy (also for Windows, where
the bridge is left out) and fmt pass.

### Session 15, part 5: Patterns menu commands (SEQ-021, APP-026)

FL's Patterns menu, observed live where unclear (Split by channel, Insert one; `fl-studio-analysis.md` §18):
Find first/next empty with FL's name prompt (Shift+F4, F4; Cmd+F4 without naming), Select in playlist, Random color,
Transpose…, Insert one, Clone and Delete with FL's keys, Move up/down, Split by channel. New commands
`InsertPattern`, `MovePattern`, `SplitPatternByChannel`; default pattern names follow their position as FL's unnamed
patterns do. `menus.json` (12 items) and `keybindings.json` (8 bindings) updated. Tests: two core tests and a UI test
driving the menu and keys; the old F4 test now expects the name prompt.

### Session 15, part 6: Edit menu (APP-026)

FL's Edit menu, observed again: Cmd+Z steps back through the history and Redo is its own item on Cmd+Opt+Z, shown
only while something can be redone (session 13's "single slot" reading is corrected in `fl-studio-analysis.md` §18).
LimeShot's Edit menu now matches: "Undo <action>", "Redo <action>" when available (Cmd+Alt+Z added; Cmd+Shift+Z and
Cmd+Y stay), and Cut / Copy / Paste routed to the focused editor through one `clipboard(app, op)` per editor (the
editors' Cmd+X/C/V use the same functions), disabled when there is nothing to act on. UI test added.

### Session 15, part 7: Tools menu and section headers (APP-026, WF-009)

A Tools menu between Options and Help: Browser find… (shows the Browser, keyboard in its search field), Last tweaked
› Create automation clip, and Macros: Stop sound, Select unused channels, Purge unused audio clips, Reset empty
playlist tracks, Set keyboard focus for all plugins to › Enabled / Disabled. FL's grey section headers now appear in
File (Recent projects, listed inline as in FL), View (Windows, Layout, Zoom), Options (System, Switches) and Tools.
`ui_shot --menu=<name>` opens a menu for a screenshot. UI test for the macros.

### Session 15, part 8: View › Layout (UI-013)

FL's Layout section in View: Close all windows (F12), Close all plugin windows (Opt+F12), Close all unfocused windows
(Cmd+F12), Align all channel editors (Shift+F12, stacks plugin windows), and Arrange windows › Realign, Into
workspace, Desktop (default), Custom and Save custom layout (kept in settings). Also: headless apps no longer write
the user's settings (`persist_settings`; before, a test that saved a project added it to the real recent list).
UI test for the commands and a saved layout.

### Session 15, part 9: FL's settings window (APP-025, APP-017)

`settings_window.rs`: one window, "Settings – <page title>", with FL's tab groups (MIDI, Audio, General, File, Theme |
Project, Info | About; MIDI and Theme disabled for now) and titled sections; changes apply at once. Audio (device,
rate, buffer, status with plugin delay compensation), General (UI scaling, delete confirmation), File (autosave,
Browser folders, recent files), Project (time signature), Info (title, author, genre, comments, web link, statistics;
new `Project::info` and `SetProjectInfo`, one undo step per field edit) and About. Options gained General settings,
File settings, Project general settings and Project info (F11); the old Audio settings and About dialogs are gone.
`ui_shot --prefs=<page>` shows a page. UI test added.

### Session 15, part 10: FL's toolbar documented (next-steps Then 0)

Third outside-in layer: FL's two toolbar rows, every control named from FL's own hint panel (hovered one by one) and
its right-click menus opened (`fl-studio-analysis.md` §20, `fl-toolbar-*.png`). Recorded in `menus.json` as the
pseudo-menu "Toolbar" (54 records; new kinds control and display), so `menus.md` shows the toolbar's open work with
the menus'. LimeShot has 9 of FL's 52 toolbar items as in FL and 9 differently, plus 2 of its own; 31 are not started, 3 not planned.
A macOS screen-capture permission prompt appeared during the capture and was left for the owner.

### Session 15, part 11: toolbar menus from the research (TR-012 and toolbar records)

From the toolbar capture: PAT/SONG's right-click shows the Channel Rack or Playlist; the Piano Roll and Playlist icons'
right-click maximises them; the tempo field's menu has the tempo tapper (also View › Tempo tapper, Cmd+F5) and BPM
presets; the song position's menu picks which format is large; the CPU panel's menu sets the refresh rate and shows
the voice count; the snap list has FL's values in FL's order (1/6 step, 1/4, 1/3, 1/6 beat added; "(none)").
UI test plus the tapper's arithmetic.

### Session 15, part 12: editor menus documented (Channel Rack, Piano Roll)

Fourth outside-in layer, first half: the Channel Rack's options and channel menus and the Piano Roll's ▸ menu with
all its submenus, recorded in `menus.json` under the pseudo-menus "Channel Rack" (74) and "Piano Roll" (137), with
LimeShot's equivalent per item (often a key where LimeShot has no menu). `fl-studio-analysis.md` §21. Playlist and
Mixer follow.

### Session 15, part 13: editor menus documented (Playlist, Mixer); where to continue

The Playlist's ▸ menu with its submenus and the Mixer's menu and effect-slot menu are recorded as well (139 and 44
records; `fl-studio-analysis.md` §21, `fl-playlist-*.png`, `fl-mixer-*.png`). `menus.json` now holds 725 records:
main menus, toolbar and the four editors. Nothing was saved in FL.

**Where to continue (session 16):** implement from the editor menus, highest value first: Piano Roll menu with
FL's Tools (Quick legato, Arpeggiate, Strum, Randomize, Chop…), Select and Zoom commands; Playlist Edit (Merge
clips, Mute/Unmute Opt+M), Time markers and Zoom; Mixer track menu (Move left/right, Reset, colours). Then the
remaining "Then" items in `next-steps.md` (live FL observation list, keybindings/settings follow-ups, Mixer
pre-fader sends, effects polish, MIDI input). Open checks: a live pass with real plugins over presets, automation,
delay compensation and the plugin bridge (sessions 12–15), and SMPTE / FL's Line and Cell snap modes.


## 2026-10-22 — Session 16: implementing the editor menus

### Session 16, part 1: the Piano Roll's ⏵ menu (PR-015, PR-002, PR-005, PR-014)

FL's Piano Roll menu from the session 15 inventory: a ⏵ button at the left of the Piano Roll header opens Edit (the
existing editing keys as items, plus Discard lengths, Shift+D), Tools (Quick legato Cmd+L, Quick quantize, Quick
quantize start times Shift+Q, Quick chop, Glue), Select (one / more at random Shift+R / Shift+M, odd Shift+O, muted,
overlapping, stacked, invert), Zoom (PgUp / PgDn, Shift+1 … 6) and Center (Shift+0). Menu items and keys run the same
`piano_menu::Act`s; the Piano Roll's key handling is one ordered list (egui matches shortcuts with extra Shift/Alt,
so Shift+D must win over the D tool). Shift+digits match the key's position, so they work on QWERTZ. New core
`note_tools` (pure tools, a seeded RNG) and `Command::EditNotes` (replace notes keeping ids; merges inside a gesture,
for the coming dialogs' live preview). `ui_shot --menu=⏵/Tools` shows a submenu. egui's font has no "▸", hence ⏵.

### Session 16, part 2: the Piano Roll's tool dialogs (PR-015, PR-014, PR-023)

FL's strumizer, arpeggiator, flamer, limiter/transposer, randomizer, level scaling and claw machine observed live
(`fl-studio-analysis.md` §22, `fl-piano-tool-*.png`). LimeShot now has Articulate… (Alt+L), Quantize… (Alt+Q, also
the header's Quantize button), Chop… (Alt+U), Arpeggiate… (Alt+A), Strum… (Alt+S), Flam… (Alt+F), Limit… (Alt+K),
Flip… (Alt+Y), Randomize… (Alt+R), Scale levels… (Alt+X) and LFO… (Alt+O) in the ⏵ menu's Tools. `piano_tools.rs`:
a dialog previews on the notes while open (each change undoes the last preview and applies the new one), Accept
keeps one undo step, × / Esc reverts; if something else changes the project meanwhile, the dialog closes and keeps
it. The transforms are pure functions in `note_tools` with tests. `ui_shot --tool=strum` shows a dialog.
Not done: Claw machine, FL's arpeggio presets and level knobs beyond velocity/pan.

### Session 16, part 3: the Playlist's ⏵ menu (PL-006, PL-008, PL-013, PL-016, PL-022)

FL's Playlist menu from the inventory: a ⏵ button at the left of the Playlist header with Edit (the clip keys as
items, Make unique Shift+Cmd+V, Mute / Unmute Alt+M / Shift+Alt+M, Insert space Cmd+Insert, Slice & insert space
Cmd+Alt+Insert, Delete space Cmd+Delete, Merge selected clips Cmd+G), Tools (Quick quantize start times Shift+Q),
Select (by selected source Shift+C, muted, overlapping, stacked, invert, time around selection Cmd+Enter), Zoom
(PgUp / PgDn, Shift+1 … 5), Time markers (Add one…, Add auto, Delete) and Center. New core commands `InsertSpace`,
`DeleteSpace` (clips cut at the edges, markers move) and `MergeClips` (a new pattern with the clips' notes where they
sounded). `playlist_menu.rs` mirrors `piano_menu.rs`; the Playlist checks the menu's keys before its own.

### Session 16, part 4: the Mixer's track menu (MIX-006, MIX-007)

FL's Mixer menu: a ⏵ in the track panel's header (and on a strip name's right-click) with Rename… (F2), Change
color (swatches) and Random color, Reset selected track(s) to default, Reset routing, Unroute tracks routed to the
selected track, Solo (S), Move left / right (Alt+←/→), Delete (Delete) and Insert one. New core commands
`MoveMixerTrack`, `InsertMixerTrack`, `RemoveMixerTrack`, `ResetMixerTrack`, `ResetMixerRouting`,
`UnrouteMixerTrack`, `SetMixerTrackColor` and `MixerTrack::color`; `remap_mixer` renumbers channels' tracks, routes,
automation targets (`AutomationTarget::remap_mixer`) and compressor sidechains, and default "Insert N" names. The
Mixer now has keys when focused (`mixer_menu::keys`, not while typing a name).

### Session 16, part 5: the Channel Rack's options menu (SEQ-007, SEQ-003)

A ⏵ at the left of the Channel Rack header with FL's options-menu commands on the selected channels: Add one,
Clone selected (Alt+C), Delete selected…, Move selected up / down, Sort by (name, type, colour; new
`Command::ReorderChannels`), Group selected…, Color selected, Random color (also in each channel's menu), Mute /
Unmute selected, Transpose selected… (the Transpose dialog now takes channels), Assign selected to free mixer
track(s), Select mixer tracks for the selected channels. Not done: Zip / Unzip, layers, swing mix per channel,
display options (button width, name alignment …), declicking and plugin-editor switches.

### Session 16, part 6: Piano Roll stamps and scale highlighting (PR-027, PR-013; next-steps Then 10)

⏵ menu › Stamp: 18 chords and the scales from `note_tools` (`CHORDS`, `Stamp`, `stamp_intervals`); with a stamp
chosen, a Draw click places it from the clicked key (one undo step, selected and draggable), and the header shows
"Stamp: <name> ×". ⏵ › View › Scale highlighting: key and scale; off-scale rows are dark, the root's tinted.
`ui_shot --scale` shows both.

### Session 16, part 7: delay tempo sync (FX-008; next-steps Then 7)

Lime Delay has a Sync parameter (Off, 1/16 … 1/1, with triplet and dotted values): `EffectSlot::resolved_params`
turns it into a time from the tempo when the sequencer compiles the mix, so tempo changes follow; the stored Time
stays and is greyed out in the effect panel. Choices with more than four labels show as a drop-down. Older projects
load with Sync off (sanitize pads parameters).


### Session 16, part 8: pre-fader sends (MIX-008; next-steps Then 6)

`Route::pre` (saved only when set) and `Command::SetRoutePre`; the track panel's Output rows have a PRE toggle. The
engine now records each track's pre-fader signal for delay compensation and applies the fader and pan per route
(`Pdc::route` takes left/right gains), then scales the bus in place so meters and sidechain keys still read the
post-fader signal. A muted track sends nothing, pre or post. Engine test with the fader at half and at zero.

### Session 16, part 9: presets for built-in effects and instruments (FX-014, INS-010; next-steps Then 7)

`limeshot_project::preset`: `.limepreset` JSON files in `<config>/presets/<Effect|LimeSynth|Sampler>/`, factory
presets in code (by parameter name), `list` / `open` / `save` / `load` with sanitising and size limits. Effect cards
and Channel Settings have a Presets menu (factory, then the user's, then Save preset as… through the name dialog).
`Command::SetEffectParams` makes loading one undo step; a sampler preset keeps the channel's sample. Headless apps
have no presets folder (`presets_dir`), so tests use a temporary one. Test lesson: an egui popup's first frame is a
sizing pass, so a test must step twice after opening a menu before clicking in it.


## 2026-10-22 — Session 17: owner interaction fixes

* Drawn/moved notes now use an audio-clock `NoteFor` deadline instead of the mouse lifetime. Piano keys and scrub
  notes retain explicit note-off handling. The timed plugin audition test verifies exact release at block sizes
  1, 64, 333 and 1024 while transport is stopped.
* Piano Roll right-click background deselects; right-delete paints a red delete cursor in both editors. Rack
  selector drags select crossed rows without moving the window; four-to-eight-beat width bounds are independent
  of MIDI length and preserve horizontal scrolling. Mixer meters/whitespace select strips; control drags do not.
  Effect presets/reorder buttons use separate rows. Export, tapper, rename and quit dialogs have bounded widths.
* Sample drops show waveform ghosts and a pointer waveform. Sampler preview clicks play the whole trimmed/pitched
  region; a playback head follows previews and sequencing, and dragging the waveform drops the sample. Picker
  patterns/audio/automation drag onto the grid. Browser channels have dragging, double-click and context actions.
* Shift-click locks channel/Playlist mute states. Playlist headers expose supported FL track actions, including
  color, solo, clip selection/merge/mute, clone, reorder, insert and delete; unsupported advanced actions are disabled.
  Clone preserves clip offsets, mute/lock states and linked sources. + Track follows the last track.
* Background toolbar/F-key windows raise before toggling closed; pinned windows stay above. Typing keyboard input
  targets the selected channel (Z–M / Q–U), releases the original channel on key-up after selection changes, and
  stops on lost focus/text entry. The toolbar button, Options menu and Cmd+T toggle it; Playlist Cmd+T still adds a marker.
* Automation points can be placed away from the line, with custom placement/move/tension cursors. Rotary defaults
  have an illuminated marker. Stop while stopped rewinds to zero or the selected range start.
* Plugin wrappers are resizable internal egui windows (D35), sharing the main NSWindow. Embedded resize requests
  never resize the OS window. Fixed-size native views keep natural dimensions/minimum wrapper size; resizable
  plugins negotiate onSize. Both tabs, presets, window state and keyboard forwarding are retained.

Validation: `cargo build -p limeshot-app`; `cargo test --workspace` (111 UI tests, plus core track edit/clone and
engine audition regressions); `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --all`;
`scripts/features.py check`. The bridge tests need sandbox escalation for local Unix sockets. UI rendered and
inspected: Mixer/FX controls, export/tapper and embedded wrapper parameter fallback; `ui_shot` now supports
`--dialog=export|quit|marker` and `--tapper` for repeatable layout review.

**Not verified:** final live native plugin embedding. Vital appeared inside the main window in the first pass,
but subsequent per-window captures showed a blank editor. The app-control tool stalled repeatedly; the terminal
fallback reports Accessibility unavailable (`AXIsProcessTrusted() == false`), so activation/input/resize could not
be validated. PLG-006 is IMPLEMENTED pending that pass, not VERIFIED. The advanced FL track menu features listed
in `known-limitations.md` remain disabled. No CI/remote changes or commits were made; the pre-existing `.claude/`
directory was left untouched.


## 2026-10-22 — Session 17 documentation and PR preparation

Audited the current-facing documentation against the source and feature inventory. Updated README, agent guidance,
architecture, milestones, screenshots guidance and known limitations; reduced next steps to open verification and
implementation tasks. Marked START.md as the original brief, preserving its requirements and historical research.
Regenerated feature/menu/shortcut/setting views and checked every relative Markdown file link. Removed generated
Python caches and ignored them plus local `.claude` tooling; no tracked scratch files were found. Retained fixtures,
reference screenshots, licenses and chronological evidence.

Final checks passed: `cargo test --workspace` (including 111 headless UI tests and Playlist track model tests),
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, formatting, inventory validation
and `git diff --check`. Earlier Session 17 offscreen reviews cover editor/dialog layout. Native plugin embedding
still requires live verification; the PR must retain this limitation and the disabled advanced Playlist track actions.


## 2026-10-22 — Session 17 CI regression fixes

The owner's macOS CI log exposed a timing assumption in
`playlist_time_selection_loops_song_playback`: a two-bar selection at 124 BPM needs about 3.87 seconds of audio
progress, while the test allowed only 4.5 seconds of wall time. The silent fallback can advance more slowly under
runner scheduling pressure. The UI test now locates a quarter beat before the selection end and waits for the
observed wrap with the existing bounded polling helper, checking every observed position stays in the selection
and playback continues with song looping disabled. The deterministic engine test still verifies full loops and
onsets at five buffer sizes; no production playback behavior changed.

Also inspected the completed Windows x86-64 job in CI run 39809093123. Its plugin-discovery assertion compared
`Vendor\B.VST3` with `Vendor/B.VST3` as strings. It now compares relative `PathBuf` values, retaining exact bundle,
ordering and nested-bundle exclusion checks without assuming a path separator. CI jobs now run; updated the old
pending-job continuation note. Linux ARM64 and Windows ARM64 cross-build jobs succeeded in that run; this does not
establish live GUI/audio verification on those platforms.

Validation: the Playlist test passed 30 consecutive runs; `cargo test --workspace` passed all 300 tests. After the
Windows assertion edit, all seven plugin library tests passed; workspace Clippy, rustfmt and inventory checks pass.
Remote macOS/Windows reruns are required to confirm the fixes on their runners.


## 2026-10-22 — Session 17 hint-panel CI regression

The Windows CI hint-panel test asserted "Stopped · Pattern" before the demo's sample loader completed. Showing
"Loading 3 samples…" was correct application behavior. The hover/transport test now starts with an empty project:
its toolbar and Playlist hint checks do not need any samples. The stopped and playing assertions remain exact;
no production status behavior changed and no sleep or retry was added to hide the loading state.

Validation: the focused test passed 30 consecutive runs, all 111 headless UI tests passed, and workspace Clippy,
rustfmt, inventory validation and `git diff --check` passed. Remote Windows confirmation remains pending.
### Session 16, part 10: automated controls follow the song (AUT-013; next-steps Then 8)

From here on session 16 works in a separate worktree (`../limeshot-s16`, branch `s16-automation`, pushed to `main`):
the main checkout holds a Codex session's uncommitted changes, which go in through their own PR.
`LimeShotApp::automated(target)` evaluates the snapshot's automation lane at the song position while the song plays;
the Channel Rack's volume/pan knobs, the Mixer's faders, pan knobs and track panel sliders, and built-in effect
parameters display it. Stored values are untouched.

### Session 16, part 11: automation point modes (AUT-012; next-steps Then 8)

`AutomationPoint::mode` (`PointMode`: single and double curve, hold, stairs, smooth stairs, pulse, wave; saved only
when set) and `automation::segment`. A point's right-click in the Playlist opens FL's point menu (Delete and the
modes) instead of deleting at once. Automation lanes now keep the automation's own points and each clip's
start/end/offset (`LaneClip`) and evaluate the curve directly, so stepped and periodic segments stay in place when a
clip starts mid-curve. `ui_shot --automation` shows stairs and a wave.

### Session 16, part 12: MIDI input (WF-003, APP-020; next-steps Then 9)

`limeshot_audio::midi` (midir 0.11, MIT): `parse` (notes, velocity-0 note-offs, controllers, pitch bend, any MIDI
channel), `input_ports`, and `MidiIn` (messages parsed on midir's thread, sent over a channel, with a waker so the
UI repaints at once). Settings › MIDI (enabled; Options › MIDI settings opens it): the inputs as radio buttons,
Link note on velocity to, Refresh device list; `settings.midi_input` / `midi_velocity`. `LimeShotApp::midi_event`
plays the selected channel and remembers each held note's channel. A unit test round-trips a note through a
virtual CoreMIDI port. Next: MIDI recording into the Piano Roll (PR-021 / REC-002).

### Session 16, part 13: note recording (REC-002, PR-021; next-steps Then 9)

`recording.rs`: the toolbar's Record button (R) arms; while playing, MIDI input and typing-keyboard notes become
notes of the current pattern (`Command::EditNotes` with `NoteTool::Record`; the take's step is replaced as notes
arrive, so a take is one undo step unless another edit comes in between). Right-click: FL's record menu with
Recording starts playback, Disarm on stop and start/end quantize. Song mode records into the current pattern's clip
under the song position. `ui_shot --record` shows the armed button.

### Session 16, part 14: macOS packaging (PF-006; next-steps Then 12)

`scripts/package-macos.sh`: release build, `dist/LimeShot.app` (both binaries, samples in `Contents/Resources`,
`LimeShot.icns` from the new `make_icon` example, Info.plist with the `.lime` document type), ad-hoc `codesign`, and
`dist/LimeShot-<version>-macos-<arch>.dmg` with an Applications link. Checked: signature, plist and DMG verify; the
bundled CLI renders the demo with the bundled samples; the app launches from the bundle. `/dist` is ignored by git.

### Session 16, part 15: audio clip fades (PL-023; next-steps Then 8)

`Clip::fade_in` / `fade_out` (ticks, saved only when set) and `Command::SetClipFades` (clamped to the clip). The
sequencer passes them in seconds on the clip's `NoteEvent`; the engine gives the voice a `ClipFade` (frames from
the clip's start and to its end) that ramps its gain, including when playback chases into a clip. The Playlist
draws fade lines and two handles on audio clips; dragging a handle sets the fade (`Drag::Fade`). `ui_shot --audio`
shows a faded clip.

### Session 16, part 16: MP3, FLAC, Ogg Vorbis and AIFF samples (INS-008; next-steps Then 8)

`limeshot_dsp::load_audio` (WAV with hound, the rest with symphonia 0.5, decision D37), `is_audio_file` and
`AUDIO_EXTENSIONS`; the sample loader, the Browser (`browser::is_sample`) and the file pickers use them. Fixtures: a
0.1 s stereo sine as AIFF (afconvert) and FLAC (a hand-written verbatim encoder, as afconvert's FLAC output failed),
both checked sample for sample; truncated and junk files fail cleanly.

### Session 16, part 17: Lime Gate (FX-006)

`EffectKind::Gate` and `limeshot_dsp::effects::Gate` (threshold, attack, hold, release, range; a fast peak detector
drives a smoothed gain), wired into the engine's effect factory, the Add effect menu and factory presets. An engine
test now runs every built-in effect as an insert.

### Session 16, part 18: sampler loop points and filter (INS-007)

`SamplerParams` gains `looping`, `loop_start` / `loop_end` and `cutoff_hz` / `resonance` (defaults keep old projects
unchanged). The voice wraps round the loop (forwards or reversed) and runs a stereo state-variable low-pass when the
filter is not open; a looping one-shot follows the note length. Channel Settings has Loop and Filter sections and
shades the loop on the waveform.

### Session 16, part 19: audio recording (REC-001, REC-003)

`limeshot_audio::record`: `InputRecorder` (cpal input stream → rtrb ring → writer thread → 32-bit float WAV;
dropped frames counted, never blocking the callback), `list_input_devices`, `Take`. The record menu's Audio filter
turns it on; `recording::tick` starts it when playback starts while armed and stops it with the transport;
`place_take` puts the take on the first free Playlist track at its start. Settings › Audio has an Input choice
(`settings.audio_input`); a level bar shows under the record button while recording; the macOS package declares
microphone use. Not tried with a real input (it needs the owner's microphone permission): next session, record a few
seconds from the built-in microphone and check the clip lines up.

### Session 16: where to continue

Done in session 16 (all on `main`): the four editor menus (Piano Roll with tool dialogs, stamps and scale
highlighting; Playlist; Mixer track menu; Channel Rack options), delay tempo sync, pre-fader sends, effect and
instrument presets, automated controls following the song, automation point modes, MIDI input and note
recording, audio recording, macOS packaging, audio clip fades, MP3/FLAC/Ogg/AIFF samples, Lime Gate, sampler loop
and filter. Work happens in a worktree (`../limeshot-s16`) rebased onto `origin/main` before each push, because
other sessions (Codex) also change this repository.

Next, in order:
1. Live checks the owner should see or that need permissions: audio recording from a real input (microphone
   permission), MIDI with a hardware keyboard, the plugin pass over presets/automation/PDC/bridge (sessions 12–15),
   the packaged app on a second Mac (needs Developer ID signing).
2. Remaining FL editor-menu work (`menus.md`): Piano Roll Time markers, Target control, Group; Playlist picker panel
   options; Mixer groups; Channel Rack Zip / Unzip.
3. `next-steps.md` Then items still open: tempo automation (AUT-008, needs a tempo map in `Timeline`), export ranges
   (EXP-010), more effects (FX-009 distortion, FX-010 chorus/flanger/phaser), LimeSynth filter modes (INS-013),
   sampler volume envelope (INS-014), CLAP/AU hosting.

### Session 16, part 20: export the time selection (EXP-010)

`ExportRange::Selection { from, to }`: the offline render plays Song mode from `from` for exactly the selection's
frames (clips already under way are chased), no loop. The export dialog offers Time selection when the Playlist
has one and starts on it, as FL Studio does.

### Session 16, part 21: Lime Drive (FX-009)

`EffectKind::Drive` / `effects::Drive`: soft clip, hard clip or wave folding after up to 36 dB of drive, a one-pole
tone low-pass, mix and output; factory presets. No oversampling yet (noted in the limitations).

### Session 16, part 22: Lime Chorus — chorus, flanger, phaser (FX-010)

`EffectKind::Chorus` / `effects::Chorus` with a Mode choice: modulated delay (chorus, flanger with feedback) or a
six-stage swept all-pass phaser; rate, depth, feedback, spread and mix; factory presets.

### Session 16, part 23: sampler volume envelope (INS-014)

`SamplerParams::attack_ms / decay_ms / sustain` feed the sampler voice's ADSR (it used only the release before);
Channel Settings shows them above Release.


### 2026-10-22 — Codex continuation: finish sampler volume envelope (INS-014)

Resumed Claude's uncommitted part 23 in `/Users/sebastian/Developer/limeshot-s16` after its usage limit.
Completed load sanitization for the new stages, backward-compatible defaults and serialization checks; exercised
attack/decay/sustain at block sizes 1, 64, 333 and 1024. Added a headless UI regression covering each new control's
reset and undo. Rendered and inspected Channel Settings; limited Attack/Decay readouts to one decimal so zero
values fit the window. Updated architecture and the sampler's known limitations.

Validation: `cargo test --workspace` (including 120 headless UI tests), workspace Clippy with warnings denied,
`cargo fmt --all --check`, inventory validation and `git diff --check`. Offscreen UI:
`cargo run -p limeshot-app --example ui_shot -- /tmp/limeshot-sampler-envelope.png --settings`.
Delay/hold stages and envelopes for pan, cutoff and pitch remain open; native plugin embedding remains unverified.


### 2026-10-22 — Codex continuation: platform CI audit

Read the current runner jobs rather than treating cancelled workflow summaries as failed tests. Windows x86-64,
Linux x86-64/ARM64, Windows ARM64 cross-build and an earlier Apple Silicon job passed; exact commits and job links
are in `platform-validation.md`. Updated the stale platform feature records and current next steps. Intel jobs
had no steps on the retired `macos-13` label, so switched that matrix entry to `macos-15-intel` following GitHub's
retirement notice. Remote confirmation of the replacement is pending. No native GUI/audio-device claims added.
Live UI verification was deferred at the owner's request while the Mac was locked.


### 2026-10-22 — Codex continuation: control value context menus (UI-010)

Channel/mixer volume and pan and built-in effect parameters now offer Reset, Copy value, Paste value and
Type in value… alongside Create automation clip. The independent value clipboard stores normalised control
position using the existing automation mappings. Typed values use percentages, mixer dB (-inf for silence) or
the effect's units. Invalid/non-finite/out-of-range input and Cancel do not edit; applying, resetting and pasting
use existing Commands and one undo step each. Controller linking, event editing, Init song and host value menus
for instrument/plugin parameters remain open; UI-010 remains IN_PROGRESS.

The context-menu tests exposed that an egui slider's display rounding could edit the project on mere hover.
`user_changed` now requires input as well as a changed/interactive response. Tests cover copy without edits,
normalised paste across pan/fader scales, undo, reset, typing validation, cancellation and deleted targets.

Validation: `cargo test --workspace` (329 tests, including 122 headless UI tests), workspace Clippy with warnings
denied, workspace build, rustfmt, inventory check (730 menus) and `git diff --check`. Rendered and inspected the
new dialog with `ui_shot --mixer --dialog=control` (`/tmp/limeshot-control-value.png`). No live UI checks performed.


### 2026-10-22 — Codex continuation: Auto scrolling (UI-016)

Added a saved `settings.auto_scroll` switch shared by Options and the toolbar's painter arrow icon. It is off by
default to preserve existing settings behavior. While enabled and playing, Playlist follows Song position and
Piano Roll follows Pattern position, paging only when the playhead leaves view; a return to the loop/locate start
brings the view back. Stopped transport and active editor drags leave the view alone. Thresholds are LimeShot's
own, not a claim about FL's exact scroll algorithm. No shortcut was added.

Tests cover both switches, no project edits, both editors' playback/return behavior, disabling follow and manual
inspection while stopped, plus settings save/load and old settings defaults. The playback tests locate outside the
viewport after sending the new pattern snapshot instead of depending on long wall-clock waits.

Validation: `cargo test --workspace` (331 tests, 124 headless UI tests), workspace Clippy with warnings denied,
rustfmt, inventory validation (189 settings, 730 menus) and `git diff --check`. Rendered and inspected the toolbar
and Options switch using `ui_shot --menu=Options --auto-scroll` (`/tmp/limeshot-auto-scroll.png`).


### 2026-10-22 — Codex new session: numbered saves and repository integrity

Continued the clean `s16-automation` worktree at `ec46e8f`; attached draft PR #3. Full dispatch CI run 39826217710
at `38d9a7b` has now passed all seven jobs, including native Intel macOS on `macos-15-intel`. Updated exact platform
job evidence; no GUI/audio-device or PLG-006 verification claims were added. The owner still defers live UI checks.

Completed APP-018: File › Save new version writes a numbered sibling, skips occupied files/directories and makes
it active after success. A numeric trailing suffix increments; unsaved projects use Save as. Uses a complete,
synced temporary file with no-clobber publishing, including concurrent saves. Original bytes, sample paths,
modified-state/error behavior and undo history are covered. No shortcut was assigned (Cmd+N remains New).
Rendered and inspected `/tmp/limeshot-save-version.png` with `ui_shot --menu=File`.

Audited tracked/untracked/ignored files, duplicate content, references, manifests, fixtures and attribution.
Removed an unused engine dev dependency; fixed stale current-facing docs and packaged missing font notices;
added the offline repository-reference check to CI. No tracked file deletion was justified. Scope, cleanup decisions,
checks and remaining questions are recorded in [repository-audit.md](repository-audit.md).

Validation: 335 workspace tests pass (125 headless UI tests), Clippy with warnings denied, all-target build,
rustfmt, inventories, repository-reference check and diff checks. Git object check passes. Rebuilt the macOS app
and DMG; signature/plist/image checks, packaged font-text comparison and bundled CLI demo/reload render comparison
pass. Native UI checks were not attempted. Continue with the latest PR CI and remaining roadmap priorities.

### 2026-10-22 — Separate continuation: Channel Rack Zip / Unzip (SEQ-013)

Started from pushed `origin/s16-automation` (`6fd1f41`) in a new managed worktree on
`codex/channel-rack-zip`; the primary checkout and PR #3's branch were left intact. PR #3 remained open when checked,
so this continuation targets `s16-automation` and depends on that PR. Its latest CI run `39830009498` was still in
progress; no live UI checks were attempted while the owner's Mac is locked.

Implemented Channel Rack › Zip selected (Alt+Z) and Unzip all (Alt+U), using Image-Line's documented actions and the
existing clean-room menu captures. Compact named buttons select channels (Cmd/Shift-click toggles multiselection);
right-click restores one row. Unzip all includes filtered-out channels. Zipping hides the graph for that row and
finishes a held step/graph edit gesture before hiding its surface. State follows stable channel IDs across patterns
and reordering, is pruned on deletion, and starts expanded for new/open projects. It is session-only; automatic
zipping of empty channels and saved zip state remain unavailable.

Three new headless regressions cover menu and shortcut actions, row spacing, selection, graph hiding, filtered rows,
deletion/undo, all rows compacted, new-project reset, unchanged project/revision and a held step stroke followed by
an independent undo step. SEQ-013 is VERIFIED for this scope. Updated shortcut help, canonical inventories,
architecture, research notes and limitations. `ui_shot --zip` renders a reproducible compact-row view; inspected
`/tmp/limeshot-rack-zip.png` offscreen.

Validation: 338 workspace tests pass, including 128 headless UI tests; workspace all-target Clippy with warnings
denied, all-target build, rustfmt, inventory/reference checks and `git diff --check`. Live plugin embedding remains
IMPLEMENTED, not VERIFIED. Continue remaining editor View/context-menu work and P1 production gaps according to
dependencies; native GUI/audio verification is still separate from CI.

### 2026-10-22 — New session: automatic empty-channel zip and session PR policy

Created managed worktree `rack-auto-zip/limeshot` from `origin/codex/channel-rack-zip` (`8488fae`) and new branch
`codex/rack-auto-zip`. PRs #4 and #3 were still open, so this session's new draft PR targets `codex/channel-rack-zip`
and depends on both. The primary checkout and previous continuation sources/branches were left intact. Recorded the
owner's standing policy in AGENTS.md and next-steps.md before implementation: every new development session must
create its own new branch and NEW draft PR, never append feature work to an earlier session's PR, and repeat this
policy in future handoffs before these docs merge. Do not merge automatically or create further sessions unless asked.

Completed the existing SEQ-013 / SET-089 automatic-zip follow-up. General › Miscellaneous › Auto zip empty channels
is saved in application settings and defaults off. Switching patterns replaces the compact set for all channels,
including filtered-out ones: no note data collapses; steps or melodic notes expand. Muted notes count as data.
Enabling waits for the next switch, and manual Zip/Unzip or right-click restoration remains effective until another
switch. Note edits do not continuously collapse a restored row. New/open projects start expanded. Deletion/undo
fallback pattern switches are handled too; held step/graph edits end before switching their surface.

Read Image-Line's General settings manual for the transition behavior; recorded unmeasured edge choices in the
research notes. Updated canonical feature/setting inventories, UI architecture and known limitations. Four new
headless tests cover the actual setting control, empty/step/melodic/muted-note patterns, filters, manual restoration,
deletion/undo fallbacks, new-project reset, unchanged project/revision and held-stroke undo separation. Settings
round-trip tests also cover the saved switch and old-file default. `ui_shot --auto-zip --prefs=General` produced
`/tmp/limeshot-auto-zip.png`, inspected offscreen.

Validation: 342 workspace tests pass, including 132 headless UI tests; all-target workspace Clippy with warnings
denied, all-target build, rustfmt, feature/shortcut/setting/menu inventories, repository-reference check and diff
checks pass. Reused the existing `limeshot-s16/target` build cache. Prior PR runs `39831714286` and `39830009498`
were still in progress when checked; no CI failure was observed. Live/native checks remain deferred while the Mac
is locked, and PLG-006 remains IMPLEMENTED, not VERIFIED. Continue current priorities in next-steps.md using a new
branch and NEW draft PR for each future development session.

### Session 16 continuation: tempo automation (AUT-008)

Branch `claude/s16-synth-tempo` (PR #7, first stacked on `s16-automation`, now targeting `main`). `AutomationTarget::Tempo` (60–180 BPM) and
`LaneTarget::Tempo`; the renderer's `retime()` (shared with project tempo changes) runs whenever the tempo lane's
value differs from the transport's, every automation interval in Song mode, so the position and scheduled note ends
carry over; `current_tempo()` and `frames_until()`. Offline export renders until the end tick is reached at the
playing tempo (re-read every block, stopping before Pattern mode wraps). The tempo field's right-click has Create
automation clip and shows the automated tempo while playing. Engine and UI tests added.

### Session 16 continuation: LimeSynth filter modes and key tracking (INS-013)

`limeshot_dsp::filter::Mode` and `Svf::process` (low-, band- and high-pass outputs of the TPT state-variable
filter); `SynthParams::filter_mode` / `key_track` (saved with defaults that keep old projects unchanged) feed
`SynthMods`; the voice adds `key_track × log2(f / C5)` octaves to the cutoff. Channel Settings: Type and Key track
rows in the Filter section.

## 2026-10-22 — Lime Native standalone VST3 suite (PLG-012)

The owner requested native Lime instruments/effects as separate VST plugins usable in other DAWs. Added the
isolated `native-plugins/` workspace: LimeSynth, Lime Sampler, Lime Utility, Lime EQ, Lime Filter, Lime Delay,
Lime Room, Lime Comp and Lime Limiter. All are self-contained VST3 bundles with statically linked existing
LimeShot DSP, stable IDs, host automation/state/bypass, resizable dark-and-lime native editors, factory presets
and .limepreset interchange. Sampler embeds bounded WAV audio in host state; Comp exposes a stereo key input;
Delay follows host tempo. Shared voice/envelope methods update held notes without retriggering, and Limiter now
reports real gain reduction through the shared DSP trait. No handwritten unsafe code or new graphic assets.

`build.py` collects bundles into `native-plugins/dist/` without installation. Added native-plugin CI for macOS,
Windows and Linux, license collection, a real exported-binary ABI verifier and reproducible offscreen editor
renders. Production editors were inspected and corrected for label/knob layout; regression tests assert every
control fits without overlap at the default and minimum sizes. Existing user changes and test-coverage audit
notes were retained.

Validation: nine release bundles built/ad-hoc signed on macOS arm64; actual VST3 ABI verification passed for all
nine at 44.1/48/96 kHz and buffers 1/64/333/1024, including native editor creation, MIDI onset, automation and state
recall. Repeated using debug plugins with realtime allocation assertions: all pass. Independent JUCE-based
pluginval 1.0.4 level 5 (seed 42) passed all nine, including Editor and Editor Automation, and passed the separate
GUI-skipped runs. Nine native unit/layout tests pass. Both workspaces' Clippy/rustfmt, the DAW's full tests/build,
feature inventories and `git diff --check` pass. Details and reproduction: `native-plugins/VALIDATION.md`.

PLG-012 is IMPLEMENTED: hands-on DAW/editor validation remains pending because the Mac was locked; Windows/Linux/
Intel runtime and remote CI results remain pending. The supplied local bundles are macOS Apple Silicon, not
universal/Windows/Linux binaries. No installed DAW files/settings or plugin folders were changed.

## 2026-10-22 — Distinct Lime Native editors and modern LimeShot finish (PLG-012)

Reviewed the official Image-Line manuals and interface screenshots for 3x Osc, Channel Sampler, Balance,
Stereo Enhancer, Parametric EQ 2, Filter, Delay 3, Reeverb 2, Compressor and Limiter. Original painter layouts
now give each of the nine plugins a distinct shape, control arrangement and accent. Research and clean-room
boundaries are recorded in `docs/research/lime-native-ui.md`; no FL Studio artwork or code is included.

Verified the FL-inspired layout revision first, then applied the owner's follow-up request for a cleaner,
modern LimeShot finish: the app's Lime Dark neutral tokens, flat dials with thin value arcs, quiet section
labels and subtle rounded panels. Sampler follows the built-in LimeShot/FL sampler organization with a large
waveform, draggable trim handles, file controls, playback settings and root-key/audition keyboard. EQ uses the
actual Eq3 response with draggable bands; Limiter shows measured input/output/reduction history. All displayed
controls use existing parameters or perform their labeled actions. Class IDs and parameter/state schemas remain
stable. Sample audition communicates with processing through an atomic request and preallocated voices.

Final validation on macOS arm64: 11 native tests passed, including every editor at its default/minimum sizes,
sampler pointer/audition requests and audio release at 1/64/333/1024-frame boundaries. All nine final production
editors rendered and visually inspected. Native workspace Clippy with warnings denied and rustfmt passed.
Rebuilt release and allocation-checked debug bundles; both passed actual VST3 ABI audio/MIDI timing,
automation, state recall and editor creation at 44.1/48/96 kHz × 1/64/333/1024 frames. All nine final release
bundles passed independent pluginval 1.0.4 strictness 5, seed 42, including Editor and Editor Automation.
Signatures and system-only dynamic dependencies verified; release hashes and logs accompany the package.

Hands-on DAW checks remain pending: computer-use discovery reconfirmed the development Mac is locked.
Windows/Linux/Intel runtime checks and notarization remain pending; PLG-012 remains IMPLEMENTED. No installed
DAW files, settings or plugin directories were changed. The prior DAW workspace's 300-test/build/Clippy pass is
recorded above; this UI revision did not change DAW workspace code.

### PR integration with current main

Prepared `codex/lime-native-vst3-suite` from `main` at `dd53fb3`, preserving its sampler loop/filter, new effects
and other production changes. Kept the original seven exported effect kinds in an explicit array and supplied
neutral defaults for the newer sampler fields without changing standalone plugin IDs/parameters. Resolved the
documentation append conflicts and renumbered the plugin-workspace decision D38. The unrelated local coverage
audit remains outside this PR. Fresh integration checks: all 11 native tests, 71 shared DSP/engine tests, native
all-target Clippy with warnings denied, both workspaces' formatting and feature inventories pass. Earlier
exported-binary results/hashes remain a pre-integration artifact baseline; platform CI builds/checks the PR.

### Session 16 continuation: PR #7 rebased onto main with the Lime Native suite

Merged `main` (PR #3 and the Lime Native VST3 suite, PR #6) into `claude/s16-synth-tempo`; documentation conflicts
kept both sides and the inventories were regenerated. The standalone LimeSynth VST3 now has the INS-013 controls:
Filter type and Key track are appended as controls 16–17 (the plain-control arrays grow to `MAX_CONTROLS` = 17), so
existing parameter IDs, host automation and saved states are unchanged and old states load with Low-pass and no key
tracking. They feed the shared `SynthMods`, round-trip through `.limepreset`, and the editor's Filter panel gains a
type selector and a Key track knob. Main's native build was broken by PR #3's sampler envelope fields; Lime Sampler
now passes a gate (attack 0, decay 0, sustain 1), and `Voice::update_instrument` applies the sampler's full ADSR.
Checks: DAW workspace fmt/clippy (warnings denied)/tests; native workspace fmt/clippy/12 tests including
`synth_filter_type_and_key_tracking_reach_the_voice` and the editor no-overlap layout test; offscreen LimeSynth
render inspected; inventories and repository links. Not re-run: exported-binary verifier, pluginval, hands-on DAW.

## 2026-10-23 — Lime VST3s replace the built-ins (PLG-013, in progress)

Owner decision: LimeShot hosts the Lime Native VST3s instead of its built-in instruments and effects (D39). Branch
`claude/lime-vst3-default`, draft PR stacked on #7. Added `limeshot_core::lime` (catalog, class ids, state writer in
the host's `LSV3` container, `convert_project`), `limeshot_plugins::lime` (`lime:` path resolution), a `Document`
normalizer installed by the app, and warning toasts for settings that cannot be carried over. Lime Sampler now takes
up a sample loaded with host state while active (nice-plug does not reactivate after a state load). Equivalence test:
a project with LimeSynth (band-pass, key tracking), Sampler (WAV, pitch, trim) and EQ renders bit-identically through
the real bundles (RMS difference 0). Added P0 records PLG-013…PLG-023. Remaining: delete the built-ins and move
their tests, ship the bundles in the package and CI, and the P0 gaps; see
[handoff-lime-vst3-replacement.md](handoff-lime-vst3-replacement.md).


### 2026-10-23 — hosted controls and headless conversion continuation

Continued on `claude/lime-vst3-default` / draft PR #8 at the owner's explicit instruction (the supplied takeover
prompt overrides the standing new-branch/new-PR session policy for this continuation only). No new chat/session
was created; no merge is authorized. Future sessions follow the standing AGENTS policy unless the owner overrides it.

Installed conversion in headless constructors, made rack/Browser add actions create Lime references, retained
sample loading and waveform audition in hosted Channel Settings, and taught Browser/Playlist source selection,
previews and clip reuse about Lime Sampler. Source hints save relative and reopen absolute; root-key edits update
sampler state. Removed the old synth/sampler and insert parameter panels. Shared the native parameter schema
with core; Lime parameter edits now write undoable state and merge per drag. Existing plugin instances follow
changed saved state. Legacy preset menus work on hosted Lime state. Offline loading failures abort CLI/GUI export.

Added P0 records PLG-024…PLG-028 for audio-clip playback, note pan, full host typed-value menus, host dynamics
meters and plugin voice statistics. The built-in engine paths remain: PLG-013 is IN_PROGRESS. Native rendering/tab checks and measured audio-device output were subsequently exercised as recorded below.

Validation: all 134 headless UI tests pass, as does the complete DAW workspace suite; 13 native tests pass.
Non-automated legacy-vs-VST3 rendering now asserts exact sample bits and trailing silence, and passes (RMS
error 0). Automated EQ comparison remains close, as documented. Both workspaces' formatting and Clippy pass.
Offscreen hosted Sampler wrapper was rendered and visually inspected. Packaging and binary verification results
are recorded in the handoff status after completion. GitHub Actions remains unavailable due account billing.

Live follow-up: reproduced and fixed macOS host rendering going black after native editor attachment. A main
viewport OpenGL context guard restores the host context around UI callbacks. Lime Sampler and its host render
together after the fix, including a switch to Settings. The packaged demo renders at −3.7 dBFS; realtime converted
playback produces 324 device callbacks over 2 s at 44.1 kHz, peak −5.3 dBFS. Native binary verification passes all
nine bundles across three sample rates/four buffer sizes; app/DMG packaging and strict deep ad-hoc signature check
pass. CLI save/import/play paths now convert too. Broader native interaction, remote CI and pluginval remain open.
Details: [lime-vst3-host-validation.md](lime-vst3-host-validation.md).

### 2026-10-23 — native-only playback replacement finished

Continued at the owner's explicit request on `claude/lime-vst3-default` / draft PR #8, stacked on unmerged #7.
Removed synth/sampler playback states and channel voice construction, host effect factories/adoption and the
independent track EQ. Shared voice DSP moved to `limeshot-dsp`; native plugins no longer link the host engine or
sequencer. Test-only reference processors exercise the real plugin rack and retain exact real-binary equivalence.
The app MVP/demo now renders through actual hosted processors. Native instruments keep 128 voices per instance;
hosted channel automation preserves interval timing with preallocated gains. Test Gain/Tone are unchanged.

Format 2 reads version 1 without losing legacy/unknown data. Gate/Drive/Chorus are preserved disabled and warn.
The old track-EQ settings survive inactive with disabled controls; PLG-029 records exact response restoration as P0.
Silent Playlist audio clips and ignored note pan warn too. Existing P0 gaps remain explicit; replacement completion
does not claim those capabilities are implemented. PLG-013 is IMPLEMENTED.

Validation: 365 root tests (134 UI), 14 native tests; both workspace fmt/strict Clippy; 290-feature inventory and
repository links; all nine rebuilt release binaries pass verifier at three rates/four buffers. Non-automated
reference-versus-VST3 audio retains exact sample bits. Mixer offscreen screenshots inspected. Current artifact checks
are recorded in [lime-vst3-host-validation.md](lime-vst3-host-validation.md). GitHub Actions remains blocked by billing;
pluginval unavailable, broader native/platform runtime remains open. No merge.

Future sessions must use a NEW `codex/` branch, separate managed worktree and NEW draft PR stacked on the latest
unmerged continuation, repeat this policy in handoffs, and create another session only when the owner asks. The
owner's same-branch takeover instruction applies only to this continuation.


### 2026-10-23 — hosted P0 parity after native-only replacement

Created a NEW managed worktree and `codex/hosted-p0-parity` from verified unmerged PR #8 head `31d09f9`,
stacked on unmerged #7. Implemented PLG-024 clips first, then note pan, exact hosted track EQ, Gate/Drive/Chorus,
Comp key buses, embedded source waveforms/previews, Lime unit-value menus, dynamics telemetry and native voice
statistics (PLG-017…020, PLG-022, PLG-025…029). No runtime built-in playback was restored. Test Gain/Tone remain.

Real hosted acceptance tests replace silence/ignored-pan gap assertions and preserve the exact sample-bit/RMS-0
reference comparison. Local/bridged audio and telemetry are compared. New native editors and Mixer/Playlist/rack
renders were visually inspected. Validation details and limits: [current handoff](handoff-hosted-p0-parity.md).
Final checks: 375 root tests (136 UI), 14 native tests; fmt and strict all-target Clippy in both workspaces;
290-feature/repository checks; twelve rebuilt release and allocation-checked debug bundles pass the binary verifier
at three rates/four buffers. All release signatures verify. CLI demo: 741207 frames at 44.1 kHz, peak −3.7 dBFS.
GitHub Actions remains billing-blocked; pluginval unavailable; broader native/platform runtime remains open.

Future sessions must use a NEW `codex/` branch, NEW separate managed worktree and NEW attached draft PR based on
the latest unmerged continuation. Leave older checkouts intact; never merge automatically. Create another session
only when the owner asks.


### 2026-10-23 — Piano Roll Target control and label views

Started a NEW managed worktree `editor-view-options/limeshot` and NEW branch `codex/editor-view-options` from
`68259aa` on `origin/claude/s16-synth-tempo`. PRs #8/#9 had merged into still-open #7; this session's NEW draft PR
stacks on #7. The primary checkout, previously attached research worktree and older continuations remain intact.

Read the latest handoff, feature dependencies, architectures and decisions. Researched Image-Line's public Piano
Roll manuals and recorded evidence plus unmeasured fallback choices in
[piano-roll-view-controls.md](../research/piano-roll-view-controls.md). No FL code, bundle artwork or disassembly
was used. Implemented Target control › Note velocity / Note pan using existing commands and note data, a central
pan baseline and channel-filtered selection edits. Pan changes are one undo step per drag, round-trip through
.lime and preserve velocity. PR-016 remains IN_PROGRESS for its other properties.

Added View › Keyboard labels (None, All, Root, Highlighted, White notes only) and Keep labels on screen. Scale
changes drive labels, black-key names fit inside the short keys and zoomed-out keys hide text. Note names stay
inside visible note bodies when scrolled. These are session-local UI choices; PR-030 tracks the implemented subset.
Fixed the previously clipped lower-lane caption during offscreen inspection. Updated architecture, canonical
features/menus, generated inventories and known limitations together.

Validation: 378 workspace tests including 138 headless UI tests; all-target workspace build and strict Clippy,
rustfmt, 291-feature/737-menu inventories, repository references and diff checks pass. Reused the prior hosted-parity
build cache and its twelve existing native bundles via `LIMESHOT_LIME_PLUGINS`; this session changes no DSP/plugin
binaries. Offscreen pan/keyboard-label menu renders were inspected. Live native/audio checks remain deferred;
PLG-006 stays IMPLEMENTED. Current #7 CI run `39924174258` (`68259aa`) has no executed steps: GitHub annotations
confirm failed account payments/spending limit, so no current remote CI pass is claimed.

Every future development session must use a NEW `codex/` branch, NEW separate managed worktree and NEW attached
draft PR based on the latest unmerged continuation. Repeat this policy in handoffs even before these docs merge.
Leave earlier checkouts intact; never merge automatically; create further sessions only when the owner asks.


### 2026-10-23 — PR #7 rewrite review and repository reconciliation

Started a new `codex/pr7-rewrite-review` branch in a new managed worktree at PR #7 head `68259aa`. The owner
explicitly requested direct updates to PR #7, overriding the usual new-draft-PR rule for this session. PRs #8/#9
have merged into #7; main is updated only when the owner merges #7. No automatic merge.

Fixed all five Codex findings: insert transport for tempo-synced effects, mixer sidechain index/key remapping,
converted slot bypass, lock-free sampler sample handoff/control-thread retirement, and held-note loop/filter edits.
Also fixed bounded selection export, per-drag hosted state serialization, preset preservation of bypass/key/fields,
and shared native sample-size acceptance with conversion warnings. Current architecture, feature records,
instructions, release gates, menus and README now describe twelve hosted bundles and restored parity. Ten app
screenshots and all twelve native editor screenshots were refreshed and visually inspected. Historical records
are labelled; source comments/counts and the PR description were reconciled.

Validation: 383 root tests (137 UI), 16 native tests; formatting and strict all-target Clippy in both workspaces;
290-feature/generated inventory and repository reference checks. All twelve rebuilt release/debug bundles pass
actual-binary verification at three rates/four buffer sizes; release signatures verify and hashes are recorded.
Real VST3 regression checks delay timing at 90/180 BPM and automated tempo, including re-enabling a converted slot;
non-automated DSP parity still requires exact sample bits. Details: [review handoff](handoff-pr7-rewrite-review.md).
Remote CI remains billing-blocked; current pluginval and broader native/platform runtime validation remain open.

Future sessions resume the standing policy: NEW `codex/` branch, NEW separate managed worktree and NEW attached
draft PR stacked on the latest unmerged continuation. Leave older checkouts intact, carry work through validation,
docs, commit and push, never merge automatically, and create further sessions only when the owner asks. The PR #7
exception is limited to this explicitly requested session.


### 2026-10-23 — all CodeRabbit PR #7 findings, including dropdowns

Continued the owner-authorized PR #7 review on `codex/pr7-rewrite-review` in the same isolated managed worktree.
Audited both CodeRabbit reviews, covering all 36 inline/minor/nitpick/outside-diff findings and the summary
migration/documentation/scope checks. [Itemized disposition](coderabbit-pr7-followup.md) distinguishes prior fixes
from this follow-up. No new session/PR or merge.

Completed mono key downmix, uniform hosted-test bundle gating, shared native-pan identity, nested CI caching and
remaining limitations/path cleanup. Preserved fractional timeline phase plus the first unprocessed event tick:
3,840-note ramp regression checks exact onset count/order and integrated duration at four block sizes. Key tracking
alone now activates the synth filter. Unchanged sampler state keeps decoded audio and held notes; effect presets
merge fresh live state and preserve that baseline on undo. Conversion emits one accurate reason per failure,
checks serialized state size, retains pending settings/source data and blocks incomplete exports. Loaded .lime and
recovery sources outside the project folder, including symlinks, require explicit selection; CLI users can opt in
with --allow-external-samples. Portable Lime resolution accepts known bundle names only.

Corrected feature evidence (FX-002 is IMPLEMENTED pending native interaction verification), FL Audio Clip comparison,
PLG-013 status and regenerated counts. Added more than 80 API contract comments and fixed rustdoc HTML. Local checks:
394 root tests including 139 UI tests, 18 native tests; both workspace formatting/strict all-target Clippy; strict
rustdoc; 290-feature and repository references pass. Rebuilt twelve release/debug plugins pass three-rate/four-buffer
actual-binary verification; release signatures and hashes are recorded in native-plugins/VALIDATION.md. Hosted
Settings was rendered and inspected. Exact native/reference sample-bit assertions remain unchanged and pass.

GitHub Actions remains blocked by owner billing; no remote CI or current pluginval pass is claimed. Broader native
and platform runtime checks remain open. Future sessions retain NEW codex/ branch, separate NEW managed worktree,
NEW attached draft PR based on latest unmerged work, leave older checkouts intact, validate/document/commit/push,
never merge automatically, and create further sessions only when the owner asks. This direct-PR exception is limited
to the explicitly requested PR #7 review follow-up.


### 2026-10-23 — PR #10 integration with updated PR #7

At the owner's request, continued the existing PR #10 branch `codex/editor-view-options` in its managed worktree
and merged `origin/claude/s16-synth-tempo` at `5cf0870`. Resolved the screenshot usage line with both --note-pan
and --missing-sample, retained both branches' progress entries, kept the latest PR #7 review guidance alongside
Piano Roll continuation notes, and regenerated features.md from the merged canonical JSON. All PR #7 review
fixes and PR #10's note-pan/label options are preserved. No new feature session or PR was created.

Validation: 399 workspace tests (141 headless UI), strict all-target Clippy/build, rustfmt, feature/menu inventories,
repository references and diff checks pass. Used the PR #7 review worktree's updated native bundles/build cache;
the merged Piano Roll pan/scale render was inspected offscreen. The owner requested marking #10 ready for review
after resolving and validating the merge; no pull request merge is authorized. Existing remote CI billing and
live native/platform verification limits remain.

Future development sessions retain the standing policy: NEW codex/ branch, NEW separate managed worktree and
NEW attached draft PR based on the latest unmerged continuation; preserve older checkouts, validate/document/
commit/push, never merge automatically, and create further sessions only when the owner asks.


### 2026-10-25 — plugin windows, sampler imports, mute locks and timeline zoom

New managed worktree and branch `codex/plugin-workflow-fixes`, based on fetched main `3e2fd641` after #7/#10
merged; the open-PR query found no unmerged continuation. Older checkouts remain intact.
[Session handoff](handoff-plugin-workflow-fixes.md) records implementation and remaining native checks.

Unified plugin/outer-wrapper sizing, shared workspace title controls (minimize/pin/close, no maximize), native
AppKit preset menus, Browser/OS sample drops and sampler audio-picker inclusion. Corrected direct clicking of
locked rack/Playlist tracks, replaced the lock indicator with a padlock and added Mixer On/mute-lock support
through saved commands and sequencer solo resolution. Native sampler file actions now use the shared compressed
audio decoder. Imports report decode/size/rate errors before creating broken channels. Native title rows are
removed, with preset/bypass controls in the footer. Both timelines zoom from 0.25 to 16,384 pixels per beat,
with wider vertical ranges.

Fixed native renderer teardown via a small, licensed egui-baseview 0.9.0 source patch that binds its own GL
context before destroying painter objects; host context suspension/restoration is covered by an offscreen GL
regression. The backend previously destroyed GL names in whichever context happened to be current.

Validation: 408 host workspace tests (149 UI), 19 native tests; strict all-target Clippy in both workspaces;
workspace build, both formatting checks, generated inventories, repository references and diff checks pass.
All twelve freshly rebuilt allocation-checked debug VST3s pass actual-binary verification at 44.1/48/96 kHz and
blocks 1/64/333/1024; debug hashes are recorded in native-plugins/VALIDATION.md. App/plugin-wrapper and native
editor offscreen renders were visually inspected. Synthetic MP3 silence tests exercise real decoding and
portable state, and FLAC checks sample equality.

Screen Recording and Accessibility checks both returned false for the execution host. Live Serum internal
resizing, native preset selection, occlusion/keyboard and repeated native close/reopen remain pending;
PLG-006/PLG-012 stay IMPLEMENTED. The existing sampler limit remains 4 million stereo frames / 8–384 kHz
(about 83 seconds at 48 kHz), including MP3; oversized imports now give a specific error.

Every future session must create a NEW codex/ branch, NEW separate managed worktree and NEW attached draft PR
based on the latest unmerged continuation after checking merge state. Preserve older checkouts, validate,
document, commit and push; never merge automatically, and create further sessions only when the owner asks.


### 2026-10-25 — requested live follow-up to PR #11

Continued the existing PR #11 branch/worktree at the owner’s request. Both current host permissions report true.
[Live results and evidence](live-plugin-checks-2026-10-25.md) separate observations from inconclusive checks.
Sampler/limiter render and close/reopen without blacking out the host; sampler tabs and host preset visibility,
Serum internal shrink with wrapper synchronization/minimize-restore, and native MP3 Load are observed.
Live native drops/modifiers were inconclusive; keyboard, external OS drops, pin/occlusion and host-edge resizing
remain unverified. PLG-006/PLG-012 remain IMPLEMENTED.

Native Load exposed a real AppKit modal re-entrancy panic in egui-baseview. Added guards to defer nested drawing
and resizing and leave modal input to AppKit; rebuilt and retested the native MP3 Open-panel import successfully.
Added real Browser-row pointer-drag regression coverage. A new live_check example runs the production UI with
shared autosave/recovery disabled and does not save preferences on exit.

The first ordinary-app test run overwrote an older unclaimed Untitled recovery slot with its disposable project.
The original could not be restored; the resulting test files are preserved locally. Added a disk-backed regression
and production guards against overwriting or normal-quit deletion of an unclaimed recovery. Explicit adoption
or Discard still resolves it. Future live-check instructions in AGENTS.md require the isolated harness.

Validation: 410 host tests including 150 UI, 19 native tests; strict all-target Clippy in both workspaces;
rustfmt/inventories/repository references; all twelve rebuilt debug binaries pass three rates/four buffer sizes.
Updated native hashes and retained live screenshots. No merge or broader platform verification is claimed.

Future development sessions require a NEW codex/ branch, NEW separate managed worktree and NEW attached draft PR,
stacked on the latest unmerged continuation after checking merge state. Preserve older checkouts; validate,
document, commit and push, never merge automatically, and create further sessions only when the owner asks.

## 2026-10-25 — Sampler, mixer and editor workflow fixes (stacked on PR #11)

New branch `codex/sampler-mixer-workflow` and managed worktree, stacked on the unmerged `codex/plugin-workflow-fixes`
(PR #11) at `b54a522`. Codex implemented most of this pass and was interrupted during live checks; Claude Code finished
validation, review fixes and documentation.

Done:
* Native sampler stays visible while a host drag passes over it (AppKit hit-test passthrough instead of hiding).
* Browser double-click only previews; Browser rows label sampler channels as Samples.
* Ctrl/Cmd+drag on the Playlist or Piano Roll ruler selects the contents of the range; Cmd+B duplicates them to
  the range end and advances the range, as in FL Studio. The Piano Roll range also loops Pattern playback.
* Plugin buttons toggle like other windows: close when the plugin is frontmost or pinned, otherwise raise it.
* Double-clicking an audio clip or picker entry opens its sampler window; Y (Playback) scrubs audio clips with
  their offsets and fades.
* OS drops on the Playlist use AppKit's pointer location and the same deferred import path as Browser drops;
  multiple files add tracks rather than overlap. Imports decode once on the loader thread with a bounded decoder.
  The sampler limit is now 6 million stereo frames (about 125 s at 48 kHz, state below 64 MiB); the owner's
  119 s MP3 imports into a sampler and a Playlist clip.
* Lock icons contrast on active channels/inserts; the rack's minimum width no longer grows while scrolling.
* New automation clips start at the playhead. Right-clicking a point offers Delete, Copy/Paste value and Type in
  value (FL Studio's point menu); the curve modes and Reset tension moved to the tension handles.
* Mixer inserts are a plain list with On and a host Mix knob (saved, undoable, latency-aligned dry path). Plugin
  parameters, presets and an On switch live in the plugin window, which opens when a plugin is added. Slots are boxed like Channel Rack
  buttons and reorder by dragging a grip handle or from the context menu (owner follow-up). The track
  panel is name, Input, Output, Inserts, Equalizer.
* Tool cursors are smaller and flat (no outline).
* CodeRabbit PR #11: live_check no longer saves preferences; locked inserts stay audible through intermediate
  buses; autosave retries after a partial write; no repeated UI-thread decoding; distinct tracks for multi-file
  drops; clip titles stay inside 12 px tracks; leading spaces removed from feature notes.

Validation: 420 host tests, 19 native tests, strict Clippy in both workspaces, rustfmt, feature/inventory and
repository checks. Offscreen mixer and plugin-window renders inspected. Live native interaction for this pass
(drag passthrough, window toggling, OS drops) was not completed and remains unverified.

Future development sessions require a NEW codex/ branch, NEW separate managed worktree and NEW attached draft PR,
stacked on the latest unmerged continuation after checking merge state. Preserve older checkouts; validate,
document, commit and push, never merge automatically, and create further sessions only when the owner asks.

## 2026-10-25 — Merges and documentation refresh

PR #12 (`codex/sampler-mixer-workflow`) merged into PR #11 and PR #11 merged into main (`befb173`). Two CodeRabbit
commits landed with them:
* `a7441e1`: failed audio imports can be retried (`SampleCache::remove_failed`); new automation clips span from the
  playhead to the song end (at least one bar); Duplicate keeps notes sorted; Mixer strips use the sequencer's
  `mixer_audibility`, so locked inserts reached through buses show as audible; insert rows open their context menu.
* `1c20d45`: Pattern mode ignores loop ranges again (`pattern_ignores_time_selection_at_any_block_size`), and
  minimized plugin wrappers hide their native view. **Correction to the previous entry:** the Piano Roll time
  selection no longer loops Pattern playback; see next-steps.md for the open decision.

Documentation refresh (Claude Code, at the owner's request): README, AGENTS.md, CONTRIBUTING.md, the architecture
docs, known limitations, next steps and the native-plugin README now describe the 6 million-frame sampler bound,
On + Mix insert slots with drag reordering, Mixer mute locks, Browser double-click preview, ruler range duplication,
native drag passthrough and the merged PR state. Older handoffs that cite the 4 million-frame bound carry a note.
No code changed and nothing was run for this refresh; test counts quoted elsewhere are the latest recorded ones
(420 host, 19 native).

## 2026-10-25 — Windows fixes from the owner's first Windows run

The owner reported on Windows: every Lime plugin failed with "the plugin has no class 4C494D45…", third-party
plugin editors did not appear, Explorer drops did nothing and `limeshot.exe` opened a console window.

* **Class ids.** On Windows a VST3 `TUID` is a COM GUID whose first eight bytes are little-endian; nice-plug
  registers Lime classes that way, while LimeShot's `lime:` references and `.vstpreset` files use the portable
  (`FUID::toString`) order. `util::tuid_to_hex` / `hex_to_tuid` now convert, so class-id strings are identical on
  every platform; `Vst3Module::create` falls back to the raw order for ids saved by older Windows builds.
* **Editors.** New Windows `platform` module in `editor.rs`: the main window is found by title among the UI
  thread's windows, plugin views go into a `STATIC` child `HWND`, hidden/disabled for occlusion and drag
  passthrough, and preset menus use `TrackPopupMenu`. `graphics.rs` saves and restores the host's WGL context
  around editor callbacks and teardown, as on macOS.
* **Explorer drops.** `NativeWindow::pointer_position` reads the Win32 cursor (winit reports no pointer during OS
  drags), so Playlist, rack and sampler drops get a position.
* **Console.** Release builds of the GUI use the Windows subsystem.

Validation: rustfmt; strict Clippy for the workspace on `x86_64-pc-windows-msvc` and for the plugin/app crates on
Linux. Tests were not run and nothing has been run on Windows; the owner's next Windows pass verifies these.

### Windows follow-up: blank, transparent plugin wrappers

The owner's next run showed the wrapper's editor area unpainted (stale pixels that smeared while dragging).
The container used the system `STATIC` class, which paints through its parent's device context (`CS_PARENTDC`);
over the host's OpenGL surface that never reaches the screen. The container now has its own registered class with
a dark background brush. Each attach logs the editor size, container and plugin child window (size, visibility,
DPI scale) to help diagnose an editor that still does not appear. Checked with rustfmt and strict Clippy for
Windows, macOS and Linux targets; not yet run on Windows.

### Windows follow-up: editors in owned popups

The next run was unchanged: the log showed the container and the plugin's window visible at the right sizes, yet
not even the container's dark brush reached the screen. Common Windows OpenGL drivers present the host's surface
over child windows, so nothing in a child `HWND` is ever seen. The container is now a borderless `WS_POPUP`
(tool window) owned by the main window: owned popups stay above the host and minimise with it. It is placed in
screen coordinates over the editor area, clipped to the host's client area with a window region, and a
`SetWindowSubclass` hook on the host (`WM_WINDOWPOSCHANGED`) moves it with the window. Strict Clippy and the
plugin crate's tests pass on Windows; not yet checked by hand with a plugin editor.

### Windows follow-up: editors that never redraw, window placement

With popups the editors appeared (Serum's dials worked), but Lime plugin dials looked dead, a sample dropped on
the Lime Sampler only showed after reopening its window, Serum's keyboard did not light up, and windows opened in
odd places. winit returns the host's pending `WM_PAINT` (the host asks for a frame every 16–50 ms) before any
`WM_TIMER`, so editors that redraw from a timer (baseview, which the Lime plugins use) were starved.
`limeshot_plugins::service_editors` now runs after the plugin windows each frame: it dispatches due `WM_TIMER`
messages and paints the invalid parts of every editor popup (`RedrawWindow` with `RDW_UPDATENOW`). A popup
stays hidden until it is first placed, rather than flashing at the main window's corner. Following FL Studio, a
plugin window with no remembered position opens in the middle of the workspace, each further one a title bar lower
and to the right. The presets menu now has a fixed width: its area had started as wide as the screen allowed and,
for a wrapper right of centre, was pushed back over the wrapper's controls. Strict Clippy passes; the workspace
tests pass except `samples_are_picked_and_drawn_in_the_playlist_like_patterns` and
`the_draw_tool_reuses_the_length_of_the_last_clip`, which fail on the base commit too, and the intermittent
`recording_turns_played_notes_into_pattern_notes`. Not yet checked by hand on Windows; Serum's keyboard may have a
separate cause.

### Windows follow-up: partly covered editors stay visible

The owner confirmed the editors now work. A wrapper whose editor another window (or menu) partly covered still
went blank until it was raised. On Windows the wrapper now samples its editor area on a 12-point grid
(`ctx.layer_id_at`), collects the layers above it and passes their rectangles to `Vst3Editor::set_cutouts`; the
popup's window region leaves them out (`CombineRgn` with `RGN_DIFF`), so the covering window shows through and the
rest of the editor stays. The editor hides only when no sample of it is uncovered. macOS (`can_cut_out` false)
keeps hiding a covered editor.


## 2026-10-25 — Project integrity audit and alpha release installers (Windows, macOS, Linux)

New managed worktree (`D:\Programming\rust\limeshot-alpha-installers`) and branch `codex/alpha-installers`, based on
`main` at `12f6542` after PR #13 merged; no open PRs existed at session start. Older checkouts remain intact.

### Project integrity check
* Fixed the single `rustfmt` violation on `main` (`crates/limeshot-ui/src/playlist.rs` line 137).
* `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass in both the root
  workspace and `native-plugins/`.
* All 428 root workspace tests (including all 156 headless UI tests in `limeshot-app/tests/ui.rs`) and all 19
  `native-plugins` tests pass (`samples_are_picked_and_drawn_in_the_playlist_like_patterns` and
  `the_draw_tool_reuses_the_length_of_the_last_clip` pass after PR #13).
* `python scripts/features.py check`, `python scripts/check_repository.py` and `python scripts/test_check_repository.py`
  pass with 0 errors.
* Deterministic `limeshot-cli` demo render and re-render produce bit-identical WAV output (`peak -3.7 dBFS`).

### Cross-platform build and installers (PF-006, D40)
* Extended `crates/limeshot-app/examples/make_icon.rs` to write a multi-resolution PNG-compressed Windows `.ico`
  (16…256 px) and a 256 px Linux `.png` alongside the macOS `.iconset`, from the same `limeshot_app::icon_rgba`
  painter code.
* **Windows (`scripts/package-windows.ps1`, `packaging/windows/limeshot.iss`, `packaging/windows/README.txt`):**
  builds the release binaries and all twelve release Lime Native VST3 bundles, runs a smoke render through the staged
  bundles, normalizes pre-1980 crate licence timestamps, and produces `dist/LimeShot-<version>-windows-x64-portable.zip`
  (47.95 MB) and `dist/LimeShot-<version>-windows-x64-setup.exe` (33.81 MB, Inno Setup 6). Supports per-machine
  (`Program Files\LimeShot` + `Common Files\VST3`) or per-user (`%LOCALAPPDATA%\Programs\LimeShot` +
  `%LOCALAPPDATA%\Programs\Common\VST3`) installation, optional private-only VST3 placement (`<app>\plugins`),
  `.lime` file association and clean uninstallation. Verified locally on Windows 11 x64 in both shared-VST3 and
  private-plugin modes, including rendering the demo with the installed `limeshot-cli.exe` while `native-plugins/dist`
  was hidden.
* **macOS (`scripts/package-macos.sh`):** builds `dist/LimeShot.app` (with the twelve Lime VST3 bundles in
  `Contents/PlugIns` and plugin/font licences in `Contents/Resources/licenses`), signs each bundle and the app ad hoc,
  verifies the demo render, builds `dist/LimeShot-<version>-macos-<arch>.dmg`, and builds a `pkgbuild`/`productbuild`
  installer `dist/LimeShot-<version>-macos-<arch>.pkg` that installs `LimeShot.app` into `/Applications` and the
  twelve Lime Native VST3 bundles into `/Library/Audio/Plug-Ins/VST3`.
* **Linux (`scripts/package-linux.sh`, `packaging/linux/{install.sh,uninstall.sh,limeshot.desktop,limeshot-mime.xml,README.txt}`):**
  stages a runnable directory, verifies the demo render, packs `dist/limeshot-<version>-linux-<arch>.tar.gz` with
  `install.sh`/`uninstall.sh` (system-wide `/opt/limeshot` + `/usr/local/lib/vst3`, `--user` in
  `~/.local/opt/limeshot` + `~/.vst3`, or `--private-plugins`), and builds `dist/limeshot_<debversion>_<debarch>.deb`
  (`/opt/limeshot`, `/usr/bin` symlinks, `/usr/lib/vst3` bundles, desktop entry, icon and `.lime` MIME type).
  Verified `install.sh` (`--user` and `--private-plugins`) and `uninstall.sh` in an isolated temp root.
* **Release workflow (`.github/workflows/release.yml`) and documentation (`docs/development/release-packaging.md`):**
  builds Windows x64, macOS Apple Silicon & Intel, and Ubuntu 22.04 x64 & ARM64 packages, tests the Linux `.deb` and
  tarball installs on the runner, and attaches all artifacts plus `SHA256SUMS.txt` to a draft GitHub release on `v*`
  tags.

Every future development session must create a NEW `codex/` branch, a NEW separate managed worktree and a NEW attached
draft PR stacked on the latest unmerged continuation after inspecting merge state. Leave older checkouts intact, carry
work through implementation, validation, documentation, commit and push, never merge automatically, and create further
sessions only when the owner asks.


## 2026-10-25 — CI and alpha release audit

Started a new managed worktree and `codex/ci-release-fixes` from main `5ab09c9`, after confirming every prior
continuation through PR #14 had merged. The original main checkout is clean and remains at `3e2fd64`;
older worktrees are intact. Main's current CI executes again, so old billing-blocked notes are historical.

Repaired the Browser-to-rack test's asynchronous sample-import assertions with bounded completion waits.
Added the missing Wayland development package to native-plugin Linux CI. Replaced the weak VST3 module cache
with process-lifetime strong retention (D41), preventing unload of framework callbacks and overlapping module
teardown/reload. A real-bundle regression checks retention without clients and identity across concurrent loads.
No UI appearance or audio scheduling changes. Updated architecture, platform evidence, feature notes and limits.

Local verification with `CI=1` (mandatory bundles) passes: 427 workspace tests (157 UI), strict all-target
Clippy/build, formatting, inventories,
repository references, five repository-checker tests, 25 independent rack-drop runs and byte-identical hosted
demo/reload WAVs (-3.7 dBFS). The module regression also fails against the original weak cache at its intended
residency assertion. Corrected its portable bundle URI after Windows CI exposed an optional-local skip.
Reused the existing plugin-workflow build cache and native debug bundles.
Fresh remote runs are linked in [platform validation](platform-validation.md); final outcomes are recorded in PR #15.
Linux ARM64 now passes both formerly crashing concurrent MVP tests and all 157 UI tests; its only failure in
`3a28f7d` is the new test's corrected bundle URI. Final complete CI remains separate from that targeted evidence.

All five original alpha packaging jobs and draft-release assembly pass. Downloaded and verified all ten package
hashes, both DMGs, ZIP CRCs and Linux tarball plugin/install contents. Found and repaired the draft checksum manifest:
GitHub rewrote Debian filename tildes to dots, leaving two manifest paths nonexistent. Replaced only SHA256SUMS.txt
with the verified actual filenames and downloaded it again to confirm. Future Debian filenames keep the original
version while control metadata keeps the prerelease tilde. The alpha tag remains at pre-fix `5ab09c9`; release remains
a draft. A manual repaired-source packaging run creates artifacts only, without moving tags or publishing.

Final native validation passes on all three OSes (`40215397199`); both Linux architectures, Windows and Intel
macOS pass full CI at `d6f3d17`. Apple Silicon exposed a further hosted-slider reset race: 8/100 local stress runs
failed immediately and 4/100 still failed with a bounded wait. Committed/reset values now also enter the existing
parameter queue, so an older queued drag cannot restore the wrong value when the saved document state is unchanged.
The test waits for the final handoff while retaining its value assertion. After the fix: 100/100 stress runs,
427 mandatory-bundle workspace tests, strict Clippy and a visually inspected settings render pass. Embedded plugin
state is still serialized once at release. Re-run final CI; current results are recorded in PR #15.

Every future development session must create a NEW `codex/` branch, NEW separate managed worktree and NEW attached
draft PR stacked on the latest unmerged continuation after inspecting merge state. Leave older checkouts intact,
validate/document/commit/push, never merge automatically, and create further sessions only when the owner asks.
