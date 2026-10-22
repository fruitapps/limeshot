//! The project document: plain data, serialisable, no audio buffers.
//!
//! The Channel Rack (`channels`) is shared by every pattern. A pattern holds notes for any
//! channel; step-sequencer steps are ordinary notes on the step grid. Playlist clips reference a
//! pattern by id, so every clip of a pattern plays the same data until it is made unique. The
//! mixer's track 0 is the master; channels route to a mixer track by index.
//!
//! Structs that users may extend in future versions keep unknown JSON fields in `extra`, so a file
//! saved by a newer LimeShot is not silently stripped by an older one.

use crate::automation::{Automation, AutomationId};
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
id_type!(MarkerId);

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
    /// Volume envelope (FL Studio's sampler envelope): attack and decay in milliseconds, sustain
    /// level 0–1, and the release (fade-out) after note-off.
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain: f32,
    pub release_ms: f32,
    /// Play the whole sample regardless of note length (one-shot, as drums usually want).
    pub one_shot: bool,
    /// Loop `loop_start..loop_end` (fractions of the sample) while the note is held (FL Studio's
    /// sampler loop). A looping sample follows the note length even when one-shot is on.
    pub looping: bool,
    pub loop_start: f32,
    pub loop_end: f32,
    /// Low-pass filter cutoff in Hz (20 000 = open) and resonance 0–0.95.
    pub cutoff_hz: f32,
    pub resonance: f32,
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
            attack_ms: 0.0,
            decay_ms: 0.0,
            sustain: 1.0,
            release_ms: 10.0,
            one_shot: true,
            looping: false,
            loop_start: 0.0,
            loop_end: 1.0,
            cutoff_hz: 20_000.0,
            resonance: 0.0,
        }
    }
}

/// LimeSynth's filter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FilterMode {
    #[default]
    LowPass,
    BandPass,
    HighPass,
}

impl FilterMode {
    pub const ALL: [FilterMode; 3] = [FilterMode::LowPass, FilterMode::BandPass, FilterMode::HighPass];

    pub fn label(self) -> &'static str {
        match self {
            FilterMode::LowPass => "Low-pass",
            FilterMode::BandPass => "Band-pass",
            FilterMode::HighPass => "High-pass",
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
    /// Low-pass cutoff in Hz (20 000 = open).
    pub cutoff_hz: f32,
    /// Filter resonance, 0–0.95.
    pub resonance: f32,
    /// How far the filter envelope opens the cutoff at the note start, in octaves (0–8).
    pub filter_env: f32,
    /// Time for the filter envelope to fall back (exponential decay to about 1/1000).
    pub filter_decay_ms: f32,
    /// LFO rate in Hz.
    pub lfo_rate_hz: f32,
    /// LFO to pitch (vibrato depth) in cents.
    pub vibrato_cents: f32,
    /// LFO to cutoff, in octaves.
    pub lfo_cutoff: f32,
    /// Which part of the spectrum the filter keeps (FL Studio's synths: low-, band- or
    /// high-pass).
    pub filter_mode: FilterMode,
    /// How much the cutoff follows the played key (0: fixed, 1: an octave per octave, around C5).
    pub key_track: f32,
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
            cutoff_hz: 20_000.0,
            resonance: 0.0,
            filter_env: 0.0,
            filter_decay_ms: 300.0,
            lfo_rate_hz: 5.0,
            vibrato_cents: 0.0,
            lfo_cutoff: 0.0,
            filter_mode: FilterMode::LowPass,
            key_track: 0.0,
        }
    }
}

/// The sound source of a channel.
#[derive(Debug, Clone, PartialEq)]
pub enum Instrument {
    Sampler(SamplerParams),
    Synth(SynthParams),
    /// A hosted plugin instrument (VST3).
    Plugin(crate::PluginRef),
    /// An instrument this version does not know, kept verbatim so saving does not lose it. It is
    /// silent.
    Unsupported(Value),
}

impl Instrument {
    pub fn kind_label(&self) -> &'static str {
        match self {
            Instrument::Sampler(_) => "Sampler",
            Instrument::Synth(_) => "LimeSynth",
            Instrument::Plugin(_) => "Plugin",
            Instrument::Unsupported(_) => "Unsupported",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum KnownInstrument {
    Sampler(SamplerParams),
    Synth(SynthParams),
    Plugin(crate::PluginRef),
}

impl Serialize for Instrument {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Instrument::Sampler(p) => KnownInstrument::Sampler(p.clone()).serialize(s),
            Instrument::Synth(p) => KnownInstrument::Synth(p.clone()).serialize(s),
            Instrument::Plugin(p) => KnownInstrument::Plugin(p.clone()).serialize(s),
            Instrument::Unsupported(v) => v.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Instrument {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        let known = matches!(
            v.get("type").and_then(Value::as_str),
            Some("Sampler" | "Synth" | "Plugin")
        );
        if !known {
            return Ok(Instrument::Unsupported(v));
        }
        match serde_json::from_value::<KnownInstrument>(v).map_err(serde::de::Error::custom)? {
            KnownInstrument::Sampler(p) => Ok(Instrument::Sampler(p)),
            KnownInstrument::Synth(p) => Ok(Instrument::Synth(p)),
            KnownInstrument::Plugin(p) => Ok(Instrument::Plugin(p)),
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
    #[serde(default)]
    pub mute_locked: bool,
    /// Mixer track index; 0 is the master.
    pub mixer_track: u16,
    /// The MIDI key that plays the sample unpitched and that new steps use (60 = C5).
    pub root_key: u8,
    /// The Channel Rack filter group (FL Studio's display filter groups); `None` is "Unsorted".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
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
    /// A muted note stays in the pattern but does not play (FL Studio's Mute tool).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub muted: bool,
    /// -1 (left) to 1 (right), added to the channel's pan (FL Studio's note panning, edited in
    /// the graph editor). Saved only when set.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub pan: f32,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
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
    /// The sample of a sampler channel, played from the clip's `offset` for the clip's length.
    /// As in FL Studio, a sample dropped on the Playlist becomes an audio-clip channel in the
    /// Channel Rack; the channel's volume, pan, sampler settings and mixer track apply.
    Audio(ChannelId),
    /// An automation curve; clips of the same automation are linked.
    Automation(AutomationId),
}

impl ClipSource {
    pub fn pattern_id(self) -> Option<PatternId> {
        match self {
            ClipSource::Pattern(p) => Some(p),
            _ => None,
        }
    }
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
    /// Audio clips: fade-in and fade-out lengths in ticks (FL Studio's clip fade handles).
    /// Saved only when set.
    #[serde(default, skip_serializing_if = "is_zero_tick")]
    pub fade_in: Tick,
    #[serde(default, skip_serializing_if = "is_zero_tick")]
    pub fade_out: Tick,
    #[serde(flatten)]
    pub extra: Extra,
}

fn is_zero_tick(t: &Tick) -> bool {
    *t == 0
}

impl Clip {
    pub fn end(&self) -> Tick {
        self.start.saturating_add(self.length)
    }

    /// The pattern a pattern clip plays.
    pub fn pattern(&self) -> Option<PatternId> {
        match self.source {
            ClipSource::Pattern(p) => Some(p),
            _ => None,
        }
    }

    /// The automation an automation clip plays.
    pub fn automation(&self) -> Option<AutomationId> {
        match self.source {
            ClipSource::Automation(a) => Some(a),
            _ => None,
        }
    }

    /// The channel an audio clip plays.
    pub fn audio_channel(&self) -> Option<ChannelId> {
        match self.source {
            ClipSource::Audio(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub name: String,
    #[serde(default)]
    pub color: Option<Rgb>,
    #[serde(default)]
    pub mute_locked: bool,
    #[serde(default)]
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub tracks: Vec<PlaylistTrack>,
    pub clips: Vec<Clip>,
    /// Time markers on the Playlist ruler, sorted by time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<Marker>,
}

/// A named point on the Playlist's timeline (FL Studio's time marker).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub id: MarkerId,
    pub tick: Tick,
    pub name: String,
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
    /// Insert effects, processed in order before the fader.
    #[serde(default)]
    pub effects: Vec<crate::effects::EffectSlot>,
    /// Where this track's (post-fader) output goes. Inserts route to the master by default; the
    /// master has no routes (it feeds the audio device). FL Studio's mixer routing.
    #[serde(default = "default_routes")]
    pub routes: Vec<Route>,
    /// The built-in 3-band EQ (FL Studio's track EQ), after the insert effects and before the
    /// fader. Saved only when changed.
    #[serde(default = "default_eq", skip_serializing_if = "is_default_eq")]
    pub eq: [EqBand; 3],
    /// The strip's colour (FL Studio's Change color… / Random color); `None`: the routed
    /// channels' colour. Saved only when set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// One band of a mixer track's EQ: band 0 is a low shelf, 1 a peak, 2 a high shelf.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EqBand {
    /// Hz, 20–20000.
    pub freq: f32,
    /// dB, ±18.
    pub gain: f32,
    /// Bandwidth 0–1 (FL Studio's "width"; 0.27 by default).
    pub width: f32,
}

impl EqBand {
    pub const FREQ: (f32, f32) = (20.0, 20_000.0);
    pub const GAIN: f32 = 18.0;
    /// FL Studio 2026's defaults: low shelf 90 Hz, peak 1500 Hz, high shelf 8000 Hz, all at 0 dB
    /// and width 0.27 (observed in its Mixer).
    pub const DEFAULTS: [EqBand; 3] = [
        EqBand {
            freq: 90.0,
            gain: 0.0,
            width: 0.27,
        },
        EqBand {
            freq: 1500.0,
            gain: 0.0,
            width: 0.27,
        },
        EqBand {
            freq: 8000.0,
            gain: 0.0,
            width: 0.27,
        },
    ];

    /// The band with every value made finite and in range.
    pub fn sanitized(self, band: usize) -> Self {
        let d = Self::DEFAULTS[band.min(2)];
        Self {
            freq: finite(self.freq, d.freq).clamp(Self::FREQ.0, Self::FREQ.1),
            gain: finite(self.gain, 0.0).clamp(-Self::GAIN, Self::GAIN),
            width: finite(self.width, d.width).clamp(0.0, 1.0),
        }
    }
}

fn default_eq() -> [EqBand; 3] {
    EqBand::DEFAULTS
}

fn is_default_eq(eq: &[EqBand; 3]) -> bool {
    *eq == EqBand::DEFAULTS
}

/// A mixer route: send this track's output to mixer track `to` at `level` (linear gain).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Route {
    pub to: u16,
    pub level: f32,
    /// Send before the fader and pan (a pre-fader send); post-fader by default. Saved only when
    /// set.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pre: bool,
}

impl Route {
    pub const fn new(to: u16, level: f32) -> Self {
        Route { to, level, pre: false }
    }
}

fn default_routes() -> Vec<Route> {
    vec![Route::new(0, 1.0)]
}

impl MixerTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            // 0 dB, FL Studio's insert default.
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            effects: Vec::new(),
            routes: default_routes(),
            eq: EqBand::DEFAULTS,
            color: None,
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

    /// Whether routing `from` → `to` would create a feedback loop (or a self/master-out route).
    pub fn would_cycle(&self, from: u16, to: u16) -> bool {
        if from == to || from == 0 {
            return true;
        }
        // Is `from` reachable from `to`?
        let mut stack = vec![to];
        let mut seen = vec![false; self.tracks.len()];
        while let Some(t) = stack.pop() {
            if t == from {
                return true;
            }
            let Some(track) = self.tracks.get(usize::from(t)) else {
                continue;
            };
            if std::mem::replace(&mut seen[usize::from(t)], true) {
                continue;
            }
            stack.extend(track.routes.iter().map(|r| r.to));
        }
        false
    }

    /// Insert tracks in processing order: every track comes before the tracks it routes to.
    /// The master (0) is not included; it is always processed last. Assumes no cycles
    /// (guaranteed by `SetRoute` and `Project::sanitize`).
    pub fn processing_order(&self) -> Vec<usize> {
        let n = self.tracks.len();
        let mut indegree = vec![0usize; n];
        for t in self.tracks.iter().skip(1) {
            for r in &t.routes {
                if let Some(d) = indegree.get_mut(usize::from(r.to)) {
                    *d += 1;
                }
            }
        }
        let mut ready: Vec<usize> = (1..n).filter(|i| indegree[*i] == 0).collect();
        ready.reverse();
        let mut order = Vec::with_capacity(n);
        while let Some(i) = ready.pop() {
            order.push(i);
            for r in &self.tracks[i].routes {
                let to = usize::from(r.to);
                if to == 0 || to >= n {
                    continue;
                }
                indegree[to] -= 1;
                if indegree[to] == 0 {
                    ready.push(to);
                }
            }
        }
        order
    }
}

impl Default for Mixer {
    fn default() -> Self {
        let mut tracks = vec![MixerTrack {
            volume: 1.0,
            routes: Vec::new(),
            ..MixerTrack::new("Master")
        }];
        tracks.extend((1..=Self::DEFAULT_INSERTS).map(|i| MixerTrack::new(format!("Insert {i}"))));
        Self { tracks }
    }
}

/// FL Studio's project credits (Options › Project info): who made the song and notes about it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectInfo {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub author: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub genre: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub comments: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
}

impl ProjectInfo {
    pub fn is_empty(&self) -> bool {
        self == &ProjectInfo::default()
    }

    /// Keep each field to a sane length (hostile files).
    pub fn sanitize(&mut self) {
        for (s, max) in [
            (&mut self.author, 1024),
            (&mut self.genre, 256),
            (&mut self.comments, 64 * 1024),
            (&mut self.url, 4096),
        ] {
            if s.len() > max {
                let mut end = max;
                while !s.is_char_boundary(end) {
                    end -= 1;
                }
                s.truncate(end);
            }
        }
    }
}

/// The whole document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    /// Author, genre, comments and web link.
    #[serde(default, skip_serializing_if = "ProjectInfo::is_empty")]
    pub info: ProjectInfo,
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub channels: Vec<Channel>,
    pub patterns: Vec<Pattern>,
    pub playlist: Playlist,
    pub mixer: Mixer,
    /// Automation curves (FL Studio's automation clips), placed on the Playlist by clips.
    #[serde(default)]
    pub automations: Vec<Automation>,
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
            info: ProjectInfo::default(),
            // FL Studio starts new projects at 140 BPM (observed, docs/research/fl-studio-analysis.md).
            tempo: Tempo::from_milli_bpm(140_000),
            time_signature: TimeSignature::default(),
            channels: Vec::new(),
            patterns: Vec::new(),
            playlist: Playlist {
                tracks: (1..=16)
                    .map(|i| PlaylistTrack {
                        color: None,
                        mute_locked: false,
                        name: format!("Track {i}"),
                        muted: false,
                    })
                    .collect(),
                clips: Vec::new(),
                markers: Vec::new(),
            },
            mixer: Mixer::default(),
            automations: Vec::new(),
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

    /// The plugin a channel or insert slot runs, to change.
    pub fn plugin_mut(&mut self, target: crate::PluginTarget) -> Option<&mut crate::PluginRef> {
        match target {
            crate::PluginTarget::Channel(c) => match &mut self.channels.iter_mut().find(|ch| ch.id == c)?.instrument {
                Instrument::Plugin(p) => Some(p),
                _ => None,
            },
            crate::PluginTarget::Effect { track, effect } => self
                .mixer
                .tracks
                .get_mut(usize::from(track))?
                .effects
                .iter_mut()
                .find(|e| e.id == effect)?
                .plugin
                .as_mut(),
        }
    }

    /// The plugin a channel or insert slot runs.
    pub fn plugin(&self, target: crate::PluginTarget) -> Option<&crate::PluginRef> {
        match target {
            crate::PluginTarget::Channel(c) => match &self.channel(c)?.instrument {
                Instrument::Plugin(p) => Some(p),
                _ => None,
            },
            crate::PluginTarget::Effect { track, effect } => self
                .mixer
                .tracks
                .get(usize::from(track))?
                .effects
                .iter()
                .find(|e| e.id == effect)?
                .plugin
                .as_ref(),
        }
    }

    /// The Channel Rack filter groups in use, sorted.
    pub fn channel_groups(&self) -> Vec<String> {
        let mut g: Vec<String> = self.channels.iter().filter_map(|c| c.group.clone()).collect();
        g.sort();
        g.dedup();
        g
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

    pub fn automation(&self, id: AutomationId) -> Option<&Automation> {
        self.automations.iter().find(|a| a.id == id)
    }

    pub fn automation_mut(&mut self, id: AutomationId) -> Option<&mut Automation> {
        self.automations.iter_mut().find(|a| a.id == id)
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
        self.info.sanitize();
        if self.mixer.tracks.is_empty() {
            self.mixer = Mixer::default();
        }
        self.swing = finite(self.swing, 0.0).clamp(0.0, 1.0);
        self.mixer.tracks.truncate(Mixer::MAX_TRACKS);
        let tracks = self.mixer.tracks.len();
        let mut max_id = 0;
        for t in &mut self.mixer.tracks {
            t.volume = finite(t.volume, 1.0).clamp(0.0, 2.0);
            t.pan = finite(t.pan, 0.0).clamp(-1.0, 1.0);
            for (i, b) in t.eq.iter_mut().enumerate() {
                *b = b.sanitized(i);
            }
            t.effects.truncate(crate::effects::MAX_EFFECTS);
            for e in &mut t.effects {
                e.sanitize();
                max_id = max_id.max(e.id);
            }
        }
        // Routing: drop invalid routes, then re-add the rest one by one, skipping any that would
        // close a feedback loop.
        let wanted: Vec<Vec<Route>> = self.mixer.tracks.iter().map(|t| t.routes.clone()).collect();
        for t in &mut self.mixer.tracks {
            t.routes.clear();
        }
        for (from, routes) in wanted.into_iter().enumerate().skip(1) {
            for r in routes {
                let Ok(from16) = u16::try_from(from) else { continue };
                let valid = usize::from(r.to) < tracks
                    && !self.mixer.tracks[from].routes.iter().any(|x| x.to == r.to)
                    && !self.mixer.would_cycle(from16, r.to);
                if valid {
                    let level = finite(r.level, 1.0).clamp(0.0, 2.0);
                    self.mixer.tracks[from].routes.push(Route::new(r.to, level));
                }
            }
        }
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
                    p.attack_ms = finite(p.attack_ms, 0.0).clamp(0.0, 10_000.0);
                    p.decay_ms = finite(p.decay_ms, 0.0).clamp(0.0, 10_000.0);
                    p.sustain = finite(p.sustain, 1.0).clamp(0.0, 1.0);
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
                    p.cutoff_hz = finite(p.cutoff_hz, 20_000.0).clamp(20.0, 20_000.0);
                    p.resonance = finite(p.resonance, 0.0).clamp(0.0, 0.95);
                    p.filter_env = finite(p.filter_env, 0.0).clamp(0.0, 8.0);
                    p.filter_decay_ms = finite(p.filter_decay_ms, 300.0).clamp(1.0, 10_000.0);
                    p.lfo_rate_hz = finite(p.lfo_rate_hz, 5.0).clamp(0.01, 40.0);
                    p.vibrato_cents = finite(p.vibrato_cents, 0.0).clamp(0.0, 200.0);
                    p.lfo_cutoff = finite(p.lfo_cutoff, 0.0).clamp(0.0, 4.0);
                }
                Instrument::Plugin(p) => p.sanitize(),
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
                n.pan = finite(n.pan, 0.0).clamp(-1.0, 1.0);
                max_id = max_id.max(n.id.0);
            }
            p.notes.sort_by_key(|n| (n.start, n.key, n.id));
            max_id = max_id.max(p.id.0);
        }
        for a in &mut self.automations {
            Automation::normalize_points(&mut a.points);
            a.length = a.length.max(1);
            max_id = max_id.max(a.id.0);
        }
        let patterns: Vec<PatternId> = self.patterns.iter().map(|p| p.id).collect();
        let automations: Vec<AutomationId> = self.automations.iter().map(|a| a.id).collect();
        self.playlist.clips.retain(|c| match c.source {
            ClipSource::Pattern(p) => patterns.contains(&p),
            ClipSource::Audio(ch) => channels.contains(&ch),
            ClipSource::Automation(a) => automations.contains(&a),
        });
        let max_track = self.playlist.clips.iter().map(|c| c.track).max().unwrap_or(0);
        while self.playlist.tracks.len() <= max_track as usize && self.playlist.tracks.len() < 500 {
            let n = self.playlist.tracks.len() + 1;
            self.playlist.tracks.push(PlaylistTrack {
                color: None,
                mute_locked: false,
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
        self.playlist.markers.truncate(10_000);
        self.playlist.markers.sort_by_key(|m| m.tick);
        for m in &self.playlist.markers {
            max_id = max_id.max(m.id.0);
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
            muted: false,
            pan: 0.0,
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
    fn note_pan_is_saved_only_when_set() {
        let mut n = Note {
            id: NoteId(1),
            channel: ChannelId(2),
            key: 60,
            start: 0,
            length: 240,
            velocity: 0.78,
            muted: false,
            pan: 0.0,
        };
        assert!(serde_json::to_value(n).unwrap().get("pan").is_none());
        n.pan = -0.5;
        let back: Note = serde_json::from_str(&serde_json::to_string(&n).unwrap()).unwrap();
        assert_eq!(back.pan, -0.5);
    }

    #[test]
    fn track_eq_is_saved_only_when_changed() {
        let mut t = MixerTrack::new("Insert 1");
        let json = serde_json::to_value(&t).unwrap();
        assert!(json.get("eq").is_none(), "a flat EQ is not written");
        t.eq[1].gain = -3.0;
        let json = serde_json::to_string(&t).unwrap();
        let back: MixerTrack = serde_json::from_str(&json).unwrap();
        assert_eq!(back.eq, t.eq);
        // Files from before the EQ load with the defaults.
        let old: MixerTrack =
            serde_json::from_str(r#"{"name":"A","volume":1.0,"pan":0.0,"muted":false,"solo":false}"#).unwrap();
        assert_eq!(old.eq, EqBand::DEFAULTS);
        // Hostile values are repaired.
        let mut p = Project::new();
        p.mixer.tracks[1].eq[0] = EqBand {
            freq: f32::NAN,
            gain: f32::INFINITY,
            width: -1.0,
        };
        p.sanitize();
        assert_eq!(
            p.mixer.tracks[1].eq[0],
            EqBand {
                freq: 90.0,
                gain: 0.0,
                width: 0.0
            }
        );
    }

    #[test]
    fn sampler_envelope_defaults_round_trip_and_sanitize() {
        // Older projects and presets omit the new stages and must retain the original onset.
        let old: SamplerParams = serde_json::from_str(r#"{"release_ms":25.0}"#).unwrap();
        assert_eq!((old.attack_ms, old.decay_ms, old.sustain), (0.0, 0.0, 1.0));
        assert_eq!(old.release_ms, 25.0);
        let settings = SamplerParams {
            attack_ms: 20.0,
            decay_ms: 100.0,
            sustain: 0.4,
            ..old
        };
        let saved = serde_json::to_string(&settings).unwrap();
        assert_eq!(serde_json::from_str::<SamplerParams>(&saved).unwrap(), settings);
        for (attack_ms, decay_ms, sustain, expected) in [
            (f32::NAN, f32::INFINITY, f32::NEG_INFINITY, (0.0, 0.0, 1.0)),
            (-1.0, 20_000.0, 2.0, (0.0, 10_000.0, 1.0)),
            (20_000.0, -1.0, -1.0, (10_000.0, 0.0, 0.0)),
        ] {
            let mut doc = crate::Document::default();
            let id = ChannelId(
                doc.execute(crate::Command::AddChannel {
                    name: "Sampler".into(),
                    instrument: Instrument::Sampler(SamplerParams {
                        attack_ms,
                        decay_ms,
                        sustain,
                        ..Default::default()
                    }),
                })
                .unwrap()
                .0
                .unwrap(),
            );
            let mut project = doc.project().clone();
            project.sanitize();
            let Instrument::Sampler(p) = &project.channel(id).unwrap().instrument else {
                panic!("sampler expected");
            };
            assert_eq!((p.attack_ms, p.decay_ms, p.sustain), expected);
        }
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
            mute_locked: false,
            mixer_track: 999,
            root_key: 200,
            group: None,
            extra: Extra::new(),
        });
        p.patterns[0].notes.push(Note {
            id: NoteId(70),
            channel: ChannelId(404),
            key: 60,
            start: 0,
            length: 0,
            velocity: 1.0,
            muted: false,
            pan: 0.0,
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
