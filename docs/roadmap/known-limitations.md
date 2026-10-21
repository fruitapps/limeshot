# Known limitations

Current as of 2026-10-20 (session 5). Everything here is either not implemented or not verified.

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
  post-fader only; there are no pre-fader sends, no sidechain, no delay tempo sync and no gain-reduction metering.
* Automation covers channel volume/pan, mixer volume/pan and effect parameters in Song mode only; no tempo automation
  (one project tempo), no synth/sampler parameter automation, single-curve segments only. Automated controls do not
  move on screen. Automation clips are edited in their Playlist clip, whose curve area is small at the default track
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
  global only (no per-channel mix). No tool palette (draw-tool semantics only), no scale highlighting, no chord or arp tools.
* Piano Roll ghost notes only show other channels in the same pattern.
* Pattern clips do not loop when longer than their pattern (FL Studio behaves the same, D14).
* No markers, loop regions or time selections; no performance mode.
* No MIDI devices or MIDI recording. MIDI files: notes, tempo and time signature only; exported files were only read
  back by LimeShot's own reader.
* There is no confirmation when deleting a channel or pattern (undo restores it).

## Project and app
* No templates. Recent files have no Alt+number shortcuts. Keyboard shortcuts cannot be remapped
  (`docs/roadmap/keybindings.json`); settings are spread over menus and dialogs (`docs/roadmap/settings.json`).
* One autosave slot per user: two LimeShot instances share it.
* `.flp` import (APP-022) converts channels, samples, levels, inserts, patterns, notes and the first arrangement's
  pattern and audio clips only. Plugin generators become default LimeSynth channels; plugin and effect states, mixer
  effects and routing, automation clip curves, layers, markers and further arrangements are dropped (the toast says
  how many). Test files were saved by FL 26.1.1 only; older versions are covered by FL's shipped demos importing
  without errors, not by checked values. Note properties other than key, position, length and velocity are ignored.
  LimeShot cannot write `.flp`.
* No plugin hosting.
* Window positions, sizes and detached/maximised states are not remembered between sessions.
* Accessibility: custom-painted editors expose no accessibility labels.

## Research
* FL Studio and LimeShot are now driven with synthetic input (Accessibility granted, session 4). Still unobserved in
  FL: in-clip automation point dragging (synthetic drags did not register), the Playlist's STEP/SLIDE automation
  options, and many shortcuts marked INFERRED in `docs/roadmap/keybindings.json`.
* No person has used the GUI yet; synthetic real-input passes are recorded in `docs/development/progress.md`.
