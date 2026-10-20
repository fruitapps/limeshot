//! The Channel Rack (step sequencer) and the Channel Settings window.
//!
//! Row layout follows the FL Studio convention: mute · pan · volume · mixer track · name ·
//! steps. Steps are notes of the current pattern on the 16th-note grid (see `limeshot_core`).
//! Left-click/drag paints steps on, right-click/drag paints them off. A channel whose notes are not
//! plain steps shows a preview of its notes instead; clicking it opens the Piano Roll.

use crate::app::{Focus, LimeShotApp};
use crate::browser::{self, DraggedSample};
use crate::theme::{Theme, mix, mono, rgb};
use crate::widgets::{knob, toggle_label};
use egui::{Align2, Context, CornerRadius, DragValue, Key, Modifiers, PointerButton, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use limeshot_core::*;

const ROW_H: f32 = 26.0;
const STEP_W: f32 = 18.0;

#[derive(Default)]
pub struct State {
    /// (channel, painting on/off) while a step drag is in progress.
    paint: Option<(ChannelId, bool)>,
    /// Steps already painted in the current drag.
    painted: Vec<u32>,
    /// Channel whose settings window is open.
    pub settings: Option<ChannelId>,
}

pub fn window(app: &mut LimeShotApp, ctx: &Context, arrange: bool) {
    let mut open = app.ui.show_rack;
    let ws = app.ui.workspace;
    let rows = app.project().channels.len().max(1) as f32;
    let mut w = egui::Window::new(RichText::new("Channel Rack").strong())
        .id(egui::Id::new("channel_rack"))
        .open(&mut open)
        .constrain_to(ws)
        .default_pos(ws.min + vec2(30.0, 40.0))
        .default_size(vec2(640.0, 90.0 + rows * ROW_H))
        .resizable(true);
    if arrange {
        w = w.current_pos(ws.min + vec2(30.0, 40.0));
    }
    let resp = w.show(ctx, |ui| body(app, ui));
    if let Some(r) = resp
        && r.response.contains_pointer()
        && ctx.input(|i| i.pointer.any_pressed())
    {
        app.ui.focus = Focus::ChannelRack;
    }
    app.ui.show_rack = open;
    settings_window(app, ctx);
}

fn body(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let pid = app.current_pattern();
    let sig = app.project().time_signature;
    let Some(pattern) = app.project().pattern(pid) else { return };
    let pat_len = pattern.length(sig);
    let steps = (pat_len / STEP_TICKS).clamp(1, 512) as u32;
    let length_steps = pattern.length_steps;
    let pat_name = pattern.name.clone();
    let pat_color = pattern.color;

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("■ {pat_name}")).color(rgb(pat_color)).strong());
        ui.separator();
        ui.label("Length");
        let mut auto = length_steps.is_none();
        if ui.checkbox(&mut auto, "Auto").on_hover_text("Length follows the notes, in whole bars").changed() {
            app.exec(Command::SetPatternLength { pattern: pid, steps: if auto { None } else { Some(steps) } });
        }
        let mut n = steps;
        let r = ui.add_enabled(!auto, DragValue::new(&mut n).range(1..=512).suffix(" steps"));
        if r.changed() {
            app.exec(Command::SetPatternLength { pattern: pid, steps: Some(n) });
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("+ Add", |ui| {
                if ui.button("Sampler").clicked() {
                    crate::menus::add_sampler(app);
                    ui.close();
                }
                if ui.button("LimeSynth").clicked() {
                    crate::menus::add_synth(app);
                    ui.close();
                }
            });
        });
    });
    ui.add_space(2.0);

    let channels: Vec<Channel> = app.project().channels.clone();
    if channels.is_empty() {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No channels yet.").color(t.text_dim));
            ui.label(RichText::new("Use + Add, or drag a sample here from the Browser.").color(t.text_dim));
        });
    }
    let any_solo = channels.iter().any(|c| c.solo);
    let tracks = app.project().mixer.tracks.len();
    let sel = app.selected_channel();
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        for ch in &channels {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                // Mute / solo LED.
                let active = !ch.muted && (!any_solo || ch.solo);
                let led_color = if ch.solo { t.warning } else { t.accent };
                let r = toggle_label(ui, active, if ch.solo { "S" } else { "" }, led_color, vec2(12.0, 18.0), "Left-click: mute · Ctrl+click or right-click: solo");
                let solo_click = r.secondary_clicked() || (r.clicked() && ui.input(|i| i.modifiers.command));
                if solo_click {
                    app.exec(Command::SetChannelSolo { channel: ch.id, solo: !ch.solo });
                } else if r.clicked() {
                    app.exec(Command::SetChannelMute { channel: ch.id, muted: !ch.muted });
                }
                let mut pan = ch.pan;
                let r = knob(ui, &mut pan, -1.0, 1.0, 0.0, 20.0, true, pan_text);
                let cmd = r.changed().then(|| Command::SetChannelPan { channel: ch.id, pan });
                app.exec_gesture(&r, cmd);
                let mut vol = ch.volume;
                let r = knob(ui, &mut vol, 0.0, 1.25, 0.78, 20.0, false, |v| format!("Volume {:.0}%", v * 100.0));
                let cmd = r.changed().then(|| Command::SetChannelVolume { channel: ch.id, volume: vol });
                app.exec_gesture(&r, cmd);
                let mut mt = ch.mixer_track;
                let r = ui
                    .add_sized(
                        vec2(28.0, 18.0),
                        DragValue::new(&mut mt)
                            .range(0..=tracks.saturating_sub(1) as u16)
                            .custom_formatter(|v, _| if v == 0.0 { "M".into() } else { format!("{v:.0}") }),
                    )
                    .on_hover_text("Mixer track (M = Master). Drag or type.");
                let cmd = r.changed().then(|| Command::SetChannelMixerTrack { channel: ch.id, track: mt });
                app.exec_gesture(&r, cmd);
                channel_button(app, ui, ch, sel == Some(ch.id));
                // Selection LED.
                let (lr, lresp) = ui.allocate_exact_size(vec2(8.0, 18.0), Sense::click());
                ui.painter().rect_filled(lr, 2.0, if sel == Some(ch.id) { t.accent } else { t.raised });
                if lresp.clicked() {
                    app.ui.selected_channel = Some(ch.id);
                }
                ui.add_space(4.0);
                let is_steps = app.project().pattern(pid).is_some_and(|p| p.is_step_sequence(ch.id, ch.root_key));
                if is_steps {
                    step_strip(app, ui, ch, pid, steps);
                } else {
                    note_preview(app, ui, ch, pid, steps, pat_len);
                }
            });
        }
    });
    // Dropping a sample anywhere else in the rack makes a new channel.
    let rack_rect = ui.min_rect();
    let drop = ui.interact(rack_rect, ui.id().with("drop"), Sense::hover());
    if let Some(p) = drop.dnd_release_payload::<DraggedSample>() {
        browser::new_sampler_channel(app, &p.0);
    }
}

fn pan_text(v: f32) -> String {
    if v.abs() < 0.005 {
        "Pan: centre".into()
    } else if v < 0.0 {
        format!("Pan: {:.0}% left", -v * 100.0)
    } else {
        format!("Pan: {:.0}% right", v * 100.0)
    }
}

fn channel_button(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, selected: bool) {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(118.0, 20.0), Sense::click());
    let base = rgb(ch.color);
    let fill = if resp.hovered() { mix(t.raised_hover, base, 0.25) } else { mix(t.raised, base, 0.15) };
    ui.painter().rect_filled(r, CornerRadius::same(3), fill);
    ui.painter().rect_filled(Rect::from_min_size(r.min, vec2(4.0, r.height())), CornerRadius::same(2), base);
    if selected {
        ui.painter().rect_stroke(r, 3.0, Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
    }
    let missing = match &ch.instrument {
        Instrument::Sampler(p) => match &p.sample {
            Some(s) => matches!(app.samples.status(&s.path), Some(limeshot_audio::assets::SampleStatus::Failed(_))),
            None => false,
        },
        Instrument::Unsupported(_) => true,
        Instrument::Synth(_) => false,
    };
    let name = elide(&ch.name, 15);
    ui.painter().text(r.left_center() + vec2(9.0, 0.0), Align2::LEFT_CENTER, name, crate::theme::prop(12.0), if missing { t.danger } else { t.text });
    if resp.clicked() {
        app.ui.selected_channel = Some(ch.id);
        app.ui.rack.settings = if app.ui.rack.settings == Some(ch.id) { None } else { Some(ch.id) };
    }
    if let Some(p) = resp.dnd_release_payload::<DraggedSample>() {
        browser::assign_sample(app, ch.id, &p.0);
    }
    let hover = format!(
        "{} ({}) — click: settings · right-click: menu{}",
        ch.name,
        ch.instrument.kind_label(),
        if missing { " · sample or instrument missing" } else { "" }
    );
    let resp = resp.on_hover_text(hover);
    resp.context_menu(|ui| channel_menu(app, ui, ch));
}

fn channel_menu(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let pid = app.current_pattern();
    if ui.button("Piano Roll").clicked() {
        app.ui.selected_channel = Some(ch.id);
        app.ui.show_piano = true;
        ui.close();
    }
    if ui.button("Settings…").clicked() {
        app.ui.rack.settings = Some(ch.id);
        ui.close();
    }
    if ui.button("Rename…").clicked() {
        app.dialog = Some(crate::dialogs::Dialog::rename_channel(app, ch.id));
        ui.close();
    }
    ui.menu_button("Color", |ui| {
        ui.horizontal_wrapped(|ui| {
            for c in PALETTE {
                let (r, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
                ui.painter().rect_filled(r, 3.0, rgb(c));
                if resp.clicked() {
                    app.exec(Command::SetChannelColor { channel: ch.id, color: c });
                    ui.close();
                }
            }
        });
    });
    if matches!(ch.instrument, Instrument::Sampler(_)) && ui.button("Load sample…").clicked() {
        if let Some(p) = rfd::FileDialog::new().add_filter("WAV audio", &["wav", "WAV"]).pick_file() {
            browser::assign_sample(app, ch.id, &p);
        }
        ui.close();
    }
    ui.separator();
    if ui.button("Route to free mixer track").clicked() {
        route_to_free(app, ch.id);
        ui.close();
    }
    if ui.button("Clear notes in this pattern").clicked() {
        app.exec(Command::ClearChannelNotes { pattern: pid, channel: ch.id });
        ui.close();
    }
    ui.separator();
    if ui.button("Move up").clicked() {
        app.exec(Command::MoveChannel { channel: ch.id, delta: -1 });
        ui.close();
    }
    if ui.button("Move down").clicked() {
        app.exec(Command::MoveChannel { channel: ch.id, delta: 1 });
        ui.close();
    }
    if ui.button("Clone").clicked() {
        if let Some(id) = app.exec(Command::CloneChannel(ch.id)) {
            app.ui.selected_channel = Some(ChannelId(id));
        }
        ui.close();
    }
    if ui.button(RichText::new("Delete").color(Theme::current().danger)).clicked() {
        app.exec(Command::RemoveChannel(ch.id));
        ui.close();
    }
}

pub fn route_to_free(app: &mut LimeShotApp, channel: ChannelId) {
    match app.project().free_mixer_track() {
        Some(track) => {
            app.exec(Command::SetChannelMixerTrack { channel, track });
        }
        None => {
            if let Some(track) = app.exec(Command::AddMixerTrack) {
                app.exec(Command::SetChannelMixerTrack { channel, track: track as u16 });
            }
        }
    }
}

fn elide(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() } else { format!("{}…", s.chars().take(n - 1).collect::<String>()) }
}

fn step_strip(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, pid: PatternId, steps: u32) {
    let t = Theme::current();
    let width = steps as f32 * STEP_W;
    let (strip, resp) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click_and_drag());
    let pattern = app.project().pattern(pid);
    let playing_step = (app.is_playing() && app.ui.mode == limeshot_engine::PlayMode::Pattern)
        .then(|| (app.player.status().position() / STEP_TICKS) as u32);
    for s in 0..steps {
        let r = Rect::from_min_size(pos2(strip.left() + s as f32 * STEP_W, strip.top()), vec2(STEP_W - 2.0, strip.height()));
        let on = pattern.and_then(|p| p.step_note(ch.id, ch.root_key, s)).is_some();
        let off_color = if (s / 4) % 2 == 0 { t.step_off_a } else { t.step_off_b };
        let fill = if on { rgb(ch.color).gamma_multiply(1.0).lerp_to_gamma(t.step_on, 0.35) } else { off_color };
        ui.painter().rect_filled(r, CornerRadius::same(2), fill);
        if on {
            ui.painter().rect_filled(Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2(r.width() - 4.0, 3.0)), 1.0, mix(fill, egui::Color32::WHITE, 0.4));
        }
        if playing_step == Some(s) {
            ui.painter().rect_stroke(r, 2.0, Stroke::new(1.5, t.playhead), egui::StrokeKind::Outside);
        }
    }
    let step_at = |x: f32| (((x - strip.left()) / STEP_W).floor().max(0.0) as u32).min(steps - 1);
    let pointer = resp.interact_pointer_pos();
    let started = resp.drag_started() || resp.is_pointer_button_down_on() && app.ui.rack.paint.is_none();
    if started && let Some(pos) = pointer {
        let s = step_at(pos.x);
        let right = ui.input(|i| i.pointer.button_down(PointerButton::Secondary));
        let currently_on = pattern.and_then(|p| p.step_note(ch.id, ch.root_key, s)).is_some();
        // Left toggles (and paints that state); right always erases.
        let on = !right && !currently_on;
        app.ui.rack.paint = Some((ch.id, on));
        app.ui.rack.painted.clear();
        app.ui.selected_channel = Some(ch.id);
        app.doc.begin_gesture();
    }
    if let (Some((cid, on)), Some(pos)) = (app.ui.rack.paint, pointer)
        && cid == ch.id
    {
        let s = step_at(pos.x);
        if !app.ui.rack.painted.contains(&s) {
            app.ui.rack.painted.push(s);
            app.exec(Command::SetStep { pattern: pid, channel: ch.id, step: s, on });
            if on {
                app.preview_on(ch.id, ch.root_key, 0.78);
            }
        }
    }
    if app.ui.rack.paint.is_some_and(|(c, _)| c == ch.id) && !ui.input(|i| i.pointer.any_down()) {
        app.ui.rack.paint = None;
        app.doc.end_gesture();
        app.preview_off(ch.id, ch.root_key);
    }
    if resp.hovered() {
        app.hint("Steps: left-click/drag to paint on, right-click/drag to erase");
    }
}

fn note_preview(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, pid: PatternId, steps: u32, len: Tick) {
    let t = Theme::current();
    let width = steps as f32 * STEP_W;
    let (r, resp) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click());
    ui.painter().rect_filled(r, CornerRadius::same(2), t.sunken);
    if let Some(p) = app.project().pattern(pid) {
        let notes: Vec<&Note> = p.notes_for(ch.id).collect();
        let (lo, hi) = notes.iter().fold((127u8, 0u8), |(a, b), n| (a.min(n.key), b.max(n.key)));
        let span = f32::from(hi.saturating_sub(lo).max(1));
        for n in notes {
            let x0 = r.left() + n.start as f32 / len.max(1) as f32 * r.width();
            let x1 = r.left() + n.end().min(len) as f32 / len.max(1) as f32 * r.width();
            let y = r.bottom() - 3.0 - f32::from(n.key - lo) / span * (r.height() - 6.0);
            ui.painter().line_segment([pos2(x0, y), pos2(x1.max(x0 + 2.0), y)], Stroke::new(2.0, rgb(ch.color)));
        }
    }
    if resp.clicked() {
        app.ui.selected_channel = Some(ch.id);
        app.ui.show_piano = true;
    }
    resp.on_hover_text("Piano Roll notes — click to edit");
}

pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key;
    let chans: Vec<ChannelId> = app.project().channels.iter().map(|c| c.id).collect();
    let cur = app.selected_channel().and_then(|c| chans.iter().position(|x| *x == c));
    if key(ctx, Modifiers::NONE, Key::ArrowUp)
        && let Some(i) = cur
    {
        app.ui.selected_channel = chans.get(i.saturating_sub(1)).copied();
    }
    if key(ctx, Modifiers::NONE, Key::ArrowDown)
        && let Some(i) = cur
    {
        app.ui.selected_channel = chans.get((i + 1).min(chans.len().saturating_sub(1))).copied();
    }
    if key(ctx, Modifiers::COMMAND, Key::L)
        && let Some(c) = app.selected_channel()
    {
        route_to_free(app, c);
    }
}

fn settings_window(app: &mut LimeShotApp, ctx: &Context) {
    let Some(cid) = app.ui.rack.settings else { return };
    let Some(ch) = app.project().channel(cid).cloned() else {
        app.ui.rack.settings = None;
        return;
    };
    let mut open = true;
    egui::Window::new(RichText::new(format!("{} — {}", ch.name, ch.instrument.kind_label())).strong())
        .id(egui::Id::new("channel_settings"))
        .open(&mut open)
        .constrain_to(app.ui.workspace)
        .default_pos(app.ui.workspace.center() - vec2(180.0, 120.0))
        .default_width(380.0)
        .resizable(false)
        .show(ctx, |ui| settings_body(app, ui, &ch));
    if !open {
        app.ui.rack.settings = None;
    }
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(vec2(80.0, 18.0), egui::Label::new(RichText::new(label).color(Theme::current().text_dim)));
        add(ui);
    });
}

fn settings_body(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        let mut kind = match ch.instrument {
            Instrument::Sampler(_) => 0,
            Instrument::Synth(_) => 1,
            Instrument::Unsupported(_) => 2,
        };
        let before = kind;
        ui.selectable_value(&mut kind, 0, "Sampler");
        ui.selectable_value(&mut kind, 1, "LimeSynth");
        if kind != before {
            let inst = if kind == 0 { Instrument::Sampler(SamplerParams::default()) } else { Instrument::Synth(SynthParams::default()) };
            app.exec(Command::SetInstrument { channel: ch.id, instrument: inst });
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut key = ch.root_key;
            let r = ui.add(DragValue::new(&mut key).range(0..=127).custom_formatter(|v, _| crate::piano_roll::key_name(v as u8)));
            if r.changed() {
                app.exec(Command::SetChannelRootKey { channel: ch.id, key });
            }
            ui.label(RichText::new("Root").color(t.text_dim));
        });
    });
    ui.separator();
    match &ch.instrument {
        Instrument::Sampler(p) => sampler_settings(app, ui, ch.id, p),
        Instrument::Synth(p) => synth_settings(app, ui, ch.id, p),
        Instrument::Unsupported(_) => {
            ui.label(RichText::new("This channel uses an instrument this version of LimeShot does not support. It is kept in the project but stays silent.").color(t.warning));
        }
    }
}

fn sampler_settings(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId, p: &SamplerParams) {
    let t = Theme::current();
    let set = |p: SamplerParams| Command::SetInstrument { channel: cid, instrument: Instrument::Sampler(p) };
    let status = p.sample.as_ref().and_then(|s| app.samples.status(&s.path).cloned());
    ui.horizontal(|ui| {
        let label = match &p.sample {
            Some(s) => browser::file_label(std::path::Path::new(&s.path)),
            None => "No sample".into(),
        };
        ui.label(RichText::new(label).strong());
        if ui.button("Load…").clicked()
            && let Some(path) = rfd::FileDialog::new().add_filter("WAV audio", &["wav", "WAV"]).pick_file()
        {
            browser::assign_sample(app, cid, &path);
        }
    });
    // Waveform with start/end markers.
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
    ui.painter().rect_filled(r, 3.0, t.sunken);
    match &status {
        Some(limeshot_audio::assets::SampleStatus::Ready(s)) => {
            let frames = s.frames().max(1);
            let cols = r.width() as usize;
            for x in 0..cols {
                let a = x * frames / cols.max(1);
                let b = ((x + 1) * frames / cols.max(1)).max(a + 1);
                let peak = (a..b).step_by(((b - a) / 32).max(1)).map(|i| s.frame(i).0.abs()).fold(0.0f32, f32::max);
                let h = peak.min(1.0) * r.height() * 0.48;
                let xx = r.left() + x as f32;
                ui.painter().line_segment([pos2(xx, r.center().y - h), pos2(xx, r.center().y + h)], Stroke::new(1.0, t.accent_dim));
            }
            for (f, c) in [(p.start, t.accent), (p.end, t.danger)] {
                let x = r.left() + f * r.width();
                ui.painter().line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.5, c));
            }
            ui.label(RichText::new(format!("{:.2} s · {} Hz", s.duration_secs(), s.sample_rate)).small().color(t.text_dim));
        }
        Some(limeshot_audio::assets::SampleStatus::Failed(e)) => {
            ui.painter().text(r.center(), Align2::CENTER_CENTER, "Could not load sample", mono(11.0), t.danger);
            ui.label(RichText::new(e).small().color(t.danger));
        }
        Some(limeshot_audio::assets::SampleStatus::Loading) => {
            ui.painter().text(r.center(), Align2::CENTER_CENTER, "Loading…", mono(11.0), t.text_dim);
        }
        None => {
            ui.painter().text(r.center(), Align2::CENTER_CENTER, "Drop or load a WAV file", mono(11.0), t.text_dim);
        }
    }
    let mut q = p.clone();
    let mut changed: Option<egui::Response> = None;
    let mut track = |r: egui::Response, c: bool| {
        if c || r.drag_started() || r.drag_stopped() {
            changed = Some(r);
        }
    };
    row(ui, "Gain", |ui| {
        let r = ui.add(egui::Slider::new(&mut q.gain_db, -24.0..=12.0).suffix(" dB"));
        let c = r.changed();
        track(r, c);
    });
    row(ui, "Pitch", |ui| {
        let r = ui.add(egui::Slider::new(&mut q.pitch, -24.0..=24.0).suffix(" st").step_by(0.01));
        let c = r.changed();
        track(r, c);
    });
    row(ui, "Start", |ui| {
        let r = ui.add(egui::Slider::new(&mut q.start, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
        let c = r.changed();
        track(r, c);
    });
    row(ui, "End", |ui| {
        let r = ui.add(egui::Slider::new(&mut q.end, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
        let c = r.changed();
        track(r, c);
    });
    row(ui, "Release", |ui| {
        let r = ui.add(egui::Slider::new(&mut q.release_ms, 0.0..=2000.0).logarithmic(true).suffix(" ms"));
        let c = r.changed();
        track(r, c);
    });
    row(ui, "Playback", |ui| {
        let r = ui.checkbox(&mut q.reverse, "Reverse");
        let c = r.changed();
        track(r, c);
        let r = ui.checkbox(&mut q.one_shot, "One-shot").on_hover_text("Play the whole sample, ignoring note length");
        let c = r.changed();
        track(r, c);
    });
    if q.end < q.start {
        q.end = q.start;
    }
    if let Some(r) = changed {
        let cmd = (q != *p).then(|| set(q));
        app.exec_gesture(&r, cmd);
    }
}

fn synth_settings(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId, p: &SynthParams) {
    let mut q = p.clone();
    let mut changed: Option<egui::Response> = None;
    let mut track = |r: egui::Response| {
        if r.changed() || r.drag_started() || r.drag_stopped() {
            changed = Some(r);
        }
    };
    row(ui, "Waveform", |ui| {
        for w in Waveform::ALL {
            track(ui.selectable_value(&mut q.waveform, w, w.label()));
        }
    });
    row(ui, "Attack", |ui| track(ui.add(egui::Slider::new(&mut q.attack_ms, 0.0..=2000.0).logarithmic(true).suffix(" ms"))));
    row(ui, "Decay", |ui| track(ui.add(egui::Slider::new(&mut q.decay_ms, 0.0..=4000.0).logarithmic(true).suffix(" ms"))));
    row(ui, "Sustain", |ui| track(ui.add(egui::Slider::new(&mut q.sustain, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0)))));
    row(ui, "Release", |ui| track(ui.add(egui::Slider::new(&mut q.release_ms, 0.0..=4000.0).logarithmic(true).suffix(" ms"))));
    row(ui, "Gain", |ui| track(ui.add(egui::Slider::new(&mut q.gain_db, -36.0..=12.0).suffix(" dB"))));
    row(ui, "Octave", |ui| track(ui.add(egui::Slider::new(&mut q.octave, -3..=3))));
    row(ui, "Detune", |ui| track(ui.add(egui::Slider::new(&mut q.detune_cents, -50.0..=50.0).suffix(" ct")).on_hover_text("A second, detuned oscillator; 0 turns it off")));
    // ADSR picture.
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::hover());
    ui.painter().rect_filled(r, 3.0, t.sunken);
    let total = (q.attack_ms + q.decay_ms + q.release_ms).max(1.0) * 1.25;
    let x = |ms: f32| r.left() + 4.0 + ms / total * (r.width() - 8.0);
    let y = |lvl: f32| r.bottom() - 4.0 - lvl * (r.height() - 8.0);
    let hold = total * 0.2;
    let pts = vec![
        pos2(x(0.0), y(0.0)),
        pos2(x(q.attack_ms), y(1.0)),
        pos2(x(q.attack_ms + q.decay_ms), y(q.sustain)),
        pos2(x(q.attack_ms + q.decay_ms + hold), y(q.sustain)),
        pos2(x(q.attack_ms + q.decay_ms + hold + q.release_ms), y(0.0)),
    ];
    ui.painter().add(egui::Shape::line(pts, Stroke::new(1.5, t.accent)));
    if let Some(r) = changed {
        let cmd = (q != *p).then(|| Command::SetInstrument { channel: cid, instrument: Instrument::Synth(q) });
        app.exec_gesture(&r, cmd);
    }
}
