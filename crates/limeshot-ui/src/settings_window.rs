//! FL Studio's settings window (observed in session 14, `fl-studio-analysis.md` §19): one window
//! titled "Settings – <page title>" with a row of tabs in groups (MIDI, Audio, General, File,
//! Theme | Project, Info | About), each page split into titled sections. Changes apply at once;
//! there is no OK or Cancel. Options › Audio / General / File settings, Project general settings
//! and Project info open it on their page.
//!
//! LimeShot's pages hold the settings it has; MIDI and Theme are shown disabled until LimeShot
//! has settings for them.

use egui::{Align2, ComboBox, Context, RichText, Ui, vec2};
use limeshot_core::{Command, TimeSignature};

use crate::app::LimeShotApp;
use crate::theme::{Theme, semibold};
use crate::widgets::Hint as _;

/// A page (tab) of the settings window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Midi,
    Audio,
    General,
    File,
    Theme,
    Project,
    Info,
    About,
}

impl Page {
    /// The tab's label.
    pub fn label(self) -> &'static str {
        match self {
            Page::Midi => "MIDI",
            Page::Audio => "Audio",
            Page::General => "General",
            Page::File => "File",
            Page::Theme => "Theme",
            Page::Project => "Project",
            Page::Info => "Info",
            Page::About => "About",
        }
    }

    /// The window title's page name, as FL Studio words it.
    pub fn title(self) -> &'static str {
        match self {
            Page::Midi => "MIDI input / output devices",
            Page::Audio => "Audio devices / mixer",
            Page::General => "System / miscellaneous",
            Page::File => "Files & folders",
            Page::Theme => "Look and feel",
            Page::Project => "Current project",
            Page::Info => "Project credits",
            Page::About => "About",
        }
    }

    /// Whether LimeShot has settings on this page yet.
    fn available(self) -> bool {
        !matches!(self, Page::Midi | Page::Theme)
    }
}

/// The tab groups, as FL Studio spaces them.
const GROUPS: [&[Page]; 3] = [
    &[Page::Midi, Page::Audio, Page::General, Page::File, Page::Theme],
    &[Page::Project, Page::Info],
    &[Page::About],
];

/// The settings window's state while it is open.
pub struct State {
    pub page: Page,
    /// Output devices, read when the Audio page was opened.
    devices: Option<Vec<limeshot_audio::DeviceInfo>>,
}

impl LimeShotApp {
    /// Open the settings window on `page` (or switch to it).
    pub fn open_settings(&mut self, page: Page) {
        match &mut self.ui.settings {
            Some(s) => s.page = page,
            None => {
                self.ui.settings = Some(State { page, devices: None });
            }
        }
    }
}

/// Show the settings window if it is open.
pub fn show(app: &mut LimeShotApp, ctx: &Context) {
    let Some(page) = app.ui.settings.as_ref().map(|s| s.page) else {
        return;
    };
    let mut open = true;
    egui::Window::new(RichText::new(format!("Settings – {}", page.title())).strong())
        .id(egui::Id::new("limeshot-settings"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size(vec2(560.0, 620.0))
        .min_size(vec2(460.0, 360.0))
        .pivot(Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .show(ctx, |ui| {
            tabs(app, ui, page);
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match page {
                    Page::Audio => audio(app, ui),
                    Page::General => general(app, ui),
                    Page::File => file(app, ui),
                    Page::Project => project(app, ui),
                    Page::Info => info(app, ui),
                    Page::About => about(ui),
                    Page::Midi | Page::Theme => {
                        ui.label(RichText::new("Not implemented yet").color(Theme::current().text_faint));
                    }
                });
        });
    if !open {
        app.ui.settings = None;
    }
}

fn tabs(app: &mut LimeShotApp, ui: &mut Ui, current: Page) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        for (g, group) in GROUPS.iter().enumerate() {
            if g > 0 {
                ui.add_space(14.0);
            }
            for &p in *group {
                let on = p == current;
                let text = RichText::new(p.label())
                    .font(semibold(12.5))
                    .color(if on { t.accent } else { t.text_dim });
                let r = ui
                    .add_enabled(p.available(), egui::Button::new(text).selected(on).frame(on))
                    .hint(if p.available() {
                        p.title()
                    } else {
                        "Not implemented yet"
                    });
                if r.clicked() {
                    app.open_settings(p);
                }
            }
        }
    });
    ui.separator();
}

/// A titled section, as FL Studio's pages have.
fn section(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    let t = Theme::current();
    egui::Frame::new()
        .fill(t.raised)
        .stroke(egui::Stroke::new(1.0, t.border))
        .corner_radius(t.radius)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).font(semibold(12.0)).color(t.text_dim));
            ui.add_space(4.0);
            add(ui);
        });
    ui.add_space(8.0);
}

// ----- Audio -----

fn audio(app: &mut LimeShotApp, ui: &mut Ui) {
    let devices = match app.ui.settings.as_mut() {
        Some(s) => s
            .devices
            .get_or_insert_with(limeshot_audio::list_output_devices)
            .clone(),
        None => return,
    };
    let mut cfg = app.settings.audio.clone();
    section(ui, "Input / output", |ui| {
        egui::Grid::new("settings-audio")
            .num_columns(2)
            .spacing(vec2(12.0, 6.0))
            .show(ui, |ui| {
                ui.label("Output");
                let current = cfg.device.clone().unwrap_or_else(|| "System default".into());
                ComboBox::from_id_salt("settings-audio-device")
                    .width(260.0)
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.device, None, "System default");
                        for d in &devices {
                            let label = if d.is_default {
                                format!("{} (default)", d.name)
                            } else {
                                d.name.clone()
                            };
                            ui.selectable_value(&mut cfg.device, Some(d.name.clone()), label);
                        }
                    });
                ui.end_row();
                let dev = devices
                    .iter()
                    .find(|d| Some(&d.name) == cfg.device.as_ref())
                    .or_else(|| devices.iter().find(|d| d.is_default));
                ui.label("Sample rate (Hz)");
                ComboBox::from_id_salt("settings-audio-rate")
                    .selected_text(
                        cfg.sample_rate
                            .map_or_else(|| "Device default".into(), |s| s.to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.sample_rate, None, "Device default");
                        for sr in dev.map(|d| d.sample_rates.clone()).unwrap_or_default() {
                            ui.selectable_value(&mut cfg.sample_rate, Some(sr), sr.to_string());
                        }
                    });
                ui.end_row();
                ui.label("Buffer length");
                ComboBox::from_id_salt("settings-audio-buffer")
                    .selected_text(
                        cfg.buffer_size
                            .map_or_else(|| "Device default".into(), |b| format!("{b} samples")),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.buffer_size, None, "Device default");
                        for b in [64u32, 128, 256, 512, 1024, 2048] {
                            let ok = dev.is_none_or(|d| {
                                d.buffer_min.is_none_or(|lo| b >= lo) && d.buffer_max.is_none_or(|hi| b <= hi)
                            });
                            if ok {
                                ui.selectable_value(&mut cfg.buffer_size, Some(b), format!("{b} samples"));
                            }
                        }
                    });
                ui.end_row();
            });
        ui.add_space(4.0);
        let sr = app.player.sample_rate();
        let pdc = app.player.status().latency();
        let ms = |samples: f64| samples * 1000.0 / f64::from(sr.max(1));
        let status = match app.player.output_description() {
            Ok(n) => format!("Open: {n}, {sr} Hz"),
            Err(e) => format!("No output ({e})"),
        };
        let buffer = cfg
            .buffer_size
            .map(|b| format!(", buffer {b} samples ({:.1} ms)", ms(f64::from(b))));
        ui.label(RichText::new(format!("Status  {status}{}", buffer.unwrap_or_default())).small());
        // FL Studio's status line also gives the plugin latency.
        ui.label(
            RichText::new(format!(
                "Plugin delay compensation: {pdc} samples ({:.1} ms)",
                ms(f64::from(pdc))
            ))
            .small()
            .color(Theme::current().text_dim),
        );
    });
    // Settings apply at once, as in FL Studio.
    if cfg != app.settings.audio {
        app.settings.audio = cfg;
        app.restart_audio();
        app.save_settings();
    }
}

// ----- General -----

fn general(app: &mut LimeShotApp, ui: &mut Ui) {
    section(ui, "Display", |ui| {
        ui.horizontal(|ui| {
            let cur = app.settings.ui_scale;
            ComboBox::from_id_salt("settings-ui-scale")
                .selected_text(format!("{:.0}%", cur * 100.0))
                .show_ui(ui, |ui| {
                    for s in [0.75f32, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0] {
                        if ui
                            .selectable_label((cur - s).abs() < 0.001, format!("{:.0}%", s * 100.0))
                            .clicked()
                        {
                            app.settings.ui_scale = s;
                            app.save_settings();
                        }
                    }
                });
            ui.label("UI scaling");
        });
        ui.label(
            RichText::new("Also Cmd+= / Cmd+- / Cmd+0")
                .small()
                .color(Theme::current().text_faint),
        );
    });
    section(ui, "Hints and warnings", |ui| {
        let mut ask = app.settings.confirm_channel_delete;
        if ui.checkbox(&mut ask, "Ask before deleting channels").changed() {
            app.settings.confirm_channel_delete = ask;
            app.save_settings();
        }
    });
}

// ----- File -----

fn file(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    section(ui, "Backup", |ui| {
        ui.horizontal(|ui| {
            let cur = app.settings.autosave_minutes;
            let label = |m: u32| match m {
                0 => "Off".to_string(),
                1 => "Every minute".to_string(),
                m => format!("Every {m} minutes"),
            };
            ComboBox::from_id_salt("settings-autosave")
                .selected_text(label(cur))
                .show_ui(ui, |ui| {
                    for m in [0, 1, 2, 5, 10] {
                        if ui.selectable_label(cur == m, label(m)).clicked() {
                            app.settings.autosave_minutes = m;
                            app.save_settings();
                        }
                    }
                });
            ui.label("Autosave unsaved changes");
        });
    });
    section(ui, "Browser extra search folders", |ui| {
        let folders = app.settings.browser_folders.clone();
        if folders.is_empty() {
            ui.label(RichText::new("None").color(t.text_faint));
        }
        for f in folders {
            ui.horizontal(|ui| {
                if ui
                    .small_button("×")
                    .hint("Remove this folder from the Browser")
                    .clicked()
                {
                    app.settings.browser_folders.retain(|x| x != &f);
                    app.ui.browser.refresh();
                    app.save_settings();
                }
                ui.label(f.display().to_string());
            });
        }
        if ui.button("Add folder…").clicked()
            && let Some(dir) = rfd::FileDialog::new().pick_folder()
            && !app.settings.browser_folders.contains(&dir)
        {
            app.settings.browser_folders.push(dir);
            app.ui.browser.refresh();
            app.save_settings();
        }
    });
    section(ui, "Recent files", |ui| {
        let recent = app.settings.recent.clone();
        if recent.is_empty() {
            ui.label(RichText::new("None").color(t.text_faint));
        }
        for p in &recent {
            ui.label(RichText::new(p.display().to_string()).small());
        }
        if ui
            .add_enabled(!recent.is_empty(), egui::Button::new("Clear list"))
            .clicked()
        {
            app.settings.recent.clear();
            app.save_settings();
        }
    });
}

// ----- Project -----

fn project(app: &mut LimeShotApp, ui: &mut Ui) {
    section(ui, "Time settings", |ui| {
        let sig = app.project().time_signature;
        let (mut num, mut den) = (sig.numerator, sig.denominator);
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut num).range(1..=16).speed(0.1));
            ui.label("Numerator");
        });
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("settings-denominator")
                .width(50.0)
                .selected_text(den.to_string())
                .show_ui(ui, |ui| {
                    for d in [2u8, 4, 8, 16] {
                        ui.selectable_value(&mut den, d, d.to_string());
                    }
                });
            ui.label("Denominator");
        });
        if (num, den) != (sig.numerator, sig.denominator) {
            app.exec(Command::SetTimeSignature(TimeSignature::new(num, den)));
        }
    });
}

// ----- Info -----

/// A text field that edits the project info; typing in it is one undo step.
fn info_field(app: &mut LimeShotApp, r: egui::Response, title: String, info: limeshot_core::ProjectInfo) {
    if r.gained_focus() {
        app.doc.begin_gesture();
    }
    if r.changed() {
        app.exec(Command::SetProjectInfo { title, info });
    }
    if r.lost_focus() {
        app.doc.end_gesture();
    }
}

fn info(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let label = |ui: &mut Ui, s: &str| {
        ui.label(RichText::new(s).font(semibold(12.0)).color(t.text_dim));
    };
    let mut title = app.project().title.clone();
    let mut info = app.project().info.clone();
    let width = ui.available_width();
    label(ui, "Title");
    let r = ui.add(egui::TextEdit::singleline(&mut title).desired_width(width));
    info_field(app, r, title.clone(), info.clone());
    label(ui, "Author");
    let r = ui.add(egui::TextEdit::singleline(&mut info.author).desired_width(width));
    info_field(app, r, title.clone(), info.clone());
    label(ui, "Genre");
    let r = ui.add(egui::TextEdit::singleline(&mut info.genre).desired_width(width));
    info_field(app, r, title.clone(), info.clone());
    label(ui, "Comments");
    let r = ui.add(
        egui::TextEdit::multiline(&mut info.comments)
            .desired_width(width)
            .desired_rows(10),
    );
    info_field(app, r, title.clone(), info.clone());
    label(ui, "Web link");
    let r = ui.add(egui::TextEdit::singleline(&mut info.url).desired_width(width));
    info_field(app, r, title.clone(), info.clone());
    ui.add_space(8.0);
    // FL Studio's project statistics.
    let p = app.project();
    let plugins = p
        .channels
        .iter()
        .filter(|c| matches!(c.instrument, limeshot_core::Instrument::Plugin(_)))
        .count()
        + p.mixer
            .tracks
            .iter()
            .flat_map(|tr| tr.effects.iter())
            .filter(|e| e.plugin.is_some())
            .count();
    let notes: usize = p.patterns.iter().map(|pat| pat.notes.len()).sum();
    let stats = [
        format!(
            "Plugins: {plugins}, Channels: {}, Mixer tracks: {}",
            p.channels.len(),
            p.mixer.tracks.len().saturating_sub(1)
        ),
        format!(
            "Patterns: {}, Playlist tracks: {}, Playlist clips: {}, Notes: {notes}",
            p.patterns.len(),
            p.playlist.tracks.len(),
            p.playlist.clips.len()
        ),
    ];
    for s in stats {
        ui.label(RichText::new(s).small().color(t.text_dim));
    }
}

// ----- About -----

fn about(ui: &mut Ui) {
    let t = Theme::current();
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        let (r, _) = ui.allocate_exact_size(vec2(80.0, 80.0), egui::Sense::hover());
        crate::icons::logo(ui.painter(), r.center(), 36.0);
        ui.heading(format!("LimeShot {}", env!("CARGO_PKG_VERSION")));
        ui.label("An open-source, offline-first music workstation written in Rust.");
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Pattern workflow inspired by FL Studio; independently implemented. FL Studio is a trademark of \
                 Image-Line, which is not affiliated with LimeShot.",
            )
            .small()
            .color(t.text_dim),
        );
        ui.label(RichText::new("MIT OR Apache-2.0").small());
    });
}
