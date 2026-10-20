//! Insert effect chain editor, shown in the Inspector for the selected mixer track.

use crate::app::LimeShotApp;
use crate::theme::{Theme, mix, semibold};
use crate::widgets::{section, segmented, toggle_label};
use egui::{CornerRadius, RichText, Stroke, Ui, vec2};
use limeshot_core::{AutomationTarget, Command, EffectKind, EffectSlot, MAX_EFFECTS};

pub fn chain(app: &mut LimeShotApp, ui: &mut Ui, track: u16) {
    let t = Theme::current();
    ui.add_space(10.0);
    let effects: Vec<EffectSlot> = app
        .project()
        .mixer
        .tracks
        .get(usize::from(track))
        .map(|t| t.effects.clone())
        .unwrap_or_default();
    ui.horizontal(|ui| {
        section(ui, &format!("Inserts  {}/{}", effects.len(), MAX_EFFECTS));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled_ui(effects.len() < MAX_EFFECTS, |ui| {
                ui.menu_button("+ Add effect", |ui| {
                    for kind in EffectKind::ALL {
                        if ui.button(kind.label()).clicked() {
                            app.exec(Command::AddEffect { track, kind });
                            ui.close();
                        }
                    }
                });
            });
        });
    });
    if effects.is_empty() {
        ui.label(RichText::new("No effects. Effects process the track before its fader.").color(t.text_faint));
    }
    for (pos, e) in effects.iter().enumerate() {
        egui::Frame::new()
            .fill(mix(t.surface, t.raised, 0.5))
            .stroke(Stroke::new(1.0, t.border))
            .corner_radius(CornerRadius::same(t.radius))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let on = toggle_label(
                        ui,
                        e.enabled,
                        if e.enabled { "ON" } else { "OFF" },
                        t.accent,
                        vec2(30.0, 18.0),
                        "Enable / bypass",
                    );
                    if on.clicked() {
                        app.exec(Command::SetEffectEnabled {
                            track,
                            effect: e.id,
                            enabled: !e.enabled,
                        });
                    }
                    ui.label(
                        RichText::new(format!("{}. {}", pos + 1, e.kind.label()))
                            .font(semibold(12.0))
                            .color(if e.enabled { t.text } else { t.text_dim }),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").on_hover_text("Remove").clicked() {
                            app.exec(Command::RemoveEffect { track, effect: e.id });
                        }
                        if ui
                            .add_enabled(pos + 1 < effects.len(), egui::Button::new("↓").small())
                            .on_hover_text("Move down")
                            .clicked()
                        {
                            app.exec(Command::MoveEffect {
                                track,
                                effect: e.id,
                                delta: 1,
                            });
                        }
                        if ui
                            .add_enabled(pos > 0, egui::Button::new("↑").small())
                            .on_hover_text("Move up")
                            .clicked()
                        {
                            app.exec(Command::MoveEffect {
                                track,
                                effect: e.id,
                                delta: -1,
                            });
                        }
                    });
                });
                ui.add_space(2.0);
                for (i, spec) in e.kind.params().iter().enumerate() {
                    let cur = e.params.get(i).copied().unwrap_or(spec.default);
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            vec2(64.0, 20.0),
                            egui::Label::new(RichText::new(spec.name).color(t.text_dim)),
                        );
                        if !spec.labels.is_empty() {
                            if let Some(choice) = segmented(ui, spec.labels, cur.round() as usize, 22.0, spec.name) {
                                app.exec(Command::SetEffectParam {
                                    track,
                                    effect: e.id,
                                    index: i,
                                    value: choice as f32,
                                });
                            }
                            return;
                        }
                        let mut v = cur;
                        ui.spacing_mut().slider_width = (ui.available_width() - 74.0).max(60.0);
                        let unit = spec.unit;
                        let r = ui.add(
                            egui::Slider::new(&mut v, spec.min..=spec.max)
                                .logarithmic(spec.log)
                                .custom_formatter(move |x, _| format_value(x as f32, unit)),
                        );
                        if r.double_clicked() {
                            app.exec(Command::SetEffectParam {
                                track,
                                effect: e.id,
                                index: i,
                                value: spec.default,
                            });
                            return;
                        }
                        let cmd = crate::widgets::user_changed(&r).then_some(Command::SetEffectParam {
                            track,
                            effect: e.id,
                            index: i,
                            value: v,
                        });
                        app.exec_gesture(&r, cmd);
                        crate::automation_ui::menu(
                            app,
                            &r,
                            AutomationTarget::EffectParam {
                                track,
                                effect: e.id,
                                index: i,
                            },
                        );
                    });
                }
            });
        ui.add_space(4.0);
    }
}

fn format_value(v: f32, unit: &str) -> String {
    match unit {
        "Hz" if v >= 1000.0 => format!("{:.2} kHz", v / 1000.0),
        "Hz" => format!("{v:.0} Hz"),
        "ms" => format!("{v:.0} ms"),
        "dB" => format!("{v:+.1} dB"),
        ":1" => format!("{v:.1}:1"),
        "×" => format!("{v:.2}×"),
        _ => format!("{:.0}%", v * 100.0),
    }
}

/// Short names of a track's effects for the mixer strip.
pub fn strip_labels(slots: &[EffectSlot]) -> Vec<(String, bool)> {
    slots
        .iter()
        .map(|e| (e.kind.label().trim_start_matches("Lime ").to_string(), e.enabled))
        .collect()
}
