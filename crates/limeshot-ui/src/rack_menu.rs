//! The Channel Rack's ⏵ options menu (FL Studio's Channel Rack menu): commands on the selected
//! channels — clone, delete, move, sort, group, colour, mute, transpose, mixer tracks.

use crate::app::LimeShotApp;
use crate::channel_rack::{move_selected, request_delete, route_to_free};
use crate::shortcuts::label as key;
use crate::workspace::Win;
use egui::{Button, Context, Key, Modifiers, Ui};
use limeshot_core::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    CloneSelected,
    DeleteSelected,
    MoveUp,
    MoveDown,
    SortByName,
    SortByType,
    SortByColor,
    GroupSelected,
    RandomColor,
    MuteSelected,
    UnmuteSelected,
    TransposeSelected,
    AssignFreeTracks,
    SelectMixerTracks,
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

/// The ⏵ menu at the left of the Channel Rack's header.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui) {
    let any = !app.selected_channels().is_empty();
    ui.menu_button("Add one", |ui| {
        if ui.button("Sampler").clicked() {
            crate::menus::add_sampler(app);
            ui.close();
        }
        if ui.button("LimeSynth").clicked() {
            crate::menus::add_synth(app);
            ui.close();
        }
        ui.separator();
        if let Some(e) = crate::plugins::menu(app, ui, true) {
            crate::plugins::add_channel(app, &e);
            ui.close();
        }
    });
    ui.separator();
    item(app, ui, "Clone selected", "Alt+C", Act::CloneSelected, any);
    item(app, ui, "Delete selected…", "Alt+Delete", Act::DeleteSelected, any);
    item(app, ui, "Move selected up", "Alt+↑", Act::MoveUp, any);
    item(app, ui, "Move selected down", "Alt+↓", Act::MoveDown, any);
    ui.menu_button("Sort by", |ui| {
        item(app, ui, "Name", "", Act::SortByName, true);
        item(app, ui, "Type", "", Act::SortByType, true);
        item(app, ui, "Color", "", Act::SortByColor, true);
    });
    item(app, ui, "Group selected…", "Alt+G", Act::GroupSelected, any);
    ui.add_enabled_ui(any, |ui| {
        ui.menu_button("Color selected", |ui| {
            let cur = app
                .selected_channels()
                .first()
                .and_then(|c| app.project().channel(*c))
                .map_or(PALETTE[0], |c| c.color);
            if let Some(color) = crate::widgets::swatch_picker(ui, cur) {
                let cmds = app
                    .selected_channels()
                    .into_iter()
                    .map(|channel| Command::SetChannelColor { channel, color })
                    .collect();
                app.exec(Command::Batch(cmds));
                ui.close();
            }
        });
    });
    item(app, ui, "Random color", "", Act::RandomColor, any);
    item(app, ui, "Mute selected", "", Act::MuteSelected, any);
    item(app, ui, "Unmute selected", "", Act::UnmuteSelected, any);
    item(app, ui, "Transpose selected…", "", Act::TransposeSelected, any);
    ui.separator();
    item(
        app,
        ui,
        "Assign selected to free mixer track(s)",
        "Cmd+L",
        Act::AssignFreeTracks,
        any,
    );
    item(
        app,
        ui,
        "Select mixer tracks for the selected channels",
        "",
        Act::SelectMixerTracks,
        any,
    );
}

/// The instrument's kind, for Sort by › Type.
fn type_rank(c: &Channel) -> (u8, String) {
    match &c.instrument {
        Instrument::Sampler(_) => (0, String::new()),
        Instrument::Synth(_) => (1, String::new()),
        Instrument::Plugin(p) => (2, p.name.to_lowercase()),
        Instrument::Unsupported(_) => (3, String::new()),
    }
}

/// The colour's hue in degrees (0–359), for Sort by › Color.
fn hue(Rgb(r, g, b): Rgb) -> i32 {
    let (r, g, b) = (f32::from(r), f32::from(g), f32::from(b));
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    if max == min {
        return 0;
    }
    let d = max - min;
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0) as i32
}

pub fn run(app: &mut LimeShotApp, act: Act) {
    let sel = app.selected_channels();
    let each = |app: &mut LimeShotApp, f: &dyn Fn(ChannelId) -> Command| {
        let cmds: Vec<Command> = app.selected_channels().into_iter().map(f).collect();
        if !cmds.is_empty() {
            app.exec(Command::Batch(cmds));
        }
    };
    match act {
        Act::CloneSelected => each(app, &Command::CloneChannel),
        Act::DeleteSelected => request_delete(app, sel),
        Act::MoveUp => move_selected(app, -1),
        Act::MoveDown => move_selected(app, 1),
        Act::SortByName | Act::SortByType | Act::SortByColor => {
            let mut chans: Vec<Channel> = app.project().channels.clone();
            match act {
                Act::SortByName => chans.sort_by_key(|c| c.name.to_lowercase()),
                Act::SortByType => chans.sort_by_key(type_rank),
                _ => chans.sort_by_key(|c| hue(c.color)),
            }
            app.exec(Command::ReorderChannels(chans.iter().map(|c| c.id).collect()));
        }
        Act::GroupSelected if !sel.is_empty() => {
            app.dialog = Some(crate::dialogs::Dialog::group_channels(app, sel));
        }
        Act::GroupSelected => {}
        Act::RandomColor => {
            let mut seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as usize)
                .unwrap_or(0);
            let cmds: Vec<Command> = sel
                .iter()
                .map(|&channel| {
                    seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    Command::SetChannelColor {
                        channel,
                        color: PALETTE[(seed >> 8) % PALETTE.len()],
                    }
                })
                .collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::MuteSelected => each(app, &|channel| Command::SetChannelMute { channel, muted: true }),
        Act::UnmuteSelected => each(app, &|channel| Command::SetChannelMute { channel, muted: false }),
        Act::TransposeSelected if !sel.is_empty() => {
            app.dialog = Some(crate::dialogs::Dialog::Transpose {
                pattern: app.current_pattern(),
                channels: sel,
                semitones: 0,
            });
        }
        Act::TransposeSelected => {}
        Act::AssignFreeTracks => route_to_free(app, &sel),
        Act::SelectMixerTracks => {
            if let Some(t) = sel
                .first()
                .and_then(|c| app.project().channel(*c))
                .map(|c| c.mixer_track)
            {
                app.ui.mixer.selected = t;
                app.show_window(Win::Mixer);
            }
        }
    }
}

/// The menu's keys that the Channel Rack does not already handle (FL Studio: Opt+C).
pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    if crate::shortcuts::key(ctx, Modifiers::ALT, Key::C) {
        run(app, Act::CloneSelected);
    }
}
