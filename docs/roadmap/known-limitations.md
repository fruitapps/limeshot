# Known limitations

Current as of v0.1.0-alpha.1 (2026-10-25). Everything here is either not implemented or not verified.

## Platforms and packages
* macOS Apple Silicon is the only platform exercised end to end by hand: build, tests, GUI, realtime audio and
  VST3 processing. The owner has run the Windows x64 build (plugin editors, Explorer drops); there has been no
  systematic hands-on pass on Windows, Linux or Intel macOS.
* Headless CI passes on Windows x86-64, Linux x86-64/ARM64 and macOS Apple Silicon/Intel; Windows ARM64 is a
  cross-build only. CI results are commit-specific: see [platform-validation.md](../development/platform-validation.md).
* Packages ([release packaging](../development/release-packaging.md)) are unsigned on Windows (SmartScreen warns)
  and signed ad hoc on macOS (Gatekeeper blocks them until allowed; no Developer ID or notarisation).
  Double-clicking a `.lime` file in Finder starts LimeShot without opening the file. No AppImage, MSI, Flatpak or
  Windows ARM64 package exists, and the Windows executables carry no embedded icon (shortcuts and the installer use
  `LimeShot.ico`). The macOS PKG/DMG and the Linux `.deb`/tarball have been tested on CI runners, not on hardware.

## Audio
* Loaded VST3 module code stays resident until LimeShot exits, to keep framework callbacks valid; closing
  instances still releases their DSP and state. Restart LimeShot after updating a loaded plugin binary.
* No ASIO, JACK or surround output (extra device channels are silent). Switching output devices was tested only with
  the default device.
* MIDI input plays the selected channel from one device; it was checked with a virtual port, not a hardware
  keyboard. Controllers, pitch bend, MIDI output and sync are not used yet.
* Recording takes notes and (with the Audio filter) the audio input. Audio recording has not been tried with a real
  input, has no latency compensation or input monitoring, and there is no automation or clip recording, count-in or
  loop takes. A recorded note appears when it ends.
* Samples can be WAV, MP3, FLAC, Ogg Vorbis or AIFF (not AAC/M4A); exports are WAV only, stereo only, without stems.
  Integer WAV export is clipped, not dithered.
* Instruments and inserts are the twelve Lime Native VST3s. Missing bundles abort export and show errors in the GUI;
  missing realtime channels stay silent and report load errors.
* Clip playback is a Lime Sampler extension (bounded, versioned SysEx); third-party samplers do not understand it.
  Clip fades and offsets use output time; changing the tempo does not stretch source audio.
* The Mixer track EQ is a dedicated hosted Lime EQ after all ten inserts, driven by the inline bands.
* Lime Sampler embeds at most 6 million stereo frames at 8–384 kHz (about 125 seconds at 48 kHz, state below
  64 MiB); longer samples must be trimmed (no streaming from disk). Projects grow with their samples. Imports decode
  on the loader thread and report failures before any channel or clip exists; failed decodes are cached by path,
  size and modification time.
* Opening a `.lime` file, a recovery copy or an `.flp` does not embed audio from outside the project folder
  (symlinks included): choose the source in the GUI, or pass `--allow-external-samples` to the CLI.
* Plugins receive tempo at block boundaries; a tempo change inside a block reaches synced effects in the next block.
  Tempo automation steps every 64 frames, works in Song mode only, and minutes:seconds displays and new audio clip
  lengths use the current tempo (no integrated tempo map).
* Automation runs in Song mode. Effect automation from projects older than the Lime VST3 switch follows the plugin
  closely but not bit for bit (smoothing and interpolation); automation of the old compressor sidechain selector
  warns instead of converting.
* Sampler: ADSR, loop points and a low-pass filter; no delay/hold stages, extra modulation envelopes, time
  stretching or multisampling. Linear interpolation aliases when pitched far up. LimeSynth: low/band/high-pass at
  12 dB/oct and key tracking; its LFO restarts per voice; no mono/glide.
* Each Lime instrument has 128 voices and steals the oldest abruptly. Third-party plugins without Lime telemetry
  contribute zero to the voice count.
* The optional process bridge uses synchronous I/O with a timeout; bridged processing is not real-time safe, and a
  bridged plugin's own editor is not shown.
* The stream error callback logs through `log`, which may take a lock (error path only).

## Plugins
* Native editor embedding (PLG-006) works on macOS and Windows but is IMPLEMENTED, not VERIFIED: keyboard
  forwarding, external OS drops onto native views, pin/occlusion stress and host-edge resizing are unverified
  ([live checks](../development/live-plugin-checks-2026-10-25.md)). Linux shows the parameter list instead.
* Plugin windows have minimize/pin/close but no ▸ menu or wrapper options page. On macOS a covered native view
  hides completely; on Windows the editor popup is clipped around covering windows. At Windows display scaling above
  100 % the wrapper can be larger than a fixed-size editor, and keyboard focus may stay in a clicked plugin view
  until the host window is clicked again.
* Presets: factory programs and `.vstpreset` files (no FL `.fst`, no preset browser in the Browser); some plugins
  name programs only by number; loading a preset is not an undo step.
* The keyboard toggle forwards typed characters only (not arrows or function keys).
* Native-editor and third-party parameter changes are not undo steps; known Lime host controls are.
* Delay compensation covers up to about 340 ms per path and 64 delayed paths; export trimming cannot be turned off.
* No custom plugin folders; a crashing plugin takes LimeShot down unless it runs in a separate process (macOS and
  Linux). No CLAP or Audio Units.
* Value menus cover channel/mixer volume and pan and known Lime controls; third-party controls have the generic
  automation menu only.

## Editing
* The clipboard is app-internal (not the system clipboard). Edit events, Init song and controller linking are not
  available.
* Quantize… has start and duration strength only (no swing templates). Swing is global (no per-channel mix).
* Piano Roll: FL's eight tools and a Stamp menu (chords and scales; no snap to scale, the highlighted scale is not
  saved); no Riff machine, chord progressions or Claw machine. Tool dialogs use sliders and cover fewer options than
  FL's (no arpeggio presets; levels for velocity and pan only); Articulate, Chop, Flip and LFO were designed without
  seeing FL's. Quick-zoom levels and Center are LimeShot's own. Left-edge note resizing cannot be turned off.
* Piano Roll Target control edits velocity and pan only (no release, fine pitch, Mod X/Y or note-properties dialog,
  PR-016). Keyboard-label modes are session-local and hide below an 11-pixel key height. Ghost notes only show
  other channels in the same pattern.
* Pattern mode ignores time selections: the Piano Roll's ruler range selects and duplicates (Cmd+B) but does not
  loop Pattern playback.
* Playlist: FL's nine tools, but Slip has no ripple edit (Shift+S) and Slice cuts only vertically (no diagonal or
  Shift+click slicing, no zero-cross snapping). The ⏵ menu lacks Merge similar pattern clips, Merge automation clips,
  audio clip pitch/reverse, crossfades, Consolidate, track groups, picker options and performance mode. Track modes,
  grouping, icons, per-track heights, lock-to-content and consolidation are shown disabled. Cloning a track shares its
  clip sources; it does not clone instrument channels, plugins or mixer effects. Insert space uses Cmd+Insert, which
  most Mac keyboards lack (the menu works).
* Markers are plain named points (no loop, key, time-signature or jump markers); the time selection is not saved.
* The Playback tool scrubs notes only (not audio clips); FL plays on from the pointer instead.
* Auto scrolling starts off and follows the Playlist in Song mode and the Piano Roll in Pattern mode only.
* Vertical zoom (Opt+wheel) was taken from FL's Alt+wheel without a live check in FL.
* Channel Rack: no Advanced fill…, layers, per-channel swing mix or display options; Sort by offers name, type and
  colour only. Zip state is session-only. The graph editor has Velocity and Pan lanes only.
* Mixer: the track menu acts on one selected track; moving, inserting or deleting tracks restarts the plugins on the
  tracks that move. No FL colour dialog, gradients, icons, groups, docking, arming or Insert multiple…. Insert slots
  show only On and Mix; plugin parameters and presets are in the plugin window.
* Pattern clips do not loop when longer than their pattern (FL behaves the same, D14). Deleting a pattern does not
  ask (undo restores it). Once channel deletion's "Remember my choice" is ticked, the question can only be turned
  back on in `settings.json`. The channel selection is not saved with the project.
* MIDI files: notes, tempo and time signature only; exported files were only read back by LimeShot's own reader.
* Older LimeShot versions drop a note's mute when they open a newer file.

## Project and app
* Save new version has no shortcut. No templates. Recent files have no Alt+number shortcuts. Shortcuts cannot be
  remapped ([keybindings](keybindings.md)). The Settings window shows implemented settings only ([settings](settings.md)).
* The main and editor menus implement a subset of FL's actions ([menus](menus.md)).
* One autosave slot per user: two LimeShot instances share it.
* `.flp` import (APP-022) converts channels, samples, levels, inserts, patterns, notes and the first arrangement's
  pattern and audio clips. Plugin generators become LimeSynth channels; plugin and effect states, mixer effects and
  routing, automation curves, layers, markers and further arrangements are dropped (the toast says how many). Test
  files come from FL 26.1.1 only. LimeShot cannot write `.flp`.
* Window positions, sizes and detached/maximised states are not remembered between sessions unless saved as the
  custom layout (View › Arrange windows › Save custom layout).
* OS file-drop positions come from AppKit on macOS and the Win32 cursor on Windows; Linux uses egui's last pointer
  position, unchecked against a real file-manager drag.
* Custom-painted editors expose no accessibility labels.

## Lime Native standalone plugins (PLG-012)
* Hands-on checks in FL Studio and other DAWs are pending; Windows/Linux/Intel binaries are built and checked by CI
  only. Validation is recorded in [VALIDATION.md](../../native-plugins/VALIDATION.md).
* Stereo layouts only. No VST2, AU or CLAP (Logic Pro needs AU). Loading a sample resets sounding notes.
* Limiter is a zero-lookahead sample-peak limiter without true-peak oversampling. The EQ graph shows the actual
  response; the filter graph is a guide. Neither is a spectrum analyzer. Pitch bend is fixed at ±2 semitones.

## Research
* Still unobserved in FL: in-clip automation point dragging, the Playlist's STEP/SLIDE automation options, and
  shortcuts marked INFERRED in [keybindings](keybindings.md).
