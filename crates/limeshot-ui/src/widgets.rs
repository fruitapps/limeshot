//! Custom controls: knobs, faders, meters, icon buttons, panel headers and tab strips.
//!
//! Value widgets edit a local copy and report `changed()`; callers turn changes into commands.
//! Shared conventions (documented in `docs/architecture/ui.md`): drag vertically to change,
//! Shift for fine control, mouse wheel steps, double-click resets to the default.
//!
//! Style: flat. Controls have no fill until hovered or active, active state uses the accent, and
//! panels are separated by 1 px rules (see `theme.rs`).

use crate::icons::{self, Icon};
use crate::theme::{Theme, mix, mono, semibold};
use egui::{Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use std::f32::consts::PI;

/// A flat icon button: transparent at rest, filled on hover; `active` shows the accent.
pub fn icon_button(ui: &mut Ui, icon: Icon, active: bool, size: f32, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let fill = if active {
        mix(t.surface, t.accent, 0.16)
    } else if resp.hovered() {
        t.raised_hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), fill);
    let fg = if active {
        t.accent
    } else if resp.hovered() {
        t.text
    } else {
        mix(t.text_dim, t.text, 0.4)
    };
    icons::paint(ui.painter(), r.shrink(size * 0.22), icon, fg);
    resp.on_hover_text(tip)
}

/// A larger, filled transport button (play/stop): the primary actions of the app.
pub fn transport_button(ui: &mut Ui, icon: Icon, active: bool, size: f32, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let fill = if active {
        t.accent
    } else if resp.hovered() {
        t.raised_hover
    } else {
        t.raised
    };
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), fill);
    let fg = if active { t.text_on_accent } else { t.text };
    icons::paint(ui.painter(), r.shrink(size * 0.26), icon, fg);
    resp.on_hover_text(tip)
}

/// A dimmed icon for features that are not implemented yet; it explains why on hover and does
/// nothing when clicked.
pub fn unavailable_button(ui: &mut Ui, icon: Icon, size: f32, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    icons::paint(ui.painter(), r.shrink(size * 0.24), icon, t.text_faint);
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
    ui.painter().rect_filled(r, CornerRadius::same(t.radius_sm), fill);
    let fg = if on { t.text_on_accent } else { t.text_dim };
    ui.painter()
        .text(r.center(), Align2::CENTER_CENTER, text, semibold(10.0), fg);
    resp.on_hover_text(tip)
}

/// A segmented control (pill group). Returns the newly chosen index when one is clicked.
pub fn segmented(ui: &mut Ui, labels: &[&str], selected: usize, height: f32, tip: &str) -> Option<usize> {
    let t = Theme::current();
    let font = semibold(10.5);
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap(l.to_string(), font.clone(), t.text)
                .size()
                .x
                + 14.0
        })
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 4.0;
    let (r, _) = ui.allocate_exact_size(vec2(total, height), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), t.sunken);
    let mut x = r.left() + 2.0;
    let mut chosen = None;
    for (i, (l, w)) in labels.iter().zip(&widths).enumerate() {
        let rr = Rect::from_min_size(pos2(x, r.top() + 2.0), vec2(*w, height - 4.0));
        let resp = ui
            .interact(rr, ui.id().with(("seg", tip, i)), Sense::click())
            .on_hover_text(tip);
        let on = i == selected;
        if on {
            ui.painter().rect_filled(rr, CornerRadius::same(t.radius_sm), t.accent);
        } else if resp.hovered() {
            ui.painter().rect_filled(rr, CornerRadius::same(t.radius_sm), t.raised);
        }
        let fg = if on { t.text_on_accent } else { t.text_dim };
        ui.painter()
            .text(rr.center(), Align2::CENTER_CENTER, *l, font.clone(), fg);
        if resp.clicked() && !on {
            chosen = Some(i);
        }
        x += w;
    }
    chosen
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

/// A rotary knob drawn as a thin arc. `bipolar` draws the value from the centre (pan).
pub fn knob(
    ui: &mut Ui,
    v: &mut f32,
    min: f32,
    max: f32,
    default: f32,
    size: f32,
    bipolar: bool,
    tip: impl Fn(f32) -> String,
) -> Response {
    let t = Theme::current();
    let (r, mut resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    if drag_value(ui, &resp, v, min, max, default, 120.0) {
        resp.mark_changed();
    }
    let p = ui.painter();
    let c = r.center();
    let rad = size * 0.40;
    let hot = resp.hovered() || resp.dragged();
    p.circle_filled(c, rad * 0.72, if hot { t.raised_hover } else { t.raised });
    let frac = if max > min { (*v - min) / (max - min) } else { 0.0 };
    let a0 = PI * 0.75;
    let sweep = PI * 1.5;
    let arc = |from: f32, to: f32, color: Color32, w: f32| {
        let n = 24;
        let pts: Vec<_> = (0..=n)
            .map(|i| {
                let a = from + (to - from) * i as f32 / n as f32;
                c + vec2(a.cos(), a.sin()) * rad
            })
            .collect();
        p.add(egui::Shape::line(pts, Stroke::new(w, color)));
    };
    arc(a0, a0 + sweep, t.sunken, 2.5);
    let (from, to) = if bipolar {
        (a0 + sweep * 0.5, a0 + sweep * frac)
    } else {
        (a0, a0 + sweep * frac)
    };
    arc(
        from.min(to),
        from.max(to),
        if hot { t.accent_hover } else { t.accent },
        2.5,
    );
    let a = a0 + sweep * frac;
    p.line_segment(
        [
            c + vec2(a.cos(), a.sin()) * rad * 0.15,
            c + vec2(a.cos(), a.sin()) * rad * 0.62,
        ],
        Stroke::new(1.5, t.text),
    );
    let text = tip(*v);
    resp = if resp.dragged() {
        resp.on_hover_text_at_pointer(text)
    } else {
        resp.on_hover_text(text)
    };
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
    let track = Rect::from_center_size(r.center(), vec2(3.0, r.height() - 10.0));
    p.rect_filled(track, 1.5, t.sunken);
    let y = track.bottom() - pos * track.height();
    // Filled part of the track below the handle.
    p.rect_filled(
        Rect::from_min_max(pos2(track.left(), y), track.right_bottom()),
        1.5,
        t.accent_dim,
    );
    // 0 dB tick.
    let unity_y = track.bottom() - to_pos(1.0) * track.height();
    p.line_segment(
        [pos2(r.left() + 1.0, unity_y), pos2(r.left() + 5.0, unity_y)],
        Stroke::new(1.0, t.text_faint),
    );
    p.line_segment(
        [pos2(r.right() - 5.0, unity_y), pos2(r.right() - 1.0, unity_y)],
        Stroke::new(1.0, t.text_faint),
    );
    let handle = Rect::from_center_size(pos2(r.center().x, y), vec2(r.width() - 2.0, 14.0));
    let fill = if resp.dragged() {
        t.accent
    } else if resp.hovered() {
        t.key_white
    } else {
        mix(t.key_white, t.raised, 0.3)
    };
    p.rect_filled(handle, CornerRadius::same(t.radius_sm), fill);
    p.line_segment(
        [pos2(handle.left() + 3.0, y), pos2(handle.right() - 3.0, y)],
        Stroke::new(1.0, mix(fill, Color32::BLACK, 0.5)),
    );
    let db = limeshot_dsp::gain_to_db(*v);
    let label = if db.is_finite() {
        format!("{tip}: {db:+.1} dB")
    } else {
        format!("{tip}: −∞ dB")
    };
    resp = if resp.dragged() {
        resp.on_hover_text_at_pointer(label)
    } else {
        resp.on_hover_text(label)
    };
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

/// A stereo peak meter with segment colors and a clip LED on top.
pub fn meter(ui: &mut Ui, size: egui::Vec2, left: f32, right: f32, clipped: bool) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 2.0, t.sunken);
    let led = Rect::from_min_size(r.min, vec2(r.width(), 3.0));
    p.rect_filled(led, 1.0, if clipped { t.danger } else { t.raised });
    let body = Rect::from_min_max(pos2(r.left(), r.top() + 5.0), r.max);
    let w = (body.width() - 1.0) / 2.0;
    for (i, level) in [left, right].into_iter().enumerate() {
        let bar = Rect::from_min_size(
            pos2(body.left() + i as f32 * (w + 1.0), body.top()),
            vec2(w, body.height()),
        );
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
    resp
}

/// An upper-case section label ("TRACKS", "ENVELOPE"), as used across the ArtCraft apps.
pub fn section(ui: &mut Ui, text: &str) {
    let t = Theme::current();
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(semibold(10.5))
            .color(t.text_dim),
    );
}

/// A panel header strip: an upper-case title on the left, optional controls on the right
/// (`right` is laid out right-to-left). Draws the bottom rule.
pub fn panel_header(ui: &mut Ui, title: &str, right: impl FnOnce(&mut Ui)) {
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, t.header);
    ui.painter()
        .line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0, t.border));
    ui.painter().text(
        r.left_center() + vec2(10.0, 0.0),
        Align2::LEFT_CENTER,
        title.to_uppercase(),
        semibold(10.5),
        t.text_dim,
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(r.shrink2(vec2(6.0, 3.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    right(&mut child);
}

/// A tab strip with an accent underline under the active tab (like ArtCraft panel tabs).
/// Returns the clicked tab, if any. `extra` adds controls at the right edge.
pub fn tab_strip(ui: &mut Ui, tabs: &[(&str, &str)], active: usize, extra: impl FnOnce(&mut Ui)) -> Option<usize> {
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, t.header);
    ui.painter()
        .line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0, t.border));
    let mut x = r.left() + 6.0;
    let mut clicked = None;
    for (i, (label, tip)) in tabs.iter().enumerate() {
        let font = semibold(11.0);
        let w = ui
            .painter()
            .layout_no_wrap(label.to_uppercase(), font.clone(), t.text)
            .size()
            .x
            + 24.0;
        let tr = Rect::from_min_size(pos2(x, r.top()), vec2(w, r.height()));
        let resp = ui
            .interact(tr, ui.id().with(("tab", i, *label)), Sense::click())
            .on_hover_text(*tip);
        let on = i == active;
        let fg = if on {
            t.text
        } else if resp.hovered() {
            mix(t.text_dim, t.text, 0.6)
        } else {
            t.text_dim
        };
        ui.painter()
            .text(tr.center(), Align2::CENTER_CENTER, label.to_uppercase(), font, fg);
        if on {
            ui.painter().rect_filled(
                Rect::from_min_max(
                    pos2(tr.left() + 8.0, tr.bottom() - 2.0),
                    pos2(tr.right() - 8.0, tr.bottom()),
                ),
                1.0,
                t.accent,
            );
        }
        if resp.clicked() {
            clicked = Some(i);
        }
        x += w;
    }
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(x, r.top() + 3.0),
                r.right_bottom() - vec2(6.0, 3.0),
            ))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    extra(&mut child);
    clicked
}

/// A labelled readout box, e.g. for tempo or the position display.
pub fn readout(ui: &mut Ui, size: egui::Vec2, main: &str, caption: &str, tip: &str) -> Response {
    let t = Theme::current();
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect_filled(r, CornerRadius::same(t.radius), t.sunken);
    ui.painter().rect_stroke(
        r,
        CornerRadius::same(t.radius),
        Stroke::new(1.0, t.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        r.center() - vec2(0.0, 5.0),
        Align2::CENTER_CENTER,
        main,
        mono(15.0),
        t.accent,
    );
    ui.painter().text(
        r.center() + vec2(0.0, 10.0),
        Align2::CENTER_CENTER,
        caption,
        mono(8.5),
        t.text_dim,
    );
    // Painted text is invisible to screen readers; expose the value as the button's label.
    let label = format!("{main} {caption}");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &label));
    resp.on_hover_text(tip)
}

/// A thin vertical rule between toolbar groups.
pub fn vrule(ui: &mut Ui, height: f32) {
    let t = Theme::current();
    let (r, _) = ui.allocate_exact_size(vec2(9.0, height), Sense::hover());
    ui.painter().line_segment(
        [r.center_top() + vec2(0.0, 4.0), r.center_bottom() - vec2(0.0, 4.0)],
        Stroke::new(1.0, t.border),
    );
}

/// A swatch grid of the project palette (channel and pattern colours); the current colour is
/// outlined. Returns the swatch clicked. Each swatch is labelled "Color N" for accessibility.
pub fn swatch_picker(ui: &mut Ui, current: limeshot_core::Rgb) -> Option<limeshot_core::Rgb> {
    let t = Theme::current();
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.set_max_width(5.0 * 24.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for (i, c) in limeshot_core::PALETTE.into_iter().enumerate() {
            let (r, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            ui.painter().rect_filled(r, CornerRadius::same(3), crate::theme::rgb(c));
            if c == current || resp.hovered() {
                ui.painter().rect_stroke(
                    r.expand(1.5),
                    CornerRadius::same(4),
                    Stroke::new(1.5, if c == current { t.text } else { t.text_dim }),
                    StrokeKind::Outside,
                );
            }
            let label = format!("Color {}", i + 1);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
            if resp.clicked() {
                picked = Some(c);
            }
        }
    });
    picked
}

/// Double-click resets a standard egui slider to `default` (as the custom knobs and faders do).
/// egui's sliders only sense drags, so their response never reports a double-click itself.
pub fn reset_on_double_click<T: Copy + PartialEq>(mut r: Response, v: &mut T, default: T) -> Response {
    let double = r
        .ctx
        .input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
    if double && r.hovered() && *v != default {
        *v = default;
        r.mark_changed();
    }
    r
}

/// Whether a standard egui widget's `changed()` came from the user. egui sliders can report a
/// change on their own (rounding to the displayed decimals), which must not edit the project.
pub fn user_changed(r: &Response) -> bool {
    r.changed() && (r.hovered() || r.has_focus() || r.lost_focus() || r.dragged() || r.drag_stopped())
}
