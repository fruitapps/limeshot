# FL Studio analysis

> Reference application: **FL Studio 2026, version 26.1.1.5313** (Image-Line), installed at
> `/Applications/FL Studio 2026.app` on macOS 26.6.2 (arm64). Executable `Contents/MacOS/OsxFL`
> (universal x86_64/arm64); engine `Contents/Libs/FLEngine_x64.dylib`. User data:
> `~/Documents/Application Documents/Image-Line/FL Studio/`.
>
> Sources: (a) the official online manual, https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/
> (pages `channelrack.htm`, `pianoroll.htm`, `playlist.htm`, `mixer.htm`, `toolbar_panels.htm`,
> `browser.htm`, `chansettings_sampler.htm`, `basics_shortcuts.htm`, fetched 2026-10-20; the
> installed app ships only an offline stub plus `HelpAliases.scr`); (b) REA static evidence
> (`rea-findings.md`); (c) bundle and settings listings on this machine.
>
> **Live observation (session 2, 2026-10-20):** Screen Recording was granted, so window screenshots
> were taken with `screencapture -l` (`docs/research/evidence/screenshots/fl-main.png`,
> `fl-dialog.png`). Accessibility is **not** granted: REA's `observe-native-ui` refuses without it
> (`evidence/rea/observe-ui-965.json`), and synthetic clicks are impossible, so FL could not be
> operated (its modal welcome dialog also blocks opening demo projects). Interaction details still
> come from the official manual. Findings marked OBSERVED below were seen in these screenshots.
>
> Confidence labels: **VERIFIED** (installed artifact or official documentation), **OBSERVED**
> (seen on this machine), **INFERRED** (reasoned from evidence), **UNKNOWN**.

---

## 1. Main window composition
- **Expected/Observed:** One main window. A top toolbar strip with menus, transport, tempo, time display, pattern selector, snap selector, metronome/count-in/step-edit/typing-keyboard toggles, window shortcut icons and the CPU/memory panel. A browser docked on the left. The editors (Channel Rack, Piano Roll, Playlist, Mixer) are internal child windows that can be detached. The CG window list shows one 1680×1020 main window plus toolbar child strips (OBSERVED). Toolbar contents are VERIFIED from `toolbar_panels.htm`.
- **Interaction:** F5 Playlist, F6 Channel Rack, F7 Piano Roll, F8 Plugin Picker, F9 Mixer, Alt+F8 Browser, F12 close all, Esc close window, Enter maximize Playlist (VERIFIED, shortcuts page).
- **Confidence:** VERIFIED (docs) / OBSERVED (window list).
- **LimeShot equivalent:** `limeshot-ui` main layout: top toolbar panel, left Browser side panel, central workspace with toggleable child windows (egui `Window`, constrained to the workspace) bound to the same F-keys.

### 1a. Observed main window (screenshot `fl-main.png`, 1680×1020 pt)
OBSERVED:
* Toolbar row 1: traffic lights, upper-case menu labels (FILE EDIT ADD PATTERNS VIEW OPTIONS TOOLS
  HELP), a vertical **PAT/SONG** pill pair, play, stop, record, tempo field **140.000** (the default
  tempo of a new project), a large time display **1:01:00** labelled **B:S:T** (bar:step:tick), then
  recording/metronome toggles (orange when on), CPU/memory panel.
* Toolbar row 2: user/login box, window buttons (Playlist, Channel Rack, Piano Roll, Mixer, Browser…),
  snap selector showing **Line**, pattern selector **Pattern 1** with a **+** button, Shift/Cmd/Opt
  touch-modifier buttons, hint/news panel.
* Browser docked left: a row of tab icons (all / project / plugins / online / starred), then a
  category tree with coloured icons; a TAGS field at the bottom.
* Playlist fills the rest of the window and opens maximised by default. Its title bar has the tool
  icons (draw, paint, delete, mute, slip, slice, select, zoom, play) and the breadcrumb
  "Playlist – Arrangement › Pattern 1". On its left is the **clip picker** listing patterns
  ("Pattern 1"). Tracks are tall rows ("Track 1…") with a green LED at the bottom right of each
  header. The ruler numbers bars, with bigger labels every 4 bars.
* Channel Rack, Piano Roll and Mixer are closed in a new project.
* Palette: blue-grey surfaces (about #34404a / #2b353d), orange accents for active toggles, green
  LEDs, light grey text.
* On start, a modal **Welcome** window offers templates, Loop Starter, chord progression, a demo list
  and news (`fl-dialog.png`).

LimeShot changes taken from this: new projects default to **140 BPM**; the Playlist is the main,
docked workspace with a pattern picker on its left; editors open docked rather than floating; track
headers keep the LED at the right.

## 2. Channel Rack and step sequencer
- **Expected behaviour (VERIFIED, `channelrack.htm`):** Channels are shared by all patterns; each pattern holds its own step and piano-roll data per channel. Row layout: Mute, Pan, Volume, mixer-track target, channel button (name), selector LED, then step buttons or a piano-roll preview. Each step is a 16th note; the default is 16 steps; the maximum is 512. Left-click a step turns it on; right-click turns it off. Steps have no note-off. Ctrl+click on a mute button solos. Ctrl+L assigns selected channels to free mixer tracks. Alt+Up/Down reorders. Pattern length is Auto (ends at the last bar with data) or Manual. Global swing delays every second 16th. A Graph editor edits per-step velocity, pan, pitch and more.
- **Key model fact:** "Piano roll mode shows zero-length, on-beat notes as steps". Steps and piano-roll notes are **one note list**. A channel whose notes are not step-like shows a preview instead of step buttons.
- **Interaction sequence:** add a channel (Add menu or drag a sample in) → select a pattern → click steps → play (Space, PAT mode) → change tempo while playing.
- **Confidence:** VERIFIED (docs); visual layout not observed live.
- **OBSERVED live (FL Studio 2026.1.1, Default template, 2026-10-20, `fl-steps-toggle.png`):** the docs' "right-click
  turns off" is not what the current version does. **Both buttons toggle**: a left- or right-click on an unlit step
  lights it, on a lit step turns it off. A drag (either button) paints the state the *first* step was switched to, so a
  drag that starts on a lit step erases the steps it crosses. Each row shows mute LED, pan knob, volume knob, mixer
  track number, name button, selector LED, then the steps in groups of four with alternating grey/red tints. The
  Default template has 808 Kick, Clap, HiHat, Snare and a fifth "808 Astronomic" channel at 130 BPM.
- **OBSERVED live (FL Studio 2026.1.1, session 7; `fl-rack-select-ctrl-shift.png`, `fl-rack-alt-up.png`,
  `fl-rack-channel-menu.png`, `fl-rack-delete-confirm.png`, `fl-rack-f2-renames-pattern.png`):** the selector LED
  selects channels. A plain click selects only that channel; Ctrl-, Shift- and right-click each toggle one channel (Shift
  is *not* a range). Clicking a step, or right-clicking a channel button, also selects that channel only. Up/Down select
  the neighbouring channel on its own; Ctrl+A / Cmd+A do not select all. Alt+Up/Down moves every selected channel one row
  (the mixer-track number moves with it). Alt+Del (forward delete) and the menu's Delete… ask "The selected channel(s)
  is/are about to be deleted." with a "Remember my choice" box; Alt+Backspace does nothing. F2 with the rack focused
  opens the *pattern* name dialog. The channel button menu: Piano roll (F7), Graph editor, Rename color and icon…,
  Change color…, Random color, Change icon…, Load sample…, Cut itself, Insert, Replace, Clone, Delete…, Assign to new
  instrument track, and a second column with Cut/Copy/Paste, Fill each 2/4/8 steps, Advanced fill…, Rotate left/right
  (Shift+Cmd+Left/Right), MIDI channel through, Receive notes from, Create DirectWave instrument, Burn MIDI to.
- **OBSERVED live (session 8, `fl-rack-fill-4.png`, `fl-rack-rotate-left.png`):** with step 2 lit, the channel menu's
  Fill each 4 steps leaves exactly steps 1, 5, 9 and 13 lit (the old step is cleared). Shift+Cmd+Right moves them to 2,
  6, 10, 14; Shift+Cmd+Left twice to 4, 8, 12, 16 (step 1 wraps round to 16). The rack's top-left dropdown is the
  channel filter group ("All", "Audio" …): with "Audio" only the audio channel showed.
- **LimeShot equivalent:** `ChannelRack` window. Rows carry mute/solo, pan knob, volume knob, mixer-target spinner, name button and step buttons. Steps are notes at `step × 240 ticks` with the channel's root key. Left-click toggles a step on, right-click toggles it off. A channel with non-step notes shows a mini preview (clicking it opens the Piano Roll). Pattern length is Auto or a fixed number of steps.

## 3. Piano Roll
- **Expected behaviour (VERIFIED, `pianoroll.htm`):** Pitch is vertical, time horizontal; the ruler shows bars. The target-channel dropdown picks which channel is edited. Tools: Draw (P), Paint (B), Delete (D), Mute (T), Slice (C), Select (E), Zoom (Z), Playback (Y). In Draw, left-click adds a note, dragging moves it, right-click deletes, right-drag deletes many, Alt bypasses snap, Ctrl selects. Resize by dragging the right edge. Shift+arrows nudge. Ctrl+B duplicates the selection. Ghost notes come from other channels. The lower pane edits note properties (velocity, pan, release, Mod X/Y, pitch). Left-clicking the preview keyboard auditions a note. Snap follows the global snap (Main) or a local value; right-click sets the default note length. Double-click a note opens Note Properties. Ctrl+Up/Down transposes by an octave.
- **Confidence:** VERIFIED (docs).
- **OBSERVED live (`fl-note-length.png`):** in a fresh project the first note drawn is **two steps** (an eighth note)
  long with snap "Line". After a note is resized, new notes take that length. Step notes from the Channel Rack show as
  short notes on the channel's root key (C5); other channels' notes appear as grey ghost notes.
- **OBSERVED live (session 7, `fl-piano-edit-menu.png`, `fl-piano-select-menu.png`):** the Piano Roll menu lists
  macOS shortcuts: Cut/Copy/Paste Cmd+X/C/V, Duplicate Cmd+B, Delete Del, Shift left/right Shift+Left/Right, Rotate
  Shift+Cmd+Left/Right, Transpose Shift+Up/Down, octave Cmd+Up/Down, Discard lengths Shift+D, Mute Opt+M, Unmute
  Shift+Opt+M, Insert/Delete space Cmd+Ins/Del; Select: Deselect Cmd+D, Select all Cmd+A, one/more at random Shift+R/M,
  by color Shift+C, odd Shift+O, Invert Shift+I, time around selection Cmd+Enter. Plain Up/Down scroll the view (about
  three rows per press). With nothing selected, Shift+Up transposes every note of the channel.
- **OBSERVED live (session 7, `fl-piano-tools-menu.png`, `fl-piano-paint.png`, `fl-piano-slice.png`):** Tools menu:
  Riff machine Opt+E, chord progression Opt+P, Quick legato Cmd+L, Articulate Opt+L, Quick quantize Cmd+Q, start times
  Shift+Q, Quantize… Opt+Q, Quick chop Cmd+U, Chop… Opt+U, Glue Cmd+G, Arpeggiate Opt+A, Strum Opt+S, Flam Opt+F, Claw
  machine Opt+W, Limit Opt+K, Flip Opt+Y, Randomize Opt+R, Scale levels Opt+X, LFO Opt+O. The hint bar labels the tools
  Draw P, Paint B, Slice C, Select E, Zoom Z, Playback Y. A Paint drag lays notes of the last length end to end, all
  selected; a Slice drag across a note cuts it at the line and selects the right piece. "Allow resizing from left" is
  ticked by default.
- **LimeShot equivalent:** custom-painted Piano Roll. Draw-tool semantics (left add/move/resize, right delete, Ctrl rectangle select, Alt no-snap), a velocity lane, keyboard auditioning, horizontal and vertical zoom and scroll, a playhead, Delete, Ctrl+A, Ctrl+B duplicate, Shift+Up/Down transpose by a semitone, Ctrl+Up/Down by an octave. Ghost notes, slice and paint tools are tracked (P1).

## 4. Playlist and arrangement
- **Expected behaviour (VERIFIED, `playlist.htm`):** Tracks are generic "clip tracks", not bound to mixer tracks. Clip types: pattern, audio, automation. Pattern clips reference a pattern; editing the pattern changes every instance until **Make unique**. Tools: Draw, Paint, Delete, Mute, Slip, Slice, Select, Zoom, Playback. In Draw, left-click places the current pattern, dragging moves, right-click deletes, Shift-drag on an edge resizes. Ctrl+B duplicates to the right. Snap options include Main/Line/Cell/beats/bar. Markers and time selection live on the timeline. Performance mode and Arrangements exist.
- **Song vs pattern playback:** the toolbar PAT/SONG switch (L key). SONG plays the Playlist; PAT loops the current pattern (VERIFIED, `toolbar_panels.htm`).
- **Confidence:** VERIFIED (docs). **OBSERVED live (`fl-clip-no-loop.png`):** a one-bar pattern clip (steps only) dragged out to five bars draws its notes only in the first bar, and playing in SONG mode from bar 4 inside the clip is silent (flat oscilloscope, empty peak meter) while playing from bar 1 sounds. Pattern clips do **not** loop their pattern; the extended region is silence. This confirms decision D14.
- **OBSERVED live (session 7, `fl-playlist-time-selection.png`, `fl-playlist-time-markers-menu.png`):** a plain drag
  on the ruler scrubs the song position (switching to SONG); Ctrl+drag makes a time selection, drawn as a red band, and
  moves the song position to its start. Time markers menu: Add one… Opt+T, Add auto Cmd+T (a marker named "Auto" at the
  song position, drawn as a flag under the ruler), Add two…, Add one every, Add jump to next bar, Add time signature
  change… Shift+Opt+T, Add key marker, Place loop Shift+T, Delete, Change to marker loop. Plain Up/Down scroll the
  Playlist one track.
- **OBSERVED live (session 8, `fl-playlist-tools.png`, `fl-playlist-slip.png`, `fl-playlist-slice.png`):** the
  Playlist tool strip reads Draw, Paint, Delete, Mute, Slip, Slice, Select, Zoom, Playback. Slip on a clip that shows
  its whole sample changes nothing; after shortening the clip, a Slip drag moves the content under the fixed clip by
  the snapped distance, slipping past the sample end leaves silence, and slipping back stops at the sample start.
  A vertical Slice drag cuts the clip in two at the snapped line; both pieces keep their names and the content runs on
  across the cut. The manual adds Shift+S ripple edit, Shift diagonal slicing and Shift+click slicing without a drag.
- **LimeShot equivalent:** Playlist window with tracks, pattern clips that reference patterns by `PatternId` (linked by default), "Make unique", draw/move/resize/delete/duplicate, a bar/beat ruler, a playhead, click-to-seek on the ruler, horizontal zoom and scroll, and snap to bar/beat/step. Song mode drives the engine from these clips. A pattern clip in LimeShot plays its pattern **once**, truncated to the clip length (documented choice).

## 5. Pattern management
- **VERIFIED:** Up to 999 patterns. The toolbar pattern selector supports rename, clone, insert, delete, find empty and render-to-audio. The Picker panel lists patterns.
- **LimeShot equivalent:** a pattern selector in the toolbar (prev/next/new/clone/rename/delete); patterns have stable ids and names.

## 6. Mixer
- **VERIFIED (`mixer.htm`):** Master plus up to 500 inserts plus a "Current" track. Per track: pan, fader (post-FX), stereo separation, phase invert, swap L/R, mute (left-click), solo (Ctrl/right-click), 10 FX slots, a 3-band EQ, peak meters in dB, record arm and PDC delay. Channels route to inserts (the default route for new channels is Master). Inserts route to Master by default; post-fader sends can go to other inserts; sidechains are supported. Insert tracks practically cannot clip; Master can.
- **OBSERVED live (session 8, `fl-mixer-eq.png`):** the Mixer's track panel ends with an Equalizer: a response graph,
  three vertical gain sliders and two rows of knobs (frequency, width). The hint bar names them "EQ band 1 (low
  shelf)", "EQ band 2 (peaking)", "EQ band 3 (high shelf)"; defaults 90 Hz, 1500 Hz and 8000 Hz, level 0.0 dB, width
  0.27 each; a gain slider dragged to the top reads +18.0 dB. Hovering the graph shows the frequency and "Tweak band
  N" for the nearest band. The manual: left-drag sets frequency and level, right-drag bandwidth and level.
- **OBSERVED live (session 8, `fl-mixer-route-menu.png`, `fl-mixer-sidechain.png`):** right-clicking a strip's route
  arrow (with another track selected) opens Route to this track / Route to this track only / Sidechain to this track /
  Sidechain to this track only. "Sidechain to this track" adds a send whose level knob sits at zero, drawn as a thin
  wire, and the source stays routed to Master. Per the manual, plugins with a sidechain input (Fruity Limiter) pick
  such a link as their input.
- **LimeShot equivalent:** Master plus 16 inserts by default (more can be added). Each strip has a name, a peak meter, a fader (dB), pan, mute and solo. Channels have a mixer-target number. Inserts route to Master. FX slots, sends and EQ are tracked (P1). A clip indicator appears on Master.

## 7. Browser
- **VERIFIED (`browser.htm`):** Tabs: All, Current Project, Plugin Database, Online Content, Starred. Search with wildcards and tags. Left-click previews a sample. Samples can be dragged to the Playlist or the Channel Rack. User folders can be added.
- **LimeShot equivalent:** a left panel with "Project" (patterns/channels) and a file tree of user folders (the bundled `assets/samples` plus user-added folders). Clicking a WAV previews it through the engine. A WAV can be loaded into the selected sampler channel, or a new sampler channel created from it. A text filter is included; tags and favourites are tracked.

## 8. Transport
- **VERIFIED:** Space play/stop, Ctrl+Space pause, L Pat/Song, R record, Ctrl+M metronome, Ctrl+H panic. Tempo range 10–522 BPM with 1/1000 BPM resolution; tap tempo. The time display switches between Bar:Step:Tick and M:S:CS.
- **LimeShot equivalent:** Play/Pause, Stop, Pat/Song toggle, metronome, tempo field (10–522 BPM, 3 decimals stored as milli-BPM), time display, Space/L/Ctrl+M shortcuts. Record is tracked (P1) and its button is shown disabled.

## 9. Instrument loading and Sampler
- **VERIFIED (`chansettings_sampler.htm`):** The Channel Sampler loads by drag-and-drop or a load button. Volume/pan/filter/pitch envelopes, PITCH knob, time stretch, Start Offset, reverse, loop points, declicking modes. The default sampler level is about −5.2 dB.
- **LimeShot equivalent:** an original sampler. WAV loading on a worker thread, gain, pitch in semitones and cents (resampling playback), start/end trim, reverse, a short declick release, polyphony from the shared voice pool, note pitch relative to a root note.

## 10. Native synthesizers
- **OBSERVED (`Plugins/Fruity/Generators`):** 52 native generators including 3x Osc, FLEX, FPC, Sytrus, Harmor, Harmless, Sakura, Transistor Bass, BooBass, MiniSynth, SimSynth, Drumaxx, Fruity DrumSynth Live, Slicex, Fruity Slicer, DirectWave, Soundfont Player, Plucked!.
- **LimeShot equivalent:** "LimeSynth", an original subtractive synth with sine/square/saw/triangle oscillators, ADSR, detune, a low-pass filter (P1) and polyphony.

## 11. Automation
- **VERIFIED (`HelpAliases.scr`, playlist page):** Automation clips in the Playlist, an event editor, LFO tool, internal controllers and formula controllers. Most controls can be automated.
- **OBSERVED live (session 5, `fl-automation-clip.png`):** right-clicking a Channel Rack knob opens a menu with Reset,
  Edit events, Init song with this position, Create automation clip, Link to controller, Copy/Paste value, Type in
  value and Set. *Create automation clip* adds an automation-clip channel ("808 Kick – Channel panning") to the Channel
  Rack (whose filter switches to the Automation group) and to the Playlist picker, and places a clip at bar 1, track 1,
  as long as the song, drawn as a flat line at the knob's value over a filled area. Clicking empty Playlist space
  then places linked instances. Right-click on the curve does not delete the clip; it added a point, and a drag
  between points changed the curve tension (hint bar: "Point 1 to 2 tension 33%"). The Playlist header shows STEP and
  SLIDE options next to the automation icon (effect not observed).
- **LimeShot equivalent:** automation clips (`limeshot_core::automation`): created from control context menus, listed in
  the Playlist picker, edited in the clip (drag points, click the line to add, right-click a point to delete, ○ tension
  handles), applied in Song mode.

## 12. Keyboard shortcuts
See `basics_shortcuts.htm` (VERIFIED). Every FL shortcut is tracked against LimeShot in
`docs/roadmap/keybindings.json` (76 so far; INFERRED entries still need checking), settings in `settings.json`.

## 13. Project lifecycle
- **VERIFIED:** `.flp` project files (Info.plist document type), templates (`Data/Templates/*`), Save (Ctrl+S), Save As, "Save new version" (Ctrl+N), Export WAV (Ctrl+R), Export MP3 (Ctrl+Shift+R), Export MIDI, recent files Alt+0..9.
- **LimeShot equivalent:** `.lime` files with New/Open/Save/Save As, modified indicator, ask-to-save on close and new, Export WAV.

## 14. Parameter interactions
- **VERIFIED (docs, general behaviour):** knobs and faders are dragged vertically; Ctrl/Alt modifiers give fine adjustment; right-click opens a context menu with reset; the hint bar shows values.
- **LimeShot equivalent:** knob widget with vertical drag, Shift for fine adjustment, double-click to reset, and a hover tooltip showing the value. The status bar shows the hover hint.

## 15. UI scaling and resizing
- **VERIFIED:** `NSHighResolutionCapable = YES`. INFERRED: vector rendering (blend2d, ILSvg) allows arbitrary scaling.
- **LimeShot equivalent:** egui scales with `pixels_per_point`. A View › Zoom setting (Ctrl+= / Ctrl+-) sets the UI scale.

## 16. Editions
FL Studio is sold in Fruity, Producer, Signature and All Plugins editions. FL Cloud features (Loop Starter, sounds, mastering, AI stem extraction) are online services. **LimeShot will not replicate online/account services** (offline-first). They are recorded in the inventory as `DEFERRED` / out of scope.
