# MVP

The MVP is a functional music-production foundation: a user can create a short composition from scratch.

## Central acceptance workflow

| # | Step | How it is done in LimeShot | Verified by |
|---|---|---|---|
| 1 | Launch | `limeshot` | Manual (GUI opens, audio device opens) |
| 2 | New project | File › New / Cmd+N | Unit tests; manual |
| 3 | Set tempo | Toolbar tempo field | `tempo_change_moves_onsets`, MVP test |
| 4 | Add sampler channel | Add › Sampler, rack + Add, or drag a WAV from the Browser | MVP test |
| 5 | Load WAV drum | Browser double-click/drag, or Channel Settings › Load… | MVP test (bundled `Lime Kick.wav`) |
| 6 | Drum sequence | Channel Rack steps | `clicking_a_step_button_writes_a_note`, `steps_sound_exactly_on_the_grid` |
| 7 | Add synth channel | Add › LimeSynth | MVP test |
| 8 | Open Piano Roll | F7, or channel menu › Piano Roll | `window_shortcuts_toggle_editors` |
| 9 | Write melody | Piano Roll draw/move/resize | `piano_roll_click_adds_drag_moves_delete_removes`, `piano_roll_notes_play_and_release` |
| 10 | Audition | Space in Pattern mode; piano keys | `realtime_transport_loops_pattern_and_reports_position`, `preview_plays_without_transport` |
| 11 | More patterns | F4 / toolbar + / Patterns menu | `pattern_and_mode_shortcuts_with_undo` |
| 12 | Arrange | Playlist clicks | `clicking_the_playlist_places_the_current_pattern` |
| 13 | Song mode | PAT/SONG or L | `pattern_and_mode_shortcuts_with_undo` |
| 14 | Play arrangement | Space in Song mode | `song_mode_plays_the_arrangement`; manual realtime `limeshot-cli play` |
| 15 | Route to mixer | Channel mixer field (auto-assigned) | `mixer_volume_pan_and_mute_route_to_master`, MVP test |
| 16 | Volume and pan | Mixer faders/knobs, channel knobs | same, MVP test (pan checked on rendered audio) |
| 17 | Save | Cmd+S | MVP test |
| 18 | Close and reopen | Quit / Open | MVP test (`load` → equality) |
| 19 | Composition preserved | — | MVP test (`assert_eq!(reopened, project)`) |
| 20 | Export WAV | File › Export WAV / Cmd+R | MVP test |
| 21 | Verify export | — | MVP test reads the WAV back: rate, length, no clipping, kick onsets on every beat, melody only in bar 2 |

**Result:** the workflow passes as an automated test (`crates/limeshot-app/tests/mvp_workflow.rs`) through the same
commands the UI issues, and the individual UI interactions have their own headless UI tests, which CI runs on
Windows, Linux and macOS. On macOS the GUI and realtime audio were exercised by hand; a full hands-on run of all 21
steps is still to be recorded ([progress log](../development/progress.md)).

**MVP status: complete on macOS and shipped in v0.1.0-alpha.1. Hands-on GUI/audio verification on Windows and Linux
is open** (PF-003, PF-004). By the [definition of done](../development/principles.md), the MVP is finished once those platforms are verified.

## Mandatory components

All present: native window, LimeShot identity, toolbar, transport, tempo and time signature, Pattern and Song modes,
Channel Rack with step sequencer, Piano Roll, Playlist, basic Browser, Mixer, sampler, synth, realtime audio, note
model, save/load, WAV export, core shortcuts, undo/redo, feature tracking and an automated test suite. See the
`mvp: true` records in `features.json` for per-feature status.
