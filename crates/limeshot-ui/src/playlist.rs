//! The Playlist: arranges pattern clips on tracks; Song mode plays it.
//!
//! Mouse: left-click empty space places the current pattern (and keeps dragging to move it);
//! drag a clip to move it (across tracks too), drag its right edge to resize; right-click/drag
//! deletes; Cmd+drag selects; double-click a clip opens its pattern in the Piano Roll; the ▾
//! marker on a clip opens its menu (Make unique …). Alt disables snapping. Click the ruler to move
//! the song position.

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

/// The Playlist fills the central area (FL Studio opens it maximised): a header strip, the
/// pattern picker on the left, and the arrangement.
pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let cur = app.current_pattern();
    let cur_name = app.project().pattern(cur).map(|p| p.name.clone()).unwrap_or_default();
    let sig = app.project().time_signature;
    let bars = app.project().playlist.song_length(sig) / sig.ticks_per_bar();
    panel_header(ui, &format!("Playlist  ·  Arrangement  ›  {cur_name}"), |ui| {
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
    if ui.rect_contains_pointer(area) && ui.input(|i| i.pointer.any_pressed()) {
        app.ui.focus = Focus::Playlist;
    }
    let picker = Rect::from_min_size(area.min, vec2(PICKER_W, area.height()));
    let body_rect = Rect::from_min_max(pos2(picker.right() + 1.0, area.top()), area.max);
    ui.painter()
        .line_segment([picker.right_top(), picker.right_bottom()], Stroke::new(1.0, t.border));
    let mut pick_ui = ui.new_child(egui::UiBuilder::new().max_rect(picker));
    picker_ui(app, &mut pick_ui);
    let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body_rect));
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
                    .filter(|c| c.pattern() == p.id)
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
                let on = id == cur;
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
                }
                if resp.double_clicked() {
                    app.show_tab(crate::app::DockTab::PianoRoll);
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
        });
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
        let Some(p) = app.project().pattern(c.pattern()) else {
            continue;
        };
        let track_muted = app
            .project()
            .playlist
            .tracks
            .get(c.track as usize)
            .is_some_and(|t| t.muted);
        let sel = app.ui.playlist.selection.contains(&c.id);
        let base = rgb(p.color);
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
            &p.name,
            crate::theme::semibold(10.5),
            fg,
        );
        painter
            .with_clip_rect(rr)
            .text(r.min + vec2(4.0, 0.5), Align2::LEFT_TOP, "▾", mono(10.0), fg);
        // Note preview.
        let body = Rect::from_min_max(pos2(r.left(), r.top() + 15.0), r.max);
        if body.height() > 6.0 && !p.notes.is_empty() {
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
    scroll_input(app, ui, full, grid);
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

fn hit(view: &View, clips: &[Clip], p: Pos2) -> Option<(Clip, bool, bool)> {
    clips.iter().rev().find_map(|c| {
        let r = view.clip_rect(c);
        r.contains(p).then(|| {
            let edge = p.x >= r.right() - EDGE.min(r.width() * 0.4);
            let menu = p.x <= r.left() + 13.0 && p.y <= r.top() + 13.0;
            (c.clone(), edge, menu)
        })
    })
}

fn grid_input(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, clips: &[Clip], tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("pl_grid"), Sense::click_and_drag());
    let sig = app.project().time_signature;
    let (mods, pointer) = ui.input(|i| (i.modifiers, i.pointer.interact_pos()));
    let snap = if mods.alt { Snap::None } else { app.ui.snap };
    if let Some(p) = resp.hover_pos() {
        match hit(view, clips, p) {
            Some((_, true, _)) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal),
            Some((_, _, true)) => ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand),
            Some(_) => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
            None => {}
        }
        let tk = view.tick_at(p.x).max(0) as u64;
        app.hint(format!(
            "{} — left: place/move/resize · right: delete · double-click: edit pattern · ▾: clip menu · Cmd+drag: select",
            BarBeatTick::from_ticks(tk, sig)
        ));
    }
    let primary = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
    let secondary =
        resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary));
    if primary && let Some(p) = pointer {
        let st = &mut app.ui.playlist;
        if mods.command {
            match hit(view, clips, p) {
                Some((c, _, _)) => {
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
                Some((c, _, true)) => {
                    st.menu = Some((c.id, p));
                }
                Some((c, edge, _)) => {
                    if !st.selection.contains(&c.id) {
                        st.selection = vec![c.id];
                    }
                    let ids = st.selection.clone();
                    app.set_current_pattern(c.pattern());
                    app.doc.begin_gesture();
                    app.ui.playlist.drag = Some(if edge {
                        Drag::Resize {
                            id: c.id,
                            start: c.start,
                        }
                    } else {
                        Drag::Move {
                            ids,
                            anchor_start: c.start,
                            grab_tick: view.tick_at(p.x),
                            grab_track: view.track_at(p.y),
                            applied_dt: 0,
                            applied_dtrack: 0,
                        }
                    });
                }
                None => {
                    let track = view.track_at(p.y);
                    if track >= 0 && (track as u32) < tracks {
                        let start = snap.floor(view.tick_at(p.x).max(0) as u64, sig);
                        let pattern = app.current_pattern();
                        app.doc.begin_gesture();
                        if let Some(id) = app.exec(Command::AddClip {
                            pattern,
                            track: track as u32,
                            start,
                            length: None,
                        }) {
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
            Some(Drag::Erase) => {
                let current = app.project().playlist.clips.clone();
                if let Some((c, _, _)) = hit(view, &current, p) {
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
        && let Some((c, _, false)) = hit(view, clips, p)
    {
        app.set_current_pattern(c.pattern());
        app.show_tab(crate::app::DockTab::PianoRoll);
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
        .filter(|c| c.pattern() == clip.pattern())
        .count();
    let mut close = false;
    let area = egui::Area::new(egui::Id::new("clip_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_min_width(170.0);
                let name = app
                    .project()
                    .pattern(clip.pattern())
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                ui.label(RichText::new(name).strong());
                ui.label(
                    RichText::new(format!("Used by {users} clip(s)"))
                        .small()
                        .color(Theme::current().text_dim),
                );
                ui.separator();
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
                    app.set_current_pattern(clip.pattern());
                    app.show_tab(crate::app::DockTab::PianoRoll);
                    close = true;
                }
                if ui.button("Rename pattern…").clicked() {
                    app.dialog = Some(crate::dialogs::Dialog::rename_pattern(app, clip.pattern()));
                    close = true;
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
                if ui.button("Fit to pattern length").clicked() {
                    let sig = app.project().time_signature;
                    if let Some(len) = app.project().pattern(clip.pattern()).map(|p| p.length(sig)) {
                        app.exec(Command::ResizeClip { clip: id, length: len });
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

fn scroll_input(app: &mut LimeShotApp, ui: &mut Ui, full: Rect, grid: Rect) {
    if !ui.rect_contains_pointer(full) {
        return;
    }
    let (delta, mods) = ui.input(|i| (i.smooth_scroll_delta, i.modifiers));
    if delta == Vec2::ZERO {
        return;
    }
    let st = &mut app.ui.playlist;
    if mods.command {
        let factor = (1.0 + delta.y * 0.004).clamp(0.5, 2.0);
        let anchor = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| p.x - grid.left())
            .unwrap_or(0.0);
        let old = st.zoom_x;
        st.zoom_x = (st.zoom_x * factor).clamp(3.0, 400.0);
        st.scroll.x = ((st.scroll.x + anchor) * st.zoom_x / old - anchor).max(0.0);
    } else if mods.shift {
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
