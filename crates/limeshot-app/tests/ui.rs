//! Headless UI tests: the real `LimeShotApp` UI driven with synthetic input (no window, no
//! audio device). Pointer positions come from the geometry each editor records while drawing
//! (`channel_rack::State::step_pos`, `piano_roll::Geometry`, `playlist::Geometry`), so the tests
//! follow layout changes.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
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

/// A click with modifier keys held (Ctrl/Cmd-click, Shift-click …).
fn click_with(h: &mut Harness<'static, LimeShotApp>, p: Pos2, button: PointerButton, mods: Modifiers) {
    h.event(Event::ModifiersChanged(mods));
    h.event(Event::PointerMoved(p));
    h.step();
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: p,
            button,
            pressed,
            modifiers: mods,
        });
        h.step();
    }
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.step();
}

/// Two clicks within egui's double-click window: the harness advances 0.25 s per frame, so the
/// frame times are set explicitly.
fn double_click(h: &mut Harness<'static, LimeShotApp>, p: Pos2) {
    h.event(Event::PointerMoved(p));
    h.step();
    let t0 = h.ctx.input(|i| i.time) + 1.0;
    for (k, pressed) in [true, false, true, false].into_iter().enumerate() {
        h.input_mut().time = Some(t0 + k as f64 * 0.03);
        h.event(Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        h.step();
    }
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
    // Rows are laid out from the panel top, which is the 32 px ruler above the grid.
    let top = g.grid.top() - 32.0;
    let p = egui::pos2(g.grid.left() - 118.0 - 150.0 + 40.0, top + 20.0 + 26.0 + 28.0 + 13.0);
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
    // Drag it two beats to the right, grabbing its body (the outer 6 px are resize edges).
    let from = g.pos(STEP_TICKS + 120, 72);
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

#[test]
fn autosave_survives_a_crash_and_is_removed_once_saved() {
    let dir = tempfile::tempdir().expect("tempdir");
    let autosave = || {
        let mut a = limeshot_ui::autosave::Autosave::new(Some(dir.path().join("autosave")));
        a.interval = Some(std::time::Duration::ZERO);
        a
    };
    let mut h = harness(true);
    h.state_mut().autosave = autosave();
    h.state_mut().exec(Command::SetTempo(Tempo::from_bpm(99.0)));
    h.step();
    let info = h.state().autosave.find().expect("an autosave was written");
    assert_eq!(info.original, None);
    // "Crash": a new instance finds the copy and offers it.
    drop(h);
    let mut h = harness(false);
    h.state_mut().autosave = autosave();
    h.step();
    h.step();
    assert!(
        h.state().autosave.find().is_some(),
        "a clean start does not delete the crash copy"
    );
    h.state_mut().offer_recovery();
    assert!(matches!(
        h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::Recover(_))
    ));
    h.state_mut().dialog = None;
    assert!(h.state_mut().recover(&info));
    assert_eq!(h.state().project().tempo, Tempo::from_bpm(99.0));
    assert!(h.state().doc.is_modified(), "recovered work is unsaved");
    assert!(h.state().project().channels.len() >= 5, "the demo's channels came back");
    // Saving removes the copy.
    let saved = dir.path().join("saved.lime");
    assert!(h.state_mut().save_to(&saved));
    h.step();
    assert!(h.state().autosave.find().is_none());
}

#[test]
fn opening_an_flp_imports_it_as_an_unsaved_project() {
    let mut h = harness(true);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../limeshot-project/tests/fixtures/flp/fl26/lime-steps.flp");
    h.state_mut().perform(limeshot_ui::app::Pending::Open(Some(path)));
    h.step();
    let app = h.state();
    assert_eq!(app.path, None, "Save asks where to write the .lime");
    assert!(app.doc.is_modified());
    assert_eq!(app.project().title, "lime-steps");
    assert!(app.window_title().starts_with("• lime-steps"), "{}", app.window_title());
    let names: Vec<&str> = app.project().channels.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        ["808 Kick", "808 Clap", "808 HiHat", "808 Snare", "808 Astronomic"]
    );
    let pat = app.current_pattern();
    let kick = app.project().channels[0].id;
    let steps: Vec<Tick> = app
        .project()
        .pattern(pat)
        .expect("pattern")
        .notes_for(kick)
        .map(|n| n.start / STEP_TICKS)
        .collect();
    assert_eq!(steps, [0, 4, 8, 12]);
    assert!(
        app.toasts.iter().any(|t| t.text.contains("5 channels")),
        "the import report is shown"
    );
    // The rack shows the imported steps.
    assert!(app.ui.rack.step_pos(kick, 0).is_some());
}

fn led(h: &Harness<'static, LimeShotApp>, c: ChannelId) -> Pos2 {
    h.state().ui.rack.led_pos(c).expect("selector LED drawn")
}

fn order(h: &Harness<'static, LimeShotApp>) -> Vec<ChannelId> {
    h.state().project().channels.iter().map(|c| c.id).collect()
}

#[test]
fn selector_leds_select_toggle_and_move_channels() {
    let mut h = harness(true);
    let ids = order(&h);
    let (a, b, c) = (ids[0], ids[1], ids[2]);
    let p = led(&h, b);
    click(&mut h, p, PointerButton::Primary);
    assert_eq!(
        h.state().ui.channel_selection,
        vec![b],
        "a click selects only that channel"
    );
    assert_eq!(h.state().selected_channel(), Some(b));
    let p = led(&h, c);
    click_with(&mut h, p, PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(h.state().selected_channels(), vec![b, c], "Ctrl-click adds");
    let p = led(&h, a);
    click_with(&mut h, p, PointerButton::Primary, Modifiers::SHIFT);
    assert_eq!(
        h.state().selected_channels(),
        vec![a, b, c],
        "Shift-click adds, no range"
    );
    let p = led(&h, a);
    click(&mut h, p, PointerButton::Secondary);
    assert_eq!(h.state().selected_channels(), vec![b, c], "right-click toggles off");

    // Alt+Up moves the selected block up one row, in one undo step.
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowUp);
    h.step();
    assert_eq!(order(&h)[..3], [b, c, a]);
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowUp);
    h.step();
    assert_eq!(order(&h)[..3], [b, c, a], "the block stops at the top");
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.step();
    assert_eq!(order(&h)[..3], [a, b, c]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(order(&h)[..3], [b, c, a], "one undo step per move");

    // Up/Down select the neighbouring channel on its own.
    h.key_press(Key::ArrowDown);
    h.step();
    assert_eq!(h.state().selected_channels(), vec![a]);
    h.key_press(Key::ArrowUp);
    h.step();
    assert_eq!(h.state().selected_channels(), vec![c]);

    // Clicking a step selects its channel only (FL Studio).
    let step = h.state().ui.rack.step_pos(b, 1).expect("strip");
    click(&mut h, step, PointerButton::Primary);
    assert_eq!(h.state().selected_channels(), vec![b]);
}

#[test]
fn deleting_channels_asks_first_and_is_one_undo_step() {
    let mut h = harness(true);
    let ids = order(&h);
    let n = ids.len();
    h.state_mut().select_channel(ids[1]);
    h.state_mut().toggle_channel_selection(ids[2]);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::Delete);
    h.step();
    assert!(
        matches!(
            &h.state().dialog,
            Some(limeshot_ui::dialogs::Dialog::ConfirmDeleteChannels { channels, .. }) if *channels == ids[1..3]
        ),
        "Alt+Delete asks before deleting"
    );
    assert_eq!(h.state().project().channels.len(), n);
    h.get_by_label("OK").click();
    h.step();
    h.step();
    assert!(h.state().dialog.is_none());
    assert_eq!(order(&h), [&ids[..1], &ids[3..]].concat());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(order(&h), ids, "undo brings both back");

    // With the question turned off ("Remember my choice"), the channels go at once.
    h.state_mut().settings.confirm_channel_delete = false;
    h.state_mut().select_channel(ids[0]);
    h.key_press_modifiers(Modifiers::ALT, Key::Delete);
    h.step();
    assert!(h.state().dialog.is_none());
    assert_eq!(h.state().project().channels.len(), n - 1);
}

#[test]
fn cmd_l_routes_each_selected_channel_to_its_own_free_track() {
    let mut h = harness(true);
    let ids = order(&h);
    h.state_mut().select_channel(ids[0]);
    h.state_mut().toggle_channel_selection(ids[1]);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    let before: Vec<u16> = h.state().project().channels.iter().map(|c| c.mixer_track).collect();
    h.key_press_modifiers(Modifiers::COMMAND, Key::L);
    h.step();
    let p = h.state().project();
    let t0 = p.channel(ids[0]).expect("ch").mixer_track;
    let t1 = p.channel(ids[1]).expect("ch").mixer_track;
    assert!(t0 != 0 && t1 != 0 && t0 != t1, "{t0} {t1}");
    for (i, ch) in p.channels.iter().enumerate().skip(2) {
        assert_eq!(ch.mixer_track, before[i]);
        assert!(ch.mixer_track == 0 || (ch.mixer_track != t0 && ch.mixer_track != t1));
    }
    assert_eq!(h.state().doc.history().len(), 1, "one undo step");
}

#[test]
fn f2_in_the_channel_rack_renames_the_current_pattern() {
    let mut h = harness(true);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press(Key::F2);
    h.step();
    let pat = h.state().current_pattern();
    assert!(matches!(
        &h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::Rename {
            target: limeshot_ui::dialogs::RenameTarget::Pattern(p),
            ..
        }) if *p == pat
    ));
}

#[test]
fn right_click_on_a_channel_button_selects_it_and_opens_its_menu() {
    let mut h = harness(true);
    let ids = order(&h);
    h.state_mut().select_channel(ids[0]);
    h.state_mut().toggle_channel_selection(ids[1]);
    let button = h.state().ui.rack.step_pos(ids[2], 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Secondary);
    assert_eq!(h.state().selected_channels(), vec![ids[2]]);
    h.get_by_label("Delete…").click();
    h.step();
    assert!(matches!(
        &h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::ConfirmDeleteChannels { channels, .. }) if *channels == [ids[2]]
    ));
}

#[test]
fn clicking_a_note_preview_in_the_rack_opens_the_piano_roll() {
    let mut h = harness(true);
    let melody = h.state().project().patterns[1].id;
    h.state_mut().set_current_pattern(melody);
    h.step();
    h.step();
    let p = h.state().project().pattern(melody).expect("melody").clone();
    let (ch, root) = h
        .state()
        .project()
        .channels
        .iter()
        .find(|c| p.notes_for(c.id).next().is_some())
        .map(|c| (c.id, c.root_key))
        .expect("a channel with notes");
    assert!(!p.is_step_sequence(ch, root), "the melody is not steps");
    assert!(!h.state().is_window_open(Win::PianoRoll));
    let pos = h.state().ui.rack.step_pos(ch, 3).expect("preview drawn");
    click(&mut h, pos, PointerButton::Primary);
    assert!(h.state().is_window_open(Win::PianoRoll));
    assert_eq!(h.state().selected_channel(), Some(ch));
}

fn bundled(name: &str) -> std::path::PathBuf {
    limeshot_app::bundled_samples_dir().expect("samples").join(name)
}

fn wait_for(h: &mut Harness<'static, LimeShotApp>, what: &str, mut done: impl FnMut(&LimeShotApp) -> bool) {
    let start = std::time::Instant::now();
    while !done(h.state()) {
        assert!(start.elapsed().as_secs() < 10, "timed out waiting for {what}");
        std::thread::sleep(std::time::Duration::from_millis(10));
        h.step();
    }
}

#[test]
fn browser_click_previews_and_double_click_loads_a_sample() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let kick = bundled("Lime Kick.wav");
    let pos = h.state().ui.browser.file_pos(&kick).expect("bundled samples listed");
    click(&mut h, pos, PointerButton::Primary);
    assert_eq!(h.state().ui.browser.selected.as_deref(), Some(kick.as_path()));
    // The preview starts once the loader thread has decoded the file.
    wait_for(&mut h, "the preview", |a| !a.ui.browser.preview_pending());
    wait_for(&mut h, "a preview voice", |a| a.player.status().voices() > 0);

    // Double-click loads it into the selected sampler channel and names the channel after it.
    let clap = h.state().project().channels[1].id;
    h.state_mut().select_channel(clap);
    let pos = h.state().ui.browser.file_pos(&kick).expect("listed");
    double_click(&mut h, pos);
    let ch = h.state().project().channel(clap).expect("clap").clone();
    assert_eq!(ch.name, "Lime Kick");
    match &ch.instrument {
        Instrument::Sampler(s) => assert_eq!(
            s.sample.as_ref().map(|s| std::path::PathBuf::from(&s.path)),
            Some(kick.clone())
        ),
        other => panic!("not a sampler: {other:?}"),
    }
    // On a synth channel a double-click makes a new sampler channel instead.
    let n = h.state().project().channels.len();
    let lead = h
        .state()
        .project()
        .channels
        .iter()
        .find(|c| matches!(c.instrument, Instrument::Synth(_)))
        .expect("synth")
        .id;
    h.state_mut().select_channel(lead);
    double_click(&mut h, pos);
    assert_eq!(h.state().project().channels.len(), n + 1);
    assert_eq!(h.state().selected_channels(), vec![h.state().project().channels[n].id]);
}

#[test]
fn browser_folders_open_and_filter() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sub = dir.path().join("Drums");
    std::fs::create_dir(&sub).expect("mkdir");
    std::fs::copy(bundled("Lime Kick.wav"), sub.join("Big Kick.wav")).expect("copy");
    std::fs::copy(bundled("Lime Clap.wav"), sub.join("Snappy Clap.wav")).expect("copy");
    std::fs::write(sub.join("notes.txt"), "not audio").expect("write");
    let mut h = harness(false);
    h.state_mut().settings.browser_folders.push(dir.path().to_path_buf());
    h.step();
    h.step();
    let name = dir.path().file_name().expect("name").to_string_lossy().into_owned();
    let listed = |h: &Harness<'static, LimeShotApp>| -> Vec<String> {
        h.state()
            .ui
            .browser
            .file_rects
            .iter()
            .filter(|(p, _)| p.starts_with(dir.path()))
            .map(|(p, _)| p.file_name().expect("n").to_string_lossy().into_owned())
            .collect()
    };
    assert!(listed(&h).is_empty(), "user folders start closed");
    h.get_by_label(&name).click();
    h.step();
    h.step();
    h.get_by_label("Drums").click();
    h.step();
    h.step();
    assert_eq!(
        listed(&h),
        ["Big Kick.wav", "Snappy Clap.wav"],
        "only WAV files, sorted"
    );
    h.state_mut().ui.browser.filter = "clap".into();
    h.step();
    h.step();
    assert_eq!(listed(&h), ["Snappy Clap.wav"]);
}

#[test]
fn dragging_a_sample_to_the_rack_makes_a_channel_or_replaces_one() {
    let mut h = harness(true);
    h.step();
    let hat = bundled("Lime Hat Open.wav");
    let n = h.state().project().channels.len();
    let first = h.state().project().channels[0].id;
    // Onto a channel button: replaces that channel's sample.
    let button = h.state().ui.rack.step_pos(first, 0).expect("strip") - vec2(60.0, 0.0);
    let from = h.state().ui.browser.file_pos(&hat).expect("listed");
    drag(&mut h, from, button);
    h.step();
    assert_eq!(h.state().project().channels.len(), n);
    assert_eq!(h.state().project().channel(first).expect("ch").name, "Lime Hat Open");
    // Onto empty rack space below the rows: a new sampler channel.
    let strips = &h.state().ui.rack.step_strips;
    let below = strips.iter().map(|(_, r, _)| r.bottom()).fold(0.0, f32::max) + 20.0;
    let x = strips[0].1.left() + 30.0;
    let from = h.state().ui.browser.file_pos(&hat).expect("listed");
    drag(&mut h, from, egui::pos2(x, below));
    h.step();
    assert_eq!(h.state().project().channels.len(), n + 1);
    assert_eq!(h.state().project().channels[n].name, "Lime Hat Open");
}

fn drag_with(h: &mut Harness<'static, LimeShotApp>, from: Pos2, to: Pos2, mods: Modifiers) {
    h.event(Event::ModifiersChanged(mods));
    drag(h, from, to);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.step();
}

/// The demo's Melody pattern open in a maximised Piano Roll on the Lead channel.
fn melody_piano_roll() -> (Harness<'static, LimeShotApp>, PatternId, ChannelId) {
    let mut h = harness(true);
    let lead = h.state().project().channels[3].id;
    let melody = h.state().project().patterns[1].id;
    h.state_mut().set_current_pattern(melody);
    h.state_mut().select_channel(lead);
    h.state_mut().show_window(Win::PianoRoll);
    h.state_mut().ui.workspace.state_mut(Win::PianoRoll).maximized = true;
    h.state_mut().ui.focus = limeshot_ui::app::Focus::PianoRoll;
    for _ in 0..3 {
        h.step();
    }
    (h, melody, lead)
}

fn pattern_notes(h: &Harness<'static, LimeShotApp>, pat: PatternId, ch: ChannelId) -> Vec<Note> {
    h.state()
        .project()
        .pattern(pat)
        .map(|p| p.notes_for(ch).copied().collect())
        .unwrap_or_default()
}

fn piano_geometry(h: &Harness<'static, LimeShotApp>) -> limeshot_ui::piano_roll::Geometry {
    h.state().ui.piano.geometry.expect("piano roll drawn")
}

#[test]
fn piano_roll_selection_gestures() {
    let (mut h, pat, lead) = melody_piano_roll();
    let notes = pattern_notes(&h, pat, lead);
    let g = piano_geometry(&h);
    let at = |n: &Note| g.pos(n.start + n.length / 3, n.key);
    click(&mut h, at(&notes[1]), PointerButton::Primary);
    assert_eq!(
        h.state().ui.piano.selection,
        vec![notes[1].id],
        "a click selects the note"
    );
    click_with(&mut h, at(&notes[2]), PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(
        h.state().ui.piano.selection,
        vec![notes[1].id, notes[2].id],
        "Cmd-click adds"
    );
    click_with(&mut h, at(&notes[1]), PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(h.state().ui.piano.selection, vec![notes[2].id], "and removes");
    assert_eq!(pattern_notes(&h, pat, lead), notes, "selecting does not edit");

    h.key_press(Key::I); // no modifier: nothing
    h.key_press_modifiers(Modifiers::SHIFT, Key::I);
    h.step();
    let mut inverted: Vec<NoteId> = notes.iter().map(|n| n.id).filter(|id| *id != notes[2].id).collect();
    inverted.sort();
    let mut got = h.state().ui.piano.selection.clone();
    got.sort();
    assert_eq!(got, inverted, "Shift+I inverts the selection");
    h.key_press_modifiers(Modifiers::COMMAND, Key::D);
    h.step();
    assert!(h.state().ui.piano.selection.is_empty(), "Cmd+D deselects");
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.step();
    assert_eq!(h.state().ui.piano.selection.len(), notes.len(), "Cmd+A selects all");

    // Cmd+drag on empty space: rectangle selection of the first bar's notes above key 70.
    let from = g.pos(0, 79) + vec2(2.0, 0.0);
    let to = g.pos(STEP_TICKS * 15, 71);
    drag_with(&mut h, from, to, Modifiers::COMMAND);
    let mut want: Vec<NoteId> = notes
        .iter()
        .filter(|n| n.start < STEP_TICKS * 15 && n.key >= 71)
        .map(|n| n.id)
        .collect();
    want.sort();
    let mut got = h.state().ui.piano.selection.clone();
    got.sort();
    assert_eq!(got, want, "rectangle selection");
    assert_eq!(pattern_notes(&h, pat, lead), notes);

    // Double-click a note: every note of that pitch.
    double_click(&mut h, at(&notes[0]));
    let mut want: Vec<NoteId> = notes.iter().filter(|n| n.key == notes[0].key).map(|n| n.id).collect();
    want.sort();
    let mut got = h.state().ui.piano.selection.clone();
    got.sort();
    assert_eq!(got, want);
    assert_eq!(want.len(), 3, "A4 appears three times in the melody");
}

#[test]
fn piano_roll_velocity_lane_sets_velocity() {
    let (mut h, pat, lead) = melody_piano_roll();
    let notes = pattern_notes(&h, pat, lead);
    let g = piano_geometry(&h);
    let undo = h.state().doc.history().len();
    click(&mut h, g.vel_pos(notes[4].start, 0.25), PointerButton::Primary);
    let vel = |h: &Harness<'static, LimeShotApp>, i: usize| pattern_notes(h, pat, lead)[i].velocity;
    assert!((vel(&h, 4) - 0.25).abs() < 0.03, "{}", vel(&h, 4));
    assert_eq!(vel(&h, 3), notes[3].velocity, "only that note");
    // With notes 0 and 1 selected, dragging note 0's stem sets both, in one undo step.
    h.state_mut().ui.piano.selection = vec![notes[0].id, notes[1].id];
    let undo = undo + 1;
    drag(&mut h, g.vel_pos(notes[0].start, 0.3), g.vel_pos(notes[0].start, 0.9));
    h.step();
    assert!((vel(&h, 0) - 0.9).abs() < 0.03 && (vel(&h, 1) - 0.9).abs() < 0.03);
    assert_eq!(vel(&h, 2), notes[2].velocity);
    assert_eq!(h.state().doc.history().len(), undo + 1, "the drag is one undo step");
}

#[test]
fn piano_roll_wheel_and_arrows_zoom_and_scroll() {
    let (mut h, _, _) = melody_piano_roll();
    let g = piano_geometry(&h);
    let p = g.grid.center();
    let wheel = |h: &mut Harness<'static, LimeShotApp>, dy: f32, mods: Modifiers| {
        h.event(Event::PointerMoved(p));
        h.event(Event::ModifiersChanged(mods));
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: vec2(0.0, dy),
            modifiers: mods,
            phase: egui::TouchPhase::Move,
        });
        for _ in 0..20 {
            h.step();
        }
        h.event(Event::ModifiersChanged(Modifiers::NONE));
        h.step();
    };
    let zoom = h.state().ui.piano.zoom_x;
    wheel(&mut h, 60.0, Modifiers::COMMAND);
    assert!(h.state().ui.piano.zoom_x > zoom, "Cmd+wheel zooms in");
    let y = h.state().ui.piano.scroll.y;
    wheel(&mut h, -60.0, Modifiers::NONE);
    assert!(h.state().ui.piano.scroll.y > y, "the wheel scrolls");
    let x = h.state().ui.piano.scroll.x;
    wheel(&mut h, -60.0, Modifiers::SHIFT);
    assert!(h.state().ui.piano.scroll.x > x, "Shift+wheel scrolls sideways");
    // Plain arrows scroll too (FL Studio); they do not edit notes.
    let y = h.state().ui.piano.scroll.y;
    h.key_press(Key::ArrowDown);
    h.step();
    assert!(h.state().ui.piano.scroll.y > y);
    assert!(!h.state().doc.is_modified());
}

#[test]
fn piano_roll_shift_arrows_transpose_and_nudge() {
    let (mut h, pat, lead) = melody_piano_roll();
    let notes = pattern_notes(&h, pat, lead);
    h.state_mut().ui.piano.selection = vec![notes[0].id];
    let first = |h: &Harness<'static, LimeShotApp>| {
        pattern_notes(h, pat, lead)
            .into_iter()
            .find(|n| n.id == notes[0].id)
            .expect("note")
    };
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowUp);
    h.step();
    assert_eq!(first(&h).key, notes[0].key + 1);
    h.key_press_modifiers(Modifiers::COMMAND, Key::ArrowDown);
    h.step();
    assert_eq!(first(&h).key, notes[0].key - 11);
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    h.step();
    assert_eq!(
        first(&h).start,
        notes[0].start + STEP_TICKS,
        "nudge by the snap (a step)"
    );
    let others: Vec<Note> = pattern_notes(&h, pat, lead)
        .into_iter()
        .filter(|n| n.id != notes[0].id)
        .collect();
    assert_eq!(others, notes[1..], "only the selection moved");
    // Nothing selected: every note of the channel moves (FL Studio).
    h.state_mut().ui.piano.selection.clear();
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    h.step();
    let after = pattern_notes(&h, pat, lead);
    assert!(after.iter().zip(&notes).skip(1).all(|(a, b)| a.key + 1 == b.key));
}

#[test]
fn piano_roll_keyboard_ruler_ghosts_and_channel_selector() {
    let (mut h, pat, lead) = melody_piano_roll();
    let bass = h.state().project().channels[4].id;
    // A Bass note in the same pattern shows as a ghost in Lead's Piano Roll.
    let ghost_start = 2 * h.state().project().time_signature.ticks_per_bar();
    h.state_mut().exec(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel: bass,
            key: 70,
            start: ghost_start,
            length: 4 * STEP_TICKS,
            velocity: 0.8,
        },
    });
    h.step();
    let g = piano_geometry(&h);
    // Right-clicking the ghost does not delete it (it belongs to another channel) …
    click(&mut h, g.pos(ghost_start + STEP_TICKS, 70), PointerButton::Secondary);
    assert_eq!(pattern_notes(&h, pat, bass).len(), 1, "ghosts cannot be edited");
    // … and a left click there adds a Lead note.
    let n = pattern_notes(&h, pat, lead).len();
    click(&mut h, g.pos(ghost_start + STEP_TICKS, 70), PointerButton::Primary);
    assert_eq!(pattern_notes(&h, pat, lead).len(), n + 1);
    assert_eq!(pattern_notes(&h, pat, bass).len(), 1);

    // Holding a key on the keyboard plays the channel.
    let key = egui::pos2(g.keys.right() - 6.0, g.pos(0, 72).y);
    h.event(Event::PointerMoved(key));
    h.step();
    press(&mut h, key, PointerButton::Primary, true);
    wait_for(&mut h, "the auditioned note", |a| a.player.status().voices() > 0);
    press(&mut h, key, PointerButton::Primary, false);

    // Clicking the ruler moves the playhead there (Pattern mode).
    let bar = h.state().project().time_signature.ticks_per_bar();
    let r = egui::pos2(g.pos(bar + 10, 72).x, g.ruler.center().y);
    click(&mut h, r, PointerButton::Primary);
    assert_eq!(h.state().ui.mode, limeshot_engine::PlayMode::Pattern);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == bar);

    // The channel selector switches the edited channel.
    h.get_by_value("Lead").click();
    h.step();
    h.get_by_label("■ Bass").click();
    h.step();
    assert_eq!(h.state().selected_channel(), Some(bass));
}

/// The demo with a maximised Playlist and the Channel Rack closed.
fn playlist_harness() -> Harness<'static, LimeShotApp> {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::Playlist;
    h.step();
    h.step();
    h
}

#[test]
fn playlist_clips_select_resize_and_move_with_shift_arrows() {
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let clips = h.state().project().playlist.clips.clone();
    let (a, b) = (clips[0].clone(), clips[1].clone());
    assert_eq!((a.track, b.track), (0, 0));
    click(&mut h, g.pos(a.start + a.length / 2, 0), PointerButton::Primary);
    assert_eq!(h.state().ui.playlist.selection, vec![a.id], "a click selects");
    click_with(
        &mut h,
        g.pos(b.start + b.length / 2, 0),
        PointerButton::Primary,
        Modifiers::COMMAND,
    );
    assert_eq!(h.state().ui.playlist.selection, vec![a.id, b.id], "Cmd-click adds");
    h.key_press_modifiers(Modifiers::SHIFT, Key::I);
    h.step();
    assert_eq!(
        h.state().ui.playlist.selection.len(),
        clips.len() - 2,
        "Shift+I inverts"
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::D);
    h.step();
    assert!(h.state().ui.playlist.selection.is_empty());

    // Shift+arrows move the selection; plain arrows scroll (FL Studio's Playlist Edit menu).
    h.state_mut().ui.playlist.selection = vec![b.id];
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    h.step();
    let moved = h.state().project().clip(b.id).expect("clip").clone();
    assert_eq!((moved.track, moved.start), (1, b.start + STEP_TICKS));
    let y = h.state().ui.playlist.scroll.y;
    h.key_press(Key::ArrowDown);
    h.step();
    assert!(h.state().ui.playlist.scroll.y > y);
    assert_eq!(
        h.state().project().clip(b.id).expect("clip").track,
        1,
        "plain arrows do not move clips"
    );

    // Dragging the right edge resizes (to the snap: a step).
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let edge = g.pos(a.start + a.length, 0) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(a.start + a.length / 2 + 7, 0));
    h.step();
    assert_eq!(h.state().project().clip(a.id).expect("clip").length, a.length / 2);
}

#[test]
fn playlist_wheel_zoom_and_ruler_set_the_song_position() {
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let zoom = h.state().ui.playlist.zoom_x;
    h.event(Event::PointerMoved(g.grid.center()));
    h.event(Event::ModifiersChanged(Modifiers::COMMAND));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: vec2(0.0, 60.0),
        modifiers: Modifiers::COMMAND,
        phase: egui::TouchPhase::Move,
    });
    for _ in 0..20 {
        h.step();
    }
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.step();
    assert!(h.state().ui.playlist.zoom_x > zoom, "Cmd+wheel zooms");
    // The ruler sits just above the grid: clicking it sets the song position (beat snap) and
    // switches to Song mode.
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let at = egui::pos2(g.pos(2 * bar + 30, 0).x, g.grid.top() - 6.0);
    click(&mut h, at, PointerButton::Primary);
    assert_eq!(h.state().ui.mode, limeshot_engine::PlayMode::Song);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == 2 * bar);
}

#[test]
fn song_playback_drives_the_master_meter() {
    let mut h = playlist_harness();
    h.state_mut().set_mode(limeshot_engine::PlayMode::Song);
    h.state_mut().show_window(Win::Mixer);
    h.step();
    h.key_press(Key::Space);
    h.step();
    wait_for(&mut h, "master level", |a| a.player.status().peak(0).0 > 0.01);
    wait_for(&mut h, "the meter hold", |a| {
        a.meter_hold.first().is_some_and(|m| m.0 > 0.01)
    });
    h.key_press(Key::Space);
    h.step();
    wait_for(&mut h, "the stop", |a| !a.is_playing());
}

#[test]
fn new_project_asks_to_save_unsaved_changes() {
    let mut h = harness(false);
    // Unmodified: Cmd+N starts a new project at once.
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    h.step();
    assert!(h.state().dialog.is_none());
    h.state_mut().exec(Command::SetTempo(Tempo::from_bpm(90.0)));
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    h.step();
    assert!(matches!(
        h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::ConfirmDiscard(
            limeshot_ui::app::Pending::New
        ))
    ));
    h.get_by_label("Cancel").click();
    h.step();
    h.step();
    assert_eq!(
        h.state().project().tempo,
        Tempo::from_bpm(90.0),
        "Cancel keeps the project"
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    h.step();
    h.get_by_label("Don't save").click();
    h.step();
    h.step();
    let p = h.state().project();
    assert_ne!(p.tempo, Tempo::from_bpm(90.0), "a fresh project");
    assert!(p.channels.is_empty() && p.patterns.len() == 1);
    assert!(!h.state().doc.is_modified());
}

#[test]
fn toolbar_time_signature_and_position_readout() {
    let mut h = harness(true);
    h.step();
    // Type 3 into the beats-per-bar field.
    let numerator = h
        .query_all_by_value("4")
        .next()
        .expect("beats-per-bar field")
        .rect()
        .center();
    click(&mut h, numerator, PointerButton::Primary);
    h.event(Event::Key {
        key: Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    });
    h.event(Event::Text("3".into()));
    h.step();
    h.key_press(Key::Enter);
    h.step();
    h.step();
    let sig = h.state().project().time_signature;
    assert_eq!((sig.numerator, sig.denominator), (3, 4));
    assert_eq!(sig.ticks_per_bar(), 3 * PPQ);
    // The readout shows bar:beat:tick of the song position; clicking it returns to the start.
    let bar = sig.ticks_per_bar();
    h.state_mut().locate(bar + PPQ);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == bar + PPQ);
    h.step();
    let readout = h.query_by_label_contains("2:02:000").map(|n| n.rect().center());
    let readout = readout.expect("readout shows 2:02:000");
    click(&mut h, readout, PointerButton::Primary);
    wait_for(&mut h, "the return to the start", |a| a.player.status().position() == 0);
}

fn notes_by_start(h: &Harness<'static, LimeShotApp>, pat: PatternId, ch: ChannelId) -> Vec<(u8, Tick, Tick, bool)> {
    let mut v: Vec<(u8, Tick, Tick, bool)> = pattern_notes(h, pat, ch)
        .iter()
        .map(|n| (n.key, n.start, n.length, n.muted))
        .collect();
    v.sort_by_key(|n| (n.1, n.0));
    v
}

/// An empty pattern for the Lead channel in a maximised Piano Roll.
fn empty_piano_roll() -> (Harness<'static, LimeShotApp>, PatternId, ChannelId) {
    let (mut h, _, lead) = melody_piano_roll();
    let pat = PatternId(
        h.state_mut()
            .exec(Command::AddPattern { name: "Tools".into() })
            .expect("pattern"),
    );
    h.state_mut().set_current_pattern(pat);
    for _ in 0..3 {
        h.step();
    }
    (h, pat, lead)
}

#[test]
fn piano_roll_tool_keys_and_palette() {
    let (mut h, _, _) = empty_piano_roll();
    let history = h.state().doc.history().len();
    use limeshot_ui::piano_roll::Tool;
    for (k, tool) in [
        (Key::B, Tool::Paint),
        (Key::D, Tool::Delete),
        (Key::T, Tool::Mute),
        (Key::C, Tool::Slice),
        (Key::E, Tool::Select),
        (Key::Z, Tool::Zoom),
        (Key::Y, Tool::Playback),
        (Key::P, Tool::Draw),
    ] {
        h.key_press(k);
        h.step();
        assert_eq!(h.state().ui.piano.tool, tool, "{k:?}");
    }
    assert_eq!(h.state().doc.history().len(), history, "switching tools edits nothing");
}

#[test]
fn paint_tool_lays_notes_end_to_end() {
    let (mut h, pat, lead) = empty_piano_roll();
    h.state_mut().ui.piano.tool = limeshot_ui::piano_roll::Tool::Paint;
    let g = piano_geometry(&h);
    let len = h.state().ui.piano.last_len;
    assert_eq!(len, 2 * STEP_TICKS);
    // From the first step to inside the fourth note.
    drag(&mut h, g.pos(10, 72), g.pos(3 * len + 20, 72));
    h.step();
    let notes = notes_by_start(&h, pat, lead);
    assert_eq!(
        notes,
        (0..4).map(|i| (72, i * len, len, false)).collect::<Vec<_>>(),
        "notes of the last length, end to end"
    );
    assert_eq!(h.state().ui.piano.selection.len(), 4, "painted notes are selected");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert!(pattern_notes(&h, pat, lead).is_empty(), "one undo step");
}

#[test]
fn slice_glue_chop_and_mute_tools() {
    let (mut h, pat, lead) = empty_piano_roll();
    let bar = h.state().project().time_signature.ticks_per_bar();
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: vec![
            NewNote {
                channel: lead,
                key: 72,
                start: 0,
                length: bar,
                velocity: 0.8,
            },
            NewNote {
                channel: lead,
                key: 67,
                start: 0,
                length: bar,
                velocity: 0.8,
            },
        ],
    });
    h.step();
    let g = piano_geometry(&h);
    // Slice: a vertical drag across the top note at beat 2 cuts only that note.
    h.state_mut().ui.piano.tool = limeshot_ui::piano_roll::Tool::Slice;
    drag(&mut h, g.pos(PPQ, 74), g.pos(PPQ, 71));
    h.step();
    assert_eq!(
        notes_by_start(&h, pat, lead),
        vec![(67, 0, bar, false), (72, 0, PPQ, false), (72, PPQ, bar - PPQ, false)]
    );
    let right = pattern_notes(&h, pat, lead)
        .into_iter()
        .find(|n| n.start == PPQ)
        .expect("piece");
    assert_eq!(
        h.state().ui.piano.selection,
        vec![right.id],
        "the right piece is selected"
    );
    // Glue (Cmd+G) on everything: the pieces join again.
    h.state_mut().ui.piano.selection.clear();
    h.key_press_modifiers(Modifiers::COMMAND, Key::G);
    h.step();
    assert_eq!(
        notes_by_start(&h, pat, lead),
        vec![(67, 0, bar, false), (72, 0, bar, false)]
    );
    // Quick chop (Cmd+U) the low note at the snap (a step).
    let low = pattern_notes(&h, pat, lead)
        .into_iter()
        .find(|n| n.key == 67)
        .expect("low");
    h.state_mut().ui.piano.selection = vec![low.id];
    h.key_press_modifiers(Modifiers::COMMAND, Key::U);
    h.step();
    let lows: Vec<_> = notes_by_start(&h, pat, lead)
        .into_iter()
        .filter(|n| n.0 == 67)
        .collect();
    assert_eq!(
        lows,
        (0..16)
            .map(|i| (67, i * STEP_TICKS, STEP_TICKS, false))
            .collect::<Vec<_>>()
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(notes_by_start(&h, pat, lead).len(), 2, "chop is one undo step");
    // Mute tool: a click toggles the note; Opt+M / Shift+Opt+M mute and unmute everything.
    h.state_mut().ui.piano.tool = limeshot_ui::piano_roll::Tool::Mute;
    h.state_mut().ui.piano.selection.clear();
    click(&mut h, g.pos(bar / 2, 72), PointerButton::Primary);
    assert_eq!(
        notes_by_start(&h, pat, lead),
        vec![(67, 0, bar, false), (72, 0, bar, true)]
    );
    h.key_press_modifiers(Modifiers::ALT, Key::M);
    h.step();
    assert!(notes_by_start(&h, pat, lead).iter().all(|n| n.3));
    h.key_press_modifiers(Modifiers::ALT | Modifiers::SHIFT, Key::M);
    h.step();
    assert!(notes_by_start(&h, pat, lead).iter().all(|n| !n.3));
}

#[test]
fn delete_select_zoom_and_draw_left_edge() {
    let (mut h, pat, lead) = empty_piano_roll();
    use limeshot_ui::piano_roll::Tool;
    let bar = h.state().project().time_signature.ticks_per_bar();
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: (0..4)
            .map(|i| NewNote {
                channel: lead,
                key: 70 + i as u8,
                start: i * PPQ,
                length: PPQ / 2,
                velocity: 0.8,
            })
            .collect(),
    });
    h.step();
    let g = piano_geometry(&h);
    // Select tool: a plain drag selects (no Cmd needed).
    h.state_mut().ui.piano.tool = Tool::Select;
    drag(&mut h, g.pos(0, 74) + vec2(1.0, 0.0), g.pos(2 * PPQ - 10, 70));
    let notes = pattern_notes(&h, pat, lead);
    let mut want: Vec<NoteId> = notes.iter().filter(|n| n.start < 2 * PPQ).map(|n| n.id).collect();
    want.sort();
    let mut got = h.state().ui.piano.selection.clone();
    got.sort();
    assert_eq!(got, want);
    // Delete tool: a click deletes the note under the pointer.
    h.state_mut().ui.piano.tool = Tool::Delete;
    click(&mut h, g.pos(3 * PPQ + PPQ / 4, 73), PointerButton::Primary);
    assert_eq!(pattern_notes(&h, pat, lead).len(), 3);
    // Draw tool: dragging a note's left edge moves its start and keeps its end.
    h.state_mut().ui.piano.tool = Tool::Draw;
    let n = pattern_notes(&h, pat, lead)
        .into_iter()
        .find(|n| n.key == 72)
        .expect("note");
    let edge = g.pos(n.start, 72) + vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(n.start + STEP_TICKS + 20, 72));
    h.step();
    let m = pattern_notes(&h, pat, lead)
        .into_iter()
        .find(|x| x.id == n.id)
        .expect("note");
    assert_eq!((m.start, m.end()), (n.start + STEP_TICKS, n.end()));
    // Zoom tool: dragging over a bar fits it to the grid; right-click zooms out.
    h.state_mut().ui.piano.tool = Tool::Zoom;
    drag(&mut h, g.pos(bar, 60), g.pos(2 * bar, 60));
    h.step();
    let zoom = h.state().ui.piano.zoom_x;
    let fitted = piano_geometry(&h).grid.width() / (bar as f32 / PPQ as f32);
    assert!((zoom - fitted).abs() < fitted * 0.1, "{zoom} vs {fitted}");
    let g2 = piano_geometry(&h);
    click(&mut h, g2.grid.center(), PointerButton::Secondary);
    assert!(h.state().ui.piano.zoom_x < zoom * 0.6);
    assert_eq!(pattern_notes(&h, pat, lead).len(), 3, "zooming does not delete");
}

#[test]
fn playlist_time_selection_loops_song_playback() {
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let y = g.grid.top() - 24.0; // the ruler's bar-number half
    // Cmd+drag from bar 2 to bar 4: a time selection snapped to the grid.
    drag_with(
        &mut h,
        egui::pos2(g.pos(bar, 0).x + 1.0, y),
        egui::pos2(g.pos(3 * bar, 0).x + 1.0, y),
        Modifiers::COMMAND,
    );
    assert_eq!(h.state().ui.playlist.time_sel, Some((bar, 3 * bar)));
    assert_eq!(h.state().ui.mode, limeshot_engine::PlayMode::Song);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == bar);
    // Playing never leaves the selection (the silent clock runs in real time; 10 ms blocks).
    h.state_mut().toggle_play();
    let start = std::time::Instant::now();
    let mut seen_wrap = false;
    let mut last = 0;
    while start.elapsed().as_secs_f32() < 4.5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        h.step();
        let pos = h.state().player.status().position();
        assert!((bar..3 * bar).contains(&pos), "{pos} left the selection");
        seen_wrap |= pos < last;
        last = pos;
    }
    assert!(seen_wrap, "playback looped back to the selection's start");
    h.state_mut().stop();
    // A Cmd+click on the ruler clears it.
    click_with(
        &mut h,
        egui::pos2(g.pos(5 * bar, 0).x, y),
        PointerButton::Primary,
        Modifiers::COMMAND,
    );
    assert_eq!(h.state().ui.playlist.time_sel, None);
}

#[test]
fn playlist_markers_add_jump_move_and_delete() {
    let mut h = playlist_harness();
    let bar = h.state().project().time_signature.ticks_per_bar();
    h.state_mut().set_mode(limeshot_engine::PlayMode::Song);
    h.state_mut().locate(2 * bar);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == 2 * bar);
    h.key_press_modifiers(Modifiers::COMMAND, Key::T);
    h.step();
    h.step();
    let m = h.state().project().playlist.markers.clone();
    assert_eq!(m.len(), 1);
    assert_eq!((m[0].tick, m[0].name.as_str()), (2 * bar, "Marker 1"));
    // Alt+T adds one and asks for its name.
    h.key_press_modifiers(Modifiers::ALT, Key::T);
    h.step();
    assert!(matches!(
        h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::Rename {
            target: limeshot_ui::dialogs::RenameTarget::Marker(_),
            ..
        })
    ));
    h.state_mut().dialog = None;
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    // Clicking the flag jumps there; dragging it moves the marker (one undo step).
    h.state_mut().locate(0);
    wait_for(&mut h, "the locate", |a| a.player.status().position() == 0);
    let id = m[0].id;
    let flag = |h: &Harness<'static, LimeShotApp>| {
        h.state()
            .ui
            .playlist
            .marker_rects
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, r)| r.center())
            .expect("marker flag drawn")
    };
    let p = flag(&h);
    click(&mut h, p, PointerButton::Primary);
    wait_for(&mut h, "the jump", |a| a.player.status().position() == 2 * bar);
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let history = h.state().doc.history().len();
    let p = flag(&h);
    drag(&mut h, p, egui::pos2(g.pos(4 * bar, 0).x + 2.0, p.y));
    h.step();
    assert_eq!(h.state().project().playlist.markers[0].tick, 4 * bar);
    assert_eq!(h.state().doc.history().len(), history + 1);
    // Right-click › Delete.
    let p = flag(&h);
    click(&mut h, p, PointerButton::Secondary);
    h.get_by_label("Delete").click();
    h.step();
    assert!(h.state().project().playlist.markers.is_empty());
}
