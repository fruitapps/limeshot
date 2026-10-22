use limeshot_core::{EffectKind, ParamSpec};
use nice_plug::prelude::*;
use std::sync::Arc;

use crate::sample::SampleStore;

pub const NAMES: [&str; 9] = [
    "LimeSynth",
    "Lime Sampler",
    "Lime Utility",
    "Lime EQ",
    "Lime Filter",
    "Lime Delay",
    "Lime Room",
    "Lime Comp",
    "Lime Limiter",
];
pub const TAGLINES: [&str; 9] = [
    "A little zest. A lot of sound.",
    "Every sound starts somewhere.",
    "Make room for your sound.",
    "Find your sweet spot.",
    "Let the good frequencies through.",
    "Good things come around.",
    "Give your sound some space.",
    "Keep the groove. Shape the punch.",
    "Big sound. Beautiful restraint.",
];
// Keep exported class/parameter order independent of new DAW-only effect kinds.
pub const KINDS: [EffectKind; 7] = [
    EffectKind::Utility,
    EffectKind::Eq3,
    EffectKind::Filter,
    EffectKind::Delay,
    EffectKind::Reverb,
    EffectKind::Compressor,
    EffectKind::Limiter,
];

/// Capacity of the plain-control arrays: the largest specification (LimeSynth) has 17 controls.
pub const MAX_CONTROLS: usize = 17;

const fn spec(name: &'static str, min: f32, max: f32, default: f32, unit: &'static str, log: bool) -> ParamSpec {
    ParamSpec {
        name,
        min,
        max,
        default,
        unit,
        log,
        labels: &[],
    }
}
const fn choice(name: &'static str, labels: &'static [&'static str], default: f32) -> ParamSpec {
    ParamSpec {
        name,
        min: 0.0,
        max: (labels.len() - 1) as f32,
        default,
        unit: "",
        log: false,
        labels,
    }
}
pub const SYNTH: &[ParamSpec] = &[
    choice("Waveform", &["Sine", "Square", "Saw", "Triangle"], 2.0),
    spec("Octave", -3.0, 3.0, 0.0, "oct", false),
    spec("Detune", -100.0, 100.0, 0.0, "ct", false),
    spec("Gain", -60.0, 12.0, -6.0, "dB", false),
    spec("Attack", 0.0, 5000.0, 5.0, "ms", false),
    spec("Decay", 1.0, 5000.0, 200.0, "ms", true),
    spec("Sustain", 0.0, 1.0, 0.7, "", false),
    spec("Release", 1.0, 10000.0, 150.0, "ms", true),
    spec("Cutoff", 20.0, 20000.0, 20000.0, "Hz", true),
    spec("Resonance", 0.0, 0.95, 0.0, "", false),
    spec("Filter envelope", 0.0, 8.0, 0.0, "oct", false),
    spec("Filter decay", 1.0, 5000.0, 300.0, "ms", true),
    spec("LFO rate", 0.1, 30.0, 5.0, "Hz", true),
    spec("Vibrato", 0.0, 200.0, 0.0, "ct", false),
    spec("LFO cutoff", 0.0, 6.0, 0.0, "oct", false),
    // Appended after the original controls so earlier parameter IDs and saved states stay valid.
    choice("Filter type", &["Low-pass", "Band-pass", "High-pass"], 0.0),
    spec("Key track", 0.0, 1.0, 0.0, "", false),
];
pub const SAMPLER: &[ParamSpec] = &[
    spec("Gain", -60.0, 12.0, 0.0, "dB", false),
    spec("Pitch", -48.0, 48.0, 0.0, "st", false),
    spec("Start", 0.0, 1.0, 0.0, "", false),
    spec("End", 0.0, 1.0, 1.0, "", false),
    spec("Release", 0.0, 5000.0, 10.0, "ms", false),
    choice("Reverse", &["Forward", "Reverse"], 0.0),
    choice("Playback", &["Gated", "One-shot"], 1.0),
    spec("Root key", 0.0, 127.0, 60.0, "MIDI", false),
];

/// Return controls in the stable native plugin order (IDs 0–8), or an empty slice for other IDs.
/// Compressor controls omit the DAW's mixer-track selector; its key comes from a host bus.
pub fn specs(id: u8) -> &'static [ParamSpec] {
    match id {
        0 => SYNTH,
        1 => SAMPLER,
        7 => &KINDS[5].params()[..5], // The external key is a bus, not a mixer track number.
        2..=8 => KINDS[(id - 2) as usize].params(),
        _ => &[],
    }
}

#[derive(Params)]
pub struct Control {
    #[id = "value"]
    pub value: FloatParam,
}

#[derive(Params)]
pub struct NativeParams {
    #[nested(array)]
    pub controls: Vec<Control>,
    #[id = "bypass"]
    pub bypass: BoolParam,
    #[id = "key"]
    pub external_key: BoolParam,
    #[persist = "sample-v1"]
    pub sample: SampleStore,
}

impl NativeParams {
    /// Create default controls for a native plugin ID, with bypass off and an empty sample.
    /// Choices, synth octave, and sampler root key are stepped; other controls use 5 ms
    /// linear smoothing. The external-key toggle is visible only for the compressor (ID 7).
    pub fn new(id: u8) -> Self {
        let controls = specs(id)
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let spec = *s;
                let range = if s.log {
                    FloatRange::Skewed {
                        min: s.min,
                        max: s.max,
                        factor: FloatRange::skew_factor(-2.0),
                    }
                } else {
                    FloatRange::Linear { min: s.min, max: s.max }
                };
                let mut value = FloatParam::new(s.name, s.default, range)
                    .with_unit(match s.unit {
                        "dB" => " dB",
                        "Hz" => " Hz",
                        "ms" => " ms",
                        "ct" => " ct",
                        "oct" => " oct",
                        "st" => " st",
                        "MIDI" => " MIDI",
                        ":1" => ":1",
                        "×" => " ×",
                        _ => "",
                    })
                    .with_value_to_string(Arc::new(move |v| display(spec, v)))
                    .with_string_to_value(Arc::new(move |text| parse(spec, text)));
                if !s.labels.is_empty() || (id == 0 && i == 1) || (id == 1 && i == 7) {
                    value = value.with_step_size(1.0);
                } else {
                    value = value.with_smoother(SmoothingStyle::Linear(5.0));
                }
                Control { value }
            })
            .collect();
        Self {
            controls,
            bypass: BoolParam::new("Bypass", false).make_bypass(),
            external_key: if id == 7 {
                BoolParam::new("External sidechain", false)
            } else {
                BoolParam::new("External sidechain", false).hide()
            },
            sample: SampleStore::default(),
        }
    }

    /// Read current plain control values in specification order without advancing smoothing.
    /// Unused entries are zero; bypass and external-key toggles are excluded.
    pub fn values(&self) -> [f32; MAX_CONTROLS] {
        let mut out = [0.0; MAX_CONTROLS];
        for (v, p) in out.iter_mut().zip(&self.controls) {
            *v = p.value.value();
        }
        out
    }

    /// Advance each control's smoother by one sample and return plain values in specification
    /// order. Unused entries are zero; bypass and external-key toggles are excluded.
    pub fn next_values(&self) -> [f32; MAX_CONTROLS] {
        let mut out = [0.0; MAX_CONTROLS];
        for (v, p) in out.iter_mut().zip(&self.controls) {
            *v = p.value.smoothed.next();
        }
        out
    }
}

/// Format a plain value without appending its unit, using choice labels when present.
/// Frequencies at or above 1,000 Hz use a `k` suffix; an unavailable label index yields empty text.
pub fn display(s: ParamSpec, value: f32) -> String {
    if !s.labels.is_empty() {
        return s.labels.get(value.round() as usize).copied().unwrap_or("").to_string();
    }
    match s.unit {
        "Hz" if value >= 1000.0 => format!("{:.2} k", value / 1000.0),
        "MIDI" | "oct" => format!("{value:.0}"),
        _ if s.max <= 2.0 => format!("{value:.2}"),
        _ => format!("{value:.1}"),
    }
}

/// Parse a case-insensitive choice label or a number with an optional unit and lowercase `k`
/// multiplier. Clamp finite numbers to the specification's bounds; invalid input returns `None`.
fn parse(s: ParamSpec, text: &str) -> Option<f32> {
    if let Some(index) = s
        .labels
        .iter()
        .position(|label| label.eq_ignore_ascii_case(text.trim()))
    {
        return Some(index as f32);
    }
    let text = text.trim().trim_end_matches(s.unit).trim();
    let value = if let Some(v) = text.strip_suffix('k') {
        v.trim().parse::<f32>().ok()? * 1000.0
    } else {
        text.parse::<f32>().ok()?
    };
    value.is_finite().then_some(value.clamp(s.min, s.max))
}
