//! Musical time.
//!
//! Positions are integer ticks at [`PPQ`] ticks per quarter note. Ticks never drift: tempo only
//! changes how ticks map to samples, and that mapping is exact integer arithmetic
//! ([`Timeline`]), so the same musical event lands on the same sample whatever the buffer size.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Ticks per quarter note.
pub const PPQ: u64 = 960;

/// A musical position or duration in ticks.
pub type Tick = u64;

/// One step of the step sequencer: a 16th note.
pub const STEP_TICKS: Tick = PPQ / 4;

/// Tempo in thousandths of a beat per minute (120 BPM = 120_000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tempo(u32);

impl Tempo {
    pub const MIN_MILLI_BPM: u32 = 10_000;
    pub const MAX_MILLI_BPM: u32 = 522_000;

    /// A tempo clamped to the supported 10–522 BPM range.
    pub fn from_milli_bpm(milli: u32) -> Self {
        Self(milli.clamp(Self::MIN_MILLI_BPM, Self::MAX_MILLI_BPM))
    }

    /// A tempo from floating BPM, rounded to 1/1000 BPM and clamped. NaN gives 120 BPM.
    pub fn from_bpm(bpm: f64) -> Self {
        if !bpm.is_finite() {
            return Self::default();
        }
        let milli = (bpm * 1000.0).round().clamp(Self::MIN_MILLI_BPM as f64, Self::MAX_MILLI_BPM as f64);
        Self(milli as u32)
    }

    pub fn milli_bpm(self) -> u32 {
        self.0
    }

    pub fn bpm(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

impl Default for Tempo {
    fn default() -> Self {
        Self(120_000)
    }
}

impl fmt::Display for Tempo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:03}", self.0 / 1000, self.0 % 1000)
    }
}

/// A time signature. The denominator is a power of two from 1 to 32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl TimeSignature {
    /// A valid signature: the numerator is clamped to 1–32 and the denominator rounded to a power
    /// of two in 1–32.
    pub fn new(numerator: u8, denominator: u8) -> Self {
        let denominator = match denominator {
            0..=1 => 1,
            2 => 2,
            3..=5 => 4,
            6..=11 => 8,
            12..=23 => 16,
            _ => 32,
        };
        Self { numerator: numerator.clamp(1, 32), denominator }
    }

    pub fn ticks_per_beat(self) -> Tick {
        PPQ * 4 / Tick::from(self.denominator.max(1))
    }

    pub fn ticks_per_bar(self) -> Tick {
        self.ticks_per_beat() * Tick::from(self.numerator.max(1))
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self { numerator: 4, denominator: 4 }
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

/// A 1-based bar:beat:tick position for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarBeatTick {
    pub bar: u64,
    pub beat: u64,
    pub tick: u64,
}

impl BarBeatTick {
    pub fn from_ticks(ticks: Tick, sig: TimeSignature) -> Self {
        let bar_len = sig.ticks_per_bar();
        let beat_len = sig.ticks_per_beat();
        let in_bar = ticks % bar_len;
        Self { bar: ticks / bar_len + 1, beat: in_bar / beat_len + 1, tick: in_bar % beat_len }
    }
}

impl fmt::Display for BarBeatTick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}:{:03}", self.bar, self.beat, self.tick)
    }
}

/// Grid resolution for snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Snap {
    None,
    Bar,
    Beat,
    HalfBeat,
    #[default]
    Step,
    HalfStep,
    QuarterStep,
}

impl Snap {
    pub const ALL: [Snap; 7] =
        [Snap::Bar, Snap::Beat, Snap::HalfBeat, Snap::Step, Snap::HalfStep, Snap::QuarterStep, Snap::None];

    /// Grid size in ticks, or `None` for no snapping.
    pub fn ticks(self, sig: TimeSignature) -> Option<Tick> {
        match self {
            Snap::None => None,
            Snap::Bar => Some(sig.ticks_per_bar()),
            Snap::Beat => Some(sig.ticks_per_beat()),
            Snap::HalfBeat => Some((sig.ticks_per_beat() / 2).max(1)),
            Snap::Step => Some(STEP_TICKS),
            Snap::HalfStep => Some(STEP_TICKS / 2),
            Snap::QuarterStep => Some(STEP_TICKS / 4),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Snap::None => "None",
            Snap::Bar => "Bar",
            Snap::Beat => "Beat",
            Snap::HalfBeat => "1/2 beat",
            Snap::Step => "Step",
            Snap::HalfStep => "1/2 step",
            Snap::QuarterStep => "1/4 step",
        }
    }

    /// Round `ticks` down to the grid.
    pub fn floor(self, ticks: Tick, sig: TimeSignature) -> Tick {
        match self.ticks(sig) {
            Some(g) if g > 0 => ticks / g * g,
            _ => ticks,
        }
    }

    /// Round `ticks` to the nearest grid line.
    pub fn round(self, ticks: Tick, sig: TimeSignature) -> Tick {
        match self.ticks(sig) {
            Some(g) if g > 0 => (ticks + g / 2) / g * g,
            _ => ticks,
        }
    }
}

/// Exact mapping between ticks and samples for one tempo, anchored at an origin.
///
/// A tick `t` sounds at the first sample whose time is at or after it:
/// `sample(t) = origin_sample + ceil((t - origin_tick) * A / B)` with
/// `A = 60_000 * sample_rate` and `B = milli_bpm * PPQ`. All arithmetic is `u128`, so there is no
/// rounding drift however long playback runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeline {
    pub origin_tick: Tick,
    pub origin_sample: u64,
    pub tempo: Tempo,
    pub sample_rate: u32,
}

impl Timeline {
    pub fn new(tempo: Tempo, sample_rate: u32) -> Self {
        Self { origin_tick: 0, origin_sample: 0, tempo, sample_rate: sample_rate.max(1) }
    }

    fn num(&self) -> u128 {
        60_000u128 * u128::from(self.sample_rate)
    }

    fn den(&self) -> u128 {
        u128::from(self.tempo.milli_bpm().max(1)) * u128::from(PPQ)
    }

    /// The sample at which tick `t` (at or after the origin) sounds.
    pub fn sample_of_tick(&self, t: Tick) -> u64 {
        let dt = u128::from(t.saturating_sub(self.origin_tick));
        let ds = (dt * self.num()).div_ceil(self.den());
        self.origin_sample.saturating_add(u64::try_from(ds).unwrap_or(u64::MAX))
    }

    /// The smallest tick that sounds at or after sample `s`.
    pub fn first_tick_at_or_after(&self, s: u64) -> Tick {
        let ds = s.saturating_sub(self.origin_sample);
        if ds == 0 {
            return self.origin_tick;
        }
        // ceil(dt * A / B) >= ds  <=>  dt > (ds - 1) * B / A
        let dt = (u128::from(ds - 1) * self.den()) / self.num() + 1;
        self.origin_tick.saturating_add(u64::try_from(dt).unwrap_or(u64::MAX))
    }

    /// The musical position (in ticks, fractional part dropped) at sample `s`.
    pub fn tick_at_sample(&self, s: u64) -> Tick {
        let ds = u128::from(s.saturating_sub(self.origin_sample));
        let dt = ds * self.den() / self.num();
        self.origin_tick.saturating_add(u64::try_from(dt).unwrap_or(u64::MAX))
    }

    /// Duration of `ticks` in seconds at this tempo.
    pub fn seconds(&self, ticks: Tick) -> f64 {
        ticks as f64 * 60.0 / (self.tempo.bpm() * PPQ as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo_clamps_and_formats() {
        assert_eq!(Tempo::from_bpm(5.0).milli_bpm(), 10_000);
        assert_eq!(Tempo::from_bpm(1000.0).milli_bpm(), 522_000);
        assert_eq!(Tempo::from_bpm(f64::NAN), Tempo::default());
        assert_eq!(Tempo::from_bpm(128.5).to_string(), "128.500");
    }

    #[test]
    fn signature_lengths() {
        let s = TimeSignature::default();
        assert_eq!(s.ticks_per_beat(), 960);
        assert_eq!(s.ticks_per_bar(), 3840);
        let s = TimeSignature::new(6, 8);
        assert_eq!(s.ticks_per_beat(), 480);
        assert_eq!(s.ticks_per_bar(), 2880);
        assert_eq!(TimeSignature::new(0, 7).denominator, 8);
    }

    #[test]
    fn bar_beat_tick_display() {
        let s = TimeSignature::default();
        assert_eq!(BarBeatTick::from_ticks(0, s).to_string(), "1:01:000");
        assert_eq!(BarBeatTick::from_ticks(3840 + 960 + 5, s).to_string(), "2:02:005");
    }

    #[test]
    fn snapping() {
        let s = TimeSignature::default();
        assert_eq!(Snap::Step.floor(250, s), 240);
        assert_eq!(Snap::Step.round(119, s), 0);
        assert_eq!(Snap::Step.round(120, s), 240);
        assert_eq!(Snap::Bar.floor(5000, s), 3840);
        assert_eq!(Snap::None.round(77, s), 77);
    }

    #[test]
    fn one_beat_at_120_bpm_is_half_a_second() {
        let tl = Timeline::new(Tempo::from_bpm(120.0), 48_000);
        assert_eq!(tl.sample_of_tick(PPQ), 24_000);
        assert_eq!(tl.sample_of_tick(STEP_TICKS), 6_000);
        assert_eq!(tl.tick_at_sample(24_000), PPQ);
    }

    #[test]
    fn no_drift_after_an_hour() {
        // 44.1 kHz at 133.333 BPM does not divide evenly; the mapping must stay exact.
        let tl = Timeline::new(Tempo::from_milli_bpm(133_333), 44_100);
        let beats: u64 = 133_333 * 60 / 1000; // about an hour of beats
        let s = tl.sample_of_tick(beats * PPQ);
        let exact = (beats as f64 * 60.0 / 133.333 * 44_100.0).ceil() as u64;
        assert!(s.abs_diff(exact) <= 1, "{s} vs {exact}");
    }

    #[test]
    fn first_tick_inverts_sample_of_tick() {
        for &(bpm, sr) in &[(120_000, 48_000), (133_333, 44_100), (87_500, 96_000), (522_000, 22_050)] {
            let tl = Timeline { origin_tick: 1000, origin_sample: 777, tempo: Tempo::from_milli_bpm(bpm), sample_rate: sr };
            for s in (777..60_000).step_by(997) {
                let t = tl.first_tick_at_or_after(s);
                assert!(tl.sample_of_tick(t) >= s);
                if t > tl.origin_tick {
                    assert!(tl.sample_of_tick(t - 1) < s);
                }
            }
        }
    }
}
