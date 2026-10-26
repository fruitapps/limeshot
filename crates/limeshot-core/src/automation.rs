//! Automation clips: a curve of points that drives one parameter during Song playback.
//!
//! As in FL Studio, an automation is created from a control ("Create automation clip"), starts as
//! a flat line at the control's current value, and is placed on the Playlist as clips that are
//! linked instances of the same curve. Points hold a normalised value (0–1 across the target's
//! range, on the same scale as the control's travel) and the tension of the segment that follows
//! them.

use crate::document::PluginTarget;
use crate::model::{ChannelId, Extra, Project, Rgb};
use crate::time::{MAX_TICK, Tick};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AutomationId(pub u32);

/// The parameter an automation drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutomationTarget {
    ChannelVolume(ChannelId),
    ChannelPan(ChannelId),
    MixerVolume(u16),
    MixerPan(u16),
    EffectParam {
        track: u16,
        effect: u32,
        index: usize,
    },
    /// Parameter `id` (VST3) of a hosted plugin; values are the plugin's own normalised ones.
    PluginParam {
        plugin: PluginTarget,
        id: u32,
    },
    /// The project tempo (FL Studio's tempo automation clip, 60–180 BPM by default).
    Tempo,
}

/// How a normalised value maps to the parameter's own units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mapping {
    Linear {
        min: f32,
        max: f32,
    },
    /// Logarithmic (frequencies, times); `min` > 0.
    Log {
        min: f32,
        max: f32,
    },
    /// Gain on a fader's perceptual travel: value = norm³ × max.
    Fader {
        max: f32,
    },
}

impl Mapping {
    pub fn to_value(self, norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        match self {
            Mapping::Linear { min, max } => min + (max - min) * n,
            Mapping::Log { min, max } => min * (max / min).powf(n),
            Mapping::Fader { max } => n * n * n * max,
        }
    }

    pub fn to_norm(self, v: f32) -> f32 {
        let n = match self {
            Mapping::Linear { min, max } if max > min => (v - min) / (max - min),
            Mapping::Linear { .. } => 0.0,
            Mapping::Log { min, max } if max > min && v > 0.0 => (v / min).ln() / (max / min).ln(),
            Mapping::Log { .. } => 0.0,
            Mapping::Fader { max } if max > 0.0 => (v / max).max(0.0).cbrt(),
            Mapping::Fader { .. } => 0.0,
        };
        if n.is_finite() { n.clamp(0.0, 1.0) } else { 0.0 }
    }
}

impl AutomationTarget {
    /// The mapping of this target, or `None` when it no longer exists (deleted channel/effect).
    pub fn mapping(&self, p: &Project) -> Option<Mapping> {
        match *self {
            AutomationTarget::ChannelVolume(c) => p.channel(c).map(|_| Mapping::Fader { max: 1.25 }),
            AutomationTarget::ChannelPan(c) => p.channel(c).map(|_| Mapping::Linear { min: -1.0, max: 1.0 }),
            AutomationTarget::MixerVolume(t) => p.mixer.tracks.get(usize::from(t)).map(|_| Mapping::Fader { max: 2.0 }),
            AutomationTarget::MixerPan(t) => p
                .mixer
                .tracks
                .get(usize::from(t))
                .map(|_| Mapping::Linear { min: -1.0, max: 1.0 }),
            AutomationTarget::EffectParam { track, effect, index } => {
                let e = p
                    .mixer
                    .tracks
                    .get(usize::from(track))?
                    .effects
                    .iter()
                    .find(|e| e.id == effect)?;
                let spec = e.kind.params().get(index)?;
                Some(if spec.log && spec.min > 0.0 {
                    Mapping::Log {
                        min: spec.min,
                        max: spec.max,
                    }
                } else {
                    Mapping::Linear {
                        min: spec.min,
                        max: spec.max,
                    }
                })
            }
            AutomationTarget::PluginParam { plugin, .. } => {
                p.plugin(plugin).map(|_| Mapping::Linear { min: 0.0, max: 1.0 })
            }
            AutomationTarget::Tempo => Some(Mapping::Linear { min: 60.0, max: 180.0 }),
        }
    }

    /// The parameter's current (unautomated) value.
    pub fn current_value(&self, p: &Project) -> Option<f32> {
        match *self {
            AutomationTarget::ChannelVolume(c) => p.channel(c).map(|c| c.volume),
            AutomationTarget::ChannelPan(c) => p.channel(c).map(|c| c.pan),
            AutomationTarget::MixerVolume(t) => p.mixer.tracks.get(usize::from(t)).map(|t| t.volume),
            AutomationTarget::MixerPan(t) => p.mixer.tracks.get(usize::from(t)).map(|t| t.pan),
            AutomationTarget::EffectParam { track, effect, index } => p
                .mixer
                .tracks
                .get(usize::from(track))?
                .effects
                .iter()
                .find(|e| e.id == effect)?
                .params
                .get(index)
                .copied(),
            // The value lives in the plugin, not the project.
            AutomationTarget::PluginParam { plugin, id } => crate::lime::parameter_value(p.plugin(plugin)?, id),
            AutomationTarget::Tempo => Some(p.tempo.bpm() as f32),
        }
    }

    /// The mixer track this target belongs to, if any.
    pub fn mixer_track(&self) -> Option<u16> {
        match *self {
            AutomationTarget::MixerVolume(t) | AutomationTarget::MixerPan(t) => Some(t),
            AutomationTarget::EffectParam { track, .. } => Some(track),
            AutomationTarget::PluginParam {
                plugin: PluginTarget::Effect { track, .. },
                ..
            } => Some(track),
            _ => None,
        }
    }

    /// The same target after mixer tracks were renumbered (a deleted track keeps its index; its
    /// automations are removed first).
    pub fn remap_mixer(self, map: impl Fn(u16) -> Option<u16>) -> Self {
        let m = |t: u16| map(t).unwrap_or(t);
        match self {
            AutomationTarget::MixerVolume(t) => AutomationTarget::MixerVolume(m(t)),
            AutomationTarget::MixerPan(t) => AutomationTarget::MixerPan(m(t)),
            AutomationTarget::EffectParam { track, effect, index } => AutomationTarget::EffectParam {
                track: m(track),
                effect,
                index,
            },
            AutomationTarget::PluginParam {
                plugin: PluginTarget::Effect { track, effect },
                id,
            } => AutomationTarget::PluginParam {
                plugin: PluginTarget::Effect {
                    track: m(track),
                    effect,
                },
                id,
            },
            other => other,
        }
    }

    /// A name such as "Kick – Volume" or "Insert 2 – Delay: Feedback".
    pub fn label(&self, p: &Project) -> String {
        let chan = |c: ChannelId| p.channel(c).map_or_else(|| "(deleted)".to_string(), |c| c.name.clone());
        let track = |t: u16| {
            p.mixer
                .tracks
                .get(usize::from(t))
                .map_or_else(|| format!("Track {t}"), |t| t.name.clone())
        };
        match *self {
            AutomationTarget::Tempo => "Tempo".into(),
            AutomationTarget::ChannelVolume(c) => format!("{} – Volume", chan(c)),
            AutomationTarget::ChannelPan(c) => format!("{} – Pan", chan(c)),
            AutomationTarget::MixerVolume(t) => format!("{} – Volume", track(t)),
            AutomationTarget::MixerPan(t) => format!("{} – Pan", track(t)),
            AutomationTarget::EffectParam {
                track: t,
                effect,
                index,
            } => {
                let e = p
                    .mixer
                    .tracks
                    .get(usize::from(t))
                    .and_then(|tr| tr.effects.iter().find(|e| e.id == effect));
                match e {
                    Some(e) => format!(
                        "{} – {}: {}",
                        track(t),
                        e.name(),
                        e.kind.params().get(index).map_or("?", |s| s.name)
                    ),
                    None => format!("{} – (deleted effect)", track(t)),
                }
            }
            AutomationTarget::PluginParam { plugin, id } => {
                let place = match plugin {
                    PluginTarget::Channel(c) => chan(c),
                    PluginTarget::Effect { track: t, .. } => track(t),
                };
                match p.plugin(plugin) {
                    Some(pl) => format!(
                        "{place} – {}: {}",
                        pl.name,
                        crate::lime::parameter_spec(pl, id)
                            .map(|(_, s)| s.name.to_string())
                            .unwrap_or_else(|| format!("parameter {id}"))
                    ),
                    None => format!("{place} – (removed plugin)"),
                }
            }
        }
    }
}

/// A breakpoint of an automation curve.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AutomationPoint {
    /// Position from the start of the automation.
    pub tick: Tick,
    /// Normalised value, 0–1.
    pub value: f32,
    /// Shape of the segment to the next point, −1–1 (0 = straight line). For the stepped and
    /// periodic modes it sets the number of steps or cycles.
    #[serde(default)]
    pub tension: f32,
    /// How the segment to the next point is drawn (FL Studio's point modes). Saved only when
    /// not the default single curve.
    #[serde(default, skip_serializing_if = "PointMode::is_single")]
    pub mode: PointMode,
}

/// FL Studio's automation point modes: the shape of the segment after a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PointMode {
    #[default]
    SingleCurve,
    DoubleCurve,
    Hold,
    Stairs,
    SmoothStairs,
    Pulse,
    Wave,
}

impl PointMode {
    pub const ALL: [PointMode; 7] = [
        PointMode::SingleCurve,
        PointMode::DoubleCurve,
        PointMode::Hold,
        PointMode::Stairs,
        PointMode::SmoothStairs,
        PointMode::Pulse,
        PointMode::Wave,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PointMode::SingleCurve => "Single curve",
            PointMode::DoubleCurve => "Double curve",
            PointMode::Hold => "Hold",
            PointMode::Stairs => "Stairs",
            PointMode::SmoothStairs => "Smooth stairs",
            PointMode::Pulse => "Pulse",
            PointMode::Wave => "Wave",
        }
    }

    fn is_single(&self) -> bool {
        *self == PointMode::SingleCurve
    }

    /// Whether the tension handle sets a count (steps, pulses, cycles) rather than a bend.
    pub fn counts(self) -> bool {
        matches!(
            self,
            PointMode::Stairs | PointMode::SmoothStairs | PointMode::Pulse | PointMode::Wave
        )
    }
}

/// Steps or cycles for the counting modes: tension −1–1 gives 1–16.
fn count(tension: f32) -> f32 {
    (((tension.clamp(-1.0, 1.0) + 1.0) / 2.0 * 15.0).round() + 1.0).max(1.0)
}

/// Position 0–1 along a segment of `mode` with `tension` at `x` (0–1): 0 is the segment's
/// first value, 1 the next point's. Allocation-free.
pub fn segment(x: f32, tension: f32, mode: PointMode) -> f32 {
    let x = x.clamp(0.0, 1.0);
    match mode {
        PointMode::SingleCurve => shape(x, tension),
        // Symmetric S: the first half bends one way, the second the other.
        PointMode::DoubleCurve => {
            if x < 0.5 {
                0.5 * shape(2.0 * x, tension)
            } else {
                1.0 - 0.5 * shape(2.0 * (1.0 - x), tension)
            }
        }
        PointMode::Hold => 0.0,
        PointMode::Stairs => {
            let n = count(tension);
            ((x * n).floor() / n).min(1.0)
        }
        PointMode::SmoothStairs => {
            let n = count(tension);
            let k = (x * n).floor();
            let f = x * n - k;
            // Each step eases into the next.
            let ease = f * f * (3.0 - 2.0 * f);
            ((k + ease) / n).min(1.0)
        }
        PointMode::Pulse => {
            let n = count(tension);
            if (x * n * 2.0).floor() as i64 % 2 == 0 {
                0.0
            } else {
                1.0
            }
        }
        PointMode::Wave => {
            let n = count(tension);
            0.5 - 0.5 * (x * n * std::f32::consts::TAU).cos()
        }
    }
}

/// An automation curve (FL Studio's automation clip), placed by `ClipSource::Automation` clips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Automation {
    pub id: AutomationId,
    pub name: String,
    pub color: Rgb,
    pub target: AutomationTarget,
    /// Sorted by tick.
    pub points: Vec<AutomationPoint>,
    /// The length new clips of this automation get.
    pub length: Tick,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Automation {
    /// The normalised value at `tick` (holding the first and last point's values outside them).
    pub fn value_at(&self, tick: Tick) -> Option<f32> {
        curve_value(&self.points, tick)
    }

    /// Clamp, sort and de-duplicate points (at most one per tick).
    pub fn normalize_points(points: &mut Vec<AutomationPoint>) {
        for p in points.iter_mut() {
            p.tick = p.tick.min(MAX_TICK);
            p.value = if p.value.is_finite() {
                p.value.clamp(0.0, 1.0)
            } else {
                0.5
            };
            p.tension = if p.tension.is_finite() {
                p.tension.clamp(-1.0, 1.0)
            } else {
                0.0
            };
        }
        points.sort_by_key(|p| p.tick);
        points.dedup_by_key(|p| p.tick);
        points.truncate(MAX_POINTS);
    }
}

/// Points per automation.
pub const MAX_POINTS: usize = 4096;

/// Position 0–1 within a segment shaped by `tension` (−1–1): positive tension rises late
/// (concave), negative rises early (convex), as when dragging FL Studio's tension handle.
pub fn shape(x: f32, tension: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if tension.abs() < 1e-4 {
        return x;
    }
    x.powf(2f32.powf(tension.clamp(-1.0, 1.0) * 3.0))
}

/// The value of a sorted point list at `tick`. Allocation-free (used on the audio thread).
pub fn curve_value(points: &[AutomationPoint], tick: Tick) -> Option<f32> {
    let first = points.first()?;
    if tick <= first.tick {
        return Some(first.value);
    }
    let i = points.partition_point(|p| p.tick <= tick);
    let (Some(a), Some(b)) = (points.get(i.wrapping_sub(1)), points.get(i)) else {
        return points.last().map(|p| p.value);
    };
    let span = (b.tick - a.tick).max(1) as f32;
    let x = (tick - a.tick) as f32 / span;
    Some(a.value + (b.value - a.value) * segment(x, a.tension, a.mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(tick: Tick, value: f32, tension: f32) -> AutomationPoint {
        AutomationPoint {
            tick,
            value,
            tension,
            mode: PointMode::SingleCurve,
        }
    }

    #[test]
    fn point_modes_shape_their_segment() {
        let x = |m, t, at| segment(at, t, m);
        assert_eq!(x(PointMode::Hold, 0.0, 0.9), 0.0);
        // Stairs with tension −1/1: 1 and 16 steps; 0 gives 9 (rounded).
        assert_eq!(x(PointMode::Stairs, -1.0, 0.99), 0.0);
        assert_eq!(x(PointMode::Stairs, 1.0, 0.5), 0.5);
        assert!((x(PointMode::Stairs, 1.0, 0.51) - 0.5).abs() < 1e-6);
        assert!((x(PointMode::DoubleCurve, 0.7, 0.5) - 0.5).abs() < 1e-6);
        assert!(x(PointMode::DoubleCurve, 0.7, 0.25) < 0.25);
        assert_eq!(x(PointMode::Pulse, -1.0, 0.25), 0.0);
        assert_eq!(x(PointMode::Pulse, -1.0, 0.75), 1.0);
        assert!((x(PointMode::Wave, -1.0, 0.5) - 1.0).abs() < 1e-6);
        let s = x(PointMode::SmoothStairs, -1.0, 0.5);
        assert!((s - 0.5).abs() < 1e-6, "{s}");
        // A point's mode shapes the curve after it, and is saved only when set.
        let mut pts = [pt(0, 0.0, 0.0), pt(100, 1.0, 0.0)];
        pts[0].mode = PointMode::Hold;
        assert_eq!(curve_value(&pts, 99), Some(0.0));
        assert!(!serde_json::to_string(&pt(0, 0.0, 0.0)).unwrap().contains("mode"));
        assert!(serde_json::to_string(&pts[0]).unwrap().contains("Hold"));
    }

    #[test]
    fn curve_holds_outside_and_interpolates_inside() {
        let pts = [pt(100, 0.0, 0.0), pt(200, 1.0, 0.5), pt(300, 0.5, 0.0)];
        assert_eq!(curve_value(&pts, 0), Some(0.0));
        assert_eq!(curve_value(&pts, 150), Some(0.5));
        assert_eq!(curve_value(&pts, 200), Some(1.0));
        assert_eq!(curve_value(&pts, 999), Some(0.5));
        assert_eq!(curve_value(&[], 5), None);
        // Positive tension on the falling segment after tick 200: the value changes late.
        let v = curve_value(&pts, 250).unwrap();
        assert!(v > 0.75 && v < 1.0, "{v}");
    }

    #[test]
    fn mappings_round_trip() {
        for m in [
            Mapping::Linear { min: -1.0, max: 1.0 },
            Mapping::Log {
                min: 20.0,
                max: 20_000.0,
            },
            Mapping::Fader { max: 2.0 },
        ] {
            for n in [0.0f32, 0.25, 0.5, 1.0] {
                assert!((m.to_norm(m.to_value(n)) - n).abs() < 1e-4, "{m:?} {n}");
            }
        }
        assert_eq!(Mapping::Fader { max: 2.0 }.to_value(1.0), 2.0);
    }

    #[test]
    fn normalize_sorts_clamps_and_dedups() {
        let max_tick = crate::MAX_TICK;
        let mut v = vec![
            pt(u64::MAX, 0.25, 0.0),
            pt(50, 2.0, 0.0),
            pt(10, f32::NAN, 9.0),
            pt(50, 0.1, 0.0),
            pt(max_tick, 0.75, 0.0),
            pt(max_tick + 1, 0.8, 0.0),
            pt(0, 0.1, 0.0),
        ];
        Automation::normalize_points(&mut v);
        assert_eq!(v.len(), 4);
        assert_eq!((v[0].tick, v[0].value), (0, 0.1));
        assert_eq!((v[1].tick, v[1].value, v[1].tension), (10, 0.5, 1.0));
        assert_eq!((v[2].tick, v[2].value), (50, 1.0));
        assert_eq!((v[3].tick, v[3].value), (max_tick, 0.25));
    }
}
