use super::displays::grid;
use super::style::panel;
use super::{Canvas, NativeEditor, Theme, set};
use crate::processor::Meters;
use egui::*;
use std::f32::consts::TAU;

impl NativeEditor {
    pub(super) fn paint_plugin(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        match self.id {
            0 => self.synth(ui, c, t),
            1 => self.sampler(ui, c, t),
            2 => self.utility(ui, c, t),
            3 => self.eq(ui, c, t),
            4 => self.filter(ui, c, t),
            5 => self.delay(ui, c, t),
            6 => self.room(ui, c, t),
            7 => self.comp(ui, c, t),
            8 => self.limiter(ui, c, t),
            _ => self.additional_effect(ui, c, t),
        }
    }
    /// Draw appended effect controls from the shared native parameter schema.
    fn additional_effect(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        panel(ui.painter(), c.r(12.0, 55.0, 736.0, 330.0), "SOUND / CHARACTER", t, c.s);
        for (index, spec) in crate::parameters::specs(self.id).iter().enumerate() {
            let col = index % 3;
            let row = index / 3;
            self.knob(
                ui,
                c,
                c.r(30.0 + col as f32 * 242.0, 88.0 + row as f32 * 145.0, 212.0, 124.0),
                index,
                spec.name,
                t.accent,
                t,
            );
        }
    }
    fn synth(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        panel(
            ui.painter(),
            c.r(10.0, 53.0, 830.0, 132.0),
            "OSCILLATOR / TUNING",
            t,
            c.s,
        );
        self.choices(
            ui,
            c,
            c.r(28.0, 91.0, 288.0, 42.0),
            0,
            &["SINE", "SQUARE", "SAW", "TRIANGLE"],
            t.accent,
            t,
        );
        let v = self.params.values();
        let wave = c.r(28.0, 142.0, 288.0, 25.0);
        grid(ui.painter(), wave, t, c.s);
        let points = (0..100)
            .map(|i| {
                let x = (i as f32 / 99.0 * 3.0).fract();
                let y = match v[0].round() as u8 {
                    0 => (x * TAU).sin(),
                    1 => {
                        if x < 0.5 {
                            0.8
                        } else {
                            -0.8
                        }
                    }
                    3 => 1.0 - 4.0 * (x - 0.5).abs(),
                    _ => 2.0 * x - 1.0,
                };
                pos2(
                    egui::lerp(wave.x_range(), i as f32 / 99.0),
                    wave.center().y - y * wave.height() * 0.4,
                )
            })
            .collect();
        ui.painter().add(Shape::line(points, Stroke::new(c.s, t.accent)));
        for (index, x, label) in [
            (1, 352.0, "COARSE / OCTAVE"),
            (2, 500.0, "FINE / DETUNE"),
            (3, 648.0, "OUTPUT GAIN"),
        ] {
            self.knob(ui, c, c.r(x, 84.0, 122.0, 91.0), index, label, t.accent, t);
        }
        panel(ui.painter(), c.r(10.0, 195.0, 470.0, 210.0), "VOLUME ENVELOPE", t, c.s);
        self.envelope(ui.painter(), c, c.r(26.0, 229.0, 438.0, 58.0), t);
        for (i, label) in [(4, "ATTACK"), (5, "DECAY"), (6, "SUSTAIN"), (7, "RELEASE")] {
            self.knob(
                ui,
                c,
                c.r(30.0 + (i - 4) as f32 * 110.0, 297.0, 98.0, 96.0),
                i,
                label,
                t.cyan,
                t,
            );
        }
        panel(ui.painter(), c.r(490.0, 195.0, 350.0, 210.0), "FILTER", t, c.s);
        self.choices(
            ui,
            c,
            c.r(508.0, 222.0, 314.0, 24.0),
            15,
            &["LOW-PASS", "BAND-PASS", "HIGH-PASS"],
            t.amber,
            t,
        );
        for (i, x, y, label) in [
            (8, 506.0, 252.0, "CUTOFF"),
            (9, 613.0, 252.0, "RESONANCE"),
            (16, 720.0, 252.0, "KEY TRACK"),
            (10, 506.0, 328.0, "ENV AMOUNT"),
            (11, 613.0, 328.0, "ENV DECAY"),
        ] {
            self.knob(ui, c, c.r(x, y, 104.0, 74.0), i, label, t.amber, t);
        }
        panel(ui.painter(), c.r(10.0, 415.0, 830.0, 151.0), "LFO / MODULATION", t, c.s);
        for (index, x, label) in [
            (12, 30.0, "RATE"),
            (13, 192.0, "PITCH DEPTH"),
            (14, 354.0, "CUTOFF DEPTH"),
        ] {
            self.knob(ui, c, c.r(x, 450.0, 140.0, 99.0), index, label, t.purple, t);
        }
        self.scope(ui.painter(), c, c.r(536.0, 453.0, 210.0, 86.0), t.accent, t);
        self.meters(ui.painter(), c, c.r(758.0, 446.0, 65.0, 105.0), t);
    }
    fn sampler(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        let file = c.r(12.0, 54.0, 736.0, 38.0);
        ui.painter().rect_filled(file, 2, t.surface);
        let name = self.params.sample.name();
        ui.painter().text(
            c.p(25.0, 73.0),
            Align2::LEFT_CENTER,
            if name.is_empty() {
                "File   (no sample loaded)".into()
            } else {
                format!("File   {}", shorten(&name, 46))
            },
            FontId::proportional(12.0 * c.s),
            t.text,
        );
        if self.button(ui, c, c.r(618.0, 60.0, 63.0, 26.0), "Load…", t)
            && let Some(path) = rfd::FileDialog::new().add_filter("WAV audio", &["wav"]).pick_file()
        {
            self.load_sample(&path);
        }
        if self.button(ui, c, c.r(687.0, 60.0, 51.0, 26.0), "Clear", t)
            && let Some(context) = &self.context
            && let Ok(sample) = serde_json::to_string(&crate::sample::EmbeddedSample::default())
        {
            let mut state = context.get_state();
            state.fields.insert("sample-v1".into(), sample);
            context.set_state(state);
            self.status = "Sample cleared".into();
        }
        panel(ui.painter(), c.r(12.0, 103.0, 359.0, 153.0), "PLAYBACK", t, c.s);
        self.slider(ui, c, c.r(22.0, 135.0, 339.0, 34.0), 0, "Gain", t);
        self.slider(ui, c, c.r(22.0, 174.0, 339.0, 34.0), 1, "Pitch", t);
        self.lamp(ui, c, c.r(22.0, 219.0, 145.0, 25.0), 6, "One-shot", t);
        self.lamp(ui, c, c.r(185.0, 219.0, 150.0, 25.0), 5, "Reverse", t);
        panel(
            ui.painter(),
            c.r(381.0, 103.0, 367.0, 153.0),
            "SAMPLE REGION / DECLICK",
            t,
            c.s,
        );
        self.slider(ui, c, c.r(391.0, 135.0, 347.0, 34.0), 2, "Start offset", t);
        self.slider(ui, c, c.r(391.0, 174.0, 347.0, 34.0), 3, "End offset", t);
        self.slider(ui, c, c.r(391.0, 212.0, 347.0, 34.0), 4, "Release", t);
        self.sample_waveform(ui, c, c.r(12.0, 270.0, 736.0, 128.0), t);
        self.slider(ui, c, c.r(12.0, 404.0, 367.0, 34.0), 11, "Cutoff", t);
        self.slider(ui, c, c.r(381.0, 404.0, 367.0, 34.0), 12, "Resonance", t);
        for (index, x, label) in [(8, 12.0, "Attack"), (9, 258.0, "Decay"), (10, 504.0, "Sustain")] {
            self.slider(ui, c, c.r(x, 442.0, 244.0, 34.0), index, label, t);
        }
        self.lamp(ui, c, c.r(12.0, 484.0, 96.0, 25.0), 13, "Loop", t);
        self.slider(ui, c, c.r(112.0, 480.0, 316.0, 34.0), 14, "Loop start", t);
        self.slider(ui, c, c.r(432.0, 480.0, 316.0, 34.0), 15, "Loop end", t);
        let info = if let Some(sample) = &self.sample {
            format!(
                "{:.2} seconds  ·  {} Hz  ·  Stereo",
                sample.duration_secs(),
                sample.sample_rate
            )
        } else {
            "WAV sample player  ·  Drag a file onto the waveform".into()
        };
        ui.painter().text(
            c.p(24.0, 523.0),
            Align2::LEFT_CENTER,
            info,
            FontId::monospace(10.0 * c.s),
            t.dim,
        );
        self.slider(ui, c, c.r(12.0, 533.0, 367.0, 32.0), 7, "Root key", t);
        ui.painter().text(
            c.p(736.0, 549.0),
            Align2::RIGHT_CENTER,
            "CLICK: AUDITION    RIGHT-CLICK: ROOT",
            FontId::monospace(9.0 * c.s),
            t.dim,
        );
        self.keyboard(ui, c, c.r(12.0, 575.0, 736.0, 48.0), t);
    }
    fn utility(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        panel(ui.painter(), c.r(10.0, 54.0, 500.0, 210.0), "STEREO UTILITY", t, c.s);
        self.knob(ui, c, c.r(22.0, 88.0, 128.0, 153.0), 1, "STEREO WIDTH", t.accent, t);
        self.knob(ui, c, c.r(170.0, 88.0, 128.0, 153.0), 0, "VOLUME", t.lime, t);
        ui.painter()
            .line_segment([c.p(318.0, 81.0), c.p(318.0, 248.0)], Stroke::new(c.s, t.edge));
        ui.painter().text(
            c.p(367.0, 99.0),
            Align2::CENTER_CENTER,
            "POLARITY",
            FontId::proportional(10.0 * c.s),
            t.dim,
        );
        self.choices(ui, c, c.r(332.0, 114.0, 78.0, 62.0), 2, &["+", "−"], t.accent, t);
        self.meters(ui.painter(), c, c.r(423.0, 83.0, 70.0, 160.0), t);
        ui.painter().text(
            c.p(366.0, 209.0),
            Align2::CENTER_CENTER,
            "L / R",
            FontId::monospace(20.0 * c.s),
            t.accent,
        );
        ui.painter().text(
            c.p(366.0, 236.0),
            Align2::CENTER_CENTER,
            "STEREO",
            FontId::monospace(9.0 * c.s),
            t.dim,
        );
    }
    fn eq(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        self.eq_graph(ui, c, c.r(12.0, 55.0, 700.0, 333.0), t);
        panel(ui.painter(), c.r(723.0, 55.0, 205.0, 333.0), "BAND LEVEL", t, c.s);
        for (i, x, label, color) in [
            (0, 735.0, "LOW", t.purple),
            (2, 798.0, "MID", t.accent),
            (5, 861.0, "HIGH", t.cyan),
        ] {
            self.fader(ui, c, c.r(x, 85.0, 55.0, 288.0), i, label, color, t);
        }
        for (x, title, color) in [
            (12.0, "LOW SHELF", t.purple),
            (318.0, "PARAMETRIC MID", t.accent),
            (624.0, "HIGH SHELF", t.cyan),
        ] {
            panel(ui.painter(), c.r(x, 399.0, 304.0, 146.0), title, t, c.s);
            ui.painter().rect_filled(c.r(x + 1.0, 400.0, 4.0, 144.0), 0, color);
        }
        self.knob(ui, c, c.r(91.0, 434.0, 136.0, 101.0), 1, "FREQUENCY", t.purple, t);
        self.knob(ui, c, c.r(331.0, 434.0, 136.0, 101.0), 3, "FREQUENCY", t.accent, t);
        self.knob(ui, c, c.r(476.0, 434.0, 132.0, 101.0), 4, "WIDTH / Q", t.accent, t);
        self.knob(ui, c, c.r(703.0, 434.0, 136.0, 101.0), 6, "FREQUENCY", t.cyan, t);
    }
    fn filter(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        self.tile(ui, c, c.r(12.0, 56.0, 309.0, 84.0), 1, "CUTOFF FREQUENCY", t.accent, t);
        self.tile(ui, c, c.r(329.0, 56.0, 309.0, 84.0), 2, "RESONANCE", t.cyan, t);
        panel(ui.painter(), c.r(12.0, 150.0, 626.0, 86.0), "FILTER MODE", t, c.s);
        self.choices(
            ui,
            c,
            c.r(30.0, 187.0, 590.0, 31.0),
            0,
            &["LOW PASS", "BAND PASS", "HIGH PASS"],
            t.accent,
            t,
        );
        self.tile(ui, c, c.r(12.0, 246.0, 309.0, 85.0), 3, "WET / DRY", t.accent, t);
        let r = c.r(333.0, 248.0, 300.0, 79.0);
        grid(ui.painter(), r, t, c.s);
        let v = self.params.values();
        let points = (0..128)
            .map(|i| {
                let hz = 20.0 * 1000.0_f32.powf(i as f32 / 127.0);
                let ratio = hz / v[1];
                let lp = 1.0 / (1.0 + ratio.powi(4)).sqrt();
                let hp = 1.0 / (1.0 + ratio.recip().powi(4)).sqrt();
                let response = match v[0].round() as u8 {
                    1 => lp * hp * 2.0,
                    2 => hp,
                    _ => lp,
                };
                pos2(
                    egui::lerp(r.x_range(), i as f32 / 127.0),
                    r.bottom() - response.clamp(0.0, 1.0) * (r.height() - 5.0 * c.s),
                )
            })
            .collect();
        ui.painter()
            .with_clip_rect(r)
            .add(Shape::line(points, Stroke::new(1.8 * c.s, t.accent)));
    }
    fn delay(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        panel(ui.painter(), c.r(10.0, 55.0, 225.0, 266.0), "DELAY TIME", t, c.s);
        self.knob(ui, c, c.r(47.0, 92.0, 150.0, 151.0), 0, "MANUAL TIME", t.accent, t);
        ui.painter().text(
            c.p(122.0, 279.0),
            Align2::CENTER_CENTER,
            if self.params.controls[5].value.value() > 0.0 {
                "HOST TEMPO SYNC"
            } else {
                "FREE / MILLISECONDS"
            },
            FontId::monospace(10.0 * c.s),
            t.dim,
        );
        panel(
            ui.painter(),
            c.r(245.0, 55.0, 500.0, 266.0),
            "FEEDBACK / STEREO",
            t,
            c.s,
        );
        self.knob(ui, c, c.r(259.0, 91.0, 150.0, 141.0), 1, "FEEDBACK", t.amber, t);
        self.knob(ui, c, c.r(425.0, 91.0, 146.0, 141.0), 3, "FEEDBACK TONE", t.cyan, t);
        self.knob(ui, c, c.r(586.0, 91.0, 144.0, 141.0), 2, "STEREO SPREAD", t.accent, t);
        self.echo_diagram(ui.painter(), c, c.r(260.0, 243.0, 469.0, 62.0), t);
        panel(ui.painter(), c.r(755.0, 55.0, 135.0, 266.0), "OUTPUT", t, c.s);
        self.fader(ui, c, c.r(777.0, 86.0, 90.0, 218.0), 4, "WET / DRY", t.accent, t);
        panel(
            ui.painter(),
            c.r(10.0, 331.0, 880.0, 75.0),
            "TEMPO SYNC / NOTE DIVISION",
            t,
            c.s,
        );
        self.choices(
            ui,
            c,
            c.r(24.0, 366.0, 852.0, 27.0),
            5,
            &[
                "OFF", "1/16", "1/8 T", "1/8", "1/8 DOT", "1/4", "1/4 DOT", "1/2", "1 BAR",
            ],
            t.accent,
            t,
        );
    }
    fn room(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        self.room_diagram(ui.painter(), c, c.r(12.0, 55.0, 270.0, 369.0), t);
        panel(ui.painter(), c.r(292.0, 55.0, 411.0, 369.0), "ROOM / CHARACTER", t, c.s);
        for (index, x, y, label) in [
            (2, 309.0, 94.0, "PRE-DELAY"),
            (0, 505.0, 94.0, "ROOM SIZE"),
            (1, 309.0, 257.0, "DAMPING"),
            (3, 505.0, 257.0, "STEREO WIDTH"),
        ] {
            self.knob(ui, c, c.r(x, y, 172.0, 137.0), index, label, t.purple, t);
        }
        panel(ui.painter(), c.r(713.0, 55.0, 125.0, 369.0), "OUTPUT", t, c.s);
        self.fader(ui, c, c.r(737.0, 87.0, 78.0, 315.0), 4, "WET / DRY", t.purple, t);
    }
    fn comp(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        for (index, x, y, label) in [
            (0, 12.0, 56.0, "THRESHOLD"),
            (1, 247.0, 56.0, "RATIO"),
            (4, 482.0, 56.0, "MAKEUP GAIN"),
            (2, 12.0, 153.0, "ATTACK"),
            (3, 247.0, 153.0, "RELEASE"),
        ] {
            self.tile(ui, c, c.r(x, y, 225.0, 87.0), index, label, t.accent, t);
        }
        let card = c.r(482.0, 153.0, 225.0, 87.0);
        panel(ui.painter(), card, "DETECTOR", t, c.s);
        let select = c.r(494.0, 190.0, 199.0, 29.0);
        let external = self.params.external_key.value();
        let response = ui.interact(select, ui.scope_id().with("external-key"), Sense::click());
        ui.painter().circle_filled(
            select.left_center() + vec2(9.0, 0.0) * c.s,
            4.0 * c.s,
            if external { t.accent } else { t.raised },
        );
        ui.painter().text(
            select.left_center() + vec2(23.0, 0.0) * c.s,
            Align2::LEFT_CENTER,
            if external { "External sidechain" } else { "Main input" },
            FontId::proportional(12.0 * c.s),
            t.text,
        );
        if response.clicked()
            && let Some(context) = &self.context
        {
            set(&context.param_setter(), &self.params.external_key, !external);
        }
        self.transfer(ui.painter(), c, c.r(12.0, 252.0, 346.0, 91.0), t);
        self.scope(ui.painter(), c, c.r(369.0, 252.0, 221.0, 91.0), t.accent, t);
        self.meters(ui.painter(), c, c.r(602.0, 252.0, 105.0, 91.0), t);
    }
    fn limiter(&mut self, ui: &mut Ui, c: Canvas, t: &Theme) {
        self.limiter_history(ui, c, c.r(12.0, 56.0, 768.0, 266.0), t);
        self.meters(ui.painter(), c, c.r(792.0, 56.0, 96.0, 266.0), t);
        panel(ui.painter(), c.r(12.0, 333.0, 345.0, 160.0), "LOUDNESS / LIMIT", t, c.s);
        self.knob(ui, c, c.r(23.0, 367.0, 145.0, 113.0), 0, "CEILING", t.accent, t);
        self.transfer(ui.painter(), c, c.r(184.0, 369.0, 153.0, 104.0), t);
        panel(ui.painter(), c.r(367.0, 333.0, 230.0, 160.0), "ENVELOPE", t, c.s);
        self.knob(ui, c, c.r(407.0, 367.0, 145.0, 113.0), 1, "RELEASE", t.cyan, t);
        let r = c.r(607.0, 333.0, 281.0, 160.0);
        panel(ui.painter(), r, "PEAK LIMITER", t, c.s);
        ui.painter().text(
            c.p(748.0, 398.0),
            Align2::CENTER_CENTER,
            format!("−{:.1} dB", Meters::read(&self.meters.reduction)),
            FontId::monospace(27.0 * c.s),
            t.purple,
        );
        ui.painter().text(
            c.p(748.0, 433.0),
            Align2::CENTER_CENTER,
            "GAIN REDUCTION",
            FontId::monospace(10.0 * c.s),
            t.dim,
        );
        ui.painter().text(
            c.p(748.0, 467.0),
            Align2::CENTER_CENTER,
            "STEREO LINKED  /  ZERO LOOKAHEAD",
            FontId::proportional(9.0 * c.s),
            t.dim,
        );
    }
}
fn shorten(text: &str, count: usize) -> String {
    if text.chars().count() > count {
        format!("{}…", text.chars().take(count).collect::<String>())
    } else {
        text.into()
    }
}
