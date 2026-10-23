//! Insert effect processors.
//!
//! Every processor allocates everything it needs in `new` (delay lines are sized for the
//! longest setting), so `set_params` and `process` never allocate and can run on the audio
//! thread. Buffers are interleaved stereo (`[l0, r0, l1, r1, …]`).
//!
//! `set_params` takes values in the units and order of `limeshot_core::EffectKind::params()`;
//! each struct documents its order. Algorithms are standard textbook techniques (RBJ biquads,
//! Zavalishin's TPT state-variable filter, a Schroeder/Moorer comb–allpass reverb), implemented
//! here from their published descriptions.

use crate::db_to_gain;
use std::f32::consts::PI;

/// An insert effect.
pub trait Effect: Send {
    /// Apply parameters (see the implementing type for their order). Missing values keep their
    /// previous setting.
    fn set_params(&mut self, params: &[f32]);
    /// Process interleaved stereo in place.
    fn process(&mut self, buf: &mut [f32]);
    /// Clear internal state (delay lines, envelopes).
    fn reset(&mut self);
    /// Process with an external sidechain signal (interleaved stereo, as long as `buf`).
    /// Effects without a sidechain input ignore it.
    fn process_sidechain(&mut self, buf: &mut [f32], _sidechain: &[f32]) {
        self.process(buf);
    }
    /// The mixer track whose signal keys this effect; 0 means its own input.
    fn sidechain_source(&self) -> usize {
        0
    }
    /// The largest gain reduction in dB during the last processed block (dynamics only).
    fn gain_reduction_db(&self) -> f32 {
        0.0
    }
    /// The transport for the next `process` call (tempo-synced plugins read it).
    fn set_transport(&mut self, _transport: &crate::Transport) {}
    /// Set a hosted plugin's parameter `id` (normalised) at frame `offset` of the next `process`
    /// call (automation). Only plugins have such parameters.
    fn set_plugin_param(&mut self, _id: u32, _offset: u32, _value: f64) {}
    /// How many frames the output lags the input (a plugin's reported latency).
    fn latency(&self) -> u32 {
        0
    }
}

fn get(params: &[f32], i: usize, fallback: f32) -> f32 {
    params.get(i).copied().filter(|v| v.is_finite()).unwrap_or(fallback)
}

/// One-pole smoothing coefficient for a time constant in seconds.
fn coeff(seconds: f32, sample_rate: f32) -> f32 {
    if seconds <= 0.0 {
        0.0
    } else {
        (-1.0 / (seconds * sample_rate)).exp()
    }
}

// ---------------------------------------------------------------- Utility

/// Params: gain dB, width (0 mono … 1 unchanged … 2 wide), polarity (0 normal, 1 inverted).
pub struct Utility {
    gain: f32,
    width: f32,
    invert: bool,
}

impl Utility {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            width: 1.0,
            invert: false,
        }
    }
}

impl Default for Utility {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Utility {
    fn set_params(&mut self, p: &[f32]) {
        self.gain = db_to_gain(get(p, 0, 0.0));
        self.width = get(p, 1, 1.0).clamp(0.0, 2.0);
        self.invert = get(p, 2, 0.0) >= 0.5;
    }

    fn process(&mut self, buf: &mut [f32]) {
        let g = if self.invert { -self.gain } else { self.gain };
        for f in buf.as_chunks_mut::<2>().0 {
            let m = 0.5 * (f[0] + f[1]);
            let s = 0.5 * (f[0] - f[1]) * self.width;
            f[0] = (m + s) * g;
            f[1] = (m - s) * g;
        }
    }

    fn reset(&mut self) {}
}

// ---------------------------------------------------------------- Biquad / EQ

/// Direct-form-I biquad with stereo state.
#[derive(Debug, Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: [f32; 2],
    x2: [f32; 2],
    y1: [f32; 2],
    y2: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    LowShelf,
    Peak,
    HighShelf,
}

impl Biquad {
    /// Audio EQ cookbook coefficients (R. Bristow-Johnson).
    fn design(&mut self, shape: Shape, freq: f32, gain_db: f32, q: f32, sr: f32) {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (freq / sr).clamp(1e-5, 0.49);
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q.max(0.05));
        let (b0, b1, b2, a0, a1, a2) = match shape {
            Shape::Peak => (
                1.0 + alpha * a,
                -2.0 * cos,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos,
                1.0 - alpha / a,
            ),
            Shape::LowShelf => {
                let k = 2.0 * a.sqrt() * alpha;
                (
                    a * ((a + 1.0) - (a - 1.0) * cos + k),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                    a * ((a + 1.0) - (a - 1.0) * cos - k),
                    (a + 1.0) + (a - 1.0) * cos + k,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                    (a + 1.0) + (a - 1.0) * cos - k,
                )
            }
            Shape::HighShelf => {
                let k = 2.0 * a.sqrt() * alpha;
                (
                    a * ((a + 1.0) + (a - 1.0) * cos + k),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                    a * ((a + 1.0) + (a - 1.0) * cos - k),
                    (a + 1.0) - (a - 1.0) * cos + k,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos),
                    (a + 1.0) - (a - 1.0) * cos - k,
                )
            }
        };
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline]
    fn tick(&mut self, c: usize, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1[c] + self.b2 * self.x2[c] - self.a1 * self.y1[c] - self.a2 * self.y2[c];
        self.x2[c] = self.x1[c];
        self.x1[c] = x;
        self.y2[c] = self.y1[c];
        self.y1[c] = if y.is_finite() { y } else { 0.0 };
        self.y1[c]
    }

    /// Magnitude response in dB at `freq`.
    fn response_db(&self, freq: f32, sr: f32) -> f32 {
        let w = 2.0 * PI * (freq / sr).clamp(0.0, 0.5);
        let (s1, c1) = w.sin_cos();
        let (s2, c2) = (2.0 * w).sin_cos();
        let (nr, ni) = (self.b0 + self.b1 * c1 + self.b2 * c2, -(self.b1 * s1 + self.b2 * s2));
        let (dr, di) = (1.0 + self.a1 * c1 + self.a2 * c2, -(self.a1 * s1 + self.a2 * s2));
        let mag2 = (nr * nr + ni * ni) / (dr * dr + di * di).max(1e-20);
        10.0 * mag2.max(1e-20).log10()
    }

    fn clear(&mut self) {
        self.x1 = [0.0; 2];
        self.x2 = [0.0; 2];
        self.y1 = [0.0; 2];
        self.y2 = [0.0; 2];
    }
}

/// Params: low gain dB, low freq Hz, mid gain dB, mid freq Hz, mid Q, high gain dB, high freq Hz.
pub struct Eq3 {
    sr: f32,
    bands: [Biquad; 3],
    active: [bool; 3],
}

impl Eq3 {
    pub fn new(sample_rate: f32) -> Self {
        let mut e = Self {
            sr: sample_rate.max(1.0),
            bands: [Biquad::default(); 3],
            active: [false; 3],
        };
        e.set_params(&[0.0, 120.0, 0.0, 1000.0, 0.8, 0.0, 6000.0]);
        e
    }
}

/// One band of a mixer track's EQ as (frequency Hz, gain dB, width 0–1).
pub type EqBandParams = (f32, f32, f32);

/// The Q a band's width 0–1 stands for. The peak's width is its bandwidth, 0.1 to 4 octaves
/// (FL Studio's default 0.27 is about 1.15 octaves); shelves get gentler as the width grows.
pub fn eq_width_to_q(band: usize, width: f32) -> f32 {
    let w = width.clamp(0.0, 1.0);
    if band == 1 {
        let octaves = 0.1 + 3.9 * w;
        let x = std::f32::consts::LN_2 / 2.0 * octaves;
        1.0 / (2.0 * x.sinh())
    } else {
        1.0 - 0.6 * w
    }
}

impl Eq3 {
    /// Design the three bands of a mixer track's EQ (low shelf, peak, high shelf). Does not
    /// allocate: safe on the audio thread.
    pub fn set_bands(&mut self, bands: &[EqBandParams; 3]) {
        let shapes = [Shape::LowShelf, Shape::Peak, Shape::HighShelf];
        for (i, ((f, g, w), shape)) in bands.iter().zip(shapes).enumerate() {
            self.bands[i].design(shape, *f, *g, eq_width_to_q(i, *w), self.sr);
            self.active[i] = g.abs() > 0.01;
        }
    }

    /// The EQ's response in dB at `freq` (for drawing its curve).
    pub fn response_db(&self, freq: f32) -> f32 {
        self.bands
            .iter()
            .zip(self.active)
            .filter(|(_, on)| *on)
            .map(|(b, _)| b.response_db(freq, self.sr))
            .sum()
    }
}

impl Effect for Eq3 {
    fn set_params(&mut self, p: &[f32]) {
        if p.len() >= 17 && get(p, 7, 0.0) >= 0.5 {
            self.set_bands(&[(p[8], p[9], p[10]), (p[11], p[12], p[13]), (p[14], p[15], p[16])]);
            return;
        }
        let (lg, lf, mg, mf, mq, hg, hf) = (
            get(p, 0, 0.0),
            get(p, 1, 120.0),
            get(p, 2, 0.0),
            get(p, 3, 1000.0),
            get(p, 4, 0.8),
            get(p, 5, 0.0),
            get(p, 6, 6000.0),
        );
        self.bands[0].design(Shape::LowShelf, lf, lg, 0.707, self.sr);
        self.bands[1].design(Shape::Peak, mf, mg, mq, self.sr);
        self.bands[2].design(Shape::HighShelf, hf, hg, 0.707, self.sr);
        self.active = [lg.abs() > 0.01, mg.abs() > 0.01, hg.abs() > 0.01];
    }

    fn process(&mut self, buf: &mut [f32]) {
        for (b, on) in self.bands.iter_mut().zip(self.active) {
            if !on {
                continue;
            }
            for f in buf.as_chunks_mut::<2>().0 {
                f[0] = b.tick(0, f[0]);
                f[1] = b.tick(1, f[1]);
            }
        }
    }

    fn reset(&mut self) {
        for b in &mut self.bands {
            b.clear();
        }
    }
}

// ---------------------------------------------------------------- Filter

/// Topology-preserving-transform state-variable filter (V. Zavalishin, "The Art of VA Filter
/// Design"). Params: mode (0 low-pass, 1 band-pass, 2 high-pass), cutoff Hz, resonance 0–0.95,
/// mix 0–1.
pub struct Filter {
    sr: f32,
    mode: u8,
    g: f32,
    k: f32,
    mix: f32,
    ic1: [f32; 2],
    ic2: [f32; 2],
}

impl Filter {
    pub fn new(sample_rate: f32) -> Self {
        let mut f = Self {
            sr: sample_rate.max(1.0),
            mode: 0,
            g: 0.0,
            k: 1.0,
            mix: 1.0,
            ic1: [0.0; 2],
            ic2: [0.0; 2],
        };
        f.set_params(&[0.0, 2000.0, 0.2, 1.0]);
        f
    }
}

impl Effect for Filter {
    fn set_params(&mut self, p: &[f32]) {
        self.mode = get(p, 0, 0.0).round().clamp(0.0, 2.0) as u8;
        let fc = get(p, 1, 2000.0).clamp(10.0, self.sr * 0.49);
        self.g = (PI * fc / self.sr).tan();
        let res = get(p, 2, 0.2).clamp(0.0, 0.98);
        self.k = 2.0 - 2.0 * res;
        self.mix = get(p, 3, 1.0).clamp(0.0, 1.0);
    }

    // Per-channel state lives in parallel arrays indexed by channel.
    #[allow(clippy::needless_range_loop)]
    fn process(&mut self, buf: &mut [f32]) {
        let (g, k) = (self.g, self.k);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        for f in buf.as_chunks_mut::<2>().0 {
            for c in 0..2 {
                let v0 = f[c];
                let v3 = v0 - self.ic2[c];
                let v1 = a1 * self.ic1[c] + a2 * v3;
                let v2 = self.ic2[c] + a2 * self.ic1[c] + a3 * v3;
                self.ic1[c] = 2.0 * v1 - self.ic1[c];
                self.ic2[c] = 2.0 * v2 - self.ic2[c];
                if !self.ic1[c].is_finite() || !self.ic2[c].is_finite() {
                    self.ic1[c] = 0.0;
                    self.ic2[c] = 0.0;
                }
                let y = match self.mode {
                    0 => v2,
                    1 => v1,
                    _ => v0 - k * v1 - v2,
                };
                f[c] = v0 + (y - v0) * self.mix;
            }
        }
    }

    fn reset(&mut self) {
        self.ic1 = [0.0; 2];
        self.ic2 = [0.0; 2];
    }
}

// ---------------------------------------------------------------- Delay

/// A circular delay line with fractional (linear) reads.
struct Line {
    buf: Vec<f32>,
    pos: usize,
}

impl Line {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len.max(2)],
            pos: 0,
        }
    }

    #[inline]
    fn read(&self, delay: f32) -> f32 {
        let n = self.buf.len();
        let d = delay.clamp(1.0, (n - 2) as f32);
        let i = d as usize;
        let frac = d - i as f32;
        let a = self.buf[(self.pos + n - i) % n];
        let b = self.buf[(self.pos + n - i - 1) % n];
        a + (b - a) * frac
    }

    #[inline]
    fn write(&mut self, x: f32) {
        self.pos = (self.pos + 1) % self.buf.len();
        self.buf[self.pos] = x;
    }

    fn clear(&mut self) {
        self.buf.fill(0.0);
    }
}

/// Stereo delay. Params: time ms, feedback 0–0.95, spread 0–1 (cross-feedback and a longer right
/// side), tone Hz (low-pass in the feedback path), mix 0–1.
pub struct Delay {
    sr: f32,
    lines: [Line; 2],
    time: f32,
    target: f32,
    feedback: f32,
    spread: f32,
    tone: f32,
    lp: [f32; 2],
    mix: f32,
    /// False until parameters are first applied after construction; then the time jumps instead
    /// of gliding.
    primed: bool,
}

impl Delay {
    pub const MAX_MS: f32 = 2500.0;

    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let len = (Self::MAX_MS / 1000.0 * sr) as usize + 4;
        let mut d = Self {
            sr,
            lines: [Line::new(len), Line::new(len)],
            time: 0.0,
            target: 0.0,
            feedback: 0.4,
            spread: 0.5,
            tone: 0.5,
            lp: [0.0; 2],
            mix: 0.3,
            primed: false,
        };
        d.set_params(&[375.0, 0.4, 0.5, 6000.0, 0.3]);
        d.primed = false;
        d
    }
}

impl Effect for Delay {
    fn set_params(&mut self, p: &[f32]) {
        self.target = (get(p, 0, 375.0).clamp(1.0, 2000.0) / 1000.0) * self.sr;
        if !self.primed {
            self.time = self.target;
            self.primed = true;
        }
        self.feedback = get(p, 1, 0.4).clamp(0.0, 0.95);
        self.spread = get(p, 2, 0.5).clamp(0.0, 1.0);
        let tone = get(p, 3, 6000.0).clamp(100.0, self.sr * 0.45);
        self.tone = 1.0 - (-2.0 * PI * tone / self.sr).exp();
        self.mix = get(p, 4, 0.3).clamp(0.0, 1.0);
    }

    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            // Glide towards a new delay time to avoid clicks.
            self.time += (self.target - self.time) * 0.0005;
            let tl = self.time;
            let tr = self.time * (1.0 + 0.12 * self.spread);
            let wl = self.lines[0].read(tl);
            let wr = self.lines[1].read(tr);
            self.lp[0] += (wl - self.lp[0]) * self.tone;
            self.lp[1] += (wr - self.lp[1]) * self.tone;
            let (fl, fr) = (self.lp[0] * self.feedback, self.lp[1] * self.feedback);
            let x = self.spread;
            self.lines[0].write(f[0] + fl * (1.0 - x) + fr * x);
            self.lines[1].write(f[1] + fr * (1.0 - x) + fl * x);
            f[0] += (wl - f[0]) * self.mix;
            f[1] += (wr - f[1]) * self.mix;
        }
    }

    fn reset(&mut self) {
        for l in &mut self.lines {
            l.clear();
        }
        self.lp = [0.0; 2];
    }
}

// ---------------------------------------------------------------- Reverb

struct Comb {
    line: Line,
    len: f32,
    store: f32,
}

struct Allpass {
    line: Line,
    len: f32,
}

/// Algorithmic room: per side, eight parallel damped feedback combs into four series allpasses,
/// after a pre-delay. Params: size 0–1 (decay), damping 0–1, pre-delay ms, width 0–1, mix 0–1.
pub struct Reverb {
    sr: f32,
    pre: [Line; 2],
    pre_len: f32,
    combs: [Vec<Comb>; 2],
    allpasses: [Vec<Allpass>; 2],
    feedback: f32,
    damp: f32,
    width: f32,
    mix: f32,
}

/// Comb and allpass lengths in milliseconds, chosen to be mutually prime-ish at common rates.
const COMB_MS: [f32; 8] = [25.31, 26.94, 28.96, 30.75, 32.24, 33.81, 35.31, 36.67];
const ALLPASS_MS: [f32; 4] = [12.61, 10.0, 7.73, 5.1];
/// Right-side offset for stereo decorrelation.
const STEREO_MS: f32 = 0.52;

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let ms = |m: f32| m / 1000.0 * sr;
        let side = |offset: f32| {
            let combs = COMB_MS
                .iter()
                .map(|m| {
                    let len = ms(m + offset);
                    Comb {
                        line: Line::new(len as usize + 4),
                        len,
                        store: 0.0,
                    }
                })
                .collect::<Vec<_>>();
            let aps = ALLPASS_MS
                .iter()
                .map(|m| {
                    let len = ms(m + offset);
                    Allpass {
                        line: Line::new(len as usize + 4),
                        len,
                    }
                })
                .collect::<Vec<_>>();
            (combs, aps)
        };
        let (cl, al) = side(0.0);
        let (cr, ar) = side(STEREO_MS);
        let pre_max = ms(130.0) as usize + 4;
        let mut r = Self {
            sr,
            pre: [Line::new(pre_max), Line::new(pre_max)],
            pre_len: 1.0,
            combs: [cl, cr],
            allpasses: [al, ar],
            feedback: 0.84,
            damp: 0.2,
            width: 1.0,
            mix: 0.25,
        };
        r.set_params(&[0.6, 0.4, 10.0, 1.0, 0.25]);
        r
    }
}

impl Effect for Reverb {
    fn set_params(&mut self, p: &[f32]) {
        self.feedback = 0.70 + 0.28 * get(p, 0, 0.6).clamp(0.0, 1.0);
        self.damp = 0.05 + 0.6 * get(p, 1, 0.4).clamp(0.0, 1.0);
        self.pre_len = (get(p, 2, 10.0).clamp(0.0, 120.0) / 1000.0 * self.sr).max(1.0);
        self.width = get(p, 3, 1.0).clamp(0.0, 1.0);
        self.mix = get(p, 4, 0.25).clamp(0.0, 1.0);
    }

    // Per-channel state lives in parallel arrays indexed by channel.
    #[allow(clippy::needless_range_loop)]
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let input = 0.5 * (f[0] + f[1]) * 0.12;
            let mut wet = [0.0f32; 2];
            for c in 0..2 {
                self.pre[c].write(input);
                let x = self.pre[c].read(self.pre_len);
                let mut sum = 0.0;
                for comb in &mut self.combs[c] {
                    let y = comb.line.read(comb.len);
                    comb.store = y + (comb.store - y) * self.damp;
                    comb.line.write(x + comb.store * self.feedback);
                    sum += y;
                }
                let mut y = sum;
                for ap in &mut self.allpasses[c] {
                    let delayed = ap.line.read(ap.len);
                    let v = y + delayed * 0.5;
                    ap.line.write(v);
                    y = delayed - v * 0.5;
                }
                wet[c] = if y.is_finite() { y } else { 0.0 };
            }
            // Width: blend the two decorrelated sides towards mono.
            let w = 0.5 + 0.5 * self.width;
            let l = wet[0] * w + wet[1] * (1.0 - w);
            let r = wet[1] * w + wet[0] * (1.0 - w);
            f[0] = f[0] * (1.0 - self.mix) + l * self.mix;
            f[1] = f[1] * (1.0 - self.mix) + r * self.mix;
        }
    }

    fn reset(&mut self) {
        for c in 0..2 {
            self.pre[c].clear();
            for comb in &mut self.combs[c] {
                comb.line.clear();
                comb.store = 0.0;
            }
            for ap in &mut self.allpasses[c] {
                ap.line.clear();
            }
        }
    }
}

// ---------------------------------------------------------------- Dynamics

/// Feed-forward compressor with a stereo-linked peak detector and a hard knee.
/// Params: threshold dB, ratio, attack ms, release ms, makeup dB.
pub struct Compressor {
    sr: f32,
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    env: f32,
    /// Gain reduction of the last sample, in dB (for metering).
    pub reduction_db: f32,
    /// The most gain reduction during the last block, in dB.
    block_reduction: f32,
    /// Mixer track keying the detector (0: the compressor's own input).
    sidechain: usize,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        let mut c = Self {
            sr: sample_rate.max(1.0),
            threshold: -18.0,
            ratio: 4.0,
            attack: 0.0,
            release: 0.0,
            makeup: 1.0,
            env: 0.0,
            reduction_db: 0.0,
            block_reduction: 0.0,
            sidechain: 0,
        };
        c.set_params(&[-18.0, 4.0, 10.0, 120.0, 0.0, 0.0]);
        c
    }

    /// Compress `buf`, following `key` (the sidechain) when given, else `buf` itself.
    fn run(&mut self, buf: &mut [f32], key: Option<&[f32]>) {
        let mut most = 0.0f32;
        for (n, f) in buf.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let peak = match key {
                Some(k) => k.get(2 * n..2 * n + 2).map_or(0.0, |s| s[0].abs().max(s[1].abs())),
                None => f[0].abs().max(f[1].abs()),
            };
            let k = if peak > self.env { self.attack } else { self.release };
            self.env = peak + (self.env - peak) * k;
            let level = crate::gain_to_db(self.env.max(1e-9));
            let over = level - self.threshold;
            let reduce = if over > 0.0 {
                over * (1.0 - 1.0 / self.ratio)
            } else {
                0.0
            };
            self.reduction_db = reduce;
            most = most.max(reduce);
            let g = db_to_gain(-reduce) * self.makeup;
            f[0] *= g;
            f[1] *= g;
        }
        self.block_reduction = most;
    }
}

impl Effect for Compressor {
    fn set_params(&mut self, p: &[f32]) {
        self.threshold = get(p, 0, -18.0).clamp(-80.0, 0.0);
        self.ratio = get(p, 1, 4.0).clamp(1.0, 100.0);
        self.attack = coeff(get(p, 2, 10.0).max(0.01) / 1000.0, self.sr);
        self.release = coeff(get(p, 3, 120.0).max(1.0) / 1000.0, self.sr);
        self.makeup = db_to_gain(get(p, 4, 0.0));
        self.sidechain = get(p, 5, 0.0).round().clamp(0.0, 1023.0) as usize;
    }

    fn process(&mut self, buf: &mut [f32]) {
        self.run(buf, None);
    }

    fn process_sidechain(&mut self, buf: &mut [f32], sidechain: &[f32]) {
        self.run(buf, Some(sidechain));
    }

    fn sidechain_source(&self) -> usize {
        self.sidechain
    }

    fn gain_reduction_db(&self) -> f32 {
        self.block_reduction
    }

    fn reset(&mut self) {
        self.env = 0.0;
        self.reduction_db = 0.0;
        self.block_reduction = 0.0;
    }
}

/// Peak limiter with instant attack: the output never exceeds the ceiling.
/// Params: ceiling dB, release ms.
pub struct Limiter {
    sr: f32,
    ceiling: f32,
    release: f32,
    env: f32,
    reduction_db: f32,
}

impl Limiter {
    /// Create a limiter with a −0.3 dB ceiling and 80 ms release.
    /// `sample_rate` is in Hz and is clamped to at least 1.
    pub fn new(sample_rate: f32) -> Self {
        let mut l = Self {
            sr: sample_rate.max(1.0),
            ceiling: 1.0,
            release: 0.0,
            env: 0.0,
            reduction_db: 0.0,
        };
        l.set_params(&[-0.3, 80.0]);
        l
    }
}

impl Effect for Limiter {
    fn set_params(&mut self, p: &[f32]) {
        self.ceiling = db_to_gain(get(p, 0, -0.3).clamp(-60.0, 0.0));
        self.release = coeff(get(p, 1, 80.0).max(1.0) / 1000.0, self.sr);
    }

    fn process(&mut self, buf: &mut [f32]) {
        self.reduction_db = 0.0;
        for f in buf.as_chunks_mut::<2>().0 {
            let peak = f[0].abs().max(f[1].abs());
            self.env = peak.max(self.env * self.release);
            if self.env > self.ceiling {
                let g = self.ceiling / self.env;
                self.reduction_db = self.reduction_db.max(-crate::gain_to_db(g));
                f[0] *= g;
                f[1] *= g;
            }
        }
    }

    /// Return the largest positive attenuation in dB from the last processed block,
    /// or zero before processing, after reset, or when no limiting occurred.
    fn gain_reduction_db(&self) -> f32 {
        self.reduction_db
    }

    /// Clear the peak envelope and gain-reduction reading, retaining parameter settings.
    fn reset(&mut self) {
        self.env = 0.0;
        self.reduction_db = 0.0;
    }
}

/// Noise gate: opens when the level rises over the threshold, stays open for the hold time
/// after it falls back, then fades to the range floor. Params: threshold dB, attack ms, hold
/// ms, release ms, range dB.
pub struct Gate {
    sr: f32,
    threshold: f32,
    attack: f32,
    hold: u32,
    release: f32,
    floor: f32,
    /// Peak detector (fast release, so the gate can tell when the sound stops).
    env: f32,
    detect_release: f32,
    held: u32,
    gain: f32,
}

impl Gate {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let mut g = Self {
            sr,
            threshold: 0.0,
            attack: 0.0,
            hold: 0,
            release: 0.0,
            floor: 0.0,
            env: 0.0,
            detect_release: coeff(0.005, sr),
            held: 0,
            gain: 0.0,
        };
        g.set_params(&[-40.0, 1.0, 30.0, 120.0, -80.0]);
        g
    }

    /// The gate's current gain (1 open, the range floor closed).
    pub fn gain(&self) -> f32 {
        self.gain
    }
}

impl Effect for Gate {
    fn set_params(&mut self, p: &[f32]) {
        self.threshold = db_to_gain(get(p, 0, -40.0).clamp(-80.0, 0.0));
        self.attack = coeff(get(p, 1, 1.0).clamp(0.1, 50.0) / 1000.0, self.sr);
        self.hold = (get(p, 2, 30.0).clamp(0.0, 500.0) / 1000.0 * self.sr) as u32;
        self.release = coeff(get(p, 3, 120.0).clamp(5.0, 2000.0) / 1000.0, self.sr);
        let range = get(p, 4, -80.0).clamp(-80.0, 0.0);
        // At the bottom of the range the gate shuts completely.
        self.floor = if range <= -80.0 { 0.0 } else { db_to_gain(range) };
    }

    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let peak = f[0].abs().max(f[1].abs());
            self.env = peak.max(self.env * self.detect_release);
            let open = if self.env >= self.threshold {
                self.held = self.hold;
                true
            } else if self.held > 0 {
                self.held -= 1;
                true
            } else {
                false
            };
            let (target, c) = if open {
                (1.0, self.attack)
            } else {
                (self.floor, self.release)
            };
            self.gain = target + (self.gain - target) * c;
            f[0] *= self.gain;
            f[1] *= self.gain;
        }
    }

    fn reset(&mut self) {
        self.env = 0.0;
        self.held = 0;
        self.gain = 0.0;
    }
}

/// Saturation: the input is driven into a shaper (soft clip `tanh`, hard clip, or wave
/// folding), then a one-pole low-pass tames the new harmonics. Params: drive dB, shape, tone Hz,
/// mix, output dB.
pub struct Drive {
    sr: f32,
    drive: f32,
    shape: u8,
    tone: f32,
    mix: f32,
    out: f32,
    lp: [f32; 2],
}

impl Drive {
    pub fn new(sample_rate: f32) -> Self {
        let mut d = Self {
            sr: sample_rate.max(1.0),
            drive: 1.0,
            shape: 0,
            tone: 1.0,
            mix: 1.0,
            out: 1.0,
            lp: [0.0; 2],
        };
        d.set_params(&[12.0, 0.0, 8000.0, 1.0, -6.0]);
        d
    }

    fn shape(&self, x: f32) -> f32 {
        match self.shape {
            1 => x.clamp(-1.0, 1.0),
            // Folding back from ±1 (a triangle-shaped transfer curve, period 4).
            2 => {
                let t = (x + 1.0).rem_euclid(4.0);
                if t < 2.0 { t - 1.0 } else { 3.0 - t }
            }
            _ => x.tanh(),
        }
    }
}

impl Effect for Drive {
    fn set_params(&mut self, p: &[f32]) {
        self.drive = db_to_gain(get(p, 0, 12.0).clamp(0.0, 36.0));
        self.shape = get(p, 1, 0.0).round().clamp(0.0, 2.0) as u8;
        let hz = get(p, 2, 8000.0).clamp(500.0, 20_000.0).min(self.sr * 0.45);
        self.tone = 1.0 - (-2.0 * PI * hz / self.sr).exp();
        self.mix = get(p, 3, 1.0).clamp(0.0, 1.0);
        self.out = db_to_gain(get(p, 4, -6.0).clamp(-24.0, 6.0));
    }

    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            for (c, x) in f.iter_mut().enumerate() {
                let wet = self.shape(*x * self.drive);
                self.lp[c] += (wet - self.lp[c]) * self.tone;
                if !self.lp[c].is_finite() {
                    self.lp[c] = 0.0;
                }
                *x = (*x * (1.0 - self.mix) + self.lp[c] * self.mix) * self.out;
            }
        }
    }

    fn reset(&mut self) {
        self.lp = [0.0; 2];
    }
}

/// Modulation: chorus and flanger read a delay line at an LFO-swept time (chorus around 15 ms,
/// flanger around 2 ms, with feedback); the phaser sweeps six first-order all-passes between
/// 200 Hz and 4 kHz. The right side's LFO runs `spread` of a cycle ahead.
/// Params: mode, rate Hz, depth, feedback, spread, mix.
pub struct Chorus {
    sr: f32,
    mode: u8,
    rate: f32,
    depth: f32,
    feedback: f32,
    spread: f32,
    mix: f32,
    phase: f32,
    lines: [Line; 2],
    fb: [f32; 2],
    /// Phaser all-pass states, per side.
    ap: [[f32; 6]; 2],
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let len = (0.04 * sr) as usize + 4;
        let mut c = Self {
            sr,
            mode: 0,
            rate: 0.8,
            depth: 0.5,
            feedback: 0.0,
            spread: 0.5,
            mix: 0.5,
            phase: 0.0,
            lines: [Line::new(len), Line::new(len)],
            fb: [0.0; 2],
            ap: [[0.0; 6]; 2],
        };
        c.set_params(&[0.0, 0.8, 0.5, 0.0, 0.5, 0.5]);
        c
    }
}

impl Effect for Chorus {
    fn set_params(&mut self, p: &[f32]) {
        self.mode = get(p, 0, 0.0).round().clamp(0.0, 2.0) as u8;
        self.rate = get(p, 1, 0.8).clamp(0.05, 10.0);
        self.depth = get(p, 2, 0.5).clamp(0.0, 1.0);
        self.feedback = get(p, 3, 0.0).clamp(-0.95, 0.95);
        self.spread = get(p, 4, 0.5).clamp(0.0, 1.0);
        self.mix = get(p, 5, 0.5).clamp(0.0, 1.0);
    }

    fn process(&mut self, buf: &mut [f32]) {
        let inc = self.rate / self.sr;
        for f in buf.as_chunks_mut::<2>().0 {
            for (c, sample) in f.iter_mut().enumerate() {
                let lfo = 0.5 + 0.5 * (2.0 * PI * (self.phase + self.spread * 0.5 * c as f32)).sin();
                let x = *sample;
                let wet = match self.mode {
                    2 => {
                        // All-pass coefficient for a corner sweeping 200 Hz … 4 kHz.
                        let hz = 200.0 * 20f32.powf(lfo * self.depth);
                        let t = (PI * hz / self.sr).tan();
                        let a = (t - 1.0) / (t + 1.0);
                        let mut y = x + self.fb[c] * self.feedback;
                        for s in &mut self.ap[c] {
                            let out = a * y + *s;
                            *s = y - a * out;
                            y = out;
                        }
                        self.fb[c] = y;
                        y
                    }
                    m => {
                        let (base, sweep) = if m == 1 { (2.0, 1.8) } else { (15.0, 7.0) };
                        let delay = (base + sweep * self.depth * (2.0 * lfo - 1.0)) * 0.001 * self.sr;
                        let y = self.lines[c].read(delay);
                        self.lines[c].write(x + y * self.feedback);
                        y
                    }
                };
                let wet = if wet.is_finite() { wet } else { 0.0 };
                *sample = x * (1.0 - self.mix) + wet * self.mix;
            }
            self.phase = (self.phase + inc).fract();
        }
    }

    fn reset(&mut self) {
        for l in &mut self.lines {
            l.clear();
        }
        self.fb = [0.0; 2];
        self.ap = [[0.0; 6]; 2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    fn sine(freq: f32, amp: f32, frames: usize) -> Vec<f32> {
        (0..frames)
            .flat_map(|i| [amp * (2.0 * PI * freq * i as f32 / SR).sin(); 2])
            .collect()
    }

    fn rms(b: &[f32]) -> f32 {
        (b.iter().map(|x| x * x).sum::<f32>() / b.len().max(1) as f32).sqrt()
    }

    fn run(e: &mut dyn Effect, mut b: Vec<f32>) -> Vec<f32> {
        for chunk in b.chunks_mut(2 * 256) {
            e.process(chunk);
        }
        b
    }

    #[test]
    fn utility_gain_width_polarity() {
        let mut u = Utility::new();
        u.set_params(&[-6.0206, 0.0, 1.0]);
        let mut b = vec![1.0, 0.0];
        u.process(&mut b);
        assert!(
            (b[0] + 0.25).abs() < 1e-4 && (b[1] + 0.25).abs() < 1e-4,
            "{b:?}: half gain, mono, inverted"
        );
    }

    #[test]
    fn track_eq_bands_shape_the_response() {
        let mut e = Eq3::new(SR);
        let flat = [(90.0, 0.0, 0.27), (1500.0, 0.0, 0.27), (8000.0, 0.0, 0.27)];
        e.set_bands(&flat);
        assert_eq!(e.response_db(1000.0), 0.0, "all bands at 0 dB are bypassed");
        e.set_bands(&[(90.0, 12.0, 0.27), (1500.0, -6.0, 0.27), (8000.0, 0.0, 0.27)]);
        assert!((e.response_db(20.0) - 12.0).abs() < 0.5, "{}", e.response_db(20.0));
        assert!((e.response_db(1500.0) + 6.0).abs() < 0.3, "{}", e.response_db(1500.0));
        assert!(e.response_db(15_000.0).abs() < 0.3);
        // The response matches what the filter does to a sine.
        let out = run(&mut e, sine(1500.0, 0.1, 48_000));
        let gain = rms(&out[48_000..]) / rms(&sine(1500.0, 0.1, 24_000));
        assert!((crate::gain_to_db(gain) + 6.0).abs() < 0.3, "{gain}");
        // A wider peak reaches further.
        let narrow = {
            e.set_bands(&[(90.0, 0.0, 0.0), (1500.0, 12.0, 0.1), (8000.0, 0.0, 0.0)]);
            e.response_db(3000.0)
        };
        e.set_bands(&[(90.0, 0.0, 0.0), (1500.0, 12.0, 0.9), (8000.0, 0.0, 0.0)]);
        assert!(e.response_db(3000.0) > narrow + 3.0);
    }

    #[test]
    fn eq_boosts_and_cuts_its_band() {
        let mut e = Eq3::new(SR);
        e.set_params(&[0.0, 120.0, 12.0, 1000.0, 1.0, 0.0, 6000.0]);
        let out = run(&mut e, sine(1000.0, 0.1, 48_000));
        let gain = rms(&out[48_000..]) / rms(&sine(1000.0, 0.1, 24_000));
        assert!((gain - 3.98).abs() < 0.2, "+12 dB at the centre: {gain}");
        let mut e = Eq3::new(SR);
        e.set_params(&[0.0, 120.0, 12.0, 1000.0, 4.0, 0.0, 6000.0]);
        let out = run(&mut e, sine(100.0, 0.1, 48_000));
        let gain = rms(&out[48_000..]) / rms(&sine(100.0, 0.1, 24_000));
        assert!(
            (gain - 1.0).abs() < 0.05,
            "far from a narrow band nothing changes: {gain}"
        );
    }

    #[test]
    fn filter_modes() {
        let level = |mode: f32, freq: f32| {
            let mut f = Filter::new(SR);
            f.set_params(&[mode, 1000.0, 0.0, 1.0]);
            let out = run(&mut f, sine(freq, 0.5, 48_000));
            rms(&out[48_000..]) / rms(&sine(freq, 0.5, 24_000))
        };
        assert!(level(0.0, 100.0) > 0.95 && level(0.0, 8000.0) < 0.05, "low-pass");
        assert!(level(2.0, 100.0) < 0.05 && level(2.0, 8000.0) > 0.95, "high-pass");
        assert!(
            level(1.0, 1000.0) > level(1.0, 100.0) * 5.0,
            "band-pass peaks at the cutoff"
        );
    }

    #[test]
    fn delay_echoes_after_the_delay_time() {
        let mut d = Delay::new(SR);
        d.set_params(&[100.0, 0.0, 0.0, 18000.0, 1.0]);
        let mut b = vec![0.0f32; 2 * 9600];
        b[0] = 1.0;
        b[1] = 1.0;
        let out = run(&mut d, b);
        let first = out
            .as_chunks::<2>()
            .0
            .iter()
            .position(|f| f[0].abs() > 0.1)
            .unwrap_or(0);
        assert!((first as i64 - 4800).abs() <= 2, "echo at 100 ms: {first}");
    }

    #[test]
    fn reverb_has_a_decaying_tail() {
        let mut r = Reverb::new(SR);
        r.set_params(&[0.7, 0.3, 0.0, 1.0, 1.0]);
        let mut b = vec![0.0f32; 2 * 96_000];
        b[0] = 1.0;
        b[1] = 1.0;
        let out = run(&mut r, b);
        let early = rms(&out[2 * 2_400..2 * 12_000]);
        let late = rms(&out[2 * 72_000..]);
        assert!(early > 1e-3, "audible tail {early}");
        assert!(late < early * 0.2, "decays {late} vs {early}");
        assert!(out.iter().all(|x| x.is_finite() && x.abs() < 1.0));
    }

    #[test]
    fn compressor_reduces_loud_signals_by_the_ratio() {
        let mut c = Compressor::new(SR);
        c.set_params(&[-20.0, 4.0, 1.0, 100.0, 0.0]);
        // A 0 dBFS-peak sine is 20 dB over: expect about 15 dB reduction at 4:1.
        let out = run(&mut c, sine(200.0, 1.0, 48_000));
        let gain = crate::gain_to_db(rms(&out[48_000..]) / rms(&sine(200.0, 1.0, 24_000)));
        assert!((-16.5..=-13.5).contains(&gain), "{gain} dB");
        let quiet = run(&mut c, sine(200.0, 0.05, 24_000));
        let g = rms(&quiet[24_000..]) / rms(&sine(200.0, 0.05, 12_000));
        assert!(g > 0.8, "below the threshold the signal recovers: {g}");
    }

    #[test]
    fn limiter_never_exceeds_the_ceiling() {
        let mut l = Limiter::new(SR);
        l.set_params(&[-6.0, 50.0]);
        let ceiling = db_to_gain(-6.0);
        let out = run(&mut l, sine(300.0, 2.0, 24_000));
        assert!(out.iter().all(|x| x.abs() <= ceiling + 1e-6));
    }

    #[test]
    fn gate_passes_loud_sound_and_closes_after_hold_and_release() {
        let mut g = Gate::new(SR);
        // -20 dB threshold, 1 ms attack, 10 ms hold, 20 ms release, closed fully.
        g.set_params(&[-20.0, 1.0, 10.0, 20.0, -80.0]);
        let loud = run(&mut g, sine(200.0, 0.5, 9_600));
        assert!(
            rms(&loud[2 * 4_800..]) / rms(&sine(200.0, 0.5, 4_800)) > 0.99,
            "open on loud sound"
        );
        let quiet = run(&mut g, sine(200.0, 0.01, 9_600));
        // Hold (480 frames) then release: shut well before the end.
        assert!(rms(&quiet[2 * 7_200..]) < 1e-5, "closed on quiet sound");
        assert!(g.gain() < 1e-3);
        // A partial range leaves the floor through.
        g.set_params(&[-20.0, 1.0, 0.0, 5.0, -20.0]);
        let quiet = run(&mut g, sine(200.0, 0.01, 9_600));
        let ratio = rms(&quiet[2 * 7_200..]) / rms(&sine(200.0, 0.01, 2_400));
        assert!((ratio - 0.1).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn drive_saturates_hard_clips_and_folds() {
        let mut d = Drive::new(SR);
        // 24 dB of drive into a hard clip with the tone open, no output change: the output is
        // a square-ish wave at ±1.
        d.set_params(&[24.0, 1.0, 20_000.0, 1.0, 0.0]);
        let out = run(&mut d, sine(100.0, 0.5, 9_600));
        let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak <= 1.0 + 1e-3 && peak > 0.95, "{peak}");
        assert!(rms(&out[2 * 4_800..]) > 0.85, "mostly at the rails");
        // Soft clip never passes ±1 either, and is smoother (lower RMS than the hard clip).
        d.set_params(&[24.0, 0.0, 20_000.0, 1.0, 0.0]);
        d.reset();
        let soft = run(&mut d, sine(100.0, 0.5, 9_600));
        assert!(soft.iter().all(|v| v.abs() <= 1.0 + 1e-3));
        // Folding keeps the level bounded too.
        d.set_params(&[36.0, 2.0, 20_000.0, 1.0, 0.0]);
        let fold = run(&mut d, sine(100.0, 0.5, 9_600));
        assert!(fold.iter().all(|v| v.abs() <= 1.0 + 1e-3));
        // Mix 0 is the dry signal times the output gain.
        d.set_params(&[36.0, 0.0, 8000.0, 0.0, -6.0206]);
        let dry = run(&mut d, sine(100.0, 0.5, 480));
        let src = sine(100.0, 0.5, 480);
        assert!(dry.iter().zip(&src).all(|(a, b)| (a - b * 0.5).abs() < 1e-4));
    }

    #[test]
    fn chorus_flanger_and_phaser_modulate_and_stay_bounded() {
        let dry = sine(1000.0, 0.5, 24_000);
        for mode in [0.0, 1.0, 2.0] {
            let mut c = Chorus::new(SR);
            c.set_params(&[mode, 2.0, 1.0, 0.5, 0.5, 0.5]);
            let out = run(&mut c, dry.clone());
            assert!(out.iter().all(|v| v.is_finite() && v.abs() < 2.0), "mode {mode}");
            // The wet signal differs from the dry one (it is modulated) …
            let diff = out.iter().zip(&dry).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
            assert!(diff > 0.05, "mode {mode}: {diff}");
            // … and the two sides differ with spread.
            let lr = out
                .as_chunks::<2>()
                .0
                .iter()
                .map(|f| (f[0] - f[1]).abs())
                .fold(0.0f32, f32::max);
            assert!(lr > 0.01, "mode {mode}: sides {lr}");
        }
        // Mix 0 passes the dry signal untouched.
        let mut c = Chorus::new(SR);
        c.set_params(&[0.0, 1.0, 1.0, 0.0, 0.5, 0.0]);
        assert_eq!(run(&mut c, dry.clone()), dry);
    }

    #[test]
    fn nonsense_params_stay_finite() {
        let effects: Vec<Box<dyn Effect>> = vec![
            Box::new(Utility::new()),
            Box::new(Eq3::new(SR)),
            Box::new(Filter::new(SR)),
            Box::new(Delay::new(SR)),
            Box::new(Reverb::new(SR)),
            Box::new(Compressor::new(SR)),
            Box::new(Limiter::new(SR)),
            Box::new(Gate::new(SR)),
            Box::new(Drive::new(SR)),
            Box::new(Chorus::new(SR)),
        ];
        for mut e in effects {
            e.set_params(&[f32::NAN, f32::INFINITY, -1e9, 1e9, f32::NAN, 0.0, 1e30]);
            let out = run(e.as_mut(), sine(440.0, 0.5, 4800));
            assert!(out.iter().all(|x| x.is_finite()));
            e.reset();
        }
    }
}
