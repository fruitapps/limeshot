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
    mix: f32,
    params: [f32; MAX_PARAMS],
    n_params: usize,
}

pub struct FxRack {
    chains: Vec<Vec<Proc>>,
    pool: Vec<Proc>,
    retired: Vec<Box<dyn Effect>>,
    dry: [f32; crate::MAX_BLOCK * 2],
    dry_lines: Vec<(Option<u32>, crate::pdc::Ring)>,
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
            dry: [0.0; crate::MAX_BLOCK * 2],
            dry_lines: (0..64).map(|_| (None, crate::pdc::Ring::new())).collect(),
        }
    }

    /// Take ownership of a processor for effect `id`; it joins a chain at the next `arrange`.
    pub fn adopt(&mut self, id: u32, fx: Box<dyn Effect>) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(Proc {
                id,
                fx,
                enabled: true,
                mix: 1.0,
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
                    p.mix = e.mix;
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

    /// Publish the current block's timeline before processing insert chains.
    pub fn set_transport(&mut self, transport: &limeshot_dsp::Transport) {
        for p in self.chains.iter_mut().flatten() {
            p.fx.set_transport(transport);
        }
    }

    /// Run track `t`'s chain over its bus. `sidechain(s)` gives mixer track `s`'s output for
    /// effects keyed by it (`None` when it does not route to `t`). Returns the largest gain
    /// reduction in dB of the chain's dynamics effects.
    pub fn process<'a>(&mut self, t: usize, buf: &mut [f32], sidechain: &dyn Fn(usize) -> Option<&'a [f32]>) -> f32 {
        // Keep dry history even at 100% wet, so changing Mix never exposes stale audio.
        for (owner, line) in &mut self.dry_lines {
            if owner.is_some_and(|id| {
                !self
                    .chains
                    .iter()
                    .flatten()
                    .any(|p| p.id == id && p.enabled && p.fx.latency() > 0)
            }) {
                *owner = None;
                line.clear();
            }
        }
        let mut reduction = 0.0f32;
        if let Some(chain) = self.chains.get_mut(t) {
            for p in chain.iter_mut().filter(|p| p.enabled) {
                let dry = &mut self.dry[..buf.len()];
                let latency = p.fx.latency() as usize;
                let line = if latency > 0 {
                    self.dry_lines
                        .iter()
                        .position(|(id, _)| *id == Some(p.id))
                        .or_else(|| self.dry_lines.iter().position(|(id, _)| id.is_none()))
                } else {
                    None
                };
                if let Some(index) = line {
                    let (owner, history) = &mut self.dry_lines[index];
                    *owner = Some(p.id);
                    history.write(buf);
                    if p.mix < 1.0 {
                        history.read(latency, dry);
                    }
                } else if p.mix < 1.0 {
                    dry.copy_from_slice(buf);
                }
                match p.fx.sidechain_source() {
                    0 => p.fx.process(buf),
                    s => match sidechain(s) {
                        Some(key) => p.fx.process_sidechain(buf, key),
                        None => p.fx.process(buf),
                    },
                }
                if p.mix < 1.0 {
                    for (wet, dry) in buf.iter_mut().zip(dry.iter()) {
                        *wet = *dry + (*wet - *dry) * p.mix;
                    }
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
        for (_, line) in &mut self.dry_lines {
            line.clear();
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use limeshot_core::{Command, Document, EffectKind, PatternId};
    struct DelayedGain {
        history: [f32; 6],
        at: usize,
    }
    impl Effect for DelayedGain {
        fn set_params(&mut self, _: &[f32]) {}
        fn process(&mut self, buf: &mut [f32]) {
            for v in buf {
                let input = *v;
                *v = self.history[self.at] * 0.25;
                self.history[self.at] = input;
                self.at = (self.at + 1) % 6;
            }
        }
        fn reset(&mut self) {
            self.history.fill(0.0);
            self.at = 0;
        }
        fn latency(&self) -> u32 {
            3
        }
    }
    #[test]
    fn insert_mix_aligns_dry_audio_and_bypass_at_every_block_size() {
        for block in [1, 64, 333, 1024] {
            for mix in [0.0, 0.5, 1.0] {
                let mut doc = Document::default();
                let id = doc
                    .execute(Command::AddEffect {
                        track: 1,
                        kind: EffectKind::Utility,
                    })
                    .unwrap()
                    .0
                    .unwrap();
                doc.execute(Command::SetEffectMix {
                    track: 1,
                    effect: id,
                    mix,
                })
                .unwrap();
                let snap = limeshot_sequencer::compile(doc.project(), PatternId(0), &limeshot_sequencer::NoSamples);
                let mut rack = FxRack::new();
                rack.adopt(
                    id,
                    Box::new(DelayedGain {
                        history: [0.0; 6],
                        at: 0,
                    }),
                );
                rack.arrange(&snap);
                let mut result = Vec::new();
                for start in (0..4096).step_by(block) {
                    let n = block.min(4096 - start);
                    let mut buf: Vec<f32> = (start..start + n)
                        .flat_map(|i| [if i == 0 { 1.0 } else { 0.0 }; 2])
                        .collect();
                    rack.process(1, &mut buf, &|_| None);
                    result.extend(buf);
                }
                assert_eq!(result[6], 1.0 - mix * 0.75);
                assert_eq!(result.iter().filter(|v| **v != 0.0).count(), 2);
                doc.execute(Command::SetEffectEnabled {
                    track: 1,
                    effect: id,
                    enabled: false,
                })
                .unwrap();
                rack.arrange(&limeshot_sequencer::compile(
                    doc.project(),
                    PatternId(0),
                    &limeshot_sequencer::NoSamples,
                ));
                let mut dry = [1.0; 8];
                rack.process(1, &mut dry, &|_| None);
                assert_eq!(dry, [1.0; 8]);
            }
        }
    }
}
