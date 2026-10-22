//! Modal-style dialogs, background export jobs and toast notifications.

use crate::app::{LimeShotApp, Pending};
use crate::theme::Theme;
use crate::widgets::Hint as _;
use egui::{Align2, ComboBox, Context, RichText, vec2};
use limeshot_core::{ChannelId, Command, PatternId};
use limeshot_engine::{BitDepth, ExportRange, RenderReport, RenderSettings};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum RenameTarget {
    Automation(limeshot_core::AutomationId),
    Pattern(PatternId),
    Channel(ChannelId),
    PlaylistTrack(u32),
    MixerTrack(u16),
    Marker(limeshot_core::MarkerId),
    /// Name the Channel Rack filter group for these channels (FL Studio's Group selected…).
    ChannelGroup(Vec<ChannelId>),
    Project,
    /// Save preset as…: the name for this preset in the user's presets folder.
    Preset(Box<limeshot_project::preset::Preset>),
}

pub struct ExportOptions {
    pub range: ExportRange,
    pub sample_rate: u32,
    pub depth: BitDepth,
}

pub enum Dialog {
    ConfirmDiscard(Pending),
    /// FL Studio asks before deleting channels; `remember` skips the question from then on.
    ConfirmDeleteChannels {
        channels: Vec<ChannelId>,
        remember: bool,
    },
    Export(ExportOptions),
    Rename {
        target: RenameTarget,
        text: String,
    },
    /// A previous session left unsaved work behind.
    Recover(crate::autosave::SessionInfo),
    /// FL Studio's Patterns › Transpose…: move every note of a pattern by semitones; with
    /// `channels`, only theirs (the Channel Rack's Transpose selected…).
    Transpose {
        pattern: PatternId,
        channels: Vec<ChannelId>,
        semitones: i32,
    },
    /// Type a control value in its displayed units, committed once on OK.
    ControlValue {
        target: limeshot_core::AutomationTarget,
        text: String,
    },
    Shortcuts,
}

impl Dialog {
    pub fn export(app: &LimeShotApp) -> Self {
        // FL Studio renders the Playlist's time selection when there is one.
        let range = match app.ui.playlist.time_sel {
            Some((from, to)) if to > from && !app.project().playlist.clips.is_empty() => {
                ExportRange::Selection { from, to }
            }
            _ if app.project().playlist.clips.is_empty() => ExportRange::Pattern,
            _ => ExportRange::Song,
        };
        Dialog::Export(ExportOptions {
            range,
            sample_rate: 44_100,
            depth: BitDepth::Int24,
        })
    }

    pub fn rename_pattern(app: &LimeShotApp, id: PatternId) -> Self {
        let text = app.project().pattern(id).map(|p| p.name.clone()).unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::Pattern(id),
            text,
        }
    }

    pub fn rename_automation(app: &LimeShotApp, id: limeshot_core::AutomationId) -> Self {
        let text = app.project().automation(id).map(|a| a.name.clone()).unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::Automation(id),
            text,
        }
    }

    pub fn rename_marker(app: &LimeShotApp, id: limeshot_core::MarkerId) -> Self {
        let text = app
            .project()
            .playlist
            .markers
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.name.clone())
            .unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::Marker(id),
            text,
        }
    }

    pub fn group_channels(app: &LimeShotApp, channels: Vec<ChannelId>) -> Self {
        let text = channels
            .first()
            .and_then(|c| app.project().channel(*c))
            .and_then(|c| c.group.clone())
            .unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::ChannelGroup(channels),
            text,
        }
    }

    pub fn rename_channel(app: &LimeShotApp, id: ChannelId) -> Self {
        let text = app.project().channel(id).map(|c| c.name.clone()).unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::Channel(id),
            text,
        }
    }

    pub fn rename_track(app: &LimeShotApp, i: u32) -> Self {
        let text = app
            .project()
            .playlist
            .tracks
            .get(i as usize)
            .map(|t| t.name.clone())
            .unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::PlaylistTrack(i),
            text,
        }
    }

    pub fn rename_mixer(app: &LimeShotApp, i: u16) -> Self {
        let text = app
            .project()
            .mixer
            .tracks
            .get(usize::from(i))
            .map(|t| t.name.clone())
            .unwrap_or_default();
        Dialog::Rename {
            target: RenameTarget::MixerTrack(i),
            text,
        }
    }
}

pub struct ExportJob {
    pub path: PathBuf,
    pub started: Instant,
    rx: Receiver<Result<RenderReport, String>>,
    /// The control halves of the export's own plugin instances, dropped on this thread when the
    /// job ends.
    _plugins: Vec<limeshot_plugins::PluginInstance>,
}

pub fn start_export(app: &mut LimeShotApp, opts: &ExportOptions, path: PathBuf) {
    app.sync_engine();
    let snap = app.snapshot().clone();
    let settings = RenderSettings {
        sample_rate: opts.sample_rate,
        range: opts.range,
        ..RenderSettings::default()
    };
    let depth = opts.depth;
    let (processors, plugins) = app.plugins.offline(app.project(), opts.sample_rate);
    let (tx, rx) = channel();
    let p = path.clone();
    let spawned = std::thread::Builder::new()
        .name("limeshot-export".into())
        .spawn(move || {
            let r = limeshot_engine::export_wav_with(snap, &settings, depth, &p, processors).map_err(|e| e.to_string());
            let _ = tx.send(r);
        });
    match spawned {
        Ok(_) => {
            app.export_job = Some(ExportJob {
                path,
                started: Instant::now(),
                rx,
                _plugins: plugins,
            })
        }
        Err(e) => app.toast_error(format!("could not start the export: {e}")),
    }
}

pub fn poll_export(app: &mut LimeShotApp) {
    let Some(job) = &app.export_job else { return };
    match job.rx.try_recv() {
        Ok(Ok(rep)) => {
            let path = job.path.clone();
            app.export_job = None;
            let clip = if rep.peak > 1.0 {
                format!(
                    " — peak {:+.1} dBFS: clipped, lower the master",
                    limeshot_dsp::gain_to_db(rep.peak)
                )
            } else {
                String::new()
            };
            app.toast(format!("Exported {} ({:.1} s){clip}", path.display(), rep.seconds()));
        }
        Ok(Err(e)) => {
            app.export_job = None;
            app.toast_error(format!("Export failed: {e}"));
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            app.export_job = None;
            app.toast_error("Export failed unexpectedly");
        }
    }
}

/// Show `add` in a centered, fixed-width dialog window under the shared dialog id.
fn modal(ctx: &Context, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Window::new(RichText::new(title).strong())
        .id(egui::Id::new("dialog"))
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            add(ui);
        });
}

/// Draw export progress and the active dialog, applying actions chosen by the user.
/// Keep unfinished dialogs open; Escape dismisses all except the save-changes prompt.
pub fn show(app: &mut LimeShotApp, ctx: &Context) {
    if let Some(job) = &app.export_job {
        let secs = job.started.elapsed().as_secs_f32();
        let path = job.path.display().to_string();
        modal(ctx, "Exporting…", |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!("Rendering to {path} ({secs:.0} s)"));
            });
        });
        ctx.request_repaint();
    }
    let Some(mut dialog) = app.dialog.take() else { return };
    let mut keep = true;
    let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    match &mut dialog {
        Dialog::ConfirmDiscard(action) => {
            let what = match action {
                Pending::New => "starting a new project",
                Pending::Open(_) => "opening another project",
                Pending::Quit => "quitting",
            };
            let action = action.clone();
            modal(ctx, "Save changes?", |ui| {
                ui.label(format!("The project has unsaved changes. Save them before {what}?"));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save").strong()).clicked() {
                        keep = false;
                        if app.save() {
                            app.perform(action.clone());
                        }
                    }
                    if ui.button("Don't save").clicked() {
                        keep = false;
                        app.perform(action.clone());
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
        }
        Dialog::ConfirmDeleteChannels { channels, remember } => {
            let text = if channels.len() == 1 {
                "The selected channel is about to be deleted."
            } else {
                "The selected channels are about to be deleted."
            };
            let channels = channels.clone();
            modal(ctx, "Confirm", |ui| {
                ui.label(text);
                ui.add_space(4.0);
                ui.checkbox(remember, "Remember my choice");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("OK").strong()).clicked() {
                        keep = false;
                        if *remember {
                            app.settings.confirm_channel_delete = false;
                            app.save_settings();
                        }
                        crate::channel_rack::delete_channels(app, &channels);
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
        }
        Dialog::Recover(info) => {
            let info = info.clone();
            modal(ctx, "Recover unsaved work?", |ui| {
                let from = info.original.as_ref().map_or_else(
                    || "a project that was never saved".to_string(),
                    |p| p.display().to_string(),
                );
                ui.label(format!(
                    "LimeShot did not close normally last time. An autosaved copy of \u{201c}{}\u{201d} ({from}) from {} can be recovered.",
                    info.title,
                    info.age()
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Recover").strong()).clicked() {
                        keep = false;
                        app.recover(&info);
                    }
                    if ui.button("Discard").clicked() {
                        keep = false;
                        app.autosave.clear();
                    }
                });
            });
        }
        Dialog::Export(opts) => {
            modal(ctx, "Export WAV", |ui| {
                egui::Grid::new("export_grid")
                    .min_col_width(82.0)
                    .num_columns(2)
                    .spacing(vec2(12.0, 6.0))
                    .show(ui, |ui| {
                        ui.label("Range");
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut opts.range, ExportRange::Song, "Song");
                            ui.selectable_value(&mut opts.range, ExportRange::Pattern, "Pattern");
                            if let Some((from, to)) = app.ui.playlist.time_sel.filter(|(a, b)| b > a) {
                                let sel = ExportRange::Selection { from, to };
                                ui.selectable_value(&mut opts.range, sel, "Time selection")
                                    .hint("The Playlist's time selection");
                            }
                        });
                        ui.end_row();
                        ui.label("Sample rate");
                        ComboBox::from_id_salt("export_sr")
                            .selected_text(format!("{} Hz", opts.sample_rate))
                            .show_ui(ui, |ui| {
                                for sr in [44_100u32, 48_000, 88_200, 96_000] {
                                    ui.selectable_value(&mut opts.sample_rate, sr, format!("{sr} Hz"));
                                }
                            });
                        ui.end_row();
                        ui.label("Format");
                        ComboBox::from_id_salt("export_depth")
                            .selected_text(opts.depth.label())
                            .show_ui(ui, |ui| {
                                for d in BitDepth::ALL {
                                    ui.selectable_value(&mut opts.depth, d, d.label());
                                }
                            });
                        ui.end_row();
                    });
                if matches!(opts.range, ExportRange::Song | ExportRange::Selection { .. })
                    && app.project().playlist.clips.is_empty()
                {
                    ui.colored_label(
                        Theme::current().warning,
                        "The Playlist is empty: place a pattern first, or export the pattern.",
                    );
                }
                ui.label(
                    RichText::new("Stereo WAV. Notes ring out for up to 4 s after the end. No dithering.")
                        .small()
                        .color(Theme::current().text_dim),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Export…").strong()).clicked() {
                        let name = format!(
                            "{}.wav",
                            app.path
                                .as_ref()
                                .and_then(|p| p.file_stem())
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "LimeShot export".into())
                        );
                        if let Some(mut path) = rfd::FileDialog::new()
                            .add_filter("WAV audio", &["wav"])
                            .set_file_name(name)
                            .save_file()
                        {
                            if path.extension().is_none() {
                                path.set_extension("wav");
                            }
                            start_export(app, opts, path);
                            keep = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
        }
        Dialog::ControlValue { target, text } => {
            let target = *target;
            modal(ctx, "Type in value", |ui| {
                ui.label(target.label(app.project()));
                if let Some((_, unit)) = crate::automation_ui::input_value(app, target) {
                    ui.horizontal(|ui| {
                        let label = ui.label("Value");
                        let r = ui
                            .add(egui::TextEdit::singleline(text).desired_width(180.0))
                            .labelled_by(label.id);
                        ui.label(unit);
                        if !r.has_focus() && !ui.memory(|m| m.had_focus_last_frame(r.id)) {
                            r.request_focus();
                        }
                    });
                    let value = crate::automation_ui::parse_value(app, target, text);
                    if value.is_none() {
                        ui.colored_label(Theme::current().danger, "Enter a value within the control's range.");
                    }
                    if matches!(target, limeshot_core::AutomationTarget::MixerVolume(_)) {
                        ui.label("Use -inf for silence.");
                    }
                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.horizontal(|ui| {
                        if (ui.add_enabled(value.is_some(), egui::Button::new("OK")).clicked() || enter)
                            && let Some(value) = value
                        {
                            crate::automation_ui::set_value(app, target, value);
                            keep = false;
                        }
                        if ui.button("Cancel").clicked() {
                            keep = false;
                        }
                    });
                } else {
                    ui.label("This control is no longer available.");
                    if ui.button("Close").clicked() {
                        keep = false;
                    }
                }
            });
        }
        Dialog::Rename { target, text } => {
            let target = target.clone();
            let title = match target {
                RenameTarget::ChannelGroup(_) => "Filter group name",
                RenameTarget::Preset(_) => "Save preset as",
                _ => "Rename",
            };
            modal(ctx, title, |ui| {
                let r = ui.add(egui::TextEdit::singleline(text).desired_width(310.0));
                r.request_focus();
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() || enter {
                        let name = text.trim().to_string();
                        if let RenameTarget::ChannelGroup(channels) = &target {
                            // An empty name moves the channels back to Unsorted.
                            let group = (!name.is_empty()).then(|| name.clone());
                            app.exec(Command::SetChannelGroup {
                                channels: channels.clone(),
                                group: group.clone(),
                            });
                            app.ui.rack.filter = match group {
                                Some(g) => crate::channel_rack::RackFilter::Group(g),
                                None => crate::channel_rack::RackFilter::Unsorted,
                            };
                        } else if let RenameTarget::Preset(preset) = &target {
                            crate::presets_ui::save(app, preset, &name);
                        } else if !name.is_empty() {
                            app.exec(match target {
                                RenameTarget::Pattern(pattern) => Command::RenamePattern { pattern, name },
                                RenameTarget::Automation(automation) => Command::RenameAutomation { automation, name },
                                RenameTarget::Channel(channel) => Command::RenameChannel { channel, name },
                                RenameTarget::PlaylistTrack(track) => Command::RenamePlaylistTrack { track, name },
                                RenameTarget::MixerTrack(track) => Command::RenameMixerTrack { track, name },
                                RenameTarget::Marker(marker) => Command::RenameMarker { marker, name },
                                RenameTarget::Project => Command::SetTitle(name),
                                RenameTarget::ChannelGroup(_) | RenameTarget::Preset(_) => return,
                            });
                        }
                        keep = false;
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
        }
        Dialog::Transpose {
            pattern,
            channels,
            semitones,
        } => {
            let pattern = *pattern;
            let channels = channels.clone();
            modal(ctx, "Transpose", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Semitones");
                    let r = ui.add(egui::DragValue::new(semitones).range(-127..=127).speed(0.2));
                    r.request_focus();
                });
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() || enter {
                        let notes: Vec<_> = app
                            .project()
                            .pattern(pattern)
                            .map(|p| {
                                p.notes
                                    .iter()
                                    .filter(|n| channels.is_empty() || channels.contains(&n.channel))
                                    .map(|n| n.id)
                                    .collect()
                            })
                            .unwrap_or_default();
                        if *semitones != 0 && !notes.is_empty() {
                            app.exec(Command::MoveNotes {
                                pattern,
                                notes,
                                dt: 0,
                                dkey: *semitones,
                            });
                        }
                        keep = false;
                    }
                    if ui.button("Cancel").clicked() {
                        keep = false;
                    }
                });
            });
        }
        Dialog::Shortcuts => {
            modal(ctx, "Keyboard shortcuts", |ui| {
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    egui::Grid::new("keys").striped(true).show(ui, |ui| {
                        for (k, a) in crate::shortcuts::GLOBAL.iter().chain(crate::shortcuts::EDITING) {
                            ui.label(RichText::new(crate::shortcuts::label(k)).monospace());
                            ui.label(*a);
                            ui.end_row();
                        }
                    });
                });
                if ui.button("Close").clicked() {
                    keep = false;
                }
            });
        }
    }
    if esc && !matches!(dialog, Dialog::ConfirmDiscard(_)) {
        keep = false;
    }
    if keep && app.dialog.is_none() {
        app.dialog = Some(dialog);
    }
}

pub fn toasts(app: &mut LimeShotApp, ctx: &Context) {
    if app.toasts.is_empty() {
        return;
    }
    let t = Theme::current();
    egui::Area::new(egui::Id::new("toasts"))
        .anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -32.0))
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            for toast in &app.toasts {
                egui::Frame::popup(ui.style())
                    .fill(if toast.error {
                        crate::theme::mix(t.surface, t.danger, 0.25)
                    } else {
                        t.surface
                    })
                    .show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.label(&toast.text);
                    });
            }
        });
}
