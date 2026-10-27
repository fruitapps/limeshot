# Known limitations

Current as of v0.1.0-alpha.1 (2026-10-25). Everything here is either not implemented or not verified.

## Test coverage
* Hostile-input tests cover `.lime`, MIDI and `.flp` loading, plus 2,400 deterministic mutations across WAV
  (16/24-bit PCM and float), FLAC, AIFF and synthetic silent MP3. Successful audio imports are checked for bounded,
  finite stereo PCM. Invalid AIFF headers can panic inside Symphonia; format probing now converts an unwinding panic
  to a decode error. This corpus does not establish exhaustive decoder robustness, memory bounds inside third-party
  codecs, real-world MP3 fidelity or Windows/Linux results. Plugin-state tests cover malformed base64, decoded
  size boundaries and corrupt/truncated host envelopes; arbitrary payloads passed to third-party plugins remain
  unverified. UI edge cases and `limeshot-audio` device-loss paths still lack corrupt-input tests; coverage and fuzz
  tooling remain open
  ([agent implementation queue](../development/next-steps.md#implementation), item 8).

## Platforms and packages

Hands-on checks and prerequisites are tracked in [human verification needed](../development/human-verification.md).

* macOS Apple Silicon is the only platform exercised end to end by hand: build, tests, GUI, realtime audio and
  VST3 processing. The owner has run the Windows x64 build (plugin editors, Explorer drops); there has been no
  systematic hands-on pass on Windows, Linux or Intel macOS.
* Headless CI passes on Windows x86-64, Linux x86-64/ARM64 and macOS Apple Silicon/Intel; Windows ARM64 is a
  cross-build only. CI results are commit-specific: see [platform-validation.md](../development/platform-validation.md).
  CI 40463705283 at `586443f` aborted in Windows UI tests when concurrent Lime plugin creation reused a hidden
  window class name. The framework now uses a DLL-specific atomic sequence, with a Windows concurrency regression
  in native-plugin CI; this does not establish live GUI/audio verification.
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
* Recording takes notes and (with the Audio filter) raw audio input. Session-local monitoring supports Off / When
  armed / On into a chosen Mixer position; one/two-bar count-in is available. Linear rate conversion can underflow
  or discard overflow; switching idle monitoring/recording reopens the input. Device latency and independent input/
  output callback clocks are not compensated; sample-exact playback count-in does not establish physical input
  alignment. No automation/clip recording, punch or loop takes. A recorded note appears when it ends. Physical
  input/platform checks remain open; see [recording research](../research/recording-monitoring-and-count-in.md).
* Samples can be WAV, MP3, FLAC, Ogg Vorbis or AIFF (not AAC/M4A); exports are WAV or MP3, stereo only, without
  stems. MP3 is CBR at 32/44.1/48 kHz; no VBR, metadata tags or simultaneous formats. The Info/LAME gapless tag is written, but LimeShot reimport retains
  codec padding; use WAV for aligned loops. MP3 quality/listening and other-platform runtime checks
  remain open ([implementation and tests](../research/mp3-export.md)).
  Integer WAV export is clipped, not dithered.
* Instruments and inserts are the twelve Lime Native VST3s. Missing bundles abort export and show errors in the GUI;
  missing realtime channels stay silent and report load errors.
* Clip playback is a Lime Sampler extension (bounded, versioned SysEx); third-party samplers do not understand it.
  Clip fades and offsets use output time; changing the tempo does not stretch source audio.
* The Mixer track EQ is a dedicated hosted Lime EQ after all ten inserts, driven by the inline bands.
* Lime Sampler embeds up to six million stereo frames (64 MiB plugin state). Longer samples use Keep on disk,
  with a 2 GB decoded stereo float cap, bounded page reads and prepared waveforms. Disk sources must travel with
  the project; an evicted temporary cache is reconstructed from the original file. New uncached seeks or many
  scattered concurrent offsets may underrun; the native editor reports missed frames. Live disk stress and
  platform checks remain open. See [implementation and FL research](../research/alpha1-sampler-and-fades.md).
* Fades support independent single curves, with selected-clip length/tension gestures. Smooth S curves and
  automatic same-track pair crossfades and smooth S fades are implemented; nested/triple overlaps remain open; the new native bundles are required for curved fade playback.
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
  name programs only by number; loading a `.vstpreset` or factory program on a third-party or non-sampler plugin is not an undo step.
* The keyboard toggle forwards typed characters only (not arrows or function keys).
* Native-editor and third-party parameter changes on non-sampler plugins mark the project dirty rather than creating undo steps; known Lime host controls and completed native Lime Sampler edits are undo steps.
* Delay compensation covers up to about 340 ms per path and 64 delayed paths; export trimming cannot be turned off.
* Custom VST3 folders are managed in Settings › File; run Find plugins after changes to replace the saved
  database. Standard folders stay enabled; there is no per-root enable switch or separate Plugin Manager.
  Missing/unreadable roots are skipped and recursion is limited to eight levels. Loaded project instances
  remain active after a folder is removed. Live picker/rescan and Windows/Linux runtime checks remain pending.
  A crashing plugin takes LimeShot down unless it runs in a separate process (macOS and Linux). No CLAP or Audio Units.
* Value menus cover channel/mixer volume and pan and known Lime controls; third-party controls have the generic
  automation menu only.

## Sampler parity and integration

The [sampler audit](../research/sampler-parity-audit.md) distinguishes documented FL behavior,
Lime source findings, existing tests and unverified live behavior. Hosted P0 field migration does
not imply complete FL Sampler parity.

* Fixed-duration preparation is available; no realtime tempo following, source BPM lock/detection,
  formants, crossfaded loop seams, extended declick modes, metadata-driven sampler setup or
  selectable interpolation quality. Sampler has volume ADSR, ping-pong/loop gestures and
  low-/band-/high-pass outputs, without extended envelopes/LFOs, further filter families/tracking,
  configurable polyphony/mono/porta or cross-channel cut groups.
* Host and native root/source synchronization, preset sample preservation and gesture-coalesced undoable
  native sampler state capture are implemented and covered by headless/hosted tests (INS-021), but live
  GUI interaction verification remains open. Audio clips normally share sampler channel state; fixed-duration preparation isolates the
  selected clip, including across inactive arrangements. FL uses a distinct Audio Clip generator.
* FLP import defaults advanced sampler settings, including root/pitch, loops, envelopes, filter,
  stretch, declick and precomputed processing, without detailed per-setting warnings (APP-022).
* Exact audible FL parity and Lime Sampler in other DAWs remain unverified. Partial FL control
  observations are recorded in the [control inventory](../research/sampler-controls-2026-10-27.md).

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
  Shift+click slicing; zero-cross snapping requires decoded PCM). The ⏵ menu lacks Merge similar pattern clips, Merge automation clips,
  audio clip pitch/reverse, automatic crossfades, Consolidate, picker options and performance mode. Track modes,
  icons, lock-to-content and consolidation are shown disabled. Track grouping and per-track heights are implemented. Cloning a track shares its
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


## Alpha.1 Rev 1

See [Rev 1 implementation](../research/alpha1-rev1.md): resample-only right-edge stretch changes pitch
and all shared-sampler uses. Use Time stretching for fixed-duration independent/pitch-preserving
preparation; edge dragging retains its existing resampling behavior. Zero-crossing requires decoded
PCM and uses tick resolution. STEP pulse drawing/tension inheritance and full arrangement merge
position/track policies remain open; extra FLP arrangements are not imported. Track selectors display
musical activity rather than measured signal/tails or complete solo/mixer audibility. New focus options
are session-local. Live FL/macOS interaction comparison and Windows/Linux runtime verification remain
pending. Windows native plugin sluggishness and the plugin Settings wrapper overhaul are queued in
[next steps](../development/next-steps.md#owner-follow-up-alpha1-rev-1).


## Sampler loop/filter continuation

Lime supports forward/reverse/ping-pong loops and three independent TPT filter outputs. These do not
establish FL's exact fractional loop endpoints or filter transfer functions. No crossfaded seams,
additional FL filter families, sampler LFOs, extended delay/hold/tension envelopes or key/velocity
modulation yet. Loop chase preserves position/direction but does not reconstruct prior filter history.
Disk realtime cache misses remain possible at extreme pitch or short loops. Native loop gestures,
state/undo/reopen and standalone hosting still need live checks. See the
[control inventory and implementation](../research/sampler-controls-2026-10-27.md).

* [Prepared sampling](../research/prepared-sampling.md): WSOLA can repeat/smear transients and
  modulate polyphonic material; music listening and live FL/standalone/cross-platform checks remain
  unverified. Clip preparations create separate channels, reused for repeated private edits; removing clips
  can leave unused channels for explicit Rack removal. Ordinary channel Pitch/keyboard transposition still resample prepared audio. No long disk
  stretching; original, result and pitch intermediate each have a 3 million frame limit. Beat fitting
  captures the current tempo once. Host embedded previews cap at 4096 buffers/256 MiB and may be
  unavailable when mapped channel sources or active voices pin that budget. Existing mapped
  previews retain their PCM; replacing/removing sources releases their pins. Embedded states
  never use external reload hints as a fallback for unavailable previews. Active automatic pairs disable manual curve selection until automatic crossfades are turned off.
  Automatic crossfades use complementary gain, not
  equal-power gain, and affect adjacent two-clip overlaps on the same Playlist track only.
