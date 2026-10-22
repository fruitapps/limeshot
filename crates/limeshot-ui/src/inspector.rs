//! The Channel Settings window: the selected channel's instrument parameters (FL Studio opens
//! this window when you click a channel button in the Channel Rack).

use crate::app::LimeShotApp;
use crate::browser;
use crate::theme::{Theme, mono, rgb, semibold};
use crate::widgets::Hint as _;
use crate::widgets::{reset_on_double_click as reset, section, segmented};
use egui::{Align2, CornerRadius, DragValue, RichText, Sense, Stroke, Ui, pos2, vec2};
use limeshot_core::*;

/// Body of the Channel Settings window for the selected channel.
pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    page(app, ui, app.selected_channel());
}

/// The Channel Settings of channel `cid` (also the Settings tab of a plugin channel's window).
pub fn channel_page(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId) {
    page(app, ui, Some(cid));
}

fn page(app: &mut LimeShotApp, ui: &mut Ui, cid: Option<ChannelId>) {
    let t = Theme::current();
    let Some(ch) = cid.and_then(|c| app.project().channel(c).cloned()) else {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| ui.label(RichText::new("Add a channel to edit its sound.").color(t.text_dim)));
        return;
    };
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(3), rgb(ch.color));
                let name = ui
                    .add(egui::Label::new(RichText::new(&ch.name).font(semibold(15.0))).sense(Sense::click()))
                    .hint("Click to rename");
                if name.clicked() {
                    app.dialog = Some(crate::dialogs::Dialog::rename_channel(app, ch.id));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "→ Mixer {}",
                            if ch.mixer_track == 0 {
                                "M".to_string()
                            } else {
                                ch.mixer_track.to_string()
                            }
                        ))
                        .font(mono(10.5))
                        .color(t.text_dim),
                    );
                });
            });
            ui.add_space(8.0);
            channel_body(app, ui, &ch);
        });
    });
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            vec2(76.0, 22.0),
            egui::Label::new(RichText::new(label).color(Theme::current().text_dim)).truncate(),
        );
        // Leave room for the value box ("1.5 kHz", "300 ms").
        ui.spacing_mut().slider_width = (ui.available_width() - 84.0).max(60.0);
        add(ui);
    });
}

fn channel_body(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        // The Sampler/LimeSynth switch is for the built-in instruments; a plugin channel keeps its
        // plugin (owner's list item 6).
        let kind = match ch.instrument {
            Instrument::Sampler(_) => Some(0),
            Instrument::Synth(_) => Some(1),
            Instrument::Plugin(_) | Instrument::Unsupported(_) => None,
        };
        if let Some(kind) = kind
            && let Some(kind) = segmented(ui, &["Sampler", "LimeSynth"], kind, 24.0, "Instrument type")
        {
            let inst = if kind == 0 {
                Instrument::Sampler(SamplerParams::default())
            } else {
                Instrument::Synth(SynthParams::default())
            };
            app.exec(Command::SetInstrument {
                channel: ch.id,
                instrument: inst,
            });
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut key = ch.root_key;
            let r = ui.add(
                DragValue::new(&mut key)
                    .range(0..=127)
                    .custom_formatter(|v, _| crate::piano_roll::key_name(v as u8)),
            );
            if crate::widgets::user_changed(&r) {
                app.exec(Command::SetChannelRootKey { channel: ch.id, key });
            }
            ui.label(RichText::new("Root").color(t.text_dim));
            crate::presets_ui::menu(app, ui, crate::presets_ui::Target::Channel(ch.id));
        });
    });
    ui.add_space(6.0);
    match &ch.instrument {
        Instrument::Sampler(p) => sampler_settings(app, ui, ch.id, p),
        Instrument::Synth(p) => synth_settings(app, ui, ch.id, p),
        Instrument::Plugin(p) => crate::plugins::channel_settings(app, ui, ch.id, p),
        Instrument::Unsupported(_) => {
            ui.label(RichText::new("This channel uses an instrument this version of LimeShot does not support. It is kept in the project but stays silent.").color(t.warning));
        }
    }
}

/// Draw sampler controls and its waveform, applying parameter edits as undoable commands.
/// Pressing a loaded waveform auditions the trimmed, pitched region; dragging carries its sample.
fn sampler_settings(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId, p: &SamplerParams) {
    let t = Theme::current();
    let set = |p: SamplerParams| Command::SetInstrument {
        channel: cid,
        instrument: Instrument::Sampler(p),
    };
    let status = p.sample.as_ref().and_then(|s| app.samples.status(&s.path).cloned());
    section(ui, "Sample");
    ui.horizontal(|ui| {
        let label = match &p.sample {
            Some(s) => browser::file_label(std::path::Path::new(&s.path)),
            None => "No sample".into(),
        };
        ui.label(RichText::new(label).font(semibold(12.5)));
        if ui.button("Load…").clicked()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio files", &limeshot_dsp::AUDIO_EXTENSIONS)
                .pick_file()
        {
            browser::assign_sample(app, cid, &path);
        }
    });
    // Waveform clicks audition the full region; dragging carries the sample to another editor.
    let (r, wave) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::click_and_drag());
    if wave.is_pointer_button_down_on()
        && ui.input(|i| i.pointer.primary_pressed())
        && let Some(key) = app.project().channel(cid).map(|c| c.root_key)
        && let Some(limeshot_audio::assets::SampleStatus::Ready(s)) = &status
    {
        let duration =
            s.duration_secs() * f64::from((p.end - p.start).max(0.0)) / 2.0_f64.powf(f64::from(p.pitch) / 12.0);
        app.preview_for(cid, key, 0.78, duration);
        app.sampler_preview = Some((cid, std::time::Instant::now(), duration));
    }
    if wave.drag_started()
        && let Some(sample) = &p.sample
    {
        let path = app.samples.resolve(&sample.path);
        wave.dnd_set_drag_payload(browser::DraggedSample(path));
    }
    if wave.hovered() && p.sample.is_some() {
        app.hint("Sample: press to hear it through the channel");
    }
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), t.sunken);
    match &status {
        Some(limeshot_audio::assets::SampleStatus::Ready(s)) => {
            let frames = s.frames().max(1);
            let cols = r.width() as usize;
            for x in 0..cols {
                let a = x * frames / cols.max(1);
                let b = ((x + 1) * frames / cols.max(1)).max(a + 1);
                let peak = (a..b)
                    .step_by(((b - a) / 32).max(1))
                    .map(|i| s.frame(i).0.abs())
                    .fold(0.0f32, f32::max);
                let h = peak.min(1.0) * r.height() * 0.48;
                let xx = r.left() + x as f32;
                ui.painter().line_segment(
                    [pos2(xx, r.center().y - h), pos2(xx, r.center().y + h)],
                    Stroke::new(1.0, t.accent_dim),
                );
            }
            if p.looping {
                // The loop region, shaded.
                let (a, b) = (p.loop_start.min(p.loop_end), p.loop_start.max(p.loop_end));
                let lr = egui::Rect::from_min_max(
                    pos2(r.left() + a * r.width(), r.top()),
                    pos2(r.left() + b * r.width(), r.bottom()),
                );
                ui.painter().rect_filled(lr, 0.0, t.accent.gamma_multiply(0.12));
            }
            for (f, c) in [(p.start, t.accent), (p.end, t.danger)] {
                let x = r.left() + f * r.width();
                ui.painter()
                    .line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.5, c));
            }
            if let Some(progress) = sample_progress(app, cid, p, s.duration_secs()) {
                let f = if p.reverse {
                    p.end - progress * (p.end - p.start)
                } else {
                    p.start + progress * (p.end - p.start)
                };
                let x = r.left() + f * r.width();
                ui.painter()
                    .line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.5, t.playhead));
                ui.ctx().request_repaint();
            }
            ui.label(
                RichText::new(format!("{:.2} s · {} Hz", s.duration_secs(), s.sample_rate))
                    .small()
                    .color(t.text_dim),
            );
        }
        Some(limeshot_audio::assets::SampleStatus::Failed(e)) => {
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                "Could not load sample",
                mono(11.0),
                t.danger,
            );
            ui.label(RichText::new(e).small().color(t.danger));
        }
        Some(limeshot_audio::assets::SampleStatus::Loading) => {
            ui.painter()
                .text(r.center(), Align2::CENTER_CENTER, "Loading…", mono(11.0), t.text_dim);
        }
        None => {
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                "Drop or load a sample (WAV, MP3, FLAC, OGG, AIFF)",
                mono(11.0),
                t.text_dim,
            );
        }
    }
    let mut q = p.clone();
    // Double-click resets a slider to the instrument's default.
    let d = SamplerParams::default();
    let mut changed: Option<egui::Response> = None;
    let mut track = |r: egui::Response, c: bool| {
        if c || r.drag_started() || r.drag_stopped() {
            changed = Some(r);
        }
    };
    ui.add_space(4.0);
    section(ui, "Playback");
    row(ui, "Gain", |ui| {
        let r = reset(
            ui.add(egui::Slider::new(&mut q.gain_db, -24.0..=12.0).suffix(" dB")),
            &mut q.gain_db,
            d.gain_db,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Pitch", |ui| {
        let r = reset(
            ui.add(
                egui::Slider::new(&mut q.pitch, -24.0..=24.0)
                    .suffix(" st")
                    .step_by(0.01),
            ),
            &mut q.pitch,
            d.pitch,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Start", |ui| {
        let r = reset(
            ui.add(egui::Slider::new(&mut q.start, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))),
            &mut q.start,
            d.start,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "End", |ui| {
        let r = reset(
            ui.add(egui::Slider::new(&mut q.end, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))),
            &mut q.end,
            d.end,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Attack", |ui| {
        let r = reset(
            ui.add(
                egui::Slider::new(&mut q.attack_ms, 0.0..=2000.0)
                    .logarithmic(true)
                    .max_decimals(1)
                    .suffix(" ms"),
            ),
            &mut q.attack_ms,
            d.attack_ms,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Decay", |ui| {
        let r = reset(
            ui.add(
                egui::Slider::new(&mut q.decay_ms, 0.0..=4000.0)
                    .logarithmic(true)
                    .max_decimals(1)
                    .suffix(" ms"),
            ),
            &mut q.decay_ms,
            d.decay_ms,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Sustain", |ui| {
        let r = reset(
            ui.add(egui::Slider::new(&mut q.sustain, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))),
            &mut q.sustain,
            d.sustain,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Release", |ui| {
        let r = reset(
            ui.add(
                egui::Slider::new(&mut q.release_ms, 0.0..=2000.0)
                    .logarithmic(true)
                    .suffix(" ms"),
            ),
            &mut q.release_ms,
            d.release_ms,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Playback", |ui| {
        let r = ui.checkbox(&mut q.reverse, "Reverse");
        let c = crate::widgets::user_changed(&r);
        track(r, c);
        let r = ui
            .checkbox(&mut q.one_shot, "One-shot")
            .hint("Play the whole sample, ignoring note length");
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    ui.add_space(4.0);
    section(ui, "Loop");
    row(ui, "Loop", |ui| {
        let r = ui
            .checkbox(&mut q.looping, "Loop while held")
            .hint("Repeat the loop region for as long as the note lasts");
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    if q.looping {
        let pct = |v: f64, _| format!("{:.0}%", v * 100.0);
        row(ui, "Loop start", |ui| {
            let r = reset(
                ui.add(egui::Slider::new(&mut q.loop_start, 0.0..=1.0).custom_formatter(pct)),
                &mut q.loop_start,
                d.loop_start,
            );
            let c = crate::widgets::user_changed(&r);
            track(r, c);
        });
        row(ui, "Loop end", |ui| {
            let r = reset(
                ui.add(egui::Slider::new(&mut q.loop_end, 0.0..=1.0).custom_formatter(pct)),
                &mut q.loop_end,
                d.loop_end,
            );
            let c = crate::widgets::user_changed(&r);
            track(r, c);
        });
    }
    ui.add_space(4.0);
    section(ui, "Filter");
    row(ui, "Cutoff", |ui| {
        let r = reset(
            ui.add(
                egui::Slider::new(&mut q.cutoff_hz, 20.0..=20_000.0)
                    .logarithmic(true)
                    .suffix(" Hz"),
            ),
            &mut q.cutoff_hz,
            d.cutoff_hz,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    row(ui, "Resonance", |ui| {
        let r = reset(
            ui.add(egui::Slider::new(&mut q.resonance, 0.0..=0.95)),
            &mut q.resonance,
            d.resonance,
        );
        let c = crate::widgets::user_changed(&r);
        track(r, c);
    });
    if q.end < q.start {
        q.end = q.start;
    }
    if q.loop_end < q.loop_start {
        q.loop_end = q.loop_start;
    }
    if let Some(r) = changed {
        let cmd = (q != *p).then(|| set(q));
        app.exec_gesture(&r, cmd);
    }
}

fn synth_settings(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId, p: &SynthParams) {
    let mut q = p.clone();
    let d = SynthParams::default();
    let mut waveform_changed = false;
    let mut changed: Option<egui::Response> = None;
    let mut track = |r: egui::Response| {
        if crate::widgets::user_changed(&r) || r.drag_started() || r.drag_stopped() {
            changed = Some(r);
        }
    };
    section(ui, "Oscillator");
    ui.horizontal(|ui| {
        let labels: Vec<&str> = Waveform::ALL.iter().map(|w| w.label()).collect();
        let cur = Waveform::ALL.iter().position(|w| *w == q.waveform).unwrap_or(0);
        if let Some(i) = segmented(ui, &labels, cur, 24.0, "Waveform")
            && let Some(w) = Waveform::ALL.get(i)
        {
            q.waveform = *w;
            waveform_changed = true;
        }
    });
    row(ui, "Octave", |ui| {
        track(reset(
            ui.add(egui::Slider::new(&mut q.octave, -3..=3)),
            &mut q.octave,
            d.octave,
        ))
    });
    row(ui, "Detune", |ui| {
        track(
            reset(
                ui.add(egui::Slider::new(&mut q.detune_cents, -50.0..=50.0).suffix(" ct")),
                &mut q.detune_cents,
                d.detune_cents,
            )
            .hint("A second, detuned oscillator; 0 turns it off"),
        )
    });
    row(ui, "Gain", |ui| {
        track(reset(
            ui.add(egui::Slider::new(&mut q.gain_db, -36.0..=12.0).suffix(" dB")),
            &mut q.gain_db,
            d.gain_db,
        ))
    });
    ui.add_space(4.0);
    section(ui, "Envelope");
    row(ui, "Attack", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.attack_ms, 0.0..=2000.0)
                    .logarithmic(true)
                    .suffix(" ms"),
            ),
            &mut q.attack_ms,
            d.attack_ms,
        ))
    });
    row(ui, "Decay", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.decay_ms, 0.0..=4000.0)
                    .logarithmic(true)
                    .suffix(" ms"),
            ),
            &mut q.decay_ms,
            d.decay_ms,
        ))
    });
    row(ui, "Sustain", |ui| {
        track(reset(
            ui.add(egui::Slider::new(&mut q.sustain, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}%", v * 100.0))),
            &mut q.sustain,
            d.sustain,
        ))
    });
    row(ui, "Release", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.release_ms, 0.0..=4000.0)
                    .logarithmic(true)
                    .suffix(" ms"),
            ),
            &mut q.release_ms,
            d.release_ms,
        ))
    });
    // ADSR picture.
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), t.sunken);
    let total = (q.attack_ms + q.decay_ms + q.release_ms).max(1.0) * 1.25;
    let x = |ms: f32| r.left() + 4.0 + ms / total * (r.width() - 8.0);
    let y = |lvl: f32| r.bottom() - 4.0 - lvl * (r.height() - 8.0);
    let hold = total * 0.2;
    let pts = vec![
        pos2(x(0.0), y(0.0)),
        pos2(x(q.attack_ms), y(1.0)),
        pos2(x(q.attack_ms + q.decay_ms), y(q.sustain)),
        pos2(x(q.attack_ms + q.decay_ms + hold), y(q.sustain)),
        pos2(x(q.attack_ms + q.decay_ms + hold + q.release_ms), y(0.0)),
    ];
    ui.painter().add(egui::Shape::line(pts, Stroke::new(1.5, t.accent)));
    ui.add_space(4.0);
    section(ui, "Filter");
    row(ui, "Cutoff", |ui| {
        track(
            reset(
                ui.add(
                    egui::Slider::new(&mut q.cutoff_hz, 20.0..=20_000.0)
                        .logarithmic(true)
                        .custom_formatter(|v, _| {
                            if v >= 19_999.0 {
                                "Open".into()
                            } else if v >= 1000.0 {
                                format!("{:.1} kHz", v / 1000.0)
                            } else {
                                format!("{v:.0} Hz")
                            }
                        }),
                ),
                &mut q.cutoff_hz,
                d.cutoff_hz,
            )
            .hint("Low-pass cutoff (12 dB/oct); fully right leaves the filter open"),
        )
    });
    row(ui, "Resonance", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.resonance, 0.0..=0.95).custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
            ),
            &mut q.resonance,
            d.resonance,
        ))
    });
    row(ui, "Envelope", |ui| {
        track(
            reset(
                ui.add(
                    egui::Slider::new(&mut q.filter_env, 0.0..=8.0)
                        .fixed_decimals(1)
                        .suffix(" oct"),
                ),
                &mut q.filter_env,
                d.filter_env,
            )
            .hint("How far each note opens the cutoff at its start"),
        )
    });
    row(ui, "Env decay", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.filter_decay_ms, 1.0..=4000.0)
                    .logarithmic(true)
                    .suffix(" ms"),
            ),
            &mut q.filter_decay_ms,
            d.filter_decay_ms,
        ))
    });
    ui.add_space(4.0);
    section(ui, "LFO");
    row(ui, "Rate", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.lfo_rate_hz, 0.05..=20.0)
                    .logarithmic(true)
                    .suffix(" Hz"),
            ),
            &mut q.lfo_rate_hz,
            d.lfo_rate_hz,
        ))
    });
    row(ui, "Vibrato", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.vibrato_cents, 0.0..=100.0)
                    .fixed_decimals(0)
                    .suffix(" ct"),
            ),
            &mut q.vibrato_cents,
            d.vibrato_cents,
        ))
    });
    row(ui, "To cutoff", |ui| {
        track(reset(
            ui.add(
                egui::Slider::new(&mut q.lfo_cutoff, 0.0..=4.0)
                    .fixed_decimals(1)
                    .suffix(" oct"),
            ),
            &mut q.lfo_cutoff,
            d.lfo_cutoff,
        ))
    });
    if waveform_changed {
        app.exec(Command::SetInstrument {
            channel: cid,
            instrument: Instrument::Synth(q),
        });
    } else if let Some(r) = changed {
        let cmd = (q != *p).then_some(Command::SetInstrument {
            channel: cid,
            instrument: Instrument::Synth(q),
        });
        app.exec_gesture(&r, cmd);
    }
}

/// Estimate fractional progress through the trimmed sample for the waveform's playback head.
/// `sample_seconds` is the full, unpitched sample duration. Prefer an active sampler preview,
/// otherwise use the most recent eligible note or audio clip; return `None` if none qualifies.
fn sample_progress(app: &LimeShotApp, cid: ChannelId, params: &SamplerParams, sample_seconds: f64) -> Option<f32> {
    if let Some((channel, began, duration)) = app.sampler_preview
        && channel == cid
        && duration > 0.0
        && began.elapsed().as_secs_f64() < duration
    {
        return Some((began.elapsed().as_secs_f64() / duration) as f32);
    }
    if !app.is_playing() {
        return None;
    }
    let tick = app.player.status().position();
    let seconds_per_tick = 60.0 / (app.project().tempo.bpm() * PPQ as f64);
    let span = sample_seconds * f64::from(params.end - params.start);
    if span <= 0.0 {
        return None;
    }
    let channel = app.project().channel(cid)?;
    let progress = |start: Tick, key: u8| {
        let ratio = 2.0_f64.powf((f64::from(key) - f64::from(channel.root_key) + f64::from(params.pitch)) / 12.0);
        ((tick - start) as f64 * seconds_per_tick * ratio / span) as f32
    };
    let mut candidates = Vec::new();
    if app.ui.mode == limeshot_engine::PlayMode::Pattern {
        let pattern = app.project().pattern(app.current_pattern())?;
        for n in pattern
            .notes_for(cid)
            .filter(|n| !n.muted && n.start <= tick && (params.one_shot || tick < n.end()))
        {
            candidates.push((n.start, progress(n.start, n.key)));
        }
    } else {
        for clip in app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| !c.muted && c.start <= tick && tick < c.end())
        {
            if app
                .project()
                .playlist
                .tracks
                .get(clip.track as usize)
                .is_some_and(|t| t.muted)
            {
                continue;
            }
            if clip.audio_channel() == Some(cid) {
                let elapsed = (tick - clip.start + clip.offset) as f64 * seconds_per_tick;
                candidates.push((
                    clip.start,
                    (elapsed * 2.0_f64.powf(f64::from(params.pitch) / 12.0) / span) as f32,
                ));
            } else if let Some(pattern) = clip.pattern().and_then(|p| app.project().pattern(p)) {
                for n in pattern.notes_for(cid).filter(|n| !n.muted && n.start >= clip.offset) {
                    let start = clip.start + n.start - clip.offset;
                    if start <= tick && (params.one_shot || tick < start + n.length) {
                        candidates.push((start, progress(start, n.key)));
                    }
                }
            }
        }
    }
    candidates
        .into_iter()
        .filter(|(_, p)| (0.0..1.0).contains(p))
        .max_by_key(|(start, _)| *start)
        .map(|(_, p)| p)
}
