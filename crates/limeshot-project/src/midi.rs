//! Standard MIDI Files: reading (import into a pattern) and writing (export of a pattern or the
//! song).
//!
//! Written from the public SMF 1.0 description: a header chunk (`MThd`: format, track count,
//! ticks per quarter note) and track chunks (`MTrk`) of delta-timed events. Only what LimeShot
//! needs is interpreted: notes, tempo, time signature and track names; everything else is
//! skipped. Input is treated as hostile: every length is checked against the data, event counts
//! are capped, and SMPTE time division is refused.

use limeshot_core::{Command, ImportedChannel, Instrument, PPQ, Project, SynthParams, Tempo, Tick, TimeSignature};
use limeshot_sequencer::Program;
use std::path::Path;

/// Larger files are refused.
pub const MAX_FILE_BYTES: usize = 32 * 1024 * 1024;
/// Notes kept per import (a whole song fits easily).
pub const MAX_NOTES: usize = 200_000;
const MAX_TRACKS: usize = 1024;

#[derive(Debug, thiserror::Error)]
pub enum MidiError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("not a MIDI file: {0}")]
    Format(String),
    #[error("{0} is not supported")]
    Unsupported(&'static str),
    #[error("the MIDI file has no notes")]
    Empty,
}

/// A note in MIDI terms, at LimeShot's resolution (960 ticks per quarter note).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiNote {
    pub tick: Tick,
    pub length: Tick,
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MidiTrack {
    pub name: Option<String>,
    /// MIDI channel 0–15.
    pub channel: u8,
    pub notes: Vec<MidiNote>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiFile {
    pub tempo: Option<Tempo>,
    pub time_signature: Option<TimeSignature>,
    /// One entry per (file track, MIDI channel) that has notes, in file order.
    pub tracks: Vec<MidiTrack>,
}

// ----- writing -----

fn vlq(out: &mut Vec<u8>, v: u64) {
    // Four bytes hold 28 bits, the format's limit.
    let mut v = v.min(0x0FFF_FFFF);
    let mut bytes = [0u8; 4];
    let mut n = 0;
    loop {
        bytes[n] = (v & 0x7F) as u8;
        v >>= 7;
        n += 1;
        if v == 0 {
            break;
        }
    }
    for i in (0..n).rev() {
        out.push(bytes[i] | if i > 0 { 0x80 } else { 0 });
    }
}

fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
}

fn meta(body: &mut Vec<u8>, delta: u64, kind: u8, data: &[u8]) {
    vlq(body, delta);
    body.extend_from_slice(&[0xFF, kind]);
    vlq(body, data.len() as u64);
    body.extend_from_slice(data);
}

/// Encode a format-1 file at 960 ticks per quarter note: a conductor track (tempo, time
/// signature), then one track per entry of `tracks`.
pub fn write(file: &MidiFile) -> Vec<u8> {
    let mut out = Vec::new();
    let mut header = Vec::new();
    header.extend_from_slice(&1u16.to_be_bytes());
    header.extend_from_slice(&((file.tracks.len() + 1).min(0xFFFF) as u16).to_be_bytes());
    header.extend_from_slice(&(PPQ as u16).to_be_bytes());
    chunk(&mut out, b"MThd", &header);

    let mut conductor = Vec::new();
    if let Some(t) = file.tempo {
        let us = (60_000_000.0 / t.bpm()).round().clamp(1.0, 16_777_215.0) as u32;
        meta(&mut conductor, 0, 0x51, &us.to_be_bytes()[1..]);
    }
    if let Some(s) = file.time_signature {
        let dd = s.denominator.max(1).trailing_zeros() as u8;
        meta(&mut conductor, 0, 0x58, &[s.numerator, dd, 24, 8]);
    }
    meta(&mut conductor, 0, 0x2F, &[]);
    chunk(&mut out, b"MTrk", &conductor);

    for t in file.tracks.iter().take(0xFFFE) {
        let mut body = Vec::new();
        if let Some(name) = &t.name {
            meta(&mut body, 0, 0x03, name.as_bytes());
        }
        // (tick, is_on, key, velocity); offs sort before ons at the same tick.
        let mut ev: Vec<(Tick, bool, u8, u8)> = Vec::with_capacity(2 * t.notes.len());
        for n in &t.notes {
            ev.push((n.tick, true, n.key.min(127), n.velocity.clamp(1, 127)));
            ev.push((n.tick + n.length.max(1), false, n.key.min(127), 64));
        }
        ev.sort_by_key(|e| (e.0, e.1, e.2));
        let ch = t.channel & 0x0F;
        let mut now = 0;
        for (tick, on, key, vel) in ev {
            vlq(&mut body, tick - now);
            now = tick;
            body.extend_from_slice(&[if on { 0x90 } else { 0x80 } | ch, key, vel]);
        }
        meta(&mut body, 0, 0x2F, &[]);
        chunk(&mut out, b"MTrk", &body);
    }
    out
}

// ----- reading -----

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn bad(what: &str) -> MidiError {
        MidiError::Format(what.to_string())
    }
    fn u8(&mut self) -> Result<u8, MidiError> {
        let b = *self
            .data
            .get(self.pos)
            .ok_or_else(|| Self::bad("unexpected end of track"))?;
        self.pos += 1;
        Ok(b)
    }
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], MidiError> {
        let end = self.pos.checked_add(n).ok_or_else(|| Self::bad("length overflow"))?;
        let s = self
            .data
            .get(self.pos..end)
            .ok_or_else(|| Self::bad("chunk runs past the end"))?;
        self.pos = end;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, MidiError> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn vlq(&mut self) -> Result<u64, MidiError> {
        let mut v = 0u64;
        for _ in 0..4 {
            let b = self.u8()?;
            v = (v << 7) | u64::from(b & 0x7F);
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(Self::bad("variable-length number is too long"))
    }
    fn done(&self) -> bool {
        self.pos >= self.data.len()
    }
}

/// Decode a MIDI file. Note ticks are rescaled to 960 per quarter note.
pub fn read(data: &[u8]) -> Result<MidiFile, MidiError> {
    if data.len() > MAX_FILE_BYTES {
        return Err(MidiError::Format("file is too large".into()));
    }
    let mut r = Reader { data, pos: 0 };
    if r.bytes(4)? != b"MThd" {
        return Err(MidiError::Format("missing MThd header".into()));
    }
    let hlen = r.u32()? as usize;
    let header = r.bytes(hlen)?;
    if header.len() < 6 {
        return Err(MidiError::Format("short header".into()));
    }
    let format = u16::from_be_bytes([header[0], header[1]]);
    let ntracks = usize::from(u16::from_be_bytes([header[2], header[3]]));
    let division = u16::from_be_bytes([header[4], header[5]]);
    if format > 2 {
        return Err(MidiError::Format(format!("unknown format {format}")));
    }
    if format == 2 {
        return Err(MidiError::Unsupported("MIDI format 2 (independent sequences)"));
    }
    if division & 0x8000 != 0 {
        return Err(MidiError::Unsupported("SMPTE time division"));
    }
    let ppq = u64::from(division.max(1));
    let scale = |t: u64| (u128::from(t) * u128::from(PPQ) / u128::from(ppq)).min(u128::from(u64::MAX / 4)) as u64;

    let mut file = MidiFile {
        tempo: None,
        time_signature: None,
        tracks: Vec::new(),
    };
    let mut total_notes = 0usize;
    let mut seen = 0;
    while !r.done() && seen < ntracks.min(MAX_TRACKS) {
        let id = r.bytes(4)?;
        let len = r.u32()? as usize;
        let body = r.bytes(len)?;
        if id != b"MTrk" {
            continue; // Unknown chunks are skipped, as the format requires.
        }
        seen += 1;
        let mut t = Reader { data: body, pos: 0 };
        let mut name = None;
        let mut now = 0u64;
        let mut running = None;
        // Per channel: notes, and the open note-ons per key (FIFO).
        let mut notes: Vec<Vec<MidiNote>> = vec![Vec::new(); 16];
        let mut open: Vec<Vec<(u64, u8)>> = vec![Vec::new(); 16 * 128];
        let close = |notes: &mut Vec<Vec<MidiNote>>, open: &mut Vec<Vec<(u64, u8)>>, ch: usize, key: u8, at: u64| {
            let slot = &mut open[ch * 128 + usize::from(key)];
            if !slot.is_empty() {
                let (start, vel) = slot.remove(0);
                notes[ch].push(MidiNote {
                    tick: scale(start),
                    length: scale(at).saturating_sub(scale(start)).max(1),
                    key,
                    velocity: vel,
                });
            }
        };
        while !t.done() {
            now = now.saturating_add(t.vlq()?);
            let mut status = t.u8()?;
            if status < 0x80 {
                // Running status: reuse the previous channel status; this byte is data.
                status = running.ok_or_else(|| MidiError::Format("data byte without a status".into()))?;
                t.pos -= 1;
            }
            match status {
                0xFF => {
                    let kind = t.u8()?;
                    let n = t.vlq()? as usize;
                    let d = t.bytes(n)?;
                    match kind {
                        0x03 if name.is_none() => name = Some(String::from_utf8_lossy(d).trim().to_string()),
                        0x51 if d.len() == 3 && file.tempo.is_none() => {
                            let us = u32::from_be_bytes([0, d[0], d[1], d[2]]).max(1);
                            file.tempo = Some(Tempo::from_bpm(60_000_000.0 / f64::from(us)));
                        }
                        0x58 if d.len() >= 2 && file.time_signature.is_none() => {
                            let den = 1u8.checked_shl(u32::from(d[1].min(7))).unwrap_or(4);
                            file.time_signature = Some(TimeSignature::new(d[0], den));
                        }
                        0x2F => break,
                        _ => {}
                    }
                }
                0xF0 | 0xF7 => {
                    let n = t.vlq()? as usize;
                    t.bytes(n)?;
                }
                0x80..=0xEF => {
                    running = Some(status);
                    let ch = usize::from(status & 0x0F);
                    let a = t.u8()? & 0x7F;
                    let kind = status & 0xF0;
                    let b = if matches!(kind, 0xC0 | 0xD0) { 0 } else { t.u8()? & 0x7F };
                    match kind {
                        0x90 if b > 0 => {
                            if total_notes < MAX_NOTES {
                                open[ch * 128 + usize::from(a)].push((now, b));
                                total_notes += 1;
                            }
                        }
                        0x80 | 0x90 => close(&mut notes, &mut open, ch, a, now),
                        _ => {}
                    }
                }
                _ => return Err(MidiError::Format(format!("unexpected status byte {status:#04x}"))),
            }
        }
        // Notes still held at the end of the track end there.
        for ch in 0..16 {
            for key in 0..128u8 {
                while !open[ch * 128 + usize::from(key)].is_empty() {
                    close(&mut notes, &mut open, ch, key, now.max(1));
                }
            }
        }
        let used: Vec<usize> = (0..16).filter(|c| !notes[*c].is_empty()).collect();
        for &ch in &used {
            let mut n = std::mem::take(&mut notes[ch]);
            n.sort_by_key(|x| (x.tick, x.key));
            let base = name.clone().filter(|s| !s.is_empty());
            file.tracks.push(MidiTrack {
                name: match (&base, used.len()) {
                    (Some(b), 1) => Some(b.clone()),
                    (Some(b), _) => Some(format!("{b} ch{}", ch + 1)),
                    (None, _) => None,
                },
                channel: ch as u8,
                notes: n,
            });
        }
    }
    Ok(file)
}

pub fn read_file(path: &Path) -> Result<MidiFile, MidiError> {
    let io = |source| MidiError::Io {
        path: path.display().to_string(),
        source,
    };
    let len = std::fs::metadata(path).map_err(io)?.len();
    if len > MAX_FILE_BYTES as u64 {
        return Err(MidiError::Format("file is too large".into()));
    }
    read(&std::fs::read(path).map_err(io)?)
}

// ----- LimeShot conversions -----

/// The command that imports `file` as a new pattern named `name` with a LimeSynth channel per
/// MIDI track. `take_tempo` also sets the project tempo from the file.
pub fn import_command(file: &MidiFile, name: &str, take_tempo: bool) -> Result<Command, MidiError> {
    let channels: Vec<ImportedChannel> = file
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.notes.is_empty())
        .map(|(i, t)| ImportedChannel {
            name: t.name.clone().unwrap_or_else(|| format!("MIDI {}", i + 1)),
            instrument: Instrument::Synth(SynthParams::default()),
            notes: t
                .notes
                .iter()
                .map(|n| (n.key, n.tick, n.length, f32::from(n.velocity) / 127.0))
                .collect(),
        })
        .collect();
    if channels.is_empty() {
        return Err(MidiError::Empty);
    }
    Ok(Command::ImportPattern {
        name: name.to_string(),
        tempo: if take_tempo { file.tempo } else { None },
        channels,
    })
}

/// A MIDI file of a compiled program (a pattern or the song, swing included): one track per
/// LimeShot channel that plays, named after it, on MIDI channels 1–16 in turn (10 skipped).
pub fn from_program(project: &Project, program: &Program) -> MidiFile {
    let mut tracks: Vec<MidiTrack> = project
        .channels
        .iter()
        .enumerate()
        .map(|(i, c)| {
            // Channel 10 is drums in General MIDI; LimeShot's channels are not GM-mapped.
            let midi_ch = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15][i % 15];
            MidiTrack {
                name: Some(c.name.clone()),
                channel: midi_ch,
                notes: Vec::new(),
            }
        })
        .collect();
    for e in &program.events {
        if e.clip {
            continue; // Audio clips have no MIDI form.
        }
        if let Some(t) = tracks.get_mut(usize::from(e.channel)) {
            t.notes.push(MidiNote {
                tick: e.tick,
                length: e.length,
                key: e.key,
                velocity: (e.velocity * 127.0).round().clamp(1.0, 127.0) as u8,
            });
        }
    }
    tracks.retain(|t| !t.notes.is_empty());
    MidiFile {
        tempo: Some(project.tempo),
        time_signature: Some(project.time_signature),
        tracks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(tick: Tick, length: Tick, key: u8, velocity: u8) -> MidiNote {
        MidiNote {
            tick,
            length,
            key,
            velocity,
        }
    }

    #[test]
    fn write_then_read_round_trips() {
        let file = MidiFile {
            tempo: Some(Tempo::from_bpm(128.0)),
            time_signature: Some(TimeSignature::new(3, 4)),
            tracks: vec![
                MidiTrack {
                    name: Some("Lead".into()),
                    channel: 0,
                    // Overlapping notes of the same key pair up first-in, first-out.
                    notes: vec![note(0, 480, 60, 100), note(240, 960, 60, 90), note(1920, 1, 72, 1)],
                },
                MidiTrack {
                    name: Some("Bass".into()),
                    channel: 1,
                    notes: vec![note(0, 3840, 36, 127)],
                },
            ],
        };
        let back = read(&write(&file)).unwrap();
        assert_eq!(back, file);
    }

    #[test]
    fn reads_running_status_other_resolutions_and_note_on_zero() {
        // Format 0, 96 ticks per quarter: running status note-on, then a velocity-0 note-on as
        // the note-off, a program change and a sysex, then end of track.
        let mut trk = vec![0x00, 0xC0, 0x05];
        trk.extend_from_slice(&[0x00, 0xF0, 0x02, 0x7E, 0xF7]);
        trk.extend_from_slice(&[0x00, 0x90, 60, 100]);
        trk.extend_from_slice(&[0x30, 64, 80]); // running status: key 64 at tick 48
        trk.extend_from_slice(&[0x30, 60, 0]); // key 60 off at tick 96
        trk.extend_from_slice(&[0x00, 64, 0]);
        trk.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);
        let mut data = b"MThd".to_vec();
        data.extend_from_slice(&[0, 0, 0, 6, 0, 0, 0, 1, 0, 96]);
        data.extend_from_slice(b"MTrk");
        data.extend_from_slice(&(trk.len() as u32).to_be_bytes());
        data.extend_from_slice(&trk);
        let f = read(&data).unwrap();
        assert_eq!(f.tracks.len(), 1);
        assert_eq!(f.tracks[0].notes, vec![note(0, 960, 60, 100), note(480, 480, 64, 80)]);
    }

    #[test]
    fn hostile_input_is_refused_not_panicked_on() {
        let good = write(&MidiFile {
            tempo: None,
            time_signature: None,
            tracks: vec![MidiTrack {
                name: None,
                channel: 0,
                notes: vec![note(0, 10, 60, 100)],
            }],
        });
        // Every truncation is an error or a smaller file, never a panic.
        for n in 0..good.len() {
            let _ = read(&good[..n]);
        }
        let mut smpte = good.clone();
        smpte[12] = 0xE7;
        assert!(matches!(read(&smpte), Err(MidiError::Unsupported(_))));
        assert!(read(b"RIFF....").is_err());
        let mut huge_len = good.clone();
        huge_len[18..22].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(read(&huge_len).is_err());
    }

    #[test]
    fn import_command_builds_one_channel_per_track() {
        let file = MidiFile {
            tempo: Some(Tempo::from_bpm(90.0)),
            time_signature: None,
            tracks: vec![MidiTrack {
                name: Some("Piano".into()),
                channel: 0,
                notes: vec![note(0, 960, 60, 127)],
            }],
        };
        let mut d = limeshot_core::Document::default();
        let pat = d
            .execute(import_command(&file, "Song", true).unwrap())
            .unwrap()
            .0
            .unwrap();
        let p = d.project();
        assert_eq!(p.tempo, Tempo::from_bpm(90.0));
        assert_eq!(p.channels[0].name, "Piano");
        let pattern = p.pattern(limeshot_core::PatternId(pat)).unwrap();
        assert_eq!(pattern.name, "Song");
        assert_eq!((pattern.notes[0].key, pattern.notes[0].length), (60, 960));
        assert_eq!(pattern.notes[0].velocity, 1.0);
        assert_eq!(d.history().len(), 1, "one undo step");
        assert!(matches!(
            import_command(
                &MidiFile {
                    tempo: None,
                    time_signature: None,
                    tracks: vec![]
                },
                "x",
                false
            ),
            Err(MidiError::Empty)
        ));
    }
}
