//! The workspace: FL Studio's window model with a modern finish.
//!
//! Editors (Playlist, Channel Rack, Piano Roll, Mixer, Channel Settings) are windows inside the
//! workspace area. Like in FL Studio each can be moved, resized, maximised to fill the workspace,
//! closed and toggled with F5/F6/F7/F9, and **detached** into its own OS window (an egui
//! viewport; on backends without multi-window support it stays embedded). The Playlist starts
//! maximised, as FL Studio opens it.
//!
//! The title bars are drawn here (flat, ArtCraft-style): title, then pin, detach, maximise and
//! close buttons. Dragging the title bar moves the window; double-clicking it maximises or
//! restores the window (FL Studio). A press anywhere in a window brings it to the front, the
//! maximised Playlist included (FL Studio); a pinned window stays above the others.

use crate::app::{Focus, LimeShotApp};
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, semibold};
use crate::widgets::Hint as _;
use egui::{
    Align2, Context, CornerRadius, Id, LayerId, Order, Rect, Sense, Stroke, Ui, ViewportBuilder, ViewportClass,
    ViewportId, pos2, vec2,
};

pub const TITLE_H: f32 = 28.0;

/// A window layout saved in the settings (View › Arrange windows › Save custom layout).
/// Rectangles are fractions of the workspace, so the layout fits another screen size.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SavedLayout {
    pub windows: Vec<SavedWindow>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SavedWindow {
    /// The window's label ("Playlist").
    pub name: String,
    pub open: bool,
    pub maximized: bool,
    pub detached: bool,
    pub pinned: bool,
    /// Left, top, width, height as fractions of the workspace (when not maximised).
    pub rect: Option<[f32; 4]>,
}

/// The workspace windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Win {
    Playlist,
    ChannelRack,
    PianoRoll,
    Mixer,
    ChannelSettings,
}

impl Win {
    pub const ALL: [Win; 5] = [
        Win::Playlist,
        Win::ChannelRack,
        Win::PianoRoll,
        Win::Mixer,
        Win::ChannelSettings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Win::Playlist => "Playlist",
            Win::ChannelRack => "Channel Rack",
            Win::PianoRoll => "Piano Roll",
            Win::Mixer => "Mixer",
            Win::ChannelSettings => "Channel Settings",
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            Win::Playlist => "F5",
            Win::ChannelRack => "F6",
            Win::PianoRoll => "F7",
            Win::Mixer => "F9",
            Win::ChannelSettings => "",
        }
    }

    fn index(self) -> usize {
        match self {
            Win::Playlist => 0,
            Win::ChannelRack => 1,
            Win::PianoRoll => 2,
            Win::Mixer => 3,
            Win::ChannelSettings => 4,
        }
    }

    pub fn focus(self) -> Focus {
        match self {
            Win::Playlist => Focus::Playlist,
            Win::ChannelRack | Win::ChannelSettings => Focus::ChannelRack,
            Win::PianoRoll => Focus::PianoRoll,
            Win::Mixer => Focus::Mixer,
        }
    }

    pub fn id(self) -> Id {
        Id::new(("limeshot-window", self.index()))
    }

    /// Default placement inside the workspace `ws`, as fractions of it (FL-like cascade).
    fn default_rect(self, ws: Rect) -> Rect {
        let at = |x: f32, y: f32, w: f32, h: f32| {
            let size = vec2((ws.width() * w).max(320.0), (ws.height() * h).max(220.0));
            Rect::from_min_size(ws.min + vec2(ws.width() * x, ws.height() * y), size).intersect(ws)
        };
        match self {
            Win::Playlist => at(0.02, 0.04, 0.9, 0.85),
            // Sized to show 16 steps (FL's default rack width).
            Win::ChannelRack => {
                Rect::from_min_size(ws.min + vec2(ws.width() * 0.03, ws.height() * 0.05), vec2(700.0, 330.0))
                    .intersect(ws)
            }
            Win::PianoRoll => at(0.10, 0.14, 0.82, 0.72),
            Win::Mixer => at(0.06, 0.30, 0.9, 0.66),
            Win::ChannelSettings => at(0.52, 0.08, 0.34, 0.62),
        }
    }

    pub fn min_size(self) -> egui::Vec2 {
        match self {
            Win::ChannelSettings => vec2(300.0, 260.0),
            // The master and two strips next to the full-width track panel (its EQ knobs), and
            // a whole strip from its name down to the route button.
            Win::Mixer => vec2(560.0, 372.0),
            _ => vec2(360.0, 200.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WinState {
    pub open: bool,
    pub maximized: bool,
    pub detached: bool,
    /// Kept above the other (unpinned) windows.
    pub pinned: bool,
}

pub struct Workspace {
    states: [WinState; 5],
    /// The workspace rectangle of the last frame.
    pub area: Rect,
    /// Put every window back at its default place on the next frame.
    pub reset_layout: bool,
    /// Bring this window to the front on the next frame.
    raise: Option<Win>,
    front: Option<Win>,
    /// Each window's last floating (not maximised) rectangle, to restore after maximising.
    normal_rect: [Option<Rect>; 5],
    /// Put the window back at `normal_rect` on the next frame (it was just restored).
    restore: [bool; 5],
    /// The size each window's content needs, reported by its body (Channel Rack: every step
    /// and five channels), on top of `Win::min_size`.
    content_min: [Option<egui::Vec2>; 5],
}

impl Default for Workspace {
    /// Open a maximized Playlist with the Channel Rack in front, requesting the initial layout.
    fn default() -> Self {
        let mut states = [WinState::default(); 5];
        // FL Studio's new-project layout: the Playlist maximised and the Channel Rack on top.
        states[Win::Playlist.index()] = WinState {
            open: true,
            maximized: true,
            ..WinState::default()
        };
        states[Win::ChannelRack.index()] = WinState {
            open: true,
            ..WinState::default()
        };
        Self {
            states,
            area: Rect::NOTHING,
            reset_layout: true,
            raise: Some(Win::ChannelRack),
            front: Some(Win::ChannelRack),
            normal_rect: [None; 5],
            restore: [false; 5],
            content_min: [None; 5],
        }
    }
}

impl Workspace {
    pub fn state(&self, w: Win) -> WinState {
        self.states[w.index()]
    }

    pub fn state_mut(&mut self, w: Win) -> &mut WinState {
        &mut self.states[w.index()]
    }

    pub fn is_open(&self, w: Win) -> bool {
        self.state(w).open
    }

    /// Open (or keep open) and bring to the front.
    pub fn show(&mut self, w: Win) {
        self.state_mut(w).open = true;
        self.raise = Some(w);
        self.front = Some(w);
    }

    /// F5/F6/F7/F9: close an open window if it is frontmost or pinned; otherwise open and raise it.
    pub fn toggle(&mut self, w: Win) {
        if self.is_open(w) && (self.front == Some(w) || self.state(w).pinned) {
            self.state_mut(w).open = false;
        } else {
            self.show(w);
        }
    }

    pub fn close(&mut self, w: Win) {
        self.state_mut(w).open = false;
    }

    /// A window body reports the size its content needs; the window is never smaller.
    pub fn set_content_min(&mut self, w: Win, size: egui::Vec2) {
        self.content_min[w.index()] = Some(size);
    }

    /// The smallest size of `w`'s window: its own minimum, what its content needs, at most the
    /// workspace.
    pub fn min_size(&self, w: Win) -> egui::Vec2 {
        let min = self.content_min[w.index()].map_or(w.min_size(), |c| c.max(w.min_size()));
        if self.area.is_positive() {
            min.min(self.area.size())
        } else {
            min
        }
    }

    /// View › Close all windows (F12).
    pub fn close_all(&mut self) {
        for s in &mut self.states {
            s.open = false;
        }
    }

    /// View › Close all unfocused windows (Cmd+F12): every window but `keep`.
    pub fn close_all_but(&mut self, keep: Option<Win>) {
        for w in Win::ALL {
            if Some(w) != keep {
                self.close(w);
            }
        }
    }

    /// View › Arrange windows › Into workspace: put detached windows back into the main window.
    pub fn into_workspace(&mut self) {
        for s in &mut self.states {
            s.detached = false;
        }
    }

    /// View › Arrange windows › Desktop (default): FL Studio's new-project layout.
    pub fn default_layout(&mut self) {
        let area = self.area;
        *self = Workspace {
            area,
            ..Workspace::default()
        };
    }

    /// The current layout, to keep in the settings.
    pub fn save_layout(&self) -> SavedLayout {
        let area = self.area;
        SavedLayout {
            windows: Win::ALL
                .iter()
                .map(|&w| {
                    let s = self.state(w);
                    let rect = self.normal_rect[w.index()].filter(|_| area.is_positive()).map(|r| {
                        [
                            (r.min.x - area.min.x) / area.width(),
                            (r.min.y - area.min.y) / area.height(),
                            r.width() / area.width(),
                            r.height() / area.height(),
                        ]
                    });
                    SavedWindow {
                        name: w.label().into(),
                        open: s.open,
                        maximized: s.maximized,
                        detached: s.detached,
                        pinned: s.pinned,
                        rect,
                    }
                })
                .collect(),
        }
    }

    /// Put the windows where a saved layout has them (on the next frame).
    pub fn apply_layout(&mut self, layout: &SavedLayout) {
        let area = self.area;
        for saved in &layout.windows {
            let Some(w) = Win::ALL.into_iter().find(|w| w.label() == saved.name) else {
                continue;
            };
            *self.state_mut(w) = WinState {
                open: saved.open,
                maximized: saved.maximized,
                detached: saved.detached,
                pinned: saved.pinned,
            };
            let ok = |v: f32| v.is_finite() && (-1.0..=2.0).contains(&v);
            if let Some([x, y, wd, ht]) = saved.rect
                && area.is_positive()
                && [x, y, wd, ht].into_iter().all(ok)
            {
                let r = Rect::from_min_size(
                    area.min + vec2(x * area.width(), y * area.height()),
                    vec2(wd * area.width(), ht * area.height()),
                );
                self.normal_rect[w.index()] = Some(r);
                self.restore[w.index()] = !saved.maximized;
            }
        }
    }

    /// The maximise/restore button: restoring puts the window back where it was before.
    pub fn toggle_maximized(&mut self, w: Win) {
        let s = self.state_mut(w);
        s.maximized = !s.maximized;
        let restored = !s.maximized;
        self.restore[w.index()] = restored;
    }
}

#[derive(Default)]
pub(crate) struct TitleActions {
    pub minimize: bool,
    pub close: bool,
    maximize: bool,
    detach: bool,
    pub pin: bool,
}

/// A flat title bar with the window buttons; returns what was clicked.
pub(crate) fn title_bar(ui: &mut Ui, title: &str, state: WinState, detached: bool, plugin: bool) -> TitleActions {
    let t = Theme::current();
    // Senses clicks for the double-click; drags still move the window.
    let (r, title_resp) = ui.allocate_exact_size(vec2(ui.available_width(), TITLE_H), Sense::click());
    ui.painter().rect_filled(
        r,
        CornerRadius {
            nw: t.radius + 1,
            ne: t.radius + 1,
            sw: 0,
            se: 0,
        },
        t.header,
    );
    ui.painter()
        .line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0, t.border));
    ui.painter().text(
        r.left_center() + vec2(12.0, 0.0),
        Align2::LEFT_CENTER,
        title.to_uppercase(),
        semibold(10.5),
        t.text_dim,
    );
    let mut actions = TitleActions::default();
    let mut x = r.right() - 4.0;
    let mut button = |ui: &mut Ui, icon: Icon, tip: &str, on: bool| {
        x -= 22.0;
        let br = Rect::from_min_size(pos2(x, r.top() + 3.0), vec2(22.0, TITLE_H - 6.0));
        let resp = ui
            .interact(br, ui.id().with(("title-btn", title, tip)), Sense::click())
            .hint(tip);
        if resp.hovered() {
            ui.painter()
                .rect_filled(br, CornerRadius::same(t.radius_sm), t.raised_hover);
        }
        icons::paint(
            ui.painter(),
            br.shrink(5.0),
            icon,
            if on {
                t.accent
            } else if resp.hovered() {
                t.text
            } else {
                t.text_dim
            },
        );
        resp.clicked()
    };
    actions.close = button(ui, Icon::Close, "Close", false);
    if plugin {
        actions.minimize = button(ui, Icon::Minimize, "Minimize", false);
    }
    if !detached && !plugin {
        actions.maximize = button(
            ui,
            if state.maximized { Icon::Restore } else { Icon::Maximize },
            if state.maximized { "Restore" } else { "Maximise" },
            false,
        );
        // Double-clicking the title maximises or restores, like the button (FL Studio).
        actions.maximize |= title_resp.double_clicked();
    }
    if !plugin {
        actions.detach = button(
            ui,
            if detached { Icon::Attach } else { Icon::Detach },
            if detached {
                "Attach to the main window"
            } else {
                "Detach into its own window"
            },
            false,
        );
    }
    if !detached {
        actions.pin = button(
            ui,
            Icon::Pin,
            if state.pinned {
                "Unpin (let other windows cover it)"
            } else {
                "Pin: keep this window on top"
            },
            state.pinned,
        );
    }
    actions
}

/// Draw every open window. `body` draws a window's content.
pub fn show(app: &mut LimeShotApp, ctx: &Context, area: Rect, body: fn(&mut LimeShotApp, &mut Ui, Win)) {
    app.ui.workspace.area = area;
    let reset = std::mem::take(&mut app.ui.workspace.reset_layout);
    let raise = app.ui.workspace.raise.take();
    let t = Theme::current();
    for w in Win::ALL {
        let st = app.ui.workspace.state(w);
        if !st.open {
            continue;
        }
        let title = window_title(app, w);
        if st.detached {
            show_detached(app, ctx, w, &title, body);
            continue;
        }
        let frame = egui::Frame::new()
            .fill(t.surface)
            .stroke(Stroke::new(1.0, mix(t.border, t.text_faint, 0.3)))
            .corner_radius(CornerRadius::same(t.radius + 1))
            .shadow(egui::epaint::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: egui::Color32::from_black_alpha(150),
            });
        let mut win = egui::Window::new(&title)
            .id(w.id())
            .title_bar(false)
            .frame(frame)
            .resizable(!st.maximized)
            .movable(!st.maximized)
            .min_size(app.ui.workspace.min_size(w))
            .constrain_to(area);
        if w == Win::ChannelRack {
            win = win.max_width(app.ui.workspace.min_size(w).x + 16.0 * 22.0);
        }
        if st.maximized {
            let rect = if w == Win::ChannelRack {
                Rect::from_min_size(
                    area.min,
                    vec2(
                        area.width().min(app.ui.workspace.min_size(w).x + 16.0 * 22.0),
                        area.height(),
                    ),
                )
            } else {
                area
            };
            win = win.fixed_rect(rect);
        } else {
            let def = w.default_rect(area);
            win = win.default_rect(def);
            if reset {
                win = win.current_pos(def.min);
            }
            // Restoring after maximise: egui kept the maximised size, so set the old rectangle
            // for one frame (a window that started maximised goes to its default place).
            if std::mem::take(&mut app.ui.workspace.restore[w.index()]) {
                let back = app.ui.workspace.normal_rect[w.index()].unwrap_or(def);
                // Not constrained this frame: egui would clamp with the maximised size.
                win = win.fixed_rect(back.intersect(area)).constrain(false);
            }
        }
        let resp = win.show(ctx, |ui| {
            let actions = title_bar(ui, &title, st, false, false);
            // The body gets exactly the window's area: a row wider than that is clipped instead
            // of growing the window. (egui windows grow to their widest content, and a row whose
            // width follows the window — a slider row with a long value — grew it every frame
            // until it filled the workspace and could no longer be narrowed.)
            let area = ui.available_rect_before_wrap();
            let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(*ui.layout()));
            inner.set_clip_rect(area.intersect(ui.clip_rect()));
            body(app, &mut inner, w);
            ui.advance_cursor_after_rect(area);
            actions
        });
        if let Some(r) = resp {
            if !st.maximized {
                app.ui.workspace.normal_rect[w.index()] = Some(r.response.rect);
            }
            // A press anywhere in the window (not covered by another) brings it to the front.
            if r.response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed()) {
                app.ui.focus = w.focus();
                if !st.pinned {
                    app.ui.workspace.front = Some(w);
                }
                ctx.move_to_top(r.response.layer_id);
            }
            if let Some(a) = r.inner {
                apply(app, w, a);
            }
        }
        if raise == Some(w) {
            ctx.move_to_top(LayerId::new(Order::Middle, w.id()));
            app.ui.focus = w.focus();
        }
    }
    // Pinned windows stay above the others.
    for w in Win::ALL {
        let st = app.ui.workspace.state(w);
        if st.open && st.pinned && !st.detached {
            ctx.move_to_top(LayerId::new(Order::Middle, w.id()));
        }
    }
}

fn apply(app: &mut LimeShotApp, w: Win, a: TitleActions) {
    if a.maximize {
        app.ui.workspace.toggle_maximized(w);
    }
    let s = app.ui.workspace.state_mut(w);
    if a.pin {
        s.pinned = !s.pinned;
    }
    if a.close {
        s.open = false;
    }
    if a.detach {
        s.detached = !s.detached;
        s.maximized = false;
    }
}

fn show_detached(app: &mut LimeShotApp, ctx: &Context, w: Win, title: &str, body: fn(&mut LimeShotApp, &mut Ui, Win)) {
    let t = Theme::current();
    let size = w.default_rect(app.ui.workspace.area).size().max(w.min_size());
    let builder = ViewportBuilder::default()
        .with_title(format!("{title} — LimeShot"))
        .with_inner_size(size)
        .with_min_inner_size(w.min_size());
    let title = title.to_string();
    let actions = ctx.show_viewport_immediate(
        ViewportId::from_hash_of(("limeshot-detached", w.index())),
        builder,
        |ui, class| {
            let vctx = ui.ctx().clone();
            // Shortcuts work in detached windows too.
            if class != ViewportClass::EmbeddedWindow {
                crate::shortcuts::handle(app, &vctx);
            }
            let closing = vctx.input(|i| i.viewport().close_requested());
            let mut actions = TitleActions::default();
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(t.surface))
                .show(ui, |ui| {
                    actions = title_bar(ui, &title, app.ui.workspace.state(w), true, false);
                    if ui.rect_contains_pointer(ui.max_rect()) && vctx.input(|i| i.pointer.any_pressed()) {
                        app.ui.focus = w.focus();
                    }
                    body(app, ui, w);
                });
            actions.close |= closing;
            actions
        },
    );
    apply(app, w, actions);
}

fn window_title(app: &LimeShotApp, w: Win) -> String {
    match w {
        Win::Playlist => {
            let cur = app.current_pattern();
            let name = app.project().pattern(cur).map(|p| p.name.clone()).unwrap_or_default();
            format!("Playlist  ·  Arrangement  ›  {name}")
        }
        Win::ChannelSettings => match app.selected_channel().and_then(|c| app.project().channel(c)) {
            Some(c) => format!("{}  ·  {}", c.name, c.instrument.kind_label()),
            None => "Channel Settings".into(),
        },
        Win::PianoRoll => {
            let ch = app
                .selected_channel()
                .and_then(|c| app.project().channel(c))
                .map(|c| c.name.clone())
                .unwrap_or_default();
            format!("Piano Roll  ·  {ch}")
        }
        _ => w.label().to_string(),
    }
}
