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

/// Centre of the picker row of `source`, as last drawn.
fn picker_row(h: &Harness<'static, LimeShotApp>, source: ClipSource) -> Pos2 {
    h.state()
        .ui
        .playlist
        .picker_rows
        .iter()
        .find(|(s, _)| *s == source)
        .map(|(_, r)| r.center())
        .expect("listed in the picker")
}

#[test]
fn picker_selects_the_pattern_to_place() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let melody = h.state().project().patterns[1].id;
    let p = picker_row(&h, ClipSource::Pattern(melody));
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

/// The owner's hands-on test: restoring a maximised window kept it at the workspace's size.
#[test]
fn restoring_a_maximised_window_brings_back_its_old_size() {
    let mut h = harness(true);
    let before = h
        .ctx
        .memory(|m| m.area_rect(Win::ChannelRack.id()))
        .expect("rack drawn");
    // The title bar's buttons from the right: close, maximise, detach (22 px each).
    let maximise = egui::pos2(before.right() - 4.0 - 33.0, before.top() + 14.0);
    click(&mut h, maximise, PointerButton::Primary);
    assert!(h.state().ui.workspace.state(Win::ChannelRack).maximized);
    let max = h
        .ctx
        .memory(|m| m.area_rect(Win::ChannelRack.id()))
        .expect("rack drawn");
    assert!(max.width() > before.width() + 100.0, "maximised: {max:?}");
    let restore = egui::pos2(max.right() - 4.0 - 33.0, max.top() + 14.0);
    click(&mut h, restore, PointerButton::Primary);
    h.step();
    assert!(!h.state().ui.workspace.state(Win::ChannelRack).maximized);
    let after = h
        .ctx
        .memory(|m| m.area_rect(Win::ChannelRack.id()))
        .expect("rack drawn");
    assert!(
        (after.min - before.min).length() < 1.0 && (after.size() - before.size()).length() < 1.0,
        "restored to {after:?}, was {before:?}"
    );
    // Still resizable afterwards (not stuck at a fixed size).
    let edge = after.right_bottom() - vec2(2.0, 2.0);
    drag(&mut h, edge, edge + vec2(60.0, 40.0));
    h.step();
    let resized = h
        .ctx
        .memory(|m| m.area_rect(Win::ChannelRack.id()))
        .expect("rack drawn");
    assert!(resized.width() > after.width() + 30.0, "resized to {resized:?}");
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

/// An empty project in the Playlist, with `n` one-bar clips of the first pattern on track 0
/// from bar 2 on (two bars apart).
fn playlist_tools_harness(n: u64) -> (Harness<'static, LimeShotApp>, Vec<ClipId>) {
    let mut h = harness(false);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::Playlist;
    let pat = h.state().project().patterns[0].id;
    let bar = h.state().project().time_signature.ticks_per_bar();
    let ids = (0..n)
        .map(|i| {
            ClipId(
                h.state_mut()
                    .exec(Command::AddClip {
                        pattern: pat,
                        track: 0,
                        start: bar + 2 * i * bar,
                        length: Some(bar),
                    })
                    .expect("clip"),
            )
        })
        .collect();
    for _ in 0..3 {
        h.step();
    }
    (h, ids)
}

fn clip(h: &Harness<'static, LimeShotApp>, id: ClipId) -> Clip {
    h.state().project().clip(id).expect("clip").clone()
}

#[test]
fn playlist_tool_keys_and_strip() {
    let (mut h, _) = playlist_tools_harness(0);
    use limeshot_ui::piano_roll::Tool;
    for (k, tool) in [
        (Key::B, Tool::Paint),
        (Key::D, Tool::Delete),
        (Key::T, Tool::Mute),
        (Key::S, Tool::Slip),
        (Key::C, Tool::Slice),
        (Key::E, Tool::Select),
        (Key::Z, Tool::Zoom),
        (Key::Y, Tool::Playback),
        (Key::P, Tool::Draw),
    ] {
        h.key_press(k);
        h.step();
        assert_eq!(h.state().ui.playlist.tool, tool, "{k:?}");
    }
    assert_eq!(h.state().ui.piano.tool, Tool::Draw, "the Piano Roll keeps its own tool");
    // The strip's buttons select the tools too.
    for tool in [Tool::Slip, Tool::Slice, Tool::Draw] {
        let r = h
            .state()
            .ui
            .playlist
            .tool_rects
            .iter()
            .find(|x| x.0 == tool)
            .expect("button")
            .1;
        click(&mut h, r.center(), PointerButton::Primary);
        assert_eq!(h.state().ui.playlist.tool, tool);
    }
}

#[test]
fn playlist_slip_tool_moves_content_inside_the_clip() {
    let (mut h, ids) = playlist_tools_harness(1);
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let c = clip(&h, ids[0]);
    h.state_mut().ui.playlist.tool = limeshot_ui::piano_roll::Tool::Slip;
    let history = h.state().doc.history().len();
    // Dragging left by a beat shows later content: the offset grows, the clip stays put.
    let mid = c.start + c.length / 2;
    drag(&mut h, g.pos(mid, 0), g.pos(mid - PPQ, 0));
    h.step();
    let s = clip(&h, ids[0]);
    assert_eq!((s.start, s.length, s.offset), (c.start, c.length, PPQ));
    assert_eq!(h.state().doc.history().len(), history + 1, "one undo step");
    // Dragging right past the source start stops at offset 0.
    drag(&mut h, g.pos(mid, 0), g.pos(mid + 2 * PPQ, 0));
    h.step();
    let s = clip(&h, ids[0]);
    assert_eq!((s.start, s.length, s.offset), (c.start, c.length, 0));
}

#[test]
fn playlist_slice_tool_cuts_the_clips_it_crosses() {
    let (mut h, ids) = playlist_tools_harness(1);
    let bar = h.state().project().time_signature.ticks_per_bar();
    let pat = h.state().project().patterns[0].id;
    let below = ClipId(
        h.state_mut()
            .exec(Command::AddClip {
                pattern: pat,
                track: 1,
                start: bar,
                length: Some(bar),
            })
            .expect("clip"),
    );
    let other = ClipId(
        h.state_mut()
            .exec(Command::AddClip {
                pattern: pat,
                track: 3,
                start: bar,
                length: Some(bar),
            })
            .expect("clip"),
    );
    h.step();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    h.state_mut().ui.playlist.tool = limeshot_ui::piano_roll::Tool::Slice;
    // A vertical drag at beat 2 of the clips, from track 0 down to track 1 (not track 3).
    let at = bar + PPQ;
    drag(&mut h, g.pos(at + 10, 0), g.pos(at + 10, 1));
    h.step();
    for id in [ids[0], below] {
        let c = clip(&h, id);
        assert_eq!((c.start, c.length, c.offset), (bar, PPQ, 0));
    }
    assert_eq!(clip(&h, other).length, bar, "track 3 was not crossed");
    let sel = h.state().ui.playlist.selection.clone();
    assert_eq!(sel.len(), 2, "the right pieces are selected");
    for id in sel {
        let c = clip(&h, id);
        assert_eq!((c.start, c.length, c.offset), (at, bar - PPQ, PPQ));
    }
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(h.state().project().playlist.clips.len(), 3, "one undo step");
}

#[test]
fn playlist_paint_mute_delete_select_zoom_and_playback_tools() {
    use limeshot_ui::piano_roll::Tool;
    let (mut h, ids) = playlist_tools_harness(2);
    let bar = h.state().project().time_signature.ticks_per_bar();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    // Paint: a drag along track 2 lays one-bar clips end to end, as one undo step.
    h.state_mut().ui.playlist.tool = Tool::Paint;
    drag(&mut h, g.pos(10, 2), g.pos(3 * bar + 20, 2));
    h.step();
    let mut painted: Vec<(Tick, Tick)> = h
        .state()
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| c.track == 2)
        .map(|c| (c.start, c.length))
        .collect();
    painted.sort();
    assert_eq!(painted, (0..4).map(|i| (i * bar, bar)).collect::<Vec<_>>());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(h.state().project().playlist.clips.len(), 2);
    // Mute: a click toggles a clip, a drag mutes every clip it crosses.
    h.state_mut().ui.playlist.tool = Tool::Mute;
    let (a, b) = (clip(&h, ids[0]), clip(&h, ids[1]));
    click(&mut h, g.pos(a.start + bar / 2, 0), PointerButton::Primary);
    assert!(clip(&h, a.id).muted);
    click(&mut h, g.pos(a.start + bar / 2, 0), PointerButton::Primary);
    assert!(!clip(&h, a.id).muted);
    drag(&mut h, g.pos(a.start + bar / 2, 0), g.pos(b.start + bar / 2, 0));
    h.step();
    assert!(clip(&h, a.id).muted && clip(&h, b.id).muted);
    // Select: a plain drag selects; a drag on a selected clip moves the selection.
    h.state_mut().ui.playlist.tool = Tool::Select;
    drag(&mut h, g.pos(10, 0), g.pos(b.end() + 10, 1));
    let mut sel = h.state().ui.playlist.selection.clone();
    sel.sort();
    assert_eq!(sel, ids);
    drag(&mut h, g.pos(a.start + bar / 2, 0), g.pos(a.start + bar / 2, 1));
    h.step();
    assert_eq!((clip(&h, a.id).track, clip(&h, b.id).track), (1, 1));
    // Delete: a click deletes the clip under the pointer.
    h.state_mut().ui.playlist.tool = Tool::Delete;
    click(&mut h, g.pos(a.start + bar / 2, 1), PointerButton::Primary);
    assert!(h.state().project().clip(a.id).is_none());
    // Playback scrubs instead of starting the transport (`playback_tool_scrubs_what_is_under_the_line`).
    h.state_mut().ui.playlist.tool = Tool::Playback;
    let p = g.pos(2 * bar, 5);
    h.event(Event::PointerMoved(p));
    h.step();
    press(&mut h, p, PointerButton::Primary, true);
    press(&mut h, p, PointerButton::Primary, false);
    assert!(!h.state().is_playing());
    // Zoom: dragging over a bar fits it to the grid; right-click zooms out.
    h.state_mut().ui.playlist.tool = Tool::Zoom;
    drag(&mut h, g.pos(bar, 4), g.pos(2 * bar, 4));
    h.step();
    let zoom = h.state().ui.playlist.zoom_x;
    let fitted = g.grid.width() / (bar as f32 / PPQ as f32);
    assert!((zoom - fitted).abs() < fitted * 0.1, "{zoom} vs {fitted}");
    let g2 = h.state().ui.playlist.geometry.expect("playlist drawn");
    click(&mut h, g2.grid.center(), PointerButton::Secondary);
    assert!(h.state().ui.playlist.zoom_x < zoom * 0.6);
    assert!(h.state().project().clip(b.id).is_some(), "zooming does not delete");
}

#[test]
fn mixer_eq_graph_and_reset_edit_the_track_eq() {
    let mut h = harness(true);
    h.state_mut().ui.mixer.selected = 1;
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    for _ in 0..3 {
        h.step();
    }
    let eq = |h: &Harness<'static, LimeShotApp>| h.state().project().mixer.tracks[1].eq;
    assert_eq!(eq(&h), EqBand::DEFAULTS);
    let g = h.state().ui.mixer.eq_graph.expect("EQ graph drawn");
    let history = h.state().doc.history().len();
    // Drag the peak's handle right and up: higher frequency, a boost.
    let from = limeshot_ui::mixer::eq_handle(g, EqBand::DEFAULTS[1]);
    let to = egui::pos2(from.x + g.width() * 0.1, g.center().y - g.height() * 0.25);
    drag(&mut h, from, to);
    h.step();
    let b = eq(&h)[1];
    assert!(b.freq > 2500.0 && b.freq < 3500.0, "{b:?}");
    assert!((b.gain - 9.0).abs() < 0.6, "{b:?}");
    assert_eq!((eq(&h)[0], eq(&h)[2]), (EqBand::DEFAULTS[0], EqBand::DEFAULTS[2]));
    assert_eq!(h.state().doc.history().len(), history + 1, "one undo step");
    // Right-drag left/right changes the width of the nearest band (the low shelf here).
    let low = limeshot_ui::mixer::eq_handle(g, EqBand::DEFAULTS[0]);
    h.event(Event::PointerMoved(low));
    h.step();
    press(&mut h, low, PointerButton::Secondary, true);
    for k in 1..=8 {
        h.event(Event::PointerMoved(low + vec2(g.width() * 0.04 * k as f32, 0.0)));
        h.step();
    }
    press(
        &mut h,
        low + vec2(g.width() * 0.32, 0.0),
        PointerButton::Secondary,
        false,
    );
    h.step();
    let l = eq(&h)[0];
    assert!(l.width > 0.5, "{l:?}");
    assert_eq!(l.freq, EqBand::DEFAULTS[0].freq, "right-drag keeps the frequency");
    // Reset puts every band back, as one undo step.
    h.get_by_label("Reset").click();
    h.step();
    assert_eq!(eq(&h), EqBand::DEFAULTS);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(eq(&h)[0], l);
}

#[test]
fn mixer_sidechain_route_menu_and_compressor_input() {
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
    let routes = |h: &Harness<'static, LimeShotApp>, t: usize| {
        let mut r: Vec<(u16, f32)> = h.state().project().mixer.tracks[t]
            .routes
            .iter()
            .map(|r| (r.to, r.level))
            .collect();
        r.sort_by_key(|x| x.0);
        r
    };
    // Right-click the bass strip's route button: FL Studio's menu. Sidechain keeps the master.
    let p = button(&h, 5);
    click(&mut h, p, PointerButton::Secondary);
    h.get_by_label("Sidechain to this track").click();
    h.step();
    assert_eq!(routes(&h, 1), vec![(0, 1.0), (5, 0.0)]);
    // "… only" drops the other routes, as one undo step.
    let p = button(&h, 6);
    click(&mut h, p, PointerButton::Secondary);
    h.get_by_label("Route to this track only").click();
    h.step();
    assert_eq!(routes(&h, 1), vec![(6, 1.0)]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(routes(&h, 1), vec![(0, 1.0), (5, 0.0)]);
    // On the bass, a compressor picks the kick as its sidechain input.
    let fx = h
        .state_mut()
        .exec(Command::AddEffect {
            track: 5,
            kind: EffectKind::Compressor,
        })
        .expect("effect");
    h.state_mut().ui.mixer.selected = 5;
    h.step();
    h.step();
    assert!(
        h.query_by_label_contains("SC← Kick").is_some(),
        "the input list shows the sidechain"
    );
    h.get_by_value("Off (own input)").click();
    h.step();
    h.step();
    h.step();
    // egui keeps a new popup's items disabled for its first frame.
    let item = h.get_by_label("1. Kick").rect().center();
    click(&mut h, item, PointerButton::Primary);
    let param = |h: &Harness<'static, LimeShotApp>| {
        h.state().project().mixer.tracks[5]
            .effects
            .iter()
            .find(|e| e.id == fx)
            .expect("fx")
            .params[COMPRESSOR_SIDECHAIN]
    };
    assert_eq!(param(&h), 1.0);
    // Playing the beat: the kick drives the bass compressor's gain reduction meter.
    h.state_mut().toggle_play();
    wait_for(&mut h, "gain reduction from the kick", |a| {
        a.player.status().gain_reduction(5) > 1.0
    });
    h.state_mut().stop();
}

#[test]
fn channel_menu_fills_and_shortcuts_rotate_steps() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let (kick, hat) = (h.state().project().channels[0].id, h.state().project().channels[2].id);
    let steps = |h: &Harness<'static, LimeShotApp>, c: ChannelId| -> Vec<Tick> {
        let p = h.state().project().pattern(pat).expect("pattern");
        p.notes_for(c).map(|n| n.start / STEP_TICKS).collect()
    };
    // Right-click the kick's channel button: Fill each 4 steps replaces its steps.
    let button = h.state().ui.rack.step_pos(kick, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Secondary);
    h.get_by_label("Fill each 4 steps").click();
    h.step();
    assert_eq!(steps(&h, kick), vec![0, 4, 8, 12]);
    assert_eq!(h.state().selected_channels(), vec![kick]);
    // Shift+Cmd+→ rotates the selected channels by a step; ← wraps round the pattern.
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::ArrowRight);
    h.step();
    assert_eq!(steps(&h, kick), vec![1, 5, 9, 13]);
    let hat_before = steps(&h, hat);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::ArrowLeft);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::ArrowLeft);
    h.step();
    assert_eq!(steps(&h, kick), vec![3, 7, 11, 15]);
    assert_eq!(steps(&h, hat), hat_before, "only the selection rotates");
    // The menu's Rotate right does the same; each is one undo step.
    let button = h.state().ui.rack.step_pos(kick, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Secondary);
    h.get_by_label_contains("Rotate right").click();
    h.step();
    assert_eq!(steps(&h, kick), vec![0, 4, 8, 12]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(steps(&h, kick), vec![3, 7, 11, 15]);
}

#[test]
fn piano_roll_rotates_notes_with_shift_cmd_arrows() {
    let (mut h, pat, lead) = empty_piano_roll();
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: [0, 3 * PPQ]
            .into_iter()
            .map(|start| NewNote {
                channel: lead,
                key: 72,
                start,
                length: STEP_TICKS,
                velocity: 0.8,
            })
            .collect(),
    });
    h.step();
    // Nothing selected: every note of the channel rotates left by the snap (a step), wrapping.
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::ArrowLeft);
    h.step();
    let starts: Vec<Tick> = notes_by_start(&h, pat, lead).iter().map(|n| n.1).collect();
    let bar = h.state().project().time_signature.ticks_per_bar();
    assert_eq!(starts, vec![3 * PPQ - STEP_TICKS, bar - STEP_TICKS]);
}

#[test]
fn graph_editor_draws_step_velocity_and_pan() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let kick = h.state().project().channels[0].id;
    let notes = |h: &Harness<'static, LimeShotApp>| -> Vec<(Tick, f32, f32)> {
        let p = h.state().project().pattern(pat).expect("pattern");
        p.notes_for(kick)
            .map(|n| (n.start / STEP_TICKS, n.velocity, n.pan))
            .collect()
    };
    assert_eq!(notes(&h).iter().map(|n| n.0).collect::<Vec<_>>(), vec![0, 4, 8, 12]);
    // The channel menu opens the graph editor on that channel.
    let button = h.state().ui.rack.step_pos(kick, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Secondary);
    h.get_by_label("Graph editor").click();
    h.step();
    h.step();
    assert_eq!(h.state().ui.rack.graph, Some(NoteProp::Velocity));
    let g = h.state().ui.rack.graph_rect.expect("graph drawn");
    // An overlay right under the channel's steps, aligned with them, over the channels below
    // (owner's list item 9, as in FL Studio 2026.1.1).
    let strip = h
        .state()
        .ui
        .rack
        .step_strips
        .iter()
        .find(|s| s.0 == kick)
        .expect("strip")
        .1;
    assert!((g.left() - strip.left()).abs() < 0.5 && (g.right() - strip.right()).abs() < 0.5);
    assert!(
        g.top() > strip.bottom() && g.top() - strip.bottom() < 10.0,
        "{g:?} under {strip:?}"
    );
    let clap = h.state().project().channels[1].id;
    let below = h
        .state()
        .ui
        .rack
        .step_strips
        .iter()
        .find(|s| s.0 == clap)
        .expect("strip")
        .1;
    assert!(g.intersects(below), "covers the next channel");
    let clap_steps = h
        .state()
        .project()
        .pattern(pat)
        .expect("pattern")
        .notes_for(clap)
        .count();
    let x = |step: f32| g.left() + (step + 0.5) * 22.0;
    let history = h.state().doc.history().len();
    // A drag from step 1 near the top to step 13 near the bottom draws a falling line.
    drag(
        &mut h,
        egui::pos2(x(0.0), g.top() + 1.0),
        egui::pos2(x(12.0), g.bottom() - g.height() * 0.25),
    );
    h.step();
    let v: Vec<f32> = notes(&h).iter().map(|n| n.1).collect();
    assert!(v[0] > 0.95 && (v[3] - 0.25).abs() < 0.05, "{v:?}");
    assert!(v[0] > v[1] && v[1] > v[2] && v[2] > v[3], "{v:?}");
    assert_eq!(h.state().doc.history().len(), history + 1, "one undo step");
    // The Pan lane: a click high on step 5 pans that step right.
    h.get_by_label("Pan").click();
    h.step();
    click(&mut h, egui::pos2(x(4.0), g.top() + 2.0), PointerButton::Primary);
    let n = notes(&h);
    assert!(n[1].2 > 0.9 && n[0].2 == 0.0, "{n:?}");
    let p = h.state().project().pattern(pat).expect("pattern");
    assert_eq!(
        p.notes_for(clap).count(),
        clap_steps,
        "the steps under the overlay were not touched"
    );
    // The header button closes it.
    h.state_mut().ui.rack.graph = None;
    h.step();
    assert!(h.state().ui.rack.graph_rect.is_none());
}

#[test]
fn channel_filter_groups_hide_other_channels() {
    let mut h = harness(true);
    let (kick, clap) = (h.state().project().channels[0].id, h.state().project().channels[1].id);
    let shown = |h: &Harness<'static, LimeShotApp>| -> Vec<ChannelId> {
        h.state().ui.rack.step_strips.iter().map(|s| s.0).collect()
    };
    assert_eq!(shown(&h).len(), 5);
    // Opt+G with the kick selected asks for a group name; Enter files it and shows that group.
    h.state_mut().select_channel(kick);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::G);
    h.step();
    h.step();
    h.event(Event::Text("Drums".into()));
    h.step();
    h.key_press(Key::Enter);
    h.step();
    h.step();
    assert_eq!(
        h.state().project().channel(kick).expect("kick").group.as_deref(),
        Some("Drums")
    );
    assert_eq!(
        h.state().ui.rack.filter,
        limeshot_ui::channel_rack::RackFilter::Group("Drums".into())
    );
    assert_eq!(shown(&h), vec![kick]);
    // The dropdown switches to Unsorted (everything else), then back to All.
    h.get_by_value("Drums").click();
    h.step();
    h.step();
    h.get_by_label("Unsorted").click();
    h.step();
    h.step();
    assert!(!shown(&h).contains(&kick) && shown(&h).contains(&clap));
    h.get_by_value("Unsorted").click();
    h.step();
    h.step();
    h.get_by_label("All").click();
    h.step();
    h.step();
    assert_eq!(shown(&h).len(), 5);
}

/// The demo with a maximised Mixer and insert `sel` selected.
fn mixer_harness(sel: u16) -> Harness<'static, LimeShotApp> {
    let mut h = harness(true);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    h.state_mut().ui.mixer.selected = sel;
    h.step();
    h.step();
    h
}

/// Owner's hands-on test: Mixer names are typed where they sit, not in a rename popup.
#[test]
fn mixer_names_are_typed_in_place() {
    let mut h = mixer_harness(1);
    let name = |h: &Harness<'static, LimeShotApp>, i: usize| h.state().project().mixer.tracks[i].name.clone();
    let r = h
        .state()
        .ui
        .mixer
        .name_rects
        .iter()
        .find(|x| x.0 == 2)
        .expect("strip name")
        .1;
    double_click(&mut h, r.center());
    assert!(h.state().dialog.is_none(), "no rename popup");
    assert_eq!(h.state().ui.mixer.selected, 2);
    // The old name is selected: typing replaces it, Enter keeps it.
    h.event(Event::Text("Snare".into()));
    h.step();
    h.key_press(Key::Enter);
    h.step();
    h.step();
    assert_eq!(name(&h, 2), "Snare");
    assert!(h.state().ui.mixer.rename.is_none());
    // The track panel's name: click, type, Esc cancels.
    let r = h.state().ui.mixer.panel_name.expect("panel name");
    click(&mut h, r.center(), PointerButton::Primary);
    h.step();
    assert!(h.state().ui.mixer.rename.as_ref().is_some_and(|r| r.in_panel));
    h.event(Event::Text("Nope".into()));
    h.step();
    h.key_press(Key::Escape);
    h.step();
    h.step();
    assert_eq!(name(&h, 2), "Snare");
    assert!(h.state().ui.mixer.rename.is_none());
}

/// Owner's hands-on test: the track panel's Output offers sidechain and "only" routing too.
#[test]
fn mixer_output_menu_has_route_and_sidechain_entries() {
    let mut h = mixer_harness(1);
    let routes = |h: &Harness<'static, LimeShotApp>| {
        let mut r: Vec<(u16, f32)> = h.state().project().mixer.tracks[1]
            .routes
            .iter()
            .map(|r| (r.to, r.level))
            .collect();
        r.sort_by_key(|x| x.0);
        r
    };
    assert_eq!(routes(&h), vec![(0, 1.0)]);
    h.get_by_label("+ Route to…").click();
    h.step();
    h.step();
    h.get_by_label("5  Bass ⏵").hover();
    h.step();
    h.step();
    h.step();
    h.get_by_label("Sidechain to this track only").click();
    h.step();
    assert_eq!(routes(&h), vec![(5, 0.0)]);
}

/// Owner's hands-on test: the send knob was at its end at 0 dB and went past it above.
#[test]
fn send_knob_covers_the_send_range() {
    use limeshot_ui::mixer::gain_pos;
    assert!((gain_pos(1.0) - 0.7937).abs() < 1e-3, "0 dB at about 80 %");
    assert_eq!(gain_pos(2.0), 1.0);
    assert_eq!(gain_pos(4.0), 1.0);
    let mut h = mixer_harness(1);
    h.state_mut().exec(Command::SetRoute {
        from: 1,
        to: 6,
        level: Some(1.0),
    });
    h.step();
    h.step();
    let level = |h: &Harness<'static, LimeShotApp>| {
        h.state().project().mixer.tracks[1]
            .routes
            .iter()
            .find(|r| r.to == 6)
            .map(|r| r.level)
    };
    let k = h
        .state()
        .ui
        .mixer
        .send_knobs
        .iter()
        .find(|x| x.0 == 6)
        .expect("send knob")
        .1;
    drag(&mut h, k.center(), k.center() - vec2(0.0, 400.0));
    let up = level(&h).expect("routed");
    assert!((up - 2.0).abs() < 1e-3, "dragged to +6 dB: {up}");
    double_click(&mut h, k.center());
    assert_eq!(level(&h), Some(1.0), "double-click resets to 0 dB");
}

/// Owner's hands-on test: double-click resets every slider and knob, and inserts default to 0 dB.
#[test]
fn double_click_resets_sliders_and_knobs() {
    assert_eq!(MixerTrack::new("x").volume, 1.0, "FL Studio's insert default is 0 dB");
    let mut h = mixer_harness(1);
    // Track panel volume slider.
    h.state_mut().exec(Command::SetMixerVolume { track: 1, volume: 0.3 });
    h.step();
    let r = h
        .get_all_by_label("Volume")
        .map(|n| n.rect())
        .max_by(|a, b| a.width().total_cmp(&b.width()))
        .expect("volume slider");
    double_click(&mut h, egui::pos2(r.left() + 20.0, r.center().y));
    assert_eq!(h.state().project().mixer.tracks[1].volume, 1.0);
    // An effect card's slider.
    let fx = h
        .state_mut()
        .exec(Command::AddEffect {
            track: 1,
            kind: EffectKind::Delay,
        })
        .expect("effect");
    h.state_mut().exec(Command::SetEffectParam {
        track: 1,
        effect: fx,
        index: 0,
        value: 0.9,
    });
    h.step();
    h.step();
    let spec = EffectKind::Delay.params()[0];
    let label = h.get_by_label(spec.name).rect();
    double_click(&mut h, egui::pos2(label.right() + 30.0, label.center().y));
    let v = h.state().project().mixer.tracks[1].effects[0].params[0];
    assert!(
        (v - spec.default).abs() < 1e-4,
        "{} back to {}: {v}",
        spec.name,
        spec.default
    );
    // Channel Settings: a LimeSynth slider goes back to the synth's default.
    let lead = h.state().project().channels[3].id;
    let mut synth = match &h.state().project().channels[3].instrument {
        Instrument::Synth(p) => p.clone(),
        _ => panic!("the demo's lead is a synth"),
    };
    synth.attack_ms = 900.0;
    h.state_mut().exec(Command::SetInstrument {
        channel: lead,
        instrument: Instrument::Synth(synth),
    });
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = false;
    h.state_mut().ui.workspace.close(Win::Mixer);
    h.state_mut().select_channel(lead);
    h.state_mut().show_window(Win::ChannelSettings);
    h.step();
    h.step();
    let label = h.get_by_label("Attack").rect();
    double_click(&mut h, egui::pos2(label.right() + 30.0, label.center().y));
    let attack = match &h.state().project().channels[3].instrument {
        Instrument::Synth(p) => p.attack_ms,
        _ => 0.0,
    };
    assert_eq!(attack, SynthParams::default().attack_ms);
}

/// Owner's hands-on test: shrinking the Channel Rack hid steps and channels, shrinking the Mixer
/// hid part of the track panel's EQ.
#[test]
fn rack_and_mixer_windows_keep_their_content_visible() {
    let mut h = harness(true);
    let shrink = |h: &mut Harness<'static, LimeShotApp>, w: Win| {
        let r = h.ctx.memory(|m| m.area_rect(w.id())).expect("window drawn");
        let corner = r.right_bottom() - vec2(2.0, 2.0);
        drag(h, corner, r.left_top() + vec2(40.0, 40.0));
        h.step();
        h.step();
        h.ctx.memory(|m| m.area_rect(w.id())).expect("window drawn")
    };
    let rack = shrink(&mut h, Win::ChannelRack);
    let strips = h.state().ui.rack.step_strips.clone();
    assert_eq!(strips.len(), 5, "the demo's five channels");
    for (_, s, _) in &strips {
        assert!(rack.contains_rect(*s), "{s:?} outside the rack {rack:?}");
    }
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().ui.mixer.selected = 1;
    h.state_mut().show_window(Win::Mixer);
    h.step();
    h.step();
    let mixer = shrink(&mut h, Win::Mixer);
    assert!(mixer.width() >= Win::Mixer.min_size().x - 1.0, "{mixer:?}");
    let g = h.state().ui.mixer.eq_graph;
    // The EQ sits at the bottom of the panel's scroll area; its width always fits.
    if let Some(g) = g {
        assert!(g.right() <= mixer.right(), "EQ {g:?} cut off in {mixer:?}");
    }
    let routes = h.state().ui.mixer.route_buttons.clone();
    for (_, b) in routes.iter().filter(|(_, b)| b.left() < mixer.right() - 300.0) {
        assert!(b.bottom() <= mixer.bottom(), "route button {b:?} below {mixer:?}");
    }
}

/// Owner's hands-on test: dragging a note's right edge snapped it to its smallest size.
#[test]
fn piano_roll_right_edge_resizes_both_ways() {
    let (mut h, pat, lead) = empty_piano_roll();
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: vec![NewNote {
            channel: lead,
            key: 72,
            start: 0,
            length: 4 * STEP_TICKS,
            velocity: 0.8,
        }],
    });
    h.step();
    h.step();
    let g = piano_geometry(&h);
    let len = |h: &Harness<'static, LimeShotApp>| pattern_notes(h, pat, lead)[0].length;
    let edge = g.pos(4 * STEP_TICKS, 72) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(7 * STEP_TICKS, 72));
    assert_eq!(len(&h), 7 * STEP_TICKS, "longer");
    let edge = g.pos(7 * STEP_TICKS, 72) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(2 * STEP_TICKS, 72));
    assert_eq!(len(&h), 2 * STEP_TICKS, "shorter");
    let edge = g.pos(2 * STEP_TICKS, 72) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(5 * STEP_TICKS, 72));
    assert_eq!(len(&h), 5 * STEP_TICKS, "longer again");
    // Past the note's start it stops at one snap step instead of a one-tick sliver, and the
    // same drag can bring it back.
    let edge = g.pos(5 * STEP_TICKS, 72) - vec2(2.0, 0.0);
    h.event(Event::PointerMoved(edge));
    h.step();
    press(&mut h, edge, PointerButton::Primary, true);
    for x in [g.pos(2 * STEP_TICKS, 72), g.pos(0, 72) - vec2(80.0, 0.0)] {
        h.event(Event::PointerMoved(x));
        h.step();
    }
    assert_eq!(len(&h), STEP_TICKS, "at least one step");
    let back = g.pos(3 * STEP_TICKS, 72);
    h.event(Event::PointerMoved(back));
    h.step();
    press(&mut h, back, PointerButton::Primary, false);
    assert_eq!(len(&h), 3 * STEP_TICKS, "follows the pointer back");
    assert_eq!(pattern_notes(&h, pat, lead)[0].start, 0);
}

/// Send a wheel turn over `p` with `mods` held, and let egui's smooth scrolling run out.
fn wheel_at(h: &mut Harness<'static, LimeShotApp>, p: Pos2, dy: f32, mods: Modifiers) {
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
}

/// Owner's hands-on test: there was no vertical zoom. Opt+wheel (or Opt+pinch) zooms the key
/// height in the Piano Roll and the track height in the Playlist, as Alt+wheel does in FL Studio.
#[test]
fn alt_wheel_zooms_vertically() {
    let (mut h, _, _) = melody_piano_roll();
    let p = piano_geometry(&h).grid.center();
    let (kh, sy, zx) = (
        h.state().ui.piano.key_h,
        h.state().ui.piano.scroll.y,
        h.state().ui.piano.zoom_x,
    );
    wheel_at(&mut h, p, 60.0, Modifiers::ALT);
    assert!(h.state().ui.piano.key_h > kh, "taller keys");
    assert_eq!(h.state().ui.piano.zoom_x, zx, "time zoom unchanged");
    assert_ne!(h.state().ui.piano.scroll.y, sy, "zoomed around the pointer");
    wheel_at(&mut h, p, -400.0, Modifiers::ALT);
    assert_eq!(
        h.state().ui.piano.key_h,
        limeshot_ui::piano_roll::KEY_H.0,
        "down to the smallest"
    );
    // A pinch with Opt held zooms vertically too.
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.event(Event::Zoom(1.5));
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.step();
    assert!(h.state().ui.piano.key_h > limeshot_ui::piano_roll::KEY_H.0);
    assert_eq!(h.state().ui.piano.zoom_x, zx);

    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let th = h.state().ui.playlist.track_h;
    wheel_at(&mut h, g.grid.center(), 60.0, Modifiers::ALT);
    assert!(h.state().ui.playlist.track_h > th, "taller tracks");
    wheel_at(&mut h, g.grid.center(), -60.0, Modifiers::ALT);
    wheel_at(&mut h, g.grid.center(), -60.0, Modifiers::ALT);
    assert!(h.state().ui.playlist.track_h < th, "shorter tracks");
}

/// Owner's question: does the Snap change the grid? It did in the Piano Roll (once the lines
/// are 6 px apart); the Playlist's grid was fixed to bars and now follows the snap too.
#[test]
fn the_grid_follows_the_snap() {
    use limeshot_ui::piano_roll::grid_step;
    let sig = TimeSignature::default();
    let ppt = 96.0 / PPQ as f32; // the Piano Roll's default zoom: 24 px per step
    assert_eq!(grid_step(ppt, sig, Snap::Step), STEP_TICKS);
    assert_eq!(grid_step(ppt, sig, Snap::HalfStep), STEP_TICKS / 2);
    assert_eq!(grid_step(ppt, sig, Snap::QuarterStep), STEP_TICKS / 4);
    // Owner's list (session 11): a 1/3-step (triplet) snap.
    assert_eq!(grid_step(ppt, sig, Snap::ThirdStep), STEP_TICKS / 3);
    // Coarser snaps keep the zoom's lines; too-dense lines are left out.
    assert_eq!(grid_step(ppt, sig, Snap::Bar), STEP_TICKS);
    assert_eq!(grid_step(ppt / 8.0, sig, Snap::QuarterStep), sig.ticks_per_bar());
    // At a zoom that shows beats (30 px per beat), a step snap adds the step lines.
    let pl = 30.0 / PPQ as f32;
    assert_eq!(grid_step(pl, sig, Snap::Beat), sig.ticks_per_beat());
    assert_eq!(grid_step(pl, sig, Snap::Step), STEP_TICKS);
}

/// Owner's hands-on test: the Playback tool (Y) should scrub, sounding only what is under a line
/// at the pointer, instead of playing on from there.
#[test]
fn playback_tool_scrubs_what_is_under_the_line() {
    use limeshot_ui::piano_roll::Tool;
    let (mut h, pat, lead) = melody_piano_roll();
    let notes = pattern_notes(&h, pat, lead);
    let g = piano_geometry(&h);
    h.state_mut().ui.piano.tool = Tool::Playback;
    let (a, b) = (notes[0], notes[1]);
    let at = |n: &Note| g.pos(n.start + n.length / 2, a.key.saturating_sub(5));
    h.event(Event::PointerMoved(at(&a)));
    h.step();
    assert!(h.state().ui.scrub.is_empty(), "hovering only shows the line");
    assert_eq!(h.state().ui.piano.cursor, Some(Tool::Playback), "the tool's cursor");
    press(&mut h, at(&a), PointerButton::Primary, true);
    assert_eq!(h.state().ui.scrub, vec![(lead, a.key)]);
    // Owner's list item 17: the sounding notes light up.
    assert_eq!(h.state().ui.piano.sounding, vec![a.id]);
    h.event(Event::PointerMoved(at(&b)));
    h.step();
    assert_eq!(
        h.state().ui.scrub,
        vec![(lead, b.key)],
        "the line moved on to the next note"
    );
    assert_eq!(h.state().ui.piano.sounding, vec![b.id]);
    press(&mut h, at(&b), PointerButton::Primary, false);
    assert!(h.state().ui.scrub.is_empty(), "releasing stops the notes");
    assert!(h.state().ui.piano.sounding.is_empty());
    assert!(!h.state().is_playing(), "the transport never started");
    assert!(!h.state().doc.is_modified());

    // The Playlist: the pattern clips' notes under the line.
    let mut h = playlist_harness();
    h.state_mut().ui.playlist.tool = Tool::Playback;
    h.step();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let p = g.pos(10, 0);
    h.event(Event::PointerMoved(p));
    h.step();
    assert_eq!(h.state().ui.playlist.cursor, Some(Tool::Playback));
    press(&mut h, p, PointerButton::Primary, true);
    let kick = h.state().project().channels[0].id;
    assert!(
        h.state().ui.scrub.iter().any(|s| s.0 == kick),
        "the beat's first kick sounds: {:?}",
        h.state().ui.scrub
    );
    press(&mut h, p, PointerButton::Primary, false);
    assert!(h.state().ui.scrub.is_empty());
    assert!(!h.state().is_playing());
}

/// Owner's hands-on test: each tool has its icon as the cursor over the grid.
#[test]
fn editor_tools_show_their_cursor() {
    use limeshot_ui::piano_roll::Tool;
    let (mut h, _, _) = melody_piano_roll();
    let p = piano_geometry(&h).grid.center();
    for tool in Tool::ALL {
        h.state_mut().ui.piano.tool = tool;
        h.event(Event::PointerMoved(p));
        h.step();
        assert_eq!(h.state().ui.piano.cursor, Some(tool));
        // Owner's list item 13: the tool replaces the system cursor.
        assert_eq!(
            h.output().platform_output.cursor_icon,
            egui::CursorIcon::None,
            "{tool:?}"
        );
    }
    h.event(Event::PointerMoved(egui::pos2(2.0, 2.0)));
    h.step();
    assert_eq!(h.state().ui.piano.cursor, None, "not outside the grid");
    assert_ne!(h.output().platform_output.cursor_icon, egui::CursorIcon::None);
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    for tool in Tool::PLAYLIST {
        h.state_mut().ui.playlist.tool = tool;
        h.event(Event::PointerMoved(g.grid.center()));
        h.step();
        assert_eq!(h.state().ui.playlist.cursor, Some(tool));
    }
}

/// Owner's hands-on test: pattern colours could not be changed. The Patterns menu (and the
/// Playlist's pattern picker and clip menus) has a Color swatch picker.
#[test]
fn pattern_color_is_picked_from_swatches() {
    let mut h = harness(true);
    let pat = h.state().current_pattern();
    let color = |h: &Harness<'static, LimeShotApp>| h.state().project().pattern(pat).expect("pattern").color;
    let before = color(&h);
    h.get_by_label("Patterns").click();
    h.step();
    h.step();
    h.get_by_label("Color ⏵").hover();
    h.step();
    h.step();
    h.step();
    let k = PALETTE.iter().position(|c| *c != before).expect("another colour");
    h.get_by_label(&format!("Color {}", k + 1)).click();
    h.step();
    assert_eq!(color(&h), PALETTE[k]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    assert_eq!(color(&h), before, "one undo step");
}

/// Owner's hands-on test: the CPU readout changes at most once per second, showing the highest
/// load of that second.
#[test]
fn cpu_readout_updates_once_per_second() {
    let mut cpu = limeshot_ui::toolbar::CpuReadout::default();
    assert_eq!(cpu.update(10.0, 0.2), 0.2, "the first frame shows the load");
    assert_eq!(cpu.update(10.3, 0.6), 0.2, "held within the second");
    assert_eq!(cpu.update(10.7, 0.3), 0.2);
    assert_eq!(cpu.update(11.0, 0.1), 0.6, "a second later: that second's peak");
    assert_eq!(cpu.update(11.5, 0.05), 0.6);
    assert_eq!(cpu.update(12.0, 0.05), 0.05);
}

/// Open `path` of nested menu labels with clicks, stepping between them. A sub-menu button's
/// label ends in egui's arrow ("LimeShot ⏵"), which also keeps it apart from other text.
fn menu_path(h: &mut Harness<'static, LimeShotApp>, path: &[&str]) {
    for label in path {
        let sub = format!("{label} ⏵");
        let node = match h.query_by_label(&sub) {
            Some(n) => n,
            None => h.get_all_by_label(label).last().expect("menu entry"),
        };
        node.click();
        h.step();
        h.step();
    }
}

#[test]
fn a_plugin_instrument_from_the_add_menu_plays_saves_and_reopens() {
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").clone();
    assert_eq!(ch.name, "Lime Test Tone");
    assert!(matches!(&ch.instrument, Instrument::Plugin(p) if p.class_id == limeshot_plugins::testing::tone_class()));
    let target = Target::Channel(ch.id);
    assert!(
        h.state().plugins.is_loaded(target),
        "created on the main thread when added"
    );
    assert_eq!(h.state().selected_channel(), Some(ch.id));

    // Channel Settings lists the plugin's parameters.
    h.state_mut().show_window(Win::ChannelSettings);
    h.step();
    h.step();
    assert!(h.query_by_label("Level").is_some(), "the plugin's parameter is shown");
    assert!(h.query_all_by_label("Lime Test Tone").next().is_some());

    // A parameter change is not an undo step but marks the project modified, and saving stores
    // the plugin's state.
    h.state_mut().doc.mark_saved();
    let undo = h.state().doc.undo_label();
    h.state_mut().plugins.set_param(target, 0, 0.25);
    h.state_mut().doc.mark_unsaved();
    assert_eq!(h.state().doc.undo_label(), undo);
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("plugin.lime");
    assert!(h.state_mut().save_to(&path));
    let Instrument::Plugin(saved) = &h.state().project().channel(ch.id).expect("ch").instrument else {
        panic!("not a plugin")
    };
    assert!(!saved.state.is_empty(), "state captured at save");

    // Reopened, the plugin comes back with that state.
    let mut h2 = harness(false);
    h2.state_mut().open_path(&path);
    h2.step();
    h2.step();
    let inst = h2.state().plugins.instance(target).expect("reloaded");
    assert!((inst.param(0) - 0.25).abs() < 1e-9, "level restored: {}", inst.param(0));

    // Notes on the channel render through a fresh offline instance at that level.
    let pat = h2.state().current_pattern();
    h2.state_mut().exec(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel: ch.id,
            key: 69,
            start: 0,
            length: 960,
            velocity: 1.0,
        },
    });
    h2.step();
    let (processors, _keep) = h2.state().plugins.offline(h2.state().project(), 48_000);
    assert_eq!(processors.instruments.len(), 1);
    let snap = h2.state().snapshot().clone();
    let settings = limeshot_engine::RenderSettings {
        sample_rate: 48_000,
        range: limeshot_engine::ExportRange::Pattern,
        max_tail_secs: 0.0,
        block: 512,
    };
    let (audio, report) = limeshot_engine::render_offline_with(snap, &settings, processors).expect("render");
    assert!(
        report.peak > 0.05 && report.peak < 0.3,
        "level 0.25 through the mix: {}",
        report.peak
    );
    assert!(audio.iter().all(|s| s.is_finite()));
}

#[test]
fn a_plugin_effect_from_the_insert_menu_loads_and_unloads() {
    use limeshot_ui::plugins::Target;
    let mut h = harness(true);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    h.state_mut().ui.mixer.selected = 1;
    for _ in 0..3 {
        h.step();
    }
    let before = h.state().project().mixer.tracks[1].effects.len();
    menu_path(&mut h, &["+ Add effect", "Plugins", "LimeShot", "Lime Test Gain"]);
    let fx = h.state().project().mixer.tracks[1]
        .effects
        .last()
        .expect("slot")
        .clone();
    assert_eq!(h.state().project().mixer.tracks[1].effects.len(), before + 1);
    assert_eq!(fx.kind, EffectKind::Plugin);
    assert_eq!(fx.plugin.as_ref().map(|p| p.name.as_str()), Some("Lime Test Gain"));
    h.step();
    let target = Target::Effect {
        track: 1,
        effect: fx.id,
    };
    assert!(h.state().plugins.is_loaded(target));
    assert!(
        h.query_by_label("Gain").is_some(),
        "the card lists the plugin's parameter"
    );
    assert!(h.query_all_by_label_contains("Lime Test Gain").next().is_some());

    h.state_mut().exec(Command::RemoveEffect {
        track: 1,
        effect: fx.id,
    });
    h.step();
    assert!(
        !h.state().plugins.is_loaded(target),
        "removing the slot drops the plugin"
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.step();
    h.step();
    assert!(h.state().plugins.is_loaded(target), "undo brings it back");
}

/// A Playlist track without clips in the demo (from `from` on).
fn empty_track(h: &Harness<'static, LimeShotApp>, from: u32) -> u32 {
    let p = h.state().project();
    (from..p.playlist.tracks.len() as u32)
        .find(|t| p.playlist.clips.iter().all(|c| c.track != *t))
        .expect("an empty track")
}

fn last_clip(h: &Harness<'static, LimeShotApp>) -> Clip {
    h.state().project().playlist.clips.last().expect("a clip").clone()
}

/// The length of `path` at the project tempo, in ticks.
fn sample_len(h: &Harness<'static, LimeShotApp>, path: &std::path::Path) -> Tick {
    let secs = limeshot_dsp::load_wav(path).expect("wav").duration_secs();
    (secs * h.state().project().tempo.bpm() / 60.0 * PPQ as f64).round() as Tick
}

/// Owner's list item 1 (observed in FL Studio 2026.1.1): the picker has Patterns, Audio and
/// Automation tabs; a sample added there (or dropped on the Playlist) is a sampler channel with
/// the sample loaded, and the Draw tool places it like a pattern.
#[test]
fn samples_are_picked_and_drawn_in_the_playlist_like_patterns() {
    use limeshot_ui::playlist::PickerTab;
    let mut h = playlist_harness();
    let kick = bundled("Lime Kick.wav");
    let n = h.state().project().channels.len();
    let ch = limeshot_ui::playlist::add_audio_source(h.state_mut(), &kick).expect("added");
    h.step();
    assert_eq!(h.state().project().channels.len(), n + 1);
    match &h.state().project().channel(ch).expect("channel").instrument {
        Instrument::Sampler(s) => assert_eq!(
            s.sample.as_ref().map(|r| std::path::PathBuf::from(&r.path)),
            Some(kick.clone())
        ),
        other => panic!("not a sampler: {other:?}"),
    }
    assert_eq!(
        h.state().ui.playlist.picker_tab,
        PickerTab::Audio,
        "the picker shows it"
    );
    assert_eq!(limeshot_ui::playlist::place_source(h.state()), ClipSource::Audio(ch));
    picker_row(&h, ClipSource::Audio(ch));

    // A click on empty space draws it, as long as the sample.
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let tr = empty_track(&h, 3);
    click(&mut h, g.pos(2 * bar + 10, tr), PointerButton::Primary);
    let c = last_clip(&h);
    assert_eq!((c.source, c.track, c.start), (ClipSource::Audio(ch), tr, 2 * bar));
    assert!(c.length.abs_diff(sample_len(&h, &kick)) <= 1);

    // The same sample again (in the picker or dropped on the Playlist) keeps its one channel.
    assert_eq!(limeshot_ui::playlist::add_audio_source(h.state_mut(), &kick), Some(ch));
    let id = limeshot_ui::playlist::add_audio_clip(h.state_mut(), &kick, tr, 6 * bar).expect("placed");
    assert_eq!(h.state().project().clip(id).and_then(Clip::audio_channel), Some(ch));
    assert_eq!(h.state().project().channels.len(), n + 1);

    // The tabs switch the list; picking a pattern places patterns again.
    h.step();
    let tab = |h: &Harness<'static, LimeShotApp>, t: PickerTab| {
        h.state()
            .ui
            .playlist
            .picker_tab_rects
            .iter()
            .find(|(x, _)| *x == t)
            .expect("tab")
            .1
            .center()
    };
    let p = tab(&h, PickerTab::Patterns);
    click(&mut h, p, PointerButton::Primary);
    assert_eq!(h.state().ui.playlist.picker_tab, PickerTab::Patterns);
    let melody = h.state().project().patterns[1].id;
    let p = picker_row(&h, ClipSource::Pattern(melody));
    click(&mut h, p, PointerButton::Primary);
    assert_eq!(
        limeshot_ui::playlist::place_source(h.state()),
        ClipSource::Pattern(melody)
    );
    // Clicking an audio clip picks its sample again.
    click(&mut h, g.pos(2 * bar + 20, tr), PointerButton::Primary);
    assert_eq!(limeshot_ui::playlist::place_source(h.state()), ClipSource::Audio(ch));
    assert_eq!(h.state().ui.playlist.picker_tab, PickerTab::Audio);
}

/// Owner's list item 4 (FL Studio): the Draw tool gives a new clip the length of the clip of the
/// same source last placed, resized or clicked.
#[test]
fn the_draw_tool_reuses_the_length_of_the_last_clip() {
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let tr = empty_track(&h, 3);
    click(&mut h, g.pos(10, tr), PointerButton::Primary);
    let a = last_clip(&h);
    // Resize it to twice its length, then place another: it is as long.
    let edge = g.pos(a.length, tr) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(2 * a.length, tr));
    h.step();
    assert_eq!(h.state().project().clip(a.id).expect("clip").length, 2 * a.length);
    click(&mut h, g.pos(8 * bar + 10, tr), PointerButton::Primary);
    assert_eq!(last_clip(&h).length, 2 * a.length, "the resized length");
    // A clicked clip of the same pattern with another length sets the length.
    let tr2 = empty_track(&h, tr + 1);
    let pattern = a.pattern().expect("pattern clip");
    h.state_mut().exec(Command::AddClip {
        pattern,
        track: tr2,
        start: 0,
        length: Some(3 * bar),
    });
    h.step();
    click(&mut h, g.pos(bar, tr2), PointerButton::Primary);
    click(&mut h, g.pos(16 * bar + 10, tr), PointerButton::Primary);
    assert_eq!(last_clip(&h).length, 3 * bar, "the clicked clip's length");
    // Samples too.
    let kick = bundled("Lime Kick.wav");
    limeshot_ui::playlist::add_audio_source(h.state_mut(), &kick).expect("added");
    h.state_mut().ui.playlist.zoom_x = 200.0;
    h.step();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let tr3 = empty_track(&h, tr2 + 1);
    click(&mut h, g.pos(2, tr3), PointerButton::Primary);
    let s = last_clip(&h);
    let edge = g.pos(s.length, tr3) - vec2(2.0, 0.0);
    drag(&mut h, edge, g.pos(s.length + STEP_TICKS, tr3));
    h.step();
    let resized = h.state().project().clip(s.id).expect("clip").length;
    assert_ne!(resized, s.length);
    click(&mut h, g.pos(2 * bar + 2, tr3), PointerButton::Primary);
    assert_eq!(last_clip(&h).length, resized);
    assert_eq!(last_clip(&h).source, s.source);
}

/// Owner's list items 2 and 3 (observed in FL Studio 2026.1.1): Shift+drag moves a copy and
/// leaves the original; right-click deletes only the clip under it, and on empty space clears
/// the selection.
#[test]
fn shift_drag_duplicates_and_right_click_on_empty_space_deselects() {
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let a = h.state().project().playlist.clips[0].clone();
    let n = h.state().project().playlist.clips.len();
    let tr = empty_track(&h, 3);
    let undo = h.state().doc.history().len();
    let from = g.pos(a.start + a.length / 2, a.track);
    drag_with(
        &mut h,
        from,
        g.pos(a.start + a.length / 2 + 8 * bar, tr),
        Modifiers::SHIFT,
    );
    let clips = h.state().project().playlist.clips.clone();
    assert_eq!(clips.len(), n + 1, "a copy");
    assert_eq!(h.state().project().clip(a.id), Some(&a), "the original stays");
    let copy = clips.last().expect("copy");
    assert_eq!((copy.source, copy.track, copy.start), (a.source, tr, a.start + 8 * bar));
    assert_eq!(h.state().ui.playlist.selection, vec![copy.id], "the copy is selected");
    assert_eq!(h.state().doc.history().len(), undo + 1, "one undo step");
    // A Shift+click that does not move leaves no copy.
    click_with(&mut h, from, PointerButton::Primary, Modifiers::SHIFT);
    assert_eq!(h.state().project().playlist.clips.len(), n + 1);

    // Right-click another clip: only it goes, the selection stays.
    let b = h.state().project().playlist.clips[1].clone();
    h.state_mut().ui.playlist.selection = vec![copy.id];
    click(&mut h, g.pos(b.start + b.length / 2, b.track), PointerButton::Secondary);
    assert!(h.state().project().clip(b.id).is_none());
    assert_eq!(h.state().ui.playlist.selection, vec![copy.id]);
    // Right-click on empty space clears it.
    let empty = g.pos(5 * bar, empty_track(&h, tr + 1));
    click(&mut h, empty, PointerButton::Secondary);
    assert!(h.state().ui.playlist.selection.is_empty());
    assert!(h.state().project().clip(copy.id).is_some());
}

/// Owner's list item 12: a sample dragged over the Playlist shows a ghost clip where it lands,
/// as long as the sample (FL Studio highlights the target cell and labels the pointer).
#[test]
fn a_sample_dragged_over_the_playlist_shows_where_it_lands() {
    let mut h = playlist_harness();
    let kick = bundled("Lime Kick.wav");
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let bar = h.state().project().time_signature.ticks_per_bar();
    let tr = empty_track(&h, 3);
    let from = h.state().ui.browser.file_pos(&kick).expect("listed");
    let to = g.pos(4 * bar + 30, tr);
    h.event(Event::PointerMoved(from));
    h.step();
    press(&mut h, from, PointerButton::Primary, true);
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    h.step();
    let len = sample_len(&h, &kick);
    let ghost = h.state().ui.playlist.drop_ghost.expect("a ghost while dragging");
    assert_eq!((ghost.0, ghost.1), (tr, 4 * bar));
    assert!(ghost.2.abs_diff(len) <= 1);
    press(&mut h, to, PointerButton::Primary, false);
    h.step();
    assert!(h.state().ui.playlist.drop_ghost.is_none());
    let c = last_clip(&h);
    assert_eq!((c.track, c.start), (tr, 4 * bar));
}

/// Owner's list item 5: Vital's and Serum's Channel Settings could not be narrowed. A row wider
/// than the window (there: a slider row with a long value such as "1.95312ms") grew the window
/// every frame until it filled the workspace. Here a channel name with no break points does the
/// same; the window keeps its size and can be narrowed and widened.
#[test]
fn a_wide_row_does_not_stop_a_window_from_being_resized() {
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").id;
    h.state_mut().exec(Command::RenameChannel {
        channel: ch,
        name: "W".repeat(200),
    });
    h.state_mut().show_window(Win::ChannelSettings);
    for _ in 0..6 {
        h.step();
    }
    let rect = |h: &Harness<'static, LimeShotApp>| {
        h.ctx
            .memory(|m| m.area_rect(Win::ChannelSettings.id()))
            .expect("settings drawn")
    };
    let ws = h.state().ui.workspace.area;
    let before = rect(&h);
    assert!(
        before.width() < ws.width() * 0.6,
        "did not grow to the content: {before:?}"
    );
    // Narrower from the right edge, then wider again.
    let edge = before.right_center() - vec2(2.0, 0.0);
    drag(&mut h, edge, edge - vec2(80.0, 0.0));
    h.step();
    let narrow = rect(&h);
    assert!(narrow.width() < before.width() - 60.0, "narrowed: {narrow:?}");
    let edge = narrow.right_center() - vec2(2.0, 0.0);
    drag(&mut h, edge, edge + vec2(120.0, 0.0));
    h.step();
    assert!(rect(&h).width() > narrow.width() + 100.0, "widened: {:?}", rect(&h));
    // A plugin channel has no Sampler/LimeSynth switch (owner's list item 6).
    assert!(h.query_by_label("LimeSynth").is_none());
}

/// Owner's list item 8: every audition note-on gets its note-off. Drawing, moving and the
/// keyboard end their audition on release, and any audition left once no mouse button is down
/// is released by the app (the engine side: `limeshot-engine` tests/plugins.rs).
#[test]
fn auditions_end_when_the_mouse_button_is_released() {
    let (mut h, pat, ch) = empty_piano_roll();
    let g = piano_geometry(&h);
    let p = g.pos(STEP_TICKS * 2, 60);
    h.event(Event::PointerMoved(p));
    h.step();
    press(&mut h, p, PointerButton::Primary, true);
    assert_eq!(h.state().auditions, vec![(ch, 60)], "drawing auditions the note");
    // Dragged up two keys: the first audition ends, the new key sounds.
    h.event(Event::PointerMoved(g.pos(STEP_TICKS * 2, 62)));
    h.step();
    assert_eq!(h.state().auditions, vec![(ch, 62)]);
    press(&mut h, g.pos(STEP_TICKS * 2, 62), PointerButton::Primary, false);
    h.step();
    assert!(h.state().auditions.is_empty());
    assert_eq!(notes_by_start(&h, pat, ch).len(), 1);
    // An audition nobody ends is released once no button is down.
    h.state_mut().preview_on(ch, 70, 0.8);
    h.step();
    assert!(h.state().auditions.is_empty());
}

/// Owner's list item 11 (FL Studio's Browser preview panel): the selected sample's waveform
/// with the preview's position; clicking it plays the sample again. Samples also play when
/// clicked in the Playlist's picker (with the Browser hidden) and when their waveform is
/// pressed in Channel Settings.
#[test]
fn samples_preview_from_the_browser_panel_the_picker_and_channel_settings() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let hat = bundled("Lime Hat Open.wav");
    let pos = h.state().ui.browser.file_pos(&hat).expect("listed");
    click(&mut h, pos, PointerButton::Primary);
    wait_for(&mut h, "the preview", |a| a.ui.browser.preview_started.is_some());
    assert_eq!(
        h.state().ui.browser.preview_started.as_ref().map(|p| p.0.clone()),
        Some(hat.clone())
    );
    let panel = h.state().ui.browser.preview_rect.expect("preview panel drawn");
    assert!(panel.width() > 100.0);
    // A click on the waveform plays it again.
    let before = h.state().ui.browser.preview_started.as_ref().map(|p| p.1);
    click(&mut h, panel.center(), PointerButton::Primary);
    wait_for(&mut h, "a second preview", |a| {
        a.ui.browser.preview_started.as_ref().map(|p| p.1) != before
    });

    // The picker's audio clips play when clicked, with the Browser hidden.
    h.state_mut().ui.show_browser = false;
    h.state_mut().ui.browser.preview_started = None;
    let ch = limeshot_ui::playlist::add_audio_source(h.state_mut(), &hat).expect("added");
    h.state_mut().ui.playlist.picker_tab = limeshot_ui::playlist::PickerTab::Audio;
    h.step();
    let p = picker_row(&h, ClipSource::Audio(ch));
    click(&mut h, p, PointerButton::Primary);
    wait_for(&mut h, "the picker's preview", |a| {
        a.ui.browser.preview_started.is_some()
    });

    // Channel Settings: pressing the waveform plays the channel until the button is released.
    h.state_mut().select_channel(ch);
    h.state_mut().show_window(Win::ChannelSettings);
    for _ in 0..3 {
        h.step();
    }
    let key = h.state().project().channel(ch).expect("ch").root_key;
    // The waveform is the 64 px strip under the sample's name and Load… button.
    let load = h.get_by_label("Load…").rect();
    let wave = egui::pos2(load.left(), load.bottom() + 36.0);
    h.event(Event::PointerMoved(wave));
    h.step();
    press(&mut h, wave, PointerButton::Primary, true);
    assert_eq!(h.state().auditions, vec![(ch, key)]);
    press(&mut h, wave, PointerButton::Primary, false);
    h.step();
    assert!(h.state().auditions.is_empty());
}

fn top_window(h: &Harness<'static, LimeShotApp>) -> Option<Win> {
    let top = h.ctx.memory(|m| m.areas().top_layer_id(egui::Order::Middle))?;
    Win::ALL.into_iter().find(|w| top.id == w.id())
}

fn window_rect(h: &Harness<'static, LimeShotApp>, w: Win) -> egui::Rect {
    h.ctx.memory(|m| m.area_rect(w.id())).expect("window drawn")
}

/// Owner's list items 14 and 15 (observed in FL Studio 2026.1.1): clicking a window behind
/// another brings it to the front — the maximised Playlist too; a pinned window stays on top;
/// double-clicking a title bar maximises the window and double-clicking again restores it.
#[test]
fn windows_raise_on_click_pin_on_top_and_maximise_on_double_click() {
    let mut h = harness(true);
    h.state_mut().show_window(Win::Mixer);
    h.step();
    h.step();
    assert_eq!(top_window(&h), Some(Win::Mixer));
    // The rack's title bar is not covered by the Mixer: a click there raises the rack.
    let rack = window_rect(&h, Win::ChannelRack);
    click(
        &mut h,
        egui::pos2(rack.left() + 60.0, rack.top() + 6.0),
        PointerButton::Primary,
    );
    h.step();
    assert_eq!(top_window(&h), Some(Win::ChannelRack));
    // So does a click in the body of the window behind.
    let mixer = window_rect(&h, Win::Mixer);
    click(
        &mut h,
        egui::pos2(mixer.right() - 30.0, mixer.bottom() - 30.0),
        PointerButton::Primary,
    );
    h.step();
    assert_eq!(top_window(&h), Some(Win::Mixer));
    // The maximised Playlist comes to the front like any window.
    let ws = h.state().ui.workspace.area;
    click(
        &mut h,
        egui::pos2(ws.right() - 20.0, ws.top() + 60.0),
        PointerButton::Primary,
    );
    h.step();
    assert_eq!(top_window(&h), Some(Win::Playlist));

    // A pinned window stays above the others.
    h.state_mut().ui.workspace.state_mut(Win::Mixer).pinned = true;
    h.step();
    h.state_mut().show_window(Win::ChannelRack);
    h.step();
    h.step();
    let mixer = window_rect(&h, Win::Mixer);
    let over = |h: &Harness<'static, LimeShotApp>, p: egui::Pos2| h.ctx.layer_id_at(p).map(|l| l.id);
    assert_eq!(over(&h, mixer.center()), Some(Win::Mixer.id()), "pinned stays visible");

    // Double-click the rack's title: maximised; again: restored to its old rectangle.
    h.state_mut().ui.workspace.state_mut(Win::Mixer).pinned = false;
    h.state_mut().ui.workspace.close(Win::Mixer);
    h.step();
    let before = window_rect(&h, Win::ChannelRack);
    double_click(&mut h, egui::pos2(before.left() + 80.0, before.top() + 12.0));
    h.step();
    assert!(h.state().ui.workspace.state(Win::ChannelRack).maximized);
    let max = window_rect(&h, Win::ChannelRack);
    double_click(&mut h, egui::pos2(max.left() + 80.0, max.top() + 12.0));
    h.step();
    h.step();
    assert!(!h.state().ui.workspace.state(Win::ChannelRack).maximized);
    let after = window_rect(&h, Win::ChannelRack);
    assert!((after.min - before.min).length() < 1.0 && (after.size() - before.size()).length() < 1.0);
}

/// Owner's list item 16: the toolbar's pattern selector has no − and + buttons; its list ends
/// with Create new (FL Studio's selector menu makes a new pattern from it too).
#[test]
fn the_pattern_selector_creates_new_patterns_from_its_list() {
    let mut h = harness(true);
    let n = h.state().project().patterns.len();
    let cur = h.state().current_pattern();
    let name = h.state().project().pattern(cur).expect("pattern").name.clone();
    assert!(h.query_by_label("Previous pattern (−)").is_none());
    let combo = h
        .query_all_by_value(&name)
        .chain(h.query_all_by_label(&name))
        .find(|node| node.rect().top() < 120.0)
        .expect("the selector in the toolbar")
        .rect()
        .center();
    click(&mut h, combo, PointerButton::Primary);
    h.get_by_label("Create new").click();
    h.step();
    h.step();
    assert_eq!(h.state().project().patterns.len(), n + 1);
    assert_eq!(h.state().current_pattern(), h.state().project().patterns[n].id);
}
