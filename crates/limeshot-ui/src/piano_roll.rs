//! The Piano Roll: edits the current pattern's notes for the selected channel.
//!
//! Mouse (FL Studio's Draw tool conventions):
//! * left-click empty space: add a note (snapped) and keep dragging to move it,
//! * left-drag a note: move it (and the rest of the selection); drag its right edge: resize,
//! * right-click / right-drag: delete notes under the pointer,
//! * Ctrl/Cmd+drag: rectangle selection; Ctrl/Cmd+click a note: toggle it in the selection,
//! * Alt while dragging: no snapping,
//! * wheel: scroll vertically, Shift+wheel: horizontally, Cmd+wheel: zoom time,
//! * keyboard on the left: click to audition; the lower lane edits velocity.

use crate::app::{Focus, LimeShotApp};
use crate::theme::{Theme, mix, mono, rgb};
use egui::{Align2, Color32, Context, CornerRadius, Key, Modifiers, PointerButton, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2, pos2, vec2};
use limeshot_core::*;

const KEYS_W: f32 = 52.0;
const RULER_H: f32 = 18.0;
const VEL_H: f32 = 72.0;
const EDGE: f32 = 6.0;

#[derive(Debug, Clone)]
enum Drag {
    Move { ids: Vec<NoteId>, anchor_start: Tick, anchor_key: u8, grab_tick: i64, grab_key: i32, applied_dt: i64, applied_dk: i32 },
    Resize { ids: Vec<NoteId>, anchor_end: Tick, grab_tick: i64, applied: i64 },
    Select { from: Pos2 },
    Erase,
    Velocity,
}

pub struct State {
    /// Pixels per beat.
    pub zoom_x: f32,
    pub key_h: f32,
    pub scroll: Vec2,
    pub selection: Vec<NoteId>,
    drag: Option<Drag>,
    pub last_len: Tick,
    pub last_vel: f32,
    audition: Option<(ChannelId, u8)>,
    scrolled_once: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom_x: 96.0,
            key_h: 14.0,
            scroll: Vec2::ZERO,
            selection: Vec::new(),
            drag: None,
            last_len: STEP_TICKS,
            last_vel: 0.78,
            audition: None,
            scrolled_once: false,
        }
    }
}

const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// "C5" for key 60 (FL Studio's octave numbering: middle C is C5).
pub fn key_name(k: u8) -> String {
    format!("{}{}", NAMES[usize::from(k % 12)], k / 12)
}

fn is_black(k: u8) -> bool {
    matches!(k % 12, 1 | 3 | 6 | 8 | 10)
}

struct View {
    grid: Rect,
    ppt: f32,
    key_h: f32,
    scroll: Vec2,
}

impl View {
    fn x(&self, t: Tick) -> f32 {
        self.grid.left() + t as f32 * self.ppt - self.scroll.x
    }
    fn tick_at(&self, x: f32) -> i64 {
        ((x - self.grid.left() + self.scroll.x) / self.ppt).floor() as i64
    }
    fn y(&self, k: u8) -> f32 {
        self.grid.top() + f32::from(127 - k) * self.key_h - self.scroll.y
    }
    fn key_at(&self, y: f32) -> i32 {
        127 - ((y - self.grid.top() + self.scroll.y) / self.key_h).floor() as i32
    }
    fn note_rect(&self, n: &Note) -> Rect {
        Rect::from_min_max(pos2(self.x(n.start), self.y(n.key)), pos2(self.x(n.end()).max(self.x(n.start) + 3.0), self.y(n.key) + self.key_h))
    }
}

pub fn window(app: &mut LimeShotApp, ctx: &Context, arrange: bool) {
    let mut open = app.ui.show_piano;
    let ws = app.ui.workspace;
    let mut w = egui::Window::new(RichText::new("Piano Roll").strong())
        .id(egui::Id::new("piano_roll"))
        .open(&mut open)
        .constrain_to(ws)
        .default_pos(ws.min + vec2(120.0, 120.0))
        .default_size(vec2(820.0, 480.0))
        .min_size(vec2(360.0, 240.0))
        .resizable(true);
    if arrange {
        w = w.current_pos(ws.min + vec2(120.0, 120.0));
    }
    let resp = w.show(ctx, |ui| body(app, ui));
    if let Some(r) = resp
        && r.response.contains_pointer()
        && ctx.input(|i| i.pointer.any_pressed())
    {
        app.ui.focus = Focus::PianoRoll;
    }
    app.ui.show_piano = open;
}

fn body(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let pid = app.current_pattern();
    let Some(cid) = app.selected_channel() else {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| ui.label(RichText::new("Add a channel in the Channel Rack to write notes.").color(t.text_dim)));
        return;
    };
    // Header: channel selector and zoom.
    ui.horizontal(|ui| {
        let chans: Vec<(ChannelId, String, Rgb)> = app.project().channels.iter().map(|c| (c.id, c.name.clone(), c.color)).collect();
        let cur_name = chans.iter().find(|c| c.0 == cid).map(|c| c.1.clone()).unwrap_or_default();
        egui::ComboBox::from_id_salt("pr_channel").selected_text(cur_name).width(140.0).show_ui(ui, |ui| {
            for (id, name, color) in &chans {
                if ui.selectable_label(*id == cid, RichText::new(format!("■ {name}")).color(rgb(*color))).clicked() {
                    app.ui.selected_channel = Some(*id);
                    app.ui.piano.selection.clear();
                }
            }
        });
        let pat_name = app.project().pattern(pid).map(|p| p.name.clone()).unwrap_or_default();
        ui.label(RichText::new(format!("in {pat_name}")).color(t.text_dim));
        ui.separator();
        ui.label(RichText::new(format!("Snap: {}", app.ui.snap.label())).color(t.text_dim));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("+").on_hover_text("Zoom in (Cmd+wheel)").clicked() {
                app.ui.piano.zoom_x = (app.ui.piano.zoom_x * 1.25).min(800.0);
            }
            if ui.small_button("−").on_hover_text("Zoom out").clicked() {
                app.ui.piano.zoom_x = (app.ui.piano.zoom_x / 1.25).max(12.0);
            }
            if ui.small_button("↕").on_hover_text("Taller keys").clicked() {
                app.ui.piano.key_h = if app.ui.piano.key_h >= 22.0 { 9.0 } else { app.ui.piano.key_h + 3.0 };
            }
            ui.label(RichText::new(format!("{} selected", app.ui.piano.selection.len())).small().color(t.text_dim));
        });
    });
    let full = ui.available_rect_before_wrap();
    let (full, _) = ui.allocate_exact_size(full.size(), Sense::hover());
    let ruler = Rect::from_min_max(pos2(full.left() + KEYS_W, full.top()), pos2(full.right(), full.top() + RULER_H));
    let keys = Rect::from_min_max(pos2(full.left(), full.top() + RULER_H), pos2(full.left() + KEYS_W, full.bottom() - VEL_H));
    let grid = Rect::from_min_max(pos2(full.left() + KEYS_W, full.top() + RULER_H), pos2(full.right(), full.bottom() - VEL_H));
    let vel = Rect::from_min_max(pos2(full.left() + KEYS_W, full.bottom() - VEL_H + 4.0), full.right_bottom());

    let st = &mut app.ui.piano;
    if !st.scrolled_once {
        st.scroll.y = (127.0 - 79.0) * st.key_h; // G6 at the top: C4–C6 visible
        st.scrolled_once = true;
    }
    let max_y = (128.0 * st.key_h - grid.height()).max(0.0);
    st.scroll.y = st.scroll.y.clamp(0.0, max_y);
    st.scroll.x = st.scroll.x.max(0.0);
    let view = View { grid, ppt: st.zoom_x / PPQ as f32, key_h: st.key_h, scroll: st.scroll };

    let sig = app.project().time_signature;
    let pat_len = app.project().pattern(pid).map(|p| p.length(sig)).unwrap_or(sig.ticks_per_bar());
    let painter = ui.painter_at(full);

    // Rows.
    painter.rect_filled(grid, 0.0, t.sunken);
    for k in 0..=127u8 {
        let y = view.y(k);
        if y > grid.bottom() || y + view.key_h < grid.top() {
            continue;
        }
        let row = Rect::from_min_max(pos2(grid.left(), y), pos2(grid.right(), y + view.key_h));
        if is_black(k) {
            painter.rect_filled(row.intersect(grid), 0.0, t.row_black_key);
        }
        if k % 12 == 0 {
            painter.line_segment([pos2(grid.left(), y + view.key_h), pos2(grid.right(), y + view.key_h)], Stroke::new(1.0, t.grid_beat));
        }
    }
    // Columns.
    draw_time_grid(&painter, &view, grid, sig, app.ui.snap);
    // Pattern end.
    let xe = view.x(pat_len);
    if xe < grid.right() {
        painter.rect_filled(Rect::from_min_max(pos2(xe.max(grid.left()), grid.top()), grid.right_bottom()), 0.0, Color32::from_black_alpha(70));
    }

    // Notes: other channels as ghosts, then this channel.
    let pattern = app.project().pattern(pid).cloned();
    let ch_color = app.project().channel(cid).map(|c| rgb(c.color)).unwrap_or(t.accent);
    if let Some(p) = &pattern {
        for n in p.notes.iter().filter(|n| n.channel != cid) {
            let r = view.note_rect(n);
            if r.intersects(grid) {
                painter.rect_filled(r.intersect(grid), CornerRadius::same(2), t.text_dim.gamma_multiply(0.18));
            }
        }
        for n in p.notes_for(cid) {
            let r = view.note_rect(n);
            if !r.intersects(grid) {
                continue;
            }
            let sel = app.ui.piano.selection.contains(&n.id);
            let base = mix(t.raised, ch_color, 0.45 + 0.55 * n.velocity);
            let fill = if sel { mix(base, Color32::WHITE, 0.35) } else { base };
            painter.rect_filled(r.intersect(grid), CornerRadius::same(2), fill);
            painter.rect_stroke(r, 2.0, Stroke::new(1.0, if sel { t.accent } else { mix(fill, Color32::BLACK, 0.4) }), egui::StrokeKind::Inside);
            if r.width() > 26.0 && view.key_h >= 11.0 {
                painter.text(r.left_center() + vec2(3.0, 0.0), Align2::LEFT_CENTER, key_name(n.key), mono(9.0), Color32::from_black_alpha(200));
            }
        }
    }
    // Selection rectangle.
    if let (Some(Drag::Select { from }), Some(p)) = (&app.ui.piano.drag, ui.input(|i| i.pointer.hover_pos())) {
        painter.rect(Rect::from_two_pos(*from, p), 0.0, t.selection, Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
    }
    // Playhead.
    if app.ui.mode == limeshot_engine::PlayMode::Pattern && app.is_playing() {
        let x = view.x(app.player.status().position());
        if x >= grid.left() && x <= grid.right() {
            painter.line_segment([pos2(x, ruler.top()), pos2(x, grid.bottom())], Stroke::new(1.5, t.playhead));
        }
    }

    ruler_ui(app, ui, &view, ruler, sig, pat_len);
    keyboard_ui(app, ui, &view, keys, cid);
    grid_input(app, ui, &view, grid, pid, cid, pattern.as_ref());
    velocity_lane(app, ui, &view, vel, pid, cid, pattern.as_ref(), ch_color);
    scroll_input(app, ui, full, grid);
}

pub fn draw_time_grid(painter: &egui::Painter, view: &impl TimeAxis, area: Rect, sig: TimeSignature, snap: Snap) {
    let t = Theme::current();
    let beat = sig.ticks_per_beat();
    let bar = sig.ticks_per_bar();
    let px_per_beat = view.ppt() * beat as f32;
    let step = if px_per_beat > 48.0 { STEP_TICKS } else if px_per_beat > 12.0 { beat } else { bar };
    let step = match snap.ticks(sig) {
        Some(s) if s < step && s as f32 * view.ppt() > 6.0 => s,
        _ => step,
    };
    let first = (view.tick_at_x(area.left()).max(0) as u64) / step * step;
    let mut tk = first;
    while view.x_of(tk) <= area.right() {
        let x = view.x_of(tk);
        let c = if tk % bar == 0 {
            t.grid_bar
        } else if tk % beat == 0 {
            t.grid_beat
        } else {
            t.grid_step
        };
        if x >= area.left() {
            painter.line_segment([pos2(x, area.top()), pos2(x, area.bottom())], Stroke::new(1.0, c));
        }
        tk += step;
    }
}

/// Shared horizontal mapping for the Piano Roll and Playlist grids.
pub trait TimeAxis {
    fn ppt(&self) -> f32;
    fn x_of(&self, t: Tick) -> f32;
    fn tick_at_x(&self, x: f32) -> i64;
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

pub fn draw_ruler(painter: &egui::Painter, view: &impl TimeAxis, ruler: Rect, sig: TimeSignature) {
    let t = Theme::current();
    painter.rect_filled(ruler, 0.0, t.surface);
    let bar = sig.ticks_per_bar();
    let px_bar = view.ppt() * bar as f32;
    let every = if px_bar > 40.0 { 1 } else if px_bar > 16.0 { 4 } else { 16 };
    let first_bar = (view.tick_at_x(ruler.left()).max(0) as u64) / bar;
    let mut b = first_bar;
    while view.x_of(b * bar) <= ruler.right() {
        let x = view.x_of(b * bar);
        if b % every == 0 && x >= ruler.left() - 1.0 {
            painter.line_segment([pos2(x, ruler.center().y), pos2(x, ruler.bottom())], Stroke::new(1.0, t.grid_bar));
            painter.text(pos2(x + 3.0, ruler.top() + 1.0), Align2::LEFT_TOP, (b + 1).to_string(), mono(10.0), t.text_dim);
        }
        b += 1;
    }
}

fn ruler_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, ruler: Rect, sig: TimeSignature, pat_len: Tick) {
    let painter = ui.painter_at(ruler);
    draw_ruler(&painter, view, ruler, sig);
    let t = Theme::current();
    let xe = view.x(pat_len);
    if xe >= ruler.left() && xe <= ruler.right() {
        painter.line_segment([pos2(xe, ruler.top()), pos2(xe, ruler.bottom())], Stroke::new(2.0, t.accent_dim));
    }
    let resp = ui.interact(ruler, ui.id().with("pr_ruler"), Sense::click());
    if resp.clicked()
        && let Some(p) = resp.interact_pointer_pos()
    {
        let tick = app.ui.snap.floor(view.tick_at(p.x).max(0) as u64, sig);
        app.set_mode(limeshot_engine::PlayMode::Pattern);
        app.locate(tick);
    }
    resp.on_hover_text("Click to move the playhead (Pattern mode)");
}

fn keyboard_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, keys: Rect, cid: ChannelId) {
    let t = Theme::current();
    let painter = ui.painter_at(keys);
    for k in 0..=127u8 {
        let y = view.y(k);
        if y > keys.bottom() || y + view.key_h < keys.top() {
            continue;
        }
        let r = Rect::from_min_max(pos2(keys.left(), y), pos2(keys.right(), y + view.key_h - 1.0));
        let pressed = app.ui.piano.audition == Some((cid, k));
        let fill = if pressed {
            t.accent
        } else if is_black(k) {
            t.key_black
        } else {
            t.key_white
        };
        let r = if is_black(k) { Rect::from_min_max(r.min, pos2(keys.left() + KEYS_W * 0.68, r.max.y)) } else { r };
        painter.rect_filled(r, CornerRadius::same(2), fill);
        if k % 12 == 0 {
            painter.text(pos2(keys.right() - 3.0, y + view.key_h / 2.0), Align2::RIGHT_CENTER, key_name(k), mono(9.0), t.key_black);
        }
    }
    let resp = ui.interact(keys, ui.id().with("pr_keys"), Sense::click_and_drag());
    let down = resp.is_pointer_button_down_on();
    let key = resp.interact_pointer_pos().map(|p| view.key_at(p.y).clamp(0, 127) as u8);
    match (down, key, app.ui.piano.audition) {
        (true, Some(k), prev) if prev != Some((cid, k)) => {
            if let Some((c, pk)) = prev {
                app.preview_off(c, pk);
            }
            app.preview_on(cid, k, app.ui.piano.last_vel);
            app.ui.piano.audition = Some((cid, k));
        }
        (false, _, Some((c, pk))) => {
            app.preview_off(c, pk);
            app.ui.piano.audition = None;
        }
        _ => {}
    }
    if resp.hovered() {
        app.hint("Keyboard: click to hear the channel");
    }
}

fn hit(view: &View, notes: &[Note], cid: ChannelId, p: Pos2) -> Option<(Note, bool)> {
    notes.iter().rev().filter(|n| n.channel == cid).find_map(|n| {
        let r = view.note_rect(n);
        r.contains(p).then(|| (*n, p.x >= r.right() - EDGE.min(r.width() * 0.4)))
    })
}

fn grid_input(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, pid: PatternId, cid: ChannelId, pattern: Option<&Pattern>) {
    let resp = ui.interact(grid, ui.id().with("pr_grid"), Sense::click_and_drag());
    let notes: &[Note] = pattern.map(|p| p.notes.as_slice()).unwrap_or(&[]);
    let sig = app.project().time_signature;
    let (mods, pointer) = ui.input(|i| (i.modifiers, i.pointer.interact_pos()));
    let snap = if mods.alt { Snap::None } else { app.ui.snap };
    let root = app.project().channel(cid).map(|c| c.root_key).unwrap_or(60);

    // Cursor feedback.
    if let Some(p) = resp.hover_pos() {
        match hit(view, notes, cid, p) {
            Some((_, true)) => ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal),
            Some(_) => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
            None => {}
        }
        let k = view.key_at(p.y).clamp(0, 127) as u8;
        let tk = view.tick_at(p.x).max(0) as u64;
        app.hint(format!(
            "{} · {} — left: add/move/resize · right: delete · Cmd+drag: select · Alt: no snap",
            key_name(k),
            BarBeatTick::from_ticks(tk, sig)
        ));
    }

    let primary_pressed = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
    let secondary_pressed = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary));

    if primary_pressed && let Some(p) = pointer {
        let st = &mut app.ui.piano;
        if mods.command {
            match hit(view, notes, cid, p) {
                Some((n, _)) => {
                    if let Some(i) = st.selection.iter().position(|x| *x == n.id) {
                        st.selection.remove(i);
                    } else {
                        st.selection.push(n.id);
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
            match hit(view, notes, cid, p) {
                Some((n, edge)) => {
                    if !st.selection.contains(&n.id) {
                        st.selection = vec![n.id];
                    }
                    let ids = st.selection.clone();
                    st.last_len = n.length;
                    st.last_vel = n.velocity;
                    app.doc.begin_gesture();
                    if edge {
                        app.ui.piano.drag = Some(Drag::Resize { ids, anchor_end: n.end(), grab_tick: view.tick_at(p.x), applied: 0 });
                    } else {
                        app.ui.piano.drag = Some(Drag::Move {
                            ids,
                            anchor_start: n.start,
                            anchor_key: n.key,
                            grab_tick: view.tick_at(p.x),
                            grab_key: view.key_at(p.y),
                            applied_dt: 0,
                            applied_dk: 0,
                        });
                        app.preview_on(cid, n.key, n.velocity);
                        app.ui.piano.audition = Some((cid, n.key));
                    }
                }
                None => {
                    let key = view.key_at(p.y).clamp(0, 127) as u8;
                    let start = snap.floor(view.tick_at(p.x).max(0) as u64, sig);
                    let note = NewNote { channel: cid, key, start, length: st.last_len.max(1), velocity: st.last_vel };
                    app.doc.begin_gesture();
                    if let Some(id) = app.exec(Command::AddNote { pattern: pid, note }) {
                        let id = NoteId(id);
                        app.ui.piano.selection = vec![id];
                        app.ui.piano.drag = Some(Drag::Move {
                            ids: vec![id],
                            anchor_start: start,
                            anchor_key: key,
                            grab_tick: view.tick_at(p.x),
                            grab_key: view.key_at(p.y),
                            applied_dt: 0,
                            applied_dk: 0,
                        });
                        app.preview_on(cid, key, note.velocity);
                        app.ui.piano.audition = Some((cid, key));
                    }
                }
            }
        }
    }
    if secondary_pressed {
        app.doc.begin_gesture();
        app.ui.piano.drag = Some(Drag::Erase);
    }

    // Continue the drag.
    if let Some(p) = pointer {
        let mut drag = app.ui.piano.drag.take();
        match &mut drag {
            Some(Drag::Move { ids, anchor_start, anchor_key, grab_tick, grab_key, applied_dt, applied_dk }) => {
                let raw = view.tick_at(p.x) - *grab_tick;
                let target = (*anchor_start as i64 + raw).max(0) as u64;
                let dt = snap.round(target, sig) as i64 - *anchor_start as i64;
                let dk = (view.key_at(p.y) - *grab_key).clamp(-i32::from(*anchor_key), 127 - i32::from(*anchor_key));
                if dt != *applied_dt || dk != *applied_dk {
                    app.exec(Command::MoveNotes { pattern: pid, notes: ids.clone(), dt: dt - *applied_dt, dkey: dk - *applied_dk });
                    if dk != *applied_dk {
                        let k = (i32::from(*anchor_key) + dk) as u8;
                        if let Some((c, old)) = app.ui.piano.audition {
                            app.preview_off(c, old);
                        }
                        app.preview_on(cid, k, app.ui.piano.last_vel);
                        app.ui.piano.audition = Some((cid, k));
                    }
                    *applied_dt = dt;
                    *applied_dk = dk;
                }
            }
            Some(Drag::Resize { ids, anchor_end, grab_tick, applied }) => {
                let raw = view.tick_at(p.x) - *grab_tick;
                let target = (*anchor_end as i64 + raw).max(1) as u64;
                let snapped = snap.round(target, sig).max(1);
                let d = snapped as i64 - *anchor_end as i64;
                if d != *applied {
                    app.exec(Command::ResizeNotes { pattern: pid, notes: ids.clone(), dlen: d - *applied });
                    *applied = d;
                    if let Some(n) = app.project().pattern(pid).and_then(|pp| pp.notes.iter().find(|n| ids.first() == Some(&n.id))) {
                        app.ui.piano.last_len = n.length;
                    }
                }
            }
            Some(Drag::Erase) => {
                let current = app.project().pattern(pid).map(|pp| pp.notes.clone()).unwrap_or_default();
                if let Some((n, _)) = hit(view, &current, cid, p) {
                    app.exec(Command::RemoveNotes { pattern: pid, notes: vec![n.id] });
                    app.ui.piano.selection.retain(|x| *x != n.id);
                }
            }
            Some(Drag::Select { from }) => {
                let r = Rect::from_two_pos(*from, p);
                for n in notes.iter().filter(|n| n.channel == cid) {
                    if view.note_rect(n).intersects(r) && !app.ui.piano.selection.contains(&n.id) {
                        app.ui.piano.selection.push(n.id);
                    }
                }
            }
            Some(Drag::Velocity) | None => {}
        }
        app.ui.piano.drag = drag;
    }

    // End of a drag.
    if app.ui.piano.drag.is_some() && !ui.input(|i| i.pointer.any_down()) {
        if !matches!(app.ui.piano.drag, Some(Drag::Select { .. })) {
            app.doc.end_gesture();
        }
        app.ui.piano.drag = None;
        if let Some((c, k)) = app.ui.piano.audition.take() {
            app.preview_off(c, k);
        }
    }
    // A plain click on the background (Cmd held) clears the selection when nothing was hit.
    if resp.double_clicked()
        && let Some(p) = pointer
        && let Some((n, _)) = hit(view, notes, cid, p)
    {
        // Double-click: select every note with the same pitch (quick way to edit a drum lane).
        app.ui.piano.selection = notes.iter().filter(|m| m.channel == cid && m.key == n.key).map(|m| m.id).collect();
    }
    let _ = root;
}

fn velocity_lane(app: &mut LimeShotApp, ui: &mut Ui, view: &View, lane: Rect, pid: PatternId, cid: ChannelId, pattern: Option<&Pattern>, color: Color32) {
    let t = Theme::current();
    let painter = ui.painter_at(lane);
    painter.rect_filled(lane, 0.0, t.sunken);
    painter.text(pos2(lane.left() - KEYS_W + 4.0, lane.top() + 2.0), Align2::LEFT_TOP, "Velocity", mono(9.5), t.text_dim);
    let Some(p) = pattern else { return };
    let h = lane.height() - 8.0;
    for n in p.notes_for(cid) {
        let x = view.x(n.start);
        if x < lane.left() - 4.0 || x > lane.right() + 4.0 {
            continue;
        }
        let y = lane.bottom() - 4.0 - n.velocity * h;
        let sel = app.ui.piano.selection.contains(&n.id);
        let c = if sel { t.accent } else { color };
        painter.line_segment([pos2(x + 1.0, lane.bottom()), pos2(x + 1.0, y)], Stroke::new(2.0, c));
        painter.circle_filled(pos2(x + 1.0, y), 3.0, c);
    }
    let resp = ui.interact(lane, ui.id().with("pr_vel"), Sense::click_and_drag());
    if resp.is_pointer_button_down_on()
        && let Some(pos) = resp.interact_pointer_pos()
    {
        if resp.drag_started() || ui.input(|i| i.pointer.any_pressed()) {
            app.doc.begin_gesture();
            app.ui.piano.drag = Some(Drag::Velocity);
        }
        let v = ((lane.bottom() - 4.0 - pos.y) / h).clamp(0.0, 1.0);
        // The note whose stem is closest to the pointer (within 8 px).
        let near = p
            .notes_for(cid)
            .map(|n| (n.id, (view.x(n.start) - pos.x).abs()))
            .filter(|(_, d)| *d < 8.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0);
        if let Some(id) = near {
            let ids = if app.ui.piano.selection.contains(&id) { app.ui.piano.selection.clone() } else { vec![id] };
            app.exec(Command::SetNoteVelocity { pattern: pid, notes: ids, velocity: v });
            app.ui.piano.last_vel = v;
        }
    }
    if resp.hovered() {
        app.hint("Velocity: click or drag a stem (applies to the selection when the note is selected)");
    }
}

fn scroll_input(app: &mut LimeShotApp, ui: &mut Ui, full: Rect, grid: Rect) {
    let hovered = ui.rect_contains_pointer(full);
    if !hovered {
        return;
    }
    let (delta, mods) = ui.input(|i| (i.smooth_scroll_delta, i.modifiers));
    if delta == Vec2::ZERO {
        return;
    }
    let st = &mut app.ui.piano;
    if mods.command {
        let factor = (1.0 + delta.y * 0.004).clamp(0.5, 2.0);
        let anchor = ui.input(|i| i.pointer.hover_pos()).map(|p| p.x - grid.left()).unwrap_or(0.0);
        let old = st.zoom_x;
        st.zoom_x = (st.zoom_x * factor).clamp(12.0, 800.0);
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
    let pid = app.current_pattern();
    let Some(cid) = app.selected_channel() else { return };
    let sel = app.ui.piano.selection.clone();
    if key(ctx, Modifiers::NONE, Key::Delete) || key(ctx, Modifiers::NONE, Key::Backspace) {
        if !sel.is_empty() {
            app.exec(Command::RemoveNotes { pattern: pid, notes: sel.clone() });
            app.ui.piano.selection.clear();
        }
    }
    if key(ctx, Modifiers::COMMAND, Key::A) {
        app.ui.piano.selection = app.project().pattern(pid).map(|p| p.notes_for(cid).map(|n| n.id).collect()).unwrap_or_default();
    }
    if key(ctx, Modifiers::COMMAND, Key::D) {
        app.ui.piano.selection.clear();
    }
    if key(ctx, Modifiers::COMMAND, Key::B) && !sel.is_empty() {
        let before: Vec<NoteId> = app.project().pattern(pid).map(|p| p.notes.iter().map(|n| n.id).collect()).unwrap_or_default();
        app.exec(Command::DuplicateNotes { pattern: pid, notes: sel.clone() });
        // Select the copies.
        app.ui.piano.selection = app.project().pattern(pid).map(|p| p.notes.iter().map(|n| n.id).filter(|id| !before.contains(id)).collect()).unwrap_or_default();
    }
    if sel.is_empty() {
        return;
    }
    let sig = app.project().time_signature;
    let grid = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS) as i64;
    for (mods, k, dt, dk) in [
        (Modifiers::NONE, Key::ArrowUp, 0, 1),
        (Modifiers::NONE, Key::ArrowDown, 0, -1),
        (Modifiers::COMMAND, Key::ArrowUp, 0, 12),
        (Modifiers::COMMAND, Key::ArrowDown, 0, -12),
        (Modifiers::NONE, Key::ArrowLeft, -grid, 0),
        (Modifiers::NONE, Key::ArrowRight, grid, 0),
    ] {
        if key(ctx, mods, k) {
            app.exec(Command::MoveNotes { pattern: pid, notes: sel.clone(), dt, dkey: dk });
        }
    }
}
