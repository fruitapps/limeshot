//! Headless UI tests: the real `LimeShotApp` UI driven with synthetic input (no window, no
//! audio device). Pointer positions come from the geometry each editor records while drawing
//! (`channel_rack::State::step_pos`, `piano_roll::Geometry`, `playlist::Geometry`), so the tests
//! follow layout changes.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use limeshot_core::*;
use limeshot_ui::app::DockTab;
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
    for tab in DockTab::ALL {
        h.state_mut().show_tab(tab);
        h.step();
        h.step();
    }
    let c = h.state().project().channels[3].id;
    h.state_mut().select_channel(c);
    h.step();
    h.state_mut().ui.inspect = limeshot_ui::app::InspectTarget::MixerTrack;
    h.step();
    let mut h = harness(false);
    h.step();
}

#[test]
fn window_shortcuts_switch_dock_tabs() {
    let mut h = harness(false);
    assert!(h.state().is_tab_visible(DockTab::ChannelRack));
    h.key_press(Key::F7);
    h.step();
    assert!(h.state().is_tab_visible(DockTab::PianoRoll));
    h.key_press(Key::F9);
    h.step();
    assert!(h.state().is_tab_visible(DockTab::Mixer));
    h.key_press(Key::F9);
    h.step();
    assert!(
        !h.state().ui.dock_open,
        "pressing the visible tab's key again hides the dock"
    );
    h.key_press(Key::F5);
    h.step();
    assert!(
        h.state().ui.dock_open,
        "F5 toggles the Playlist between maximised and docked"
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
    assert!(h.state().doc.is_modified());
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
    let before = h.state().project().playlist.clips.len();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    // Track 5, bar 3 — empty in the demo.
    let bar = h.state().project().time_signature.ticks_per_bar();
    click(&mut h, g.pos(2 * bar + 100, 4), PointerButton::Primary);
    let clips = &h.state().project().playlist.clips;
    assert_eq!(clips.len(), before + 1);
    let c = clips.last().expect("clip").clone();
    assert_eq!(c.track, 4);
    assert_eq!(c.pattern(), h.state().current_pattern());
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
    h.state_mut().show_tab(DockTab::PianoRoll);
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
        (STEP_TICKS, STEP_TICKS, 72),
        "snapped to the step grid"
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
    h.state_mut().show_tab(DockTab::PianoRoll);
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
