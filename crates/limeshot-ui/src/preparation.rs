//! Fixed-duration sample preparation dialog and cancellable worker. No audio-thread work.
use crate::app::LimeShotApp;
use limeshot_core::{ChannelId, ClipId, Command, Instrument, PluginState, PluginTarget, lime};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, channel},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Channel(ChannelId),
    Clip(ClipId),
}
struct Job {
    rx: Receiver<Result<String, String>>,
    cancel: Arc<AtomicBool>,
    target: Target,
    expected: PluginState,
    revision: u64,
    generation: u64,
    fit: Option<u64>,
    restore: bool,
}
pub struct State {
    pub target: Option<Target>,
    pub ratio: f64,
    source_seconds: f64,
    has_original: bool,
    pub pitch: f64,
    pub stretch: bool,
    pub beats: f64,
    pub use_beats: bool,
    pub fit: bool,
    job: Option<Job>,
    pub status: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            target: None,
            ratio: 1.,
            source_seconds: 0.,
            has_original: false,
            pitch: 0.,
            stretch: true,
            beats: 4.,
            use_beats: false,
            fit: true,
            job: None,
            status: String::new(),
        }
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
}
fn plugin(app: &LimeShotApp, target: Target) -> Option<&limeshot_core::PluginRef> {
    let channel = match target {
        Target::Channel(id) => id,
        Target::Clip(id) => app.project().clip(id)?.audio_channel()?,
    };
    let Instrument::Plugin(p) = &app.project().channel(channel)?.instrument else {
        return None;
    };
    (lime::LimePlugin::of(p) == Some(lime::LimePlugin::SAMPLER)).then_some(p)
}
pub fn open(app: &mut LimeShotApp, target: Target) {
    if app.preparation.job.is_none() && plugin(app, target).is_some() {
        app.preparation.target = Some(target);
        app.preparation.status.clear();
        let recipe = plugin(app, target)
            .and_then(lime::sample_state)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|s| s.get("preparation-v1").cloned());
        app.preparation.has_original = recipe.is_some();
        app.preparation.ratio = recipe.as_ref().and_then(|p| p["ratio"].as_f64()).unwrap_or(1.);
        app.preparation.pitch = recipe.as_ref().and_then(|p| p["pitch"].as_f64()).unwrap_or(0.);
        app.preparation.stretch = recipe.as_ref().is_none_or(|p| p["mode"].as_str() != Some("Resample"));
        app.preparation.source_seconds = plugin(app, target)
            .and_then(|p| {
                let sample: serde_json::Value = serde_json::from_str(&lime::sample_state(p)?).ok()?;
                let original = sample
                    .get("preparation-v1")
                    .and_then(|p| p.get("original"))
                    .unwrap_or(&sample);
                let text = original["pcm"].as_str()?;
                let bytes = (text.len() / 4 * 3).saturating_sub(text.bytes().rev().take_while(|b| *b == b'=').count());
                let params = lime::sampler_params(&Instrument::Plugin(p.clone()))?;
                Some(
                    (bytes / 8) as f64 / original["sample_rate"].as_u64()? as f64
                        * f64::from((params.end - params.start).abs())
                        / 2f64.powf(f64::from(params.pitch) / 12.),
                )
            })
            .unwrap_or(0.);
        if let Target::Clip(id) = target
            && let Some(c) = app.project().clip(id)
        {
            app.preparation.beats = (c.length + c.offset) as f64 / 960.;
            let target = app.preparation.beats * 60. / app.project().tempo.bpm();
            app.preparation.use_beats = (0.5..=2.).contains(&(target / app.preparation.source_seconds));
        }
    }
}
pub fn poll(app: &mut LimeShotApp) {
    let result = app.preparation.job.as_ref().and_then(|j| j.rx.try_recv().ok());
    if let Some(result) = result {
        let Some(job) = app.preparation.job.take() else { return };
        if job.cancel.load(Ordering::Relaxed) {
            app.preparation.status = "Cancelled".into();
            return;
        }
        if app.doc.revision() != job.revision
            || app.doc.generation() != job.generation
            || plugin(app, job.target).is_none_or(|p| p.state != job.expected)
        {
            app.preparation.status = "Project changed while preparing; retry".into();
            return;
        }
        match result.and_then(|text| {
            let p = plugin(app, job.target).ok_or("Sampler removed")?;
            lime::with_sample_state(p, &text).ok_or_else(|| "Prepared state exceeds limits".into())
        }) {
            Ok(state) => {
                match job.target {
                    Target::Channel(id) => app.exec(Command::SetPluginState {
                        plugin: PluginTarget::Channel(id),
                        state,
                    }),
                    Target::Clip(clip) => app.exec(Command::PrepareAudioClip {
                        clip,
                        expected: job.expected,
                        state,
                        length: job.fit,
                    }),
                };
                app.preparation.has_original = !job.restore;
                app.preparation.status = if job.restore {
                    "Original restored."
                } else {
                    "Sample prepared."
                }
                .into();
            }
            Err(error) => app.preparation.status = error,
        }
    }
}
pub fn start(app: &mut LimeShotApp, restore: bool) {
    if app.preparation.job.is_some() {
        return;
    }
    let Some(target) = app.preparation.target else { return };
    let Some(p) = plugin(app, target) else { return };
    let expected = p.state.clone();
    let Some(text) = lime::sample_state(p) else {
        app.preparation.status = "Load an embedded sample first".into();
        return;
    };
    let Ok(sample) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    let source = sample
        .get("preparation-v1")
        .and_then(|p| p.get("original"))
        .unwrap_or(&sample);
    let encoded = source["pcm"].as_str().unwrap_or("");
    let bytes = (encoded.len() / 4 * 3).saturating_sub(encoded.bytes().rev().take_while(|b| *b == b'=').count());
    let frames = bytes / 8;
    let rate = source["sample_rate"].as_u64().unwrap_or(1) as f64;
    let params = lime::sampler_params(&Instrument::Plugin(p.clone())).unwrap_or_default();
    let seconds =
        frames as f64 / rate * f64::from((params.end - params.start).abs()) / 2f64.powf(f64::from(params.pitch) / 12.);
    let mut ratio = if restore { 1. } else { app.preparation.ratio };
    if app.preparation.use_beats && !restore {
        ratio = (app.preparation.beats * 60. / app.project().tempo.bpm()) / seconds;
    }
    let pitch = if app.preparation.stretch {
        app.preparation.pitch
    } else {
        0.
    };
    let stretch = app.preparation.stretch;
    let fit = if app.preparation.fit {
        match target {
            Target::Clip(id) => app.project().clip(id).map(|c| {
                ((seconds * ratio * app.project().tempo.bpm() / 60. * 960.).round() as u64)
                    .saturating_sub(c.offset)
                    .max(1)
            }),
            _ => None,
        }
    } else {
        None
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    let (tx, rx) = channel();
    let spawned = std::thread::Builder::new()
        .name("lime-sample-prepare".into())
        .spawn(move || {
            let result = if restore {
                limeshot_sequencer::preparation::restore_sample(&text)
            } else {
                limeshot_sequencer::preparation::prepare_sample(&text, ratio, pitch, stretch, &worker_cancel)
            };
            let _ = tx.send(result);
        });
    if let Err(error) = spawned {
        app.preparation.status = error.to_string();
        return;
    }
    app.preparation.job = Some(Job {
        rx,
        cancel,
        target,
        expected,
        revision: app.doc.revision(),
        generation: app.doc.generation(),
        fit,
        restore,
    });
    app.preparation.status = "Preparing…".into();
}
pub fn show(app: &mut LimeShotApp, ctx: &egui::Context) {
    if app.preparation.target.is_none() {
        return;
    }
    let mut open = true;
    let mut prepare = false;
    let mut restore = false;
    crate::dialogs::modal_with_id(ctx, "sample-prepare", "Time stretching", 380., |ui| {
        let busy = app.preparation.job.is_some();
        let theme = crate::theme::Theme::current();
        let scope = match app.preparation.target {
            Some(Target::Clip(_)) => "Only this audio clip changes.",
            _ => "Affects all clips and notes using this channel.",
        };
        ui.label(egui::RichText::new(scope).small().color(theme.text_dim));
        ui.add_space(6.);
        ui.add_enabled_ui(!busy,|ui| {
            egui::Grid::new("sample_prepare_grid").num_columns(2).min_col_width(76.).spacing(egui::vec2(12.,6.)).show(ui,|ui| {
                ui.label("Mode");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut app.preparation.stretch,false,"Resample");
                    ui.selectable_value(&mut app.preparation.stretch,true,"Stretch");
                });ui.end_row();
                ui.label("Length");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut app.preparation.use_beats,false,"Multiplier");
                    ui.selectable_value(&mut app.preparation.use_beats,true,"Beats");
                });ui.end_row();
                ui.label("Duration");
                if app.preparation.use_beats {
                    ui.add(egui::DragValue::new(&mut app.preparation.beats).range(0.125..=128.).speed(0.125).suffix(" beats"))
                        .on_hover_text("Uses the current tempo once. For bars, enter bars × beats per bar.");
                }else{
                    ui.add(egui::Slider::new(&mut app.preparation.ratio,0.5..=2.).suffix(" ×"));
                }ui.end_row();
                ui.label("Pitch");
                ui.add_enabled(app.preparation.stretch,egui::Slider::new(&mut app.preparation.pitch,-12. ..=12.).suffix(" st"))
                    .on_hover_text("Stretch changes pitch without changing the prepared duration. Channel Pitch and keyboard notes still change playback speed.");
                ui.end_row();
            });
        });
        let duration = if app.preparation.use_beats {
            app.preparation.beats * 60. / app.project().tempo.bpm()
        } else {
            app.preparation.source_seconds * app.preparation.ratio
        };
        let ratio = duration / app.preparation.source_seconds;
        ui.add_space(6.);
        ui.label(
            egui::RichText::new(format!("{:.3} s → {:.3} s", app.preparation.source_seconds, duration)).monospace(),
        );
        if app.preparation.source_seconds <= 0. {
            ui.colored_label(
                theme.warning,
                "Load an embedded sample first. Turn off Keep on disk to prepare it.",
            );
        } else if !(0.5..=2.).contains(&ratio) {
            ui.colored_label(theme.warning, "Choose a duration between 0.5× and 2× the original.");
        }
        ui.label(egui::RichText::new("Original audio is preserved. Tempo changes do not update prepared audio.").small().color(theme.text_dim))
            .on_hover_text("Up to 3 million frames per source/result. Sources shorter than 20 ms resample. Larger pitch intermediates are rejected.");
        if !app.preparation.status.is_empty() {
            ui.label(&app.preparation.status);
        }
        ui.add_space(6.);
        ui.horizontal(|ui| {
            prepare = ui
                .add_enabled(
                    !busy && (0.5..=2.).contains(&ratio),
                    egui::Button::new(egui::RichText::new("Prepare").strong()),
                )
                .clicked();
            restore = ui
                .add_enabled(
                    !busy && app.preparation.has_original,
                    egui::Button::new("Restore original"),
                )
                .clicked();
            if ui.button(if busy { "Cancel" } else { "Close" }).clicked() {
                open = false;
            }
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }
    if prepare || restore {
        start(app, restore);
    }
    if !open {
        if let Some(job) = &app.preparation.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
        app.preparation.target = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use limeshot_core::*;
    fn app() -> (LimeShotApp, ClipId, ChannelId) {
        let mut app = LimeShotApp::new_silent(crate::settings::Settings::default(), None);
        let p = LimePlugin::SAMPLER.with_values(
            &lime::sampler_values(&SamplerParams::default(), 60),
            false,
            Some(&EmbeddedAudio {
                name: "tone".into(),
                sample_rate: 8000,
                stereo: (0..8000).flat_map(|i| [(i as f32 * 0.3).sin(); 2]).collect(),
            }),
        );
        let channel = ChannelId(
            app.exec(Command::AddChannel {
                name: "source".into(),
                instrument: Instrument::Plugin(p),
            })
            .unwrap(),
        );
        let clip = ClipId(
            app.exec(Command::PlaceClip {
                source: ClipSource::Audio(channel),
                track: 0,
                start: 0,
                length: 960,
                offset: 120,
            })
            .unwrap(),
        );
        (app, clip, channel)
    }
    fn finish(app: &mut LimeShotApp) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.preparation.job.is_some() && std::time::Instant::now() < deadline {
            poll(app);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(app.preparation.job.is_none());
    }
    #[test]
    fn clip_worker_isolates_notes_preserves_controls_reopens_and_undoes() {
        let (mut app, clip, channel) = app();
        let original = app.project().channel(channel).unwrap().clone();
        open(&mut app, Target::Clip(clip));
        app.preparation.use_beats = false;
        app.preparation.ratio = 2.;
        app.preparation.pitch = 12.;
        start(&mut app, false);
        finish(&mut app);
        assert!(
            app.preparation.status == "Sample prepared.",
            "{}",
            app.preparation.status
        );
        assert_eq!(app.project().channel(channel).unwrap(), &original);
        let unique = app.project().clip(clip).unwrap().audio_channel().unwrap();
        assert_ne!(unique, channel);
        assert_eq!(app.project().clip(clip).unwrap().offset, 120);
        let Instrument::Plugin(p) = &app.project().channel(unique).unwrap().instrument else {
            panic!()
        };
        let text = lime::sample_state(p).unwrap();
        assert!(text.contains("preparation-v1"));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prepared.lime");
        limeshot_project::save(app.project(), &path).unwrap();
        let loaded = limeshot_project::load(&path).unwrap();
        assert_eq!(&loaded, app.project());
        assert!(app.doc.undo());
        assert_eq!(app.project().channels.len(), 1);
        assert_eq!(app.project().clip(clip).unwrap().audio_channel(), Some(channel));
        assert!(app.doc.redo());
        assert_eq!(&loaded, app.project());
        open(&mut app, Target::Clip(clip));
        app.preparation.use_beats = false;
        app.preparation.ratio = 1.;
        start(&mut app, false);
        finish(&mut app);
        assert_eq!(
            app.project().channels.len(),
            2,
            "repreparing a private clip reuses its channel"
        );
        assert_eq!(app.project().clip(clip).unwrap().audio_channel(), Some(unique));
        assert!(app.doc.undo());
        assert_eq!(&loaded, app.project());
        app.exec(Command::CloneArrangement {
            index: 0,
            name: Some("Other".into()),
        });
        app.exec(Command::SelectArrangement { index: 0 });
        let shared = app.project().channel(unique).unwrap().clone();
        open(&mut app, Target::Clip(clip));
        app.preparation.use_beats = false;
        app.preparation.ratio = 1.;
        start(&mut app, false);
        finish(&mut app);
        assert_eq!(
            app.project().channel(unique).unwrap(),
            &shared,
            "inactive arrangements retain their shared source"
        );
        assert_ne!(app.project().clip(clip).unwrap().audio_channel(), Some(unique));
    }
    #[test]
    fn cancel_and_concurrent_edits_never_overwrite_the_document() {
        for cancel in [true, false] {
            let (mut app, clip, _) = app();
            open(&mut app, Target::Clip(clip));
            start(&mut app, false);
            if cancel {
                app.preparation
                    .job
                    .as_ref()
                    .unwrap()
                    .cancel
                    .store(true, Ordering::Relaxed);
            } else {
                app.exec(Command::SetTempo(Tempo::from_bpm(130.)));
            }
            let expected = app.project().clone();
            finish(&mut app);
            assert_eq!(app.project(), &expected);
        }
    }
}

#[cfg(test)]
mod arrangement_tests {
    use limeshot_core::*;
    #[test]
    fn automatic_crossfades_follow_moves_and_restore_manual_fades_with_undo() {
        let mut doc = Document::default();
        let channel = ChannelId(
            doc.execute(Command::AddChannel {
                name: "audio".into(),
                instrument: Instrument::Plugin(LimePlugin::SAMPLER.plugin_ref()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let mut ids = Vec::new();
        for start in [0, 600] {
            ids.push(ClipId(
                doc.execute(Command::PlaceClip {
                    source: ClipSource::Audio(channel),
                    track: 0,
                    start,
                    length: 960,
                    offset: 0,
                })
                .unwrap()
                .0
                .unwrap(),
            ));
        }
        doc.execute(Command::SetClipFades {
            clip: ids[0],
            fade_in: 50,
            fade_out: 20,
        })
        .unwrap();
        doc.execute(Command::SetAutoCrossfades(true)).unwrap();
        assert_eq!(doc.project().clip(ids[0]).unwrap().fade_out, 360);
        assert_eq!(doc.project().clip(ids[1]).unwrap().fade_in, 360);
        assert_eq!(doc.project().clip(ids[0]).unwrap().fade_curve(true), 2.);
        doc.execute(Command::MoveClips {
            clips: vec![ids[1]],
            dt: 960,
            dtrack: 0,
        })
        .unwrap();
        assert_eq!(doc.project().clip(ids[0]).unwrap().fade_out, 20);
        assert_eq!(doc.project().clip(ids[1]).unwrap().fade_in, 0);
        assert!(doc.undo());
        assert_eq!(doc.project().clip(ids[0]).unwrap().fade_out, 360);
        doc.execute(Command::SetAutoCrossfades(false)).unwrap();
        assert_eq!(doc.project().clip(ids[0]).unwrap().fade_out, 20);
    }
}
