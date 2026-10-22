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
- **OBSERVED live (session 8, `fl-rack-graph-editor.png`):** Graph editor (channel menu) opens a panel attached under
  the channel's step row, one column per step, with lane tabs Note, Vel, Rel, Fine, Pan, X, Y, Shift, Rep. Vel shows
  a bar per lit step (default about 78 %); a fast drag across the panel draws a straight line through the steps it
  passes. Pan shows a short centre mark per lit step; the hint bar names the lane and the value under the pointer.
- **OBSERVED live (session 8, `fl-rack-filter-groups.png`, `fl-rack-menu.png`, `fl-rack-group-name.png`):** the
  filter dropdown listed All, Audio (the channel of an audio clip dragged into the Playlist) and Unsorted. The rack
  menu's Group selected… (Opt+G) opens a small "Filter group name" field; naming it "Loops" moved the selected channel
  there and the empty "Audio" group disappeared. "Loops" then showed only that channel, "Unsorted" the other five.
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
`docs/roadmap/keybindings.json` (118 so far; INFERRED entries still need checking), settings in `settings.json`, main
menu items in `menus.json` (§18).

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

## 17. Plugin windows (Fruity Wrapper)
- **OBSERVED live (session 12, FL Studio 2026.1.1, synthetic input; Vital and Surge XT Effects as VST3):**
  screenshots `fl-plugin-window-*.png`.
  - **One window per plugin**, titled with the plugin and where it runs: "Vital (Master)" for a channel (the name of
    the channel and, in grey, its mixer target), "Surge XT Effects (Insert 1)" for an insert slot. Each is its own
    floating OS window (window level 3, above FL's main window), and the plugin's native view sits in a separate child
    window inside it (`CGWindowList` shows two windows: the wrapper and, 5 pt in and below the 26 pt title bar, the
    plugin's view at exactly the size the plugin asks for).
  - **Opening:** adding a plugin opens its window. A click on the channel's button in the Channel Rack opens it and a
    second click closes it; in the Mixer a click on the insert slot's name does the same. Closing (×) destroys the OS
    window (a new window id each time it opens), but FL reopens it at the same position, size and page.
  - **Title bar** (left to right): ▸ menu, a gear (Detailed settings) that shows or hides the wrapper's page strip,
    the title; on the right a keyboard toggle ("Allow the plugin to handle keystrokes before the host does"),
    Presets ◀ ▶ (previous/next preset; channels only), − (Captionize: shrink to just the title bar, right-aligned;
    − again restores) and ×. Dragging the title bar or the window's border moves the window.
  - **Pages** (the gear's strip, 46 pt high, with page icons on the left and the channel's ON, PAN, VOL, PITCH and
    RANGE knobs and mixer TRACK selector on the right — an effect shows ON and MIX LEVEL instead):
    1. *Plugin editor* — the plugin's own view (`fl-plugin-window-detailed.png`).
    2. *Plugin options* — FL's wrapper settings, with Settings / Processing / Troubleshooting sub-tabs: name, format
       ("VST3 (Apple - native)"), latency, Reload plugin, MIDI ports, preset Load/Save, GUI options, "Notify about
       parameter changes" (`fl-plugin-window-options.png`).
    3. *Miscellaneous functions* (channels only) — the channel settings page shared with every FL generator: levels
       (pan, vol, mod X/Y), polyphony, time (gate, shift, swing), keyboard tracking, group cut, arpeggiator, echo
       delay, root note and keyboard (`fl-plugin-window-misc.png`).
    An effect has only pages 1 and 2 (`fl-plugin-window-effect-detailed.png`). The window resizes to the page shown:
    568×466 pt for Miscellaneous, 571×487 for Plugin options, the plugin's size plus the strip for the editor; on the
    other pages the plugin's child window is hidden.
  - **Resizing:** the wrapper has no resize handle of its own. A plugin that resizes itself (Vital's corner handle)
    resizes the wrapper around it (1344×787 → 1108×649 view, wrapper 10 pt wider and 31 pt taller than the view
    without the strip). The size is kept when the window is closed and reopened.
  - **▸ menu** (`fl-plugin-window-menu.png`): Piano roll, Replace, Presets, Browse presets, Save preset as…, Add to
    plugin database, Display preset name; Spare state (Store, Flip); Parameters (Link all parameters…, Browse
    parameters); CPU (Allow threaded processing, Smart disable); Make editor thumbnail; Rename, color and icon… (F2),
    Change color…, Random color, Change icon…, Detached; and for a channel: Save channel state as…, Browse channel
    states, Route to free mixer track.
- **LimeShot equivalent (PLG-006):** see `features.json`. Each plugin channel and plugin insert has one wrapper window
  (its own OS window): a title bar row with Editor and Settings tabs (channels: the LimeShot Channel Settings page;
  effects: the parameter list), then the plugin's native view on the Editor tab. Session 15 added the right end of
  FL's title bar to the tab row: a presets menu (PLG-008), Presets ◀ ▶ for channels and the keyboard toggle; window
  places are saved in the project.

## 18. Main menu bar (session 13, outside-in pass)
- **OBSERVED live (2026-10-22, `fl-menu-*.png`; Accessibility dump of the mirrored macOS menu bar in
  `evidence/menus/fl-native-menubar-dump.txt`, made with `evidence/menus/menudump.swift`):** every item of FILE, EDIT,
  ADD, PATTERNS, VIEW, OPTIONS, TOOLS and HELP, with submenus, is recorded in `docs/roadmap/menus.json` (generated
  view: `menus.md`). Dialogs those items open are not documented yet (next inward pass).
- **Menu structure:** FL draws its own menus (upper-case labels left of the transport) and mirrors the same tree into
  the native macOS menu bar (File, Edit, Add, Patterns, View, Options, Tools, Help after the FL Studio app menu).
  In-window menus use grey **section headers** (View: Windows, Layout, Browser, News and notifications; Options: System,
  Project, Info, MIDI, Switches, 3rd party services; Tools: FL Cloud, Clipboard, Score logger, Audio logger, Last
  tweaked parameters, Macros; Help: Registration, Support, Shop, Internet links; File › Export: Audio, Video, Project).
  Long menus become columns (Add: three columns; Tools: Macros in a second column). Toggles show a dark check when on
  and a faint check when off; radio groups show a dot. Shortcuts are right-aligned in italics. The native mirror loses
  the headers and the enabled state (File › Save to new project folder… is disabled in-window, enabled in the mirror)
  and adds macOS's own Edit items (AutoFill, Dictation, Emoji).
- **File:** the first item names the default template, "New (Basic with limiter)". New from template shows templates as
  cards (title, description, icon) in six categories that match `Contents/Resources/FL/Data/Templates/<category>`.
  Recent projects are listed inline under a header with Opt+1…Opt+9. Cmd+N is Save new version, not New.
- **Edit:** only four items. The first is a single Undo/Redo slot naming the last action: after painting a clip it read
  "Undo playlist fill paint clip" (Cmd+Z); after undoing, "Redo playlist fill paint clip" (Cmd+Opt+Z). Cut/Copy/Paste
  act on the focused editor; Paste is disabled while the clipboard is empty.
  *Corrected in session 15 (observed again):* Undo and Redo are two items. Cmd+Z steps back through the history
  (after undoing the clip the item read "Undo step seq edit"), and "Redo <action>" (Cmd+Opt+Z) appears below it only
  while something can be redone.
- **Add:** generators grouped by category (Controller, Drum, MIDI, Misc, Patcher, Sampler, Synth classic, Synth special,
  Visual), a Categories/Simple/Tree view switch, and an Other column (Effect › …, Plugin picker F8, plugin database and
  preset browsing, fast rescan, Manage plugins, Automation for last tweaked parameter, Pattern).
- **Patterns:** a column listing the patterns sits left of the commands. Group selected…/Rename group…/Ungroup selected…
  appear in the native mirror but not in-window with a single pattern.
- **Tools › Last tweaked / Next to last tweaked:** the same menu as a control's right-click, headed by the parameter
  name ("No parameter" before anything was tweaked).
- **Confidence:** OBSERVED. **LimeShot equivalent:** `crates/limeshot-ui/src/menus.rs`; the per-item comparison is
  `menus.md` (APP-026 tracks the menu bar as a whole).

- **Patterns commands (observed live, session 15).** Split by channel on "Pattern 1" with an 808 Kick and an
  808 Snare step: the pattern became "808 Kick" (kick steps only) in the same slot, "808 Snare" was added after it,
  and a Playlist clip of the pattern stayed (now playing only the kick); one undo step ("Undo split by channel").
  Insert one: an empty pattern appeared before the current one and became current, unnamed patterns renumbered by
  position (the old "Pattern 1" showed as "Pattern 2"), and a name field opened; it was not an undo step.

## 19. Dialogs opened from the main menus (session 14)
- **OBSERVED live (2026-10-22, `fl-dlg-*.png`):** every control is recorded in `docs/roadmap/settings.json` (page =
  tab or dialog, section, control type, default value; generated view `settings.md`). Nothing was changed or saved:
  each dialog was opened, read and closed.
- **One settings window, ten tabs.** Options › MIDI / Audio / General / File / Theme settings, Project general settings,
  Project info and Debugging log all open the *same* window ("Settings – <page title>") on a different tab: MIDI, Audio,
  General, File, Theme | Project, Info | Debug, Account, About (three tab groups). Page titles: MIDI input / output
  devices, Audio devices / mixer, System / miscellaneous, Files & folders, Look and feel, Current project, Project
  credits, Debugging log. Pages are split into titled, collapsible sections (General: Display, Language, Input, Undo
  history, Hints and warnings, Updates and privacy, Miscellaneous, Advanced; it scrolls). Switches are round LEDs,
  choices are drop-down buttons with a ▸, numbers are red value fields. Settings apply at once; there is no OK/Cancel.
- **Render dialog.** File › Export › Wave file… (Cmd+R) first shows the macOS save panel (name, folder, File Format),
  then a window "Rendering to <file>": Project type (Mode Full song / Pattern; Tail Cut / Leave / Wrap remainder; length,
  time and disk space), Output format (several of WAV, MP3, OGG, FLAC, MID, M4A at once; WAV bit depth 16/24 int, 32
  float; Stereo / Mono merged / left / right; MP3/OGG bitrate, FLAC depth and compression), Quality (Resampling 2-point
  linear … 256-point sinc in three quality groups; HQ for all plugins; Disable maximum polyphony; Dithering),
  Miscellaneous (markers, tempo info, Split mixer tracks, Trim PDC silence, insert/master effects), Upload, Show file(s)
  when complete, Start. Nothing is written before Start. Esc closes an open drop-down, not the dialog.
- **Plugin Manager** (Options › Manage plugins) is a separate app with its own menu bar: Find installed plugins, Verify
  plugins, FL Studio folder, Plugin search paths (format + folder, checkable), Scan options, Always on top, and a
  sortable table (favourite, Name, Status, Format, Kind, Effect/Synth, Vendor, Filename; 315 plugins here) with Find.
- **Patterns › Set time signature…** opens the Piano Roll with a small "Time signature change" dialog (Numerator,
  Denominator, Accept). **Patterns › Transpose…** shows an inline "Semitones" field with ✓/✗ at the pointer.
  **File › New from template › Change default template…** showed nothing visible (UNKNOWN; the default template is also
  set in General › Miscellaneous). The **Welcome** window (`fl-dlg-welcome.png`): Start a new project cards (Launch Loop
  Starter, Create a chord progression, Remix a song, Record Audio, Default, Empty), News, Recent projects, Ask Gopher,
  "Do not show this in the future".
- **LimeShot equivalent:** an Audio settings dialog and an Export WAV dialog (range, sample rate, bit depth). APP-025
  (one settings window with pages) should follow FL's tab layout; the per-control gaps are in `settings.md`.

## 20. Toolbar (session 15, outside-in pass)
- **OBSERVED live (2026-10-22, synthetic input):** each control was hovered and FL's hint panel read for its name and
  keys (`fl-toolbar-hints-1.png`, `-2.png`; the whole bar in `fl-toolbar.png`), and the right-click menus opened
  (`fl-toolbar-record-menu.png`, `-tempo-menu.png`, `-time-menu.png`, `-cpu-menu.png`, `fl-toolbar-snap.png`). Every
  control is a record in `docs/roadmap/menus.json` under the menu "Toolbar" (54 records). Nothing was saved.
- **Row 1** (right of the menu bar): PAT/SONG ("Pattern or song mode / Show channel rack or playlist", L), Play,
  Stop (Space), Record ("automation, score, audio, clips", R; right-click: Recording filter Automation/Notes/Audio/
  Clips, Options Recording starts playback/Disarm on stop/Enable recording markers, Quantize Note start time/Note end
  time/Leave note duration/Record to step sequencer/Automations), Tempo (right-click: the control menu plus Tempo
  tapper Cmd+F5 and BPM presets from 80 in steps of 10), Song position (right-click: Mode Bar:Beat/Step:Tick,
  Minute:Second:Centisecond, SMPTE 23.976–120 fps; Middle unit Step/Beat), Typing keyboard to piano (Cmd+T),
  Recording precount (Cmd+P), Loop record, Metronome (Cmd+M), Detected chord, Monitor (master), Peak meter (master),
  CPU load (right-click: Refresh Slow/Medium/Fast, View voice count, View FPS) with memory and CPU load history.
- **Row 2:** Hint panel; window icons Browser / Plugin picker (Opt+F8), Channel rack (F6), Piano roll / Maximize
  (F7), Playlist / Maximize (F5), Mixer (F9); Auto scrolling; One-click audio recording; Snap panel with Main snap
  (Line, Cell, (none), 1/6–1/2 step, Step, 1/6–1/2 beat, Beat, Bar; the hint names BkSp); Pattern options (▸, the
  Patterns menu), Pattern selector (+/−), New pattern (find first / next empty); on-screen Shift / Command / Option
  modifier keys; Gopher (Opt+F1), Buy, and the Online content & events panel.
- A macOS permission prompt for this session's screen capture appeared during the capture and was left for the owner;
  it hides part of the tempo preset list and the SMPTE entries' right halves.

