# LimeShot feature inventory

> Generated from [`features.json`](features.json) by `python3 scripts/features.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313). Updated 2026-10-20.

**IMPLEMENTED** = the code exists and meets its implementation criteria. **VERIFIED** = it also passed the listed
automated tests or recorded manual verification. Everything else is not done.

## Summary

- Features tracked: **217**
- MVP features: **69** — 67 implemented or verified, 44 verified
- By status: NOT_STARTED 120, RESEARCHED 1, IN_PROGRESS 6, IMPLEMENTED 37, VERIFIED 51, DEFERRED 2

| Category | Total | Verified | Implemented | In progress | Not started / other |
|---|---|---|---|---|---|
| Application | 22 | 10 | 4 | 1 | 7 |
| Channel Rack | 18 | 7 | 3 | 0 | 8 |
| Piano Roll | 25 | 3 | 8 | 0 | 14 |
| Playlist | 24 | 9 | 4 | 1 | 10 |
| Mixer | 16 | 4 | 2 | 0 | 10 |
| Instruments | 12 | 5 | 0 | 0 | 7 |
| Effects | 15 | 0 | 0 | 0 | 15 |
| Automation | 9 | 0 | 0 | 0 | 9 |
| Recording | 9 | 0 | 0 | 0 | 9 |
| Browser | 10 | 0 | 5 | 0 | 5 |
| Plugins | 10 | 0 | 0 | 0 | 10 |
| Export | 12 | 4 | 0 | 0 | 8 |
| Transport & Time | 13 | 7 | 4 | 0 | 2 |
| Workflow | 8 | 0 | 1 | 1 | 6 |
| UI | 8 | 1 | 5 | 0 | 2 |
| Platform | 6 | 1 | 1 | 3 | 1 |

## Application

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| APP-001 | New project | P0 | ✓ | IMPLEMENTED | File › New / Cmd+N: one empty pattern, 16 playlist tracks, master + 16 inserts; asks to save unsaved changes.<br>*The ask-to-save flow is exercised manually only.* | `limeshot-core model::tests::new_project_has_one_pattern_and_master` |
| APP-002 | Open project | P0 | ✓ | VERIFIED | File › Open / Cmd+O loads .lime files; damaged or newer files show an error and leave the open project untouched. | `limeshot-project tests::save_and_reload_is_identical`<br>`limeshot-project tests::refuses_newer_and_foreign_files`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| APP-003 | Save / Save as (atomic) | P0 | ✓ | VERIFIED | Writes a temp file, fsyncs, renames over the target; Save as picks a new path. | `limeshot-project tests::save_and_reload_is_identical`<br>`limeshot-project tests::overwriting_keeps_one_file` |
| APP-004 | Modified-state detection | P0 | ✓ | VERIFIED | Window title shows • when modified; undoing back to the saved state clears it. | `limeshot-core document::tests::modified_tracks_saves` |
| APP-005 | Undo / redo | P0 | ✓ | VERIFIED | Cmd+Z / Cmd+Shift+Z, 200 steps; drags merge into one step; failed commands change nothing.<br>*Deviation: FL's Ctrl+Z toggles undo/redo; LimeShot uses the platform convention.* | `limeshot-core document::tests::undo_and_redo_restore_state`<br>`limeshot-core document::tests::gesture_merges_into_one_undo_step`<br>`limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes` |
| APP-006 | Undo history list | P1 |  | IMPLEMENTED | Edit › History lists the last 30 steps (read-only). | — |
| APP-007 | Versioned project format with migrations | P0 | ✓ | VERIFIED | .lime JSON envelope with format/version; migrate() chain; newer versions refused. | `limeshot-project tests::refuses_newer_and_foreign_files` |
| APP-008 | Preserve unknown project data | P0 | ✓ | VERIFIED | Unknown fields on project/channel/pattern/clip/mixer track and unknown instrument types are kept and re-saved. | `limeshot-project tests::unknown_fields_survive`<br>`limeshot-core model::tests::unknown_instrument_round_trips` |
| APP-009 | Relative asset references | P0 | ✓ | VERIFIED | Sample paths inside the project folder are stored relative (with / separators), others absolute. | `limeshot-project tests::save_and_reload_is_identical` |
| APP-010 | Recent files | P1 |  | VERIFIED | File › Open recent lists the last 10 projects (persisted in settings).<br>*Alt+0..9 not bound.* | `limeshot-ui settings::tests::round_trip_and_recent_list` |
| APP-011 | Application settings persistence | P1 |  | VERIFIED | JSON in the platform config folder (directories crate): audio device, recent files, browser folders, UI scale. | `limeshot-ui settings::tests::round_trip_and_recent_list` |
| APP-012 | Audio device settings | P0 | ✓ | IMPLEMENTED | Options › Audio settings: device, sample rate and buffer size from what the device reports; Apply restarts the stream.<br>*Verified manually that the default device opens (MacBook Pro Speakers, 48 kHz). Switching devices has not been tested on hardware with several outputs. No ASIO.* | `limeshot-audio devices::tests::buffer_size_requests_are_validated` |
| APP-013 | Keyboard shortcuts for core actions | P0 | ✓ | VERIFIED | Space, Cmd+Space, L, F4–F9, Alt+F8, Cmd+Z/S/O/N/R/M/H/B/A/D, Delete, arrows; listed in Help › Keyboard shortcuts. | `limeshot-app tests/ui.rs::window_shortcuts_toggle_editors`<br>`limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo`<br>`limeshot-app tests/ui.rs::space_starts_and_stops_the_transport` |
| APP-014 | Ask to save on close | P0 | ✓ | IMPLEMENTED | Closing the window with unsaved changes asks Save / Don't save / Cancel.<br>*Manual test only.* | — |
| APP-015 | Project templates | P2 |  | NOT_STARTED | Templates folder with .lime files offered on New. | — |
| APP-016 | Autosave and crash recovery | P1 |  | NOT_STARTED | Background autosave to the config folder; offer recovery after a crash. | — |
| APP-017 | Project info (title, author, comments) | P2 |  | IN_PROGRESS | Title is stored and editable via SetTitle (no UI dialog yet). | — |
| APP-018 | Save new version | P2 |  | NOT_STARTED | File › Save new version. (LimeShot uses Cmd+N for New.) | — |
| APP-019 | Project bundle (zip with samples) | P2 |  | NOT_STARTED | Export project + samples as a single archive. | — |
| APP-020 | MIDI settings | P1 |  | NOT_STARTED | Device list, enable inputs, controller type. | — |
| APP-021 | Plugin manager | P2 |  | NOT_STARTED | Scan folders for CLAP/VST3/AU. | — |
| APP-022 | Import FL Studio projects (.flp) | P3 |  | RESEARCHED | Read a documented subset of .flp. Needs an independently documented format description.<br>*Clean-room: only from public/independent format documentation; no decompilation.* | — |

## Channel Rack

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| SEQ-001 | Step sequencing | P0 | ✓ | VERIFIED | Step buttons per channel; left-click/drag paints on, right-click/drag paints off; steps are notes at 240-tick steps with the channel's root key. | `limeshot-engine tests/audio.rs::steps_sound_exactly_on_the_grid`<br>`limeshot-engine tests/audio.rs::tempo_change_moves_onsets`<br>`limeshot-app tests/ui.rs::clicking_a_step_button_writes_a_note`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| SEQ-002 | Add sampler / synth channels | P0 | ✓ | VERIFIED | Add menu, Channel Rack + Add, or drop a WAV from the Browser. | `limeshot-core document::tests::new_channels_take_free_inserts`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| SEQ-003 | Channel mute and solo | P0 | ✓ | VERIFIED | LED: left-click mutes, right-click or Cmd+click solos. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves`<br>`limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| SEQ-004 | Channel volume and pan | P0 | ✓ | VERIFIED | Knobs (drag, Shift fine, wheel, double-click reset); default 78% volume. | `limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| SEQ-005 | Channel → mixer track routing | P0 | ✓ | VERIFIED | Mixer field per row (M = master), drag or type; new channels take the next free insert; Cmd+L routes to a free insert.<br>*Deviation: FL defaults new channels to Master.* | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master`<br>`limeshot-core document::tests::new_channels_take_free_inserts` |
| SEQ-006 | Pattern length (auto / manual) | P0 | ✓ | VERIFIED | Auto (whole bars covering the last note) or a fixed number of steps. | `limeshot-core model::tests::auto_pattern_length_covers_last_note` |
| SEQ-007 | Rename, recolor, clone, delete, reorder channels | P1 |  | IMPLEMENTED | Right-click channel button: rename, color, clone, delete, move up/down. | `limeshot-core document::tests::removing_channel_removes_its_notes` |
| SEQ-008 | Piano-roll preview in rack | P1 |  | IMPLEMENTED | Mini note preview; click opens the Piano Roll for that channel. | — |
| SEQ-009 | Channel settings window | P0 | ✓ | IMPLEMENTED | Click channel name: sampler (sample, waveform, gain, pitch, start/end, release, reverse, one-shot) or synth parameters, root key.<br>*Exercised by the offscreen screenshot and render-without-panic test; parameter widgets are standard egui sliders.* | — |
| SEQ-010 | Swing (global and per channel) | P1 |  | NOT_STARTED | Global swing in the rack header, per-channel amount; applied in the sequencer compile step. | — |
| SEQ-011 | Graph editor (per-step properties) | P2 |  | NOT_STARTED | Edit step velocity/pan/release/pitch in the rack. | — |
| SEQ-012 | Channel filter groups | P2 |  | NOT_STARTED | Named channel groups with a filter selector. | — |
| SEQ-013 | Zip / unzip channels | P3 |  | NOT_STARTED | Collapse rows. | — |
| SEQ-014 | Fill / rotate / advanced fill | P2 |  | NOT_STARTED | Channel menu fill and rotate commands. | — |
| SEQ-015 | Copy / paste steps between channels and patterns | P1 |  | NOT_STARTED | Copy the selected channel's notes and paste into another channel/pattern. | — |
| SEQ-016 | Channel step looping / ghost steps | P3 |  | NOT_STARTED | Per-channel loop length. | — |
| SEQ-017 | Loop Starter (FL Cloud) | P3 |  | DEFERRED | Out of scope: requires an online service (LimeShot is offline-first). | — |
| SEQ-018 | Multiple patterns | P0 | ✓ | VERIFIED | Toolbar pattern selector (prev/next/new/rename), F4 new, +/− switch; Patterns menu clone/delete. | `limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement`<br>`limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo` |

## Piano Roll

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PR-001 | Add, move, resize, delete notes | P0 | ✓ | VERIFIED | Same mouse conventions; Alt bypasses snap; Delete removes the selection. | `limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes`<br>`limeshot-core document::tests::notes_move_resize_and_duplicate` |
| PR-002 | Note selection (click, rectangle, select all) | P0 | ✓ | IMPLEMENTED | Cmd+click toggles, Cmd+drag rectangle, Cmd+A all, Cmd+D none, double-click a note selects its pitch row.<br>*Invert selection not implemented.* | — |
| PR-003 | Snap to grid | P0 | ✓ | VERIFIED | Global snap (bar, beat, ½ beat, step, ½, ¼ step, none); Alt bypasses. | `limeshot-core time::tests::snapping`<br>`limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes` |
| PR-004 | Velocity editing | P0 | ✓ | IMPLEMENTED | Velocity lane: click/drag a stem; applies to the selection when the note is selected.<br>*SetNoteVelocity is unit-tested through the engine; the lane drag is manual-tested only.* | `limeshot-engine tests/audio.rs::piano_roll_notes_play_and_release` |
| PR-005 | Zoom and scroll | P0 | ✓ | IMPLEMENTED | Wheel scrolls, Shift+wheel horizontal, Cmd+wheel zoom (anchored at the pointer), +/− buttons, taller keys. | — |
| PR-006 | Piano-key auditioning | P0 | ✓ | IMPLEMENTED | Click/drag on the keyboard plays the channel; new and dragged notes are auditioned. | `limeshot-engine tests/audio.rs::preview_plays_without_transport` |
| PR-007 | Playback position display | P0 | ✓ | IMPLEMENTED | Playhead line in Pattern mode; ruler click locates. | — |
| PR-008 | Duplicate selection (Ctrl+B) | P1 |  | VERIFIED | Cmd+B copies the selection right by its span rounded up to beats; copies become selected. | `limeshot-core document::tests::notes_move_resize_and_duplicate` |
| PR-009 | Transpose and nudge with keys | P1 |  | IMPLEMENTED | ↑/↓ semitone, Cmd+↑/↓ octave, ←/→ by snap. | — |
| PR-010 | Ghost notes | P1 |  | IMPLEMENTED | Notes of other channels in the same pattern are drawn as ghosts (not editable).<br>*Ghosts from overlapping playlist clips not implemented.* | — |
| PR-011 | Channel target selector | P0 | ✓ | IMPLEMENTED | Channel combo box in the header. | — |
| PR-012 | Paint, slice, mute, stamp, zoom tools | P1 |  | NOT_STARTED | Tool selector with paint, slice, mute, chord stamp. | — |
| PR-013 | Scale highlighting / snap to scale | P2 |  | NOT_STARTED | Scale selector, shading rows out of key, optional snapping. | — |
| PR-014 | Quantize | P1 |  | NOT_STARTED | Quantize command with strength. | — |
| PR-015 | Chords, arpeggiator, strum, humanize, legato | P2 |  | NOT_STARTED | Tools menu with these transforms as commands. | — |
| PR-016 | Note properties: pan, release, mod X/Y, fine pitch | P2 |  | NOT_STARTED | Additional lane targets stored per note. | — |
| PR-017 | Slide and portamento notes | P2 |  | NOT_STARTED | Slide flag on notes interpreted by LimeSynth. | — |
| PR-018 | Note color groups | P3 |  | NOT_STARTED | Per-note color group. | — |
| PR-019 | Copy / cut / paste notes | P1 |  | NOT_STARTED | Clipboard of notes relative to the selection start. | — |
| PR-020 | MIDI file import / export | P1 |  | NOT_STARTED | Standard MIDI File reader/writer (own implementation). | — |
| PR-021 | MIDI recording into the Piano Roll | P1 |  | NOT_STARTED | Record from MIDI devices/typing keyboard. | — |
| PR-022 | Time signature per pattern / time markers | P3 |  | NOT_STARTED | Project-wide signature only for now. | — |
| PR-023 | Glue / slice notes | P2 |  | NOT_STARTED | Commands to glue touching notes and slice at a time. | — |
| PR-024 | Resize from the left edge | P2 |  | NOT_STARTED | Drag left edge to change start and length. | — |
| PR-025 | Waveform helper background | P3 |  | NOT_STARTED | Show a sample waveform behind the grid. | — |

## Playlist

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PL-001 | Pattern clips (linked instances) | P0 | ✓ | VERIFIED | Left-click empty space places the current pattern; every clip of a pattern plays the same data. | `limeshot-core document::tests::clips_are_linked_until_made_unique`<br>`limeshot-sequencer tests::playlist_flattens_linked_clips`<br>`limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern` |
| PL-002 | Move clips (time and track) | P0 | ✓ | VERIFIED | Drag moves the selection with snap; arrows nudge. | `limeshot-core document::tests::clip_duplicate_and_move` |
| PL-003 | Resize clips | P0 | ✓ | IMPLEMENTED | Drag the right edge; content is truncated (pattern plays once).<br>*Deviation/INFERRED: LimeShot does not loop a pattern inside a longer clip.* | `limeshot-sequencer tests::clip_truncates_notes` |
| PL-004 | Delete clips | P0 | ✓ | VERIFIED | Right-click or right-drag deletes; Delete key removes the selection. | `limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern` |
| PL-005 | Duplicate clips (Ctrl+B) | P0 | ✓ | VERIFIED | Cmd+B or clip menu › Duplicate. | `limeshot-core document::tests::clip_duplicate_and_move` |
| PL-006 | Select clips | P0 | ✓ | IMPLEMENTED | Click, Cmd+click toggle, Cmd+drag rectangle, Cmd+A / Cmd+D. | — |
| PL-007 | Snap clips to grid | P0 | ✓ | VERIFIED | Uses the global snap; Alt bypasses. | `limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern` |
| PL-008 | Zoom, scroll, bar/beat ruler | P0 | ✓ | IMPLEMENTED | Cmd+wheel zoom, wheel/Shift+wheel scroll, ruler with bar numbers. | — |
| PL-009 | Playhead and set song position | P0 | ✓ | IMPLEMENTED | Ruler click/drag sets the song position and switches to Song mode; Home returns to start. | — |
| PL-010 | Song mode drives the engine | P0 | ✓ | VERIFIED | Song program = flattened clips; loops at the end (toggle) or stops. | `limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement`<br>`limeshot-engine tests/audio.rs::song_stops_at_the_end_without_loop`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| PL-011 | Make unique | P0 | ✓ | VERIFIED | Clip ▾ menu › Make unique clones the pattern for that clip. | `limeshot-core document::tests::clips_are_linked_until_made_unique` |
| PL-012 | Playlist tracks: add, rename, mute | P0 | ✓ | VERIFIED | + Track; double-click to rename; LED mutes the track. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves` |
| PL-013 | Clip mute | P1 |  | VERIFIED | Clip menu › Mute clip. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves` |
| PL-014 | Audio clips | P1 |  | NOT_STARTED | Audio clip source type rendered by the engine. | — |
| PL-015 | Automation clips | P1 |  | NOT_STARTED | Third clip source type. | — |
| PL-016 | Markers and loop region / time selection | P1 |  | NOT_STARTED | Markers and a loop range for playback/export. | — |
| PL-017 | Slip, slice clips | P2 |  | NOT_STARTED | Slip content inside clip; slice clip at a time. | — |
| PL-018 | Track grouping | P2 |  | NOT_STARTED | Collapsible track groups. | — |
| PL-019 | Performance mode | P3 |  | NOT_STARTED | Clip launch grid. | — |
| PL-020 | Arrangements | P2 |  | NOT_STARTED | Several playlists sharing channels/mixer. | — |
| PL-021 | Picker panel | P2 |  | NOT_STARTED | Panel listing patterns to drag in. | — |
| PL-022 | Consolidate / render to audio clip | P2 |  | NOT_STARTED | Render a selection to an audio clip. | — |
| PL-023 | Crossfades and time stretching | P2 |  | NOT_STARTED | Fades and stretch on audio clips. | — |
| PL-024 | Clip offset (start inside pattern) | P2 |  | IN_PROGRESS | Model and sequencer support `offset`; no UI to set it yet. | — |

## Mixer

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| MIX-001 | Master and insert tracks | P0 | ✓ | VERIFIED | Master + 16 inserts (up to 128); inserts sum into the master. | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-002 | Volume faders | P0 | ✓ | VERIFIED | Fader 0…+6 dB with a perceptual travel; Shift fine; double-click resets. | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-003 | Pan | P0 | ✓ | VERIFIED | Pan knob; balance law (centre unity, never boosts). | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master`<br>`limeshot-dsp tests::pan_law` |
| MIX-004 | Mute and solo | P0 | ✓ | VERIFIED | M and S buttons; right-click/Cmd+click on M solos; master never solo-muted. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves`<br>`limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-005 | Level meters and clip indicator | P0 | ✓ | IMPLEMENTED | Stereo peak meters (−60…+6 dB, 20 dB/s fall-back) per strip and in the toolbar; master clip LED.<br>*Peak values come from the engine (tested); meter drawing is manual-tested.* | — |
| MIX-006 | Rename and add mixer tracks | P1 |  | IMPLEMENTED | Double-click name to rename; + adds a track. | — |
| MIX-007 | Insert effect slots | P1 |  | NOT_STARTED | Ordered effect slots processed before the fader. | — |
| MIX-008 | Sends between tracks | P1 |  | NOT_STARTED | Post-fader sends to other inserts with feedback prevention. | — |
| MIX-009 | Sidechain routing | P2 |  | NOT_STARTED | Sidechain inputs for dynamics. | — |
| MIX-010 | Built-in 3-band EQ | P1 |  | NOT_STARTED | Per-track 3-band parametric EQ. | — |
| MIX-011 | Stereo separation, phase invert, swap L/R | P2 |  | NOT_STARTED | Per-track stereo controls. | — |
| MIX-012 | Plugin delay compensation | P2 |  | NOT_STARTED | Latency reported by effects aligned across paths. | — |
| MIX-013 | Insert-to-insert routing | P1 |  | NOT_STARTED | Insert output target other than master. | — |
| MIX-014 | Current track (C) | P3 |  | NOT_STARTED | — | — |
| MIX-015 | Mixer presets / track states | P3 |  | NOT_STARTED | — | — |
| MIX-016 | External audio I/O per track | P2 |  | NOT_STARTED | Per-track device inputs/outputs. | — |

## Instruments

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| INS-001 | Sampler: WAV loading and playback | P0 | ✓ | VERIFIED | WAV (8/16/24/32-bit int, 32-bit float), mono duplicated; decoded on a loader thread; polyphonic voices; one-shot or gated. | `limeshot-dsp sample::tests::wav_round_trip_all_depths`<br>`limeshot-dsp sample::tests::mono_is_duplicated`<br>`limeshot-audio assets::tests::loads_in_the_background_and_reports_failures`<br>`limeshot-engine tests/audio.rs::steps_sound_exactly_on_the_grid`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| INS-002 | Sampler: gain, pitch, start/end, reverse, release | P0 | ✓ | VERIFIED | Gain (dB), pitch (semitones, resampling), start/end fractions, reverse, release (ms), root key. | `limeshot-engine tests/audio.rs::sampler_pitch_start_end_and_reverse` |
| INS-003 | LimeSynth: oscillators | P0 | ✓ | VERIFIED | Sine, square, saw, triangle (PolyBLEP), octave, optional detuned second oscillator. | `limeshot-dsp osc::tests::shapes_have_the_right_pitch_and_range`<br>`limeshot-engine tests/audio.rs::piano_roll_notes_play_and_release` |
| INS-004 | LimeSynth: ADSR envelope | P0 | ✓ | VERIFIED | Linear ADSR with release from current level. | `limeshot-dsp envelope::tests::stages_follow_the_settings`<br>`limeshot-dsp envelope::tests::release_during_attack_starts_from_current_level` |
| INS-005 | Polyphony and voice stealing | P0 | ✓ | VERIFIED | 128-voice pool shared by all channels; oldest voice stolen; no allocation. | `limeshot-engine tests/audio.rs::synth_polyphony_and_voice_limit` |
| INS-006 | Synth filter and modulation (LFO, filter env) | P1 |  | NOT_STARTED | State-variable low-pass with envelope and LFO. | — |
| INS-007 | Sampler loop points, time stretch, filter | P1 |  | NOT_STARTED | Loop region, ping-pong, filter. | — |
| INS-008 | Additional audio formats (MP3, FLAC, OGG, AIFF) | P1 |  | NOT_STARTED | Decoder crate (e.g. symphonia) behind the SampleData interface. | — |
| INS-009 | Multi-sample / drum kit instrument | P2 |  | NOT_STARTED | Key zones mapping samples. | — |
| INS-010 | Instrument presets | P2 |  | NOT_STARTED | Preset files for sampler/synth parameters. | — |
| INS-011 | Per-channel polyphony limit / mono / porta | P2 |  | NOT_STARTED | Max voices, mono mode, glide. | — |
| INS-012 | Drum synthesizer | P3 |  | NOT_STARTED | — | — |

## Effects

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| FX-001 | Effect processing framework (insert chain) | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-002 | Parametric equalizer | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-003 | Filters (LP/HP/BP) | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-004 | Compressor | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-005 | Limiter / maximiser | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-006 | Noise gate | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-007 | Reverb | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-008 | Delay | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-009 | Distortion / saturation / waveshaper | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-010 | Chorus / flanger / phaser | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-011 | Pitch shifting | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-012 | Stereo processing | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-013 | Utility (gain, phase, balance, mono) | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-014 | Effect presets | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-015 | Analysers (spectrum, oscilloscope) | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |

## Automation

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| AUT-001 | Automation clips with envelope points | P1 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-002 | Curve shapes and interpolation | P1 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-003 | Automate mixer and channel parameters | P1 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-004 | Record automation from control movements | P2 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-005 | Event editor | P2 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-006 | Parameter linking to controllers | P2 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-007 | LFO / formula / peak controllers | P3 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-008 | Tempo automation | P2 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |
| AUT-009 | Plugin parameter automation | P2 |  | NOT_STARTED | Planned; the model reserves a third clip source type and the engine renders parameter values per block. | — |

## Recording

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| REC-001 | Audio recording from inputs | P1 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-002 | MIDI recording | P1 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-003 | Input selection and monitoring | P1 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-004 | Punch in/out | P2 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-005 | Loop recording / takes | P2 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-006 | Recording latency compensation | P2 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-007 | Count-in | P2 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-008 | Recorded clip management | P2 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |
| REC-009 | Step edit mode | P3 |  | NOT_STARTED | Planned. The toolbar Record button is shown disabled with a 'not implemented' hint. | — |

## Browser

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| BR-001 | Folder navigation (bundled + user folders) | P0 | ✓ | IMPLEMENTED | Bundled LimeShot samples plus folders added with +; collapsible tree; ⟳ re-reads. | — |
| BR-002 | Sample preview | P0 | ✓ | IMPLEMENTED | Click a WAV to preview it through the engine (master).<br>*Engine preview path verified by unit tests; UI click manual-tested.* | — |
| BR-003 | Load sample into channel / new channel | P0 | ✓ | IMPLEMENTED | Double-click loads into the selected sampler (or creates one); context menu; drag onto the rack or a channel name. | — |
| BR-004 | Search / filter | P1 |  | IMPLEMENTED | Substring filter on file names.<br>*No wildcards/boolean logic.* | — |
| BR-005 | Current project tree | P1 |  | IMPLEMENTED | Lists patterns (click to select) and channels. | — |
| BR-006 | Favorites / starred | P2 |  | NOT_STARTED | Star files; favorites section. | — |
| BR-007 | Tags | P3 |  | NOT_STARTED | — | — |
| BR-008 | Plugin and preset browser | P2 |  | NOT_STARTED | — | — |
| BR-009 | Online content | P3 |  | DEFERRED | Out of scope (offline-first). | — |
| BR-010 | Drag samples to the Playlist (audio clips) | P1 |  | NOT_STARTED | — | — |

## Plugins

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PLG-001 | CLAP hosting | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-002 | VST3 hosting | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-003 | Audio Unit hosting (macOS) | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-004 | Plugin scanning | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-005 | Plugin state persistence | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-006 | Plugin editor windows | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-007 | Plugin parameter automation | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-008 | Plugin preset management | P3 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-009 | Plugin latency reporting | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-010 | Plugin crash isolation | P3 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |

## Export

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| EXP-001 | WAV export (offline bounce) | P0 | ✓ | VERIFIED | File › Export WAV / Cmd+R: renders on a background thread with the same renderer as playback; notes ring out up to 4 s. | `limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end`<br>`limeshot-engine tests/audio.rs::block_size_does_not_change_the_output`<br>`limeshot-app tests/mvp_workflow.rs::demo_song_renders_and_reloads_identically` |
| EXP-002 | Sample-rate selection | P0 | ✓ | VERIFIED | 44.1, 48, 88.2, 96 kHz. | `limeshot-engine tests/audio.rs::onsets_scale_with_sample_rate`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| EXP-003 | Bit-depth selection | P0 | ✓ | VERIFIED | 16-bit, 24-bit PCM, 32-bit float. | `limeshot-dsp sample::tests::wav_round_trip_all_depths` |
| EXP-004 | Pattern export | P1 |  | VERIFIED | Export range: current pattern (one pass). | `limeshot-engine tests/audio.rs::steps_sound_exactly_on_the_grid` |
| EXP-005 | MP3 export | P1 |  | NOT_STARTED | Needs a permissively licensed encoder; not started. | — |
| EXP-006 | FLAC / OGG export | P2 |  | NOT_STARTED | — | — |
| EXP-007 | Dithering | P2 |  | NOT_STARTED | TPDF dither for 16/24-bit. | — |
| EXP-008 | Normalization | P2 |  | NOT_STARTED | — | — |
| EXP-009 | Stem export (per mixer track) | P2 |  | NOT_STARTED | — | — |
| EXP-010 | Export ranges (time selection, loop) | P2 |  | NOT_STARTED | — | — |
| EXP-011 | File metadata | P3 |  | NOT_STARTED | — | — |
| EXP-012 | MIDI export | P1 |  | NOT_STARTED | — | — |

## Transport & Time

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| TR-001 | Realtime audio output | P0 | ✓ | VERIFIED | cpal output (CoreAudio/WASAPI/ALSA); lock-free control queue; snapshots returned for freeing off the audio thread; silent clock when no device.<br>*Manual: `limeshot-cli play out/demo.lime` on macOS 26 (MacBook Pro Speakers, 48 kHz): 563 callbacks in 6 s, peak −3.7 dBFS, 1.1% load, position 4:01 at 6 s (correct for 124 BPM).* | `limeshot-audio tests::silent_player_runs_the_transport`<br>`limeshot-audio tests::device_writer_handles_mono_and_wide_devices`<br>`limeshot-engine tests/audio.rs::realtime_transport_loops_pattern_and_reports_position` |
| TR-002 | Play / pause / stop | P0 | ✓ | VERIFIED | Toolbar buttons and Space/Cmd+Space; stop returns to where playback started. | `limeshot-app tests/ui.rs::space_starts_and_stops_the_transport`<br>`limeshot-engine tests/audio.rs::realtime_transport_loops_pattern_and_reports_position` |
| TR-003 | Pattern / Song mode | P0 | ✓ | VERIFIED | Toolbar PAT/SONG switch and L. | `limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo`<br>`limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement` |
| TR-004 | Tempo | P0 | ✓ | VERIFIED | Toolbar field (drag/type), 10–522 BPM stored as milli-BPM; changes while playing keep the musical position. | `limeshot-core time::tests::tempo_clamps_and_formats`<br>`limeshot-engine tests/audio.rs::tempo_change_while_playing_keeps_position`<br>`limeshot-engine tests/audio.rs::tempo_change_moves_onsets` |
| TR-005 | Time signature | P0 | ✓ | IMPLEMENTED | Toolbar numerator/denominator; affects bar length, grid, metronome downbeat and auto pattern length.<br>*Project-wide only.* | `limeshot-core time::tests::signature_lengths` |
| TR-006 | Position display | P0 | ✓ | IMPLEMENTED | Bar:beat:tick and minutes:seconds; click returns to start. | `limeshot-core time::tests::bar_beat_tick_display` |
| TR-007 | Exact musical timing (ticks ↔ samples) | P0 | ✓ | VERIFIED | 960 PPQ integer ticks; exact u128 tick↔sample mapping; block-size independent onsets. | `limeshot-core time::tests::no_drift_after_an_hour`<br>`limeshot-core time::tests::first_tick_inverts_sample_of_tick`<br>`limeshot-engine tests/audio.rs::block_size_does_not_change_the_output` |
| TR-008 | Metronome | P1 |  | VERIFIED | Toolbar toggle / Cmd+M; synthesized click, higher on the downbeat. | `limeshot-engine tests/audio.rs::metronome_clicks_on_every_beat` |
| TR-009 | Song loop toggle | P1 |  | VERIFIED | Toolbar loop toggle. | `limeshot-engine tests/audio.rs::song_stops_at_the_end_without_loop` |
| TR-010 | Panic (all notes off) | P1 |  | IMPLEMENTED | Cmd+H kills every voice. | — |
| TR-011 | Audio load and voice count display | P1 |  | IMPLEMENTED | Toolbar load meter (callback time / buffer time) and status-bar voice count. | — |
| TR-012 | Tap tempo | P2 |  | NOT_STARTED | — | — |
| TR-013 | Master pitch | P3 |  | NOT_STARTED | — | — |

## Workflow

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| WF-001 | Headless command scripting (CLI) | P1 |  | IMPLEMENTED | `limeshot-cli run commands.json` applies typed commands, `render`, `info`, `demo`, `play`; no network service. | `CI: headless demo render + reload compare` |
| WF-002 | Typing keyboard to piano | P2 |  | NOT_STARTED | — | — |
| WF-003 | MIDI controller input | P1 |  | NOT_STARTED | midir-based input routed to the selected channel. | — |
| WF-004 | Controller mapping (link to controller) | P2 |  | NOT_STARTED | — | — |
| WF-005 | External MIDI devices / MIDI out | P2 |  | NOT_STARTED | — | — |
| WF-006 | Multichannel / surround output | P3 |  | NOT_STARTED | Stereo only; extra device channels receive silence. | — |
| WF-007 | Accessibility (screen readers, keyboard nav) | P2 |  | IN_PROGRESS | eframe AccessKit enabled; standard egui widgets are exposed; custom-painted editors are not yet labelled. | — |
| WF-008 | Python-style scripting API | P3 |  | NOT_STARTED | Possible future scripting over the typed command set. | — |

## UI

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| UI-001 | Main window layout (toolbar, browser, workspace) | P0 | ✓ | IMPLEMENTED | Menu bar, global toolbar, resizable browser side panel, workspace with child windows, status/hint bar. | `limeshot-app tests/ui.rs::renders_frames_without_panicking` |
| UI-002 | Child windows with toggles (F5–F9) and arrange | P0 | ✓ | VERIFIED | egui windows constrained to the workspace; View menu; Cmd+Shift+H arranges. | `limeshot-app tests/ui.rs::window_shortcuts_toggle_editors` |
| UI-003 | LimeShot theme tokens and drawn icons | P0 | ✓ | IMPLEMENTED | Single semantic token set (Lime Dark); every icon and the logo drawn in code; app icon rasterised in code. | — |
| UI-004 | Hint bar and tooltips | P1 |  | IMPLEMENTED | Status bar shows hover hints; every control has a tooltip. | — |
| UI-005 | UI scaling / HiDPI | P1 |  | IMPLEMENTED | egui pixels-per-point plus View › Zoom (Cmd+= / Cmd+- / Cmd+0), persisted. | — |
| UI-006 | Additional themes (light, high contrast) | P3 |  | NOT_STARTED | Token set is ready for more themes. | — |
| UI-007 | Detachable windows (multi-monitor) | P2 |  | NOT_STARTED | egui multi-viewport. | — |
| UI-008 | Offscreen UI rendering for review/regression | P1 |  | IMPLEMENTED | `cargo run -p limeshot-app --example ui_shot -- out.png` renders the window with wgpu offscreen. | — |

## Platform

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PF-001 | macOS Apple Silicon | P0 | ✓ | VERIFIED | Built, tested (all tests), GUI launched, realtime audio verified on macOS 26.6.2 arm64. | `cargo test --workspace (local)` |
| PF-002 | macOS Intel | P0 | ✓ | IMPLEMENTED | x86_64-apple-darwin release build runs under Rosetta; headless render bit-identical to arm64. GUI and audio device not exercised on Intel. | `local: arch -x86_64 limeshot-cli demo … (render compared with cmp)` |
| PF-003 | Windows x86-64 | P0 | ✓ | IN_PROGRESS | `cargo check --target x86_64-pc-windows-msvc` passes locally; CI job defined but not yet run. | — |
| PF-004 | Linux x86-64 | P0 | ✓ | IN_PROGRESS | CI job defined (ALSA, X11/Wayland, GTK deps); not yet run. Local cross-check stops at alsa-sys (needs target pkg-config). | — |
| PF-005 | Windows ARM64 / Linux ARM64 | P2 |  | IN_PROGRESS | `cargo check --target aarch64-pc-windows-msvc` passes locally; CI jobs defined. | — |
| PF-006 | Packaging (DMG, MSI, AppImage/deb) | P1 |  | NOT_STARTED | Signed bundles with the samples folder next to the binary. | — |
