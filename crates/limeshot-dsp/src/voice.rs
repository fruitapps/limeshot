//! Voices: one sounding note of a sampler, synth, metronome click or browser preview.

use crate::filter::Svf;
use crate::osc::Shape;
use crate::{Adsr, Envelope, Oscillator, SampleData, key_to_hz};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum VoiceInstrument {
    Sampler {
        sample: Option<Arc<SampleData>>,
        gain: f32,
        /// Semitones.
        pitch: f32,
        start: f32,
        end: f32,
        reverse: bool,
        /// Attack, decay (seconds) and sustain level of the volume envelope.
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        one_shot: bool,
        /// Loop range as fractions of the sample, when looping.
        looping: Option<(f32, f32)>,
        /// Low-pass cutoff (Hz) and resonance, when the filter is not fully open.
        filter: Option<(f32, f32)>,
    },
    Synth {
        shape: Shape,
        adsr: Adsr,
        gain: f32,
        octave: i8,
        detune_cents: f32,
        mods: SynthMods,
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
    pub mode: crate::filter::Mode,
    /// Cutoff key tracking, 0–1 (octaves of cutoff per octave of pitch, around C5).
    pub key_track: f32,
}

impl SynthMods {
    /// Whether the filter does anything (an open filter without modulation is bypassed, so
    /// default settings sound exactly as before the filter existed).
    pub fn filter_on(&self) -> bool {
        self.cutoff < 19_999.0
            || self.env_octaves > 0.0
            || self.lfo_cutoff > 0.0
            || self.resonance > 0.0
            || self.key_track != 0.0
            || self.mode != crate::filter::Mode::LowPass
    }

    /// Whether a synth voice needs inline modulation, including key tracking alone.
    pub fn any(&self) -> bool {
        self.filter_on() || self.vibrato_cents > 0.0
    }
}

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
    /// Start per-note modulation at phase zero; coefficients remain inline for allocation-free updates.
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

    /// Recompute slow modulation at fixed intervals and preserve filter history between samples.
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
                // Key tracking: octaves of the note above or below C5 (261.63 Hz).
                let keys = self.cfg.key_track * (self.freq_a.max(1.0) / 261.626).log2();
                let oct = self.cfg.env_octaves * self.env + self.cfg.lfo_cutoff * lfo + keys;
                self.svf
                    .set(self.cfg.cutoff * 2f32.powf(oct), self.cfg.resonance, self.sample_rate);
            }
        }
        self.countdown -= 1;
        self.lfo_phase = (self.lfo_phase + self.lfo_inc).fract();
        self.env *= self.env_coef;
        if self.cfg.filter_on() {
            self.svf.process(x, self.cfg.mode)
        } else {
            x
        }
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
    /// Construct an inactive slot that owns no sample or oscillator allocation.
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
    /// Whether a source still sounds; release tails remain active until finished.
    pub fn is_active(&self) -> bool {
        !matches!(self.source, Source::Off)
    }

    /// Set up a sampler or synth voice with its volume envelope and clamped note velocity.
    /// Returns `None` for plugin or silent instruments, missing samples, samples shorter than
    /// two frames, or a playback window shorter than one source frame.
    pub fn for_instrument(
        instrument: &VoiceInstrument,
        root_key: u8,
        key: u8,
        velocity: f32,
        sample_rate: f32,
    ) -> Option<Self> {
        let velocity = velocity.clamp(0.0, 1.0);
        match instrument {
            VoiceInstrument::Sampler {
                sample,
                gain,
                pitch,
                start,
                end,
                reverse,
                attack,
                decay,
                sustain,
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
                    // With no attack set, the first sample frame lands exactly on the note's onset
                    // (samples are expected to start cleanly); the release declicks the end.
                    env: Envelope::start(
                        Adsr {
                            attack: *attack,
                            decay: *decay,
                            sustain: *sustain,
                            release: *release,
                        },
                        sample_rate,
                    ),
                    key,
                    gain: velocity * gain,
                    ..Self::default()
                })
            }
            VoiceInstrument::Synth {
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
            VoiceInstrument::Silent => None,
        }
    }

    /// Apply instrument automation and pitch bend while preserving oscillator phase,
    /// sample position, and the current envelope stage. Call with the note's original velocity.
    /// `root_key` is the sampler's MIDI root note; `bend_semitones` offsets pitch and
    /// `sample_rate` is the output rate in Hz. Mismatched source/instrument kinds do nothing.
    /// Sampler updates keep the loaded audio, loop, and filter; trim and direction changes
    /// take effect at the existing playback position. Newly enabled synth oscillators or
    /// modulation start with fresh state.
    pub fn update_instrument(
        &mut self,
        instrument: &VoiceInstrument,
        root_key: u8,
        velocity: f32,
        bend_semitones: f32,
        sample_rate: f32,
    ) {
        match (&mut self.source, instrument) {
            (
                Source::Synth { a, b, mods },
                VoiceInstrument::Synth {
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
                    looping: current_loop,
                    ..
                },
                VoiceInstrument::Sampler {
                    gain,
                    pitch,
                    start,
                    end,
                    reverse: rev,
                    attack,
                    decay,
                    sustain,
                    release,
                    looping,
                    filter,
                    ..
                },
            ) => {
                let semis = f64::from(self.key) - f64::from(root_key) + f64::from(*pitch + bend_semitones);
                *step = f64::from(sample.sample_rate) / f64::from(sample_rate) * 2f64.powf(semis / 12.0);
                *lo = f64::from(start.min(*end)) * sample.frames() as f64;
                *hi = f64::from(start.max(*end)) * sample.frames() as f64;
                *reverse = *rev;
                *current_loop = looping.and_then(|(a, b)| {
                    let a = (f64::from(a) * sample.frames() as f64).max(*lo);
                    let b = (f64::from(b) * sample.frames() as f64).min(*hi);
                    (b - a >= 4.0).then_some((a, b))
                });
                if let Some((cutoff, resonance)) = filter {
                    let filters = self.filter.get_or_insert_with(|| [Svf::default(); 2]);
                    for f in filters {
                        f.set(*cutoff, *resonance, sample_rate);
                    }
                } else {
                    self.filter = None;
                }
                self.gain = velocity * gain;
                self.env.set_adsr(Adsr {
                    attack: *attack,
                    decay: *decay,
                    sustain: *sustain,
                    release: *release,
                });
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

    /// Begin release from the current envelope level without retriggering.
    pub fn release(&mut self) {
        self.env.release();
    }

    /// Start `secs` (of output time) into the source: audio clips trimmed at the start, or
    /// playback starting inside a clip. Negative durations are treated as zero. `sample_rate`
    /// is the output rate in Hz; elapsed whole output samples advance every voice's envelope.
    /// Only sampler playback positions move; crossing the playback window or finishing the
    /// envelope turns the voice off, including for looping samples.
    pub fn skip(&mut self, secs: f64, sample_rate: f32) {
        let output_samples = secs.max(0.0) * f64::from(sample_rate);
        self.env.advance(output_samples as u64);
        if let Source::Sampler {
            pos,
            step,
            lo,
            hi,
            reverse,
            ..
        } = &mut self.source
        {
            let frames = output_samples * *step;
            if *reverse {
                *pos -= frames;
            } else {
                *pos += frames;
            }
            if *pos < *lo || *pos >= *hi {
                self.source = Source::Off;
            }
        }
        if self.env.is_done() {
            self.source = Source::Off;
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
