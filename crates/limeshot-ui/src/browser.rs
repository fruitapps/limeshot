//! The Browser: project contents and sample folders.
//!
//! Click or double-click a sample to preview it; use its explicit menu to replace a sampler
//! channel; drag it onto the Channel Rack to create a channel or onto a channel to replace its
//! sample. As in FL Studio, the panel at the bottom shows the selected sample's waveform with the
//! preview's position; click it (or its play button) to hear the sample again.

use crate::app::LimeShotApp;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, semibold};
use crate::widgets::Hint as _;
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
    pending_imports: Vec<(PathBuf, ImportTarget)>,
    /// Put the keyboard in the search field on the next frame (Tools › Browser find…).
    pub focus_search: bool,
    listings: HashMap<PathBuf, Vec<(PathBuf, bool)>>,
    drag_peaks: HashMap<PathBuf, std::sync::Arc<crate::playlist::Peaks>>,
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
                            !hidden && (*d || is_sample(p))
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

/// A sample file the Browser lists and LimeShot can load (WAV, MP3, FLAC, Ogg Vorbis, AIFF).
pub fn is_sample(p: &Path) -> bool {
    limeshot_dsp::is_audio_file(p)
}

pub fn file_label(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

/// A requested edit waits for the background decoder; no broken channel is created on failure.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImportTarget {
    Channel(ChannelId),
    NewChannel,
    Picker,
    Playlist { track: u32, start: u64 },
}

pub(crate) fn import_audio(
    app: &mut LimeShotApp,
    path: &Path,
    target: ImportTarget,
) -> Option<std::sync::Arc<limeshot_dsp::SampleData>> {
    app.samples.request_path(path.to_path_buf());
    match app.samples.status_of_path(path).cloned() {
        Some(SampleStatus::Ready(sample)) => Some(sample),
        Some(SampleStatus::Failed(error)) => {
            app.toast_error(error);
            app.samples.remove_failed(path);
            None
        }
        _ => {
            let pending = (path.to_path_buf(), target);
            if !app.ui.browser.pending_imports.contains(&pending) {
                app.ui.browser.pending_imports.push(pending);
            }
            None
        }
    }
}

pub(crate) fn poll_imports(app: &mut LimeShotApp) {
    let pending = std::mem::take(&mut app.ui.browser.pending_imports);
    for (path, target) in pending {
        let Some(status) = app.samples.status_of_path(&path) else {
            // Another target for this path already reported and removed the failed load.
            continue;
        };
        if matches!(status, SampleStatus::Loading) {
            app.ui.browser.pending_imports.push((path, target));
            continue;
        }
        match target {
            ImportTarget::Channel(ch) => assign_sample(app, ch, &path),
            ImportTarget::NewChannel => {
                new_sampler_channel(app, &path);
            }
            ImportTarget::Picker => {
                crate::playlist::add_audio_source(app, &path);
            }
            ImportTarget::Playlist { track, start } => {
                crate::playlist::add_audio_clip(app, &path, track, start);
            }
        }
    }
}

pub(crate) fn sampler_with_audio(
    params: &SamplerParams,
    root_key: u8,
    sample: &limeshot_dsp::SampleData,
) -> limeshot_core::PluginRef {
    let audio = limeshot_core::EmbeddedAudio {
        name: sample
            .source
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        sample_rate: sample.sample_rate,
        stereo: sample.data.clone(),
    };
    let mut plugin = limeshot_core::LimePlugin::SAMPLER.with_values(
        &limeshot_core::lime::sampler_values(params, root_key),
        false,
        Some(&audio),
    );
    if let Some(source) = &params.sample {
        plugin
            .extra
            .insert("lime-sample-path".into(), serde_json::json!(source.path));
    }
    plugin
}

/// Replace a sampler only after its requested source has decoded successfully.
pub fn assign_sample(app: &mut LimeShotApp, channel: ChannelId, path: &Path) {
    let Some(sample) = import_audio(app, path, ImportTarget::Channel(channel)) else {
        return;
    };
    let Some(ch) = app.project().channel(channel) else {
        return;
    };
    let root_key = ch.root_key;
    let mut params = limeshot_core::lime::sampler_params(&ch.instrument).unwrap_or_default();
    params.sample = Some(SampleRef {
        path: path.to_string_lossy().into_owned(),
    });
    let plugin = sampler_with_audio(&params, root_key, &sample);
    app.exec(Command::Batch(vec![
        Command::ApproveSamplerSource { channel },
        Command::SetInstrument {
            channel,
            instrument: Instrument::Plugin(plugin),
        },
        Command::RenameChannel {
            channel,
            name: file_label(path),
        },
    ]));
}

/// Queue a new sampler; returns its id immediately only when the source is already cached.
pub fn new_sampler_channel(app: &mut LimeShotApp, path: &Path) -> Option<ChannelId> {
    let sample = import_audio(app, path, ImportTarget::NewChannel)?;
    let params = SamplerParams {
        sample: Some(SampleRef {
            path: path.to_string_lossy().into_owned(),
        }),
        ..Default::default()
    };
    let plugin = sampler_with_audio(&params, 60, &sample);
    let id = app.exec(Command::AddChannel {
        name: file_label(path),
        instrument: Instrument::Plugin(plugin),
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
        if ui.small_button("⟳").hint("Re-read folders").clicked() {
            app.ui.browser.refresh();
        }
        if icon_button(ui, Icon::Plus, false, 22.0, "Add a folder to the Browser").clicked()
            && let Some(dir) = rfd::FileDialog::new().pick_folder()
            && !app.settings.browser_folders.contains(&dir)
        {
            app.settings.browser_folders.push(dir);
            app.save_settings();
        }
    });
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            let r = ui.add(
                TextEdit::singleline(&mut app.ui.browser.filter)
                    .hint_text("Search samples")
                    .desired_width(f32::INFINITY),
            );
            if std::mem::take(&mut app.ui.browser.focus_search) {
                r.request_focus();
            }
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
            app.save_settings();
        }
    });
}

/// Show the project's channels with selection, drag payloads, and channel context actions.
/// Double-click opens a loaded plugin's wrapper or the channel's settings; sampler drags carry
/// the sample path when assigned, and other channel drags carry the channel id.
fn project_tree(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let sel = app.selected_channel();
    let chans: Vec<_> = app
        .project()
        .channels
        .iter()
        .map(|c| {
            (
                c.id,
                c.name.clone(),
                c.color,
                if limeshot_core::lime::sampler_params(&c.instrument).is_some() {
                    "Samples"
                } else {
                    c.instrument.kind_label()
                },
            )
        })
        .collect();
    if chans.is_empty() {
        ui.label(RichText::new("No channels yet").color(t.text_faint));
    }
    for (id, name, color, kind) in chans {
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::click_and_drag());
        if resp.drag_started() {
            if let Some(s) = app
                .project()
                .channel(id)
                .and_then(|c| limeshot_core::lime::sampler_params(&c.instrument))
                && let Some(sample) = &s.sample
                && std::path::Path::new(&sample.path).is_file()
            {
                resp.dnd_set_drag_payload(DraggedSample(app.samples.resolve(&sample.path)));
            } else if app.sampler_source(id).is_some() {
                resp.dnd_set_drag_payload(limeshot_core::ClipSource::Audio(id));
            } else {
                resp.dnd_set_drag_payload(id);
            }
        }
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
        if resp.clicked() || resp.secondary_clicked() {
            app.select_channel(id);
        }
        if resp.double_clicked() {
            app.select_channel(id);
            let target = crate::plugins::Target::Channel(id);
            if app.plugins.is_loaded(target) {
                app.plugins.open_window(target, None);
            } else {
                app.show_window(crate::workspace::Win::ChannelSettings);
            }
        }
        resp.context_menu(|ui| {
            if let Some(ch) = app.project().channel(id).cloned() {
                crate::channel_rack::channel_menu(app, ui, &ch);
            }
        });
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
        if resp.hovered() {
            app.hint(format!(
                "{} — click or double-click to preview, drag to the Channel Rack",
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
            if app
                .project()
                .channel(c)
                .and_then(|c| limeshot_core::lime::sampler_params(&c.instrument))
                .is_some() =>
        {
            assign_sample(app, c, path)
        }
        _ => {
            new_sampler_channel(app, path);
        }
    }
}

/// Waveform shared by the sample drag's pointer preview and its snapped Playlist ghost.
/// Caches peaks for loaded samples; draws nothing when the sample is not ready or the rect is
/// smaller than one point in either dimension.
pub(crate) fn paint_sample_wave(
    app: &mut LimeShotApp,
    painter: &egui::Painter,
    rect: Rect,
    path: &Path,
    color: egui::Color32,
) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    let Some(SampleStatus::Ready(sample)) = app.samples.status_of_path(path).cloned() else {
        return;
    };
    let peaks = app
        .ui
        .browser
        .drag_peaks
        .entry(path.to_path_buf())
        .or_insert_with(|| std::sync::Arc::new(crate::playlist::Peaks::of(&sample)))
        .clone();
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let cols = rect.width().ceil().min(4096.0) as usize;
    for x in 0..cols {
        let a = x as f64 / cols as f64 * sample.frames() as f64;
        let b = (x + 1) as f64 / cols as f64 * sample.frames() as f64;
        if let Some((lo, hi)) = peaks.range(a, b) {
            let x = rect.left() + x as f32;
            let h = rect.height() * 0.48;
            painter.line_segment(
                [pos2(x, rect.center().y - hi * h), pos2(x, rect.center().y - lo * h)],
                Stroke::new(1.0, color),
            );
        }
    }
}

/// Show a waveform beside the pointer for a dragged sample or audio clip source.
/// Requests the sample in the background; draws nothing without a sample payload or pointer.
pub(crate) fn drag_preview(app: &mut LimeShotApp, ctx: &egui::Context) {
    let sample = egui::DragAndDrop::payload::<DraggedSample>(ctx)
        .map(|s| s.0.clone())
        .or_else(|| {
            let source = egui::DragAndDrop::payload::<limeshot_core::ClipSource>(ctx)?;
            let limeshot_core::ClipSource::Audio(channel) = *source else {
                return None;
            };
            let params = limeshot_core::lime::sampler_params(&app.project().channel(channel)?.instrument)?;
            Some(app.samples.resolve(&params.sample.as_ref()?.path))
        });
    let Some(sample) = sample else { return };
    app.samples.request_path(sample.clone());
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let rect = Rect::from_min_size(pos + vec2(12.0, 18.0), vec2(160.0, 48.0));
    let t = Theme::current();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("sample-drag-wave"),
    ));
    painter.rect_filled(rect, 3.0, t.raised.gamma_multiply(0.95));
    paint_sample_wave(app, &painter, rect.shrink(3.0), &sample, t.accent);
}
