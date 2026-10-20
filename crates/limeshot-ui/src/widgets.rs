//! Custom controls: knobs, faders, meters, icon buttons.
//!
//! Value widgets edit a local copy and report `changed()`; callers turn changes into commands.
//! Shared conventions (documented in `docs/architecture/ui.md`): drag vertically to change,
//! Shift for fine control, mouse wheel steps, double-click resets to the default.

use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, mono};
use egui::{Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use std::f32::consts::PI;

/// An icon button. `active` draws it in the accent color (a toggled state).
pub fn icon_button(ui: &mut Ui, icon: Icon, active: bool, size: f32, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let fill = if active {
        t.accent_dim
    } else if resp.hovered() {
        t.raised_hover
    } else {
        t.raised
    };
    ui.painter().rect_filled(r, CornerRadius::same(4), fill);
    let fg = if active { t.accent } else { t.text };
    icons::paint(ui.painter(), r.shrink(size * 0.18), icon, fg);
    resp.on_hover_text(tip)
}

/// A disabled-looking icon button for features that are not implemented yet; it explains why
/// on hover and does nothing when clicked.
pub fn unavailable_button(ui: &mut Ui, icon: Icon, size: f32, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(4), t.sunken);
    icons::paint(ui.painter(), r.shrink(size * 0.18), icon, t.text_dim.gamma_multiply(0.5));
    resp.on_hover_text(format!("{tip} (not implemented yet)"))
}

/// A small toggle with a text label, e.g. M/S buttons.
pub fn toggle_label(ui: &mut Ui, on: bool, text: &str, on_color: Color32, size: egui::Vec2, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if on {
        on_color
    } else if resp.hovered() {
        t.raised_hover
    } else {
        t.raised
    };
    ui.painter().rect_filled(r, CornerRadius::same(3), fill);
    let fg = if on { t.text_on_accent } else { t.text_dim };
    ui.painter().text(r.center(), Align2::CENTER_CENTER, text, mono(10.5), fg);
    resp.on_hover_text(tip)
}

/// Handle drag/wheel/double-click for a value control. Returns true if `v` changed.
fn drag_value(ui: &Ui, resp: &Response, v: &mut f32, min: f32, max: f32, default: f32, pixels_full: f32) -> bool {
    let old = *v;
    if resp.double_clicked() {
        *v = default;
    } else if resp.dragged() {
        let fine = ui.input(|i| i.modifiers.shift);
        let scale = if fine { 0.1 } else { 1.0 };
        let dy = -resp.drag_delta().y;
        *v = (*v + dy / pixels_full * (max - min) * scale).clamp(min, max);
    } else if resp.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel != 0.0 {
            let step = (max - min) / 50.0 * wheel.signum();
            *v = (*v + step).clamp(min, max);
        }
    }
    *v != old
}

/// A rotary knob. `bipolar` draws the arc from the centre (pan).
pub fn knob(ui: &mut Ui, v: &mut f32, min: f32, max: f32, default: f32, size: f32, bipolar: bool, tip: impl Fn(f32) -> String) -> Response {
    let t = Theme::current();
    let (r, mut resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    if drag_value(ui, &resp, v, min, max, default, 120.0) {
        resp.mark_changed();
    }
    let p = ui.painter();
    let c = r.center();
    let rad = size * 0.42;
    p.circle_filled(c, rad, if resp.hovered() || resp.dragged() { t.raised_hover } else { t.raised });
    let frac = if max > min { (*v - min) / (max - min) } else { 0.0 };
    let a0 = PI * 0.75;
    let sweep = PI * 1.5;
    let arc = |from: f32, to: f32, color: Color32, w: f32| {
        let n = 20;
        let pts: Vec<_> = (0..=n)
            .map(|i| {
                let a = from + (to - from) * i as f32 / n as f32;
                c + vec2(a.cos(), a.sin()) * (rad + 1.5)
            })
            .collect();
        p.add(egui::Shape::line(pts, Stroke::new(w, color)));
    };
    arc(a0, a0 + sweep, t.sunken, 2.0);
    let (from, to) = if bipolar { (a0 + sweep * 0.5, a0 + sweep * frac) } else { (a0, a0 + sweep * frac) };
    arc(from.min(to), from.max(to), t.accent, 2.0);
    let a = a0 + sweep * frac;
    p.line_segment([c + vec2(a.cos(), a.sin()) * rad * 0.25, c + vec2(a.cos(), a.sin()) * rad * 0.9], Stroke::new(1.6, t.text));
    let text = tip(*v);
    if resp.dragged() {
        resp = resp.on_hover_text_at_pointer(text);
    } else {
        resp = resp.on_hover_text(text);
    }
    resp
}

/// A vertical fader. `v` is linear gain; the travel is shaped so 0 dB sits at about 3/4.
pub fn fader(ui: &mut Ui, v: &mut f32, max: f32, default: f32, size: egui::Vec2, tip: &str) -> Response {
    let t = Theme::current();
    let (r, mut resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    // Work in a perceptual position (cube root of gain) so the travel feels even.
    let to_pos = |g: f32| (g / max).max(0.0).cbrt();
    let from_pos = |p: f32| p.clamp(0.0, 1.0).powi(3) * max;
    let mut pos = to_pos(*v);
    let def_pos = to_pos(default);
    if drag_value(ui, &resp, &mut pos, 0.0, 1.0, def_pos, r.height() - 12.0) {
        *v = from_pos(pos);
        resp.mark_changed();
    }
    let p = ui.painter();
    let track = Rect::from_center_size(r.center(), vec2(4.0, r.height() - 10.0));
    p.rect_filled(track, 2.0, t.sunken);
    // 0 dB tick.
    let unity_y = track.bottom() - to_pos(1.0) * track.height();
    p.line_segment([pos2(r.left() + 2.0, unity_y), pos2(r.right() - 2.0, unity_y)], Stroke::new(1.0, t.grid_bar));
    let y = track.bottom() - pos * track.height();
    let handle = Rect::from_center_size(pos2(r.center().x, y), vec2(r.width() - 4.0, 10.0));
    let fill = if resp.dragged() { t.accent } else if resp.hovered() { t.key_white } else { mix(t.key_white, t.raised, 0.25) };
    p.rect_filled(handle, 2.0, fill);
    p.line_segment([pos2(handle.left() + 2.0, y), pos2(handle.right() - 2.0, y)], Stroke::new(1.0, t.raised));
    let db = limeshot_dsp::gain_to_db(*v);
    let label = if db.is_finite() { format!("{tip}: {db:+.1} dB") } else { format!("{tip}: −∞ dB") };
    resp = if resp.dragged() { resp.on_hover_text_at_pointer(label) } else { resp.on_hover_text(label) };
    resp
}

/// Map a linear peak to meter height (−60 dB … +6 dB).
pub fn meter_fraction(peak: f32) -> f32 {
    let db = limeshot_dsp::gain_to_db(peak);
    if !db.is_finite() {
        return 0.0;
    }
    ((db + 60.0) / 66.0).clamp(0.0, 1.0)
}

/// A stereo peak meter.
pub fn meter(ui: &mut Ui, size: egui::Vec2, left: f32, right: f32, clipped: bool) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 2.0, t.sunken);
    let w = (r.width() - 3.0) / 2.0;
    for (i, level) in [left, right].into_iter().enumerate() {
        let bar = Rect::from_min_size(pos2(r.left() + 1.0 + i as f32 * (w + 1.0), r.top() + 1.0), vec2(w, r.height() - 2.0));
        let f = meter_fraction(level);
        let top = bar.bottom() - f * bar.height();
        // Green to −12 dB, yellow to 0 dB, red above.
        let y_yellow = bar.bottom() - meter_fraction(limeshot_dsp::db_to_gain(-12.0)) * bar.height();
        let y_red = bar.bottom() - meter_fraction(1.0) * bar.height();
        let seg = |lo: f32, hi: f32, c: Color32| {
            let a = top.max(hi);
            if a < lo {
                p.rect_filled(Rect::from_min_max(pos2(bar.left(), a), pos2(bar.right(), lo)), 0.0, c);
            }
        };
        seg(bar.bottom(), y_yellow, t.meter_low);
        seg(y_yellow, y_red, t.meter_mid);
        seg(y_red, bar.top(), t.meter_high);
    }
    if clipped {
        p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), 3.0)), 0.0, t.danger);
    }
    p.rect_stroke(r, 2.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
    resp
}

/// A thin section heading line used inside panels.
pub fn section(ui: &mut Ui, text: &str) {
    let t = Theme::current();
    ui.label(egui::RichText::new(text).color(t.text_dim).small());
}
