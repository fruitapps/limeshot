//! Typed, serialisable project mutations.
//!
//! Every change to a [`Project`] goes through a [`Command`], whether it comes from the UI, a
//! keyboard shortcut, the CLI or a test. [`crate::Document`] wraps commands with undo/redo.

use crate::automation::*;
use crate::model::*;
use crate::note_tools::NoteTool;
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
    /// Options › Project info: the title and the credits at once (typing in a field merges into
    /// one undo step).
    SetProjectInfo {
        title: String,
        info: crate::model::ProjectInfo,
    },
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
    /// Put the channels in this order (a permutation of the project's channels; FL Studio's
    /// Sort by).
    ReorderChannels(Vec<ChannelId>),
    SetChannelMute {
        channel: ChannelId,
        muted: bool,
    },
    SetChannelMuteLock {
        channel: ChannelId,
        locked: bool,
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
    /// Explicit source-file selection permits embedding that channel's pending external audio.
    /// Preset and parameter changes do not carry this permission.
    ApproveSamplerSource {
        channel: ChannelId,
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
    DuplicateNotesBy {
        pattern: PatternId,
        notes: Vec<NoteId>,
        ticks: Tick,
    },
    DuplicateNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
    },
    ClearChannelNotes {
        pattern: PatternId,
        channel: ChannelId,
    },
    /// Set one property of several notes, each to its own value (FL Studio's graph editor). A
    /// drag sends the whole set each time, so it stays one undo step.
    SetNoteValues {
        pattern: PatternId,
        prop: NoteProp,
        values: Vec<(NoteId, f32)>,
    },
    /// Put channels in a Channel Rack filter group (`None`: Unsorted). FL Studio's Group
    /// selected… (Opt+G).
    SetChannelGroup {
        channels: Vec<ChannelId>,
        group: Option<String>,
    },
    /// Replace the channel's notes with a step every `every` steps over the pattern's length
    /// (FL Studio's Fill each 2/4/8 steps).
    FillSteps {
        pattern: PatternId,
        channel: ChannelId,
        every: u32,
    },
    /// Move the notes by `by` ticks, wrapping around the pattern's length (FL Studio's Rotate
    /// left/right).
    RotateNotes {
        pattern: PatternId,
        notes: Vec<NoteId>,
        by: i64,
    },
    /// Replace the notes `remove` with `notes` in one step (FL Studio's Piano Roll tools,
    /// [`crate::note_tools`]). A replacement whose id is in `remove` keeps it; any other gets a
    /// new id. Inside a gesture, edits by the same tool merge (a tool dialog's live preview).
    EditNotes {
        pattern: PatternId,
        tool: NoteTool,
        remove: Vec<NoteId>,
        notes: Vec<Note>,
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
    /// Insert an empty pattern before `before` (FL Studio's Patterns › Insert one). Patterns
    /// with default names ("Pattern 3") are renumbered by position, as FL numbers its unnamed
    /// patterns. Returns the new pattern's id.
    InsertPattern {
        before: PatternId,
    },
    /// Move a pattern up (−1) or down (+1) the list (Patterns › Move up / Move down); default
    /// names are renumbered.
    MovePattern {
        pattern: PatternId,
        delta: i32,
    },
    /// FL Studio's Patterns › Split by channel: the pattern keeps the notes of its first channel
    /// (in Channel Rack order) and takes that channel's name; every other channel's notes move
    /// to a new pattern named after the channel, added at the end of the list. Playlist clips
    /// keep pointing at the original pattern (observed in FL). Returns the first new pattern.
    SplitPatternByChannel(PatternId),
    RemovePattern(PatternId),
    RenamePattern {
        pattern: PatternId,
        name: String,
    },
    SetPatternColor {
        pattern: PatternId,
        color: Rgb,
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
        /// The control's normalised value, for targets whose value the project does not hold
        /// (plugin parameters).
        #[serde(default)]
        value: Option<f32>,
        /// The automation's name instead of the target's label (a plugin parameter's title).
        #[serde(default)]
        name: Option<String>,
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
    DuplicateClipsBy {
        clips: Vec<ClipId>,
        ticks: Tick,
    },
    /// Give the clip its own copy of its pattern.
    MakeClipUnique(ClipId),
    SetClipMuted {
        clip: ClipId,
        muted: bool,
    },
    /// An audio clip's fade-in and fade-out lengths (each clamped so they fit the clip).
    SetClipFades {
        clip: ClipId,
        fade_in: Tick,
        fade_out: Tick,
    },
    /// Insert `length` ticks of empty time at `at` (FL Studio's Playlist Insert space): clips
    /// and time markers from `at` on move right. With `slice`, clips crossing `at` are cut there
    /// first (Slice & insert space); otherwise they stay.
    InsertSpace {
        at: Tick,
        length: Tick,
        slice: bool,
    },
    /// Remove the time `from..to` (FL Studio's Delete space): clips are cut at both ends, what
    /// lies inside goes, later clips and markers move left.
    DeleteSpace {
        from: Tick,
        to: Tick,
    },
    /// Merge pattern clips into one new pattern holding their notes where they sound, placed
    /// as one clip on the topmost of their tracks (FL Studio's Merge selected clips). Returns the
    /// new clip's id.
    MergeClips(Vec<ClipId>),
    AddPlaylistTrack,
    ClonePlaylistTrack(u32),
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
    SetPlaylistTrackMuteLock {
        track: u32,
        locked: bool,
    },
    SetPlaylistTrackColor {
        track: u32,
        color: Option<Rgb>,
    },
    MovePlaylistTrack {
        track: u32,
        delta: i32,
    },
    InsertPlaylistTrack {
        track: u32,
    },
    RemovePlaylistTrack {
        track: u32,
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
    SetMixerMuteLock {
        track: u16,
        locked: bool,
    },
    SetMixerMute {
        track: u16,
        muted: bool,
    },
    SetMixerSolo {
        track: u16,
        solo: bool,
    },
    /// Move an insert track to another position (FL Studio's Move left / right, Opt+←/→); every
    /// reference to a track index (channels, routes, automation, sidechains) follows.
    MoveMixerTrack {
        track: u16,
        to: u16,
    },
    /// Insert an empty track at `at` (FL Studio's Insert one); later tracks move up one.
    InsertMixerTrack(u16),
    /// Delete an insert track (FL Studio's Delete). Its channels go to the master, routes to it
    /// are dropped, automation of it is deleted; later tracks move down one.
    RemoveMixerTrack(u16),
    /// Volume, pan, EQ, mute, solo and colour back to their defaults (FL Studio's Reset
    /// selected track(s) to default); effects and routing stay.
    ResetMixerTrack(u16),
    /// Send only to the master (FL Studio's Reset selected track(s) routing).
    ResetMixerRouting(u16),
    /// Drop every route into `track` (FL Studio's Unroute tracks routed to the selected track);
    /// a track left without routes goes to the master.
    UnrouteMixerTrack(u16),
    SetMixerTrackColor {
        track: u16,
        color: Option<Rgb>,
    },
    /// Route mixer track `from` to `to` at `level` (linear), update an existing route's level,
    /// or (with `None`) remove the route. Routes that would create a feedback loop are refused.
    SetRoute {
        from: u16,
        to: u16,
        level: Option<f32>,
    },
    /// Set all of an effect's parameters at once (a preset); they are brought to its ranges.
    SetEffectParams {
        track: u16,
        effect: u32,
        params: Vec<f32>,
    },
    /// Make the route `from` → `to` a pre-fader send (before the fader and pan) or post-fader.
    SetRoutePre {
        from: u16,
        to: u16,
        pre: bool,
    },
    /// Append an effect to a mixer track's insert chain (returns the effect id).
    AddEffect {
        track: u16,
        kind: crate::effects::EffectKind,
    },
    /// Append a plugin to a mixer track's insert chain (returns the effect id).
    AddPluginEffect {
        track: u16,
        plugin: crate::PluginRef,
    },
    /// Run a channel's or insert's plugin in a process of its own (FL Studio's bridged plugins),
    /// or back in LimeShot's.
    SetPluginState {
        plugin: crate::PluginTarget,
        state: crate::PluginState,
    },
    SetPluginSidechain {
        plugin: crate::PluginTarget,
        source: u16,
    },
    SetPluginSeparateProcess {
        plugin: crate::PluginTarget,
        on: bool,
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
    SetEffectMix {
        track: u16,
        effect: u32,
        mix: f32,
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

fn add_effect(
    p: &mut Project,
    track: u16,
    slot: impl FnOnce(u32) -> crate::effects::EffectSlot,
) -> Result<Created, CommandError> {
    if mixer_track(p, track)?
        .effects
        .iter()
        .filter(|e| !crate::lime::is_track_eq(e))
        .count()
        >= crate::effects::MAX_EFFECTS
    {
        return Err(CommandError::Invalid(format!(
            "a mixer track holds at most {} effects",
            crate::effects::MAX_EFFECTS
        )));
    }
    let id = p.alloc_id();
    let t = mixer_track(p, track)?;
    let index = t
        .effects
        .iter()
        .position(crate::lime::is_track_eq)
        .unwrap_or(t.effects.len());
    t.effects.insert(index, slot(id));
    Ok(Created(Some(id)))
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

/// Renumber every reference to a mixer track after tracks moved, were inserted or deleted.
/// `map` gives a track's new index, or `None` when it was deleted: its channels then play on the
/// master, routes to it are dropped (a track left without routes goes to the master) and
/// sidechains keyed by it fall back to the compressor's own input.
fn remap_mixer(p: &mut Project, map: impl Fn(u16) -> Option<u16>) {
    for c in &mut p.channels {
        c.mixer_track = map(c.mixer_track).unwrap_or(0);
    }
    for (k, t) in p.mixer.tracks.iter_mut().enumerate() {
        t.routes = t
            .routes
            .iter()
            .filter_map(|r| map(r.to).map(|to| Route { to, ..*r }))
            .collect();
        if t.routes.is_empty() && k != 0 {
            t.routes.push(Route::new(0, 1.0));
        }
        for e in &mut t.effects {
            if let Some(plugin) = &mut e.plugin
                && let Some(source) = plugin
                    .extra
                    .get("lime-sidechain-track")
                    .and_then(serde_json::Value::as_u64)
            {
                let source = u16::try_from(source).ok().and_then(&map).unwrap_or(0);
                plugin
                    .extra
                    .insert("lime-sidechain-track".into(), serde_json::json!(source));
                let key = b"key"
                    .iter()
                    .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)));
                if let Some(state) = crate::lime::edited_param_state(plugin, key, if source == 0 { 0.0 } else { 1.0 }) {
                    plugin.state = state;
                }
            }
            if e.kind == crate::effects::EffectKind::Compressor
                && let Some(v) = e.params.get_mut(crate::effects::COMPRESSOR_SIDECHAIN)
                && *v >= 1.0
            {
                *v = map(*v as u16).map_or(0.0, f32::from);
            }
        }
    }
    for a in &mut p.automations {
        a.target = a.target.remap_mixer(&map);
    }
    // Default names follow the position, as FL Studio's unnamed inserts do.
    for (k, t) in p.mixer.tracks.iter_mut().enumerate().skip(1) {
        if t.name.strip_prefix("Insert ").is_some_and(|n| n.parse::<u32>().is_ok()) {
            t.name = format!("Insert {k}");
        }
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
            SetProjectInfo { .. } => "Project info",
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
            ReorderChannels(_) => "Sort channels",
            SetChannelMuteLock { .. } => "Lock channel mute",
            SetChannelSolo { .. } => "Solo channel",
            SetChannelMixerTrack { .. } => "Route channel",
            SetChannelRootKey { .. } => "Set root key",
            ApproveSamplerSource { .. } => "Choose sample source",
            SetInstrument { .. } => "Edit instrument",
            SetStep { .. } => "Toggle step",
            AddNote { .. } | AddNotes { .. } => "Add note",
            RemoveNotes { .. } => "Delete notes",
            MoveNotes { .. } => "Move notes",
            ResizeNotes { .. } | ResizeNotesStart { .. } => "Resize notes",
            SetNoteVelocity { .. } => "Note velocity",
            DuplicateNotesBy { .. } | DuplicateNotes { .. } => "Duplicate notes",
            SetNotesMuted { muted: true, .. } => "Mute notes",
            SetNotesMuted { .. } => "Unmute notes",
            SliceNotes { .. } => "Slice notes",
            GlueNotes { .. } => "Glue notes",
            ClearChannelNotes { .. } => "Clear channel",
            FillSteps { .. } => "Fill steps",
            SetChannelGroup { .. } => "Group channels",
            SetNoteValues { prop, .. } => match prop {
                NoteProp::Velocity => "Note velocity",
                NoteProp::Pan => "Note pan",
            },
            RotateNotes { .. } => "Rotate",
            EditNotes { tool, .. } => tool.label(),
            Quantize { .. } => "Quantize",
            AddPattern { .. } => "Add pattern",
            ImportPattern { .. } => "Import MIDI",
            ClonePattern(_) => "Clone pattern",
            InsertPattern { .. } => "Insert pattern",
            MovePattern { .. } => "Move pattern",
            SplitPatternByChannel(_) => "Split by channel",
            RemovePattern(_) => "Delete pattern",
            RenamePattern { .. } => "Rename pattern",
            SetPatternColor { .. } => "Recolor pattern",
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
            DuplicateClipsBy { .. } | DuplicateClips(_) => "Duplicate clips",
            MakeClipUnique(_) => "Make unique",
            SetClipMuted { .. } => "Mute clip",
            SetClipFades { .. } => "Clip fade",
            InsertSpace { .. } => "Insert space",
            DeleteSpace { .. } => "Delete space",
            MergeClips(_) => "Merge clips",
            AddPlaylistTrack => "Add playlist track",
            ClonePlaylistTrack(_) => "Clone playlist track",
            RenamePlaylistTrack { .. } => "Rename track",
            AddMarker { .. } => "Add marker",
            RemoveMarker(_) => "Delete marker",
            RenameMarker { .. } => "Rename marker",
            MoveMarker { .. } => "Move marker",
            SetPlaylistTrackMuteLock { .. } => "Lock track mute",
            SetPlaylistTrackColor { .. } => "Track color",
            MovePlaylistTrack { .. } => "Move playlist track",
            InsertPlaylistTrack { .. } => "Insert playlist track",
            RemovePlaylistTrack { .. } => "Delete playlist track",
            SetPlaylistTrackMute { .. } => "Mute track",
            AddMixerTrack => "Add mixer track",
            RenameMixerTrack { .. } => "Rename mixer track",
            SetMixerVolume { .. } => "Mixer volume",
            SetMixerPan { .. } => "Mixer pan",
            SetMixerEq { .. } => "Mixer EQ",
            SetMixerMuteLock { .. } => "Lock mixer mute",
            SetMixerMute { .. } => "Mute mixer track",
            SetMixerSolo { .. } => "Solo mixer track",
            MoveMixerTrack { .. } => "Move mixer track",
            InsertMixerTrack(_) => "Insert mixer track",
            RemoveMixerTrack(_) => "Delete mixer track",
            ResetMixerTrack(_) => "Reset mixer track",
            ResetMixerRouting(_) => "Reset routing",
            UnrouteMixerTrack(_) => "Unroute",
            SetMixerTrackColor { .. } => "Mixer track color",
            SetRoute { level: None, .. } => "Remove route",
            SetRoute { .. } => "Route mixer track",
            SetEffectParams { .. } => "Load preset",
            SetRoutePre { pre: true, .. } => "Pre-fader send",
            SetRoutePre { .. } => "Post-fader send",
            AddEffect { .. } => "Add effect",
            AddPluginEffect { .. } => "Add plugin",
            SetPluginState { .. } => "Edit plugin",
            SetPluginSidechain { .. } => "Plugin sidechain",
            SetPluginSeparateProcess { .. } => "Plugin process",
            RemoveEffect { .. } => "Remove effect",
            MoveEffect { .. } => "Move effect",
            SetEffectMix { .. } => "Effect mix",
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
            SetProjectInfo { .. } => "projinfo".into(),
            SetRoute {
                from,
                to,
                level: Some(_),
            } => format!("route{from}:{to}"),
            SetEffectParam {
                track, effect, index, ..
            } => format!("fx{track}:{effect}:{index}"),
            SetEffectMix { track, effect, .. } => format!("fxmix{track}:{effect}"),
            SetSwing(_) => "swing".into(),
            SetChannelVolume { channel, .. } => format!("chvol{}", channel.0),
            SetChannelPan { channel, .. } => format!("chpan{}", channel.0),
            SetChannelMixerTrack { channel, .. } => format!("chmix{}", channel.0),
            SetInstrument { channel, .. } => format!("inst{}", channel.0),
            SetPluginState { plugin, .. } => format!("plugin{plugin:?}"),
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
            SetNoteValues { pattern, prop, .. } => format!("graph{}:{prop:?}", pattern.0),
            EditNotes { pattern, tool, .. } => format!("tool{}:{tool:?}", pattern.0),
            // A Playlist paint stroke or mute drag is one undo step too.
            AddClip { track, .. } | PlaceClip { track, .. } => format!("addc{track}"),
            SetClipMuted { muted, .. } => format!("cmute{muted}"),
            SetClipFades { clip, .. } => format!("fade{}", clip.0),
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
    ///
    /// Returns the created id or index when the command supplies one, otherwise `Created(None)`.
    /// Returns [`CommandError`] for missing targets or invalid values and operations, including
    /// errors propagated from nested commands in a batch.
    pub fn apply(self, p: &mut Project) -> R {
        use Command::*;
        match self {
            SetTitle(t) => p.title = t,
            SetProjectInfo { title, mut info } => {
                info.sanitize();
                p.title = title;
                p.info = info;
            }
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
                    mute_locked: false,
                    mixer_track,
                    root_key: 60,
                    group: None,
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
            ReorderChannels(order) => {
                let mut ids: Vec<ChannelId> = p.channels.iter().map(|c| c.id).collect();
                let mut want = order.clone();
                ids.sort_by_key(|c| c.0);
                want.sort_by_key(|c| c.0);
                if ids != want {
                    return Err(CommandError::Invalid(
                        "the new order must list every channel once".into(),
                    ));
                }
                p.channels
                    .sort_by_key(|c| order.iter().position(|o| *o == c.id).unwrap_or(usize::MAX));
            }
            SetChannelMuteLock { channel: id, locked } => channel(p, id)?.mute_locked = locked,
            SetChannelSolo { channel: id, solo } => channel(p, id)?.solo = solo,
            SetChannelMixerTrack { channel: id, track } => {
                if usize::from(track) >= p.mixer.tracks.len() {
                    return Err(CommandError::NoMixerTrack(track));
                }
                channel(p, id)?.mixer_track = track;
            }
            SetChannelRootKey { channel: id, key } => {
                let ch = channel(p, id)?;
                ch.root_key = key.min(127);
                if let Instrument::Plugin(plugin) = &mut ch.instrument
                    && crate::LimePlugin::of(plugin) == Some(crate::LimePlugin::SAMPLER)
                    && let Some(state) = crate::lime::edited_param_state(
                        plugin,
                        crate::lime::param_id(7),
                        f64::from(ch.root_key) / 127.0,
                    )
                {
                    plugin.state = state;
                }
            }
            ApproveSamplerSource { channel: id } => {
                channel(p, id)?.extra.remove("lime-external-source");
            }
            SetInstrument {
                channel: id,
                instrument,
            } => {
                let c = channel(p, id)?;
                c.instrument = instrument;
            }

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
                            pan: 0.0,
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
            DuplicateNotesBy {
                pattern: pid,
                notes,
                ticks,
            } => {
                let selected: Vec<_> = pattern(p, pid)?
                    .notes
                    .iter()
                    .filter(|n| notes.contains(&n.id))
                    .copied()
                    .collect();
                for mut n in selected {
                    n.id = NoteId(p.alloc_id());
                    n.start = n.start.saturating_add(ticks);
                    pattern(p, pid)?.notes.push(n);
                }
                sort_notes(pattern(p, pid)?);
            }
            DuplicateClipsBy { clips, ticks } => {
                let selected: Vec<_> = p
                    .playlist
                    .clips
                    .iter()
                    .filter(|c| clips.contains(&c.id))
                    .cloned()
                    .collect();
                for mut c in selected {
                    c.id = ClipId(p.alloc_id());
                    c.start = c.start.saturating_add(ticks);
                    p.playlist.clips.push(c);
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
            SetNoteValues {
                pattern: pid,
                prop,
                values,
            } => {
                let pat = pattern(p, pid)?;
                for (id, v) in values {
                    let v = finite(v)?;
                    if let Some(n) = pat.notes.iter_mut().find(|n| n.id == id) {
                        match prop {
                            NoteProp::Velocity => n.velocity = v.clamp(0.0, 1.0),
                            NoteProp::Pan => n.pan = v.clamp(-1.0, 1.0),
                        }
                    }
                }
            }
            SetChannelGroup { channels, group } => {
                let group = group.map(|g| g.trim().to_string()).filter(|g| !g.is_empty());
                for c in p.channels.iter_mut().filter(|c| channels.contains(&c.id)) {
                    c.group = group.clone();
                }
            }
            FillSteps {
                pattern: pid,
                channel: cid,
                every,
            } => {
                if every == 0 {
                    return Err(CommandError::Invalid("fill needs a step interval".into()));
                }
                let root = p.channel(cid).ok_or(CommandError::NoChannel(cid))?.root_key;
                let sig = p.time_signature;
                let steps = (pattern(p, pid)?.length(sig) / STEP_TICKS).clamp(1, 512);
                let ids: Vec<u32> = (0..steps.div_ceil(u64::from(every))).map(|_| p.alloc_id()).collect();
                let pat = pattern(p, pid)?;
                pat.notes.retain(|n| n.channel != cid);
                for (k, id) in ids.into_iter().enumerate() {
                    pat.notes.push(Note {
                        id: NoteId(id),
                        channel: cid,
                        key: root,
                        start: k as Tick * u64::from(every) * STEP_TICKS,
                        length: STEP_TICKS,
                        velocity: 0.78,
                        muted: false,
                        pan: 0.0,
                    });
                }
                sort_notes(pat);
            }
            RotateNotes {
                pattern: pid,
                notes,
                by,
            } => {
                let sig = p.time_signature;
                let pat = pattern(p, pid)?;
                let len = pat.length(sig) as i64;
                for n in pat.notes.iter_mut().filter(|n| notes.contains(&n.id)) {
                    n.start = (n.start as i64 + by).rem_euclid(len.max(1)) as Tick;
                }
                sort_notes(pat);
            }
            EditNotes {
                pattern: pid,
                remove,
                notes,
                ..
            } => {
                let mut kept: Vec<NoteId> = Vec::new();
                let mut new = Vec::with_capacity(notes.len());
                for n in notes {
                    if p.channel(n.channel).is_none() {
                        return Err(CommandError::NoChannel(n.channel));
                    }
                    let id = if remove.contains(&n.id) && !kept.contains(&n.id) {
                        kept.push(n.id);
                        n.id
                    } else {
                        NoteId(p.alloc_id())
                    };
                    new.push(Note {
                        id,
                        key: n.key.min(127),
                        length: n.length.max(1),
                        velocity: finite(n.velocity)?.clamp(0.0, 1.0),
                        pan: finite(n.pan)?.clamp(-1.0, 1.0),
                        ..n
                    });
                }
                let pat = pattern(p, pid)?;
                pat.notes.retain(|n| !remove.contains(&n.id));
                pat.notes.extend(new);
                sort_notes(pat);
            }
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
            InsertPattern { before } => {
                let i = pattern_index(p, before)?;
                let id = PatternId(p.alloc_id());
                let color = PALETTE[p.patterns.len() % PALETTE.len()];
                p.patterns.insert(
                    i,
                    Pattern {
                        id,
                        name: format!("Pattern {}", i + 1),
                        color,
                        length_steps: None,
                        notes: Vec::new(),
                        extra: Extra::new(),
                    },
                );
                renumber_default_names(p);
                return Ok(Created(Some(id.0)));
            }
            MovePattern { pattern: id, delta } => {
                let i = pattern_index(p, id)?;
                let j = (i as i64 + i64::from(delta)).clamp(0, p.patterns.len() as i64 - 1) as usize;
                if i == j {
                    return Err(CommandError::Invalid("the pattern cannot move further".into()));
                }
                let pat = p.patterns.remove(i);
                p.patterns.insert(j, pat);
                renumber_default_names(p);
            }
            SplitPatternByChannel(id) => {
                let i = pattern_index(p, id)?;
                let order: Vec<ChannelId> = p
                    .channels
                    .iter()
                    .map(|c| c.id)
                    .filter(|c| p.patterns[i].notes.iter().any(|n| n.channel == *c))
                    .collect();
                let Some((&first, rest)) = order.split_first() else {
                    return Err(CommandError::Invalid("the pattern has no notes to split".into()));
                };
                let name_of = |p: &Project, c: ChannelId| p.channel(c).map(|c| c.name.clone()).unwrap_or_default();
                let mut first_new = None;
                for &c in rest {
                    let id = PatternId(p.alloc_id());
                    first_new.get_or_insert(id.0);
                    let src = &mut p.patterns[i];
                    let (moved, kept): (Vec<_>, Vec<_>) =
                        std::mem::take(&mut src.notes).into_iter().partition(|n| n.channel == c);
                    src.notes = kept;
                    let (color, length_steps) = (src.color, src.length_steps);
                    let name = name_of(p, c);
                    p.patterns.push(Pattern {
                        id,
                        name,
                        color,
                        length_steps,
                        notes: moved,
                        extra: Extra::new(),
                    });
                }
                p.patterns[i].name = name_of(p, first);
                return Ok(Created(first_new));
            }
            RemovePattern(id) => {
                if p.patterns.len() <= 1 {
                    return Err(CommandError::Invalid("a project keeps at least one pattern".into()));
                }
                let i = pattern_index(p, id)?;
                p.patterns.remove(i);
                p.playlist.clips.retain(|c| c.pattern() != Some(id));
                renumber_default_names(p);
            }
            RenamePattern { pattern: id, name } => pattern(p, id)?.name = name,
            SetPatternColor { pattern: id, color } => pattern(p, id)?.color = color,
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
                    fade_in: 0,
                    fade_out: 0,
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
                        // Audio clip channels: used by an audio clip, or added to the picker.
                        (audio_channels.contains(&c.id) || c.group.as_deref() == Some("Audio"))
                            && crate::lime::sampler_params(&c.instrument)
                                .is_some_and(|s| s.sample.as_ref() == Some(&sample))
                    })
                    .map(|c| c.id);
                let channel = match reuse {
                    Some(c) => c,
                    None => {
                        let instrument = Instrument::Plugin(crate::lime::sampler_ref(
                            &SamplerParams {
                                sample: Some(sample),
                                // An audio clip plays for the clip's length, not the whole sample.
                                one_shot: false,
                                ..SamplerParams::default()
                            },
                            60,
                        ));
                        let id = ChannelId(Command::AddChannel { name, instrument }.apply(p)?.0.unwrap_or_default());
                        // FL Studio files the channels of audio clips under an "Audio" filter group.
                        if let Some(c) = p.channel_mut(id) {
                            c.group = Some("Audio".into());
                        }
                        id
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
            SetClipFades {
                clip,
                fade_in,
                fade_out,
            } => {
                let c = p
                    .playlist
                    .clips
                    .iter_mut()
                    .find(|c| c.id == clip)
                    .ok_or(CommandError::NoClip(clip))?;
                c.fade_in = fade_in.min(c.length);
                c.fade_out = fade_out.min(c.length - c.fade_in);
            }
            InsertSpace { at, length, slice } => {
                if slice {
                    let all = p.playlist.clips.iter().map(|c| c.id).collect();
                    SliceClips { clips: all, at }.apply(p)?;
                }
                for c in p.playlist.clips.iter_mut().filter(|c| c.start >= at) {
                    c.start = c.start.saturating_add(length);
                }
                for m in p.playlist.markers.iter_mut().filter(|m| m.tick >= at) {
                    m.tick = m.tick.saturating_add(length);
                }
            }
            DeleteSpace { from, to } => {
                if to <= from {
                    return Err(CommandError::Invalid("the time to delete is empty".into()));
                }
                for at in [from, to] {
                    let all = p.playlist.clips.iter().map(|c| c.id).collect();
                    SliceClips { clips: all, at }.apply(p)?;
                }
                let gap = to - from;
                p.playlist.clips.retain(|c| !(c.start >= from && c.end() <= to));
                for c in p.playlist.clips.iter_mut().filter(|c| c.start >= to) {
                    c.start -= gap;
                }
                p.playlist.markers.retain(|m| !(from..to).contains(&m.tick));
                for m in p.playlist.markers.iter_mut().filter(|m| m.tick >= to) {
                    m.tick -= gap;
                }
            }
            MergeClips(ids) => {
                let clips: Vec<Clip> = p
                    .playlist
                    .clips
                    .iter()
                    .filter(|c| ids.contains(&c.id) && c.pattern().is_some())
                    .cloned()
                    .collect();
                let (Some(origin), Some(end), Some(track)) = (
                    clips.iter().map(|c| c.start).min(),
                    clips.iter().map(Clip::end).max(),
                    clips.iter().map(|c| c.track).min(),
                ) else {
                    return Err(CommandError::Invalid("select pattern clips to merge".into()));
                };
                let mut notes: Vec<Note> = Vec::new();
                for c in &clips {
                    let Some(pat) = c.pattern().and_then(|id| p.pattern(id)) else {
                        continue;
                    };
                    let window = c.offset..c.offset.saturating_add(c.length);
                    notes.extend(pat.notes.iter().filter(|n| window.contains(&n.start)).map(|n| Note {
                        start: n.start - c.offset + c.start - origin,
                        length: n.length.min(window.end - n.start),
                        ..*n
                    }));
                }
                let name = clips
                    .first()
                    .and_then(|c| c.pattern())
                    .and_then(|id| p.pattern(id))
                    .map(|pat| format!("{} (merged)", pat.name))
                    .unwrap_or_default();
                let Created(Some(pid)) = AddPattern { name }.apply(p)? else {
                    return ok();
                };
                for n in &mut notes {
                    n.id = NoteId(p.alloc_id());
                }
                let pat = pattern(p, PatternId(pid))?;
                pat.notes = notes;
                pat.length_steps = Some((end - origin).div_ceil(STEP_TICKS).clamp(1, 512) as u32);
                sort_notes(pat);
                p.playlist.clips.retain(|c| !clips.iter().any(|m| m.id == c.id));
                return place_clip(p, ClipSource::Pattern(PatternId(pid)), track, origin, end - origin, 0);
            }
            AddAutomationClip {
                target,
                track,
                start,
                length,
                value: norm,
                name,
            } => {
                let current = target.current_value(p);
                let mapping = target.mapping(p);
                let norm = match (mapping, current, norm) {
                    (Some(m), Some(v), _) => Some(m.to_norm(v)),
                    (Some(_), None, Some(n)) if n.is_finite() => Some(n.clamp(0.0, 1.0)),
                    _ => None,
                };
                let Some(norm) = norm else {
                    return Err(CommandError::Invalid("that control no longer exists".into()));
                };
                if track as usize >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                let id = AutomationId(p.alloc_id());
                let color = PALETTE[(p.automations.len() + 6) % PALETTE.len()];
                p.automations.push(Automation {
                    id,
                    name: name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| target.label(p)),
                    color,
                    target,
                    points: vec![AutomationPoint {
                        tick: 0,
                        value: norm,
                        tension: 0.0,
                        mode: Default::default(),
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
            ClonePlaylistTrack(track) => {
                let mut source = p
                    .playlist
                    .tracks
                    .get(track as usize)
                    .cloned()
                    .ok_or(CommandError::NoPlaylistTrack(track))?;
                let clips: Vec<Clip> = p.playlist.clips.iter().filter(|c| c.track == track).cloned().collect();
                InsertPlaylistTrack { track: track + 1 }.apply(p)?;
                source.name = format!("{} (copy)", source.name);
                p.playlist.tracks[(track + 1) as usize] = source;
                for mut clip in clips {
                    clip.id = ClipId(p.alloc_id());
                    clip.track = track + 1;
                    p.playlist.clips.push(clip);
                }
            }
            AddPlaylistTrack => {
                if p.playlist.tracks.len() >= 500 {
                    return Err(CommandError::Invalid("the playlist holds at most 500 tracks".into()));
                }
                let n = p.playlist.tracks.len() + 1;
                p.playlist.tracks.push(PlaylistTrack {
                    color: None,
                    mute_locked: false,
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
            SetPlaylistTrackMuteLock { track, locked } => {
                p.playlist
                    .tracks
                    .get_mut(track as usize)
                    .ok_or(CommandError::NoPlaylistTrack(track))?
                    .mute_locked = locked;
            }
            SetPlaylistTrackColor { track, color } => {
                p.playlist
                    .tracks
                    .get_mut(track as usize)
                    .ok_or(CommandError::NoPlaylistTrack(track))?
                    .color = color;
            }
            MovePlaylistTrack { track, delta } => {
                let i = track as usize;
                if i >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                let j = (i as i64 + i64::from(delta)).clamp(0, p.playlist.tracks.len() as i64 - 1) as usize;
                let item = p.playlist.tracks.remove(i);
                p.playlist.tracks.insert(j, item);
                for c in &mut p.playlist.clips {
                    let n = c.track as usize;
                    c.track = if n == i {
                        j as u32
                    } else if i < j && n > i && n <= j {
                        c.track - 1
                    } else if j < i && n >= j && n < i {
                        c.track + 1
                    } else {
                        c.track
                    };
                }
            }
            InsertPlaylistTrack { track } => {
                if p.playlist.tracks.len() >= 500 {
                    return Err(CommandError::Invalid("the playlist holds at most 500 tracks".into()));
                }
                let i = (track as usize).min(p.playlist.tracks.len());
                p.playlist.tracks.insert(
                    i,
                    PlaylistTrack {
                        name: format!("Track {}", i + 1),
                        color: None,
                        muted: false,
                        mute_locked: false,
                    },
                );
                for c in &mut p.playlist.clips {
                    if c.track >= i as u32 {
                        c.track += 1;
                    }
                }
            }
            RemovePlaylistTrack { track } => {
                if track as usize >= p.playlist.tracks.len() {
                    return Err(CommandError::NoPlaylistTrack(track));
                }
                p.playlist.tracks.remove(track as usize);
                p.playlist.clips.retain(|c| c.track != track);
                for c in &mut p.playlist.clips {
                    if c.track > track {
                        c.track -= 1;
                    }
                }
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
            MoveMixerTrack { track, to } => {
                let n = p.mixer.tracks.len() as u16;
                if track == 0 || to == 0 || track >= n || to >= n {
                    return Err(CommandError::Invalid("only insert tracks move".into()));
                }
                let t = p.mixer.tracks.remove(usize::from(track));
                p.mixer.tracks.insert(usize::from(to), t);
                remap_mixer(p, |i| {
                    Some(if i == track {
                        to
                    } else if track < to && (track + 1..=to).contains(&i) {
                        i - 1
                    } else if to < track && (to..track).contains(&i) {
                        i + 1
                    } else {
                        i
                    })
                });
            }
            InsertMixerTrack(at) => {
                let n = p.mixer.tracks.len();
                if at == 0 || usize::from(at) > n || n >= Mixer::MAX_TRACKS {
                    return Err(CommandError::Invalid("no room for a mixer track there".into()));
                }
                p.mixer
                    .tracks
                    .insert(usize::from(at), MixerTrack::new(format!("Insert {at}")));
                // Its own route goes to the master, which does not move.
                remap_mixer(p, |i| Some(if i >= at { i + 1 } else { i }));
                return Ok(Created(Some(u32::from(at))));
            }
            RemoveMixerTrack(track) => {
                if track == 0 {
                    return Err(CommandError::Invalid("the master cannot be deleted".into()));
                }
                mixer_track(p, track)?;
                p.mixer.tracks.remove(usize::from(track));
                let gone: Vec<AutomationId> = p
                    .automations
                    .iter()
                    .filter(|a| a.target.mixer_track() == Some(track))
                    .map(|a| a.id)
                    .collect();
                p.automations.retain(|a| !gone.contains(&a.id));
                p.playlist
                    .clips
                    .retain(|c| c.automation().is_none_or(|a| !gone.contains(&a)));
                remap_mixer(p, |i| match i.cmp(&track) {
                    std::cmp::Ordering::Less => Some(i),
                    std::cmp::Ordering::Equal => None,
                    std::cmp::Ordering::Greater => Some(i - 1),
                });
            }
            ResetMixerTrack(track) => {
                let t = mixer_track(p, track)?;
                let fresh = MixerTrack::new("");
                t.volume = fresh.volume;
                t.pan = fresh.pan;
                t.muted = false;
                t.solo = false;
                t.eq = fresh.eq;
                t.color = None;
            }
            ResetMixerRouting(track) => {
                if track != 0 {
                    mixer_track(p, track)?.routes = vec![Route::new(0, 1.0)];
                }
            }
            UnrouteMixerTrack(track) => {
                mixer_track(p, track)?;
                for t in p.mixer.tracks.iter_mut().skip(1) {
                    t.routes.retain(|r| r.to != track);
                    if t.routes.is_empty() {
                        t.routes.push(Route::new(0, 1.0));
                    }
                }
            }
            SetMixerTrackColor { track, color } => mixer_track(p, track)?.color = color,
            SetEffectParams { track, effect, params } => {
                let e = mixer_track(p, track)?
                    .effects
                    .iter_mut()
                    .find(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?;
                if params.len() != e.kind.params().len() || params.iter().any(|v| !v.is_finite()) {
                    return Err(CommandError::Invalid("the preset does not fit this effect".into()));
                }
                e.params = params;
                e.sanitize();
            }
            SetRoutePre { from, to, pre } => {
                let r = mixer_track(p, from)?
                    .routes
                    .iter_mut()
                    .find(|r| r.to == to)
                    .ok_or_else(|| CommandError::Invalid(format!("track {from} does not route to {to}")))?;
                r.pre = pre;
            }
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
                crate::lime::sync_track_eq(p, usize::from(track));
            }
            SetMixerMuteLock { track, locked } => mixer_track(p, track)?.mute_locked = locked,
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
                            mixer_track(p, from)?.routes.push(Route::new(to, level));
                        }
                    }
                }
            }
            AddEffect { track, kind } => {
                let slot = |id| crate::effects::EffectSlot::new(id, kind);
                return add_effect(p, track, slot);
            }
            AddPluginEffect { track, plugin } => {
                let slot = |id| crate::effects::EffectSlot::plugin(id, plugin);
                return add_effect(p, track, slot);
            }
            SetPluginState { plugin, state } => {
                if state.len() > crate::plugin::MAX_STATE_BYTES {
                    return Err(CommandError::Invalid("the plugin state is too large".into()));
                }
                p.plugin_mut(plugin)
                    .ok_or_else(|| CommandError::Invalid("that plugin no longer exists".into()))?
                    .state = state;
            }
            SetPluginSidechain { plugin, source } => {
                if usize::from(source) >= p.mixer.tracks.len() {
                    return Err(CommandError::NoMixerTrack(source));
                }
                let reference = p
                    .plugin_mut(plugin)
                    .ok_or_else(|| CommandError::Invalid("that plugin no longer exists".into()))?;
                if crate::lime::LimePlugin::of(reference) != Some(crate::lime::LimePlugin(7)) {
                    return Err(CommandError::Invalid(
                        "this plugin has no Lime sidechain selector".into(),
                    ));
                }
                reference
                    .extra
                    .insert("lime-sidechain-track".into(), serde_json::json!(source));
                let id = b"key"
                    .iter()
                    .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)));
                if let Some(state) = crate::lime::edited_param_state(reference, id, if source == 0 { 0.0 } else { 1.0 })
                {
                    reference.state = state;
                }
            }
            SetPluginSeparateProcess { plugin, on } => {
                p.plugin_mut(plugin)
                    .ok_or_else(|| CommandError::Invalid("that plugin no longer exists".into()))?
                    .separate_process = on;
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
            SetEffectMix { track, effect, mix } => {
                let mix = finite(mix)?.clamp(0.0, 1.0);
                mixer_track(p, track)?
                    .effects
                    .iter_mut()
                    .find(|e| e.id == effect)
                    .ok_or(CommandError::NoEffect(effect))?
                    .mix = mix;
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

/// A note property the graph editor edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteProp {
    Velocity,
    Pan,
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
        fade_in: 0,
        fade_out: 0,
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
            pan: 0.0,
        });
    }
    let pat = pattern(p, pid)?;
    pat.notes.extend(new);
    sort_notes(pat);
    Ok(Created(last))
}

fn pattern_index(p: &Project, id: PatternId) -> Result<usize, CommandError> {
    p.patterns
        .iter()
        .position(|x| x.id == id)
        .ok_or(CommandError::NoPattern(id))
}

/// FL Studio numbers unnamed patterns by position; LimeShot's default names ("Pattern 3")
/// follow the same rule when patterns are inserted, moved or deleted.
fn renumber_default_names(p: &mut Project) {
    for (i, pat) in p.patterns.iter_mut().enumerate() {
        let default = pat
            .name
            .strip_prefix("Pattern ")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if default {
            pat.name = format!("Pattern {}", i + 1);
        }
    }
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
