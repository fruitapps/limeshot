//! The Mixer window: master plus insert tracks. Channels route to inserts (Channel Rack's
//! mixer field); inserts sum into the master. Each strip has a meter, fader, pan, mute and solo.
//! Mute: left-click · Solo: right-click or Cmd+click (FL Studio convention). Clicking a strip's
//! name or FX area selects it; the panel on the right shows its level, routing and FX slots (as FL Studio's
//! Mixer does).

use crate::app::LimeShotApp;
use crate::theme::{Theme, mix, mono, rgb, semibold};
use crate::widgets::{fader, knob, meter, panel_header, section, toggle_label};
use egui::{CornerRadius, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use limeshot_core::Command;

const STRIP_W: f32 = 68.0;

#[derive(Default)]
pub struct State {
    pub selected: u16,
}

pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let n = app.project().mixer.tracks.len();
    if app.meter_hold.len() != n {
        app.meter_hold = vec![(0.0, 0.0); n];
    }
    // Meters fall back at about 20 dB/s.
    let dt = ui.input(|i| i.stable_dt).min(0.1);
    let fall = limeshot_dsp::db_to_gain(-20.0 * dt);
    for i in 0..n {
        let (l, r) = app.player.status().peak(i);
        if let Some(h) = app.meter_hold.get_mut(i) {
            h.0 = l.max(h.0 * fall);
            h.1 = r.max(h.1 * fall);
        }
    }
    // FL Studio layout: strips on the left, the selected track's FX slots on the right.
    let full = ui.available_rect_before_wrap();
    let side_w = 300.0f32.min(full.width() * 0.45);
    let strips_rect = Rect::from_min_max(full.min, pos2(full.right() - side_w, full.bottom()));
    let side_rect = Rect::from_min_max(pos2(full.right() - side_w + 1.0, full.top()), full.max);
    let t = Theme::current();
    ui.painter().line_segment(
        [side_rect.left_top(), side_rect.left_bottom()],
        Stroke::new(1.0, t.border),
    );
    let mut side_ui = ui.new_child(egui::UiBuilder::new().id_salt("mixer-track-panel").max_rect(side_rect));
    track_panel(app, &mut side_ui);
    let mut strips_ui = ui.new_child(egui::UiBuilder::new().id_salt("mixer-strips").max_rect(strips_rect));
    let ui = &mut strips_ui;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let height = ui.available_height() - 12.0;
            ui.add_space(6.0);
            ui.horizontal_top(|ui| {
                ui.add_space(8.0);
                ui.spacing_mut().item_spacing.x = 4.0;
                for i in 0..n {
                    strip(app, ui, i as u16, height);
                    if i == 0 {
                        ui.add_space(8.0);
                    }
                }
                if ui
                    .add_sized(vec2(28.0, 28.0), egui::Button::new("+"))
                    .on_hover_text("Add a mixer track")
                    .clicked()
                {
                    app.exec(Command::AddMixerTrack);
                }
            });
        });
}

/// Channels routed to track `i`, with their colors.
fn routed(app: &LimeShotApp, i: u16) -> Vec<(String, egui::Color32)> {
    app.project()
        .channels
        .iter()
        .filter(|c| c.mixer_track == i)
        .map(|c| (c.name.clone(), rgb(c.color)))
        .collect()
}

fn strip(app: &mut LimeShotApp, ui: &mut Ui, i: u16, height: f32) {
    let t = Theme::current();
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        return;
    };
    let any_solo = app.project().mixer.tracks.iter().skip(1).any(|t| t.solo);
    let audible = !track.muted && (i == 0 || !any_solo || track.solo);
    let selected = app.ui.mixer.selected == i;
    let routed = routed(app, i);
    let color = routed
        .first()
        .map(|r| r.1)
        .unwrap_or(if i == 0 { t.accent } else { t.text_faint });
    let (rect, _) = ui.allocate_exact_size(vec2(STRIP_W, height.max(220.0)), Sense::hover());
    let p = ui.painter();
    p.rect_filled(
        rect,
        CornerRadius::same(t.radius),
        if selected {
            mix(t.raised, t.accent, 0.07)
        } else {
            mix(t.surface, t.raised, 0.55)
        },
    );
    if selected {
        p.rect_stroke(
            rect,
            CornerRadius::same(t.radius),
            Stroke::new(1.0, t.accent_dim),
            egui::StrokeKind::Inside,
        );
    }
    // Colour bar at the bottom identifies the routed channel(s).
    p.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + 6.0, rect.bottom() - 4.0),
            pos2(rect.right() - 6.0, rect.bottom() - 2.0),
        ),
        1.0,
        color,
    );
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(5.0, 6.0)))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    let ui = &mut ui;
    ui.spacing_mut().item_spacing.y = 5.0;
    ui.label(
        RichText::new(if i == 0 { "M".to_string() } else { i.to_string() })
            .font(mono(10.0))
            .color(t.text_faint),
    );
    let name = ui
        .add(
            egui::Label::new(
                RichText::new(elide(&track.name, 8))
                    .font(semibold(11.0))
                    .color(if audible { t.text } else { t.text_dim }),
            )
            .sense(Sense::click()),
        )
        .on_hover_text(if routed.is_empty() {
            format!("{} — click: select · double-click: rename", track.name)
        } else {
            format!(
                "{} — channels: {} — click: select · double-click: rename",
                track.name,
                routed.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
            )
        });
    if name.clicked() {
        select(app, i);
    }
    if name.double_clicked() {
        app.dialog = Some(crate::dialogs::Dialog::rename_mixer(app, i));
    }
    // Insert slots (names only; edit them in the track panel on the right).
    let labels = crate::effects_ui::strip_labels(&track.effects);
    let (fx_rect, fx_resp) = ui.allocate_exact_size(vec2(STRIP_W - 12.0, 44.0), Sense::click());
    ui.painter()
        .rect_filled(fx_rect, CornerRadius::same(t.radius_sm), t.sunken);
    for (k, (name, on)) in labels.iter().take(3).enumerate() {
        let y = fx_rect.top() + 7.0 + k as f32 * 13.0;
        ui.painter().circle_filled(
            pos2(fx_rect.left() + 6.0, y),
            2.5,
            if *on { t.accent } else { t.text_faint },
        );
        ui.painter().text(
            pos2(fx_rect.left() + 12.0, y),
            egui::Align2::LEFT_CENTER,
            elide(name, 7),
            mono(9.5),
            if *on { t.text } else { t.text_dim },
        );
    }
    if labels.len() > 3 {
        ui.painter().text(
            fx_rect.right_bottom() - vec2(4.0, 2.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("+{}", labels.len() - 3),
            mono(9.0),
            t.text_dim,
        );
    }
    if labels.is_empty() {
        ui.painter().text(
            fx_rect.center(),
            egui::Align2::CENTER_CENTER,
            "FX",
            semibold(9.5),
            t.text_faint,
        );
    }
    if fx_resp
        .on_hover_text("Insert effects — click to edit them in the panel on the right")
        .clicked()
    {
        select(app, i);
    }
    let mut pan = track.pan;
    let r = knob(ui, &mut pan, -1.0, 1.0, 0.0, 30.0, true, |v| {
        if v.abs() < 0.005 {
            "Pan: centre".into()
        } else {
            format!("Pan {:+.0}%", v * 100.0)
        }
    });
    let cmd = r.changed().then_some(Command::SetMixerPan { track: i, pan });
    app.exec_gesture(&r, cmd);
    let fader_h = (ui.available_height() - 58.0).max(80.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(4.0);
        let (l, r) = app.meter_hold.get(usize::from(i)).copied().unwrap_or_default();
        let clipped = i == 0 && app.clip_until.is_some_and(|u| u > std::time::Instant::now());
        meter(ui, vec2(12.0, fader_h), l, r, clipped);
        let mut v = track.volume;
        let r = fader(
            ui,
            &mut v,
            2.0,
            if i == 0 { 1.0 } else { 0.8 },
            vec2(24.0, fader_h),
            "Volume",
        );
        let cmd = r.changed().then_some(Command::SetMixerVolume { track: i, volume: v });
        app.exec_gesture(&r, cmd);
    });
    let db = limeshot_dsp::gain_to_db(track.volume);
    ui.label(
        RichText::new(if db.is_finite() {
            format!("{db:+.1}")
        } else {
            "−∞".into()
        })
        .font(mono(10.0))
        .color(t.text_dim),
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.add_space(if i == 0 { 17.0 } else { 4.0 });
        let m = toggle_label(
            ui,
            track.muted,
            "M",
            t.danger,
            vec2(24.0, 18.0),
            "Mute (left-click) · Solo (right-click / Cmd+click)",
        );
        let solo_click = m.secondary_clicked() || (m.clicked() && ui.input(|x| x.modifiers.command));
        if solo_click && i != 0 {
            app.exec(Command::SetMixerSolo {
                track: i,
                solo: !track.solo,
            });
        } else if m.clicked() {
            app.exec(Command::SetMixerMute {
                track: i,
                muted: !track.muted,
            });
        }
        if i != 0 {
            let s = toggle_label(ui, track.solo, "S", t.warning, vec2(24.0, 18.0), "Solo");
            if s.clicked() {
                app.exec(Command::SetMixerSolo {
                    track: i,
                    solo: !track.solo,
                });
            }
        }
    });
}

fn select(app: &mut LimeShotApp, i: u16) {
    app.ui.mixer.selected = i;
}

/// The selected track's panel on the right of the Mixer: level, routing and insert slots.
fn track_panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let i = app.ui.mixer.selected;
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        app.ui.mixer.selected = 0;
        return;
    };
    panel_header(ui, "Track", |ui| {
        ui.label(
            RichText::new(if i == 0 { "Master".into() } else { format!("Insert {i}") })
                .font(mono(10.5))
                .color(t.text_dim),
        );
    });
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            let name = ui
                .add(egui::Label::new(RichText::new(&track.name).font(semibold(15.0))).sense(Sense::click()))
                .on_hover_text("Click to rename");
            if name.clicked() {
                app.dialog = Some(crate::dialogs::Dialog::rename_mixer(app, i));
            }
            ui.add_space(8.0);
            section(ui, "Level");
            let mut db = limeshot_dsp::gain_to_db(track.volume).max(-60.0);
            let r = ui.add(
                egui::Slider::new(&mut db, -60.0..=6.0)
                    .suffix(" dB")
                    .fixed_decimals(1)
                    .text("Volume"),
            );
            let cmd = crate::widgets::user_changed(&r).then(|| Command::SetMixerVolume {
                track: i,
                volume: if db <= -60.0 { 0.0 } else { limeshot_dsp::db_to_gain(db) },
            });
            app.exec_gesture(&r, cmd);
            let mut pan = track.pan;
            let r = ui.add(egui::Slider::new(&mut pan, -1.0..=1.0).fixed_decimals(2).text("Pan"));
            let cmd = crate::widgets::user_changed(&r).then_some(Command::SetMixerPan { track: i, pan });
            app.exec_gesture(&r, cmd);
            ui.add_space(8.0);
            section(ui, "Input");
            let routed = routed(app, i);
            if routed.is_empty() {
                ui.label(RichText::new("No channels route here.").color(t.text_dim));
            }
            for (n, c) in routed {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect_filled(r, 2.0, c);
                    ui.label(n);
                });
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(if i == 0 {
                    "Output: audio device"
                } else {
                    "Output: Master"
                })
                .color(t.text_dim),
            );
            crate::effects_ui::chain(app, ui, i);
        });
    });
}

fn elide(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    }
}
