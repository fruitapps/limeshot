//! The Mixer window: master plus insert tracks. Channels route to inserts (Channel Rack's
//! mixer field); inserts sum into the master. Each strip has a meter, fader, pan, mute and solo.
//! Mute: left-click · Solo: right-click or Cmd+click (FL Studio convention). Clicking a strip's
//! name or FX area selects it; the panel on the right shows its level, routing and FX slots (as FL Studio's
//! Mixer does), and its built-in 3-band EQ: drag a band's handle in the graph left/right for
//! its frequency and up/down for its gain, right-drag left/right for its width (FL Studio).

use crate::app::LimeShotApp;
use crate::theme::{Theme, mix, mono, rgb, semibold};
use crate::widgets::{fader, knob, meter, panel_header, reset_on_double_click as reset, section, toggle_label};
use egui::{CornerRadius, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use limeshot_core::{AutomationTarget, Command, EqBand};

const STRIP_W: f32 = 68.0;
/// The track panel's width.
const SIDE_W: f32 = 300.0;

#[derive(Default)]
pub struct State {
    pub selected: u16,
    /// Route buttons drawn in the last frame: (strip, rectangle). For tests and tools.
    pub route_buttons: Vec<(u16, Rect)>,
    /// The EQ graph drawn in the last frame.
    pub eq_graph: Option<Rect>,
    /// The EQ band being dragged in the graph.
    eq_drag: Option<usize>,
    /// A track name being typed in place (strip or track panel).
    pub rename: Option<Rename>,
    /// Strip name labels drawn in the last frame: (strip, rectangle).
    pub name_rects: Vec<(u16, Rect)>,
    /// The track panel's name label drawn in the last frame.
    pub panel_name: Option<Rect>,
    /// Send-level knobs drawn in the last frame: (strip, rectangle).
    pub send_knobs: Vec<(u16, Rect)>,
}

/// An inline rename of a mixer track's name.
pub struct Rename {
    pub track: u16,
    pub text: String,
    /// Edited in the track panel (else in the strip).
    pub in_panel: bool,
    /// Give the text field the keyboard focus on its first frame.
    focus: bool,
}

impl State {
    /// Start typing a new name for `track` where its name sits.
    pub fn start_rename(&mut self, track: u16, name: &str, in_panel: bool) {
        self.rename = Some(Rename {
            track,
            text: name.to_string(),
            in_panel,
            focus: true,
        });
    }
}

/// Send levels and insert faders share this travel: linear gain up to +6 dB, shaped so 0 dB
/// sits at about 80 % (as FL Studio's faders and send knobs).
pub const MAX_GAIN: f32 = 2.0;

/// Gain → knob position 0–1 (the fader's cube-root curve).
pub fn gain_pos(g: f32) -> f32 {
    (g / MAX_GAIN).max(0.0).cbrt().min(1.0)
}

fn pos_gain(p: f32) -> f32 {
    p.clamp(0.0, 1.0).powi(3) * MAX_GAIN
}

pub fn panel(app: &mut LimeShotApp, ui: &mut Ui) {
    app.ui.mixer.route_buttons.clear();
    app.ui.mixer.name_rects.clear();
    app.ui.mixer.send_knobs.clear();
    let n = app.project().mixer.tracks.len();
    if app.ui.mixer.rename.as_ref().is_some_and(|r| usize::from(r.track) >= n) {
        app.ui.mixer.rename = None;
    }
    if app.meter_hold.len() != n {
        app.meter_hold = vec![(0.0, 0.0); n];
    }
    // Meters fall back at about 20 dB/s.
    let dt = ui.input(|i| i.stable_dt).min(0.1);
    let fall = limeshot_dsp::db_to_gain(-20.0 * dt);
    for i in 0..n {
        let (l, r) = app.player.status().peak(i);
        if let Some(h) = app.meter_hold.get_mut(i) {
            h.0 = l.max(h.0 * fall);
            h.1 = r.max(h.1 * fall);
        }
    }
    // FL Studio layout: strips on the left, the selected track's FX slots on the right.
    let full = ui.available_rect_before_wrap();
    // The children below don't advance this cursor; without claiming the space the window frame
    // shrinks to the title bar and the windows behind show through the Mixer's body.
    ui.advance_cursor_after_rect(full);
    // The track panel keeps its full width (its EQ knobs need it); the window's minimum size
    // leaves room for it next to the master and two strips.
    let side_w = SIDE_W.min(full.width() - STRIP_W);
    let strips_rect = Rect::from_min_max(full.min, pos2(full.right() - side_w, full.bottom()));
    let side_rect = Rect::from_min_max(pos2(full.right() - side_w + 1.0, full.top()), full.max);
    let t = Theme::current();
    ui.painter().line_segment(
        [side_rect.left_top(), side_rect.left_bottom()],
        Stroke::new(1.0, t.border),
    );
    let mut side_ui = ui.new_child(egui::UiBuilder::new().id_salt("mixer-track-panel").max_rect(side_rect));
    track_panel(app, &mut side_ui);
    let mut strips_ui = ui.new_child(egui::UiBuilder::new().id_salt("mixer-strips").max_rect(strips_rect));
    let ui = &mut strips_ui;
    // A thin bar in its own gutter under the strips, so it never covers their colour bars.
    ui.spacing_mut().scroll = crate::theme::gutter_scroll_style();
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let height = ui.available_height() - 12.0;
            ui.add_space(6.0);
            ui.horizontal_top(|ui| {
                ui.add_space(8.0);
                ui.spacing_mut().item_spacing.x = 4.0;
                for i in 0..n {
                    strip(app, ui, i as u16, height);
                    if i == 0 {
                        ui.add_space(8.0);
                    }
                }
                if ui
                    .add_sized(vec2(28.0, 28.0), egui::Button::new("+"))
                    .on_hover_text("Add a mixer track")
                    .clicked()
                {
                    app.exec(Command::AddMixerTrack);
                }
            });
        });
}

/// Channels routed to track `i`, with their colors.
fn routed(app: &LimeShotApp, i: u16) -> Vec<(String, egui::Color32)> {
    app.project()
        .channels
        .iter()
        .filter(|c| c.mixer_track == i)
        .map(|c| (c.name.clone(), rgb(c.color)))
        .collect()
}

fn strip(app: &mut LimeShotApp, ui: &mut Ui, i: u16, height: f32) {
    let t = Theme::current();
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        return;
    };
    let any_solo = app.project().mixer.tracks.iter().skip(1).any(|t| t.solo);
    let audible = !track.muted && (i == 0 || !any_solo || track.solo);
    let selected = app.ui.mixer.selected == i;
    let routed = routed(app, i);
    let color = routed
        .first()
        .map(|r| r.1)
        .unwrap_or(if i == 0 { t.accent } else { t.text_faint });
    let (rect, _) = ui.allocate_exact_size(vec2(STRIP_W, height.max(220.0)), Sense::hover());
    let p = ui.painter();
    p.rect_filled(
        rect,
        CornerRadius::same(t.radius),
        if selected {
            mix(t.raised, t.accent, 0.07)
        } else {
            mix(t.surface, t.raised, 0.55)
        },
    );
    if selected {
        p.rect_stroke(
            rect,
            CornerRadius::same(t.radius),
            Stroke::new(1.0, t.accent_dim),
            egui::StrokeKind::Inside,
        );
    }
    // Colour bar at the bottom identifies the routed channel(s).
    p.rect_filled(
        Rect::from_min_max(
            pos2(rect.left() + 6.0, rect.bottom() - 4.0),
            pos2(rect.right() - 6.0, rect.bottom() - 2.0),
        ),
        1.0,
        color,
    );
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(5.0, 6.0)))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    let ui = &mut ui;
    ui.spacing_mut().item_spacing.y = 5.0;
    ui.label(
        RichText::new(if i == 0 { "M".to_string() } else { i.to_string() })
            .font(mono(10.0))
            .color(t.text_faint),
    );
    if !name_editor(app, ui, i, false, STRIP_W - 12.0, semibold(11.0)) {
        let name = ui
            .add(
                egui::Label::new(
                    RichText::new(elide(&track.name, 8))
                        .font(semibold(11.0))
                        .color(if audible { t.text } else { t.text_dim }),
                )
                .sense(Sense::click()),
            )
            .on_hover_text(if routed.is_empty() {
                format!("{} — click: select · double-click: rename", track.name)
            } else {
                format!(
                    "{} — channels: {} — click: select · double-click: rename",
                    track.name,
                    routed.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
                )
            });
        app.ui.mixer.name_rects.push((i, name.rect));
        if name.clicked() {
            select(app, i);
        }
        if name.double_clicked() {
            select(app, i);
            app.ui.mixer.start_rename(i, &track.name, false);
        }
    }
    // Insert slots (names only; edit them in the track panel on the right).
    let labels = crate::effects_ui::strip_labels(&track.effects);
    let (fx_rect, fx_resp) = ui.allocate_exact_size(vec2(STRIP_W - 12.0, 44.0), Sense::click());
    ui.painter()
        .rect_filled(fx_rect, CornerRadius::same(t.radius_sm), t.sunken);
    for (k, (name, on)) in labels.iter().take(3).enumerate() {
        let y = fx_rect.top() + 7.0 + k as f32 * 13.0;
        ui.painter().circle_filled(
            pos2(fx_rect.left() + 6.0, y),
            2.5,
            if *on { t.accent } else { t.text_faint },
        );
        ui.painter().text(
            pos2(fx_rect.left() + 12.0, y),
            egui::Align2::LEFT_CENTER,
            elide(name, 7),
            mono(9.5),
            if *on { t.text } else { t.text_dim },
        );
    }
    if labels.len() > 3 {
        ui.painter().text(
            fx_rect.right_bottom() - vec2(4.0, 2.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("+{}", labels.len() - 3),
            mono(9.0),
            t.text_dim,
        );
    }
    if labels.is_empty() {
        ui.painter().text(
            fx_rect.center(),
            egui::Align2::CENTER_CENTER,
            "FX",
            semibold(9.5),
            t.text_faint,
        );
    }
    if fx_resp
        .on_hover_text("Insert effects — click to edit them in the panel on the right")
        .clicked()
    {
        select(app, i);
    }
    let mut pan = track.pan;
    let r = knob(ui, &mut pan, -1.0, 1.0, 0.0, 30.0, true, |v| {
        if v.abs() < 0.005 {
            "Pan: centre".into()
        } else {
            format!("Pan {:+.0}%", v * 100.0)
        }
    });
    let cmd = r.changed().then_some(Command::SetMixerPan { track: i, pan });
    app.exec_gesture(&r, cmd);
    crate::automation_ui::menu(app, &r, AutomationTarget::MixerPan(i));
    let fader_h = (ui.available_height() - 86.0).max(80.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(4.0);
        let (l, r) = app.meter_hold.get(usize::from(i)).copied().unwrap_or_default();
        let clipped = i == 0 && app.clip_until.is_some_and(|u| u > std::time::Instant::now());
        meter(ui, vec2(12.0, fader_h), l, r, clipped);
        let mut v = track.volume;
        // Every track defaults to 0 dB, as in FL Studio's Mixer.
        let r = fader(ui, &mut v, MAX_GAIN, 1.0, vec2(24.0, fader_h), "Volume");
        let cmd = r.changed().then_some(Command::SetMixerVolume { track: i, volume: v });
        app.exec_gesture(&r, cmd);
        crate::automation_ui::menu(app, &r, AutomationTarget::MixerVolume(i));
    });
    let db = limeshot_dsp::gain_to_db(track.volume);
    ui.label(
        RichText::new(if db.is_finite() {
            format!("{db:+.1}")
        } else {
            "−∞".into()
        })
        .font(mono(10.0))
        .color(t.text_dim),
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.add_space(if i == 0 { 17.0 } else { 4.0 });
        let m = toggle_label(
            ui,
            track.muted,
            "M",
            t.danger,
            vec2(24.0, 18.0),
            "Mute (left-click) · Solo (right-click / Cmd+click)",
        );
        let solo_click = m.secondary_clicked() || (m.clicked() && ui.input(|x| x.modifiers.command));
        if solo_click && i != 0 {
            app.exec(Command::SetMixerSolo {
                track: i,
                solo: !track.solo,
            });
        } else if m.clicked() {
            app.exec(Command::SetMixerMute {
                track: i,
                muted: !track.muted,
            });
        }
        if i != 0 {
            let s = toggle_label(ui, track.solo, "S", t.warning, vec2(24.0, 18.0), "Solo");
            if s.clicked() {
                app.exec(Command::SetMixerSolo {
                    track: i,
                    solo: !track.solo,
                });
            }
        }
    });
    route_button(app, ui, i);
}

/// FL Studio's routing control at the bottom of each strip: with a track selected, every other
/// strip shows whether the selected track sends to it. Click to route/unroute; drag the knob for
/// the send level.
fn route_button(app: &mut LimeShotApp, ui: &mut Ui, i: u16) {
    let t = Theme::current();
    let sel = app.ui.mixer.selected;
    let (r, _) = ui.allocate_exact_size(vec2(STRIP_W - 12.0, 22.0), Sense::hover());
    if sel == i || sel == 0 {
        return;
    }
    let route = app
        .project()
        .mixer
        .tracks
        .get(usize::from(sel))
        .and_then(|s| s.routes.iter().find(|x| x.to == i).copied());
    let blocked = route.is_none() && app.project().mixer.would_cycle(sel, i);
    let btn = Rect::from_min_size(r.min, vec2(22.0, 22.0));
    app.ui.mixer.route_buttons.push((i, btn));
    let resp = ui.interact(
        btn,
        ui.id().with(("route", i)),
        if blocked { Sense::hover() } else { Sense::click() },
    );
    let fill = if route.is_some() {
        t.accent
    } else if resp.hovered() && !blocked {
        t.raised_hover
    } else {
        t.raised
    };
    ui.painter().rect_filled(btn, CornerRadius::same(t.radius_sm), fill);
    // A small "send" arrow pointing up into this strip.
    let c = btn.center();
    let col = if route.is_some() {
        t.text_on_accent
    } else if blocked {
        t.text_faint
    } else {
        t.text_dim
    };
    ui.painter().add(egui::Shape::convex_polygon(
        vec![c + vec2(-5.0, 2.0), c + vec2(5.0, 2.0), c + vec2(0.0, -5.0)],
        col,
        Stroke::NONE,
    ));
    ui.painter()
        .line_segment([c + vec2(0.0, 2.0), c + vec2(0.0, 6.0)], Stroke::new(2.0, col));
    let from_name = app
        .project()
        .mixer
        .tracks
        .get(usize::from(sel))
        .map(|x| x.name.clone())
        .unwrap_or_default();
    let tip = if blocked {
        format!("{from_name} cannot route here: it would create a feedback loop")
    } else if route.is_some() {
        format!("{from_name} sends here — click to remove the route")
    } else {
        format!("Route {from_name} to this track")
    };
    let resp = resp.on_hover_text(tip);
    if resp.clicked() {
        app.exec(Command::SetRoute {
            from: sel,
            to: i,
            level: if route.is_some() { None } else { Some(1.0) },
        });
    }
    // FL Studio's menu on this button: route or sidechain, optionally dropping the other routes.
    if !blocked {
        resp.context_menu(|ui| route_menu_items(app, ui, sel, i));
    }
    if let Some(route) = route {
        // Sends go up to +6 dB like the faders, with the same travel (0 dB at about 80 %).
        let mut pos = gain_pos(route.level);
        let mut kui = ui
            .new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(r.min + vec2(28.0, 0.0), vec2(26.0, 22.0))));
        let k = knob(&mut kui, &mut pos, 0.0, 1.0, gain_pos(1.0), 22.0, false, |v| {
            let db = limeshot_dsp::gain_to_db(pos_gain(v));
            if db.is_finite() {
                format!("Send level {db:+.1} dB")
            } else {
                "Send level −∞ dB (sidechain only)".into()
            }
        });
        app.ui.mixer.send_knobs.push((i, k.rect));
        let cmd = k.changed().then(|| Command::SetRoute {
            from: sel,
            to: i,
            level: Some(pos_gain(pos)),
        });
        app.exec_gesture(&k, cmd);
    }
}

/// FL Studio's four routing entries (the strip route button's menu, and the track panel's
/// Output menu per target).
fn route_menu_items(app: &mut LimeShotApp, ui: &mut Ui, from: u16, to: u16) {
    for (label, level, only) in [
        ("Route to this track", 1.0, false),
        ("Route to this track only", 1.0, true),
        ("Sidechain to this track", 0.0, false),
        ("Sidechain to this track only", 0.0, true),
    ] {
        if ui.button(label).clicked() {
            route_menu(app, from, to, level, only);
            ui.close();
        }
    }
}

/// Route `from` to `to` at `level` (0: sidechain only), removing `from`'s other routes when
/// `only` (one undo step).
pub fn route_menu(app: &mut LimeShotApp, from: u16, to: u16, level: f32, only: bool) {
    let mut cmds: Vec<Command> = Vec::new();
    if only {
        let others: Vec<u16> = app
            .project()
            .mixer
            .tracks
            .get(usize::from(from))
            .map(|t| t.routes.iter().map(|r| r.to).filter(|x| *x != to).collect())
            .unwrap_or_default();
        cmds.extend(others.into_iter().map(|x| Command::SetRoute {
            from,
            to: x,
            level: None,
        }));
    }
    cmds.push(Command::SetRoute {
        from,
        to,
        level: Some(level),
    });
    app.exec(Command::Batch(cmds));
}

/// The inline name field (the owner asked for typing in place; FL Studio opens a small
/// popup). Enter or clicking elsewhere keeps the name, Esc cancels. Returns whether it drew.
fn name_editor(app: &mut LimeShotApp, ui: &mut Ui, i: u16, in_panel: bool, width: f32, font: egui::FontId) -> bool {
    let Some(r) = app
        .ui
        .mixer
        .rename
        .as_mut()
        .filter(|r| r.track == i && r.in_panel == in_panel)
    else {
        return false;
    };
    let id = ui.id().with(("mixer-rename", i, in_panel));
    let resp = ui.add(
        egui::TextEdit::singleline(&mut r.text)
            .id(id)
            .font(font)
            .desired_width(width)
            .margin(vec2(2.0, 1.0)),
    );
    if std::mem::take(&mut r.focus) {
        // Start with the whole name selected, so typing replaces it.
        resp.request_focus();
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
        let all = egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(r.text.chars().count()),
        );
        state.cursor.set_char_range(Some(all));
        state.store(ui.ctx(), id);
    }
    if resp.lost_focus() {
        let cancel = ui.input(|x| x.key_pressed(egui::Key::Escape));
        let name = r.text.trim().to_string();
        app.ui.mixer.rename = None;
        let old = app.project().mixer.tracks.get(usize::from(i)).map(|t| t.name.clone());
        if !cancel && !name.is_empty() && old.as_deref() != Some(name.as_str()) {
            app.exec(Command::RenameMixerTrack { track: i, name });
        }
    }
    true
}

fn select(app: &mut LimeShotApp, i: u16) {
    app.ui.mixer.selected = i;
}

/// The selected track's panel on the right of the Mixer: level, routing and insert slots.
fn track_panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let i = app.ui.mixer.selected;
    let Some(track) = app.project().mixer.tracks.get(usize::from(i)).cloned() else {
        app.ui.mixer.selected = 0;
        return;
    };
    panel_header(ui, "Track", |ui| {
        ui.label(
            RichText::new(if i == 0 { "Master".into() } else { format!("Insert {i}") })
                .font(mono(10.5))
                .color(t.text_dim),
        );
    });
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            if !name_editor(app, ui, i, true, ui.available_width(), semibold(15.0)) {
                let name = ui
                    .add(egui::Label::new(RichText::new(&track.name).font(semibold(15.0))).sense(Sense::click()))
                    .on_hover_text("Click to rename");
                app.ui.mixer.panel_name = Some(name.rect);
                if name.clicked() {
                    app.ui.mixer.start_rename(i, &track.name, true);
                }
            }
            ui.add_space(8.0);
            section(ui, "Level");
            let mut db = limeshot_dsp::gain_to_db(track.volume).max(-60.0);
            let r = reset(
                ui.add(
                    egui::Slider::new(&mut db, -60.0..=6.0)
                        .suffix(" dB")
                        .fixed_decimals(1)
                        .text("Volume"),
                ),
                &mut db,
                0.0,
            );
            let cmd = crate::widgets::user_changed(&r).then(|| Command::SetMixerVolume {
                track: i,
                volume: if db <= -60.0 { 0.0 } else { limeshot_dsp::db_to_gain(db) },
            });
            app.exec_gesture(&r, cmd);
            crate::automation_ui::menu(app, &r, AutomationTarget::MixerVolume(i));
            let mut pan = track.pan;
            let r = reset(
                ui.add(egui::Slider::new(&mut pan, -1.0..=1.0).fixed_decimals(2).text("Pan")),
                &mut pan,
                0.0,
            );
            let cmd = crate::widgets::user_changed(&r).then_some(Command::SetMixerPan { track: i, pan });
            app.exec_gesture(&r, cmd);
            crate::automation_ui::menu(app, &r, AutomationTarget::MixerPan(i));
            ui.add_space(8.0);
            section(ui, "Input");
            let routed = routed(app, i);
            // Mixer tracks sending here: audio sends, or sidechain-only links (level 0).
            let incoming: Vec<(String, bool)> = app
                .project()
                .mixer
                .tracks
                .iter()
                .filter_map(|x| {
                    x.routes
                        .iter()
                        .find(|r| r.to == i)
                        .map(|r| (x.name.clone(), r.level == 0.0))
                })
                // Every insert feeds the master by default: list only its sidechains.
                .filter(|(_, sc)| i != 0 || *sc)
                .collect();
            if routed.is_empty() && incoming.is_empty() {
                ui.label(RichText::new("No channels route here.").color(t.text_dim));
            }
            for (n, sidechain) in incoming {
                let text = if sidechain {
                    format!("SC← {n}")
                } else {
                    format!("← {n}")
                };
                ui.label(RichText::new(text).color(t.text_dim))
                    .on_hover_text(if sidechain {
                        "Sidechain only: a compressor here can follow it"
                    } else {
                        "Mixer track sending here"
                    });
            }
            for (n, c) in routed {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect_filled(r, 2.0, c);
                    ui.label(n);
                });
            }
            ui.add_space(8.0);
            section(ui, "Output");
            if i == 0 {
                ui.label(RichText::new("Audio device").color(t.text_dim));
            } else if track.routes.is_empty() {
                ui.label(RichText::new("Not routed: this track is silent.").color(t.warning));
            }
            for route in &track.routes {
                let name = app
                    .project()
                    .mixer
                    .tracks
                    .get(usize::from(route.to))
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
                ui.horizontal(|ui| {
                    let (arrow, tip) = if route.level == 0.0 {
                        ("SC→", "Sidechain only: plugins here can use it, no audio is mixed in")
                    } else {
                        ("→", "Send")
                    };
                    ui.add_sized(
                        vec2(70.0, 20.0),
                        egui::Label::new(RichText::new(format!("{arrow} {name}")).color(t.text)),
                    )
                    .on_hover_text(tip);
                    let mut db = limeshot_dsp::gain_to_db(route.level).max(-60.0);
                    ui.spacing_mut().slider_width = (ui.available_width() - 110.0).max(50.0);
                    let r = reset(
                        ui.add(egui::Slider::new(&mut db, -60.0..=6.0).fixed_decimals(1).suffix(" dB")),
                        &mut db,
                        0.0,
                    );
                    let level = if db <= -60.0 { 0.0 } else { limeshot_dsp::db_to_gain(db) };
                    let cmd = crate::widgets::user_changed(&r).then_some(Command::SetRoute {
                        from: i,
                        to: route.to,
                        level: Some(level),
                    });
                    app.exec_gesture(&r, cmd);
                    if ui.small_button("×").on_hover_text("Remove route").clicked() {
                        app.exec(Command::SetRoute {
                            from: i,
                            to: route.to,
                            level: None,
                        });
                    }
                });
            }
            if i != 0 {
                // Each target has the route button's four entries: route or sidechain, and
                // the "only" variants that drop this track's other routes.
                ui.menu_button("+ Route to…", |ui| {
                    let targets: Vec<(u16, String, bool)> = app
                        .project()
                        .mixer
                        .tracks
                        .iter()
                        .enumerate()
                        .filter_map(|(j, x)| u16::try_from(j).ok().map(|j| (j, x.name.clone())))
                        .filter(|(j, _)| *j != i)
                        .map(|(j, n)| {
                            let routed = track.routes.iter().any(|r| r.to == j);
                            (j, n, routed || !app.project().mixer.would_cycle(i, j))
                        })
                        .collect();
                    for (j, n, ok) in targets {
                        let label = if j == 0 { n } else { format!("{j}  {n}") };
                        ui.add_enabled_ui(ok, |ui| {
                            ui.menu_button(label, |ui| route_menu_items(app, ui, i, j))
                                .response
                                .on_disabled_hover_text("Would create a feedback loop");
                        });
                    }
                });
            }
            crate::effects_ui::chain(app, ui, i);
            ui.add_space(10.0);
            eq_section(app, ui, i, &track.eq);
        });
    });
}

const EQ_NAMES: [&str; 3] = ["Low shelf", "Peak", "High shelf"];

/// Frequency ↔ position 0–1 on the EQ graph's logarithmic axis.
fn eq_freq_pos(f: f32) -> f32 {
    let (lo, hi) = EqBand::FREQ;
    ((f / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0)
}

fn eq_pos_freq(x: f32) -> f32 {
    let (lo, hi) = EqBand::FREQ;
    lo * (hi / lo).powf(x.clamp(0.0, 1.0))
}

/// Where a band's handle sits in the graph.
pub fn eq_handle(graph: Rect, b: EqBand) -> egui::Pos2 {
    pos2(
        graph.left() + eq_freq_pos(b.freq) * graph.width(),
        graph.center().y - b.gain / EqBand::GAIN * graph.height() * 0.5,
    )
}

/// The track's built-in EQ (FL Studio's Equalizer in the Mixer): a response graph with a handle
/// per band, and gain, frequency and width knobs.
fn eq_section(app: &mut LimeShotApp, ui: &mut Ui, i: u16, eq: &[EqBand; 3]) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        section(ui, "Equalizer");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let flat = *eq == EqBand::DEFAULTS;
            if ui
                .add_enabled(!flat, egui::Button::new("Reset").small())
                .on_hover_text("All bands back to 0 dB and their default frequency and width")
                .clicked()
            {
                let cmds = (0..3u8)
                    .map(|band| Command::SetMixerEq {
                        track: i,
                        band,
                        value: EqBand::DEFAULTS[usize::from(band)],
                    })
                    .collect();
                app.exec(Command::Batch(cmds));
            }
        });
    });
    let colors = [
        t.accent,
        rgb(limeshot_core::Rgb(0x4f, 0xb4, 0xe8)),
        rgb(limeshot_core::Rgb(0xf0, 0x9a, 0x3e)),
    ];
    let (graph, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::click_and_drag());
    app.ui.mixer.eq_graph = Some(graph);
    let p = ui.painter_at(graph);
    p.rect_filled(graph, CornerRadius::same(t.radius_sm), t.sunken);
    for f in [100.0f32, 1000.0, 10_000.0] {
        let x = graph.left() + eq_freq_pos(f) * graph.width();
        p.line_segment(
            [pos2(x, graph.top()), pos2(x, graph.bottom())],
            Stroke::new(1.0, t.grid_step),
        );
    }
    p.line_segment(
        [
            pos2(graph.left(), graph.center().y),
            pos2(graph.right(), graph.center().y),
        ],
        Stroke::new(1.0, t.grid_beat),
    );
    let mut dsp = limeshot_dsp::effects::Eq3::new(48_000.0);
    dsp.set_bands(&eq.map(|b| (b.freq, b.gain, b.width)));
    let pts: Vec<egui::Pos2> = (0..=graph.width() as usize)
        .map(|k| {
            let x = graph.left() + k as f32;
            let db = dsp.response_db(eq_pos_freq(k as f32 / graph.width()));
            pos2(x, graph.center().y - db / EqBand::GAIN * graph.height() * 0.5)
        })
        .collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.5, t.text)));
    for (b, c) in eq.iter().zip(colors) {
        p.circle(eq_handle(graph, *b), 4.5, c, Stroke::new(1.0, t.sunken));
    }
    // Graph gestures: press near a band's handle (or anywhere: the nearest band) and drag.
    let pointer = resp.interact_pointer_pos();
    if resp.drag_started()
        && let Some(pp) = pointer
    {
        let nearest = (0..3).min_by(|&a, &b| {
            let da = eq_handle(graph, eq[a]).distance(pp);
            let db = eq_handle(graph, eq[b]).distance(pp);
            da.total_cmp(&db)
        });
        app.ui.mixer.eq_drag = nearest;
    }
    if let (Some(band), Some(pp)) = (app.ui.mixer.eq_drag, pointer)
        && resp.dragged()
    {
        let mut v = eq[band];
        let gain =
            ((graph.center().y - pp.y) / (graph.height() * 0.5) * EqBand::GAIN).clamp(-EqBand::GAIN, EqBand::GAIN);
        v.gain = (gain * 10.0).round() / 10.0;
        if resp.dragged_by(egui::PointerButton::Secondary) {
            v.width = (v.width + resp.drag_delta().x / graph.width()).clamp(0.0, 1.0);
        } else {
            v.freq = eq_pos_freq((pp.x - graph.left()) / graph.width()).round();
        }
        let cmd = (v != eq[band]).then_some(Command::SetMixerEq {
            track: i,
            band: band as u8,
            value: v,
        });
        app.exec_gesture(&resp, cmd);
        app.hint(format!(
            "EQ band {} ({}): {:.0} Hz, {:+.1} dB, width {:.2}",
            band + 1,
            EQ_NAMES[band],
            v.freq,
            v.gain,
            v.width
        ));
    } else {
        app.exec_gesture(&resp, None);
    }
    if resp.drag_stopped() {
        app.ui.mixer.eq_drag = None;
    }
    if resp.hovered()
        && let Some(pp) = resp.hover_pos()
    {
        let f = eq_pos_freq((pp.x - graph.left()) / graph.width());
        app.hint(format!(
            "{f:.0} Hz — drag a band: left/right frequency, up/down gain · right-drag: width"
        ));
    }

    // Knobs: one column per band (gain, frequency, width), as in FL Studio's track EQ.
    ui.add_space(4.0);
    ui.columns(3, |cols| {
        for (band, col) in cols.iter_mut().enumerate() {
            let b = eq[band];
            col.vertical_centered(|ui| {
                ui.label(RichText::new(EQ_NAMES[band]).font(mono(9.5)).color(colors[band]));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let set = |v: EqBand| Command::SetMixerEq {
                        track: i,
                        band: band as u8,
                        value: v,
                    };
                    let mut g = b.gain;
                    let r = knob(ui, &mut g, -EqBand::GAIN, EqBand::GAIN, 0.0, 26.0, true, |v| {
                        format!("Band {} gain {v:+.1} dB", band + 1)
                    });
                    let cmd = r.changed().then(|| set(EqBand { gain: g, ..b }));
                    app.exec_gesture(&r, cmd);
                    let d = EqBand::DEFAULTS[band];
                    let mut f = eq_freq_pos(b.freq);
                    let r = knob(ui, &mut f, 0.0, 1.0, eq_freq_pos(d.freq), 26.0, false, |v| {
                        format!("Band {} frequency {:.0} Hz", band + 1, eq_pos_freq(v))
                    });
                    let cmd = r.changed().then(|| {
                        set(EqBand {
                            freq: eq_pos_freq(f).round(),
                            ..b
                        })
                    });
                    app.exec_gesture(&r, cmd);
                    let mut w = b.width;
                    let r = knob(ui, &mut w, 0.0, 1.0, d.width, 26.0, false, |v| {
                        format!("Band {} width {v:.2}", band + 1)
                    });
                    let cmd = r.changed().then(|| set(EqBand { width: w, ..b }));
                    app.exec_gesture(&r, cmd);
                });
            });
        }
    });
}

fn elide(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    }
}
