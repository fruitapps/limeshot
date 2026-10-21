# LimeShot feature inventory

> Generated from [`features.json`](features.json) by `python3 scripts/features.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313). Updated 2026-10-21 (session 7).

**IMPLEMENTED** = the code exists and meets its implementation criteria. **VERIFIED** = it also passed the listed
automated tests or recorded manual verification. Everything else is not done.

## Summary

- Features tracked: **238**
- MVP features: **69** — 67 implemented or verified, 66 verified
- By status: NOT_STARTED 97, RESEARCHED 1, IN_PROGRESS 10, IMPLEMENTED 10, VERIFIED 118, DEFERRED 2

| Category | Total | Verified | Implemented | In progress | Not started / other |
|---|---|---|---|---|---|
| Application | 25 | 16 | 1 | 2 | 6 |
| Channel Rack | 20 | 14 | 0 | 0 | 6 |
| Piano Roll | 27 | 18 | 0 | 0 | 9 |
| Playlist | 28 | 18 | 1 | 2 | 7 |
| Mixer | 16 | 9 | 1 | 1 | 5 |
| Instruments | 14 | 6 | 0 | 0 | 8 |
| Effects | 15 | 8 | 0 | 0 | 7 |
| Automation | 13 | 3 | 0 | 0 | 10 |
| Recording | 9 | 0 | 0 | 0 | 9 |
| Browser | 10 | 5 | 1 | 0 | 4 |
| Plugins | 10 | 0 | 0 | 0 | 10 |
| Export | 13 | 5 | 0 | 0 | 8 |
| Transport & Time | 13 | 9 | 2 | 0 | 2 |
| Workflow | 8 | 0 | 1 | 1 | 6 |
| UI | 11 | 6 | 2 | 1 | 2 |
| Platform | 6 | 1 | 1 | 3 | 1 |

## Application

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| APP-001 | New project | P0 | ✓ | VERIFIED | File › New / Cmd+N: a 140 BPM project (FL Studio's default, observed) with one pattern, 16 playlist tracks, master + 16 inserts; asks to save unsaved changes. | `limeshot-core model::tests::new_project_has_one_pattern_and_master`<br>`crates/limeshot-app/tests/ui.rs::new_project_asks_to_save_unsaved_changes` |
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
| APP-012 | Audio device settings | P0 | ✓ | VERIFIED | Options › Audio settings: device, sample rate and buffer size from what the device reports; Apply restarts the stream.<br>*Verified manually that the default device opens (MacBook Pro Speakers, 48 kHz). Switching devices has not been tested on hardware with several outputs. No ASIO. Recorded manual verification: the default device opens and plays (sessions 4–7, MacBook Pro Speakers, 48 kHz).* | `limeshot-audio devices::tests::buffer_size_requests_are_validated` |
| APP-013 | Keyboard shortcuts for core actions | P0 | ✓ | VERIFIED | Space, Cmd+Space, L, F4–F9, Alt+F8, Cmd+Z/S/O/N/R/M/H/B/A/D, Delete, arrows; listed in Help › Keyboard shortcuts. | `limeshot-app tests/ui.rs::window_shortcuts_toggle_editors`<br>`limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo`<br>`limeshot-app tests/ui.rs::space_starts_and_stops_the_transport` |
| APP-014 | Ask to save on close | P0 | ✓ | VERIFIED | Closing the window with unsaved changes asks Save / Don't save / Cancel.<br>*The UI test covers New (Cancel and Don't save); closing the window uses the same dialog (Pending::Quit).* | `crates/limeshot-app/tests/ui.rs::new_project_asks_to_save_unsaved_changes` |
| APP-015 | Project templates | P2 |  | NOT_STARTED | Templates folder with .lime files offered on New. | — |
| APP-016 | Autosave and crash recovery | P1 |  | VERIFIED | While there are unsaved changes, the project is written atomically every N minutes (Options › Autosave: off, 1, 2, 5 (default) or 10) to autosave/autosave.lime in the data folder with a session record (original file, time). Saving, a clean New/Open and quitting remove it. At the next start a leftover copy (a crash) opens 'Recover unsaved work?': Recover restores it as unsaved changes to its original file; Discard deletes it. A clean start never deletes another session's copy.<br>*One autosave slot per user: two LimeShot instances share it (the later write wins).* | `limeshot-app tests/ui.rs::autosave_survives_a_crash_and_is_removed_once_saved` |
| APP-017 | Project info (title, author, comments) | P2 |  | IN_PROGRESS | Title is stored and editable via SetTitle (no UI dialog yet). | — |
| APP-018 | Save new version | P2 |  | NOT_STARTED | File › Save new version. (LimeShot uses Cmd+N for New.) | — |
| APP-019 | Project bundle (zip with samples) | P2 |  | NOT_STARTED | Export project + samples as a single archive. | — |
| APP-020 | MIDI settings | P1 |  | NOT_STARTED | Device list, enable inputs, controller type. | — |
| APP-021 | Plugin manager | P2 |  | NOT_STARTED | Scan folders for CLAP/VST3/AU. | — |
| APP-022 | Import FL Studio projects (.flp) | P3 |  | VERIFIED | File › Open (and `limeshot song.flp`, `limeshot-cli import-flp in.flp [out.lime]`) reads an .flp as a new unsaved project titled after the file; Save asks where to write the .lime. Converted: tempo, time signature, title, channels (samplers with their sample, `%FLStudioFactoryData%` resolved; plugin generators become LimeSynth), volume, pan, mixer insert, patterns with names and notes (key, position, length, velocity; steps), and the first arrangement's pattern and audio clips with track, offset and mute. A toast reports what was left out (plugin channels, automation clips, unknown items). Format notes: docs/research/flp-format.md.<br>*Clean-room: from the bytes of project files (FL's shipped templates/demos read-only and files saved by FL 26.1.1 for the tests); no decompilation. Not converted: plugin states, mixer effects and routing, automation curves, layers, further arrangements. Only FL 26.1.1 was used to make test files; older versions are covered by the shipped demos importing without errors.* | `limeshot-project tests/flp.rs::fl26_saved_steps_project`<br>`limeshot-project tests/flp.rs::fl26_saved_notes_project`<br>`limeshot-project tests/flp.rs::fl26_saved_arrangement_project`<br>`limeshot-project tests/flp.rs::installed_fl_projects_all_import`<br>`limeshot-project tests/flp.rs::fixture_files_match_their_definitions`<br>`limeshot-project src/flp.rs::tests::hostile_bytes_never_panic`<br>`limeshot-app tests/ui.rs::opening_an_flp_imports_it_as_an_unsaved_project` |
| APP-023 | Welcome window | P3 |  | NOT_STARTED | Planned (offline parts only: templates, recent files, demo song). | — |
| APP-024 | Autosave interval setting | P2 |  | VERIFIED | Options › Autosave: off, 1, 2, 5 (default) or 10 minutes; stored in the settings. | `limeshot-app tests/ui.rs::autosave_survives_a_crash_and_is_removed_once_saved` |
| APP-025 | Settings window | P2 |  | IN_PROGRESS | Audio settings have a dialog; UI scale, autosave and browser folders are set from menus and the Browser. A single window with pages is planned; settings.json tracks every setting. | — |

## Channel Rack

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| SEQ-001 | Step sequencing | P0 | ✓ | VERIFIED | Step buttons per channel; either mouse button toggles a step and a drag paints the state of the first step (FL Studio); steps are notes at 240-tick steps with the channel's root key.<br>*Toggle/paint semantics observed live in FL Studio (docs/research/evidence/screenshots/fl-steps-toggle.png) and checked with real input in the live LimeShot window (2026-10-20, session 4).* | `limeshot-engine tests/audio.rs::steps_sound_exactly_on_the_grid`<br>`limeshot-engine tests/audio.rs::tempo_change_moves_onsets`<br>`limeshot-app tests/ui.rs::clicking_a_step_button_writes_a_note`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end`<br>`limeshot-app tests/ui.rs::dragging_across_steps_paints_them_in_one_undo_step`<br>`limeshot-app tests/ui.rs::a_drag_starting_on_a_lit_step_erases` |
| SEQ-002 | Add sampler / synth channels | P0 | ✓ | VERIFIED | Add menu, Channel Rack + Add, or drop a WAV from the Browser. | `limeshot-core document::tests::new_channels_take_free_inserts`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| SEQ-003 | Channel mute and solo | P0 | ✓ | VERIFIED | LED: left-click mutes, right-click or Cmd+click solos. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves`<br>`limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| SEQ-004 | Channel volume and pan | P0 | ✓ | VERIFIED | Knobs (drag, Shift fine, wheel, double-click reset); default 78% volume. | `limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| SEQ-005 | Channel → mixer track routing | P0 | ✓ | VERIFIED | Mixer field per row (M = master), drag or type; new channels take the next free insert; Cmd+L routes to a free insert.<br>*Deviation: FL defaults new channels to Master.* | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master`<br>`limeshot-core document::tests::new_channels_take_free_inserts` |
| SEQ-006 | Pattern length (auto / manual) | P0 | ✓ | VERIFIED | Auto (whole bars covering the last note) or a fixed number of steps. | `limeshot-core model::tests::auto_pattern_length_covers_last_note` |
| SEQ-007 | Rename, recolor, clone, delete, reorder channels | P1 |  | VERIFIED | Right-click on a channel button selects only that channel and opens its menu: rename, color, clone, Delete… (asks first), move up/down. Alt+Up/Down moves every selected channel one row (the block stops at the edge); Alt+Delete deletes the selected channels after FL's confirmation ("Remember my choice" turns it off). Each is one undo step.<br>*Observed live in FL Studio 2026.1.1 (session 7): Alt+Up moves each selected channel up one row and the mixer track stays with the channel; the channel menu's Delete… and Alt+Del (forward delete; Alt+Backspace does nothing) ask "The selected channel(s) is/are about to be deleted." with "Remember my choice". FL's menu also has Fill each 2/4/8 steps and Rotate left/right (SEQ-014), Graph editor (SEQ-011), Random color and icons (not tracked).* | `limeshot-core document::tests::removing_channel_removes_its_notes`<br>`crates/limeshot-app/tests/ui.rs::selector_leds_select_toggle_and_move_channels`<br>`crates/limeshot-app/tests/ui.rs::deleting_channels_asks_first_and_is_one_undo_step`<br>`crates/limeshot-app/tests/ui.rs::right_click_on_a_channel_button_selects_it_and_opens_its_menu` |
| SEQ-008 | Piano-roll preview in rack | P1 |  | VERIFIED | Mini note preview; click opens the Piano Roll for that channel. | `crates/limeshot-app/tests/ui.rs::clicking_a_note_preview_in_the_rack_opens_the_piano_roll` |
| SEQ-009 | Channel settings window | P0 | ✓ | VERIFIED | Channel Settings window (opened by clicking the channel button): sampler (sample, waveform, gain, pitch, start/end, release, reverse, one-shot) or LimeSynth (waveform, octave, detune, gain, ADSR with a curve preview), plus root key. | `limeshot-app tests/ui.rs::renders_every_panel_without_panicking` |
| SEQ-010 | Swing (global and per channel) | P1 |  | VERIFIED | Global swing knob in the Channel Rack header (0–100%): notes exactly on the second 16th of each pair are delayed by up to half a step (≈67% gives a triplet feel), in Pattern and Song playback.<br>*Per-channel swing mix and 'truncate swing notes' are not implemented.* | `limeshot-sequencer tests::swing_delays_every_second_sixteenth`<br>`limeshot-core document::tests::swing_is_clamped_and_undoable` |
| SEQ-011 | Graph editor (per-step properties) | P2 |  | NOT_STARTED | Edit step velocity/pan/release/pitch in the rack. | — |
| SEQ-012 | Channel filter groups | P2 |  | NOT_STARTED | Named channel groups with a filter selector. | — |
| SEQ-013 | Zip / unzip channels | P3 |  | NOT_STARTED | Collapse rows. | — |
| SEQ-014 | Fill / rotate / advanced fill | P2 |  | NOT_STARTED | Channel menu fill and rotate commands. | — |
| SEQ-015 | Copy / paste steps between channels and patterns | P1 |  | VERIFIED | With the Channel Rack focused, Cmd+C/X copy/cut the selected channel's notes in the current pattern; Cmd+V replaces the selected channel's notes. Step patterns are re-pitched to the target channel's root key so they stay steps. | `limeshot-app tests/ui.rs::channel_rack_copies_steps_between_channels` |
| SEQ-016 | Channel step looping / ghost steps | P3 |  | NOT_STARTED | Per-channel loop length. | — |
| SEQ-017 | Loop Starter (FL Cloud) | P3 |  | DEFERRED | Out of scope: requires an online service (LimeShot is offline-first). | — |
| SEQ-018 | Multiple patterns | P0 | ✓ | VERIFIED | Toolbar pattern selector (prev/next/new/rename), F4 new, +/− switch; Patterns menu clone/delete. | `limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement`<br>`limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo` |
| SEQ-019 | Audio clip channels | P1 |  | VERIFIED | A sample dropped on the Playlist adds a sampler channel (one-shot off) that the audio clip plays; dropping the same sample again reuses it.<br>*LimeShot uses its sampler for these channels instead of a separate Audio Clip generator.* | `limeshot-core document::tests::audio_clips_reuse_their_channel_and_follow_it` |
| SEQ-020 | Channel selection (selector LEDs) | P1 |  | VERIFIED | The same selector LEDs and gestures (Cmd on macOS for Ctrl). The selection is UI state; the current channel (Piano Roll, Channel Settings) is the last one selected. Alt+Up/Down, Alt+Delete and Cmd+L act on all selected channels.<br>*Observed live (session 7). Neither Ctrl+A nor Cmd+A selects all channels in FL's Channel Rack. FL also saves the selection in the project; LimeShot does not. Channel filter groups (SEQ-012) are not implemented.* | `crates/limeshot-app/tests/ui.rs::selector_leds_select_toggle_and_move_channels`<br>`crates/limeshot-app/tests/ui.rs::cmd_l_routes_each_selected_channel_to_its_own_free_track` |

## Piano Roll

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PR-001 | Add, move, resize, delete notes | P0 | ✓ | VERIFIED | Same mouse conventions; Alt bypasses snap; Delete removes the selection. New notes are two steps long until a note is drawn or resized; after that they take its length (as observed in FL Studio).<br>*Default note length observed live in FL Studio 2026.1.1 (evidence/screenshots/fl-note-length.png).* | `limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes`<br>`limeshot-core document::tests::notes_move_resize_and_duplicate` |
| PR-002 | Note selection (click, rectangle, select all) | P0 | ✓ | VERIFIED | Click selects; Cmd+click toggles; Cmd+drag rectangle; Cmd+A all, Cmd+D none, Shift+I invert; double-click a note selects its pitch row.<br>*Observed in FL 2026.1.1's Select menu (fl-piano-select-menu.png): Cmd+A, Cmd+D, Shift+I invert. Not implemented: select by colour/odd/random/muted/overlapping, select time around selection, magic lasso.* | `crates/limeshot-app/tests/ui.rs::piano_roll_selection_gestures` |
| PR-003 | Snap to grid | P0 | ✓ | VERIFIED | Global snap (bar, beat, ½ beat, step, ½, ¼ step, none); Alt bypasses. | `limeshot-core time::tests::snapping`<br>`limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes` |
| PR-004 | Velocity editing | P0 | ✓ | VERIFIED | Velocity lane: click/drag a stem; applies to the selection when the note is selected.<br>*The UI test sets one note's velocity by clicking its stem and a selection's by dragging, as one undo step.* | `limeshot-engine tests/audio.rs::piano_roll_notes_play_and_release`<br>`crates/limeshot-app/tests/ui.rs::piano_roll_velocity_lane_sets_velocity` |
| PR-005 | Zoom and scroll | P0 | ✓ | VERIFIED | Wheel scrolls, Shift+wheel horizontal, Cmd+wheel or pinch zooms (anchored at the pointer), arrow keys scroll, +/− buttons, taller keys.<br>*Session 7 fixed Cmd+wheel zoom, which never worked: egui reports Cmd+wheel (and pinch) as a zoom delta, not as scrolling. Plain arrow keys scroll the view, as observed in FL 2026.1.1.* | `crates/limeshot-app/tests/ui.rs::piano_roll_wheel_and_arrows_zoom_and_scroll` |
| PR-006 | Piano-key auditioning | P0 | ✓ | VERIFIED | Click/drag on the keyboard plays the channel; new and dragged notes are auditioned. | `limeshot-engine tests/audio.rs::preview_plays_without_transport`<br>`crates/limeshot-app/tests/ui.rs::piano_roll_keyboard_ruler_ghosts_and_channel_selector` |
| PR-007 | Playback position display | P0 | ✓ | VERIFIED | Playhead line in Pattern mode; ruler click locates.<br>*The playhead line during Pattern playback was checked in the live app (session 7 real-input pass); the ruler click is tested.* | `crates/limeshot-app/tests/ui.rs::piano_roll_keyboard_ruler_ghosts_and_channel_selector` |
| PR-008 | Duplicate selection (Ctrl+B) | P1 |  | VERIFIED | Cmd+B copies the selection right by its span rounded up to beats; copies become selected. | `limeshot-core document::tests::notes_move_resize_and_duplicate` |
| PR-009 | Transpose and nudge with keys | P1 |  | VERIFIED | Shift+↑/↓ semitone, Cmd+↑/↓ octave, Shift+←/→ by the snap; the selection, or every note of the channel when nothing is selected.<br>*Observed in FL 2026.1.1 (fl-piano-edit-menu.png): Shift+Up/Down transpose, Cmd+Up/Down an octave, Shift+Left/Right shift; with nothing selected they apply to all of the channel's notes. Plain arrows scroll. Rotate (Shift+Cmd+Left/Right) and Discard lengths (Shift+D) are not implemented.* | `crates/limeshot-app/tests/ui.rs::piano_roll_shift_arrows_transpose_and_nudge` |
| PR-010 | Ghost notes | P1 |  | VERIFIED | Notes of other channels in the same pattern are drawn as ghosts (not editable).<br>*Ghosts from overlapping playlist clips not implemented. The UI test checks ghosts cannot be deleted and that clicking on one adds a note to the edited channel.* | `crates/limeshot-app/tests/ui.rs::piano_roll_keyboard_ruler_ghosts_and_channel_selector` |
| PR-011 | Channel target selector | P0 | ✓ | VERIFIED | Channel combo box in the header. | `crates/limeshot-app/tests/ui.rs::piano_roll_keyboard_ruler_ghosts_and_channel_selector` |
| PR-012 | Tool palette: draw, paint, delete, mute, slice, select, zoom, playback | P1 |  | VERIFIED | Tool buttons in the Piano Roll header and FL's keys. Paint lays notes of the last length end to end along a drag (selected, one undo step); Delete erases; Mute toggles notes' mute (muted notes are dimmed and silent); Slice cuts crossed notes at the snapped x of the drag and selects the right pieces; Select drags a rectangle without Cmd; Zoom fits a dragged range (click zooms in, right-click out); Playback plays from the pointer while held.<br>*Observed live in FL 2026.1.1 (session 7): Paint lays end-to-end notes of the last length, all selected (fl-piano-paint.png); Slice cuts at the line and selects the right piece (fl-piano-slice.png). Playback is press-to-play, not scrubbing. Not observed live: the Mute, Delete, Zoom and Playback tools' details (taken from the manual). The Stamp tool is PR-027.* | `crates/limeshot-app/tests/ui.rs::piano_roll_tool_keys_and_palette`<br>`crates/limeshot-app/tests/ui.rs::paint_tool_lays_notes_end_to_end`<br>`crates/limeshot-app/tests/ui.rs::slice_glue_chop_and_mute_tools`<br>`crates/limeshot-app/tests/ui.rs::delete_select_zoom_and_draw_left_edge`<br>`limeshot-core document::tests::slice_glue_and_mute_notes`<br>`limeshot-sequencer tests::muted_notes_do_not_play` |
| PR-013 | Scale highlighting / snap to scale | P2 |  | NOT_STARTED | Scale selector, shading rows out of key, optional snapping. | — |
| PR-014 | Quantize | P1 |  | VERIFIED | Alt+Q or the Quantize button snaps note starts of the selection (or the whole channel) to the snap grid. The command supports a strength and length rounding; the UI uses 100% starts only.<br>*No quantize dialog (strength, lengths, swing template) yet.* | `limeshot-core document::tests::quantize_moves_starts_by_strength_and_rounds_lengths`<br>`limeshot-app tests/ui.rs::copy_paste_and_quantize_in_the_piano_roll` |
| PR-015 | Chords, arpeggiator, strum, humanize, legato | P2 |  | NOT_STARTED | Tools menu with these transforms as commands. | — |
| PR-016 | Note properties: pan, release, mod X/Y, fine pitch | P2 |  | NOT_STARTED | Additional lane targets stored per note. | — |
| PR-017 | Slide and portamento notes | P2 |  | NOT_STARTED | Slide flag on notes interpreted by LimeSynth. | — |
| PR-018 | Note color groups | P3 |  | NOT_STARTED | Per-note color group. | — |
| PR-019 | Copy / cut / paste notes | P1 |  | VERIFIED | Cmd+C/X/V copy, cut and paste notes; pasting puts them at the same times into the channel being edited and selects them (FL Studio behaviour).<br>*App-internal clipboard (not the system clipboard).* | `limeshot-app tests/ui.rs::copy_paste_and_quantize_in_the_piano_roll` |
| PR-020 | MIDI file import / export | P1 |  | VERIFIED | Own Standard MIDI File reader/writer (format 0/1, running status, note-on velocity 0, tempo, time signature, track names; SMPTE and format 2 refused; hostile input bounded). File › Import MIDI file… adds one pattern named after the file with a LimeSynth channel per MIDI track (and per MIDI channel within a track), in one undo step; an empty project also takes the file's tempo. CLI: limeshot-cli import-midi in.mid out.lime.<br>*Exported files were read back by LimeShot's own reader only; not yet checked in another program. Controller events, pitch bend and program changes are ignored on import.* | `limeshot-project midi::tests::write_then_read_round_trips`<br>`limeshot-project midi::tests::reads_running_status_other_resolutions_and_note_on_zero`<br>`limeshot-project midi::tests::hostile_input_is_refused_not_panicked_on`<br>`limeshot-project midi::tests::import_command_builds_one_channel_per_track`<br>`limeshot-app tests/ui.rs::midi_export_and_import_round_trip_through_the_app` |
| PR-021 | MIDI recording into the Piano Roll | P1 |  | NOT_STARTED | Record from MIDI devices/typing keyboard. | — |
| PR-022 | Time signature per pattern / time markers | P3 |  | NOT_STARTED | Project-wide signature only for now. | — |
| PR-023 | Glue / slice notes | P2 |  | VERIFIED | Glue (Cmd+G) joins selected notes of the same key that touch or overlap; Quick chop (Cmd+U) cuts notes at every snap step; the Slice tool cuts at a point. With nothing selected they apply to all of the channel's notes.<br>*Shortcuts from FL 2026.1.1's Piano Roll Tools menu (fl-piano-tools-menu.png). The Chop… dialog is not implemented.* | `crates/limeshot-app/tests/ui.rs::slice_glue_chop_and_mute_tools`<br>`limeshot-core document::tests::slice_glue_and_mute_notes` |
| PR-024 | Resize from the left edge | P2 |  | VERIFIED | Draw/Paint tools: dragging a note's left edge (outer 6 px) moves its start and keeps its end (snapped; one undo step). Always on.<br>*The option to turn it off is not implemented.* | `crates/limeshot-app/tests/ui.rs::delete_select_zoom_and_draw_left_edge` |
| PR-025 | Waveform helper background | P3 |  | NOT_STARTED | Show a sample waveform behind the grid. | — |
| PR-026 | New notes take the last note's length | P1 |  | VERIFIED | Identical: first note two steps, then the length of the last drawn or resized note. | `limeshot-app tests/ui.rs::piano_roll_click_adds_drag_moves_delete_removes` |
| PR-027 | Stamp tool (chords and scales) | P2 |  | NOT_STARTED | A Stamp selector: drawing a note adds the chosen chord or scale.<br>*Split out of PR-012 in session 7.* | — |

## Playlist

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PL-001 | Pattern clips (linked instances) | P0 | ✓ | VERIFIED | Left-click empty space places the current pattern; every clip of a pattern plays the same data. | `limeshot-core document::tests::clips_are_linked_until_made_unique`<br>`limeshot-sequencer tests::playlist_flattens_linked_clips`<br>`limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern`<br>`limeshot-app tests/ui.rs::picker_selects_the_pattern_to_place` |
| PL-002 | Move clips (time and track) | P0 | ✓ | VERIFIED | Drag moves the selection with snap; arrows nudge. | `limeshot-core document::tests::clip_duplicate_and_move` |
| PL-003 | Resize clips | P0 | ✓ | VERIFIED | Drag the right edge to change the length; drag the left edge to trim or reveal the start (the clip offset moves, the end stays). A pattern clip plays its pattern once and is silent after it (FL Studio, D14).<br>*Matches FL Studio: observed live in FL Studio 2026.1.1 that a pattern clip lengthened past its pattern is silent after the pattern ends (no looping; evidence/screenshots/fl-clip-no-loop.png). Decision D14.* | `limeshot-sequencer tests::clip_truncates_notes`<br>`limeshot-core document::tests::trimming_a_clip_start_moves_its_offset`<br>`crates/limeshot-app/tests/ui.rs::playlist_clips_select_resize_and_move_with_shift_arrows`<br>`crates/limeshot-app/tests/ui.rs::samples_become_audio_clips_that_trim_from_the_left_edge` |
| PL-004 | Delete clips | P0 | ✓ | VERIFIED | Right-click or right-drag deletes; Delete key removes the selection. | `limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern` |
| PL-005 | Duplicate clips (Ctrl+B) | P0 | ✓ | VERIFIED | Cmd+B or clip menu › Duplicate. | `limeshot-core document::tests::clip_duplicate_and_move` |
| PL-006 | Select clips | P0 | ✓ | VERIFIED | Click, Cmd+click toggle, Cmd+drag rectangle, Cmd+A / Cmd+D, Shift+I invert.<br>*FL 2026.1.1's Playlist Select menu (fl-playlist-select-menu.png): Cmd+A, Cmd+D, Shift+I invert; LimeShot has those. Select by source/muted/overlapping/stacked and time around selection are not implemented.* | `crates/limeshot-app/tests/ui.rs::playlist_clips_select_resize_and_move_with_shift_arrows` |
| PL-007 | Snap clips to grid | P0 | ✓ | VERIFIED | Uses the global snap; Alt bypasses. | `limeshot-app tests/ui.rs::clicking_the_playlist_places_the_current_pattern` |
| PL-008 | Zoom, scroll, bar/beat ruler | P0 | ✓ | VERIFIED | Cmd+wheel or pinch zoom, wheel/Shift+wheel and arrow keys scroll, ruler with bar numbers.<br>*Session 7 fixed Cmd+wheel zoom (it never worked: egui reports it as a zoom delta). Plain arrows scroll, inferred from the Piano Roll (not observed in FL's Playlist).* | `crates/limeshot-app/tests/ui.rs::playlist_wheel_zoom_and_ruler_set_the_song_position`<br>`crates/limeshot-app/tests/ui.rs::playlist_clips_select_resize_and_move_with_shift_arrows` |
| PL-009 | Playhead and set song position | P0 | ✓ | VERIFIED | Ruler click/drag sets the song position and switches to Song mode; Home returns to start. | `crates/limeshot-app/tests/ui.rs::playlist_wheel_zoom_and_ruler_set_the_song_position` |
| PL-010 | Song mode drives the engine | P0 | ✓ | VERIFIED | Song program = flattened clips; loops at the end (toggle) or stops. | `limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement`<br>`limeshot-engine tests/audio.rs::song_stops_at_the_end_without_loop`<br>`limeshot-app tests/mvp_workflow.rs::mvp_workflow_end_to_end` |
| PL-011 | Make unique | P0 | ✓ | VERIFIED | Clip ▾ menu › Make unique clones the pattern for that clip. | `limeshot-core document::tests::clips_are_linked_until_made_unique` |
| PL-012 | Playlist tracks: add, rename, mute | P0 | ✓ | VERIFIED | + Track; double-click to rename; LED mutes the track. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves` |
| PL-013 | Clip mute | P1 |  | VERIFIED | Clip menu › Mute clip. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves` |
| PL-014 | Audio clips | P1 |  | VERIFIED | Audio clips reference an audio-clip channel (a sampler channel, as in FL Studio), so its volume, pan, sampler settings and mixer track apply. The clip plays its sample from its offset for its length (sample-accurate, ends with the sampler's declick release); starting playback inside a clip starts the audio part-way. Waveform overview in the clip; double-click opens the channel settings; ▾ menu: channel settings, mute, fit to sample length, duplicate, delete.<br>*No time-stretching: the clip offset is in ticks and plays at the sample's own speed (changing the tempo moves the audio relative to the grid, as with FL's non-stretched clips). WAV only until INS-008.* | `limeshot-engine tests/audio.rs::audio_clips_play_their_sample_window_sample_accurately`<br>`limeshot-engine tests/audio.rs::playback_starting_inside_an_audio_clip_starts_the_audio_part_way`<br>`limeshot-core document::tests::audio_clips_reuse_their_channel_and_follow_it`<br>`limeshot-core document::tests::sanitize_drops_audio_clips_without_a_channel`<br>`limeshot-app tests/ui.rs::samples_become_audio_clips_that_trim_from_the_left_edge` |
| PL-015 | Automation clips | P1 |  | VERIFIED | Automation clips on the Playlist with a filled curve, points and tension handles; move, resize either edge (trims the curve window), copy/paste and duplicate like other clips. | `limeshot-app tests/ui.rs::automation_clips_are_created_from_controls_and_edited_in_the_playlist` |
| PL-016 | Markers and loop region / time selection | P1 |  | VERIFIED | Cmd+drag on the ruler: time selection (snapped; Alt: free), shaded on the ruler and the arrangement; Song playback starts at it and loops it, even with song loop off; Cmd+click or Cmd+D clears it. Markers: Cmd+T adds one at the song position, Alt+T adds one and asks its name; flags in the ruler's lower half — click jumps, drag moves (one undo step), right-click renames or deletes; markers are saved with the project.<br>*Observed live in FL 2026.1.1 (session 7, fl-playlist-time-selection.png, fl-playlist-time-markers-menu.png). FL names an auto marker "Auto"; LimeShot numbers them. Not implemented: loop/key/time-signature/jump markers, Place loop (Shift+T), selecting time in the Piano Roll, saving the time selection with the project.* | `crates/limeshot-app/tests/ui.rs::playlist_time_selection_loops_song_playback`<br>`crates/limeshot-app/tests/ui.rs::playlist_markers_add_jump_move_and_delete`<br>`limeshot-engine tests/audio.rs::song_loops_the_time_selection_at_any_block_size`<br>`limeshot-core document::tests::markers_are_added_sorted_renamed_moved_and_removed` |
| PL-017 | Slip, slice clips | P2 |  | VERIFIED | FL Studio's Playlist tool strip (Draw P, Paint B, Delete D, Mute T, Slip S, Slice C, Select E, Zoom Z, Playback Y). Slip drags a clip's content inside it (offset snapped, never before the source start, may run past its end into silence); Slice cuts the clips a vertical drag crosses at the snapped line, the right pieces continuing the source and selected. Each gesture is one undo step.<br>*Observed live in FL 2026.1.1 (session 8). Not done: Shift+S ripple edit, diagonal (Shift) slicing, slicing without dragging (Shift+click), zero-cross snapping for audio slices, Right-Shift shortcuts.* | `limeshot-core document::tests::slip_and_slice_clips`<br>`ui.rs playlist_tool_keys_and_strip`<br>`ui.rs playlist_slip_tool_moves_content_inside_the_clip`<br>`ui.rs playlist_slice_tool_cuts_the_clips_it_crosses`<br>`ui.rs playlist_paint_mute_delete_select_zoom_and_playback_tools` |
| PL-018 | Track grouping | P2 |  | NOT_STARTED | Collapsible track groups. | — |
| PL-019 | Performance mode | P3 |  | NOT_STARTED | Clip launch grid. | — |
| PL-020 | Arrangements | P2 |  | NOT_STARTED | Several playlists sharing channels/mixer. | — |
| PL-021 | Picker panel | P2 |  | NOT_STARTED | Panel listing patterns to drag in. | — |
| PL-022 | Consolidate / render to audio clip | P2 |  | NOT_STARTED | Render a selection to an audio clip. | — |
| PL-023 | Crossfades and time stretching | P2 |  | NOT_STARTED | Fades and stretch on audio clips. | — |
| PL-024 | Clip offset (start inside pattern) | P2 |  | IN_PROGRESS | Model and sequencer support `offset`; no UI to set it yet. | — |
| PL-025 | Pattern picker | P1 |  | VERIFIED | Patterns column inside the Playlist window, left of the track headers: colour, name, usage count; click to choose what the Playlist places, double-click to edit in the Piano Roll, right-click to rename/clone/delete, + for a new pattern. | `limeshot-app tests/ui.rs::picker_selects_the_pattern_to_place` |
| PL-026 | Copy / cut / paste clips | P1 |  | IMPLEMENTED | Cmd+C/X/V for every clip kind (source, offset and length are kept); in Song mode pasted clips start at the song position (snapped to the bar), otherwise at their original times; pasted clips are selected.<br>*No automated test yet.* | — |
| PL-027 | Picker lists audio and automation clips | P1 |  | IN_PROGRESS | Patterns and automation clips are listed and can be picked to place; audio clips are placed by dropping samples. | — |
| PL-028 | Automation STEP / SLIDE display options | P3 |  | RESEARCHED | Not started; research first.<br>*UNKNOWN: what STEP and SLIDE change.* | — |

## Mixer

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| MIX-001 | Master and insert tracks | P0 | ✓ | VERIFIED | Master + 16 inserts (up to 128); inserts sum into the master. | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-002 | Volume faders | P0 | ✓ | VERIFIED | Fader 0…+6 dB with a perceptual travel; Shift fine; double-click resets. | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-003 | Pan | P0 | ✓ | VERIFIED | Pan knob; balance law (centre unity, never boosts). | `limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master`<br>`limeshot-dsp tests::pan_law` |
| MIX-004 | Mute and solo | P0 | ✓ | VERIFIED | M and S buttons; right-click/Cmd+click on M solos; master never solo-muted. | `limeshot-sequencer tests::muted_tracks_and_clips_are_skipped_and_solo_resolves`<br>`limeshot-engine tests/audio.rs::mixer_volume_pan_and_mute_route_to_master` |
| MIX-005 | Level meters and clip indicator | P0 | ✓ | VERIFIED | Stereo peak meters (−60…+6 dB, 20 dB/s fall-back) per strip and in the toolbar; master clip LED.<br>*The UI test plays the demo in Song mode and checks the master peak reaches the Mixer's meter hold; the drawing itself was checked in screenshots.* | `crates/limeshot-app/tests/ui.rs::song_playback_drives_the_master_meter` |
| MIX-006 | Rename and add mixer tracks | P1 |  | IMPLEMENTED | Double-click name to rename; + adds a track. | — |
| MIX-007 | Insert effect slots | P1 |  | VERIFIED | Up to 10 insert effects per mixer track (master included), processed in order before the fader. The Mixer window's right-hand panel (FL layout) shows the selected track's slots: add, bypass, reorder, remove, edit; slot names also show on the strips.<br>*Processors are built on the control thread and handed to the audio thread by id (see audio-engine.md). No pre-fader/post-fader choice, no plugin effects.* | `limeshot-core effects::tests`<br>`limeshot-core document::tests::effect_chain_commands`<br>`limeshot-engine tests/audio.rs::insert_effects_process_the_track_and_extend_the_tail`<br>`limeshot-engine tests/audio.rs::effects_without_processors_are_bypassed` |
| MIX-008 | Routing and sends between tracks | P1 |  | VERIFIED | Each insert routes (post-fader) to the master by default and can route to any other insert at a send level, to several destinations at once (parallel buses). In the Mixer, the selected track shows a route button on every other strip (lit when routed, with a send knob; dimmed when it would loop); the track panel lists outputs with levels and '+ Route to…'. Routes that would create feedback are refused; a track without routes is silent.<br>*Sends are post-fader only (no pre-fader option yet); routes are summed with no latency compensation (not needed until effects report latency).* | `limeshot-core document::tests::routing_refuses_feedback_and_orders_tracks`<br>`limeshot-core document::tests::sanitize_breaks_routing_loops_from_files`<br>`limeshot-engine tests/audio.rs::mixer_routes_through_buses`<br>`limeshot-app tests/ui.rs::mixer_route_buttons_route_the_selected_track` |
| MIX-009 | Sidechain routing | P2 |  | NOT_STARTED | Sidechain inputs for dynamics. | — |
| MIX-010 | Built-in 3-band EQ | P1 |  | VERIFIED | Every mixer track (master included) has FL Studio's built-in EQ after its insert effects and before the fader: a low shelf, a peak and a high shelf with FL's defaults (90 Hz, 1500 Hz, 8000 Hz, 0 dB, width 0.27), ±18 dB. The Mixer's track panel shows the response curve with a handle per band (drag: frequency and gain; right-drag: width), knobs for gain, frequency and width, and Reset. Saved in the .lime file only when changed.<br>*Observed live in FL 2026.1.1 (session 8): band names, defaults and the ±18 dB range from the hint bar. LimeShot's mapping from width to Q is its own. Not automatable yet; the Lime EQ insert effect (FX-002) remains.* | `limeshot-dsp effects::tests::track_eq_bands_shape_the_response`<br>`limeshot-core model::tests::track_eq_is_saved_only_when_changed`<br>`limeshot-engine audio.rs mixer_track_eq_shapes_the_track_and_the_master`<br>`ui.rs mixer_eq_graph_and_reset_edit_the_track_eq` |
| MIX-011 | Stereo separation, phase invert, swap L/R | P2 |  | IN_PROGRESS | Width and polarity are available through the Utility insert (FX-013); no per-strip controls or L/R swap. | — |
| MIX-012 | Plugin delay compensation | P2 |  | NOT_STARTED | Latency reported by effects aligned across paths. | — |
| MIX-013 | Insert-to-insert routing | P1 |  | VERIFIED | Covered by MIX-008: an insert can route to another insert instead of (or as well as) the master; inserts are processed in dependency order. | `limeshot-engine tests/audio.rs::mixer_routes_through_buses` |
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
| INS-006 | Synth filter and modulation (LFO, filter env) | P1 |  | VERIFIED | LimeSynth has a resonant 12 dB/oct low-pass per voice (TPT state-variable filter), a filter envelope (depth in octaves, exponential decay) and a sine LFO to pitch (vibrato, cents) and to cutoff (octaves); modulation updates every 16 samples. An open filter without modulation is bypassed, so existing sounds are unchanged. Channel Settings › Filter / LFO.<br>*No high-pass/band-pass modes or key tracking yet; the LFO is free-running per voice (starts at phase 0 on each note).* | `limeshot-dsp filter::tests::passes_below_and_cuts_above_the_cutoff`<br>`limeshot-engine tests/audio.rs::synth_filter_darkens_and_its_envelope_opens_it_at_the_note_start`<br>`limeshot-engine tests/audio.rs::synth_defaults_bypass_the_filter_and_vibrato_moves_the_pitch` |
| INS-007 | Sampler loop points, time stretch, filter | P1 |  | NOT_STARTED | Loop region, ping-pong, filter. | — |
| INS-008 | Additional audio formats (MP3, FLAC, OGG, AIFF) | P1 |  | NOT_STARTED | Decoder crate (e.g. symphonia) behind the SampleData interface. | — |
| INS-009 | Multi-sample / drum kit instrument | P2 |  | NOT_STARTED | Key zones mapping samples. | — |
| INS-010 | Instrument presets | P2 |  | NOT_STARTED | Preset files for sampler/synth parameters. | — |
| INS-011 | Per-channel polyphony limit / mono / porta | P2 |  | NOT_STARTED | Max voices, mono mode, glide. | — |
| INS-012 | Drum synthesizer | P3 |  | NOT_STARTED | — | — |
| INS-013 | LimeSynth filter modes and key tracking | P2 |  | NOT_STARTED | Planned (after INS-006). | — |
| INS-014 | Sampler volume envelope | P2 |  | NOT_STARTED | Planned: the sampler has a release only. | — |

## Effects

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| FX-001 | Effect processing framework (insert chain) | P1 |  | VERIFIED | Effect trait (set_params/process/reset), FxRack chains keyed by effect id, real-time-safe adoption and retirement, tails in exports. | `limeshot-engine tests/audio.rs::insert_effects_process_the_track_and_extend_the_tail`<br>`limeshot-engine tests/audio.rs::effects_without_processors_are_bypassed`<br>`limeshot-dsp effects::tests::nonsense_params_stay_finite` |
| FX-002 | Parametric equalizer | P1 |  | VERIFIED | Lime EQ: low shelf, peaking mid (Q), high shelf (RBJ biquads).<br>*Three bands only; no graphical EQ curve yet.* | `limeshot-dsp effects::tests::eq_boosts_and_cuts_its_band` |
| FX-003 | Filters (LP/HP/BP) | P1 |  | VERIFIED | Lime Filter: TPT state-variable filter, low/band/high-pass, resonance, mix. | `limeshot-dsp effects::tests::filter_modes` |
| FX-004 | Compressor | P1 |  | VERIFIED | Lime Comp: feed-forward, stereo-linked peak detector, hard knee; threshold, ratio, attack, release, makeup.<br>*No sidechain input, no knee or lookahead, no gain-reduction meter in the UI.* | `limeshot-dsp effects::tests::compressor_reduces_loud_signals_by_the_ratio` |
| FX-005 | Limiter / maximiser | P1 |  | VERIFIED | Lime Limiter: instant-attack peak limiter; ceiling and release. The output never exceeds the ceiling.<br>*No lookahead, so fast peaks are shaped hard.* | `limeshot-dsp effects::tests::limiter_never_exceeds_the_ceiling` |
| FX-006 | Noise gate | P1 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-007 | Reverb | P1 |  | VERIFIED | Lime Room: pre-delay, 8 damped combs and 4 allpasses per side; size, damping, pre-delay, width, mix. | `limeshot-dsp effects::tests::reverb_has_a_decaying_tail` |
| FX-008 | Delay | P1 |  | VERIFIED | Lime Delay: stereo, up to 2 s, feedback with a tone low-pass, cross-feedback spread, mix; time changes glide.<br>*No tempo sync yet.* | `limeshot-dsp effects::tests::delay_echoes_after_the_delay_time`<br>`limeshot-engine tests/audio.rs::insert_effects_process_the_track_and_extend_the_tail` |
| FX-009 | Distortion / saturation / waveshaper | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-010 | Chorus / flanger / phaser | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-011 | Pitch shifting | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-012 | Stereo processing | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-013 | Utility (gain, phase, balance, mono) | P2 |  | VERIFIED | Utility: gain, stereo width (M/S), polarity. | `limeshot-dsp effects::tests::utility_gain_width_polarity` |
| FX-014 | Effect presets | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |
| FX-015 | Analysers (spectrum, oscilloscope) | P2 |  | NOT_STARTED | Original implementation from documented DSP algorithms. | — |

## Automation

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| AUT-001 | Automation clips with envelope points | P1 |  | VERIFIED | Right-click a channel volume/pan knob, a mixer fader/pan or an effect parameter › Create automation clip: a flat curve at the current value spanning the song, on the first free Playlist track. Clips are linked instances (Make unique copies the curve). In the clip: drag a point (snapped; Alt free), left- or right-click the line to add a point, right-click a point to delete it. The picker lists automations to place them; ▾ menu: rename, reset to a flat line, delete automation.<br>*Evidence: docs/research/evidence/screenshots/fl-automation-clip.png. FL also adds an automation-clip channel to the Channel Rack; LimeShot lists automations in the Playlist picker instead (known limitation).* | `limeshot-core automation::tests::curve_holds_outside_and_interpolates_inside`<br>`limeshot-core automation::tests::normalize_sorts_clamps_and_dedups`<br>`limeshot-engine tests/audio.rs::automation_sweeps_channel_pan_during_song_playback`<br>`limeshot-app tests/ui.rs::automation_clips_are_created_from_controls_and_edited_in_the_playlist` |
| AUT-002 | Curve shapes and interpolation | P1 |  | VERIFIED | Single-curve segments with a tension of −1…1 (power curve), edited by dragging the ○ handle between two points vertically; values hold before the first and after the last point and between clips.<br>*FL's other curve modes (hold, stairs, pulse, wave) are not implemented.* | `limeshot-core automation::tests::curve_holds_outside_and_interpolates_inside`<br>`limeshot-engine tests/audio.rs::mixer_automation_holds_between_clips_and_only_runs_in_song_mode` |
| AUT-003 | Automate mixer and channel parameters | P1 |  | VERIFIED | Targets: channel volume and pan, mixer track volume and pan (master included), and every insert-effect parameter. Values use the control's own scale (faders cube-root, frequencies logarithmic). Applied in Song mode every 64 frames, also in exports.<br>*Controls do not move on screen while automated. Synth/sampler parameters and tempo (AUT-008) are not automatable yet.* | `limeshot-engine tests/audio.rs::automation_sweeps_channel_pan_during_song_playback`<br>`limeshot-engine tests/audio.rs::mixer_automation_holds_between_clips_and_only_runs_in_song_mode`<br>`limeshot-engine tests/audio.rs::automation_drives_effect_parameters`<br>`limeshot-core automation::tests::mappings_round_trip` |
| AUT-004 | Record automation from control movements | P2 |  | NOT_STARTED | Planned. | — |
| AUT-005 | Event editor | P2 |  | NOT_STARTED | Planned. | — |
| AUT-006 | Parameter linking to controllers | P2 |  | NOT_STARTED | Planned. | — |
| AUT-007 | LFO / formula / peak controllers | P3 |  | NOT_STARTED | Planned. | — |
| AUT-008 | Tempo automation | P2 |  | NOT_STARTED | Planned: the engine timeline assumes one tempo; tempo automation needs a tempo map. | — |
| AUT-009 | Plugin parameter automation | P2 |  | NOT_STARTED | Planned (after plugin hosting). | — |
| AUT-010 | Automation clip channels in the Channel Rack | P2 |  | NOT_STARTED | Planned. Automations are listed in the Playlist picker instead. | — |
| AUT-011 | Init song with this position | P3 |  | NOT_STARTED | Planned. | — |
| AUT-012 | Automation point modes | P2 |  | NOT_STARTED | Planned: LimeShot has single-curve tension only (AUT-002). | — |
| AUT-013 | Automated controls follow their automation | P2 |  | NOT_STARTED | Planned: the engine applies automation but the UI shows the set value. | — |

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
| BR-001 | Folder navigation (bundled + user folders) | P0 | ✓ | VERIFIED | Bundled LimeShot samples plus folders added with +; collapsible tree; ⟳ re-reads.<br>*The UI test opens a user folder and a subfolder; only WAV files are listed, sorted by name.* | `crates/limeshot-app/tests/ui.rs::browser_folders_open_and_filter` |
| BR-002 | Sample preview | P0 | ✓ | VERIFIED | Click a WAV to preview it through the engine (master).<br>*The UI test clicks a bundled sample and waits for a preview voice in the (silent) engine.* | `crates/limeshot-app/tests/ui.rs::browser_click_previews_and_double_click_loads_a_sample` |
| BR-003 | Load sample into channel / new channel | P0 | ✓ | VERIFIED | Double-click loads into the selected sampler (or creates one); context menu; drag onto the rack or a channel name. | `crates/limeshot-app/tests/ui.rs::browser_click_previews_and_double_click_loads_a_sample`<br>`crates/limeshot-app/tests/ui.rs::dragging_a_sample_to_the_rack_makes_a_channel_or_replaces_one` |
| BR-004 | Search / filter | P1 |  | VERIFIED | Substring filter on file names.<br>*No wildcards/boolean logic.* | `crates/limeshot-app/tests/ui.rs::browser_folders_open_and_filter` |
| BR-005 | Current project tree | P1 |  | IMPLEMENTED | Lists patterns (click to select) and channels. | — |
| BR-006 | Favorites / starred | P2 |  | NOT_STARTED | Star files; favorites section. | — |
| BR-007 | Tags | P3 |  | NOT_STARTED | — | — |
| BR-008 | Plugin and preset browser | P2 |  | NOT_STARTED | — | — |
| BR-009 | Online content | P3 |  | DEFERRED | Out of scope (offline-first). | — |
| BR-010 | Drag samples to the Playlist (audio clips) | P1 |  | VERIFIED | Drag a sample from the Browser onto the Playlist: it is decoded, placed at the snapped drop position as an audio clip as long as the sample, and its audio-clip channel is reused when the same sample is dropped again. | `limeshot-app tests/ui.rs::samples_become_audio_clips_that_trim_from_the_left_edge` |

## Plugins

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PLG-001 | CLAP hosting | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-002 | VST3 hosting | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
| PLG-003 | Audio Unit hosting (macOS) | P2 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-004 | Plugin scanning | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
| PLG-005 | Plugin state persistence | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
| PLG-006 | Plugin editor windows | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
| PLG-007 | Plugin parameter automation | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
| PLG-008 | Plugin preset management | P3 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started. | — |
| PLG-009 | Plugin latency reporting | P1 |  | NOT_STARTED | Planned behind a dedicated crate where FFI `unsafe` is confined (as ArtCraft's hosts do); not started.<br>*VST support is a mandatory next step (project owner, 2026-10-21; next-steps.md).* | — |
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
| EXP-012 | MIDI export | P1 |  | VERIFIED | File › Export MIDI file (pattern / song)… writes a format-1 file at 960 PPQ from the compiled program, so it matches playback (swing included): a conductor track with tempo and time signature, then one named track per channel. Audio clips are skipped. CLI: limeshot-cli midi project.lime out.mid [--pattern]. | `limeshot-project midi::tests::write_then_read_round_trips`<br>`limeshot-app tests/ui.rs::midi_export_and_import_round_trip_through_the_app` |
| EXP-013 | Wrap the tail into the loop | P3 |  | NOT_STARTED | Planned. | — |

## Transport & Time

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| TR-001 | Realtime audio output | P0 | ✓ | VERIFIED | cpal output (CoreAudio/WASAPI/ALSA); lock-free control queue; snapshots returned for freeing off the audio thread; silent clock when no device.<br>*Manual: `limeshot-cli play out/demo.lime` on macOS 26 (MacBook Pro Speakers, 48 kHz): 563 callbacks in 6 s, peak −3.7 dBFS, 1.1% load, position 4:01 at 6 s (correct for 124 BPM).* | `limeshot-audio tests::silent_player_runs_the_transport`<br>`limeshot-audio tests::device_writer_handles_mono_and_wide_devices`<br>`limeshot-engine tests/audio.rs::realtime_transport_loops_pattern_and_reports_position` |
| TR-002 | Play / pause / stop | P0 | ✓ | VERIFIED | Toolbar buttons and Space/Cmd+Space; stop returns to where playback started. | `limeshot-app tests/ui.rs::space_starts_and_stops_the_transport`<br>`limeshot-engine tests/audio.rs::realtime_transport_loops_pattern_and_reports_position` |
| TR-003 | Pattern / Song mode | P0 | ✓ | VERIFIED | Toolbar PAT/SONG switch and L. | `limeshot-app tests/ui.rs::pattern_and_mode_shortcuts_with_undo`<br>`limeshot-engine tests/audio.rs::song_mode_plays_the_arrangement` |
| TR-004 | Tempo | P0 | ✓ | VERIFIED | Toolbar field (drag/type), 10–522 BPM stored as milli-BPM; changes while playing keep the musical position. | `limeshot-core time::tests::tempo_clamps_and_formats`<br>`limeshot-engine tests/audio.rs::tempo_change_while_playing_keeps_position`<br>`limeshot-engine tests/audio.rs::tempo_change_moves_onsets` |
| TR-005 | Time signature | P0 | ✓ | VERIFIED | Toolbar numerator/denominator; affects bar length, grid, metronome downbeat and auto pattern length.<br>*Project-wide only.* | `limeshot-core time::tests::signature_lengths`<br>`crates/limeshot-app/tests/ui.rs::toolbar_time_signature_and_position_readout` |
| TR-006 | Position display | P0 | ✓ | VERIFIED | Bar:beat:tick and minutes:seconds; click returns to start.<br>*The readout exposes its text to accessibility (session 7), which the test reads.* | `limeshot-core time::tests::bar_beat_tick_display`<br>`crates/limeshot-app/tests/ui.rs::toolbar_time_signature_and_position_readout` |
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
| UI-001 | Main window layout (toolbar, browser, workspace) | P0 | ✓ | VERIFIED | FL Studio's window model with a modern, ArtCraft-like finish: header with menus, flat toolbar with window buttons, Browser docked left, and a workspace where the Playlist (maximised by default), Channel Rack, Piano Roll, Mixer and Channel Settings are windows with flat custom title bars; status bar with hints. | `limeshot-app tests/ui.rs::renders_every_panel_without_panicking`<br>`limeshot-app tests/ui.rs::new_project_opens_fl_studios_window_layout`<br>`limeshot-app tests/ui.rs::opening_windows_does_not_modify_the_project` |
| UI-002 | Workspace windows: toggle, maximise, arrange | P0 | ✓ | VERIFIED | F5/F6/F7/F9 and the toolbar buttons toggle the Playlist, Channel Rack, Piano Roll and Mixer windows (FL); each window can be moved, resized, maximised to the workspace and closed; Cmd+Shift+H arranges windows to the defaults. | `limeshot-app tests/ui.rs::f_keys_toggle_windows`<br>`limeshot-app tests/ui.rs::renders_every_panel_without_panicking` |
| UI-003 | LimeShot theme tokens and drawn icons | P0 | ✓ | VERIFIED | Semantic tokens (Lime Dark: neutral near-black surfaces, lime accent), Inter and JetBrains Mono (OFL) fonts, flat controls, upper-case section labels; all icons and the logo drawn in code.<br>*Single theme. Verified visually: offscreen renders (docs/screenshots) and a live macOS capture.* | `limeshot-app tests/ui.rs::renders_every_panel_without_panicking` |
| UI-004 | Hint bar and tooltips | P1 |  | IMPLEMENTED | Status bar shows hover hints; every control has a tooltip. | — |
| UI-005 | UI scaling / HiDPI | P1 |  | IMPLEMENTED | egui pixels-per-point plus View › Zoom (Cmd+= / Cmd+- / Cmd+0), persisted. | — |
| UI-006 | Additional themes (light, high contrast) | P3 |  | NOT_STARTED | Token set is ready for more themes. | — |
| UI-007 | Detachable windows (multi-monitor) | P2 |  | VERIFIED | Any editor window can be detached into its own OS window (egui viewport) from its title bar and re-attached; shortcuts work inside detached windows. `limeshot --detach=mixer` opens one detached at start.<br>*Detached state and positions are not persisted.* | `limeshot-app tests/ui.rs::renders_every_panel_without_panicking (embedded fallback)`<br>`manual: limeshot --demo --detach=mixer --show=piano on macOS 26 — a second OS window 'Mixer — LimeShot' appeared and was captured (docs/screenshots/live-mixer-detached.png)` |
| UI-008 | Offscreen UI rendering for review/regression | P1 |  | VERIFIED | `cargo run -p limeshot-app --example ui_shot -- out.png` renders the window with wgpu offscreen. | `cargo run -p limeshot-app --example ui_shot (docs/screenshots/*.png)` |
| UI-009 | Channel Settings window | P1 |  | VERIFIED | Clicking a channel button in the Channel Rack toggles the Channel Settings window for that channel (sampler or LimeSynth parameters, root key, routing).<br>*Was a docked Inspector in session 2 (D19, superseded by D24).* | `limeshot-app tests/ui.rs::clicking_a_channel_button_toggles_its_settings_window`<br>`limeshot-app tests/ui.rs::renders_every_panel_without_panicking` |
| UI-010 | Control context menu | P1 |  | IN_PROGRESS | Right-click › Create automation clip on channel volume/pan, mixer volume/pan and effect parameters. Reset is double-click. Copy/paste value and type-in are planned.<br>*Observed 2026-10-20 (session 4) on a Channel Rack pan knob.* | — |
| UI-011 | Customizable keyboard shortcuts | P2 |  | NOT_STARTED | Planned: a shortcuts page in the settings that rebinds the actions in keybindings.json, stored in settings.json, with conflict detection. Today the bindings are fixed and listed in Help › Keyboard shortcuts (F1). | — |

## Platform

| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |
|---|---|---|---|---|---|---|
| PF-001 | macOS Apple Silicon | P0 | ✓ | VERIFIED | Built, tested (all tests), GUI launched, realtime audio verified on macOS 26.6.2 arm64. | `cargo test --workspace (local)` |
| PF-002 | macOS Intel | P0 | ✓ | IMPLEMENTED | x86_64-apple-darwin release build runs under Rosetta; headless render bit-identical to arm64. GUI and audio device not exercised on Intel. | `local: arch -x86_64 limeshot-cli demo … (render compared with cmp)` |
| PF-003 | Windows x86-64 | P0 | ✓ | IN_PROGRESS | `cargo check --target x86_64-pc-windows-msvc` passes locally; CI job defined but not yet run. | — |
| PF-004 | Linux x86-64 | P0 | ✓ | IN_PROGRESS | CI job defined (ALSA, X11/Wayland, GTK deps); not yet run. Local cross-check stops at alsa-sys (needs target pkg-config). | — |
| PF-005 | Windows ARM64 / Linux ARM64 | P2 |  | IN_PROGRESS | `cargo check --target aarch64-pc-windows-msvc` passes locally; CI jobs defined. | — |
| PF-006 | Packaging (DMG, MSI, AppImage/deb) | P1 |  | NOT_STARTED | Signed bundles with the samples folder next to the binary. | — |
