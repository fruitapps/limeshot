//! The Piano Roll's tool dialogs (FL Studio's Tools menu: Quantize…, Chop…, Arpeggiate…, Strum…
//! …). As in FL Studio, a dialog changes the notes while it is open: every change replaces the
//! last preview, Accept keeps the result as one undo step, and closing the dialog (× or Esc)
//! puts the notes back. The transforms are in `limeshot_core::note_tools`.

use crate::app::LimeShotApp;
use crate::piano_roll::key_name;
use egui::{Context, Slider, Ui};
use limeshot_core::note_tools::{self as nt, ArpPattern, Level, Scale, Shape, SnapDir};
use limeshot_core::*;

/// Which dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Quantize,
    Chop,
    Articulate,
    Arpeggiate,
    Strum,
    Flam,
    Limit,
    Flip,
    Randomize,
    ScaleLevels,
    Lfo,
}

impl Kind {
    /// The window title, as FL Studio's ("Piano roll - strumizer").
    fn title(self) -> &'static str {
        match self {
            Kind::Quantize => "Piano roll - quantizer",
            Kind::Chop => "Piano roll - chopper",
            Kind::Articulate => "Piano roll - articulator",
            Kind::Arpeggiate => "Piano roll - arpeggiator",
            Kind::Strum => "Piano roll - strumizer",
            Kind::Flam => "Piano roll - flamer",
            Kind::Limit => "Piano roll - limiter / transposer",
            Kind::Flip => "Piano roll - flipper",
            Kind::Randomize => "Piano roll - randomizer",
            Kind::ScaleLevels => "Piano roll - level scaling",
            Kind::Lfo => "Piano roll - LFO",
        }
    }

    fn tool(self) -> NoteTool {
        match self {
            Kind::Quantize => NoteTool::Quantize,
            Kind::Chop => NoteTool::Chop,
            Kind::Articulate => NoteTool::Articulate,
            Kind::Arpeggiate => NoteTool::Arpeggiate,
            Kind::Strum => NoteTool::Strum,
            Kind::Flam => NoteTool::Flam,
            Kind::Limit => NoteTool::Limit,
            Kind::Flip => NoteTool::Flip,
            Kind::Randomize => NoteTool::Randomize,
            Kind::ScaleLevels => NoteTool::ScaleLevels,
            Kind::Lfo => NoteTool::Lfo,
        }
    }
}

/// A dialog's settings.
#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    Quantize {
        start: f32,
        length: f32,
    },
    /// Piece length in snap steps.
    Chop {
        steps: u32,
        gate: f32,
    },
    Articulate {
        amount: f32,
        fixed: bool,
        steps: u32,
    },
    /// FL's Time mul: the step is the snap times this.
    Arpeggiate {
        arp: nt::Arp,
        time_mul: f32,
    },
    Strum(nt::Strum),
    Flam(nt::Flam),
    Limit(nt::Limit),
    Flip {
        horizontal: bool,
        vertical: bool,
    },
    Randomize(nt::Randomize),
    ScaleLevels(nt::ScaleLevels),
    /// LFO period in beats.
    Lfo {
        lfo: nt::Lfo,
        beats: f32,
    },
}

impl Params {
    fn new(kind: Kind, notes: &[Note]) -> Self {
        match kind {
            Kind::Quantize => Params::Quantize {
                start: 1.0,
                length: 0.0,
            },
            Kind::Chop => Params::Chop { steps: 1, gate: 1.0 },
            Kind::Articulate => Params::Articulate {
                amount: 0.5,
                fixed: false,
                steps: 1,
            },
            Kind::Arpeggiate => Params::Arpeggiate {
                arp: nt::Arp::default(),
                time_mul: 1.0,
            },
            Kind::Strum => Params::Strum(nt::Strum::default()),
            Kind::Flam => Params::Flam(nt::Flam::default()),
            Kind::Limit => Params::Limit(nt::Limit::around(notes)),
            Kind::Flip => Params::Flip {
                horizontal: true,
                vertical: false,
            },
            Kind::Randomize => Params::Randomize(nt::Randomize::default()),
            Kind::ScaleLevels => Params::ScaleLevels(nt::ScaleLevels::default()),
            Kind::Lfo => Params::Lfo {
                lfo: nt::Lfo::default(),
                beats: 4.0,
            },
        }
    }
}

/// An open tool dialog.
#[derive(Debug, Clone)]
pub struct ToolDialog {
    pub kind: Kind,
    pub params: Params,
    pattern: PatternId,
    /// The notes the tool works on, as they were when the dialog opened.
    original: Vec<Note>,
    /// The channel's template for new notes (Randomize) and the span it fills.
    template: Note,
    span: (Tick, Tick),
    /// The revision after the last preview, while that preview is the newest undo step.
    applied: Option<u64>,
    /// The notes the last preview left (selected again when the selection was not empty).
    current: Vec<NoteId>,
    keep_selection: bool,
}

/// Open `kind` on the selection (or the channel's notes when none is selected) and preview it.
pub fn open(app: &mut LimeShotApp, kind: Kind) {
    close(app, false);
    let pid = app.current_pattern();
    let Some(cid) = app.selected_channel() else { return };
    let sig = app.project().time_signature;
    let all: Vec<Note> = app
        .project()
        .pattern(pid)
        .map(|p| p.notes_for(cid).copied().collect())
        .unwrap_or_default();
    let sel = app.ui.piano.selection.clone();
    let original: Vec<Note> = if sel.is_empty() {
        all
    } else {
        all.into_iter().filter(|n| sel.contains(&n.id)).collect()
    };
    let pat_len = app
        .project()
        .pattern(pid)
        .map(|p| p.length(sig))
        .unwrap_or(sig.ticks_per_bar());
    let span = match (
        original.iter().map(|n| n.start).min(),
        original.iter().map(Note::end).max(),
    ) {
        (Some(a), Some(b)) if !sel.is_empty() => (a, b),
        _ => (0, pat_len),
    };
    let template = Note {
        id: NoteId(0),
        channel: cid,
        key: 60,
        start: 0,
        length: STEP_TICKS,
        velocity: app.ui.piano.last_vel,
        muted: false,
        pan: 0.0,
    };
    if original.is_empty() && kind != Kind::Randomize {
        app.hint("Draw some notes first: the tool works on the selection or the channel's notes");
        return;
    }
    app.ui.piano.tool_dialog = Some(ToolDialog {
        kind,
        params: Params::new(kind, &original),
        pattern: pid,
        original,
        template,
        span,
        applied: None,
        current: Vec::new(),
        keep_selection: !sel.is_empty(),
    });
    preview(app);
}

fn compute(d: &ToolDialog, snap: Tick) -> Vec<Note> {
    let o = &d.original;
    match &d.params {
        Params::Quantize { start, length } => nt::quantize(o, snap, *start, *length),
        Params::Chop { steps, gate } => nt::chop(o, snap * Tick::from(*steps), *gate),
        Params::Articulate { amount, fixed, steps } => nt::articulate(
            o,
            &nt::Articulate {
                amount: *amount,
                fixed: fixed.then_some(snap * Tick::from(*steps)),
            },
        ),
        Params::Arpeggiate { arp, time_mul } => nt::arpeggiate(
            o,
            &nt::Arp {
                time: ((snap as f32 * time_mul).round() as Tick).max(1),
                ..*arp
            },
        ),
        Params::Strum(s) => nt::strum(o, s, PPQ),
        Params::Flam(f) => nt::flam(o, f),
        Params::Limit(l) => nt::limit(o, l),
        Params::Flip { horizontal, vertical } => nt::flip(o, *horizontal, *vertical),
        Params::Randomize(r) => nt::randomize(d.template, d.span.0, d.span.1, snap, r),
        Params::ScaleLevels(s) => nt::scale_levels(o, s),
        Params::Lfo { lfo, beats } => nt::lfo(
            o,
            &nt::Lfo {
                period: ((PPQ as f32 * beats).round() as Tick).max(1),
                ..*lfo
            },
        ),
    }
}

fn note_ids(app: &LimeShotApp, pid: PatternId) -> Vec<NoteId> {
    app.project()
        .pattern(pid)
        .map(|p| p.notes.iter().map(|n| n.id).collect())
        .unwrap_or_default()
}

/// Replace the last preview with the dialog's current result.
fn preview(app: &mut LimeShotApp) {
    let Some(mut d) = app.ui.piano.tool_dialog.take() else {
        return;
    };
    if let Some(rev) = d.applied {
        if app.doc.revision() != rev {
            // Something else changed the project (an edit, an undo): keep it and stop.
            return;
        }
        app.doc.undo();
    }
    let sig = app.project().time_signature;
    let snap = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS);
    let after = compute(&d, snap);
    let remove: Vec<NoteId> = d.original.iter().map(|n| n.id).collect();
    let others: Vec<NoteId> = note_ids(app, d.pattern)
        .into_iter()
        .filter(|id| !remove.contains(id))
        .collect();
    let before = app.doc.revision();
    app.exec(Command::EditNotes {
        pattern: d.pattern,
        tool: d.kind.tool(),
        remove,
        notes: after,
    });
    d.applied = (app.doc.revision() != before).then(|| app.doc.revision());
    d.current = note_ids(app, d.pattern)
        .into_iter()
        .filter(|id| !others.contains(id))
        .collect();
    if d.keep_selection {
        app.ui.piano.selection = d.current.clone();
    }
    app.ui.piano.tool_dialog = Some(d);
}

/// Close the dialog: `accept` keeps the preview, otherwise the notes go back.
pub fn close(app: &mut LimeShotApp, accept: bool) {
    let Some(d) = app.ui.piano.tool_dialog.take() else {
        return;
    };
    if accept {
        return;
    }
    if d.applied.is_some_and(|rev| rev == app.doc.revision()) {
        app.doc.undo();
        if d.keep_selection {
            app.ui.piano.selection = d.original.iter().map(|n| n.id).collect();
        }
    }
}

fn key_slider(ui: &mut Ui, label: &str, key: &mut u8) -> bool {
    ui.label(label);
    let r = ui
        .add(Slider::new(key, 0..=127).custom_formatter(|v, _| key_name(v as u8)))
        .changed();
    ui.end_row();
    r
}

fn slider(ui: &mut Ui, label: &str, v: &mut f32, range: std::ops::RangeInclusive<f32>, pct: bool) -> bool {
    ui.label(label);
    let mut s = Slider::new(v, range);
    if pct {
        s = s
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
            .custom_parser(|t| t.trim_end_matches('%').trim().parse::<f64>().ok().map(|v| v / 100.0));
    }
    let r = ui.add(s).changed();
    ui.end_row();
    r
}

fn check(ui: &mut Ui, label: &str, v: &mut bool) -> bool {
    ui.label("");
    let r = ui.checkbox(v, label).changed();
    ui.end_row();
    r
}

fn choice<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    v: &mut T,
    all: &[T],
    name: impl Fn(T) -> &'static str,
) -> bool {
    ui.label(label);
    let mut changed = false;
    egui::ComboBox::from_id_salt(label)
        .selected_text(name(*v))
        .show_ui(ui, |ui| {
            for &o in all {
                changed |= ui.selectable_value(v, o, name(o)).changed();
            }
        });
    ui.end_row();
    changed
}

const ROOTS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

fn root_name(r: u8) -> &'static str {
    ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"][usize::from(r % 12)]
}

fn level_name(l: Level) -> &'static str {
    match l {
        Level::Velocity => "Note velocity",
        Level::Pan => "Note pan",
    }
}

/// The settings of one dialog; true when one changed.
fn controls(ui: &mut Ui, p: &mut Params) -> bool {
    let mut c = false;
    match p {
        Params::Quantize { start, length } => {
            c |= slider(ui, "Start time", start, 0.0..=1.0, true);
            c |= slider(ui, "Duration", length, 0.0..=1.0, true);
        }
        Params::Chop { steps, gate } => {
            ui.label("Time (steps of the snap)");
            c |= ui.add(Slider::new(steps, 1..=16)).changed();
            ui.end_row();
            c |= slider(ui, "Gate", gate, 0.05..=1.0, true);
        }
        Params::Articulate { amount, fixed, steps } => {
            c |= check(ui, "Fixed length", fixed);
            if *fixed {
                ui.label("Length (steps of the snap)");
                c |= ui.add(Slider::new(steps, 1..=16)).changed();
                ui.end_row();
            } else {
                c |= slider(ui, "Length (of the gap)", amount, 0.05..=1.0, true);
            }
        }
        Params::Arpeggiate { arp, time_mul } => {
            c |= choice(ui, "Pattern", &mut arp.pattern, &ArpPattern::ALL, ArpPattern::label);
            ui.label("Range (octaves)");
            c |= ui.add(Slider::new(&mut arp.range, 1..=4)).changed();
            ui.end_row();
            ui.label("Time mul");
            c |= ui
                .add(Slider::new(time_mul, 0.25..=4.0).logarithmic(true).suffix(" × snap"))
                .changed();
            ui.end_row();
            c |= slider(ui, "Gate", &mut arp.gate, 0.05..=1.0, true);
            c |= slider(ui, "Velocity slope", &mut arp.velocity, -1.0..=1.0, true);
            if arp.pattern == ArpPattern::Random {
                ui.label("Seed");
                c |= ui.add(egui::DragValue::new(&mut arp.seed).range(1..=9999)).changed();
                ui.end_row();
            }
        }
        Params::Strum(s) => {
            ui.label(egui::RichText::new("Start").strong());
            ui.end_row();
            c |= slider(ui, "Time", &mut s.time, 0.0..=1.0, true);
            c |= slider(ui, "Time tension", &mut s.time_tension, -1.0..=1.0, false);
            c |= slider(ui, "Velocity", &mut s.velocity, -1.0..=1.0, true);
            c |= slider(ui, "Velocity tension", &mut s.velocity_tension, -1.0..=1.0, false);
            c |= check(ui, "Preserve end", &mut s.preserve_end);
            c |= check(ui, "Trigger ahead", &mut s.trigger_ahead);
            ui.label(egui::RichText::new("End").strong());
            ui.end_row();
            c |= slider(ui, "End time", &mut s.end, 0.0..=1.0, true);
            c |= slider(ui, "End tension", &mut s.end_tension, -1.0..=1.0, false);
            c |= check(ui, "Chop chords", &mut s.chop_chords);
            c |= check(ui, "Alternate direction", &mut s.alternate);
        }
        Params::Flam(f) => {
            ui.label("Time (ticks)");
            c |= ui.add(Slider::new(&mut f.time, 10..=PPQ / 2)).changed();
            ui.end_row();
            c |= check(ui, "Before", &mut f.before);
            c |= slider(ui, "Velocity", &mut f.velocity, 0.0..=1.0, true);
        }
        Params::Limit(l) => {
            c |= choice(ui, "Key", &mut l.root, &ROOTS, root_name);
            c |= choice(ui, "Scale", &mut l.scale, &Scale::ALL, Scale::label);
            c |= choice(
                ui,
                "Snap",
                &mut l.snap,
                &[SnapDir::Above, SnapDir::Below, SnapDir::Nearest],
                |s| match s {
                    SnapDir::Above => "Above",
                    SnapDir::Below => "Below",
                    SnapDir::Nearest => "Nearest",
                },
            );
            c |= key_slider(ui, "Lowest key", &mut l.low);
            c |= key_slider(ui, "Highest key", &mut l.high);
            c |= check(ui, "Wrap to top / bottom", &mut l.wrap);
        }
        Params::Flip { horizontal, vertical } => {
            c |= check(ui, "Horizontally (time)", horizontal);
            c |= check(ui, "Vertically (pitch)", vertical);
        }
        Params::Randomize(r) => {
            ui.label("Octave");
            c |= ui.add(Slider::new(&mut r.octave, 0..=9)).changed();
            ui.end_row();
            ui.label("Range (octaves)");
            c |= ui.add(Slider::new(&mut r.range, 1..=5)).changed();
            ui.end_row();
            c |= choice(ui, "Key", &mut r.root, &ROOTS, root_name);
            c |= choice(ui, "Scale", &mut r.scale, &Scale::ALL, Scale::label);
            ui.label("Length (steps of the snap)");
            c |= ui.add(Slider::new(&mut r.length, 1..=16)).changed();
            ui.end_row();
            c |= slider(ui, "Variation", &mut r.variation, 0.0..=1.0, true);
            c |= slider(ui, "Population", &mut r.population, 0.0..=1.0, true);
            ui.label("Stack");
            c |= ui.add(Slider::new(&mut r.stack, 1..=6)).changed();
            ui.end_row();
            c |= check(ui, "Glue same notes", &mut r.glue);
            c |= slider(ui, "Velocity spread", &mut r.velocity, 0.0..=1.0, true);
            c |= slider(ui, "Pan spread", &mut r.pan, 0.0..=1.0, true);
            ui.label("Seed");
            ui.horizontal(|ui| {
                if ui.button("<").clicked() {
                    r.seed = r.seed.saturating_sub(1).max(1);
                    c = true;
                }
                ui.label(r.seed.to_string());
                if ui.button(">").clicked() {
                    r.seed += 1;
                    c = true;
                }
            });
            ui.end_row();
        }
        Params::ScaleLevels(s) => {
            c |= choice(ui, "Target", &mut s.level, &[Level::Velocity, Level::Pan], level_name);
            c |= slider(ui, "Center", &mut s.center, 0.0..=1.0, true);
            c |= slider(ui, "Multiply", &mut s.multiply, 0.0..=2.0, true);
            c |= slider(ui, "Offset", &mut s.offset, -1.0..=1.0, true);
            c |= slider(ui, "Tension", &mut s.tension, -1.0..=1.0, false);
        }
        Params::Lfo { lfo, beats } => {
            c |= choice(ui, "Target", &mut lfo.level, &[Level::Velocity, Level::Pan], level_name);
            c |= choice(ui, "Shape", &mut lfo.shape, &Shape::ALL, Shape::label);
            ui.label("Period (beats)");
            c |= ui.add(Slider::new(beats, 0.25..=16.0).logarithmic(true)).changed();
            ui.end_row();
            c |= slider(ui, "Amount", &mut lfo.amount, 0.0..=1.0, true);
            c |= slider(ui, "Center", &mut lfo.center, 0.0..=1.0, true);
            c |= slider(ui, "Phase", &mut lfo.phase, 0.0..=1.0, true);
        }
    }
    c
}

/// Draw the open tool dialog, if any.
pub fn window(app: &mut LimeShotApp, ctx: &Context) {
    let Some(mut d) = app.ui.piano.tool_dialog.clone() else {
        return;
    };
    let mut open = true;
    let mut changed = false;
    let mut accept = false;
    let mut reset = false;
    egui::Window::new(d.kind.title())
        .id(egui::Id::new("piano_tool_dialog"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_pos(ctx.content_rect().center() - egui::vec2(160.0, 160.0))
        .show(ctx, |ui| {
            egui::Grid::new("piano_tool_grid")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| changed = controls(ui, &mut d.params));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                reset = ui.button("Reset").clicked();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    accept = ui.button("Accept").clicked();
                });
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }
    if reset {
        d.params = Params::new(d.kind, &d.original);
        changed = true;
    }
    if let Some(cur) = app.ui.piano.tool_dialog.as_mut() {
        cur.params = d.params;
    }
    if accept {
        close(app, true);
    } else if !open {
        close(app, false);
    } else if changed {
        preview(app);
    }
}
