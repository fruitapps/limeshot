//! The application menu bar. Every item runs a command or an app action that also has a
//! shortcut or toolbar control.

use crate::app::{LimeShotApp, Pending};
use crate::dialogs::Dialog;
use crate::shortcuts::label as key;
use egui::{Button, Ui};
use limeshot_core::{Command, Instrument, PatternId, SamplerParams, SynthParams};

fn item(ui: &mut Ui, text: &str, shortcut: &str) -> bool {
    let clicked = ui.add(Button::new(text).shortcut_text(shortcut)).clicked();
    if clicked {
        ui.close();
    }
    clicked
}

fn toggle(ui: &mut Ui, on: bool, text: &str, shortcut: &str) -> bool {
    let clicked = ui.add(Button::selectable(on, text).shortcut_text(shortcut)).clicked();
    if clicked {
        ui.close();
    }
    clicked
}

pub fn add_sampler(app: &mut LimeShotApp) {
    let n = app.project().channels.len() + 1;
    if let Some(id) = app.exec(Command::AddChannel {
        name: format!("Sampler {n}"),
        instrument: Instrument::Sampler(SamplerParams::default()),
    }) {
        app.select_channel(limeshot_core::ChannelId(id));
    }
}

pub fn add_synth(app: &mut LimeShotApp) {
    let n = app.project().channels.len() + 1;
    if let Some(id) = app.exec(Command::AddChannel {
        name: format!("LimeSynth {n}"),
        instrument: Instrument::Synth(SynthParams::default()),
    }) {
        app.select_channel(limeshot_core::ChannelId(id));
    }
}

pub fn add_pattern(app: &mut LimeShotApp) {
    let n = app.project().patterns.len() + 1;
    if let Some(id) = app.exec(Command::AddPattern {
        name: format!("Pattern {n}"),
    }) {
        app.set_current_pattern(PatternId(id));
    }
}

/// The menu bar; returns the right edge of its last menu.
pub fn menu_bar(app: &mut LimeShotApp, ui: &mut Ui) -> f32 {
    egui::MenuBar::new()
        .ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if item(ui, "New", &key("Cmd+N")) {
                    app.request(Pending::New);
                }
                if item(ui, "Open…", &key("Cmd+O")) {
                    app.request(Pending::Open(None));
                }
                ui.menu_button("Open recent", |ui| {
                    let recent = app.settings.recent.clone();
                    if recent.is_empty() {
                        ui.label("No recent projects");
                    }
                    for p in recent {
                        if item(ui, &p.display().to_string(), "") {
                            app.request(Pending::Open(Some(p)));
                        }
                    }
                });
                ui.separator();
                if item(ui, "Save", &key("Cmd+S")) {
                    app.save();
                }
                if item(ui, "Save as…", &key("Cmd+Shift+S")) {
                    app.save_as();
                }
                ui.separator();
                if item(ui, "Import MIDI file…", "") {
                    let dialog = rfd::FileDialog::new().add_filter("MIDI file", &["mid", "midi"]);
                    if let Some(p) = dialog.pick_file() {
                        app.import_midi(&p);
                    }
                }
                ui.separator();
                if item(ui, "Export WAV…", &key("Cmd+R")) {
                    app.dialog = Some(Dialog::export(app));
                }
                if item(ui, "Export MIDI file (pattern)…", "") {
                    app.export_midi_dialog(false);
                }
                if item(ui, "Export MIDI file (song)…", "") {
                    app.export_midi_dialog(true);
                }
                ui.separator();
                if item(ui, "Quit", &key("Cmd+Q")) {
                    app.request(Pending::Quit);
                }
            });
            ui.menu_button("Edit", |ui| {
                let undo = app
                    .doc
                    .undo_label()
                    .map(|l| format!("Undo {l}"))
                    .unwrap_or_else(|| "Undo".into());
                if ui
                    .add_enabled(
                        app.doc.undo_label().is_some(),
                        Button::new(undo).shortcut_text(key("Cmd+Z")),
                    )
                    .clicked()
                {
                    app.doc.undo();
                    ui.close();
                }
                let redo = app
                    .doc
                    .redo_label()
                    .map(|l| format!("Redo {l}"))
                    .unwrap_or_else(|| "Redo".into());
                if ui
                    .add_enabled(
                        app.doc.redo_label().is_some(),
                        Button::new(redo).shortcut_text(key("Cmd+Shift+Z")),
                    )
                    .clicked()
                {
                    app.doc.redo();
                    ui.close();
                }
                ui.separator();
                ui.menu_button("History", |ui| {
                    let h = app.doc.history();
                    if h.is_empty() {
                        ui.label("Nothing to undo");
                    }
                    for (i, l) in h.iter().enumerate().rev().take(30) {
                        ui.label(format!("{}. {l}", i + 1));
                    }
                });
            });
            ui.menu_button("Add", |ui| {
                if item(ui, "Sampler channel", "") {
                    add_sampler(app);
                }
                if item(ui, "LimeSynth channel", "") {
                    add_synth(app);
                }
                ui.separator();
                if item(ui, "Pattern", &key("F4")) {
                    add_pattern(app);
                }
                if item(ui, "Playlist track", "") {
                    app.exec(Command::AddPlaylistTrack);
                }
                if item(ui, "Mixer track", "") {
                    app.exec(Command::AddMixerTrack);
                }
            });
            ui.menu_button("Patterns", |ui| {
                let cur = app.current_pattern();
                if item(ui, "New pattern", &key("F4")) {
                    add_pattern(app);
                }
                if item(ui, "Clone pattern", "")
                    && let Some(id) = app.exec(Command::ClonePattern(cur))
                {
                    app.set_current_pattern(PatternId(id));
                }
                if item(ui, "Rename pattern…", "") {
                    app.dialog = Some(Dialog::rename_pattern(app, cur));
                }
                crate::playlist::pattern_color_menu(app, ui, cur);
                let can_delete = app.project().patterns.len() > 1;
                if ui.add_enabled(can_delete, Button::new("Delete pattern")).clicked() {
                    app.exec(Command::RemovePattern(cur));
                    ui.close();
                }
                ui.separator();
                let pats: Vec<(PatternId, String)> =
                    app.project().patterns.iter().map(|p| (p.id, p.name.clone())).collect();
                for (id, name) in pats {
                    if toggle(ui, id == cur, &name, "") {
                        app.set_current_pattern(id);
                    }
                }
            });
            ui.menu_button("View", |ui| {
                use crate::workspace::Win;
                for w in [
                    Win::Playlist,
                    Win::ChannelRack,
                    Win::PianoRoll,
                    Win::Mixer,
                    Win::ChannelSettings,
                ] {
                    if toggle(ui, app.is_window_open(w), w.label(), &key(w.shortcut())) {
                        app.toggle_window(w);
                    }
                }
                if toggle(ui, app.ui.show_browser, "Browser", &key("Alt+F8")) {
                    app.ui.show_browser ^= true;
                }
                ui.separator();
                if item(ui, "Arrange windows", &key("Cmd+Shift+H")) {
                    app.ui.workspace.reset_layout = true;
                }
                ui.separator();
                if item(ui, "Zoom in", &key("Cmd+=")) {
                    app.settings.ui_scale = (app.settings.ui_scale + 0.1).min(3.0);
                }
                if item(ui, "Zoom out", &key("Cmd+-")) {
                    app.settings.ui_scale = (app.settings.ui_scale - 0.1).max(0.5);
                }
                if item(ui, "Actual size", &key("Cmd+0")) {
                    app.settings.ui_scale = 1.0;
                }
            });
            ui.menu_button("Options", |ui| {
                if item(ui, "Audio settings…", "") {
                    app.dialog = Some(Dialog::audio(app));
                }
                if toggle(ui, app.ui.metronome, "Metronome", &key("Cmd+M")) {
                    let m = !app.ui.metronome;
                    app.set_metronome(m);
                }
                if toggle(ui, app.ui.song_loop, "Loop song", "") {
                    let l = !app.ui.song_loop;
                    app.set_song_loop(l);
                }
                ui.menu_button("Autosave", |ui| {
                    for (m, label) in [
                        (0, "Off"),
                        (1, "Every minute"),
                        (2, "Every 2 minutes"),
                        (5, "Every 5 minutes"),
                        (10, "Every 10 minutes"),
                    ] {
                        if toggle(ui, app.settings.autosave_minutes == m, label, "") {
                            app.settings.autosave_minutes = m;
                            app.settings.save();
                        }
                    }
                });
            });
            ui.menu_button("Help", |ui| {
                if item(ui, "Keyboard shortcuts", &key("F1")) {
                    app.dialog = Some(Dialog::Shortcuts);
                }
                if item(ui, "About LimeShot", "") {
                    app.dialog = Some(Dialog::About);
                }
            })
            .response
            .rect
            .right()
        })
        .inner
}
