//! Header (logo, menus, project name), the global toolbar (transport, tempo, time, pattern, snap,
//! panel toggles) and the status bar.

use crate::app::LimeShotApp;
use crate::dialogs::Dialog;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, mono, prop, semibold};
use crate::widgets::Hint as _;
use crate::widgets::{icon_button, meter, readout, segmented, transport_button, vrule};
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

/// Draw the main toolbar and handle transport, pattern, snap, window, and keyboard-note controls.
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
        let typing = icon_button(
            ui,
            Icon::PianoRoll,
            app.keyboard_notes,
            B,
            "Typing keyboard to piano (Cmd+T) — Z–M and Q–U play the selected channel",
        );
        if typing.clicked() {
            app.keyboard_notes = !app.keyboard_notes;
        }
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
    let start = ui.cursor().min;
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
    // FL Studio: a right-click shows the Channel Rack (pattern mode) or the Playlist (song).
    let area = egui::Rect::from_min_max(start, egui::pos2(ui.cursor().min.x, start.y + B));
    if right_clicked(ui, area) {
        app.show_window(if song { Win::Playlist } else { Win::ChannelRack });
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
    let armed = app.ui.record.armed;
    let r = transport_button(
        ui,
        Icon::Record,
        armed,
        B,
        "Record (R): notes from the MIDI input or the typing keyboard go into the current pattern while playing. Right-click: options",
    );
    if r.clicked() {
        crate::recording::toggle(app);
    }
    // FL Studio's record menu (only notes are recorded; the other filters are not in yet).
    r.context_menu(|ui| {
        let t = Theme::current();
        ui.label(RichText::new("Recording filter").small().color(t.text_faint));
        ui.add_enabled(false, egui::Button::selectable(false, "Automation"));
        let _ = ui.add(egui::Button::selectable(true, "Notes"));
        ui.add_enabled(false, egui::Button::selectable(false, "Audio"));
        ui.add_enabled(false, egui::Button::selectable(false, "Clips"));
        ui.separator();
        ui.label(RichText::new("Options").small().color(t.text_faint));
        let st = &mut app.ui.record;
        for (on, label) in [
            (&mut st.starts_playback, "Recording starts playback"),
            (&mut st.disarm_on_stop, "Disarm on stop"),
        ] {
            if ui.add(egui::Button::selectable(*on, label)).clicked() {
                *on = !*on;
            }
        }
        ui.separator();
        ui.label(RichText::new("Quantize").small().color(t.text_faint));
        for (on, label) in [
            (&mut st.quantize_start, "Note start time"),
            (&mut st.quantize_end, "Note end time"),
        ] {
            if ui.add(egui::Button::selectable(*on, label)).clicked() {
                *on = !*on;
            }
        }
    });
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
    // FL Studio's tempo menu: the tempo tapper and presets from 80 BPM in steps of 10.
    crate::widgets::clickable(&r).context_menu(|ui| {
        if ui.button("Tempo tapper").clicked() {
            app.ui.toolbar.tapper = Some(Tapper::default());
            ui.close();
        }
        ui.separator();
        for preset in (80..=180).step_by(10) {
            let on = (app.project().tempo.bpm() - f64::from(preset)).abs() < 1e-6;
            if ui.selectable_label(on, format!("{preset} BPM")).clicked() {
                app.exec(Command::SetTempo(Tempo::from_bpm(f64::from(preset))));
                ui.close();
            }
        }
    });
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
    let mss = format!("{}:{:05.2}", (secs / 60.0) as u32, secs % 60.0);
    let (main, caption) = if app.ui.toolbar.time_in_seconds {
        (mss, format!("M:S:CS   {bbt}"))
    } else {
        (bbt.to_string(), format!("B:B:T   {mss}"))
    };
    let r = readout(
        ui,
        vec2(118.0, B + 6.0),
        &main,
        &caption,
        "Song position (bar:beat:tick, minutes:seconds). Click to return to the start; right-click for the format.",
    );
    if r.clicked() {
        app.locate(0);
    }
    // FL Studio's Mode menu (LimeShot has two of its formats).
    r.context_menu(|ui| {
        ui.label(RichText::new("Mode").small().color(t.text_faint));
        let s = &mut app.ui.toolbar.time_in_seconds;
        if ui.radio(!*s, "Bar : Beat : Tick").clicked() {
            *s = false;
            ui.close();
        }
        if ui.radio(*s, "Minute : Second : Centisecond").clicked() {
            *s = true;
            ui.close();
        }
    });
}

/// Whether the secondary button was clicked inside `area` this frame.
fn right_clicked(ui: &Ui, area: egui::Rect) -> bool {
    ui.input(|i| i.pointer.secondary_clicked() && i.pointer.interact_pos().is_some_and(|p| area.contains(p)))
}

/// The tempo tapper window: tap and the project's tempo follows (one undo step while it is
/// open).
pub fn tapper(app: &mut LimeShotApp, ctx: &egui::Context) {
    if app.ui.toolbar.tapper.is_none() {
        return;
    }
    let mut open = true;
    let now = ctx.input(|i| i.time);
    egui::Window::new("Tempo tapper")
        .id(egui::Id::new("limeshot-tempo-tapper"))
        .default_width(190.0)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_width(190.0);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(format!("{:.3} BPM", app.project().tempo.bpm()));
            let tap = ui.add_sized(vec2(160.0, 60.0), egui::Button::new(RichText::new("Tap").size(20.0)));
            ui.label(RichText::new("Click in time; a two-second pause starts over.").small());
            if tap.clicked()
                && let Some(bpm) = app.ui.toolbar.tapper.as_mut().and_then(|t| t.tap(now))
            {
                app.doc.begin_gesture();
                app.exec(Command::SetTempo(Tempo::from_bpm(bpm)));
            }
        });
    if !open {
        app.ui.toolbar.tapper = None;
        app.doc.end_gesture();
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
        let r = icon_button(ui, icon, app.is_window_open(w), B, &tip);
        if r.clicked() {
            app.toggle_window(w);
        }
        // FL Studio: a right-click on the Playlist and Piano Roll icons maximises the window.
        if r.secondary_clicked() && matches!(w, Win::Playlist | Win::PianoRoll) {
            app.show_window(w);
            app.ui.workspace.state_mut(w).maximized = true;
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

/// Toolbar choices made from its right-click menus (FL Studio's, observed in session 15).
#[derive(Debug, Default)]
pub struct State {
    /// The song position shows minutes:seconds:centiseconds first (FL's Mode menu).
    pub time_in_seconds: bool,
    /// The CPU panel shows the voice count instead of "CPU" (FL's View voice count).
    pub show_voices: bool,
    /// The tempo tapper window, while open.
    pub tapper: Option<Tapper>,
}

/// FL Studio's tempo tapper (View › Tempo tapper, Cmd+F5): tap along and the tempo follows.
#[derive(Debug, Default)]
pub struct Tapper {
    /// Times of the recent taps (seconds).
    taps: Vec<f64>,
}

impl Tapper {
    /// A tap at `now`; returns the tempo the taps so far give (two taps or more). A pause of
    /// two seconds starts over.
    pub fn tap(&mut self, now: f64) -> Option<f64> {
        if self.taps.last().is_some_and(|&t| now - t > 2.0) {
            self.taps.clear();
        }
        self.taps.push(now);
        if self.taps.len() > 9 {
            self.taps.remove(0);
        }
        let (first, last) = (*self.taps.first()?, *self.taps.last()?);
        let n = self.taps.len() - 1;
        (n > 0 && last > first).then(|| (60.0 * n as f64 / (last - first)).clamp(10.0, 522.0))
    }
}

/// The CPU readout changes once per second (owner's hands-on test: a value redrawn every frame
/// was visual clutter). It shows the highest load of the last second, so short spikes still show.
#[derive(Debug, Default, Clone, Copy)]
pub struct CpuReadout {
    /// Seconds between updates (FL's Refresh: Slow 1 s, Medium 0.5 s, Fast 0.2 s); 0 means 1 s.
    pub interval: f64,
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
            Some(t) if now - t >= if self.interval > 0.0 { self.interval } else { 1.0 } => {
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
    let voices = app.player.status().voices();
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
    let caption = if app.ui.toolbar.show_voices {
        format!("{voices} V")
    } else {
        "CPU".to_string()
    };
    ui.painter().text(
        rect.center() + vec2(0.0, 7.0),
        Align2::CENTER_CENTER,
        caption,
        semibold(8.0),
        t.text_dim,
    );
    let resp = resp
        .interact(Sense::click())
        .hint("Audio load: time spent rendering per buffer; right-click for options");
    // FL Studio's CPU panel menu.
    resp.context_menu(|ui| {
        ui.label(RichText::new("Refresh").small().color(t.text_faint));
        for (label, secs) in [("Slow", 1.0), ("Medium", 0.5), ("Fast", 0.2)] {
            let cur = if app.cpu.interval > 0.0 { app.cpu.interval } else { 1.0 };
            if ui.radio((cur - secs).abs() < 1e-9, label).clicked() {
                app.cpu.interval = secs;
                ui.close();
            }
        }
        ui.label(RichText::new("Misc").small().color(t.text_faint));
        if ui
            .checkbox(&mut app.ui.toolbar.show_voices, "View voice count")
            .clicked()
        {
            ui.close();
        }
    });
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
                app.open_settings(crate::settings_window::Page::Audio);
            }
            ui.label(
                RichText::new(format!("{} voices  ·", app.player.status().voices()))
                    .font(prop(11.0))
                    .color(t.text_dim),
            );
        });
    });
}
