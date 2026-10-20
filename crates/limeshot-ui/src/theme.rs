//! Design tokens. Every color in the UI comes from a [`Theme`]; widgets never hard-code colors.
//! The palette (charcoal and graphite surfaces, lime accent) is LimeShot's own.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, Visuals};
use limeshot_core::Rgb;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    /// Behind everything (workspace).
    pub bg: Color32,
    /// Panels and window bodies.
    pub surface: Color32,
    /// Raised controls (buttons, step buttons off).
    pub raised: Color32,
    pub raised_hover: Color32,
    /// Sunken areas (grids, fields).
    pub sunken: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_on_accent: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub grid_bar: Color32,
    pub grid_beat: Color32,
    pub grid_step: Color32,
    pub key_white: Color32,
    pub key_black: Color32,
    pub row_black_key: Color32,
    pub playhead: Color32,
    pub selection: Color32,
    pub meter_low: Color32,
    pub meter_mid: Color32,
    pub meter_high: Color32,
    pub step_on: Color32,
    pub step_off_a: Color32,
    pub step_off_b: Color32,
}

impl Theme {
    pub const LIME_DARK: Theme = Theme {
        name: "Lime Dark",
        bg: Color32::from_rgb(0x17, 0x19, 0x1C),
        surface: Color32::from_rgb(0x22, 0x25, 0x29),
        raised: Color32::from_rgb(0x30, 0x34, 0x3A),
        raised_hover: Color32::from_rgb(0x3B, 0x40, 0x47),
        sunken: Color32::from_rgb(0x1D, 0x20, 0x24),
        border: Color32::from_rgb(0x3A, 0x3E, 0x45),
        text: Color32::from_rgb(0xE4, 0xE7, 0xEA),
        text_dim: Color32::from_rgb(0x8E, 0x95, 0x9D),
        text_on_accent: Color32::from_rgb(0x14, 0x1A, 0x08),
        accent: Color32::from_rgb(0xA6, 0xE0, 0x3A),
        accent_dim: Color32::from_rgb(0x5C, 0x7D, 0x24),
        warning: Color32::from_rgb(0xE8, 0xB3, 0x3C),
        danger: Color32::from_rgb(0xE5, 0x53, 0x4B),
        grid_bar: Color32::from_rgb(0x4A, 0x50, 0x58),
        grid_beat: Color32::from_rgb(0x34, 0x38, 0x3E),
        grid_step: Color32::from_rgb(0x28, 0x2B, 0x30),
        key_white: Color32::from_rgb(0xD8, 0xDC, 0xE0),
        key_black: Color32::from_rgb(0x26, 0x29, 0x2E),
        row_black_key: Color32::from_rgb(0x14, 0x16, 0x19),
        playhead: Color32::from_rgb(0xA6, 0xE0, 0x3A),
        selection: Color32::from_rgba_premultiplied(0x50, 0x6B, 0x1A, 0x60),
        meter_low: Color32::from_rgb(0x7C, 0xC8, 0x3A),
        meter_mid: Color32::from_rgb(0xD9, 0xC1, 0x4F),
        meter_high: Color32::from_rgb(0xE5, 0x53, 0x4B),
        step_on: Color32::from_rgb(0xA6, 0xE0, 0x3A),
        step_off_a: Color32::from_rgb(0x3A, 0x3F, 0x46),
        step_off_b: Color32::from_rgb(0x4A, 0x40, 0x46),
    };

    /// The active theme. A single built-in theme for now; the token set is what a future theme
    /// switcher would swap.
    pub fn current() -> &'static Theme {
        &Self::LIME_DARK
    }

    /// Configure egui's own widgets (menus, text fields, scroll bars) from the tokens.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = Visuals::dark();
        v.panel_fill = self.surface;
        v.window_fill = self.surface;
        v.extreme_bg_color = self.sunken;
        v.faint_bg_color = self.raised;
        v.code_bg_color = self.sunken;
        v.window_stroke = Stroke::new(1.0, self.border);
        v.window_corner_radius = CornerRadius::same(6);
        v.menu_corner_radius = CornerRadius::same(5);
        v.selection.bg_fill = self.accent_dim;
        v.selection.stroke = Stroke::new(1.0, self.accent);
        v.hyperlink_color = self.accent;
        v.warn_fg_color = self.warning;
        v.error_fg_color = self.danger;
        v.override_text_color = Some(self.text);
        let w = &mut v.widgets;
        for (state, fill) in [
            (&mut w.noninteractive, self.surface),
            (&mut w.inactive, self.raised),
            (&mut w.hovered, self.raised_hover),
            (&mut w.active, self.accent_dim),
            (&mut w.open, self.raised_hover),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.corner_radius = CornerRadius::same(4);
        }
        w.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.bg_stroke = Stroke::new(1.0, self.accent_dim);
        w.active.bg_stroke = Stroke::new(1.0, self.accent);
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.text_dim);
        w.inactive.fg_stroke = Stroke::new(1.0, self.text);
        w.hovered.fg_stroke = Stroke::new(1.0, self.text);
        w.active.fg_stroke = Stroke::new(1.0, self.text);
        ctx.set_visuals(v);
        ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = egui::vec2(6.0, 4.0);
            s.spacing.button_padding = egui::vec2(6.0, 2.0);
            s.spacing.interact_size.y = 20.0;
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::new(12.5, FontFamily::Proportional));
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::new(12.5, FontFamily::Proportional));
            s.text_styles
                .insert(egui::TextStyle::Small, FontId::new(10.5, FontFamily::Proportional));
            s.text_styles
                .insert(egui::TextStyle::Heading, FontId::new(15.0, FontFamily::Proportional));
            s.text_styles
                .insert(egui::TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace));
        });
    }
}

pub fn rgb(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

/// `c` mixed towards `with` by `t` (0–1).
pub fn mix(c: Color32, with: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgb(m(c.r(), with.r()), m(c.g(), with.g()), m(c.b(), with.b()))
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn prop(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}
