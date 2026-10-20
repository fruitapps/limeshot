//! The Mixer: master plus insert tracks. Channels route to inserts (Channel Rack's mixer field);
//! inserts sum into the master. Each strip has a meter, fader, pan, mute and solo.
//! Mute: left-click · Solo: right-click or Cmd+click (FL Studio convention).

use crate::app::{Focus, LimeShotApp};
use crate::theme::{Theme, mix, mono};
use crate::widgets::{fader, knob, meter, toggle_label};
use egui::{Align2, Context, CornerRadius, RichText, Sense, Stroke, Ui, vec2};
use limeshot_core::Command;

const STRIP_W: f32 = 62.0;

#[derive(Default)]
pub struct State {
    pub selected: u16,
}

pub fn window(app: &mut LimeShotApp, ctx: &Context, arrange: bool) {
    let mut open = app.ui.show_mixer;
    let ws = app.ui.workspace;
    let pos = ws.right_top() + vec2(-760.0, 40.0);
    let mut w = egui::Window::new(RichText::new("Mixer").strong())
        .id(egui::Id::new("mixer"))
        .open(&mut open)
        .constrain_to(ws)
        .default_pos(pos)
        .default_size(vec2(740.0, 330.0))
        .min_size(vec2(260.0, 300.0))
        .resizable(true);
    if arrange {
        w = w.current_pos(pos);
    }
    let resp = w.show(ctx, |ui| body(app, ui));
    if let Some(r) = resp
        && r.response.contains_pointer()
        && ctx.input(|i| i.pointer.any_pressed())
    {
        app.ui.focus = Focus::Mixer;
    }
    app.ui.show_mixer = open;
}

fn body(app: &mut LimeShotApp, ui: &mut Ui) {
    let n = app.project().mixer.tracks.len();
    if app.meter_hold.len() != n {
        app.meter_hold = vec![(0.0, 0.0); n];
    }
    // Meters fall back at about 20 dB/s.
    let dt = ui.input(|i| i.stable_dt).min(0.1);
    let fall = limeshot_dsp::db_to_gain(-20.0 * dt);
    for i in 0..n {
        let (l, r) = app.player.status().peak(i);
        let h = &mut app.meter_hold[i];
        h.0 = l.max(h.0 * fall);
        h.1 = r.max(h.1 * fall);
    }
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                for i in 0..n {
                    strip(app, ui, i as u16);
                    if i == 0 {
                        ui.add_space(6.0);
                        ui.separator();
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

fn strip(app: &mut LimeShotApp, ui: &mut Ui, i: u16) {
    let t = Theme::current();
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        return;
    };
    let any_solo = app.project().mixer.tracks.iter().skip(1).any(|t| t.solo);
    let audible = !track.muted && (i == 0 || !any_solo || track.solo);
    let selected = app.ui.mixer.selected == i;
    let routed: Vec<String> = app
        .project()
        .channels
        .iter()
        .filter(|c| c.mixer_track == i)
        .map(|c| c.name.clone())
        .collect();
    let frame = egui::Frame::new()
        .fill(if selected {
            mix(t.raised, t.accent, 0.08)
        } else {
            t.raised
        })
        .stroke(Stroke::new(1.0, if selected { t.accent_dim } else { t.border }))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::same(4));
    frame.show(ui, |ui| {
        ui.set_width(STRIP_W - 8.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let label = if i == 0 { "M".to_string() } else { i.to_string() };
            ui.label(RichText::new(label).font(mono(10.0)).color(t.text_dim));
            let name_resp = ui
                .add(
                    egui::Label::new(RichText::new(elide(&track.name, 8)).small().color(if audible {
                        t.text
                    } else {
                        t.text_dim
                    }))
                    .sense(Sense::click()),
                )
                .on_hover_text(if routed.is_empty() {
                    format!("{} — double-click to rename", track.name)
                } else {
                    format!(
                        "{} — channels: {} — double-click to rename",
                        track.name,
                        routed.join(", ")
                    )
                });
            if name_resp.clicked() {
                app.ui.mixer.selected = i;
            }
            if name_resp.double_clicked() {
                app.dialog = Some(crate::dialogs::Dialog::rename_mixer(app, i));
            }
            let mut pan = track.pan;
            let r = knob(ui, &mut pan, -1.0, 1.0, 0.0, 26.0, true, |v| {
                if v.abs() < 0.005 {
                    "Pan: centre".into()
                } else {
                    format!("Pan {:+.0}%", v * 100.0)
                }
            });
            let cmd = r.changed().then_some(Command::SetMixerPan { track: i, pan });
            app.exec_gesture(&r, cmd);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                let (l, r) = app.meter_hold.get(usize::from(i)).copied().unwrap_or_default();
                let clipped = i == 0 && app.clip_until.is_some_and(|u| u > std::time::Instant::now());
                meter(ui, vec2(14.0, 150.0), l, r, clipped);
                let mut v = track.volume;
                let r = fader(
                    ui,
                    &mut v,
                    2.0,
                    if i == 0 { 1.0 } else { 0.8 },
                    vec2(22.0, 150.0),
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
                let m = toggle_label(
                    ui,
                    track.muted,
                    "M",
                    t.danger,
                    vec2(22.0, 16.0),
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
                    let s = toggle_label(ui, track.solo, "S", t.warning, vec2(22.0, 16.0), "Solo");
                    if s.clicked() {
                        app.exec(Command::SetMixerSolo {
                            track: i,
                            solo: !track.solo,
                        });
                    }
                }
            });
            // Routed channel count.
            let (r, _) = ui.allocate_exact_size(vec2(STRIP_W - 12.0, 12.0), Sense::hover());
            if !routed.is_empty() {
                ui.painter().text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    format!("{} ch", routed.len()),
                    mono(9.0),
                    t.accent_dim,
                );
            }
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
