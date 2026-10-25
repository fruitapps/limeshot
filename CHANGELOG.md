# Changelog

Notable changes per release. Feature details and evidence are in the
[feature inventory](docs/roadmap/features.md); open issues are in [known limitations](docs/roadmap/known-limitations.md).

## v0.1.0-alpha.1 — 2026-10-25

First prerelease. Installers for Windows x64 (setup and portable ZIP), macOS Apple Silicon and Intel (PKG and DMG)
and Linux x86-64 and ARM64 (`.deb` and tarball), each with the twelve Lime Native VST3s.

* **Pattern workflow:** Channel Rack with step sequencing, swing, fill, rotate, groups, Zip/Unzip and a graph
  editor; Piano Roll with FL's eight tools, tool dialogs, stamps, scale highlighting, ghost notes and velocity/pan
  lanes; Playlist with pattern, audio and automation clips, slip, slice, fades, markers and time selection; Mixer
  with routing, pre/post sends, sidechain, track EQ, ten insert slots, mute locks and meters.
* **Sound:** LimeSynth, Lime Sampler and ten Lime effects (EQ, Filter, Delay, Room, Comp, Limiter, Gate, Drive,
  Chorus, Utility), all as VST3s that also work in other hosts; WAV, MP3, FLAC, Ogg Vorbis and AIFF samples.
* **Plugins:** VST3 hosting with scanning, saved state, presets, parameter automation, delay compensation, an
  optional separate process per plugin, and plugin editors inside LimeShot windows on macOS and Windows.
* **Automation and recording:** Song-mode automation of volume, pan, tempo and plugin parameters with point modes;
  MIDI input, note recording and audio recording.
* **Files:** versioned `.lime` projects with atomic saves, numbered versions and autosave/recovery; WAV export of the
  song, a pattern or a time selection; MIDI import/export; partial FL Studio `.flp` import; `limeshot-cli` for
  headless rendering, playback and scripted commands.
* **Workspace:** FL Studio–style editor windows with maximise, detach and pinning, saved custom layouts, FL's main
  and editor menus (in part), a hint panel and a settings window.

Packages are unsigned (Windows) or signed ad hoc (macOS). Plugin editors are not embedded on Linux. Hands-on
checks on Windows, Linux and Intel macOS are still open.
