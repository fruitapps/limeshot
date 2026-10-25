# Milestones

| # | Milestone | Deliverable | State |
|---|---|---|---|
| 0 | Research and architecture | ArtCraft and FL Studio research, REA, inventories, decisions | **Done** |
| 1 | Native app foundation | Workspace, windows, theme, menus, commands, project model, CI | **Done**; CI on Windows, Linux and macOS |
| 2 | Sequencer and audio | Device output, clock, patterns, channels, step sequencer, samples, transport, tempo | **Done** |
| 3 | Piano Roll and synth | Notes, tools, velocity/pan, polyphonic synth, scheduling | **Done** |
| 4 | Arrangement | Patterns, Playlist clips, Song mode, markers, time selection | **Done** |
| 5 | Mixing and project lifecycle | Mixer, routing, meters, save/load, undo, autosave/recovery | **Done** |
| 6 | Export and MVP verification | Offline render, WAV export, end-to-end tests | **Done** on macOS; hands-on Windows/Linux checks open |
| 7 | Editing depth | Copy/paste, quantize, swing, tool dialogs, scales, MIDI import/export | Core done; advanced FL tools remain |
| 8 | Effects and routing | Inserts, ten Lime effects, sends, sidechain, track EQ | Core done; Mixer groups and advanced routing remain |
| 9 | Audio and automation clips | Audio clips, compressed decoders, fades, automation clips, tempo automation | Done; crossfades and time stretching remain |
| 10 | Recording and MIDI | MIDI input, note and audio recording | Implemented; real-device checks, monitoring, latency compensation and count-in remain |
| 11 | Plugins | VST3 hosting, Lime Native suite, editor embedding | VST3 done (embedding partly verified); CLAP/AU remain |
| 12 | Distribution | Installers for Windows, macOS and Linux | **v0.1.0-alpha.1 released**; signing and notarisation remain |

Implementation and verification states live in [features.md](features.md); open restrictions are in
[known limitations](known-limitations.md).
