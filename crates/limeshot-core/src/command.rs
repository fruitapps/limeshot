//! Typed, serialisable project mutations.
//!
//! Every change to a [`Project`] goes through a [`Command`], whether it comes from the UI, a
//! keyboard shortcut, the CLI or a test. [`crate::Document`] wraps commands with undo/redo.

use crate::model::*;
use crate::time::{STEP_TICKS, Tempo, Tick, TimeSignature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    #[error("no channel {0:?}")]
    NoChannel(ChannelId),
    #[error("no pattern {0:?}")]
    NoPattern(PatternId),
    #[error("no clip {0:?}")]
    NoClip(ClipId),
    #[error("no mixer track {0}")]
    NoMixerTrack(u16),
    #[error("no playlist track {0}")]
    NoPlaylistTrack(u32),
    #[error("no effect {0}")]
    NoEffect(u32),
    #[error("{0}")]
    Invalid(String),
}

/// What a command created, for callers that need the new id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Created(pub Option<u32>);

/// A note to add, before it has an id.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NewNote {
    pub channel: ChannelId,
    pub key: u8,
    pub start: Tick,
    pub length: Tick,
    pub velocity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    SetTitle(String),
    SetTempo(Tempo),
    SetTimeSignature(TimeSignature),
    /// Global swing, 0–1.
    SetSwing(f32),

    AddChannel {
        name: String,
        instrument: Instrument,
    },
    RemoveChannel(ChannelId),
    CloneChannel(ChannelId),
    RenameChannel {
        channel: ChannelId,
        name: String,
    },
    SetChannelColor {
        channel: ChannelId,
        color: Rgb,
    },
    MoveChannel {
        channel: ChannelId,
        delta: i32,
    },
    SetChannelVolume {
        channel: ChannelId,
        volume: f32,
    },
    SetChannelPan {
        channel: ChannelId,
        pan: f32,
    },
    SetChannelMute {
        channel: ChannelId,
        muted: bool,
    },
    SetChannelSolo {
        channel: ChannelId,
        solo: bool,
    },
    SetChannelMixerTrack {
        channel: ChannelId,
        track: u16,
    },
    SetChannelRootKey {
        channel: ChannelId,
        key: u8,
    },
    SetInstrument {
        channel: ChannelId,
        instrument: Instrument,
    },

    SetStep {
        pattern: PatternId,
        channel: ChannelId,
        step: u32,
        on: bool,
    },
    AddNote {
        pattern: PatternId,
        note: NewNote,
    },
    AddNotes {
        pattern: PatternId,
        notes: Vec<NewNote>,
    },
    RemoveNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
    },
    /// Move notes by a time and pitch offset (clamped at 0 and to the MIDI range).
    MoveNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
        dt: i64,
        dkey: i32,
    },
    /// Change note lengths by `dlen` ticks (minimum one tick).
    ResizeNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
        dlen: i64,
    },
    SetNoteVelocity {
        pattern: PatternId,
        notes: Vec<NoteId>,
        velocity: f32,
    },
    /// Copy the notes right after the selection's span.
    DuplicateNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
    },
    ClearChannelNotes {
        pattern: PatternId,
        channel: ChannelId,
    },
    /// Move note starts towards the nearest multiple of `grid` ticks by `strength` (0–1); with
    /// `lengths`, also round note lengths to the grid (minimum one grid unit).
    Quantize {
        pattern: PatternId,
        notes: Vec<NoteId>,
        grid: Tick,
        strength: f32,
        lengths: bool,
    },

    AddPattern {
        name: String,
    },
    ClonePattern(PatternId),
    RemovePattern(PatternId),
    RenamePattern {
        pattern: PatternId,
        name: String,
    },
    SetPatternLength {
        pattern: PatternId,
        steps: Option<u32>,
    },

    AddClip {
        pattern: PatternId,
        track: u32,
        start: Tick,
        length: Option<Tick>,
    },
    RemoveClips(Vec<ClipId>),
    MoveClips {
        clips: Vec<ClipId>,
        dt: i64,
        dtrack: i32,
    },
    ResizeClip {
        clip: ClipId,
        length: Tick,
    },
    DuplicateClips(Vec<ClipId>),
    /// Give the clip its own copy of its pattern.
    MakeClipUnique(ClipId),
    SetClipMuted {
        clip: ClipId,
        muted: bool,
    },
    AddPlaylistTrack,
    RenamePlaylistTrack {
        track: u32,
        name: String,
    },
    SetPlaylistTrackMute {
        track: u32,
        muted: bool,
    },

    AddMixerTrack,
    RenameMixerTrack {
        track: u16,
        name: String,
    },
    SetMixerVolume {
        track: u16,
        volume: f32,
    },
    SetMixerPan {
        track: u16,
        pan: f32,
    },
    SetMixerMute {
        track: u16,
        muted: bool,
    },
    SetMixerSolo {
        track: u16,
        solo: bool,
    },
    /// Route mixer track `from` to `to` at `level` (linear), update an existing route's level,
    /// or (with `None`) remove the route. Routes that would create a feedback loop are refused.
    SetRoute {
        from: u16,
        to: u16,
        level: Option<f32>,
    },
    /// Append an effect to a mixer track's insert chain (returns the effect id).
    AddEffect {
        track: u16,
        kind: crate::effects::EffectKind,
    },
    RemoveEffect {
        track: u16,
        effect: u32,
    },
    MoveEffect {
        track: u16,
        effect: u32,
        delta: i32,
    },
    SetEffectEnabled {
        track: u16,
        effect: u32,
        enabled: bool,
    },
    SetEffectParam {
        track: u16,
        effect: u32,
        index: usize,
        value: f32,
    },

    /// Several commands applied as one undo step. Stops at the first error.
    Batch(Vec<Command>),
}

type R = Result<Created, CommandError>;

fn ok() -> R {
    Ok(Created(None))
}

fn finite(v: f32) -> Result<f32, CommandError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(CommandError::Invalid("value is not a number".into()))
    }
}

fn channel(p: &mut Project, id: ChannelId) -> Result<&mut Channel, CommandError> {
    p.channel_mut(id).ok_or(CommandError::NoChannel(id))
}

fn pattern(p: &mut Project, id: PatternId) -> Result<&mut Pattern, CommandError> {
    p.pattern_mut(id).ok_or(CommandError::NoPattern(id))
}

fn mixer_track(p: &mut Project, i: u16) -> Result<&mut MixerTrack, CommandError> {
    p.mixer
        .tracks
        .get_mut(usize::from(i))
        .ok_or(CommandError::NoMixerTrack(i))
}

fn offset(t: Tick, d: i64) -> Tick {
    if d >= 0 {
        t.saturating_add(d as u64)
    } else {
        t.saturating_sub(d.unsigned_abs())
    }
}

fn sort_notes(p: &mut Pattern) {
    p.notes.sort_by_key(|n| (n.start, n.key, n.id));
}

impl Command {
    /// A short human-readable name for the undo history.
    pub fn label(&self) -> &'static str {
        use Command::*;
        match self {
            SetTitle(_) => "Rename project",
            SetTempo(_) => "Set tempo",
            SetTimeSignature(_) => "Set time signature",
            SetSwing(_) => "Swing",
            AddChannel { .. } => "Add channel",
            RemoveChannel(_) => "Delete channel",
            CloneChannel(_) => "Clone channel",
            RenameChannel { .. } => "Rename channel",
            SetChannelColor { .. } => "Recolor channel",
            MoveChannel { .. } => "Move channel",
            SetChannelVolume { .. } => "Channel volume",
            SetChannelPan { .. } => "Channel pan",
            SetChannelMute { .. } => "Mute channel",
            SetChannelSolo { .. } => "Solo channel",
            SetChannelMixerTrack { .. } => "Route channel",
            SetChannelRootKey { .. } => "Set root key",
            SetInstrument { .. } => "Edit instrument",
            SetStep { .. } => "Toggle step",
            AddNote { .. } | AddNotes { .. } => "Add note",
            RemoveNotes { .. } => "Delete notes",
            MoveNotes { .. } => "Move notes",
            ResizeNotes { .. } => "Resize notes",
            SetNoteVelocity { .. } => "Note velocity",
            DuplicateNotes { .. } => "Duplicate notes",
            ClearChannelNotes { .. } => "Clear channel",
            Quantize { .. } => "Quantize",
            AddPattern { .. } => "Add pattern",
            ClonePattern(_) => "Clone pattern",
            RemovePattern(_) => "Delete pattern",
            RenamePattern { .. } => "Rename pattern",
            SetPatternLength { .. } => "Pattern length",
            AddClip { .. } => "Add clip",
            RemoveClips(_) => "Delete clips",
            MoveClips { .. } => "Move clips",
            ResizeClip { .. } => "Resize clip",
            DuplicateClips(_) => "Duplicate clips",
            MakeClipUnique(_) => "Make unique",
            SetClipMuted { .. } => "Mute clip",
            AddPlaylistTrack => "Add playlist track",
            RenamePlaylistTrack { .. } => "Rename track",
            SetPlaylistTrackMute { .. } => "Mute track",
            AddMixerTrack => "Add mixer track",
            RenameMixerTrack { .. } => "Rename mixer track",
            SetMixerVolume { .. } => "Mixer volume",
            SetMixerPan { .. } => "Mixer pan",
            SetMixerMute { .. } => "Mute mixer track",
            SetMixerSolo { .. } => "Solo mixer track",
            SetRoute { level: None, .. } => "Remove route",
            SetRoute { .. } => "Route mixer track",
            AddEffect { .. } => "Add effect",
            RemoveEffect { .. } => "Remove effect",
            MoveEffect { .. } => "Move effect",
            SetEffectEnabled { .. } => "Bypass effect",
            SetEffectParam { .. } => "Effect parameter",
            Batch(_) => "Edit",
        }
    }

    /// Commands with the same merge key, issued during one gesture, collapse into one undo step
    /// (a knob drag sends many `SetChannelVolume`s).
    pub fn merge_key(&self) -> Option<String> {
        use Command::*;
        Some(match self {
            SetTempo(_) => "tempo".into(),
            SetRoute {
                from,
                to,
                level: Some(_),
            } => format!("route{from}:{to}"),
            SetEffectParam {
                track, effect, index, ..
            } => format!("fx{track}:{effect}:{index}"),
            SetSwing(_) => "swing".into(),
            SetChannelVolume { channel, .. } => format!("chvol{}", channel.0),
            SetChannelPan { channel, .. } => format!("chpan{}", channel.0),
            SetChannelMixerTrack { channel, .. } => format!("chmix{}", channel.0),
            SetInstrument { channel, .. } => format!("inst{}", channel.0),
            SetMixerVolume { track, .. } => format!("mxvol{track}"),
            SetMixerPan { track, .. } => format!("mxpan{track}"),
            MoveNotes { pattern, notes, .. } => format!("mvn{}:{notes:?}", pattern.0),
            ResizeNotes { pattern, notes, .. } => format!("rsn{}:{notes:?}", pattern.0),
            SetNoteVelocity { pattern, notes, .. } => format!("vel{}:{notes:?}", pattern.0),
            MoveClips { clips, .. } => format!("mvc{clips:?}"),
            ResizeClip { clip, .. } => format!("rsc{}", clip.0),
            SetStep {
                pattern, channel, on, ..
            } => format!("step{}:{}:{on}", pattern.0, channel.0),
            _ => return None,
        })
    }

    /// Apply to `p`. On error `p` may be partially modified; [`crate::Document`] applies to a copy.
    pub fn apply(self, p: &mut Project) -> R {
        use Command::*;
        match self {
            SetTitle(t) => p.title = t,
            SetTempo(t) => p.tempo = t,
            SetTimeSignature(s) => p.time_signature = TimeSignature::new(s.numerator, s.denominator),
            SetSwing(v) => p.swing = finite(v)?.clamp(0.0, 1.0),

            AddChannel { name, instrument } => {
                let id = p.alloc_id();
                let color = PALETTE[p.channels.len() % PALETTE.len()];
                let mixer_track = p.free_mixer_track().unwrap_or(0);
                p.channels.push(Channel {
                    id: ChannelId(id),
                    name,
                    color,
                    instrument,
                    volume: 0.78,
                    pan: 0.0,
                    muted: false,
                    solo: false,
                    mixer_track,
                    root_key: 60,
                    extra: Extra::new(),
                });
                return Ok(Created(Some(id)));
            }
            RemoveChannel(id) => {
                let i = p.channel_index(id).ok_or(CommandError::NoChannel(id))?;
                p.channels.remove(i);
                for pat in &mut p.patterns {
                    pat.notes.retain(|n| n.channel != id);
                }
            }
            CloneChannel(id) => {
                let i = p.channel_index(id).ok_or(CommandError::NoChannel(id))?;
                let new_id = ChannelId(p.alloc_id());
                let mut c = p.channels[i].clone();
                c.id = new_id;
                c.name = format!("{} #2", c.name);
                p.channels.insert(i + 1, c);
                for pi in 0..p.patterns.len() {
                    let copies: Vec<Note> = p.patterns[pi]
                        .notes
                        .iter()
                        .filter(|n| n.channel == id)
                        .copied()
                        .collect();
                    for mut n in copies {
                        n.id = NoteId(p.alloc_id());
                        n.channel = new_id;
                        p.patterns[pi].notes.push(n);
                    }
                    sort_notes(&mut p.patterns[pi]);
                }
                return Ok(Created(Some(new_id.0)));
            }
            RenameChannel { channel: id, name } => channel(p, id)?.name = name,
            SetChannelColor { channel: id, color } => channel(p, id)?.color = color,
            MoveChannel { channel: id, delta } => {
                let i = p.channel_index(id).ok_or(CommandError::NoChannel(id))?;
                let j = (i as i64 + i64::from(delta)).clamp(0, p.channels.len() as i64 - 1) as usize;
                let c = p.channels.remove(i);
                p.channels.insert(j, c);
            }
            SetChannelVolume { channel: id, volume } => channel(p, id)?.volume = finite(volume)?.clamp(0.0, 1.25),
            SetChannelPan { channel: id, pan } => channel(p, id)?.pan = finite(pan)?.clamp(-1.0, 1.0),
            SetChannelMute { channel: id, muted } => channel(p, id)?.muted = muted,
            SetChannelSolo { channel: id, solo } => channel(p, id)?.solo = solo,
            SetChannelMixerTrack { channel: id, track } => {
                if usize::from(track) >= p.mixer.tracks.len() {
                    return Err(CommandError::NoMixerTrack(track));
                }
                channel(p, id)?.mixer_track = track;
            }
            SetChannelRootKey { channel: id, key } => channel(p, id)?.root_key = key.min(127),
            SetInstrument {
                channel: id,
                instrument,
            } => channel(p, id)?.instrument = instrument,

            SetStep {
                pattern: pid,
                channel: cid,
                step,
                on,
            } => {
                if step >= 512 {
                    return Err(CommandError::Invalid(format!("step {step} is past the 512-step limit")));
                }
                let root = p.channel(cid).ok_or(CommandError::NoChannel(cid))?.root_key;
                let note_id = p.alloc_id();
                let pat = pattern(p, pid)?;
                let t = Tick::from(step) * STEP_TICKS;
                let existing = pat
                    .notes
                    .iter()
                    .position(|n| n.channel == cid && n.start == t && n.key == root);
                match (existing, on) {
                    (None, true) => {
                        pat.notes.push(Note {
                            id: NoteId(note_id),
                            channel: cid,
                            key: root,
                            start: t,
                            length: STEP_TICKS,
                            velocity: 0.78,
                        });
                        sort_notes(pat);
                    }
                    (Some(i), false) => {
                        pat.notes.remove(i);
                    }
                    _ => {}
                }
            }
            AddNote { pattern: pid, note } => return add_notes(p, pid, vec![note]),
            AddNotes { pattern: pid, notes } => return add_notes(p, pid, notes),
            RemoveNotes { pattern: pid, notes } => pattern(p, pid)?.notes.retain(|n| !notes.contains(&n.id)),
            MoveNotes {
                pattern: pid,
                notes,
                dt,
                dkey,
            } => {
                let pat = pattern(p, pid)?;
                for n in pat.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    n.start = offset(n.start, dt);
                    n.key = (i32::from(n.key) + dkey).clamp(0, 127) as u8;
                }
                sort_notes(pat);
            }
            ResizeNotes {
                pattern: pid,
                notes,
                dlen,
            } => {
                for n in pattern(p, pid)?.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    n.length = offset(n.length, dlen).max(1);
                }
            }
            SetNoteVelocity {
                pattern: pid,
                notes,
                velocity,
            } => {
                let v = finite(velocity)?.clamp(0.0, 1.0);
                for n in pattern(p, pid)?.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    n.velocity = v;
                }
            }
            DuplicateNotes { pattern: pid, notes } => {
                let sig = p.time_signature;
                let pat = pattern(p, pid)?;
                let sel: Vec<Note> = pat.notes.iter().filter(|n| notes.contains(&n.id)).copied().collect();
                let (Some(lo), Some(hi)) = (sel.iter().map(|n| n.start).min(), sel.iter().map(Note::end).max()) else {
                    return ok();
                };
                // Repeat on whole beats so the copy stays on the grid.
                let beat = sig.ticks_per_beat();
                let span = (hi - lo).div_ceil(beat).max(1) * beat;
                let new: Vec<NewNote> = sel
                    .iter()
                    .map(|n| NewNote {
                        channel: n.channel,
                        key: n.key,
                        start: n.start + span,
                        length: n.length,
                        velocity: n.velocity,
                    })
                    .collect();
                return add_notes(p, pid, new);
            }
            ClearChannelNotes {
                pattern: pid,
                channel: cid,
            } => pattern(p, pid)?.notes.retain(|n| n.channel != cid),
            Quantize {
                pattern: pid,
                notes,
                grid,
                strength,
                lengths,
            } => {
                if grid == 0 {
                    return Err(CommandError::Invalid("quantize grid must be at least one tick".into()));
                }
                let k = f64::from(finite(strength)?.clamp(0.0, 1.0));
                let pat = pattern(p, pid)?;
                for n in pat.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    let target = (n.start + grid / 2) / grid * grid;
                    let moved = n.start as f64 + (target as f64 - n.start as f64) * k;
                    n.start = moved.round().max(0.0) as Tick;
                    if lengths {
                        n.length = ((n.length + grid / 2) / grid * grid).max(grid);
                    }
                }
                sort_notes(pat);
            }

            AddPattern { name } => {
                let id = p.alloc_id();
                let color = PALETTE[p.patterns.len() % PALETTE.len()];
                p.patterns.push(Pattern {
                    id: PatternId(id),
                    name,
                    color,
                    length_steps: None,
                    notes: Vec::new(),
                    extra: Extra::new(),
                });
                return Ok(Created(Some(id)));
            }
            ClonePattern(id) => {
                let i = p
                    .patterns
                    .iter()
                    .position(|x| x.id == id)
                    .ok_or(CommandError::NoPattern(id))?;
                let new = clone_pattern(p, i);
                return Ok(Created(Some(new.0)));
            }
            RemovePattern(id) => {
                if p.patterns.len() <= 1 {
                    return Err(CommandError::Invalid("a project keeps at least one pattern".into()));
                }
                let i = p
                    .patterns
                    .iter()
                    .position(|x| x.id == id)
                    .ok_or(CommandError::NoPattern(id))?;
                p.patterns.remove(i);
                p.playlist.clips.retain(|c| c.pattern() != id);
            }
            RenamePattern { pattern: id, name } => pattern(p, id)?.name = name,
            SetPatternLength { pattern: id, steps } => pattern(p, id)?.length_steps = steps.map(|s| s.clamp(1, 512)),

            AddClip {
                pattern: pid,
                track,
                start,
                length,
            } => {
                let sig = p.time_signature;
                let len = length
                    .unwrap_or_else(|| p.pattern(pid).map(|x| x.length(sig)).unwrap_or(0))
                    .max(1);
                if p.pattern(pid).is_none() {
                    return Err(CommandError::NoPattern(pid));
                }
                if track as usize >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                let id = p.alloc_id();
                p.playlist.clips.push(Clip {
                    id: ClipId(id),
                    source: ClipSource::Pattern(pid),
                    track,
                    start,
                    length: len,
                    offset: 0,
                    muted: false,
                    extra: Extra::new(),
                });
                return Ok(Created(Some(id)));
            }
            RemoveClips(ids) => p.playlist.clips.retain(|c| !ids.contains(&c.id)),
            MoveClips { clips, dt, dtrack } => {
                let max_track = p.playlist.tracks.len().saturating_sub(1) as i64;
                for c in p.playlist.clips.iter_mut().filter(|c| clips.contains(&c.id)) {
                    c.start = offset(c.start, dt);
                    c.track = (i64::from(c.track) + i64::from(dtrack)).clamp(0, max_track) as u32;
                }
            }
            ResizeClip { clip, length } => p.clip_mut(clip).ok_or(CommandError::NoClip(clip))?.length = length.max(1),
            DuplicateClips(ids) => {
                let sel: Vec<Clip> = p
                    .playlist
                    .clips
                    .iter()
                    .filter(|c| ids.contains(&c.id))
                    .cloned()
                    .collect();
                let (Some(lo), Some(hi)) = (sel.iter().map(|c| c.start).min(), sel.iter().map(Clip::end).max()) else {
                    return ok();
                };
                let span = hi - lo;
                for mut c in sel {
                    c.id = ClipId(p.alloc_id());
                    c.start += span;
                    p.playlist.clips.push(c);
                }
            }
            MakeClipUnique(id) => {
                let pid = p.clip(id).ok_or(CommandError::NoClip(id))?.pattern();
                let i = p
                    .patterns
                    .iter()
                    .position(|x| x.id == pid)
                    .ok_or(CommandError::NoPattern(pid))?;
                let new = clone_pattern(p, i);
                if let Some(c) = p.clip_mut(id) {
                    c.source = ClipSource::Pattern(new);
                }
                return Ok(Created(Some(new.0)));
            }
            SetClipMuted { clip, muted } => p.clip_mut(clip).ok_or(CommandError::NoClip(clip))?.muted = muted,
            AddPlaylistTrack => {
                if p.playlist.tracks.len() >= 500 {
                    return Err(CommandError::Invalid("the playlist holds at most 500 tracks".into()));
                }
                let n = p.playlist.tracks.len() + 1;
                p.playlist.tracks.push(PlaylistTrack {
                    name: format!("Track {n}"),
                    muted: false,
                });
            }
            RenamePlaylistTrack { track, name } => {
                p.playlist
                    .tracks
                    .get_mut(track as usize)
                    .ok_or(CommandError::NoPlaylistTrack(track))?
                    .name = name
            }
            SetPlaylistTrackMute { track, muted } => {
                p.playlist
                    .tracks
                    .get_mut(track as usize)
                    .ok_or(CommandError::NoPlaylistTrack(track))?
                    .muted = muted
            }

            AddMixerTrack => {
                if p.mixer.tracks.len() >= Mixer::MAX_TRACKS {
                    return Err(CommandError::Invalid(format!(
                        "the mixer holds at most {} tracks",
                        Mixer::MAX_TRACKS
                    )));
                }
                let n = p.mixer.tracks.len();
                p.mixer.tracks.push(MixerTrack::new(format!("Insert {n}")));
                return Ok(Created(Some(n as u32)));
            }
            RenameMixerTrack { track, name } => mixer_track(p, track)?.name = name,
            SetMixerVolume { track, volume } => mixer_track(p, track)?.volume = finite(volume)?.clamp(0.0, 2.0),
            SetMixerPan { track, pan } => mixer_track(p, track)?.pan = finite(pan)?.clamp(-1.0, 1.0),
            SetMixerMute { track, muted } => mixer_track(p, track)?.muted = muted,
            SetMixerSolo { track, solo } => mixer_track(p, track)?.solo = solo,
            SetRoute { from, to, level } => {
                if usize::from(to) >= p.mixer.tracks.len() {
                    return Err(CommandError::NoMixerTrack(to));
                }
                mixer_track(p, from)?;
                match level {
                    None => mixer_track(p, from)?.routes.retain(|r| r.to != to),
                    Some(level) => {
                        let level = finite(level)?.clamp(0.0, 2.0);
                        if let Some(r) = mixer_track(p, from)?.routes.iter_mut().find(|r| r.to == to) {
                            r.level = level;
                        } else {
                            if p.mixer.would_cycle(from, to) {
                                return Err(CommandError::Invalid(format!(
                                    "routing mixer track {from} to {to} would create a feedback loop"
                                )));
                            }
                            mixer_track(p, from)?.routes.push(Route { to, level });
                        }
                    }
                }
            }
            AddEffect { track, kind } => {
                if mixer_track(p, track)?.effects.len() >= crate::effects::MAX_EFFECTS {
                    return Err(CommandError::Invalid(format!(
                        "a mixer track holds at most {} effects",
                        crate::effects::MAX_EFFECTS
                    )));
                }
                let id = p.alloc_id();
                mixer_track(p, track)?
                    .effects
                    .push(crate::effects::EffectSlot::new(id, kind));
                return Ok(Created(Some(id)));
            }
            RemoveEffect { track, effect } => {
                let t = mixer_track(p, track)?;
                let i = t
                    .effects
                    .iter()
                    .position(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?;
                t.effects.remove(i);
            }
            MoveEffect { track, effect, delta } => {
                let t = mixer_track(p, track)?;
                let i = t
                    .effects
                    .iter()
                    .position(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?;
                let j = (i as i64 + i64::from(delta)).clamp(0, t.effects.len() as i64 - 1) as usize;
                let e = t.effects.remove(i);
                t.effects.insert(j, e);
            }
            SetEffectEnabled { track, effect, enabled } => {
                let t = mixer_track(p, track)?;
                t.effects
                    .iter_mut()
                    .find(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?
                    .enabled = enabled;
            }
            SetEffectParam {
                track,
                effect,
                index,
                value,
            } => {
                let v = finite(value)?;
                let t = mixer_track(p, track)?;
                let e = t
                    .effects
                    .iter_mut()
                    .find(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?;
                let spec = e
                    .kind
                    .params()
                    .get(index)
                    .ok_or_else(|| CommandError::Invalid(format!("no parameter {index}")))?;
                let v = v.clamp(spec.min, spec.max);
                if let Some(slot) = e.params.get_mut(index) {
                    *slot = if spec.labels.is_empty() { v } else { v.round() };
                }
            }

            Batch(cmds) => {
                let mut last = Created(None);
                for c in cmds {
                    let out = c.apply(p)?;
                    if out.0.is_some() {
                        last = out;
                    }
                }
                return Ok(last);
            }
        }
        ok()
    }
}

fn add_notes(p: &mut Project, pid: PatternId, notes: Vec<NewNote>) -> R {
    p.pattern(pid).ok_or(CommandError::NoPattern(pid))?;
    let mut last = None;
    let mut new = Vec::with_capacity(notes.len());
    for n in notes {
        if p.channel(n.channel).is_none() {
            return Err(CommandError::NoChannel(n.channel));
        }
        let id = p.alloc_id();
        last = Some(id);
        new.push(Note {
            id: NoteId(id),
            channel: n.channel,
            key: n.key.min(127),
            start: n.start,
            length: n.length.max(1),
            velocity: finite(n.velocity)?.clamp(0.0, 1.0),
        });
    }
    let pat = pattern(p, pid)?;
    pat.notes.extend(new);
    sort_notes(pat);
    Ok(Created(last))
}

fn clone_pattern(p: &mut Project, i: usize) -> PatternId {
    let new_id = PatternId(p.alloc_id());
    let mut pat = p.patterns[i].clone();
    pat.id = new_id;
    pat.name = format!("{} (copy)", pat.name);
    for n in &mut pat.notes {
        n.id = NoteId(p.alloc_id());
    }
    p.patterns.insert(i + 1, pat);
    new_id
}
