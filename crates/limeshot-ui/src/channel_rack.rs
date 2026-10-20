//! The Channel Rack (step sequencer), shown in the bottom dock.
//!
//! Row layout follows the FL Studio convention: mute · pan · volume · mixer track · name ·
//! steps. Steps are notes of the current pattern on the 16th-note grid (see `limeshot_core`).
//! Left-click/drag paints steps on, right-click/drag paints them off. A channel whose notes are not
//! plain steps shows a preview of its notes instead; clicking it opens the Piano Roll. Clicking a
//! channel name shows its instrument in the Inspector.

use crate::app::{DockTab, LimeShotApp};
use crate::browser::{self, DraggedSample};
use crate::theme::{Theme, mix, rgb, semibold};
use crate::widgets::{knob, toggle_label};
use egui::{
    Align2, Context, CornerRadius, DragValue, Key, Modifiers, PointerButton, Rect, RichText, Sense, Stroke, Ui, pos2,
    vec2,
};
use limeshot_core::*;

const ROW_H: f32 = 30.0;
const STEP_W: f32 = 22.0;

#[derive(Default)]
pub struct State {
    /// (channel, painting on/off) while a step drag is in progress.
    paint: Option<(ChannelId, bool)>,
    /// Steps already painted in the current drag.
    painted: Vec<u32>,
    /// Screen rectangles of each channel's step strip in the last frame (for tests and tools).
    pub step_strips: Vec<(ChannelId, Rect, u32)>,
}

impl State {
    /// Centre of `step` of `channel`'s step strip, as last drawn.
    pub fn step_pos(&self, channel: ChannelId, step: u32) -> Option<egui::Pos2> {
        self.step_strips
            .iter()
            .find(|(c, _, n)| *c == channel && step < *n)
            .map(|(_, r, _)| pos2(r.left() + (step as f32 + 0.5) * STEP_W - 1.0, r.center().y))
    }
}

pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    app.ui.rack.step_strips.clear();
    let t = Theme::current();
    let pid = app.current_pattern();
    let sig = app.project().time_signature;
    let Some(pattern) = app.project().pattern(pid) else {
        return;
    };
    let pat_len = pattern.length(sig);
    let steps = (pat_len / STEP_TICKS).clamp(1, 512) as u32;
    let length_steps = pattern.length_steps;
    let pat_name = pattern.name.clone();
    let pat_color = pattern.color;

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(2), rgb(pat_color));
                ui.label(RichText::new(pat_name).font(semibold(12.5)));
                ui.add_space(10.0);
                ui.label(RichText::new("Length").color(t.text_dim));
                let mut auto = length_steps.is_none();
                if ui
                    .checkbox(&mut auto, "Auto")
                    .on_hover_text("Length follows the notes, in whole bars")
                    .changed()
                {
                    app.exec(Command::SetPatternLength {
                        pattern: pid,
                        steps: if auto { None } else { Some(steps) },
                    });
                }
                let mut n = steps;
                let r = ui.add_enabled(!auto, DragValue::new(&mut n).range(1..=512).suffix(" steps"));
                if r.changed() {
                    app.exec(Command::SetPatternLength {
                        pattern: pid,
                        steps: Some(n),
                    });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("+ Add channel", |ui| {
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
        });

    let channels: Vec<Channel> = app.project().channels.clone();
    if channels.is_empty() {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No channels yet.").color(t.text_dim));
            ui.label(RichText::new("Use + Add channel, or drag a sample here from the Browser.").color(t.text_dim));
        });
    }
    let any_solo = channels.iter().any(|c| c.solo);
    let tracks = app.project().mixer.tracks.len();
    let sel = app.selected_channel();
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(2.0);
        for ch in &channels {
            let row_top = ui.cursor().top();
            let selected = sel == Some(ch.id);
            if selected {
                let r = Rect::from_min_size(pos2(ui.max_rect().left(), row_top), vec2(ui.max_rect().width(), ROW_H));
                ui.painter().rect_filled(r, 0.0, mix(t.surface, t.accent, 0.05));
            }
            ui.horizontal(|ui| {
                ui.set_height(ROW_H);
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.add_space(8.0);
                // Mute / solo LED.
                let active = !ch.muted && (!any_solo || ch.solo);
                let led_color = if ch.solo { t.warning } else { t.accent };
                let r = toggle_label(
                    ui,
                    active,
                    if ch.solo { "S" } else { "" },
                    led_color,
                    vec2(10.0, 18.0),
                    "Left-click: mute · Ctrl+click or right-click: solo",
                );
                let solo_click = r.secondary_clicked() || (r.clicked() && ui.input(|i| i.modifiers.command));
                if solo_click {
                    app.exec(Command::SetChannelSolo {
                        channel: ch.id,
                        solo: !ch.solo,
                    });
                } else if r.clicked() {
                    app.exec(Command::SetChannelMute {
                        channel: ch.id,
                        muted: !ch.muted,
                    });
                }
                let mut pan = ch.pan;
                let r = knob(ui, &mut pan, -1.0, 1.0, 0.0, 24.0, true, pan_text);
                let cmd = r.changed().then_some(Command::SetChannelPan { channel: ch.id, pan });
                app.exec_gesture(&r, cmd);
                let mut vol = ch.volume;
                let r = knob(ui, &mut vol, 0.0, 1.25, 0.78, 24.0, false, |v| {
                    format!("Volume {:.0}%", v * 100.0)
                });
                let cmd = r.changed().then_some(Command::SetChannelVolume {
                    channel: ch.id,
                    volume: vol,
                });
                app.exec_gesture(&r, cmd);
                let mut mt = ch.mixer_track;
                let r = ui
                    .add_sized(
                        vec2(30.0, 20.0),
                        DragValue::new(&mut mt)
                            .range(0..=tracks.saturating_sub(1) as u16)
                            .custom_formatter(|v, _| if v == 0.0 { "M".into() } else { format!("{v:.0}") }),
                    )
                    .on_hover_text("Mixer track (M = Master). Drag or type.");
                let cmd = r.changed().then_some(Command::SetChannelMixerTrack {
                    channel: ch.id,
                    track: mt,
                });
                app.exec_gesture(&r, cmd);
                channel_button(app, ui, ch, selected);
                ui.add_space(6.0);
                let is_steps = app
                    .project()
                    .pattern(pid)
                    .is_some_and(|p| p.is_step_sequence(ch.id, ch.root_key));
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
    let (r, resp) = ui.allocate_exact_size(vec2(132.0, 22.0), Sense::click());
    let base = rgb(ch.color);
    let fill = if resp.hovered() {
        mix(t.raised_hover, base, 0.22)
    } else {
        mix(t.raised, base, 0.14)
    };
    ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), fill);
    ui.painter().rect_filled(
        Rect::from_min_size(r.min, vec2(3.0, r.height())),
        CornerRadius::same(1),
        base,
    );
    if selected {
        ui.painter().rect_stroke(
            r,
            CornerRadius::same(t.radius_sm),
            Stroke::new(1.0, t.accent),
            egui::StrokeKind::Inside,
        );
    }
    let missing = match &ch.instrument {
        Instrument::Sampler(p) => match &p.sample {
            Some(s) => matches!(
                app.samples.status(&s.path),
                Some(limeshot_audio::assets::SampleStatus::Failed(_))
            ),
            None => false,
        },
        Instrument::Unsupported(_) => true,
        Instrument::Synth(_) => false,
    };
    let name = elide(&ch.name, 16);
    ui.painter().text(
        r.left_center() + vec2(10.0, 0.0),
        Align2::LEFT_CENTER,
        name,
        crate::theme::prop(12.0),
        if missing { t.danger } else { t.text },
    );
    if resp.clicked() {
        app.select_channel(ch.id);
        app.ui.show_inspector = true;
    }
    if let Some(p) = resp.dnd_release_payload::<DraggedSample>() {
        browser::assign_sample(app, ch.id, &p.0);
    }
    let hover = format!(
        "{} ({}) — click: show in Inspector · right-click: menu{}",
        ch.name,
        ch.instrument.kind_label(),
        if missing {
            " · sample or instrument missing"
        } else {
            ""
        }
    );
    let resp = resp.on_hover_text(hover);
    resp.context_menu(|ui| channel_menu(app, ui, ch));
}

fn channel_menu(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let pid = app.current_pattern();
    if ui.button("Piano Roll").clicked() {
        app.select_channel(ch.id);
        app.show_tab(DockTab::PianoRoll);
        ui.close();
    }
    if ui.button("Show in Inspector").clicked() {
        app.select_channel(ch.id);
        app.ui.show_inspector = true;
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
                    app.exec(Command::SetChannelColor {
                        channel: ch.id,
                        color: c,
                    });
                    ui.close();
                }
            }
        });
    });
    if matches!(ch.instrument, Instrument::Sampler(_)) && ui.button("Load sample…").clicked() {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("WAV audio", &["wav", "WAV"])
            .pick_file()
        {
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
        app.exec(Command::ClearChannelNotes {
            pattern: pid,
            channel: ch.id,
        });
        ui.close();
    }
    ui.separator();
    if ui.button("Move up").clicked() {
        app.exec(Command::MoveChannel {
            channel: ch.id,
            delta: -1,
        });
        ui.close();
    }
    if ui.button("Move down").clicked() {
        app.exec(Command::MoveChannel {
            channel: ch.id,
            delta: 1,
        });
        ui.close();
    }
    if ui.button("Clone").clicked() {
        if let Some(id) = app.exec(Command::CloneChannel(ch.id)) {
            app.select_channel(ChannelId(id));
        }
        ui.close();
    }
    if ui
        .button(RichText::new("Delete").color(Theme::current().danger))
        .clicked()
    {
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
                app.exec(Command::SetChannelMixerTrack {
                    channel,
                    track: track as u16,
                });
            }
        }
    }
}

fn elide(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    }
}

fn step_strip(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, pid: PatternId, steps: u32) {
    let t = Theme::current();
    let width = steps as f32 * STEP_W;
    let (strip, resp) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click_and_drag());
    app.ui.rack.step_strips.push((ch.id, strip, steps));
    let pattern = app.project().pattern(pid);
    let playing_step = (app.is_playing() && app.ui.mode == limeshot_engine::PlayMode::Pattern)
        .then(|| (app.player.status().position() / STEP_TICKS) as u32);
    for s in 0..steps {
        let r = Rect::from_min_size(
            pos2(strip.left() + s as f32 * STEP_W, strip.top()),
            vec2(STEP_W - 2.0, strip.height()),
        );
        let on = pattern.and_then(|p| p.step_note(ch.id, ch.root_key, s)).is_some();
        let off_color = if (s / 4) % 2 == 0 { t.step_off_a } else { t.step_off_b };
        let fill = if on {
            rgb(ch.color).gamma_multiply(1.0).lerp_to_gamma(t.step_on, 0.35)
        } else {
            off_color
        };
        ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), fill);
        if on {
            ui.painter().rect_filled(
                Rect::from_min_size(r.min + vec2(3.0, 3.0), vec2(r.width() - 6.0, 2.0)),
                1.0,
                mix(fill, egui::Color32::WHITE, 0.45),
            );
        }
        if playing_step == Some(s) {
            ui.painter()
                .rect_stroke(r, 2.0, Stroke::new(1.5, t.playhead), egui::StrokeKind::Outside);
        }
    }
    let step_at = |x: f32| (((x - strip.left()) / STEP_W).floor().max(0.0) as u32).min(steps - 1);
    let pointer = resp.interact_pointer_pos();
    // A stroke starts once, on press; `drag_started` arrives a frame later and must not restart it
    // (it would see the step just painted on and switch to erasing).
    let started = app.ui.rack.paint.is_none() && (resp.drag_started() || resp.is_pointer_button_down_on());
    if started && let Some(pos) = pointer {
        let s = step_at(pos.x);
        let right = ui.input(|i| i.pointer.button_down(PointerButton::Secondary));
        let currently_on = pattern.and_then(|p| p.step_note(ch.id, ch.root_key, s)).is_some();
        // Left toggles (and paints that state); right always erases.
        let on = !right && !currently_on;
        app.ui.rack.paint = Some((ch.id, on));
        app.ui.rack.painted.clear();
        app.select_channel(ch.id);
        app.doc.begin_gesture();
    }
    if let (Some((cid, on)), Some(pos)) = (app.ui.rack.paint, pointer)
        && cid == ch.id
    {
        let s = step_at(pos.x);
        if !app.ui.rack.painted.contains(&s) {
            app.ui.rack.painted.push(s);
            app.exec(Command::SetStep {
                pattern: pid,
                channel: ch.id,
                step: s,
                on,
            });
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
    let (r, resp) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
    ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), t.sunken);
    if let Some(p) = app.project().pattern(pid) {
        let notes: Vec<&Note> = p.notes_for(ch.id).collect();
        let (lo, hi) = notes
            .iter()
            .fold((127u8, 0u8), |(a, b), n| (a.min(n.key), b.max(n.key)));
        let span = f32::from(hi.saturating_sub(lo).max(1));
        for n in notes {
            let x0 = r.left() + n.start as f32 / len.max(1) as f32 * r.width();
            let x1 = r.left() + n.end().min(len) as f32 / len.max(1) as f32 * r.width();
            let y = r.bottom() - 3.0 - f32::from(n.key - lo) / span * (r.height() - 6.0);
            ui.painter().line_segment(
                [pos2(x0, y), pos2(x1.max(x0 + 2.0), y)],
                Stroke::new(2.0, rgb(ch.color)),
            );
        }
    }
    if resp.clicked() {
        app.select_channel(ch.id);
        app.show_tab(DockTab::PianoRoll);
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
