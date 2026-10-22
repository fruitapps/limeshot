//! FL Studio's Piano Roll tools as pure functions over notes: each takes the notes to work on
//! and returns the notes that replace them, applied with [`crate::Command::EditNotes`] so a tool
//! is one undo step. Ids are kept where a note survives, so the selection follows the notes.

use crate::model::{Note, NoteId};
use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// Which Piano Roll tool an [`crate::Command::EditNotes`] comes from (its undo label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteTool {
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
}
