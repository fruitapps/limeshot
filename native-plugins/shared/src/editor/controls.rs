use super::style::mix;
use super::{Canvas, NativeEditor, Theme, set};
use egui::*;
use nice_plug::prelude::*;
use std::f32::consts::PI;

impl NativeEditor {
    /// Handle host parameter gestures in an egui rectangle, returning its interaction response.
    /// `index` selects a control; `absolute` supplies a normalized click/drag value, otherwise
    /// vertical dragging adjusts relatively. Supports fine dragging, reset, wheel, and text entry.
    /// Parameter writes require a host context; synced delay disables manual-time interaction.
    pub(super) fn control_response(&mut self, ui: &mut Ui, r: Rect, index: usize, absolute: Option<f32>) -> Response {
        let p = &self.params.controls[index].value;
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::unique(("native-control", p.name())), r));
        let disabled = self.id == 5 && index == 0 && self.params.controls[5].value.value() > 0.0;
        let response = ui.interact(
            r,
            ui.scope_id().with(("control", index)),
            if disabled {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            },
        );
        if disabled {
            return response.on_hover_text("Manual time is inactive while tempo sync is selected");
        }
        response.clone().on_hover_text(format!("{}: {}", p.name(), readout(p)));
        if let Some(context) = self.context.clone() {
            let setter = context.param_setter();
            if response.drag_started() {
                setter.begin_set_parameter(p);
                ui.ctx()
                    .data_mut(|d| d.insert_temp(response.id.with("drag-value"), p.unmodulated_normalized_value()));
            }
            if response.dragged() {
                let normalized = absolute.unwrap_or_else(|| {
                    let step = if ui.input(|i| i.modifiers.shift) { 0.0005 } else { 0.005 };
                    let id = response.id.with("drag-value");
                    let previous = ui
                        .ctx()
                        .data(|d| d.get_temp::<f32>(id))
                        .unwrap_or_else(|| p.unmodulated_normalized_value());
                    let next = (previous - ui.input(|i| i.pointer.delta().y) * step).clamp(0.0, 1.0);
                    ui.ctx().data_mut(|d| d.insert_temp(id, next));
                    next
                });
                setter.set_parameter(p, p.preview_plain(normalized.clamp(0.0, 1.0)));
            }
            if response.drag_stopped() {
                setter.end_set_parameter(p);
            }
            if response.clicked()
                && let Some(v) = absolute
            {
                set(&setter, p, p.preview_plain(v.clamp(0.0, 1.0)));
            }
            if response.double_clicked() {
                set(&setter, p, p.default_plain_value());
            }
            let wheel = if response.hovered() {
                ui.input(|i| {
                    i.raw
                        .events
                        .iter()
                        .filter_map(|event| match event {
                            Event::MouseWheel { delta, .. } => Some(delta.y),
                            _ => None,
                        })
                        .sum()
                })
            } else {
                0.0
            };
            if wheel != 0.0 {
                set(
                    &setter,
                    p,
                    if p.step_count().is_some() {
                        if wheel > 0.0 {
                            p.next_step(p.value(), false)
                        } else {
                            p.previous_step(p.value(), false)
                        }
                    } else {
                        p.preview_plain((p.unmodulated_normalized_value() + wheel * 0.001).clamp(0.0, 1.0))
                    },
                );
            }
        }
        response
    }
    fn value_field(&mut self, ui: &mut Ui, rect: Rect, index: usize, color: Color32, font: FontId) {
        let param = &self.params.controls[index].value;
        let id = ui.scope_id().with(("value-entry", index));
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::unique(("native-value", param.name())), rect));
        if self.id == 5 && index == 0 && self.params.controls[5].value.value() > 0.0 {
            if self.editing == Some(index) {
                self.editing = None;
                ui.memory_mut(|m| m.surrender_focus(id.with("text")));
            }
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, readout(param), font, color);
            ui.interact(rect, id, Sense::hover())
                .on_hover_text("Manual time is inactive while tempo sync is selected");
            return;
        }
        let mut focus = false;
        if self.editing != Some(index) {
            let response = ui
                .interact(rect, id, Sense::click())
                .on_hover_text("Click to type a value; Enter applies, Escape cancels");
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                readout(param),
                font.clone(),
                color,
            );
            if !response.clicked() {
                return;
            }
            self.entry = param.normalized_value_to_string(param.unmodulated_normalized_value(), false);
            self.editing = Some(index);
            focus = true;
        }
        let response = ui.put(
            rect,
            TextEdit::singleline(&mut self.entry)
                .id(id.with("text"))
                .font(font)
                .align(Align2::CENTER_CENTER)
                .margin(vec2(1.0, 0.0)),
        );
        if focus {
            response.request_focus();
            let mut state = TextEdit::load_state(ui.ctx(), response.id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(text::CCursorRange::select_all_str(&self.entry)));
            state.store(ui.ctx(), response.id);
        }
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            self.editing = None;
            response.surrender_focus();
        } else if ui.input(|i| i.key_pressed(Key::Enter)) || (!focus && response.lost_focus()) {
            if let Some(value) = param.string_to_normalized_value(&self.entry) {
                if let Some(context) = &self.context {
                    set(&context.param_setter(), param, param.preview_plain(value));
                }
                self.status.clear();
                self.editing = None;
                response.surrender_focus();
            } else {
                self.status = format!("Invalid value for {}", param.name());
                response.request_focus();
            }
        }
    }
    pub(super) fn knob(
        &mut self,
        ui: &mut Ui,
        c: Canvas,
        rect: Rect,
        index: usize,
        label: &str,
        color: Color32,
        t: &Theme,
    ) {
        self.control_response(
            ui,
            Rect::from_min_max(rect.min, rect.max - vec2(0.0, 34.0 * c.s)),
            index,
            None,
        );
        let param = &self.params.controls[index].value;
        let radius = (rect.width() * 0.36)
            .min((rect.height() - 38.0 * c.s) * 0.5)
            .max(10.0 * c.s);
        let center = pos2(rect.center().x, rect.top() + radius + 5.0 * c.s);
        let color = if self.id == 5 && index == 0 && self.params.controls[5].value.value() > 0.0 {
            t.dim
        } else {
            color
        };
        dial(
            ui.painter(),
            center,
            radius,
            param.unmodulated_normalized_value(),
            color,
            t,
            c.s,
        );
        self.value_field(
            ui,
            Rect::from_center_size(
                pos2(rect.center().x, rect.bottom() - 24.0 * c.s),
                vec2(rect.width(), 20.0 * c.s),
            ),
            index,
            color,
            FontId::monospace(11.0 * c.s),
        );
        ui.painter().text(
            pos2(rect.center().x, rect.bottom() - 8.0 * c.s),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.0 * c.s),
            t.text,
        );
    }
    pub(super) fn tile(
        &mut self,
        ui: &mut Ui,
        c: Canvas,
        r: Rect,
        index: usize,
        label: &str,
        color: Color32,
        t: &Theme,
    ) {
        let response = self.control_response(
            ui,
            Rect::from_min_size(r.min, vec2(68.0 * c.s, r.height())),
            index,
            None,
        );
        let p = ui.painter();
        p.rect_filled(
            r,
            3,
            if response.hovered() {
                mix(t.surface, t.raised, 0.5)
            } else {
                t.surface
            },
        );
        p.rect_stroke(r, 3, Stroke::new(c.s, t.border), StrokeKind::Inside);
        let param = &self.params.controls[index].value;
        let center = r.min + vec2(34.0, r.height() / c.s * 0.5) * c.s;
        dial(
            p,
            center,
            25.0 * c.s,
            param.unmodulated_normalized_value(),
            color,
            t,
            c.s,
        );
        p.text(
            r.min + vec2(80.0, 22.0) * c.s,
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.0 * c.s),
            t.dim,
        );
        self.value_field(
            ui,
            Rect::from_min_size(r.min + vec2(72.0, 37.0) * c.s, vec2(r.width() - 76.0 * c.s, 24.0 * c.s)),
            index,
            color,
            FontId::monospace(13.0 * c.s),
        );
    }
    pub(super) fn fader(
        &mut self,
        ui: &mut Ui,
        c: Canvas,
        r: Rect,
        index: usize,
        label: &str,
        color: Color32,
        t: &Theme,
    ) {
        let rail = Rect::from_min_max(
            r.min + vec2(r.width() * 0.5 - 3.0 * c.s, 23.0 * c.s),
            r.max - vec2(r.width() * 0.5 - 3.0 * c.s, 45.0 * c.s),
        );
        let absolute = ui
            .input(|i| i.pointer.interact_pos())
            .map(|p| (rail.bottom() - p.y) / rail.height());
        self.control_response(ui, rail.expand(12.0 * c.s), index, absolute);
        let p = ui.painter();
        let param = &self.params.controls[index].value;
        let value = param.unmodulated_normalized_value();
        let y = egui::lerp(rail.y_range(), 1.0 - value);
        p.text(
            pos2(r.center().x, r.top() + 9.0 * c.s),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.0 * c.s),
            t.text,
        );
        p.rect_filled(rail, 2, t.sunken);
        p.line_segment(
            [rail.center_bottom(), pos2(rail.center().x, y)],
            Stroke::new(2.0 * c.s, color),
        );
        for n in 0..=10 {
            let y = egui::lerp(rail.y_range(), n as f32 / 10.0);
            p.line_segment(
                [pos2(rail.left() - 11.0 * c.s, y), pos2(rail.left() - 5.0 * c.s, y)],
                Stroke::new(c.s, t.edge),
            );
        }
        let thumb = Rect::from_center_size(pos2(rail.center().x, y), vec2(26.0, 13.0) * c.s);
        p.rect_filled(thumb, 2, t.raised);
        p.line_segment(
            [
                thumb.left_center() + vec2(3.0, 0.0) * c.s,
                thumb.right_center() - vec2(3.0, 0.0) * c.s,
            ],
            Stroke::new(2.0 * c.s, color),
        );
        self.value_field(
            ui,
            Rect::from_center_size(pos2(r.center().x, r.bottom() - 24.0 * c.s), vec2(r.width(), 20.0 * c.s)),
            index,
            color,
            FontId::monospace(10.0 * c.s),
        );
    }
    pub(super) fn slider(&mut self, ui: &mut Ui, c: Canvas, r: Rect, index: usize, label: &str, t: &Theme) {
        let rail = Rect::from_min_max(r.min + vec2(102.0, 16.0) * c.s, r.max - vec2(88.0, 13.0) * c.s);
        let absolute = ui
            .input(|i| i.pointer.interact_pos())
            .map(|p| (p.x - rail.left()) / rail.width());
        self.control_response(ui, rail.expand(8.0 * c.s), index, absolute);
        let param = &self.params.controls[index].value;
        let p = ui.painter();
        p.text(
            r.min + vec2(10.0, r.height() / c.s * 0.5) * c.s,
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.0 * c.s),
            t.dim,
        );
        p.rect_filled(rail, 2, t.sunken);
        let x = egui::lerp(rail.x_range(), param.unmodulated_normalized_value());
        p.rect_filled(
            Rect::from_min_max(rail.min, pos2(x, rail.bottom())),
            2,
            t.accent.gamma_multiply(0.5),
        );
        p.rect_filled(
            Rect::from_center_size(pos2(x, rail.center().y), vec2(6.0, 15.0) * c.s),
            1,
            t.accent,
        );
        self.value_field(
            ui,
            Rect::from_min_max(pos2(r.right() - 80.0 * c.s, r.top()), r.max),
            index,
            t.text,
            FontId::monospace(10.0 * c.s),
        );
    }
    pub(super) fn choices(
        &mut self,
        ui: &mut Ui,
        c: Canvas,
        r: Rect,
        index: usize,
        labels: &[&str],
        color: Color32,
        t: &Theme,
    ) {
        let param = &self.params.controls[index].value;
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::unique(("native-control", param.name())), r));
        let selected = param.value().round() as usize;
        let count = labels.len();
        for (i, label) in labels.iter().enumerate() {
            let cell = Rect::from_min_max(
                pos2(r.left() + r.width() * i as f32 / count as f32, r.top()),
                pos2(
                    r.left() + r.width() * (i + 1) as f32 / count as f32 - 2.0 * c.s,
                    r.bottom(),
                ),
            );
            let response = ui.interact(cell, ui.scope_id().with(("choice", index, i)), Sense::click());
            let p = ui.painter();
            let on = i == selected;
            p.rect_filled(cell, 2, if on { mix(t.surface, color, 0.22) } else { t.header });
            p.rect_stroke(
                cell,
                2,
                Stroke::new(c.s, if on { color } else { t.border }),
                StrokeKind::Inside,
            );
            p.text(
                cell.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(10.0 * c.s),
                if on { color } else { t.dim },
            );
            if response.clicked()
                && let Some(context) = &self.context
            {
                set(&context.param_setter(), param, i as f32);
            }
        }
    }
    pub(super) fn lamp(&mut self, ui: &mut Ui, c: Canvas, r: Rect, index: usize, label: &str, t: &Theme) {
        let response = self.control_response(ui, r, index, None);
        let param = &self.params.controls[index].value;
        let on = param.value() >= 0.5;
        let p = ui.painter();
        let center = r.left_center() + vec2(12.0, 0.0) * c.s;
        p.circle_filled(center, 7.0 * c.s, t.border);
        p.circle_filled(center, 4.5 * c.s, if on { t.accent } else { t.raised });
        p.text(
            center + vec2(15.0, 0.0) * c.s,
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.0 * c.s),
            t.text,
        );
        if response.clicked()
            && !response.double_clicked()
            && let Some(context) = &self.context
        {
            set(&context.param_setter(), param, if on { 0.0 } else { 1.0 });
        }
    }
    pub(super) fn button(&self, ui: &mut Ui, c: Canvas, r: Rect, label: &str, t: &Theme) -> bool {
        let response = ui.interact(r, ui.scope_id().with(("button", label)), Sense::click());
        ui.painter()
            .rect_filled(r, 2, if response.hovered() { t.raised } else { t.header });
        ui.painter()
            .rect_stroke(r, 2, Stroke::new(c.s, t.border), StrokeKind::Inside);
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0 * c.s),
            t.text,
        );
        response.clicked()
    }
}

pub(super) fn readout(p: &FloatParam) -> String {
    p.normalized_value_to_string(p.unmodulated_normalized_value(), true)
        .replace(" k Hz", " kHz")
}

pub(super) fn dial(p: &Painter, center: Pos2, r: f32, v: f32, color: Color32, t: &Theme, s: f32) {
    let arc = |radius: f32, start: f32, end: f32| -> Vec<Pos2> {
        (0..=48)
            .map(|i| {
                let a = egui::lerp(start..=end, i as f32 / 48.0);
                center + vec2(a.cos(), a.sin()) * radius
            })
            .collect()
    };
    p.circle_filled(center, r - 4.0 * s, t.raised);
    p.circle_stroke(center, r - 4.0 * s, Stroke::new(s, t.border));
    p.add(Shape::line(
        arc(r, 0.75 * PI, 2.25 * PI),
        Stroke::new(2.0 * s, t.border),
    ));
    p.add(Shape::line(
        arc(r, 0.75 * PI, 0.75 * PI + v * 1.5 * PI),
        Stroke::new(2.5 * s, color),
    ));
    let a = 0.75 * PI + v * 1.5 * PI;
    p.line_segment(
        [
            center + vec2(a.cos(), a.sin()) * (r * 0.40),
            center + vec2(a.cos(), a.sin()) * (r - 9.0 * s),
        ],
        Stroke::new(2.0 * s, color),
    );
}
