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
        ping_pong: bool,
        filter_mode: crate::filter::Mode,
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
        ping_pong: bool,
        /// Last requested direction; `reverse` itself also changes at ping-pong turns.
        reverse_setting: bool,
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
    sampler_filter_mode: crate::filter::Mode,
}

/// An audio clip's fade-in and fade-out, counted in frames: `pos` from the clip's start, `left`
/// to its end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipFade {
    pub in_curve: f32,
    pub out_curve: f32,
    pub in_len: f64,
    pub pos: f64,
    pub out_len: f64,
    pub left: f64,
}

/// Monotonic quartic fade blend. Zero tension preserves linear gain exactly.
pub fn fade_curve(value: f32, tension: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    let bend = if tension >= 0.0 {
        1.0 - (1.0 - t).powi(4)
    } else {
        t.powi(4)
    };
    t + tension.abs().min(1.0) * (bend - t)
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
        fade_curve(g_in as f32, self.in_curve) * fade_curve(g_out as f32, self.out_curve)
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
            sampler_filter_mode: crate::filter::Mode::LowPass,
        }
    }
}

/// Fold arbitrarily large steps in constant time. Ping-pong endpoints are frame centers,
/// so a turn does not play an endpoint twice or interpolate beyond the loop's last frame.
fn loop_position(pos: &mut f64, reverse: &mut bool, ping_pong: bool, a: f64, b: f64) {
    if ping_pong {
        let span = b - 1.0 - a;
        if (!*reverse && *pos > b - 1.0) || (*reverse && *pos < a) {
            let phase = if *reverse { 2.0 * span - (*pos - a) } else { *pos - a };
            let phase = phase.rem_euclid(2.0 * span);
            *reverse = phase >= span;
            *pos = a + if *reverse { 2.0 * span - phase } else { phase };
        }
    } else if (!*reverse && *pos >= b) || (*reverse && *pos < a) {
        *pos = a + (*pos - a).rem_euclid(b - a);
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
                ping_pong,
                filter_mode,
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
                        ping_pong: *ping_pong,
                        reverse_setting: *reverse,
                    },
                    filter,
                    sampler_filter_mode: *filter_mode,
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
                    ping_pong: current_ping_pong,
                    reverse_setting,
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
                    ping_pong,
                    filter_mode,
                    filter,
                    ..
                },
            ) => {
                let semis = f64::from(self.key) - f64::from(root_key) + f64::from(*pitch + bend_semitones);
                *step = f64::from(sample.sample_rate) / f64::from(sample_rate) * 2f64.powf(semis / 12.0);
                *lo = f64::from(start.min(*end)) * sample.frames() as f64;
                *hi = f64::from(start.max(*end)) * sample.frames() as f64;
                // Parameter polling must not reset a ping-pong turn every audio block.
                if *reverse_setting != *rev || !*ping_pong || !*current_ping_pong {
                    *reverse = *rev;
                }
                *reverse_setting = *rev;
                *current_ping_pong = *ping_pong;
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
                self.sampler_filter_mode = *filter_mode;
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
        Self::preview_from(sample, 0.0, sample_rate)
    }

    /// Plays a sample once at its own pitch starting at `start_fraction` (`0.0..=1.0`) of the
    /// sample's length (browser scrub preview).
    pub fn preview_from(sample: Arc<SampleData>, start_fraction: f32, sample_rate: f32) -> Option<Self> {
        let frames = sample.frames() as f64;
        if frames < 2.0 {
            return None;
        }
        let frac = if start_fraction.is_finite() {
            f64::from(start_fraction.clamp(0.0, 1.0))
        } else {
            0.0
        };
        let pos = (frac * frames).min((frames - 1.0).max(0.0));
        let step = f64::from(sample.sample_rate) / f64::from(sample_rate);
        Some(Self {
            source: Source::Sampler {
                sample,
                pos,
                step,
                lo: 0.0,
                hi: frames,
                reverse: false,
                looping: None,
                ping_pong: false,
                reverse_setting: false,
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
    /// Only sampler playback positions move; held loops wrap/reflect like normal playback.
    /// Leaving a nonlooped playback window or finishing the envelope turns the voice off.
    pub fn skip(&mut self, secs: f64, sample_rate: f32) {
        let output_samples = secs.max(0.0) * f64::from(sample_rate);
        // Seconds reconstructed from an exact frame can land just below an integer. At a
        // loop discontinuity that tiny error would select the opposite side of the seam.
        let nearest = output_samples.round();
        let output_samples = if (output_samples - nearest).abs() <= 8.0 * f64::EPSILON * output_samples.max(1.0) {
            nearest
        } else {
            output_samples
        };
        self.env.advance(output_samples as u64);
        if let Source::Sampler {
            pos,
            step,
            lo,
            hi,
            reverse,
            looping,
            ping_pong,
            ..
        } = &mut self.source
        {
            let frames = output_samples * *step;
            if *reverse {
                *pos -= frames;
            } else {
                *pos += frames;
            }
            if let Some((a, b)) = *looping {
                loop_position(pos, reverse, *ping_pong, a, b);
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
                ping_pong,
                ..
            } => {
                if let Some((a, b)) = *looping {
                    loop_position(pos, reverse, *ping_pong, a, b);
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
                    f = (
                        l.process(f.0, self.sampler_filter_mode),
                        r.process(f.1, self.sampler_filter_mode),
                    );
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

#[cfg(test)]
mod fade_tests {
    #[test]
    fn fade_tension_is_monotonic_and_keeps_endpoints() {
        for tension in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            assert_eq!(super::fade_curve(0.0, tension), 0.0);
            assert_eq!(super::fade_curve(1.0, tension), 1.0);
            let mut previous = 0.0;
            for i in 0..=1000 {
                let t = i as f32 / 1000.0;
                let value = super::fade_curve(t, tension);
                assert!(value >= previous && value <= 1.0);
                if tension == 0.0 {
                    assert_eq!(value, t);
                }
                previous = value;
            }
        }
    }
}

#[cfg(test)]
mod sampler_loop_tests {
    use super::*;

    fn instrument(reverse: bool, ping_pong: bool) -> VoiceInstrument {
        VoiceInstrument::Sampler {
            sample: Some(Arc::new(SampleData::from_stereo(
                48000,
                (0..8).flat_map(|i| [i as f32 / 8.0; 2]).collect(),
            ))),
            gain: 1.0,
            pitch: 0.0,
            start: 0.0,
            end: 1.0,
            reverse,
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
            one_shot: false,
            looping: Some((0.25, 0.75)),
            ping_pong,
            filter: None,
            filter_mode: crate::filter::Mode::LowPass,
        }
    }

    #[test]
    fn ping_pong_turns_without_duplicate_endpoints_and_survives_parameter_polling() {
        for (reverse, expected) in [
            (false, vec![0, 1, 2, 3, 4, 5, 4, 3, 2, 3, 4, 5, 4, 3, 2, 3]),
            (true, vec![7, 6, 5, 4, 3, 2, 3, 4, 5, 4, 3, 2, 3, 4, 5, 4]),
        ] {
            let cfg = instrument(reverse, true);
            for block in [1, 3, 64, 333, 1024] {
                let mut voice = Voice::for_instrument(&cfg, 60, 60, 1.0, 48000.0).unwrap();
                let mut got = Vec::new();
                for i in 0..expected.len() {
                    if i % block == 0 {
                        voice.update_instrument(&cfg, 60, 1.0, 0.0, 48000.0);
                    }
                    got.push((voice.next_frame().0 * 8.0).round() as i32);
                }
                assert_eq!(got, expected, "reverse {reverse}, block {block}");
            }
        }
    }

    #[test]
    fn loop_seek_matches_rendering_and_extreme_steps_stay_bounded() {
        for reverse in [false, true] {
            for ping_pong in [false, true] {
                let cfg = instrument(reverse, ping_pong);
                for skipped in 0..100 {
                    let mut played = Voice::for_instrument(&cfg, 60, 60, 1.0, 48000.0).unwrap();
                    let mut sought = played.clone();
                    for _ in 0..skipped {
                        played.next_frame();
                    }
                    sought.skip(f64::from(skipped) / 48000.0, 48000.0);
                    for _ in 0..16 {
                        let (a, b) = (played.next_frame().0, sought.next_frame().0);
                        assert!(
                            (a - b).abs() < 1e-6,
                            "reverse {reverse}, ping-pong {ping_pong}, seek {skipped}: {a} != {b}"
                        );
                    }
                }
                for rate in [8000.0, 44100.0, 48000.0, 96000.0] {
                    let mut voice = Voice::for_instrument(&cfg, 0, 127, 1.0, rate).unwrap();
                    for _ in 0..4096 {
                        let (l, r) = voice.next_frame();
                        assert!(l.is_finite() && (0.0..1.0).contains(&l));
                        assert_eq!(l, r);
                        assert!(voice.is_active());
                    }
                    voice.release();
                    voice.next_frame();
                    assert!(!voice.is_active());
                }
            }
        }
    }

    #[test]
    fn sampler_high_pass_is_active_and_removes_low_frequency_content() {
        let render = |mode| {
            let mut cfg = instrument(false, false);
            if let VoiceInstrument::Sampler {
                sample,
                looping,
                filter,
                filter_mode,
                ..
            } = &mut cfg
            {
                *sample = Some(Arc::new(SampleData::from_stereo(
                    48000,
                    (0..48000)
                        .flat_map(|i| [(i as f32 * std::f32::consts::TAU * 100.0 / 48000.0).sin(); 2])
                        .collect(),
                )));
                *looping = None;
                *filter = Some((2000.0, 0.0));
                *filter_mode = mode;
            }
            let mut voice = Voice::for_instrument(&cfg, 60, 60, 1.0, 48000.0).unwrap();
            let mut energy = 0.0;
            for i in 0..48000 {
                let x = voice.next_frame().0;
                if i > 1000 {
                    energy += x * x;
                }
            }
            energy
        };
        assert!(render(crate::filter::Mode::HighPass) < render(crate::filter::Mode::LowPass) * 0.001);
        assert!(render(crate::filter::Mode::BandPass) < render(crate::filter::Mode::LowPass) * 0.01);
    }
}
