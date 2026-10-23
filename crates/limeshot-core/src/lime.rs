//! LimeShot's own instruments and effects are the Lime Native VST3 plugins (`native-plugins/`,
//! PLG-012), hosted like any other plugin. This module names them, writes their state from
//! LimeShot's settings, and converts projects that still use the former built-in instruments and
//! effects (decision D39).
//!
//! A Lime plugin's [`PluginRef::path`] is `lime:<bundle>` rather than a file path, so projects stay
//! portable; the plugin host resolves it to the bundle shipped with LimeShot. The state is the
//! nice-plug JSON the plugins save themselves: controls are `value_1`, `value_2`, … in the order of
//! `native-plugins/shared/src/parameters.rs`, and Lime Sampler's audio is the `sample-v1` field.

use serde_json::{Map, Value, json};

use crate::automation::AutomationTarget;
use crate::document::PluginTarget;
use crate::effects::{COMPRESSOR_SIDECHAIN, EffectKind, ParamSpec};
use crate::model::{FilterMode, Instrument, Project, SamplerParams, SynthParams, Waveform};
use crate::plugin::{PluginRef, PluginState, base64_encode};

/// The prefix of a Lime plugin's path.
pub const PATH_PREFIX: &str = "lime:";
/// The vendor the Lime plugins report.
pub const VENDOR: &str = "LimeShot";

/// One of the nine Lime Native plugins, by its stable plugin number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimePlugin(pub u8);

impl LimePlugin {
    pub const SYNTH: Self = Self(0);
    pub const SAMPLER: Self = Self(1);
    pub const ALL: [Self; 9] = [
        Self(0),
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
    ];
    /// The effects exported as Lime plugins, in plugin-number order from 2.
    pub const EFFECTS: [EffectKind; 7] = [
        EffectKind::Utility,
        EffectKind::Eq3,
        EffectKind::Filter,
        EffectKind::Delay,
        EffectKind::Reverb,
        EffectKind::Compressor,
        EffectKind::Limiter,
    ];

    /// The Lime plugin that replaces a former built-in effect (Gate, Drive and Chorus have none
    /// yet).
    pub fn for_effect(kind: EffectKind) -> Option<Self> {
        Self::EFFECTS.iter().position(|k| *k == kind).map(|i| Self(i as u8 + 2))
    }

    pub fn name(self) -> &'static str {
        [
            "LimeSynth",
            "Lime Sampler",
            "Lime Utility",
            "Lime EQ",
            "Lime Filter",
            "Lime Delay",
            "Lime Room",
            "Lime Comp",
            "Lime Limiter",
        ][self.0 as usize % 9]
    }

    pub fn is_instrument(self) -> bool {
        self.0 < 2
    }

    /// The bundle's file name, as `build.py` collects it.
    pub fn bundle(self) -> String {
        format!("{}.vst3", self.name())
    }

    /// The VST3 class id: "LIMENATIVE", four zero bytes, 1, the plugin number.
    pub fn class_id(self) -> String {
        let mut id = *b"LIMENATIVE\0\0\0\0\x01\0";
        id[15] = self.0;
        id.iter().map(|b| format!("{b:02X}")).collect()
    }

    pub fn path(self) -> String {
        format!("{PATH_PREFIX}{}", self.bundle())
    }

    /// The Lime plugin a reference points at, if any.
    pub fn of(p: &PluginRef) -> Option<Self> {
        p.path.strip_prefix(PATH_PREFIX)?;
        Self::ALL
            .into_iter()
            .find(|l| l.class_id().eq_ignore_ascii_case(&p.class_id))
    }

    /// A reference with the plugin's default settings (empty state).
    pub fn plugin_ref(self) -> PluginRef {
        let mut p = PluginRef::vst3(self.path(), self.class_id(), self.name());
        p.vendor = VENDOR.into();
        p
    }

    /// A reference whose state sets the controls to `values` (plain values, in order) and the
    /// embedded sample to `sample`.
    pub fn with_values(self, values: &[f32], bypass: bool, sample: Option<&EmbeddedAudio>) -> PluginRef {
        let mut params = Map::new();
        for (i, v) in values.iter().enumerate() {
            params.insert(format!("value_{}", i + 1), json!({ "f32": v }));
        }
        params.insert("bypass".into(), json!({ "bool": bypass }));
        let mut fields = Map::new();
        if let Some(s) = sample {
            let pcm: Vec<u8> = s.stereo.iter().flat_map(|f| f.to_le_bytes()).collect();
            let embedded = json!({ "name": s.name, "sample_rate": s.sample_rate, "pcm": base64_encode(&pcm) });
            fields.insert("sample-v1".into(), Value::String(embedded.to_string()));
        }
        let component = json!({ "version": "0.1.0", "params": params, "fields": fields }).to_string();
        let mut p = self.plugin_ref();
        p.state = PluginState::new(host_state(component.as_bytes()));
        p
    }
}

/// The plugin host's state container (`limeshot_plugins::Vst3Instance::state`): "LSV3", then the
/// component state and the controller state (empty: the Lime plugins have one object), each after
/// its length as a little-endian `u32`.
fn host_state(component: &[u8]) -> Vec<u8> {
    let mut out = b"LSV3".to_vec();
    out.extend_from_slice(&(component.len() as u32).to_le_bytes());
    out.extend_from_slice(component);
    out.extend_from_slice(&0u32.to_le_bytes());
    out
}

/// The component state (the plugin's own JSON) inside a host state, if it is one.
pub fn component_state(state: &PluginState) -> Option<&[u8]> {
    let rest = state.bytes().strip_prefix(b"LSV3")?;
    let n = u32::from_le_bytes(rest.get(..4)?.try_into().ok()?) as usize;
    rest.get(4..4 + n)
}

/// The VST3 parameter id of a Lime plugin's control `index` (0-based): nice-plug hashes the
/// control's string id (`value_<n>`) with `hash = hash * 31 + byte` and clears the top bit.
pub fn param_id(index: usize) -> u32 {
    let id = format!("value_{}", index + 1);
    id.bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
        & !(1 << 31)
}

/// A plain value as the Lime plugin's normalised parameter value: linear, or nice-plug's skewed
/// range (factor 2^-2) for logarithmic controls.
pub fn plugin_normalized(spec: &ParamSpec, plain: f32) -> f32 {
    let x = (plain.clamp(spec.min, spec.max) - spec.min) / (spec.max - spec.min).max(f32::EPSILON);
    if spec.log { x.powf(0.25) } else { x }
}

/// Decoded audio for Lime Sampler's state: interleaved stereo frames.
#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddedAudio {
    pub name: String,
    pub sample_rate: u32,
    pub stereo: Vec<f32>,
}

/// LimeSynth controls for `p`, in the plugin's order.
pub fn synth_values(p: &SynthParams) -> Vec<f32> {
    vec![
        match p.waveform {
            Waveform::Sine => 0.0,
            Waveform::Square => 1.0,
            Waveform::Saw => 2.0,
            Waveform::Triangle => 3.0,
        },
        f32::from(p.octave),
        p.detune_cents,
        p.gain_db,
        p.attack_ms,
        p.decay_ms,
        p.sustain,
        p.release_ms,
        p.cutoff_hz,
        p.resonance,
        p.filter_env,
        p.filter_decay_ms,
        p.lfo_rate_hz,
        p.vibrato_cents,
        p.lfo_cutoff,
        match p.filter_mode {
            FilterMode::LowPass => 0.0,
            FilterMode::BandPass => 1.0,
            FilterMode::HighPass => 2.0,
        },
        p.key_track,
    ]
}

/// Lime Sampler controls for `p` and the channel's root key, in the plugin's order.
pub fn sampler_values(p: &SamplerParams, root_key: u8) -> Vec<f32> {
    vec![
        p.gain_db,
        p.pitch,
        p.start,
        p.end,
        p.release_ms,
        f32::from(u8::from(p.reverse)),
        f32::from(u8::from(p.one_shot)),
        f32::from(root_key),
        p.attack_ms,
        p.decay_ms,
        p.sustain,
        p.cutoff_hz,
        p.resonance,
        f32::from(u8::from(p.looping)),
        p.loop_start,
        p.loop_end,
    ]
}

/// Plain controls from a Lime plugin's component state, using defaults for older states.
pub fn plain_values(plugin: &PluginRef, defaults: &[f32]) -> Vec<f32> {
    let state = component_state(&plugin.state).and_then(|b| serde_json::from_slice::<Value>(b).ok());
    defaults
        .iter()
        .enumerate()
        .map(|(i, default)| {
            state
                .as_ref()
                .and_then(|s| s["params"][format!("value_{}", i + 1)]["f32"].as_f64())
                .filter(|v| v.is_finite())
                .map(|v| v as f32)
                .unwrap_or(*default)
        })
        .collect()
}

/// Legacy preset settings reconstructed from a LimeSynth component state.
pub fn synth_params(plugin: &PluginRef) -> Option<SynthParams> {
    if LimePlugin::of(plugin) != Some(LimePlugin::SYNTH) {
        return None;
    }
    let v = plain_values(plugin, &synth_values(&SynthParams::default()));
    Some(SynthParams {
        waveform: match v[0].round() as u8 {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            3 => Waveform::Triangle,
            _ => Waveform::Saw,
        },
        octave: v[1] as i8,
        detune_cents: v[2],
        gain_db: v[3],
        attack_ms: v[4],
        decay_ms: v[5],
        sustain: v[6],
        release_ms: v[7],
        cutoff_hz: v[8],
        resonance: v[9],
        filter_env: v[10],
        filter_decay_ms: v[11],
        lfo_rate_hz: v[12],
        vibrato_cents: v[13],
        lfo_cutoff: v[14],
        filter_mode: match v[15].round() as u8 {
            1 => FilterMode::BandPass,
            2 => FilterMode::HighPass,
            _ => FilterMode::LowPass,
        },
        key_track: v[16],
    })
}

/// Create a sampler reference without doing I/O in the project model. The document's
/// normalizer embeds its source audio before the reference reaches the host.
pub fn sampler_ref(p: &SamplerParams, root_key: u8) -> PluginRef {
    let mut plugin = LimePlugin::SAMPLER.with_values(&sampler_values(p, root_key), false, None);
    if let Some(sample) = &p.sample {
        plugin.extra.insert("lime-sample-path".into(), json!(sample.path));
        plugin.extra.insert("lime-sample-pending".into(), json!(true));
    }
    plugin
}

/// Sampler settings in a legacy channel or a Lime Sampler's saved component state.
/// The source path is a preview/reload hint; playback uses the embedded audio.
pub fn sampler_params(instrument: &Instrument) -> Option<SamplerParams> {
    let Instrument::Plugin(plugin) = instrument else {
        return match instrument {
            Instrument::Sampler(p) => Some(p.clone()),
            _ => None,
        };
    };
    if LimePlugin::of(plugin) != Some(LimePlugin::SAMPLER) {
        return None;
    }
    let mut p = SamplerParams {
        sample: plugin
            .extra
            .get("lime-sample-path")
            .and_then(Value::as_str)
            .map(|path| crate::SampleRef { path: path.into() }),
        ..Default::default()
    };
    if let Some(state) = component_state(&plugin.state).and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok()) {
        let defaults = sampler_values(&p, 60);
        let value = |i: usize| {
            state["params"][format!("value_{}", i + 1)]["f32"]
                .as_f64()
                .filter(|v| v.is_finite())
                .map(|v| v as f32)
                .unwrap_or(defaults[i])
        };
        p.gain_db = value(0);
        p.pitch = value(1);
        p.start = value(2);
        p.end = value(3);
        p.release_ms = value(4);
        p.reverse = value(5) >= 0.5;
        p.one_shot = value(6) >= 0.5;
        p.attack_ms = value(8);
        p.decay_ms = value(9);
        p.sustain = value(10);
        p.cutoff_hz = value(11);
        p.resonance = value(12);
        p.looping = value(13) >= 0.5;
        p.loop_start = value(14);
        p.loop_end = value(15);
    }
    Some(p)
}

/// Settings of a former built-in sampler that Lime Sampler cannot reproduce yet (the P0 features
/// listed in decision D39), as user-facing phrases. Lime Sampler now has every sampler setting.
pub fn sampler_losses(_: &SamplerParams) -> Vec<&'static str> {
    Vec::new()
}

/// Replace the former built-in instruments and exported effects of `project` with Lime plugins,
/// keeping their settings. `audio` decodes a sampler's stored sample path (`None`: missing or
/// unreadable, and the sampler is converted without audio). Returns one warning per setting that
/// could not be carried over; an empty list means nothing was lost. Gate, Drive and Chorus slots
/// are preserved disabled until they have Lime plugins.
pub fn convert_project(project: &mut Project, audio: &dyn Fn(&str) -> Option<EmbeddedAudio>) -> Vec<String> {
    let mut warnings = Vec::new();
    if project.playlist.clips.iter().any(|c| c.audio_channel().is_some())
        && !project.extra.contains_key("lime-hosted-audio-warning")
    {
        warnings.push("Playlist audio clips are preserved but silent with hosted samplers (PLG-024)".into());
        project.extra.insert("lime-hosted-audio-warning".into(), json!(true));
    }
    if project.patterns.iter().flat_map(|p| &p.notes).any(|n| n.pan != 0.0)
        && !project.extra.contains_key("lime-note-pan-warning")
    {
        warnings.push("Per-note pan is preserved but inaudible with hosted instruments (PLG-025)".into());
        project.extra.insert("lime-note-pan-warning".into(), json!(true));
    }
    // Effect automation becomes automation of the Lime plugin's parameter, before the slots change.
    let mut retarget = Vec::new();
    for (i, a) in project.automations.iter().enumerate() {
        let AutomationTarget::EffectParam { track, effect, index } = a.target else {
            continue;
        };
        let Some(slot) = project
            .mixer
            .tracks
            .get(usize::from(track))
            .and_then(|t| t.effects.iter().find(|e| e.id == effect))
        else {
            continue;
        };
        if LimePlugin::for_effect(slot.kind).is_none() {
            continue;
        }
        if slot.kind == EffectKind::Compressor && index >= COMPRESSOR_SIDECHAIN {
            warnings.push(format!(
                "{}: Lime Comp has no mixer-track sidechain input to automate",
                a.name
            ));
            continue;
        }
        let (Some(mapping), Some(spec)) = (a.target.mapping(project), slot.kind.params().get(index)) else {
            continue;
        };
        let values: Vec<f32> = a
            .points
            .iter()
            .map(|p| plugin_normalized(spec, mapping.to_value(p.value)))
            .collect();
        let target = AutomationTarget::PluginParam {
            plugin: PluginTarget::Effect { track, effect },
            id: param_id(index),
        };
        retarget.push((i, target, values));
    }
    for (i, target, values) in retarget {
        let a = &mut project.automations[i];
        a.target = target;
        for (p, v) in a.points.iter_mut().zip(values) {
            p.value = v;
        }
    }
    for c in &mut project.channels {
        let converted = match &c.instrument {
            Instrument::Synth(p) => Some(LimePlugin::SYNTH.with_values(&synth_values(p), false, None)),
            Instrument::Sampler(p) => {
                let sample = p.sample.as_ref().and_then(|s| {
                    let a = audio(&s.path);
                    if a.is_none() {
                        warnings.push(format!("{}: the sample {} could not be loaded", c.name, s.path));
                    }
                    a
                });
                for lost in sampler_losses(p) {
                    warnings.push(format!("{}: Lime Sampler does not have {lost} yet", c.name));
                }
                {
                    let mut plugin =
                        LimePlugin::SAMPLER.with_values(&sampler_values(p, c.root_key), false, sample.as_ref());
                    if let Some(source) = &p.sample {
                        plugin.extra.insert("lime-sample-path".into(), json!(source.path));
                    }
                    Some(plugin)
                }
            }
            Instrument::Plugin(plugin)
                if plugin.extra.get("lime-sample-pending").and_then(Value::as_bool) == Some(true)
                    && LimePlugin::of(plugin) == Some(LimePlugin::SAMPLER) =>
            {
                let mut plugin = plugin.clone();
                if let Some(p) = sampler_params(&c.instrument) {
                    let sample = p.sample.as_ref().and_then(|s| audio(&s.path));
                    if sample.is_none() {
                        warnings.push(format!("{}: the sample could not be loaded", c.name));
                    }
                    let root = plain_values(&plugin, &sampler_values(&p, c.root_key))[7]
                        .round()
                        .clamp(0.0, 127.0) as u8;
                    let state = LimePlugin::SAMPLER.with_values(&sampler_values(&p, root), false, sample.as_ref());
                    plugin.state = state.state;
                }
                plugin.extra.remove("lime-sample-pending");
                Some(plugin)
            }
            _ => None,
        };
        if let Some(p) = converted {
            c.instrument = Instrument::Plugin(p);
        }
    }
    for (t, track) in project.mixer.tracks.iter_mut().enumerate() {
        if track.eq != crate::EqBand::DEFAULTS && !track.extra.contains_key("lime-legacy-eq-inactive") {
            warnings.push(format!(
                "Mixer track {t}: legacy track equalizer is inactive; use a Lime EQ insert"
            ));
            track.extra.insert("lime-legacy-eq-inactive".into(), json!(true));
        }
        for slot in &mut track.effects {
            let Some(lime) = LimePlugin::for_effect(slot.kind) else {
                if slot.kind != EffectKind::Plugin && slot.enabled {
                    warnings.push(format!(
                        "Mixer track {t}: {:?} disabled; no Lime VST3 replacement yet",
                        slot.kind
                    ));
                    slot.enabled = false;
                }
                continue;
            };
            let mut values = slot.params.clone();
            if slot.kind == EffectKind::Compressor && values.len() > COMPRESSOR_SIDECHAIN {
                if values[COMPRESSOR_SIDECHAIN] > 0.0 {
                    warnings.push(format!(
                        "Mixer track {t}: Lime Comp has no mixer-track sidechain input yet"
                    ));
                }
                values.truncate(COMPRESSOR_SIDECHAIN);
            }
            slot.plugin = Some(lime.with_values(&values, !slot.enabled, None));
            slot.kind = EffectKind::Plugin;
            slot.params.clear();
        }
    }
    warnings
}

/// Whether `project` still uses a former built-in instrument or an effect that has a Lime plugin.
pub fn needs_conversion(project: &Project) -> bool {
    if (project.playlist.clips.iter().any(|c| c.audio_channel().is_some())
        && !project.extra.contains_key("lime-hosted-audio-warning"))
        || (project.patterns.iter().flat_map(|p| &p.notes).any(|n| n.pan != 0.0)
            && !project.extra.contains_key("lime-note-pan-warning"))
    {
        return true;
    }
    project.channels.iter().any(|c| {
        matches!(c.instrument, Instrument::Synth(_) | Instrument::Sampler(_))
            || matches!(&c.instrument, Instrument::Plugin(p) if LimePlugin::of(p) == Some(LimePlugin::SAMPLER)
                && p.extra.get("lime-sample-pending").and_then(Value::as_bool) == Some(true))
    }) || project
        .mixer
        .tracks
        .iter()
        .any(|t| t.eq != crate::EqBand::DEFAULTS && !t.extra.contains_key("lime-legacy-eq-inactive"))
        || project
            .mixer
            .tracks
            .iter()
            .flat_map(|t| &t.effects)
            .any(|e| LimePlugin::for_effect(e.kind).is_some() || (e.kind != EffectKind::Plugin && e.enabled))
}

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
pub const SYNTH_CONTROLS: &[ParamSpec] = &[
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
pub const SAMPLER_CONTROLS: &[ParamSpec] = &[
    spec("Gain", -60.0, 12.0, 0.0, "dB", false),
    spec("Pitch", -48.0, 48.0, 0.0, "st", false),
    spec("Start", 0.0, 1.0, 0.0, "", false),
    spec("End", 0.0, 1.0, 1.0, "", false),
    spec("Release", 0.0, 5000.0, 10.0, "ms", false),
    choice("Reverse", &["Forward", "Reverse"], 0.0),
    choice("Playback", &["Gated", "One-shot"], 1.0),
    spec("Root key", 0.0, 127.0, 60.0, "MIDI", false),
    // Appended (PLG-014) so earlier parameter IDs and saved states stay valid; the defaults are a gate.
    spec("Attack", 0.0, 10000.0, 0.0, "ms", false),
    spec("Decay", 0.0, 10000.0, 0.0, "ms", false),
    spec("Sustain", 0.0, 1.0, 1.0, "", false),
    // Appended (PLG-016): the low-pass filter, open by default.
    spec("Cutoff", 20.0, 20000.0, 20000.0, "Hz", true),
    spec("Resonance", 0.0, 0.95, 0.0, "", false),
    // Appended (PLG-015): loop the region while the note is held, as fractions of the sample.
    choice("Loop", &["Off", "On"], 0.0),
    spec("Loop start", 0.0, 1.0, 0.0, "", false),
    spec("Loop end", 0.0, 1.0, 1.0, "", false),
];

/// Return controls in the stable native plugin order (IDs 0–8), or an empty slice for other IDs.
/// Compressor controls omit the DAW's mixer-track selector; its key comes from a host bus.
pub fn control_specs(id: u8) -> &'static [ParamSpec] {
    match id {
        0 => SYNTH_CONTROLS,
        1 => SAMPLER_CONTROLS,
        7 => &LimePlugin::EFFECTS[5].params()[..5], // The external key is a bus, not a mixer track number.
        2..=8 => LimePlugin::EFFECTS[(id - 2) as usize].params(),
        _ => &[],
    }
}

/// Update one of our known controls without waiting for an audio callback to serialize it.
/// Other fields, including embedded sample audio, remain intact.
pub fn edited_param_state(plugin: &PluginRef, id: u32, normalized: f64) -> Option<PluginState> {
    let lime = LimePlugin::of(plugin)?;
    let mut state: Value = component_state(&plugin.state)
        .and_then(|s| serde_json::from_slice(s).ok())
        .unwrap_or_else(|| json!({"version": "0.1.0", "params": {}, "fields": {}}));
    if !normalized.is_finite() {
        return None;
    }
    let params = state
        .as_object_mut()?
        .entry("params")
        .or_insert_with(|| json!({}))
        .as_object_mut()?;
    let hash = |s: &str| {
        s.bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
            & !(1 << 31)
    };
    if let Some(index) = control_specs(lime.0)
        .iter()
        .enumerate()
        .find_map(|(i, _)| (param_id(i) == id).then_some(i))
    {
        let spec = &control_specs(lime.0)[index];
        let fraction = (normalized.clamp(0.0, 1.0) as f32).powf(if spec.log { 4.0 } else { 1.0 });
        let mut plain = spec.min + fraction * (spec.max - spec.min);
        if !spec.labels.is_empty()
            || (lime == LimePlugin::SYNTH && index == 1)
            || (lime == LimePlugin::SAMPLER && index == 7)
        {
            plain = plain.round();
        }
        params.insert(format!("value_{}", index + 1), json!({"f32": plain}));
    } else {
        let key = ["bypass", "key"].into_iter().find(|name| hash(name) == id)?;
        params.insert(key.into(), json!({"bool": normalized >= 0.5}));
    }
    Some(PluginState::new(host_state(state.to_string().as_bytes())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_sampler_embeds_once_and_keeps_source_settings() {
        let params = SamplerParams {
            sample: Some(crate::SampleRef {
                path: "kick.wav".into(),
            }),
            attack_ms: 25.0,
            looping: true,
            loop_start: 0.2,
            loop_end: 0.8,
            ..Default::default()
        };
        let mut doc = crate::Document::default();
        doc.execute(crate::Command::AddChannel {
            name: "Kick".into(),
            instrument: Instrument::Plugin(sampler_ref(&params, 48)),
        })
        .unwrap();
        let mut project = doc.project().clone();
        assert!(needs_conversion(&project));
        let loaded = std::cell::Cell::new(0);
        let audio = |path: &str| {
            assert_eq!(path, "kick.wav");
            loaded.set(loaded.get() + 1);
            Some(EmbeddedAudio {
                name: path.into(),
                sample_rate: 48_000,
                stereo: vec![0.5, -0.5],
            })
        };
        assert!(convert_project(&mut project, &audio).is_empty());
        assert!(!needs_conversion(&project));
        assert!(convert_project(&mut project, &audio).is_empty());
        assert_eq!(loaded.get(), 1);
        assert_eq!(sampler_params(&project.channels[0].instrument), Some(params));
        let Instrument::Plugin(plugin) = &project.channels[0].instrument else {
            panic!("plugin")
        };
        let state: Value = serde_json::from_slice(component_state(&plugin.state).unwrap()).unwrap();
        assert_eq!(state["params"]["value_8"]["f32"], 48.0);
        assert!(state["fields"]["sample-v1"].is_string());
    }

    #[test]
    fn parameter_state_edits_keep_audio_and_undo_as_one_gesture() {
        let audio = EmbeddedAudio {
            name: "hit.wav".into(),
            sample_rate: 48_000,
            stereo: vec![0.5, -0.5],
        };
        let plugin =
            LimePlugin::SAMPLER.with_values(&sampler_values(&SamplerParams::default(), 60), false, Some(&audio));
        let mut doc = crate::Document::default();
        let channel = crate::ChannelId(
            doc.execute(crate::Command::AddChannel {
                name: "Hit".into(),
                instrument: Instrument::Plugin(plugin.clone()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let target = PluginTarget::Channel(channel);
        doc.begin_gesture();
        for normalized in [0.25, 0.5, 0.75] {
            let state = edited_param_state(doc.project().plugin(target).unwrap(), param_id(8), normalized).unwrap();
            doc.execute(crate::Command::SetPluginState { plugin: target, state })
                .unwrap();
        }
        doc.end_gesture();
        assert_eq!(doc.history().len(), 2);
        let before: Value = serde_json::from_slice(component_state(&plugin.state).unwrap()).unwrap();
        let after: Value =
            serde_json::from_slice(component_state(&doc.project().plugin(target).unwrap().state).unwrap()).unwrap();
        assert_eq!(after["params"]["value_9"]["f32"], 7500.0);
        assert_eq!(before["fields"], after["fields"]);
        assert!(doc.undo());
        assert_eq!(doc.project().plugin(target).unwrap().state, plugin.state);
        assert!(doc.redo());
        assert_eq!(
            sampler_params(&doc.project().channel(channel).unwrap().instrument)
                .unwrap()
                .attack_ms,
            7500.0
        );
        let mut malformed = plugin;
        malformed.state = PluginState::new(host_state(b"false"));
        assert!(edited_param_state(&malformed, param_id(8), 0.5).is_none());
        assert!(edited_param_state(&malformed, param_id(8), f64::NAN).is_none());
    }

    #[test]
    fn unsupported_legacy_audio_warns_once_and_preserves_settings() {
        let mut project = Project::default();
        project.mixer.tracks[0].eq[1].gain = 8.0;
        let kinds = [EffectKind::Gate, EffectKind::Drive, EffectKind::Chorus];
        project.mixer.tracks[0].effects = kinds
            .into_iter()
            .enumerate()
            .map(|(i, kind)| crate::EffectSlot::new(i as u32 + 100, kind))
            .collect();
        let before = project.mixer.tracks[0].clone();
        assert!(needs_conversion(&project));
        let warnings = convert_project(&mut project, &|_| None);
        assert_eq!(warnings.len(), 4, "{warnings:?}");
        assert_eq!(project.mixer.tracks[0].eq, before.eq);
        for (after, before) in project.mixer.tracks[0].effects.iter().zip(&before.effects) {
            assert!(!after.enabled);
            assert_eq!(after.kind, before.kind);
            assert_eq!(after.params, before.params);
        }
        assert!(!needs_conversion(&project));
        assert!(convert_project(&mut project, &|_| None).is_empty());
    }

    #[test]
    fn class_ids_match_the_exported_plugins() {
        // "LIMENATIVE" = 4C 49 4D 45 4E 41 54 49 56 45.
        assert_eq!(LimePlugin::SYNTH.class_id(), "4C494D454E4154495645000000000100");
        assert_eq!(LimePlugin(8).class_id(), "4C494D454E4154495645000000000108");
        assert_eq!(LimePlugin::for_effect(EffectKind::Limiter), Some(LimePlugin(8)));
        assert_eq!(LimePlugin::for_effect(EffectKind::Chorus), None);
        let r = LimePlugin(3).plugin_ref();
        assert_eq!(r.path, "lime:Lime EQ.vst3");
        assert_eq!(LimePlugin::of(&r), Some(LimePlugin(3)));
    }

    #[test]
    fn parameter_ids_and_values_match_nice_plug() {
        // nice-plug's hash of "value_1".
        let expected = "value_1"
            .bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
            & 0x7FFF_FFFF;
        assert_eq!(param_id(0), expected);
        assert_ne!(param_id(0), param_id(1));
        let freq = *EffectKind::Filter.params().iter().find(|p| p.log).unwrap();
        assert_eq!(plugin_normalized(&freq, freq.max), 1.0);
        assert_eq!(plugin_normalized(&freq, freq.min), 0.0);
        let mid = (0.5f32).powf(0.25);
        assert!((plugin_normalized(&freq, (freq.min + freq.max) / 2.0) - mid).abs() < 1e-6);
    }

    #[test]
    fn state_is_the_plugins_own_json() {
        let audio = EmbeddedAudio {
            name: "a.wav".into(),
            sample_rate: 44_100,
            stereo: vec![0.5, -0.5],
        };
        let r = LimePlugin::SAMPLER.with_values(&[1.0, 2.0], true, Some(&audio));
        let v: Value = serde_json::from_slice(component_state(&r.state).unwrap()).unwrap();
        assert_eq!(v["params"]["value_1"]["f32"], 1.0);
        assert_eq!(v["params"]["value_2"]["f32"], 2.0);
        assert_eq!(v["params"]["bypass"]["bool"], true);
        let sample: Value = serde_json::from_str(v["fields"]["sample-v1"].as_str().unwrap()).unwrap();
        assert_eq!(sample["sample_rate"], 44_100);
        assert_eq!(sample["pcm"], base64_encode(&[0, 0, 0, 63, 0, 0, 0, 191]));
    }

    #[test]
    fn converts_built_in_instruments_and_effects() {
        let mut project = Project::default();
        let synth = SynthParams {
            filter_mode: FilterMode::HighPass,
            ..SynthParams::default()
        };
        let mut c = project.channels.first().cloned().unwrap_or_else(|| crate::Channel {
            id: crate::ChannelId(1),
            name: "Lead".into(),
            color: crate::Rgb(0, 0, 0),
            instrument: Instrument::Synth(SynthParams::default()),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            mute_locked: false,
            mixer_track: 0,
            root_key: 60,
            group: None,
            extra: Default::default(),
        });
        c.instrument = Instrument::Synth(synth.clone());
        let mut s = c.clone();
        s.name = "Kick".into();
        s.instrument = Instrument::Sampler(SamplerParams {
            sample: Some(crate::SampleRef {
                path: "kick.wav".into(),
            }),
            looping: true,
            attack_ms: 12.0,
            ..SamplerParams::default()
        });
        project.channels = vec![c, s];
        let mut comp = crate::EffectSlot::new(7, EffectKind::Compressor);
        comp.enabled = false;
        project.mixer.tracks[0].effects = vec![comp, crate::EffectSlot::new(8, EffectKind::Chorus)];
        assert!(needs_conversion(&project));

        let warnings = convert_project(&mut project, &|path| {
            (path == "kick.wav").then(|| EmbeddedAudio {
                name: path.into(),
                sample_rate: 48_000,
                stereo: vec![0.0; 4],
            })
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("Chorus disabled"));
        let Instrument::Plugin(p) = &project.channels[0].instrument else {
            panic!("not converted")
        };
        assert_eq!(LimePlugin::of(p), Some(LimePlugin::SYNTH));
        let v: Value = serde_json::from_slice(component_state(&p.state).unwrap()).unwrap();
        assert_eq!(v["params"]["value_16"]["f32"], 2.0);
        let Instrument::Plugin(p) = &project.channels[1].instrument else {
            panic!("not converted")
        };
        let v: Value = serde_json::from_slice(component_state(&p.state).unwrap()).unwrap();
        assert_eq!(v["params"]["value_8"]["f32"], 60.0);
        assert_eq!(v["params"]["value_9"]["f32"], 12.0);
        assert_eq!(v["params"]["value_14"]["f32"], 1.0);
        assert!(v["fields"]["sample-v1"].is_string());
        let fx = &project.mixer.tracks[0].effects;
        assert_eq!(fx[0].kind, EffectKind::Plugin);
        let v: Value = serde_json::from_slice(component_state(&fx[0].plugin.as_ref().unwrap().state).unwrap()).unwrap();
        assert_eq!(v["params"]["bypass"]["bool"], true);
        assert!(v["params"].get("value_6").is_none());
        assert_eq!(fx[1].kind, EffectKind::Chorus);
        assert!(!fx[1].enabled);
        assert!(convert_project(&mut project, &|_| None).is_empty());
        assert!(!needs_conversion(&project));
    }
}
