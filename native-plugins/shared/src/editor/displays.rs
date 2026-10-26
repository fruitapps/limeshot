use super::style::mix;
use super::{Canvas, NativeEditor, Theme, set};
use crate::processor::Meters;
use egui::*;
use limeshot_dsp::effects::{Effect, Eq3};
use nice_plug::prelude::*;
use std::sync::atomic::Ordering;

impl NativeEditor {
    pub(super) fn paint_bypass(&self, ui: &mut Ui, c: Canvas, t: &Theme) {
        let p = ui.painter();
        let rect = c.r(c.w - 84.0, c.h - 57.0, 73.0, 25.0);
        let response = ui.interact(rect, ui.scope_id().with("active"), Sense::click());
        let active = !self.params.bypass.value();
        p.rect_filled(rect, 2, t.surface);
        p.circle_filled(
            rect.left_center() + vec2(11.0, 0.0) * c.s,
            3.5 * c.s,
            if active { t.lime } else { t.dim },
        );
        p.text(
            rect.center() + vec2(5.0, 0.0) * c.s,
            Align2::CENTER_CENTER,
            if active { "ON" } else { "BYPASS" },
            FontId::proportional(10.0 * c.s),
            t.text,
        );
        if response.clicked()
            && let Some(context) = &self.context
        {
            set(&context.param_setter(), &self.params.bypass, active);
        }
    }
    pub(super) fn meters(&self, p: &Painter, c: Canvas, r: Rect, t: &Theme) {
        p.rect_filled(r, 2, t.sunken);
        let columns = if self.id >= 7 { 3 } else { 2 };
        let vals = [
            Meters::read(&self.meters.input),
            Meters::read(&self.meters.output),
            Meters::read(&self.meters.reduction),
        ];
        for i in 0..columns {
            let width = r.width() / columns as f32;
            let x = r.left() + width * (i as f32 + 0.5);
            let color = if i == 2 { t.purple } else { t.accent };
            let norm = if i == 2 {
                vals[2] / 24.0
            } else {
                (limeshot_dsp::gain_to_db(vals[i]) + 60.0) / 60.0
            };
            p.text(
                pos2(x, r.top() + 12.0 * c.s),
                Align2::CENTER_CENTER,
                ["IN", "OUT", "GR"][i],
                FontId::monospace(9.0 * c.s),
                t.dim,
            );
            for n in 0..25 {
                let y = r.bottom() - 32.0 * c.s - n as f32 * (r.height() - 58.0 * c.s) / 25.0;
                let cell = Rect::from_center_size(
                    pos2(x, y),
                    vec2(
                        (width / c.s - 10.0).min(22.0),
                        ((r.height() / c.s - 58.0) / 25.0 * 0.65).max(1.0),
                    ) * c.s,
                );
                p.rect_filled(
                    cell,
                    0,
                    if n as f32 / 25.0 < norm.clamp(0.0, 1.0) {
                        if n > 22 && i != 2 { t.danger } else { color }
                    } else {
                        t.surface
                    },
                );
            }
            let text = if i == 2 {
                format!("{:.1}", vals[i])
            } else if vals[i] > 0.00001 {
                format!("{:.0}", limeshot_dsp::gain_to_db(vals[i]))
            } else {
                "−∞".into()
            };
            p.text(
                pos2(x, r.bottom() - 13.0 * c.s),
                Align2::CENTER_CENTER,
                text,
                FontId::monospace(9.0 * c.s),
                color,
            );
        }
    }
    pub(super) fn scope(&self, p: &Painter, c: Canvas, r: Rect, color: Color32, t: &Theme) {
        grid(p, r, t, c.s);
        let points = (0..128)
            .map(|i| {
                pos2(
                    egui::lerp(r.x_range(), i as f32 / 127.0),
                    r.center().y - Meters::read(&self.meters.scope[i]).clamp(-1.0, 1.0) * r.height() * 0.45,
                )
            })
            .collect();
        p.with_clip_rect(r)
            .add(Shape::line(points, Stroke::new(1.4 * c.s, color)));
    }
    /// Draw the configured synth ADSR with an illustrative 400 ms sustain segment.
    pub(super) fn envelope(&self, p: &Painter, c: Canvas, r: Rect, t: &Theme) {
        grid(p, r, t, c.s);
        let v = self.params.values();
        let times = [v[4], v[5], 400.0, v[7]];
        let total = times.iter().sum::<f32>().max(1.0);
        let mut x = r.left();
        let mut points = vec![r.left_bottom()];
        for (time, level) in times.into_iter().zip([1.0, v[6], v[6], 0.0]) {
            x += time / total * r.width();
            points.push(pos2(x, r.bottom() - level * (r.height() - 8.0 * c.s)));
        }
        let mut fill = points.clone();
        fill.push(r.right_bottom());
        p.add(Shape::convex_polygon(fill, t.accent.gamma_multiply(0.08), Stroke::NONE));
        p.add(Shape::line(points, Stroke::new(2.0 * c.s, t.accent)));
    }
    /// Plot the shared EQ's response at the metered sample rate with editable band nodes.
    /// Dragging changes frequency/gain, the middle node's wheel changes Q, and double-click
    /// resets band frequency/gain. Parameter edits require a host context.
    pub(super) fn eq_graph(&mut self, ui: &mut Ui, c: Canvas, r: Rect, t: &Theme) {
        let p = ui.painter_at(r);
        grid(&p, r, t, c.s);
        let v = self.params.values();
        let sr = Meters::read(&self.meters.sample_rate).max(8000.0);
        let mut eq = Eq3::new(sr);
        eq.set_params(&v[..7]);
        let bounds = r.shrink2(vec2(29.0, 23.0) * c.s);
        for hz in [
            20.0_f32, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
        ] {
            let x = frequency_x(hz, bounds);
            p.line_segment(
                [pos2(x, bounds.top()), pos2(x, bounds.bottom())],
                Stroke::new(c.s, t.surface),
            );
            p.text(
                pos2(x, r.bottom() - 10.0 * c.s),
                Align2::CENTER_CENTER,
                if hz >= 1000.0 {
                    format!("{}k", hz / 1000.0)
                } else {
                    format!("{hz:.0}")
                },
                FontId::monospace(9.0 * c.s),
                t.dim,
            );
        }
        for db in [-18.0_f32, -12.0, -6.0, 0.0, 6.0, 12.0, 18.0] {
            let y = bounds.center().y - db / 36.0 * bounds.height();
            p.line_segment(
                [pos2(bounds.left(), y), pos2(bounds.right(), y)],
                Stroke::new(c.s, if db == 0.0 { t.edge } else { t.surface }),
            );
            p.text(
                pos2(r.left() + 3.0 * c.s, y),
                Align2::LEFT_CENTER,
                format!("{db:+.0}"),
                FontId::monospace(8.0 * c.s),
                t.dim,
            );
        }
        let points = (0..240)
            .map(|i| {
                let hz = 20.0_f32 * 1000.0_f32.powf(i as f32 / 239.0);
                pos2(
                    frequency_x(hz, bounds),
                    bounds.center().y - eq.response_db(hz) / 36.0 * bounds.height(),
                )
            })
            .collect();
        p.with_clip_rect(bounds)
            .add(Shape::line(points, Stroke::new(2.0 * c.s, t.text)));
        for (n, (freq, gain, color)) in [(1, 0, t.purple), (3, 2, t.accent), (6, 5, t.cyan)]
            .into_iter()
            .enumerate()
        {
            let center = pos2(
                frequency_x(v[freq], bounds),
                bounds.center().y - v[gain] / 36.0 * bounds.height(),
            );
            let hit = Rect::from_center_size(center, vec2(27.0, 27.0) * c.s);
            let response = ui.interact(hit, ui.scope_id().with(("eq-node", n)), Sense::click_and_drag());
            p.circle_filled(center, 12.0 * c.s, t.header);
            p.circle_stroke(center, 11.0 * c.s, Stroke::new(2.0 * c.s, color));
            p.text(
                center,
                Align2::CENTER_CENTER,
                (n + 1).to_string(),
                FontId::monospace(12.0 * c.s),
                color,
            );
            response
                .clone()
                .on_hover_text("Drag: frequency & gain. Wheel on middle band: Q. Double-click: reset band.");
            if let Some(context) = &self.context {
                let setter = context.param_setter();
                let f = &self.params.controls[freq].value;
                let g = &self.params.controls[gain].value;
                if response.drag_started() {
                    setter.begin_set_parameter(f);
                    setter.begin_set_parameter(g);
                }
                if response.dragged()
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    setter.set_parameter(f, clamp(f, x_frequency(pos.x, bounds)));
                    setter.set_parameter(g, clamp(g, (bounds.center().y - pos.y) / bounds.height() * 36.0));
                }
                if response.drag_stopped() {
                    setter.end_set_parameter(f);
                    setter.end_set_parameter(g);
                }
                if response.double_clicked() {
                    set(&setter, f, f.default_plain_value());
                    set(&setter, g, 0.0);
                }
                let wheel = if response.hovered() && n == 1 {
                    ui.input(|i| i.smooth_scroll_delta.y)
                } else {
                    0.0
                };
                if wheel != 0.0 {
                    let q = &self.params.controls[4].value;
                    set(
                        &setter,
                        q,
                        q.preview_plain((q.unmodulated_normalized_value() + wheel * 0.001).clamp(0.0, 1.0)),
                    );
                }
            }
        }
        p.text(
            r.min + vec2(38.0, 12.0) * c.s,
            Align2::LEFT_CENTER,
            "LOW SHELF                    PEAK                    HIGH SHELF",
            FontId::monospace(9.0 * c.s),
            t.dim,
        );
    }
    pub(super) fn transfer(&self, p: &Painter, c: Canvas, r: Rect, t: &Theme) {
        grid(p, r, t, c.s);
        let v = self.params.values();
        let points = (0..128)
            .map(|i| {
                let input = -60.0 + i as f32 / 127.0 * 60.0;
                let out = if self.id == 8 {
                    input.min(v[0])
                } else if input > v[0] {
                    v[0] + (input - v[0]) / v[1] + v[4]
                } else {
                    input + v[4]
                };
                pos2(
                    egui::lerp(r.x_range(), i as f32 / 127.0),
                    r.bottom() - ((out + 60.0) / 60.0).clamp(0.0, 1.0) * r.height(),
                )
            })
            .collect();
        p.add(Shape::line(points, Stroke::new(1.8 * c.s, t.accent)));
    }
    /// Draw input/output levels and reduction history with the configured ceiling.
    /// Append at most one meter snapshot per call, spaced more than 1/60 second apart.
    pub(super) fn limiter_history(&mut self, ui: &mut Ui, c: Canvas, r: Rect, t: &Theme) {
        let now = ui.input(|i| i.time);
        if now - self.history_time > 1.0 / 60.0 {
            self.history[self.history_cursor] = [
                Meters::read(&self.meters.input),
                Meters::read(&self.meters.output),
                Meters::read(&self.meters.reduction),
            ];
            self.history_cursor = (self.history_cursor + 1) % self.history.len();
            self.history_time = now;
        }
        let p = ui.painter_at(r);
        grid(&p, r, t, c.s);
        let area = r.shrink2(vec2(15.0, 22.0) * c.s);
        for (index, color) in [(0, t.dim), (1, t.accent), (2, t.purple)] {
            let points = (0..256)
                .map(|i| {
                    let v = self.history[(i + self.history_cursor) % 256][index];
                    let value = if index == 2 {
                        1.0 - (v / 30.0).clamp(0.0, 1.0)
                    } else {
                        ((limeshot_dsp::gain_to_db(v) + 60.0) / 60.0).clamp(0.0, 1.0)
                    };
                    pos2(
                        egui::lerp(area.x_range(), i as f32 / 255.0),
                        area.bottom() - value * area.height(),
                    )
                })
                .collect();
            p.add(Shape::line(points, Stroke::new(1.3 * c.s, color)));
        }
        let ceiling = self.params.controls[0].value.value();
        let y = area.bottom() - (ceiling + 60.0) / 60.0 * area.height();
        p.line_segment(
            [pos2(area.left(), y), pos2(area.right(), y)],
            Stroke::new(c.s, t.accent.gamma_multiply(0.6)),
        );
        p.text(
            r.min + vec2(15.0, 12.0) * c.s,
            Align2::LEFT_CENTER,
            "INPUT / OUTPUT LEVEL    ·    GAIN REDUCTION",
            FontId::monospace(9.0 * c.s),
            t.dim,
        );
        p.text(
            r.right_top() + vec2(-15.0, 12.0) * c.s,
            Align2::RIGHT_CENTER,
            "LIVE HISTORY",
            FontId::monospace(9.0 * c.s),
            t.purple,
        );
    }
    pub(super) fn room_diagram(&self, p: &Painter, c: Canvas, r: Rect, t: &Theme) {
        p.rect_filled(r, 2, mix(t.sunken, t.purple, 0.08));
        let v = self.params.values();
        let size = 0.65 + v[0] * 0.3;
        let width = (0.6 + v[3] * 0.35) * r.width() * 0.42;
        let center = r.center() + vec2(0.0, 8.0) * c.s;
        let a = center + vec2(-width, -r.height() * 0.27) * size;
        let b = center + vec2(width, -r.height() * 0.27) * size;
        let d = center + vec2(-width, r.height() * 0.29) * size;
        let e = center + vec2(width, r.height() * 0.29) * size;
        let offset = vec2(30.0, -28.0) * c.s;
        for (start, end) in [
            (a, b),
            (b, e),
            (e, d),
            (d, a),
            (a + offset, b + offset),
            (b + offset, e + offset),
            (e + offset, d + offset),
            (d + offset, a + offset),
            (a, a + offset),
            (b, b + offset),
            (d, d + offset),
            (e, e + offset),
        ] {
            p.line_segment([start, end], Stroke::new(1.4 * c.s, t.purple));
        }
        for i in 1..7 {
            let f = i as f32 / 7.0;
            p.line_segment(
                [a.lerp(b, f), d.lerp(e, f)],
                Stroke::new(c.s, t.purple.gamma_multiply(0.2)),
            );
        }
        let origin = center + vec2(0.0, 25.0) * c.s;
        for i in 0..4 {
            let radius = (20.0 + i as f32 * 17.0) * c.s;
            p.circle_stroke(
                origin,
                radius,
                Stroke::new(c.s, t.purple.gamma_multiply(0.15 + 0.1 * (1.0 - v[1]))),
            );
        }
        p.text(
            r.center_top() + vec2(0.0, 19.0) * c.s,
            Align2::CENTER_CENTER,
            "ACOUSTIC SPACE",
            FontId::monospace(10.0 * c.s),
            t.purple,
        );
        p.text(
            r.center_bottom() - vec2(0.0, 16.0) * c.s,
            Align2::CENTER_CENTER,
            "ROOM / EARLY REFLECTIONS",
            FontId::proportional(10.0 * c.s),
            t.dim,
        );
    }
    pub(super) fn echo_diagram(&self, p: &Painter, c: Canvas, r: Rect, t: &Theme) {
        grid(p, r, t, c.s);
        let v = self.params.values();
        for i in 0..8 {
            let x = r.left() + r.width() * (i as f32 + 0.5) / 8.0;
            let gain = if i == 0 { 1.0 - v[4] } else { v[4] * v[1].powi(i - 1) };
            let height = gain.clamp(0.0, 1.0) * (r.height() - 28.0 * c.s);
            let bar = Rect::from_min_max(
                pos2(x - 2.0 * c.s, r.bottom() - height - 11.0 * c.s),
                pos2(x + 2.0 * c.s, r.bottom() - 11.0 * c.s),
            );
            p.rect_filled(bar, 1, if i % 2 == 0 { t.amber } else { t.cyan });
        }
        p.text(
            r.left_top() + vec2(8.0, 10.0) * c.s,
            Align2::LEFT_CENTER,
            "ECHO PATTERN",
            FontId::monospace(9.0 * c.s),
            t.dim,
        );
    }
    /// Refresh and draw the cached sample peaks, handle trim gestures, and request root-note
    /// audition while held away from trim handles. Loads the first dropped file through
    /// the host context; trim gestures keep start at or before end.
    pub(super) fn sample_waveform(&mut self, ui: &mut Ui, c: Canvas, r: Rect, t: &Theme) {
        ui.ctx().data_mut(|d| d.insert_temp(Id::unique("sampler-waveform"), r));
        let sample = self.params.sample.audio();
        let changed = match (&self.sample, &sample) {
            (Some(a), Some(b)) => !std::sync::Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };
        if changed {
            self.sample_name = self.params.sample.name();
            self.waveform.fill(0.0);
            if let Some(audio) = &sample {
                for (i, peak) in self.waveform.iter_mut().enumerate() {
                    let begin = i * audio.frames() / 256;
                    let end = ((i + 1) * audio.frames() / 256).max(begin + 1);
                    if let Some((lo, hi)) = audio.overview_range(begin, end) {
                        *peak = lo.abs().max(hi.abs()).min(1.0);
                        continue;
                    }
                    *peak = (begin..end)
                        .map(|f| {
                            let (l, r) = audio.frame(f);
                            l.abs().max(r.abs())
                        })
                        .fold(0.0_f32, f32::max)
                        .min(1.0);
                }
            }
            self.sample = sample;
        }
        let p = ui.painter_at(r);
        grid(&p, r, t, c.s);
        let v = self.params.values();
        let lo = egui::lerp(r.x_range(), v[2]);
        let hi = egui::lerp(r.x_range(), v[3]);
        p.rect_filled(
            Rect::from_min_max(pos2(lo, r.top()), pos2(hi.max(lo), r.bottom())),
            0,
            t.accent.gamma_multiply(0.08),
        );
        for (i, peak) in self.waveform.iter().enumerate() {
            let x = egui::lerp(r.x_range(), i as f32 / 255.0);
            let h = peak * r.height() * 0.44;
            let color = if x >= lo && x <= hi {
                t.accent
            } else {
                t.dim.gamma_multiply(0.4)
            };
            p.line_segment(
                [pos2(x, r.center().y - h), pos2(x, r.center().y + h)],
                Stroke::new(1.6 * c.s, color),
            );
        }
        for (index, x, color, label) in [(2, lo, t.accent, "START"), (3, hi, t.danger, "END")] {
            let handle = Rect::from_center_size(pos2(x, r.center().y), vec2(14.0, r.height() / c.s) * c.s);
            let response = ui.interact(handle, ui.scope_id().with(("trim", index)), Sense::click_and_drag());
            p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(2.0 * c.s, color));
            p.text(
                pos2(
                    if index == 2 { x + 5.0 * c.s } else { x - 5.0 * c.s },
                    r.top() + 12.0 * c.s,
                ),
                if index == 2 {
                    Align2::LEFT_CENTER
                } else {
                    Align2::RIGHT_CENTER
                },
                label,
                FontId::monospace(9.0 * c.s),
                color,
            );
            if let Some(context) = &self.context {
                let setter = context.param_setter();
                let param = &self.params.controls[index].value;
                if response.drag_started() {
                    setter.begin_set_parameter(param);
                }
                if response.dragged()
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    let f = ((pos.x - r.left()) / r.width()).clamp(0.0, 1.0);
                    let f = if index == 2 { f.min(v[3]) } else { f.max(v[2]) };
                    setter.set_parameter(param, f);
                }
                if response.drag_stopped() {
                    setter.end_set_parameter(param);
                }
            }
        }
        let over_handle = ui
            .input(|i| i.pointer.interact_pos())
            .is_some_and(|pos| (pos.x - lo).abs() < 10.0 * c.s || (pos.x - hi).abs() < 10.0 * c.s);
        let preview = ui.interact(
            r.shrink2(vec2(9.0, 22.0) * c.s),
            ui.scope_id().with("sample-preview"),
            if over_handle {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            },
        );
        preview
            .clone()
            .on_hover_text("Hold to audition · Drag START / END handles to trim · Drop an audio file to replace");
        if preview.is_pointer_button_down_on() {
            self.meters.preview_note.store(
                self.params.controls[7].value.value().round() as u32 + 1,
                Ordering::Relaxed,
            );
        }
        if self.sample.is_none() {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                "DROP AN AUDIO FILE HERE",
                FontId::monospace(13.0 * c.s),
                t.dim,
            );
        }
        let paths: Vec<_> = ui
            .ctx()
            .input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        if let Some(path) = paths.first() {
            self.load_sample(path);
        }
    }
    /// Draw the sampler keyboard and request audition of the held key, prioritizing black keys.
    /// A right-click changes the root key when a host context is attached.
    pub(super) fn keyboard(&mut self, ui: &mut Ui, c: Canvas, r: Rect, t: &Theme) {
        ui.ctx().data_mut(|d| d.insert_temp(Id::unique("sampler-keyboard"), r));
        let root = self.params.controls[7].value.value().round() as u8;
        let start = ((root / 12).saturating_sub(1) * 12).min(96);
        let end = (start + 36).min(128);
        let white_count = (start..end).filter(|k| !is_black(*k)).count();
        let width = r.width() / white_count as f32;
        let mut white = 0;
        let mut keys = Vec::new();
        for key in start..end {
            if is_black(key) {
                continue;
            }
            let rect = Rect::from_min_size(
                pos2(r.left() + white as f32 * width, r.top()),
                vec2(width - 1.0 * c.s, r.height()),
            );
            keys.push((key, rect));
            white += 1;
        }
        for (key, rect) in &keys {
            let color = if *key == root {
                t.accent
            } else {
                Color32::from_rgb(184, 197, 198)
            };
            ui.painter().rect_filled(*rect, 1, color);
            if key % 12 == 0 {
                ui.painter().text(
                    rect.center_bottom() - vec2(0.0, 8.0) * c.s,
                    Align2::CENTER_CENTER,
                    format!("C{}", *key as i32 / 12 - 1),
                    FontId::monospace(8.0 * c.s),
                    t.header,
                );
            }
        }
        let mut whites = 0;
        for key in start..end {
            if !is_black(key) {
                whites += 1;
                continue;
            }
            let rect = Rect::from_min_size(
                pos2(r.left() + whites as f32 * width - width * 0.31, r.top()),
                vec2(width * 0.62, r.height() * 0.59),
            );
            ui.painter()
                .rect_filled(rect, 1, if key == root { t.accent } else { t.header });
            keys.push((key, rect));
        }
        let response = ui.interact(r, ui.scope_id().with("sampler-keyboard"), Sense::click_and_drag());
        // Black keys take hit priority over the white keys beneath them.
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos())
            && let Some((key, _)) = keys.iter().rev().find(|(_, rect)| rect.contains(pos))
        {
            if response.is_pointer_button_down_on() {
                self.meters.preview_note.store(*key as u32 + 1, Ordering::Relaxed);
            }
            if response.secondary_clicked()
                && let Some(context) = &self.context
            {
                set(&context.param_setter(), &self.params.controls[7].value, *key as f32);
            }
        }
    }
}
fn is_black(key: u8) -> bool {
    matches!(key % 12, 1 | 3 | 6 | 8 | 10)
}
fn clamp(p: &FloatParam, v: f32) -> f32 {
    p.preview_plain(p.preview_normalized(v))
}
/// Map Hz to the graph's logarithmic 20 Hz–20 kHz x-axis without clamping.
fn frequency_x(hz: f32, r: Rect) -> f32 {
    r.left() + (hz / 20.0).log10() / 3.0 * r.width()
}
/// Map an x-coordinate to Hz on the logarithmic graph, clamping to 20 Hz–20 kHz.
fn x_frequency(x: f32, r: Rect) -> f32 {
    20.0 * 1000.0_f32.powf(((x - r.left()) / r.width()).clamp(0.0, 1.0))
}
pub(super) fn grid(p: &Painter, r: Rect, t: &Theme, s: f32) {
    p.rect_filled(r, 2, t.sunken);
    p.rect_stroke(r, 2, Stroke::new(s, t.border), StrokeKind::Inside);
    for i in 1..8 {
        let x = egui::lerp(r.x_range(), i as f32 / 8.0);
        p.line_segment(
            [pos2(x, r.top()), pos2(x, r.bottom())],
            Stroke::new(s, t.surface.gamma_multiply(0.45)),
        );
    }
    for i in 1..4 {
        let y = egui::lerp(r.y_range(), i as f32 / 4.0);
        p.line_segment(
            [pos2(r.left(), y), pos2(r.right(), y)],
            Stroke::new(s, t.surface.gamma_multiply(0.45)),
        );
    }
}
