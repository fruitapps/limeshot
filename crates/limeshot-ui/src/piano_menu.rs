//! The Piano Roll's ⏵ options menu (FL Studio's Piano Roll menu: Edit, Tools, Select, Zoom …)
//! and the commands behind it. The editor's keys run the same [`Act`]s.

use crate::app::{ClipOp, LimeShotApp};
use crate::piano_roll::chop;
use crate::piano_tools::Kind;
use crate::shortcuts::label as key;
use egui::{Button, Context, Event, Key, Modifiers, Ui};
use limeshot_core::note_tools::{self, Rng};
use limeshot_core::*;

/// A Piano Roll command from FL Studio's menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    ShiftLeft,
    ShiftRight,
    RotateLeft,
    RotateRight,
    TransposeUp,
    TransposeDown,
    OctaveUp,
    OctaveDown,
    DiscardLengths,
    Mute,
    Unmute,
    QuickLegato,
    QuickQuantize,
    QuickQuantizeStarts,
    QuickChop,
    Glue,
    Deselect,
    SelectAll,
    SelectRandom,
    SelectMoreRandom,
    SelectOdd,
    SelectMuted,
    SelectOverlapping,
    SelectStacked,
    InvertSelection,
    ZoomIn,
    ZoomOut,
    /// Quick zoom 1–3, Zoom out far, Zoom on selection, Step sequencer mode zoom (Shift+1 … 6).
    QuickZoom(u8),
    Center,
    /// A tool dialog (Quantize…, Strum… …).
    Dialog(Kind),
}

/// The menu's items per submenu: (label, FL's key as LimeShot binds it, action).
const EDIT: &[(&str, &str, Act)] = &[
    ("Cut", "Cmd+X", Act::Cut),
    ("Copy", "Cmd+C", Act::Copy),
    ("Paste", "Cmd+V", Act::Paste),
    ("Duplicate", "Cmd+B", Act::Duplicate),
    ("Delete", "Delete", Act::Delete),
    ("Shift left", "Shift+←", Act::ShiftLeft),
    ("Shift right", "Shift+→", Act::ShiftRight),
    ("Rotate left", "Shift+Cmd+←", Act::RotateLeft),
    ("Rotate right", "Shift+Cmd+→", Act::RotateRight),
    ("Transpose up", "Shift+↑", Act::TransposeUp),
    ("Transpose down", "Shift+↓", Act::TransposeDown),
    ("Transpose one octave up", "Cmd+↑", Act::OctaveUp),
    ("Transpose one octave down", "Cmd+↓", Act::OctaveDown),
    ("Discard lengths", "Shift+D", Act::DiscardLengths),
    ("Mute", "Alt+M", Act::Mute),
    ("Unmute", "Shift+Alt+M", Act::Unmute),
];

const TOOLS: &[(&str, &str, Act)] = &[
    ("Quick legato", "Cmd+L", Act::QuickLegato),
    ("Articulate…", "Alt+L", Act::Dialog(Kind::Articulate)),
    ("Quick quantize", "", Act::QuickQuantize),
    ("Quick quantize start times", "Shift+Q", Act::QuickQuantizeStarts),
    ("Quantize…", "Alt+Q", Act::Dialog(Kind::Quantize)),
    ("Quick chop", "Cmd+U", Act::QuickChop),
    ("Chop…", "Alt+U", Act::Dialog(Kind::Chop)),
    ("Glue", "Cmd+G", Act::Glue),
    ("Arpeggiate…", "Alt+A", Act::Dialog(Kind::Arpeggiate)),
    ("Strum…", "Alt+S", Act::Dialog(Kind::Strum)),
    ("Flam…", "Alt+F", Act::Dialog(Kind::Flam)),
    ("Limit…", "Alt+K", Act::Dialog(Kind::Limit)),
    ("Flip…", "Alt+Y", Act::Dialog(Kind::Flip)),
    ("Randomize…", "Alt+R", Act::Dialog(Kind::Randomize)),
    ("Scale levels…", "Alt+X", Act::Dialog(Kind::ScaleLevels)),
    ("LFO…", "Alt+O", Act::Dialog(Kind::Lfo)),
];

/// The tool dialogs' keys (FL Studio's Opt+letter).
const DIALOG_KEYS: [(Key, Kind); 11] = [
    (Key::L, Kind::Articulate),
    (Key::Q, Kind::Quantize),
    (Key::U, Kind::Chop),
    (Key::A, Kind::Arpeggiate),
    (Key::S, Kind::Strum),
    (Key::F, Kind::Flam),
    (Key::K, Kind::Limit),
    (Key::Y, Kind::Flip),
    (Key::R, Kind::Randomize),
    (Key::X, Kind::ScaleLevels),
    (Key::O, Kind::Lfo),
];

const SELECT: &[(&str, &str, Act)] = &[
    ("Deselect", "Cmd+D", Act::Deselect),
    ("Select all", "Cmd+A", Act::SelectAll),
    ("Select one at random", "Shift+R", Act::SelectRandom),
    ("Select more at random", "Shift+M", Act::SelectMoreRandom),
    ("Select odd", "Shift+O", Act::SelectOdd),
    ("Select muted", "", Act::SelectMuted),
    ("Select overlapping notes", "", Act::SelectOverlapping),
    ("Select stacked notes", "", Act::SelectStacked),
    ("Invert selection", "Shift+I", Act::InvertSelection),
];

const ZOOM: &[(&str, &str, Act)] = &[
    ("Zoom in", "PgUp", Act::ZoomIn),
    ("Zoom out", "PgDn", Act::ZoomOut),
    ("Quick zoom 1", "Shift+1", Act::QuickZoom(1)),
    ("Quick zoom 2", "Shift+2", Act::QuickZoom(2)),
    ("Quick zoom 3", "Shift+3", Act::QuickZoom(3)),
    ("Zoom out far", "Shift+4", Act::QuickZoom(4)),
    ("Zoom on selection", "Shift+5", Act::QuickZoom(5)),
    ("Step sequencer mode zoom", "Shift+6", Act::QuickZoom(6)),
];

fn items(app: &mut LimeShotApp, ui: &mut Ui, list: &[(&str, &str, Act)]) {
    for &(text, keys, act) in list {
        if ui.add(Button::new(text).shortcut_text(key(keys))).clicked() {
            ui.close();
            run(app, act);
        }
    }
}

/// The ⏵ menu at the left of the Piano Roll's header.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui) {
    ui.menu_button("Edit", |ui| items(app, ui, EDIT));
    ui.menu_button("Tools", |ui| items(app, ui, TOOLS));
    ui.menu_button("Select", |ui| items(app, ui, SELECT));
    ui.menu_button("Zoom", |ui| items(app, ui, ZOOM));
    ui.menu_button("Stamp", |ui| stamp_menu(app, ui));
    ui.menu_button("View", |ui| {
        ui.menu_button("Scale highlighting", |ui| scale_menu(app, ui));
        ui.menu_button("Keyboard labels", |ui| {
            for (mode, label) in crate::piano_roll::KeyboardLabels::ALL {
                if ui.radio_value(&mut app.ui.piano.keyboard_labels, mode, label).clicked() {
                    ui.close();
                }
            }
            ui.separator();
            ui.checkbox(&mut app.ui.piano.white_key_labels_only, "White notes only");
        });
        ui.checkbox(&mut app.ui.piano.keep_labels_on_screen, "Keep labels on screen");
    });
    ui.menu_button("Target control", |ui| {
        for (prop, label) in [(NoteProp::Velocity, "Note velocity"), (NoteProp::Pan, "Note pan")] {
            if ui.radio_value(&mut app.ui.piano.target_control, prop, label).clicked() {
                app.doc.end_gesture();
                ui.close();
            }
        }
    });
    if ui.add(Button::new("Center").shortcut_text(key("Shift+0"))).clicked() {
        ui.close();
        run(app, Act::Center);
    }
}

const ROOTS: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Stamp: a chord or scale that the Draw tool places (FL Studio's Stamp tool), or none.
fn stamp_menu(app: &mut LimeShotApp, ui: &mut Ui) {
    use limeshot_core::note_tools::{CHORDS, Scale, Stamp};
    if ui
        .add(Button::selectable(app.ui.piano.stamp.is_none(), "(none)"))
        .clicked()
    {
        app.ui.piano.stamp = None;
        ui.close();
    }
    ui.separator();
    for i in 0..CHORDS.len() {
        let s = Stamp::Chord(i);
        if ui
            .add(Button::selectable(app.ui.piano.stamp == Some(s), s.label()))
            .clicked()
        {
            app.ui.piano.stamp = Some(s);
            app.ui.piano.tool = crate::piano_roll::Tool::Draw;
            ui.close();
        }
    }
    ui.menu_button("Scales", |ui| {
        for scale in Scale::ALL.into_iter().skip(1) {
            let s = Stamp::Scale(scale);
            if ui
                .add(Button::selectable(app.ui.piano.stamp == Some(s), s.label()))
                .clicked()
            {
                app.ui.piano.stamp = Some(s);
                app.ui.piano.tool = crate::piano_roll::Tool::Draw;
                ui.close();
            }
        }
    });
}

/// View › Scale highlighting: the key, then the scale ("(none)" turns it off).
fn scale_menu(app: &mut LimeShotApp, ui: &mut Ui) {
    use limeshot_core::note_tools::Scale;
    let (root, cur) = app.ui.piano.scale.unwrap_or((0, Scale::None));
    ui.menu_button(format!("Key: {}", ROOTS[usize::from(root % 12)]), |ui| {
        for (r, name) in ROOTS.iter().enumerate() {
            if ui.add(Button::selectable(r as u8 == root, *name)).clicked() {
                let scale = if cur == Scale::None { Scale::Major } else { cur };
                app.ui.piano.scale = Some((r as u8, scale));
                ui.close();
            }
        }
    });
    ui.separator();
    for scale in Scale::ALL {
        if ui.add(Button::selectable(scale == cur, scale.label())).clicked() {
            app.ui.piano.scale = (scale != Scale::None).then_some((root, scale));
            ui.close();
        }
    }
}

/// The current channel's notes in the current pattern.
fn channel_notes(app: &LimeShotApp, pid: PatternId, cid: ChannelId) -> Vec<Note> {
    app.project()
        .pattern(pid)
        .map(|p| p.notes_for(cid).copied().collect())
        .unwrap_or_default()
}

/// FL Studio's tools act on the selection, or on every note of the channel when none is selected.
fn targets(app: &LimeShotApp, pid: PatternId, cid: ChannelId) -> Vec<Note> {
    let all = channel_notes(app, pid, cid);
    let sel = &app.ui.piano.selection;
    if sel.is_empty() {
        all
    } else {
        all.into_iter().filter(|n| sel.contains(&n.id)).collect()
    }
}

fn ids(notes: &[Note]) -> Vec<NoteId> {
    notes.iter().map(|n| n.id).collect()
}

/// Replace `before` with `after` as one undo step labelled by `tool`.
pub fn edit(app: &mut LimeShotApp, pid: PatternId, tool: NoteTool, before: &[Note], after: Vec<Note>) {
    if before.is_empty() && after.is_empty() {
        return;
    }
    app.exec(Command::EditNotes {
        pattern: pid,
        tool,
        remove: ids(before),
        notes: after,
    });
}

fn seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
}

pub fn run(app: &mut LimeShotApp, act: Act) {
    let pid = app.current_pattern();
    let Some(cid) = app.selected_channel() else { return };
    let sig = app.project().time_signature;
    let snap = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS);
    let sel = app.ui.piano.selection.clone();
    let notes = targets(app, pid, cid);
    let tids = ids(&notes);
    let mv = |app: &mut LimeShotApp, dt: i64, dkey: i32| {
        if !tids.is_empty() {
            app.exec(Command::MoveNotes {
                pattern: pid,
                notes: tids.clone(),
                dt,
                dkey,
            });
        }
    };
    match act {
        Act::Cut => crate::piano_roll::clipboard(app, ClipOp::Cut),
        Act::Copy => crate::piano_roll::clipboard(app, ClipOp::Copy),
        Act::Paste => crate::piano_roll::clipboard(app, ClipOp::Paste),
        Act::Duplicate if !sel.is_empty() => {
            let before: Vec<NoteId> = app
                .project()
                .pattern(pid)
                .map(|p| p.notes.iter().map(|n| n.id).collect())
                .unwrap_or_default();
            app.exec(Command::DuplicateNotes {
                pattern: pid,
                notes: sel,
            });
            // Select the copies.
            app.ui.piano.selection = app
                .project()
                .pattern(pid)
                .map(|p| p.notes.iter().map(|n| n.id).filter(|id| !before.contains(id)).collect())
                .unwrap_or_default();
        }
        Act::Delete if !sel.is_empty() => {
            app.exec(Command::RemoveNotes {
                pattern: pid,
                notes: sel,
            });
            app.ui.piano.selection.clear();
        }
        Act::Duplicate | Act::Delete => {}
        Act::ShiftLeft => mv(app, -(snap as i64), 0),
        Act::ShiftRight => mv(app, snap as i64, 0),
        Act::TransposeUp => mv(app, 0, 1),
        Act::TransposeDown => mv(app, 0, -1),
        Act::OctaveUp => mv(app, 0, 12),
        Act::OctaveDown => mv(app, 0, -12),
        Act::RotateLeft | Act::RotateRight if !tids.is_empty() => {
            let by = if act == Act::RotateLeft {
                -(snap as i64)
            } else {
                snap as i64
            };
            app.exec(Command::RotateNotes {
                pattern: pid,
                notes: tids,
                by,
            });
        }
        Act::RotateLeft | Act::RotateRight => {}
        // FL Studio's short "discarded" length: one step.
        Act::DiscardLengths => edit(
            app,
            pid,
            NoteTool::DiscardLengths,
            &notes,
            note_tools::discard_lengths(&notes, STEP_TICKS),
        ),
        Act::Mute | Act::Unmute if !tids.is_empty() => {
            app.exec(Command::SetNotesMuted {
                pattern: pid,
                notes: tids,
                muted: act == Act::Mute,
            });
        }
        Act::Mute | Act::Unmute => {}
        Act::QuickLegato => edit(app, pid, NoteTool::Legato, &notes, note_tools::legato(&notes)),
        Act::QuickQuantize | Act::QuickQuantizeStarts if !tids.is_empty() => {
            app.exec(Command::Quantize {
                pattern: pid,
                notes: tids,
                grid: snap,
                strength: 1.0,
                lengths: act == Act::QuickQuantize,
            });
        }
        Act::QuickQuantize | Act::QuickQuantizeStarts => {}
        Act::QuickChop => chop(app, pid, &tids, snap),
        Act::Glue => {
            app.exec(Command::GlueNotes {
                pattern: pid,
                notes: tids,
            });
            let left: Vec<NoteId> = ids(&channel_notes(app, pid, cid));
            app.ui.piano.selection.retain(|id| left.contains(id));
        }
        Act::Deselect => app.ui.piano.selection.clear(),
        Act::SelectAll => app.ui.piano.selection = ids(&channel_notes(app, pid, cid)),
        Act::SelectRandom | Act::SelectMoreRandom => {
            let keep = if act == Act::SelectMoreRandom { sel } else { Vec::new() };
            let pool: Vec<NoteId> = ids(&channel_notes(app, pid, cid))
                .into_iter()
                .filter(|id| !keep.contains(id))
                .collect();
            let mut rng = Rng::new(seed());
            if let Some(&pick) = pool.get(rng.below(pool.len())) {
                app.ui.piano.selection = keep;
                app.ui.piano.selection.push(pick);
            }
        }
        Act::SelectOdd => app.ui.piano.selection = note_tools::odd(&notes),
        Act::SelectMuted => {
            app.ui.piano.selection = channel_notes(app, pid, cid)
                .iter()
                .filter(|n| n.muted)
                .map(|n| n.id)
                .collect()
        }
        Act::SelectOverlapping => app.ui.piano.selection = note_tools::overlapping(&channel_notes(app, pid, cid)),
        Act::SelectStacked => app.ui.piano.selection = note_tools::stacked(&channel_notes(app, pid, cid)),
        Act::InvertSelection => {
            app.ui.piano.selection = ids(&channel_notes(app, pid, cid))
                .into_iter()
                .filter(|id| !sel.contains(id))
                .collect()
        }
        Act::ZoomIn => app.ui.piano.zoom_x = (app.ui.piano.zoom_x * 1.25).min(MAX_ZOOM),
        Act::ZoomOut => app.ui.piano.zoom_x = (app.ui.piano.zoom_x / 1.25).max(MIN_ZOOM),
        Act::QuickZoom(n) => quick_zoom(app, pid, cid, n),
        Act::Center => center(app, pid, cid),
        Act::Dialog(kind) => crate::piano_tools::open(app, kind),
    }
}

pub const MIN_ZOOM: f32 = 12.0;
pub const MAX_ZOOM: f32 = 800.0;

/// Fit the ticks `from..to` into the grid's width.
fn fit(app: &mut LimeShotApp, from: Tick, to: Tick) {
    let Some(g) = app.ui.piano.geometry else { return };
    let beats = (to.saturating_sub(from)).max(1) as f32 / PPQ as f32;
    let st = &mut app.ui.piano;
    st.zoom_x = (g.grid.width() / beats).clamp(MIN_ZOOM, MAX_ZOOM);
    st.scroll.x = from as f32 / PPQ as f32 * st.zoom_x;
}

/// FL Studio's quick zooms (Shift+1 … 6). The levels are LimeShot's: 1, 4 and 16 bars from the
/// view's left edge, the whole pattern (Zoom out far), the selection, and Channel Rack step size.
fn quick_zoom(app: &mut LimeShotApp, pid: PatternId, cid: ChannelId, n: u8) {
    let sig = app.project().time_signature;
    let bar = sig.ticks_per_bar();
    let left = {
        let st = &app.ui.piano;
        let t = (st.scroll.x / st.zoom_x * PPQ as f32).max(0.0) as Tick;
        t / bar * bar
    };
    match n {
        1 => fit(app, left, left + bar),
        2 => fit(app, left, left + 4 * bar),
        3 => fit(app, left, left + 16 * bar),
        4 => {
            let len = app.project().pattern(pid).map(|p| p.length(sig)).unwrap_or(bar);
            fit(app, 0, len.max(bar));
        }
        5 => {
            let notes = targets(app, pid, cid);
            if let (Some(a), Some(b)) = (notes.iter().map(|n| n.start).min(), notes.iter().map(Note::end).max()) {
                fit(app, a, b);
            }
        }
        _ => {
            // About a Channel Rack step button per step.
            let st = &mut app.ui.piano;
            let left_beats = st.scroll.x / st.zoom_x;
            st.zoom_x = 24.0 * (PPQ / STEP_TICKS) as f32;
            st.scroll.x = left_beats * st.zoom_x;
        }
    }
}

/// Center (Shift+0): bring the playback position (or the start) and the notes into view.
fn center(app: &mut LimeShotApp, pid: PatternId, cid: ChannelId) {
    let Some(g) = app.ui.piano.geometry else { return };
    let pos = if app.is_playing() {
        app.player.status().position()
    } else {
        0
    };
    let notes = channel_notes(app, pid, cid);
    let st = &mut app.ui.piano;
    st.scroll.x = (pos as f32 / PPQ as f32 * st.zoom_x - g.grid.width() / 2.0).max(0.0);
    if !notes.is_empty() {
        let mid = notes.iter().map(|n| f32::from(n.key)).sum::<f32>() / notes.len() as f32;
        st.scroll.y = ((127.0 - mid) * st.key_h - g.grid.height() / 2.0).max(0.0);
    }
}

/// Shift+digit, by the key's position (on QWERTZ and other layouts Shift+1 types another
/// character, so the logical key alone would miss it).
pub(crate) fn shift_digit(ctx: &Context, digit: Key) -> bool {
    ctx.input_mut(|i| {
        let pos = i.events.iter().position(|e| {
            matches!(e, Event::Key { key, physical_key, pressed: true, modifiers, .. }
                if modifiers.shift && !modifiers.command && !modifiers.alt
                    && (*key == digit || *physical_key == Some(digit)))
        });
        pos.map(|p| i.events.remove(p)).is_some()
    })
}

/// The menu's keys (checked before the editor's single-letter tool keys, which egui would also
/// match with Shift held).
pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key as k;
    let shift = Modifiers::SHIFT;
    for (key, kind) in DIALOG_KEYS {
        if k(ctx, Modifiers::ALT, key) {
            run(app, Act::Dialog(kind));
        }
    }
    for (mods, key, act) in [
        (shift, Key::D, Act::DiscardLengths),
        (Modifiers::COMMAND, Key::L, Act::QuickLegato),
        (shift, Key::Q, Act::QuickQuantizeStarts),
        (shift, Key::R, Act::SelectRandom),
        (shift, Key::M, Act::SelectMoreRandom),
        (shift, Key::O, Act::SelectOdd),
        (Modifiers::NONE, Key::PageUp, Act::ZoomIn),
        (Modifiers::NONE, Key::PageDown, Act::ZoomOut),
    ] {
        if k(ctx, mods, key) {
            run(app, act);
        }
    }
    for (n, digit) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6]
        .into_iter()
        .enumerate()
    {
        if shift_digit(ctx, digit) {
            run(app, Act::QuickZoom(n as u8 + 1));
        }
    }
    if shift_digit(ctx, Key::Num0) {
        run(app, Act::Center);
    }
}
