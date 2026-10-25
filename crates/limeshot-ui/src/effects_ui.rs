//! Insert effect chain editor, shown in the Inspector for the selected mixer track.

use crate::app::LimeShotApp;
use crate::theme::Theme;
use crate::widgets::Hint as _;
use crate::widgets::{section, toggle_label};
use egui::{CornerRadius, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
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
    let mut moved = None;
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
                // A boxed slot like the Channel Rack's channel buttons: click opens, drag reorders.
                let width = (ui.available_width() - 30.0 - 22.0 - 2.0 * ui.spacing().item_spacing.x).max(60.0);
                let (r, slot) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click_and_drag());
                let open = app.plugins.is_window_open(target);
                let fill = if slot.hovered() || slot.dragged() {
                    t.raised_hover
                } else {
                    t.raised
                };
                ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), fill);
                ui.painter().rect_filled(
                    Rect::from_min_size(r.min, vec2(3.0, r.height())),
                    CornerRadius::same(1),
                    if e.enabled { t.accent } else { t.text_faint },
                );
                ui.painter().rect_stroke(
                    r,
                    CornerRadius::same(t.radius_sm),
                    Stroke::new(1.0, if open { t.accent } else { t.border }),
                    StrokeKind::Inside,
                );
                let label = format!("{}. {}", pos + 1, e.name());
                slot.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
                let name = ui.painter().layout(
                    label.clone(),
                    egui::FontId::proportional(12.0),
                    if e.enabled { t.text } else { t.text_dim },
                    r.width() - 16.0,
                );
                let at = pos2(r.left() + 10.0, r.center().y - name.size().y / 2.0);
                ui.painter().with_clip_rect(r.shrink(2.0)).galley(at, name, t.text);
                if slot.clicked() {
                    crate::plugin_window::toggle(app, target);
                }
                slot.dnd_set_drag_payload(InsertDrag { track, pos });
                if let Some(from) = slot.dnd_release_payload::<InsertDrag>()
                    && from.track == track
                    && from.pos != pos
                {
                    moved = Some((from.pos, pos));
                }
                if let Some(from) = slot.dnd_hover_payload::<InsertDrag>()
                    && from.track == track
                    && from.pos != pos
                {
                    let y = if from.pos < pos { r.bottom() } else { r.top() };
                    ui.painter().hline(r.x_range(), y, Stroke::new(2.0, t.accent));
                }
                slot.hint("Click to open or close the plugin · drag to reorder");
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for (label, to, enabled, tip) in [
                        ("▲", pos.wrapping_sub(1), pos > 0, "Move up"),
                        ("▼", pos + 1, pos + 1 < effects.len(), "Move down"),
                    ] {
                        let b = egui::Button::new(RichText::new(label).size(8.0))
                            .min_size(vec2(18.0, 12.0))
                            .frame(false);
                        if ui.add_enabled(enabled, b).hint(tip).clicked() {
                            moved = Some((pos, to));
                        }
                    }
                });
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
            if ui.button("Open plugin").clicked() {
                app.plugins.open_window(target, None);
                ui.close();
            }
            for (name, to, enabled) in [
                ("Move up", pos.wrapping_sub(1), pos > 0),
                ("Move down", pos + 1, pos + 1 < effects.len()),
            ] {
                if ui.add_enabled(enabled, egui::Button::new(name)).clicked() {
                    moved = Some((pos, to));
                    ui.close();
                }
            }
            ui.separator();
            if ui.button(RichText::new("Remove").color(t.danger)).clicked() {
                app.exec(Command::RemoveEffect { track, effect: e.id });
                ui.close();
            }
        });
        ui.add_space(2.0);
    }
    if let Some((from, to)) = moved {
        move_insert(app, track, &effects, from, to);
    }
}

/// The insert slot being dragged: its track and position among the visible inserts.
#[derive(Clone, Copy)]
struct InsertDrag {
    track: u16,
    pos: usize,
}

/// Move the visible insert at `from` to `to`, counting the hidden track EQ slot in the real chain.
fn move_insert(app: &mut LimeShotApp, track: u16, visible: &[EffectSlot], from: usize, to: usize) {
    let (Some(e), Some(dest)) = (visible.get(from), visible.get(to)) else {
        return;
    };
    let Some(chain) = app.project().mixer.tracks.get(usize::from(track)) else {
        return;
    };
    let index = |id: u32| chain.effects.iter().position(|x| x.id == id);
    let (Some(i), Some(j)) = (index(e.id), index(dest.id)) else {
        return;
    };
    app.exec(Command::MoveEffect {
        track,
        effect: e.id,
        delta: j as i32 - i as i32,
    });
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
