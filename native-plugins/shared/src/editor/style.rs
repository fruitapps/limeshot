use egui::*;

/// LimeShot's Lime Dark surfaces with a restrained identity color for each editor.
#[derive(Clone, Copy)]
pub(super) struct Theme {
    pub bg: Color32,
    pub header: Color32,
    pub surface: Color32,
    pub sunken: Color32,
    pub raised: Color32,
    pub border: Color32,
    pub edge: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub lime: Color32,
    pub danger: Color32,
    pub cyan: Color32,
    pub purple: Color32,
    pub amber: Color32,
}
impl Theme {
    pub fn current() -> Self {
        Self::for_plugin(0)
    }
    pub fn for_plugin(id: u8) -> Self {
        let accents = [
            (185, 218, 108),
            (115, 217, 174),
            (144, 190, 222),
            (173, 219, 111),
            (151, 220, 177),
            (240, 181, 99),
            (185, 162, 215),
            (231, 162, 180),
            (181, 221, 103),
        ];
        let (r, g, b) = accents[id as usize];
        Self {
            bg: Color32::from_rgb(11, 12, 13),
            header: Color32::from_rgb(20, 21, 23),
            surface: Color32::from_rgb(26, 27, 30),
            sunken: Color32::from_rgb(18, 19, 21),
            raised: Color32::from_rgb(39, 41, 45),
            border: Color32::from_rgb(42, 44, 48),
            edge: Color32::from_rgb(75, 79, 86),
            text: Color32::from_rgb(220, 222, 225),
            dim: Color32::from_rgb(142, 147, 154),
            accent: Color32::from_rgb(r, g, b),
            lime: Color32::from_rgb(168, 224, 60),
            danger: Color32::from_rgb(235, 127, 116),
            cyan: Color32::from_rgb(109, 195, 218),
            purple: Color32::from_rgb(187, 151, 222),
            amber: Color32::from_rgb(241, 181, 103),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Canvas {
    pub origin: Pos2,
    pub s: f32,
    pub w: f32,
    pub h: f32,
}
impl Canvas {
    /// Fit a design-space size uniformly inside an egui rectangle, centered in unused space.
    pub fn new(rect: Rect, size: (f32, f32)) -> Self {
        let s = (rect.width() / size.0).min(rect.height() / size.1);
        Self {
            origin: rect.center() - vec2(size.0, size.1) * s * 0.5,
            s,
            w: size.0,
            h: size.1,
        }
    }
    pub fn p(self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.s
    }
    pub fn r(self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.p(x, y), vec2(w, h) * self.s)
    }
}

pub(super) fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let v = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8;
    Color32::from_rgb(v(a.r(), b.r()), v(a.g(), b.g()), v(a.b(), b.b()))
}

pub(super) fn panel(p: &Painter, r: Rect, title: &str, t: &Theme, s: f32) {
    p.rect_filled(r, 7, t.surface);
    p.rect_stroke(r, 7, Stroke::new(s, t.border), StrokeKind::Inside);
    if !title.is_empty() {
        p.text(
            r.min + vec2(13.0, 13.0) * s,
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(10.0 * s),
            t.dim,
        );
    }
}
