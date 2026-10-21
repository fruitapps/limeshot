//! The Browser: project contents and sample folders.
//!
//! Click a WAV to preview it; double-click (or use its menu) to load it into the selected sampler
//! channel; drag it onto the Channel Rack to create a channel or onto a channel to replace its
//! sample.

use crate::app::LimeShotApp;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, semibold};
use crate::widgets::{icon_button, panel_header};
use egui::{CollapsingHeader, CornerRadius, Rect, RichText, Sense, TextEdit, Ui, vec2};
use limeshot_audio::Msg;
use limeshot_audio::assets::SampleStatus;
use limeshot_core::{ChannelId, Command, Instrument, SampleRef, SamplerParams};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A sample being dragged from the Browser.
#[derive(Debug, Clone)]
pub struct DraggedSample(pub PathBuf);

#[derive(Default)]
pub struct State {
    pub filter: String,
    listings: HashMap<PathBuf, Vec<(PathBuf, bool)>>,
    pending_preview: Option<PathBuf>,
    pub selected: Option<PathBuf>,
    /// Screen rectangles of the files listed in the last frame (for tests and tools).
    pub file_rects: Vec<(PathBuf, Rect)>,
}

impl State {
    fn list(&mut self, dir: &Path) -> &[(PathBuf, bool)] {
        self.listings.entry(dir.to_path_buf()).or_insert_with(|| {
            let mut v: Vec<(PathBuf, bool)> = std::fs::read_dir(dir)
                .map(|rd| {
                    rd.filter_map(Result::ok)
                        .map(|e| {
                            let p = e.path();
                            let is_dir = p.is_dir();
                            (p, is_dir)
                        })
                        .filter(|(p, d)| {
                            let hidden = p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'));
                            !hidden && (*d || is_wav(p))
                        })
                        .collect()
                })
                .unwrap_or_default();
            v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.file_name().cmp(&b.0.file_name())));
            v
        })
    }

    /// Centre of `path`'s row, as last drawn.
    pub fn file_pos(&self, path: &Path) -> Option<egui::Pos2> {
        self.file_rects.iter().find(|(p, _)| p == path).map(|(_, r)| r.center())
    }

    /// Whether a preview is waiting for its sample to load.
    pub fn preview_pending(&self) -> bool {
        self.pending_preview.is_some()
    }

    /// Forget a preview that is waiting for its sample to load.
    pub fn cancel_preview(&mut self) {
        self.pending_preview = None;
    }

    pub fn refresh(&mut self) {
        self.listings.clear();
    }
}

pub fn is_wav(p: &Path) -> bool {
    p.extension().is_some_and(|e| e.eq_ignore_ascii_case("wav"))
}

pub fn file_label(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

/// Load `path` into `channel` (making it a sampler) and name the channel after the file.
pub fn assign_sample(app: &mut LimeShotApp, channel: ChannelId, path: &Path) {
    let Some(ch) = app.project().channel(channel) else {
        return;
    };
    let mut params = match &ch.instrument {
        Instrument::Sampler(p) => p.clone(),
        _ => SamplerParams::default(),
    };
    params.sample = Some(SampleRef {
        path: path.to_string_lossy().into_owned(),
    });
    app.samples.request_path(path.to_path_buf());
    app.exec(Command::Batch(vec![
        Command::SetInstrument {
            channel,
            instrument: Instrument::Sampler(params),
        },
        Command::RenameChannel {
            channel,
            name: file_label(path),
        },
    ]));
}

/// A new sampler channel playing `path`.
pub fn new_sampler_channel(app: &mut LimeShotApp, path: &Path) -> Option<ChannelId> {
    app.samples.request_path(path.to_path_buf());
    let params = SamplerParams {
        sample: Some(SampleRef {
            path: path.to_string_lossy().into_owned(),
        }),
        ..SamplerParams::default()
    };
    let id = app.exec(Command::AddChannel {
        name: file_label(path),
        instrument: Instrument::Sampler(params),
    })?;
    app.select_channel(ChannelId(id));
    Some(ChannelId(id))
}

pub fn preview(app: &mut LimeShotApp, path: &Path) {
    app.samples.request_path(path.to_path_buf());
    app.ui.browser.pending_preview = Some(path.to_path_buf());
}

pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    app.ui.browser.file_rects.clear();
    // Start a requested preview once its sample has loaded.
    if let Some(p) = app.ui.browser.pending_preview.clone() {
        match app.samples.status_of_path(&p) {
            Some(SampleStatus::Ready(s)) => {
                let s = s.clone();
                app.send(Msg::PreviewSample(s));
                app.ui.browser.pending_preview = None;
            }
            Some(SampleStatus::Failed(e)) => {
                let e = e.clone();
                app.toast_error(e);
                app.ui.browser.pending_preview = None;
            }
            _ => ui.ctx().request_repaint(),
        }
    }
    panel_header(ui, "Browser", |ui| {
        if ui.small_button("⟳").on_hover_text("Re-read folders").clicked() {
            app.ui.browser.refresh();
        }
        if icon_button(ui, Icon::Plus, false, 22.0, "Add a folder to the Browser").clicked()
            && let Some(dir) = rfd::FileDialog::new().pick_folder()
            && !app.settings.browser_folders.contains(&dir)
        {
            app.settings.browser_folders.push(dir);
            app.settings.save();
        }
    });
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.add(
                TextEdit::singleline(&mut app.ui.browser.filter)
                    .hint_text("Search samples")
                    .desired_width(f32::INFINITY),
            );
        });
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().indent = 12.0;
        CollapsingHeader::new(RichText::new("CHANNELS").font(semibold(10.5)).color(t.text_dim))
            .default_open(true)
            .show(ui, |ui| project_tree(app, ui));
        let mut roots: Vec<(String, PathBuf, bool)> = Vec::new();
        if let Some(b) = &app.bundled_samples {
            roots.push(("LIMESHOT SAMPLES".into(), b.clone(), false));
        }
        for f in &app.settings.browser_folders {
            roots.push((
                f.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| f.display().to_string()),
                f.clone(),
                true,
            ));
        }
        let mut remove = None;
        for (name, dir, user) in roots {
            let header = CollapsingHeader::new(RichText::new(name).font(semibold(10.5)).color(if user {
                t.accent
            } else {
                t.text_dim
            }))
            .id_salt(&dir)
            .default_open(!user)
            .show(ui, |ui| folder(app, ui, &dir, 0));
            if user {
                header.header_response.context_menu(|ui| {
                    if ui.button("Remove from Browser").clicked() {
                        remove = Some(dir.clone());
                        ui.close();
                    }
                });
            }
        }
        if let Some(d) = remove {
            app.settings.browser_folders.retain(|f| f != &d);
            app.settings.save();
        }
    });
}

/// The project's channels (patterns are listed in the Playlist's picker). Click to select and
/// show in the Inspector.
fn project_tree(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let sel = app.selected_channel();
    let chans: Vec<_> = app
        .project()
        .channels
        .iter()
        .map(|c| (c.id, c.name.clone(), c.color, c.instrument.kind_label()))
        .collect();
    if chans.is_empty() {
        ui.label(RichText::new("No channels yet").color(t.text_faint));
    }
    for (id, name, color, kind) in chans {
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::click());
        if Some(id) == sel {
            ui.painter()
                .rect_filled(r, CornerRadius::same(t.radius_sm), mix(t.surface, t.accent, 0.12));
        } else if resp.hovered() {
            ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), t.raised);
        }
        ui.painter().rect_filled(
            Rect::from_min_size(r.min + vec2(4.0, 6.0), vec2(10.0, 10.0)),
            CornerRadius::same(2),
            crate::theme::rgb(color),
        );
        ui.painter().text(
            r.left_center() + vec2(22.0, 0.0),
            egui::Align2::LEFT_CENTER,
            &name,
            crate::theme::prop(12.0),
            t.text,
        );
        ui.painter().text(
            r.right_center() - vec2(6.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            kind,
            crate::theme::prop(10.5),
            t.text_faint,
        );
        if resp.clicked() {
            app.select_channel(id);
        }
    }
}

fn folder(app: &mut LimeShotApp, ui: &mut Ui, dir: &Path, depth: usize) {
    if depth > 8 {
        return;
    }
    let t = Theme::current();
    let filter = app.ui.browser.filter.to_lowercase();
    let entries = app.ui.browser.list(dir).to_vec();
    if entries.is_empty() {
        ui.label(RichText::new("(no WAV files)").small().color(t.text_dim));
    }
    for (path, is_dir) in entries {
        if is_dir {
            CollapsingHeader::new(file_label(&path))
                .id_salt(&path)
                .show(ui, |ui| folder(app, ui, &path, depth + 1));
            continue;
        }
        let name = file_label(&path);
        if !filter.is_empty() && !name.to_lowercase().contains(&filter) {
            continue;
        }
        let selected = app.ui.browser.selected.as_deref() == Some(path.as_path());
        let id = egui::Id::new(("browser-file", &path));
        let resp = ui
            .dnd_drag_source(id, DraggedSample(path.clone()), |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                    icons::paint(
                        ui.painter(),
                        r,
                        Icon::Wave,
                        if selected { t.accent } else { t.text_dim },
                    );
                    ui.add(
                        egui::Label::new(RichText::new(&name).color(if selected { t.accent } else { t.text }))
                            .selectable(false),
                    );
                });
            })
            .response;
        let resp = ui.interact(resp.rect, id.with("click"), Sense::click());
        app.ui.browser.file_rects.push((path.clone(), resp.rect));
        if resp.clicked() {
            app.ui.browser.selected = Some(path.clone());
            preview(app, &path);
        }
        if resp.double_clicked() {
            load_into_selected(app, &path);
        }
        if resp.hovered() {
            app.hint(format!(
                "{} — click to preview, double-click to load into the selected channel, drag to the Channel Rack",
                path.display()
            ));
        }
        resp.context_menu(|ui| {
            if ui.button("Preview").clicked() {
                preview(app, &path);
                ui.close();
            }
            if ui.button("Load into selected channel").clicked() {
                load_into_selected(app, &path);
                ui.close();
            }
            if ui.button("New sampler channel").clicked() {
                new_sampler_channel(app, &path);
                ui.close();
            }
        });
    }
}

fn load_into_selected(app: &mut LimeShotApp, path: &Path) {
    match app.selected_channel() {
        Some(c)
            if matches!(
                app.project().channel(c).map(|c| &c.instrument),
                Some(Instrument::Sampler(_))
            ) =>
        {
            assign_sample(app, c, path)
        }
        _ => {
            new_sampler_channel(app, path);
        }
    }
}
