use limeshot_core::{EffectKind, ParamSpec};
use nice_plug::prelude::*;
use std::sync::Arc;

use crate::sample::SampleStore;

pub const NAMES: [&str; 12] = [
    "LimeSynth",
    "Lime Sampler",
    "Lime Utility",
    "Lime EQ",
    "Lime Filter",
    "Lime Delay",
    "Lime Room",
    "Lime Comp",
    "Lime Limiter",
    "Lime Gate",
    "Lime Drive",
    "Lime Chorus",
];
pub const TAGLINES: [&str; 12] = [
    "A little zest. A lot of sound.",
    "Every sound starts somewhere.",
    "Make room for your sound.",
    "Find your sweet spot.",
    "Let the good frequencies through.",
    "Good things come around.",
    "Give your sound some space.",
    "Keep the groove. Shape the punch.",
    "Big sound. Beautiful restraint.",
    "Keep the silence clean.",
    "Find your edge.",
    "Add a little motion.",
];
// Keep exported class/parameter order independent of new DAW-only effect kinds.
pub const KINDS: [EffectKind; 10] = [
    EffectKind::Utility,
    EffectKind::Eq3,
    EffectKind::Filter,
    EffectKind::Delay,
    EffectKind::Reverb,
    EffectKind::Compressor,
    EffectKind::Limiter,
    EffectKind::Gate,
    EffectKind::Drive,
    EffectKind::Chorus,
];

/// Capacity of the plain-control arrays: the largest specification (Lime Sampler) has 18 controls.
pub const MAX_CONTROLS: usize = 18;

pub use limeshot_core::lime::{SAMPLER_CONTROLS as SAMPLER, SYNTH_CONTROLS as SYNTH, control_specs as specs};

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
                // Dedicated track EQ is edited by LimeShot's inline Mixer controls. Keep
                // its extension out of the standalone insert editor and generic host lists.
                if id == 3 && i >= 7 {
                    value = value.hide();
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
