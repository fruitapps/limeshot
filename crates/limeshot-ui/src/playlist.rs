//! The Playlist: arranges pattern and audio clips on tracks; Song mode plays it.
//!
//! Mouse: left-click empty space places the current pattern (and keeps dragging to move it);
//! drag a clip to move it (across tracks too), drag its right edge to resize and its left edge to
//! trim the start; right-click/drag deletes; Cmd+drag selects; double-click a pattern clip opens
//! the Piano Roll, an audio clip its channel settings; the ▾ marker on a clip opens its menu.
//! Alt disables snapping. Click the ruler to move the song position. Samples dragged in from the
//! Browser become audio clips (FL Studio).

use crate::app::{Focus, LimeShotApp};
use crate::icons::Icon;
use crate::piano_roll::{TimeAxis, draw_ruler, draw_time_grid};
use crate::theme::{Theme, mix, mono, rgb};
use crate::widgets::{icon_button, panel_header, section};
use egui::{
    Align2, Color32, Context, CornerRadius, Key, Modifiers, PointerButton, Pos2, Rect, RichText, Sense, Stroke, Ui,
    Vec2, pos2, vec2,
};
use limeshot_core::*;
use limeshot_engine::PlayMode;

const HEADER_W: f32 = 118.0;
const RULER_H: f32 = 20.0;
const EDGE: f32 = 7.0;

#[derive(Debug, Clone)]
enum Drag {
    Move {
        ids: Vec<ClipId>,
        anchor_start: Tick,
        grab_tick: i64,
        grab_track: i32,
        applied_dt: i64,
        applied_dtrack: i32,
    },
    Resize {
        id: ClipId,
        start: Tick,
    },
    ResizeStart {
        id: ClipId,
    },
    /// Dragging point `index` of an automation shown by `clip`.
    AutoPoint {
        clip: ClipId,
        automation: AutomationId,
        index: usize,
    },
    /// Dragging the tension handle of the segment after point `index`.
    AutoTension {
        automation: AutomationId,
        index: usize,
        grab_y: f32,
        tension: f32,
    },
    Select {
        from: Pos2,
    },
    Erase,
}

pub struct State {
    /// Pixels per beat.
    pub zoom_x: f32,
    pub track_h: f32,
    pub scroll: Vec2,
    pub selection: Vec<ClipId>,
    drag: Option<Drag>,
    menu: Option<(ClipId, Pos2)>,
    /// Waveform overviews of audio clip samples, by stored sample path.
    peaks: std::collections::HashMap<String, std::sync::Arc<Peaks>>,
    /// Geometry of the last drawn frame.
    pub geometry: Option<Geometry>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom_x: 24.0,
            track_h: 44.0,
            scroll: Vec2::ZERO,
            selection: Vec::new(),
            drag: None,
            menu: None,
            peaks: Default::default(),
            geometry: None,
        }
    }
}

struct View {
    grid: Rect,
    ppt: f32,
    track_h: f32,
    scroll: Vec2,
}

impl View {
    fn x(&self, t: Tick) -> f32 {
        self.grid.left() + t as f32 * self.ppt - self.scroll.x
    }
    fn tick_at(&self, x: f32) -> i64 {
        ((x - self.grid.left() + self.scroll.x) / self.ppt).floor() as i64
    }
    fn y(&self, track: u32) -> f32 {
        self.grid.top() + track as f32 * self.track_h - self.scroll.y
    }
    fn track_at(&self, y: f32) -> i32 {
        ((y - self.grid.top() + self.scroll.y) / self.track_h).floor() as i32
    }
    fn clip_rect(&self, c: &Clip) -> Rect {
        Rect::from_min_max(
            pos2(self.x(c.start), self.y(c.track) + 1.0),
            pos2(
                self.x(c.end()).max(self.x(c.start) + 4.0),
                self.y(c.track) + self.track_h - 1.0,
            ),
        )
    }
}

impl TimeAxis for View {
    fn ppt(&self) -> f32 {
        self.ppt
    }
    fn x_of(&self, t: Tick) -> f32 {
        self.x(t)
    }
    fn tick_at_x(&self, x: f32) -> i64 {
        self.tick_at(x)
    }
}

/// Screen geometry of the last drawn Playlist (for tests and tools).
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub grid: Rect,
    pub ppt: f32,
    pub track_h: f32,
    pub scroll: Vec2,
}

impl Geometry {
    /// Screen position of `tick` in the middle of `track`.
    pub fn pos(&self, tick: Tick, track: u32) -> Pos2 {
        pos2(
            self.grid.left() + tick as f32 * self.ppt - self.scroll.x,
            self.grid.top() + (track as f32 + 0.5) * self.track_h - self.scroll.y,
        )
    }
}

const PICKER_W: f32 = 150.0;

/// Body of the Playlist window (FL Studio opens it maximised): a tool strip, the pattern picker
/// on the left, and the arrangement.
pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let sig = app.project().time_signature;
    let bars = app.project().playlist.song_length(sig) / sig.ticks_per_bar();
    panel_header(ui, "", |ui| {
        if icon_button(ui, Icon::Plus, false, 22.0, "Zoom in (Cmd+wheel)").clicked() {
            app.ui.playlist.zoom_x = (app.ui.playlist.zoom_x * 1.25).min(400.0);
        }
        if icon_button(ui, Icon::Minus, false, 22.0, "Zoom out").clicked() {
            app.ui.playlist.zoom_x = (app.ui.playlist.zoom_x / 1.25).max(3.0);
        }
        if ui
            .small_button("+ Track")
            .on_hover_text("Add a playlist track")
            .clicked()
        {
            app.exec(Command::AddPlaylistTrack);
        }
        ui.label(RichText::new(format!("{bars} bars")).font(mono(10.5)).color(t.text_dim));
    });
    let area = ui.available_rect_before_wrap();
    // Claim the space the child UIs below fill, so a floating Playlist's frame covers it.
    ui.advance_cursor_after_rect(area);
    if ui.rect_contains_pointer(area) && ui.input(|i| i.pointer.any_pressed()) {
        app.ui.focus = Focus::Playlist;
    }
    let picker = Rect::from_min_size(area.min, vec2(PICKER_W, area.height()));
    let body_rect = Rect::from_min_max(pos2(picker.right() + 1.0, area.top()), area.max);
    ui.painter()
        .line_segment([picker.right_top(), picker.right_bottom()], Stroke::new(1.0, t.border));
    let mut pick_ui = ui.new_child(egui::UiBuilder::new().id_salt("playlist-picker").max_rect(picker));
    picker_ui(app, &mut pick_ui);
    let mut body_ui = ui.new_child(egui::UiBuilder::new().id_salt("playlist-body").max_rect(body_rect));
    body(app, &mut body_ui);
    let ctx = ui.ctx().clone();
    clip_menu(app, &ctx);
}

/// FL Studio's clip picker: the patterns of the project. Click to choose what the Playlist
/// places; double-click to edit in the Piano Roll.
fn picker_ui(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        section(ui, "Patterns");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            if icon_button(ui, Icon::Plus, false, 20.0, "New pattern (F4)").clicked() {
                crate::menus::add_pattern(app);
            }
        });
    });
    ui.add_space(2.0);
    let cur = app.current_pattern();
    let pats: Vec<(PatternId, String, Rgb, usize)> = app
        .project()
        .patterns
        .iter()
        .map(|p| {
            (
                p.id,
                p.name.clone(),
                p.color,
                app.project()
                    .playlist
                    .clips
                    .iter()
                    .filter(|c| c.pattern() == Some(p.id))
                    .count(),
            )
        })
        .collect();
    egui::ScrollArea::vertical()
        .id_salt("picker")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (id, name, color, uses) in pats {
                let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
                let on = id == cur && app.ui.place.is_none();
                if on {
                    ui.painter().rect_filled(r, 0.0, mix(t.surface, rgb(color), 0.18));
                } else if resp.hovered() {
                    ui.painter().rect_filled(r, 0.0, t.raised);
                }
                ui.painter().rect_filled(
                    Rect::from_min_size(r.min + vec2(10.0, 8.0), vec2(10.0, 10.0)),
                    CornerRadius::same(2),
                    rgb(color),
                );
                ui.painter().text(
                    r.left_center() + vec2(28.0, 0.0),
                    Align2::LEFT_CENTER,
                    &name,
                    crate::theme::prop(12.0),
                    if on { t.text } else { mix(t.text_dim, t.text, 0.5) },
                );
                if uses > 0 {
                    ui.painter().text(
                        r.right_center() - vec2(10.0, 0.0),
                        Align2::RIGHT_CENTER,
                        format!("×{uses}"),
                        mono(10.0),
                        t.text_faint,
                    );
                }
                if resp.clicked() {
                    app.set_current_pattern(id);
                    app.ui.place = None;
                }
                if resp.double_clicked() {
                    app.show_window(crate::workspace::Win::PianoRoll);
                }
                resp.on_hover_text("Click: place this pattern · double-click: edit in the Piano Roll")
                    .context_menu(|ui| {
                        if ui.button("Rename…").clicked() {
                            app.dialog = Some(crate::dialogs::Dialog::rename_pattern(app, id));
                            ui.close();
                        }
                        if ui.button("Clone").clicked() {
                            if let Some(n) = app.exec(Command::ClonePattern(id)) {
                                app.set_current_pattern(PatternId(n));
                            }
                            ui.close();
                        }
                        if ui
                            .add_enabled(app.project().patterns.len() > 1, egui::Button::new("Delete"))
                            .clicked()
                        {
                            app.exec(Command::RemovePattern(id));
                            ui.close();
                        }
                    });
            }
            automation_picker(app, ui);
        });
}

/// The automation clips in the picker (FL Studio lists them with the patterns): click to place.
fn automation_picker(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let autos: Vec<(AutomationId, String, Rgb, usize)> = app
        .project()
        .automations
        .iter()
        .map(|a| {
            let uses = app
                .project()
                .playlist
                .clips
                .iter()
                .filter(|c| c.automation() == Some(a.id))
                .count();
            (a.id, a.name.clone(), a.color, uses)
        })
        .collect();
    if autos.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        section(ui, "Automation");
    });
    for (id, name, color, uses) in autos {
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
        let on = app.ui.place == Some(ClipSource::Automation(id));
        if on {
            ui.painter().rect_filled(r, 0.0, mix(t.surface, rgb(color), 0.18));
        } else if resp.hovered() {
            ui.painter().rect_filled(r, 0.0, t.raised);
        }
        let icon = Rect::from_min_size(r.min + vec2(8.0, 6.0), vec2(14.0, 14.0));
        crate::icons::paint(ui.painter(), icon, Icon::Automation, rgb(color));
        ui.painter().with_clip_rect(r.shrink2(vec2(0.0, 0.0))).text(
            r.left_center() + vec2(28.0, 0.0),
            Align2::LEFT_CENTER,
            &name,
            crate::theme::prop(12.0),
            if on { t.text } else { mix(t.text_dim, t.text, 0.5) },
        );
        if uses > 0 {
            ui.painter().text(
                r.right_center() - vec2(10.0, 0.0),
                Align2::RIGHT_CENTER,
                format!("×{uses}"),
                mono(10.0),
                t.text_faint,
            );
        }
        if resp.clicked() {
            app.ui.place = Some(ClipSource::Automation(id));
        }
        resp.on_hover_text(format!("{name}\nClick: place this automation clip"))
            .context_menu(|ui| {
                if ui.button("Rename…").clicked() {
                    app.dialog = Some(crate::dialogs::Dialog::rename_automation(app, id));
                    ui.close();
                }
                if ui.button("Delete").clicked() {
                    app.exec(Command::RemoveAutomation(id));
                    ui.close();
                }
            });
    }
}

fn body(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let full = ui.available_rect_before_wrap();
    let (full, _) = ui.allocate_exact_size(full.size(), Sense::hover());
    let ruler = Rect::from_min_max(
        pos2(full.left() + HEADER_W, full.top()),
        pos2(full.right(), full.top() + RULER_H),
    );
    let headers = Rect::from_min_max(
        pos2(full.left(), full.top() + RULER_H),
        pos2(full.left() + HEADER_W, full.bottom()),
    );
    let grid = Rect::from_min_max(pos2(full.left() + HEADER_W, full.top() + RULER_H), full.right_bottom());

    let tracks = app.project().playlist.tracks.len() as u32;
    let st = &mut app.ui.playlist;
    let max_y = (tracks as f32 * st.track_h - grid.height()).max(0.0);
    st.scroll.y = st.scroll.y.clamp(0.0, max_y);
    st.scroll.x = st.scroll.x.max(0.0);
    let view = View {
        grid,
        ppt: st.zoom_x / PPQ as f32,
        track_h: st.track_h,
        scroll: st.scroll,
    };
    st.geometry = Some(Geometry {
        grid,
        ppt: view.ppt,
        track_h: view.track_h,
        scroll: view.scroll,
    });
    let sig = app.project().time_signature;
    let painter = ui.painter_at(full);

    painter.rect_filled(grid, 0.0, t.sunken);
    for tr in 0..tracks {
        let y = view.y(tr);
        if y > grid.bottom() || y + view.track_h < grid.top() {
            continue;
        }
        if tr % 2 == 1 {
            painter.rect_filled(
                Rect::from_min_max(pos2(grid.left(), y), pos2(grid.right(), y + view.track_h)).intersect(grid),
                0.0,
                mix(t.sunken, t.surface, 0.35),
            );
        }
        painter.line_segment(
            [
                pos2(grid.left(), y + view.track_h),
                pos2(grid.right(), y + view.track_h),
            ],
            Stroke::new(1.0, t.grid_step),
        );
    }
    draw_time_grid(&painter, &view, grid, sig, Snap::Bar);
    let song_end = app.project().playlist.song_length(sig);
    let xe = view.x(song_end);
    if song_end > 0 && xe > grid.left() && xe < grid.right() {
        painter.line_segment(
            [pos2(xe, grid.top()), pos2(xe, grid.bottom())],
            Stroke::new(1.0, t.accent_dim),
        );
    }

    // Clips.
    let clips = app.project().playlist.clips.clone();
    for c in &clips {
        let r = view.clip_rect(c);
        if !r.intersects(grid) {
            continue;
        }
        let Some((name, color)) = clip_label(app.project(), c) else {
            continue;
        };
        let track_muted = app
            .project()
            .playlist
            .tracks
            .get(c.track as usize)
            .is_some_and(|t| t.muted);
        let sel = app.ui.playlist.selection.contains(&c.id);
        let base = rgb(color);
        let muted = c.muted || track_muted;
        // Saturated body with a solid title strip, as in the ArtCraft timelines.
        let fill = if muted {
            mix(t.raised, base, 0.18)
        } else {
            mix(t.sunken, base, 0.42)
        };
        let strip = if muted { mix(t.raised, base, 0.3) } else { base };
        let rr = r.intersect(grid);
        painter.rect_filled(rr, CornerRadius::same(t.radius_sm), fill);
        let title = Rect::from_min_size(r.min, vec2(r.width(), 14.0)).intersect(grid);
        painter.rect_filled(
            title,
            CornerRadius {
                nw: t.radius_sm,
                ne: t.radius_sm,
                sw: 0,
                se: 0,
            },
            strip,
        );
        let fg = crate::theme::text_on(strip);
        painter.with_clip_rect(rr).text(
            r.min + vec2(15.0, 1.0),
            Align2::LEFT_TOP,
            &name,
            crate::theme::semibold(10.5),
            fg,
        );
        painter
            .with_clip_rect(rr)
            .text(r.min + vec2(4.0, 0.5), Align2::LEFT_TOP, "▾", mono(10.0), fg);
        let body = Rect::from_min_max(pos2(r.left(), r.top() + 15.0), r.max);
        match c.source {
            ClipSource::Audio(ch) => {
                draw_waveform(app, &painter, &view, grid, c, ch, body, mix(base, Color32::WHITE, 0.55))
            }
            ClipSource::Automation(a) => {
                if let Some(a) = app.project().automation(a) {
                    draw_automation(&painter, &view, grid, c, a, r, base);
                }
            }
            ClipSource::Pattern(_) => {}
        }
        // Note preview.
        let p = match c.source {
            ClipSource::Pattern(id) => app.project().pattern(id),
            _ => None,
        };
        if let Some(p) = p
            && body.height() > 6.0
            && !p.notes.is_empty()
        {
            let (lo, hi) = p
                .notes
                .iter()
                .fold((127u8, 0u8), |(a, b), n| (a.min(n.key), b.max(n.key)));
            let span = f32::from(hi.saturating_sub(lo).max(1));
            let window_end = c.offset + c.length;
            for n in p.notes.iter().filter(|n| n.start >= c.offset && n.start < window_end) {
                let x0 = view.x(c.start + n.start - c.offset);
                let x1 = view.x(c.start + (n.end().min(window_end)) - c.offset);
                let y = body.bottom() - 2.0 - f32::from(n.key - lo) / span * (body.height() - 4.0);
                if x0 < grid.right() && x1 > grid.left() {
                    painter.line_segment(
                        [
                            pos2(x0.max(grid.left()), y),
                            pos2(x1.max(x0 + 1.5).min(grid.right()), y),
                        ],
                        Stroke::new(2.0, mix(base, Color32::WHITE, 0.55)),
                    );
                }
            }
        }
        if sel {
            painter.rect_stroke(
                r,
                CornerRadius::same(t.radius_sm),
                Stroke::new(1.5, t.text),
                egui::StrokeKind::Inside,
            );
        }
    }
    if let (Some(Drag::Select { from }), Some(p)) = (&app.ui.playlist.drag, ui.input(|i| i.pointer.hover_pos())) {
        painter.rect(
            Rect::from_two_pos(*from, p),
            0.0,
            t.selection,
            Stroke::new(1.0, t.accent),
            egui::StrokeKind::Inside,
        );
    }
    // Playhead (Song mode).
    if app.ui.mode == PlayMode::Song {
        let x = view.x(app.player.status().position());
        if x >= grid.left() && x <= grid.right() {
            painter.line_segment(
                [pos2(x, ruler.top()), pos2(x, grid.bottom())],
                Stroke::new(1.5, t.playhead),
            );
            painter.add(egui::Shape::convex_polygon(
                vec![
                    pos2(x - 5.0, ruler.top()),
                    pos2(x + 5.0, ruler.top()),
                    pos2(x, ruler.top() + 7.0),
                ],
                t.playhead,
                Stroke::NONE,
            ));
        }
    }

    ruler_ui(app, ui, &view, ruler, sig);
    headers_ui(app, ui, &view, headers, tracks);
    grid_input(app, ui, &view, grid, &clips, tracks);
    sample_drop(app, ui, &view, grid, tracks);
    scroll_input(app, ui, full, grid);
}

/// Name and color shown on a clip: its pattern's, or for an audio clip its channel's.
fn clip_label(p: &Project, c: &Clip) -> Option<(String, Rgb)> {
    match c.source {
        ClipSource::Pattern(id) => p.pattern(id).map(|x| (x.name.clone(), x.color)),
        ClipSource::Audio(id) => p.channel(id).map(|x| (x.name.clone(), x.color)),
        ClipSource::Automation(id) => p.automation(id).map(|x| (x.name.clone(), x.color)),
    }
}

/// Min/max of the mono signal per block of `Peaks::BLOCK` frames.
pub struct Peaks {
    blocks: Vec<(f32, f32)>,
}

impl Peaks {
    const BLOCK: usize = 256;

    fn of(s: &limeshot_dsp::SampleData) -> Self {
        let blocks = (0..s.frames().div_ceil(Self::BLOCK))
            .map(|b| {
                let (mut lo, mut hi) = (0.0f32, 0.0f32);
                for i in b * Self::BLOCK..((b + 1) * Self::BLOCK).min(s.frames()) {
                    let (l, r) = s.frame(i);
                    let m = 0.5 * (l + r);
                    lo = lo.min(m);
                    hi = hi.max(m);
                }
                (lo, hi)
            })
            .collect();
        Self { blocks }
    }

    /// Min/max over frames `a..b`.
    fn range(&self, a: f64, b: f64) -> Option<(f32, f32)> {
        let i = (a.max(0.0) as usize) / Self::BLOCK;
        let j = ((b.max(0.0) as usize) / Self::BLOCK)
            .max(i)
            .min(self.blocks.len().checked_sub(1)?);
        self.blocks
            .get(i..=j)?
            .iter()
            .copied()
            .reduce(|x, y| (x.0.min(y.0), x.1.max(y.1)))
    }
}

/// The sample an audio clip plays, its first frame (sampler start trim) and the source frames per
/// tick at the project tempo (including the sampler's pitch).
fn clip_audio(
    app: &LimeShotApp,
    ch: ChannelId,
) -> Option<(String, std::sync::Arc<limeshot_dsp::SampleData>, f64, f64)> {
    let Instrument::Sampler(sp) = &app.project().channel(ch)?.instrument else {
        return None;
    };
    let stored = sp.sample.as_ref()?.path.clone();
    let s = limeshot_sequencer::SampleLookup::sample(&app.samples, &stored)?;
    let secs_per_tick = 60.0 / (app.project().tempo.bpm() * PPQ as f64);
    let rate = 2f64.powf(f64::from(sp.pitch) / 12.0);
    let per_tick = secs_per_tick * f64::from(s.sample_rate) * rate;
    let first = f64::from(sp.start.min(sp.end)) * s.frames() as f64;
    Some((stored, s, first, per_tick))
}

fn draw_waveform(
    app: &mut LimeShotApp,
    painter: &egui::Painter,
    view: &View,
    grid: Rect,
    c: &Clip,
    ch: ChannelId,
    body: Rect,
    color: Color32,
) {
    if body.height() < 6.0 {
        return;
    }
    let Some((stored, sample, first, per_tick)) = clip_audio(app, ch) else {
        return;
    };
    let peaks = app
        .ui
        .playlist
        .peaks
        .entry(stored)
        .or_insert_with(|| std::sync::Arc::new(Peaks::of(&sample)))
        .clone();
    let mid = body.center().y;
    let half = body.height() * 0.5 - 2.0;
    let x0 = body.left().max(grid.left()).floor();
    let x1 = body.right().min(grid.right());
    let mut x = x0;
    while x < x1 {
        let t0 = view.tick_at(x) - c.start as i64 + c.offset as i64;
        let t1 = view.tick_at(x + 1.0) - c.start as i64 + c.offset as i64;
        let (a, b) = (first + t0 as f64 * per_tick, first + t1 as f64 * per_tick);
        if let Some((lo, hi)) = peaks.range(a, b) {
            painter.line_segment(
                [
                    pos2(x + 0.5, mid - hi.clamp(-1.0, 1.0) * half),
                    pos2(x + 0.5, mid - lo.clamp(-1.0, 1.0) * half + 0.5),
                ],
                Stroke::new(1.0, color),
            );
        }
        x += 1.0;
    }
}

/// The length in ticks of a whole sample at the project tempo (for new audio clips).
fn sample_ticks(app: &LimeShotApp, s: &limeshot_dsp::SampleData) -> Tick {
    let beats = s.duration_secs() * app.project().tempo.bpm() / 60.0;
    ((beats * PPQ as f64).round() as Tick).max(1)
}

/// Samples dragged from the Browser become audio clips where they are dropped.
fn sample_drop(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("pl_drop"), Sense::hover());
    if resp.dnd_hover_payload::<crate::browser::DraggedSample>().is_some() {
        app.hint("Drop to place the sample as an audio clip");
    }
    let Some(payload) = resp.dnd_release_payload::<crate::browser::DraggedSample>() else {
        return;
    };
    let Some(pos) = ui.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) else {
        return;
    };
    let track = view.track_at(pos.y).clamp(0, tracks.saturating_sub(1) as i32) as u32;
    let sig = app.project().time_signature;
    let start = app.ui.snap.floor(view.tick_at(pos.x).max(0) as u64, sig);
    if let Some(id) = add_audio_clip(app, &payload.0, track, start) {
        app.ui.playlist.selection = vec![id];
    }
}

/// Place the sample file at `path` as an audio clip (decoding it now to know its length).
pub fn add_audio_clip(app: &mut LimeShotApp, path: &std::path::Path, track: u32, start: Tick) -> Option<ClipId> {
    let stored = path.to_string_lossy().into_owned();
    let sample = match app.samples.load_blocking(&stored) {
        Ok(s) => s,
        Err(e) => {
            app.toast_error(format!("Could not load {}: {e}", path.display()));
            return None;
        }
    };
    let length = sample_ticks(app, &sample);
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Audio".into());
    let id = app.exec(Command::AddAudioClip {
        sample: SampleRef { path: stored },
        name,
        track,
        start,
        length,
    })?;
    if let Some(ch) = app.project().clip(ClipId(id)).and_then(Clip::audio_channel) {
        app.select_channel(ch);
    }
    Some(ClipId(id))
}

/// Where an automation clip draws its curve: below the title strip.
fn curve_area(r: Rect) -> Rect {
    Rect::from_min_max(pos2(r.left(), r.top() + 17.0), pos2(r.right(), r.bottom() - 3.0))
}

fn draw_automation(painter: &egui::Painter, view: &View, grid: Rect, c: &Clip, a: &Automation, r: Rect, base: Color32) {
    let area = curve_area(r);
    if area.height() < 6.0 {
        return;
    }
    let y_of = |v: f32| area.bottom() - v.clamp(0.0, 1.0) * area.height();
    let local = |x: f32| (view.tick_at(x) - c.start as i64 + c.offset as i64).max(0) as u64;
    let x0 = r.left().max(grid.left());
    let x1 = r.right().min(grid.right());
    let line = mix(base, Color32::WHITE, 0.6);
    let fill = base.gamma_multiply(0.45);
    let mut pts = Vec::new();
    let mut x = x0;
    while x <= x1 {
        if let Some(v) = a.value_at(local(x)) {
            let y = y_of(v);
            painter.rect_filled(
                Rect::from_min_max(pos2(x, y), pos2((x + 2.0).min(x1), area.bottom())),
                0.0,
                fill,
            );
            pts.push(pos2(x, y));
        }
        x += 2.0;
    }
    painter.add(egui::Shape::line(pts, Stroke::new(1.5, line)));
    let window = c.offset..c.offset + c.length;
    for (i, p) in a.points.iter().enumerate() {
        if window.contains(&p.tick) {
            let x = view.x(c.start + p.tick - c.offset);
            if x >= grid.left() && x <= grid.right() {
                painter.circle(pos2(x, y_of(p.value)), 3.5, base, Stroke::new(1.5, line));
            }
        }
        if let Some(n) = a.points.get(i + 1) {
            let mid = (p.tick + n.tick) / 2;
            if window.contains(&mid) && n.tick - p.tick > 1 {
                let x = view.x(c.start + mid - c.offset);
                if x >= grid.left() && x <= grid.right() && n.tick - p.tick > 0 {
                    let y = y_of(a.value_at(mid).unwrap_or(p.value));
                    painter.circle_stroke(pos2(x, y), 2.5, Stroke::new(1.0, line));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AutoHit {
    Point(usize),
    /// The tension handle of the segment after this point.
    Tension(usize),
    /// On the line between points: where a new point would go (local tick, value).
    Curve(Tick, f32),
}

/// What of an automation curve is under `p`, if any.
fn automation_hit(app: &LimeShotApp, view: &View, clips: &[Clip], p: Pos2) -> Option<(Clip, AutomationId, AutoHit)> {
    let c = clips.iter().rev().find(|c| view.clip_rect(c).contains(p))?;
    let aid = c.automation()?;
    let a = app.project().automation(aid)?;
    let r = view.clip_rect(c);
    let area = curve_area(r);
    if area.height() < 12.0 || p.y < area.top() - 4.0 {
        return None;
    }
    let y_of = |v: f32| area.bottom() - v.clamp(0.0, 1.0) * area.height();
    let window = c.offset..c.offset + c.length;
    let near = |q: Pos2, d: f32| q.distance(p) <= d;
    for (i, pt) in a.points.iter().enumerate() {
        if window.contains(&pt.tick) && near(pos2(view.x(c.start + pt.tick - c.offset), y_of(pt.value)), 6.0) {
            return Some((c.clone(), aid, AutoHit::Point(i)));
        }
    }
    for (i, w) in a.points.windows(2).enumerate() {
        let mid = (w[0].tick + w[1].tick) / 2;
        if window.contains(&mid) && w[1].tick - w[0].tick > 1 {
            let q = pos2(
                view.x(c.start + mid - c.offset),
                y_of(a.value_at(mid).unwrap_or(w[0].value)),
            );
            if near(q, 5.0) {
                return Some((c.clone(), aid, AutoHit::Tension(i)));
            }
        }
    }
    // Keep clear of the edges so they still resize the clip.
    if p.x < r.left() + EDGE || p.x > r.right() - EDGE {
        return None;
    }
    let sig = app.project().time_signature;
    let local = (view.tick_at(p.x) - c.start as i64 + c.offset as i64).max(0) as u64;
    let v = a.value_at(local)?;
    if (y_of(v) - p.y).abs() <= 5.0 {
        let tick = app.ui.snap.round(local, sig).clamp(c.offset, c.offset + c.length - 1);
        return Some((c.clone(), aid, AutoHit::Curve(tick, v)));
    }
    None
}

fn ruler_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, ruler: Rect, sig: TimeSignature) {
    draw_ruler(&ui.painter_at(ruler), view, ruler, sig);
    let resp = ui.interact(ruler, ui.id().with("pl_ruler"), Sense::click_and_drag());
    if (resp.clicked() || resp.dragged())
        && let Some(p) = resp.interact_pointer_pos()
    {
        let tick = Snap::Beat.floor(view.tick_at(p.x).max(0) as u64, sig);
        app.set_mode(PlayMode::Song);
        app.locate(tick);
    }
    resp.on_hover_text("Click or drag to set the song position (switches to Song mode)");
}

fn headers_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, area: Rect, tracks: u32) {
    let t = Theme::current();
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 0.0, t.surface);
    let names: Vec<(String, bool)> = app
        .project()
        .playlist
        .tracks
        .iter()
        .map(|tr| (tr.name.clone(), tr.muted))
        .collect();
    for tr in 0..tracks {
        let y = view.y(tr);
        if y > area.bottom() || y + view.track_h < area.top() {
            continue;
        }
        let r = Rect::from_min_max(pos2(area.left(), y), pos2(area.right() - 2.0, y + view.track_h - 1.0));
        if r.bottom() < area.top() {
            continue;
        }
        let (name, muted) = names.get(tr as usize).cloned().unwrap_or_default();
        painter.rect_filled(r.intersect(area), CornerRadius::same(2), t.raised);
        let led = Rect::from_center_size(pos2(r.right() - 12.0, r.center().y), vec2(10.0, 10.0));
        if led.top() >= area.top() {
            painter.circle_filled(led.center(), 4.5, if muted { t.raised_hover } else { t.accent });
        }
        if r.center().y > area.top() {
            painter.text(
                pos2(r.left() + 6.0, r.center().y),
                Align2::LEFT_CENTER,
                &name,
                crate::theme::prop(11.5),
                if muted { t.text_dim } else { t.text },
            );
        }
        let led_resp = ui.interact(led.intersect(area), ui.id().with(("pl_mute", tr)), Sense::click());
        if led_resp.clicked() {
            app.exec(Command::SetPlaylistTrackMute {
                track: tr,
                muted: !muted,
            });
        }
        led_resp.on_hover_text("Mute track");
        let name_resp = ui.interact(
            Rect::from_min_max(r.min, pos2(led.left() - 2.0, r.max.y)).intersect(area),
            ui.id().with(("pl_name", tr)),
            Sense::click(),
        );
        if name_resp.double_clicked() {
            app.dialog = Some(crate::dialogs::Dialog::rename_track(app, tr));
        }
        name_resp.on_hover_text("Double-click to rename");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Body,
    LeftEdge,
    RightEdge,
    Menu,
}

fn hit(view: &View, clips: &[Clip], p: Pos2) -> Option<(Clip, Part)> {
    clips.iter().rev().find_map(|c| {
        let r = view.clip_rect(c);
        r.contains(p).then(|| {
            let edge = EDGE.min(r.width() * 0.3);
            let part = if p.x <= r.left() + 13.0 && p.y <= r.top() + 13.0 {
                Part::Menu
            } else if p.x >= r.right() - edge {
                Part::RightEdge
            } else if p.x <= r.left() + edge {
                Part::LeftEdge
            } else {
                Part::Body
            };
            (c.clone(), part)
        })
    })
}

/// Make a clicked clip's source the thing being edited: its pattern, or its audio channel.
fn focus_source(app: &mut LimeShotApp, c: &Clip) {
    match c.source {
        ClipSource::Pattern(p) => {
            app.set_current_pattern(p);
            app.ui.place = None;
        }
        ClipSource::Audio(ch) => app.select_channel(ch),
        ClipSource::Automation(_) => app.ui.place = Some(c.source),
    }
}

fn grid_input(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, clips: &[Clip], tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("pl_grid"), Sense::click_and_drag());
    let sig = app.project().time_signature;
    let (mods, pointer) = ui.input(|i| (i.modifiers, i.pointer.interact_pos()));
    let snap = if mods.alt { Snap::None } else { app.ui.snap };
    if let Some(p) = resp.hover_pos() {
        match hit(view, clips, p).map(|h| h.1) {
            Some(Part::LeftEdge | Part::RightEdge) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal),
            Some(Part::Menu) => ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand),
            Some(Part::Body) => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
            None => {}
        }
        let tk = view.tick_at(p.x).max(0) as u64;
        app.hint(format!(
            "{} — left: place/move/resize (either edge) · right: delete · double-click: edit · ▾: clip menu · Cmd+drag: select · drop samples here",
            BarBeatTick::from_ticks(tk, sig)
        ));
    }
    let primary = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
    let secondary =
        resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary));
    // Automation curves take presses on their points, tension handles and line first.
    let auto = pointer.and_then(|p| automation_hit(app, view, clips, p).map(|h| (p, h)));
    if let Some((_, (_, a, h))) = &auto
        && resp.hovered()
        && app.ui.playlist.drag.is_none()
    {
        ui.ctx().set_cursor_icon(match h {
            AutoHit::Point(_) => egui::CursorIcon::Move,
            AutoHit::Tension(_) => egui::CursorIcon::ResizeVertical,
            AutoHit::Curve(..) => egui::CursorIcon::Crosshair,
        });
        let name = app.project().automation(*a).map(|a| a.name.clone()).unwrap_or_default();
        app.hint(format!(
            "{name} — drag a point · drag ○ between points: curve · left/right-click the line: add a point · right-click a point: delete"
        ));
    }
    let mut primary = primary;
    let mut secondary = secondary;
    if (primary || secondary)
        && !mods.command
        && let Some((p, (clip, a, h))) = auto
    {
        let points = app
            .project()
            .automation(a)
            .map(|a| a.points.clone())
            .unwrap_or_default();
        match (h, primary) {
            (AutoHit::Point(i), true) => {
                app.doc.begin_gesture();
                app.ui.playlist.drag = Some(Drag::AutoPoint {
                    clip: clip.id,
                    automation: a,
                    index: i,
                });
            }
            (AutoHit::Point(i), false) => {
                if points.len() > 1 {
                    let mut pts = points;
                    pts.remove(i);
                    app.exec(Command::SetAutomationPoints {
                        automation: a,
                        points: pts,
                    });
                }
            }
            (AutoHit::Tension(i), true) => {
                app.doc.begin_gesture();
                app.ui.playlist.drag = Some(Drag::AutoTension {
                    automation: a,
                    index: i,
                    grab_y: p.y,
                    tension: points.get(i).map_or(0.0, |x| x.tension),
                });
            }
            (AutoHit::Tension(_), false) => {}
            (AutoHit::Curve(tick, value), left) => {
                let mut pts = points;
                let index = pts.partition_point(|x| x.tick < tick);
                if pts.get(index).is_none_or(|x| x.tick != tick) {
                    pts.insert(
                        index,
                        AutomationPoint {
                            tick,
                            value,
                            tension: 0.0,
                        },
                    );
                    if left {
                        app.doc.begin_gesture();
                    }
                    app.exec(Command::SetAutomationPoints {
                        automation: a,
                        points: pts,
                    });
                    if left {
                        app.ui.playlist.drag = Some(Drag::AutoPoint {
                            clip: clip.id,
                            automation: a,
                            index,
                        });
                    }
                }
            }
        }
        primary = false;
        secondary = false;
    }
    if primary && let Some(p) = pointer {
        let st = &mut app.ui.playlist;
        if mods.command {
            match hit(view, clips, p) {
                Some((c, _)) => {
                    if let Some(i) = st.selection.iter().position(|x| *x == c.id) {
                        st.selection.remove(i);
                    } else {
                        st.selection.push(c.id);
                    }
                }
                None => {
                    if !mods.shift {
                        st.selection.clear();
                    }
                    st.drag = Some(Drag::Select { from: p });
                }
            }
        } else {
            match hit(view, clips, p) {
                Some((c, Part::Menu)) => {
                    st.menu = Some((c.id, p));
                }
                Some((c, part)) => {
                    if !st.selection.contains(&c.id) {
                        st.selection = vec![c.id];
                    }
                    let ids = st.selection.clone();
                    focus_source(app, &c);
                    app.doc.begin_gesture();
                    app.ui.playlist.drag = Some(match part {
                        Part::RightEdge => Drag::Resize {
                            id: c.id,
                            start: c.start,
                        },
                        Part::LeftEdge => Drag::ResizeStart { id: c.id },
                        _ => Drag::Move {
                            ids,
                            anchor_start: c.start,
                            grab_tick: view.tick_at(p.x),
                            grab_track: view.track_at(p.y),
                            applied_dt: 0,
                            applied_dtrack: 0,
                        },
                    });
                }
                None => {
                    let track = view.track_at(p.y);
                    if track >= 0 && (track as u32) < tracks {
                        let start = snap.floor(view.tick_at(p.x).max(0) as u64, sig);
                        let pattern = app.current_pattern();
                        let automation = match app.ui.place {
                            Some(ClipSource::Automation(a)) => app.project().automation(a).map(|a| (a.id, a.length)),
                            _ => None,
                        };
                        app.doc.begin_gesture();
                        let cmd = match automation {
                            Some((a, length)) => Command::PlaceClip {
                                source: ClipSource::Automation(a),
                                track: track as u32,
                                start,
                                length,
                                offset: 0,
                            },
                            None => Command::AddClip {
                                pattern,
                                track: track as u32,
                                start,
                                length: None,
                            },
                        };
                        if let Some(id) = app.exec(cmd) {
                            let id = ClipId(id);
                            app.ui.playlist.selection = vec![id];
                            app.ui.playlist.drag = Some(Drag::Move {
                                ids: vec![id],
                                anchor_start: start,
                                grab_tick: view.tick_at(p.x),
                                grab_track: track,
                                applied_dt: 0,
                                applied_dtrack: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    if secondary {
        app.doc.begin_gesture();
        app.ui.playlist.drag = Some(Drag::Erase);
    }
    if let Some(p) = pointer {
        let mut drag = app.ui.playlist.drag.take();
        match &mut drag {
            Some(Drag::Move {
                ids,
                anchor_start,
                grab_tick,
                grab_track,
                applied_dt,
                applied_dtrack,
            }) => {
                let target = (*anchor_start as i64 + view.tick_at(p.x) - *grab_tick).max(0) as u64;
                let dt = snap.round(target, sig) as i64 - *anchor_start as i64;
                let dtrack = view.track_at(p.y) - *grab_track;
                if dt != *applied_dt || dtrack != *applied_dtrack {
                    app.exec(Command::MoveClips {
                        clips: ids.clone(),
                        dt: dt - *applied_dt,
                        dtrack: dtrack - *applied_dtrack,
                    });
                    *applied_dt = dt;
                    *applied_dtrack = dtrack;
                }
            }
            Some(Drag::Resize { id, start }) => {
                let end = snap.round(view.tick_at(p.x).max(0) as u64, sig);
                let min = snap.ticks(sig).unwrap_or(STEP_TICKS).max(1);
                let len = end.saturating_sub(*start).max(min);
                if app.project().clip(*id).is_some_and(|c| c.length != len) {
                    app.exec(Command::ResizeClip { clip: *id, length: len });
                }
            }
            Some(Drag::AutoPoint {
                clip,
                automation,
                index,
            }) => {
                if let (Some(c), Some(a)) = (
                    app.project().clip(*clip).cloned(),
                    app.project().automation(*automation).cloned(),
                ) {
                    let r = view.clip_rect(&c);
                    let area = curve_area(r);
                    let local = (view.tick_at(p.x) - c.start as i64 + c.offset as i64).max(0) as u64;
                    let local = snap.round(local, sig);
                    let lo = if *index == 0 { 0 } else { a.points[*index - 1].tick + 1 };
                    let hi = a.points.get(*index + 1).map_or(Tick::MAX, |x| x.tick.saturating_sub(1));
                    let tick = local.clamp(lo, hi.max(lo));
                    let value = ((area.bottom() - p.y) / area.height().max(1.0)).clamp(0.0, 1.0);
                    let mut pts = a.points.clone();
                    if let Some(pt) = pts.get_mut(*index)
                        && (pt.tick != tick || pt.value != value)
                    {
                        pt.tick = tick;
                        pt.value = value;
                        app.exec(Command::SetAutomationPoints {
                            automation: *automation,
                            points: pts,
                        });
                    }
                    if let Some(m) = a.target.mapping(app.project()) {
                        app.hint(format!("{}: {:.3}", a.name, m.to_value(value)));
                    }
                }
            }
            Some(Drag::AutoTension {
                automation,
                index,
                grab_y,
                tension,
            }) => {
                if let Some(a) = app.project().automation(*automation).cloned()
                    && let (Some(x), Some(y)) = (a.points.get(*index), a.points.get(*index + 1))
                {
                    // Dragging up bends the curve up, whichever way the segment goes.
                    let dir = if y.value >= x.value { -1.0 } else { 1.0 };
                    let t = (*tension + dir * (*grab_y - p.y) / 80.0).clamp(-1.0, 1.0);
                    let mut pts = a.points.clone();
                    pts[*index].tension = t;
                    app.exec(Command::SetAutomationPoints {
                        automation: *automation,
                        points: pts,
                    });
                    app.hint(format!(
                        "Point {} to {} tension {:.0}%",
                        *index + 1,
                        *index + 2,
                        t * 100.0
                    ));
                }
            }
            Some(Drag::ResizeStart { id }) => {
                let start = snap.round(view.tick_at(p.x).max(0) as u64, sig);
                if app.project().clip(*id).is_some_and(|c| c.start != start) {
                    app.exec(Command::ResizeClipStart { clip: *id, start });
                }
            }
            Some(Drag::Erase) => {
                let current = app.project().playlist.clips.clone();
                if let Some((c, _)) = hit(view, &current, p) {
                    app.exec(Command::RemoveClips(vec![c.id]));
                    app.ui.playlist.selection.retain(|x| *x != c.id);
                }
            }
            Some(Drag::Select { from }) => {
                let r = Rect::from_two_pos(*from, p);
                for c in clips {
                    if view.clip_rect(c).intersects(r) && !app.ui.playlist.selection.contains(&c.id) {
                        app.ui.playlist.selection.push(c.id);
                    }
                }
            }
            None => {}
        }
        app.ui.playlist.drag = drag;
    }
    if app.ui.playlist.drag.is_some() && !ui.input(|i| i.pointer.any_down()) {
        if !matches!(app.ui.playlist.drag, Some(Drag::Select { .. })) {
            app.doc.end_gesture();
        }
        app.ui.playlist.drag = None;
    }
    if resp.double_clicked()
        && let Some(p) = pointer
        && let Some((c, part)) = hit(view, clips, p)
        && part != Part::Menu
    {
        focus_source(app, &c);
        match c.source {
            ClipSource::Pattern(_) => app.show_window(crate::workspace::Win::PianoRoll),
            ClipSource::Audio(_) => app.show_window(crate::workspace::Win::ChannelSettings),
            ClipSource::Automation(_) => {}
        }
    }
}

fn clip_menu(app: &mut LimeShotApp, ctx: &Context) {
    let Some((id, pos)) = app.ui.playlist.menu else { return };
    let Some(clip) = app.project().clip(id).cloned() else {
        app.ui.playlist.menu = None;
        return;
    };
    let users = app
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| c.source == clip.source)
        .count();
    let mut close = false;
    let area = egui::Area::new(egui::Id::new("clip_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_min_width(170.0);
                let name = clip_label(app.project(), &clip).map(|l| l.0).unwrap_or_default();
                ui.label(RichText::new(name).strong());
                ui.label(
                    RichText::new(format!("Used by {users} clip(s)"))
                        .small()
                        .color(Theme::current().text_dim),
                );
                ui.separator();
                match clip.source {
                    ClipSource::Pattern(pid) => {
                        if ui
                            .add_enabled(users > 1, egui::Button::new("Make unique"))
                            .on_hover_text("Give this clip its own copy of the pattern")
                            .clicked()
                        {
                            if let Some(p) = app.exec(Command::MakeClipUnique(id)) {
                                app.set_current_pattern(PatternId(p));
                            }
                            close = true;
                        }
                        if ui.button("Edit pattern").clicked() {
                            app.set_current_pattern(pid);
                            app.show_window(crate::workspace::Win::PianoRoll);
                            close = true;
                        }
                        if ui.button("Rename pattern…").clicked() {
                            app.dialog = Some(crate::dialogs::Dialog::rename_pattern(app, pid));
                            close = true;
                        }
                    }
                    ClipSource::Audio(ch) => {
                        if ui.button("Channel settings").clicked() {
                            app.select_channel(ch);
                            app.show_window(crate::workspace::Win::ChannelSettings);
                            close = true;
                        }
                    }
                    ClipSource::Automation(aid) => {
                        if ui
                            .add_enabled(users > 1, egui::Button::new("Make unique"))
                            .on_hover_text("Give this clip its own copy of the automation")
                            .clicked()
                        {
                            app.exec(Command::MakeClipUnique(id));
                            close = true;
                        }
                        if ui.button("Rename automation…").clicked() {
                            app.dialog = Some(crate::dialogs::Dialog::rename_automation(app, aid));
                            close = true;
                        }
                        if ui.button("Reset to a flat line").clicked() {
                            let v = app.project().automation(aid).and_then(|a| a.value_at(clip.offset));
                            if let Some(value) = v {
                                app.exec(Command::SetAutomationPoints {
                                    automation: aid,
                                    points: vec![AutomationPoint {
                                        tick: 0,
                                        value,
                                        tension: 0.0,
                                    }],
                                });
                            }
                            close = true;
                        }
                        if ui
                            .button(RichText::new("Delete automation").color(Theme::current().danger))
                            .on_hover_text("Delete the automation and all its clips")
                            .clicked()
                        {
                            app.exec(Command::RemoveAutomation(aid));
                            close = true;
                        }
                    }
                }
                if ui
                    .button(if clip.muted { "Unmute clip" } else { "Mute clip" })
                    .clicked()
                {
                    app.exec(Command::SetClipMuted {
                        clip: id,
                        muted: !clip.muted,
                    });
                    close = true;
                }
                let fit = match clip.source {
                    ClipSource::Pattern(_) => "Fit to pattern length",
                    ClipSource::Audio(_) => "Fit to sample length",
                    ClipSource::Automation(_) => "Fit to automation length",
                };
                if ui.button(fit).clicked() {
                    if let Some(len) = source_length(app, &clip) {
                        app.exec(Command::ResizeClip {
                            clip: id,
                            length: len.saturating_sub(clip.offset).max(1),
                        });
                    }
                    close = true;
                }
                if ui.button("Duplicate").clicked() {
                    app.exec(Command::DuplicateClips(vec![id]));
                    close = true;
                }
                if ui
                    .button(RichText::new("Delete").color(Theme::current().danger))
                    .clicked()
                {
                    app.exec(Command::RemoveClips(vec![id]));
                    close = true;
                }
            });
        });
    let clicked_outside = ctx.input(|i| i.pointer.any_pressed()) && !area.response.contains_pointer();
    if close || clicked_outside || ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.ui.playlist.menu = None;
    }
}

/// The full length of a clip's source in ticks: its pattern, or its whole sample.
fn source_length(app: &LimeShotApp, c: &Clip) -> Option<Tick> {
    match c.source {
        ClipSource::Pattern(p) => {
            let sig = app.project().time_signature;
            app.project().pattern(p).map(|p| p.length(sig))
        }
        ClipSource::Audio(ch) => {
            let (_, s, first, per_tick) = clip_audio(app, ch)?;
            let Instrument::Sampler(sp) = &app.project().channel(ch)?.instrument else {
                return None;
            };
            let last = f64::from(sp.start.max(sp.end)) * s.frames() as f64;
            Some((((last - first) / per_tick).round() as Tick).max(1))
        }
        ClipSource::Automation(a) => app.project().automation(a).map(|a| {
            let last = a.points.last().map_or(0, |p| p.tick + 1);
            a.length.max(last)
        }),
    }
}

fn scroll_input(app: &mut LimeShotApp, ui: &mut Ui, full: Rect, grid: Rect) {
    if !ui.rect_contains_pointer(full) {
        return;
    }
    let st = &mut app.ui.playlist;
    if crate::piano_roll::wheel_zoom(ui, grid, &mut st.zoom_x, &mut st.scroll.x, 3.0, 400.0) {
        return;
    }
    let (delta, mods) = ui.input(|i| (i.smooth_scroll_delta, i.modifiers));
    if delta == Vec2::ZERO {
        return;
    }
    if mods.shift {
        st.scroll.x = (st.scroll.x - delta.y - delta.x).max(0.0);
    } else {
        st.scroll.x = (st.scroll.x - delta.x).max(0.0);
        st.scroll.y -= delta.y;
    }
}

pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key;
    let sel = app.ui.playlist.selection.clone();
    if (key(ctx, Modifiers::NONE, Key::Delete) || key(ctx, Modifiers::NONE, Key::Backspace)) && !sel.is_empty() {
        app.exec(Command::RemoveClips(sel.clone()));
        app.ui.playlist.selection.clear();
    }
    if key(ctx, Modifiers::COMMAND, Key::A) {
        app.ui.playlist.selection = app.project().playlist.clips.iter().map(|c| c.id).collect();
    }
    if key(ctx, Modifiers::COMMAND, Key::D) {
        app.ui.playlist.selection.clear();
    }
    let cut = key(ctx, Modifiers::COMMAND, Key::X);
    if (cut || key(ctx, Modifiers::COMMAND, Key::C)) && !sel.is_empty() {
        let clips: Vec<crate::app::CopiedClip> = app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| sel.contains(&c.id))
            .map(|c| crate::app::CopiedClip {
                source: c.source,
                track: c.track,
                start: c.start,
                length: c.length,
                offset: c.offset,
            })
            .collect();
        app.clipboard = Some(crate::app::Clipboard::Clips(clips));
        if cut {
            app.exec(Command::RemoveClips(sel.clone()));
            app.ui.playlist.selection.clear();
        }
    }
    if key(ctx, Modifiers::COMMAND, Key::V)
        && let Some(crate::app::Clipboard::Clips(clips)) = app.clipboard.clone()
    {
        let sig = app.project().time_signature;
        let origin = clips.iter().map(|c| c.start).min().unwrap_or(0);
        let target = if app.ui.mode == PlayMode::Song {
            Snap::Bar.floor(app.player.status().position(), sig)
        } else {
            origin
        };
        let before: Vec<ClipId> = app.project().playlist.clips.iter().map(|c| c.id).collect();
        let cmds = clips
            .iter()
            .map(|c| Command::PlaceClip {
                source: c.source,
                track: c.track,
                start: c.start - origin + target,
                length: c.length,
                offset: c.offset,
            })
            .collect();
        app.exec(Command::Batch(cmds));
        app.ui.playlist.selection = app
            .project()
            .playlist
            .clips
            .iter()
            .map(|c| c.id)
            .filter(|id| !before.contains(id))
            .collect();
    }
    if key(ctx, Modifiers::COMMAND, Key::B) && !sel.is_empty() {
        let before: Vec<ClipId> = app.project().playlist.clips.iter().map(|c| c.id).collect();
        app.exec(Command::DuplicateClips(sel.clone()));
        app.ui.playlist.selection = app
            .project()
            .playlist
            .clips
            .iter()
            .map(|c| c.id)
            .filter(|id| !before.contains(id))
            .collect();
    }
    if key(ctx, Modifiers::NONE, Key::Home) {
        app.locate(0);
    }
    if sel.is_empty() {
        return;
    }
    let sig = app.project().time_signature;
    let g = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS) as i64;
    for (k, dt, dtrack) in [
        (Key::ArrowLeft, -g, 0),
        (Key::ArrowRight, g, 0),
        (Key::ArrowUp, 0, -1),
        (Key::ArrowDown, 0, 1),
    ] {
        if key(ctx, Modifiers::NONE, k) {
            app.exec(Command::MoveClips {
                clips: sel.clone(),
                dt,
                dtrack,
            });
        }
    }
}
