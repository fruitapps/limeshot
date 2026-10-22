//! Insert effects as project data: which effects sit on a mixer track, in what order, with what
//! parameter values. The DSP lives in `limeshot-dsp::effects`; this module only describes the
//! effects (kinds, parameter specs) so the UI, validation and the engine agree.

use serde::{Deserialize, Serialize};

/// The built-in effects. All are original implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectKind {
    /// Gain, stereo width and polarity.
    Utility,
    /// Low shelf, peaking mid and high shelf.
    Eq3,
    /// Resonant state-variable filter (low-, band- or high-pass).
    Filter,
    /// Stereo feedback delay.
    Delay,
    /// Algorithmic room reverb.
    Reverb,
    /// Feed-forward compressor.
    Compressor,
    /// Brick-wall peak limiter.
    Limiter,
    /// Noise gate: quiets the signal below a threshold.
    Gate,
    /// A hosted plugin ([`EffectSlot::plugin`]); its parameters are the plugin's own.
    Plugin,
}

/// One parameter of an effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: &'static str,
    /// Display/edit on a logarithmic scale (frequencies, times).
    pub log: bool,
    /// Integer choices (e.g. filter mode); `labels` names them.
    pub labels: &'static [&'static str],
}

const fn p(name: &'static str, min: f32, max: f32, default: f32, unit: &'static str, log: bool) -> ParamSpec {
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

const UTILITY: &[ParamSpec] = &[
    p("Gain", -24.0, 24.0, 0.0, "dB", false),
    p("Width", 0.0, 2.0, 1.0, "×", false),
    ParamSpec {
        name: "Polarity",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        unit: "",
        log: false,
        labels: &["Normal", "Inverted"],
    },
];
const EQ3: &[ParamSpec] = &[
    p("Low gain", -18.0, 18.0, 0.0, "dB", false),
    p("Low freq", 30.0, 600.0, 120.0, "Hz", true),
    p("Mid gain", -18.0, 18.0, 0.0, "dB", false),
    p("Mid freq", 200.0, 8000.0, 1000.0, "Hz", true),
    p("Mid Q", 0.2, 8.0, 0.8, "", true),
    p("High gain", -18.0, 18.0, 0.0, "dB", false),
    p("High freq", 1500.0, 16000.0, 6000.0, "Hz", true),
];
const FILTER: &[ParamSpec] = &[
    ParamSpec {
        name: "Mode",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        unit: "",
        log: false,
        labels: &["Low-pass", "Band-pass", "High-pass"],
    },
    p("Cutoff", 20.0, 20000.0, 2000.0, "Hz", true),
    p("Resonance", 0.0, 0.95, 0.2, "", false),
    p("Mix", 0.0, 1.0, 1.0, "", false),
];
const DELAY: &[ParamSpec] = &[
    p("Time", 10.0, 2000.0, 375.0, "ms", true),
    p("Feedback", 0.0, 0.95, 0.4, "", false),
    p("Spread", 0.0, 1.0, 0.5, "", false),
    p("Tone", 500.0, 18000.0, 6000.0, "Hz", true),
    p("Mix", 0.0, 1.0, 0.3, "", false),
    // Tempo sync (FL Studio's Fruity Delay sets its time in steps of the tempo): when on, the
    // time follows the project tempo and the Time parameter is ignored.
    ParamSpec {
        name: "Sync",
        min: 0.0,
        max: 8.0,
        default: 0.0,
        unit: "",
        log: false,
        labels: &["Off", "1/16", "1/8T", "1/8", "1/8.", "1/4", "1/4.", "1/2", "1/1"],
    },
];

/// Index of the delay's tempo-sync parameter, and each choice's length in beats.
pub const DELAY_SYNC: usize = 5;
const DELAY_SYNC_BEATS: [f64; 9] = [0.0, 0.25, 1.0 / 3.0, 0.5, 0.75, 1.0, 1.5, 2.0, 4.0];
const REVERB: &[ParamSpec] = &[
    p("Size", 0.0, 1.0, 0.6, "", false),
    p("Damping", 0.0, 1.0, 0.4, "", false),
    p("Pre-delay", 0.0, 120.0, 10.0, "ms", false),
    p("Width", 0.0, 1.0, 1.0, "", false),
    p("Mix", 0.0, 1.0, 0.25, "", false),
];
const COMPRESSOR: &[ParamSpec] = &[
    p("Threshold", -60.0, 0.0, -18.0, "dB", false),
    p("Ratio", 1.0, 20.0, 4.0, ":1", true),
    p("Attack", 0.1, 200.0, 10.0, "ms", true),
    p("Release", 10.0, 2000.0, 120.0, "ms", true),
    p("Makeup", 0.0, 24.0, 0.0, "dB", false),
    // The mixer track keying the detector (0: own input); it must route to this track. The
    // Mixer shows it as a choice of those tracks (FL Studio's sidechain input).
    ParamSpec {
        name: "Sidechain",
        min: 0.0,
        max: 127.0,
        default: 0.0,
        unit: "",
        log: false,
        labels: &[],
    },
];

/// Index of the compressor's sidechain parameter.
pub const COMPRESSOR_SIDECHAIN: usize = 5;
const GATE: &[ParamSpec] = &[
    p("Threshold", -80.0, 0.0, -40.0, "dB", false),
    p("Attack", 0.1, 50.0, 1.0, "ms", true),
    p("Hold", 0.0, 500.0, 30.0, "ms", false),
    p("Release", 5.0, 2000.0, 120.0, "ms", true),
    // How far the closed gate turns the signal down.
    p("Range", -80.0, 0.0, -80.0, "dB", false),
];
const LIMITER: &[ParamSpec] = &[
    p("Ceiling", -24.0, 0.0, -0.3, "dB", false),
    p("Release", 5.0, 1000.0, 80.0, "ms", true),
];

impl EffectKind {
    pub const ALL: [EffectKind; 8] = [
        EffectKind::Utility,
        EffectKind::Eq3,
        EffectKind::Filter,
        EffectKind::Delay,
        EffectKind::Reverb,
        EffectKind::Compressor,
        EffectKind::Limiter,
        EffectKind::Gate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            EffectKind::Utility => "Utility",
            EffectKind::Eq3 => "Lime EQ",
            EffectKind::Filter => "Lime Filter",
            EffectKind::Delay => "Lime Delay",
            EffectKind::Reverb => "Lime Room",
            EffectKind::Compressor => "Lime Comp",
            EffectKind::Limiter => "Lime Limiter",
            EffectKind::Gate => "Lime Gate",
            EffectKind::Plugin => "Plugin",
        }
    }

    pub fn params(self) -> &'static [ParamSpec] {
        match self {
            EffectKind::Utility => UTILITY,
            EffectKind::Eq3 => EQ3,
            EffectKind::Filter => FILTER,
            EffectKind::Delay => DELAY,
            EffectKind::Reverb => REVERB,
            EffectKind::Compressor => COMPRESSOR,
            EffectKind::Limiter => LIMITER,
            EffectKind::Gate => GATE,
            EffectKind::Plugin => &[],
        }
    }

    pub fn defaults(self) -> Vec<f32> {
        self.params().iter().map(|p| p.default).collect()
    }
}

/// An effect in a mixer track's insert chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectSlot {
    /// Project-unique id (from the project's id counter); the engine keys processors by it.
    pub id: u32,
    pub kind: EffectKind,
    #[serde(default = "enabled_default")]
    pub enabled: bool,
    /// Parameter values in the units of `kind.params()`.
    pub params: Vec<f32>,
    /// The plugin of a [`EffectKind::Plugin`] slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<crate::PluginRef>,
    #[serde(flatten)]
    pub extra: crate::Extra,
}

fn enabled_default() -> bool {
    true
}

impl EffectSlot {
    pub fn new(id: u32, kind: EffectKind) -> Self {
        Self {
            id,
            kind,
            enabled: true,
            params: kind.defaults(),
            plugin: None,
            extra: crate::Extra::new(),
        }
    }

    /// What the slot is called: the plugin's name, or the built-in effect's.
    pub fn name(&self) -> &str {
        match &self.plugin {
            Some(p) => &p.name,
            None => self.kind.label(),
        }
    }

    /// A slot running `plugin`.
    pub fn plugin(id: u32, plugin: crate::PluginRef) -> Self {
        Self {
            plugin: Some(plugin),
            ..Self::new(id, EffectKind::Plugin)
        }
    }

    /// The parameters as the processor gets them: a tempo-synced delay's time in ms at `bpm`
    /// (within the Time range).
    pub fn resolved_params(&self, bpm: f64) -> Vec<f32> {
        let mut out = self.params.clone();
        if self.kind == EffectKind::Delay {
            let choice = out.get(DELAY_SYNC).map_or(0, |v| v.round().max(0.0) as usize);
            if let (Some(&beats), Some(time)) = (DELAY_SYNC_BEATS.get(choice), out.first_mut())
                && choice > 0
                && bpm > 0.0
            {
                *time = (beats * 60_000.0 / bpm).clamp(10.0, 2000.0) as f32;
            }
        }
        out
    }

    /// Bring `params` to the right length and clamp every value to its range.
    pub fn sanitize(&mut self) {
        if self.kind != EffectKind::Plugin {
            self.plugin = None;
        }
        if let Some(p) = &mut self.plugin {
            p.sanitize();
        }
        let specs = self.kind.params();
        self.params.resize(specs.len(), 0.0);
        for (v, s) in self.params.iter_mut().zip(specs) {
            *v = if v.is_finite() {
                v.clamp(s.min, s.max)
            } else {
                s.default
            };
            if !s.labels.is_empty() {
                *v = v.round();
            }
        }
    }
}

/// Maximum effects per mixer track (FL Studio has 10 slots).
pub const MAX_EFFECTS: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_within_ranges() {
        for k in EffectKind::ALL {
            for s in k.params() {
                assert!(s.min <= s.default && s.default <= s.max, "{k:?} {}", s.name);
                assert!(
                    !s.log || s.min > 0.0,
                    "{k:?} {}: log scale needs a positive minimum",
                    s.name
                );
            }
        }
    }

    #[test]
    fn a_synced_delay_takes_its_time_from_the_tempo() {
        let mut e = EffectSlot::new(1, EffectKind::Delay);
        assert_eq!(e.resolved_params(120.0)[0], 375.0, "off: the Time parameter");
        e.params[DELAY_SYNC] = 5.0; // 1/4
        assert_eq!(e.resolved_params(120.0)[0], 500.0);
        e.params[DELAY_SYNC] = 4.0; // 1/8 dotted
        assert_eq!(e.resolved_params(100.0)[0], 450.0);
        e.params[DELAY_SYNC] = 8.0; // a whole note at 60 BPM is longer than the delay allows
        assert_eq!(e.resolved_params(60.0)[0], 2000.0);
        // Projects from before the Sync parameter load with it off.
        let mut old = EffectSlot::new(2, EffectKind::Delay);
        old.params.truncate(5);
        old.sanitize();
        assert_eq!(old.params[DELAY_SYNC], 0.0);
    }

    #[test]
    fn sanitize_repairs_params() {
        let mut e = EffectSlot::new(1, EffectKind::Filter);
        e.params = vec![7.3, f32::NAN];
        e.sanitize();
        assert_eq!(e.params, vec![2.0, 2000.0, 0.0, 0.0]);
    }
}
