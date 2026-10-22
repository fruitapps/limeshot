# Milestones

| # | Milestone | Deliverable | State |
|---|---|---|---|
| 0 | Research and architecture | ArtCraft and FL Studio research, REA set up, inventory, decisions | **Done** (research evidence and live observation recorded; permissions vary by execution host) |
| 1 | Native app foundation | Workspace, window, layout, theme tokens, menus, commands, project model, CI | **Done** on macOS; CI defined, not yet run |
| 2 | Sequencer and audio | Device output, clock, patterns, channels, step sequencer, WAV playback, transport, tempo | **Done** (tests + realtime check) |
| 3 | Piano Roll and synth | Notes, editing, velocity, polyphonic synth, envelope, scheduling | **Done** |
| 4 | Arrangement | Multiple patterns, Playlist clips, Song mode, navigation | **Done** |
| 5 | Mixing and project lifecycle | Mixer, gain, pan, master, meters, save/load, undo coverage | **Done** |
| 6 | Export and MVP verification | Offline render, WAV export, end-to-end tests, cross-platform validation, docs | **Mostly done**: end-to-end test passes; Windows/Linux validation pending |
| 7 | Editing depth | Copy/paste, quantize, swing, tools, scale, MIDI import/export | Core implemented/tested; advanced tools remain |
| 8 | Effects and routing | Insert FX framework, core effects, sends, sidechain, EQ | Core implemented/tested; advanced routing remains |
| 9 | Audio and automation clips | Audio clips, more decoders, automation clips, tempo automation | WAV and control automation implemented; decoders/tempo remain |
| 10 | Recording and MIDI | MIDI input, recording, audio recording, count-in | Not started |
| 11 | Plugins | CLAP, VST3, AU hosting | VST3 implemented; embedded editor verification and CLAP/AU remain |
| 12 | Distribution | Installers, signing, autosave/recovery | Autosave/recovery implemented; packaging/signing remain |

Detailed implementation and verification states live in [features.md](features.md). Milestone completion does not
imply verification of every later interaction change; see [known limitations](known-limitations.md).
