//! Voices: one sounding note of a sampler, synth, metronome click or browser preview.

use limeshot_dsp::osc::Shape;
use limeshot_dsp::{Adsr, Envelope, Oscillator, SampleData, key_to_hz};
use limeshot_sequencer::InstrumentState;
use std::sync::Arc;

/// Voices that are not tied to a channel play straight into the master.
pub const NO_CHANNEL: u16 = u16::MAX;

#[derive(Debug, Clone)]
pub enum Source {
    Off,
    Sampler {
        sample: Arc<SampleData>,
        pos: f64,
        step: f64,
        /// Playback stops when `pos` leaves `[lo, hi)`.
        lo: f64,
        hi: f64,
        reverse: bool,
    },
    Synth {
        a: Oscillator,
        b: Option<Oscillator>,
    },
    Click {
        osc: Oscillator,
    },
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub source: Source,
    pub env: Envelope,
    /// Index into the snapshot's channels, or [`NO_CHANNEL`].
    pub channel: u16,
    pub key: u8,
    /// Velocity × instrument gain.
    pub gain: f32,
    /// Started by the sequencer (as opposed to a preview).
    pub sequenced: bool,
    /// Absolute engine sample at which the note releases (`u64::MAX`: no scheduled release).
    pub release_at: u64,
    /// Pass-relative tick of the note end, to recompute `release_at` after a tempo change.
    pub end_tick: u64,
    /// Start order, for stealing the oldest voice.
    pub serial: u64,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            source: Source::Off,
            env: Envelope::start(Adsr::gate(0.0), 48_000.0),
            channel: NO_CHANNEL,
            key: 0,
            gain: 0.0,
            sequenced: false,
            release_at: u64::MAX,
            end_tick: u64::MAX,
            serial: 0,
        }
    }
}

impl Voice {
    pub fn is_active(&self) -> bool {
        !matches!(self.source, Source::Off)
    }

    /// Set up a voice for `instrument`. Returns `None` when the instrument makes no sound (no
    /// sample loaded, unsupported instrument).
    pub fn for_instrument(
        instrument: &InstrumentState,
        root_key: u8,
        key: u8,
        velocity: f32,
        sample_rate: f32,
    ) -> Option<Self> {
        let velocity = velocity.clamp(0.0, 1.0);
        match instrument {
            InstrumentState::Sampler {
                sample,
                gain,
                pitch,
                start,
                end,
                reverse,
                release,
                one_shot: _,
            } => {
                let sample = sample.as_ref()?;
                let frames = sample.frames() as f64;
                if frames < 2.0 {
                    return None;
                }
                let semis = f64::from(key) - f64::from(root_key) + f64::from(*pitch);
                let step = f64::from(sample.sample_rate) / f64::from(sample_rate) * 2f64.powf(semis / 12.0);
                let lo = f64::from(start.min(*end)) * frames;
                let hi = f64::from(start.max(*end)) * frames;
                if hi - lo < 1.0 {
                    return None;
                }
                let pos = if *reverse { hi - 1.0 } else { lo };
                Some(Self {
                    source: Source::Sampler {
                        sample: sample.clone(),
                        pos,
                        step,
                        lo,
                        hi,
                        reverse: *reverse,
                    },
                    // Declick on the way out only (samples are expected to start cleanly), so the
                    // first sample frame lands exactly on the note's onset.
                    env: Envelope::start(
                        Adsr {
                            attack: 0.0,
                            decay: 0.0,
                            sustain: 1.0,
                            release: *release,
                        },
                        sample_rate,
                    ),
                    key,
                    gain: velocity * gain,
                    ..Self::default()
                })
            }
            InstrumentState::Synth {
                shape,
                adsr,
                gain,
                octave,
                detune_cents,
            } => {
                let k = f32::from(key) + 12.0 * f32::from(*octave);
                let f = key_to_hz(k);
                let a = Oscillator::new(*shape, f, sample_rate, 0.0);
                let b = (detune_cents.abs() > 0.01)
                    .then(|| Oscillator::new(*shape, f * 2f32.powf(detune_cents / 1200.0), sample_rate, 0.25));
                // Square and saw are harmonically dense; trim them so all shapes feel equally loud.
                let shape_gain = match shape {
                    Shape::Sine | Shape::Triangle => 1.0,
                    Shape::Square => 0.5,
                    Shape::Saw => 0.6,
                };
                let osc_gain = if b.is_some() { 0.5 } else { 1.0 };
                Some(Self {
                    source: Source::Synth { a, b },
                    env: Envelope::start(*adsr, sample_rate),
                    key,
                    gain: velocity * gain * shape_gain * osc_gain,
                    ..Self::default()
                })
            }
            InstrumentState::Silent => None,
        }
    }

    /// A metronome click: a short pitched blip, higher on the downbeat.
    pub fn click(downbeat: bool, sample_rate: f32) -> Self {
        let f = if downbeat { 1760.0 } else { 1320.0 };
        Self {
            source: Source::Click {
                osc: Oscillator::new(Shape::Sine, f, sample_rate, 0.0),
            },
            env: Envelope::start(
                Adsr {
                    attack: 0.0005,
                    decay: 0.03,
                    sustain: 0.0,
                    release: 0.005,
                },
                sample_rate,
            ),
            gain: if downbeat { 0.5 } else { 0.35 },
            ..Self::default()
        }
    }

    /// Plays a whole sample once at its own pitch (browser preview).
    pub fn preview(sample: Arc<SampleData>, sample_rate: f32) -> Option<Self> {
        let frames = sample.frames() as f64;
        if frames < 2.0 {
            return None;
        }
        let step = f64::from(sample.sample_rate) / f64::from(sample_rate);
        Some(Self {
            source: Source::Sampler {
                sample,
                pos: 0.0,
                step,
                lo: 0.0,
                hi: frames,
                reverse: false,
            },
            env: Envelope::start(
                Adsr {
                    attack: 0.001,
                    decay: 0.0,
                    sustain: 1.0,
                    release: 0.01,
                },
                sample_rate,
            ),
            gain: 0.8,
            ..Self::default()
        })
    }

    pub fn release(&mut self) {
        self.env.release();
    }

    /// Start `secs` (of output time) into the source: audio clips trimmed at the start, or
    /// playback starting inside a clip. Only samplers move; a voice skipped past its end stops.
    pub fn skip(&mut self, secs: f64, sample_rate: f32) {
        if let Source::Sampler {
            pos,
            step,
            lo,
            hi,
            reverse,
            ..
        } = &mut self.source
        {
            let frames = secs.max(0.0) * f64::from(sample_rate) * *step;
            if *reverse {
                *pos -= frames;
            } else {
                *pos += frames;
            }
            if *pos < *lo || *pos >= *hi {
                self.source = Source::Off;
            }
        }
    }

    /// Render one stereo frame (before channel gain and pan). Turns itself off when finished.
    #[inline]
    pub fn next_frame(&mut self) -> (f32, f32) {
        let (l, r) = match &mut self.source {
            Source::Off => return (0.0, 0.0),
            Source::Sampler {
                sample,
                pos,
                step,
                lo,
                hi,
                reverse,
            } => {
                if *pos < *lo || *pos >= *hi {
                    self.source = Source::Off;
                    return (0.0, 0.0);
                }
                let f = sample.frame_at(*pos);
                if *reverse {
                    *pos -= *step;
                } else {
                    *pos += *step;
                }
                f
            }
            Source::Synth { a, b } => {
                let s = a.next() + b.as_mut().map_or(0.0, |o| o.next());
                (s, s)
            }
            Source::Click { osc } => {
                let s = osc.next();
                (s, s)
            }
        };
        let e = self.env.next() * self.gain;
        if self.env.is_done() {
            self.source = Source::Off;
        }
        (l * e, r * e)
    }
}
