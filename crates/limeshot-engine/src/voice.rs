//! Voices: one sounding note of a sampler, synth, metronome click or browser preview.

use limeshot_dsp::filter::Svf;
use limeshot_dsp::osc::Shape;
use limeshot_dsp::{Adsr, Envelope, Oscillator, SampleData, key_to_hz};
use limeshot_sequencer::{InstrumentState, SynthMods};
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
        /// A loop inside `[lo, hi)`: playback wraps round it instead of running out.
        looping: Option<(f64, f64)>,
    },
    Synth {
        a: Oscillator,
        b: Option<Oscillator>,
        /// Filter and LFO; `None` when they do nothing.
        mods: Option<ModState>,
    },
    Click {
        osc: Oscillator,
    },
}

/// Modulation is recomputed every this many samples.
const MOD_INTERVAL: u32 = 16;

/// Per-voice LimeSynth filter, filter envelope and LFO.
#[derive(Debug, Clone, Copy)]
pub struct ModState {
    cfg: SynthMods,
    sample_rate: f32,
    /// Oscillator frequencies before vibrato.
    freq_a: f32,
    freq_b: f32,
    lfo_phase: f32,
    lfo_inc: f32,
    /// Filter envelope level, 1 at the note start, decaying towards 0.
    env: f32,
    env_coef: f32,
    svf: Svf,
    countdown: u32,
}

impl ModState {
    fn new(cfg: SynthMods, sample_rate: f32, freq_a: f32, freq_b: f32) -> Self {
        let decay_samples = (cfg.env_decay.max(0.001) * sample_rate).max(1.0);
        Self {
            cfg,
            sample_rate,
            freq_a,
            freq_b,
            lfo_phase: 0.0,
            lfo_inc: cfg.lfo_rate.max(0.0) / sample_rate.max(1.0),
            env: 1.0,
            // Down to 1/1000 (−60 dB) over the decay time.
            env_coef: 0.001f32.powf(1.0 / decay_samples),
            svf: Svf::default(),
            countdown: 0,
        }
    }

    #[inline]
    fn process(&mut self, x: f32, a: &mut Oscillator, b: Option<&mut Oscillator>) -> f32 {
        if self.countdown == 0 {
            self.countdown = MOD_INTERVAL;
            let lfo = (self.lfo_phase * std::f32::consts::TAU).sin();
            if self.cfg.vibrato_cents > 0.0 {
                let k = 2f32.powf(self.cfg.vibrato_cents * lfo / 1200.0);
                a.set_frequency(self.freq_a * k, self.sample_rate);
                if let Some(b) = b {
                    b.set_frequency(self.freq_b * k, self.sample_rate);
                }
            }
            if self.cfg.filter_on() {
                let oct = self.cfg.env_octaves * self.env + self.cfg.lfo_cutoff * lfo;
                self.svf
                    .set(self.cfg.cutoff * 2f32.powf(oct), self.cfg.resonance, self.sample_rate);
            }
        }
        self.countdown -= 1;
        self.lfo_phase = (self.lfo_phase + self.lfo_inc).fract();
        self.env *= self.env_coef;
        if self.cfg.filter_on() { self.svf.lowpass(x) } else { x }
    }
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
    /// The note's pan, added to the channel's.
    pub pan: f32,
    /// Started by the sequencer (as opposed to a preview).
    pub sequenced: bool,
    /// Absolute engine sample at which the note releases (`u64::MAX`: no scheduled release).
    pub release_at: u64,
    /// Pass-relative tick of the note end, to recompute `release_at` after a tempo change.
    pub end_tick: u64,
    /// Start order, for stealing the oldest voice.
    pub serial: u64,
    /// An audio clip's fades.
    pub fade: Option<ClipFade>,
    /// The sampler's low-pass filter, one per side.
    pub filter: Option<[Svf; 2]>,
}

/// An audio clip's fade-in and fade-out, counted in frames: `pos` from the clip's start, `left`
/// to its end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipFade {
    pub in_len: f32,
    pub pos: f32,
    pub out_len: f32,
    pub left: f32,
}

impl ClipFade {
    /// The gain for the next frame (linear ramps), advancing by one frame.
    fn next(&mut self) -> f32 {
        let g_in = if self.in_len > 0.0 {
            (self.pos / self.in_len).min(1.0)
        } else {
            1.0
        };
        let g_out = if self.out_len > 0.0 {
            (self.left / self.out_len).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.pos += 1.0;
        self.left -= 1.0;
        g_in * g_out
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            source: Source::Off,
            env: Envelope::start(Adsr::gate(0.0), 48_000.0),
            channel: NO_CHANNEL,
            key: 0,
            gain: 0.0,
            pan: 0.0,
            sequenced: false,
            release_at: u64::MAX,
            end_tick: u64::MAX,
            serial: 0,
            fade: None,
            filter: None,
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
                looping,
                filter,
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
                // The loop, inside the played window and at least a few frames long.
                let looping = looping.and_then(|(a, b)| {
                    let (a, b) = ((f64::from(a) * frames).max(lo), (f64::from(b) * frames).min(hi));
                    (b - a >= 4.0).then_some((a, b))
                });
                let filter = filter.map(|(cutoff, res)| {
                    let mut f = Svf::default();
                    f.set(cutoff, res, sample_rate);
                    [f, f]
                });
                Some(Self {
                    source: Source::Sampler {
                        sample: sample.clone(),
                        pos,
                        step,
                        lo,
                        hi,
                        reverse: *reverse,
                        looping,
                    },
                    filter,
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
                mods,
            } => {
                let k = f32::from(key) + 12.0 * f32::from(*octave);
                let f = key_to_hz(k);
                let fb = f * 2f32.powf(detune_cents / 1200.0);
                let a = Oscillator::new(*shape, f, sample_rate, 0.0);
                let b = (detune_cents.abs() > 0.01).then(|| Oscillator::new(*shape, fb, sample_rate, 0.25));
                let mods = mods.any().then(|| ModState::new(*mods, sample_rate, f, fb));
                // Square and saw are harmonically dense; trim them so all shapes feel equally loud.
                let shape_gain = match shape {
                    Shape::Sine | Shape::Triangle => 1.0,
                    Shape::Square => 0.5,
                    Shape::Saw => 0.6,
                };
                let osc_gain = if b.is_some() { 0.5 } else { 1.0 };
                Some(Self {
                    source: Source::Synth { a, b, mods },
                    env: Envelope::start(*adsr, sample_rate),
                    key,
                    gain: velocity * gain * shape_gain * osc_gain,
                    ..Self::default()
                })
            }
            // Plugin channels have no voices; the renderer sends them note events.
            InstrumentState::Plugin { .. } | InstrumentState::Silent => None,
        }
    }

    /// Apply instrument automation and pitch bend while preserving oscillator phase,
    /// sample position, and the current envelope stage. Call with the note's original velocity.
    pub fn update_instrument(
        &mut self,
        instrument: &InstrumentState,
        root_key: u8,
        velocity: f32,
        bend_semitones: f32,
        sample_rate: f32,
    ) {
        match (&mut self.source, instrument) {
            (
                Source::Synth { a, b, mods },
                InstrumentState::Synth {
                    shape,
                    adsr,
                    gain,
                    octave,
                    detune_cents,
                    mods: cfg,
                },
            ) => {
                let f = key_to_hz(f32::from(self.key) + 12.0 * f32::from(*octave) + bend_semitones);
                let fb = f * 2f32.powf(detune_cents / 1200.0);
                a.shape = *shape;
                a.set_frequency(f, sample_rate);
                if detune_cents.abs() > 0.01 {
                    let second = b.get_or_insert_with(|| Oscillator::new(*shape, fb, sample_rate, 0.25));
                    second.shape = *shape;
                    second.set_frequency(fb, sample_rate);
                } else {
                    *b = None;
                }
                if cfg.any() {
                    let m = mods.get_or_insert_with(|| ModState::new(*cfg, sample_rate, f, fb));
                    m.cfg = *cfg;
                    m.freq_a = f;
                    m.freq_b = fb;
                    m.lfo_inc = cfg.lfo_rate.max(0.0) / sample_rate.max(1.0);
                    m.env_coef = 0.001f32.powf(1.0 / (cfg.env_decay.max(0.001) * sample_rate).max(1.0));
                    m.countdown = 0;
                } else {
                    *mods = None;
                }
                let shape_gain = match shape {
                    Shape::Sine | Shape::Triangle => 1.0,
                    Shape::Square => 0.5,
                    Shape::Saw => 0.6,
                };
                self.gain = velocity * gain * shape_gain * if b.is_some() { 0.5 } else { 1.0 };
                self.env.set_adsr(*adsr);
            }
            (
                Source::Sampler {
                    sample,
                    step,
                    lo,
                    hi,
                    reverse,
                    ..
                },
                InstrumentState::Sampler {
                    gain,
                    pitch,
                    start,
                    end,
                    reverse: rev,
                    release,
                    ..
                },
            ) => {
                let semis = f64::from(self.key) - f64::from(root_key) + f64::from(*pitch + bend_semitones);
                *step = f64::from(sample.sample_rate) / f64::from(sample_rate) * 2f64.powf(semis / 12.0);
                *lo = f64::from(start.min(*end)) * sample.frames() as f64;
                *hi = f64::from(start.max(*end)) * sample.frames() as f64;
                *reverse = *rev;
                self.gain = velocity * gain;
                self.env.set_adsr(Adsr::gate(*release));
            }
            _ => {}
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
                looping: None,
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
                looping,
            } => {
                if let Some((a, b)) = *looping {
                    // Wrap round the loop (both ways, for reversed samples).
                    if !*reverse && *pos >= b {
                        *pos = a + (*pos - b) % (b - a);
                    } else if *reverse && *pos < a {
                        *pos = b - (a - *pos) % (b - a);
                    }
                }
                if *pos < *lo || *pos >= *hi {
                    self.source = Source::Off;
                    return (0.0, 0.0);
                }
                let mut f = sample.frame_at(*pos);
                if *reverse {
                    *pos -= *step;
                } else {
                    *pos += *step;
                }
                if let Some([l, r]) = &mut self.filter {
                    f = (l.lowpass(f.0), r.lowpass(f.1));
                }
                f
            }
            Source::Synth { a, b, mods } => {
                let mut s = a.next() + b.as_mut().map_or(0.0, |o| o.next());
                if let Some(m) = mods {
                    s = m.process(s, a, b.as_mut());
                }
                (s, s)
            }
            Source::Click { osc } => {
                let s = osc.next();
                (s, s)
            }
        };
        let mut e = self.env.next() * self.gain;
        if let Some(f) = &mut self.fade {
            e *= f.next();
        }
        if self.env.is_done() {
            self.source = Source::Off;
        }
        (l * e, r * e)
    }
}
