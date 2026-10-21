//! Turns the editable [`Project`] into an immutable [`Snapshot`] the audio thread can play.
//!
//! The snapshot is built on the control (UI) thread whenever the project changes and handed to the
//! engine whole. It contains two [`Program`]s, one for Pattern mode (the current pattern) and one
//! for Song mode (the playlist flattened), each a tick-sorted list of note events with a loop
//! length. Channel and mixer settings are resolved too: mute and solo become a single `audible`
//! flag, dB become linear gains, and sample references become decoded buffers.
//!
//! The Piano Roll, Channel Rack and Playlist all edit the same `Project`; this module is the only
//! translation from that model to sound, so there is no second note representation to drift.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use limeshot_core::automation::{curve_value, shape as curve_shape};
use limeshot_core::{
    AutomationTarget, ClipSource, Instrument, Mapping, PPQ, PatternId, Project, STEP_TICKS, Tempo, Tick, TimeSignature,
    Waveform,
};
use limeshot_dsp::osc::Shape;
use limeshot_dsp::{Adsr, SampleData, db_to_gain};
use std::sync::Arc;

/// A note to start at `tick`, lasting `length` ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteEvent {
    pub tick: Tick,
    pub length: Tick,
    /// Index into [`Snapshot::channels`].
    pub channel: u16,
    pub key: u8,
    pub velocity: f32,
    /// The note's own pan, added to the channel's (-1 to 1).
    pub pan: f32,
    /// Seconds into the channel's sample at which playback starts (audio clips trimmed at the
    /// start); 0 for notes.
    pub offset_secs: f64,
    /// An audio clip: plays for exactly `length` even on a one-shot sampler, and starts mid-way
    /// when playback begins inside it.
    pub clip: bool,
}

impl NoteEvent {
    pub fn note(tick: Tick, length: Tick, channel: u16, key: u8, velocity: f32) -> Self {
        Self {
            tick,
            length,
            channel,
            key,
            velocity,
            pan: 0.0,
            offset_secs: 0.0,
            clip: false,
        }
    }

    pub fn end(&self) -> Tick {
        self.tick.saturating_add(self.length)
    }
}

/// A loopable, tick-sorted list of note events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Program {
    pub events: Vec<NoteEvent>,
    /// Loop/end point in ticks. 0 means there is nothing to play.
    pub length: Tick,
}

impl Program {
    /// Events with `lo <= tick < hi`.
    pub fn range(&self, lo: Tick, hi: Tick) -> &[NoteEvent] {
        let a = self.events.partition_point(|e| e.tick < lo);
        let b = self.events.partition_point(|e| e.tick < hi);
        self.events.get(a..b.max(a)).unwrap_or(&[])
    }
}

/// A channel's sound source, resolved for playback.
#[derive(Debug, Clone)]
pub enum InstrumentState {
    Sampler {
        sample: Option<Arc<SampleData>>,
        gain: f32,
        /// Semitones.
        pitch: f32,
        start: f32,
        end: f32,
        reverse: bool,
        release: f32,
        one_shot: bool,
    },
    Synth {
        shape: Shape,
        adsr: Adsr,
        gain: f32,
        octave: i8,
        detune_cents: f32,
        mods: SynthMods,
    },
    /// A hosted plugin; the engine plays the instrument adopted for this channel id.
    Plugin {
        channel: u32,
    },
    Silent,
}

/// LimeSynth's filter and LFO settings, resolved for playback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynthMods {
    /// Hz.
    pub cutoff: f32,
    pub resonance: f32,
    /// Filter envelope depth in octaves.
    pub env_octaves: f32,
    /// Seconds for the filter envelope to decay.
    pub env_decay: f32,
    /// Hz.
    pub lfo_rate: f32,
    pub vibrato_cents: f32,
    /// LFO to cutoff in octaves.
    pub lfo_cutoff: f32,
}

impl SynthMods {
    /// Whether the filter does anything (an open filter without modulation is bypassed, so
    /// default settings sound exactly as before the filter existed).
    pub fn filter_on(&self) -> bool {
        self.cutoff < 19_999.0 || self.env_octaves > 0.0 || self.lfo_cutoff > 0.0 || self.resonance > 0.0
    }

    pub fn any(&self) -> bool {
        self.filter_on() || self.vibrato_cents > 0.0
    }
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub instrument: InstrumentState,
    pub volume: f32,
    pub pan: f32,
    /// Not muted, and soloed if any channel is.
    pub audible: bool,
    pub mixer_track: usize,
    pub root_key: u8,
}

#[derive(Debug, Clone)]
pub struct MixerTrackState {
    pub gain: f32,
    pub pan: f32,
    pub audible: bool,
    /// Insert chain in processing order. The engine keys processors by `id`.
    pub effects: Vec<EffectState>,
    /// Post-fader outputs: (destination track, linear level). Empty for the master.
    pub routes: Vec<(usize, f32)>,
    /// The built-in EQ's bands as (frequency, gain dB, width).
    pub eq: [(f32, f32, f32); 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectState {
    pub id: u32,
    pub kind: limeshot_core::EffectKind,
    pub enabled: bool,
    pub params: Vec<f32>,
}

/// The parameter an automation lane drives, resolved to snapshot indices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LaneTarget {
    ChannelVolume(usize),
    ChannelPan(usize),
    MixerVolume(usize),
    MixerPan(usize),
    EffectParam { track: usize, effect: u32, index: usize },
}

/// A breakpoint in song time. `hold` keeps the value until the next point (gaps between clips).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LanePoint {
    pub tick: Tick,
    /// Normalised value, 0–1.
    pub value: f32,
    pub tension: f32,
    pub hold: bool,
}

/// One automation flattened over the arrangement (Song mode only, as in FL Studio).
#[derive(Debug, Clone, PartialEq)]
pub struct AutomationLane {
    pub target: LaneTarget,
    pub mapping: Mapping,
    /// Sorted by tick.
    pub points: Vec<LanePoint>,
}

impl AutomationLane {
    /// The parameter value at song tick `tick`, in the parameter's own units. Before the first
    /// clip the lane takes the first clip's starting value; after the last it holds the last.
    /// Allocation-free.
    pub fn value_at(&self, tick: Tick) -> Option<f32> {
        let first = self.points.first()?;
        let norm = if tick <= first.tick {
            first.value
        } else {
            let i = self.points.partition_point(|p| p.tick <= tick);
            match (self.points.get(i.wrapping_sub(1)), self.points.get(i)) {
                (Some(a), Some(b)) if !a.hold => {
                    let x = (tick - a.tick) as f32 / (b.tick - a.tick).max(1) as f32;
                    a.value + (b.value - a.value) * curve_shape(x, a.tension)
                }
                (Some(a), _) => a.value,
                _ => first.value,
            }
        };
        Some(self.mapping.to_value(norm))
    }
}

/// Everything the engine needs to play, immutable once built.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub channels: Vec<ChannelState>,
    /// Index 0 is the master.
    pub mixer: Vec<MixerTrackState>,
    /// Insert tracks in processing order (sources before the tracks they route to); the master
    /// is processed after all of them.
    pub mix_order: Vec<usize>,
    pub pattern: Program,
    pub song: Program,
    /// Automation applied while in Song mode.
    pub automation: Vec<AutomationLane>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            tempo: Tempo::default(),
            time_signature: TimeSignature::default(),
            channels: Vec::new(),
            mixer: vec![MixerTrackState {
                gain: 1.0,
                pan: 0.0,
                audible: true,
                effects: Vec::new(),
                routes: Vec::new(),
                eq: flat_eq(),
            }],
            mix_order: Vec::new(),
            pattern: Program::default(),
            song: Program::default(),
            automation: Vec::new(),
        }
    }
}

fn flat_eq() -> [(f32, f32, f32); 3] {
    limeshot_core::EqBand::DEFAULTS.map(|b| (b.freq, b.gain, b.width))
}

/// Supplies decoded samples for stored sample paths. The UI's sample cache implements it; tests
/// use closures or maps.
pub trait SampleLookup {
    fn sample(&self, stored_path: &str) -> Option<Arc<SampleData>>;
}

impl<F: Fn(&str) -> Option<Arc<SampleData>>> SampleLookup for F {
    fn sample(&self, stored_path: &str) -> Option<Arc<SampleData>> {
        self(stored_path)
    }
}

/// No samples available (synth-only tests, or before loading finishes).
pub struct NoSamples;

impl SampleLookup for NoSamples {
    fn sample(&self, _: &str) -> Option<Arc<SampleData>> {
        None
    }
}

fn shape(w: Waveform) -> Shape {
    match w {
        Waveform::Sine => Shape::Sine,
        Waveform::Square => Shape::Square,
        Waveform::Saw => Shape::Saw,
        Waveform::Triangle => Shape::Triangle,
    }
}

/// How far swing delays a note starting at pattern tick `start`: only notes exactly on the
/// second 16th of each pair move, by up to half a step.
pub fn swing_offset(start: Tick, swing: f32) -> Tick {
    if swing <= 0.0 || !swing.is_finite() || start % (2 * STEP_TICKS) != STEP_TICKS {
        return 0;
    }
    (f64::from(swing.min(1.0)) * (STEP_TICKS / 2) as f64).round() as Tick
}

/// Build the playback snapshot. `current_pattern` is what Pattern mode loops; an unknown id falls
/// back to the first pattern.
pub fn compile(project: &Project, current_pattern: PatternId, samples: &dyn SampleLookup) -> Snapshot {
    let sig = project.time_signature;
    let any_solo = project.channels.iter().any(|c| c.solo);
    let tracks = project.mixer.tracks.len().max(1);
    let channels = project
        .channels
        .iter()
        .map(|c| ChannelState {
            instrument: match &c.instrument {
                Instrument::Sampler(p) => InstrumentState::Sampler {
                    sample: p.sample.as_ref().and_then(|r| samples.sample(&r.path)),
                    gain: db_to_gain(p.gain_db),
                    pitch: p.pitch,
                    start: p.start.clamp(0.0, 1.0),
                    end: p.end.clamp(0.0, 1.0),
                    reverse: p.reverse,
                    release: p.release_ms.max(0.0) / 1000.0,
                    one_shot: p.one_shot,
                },
                Instrument::Synth(p) => InstrumentState::Synth {
                    shape: shape(p.waveform),
                    adsr: Adsr {
                        attack: p.attack_ms / 1000.0,
                        decay: p.decay_ms / 1000.0,
                        sustain: p.sustain,
                        release: p.release_ms / 1000.0,
                    },
                    gain: db_to_gain(p.gain_db),
                    octave: p.octave,
                    detune_cents: p.detune_cents,
                    mods: SynthMods {
                        cutoff: p.cutoff_hz,
                        resonance: p.resonance,
                        env_octaves: p.filter_env,
                        env_decay: p.filter_decay_ms / 1000.0,
                        lfo_rate: p.lfo_rate_hz,
                        vibrato_cents: p.vibrato_cents,
                        lfo_cutoff: p.lfo_cutoff,
                    },
                },
                Instrument::Plugin(_) => InstrumentState::Plugin { channel: c.id.0 },
                Instrument::Unsupported(_) => InstrumentState::Silent,
            },
            volume: c.volume,
            pan: c.pan,
            audible: !c.muted && (!any_solo || c.solo),
            mixer_track: usize::from(c.mixer_track).min(tracks - 1),
            root_key: c.root_key,
        })
        .collect();

    let any_insert_solo = project.mixer.tracks.iter().skip(1).any(|t| t.solo);
    let mixer = project
        .mixer
        .tracks
        .iter()
        .enumerate()
        .map(|(i, t)| MixerTrackState {
            eq: t.eq.map(|b| (b.freq, b.gain, b.width)),
            gain: t.volume,
            pan: t.pan,
            audible: !t.muted && (i == 0 || !any_insert_solo || t.solo),
            effects: t
                .effects
                .iter()
                .map(|e| EffectState {
                    id: e.id,
                    kind: e.kind,
                    enabled: e.enabled,
                    params: e.params.clone(),
                })
                .collect(),
            routes: if i == 0 {
                Vec::new()
            } else {
                t.routes.iter().map(|r| (usize::from(r.to), r.level)).collect()
            },
        })
        .collect::<Vec<_>>();

    let channel_index = |id| project.channel_index(id).and_then(|i| u16::try_from(i).ok());
    let swing = |start: Tick| swing_offset(start, project.swing);

    let pattern = project.pattern(current_pattern).or_else(|| project.patterns.first());
    let pattern_program = match pattern {
        Some(p) => {
            let length = p.length(sig);
            let mut events: Vec<NoteEvent> = p
                .notes
                .iter()
                .filter(|n| n.start < length && !n.muted)
                .filter_map(|n| {
                    let tick = n.start + swing(n.start);
                    Some(NoteEvent {
                        pan: n.pan,
                        ..NoteEvent::note(
                            tick,
                            n.length.min(length.saturating_sub(tick)).max(1),
                            channel_index(n.channel)?,
                            n.key,
                            n.velocity,
                        )
                    })
                })
                .collect();
            events.sort_by_key(|e| (e.tick, e.channel, e.key));
            Program { events, length }
        }
        None => Program::default(),
    };

    let mut song_events = Vec::new();
    for clip in &project.playlist.clips {
        let track_muted = project.playlist.tracks.get(clip.track as usize).is_none_or(|t| t.muted);
        if clip.muted || track_muted {
            continue;
        }
        let p = match clip.source {
            ClipSource::Pattern(id) => match project.pattern(id) {
                Some(p) => p,
                None => continue,
            },
            ClipSource::Audio(ch) => {
                if let (Some(channel), Some(c)) = (channel_index(ch), project.channel(ch)) {
                    // Tempo is constant within a project, so a tick offset is a fixed time.
                    let secs = project.tempo.bpm().recip() * 60.0 * clip.offset as f64 / PPQ as f64;
                    song_events.push(NoteEvent {
                        offset_secs: secs,
                        clip: true,
                        ..NoteEvent::note(clip.start, clip.length, channel, c.root_key, 1.0)
                    });
                }
                continue;
            }
            ClipSource::Automation(_) => continue,
        };
        let window_end = clip.offset.saturating_add(clip.length);
        for n in &p.notes {
            if n.muted || n.start < clip.offset || n.start >= window_end {
                continue;
            }
            let Some(channel) = channel_index(n.channel) else {
                continue;
            };
            let tick = clip.start + (n.start - clip.offset) + swing(n.start);
            if tick >= clip.end() {
                continue;
            }
            song_events.push(NoteEvent {
                pan: n.pan,
                ..NoteEvent::note(tick, n.length.min(clip.end() - tick), channel, n.key, n.velocity)
            });
        }
    }
    song_events.sort_by_key(|e| (e.tick, e.channel, e.key));
    let song = Program {
        events: song_events,
        length: project.playlist.song_length(sig),
    };

    Snapshot {
        tempo: project.tempo,
        time_signature: sig,
        channels,
        mixer: if mixer.is_empty() {
            Snapshot::default().mixer
        } else {
            mixer
        },
        mix_order: project.mixer.processing_order(),
        pattern: pattern_program,
        song,
        automation: automation_lanes(project),
    }
}

/// Flatten every automation's clips into a lane in song time.
fn automation_lanes(project: &Project) -> Vec<AutomationLane> {
    let channel = |c| project.channel_index(c);
    let track = |t: u16| (usize::from(t) < project.mixer.tracks.len()).then_some(usize::from(t));
    let mut lanes = Vec::new();
    for a in &project.automations {
        let Some(mapping) = a.target.mapping(project) else {
            continue;
        };
        let target = match a.target {
            AutomationTarget::ChannelVolume(c) => channel(c).map(LaneTarget::ChannelVolume),
            AutomationTarget::ChannelPan(c) => channel(c).map(LaneTarget::ChannelPan),
            AutomationTarget::MixerVolume(t) => track(t).map(LaneTarget::MixerVolume),
            AutomationTarget::MixerPan(t) => track(t).map(LaneTarget::MixerPan),
            AutomationTarget::EffectParam {
                track: t,
                effect,
                index,
            } => track(t).map(|track| LaneTarget::EffectParam { track, effect, index }),
        };
        let Some(target) = target else { continue };
        let mut clips: Vec<_> = project
            .playlist
            .clips
            .iter()
            .filter(|c| c.automation() == Some(a.id) && !c.muted)
            .filter(|c| project.playlist.tracks.get(c.track as usize).is_some_and(|t| !t.muted))
            .collect();
        if clips.is_empty() || a.points.is_empty() {
            continue;
        }
        clips.sort_by_key(|c| c.start);
        let tension_at = |t: Tick| {
            let i = a.points.partition_point(|p| p.tick <= t);
            a.points.get(i.wrapping_sub(1)).map_or(0.0, |p| p.tension)
        };
        let value = |t| curve_value(&a.points, t).unwrap_or(0.0);
        let mut points = Vec::new();
        for c in clips {
            let (o, len) = (c.offset, c.length);
            points.push(LanePoint {
                tick: c.start,
                value: value(o),
                tension: tension_at(o),
                hold: false,
            });
            for p in a.points.iter().filter(|p| p.tick > o && p.tick < o + len) {
                points.push(LanePoint {
                    tick: c.start + (p.tick - o),
                    value: p.value,
                    tension: p.tension,
                    hold: false,
                });
            }
            points.push(LanePoint {
                tick: c.end(),
                value: value(o + len),
                tension: 0.0,
                hold: true,
            });
        }
        // Stable: at a shared tick the next clip's start comes after the previous clip's end.
        points.sort_by_key(|p| p.tick);
        lanes.push(AutomationLane {
            target,
            mapping,
            points,
        });
    }
    lanes
}

#[cfg(test)]
mod tests {
    use super::*;
    use limeshot_core::*;

    fn setup() -> (Document, ChannelId, ChannelId, PatternId) {
        let mut d = Document::default();
        let pat = d.project().patterns[0].id;
        let a = ChannelId(
            d.execute(Command::AddChannel {
                name: "A".into(),
                instrument: Instrument::Sampler(SamplerParams::default()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let b = ChannelId(
            d.execute(Command::AddChannel {
                name: "B".into(),
                instrument: Instrument::Synth(SynthParams::default()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        (d, a, b, pat)
    }

    #[test]
    fn steps_become_events_on_the_grid() {
        let (mut d, a, _, pat) = setup();
        for s in [0, 4, 8, 12] {
            d.execute(Command::SetStep {
                pattern: pat,
                channel: a,
                step: s,
                on: true,
            })
            .unwrap();
        }
        let snap = compile(d.project(), pat, &NoSamples);
        let ticks: Vec<Tick> = snap.pattern.events.iter().map(|e| e.tick).collect();
        assert_eq!(ticks, vec![0, 960, 1920, 2880]);
        assert_eq!(snap.pattern.length, 3840);
        assert_eq!(snap.pattern.range(900, 1921).len(), 2);
    }

    #[test]
    fn swing_delays_every_second_sixteenth() {
        let (mut d, a, _, pat) = setup();
        for s in [0, 1, 2, 3] {
            d.execute(Command::SetStep {
                pattern: pat,
                channel: a,
                step: s,
                on: true,
            })
            .unwrap();
        }
        d.execute(Command::SetSwing(0.5)).unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 3840,
            length: None,
        })
        .unwrap();
        let snap = compile(d.project(), pat, &NoSamples);
        let ticks: Vec<Tick> = snap.pattern.events.iter().map(|e| e.tick).collect();
        assert_eq!(ticks, vec![0, 300, 480, 780], "odd steps move by a quarter step at 50%");
        let song: Vec<Tick> = snap.song.events.iter().map(|e| e.tick).collect();
        assert_eq!(song, vec![3840, 4140, 4320, 4620]);
        assert_eq!(swing_offset(250, 1.0), 0, "off-grid notes are left alone");
    }

    #[test]
    fn playlist_flattens_linked_clips() {
        let (mut d, a, _, pat) = setup();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: a,
            step: 0,
            on: true,
        })
        .unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 0,
            length: None,
        })
        .unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 1,
            start: 7680,
            length: None,
        })
        .unwrap();
        let snap = compile(d.project(), pat, &NoSamples);
        let ticks: Vec<Tick> = snap.song.events.iter().map(|e| e.tick).collect();
        assert_eq!(ticks, vec![0, 7680]);
        assert_eq!(snap.song.length, 3 * 3840);
    }

    #[test]
    fn muted_notes_do_not_play() {
        let (mut d, _, b, pat) = setup();
        let id = d
            .execute(Command::AddNote {
                pattern: pat,
                note: NewNote {
                    channel: b,
                    key: 60,
                    start: 0,
                    length: 480,
                    velocity: 1.0,
                },
            })
            .unwrap()
            .0
            .unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 0,
            length: None,
        })
        .unwrap();
        let count = |d: &Document| {
            let snap = compile(d.project(), pat, &NoSamples);
            (snap.pattern.events.len(), snap.song.events.len())
        };
        let before = count(&d);
        d.execute(Command::SetNotesMuted {
            pattern: pat,
            notes: vec![NoteId(id)],
            muted: true,
        })
        .unwrap();
        let after = count(&d);
        assert_eq!((after.0 + 1, after.1 + 1), before, "silent in Pattern and Song mode");
    }

    #[test]
    fn clip_truncates_notes() {
        let (mut d, _, b, pat) = setup();
        d.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: b,
                key: 60,
                start: 0,
                length: 3000,
                velocity: 1.0,
            },
        })
        .unwrap();
        d.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: b,
                key: 62,
                start: 2000,
                length: 100,
                velocity: 1.0,
            },
        })
        .unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 100,
            length: Some(1000),
        })
        .unwrap();
        let snap = compile(d.project(), pat, &NoSamples);
        assert_eq!(snap.song.events.len(), 1);
        assert_eq!((snap.song.events[0].tick, snap.song.events[0].length), (100, 1000));
    }

    #[test]
    fn muted_tracks_and_clips_are_skipped_and_solo_resolves() {
        let (mut d, a, b, pat) = setup();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: a,
            step: 0,
            on: true,
        })
        .unwrap();
        let c = ClipId(
            d.execute(Command::AddClip {
                pattern: pat,
                track: 2,
                start: 0,
                length: None,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        d.execute(Command::SetPlaylistTrackMute { track: 2, muted: true })
            .unwrap();
        assert!(compile(d.project(), pat, &NoSamples).song.events.is_empty());
        d.execute(Command::SetPlaylistTrackMute { track: 2, muted: false })
            .unwrap();
        d.execute(Command::SetClipMuted { clip: c, muted: true }).unwrap();
        assert!(compile(d.project(), pat, &NoSamples).song.events.is_empty());

        d.execute(Command::SetChannelSolo { channel: b, solo: true }).unwrap();
        d.execute(Command::SetMixerSolo { track: 2, solo: true }).unwrap();
        let snap = compile(d.project(), pat, &NoSamples);
        assert!(!snap.channels[0].audible);
        assert!(snap.channels[1].audible);
        assert!(snap.mixer[0].audible, "master is never solo-muted");
        assert!(!snap.mixer[1].audible);
        assert!(snap.mixer[2].audible);
    }

    #[test]
    fn samples_are_resolved_through_the_lookup() {
        let mut d = Document::default();
        let pat = d.project().patterns[0].id;
        d.execute(Command::AddChannel {
            name: "Kick".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: "kick.wav".into(),
                }),
                ..Default::default()
            }),
        })
        .unwrap();
        let data = Arc::new(SampleData::from_stereo(48_000, vec![0.5; 20]));
        let lookup = |p: &str| (p == "kick.wav").then(|| data.clone());
        let snap = compile(d.project(), pat, &lookup);
        match &snap.channels[0].instrument {
            InstrumentState::Sampler { sample: Some(s), .. } => assert_eq!(s.frames(), 10),
            other => panic!("{other:?}"),
        }
    }
}
