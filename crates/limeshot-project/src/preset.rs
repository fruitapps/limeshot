//! Presets for the built-in effects and instruments (FL Studio's .fst presets): small JSON files
//! in a folder per effect kind or instrument, plus factory presets built into LimeShot.
//!
//! A preset file: `{"format": "limeshot-preset", "version": 1, "name": "…", "effect": {"kind": "Delay",
//! "params": [...]}}` for an effect, or `"synth": {...}` / `"sampler": {...}` for an instrument.
//! A sampler preset leaves the sample out: loading it keeps the channel's own sample.

use limeshot_core::{EffectKind, EffectSlot, SamplerParams, SynthParams, Waveform};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const EXTENSION: &str = "limepreset";
const FORMAT: &str = "limeshot-preset";
const VERSION: u64 = 1;
/// Preset files are a few hundred bytes; anything much larger is not one.
const MAX_BYTES: u64 = 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum PresetError {
    #[error("could not read or write {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{0} is not a LimeShot preset")]
    NotAPreset(PathBuf),
    #[error("{0} is a preset for something else")]
    WrongKind(PathBuf),
    #[error("a preset needs a name")]
    NoName,
}

/// What a preset sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Content {
    /// An effect's parameters, in its parameter order.
    Effect {
        kind: EffectKind,
        params: Vec<f32>,
    },
    Synth(SynthParams),
    /// A sampler's settings without the sample.
    Sampler(SamplerParams),
}

/// Which presets a folder holds: one folder per effect kind and per instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Effect(EffectKind),
    Synth,
    Sampler,
}

impl Slot {
    pub fn folder(self) -> String {
        match self {
            Slot::Effect(k) => format!("{k:?}"),
            Slot::Synth => "LimeSynth".into(),
            Slot::Sampler => "Sampler".into(),
        }
    }

    fn matches(self, c: &Content) -> bool {
        match (self, c) {
            (Slot::Effect(k), Content::Effect { kind, .. }) => k == *kind,
            (Slot::Synth, Content::Synth(_)) | (Slot::Sampler, Content::Sampler(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    #[serde(flatten)]
    pub content: Content,
}

#[derive(Serialize, Deserialize)]
struct File {
    format: String,
    version: u64,
    #[serde(flatten)]
    preset: Preset,
}

/// A preset that can be loaded: a factory one, or a file in the user's folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    /// `None` for a factory preset.
    pub path: Option<PathBuf>,
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> PresetError + '_ {
    move |source| PresetError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// A file name for a preset name: letters, digits, spaces, dashes and underscores kept.
fn file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || " -_".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}.{EXTENSION}", clean.trim())
}

/// Save `preset` in `root/<slot folder>/`, replacing one of the same name. Returns its path.
pub fn save(root: &Path, preset: &Preset) -> Result<PathBuf, PresetError> {
    let name = preset.name.trim();
    if name.is_empty() {
        return Err(PresetError::NoName);
    }
    let slot = match &preset.content {
        Content::Effect { kind, .. } => Slot::Effect(*kind),
        Content::Synth(_) => Slot::Synth,
        Content::Sampler(_) => Slot::Sampler,
    };
    let dir = root.join(slot.folder());
    fs::create_dir_all(&dir).map_err(io(&dir))?;
    let path = dir.join(file_name(name));
    let mut p = preset.clone();
    p.name = name.to_string();
    if let Content::Sampler(s) = &mut p.content {
        s.sample = None;
    }
    let text = serde_json::to_string_pretty(&File {
        format: FORMAT.into(),
        version: VERSION,
        preset: p,
    })
    .map_err(|e| PresetError::Io {
        path: path.clone(),
        source: std::io::Error::other(e),
    })?;
    fs::write(&path, text).map_err(io(&path))?;
    Ok(path)
}

/// Read a preset file meant for `slot`. Effect parameters are brought to the effect's count and
/// ranges.
pub fn load(path: &Path, slot: Slot) -> Result<Preset, PresetError> {
    let meta = fs::metadata(path).map_err(io(path))?;
    if meta.len() > MAX_BYTES {
        return Err(PresetError::NotAPreset(path.to_path_buf()));
    }
    let text = fs::read_to_string(path).map_err(io(path))?;
    let file: File = serde_json::from_str(&text).map_err(|_| PresetError::NotAPreset(path.to_path_buf()))?;
    if file.format != FORMAT || file.version > VERSION {
        return Err(PresetError::NotAPreset(path.to_path_buf()));
    }
    let mut preset = file.preset;
    if !slot.matches(&preset.content) {
        return Err(PresetError::WrongKind(path.to_path_buf()));
    }
    match &mut preset.content {
        Content::Effect { kind, params } => {
            let mut e = EffectSlot::new(0, *kind);
            e.params = std::mem::take(params);
            e.sanitize();
            *params = e.params;
        }
        Content::Sampler(s) => s.sample = None,
        Content::Synth(_) => {}
    }
    Ok(preset)
}

/// The presets for `slot`: the factory ones, then the user's files in `root` (sorted by name).
pub fn list(root: Option<&Path>, slot: Slot) -> Vec<Entry> {
    let mut out: Vec<Entry> = factory(slot)
        .into_iter()
        .map(|p| Entry {
            name: p.name,
            path: None,
        })
        .collect();
    let mut user: Vec<Entry> = root
        .map(|r| r.join(slot.folder()))
        .and_then(|d| fs::read_dir(d).ok())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == EXTENSION))
        .filter_map(|p| {
            load(&p, slot).ok().map(|preset| Entry {
                name: preset.name,
                path: Some(p),
            })
        })
        .collect();
    user.sort_by_key(|e| e.name.to_lowercase());
    out.extend(user);
    out
}

/// Load an entry from [`list`].
pub fn open(entry: &Entry, slot: Slot) -> Result<Preset, PresetError> {
    match &entry.path {
        Some(p) => load(p, slot),
        None => factory(slot)
            .into_iter()
            .find(|p| p.name == entry.name)
            .ok_or_else(|| PresetError::NotAPreset(PathBuf::from(&entry.name))),
    }
}

/// An effect preset from parameter names and values; the rest keep their defaults.
fn effect(name: &str, kind: EffectKind, values: &[(&str, f32)]) -> Preset {
    let mut e = EffectSlot::new(0, kind);
    for (i, spec) in kind.params().iter().enumerate() {
        if let (Some(v), Some(slot)) = (values.iter().find(|(n, _)| *n == spec.name), e.params.get_mut(i)) {
            *slot = v.1;
        }
    }
    e.sanitize();
    Preset {
        name: name.into(),
        content: Content::Effect { kind, params: e.params },
    }
}

fn synth(name: &str, f: impl FnOnce(&mut SynthParams)) -> Preset {
    let mut s = SynthParams::default();
    f(&mut s);
    Preset {
        name: name.into(),
        content: Content::Synth(s),
    }
}

/// LimeShot's own factory presets (not FL Studio's).
pub fn factory(slot: Slot) -> Vec<Preset> {
    use EffectKind as K;
    match slot {
        Slot::Effect(K::Delay) => vec![
            effect(
                "Slapback",
                K::Delay,
                &[("Time", 90.0), ("Feedback", 0.1), ("Mix", 0.25)],
            ),
            effect(
                "Dotted eighth",
                K::Delay,
                &[("Sync", 4.0), ("Feedback", 0.45), ("Mix", 0.3)],
            ),
            effect(
                "Wide quarter",
                K::Delay,
                &[("Sync", 5.0), ("Feedback", 0.5), ("Spread", 1.0), ("Tone", 3000.0)],
            ),
        ],
        Slot::Effect(K::Reverb) => vec![
            effect(
                "Small room",
                K::Reverb,
                &[("Size", 0.3), ("Damping", 0.6), ("Mix", 0.2)],
            ),
            effect(
                "Hall",
                K::Reverb,
                &[("Size", 0.85), ("Damping", 0.3), ("Pre-delay", 30.0), ("Mix", 0.3)],
            ),
            effect(
                "Dark ambience",
                K::Reverb,
                &[("Size", 0.95), ("Damping", 0.8), ("Mix", 0.45)],
            ),
        ],
        Slot::Effect(K::Compressor) => vec![
            effect(
                "Gentle bus",
                K::Compressor,
                &[
                    ("Threshold", -12.0),
                    ("Ratio", 2.0),
                    ("Attack", 30.0),
                    ("Release", 200.0),
                ],
            ),
            effect(
                "Punchy drums",
                K::Compressor,
                &[
                    ("Threshold", -20.0),
                    ("Ratio", 4.0),
                    ("Attack", 15.0),
                    ("Release", 80.0),
                    ("Makeup", 4.0),
                ],
            ),
            effect(
                "Squash",
                K::Compressor,
                &[
                    ("Threshold", -30.0),
                    ("Ratio", 12.0),
                    ("Attack", 1.0),
                    ("Release", 60.0),
                    ("Makeup", 10.0),
                ],
            ),
        ],
        Slot::Effect(K::Limiter) => vec![
            effect("Master safety", K::Limiter, &[("Ceiling", -0.3), ("Release", 80.0)]),
            effect("Loud", K::Limiter, &[("Ceiling", -1.0), ("Release", 30.0)]),
        ],
        Slot::Effect(K::Filter) => vec![
            effect(
                "Muffled",
                K::Filter,
                &[("Mode", 0.0), ("Cutoff", 800.0), ("Resonance", 0.1)],
            ),
            effect(
                "Telephone",
                K::Filter,
                &[("Mode", 1.0), ("Cutoff", 1500.0), ("Resonance", 0.5)],
            ),
            effect(
                "Low cut",
                K::Filter,
                &[("Mode", 2.0), ("Cutoff", 120.0), ("Resonance", 0.0)],
            ),
        ],
        Slot::Effect(K::Eq3) => vec![
            effect("Bass boost", K::Eq3, &[("Low gain", 6.0), ("Low freq", 90.0)]),
            effect("Air", K::Eq3, &[("High gain", 4.0), ("High freq", 10000.0)]),
            effect(
                "Scoop",
                K::Eq3,
                &[("Mid gain", -6.0), ("Mid freq", 700.0), ("Mid Q", 0.7)],
            ),
        ],
        Slot::Effect(K::Utility) => vec![
            effect("Mono", K::Utility, &[("Width", 0.0)]),
            effect("Wide", K::Utility, &[("Width", 1.6)]),
        ],
        Slot::Effect(_) => Vec::new(),
        Slot::Synth => vec![
            synth("Saw lead", |s| {
                s.waveform = Waveform::Saw;
                s.detune_cents = 8.0;
                s.cutoff_hz = 6000.0;
                s.vibrato_cents = 10.0;
                s.lfo_rate_hz = 5.5;
            }),
            synth("Soft pad", |s| {
                s.waveform = Waveform::Triangle;
                s.attack_ms = 600.0;
                s.release_ms = 1200.0;
                s.sustain = 0.9;
                s.detune_cents = 12.0;
                s.cutoff_hz = 3000.0;
            }),
            synth("Pluck", |s| {
                s.waveform = Waveform::Square;
                s.attack_ms = 2.0;
                s.decay_ms = 180.0;
                s.sustain = 0.0;
                s.release_ms = 120.0;
                s.cutoff_hz = 900.0;
                s.resonance = 0.4;
                s.filter_env = 4.0;
                s.filter_decay_ms = 160.0;
            }),
            synth("Sub bass", |s| {
                s.waveform = Waveform::Sine;
                s.octave = -1;
                s.attack_ms = 5.0;
                s.sustain = 1.0;
                s.release_ms = 80.0;
                s.gain_db = -3.0;
            }),
        ],
        Slot::Sampler => vec![
            Preset {
                name: "One-shot drum".into(),
                content: Content::Sampler(SamplerParams::default()),
            },
            Preset {
                name: "Sustained (follows notes)".into(),
                content: Content::Sampler(SamplerParams {
                    one_shot: false,
                    release_ms: 120.0,
                    ..SamplerParams::default()
                }),
            },
            Preset {
                name: "Reversed".into(),
                content: Content::Sampler(SamplerParams {
                    reverse: true,
                    ..SamplerParams::default()
                }),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_presets_are_valid_for_their_slot() {
        for k in EffectKind::ALL {
            for p in factory(Slot::Effect(k)) {
                let Content::Effect { kind, params } = &p.content else {
                    panic!("{}", p.name)
                };
                assert_eq!(*kind, k);
                assert_eq!(params.len(), k.params().len(), "{}", p.name);
            }
        }
        assert_eq!(factory(Slot::Synth).len(), 4);
        // Named values land on the right parameter.
        let slap = &factory(Slot::Effect(EffectKind::Delay))[0];
        let Content::Effect { params, .. } = &slap.content else {
            panic!()
        };
        assert_eq!((params[0], params[1]), (90.0, 0.1));
    }

    #[test]
    fn saved_presets_list_after_the_factory_ones_and_load_back() {
        let dir = tempfile::tempdir().unwrap();
        let slot = Slot::Effect(EffectKind::Delay);
        let mut p = factory(slot)[1].clone();
        p.name = "My echo: v2".into();
        let path = save(dir.path(), &p).unwrap();
        assert!(path.ends_with("Delay/My echo_ v2.limepreset"));
        let entries = list(Some(dir.path()), slot);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[3].name, "My echo: v2");
        assert_eq!(open(&entries[3], slot).unwrap(), p);
        assert_eq!(open(&entries[0], slot).unwrap(), factory(slot)[0]);
        // Wrong slot, no name, not a preset.
        assert!(matches!(load(&path, Slot::Synth), Err(PresetError::WrongKind(_))));
        assert!(matches!(
            save(
                dir.path(),
                &Preset {
                    name: " ".into(),
                    ..p.clone()
                }
            ),
            Err(PresetError::NoName)
        ));
        let junk = dir.path().join("Delay/junk.limepreset");
        fs::write(&junk, "{}").unwrap();
        assert!(load(&junk, slot).is_err());
        assert_eq!(list(Some(dir.path()), slot).len(), 4, "junk is skipped");
        // A sampler preset never keeps a sample.
        let s = SamplerParams {
            sample: Some(limeshot_core::SampleRef {
                path: "kick.wav".into(),
            }),
            ..SamplerParams::default()
        };
        let sp = Preset {
            name: "S".into(),
            content: Content::Sampler(s),
        };
        let path = save(dir.path(), &sp).unwrap();
        let Content::Sampler(back) = load(&path, Slot::Sampler).unwrap().content else {
            panic!()
        };
        assert!(back.sample.is_none());
    }
}
