# Known limitations

Current as of 2026-10-22 (session 17). Everything here is either not implemented or not verified.

## Platforms
* Only macOS (Apple Silicon) has been fully exercised: build, tests, GUI and realtime audio. The macOS Intel build runs
  headless under Rosetta, but its GUI and audio were not exercised.
* Windows x86-64 and Linux x86-64/ARM64 pass native headless CI; Windows ARM64 passes a cross-build.
  Native GUI/audio-device checks remain open. Intel CI used a retired runner; its supported replacement needs a run.
  See [platform-validation.md](../development/platform-validation.md) for commit-specific evidence.

## Audio
* Output only: no audio input or recording, no ASIO, no JACK, no surround (extra device channels are silent).
* Switching output devices was tested only with the default device.
* MIDI input plays the selected channel from one device; it was checked with a virtual port, not a hardware
  keyboard. Controllers, pitch bend, MIDI output and sync are not used yet. Recording takes notes and (with the Audio filter) the audio input;
  audio recording was not tried with a real input yet, has no latency compensation or input monitoring, and there is
  no automation or clip recording, count-in or loop takes. A recorded note appears when it ends.
* Samples can be WAV, MP3, FLAC, Ogg Vorbis or AIFF (not AAC/M4A); exports are WAV only.
* Built-in insert effects include EQ, filter, delay, reverb, compressor, limiter, gate, drive, chorus/flanger/phaser and utility (Lime Drive aliases at high drive: no oversampling); VST3 inserts are supported. A route is pre- or
  post-fader per route (no per-track default); Lime Delay syncs to the tempo but not to tempo automation. Only Lime Comp takes a sidechain input
  (one per compressor), and the gain-reduction meter shows the track's largest compressor reduction.
* Presets for the built-in effects and instruments are LimeShot's own JSON files (no FL .fst import), and only
  plugin windows step through presets with ◀ ▶.
* Each mixer track's built-in EQ (MIX-010) cannot be automated, and its width-to-Q mapping is LimeShot's own (FL
  Studio's exact curve shapes were not measured).
* Automation covers channel volume/pan, mixer volume/pan, effect parameters and hosted plugins' parameters in Song
  mode only; no tempo automation (one project tempo), no synth/sampler parameter automation, single-curve segments
  only. Automated controls do not move on screen, except inside a plugin's own editor. Add › Automation for last
  tweaked parameter knows plugin parameters only (other controls offer it on right-click). Automation clips are edited in their Playlist clip, whose curve area is small at the default track
  height.
* Audio clips are not time-stretched; there are no crossfades or fades on clip edges.
* Sampler: volume ADSR, loop points and a low-pass filter are available. No envelope delay/hold stages or
  panning, cutoff or pitch envelopes; no time stretching. Linear interpolation aliases when pitched far up.
* LimeSynth: low-pass only (no high/band-pass, no key tracking); the LFO restarts with each note; mono/legato and
  glide are not available.
* The voice pool is fixed at 128 voices; the oldest voice is stolen abruptly (no fade).
* Integer WAV export is clipped, not dithered. Export is stereo only; there are no stems.
* The stream error callback logs through `log`, which may take a lock (error path only).

## Editing
* Clipboard is app-internal (not the system clipboard). The separate value clipboard supports channel/mixer volume/pan
  and built-in effect parameters; instrument and hosted-plugin host value menus, Edit events, Init song and controller
  linking remain unavailable. Cross-control paste uses LimeShot's automation mappings. Quantize… has start and duration strength only (no swing templates). Swing is
  global only (no per-channel mix). The Piano Roll has FL's eight tools and a Stamp
  menu (chords and scales; no snap to scale, and the highlighted scale is not saved), no Riff machine or chord progressions and no Claw machine; its tool dialogs (Strum, Arpeggiate, Randomize …) use sliders and cover fewer options than FL's (no arpeggio presets, levels for velocity and pan only), and the Articulate, Chop, Flip and LFO dialogs were designed without seeing FL's; the quick-zoom levels and Center are LimeShot's own; the Playlist has FL's nine tools, but Slip has no ripple edit (Shift+S) and Slice cuts only vertically (no diagonal or Shift+click slicing, no zero-cross snapping). Resizing notes
  from the left edge cannot be turned off. Older LimeShot versions drop a note's mute when they open a newer file.
* Piano Roll ghost notes only show other channels in the same pattern.
* The Mixer's track menu acts on one selected track (FL selects several). Moving, inserting or deleting a track
  restarts the plugins on the tracks that move (from their state, read just before). No FL colour dialog, gradient
  colours, icons, groups, docking, arming or Insert multiple….
* The Playlist's ⏵ menu covers Edit, Select, Zoom and Time markers in part: no Merge similar pattern clips, Merge
  automation clips, audio clip pitch/reverse keys, crossfade options, Consolidate, track groups, picker panel options
  or performance mode. Merged clips get a new pattern named "<first> (merged)" (FL's naming not checked). Insert space
  uses Cmd+Insert as FL does, a key most Mac keyboards lack (the menu works).
* The Playback tool scrubs notes only: the Playlist's line sounds the pattern clips' notes under it, not audio clips
  (no audio scrubbing). Notes start from their beginning when the line reaches them. FL Studio's Playback tool plays
  on from the pointer instead; LimeShot scrubs at the owner's request.
* Auto scrolling follows the existing playheads: Playlist in Song mode and Piano Roll in Pattern mode only.
  It starts off; enable it from Options or the toolbar. Paging thresholds are LimeShot's own.
* Vertical zoom (Opt+wheel) was taken from FL Studio's Alt+wheel without a live check in FL.
* The Channel Rack has Fill each 2/4/8 steps and Rotate, but not FL's Advanced fill… dialog. Its ⏵ menu has no Zip /
  Unzip, layers, per-channel swing mix or display options; Sort by offers name, type and colour only.
* The graph editor has the Velocity and Pan lanes only (no note pitch, release, fine pitch, Mod X/Y, shift or repeat).
* Pattern clips do not loop when longer than their pattern (FL Studio behaves the same, D14).
* Playlist markers are plain named points (no loop, key, time-signature or jump markers); the time selection is
  Playlist-only and not saved with the project. No performance mode.
* No MIDI devices or MIDI recording. MIDI files: notes, tempo and time signature only; exported files were only read
  back by LimeShot's own reader.
* There is no confirmation when deleting a pattern (undo restores it). Channel deletion asks first, as in FL Studio;
  once "Remember my choice" is ticked the question can only be turned back on in `settings.json`.
* The channel selection is not saved with the project (FL Studio saves it).

## Project and app
* No templates. Recent files have no Alt+number shortcuts. Keyboard shortcuts cannot be remapped
  (`docs/roadmap/keybindings.json`). The tabbed Settings window exposes implemented settings; FL-only controls remain absent or disabled (`docs/roadmap/settings.json`).
* The main menus and editor menus implement a subset of FL Studio actions. Tools, pattern commands and layouts exist;
  recording, templates and many advanced editor actions remain missing (`docs/roadmap/menus.md` tracks each item).
* One autosave slot per user: two LimeShot instances share it.
* `.flp` import (APP-022) converts channels, samples, levels, inserts, patterns, notes and the first arrangement's
  pattern and audio clips only. Plugin generators become default LimeSynth channels; plugin and effect states, mixer
  effects and routing, automation clip curves, layers, markers and further arrangements are dropped (the toast says
  how many). Test files were saved by FL 26.1.1 only; older versions are covered by FL's shipped demos importing
  without errors, not by checked values. Note properties other than key, position, length and velocity are ignored.
  LimeShot cannot write `.flp`.
* VST3 plugins: native plugin editor embedding is implemented for macOS only and awaits final live verification (on Windows and Linux the Plugin tab lists the
  parameters); the internal windows have no ▸ menu, captionize or wrapper options page (FL's). Fixed-size plugins
  retain a minimum wrapper size; resizable plugins negotiate their editor size. Native views hide completely
  when another window or popup covers them (partial native-view occlusion is not implemented). The presets menu lists factory
  programs and `.vstpreset` files only (no FL `.fst` presets, no preset browser in the Browser); some plugins name
  their programs only by number. Loading a preset is not an undo step. The keyboard toggle forwards typed
  characters only (not arrows or function keys) and has no effect where the platform gives keys to the plugin's
  view directly. Plugin parameter changes are not undo steps; delay compensation covers up to about 340 ms per path and 64
  delayed paths, and its trimming in exports cannot be switched off,
  no custom plugin folders, and a plugin that crashes takes LimeShot down unless it was set to run in a separate
  process (macOS and Linux; such a plugin's own editor is not shown).
  Plugin channels apply channel volume and pan automation once per block rather than every 64 frames. Plugin
  channels do not play audio clips. No CLAP or Audio Units. Processing tests and prior native OS-window checks were on macOS; the replacement embedded view is not yet verified.
* Window positions, sizes and detached/maximised states are not remembered between sessions unless saved as the
  custom layout (View › Arrange windows › Save custom layout, applied with Custom).
* Accessibility: custom-painted editors expose no accessibility labels.

## Research
* FL Studio and LimeShot are now driven with synthetic input (Accessibility granted, session 4). Still unobserved in
  FL: in-clip automation point dragging (synthetic drags did not register), the Playlist's STEP/SLIDE automation
  options, and many shortcuts marked INFERRED in `docs/roadmap/keybindings.json`.
* Owner hands-on feedback informed the interaction fixes; reproducible synthetic real-input passes are recorded in `docs/development/progress.md`.

* Session 17's Playlist track header menu exposes rename/color, auto name/reset, mute/solo/lock, selection, merge
  patterns, clip mute, clone linked sources, reorder, insert and delete. FL's track modes, grouping, icons, per-track
  heights, lock-to-content and consolidation are visible but disabled pending their model/engine implementations.
  Cloning tracks shares the clip sources; it does not clone instrument channels, plugins or mixer effects.
* Sampler playback heads use the transport/preview time, rather than a per-voice audio-thread playhead; when a sample
  is polyphonic the most recently started visible voice is shown. Previews honor trim, pitch and reverse.

* Session 17 native embedding is IMPLEMENTED, with final live resize/tab/keyboard verification pending. A native
  Vital view appeared during the first live pass, but subsequent window captures showed an empty editor. The
  terminal helper reports `AXIsProcessTrusted() == false`, so activation and input could not be verified reliably.
  Repeat with Accessibility enabled for the Codex execution host; test Vital and a resize-capable plugin, both tabs,
  covering/pinning windows, typing keyboard input, and reopening saved wrappers before calling this VERIFIED.
* The macOS package (`scripts/package-macos.sh`) is signed ad hoc: other Macs need a Developer ID signature and
  notarisation. Double-clicking a .lime file in Finder starts LimeShot without opening the file. No Windows or Linux
  packages yet.
