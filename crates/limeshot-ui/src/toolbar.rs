//! The global toolbar (transport, tempo, time, pattern, snap, window buttons) and status bar.

use crate::app::LimeShotApp;
use crate::dialogs::Dialog;
use crate::icons::{self, Icon};
use crate::theme::{Theme, mono, prop};
use crate::widgets::{icon_button, meter, unavailable_button};
use egui::{Align2, ComboBox, CornerRadius, DragValue, RichText, Sense, Stroke, StrokeKind, Ui, vec2};
use limeshot_core::{BarBeatTick, Command, Snap, Tempo, TimeSignature, Timeline};
use limeshot_engine::PlayMode;

pub const HEIGHT: f32 = 44.0;
const B: f32 = 26.0;

fn group(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let t = Theme::current();
    egui::Frame::new()
        .fill(t.sunken)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(5, 3))
        .show(ui, |ui| {
            ui.horizontal_centered(add);
        });
}

pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    ui.painter().rect_filled(ui.max_rect(), 0.0, t.surface);
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(4.0);
        let (r, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
        icons::logo(ui.painter(), r.center(), 13.0);
        ui.add_space(2.0);
        transport(app, ui);
        tempo_and_time(app, ui);
        pattern_selector(app, ui);
        snap_and_options(app, ui);
        windows(app, ui);
        edit_buttons(app, ui);
        output(app, ui);
    });
}

fn transport(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    group(ui, |ui| {
        // PAT / SONG switch.
        let song = app.ui.mode == PlayMode::Song;
        let (r, resp) = ui.allocate_exact_size(vec2(52.0, B), Sense::click());
        let half = r.width() / 2.0;
        for (i, label) in ["PAT", "SONG"].iter().enumerate() {
            let rr = egui::Rect::from_min_size(r.min + vec2(i as f32 * half, 0.0), vec2(half, r.height()));
            let on = (i == 1) == song;
            ui.painter()
                .rect_filled(rr, CornerRadius::same(3), if on { t.accent } else { t.raised });
            ui.painter().text(
                rr.center(),
                Align2::CENTER_CENTER,
                *label,
                mono(9.5),
                if on { t.text_on_accent } else { t.text_dim },
            );
        }
        if resp.clicked() {
            app.toggle_mode();
        }
        resp.on_hover_text("Pattern / Song playback (L)");
        let playing = app.is_playing();
        if icon_button(
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
        if icon_button(ui, Icon::Stop, false, B, "Stop (Space)").clicked() {
            app.stop();
        }
        unavailable_button(ui, Icon::Record, B, "Record");
    });
}

fn tempo_and_time(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    group(ui, |ui| {
        let mut bpm = app.project().tempo.bpm();
        let r = ui
            .add(
                DragValue::new(&mut bpm)
                    .range(10.0..=522.0)
                    .speed(0.2)
                    .fixed_decimals(3)
                    .suffix(" BPM"),
            )
            .on_hover_text("Tempo: drag or type (10–522 BPM)");
        let cmd = r.changed().then(|| Command::SetTempo(Tempo::from_bpm(bpm)));
        app.exec_gesture(&r, cmd);
        let mut sig = app.project().time_signature;
        let (mut n, mut d) = (sig.numerator, sig.denominator);
        let a = ui
            .add(DragValue::new(&mut n).range(1..=32))
            .on_hover_text("Beats per bar");
        ui.label(RichText::new("/").color(t.text_dim));
        let b = ui
            .add(DragValue::new(&mut d).range(1..=32))
            .on_hover_text("Beat unit (power of two)");
        if a.changed() || b.changed() {
            sig = TimeSignature::new(n, d);
            app.exec(Command::SetTimeSignature(sig));
        }
        // Position display.
        let pos = app.player.status().position();
        let sig = app.project().time_signature;
        let bbt = BarBeatTick::from_ticks(pos, sig);
        let secs = Timeline::new(app.project().tempo, 48_000).seconds(pos);
        let (r, resp) = ui.allocate_exact_size(vec2(96.0, B), Sense::click());
        ui.painter().rect_filled(r, CornerRadius::same(3), t.bg);
        ui.painter().text(
            r.center() - vec2(0.0, 5.0),
            Align2::CENTER_CENTER,
            bbt.to_string(),
            mono(14.0),
            t.accent,
        );
        ui.painter().text(
            r.center() + vec2(0.0, 8.0),
            Align2::CENTER_CENTER,
            format!("{}:{:05.2}", (secs / 60.0) as u32, secs % 60.0),
            mono(9.0),
            t.text_dim,
        );
        if resp.clicked() {
            app.locate(0);
        }
        resp.on_hover_text("Song position (bar:beat:tick and minutes:seconds). Click to return to the start.");
    });
}

fn pattern_selector(app: &mut LimeShotApp, ui: &mut Ui) {
    group(ui, |ui| {
        let cur = app.current_pattern();
        let pats: Vec<_> = app
            .project()
            .patterns
            .iter()
            .map(|p| (p.id, p.name.clone(), p.color))
            .collect();
        let name = pats
            .iter()
            .find(|p| p.0 == cur)
            .map(|p| p.1.clone())
            .unwrap_or_default();
        if icon_button(ui, Icon::Minus, false, 20.0, "Previous pattern (−)").clicked()
            && let Some(i) = pats.iter().position(|p| p.0 == cur)
            && i > 0
        {
            app.set_current_pattern(pats[i - 1].0);
        }
        ComboBox::from_id_salt("pattern_selector")
            .width(110.0)
            .selected_text(name)
            .show_ui(ui, |ui| {
                for (id, n, c) in &pats {
                    let text = RichText::new(format!("■ {n}")).color(crate::theme::rgb(*c));
                    if ui.selectable_label(*id == cur, text).clicked() {
                        app.set_current_pattern(*id);
                    }
                }
            });
        if icon_button(
            ui,
            Icon::Plus,
            false,
            20.0,
            "Next pattern (+); after the last, adds a new one",
        )
        .clicked()
        {
            match pats.iter().position(|p| p.0 == cur) {
                Some(i) if i + 1 < pats.len() => app.set_current_pattern(pats[i + 1].0),
                _ => crate::menus::add_pattern(app),
            }
        }
        if ui.small_button("…").on_hover_text("Rename pattern").clicked() {
            app.dialog = Some(Dialog::rename_pattern(app, cur));
        }
    });
}

fn snap_and_options(app: &mut LimeShotApp, ui: &mut Ui) {
    group(ui, |ui| {
        ComboBox::from_id_salt("snap")
            .width(70.0)
            .selected_text(app.ui.snap.label())
            .show_ui(ui, |ui| {
                for s in Snap::ALL {
                    ui.selectable_value(&mut app.ui.snap, s, s.label());
                }
            })
            .response
            .on_hover_text("Snap: grid for the Piano Roll and Playlist (hold Alt to bypass)");
        if icon_button(ui, Icon::Metronome, app.ui.metronome, B, "Metronome (Cmd+M)").clicked() {
            let m = !app.ui.metronome;
            app.set_metronome(m);
        }
        if icon_button(ui, Icon::Loop, app.ui.song_loop, B, "Loop the song in Song mode").clicked() {
            let l = !app.ui.song_loop;
            app.set_song_loop(l);
        }
    });
}

fn windows(app: &mut LimeShotApp, ui: &mut Ui) {
    group(ui, |ui| {
        let u = &mut app.ui;
        if icon_button(ui, Icon::Browser, u.show_browser, B, "Browser (Alt+F8)").clicked() {
            u.show_browser ^= true;
        }
        if icon_button(ui, Icon::Playlist, u.show_playlist, B, "Playlist (F5)").clicked() {
            u.show_playlist ^= true;
        }
        if icon_button(ui, Icon::ChannelRack, u.show_rack, B, "Channel Rack (F6)").clicked() {
            u.show_rack ^= true;
        }
        if icon_button(ui, Icon::PianoRoll, u.show_piano, B, "Piano Roll (F7)").clicked() {
            u.show_piano ^= true;
        }
        if icon_button(ui, Icon::Mixer, u.show_mixer, B, "Mixer (F9)").clicked() {
            u.show_mixer ^= true;
        }
    });
}

fn edit_buttons(app: &mut LimeShotApp, ui: &mut Ui) {
    group(ui, |ui| {
        let undo_tip = app
            .doc
            .undo_label()
            .map(|l| format!("Undo {l}"))
            .unwrap_or_else(|| "Nothing to undo".into());
        if icon_button(ui, Icon::Undo, false, B, &undo_tip).clicked() {
            app.doc.undo();
        }
        let redo_tip = app
            .doc
            .redo_label()
            .map(|l| format!("Redo {l}"))
            .unwrap_or_else(|| "Nothing to redo".into());
        if icon_button(ui, Icon::Redo, false, B, &redo_tip).clicked() {
            app.doc.redo();
        }
        if icon_button(ui, Icon::Save, app.doc.is_modified(), B, "Save (Cmd+S)").clicked() {
            app.save();
        }
        if icon_button(ui, Icon::Export, false, B, "Export WAV (Cmd+R)").clicked() {
            app.dialog = Some(Dialog::export(app));
        }
    });
}

fn output(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    group(ui, |ui| {
        let (l, r) = app.player.status().peak(0);
        if app.player.status().take_clipped() {
            app.clip_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
        }
        let clipped = app.clip_until.is_some_and(|u| u > std::time::Instant::now());
        meter(ui, vec2(14.0, B), l, r, clipped).on_hover_text("Master output");
        let load = app.player.status().load();
        let (rect, resp) = ui.allocate_exact_size(vec2(44.0, B), Sense::hover());
        ui.painter()
            .rect_stroke(rect, 3.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let f = load.clamp(0.0, 1.0);
        let fill = egui::Rect::from_min_size(
            rect.min + vec2(1.0, 1.0),
            vec2((rect.width() - 2.0) * f, rect.height() - 2.0),
        );
        ui.painter()
            .rect_filled(fill, 2.0, if load > 0.8 { t.danger } else { t.accent_dim });
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("{:.0}%", load * 100.0),
            mono(10.0),
            t.text,
        );
        resp.on_hover_text("Audio load: time spent rendering per buffer");
    });
}

pub fn status_bar(app: &mut LimeShotApp, ui: &mut Ui) {
    let t = Theme::current();
    ui.horizontal_centered(|ui| {
        let hint = if app.ui.hint.is_empty() {
            "Ready".to_string()
        } else {
            app.ui.hint.clone()
        };
        ui.label(RichText::new(hint).font(prop(11.5)).color(t.text_dim));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let out = match app.player.output_description() {
                Ok(name) => {
                    let buf = app
                        .player
                        .buffer_frames()
                        .map(|b| format!(", {b} frames"))
                        .unwrap_or_default();
                    RichText::new(format!("{name} · {} Hz{buf}", app.player.sample_rate())).color(t.text_dim)
                }
                Err(reason) => RichText::new(format!("No audio output: {reason}")).color(t.warning),
            };
            if ui
                .add(egui::Label::new(out.font(prop(11.0))).sense(Sense::click()))
                .on_hover_text("Audio settings")
                .clicked()
            {
                app.dialog = Some(Dialog::audio(app));
            }
            ui.label(
                RichText::new(format!("{} voices", app.player.status().voices()))
                    .font(prop(11.0))
                    .color(t.text_dim),
            );
        });
    });
}
