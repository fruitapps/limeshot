//! Linear-segment ADSR envelope.

/// Envelope settings in seconds (sustain is a level, 0–1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Adsr {
    /// A gate: instant attack, full sustain, `release` seconds of fade-out.
    pub fn gate(release: f32) -> Self {
        Self {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// A running envelope. Call [`Envelope::next`] once per sample.
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    stage: Stage,
    level: f32,
    stage_start: f32,
    stage_elapsed: u64,
    attack_step: f32,
    decay_step: f32,
    sustain: f32,
    release_secs: f32,
    release_step: f32,
    sample_rate: f32,
}

fn step(seconds: f32, span: f32, sample_rate: f32) -> f32 {
    let samples = (seconds.max(0.0) * sample_rate).max(1.0);
    span / samples
}

impl Envelope {
    /// Start an envelope at the attack stage.
    pub fn start(adsr: Adsr, sample_rate: f32) -> Self {
        let sustain = adsr.sustain.clamp(0.0, 1.0);
        let sr = sample_rate.max(1.0);
        let mut e = Self {
            stage: Stage::Attack,
            level: 0.0,
            stage_start: 0.0,
            stage_elapsed: 0,
            attack_step: step(adsr.attack, 1.0, sr),
            decay_step: step(adsr.decay, 1.0 - sustain, sr),
            sustain,
            release_secs: adsr.release,
            release_step: 0.0,
            sample_rate: sr,
        };
        if adsr.attack <= 0.0 {
            e.level = 1.0;
            e.stage_start = 1.0;
            e.stage = Stage::Decay;
        }
        e
    }

    /// Enter the release stage from wherever the envelope is.
    pub fn release(&mut self) {
        if matches!(self.stage, Stage::Release | Stage::Done) {
            return;
        }
        self.stage = Stage::Release;
        self.stage_start = self.level;
        self.stage_elapsed = 0;
        self.release_step = step(self.release_secs, self.level, self.sample_rate);
        if self.release_secs <= 0.0 {
            self.level = 0.0;
            self.stage = Stage::Done;
        }
    }

    /// Fade out quickly (voice stealing): 5 ms.
    pub fn kill(&mut self) {
        self.release_secs = 0.005;
        self.stage = Stage::Sustain;
        self.release();
    }

    pub fn is_done(&self) -> bool {
        self.stage == Stage::Done
    }

    pub fn is_released(&self) -> bool {
        matches!(self.stage, Stage::Release | Stage::Done)
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    /// Advance by output samples without rendering them. Work is bounded by the ADSR stages.
    pub fn advance(&mut self, mut samples: u64) {
        while samples > 0 {
            let (target, step, next) = match self.stage {
                Stage::Attack => (1.0, self.attack_step, Stage::Decay),
                Stage::Decay => (self.sustain, -self.decay_step, Stage::Sustain),
                Stage::Release => (0.0, -self.release_step, Stage::Done),
                Stage::Sustain => {
                    if self.sustain <= 0.0 {
                        self.stage = Stage::Done;
                    }
                    return;
                }
                Stage::Done => return,
            };
            // Measure from the stage's origin so rounding never depends on call sizes.
            let duration = if step == 0.0 {
                1
            } else {
                ((f64::from(target) - f64::from(self.stage_start)) / f64::from(step)).ceil() as u64
            }
            .max(1);
            let remaining = duration - self.stage_elapsed;
            if samples < remaining {
                self.stage_elapsed += samples;
                self.level = (f64::from(self.stage_start) + f64::from(step) * self.stage_elapsed as f64) as f32;
                return;
            }
            self.level = target;
            self.stage_start = target;
            self.stage_elapsed = 0;
            self.stage = next;
            samples -= remaining;
        }
    }

    /// The level for this sample, then advance.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        let out = self.level;
        self.advance(1);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_matches_next_with_non_exact_steps() {
        let adsr = Adsr {
            attack: 0.01,
            decay: 0.013,
            sustain: 0.37,
            release: 0.017,
        };
        for sr in [1000.0, 48_000.0] {
            // Release during attack, decay and sustain, as well as a stolen voice.
            for (release_at, kill) in [
                (None, false),
                (Some(0.005), false),
                (Some(0.015), false),
                (Some(0.03), false),
                (Some(0.015), true),
            ] {
                let mut start = Envelope::start(adsr, sr);
                if let Some(seconds) = release_at {
                    for _ in 0..(seconds * sr) as u64 {
                        start.next();
                    }
                    if kill {
                        start.kill();
                    } else {
                        start.release();
                    }
                }
                let mut rendered = start;
                let mut chunked = start;
                for samples in 0..=(0.06 * sr) as u64 {
                    let mut skipped = start;
                    skipped.advance(samples);
                    let context = format!("sr {sr}, offset {samples}, release {release_at:?}, kill {kill}");
                    assert_eq!(skipped.stage, rendered.stage, "{context}");
                    assert_eq!(skipped.level(), rendered.level(), "{context}");
                    if samples % 7 == 0 {
                        assert_eq!(chunked.stage, rendered.stage, "chunked {context}");
                        assert_eq!(chunked.level(), rendered.level(), "chunked {context}");
                        chunked.advance(7);
                    }
                    rendered.next();
                }
            }
        }
    }

    #[test]
    fn advance_crosses_stages_and_handles_large_offsets() {
        let adsr = Adsr {
            attack: 0.125,
            decay: 0.125,
            sustain: 0.5,
            release: 0.125,
        };
        for samples in [0, 1, 4, 8, 12, 16, 32] {
            let mut expected = Envelope::start(adsr, 64.0);
            let mut skipped = expected;
            for _ in 0..samples {
                expected.next();
            }
            skipped.advance(samples);
            assert_eq!(skipped.stage, expected.stage, "offset {samples}");
            assert_eq!(skipped.level(), expected.level());
            skipped.release();
            expected.release();
            skipped.advance(4);
            for _ in 0..4 {
                expected.next();
            }
            assert_eq!(skipped.stage, expected.stage);
            assert_eq!(skipped.next(), expected.next());
            skipped.advance(u64::MAX);
            assert!(skipped.is_done());
        }
        for sustain in [0.0, 1.0] {
            let mut e = Envelope::start(
                Adsr {
                    attack: 0.0,
                    decay: 0.0,
                    sustain,
                    ..adsr
                },
                64.0,
            );
            e.advance(u64::MAX);
            assert_eq!(e.level(), sustain);
            assert_eq!(e.is_done(), sustain == 0.0);
        }
    }

    #[test]
    fn stages_follow_the_settings() {
        let sr = 1000.0;
        let mut e = Envelope::start(
            Adsr {
                attack: 0.01,
                decay: 0.01,
                sustain: 0.5,
                release: 0.02,
            },
            sr,
        );
        let attack: Vec<f32> = (0..10).map(|_| e.next()).collect();
        assert_eq!(attack[0], 0.0);
        assert!((e.level() - 1.0).abs() < 1e-5, "peak after 10 ms");
        for _ in 0..10 {
            e.next();
        }
        assert!((e.level() - 0.5).abs() < 1e-5, "sustain after decay");
        for _ in 0..100 {
            assert!((e.next() - 0.5).abs() < 1e-5);
        }
        e.release();
        let mut n = 0;
        while !e.is_done() {
            e.next();
            n += 1;
            assert!(n < 1000);
        }
        assert_eq!(n, 20, "release lasts 20 ms");
        assert_eq!(e.next(), 0.0);
    }

    #[test]
    fn instant_attack_and_zero_release() {
        let mut e = Envelope::start(Adsr::gate(0.0), 48_000.0);
        assert_eq!(e.next(), 1.0);
        e.release();
        assert!(e.is_done());
        assert_eq!(e.next(), 0.0);
    }

    #[test]
    fn release_during_attack_starts_from_current_level() {
        let mut e = Envelope::start(
            Adsr {
                attack: 1.0,
                decay: 0.0,
                sustain: 1.0,
                release: 0.1,
            },
            100.0,
        );
        for _ in 0..50 {
            e.next();
        }
        let level = e.level();
        assert!((level - 0.5).abs() < 0.02);
        e.release();
        let first = e.next();
        assert!((first - level).abs() < 1e-6, "no jump at release");
        let mut n = 1;
        while !e.is_done() {
            e.next();
            n += 1;
        }
        assert!((9..=11).contains(&n), "release takes 0.1 s: {n}");
    }

    #[test]
    fn zero_sustain_ends_the_voice() {
        let mut e = Envelope::start(
            Adsr {
                attack: 0.0,
                decay: 0.01,
                sustain: 0.0,
                release: 0.1,
            },
            1000.0,
        );
        for _ in 0..20 {
            e.next();
        }
        assert!(e.is_done());
    }
}
