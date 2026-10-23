//! Presets menus for the built-in effects (on each effect card) and instruments (in Channel
//! Settings): LimeShot's factory presets, then the user's own, and Save preset as…. Files and
//! factory presets are in `limeshot_project::preset`.

use crate::app::LimeShotApp;
use crate::dialogs::{Dialog, RenameTarget};
use egui::Ui;
use limeshot_core::*;
use limeshot_project::preset::{self, Content, Preset, Slot};

/// What a presets menu loads into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Effect { track: u16, effect: u32 },
    Channel(ChannelId),
}

/// The preset slot and current settings of `target`, if it has presets (built-in effects and
/// the sampler and synth do; plugins have their own).
fn current(app: &LimeShotApp, target: Target) -> Option<(Slot, Content)> {
    match target {
        Target::Effect { track, effect } => {
            let e = app
                .project()
                .mixer
                .tracks
                .get(usize::from(track))?
                .effects
                .iter()
                .find(|e| e.id == effect)?;
            if let Some(plugin) = &e.plugin
                && let Some(lime) = LimePlugin::of(plugin)
                && let Some(kind) = LimePlugin::EFFECTS
                    .iter()
                    .copied()
                    .find(|kind| LimePlugin::for_effect(*kind) == Some(lime))
            {
                let mut plugin = plugin.clone();
                if let Some(Ok(state)) = app
                    .plugins
                    .instance(PluginTarget::Effect { track, effect })
                    .map(|i| i.state())
                {
                    plugin.state = PluginState::new(state);
                }
                return Some((
                    Slot::Effect(kind),
                    Content::Effect {
                        kind,
                        params: lime::plain_values(&plugin, &kind.defaults()),
                    },
                ));
            }
            (e.kind != EffectKind::Plugin).then(|| {
                (
                    Slot::Effect(e.kind),
                    Content::Effect {
                        kind: e.kind,
                        params: e.params.clone(),
                    },
                )
            })
        }
        Target::Channel(c) => match &app.project().channel(c)?.instrument {
            Instrument::Synth(s) => Some((Slot::Synth, Content::Synth(s.clone()))),
            Instrument::Sampler(s) => Some((Slot::Sampler, Content::Sampler(s.clone()))),
            Instrument::Plugin(p) => {
                let mut p = p.clone();
                if let Some(Ok(state)) = app.plugins.instance(PluginTarget::Channel(c)).map(|i| i.state()) {
                    p.state = PluginState::new(state);
                }
                lime::synth_params(&p)
                    .map(|s| (Slot::Synth, Content::Synth(s)))
                    .or_else(|| {
                        lime::sampler_params(&Instrument::Plugin(p)).map(|s| (Slot::Sampler, Content::Sampler(s)))
                    })
            }
            _ => None,
        },
    }
}

/// The Presets menu button.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui, target: Target) {
    let Some((slot, content)) = current(app, target) else {
        return;
    };
    ui.menu_button("Presets", |ui| {
        let entries = preset::list(app.presets_dir.as_deref(), slot);
        let factory = entries.iter().filter(|e| e.path.is_none()).count();
        for (k, entry) in entries.iter().enumerate() {
            if k == factory && k > 0 {
                ui.separator();
            }
            if ui.button(&entry.name).clicked() {
                match preset::open(entry, slot) {
                    Ok(p) => apply(app, target, p.content),
                    Err(e) => app.toast_error(e.to_string()),
                }
                ui.close();
            }
        }
        ui.separator();
        let can_save = app.presets_dir.is_some();
        if ui.add_enabled(can_save, egui::Button::new("Save preset as…")).clicked() {
            app.dialog = Some(Dialog::Rename {
                target: RenameTarget::Preset(Box::new(Preset {
                    name: String::new(),
                    content: content.clone(),
                })),
                text: String::new(),
            });
            ui.close();
        }
    });
}

/// Load preset settings into `target` (one undo step). A sampler keeps its own sample.
pub fn apply(app: &mut LimeShotApp, target: Target, content: Content) {
    match (target, content) {
        (Target::Effect { track, effect }, Content::Effect { kind, params }) => {
            let target = PluginTarget::Effect { track, effect };
            if let Some(plugin) = app.project().plugin(target)
                && let Some(lime) = LimePlugin::of(plugin)
            {
                if LimePlugin::for_effect(kind) != Some(lime) {
                    app.toast_error("That preset does not fit here".to_string());
                    return;
                }
                if let Some(state) = limeshot_core::lime::edited_values_state(plugin, &params) {
                    app.exec(Command::SetPluginState { plugin: target, state });
                }
            } else {
                app.exec(Command::SetEffectParams { track, effect, params });
            }
        }
        (Target::Channel(channel), Content::Synth(s)) => {
            app.exec(Command::SetInstrument {
                channel,
                instrument: Instrument::Synth(s),
            });
        }
        (Target::Channel(channel), Content::Sampler(mut s)) => {
            if let Some(cur) = app
                .project()
                .channel(channel)
                .and_then(|c| lime::sampler_params(&c.instrument))
            {
                s.sample = cur.sample;
            }
            app.exec(Command::SetInstrument {
                channel,
                instrument: Instrument::Sampler(s),
            });
        }
        _ => app.toast_error("That preset does not fit here".to_string()),
    }
}

/// Save preset as… (from the name dialog).
pub fn save(app: &mut LimeShotApp, preset: &Preset, name: &str) {
    let Some(dir) = app.presets_dir.clone() else { return };
    let p = Preset {
        name: name.to_string(),
        content: preset.content.clone(),
    };
    match preset::save(&dir, &p) {
        Ok(_) => app.hint(format!("Saved preset \"{name}\"")),
        Err(e) => app.toast_error(e.to_string()),
    }
}
