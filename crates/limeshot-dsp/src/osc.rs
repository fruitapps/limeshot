//! Band-limited oscillators (PolyBLEP for the discontinuous shapes).

/// The oscillator shape. Mirrors `limeshot_core::Waveform` without depending on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Sine,
    Square,
    Saw,
    Triangle,
}

/// A phase-accumulator oscillator.
#[derive(Debug, Clone, Copy)]
pub struct Oscillator {
    pub shape: Shape,
    phase: f32,
    inc: f32,
    /// Leaky-integrator state for the triangle.
    tri: f32,
}

/// PolyBLEP residual for a discontinuity at phase 0, `t` in [0, 1), `dt` = phase increment.
fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        0.0
    } else if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

impl Oscillator {
    pub fn new(shape: Shape, freq: f32, sample_rate: f32, phase: f32) -> Self {
        let mut o = Self { shape, phase: phase.rem_euclid(1.0), inc: 0.0, tri: 0.0 };
        o.set_frequency(freq, sample_rate);
        o
    }

    pub fn set_frequency(&mut self, freq: f32, sample_rate: f32) {
        let inc = freq / sample_rate.max(1.0);
        self.inc = if inc.is_finite() { inc.clamp(0.0, 0.5) } else { 0.0 };
    }

    /// One sample in −1..1, then advance.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        let t = self.phase;
        let dt = self.inc;
        let out = match self.shape {
            Shape::Sine => (t * std::f32::consts::TAU).sin(),
            Shape::Saw => 2.0 * t - 1.0 - poly_blep(t, dt),
            Shape::Square => square(t, dt),
            Shape::Triangle => {
                // Integrate a band-limited square; the leak keeps it centred.
                self.tri = dt * 4.0 * square(t, dt) + (1.0 - dt * 0.1) * self.tri;
                self.tri
            }
        };
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        out
    }
}

fn square(t: f32, dt: f32) -> f32 {
    let naive = if t < 0.5 { 1.0 } else { -1.0 };
    naive + poly_blep(t, dt) - poly_blep((t + 0.5).rem_euclid(1.0), dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(shape: Shape, freq: f32, n: usize) -> Vec<f32> {
        let mut o = Oscillator::new(shape, freq, 48_000.0, 0.0);
        (0..n).map(|_| o.next()).collect()
    }

    fn zero_crossings(x: &[f32]) -> usize {
        x.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count()
    }

    #[test]
    fn shapes_have_the_right_pitch_and_range() {
        for shape in [Shape::Sine, Shape::Square, Shape::Saw, Shape::Triangle] {
            let x = render(shape, 440.0, 48_000);
            let peak = x.iter().skip(4800).fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(peak > 0.7 && peak < 1.3, "{shape:?} peak {peak}");
            let zc = zero_crossings(&x[4800..]);
            assert!((394..=398).contains(&zc), "{shape:?} crossings {zc}");
            let mean: f32 = x[4800..].iter().sum::<f32>() / (x.len() - 4800) as f32;
            assert!(mean.abs() < 0.05, "{shape:?} DC {mean}");
        }
    }

    #[test]
    fn nonsense_frequency_is_silent() {
        let mut o = Oscillator::new(Shape::Saw, f32::NAN, 48_000.0, 0.0);
        let a = o.next();
        let b = o.next();
        assert_eq!(a, b);
    }
}
