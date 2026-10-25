//! Insert effect chain editor, shown in the Inspector for the selected mixer track.

use crate::app::LimeShotApp;
use crate::theme::Theme;
use crate::widgets::{section, toggle_label};
use egui::{RichText, Ui, vec2};
use limeshot_core::{Command, EffectSlot, MAX_EFFECTS};

/// Draw a mixer track's insert chain with controls for adding, editing, bypassing, and reordering effects.
pub fn chain(app: &mut LimeShotApp, ui: &mut Ui, track: u16) {
    let t = Theme::current();
    ui.add_space(10.0);
    let effects: Vec<EffectSlot> = app
        .project()
        .mixer
        .tracks
        .get(usize::from(track))
        .map(|t| {
            t.effects
                .iter()
                .filter(|e| !limeshot_core::lime::is_track_eq(e))
                .cloned()
                .collect()
        })
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
                            add_and_open(app, track, lime.plugin_ref());
                            ui.close();
                        }
                    }
                    ui.separator();
                    if let Some(e) = crate::plugins::menu(app, ui, false) {
                        add_and_open(app, track, e.plugin_ref());
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
        let target = crate::plugins::Target::Effect { track, effect: e.id };
        let row = ui
            .horizontal(|ui| {
                let on = toggle_label(
                    ui,
                    e.enabled,
                    if e.enabled { "ON" } else { "OFF" },
                    t.accent,
                    vec2(30.0, 22.0),
                    "Enable / bypass",
                );
                if on.clicked() {
                    app.exec(Command::SetEffectEnabled {
                        track,
                        effect: e.id,
                        enabled: !e.enabled,
                    });
                }
                let name = ui.add_sized(
                    vec2((ui.available_width() - 66.0).max(60.0), 24.0),
                    egui::Button::new(
                        egui::RichText::new(format!("{}. {}", pos + 1, e.name())).color(if e.enabled {
                            t.text
                        } else {
                            t.text_dim
                        }),
                    )
                    .frame(false),
                );
                if name.clicked() {
                    crate::plugin_window::toggle(app, target);
                }
                let mut mix = e.mix;
                let r = crate::widgets::knob(ui, &mut mix, 0.0, 1.0, 1.0, 22.0, false, |v| {
                    format!("Mix: {:.0}%", v * 100.0)
                });
                let command = crate::widgets::user_changed(&r).then_some(Command::SetEffectMix {
                    track,
                    effect: e.id,
                    mix,
                });
                app.exec_gesture(&r, command);
            })
            .response;
        row.context_menu(|ui| {
            if ui.button("Remove").clicked() {
                app.exec(Command::RemoveEffect { track, effect: e.id });
                ui.close();
            }
            for (name, delta, enabled) in [("Move up", -1, pos > 0), ("Move down", 1, pos + 1 < effects.len())] {
                if ui.add_enabled(enabled, egui::Button::new(name)).clicked() {
                    app.exec(Command::MoveEffect {
                        track,
                        effect: e.id,
                        delta,
                    });
                    ui.close();
                }
            }
        });
        ui.separator();
    }
}

fn add_and_open(app: &mut LimeShotApp, track: u16, plugin: limeshot_core::PluginRef) {
    if let Some(effect) = app.exec(Command::AddPluginEffect { track, plugin }) {
        app.sync_engine();
        app.plugins
            .open_window(crate::plugins::Target::Effect { track, effect }, None);
    }
}

/// Short names of a track's effects for the mixer strip.
pub fn strip_labels(slots: &[EffectSlot]) -> Vec<(String, bool)> {
    slots
        .iter()
        .filter(|e| !limeshot_core::lime::is_track_eq(e))
        .map(|e| (e.name().trim_start_matches("Lime ").to_string(), e.enabled))
        .collect()
}
