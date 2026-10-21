//! Header (logo, menus, project name), the global toolbar (transport, tempo, time, pattern, snap,
//! panel toggles) and the status bar.

use crate::app::LimeShotApp;
use crate::dialogs::Dialog;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, mono, prop, semibold};
use crate::widgets::Hint as _;
use crate::widgets::{icon_button, meter, readout, segmented, transport_button, unavailable_button, vrule};
use crate::workspace::Win;
use egui::{Align2, ComboBox, CornerRadius, DragValue, Rect, RichText, Sense, Stroke, Ui, vec2};
use limeshot_core::{BarBeatTick, Command, Snap, Tempo, TimeSignature, Timeline};
use limeshot_engine::PlayMode;

pub const HEIGHT: f32 = 50.0;
const HINT_W: f32 = 380.0;
const B: f32 = 28.0;

/// Top header: logo, menus, the hint panel and the project name.
pub fn header(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let full = ui.max_rect();
    ui.painter()
        .line_segment([full.left_bottom(), full.right_bottom()], Stroke::new(1.0, t.border));
    let menus_end = ui
        .horizontal_centered(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
            icons::logo(ui.painter(), r.center(), 8.0);
            ui.label(RichText::new("LimeShot").font(semibold(12.5)));
            ui.add_space(8.0);
            crate::menus::menu_bar(app, ui)
        })
        .inner;
    // FL Studio's hint panel (its top left), after the menus; filled by `hint_panel` once
    // everything else has been drawn. The project name keeps 200 px on the right.
    let left = menus_end + 16.0;
    let w = HINT_W.min(full.right() - 200.0 - left);
    app.ui.hint_rect =
        (w > 80.0).then(|| Rect::from_min_size(egui::pos2(left, full.top() + 2.0), vec2(w, full.height() - 4.0)));
    let name = app
        .path
        .as_ref()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| app.project().title.clone());
    let modified = if app.doc.is_modified() { "  •  edited" } else { "" };
    ui.painter().text(
        full.right_center() - vec2(4.0, 0.0),
        Align2::RIGHT_CENTER,
        format!("{name}{modified}"),
        prop(12.0),
        t.text_dim,
    );
}

pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let full = ui.max_rect();
    ui.painter()
        .line_segment([full.left_bottom(), full.right_bottom()], Stroke::new(1.0, t.border));
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        transport(app, ui);
        vrule(ui, B);
        tempo_and_time(app, ui);
        vrule(ui, B);
        pattern_selector(app, ui);
        vrule(ui, B);
        snap_and_options(app, ui);
        vrule(ui, B);
        panels(app, ui);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            output(app, ui);
            vrule(ui, B);
            edit_buttons(app, ui);
        });
    });
}

fn transport(app: &mut LimeShotApp, ui: &mut Ui) {
    let song = app.ui.mode == PlayMode::Song;
    if segmented(
        ui,
        &["PAT", "SONG"],
        usize::from(song),
        B,
        "Pattern / Song playback (L)",
    )
    .is_some()
    {
        app.toggle_mode();
    }
    ui.add_space(4.0);
    let playing = app.is_playing();
    if transport_button(
        ui,
        if playing { Icon::Pause } else { Icon::Play },
        playing,
        B,
        "Play / pause (Space)",
    )
    .clicked()
    {
        app.toggle_play();
    }
    if transport_button(ui, Icon::Stop, false, B, "Stop (Space)").clicked() {
        app.stop();
    }
    unavailable_button(ui, Icon::Record, B, "Record");
}

fn tempo_and_time(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let mut bpm = app.project().tempo.bpm();
    let r = ui
        .add_sized(
            vec2(78.0, B),
            DragValue::new(&mut bpm)
                .range(10.0..=522.0)
                .speed(0.2)
                .fixed_decimals(3),
        )
        .hint("Tempo in BPM: drag or type (10–522)");
    let cmd = crate::widgets::user_changed(&r).then(|| Command::SetTempo(Tempo::from_bpm(bpm)));
    app.exec_gesture(&r, cmd);
    ui.label(RichText::new("BPM").font(semibold(9.5)).color(t.text_dim));
    ui.add_space(4.0);
    let mut sig = app.project().time_signature;
    let (mut n, mut d) = (sig.numerator, sig.denominator);
    let a = ui
        .add_sized(vec2(26.0, B), DragValue::new(&mut n).range(1..=32))
        .hint("Beats per bar");
    ui.label(RichText::new("/").color(t.text_dim));
    let b = ui
        .add_sized(vec2(26.0, B), DragValue::new(&mut d).range(1..=32))
        .hint("Beat unit (power of two)");
    if crate::widgets::user_changed(&a) || crate::widgets::user_changed(&b) {
        sig = TimeSignature::new(n, d);
        app.exec(Command::SetTimeSignature(sig));
    }
    ui.add_space(4.0);
    let pos = app.player.status().position();
    let sig = app.project().time_signature;
    let bbt = BarBeatTick::from_ticks(pos, sig);
    let secs = Timeline::new(app.project().tempo, 48_000).seconds(pos);
    let caption = format!("B:B:T   {}:{:05.2}", (secs / 60.0) as u32, secs % 60.0);
    if readout(
        ui,
        vec2(118.0, B + 6.0),
        &bbt.to_string(),
        &caption,
        "Song position (bar:beat:tick, minutes:seconds). Click to return to the start.",
    )
    .clicked()
    {
        app.locate(0);
    }
}

fn pattern_selector(app: &mut LimeShotApp, ui: &mut Ui) {
    let cur = app.current_pattern();
    let pats: Vec<_> = app
        .project()
        .patterns
        .iter()
        .map(|p| (p.id, p.name.clone(), p.color))
        .collect();
    let (name, color) = pats
        .iter()
        .find(|p| p.0 == cur)
        .map(|p| (p.1.clone(), p.2))
        .unwrap_or((String::new(), limeshot_core::PALETTE[0]));
    let (r, _) = ui.allocate_exact_size(vec2(4.0, 18.0), Sense::hover());
    ui.painter()
        .rect_filled(r, CornerRadius::same(2), crate::theme::rgb(color));
    // The patterns, then "Create new" (FL Studio's selector menu: its Find first empty… makes a
    // new pattern). The − and + buttons are gone; −/+ on the keyboard still step through them.
    ComboBox::from_id_salt("pattern_selector")
        .width(140.0)
        .selected_text(name)
        .show_ui(ui, |ui| {
            for (id, n, c) in &pats {
                let text = RichText::new(format!("■  {n}")).color(crate::theme::rgb(*c));
                if ui.selectable_label(*id == cur, text).clicked() {
                    app.set_current_pattern(*id);
                }
            }
            ui.separator();
            if ui.button("Create new").hint("A new, empty pattern (F4)").clicked() {
                crate::menus::add_pattern(app);
                ui.close();
            }
        });
}

fn snap_and_options(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    ui.label(RichText::new("SNAP").font(semibold(9.5)).color(t.text_dim));
    ComboBox::from_id_salt("snap")
        .width(74.0)
        .selected_text(app.ui.snap.label())
        .show_ui(ui, |ui| {
            for s in Snap::ALL {
                ui.selectable_value(&mut app.ui.snap, s, s.label());
            }
        })
        .response
        .hint("Snap: grid for the Piano Roll and Playlist (hold Alt to bypass)");
    if icon_button(ui, Icon::Metronome, app.ui.metronome, B, "Metronome (Cmd+M)").clicked() {
        let m = !app.ui.metronome;
        app.set_metronome(m);
    }
    if icon_button(ui, Icon::Loop, app.ui.song_loop, B, "Loop the song in Song mode").clicked() {
        let l = !app.ui.song_loop;
        app.set_song_loop(l);
    }
}

fn panels(app: &mut LimeShotApp, ui: &mut Ui) {
    if icon_button(ui, Icon::Browser, app.ui.show_browser, B, "Browser (Alt+F8)").clicked() {
        app.ui.show_browser ^= true;
    }
    // FL Studio's window shortcut buttons.
    for (w, icon) in [
        (Win::Playlist, Icon::Playlist),
        (Win::ChannelRack, Icon::ChannelRack),
        (Win::PianoRoll, Icon::PianoRoll),
        (Win::Mixer, Icon::Mixer),
    ] {
        let tip = format!("{} ({})", w.label(), w.shortcut());
        if icon_button(ui, icon, app.is_window_open(w), B, &tip).clicked() {
            app.toggle_window(w);
        }
    }
}

fn edit_buttons(app: &mut LimeShotApp, ui: &mut Ui) {
    // Right-to-left layout: listed in reverse visual order.
    if icon_button(ui, Icon::Export, false, B, "Export WAV (Cmd+R)").clicked() {
        app.dialog = Some(Dialog::export(app));
    }
    if icon_button(ui, Icon::Save, app.doc.is_modified(), B, "Save (Cmd+S)").clicked() {
        app.save();
    }
    let redo_tip = app
        .doc
        .redo_label()
        .map(|l| format!("Redo {l}"))
        .unwrap_or_else(|| "Nothing to redo".into());
    if icon_button(ui, Icon::Redo, false, B, &redo_tip).clicked() {
        app.doc.redo();
    }
    let undo_tip = app
        .doc
        .undo_label()
        .map(|l| format!("Undo {l}"))
        .unwrap_or_else(|| "Nothing to undo".into());
    if icon_button(ui, Icon::Undo, false, B, &undo_tip).clicked() {
        app.doc.undo();
    }
}

/// The CPU readout changes once per second (owner's hands-on test: a value redrawn every frame
/// was visual clutter). It shows the highest load of the last second, so short spikes still show.
#[derive(Debug, Default, Clone, Copy)]
pub struct CpuReadout {
    shown: f32,
    peak: f32,
    /// When `shown` was last updated (egui time in seconds); `None` before the first frame.
    since: Option<f64>,
}

impl CpuReadout {
    /// Feed the current load at time `now`; returns the value to display.
    pub fn update(&mut self, now: f64, load: f32) -> f32 {
        self.peak = self.peak.max(load);
        match self.since {
            None => {
                self.shown = load;
                self.peak = 0.0;
                self.since = Some(now);
            }
            Some(t) if now - t >= 1.0 => {
                self.shown = self.peak;
                self.peak = 0.0;
                self.since = Some(now);
            }
            Some(_) => {}
        }
        self.shown
    }
}

fn output(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let now = ui.input(|i| i.time);
    let load = app.cpu.update(now, app.player.status().load());
    // Keep the readout current when nothing else repaints (it drops back to idle after a stop).
    ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
    let (rect, resp) = ui.allocate_exact_size(vec2(46.0, B), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(t.radius), t.sunken);
    let f = load.clamp(0.0, 1.0);
    let fill = egui::Rect::from_min_size(rect.min, vec2(rect.width() * f, rect.height()));
    ui.painter().rect_filled(
        fill,
        CornerRadius::same(t.radius),
        if load > 0.8 {
            t.danger
        } else {
            mix(t.sunken, t.accent, 0.25)
        },
    );
    ui.painter().text(
        rect.center() - vec2(0.0, 5.0),
        Align2::CENTER_CENTER,
        format!("{:.0}%", load * 100.0),
        mono(10.5),
        t.text,
    );
    ui.painter().text(
        rect.center() + vec2(0.0, 7.0),
        Align2::CENTER_CENTER,
        "CPU",
        semibold(8.0),
        t.text_dim,
    );
    resp.hint("Audio load: time spent rendering per buffer");
    let (l, r) = app.player.status().peak(0);
    if app.player.status().take_clipped() {
        app.clip_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
    }
    let clipped = app.clip_until.is_some_and(|u| u > std::time::Instant::now());
    meter(ui, vec2(12.0, B + 6.0), l, r, clipped).hint("Master output");
}

/// What LimeShot is doing, for the hint panel's first line.
pub fn activity(app: &LimeShotApp) -> String {
    if let Some(job) = &app.export_job {
        return format!(
            "Exporting {}…",
            job.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()
        );
    }
    if app.plugins.is_scanning() {
        return "Scanning plugins…".into();
    }
    let loading = app.samples.loading();
    if loading > 0 {
        return format!("Loading {loading} sample{}…", if loading == 1 { "" } else { "s" });
    }
    let mode = if app.ui.mode == PlayMode::Song {
        "Song"
    } else {
        "Pattern"
    };
    if app.is_playing() {
        return format!("Playing · {mode}");
    }
    match app.doc.undo_label() {
        Some(label) => format!("Stopped · {mode} · last edit: {label}"),
        None => format!("Stopped · {mode}"),
    }
}

/// FL Studio's hint panel (top left): what LimeShot is doing, and below it what the control
/// under the pointer does, with its shortcut on the right when the description ends with one.
pub fn hint_panel(app: &mut LimeShotApp, ui: &mut Ui) {
    let Some(r) = app.ui.hint_rect else { return };
    let t = Theme::current();
    let hover = crate::widgets::take_hover_hint(ui.ctx());
    let text = if app.ui.hint.is_empty() {
        hover.unwrap_or_default()
    } else {
        app.ui.hint.clone()
    };
    let (text, key) = split_shortcut(&text);
    let painter = ui.painter_at(r);
    painter.rect(
        r,
        CornerRadius::same(t.radius_sm),
        t.sunken,
        Stroke::new(1.0, t.border),
        egui::StrokeKind::Inside,
    );
    let inner = r.shrink2(vec2(8.0, 1.0));
    let top = Rect::from_min_max(inner.min, egui::pos2(inner.right(), inner.center().y));
    painter.with_clip_rect(top).text(
        top.left_center(),
        Align2::LEFT_CENTER,
        activity(app),
        prop(9.5),
        t.text_faint,
    );
    let key_w = key.as_ref().map_or(0.0, |k| {
        painter
            .text(
                egui::pos2(inner.right(), inner.bottom() - 1.0),
                Align2::RIGHT_BOTTOM,
                k,
                mono(9.5),
                t.text_dim,
            )
            .width()
            + 8.0
    });
    let line = Rect::from_min_max(
        egui::pos2(inner.left(), inner.center().y),
        egui::pos2(inner.right() - key_w, inner.bottom()),
    );
    app.ui.hint_shown = text.clone();
    painter
        .with_clip_rect(line)
        .text(line.left_center(), Align2::LEFT_CENTER, text, prop(11.0), t.text);
}

/// Split "Zoom in (Cmd+wheel)" into the text and its shortcut.
fn split_shortcut(text: &str) -> (String, Option<String>) {
    if let Some(open) = text.rfind(" (")
        && text.ends_with(')')
    {
        let key = &text[open + 2..text.len() - 1];
        if key.len() <= 16 && !key.contains(' ') {
            return (text[..open].to_string(), Some(key.to_string()));
        }
    }
    (text.to_string(), None)
}

pub fn status_bar(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    let full = ui.max_rect();
    ui.painter()
        .line_segment([full.left_top(), full.right_top()], Stroke::new(1.0, t.border));
    ui.horizontal_centered(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let out = match app.player.output_description() {
                Ok(name) => {
                    let buf = app
                        .player
                        .buffer_frames()
                        .map(|b| format!(" · {b} frames"))
                        .unwrap_or_default();
                    RichText::new(format!("{name} · {} Hz{buf}", app.player.sample_rate())).color(t.text_dim)
                }
                Err(reason) => RichText::new(format!("No audio output: {reason}")).color(t.warning),
            };
            if ui
                .add(egui::Label::new(out.font(prop(11.0))).sense(Sense::click()))
                .hint("Audio settings")
                .clicked()
            {
                app.dialog = Some(Dialog::audio(app));
            }
            ui.label(
                RichText::new(format!("{} voices  ·", app.player.status().voices()))
                    .font(prop(11.0))
                    .color(t.text_dim),
            );
        });
    });
}
