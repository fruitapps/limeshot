//! Test-only reference DSP for the pre-VST3 project format. Production snapshots cannot
//! instantiate these instruments or effects. Scheduling/mixing still uses the real plugin rack.
#![allow(dead_code)]
use limeshot_core::*;
use limeshot_dsp::effects::{Chorus, Compressor, Delay, Drive, Effect, Eq3, Filter, Gate, Limiter, Reverb, Utility};
use limeshot_dsp::{
    Adsr, db_to_gain,
    osc::Shape,
    voice::{Source, SynthMods, Voice, VoiceInstrument},
};
use limeshot_engine::{ExportError, Processors, RenderReport, RenderSettings};
use limeshot_sequencer::{InstrumentState, SampleLookup};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

#[derive(Clone)]
pub struct Snapshot {
    pub inner: limeshot_sequencer::Snapshot,
    pub voices: Vec<(u32, u8, VoiceInstrument)>,
}
impl Deref for Snapshot {
    type Target = limeshot_sequencer::Snapshot;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
fn shape(w: Waveform) -> Shape {
    match w {
        Waveform::Sine => Shape::Sine,
        Waveform::Square => Shape::Square,
        Waveform::Saw => Shape::Saw,
        Waveform::Triangle => Shape::Triangle,
    }
}

pub fn compile(project: &Project, pattern: PatternId, samples: &dyn SampleLookup) -> Snapshot {
    let mut legacy = project.clone();
    for channel in &mut legacy.channels {
        if let Instrument::Plugin(plugin) = &channel.instrument {
            if let Some(sampler) = lime::sampler_params(&channel.instrument) {
                channel.instrument = Instrument::Sampler(sampler);
            } else if let Some(synth) = lime::synth_params(plugin) {
                channel.instrument = Instrument::Synth(synth);
            }
        }
    }
    let project = &legacy;
    let mut inner = limeshot_sequencer::compile(project, pattern, samples);
    let voices = project
        .channels
        .iter()
        .map(|c| {
            let voice = match &c.instrument {
                Instrument::Sampler(p) => VoiceInstrument::Sampler {
                    sample: p.sample.as_ref().and_then(|r| samples.sample(&r.path)),
                    gain: db_to_gain(p.gain_db),
                    pitch: p.pitch,
                    start: p.start.clamp(0.0, 1.0),
                    end: p.end.clamp(0.0, 1.0),
                    reverse: p.reverse,
                    attack: p.attack_ms.clamp(0.0, 10_000.0) / 1000.0,
                    decay: p.decay_ms.clamp(0.0, 10_000.0) / 1000.0,
                    sustain: p.sustain.clamp(0.0, 1.0),
                    release: p.release_ms.max(0.0) / 1000.0,
                    one_shot: p.one_shot,
                    looping: p.looping.then(|| {
                        let (a, b) = (p.loop_start.clamp(0.0, 1.0), p.loop_end.clamp(0.0, 1.0));
                        (a.min(b), a.max(b))
                    }),
                    filter: (p.cutoff_hz < 19_999.0 || p.resonance > 0.0)
                        .then(|| (p.cutoff_hz.clamp(20.0, 20_000.0), p.resonance.clamp(0.0, 0.95))),
                },
                Instrument::Synth(p) => VoiceInstrument::Synth {
                    shape: shape(p.waveform),
                    adsr: Adsr {
                        attack: p.attack_ms / 1000.0,
                        decay: p.decay_ms / 1000.0,
                        sustain: p.sustain,
                        release: p.release_ms / 1000.0,
                    },
                    gain: db_to_gain(p.gain_db),
                    octave: p.octave,
                    detune_cents: p.detune_cents,
                    mods: SynthMods {
                        cutoff: p.cutoff_hz,
                        resonance: p.resonance,
                        env_octaves: p.filter_env,
                        env_decay: p.filter_decay_ms / 1000.0,
                        lfo_rate: p.lfo_rate_hz,
                        vibrato_cents: p.vibrato_cents,
                        lfo_cutoff: p.lfo_cutoff,
                        mode: match p.filter_mode {
                            limeshot_core::FilterMode::LowPass => limeshot_dsp::filter::Mode::LowPass,
                            limeshot_core::FilterMode::BandPass => limeshot_dsp::filter::Mode::BandPass,
                            limeshot_core::FilterMode::HighPass => limeshot_dsp::filter::Mode::HighPass,
                        },
                        key_track: p.key_track.clamp(0.0, 1.0),
                    },
                },

                _ => VoiceInstrument::Silent,
            };
            (c.id.0, c.root_key, voice)
        })
        .collect();
    for (ch, source) in inner.channels.iter_mut().zip(&project.channels) {
        if matches!(source.instrument, Instrument::Synth(_) | Instrument::Sampler(_)) {
            ch.instrument = InstrumentState::Plugin { channel: source.id.0 };
        }
    }
    Snapshot { inner, voices }
}

struct Reference {
    cfg: VoiceInstrument,
    root: u8,
    rate: f32,
    slots: Vec<(i32, Voice)>,
}
impl limeshot_dsp::Instrument for Reference {
    fn process(&mut self, out: &mut [f32], events: &[limeshot_dsp::NoteEvent], _: &limeshot_dsp::Transport) {
        out.fill(0.0);
        let mut next = 0;
        for (frame, pair) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            while let Some(event) = events.get(next).filter(|e| e.offset as usize == frame) {
                match event.kind {
                    limeshot_dsp::NoteEventKind::On { key, velocity, id } => {
                        if let Some(voice) = Voice::for_instrument(&self.cfg, self.root, key, velocity, self.rate) {
                            let slot = self.slots.iter_mut().find(|(_, v)| !v.is_active());
                            if let Some(slot) = slot {
                                *slot = (id, voice);
                            }
                        }
                    }
                    limeshot_dsp::NoteEventKind::Off { id, .. } => {
                        let one_shot = matches!(
                            self.cfg,
                            VoiceInstrument::Sampler {
                                one_shot: true,
                                looping: None,
                                ..
                            }
                        );
                        if !one_shot {
                            for (voice_id, voice) in &mut self.slots {
                                if *voice_id == id {
                                    voice.release();
                                }
                            }
                        }
                    }
                }
                next += 1;
            }
            for (_, voice) in &mut self.slots {
                if voice.is_active() {
                    let (l, r) = voice.next_frame();
                    pair[0] += l;
                    pair[1] += r;
                }
            }
        }
    }
    fn reset(&mut self) {
        for (_, voice) in &mut self.slots {
            voice.source = Source::Off;
        }
    }
}

pub fn instruments(snapshot: &Snapshot, rate: u32) -> Vec<(u32, Box<dyn limeshot_dsp::Instrument>)> {
    snapshot
        .voices
        .iter()
        .map(|(id, root, cfg)| {
            (
                *id,
                Box::new(Reference {
                    cfg: cfg.clone(),
                    root: *root,
                    rate: rate as f32,
                    slots: (0..128).map(|_| (-1, Voice::default())).collect(),
                }) as Box<dyn limeshot_dsp::Instrument>,
            )
        })
        .collect()
}

pub mod fx {
    use super::*;
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
            EffectKind::Gate => Box::new(Gate::new(sr)),
            EffectKind::Drive => Box::new(Drive::new(sr)),
            EffectKind::Chorus => Box::new(Chorus::new(sr)),
            // The host adopts the real processor; until then the slot passes audio through.
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
    pub fn build_all(snapshot: &Snapshot, rate: u32) -> Vec<(u32, Box<dyn Effect>)> {
        snapshot
            .mixer
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| e.kind != EffectKind::Plugin)
            .map(|e| (e.id, build(e.kind, rate, &e.params)))
            .collect()
    }
}

pub fn processors(snapshot: &Snapshot, rate: u32) -> Processors {
    Processors {
        instruments: instruments(snapshot, rate),
        effects: fx::build_all(snapshot, rate),
    }
}
pub fn render_offline(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
) -> Result<(Vec<f32>, RenderReport), ExportError> {
    let processors = processors(&snapshot, settings.sample_rate);
    limeshot_engine::render_offline_with(Arc::new(snapshot.inner.clone()), settings, processors)
}
pub struct Renderer(limeshot_engine::Renderer);
impl Renderer {
    pub fn new(rate: u32, snapshot: Arc<Snapshot>) -> Self {
        let mut inner = limeshot_engine::Renderer::new(rate, Arc::new(snapshot.inner.clone()));
        for (id, instrument) in instruments(&snapshot, rate) {
            inner.adopt_instrument(id, instrument);
        }
        inner.rearrange();
        Self(inner)
    }
    pub fn set_snapshot(&mut self, snapshot: Arc<Snapshot>) {
        self.0.set_snapshot(Arc::new(snapshot.inner.clone()));
    }
}
impl Deref for Renderer {
    type Target = limeshot_engine::Renderer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
