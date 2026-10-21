//! Icons drawn with the egui painter: resolution independent, themeable, no image assets.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Play,
    Pause,
    Stop,
    Record,
    Metronome,
    Loop,
    Browser,
    ChannelRack,
    PianoRoll,
    Playlist,
    Mixer,
    Plus,
    Minus,
    Close,
    Folder,
    File,
    Wave,
    Synth,
    Undo,
    Redo,
    Save,
    Export,
    Inspector,
    Maximize,
    Restore,
    Detach,
    Attach,
    Automation,
    // Editor tools (FL Studio's Piano Roll and Playlist tool strip).
    ToolDraw,
    ToolPaint,
    ToolDelete,
    ToolMute,
    ToolSlice,
    ToolSelect,
    ToolZoom,
    ToolPlayback,
}

fn tri(p: &Painter, pts: [Pos2; 3], c: Color32) {
    p.add(Shape::convex_polygon(pts.to_vec(), c, Stroke::NONE));
}

/// Paint `icon` centred in `r`.
pub fn paint(p: &Painter, r: Rect, icon: Icon, c: Color32) {
    let s = r.width().min(r.height());
    let c0 = r.center();
    let u = s / 16.0;
    let at = |x: f32, y: f32| pos2(c0.x + x * u, c0.y + y * u);
    let line = Stroke::new((1.4 * u).max(1.0), c);
    match icon {
        Icon::Play => tri(p, [at(-4.0, -5.5), at(5.5, 0.0), at(-4.0, 5.5)], c),
        Icon::Pause => {
            p.rect_filled(Rect::from_min_max(at(-4.5, -5.0), at(-1.5, 5.0)), 1.0, c);
            p.rect_filled(Rect::from_min_max(at(1.5, -5.0), at(4.5, 5.0)), 1.0, c);
        }
        Icon::Stop => {
            p.rect_filled(Rect::from_min_max(at(-4.5, -4.5), at(4.5, 4.5)), 1.5, c);
        }
        Icon::Record => {
            p.circle_filled(c0, 5.0 * u, c);
        }
        Icon::Metronome => {
            tri(p, [at(-4.5, 6.0), at(4.5, 6.0), at(0.0, -6.5)], c);
            p.line_segment(
                [at(0.0, 3.0), at(4.5, -5.0)],
                Stroke::new(1.6 * u, Color32::BLACK.gamma_multiply(0.6)),
            );
            p.line_segment([at(0.0, 3.0), at(4.5, -5.0)], Stroke::new(0.9 * u, c));
        }
        Icon::Loop => {
            let pts: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let a = 0.15 * TAU + i as f32 / 24.0 * 0.78 * TAU;
                    at(5.0 * a.cos(), 5.0 * a.sin())
                })
                .collect();
            p.add(Shape::line(pts, line));
            tri(p, [at(4.6, -4.2), at(6.6, 0.4), at(1.8, -0.4)], c);
        }
        Icon::Browser => {
            for (i, y) in [-4.5f32, 0.0, 4.5].iter().enumerate() {
                let x0 = if i == 0 { -5.5 } else { -3.0 };
                p.rect_filled(Rect::from_min_max(at(x0, y - 1.3), at(5.5, y + 1.3)), 0.8, c);
            }
            p.line_segment([at(-5.5, -3.0), at(-5.5, 4.5)], line);
        }
        Icon::ChannelRack => {
            for row in 0..3 {
                for col in 0..4 {
                    let x = -6.0 + col as f32 * 3.4;
                    let y = -5.5 + row as f32 * 4.0;
                    let on = (row + col) % 3 == 0;
                    let rr = Rect::from_min_size(at(x, y), vec2(2.6 * u, 3.0 * u));
                    if on {
                        p.rect_filled(rr, 0.6, c);
                    } else {
                        p.rect_stroke(rr, 0.6, Stroke::new(0.9 * u, c), egui::StrokeKind::Inside);
                    }
                }
            }
        }
        Icon::PianoRoll => {
            p.rect_stroke(
                Rect::from_min_max(at(-6.5, -6.0), at(-2.5, 6.0)),
                0.8,
                line,
                egui::StrokeKind::Inside,
            );
            for y in [-3.0f32, 1.0] {
                p.rect_filled(Rect::from_min_max(at(-6.5, y - 0.9), at(-4.2, y + 0.9)), 0.3, c);
            }
            p.rect_filled(Rect::from_min_max(at(-1.0, -4.5), at(3.0, -2.5)), 0.6, c);
            p.rect_filled(Rect::from_min_max(at(1.0, -0.5), at(6.5, 1.5)), 0.6, c);
            p.rect_filled(Rect::from_min_max(at(-1.0, 3.0), at(2.0, 5.0)), 0.6, c);
        }
        Icon::Playlist => {
            for (y, x0, x1) in [(-4.5f32, -6.5f32, 0.0f32), (0.0, -2.0, 6.5), (4.5, -6.5, 3.0)] {
                p.rect_filled(Rect::from_min_max(at(x0, y - 1.6), at(x1, y + 1.6)), 0.8, c);
            }
        }
        Icon::Mixer => {
            for (i, knob) in [2.0f32, -3.0, 0.5].iter().enumerate() {
                let x = -4.5 + i as f32 * 4.5;
                p.line_segment([at(x, -6.0), at(x, 6.0)], Stroke::new(0.9 * u, c));
                p.rect_filled(Rect::from_center_size(at(x, *knob), vec2(3.4 * u, 2.2 * u)), 0.6, c);
            }
        }
        Icon::Plus => {
            p.line_segment([at(-5.0, 0.0), at(5.0, 0.0)], line);
            p.line_segment([at(0.0, -5.0), at(0.0, 5.0)], line);
        }
        Icon::ToolDraw => {
            // Pencil: a diagonal body with a tip.
            p.line_segment([at(-3.0, 3.0), at(4.0, -4.0)], Stroke::new(3.0 * u, c));
            tri(p, [at(-3.9, 1.9), at(-1.9, 3.9), at(-5.5, 5.5)], c);
        }
        Icon::ToolPaint => {
            // Brush: a handle and a wide head.
            p.line_segment([at(1.0, -1.0), at(5.5, -5.5)], Stroke::new(1.6 * u, c));
            p.add(Shape::convex_polygon(
                vec![
                    at(-1.0, -2.0),
                    at(2.0, 1.0),
                    at(-2.0, 5.5),
                    at(-6.0, 5.5),
                    at(-5.5, 2.0),
                ],
                c,
                Stroke::NONE,
            ));
        }
        Icon::ToolDelete => {
            p.circle_stroke(c0, 5.0 * u, line);
            p.line_segment([at(-3.5, 3.5), at(3.5, -3.5)], line);
        }
        Icon::ToolMute => {
            // Speaker with a cross.
            p.add(Shape::convex_polygon(
                vec![
                    at(-6.0, -2.0),
                    at(-3.5, -2.0),
                    at(0.0, -5.0),
                    at(0.0, 5.0),
                    at(-3.5, 2.0),
                    at(-6.0, 2.0),
                ],
                c,
                Stroke::NONE,
            ));
            p.line_segment([at(2.0, -2.5), at(6.0, 2.5)], line);
            p.line_segment([at(2.0, 2.5), at(6.0, -2.5)], line);
        }
        Icon::ToolSlice => {
            // Knife: a handle, a wide blade and a dashed cut line below it.
            p.rect_filled(Rect::from_min_max(at(-1.2, -7.0), at(1.8, -3.5)), 0.8, c);
            p.add(Shape::convex_polygon(
                vec![at(-3.0, -3.0), at(3.5, -3.0), at(3.5, 2.5), at(-3.0, 0.5)],
                c,
                Stroke::NONE,
            ));
            p.line_segment([at(0.3, 4.0), at(0.3, 7.0)], Stroke::new(1.0 * u, c));
        }
        Icon::ToolSelect => {
            for (a, b) in [
                ((-5.5, -5.5), (-1.5, -5.5)),
                ((1.5, -5.5), (5.5, -5.5)),
                ((-5.5, 5.5), (-1.5, 5.5)),
                ((1.5, 5.5), (5.5, 5.5)),
                ((-5.5, -5.5), (-5.5, -1.5)),
                ((-5.5, 1.5), (-5.5, 5.5)),
                ((5.5, -5.5), (5.5, -1.5)),
                ((5.5, 1.5), (5.5, 5.5)),
            ] {
                p.line_segment([at(a.0, a.1), at(b.0, b.1)], line);
            }
        }
        Icon::ToolZoom => {
            p.circle_stroke(at(-1.0, -1.0), 4.0 * u, line);
            p.line_segment([at(2.0, 2.0), at(6.0, 6.0)], Stroke::new(2.0 * u, c));
        }
        Icon::ToolPlayback => {
            // Speaker with sound waves.
            p.add(Shape::convex_polygon(
                vec![
                    at(-6.0, -2.0),
                    at(-3.5, -2.0),
                    at(0.0, -5.0),
                    at(0.0, 5.0),
                    at(-3.5, 2.0),
                    at(-6.0, 2.0),
                ],
                c,
                Stroke::NONE,
            ));
            for r in [3.0f32, 5.5] {
                let pts: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let a = -0.6 + 1.2 * i as f32 / 8.0;
                        at(r * a.cos(), r * a.sin())
                    })
                    .collect();
                p.add(Shape::line(pts, line));
            }
        }
        Icon::Minus => {
            p.line_segment([at(-5.0, 0.0), at(5.0, 0.0)], line);
        }
        Icon::Close => {
            p.line_segment([at(-4.0, -4.0), at(4.0, 4.0)], line);
            p.line_segment([at(-4.0, 4.0), at(4.0, -4.0)], line);
        }
        Icon::Folder => {
            p.add(Shape::convex_polygon(
                vec![
                    at(-6.5, -4.5),
                    at(-2.0, -4.5),
                    at(-0.5, -3.0),
                    at(6.5, -3.0),
                    at(6.5, 5.0),
                    at(-6.5, 5.0),
                ],
                c.gamma_multiply(0.85),
                Stroke::NONE,
            ));
        }
        Icon::File => {
            p.add(Shape::convex_polygon(
                vec![
                    at(-4.5, -6.0),
                    at(2.0, -6.0),
                    at(4.5, -3.5),
                    at(4.5, 6.0),
                    at(-4.5, 6.0),
                ],
                Color32::TRANSPARENT,
                line,
            ));
        }
        Icon::Automation => {
            p.line_segment([at(-4.5, 3.5), at(4.5, -3.5)], line);
            p.circle_stroke(at(-5.0, 4.0), 2.0 * u, line);
            p.circle_stroke(at(5.0, -4.0), 2.0 * u, line);
        }
        Icon::Wave => {
            let pts: Vec<Pos2> = (0..=28)
                .map(|i| {
                    let x = -6.5 + i as f32 * 13.0 / 28.0;
                    let env = 1.0 - (x / 6.5).abs();
                    at(x, (i as f32 * 1.3).sin() * 5.5 * env)
                })
                .collect();
            p.add(Shape::line(pts, line));
        }
        Icon::Synth => {
            let pts = vec![
                at(-6.5, 3.0),
                at(-3.0, -4.5),
                at(-3.0, 4.5),
                at(0.5, -4.5),
                at(0.5, 4.5),
                at(4.0, -4.5),
                at(4.0, 3.0),
                at(6.5, 3.0),
            ];
            p.add(Shape::line(pts, line));
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Redo { -1.0 } else { 1.0 };
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let a = -0.25 * TAU + i as f32 / 16.0 * 0.5 * TAU;
                    at(flip * (1.0 + 4.5 * a.cos()), 1.0 + 4.5 * a.sin())
                })
                .collect();
            p.add(Shape::line(pts, line));
            p.line_segment([at(flip * 1.0, -3.5), at(flip * -5.0, -3.5)], line);
            tri(
                p,
                [at(flip * -6.5, -3.5), at(flip * -3.5, -6.5), at(flip * -3.5, -0.5)],
                c,
            );
        }
        Icon::Save => {
            p.rect_stroke(
                Rect::from_min_max(at(-5.5, -5.5), at(5.5, 5.5)),
                1.0,
                line,
                egui::StrokeKind::Inside,
            );
            p.rect_filled(Rect::from_min_max(at(-3.0, -5.5), at(3.0, -1.5)), 0.5, c);
            p.rect_filled(
                Rect::from_min_max(at(-3.5, 1.5), at(3.5, 5.5)),
                0.5,
                c.gamma_multiply(0.6),
            );
        }
        Icon::Maximize => {
            p.rect_stroke(
                Rect::from_min_max(at(-5.5, -5.0), at(5.5, 5.0)),
                1.0,
                line,
                egui::StrokeKind::Inside,
            );
            p.line_segment([at(-5.5, -3.0), at(5.5, -3.0)], line);
        }
        Icon::Restore => {
            p.rect_stroke(
                Rect::from_min_max(at(-6.0, -2.5), at(3.0, 6.0)),
                1.0,
                line,
                egui::StrokeKind::Inside,
            );
            p.add(Shape::line(
                vec![
                    at(-3.0, -2.5),
                    at(-3.0, -6.0),
                    at(6.0, -6.0),
                    at(6.0, 3.0),
                    at(3.0, 3.0),
                ],
                line,
            ));
        }
        Icon::Detach | Icon::Attach => {
            p.add(Shape::line(
                vec![
                    at(1.0, -6.0),
                    at(-6.0, -6.0),
                    at(-6.0, 6.0),
                    at(6.0, 6.0),
                    at(6.0, -1.0),
                ],
                line,
            ));
            let (from, to) = if icon == Icon::Detach {
                (at(-1.5, 1.5), at(6.0, -6.0))
            } else {
                (at(6.0, -6.0), at(-1.5, 1.5))
            };
            p.line_segment([from, to], line);
            let tip = to;
            let back = if icon == Icon::Detach {
                vec2(-3.5 * u, 0.0)
            } else {
                vec2(3.5 * u, 0.0)
            };
            let down = if icon == Icon::Detach {
                vec2(0.0, 3.5 * u)
            } else {
                vec2(0.0, -3.5 * u)
            };
            p.line_segment([tip, tip + back], line);
            p.line_segment([tip, tip + down], line);
        }
        Icon::Inspector => {
            p.rect_stroke(
                Rect::from_min_max(at(-6.5, -5.5), at(6.5, 5.5)),
                1.0,
                line,
                egui::StrokeKind::Inside,
            );
            p.line_segment([at(2.0, -5.5), at(2.0, 5.5)], line);
            for y in [-2.5f32, 0.0, 2.5] {
                p.line_segment([at(3.5, y), at(5.0, y)], Stroke::new(0.9 * u, c));
            }
        }
        Icon::Export => {
            p.line_segment([at(0.0, 2.0), at(0.0, -6.0)], line);
            tri(p, [at(-3.5, -3.0), at(3.5, -3.0), at(0.0, -7.0)], c);
            p.add(Shape::line(
                vec![at(-6.0, 0.0), at(-6.0, 6.0), at(6.0, 6.0), at(6.0, 0.0)],
                line,
            ));
        }
    }
}

/// The LimeShot mark: a lime slice with a "shot" notch. Drawn, never loaded.
pub fn logo(p: &Painter, center: Pos2, radius: f32) {
    let t = crate::theme::Theme::current();
    let rind = Color32::from_rgb(0x5E, 0x9E, 0x1E);
    let pith = Color32::from_rgb(0xEA, 0xF5, 0xC8);
    p.circle_filled(center, radius, rind);
    p.circle_filled(center, radius * 0.86, pith);
    p.circle_filled(center, radius * 0.78, t.accent);
    // Segments: wedges separated by pith-colored spokes.
    let n = 8;
    for i in 0..n {
        let a = i as f32 / n as f32 * TAU + 0.2;
        let d = vec2(a.cos(), a.sin());
        p.line_segment(
            [center + d * radius * 0.12, center + d * radius * 0.8],
            Stroke::new(radius * 0.07, pith),
        );
    }
    p.circle_filled(center, radius * 0.14, pith);
}
