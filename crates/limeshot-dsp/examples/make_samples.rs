//! Synthesises LimeShot's bundled drum samples into `assets/samples/` (or the given folder).
//!
//! `cargo run -p limeshot-dsp --example make_samples [out_dir]`
//!
//! Every sound is generated from oscillators and a seeded noise source, so the files are
//! original, reproducible and free of third-party rights.

use limeshot_dsp::{BitDepth, write_wav};
use std::f32::consts::TAU;
use std::path::PathBuf;

const SR: u32 = 44_100;

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        // xorshift32
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn secs(n: usize) -> f32 {
    n as f32 / SR as f32
}

fn kick() -> Vec<f32> {
    let n = (0.6 * SR as f32) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = secs(i);
            let f = 45.0 + 110.0 * (-t * 28.0).exp();
            phase += f / SR as f32;
            let body = (phase * TAU).sin() * (-t * 6.5).exp();
            let click = (-t * 400.0).exp() * 0.35 * (phase * TAU * 3.0).sin();
            (body + click).tanh() * 0.95
        })
        .collect()
}

fn snare() -> Vec<f32> {
    let n = (0.35 * SR as f32) as usize;
    let mut noise = Noise(0x5eed_0001);
    let mut lp = 0.0f32;
    (0..n)
        .map(|i| {
            let t = secs(i);
            let tone =
                (t * TAU * 185.0).sin() * (-t * 22.0).exp() * 0.6 + (t * TAU * 330.0).sin() * (-t * 30.0).exp() * 0.3;
            let w = noise.next();
            lp += 0.55 * (w - lp);
            let hiss = (w - lp) * (-t * 13.0).exp() * 0.8;
            (tone + hiss).tanh() * 0.9
        })
        .collect()
}

fn clap() -> Vec<f32> {
    let n = (0.4 * SR as f32) as usize;
    let mut noise = Noise(0x0c1a_9999);
    let mut bp1 = 0.0f32;
    let mut bp2 = 0.0f32;
    (0..n)
        .map(|i| {
            let t = secs(i);
            // Three quick bursts, then a decaying tail.
            let bursts = [0.0f32, 0.011, 0.023]
                .iter()
                .map(|s| if t >= *s { (-(t - s) * 180.0).exp() } else { 0.0 })
                .fold(0.0, f32::max);
            let env = bursts.max((-(t - 0.03).max(0.0) * 14.0).exp() * if t > 0.03 { 0.7 } else { 0.0 });
            let w = noise.next();
            // Crude band-pass around 1–2 kHz.
            bp1 += 0.25 * (w - bp1);
            bp2 += 0.05 * (bp1 - bp2);
            (bp1 - bp2) * env * 2.2
        })
        .collect()
}

fn hat(open: bool) -> Vec<f32> {
    let len = if open { 0.5 } else { 0.09 };
    let decay = if open { 7.0 } else { 55.0 };
    let n = (len * SR as f32) as usize;
    let mut noise = Noise(if open { 0x0be7_0042 } else { 0x0ac7_0042 });
    let mut prev = 0.0f32;
    let ratios = [1.0f32, 1.342, 1.2312, 1.6532, 1.9523, 2.1523];
    (0..n)
        .map(|i| {
            let t = secs(i);
            let metal: f32 = ratios
                .iter()
                .map(|r| if ((t * 4300.0 * r) % 1.0) < 0.5 { 1.0 } else { -1.0 })
                .sum::<f32>()
                / 6.0;
            let w = 0.6 * noise.next() + 0.4 * metal;
            let hp = w - prev; // first-order high-pass
            prev = w;
            hp * (-t * decay).exp() * 0.6
        })
        .collect()
}

fn perc() -> Vec<f32> {
    let n = (0.3 * SR as f32) as usize;
    (0..n)
        .map(|i| {
            let t = secs(i);
            let f = 520.0 + 300.0 * (-t * 40.0).exp();
            ((t * TAU * f).sin() * 0.8 + (t * TAU * f * 1.51).sin() * 0.3) * (-t * 16.0).exp()
        })
        .collect()
}

fn main() {
    let dir: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/samples"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        std::process::exit(1);
    }
    let sounds: [(&str, Vec<f32>); 6] = [
        ("Lime Kick.wav", kick()),
        ("Lime Snare.wav", snare()),
        ("Lime Clap.wav", clap()),
        ("Lime Hat Closed.wav", hat(false)),
        ("Lime Hat Open.wav", hat(true)),
        ("Lime Perc.wav", perc()),
    ];
    for (name, mono) in sounds {
        let stereo: Vec<f32> = mono.iter().flat_map(|&s| [s, s]).collect();
        let path = dir.join(name);
        match write_wav(&path, SR, &stereo, BitDepth::Int24) {
            Ok(()) => println!("wrote {}", path.display()),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
