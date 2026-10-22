//! The Mixer's ⏵ track menu (FL Studio's Mixer menu: rename, colour, reset, routing, solo,
//! Move left / right, Delete, Insert one), also on a strip's right-click. It acts on the
//! selected track; the Mixer's keys run the same [`Act`]s.

use crate::app::LimeShotApp;
use crate::shortcuts::label as key;
use egui::{Button, Context, Key, Modifiers, Ui};
use limeshot_core::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Rename,
    RandomColor,
    Reset,
    ResetRouting,
    Unroute,
    Solo,
    MoveLeft,
    MoveRight,
    Delete,
    InsertOne,
}

fn item(app: &mut LimeShotApp, ui: &mut Ui, text: &str, keys: &str, act: Act, enabled: bool) {
    if ui
        .add_enabled(enabled, Button::new(text).shortcut_text(key(keys)))
        .clicked()
    {
        ui.close();
        run(app, act);
    }
}

/// The menu for the selected track.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui) {
    let i = app.ui.mixer.selected;
    let n = app.project().mixer.tracks.len() as u16;
    let insert = i > 0;
    item(app, ui, "Rename…", "F2", Act::Rename, true);
    ui.menu_button("Change color", |ui| {
        let cur = app
            .project()
            .mixer
            .tracks
            .get(usize::from(i))
            .and_then(|t| t.color)
            .unwrap_or(PALETTE[0]);
        if let Some(color) = crate::widgets::swatch_picker(ui, cur) {
            app.exec(Command::SetMixerTrackColor {
                track: i,
                color: Some(color),
            });
            ui.close();
        }
    });
    item(app, ui, "Random color", "", Act::RandomColor, true);
    ui.separator();
    item(app, ui, "Reset selected track(s) to default", "", Act::Reset, true);
    item(
        app,
        ui,
        "Reset selected track(s) routing",
        "",
        Act::ResetRouting,
        insert,
    );
    item(
        app,
        ui,
        "Unroute tracks routed to the selected track(s)",
        "",
        Act::Unroute,
        true,
    );
    item(app, ui, "Solo selected track(s)", "S", Act::Solo, insert);
    ui.separator();
    item(app, ui, "Move left", "Alt+←", Act::MoveLeft, i > 1);
    item(app, ui, "Move right", "Alt+→", Act::MoveRight, insert && i + 1 < n);
    item(app, ui, "Delete", "Delete", Act::Delete, insert);
    item(
        app,
        ui,
        "Insert one",
        "",
        Act::InsertOne,
        usize::from(n) < Mixer::MAX_TRACKS,
    );
}

pub fn run(app: &mut LimeShotApp, act: Act) {
    let i = app.ui.mixer.selected;
    let n = app.project().mixer.tracks.len() as u16;
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        return;
    };
    match act {
        Act::Rename => app.ui.mixer.start_rename(i, &track.name, false),
        Act::RandomColor => {
            let others: Vec<Rgb> = PALETTE.iter().copied().filter(|c| Some(*c) != track.color).collect();
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as usize)
                .unwrap_or(0);
            if let Some(&color) = others.get(seed % others.len().max(1)) {
                app.exec(Command::SetMixerTrackColor {
                    track: i,
                    color: Some(color),
                });
            }
        }
        Act::Reset => {
            app.exec(Command::ResetMixerTrack(i));
        }
        Act::ResetRouting if i > 0 => {
            app.exec(Command::ResetMixerRouting(i));
        }
        Act::Unroute => {
            app.exec(Command::UnrouteMixerTrack(i));
        }
        Act::Solo if i > 0 => {
            app.exec(Command::SetMixerSolo {
                track: i,
                solo: !track.solo,
            });
        }
        Act::MoveLeft | Act::MoveRight => {
            let to = if act == Act::MoveLeft { i.wrapping_sub(1) } else { i + 1 };
            if i == 0 || to == 0 || to >= n {
                return;
            }
            // Moved plugins restart under their new place: keep their sound.
            app.plugins.store_states(&mut app.doc);
            let rev = app.doc.revision();
            app.exec(Command::MoveMixerTrack { track: i, to });
            if app.doc.revision() != rev {
                app.ui.mixer.selected = to;
            }
        }
        Act::Delete if i > 0 => {
            app.plugins.store_states(&mut app.doc);
            app.exec(Command::RemoveMixerTrack(i));
            app.ui.mixer.selected = i.min(n.saturating_sub(2));
        }
        Act::InsertOne => {
            app.plugins.store_states(&mut app.doc);
            let at = (i + 1).min(n);
            if let Some(t) = app.exec(Command::InsertMixerTrack(at)) {
                app.ui.mixer.selected = t as u16;
            }
        }
        Act::ResetRouting | Act::Solo | Act::Delete => {}
    }
}

/// The Mixer's keys (FL Studio: Opt+←/→ move, Delete, S solo, F2 rename).
pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key as k;
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    for (mods, key, act) in [
        (Modifiers::ALT, Key::ArrowLeft, Act::MoveLeft),
        (Modifiers::ALT, Key::ArrowRight, Act::MoveRight),
        (Modifiers::NONE, Key::Delete, Act::Delete),
        (Modifiers::NONE, Key::Backspace, Act::Delete),
        (Modifiers::NONE, Key::S, Act::Solo),
        (Modifiers::NONE, Key::F2, Act::Rename),
    ] {
        if k(ctx, mods, key) {
            run(app, act);
        }
    }
}
