# Known limitations

Current as of 2026-10-23 (VST3 replacement). Everything here is either not implemented or not verified.

## Platforms
* Only macOS (Apple Silicon) has been fully exercised: build, tests, GUI and realtime audio. The macOS Intel build runs
  headless under Rosetta, but its GUI and audio were not exercised.
* Windows x86-64 and Linux x86-64/ARM64 pass native headless CI; Windows ARM64 passes a cross-build.
  Native GUI/audio-device checks remain open. Intel headless CI now passes on `macos-15-intel`.
  See [platform-validation.md](../development/platform-validation.md) for commit-specific evidence.

## Audio
* No ASIO, JACK or surround output (extra device channels are silent). Audio recording is implemented;
  real input-device validation remains open.
* Switching output devices was tested only with the default device.
* MIDI input plays the selected channel from one device; it was checked with a virtual port, not a hardware
  keyboard. Controllers, pitch bend, MIDI output and sync are not used yet. Recording takes notes and (with the Audio filter) the audio input;
  audio recording was not tried with a real input yet, has no latency compensation or input monitoring, and there is
  no automation or clip recording, count-in or loop takes. A recorded note appears when it ends.
* Samples can be WAV, MP3, FLAC, Ogg Vorbis or AIFF (not AAC/M4A); exports are WAV only.
* Instruments and active inserts use the twelve Lime Native VST3 bundles; missing bundles abort export and produce GUI errors.
  Playlist audio windows/chase/overlap/fades, per-note pan, exact track EQ, Gate/Drive/Chorus, key-bus routing,
  host Lime value menus, dynamics reduction and native voice statistics are restored (PLG-017…029).
* Clip playback is a Lime Sampler extension; arbitrary third-party samplers do not understand its clip SysEx.
  Clip fades/offsets use output time without time stretching. Changing tempo does not stretch source audio.
* Track EQ uses a dedicated hosted processor after all ten inserts. Its hidden extension controls are driven by
  the undoable inline Mixer bands; ordinary Lime EQ inserts retain their original parameter IDs and ranges.
* Factory/user `.limepreset` files remain supported. Host unit-value menus cover known Lime controls;
  arbitrary third-party controls retain the generic automation menu. Compressor key-track selection is discrete,
  and automation of the legacy sidechain selector still warns instead of converting to dynamic bus routing.
* Embedded sampler state accepts at most four million stereo frames at 8–384 kHz. Conversion warns and retains
  source-path metadata and pending state when audio cannot fit; reassign a supported sample or reopen/reload after fixing the source.
  Incomplete conversions fail export explicitly. Loading a .lime file or recovery does not automatically embed
  outside-project audio (including symlink escapes): choose that source in the GUI, or explicitly use the CLI
  `--allow-external-samples` option when including outside recordings is intended.
* Embedded sampler audio is cached for host waveforms/auditions without writing temporary sample files.
  Native editor changes and platform/native GUI interaction still need broader hands-on validation.
* Plugins receive tempo context at block boundaries; a tempo change inside a block reaches synced effects in the next block.
* Automation runs in Song mode. Converted effect automation is re-expressed in plugin ranges; smoothing and curve
  differences mean automated output is close rather than bit-identical. Non-automated reference equivalence is exact.
* Sampler supports ADSR, loop points and a low-pass filter. No delay/hold stages, extra modulation envelopes or time
  stretching. Linear interpolation aliases when pitched far up. LimeSynth has low/band/high-pass and key tracking;
  its LFO restarts per voice and mono/glide are absent.
* Each Lime instrument has 128 voices and steals the oldest abruptly. The host counts actual native voices plus browser
  previews/metronome; third-party plugins without Lime telemetry contribute zero (PLG-028).
* Integer WAV export is clipped, not dithered. Export is stereo only; there are no stems.
* The stream error callback logs through `log`, which may take a lock (error path only).

## Editing
* Clipboard is app-internal (not the system clipboard). The separate value clipboard supports channel/mixer volume/pan
  and known hosted Lime instrument/effect parameters. Arbitrary third-party controls retain generic automation menus;
  Edit events, Init song and controller linking remain unavailable. Cross-control paste uses LimeShot's automation mappings. Quantize… has start and duration strength only (no swing templates). Swing is
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
* The Channel Rack has Fill each 2/4/8 steps and Rotate, but not FL's Advanced fill… dialog. Zip selected / Unzip all
  and right-click restoration are available. General settings can automatically zip empty channels on pattern switches;
  it waits for a switch after enabling, counts muted notes as data and leaves manual restoration usable until the
  next switch. Zip state is session-only; new/open projects start expanded. No live FL timing/parity pass performed.
  Its ⏵ menu has no layers, per-channel swing mix or display options; Sort by offers name, type and colour only.
* The graph editor has the Velocity and Pan lanes only (no note pitch, release, fine pitch, Mod X/Y, shift or repeat).
* Pattern clips do not loop when longer than their pattern (FL Studio behaves the same, D14).
* Playlist markers are plain named points (no loop, key, time-signature or jump markers); the time selection is
  Playlist-only and not saved with the project. No performance mode.
* MIDI files: notes, tempo and time signature only; exported files were only read
  back by LimeShot's own reader.
* There is no confirmation when deleting a pattern (undo restores it). Channel deletion asks first, as in FL Studio;
  once "Remember my choice" is ticked the question can only be turned back on in `settings.json`.
* The channel selection is not saved with the project (FL Studio saves it).

## Project and app
* Save new version is available from File only; it has no shortcut. No templates. Recent files have no Alt+number shortcuts. Keyboard shortcuts cannot be remapped
  (`docs/roadmap/keybindings.json`). The tabbed Settings window exposes implemented settings; FL-only controls remain absent or disabled (`docs/roadmap/settings.json`).
* The main menus and editor menus implement a subset of FL Studio actions. Tools, pattern commands and layouts exist;
  templates and many advanced editor actions remain missing (`docs/roadmap/menus.md` tracks each item).
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
  view directly. Native-editor and arbitrary third-party parameter changes are not undo steps; known Lime host controls use undoable state edits. delay compensation covers up to about 340 ms per path and 64
  delayed paths, and its trimming in exports cannot be switched off,
  no custom plugin folders, and a plugin that crashes takes LimeShot down unless it was set to run in a separate
  process (macOS and Linux; such a plugin's own editor is not shown).
  Hosted channel volume/pan uses per-frame gains captured every 64-frame automation interval. Lime Sampler
  handles audio clips; arbitrary third-party samplers do not implement the clip protocol. No CLAP or Audio Units. Processing tests and prior native OS-window checks were on macOS; the replacement embedded view is not yet verified.
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
* Tempo automation steps every 64 frames and works in Song mode only; minutes:seconds displays and new audio clip
  lengths use the tempo playing now (no integrated tempo map), and audio clips do not stretch with the tempo.



## Lime Native standalone plugins (PLG-012)

* `native-plugins/` exports independent VST3 instruments/effects for other DAWs; it does not require LimeShot.
  The supplied development builds are macOS Apple Silicon, ad-hoc signed, not notarized. Windows/Linux/Intel
  runtime and hands-on FL Studio/other-DAW editor checks remain pending; the development Mac was locked.
  Validator and actual VST3 ABI tests are recorded in `native-plugins/VALIDATION.md`.
* Stereo layouts only. No VST2/AU/CLAP export (Logic Pro needs AU). Sampler accepts WAV and embeds at most
  4 million stereo frames in host state. Loading a sample resets sounding notes. Sampler .limepreset files
  preserve the sample/root key; host state saves both. Loop points are supported; no time stretching or multisampling.
* Limiter is a zero-lookahead sample-peak limiter, without true-peak oversampling. The EQ graph shows the actual Eq3 frequency
  response; the filter graph is a response guide. Neither is a measured spectrum analyzer. Pitch bend is fixed at ±2 semitones; no MPE-specific expression controls.

## Lime VST3s as LimeShot's instruments and effects (PLG-013, replacement complete)

* GUI and headless apps convert legacy instruments and all ten effect kinds to Lime VST3s. Runtime built-in
  factories remain deleted. Hosted Playlist clips, note pan and exact track EQ have positive acceptance coverage.
* CI builds debug bundles; macOS packaging builds and copies release bundles into `Contents/PlugIns` (PLG-023).
  GitHub Actions remains blocked by owner account billing. Missing plugins fail CLI/GUI export; realtime missing
  channels remain silent and report load errors. No remote CI pass is claimed.
* Native clip/channel-pan events and processor telemetry use bounded, versioned, allocation-free SysEx messages.
  Stereo key audio, pan and telemetry also traverse the optional process bridge. Its existing synchronous I/O
  and timeout limitations remain; this work does not make bridged processing real-time safe.
* Broader native resizing/occlusion/keyboard/detached/third-party interaction, pluginval and Windows/Linux/Intel
  runtime are still open. See [current handoff](../development/handoff-hosted-p0-parity.md).
* Converted effect automation follows the plugin closely but not bit for bit (smoothing and interpolation).
  Non-automated equivalence is checked bit for bit, allowing only trailing silent frames to differ in length.
  Lime Sampler embeds sample audio, so projects grow with their samples.
