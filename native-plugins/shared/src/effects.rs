use limeshot_core::EffectKind;
use limeshot_dsp::effects::{Chorus, Compressor, Delay, Drive, Effect, Eq3, Filter, Gate, Limiter, Reverb, Utility};
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
        // The host adopts the real processor; until then the slot passes audio through.
        EffectKind::Gate => Box::new(Gate::new(sr)),
        EffectKind::Drive => Box::new(Drive::new(sr)),
        EffectKind::Chorus => Box::new(Chorus::new(sr)),
        EffectKind::Plugin => Box::new(Passthrough),
    };
    e.set_params(params);
    e
}

struct Passthrough;
impl Effect for Passthrough {
    fn set_params(&mut self, _: &[f32]) {}
    fn process(&mut self, _: &mut [f32]) {}
    fn reset(&mut self) {}
}
