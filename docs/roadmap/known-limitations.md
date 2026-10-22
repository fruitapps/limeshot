# Known limitations

Current as of 2026-10-21 (session 9). Everything here is either not implemented or not verified.

## Platforms
* Only macOS (Apple Silicon) has been fully exercised: build, tests, GUI and realtime audio. The macOS Intel build runs
  headless under Rosetta, but its GUI and audio were not exercised.
* Windows and Linux: CI jobs exist, but no CI run has happened yet (there is no remote repository). Windows type-checks
  locally. Linux was not built.

## Audio
* Output only: no audio input or recording, no ASIO, no JACK, no surround (extra device channels are silent).
* Switching output devices was tested only with the default device.
* WAV is the only sample format (no MP3/FLAC/OGG/AIFF); this includes audio clips.
* Insert effects are built in only (EQ, filter, delay, reverb, compressor, limiter, utility). Mixer routing is
  post-fader only; there are no pre-fader sends and no delay tempo sync. Only Lime Comp takes a sidechain input
  (one per compressor), and the gain-reduction meter shows the track's largest compressor reduction.
* Each mixer track's built-in EQ (MIX-010) cannot be automated, and its width-to-Q mapping is LimeShot's own (FL
  Studio's exact curve shapes were not measured).
* Automation covers channel volume/pan, mixer volume/pan, effect parameters and hosted plugins' parameters in Song
  mode only; no tempo automation (one project tempo), no synth/sampler parameter automation, single-curve segments
  only. Automated controls do not move on screen, except inside a plugin's own editor. Add › Automation for last
  tweaked parameter knows plugin parameters only (other controls offer it on right-click). Automation clips are edited in their Playlist clip, whose curve area is small at the default track
  height.
* Audio clips are not time-stretched; there are no crossfades or fades on clip edges.
* Sampler: no loop points, no time stretching, no filter; it declicks only on note-off. Linear interpolation (some
  aliasing when pitched far up).
* LimeSynth: low-pass only (no high/band-pass, no key tracking); the LFO restarts with each note; mono/legato and
  glide are not available.
* The voice pool is fixed at 128 voices; the oldest voice is stolen abruptly (no fade).
* Integer WAV export is clipped, not dithered. Export is stereo only; there are no stems.
* The stream error callback logs through `log`, which may take a lock (error path only).

## Editing
* Clipboard is app-internal (not the system clipboard). Quantize has no dialog (100% starts only from the UI). Swing is
  global only (no per-channel mix). The Piano Roll has FL's eight tools but no Stamp
  tool (PR-027), no scale highlighting and no chord or arp tools; the Playlist has FL's nine tools, but Slip has no ripple edit (Shift+S) and Slice cuts only vertically (no diagonal or Shift+click slicing, no zero-cross snapping). Resizing notes
  from the left edge cannot be turned off. Older LimeShot versions drop a note's mute when they open a newer file.
* Piano Roll ghost notes only show other channels in the same pattern.
* The Playback tool scrubs notes only: the Playlist's line sounds the pattern clips' notes under it, not audio clips
  (no audio scrubbing). Notes start from their beginning when the line reaches them. FL Studio's Playback tool plays
  on from the pointer instead; LimeShot scrubs at the owner's request.
* Vertical zoom (Opt+wheel) was taken from FL Studio's Alt+wheel without a live check in FL.
* The Channel Rack has Fill each 2/4/8 steps and Rotate, but not FL's Advanced fill… dialog.
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
  (`docs/roadmap/keybindings.json`); settings are spread over menus and dialogs (`docs/roadmap/settings.json`).
* The menu bar has a subset of FL Studio's menus: no Tools menu, no section headers, no template, layout,
  recording-switch or pattern-management commands (`docs/roadmap/menus.md` lists all 194 missing items).
* One autosave slot per user: two LimeShot instances share it.
* `.flp` import (APP-022) converts channels, samples, levels, inserts, patterns, notes and the first arrangement's
  pattern and audio clips only. Plugin generators become default LimeSynth channels; plugin and effect states, mixer
  effects and routing, automation clip curves, layers, markers and further arrangements are dropped (the toast says
  how many). Test files were saved by FL 26.1.1 only; older versions are covered by FL's shipped demos importing
  without errors, not by checked values. Note properties other than key, position, length and velocity are ignored.
  LimeShot cannot write `.flp`.
* VST3 plugins: plugin editor windows work on macOS only (on Windows and Linux the Plugin tab lists the
  parameters); the windows have no ▸ menu, captionize or wrapper options page (FL's), the user cannot resize a
  resizable editor by its window edge (only by the plugin's own handle, as in FL). The presets menu lists factory
  programs and `.vstpreset` files only (no FL `.fst` presets, no preset browser in the Browser); some plugins name
  their programs only by number. Loading a preset is not an undo step. The keyboard toggle forwards typed
  characters only (not arrows or function keys) and has no effect where the platform gives keys to the plugin's
  view directly. Plugin parameter changes are not undo steps; delay compensation covers up to about 340 ms per path and 64
  delayed paths, and its trimming in exports cannot be switched off,
  no custom plugin folders, and a plugin that crashes takes LimeShot down unless it was set to run in a separate
  process (macOS and Linux; such a plugin's own editor is not shown).
  Plugin channels apply channel volume and pan automation once per block rather than every 64 frames. Plugin
  channels do not play audio clips. No CLAP or Audio Units. Verified on macOS only.
* Window positions, sizes and detached/maximised states are not remembered between sessions.
* Accessibility: custom-painted editors expose no accessibility labels.

## Research
* FL Studio and LimeShot are now driven with synthetic input (Accessibility granted, session 4). Still unobserved in
  FL: in-clip automation point dragging (synthetic drags did not register), the Playlist's STEP/SLIDE automation
  options, and many shortcuts marked INFERRED in `docs/roadmap/keybindings.json`.
* No person has used the GUI yet; synthetic real-input passes are recorded in `docs/development/progress.md`.
