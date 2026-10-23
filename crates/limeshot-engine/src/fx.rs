//! Insert effect chains on the audio thread.
//!
//! Processors are created off the audio thread by the plugin host and handed to the renderer one by one
//! ([`FxRack::adopt`]). When a snapshot arrives, [`FxRack::arrange`] moves processors between the
//! per-track chains by effect id and pushes their parameters. All vectors are preallocated, so
//! arranging never allocates. Processors no longer referenced are parked in `retired` for the
//! control thread to free ([`FxRack::take_retired`]).

use limeshot_core::{MAX_EFFECTS, Mixer};
use limeshot_dsp::effects::Effect;
use limeshot_sequencer::Snapshot;

/// Parameters kept per processor (more than any effect has), so automation can change one.
const MAX_PARAMS: usize = 16;

struct Proc {
    id: u32,
    fx: Box<dyn Effect>,
    enabled: bool,
    params: [f32; MAX_PARAMS],
    n_params: usize,
}

pub struct FxRack {
    chains: Vec<Vec<Proc>>,
    pool: Vec<Proc>,
    retired: Vec<Box<dyn Effect>>,
}

const POOL: usize = Mixer::MAX_TRACKS * (MAX_EFFECTS + 1) + 64;

impl Default for FxRack {
    fn default() -> Self {
        Self::new()
    }
}

impl FxRack {
    pub fn new() -> Self {
        Self {
            chains: (0..Mixer::MAX_TRACKS)
                .map(|_| Vec::with_capacity(MAX_EFFECTS + 1))
                .collect(),
            pool: Vec::with_capacity(POOL),
            retired: Vec::with_capacity(POOL),
        }
    }

    /// Take ownership of a processor for effect `id`; it joins a chain at the next `arrange`.
    pub fn adopt(&mut self, id: u32, fx: Box<dyn Effect>) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(Proc {
                id,
                fx,
                enabled: true,
                params: [0.0; MAX_PARAMS],
                n_params: 0,
            });
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
                    p.n_params = e.params.len().min(MAX_PARAMS);
                    p.params[..p.n_params].copy_from_slice(&e.params[..p.n_params]);
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

    /// Run track `t`'s chain over its bus. `sidechain(s)` gives mixer track `s`'s output for
    /// effects keyed by it (`None` when it does not route to `t`). Returns the largest gain
    /// reduction in dB of the chain's dynamics effects.
    pub fn process<'a>(&mut self, t: usize, buf: &mut [f32], sidechain: &dyn Fn(usize) -> Option<&'a [f32]>) -> f32 {
        let mut reduction = 0.0f32;
        if let Some(chain) = self.chains.get_mut(t) {
            for p in chain.iter_mut().filter(|p| p.enabled) {
                match p.fx.sidechain_source() {
                    0 => p.fx.process(buf),
                    s => match sidechain(s) {
                        Some(key) => p.fx.process_sidechain(buf, key),
                        None => p.fx.process(buf),
                    },
                }
                reduction = reduction.max(p.fx.gain_reduction_db());
            }
        }
        reduction
    }

    /// Set one parameter of effect `effect` on track `t` (automation). Does nothing when the
    /// value is unchanged or the effect has no processor yet.
    pub fn set_param(&mut self, t: usize, effect: u32, index: usize, value: f32) {
        let Some(p) = self
            .chains
            .get_mut(t)
            .and_then(|c| c.iter_mut().find(|p| p.id == effect))
        else {
            return;
        };
        if index < p.n_params && p.params[index] != value {
            p.params[index] = value;
            p.fx.set_params(&p.params[..p.n_params]);
        }
    }

    /// Set parameter `id` of the plugin in effect slot `effect` on track `t` at frame `offset` of
    /// the next block (automation).
    pub fn set_plugin_param(&mut self, t: usize, effect: u32, id: u32, offset: u32, value: f32) {
        if let Some(p) = self
            .chains
            .get_mut(t)
            .and_then(|c| c.iter_mut().find(|p| p.id == effect))
        {
            p.fx.set_plugin_param(id, offset, f64::from(value));
        }
    }

    /// The summed latency of track `t`'s enabled effects, in frames.
    pub fn track_latency(&self, t: usize) -> u32 {
        self.chains.get(t).map_or(0, |c| {
            c.iter()
                .filter(|p| p.enabled)
                .fold(0u32, |sum, p| sum.saturating_add(p.fx.latency()))
        })
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
