//! Headless UI tests: the real `LimeShotApp` UI driven with synthetic input (no window, no
//! audio device). Pointer positions come from the geometry each editor records while drawing
//! (`channel_rack::State::step_pos`, `piano_roll::Geometry`, `playlist::Geometry`), so the tests
//! follow layout changes.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use limeshot_core::*;
use limeshot_ui::workspace::Win;
use limeshot_ui::{LimeShotApp, Settings};

fn harness(demo: bool) -> Harness<'static, LimeShotApp> {
    let samples = limeshot_app::bundled_samples_dir();
    let mut app = LimeShotApp::new_silent(Settings::default(), samples.clone());
    if demo {
        let doc = limeshot_app::demo::build(&samples.expect("samples")).expect("demo");
        app.doc.replace(doc.project().clone());
    }
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
        |ui, app: &mut LimeShotApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    for _ in 0..4 {
        h.step();
    }
    h
}

fn press(h: &mut Harness<'static, LimeShotApp>, p: Pos2, button: PointerButton, pressed: bool) {
    h.event(Event::PointerButton {
        pos: p,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    });
    h.step();
}

fn click(h: &mut Harness<'static, LimeShotApp>, p: Pos2, button: PointerButton) {
    h.event(Event::PointerMoved(p));
    h.step();
    press(h, p, button, true);
    press(h, p, button, false);
    h.step();
}

fn drag(h: &mut Harness<'static, LimeShotApp>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.step();
    press(h, from, PointerButton::Primary, true);
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    press(h, to, PointerButton::Primary, false);
}

#[test]
fn renders_every_panel_without_panicking() {
    let mut h = harness(true);
    for w in Win::ALL {
        h.state_mut().show_window(w);
        h.step();
        h.step();
    }
    let c = h.state().project().channels[3].id;
    h.state_mut().select_channel(c);
    h.state_mut().ui.mixer.selected = 4;
    h.step();
    // Maximise, detach (embedded in tests: no multi-viewport backend) and close every window.
    for w in Win::ALL {
        h.state_mut().ui.workspace.state_mut(w).maximized = true;
        h.step();
        h.state_mut().ui.workspace.state_mut(w).detached = true;
        h.step();
        h.state_mut().ui.workspace.close(w);
        h.step();
    }
    let mut h = harness(false);
    h.step();
}

#[test]
fn new_project_opens_fl_studios_window_layout() {
    let h = harness(false);
    let ws = &h.state().ui.workspace;
    assert!(
        ws.is_open(Win::Playlist) && ws.state(Win::Playlist).maximized,
        "Playlist maximised"
    );
    assert!(
        ws.is_open(Win::ChannelRack) && !ws.state(Win::ChannelRack).maximized,
        "Channel Rack floating"
    );
    assert!(!ws.is_open(Win::PianoRoll) && !ws.is_open(Win::Mixer));
}

#[test]
fn f_keys_toggle_windows() {
    let mut h = harness(false);
    h.key_press(Key::F7);
    h.step();
    assert!(h.state().is_window_open(Win::PianoRoll));
    h.key_press(Key::F9);
    h.step();
    assert!(h.state().is_window_open(Win::Mixer));
    h.key_press(Key::F9);
    h.step();
    assert!(!h.state().is_window_open(Win::Mixer), "pressing again closes it");
    h.key_press(Key::F6);
    h.step();
    assert!(!h.state().is_window_open(Win::ChannelRack));
}

#[test]
fn clicking_a_channel_button_toggles_its_settings_window() {
    let mut h = harness(true);
    let kick = h.state().project().channels[0].id;
    let step = h.state().ui.rack.step_pos(kick, 0).expect("strip");
    // The channel button sits left of the step strip (its right edge is ~6 px before the strip).
    let button = step - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Primary);
    assert!(h.state().is_window_open(Win::ChannelSettings));
    assert_eq!(h.state().selected_channel(), Some(kick));
    for _ in 0..3 {
        h.step();
    }
    let button = h.state().ui.rack.step_pos(kick, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Primary);
    assert!(
        !h.state().is_window_open(Win::ChannelSettings),
        "a second click closes it"
    );
}

#[test]
fn pattern_and_mode_shortcuts_with_undo() {
    let mut h = harness(false);
    assert_eq!(h.state().project().patterns.len(), 1);
    h.key_press(Key::F4);
    h.step();
    assert_eq!(h.state().project().patterns.len(), 2);
    let new = h.state().project().patterns[1].id;
    assert_eq!(h.state().current_pattern(), new, "a new pattern becomes current");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(h.state().project().patterns.len(), 1, "Cmd+Z undoes it");
    h.key_press(Key::L);
    h.step();
    assert_eq!(h.state().ui.mode, limeshot_engine::PlayMode::Song);
}

#[test]
fn space_starts_and_stops_the_transport() {
    let mut h = harness(true);
    h.key_press(Key::Space);
    h.step();
    let t0 = std::time::Instant::now();
    while !h.state().is_playing() && t0.elapsed().as_secs() < 3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.step();
    }
    assert!(h.state().is_playing());
    h.key_press(Key::Space);
    h.step();
    let t0 = std::time::Instant::now();
    while h.state().is_playing() && t0.elapsed().as_secs() < 3 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.step();
    }
    assert!(!h.state().is_playing());
}

#[test]
fn clicking_a_step_button_writes_a_note() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let lead = h.state().project().channels[3].id;
    let has_step = |h: &Harness<'static, LimeShotApp>| {
        h.state()
            .project()
            .pattern(pat)
            .is_some_and(|p| p.notes_for(lead).any(|n| n.start == 0))
    };
    assert!(!has_step(&h));
    let p = h
        .state()
        .ui
        .rack
        .step_pos(lead, 0)
        .expect("Lead's step strip is visible");
    click(&mut h, p, PointerButton::Primary);
    assert!(has_step(&h), "left-click turned the step on");
    click(&mut h, p, PointerButton::Secondary);
    assert!(!has_step(&h), "right-click turned it off");
    // As in FL Studio 2026, both buttons toggle: a right-click on an unlit step lights it.
    click(&mut h, p, PointerButton::Secondary);
    assert!(has_step(&h), "right-click turned it back on");
    click(&mut h, p, PointerButton::Primary);
    assert!(!has_step(&h), "left-click on a lit step turns it off");
    assert!(h.state().doc.is_modified());
}

#[test]
fn a_drag_starting_on_a_lit_step_erases() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let lead = h.state().project().channels[3].id;
    let steps = |h: &Harness<'static, LimeShotApp>| -> Vec<Tick> {
        h.state()
            .project()
            .pattern(pat)
            .map(|p| p.notes_for(lead).map(|n| n.start).collect())
            .unwrap_or_default()
    };
    let at = |h: &Harness<'static, LimeShotApp>, s| h.state().ui.rack.step_pos(lead, s).expect("strip");
    let (s0, s3) = (at(&h, 0), at(&h, 3));
    drag(&mut h, s0, s3);
    assert_eq!(steps(&h), vec![0, 240, 480, 720]);
    // Starting on lit step 2 and dragging to step 3 paints "off" (FL: the first step decides).
    let (s1, s2) = (at(&h, 1), at(&h, 2));
    drag(&mut h, s1, s2);
    assert_eq!(steps(&h), vec![0, 720]);
}

#[test]
fn dragging_across_steps_paints_them_in_one_undo_step() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let lead = h.state().project().channels[3].id;
    let from = h.state().ui.rack.step_pos(lead, 0).expect("strip");
    let to = h.state().ui.rack.step_pos(lead, 3).expect("strip");
    let undo_before = h.state().doc.history().len();
    drag(&mut h, from, to);
    let steps: Vec<Tick> = h
        .state()
        .project()
        .pattern(pat)
        .map(|p| p.notes_for(lead).map(|n| n.start).collect())
        .unwrap_or_default();
    assert_eq!(steps, vec![0, 240, 480, 720]);
    assert_eq!(h.state().doc.history().len(), undo_before + 1);
}

#[test]
fn clicking_the_playlist_places_the_current_pattern() {
    let mut h = harness(true);
    // The floating Channel Rack covers part of the maximised Playlist (as in FL Studio).
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let before = h.state().project().playlist.clips.len();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    // Track 5, bar 3 — empty in the demo.
    let bar = h.state().project().time_signature.ticks_per_bar();
    click(&mut h, g.pos(2 * bar + 100, 4), PointerButton::Primary);
    let clips = &h.state().project().playlist.clips;
    assert_eq!(clips.len(), before + 1);
    let c = clips.last().expect("clip").clone();
    assert_eq!(c.track, 4);
    assert_eq!(c.pattern(), Some(h.state().current_pattern()));
    assert_eq!(c.start, 2 * bar, "snapped down to the grid");
    click(
        &mut h,
        g.pos(c.start + 200, 4) + vec2(0.0, 6.0),
        PointerButton::Secondary,
    );
    assert_eq!(
        h.state().project().playlist.clips.len(),
        before,
        "right-click deletes it"
    );
}

#[test]
fn picker_selects_the_pattern_to_place() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let melody = h.state().project().patterns[1].id;
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    // The picker column sits left of the track headers; the second row is the Melody pattern.
    let p = egui::pos2(g.grid.left() - 118.0 - 150.0 + 40.0, g.grid.top() + 26.0 + 28.0 + 13.0);
    click(&mut h, p, PointerButton::Primary);
    assert_eq!(h.state().current_pattern(), melody);
}

#[test]
fn piano_roll_click_adds_drag_moves_delete_removes() {
    let mut h = harness(true);
    let lead = h.state().project().channels[3].id;
    h.state_mut().select_channel(lead);
    h.state_mut().show_window(Win::PianoRoll);
    h.state_mut().ui.workspace.state_mut(Win::PianoRoll).maximized = true;
    for _ in 0..3 {
        h.step();
    }
    let pat = h.state().current_pattern();
    let notes = |h: &Harness<'static, LimeShotApp>| -> Vec<Note> {
        h.state()
            .project()
            .pattern(pat)
            .map(|p| p.notes_for(lead).copied().collect())
            .unwrap_or_default()
    };
    assert!(notes(&h).is_empty());
    let g = h.state().ui.piano.geometry.expect("piano roll drawn");
    // Click in the second step of C6 (slightly right of its start): snaps down to step 1.
    click(&mut h, g.pos(STEP_TICKS + 60, 72), PointerButton::Primary);
    let n = notes(&h);
    assert_eq!(n.len(), 1, "click adds a note");
    assert_eq!(
        (n[0].start, n[0].length, n[0].key),
        (STEP_TICKS, 2 * STEP_TICKS, 72),
        "snapped to the step grid; FL's first note is two steps long"
    );
    // Drag it two beats to the right.
    let from = g.pos(STEP_TICKS + 40, 72);
    drag(&mut h, from, from + vec2(2.0 * PPQ as f32 * g.ppt, 0.0));
    let n = notes(&h);
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].start, STEP_TICKS + 2 * PPQ, "moved by two beats");
    assert_eq!(n[0].key, 72, "same pitch");
    // The moved note is selected; Delete removes it.
    h.key_press(Key::Delete);
    h.step();
    assert!(notes(&h).is_empty());
    // One undo brings back the moved note; the move was a single undo step.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(notes(&h).first().map(|n| n.start), Some(STEP_TICKS + 2 * PPQ));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(notes(&h).first().map(|n| n.start), Some(STEP_TICKS));
}

#[test]
fn copy_paste_and_quantize_in_the_piano_roll() {
    let mut h = harness(true);
    let melody = h.state().project().patterns[1].id;
    let lead = h.state().project().channels[3].id;
    let bass = h.state().project().channels[4].id;
    h.state_mut().set_current_pattern(melody);
    h.state_mut().select_channel(lead);
    h.state_mut().show_window(Win::PianoRoll);
    h.step();
    // Focus follows the last clicked editor; set it as a click would.
    h.state_mut().ui.focus = limeshot_ui::app::Focus::PianoRoll;
    let count = |h: &Harness<'static, LimeShotApp>, c| {
        h.state()
            .project()
            .pattern(melody)
            .map(|p| p.notes_for(c).count())
            .unwrap_or(0)
    };
    assert_eq!((count(&h, lead), count(&h, bass)), (10, 0));
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::C);
    h.step();
    h.state_mut().select_channel(bass);
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::V);
    h.step();
    assert_eq!(count(&h, bass), 10, "pasted into the other channel at the same times");
    assert_eq!(h.state().ui.piano.selection.len(), 10, "pasted notes are selected");
    // Nudge the pasted notes off the grid with Alt (no snap) → quantize brings them back.
    let ids = h.state().ui.piano.selection.clone();
    h.state_mut().exec(Command::MoveNotes {
        pattern: melody,
        notes: ids,
        dt: 37,
        dkey: 0,
    });
    h.key_press_modifiers(Modifiers::ALT, Key::Q);
    h.step();
    let starts: Vec<Tick> = h
        .state()
        .project()
        .pattern(melody)
        .map(|p| p.notes_for(bass).map(|n| n.start).collect())
        .unwrap_or_default();
    assert!(starts.iter().all(|s| s % STEP_TICKS == 0), "{starts:?}");
}

#[test]
fn channel_rack_copies_steps_between_channels() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let kick = h.state().project().channels[0].id;
    let lead = h.state().project().channels[3].id;
    h.state_mut().select_channel(kick);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::COMMAND, Key::C);
    h.step();
    h.state_mut().select_channel(lead);
    h.key_press_modifiers(Modifiers::COMMAND, Key::V);
    h.step();
    let p = h.state().project().pattern(pat).expect("pattern").clone();
    let root = h.state().project().channel(lead).expect("lead").root_key;
    assert!(
        p.is_step_sequence(lead, root),
        "pasted steps stay steps on the target channel"
    );
    assert_eq!(p.notes_for(lead).count(), 4);
}

#[test]
fn opening_windows_does_not_modify_the_project() {
    let mut h = harness(true);
    h.state_mut().ui.mixer.selected = 4;
    for w in Win::ALL {
        h.state_mut().show_window(w);
        for _ in 0..3 {
            h.step();
        }
    }
    assert!(
        !h.state().doc.is_modified(),
        "viewing changed the project: {:?}",
        h.state().doc.history()
    );
}

#[test]
fn mixer_route_buttons_route_the_selected_track() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    h.state_mut().ui.mixer.selected = 1;
    for _ in 0..3 {
        h.step();
    }
    let button = |h: &Harness<'static, LimeShotApp>, strip: u16| {
        h.state()
            .ui
            .mixer
            .route_buttons
            .iter()
            .find(|(s, _)| *s == strip)
            .map(|(_, r)| r.center())
            .expect("route button")
    };
    let p = button(&h, 6);
    click(&mut h, p, PointerButton::Primary);
    let routes = h.state().project().mixer.tracks[1].routes.clone();
    assert!(routes.iter().any(|r| r.to == 6), "{routes:?}");
    // Selecting the bus: the kick's strip button is blocked (it would loop) and clicking does nothing.
    h.state_mut().ui.mixer.selected = 6;
    h.step();
    h.step();
    let p = button(&h, 1);
    click(&mut h, p, PointerButton::Primary);
    assert!(h.state().project().mixer.tracks[6].routes.iter().all(|r| r.to != 1));
}

/// Found in a live session: the Mixer laid its body out in child UIs that did not advance the
/// cursor, so its frame covered only the title bar and the windows behind showed through.
#[test]
fn floating_window_frames_cover_their_body() {
    let mut h = harness(true);
    for w in Win::ALL {
        h.state_mut().show_window(w);
        h.state_mut().ui.workspace.state_mut(w).maximized = false;
    }
    h.step();
    h.step();
    for w in Win::ALL {
        let r = h.ctx.memory(|m| m.area_rect(w.id())).expect("window drawn");
        assert!(
            r.height() >= w.min_size().y,
            "{w:?} frame is only {} px high",
            r.height()
        );
    }
}

#[test]
fn samples_become_audio_clips_that_trim_from_the_left_edge() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().ui.playlist.zoom_x = 200.0;
    h.step();
    let kick = limeshot_app::bundled_samples_dir()
        .expect("samples")
        .join("Lime Kick.wav");
    let channels = h.state().project().channels.len();
    let bar = h.state().project().time_signature.ticks_per_bar();
    let id = limeshot_ui::playlist::add_audio_clip(h.state_mut(), &kick, 4, bar).expect("placed");
    let c = h.state().project().clip(id).expect("clip").clone();
    assert_eq!(
        h.state().project().channels.len(),
        channels + 1,
        "an audio clip channel"
    );
    assert!(c.audio_channel().is_some());
    // The clip is as long as the sample at the project tempo.
    let secs = limeshot_dsp::load_wav(&kick).expect("wav").duration_secs();
    let want = secs * h.state().project().tempo.bpm() / 60.0 * PPQ as f64;
    assert!((c.length as f64 - want).abs() <= 1.0, "{} vs {want}", c.length);
    for _ in 0..3 {
        h.step();
    }
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let from = g.pos(c.start, 4) + vec2(2.0, 0.0);
    drag(&mut h, from, from + vec2(STEP_TICKS as f32 * g.ppt, 0.0));
    let c = h.state().project().clip(id).expect("clip").clone();
    assert_eq!(
        (c.start, c.offset),
        (bar + STEP_TICKS, STEP_TICKS),
        "left edge trimmed the start"
    );
}

#[test]
fn automation_clips_are_created_from_controls_and_edited_in_the_playlist() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let clip = limeshot_ui::automation_ui::create(h.state_mut(), AutomationTarget::MixerVolume(1)).expect("created");
    for _ in 0..3 {
        h.step();
    }
    let c = h.state().project().clip(clip).expect("clip").clone();
    let a = c.automation().expect("automation clip");
    let sig = h.state().project().time_signature;
    assert_eq!((c.start, c.length), (0, h.state().project().playlist.song_length(sig)));
    assert!(
        h.state()
            .project()
            .playlist
            .clips
            .iter()
            .all(|o| o.id == clip || o.track != c.track),
        "placed on a free track"
    );
    let points = |h: &Harness<'static, LimeShotApp>| h.state().project().automation(a).expect("a").points.clone();
    // Flat at the fader's position (cube-root scale of a 2.0 travel).
    let flat = (h.state().project().mixer.tracks[1].volume / 2.0).cbrt();
    assert_eq!(points(&h).len(), 1);
    assert!((points(&h)[0].value - flat).abs() < 1e-4);

    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let top = g.grid.top() + c.track as f32 * g.track_h - g.scroll.y;
    let (area_top, area_bottom) = (top + 18.0, top + g.track_h - 4.0);
    let y_of = |v: f32| area_bottom - v * (area_bottom - area_top);
    let bar = sig.ticks_per_bar();
    let x = g.pos(2 * bar, c.track).x;
    // Right-click on the line adds a point there (FL Studio).
    click(&mut h, egui::pos2(x, y_of(flat)), PointerButton::Secondary);
    assert_eq!(points(&h).len(), 2, "right-click on the line adds a point");
    assert!(h.state().project().clip(clip).is_some(), "the clip was not deleted");
    // Drag the new point to the top: one undo step.
    let undo = h.state().doc.history().len();
    drag(&mut h, egui::pos2(x, y_of(flat)), egui::pos2(x, area_top - 2.0));
    let p = points(&h)[1];
    assert_eq!((p.tick, p.value), (2 * bar, 1.0));
    assert_eq!(h.state().doc.history().len(), undo + 1);
    // Right-click the point deletes it.
    click(&mut h, egui::pos2(x, y_of(1.0)), PointerButton::Secondary);
    assert_eq!(points(&h).len(), 1);
}

#[test]
fn midi_export_and_import_round_trip_through_the_app() {
    let mut h = harness(true);
    let dir = std::env::temp_dir().join(format!("limeshot-midi-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("song.mid");
    let song_notes = {
        let snap = h.state().snapshot().clone();
        snap.song.events.len()
    };
    assert!(h.state_mut().export_midi(&path, true));
    let patterns = h.state().project().patterns.len();
    let channels = h.state().project().channels.len();
    assert!(h.state_mut().import_midi(&path));
    let p = h.state().project();
    assert_eq!(p.patterns.len(), patterns + 1);
    let imported = p.patterns.last().expect("pattern");
    assert_eq!(imported.name, "song");
    assert_eq!(imported.notes.len(), song_notes, "every song note comes back");
    assert_eq!(h.state().current_pattern(), imported.id, "the import is selected");
    assert!(p.channels.len() > channels);
    assert_eq!(p.tempo, Tempo::from_bpm(124.0), "a project with notes keeps its tempo");
    let _ = std::fs::remove_dir_all(&dir);
}
