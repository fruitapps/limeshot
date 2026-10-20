//! Insert effect chains on the audio thread.
//!
//! Processors are created off the audio thread ([`build`]) and handed to the renderer one by one
//! ([`FxRack::adopt`]). When a snapshot arrives, [`FxRack::arrange`] moves processors between the
//! per-track chains by effect id and pushes their parameters. All vectors are preallocated, so
//! arranging never allocates. Processors no longer referenced are parked in `retired` for the
//! control thread to free ([`FxRack::take_retired`]).

use limeshot_core::{EffectKind, MAX_EFFECTS, Mixer};
use limeshot_dsp::effects::{Compressor, Delay, Effect, Eq3, Filter, Limiter, Reverb, Utility};
use limeshot_sequencer::Snapshot;

/// Create a processor for `kind` at `sample_rate` with `params` applied. Allocates: call it on
/// the control thread.
pub fn build(kind: EffectKind, sample_rate: u32, params: &[f32]) -> Box<dyn Effect> {
    let sr = sample_rate as f32;
    let mut e: Box<dyn Effect> = match kind {
        EffectKind::Utility => Box::new(Utility::new()),
        EffectKind::Eq3 => Box::new(Eq3::new(sr)),
        EffectKind::Filter => Box::new(Filter::new(sr)),
        EffectKind::Delay => Box::new(Delay::new(sr)),
        EffectKind::Reverb => Box::new(Reverb::new(sr)),
        EffectKind::Compressor => Box::new(Compressor::new(sr)),
        EffectKind::Limiter => Box::new(Limiter::new(sr)),
    };
    e.set_params(params);
    e
}

/// Processors for every effect in `snapshot` (for offline rendering).
pub fn build_all(snapshot: &Snapshot, sample_rate: u32) -> Vec<(u32, Box<dyn Effect>)> {
    snapshot
        .mixer
        .iter()
        .flat_map(|t| t.effects.iter())
        .map(|e| (e.id, build(e.kind, sample_rate, &e.params)))
        .collect()
}

struct Proc {
    id: u32,
    fx: Box<dyn Effect>,
    enabled: bool,
}

pub struct FxRack {
    chains: Vec<Vec<Proc>>,
    pool: Vec<Proc>,
    retired: Vec<Box<dyn Effect>>,
}

const POOL: usize = Mixer::MAX_TRACKS * MAX_EFFECTS + 64;

impl Default for FxRack {
    fn default() -> Self {
        Self::new()
    }
}

impl FxRack {
    pub fn new() -> Self {
        Self {
            chains: (0..Mixer::MAX_TRACKS)
                .map(|_| Vec::with_capacity(MAX_EFFECTS))
                .collect(),
            pool: Vec::with_capacity(POOL),
            retired: Vec::with_capacity(POOL),
        }
    }

    /// Take ownership of a processor for effect `id`; it joins a chain at the next `arrange`.
    pub fn adopt(&mut self, id: u32, fx: Box<dyn Effect>) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(Proc { id, fx, enabled: true });
        } else {
            self.retire(fx);
        }
    }

    fn retire(&mut self, fx: Box<dyn Effect>) {
        if self.retired.len() < self.retired.capacity() {
            self.retired.push(fx);
        }
        // Otherwise `fx` drops here; only reachable if the control thread stops collecting.
    }

    /// Lay out the chains as `snapshot` describes. Effects without a processor are skipped
    /// (bypassed) until one is adopted.
    pub fn arrange(&mut self, snapshot: &Snapshot) {
        for chain in &mut self.chains {
            while let Some(p) = chain.pop() {
                if self.pool.len() < self.pool.capacity() {
                    self.pool.push(p);
                } else {
                    self.retired.push(p.fx);
                }
            }
        }
        for (t, track) in snapshot.mixer.iter().enumerate().take(self.chains.len()) {
            for e in &track.effects {
                if let Some(i) = self.pool.iter().position(|p| p.id == e.id) {
                    let mut p = self.pool.swap_remove(i);
                    p.fx.set_params(&e.params);
                    p.enabled = e.enabled;
                    if let Some(chain) = self.chains.get_mut(t)
                        && chain.len() < chain.capacity()
                    {
                        chain.push(p);
                    } else {
                        self.retire(p.fx);
                    }
                }
            }
        }
        while let Some(p) = self.pool.pop() {
            self.retire(p.fx);
        }
    }

    /// Retire every processor (the project was replaced; effect ids may be reused).
    pub fn clear(&mut self) {
        for c in 0..self.chains.len() {
            while let Some(p) = self.chains[c].pop() {
                self.retire(p.fx);
            }
        }
        while let Some(p) = self.pool.pop() {
            self.retire(p.fx);
        }
    }

    /// Run track `t`'s chain over its bus.
    pub fn process(&mut self, t: usize, buf: &mut [f32]) {
        if let Some(chain) = self.chains.get_mut(t) {
            for p in chain.iter_mut().filter(|p| p.enabled) {
                p.fx.process(buf);
            }
        }
    }

    pub fn has_effects(&self) -> bool {
        self.chains.iter().any(|c| !c.is_empty())
    }

    /// Clear delay lines and envelopes (after stopping, so tails do not resume).
    pub fn reset(&mut self) {
        for p in self.chains.iter_mut().flatten() {
            p.fx.reset();
        }
    }

    /// A processor that is no longer used, for the control thread to drop.
    pub fn take_retired(&mut self) -> Option<Box<dyn Effect>> {
        self.retired.pop()
    }

    /// Put back a processor that could not be handed over.
    pub fn keep_retired(&mut self, fx: Box<dyn Effect>) {
        self.retire(fx);
    }
}
