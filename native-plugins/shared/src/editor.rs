use crate::{
    parameters::{self, NativeParams},
    processor::Meters,
    sample::EmbeddedSample,
};
use egui::{self, *};
use limeshot_project::preset::{self, Content, Slot};
use nice_plug::{context::gui::GuiContext, prelude::*};
use nice_plug_egui::NiceEguiApp;
use std::{sync::Arc, time::Duration};

mod controls;
mod displays;
mod layouts;
mod style;

use style::{Canvas, Theme};

pub const EDITOR_SIZES: [(f32, f32); 12] = [
    (850.0, 600.0),
    (760.0, 650.0),
    (520.0, 300.0),
    (940.0, 580.0),
    (650.0, 370.0),
    (900.0, 440.0),
    (850.0, 460.0),
    (720.0, 380.0),
    (900.0, 530.0),
    (760.0, 420.0),
    (760.0, 420.0),
    (760.0, 420.0),
];
/// Return the default editor width and height in logical points for plugin IDs 0–11.
///
/// # Panics
/// Panics for an ID outside 0–11.
pub fn editor_size(id: u8) -> (f32, f32) {
    EDITOR_SIZES[id as usize]
}
/// Return 90% of the default editor dimensions in logical points.
///
/// # Panics
/// Panics for a plugin ID outside 0–11.
pub fn minimum_size(id: u8) -> (f32, f32) {
    let (w, h) = editor_size(id);
    (w * 0.9, h * 0.9)
}

/// Install the bundled fonts and Lime Dark styling in the supplied egui context.
pub fn install_theme(ctx: &egui::Context) {
    let t = Theme::current();
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "Mono".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "Mono".into());
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = Visuals::dark();
    style.visuals.panel_fill = t.bg;
    style.visuals.window_fill = t.surface;
    style.visuals.extreme_bg_color = t.sunken;
    style.visuals.override_text_color = Some(t.text);
    style.visuals.selection.bg_fill = t.accent.gamma_multiply(0.25);
    style.visuals.selection.stroke = Stroke::new(1.0, t.accent);
    style.visuals.widgets.inactive.bg_fill = t.raised;
    style.visuals.widgets.hovered.bg_fill = t.raised;
    style.visuals.widgets.active.bg_fill = t.accent.gamma_multiply(0.3);
    style.spacing.item_spacing = vec2(10.0, 8.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub struct NativeEditor {
    id: u8,
    params: Arc<NativeParams>,
    meters: Arc<Meters>,
    context: Option<GuiContext>,
    status: String,
    preset_name: String,
    entry: String,
    editing: Option<usize>,
    sample_loading: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    sample_name: String,
    sample: Option<Arc<limeshot_dsp::SampleData>>,
    waveform: [f32; 256],
    history: [[f32; 3]; 256],
    history_cursor: usize,
    history_time: f64,
}

impl NativeEditor {
    /// Create editor state sharing plugin parameters and meters, initially without a host context.
    pub(crate) fn new(id: u8, params: Arc<NativeParams>, meters: Arc<Meters>) -> Self {
        Self {
            id,
            params,
            meters,
            context: None,
            status: String::new(),
            preset_name: "Init".into(),
            entry: String::new(),
            editing: None,
            sample_loading: None,
            sample_name: String::new(),
            sample: None,
            waveform: [0.0; 256],
            history: [[0.0; 3]; 256],
            history_cursor: 0,
            history_time: -1.0,
        }
    }
    /// Create an editor with independent default parameters and meters for offscreen use.
    /// Use a plugin ID in 0–11; drawing requires a valid ID. No host context is attached.
    pub fn preview(id: u8) -> Self {
        Self::new(id, Arc::new(NativeParams::new(id)), Arc::new(Meters::default()))
    }
    /// Create an offscreen sampler editor with embedded audio.
    /// Propagates file/header, decode, and size-limit errors from [`EmbeddedSample::load`];
    /// embedded data rejected by the sample store instead leaves the preview empty.
    pub fn preview_sampler(path: &std::path::Path) -> Result<Self, crate::sample::SampleError> {
        use nice_plug::params::persist::PersistentField;
        let editor = Self::preview(1);
        editor.params.sample.set(EmbeddedSample::load(path)?);
        Ok(editor)
    }
    fn slot(&self) -> Slot {
        match self.id {
            0 => Slot::Synth,
            1 => Slot::Sampler,
            _ => Slot::Effect(parameters::KINDS[(self.id - 2) as usize]),
        }
    }

    /// Draw the plugin and handle parameter, preset, sample, and audition interactions.
    /// Shared by the embedded editor and the screenshot/interaction harness.
    /// File actions may open dialogs and perform I/O; failures appear in the status text.
    /// Host parameter/state edits require an attached context. Requests a repaint after 33 ms.
    pub fn draw(&mut self, ui: &mut Ui) {
        if let Some(result) = self.sample_loading.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.sample_loading = None;
            match result {
                Ok(sample) => {
                    if let Some(context) = &self.context {
                        let mut state = context.get_state();
                        state.fields.insert("sample-v1".into(), sample);
                        context.set_state(state);
                        self.status = "Sample loaded".into();
                    }
                }
                Err(error) => self.status = error,
            }
        }
        if let Some(disk) = self.params.sample.audio().as_ref().and_then(|s| s.disk.clone()) {
            if disk.failed() {
                self.status = "Disk sample read failed; reload its source".into();
            } else if disk.underruns() > 0 {
                self.status = format!("Disk read-ahead missed {} frames; reduce disk load", disk.underruns());
            }
        }
        let t = Theme::for_plugin(self.id);
        let params = self.params.clone();
        let context = self.context.clone();
        let setter = context.as_ref().map(|c| c.param_setter());
        let (whole, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let c = Canvas::new(whole, editor_size(self.id));
        let painter = ui.painter_at(whole);
        painter.rect_filled(whole, 3, t.bg);
        self.paint_bypass(ui, c, &t);
        let mut load_factory = None;
        let mut load_file = false;
        let mut save_file = false;
        ui.scope_builder(
            UiBuilder::new().max_rect(c.r(c.w - 245.0, c.h - 57.0, 148.0, 25.0)),
            |ui| {
                ui.menu_button(format!("Presets: {}", self.preset_name), |ui| {
                    if ui.button("Init / reset all controls").clicked() {
                        load_factory = Some(None);
                        ui.close();
                    }
                    ui.separator();
                    for p in preset::factory(self.slot()) {
                        if ui.button(&p.name).clicked() {
                            load_factory = Some(Some(p));
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Open .limepreset…").clicked() {
                        load_file = true;
                        ui.close();
                    }
                    if ui.button("Save .limepreset…").clicked() {
                        save_file = true;
                        ui.close();
                    }
                });
            },
        );
        // The redundant branded header is gone; keep presets/bypass in the footer instead.
        let body = Canvas {
            origin: c.origin - vec2(0.0, 43.0 * c.s),
            ..c
        };
        self.paint_plugin(ui, body, &t);
        let status = if self.status.is_empty() {
            "Drag: adjust   ·   Shift: fine   ·   Double-click: reset   ·   Click value: type"
        } else {
            &self.status
        };
        painter.rect_filled(c.r(0.0, c.h - 23.0, c.w, 23.0), 0, t.header);
        painter.text(
            c.p(12.0, c.h - 11.0),
            Align2::LEFT_CENTER,
            status,
            FontId::proportional(10.0 * c.s),
            t.dim,
        );
        painter.text(
            c.p(c.w - 12.0, c.h - 11.0),
            Align2::RIGHT_CENTER,
            "LIME NATIVE",
            FontId::monospace(9.0 * c.s),
            t.accent,
        );
        if !ui.ctx().input(|i| i.pointer.primary_down()) {
            self.meters.preview_note.store(0, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(preset) = load_factory {
            if let Some(p) = preset {
                self.apply_preset(p);
            } else {
                if let Some(s) = &setter {
                    for p in &params.controls {
                        set(s, &p.value, p.value.default_plain_value());
                    }
                }
                self.preset_name = "Init".into();
            }
        }
        if load_file
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Lime preset", &["limepreset"])
                .pick_file()
        {
            match preset::load(&path, self.slot()) {
                Ok(p) => self.apply_preset(p),
                Err(e) => self.status = e.to_string(),
            }
        }
        if save_file
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Lime preset", &["limepreset"])
                .set_file_name("My preset.limepreset")
                .save_file()
        {
            let p = preset::Preset {
                name: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "My preset".into()),
                content: content(self.id, &params.values()),
            };
            match save_preset_file(&path, &p) {
                Ok(()) => {
                    self.preset_name = p.name;
                    self.status = "Preset saved".into();
                }
                Err(e) => self.status = e,
            }
        }
        ui.ctx().request_repaint_after(Duration::from_millis(33));
    }

    /// Apply preset controls through host gestures and update the displayed preset name.
    /// Sampler presets preserve embedded audio and root key. Does nothing without a host context.
    fn apply_preset(&mut self, preset: preset::Preset) {
        let Some(context) = &self.context else {
            return;
        };
        let values = values(&preset.content);
        let setter = context.param_setter();
        for (p, value) in self.params.controls.iter().zip(values) {
            if value.is_finite() {
                set(&setter, &p.value, value);
            }
        }
        self.preset_name = preset.name;
        self.status.clear();
    }

    /// Load supported audio into the host's embedded-sample state, preserving other state fields.
    /// Load or serialization errors become status text. Does nothing without a host context.
    fn load_sample(&mut self, path: &std::path::Path) {
        self.load_sample_mode(path, None);
    }

    fn load_sample_mode(&mut self, path: &std::path::Path, disk: Option<bool>) {
        if self.context.is_none() {
            return;
        }
        let path = path.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();
        self.sample_loading = Some(rx);
        self.status = "Preparing sample…".into();
        std::thread::spawn(move || {
            let sample = match disk {
                Some(mode) => EmbeddedSample::load_mode(&path, mode),
                None => EmbeddedSample::load(&path),
            };
            let result = sample
                .map_err(|e| e.to_string())
                .and_then(|sample| serde_json::to_string(&sample).map_err(|e| e.to_string()));
            let _ = tx.send(result);
        });
    }
}

impl NiceEguiApp for NativeEditor {
    fn build(
        &mut self,
        ctx: egui::Context,
        context: GuiContext,
        _frame: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        install_theme(&ctx);
        self.context = Some(context);
        Ok(())
    }
    fn ui(&mut self, ui: &mut Ui, _frame: &mut nice_plug_egui::Frame) {
        self.draw(ui);
    }
    /// Clear the audition request and detach the host context when the editor closes.
    fn editor_closed(&mut self) {
        self.meters.preview_note.store(0, std::sync::atomic::Ordering::Relaxed);
        self.context = None;
    }
}

/// Set a plain parameter value within one complete host automation gesture.
fn set<P: Param>(setter: &ParamSetter<'_>, p: &P, value: P::Plain) {
    setter.begin_set_parameter(p);
    setter.set_parameter(p, value);
    setter.end_set_parameter(p);
}

/// Convert preset content to plain native controls in specification order.
/// Sampler content excludes audio and root key (NaN, skipped when applied) so applying it
/// preserves both.
fn values(content: &Content) -> Vec<f32> {
    match content {
        Content::Effect { params, .. } => params.clone(),
        Content::Synth(s) => vec![
            match s.waveform {
                limeshot_core::Waveform::Sine => 0.0,
                limeshot_core::Waveform::Square => 1.0,
                limeshot_core::Waveform::Saw => 2.0,
                limeshot_core::Waveform::Triangle => 3.0,
            },
            s.octave as f32,
            s.detune_cents,
            s.gain_db,
            s.attack_ms,
            s.decay_ms,
            s.sustain,
            s.release_ms,
            s.cutoff_hz,
            s.resonance,
            s.filter_env,
            s.filter_decay_ms,
            s.lfo_rate_hz,
            s.vibrato_cents,
            s.lfo_cutoff,
            match s.filter_mode {
                limeshot_core::FilterMode::LowPass => 0.0,
                limeshot_core::FilterMode::BandPass => 1.0,
                limeshot_core::FilterMode::HighPass => 2.0,
            },
            s.key_track,
        ],
        Content::Sampler(s) => vec![
            s.gain_db,
            s.pitch,
            s.start,
            s.end,
            s.release_ms,
            f32::from(s.reverse),
            f32::from(s.one_shot),
            // The root key is the channel's, not the preset's: kept as it is.
            f32::NAN,
            s.attack_ms,
            s.decay_ms,
            s.sustain,
            s.cutoff_hz,
            s.resonance,
            f32::from(s.looping),
            s.loop_start,
            s.loop_end,
        ],
    }
}
/// Convert plain native controls to preset content, omitting bypass and external-key toggles.
/// Sampler content omits embedded audio and root key.
fn content(id: u8, p: &[f32; parameters::MAX_CONTROLS]) -> Content {
    match id {
        0 => Content::Synth(limeshot_core::SynthParams {
            waveform: match p[0].round() as u8 {
                0 => limeshot_core::Waveform::Sine,
                1 => limeshot_core::Waveform::Square,
                3 => limeshot_core::Waveform::Triangle,
                _ => limeshot_core::Waveform::Saw,
            },
            octave: p[1].round() as i8,
            detune_cents: p[2],
            gain_db: p[3],
            attack_ms: p[4],
            decay_ms: p[5],
            sustain: p[6],
            release_ms: p[7],
            cutoff_hz: p[8],
            resonance: p[9],
            filter_env: p[10],
            filter_decay_ms: p[11],
            lfo_rate_hz: p[12],
            vibrato_cents: p[13],
            lfo_cutoff: p[14],
            filter_mode: match p[15].round() as u8 {
                1 => limeshot_core::FilterMode::BandPass,
                2 => limeshot_core::FilterMode::HighPass,
                _ => limeshot_core::FilterMode::LowPass,
            },
            key_track: p[16],
        }),
        1 => Content::Sampler(limeshot_core::SamplerParams {
            sample: None,
            gain_db: p[0],
            pitch: p[1],
            start: p[2],
            end: p[3],
            release_ms: p[4],
            reverse: p[5] >= 0.5,
            one_shot: p[6] >= 0.5,
            attack_ms: p[8],
            decay_ms: p[9],
            sustain: p[10],
            cutoff_hz: p[11],
            resonance: p[12],
            looping: p[13] >= 0.5,
            loop_start: p[14],
            loop_end: p[15],
        }),
        _ => Content::Effect {
            kind: parameters::KINDS[(id - 2) as usize],
            params: p[..parameters::specs(id).len()].to_vec(),
        },
    }
}

/// Write or overwrite a preset with LimeShot's format marker and version 1 envelope.
/// Returns serialization, invalid-object, or filesystem errors as strings.
fn save_preset_file(path: &std::path::Path, preset: &preset::Preset) -> Result<(), String> {
    let mut object = serde_json::to_value(preset).map_err(|e| e.to_string())?;
    let map = object.as_object_mut().ok_or_else(|| "Invalid preset".to_string())?;
    map.insert("format".into(), "limeshot-preset".into());
    map.insert("version".into(), 1.into());
    let data = serde_json::to_vec_pretty(&object).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_presets_round_trip_through_limeshots_reader() {
        let folder = tempfile::tempdir().unwrap();
        for id in 0..parameters::NAMES.len() as u8 {
            let params = NativeParams::new(id);
            let path = folder.path().join(format!("{id}.limepreset"));
            let original = preset::Preset {
                name: "Test".into(),
                content: content(id, &params.values()),
            };
            save_preset_file(&path, &original).unwrap();
            let loaded = preset::load(&path, NativeEditor::preview(id).slot()).unwrap();
            let got = values(&loaded.content);
            for (want, got) in params.values().iter().zip(&got) {
                assert!(got.is_nan() || want == got, "{want} vs {got}");
            }
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn all_plugin_controls_fit_and_do_not_overlap_at_supported_sizes() {
        for id in 0..parameters::NAMES.len() as u8 {
            let (w, h) = editor_size(id);
            let (mw, mh) = minimum_size(id);
            for size in [vec2(w, h), vec2(mw, mh)] {
                let mut editor = NativeEditor::preview(id);
                let mut initialized = false;
                let mut harness = egui_kittest::Harness::builder().with_size(size).build_ui(move |ui| {
                    if !initialized {
                        install_theme(ui.ctx());
                        initialized = true;
                        return;
                    }
                    editor.draw(ui);
                });
                harness.run_steps(3);
                let mut rectangles = Vec::new();
                for spec in parameters::specs(id).iter().take(if id == 3 { 7 } else { usize::MAX }) {
                    let rect = harness
                        .ctx
                        .data(|d| d.get_temp::<Rect>(Id::unique(("native-control", spec.name))))
                        .unwrap();
                    assert!(
                        Rect::from_min_size(Pos2::ZERO, size).contains_rect(rect),
                        "{}: {} outside editor: {rect:?}",
                        parameters::NAMES[id as usize],
                        spec.name
                    );
                    for (name, previous) in &rectangles {
                        assert!(
                            !rect.intersects(*previous),
                            "{}: {} overlaps {name}",
                            parameters::NAMES[id as usize],
                            spec.name
                        );
                    }
                    rectangles.push((spec.name, rect));
                }
            }
        }
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn synced_delay_cancels_manual_time_entry_and_ignores_clicks() {
        let mut editor = NativeEditor::preview(5);
        Arc::get_mut(&mut editor.params).unwrap().controls[5].value = nice_plug::prelude::FloatParam::new(
            "Sync",
            1.0,
            nice_plug::prelude::FloatRange::Linear { min: 0.0, max: 8.0 },
        );
        editor.editing = Some(0);
        editor.entry = "2000".into();
        let mut harness = egui_kittest::Harness::builder()
            .with_size(vec2(850.0, 600.0))
            .build_ui_state(
                |ui, (editor, initialized)| {
                    if !*initialized {
                        install_theme(ui.ctx());
                        *initialized = true;
                        return;
                    }
                    editor.draw(ui);
                },
                (editor, false),
            );
        harness.run_steps(3);
        assert_eq!(harness.state().0.editing, None);
        let position = harness
            .ctx
            .data(|d| d.get_temp::<Rect>(Id::unique(("native-value", "Time"))))
            .unwrap()
            .center();
        harness.input_mut().events.push(Event::PointerMoved(position));
        harness.step();
        for pressed in [true, false] {
            harness.input_mut().events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
        }
        assert_eq!(harness.state().0.editing, None);
    }

    #[test]
    fn numeric_value_click_selects_text_and_accepts_direct_input() {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(vec2(850.0, 600.0))
            .build_ui_state(
                |ui, (editor, initialized)| {
                    if !*initialized {
                        install_theme(ui.ctx());
                        *initialized = true;
                        return;
                    }
                    editor.draw(ui);
                },
                (NativeEditor::preview(0), false),
            );
        harness.run_steps(3);
        let position = harness
            .ctx
            .data(|d| d.get_temp::<Rect>(Id::unique(("native-value", "Octave"))))
            .unwrap()
            .center();
        harness.input_mut().events.push(Event::PointerMoved(position));
        harness.step();
        for pressed in [true, false] {
            harness.input_mut().events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
        }
        assert_eq!(harness.state().0.editing, Some(1));
        harness.input_mut().events.push(Event::Text("-2".into()));
        harness.step();
        assert_eq!(harness.state().0.entry, "-2", "typing replaces the selected readout");
        harness.input_mut().events.push(Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
        assert_eq!(harness.state().0.editing, None);
        assert!(harness.state().0.status.is_empty());
    }

    #[test]
    fn waveform_and_keyboard_send_audition_and_release_requests() {
        let (w, h) = editor_size(1);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/samples/Lime Snare.wav");
        let editor = NativeEditor::preview_sampler(&path).unwrap();
        let mut harness = egui_kittest::Harness::builder().with_size(vec2(w, h)).build_ui_state(
            |ui, (editor, initialized)| {
                if !*initialized {
                    install_theme(ui.ctx());
                    *initialized = true;
                    return;
                }
                editor.draw(ui);
            },
            (editor, false),
        );
        harness.run_steps(3);
        for (target, expected) in [("sampler-waveform", 61), ("sampler-keyboard", 49)] {
            let rect = harness.ctx.data(|d| d.get_temp::<Rect>(Id::unique(target))).unwrap();
            let pos = if target == "sampler-keyboard" {
                pos2(rect.left() + rect.width() / 42.0, rect.top() + rect.height() * 0.8)
            } else {
                rect.center()
            };
            harness.input_mut().events.push(Event::PointerMoved(pos));
            harness.step();
            harness.input_mut().events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
            assert_eq!(
                harness.state().0.meters.preview_note.load(Ordering::Relaxed),
                expected,
                "{}",
                target
            );
            harness.input_mut().events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
            assert_eq!(harness.state().0.meters.preview_note.load(Ordering::Relaxed), 0);
        }
    }
}
