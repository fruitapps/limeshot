//! The Channel Rack (step sequencer), shown in the bottom dock.
//!
//! Row layout follows the FL Studio convention: mute · pan · volume · mixer track · name ·
//! steps. Steps are notes of the current pattern on the 16th-note grid (see `limeshot_core`).
//! A click (either button) toggles a step; a drag paints the state the first step was switched to,
//! as in FL Studio. A channel whose notes are not
//! plain steps shows a preview of its notes instead; clicking it opens the Piano Roll. Clicking a
//! channel name shows its instrument in the Inspector.
//!
//! The selector LED between the name and the steps is FL Studio's channel selection (observed in
//! FL Studio 2026.1.1): a click selects only that channel, Ctrl-, Shift- or right-click toggles it.
//! Alt+Up/Down moves, Alt+Delete deletes (after a confirmation) and Ctrl+L routes the selected
//! channels.
//!
//! The graph editor (FL Studio's, toggled from the header or the channel menu) shows the selected
//! channel's step velocities or pans under the rack, one bar per step; drag across it to draw.

use crate::app::LimeShotApp;
use crate::browser::{self, DraggedSample};
use crate::theme::{Theme, mix, rgb, semibold};
use crate::widgets::{knob, toggle_label};
use crate::workspace::Win;
use egui::{Align2, Context, CornerRadius, DragValue, Key, Modifiers, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use limeshot_core::*;

const ROW_H: f32 = 30.0;
const STEP_W: f32 = 22.0;
const LED_W: f32 = 8.0;

#[derive(Default)]
pub struct State {
    /// (channel, painting on/off) while a step drag is in progress.
    paint: Option<(ChannelId, bool)>,
    /// Steps already painted in the current drag.
    painted: Vec<u32>,
    /// Screen rectangles of each channel's step strip (or note preview) in the last frame (for
    /// tests and tools).
    pub step_strips: Vec<(ChannelId, Rect, u32)>,
    /// Screen rectangles of each channel's selector LED in the last frame.
    pub leds: Vec<(ChannelId, Rect)>,
    /// The graph editor's lane, when it is open.
    pub graph: Option<NoteProp>,
    /// The graph editor's drawing area in the last frame.
    pub graph_rect: Option<Rect>,
    /// While drawing in the graph: the last (step, value) and the values set so far.
    graph_drag: Option<((u32, f32), Vec<(NoteId, f32)>)>,
}

impl State {
    /// Centre of `step` of `channel`'s step strip, as last drawn.
    pub fn step_pos(&self, channel: ChannelId, step: u32) -> Option<egui::Pos2> {
        self.step_strips
            .iter()
            .find(|(c, _, n)| *c == channel && step < *n)
            .map(|(_, r, _)| pos2(r.left() + (step as f32 + 0.5) * STEP_W - 1.0, r.center().y))
    }

    /// Centre of `channel`'s selector LED, as last drawn.
    pub fn led_pos(&self, channel: ChannelId) -> Option<egui::Pos2> {
        self.leds.iter().find(|(c, _)| *c == channel).map(|(_, r)| r.center())
    }
}

pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    app.ui.rack.step_strips.clear();
    app.ui.rack.leds.clear();
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
                if crate::widgets::user_changed(&r) {
                    app.exec(Command::SetPatternLength {
                        pattern: pid,
                        steps: Some(n),
                    });
                }
                ui.add_space(12.0);
                ui.label(RichText::new("Swing").color(t.text_dim));
                let mut swing = app.project().swing;
                let r = knob(ui, &mut swing, 0.0, 1.0, 0.0, 24.0, false, |v| {
                    format!(
                        "Swing {:.0}% — delays every second 16th (about 67% is a triplet feel)",
                        v * 100.0
                    )
                });
                let cmd = r.changed().then_some(Command::SetSwing(swing));
                app.exec_gesture(&r, cmd);
                ui.label(
                    RichText::new(format!("{:.0}%", app.project().swing * 100.0))
                        .font(crate::theme::mono(10.5))
                        .color(t.text_dim),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let open = app.ui.rack.graph.is_some();
                    if crate::widgets::icon_button(
                        ui,
                        crate::icons::Icon::Graph,
                        open,
                        22.0,
                        "Graph editor: step velocity and pan",
                    )
                    .clicked()
                    {
                        app.ui.rack.graph = if open { None } else { Some(NoteProp::Velocity) };
                    }
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
                crate::automation_ui::menu(app, &r, AutomationTarget::ChannelPan(ch.id));
                let mut vol = ch.volume;
                let r = knob(ui, &mut vol, 0.0, 1.25, 0.78, 24.0, false, |v| {
                    format!("Volume {:.0}%", v * 100.0)
                });
                let cmd = r.changed().then_some(Command::SetChannelVolume {
                    channel: ch.id,
                    volume: vol,
                });
                app.exec_gesture(&r, cmd);
                crate::automation_ui::menu(app, &r, AutomationTarget::ChannelVolume(ch.id));
                let mut mt = ch.mixer_track;
                let r = ui
                    .add_sized(
                        vec2(30.0, 20.0),
                        DragValue::new(&mut mt)
                            .range(0..=tracks.saturating_sub(1) as u16)
                            .custom_formatter(|v, _| if v == 0.0 { "M".into() } else { format!("{v:.0}") }),
                    )
                    .on_hover_text("Mixer track (M = Master). Drag or type.");
                let cmd = crate::widgets::user_changed(&r).then_some(Command::SetChannelMixerTrack {
                    channel: ch.id,
                    track: mt,
                });
                app.exec_gesture(&r, cmd);
                channel_button(app, ui, ch, selected);
                selector_led(app, ui, ch);
                ui.add_space(2.0);
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
        app.ui.rack.graph_rect = None;
        if let (Some(prop), Some(ch)) = (app.ui.rack.graph, sel.and_then(|c| channels.iter().find(|x| x.id == c))) {
            graph_editor(app, ui, ch, pid, prop);
        }
    });
    // Dropping a sample anywhere else in the rack makes a new channel.
    let rack_rect = ui.min_rect();
    let drop = ui.interact(rack_rect, ui.id().with("drop"), Sense::hover());
    if let Some(p) = drop.dnd_release_payload::<DraggedSample>() {
        browser::new_sampler_channel(app, &p.0);
    }
}

const GRAPH_H: f32 = 96.0;

/// FL Studio's graph editor for the selected channel: a bar per lit step, under the step strip.
fn graph_editor(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, pid: PatternId, prop: NoteProp) {
    let t = Theme::current();
    let Some((_, strip, steps)) = app.ui.rack.step_strips.iter().find(|s| s.0 == ch.id).copied() else {
        return;
    };
    ui.add_space(4.0);
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width().max(strip.right()), GRAPH_H), Sense::hover());
    let graph = Rect::from_min_max(pos2(strip.left(), row.top()), pos2(strip.right(), row.bottom()));
    app.ui.rack.graph_rect = Some(graph);
    // Lane tabs, right-aligned under the channel button.
    let mut tab_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(graph.left() - 150.0, row.top()),
                pos2(graph.left() - 8.0, row.bottom()),
            ))
            .layout(egui::Layout::top_down(egui::Align::Max)),
    );
    for (p, label) in [(NoteProp::Velocity, "Velocity"), (NoteProp::Pan, "Pan")] {
        if tab_ui.selectable_label(prop == p, label).clicked() {
            app.ui.rack.graph = Some(p);
        }
    }
    let painter = ui.painter_at(graph);
    painter.rect_filled(graph, CornerRadius::same(t.radius_sm), t.sunken);
    if prop == NoteProp::Pan {
        painter.line_segment(
            [
                pos2(graph.left(), graph.center().y),
                pos2(graph.right(), graph.center().y),
            ],
            Stroke::new(1.0, t.grid_beat),
        );
    }
    let notes: Vec<(u32, Note)> = app
        .project()
        .pattern(pid)
        .map(|p| {
            (0..steps)
                .filter_map(|s| p.step_note(ch.id, ch.root_key, s).map(|n| (s, *n)))
                .collect()
        })
        .unwrap_or_default();
    let color = rgb(ch.color);
    let y_of = |v: f32| match prop {
        NoteProp::Velocity => graph.bottom() - v * graph.height(),
        NoteProp::Pan => graph.center().y - v * graph.height() * 0.5,
    };
    for (s, n) in &notes {
        let x0 = graph.left() + *s as f32 * STEP_W;
        let v = match prop {
            NoteProp::Velocity => n.velocity,
            NoteProp::Pan => n.pan,
        };
        let base = y_of(0.0);
        let r = Rect::from_x_y_ranges(
            x0 + 4.0..=x0 + STEP_W - 6.0,
            y_of(v).min(base)..=y_of(v).max(base) + 1.0,
        );
        painter.rect_filled(r, 0.0, color);
    }
    let resp = ui.interact(graph, ui.id().with(("graph", ch.id)), Sense::click_and_drag());
    let value_at = |p: egui::Pos2| match prop {
        NoteProp::Velocity => ((graph.bottom() - p.y) / graph.height()).clamp(0.0, 1.0),
        NoteProp::Pan => ((graph.center().y - p.y) / (graph.height() * 0.5)).clamp(-1.0, 1.0),
    };
    let step_at = |x: f32| (((x - graph.left()) / STEP_W).floor().max(0.0) as u32).min(steps - 1);
    if let Some(p) = resp.hover_pos() {
        let v = value_at(p);
        app.hint(match prop {
            NoteProp::Velocity => format!("Velocity {:.0}% — drag across the steps to draw", v * 100.0),
            NoteProp::Pan => format!("{} — drag across the steps to draw", pan_text(v)),
        });
    }
    let pressed = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.any_pressed());
    if let Some(p) = resp.interact_pointer_pos() {
        let here = (step_at(p.x), value_at(p));
        if pressed || app.ui.rack.graph_drag.is_none() {
            app.doc.begin_gesture();
            app.ui.rack.graph_drag = Some((here, Vec::new()));
        }
        if let Some((last, mut values)) = app.ui.rack.graph_drag.take() {
            // Draw a line from the last point, as FL Studio does for fast drags.
            let (a, b) = if last.0 <= here.0 { (last, here) } else { (here, last) };
            for (s, n) in &notes {
                if *s < a.0 || *s > b.0 {
                    continue;
                }
                let k = if b.0 > a.0 {
                    (*s - a.0) as f32 / (b.0 - a.0) as f32
                } else {
                    0.0
                };
                let v = a.1 + (b.1 - a.1) * k;
                match values.iter_mut().find(|x| x.0 == n.id) {
                    Some(x) => x.1 = v,
                    None => values.push((n.id, v)),
                }
            }
            if !values.is_empty() {
                app.exec(Command::SetNoteValues {
                    pattern: pid,
                    prop,
                    values: values.clone(),
                });
            }
            app.ui.rack.graph_drag = Some((here, values));
        }
    }
    if app.ui.rack.graph_drag.is_some() && !ui.input(|i| i.pointer.any_down()) {
        app.ui.rack.graph_drag = None;
        app.doc.end_gesture();
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
        // FL Studio: clicking a channel button toggles its Channel Settings window.
        let same = app.selected_channel() == Some(ch.id);
        app.select_channel(ch.id);
        if same && app.is_window_open(Win::ChannelSettings) {
            app.ui.workspace.close(Win::ChannelSettings);
        } else {
            app.show_window(Win::ChannelSettings);
        }
    }
    if let Some(p) = resp.dnd_release_payload::<DraggedSample>() {
        browser::assign_sample(app, ch.id, &p.0);
    }
    let hover = format!(
        "{} ({}) — click: Channel Settings · right-click: menu{}",
        ch.name,
        ch.instrument.kind_label(),
        if missing {
            " · sample or instrument missing"
        } else {
            ""
        }
    );
    let resp = resp.on_hover_text(hover);
    // FL Studio: right-clicking a channel button selects that channel only; the menu acts on it.
    if resp.secondary_clicked() {
        app.select_channel(ch.id);
    }
    resp.context_menu(|ui| channel_menu(app, ui, ch));
}

fn selector_led(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(LED_W, 22.0), Sense::click());
    app.ui.rack.leds.push((ch.id, r));
    let on = app.ui.channel_selection.contains(&ch.id);
    ui.painter().rect_filled(r, CornerRadius::same(2), t.sunken);
    if on {
        ui.painter()
            .rect_filled(r.shrink(2.0), CornerRadius::same(1), mix(t.sunken, t.accent, 0.35));
        ui.painter().rect_stroke(
            r,
            CornerRadius::same(2),
            Stroke::new(1.5, t.accent),
            egui::StrokeKind::Inside,
        );
    }
    let mods = ui.input(|i| i.modifiers);
    if resp.secondary_clicked() || (resp.clicked() && (mods.command || mods.shift)) {
        app.toggle_channel_selection(ch.id);
    } else if resp.clicked() {
        app.select_channel(ch.id);
    }
    if resp.hovered() {
        app.hint("Select channel — Ctrl, Shift or right-click: add to / remove from the selection");
    }
}

/// Delete `channels` in one undo step.
pub fn delete_channels(app: &mut LimeShotApp, channels: &[ChannelId]) {
    if channels.is_empty() {
        return;
    }
    app.exec(Command::Batch(
        channels.iter().map(|c| Command::RemoveChannel(*c)).collect(),
    ));
    app.ui.channel_selection.retain(|c| !channels.contains(c));
}

/// Delete `channels`, asking first as FL Studio does unless the user turned the question off.
pub fn request_delete(app: &mut LimeShotApp, channels: Vec<ChannelId>) {
    if channels.is_empty() {
        return;
    }
    if app.settings.confirm_channel_delete {
        app.dialog = Some(crate::dialogs::Dialog::ConfirmDeleteChannels {
            channels,
            remember: false,
        });
    } else {
        delete_channels(app, &channels);
    }
}

/// Move every selected channel one row up (`delta` -1) or down (+1). A selected channel at the
/// edge stays, and so do selected channels stacked against it, keeping the block together.
pub fn move_selected(app: &mut LimeShotApp, delta: i32) {
    let sel = app.selected_channels();
    let mut order: Vec<ChannelId> = app.project().channels.iter().map(|c| c.id).collect();
    let mut cmds = Vec::new();
    let n = order.len();
    let idx: Vec<usize> = if delta < 0 {
        (1..n).collect()
    } else {
        (0..n.saturating_sub(1)).rev().collect()
    };
    for i in idx {
        let j = if delta < 0 { i - 1 } else { i + 1 };
        if sel.contains(&order[i]) && !sel.contains(&order[j]) {
            cmds.push(Command::MoveChannel {
                channel: order[i],
                delta: delta.signum(),
            });
            order.swap(i, j);
        }
    }
    if !cmds.is_empty() {
        app.exec(Command::Batch(cmds));
    }
}

fn channel_menu(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let pid = app.current_pattern();
    if ui.button("Piano Roll").clicked() {
        app.select_channel(ch.id);
        app.show_window(Win::PianoRoll);
        ui.close();
    }
    if ui.button("Graph editor").clicked() {
        app.select_channel(ch.id);
        app.ui.rack.graph = Some(app.ui.rack.graph.unwrap_or(NoteProp::Velocity));
        ui.close();
    }
    if ui.button("Channel Settings").clicked() {
        app.select_channel(ch.id);
        app.show_window(Win::ChannelSettings);
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
        route_to_free(app, &[ch.id]);
        ui.close();
    }
    // FL Studio's step tools (the second column of its channel menu).
    for every in [2, 4, 8] {
        if ui.button(format!("Fill each {every} steps")).clicked() {
            fill_steps(app, &[ch.id], every);
            ui.close();
        }
    }
    if ui
        .add(egui::Button::new("Rotate left").shortcut_text("Shift+Cmd+←"))
        .clicked()
    {
        rotate_steps(app, &[ch.id], -1);
        ui.close();
    }
    if ui
        .add(egui::Button::new("Rotate right").shortcut_text("Shift+Cmd+→"))
        .clicked()
    {
        rotate_steps(app, &[ch.id], 1);
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
        .button(RichText::new("Delete…").color(Theme::current().danger))
        .clicked()
    {
        request_delete(app, vec![ch.id]);
        ui.close();
    }
}

/// Rotate the notes of `channels` in the current pattern by `by` steps, wrapping round the
/// pattern (one undo step).
pub fn rotate_steps(app: &mut LimeShotApp, channels: &[ChannelId], by: i64) {
    let pid = app.current_pattern();
    let notes: Vec<NoteId> = app
        .project()
        .pattern(pid)
        .map(|p| {
            p.notes
                .iter()
                .filter(|n| channels.contains(&n.channel))
                .map(|n| n.id)
                .collect()
        })
        .unwrap_or_default();
    if !notes.is_empty() {
        app.exec(Command::RotateNotes {
            pattern: pid,
            notes,
            by: by * STEP_TICKS as i64,
        });
    }
}

/// Fill each of `channels` with a step every `every` steps (FL Studio's Fill each 2/4/8 steps).
pub fn fill_steps(app: &mut LimeShotApp, channels: &[ChannelId], every: u32) {
    let pattern = app.current_pattern();
    let cmds = channels
        .iter()
        .map(|&channel| Command::FillSteps {
            pattern,
            channel,
            every,
        })
        .collect();
    app.exec(Command::Batch(cmds));
}

/// Route each of `channels` to its own free mixer track (adding tracks when none is free), in one
/// undo step. FL Studio's Ctrl+L.
pub fn route_to_free(app: &mut LimeShotApp, channels: &[ChannelId]) {
    let p = app.project();
    let tracks = p.mixer.tracks.len();
    let mut used: Vec<u16> = p.channels.iter().map(|c| c.mixer_track).collect();
    let mut next_new = tracks;
    let mut cmds = Vec::new();
    for &channel in channels {
        let free = (1..tracks)
            .filter_map(|i| u16::try_from(i).ok())
            .find(|i| !used.contains(i));
        let track = match free {
            Some(t) => t,
            None if next_new < limeshot_core::Mixer::MAX_TRACKS => {
                cmds.push(Command::AddMixerTrack);
                next_new += 1;
                (next_new - 1) as u16
            }
            None => break,
        };
        used.push(track);
        cmds.push(Command::SetChannelMixerTrack { channel, track });
    }
    if !cmds.is_empty() {
        app.exec(Command::Batch(cmds));
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
        let currently_on = pattern.and_then(|p| p.step_note(ch.id, ch.root_key, s)).is_some();
        // Either button toggles the first step and paints that state (observed in FL Studio 2026:
        // a right-click on an unlit step lights it; see fl-studio-analysis.md §2).
        let on = !currently_on;
        app.ui.rack.paint = Some((ch.id, on));
        app.ui.rack.painted.clear();
        // FL Studio: clicking a step selects that channel only.
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
        app.hint("Steps: click to toggle; drag paints the state of the first step");
    }
}

fn note_preview(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, pid: PatternId, steps: u32, len: Tick) {
    let t = Theme::current();
    let width = steps as f32 * STEP_W;
    let (r, resp) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click());
    app.ui.rack.step_strips.push((ch.id, r, steps));
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
        app.show_window(Win::PianoRoll);
    }
    resp.on_hover_text("Piano Roll notes — click to edit");
}

pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key;
    if key(ctx, Modifiers::ALT, Key::ArrowUp) {
        move_selected(app, -1);
    }
    if key(ctx, Modifiers::ALT, Key::ArrowDown) {
        move_selected(app, 1);
    }
    if key(ctx, Modifiers::ALT, Key::Delete) {
        let sel = app.selected_channels();
        request_delete(app, sel);
    }
    // F2 renames the current pattern (observed in FL Studio 2026.1.1 with the Channel Rack
    // focused; channels are renamed from the channel button's menu).
    if key(ctx, Modifiers::NONE, Key::F2) {
        let pid = app.current_pattern();
        app.dialog = Some(crate::dialogs::Dialog::rename_pattern(app, pid));
        return;
    }
    // Up/Down select the channel before the first / after the last selected one, on its own.
    let chans: Vec<ChannelId> = app.project().channels.iter().map(|c| c.id).collect();
    let sel = app.selected_channels();
    let first = sel.first().and_then(|c| chans.iter().position(|x| x == c));
    let last = sel.last().and_then(|c| chans.iter().position(|x| x == c));
    if key(ctx, Modifiers::NONE, Key::ArrowUp)
        && let Some(c) = first.and_then(|i| chans.get(i.saturating_sub(1)))
    {
        app.select_channel(*c);
    }
    if key(ctx, Modifiers::NONE, Key::ArrowDown)
        && let Some(c) = last.and_then(|i| chans.get((i + 1).min(chans.len().saturating_sub(1))))
    {
        app.select_channel(*c);
    }
    if key(ctx, Modifiers::COMMAND, Key::L) {
        let sel = app.selected_channels();
        route_to_free(app, &sel);
    }
    // Rotate the selected channels' steps by one, wrapping (FL Studio: Shift+Cmd+Left/Right).
    for (k, by) in [(Key::ArrowLeft, -1), (Key::ArrowRight, 1)] {
        if key(ctx, Modifiers::COMMAND | Modifiers::SHIFT, k) {
            let sel = app.selected_channels();
            rotate_steps(app, &sel, by);
        }
    }
    // Copy/cut/paste the selected channel's notes in the current pattern (steps included);
    // pasting replaces the target channel's notes, as in FL Studio.
    let pid = app.current_pattern();
    let Some(c) = app.selected_channel() else { return };
    let cut = key(ctx, Modifiers::COMMAND, Key::X);
    if cut || key(ctx, Modifiers::COMMAND, Key::C) {
        let notes: Vec<(u8, Tick, Tick, f32)> = app
            .project()
            .pattern(pid)
            .map(|p| p.notes_for(c).map(|n| (n.key, n.start, n.length, n.velocity)).collect())
            .unwrap_or_default();
        app.clipboard = Some(crate::app::Clipboard::Notes(notes));
        if cut {
            app.exec(Command::ClearChannelNotes {
                pattern: pid,
                channel: c,
            });
        }
    }
    if key(ctx, Modifiers::COMMAND, Key::V)
        && let Some(crate::app::Clipboard::Notes(notes)) = app.clipboard.clone()
    {
        let root = app.project().channel(c).map(|ch| ch.root_key).unwrap_or(60);
        let src_root = notes.iter().map(|n| n.0).min().unwrap_or(root);
        // Steps copied from another channel play that channel's root key; re-pitch them so they
        // remain steps on the target channel.
        let new = notes
            .iter()
            .map(|&(key, start, length, velocity)| NewNote {
                channel: c,
                key: if notes.iter().all(|n| n.0 == src_root) {
                    root
                } else {
                    key
                },
                start,
                length,
                velocity,
            })
            .collect();
        app.exec(Command::Batch(vec![
            Command::ClearChannelNotes {
                pattern: pid,
                channel: c,
            },
            Command::AddNotes {
                pattern: pid,
                notes: new,
            },
        ]));
    }
}
