//! Headless UI tests: the real `LimeShotApp` UI driven with synthetic input (no window, no
//! audio device). Pointer positions refer to the default 1440×900 layout with the demo project;
//! `cargo run -p limeshot-app --example ui_shot -- out.png` renders that layout for reference.

use egui::{Event, Key, Modifiers, PointerButton, Pos2, pos2};
use egui_kittest::Harness;
use limeshot_core::*;
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

fn click(h: &mut Harness<'static, LimeShotApp>, p: Pos2, button: PointerButton) {
    h.event(Event::PointerMoved(p));
    h.step();
    h.event(Event::PointerButton {
        pos: p,
        button,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::PointerButton {
        pos: p,
        button,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.step();
}

#[test]
fn renders_frames_without_panicking() {
    let mut h = harness(true);
    h.state_mut().ui.show_piano = true;
    h.state_mut().ui.show_mixer = true;
    h.state_mut().ui.rack.settings = h.state().project().channels.first().map(|c| c.id);
    for _ in 0..5 {
        h.step();
    }
    let mut h = harness(false);
    h.step();
}

#[test]
fn window_shortcuts_toggle_editors() {
    let mut h = harness(false);
    assert!(!h.state().ui.show_piano);
    h.key_press(Key::F7);
    h.step();
    assert!(h.state().ui.show_piano);
    h.key_press(Key::F9);
    h.step();
    assert!(h.state().ui.show_mixer);
    h.key_press(Key::F5);
    h.step();
    assert!(!h.state().ui.show_playlist);
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
    // Lead row, first step (see the module docs for the reference layout).
    click(&mut h, pos2(518.0, 265.0), PointerButton::Primary);
    assert!(has_step(&h), "left-click turned the step on");
    click(&mut h, pos2(518.0, 265.0), PointerButton::Secondary);
    assert!(!has_step(&h), "right-click turned it off");
    assert!(h.state().doc.is_modified());
}

#[test]
fn clicking_the_playlist_places_the_current_pattern() {
    let mut h = harness(true);
    let before = h.state().project().playlist.clips.len();
    // Track 5, around bar 3 — empty in the demo.
    click(&mut h, pos2(600.0, 610.0), PointerButton::Primary);
    let clips = &h.state().project().playlist.clips;
    assert_eq!(clips.len(), before + 1);
    let c = clips.last().expect("clip").clone();
    assert_eq!(c.track, 4);
    assert_eq!(c.pattern(), h.state().current_pattern());
    assert_eq!(c.start % STEP_TICKS, 0, "snapped to the grid");
    let x = c_x(c.start) + 4.0;
    click(&mut h, pos2(x, 610.0), PointerButton::Secondary);
    assert_eq!(
        h.state().project().playlist.clips.len(),
        before,
        "right-click deletes it"
    );
}

/// Screen x of a tick in the Playlist at its default zoom (24 px per beat, no scroll), using
/// the grid's left edge from the reference layout.
fn c_x(tick: Tick) -> f32 {
    374.0 + tick as f32 * 24.0 / PPQ as f32
}

#[test]
fn piano_roll_click_adds_drag_moves_delete_removes() {
    let mut h = harness(true);
    let lead = h.state().project().channels[3].id;
    h.state_mut().ui.selected_channel = Some(lead);
    h.state_mut().ui.show_piano = true;
    h.state_mut().ui.show_playlist = false;
    h.state_mut().ui.show_rack = false;
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
    // The Piano Roll grid starts at x≈408 at the default zoom of 96 px per beat (24 px per step).
    click(&mut h, pos2(450.0, 330.0), PointerButton::Primary);
    let n = notes(&h);
    assert_eq!(n.len(), 1, "click adds a note");
    assert_eq!(n[0].start, STEP_TICKS, "snapped down to the step grid");
    assert_eq!(n[0].length, STEP_TICKS);
    let key = n[0].key;
    // Drag it two beats to the right.
    let from = pos2(445.0, 330.0);
    let to = pos2(445.0 + 192.0, 330.0);
    h.event(Event::PointerMoved(from));
    h.step();
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    let n = notes(&h);
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].start, STEP_TICKS + 2 * PPQ, "moved by two beats");
    assert_eq!(n[0].key, key, "same pitch");
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
