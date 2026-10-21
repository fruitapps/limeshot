//! Typed, serialisable project mutations.
//!
//! Every change to a [`Project`] goes through a [`Command`], whether it comes from the UI, a
//! keyboard shortcut, the CLI or a test. [`crate::Document`] wraps commands with undo/redo.

use crate::automation::*;
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

/// A channel to create with its notes, for [`Command::ImportPattern`]. Notes are
/// (key, start, length, velocity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportedChannel {
    pub name: String,
    pub instrument: Instrument,
    pub notes: Vec<(u8, Tick, Tick, f32)>,
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
    /// Move note starts by `dstart` ticks keeping their ends (resizing from the left edge); each
    /// note keeps at least one tick and starts at 0 or later.
    ResizeNotesStart {
        pattern: PatternId,
        notes: Vec<NoteId>,
        dstart: i64,
    },
    SetNoteVelocity {
        pattern: PatternId,
        notes: Vec<NoteId>,
        velocity: f32,
    },
    /// Mute or unmute notes (they stay in the pattern).
    SetNotesMuted {
        pattern: PatternId,
        notes: Vec<NoteId>,
        muted: bool,
    },
    /// Split every listed note that spans `at` into two notes meeting there. Returns the id of the
    /// last new (right-hand) note.
    SliceNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
        at: Tick,
    },
    /// Join listed notes of the same channel and key that touch or overlap into one note each
    /// (FL Studio's Glue).
    GlueNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
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
    /// Add a pattern holding notes for newly created channels (MIDI import), optionally setting
    /// the tempo. One undo step; returns the pattern id.
    ImportPattern {
        name: String,
        tempo: Option<Tempo>,
        channels: Vec<ImportedChannel>,
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
    /// Place a sample on the Playlist as an audio clip. Reuses the channel of an audio clip that
    /// already plays this sample, otherwise adds a sampler channel for it (FL Studio's audio clip
    /// channel). Returns the clip id.
    AddAudioClip {
        sample: SampleRef,
        name: String,
        track: u32,
        start: Tick,
        length: Tick,
    },
    /// Place a clip of any kind (paste).
    PlaceClip {
        source: ClipSource,
        track: u32,
        start: Tick,
        length: Tick,
        offset: Tick,
    },
    /// Create an automation of `target`, flat at the target's current value, and place one clip
    /// of it (FL Studio's "Create automation clip"). Returns the clip id.
    AddAutomationClip {
        target: AutomationTarget,
        track: u32,
        start: Tick,
        length: Tick,
    },
    /// Replace an automation's points (they are sorted and clamped).
    SetAutomationPoints {
        automation: AutomationId,
        points: Vec<AutomationPoint>,
    },
    RenameAutomation {
        automation: AutomationId,
        name: String,
    },
    /// Delete an automation and all its clips.
    RemoveAutomation(AutomationId),
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
    /// Move a clip's start while keeping its end, trimming or revealing the beginning of its
    /// source (dragging the left edge).
    ResizeClipStart {
        clip: ClipId,
        start: Tick,
    },
    /// Slide a clip's content under it (FL Studio's Slip tool): the clip keeps its place and
    /// length, only where in the source it starts changes.
    SlipClip {
        clip: ClipId,
        offset: Tick,
    },
    /// Cut the listed clips at `at` (FL Studio's Slice tool). The right pieces continue the
    /// source where the left ones stop; returns the id of the last new piece.
    SliceClips {
        clips: Vec<ClipId>,
        at: Tick,
    },
    DuplicateClips(Vec<ClipId>),
    /// Give the clip its own copy of its pattern.
    MakeClipUnique(ClipId),
    SetClipMuted {
        clip: ClipId,
        muted: bool,
    },
    AddPlaylistTrack,
    /// A time marker on the Playlist; returns its id.
    AddMarker {
        tick: Tick,
        name: String,
    },
    RemoveMarker(MarkerId),
    RenameMarker {
        marker: MarkerId,
        name: String,
    },
    MoveMarker {
        marker: MarkerId,
        tick: Tick,
    },
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
    /// Set one band (0 low, 1 mid, 2 high) of a mixer track's built-in EQ.
    SetMixerEq {
        track: u16,
        band: u8,
        value: EqBand,
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
            ResizeNotes { .. } | ResizeNotesStart { .. } => "Resize notes",
            SetNoteVelocity { .. } => "Note velocity",
            DuplicateNotes { .. } => "Duplicate notes",
            SetNotesMuted { muted: true, .. } => "Mute notes",
            SetNotesMuted { .. } => "Unmute notes",
            SliceNotes { .. } => "Slice notes",
            GlueNotes { .. } => "Glue notes",
            ClearChannelNotes { .. } => "Clear channel",
            Quantize { .. } => "Quantize",
            AddPattern { .. } => "Add pattern",
            ImportPattern { .. } => "Import MIDI",
            ClonePattern(_) => "Clone pattern",
            RemovePattern(_) => "Delete pattern",
            RenamePattern { .. } => "Rename pattern",
            SetPatternLength { .. } => "Pattern length",
            AddClip { .. } | PlaceClip { .. } => "Add clip",
            AddAudioClip { .. } => "Add audio clip",
            AddAutomationClip { .. } => "Create automation clip",
            SetAutomationPoints { .. } => "Edit automation",
            RenameAutomation { .. } => "Rename automation",
            RemoveAutomation(_) => "Delete automation",
            RemoveClips(_) => "Delete clips",
            MoveClips { .. } => "Move clips",
            ResizeClip { .. } | ResizeClipStart { .. } => "Resize clip",
            SlipClip { .. } => "Slip clip",
            SliceClips { .. } => "Slice clips",
            DuplicateClips(_) => "Duplicate clips",
            MakeClipUnique(_) => "Make unique",
            SetClipMuted { .. } => "Mute clip",
            AddPlaylistTrack => "Add playlist track",
            RenamePlaylistTrack { .. } => "Rename track",
            AddMarker { .. } => "Add marker",
            RemoveMarker(_) => "Delete marker",
            RenameMarker { .. } => "Rename marker",
            MoveMarker { .. } => "Move marker",
            SetPlaylistTrackMute { .. } => "Mute track",
            AddMixerTrack => "Add mixer track",
            RenameMixerTrack { .. } => "Rename mixer track",
            SetMixerVolume { .. } => "Mixer volume",
            SetMixerPan { .. } => "Mixer pan",
            SetMixerEq { .. } => "Mixer EQ",
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
            SetMixerEq { track, band, .. } => format!("mxeq{track}:{band}"),
            MoveNotes { pattern, notes, .. } => format!("mvn{}:{notes:?}", pattern.0),
            ResizeNotes { pattern, notes, .. } => format!("rsn{}:{notes:?}", pattern.0),
            ResizeNotesStart { pattern, notes, .. } => format!("rsns{}:{notes:?}", pattern.0),
            // A paint stroke or a mute drag is one undo step.
            AddNote { pattern, note } => format!("addn{}:{}", pattern.0, note.channel.0),
            SetNotesMuted { pattern, muted, .. } => format!("mute{}:{muted}", pattern.0),
            SetNoteVelocity { pattern, notes, .. } => format!("vel{}:{notes:?}", pattern.0),
            // A Playlist paint stroke or mute drag is one undo step too.
            AddClip { track, .. } | PlaceClip { track, .. } => format!("addc{track}"),
            SetClipMuted { muted, .. } => format!("cmute{muted}"),
            MoveClips { clips, .. } => format!("mvc{clips:?}"),
            ResizeClip { clip, .. } => format!("rsc{}", clip.0),
            ResizeClipStart { clip, .. } => format!("rss{}", clip.0),
            SlipClip { clip, .. } => format!("slip{}", clip.0),
            SetAutomationPoints { automation, .. } => format!("autopts{}", automation.0),
            SetStep {
                pattern, channel, on, ..
            } => format!("step{}:{}:{on}", pattern.0, channel.0),
            MoveMarker { marker, .. } => format!("mvm{}", marker.0),
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
                p.playlist.clips.retain(|c| c.audio_channel() != Some(id));
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
                            muted: false,
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
            ResizeNotesStart {
                pattern: pid,
                notes,
                dstart,
            } => {
                for n in pattern(p, pid)?.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    let end = n.end();
                    n.start = offset(n.start, dstart).min(end.saturating_sub(1));
                    n.length = end - n.start;
                }
                sort_notes(pattern(p, pid)?);
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
            SetNotesMuted {
                pattern: pid,
                notes,
                muted,
            } => {
                for n in pattern(p, pid)?.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    n.muted = muted;
                }
            }
            SliceNotes {
                pattern: pid,
                notes,
                at,
            } => {
                let pat = pattern(p, pid)?;
                let cut: Vec<Note> = pat
                    .notes
                    .iter()
                    .filter(|n| notes.contains(&n.id) && n.start < at && at < n.end())
                    .copied()
                    .collect();
                let mut last = None;
                for n in cut {
                    let id = NoteId(p.alloc_id());
                    last = Some(id.0);
                    let pat = pattern(p, pid)?;
                    if let Some(left) = pat.notes.iter_mut().find(|m| m.id == n.id) {
                        left.length = at - n.start;
                    }
                    pat.notes.push(Note {
                        id,
                        start: at,
                        length: n.end() - at,
                        ..n
                    });
                }
                sort_notes(pattern(p, pid)?);
                return Ok(Created(last));
            }
            GlueNotes { pattern: pid, notes } => {
                let pat = pattern(p, pid)?;
                let mut sel: Vec<Note> = pat.notes.iter().filter(|n| notes.contains(&n.id)).copied().collect();
                sel.sort_by_key(|n| (n.channel.0, n.key, n.start));
                let mut keep: Option<Note> = None;
                let mut glued: Vec<(NoteId, Tick)> = Vec::new();
                let mut gone: Vec<NoteId> = Vec::new();
                for n in sel {
                    match keep {
                        Some(k) if k.channel == n.channel && k.key == n.key && n.start <= k.end() => {
                            let end = k.end().max(n.end());
                            keep = Some(Note {
                                length: end - k.start,
                                ..k
                            });
                            gone.push(n.id);
                        }
                        _ => {
                            if let Some(k) = keep {
                                glued.push((k.id, k.length));
                            }
                            keep = Some(n);
                        }
                    }
                }
                if let Some(k) = keep {
                    glued.push((k.id, k.length));
                }
                pat.notes.retain(|n| !gone.contains(&n.id));
                for n in &mut pat.notes {
                    if let Some((_, len)) = glued.iter().find(|(id, _)| *id == n.id) {
                        n.length = *len;
                    }
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
            ImportPattern { name, tempo, channels } => {
                if let Some(t) = tempo {
                    p.tempo = t;
                }
                let pat = PatternId(AddPattern { name }.apply(p)?.0.unwrap_or_default());
                for c in channels {
                    let ch = ChannelId(
                        AddChannel {
                            name: c.name,
                            instrument: c.instrument,
                        }
                        .apply(p)?
                        .0
                        .unwrap_or_default(),
                    );
                    let notes = c
                        .notes
                        .into_iter()
                        .map(|(key, start, length, velocity)| NewNote {
                            channel: ch,
                            key,
                            start,
                            length,
                            velocity,
                        })
                        .collect();
                    add_notes(p, pat, notes)?;
                }
                return Ok(Created(Some(pat.0)));
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
                p.playlist.clips.retain(|c| c.pattern() != Some(id));
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
            AddAudioClip {
                sample,
                name,
                track,
                start,
                length,
            } => {
                if track as usize >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                let audio_channels: Vec<ChannelId> = p.playlist.clips.iter().filter_map(Clip::audio_channel).collect();
                let reuse = p
                    .channels
                    .iter()
                    .find(|c| {
                        audio_channels.contains(&c.id)
                            && matches!(&c.instrument, Instrument::Sampler(s) if s.sample.as_ref() == Some(&sample))
                    })
                    .map(|c| c.id);
                let channel = match reuse {
                    Some(c) => c,
                    None => {
                        let instrument = Instrument::Sampler(SamplerParams {
                            sample: Some(sample),
                            // An audio clip plays for the clip's length, not the whole sample.
                            one_shot: false,
                            ..SamplerParams::default()
                        });
                        let id = Command::AddChannel { name, instrument }.apply(p)?;
                        ChannelId(id.0.unwrap_or_default())
                    }
                };
                return place_clip(p, ClipSource::Audio(channel), track, start, length, 0);
            }
            PlaceClip {
                source,
                track,
                start,
                length,
                offset,
            } => return place_clip(p, source, track, start, length, offset),
            AddAutomationClip {
                target,
                track,
                start,
                length,
            } => {
                let (Some(mapping), Some(value)) = (target.mapping(p), target.current_value(p)) else {
                    return Err(CommandError::Invalid("that control no longer exists".into()));
                };
                if track as usize >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                let id = AutomationId(p.alloc_id());
                let color = PALETTE[(p.automations.len() + 6) % PALETTE.len()];
                p.automations.push(Automation {
                    id,
                    name: target.label(p),
                    color,
                    target,
                    points: vec![AutomationPoint {
                        tick: 0,
                        value: mapping.to_norm(value),
                        tension: 0.0,
                    }],
                    length: length.max(1),
                    extra: Extra::new(),
                });
                return place_clip(p, ClipSource::Automation(id), track, start, length, 0);
            }
            SetAutomationPoints { automation, mut points } => {
                Automation::normalize_points(&mut points);
                p.automation_mut(automation)
                    .ok_or_else(|| CommandError::Invalid(format!("no automation {}", automation.0)))?
                    .points = points;
            }
            RenameAutomation { automation, name } => {
                p.automation_mut(automation)
                    .ok_or_else(|| CommandError::Invalid(format!("no automation {}", automation.0)))?
                    .name = name
            }
            RemoveAutomation(id) => {
                let i = p
                    .automations
                    .iter()
                    .position(|a| a.id == id)
                    .ok_or_else(|| CommandError::Invalid(format!("no automation {}", id.0)))?;
                p.automations.remove(i);
                p.playlist.clips.retain(|c| c.automation() != Some(id));
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
            ResizeClipStart { clip, start } => {
                let c = p.clip_mut(clip).ok_or(CommandError::NoClip(clip))?;
                let end = c.end();
                let start = start.clamp(c.start.saturating_sub(c.offset), end.saturating_sub(1));
                c.offset = (c.offset + start).saturating_sub(c.start);
                c.length = end - start;
                c.start = start;
            }
            SlipClip { clip, offset } => p.clip_mut(clip).ok_or(CommandError::NoClip(clip))?.offset = offset,
            SliceClips { clips, at } => {
                let cut: Vec<Clip> = p
                    .playlist
                    .clips
                    .iter()
                    .filter(|c| clips.contains(&c.id) && c.start < at && at < c.end())
                    .cloned()
                    .collect();
                let mut last = None;
                for c in cut {
                    let id = ClipId(p.alloc_id());
                    last = Some(id.0);
                    if let Some(left) = p.clip_mut(c.id) {
                        left.length = at - c.start;
                    }
                    p.playlist.clips.push(Clip {
                        id,
                        start: at,
                        length: c.end() - at,
                        offset: c.offset + (at - c.start),
                        ..c
                    });
                }
                return Ok(Created(last));
            }
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
                let source = p.clip(id).ok_or(CommandError::NoClip(id))?.source;
                if let ClipSource::Automation(aid) = source {
                    let mut a = p
                        .automation(aid)
                        .cloned()
                        .ok_or_else(|| CommandError::Invalid(format!("no automation {}", aid.0)))?;
                    let new = AutomationId(p.alloc_id());
                    a.id = new;
                    a.name = format!("{} (copy)", a.name);
                    p.automations.push(a);
                    if let Some(c) = p.clip_mut(id) {
                        c.source = ClipSource::Automation(new);
                    }
                    return Ok(Created(Some(new.0)));
                }
                let pid = source
                    .pattern_id()
                    .ok_or_else(|| CommandError::Invalid("audio clips cannot be made unique".into()))?;
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
            AddMarker { tick, name } => {
                let id = MarkerId(p.alloc_id());
                p.playlist.markers.push(Marker { id, tick, name });
                p.playlist.markers.sort_by_key(|m| m.tick);
                return Ok(Created(Some(id.0)));
            }
            RemoveMarker(id) => {
                let before = p.playlist.markers.len();
                p.playlist.markers.retain(|m| m.id != id);
                if p.playlist.markers.len() == before {
                    return Err(CommandError::Invalid(format!("no marker {}", id.0)));
                }
            }
            RenameMarker { marker, name } => marker_mut(p, marker)?.name = name,
            MoveMarker { marker, tick } => {
                marker_mut(p, marker)?.tick = tick;
                p.playlist.markers.sort_by_key(|m| m.tick);
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
            SetMixerEq { track, band, value } => {
                let b = usize::from(band);
                if b > 2 {
                    return Err(CommandError::Invalid(format!("no EQ band {band}")));
                }
                finite(value.freq)?;
                finite(value.gain)?;
                finite(value.width)?;
                mixer_track(p, track)?.eq[b] = value.sanitized(b);
            }
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

fn place_clip(p: &mut Project, source: ClipSource, track: u32, start: Tick, length: Tick, offset: Tick) -> R {
    match source {
        ClipSource::Pattern(id) if p.pattern(id).is_none() => return Err(CommandError::NoPattern(id)),
        ClipSource::Audio(id) if p.channel(id).is_none() => return Err(CommandError::NoChannel(id)),
        ClipSource::Automation(id) if p.automation(id).is_none() => {
            return Err(CommandError::Invalid(format!("no automation {}", id.0)));
        }
        _ => {}
    }
    if track as usize >= p.playlist.tracks.len() {
        return Err(CommandError::NoPlaylistTrack(track));
    }
    let id = p.alloc_id();
    p.playlist.clips.push(Clip {
        id: ClipId(id),
        source,
        track,
        start,
        length: length.max(1),
        offset,
        muted: false,
        extra: Extra::new(),
    });
    Ok(Created(Some(id)))
}

fn marker_mut(p: &mut Project, id: MarkerId) -> Result<&mut Marker, CommandError> {
    p.playlist
        .markers
        .iter_mut()
        .find(|m| m.id == id)
        .ok_or_else(|| CommandError::Invalid(format!("no marker {}", id.0)))
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
            muted: false,
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
