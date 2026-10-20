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
> **Limitation:** live screenshots and accessibility trees could not be captured. macOS Screen
> Recording permission is not granted to this session (REA `observe-native-ui` and `screencapture`
> both refused). Anything marked OBSERVED comes from file and window-list observation. Interaction
> details come from the official manual (VERIFIED against documentation, not against the running UI).
>
> Confidence labels: **VERIFIED** (installed artifact or official documentation), **OBSERVED**
> (seen on this machine), **INFERRED** (reasoned from evidence), **UNKNOWN**.

---

## 1. Main window composition
- **Expected/Observed:** One main window. A top toolbar strip with menus, transport, tempo, time display, pattern selector, snap selector, metronome/count-in/step-edit/typing-keyboard toggles, window shortcut icons and the CPU/memory panel. A browser docked on the left. The editors (Channel Rack, Piano Roll, Playlist, Mixer) are internal child windows that can be detached. The CG window list shows one 1680×1020 main window plus toolbar child strips (OBSERVED). Toolbar contents are VERIFIED from `toolbar_panels.htm`.
- **Interaction:** F5 Playlist, F6 Channel Rack, F7 Piano Roll, F8 Plugin Picker, F9 Mixer, Alt+F8 Browser, F12 close all, Esc close window, Enter maximize Playlist (VERIFIED, shortcuts page).
- **Confidence:** VERIFIED (docs) / OBSERVED (window list).
- **LimeShot equivalent:** `limeshot-ui` main layout: top toolbar panel, left Browser side panel, central workspace with toggleable child windows (egui `Window`, constrained to the workspace) bound to the same F-keys.

## 2. Channel Rack and step sequencer
- **Expected behaviour (VERIFIED, `channelrack.htm`):** Channels are shared by all patterns; each pattern holds its own step and piano-roll data per channel. Row layout: Mute, Pan, Volume, mixer-track target, channel button (name), selector LED, then step buttons or a piano-roll preview. Each step is a 16th note; the default is 16 steps; the maximum is 512. Left-click a step turns it on; right-click turns it off. Steps have no note-off. Ctrl+click on a mute button solos. Ctrl+L assigns selected channels to free mixer tracks. Alt+Up/Down reorders. Pattern length is Auto (ends at the last bar with data) or Manual. Global swing delays every second 16th. A Graph editor edits per-step velocity, pan, pitch and more.
- **Key model fact:** "Piano roll mode shows zero-length, on-beat notes as steps". Steps and piano-roll notes are **one note list**. A channel whose notes are not step-like shows a preview instead of step buttons.
- **Interaction sequence:** add a channel (Add menu or drag a sample in) → select a pattern → click steps → play (Space, PAT mode) → change tempo while playing.
- **Confidence:** VERIFIED (docs); visual layout not observed live.
- **LimeShot equivalent:** `ChannelRack` window. Rows carry mute/solo, pan knob, volume knob, mixer-target spinner, name button and step buttons. Steps are notes at `step × 240 ticks` with the channel's root key. Left-click toggles a step on, right-click toggles it off. A channel with non-step notes shows a mini preview (clicking it opens the Piano Roll). Pattern length is Auto or a fixed number of steps.

## 3. Piano Roll
- **Expected behaviour (VERIFIED, `pianoroll.htm`):** Pitch is vertical, time horizontal; the ruler shows bars. The target-channel dropdown picks which channel is edited. Tools: Draw (P), Paint (B), Delete (D), Mute (T), Slice (C), Select (E), Zoom (Z), Playback (Y). In Draw, left-click adds a note, dragging moves it, right-click deletes, right-drag deletes many, Alt bypasses snap, Ctrl selects. Resize by dragging the right edge. Shift+arrows nudge. Ctrl+B duplicates the selection. Ghost notes come from other channels. The lower pane edits note properties (velocity, pan, release, Mod X/Y, pitch). Left-clicking the preview keyboard auditions a note. Snap follows the global snap (Main) or a local value; right-click sets the default note length. Double-click a note opens Note Properties. Ctrl+Up/Down transposes by an octave.
- **Confidence:** VERIFIED (docs).
- **LimeShot equivalent:** custom-painted Piano Roll. Draw-tool semantics (left add/move/resize, right delete, Ctrl rectangle select, Alt no-snap), a velocity lane, keyboard auditioning, horizontal and vertical zoom and scroll, a playhead, Delete, Ctrl+A, Ctrl+B duplicate, Shift+Up/Down transpose by a semitone, Ctrl+Up/Down by an octave. Ghost notes, slice and paint tools are tracked (P1).

## 4. Playlist and arrangement
- **Expected behaviour (VERIFIED, `playlist.htm`):** Tracks are generic "clip tracks", not bound to mixer tracks. Clip types: pattern, audio, automation. Pattern clips reference a pattern; editing the pattern changes every instance until **Make unique**. Tools: Draw, Paint, Delete, Mute, Slip, Slice, Select, Zoom, Playback. In Draw, left-click places the current pattern, dragging moves, right-click deletes, Shift-drag on an edge resizes. Ctrl+B duplicates to the right. Snap options include Main/Line/Cell/beats/bar. Markers and time selection live on the timeline. Performance mode and Arrangements exist.
- **Song vs pattern playback:** the toolbar PAT/SONG switch (L key). SONG plays the Playlist; PAT loops the current pattern (VERIFIED, `toolbar_panels.htm`).
- **Confidence:** VERIFIED (docs). INFERRED: when a pattern clip is resized longer than its pattern, the extra region plays silence for piano-roll data. Step looping behaviour is not documented on this page and is marked UNKNOWN.
- **LimeShot equivalent:** Playlist window with tracks, pattern clips that reference patterns by `PatternId` (linked by default), "Make unique", draw/move/resize/delete/duplicate, a bar/beat ruler, a playhead, click-to-seek on the ruler, horizontal zoom and scroll, and snap to bar/beat/step. Song mode drives the engine from these clips. A pattern clip in LimeShot plays its pattern **once**, truncated to the clip length (documented choice).

## 5. Pattern management
- **VERIFIED:** Up to 999 patterns. The toolbar pattern selector supports rename, clone, insert, delete, find empty and render-to-audio. The Picker panel lists patterns.
- **LimeShot equivalent:** a pattern selector in the toolbar (prev/next/new/clone/rename/delete); patterns have stable ids and names.

## 6. Mixer
- **VERIFIED (`mixer.htm`):** Master plus up to 500 inserts plus a "Current" track. Per track: pan, fader (post-FX), stereo separation, phase invert, swap L/R, mute (left-click), solo (Ctrl/right-click), 10 FX slots, a 3-band EQ, peak meters in dB, record arm and PDC delay. Channels route to inserts (the default route for new channels is Master). Inserts route to Master by default; post-fader sends can go to other inserts; sidechains are supported. Insert tracks practically cannot clip; Master can.
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
- **LimeShot equivalent:** tracked (P1). The model reserves automation clips as a third `ClipKind`.

## 12. Keyboard shortcuts
See `basics_shortcuts.htm` (VERIFIED). The adopted subset is listed in `docs/architecture/ui.md`.

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
