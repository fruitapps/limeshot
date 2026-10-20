//! A mono low-pass for synth voices: the topology-preserving-transform state-variable filter
//! (V. Zavalishin, "The Art of VA Filter Design"), as in the Filter effect.

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, Default)]
pub struct Svf {
    g: f32,
    k: f32,
    ic1: f32,
    ic2: f32,
}

impl Svf {
    /// Cutoff in Hz, resonance 0–0.98.
    pub fn set(&mut self, cutoff: f32, resonance: f32, sample_rate: f32) {
        let fc = cutoff.clamp(10.0, sample_rate * 0.49);
        self.g = (PI * fc / sample_rate.max(1.0)).tan();
        self.k = 2.0 - 2.0 * resonance.clamp(0.0, 0.98);
    }

    /// Filter one sample (low-pass output).
    #[inline]
    pub fn lowpass(&mut self, x: f32) -> f32 {
        let a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        let a2 = self.g * a1;
        let a3 = self.g * a2;
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + a2 * v3;
        let v2 = self.ic2 + a2 * self.ic1 + a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        if !self.ic1.is_finite() || !self.ic2.is_finite() {
            self.ic1 = 0.0;
            self.ic2 = 0.0;
            return 0.0;
        }
        v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gain_at(freq: f32, cutoff: f32) -> f32 {
        let sr = 48_000.0;
        let mut f = Svf::default();
        f.set(cutoff, 0.0, sr);
        let mut peak = 0.0f32;
        for i in 0..48_000 {
            let y = f.lowpass((2.0 * PI * freq * i as f32 / sr).sin());
            if i > 24_000 {
                peak = peak.max(y.abs());
            }
        }
        peak
    }

    #[test]
    fn passes_below_and_cuts_above_the_cutoff() {
        assert!(gain_at(100.0, 2000.0) > 0.95);
        // 12 dB per octave: three octaves up is about −36 dB.
        assert!(gain_at(16_000.0, 2000.0) < 0.03);
    }
}
