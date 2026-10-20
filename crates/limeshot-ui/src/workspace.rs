//! The workspace: FL Studio's window model with a modern finish.
//!
//! Editors (Playlist, Channel Rack, Piano Roll, Mixer, Channel Settings) are windows inside the
//! workspace area. Like in FL Studio each can be moved, resized, maximised to fill the workspace,
//! closed and toggled with F5/F6/F7/F9, and **detached** into its own OS window (an egui
//! viewport; on backends without multi-window support it stays embedded). The Playlist starts
//! maximised, as FL Studio opens it.
//!
//! The title bars are drawn here (flat, ArtCraft-style): title, then detach, maximise and close
//! buttons. Dragging the title bar moves the window.

use crate::app::{Focus, LimeShotApp};
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, semibold};
use egui::{
    Align2, Context, CornerRadius, Id, LayerId, Order, Rect, Sense, Stroke, Ui, ViewportBuilder, ViewportClass,
    ViewportId, pos2, vec2,
};

pub const TITLE_H: f32 = 28.0;

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
            Win::Mixer => vec2(420.0, 300.0),
            _ => vec2(360.0, 200.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WinState {
    pub open: bool,
    pub maximized: bool,
    pub detached: bool,
}

pub struct Workspace {
    states: [WinState; 5],
    /// The workspace rectangle of the last frame.
    pub area: Rect,
    /// Put every window back at its default place on the next frame.
    pub reset_layout: bool,
    /// Bring this window to the front on the next frame.
    raise: Option<Win>,
}

impl Default for Workspace {
    fn default() -> Self {
        let mut states = [WinState::default(); 5];
        // FL Studio's new-project layout: the Playlist maximised and the Channel Rack on top.
        states[Win::Playlist.index()] = WinState {
            open: true,
            maximized: true,
            detached: false,
        };
        states[Win::ChannelRack.index()] = WinState {
            open: true,
            maximized: false,
            detached: false,
        };
        Self {
            states,
            area: Rect::NOTHING,
            reset_layout: true,
            raise: Some(Win::ChannelRack),
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
    }

    /// F5/F6/F7/F9: open and raise a closed window, close an open one.
    pub fn toggle(&mut self, w: Win) {
        if self.is_open(w) {
            self.state_mut(w).open = false;
        } else {
            self.show(w);
        }
    }

    pub fn close(&mut self, w: Win) {
        self.state_mut(w).open = false;
    }
}

#[derive(Default)]
struct TitleActions {
    close: bool,
    maximize: bool,
    detach: bool,
}

/// A flat title bar with the window buttons; returns what was clicked.
fn title_bar(ui: &mut Ui, title: &str, state: WinState, detached: bool) -> TitleActions {
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), TITLE_H), Sense::hover());
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
    let mut button = |ui: &mut Ui, icon: Icon, tip: &str| {
        x -= 22.0;
        let br = Rect::from_min_size(pos2(x, r.top() + 3.0), vec2(22.0, TITLE_H - 6.0));
        let resp = ui
            .interact(br, ui.id().with(("title-btn", title, tip)), Sense::click())
            .on_hover_text(tip);
        if resp.hovered() {
            ui.painter()
                .rect_filled(br, CornerRadius::same(t.radius_sm), t.raised_hover);
        }
        icons::paint(
            ui.painter(),
            br.shrink(5.0),
            icon,
            if resp.hovered() { t.text } else { t.text_dim },
        );
        resp.clicked()
    };
    actions.close = button(ui, Icon::Close, "Close");
    if !detached {
        actions.maximize = button(
            ui,
            if state.maximized { Icon::Restore } else { Icon::Maximize },
            if state.maximized { "Restore" } else { "Maximise" },
        );
    }
    actions.detach = button(
        ui,
        if detached { Icon::Attach } else { Icon::Detach },
        if detached {
            "Attach to the main window"
        } else {
            "Detach into its own window"
        },
    );
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
            .min_size(w.min_size())
            .constrain_to(area);
        if st.maximized {
            win = win.fixed_rect(area);
            // A maximised Playlist is the backdrop the other windows float over (FL Studio);
            // other maximised windows stay in the normal stacking order.
            if w == Win::Playlist {
                win = win.order(Order::Background);
            }
        } else {
            let def = w.default_rect(area);
            win = win.default_rect(def);
            if reset {
                win = win.current_pos(def.min);
            }
        }
        let resp = win.show(ctx, |ui| {
            let actions = title_bar(ui, &title, st, false);
            body(app, ui, w);
            actions
        });
        if let Some(r) = resp {
            if r.response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed()) {
                app.ui.focus = w.focus();
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
}

fn apply(app: &mut LimeShotApp, w: Win, a: TitleActions) {
    let s = app.ui.workspace.state_mut(w);
    if a.close {
        s.open = false;
    }
    if a.maximize {
        s.maximized = !s.maximized;
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
                    actions = title_bar(ui, &title, app.ui.workspace.state(w), true);
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
