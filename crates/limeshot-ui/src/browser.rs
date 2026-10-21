//! The Browser: project contents and sample folders.
//!
//! Click a WAV to preview it; double-click (or use its menu) to load it into the selected sampler
//! channel; drag it onto the Channel Rack to create a channel or onto a channel to replace its
//! sample. As in FL Studio, the panel at the bottom shows the selected sample's waveform with the
//! preview's position; click it (or its play button) to hear the sample again.

use crate::app::LimeShotApp;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, semibold};
use crate::widgets::{icon_button, panel_header};
use egui::{CollapsingHeader, CornerRadius, Rect, RichText, Sense, Stroke, TextEdit, Ui, pos2, vec2};
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
    /// The preview playing: its file, when it started and how long it lasts (seconds).
    pub preview_started: Option<(PathBuf, std::time::Instant, f64)>,
    /// The preview panel's waveform: min/max per column of the selected sample.
    preview_peaks: Option<(PathBuf, Vec<(f32, f32)>)>,
    /// The preview panel's waveform rectangle in the last frame (for tests).
    pub preview_rect: Option<Rect>,
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

/// Play the sample file at `path` (once it has loaded), as the Browser's preview.
pub fn preview(app: &mut LimeShotApp, path: &Path) {
    app.samples.request_path(path.to_path_buf());
    app.ui.browser.pending_preview = Some(path.to_path_buf());
}

/// Start a requested preview once its sample has loaded (every frame, so previews from the
/// Playlist's picker play with the Browser hidden too).
pub fn poll_preview(app: &mut LimeShotApp, ctx: &egui::Context) {
    if let Some(p) = app.ui.browser.pending_preview.clone() {
        match app.samples.status_of_path(&p) {
            Some(SampleStatus::Ready(s)) => {
                let s = s.clone();
                app.ui.browser.preview_started = Some((p, std::time::Instant::now(), s.duration_secs()));
                app.send(Msg::PreviewSample(s));
                app.ui.browser.pending_preview = None;
            }
            Some(SampleStatus::Failed(e)) => {
                let e = e.clone();
                app.toast_error(e);
                app.ui.browser.pending_preview = None;
            }
            _ => ctx.request_repaint(),
        }
    }
}

/// FL Studio's preview panel: the selected sample's waveform, the preview's position, its rate
/// and length. A click on it (or the play button) plays the sample again.
fn preview_panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    app.ui.browser.preview_rect = None;
    let Some(path) = app.ui.browser.selected.clone() else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Click a sample to hear it").small().color(t.text_faint));
        });
        return;
    };
    let sample = match app.samples.status_of_path(&path) {
        Some(SampleStatus::Ready(s)) => Some(s.clone()),
        _ => None,
    };
    ui.horizontal(|ui| {
        if icon_button(ui, Icon::Play, false, 20.0, "Play the sample").clicked() {
            preview(app, &path);
        }
        ui.label(RichText::new(file_label(&path)).font(semibold(11.0)).color(t.text));
        if let Some(s) = &sample {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{:.1} kHz · {:.2} s",
                        f64::from(s.sample_rate) / 1000.0,
                        s.duration_secs()
                    ))
                    .font(crate::theme::mono(9.5))
                    .color(t.text_dim),
                );
            });
        }
    });
    let (r, resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), ui.available_height().max(24.0)),
        Sense::click(),
    );
    app.ui.browser.preview_rect = Some(r);
    let painter = ui.painter_at(r);
    painter.rect_filled(r, CornerRadius::same(t.radius_sm), t.sunken);
    let Some(s) = sample else {
        painter.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            "Loading…",
            crate::theme::mono(10.0),
            t.text_dim,
        );
        return;
    };
    let cols = r.width().max(1.0) as usize;
    if app
        .ui
        .browser
        .preview_peaks
        .as_ref()
        .is_none_or(|(p, v)| *p != path || v.len() != cols)
    {
        let frames = s.frames().max(1);
        let peaks = (0..cols)
            .map(|x| {
                let (a, b) = (x * frames / cols, ((x + 1) * frames / cols).max(x * frames / cols + 1));
                (a..b.min(frames))
                    .step_by(((b - a) / 64).max(1))
                    .map(|i| {
                        let (l, r) = s.frame(i);
                        0.5 * (l + r)
                    })
                    .fold((0.0f32, 0.0f32), |(lo, hi), v| (lo.min(v), hi.max(v)))
            })
            .collect();
        app.ui.browser.preview_peaks = Some((path.clone(), peaks));
    }
    let mid = r.center().y;
    let half = r.height() * 0.5 - 2.0;
    if let Some((_, peaks)) = &app.ui.browser.preview_peaks {
        for (x, (lo, hi)) in peaks.iter().enumerate() {
            let xx = r.left() + x as f32 + 0.5;
            painter.line_segment(
                [
                    pos2(xx, mid - hi.clamp(-1.0, 1.0) * half),
                    pos2(xx, mid - lo.clamp(-1.0, 1.0) * half + 0.5),
                ],
                Stroke::new(1.0, t.accent_dim),
            );
        }
    }
    // The preview's position while it plays.
    if let Some((p, since, secs)) = &app.ui.browser.preview_started
        && *p == path
    {
        let k = since.elapsed().as_secs_f64() / secs.max(1e-3);
        if k < 1.0 {
            let x = r.left() + k as f32 * r.width();
            painter.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.5, t.accent));
            ui.ctx().request_repaint();
        }
    }
    if resp.clicked() {
        preview(app, &path);
    }
    if resp.hovered() {
        app.hint(format!("{} — click to hear it", path.display()));
    }
}

pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    app.ui.browser.file_rects.clear();
    egui::Panel::bottom("browser_preview")
        .exact_size(84.0)
        .frame(
            egui::Frame::new()
                .fill(t.surface)
                .inner_margin(egui::Margin::symmetric(8, 6))
                .stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| preview_panel(app, ui));
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
