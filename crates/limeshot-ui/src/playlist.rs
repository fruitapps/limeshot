//! The Playlist: arranges pattern and audio clips on tracks; Song mode plays it.
//!
//! Mouse: left-click empty space places the current pattern (and keeps dragging to move it);
//! drag a clip to move it (across tracks too), drag its right edge to resize and its left edge to
//! trim the start; right-click/drag deletes; Cmd+drag selects; double-click a pattern clip opens
//! the Piano Roll, an audio clip its channel settings; the ▾ marker on a clip opens its menu.
//! Alt disables snapping. Click the ruler to move the song position. Samples dragged in from the
//! Browser become audio clips (FL Studio), shown as a ghost clip while dragged over the grid.
//! Shift+drag a clip moves a copy; right-click on empty space clears the selection.
//!
//! The clip picker has FL Studio's three tabs (patterns, audio clips, automation clips); the
//! Draw and Paint tools place the picked entry, with the length of the clip of that source last
//! placed, resized or clicked.
//!
//! Other tools (FL Studio's Playlist tool strip and keys, observed in FL Studio 2026.1.1): Paint (B)
//! lays clips end to end, Delete (D) and Mute (T) act on clips clicked or dragged over, Slip (S)
//! slides a clip's content inside it, Slice (C) cuts the clips a vertical drag crosses, Select (E),
//! Zoom (Z) and Playback (Y), which scrubs: a line follows the pointer and, while held, the notes
//! of the pattern clips under it sound. Each tool's icon follows the pointer over the grid.

use crate::app::ClipOp;
use crate::app::{Focus, LimeShotApp};
use crate::icons::Icon;
use crate::piano_roll::{TimeAxis, Tool, draw_ruler, draw_time_grid};
use crate::theme::{Theme, mix, mono, rgb};
use crate::widgets::Hint as _;
use crate::widgets::{icon_button, panel_header};
use egui::{
    Align2, Color32, Context, CornerRadius, Key, Modifiers, PointerButton, Pos2, Rect, RichText, Sense, Stroke, Ui,
    Vec2, pos2, vec2,
};
use limeshot_core::*;
use limeshot_engine::PlayMode;

const HEADER_W: f32 = 128.0;
const TOP_PREVIEW_H: f32 = 16.0;
/// Bar numbers in the top half, time markers in the bottom half.
const RULER_H: f32 = 32.0;
const COLLAPSED_TRACK_H: f32 = 16.0;
const EDGE: f32 = 7.0;

/// The three clip-focus tools in the top-left of the Playlist (FL Studio: Sample, Automation, Patterns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipFocus {
    #[default]
    Patterns,
    Sample,
    Automation,
}

#[derive(Debug, Clone)]
enum Drag {
    Move {
        ids: Vec<ClipId>,
        anchor_start: Tick,
        grab_tick: i64,
        grab_track: i32,
        applied_dt: i64,
        applied_dtrack: i32,
        /// Shift+drag: the first movement swaps `ids` for copies, which move instead.
        duplicate: bool,
    },
    Resize {
        id: ClipId,
        start: Tick,
        orig_len: Tick,
        orig_pitch: Option<f32>,
    },
    ResizeStart {
        id: ClipId,
    },
    /// Dragging a track header's bottom border to adjust its height (or collapse it).
    ResizeTrack {
        track: u32,
        grab_y: f32,
        orig_h: f32,
    },
    /// Dragging an audio clip's fade-in (or fade-out) handle.
    Fade {
        id: ClipId,
        out: bool,
        originals: Vec<Clip>,
    },
    FadeCurve {
        out: bool,
        originals: Vec<Clip>,
        grab_y: f32,
    },
    /// Dragging point `index` of an automation shown by `clip`.
    AutoPoint {
        clip: ClipId,
        automation: AutomationId,
        index: usize,
    },
    /// Step-editing points across an automation clip (`auto_step`).
    AutoStep {
        clip: ClipId,
        automation: AutomationId,
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
    /// Paint tool: the next clip on `track` goes at `next`.
    Paint {
        source: ClipSource,
        track: u32,
        next: Tick,
        len: Tick,
        offset: Tick,
    },
    /// Mute tool: clips crossed get `muted`.
    Mute {
        muted: bool,
        done: Vec<ClipId>,
    },
    /// Slip tool: the clip's content follows the pointer.
    Slip {
        id: ClipId,
        anchor: Tick,
        offset: Tick,
        grab_tick: i64,
    },
    Slice {
        from: Pos2,
    },
    Zoom {
        from: Pos2,
    },
    Playback,
}

pub struct State {
    pub tool: Tool,
    /// Screen rectangles of the tool strip's buttons in the last frame.
    pub tool_rects: Vec<(Tool, Rect)>,
    /// The top-left clip-focus tool (Sample, Automation, Patterns) and its sub-options.
    pub clip_focus: ClipFocus,
    pub focus_tab_rects: Vec<(ClipFocus, Rect)>,
    pub focus_option_rects: Vec<(&'static str, Rect)>,
    /// Sample sub-options: Show fade edit controls, Check for zero crossing, Stretch audio when resizing.
    pub audio_show_fades: bool,
    pub audio_zero_cross: bool,
    pub audio_stretch: bool,
    /// Automation sub-options: Step editing, Slide remaining points.
    pub auto_step: bool,
    pub auto_slide: bool,
    /// Pixels per beat.
    pub zoom_x: f32,
    pub track_h: f32,
    pub scroll: Vec2,
    pub selection: Vec<ClipId>,
    /// Selected Playlist track indices (Channel Rack–style selector bars).
    pub selected_tracks: Vec<u32>,
    /// Screen rectangles of the track selector bars in the last frame (for tests).
    pub track_leds: Vec<(u32, Rect)>,
    pub track_mute_rects: Vec<(u32, Rect)>,
    /// Screen rectangles of the track resize bottom borders in the last frame (for tests).
    pub track_resize_rects: Vec<(u32, Rect)>,
    /// Screen rectangles of track group fold buttons in the last frame (for tests).
    pub track_group_rects: Vec<(u32, Rect)>,
    /// Playback monitoring activity per track (`0.0..=1.0`) in the last frame.
    pub track_activity: Vec<f32>,
    /// Where the Arrangement dropdown button in the top bar was drawn in the last frame.
    pub arrangement_button: Option<Rect>,
    /// The time selection made with Ctrl/Cmd+drag on the ruler; Song mode loops it.
    pub time_sel: Option<(Tick, Tick)>,
    scrub_tick: Option<Tick>,
    scrub_audio: Vec<(ChannelId, u8)>,
    /// Ruler gestures: the time selection's anchor, or the marker being dragged (and whether it
    /// moved).
    sel_anchor: Option<Tick>,
    marker_drag: Option<(MarkerId, bool)>,
    menu_marker: Option<MarkerId>,
    /// Screen rectangles of the markers' flags in the last frame.
    pub marker_rects: Vec<(MarkerId, Rect)>,
    drag: Option<Drag>,
    menu: Option<(ClipId, Pos2, u64)>,
    /// The automation point menu (FL Studio's right-click on a point): automation, point index,
    /// where, and the frame it opened in.
    pub point_menu: Option<(AutomationId, usize, Pos2, u64)>,
    pub point_menu_curve: bool,
    /// Waveform overviews of audio clip samples, by sample source path (read by tests).
    pub peaks: std::collections::HashMap<String, std::sync::Arc<Peaks>>,
    /// Geometry of the last drawn frame.
    pub geometry: Option<Geometry>,
    /// The tool whose cursor was drawn over the grid in the last frame (for tests).
    pub cursor: Option<Tool>,
    /// The clip picker's tab (FL Studio: patterns, audio clips, automation clips).
    pub picker_tab: PickerTab,
    /// Screen rectangles of the picker's tabs and rows in the last frame (for tests).
    pub picker_tab_rects: Vec<(PickerTab, Rect)>,
    pub picker_rows: Vec<(ClipSource, Rect)>,
    /// The length the Draw tool gives the next clip of a source: that of the clip of that source
    /// last placed, resized or clicked (FL Studio).
    pub last_len: Option<(ClipSource, Tick)>,
    /// Where a sample dragged over the grid would land (track, start, length), shown as a ghost.
    pub drop_ghost: Option<(u32, Tick, Tick)>,
    /// The length of the sample being dragged, by path (decoded once per drag).
    drop_len: Option<(std::path::PathBuf, Option<Tick>)>,
    /// Whether an external/Browser drag has entered the Playlist grid during the current drag.
    dnd_entered_grid: bool,
    /// Where the ⏵ menu button was drawn in the last frame (for tests).
    pub menu_button: Option<Rect>,
}

/// The clip picker's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerTab {
    #[default]
    Patterns,
    Audio,
    Automation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tool: Tool::Draw,
            tool_rects: Vec::new(),
            clip_focus: ClipFocus::Patterns,
            focus_tab_rects: Vec::new(),
            focus_option_rects: Vec::new(),
            audio_show_fades: true,
            audio_zero_cross: false,
            audio_stretch: false,
            auto_step: false,
            auto_slide: false,
            zoom_x: 24.0,
            track_h: 44.0,
            scroll: Vec2::ZERO,
            selection: Vec::new(),
            selected_tracks: Vec::new(),
            track_leds: Vec::new(),
            track_mute_rects: Vec::new(),
            track_resize_rects: Vec::new(),
            track_group_rects: Vec::new(),
            track_activity: Vec::new(),
            arrangement_button: None,
            time_sel: None,
            scrub_tick: None,
            scrub_audio: Vec::new(),
            sel_anchor: None,
            marker_drag: None,
            menu_marker: None,
            marker_rects: Vec::new(),
            drag: None,
            menu: None,
            point_menu: None,
            point_menu_curve: false,
            peaks: Default::default(),
            geometry: None,
            cursor: None,
            picker_tab: PickerTab::Patterns,
            picker_tab_rects: Vec::new(),
            picker_rows: Vec::new(),
            last_len: None,
            drop_ghost: None,
            drop_len: None,
            dnd_entered_grid: false,
            menu_button: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TrackLayout {
    y: f32,
    h: f32,
    hidden: bool,
}

struct View {
    grid: Rect,
    ppt: f32,
    track_h: f32,
    scroll: Vec2,
    layouts: Vec<TrackLayout>,
    total_h: f32,
}

impl View {
    fn new(grid: Rect, ppt: f32, track_h: f32, scroll: Vec2, tracks: &[PlaylistTrack]) -> Self {
        let mut layouts = Vec::with_capacity(tracks.len());
        let mut y = 0.0;
        let mut group_hidden = false;
        for tr in tracks {
            if !tr.grouped_with_above {
                group_hidden = false;
            }
            let hidden = tr.grouped_with_above && group_hidden;
            let h = if hidden {
                0.0
            } else if tr.collapsed {
                COLLAPSED_TRACK_H
            } else {
                tr.height.unwrap_or(track_h).clamp(TRACK_H.0, TRACK_H.1)
            };
            layouts.push(TrackLayout { y, h, hidden });
            y += h;
            if !tr.grouped_with_above && tr.group_collapsed {
                group_hidden = true;
            }
        }
        Self {
            grid,
            ppt,
            track_h,
            scroll,
            layouts,
            total_h: y,
        }
    }
    fn x(&self, t: Tick) -> f32 {
        self.grid.left() + t as f32 * self.ppt - self.scroll.x
    }
    fn tick_at(&self, x: f32) -> i64 {
        ((x - self.grid.left() + self.scroll.x) / self.ppt).floor() as i64
    }
    fn y(&self, track: u32) -> f32 {
        let off = if let Some(l) = self.layouts.get(track as usize) {
            l.y
        } else {
            self.total_h + (track as usize).saturating_sub(self.layouts.len()) as f32 * self.track_h
        };
        self.grid.top() + off - self.scroll.y
    }
    fn track_height(&self, track: u32) -> f32 {
        self.layouts.get(track as usize).map_or(self.track_h, |l| l.h)
    }
    fn is_track_hidden(&self, track: u32) -> bool {
        self.layouts.get(track as usize).is_some_and(|l| l.hidden)
    }
    fn track_at(&self, y: f32) -> i32 {
        let rel = y - self.grid.top() + self.scroll.y;
        if rel < 0.0 {
            return (rel / self.track_h.max(1.0)).floor() as i32;
        }
        for (i, l) in self.layouts.iter().enumerate() {
            if !l.hidden && rel >= l.y && rel < l.y + l.h {
                return i as i32;
            }
        }
        if self.layouts.is_empty() {
            (rel / self.track_h.max(1.0)).floor() as i32
        } else {
            self.layouts.len() as i32 + ((rel - self.total_h).max(0.0) / self.track_h.max(1.0)).floor() as i32
        }
    }
    fn clip_rect(&self, c: &Clip) -> Rect {
        let h = self.track_height(c.track);
        Rect::from_min_max(
            pos2(self.x(c.start), self.y(c.track) + 1.0),
            pos2(
                self.x(c.end()).max(self.x(c.start) + 4.0),
                self.y(c.track) + (h - 1.0).max(1.0),
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
    layouts: [TrackLayout; 500],
    layout_count: usize,
    total_h: f32,
}

impl Geometry {
    /// Screen position of `tick` in the middle of `track`.
    pub fn pos(self, tick: Tick, track: u32) -> Pos2 {
        let (y, h) = if (track as usize) < self.layout_count {
            let l = self.layouts[track as usize];
            (l.y, l.h)
        } else {
            (
                self.total_h + (track as usize).saturating_sub(self.layout_count) as f32 * self.track_h,
                self.track_h,
            )
        };
        pos2(
            self.grid.left() + tick as f32 * self.ppt - self.scroll.x,
            self.grid.top() + y + 0.5 * h - self.scroll.y,
        )
    }

    /// Screen height of `track`.
    pub fn track_height(self, track: u32) -> f32 {
        if (track as usize) < self.layout_count {
            self.layouts[track as usize].h
        } else {
            self.track_h
        }
    }
}

const PICKER_W: f32 = 150.0;

/// The Playlist's wording of the tools' tooltips.
fn tool_tip(tool: Tool) -> &'static str {
    match tool {
        Tool::Draw => "Draw (P): place, move and resize clips",
        Tool::Paint => "Paint (B): drag to lay clips end to end",
        Tool::Delete => "Delete (D): click or drag over clips to delete them",
        Tool::Mute => "Mute (T): click or drag over clips to mute / unmute them",
        Tool::Slip => "Slip (S): drag a clip's content left or right inside the clip",
        Tool::Slice => "Slice (C): drag down across clips to cut them",
        Tool::Select => "Select (E): click or drag to select, drag a selected clip to move",
        Tool::Zoom => "Zoom (Z): drag a range to zoom in, right-click to zoom out",
        Tool::Playback => "Playback (Y): hold and drag to hear the pattern notes under the line",
    }
}

/// Body of the Playlist window (FL Studio opens it maximised): a tool strip, the pattern picker
/// on the left, and the arrangement.
pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let sig = app.project().time_signature;
    let bars = app.project().playlist.song_length(sig) / sig.ticks_per_bar();
    panel_header(ui, "", |ui| {
        if icon_button(ui, Icon::Plus, false, 22.0, "Zoom in (PgUp, Cmd+wheel)").clicked() {
            crate::playlist_menu::run(app, crate::playlist_menu::Act::ZoomIn);
        }
        if icon_button(ui, Icon::Minus, false, 22.0, "Zoom out (PgDn)").clicked() {
            crate::playlist_menu::run(app, crate::playlist_menu::Act::ZoomOut);
        }
        ui.label(RichText::new(format!("{bars} bars")).font(mono(10.5)).color(t.text_dim));
        // The tool strip on the left, as in FL Studio.
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let menu = ui.menu_button("⏵", |ui| crate::playlist_menu::menu(app, ui)).response;
            app.ui.playlist.menu_button = Some(menu.rect);
            menu.hint("Playlist options: Edit, Tools, View, Select, Zoom, Time markers, Arrangement");
            app.ui.playlist.tool_rects.clear();
            for tool in Tool::PLAYLIST {
                let resp = icon_button(ui, tool.icon(), app.ui.playlist.tool == tool, 22.0, tool_tip(tool));
                app.ui.playlist.tool_rects.push((tool, resp.rect));
                if resp.clicked() {
                    app.ui.playlist.tool = tool;
                }
            }
            ui.add_space(6.0);
            let arr_name = app.project().playlist.name.clone();
            let arr_resp = ui
                .menu_button(format!("{arr_name}    "), |ui| {
                    crate::playlist_menu::arrangement_menu(app, ui);
                })
                .response;
            let arrow = arr_resp.rect.right_center() - vec2(8.0, 0.0);
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    arrow + vec2(-3.0, -2.0),
                    arrow + vec2(3.0, -2.0),
                    arrow + vec2(0.0, 2.0),
                ],
                t.text_dim,
                Stroke::NONE,
            ));
            app.ui.playlist.arrangement_button = Some(arr_resp.rect);
            arr_resp.hint("Playlist arrangement: switch, add, clone, rename, merge or delete arrangements");
        });
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
    point_menu(app, &ctx);
}

/// An automation point's menu: Delete, then FL Studio's point modes for the segment after it.
fn point_menu(app: &mut LimeShotApp, ctx: &Context) {
    let Some((a, i, pos, opened)) = app.ui.playlist.point_menu else {
        return;
    };
    let Some(points) = app.project().automation(a).map(|x| x.points.clone()) else {
        app.ui.playlist.point_menu = None;
        return;
    };
    let Some(point) = points.get(i).copied() else {
        app.ui.playlist.point_menu = None;
        return;
    };
    let curve = app.ui.playlist.point_menu_curve;
    let mut close = false;
    let area = egui::Area::new(egui::Id::new("automation_point_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_min_width(150.0);
                if !curve
                    && ui
                        .add_enabled(
                            points.len() > 1,
                            egui::Button::new(RichText::new("Delete").color(Theme::current().danger)),
                        )
                        .clicked()
                {
                    let mut pts = points.clone();
                    pts.remove(i);
                    app.exec(Command::SetAutomationPoints {
                        automation: a,
                        points: pts,
                    });
                    close = true;
                }
                if !curve {
                    if ui.button("Copy value").clicked() {
                        app.control_clipboard = Some(point.value);
                        close = true;
                    }
                    if ui
                        .add_enabled(app.control_clipboard.is_some(), egui::Button::new("Paste value"))
                        .clicked()
                    {
                        if let Some(value) = app.control_clipboard {
                            let mut pts = points.clone();
                            pts[i].value = value.clamp(0.0, 1.0);
                            app.exec(Command::SetAutomationPoints {
                                automation: a,
                                points: pts,
                            });
                        }
                        close = true;
                    }
                    if ui.button("Type in value…").clicked() {
                        app.dialog = Some(crate::dialogs::Dialog::AutomationPointValue {
                            automation: a,
                            tick: point.tick,
                            text: point.value.to_string(),
                        });
                        close = true;
                    }
                } else if ui.button("Reset tension").clicked() {
                    let mut pts = points.clone();
                    pts[i].tension = 0.0;
                    app.exec(Command::SetAutomationPoints {
                        automation: a,
                        points: pts,
                    });
                    close = true;
                }
                if curve {
                    ui.separator();
                    for mode in PointMode::ALL {
                        if ui
                            .add(egui::Button::selectable(point.mode == mode, mode.label()))
                            .clicked()
                        {
                            let mut pts = points.clone();
                            if let Some(p) = pts.get_mut(i) {
                                p.mode = mode;
                                // A counting mode starts at a few steps rather than the curve's bend.
                                if mode.counts() && !point.mode.counts() {
                                    p.tension = -1.0 + 2.0 * 3.0 / 15.0;
                                }
                            }
                            app.exec(Command::SetAutomationPoints {
                                automation: a,
                                points: pts,
                            });
                            close = true;
                        }
                    }
                }
            });
        });
    let fresh = ctx.cumulative_pass_nr() == opened;
    let clicked_outside = !fresh && ctx.input(|i| i.pointer.any_pressed()) && !area.response.contains_pointer();
    if close || clicked_outside || ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.ui.playlist.point_menu = None;
    }
}

/// FL Studio's clip picker, with its three tabs: the project's patterns, audio clips and
/// automation clips. Click an entry to choose what the Playlist's Draw and Paint tools place;
/// double-click a pattern to edit it in the Piano Roll (an audio clip: its channel settings).
/// Samples dropped on the picker become audio clips without being placed.
fn picker_ui(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let area = ui.max_rect();
    app.ui.playlist.picker_tab_rects.clear();
    app.ui.playlist.picker_rows.clear();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.x = 2.0;
        for (tab, icon, tip) in [
            (PickerTab::Patterns, Icon::PianoRoll, "Show the patterns"),
            (PickerTab::Audio, Icon::Wave, "Show the audio clips"),
            (PickerTab::Automation, Icon::Automation, "Show the automation clips"),
        ] {
            let resp = icon_button(ui, icon, app.ui.playlist.picker_tab == tab, 22.0, tip);
            app.ui.playlist.picker_tab_rects.push((tab, resp.rect));
            if resp.clicked() {
                app.ui.playlist.picker_tab = tab;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            match app.ui.playlist.picker_tab {
                PickerTab::Patterns => {
                    if icon_button(ui, Icon::Plus, false, 20.0, "New pattern (F4)").clicked() {
                        crate::menus::add_pattern(app);
                    }
                }
                PickerTab::Audio => {
                    if icon_button(ui, Icon::Plus, false, 20.0, "Add a sample as an audio clip").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio files", &limeshot_dsp::AUDIO_EXTENSIONS)
                            .pick_file()
                    {
                        add_audio_source(app, &path);
                    }
                }
                PickerTab::Automation => {}
            }
        });
    });
    ui.add_space(2.0);
    let rows = picker_entries(app, app.ui.playlist.picker_tab);
    egui::ScrollArea::vertical()
        .id_salt("picker")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if rows.is_empty() {
                let empty = match app.ui.playlist.picker_tab {
                    PickerTab::Patterns => "No patterns",
                    PickerTab::Audio => "Drop samples here or on the Playlist",
                    PickerTab::Automation => "Right-click a control: Create automation clip",
                };
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new(empty).small().color(t.text_faint));
                });
            }
            for (source, name, color, uses) in rows {
                picker_row(app, ui, source, &name, color, uses);
            }
        });
    // Samples dropped on the picker join the Audio tab.
    let resp = ui.interact(area, ui.id().with("picker_drop"), Sense::hover());
    if resp.dnd_hover_payload::<crate::browser::DraggedSample>().is_some() {
        app.hint("Drop to add the sample to the audio clips");
    }
    if let Some(payload) = resp.dnd_release_payload::<crate::browser::DraggedSample>() {
        add_audio_source(app, &payload.0);
    }
}

/// The picker's entries on `tab`: (source, name, colour, clips using it).
fn picker_entries(app: &LimeShotApp, tab: PickerTab) -> Vec<(ClipSource, String, Rgb, usize)> {
    let p = app.project();
    let uses = |s: ClipSource| p.playlist.clips.iter().filter(|c| c.source == s).count();
    match tab {
        PickerTab::Patterns => p
            .patterns
            .iter()
            .map(|x| {
                (
                    ClipSource::Pattern(x.id),
                    x.name.clone(),
                    x.color,
                    uses(ClipSource::Pattern(x.id)),
                )
            })
            .collect(),
        PickerTab::Audio => audio_channels(p)
            .into_iter()
            .filter_map(|id| p.channel(id))
            .map(|c| {
                (
                    ClipSource::Audio(c.id),
                    c.name.clone(),
                    c.color,
                    uses(ClipSource::Audio(c.id)),
                )
            })
            .collect(),
        PickerTab::Automation => p
            .automations
            .iter()
            .map(|a| {
                let s = ClipSource::Automation(a.id);
                (s, a.name.clone(), a.color, uses(s))
            })
            .collect(),
    }
}

/// Every loaded sampler is also available as an audio source, including Channel Rack instruments.
pub fn audio_channels(p: &Project) -> Vec<ChannelId> {
    p.channels
        .iter()
        .filter(|c| {
            limeshot_core::lime::sampler_params(&c.instrument).is_some_and(|s| {
                s.sample.is_some() || matches!(&c.instrument,Instrument::Plugin(p) if !p.state.is_empty())
            })
        })
        .map(|c| c.id)
        .collect()
}

/// Draw a draggable clip-source row with its Playlist use count and source actions.
/// Clicking selects what to place and previews audio sources. Double-click opens the Piano Roll
/// for patterns or Channel Settings for audio.
fn picker_row(app: &mut LimeShotApp, ui: &mut Ui, source: ClipSource, name: &str, color: Rgb, uses: usize) {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click_and_drag());
    if resp.drag_started() {
        resp.dnd_set_drag_payload(source);
    }
    app.ui.playlist.picker_rows.push((source, r));
    let on = place_source(app) == source;
    if on {
        ui.painter().rect_filled(r, 0.0, mix(t.surface, rgb(color), 0.18));
    } else if resp.hovered() {
        ui.painter().rect_filled(r, 0.0, t.raised);
    }
    match source {
        ClipSource::Pattern(_) => {
            ui.painter().rect_filled(
                Rect::from_min_size(r.min + vec2(10.0, 8.0), vec2(10.0, 10.0)),
                CornerRadius::same(2),
                rgb(color),
            );
        }
        ClipSource::Audio(_) => {
            let icon = Rect::from_min_size(r.min + vec2(8.0, 6.0), vec2(14.0, 14.0));
            crate::icons::paint(ui.painter(), icon, Icon::Wave, rgb(color));
        }
        ClipSource::Automation(_) => {
            let icon = Rect::from_min_size(r.min + vec2(8.0, 6.0), vec2(14.0, 14.0));
            crate::icons::paint(ui.painter(), icon, Icon::Automation, rgb(color));
        }
    }
    ui.painter().with_clip_rect(r).text(
        r.left_center() + vec2(28.0, 0.0),
        Align2::LEFT_CENTER,
        name,
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
        pick_source(app, source);
        if let ClipSource::Audio(ch) = source {
            preview_audio_source(app, ch);
        }
    }
    if resp.double_clicked() {
        match source {
            ClipSource::Pattern(_) => app.show_window(crate::workspace::Win::PianoRoll),
            ClipSource::Audio(ch) => {
                app.select_channel(ch);
                app.plugins.open_window(crate::plugins::Target::Channel(ch), None);
            }
            ClipSource::Automation(_) => {}
        }
    }
    let tip = match source {
        ClipSource::Pattern(_) => "Click: place this pattern · double-click: edit in the Piano Roll",
        ClipSource::Audio(_) => "Click: place this audio clip (and hear it) · double-click: channel settings",
        ClipSource::Automation(_) => "Click: place this automation clip",
    };
    if resp.hovered() {
        app.hint(format!("{name} — {tip}"));
    }
    resp.hint(tip).context_menu(|ui| match source {
        ClipSource::Pattern(id) => {
            if ui.button("Rename…").clicked() {
                app.dialog = Some(crate::dialogs::Dialog::rename_pattern(app, id));
                ui.close();
            }
            pattern_color_menu(app, ui, id);
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
        }
        ClipSource::Audio(ch) => {
            if ui.button("Channel settings").clicked() {
                app.select_channel(ch);
                app.show_window(crate::workspace::Win::ChannelSettings);
                ui.close();
            }
            if ui
                .button("Delete")
                .hint("Delete the channel and its audio clips")
                .clicked()
            {
                app.exec(Command::RemoveChannel(ch));
                ui.close();
            }
        }
        ClipSource::Automation(id) => {
            if ui.button("Rename…").clicked() {
                app.dialog = Some(crate::dialogs::Dialog::rename_automation(app, id));
                ui.close();
            }
            if ui.button("Delete").clicked() {
                app.exec(Command::RemoveAutomation(id));
                ui.close();
            }
        }
    });
}

/// What a click on empty Playlist space places: the picked audio or automation clip, otherwise
/// the current pattern.
pub fn place_source(app: &LimeShotApp) -> ClipSource {
    match app.ui.place {
        Some(s @ ClipSource::Audio(ch)) if app.project().channel(ch).is_some() => s,
        Some(s @ ClipSource::Automation(a)) if app.project().automation(a).is_some() => s,
        _ => ClipSource::Pattern(app.current_pattern()),
    }
}

/// Make `source` what the Playlist places (and the picker shows): a pattern becomes the current
/// pattern, an audio clip's channel the selected channel.
pub fn pick_source(app: &mut LimeShotApp, source: ClipSource) {
    match source {
        ClipSource::Pattern(p) => {
            app.set_current_pattern(p);
            app.ui.place = None;
            app.ui.playlist.picker_tab = PickerTab::Patterns;
            app.ui.playlist.clip_focus = ClipFocus::Patterns;
        }
        ClipSource::Audio(ch) => {
            app.select_channel(ch);
            app.ui.place = Some(source);
            app.ui.playlist.picker_tab = PickerTab::Audio;
            app.ui.playlist.clip_focus = ClipFocus::Sample;
        }
        ClipSource::Automation(_) => {
            app.ui.place = Some(source);
            app.ui.playlist.picker_tab = PickerTab::Automation;
            app.ui.playlist.clip_focus = ClipFocus::Automation;
        }
    }
}

/// Play an audio clip channel's sample, as the Browser previews files.
fn preview_audio_source(app: &mut LimeShotApp, ch: ChannelId) {
    if let Some(path) = app.sampler_source(ch) {
        crate::browser::preview(app, &path);
    }
}

/// Add the sample at `path` to the picker's audio clips without placing it: a sampler channel
/// with the sample loaded, filed under "Audio" (reused when the sample is already there).
fn existing_audio_source(app: &LimeShotApp, path: &std::path::Path) -> Option<ChannelId> {
    audio_channels(app.project()).into_iter().find(|id| {
        app.project()
            .channel(*id)
            .and_then(|c| limeshot_core::lime::sampler_params(&c.instrument))
            .is_some_and(|s| s.sample.is_some_and(|r| app.samples.resolve(&r.path) == path))
    })
}

fn audio_source_commands(
    app: &LimeShotApp,
    path: &std::path::Path,
    sample: &limeshot_dsp::SampleData,
) -> (ChannelId, Vec<Command>) {
    if let Some(channel) = existing_audio_source(app, path) {
        return (channel, Vec::new());
    }
    let channel = ChannelId(app.project().next_id);
    let params = SamplerParams {
        sample: Some(SampleRef {
            path: path.to_string_lossy().into_owned(),
        }),
        one_shot: false,
        ..Default::default()
    };
    let plugin = crate::browser::sampler_with_audio(&params, 60, sample);
    (
        channel,
        vec![
            Command::AddChannel {
                name: crate::browser::file_label(path),
                instrument: Instrument::Plugin(plugin),
            },
            Command::SetChannelGroup {
                channels: vec![channel],
                group: Some("Audio".into()),
            },
        ],
    )
}

pub fn add_audio_source(app: &mut LimeShotApp, path: &std::path::Path) -> Option<ChannelId> {
    let ch = if let Some(ch) = existing_audio_source(app, path) {
        ch
    } else {
        let sample = crate::browser::import_audio(app, path, crate::browser::ImportTarget::Picker)?;
        let (ch, commands) = audio_source_commands(app, path, &sample);
        app.exec(Command::Batch(commands));
        app.project().channel(ch)?;
        ch
    };
    pick_source(app, ClipSource::Audio(ch));
    Some(ch)
}

/// A scrollbar controls the same pixel offset as wheel navigation.
fn scrollbar(ui: &mut Ui, name: &str, rect: Rect, offset: &mut f32, visible: f32, total: f32, vertical: bool) {
    let t = Theme::current();
    let axis = |p: Pos2| if vertical { p.y } else { p.x };
    let length = if vertical { rect.height() } else { rect.width() };
    let thumb_len = (length * visible / total.max(visible)).clamp(18.0_f32.min(length), length);
    let max = (total - visible).max(0.0);
    *offset = offset.clamp(0.0, max);
    let travel = (length - thumb_len).max(0.0);
    let start = axis(rect.min) + if max > 0.0 { *offset / max * travel } else { 0.0 };
    let thumb = if vertical {
        Rect::from_min_size(pos2(rect.left(), start), vec2(rect.width(), thumb_len))
    } else {
        Rect::from_min_size(pos2(start, rect.top()), vec2(thumb_len, rect.height()))
    };
    let response = ui.interact(rect, ui.id().with(name), Sense::click_and_drag());
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(name), rect));
    if (response.drag_started() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = if thumb.contains(pointer) {
            axis(pointer) - start
        } else {
            thumb_len / 2.0
        };
        ui.ctx().data_mut(|d| d.insert_temp(response.id.with("grab"), grab));
    }
    if (response.dragged() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(response.id.with("grab")))
            .unwrap_or(thumb_len / 2.0);
        *offset = ((axis(pointer) - axis(rect.min) - grab) / travel.max(1.0)).clamp(0.0, 1.0) * max;
    }
    ui.painter().rect_filled(rect, 0.0, t.sunken);
    ui.painter().rect_filled(
        thumb.shrink(2.0),
        3.0,
        if response.hovered() || response.dragged() {
            t.text_dim
        } else {
            t.border
        },
    );
}

/// FL Studio's translucent track preview bar at the top of the Playlist timeline.
fn preview_scrollbar(ui: &mut Ui, project: &Project, rect: Rect, offset: &mut f32, visible: f32, total: f32, ppt: f32) {
    let t = Theme::current();
    let length = rect.width();
    let thumb_len = (length * visible / total.max(visible)).clamp(18.0_f32.min(length), length);
    let max = (total - visible).max(0.0);
    *offset = offset.clamp(0.0, max);
    let travel = (length - thumb_len).max(0.0);
    let start = rect.left() + if max > 0.0 { *offset / max * travel } else { 0.0 };
    let thumb = Rect::from_min_size(pos2(start, rect.top()), vec2(thumb_len, rect.height()));
    let response = ui.interact(rect, ui.id().with("playlist-horizontal"), Sense::click_and_drag());
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new("playlist-horizontal"), rect));
    if (response.drag_started() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = if thumb.contains(pointer) {
            pointer.x - start
        } else {
            thumb_len / 2.0
        };
        ui.ctx().data_mut(|d| d.insert_temp(response.id.with("grab"), grab));
    }
    if (response.dragged() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let grab = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(response.id.with("grab")))
            .unwrap_or(thumb_len / 2.0);
        *offset = ((pointer.x - rect.left() - grab) / travel.max(1.0)).clamp(0.0, 1.0) * max;
    }
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, t.sunken);
    // Mini clip bars across all tracks.
    let track_count = project.playlist.tracks.len().max(1) as f32;
    let total_px = total.max(1.0);
    for c in &project.playlist.clips {
        let Some((_, color)) = clip_label(project, c) else {
            continue;
        };
        let x0 = rect.left() + (c.start as f32 * ppt) / total_px * length;
        let x1 = (rect.left() + (c.end() as f32 * ppt) / total_px * length).max(x0 + 1.5);
        let y0 = rect.top() + 1.0 + (c.track as f32 / track_count) * (rect.height() - 2.0);
        let y1 = (y0 + ((rect.height() - 2.0) / track_count).clamp(1.5, 4.0)).min(rect.bottom() - 1.0);
        painter.rect_filled(
            Rect::from_min_max(pos2(x0, y0), pos2(x1.min(rect.right()), y1)),
            0.5,
            rgb(color).gamma_multiply(if c.muted { 0.4 } else { 0.85 }),
        );
    }
    // Translucent viewport box over the preview bar.
    let active = response.hovered() || response.dragged();
    painter.rect_filled(
        thumb.shrink(1.0),
        CornerRadius::same(2),
        t.text.gamma_multiply(if active { 0.24 } else { 0.14 }),
    );
    painter.rect_stroke(
        thumb.shrink(1.0),
        CornerRadius::same(2),
        Stroke::new(1.0, if active { t.accent } else { t.text_dim }),
        egui::StrokeKind::Inside,
    );
}

/// FL Studio's three clip-focus tools in the top-left corner of the Playlist (Sample, Automation,
/// Patterns) and their sub-option toggles below them.
fn top_left_tools_ui(app: &mut LimeShotApp, ui: &mut Ui, rect: Rect) {
    let t = Theme::current();
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, t.surface);
    painter.line_segment([rect.right_top(), rect.right_bottom()], Stroke::new(1.0, t.border));
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.border));
    app.ui.playlist.focus_tab_rects.clear();
    app.ui.playlist.focus_option_rects.clear();
    let top_h = 22.0;
    let tabs = [
        (
            ClipFocus::Sample,
            Icon::Wave,
            "Audio clips focus: show fade, zero-crossing and stretch options",
        ),
        (
            ClipFocus::Automation,
            Icon::Automation,
            "Automation clips focus: show step editing and slide options",
        ),
        (ClipFocus::Patterns, Icon::PianoRoll, "Pattern clips focus"),
    ];
    let tab_w = (rect.width() - 8.0) / 3.0;
    for (i, (focus, icon, tip)) in tabs.into_iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(rect.left() + 4.0 + i as f32 * tab_w, rect.top() + 2.0),
            vec2(tab_w - 2.0, top_h - 2.0),
        );
        let resp = ui.interact(r, ui.id().with(("pl_focus_tab", i)), Sense::click());
        let on = app.ui.playlist.clip_focus == focus;
        let fill = if on {
            mix(t.raised, t.accent, 0.28)
        } else if resp.hovered() {
            t.raised_hover
        } else {
            t.raised
        };
        ui.painter().rect_filled(r, CornerRadius::same(2), fill);
        if on {
            ui.painter().rect_stroke(
                r,
                CornerRadius::same(2),
                Stroke::new(1.0, t.accent),
                egui::StrokeKind::Inside,
            );
        }
        let icon_r = Rect::from_center_size(r.center(), vec2(13.0, 13.0));
        crate::icons::paint(ui.painter(), icon_r, icon, if on { t.accent } else { t.text_dim });
        app.ui.playlist.focus_tab_rects.push((focus, r));
        if resp.clicked() {
            app.ui.playlist.clip_focus = focus;
            app.ui.playlist.picker_tab = match focus {
                ClipFocus::Sample => PickerTab::Audio,
                ClipFocus::Automation => PickerTab::Automation,
                ClipFocus::Patterns => PickerTab::Patterns,
            };
        }
        if resp.hovered() {
            app.hint(tip);
        }
        resp.hint(tip);
    }

    let sub_top = rect.top() + top_h + 2.0;
    let sub_h = (rect.bottom() - sub_top - 2.0).max(16.0);
    let toggle = |app: &mut LimeShotApp,
                  i: usize,
                  count: usize,
                  key: &'static str,
                  short: &str,
                  on: &mut bool,
                  tip: &'static str| {
        let w = (rect.width() - 8.0) / count as f32;
        let r = Rect::from_min_size(pos2(rect.left() + 4.0 + i as f32 * w, sub_top), vec2(w - 2.0, sub_h));
        let resp = ui.interact(r, ui.id().with(("pl_focus_opt", key)), Sense::click());
        let fill = if *on {
            mix(t.raised, t.accent, 0.28)
        } else if resp.hovered() {
            t.raised_hover
        } else {
            t.sunken
        };
        ui.painter().rect_filled(r, CornerRadius::same(2), fill);
        if *on {
            ui.painter().rect_stroke(
                r,
                CornerRadius::same(2),
                Stroke::new(1.0, t.accent),
                egui::StrokeKind::Inside,
            );
        }
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            short,
            mono(9.0),
            if *on { t.text } else { t.text_dim },
        );
        app.ui.playlist.focus_option_rects.push((key, r));
        if resp.clicked() {
            *on = !*on;
        }
        if resp.hovered() {
            app.hint(tip);
        }
        resp.hint(tip);
    };
    match app.ui.playlist.clip_focus {
        ClipFocus::Sample => {
            let mut fades = app.ui.playlist.audio_show_fades;
            let mut zcross = app.ui.playlist.audio_zero_cross;
            let mut stretch = app.ui.playlist.audio_stretch;
            toggle(
                app,
                0,
                3,
                "Show fade edit controls",
                "FADE",
                &mut fades,
                "Show fade edit controls",
            );
            toggle(
                app,
                1,
                3,
                "Check for zero crossing when editing audio clips",
                "Z-CROSS",
                &mut zcross,
                "Check for zero crossing when editing audio clips",
            );
            toggle(
                app,
                2,
                3,
                "Stretch audio when resizing audio clips",
                "STRETCH",
                &mut stretch,
                "Stretch by resampling (changes pitch and all clips sharing this sampler)",
            );
            app.ui.playlist.audio_show_fades = fades;
            app.ui.playlist.audio_zero_cross = zcross;
            app.ui.playlist.audio_stretch = stretch;
        }
        ClipFocus::Automation => {
            let mut step = app.ui.playlist.auto_step;
            let mut slide = app.ui.playlist.auto_slide;
            toggle(app, 0, 2, "Step editing", "STEP", &mut step, "Step editing");
            toggle(
                app,
                1,
                2,
                "Slide remaining points",
                "SLIDE",
                &mut slide,
                "Slide remaining points",
            );
            app.ui.playlist.auto_step = step;
            app.ui.playlist.auto_slide = slide;
        }
        ClipFocus::Patterns => {}
    }
}

/// Scroll the Playlist viewport when dragging clips or samples near/outside the grid edges.
fn auto_scroll_on_drag(st: &mut State, ctx: &Context, grid: Rect, max_y: f32) {
    let dragging_clip = matches!(
        st.drag,
        Some(
            Drag::Move { .. }
                | Drag::Resize { .. }
                | Drag::ResizeStart { .. }
                | Drag::Paint { .. }
                | Drag::Select { .. }
                | Drag::AutoPoint { .. }
                | Drag::AutoStep { .. }
                | Drag::Slip { .. }
        )
    );
    if !ui_has_active_drag(ctx, dragging_clip, &mut st.dnd_entered_grid, grid) {
        return;
    }
    let Some(pos) = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) else {
        return;
    };
    let margin = 24.0;
    let speed = 0.6;
    let mut dx = 0.0;
    let mut dy = 0.0;
    if pos.x > grid.right() - margin {
        dx = ((pos.x - (grid.right() - margin)) * speed).clamp(2.0, 36.0);
    } else if pos.x < grid.left() + margin && st.scroll.x > 0.0 {
        dx = -(((grid.left() + margin) - pos.x) * speed).clamp(2.0, 36.0);
    }
    if pos.y > grid.bottom() - margin && st.scroll.y < max_y {
        dy = ((pos.y - (grid.bottom() - margin)) * speed).clamp(2.0, 28.0);
    } else if pos.y < grid.top() + margin && st.scroll.y > 0.0 {
        dy = -(((grid.top() + margin) - pos.y) * speed).clamp(2.0, 28.0);
    }
    if dx != 0.0 || dy != 0.0 {
        st.scroll.x = (st.scroll.x + dx).max(0.0);
        st.scroll.y = (st.scroll.y + dy).clamp(0.0, max_y);
        ctx.request_repaint();
    }
}

fn ui_has_active_drag(ctx: &Context, dragging_clip: bool, dnd_entered: &mut bool, grid: Rect) -> bool {
    if !ctx.input(|i| i.pointer.any_down()) {
        *dnd_entered = false;
        return false;
    }
    if dragging_clip {
        return true;
    }
    let has_dnd = egui::DragAndDrop::has_payload_of_type::<crate::browser::DraggedSample>(ctx)
        || egui::DragAndDrop::has_payload_of_type::<ClipSource>(ctx);
    if !has_dnd {
        *dnd_entered = false;
        return false;
    }
    if ctx
        .input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()))
        .is_some_and(|p| grid.contains(p))
    {
        *dnd_entered = true;
    }
    *dnd_entered
}

/// Update `st.track_activity` for tracks that have an unmuted clip sounding at the current Song
/// playhead or scrub tick.
fn update_track_activity(app: &mut LimeShotApp) {
    let tracks_len = app.project().playlist.tracks.len();
    let playing_song = app.is_playing() && app.ui.mode == PlayMode::Song;
    let active_tick = if playing_song {
        Some(app.player.status().position())
    } else {
        app.ui.playlist.scrub_tick
    };
    let sig = app.project().time_signature;
    let mut active = vec![false; tracks_len];
    if let Some(tick) = active_tick {
        let p = app.project();
        for c in p
            .playlist
            .clips
            .iter()
            .filter(|c| !c.muted && c.start <= tick && tick < c.end())
        {
            let tr = c.track as usize;
            if tr >= tracks_len || p.playlist.tracks[tr].muted {
                continue;
            }
            let sounds = match c.source {
                ClipSource::Pattern(id) => p.pattern(id).is_some_and(|pat| {
                    let local = tick - c.start + c.offset;
                    local < pat.length(sig)
                        && pat.notes.iter().any(|n| {
                            !n.muted
                                && n.start <= local
                                && local < n.end()
                                && p.channel(n.channel).is_some_and(|ch| !ch.muted)
                        })
                }),
                ClipSource::Audio(ch) => p.channel(ch).is_some_and(|ch| !ch.muted),
                ClipSource::Automation(_) => true,
            };
            if sounds {
                active[tr] = true;
            }
        }
    }
    let st = &mut app.ui.playlist;
    st.track_activity.resize(tracks_len, 0.0);
    for (tr, on) in active.into_iter().enumerate() {
        if on {
            st.track_activity[tr] = 1.0;
        } else {
            st.track_activity[tr] = (st.track_activity[tr] - 0.15).max(0.0);
        }
    }
}

/// Draw the arrangement, ruler, and track headers, then handle edits, drops, and scrolling.
fn body(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let full = ui.available_rect_before_wrap();
    let (full, _) = ui.allocate_exact_size(full.size(), Sense::hover());
    let content = Rect::from_min_max(full.min, pos2(full.right() - 12.0, full.bottom()));
    let top_h = TOP_PREVIEW_H + RULER_H;
    let top_left = Rect::from_min_max(content.min, pos2(content.left() + HEADER_W, content.top() + top_h));
    let preview_rect = Rect::from_min_max(
        pos2(content.left() + HEADER_W, content.top()),
        pos2(content.right(), content.top() + TOP_PREVIEW_H),
    );
    let ruler = Rect::from_min_max(
        pos2(content.left() + HEADER_W, content.top() + TOP_PREVIEW_H),
        pos2(content.right(), content.top() + top_h),
    );
    let headers = Rect::from_min_max(
        pos2(content.left(), content.top() + top_h),
        pos2(content.left() + HEADER_W, content.bottom()),
    );
    let grid = Rect::from_min_max(
        pos2(content.left() + HEADER_W, content.top() + top_h),
        content.right_bottom(),
    );

    top_left_tools_ui(app, ui, top_left);
    update_track_activity(app);

    let tracks_vec = app.project().playlist.tracks.clone();
    let tracks = tracks_vec.len() as u32;
    app.ui.playlist.selected_tracks.retain(|&tr| tr < tracks);
    let follow = app.settings.auto_scroll && app.is_playing() && app.ui.mode == limeshot_engine::PlayMode::Song;
    let position = app.player.status().position();
    let ppt = app.ui.playlist.zoom_x / PPQ as f32;
    let song_pixels = app.project().playlist.song_length(app.project().time_signature) as f32 * ppt;
    let horizontal_extent = (song_pixels + grid.width()).max(app.ui.playlist.scroll.x + grid.width());
    let mut scroll_x = app.ui.playlist.scroll.x;
    preview_scrollbar(
        ui,
        app.project(),
        preview_rect,
        &mut scroll_x,
        grid.width(),
        horizontal_extent,
        ppt,
    );
    app.ui.playlist.scroll.x = scroll_x;

    let temp_view = View::new(grid, ppt, app.ui.playlist.track_h, app.ui.playlist.scroll, &tracks_vec);
    let total_y = temp_view.total_h + app.ui.playlist.track_h;
    let sig = app.project().time_signature;
    let st = &mut app.ui.playlist;
    scrollbar(
        ui,
        "playlist-vertical",
        Rect::from_min_max(pos2(content.right(), grid.top()), pos2(full.right(), content.bottom())),
        &mut st.scroll.y,
        grid.height(),
        total_y,
        true,
    );
    let max_y = (total_y - grid.height()).max(0.0);
    auto_scroll_on_drag(st, ui.ctx(), grid, max_y);
    st.scroll.y = st.scroll.y.clamp(0.0, max_y);
    st.scroll.x = st.scroll.x.max(0.0);
    if follow && st.drag.is_none() && st.sel_anchor.is_none() && st.marker_drag.is_none() {
        crate::widgets::follow_playhead(&mut st.scroll.x, position as f32 * st.zoom_x / PPQ as f32, grid.width());
    }
    let view = View::new(grid, ppt, st.track_h, st.scroll, &tracks_vec);
    let mut geom_layouts = [TrackLayout {
        y: 0.0,
        h: view.track_h,
        hidden: false,
    }; 500];
    let layout_count = view.layouts.len().min(geom_layouts.len());
    geom_layouts[..layout_count].copy_from_slice(&view.layouts[..layout_count]);
    st.geometry = Some(Geometry {
        grid,
        ppt: view.ppt,
        track_h: view.track_h,
        scroll: view.scroll,
        layouts: geom_layouts,
        layout_count,
        total_h: view.total_h,
    });
    let show_fades = st.audio_show_fades;
    let painter = ui.painter_at(full);

    painter.rect_filled(grid, 0.0, t.sunken);
    for tr in 0..tracks {
        if view.is_track_hidden(tr) {
            continue;
        }
        let y = view.y(tr);
        let h = view.track_height(tr);
        if y > grid.bottom() || y + h < grid.top() {
            continue;
        }
        if tr % 2 == 1 {
            painter.rect_filled(
                Rect::from_min_max(pos2(grid.left(), y), pos2(grid.right(), y + h)).intersect(grid),
                0.0,
                mix(t.sunken, t.surface, 0.35),
            );
        }
        painter.line_segment(
            [pos2(grid.left(), y + h), pos2(grid.right(), y + h)],
            Stroke::new(1.0, t.grid_step),
        );
    }
    // The grid follows the snap, as in the Piano Roll (it was fixed to bars).
    draw_time_grid(&painter, &view, grid, sig, app.ui.snap);
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
        if view.is_track_hidden(c.track) {
            continue;
        }
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
        let title_h = 14.0_f32.min(r.height());
        let title = Rect::from_min_size(r.min, vec2(r.width(), title_h))
            .intersect(r)
            .intersect(grid);
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
                draw_waveform(app, &painter, &view, grid, c, ch, body, mix(base, Color32::WHITE, 0.55));
                if show_fades {
                    draw_fades(&painter.with_clip_rect(rr), &view, c, body, fg);
                }
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
    if let (Some(Drag::Select { from } | Drag::Zoom { from }), Some(p)) =
        (&app.ui.playlist.drag, ui.input(|i| i.pointer.hover_pos()))
    {
        painter.rect(
            Rect::from_two_pos(*from, p),
            0.0,
            t.selection,
            Stroke::new(1.0, t.accent),
            egui::StrokeKind::Inside,
        );
    }
    // The Slice tool's cut line.
    if let (Some(Drag::Slice { from }), Some(p)) = (&app.ui.playlist.drag, ui.input(|i| i.pointer.hover_pos())) {
        let x = view.x(app.ui.snap.round(view.tick_at(from.x).max(0) as u64, sig));
        painter.line_segment(
            [pos2(x, from.y.min(p.y) - 4.0), pos2(x, from.y.max(p.y) + 4.0)],
            Stroke::new(1.5, t.accent),
        );
    }
    // The time selection, shaded over the arrangement.
    if let Some((a, b)) = app.ui.playlist.time_sel {
        let r = Rect::from_min_max(
            pos2(view.x(a).max(grid.left()), grid.top()),
            pos2(view.x(b).min(grid.right()), grid.bottom()),
        );
        if r.width() > 0.0 {
            painter.rect_filled(r, 0.0, t.selection.gamma_multiply(0.5));
        }
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
    block: usize,
}

impl Peaks {
    const BLOCK: usize = 256;

    /// Cache mono extrema in 256-frame blocks, including zero in each block's range.
    pub(crate) fn of(s: &limeshot_dsp::SampleData) -> Self {
        if let Some(disk) = &s.disk {
            return Self {
                blocks: disk.peaks.clone(),
                block: limeshot_dsp::disk::PEAK_BLOCK,
            };
        }
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
        Self {
            blocks,
            block: Self::BLOCK,
        }
    }

    /// Return cached extrema for the blocks containing frame positions `a` through `b`, inclusive.
    /// Negative positions clamp to zero; return `None` if the cache is empty or `a` is beyond it.
    pub(crate) fn range(&self, a: f64, b: f64) -> Option<(f32, f32)> {
        let i = (a.max(0.0) as usize) / self.block;
        let j = ((b.max(0.0) as usize) / self.block)
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
    let sp = limeshot_core::lime::sampler_params(&app.project().channel(ch)?.instrument)?;
    // Look up the resolved source path as is: re-resolving it as a stored path breaks embedded
    // sources (`/.limeshot-embedded/…` is not absolute on Windows).
    let path = app.sampler_source(ch)?;
    let Some(limeshot_audio::assets::SampleStatus::Ready(s)) = app.samples.status_of_path(&path).cloned() else {
        return None;
    };
    let stored = path.to_string_lossy().into_owned();
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

/// Samples dragged from the Browser become audio clips where they are dropped. While one is
/// dragged over the grid, a ghost clip shows where it will land and how long it will be.
fn sample_drop(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("pl_drop"), Sense::hover());
    app.ui.playlist.drop_ghost = None;
    source_drop(app, ui, view, grid, tracks);
    let drop_pos = app.external_drop_position(ui.ctx());
    let on_layer = drop_pos.is_some_and(|p| ui.ctx().layer_id_at(p) == Some(ui.layer_id()));
    let external = ui.input_mut(|i| {
        if drop_pos.is_some_and(|p| grid.contains(p)) && on_layer {
            std::mem::take(&mut i.raw.dropped_files)
        } else {
            Vec::new()
        }
    });
    if let Some(pos) = drop_pos {
        let mut track = view.track_at(pos.y).clamp(0, tracks.saturating_sub(1) as i32) as u32;
        let start = app
            .ui
            .snap
            .floor(view.tick_at(pos.x).max(0) as u64, app.project().time_signature);
        for path in external
            .into_iter()
            .map(|f| f.path().to_path_buf())
            .filter(|p| limeshot_dsp::is_audio_file(p))
        {
            add_audio_clip(app, &path, track, start);
            track = track.saturating_add(1);
        }
    }
    let sig = app.project().time_signature;
    let target = |app: &LimeShotApp, pos: Pos2| {
        let track = view.track_at(pos.y).clamp(0, tracks.saturating_sub(1) as i32) as u32;
        (track, app.ui.snap.floor(view.tick_at(pos.x).max(0) as u64, sig))
    };
    if let Some(payload) = resp.dnd_hover_payload::<crate::browser::DraggedSample>()
        && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
        && grid.contains(pos)
    {
        app.hint("Drop to place the sample as an audio clip");
        if app
            .ui
            .playlist
            .drop_len
            .as_ref()
            .is_none_or(|(p, len)| *p != payload.0 || len.is_none())
        {
            let stored = payload.0.to_string_lossy().into_owned();
            app.samples.request(&stored);
            let len = match app.samples.status(&stored) {
                Some(limeshot_audio::assets::SampleStatus::Ready(s)) => Some(sample_ticks(app, s)),
                _ => None,
            };
            app.ui.playlist.drop_len = Some((payload.0.clone(), len));
        }
        let (track, start) = target(app, pos);
        if let Some((_, Some(len))) = app.ui.playlist.drop_len {
            app.ui.playlist.drop_ghost = Some((track, start, len));
            let t = Theme::current();
            let h = view.track_height(track);
            let r = Rect::from_min_max(
                pos2(view.x(start), view.y(track) + 1.0),
                pos2(
                    view.x(start + len).max(view.x(start) + 4.0),
                    view.y(track) + (h - 1.0).max(1.0),
                ),
            );
            let painter = ui.painter_at(grid);
            painter.rect(
                r,
                CornerRadius::same(t.radius_sm),
                t.accent.gamma_multiply(0.25),
                Stroke::new(1.0, t.accent),
                egui::StrokeKind::Inside,
            );
            crate::browser::paint_sample_wave(
                app,
                &painter,
                r.shrink2(vec2(1.0, 14.0)).intersect(grid),
                &payload.0,
                t.accent,
            );
            painter.with_clip_rect(r.intersect(grid)).text(
                r.min + vec2(5.0, 1.0),
                Align2::LEFT_TOP,
                crate::browser::file_label(&payload.0),
                crate::theme::semibold(10.5),
                t.text,
            );
        }
    }
    let Some(payload) = resp.dnd_release_payload::<crate::browser::DraggedSample>() else {
        return;
    };
    app.ui.playlist.drop_len = None;
    let Some(pos) = ui.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) else {
        return;
    };
    let (track, start) = target(app, pos);
    if let Some(id) = add_audio_clip(app, &payload.0, track, start) {
        app.ui.playlist.selection = vec![id];
    }
}

/// Place the sample file at `path` as an audio clip (decoding it now to know its length).
pub fn add_audio_clip(app: &mut LimeShotApp, path: &std::path::Path, track: u32, start: Tick) -> Option<ClipId> {
    let sample = crate::browser::import_audio(app, path, crate::browser::ImportTarget::Playlist { track, start })?;
    let length = sample_ticks(app, &sample);
    let (channel, mut commands) = audio_source_commands(app, path, &sample);
    if track >= 500 {
        app.toast_error("The Playlist holds at most 500 tracks");
        return None;
    }
    commands.extend((app.project().playlist.tracks.len()..=track as usize).map(|_| Command::AddPlaylistTrack));
    commands.push(Command::PlaceClip {
        source: ClipSource::Audio(channel),
        track,
        start,
        length,
        offset: 0,
    });
    let id = app.exec(Command::Batch(commands))?;

    if let Some(c) = app.project().clip(ClipId(id)).cloned() {
        // The picker shows the new audio clip, ready to be drawn elsewhere (FL Studio).
        focus_source(app, &c);
    }
    Some(ClipId(id))
}

/// An audio clip's fade lines (from the corners to the handles) and the handles themselves.
fn draw_fades(painter: &egui::Painter, view: &View, c: &Clip, body: Rect, color: Color32) {
    let Some((a, b)) = fade_handles(view, c) else { return };
    let line = Stroke::new(1.2, color.gamma_multiply(0.85));
    for (out, end) in [(false, a), (true, b)] {
        let length = if out { c.fade_out } else { c.fade_in };
        if length > 0 {
            let points = (0..=32)
                .map(|i| {
                    let u = i as f32 / 32.0;
                    let gain = limeshot_dsp::voice::fade_curve(if out { 1.0 - u } else { u }, c.fade_curve(out));
                    let x = if out {
                        egui::lerp(end.x..=body.right(), u)
                    } else {
                        egui::lerp(body.left()..=end.x, u)
                    };
                    pos2(x, egui::lerp(body.bottom()..=end.y, gain))
                })
                .collect();
            painter.add(egui::Shape::line(points, line));
            if let Some(handle) = fade_curve_handle(view, c, out) {
                painter.circle(handle, 3.0, Theme::current().surface, line);
            }
        }
        let direction = if out { -1.0 } else { 1.0 };
        painter.add(egui::Shape::convex_polygon(
            vec![
                end + vec2(-4.0 * direction, -3.0),
                end + vec2(5.0 * direction, -3.0),
                end + vec2(-4.0 * direction, 5.0),
            ],
            color,
            Stroke::NONE,
        ));
    }
}

fn fade_curve_handle(view: &View, clip: &Clip, out: bool) -> Option<Pos2> {
    let (a, b) = fade_handles(view, clip)?;
    let length = if out { clip.fade_out } else { clip.fade_in };
    if length == 0 {
        return None;
    }
    let rect = view.clip_rect(clip);
    let end = if out { b } else { a };
    let x = if out {
        (end.x + rect.right()) / 2.0
    } else {
        (end.x + rect.left()) / 2.0
    };
    let gain = limeshot_dsp::voice::fade_curve(0.5, clip.fade_curve(out));
    Some(pos2(x, egui::lerp((rect.bottom() - 2.0)..=end.y, gain)))
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

/// Hit-test the topmost clip for an automation point, tension handle, or new-point position.
/// New points may be placed anywhere in the curve body away from resize edges; their local tick
/// is snapped and clamped to the clip's source window, and their value is normalized to 0–1.
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
    let v = ((area.bottom() - p.y) / area.height()).clamp(0.0, 1.0);
    if area.contains(p) {
        let tick = app.ui.snap.round(local, sig).clamp(c.offset, c.offset + c.length - 1);
        return Some((c.clone(), aid, AutoHit::Curve(tick, v)));
    }
    None
}

fn ruler_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, ruler: Rect, sig: TimeSignature) {
    let t = Theme::current();
    let painter = ui.painter_at(ruler);
    draw_ruler(&painter, view, ruler, sig);
    if let Some((a, b)) = app.ui.playlist.time_sel {
        let r = Rect::from_min_max(
            pos2(view.x(a).max(ruler.left()), ruler.top()),
            pos2(view.x(b).min(ruler.right()), ruler.bottom()),
        );
        if r.width() > 0.0 {
            painter.rect_filled(r, 0.0, mix(t.surface, t.danger, 0.45).gamma_multiply(0.8));
        }
    }
    // Time markers: flags in the ruler's lower half.
    app.ui.playlist.marker_rects.clear();
    let markers = app.project().playlist.markers.clone();
    let flag_h = ruler.height() / 2.0;
    for m in &markers {
        let x = view.x(m.tick);
        if x < ruler.left() - 200.0 || x > ruler.right() {
            continue;
        }
        let galley = painter.layout_no_wrap(m.name.clone(), crate::theme::prop(10.5), t.text);
        let r = Rect::from_min_size(pos2(x, ruler.bottom() - flag_h), vec2(galley.size().x + 8.0, flag_h));
        painter.rect_filled(
            r,
            CornerRadius {
                nw: 0,
                ne: 4,
                sw: 0,
                se: 0,
            },
            mix(t.raised, t.accent, 0.3),
        );
        painter.line_segment(
            [pos2(x, ruler.top()), pos2(x, ruler.bottom())],
            Stroke::new(1.0, t.accent),
        );
        painter.galley(pos2(x + 4.0, r.center().y - galley.size().y / 2.0), galley, t.text);
        app.ui.playlist.marker_rects.push((m.id, r));
    }

    let resp = ui.interact(ruler, ui.id().with("pl_ruler"), Sense::click_and_drag());
    let mods = ui.input(|i| i.modifiers);
    let snap = if mods.alt { Snap::None } else { app.ui.snap };
    let snapped = |p: Pos2| snap.round(view.tick_at(p.x).max(0) as u64, sig);
    let marker_at = |app: &LimeShotApp, p: Pos2| {
        app.ui
            .playlist
            .marker_rects
            .iter()
            .rev()
            .find(|(_, r)| r.contains(p))
            .map(|(id, _)| *id)
    };
    let pressed =
        resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
    if pressed && let Some(p) = resp.interact_pointer_pos() {
        if let Some(id) = marker_at(app, p) {
            app.doc.begin_gesture();
            app.ui.playlist.marker_drag = Some((id, false));
        } else if mods.command {
            // Ctrl/Cmd+drag: a time selection (observed in FL Studio 2026.1.1).
            let a = snapped(p);
            app.ui.playlist.sel_anchor = Some(a);
            app.ui.playlist.time_sel = None;
        }
    }
    let gesture = app.ui.playlist.marker_drag.is_some() || app.ui.playlist.sel_anchor.is_some();
    if let Some(p) = resp.interact_pointer_pos() {
        if let Some((id, moved)) = app.ui.playlist.marker_drag {
            let tick = snapped(p);
            let at = markers.iter().find(|m| m.id == id).map(|m| m.tick);
            if resp.dragged() && at != Some(tick) && (moved || resp.drag_delta().x.abs() > 0.0) {
                app.exec(Command::MoveMarker { marker: id, tick });
                app.ui.playlist.marker_drag = Some((id, true));
            }
        } else if let Some(a) = app.ui.playlist.sel_anchor {
            let b = snapped(p);
            app.ui.playlist.time_sel = Some((a.min(b), a.max(b)));
            app.ui.playlist.selection = app
                .project()
                .playlist
                .clips
                .iter()
                .filter(|c| c.start < a.max(b) && c.end() > a.min(b))
                .map(|c| c.id)
                .collect();
        } else if resp.clicked() || resp.dragged() {
            let tick = Snap::Beat.floor(view.tick_at(p.x).max(0) as u64, sig);
            app.set_mode(PlayMode::Song);
            app.locate(tick);
        }
    }
    if gesture && !ui.input(|i| i.pointer.primary_down()) {
        if let Some((id, moved)) = app.ui.playlist.marker_drag.take() {
            app.doc.end_gesture();
            if !moved && let Some(m) = app.project().playlist.markers.iter().find(|m| m.id == id) {
                // Clicking a marker jumps to it.
                let tick = m.tick;
                app.set_mode(PlayMode::Song);
                app.locate(tick);
            }
        }
        if app.ui.playlist.sel_anchor.take().is_some() {
            match app.ui.playlist.time_sel {
                Some((a, b)) if b > a => {
                    app.set_mode(PlayMode::Song);
                    app.locate(a);
                }
                // A Cmd+click without a drag clears the selection.
                _ => app.ui.playlist.time_sel = None,
            }
        }
    }
    if resp.secondary_clicked() {
        app.ui.playlist.menu_marker = resp.interact_pointer_pos().and_then(|p| marker_at(app, p));
    }
    let click_tick = resp.interact_pointer_pos().map(snapped);
    resp.context_menu(|ui| match app.ui.playlist.menu_marker {
        Some(id) => {
            if ui.button("Rename…").clicked() {
                app.dialog = Some(crate::dialogs::Dialog::rename_marker(app, id));
                ui.close();
            }
            if ui.button("Delete").clicked() {
                app.exec(Command::RemoveMarker(id));
                ui.close();
            }
        }
        None => {
            if ui.button("Add marker here").clicked() {
                let tick = click_tick.unwrap_or(0);
                add_marker(app, tick);
                ui.close();
            }
            if app.ui.playlist.time_sel.is_some() && ui.button("Clear time selection").clicked() {
                app.ui.playlist.time_sel = None;
                ui.close();
            }
        }
    });
    if resp.hovered() {
        app.hint("Ruler: click/drag sets the song position · Cmd+drag: time selection (loops in Song mode) · markers: click to jump, drag to move, right-click for a menu");
    }
}

/// Add a time marker named after the number of markers ("Marker 3"); returns its id.
pub fn add_marker(app: &mut LimeShotApp, tick: Tick) -> Option<MarkerId> {
    let name = format!("Marker {}", app.project().playlist.markers.len() + 1);
    app.exec(Command::AddMarker { tick, name }).map(MarkerId)
}

/// Draw track headers and handle mute locks, solo, renaming, track selection, height resizing,
/// grouping, track menus, and adding a track.
fn headers_ui(app: &mut LimeShotApp, ui: &mut Ui, view: &View, area: Rect, tracks: u32) {
    let t = Theme::current();
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 0.0, t.surface);
    app.ui.playlist.track_leds.clear();
    app.ui.playlist.track_mute_rects.clear();
    app.ui.playlist.track_resize_rects.clear();
    app.ui.playlist.track_group_rects.clear();
    let tracks_vec: Vec<PlaylistTrack> = app.project().playlist.tracks.clone();
    for tr in 0..tracks {
        if view.is_track_hidden(tr) {
            continue;
        }
        let y = view.y(tr);
        let h = view.track_height(tr);
        if y > area.bottom() || y + h < area.top() {
            continue;
        }
        let r = Rect::from_min_max(pos2(area.left(), y), pos2(area.right() - 2.0, y + h - 1.0));
        if r.bottom() < area.top() {
            continue;
        }
        let track_data = tracks_vec
            .get(tr as usize)
            .cloned()
            .unwrap_or_else(|| PlaylistTrack::new(format!("Track {}", tr + 1)));
        let muted = track_data.muted;
        let locked = track_data.mute_locked;
        let color = track_data.color;
        let grouped = track_data.grouped_with_above;
        let is_parent = !grouped
            && tracks_vec
                .get(tr as usize + 1)
                .is_some_and(|next| next.grouped_with_above);
        let selected = app.ui.playlist.selected_tracks.contains(&tr);
        let activity = app.ui.playlist.track_activity.get(tr as usize).copied().unwrap_or(0.0);

        let left_indent = if grouped { 10.0 } else { 0.0 };
        let header_r = Rect::from_min_max(pos2(r.left() + left_indent, r.top()), r.max);
        let mut base_fill = color.map_or(t.raised, |c| mix(t.raised, rgb(c), 0.25));
        if selected {
            base_fill = mix(base_fill, t.accent, 0.14);
        }
        painter.rect_filled(header_r.intersect(area), CornerRadius::same(2), base_fill);
        if grouped {
            // Visual group connector bar on the left of a child track.
            let conn = Rect::from_min_max(pos2(r.left() + 3.0, r.top() - 1.0), pos2(r.left() + 6.0, r.bottom()));
            painter.rect_filled(
                conn.intersect(area),
                CornerRadius::same(1),
                color.map_or(t.accent_dim, rgb),
            );
        }

        // Channel-Rack-style track selector + playback monitoring bar on the right edge.
        let sel_w = 8.0;
        let sel_rect = Rect::from_min_max(
            pos2(r.right() - sel_w - 2.0, r.top() + 3.0),
            pos2(r.right() - 2.0, (r.bottom() - 3.0).max(r.top() + 7.0)),
        );
        app.ui.playlist.track_leds.push((tr, sel_rect));
        if sel_rect.bottom() > area.top() && sel_rect.top() < area.bottom() {
            let sr = sel_rect.intersect(area);
            painter.rect_filled(sr, CornerRadius::same(2), t.sunken);
            if selected || activity > 0.01 {
                let inner_c = if activity > 0.01 {
                    mix(t.sunken, t.accent, 0.45 + 0.55 * activity)
                } else {
                    mix(t.sunken, t.accent, 0.35)
                };
                painter.rect_filled(sr.shrink(1.5), CornerRadius::same(1), inner_c);
            }
            if selected {
                painter.rect_stroke(
                    sr,
                    CornerRadius::same(2),
                    Stroke::new(1.2, t.accent),
                    egui::StrokeKind::Inside,
                );
            }
        }
        let sel_resp = ui.interact(
            sel_rect.intersect(area),
            ui.id().with(("pl_track_sel", tr)),
            Sense::click_and_drag(),
        );
        let mods = ui.input(|i| i.modifiers);
        let stroke = egui::Id::new("playlist-track-selection-stroke");
        if sel_resp.drag_started() {
            let origin = ui.input(|i| i.pointer.press_origin()).unwrap_or(sel_rect.center()).y;
            ui.ctx().data_mut(|d| d.insert_temp(stroke, origin));
            if !mods.command && !mods.shift && !app.ui.playlist.selected_tracks.contains(&tr) {
                app.ui.playlist.selected_tracks = vec![tr];
            }
        }
        if let Some(origin) = ui.ctx().data(|d| d.get_temp::<f32>(stroke)) {
            if ui.input(|i| i.pointer.primary_down()) {
                if let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
                    && (origin.min(pointer.y)..=origin.max(pointer.y)).contains(&sel_rect.center().y)
                    && !app.ui.playlist.selected_tracks.contains(&tr)
                {
                    app.ui.playlist.selected_tracks.push(tr);
                }
            } else {
                ui.ctx().data_mut(|d| d.remove::<f32>(stroke));
            }
        }
        if sel_resp.secondary_clicked() || (sel_resp.clicked() && mods.command) {
            if let Some(pos) = app.ui.playlist.selected_tracks.iter().position(|&x| x == tr) {
                app.ui.playlist.selected_tracks.remove(pos);
            } else {
                app.ui.playlist.selected_tracks.push(tr);
            }
        } else if sel_resp.clicked() && mods.shift {
            if let Some(&anchor) = app.ui.playlist.selected_tracks.last() {
                let (a, b) = (anchor.min(tr), anchor.max(tr));
                for k in a..=b {
                    if !app.ui.playlist.selected_tracks.contains(&k) {
                        app.ui.playlist.selected_tracks.push(k);
                    }
                }
            } else {
                app.ui.playlist.selected_tracks.push(tr);
            }
        } else if sel_resp.clicked() {
            app.ui.playlist.selected_tracks = vec![tr];
        }
        sel_resp.hint("Select track (click: select one, drag: select multiple, Cmd/right-click: toggle, Shift+click: range) · lights up with playback activity");

        // Mute / solo LED to the left of the selector bar.
        let led = Rect::from_center_size(pos2(sel_rect.left() - 9.0, r.center().y), vec2(10.0, 10.0));
        if led.top() >= area.top() && led.bottom() <= area.bottom() {
            let led_color = if activity > 0.01 && !muted {
                mix(t.accent, Color32::WHITE, 0.35 * activity)
            } else if muted {
                t.text_faint
            } else {
                t.accent
            };
            if locked {
                crate::icons::paint(&painter, led.expand(2.0), Icon::Lock, led_color);
            } else {
                painter.circle_filled(led.center(), 4.5, led_color);
            }
        }

        // Group fold chevron on parent tracks.
        let mut text_left = header_r.left() + 6.0;
        if is_parent {
            let grp_rect = Rect::from_center_size(pos2(header_r.left() + 8.0, r.center().y), vec2(12.0, 12.0));
            app.ui.playlist.track_group_rects.push((tr, grp_rect));
            text_left = header_r.left() + 16.0;
            if grp_rect.top() >= area.top() && grp_rect.bottom() <= area.bottom() {
                painter.text(
                    grp_rect.center(),
                    Align2::CENTER_CENTER,
                    if track_data.group_collapsed { "▸" } else { "▾" },
                    mono(10.0),
                    t.text_dim,
                );
            }
            let grp_resp = ui.interact(
                grp_rect.intersect(area),
                ui.id().with(("pl_grp_fold", tr)),
                Sense::click(),
            );
            if grp_resp.clicked() {
                app.exec(Command::SetPlaylistTrackGroupCollapsed {
                    track: tr,
                    collapsed: !track_data.group_collapsed,
                });
            }
            grp_resp.hint(if track_data.group_collapsed {
                "Expand grouped tracks"
            } else {
                "Collapse grouped tracks"
            });
        }

        if r.center().y > area.top() && r.center().y < area.bottom() {
            let text_clip =
                Rect::from_min_max(pos2(text_left, r.top()), pos2(led.left() - 2.0, r.bottom())).intersect(area);
            painter.with_clip_rect(text_clip).text(
                pos2(text_left, r.center().y),
                Align2::LEFT_CENTER,
                &track_data.name,
                crate::theme::prop(if h <= COLLAPSED_TRACK_H + 2.0 { 10.0 } else { 11.5 }),
                if muted { t.text_dim } else { t.text },
            );
        }
        app.ui.playlist.track_mute_rects.push((tr, led.intersect(area)));
        let led_resp = ui.interact(led.intersect(area), ui.id().with(("pl_mute", tr)), Sense::click());
        if led_resp.clicked() && ui.input(|i| i.modifiers.shift) {
            app.exec(Command::SetPlaylistTrackMuteLock {
                track: tr,
                locked: !locked,
            });
        } else if led_resp.secondary_clicked() || (led_resp.clicked() && ui.input(|i| i.modifiers.command)) {
            solo_track(app, tr);
        } else if led_resp.clicked() {
            app.exec(Command::SetPlaylistTrackMute {
                track: tr,
                muted: !muted,
            });
        }
        led_resp.hint("Mute track · right-click: solo · Shift+click: lock mute state");

        // Bottom-edge drag handle for resizing/collapsing track height (unless size_locked).
        let resize_rect = Rect::from_min_max(
            pos2(header_r.left(), r.bottom() - 3.0),
            pos2(led.left() - 2.0, r.bottom() + 2.0),
        );
        app.ui.playlist.track_resize_rects.push((tr, resize_rect));
        let resize_resp = ui.interact(
            resize_rect.intersect(area),
            ui.id().with(("pl_resize_tr", tr)),
            Sense::click_and_drag(),
        );
        if !track_data.size_locked && (resize_resp.hovered() || resize_resp.dragged()) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if !track_data.size_locked && resize_resp.drag_started() {
            let gy = resize_resp.interact_pointer_pos().map_or(r.bottom(), |p| p.y);
            app.doc.begin_gesture();
            app.ui.playlist.drag = Some(Drag::ResizeTrack {
                track: tr,
                grab_y: gy,
                orig_h: h,
            });
        }
        if !track_data.size_locked && resize_resp.double_clicked() {
            app.exec(Command::SetPlaylistTrackHeight {
                track: tr,
                height: None,
                collapsed: false,
            });
        }
        resize_resp.hint(if track_data.size_locked {
            "Track size is locked"
        } else {
            "Drag to resize track height (drag to minimum to collapse · double-click to reset)"
        });

        let name_rect = Rect::from_min_max(
            pos2(text_left, r.top()),
            pos2(led.left() - 2.0, (r.bottom() - 3.0).max(r.top() + 1.0)),
        );
        let name_resp = ui.interact(name_rect.intersect(area), ui.id().with(("pl_name", tr)), Sense::click());
        if name_resp.clicked() {
            if mods.command {
                if let Some(pos) = app.ui.playlist.selected_tracks.iter().position(|&x| x == tr) {
                    app.ui.playlist.selected_tracks.remove(pos);
                } else {
                    app.ui.playlist.selected_tracks.push(tr);
                }
            } else if mods.shift {
                if let Some(&anchor) = app.ui.playlist.selected_tracks.last() {
                    let (a, b) = (anchor.min(tr), anchor.max(tr));
                    for k in a..=b {
                        if !app.ui.playlist.selected_tracks.contains(&k) {
                            app.ui.playlist.selected_tracks.push(k);
                        }
                    }
                } else {
                    app.ui.playlist.selected_tracks.push(tr);
                }
            } else {
                app.ui.playlist.selected_tracks = vec![tr];
            }
        }
        if name_resp.double_clicked() {
            app.dialog = Some(crate::dialogs::Dialog::rename_track(app, tr));
        }
        name_resp
            .hint("Click to select track · double-click to rename · right-click: track options")
            .context_menu(|ui| track_menu(app, ui, tr));
    }
    if let Some(Drag::ResizeTrack { track, grab_y, orig_h }) = app.ui.playlist.drag.clone() {
        if ui.input(|i| i.pointer.any_down()) {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                let target_h = orig_h + (p.y - grab_y);
                let (height, collapsed) = if target_h <= COLLAPSED_TRACK_H + 2.0 {
                    (Some(COLLAPSED_TRACK_H), true)
                } else {
                    (Some(target_h.clamp(TRACK_H.0, TRACK_H.1)), false)
                };
                app.exec(Command::SetPlaylistTrackHeight {
                    track,
                    height,
                    collapsed,
                });
            }
        } else {
            app.doc.end_gesture();
            app.ui.playlist.drag = None;
        }
    }
    let y = view.y(tracks);
    let add_rect = Rect::from_min_size(pos2(area.left() + 4.0, y + 4.0), vec2(area.width() - 8.0, 24.0));
    if area.contains(add_rect.center()) && ui.put(add_rect, egui::Button::new("+ Track")).clicked() {
        app.exec(Command::AddPlaylistTrack);
    }
}

/// Solo `track` by muting other unlocked tracks in one undo step, leaving locked tracks alone.
/// If the unlocked tracks already match that solo state, unmute all unlocked tracks instead.
fn solo_track(app: &mut LimeShotApp, track: u32) {
    let already = app
        .project()
        .playlist
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.mute_locked)
        .all(|(i, t)| t.muted == (i != track as usize));
    let cmds = app
        .project()
        .playlist
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.mute_locked)
        .map(|(i, _)| Command::SetPlaylistTrackMute {
            track: i as u32,
            muted: !already && i != track as usize,
        })
        .collect();
    app.exec(Command::Batch(cmds));
}

/// Show actions for a Playlist track, applying track and clip edits through undoable commands.
/// Draw nothing if the track no longer exists.
fn track_menu(app: &mut LimeShotApp, ui: &mut Ui, track: u32) {
    let Some(t) = app.project().playlist.tracks.get(track as usize).cloned() else {
        return;
    };
    if ui.button("Rename…").clicked() {
        app.dialog = Some(crate::dialogs::Dialog::rename_track(app, track));
        ui.close();
    }
    ui.menu_button("Change color", |ui| {
        for color in PALETTE {
            if ui.add(egui::Button::new("      ").fill(rgb(color))).clicked() {
                app.exec(Command::SetPlaylistTrackColor {
                    track,
                    color: Some(color),
                });
                ui.close();
            }
        }
    });
    if ui.button("Auto name").clicked() {
        if let Some(c) = app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| c.track == track)
            .min_by_key(|c| c.start)
        {
            let (name, color) = source_name_color(app, c.source);
            app.exec(Command::Batch(vec![
                Command::RenamePlaylistTrack { track, name },
                Command::SetPlaylistTrackColor {
                    track,
                    color: Some(color),
                },
            ]));
        }
        ui.close();
    }
    if ui.button("Reset").clicked() {
        app.exec(Command::Batch(vec![
            Command::RenamePlaylistTrack {
                track,
                name: format!("Track {}", track + 1),
            },
            Command::SetPlaylistTrackColor { track, color: None },
        ]));
        ui.close();
    }
    ui.separator();
    if ui
        .add_enabled(true, egui::Button::new(if t.muted { "Unmute" } else { "Mute" }))
        .clicked()
    {
        app.exec(Command::SetPlaylistTrackMute { track, muted: !t.muted });
        ui.close();
    }
    if ui.button("Solo").clicked() {
        solo_track(app, track);
        ui.close();
    }
    if ui.selectable_label(t.mute_locked, "Lock mute state").clicked() {
        app.exec(Command::SetPlaylistTrackMuteLock {
            track,
            locked: !t.mute_locked,
        });
        ui.close();
    }
    if ui.button("Select all clips on this track").clicked() {
        app.ui.playlist.selection = app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| c.track == track)
            .map(|c| c.id)
            .collect();
        ui.close();
    }
    ui.separator();
    if ui.button("Merge pattern clips").clicked() {
        let ids = app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| c.track == track && c.pattern().is_some())
            .map(|c| c.id)
            .collect();
        app.exec(Command::MergeClips(ids));
        ui.close();
    }
    for (label, muted) in [("Mute all clips", true), ("Unmute all clips", false)] {
        if ui.button(label).clicked() {
            let commands = app
                .project()
                .playlist
                .clips
                .iter()
                .filter(|c| c.track == track)
                .map(|c| Command::SetClipMuted { clip: c.id, muted })
                .collect();
            app.exec(Command::Batch(commands));
            ui.close();
        }
    }
    if ui
        .add_enabled(
            app.project().playlist.tracks.len() < 500,
            egui::Button::new("Clone track"),
        )
        .clicked()
    {
        app.exec(Command::ClonePlaylistTrack(track));
        ui.close();
    }
    for (label, delta) in [("Move up", -1), ("Move down", 1)] {
        if ui
            .add_enabled(
                if delta < 0 {
                    track > 0
                } else {
                    (track as usize) + 1 < app.project().playlist.tracks.len()
                },
                egui::Button::new(label),
            )
            .clicked()
        {
            app.exec(Command::MovePlaylistTrack { track, delta });
            ui.close();
        }
    }
    if ui.button("Insert one").clicked() {
        app.exec(Command::InsertPlaylistTrack { track });
        ui.close();
    }
    if ui.button("Delete").clicked() {
        app.exec(Command::RemovePlaylistTrack { track });
        ui.close();
    }
    ui.separator();
    ui.menu_button("Size", |ui| {
        if ui.button("Reset to default").clicked() {
            app.exec(Command::SetPlaylistTrackHeight {
                track,
                height: None,
                collapsed: false,
            });
            ui.close();
        }
        if ui.selectable_label(t.collapsed, "Min size (collapse)").clicked() {
            app.exec(Command::SetPlaylistTrackHeight {
                track,
                height: if t.collapsed { None } else { Some(COLLAPSED_TRACK_H) },
                collapsed: !t.collapsed,
            });
            ui.close();
        }
        ui.separator();
        for (pct, h) in [
            ("50%", 24.0),
            ("100%", 44.0),
            ("150%", 66.0),
            ("200%", 88.0),
            ("300%", 132.0),
        ] {
            if ui.button(pct).clicked() {
                app.exec(Command::SetPlaylistTrackHeight {
                    track,
                    height: Some(h),
                    collapsed: false,
                });
                ui.close();
            }
        }
    });
    if ui.selectable_label(t.size_locked, "Lock to this size").clicked() {
        app.exec(Command::SetPlaylistTrackSizeLock {
            track,
            locked: !t.size_locked,
        });
        ui.close();
    }
    if ui
        .add_enabled(
            track > 0,
            egui::Button::selectable(t.grouped_with_above, "Group with above track"),
        )
        .clicked()
    {
        app.exec(Command::SetPlaylistTracksGrouped {
            tracks: vec![track],
            grouped: !t.grouped_with_above,
        });
        ui.close();
    }
    let tracks_slice = &app.project().playlist.tracks;
    let is_in_group = t.grouped_with_above
        || tracks_slice
            .get(track as usize + 1)
            .is_some_and(|next| next.grouped_with_above);
    if ui
        .add_enabled(is_in_group, egui::Button::new("Auto color group"))
        .clicked()
    {
        let mut root = track as usize;
        while root > 0 && tracks_slice[root].grouped_with_above {
            root -= 1;
        }
        let color = tracks_slice[root].color.unwrap_or(PALETTE[root % PALETTE.len()]);
        let mut cmds = vec![Command::SetPlaylistTrackColor {
            track: root as u32,
            color: Some(color),
        }];
        let mut k = root + 1;
        while k < tracks_slice.len() && tracks_slice[k].grouped_with_above {
            cmds.push(Command::SetPlaylistTrackColor {
                track: k as u32,
                color: Some(color),
            });
            k += 1;
        }
        app.exec(Command::Batch(cmds));
        ui.close();
    }
    ui.menu_button("Track mode", |ui| {
        ui.label("Unassigned");
        ui.add_enabled(
            false,
            egui::Button::new("Audio / instrument track — not implemented yet"),
        );
    });
    for label in [
        "Set icon",
        "Auto name clips",
        "Performance settings",
        "Lock to content",
        "Consolidate track/s",
    ] {
        ui.add_enabled(false, egui::Button::new(label))
            .on_hover_text("Not implemented yet");
    }
}

/// Return a clip source's display name and color, falling back to `Clip` and the first palette color.
fn source_name_color(app: &LimeShotApp, source: ClipSource) -> (String, Rgb) {
    match source {
        ClipSource::Pattern(id) => app.project().pattern(id).map(|p| (p.name.clone(), p.color)),
        ClipSource::Audio(id) => app.project().channel(id).map(|c| (c.name.clone(), c.color)),
        ClipSource::Automation(id) => app.project().automation(id).map(|a| (a.name.clone(), a.color)),
    }
    .unwrap_or_else(|| ("Clip".into(), PALETTE[0]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Body,
    LeftEdge,
    RightEdge,
    Menu,
    /// An audio clip's fade-in / fade-out handle (FL Studio's fade handles).
    FadeCurveIn,
    FadeCurveOut,
    FadeIn,
    FadeOut,
}

/// Where an audio clip's fade handles are: on the top of its body, at the end of the fade in and
/// the start of the fade out.
fn fade_handles(view: &View, c: &Clip) -> Option<(Pos2, Pos2)> {
    c.audio_channel()?;
    let r = view.clip_rect(c);
    // Inside the edges (which resize the clip) and just under the title strip.
    if r.height() < 30.0 || r.width() < 40.0 {
        return None;
    }
    let y = r.top() + 19.0;
    let x_in = (view.x(c.start + c.fade_in)).clamp(r.left() + 12.0, r.right() - 12.0);
    let x_out = (view.x(c.end() - c.fade_out)).clamp(r.left() + 12.0, r.right() - 12.0);
    Some((pos2(x_in, y), pos2(x_out, y)))
}

fn hit(view: &View, clips: &[Clip], p: Pos2, show_fades: bool) -> Option<(Clip, Part)> {
    clips.iter().rev().find_map(|c| {
        if view.is_track_hidden(c.track) {
            return None;
        }
        let r = view.clip_rect(c);
        r.contains(p).then(|| {
            let edge = EDGE.min(r.width() * 0.3);
            let fades = show_fades.then(|| fade_handles(view, c)).flatten();
            let inside_edges = p.x > r.left() + edge && p.x < r.right() - edge;
            let part = if show_fades
                && inside_edges
                && fade_curve_handle(view, c, false).is_some_and(|h| h.distance(p) <= 6.0)
            {
                Part::FadeCurveIn
            } else if show_fades
                && inside_edges
                && fade_curve_handle(view, c, true).is_some_and(|h| h.distance(p) <= 6.0)
            {
                Part::FadeCurveOut
            } else if fades.is_some_and(|(a, _)| a.distance(p) <= 5.0) {
                Part::FadeIn
            } else if fades.is_some_and(|(_, b)| b.distance(p) <= 5.0) {
                Part::FadeOut
            } else if p.x <= r.left() + 13.0 && p.y <= r.top() + 13.0 {
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

/// Snap an audio clip's local tick offset (`local_tick` from sample start) to the nearest zero
/// crossing in the channel's sample within a short window (~15 ms), returning the adjusted local tick.
fn snap_local_to_zero_crossing(app: &LimeShotApp, ch: ChannelId, local_tick: Tick) -> Tick {
    let Some((_, sample, first, per_tick)) = clip_audio(app, ch) else {
        return local_tick;
    };
    if per_tick <= 0.0 || sample.frames() < 2 {
        return local_tick;
    }
    let center = (first + local_tick as f64 * per_tick).round() as isize;
    let radius = ((sample.sample_rate as usize) / 60).clamp(16, 2048) as isize;
    let max_frame = sample.frames().saturating_sub(2) as isize;
    if max_frame <= 0 {
        return local_tick;
    }
    let mut best_frame = None;
    let mut best_dist = isize::MAX;
    let lo = (center - radius).clamp(0, max_frame);
    let hi = (center + radius).clamp(0, max_frame);
    for i in lo..=hi {
        let (l0, r0) = sample.frame(i as usize);
        let (l1, r1) = sample.frame(i as usize + 1);
        let m0 = 0.5 * (l0 + r0);
        let m1 = 0.5 * (l1 + r1);
        if (m0 <= 0.0 && m1 >= 0.0) || (m0 >= 0.0 && m1 <= 0.0) {
            let d = (i - center).abs();
            if d < best_dist {
                best_dist = d;
                best_frame = Some(i as f64);
            }
        }
    }
    if let Some(f) = best_frame {
        (((f - first) / per_tick).round().max(0.0)) as Tick
    } else {
        local_tick
    }
}

/// Adjust an audio channel's sampler pitch so the clip stretches from `orig_len` to `new_len`.
fn apply_audio_stretch(app: &mut LimeShotApp, ch: ChannelId, orig_pitch: f32, orig_len: Tick, new_len: Tick) {
    if orig_len == 0 || new_len == 0 {
        return;
    }
    let Some(Instrument::Plugin(plugin)) = app.project().channel(ch).map(|c| c.instrument.clone()) else {
        return;
    };
    let Some(spec) = limeshot_core::lime::SAMPLER_CONTROLS.get(1) else {
        return;
    };
    let ratio = orig_len as f32 / new_len as f32;
    let pitch = (orig_pitch + 12.0 * ratio.log2()).clamp(spec.min, spec.max);
    let norm = f64::from(limeshot_core::lime::plugin_normalized(spec, pitch));
    let id = limeshot_core::lime::param_id(1);
    let target = PluginTarget::Channel(ch);
    app.plugins.set_param(target, id, norm);
    let Some(state) = limeshot_core::lime::edited_param_state(&plugin, id, norm) else {
        return;
    };
    app.exec(Command::SetPluginState { plugin: target, state });
}

/// Make a clicked clip's source the thing being edited and placed (FL Studio selects it in the
/// picker), and remember its length for the next clip of that source.
fn focus_source(app: &mut LimeShotApp, c: &Clip) {
    pick_source(app, c.source);
    app.ui.playlist.last_len = Some((c.source, c.length));
}

/// A "Color" submenu with the palette swatches for pattern `id`. Returns whether one was picked.
pub fn pattern_color_menu(app: &mut LimeShotApp, ui: &mut Ui, id: PatternId) -> bool {
    let current = app
        .project()
        .pattern(id)
        .map(|p| p.color)
        .unwrap_or(limeshot_core::PALETTE[0]);
    let mut picked = false;
    ui.menu_button("Color", |ui| {
        if let Some(color) = crate::widgets::swatch_picker(ui, current) {
            app.exec(Command::SetPatternColor { pattern: id, color });
            picked = true;
            ui.close();
        }
    });
    picked
}

fn scrub_audio(app: &mut LimeShotApp, clips: &[Clip], tick: Option<Tick>) {
    if app.ui.playlist.scrub_tick == tick {
        return;
    }
    for (channel, key) in std::mem::take(&mut app.ui.playlist.scrub_audio) {
        app.preview_off(channel, key);
    }
    app.ui.playlist.scrub_tick = tick;
    let Some(tick) = tick else { return };
    let seconds_per_tick = 60.0 / app.project().tempo.bpm() / PPQ as f64;
    app.sync_engine();
    for c in clips.iter().filter(|c| !c.muted && c.start <= tick && tick < c.end()) {
        if app
            .project()
            .playlist
            .tracks
            .get(c.track as usize)
            .is_none_or(|t| t.muted)
        {
            continue;
        }
        let Some(channel) = c.audio_channel() else { continue };
        let Some(index) = app.project().channel_index(channel).and_then(|i| u16::try_from(i).ok()) else {
            continue;
        };
        let Some(ch) = app.project().channel(channel) else {
            continue;
        };
        let key = ch.root_key;
        app.send(limeshot_audio::Msg::PreviewClip {
            channel: index,
            playback: limeshot_dsp::ClipPlayback {
                offset_secs: c.offset as f64 * seconds_per_tick,
                elapsed_secs: (tick - c.start) as f64 * seconds_per_tick,
                remaining_secs: (c.end() - tick) as f64 * seconds_per_tick,
                fade_in_secs: (c.fade_in as f64 * seconds_per_tick) as f32,
                fade_out_secs: (c.fade_out as f64 * seconds_per_tick) as f32,
                fade_in_curve: c.fade_curve(false),
                fade_out_curve: c.fade_curve(true),
            },
        });
        app.ui.playlist.scrub_audio.push((channel, key));
    }
}

/// The notes of the (unmuted) pattern clips sounding at song position `tick`, for the Playback
/// tool's scrub. A pattern clip plays its pattern once from its offset (silent past the end).
fn notes_under(app: &LimeShotApp, clips: &[Clip], tick: Tick) -> Vec<(ChannelId, u8, f32)> {
    let p = app.project();
    let sig = p.time_signature;
    let mut out = Vec::new();
    for c in clips.iter().filter(|c| !c.muted && c.start <= tick && tick < c.end()) {
        if p.playlist.tracks.get(c.track as usize).is_some_and(|t| t.muted) {
            continue;
        }
        let Some(pat) = c.pattern().and_then(|id| p.pattern(id)) else {
            continue;
        };
        let local = tick - c.start + c.offset;
        if local >= pat.length(sig) {
            continue;
        }
        for n in pat
            .notes
            .iter()
            .filter(|n| !n.muted && n.start <= local && local < n.end())
        {
            if p.channel(n.channel).is_some_and(|ch| !ch.muted) {
                out.push((n.channel, n.key, n.velocity));
            }
        }
    }
    out
}

/// Handle Playlist pointer tools, automation edits, selection, and scrub auditions.
/// Apply clip and automation changes through commands grouped by the current drag gesture.
fn grid_input(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, clips: &[Clip], tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("pl_grid"), Sense::click_and_drag());
    let sig = app.project().time_signature;
    let (mods, pointer) = ui.input(|i| (i.modifiers, i.pointer.interact_pos()));
    let snap = if mods.alt { Snap::None } else { app.ui.snap };
    let tool = app.ui.playlist.tool;
    let show_fades = app.ui.playlist.audio_show_fades;
    app.ui.playlist.cursor = None;
    if let Some(p) = resp.hover_pos() {
        let auto_hit = (app.ui.playlist.drag.is_none())
            .then(|| automation_hit(app, view, clips, p))
            .flatten();
        let auto_icon = auto_hit.as_ref().map(|_| egui::CursorIcon::None);
        let auto_cursor = match app.ui.playlist.drag {
            Some(Drag::AutoPoint { .. } | Drag::AutoStep { .. }) => Some(0),
            Some(Drag::AutoTension { .. }) => Some(1),
            _ => auto_hit.map(|(_, _, h)| match h {
                AutoHit::Point(_) => 0,
                AutoHit::Tension(_) => 1,
                AutoHit::Curve(..) => 2,
            }),
        };
        let special = auto_icon.or(match (tool, hit(view, clips, p, show_fades).map(|h| h.1)) {
            (Tool::Draw | Tool::Paint, Some(Part::LeftEdge | Part::RightEdge | Part::FadeIn | Part::FadeOut)) => {
                Some(egui::CursorIcon::ResizeHorizontal)
            }
            (_, Some(Part::Menu)) => Some(egui::CursorIcon::PointingHand),
            (Tool::Draw | Tool::Paint | Tool::Select, Some(Part::Body)) => Some(egui::CursorIcon::Grab),
            _ => None,
        });
        let erase = ui.input(|i| i.pointer.secondary_down()) && auto_icon.is_none();
        let cursor = if erase { Tool::Delete } else { tool };
        if crate::piano_roll::tool_cursor(ui.ctx(), cursor, p, if erase { None } else { special }) {
            app.ui.playlist.cursor = Some(cursor);
        }
        if let Some(kind) = auto_cursor {
            automation_cursor(ui.ctx(), p, kind);
        }
        let tk = view.tick_at(p.x).max(0) as u64;
        let help = match tool {
            Tool::Draw => {
                "left: place/move/resize (either edge) · right: delete · double-click: edit · ▾: clip menu · Cmd+drag: select · drop samples here"
            }
            tool => tool_tip(tool),
        };
        app.hint(format!("{} — {help}", BarBeatTick::from_ticks(tk, sig)));
    }
    let primary = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
    let secondary =
        resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary));
    // Automation curves take presses on their points, tension handles and line first.
    let auto = pointer.and_then(|p| automation_hit(app, view, clips, p).map(|h| (p, h)));
    if let Some((_, (_, a, _))) = &auto
        && resp.hovered()
        && app.ui.playlist.drag.is_none()
    {
        let name = app.project().automation(*a).map(|a| a.name.clone()).unwrap_or_default();
        app.hint(format!(
            "{name} — drag a point · drag ○ between points: curve (or steps) · left/right-click the line: add a point · right-click a point: delete, point mode"
        ));
    }
    let mut primary = primary;
    let mut secondary = secondary;
    if (primary || secondary)
        && !mods.command
        && let Some((p, (clip, a, h))) = auto
    {
        focus_source(app, &clip);
        let points = app
            .project()
            .automation(a)
            .map(|a| a.points.clone())
            .unwrap_or_default();
        match (h, primary) {
            (AutoHit::Point(i), true) => {
                app.doc.begin_gesture();
                if app.ui.playlist.auto_step {
                    app.ui.playlist.drag = Some(Drag::AutoStep {
                        clip: clip.id,
                        automation: a,
                    });
                } else {
                    app.ui.playlist.drag = Some(Drag::AutoPoint {
                        clip: clip.id,
                        automation: a,
                        index: i,
                    });
                }
            }
            (AutoHit::Point(i), false) => {
                // FL Studio: a point's right-click menu (Delete and the point modes).
                let pass = ui.ctx().cumulative_pass_nr();
                app.ui.playlist.point_menu = Some((a, i, p, pass));
                app.ui.playlist.point_menu_curve = false;
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
            (AutoHit::Tension(i), false) => {
                app.ui.playlist.point_menu = Some((a, i, p, ui.ctx().cumulative_pass_nr()));
                app.ui.playlist.point_menu_curve = true;
            }
            (AutoHit::Curve(tick, value), left) => {
                let mut pts = points;
                let step_mode = app.ui.playlist.auto_step && left;
                let mode = if step_mode { PointMode::Hold } else { Default::default() };
                let index = pts.partition_point(|x| x.tick < tick);
                if let Some(existing) = pts.get_mut(index).filter(|x| x.tick == tick) {
                    existing.value = value;
                    if step_mode {
                        existing.mode = PointMode::Hold;
                    }
                } else {
                    pts.insert(
                        index,
                        AutomationPoint {
                            tick,
                            value,
                            tension: 0.0,
                            mode,
                        },
                    );
                }
                if left {
                    app.doc.begin_gesture();
                }
                app.exec(Command::SetAutomationPoints {
                    automation: a,
                    points: pts,
                });
                if left {
                    app.ui.playlist.drag = Some(if step_mode {
                        Drag::AutoStep {
                            clip: clip.id,
                            automation: a,
                        }
                    } else {
                        Drag::AutoPoint {
                            clip: clip.id,
                            automation: a,
                            index,
                        }
                    });
                }
            }
        }
        primary = false;
        secondary = false;
    }
    if primary
        && let Some(p) = pointer
        && !mods.command
        && !matches!(tool, Tool::Draw | Tool::Paint)
        && !matches!(hit(view, clips, p, show_fades), Some((_, Part::Menu)))
    {
        primary = false;
        match tool {
            Tool::Delete => {
                app.doc.begin_gesture();
                app.ui.playlist.drag = Some(Drag::Erase);
            }
            Tool::Mute => {
                if let Some((c, _)) = hit(view, clips, p, show_fades) {
                    app.doc.begin_gesture();
                    app.ui.playlist.drag = Some(Drag::Mute {
                        muted: !c.muted,
                        done: Vec::new(),
                    });
                }
            }
            Tool::Slip => {
                if let Some((c, _)) = hit(view, clips, p, show_fades) {
                    app.ui.playlist.selection = vec![c.id];
                    app.doc.begin_gesture();
                    app.ui.playlist.drag = Some(Drag::Slip {
                        id: c.id,
                        anchor: c.start,
                        offset: c.offset,
                        grab_tick: view.tick_at(p.x),
                    });
                }
            }
            Tool::Slice => app.ui.playlist.drag = Some(Drag::Slice { from: p }),
            Tool::Select => match hit(view, clips, p, show_fades) {
                Some((c, _)) => {
                    let st = &mut app.ui.playlist;
                    if mods.shift {
                        if let Some(i) = st.selection.iter().position(|x| *x == c.id) {
                            st.selection.remove(i);
                        } else {
                            st.selection.push(c.id);
                        }
                    } else {
                        if !st.selection.contains(&c.id) {
                            st.selection = vec![c.id];
                        }
                        let ids = st.selection.clone();
                        app.doc.begin_gesture();
                        app.ui.playlist.drag = Some(Drag::Move {
                            ids,
                            anchor_start: c.start,
                            grab_tick: view.tick_at(p.x),
                            grab_track: view.track_at(p.y),
                            applied_dt: 0,
                            applied_dtrack: 0,
                            duplicate: false,
                        });
                    }
                }
                None => {
                    if !mods.shift {
                        app.ui.playlist.selection.clear();
                    }
                    app.ui.playlist.drag = Some(Drag::Select { from: p });
                }
            },
            Tool::Zoom => app.ui.playlist.drag = Some(Drag::Zoom { from: p }),
            Tool::Playback => app.ui.playlist.drag = Some(Drag::Playback),
            Tool::Draw | Tool::Paint => {}
        }
    }
    if primary && let Some(p) = pointer {
        let st = &mut app.ui.playlist;
        if mods.command {
            match hit(view, clips, p, show_fades) {
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
            match hit(view, clips, p, show_fades) {
                Some((c, Part::Menu)) => {
                    st.menu = Some((c.id, p, ui.ctx().cumulative_pass_nr()));
                }
                Some((c, part)) => {
                    if !st.selection.contains(&c.id) {
                        st.selection = vec![c.id];
                    }
                    let ids = st.selection.clone();
                    focus_source(app, &c);
                    // Shift+drag moves copies and leaves the originals (FL Studio).
                    let duplicate = mods.shift && part == Part::Body;
                    let stretching =
                        part == Part::RightEdge && c.audio_channel().is_some() && app.ui.playlist.audio_stretch;
                    if duplicate
                        || stretching
                        || matches!(
                            part,
                            Part::FadeIn | Part::FadeOut | Part::FadeCurveIn | Part::FadeCurveOut
                        )
                    {
                        app.doc.begin_compound_gesture();
                    } else {
                        app.doc.begin_gesture();
                    }
                    let orig_pitch = c
                        .audio_channel()
                        .and_then(|ch| app.project().channel(ch))
                        .and_then(|ch| limeshot_core::lime::sampler_params(&ch.instrument))
                        .map(|sp| sp.pitch);
                    app.ui.playlist.drag = Some(match part {
                        Part::RightEdge => Drag::Resize {
                            id: c.id,
                            start: c.start,
                            orig_len: c.length,
                            orig_pitch,
                        },
                        Part::LeftEdge => Drag::ResizeStart { id: c.id },
                        Part::FadeCurveIn | Part::FadeCurveOut => Drag::FadeCurve {
                            out: part == Part::FadeCurveOut,
                            grab_y: p.y,
                            originals: clips
                                .iter()
                                .filter(|clip| {
                                    clip.audio_channel().is_some() && app.ui.playlist.selection.contains(&clip.id)
                                })
                                .cloned()
                                .collect(),
                        },
                        Part::FadeIn | Part::FadeOut => Drag::Fade {
                            id: c.id,
                            out: part == Part::FadeOut,
                            originals: clips
                                .iter()
                                .filter(|clip| {
                                    clip.audio_channel().is_some()
                                        && (clip.id == c.id
                                            || (app.ui.playlist.selection.contains(&c.id)
                                                && app.ui.playlist.selection.contains(&clip.id)))
                                })
                                .cloned()
                                .collect(),
                        },
                        _ => Drag::Move {
                            ids,
                            anchor_start: c.start,
                            grab_tick: view.tick_at(p.x),
                            grab_track: view.track_at(p.y),
                            applied_dt: 0,
                            applied_dtrack: 0,
                            duplicate,
                        },
                    });
                }
                None => {
                    let track = view.track_at(p.y);
                    if track >= 0 && (track as u32) < tracks {
                        let start = snap.floor(view.tick_at(p.x).max(0) as u64, sig);
                        let source = place_source(app);
                        // The length of the clip of this source last placed, resized or clicked.
                        let length = match app.ui.playlist.last_len {
                            Some((s, l)) if s == source => Some(l),
                            _ => full_length(app, source),
                        };
                        let cmd = match (source, length) {
                            (ClipSource::Pattern(pattern), length) => Some(Command::AddClip {
                                pattern,
                                track: track as u32,
                                start,
                                length,
                            }),
                            (source, Some(length)) => Some(Command::PlaceClip {
                                source,
                                track: track as u32,
                                start,
                                length,
                                offset: 0,
                            }),
                            (_, None) => None,
                        };
                        // Placing and dragging the new clip is one undo step.
                        app.doc.begin_compound_gesture();
                        if let Some(id) = cmd.and_then(|cmd| app.exec(cmd)) {
                            let id = ClipId(id);
                            app.ui.playlist.selection = vec![id];
                            let placed = app.project().clip(id).cloned();
                            app.ui.playlist.last_len = placed.as_ref().map(|c| (c.source, c.length));
                            app.ui.playlist.drag = Some(match placed {
                                // Paint: keep laying clips of this length end to end while dragging.
                                Some(c) if tool == Tool::Paint => Drag::Paint {
                                    source: c.source,
                                    track: c.track,
                                    next: c.end(),
                                    len: c.length,
                                    offset: c.offset,
                                },
                                _ => Drag::Move {
                                    ids: vec![id],
                                    anchor_start: start,
                                    grab_tick: view.tick_at(p.x),
                                    grab_track: track,
                                    applied_dt: 0,
                                    applied_dtrack: 0,
                                    duplicate: false,
                                },
                            });
                        } else {
                            app.doc.end_gesture();
                        }
                    }
                }
            }
        }
    }
    if secondary && tool == Tool::Zoom {
        let anchor = pointer.map_or(0.0, |p| p.x - grid.left());
        let st = &mut app.ui.playlist;
        let old = st.zoom_x;
        st.zoom_x = (st.zoom_x / 2.0).max(crate::playlist_menu::MIN_ZOOM);
        st.scroll.x = ((st.scroll.x + anchor) * st.zoom_x / old - anchor).max(0.0);
    } else if secondary {
        // Right-click deletes the clip under the pointer; on empty space it clears the
        // selection (FL Studio).
        if pointer.is_some_and(|p| hit(view, clips, p, show_fades).is_none()) {
            app.ui.playlist.selection.clear();
        }
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
                duplicate,
            }) => {
                let target = (*anchor_start as i64 + view.tick_at(p.x) - *grab_tick).max(0) as u64;
                let dt = snap.round(target, sig) as i64 - *anchor_start as i64;
                let dtrack = view.track_at(p.y) - *grab_track;
                if dt != *applied_dt || dtrack != *applied_dtrack {
                    if std::mem::take(duplicate) {
                        *ids = duplicate_in_place(app, ids);
                        app.ui.playlist.selection = ids.clone();
                    }
                    app.exec(Command::MoveClips {
                        clips: ids.clone(),
                        dt: dt - *applied_dt,
                        dtrack: dtrack - *applied_dtrack,
                    });
                    *applied_dt = dt;
                    *applied_dtrack = dtrack;
                }
            }
            Some(Drag::Resize {
                id,
                start,
                orig_len,
                orig_pitch,
            }) => {
                let end = snap.round(view.tick_at(p.x).max(0) as u64, sig);
                let min = snap.ticks(sig).unwrap_or(STEP_TICKS).max(1);
                let mut len = end.saturating_sub(*start).max(min);
                let clip_before = app.project().clip(*id).cloned();
                if let Some(c) = &clip_before
                    && let Some(ch) = c.audio_channel()
                    && app.ui.playlist.audio_zero_cross
                    && !app.ui.playlist.audio_stretch
                {
                    let snapped_local = snap_local_to_zero_crossing(app, ch, c.offset + len);
                    len = snapped_local.saturating_sub(c.offset).max(1);
                }
                if clip_before.as_ref().is_some_and(|c| c.length != len) {
                    app.exec(Command::ResizeClip { clip: *id, length: len });
                    if let Some(c) = &clip_before
                        && let Some(ch) = c.audio_channel()
                        && app.ui.playlist.audio_stretch
                        && let Some(pitch) = *orig_pitch
                    {
                        apply_audio_stretch(app, ch, pitch, *orig_len, len);
                    }
                    app.ui.playlist.last_len = app.project().clip(*id).map(|c| (c.source, c.length));
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
                    let slide = app.ui.playlist.auto_slide;
                    let lo = if *index == 0 { 0 } else { a.points[*index - 1].tick + 1 };
                    let hi = if slide {
                        Tick::MAX / 2
                    } else {
                        a.points.get(*index + 1).map_or(Tick::MAX, |x| x.tick.saturating_sub(1))
                    };
                    let tick = local.clamp(lo, hi.max(lo));
                    let value = ((area.bottom() - p.y) / area.height().max(1.0)).clamp(0.0, 1.0);
                    let mut pts = a.points.clone();
                    if let Some(old_pt) = pts.get(*index).copied()
                        && (old_pt.tick != tick || old_pt.value != value)
                    {
                        let dt = tick as i64 - old_pt.tick as i64;
                        pts[*index].tick = tick;
                        pts[*index].value = value;
                        if slide && dt != 0 {
                            for sub in &mut pts[*index + 1..] {
                                sub.tick = (sub.tick as i64 + dt).max(tick as i64 + 1) as Tick;
                            }
                        }
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
            Some(Drag::AutoStep { clip, automation }) => {
                if let (Some(c), Some(a)) = (
                    app.project().clip(*clip).cloned(),
                    app.project().automation(*automation).cloned(),
                ) {
                    let r = view.clip_rect(&c);
                    let area = curve_area(r);
                    let local = (view.tick_at(p.x) - c.start as i64 + c.offset as i64).max(0) as u64;
                    let step = snap.ticks(sig).unwrap_or(STEP_TICKS).max(1);
                    let tick = (local / step * step).clamp(c.offset, c.offset + c.length.saturating_sub(1));
                    let value = ((area.bottom() - p.y) / area.height().max(1.0)).clamp(0.0, 1.0);
                    let mut pts = a.points.clone();
                    let idx = pts.partition_point(|x| x.tick < tick);
                    let changed = if let Some(pt) = pts.get_mut(idx).filter(|x| x.tick == tick) {
                        if (pt.value - value).abs() > f32::EPSILON || pt.mode != PointMode::Hold {
                            pt.value = value;
                            pt.mode = PointMode::Hold;
                            true
                        } else {
                            false
                        }
                    } else {
                        pts.insert(
                            idx,
                            AutomationPoint {
                                tick,
                                value,
                                tension: 0.0,
                                mode: PointMode::Hold,
                            },
                        );
                        true
                    };
                    if changed {
                        app.exec(Command::SetAutomationPoints {
                            automation: *automation,
                            points: pts,
                        });
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
            Some(Drag::FadeCurve { out, originals, grab_y }) => {
                for clip in originals {
                    let tension = (clip.fade_curve(*out) + (*grab_y - p.y) / 80.0).clamp(-1.0, 1.0);
                    app.exec(Command::SetClipFadeCurve {
                        clip: clip.id,
                        out: *out,
                        tension,
                    });
                }
            }
            Some(Drag::Fade { id, out, originals }) => {
                if let Some(anchor) = originals.iter().find(|c| c.id == *id) {
                    let tick = view.tick_at(p.x).clamp(anchor.start as i64, anchor.end() as i64) as Tick;
                    let length = if *out { anchor.end() - tick } else { tick - anchor.start };
                    let previous = if *out { anchor.fade_out } else { anchor.fade_in };
                    let delta = length as i64 - previous as i64;
                    let commands: Vec<_> = originals
                        .iter()
                        .filter_map(|original| {
                            let c = app.project().clip(original.id)?;
                            let (fade_in, fade_out) = if *out {
                                let fade_out = (original.fade_out as i64 + delta).clamp(0, c.length as i64) as Tick;
                                (original.fade_in.min(c.length - fade_out), fade_out)
                            } else {
                                let fade_in = (original.fade_in as i64 + delta).clamp(0, c.length as i64) as Tick;
                                (fade_in, original.fade_out.min(c.length - fade_in))
                            };
                            ((fade_in, fade_out) != (c.fade_in, c.fade_out)).then_some(Command::SetClipFades {
                                clip: c.id,
                                fade_in,
                                fade_out,
                            })
                        })
                        .collect();
                    for command in commands {
                        app.exec(command);
                    }
                    app.hint(format!(
                        "Fade {}: {:.2} s",
                        if *out { "out" } else { "in" },
                        length as f64 * 60.0 / app.project().tempo.bpm() / PPQ as f64
                    ));
                }
            }
            Some(Drag::ResizeStart { id }) => {
                let mut start = snap.round(view.tick_at(p.x).max(0) as u64, sig);
                if let Some(c) = app.project().clip(*id).cloned()
                    && let Some(ch) = c.audio_channel()
                    && app.ui.playlist.audio_zero_cross
                {
                    let delta = start as i64 - c.start as i64;
                    let want_offset = (c.offset as i64 + delta).max(0) as Tick;
                    let snapped_offset = snap_local_to_zero_crossing(app, ch, want_offset);
                    start = (c.start as i64 + (snapped_offset as i64 - c.offset as i64)).max(0) as Tick;
                }
                if app.project().clip(*id).is_some_and(|c| c.start != start) {
                    app.exec(Command::ResizeClipStart { clip: *id, start });
                    app.ui.playlist.last_len = app.project().clip(*id).map(|c| (c.source, c.length));
                }
            }
            Some(Drag::Erase) => {
                let current = app.project().playlist.clips.clone();
                if let Some((c, _)) = hit(view, &current, p, show_fades) {
                    app.exec(Command::RemoveClips(vec![c.id]));
                    app.ui.playlist.selection.retain(|x| *x != c.id);
                }
            }
            Some(Drag::Select { from }) => {
                let r = Rect::from_two_pos(*from, p);
                for c in clips {
                    if !view.is_track_hidden(c.track)
                        && view.clip_rect(c).intersects(r)
                        && !app.ui.playlist.selection.contains(&c.id)
                    {
                        app.ui.playlist.selection.push(c.id);
                    }
                }
            }
            Some(Drag::Paint {
                source,
                track,
                next,
                len,
                offset,
            }) => {
                let tick = view.tick_at(p.x).max(0) as u64;
                while tick >= *next {
                    let cmd = Command::PlaceClip {
                        source: *source,
                        track: *track,
                        start: *next,
                        length: *len,
                        offset: *offset,
                    };
                    if let Some(id) = app.exec(cmd) {
                        app.ui.playlist.selection.push(ClipId(id));
                    }
                    *next += *len;
                }
            }
            Some(Drag::Mute { muted, done }) => {
                let current = app.project().playlist.clips.clone();
                if let Some((c, _)) = hit(view, &current, p, show_fades)
                    && !done.contains(&c.id)
                {
                    done.push(c.id);
                    if c.muted != *muted {
                        app.exec(Command::SetClipMuted {
                            clip: c.id,
                            muted: *muted,
                        });
                    }
                }
            }
            Some(Drag::Slip {
                id,
                anchor,
                offset,
                grab_tick,
            }) => {
                // The content moves by the snapped pointer travel; it cannot start before the
                // source does, but may run past its end (silence, as in FL Studio).
                let target = (*anchor as i64 + view.tick_at(p.x) - *grab_tick).max(0) as u64;
                let d = snap.round(target, sig) as i64 - *anchor as i64;
                let mut new = (*offset as i64 - d).max(0) as u64;
                if let Some(c) = app.project().clip(*id)
                    && let Some(ch) = c.audio_channel()
                    && app.ui.playlist.audio_zero_cross
                {
                    new = snap_local_to_zero_crossing(app, ch, new);
                }
                if app.project().clip(*id).is_some_and(|c| c.offset != new) {
                    app.exec(Command::SlipClip { clip: *id, offset: new });
                }
            }
            Some(Drag::Slice { .. } | Drag::Zoom { .. } | Drag::Playback | Drag::ResizeTrack { .. }) | None => {}
        }
        app.ui.playlist.drag = drag;
    }
    // The Playback tool's line and scrub (see `piano_roll::scrub`).
    if tool == Tool::Playback
        && let Some(p) = resp.hover_pos().or(pointer.filter(|_| app.ui.playlist.drag.is_some()))
    {
        let held = matches!(app.ui.playlist.drag, Some(Drag::Playback));
        crate::piano_roll::playback_line(ui, grid, p.x, held);
        if held {
            let tick = view.tick_at(p.x).max(0) as u64;
            scrub_audio(app, clips, Some(tick));
            let want = notes_under(app, clips, tick);
            crate::piano_roll::scrub(app, &want);
        }
    }
    // Slice and Zoom act when the button is released.
    if !ui.input(|i| i.pointer.any_down())
        && let Some(p) = pointer
    {
        match app.ui.playlist.drag.clone() {
            Some(Drag::Slice { from }) => {
                app.ui.playlist.drag = None;
                slice_at(app, view, from, p, snap, sig);
            }
            Some(Drag::Zoom { from }) => {
                app.ui.playlist.drag = None;
                let (t0, t1) = (
                    view.tick_at(from.x.min(p.x)).max(0),
                    view.tick_at(from.x.max(p.x)).max(0),
                );
                let st = &mut app.ui.playlist;
                if (p.x - from.x).abs() > 4.0 && t1 > t0 {
                    // Fit the dragged range to the grid width.
                    st.zoom_x = (grid.width() / (t1 - t0) as f32 * PPQ as f32)
                        .clamp(crate::playlist_menu::MIN_ZOOM, crate::playlist_menu::MAX_ZOOM);
                    st.scroll.x = t0 as f32 * st.zoom_x / PPQ as f32;
                } else {
                    let anchor = p.x - grid.left();
                    let old = st.zoom_x;
                    st.zoom_x = (st.zoom_x * 2.0).min(crate::playlist_menu::MAX_ZOOM);
                    st.scroll.x = ((st.scroll.x + anchor) * st.zoom_x / old - anchor).max(0.0);
                }
            }
            Some(Drag::Playback) => {
                app.ui.playlist.drag = None;
                crate::piano_roll::scrub(app, &[]);
                scrub_audio(app, clips, None);
            }
            _ => {}
        }
    }
    if app.ui.playlist.drag.is_some()
        && !matches!(app.ui.playlist.drag, Some(Drag::ResizeTrack { .. }))
        && !ui.input(|i| i.pointer.any_down())
    {
        if !matches!(app.ui.playlist.drag, Some(Drag::Select { .. })) {
            app.doc.end_gesture();
        }
        app.ui.playlist.drag = None;
    }
    if resp.double_clicked()
        && let Some(p) = pointer
        && let Some((c, part)) = hit(view, clips, p, show_fades)
        && part != Part::Menu
    {
        focus_source(app, &c);
        match c.source {
            ClipSource::Pattern(_) => app.show_window(crate::workspace::Win::PianoRoll),
            ClipSource::Audio(ch) => {
                app.select_channel(ch);
                app.plugins.open_window(crate::plugins::Target::Channel(ch), None);
            }
            ClipSource::Automation(_) => {}
        }
    }
}

/// Copies of `ids` at the same places (FL Studio's Shift+drag); returns the copies' ids.
fn duplicate_in_place(app: &mut LimeShotApp, ids: &[ClipId]) -> Vec<ClipId> {
    let before: Vec<ClipId> = app.project().playlist.clips.iter().map(|c| c.id).collect();
    let cmds = app
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| ids.contains(&c.id))
        .map(|c| Command::PlaceClip {
            source: c.source,
            track: c.track,
            start: c.start,
            length: c.length,
            offset: c.offset,
        })
        .collect();
    app.exec(Command::Batch(cmds));
    app.project()
        .playlist
        .clips
        .iter()
        .map(|c| c.id)
        .filter(|id| !before.contains(id))
        .collect()
}

/// The Slice tool: cut the clips on the tracks the drag from `from` to `to` crosses, at `from`'s
/// snapped time. The right-hand pieces end up selected.
fn slice_at(app: &mut LimeShotApp, view: &View, from: Pos2, to: Pos2, snap: Snap, sig: TimeSignature) {
    let at = snap.round(view.tick_at(from.x).max(0) as u64, sig);
    let (t0, t1) = {
        let (a, b) = (view.track_at(from.y), view.track_at(to.y));
        (a.min(b), a.max(b))
    };
    let zero_cross = app.ui.playlist.audio_zero_cross;
    let crossed: Vec<(ClipId, Tick)> = app
        .project()
        .playlist
        .clips
        .iter()
        .filter(|c| (t0..=t1).contains(&(c.track as i32)) && c.start < at && at < c.end())
        .map(|c| {
            let cut = match c.source {
                ClipSource::Audio(ch) if zero_cross => {
                    let local = snap_local_to_zero_crossing(app, ch, c.offset + (at - c.start));
                    (c.start + local.saturating_sub(c.offset)).clamp(c.start + 1, c.end().saturating_sub(1))
                }
                _ => at,
            };
            (c.id, cut)
        })
        .collect();
    if crossed.is_empty() {
        return;
    }
    let before: Vec<ClipId> = app.project().playlist.clips.iter().map(|c| c.id).collect();
    let cmds = crossed
        .into_iter()
        .map(|(clip, cut)| Command::SliceClips {
            clips: vec![clip],
            at: cut,
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

fn clip_menu(app: &mut LimeShotApp, ctx: &Context) {
    let Some((id, pos, opened)) = app.ui.playlist.menu else {
        return;
    };
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
                            .hint("Give this clip its own copy of the pattern")
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
                        close |= pattern_color_menu(app, ui, pid);
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
                            .hint("Give this clip its own copy of the automation")
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
                                        mode: Default::default(),
                                    }],
                                });
                            }
                            close = true;
                        }
                        if ui
                            .button(RichText::new("Delete automation").color(Theme::current().danger))
                            .hint("Delete the automation and all its clips")
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
    let fresh = ctx.cumulative_pass_nr() == opened;
    let clicked_outside = !fresh && ctx.input(|i| i.pointer.any_pressed()) && !area.response.contains_pointer();
    if close || clicked_outside || ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.ui.playlist.menu = None;
    }
}

/// The full length of a clip's source in ticks: its pattern, or its whole sample.
fn source_length(app: &LimeShotApp, c: &Clip) -> Option<Tick> {
    full_length(app, c.source)
}

/// The full length of `source` in ticks (for a new clip of it).
fn full_length(app: &LimeShotApp, source: ClipSource) -> Option<Tick> {
    match source {
        ClipSource::Pattern(p) => {
            let sig = app.project().time_signature;
            app.project().pattern(p).map(|p| p.length(sig))
        }
        ClipSource::Audio(ch) => {
            let (_, s, first, per_tick) = clip_audio(app, ch)?;
            let sp = limeshot_core::lime::sampler_params(&app.project().channel(ch)?.instrument)?;
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
    let (lo, hi) = TRACK_H;
    if crate::piano_roll::wheel_zoom_y(ui, grid, &mut st.track_h, &mut st.scroll.y, lo, hi) {
        return;
    }
    if crate::piano_roll::wheel_zoom(
        ui,
        grid,
        &mut st.zoom_x,
        &mut st.scroll.x,
        crate::playlist_menu::MIN_ZOOM,
        crate::playlist_menu::MAX_ZOOM,
    ) {
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

/// The Playlist's track height range (vertical zoom).
pub const TRACK_H: (f32, f32) = (12.0, 640.0);

/// Edit › Cut / Copy / Paste (and Cmd+X / C / V) in the Playlist: the selected clips; pasting
/// puts them at the song position in Song mode, else where they were.
pub fn clipboard(app: &mut LimeShotApp, op: ClipOp) {
    let sel = app.ui.playlist.selection.clone();
    if matches!(op, ClipOp::Cut | ClipOp::Copy) && !sel.is_empty() {
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
        if op == ClipOp::Cut {
            app.exec(Command::RemoveClips(sel.clone()));
            app.ui.playlist.selection.clear();
        }
    }
    if op == ClipOp::Paste
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
}

pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key;
    // The ⏵ menu's keys first: Cmd+Shift+V before Cmd+V, Alt+T before the T tool …
    crate::playlist_menu::keys(app, ctx);
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
        app.ui.playlist.time_sel = None;
    }
    for (k, op) in [(Key::X, ClipOp::Cut), (Key::C, ClipOp::Copy), (Key::V, ClipOp::Paste)] {
        if key(ctx, Modifiers::COMMAND, k) {
            clipboard(app, op);
        }
    }
    if key(ctx, Modifiers::COMMAND, Key::B) {
        crate::playlist_menu::run(app, crate::playlist_menu::Act::Duplicate);
    }
    if key(ctx, Modifiers::NONE, Key::Home) {
        app.locate(0);
    }
    for tool in Tool::PLAYLIST {
        if key(ctx, Modifiers::NONE, tool.key()) {
            app.ui.playlist.tool = tool;
        }
    }
    if key(ctx, Modifiers::SHIFT, Key::I) {
        // Invert selection (FL Studio's Playlist Select menu: Shift+I).
        app.ui.playlist.selection = app
            .project()
            .playlist
            .clips
            .iter()
            .map(|c| c.id)
            .filter(|id| !sel.contains(id))
            .collect();
    }
    // FL Studio's Playlist Edit menu: Shift+Left/Right/Up/Down shift the selected clips. Plain
    // arrows scroll the view, as in the Piano Roll.
    let sig = app.project().time_signature;
    let g = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS) as i64;
    for (k, dt, dtrack) in [
        (Key::ArrowLeft, -g, 0),
        (Key::ArrowRight, g, 0),
        (Key::ArrowUp, 0, -1),
        (Key::ArrowDown, 0, 1),
    ] {
        if key(ctx, Modifiers::SHIFT, k) && !sel.is_empty() {
            app.exec(Command::MoveClips {
                clips: sel.clone(),
                dt,
                dtrack,
            });
        }
    }
    // Checked after the Shift variants: a shortcut without modifiers also matches Shift.
    let (step_x, step_y) = (app.ui.playlist.zoom_x, app.ui.playlist.track_h);
    for (k, dx, dy) in [
        (Key::ArrowLeft, -step_x, 0.0),
        (Key::ArrowRight, step_x, 0.0),
        (Key::ArrowUp, 0.0, -step_y),
        (Key::ArrowDown, 0.0, step_y),
    ] {
        if key(ctx, Modifiers::NONE, k) {
            let st = &mut app.ui.playlist;
            st.scroll.x = (st.scroll.x + dx).max(0.0);
            st.scroll.y = (st.scroll.y + dy).max(0.0);
        }
    }
}

/// Preview a dragged clip source at a snapped grid position and place a linked clip on release.
/// Reuse its remembered length when available, otherwise its full length; select the new clip.
fn source_drop(app: &mut LimeShotApp, ui: &mut Ui, view: &View, grid: Rect, tracks: u32) {
    let resp = ui.interact(grid, ui.id().with("source-drop"), Sense::hover());
    let Some(source) = resp.dnd_hover_payload::<ClipSource>() else {
        return;
    };
    let Some(pos) = ui.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let track = view.track_at(pos.y).clamp(0, tracks.saturating_sub(1) as i32) as u32;
    let start = app
        .ui
        .snap
        .floor(view.tick_at(pos.x).max(0) as u64, app.project().time_signature);
    let Some(length) = (match app.ui.playlist.last_len {
        Some((s, l)) if s == *source => Some(l),
        _ => full_length(app, *source),
    }) else {
        return;
    };
    let (name, color) = source_name_color(app, *source);
    let r = Rect::from_min_max(
        pos2(view.x(start), view.y(track)),
        pos2(view.x(start + length), view.y(track) + view.track_height(track)),
    );
    let painter = ui.painter_at(grid);
    painter.rect_filled(r, 3.0, rgb(color).gamma_multiply(0.35));
    if let ClipSource::Audio(ch) = *source {
        let clip = Clip {
            id: ClipId(0),
            source: *source,
            track,
            start,
            length,
            offset: 0,
            muted: false,
            fade_in: 0,
            fade_out: 0,
            extra: Default::default(),
        };
        draw_waveform(
            app,
            &painter,
            view,
            grid,
            &clip,
            ch,
            r.shrink2(vec2(2.0, 13.0)),
            rgb(color),
        );
    }
    painter.text(
        r.min + vec2(5.0, 2.0),
        Align2::LEFT_TOP,
        name,
        mono(11.0),
        Theme::current().text,
    );
    if let Some(source) = resp.dnd_release_payload::<ClipSource>() {
        pick_source(app, *source);
        if let Some(id) = app.exec(Command::PlaceClip {
            source: *source,
            track,
            start,
            length,
            offset: 0,
        }) {
            app.ui.playlist.selection = vec![ClipId(id)];
        }
    }
}

/// Draw an automation cursor at `at`, hiding the system cursor.
/// `kind` is 0 for moving a point, 1 for tension, or 2 for placing a point.
fn automation_cursor(ctx: &Context, at: Pos2, kind: u8) {
    ctx.set_cursor_icon(egui::CursorIcon::None);
    let p = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("automation-cursor"),
    ));
    let t = Theme::current();
    let line = |a: Vec2, b: Vec2| {
        p.line_segment([at + a, at + b], Stroke::new(4.0, t.sunken));
        p.line_segment([at + a, at + b], Stroke::new(1.5, t.accent));
    };
    if kind == 2 {
        line(vec2(-6.0, 0.0), vec2(6.0, 0.0));
        line(vec2(0.0, -6.0), vec2(0.0, 6.0));
        p.circle_stroke(at, 3.0, Stroke::new(1.5, t.accent));
    } else {
        line(vec2(0.0, -10.0), vec2(0.0, 10.0));
        for sign in [-1.0, 1.0] {
            line(vec2(-3.0, sign * 6.0), vec2(0.0, sign * 10.0));
            line(vec2(3.0, sign * 6.0), vec2(0.0, sign * 10.0));
        }
        if kind == 0 {
            line(vec2(-10.0, 0.0), vec2(10.0, 0.0));
            for sign in [-1.0, 1.0] {
                line(vec2(sign * 6.0, -3.0), vec2(sign * 10.0, 0.0));
                line(vec2(sign * 6.0, 3.0), vec2(sign * 10.0, 0.0));
            }
        } else {
            p.circle_stroke(at, 4.0, Stroke::new(1.5, t.accent));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_fade_curve_handles_leave_clip_edges_resizable() {
        let tracks = vec![limeshot_core::PlaylistTrack::new("Track 1")];
        let view = View::new(
            Rect::from_min_size(Pos2::ZERO, vec2(500.0, 100.0)),
            1.0,
            60.0,
            Vec2::ZERO,
            &tracks,
        );
        let clip = Clip {
            id: ClipId(1),
            source: ClipSource::Audio(ChannelId(1)),
            track: 0,
            start: 10,
            length: 200,
            offset: 0,
            muted: false,
            fade_in: 1,
            fade_out: 1,
            extra: Default::default(),
        };
        for out in [false, true] {
            let handle = fade_curve_handle(&view, &clip, out).unwrap();
            let edge = if out {
                view.clip_rect(&clip).right() - 1.0
            } else {
                view.clip_rect(&clip).left() + 1.0
            };
            let (_, part) = hit(&view, std::slice::from_ref(&clip), pos2(edge, handle.y), true).unwrap();
            assert!(matches!((out, part), (false, Part::LeftEdge) | (true, Part::RightEdge)));
        }
    }
}
