//! Insert effect chain editor, shown in the Inspector for the selected mixer track.

use crate::app::LimeShotApp;
use crate::theme::{Theme, mix, semibold};
use crate::widgets::Hint as _;
use crate::widgets::{section, toggle_label};
use egui::{CornerRadius, RichText, Stroke, Ui, vec2};
use limeshot_core::{Command, EffectKind, EffectSlot, MAX_EFFECTS};

/// Draw a mixer track's insert chain with controls for adding, editing, bypassing, and reordering effects.
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
                    for lime in limeshot_core::LimePlugin::ALL
                        .into_iter()
                        .filter(|p| !p.is_instrument())
                    {
                        if ui.button(lime.name()).clicked() {
                            app.exec(Command::AddPluginEffect {
                                track,
                                plugin: lime.plugin_ref(),
                            });
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
                    let on = ui
                        .add_enabled_ui(e.kind == EffectKind::Plugin, |ui| {
                            toggle_label(
                                ui,
                                e.enabled,
                                if e.enabled { "ON" } else { "OFF" },
                                t.accent,
                                vec2(30.0, 18.0),
                                "Enable / bypass",
                            )
                        })
                        .inner;
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
                });
                ui.horizontal(|ui| {
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
                if e.kind != EffectKind::Plugin {
                    ui.label(RichText::new("Legacy effect: not implemented as a Lime plugin yet").color(t.warning));
                }
            });
        ui.add_space(4.0);
    }
}

/// Short names of a track's effects for the mixer strip.
pub fn strip_labels(slots: &[EffectSlot]) -> Vec<(String, bool)> {
    slots
        .iter()
        .map(|e| (e.name().trim_start_matches("Lime ").to_string(), e.enabled))
        .collect()
}
