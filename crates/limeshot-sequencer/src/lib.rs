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

use limeshot_core::{Instrument, PatternId, Project, Tempo, Tick, TimeSignature, Waveform};
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
    },
    Silent,
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

#[derive(Debug, Clone, Copy)]
pub struct MixerTrackState {
    pub gain: f32,
    pub pan: f32,
    pub audible: bool,
}

/// Everything the engine needs to play, immutable once built.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub channels: Vec<ChannelState>,
    /// Index 0 is the master.
    pub mixer: Vec<MixerTrackState>,
    pub pattern: Program,
    pub song: Program,
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
            }],
            pattern: Program::default(),
            song: Program::default(),
        }
    }
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
                },
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
            gain: t.volume,
            pan: t.pan,
            audible: !t.muted && (i == 0 || !any_insert_solo || t.solo),
        })
        .collect::<Vec<_>>();

    let channel_index = |id| project.channel_index(id).and_then(|i| u16::try_from(i).ok());

    let pattern = project.pattern(current_pattern).or_else(|| project.patterns.first());
    let pattern_program = match pattern {
        Some(p) => {
            let length = p.length(sig);
            let mut events: Vec<NoteEvent> = p
                .notes
                .iter()
                .filter(|n| n.start < length)
                .filter_map(|n| {
                    Some(NoteEvent {
                        tick: n.start,
                        length: n.length.min(length - n.start),
                        channel: channel_index(n.channel)?,
                        key: n.key,
                        velocity: n.velocity,
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
        let Some(p) = project.pattern(clip.pattern()) else {
            continue;
        };
        let window_end = clip.offset.saturating_add(clip.length);
        for n in &p.notes {
            if n.start < clip.offset || n.start >= window_end {
                continue;
            }
            let Some(channel) = channel_index(n.channel) else {
                continue;
            };
            let tick = clip.start + (n.start - clip.offset);
            song_events.push(NoteEvent {
                tick,
                length: n.length.min(clip.end() - tick),
                channel,
                key: n.key,
                velocity: n.velocity,
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
        pattern: pattern_program,
        song,
    }
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
