//! Insert effect chain editor, shown in the Inspector for the selected mixer track.

use crate::app::LimeShotApp;
use crate::theme::{Theme, mix, semibold};
use crate::widgets::Hint as _;
use crate::widgets::{section, segmented, toggle_label};
use egui::{CornerRadius, RichText, Stroke, Ui, vec2};
use limeshot_core::{AutomationTarget, COMPRESSOR_SIDECHAIN, Command, EffectKind, EffectSlot, MAX_EFFECTS};

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
                    ui.separator();
                    if let Some(e) = crate::plugins::menu(app, ui, false) {
                        app.exec(Command::AddPluginEffect {
                            track,
                            plugin: e.plugin_ref(),
                        });
                        ui.close();
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
                    let name = e.name();
                    let label = ui.add(
                        egui::Label::new(
                            RichText::new(format!("{}. {name}", pos + 1))
                                .font(semibold(12.0))
                                .color(if e.enabled { t.text } else { t.text_dim }),
                        )
                        .sense(egui::Sense::click()),
                    );
                    // FL Studio: a click on a plugin slot opens (or closes) the plugin's window.
                    let target = crate::plugins::Target::Effect { track, effect: e.id };
                    if e.kind == EffectKind::Plugin && app.plugins.is_loaded(target) {
                        let label = label.hint("Click to open or close the plugin's window");
                        if label.clicked() {
                            crate::plugin_window::toggle(app, target);
                        }
                    }
                    crate::presets_ui::menu(app, ui, crate::presets_ui::Target::Effect { track, effect: e.id });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").hint("Remove").clicked() {
                            app.exec(Command::RemoveEffect { track, effect: e.id });
                        }
                        if ui
                            .add_enabled(pos + 1 < effects.len(), egui::Button::new("↓").small())
                            .hint("Move down")
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
                            .hint("Move up")
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
                if e.kind == EffectKind::Plugin {
                    crate::plugins::effect_card(app, ui, track, e.id, e.plugin.as_ref());
                }
                for (i, spec) in e.kind.params().iter().enumerate() {
                    let cur = e.params.get(i).copied().unwrap_or(spec.default);
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            vec2(64.0, 20.0),
                            egui::Label::new(RichText::new(spec.name).color(t.text_dim)),
                        );
                        if e.kind == EffectKind::Compressor && i == COMPRESSOR_SIDECHAIN {
                            sidechain_choice(app, ui, track, e.id, cur.round() as u16);
                            return;
                        }
                        if spec.labels.len() > 4 {
                            // Many choices (the delay's Sync): a drop-down, not a segmented row.
                            let mut choice = cur.round() as usize;
                            egui::ComboBox::from_id_salt((track, e.id, i))
                                .width(90.0)
                                .selected_text(spec.labels.get(choice).copied().unwrap_or("?"))
                                .show_ui(ui, |ui| {
                                    for (k, l) in spec.labels.iter().enumerate() {
                                        ui.selectable_value(&mut choice, k, *l);
                                    }
                                });
                            if choice != cur.round() as usize {
                                app.exec(Command::SetEffectParam {
                                    track,
                                    effect: e.id,
                                    index: i,
                                    value: choice as f32,
                                });
                            }
                            return;
                        }
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
                        // A tempo-synced delay ignores its Time.
                        let synced = e.kind == EffectKind::Delay
                            && i == 0
                            && e.params.get(limeshot_core::DELAY_SYNC).is_some_and(|s| *s >= 1.0);
                        ui.add_enabled_ui(!synced, |ui| {
                            ui.spacing_mut().slider_width = (ui.available_width() - 74.0).max(60.0);
                            let unit = spec.unit;
                            let r = crate::widgets::reset_on_double_click(
                                ui.add(
                                    egui::Slider::new(&mut v, spec.min..=spec.max)
                                        .logarithmic(spec.log)
                                        .custom_formatter(move |x, _| format_value(x as f32, unit)),
                                ),
                                &mut v,
                                spec.default,
                            );
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
                    });
                }
                if e.kind == EffectKind::Compressor {
                    reduction_meter(app, ui, track);
                }
            });
        ui.add_space(4.0);
    }
}

/// The compressor's sidechain input: its own signal, or a track that routes here (FL Studio's
/// "Sidechain to this track" makes such a route at level 0).
fn sidechain_choice(app: &mut LimeShotApp, ui: &mut Ui, track: u16, effect: u32, cur: u16) {
    let mixer = &app.project().mixer;
    let name = |j: u16| {
        mixer
            .tracks
            .get(usize::from(j))
            .map(|x| x.name.clone())
            .unwrap_or_default()
    };
    let sources: Vec<(u16, String)> = mixer
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, x)| x.routes.iter().any(|r| r.to == track))
        .filter_map(|(j, _)| u16::try_from(j).ok())
        .map(|j| (j, format!("{j}. {}", name(j))))
        .collect();
    let text = match cur {
        0 => "Off (own input)".to_string(),
        j if sources.iter().any(|s| s.0 == j) => format!("{j}. {}", name(j)),
        j => format!("{j}. {} (not routed here)", name(j)),
    };
    let mut pick = None;
    egui::ComboBox::from_id_salt(("sidechain", track, effect))
        .selected_text(text)
        .width(ui.available_width() - 8.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(cur == 0, "Off (own input)").clicked() {
                pick = Some(0);
            }
            for (j, label) in &sources {
                if ui.selectable_label(cur == *j, label).clicked() {
                    pick = Some(*j);
                }
            }
            if sources.is_empty() {
                ui.label(
                    RichText::new("Route a track here first (right-click its route button: Sidechain to this track)")
                        .small(),
                );
            }
        })
        .response
        .hint("The track whose signal drives the compressor; it must route to this track");
    if let Some(j) = pick
        && j != cur
    {
        app.exec(Command::SetEffectParam {
            track,
            effect,
            index: COMPRESSOR_SIDECHAIN,
            value: f32::from(j),
        });
    }
}

/// Gain reduction of the track's compressors, as a bar growing from the right (0–24 dB).
fn reduction_meter(app: &LimeShotApp, ui: &mut Ui, track: u16) {
    let t = Theme::current();
    let gr = app.player.status().gain_reduction(usize::from(track)).clamp(0.0, 24.0);
    ui.horizontal(|ui| {
        ui.add_sized(
            vec2(64.0, 16.0),
            egui::Label::new(RichText::new("Reduction").color(t.text_dim)),
        );
        let w = (ui.available_width() - 74.0).max(60.0);
        let (r, _) = ui.allocate_exact_size(vec2(w, 8.0), egui::Sense::hover());
        ui.painter().rect_filled(r, CornerRadius::same(2), t.sunken);
        let fill = egui::Rect::from_min_max(egui::pos2(r.right() - r.width() * gr / 24.0, r.top()), r.max);
        ui.painter().rect_filled(fill, CornerRadius::same(2), t.warning);
        ui.label(
            RichText::new(if gr < 0.05 {
                "0.0 dB".to_string()
            } else {
                format!("-{gr:.1} dB")
            })
            .font(crate::theme::mono(10.0))
            .color(t.text_dim),
        );
    });
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
        .map(|e| (e.name().trim_start_matches("Lime ").to_string(), e.enabled))
        .collect()
}
