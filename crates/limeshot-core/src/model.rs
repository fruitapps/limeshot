//! The project document: plain data, serialisable, no audio buffers.
//!
//! The Channel Rack (`channels`) is shared by every pattern. A pattern holds notes for any
//! channel; step-sequencer steps are ordinary notes on the step grid. Playlist clips reference a
//! pattern by id, so every clip of a pattern plays the same data until it is made unique. The
//! mixer's track 0 is the master; channels route to a mixer track by index.
//!
//! Structs that users may extend in future versions keep unknown JSON fields in `extra`, so a file
//! saved by a newer LimeShot is not silently stripped by an older one.

use crate::time::{STEP_TICKS, Tempo, Tick, TimeSignature};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// Unknown fields preserved on load and written back on save.
pub type Extra = Map<String, Value>;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u32);
    };
}

id_type!(ChannelId);
id_type!(PatternId);
id_type!(ClipId);
id_type!(NoteId);

/// An sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Colors handed out to new channels and patterns, in order. Chosen for LimeShot.
pub const PALETTE: [Rgb; 10] = [
    Rgb(0x9C, 0xD8, 0x3A),
    Rgb(0x4F, 0xB3, 0xD9),
    Rgb(0xE0, 0x8A, 0x3C),
    Rgb(0xC4, 0x6B, 0xD6),
    Rgb(0xE0, 0x5C, 0x6E),
    Rgb(0x5C, 0xC9, 0x9E),
    Rgb(0xD9, 0xC1, 0x4F),
    Rgb(0x7A, 0x8C, 0xE8),
    Rgb(0xB0, 0x86, 0x60),
    Rgb(0x8E, 0x9A, 0xA6),
];

/// The oscillator shape of the built-in synthesizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Square,
    #[default]
    Saw,
    Triangle,
}

impl Waveform {
    pub const ALL: [Waveform; 4] = [Waveform::Sine, Waveform::Square, Waveform::Saw, Waveform::Triangle];

    pub fn label(self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Square => "Square",
            Waveform::Saw => "Saw",
            Waveform::Triangle => "Triangle",
        }
    }
}

/// Where a sample file lives. `path` is relative to the project file when the sample sits next
/// to it (or below it), otherwise absolute.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SampleRef {
    pub path: String,
}

/// Settings of the built-in sampler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SamplerParams {
    pub sample: Option<SampleRef>,
    pub gain_db: f32,
    /// Transposition in semitones (fractional values detune).
    pub pitch: f32,
    /// Playback start, as a fraction of the sample length.
    pub start: f32,
    /// Playback end, as a fraction of the sample length.
    pub end: f32,
    pub reverse: bool,
    /// Release (fade-out) after note-off, in milliseconds.
    pub release_ms: f32,
    /// Play the whole sample regardless of note length (one-shot, as drums usually want).
    pub one_shot: bool,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            sample: None,
            gain_db: 0.0,
            pitch: 0.0,
            start: 0.0,
            end: 1.0,
            reverse: false,
            release_ms: 10.0,
            one_shot: true,
        }
    }
}

/// Settings of the built-in synthesizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SynthParams {
    pub waveform: Waveform,
    pub attack_ms: f32,
    pub decay_ms: f32,
    /// Sustain level, 0–1.
    pub sustain: f32,
    pub release_ms: f32,
    pub gain_db: f32,
    /// Octave offset, −3 to +3.
    pub octave: i8,
    /// Detune of a second oscillator in cents; 0 disables it.
    pub detune_cents: f32,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Saw,
            attack_ms: 5.0,
            decay_ms: 200.0,
            sustain: 0.7,
            release_ms: 150.0,
            gain_db: -6.0,
            octave: 0,
            detune_cents: 0.0,
        }
    }
}

/// The sound source of a channel.
#[derive(Debug, Clone, PartialEq)]
pub enum Instrument {
    Sampler(SamplerParams),
    Synth(SynthParams),
    /// An instrument this version does not know, kept verbatim so saving does not lose it. It is
    /// silent.
    Unsupported(Value),
}

impl Instrument {
    pub fn kind_label(&self) -> &'static str {
        match self {
            Instrument::Sampler(_) => "Sampler",
            Instrument::Synth(_) => "LimeSynth",
            Instrument::Unsupported(_) => "Unsupported",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum KnownInstrument {
    Sampler(SamplerParams),
    Synth(SynthParams),
}

impl Serialize for Instrument {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Instrument::Sampler(p) => KnownInstrument::Sampler(p.clone()).serialize(s),
            Instrument::Synth(p) => KnownInstrument::Synth(p.clone()).serialize(s),
            Instrument::Unsupported(v) => v.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Instrument {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        let known = matches!(v.get("type").and_then(Value::as_str), Some("Sampler" | "Synth"));
        if !known {
            return Ok(Instrument::Unsupported(v));
        }
        match serde_json::from_value::<KnownInstrument>(v).map_err(serde::de::Error::custom)? {
            KnownInstrument::Sampler(p) => Ok(Instrument::Sampler(p)),
            KnownInstrument::Synth(p) => Ok(Instrument::Synth(p)),
        }
    }
}

/// A Channel Rack channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub color: Rgb,
    pub instrument: Instrument,
    /// Linear gain, 0–1.25 (1.0 = unity).
    pub volume: f32,
    /// −1 (left) to +1 (right).
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    /// Mixer track index; 0 is the master.
    pub mixer_track: u16,
    /// The MIDI key that plays the sample unpitched and that new steps use (60 = C5).
    pub root_key: u8,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A note in a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub channel: ChannelId,
    /// MIDI key, 0–127.
    pub key: u8,
    pub start: Tick,
    pub length: Tick,
    /// 0–1.
    pub velocity: f32,
}

impl Note {
    pub fn end(&self) -> Tick {
        self.start.saturating_add(self.length)
    }
}

/// A reusable musical container of notes for any channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub id: PatternId,
    pub name: String,
    pub color: Rgb,
    /// Length in steps; `None` = automatic (whole bars covering the last note, at least one bar).
    #[serde(default)]
    pub length_steps: Option<u32>,
    pub notes: Vec<Note>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Pattern {
    /// The pattern length in ticks.
    pub fn length(&self, sig: TimeSignature) -> Tick {
        if let Some(steps) = self.length_steps {
            return Tick::from(steps.clamp(1, 512)) * STEP_TICKS;
        }
        let bar = sig.ticks_per_bar();
        let last = self.notes.iter().map(|n| n.start + 1).max().unwrap_or(0);
        last.div_ceil(bar).max(1) * bar
    }

    pub fn notes_for(&self, channel: ChannelId) -> impl Iterator<Item = &Note> {
        self.notes.iter().filter(move |n| n.channel == channel)
    }

    /// Whether `channel`'s notes in this pattern are all plain steps (root key, on the step grid,
    /// at most one step long), so the Channel Rack can show them as step buttons.
    pub fn is_step_sequence(&self, channel: ChannelId, root_key: u8) -> bool {
        self.notes_for(channel)
            .all(|n| n.key == root_key && n.start % STEP_TICKS == 0 && n.length <= STEP_TICKS && n.length > 0)
    }

    /// The step note of `channel` at `step`, if any.
    pub fn step_note(&self, channel: ChannelId, root_key: u8, step: u32) -> Option<&Note> {
        let t = Tick::from(step) * STEP_TICKS;
        self.notes_for(channel).find(|n| n.start == t && n.key == root_key)
    }
}

/// What a playlist clip plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipSource {
    Pattern(PatternId),
}

/// A clip on the Playlist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub source: ClipSource,
    /// Playlist track index.
    pub track: u32,
    pub start: Tick,
    pub length: Tick,
    /// Where in the source the clip begins.
    #[serde(default)]
    pub offset: Tick,
    #[serde(default)]
    pub muted: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Clip {
    pub fn end(&self) -> Tick {
        self.start.saturating_add(self.length)
    }

    pub fn pattern(&self) -> PatternId {
        match self.source {
            ClipSource::Pattern(p) => p,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub name: String,
    #[serde(default)]
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub tracks: Vec<PlaylistTrack>,
    pub clips: Vec<Clip>,
}

impl Playlist {
    /// The end of the last unmuted clip, rounded up to whole bars (0 when empty).
    pub fn song_length(&self, sig: TimeSignature) -> Tick {
        let end = self.clips.iter().map(Clip::end).max().unwrap_or(0);
        let bar = sig.ticks_per_bar();
        end.div_ceil(bar) * bar
    }
}

/// A mixer track (insert, or the master at index 0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixerTrack {
    pub name: String,
    /// Linear gain (1.0 = 0 dB, max 2.0 ≈ +6 dB).
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

impl MixerTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: 0.8,
            pan: 0.0,
            muted: false,
            solo: false,
            extra: Extra::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mixer {
    pub tracks: Vec<MixerTrack>,
}

impl Mixer {
    pub const DEFAULT_INSERTS: usize = 16;
    pub const MAX_TRACKS: usize = 128;
}

impl Default for Mixer {
    fn default() -> Self {
        let mut tracks = vec![MixerTrack {
            volume: 1.0,
            ..MixerTrack::new("Master")
        }];
        tracks.extend((1..=Self::DEFAULT_INSERTS).map(|i| MixerTrack::new(format!("Insert {i}"))));
        Self { tracks }
    }
}

/// The whole document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub channels: Vec<Channel>,
    pub patterns: Vec<Pattern>,
    pub playlist: Playlist,
    pub mixer: Mixer,
    /// Global swing, 0–1: delays every second 16th note by up to half a step (about 0.67 gives a
    /// triplet feel). FL Studio's Channel Rack swing knob.
    #[serde(default)]
    pub swing: f32,
    /// Next free id for channels, patterns, clips and notes.
    pub next_id: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    /// An empty project at 140 BPM: one pattern, a playlist with 16 tracks, a mixer with 16 inserts.
    pub fn new() -> Self {
        let mut p = Self {
            title: "Untitled".into(),
            // FL Studio starts new projects at 140 BPM (observed, docs/research/fl-studio-analysis.md).
            tempo: Tempo::from_milli_bpm(140_000),
            time_signature: TimeSignature::default(),
            channels: Vec::new(),
            patterns: Vec::new(),
            playlist: Playlist {
                tracks: (1..=16)
                    .map(|i| PlaylistTrack {
                        name: format!("Track {i}"),
                        muted: false,
                    })
                    .collect(),
                clips: Vec::new(),
            },
            mixer: Mixer::default(),
            swing: 0.0,
            next_id: 1,
            extra: Extra::new(),
        };
        let id = PatternId(p.alloc_id());
        p.patterns.push(Pattern {
            id,
            name: "Pattern 1".into(),
            color: PALETTE[0],
            length_steps: None,
            notes: Vec::new(),
            extra: Extra::new(),
        });
        p
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    pub fn channel(&self, id: ChannelId) -> Option<&Channel> {
        self.channels.iter().find(|c| c.id == id)
    }

    pub fn channel_mut(&mut self, id: ChannelId) -> Option<&mut Channel> {
        self.channels.iter_mut().find(|c| c.id == id)
    }

    pub fn channel_index(&self, id: ChannelId) -> Option<usize> {
        self.channels.iter().position(|c| c.id == id)
    }

    pub fn pattern(&self, id: PatternId) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    pub fn pattern_mut(&mut self, id: PatternId) -> Option<&mut Pattern> {
        self.patterns.iter_mut().find(|p| p.id == id)
    }

    pub fn clip(&self, id: ClipId) -> Option<&Clip> {
        self.playlist.clips.iter().find(|c| c.id == id)
    }

    pub fn clip_mut(&mut self, id: ClipId) -> Option<&mut Clip> {
        self.playlist.clips.iter_mut().find(|c| c.id == id)
    }

    /// The next free insert (not master, not targeted by any channel), if any.
    pub fn free_mixer_track(&self) -> Option<u16> {
        (1..self.mixer.tracks.len())
            .filter_map(|i| u16::try_from(i).ok())
            .find(|i| self.channels.iter().all(|c| c.mixer_track != *i))
    }

    /// Every sample path the project references.
    pub fn sample_refs(&self) -> impl Iterator<Item = &SampleRef> {
        self.channels.iter().filter_map(|c| match &c.instrument {
            Instrument::Sampler(p) => p.sample.as_ref(),
            _ => None,
        })
    }

    /// Repair values that could come from a hand-edited or hostile file, so the rest of the
    /// program can trust them: ranges are clamped, notes sorted, dangling references dropped and
    /// `next_id` moved past every id in use.
    pub fn sanitize(&mut self) {
        if self.mixer.tracks.is_empty() {
            self.mixer = Mixer::default();
        }
        self.swing = finite(self.swing, 0.0).clamp(0.0, 1.0);
        self.mixer.tracks.truncate(Mixer::MAX_TRACKS);
        let tracks = self.mixer.tracks.len();
        for t in &mut self.mixer.tracks {
            t.volume = finite(t.volume, 0.8).clamp(0.0, 2.0);
            t.pan = finite(t.pan, 0.0).clamp(-1.0, 1.0);
        }
        let mut max_id = 0;
        for c in &mut self.channels {
            c.volume = finite(c.volume, 0.78).clamp(0.0, 1.25);
            c.pan = finite(c.pan, 0.0).clamp(-1.0, 1.0);
            c.root_key = c.root_key.min(127);
            if usize::from(c.mixer_track) >= tracks {
                c.mixer_track = 0;
            }
            match &mut c.instrument {
                Instrument::Sampler(p) => {
                    p.gain_db = finite(p.gain_db, 0.0).clamp(-60.0, 24.0);
                    p.pitch = finite(p.pitch, 0.0).clamp(-48.0, 48.0);
                    p.start = finite(p.start, 0.0).clamp(0.0, 1.0);
                    p.end = finite(p.end, 1.0).clamp(p.start, 1.0);
                    p.release_ms = finite(p.release_ms, 10.0).clamp(0.0, 10_000.0);
                }
                Instrument::Synth(p) => {
                    p.attack_ms = finite(p.attack_ms, 5.0).clamp(0.0, 10_000.0);
                    p.decay_ms = finite(p.decay_ms, 200.0).clamp(0.0, 10_000.0);
                    p.sustain = finite(p.sustain, 0.7).clamp(0.0, 1.0);
                    p.release_ms = finite(p.release_ms, 150.0).clamp(0.0, 10_000.0);
                    p.gain_db = finite(p.gain_db, -6.0).clamp(-60.0, 24.0);
                    p.octave = p.octave.clamp(-3, 3);
                    p.detune_cents = finite(p.detune_cents, 0.0).clamp(-100.0, 100.0);
                }
                Instrument::Unsupported(_) => {}
            }
            max_id = max_id.max(c.id.0);
        }
        let channels: Vec<ChannelId> = self.channels.iter().map(|c| c.id).collect();
        if self.patterns.is_empty() {
            self.patterns.push(Pattern {
                id: PatternId(0),
                name: "Pattern 1".into(),
                color: PALETTE[0],
                length_steps: None,
                notes: Vec::new(),
                extra: Extra::new(),
            });
        }
        for p in &mut self.patterns {
            p.length_steps = p.length_steps.map(|s| s.clamp(1, 512));
            p.notes.retain(|n| channels.contains(&n.channel));
            for n in &mut p.notes {
                n.key = n.key.min(127);
                n.length = n.length.max(1);
                n.velocity = finite(n.velocity, 0.8).clamp(0.0, 1.0);
                max_id = max_id.max(n.id.0);
            }
            p.notes.sort_by_key(|n| (n.start, n.key, n.id));
            max_id = max_id.max(p.id.0);
        }
        let patterns: Vec<PatternId> = self.patterns.iter().map(|p| p.id).collect();
        self.playlist.clips.retain(|c| patterns.contains(&c.pattern()));
        let max_track = self.playlist.clips.iter().map(|c| c.track).max().unwrap_or(0);
        while self.playlist.tracks.len() <= max_track as usize && self.playlist.tracks.len() < 500 {
            let n = self.playlist.tracks.len() + 1;
            self.playlist.tracks.push(PlaylistTrack {
                name: format!("Track {n}"),
                muted: false,
            });
        }
        self.playlist
            .clips
            .retain(|c| (c.track as usize) < self.playlist.tracks.len());
        for c in &mut self.playlist.clips {
            c.length = c.length.max(1);
            max_id = max_id.max(c.id.0);
        }
        self.next_id = self.next_id.max(max_id.saturating_add(1));
        // A pattern id of 0 can only come from the repair above; give it a real id.
        if let Some(p) = self.patterns.iter_mut().find(|p| p.id.0 == 0) {
            p.id = PatternId(self.next_id);
            self.next_id = self.next_id.saturating_add(1);
        }
    }
}

fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_has_one_pattern_and_master() {
        let p = Project::new();
        assert_eq!(p.patterns.len(), 1);
        assert_eq!(p.mixer.tracks.len(), 17);
        assert_eq!(p.mixer.tracks[0].name, "Master");
    }

    #[test]
    fn auto_pattern_length_covers_last_note() {
        let mut p = Project::new();
        let sig = p.time_signature;
        assert_eq!(p.patterns[0].length(sig), 3840);
        p.patterns[0].notes.push(Note {
            id: NoteId(9),
            channel: ChannelId(1),
            key: 60,
            start: 3840,
            length: 10,
            velocity: 1.0,
        });
        assert_eq!(p.patterns[0].length(sig), 7680);
        p.patterns[0].length_steps = Some(8);
        assert_eq!(p.patterns[0].length(sig), 1920);
    }

    #[test]
    fn unknown_instrument_round_trips() {
        let raw = serde_json::json!({"type": "Granulator", "grains": 32});
        let i: Instrument = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(i, Instrument::Unsupported(_)));
        assert_eq!(serde_json::to_value(&i).unwrap(), raw);
    }

    #[test]
    fn sanitize_repairs_hostile_values() {
        let mut p = Project::new();
        p.channels.push(Channel {
            id: ChannelId(50),
            name: "x".into(),
            color: PALETTE[0],
            instrument: Instrument::Synth(SynthParams {
                sustain: f32::NAN,
                ..SynthParams::default()
            }),
            volume: 99.0,
            pan: f32::INFINITY,
            muted: false,
            solo: false,
            mixer_track: 999,
            root_key: 200,
            extra: Extra::new(),
        });
        p.patterns[0].notes.push(Note {
            id: NoteId(70),
            channel: ChannelId(404),
            key: 60,
            start: 0,
            length: 0,
            velocity: 1.0,
        });
        p.sanitize();
        let c = &p.channels[0];
        assert_eq!(c.volume, 1.25);
        assert_eq!(c.pan, 0.0);
        assert_eq!(c.mixer_track, 0);
        assert_eq!(c.root_key, 127);
        assert!(p.patterns[0].notes.is_empty());
        assert!(p.next_id > 50);
    }
}
