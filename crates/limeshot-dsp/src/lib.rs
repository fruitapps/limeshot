//! LimeShot DSP building blocks. Everything here is original and allocation-free per sample.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod envelope;
pub mod osc;
pub mod sample;

pub use envelope::{Adsr, Envelope};
pub use osc::Oscillator;
pub use sample::{BitDepth, SampleData, SampleError, load_wav, write_wav};

/// Decibels to linear gain. −120 dB and below is silence.
pub fn db_to_gain(db: f32) -> f32 {
    if !db.is_finite() || db <= -120.0 { 0.0 } else { 10f32.powf(db / 20.0) }
}

/// Linear gain to decibels (−∞ for silence).
pub fn gain_to_db(gain: f32) -> f32 {
    if gain <= 0.0 { f32::NEG_INFINITY } else { 20.0 * gain.log10() }
}

/// Balance pan law: centre is unity on both sides, the opposite side fades linearly, so panning
/// never boosts (and cannot push a centred signal into clipping). `pan` is −1 (left) to +1.
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let p = if pan.is_finite() { pan.clamp(-1.0, 1.0) } else { 0.0 };
    ((1.0 - p).min(1.0), (1.0 + p).min(1.0))
}

/// MIDI key to frequency (A4 = key 69 = 440 Hz).
pub fn key_to_hz(key: f32) -> f32 {
    440.0 * 2f32.powf((key - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_conversions() {
        assert!((db_to_gain(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_gain(-6.0206) - 0.5).abs() < 1e-4);
        assert_eq!(db_to_gain(-200.0), 0.0);
        assert!((gain_to_db(0.5) + 6.0206).abs() < 1e-3);
        assert_eq!(gain_to_db(0.0), f32::NEG_INFINITY);
    }

    #[test]
    fn pan_law() {
        assert_eq!(pan_gains(0.0), (1.0, 1.0));
        assert_eq!(pan_gains(-1.0), (1.0, 0.0));
        assert_eq!(pan_gains(1.0), (0.0, 1.0));
        assert_eq!(pan_gains(0.5), (0.5, 1.0));
        assert_eq!(pan_gains(f32::NAN), (1.0, 1.0));
    }

    #[test]
    fn tuning() {
        assert!((key_to_hz(69.0) - 440.0).abs() < 1e-3);
        assert!((key_to_hz(57.0) - 220.0).abs() < 1e-3);
        assert!((key_to_hz(60.0) - 261.626).abs() < 1e-2);
    }
}
