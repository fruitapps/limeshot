//! Note recording (FL Studio's Record button, R): while armed and playing, notes played on the
//! MIDI input or the typing keyboard become notes of the current pattern on the channel they
//! played. The record button's right-click menu has FL's options (recording starts playback,
//! disarm on stop, quantize start and end times). A take is one undo step: each recorded note
//! replaces the take's last step with one holding all its notes, unless something else was
//! edited in between.

use crate::app::LimeShotApp;
use limeshot_core::*;
use limeshot_engine::PlayMode;

/// The record button's state and options.
#[derive(Debug, Clone)]
pub struct State {
    pub armed: bool,
    /// Arming while stopped starts playback (FL Studio: Recording starts playback).
    pub starts_playback: bool,
    /// Stopping disarms (FL Studio: Disarm on stop).
    pub disarm_on_stop: bool,
    /// Snap recorded note starts / ends to the snap setting (FL Studio's Quantize submenu).
    pub quantize_start: bool,
    pub quantize_end: bool,
    /// Notes being held: channel, key, start (pattern ticks), velocity.
    held: Vec<(ChannelId, u8, Tick, f32)>,
    /// The current take: its pattern, notes, and the revision after its last step.
    take: Vec<NewNote>,
    take_pattern: Option<PatternId>,
    applied: Option<u64>,
}

impl Default for State {
    fn default() -> Self {
        State {
            armed: false,
            starts_playback: true,
            disarm_on_stop: false,
            quantize_start: true,
            quantize_end: false,
            held: Vec::new(),
            take: Vec::new(),
            take_pattern: None,
            applied: None,
        }
    }
}

/// The Record button (R).
pub fn toggle(app: &mut LimeShotApp) {
    let st = &mut app.ui.record;
    st.armed = !st.armed;
    st.take.clear();
    st.applied = None;
    if st.armed && st.starts_playback && !app.is_playing() {
        app.toggle_play();
    }
    if !app.ui.record.armed {
        finish_held(app);
    }
}

/// Where the transport is in the current pattern: the pattern position in Pattern mode; in
/// Song mode, inside the current pattern's clip under the song position (or the song position
/// itself when none is there).
fn position(app: &LimeShotApp) -> Tick {
    let pos = app.player.status().position();
    if app.ui.mode == PlayMode::Pattern {
        return pos;
    }
    let pid = app.current_pattern();
    app.project()
        .playlist
        .clips
        .iter()
        .find(|c| c.pattern() == Some(pid) && c.start <= pos && pos < c.end())
        .map_or(pos, |c| pos - c.start + c.offset)
}

fn recording(app: &LimeShotApp) -> bool {
    app.ui.record.armed && app.is_playing()
}

/// A note started on `channel` (MIDI input or typing keyboard).
pub fn note_on(app: &mut LimeShotApp, channel: ChannelId, key: u8, velocity: f32) {
    if !recording(app) {
        return;
    }
    let at = position(app);
    let held = &mut app.ui.record.held;
    held.retain(|h| (h.0, h.1) != (channel, key));
    held.push((channel, key, at, velocity));
}

/// A note ended: if it was recorded, it becomes a note of the take.
pub fn note_off(app: &mut LimeShotApp, channel: ChannelId, key: u8) {
    let Some(i) = app.ui.record.held.iter().position(|h| (h.0, h.1) == (channel, key)) else {
        return;
    };
    let (_, _, start, velocity) = app.ui.record.held.remove(i);
    let end = position(app);
    add(app, channel, key, start, end, velocity);
}

/// End every held note (stop, disarm).
fn finish_held(app: &mut LimeShotApp) {
    let end = position(app);
    for (channel, key, start, velocity) in std::mem::take(&mut app.ui.record.held) {
        add(app, channel, key, start, end, velocity);
    }
}

/// Called when the transport stops.
pub fn stopped(app: &mut LimeShotApp) {
    finish_held(app);
    let st = &mut app.ui.record;
    st.take.clear();
    st.applied = None;
    if st.disarm_on_stop {
        st.armed = false;
    }
}

fn add(app: &mut LimeShotApp, channel: ChannelId, key: u8, start: Tick, end: Tick, velocity: f32) {
    let pid = app.current_pattern();
    let sig = app.project().time_signature;
    let pat_len = app
        .project()
        .pattern(pid)
        .map_or(sig.ticks_per_bar(), |p| p.length(sig));
    // A note held across the pattern's loop point wraps round it.
    let mut length = if end >= start {
        end - start
    } else {
        pat_len.saturating_sub(start) + end
    };
    let grid = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS).max(1);
    let st = &app.ui.record;
    let mut start = start;
    if st.quantize_start {
        start = app.ui.snap.round(start, sig);
        // Rounded up onto the loop point: the pattern's start.
        if app.ui.mode == PlayMode::Pattern && start >= pat_len {
            start = 0;
        }
    }
    if st.quantize_end {
        length = ((length + grid / 2) / grid * grid).max(grid);
    }
    let note = NewNote {
        channel,
        key,
        start,
        length: length.max(STEP_TICKS / 4),
        velocity,
    };
    // Replace the take's last step, unless another edit came in between: then start a new one.
    if app.ui.record.take_pattern != Some(pid) || app.ui.record.applied != Some(app.doc.revision()) {
        app.ui.record.take.clear();
    } else {
        app.doc.undo();
    }
    app.ui.record.take.push(note);
    app.ui.record.take_pattern = Some(pid);
    let notes = app
        .ui
        .record
        .take
        .iter()
        .map(|n| Note {
            id: NoteId(0),
            channel: n.channel,
            key: n.key,
            start: n.start,
            length: n.length,
            velocity: n.velocity,
            muted: false,
            pan: 0.0,
        })
        .collect();
    let before = app.doc.revision();
    app.exec(Command::EditNotes {
        pattern: pid,
        tool: NoteTool::Record,
        remove: Vec::new(),
        notes,
    });
    app.ui.record.applied = (app.doc.revision() != before).then(|| app.doc.revision());
}
