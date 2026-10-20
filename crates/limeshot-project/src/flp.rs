//! Import of FL Studio projects (`.flp`).
//!
//! Clean-room: the layout below was worked out from the public outline of the format and by
//! inspecting the bytes of project files (FL's own templates and demo songs, read-only, and files
//! made for the tests); no FL Studio code was examined. Details and confidence levels:
//! `docs/research/flp-format.md`.
//!
//! An `.flp` file is a `FLhd` chunk (format, channel count, pulses per quarter note) and a `FLdt`
//! chunk holding a flat stream of events. An event is an id byte whose range gives the payload:
//! 0–63 one byte, 64–127 two bytes, 128–191 four bytes (little-endian), 192–255 a variable-length
//! size followed by that many bytes. Context comes from order: settings that follow a "new
//! channel" event belong to that channel, notes that follow a "pattern" event to that pattern.
//!
//! What is converted: tempo, time signature, title, channels (samplers with their sample, other
//! generators as LimeSynth channels), volume, pan, mixer insert, patterns with their notes, and
//! the first arrangement's pattern and audio clips. Everything else (plugin states, mixer effects,
//! automation clips, layers' children) is skipped and counted in the [`ImportReport`].

use limeshot_core::{
    Channel, ChannelId, Clip, ClipId, ClipSource, Extra, Instrument, Mixer, MixerTrack, Note, NoteId, PALETTE, Pattern,
    PatternId, Playlist, PlaylistTrack, Project, STEP_TICKS, SampleRef, SamplerParams, SynthParams, Tempo,
    TimeSignature,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Larger files are refused.
pub const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_EVENTS: usize = 20_000_000;
const MAX_CHANNELS: usize = 2_000;
const MAX_NOTES: usize = 2_000_000;
/// FL's playlist has 500 tracks, stored as `499 − track`.
const TRACKS: u16 = 500;
/// Playlist items name their source as `PATTERN_BASE + pattern` or as a channel index.
const PATTERN_BASE: u16 = 0x5000;

pub mod id {
    //! Event ids used by the importer (OBSERVED in project files unless noted).
    pub const TIME_SIG_NUM: u8 = 17;
    pub const TIME_SIG_DEN: u8 = 18;
    pub const CHANNEL_TYPE: u8 = 21;
    pub const CHANNEL_MIXER_TRACK: u8 = 22;
    pub const NEW_CHANNEL: u8 = 64;
    pub const NEW_PATTERN: u8 = 65;
    pub const NEW_ARRANGEMENT: u8 = 99;
    pub const CHANNEL_COLOR: u8 = 128;
    pub const TEMPO: u8 = 156;
    pub const PATTERN_NAME: u8 = 193;
    pub const TITLE: u8 = 194;
    pub const SAMPLE_PATH: u8 = 196;
    pub const VERSION: u8 = 199;
    pub const CHANNEL_NAME: u8 = 203;
    pub const CHANNEL_LEVELS: u8 = 219;
    pub const PATTERN_NOTES: u8 = 224;
    pub const PLAYLIST_ITEMS: u8 = 233;
}

#[derive(Debug, thiserror::Error)]
pub enum FlpError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("not an FL Studio project: {0}")]
    Format(String),
}

fn bad(s: &str) -> FlpError {
    FlpError::Format(s.to_string())
}

/// One event's payload.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'a> {
    Byte(u8),
    Word(u16),
    Dword(u32),
    Data(&'a [u8]),
}

impl Value<'_> {
    pub fn int(&self) -> Option<u32> {
        match *self {
            Value::Byte(v) => Some(u32::from(v)),
            Value::Word(v) => Some(u32::from(v)),
            Value::Dword(v) => Some(v),
            Value::Data(_) => None,
        }
    }

    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Value::Data(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub format: u16,
    pub channels: u16,
    pub ppq: u16,
}

fn le16(d: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(at..at + 2)?.try_into().ok()?))
}

fn le32(d: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(at..at + 4)?.try_into().ok()?))
}

/// Split a file into its header and event stream.
pub fn read_events(data: &[u8]) -> Result<(Header, Vec<(u8, Value<'_>)>), FlpError> {
    if data.get(..4) != Some(b"FLhd") {
        return Err(bad("missing FLhd header"));
    }
    let hlen = le32(data, 4).ok_or_else(|| bad("short header"))? as usize;
    let hdr = data
        .get(8..8usize.saturating_add(hlen))
        .ok_or_else(|| bad("short header"))?;
    let header = Header {
        format: le16(hdr, 0).ok_or_else(|| bad("short header"))?,
        channels: le16(hdr, 2).ok_or_else(|| bad("short header"))?,
        ppq: le16(hdr, 4).ok_or_else(|| bad("short header"))?,
    };
    if header.ppq == 0 {
        return Err(bad("zero time base"));
    }
    let p = 8 + hlen;
    if data.get(p..p + 4) != Some(b"FLdt") {
        return Err(bad("missing FLdt chunk"));
    }
    let dlen = le32(data, p + 4).ok_or_else(|| bad("short data chunk"))? as usize;
    // A truncated file keeps the events that are complete.
    let body = data.get(p + 8..).unwrap_or(&[]);
    let body = &body[..dlen.min(body.len())];
    let mut events = Vec::new();
    let mut i = 0;
    while i < body.len() && events.len() < MAX_EVENTS {
        let id = body[i];
        i += 1;
        let v = match id {
            0..=63 => {
                let Some(&b) = body.get(i) else { break };
                i += 1;
                Value::Byte(b)
            }
            64..=127 => {
                let Some(w) = le16(body, i) else { break };
                i += 2;
                Value::Word(w)
            }
            128..=191 => {
                let Some(d) = le32(body, i) else { break };
                i += 4;
                Value::Dword(d)
            }
            192..=255 => {
                let mut n = 0usize;
                let mut shift = 0;
                loop {
                    let Some(&b) = body.get(i) else {
                        return Ok((header, events));
                    };
                    i += 1;
                    n |= usize::from(b & 0x7F) << shift;
                    shift += 7;
                    if b & 0x80 == 0 || shift > 28 {
                        break;
                    }
                }
                let Some(d) = body.get(i..i.saturating_add(n)) else {
                    break;
                };
                i += n;
                Value::Data(d)
            }
        };
        events.push((id, v));
    }
    Ok((header, events))
}

/// A channel of the Channel Rack.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlpChannel {
    /// 0 sampler, 2 plugin generator, 3 layer, 4 audio-clip channel, 5 automation clip.
    pub kind: u8,
    pub name: String,
    pub sample: Option<String>,
    /// 0–1.25 (FL's 0–12800 scale, 10000 = default).
    pub volume: Option<f32>,
    /// −1–1.
    pub pan: Option<f32>,
    pub mixer_track: Option<u32>,
    /// Packed as `0x00BBGGRR`.
    pub color: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlpNote {
    /// In file ticks.
    pub position: u32,
    pub length: u32,
    pub channel: u16,
    pub key: u16,
    /// 0–128 (100 default).
    pub velocity: u8,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlpPattern {
    pub name: Option<String>,
    pub notes: Vec<FlpNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSource {
    Pattern(u16),
    Channel(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlpItem {
    pub position: u32,
    pub length: u32,
    pub track: u16,
    pub source: ItemSource,
    /// Start offset into the pattern (file ticks), for pattern items.
    pub offset: Option<u32>,
    pub muted: bool,
}

/// The parts of an `.flp` file LimeShot understands.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlpFile {
    pub ppq: u16,
    pub version: Option<String>,
    pub title: Option<String>,
    pub tempo: Option<f64>,
    pub time_signature: Option<(u8, u8)>,
    pub channels: Vec<FlpChannel>,
    pub patterns: BTreeMap<u16, FlpPattern>,
    pub items: Vec<FlpItem>,
    /// Item records whose layout was not recognised (whole events skipped).
    pub unreadable_item_blocks: usize,
}

fn utf16(d: &[u8]) -> String {
    let units: Vec<u16> = d.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    String::from_utf16_lossy(&units)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

/// Text events are UTF-16 in current files and 8-bit in old ones.
fn text(d: &[u8]) -> String {
    if d.len() >= 2 && d.len().is_multiple_of(2) && d[1] == 0 {
        utf16(d)
    } else {
        String::from_utf8_lossy(d).trim_end_matches('\0').trim().to_string()
    }
}

/// Playlist item records grew over FL versions; find the size whose every record carries the
/// pattern base marker in a pattern item or a small channel index otherwise.
fn item_size(d: &[u8]) -> Option<usize> {
    [240usize, 120, 60, 32].into_iter().find(|&s| {
        !d.is_empty()
            && d.len().is_multiple_of(s)
            && d.chunks_exact(s)
                .all(|r| le16(r, 4).is_some_and(|b| b == PATTERN_BASE || b == 0))
            && d.chunks_exact(s).any(|r| le16(r, 4) == Some(PATTERN_BASE))
    })
}

/// Decode the events into an [`FlpFile`].
pub fn parse(data: &[u8]) -> Result<FlpFile, FlpError> {
    let (header, events) = read_events(data)?;
    let mut f = FlpFile {
        ppq: header.ppq,
        ..Default::default()
    };
    let mut channel: Option<usize> = None;
    let mut pattern: Option<u16> = None;
    let mut arrangement_seen = false;
    let mut items_done = false;
    let mut notes = 0usize;
    for (id, v) in events {
        match id {
            id::VERSION => {
                f.version = v
                    .data()
                    .map(|d| String::from_utf8_lossy(d).trim_end_matches('\0').to_string())
            }
            id::TITLE => f.title = v.data().map(text).filter(|s| !s.is_empty()),
            id::TEMPO => f.tempo = v.int().map(|t| f64::from(t) / 1000.0),
            id::TIME_SIG_NUM => {
                let n = v.int().unwrap_or(4) as u8;
                f.time_signature = Some((n, f.time_signature.map_or(4, |s| s.1)));
            }
            id::TIME_SIG_DEN => {
                let d = v.int().unwrap_or(4) as u8;
                f.time_signature = Some((f.time_signature.map_or(4, |s| s.0), d));
            }
            id::NEW_CHANNEL => {
                let i = v.int().unwrap_or(0) as usize;
                if i < MAX_CHANNELS {
                    if f.channels.len() <= i {
                        f.channels.resize(i + 1, FlpChannel::default());
                    }
                    channel = Some(i);
                } else {
                    channel = None;
                }
                pattern = None;
            }
            id::NEW_PATTERN => {
                pattern = v.int().and_then(|p| u16::try_from(p).ok());
                channel = None;
            }
            id::NEW_ARRANGEMENT => {
                // Only the first arrangement is imported.
                if arrangement_seen {
                    items_done = true;
                }
                arrangement_seen = true;
                channel = None;
                pattern = None;
            }
            id::PATTERN_NAME => {
                if let (Some(p), Some(d)) = (pattern, v.data()) {
                    f.patterns.entry(p).or_default().name = Some(text(d)).filter(|s| !s.is_empty());
                }
            }
            id::PATTERN_NOTES => {
                if let (Some(p), Some(d)) = (pattern, v.data()) {
                    let pat = f.patterns.entry(p).or_default();
                    for r in d.chunks_exact(24) {
                        if notes >= MAX_NOTES {
                            break;
                        }
                        notes += 1;
                        pat.notes.push(FlpNote {
                            position: le32(r, 0).unwrap_or(0),
                            channel: le16(r, 6).unwrap_or(0),
                            length: le32(r, 8).unwrap_or(0),
                            key: le16(r, 12).unwrap_or(60),
                            velocity: r[21],
                        });
                    }
                }
            }
            id::PLAYLIST_ITEMS if !items_done => {
                items_done = true;
                let Some(d) = v.data() else { continue };
                if d.is_empty() {
                    continue;
                }
                let Some(size) = item_size(d) else {
                    f.unreadable_item_blocks += 1;
                    continue;
                };
                for r in d.chunks_exact(size) {
                    let idx = le16(r, 6).unwrap_or(0);
                    let base = le16(r, 4).unwrap_or(0);
                    let source = if base == PATTERN_BASE && idx > PATTERN_BASE {
                        ItemSource::Pattern(idx - PATTERN_BASE)
                    } else {
                        ItemSource::Channel(idx)
                    };
                    let offset = (size >= 32)
                        .then(|| le32(r, 24))
                        .flatten()
                        .filter(|o| *o != u32::MAX && matches!(source, ItemSource::Pattern(_)));
                    let flags = le16(r, 18).unwrap_or(0);
                    f.items.push(FlpItem {
                        position: le32(r, 0).unwrap_or(0),
                        length: le32(r, 8).unwrap_or(0),
                        track: (TRACKS - 1).saturating_sub(le16(r, 12).unwrap_or(TRACKS - 1)),
                        source,
                        offset,
                        // INFERRED: bit 12 of the item flags marks a muted clip.
                        muted: flags & 0x1000 != 0,
                    });
                }
            }
            _ if channel.is_some() => {
                let Some(c) = channel.and_then(|i| f.channels.get_mut(i)) else {
                    continue;
                };
                match id {
                    id::CHANNEL_TYPE => c.kind = v.int().unwrap_or(0) as u8,
                    id::CHANNEL_NAME if c.name.is_empty() => c.name = v.data().map(text).unwrap_or_default(),
                    id::SAMPLE_PATH => c.sample = v.data().map(text).filter(|s| !s.is_empty()),
                    id::CHANNEL_MIXER_TRACK => c.mixer_track = v.int(),
                    id::CHANNEL_COLOR => c.color = v.int(),
                    id::CHANNEL_LEVELS => {
                        if let Some(d) = v.data() {
                            c.pan = le32(d, 0).map(|p| ((p as f32 - 6400.0) / 6400.0).clamp(-1.0, 1.0));
                            c.volume = le32(d, 4).map(|x| (x as f32 / 12800.0).clamp(0.0, 1.25));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(f)
}

/// What an import kept and what it had to leave out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportReport {
    pub version: Option<String>,
    pub channels: usize,
    pub patterns: usize,
    pub notes: usize,
    pub clips: usize,
    /// Channels that were plugin generators or layers, imported as LimeSynth channels.
    pub replaced_generators: Vec<String>,
    pub skipped_automation_channels: usize,
    pub skipped_items: usize,
    pub missing_tempo: bool,
}

impl ImportReport {
    /// A one-line summary for a toast or the CLI.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} channels, {} patterns, {} notes, {} clips",
            self.channels, self.patterns, self.notes, self.clips
        );
        if !self.replaced_generators.is_empty() {
            s += &format!(
                "; {} plugin channel(s) became LimeSynth",
                self.replaced_generators.len()
            );
        }
        if self.skipped_automation_channels > 0 {
            s += &format!("; {} automation clip(s) skipped", self.skipped_automation_channels);
        }
        if self.skipped_items > 0 {
            s += &format!("; {} playlist item(s) skipped", self.skipped_items);
        }
        if self.missing_tempo {
            s += "; no tempo stored (140 BPM used)";
        }
        s
    }
}

/// Turn a sample path from the file into one LimeShot can open: `%FLStudioFactoryData%` becomes
/// `factory_dir` when given, and Windows separators become `/`.
pub fn resolve_sample(path: &str, factory_dir: Option<&Path>) -> String {
    let p = path.replace('\\', "/");
    match (p.strip_prefix("%FLStudioFactoryData%"), factory_dir) {
        (Some(rest), Some(dir)) => format!("{}{rest}", dir.display()),
        _ => p,
    }
}

/// FL's factory data folder on this machine, if FL Studio is installed in the usual place.
pub fn default_factory_dir() -> Option<std::path::PathBuf> {
    let candidates = [
        "/Applications/FL Studio 2026.app/Contents/Resources/FL",
        "/Applications/FL Studio 2025.app/Contents/Resources/FL",
        "/Applications/FL Studio 2024.app/Contents/Resources/FL",
        "C:/Program Files/Image-Line/FL Studio 2026",
        "C:/Program Files/Image-Line/FL Studio 2025",
        "C:/Program Files/Image-Line/FL Studio 2024",
    ];
    candidates.iter().map(std::path::PathBuf::from).find(|p| p.is_dir())
}

/// Convert a parsed file to a LimeShot project.
pub fn to_project(f: &FlpFile, factory_dir: Option<&Path>) -> (Project, ImportReport) {
    let mut p = Project::new();
    p.patterns.clear();
    let mut r = ImportReport {
        version: f.version.clone(),
        ..Default::default()
    };
    let scale = |t: u32| u64::from(t) * limeshot_core::PPQ / u64::from(f.ppq.max(1));
    if let Some(t) = f.tempo.filter(|t| t.is_finite() && *t > 0.0) {
        p.tempo = Tempo::from_bpm(t);
    } else {
        r.missing_tempo = true;
    }
    if let Some((n, d)) = f.time_signature {
        p.time_signature = TimeSignature::new(n.max(1), d.max(1));
    }
    if let Some(t) = &f.title {
        p.title = t.clone();
    }

    // Channels: rack index → LimeShot channel id (automation channels have none).
    let mut channel_ids: Vec<Option<ChannelId>> = Vec::with_capacity(f.channels.len());
    let mut max_mixer = 0u32;
    for (i, c) in f.channels.iter().enumerate() {
        if c.kind == 5 {
            r.skipped_automation_channels += 1;
            channel_ids.push(None);
            continue;
        }
        let name = if c.name.is_empty() {
            format!("Channel {}", i + 1)
        } else {
            c.name.clone()
        };
        let instrument = match (&c.sample, c.kind) {
            (Some(s), _) => Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: resolve_sample(s, factory_dir),
                }),
                // Audio-clip channels play for the clip's length.
                one_shot: c.kind != 4,
                ..SamplerParams::default()
            }),
            (None, 0) => Instrument::Sampler(SamplerParams::default()),
            (None, _) => {
                r.replaced_generators.push(name.clone());
                Instrument::Synth(SynthParams::default())
            }
        };
        let id = ChannelId(p.alloc_id());
        let mixer = c.mixer_track.filter(|m| *m < 127).unwrap_or(0);
        max_mixer = max_mixer.max(mixer);
        p.channels.push(Channel {
            id,
            name,
            color: c.color.map_or(PALETTE[p.channels.len() % PALETTE.len()], |v| {
                limeshot_core::Rgb(v as u8, (v >> 8) as u8, (v >> 16) as u8)
            }),
            instrument,
            volume: c.volume.unwrap_or(0.78),
            pan: c.pan.unwrap_or(0.0),
            muted: false,
            solo: false,
            mixer_track: mixer as u16,
            root_key: 60,
            extra: Extra::new(),
        });
        channel_ids.push(Some(id));
    }
    while p.mixer.tracks.len() <= max_mixer as usize && p.mixer.tracks.len() < Mixer::MAX_TRACKS {
        let n = p.mixer.tracks.len();
        p.mixer.tracks.push(MixerTrack::new(format!("Insert {n}")));
    }
    r.channels = p.channels.len();

    // Patterns: FL numbers them from 1.
    let mut pattern_ids: BTreeMap<u16, PatternId> = BTreeMap::new();
    for (num, fp) in &f.patterns {
        let id = PatternId(p.alloc_id());
        pattern_ids.insert(*num, id);
        let mut notes = Vec::with_capacity(fp.notes.len());
        for n in &fp.notes {
            let Some(Some(ch)) = channel_ids.get(usize::from(n.channel)) else {
                continue;
            };
            let start = scale(n.position);
            // Steps are stored with no length; LimeShot's steps are one step long.
            let length = if n.length == 0 {
                STEP_TICKS
            } else {
                scale(n.length).max(1)
            };
            notes.push(Note {
                id: NoteId(p.alloc_id()),
                channel: *ch,
                key: n.key.min(127) as u8,
                start,
                length,
                velocity: (f32::from(n.velocity) / 128.0).clamp(0.0, 1.0),
            });
        }
        r.notes += notes.len();
        p.patterns.push(Pattern {
            id,
            name: fp.name.clone().unwrap_or_else(|| format!("Pattern {num}")),
            color: PALETTE[p.patterns.len() % PALETTE.len()],
            length_steps: None,
            notes,
            extra: Extra::new(),
        });
    }
    r.patterns = p.patterns.len();
    if p.patterns.is_empty() {
        let id = PatternId(p.alloc_id());
        p.patterns.push(Pattern {
            id,
            name: "Pattern 1".into(),
            color: PALETTE[0],
            length_steps: None,
            notes: Vec::new(),
            extra: Extra::new(),
        });
    }

    // The arrangement.
    let mut clips = Vec::new();
    let mut max_track = 15u16;
    for it in &f.items {
        let source = match it.source {
            ItemSource::Pattern(n) => pattern_ids.get(&n).map(|id| ClipSource::Pattern(*id)),
            ItemSource::Channel(c) => match (channel_ids.get(usize::from(c)), f.channels.get(usize::from(c))) {
                (Some(Some(id)), Some(ch)) if ch.sample.is_some() => Some(ClipSource::Audio(*id)),
                _ => None,
            },
        };
        let Some(source) = source else {
            r.skipped_items += 1;
            continue;
        };
        max_track = max_track.max(it.track);
        clips.push(Clip {
            id: ClipId(p.alloc_id()),
            source,
            track: u32::from(it.track),
            start: scale(it.position),
            length: scale(it.length).max(1),
            offset: it.offset.map(scale).unwrap_or(0),
            muted: it.muted,
            extra: Extra::new(),
        });
    }
    r.clips = clips.len();
    p.playlist = Playlist {
        tracks: (1..=u32::from(max_track) + 1)
            .map(|i| PlaylistTrack {
                name: format!("Track {i}"),
                muted: false,
            })
            .collect(),
        clips,
    };
    p.sanitize();
    (p, r)
}

/// Read and convert an `.flp` file.
pub fn import_file(path: &Path, factory_dir: Option<&Path>) -> Result<(Project, ImportReport), FlpError> {
    let io = |source| FlpError::Io {
        path: path.display().to_string(),
        source,
    };
    if std::fs::metadata(path).map_err(io)?.len() > MAX_FILE_BYTES {
        return Err(bad("file is too large"));
    }
    let data = std::fs::read(path).map_err(io)?;
    let f = parse(&data)?;
    let (mut p, r) = to_project(&f, factory_dir);
    if f.title.is_none()
        && let Some(stem) = path.file_stem()
    {
        p.title = stem.to_string_lossy().into_owned();
    }
    Ok((p, r))
}

/// Writes `.flp` event streams, for tests and fixtures (the importer's counterpart; LimeShot does
/// not export to FL Studio).
pub mod writer {
    use super::{PATTERN_BASE, TRACKS, id};

    #[derive(Debug, Default, Clone)]
    pub struct Writer {
        events: Vec<u8>,
        channels: u16,
    }

    impl Writer {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn byte(&mut self, id: u8, v: u8) -> &mut Self {
            self.events.extend_from_slice(&[id, v]);
            self
        }

        pub fn word(&mut self, id: u8, v: u16) -> &mut Self {
            self.events.push(id);
            self.events.extend_from_slice(&v.to_le_bytes());
            self
        }

        pub fn dword(&mut self, id: u8, v: u32) -> &mut Self {
            self.events.push(id);
            self.events.extend_from_slice(&v.to_le_bytes());
            self
        }

        pub fn data(&mut self, id: u8, d: &[u8]) -> &mut Self {
            self.events.push(id);
            let mut n = d.len();
            loop {
                let b = (n & 0x7F) as u8;
                n >>= 7;
                self.events.push(if n > 0 { b | 0x80 } else { b });
                if n == 0 {
                    break;
                }
            }
            self.events.extend_from_slice(d);
            self
        }

        pub fn text(&mut self, id: u8, s: &str) -> &mut Self {
            let mut d: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
            d.extend_from_slice(&[0, 0]);
            self.data(id, &d)
        }

        pub fn tempo(&mut self, bpm: f64) -> &mut Self {
            self.dword(id::TEMPO, (bpm * 1000.0).round() as u32)
        }

        /// A channel: type (0 sampler, 2 generator, 4 audio clip, 5 automation), name, optional
        /// sample, volume/pan on FL's scales (12800 max, pan 0–12800 with 6400 centre), insert.
        pub fn channel(
            &mut self,
            kind: u8,
            name: &str,
            sample: Option<&str>,
            volume: u32,
            pan: u32,
            mixer: u32,
        ) -> &mut Self {
            let index = self.channels;
            self.channels += 1;
            self.word(id::NEW_CHANNEL, index)
                .byte(id::CHANNEL_TYPE, kind)
                .text(id::CHANNEL_NAME, name);
            let mut levels = Vec::new();
            levels.extend_from_slice(&pan.to_le_bytes());
            levels.extend_from_slice(&volume.to_le_bytes());
            levels.extend_from_slice(&[0; 16]);
            self.data(id::CHANNEL_LEVELS, &levels)
                .byte(id::CHANNEL_MIXER_TRACK, mixer.min(255) as u8);
            if let Some(s) = sample {
                self.text(id::SAMPLE_PATH, s);
            }
            self
        }

        /// Notes as (position, length, channel, key, velocity) in file ticks.
        pub fn pattern(&mut self, number: u16, name: Option<&str>, notes: &[(u32, u32, u16, u16, u8)]) -> &mut Self {
            let mut d = Vec::new();
            for &(pos, len, ch, key, vel) in notes {
                d.extend_from_slice(&pos.to_le_bytes());
                d.extend_from_slice(&0x4000u16.to_le_bytes());
                d.extend_from_slice(&ch.to_le_bytes());
                d.extend_from_slice(&len.to_le_bytes());
                d.extend_from_slice(&key.to_le_bytes());
                d.extend_from_slice(&0u16.to_le_bytes());
                d.extend_from_slice(&120u16.to_le_bytes());
                d.extend_from_slice(&64u16.to_le_bytes());
                d.extend_from_slice(&[64, vel, 128, 128]);
            }
            self.word(id::NEW_PATTERN, number).data(id::PATTERN_NOTES, &d);
            if let Some(n) = name {
                self.word(id::NEW_PATTERN, number).text(id::PATTERN_NAME, n);
            }
            self
        }

        /// Playlist items as (position, length, track, pattern number or `None` + channel,
        /// pattern start offset) in file ticks, written as `size`-byte records.
        pub fn items(&mut self, size: usize, items: &[(u32, u32, u16, Result<u16, u16>, Option<u32>)]) -> &mut Self {
            let mut d = Vec::new();
            for &(pos, len, track, src, offset) in items {
                let mut r = vec![0u8; size];
                r[0..4].copy_from_slice(&pos.to_le_bytes());
                r[4..6].copy_from_slice(&PATTERN_BASE.to_le_bytes());
                let idx = match src {
                    Ok(pattern) => PATTERN_BASE + pattern,
                    Err(channel) => channel,
                };
                r[6..8].copy_from_slice(&idx.to_le_bytes());
                r[8..12].copy_from_slice(&len.to_le_bytes());
                r[12..14].copy_from_slice(&(TRACKS - 1 - track).to_le_bytes());
                if size >= 32 {
                    r[24..28].copy_from_slice(&offset.unwrap_or(u32::MAX).to_le_bytes());
                    r[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
                }
                d.extend_from_slice(&r);
            }
            self.data(id::PLAYLIST_ITEMS, &d)
        }

        /// The complete file.
        pub fn finish(&self, ppq: u16) -> Vec<u8> {
            let mut out = b"FLhd".to_vec();
            out.extend_from_slice(&6u32.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&self.channels.to_le_bytes());
            out.extend_from_slice(&ppq.to_le_bytes());
            out.extend_from_slice(b"FLdt");
            out.extend_from_slice(&(self.events.len() as u32).to_le_bytes());
            out.extend_from_slice(&self.events);
            out
        }
    }
}

/// The test files in `crates/limeshot-project/tests/fixtures/flp`, generated by this function
/// (`cargo run -p limeshot-project --example make_flp_fixtures`); a test checks they match.
pub fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    use writer::Writer;
    let version = |w: &mut Writer| {
        w.data(id::VERSION, b"25.2.3.5164\0");
    };
    let mut out = Vec::new();

    let mut w = Writer::new();
    version(&mut w);
    w.tempo(140.0).channel(0, "Sampler", None, 10_000, 6400, 0);
    out.push(("empty.flp", w.finish(96)));

    // Drum steps: zero-length notes on 16th-note positions (24 ticks at 96 PPQ).
    let mut w = Writer::new();
    version(&mut w);
    w.tempo(120.0)
        .byte(id::TIME_SIG_NUM, 4)
        .byte(id::TIME_SIG_DEN, 4)
        .text(id::TITLE, "Steps")
        .channel(
            0,
            "Kick",
            Some("%FLStudioFactoryData%\\Data\\Kick.wav"),
            10_000,
            6400,
            1,
        )
        .channel(0, "Snare", Some("C:\\Samples\\Snare.wav"), 12_800, 3200, 2)
        .channel(0, "Hat", Some("hat.wav"), 6400, 9600, 3);
    let mut steps = Vec::new();
    for s in 0..16u32 {
        if s % 4 == 0 {
            steps.push((s * 24, 0, 0, 60, 100));
        }
        if s % 8 == 4 {
            steps.push((s * 24, 0, 1, 60, 128));
        }
        steps.push((s * 24, 0, 2, 60, 64));
    }
    w.pattern(1, Some("Beat"), &steps)
        .items(240, &[(0, 384, 0, Ok(1), None), (384, 384, 0, Ok(1), None)]);
    out.push(("steps.flp", w.finish(96)));

    // A melody in 3/4 with lengths, velocities, two patterns and a clip that starts inside its
    // pattern, using the 60-byte item records of FL 20–24.
    let mut w = Writer::new();
    version(&mut w);
    w.tempo(128.5)
        .byte(id::TIME_SIG_NUM, 3)
        .byte(id::TIME_SIG_DEN, 4)
        .channel(2, "Lead", None, 10_000, 6400, 4)
        .pattern(
            1,
            Some("Verse"),
            &[(0, 96, 0, 60, 100), (96, 48, 0, 64, 80), (144, 144, 0, 67, 128)],
        )
        .pattern(2, None, &[(0, 288, 0, 72, 50)])
        .items(
            60,
            &[
                (0, 288, 2, Ok(1), None),
                (288, 144, 2, Ok(1), Some(96)),
                (576, 288, 3, Ok(2), None),
            ],
        );
    out.push(("melody.flp", w.finish(96)));

    // An audio clip channel with a clip, and an automation channel whose clip LimeShot skips.
    let mut w = Writer::new();
    version(&mut w);
    w.tempo(100.0)
        .channel(4, "Vocal", Some("vocal.wav"), 10_000, 6400, 5)
        .channel(5, "Volume Duck", None, 10_000, 6400, 0)
        .items(240, &[(192, 768, 6, Err(0), None), (0, 1536, 7, Err(1), None)]);
    out.push(("audio_and_automation.flp", w.finish(96)));

    // Another time base (192 PPQ) and the 32-byte item records of older versions.
    let mut w = Writer::new();
    w.tempo(90.0)
        .channel(2, "Pad", None, 10_000, 6400, 0)
        .pattern(1, Some("Long"), &[(0, 768, 0, 48, 100), (768, 192, 0, 50, 100)])
        .items(32, &[(768, 1536, 1, Ok(1), None)]);
    out.push(("ppq192.flp", w.finish(192)));

    // The steps file cut in the middle of the playlist event.
    let mut t = out[1].1.clone();
    t.truncate(t.len() - 100);
    out.push(("truncated.flp", t));
    out
}

#[cfg(test)]
mod tests {
    use super::writer::Writer;
    use super::*;

    #[test]
    fn events_of_every_size_round_trip() {
        let mut w = Writer::new();
        w.byte(1, 7).word(70, 513).dword(150, 70_000).data(200, &[1; 200]);
        let file = w.finish(96);
        let (h, ev) = read_events(&file).unwrap();
        assert_eq!(h.ppq, 96);
        let kinds: Vec<(u8, Option<u32>)> = ev.iter().map(|(i, v)| (*i, v.int())).collect();
        assert_eq!(
            kinds,
            vec![(1, Some(7)), (70, Some(513)), (150, Some(70_000)), (200, None)]
        );
        assert_eq!(ev[3].1.data().map(<[u8]>::len), Some(200), "two-byte length prefix");
    }

    #[test]
    fn factory_paths_are_resolved() {
        let dir = Path::new("/FL");
        assert_eq!(
            resolve_sample("%FLStudioFactoryData%\\Data\\Kick.wav", Some(dir)),
            "/FL/Data/Kick.wav"
        );
        assert_eq!(resolve_sample("C:\\x\\a.wav", None), "C:/x/a.wav");
    }

    #[test]
    fn hostile_bytes_never_panic() {
        let mut w = Writer::new();
        w.tempo(128.0)
            .channel(0, "Kick", Some("k.wav"), 10_000, 6400, 1)
            .pattern(1, Some("A"), &[(0, 0, 0, 60, 100)])
            .items(240, &[(0, 384, 0, Ok(1), None)]);
        let good = w.finish(96);
        for n in 0..good.len() {
            let _ = parse(&good[..n]).map(|f| to_project(&f, None));
        }
        let mut huge = good.clone();
        huge[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse(&huge).is_err());
        assert!(parse(b"RIFF").is_err());
    }
}
