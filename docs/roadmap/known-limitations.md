# Known limitations

Current as of 2026-10-20. Everything here is either not implemented or not verified.

## Platforms
* Only macOS (Apple Silicon) has been fully exercised: build, tests, GUI and realtime audio. The macOS Intel build runs
  headless under Rosetta, but its GUI and audio were not exercised.
* Windows and Linux: CI jobs exist, but no CI run has happened yet (there is no remote repository). Windows type-checks
  locally. Linux was not built.

## Audio
* Output only: no audio input or recording, no ASIO, no JACK, no surround (extra device channels are silent).
* Switching output devices was tested only with the default device.
* WAV is the only sample format (no MP3/FLAC/OGG/AIFF).
* No effects of any kind, no sends, no insert-to-insert routing, no EQ.
* No automation, no tempo changes over time (one project tempo), one project-wide time signature.
* Sampler: no loop points, no time stretching, no filter; it declicks only on note-off. Linear interpolation (some
  aliasing when pitched far up).
* LimeSynth: no filter, LFO or modulation; mono/legato and glide are not available.
* The voice pool is fixed at 128 voices; the oldest voice is stolen abruptly (no fade).
* Integer WAV export is clipped, not dithered. Export is stereo only; there are no stems.
* The stream error callback logs through `log`, which may take a lock (error path only).

## Editing
* No copy/paste, no quantize, no swing, no tool palette (draw-tool semantics only), no scale highlighting, no chord or arp tools.
* Piano Roll ghost notes only show other channels in the same pattern.
* Pattern clips do not loop when longer than their pattern; clip offsets exist in the model but have no UI.
* No markers, loop regions or time selections; no audio or automation clips; no performance mode.
* No MIDI devices, MIDI recording or MIDI files.
* There is no confirmation when deleting a channel or pattern (undo restores it).

## Project and app
* No templates, autosave or crash recovery. Recent files have no Alt+number shortcuts.
* `.flp` files cannot be imported.
* No plugin hosting.
* Child windows cannot detach into OS windows. Default positions can overlap on small screens (View › Arrange
  windows, Cmd+Shift+H).
* Accessibility: custom-painted editors expose no accessibility labels.

## Research
* FL Studio was not observed live: Screen Recording permission was missing and the REA MCP tools need a Claude Code
  restart. Interaction details come from the official manual. Items marked INFERRED/UNKNOWN in
  `docs/research/fl-studio-analysis.md` still need confirmation.
