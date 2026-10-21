//! Design tokens. Every color and font in the UI comes from here; widgets never hard-code colors.
//!
//! The look follows the ArtCraft apps' approach (see `docs/research/artcraft-architecture.md`):
//! near-black neutral surfaces, docked panels separated by 1 px rules, small upper-case section
//! labels, flat controls, Inter for text and JetBrains Mono for numbers. The palette and the lime
//! accent are LimeShot's own.

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Visuals};
use limeshot_core::Rgb;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    /// Behind everything; shows in the gaps between panels.
    pub bg: Color32,
    /// Header bar and toolbars.
    pub header: Color32,
    /// Panel bodies.
    pub surface: Color32,
    /// Raised controls (buttons, steps that are off).
    pub raised: Color32,
    pub raised_hover: Color32,
    /// Recessed areas (grids, fields, meters).
    pub sunken: Color32,
    /// 1 px rules between panels and around fields.
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub text_on_accent: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
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
    /// Corner radii in points.
    pub radius_sm: u8,
    pub radius: u8,
}

impl Theme {
    pub const LIME_DARK: Theme = Theme {
        name: "Lime Dark",
        bg: Color32::from_rgb(0x0B, 0x0C, 0x0D),
        header: Color32::from_rgb(0x14, 0x15, 0x17),
        surface: Color32::from_rgb(0x1A, 0x1B, 0x1E),
        raised: Color32::from_rgb(0x27, 0x29, 0x2D),
        raised_hover: Color32::from_rgb(0x31, 0x34, 0x39),
        sunken: Color32::from_rgb(0x12, 0x13, 0x15),
        border: Color32::from_rgb(0x2A, 0x2C, 0x30),
        text: Color32::from_rgb(0xDC, 0xDE, 0xE1),
        text_dim: Color32::from_rgb(0x8E, 0x93, 0x9A),
        text_faint: Color32::from_rgb(0x5C, 0x60, 0x67),
        text_on_accent: Color32::from_rgb(0x11, 0x16, 0x07),
        accent: Color32::from_rgb(0xA8, 0xE0, 0x3C),
        accent_hover: Color32::from_rgb(0xBC, 0xEC, 0x5A),
        accent_dim: Color32::from_rgb(0x3E, 0x55, 0x1A),
        warning: Color32::from_rgb(0xE8, 0xB3, 0x3C),
        danger: Color32::from_rgb(0xE5, 0x53, 0x4B),
        grid_bar: Color32::from_rgb(0x3A, 0x3D, 0x43),
        grid_beat: Color32::from_rgb(0x27, 0x29, 0x2D),
        grid_step: Color32::from_rgb(0x1D, 0x1F, 0x22),
        key_white: Color32::from_rgb(0xD9, 0xDC, 0xDF),
        key_black: Color32::from_rgb(0x1B, 0x1C, 0x1F),
        row_black_key: Color32::from_rgb(0x0E, 0x0F, 0x10),
        playhead: Color32::from_rgb(0xC6, 0xF2, 0x6A),
        selection: Color32::from_rgba_premultiplied(0x2C, 0x3A, 0x10, 0x60),
        meter_low: Color32::from_rgb(0x8C, 0xD1, 0x3C),
        meter_mid: Color32::from_rgb(0xE0, 0xC4, 0x4C),
        meter_high: Color32::from_rgb(0xE5, 0x53, 0x4B),
        step_on: Color32::from_rgb(0xA8, 0xE0, 0x3C),
        step_off_a: Color32::from_rgb(0x2A, 0x2C, 0x31),
        step_off_b: Color32::from_rgb(0x34, 0x2E, 0x31),
        radius_sm: 3,
        radius: 5,
    };

    /// The active theme. A single built-in theme for now; the token set is what a future theme
    /// switcher would swap.
    pub fn current() -> &'static Theme {
        &Self::LIME_DARK
    }

    /// Configure egui's own widgets (menus, text fields, scroll bars, sliders) from the tokens.
    pub fn apply(&self, ctx: &egui::Context) {
        install_fonts(ctx);
        let mut v = Visuals::dark();
        v.panel_fill = self.surface;
        v.window_fill = self.surface;
        v.extreme_bg_color = self.sunken;
        v.faint_bg_color = self.raised;
        v.code_bg_color = self.sunken;
        v.window_stroke = Stroke::new(1.0, self.border);
        v.window_corner_radius = CornerRadius::same(self.radius + 1);
        v.menu_corner_radius = CornerRadius::same(self.radius + 1);
        v.window_shadow = egui::epaint::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: Color32::from_black_alpha(140),
        };
        v.popup_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };
        v.selection.bg_fill = self.accent_dim;
        v.selection.stroke = Stroke::new(1.0, self.accent);
        v.hyperlink_color = self.accent;
        v.warn_fg_color = self.warning;
        v.error_fg_color = self.danger;
        v.override_text_color = Some(self.text);
        v.slider_trailing_fill = true;
        v.handle_shape = egui::style::HandleShape::Circle;
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
            state.corner_radius = CornerRadius::same(self.radius_sm + 1);
            state.expansion = 0.0;
        }
        w.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.bg_stroke = Stroke::NONE;
        w.active.bg_stroke = Stroke::new(1.0, self.accent);
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.text_dim);
        w.inactive.fg_stroke = Stroke::new(1.0, self.text);
        w.hovered.fg_stroke = Stroke::new(1.5, self.text);
        w.active.fg_stroke = Stroke::new(1.5, self.accent);
        ctx.set_visuals(v);
        ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = egui::vec2(6.0, 4.0);
            s.spacing.button_padding = egui::vec2(8.0, 3.0);
            s.spacing.interact_size.y = 22.0;
            s.spacing.slider_rail_height = 3.0;
            s.spacing.scroll = scroll_style();
            s.spacing.window_margin = egui::Margin::same(10);
            s.spacing.menu_margin = egui::Margin::same(6);
            s.text_styles.insert(egui::TextStyle::Body, prop(12.5));
            s.text_styles.insert(egui::TextStyle::Button, prop(12.5));
            s.text_styles.insert(egui::TextStyle::Small, prop(11.0));
            s.text_styles.insert(egui::TextStyle::Heading, semibold(15.0));
            s.text_styles.insert(egui::TextStyle::Monospace, mono(12.0));
        });
    }
}

const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");
const JETBRAINS_MONO: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// Inter for text, Inter SemiBold for labels, JetBrains Mono for numbers. egui's default fonts stay
/// as fallbacks (symbols, emoji).
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("inter".into(), Arc::new(FontData::from_static(INTER)));
    fonts
        .font_data
        .insert("inter-semibold".into(), Arc::new(FontData::from_static(INTER_SEMIBOLD)));
    fonts
        .font_data
        .insert("jetbrains-mono".into(), Arc::new(FontData::from_static(JETBRAINS_MONO)));
    let fallbacks: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut prop_fam = vec!["inter".to_string()];
    prop_fam.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, prop_fam);
    let mut semi = vec!["inter-semibold".to_string()];
    semi.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Name("semibold".into()), semi);
    let mut mono_fam = vec!["jetbrains-mono".to_string()];
    mono_fam.extend(fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default());
    fonts.families.insert(FontFamily::Monospace, mono_fam);
    ctx.set_fonts(fonts);
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

/// Black or white, whichever reads better on `c`.
pub fn text_on(c: Color32) -> Color32 {
    let l = 0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b());
    if l > 140.0 {
        Color32::from_rgb(0x12, 0x13, 0x15)
    } else {
        Color32::from_rgb(0xF2, 0xF3, 0xF4)
    }
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn prop(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

/// Scroll bars: thin floating bars in the widget fills (raised, hover, accent) instead of egui's
/// bright foreground-coloured bar.
pub fn scroll_style() -> egui::style::ScrollStyle {
    egui::style::ScrollStyle {
        bar_width: 8.0,
        floating_width: 4.0,
        foreground_color: false,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.5,
        dormant_handle_opacity: 0.0,
        active_handle_opacity: 0.8,
        interact_handle_opacity: 1.0,
        ..egui::style::ScrollStyle::floating()
    }
}

/// A solid thin bar with its own gutter, for content that must not be covered (Mixer strips).
pub fn gutter_scroll_style() -> egui::style::ScrollStyle {
    egui::style::ScrollStyle {
        bar_width: 6.0,
        bar_inner_margin: 3.0,
        foreground_color: false,
        dormant_background_opacity: 0.6,
        active_background_opacity: 0.8,
        interact_background_opacity: 1.0,
        dormant_handle_opacity: 0.7,
        active_handle_opacity: 0.9,
        interact_handle_opacity: 1.0,
        ..egui::style::ScrollStyle::solid()
    }
}
