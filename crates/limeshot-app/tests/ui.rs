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
    assert!(
        h.state().is_window_open(Win::ChannelRack),
        "a background window comes forward"
    );
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
    assert!(h.state().plugins.is_window_open(PluginTarget::Channel(kick)));
    assert_eq!(h.state().selected_channel(), Some(kick));
    for _ in 0..3 {
        h.step();
    }
    h.state_mut().show_window(Win::ChannelRack);
    h.step();
    let button = h.state().ui.rack.step_pos(kick, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Primary);
    assert!(
        h.state().plugins.is_window_open(PluginTarget::Channel(kick)),
        "a background plugin is raised"
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
    // FL Studio asks for its name (Find next empty…).
    h.get_by_label("Cancel").click();
    h.step();
    assert!(h.state().dialog.is_none());
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
    wait_for(&mut h, "external Playlist import", |a| {
        a.project().playlist.clips.len() == before + 1
    });
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
        channels,
        "reuse the existing sampler for this source"
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
    // Curve modes belong to the between-point tension handle.
    let handle = egui::pos2(g.pos(bar, c.track).x, y_of((flat + 1.0) * 0.5));
    click(&mut h, handle, PointerButton::Secondary);
    h.step();
    h.get_by_label("Stairs").click();
    h.step();
    assert_eq!(points(&h)[0].mode, PointMode::Stairs);
    click(&mut h, egui::pos2(x, y_of(1.0)), PointerButton::Secondary);
    h.step();
    h.get_by_label("Delete").click();
    h.step();
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

#[test]
fn rack_zip_shortcuts_compact_selected_rows_and_right_click_restores_them() {
    let mut h = harness(true);
    let ids = order(&h);
    let pid = h.state().current_pattern();
    let before_y = led(&h, ids[2]).y;
    let before = h.state().project().clone();
    let revision = h.state().doc.revision();
    h.state_mut().select_channel(ids[0]);
    h.state_mut().toggle_channel_selection(ids[1]);
    h.state_mut().ui.rack.graph = Some(NoteProp::Velocity);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::Z);
    h.step();
    h.step();
    assert_eq!(h.state().ui.rack.zipped.len(), 2);
    assert!(h.state().ui.rack.step_pos(ids[0], 0).is_none());
    assert!(h.state().ui.rack.led_pos(ids[0]).is_none());
    assert!(h.state().ui.rack.graph_rect.is_none(), "no graph over another row");
    assert!(led(&h, ids[2]).y < before_y - 20.0, "compact rows reclaim space");
    let compact = |h: &Harness<'static, LimeShotApp>, id| {
        h.state()
            .ui
            .rack
            .zipped_buttons
            .iter()
            .find(|(c, _)| *c == id)
            .unwrap()
            .1
            .center()
    };
    let p = compact(&h, ids[0]);
    click(&mut h, p, PointerButton::Primary);
    assert_eq!(h.state().selected_channels(), vec![ids[0]]);
    let p = compact(&h, ids[1]);
    click_with(&mut h, p, PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(h.state().selected_channels(), vec![ids[0], ids[1]]);
    let p = compact(&h, ids[0]);
    click(&mut h, p, PointerButton::Secondary);
    assert!(!h.state().ui.rack.zipped.contains(&ids[0]));
    assert!(h.state().ui.rack.zipped.contains(&ids[1]));
    assert!(h.state().ui.rack.step_pos(ids[0], 0).is_some());
    assert!(
        h.query_by_label("Piano roll").is_none(),
        "unzip does not open the channel menu"
    );
    let other_pattern = h.state().project().patterns[1].id;
    h.state_mut().set_current_pattern(other_pattern);
    h.step();
    assert!(
        h.state().ui.rack.zipped.contains(&ids[1]),
        "zip follows the channel across patterns"
    );
    h.state_mut().set_current_pattern(pid);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::U);
    h.step();
    h.step();
    assert!(h.state().ui.rack.zipped.is_empty());
    assert_eq!(led(&h, ids[2]).y, before_y);
    assert_eq!(h.state().project(), &before);
    assert_eq!(
        h.state().doc.revision(),
        revision,
        "view changes leave undo history alone"
    );
}

#[test]
fn rack_zip_menu_and_filters_restore_hidden_rows_and_forget_deleted_channels() {
    use limeshot_ui::channel_rack::RackFilter;
    let mut h = harness(true);
    let ids = order(&h);
    h.state_mut().select_channel(ids[0]);
    let button = h.state().ui.rack.menu_button.unwrap().center();
    click(&mut h, button, PointerButton::Primary);
    h.get_by_label_contains("Zip selected").click();
    h.step();
    assert!(h.state().ui.rack.zipped.contains(&ids[0]));
    h.state_mut().exec(Command::SetChannelGroup {
        channels: vec![ids[0]],
        group: Some("Drums".into()),
    });
    h.state_mut().ui.rack.filter = RackFilter::Unsorted;
    h.step();
    assert!(h.state().ui.rack.zipped_buttons.is_empty());
    let button = h.state().ui.rack.menu_button.unwrap().center();
    click(&mut h, button, PointerButton::Primary);
    h.get_by_label_contains("Unzip all").click();
    h.step();
    h.state_mut().ui.rack.filter = RackFilter::All;
    h.step();
    assert!(
        h.state().ui.rack.step_pos(ids[0], 0).is_some(),
        "unzip includes filtered-out rows"
    );
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::Z);
    h.step();
    h.state_mut().exec(Command::RemoveChannel(ids[0]));
    h.step();
    assert!(!h.state().ui.rack.zipped.contains(&ids[0]));
    h.state_mut().doc.undo();
    h.step();
    assert!(
        h.state().ui.rack.step_pos(ids[0], 0).is_some(),
        "undo restores an accessible full row"
    );
    h.state_mut().ui.channel_selection = ids.clone();
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::Z);
    h.step();
    assert!(h.state().ui.rack.step_strips.is_empty(), "all rows can be compacted");
    assert_eq!(h.state().ui.rack.zipped_buttons.len(), ids.len());
    h.state_mut().new_project();
    h.step();
    assert!(h.state().ui.rack.zipped.is_empty(), "new projects start expanded");
}

#[test]
fn rack_zip_finishes_a_held_step_stroke_before_hiding_it() {
    let mut h = harness(true);
    let cid = h.state().project().channels[3].id;
    let pid = h.state().current_pattern();
    let p = h.state().ui.rack.step_pos(cid, 0).unwrap();
    h.event(Event::PointerMoved(p));
    h.step();
    press(&mut h, p, PointerButton::Primary, true);
    assert!(
        h.state()
            .project()
            .pattern(pid)
            .unwrap()
            .step_note(cid, 60, 0)
            .is_some()
    );
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::Z);
    h.step();
    press(&mut h, p, PointerButton::Primary, false);
    h.key_press_modifiers(Modifiers::ALT, Key::U);
    h.step();
    let p = h.state().ui.rack.step_pos(cid, 1).unwrap();
    click(&mut h, p, PointerButton::Primary);
    let pattern = h.state().project().pattern(pid).unwrap();
    assert!(pattern.step_note(cid, 60, 0).is_some());
    assert!(pattern.step_note(cid, 60, 1).is_some());
    h.state_mut().doc.undo();
    let pattern = h.state().project().pattern(pid).unwrap();
    assert!(
        pattern.step_note(cid, 60, 0).is_some(),
        "the earlier stroke is a separate undo step"
    );
    assert!(pattern.step_note(cid, 60, 1).is_none());
}

#[test]
fn rack_auto_zip_setting_is_off_by_default_and_does_not_edit_the_project() {
    let mut h = harness(true);
    let before = h.state().project().clone();
    let revision = h.state().doc.revision();
    assert!(!h.state().settings.auto_zip_empty_channels);
    h.get_by_label("Options").click();
    h.step();
    h.get_by_label_contains("General settings").click();
    h.step();
    h.get_by_label("Auto zip empty channels").click();
    h.step();
    assert!(h.state().settings.auto_zip_empty_channels);
    assert!(
        h.state().ui.rack.zipped.is_empty(),
        "applies on the next pattern switch"
    );
    h.get_by_label("Auto zip empty channels").click();
    h.step();
    assert!(!h.state().settings.auto_zip_empty_channels);
    assert_eq!(h.state().project(), &before);
    assert_eq!(h.state().doc.revision(), revision);
}

#[test]
fn rack_auto_zip_follows_pattern_notes_and_preserves_manual_restoration_between_switches() {
    use limeshot_ui::channel_rack::RackFilter;
    let mut h = harness(true);
    let ids = order(&h);
    let original = h.state().current_pattern();
    let sparse = PatternId(
        h.state_mut()
            .exec(Command::AddPattern { name: "Sparse".into() })
            .unwrap(),
    );
    let empty = PatternId(
        h.state_mut()
            .exec(Command::AddPattern { name: "Empty".into() })
            .unwrap(),
    );
    h.state_mut().exec(Command::SetStep {
        pattern: sparse,
        channel: ids[0],
        step: 0,
        on: true,
    });
    let note = NoteId(
        h.state_mut()
            .exec(Command::AddNote {
                pattern: sparse,
                note: NewNote {
                    channel: ids[1],
                    key: 73,
                    start: 123,
                    length: 700,
                    velocity: 0.8,
                },
            })
            .unwrap(),
    );
    h.state_mut().exec(Command::SetNotesMuted {
        pattern: sparse,
        notes: vec![note],
        muted: true,
    });
    h.state_mut().exec(Command::SetChannelGroup {
        channels: vec![ids[2]],
        group: Some("Hidden".into()),
    });
    h.state_mut().ui.rack.filter = RackFilter::Unsorted;
    h.state_mut().ui.rack.zipped.insert(ids[0]);
    h.state_mut().set_current_pattern(sparse);
    assert!(
        h.state().ui.rack.zipped.contains(&ids[0]),
        "disabled leaves manual state alone"
    );
    h.state_mut().set_current_pattern(original);
    h.state_mut().settings.auto_zip_empty_channels = true;
    let before = h.state().project().clone();
    let revision = h.state().doc.revision();
    h.state_mut().set_current_pattern(sparse);
    h.step();
    assert_eq!(h.state().ui.rack.zipped.iter().copied().collect::<Vec<_>>(), ids[2..]);
    assert!(h.state().ui.rack.step_pos(ids[0], 0).is_some());
    assert!(
        h.state().ui.rack.step_pos(ids[1], 0).is_some(),
        "muted melodic note data is nonempty"
    );
    h.state_mut().ui.rack.filter = RackFilter::All;
    h.step();
    let p = h
        .state()
        .ui
        .rack
        .zipped_buttons
        .iter()
        .find(|(c, _)| *c == ids[2])
        .unwrap()
        .1
        .center();
    click(&mut h, p, PointerButton::Secondary);
    h.state_mut().set_current_pattern(sparse);
    h.step();
    assert!(
        h.state().ui.rack.step_pos(ids[2], 0).is_some(),
        "same-pattern selection keeps restoration"
    );
    h.state_mut().set_current_pattern(empty);
    h.step();
    assert_eq!(h.state().ui.rack.zipped.len(), ids.len());
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press_modifiers(Modifiers::ALT, Key::U);
    h.step();
    assert!(h.state().ui.rack.zipped.is_empty());
    h.state_mut().set_current_pattern(sparse);
    h.step();
    assert_eq!(h.state().ui.rack.zipped.len(), ids.len() - 2);
    assert_eq!(h.state().project(), &before);
    assert_eq!(h.state().doc.revision(), revision);
    h.state_mut().settings.auto_zip_empty_channels = false;
    h.state_mut().set_current_pattern(empty);
    assert!(
        !h.state().ui.rack.zipped.contains(&ids[0]),
        "disabled preserves the current view"
    );
    h.state_mut().new_project();
    h.step();
    assert!(h.state().ui.rack.zipped.is_empty(), "new projects start expanded");
}

#[test]
fn rack_auto_zip_handles_pattern_deletion_and_undo_fallbacks() {
    let mut h = harness(true);
    let ids = order(&h);
    let original = h.state().current_pattern();
    let empty = PatternId(
        h.state_mut()
            .exec(Command::AddPattern { name: "Empty".into() })
            .unwrap(),
    );
    h.state_mut().settings.auto_zip_empty_channels = true;
    h.state_mut().set_current_pattern(empty);
    h.step();
    assert_eq!(h.state().ui.rack.zipped.len(), ids.len());
    h.state_mut().exec(Command::RemovePattern(empty));
    h.step();
    assert_eq!(h.state().current_pattern(), original);
    assert!(
        !h.state().ui.rack.zipped.contains(&ids[0]),
        "fallback expands populated rows"
    );
    h.state_mut().doc.undo();
    h.step();
    assert_eq!(h.state().current_pattern(), empty);
    assert_eq!(h.state().ui.rack.zipped.len(), ids.len());
}

#[test]
fn rack_auto_zip_finishes_a_held_stroke_in_the_old_pattern() {
    let mut h = harness(true);
    let cid = h.state().project().channels[3].id;
    let old = h.state().current_pattern();
    let next = PatternId(h.state_mut().exec(Command::AddPattern { name: "Next".into() }).unwrap());
    // Keep the destination rows expanded so the held strip keeps receiving pointer input.
    let channels: Vec<_> = h.state().project().channels.iter().map(|c| c.id).collect();
    for channel in channels {
        h.state_mut().exec(Command::SetStep {
            pattern: next,
            channel,
            step: 8,
            on: true,
        });
    }
    let next_before = h.state().project().pattern(next).unwrap().clone();
    h.state_mut().settings.auto_zip_empty_channels = true;
    let p = h.state().ui.rack.step_pos(cid, 0).unwrap();
    h.event(Event::PointerMoved(p));
    h.step();
    press(&mut h, p, PointerButton::Primary, true);
    press(&mut h, p, PointerButton::Secondary, true);
    let history = h.state().doc.history().len();
    h.state_mut().set_current_pattern(next);
    h.step();
    let p = h.state().ui.rack.step_pos(cid, 1).unwrap();
    h.event(Event::PointerMoved(p));
    h.step();
    assert_eq!(h.state().project().pattern(next).unwrap(), &next_before);
    press(&mut h, p, PointerButton::Primary, false);
    h.step();
    assert_eq!(
        h.state().project().pattern(next).unwrap(),
        &next_before,
        "releasing one button must not unblock a stroke while another is held"
    );
    press(&mut h, p, PointerButton::Secondary, false);
    assert_eq!(h.state().project().pattern(next).unwrap(), &next_before);
    assert_eq!(h.state().doc.history().len(), history);
    click(&mut h, p, PointerButton::Primary);
    assert!(
        h.state()
            .project()
            .pattern(next)
            .unwrap()
            .step_note(cid, 60, 1)
            .is_some()
    );
    assert_eq!(h.state().doc.history().len(), history + 1);
    h.state_mut().doc.undo();
    assert_eq!(h.state().project().pattern(next).unwrap(), &next_before);
    let pattern = h.state().project().pattern(old).unwrap();
    assert!(
        pattern.step_note(cid, 60, 0).is_some(),
        "old stroke is its own undo step"
    );
    assert!(pattern.step_note(cid, 60, 1).is_none());
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
fn browser_click_and_double_click_only_preview_samples() {
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

    // Double-click never edits or replaces a channel, even when a sampler is selected.
    let clap = h.state().project().channels[1].id;
    h.state_mut().select_channel(clap);
    let before = h.state().project().clone();
    let pos = h.state().ui.browser.file_pos(&kick).expect("listed");
    double_click(&mut h, pos);
    assert_eq!(h.state().project(), &before);
    let lead = h.state().project().channels.last().unwrap().id;
    h.state_mut().select_channel(lead);
    double_click(&mut h, pos);
    assert_eq!(h.state().project(), &before);
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

/// PR-016: Target control changes the actual lower lane; pan edits preserve velocity,
/// channel isolation, save/reload and the one-drag/one-undo contract.
#[test]
fn piano_roll_target_control_edits_note_pan() {
    let (mut h, pat, lead) = melody_piano_roll();
    let other_channel = h.state().project().channels[4].id;
    h.state_mut().exec(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel: other_channel,
            key: 48,
            start: 0,
            length: STEP_TICKS,
            velocity: 0.6,
        },
    });
    h.step();
    let before = pattern_notes(&h, pat, lead);
    let project = h.state().project().clone();
    let revision = h.state().doc.revision();
    piano_menu(&mut h, "Target control", "Note pan");
    assert_eq!(h.state().ui.piano.target_control, NoteProp::Pan);
    assert_eq!(h.state().doc.revision(), revision, "target choice is only view state");
    let g = piano_geometry(&h);
    let undo = h.state().doc.history().len();
    click(&mut h, g.vel_pos(before[4].start, 0.0), PointerButton::Primary);
    assert_eq!(pattern_notes(&h, pat, lead)[4].pan, -1.0);
    h.state_mut().doc.undo();
    assert_eq!(h.state().project(), &project);
    // Include a stale selection from another channel: only the current channel may change.
    let other = project
        .pattern(pat)
        .unwrap()
        .notes
        .iter()
        .find(|n| n.channel != lead)
        .unwrap()
        .id;
    h.state_mut().ui.piano.selection = vec![before[0].id, before[1].id, other];
    let last_vel = h.state().ui.piano.last_vel;
    drag(&mut h, g.vel_pos(before[0].start, 0.2), g.vel_pos(before[0].start, 1.0));
    h.step();
    let after = pattern_notes(&h, pat, lead);
    assert_eq!(after[0].pan, 1.0);
    assert_eq!(after[1].pan, 1.0);
    assert_eq!(after[2], before[2]);
    assert_eq!(
        h.state()
            .project()
            .pattern(pat)
            .unwrap()
            .notes
            .iter()
            .find(|n| n.id == other),
        project.pattern(pat).unwrap().notes.iter().find(|n| n.id == other)
    );
    assert_eq!(
        after.iter().map(|n| n.velocity).collect::<Vec<_>>(),
        before.iter().map(|n| n.velocity).collect::<Vec<_>>()
    );
    assert_eq!(h.state().ui.piano.last_vel, last_vel);
    assert_eq!(h.state().doc.history().len(), undo + 1);
    let saved = limeshot_project::to_json(h.state().project(), None).unwrap();
    let loaded = limeshot_project::from_json(&saved, std::path::Path::new("pan.lime")).unwrap();
    assert_eq!(loaded, *h.state().project());
    h.state_mut().doc.undo();
    assert_eq!(h.state().project(), &project);
    h.state_mut().doc.redo();
    click(&mut h, g.vel_pos(before[0].start, 0.5), PointerButton::Primary);
    assert!(pattern_notes(&h, pat, lead)[0].pan.abs() < 0.03, "middle is balanced");
    piano_menu(&mut h, "Target control", "Note velocity");
    click(&mut h, g.vel_pos(before[0].start, 0.3), PointerButton::Primary);
    let n = pattern_notes(&h, pat, lead)[0];
    assert!((n.velocity - 0.3).abs() < 0.03);
    assert!(n.pan.abs() < 0.03, "velocity editing preserves pan");
}

#[test]
fn piano_roll_keyboard_label_view_options_do_not_edit_project() {
    use limeshot_ui::piano_roll::KeyboardLabels;
    let (mut h, _, _) = melody_piano_roll();
    let project = h.state().project().clone();
    let revision = h.state().doc.revision();
    let button = h.state().ui.piano.menu_button;
    editor_menu(&mut h, button, "View", "Keyboard labels");
    open_item(&mut h, "Keyboard labels", "All notes");
    assert_eq!(h.state().ui.piano.keyboard_labels, KeyboardLabels::All);
    let button = h.state().ui.piano.menu_button;
    editor_menu(&mut h, button, "View", "Keyboard labels");
    open_item(&mut h, "Keyboard labels", "White notes only");
    assert!(h.state().ui.piano.white_key_labels_only);
    h.key_press(Key::Escape);
    h.step();
    h.key_press(Key::Escape);
    h.step();
    piano_menu(&mut h, "View", "Keep labels on screen");
    assert!(h.state().ui.piano.keep_labels_on_screen);
    h.key_press(Key::Escape);
    h.step();
    h.state_mut().ui.piano.scroll.x = 120.0;
    h.step();
    assert_eq!(h.state().project(), &project);
    assert_eq!(h.state().doc.revision(), revision);
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
    wait_for(&mut h, "the auditioned note", |a| a.player.status().peak(0).0 > 0.001);
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

    // Bring the top track back into view before resizing its clip.
    h.state_mut().ui.playlist.scroll.y = 0.0;
    h.step();
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
    // Exercise the UI -> player loop range without requiring several seconds of audio progress
    // in a fixed wall-clock window: the silent thread can fall behind on busy CI runners.
    // Full loops at multiple buffer sizes are covered by the engine's deterministic tests.
    let near_end = 3 * bar - PPQ / 4;
    h.state_mut().locate(near_end);
    wait_for(&mut h, "the locate near the selection's end", |a| {
        a.player.status().position() == near_end
    });
    h.state_mut().set_song_loop(false);
    h.state_mut().toggle_play();
    wait_for(&mut h, "playback to loop to the selection's start", |a| {
        let pos = a.player.status().position();
        assert!((bar..3 * bar).contains(&pos), "{pos} left the selection");
        a.is_playing() && pos < near_end
    });
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
fn mixer_eq_edits_hosted_state_and_undoes() {
    let mut h = harness(true);
    h.state_mut().ui.mixer.selected = 1;
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    for _ in 0..3 {
        h.step();
    }
    let before = h.state().project().mixer.tracks[1].eq;
    let g = h.state().ui.mixer.eq_graph.expect("EQ graph drawn");
    let from = limeshot_ui::mixer::eq_handle(g, EqBand::DEFAULTS[1]);
    drag(&mut h, from, from + egui::vec2(30.0, -20.0));
    assert_ne!(h.state().project().mixer.tracks[1].eq, before);
    assert!(
        h.state().project().mixer.tracks[1]
            .effects
            .iter()
            .any(limeshot_core::lime::is_track_eq)
    );
    h.state_mut().doc.undo();
    assert_eq!(h.state().project().mixer.tracks[1].eq, before);
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
    let fx = h
        .state_mut()
        .exec(Command::AddEffect {
            track: 5,
            kind: EffectKind::Compressor,
        })
        .unwrap();
    h.step();
    let slot = h.state().project().mixer.tracks[5]
        .effects
        .iter()
        .find(|e| e.id == fx)
        .unwrap();
    assert_eq!(slot.kind, EffectKind::Plugin);
    assert_eq!(slot.plugin.as_ref().and_then(LimePlugin::of), Some(LimePlugin(7)));
    h.state_mut().ui.mixer.selected = 5;
    for _ in 0..3 {
        h.step();
    }
    h.state_mut().plugins.open_window(
        PluginTarget::Effect { track: 5, effect: fx },
        Some(limeshot_ui::plugin_window::Tab::Settings),
    );
    h.run_steps(3);
    h.get_by_label("Sidechain: own input").click();
    h.run_steps(4);
    let option = h.get_by_label("Insert 1").rect();
    click(&mut h, option.center(), PointerButton::Primary);
    h.run_steps(3);
    let source = |h: &Harness<'static, LimeShotApp>| {
        h.state()
            .project()
            .plugin(PluginTarget::Effect { track: 5, effect: fx })
            .unwrap()
            .extra
            .get("lime-sidechain-track")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    assert_eq!(source(&h), 1);
    h.state_mut().doc.undo();
    h.step();
    assert_eq!(source(&h), 0);
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
    // Volume remains on the mixer strip.
    h.state_mut().exec(Command::SetMixerVolume { track: 1, volume: 0.3 });
    h.step();
    let r = h.state().ui.mixer.fader_rects.iter().find(|(i, _)| *i == 1).unwrap().1;
    double_click(&mut h, r.center());
    assert_eq!(h.state().project().mixer.tracks[1].volume, 1.0);
    // Hosted effects use the same reset gesture, with an undoable plugin state.
    let fx = h
        .state_mut()
        .exec(Command::AddEffect {
            track: 1,
            kind: EffectKind::Delay,
        })
        .unwrap();
    let target = PluginTarget::Effect { track: 1, effect: fx };
    h.step();
    h.step();
    let id = lime::param_id(0);
    h.state_mut()
        .plugins
        .open_window(target, Some(limeshot_ui::plugin_window::Tab::Settings));
    h.run_steps(3);
    h.state_mut().plugins.set_param(target, id, 0.9);
    h.step();
    let rect = h.state().plugins.parameter_rect(target, id).unwrap();
    double_click(&mut h, rect.center());
    let inst = h.state().plugins.instance(target).unwrap();
    let default = inst.params().iter().find(|p| p.id == id).unwrap().default;
    assert!(
        (inst.param(id) - default).abs() < 1e-6,
        "actual {}, default {}",
        inst.param(id),
        default
    );
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
    wheel_at(&mut h, p, -800.0, Modifiers::ALT);
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

/// APP-026: FL Studio's Edit menu (observed live, session 15): "Undo <action>", a "Redo <action>"
/// item only while there is something to redo (Cmd+Opt+Z), and Cut / Copy / Paste acting on the
/// focused editor.
#[test]
fn the_edit_menu_undoes_redoes_and_cuts_copies_pastes_in_the_focused_editor() {
    let mut h = harness(true);
    let clips = |h: &Harness<'static, LimeShotApp>| h.state().project().playlist.clips.len();
    h.state_mut().exec(Command::AddPlaylistTrack);
    h.step();
    h.get_by_label("Edit").click();
    h.step();
    assert!(h.query_by_label_contains("Undo add playlist track").is_some());
    assert!(
        h.query_by_label_contains("Redo").is_none(),
        "no Redo item with nothing to redo"
    );
    h.get_by_label_contains("Undo add playlist track").click();
    h.step();
    h.get_by_label("Edit").click();
    h.step();
    assert!(h.query_by_label_contains("Redo add playlist track").is_some());
    h.key_press(Key::Escape);
    h.step();
    // Cmd+Opt+Z redoes, as in FL Studio.
    let tracks = h.state().project().playlist.tracks.len();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::Z);
    h.step();
    assert_eq!(h.state().project().playlist.tracks.len(), tracks + 1);

    // With the Playlist focused, Edit › Copy and Paste act on its selected clips.
    let n = clips(&h);
    let first = h.state().project().playlist.clips[0].id;
    h.state_mut().ui.focus = limeshot_ui::app::Focus::Playlist;
    h.state_mut().ui.playlist.selection = vec![first];
    h.step();
    h.get_by_label("Edit").click();
    h.step();
    h.get_by_label_contains("Copy").click();
    h.step();
    assert!(matches!(
        h.state().clipboard,
        Some(limeshot_ui::app::Clipboard::Clips(_))
    ));
    h.get_by_label("Edit").click();
    h.step();
    h.get_by_label_contains("Paste").click();
    h.step();
    assert_eq!(clips(&h), n + 1);
    // Edit › Cut removes the pasted (now selected) clip.
    h.get_by_label("Edit").click();
    h.step();
    h.get_by_label_contains("Cut").click();
    h.step();
    assert_eq!(clips(&h), n);
    // Nothing focused: Cut / Copy / Paste are disabled.
    h.state_mut().ui.focus = limeshot_ui::app::Focus::None;
    assert!(!h.state().can_clipboard(limeshot_ui::app::ClipOp::Paste));
}

/// APP-026: the Tools menu's macros that LimeShot has (FL Studio's Tools › Macros).
#[test]
fn tools_macros_select_unused_channels_purge_audio_clips_and_reset_tracks() {
    let mut h = harness(true);
    let tools = |h: &mut Harness<'static, LimeShotApp>, label: &str| {
        h.get_by_label("Tools").click();
        h.step();
        h.get_by_label_contains(label).click();
        h.step();
    };
    // An unused sampler channel.
    let spare = ChannelId(
        h.state_mut()
            .exec(Command::AddChannel {
                name: "Spare".into(),
                instrument: Instrument::Sampler(SamplerParams::default()),
            })
            .expect("channel"),
    );
    tools(&mut h, "Select unused channels");
    assert_eq!(h.state().ui.channel_selection, vec![spare]);

    // An audio clip whose clip was deleted leaves its channel behind; the purge removes it.
    let dir = limeshot_app::bundled_samples_dir().expect("samples");
    h.state_mut()
        .samples
        .load_blocking(&dir.join("Lime Snare.wav").to_string_lossy())
        .unwrap();
    limeshot_ui::playlist::add_audio_clip(h.state_mut(), &dir.join("Lime Snare.wav"), 7, 0);
    let clip = h
        .state()
        .project()
        .playlist
        .clips
        .iter()
        .find(|c| c.audio_channel().is_some())
        .map(|c| c.id)
        .expect("clip");
    h.state_mut().exec(Command::RemoveClips(vec![clip]));
    let channels = h.state().project().channels.len();
    tools(&mut h, "Purge unused audio clips");
    assert_eq!(h.state().project().channels.len(), channels - 1);
    assert!(
        h.state().project().channels.iter().any(|c| c.id == spare),
        "other unused channels stay"
    );

    // Reset empty playlist tracks: an empty, renamed, muted track gets its default back.
    h.state_mut().exec(Command::RenamePlaylistTrack {
        track: 12,
        name: "Old".into(),
    });
    h.state_mut()
        .exec(Command::SetPlaylistTrackMute { track: 12, muted: true });
    tools(&mut h, "Reset empty playlist tracks");
    let t = &h.state().project().playlist.tracks[12];
    assert_eq!((t.name.as_str(), t.muted), ("Track 13", false));
    assert!(h.state_mut().doc.undo(), "one undo step");
    assert_eq!(h.state().project().playlist.tracks[12].name, "Old");
}

/// UI-013: View › Layout — close all / plugin / unfocused windows, align channel editors, and
/// Arrange windows (into workspace, default and custom layouts).
#[test]
fn view_layout_commands_close_align_and_restore_windows() {
    use limeshot_ui::plugins::Target;
    let mut h = harness(true);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let tone = Target::Channel(h.state().project().channels.last().expect("channel").id);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().plugins.open_window(tone, None);
    for _ in 0..3 {
        h.step();
    }
    // Shift+F12 stacks the plugin windows.
    h.key_press_modifiers(Modifiers::SHIFT, Key::F12);
    h.step();
    assert!(h.state().plugins.is_window_open(tone));
    // Opt+F12 closes the plugin windows only.
    h.key_press_modifiers(Modifiers::ALT, Key::F12);
    h.step();
    assert!(!h.state().plugins.is_window_open(tone));
    assert!(h.state().is_window_open(Win::Mixer));
    // Cmd+F12 keeps only the focused editor.
    h.state_mut().ui.focus = limeshot_ui::app::Focus::Mixer;
    h.key_press_modifiers(Modifiers::COMMAND, Key::F12);
    h.step();
    assert!(h.state().is_window_open(Win::Mixer));
    assert!(!h.state().is_window_open(Win::Playlist) && !h.state().is_window_open(Win::ChannelRack));
    // F12 closes everything.
    h.key_press(Key::F12);
    h.step();
    assert!(Win::ALL.iter().all(|w| !h.state().is_window_open(*w)));

    // Arrange windows › Desktop (default): FL's new-project layout.
    h.get_by_label("View").click();
    h.step();
    h.get_by_label("Arrange windows ⏵").hover();
    for _ in 0..3 {
        h.step();
    }
    h.get_by_label_contains("Desktop (default)").click();
    for _ in 0..2 {
        h.step();
    }
    assert!(h.state().is_window_open(Win::Playlist) && h.state().is_window_open(Win::ChannelRack));
    assert!(h.state().ui.workspace.state(Win::Playlist).maximized);

    // A saved layout comes back: the Mixer open and detached windows put back by Into workspace.
    h.state_mut().show_window(Win::Mixer);
    h.step();
    let saved = h.state().ui.workspace.save_layout();
    h.state_mut().ui.workspace.close_all();
    h.state_mut().ui.workspace.state_mut(Win::PianoRoll).detached = true;
    h.state_mut().ui.workspace.into_workspace();
    assert!(!h.state().ui.workspace.state(Win::PianoRoll).detached);
    h.state_mut().ui.workspace.apply_layout(&saved);
    for _ in 0..2 {
        h.step();
    }
    assert!(h.state().is_window_open(Win::Mixer) && h.state().is_window_open(Win::Playlist));
    let json = serde_json::to_string(&saved).expect("json");
    assert_eq!(
        serde_json::from_str::<limeshot_ui::workspace::SavedLayout>(&json).expect("back"),
        saved
    );
}

/// APP-025: FL Studio's settings window — one window, a tab per page, opened on its page from
/// the Options menu (and F11 for Project info); changes apply at once.
#[test]
fn the_settings_window_opens_on_its_page_and_edits_project_info_in_one_undo_step() {
    use limeshot_ui::settings_window::Page;
    let mut h = harness(true);
    let page = |h: &Harness<'static, LimeShotApp>| h.state().ui.settings.as_ref().map(|s| s.page);
    h.get_by_label("Options").click();
    h.step();
    h.get_by_label_contains("General settings").click();
    h.step();
    assert_eq!(page(&h), Some(Page::General));
    assert!(h.query_by_label_contains("Settings – System / miscellaneous").is_some());
    // The tabs switch pages (the File tab, not the File menu).
    h.get_all_by_label("File")
        .find(|n| n.rect().top() > 100.0)
        .expect("File tab")
        .click();
    h.step();
    h.step();
    assert_eq!(page(&h), Some(Page::File));
    assert!(h.query_by_label("Autosave unsaved changes").is_some());

    // F11: Project info. Typing in Author edits the project; the whole edit is one undo step.
    h.key_press(Key::F11);
    h.step();
    h.step();
    assert_eq!(page(&h), Some(Page::Info));
    let author = h.get_by_label("Author").rect();
    click(
        &mut h,
        egui::pos2(author.left() + 40.0, author.bottom() + 12.0),
        PointerButton::Primary,
    );
    h.event(Event::Text("Hodd".into()));
    h.step();
    h.event(Event::Text(" Lime".into()));
    h.step();
    assert_eq!(h.state().project().info.author, "Hodd Lime");
    h.key_press(Key::Escape);
    click(
        &mut h,
        egui::pos2(author.left() + 200.0, author.top() - 40.0),
        PointerButton::Primary,
    );
    h.step();
    assert!(h.state_mut().doc.undo());
    assert_eq!(h.state().project().info.author, "");

    // Help › About shows the About page.
    h.get_by_label("Help").click();
    h.step();
    h.get_by_label_contains("About LimeShot").click();
    h.step();
    assert_eq!(page(&h), Some(Page::About));
}

/// The toolbar's right-click menus as in FL Studio (observed in session 15): tempo presets and
/// the tempo tapper, the song position's format, PAT/SONG showing its window, and the snap list.
#[test]
fn toolbar_menus_set_tempo_presets_time_format_and_show_windows() {
    let mut h = harness(true);
    // Tempo: right-click → 120 BPM.
    let bpm_label = h.get_by_label("BPM").rect();
    click(
        &mut h,
        bpm_label.left_center() - vec2(30.0, 0.0),
        PointerButton::Secondary,
    );
    h.step();
    h.get_by_label("120 BPM").click();
    h.step();
    assert!((h.state().project().tempo.bpm() - 120.0).abs() < 1e-9);
    // The tempo tapper: four taps half a second apart give 120 BPM.
    let mut tapper = limeshot_ui::toolbar::Tapper::default();
    let bpm = [10.0, 10.5, 11.0, 11.5].map(|t| tapper.tap(t));
    assert_eq!(bpm[0], None);
    assert!((bpm[3].expect("bpm") - 120.0).abs() < 1e-9);
    assert_eq!(tapper.tap(20.0), None, "a long pause starts over");
    h.key_press_modifiers(Modifiers::COMMAND, Key::F5);
    h.step();
    assert!(h.state().ui.toolbar.tapper.is_some());

    // Song position: right-click → minutes first.
    let pos = h.get_by_label_contains("B:B:T").rect();
    click(&mut h, pos.center(), PointerButton::Secondary);
    h.step();
    h.get_by_label("Minute : Second : Centisecond").click();
    h.step();
    assert!(h.state().ui.toolbar.time_in_seconds);
    h.step();
    assert!(h.query_by_label_contains("M:S:CS").is_some());

    // PAT/SONG: a right-click shows the Channel Rack in pattern mode.
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    // PAT is the toolbar's first control, at its left end.
    click(&mut h, egui::pos2(34.0, bpm_label.center().y), PointerButton::Secondary);
    assert!(h.state().is_window_open(Win::ChannelRack));

    // FL's snap values, in FL's order.
    let labels: Vec<_> = Snap::ALL.iter().map(|s| s.label()).collect();
    assert_eq!(labels[..3], ["(none)", "1/6 step", "1/4 step"]);
    assert_eq!(labels.last(), Some(&"Bar"));
    assert_eq!(Snap::SixthStep.ticks(TimeSignature::default()), Some(STEP_TICKS / 6));
}

/// SEQ-021: FL Studio's Patterns menu commands and keys (observed live in session 15).
#[test]
fn the_patterns_menu_finds_inserts_moves_splits_and_transposes_patterns() {
    use limeshot_ui::dialogs::Dialog;
    let mut h = harness(true);
    let names = |h: &Harness<'static, LimeShotApp>| {
        h.state()
            .project()
            .patterns
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
    };
    let beat = h.state().project().patterns[0].id;
    h.state_mut().set_current_pattern(beat);
    h.step();
    let before = names(&h);

    // F4: find next empty — none is empty, so a new pattern, then FL's name prompt.
    h.key_press(Key::F4);
    h.step();
    assert_eq!(h.state().project().patterns.len(), before.len() + 1);
    let new = h.state().current_pattern();
    assert!(matches!(h.state().dialog, Some(Dialog::Rename { .. })));
    h.state_mut().dialog = None;
    // Shift+F4 from a pattern with notes finds that empty one (the first empty from the start).
    h.state_mut().set_current_pattern(beat);
    h.key_press_modifiers(Modifiers::SHIFT, Key::F4);
    h.step();
    assert_eq!(h.state().current_pattern(), new);
    h.state_mut().dialog = None;

    // Insert one (menu): before the current pattern, made current, default names renumbered.
    h.state_mut().set_current_pattern(beat);
    h.get_by_label("Patterns").click();
    h.step();
    h.get_by_label_contains("Insert one").click();
    h.step();
    let inserted = h.state().current_pattern();
    assert_eq!(h.state().project().patterns[0].id, inserted);
    assert_eq!(h.state().project().patterns[1].id, beat);
    h.state_mut().dialog = None;
    // Cmd+Shift+↓ moves it down one.
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::ArrowDown);
    h.step();
    assert_eq!(h.state().project().patterns[1].id, inserted);
    assert!(h.state_mut().doc.undo());
    assert_eq!(h.state().project().patterns[0].id, inserted);

    // Split by channel: the demo's Beat has kick, clap and hat steps.
    h.state_mut().set_current_pattern(beat);
    let channels = {
        let p = h.state().project().pattern(beat).expect("beat");
        let mut c: Vec<_> = p.notes.iter().map(|n| n.channel).collect();
        c.sort_unstable();
        c.dedup();
        c.len()
    };
    assert!(channels > 1);
    let count = h.state().project().patterns.len();
    h.get_by_label("Patterns").click();
    h.step();
    h.get_by_label("Split by channel").click();
    h.step();
    assert_eq!(h.state().project().patterns.len(), count + channels - 1);
    let first = h.state().project().channels.iter().find(|c| {
        h.state()
            .project()
            .pattern(beat)
            .is_some_and(|p| p.notes.iter().any(|n| n.channel == c.id))
    });
    assert_eq!(
        h.state().project().pattern(beat).map(|p| p.name.clone()),
        first.map(|c| c.name.clone())
    );
    assert!(h.state_mut().doc.undo());

    // Transpose… moves every note of the pattern.
    let keys = |h: &Harness<'static, LimeShotApp>| {
        h.state()
            .project()
            .pattern(beat)
            .map(|p| p.notes.iter().map(|n| n.key).collect::<Vec<_>>())
    };
    let was = keys(&h).expect("keys");
    h.get_by_label("Patterns").click();
    h.step();
    h.get_by_label("Transpose…").click();
    h.step();
    if let Some(Dialog::Transpose { semitones, .. }) = &mut h.state_mut().dialog {
        *semitones = 3;
    } else {
        panic!("the Transpose dialog");
    }
    h.step();
    h.get_by_label("OK").click();
    h.step();
    assert_eq!(keys(&h).expect("keys"), was.iter().map(|k| k + 3).collect::<Vec<_>>());

    // Select in playlist selects the pattern's clips.
    let clips = h
        .state()
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| c.pattern() == Some(beat))
        .count();
    assert!(clips > 0);
    assert!(h.state().dialog.is_none());
    h.step();
    h.get_by_label("Patterns").click();
    h.step();
    h.step();
    h.get_by_label_contains("Select in playlist").click();
    h.step();
    assert_eq!(h.state().ui.playlist.selection.len(), clips);
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
    let (processors, _keep) = h2.state().plugins.offline(h2.state().project(), 48_000).unwrap();
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
    assert_eq!(h.state().project().channels.len(), n);
    let sampler =
        lime::sampler_params(&h.state().project().channel(ch).expect("channel").instrument).expect("Lime sampler");
    assert_eq!(
        sampler.sample.as_ref().map(|r| std::path::PathBuf::from(&r.path)),
        Some(kick.clone())
    );
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
    assert_eq!(h.state().project().channels.len(), n);

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

/// Drawn/moved notes have timed releases; held piano keys retain the mouse-release safety net.
#[test]
fn drawing_auditions_have_a_duration_and_keyboard_auditions_are_held() {
    let (mut h, pat, ch) = empty_piano_roll();
    let g = piano_geometry(&h);
    let p = g.pos(STEP_TICKS * 2, 60);
    h.event(Event::PointerMoved(p));
    h.step();
    press(&mut h, p, PointerButton::Primary, true);
    assert!(
        h.state().auditions.is_empty(),
        "drawn notes use the audio clock, not the mouse lifetime"
    );
    // Dragged up two keys: the first audition ends, the new key sounds.
    h.event(Event::PointerMoved(g.pos(STEP_TICKS * 2, 62)));
    h.step();
    assert!(h.state().auditions.is_empty());
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
    // The waveform is the 64 px strip under the sample's name and Load… button.
    let load = h.get_by_label("Load…").rect();
    let wave = egui::pos2(load.left(), load.bottom() + 36.0);
    h.event(Event::PointerMoved(wave));
    h.step();
    press(&mut h, wave, PointerButton::Primary, true);
    assert_eq!(h.state().sampler_preview.map(|p| p.0), Some(ch));
    assert!(
        h.state().auditions.is_empty(),
        "sampler preview is independent of mouse release"
    );
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

/// Owner's list item 18: FL Studio's hint panel (top left) says what LimeShot is doing and what
/// the control under the pointer does, with its shortcut.
#[test]
fn the_hint_panel_describes_the_control_under_the_pointer() {
    // Hints and transport need no samples. Demo loading legitimately overrides the stopped
    // activity until its worker finishes, which is unrelated to the controls tested here.
    let mut h = harness(false);
    let r = h.state().ui.hint_rect.expect("hint panel in the header");
    assert!(r.width() > 200.0 && r.top() < 30.0, "{r:?}");
    assert_eq!(limeshot_ui::toolbar::activity(h.state()), "Stopped · Pattern");
    // A toolbar button: its description, with the shortcut split off.
    let play = h.get_by_label("Play / pause (Space)").rect().center();
    h.event(Event::PointerMoved(play));
    h.step();
    h.step();
    assert_eq!(h.state().ui.hint_shown, "Play / pause");
    // An editor's own description: the Playlist's position and tool help.
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    h.event(Event::PointerMoved(g.pos(10, 12)));
    h.step();
    h.step();
    assert!(
        h.state().ui.hint_shown.starts_with("1:01:"),
        "{}",
        h.state().ui.hint_shown
    );
    // While playing the panel says so.
    h.state_mut().toggle_play();
    wait_for(&mut h, "playback", |a| a.is_playing());
    assert!(limeshot_ui::toolbar::activity(h.state()).starts_with("Playing"));
}

/// Press at `from`, hold until egui counts it as a drag, then move by `d` in 1 px steps.
fn drag_px<S>(h: &mut Harness<'_, S>, from: Pos2, d: egui::Vec2, release: bool) {
    h.event(Event::PointerMoved(from));
    h.step();
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    for _ in 0..4 {
        h.step();
    }
    let n = d.length().round() as i32;
    for i in 1..=n {
        h.event(Event::PointerMoved(from + d.normalized() * i as f32));
        h.step();
    }
    if release {
        h.event(Event::PointerButton {
            pos: from + d,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        });
        h.step();
    }
}

/// Owner's list item 19 (FL Studio, observed live: a drag off the centre moves at once, one
/// across it holds "Centered" for several pixels): dials and sliders catch at their default
/// and let go after a few pixels; from the default, small moves are free.
#[test]
fn dials_and_sliders_catch_at_their_default() {
    use limeshot_ui::widgets::{DETENT_PX, knob, reset_on_double_click};
    // A pan knob: −1…1 over 120 px, so 1 px is 1/60. State: (value, the knob's rect).
    let mut h = Harness::new_ui_state(
        |ui, s: &mut (f32, egui::Rect)| {
            s.1 = knob(ui, &mut s.0, -1.0, 1.0, 0.0, 40.0, true, |_| String::new()).rect;
        },
        (0.0f32, egui::Rect::NOTHING),
    );
    h.step();
    let at = h.state().1.center();
    drag_px(&mut h, at, vec2(0.0, -2.0), true);
    assert!(
        (h.state().0 - 2.0 / 60.0).abs() < 1e-4,
        "free from the default: {}",
        h.state().0
    );
    // From 0.2 down across the centre: 12 px reach it, it holds for DETENT_PX more.
    h.state_mut().0 = 0.2;
    h.step();
    drag_px(&mut h, at, vec2(0.0, 12.0 + DETENT_PX - 1.0), true);
    assert_eq!(h.state().0, 0.0, "caught");
    h.state_mut().0 = 0.2;
    h.step();
    drag_px(&mut h, at, vec2(0.0, 12.0 + DETENT_PX + 3.0), true);
    assert!(
        (h.state().0 + 3.0 / 60.0).abs() < 1e-4,
        "let go past the detent: {}",
        h.state().0
    );

    // An egui slider (Channel Settings, the Mixer's panel), default 0.5: its value follows the
    // pointer, and holds at 0.5 for DETENT_PX past it.
    let mut h = Harness::new_ui_state(
        |ui, s: &mut (f32, egui::Rect)| {
            ui.spacing_mut().slider_width = 200.0;
            let r = ui.add(egui::Slider::new(&mut s.0, 0.0..=1.0).show_value(false));
            s.1 = r.rect;
            reset_on_double_click(r, &mut s.0, 0.5);
        },
        (0.2f32, egui::Rect::NOTHING),
    );
    h.step();
    let rail = h.state().1;
    let from = egui::pos2(rail.left() + 0.2 * rail.width(), rail.center().y);
    let x_half = rail.left() + 0.5 * rail.width();
    drag_px(&mut h, from, vec2(x_half - from.x + DETENT_PX - 2.0, 0.0), false);
    assert_eq!(h.state().0, 0.5, "the slider caught at its default");
    drag_px(
        &mut h,
        from + vec2(x_half - from.x + DETENT_PX - 2.0, 0.0),
        vec2(12.0, 0.0),
        true,
    );
    assert!(h.state().0 > 0.52, "and lets go further on: {}", h.state().0);
}

#[test]
fn a_plugin_channel_button_toggles_the_plugin_window_with_its_tabs() {
    use limeshot_ui::plugin_window::{self, Tab};
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").id;
    let target = Target::Channel(ch);
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().plugins.is_window_open(target));

    // FL Studio: the channel button opens the plugin's window (not Channel Settings).
    let button = h.state().ui.rack.step_pos(ch, 0).expect("strip") - vec2(60.0, 0.0);
    click(&mut h, button, PointerButton::Primary);
    assert!(h.state().plugins.is_window_open(target));
    assert!(!h.state().is_window_open(Win::ChannelSettings));
    for _ in 0..3 {
        h.step();
    }
    let w = h.state().plugins.window(target).expect("window");
    assert_eq!(w.tab, Tab::Editor);
    assert!(!w.has_editor(), "Lime Test Tone has no editor");
    assert!(
        h.query_by_label_contains("no editor of its own").is_some(),
        "the Plugin tab lists the parameters instead"
    );
    assert!(h.query_by_label("Level").is_some());

    // The Settings tab is the channel's Channel Settings.
    h.get_by_label("Settings").click();
    for _ in 0..3 {
        h.step();
    }
    assert_eq!(h.state().plugins.window(target).map(|w| w.tab), Some(Tab::Settings));
    assert!(h.query_by_label("Root").is_some(), "Channel Settings: root key");

    // Closed and opened again, it comes back on the tab it was on.
    plugin_window::toggle(h.state_mut(), target);
    h.step();
    assert!(!h.state().plugins.is_window_open(target));
    plugin_window::toggle(h.state_mut(), target);
    h.step();
    assert_eq!(h.state().plugins.window(target).map(|w| w.tab), Some(Tab::Settings));

    // A plugin changed while its window was open marks the project modified on close, even if the
    // plugin did not report it.
    h.state_mut().doc.mark_saved();
    h.state_mut().plugins.set_param(target, 0, 0.125);
    assert!(!h.state().doc.is_modified());
    plugin_window::close(h.state_mut(), target);
    assert!(h.state().doc.is_modified());

    // Deleting the channel closes its window.
    h.state_mut().plugins.open_window(target, None);
    h.step();
    h.state_mut().exec(Command::RemoveChannel(ch));
    h.step();
    assert!(!h.state().plugins.is_window_open(target));
}

#[test]
fn plugin_window_presets_step_and_the_window_place_is_saved_in_the_project() {
    use limeshot_ui::plugin_window::{Preset, Tab};
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").id;
    let target = Target::Channel(ch);
    h.state_mut().plugins.open_window(target, None);
    for _ in 0..3 {
        h.step();
    }
    // The presets menu lists the plugin's factory programs; the current one is selected.
    let w = h.state().plugins.window(target).expect("window");
    let names: Vec<String> = w.presets().iter().map(Preset::name).collect();
    assert_eq!(names[..3], ["Medium", "Quiet", "Loud"]);
    assert_eq!(w.current_preset(), Some(0));

    // FL Studio's Presets ▶ loads the next program and marks the project modified.
    h.state_mut().doc.mark_saved();
    h.get_by_label("▶").click();
    for _ in 0..2 {
        h.step();
    }
    assert_eq!(
        h.state().plugins.window(target).and_then(|w| w.current_preset()),
        Some(1)
    );
    assert_eq!(
        h.state().plugins.instance(target).map(|i| i.param(0)),
        Some(0.25),
        "Quiet"
    );
    assert!(h.state().doc.is_modified());
    // ◀ twice wraps round to the last entry.
    h.get_by_label("◀").click();
    h.step();
    h.get_by_label("◀").click();
    h.step();
    let last = h.state().plugins.window(target).map(|w| w.presets().len() - 1);
    assert_eq!(h.state().plugins.window(target).and_then(|w| w.current_preset()), last);
    // The menu picks one by name.
    h.get_by_label_contains("⏷").click();
    h.step();
    h.get_by_label("Loud").click();
    for _ in 0..2 {
        h.step();
    }
    assert_eq!(h.state().plugins.instance(target).map(|i| i.param(0)), Some(1.0));

    // The keyboard toggle switches on and off.
    h.get_by_label("⌨").click();
    h.step();
    assert_eq!(
        h.state().plugins.window(target).map(|w| w.keyboard_to_plugin),
        Some(true)
    );

    // Saving keeps the window's place, tab and toggle; reopening the project reopens it.
    h.get_by_label("Settings").click();
    for _ in 0..3 {
        h.step();
    }
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("windows.lime");
    assert!(h.state_mut().save_to(&path));
    let Instrument::Plugin(saved) = &h.state().project().channel(ch).expect("ch").instrument else {
        panic!("plugin channel");
    };
    let place = saved.window.clone().expect("window saved");
    assert!(place.open);
    assert_eq!(place.tab, "settings");
    assert!(place.keyboard_to_plugin);

    let mut h2 = harness(false);
    h2.state_mut().open_path(&path);
    for _ in 0..3 {
        h2.step();
    }
    let w = h2.state().plugins.window(target).expect("reopened with the project");
    assert_eq!(w.tab, Tab::Settings);
    assert!(w.keyboard_to_plugin);
}

#[test]
fn a_plugin_parameter_is_automated_from_its_menu_and_as_the_last_tweaked_parameter() {
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").id;
    let target = Target::Channel(ch);
    h.state_mut().plugins.open_window(target, None);
    for _ in 0..3 {
        h.step();
    }
    h.state_mut().plugins.set_param(target, 0, 0.25);
    h.step();
    // Right-click on the Level slider: Create automation clip, flat at the plugin's value.
    let label = h.get_by_label("Level").rect();
    let slider = egui::pos2(label.left() + 140.0, label.center().y);
    click(&mut h, slider, PointerButton::Secondary);
    h.step();
    assert_eq!(
        h.state().plugins.instance(target).map(|i| i.param(0)),
        Some(0.25),
        "a right-click does not move the slider"
    );
    h.get_by_label("Create automation clip").click();
    h.step();
    let a = h.state().project().automations.last().expect("automation").clone();
    assert_eq!(a.target, AutomationTarget::PluginParam { plugin: target, id: 0 });
    assert_eq!(a.name, "Level – Lime Test Tone");
    assert!((a.points[0].value - 0.25).abs() < 1e-6);
    assert!(
        h.state()
            .project()
            .playlist
            .clips
            .iter()
            .any(|c| c.automation() == Some(a.id))
    );

    // FL Studio's Add › Automation for last tweaked parameter: off until a parameter changes.
    assert_eq!(h.state().plugins.last_tweaked, None);
    h.state_mut().plugins.open_window(target, None);
    h.step();
    h.step();
    let label = h.get_by_label("Level").rect();
    let slider = egui::pos2(label.left() + 140.0, label.center().y);
    click(&mut h, slider, PointerButton::Primary);
    assert_eq!(h.state().plugins.last_tweaked, Some((target, 0)));
    let before = h.state().project().automations.len();
    h.get_by_label("Add").click();
    h.step();
    h.get_by_label("Automation for last tweaked parameter").click();
    h.step();
    assert_eq!(h.state().project().automations.len(), before + 1);

    // During Song playback the plugin's own control follows the curve (the second automation of
    // the same parameter removed first: the later one would win).
    let second = h.state().project().automations.last().map(|a| a.id).expect("second");
    h.state_mut().exec(Command::RemoveAutomation(second));
    let id = a.id;
    h.state_mut().exec(Command::SetAutomationPoints {
        automation: id,
        points: vec![
            AutomationPoint {
                tick: 0,
                value: 0.0,
                tension: 0.0,
                mode: Default::default(),
            },
            AutomationPoint {
                tick: 960,
                value: 1.0,
                tension: 0.0,
                mode: Default::default(),
            },
        ],
    });
    let project = h.state().project().clone();
    let snap = limeshot_sequencer::compile(&project, project.patterns[0].id, &limeshot_sequencer::NoSamples);
    h.state_mut().plugins.follow_automation(&project, &snap, 480);
    let v = h.state().plugins.instance(target).expect("plugin").param(0);
    assert!((v - 0.5).abs() < 1e-3, "{v}");
}

/// A right-click on an effect's slider opens its automation menu and leaves the value alone (FL
/// Studio). egui sliders sense only drags, so the menu never opened before session 15.
#[test]
fn right_click_on_an_effect_slider_offers_automation_without_moving_it() {
    let mut h = mixer_harness(1);
    let fx = h
        .state_mut()
        .exec(Command::AddEffect {
            track: 1,
            kind: EffectKind::Delay,
        })
        .expect("effect");
    h.step();
    h.step();
    let spec = EffectKind::Delay.params()[0];
    let target = PluginTarget::Effect { track: 1, effect: fx };
    let id = lime::param_id(0);
    h.state_mut()
        .plugins
        .open_window(target, Some(limeshot_ui::plugin_window::Tab::Settings));
    h.run_steps(3);
    let before = h.state().plugins.instance(target).unwrap().param(id);
    let label = h.get_by_label(spec.name).rect();
    click(
        &mut h,
        egui::pos2(label.left() + 140.0, label.center().y),
        PointerButton::Secondary,
    );
    h.step();
    assert_eq!(h.state().plugins.instance(target).unwrap().param(id), before);
    h.get_by_label("Create automation clip").click();
    h.step();
    let a = h.state().project().automations.last().expect("automation");
    assert_eq!(a.target, AutomationTarget::PluginParam { plugin: target, id });
}

/// PLG-010: a plugin set to run in a separate process survives as a silent slot when its process
/// dies, and Reload brings it back.
#[cfg(unix)]
#[test]
fn a_plugin_in_its_own_process_crashes_without_taking_limeshot_down() {
    use limeshot_plugins::PluginInstance;
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.bridge_exe = Some(env!("CARGO_BIN_EXE_limeshot").into());
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let ch = h.state().project().channels.last().expect("channel").id;
    let target = Target::Channel(ch);
    h.state_mut().plugins.set_param(target, 0, 0.25);
    h.state_mut()
        .plugins
        .open_window(target, Some(limeshot_ui::plugin_window::Tab::Settings));
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().plugins.instance(target).expect("plugin").is_bridged());
    h.get_by_label("Run in a separate process").click();
    for _ in 0..3 {
        h.step();
    }
    let inst = h
        .state()
        .plugins
        .instance(target)
        .unwrap_or_else(|| panic!("plugin in its own process: {:?}", h.state().plugins.error(target)));
    assert!(inst.is_bridged());
    assert_eq!(inst.param(0), 0.25, "the state moved with it");
    let Instrument::Plugin(p) = &h.state().project().channel(ch).expect("ch").instrument else {
        panic!("plugin channel");
    };
    assert!(p.separate_process, "saved with the project");

    // The plugin's process dies: LimeShot reports it and keeps running.
    let PluginInstance::Bridged(b) = h.state().plugins.instance(target).expect("plugin") else {
        panic!("bridged");
    };
    let pid = b.process_id();
    assert!(
        std::process::Command::new("kill")
            .arg("-KILL")
            .arg(pid.to_string())
            .status()
            .expect("kill")
            .success()
    );
    std::thread::sleep(std::time::Duration::from_millis(100));
    for _ in 0..3 {
        h.step();
    }
    assert!(h.state().plugins.has_crashed(target));
    assert!(h.state().toasts.iter().any(|t| t.text.contains("crashed")));
    assert!(h.query_by_label_contains("The plugin crashed").is_some());

    // Reload starts it again.
    h.get_by_label("Reload").click();
    for _ in 0..4 {
        h.step();
    }
    let inst = h.state().plugins.instance(target).expect("reloaded");
    assert!(inst.is_bridged() && !inst.crashed());
    assert!(!h.state().plugins.has_crashed(target));

    // Undo puts it back into LimeShot's process.
    h.state_mut().doc.undo();
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().plugins.instance(target).expect("plugin").is_bridged());
}

#[test]
fn a_plugin_insert_with_an_editor_opens_its_window_from_the_slot() {
    use limeshot_ui::plugin_window::Tab;
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
    menu_path(&mut h, &["+ Add effect", "Plugins", "LimeShot", "Lime Test Gain"]);
    let fx = h.state().project().mixer.tracks[1].effects.clone();
    let slot = fx.len();
    let target = Target::Effect {
        track: 1,
        effect: fx[slot - 1].id,
    };
    h.step();
    // Adding an effect opens its editor immediately.
    assert!(h.state().plugins.is_window_open(target));
    for _ in 0..3 {
        h.step();
    }
    assert!(h.state().plugins.is_window_open(target));
    limeshot_ui::plugin_window::toggle(h.state_mut(), target);
    h.step();
    // FL Studio: a click on the slot opens the plugin's window; a second click closes it.
    h.get_by_label(&format!("{slot}. Lime Test Gain")).click();
    for _ in 0..3 {
        h.step();
    }
    let w = h.state().plugins.window(target).expect("window");
    assert_eq!(w.tab, Tab::Editor);
    assert!(w.has_editor(), "Lime Test Gain has an editor");
    assert!(!w.shows_native_editor(), "no native window in the headless harness");
    assert!(h.query_by_label_contains("cannot be shown here").is_some());
    h.get_by_label(&format!("{slot}. Lime Test Gain")).click();
    h.step();
    assert!(!h.state().plugins.is_window_open(target));
}

/// Open the Piano Roll's ⏵ menu, then `submenu` (hovered, as a person would), then click `item`.
fn piano_menu(h: &mut Harness<'static, LimeShotApp>, submenu: &str, item: &str) {
    let button = h.state().ui.piano.menu_button;
    editor_menu(h, button, submenu, item);
}

/// Click an editor's ⏵ menu button (its rectangle from the editor's state), then
/// `submenu` › `item`.
fn editor_menu(h: &mut Harness<'static, LimeShotApp>, button: Option<egui::Rect>, submenu: &str, item: &str) {
    let at = button.expect("the editor's ⏵ menu").center();
    click(h, at, PointerButton::Primary);
    open_item(h, submenu, item);
}

fn playlist_menu(h: &mut Harness<'static, LimeShotApp>, submenu: &str, item: &str) {
    let button = h.state().ui.playlist.menu_button;
    editor_menu(h, button, submenu, item);
}

fn rack_menu(h: &mut Harness<'static, LimeShotApp>, submenu: &str, item: &str) {
    let button = h.state().ui.rack.menu_button;
    editor_menu(h, button, submenu, item);
}

fn open_item(h: &mut Harness<'static, LimeShotApp>, submenu: &str, item: &str) {
    h.get_by_label(&format!("{submenu} ⏵")).hover();
    for _ in 0..3 {
        h.step();
    }
    // Menu items carry their shortcut in the label ("Quick legato ⌘L").
    let with_key = format!("{item} ");
    h.get_by(|n| {
        n.label()
            .is_some_and(|l| l.trim_end() == item || l.strip_prefix(&with_key).is_some_and(|k| !k.contains(' ')))
    })
    .click();
    h.step();
    h.step();
}

fn shift_digit(h: &mut Harness<'static, LimeShotApp>, digit: Key, typed: Key) {
    // QWERTZ: Shift+1 types "!", so the logical key differs from the physical one.
    for pressed in [true, false] {
        h.event(Event::Key {
            key: typed,
            physical_key: Some(digit),
            pressed,
            repeat: false,
            modifiers: Modifiers::SHIFT,
        });
    }
    h.step();
}

#[test]
fn piano_roll_menu_tools_select_and_zoom() {
    let (mut h, pat, lead) = empty_piano_roll();
    let add = |key: u8, start: Tick, length: Tick| NewNote {
        channel: lead,
        key,
        start,
        length,
        velocity: 0.8,
    };
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: vec![add(60, 0, 100), add(64, 0, 100), add(67, 960, 100), add(72, 1920, 100)],
    });
    h.step();
    // Tools › Quick legato from the ⏵ menu: every note reaches the next start.
    piano_menu(&mut h, "Tools", "Quick legato");
    assert_eq!(
        notes_by_start(&h, pat, lead),
        vec![
            (60, 0, 960, false),
            (64, 0, 960, false),
            (67, 960, 960, false),
            (72, 1920, 100, false)
        ]
    );
    assert_eq!(h.state().doc.undo_label(), Some("Quick legato"));
    // Select › Select stacked notes: the chord.
    piano_menu(&mut h, "Select", "Select stacked notes");
    let ids = |h: &Harness<'static, LimeShotApp>, keys: &[u8]| -> Vec<NoteId> {
        let mut v: Vec<NoteId> = pattern_notes(h, pat, lead)
            .iter()
            .filter(|n| keys.contains(&n.key))
            .map(|n| n.id)
            .collect();
        v.sort();
        v
    };
    let mut sel = h.state().ui.piano.selection.clone();
    sel.sort();
    assert_eq!(sel, ids(&h, &[60, 64]));
    // Shift+D discards the selection's lengths (one step).
    h.key_press_modifiers(Modifiers::SHIFT, Key::D);
    h.step();
    assert_eq!(
        h.state().ui.piano.tool,
        limeshot_ui::piano_roll::Tool::Draw,
        "Shift+D is not the Delete tool"
    );
    assert!(
        pattern_notes(&h, pat, lead)
            .iter()
            .all(|n| (n.length == STEP_TICKS) == [60, 64].contains(&n.key))
    );
    // Shift+O: every other note of the channel (none selected after Cmd+D).
    h.key_press_modifiers(Modifiers::COMMAND, Key::D);
    h.key_press_modifiers(Modifiers::SHIFT, Key::O);
    h.step();
    let mut sel = h.state().ui.piano.selection.clone();
    sel.sort();
    assert_eq!(sel, ids(&h, &[60, 67]));
    // Shift+R picks one, Shift+M adds another.
    h.key_press_modifiers(Modifiers::SHIFT, Key::R);
    h.step();
    assert_eq!(h.state().ui.piano.selection.len(), 1);
    h.key_press_modifiers(Modifiers::SHIFT, Key::M);
    h.step();
    assert_eq!(h.state().ui.piano.selection.len(), 2);
    // Shift+Q quantizes starts only; Cmd+L is quick legato.
    let top = ids(&h, &[72]);
    h.state_mut().exec(Command::MoveNotes {
        pattern: pat,
        notes: top,
        dt: 50,
        dkey: 0,
    });
    h.key_press_modifiers(Modifiers::COMMAND, Key::D);
    h.key_press_modifiers(Modifiers::SHIFT, Key::Q);
    h.step();
    assert!(
        pattern_notes(&h, pat, lead)
            .iter()
            .any(|n| n.key == 72 && n.start == 1920 && n.length == 100)
    );
    // Zoom: PgUp zooms in, Shift+4 fits the pattern, Shift+1 (typed "!") one bar.
    let z = h.state().ui.piano.zoom_x;
    h.key_press(Key::PageUp);
    h.step();
    assert!(h.state().ui.piano.zoom_x > z);
    shift_digit(&mut h, Key::Num4, Key::Num4);
    let g = piano_geometry(&h);
    let bar = h.state().project().time_signature.ticks_per_bar();
    let len = h
        .state()
        .project()
        .pattern(pat)
        .map(|p| p.length(h.state().project().time_signature))
        .unwrap_or(bar);
    assert!(
        (g.ppt * len as f32 - g.grid.width()).abs() < 2.0,
        "the pattern fills the grid"
    );
    shift_digit(&mut h, Key::Num1, Key::Exclamationmark);
    let g = piano_geometry(&h);
    assert!(
        (g.ppt * bar as f32 - g.grid.width()).abs() < 2.0,
        "one bar fills the grid"
    );
    // Zoom › Zoom on selection from the menu.
    let mid = ids(&h, &[60, 67]);
    h.state_mut().ui.piano.selection = mid;
    piano_menu(&mut h, "Zoom", "Zoom on selection");
    let g = piano_geometry(&h);
    assert!(
        (g.ppt * 1920.0 - g.grid.width()).abs() < 2.0,
        "the selected notes fill the grid"
    );
}

/// PR-015: the tool dialogs preview on the notes; Accept keeps one undo step, closing reverts.
#[test]
fn piano_roll_tool_dialogs_preview_accept_and_cancel() {
    use limeshot_ui::piano_tools::{Kind, Params};
    let (mut h, pat, lead) = empty_piano_roll();
    let chord: Vec<NewNote> = [60, 64, 67]
        .into_iter()
        .map(|key| NewNote {
            channel: lead,
            key,
            start: 0,
            length: PPQ,
            velocity: 0.8,
        })
        .collect();
    h.state_mut().exec(Command::AddNotes {
        pattern: pat,
        notes: chord,
    });
    h.step();
    let before = notes_by_start(&h, pat, lead);
    // Opt+S opens the strumizer; its default spreads the chord at once.
    h.key_press_modifiers(Modifiers::ALT, Key::S);
    h.step();
    assert_eq!(
        h.state().ui.piano.tool_dialog.as_ref().map(|d| d.kind),
        Some(Kind::Strum)
    );
    h.get_by_label("Piano roll - strumizer");
    let starts: Vec<Tick> = notes_by_start(&h, pat, lead).iter().map(|n| n.1).collect();
    assert_eq!(starts, vec![0, PPQ / 8, PPQ / 4], "previewed");
    // A change replaces the preview: still one step away from the original.
    if let Some(Params::Strum(s)) = h.state_mut().ui.piano.tool_dialog.as_mut().map(|d| &mut d.params) {
        s.time = 0.5;
    }
    h.get_by_label("Reset").click();
    h.step();
    h.get_by_label("Accept").click();
    h.step();
    assert!(h.state().ui.piano.tool_dialog.is_none());
    assert_eq!(h.state().doc.undo_label(), Some("Strum"));
    h.state_mut().doc.undo();
    assert_eq!(notes_by_start(&h, pat, lead), before, "Accept is one undo step");
    // Arpeggiate from the ⏵ menu, then Esc: the notes come back.
    piano_menu(&mut h, "Tools", "Arpeggiate…");
    assert!(notes_by_start(&h, pat, lead).len() > 3, "the chord became an arpeggio");
    h.key_press(Key::Escape);
    h.step();
    assert!(h.state().ui.piano.tool_dialog.is_none());
    assert_eq!(notes_by_start(&h, pat, lead), before, "closing reverts the preview");
    // Alt+Q opens the quantizer.
    h.key_press_modifiers(Modifiers::ALT, Key::Q);
    h.step();
    assert_eq!(
        h.state().ui.piano.tool_dialog.as_ref().map(|d| d.kind),
        Some(Kind::Quantize)
    );
    h.get_by_label("Accept").click();
    h.step();
    assert_eq!(
        notes_by_start(&h, pat, lead),
        before,
        "already on the grid: nothing to do"
    );
}

/// FL's Playlist menu: Edit (mute, insert/delete space, merge), Select, Zoom, Time markers.
#[test]
fn playlist_menu_edit_select_zoom_and_markers() {
    let mut h = playlist_harness();
    let bar = h.state().project().time_signature.ticks_per_bar();
    let spans = |h: &Harness<'static, LimeShotApp>| -> Vec<(u32, Tick, Tick, bool)> {
        let mut v: Vec<_> = h
            .state()
            .project()
            .playlist
            .clips
            .iter()
            .map(|c| (c.track, c.start, c.length, c.muted))
            .collect();
        v.sort_by_key(|c| (c.0, c.1));
        v
    };
    let first = h.state().project().playlist.clips[0].clone();
    // Alt+M mutes the selection, Shift+Alt+M unmutes; Select muted finds it.
    h.state_mut().ui.playlist.selection = vec![first.id];
    h.key_press_modifiers(Modifiers::ALT, Key::M);
    h.step();
    assert!(
        h.state()
            .project()
            .playlist
            .clips
            .iter()
            .any(|c| c.id == first.id && c.muted)
    );
    h.state_mut().ui.playlist.selection.clear();
    playlist_menu(&mut h, "Select", "Select muted");
    assert_eq!(h.state().ui.playlist.selection, vec![first.id]);
    h.key_press_modifiers(Modifiers::ALT | Modifiers::SHIFT, Key::M);
    h.step();
    assert!(h.state().project().playlist.clips.iter().all(|c| !c.muted));
    // Insert space (Cmd+Insert) over a one-bar time selection at bar 1, then Delete space.
    let before = spans(&h);
    h.state_mut().ui.playlist.time_sel = Some((bar, 2 * bar));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Insert);
    h.step();
    let moved = spans(&h);
    assert!(
        before
            .iter()
            .zip(&moved)
            .all(|(a, b)| if a.1 >= bar { b.1 == a.1 + bar } else { b.1 == a.1 }),
        "{before:?} → {moved:?}"
    );
    playlist_menu(&mut h, "Edit", "Delete space");
    assert_eq!(spans(&h), before);
    // Select time around selection (Cmd+Enter), then Shift+5 zooms on it.
    h.state_mut().ui.playlist.selection = vec![first.id];
    h.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    h.step();
    assert_eq!(h.state().ui.playlist.time_sel, Some((first.start, first.end())));
    h.event(Event::Key {
        key: Key::Num5,
        physical_key: Some(Key::Num5),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::SHIFT,
    });
    h.step();
    h.step();
    let g2 = h.state().ui.playlist.geometry.expect("playlist drawn");
    assert!((g2.ppt * first.length as f32 - g2.grid.width()).abs() < 2.0 || g2.ppt * PPQ as f32 >= 399.0);
    // Merge the selected pattern clips (Cmd+G): one clip on the topmost track.
    let pats: Vec<ClipId> = h
        .state()
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| c.pattern().is_some())
        .map(|c| c.id)
        .take(2)
        .collect();
    let count = h.state().project().playlist.clips.len();
    h.state_mut().ui.playlist.selection = pats;
    h.key_press_modifiers(Modifiers::COMMAND, Key::G);
    h.step();
    assert_eq!(h.state().project().playlist.clips.len(), count - 1);
    assert_eq!(h.state().ui.playlist.selection.len(), 1);
    assert_eq!(h.state().doc.undo_label(), Some("Merge clips"));
    // Time markers › Add auto, then Delete (the one at the position).
    h.state_mut().ui.playlist.time_sel = None;
    playlist_menu(&mut h, "Time markers", "Add auto");
    assert_eq!(h.state().project().playlist.markers.len(), 1);
    playlist_menu(&mut h, "Time markers", "Delete");
    assert!(h.state().project().playlist.markers.is_empty());
}

/// FL's Mixer track menu: move right / left, colour, solo, delete and insert, with the keys.
#[test]
fn mixer_track_menu_moves_colors_deletes_and_inserts() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    h.state_mut().ui.focus = limeshot_ui::app::Focus::Mixer;
    h.state_mut().ui.mixer.selected = 1;
    for _ in 0..3 {
        h.step();
    }
    let kick = h.state().project().channels[0].id;
    let routed = |h: &Harness<'static, LimeShotApp>| h.state().project().channel(kick).map(|c| c.mixer_track);
    let was = routed(&h).expect("kick");
    h.state_mut().ui.mixer.selected = was;
    h.step();
    let name = h.state().project().mixer.tracks[usize::from(was)].name.clone();
    // ⏵ › Move right: the track and its channel move one place, the selection follows.
    let panel = h.state().ui.mixer.panel_name.expect("track panel");
    let at = h
        .get_all_by_label("⏵")
        .map(|n| {
            let c = n.rect().center();
            Pos2::new(c.x, c.y)
        })
        .min_by(|a, b| a.distance(panel.center()).total_cmp(&b.distance(panel.center())))
        .expect("menu");
    click(&mut h, at, PointerButton::Primary);
    h.get_by(|n| n.label().is_some_and(|l| l.starts_with("Move right")))
        .click();
    h.step();
    assert_eq!(routed(&h), Some(was + 1));
    assert_eq!(h.state().ui.mixer.selected, was + 1);
    assert_eq!(h.state().project().mixer.tracks[usize::from(was + 1)].name, name);
    // Alt+← moves it back.
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowLeft);
    h.step();
    assert_eq!(routed(&h), Some(was));
    // S solos, again unsolos.
    h.key_press(Key::S);
    h.step();
    assert!(h.state().project().mixer.tracks[usize::from(was)].solo);
    h.key_press(Key::S);
    h.step();
    // Delete: its channel plays on the master; Insert one puts an empty track after the selection.
    let n = h.state().project().mixer.tracks.len();
    h.key_press(Key::Delete);
    h.step();
    assert_eq!(h.state().project().mixer.tracks.len(), n - 1);
    assert_eq!(routed(&h), Some(0));
    limeshot_ui::mixer_menu::run(h.state_mut(), limeshot_ui::mixer_menu::Act::InsertOne);
    h.step();
    assert_eq!(h.state().project().mixer.tracks.len(), n);
    limeshot_ui::mixer_menu::run(h.state_mut(), limeshot_ui::mixer_menu::Act::RandomColor);
    let sel = h.state().ui.mixer.selected;
    assert!(h.state().project().mixer.tracks[usize::from(sel)].color.is_some());
}

/// FL's Channel Rack options menu: sort, mute / unmute, clone (Opt+C), transpose and colour the
/// selected channels.
#[test]
fn channel_rack_menu_acts_on_the_selected_channels() {
    let mut h = harness(true);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.step();
    let names = |h: &Harness<'static, LimeShotApp>| -> Vec<String> {
        h.state().project().channels.iter().map(|c| c.name.clone()).collect()
    };
    // Sort by › Name.
    rack_menu(&mut h, "Sort by", "Name");
    let mut sorted = names(&h);
    sorted.sort_by_key(|n| n.to_lowercase());
    assert_eq!(names(&h), sorted);
    // Select two channels: mute, unmute, clone (Opt+C), transpose.
    let ids: Vec<ChannelId> = h.state().project().channels.iter().take(2).map(|c| c.id).collect();
    h.state_mut().select_channel(ids[0]);
    h.state_mut().toggle_channel_selection(ids[1]);
    h.step();
    assert_eq!(h.state().selected_channels().len(), 2);
    limeshot_ui::rack_menu::run(h.state_mut(), limeshot_ui::rack_menu::Act::MuteSelected);
    assert!(
        ids.iter()
            .all(|c| h.state().project().channel(*c).is_some_and(|c| c.muted))
    );
    limeshot_ui::rack_menu::run(h.state_mut(), limeshot_ui::rack_menu::Act::UnmuteSelected);
    assert!(
        ids.iter()
            .all(|c| h.state().project().channel(*c).is_some_and(|c| !c.muted))
    );
    let n = h.state().project().channels.len();
    h.key_press_modifiers(Modifiers::ALT, Key::C);
    h.step();
    assert_eq!(h.state().project().channels.len(), n + 2, "Opt+C clones both");
    h.state_mut().select_channel(ids[0]);
    h.state_mut().toggle_channel_selection(ids[1]);
    h.step();
    let pat = h.state().current_pattern();
    let keys = |h: &Harness<'static, LimeShotApp>, c: ChannelId| -> Vec<u8> {
        h.state()
            .project()
            .pattern(pat)
            .map(|p| p.notes_for(c).map(|n| n.key).collect())
            .unwrap_or_default()
    };
    let before: Vec<Vec<u8>> = h.state().project().channels.iter().map(|c| keys(&h, c.id)).collect();
    rack_menu(&mut h, "Color selected", "Color selected");
    h.key_press(Key::Escape);
    h.step();
    limeshot_ui::rack_menu::run(h.state_mut(), limeshot_ui::rack_menu::Act::TransposeSelected);
    h.step();
    if let Some(limeshot_ui::dialogs::Dialog::Transpose {
        semitones, channels, ..
    }) = &mut h.state_mut().dialog
    {
        assert_eq!(channels, &ids);
        *semitones = 2;
    } else {
        panic!("the Transpose dialog");
    }
    h.step();
    h.get_by_label("OK").click();
    h.step();
    let chans: Vec<ChannelId> = h.state().project().channels.iter().map(|c| c.id).collect();
    for (c, was) in chans.iter().zip(before) {
        let now = keys(&h, *c);
        let want: Vec<u8> = if ids.contains(c) {
            was.iter().map(|k| k + 2).collect()
        } else {
            was
        };
        assert_eq!(now, want);
    }
}

/// PR-027 / PR-013: a stamp places a chord from the clicked key; scale highlighting is a view
/// setting from the ⏵ menu.
#[test]
fn piano_roll_stamp_places_chords_and_scale_highlighting() {
    use limeshot_core::note_tools::{Scale, Stamp};
    let (mut h, pat, lead) = empty_piano_roll();
    piano_menu(&mut h, "Stamp", "Minor 7");
    assert_eq!(h.state().ui.piano.stamp, Some(Stamp::Chord(11)));
    let g = piano_geometry(&h);
    click(&mut h, g.pos(PPQ / 2 + 10, 62), PointerButton::Primary);
    let keys: Vec<u8> = notes_by_start(&h, pat, lead).iter().map(|n| n.0).collect();
    assert_eq!(keys, vec![62, 65, 69, 72], "D minor 7 from D");
    assert_eq!(h.state().ui.piano.selection.len(), 4);
    h.state_mut().doc.undo();
    assert!(notes_by_start(&h, pat, lead).is_empty(), "one undo step");
    // The header's "Stamp: …" button stops stamping.
    h.get_by(|n| n.label().is_some_and(|l| l.starts_with("Stamp: Minor 7")))
        .click();
    h.step();
    assert_eq!(h.state().ui.piano.stamp, None);
    // View › Scale highlighting › Dorian (key C).
    let button = h.state().ui.piano.menu_button;
    editor_menu(&mut h, button, "View", "Scale highlighting");
    h.get_by_label("Scale highlighting ⏵").hover();
    for _ in 0..3 {
        h.step();
    }
    h.get_by_label("Dorian").click();
    h.step();
    assert_eq!(h.state().ui.piano.scale, Some((0, Scale::Dorian)));
}

/// FX-014 / INS-010: an effect card's Presets menu loads a factory preset in one undo step and
/// saves the user's own; an instrument preset keeps the sampler's sample.
#[test]
fn effect_and_instrument_presets_load_and_save() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut h = harness(true);
    h.state_mut().presets_dir = Some(dir.path().to_path_buf());
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    let track = h
        .state()
        .project()
        .mixer
        .tracks
        .iter()
        .position(|t| {
            t.effects
                .iter()
                .any(|e| e.plugin.as_ref().and_then(LimePlugin::of) == Some(LimePlugin(5)))
        })
        .expect("a delay in the demo") as u16;
    h.state_mut().ui.mixer.selected = track;
    for _ in 0..3 {
        h.step();
    }
    let delay = |h: &Harness<'static, LimeShotApp>| {
        h.state().project().mixer.tracks[usize::from(track)]
            .effects
            .iter()
            .find(|e| e.plugin.as_ref().and_then(LimePlugin::of) == Some(LimePlugin(5)))
            .cloned()
            .expect("delay")
    };
    let values = |h: &Harness<'static, LimeShotApp>| {
        lime::plain_values(delay(h).plugin.as_ref().unwrap(), &EffectKind::Delay.defaults())
    };
    let effect = delay(&h).id;
    h.state_mut().plugins.open_window(
        PluginTarget::Effect { track, effect },
        Some(limeshot_ui::plugin_window::Tab::Settings),
    );
    h.run_steps(3);
    let before = values(&h);
    h.get_by_label("Presets").click();
    h.step();
    h.step();
    h.get_by_label("Slapback").click();
    h.step();
    assert_eq!((values(&h)[0], values(&h)[1]), (90.0, 0.1));
    assert_eq!(h.state().doc.undo_label(), Some("Edit plugin"));
    // Save preset as…: named, stored, then listed after the factory presets.
    h.get_by_label("Presets").click();
    h.step();
    h.step();
    h.get_by_label("Save preset as…").click();
    h.step();
    if let Some(limeshot_ui::dialogs::Dialog::Rename { text, .. }) = &mut h.state_mut().dialog {
        *text = "My slap".into();
    } else {
        panic!("the name dialog");
    }
    h.step();
    h.get_by_label("OK").click();
    h.step();
    assert!(dir.path().join("Delay/My slap.limepreset").exists());
    h.state_mut().doc.undo();
    assert_eq!(values(&h), before);
    h.get_by_label("Presets").click();
    h.step();
    h.step();
    h.get_by_label("My slap").click();
    h.step();
    assert_eq!(values(&h)[0], 90.0, "the user's preset loads");
    // An instrument preset: the sampler keeps its sample.
    use limeshot_project::preset::{Slot, factory};
    let kick = h.state().project().channels[0].clone();
    let was = lime::sampler_params(&kick.instrument).unwrap();
    let reversed = factory(Slot::Sampler)
        .into_iter()
        .find(|p| p.name == "Reversed")
        .expect("factory");
    limeshot_ui::presets_ui::apply(
        h.state_mut(),
        limeshot_ui::presets_ui::Target::Channel(kick.id),
        reversed.content,
    );
    let now = lime::sampler_params(&h.state().project().channel(kick.id).unwrap().instrument).unwrap();
    assert!(now.reverse);
    assert_eq!(now.sample, was.sample);
}

#[test]
fn piano_right_click_deselects_and_shows_red_delete_cursor() {
    let (mut h, pat, ch) = empty_piano_roll();
    let g = piano_geometry(&h);
    click(&mut h, g.pos(PPQ, 60), PointerButton::Primary);
    assert!(!h.state().ui.piano.selection.is_empty());
    let empty = g.pos(PPQ * 2, 64);
    h.event(Event::PointerMoved(empty));
    h.step();
    press(&mut h, empty, PointerButton::Secondary, true);
    assert!(h.state().ui.piano.selection.is_empty());
    assert_eq!(h.state().ui.piano.cursor, Some(limeshot_ui::piano_roll::Tool::Delete));
    press(&mut h, empty, PointerButton::Secondary, false);
    assert_eq!(notes_by_start(&h, pat, ch).len(), 1);
}

#[test]
fn rack_selector_drag_selects_channels_without_moving_the_window() {
    let mut h = harness(true);
    let ids: Vec<_> = h.state().project().channels.iter().take(3).map(|c| c.id).collect();
    h.state_mut().select_channel(ids[0]);
    let before = window_rect(&h, Win::ChannelRack);
    let from = led(&h, ids[0]);
    let to = led(&h, ids[2]);
    drag(&mut h, from, to);
    for id in ids {
        assert!(h.state().ui.channel_selection.contains(&id));
    }
    assert_eq!(before, window_rect(&h, Win::ChannelRack));
}

#[test]
fn typing_keyboard_targets_selected_channel_and_releases_the_original() {
    let mut h = harness(true);
    let a = h.state().project().channels[0].id;
    let b = h.state().project().channels[1].id;
    h.state_mut().keyboard_notes = true;
    h.state_mut().select_channel(a);
    h.event(Event::Key {
        key: Key::Z,
        physical_key: Some(Key::Z),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    assert_eq!(h.state().typing_notes, vec![(Key::Z, a, 48)]);
    h.state_mut().select_channel(b);
    h.event(Event::Key {
        key: Key::Z,
        physical_key: Some(Key::Z),
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    assert!(h.state().typing_notes.is_empty());
    h.event(Event::Key {
        key: Key::Q,
        physical_key: Some(Key::Q),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    assert_eq!(h.state().typing_notes, vec![(Key::Q, b, 60)]);
    h.state_mut().ui.focus = limeshot_ui::app::Focus::ChannelRack;
    h.key_press(Key::ArrowUp);
    h.step();
    assert_eq!(h.state().selected_channel(), Some(a), "arrow keys still reach the rack");
    h.state_mut().keyboard_notes = false;
    h.step();
    assert!(h.state().typing_notes.is_empty());
}

#[test]
fn typing_keyboard_keeps_note_keys_from_tools_and_allows_delete() {
    let (mut h, pat, ch) = melody_piano_roll();
    let note = pattern_notes(&h, pat, ch)[0].id;
    h.state_mut().ui.piano.selection = vec![note];
    h.state_mut().ui.piano.tool = limeshot_ui::piano_roll::Tool::Draw;
    h.state_mut().ui.focus = limeshot_ui::app::Focus::PianoRoll;
    h.state_mut().keyboard_notes = true;
    h.key_down(Key::D);
    h.step();
    assert_eq!(h.state().ui.piano.tool, limeshot_ui::piano_roll::Tool::Draw);
    assert_eq!(h.state().typing_notes, vec![(Key::D, ch, 51)]);
    h.key_press(Key::Delete);
    h.step();
    assert!(pattern_notes(&h, pat, ch).iter().all(|n| n.id != note));
}

#[test]
fn picker_entries_drag_into_the_playlist() {
    let mut h = playlist_harness();
    let source = ClipSource::Pattern(h.state().current_pattern());
    let from = h
        .state()
        .ui
        .playlist
        .picker_rows
        .iter()
        .find(|r| r.0 == source)
        .unwrap()
        .1
        .center();
    let g = h.state().ui.playlist.geometry.unwrap();
    let track = empty_track(&h, 3);
    let n = h.state().project().playlist.clips.len();
    drag(&mut h, from, g.pos(PPQ * 8, track));
    assert_eq!(h.state().project().playlist.clips.len(), n + 1);
    let clip = last_clip(&h);
    assert_eq!((clip.source, clip.start, clip.track), (source, PPQ * 8, track));
}

#[test]
fn mixer_meter_and_background_select_but_control_drags_do_not() {
    let mut h = mixer_harness(1);
    let pan = h.state().ui.mixer.pan_rects.iter().find(|r| r.0 == 2).unwrap().1;
    drag(&mut h, pan.center(), pan.center() - vec2(0.0, 25.0));
    assert_eq!(h.state().ui.mixer.selected, 1);
    let meter = h.state().ui.mixer.meter_rects.iter().find(|r| r.0 == 2).unwrap().1;
    click(&mut h, meter.center(), PointerButton::Primary);
    assert_eq!(h.state().ui.mixer.selected, 2);
    let strip = h.state().ui.mixer.strip_rects.iter().find(|r| r.0 == 3).unwrap().1;
    click(
        &mut h,
        strip.min + vec2(2.0, strip.height() * 0.5),
        PointerButton::Primary,
    );
    assert_eq!(h.state().ui.mixer.selected, 3);
}

#[test]
fn channel_and_playlist_mute_locks_preserve_state_when_soloing() {
    let mut h = harness(true);
    let channel = h.state().project().channels[0].id;
    let step = h.state().ui.rack.step_pos(channel, 0).unwrap();
    let rack = window_rect(&h, Win::ChannelRack);
    let mute = egui::pos2(rack.left() + 14.0, step.y);
    click_with(&mut h, mute, PointerButton::Primary, Modifiers::SHIFT);
    assert!(h.state().project().channel(channel).unwrap().mute_locked);
    let other = h.state().project().channels[1].id;
    h.state_mut().exec(Command::SetChannelSolo {
        channel: other,
        solo: true,
    });
    h.state_mut().sync_engine();
    assert!(h.state().snapshot().channels[0].audible);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let g = h.state().ui.playlist.geometry.unwrap();
    let mute = egui::pos2(g.grid.left() - 14.0, g.pos(0, 0).y);
    click_with(&mut h, mute, PointerButton::Primary, Modifiers::SHIFT);
    assert!(h.state().project().playlist.tracks[0].mute_locked);
    let other = mute + vec2(0.0, g.track_h);
    click(&mut h, other, PointerButton::Secondary);
    assert!(!h.state().project().playlist.tracks[0].muted);
    assert!(!h.state().project().playlist.tracks[1].muted);
    assert!(h.state().project().playlist.tracks[2].muted);
}

#[test]
fn automation_points_can_be_placed_far_from_the_existing_curve() {
    let mut h = playlist_harness();
    let clip = limeshot_ui::automation_ui::create(h.state_mut(), AutomationTarget::MixerVolume(1)).unwrap();
    h.step();
    h.step();
    let c = h.state().project().clip(clip).unwrap().clone();
    let id = c.automation().unwrap();
    let g = h.state().ui.playlist.geometry.unwrap();
    let top = g.grid.top() + c.track as f32 * g.track_h - g.scroll.y;
    let p = egui::pos2(g.pos(PPQ * 2, c.track).x, top + g.track_h - 7.0);
    click(&mut h, p, PointerButton::Primary);
    let a = h.state().project().automation(id).unwrap();
    assert_eq!(a.points.len(), 2);
    assert_eq!(a.points[1].tick, PPQ * 2);
    assert!(a.points[1].value < 0.2);
}

#[test]
fn short_pattern_lengths_are_displayed_without_padding_the_length_control() {
    let mut h = harness(true);
    let pattern = h.state().current_pattern();
    for steps in [1, 8, 15, 16, 32, 512] {
        h.state_mut().exec(Command::SetPatternLength {
            pattern,
            steps: Some(steps),
        });
        h.step();
        assert!(h.query_by_value(&format!("{steps} steps")).is_some());
        assert!(h.state().ui.rack.step_strips.iter().all(|s| s.2 == steps.max(16)));
        assert_eq!(h.state().project().pattern(pattern).unwrap().length_steps, Some(steps));
    }
}

#[test]
fn long_midi_patterns_do_not_increase_the_rack_minimum_width() {
    let mut h = harness(true);
    let channel = h.state().project().channels[3].id;
    let pattern = h.state().current_pattern();
    h.state_mut().exec(Command::SetPatternLength { pattern, steps: None });
    let before = h.state().ui.workspace.min_size(Win::ChannelRack).x;
    h.state_mut().exec(Command::AddNote {
        pattern,
        note: NewNote {
            channel,
            key: 72,
            start: PPQ * 64,
            length: PPQ,
            velocity: 0.8,
        },
    });
    h.step();
    h.step();
    assert_eq!(h.state().ui.workspace.min_size(Win::ChannelRack).x, before);
    assert!(window_rect(&h, Win::ChannelRack).width() < 1060.0);
    let strip = h.state().ui.rack.step_strips.iter().find(|r| r.0 == channel).unwrap();
    assert!(
        strip.1.width() > window_rect(&h, Win::ChannelRack).width(),
        "long MIDI stays scrollable"
    );
}

#[test]
fn stopping_twice_returns_to_song_or_selection_start() {
    let mut h = playlist_harness();
    h.state_mut().set_mode(limeshot_engine::PlayMode::Song);
    h.state_mut().locate(PPQ * 8);
    wait_for(&mut h, "locate", |a| a.player.status().position() == PPQ * 8);
    h.state_mut().stop();
    wait_for(&mut h, "rewind", |a| a.player.status().position() == 0);
    h.state_mut().ui.playlist.time_sel = Some((PPQ * 4, PPQ * 12));
    h.step();
    h.state_mut().locate(PPQ * 8);
    wait_for(&mut h, "locate", |a| a.player.status().position() == PPQ * 8);
    h.state_mut().stop();
    wait_for(&mut h, "selection rewind", |a| a.player.status().position() == PPQ * 4);
}

/// AUT-013: while the song plays, an automated fader shows the automation's value; stopped, it
/// shows its own setting again.
#[test]
fn automated_controls_follow_the_song() {
    let mut h = harness(true);
    let bar = h.state().project().time_signature.ticks_per_bar();
    let clip = h.state_mut().exec(Command::AddAutomationClip {
        target: AutomationTarget::MixerPan(1),
        track: 10,
        start: 0,
        length: 8 * bar,
        value: None,
        name: None,
    });
    assert!(clip.is_some());
    let id = h
        .state()
        .project()
        .automations
        .last()
        .map(|a| a.id)
        .expect("automation");
    h.state_mut().exec(Command::SetAutomationPoints {
        automation: id,
        points: vec![
            AutomationPoint {
                tick: 0,
                value: 0.0,
                tension: 0.0,
                mode: Default::default(),
            },
            AutomationPoint {
                tick: 8 * bar,
                value: 1.0,
                tension: 0.0,
                mode: Default::default(),
            },
        ],
    });
    h.step();
    let pan = h.state().project().mixer.tracks[1].pan;
    assert_eq!(
        h.state().automated(AutomationTarget::MixerPan(1)),
        None,
        "stopped: no automated value"
    );
    h.state_mut().set_mode(limeshot_engine::PlayMode::Song);
    h.state_mut().locate(4 * bar);
    h.state_mut().toggle_play();
    wait_for(&mut h, "playback", |a| {
        a.is_playing() && a.player.status().position() > 4 * bar
    });
    let v = h
        .state()
        .automated(AutomationTarget::MixerPan(1))
        .expect("automated while playing");
    let pos = h.state().player.status().position() as f32 / (8 * bar) as f32;
    assert!((v - (pos * 2.0 - 1.0)).abs() < 0.1, "pan {v} at {pos}");
    assert_eq!(
        h.state().project().mixer.tracks[1].pan,
        pan,
        "the stored value is untouched"
    );
    h.state_mut().stop();
    wait_for(&mut h, "the stop", |a| !a.is_playing());
    assert_eq!(h.state().automated(AutomationTarget::MixerPan(1)), None);
}

/// WF-003: notes from the MIDI input play the selected channel and stay held until their
/// note-off (not ended by the mouse safety net); a note ends on the channel it started on.
#[test]
fn midi_input_notes_play_the_selected_channel() {
    use limeshot_audio::midi::MidiEvent;
    let mut h = harness(true);
    let lead = h.state().project().channels[3].id;
    let bass = h.state().project().channels[4].id;
    h.state_mut().select_channel(lead);
    h.state_mut().midi_event(MidiEvent::NoteOn { key: 60, velocity: 0.5 });
    for _ in 0..3 {
        h.step();
    }
    assert_eq!(h.state().midi_notes, vec![(60, lead)], "held without a mouse button");
    wait_for(&mut h, "a voice", |a| a.player.status().peak(0).0 > 0.001);
    // The selection changes while the key is down: the note still ends on the lead.
    h.state_mut().select_channel(bass);
    h.state_mut().midi_event(MidiEvent::NoteOn { key: 64, velocity: 1.0 });
    assert_eq!(h.state().midi_notes, vec![(60, lead), (64, bass)]);
    h.state_mut().midi_event(MidiEvent::NoteOff { key: 60 });
    h.state_mut().midi_event(MidiEvent::NoteOff { key: 64 });
    assert!(h.state().midi_notes.is_empty());
    // The settings window's MIDI page lists inputs and can refresh them.
    h.state_mut().open_settings(limeshot_ui::settings_window::Page::Midi);
    h.step();
    h.step();
    h.get_by_label("Refresh device list").click();
    h.step();
    assert!(h.state().settings.midi_input.is_none());
}

/// REC-002 / PR-021: R arms recording and starts playback; notes from the MIDI input become
/// notes of the current pattern (starts on the snap), one undo step per take.
#[test]
fn recording_turns_played_notes_into_pattern_notes() {
    use limeshot_audio::midi::MidiEvent;
    let (mut h, pat, lead) = empty_piano_roll();
    h.state_mut().ui.focus = limeshot_ui::app::Focus::None;
    h.key_press(Key::R);
    h.step();
    assert!(h.state().ui.record.armed);
    wait_for(&mut h, "playback", |a| {
        a.is_playing() && a.player.status().position() > PPQ / 2
    });
    let play = |h: &mut Harness<'static, LimeShotApp>, key: u8| {
        h.state_mut().midi_event(MidiEvent::NoteOn { key, velocity: 0.5 });
        // The empty pattern loops every bar: measure the time played across the loop point.
        let bar = h.state().project().time_signature.ticks_per_bar();
        let from = h.state().player.status().position();
        wait_for(h, "the note to last", |a| {
            (a.player.status().position() + bar - from) % bar >= PPQ / 4
        });
        h.state_mut().midi_event(MidiEvent::NoteOff { key });
    };
    play(&mut h, 60);
    play(&mut h, 64);
    let notes = pattern_notes(&h, pat, lead);
    assert_eq!(notes.iter().map(|n| n.key).collect::<Vec<_>>().len(), 2, "{notes:?}");
    assert!(
        notes
            .iter()
            .all(|n| n.start % STEP_TICKS == 0 && n.length >= PPQ / 4 - 60),
        "{notes:?}"
    );
    assert!((notes[0].velocity - 0.5).abs() < 1e-6);
    assert_eq!(h.state().doc.undo_label(), Some("Record"));
    h.state_mut().doc.undo();
    assert!(pattern_notes(&h, pat, lead).is_empty(), "the take is one undo step");
    // Disarm on stop.
    h.state_mut().ui.record.disarm_on_stop = true;
    h.state_mut().stop();
    assert!(!h.state().ui.record.armed);
    // Not armed: playing records nothing.
    h.state_mut().toggle_play();
    wait_for(&mut h, "playback", |a| a.is_playing());
    play(&mut h, 67);
    assert!(pattern_notes(&h, pat, lead).is_empty());
    h.state_mut().stop();
}

/// PL-023: an audio clip's fade handles (top of its body, FL Studio) drag its fade-in and
/// fade-out, one undo step each.
#[test]
fn audio_clip_fade_handles_set_the_fades() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().ui.playlist.zoom_x = 100.0;
    h.step();
    let snare = limeshot_app::bundled_samples_dir()
        .expect("samples")
        .join("Lime Snare.wav");
    let bar = h.state().project().time_signature.ticks_per_bar();
    h.state_mut().samples.load_blocking(&snare.to_string_lossy()).unwrap();
    let id = limeshot_ui::playlist::add_audio_clip(h.state_mut(), &snare, 4, bar).expect("placed");
    h.state_mut().exec(Command::ResizeClip { clip: id, length: bar });
    h.step();
    h.step();
    let g = h.state().ui.playlist.geometry.expect("playlist drawn");
    let top = g.grid.top() + 4.0 * g.track_h - g.scroll.y;
    let x = |t: Tick| g.pos(t, 4).x;
    let fades = |h: &Harness<'static, LimeShotApp>| {
        let c = h.state().project().clip(id).expect("clip");
        (c.fade_in, c.fade_out)
    };
    assert_eq!(fades(&h), (0, 0));
    // The fade-in handle sits at the clip's left, the fade-out one at its right.
    drag(
        &mut h,
        egui::pos2(x(bar) + 12.0, top + 20.0),
        egui::pos2(x(bar + PPQ), top + 20.0),
    );
    let (fi, _) = fades(&h);
    assert!((fi as i64 - PPQ as i64).abs() <= 10, "fade in {fi}");
    assert_eq!(h.state().doc.undo_label(), Some("Clip fade"));
    let undo = h.state().doc.history().len();
    drag(
        &mut h,
        egui::pos2(x(2 * bar) - 12.0, top + 20.0),
        egui::pos2(x(2 * bar - PPQ / 2), top + 20.0),
    );
    let (_, fo) = fades(&h);
    assert!((fo as i64 - (PPQ / 2) as i64).abs() <= 10, "fade out {fo}");
    assert_eq!(h.state().doc.history().len(), undo + 1, "a drag is one undo step");
    assert_eq!(
        h.state().project().clip(id).map(|c| (c.start, c.length)),
        Some((bar, bar)),
        "the clip did not move"
    );
}

/// INS-008: a FLAC (or MP3, Ogg, AIFF) sample shows in the Browser and loads into a sampler
/// like a WAV.
#[test]
fn compressed_samples_list_load_and_play() {
    let dir = tempfile::tempdir().expect("temp dir");
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../limeshot-dsp/tests/fixtures/sine.flac");
    let flac = dir.path().join("Sine.flac");
    std::fs::copy(&src, &flac).expect("copy");
    std::fs::write(dir.path().join("notes.txt"), "not a sample").expect("write");
    assert!(limeshot_ui::browser::is_sample(&flac));
    assert!(!limeshot_ui::browser::is_sample(&dir.path().join("notes.txt")));
    let mut h = harness(true);
    let kick = h.state().project().channels[0].id;
    limeshot_ui::browser::assign_sample(h.state_mut(), kick, &flac);
    let stored = flac.to_string_lossy().into_owned();
    wait_for(&mut h, "the FLAC to load", |a| {
        matches!(
            a.samples.status(&stored),
            Some(limeshot_audio::assets::SampleStatus::Ready(_))
        )
    });
    if let Some(limeshot_audio::assets::SampleStatus::Ready(s)) = h.state().samples.status(&stored) {
        assert_eq!((s.sample_rate, s.frames()), (22_050, 2205));
    }
    wait_for(&mut h, "the Lime Sampler", |a| {
        a.plugins.is_loaded(PluginTarget::Channel(kick))
    });
    let Instrument::Plugin(plugin) = &h.state().project().channel(kick).unwrap().instrument else {
        panic!("plugin")
    };
    let state: serde_json::Value = serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
    let sample: serde_json::Value = serde_json::from_str(state["fields"]["sample-v1"].as_str().unwrap()).unwrap();
    assert_eq!(sample["sample_rate"], 22_050);
    let pcm = limeshot_core::plugin::base64_decode(sample["pcm"].as_str().unwrap()).unwrap();
    assert_eq!(pcm.len(), 2205 * 2 * 4);
}

/// REC-001: a finished audio take lands on the Playlist as an audio clip at the song position
/// where recording started, on a free track, as long as the recording.
#[test]
fn an_audio_take_becomes_an_audio_clip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("Untitled take 1.wav");
    let data: Vec<f32> = (0..48_000).flat_map(|i| [(i as f32 * 0.05).sin() * 0.3; 2]).collect();
    limeshot_dsp::write_wav(&path, 48_000, &data, limeshot_dsp::BitDepth::Float32).expect("wav");
    let mut h = harness(true);
    let bar = h.state().project().time_signature.ticks_per_bar();
    let clips = h.state().project().playlist.clips.len();
    let take = limeshot_audio::record::Take {
        path: path.clone(),
        frames: 48_000,
        sample_rate: 48_000,
        dropped: 0,
    };
    // The demo's first tracks are busy at bar 2: the take goes below them.
    limeshot_ui::recording::place_take(h.state_mut(), &take, 2 * bar);
    wait_for(&mut h, "recorded audio import", |a| {
        a.project().playlist.clips.len() == clips + 1
    });
    let p = h.state().project();
    assert_eq!(p.playlist.clips.len(), clips + 1);
    let c = p
        .playlist
        .clips
        .iter()
        .find(|c| c.audio_channel().is_some())
        .expect("audio clip");
    assert_eq!(c.start, 2 * bar);
    let want = p.tempo.bpm() / 60.0 * PPQ as f64; // one second
    assert!((c.length as f64 - want).abs() <= 1.0, "{} vs {want}", c.length);
    assert!(
        !p.playlist
            .clips
            .iter()
            .any(|o| o.id != c.id && o.track == c.track && o.start < c.end() && c.start < o.end()),
        "on a free track"
    );
    // Audio recording is off until the Audio filter is chosen.
    assert!(!h.state().ui.record.audio);
}

/// INS-014: sampler envelope controls reset through commands and remain undoable.
#[test]
fn sampler_envelope_controls_reset_and_undo() {
    let mut h = harness(true);
    let channel = h.state().project().channels[0].id;
    let mut params = lime::sampler_params(&h.state().project().channels[0].instrument).unwrap();
    params.attack_ms = 500.0;
    params.decay_ms = 800.0;
    params.sustain = 0.3;
    h.state_mut().exec(Command::SetInstrument {
        channel,
        instrument: Instrument::Sampler(params),
    });
    h.state_mut().select_channel(channel);
    h.state_mut().show_window(Win::ChannelSettings);
    let target = PluginTarget::Channel(channel);
    for (label, index) in [("Attack", 8), ("Decay", 9), ("Sustain", 10)] {
        h.state_mut().plugins.filter_parameters(target, label);
        h.step();
        h.step();
        let before = h.state().project().channel(channel).unwrap().instrument.clone();
        let history = h.state().doc.history().len();
        let id = lime::param_id(index);
        let rect = h.state().plugins.parameter_rect(target, id).unwrap();
        double_click(&mut h, rect.center());
        let inst = h.state().plugins.instance(target).unwrap();
        let default = inst.params().iter().find(|p| p.id == id).unwrap().default;
        assert!(
            (inst.param(id) - default).abs() < 1e-6,
            "actual {}, default {}",
            inst.param(id),
            default
        );
        while h.state().doc.history().len() > history {
            assert!(h.state_mut().doc.undo());
        }
        h.step();
        let Instrument::Plugin(expected) = before else {
            panic!("plugin")
        };
        let expected = lime::plain_values(&expected, &lime::sampler_values(&SamplerParams::default(), 60));
        let actual = lime::sampler_params(&h.state().project().channel(channel).unwrap().instrument).unwrap();
        assert_eq!(lime::sampler_values(&actual, 60)[index], expected[index]);
    }
}

fn mixer_control_menu(h: &mut Harness<'static, LimeShotApp>, label: &str) {
    let rects = if label == "Volume" {
        &h.state().ui.mixer.fader_rects
    } else {
        &h.state().ui.mixer.pan_rects
    };
    let r = rects.iter().find(|(i, _)| *i == h.state().ui.mixer.selected).unwrap().1;
    click(h, r.center(), PointerButton::Secondary);
    h.step();
}

#[test]
fn control_menus_copy_paste_reset_and_undo() {
    let mut h = mixer_harness(1);
    let original_volume = h.state().project().mixer.tracks[1].volume;
    h.state_mut().exec(Command::SetMixerPan { track: 1, pan: -0.5 });
    h.step();
    let history = h.state().doc.history().len();
    mixer_control_menu(&mut h, "Pan");
    h.get_by_label("Copy value").click();
    h.step();
    assert_eq!(h.state().control_clipboard, Some(0.25));
    assert_eq!(h.state().doc.history().len(), history, "copy does not edit the project");
    // Paste transfers the normalised control position onto the fader's cubic scale.
    mixer_control_menu(&mut h, "Volume");
    h.get_by_label("Paste value").click();
    h.step();
    assert!((h.state().project().mixer.tracks[1].volume - 0.03125).abs() < 1e-6);
    assert_eq!(h.state().doc.history().len(), history + 1);
    assert!(h.state_mut().doc.undo());
    h.step();
    assert_eq!(h.state().project().mixer.tracks[1].volume, original_volume);
    mixer_control_menu(&mut h, "Pan");
    {
        use egui_kittest::kittest::NodeT;
        h.get_all_by_label("Reset")
            .find(|n| !n.accesskit_node().is_disabled())
            .expect("control menu Reset")
            .click();
    }
    h.step();
    assert_eq!(h.state().project().mixer.tracks[1].pan, 0.0);
    assert!(h.state_mut().doc.undo());
    assert_eq!(h.state().project().mixer.tracks[1].pan, -0.5);
}

#[test]
fn control_value_dialog_validates_commits_once_and_cancels() {
    let mut h = mixer_harness(1);
    let original_volume = h.state().project().mixer.tracks[1].volume;
    let history = h.state().doc.history().len();
    mixer_control_menu(&mut h, "Volume");
    h.get_by_label("Type in value…").click();
    h.step();
    assert!(matches!(
        h.state().dialog,
        Some(limeshot_ui::dialogs::Dialog::ControlValue { .. })
    ));
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("NaN".into()));
    h.step();
    h.key_press(Key::Enter);
    h.step();
    assert!(h.state().dialog.is_some(), "invalid input keeps the dialog open");
    assert_eq!(h.state().doc.history().len(), history);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("-12".into()));
    h.step();
    h.get_by_label("OK").click();
    h.step();
    assert!(h.state().dialog.is_none());
    assert!((h.state().project().mixer.tracks[1].volume - limeshot_dsp::db_to_gain(-12.0)).abs() < 1e-5);
    assert_eq!(h.state().doc.history().len(), history + 1);
    assert!(h.state_mut().doc.undo());
    h.step();
    assert_eq!(h.state().project().mixer.tracks[1].volume, original_volume);
    mixer_control_menu(&mut h, "Pan");
    h.get_by_label("Type in value…").click();
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("50".into()));
    h.step();
    h.get_by_label("Cancel").click();
    h.step();
    assert!(h.state().dialog.is_none());
    assert_eq!(h.state().project().mixer.tracks[1].pan, 0.0);
    assert_eq!(h.state().doc.history().len(), history);
}

#[test]
fn auto_scrolling_switches_follow_each_other_without_editing_the_project() {
    let mut h = harness(false);
    assert!(!h.state().settings.auto_scroll);
    h.get_by_label("Auto scrolling — follow the playhead").click();
    h.step();
    assert!(h.state().settings.auto_scroll);
    let options = h.get_by_label("Options").rect().center();
    click(&mut h, options, PointerButton::Primary);
    h.step();
    h.get_all_by_label_contains("Auto scrolling")
        .find(|n| n.rect().top() > 90.0)
        .expect("Options menu switch")
        .click();
    h.step();
    assert!(!h.state().settings.auto_scroll);
    assert!(!h.state().doc.is_modified());
    assert!(h.state().doc.history().is_empty());
}

/// UI-016: locate beyond the viewport instead of waiting for wall-clock playback to get there.
#[test]
fn auto_scrolling_follows_playback_and_loop_returns_only_when_enabled() {
    use limeshot_engine::PlayMode;
    for (mode, window) in [(PlayMode::Pattern, Win::PianoRoll), (PlayMode::Song, Win::Playlist)] {
        let mut h = harness(true);
        let channel = h.state().project().channels[0].id;
        let pattern = h.state().current_pattern();
        h.state_mut().select_channel(channel);
        h.state_mut().exec(Command::SetPatternLength {
            pattern,
            steps: Some(128),
        });
        h.state_mut().doc.mark_saved();
        h.state_mut().ui.workspace.close(Win::ChannelRack);
        h.state_mut().show_window(window);
        h.state_mut().ui.workspace.state_mut(window).maximized = true;
        h.state_mut().ui.piano.zoom_x = 160.0;
        h.state_mut().ui.playlist.zoom_x = 160.0;
        h.state_mut().set_auto_scroll(true);
        h.state_mut().set_mode(mode);
        let target = 16 * PPQ;
        h.step(); // send the new pattern length before locating beyond the original loop
        h.state_mut().locate(target);
        wait_for(&mut h, "locate before playback", |a| {
            a.player.status().position() == target
        });
        h.step();
        let scroll = |a: &LimeShotApp| {
            if mode == PlayMode::Pattern {
                a.ui.piano.scroll.x
            } else {
                a.ui.playlist.scroll.x
            }
        };
        assert_eq!(scroll(h.state()), 0.0, "stopped editors do not follow");
        h.state_mut().toggle_play();
        wait_for(&mut h, "playback", |a| a.is_playing());
        h.step();
        assert!(scroll(h.state()) > 0.0, "the active editor follows");
        let position = h.state().player.status().position();
        let (x, grid) = if mode == PlayMode::Pattern {
            let g = h.state().ui.piano.geometry.expect("piano");
            (g.pos(position, 60).x, g.grid)
        } else {
            let g = h.state().ui.playlist.geometry.expect("playlist");
            (g.pos(position, 0).x, g.grid)
        };
        assert!(x >= grid.left() && x <= grid.right(), "the playhead is visible");
        h.state_mut().set_auto_scroll(false);
        h.state_mut().ui.piano.scroll.x = 0.0;
        h.state_mut().ui.playlist.scroll.x = 0.0;
        h.step();
        h.step();
        assert_eq!(scroll(h.state()), 0.0, "switch off leaves the view alone");
        h.state_mut().set_auto_scroll(true);
        h.step();
        assert!(scroll(h.state()) > 0.0);
        h.state_mut().locate(0);
        wait_for(&mut h, "return to the start", |a| a.player.status().position() < PPQ);
        h.step();
        assert_eq!(scroll(h.state()), 0.0, "loop/locate returns bring the view back");
        h.state_mut().stop();
        wait_for(&mut h, "stop", |a| !a.is_playing());
        h.state_mut().ui.piano.scroll.x = 1000.0;
        h.state_mut().ui.playlist.scroll.x = 1000.0;
        h.step();
        assert_eq!(scroll(h.state()), 1000.0, "manual inspection works while stopped");
        assert!(!h.state().doc.is_modified());
    }
}

#[test]
fn playlist_auto_scroll_pauses_during_ruler_gestures() {
    for marker in [false, true] {
        let mut h = playlist_harness();
        let bar = h.state().project().time_signature.ticks_per_bar();
        if marker {
            h.state_mut().exec(Command::AddMarker {
                tick: 0,
                name: "Intro".into(),
            });
        }
        h.state_mut().set_mode(limeshot_engine::PlayMode::Song);
        h.state_mut().set_auto_scroll(true);
        h.state_mut().locate(0);
        wait_for(&mut h, "locate", |a| a.player.status().position() == 0);
        h.state_mut().toggle_play();
        wait_for(&mut h, "playback", |a| a.is_playing());
        h.step();
        let g = h.state().ui.playlist.geometry.unwrap();
        let from = if marker {
            h.state().ui.playlist.marker_rects[0].1.center()
        } else {
            egui::pos2(g.pos(bar, 0).x, g.grid.top() - 24.0)
        };
        let mods = if marker { Modifiers::NONE } else { Modifiers::COMMAND };
        h.event(Event::ModifiersChanged(mods));
        h.event(Event::PointerMoved(from));
        h.event(Event::PointerButton {
            pos: from,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: mods,
        });
        h.step();
        // Move the view away from the playing position while the ruler gesture is held.
        h.state_mut().ui.playlist.scroll.x = 1000.0;
        h.event(Event::PointerMoved(from + vec2(20.0, 0.0)));
        h.step();
        h.step();
        assert_eq!(h.state().ui.playlist.scroll.x, 1000.0, "marker drag: {marker}");
        h.event(Event::PointerButton {
            pos: from + vec2(20.0, 0.0),
            button: PointerButton::Primary,
            pressed: false,
            modifiers: mods,
        });
        h.event(Event::ModifiersChanged(Modifiers::NONE));
        h.step();
        h.state_mut().stop();
    }
}

#[test]
fn save_new_version_from_file_menu_keeps_previous_project_and_undo_history() {
    let dir = tempfile::tempdir().expect("dir");
    let original = dir.path().join("beat.lime");
    let mut h = harness(true);
    assert!(h.state_mut().save_to(&original));
    let previous = std::fs::read(&original).expect("saved file");
    h.state_mut().exec(Command::SetTempo(Tempo::from_bpm(103.0)));
    let revision = h.state().doc.revision();
    let undo = h.state().doc.undo_label();
    h.step();
    let file = h.get_by_label("File").rect().center();
    click(&mut h, file, PointerButton::Primary);
    h.get_by_label_contains("Save new version").click();
    h.step();
    let next = dir.path().join("beat_2.lime");
    assert_eq!(h.state().path.as_ref(), Some(&next));
    assert_eq!(h.state().settings.recent.first(), Some(&next));
    assert!(!h.state().doc.is_modified());
    assert_eq!(h.state().doc.revision(), revision);
    assert_eq!(h.state().doc.undo_label(), undo);
    assert_eq!(std::fs::read(&original).expect("original"), previous);
    assert_eq!(
        limeshot_project::load(&next).expect("new version").tempo,
        Tempo::from_bpm(103.0)
    );
    assert!(h.state_mut().doc.undo(), "undo previous edit");
    assert!(h.state().doc.is_modified());
    assert_ne!(h.state().project().tempo, Tempo::from_bpm(103.0));

    // A failed save retains the active path and unsaved state.
    let unavailable = dir.path().join("missing/beat.lime");
    h.state_mut().path = Some(unavailable.clone());
    assert!(!h.state_mut().save_new_version());
    assert_eq!(h.state().path.as_ref(), Some(&unavailable));
    assert!(h.state().doc.is_modified());
}

/// AUT-008: the tempo field's right-click creates a tempo automation clip, starting at the
/// project tempo.
#[test]
fn the_tempo_field_creates_a_tempo_automation_clip() {
    let mut h = harness(true);
    let tempo = h.state().project().tempo.bpm() as f32;
    let bpm_label = h.get_by_label("BPM").rect();
    click(
        &mut h,
        bpm_label.left_center() - vec2(30.0, 0.0),
        PointerButton::Secondary,
    );
    h.step();
    h.get_by_label("Create automation clip").click();
    h.step();
    let p = h.state().project();
    let a = p
        .automations
        .iter()
        .find(|a| a.target == AutomationTarget::Tempo)
        .expect("a tempo automation");
    assert!(
        p.playlist.clips.iter().any(|c| c.automation() == Some(a.id)),
        "placed on the Playlist"
    );
    let mapping = AutomationTarget::Tempo.mapping(p).expect("mapping");
    let start = mapping.to_value(a.points[0].value);
    assert!(
        (start - tempo).abs() < 0.01,
        "starts at the project tempo: {start} vs {tempo}"
    );
}

#[test]
fn embedded_sampler_previews_without_the_source_file() {
    let mut h = harness(false);
    let audio = limeshot_core::EmbeddedAudio {
        name: "Embedded.wav".into(),
        sample_rate: 48000,
        stereo: vec![0.3; 48000],
    };
    let params = SamplerParams::default();
    let reference = limeshot_core::LimePlugin::SAMPLER.with_values(
        &limeshot_core::lime::sampler_values(&params, 60),
        false,
        Some(&audio),
    );
    let channel = ChannelId(
        h.state_mut()
            .exec(Command::AddChannel {
                name: "Embedded".into(),
                instrument: Instrument::Plugin(reference),
            })
            .unwrap(),
    );
    h.state_mut().select_channel(channel);
    h.state_mut().show_window(Win::ChannelSettings);
    for _ in 0..4 {
        h.step();
    }
    let source = h.state().sampler_source(channel).expect("embedded source cached");
    assert!(!source.exists(), "no temporary sample file required");
    assert!(matches!(
        h.state().samples.status_of_path(&source),
        Some(limeshot_audio::assets::SampleStatus::Ready(_))
    ));
    let load = h.get_by_label("Load…").rect();
    let wave = egui::pos2(load.left(), load.bottom() + 36.0);
    press(&mut h, wave, PointerButton::Primary, true);
    assert_eq!(h.state().sampler_preview.map(|p| p.0), Some(channel));
    press(&mut h, wave, PointerButton::Primary, false);
}

#[test]
fn hosted_parameter_menu_types_native_units_and_undoes() {
    let mut h = harness(true);
    let channel = h.state().project().channels[0].id;
    h.state_mut().select_channel(channel);
    h.state_mut().show_window(Win::ChannelSettings);
    let target = PluginTarget::Channel(channel);
    let id = lime::param_id(8);
    h.state_mut().plugins.filter_parameters(target, "Attack");
    for _ in 0..3 {
        h.step();
    }
    let rect = h.state().plugins.parameter_rect(target, id).unwrap();
    click(&mut h, rect.center(), PointerButton::Secondary);
    h.get_by_label("Type in value…").click();
    h.step();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("500".into()));
    h.step();
    h.get_by_label("OK").click();
    h.step();
    let params = lime::sampler_params(&h.state().project().channel(channel).unwrap().instrument).unwrap();
    assert!((params.attack_ms - 500.0).abs() < 0.01);
    assert!(h.state_mut().doc.undo());
    h.step();
    let params = lime::sampler_params(&h.state().project().channel(channel).unwrap().instrument).unwrap();
    assert_eq!(params.attack_ms, 0.0);
}

#[test]
fn hosted_sampler_slider_drag_commits_state_only_on_release() {
    let mut h = harness(true);
    let channel = h.state().project().channels[0].id;
    h.state_mut().select_channel(channel);
    h.state_mut().show_window(Win::ChannelSettings);
    let target = PluginTarget::Channel(channel);
    let id = lime::param_id(8);
    h.state_mut().plugins.filter_parameters(target, "Attack");
    for _ in 0..3 {
        h.step();
    }
    let rect = h.state().plugins.parameter_rect(target, id).unwrap();
    let before = h.state().project().plugin(target).unwrap().state.clone();
    let revision = h.state().doc.revision();
    let from = egui::pos2(rect.left() + 5.0, rect.center().y);
    let to = egui::pos2(rect.right() - 45.0, rect.center().y);
    press(&mut h, from, PointerButton::Primary, true);
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
        assert_eq!(h.state().project().plugin(target).unwrap().state, before);
        assert_eq!(h.state().doc.revision(), revision);
    }
    press(&mut h, to, PointerButton::Primary, false);
    assert_ne!(h.state().project().plugin(target).unwrap().state, before);
    assert!(h.state_mut().doc.undo());
    assert_eq!(h.state().project().plugin(target).unwrap().state, before);
}

#[test]
fn effect_presets_preserve_live_bypass_and_undo_to_the_live_state() {
    let mut h = harness(true);
    let (track, effect) = h
        .state()
        .project()
        .mixer
        .tracks
        .iter()
        .enumerate()
        .find_map(|(track, t)| {
            t.effects
                .iter()
                .find(|e| e.plugin.as_ref().and_then(LimePlugin::of) == Some(LimePlugin(5)))
                .map(|e| (track as u16, e.id))
        })
        .unwrap();
    let target = PluginTarget::Effect { track, effect };
    let bypass = b"bypass"
        .iter()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)))
        & 0x7fff_ffff;
    assert_eq!(
        lime::parameter_value(h.state().project().plugin(target).unwrap(), bypass),
        Some(0.0)
    );
    h.state_mut().plugins.set_param(target, bypass, 1.0);
    for _ in 0..100 {
        let mut live = h.state().project().plugin(target).unwrap().clone();
        live.state = PluginState::new(h.state().plugins.instance(target).unwrap().state().unwrap());
        if lime::parameter_value(&live, bypass) == Some(1.0) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        h.step();
    }
    limeshot_ui::presets_ui::apply(
        h.state_mut(),
        limeshot_ui::presets_ui::Target::Effect { track, effect },
        limeshot_project::preset::Content::Effect {
            kind: EffectKind::Delay,
            params: vec![90.0, 0.1, 0.0, 6000.0, 0.5, 0.0],
        },
    );
    assert_eq!(
        lime::parameter_value(h.state().project().plugin(target).unwrap(), bypass),
        Some(1.0)
    );
    assert!(h.state_mut().doc.undo());
    assert_eq!(
        lime::parameter_value(h.state().project().plugin(target).unwrap(), bypass),
        Some(1.0)
    );
}

#[test]
fn missing_sample_error_can_reload_after_its_in_project_source_is_restored() {
    let dir = tempfile::tempdir().unwrap();
    let sample = dir.path().join("late.wav");
    let file = dir.path().join("legacy.lime");
    let mut doc = Document::default();
    let channel = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Missing".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: sample.to_string_lossy().into_owned(),
                }),
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    limeshot_project::save(doc.project(), &file).unwrap();
    let mut h = harness(false);
    h.state_mut().open_path(&file);
    h.state_mut().select_channel(channel);
    h.state_mut().show_window(Win::ChannelSettings);
    for _ in 0..4 {
        h.step();
    }
    let target = PluginTarget::Channel(channel);
    assert!(h.state().plugins.error(target).is_some());
    limeshot_dsp::write_wav(&sample, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
    h.get_by_label("Reload").click();
    for _ in 0..4 {
        h.step();
    }
    assert!(
        h.state()
            .project()
            .plugin(target)
            .unwrap()
            .extra
            .get("lime-sample-pending")
            .is_none()
    );
    assert!(h.state().plugins.error(target).is_none());
    assert!(h.state_mut().doc.undo());
    assert_eq!(
        h.state().project().plugin(target).unwrap().extra["lime-sample-pending"],
        true
    );
}

#[derive(Debug)]
struct ExternalAudioFile(std::path::PathBuf);
impl egui::DroppedFile for ExternalAudioFile {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn silent_mp3(path: &std::path::Path) {
    // Synthetic MPEG-1 Layer III silence: 128 kb/s, 44.1 kHz, no bit reservoir or coded spectrum.
    let mut frame = vec![0; 417];
    frame[..4].copy_from_slice(&[0xff, 0xfb, 0x90, 0x00]);
    std::fs::write(path, frame.repeat(8)).unwrap();
}

#[test]
fn mp3_samples_embed_and_join_the_playlist_picker_from_the_rack() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("silence.mp3");
    silent_mp3(&source);
    let decoded = limeshot_dsp::load_audio(&source).unwrap();
    assert_eq!(decoded.sample_rate, 44_100);
    assert!(decoded.frames() > 0);
    let mut h = harness(false);
    h.state_mut().samples.load_blocking(&source.to_string_lossy()).unwrap();
    let id = limeshot_ui::browser::new_sampler_channel(h.state_mut(), &source).unwrap();
    let plugin = h.state().project().plugin(PluginTarget::Channel(id)).unwrap();
    assert!(plugin.extra.get("lime-sample-pending").is_none());
    assert!(limeshot_ui::playlist::audio_channels(h.state().project()).contains(&id));
    assert_ne!(h.state().project().channel(id).unwrap().group.as_deref(), Some("Audio"));
    h.state_mut().sync_engine();
    assert!(h.state().sampler_source(id).is_some());
    std::fs::remove_file(source).unwrap();
    h.state_mut().sync_engine();
    assert!(
        h.state().sampler_source(id).is_some(),
        "embedded playback survives source removal"
    );
}

#[test]
fn external_files_drop_on_playlist_at_the_pointer_and_undo() {
    let mut h = playlist_harness();
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("external.mp3");
    silent_mp3(&source);
    let g = h.state().ui.playlist.geometry.unwrap();
    let position = g.pos(4 * PPQ, 2);
    h.event(Event::PointerMoved(position));
    h.step();
    let before = h.state().project().playlist.clips.len();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(ExternalAudioFile(source)));
    h.step();
    wait_for(&mut h, "external Playlist import", |a| {
        a.project().playlist.clips.len() == before + 1
    });
    let clips = &h.state().project().playlist.clips;
    assert_eq!(clips.len(), before + 1);
    let clip = clips.last().unwrap();
    assert_eq!(clip.track, 2);
    assert_eq!(clip.start, 4 * PPQ);
    assert!(matches!(clip.source, ClipSource::Audio(_)));
    h.state_mut().doc.undo();
    assert_eq!(h.state().project().playlist.clips.len(), before);
}

#[test]
fn external_files_drop_on_rack_without_creating_a_playlist_clip() {
    let mut h = harness(false);
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("external.mp3");
    silent_mp3(&source);
    let rack = window_rect(&h, Win::ChannelRack);
    h.event(Event::PointerMoved(rack.center()));
    h.step();
    let before = h.state().project().channels.len();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(ExternalAudioFile(source)));
    h.step();
    wait_for(&mut h, "sample channel import", |a| {
        a.project().channels.len() == before + 1
    });
    assert_eq!(h.state().project().channels.len(), before + 1);
    assert!(h.state().project().playlist.clips.is_empty());
    let id = h.state().project().channels.last().unwrap().id;
    assert!(limeshot_ui::playlist::audio_channels(h.state().project()).contains(&id));
}

#[test]
fn mixer_on_buttons_keep_locked_inserts_audible_and_allow_direct_toggles() {
    let mut h = harness(true);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.state_mut().show_window(Win::Mixer);
    h.state_mut().ui.workspace.state_mut(Win::Mixer).maximized = true;
    h.step();
    h.step();
    let on = h
        .state()
        .ui
        .mixer
        .on_buttons
        .iter()
        .find(|(i, _)| *i == 1)
        .unwrap()
        .1
        .center();
    click_with(&mut h, on, PointerButton::Primary, Modifiers::SHIFT);
    assert!(h.state().project().mixer.tracks[1].mute_locked);
    h.state_mut().exec(Command::SetMixerSolo { track: 2, solo: true });
    h.state_mut().sync_engine();
    assert!(h.state().snapshot().mixer[1].audible);
    click(&mut h, on, PointerButton::Primary);
    h.state_mut().sync_engine();
    assert!(h.state().project().mixer.tracks[1].muted);
    assert!(!h.state().snapshot().mixer[1].audible);
    click(&mut h, on, PointerButton::Primary);
    h.state_mut().sync_engine();
    assert!(h.state().snapshot().mixer[1].audible);
    let saved: Project = serde_json::from_str(&serde_json::to_string(h.state().project()).unwrap()).unwrap();
    assert!(saved.mixer.tracks[1].mute_locked);
}

#[test]
fn locked_rack_and_playlist_tracks_allow_explicit_on_off_clicks() {
    let mut h = harness(true);
    let channel = h.state().project().channels[0].id;
    let step = h.state().ui.rack.step_pos(channel, 0).unwrap();
    let rack = window_rect(&h, Win::ChannelRack);
    let on = egui::pos2(rack.left() + 14.0, step.y);
    click_with(&mut h, on, PointerButton::Primary, Modifiers::SHIFT);
    click(&mut h, on, PointerButton::Primary);
    assert!(h.state().project().channel(channel).unwrap().muted);
    click(&mut h, on, PointerButton::Primary);
    assert!(!h.state().project().channel(channel).unwrap().muted);
    assert!(h.state().project().channel(channel).unwrap().mute_locked);
    h.state_mut().ui.workspace.close(Win::ChannelRack);
    h.step();
    let g = h.state().ui.playlist.geometry.unwrap();
    let on = egui::pos2(g.grid.left() - 14.0, g.pos(0, 0).y);
    click_with(&mut h, on, PointerButton::Primary, Modifiers::SHIFT);
    click(&mut h, on, PointerButton::Primary);
    assert!(h.state().project().playlist.tracks[0].muted);
    click(&mut h, on, PointerButton::Primary);
    assert!(!h.state().project().playlist.tracks[0].muted);
    assert!(h.state().project().playlist.tracks[0].mute_locked);
}

#[test]
fn timeline_zoom_extends_to_tick_detail_and_whole_arrangements() {
    use limeshot_ui::{piano_menu, playlist_menu};
    let (mut h, _, _) = melody_piano_roll();
    for _ in 0..50 {
        piano_menu::run(h.state_mut(), piano_menu::Act::ZoomIn);
    }
    assert_eq!(h.state().ui.piano.zoom_x, piano_menu::MAX_ZOOM);
    assert!(h.state().ui.piano.zoom_x / PPQ as f32 > 10.0);
    for _ in 0..100 {
        piano_menu::run(h.state_mut(), piano_menu::Act::ZoomOut);
    }
    assert_eq!(h.state().ui.piano.zoom_x, piano_menu::MIN_ZOOM);
    h.step();
    for _ in 0..100 {
        playlist_menu::run(h.state_mut(), playlist_menu::Act::ZoomIn);
    }
    assert_eq!(h.state().ui.playlist.zoom_x, playlist_menu::MAX_ZOOM);
    for _ in 0..100 {
        playlist_menu::run(h.state_mut(), playlist_menu::Act::ZoomOut);
    }
    assert_eq!(h.state().ui.playlist.zoom_x, playlist_menu::MIN_ZOOM);
}

#[test]
fn plugin_wrapper_title_controls_minimize_pin_restore_and_close() {
    use limeshot_ui::plugins::Target;
    let mut h = harness(false);
    h.state_mut().plugins.add_builtin_test_plugins();
    h.step();
    menu_path(&mut h, &["+ Add channel", "Plugins", "LimeShot", "Lime Test Tone"]);
    let channel = h.state().project().channels.last().unwrap().id;
    let target = Target::Channel(channel);
    h.state_mut().plugins.open_window(target, None);
    h.step();
    h.step();
    let rect = h.state().plugins.window(target).unwrap().rect().unwrap();
    // Shared title bar: X, minimize and pin, with no maximize or detach button.
    click(
        &mut h,
        egui::pos2(rect.right() - 59.0, rect.top() + 14.0),
        PointerButton::Primary,
    );
    assert!(h.state().plugins.window(target).unwrap().is_pinned());
    click(
        &mut h,
        egui::pos2(rect.right() - 37.0, rect.top() + 14.0),
        PointerButton::Primary,
    );
    assert!(h.state().plugins.window(target).unwrap().is_minimized());
    h.step();
    let small = h.state().plugins.window(target).unwrap().rect().unwrap();
    assert!(small.height() < 40.0, "{small:?}");
    click(
        &mut h,
        egui::pos2(small.right() - 37.0, small.top() + 14.0),
        PointerButton::Primary,
    );
    h.step();
    h.step();
    let restored = h.state().plugins.window(target).unwrap().rect().unwrap();
    assert!(restored.height() > 150.0, "{restored:?}");
    assert!(
        h.state()
            .plugins
            .window_place(target)
            .unwrap()
            .settings_size
            .unwrap()
            .iter()
            .all(|v| v.is_finite())
    );
    click(
        &mut h,
        egui::pos2(restored.right() - 15.0, restored.top() + 14.0),
        PointerButton::Primary,
    );
    assert!(!h.state().plugins.is_window_open(target));
}

#[test]
fn sampler_wrapper_accepts_browser_payload_and_external_sample_replacement() {
    use limeshot_ui::plugin_window::Tab;
    use limeshot_ui::plugins::Target;
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.wav");
    let second = dir.path().join("second.mp3");
    limeshot_dsp::write_wav(&first, 44_100, &[0.2; 128], limeshot_dsp::BitDepth::Float32).unwrap();
    silent_mp3(&second);
    let mut h = harness(false);
    h.state_mut().samples.load_blocking(&first.to_string_lossy()).unwrap();
    let id = limeshot_ui::browser::new_sampler_channel(h.state_mut(), &first).unwrap();
    h.step();
    let target = Target::Channel(id);
    assert!(h.state().plugins.is_loaded(target));
    h.state_mut().plugins.open_window(target, Some(Tab::Settings));
    h.step();
    h.step();
    let rect = h.state().plugins.window(target).unwrap().rect().unwrap();
    let position = rect.center();
    h.event(Event::PointerMoved(position));
    press(&mut h, position, PointerButton::Primary, true);
    egui::DragAndDrop::set_payload(&h.ctx, limeshot_ui::browser::DraggedSample(second.clone()));
    press(&mut h, position, PointerButton::Primary, false);
    h.step();
    let source = |h: &Harness<'static, LimeShotApp>| {
        lime::sampler_params(&h.state().project().channel(id).unwrap().instrument)
            .unwrap()
            .sample
            .unwrap()
            .path
    };
    wait_for(&mut h, "sample replacement", |a| {
        lime::sampler_params(&a.project().channel(id).unwrap().instrument)
            .unwrap()
            .sample
            .unwrap()
            .path
            == second.to_string_lossy()
    });
    assert_eq!(source(&h), second.to_string_lossy());
    h.event(Event::PointerMoved(position));
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(ExternalAudioFile(first.clone())));
    h.step();
    assert_eq!(source(&h), first.to_string_lossy());
    assert!(limeshot_ui::playlist::audio_channels(h.state().project()).contains(&id));
}

#[test]
fn browser_file_drag_creates_a_rack_sampler_from_real_pointer_events() {
    let mut h = harness(false);
    let path = bundled("Lime Clap.wav");
    let from = h
        .state()
        .ui
        .browser
        .file_rects
        .iter()
        .find(|(p, _)| *p == path)
        .unwrap()
        .1
        .center();
    let rack = window_rect(&h, Win::ChannelRack);
    let before = h.state().project().channels.len();
    drag(&mut h, from, rack.center());
    h.step();
    wait_for(&mut h, "sample channel import", |a| {
        a.project().channels.len() == before + 1
    });
    assert_eq!(h.state().project().channels.len(), before + 1);
    let channel = h.state().project().channels.last().unwrap();
    assert_eq!(
        lime::sampler_params(&channel.instrument).unwrap().sample.unwrap().path,
        path.to_string_lossy()
    );
}

#[test]
fn ruler_ranges_select_contents_and_duplicate_by_the_range_length() {
    let (mut h, pid, cid) = melody_piano_roll();
    let g = h.state().ui.piano.geometry.unwrap();
    let from = egui::pos2(g.pos(0, 60).x, g.ruler.center().y);
    let to = egui::pos2(g.pos(2 * PPQ, 60).x, g.ruler.center().y);
    drag_with(&mut h, from, to, Modifiers::COMMAND);
    assert_eq!(h.state().ui.piano.time_sel, Some((0, 2 * PPQ)));
    let selected = h.state().ui.piano.selection.clone();
    assert!(!selected.is_empty());
    let original: Vec<_> = h
        .state()
        .project()
        .pattern(pid)
        .unwrap()
        .notes
        .iter()
        .filter(|n| selected.contains(&n.id))
        .copied()
        .collect();
    let history = h.state().doc.history().len();
    h.key_press_modifiers(Modifiers::COMMAND, Key::B);
    h.step();
    assert_eq!(h.state().doc.history().len(), history + 1);
    for n in original {
        assert!(
            h.state()
                .project()
                .pattern(pid)
                .unwrap()
                .notes
                .iter()
                .any(|copy| copy.channel == cid
                    && copy.id != n.id
                    && copy.start == n.start + 2 * PPQ
                    && copy.key == n.key)
        );
    }
    assert_eq!(h.state().ui.piano.time_sel, Some((2 * PPQ, 4 * PPQ)));
    let mut h = playlist_harness();
    let g = h.state().ui.playlist.geometry.unwrap();
    let y = g.grid.top() - 6.0;
    drag_with(
        &mut h,
        egui::pos2(g.pos(0, 0).x, y),
        egui::pos2(g.pos(4 * PPQ, 0).x, y),
        Modifiers::COMMAND,
    );
    assert_eq!(h.state().ui.playlist.time_sel, Some((0, 4 * PPQ)));
    let selected = h.state().ui.playlist.selection.clone();
    assert!(!selected.is_empty());
    let original: Vec<_> = h
        .state()
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| selected.contains(&c.id))
        .cloned()
        .collect();
    h.key_press_modifiers(Modifiers::COMMAND, Key::B);
    h.step();
    for c in original {
        assert!(
            h.state()
                .project()
                .playlist
                .clips
                .iter()
                .any(|copy| copy.id != c.id && copy.source == c.source && copy.start == c.start + 4 * PPQ)
        );
    }
    assert_eq!(h.state().ui.playlist.time_sel, Some((4 * PPQ, 8 * PPQ)));
}

#[test]
fn automation_creation_uses_current_playhead_even_with_an_old_range() {
    let mut h = playlist_harness();
    let at = 3 * PPQ + 120;
    h.state_mut().locate(at);
    wait_for(&mut h, "locate", |a| a.player.status().position() == at);
    h.state_mut().ui.playlist.time_sel = Some((0, PPQ));
    let id = limeshot_ui::automation_ui::create(h.state_mut(), AutomationTarget::MixerPan(1)).unwrap();
    assert_eq!(h.state().project().clip(id).unwrap().start, at);
}

#[test]
fn external_multiple_samples_extend_playlist_tracks_and_undo_atomically() {
    let mut h = playlist_harness();
    let dir = tempfile::tempdir().unwrap();
    let paths: Vec<_> = (0..3)
        .map(|i| {
            let path = dir.path().join(format!("drop-{i}.wav"));
            limeshot_dsp::write_wav(&path, 48000, &[0.1; 256], limeshot_dsp::BitDepth::Float32).unwrap();
            path
        })
        .collect();
    let tracks = h.state().project().playlist.tracks.len();
    let before = h.state().project().playlist.clips.len();
    // Bring the last track into view.
    h.state_mut().ui.playlist.scroll.y = tracks as f32 * h.state().ui.playlist.track_h;
    h.step();
    let g = h.state().ui.playlist.geometry.unwrap();
    let pos = g.pos(2 * PPQ, tracks as u32 - 1);
    h.event(Event::PointerMoved(pos));
    h.step();
    for path in paths {
        h.input_mut()
            .dropped_files
            .push(std::sync::Arc::new(ExternalAudioFile(path)));
    }
    h.step();
    wait_for(&mut h, "three drops", |a| {
        a.project().playlist.clips.len() == before + 3
    });
    let added = &h.state().project().playlist.clips[before..];
    assert_eq!(
        added.iter().map(|c| c.track).collect::<Vec<_>>(),
        vec![tracks as u32 - 1, tracks as u32, tracks as u32 + 1]
    );
    assert!(added.iter().all(|c| c.start == 2 * PPQ));
    for _ in 0..3 {
        assert!(h.state_mut().doc.undo());
    }
    assert_eq!(h.state().project().playlist.clips.len(), before);
    assert_eq!(h.state().project().playlist.tracks.len(), tracks);
}

#[test]
fn long_rack_horizontal_scrolling_does_not_grow_the_window() {
    let mut h = harness(true);
    let pid = h.state().current_pattern();
    h.state_mut().exec(Command::SetPatternLength {
        pattern: pid,
        steps: Some(512),
    });
    h.run_steps(4);
    let before = window_rect(&h, Win::ChannelRack);
    h.event(Event::PointerMoved(before.center()));
    h.step();
    for _ in 0..10 {
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: vec2(-220.0, 0.0),
            phase: egui::TouchPhase::Move,
            modifiers: Modifiers::NONE,
        });
        h.step();
    }
    h.run_steps(3);
    assert!((window_rect(&h, Win::ChannelRack).width() - before.width()).abs() < 1.0);
}
