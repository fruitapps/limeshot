//! FL Studio's Piano Roll tools as pure functions over notes: each takes the notes to work on
//! and returns the notes that replace them, applied with [`crate::Command::EditNotes`] so a tool
//! is one undo step. Ids are kept where a note survives, so the selection follows the notes.

use crate::model::{Note, NoteId};
use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// Which Piano Roll tool an [`crate::Command::EditNotes`] comes from (its undo label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteTool {
    /// Notes recorded from the MIDI input or the typing keyboard (one undo step per take).
    Record,
    Quantize,
    Legato,
    DiscardLengths,
    Articulate,
    Chop,
    Arpeggiate,
    Strum,
    Flam,
    Limit,
    Flip,
    Randomize,
    ScaleLevels,
    Lfo,
}

impl NoteTool {
    pub fn label(self) -> &'static str {
        match self {
            NoteTool::Record => "Record",
            NoteTool::Quantize => "Quantize",
            NoteTool::Legato => "Quick legato",
            NoteTool::DiscardLengths => "Discard lengths",
            NoteTool::Articulate => "Articulate",
            NoteTool::Chop => "Chop",
            NoteTool::Arpeggiate => "Arpeggiate",
            NoteTool::Strum => "Strum",
            NoteTool::Flam => "Flam",
            NoteTool::Limit => "Limit",
            NoteTool::Flip => "Flip",
            NoteTool::Randomize => "Randomize",
            NoteTool::ScaleLevels => "Scale levels",
            NoteTool::Lfo => "LFO",
        }
    }
}

/// A small deterministic random generator (xorshift64*), so a tool's result depends only on its
/// seed and tests can check it.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..n` (0 when `n` is 0).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// Uniform in 0–1.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

fn by_time(notes: &[Note]) -> Vec<Note> {
    let mut v = notes.to_vec();
    v.sort_by_key(|n| (n.start, n.key, n.id.0));
    v
}

/// Quick legato (Cmd+L): every note ends where the next later note starts; notes that start
/// together (a chord) all reach the next start. The last notes keep their lengths.
pub fn legato(notes: &[Note]) -> Vec<Note> {
    let sorted = by_time(notes);
    sorted
        .iter()
        .map(|n| match sorted.iter().map(|m| m.start).find(|&s| s > n.start) {
            Some(next) => Note {
                length: next - n.start,
                ..*n
            },
            None => *n,
        })
        .collect()
}

/// Discard lengths (Shift+D): every note gets the same short length.
pub fn discard_lengths(notes: &[Note], length: Tick) -> Vec<Note> {
    notes
        .iter()
        .map(|n| Note {
            length: length.max(1),
            ..*n
        })
        .collect()
}

/// Select odd (Shift+O): the 1st, 3rd, 5th … note in time order.
pub fn odd(notes: &[Note]) -> Vec<NoteId> {
    by_time(notes).iter().step_by(2).map(|n| n.id).collect()
}

/// Notes that overlap another note of the same key (Select overlapping notes).
pub fn overlapping(notes: &[Note]) -> Vec<NoteId> {
    notes
        .iter()
        .filter(|n| {
            notes
                .iter()
                .any(|m| m.id != n.id && m.key == n.key && m.start < n.end() && n.start < m.end())
        })
        .map(|n| n.id)
        .collect()
}

/// Notes that start together with another note (Select stacked notes: chords).
pub fn stacked(notes: &[Note]) -> Vec<NoteId> {
    notes
        .iter()
        .filter(|n| notes.iter().any(|m| m.id != n.id && m.start == n.start))
        .map(|n| n.id)
        .collect()
}

/// Group notes that start together (chords), in time order; each chord sorted low to high.
fn chords(notes: &[Note]) -> Vec<Vec<Note>> {
    let mut out: Vec<Vec<Note>> = Vec::new();
    for n in by_time(notes) {
        match out.last_mut() {
            Some(c) if c[0].start == n.start => c.push(n),
            _ => out.push(vec![n]),
        }
    }
    for c in &mut out {
        c.sort_by_key(|n| (n.key, n.id.0));
    }
    out
}

/// FL Studio's tension: bends 0–1 towards its ends (`t` −1–1; 0 is linear).
pub fn tension(x: f32, t: f32) -> f32 {
    x.clamp(0.0, 1.0).powf(4f32.powf(t.clamp(-1.0, 1.0)))
}

fn offset(t: Tick, d: f32) -> Tick {
    (t as f32 + d).round().max(0.0) as Tick
}

/// Piano roll – strumizer (Opt+S).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strum {
    /// Start time spread over a chord, as a fraction of a beat (0–1), and its tension.
    pub time: f32,
    pub time_tension: f32,
    /// Velocity fade over the chord (−1–1: the last note's velocity drops by this share).
    pub velocity: f32,
    pub velocity_tension: f32,
    /// End time spread (fraction of a beat) and its tension.
    pub end: f32,
    pub end_tension: f32,
    /// Keep note ends where they were.
    pub preserve_end: bool,
    /// Strum ahead of the beat: the last note keeps its start, the others move earlier.
    pub trigger_ahead: bool,
    /// Cut notes at the next chord's start.
    pub chop_chords: bool,
    /// Every other chord strums downwards.
    pub alternate: bool,
}

impl Default for Strum {
    fn default() -> Self {
        Strum {
            time: 0.25,
            time_tension: 0.0,
            velocity: 0.0,
            velocity_tension: 0.0,
            end: 0.0,
            end_tension: 0.0,
            preserve_end: false,
            trigger_ahead: false,
            chop_chords: false,
            alternate: false,
        }
    }
}

pub fn strum(notes: &[Note], s: &Strum, ppq: Tick) -> Vec<Note> {
    let groups = chords(notes);
    let starts: Vec<Tick> = groups.iter().map(|c| c[0].start).collect();
    let mut out = Vec::with_capacity(notes.len());
    for (ci, chord) in groups.iter().enumerate() {
        let n = chord.len();
        let next = starts.get(ci + 1).copied();
        for (i, note) in chord.iter().enumerate() {
            let order = if s.alternate && ci % 2 == 1 { n - 1 - i } else { i };
            let x = if n > 1 { order as f32 / (n - 1) as f32 } else { 0.0 };
            let mut dt = s.time * ppq as f32 * tension(x, s.time_tension);
            if s.trigger_ahead {
                dt -= s.time * ppq as f32;
            }
            let end = note.end();
            let start = offset(note.start, dt);
            let mut end = if s.preserve_end { end } else { offset(end, dt) };
            end = offset(end, s.end * ppq as f32 * tension(x, s.end_tension));
            if let Some(nx) = next.filter(|_| s.chop_chords) {
                end = end.min(nx.max(start + 1));
            }
            let v = note.velocity * (1.0 - s.velocity * tension(x, s.velocity_tension));
            out.push(Note {
                start,
                length: end.saturating_sub(start).max(1),
                velocity: v.clamp(0.0, 1.0),
                ..*note
            });
        }
    }
    out
}

/// Arpeggiator directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpPattern {
    #[default]
    Up,
    Down,
    UpDown,
    DownUp,
    Random,
}

impl ArpPattern {
    pub const ALL: [ArpPattern; 5] = [
        ArpPattern::Up,
        ArpPattern::Down,
        ArpPattern::UpDown,
        ArpPattern::DownUp,
        ArpPattern::Random,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ArpPattern::Up => "Up",
            ArpPattern::Down => "Down",
            ArpPattern::UpDown => "Up / down",
            ArpPattern::DownUp => "Down / up",
            ArpPattern::Random => "Random",
        }
    }
}

/// Piano roll – arpeggiator (Opt+A).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arp {
    pub pattern: ArpPattern,
    /// Octaves the arpeggio climbs (1–4).
    pub range: u8,
    /// Step length in ticks (FL's Time, with Time mul applied).
    pub time: Tick,
    /// Note length as a share of the step (0.05–1).
    pub gate: f32,
    /// Velocity change per step (−1–1 over the whole arpeggio).
    pub velocity: f32,
    pub seed: u64,
}

impl Default for Arp {
    fn default() -> Self {
        Arp {
            pattern: ArpPattern::Up,
            range: 1,
            time: crate::time::STEP_TICKS,
            gate: 0.5,
            velocity: 0.0,
            seed: 1,
        }
    }
}

/// Each chord plays its keys one after another, every `time` ticks, for as long as the chord lasts.
pub fn arpeggiate(notes: &[Note], a: &Arp) -> Vec<Note> {
    let time = a.time.max(1);
    let mut rng = Rng::new(a.seed);
    let mut out = Vec::new();
    for chord in chords(notes) {
        let base = chord[0];
        let end = chord.iter().map(Note::end).max().unwrap_or(base.end());
        let mut keys: Vec<(u8, f32)> = Vec::new();
        for o in 0..a.range.clamp(1, 4) {
            for n in &chord {
                let k = u16::from(n.key) + 12 * u16::from(o);
                if k <= 127 {
                    keys.push((k as u8, n.velocity));
                }
            }
        }
        let cycle: Vec<(u8, f32)> = match a.pattern {
            ArpPattern::Up | ArpPattern::Random => keys.clone(),
            ArpPattern::Down => keys.iter().rev().copied().collect(),
            ArpPattern::UpDown => {
                let mut v = keys.clone();
                v.extend(keys.iter().rev().skip(1).take(keys.len().saturating_sub(2)));
                v
            }
            ArpPattern::DownUp => {
                let mut v: Vec<(u8, f32)> = keys.iter().rev().copied().collect();
                v.extend(keys.iter().skip(1).take(keys.len().saturating_sub(2)));
                v
            }
        };
        if cycle.is_empty() {
            continue;
        }
        let steps = (end - base.start).div_ceil(time).max(1);
        for k in 0..steps {
            let (key, vel) = if a.pattern == ArpPattern::Random {
                cycle[rng.below(cycle.len())]
            } else {
                cycle[k as usize % cycle.len()]
            };
            let start = base.start + k * time;
            let x = if steps > 1 { k as f32 / (steps - 1) as f32 } else { 0.0 };
            out.push(Note {
                key,
                start,
                length: ((time as f32 * a.gate.clamp(0.05, 1.0)).round() as Tick).clamp(1, end - start),
                velocity: (vel * (1.0 + a.velocity * x)).clamp(0.0, 1.0),
                ..base
            });
        }
    }
    out
}

/// Piano roll – flamer (Opt+F).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flam {
    /// Ticks between the grace note and the note.
    pub time: Tick,
    /// The grace note goes before the note; otherwise the note moves later by `time`.
    pub before: bool,
    /// The grace note's velocity as a share of the note's.
    pub velocity: f32,
}

impl Default for Flam {
    fn default() -> Self {
        Flam {
            time: crate::time::STEP_TICKS / 2,
            before: false,
            velocity: 0.6,
        }
    }
}

/// Every note gets a softer grace note `time` ticks ahead of it.
pub fn flam(notes: &[Note], f: &Flam) -> Vec<Note> {
    let time = f.time.max(1);
    let mut out = Vec::with_capacity(notes.len() * 2);
    for n in by_time(notes) {
        let (grace, main) = if f.before {
            (n.start.saturating_sub(time), n)
        } else {
            (
                n.start,
                Note {
                    start: n.start + time,
                    length: n.length.saturating_sub(time).max(1),
                    ..n
                },
            )
        };
        out.push(main);
        out.push(Note {
            id: NoteId(0),
            start: grace,
            length: time.min(n.length).max(1),
            velocity: (n.velocity * f.velocity).clamp(0.0, 1.0),
            ..n
        });
    }
    out
}

/// Scales for the limiter and the randomizer: semitones from the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scale {
    #[default]
    None,
    Major,
    Minor,
    HarmonicMinor,
    MajorPentatonic,
    MinorPentatonic,
    Blues,
    Dorian,
}

impl Scale {
    pub const ALL: [Scale; 8] = [
        Scale::None,
        Scale::Major,
        Scale::Minor,
        Scale::HarmonicMinor,
        Scale::MajorPentatonic,
        Scale::MinorPentatonic,
        Scale::Blues,
        Scale::Dorian,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Scale::None => "(none)",
            Scale::Major => "Major",
            Scale::Minor => "Minor",
            Scale::HarmonicMinor => "Harmonic minor",
            Scale::MajorPentatonic => "Major pentatonic",
            Scale::MinorPentatonic => "Minor pentatonic",
            Scale::Blues => "Blues",
            Scale::Dorian => "Dorian",
        }
    }

    /// Semitones from the root.
    pub fn steps(self) -> &'static [u8] {
        match self {
            Scale::None => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Scale::MajorPentatonic => &[0, 2, 4, 7, 9],
            Scale::MinorPentatonic => &[0, 3, 5, 7, 10],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
        }
    }

    /// Is `key` in the scale on `root` (0 = C)?
    pub fn contains(self, root: u8, key: u8) -> bool {
        self.steps().contains(&((key + 12 - root % 12) % 12))
    }
}

/// FL Studio's stamps: chords (semitones above the clicked key) …
pub const CHORDS: &[(&str, &[u8])] = &[
    ("Major", &[0, 4, 7]),
    ("Minor", &[0, 3, 7]),
    ("Diminished", &[0, 3, 6]),
    ("Augmented", &[0, 4, 8]),
    ("Sus2", &[0, 2, 7]),
    ("Sus4", &[0, 5, 7]),
    ("Power (5)", &[0, 7, 12]),
    ("Major 6", &[0, 4, 7, 9]),
    ("Minor 6", &[0, 3, 7, 9]),
    ("Dominant 7", &[0, 4, 7, 10]),
    ("Major 7", &[0, 4, 7, 11]),
    ("Minor 7", &[0, 3, 7, 10]),
    ("Minor 7 flat 5", &[0, 3, 6, 10]),
    ("Diminished 7", &[0, 3, 6, 9]),
    ("Dominant 9", &[0, 4, 7, 10, 14]),
    ("Major 9", &[0, 4, 7, 11, 14]),
    ("Minor 9", &[0, 3, 7, 10, 14]),
    ("Add 9", &[0, 4, 7, 14]),
];

/// … and scales, one octave of them stacked from the clicked key.
pub fn stamp_intervals(stamp: Stamp) -> Vec<u8> {
    match stamp {
        Stamp::Chord(i) => CHORDS.get(i).map(|c| c.1.to_vec()).unwrap_or_default(),
        Stamp::Scale(s) => s.steps().to_vec(),
    }
}

/// A Piano Roll stamp (FL Studio's Stamp tool).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stamp {
    /// An index into [`CHORDS`].
    Chord(usize),
    Scale(Scale),
}

impl Stamp {
    pub fn label(self) -> &'static str {
        match self {
            Stamp::Chord(i) => CHORDS.get(i).map_or("?", |c| c.0),
            Stamp::Scale(s) => s.label(),
        }
    }
}

/// Where the limiter moves a key that is not in the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapDir {
    #[default]
    Above,
    Below,
    Nearest,
}

/// Piano roll – limiter / transposer (Opt+K).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limit {
    pub root: u8,
    pub scale: Scale,
    pub snap: SnapDir,
    /// Keyboard range, inclusive.
    pub low: u8,
    pub high: u8,
    /// Out-of-range notes move by octaves into the range (otherwise they stop at its edge).
    pub wrap: bool,
}

impl Limit {
    /// The notes' own range, as FL Studio opens the dialog.
    pub fn around(notes: &[Note]) -> Self {
        Limit {
            root: 0,
            scale: Scale::None,
            snap: SnapDir::Above,
            low: notes.iter().map(|n| n.key).min().unwrap_or(48),
            high: notes.iter().map(|n| n.key).max().unwrap_or(72),
            wrap: false,
        }
    }
}

fn snap_to_scale(key: u8, l: &Limit) -> u8 {
    if l.scale.contains(l.root, key) {
        return key;
    }
    let up = (key..=127).find(|k| l.scale.contains(l.root, *k));
    let down = (0..=key).rev().find(|k| l.scale.contains(l.root, *k));
    match (l.snap, up, down) {
        (SnapDir::Above, Some(u), _) | (SnapDir::Below, None, Some(u)) => u,
        (SnapDir::Below, _, Some(d)) | (SnapDir::Above, None, Some(d)) => d,
        (SnapDir::Nearest, Some(u), Some(d)) => {
            if u - key <= key - d {
                u
            } else {
                d
            }
        }
        (_, Some(k), None) | (_, None, Some(k)) => k,
        (_, None, None) => key,
    }
}

/// Keep keys inside the range (wrapping by octaves or clamping) and on the scale.
pub fn limit(notes: &[Note], l: &Limit) -> Vec<Note> {
    let (low, high) = (l.low.min(l.high), l.high.max(l.low));
    notes
        .iter()
        .map(|n| {
            let mut k = n.key;
            if l.wrap && high - low >= 11 {
                while k < low {
                    k += 12;
                }
                while k > high {
                    k -= 12;
                }
            } else {
                k = k.clamp(low, high);
            }
            Note {
                key: snap_to_scale(k, l).min(127),
                ..*n
            }
        })
        .collect()
}

/// Piano roll – flip (Opt+Y): mirror the notes in time and/or pitch around their own span.
pub fn flip(notes: &[Note], horizontal: bool, vertical: bool) -> Vec<Note> {
    let (Some(t0), Some(t1)) = (notes.iter().map(|n| n.start).min(), notes.iter().map(Note::end).max()) else {
        return Vec::new();
    };
    let (k0, k1) = (
        notes.iter().map(|n| n.key).min().unwrap_or(0),
        notes.iter().map(|n| n.key).max().unwrap_or(0),
    );
    notes
        .iter()
        .map(|n| Note {
            start: if horizontal { t0 + (t1 - n.end()) } else { n.start },
            key: if vertical { k0 + (k1 - n.key) } else { n.key },
            ..*n
        })
        .collect()
}

/// Piano roll – randomizer (Opt+R): new random notes over a time span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Randomize {
    /// Lowest octave (FL's octave numbers: C5 is key 60) and how many octaves up.
    pub octave: u8,
    pub range: u8,
    pub root: u8,
    pub scale: Scale,
    /// Note length in grid steps, and how much it varies (0–1).
    pub length: u32,
    pub variation: f32,
    /// Chance of a note at each step (0–1).
    pub population: f32,
    /// Notes per position.
    pub stack: u8,
    /// Merge a note into the previous one of the same key when they touch.
    pub glue: bool,
    /// Velocity spread around the base velocity (0–1) and pan spread (0–1).
    pub velocity: f32,
    pub pan: f32,
    pub seed: u64,
}

impl Default for Randomize {
    fn default() -> Self {
        Randomize {
            octave: 4,
            range: 3,
            root: 0,
            scale: Scale::None,
            length: 2,
            variation: 0.5,
            population: 0.5,
            stack: 1,
            glue: true,
            velocity: 0.0,
            pan: 0.0,
            seed: 1,
        }
    }
}

/// Random notes for `template` (its channel and velocity) from `from` to `to`, on a `grid`.
pub fn randomize(template: Note, from: Tick, to: Tick, grid: Tick, r: &Randomize) -> Vec<Note> {
    let grid = grid.max(1);
    let mut rng = Rng::new(r.seed);
    let lo = u16::from(r.octave) * 12;
    let keys: Vec<u8> = (lo..(lo + 12 * u16::from(r.range.max(1))).min(128))
        .map(|k| k as u8)
        .filter(|k| r.scale.contains(r.root, *k))
        .collect();
    let mut out: Vec<Note> = Vec::new();
    let mut t = from;
    while t < to && !keys.is_empty() {
        let len_steps = {
            let v = r.length.max(1) as f32 * (1.0 + r.variation * (rng.unit() * 2.0 - 1.0));
            (v.round() as Tick).max(1)
        };
        if rng.unit() < r.population {
            for _ in 0..r.stack.max(1) {
                let key = keys[rng.below(keys.len())];
                let length = (len_steps * grid).min(to - t).max(1);
                let velocity = (template.velocity + r.velocity * (rng.unit() - 0.5)).clamp(0.0, 1.0);
                let pan = (r.pan * (rng.unit() * 2.0 - 1.0)).clamp(-1.0, 1.0);
                let glued = r.glue
                    && out
                        .iter_mut()
                        .rev()
                        .find(|n| n.key == key && n.end() == t)
                        .map(|n| n.length += length)
                        .is_some();
                if !glued && !out.iter().any(|n| n.key == key && n.start == t) {
                    out.push(Note {
                        id: NoteId(0),
                        key,
                        start: t,
                        length,
                        velocity,
                        pan,
                        muted: false,
                        ..template
                    });
                }
            }
        }
        t += len_steps * grid;
    }
    out
}

/// Which note value a level tool changes (FL Studio's target control).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    #[default]
    Velocity,
    Pan,
}

fn get(n: &Note, l: Level) -> f32 {
    match l {
        Level::Velocity => n.velocity,
        Level::Pan => (n.pan + 1.0) / 2.0,
    }
}

fn set(n: &Note, l: Level, v: f32) -> Note {
    let v = v.clamp(0.0, 1.0);
    match l {
        Level::Velocity => Note { velocity: v, ..*n },
        Level::Pan => Note {
            pan: v * 2.0 - 1.0,
            ..*n
        },
    }
}

/// Piano roll – level scaling (Opt+X): values (0–1) are bent by `tension`, scaled by `multiply`
/// around `center`, then moved by `offset`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleLevels {
    pub level: Level,
    pub center: f32,
    pub multiply: f32,
    pub offset: f32,
    pub tension: f32,
}

impl Default for ScaleLevels {
    fn default() -> Self {
        ScaleLevels {
            level: Level::Velocity,
            center: 0.5,
            multiply: 1.0,
            offset: 0.0,
            tension: 0.0,
        }
    }
}

pub fn scale_levels(notes: &[Note], s: &ScaleLevels) -> Vec<Note> {
    notes
        .iter()
        .map(|n| {
            let v = tension(get(n, s.level), s.tension);
            set(n, s.level, s.center + (v - s.center) * s.multiply + s.offset)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Sine,
    Triangle,
    Square,
    Saw,
}

impl Shape {
    pub const ALL: [Shape; 4] = [Shape::Sine, Shape::Triangle, Shape::Square, Shape::Saw];

    pub fn label(self) -> &'static str {
        match self {
            Shape::Sine => "Sine",
            Shape::Triangle => "Triangle",
            Shape::Square => "Square",
            Shape::Saw => "Saw",
        }
    }

    /// −1–1 at phase `p` (cycles).
    fn at(self, p: f32) -> f32 {
        let f = p.rem_euclid(1.0);
        match self {
            Shape::Sine => (f * std::f32::consts::TAU).sin(),
            Shape::Triangle => 1.0 - 4.0 * (f - 0.25).rem_euclid(1.0).min(1.0 - (f - 0.25).rem_euclid(1.0)),
            Shape::Square => {
                if f < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Shape::Saw => 2.0 * f - 1.0,
        }
    }
}

/// Piano roll – LFO (Opt+O): sets a value from an oscillator at each note's start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lfo {
    pub level: Level,
    pub shape: Shape,
    /// Cycle length in ticks.
    pub period: Tick,
    /// 0–1 around `center`.
    pub amount: f32,
    pub center: f32,
    /// Start phase, 0–1.
    pub phase: f32,
}

impl Default for Lfo {
    fn default() -> Self {
        Lfo {
            level: Level::Velocity,
            shape: Shape::Sine,
            period: crate::time::PPQ * 4,
            amount: 0.5,
            center: 0.6,
            phase: 0.0,
        }
    }
}

pub fn lfo(notes: &[Note], l: &Lfo) -> Vec<Note> {
    let t0 = notes.iter().map(|n| n.start).min().unwrap_or(0);
    notes
        .iter()
        .map(|n| {
            let p = l.phase + (n.start - t0) as f32 / l.period.max(1) as f32;
            set(n, l.level, l.center + l.amount / 2.0 * l.shape.at(p))
        })
        .collect()
}

/// Piano roll – articulator (Opt+L): note lengths as a share of the gap to the next note, or a
/// fixed length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Articulate {
    /// Share of the distance to the next start (0.05–1; 1 is legato).
    pub amount: f32,
    /// A fixed length in ticks instead (`None`: relative).
    pub fixed: Option<Tick>,
}

impl Default for Articulate {
    fn default() -> Self {
        Articulate {
            amount: 0.5,
            fixed: None,
        }
    }
}

pub fn articulate(notes: &[Note], a: &Articulate) -> Vec<Note> {
    if let Some(len) = a.fixed {
        return discard_lengths(notes, len);
    }
    legato(notes)
        .into_iter()
        .zip(by_time(notes))
        .map(|(full, orig)| {
            let has_next = notes.iter().any(|m| m.start > orig.start);
            let length = if has_next {
                ((full.length as f32 * a.amount.clamp(0.05, 1.0)).round() as Tick).max(1)
            } else {
                orig.length
            };
            Note { length, ..full }
        })
        .collect()
}

/// Piano roll – chopper (Opt+U): cut notes into pieces of `time` ticks, each `gate` (0.05–1) long.
pub fn chop(notes: &[Note], time: Tick, gate: f32) -> Vec<Note> {
    let time = time.max(1);
    let mut out = Vec::new();
    for n in notes {
        let mut t = n.start;
        let mut first = true;
        while t < n.end() {
            let piece = time.min(n.end() - t);
            out.push(Note {
                id: if first { n.id } else { NoteId(0) },
                start: t,
                length: ((piece as f32 * gate.clamp(0.05, 1.0)).round() as Tick).max(1),
                ..*n
            });
            first = false;
            t += time;
        }
    }
    out
}

/// Piano roll – quantizer (Opt+Q): starts move towards the grid by `start` (0–1); lengths round
/// to the grid by `length` (0–1).
pub fn quantize(notes: &[Note], grid: Tick, start: f32, length: f32) -> Vec<Note> {
    let grid = grid.max(1);
    let near = |t: Tick| (t + grid / 2) / grid * grid;
    let toward =
        |a: Tick, b: Tick, k: f32| (a as f32 + (b as f32 - a as f32) * k.clamp(0.0, 1.0)).round().max(0.0) as Tick;
    notes
        .iter()
        .map(|n| Note {
            start: toward(n.start, near(n.start), start),
            length: toward(n.length, near(n.length).max(grid), length).max(1),
            ..*n
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChannelId;

    pub(crate) fn note(id: u32, key: u8, start: Tick, length: Tick) -> Note {
        Note {
            id: NoteId(id),
            channel: ChannelId(1),
            key,
            start,
            length,
            velocity: 0.8,
            muted: false,
            pan: 0.0,
        }
    }

    #[test]
    fn legato_reaches_the_next_start_and_keeps_chords_together() {
        let notes = [
            note(1, 60, 0, 10),
            note(2, 64, 0, 20),
            note(3, 67, 100, 5),
            note(4, 72, 300, 7),
        ];
        let out = legato(&notes);
        let len = |id| out.iter().find(|n| n.id == NoteId(id)).map(|n| n.length);
        assert_eq!(len(1), Some(100));
        assert_eq!(len(2), Some(100));
        assert_eq!(len(3), Some(200));
        assert_eq!(len(4), Some(7), "the last note keeps its length");
    }

    #[test]
    fn selections_by_order_overlap_and_stack() {
        let notes = [
            note(1, 60, 0, 50),
            note(2, 60, 40, 50),
            note(3, 64, 0, 10),
            note(4, 67, 200, 10),
        ];
        assert_eq!(odd(&notes), vec![NoteId(1), NoteId(2)]);
        assert_eq!(overlapping(&notes), vec![NoteId(1), NoteId(2)]);
        assert_eq!(stacked(&notes), vec![NoteId(1), NoteId(3)]);
        assert!(discard_lengths(&notes, 30).iter().all(|n| n.length == 30));
    }

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            let (x, y) = (a.below(5), b.below(5));
            assert_eq!(x, y);
            assert!(x < 5);
            let u = a.unit();
            b.unit();
            assert!((0.0..=1.0).contains(&u));
        }
    }

    fn chord() -> Vec<Note> {
        vec![
            note(1, 60, 0, 960),
            note(2, 64, 0, 960),
            note(3, 67, 0, 960),
            note(4, 62, 1920, 960),
        ]
    }

    #[test]
    fn strum_spreads_a_chord_upwards_and_alternates() {
        let s = Strum {
            time: 0.5,
            alternate: true,
            preserve_end: true,
            ..Strum::default()
        };
        let out = strum(&chord(), &s, 960);
        let at = |id| out.iter().find(|n| n.id == NoteId(id)).copied().unwrap();
        assert_eq!((at(1).start, at(2).start, at(3).start), (0, 240, 480));
        assert_eq!(at(3).end(), 960, "Preserve end");
        let ahead = strum(
            &chord(),
            &Strum {
                time: 0.5,
                trigger_ahead: true,
                ..Strum::default()
            },
            960,
        );
        assert_eq!(
            ahead.iter().find(|n| n.id == NoteId(3)).unwrap().start,
            0,
            "the last note stays on the beat"
        );
        assert_eq!(
            ahead.iter().find(|n| n.id == NoteId(1)).unwrap().start,
            0,
            "clamped at zero"
        );
    }

    #[test]
    fn arpeggiate_walks_the_chord_for_its_length() {
        let a = Arp {
            time: 240,
            gate: 0.5,
            ..Arp::default()
        };
        let out = arpeggiate(&chord()[..3], &a);
        let keys: Vec<(u8, Tick, Tick)> = out.iter().map(|n| (n.key, n.start, n.length)).collect();
        assert_eq!(keys, vec![(60, 0, 120), (64, 240, 120), (67, 480, 120), (60, 720, 120)]);
        let ud = arpeggiate(
            &chord()[..3],
            &Arp {
                pattern: ArpPattern::UpDown,
                range: 2,
                time: 60,
                ..a
            },
        );
        let k: Vec<u8> = ud.iter().take(11).map(|n| n.key).collect();
        assert_eq!(k, vec![60, 64, 67, 72, 76, 79, 76, 72, 67, 64, 60]);
    }

    #[test]
    fn flam_limit_flip_and_levels() {
        let f = flam(
            &[note(1, 60, 480, 240)],
            &Flam {
                time: 60,
                before: true,
                velocity: 0.5,
            },
        );
        assert_eq!(f.len(), 2);
        assert!(
            f.iter()
                .any(|n| n.start == 420 && n.length == 60 && (n.velocity - 0.4).abs() < 1e-6)
        );
        let l = limit(
            &[note(1, 40, 0, 1), note(2, 90, 0, 1), note(3, 61, 0, 1)],
            &Limit {
                root: 0,
                scale: Scale::Major,
                snap: SnapDir::Above,
                low: 60,
                high: 72,
                wrap: true,
            },
        );
        assert_eq!(l.iter().map(|n| n.key).collect::<Vec<_>>(), vec![64, 67, 62]);
        let fl = flip(&[note(1, 60, 0, 100), note(2, 72, 300, 100)], true, true);
        assert_eq!((fl[0].start, fl[0].key, fl[1].start, fl[1].key), (300, 72, 0, 60));
        let s = scale_levels(
            &[note(1, 60, 0, 1)],
            &ScaleLevels {
                multiply: 0.5,
                offset: 0.1,
                ..ScaleLevels::default()
            },
        );
        assert!((s[0].velocity - 0.75).abs() < 1e-6);
        let sq = lfo(
            &[note(1, 60, 0, 1), note(2, 60, 480, 1)],
            &Lfo {
                shape: Shape::Square,
                period: 960,
                amount: 0.4,
                center: 0.5,
                ..Lfo::default()
            },
        );
        assert!((sq[0].velocity - 0.7).abs() < 1e-6 && (sq[1].velocity - 0.3).abs() < 1e-6);
    }

    #[test]
    fn randomize_is_seeded_and_stays_on_the_scale() {
        let r = Randomize {
            scale: Scale::Major,
            population: 1.0,
            ..Randomize::default()
        };
        let a = randomize(note(0, 0, 0, 1), 0, 3840, 240, &r);
        assert_eq!(a, randomize(note(0, 0, 0, 1), 0, 3840, 240, &r));
        assert!(!a.is_empty());
        assert!(
            a.iter()
                .all(|n| Scale::Major.contains(0, n.key) && (48..84).contains(&n.key) && n.end() <= 3840)
        );
        assert_ne!(
            a,
            randomize(note(0, 0, 0, 1), 0, 3840, 240, &Randomize { seed: 2, ..r })
        );
    }

    #[test]
    fn stamps_and_scales() {
        assert_eq!(stamp_intervals(Stamp::Chord(0)), vec![0, 4, 7]);
        assert_eq!(stamp_intervals(Stamp::Scale(Scale::Minor)).len(), 7);
        assert_eq!(Stamp::Chord(10).label(), "Major 7");
        assert!(
            Scale::Major.contains(2, 66) && !Scale::Major.contains(2, 65),
            "F# is in D major, F is not"
        );
        assert!(CHORDS.iter().all(|c| c.1[0] == 0));
    }

    #[test]
    fn articulate_chop_and_quantize() {
        let notes = [note(1, 60, 0, 10), note(2, 62, 400, 10)];
        let a = articulate(
            &notes,
            &Articulate {
                amount: 0.5,
                fixed: None,
            },
        );
        assert_eq!(a.iter().find(|n| n.id == NoteId(1)).unwrap().length, 200);
        let c = chop(&[note(1, 60, 0, 1000)], 240, 0.5);
        assert_eq!(
            c.iter().map(|n| (n.start, n.length)).collect::<Vec<_>>(),
            vec![(0, 120), (240, 120), (480, 120), (720, 120), (960, 20)]
        );
        assert_eq!(c[0].id, NoteId(1));
        let q = quantize(&[note(1, 60, 130, 230)], 240, 1.0, 0.5);
        assert_eq!((q[0].start, q[0].length), (240, 235));
    }
}
