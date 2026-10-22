//! Control context menus: value editing and automation clips. Values copy on the same
//! normalised scale as automation, so pasting between controls preserves their position.

use crate::app::LimeShotApp;
use crate::theme::Theme;
use egui::{Response, RichText};
use limeshot_core::{AutomationTarget, ClipId, ClipSource, Command};

/// Attach value and automation actions to an automatable control.
pub fn menu(app: &mut LimeShotApp, resp: &Response, target: AutomationTarget) {
    crate::widgets::clickable(resp).context_menu(|ui| {
        ui.label(
            RichText::new(target.label(app.project()))
                .small()
                .color(Theme::current().text_dim),
        );
        ui.separator();
        if let Some(default) = default_value(app, target) {
            if ui.button("Reset").clicked() {
                set_value(app, target, default);
                ui.close();
            }
            if ui.button("Copy value").clicked() {
                if let Some(mapping) = target.mapping(app.project()) {
                    app.control_clipboard = app
                        .automated(target)
                        .or_else(|| target.current_value(app.project()))
                        .map(|v| mapping.to_norm(v));
                }
                ui.close();
            }
            if ui
                .add_enabled(app.control_clipboard.is_some(), egui::Button::new("Paste value"))
                .clicked()
            {
                if let (Some(norm), Some(mapping)) = (app.control_clipboard, target.mapping(app.project())) {
                    set_value(app, target, mapping.to_value(norm));
                }
                ui.close();
            }
            if ui.button("Type in value…").clicked() {
                if let Some((text, _)) = input_value(app, target) {
                    app.dialog = Some(crate::dialogs::Dialog::ControlValue { target, text });
                }
                ui.close();
            }
            ui.separator();
        }
        if ui.button("Create automation clip").clicked() {
            create(app, target);
            ui.close();
        }
    });
}

fn default_value(app: &LimeShotApp, target: AutomationTarget) -> Option<f32> {
    use AutomationTarget::*;
    target.current_value(app.project())?;
    match target {
        ChannelVolume(_) => Some(0.78),
        ChannelPan(_) | MixerPan(_) => Some(0.0),
        MixerVolume(_) => Some(1.0),
        EffectParam { track, effect, index } => app
            .project()
            .mixer
            .tracks
            .get(usize::from(track))?
            .effects
            .iter()
            .find(|e| e.id == effect)?
            .kind
            .params()
            .get(index)
            .map(|s| s.default),
        PluginParam { .. } => None,
    }
}

/// Apply a finite value through the same commands as dragging the control.
pub(crate) fn set_value(app: &mut LimeShotApp, target: AutomationTarget, value: f32) {
    use AutomationTarget::*;
    if !value.is_finite() || target.current_value(app.project()).is_none() {
        return;
    }
    let command = match target {
        ChannelVolume(channel) => Command::SetChannelVolume { channel, volume: value },
        ChannelPan(channel) => Command::SetChannelPan { channel, pan: value },
        MixerVolume(track) => Command::SetMixerVolume { track, volume: value },
        MixerPan(track) => Command::SetMixerPan { track, pan: value },
        EffectParam { track, effect, index } => Command::SetEffectParam {
            track,
            effect,
            index,
            value,
        },
        PluginParam { .. } => return,
    };
    app.exec(command);
}

/// Use the control's displayed units for typing; copied values use normalised travel instead.
pub(crate) fn input_value(app: &LimeShotApp, target: AutomationTarget) -> Option<(String, &'static str)> {
    use AutomationTarget::*;
    let v = app.automated(target).or_else(|| target.current_value(app.project()))?;
    match target {
        ChannelVolume(_) | ChannelPan(_) | MixerPan(_) => Some(((v * 100.0).to_string(), "%")),
        MixerVolume(_) => Some((
            if v <= 0.0 {
                "-inf".into()
            } else {
                limeshot_dsp::gain_to_db(v).to_string()
            },
            "dB",
        )),
        EffectParam { track, effect, index } => {
            let spec = app
                .project()
                .mixer
                .tracks
                .get(usize::from(track))?
                .effects
                .iter()
                .find(|e| e.id == effect)?
                .kind
                .params()
                .get(index)?;
            Some((v.to_string(), spec.unit))
        }
        PluginParam { .. } => None,
    }
}

pub(crate) fn parse_value(app: &LimeShotApp, target: AutomationTarget, text: &str) -> Option<f32> {
    use AutomationTarget::*;
    let mapping = target.mapping(app.project())?;
    if matches!(target, MixerVolume(_)) && text.trim() == "-inf" {
        return Some(0.0);
    }
    let typed: f32 = text.trim().parse().ok()?;
    if !typed.is_finite() {
        return None;
    }
    let value = match target {
        ChannelVolume(_) | ChannelPan(_) | MixerPan(_) => typed / 100.0,
        MixerVolume(_) if (-60.0..=limeshot_dsp::gain_to_db(2.0)).contains(&typed) => {
            limeshot_dsp::db_to_gain(typed).min(2.0)
        }
        MixerVolume(_) | PluginParam { .. } => return None,
        EffectParam { .. } => typed,
    };
    (value.is_finite() && (mapping.to_value(0.0)..=mapping.to_value(1.0)).contains(&value)).then_some(value)
}

/// Create an automation clip for `target` on the first Playlist track that is free for the
/// length of the song, and show it.
pub fn create(app: &mut LimeShotApp, target: AutomationTarget) -> Option<ClipId> {
    create_with(app, target, None, None)
}

/// [`create`] for a control whose value the project does not hold (a plugin parameter):
/// `value` is its normalised value now and `name` the automation's name.
pub fn create_with(
    app: &mut LimeShotApp,
    target: AutomationTarget,
    value: Option<f32>,
    name: Option<String>,
) -> Option<ClipId> {
    let sig = app.project().time_signature;
    let length = app.project().playlist.song_length(sig).max(sig.ticks_per_bar());
    let playlist = &app.project().playlist;
    let free =
        (0..playlist.tracks.len() as u32).find(|t| !playlist.clips.iter().any(|c| c.track == *t && c.start < length));
    let track = match free {
        Some(t) => t,
        None => {
            app.exec(Command::AddPlaylistTrack)?;
            app.project().playlist.tracks.len() as u32 - 1
        }
    };
    let id = ClipId(app.exec(Command::AddAutomationClip {
        target,
        track,
        start: 0,
        length,
        value,
        name,
    })?);
    if let Some(a) = app.project().clip(id).and_then(|c| c.automation()) {
        app.ui.place = Some(ClipSource::Automation(a));
    }
    app.show_window(crate::workspace::Win::Playlist);
    let name = app
        .project()
        .playlist
        .tracks
        .get(track as usize)
        .map(|t| t.name.clone())
        .unwrap_or_default();
    app.toast(format!("Automation clip created on {name}"));
    Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use limeshot_core::{ChannelId, EffectKind, Instrument, SamplerParams};

    #[test]
    fn typed_values_use_control_units_and_reject_invalid_or_deleted_targets() {
        let mut app = LimeShotApp::new_silent(crate::Settings::default(), None);
        let channel = ChannelId(
            app.exec(Command::AddChannel {
                name: "Test".into(),
                instrument: Instrument::Sampler(SamplerParams::default()),
            })
            .unwrap(),
        );
        let effect = app
            .exec(Command::AddEffect {
                track: 1,
                kind: EffectKind::Filter,
            })
            .unwrap();
        for (target, text, expected) in [
            (AutomationTarget::ChannelVolume(channel), "78", 0.78),
            (AutomationTarget::ChannelPan(channel), "-50", -0.5),
            (AutomationTarget::MixerPan(1), "100", 1.0),
            (AutomationTarget::MixerVolume(1), "0", 1.0),
            (AutomationTarget::MixerVolume(1), "-inf", 0.0),
            (
                AutomationTarget::EffectParam {
                    track: 1,
                    effect,
                    index: 1,
                },
                "1000",
                1000.0,
            ),
        ] {
            let v = parse_value(&app, target, text).expect("valid value");
            assert!((v - expected).abs() < 1e-5);
            set_value(&mut app, target, v);
            assert!((target.current_value(app.project()).unwrap() - expected).abs() < 1e-5);
            let (text, _) = input_value(&app, target).unwrap();
            assert!((parse_value(&app, target, &text).unwrap() - expected).abs() < 1e-5);
        }
        for text in ["NaN", "inf", "-inf", "", "abc", "101", "-101"] {
            assert_eq!(parse_value(&app, AutomationTarget::MixerPan(1), text), None);
        }
        assert!(parse_value(&app, AutomationTarget::ChannelVolume(channel), "126").is_none());
        assert!(parse_value(&app, AutomationTarget::MixerVolume(1), "7").is_none());
        let before = app.project().clone();
        set_value(&mut app, AutomationTarget::MixerPan(1), f32::NAN);
        assert_eq!(app.project(), &before);
        let deleted = AutomationTarget::ChannelVolume(ChannelId(u32::MAX));
        assert!(input_value(&app, deleted).is_none());
        assert!(parse_value(&app, deleted, "20").is_none());
    }
}
