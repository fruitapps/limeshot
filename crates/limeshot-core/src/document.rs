//! A project plus its undo history and modified state.

use crate::command::{Command, CommandError, Created};
use crate::model::Project;

/// How many undo steps are kept.
pub const UNDO_LIMIT: usize = 200;

struct Step {
    label: &'static str,
    before: Project,
    before_state: u64,
    merge: Option<String>,
}

/// The open project. All edits go through [`Document::execute`].
pub struct Document {
    project: Project,
    undo: Vec<Step>,
    redo: Vec<(&'static str, Project, u64)>,
    /// Increments on every change (edit, undo, redo, replace).
    revision: u64,
    /// Identifies the current content: undoing back to a saved state makes it unmodified again.
    state: u64,
    next_state: u64,
    saved_state: u64,
    /// While a gesture is open, commands with the same merge key share one undo step.
    gesture_open: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self::new(Project::new())
    }
}

impl Document {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            state: 0,
            next_state: 1,
            saved_state: 0,
            gesture_open: false,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_modified(&self) -> bool {
        self.state != self.saved_state
    }

    pub fn mark_saved(&mut self) {
        self.saved_state = self.state;
    }

    fn new_state(&mut self) -> u64 {
        let s = self.next_state;
        self.next_state += 1;
        s
    }

    /// Replace the whole project (new/open). Clears the history.
    pub fn replace(&mut self, project: Project) {
        self.project = project;
        self.undo.clear();
        self.redo.clear();
        self.revision += 1;
        self.state = self.new_state();
        self.saved_state = self.state;
        self.gesture_open = false;
    }

    /// Start a continuous gesture (a drag). Until [`Document::end_gesture`], consecutive commands
    /// with equal merge keys become one undo step.
    pub fn begin_gesture(&mut self) {
        self.gesture_open = true;
    }

    pub fn end_gesture(&mut self) {
        self.gesture_open = false;
        if let Some(s) = self.undo.last_mut() {
            s.merge = None;
        }
    }

    /// Apply a command. On error the project is unchanged.
    pub fn execute(&mut self, cmd: Command) -> Result<Created, CommandError> {
        let label = cmd.label();
        let merge = cmd.merge_key();
        let mut next = self.project.clone();
        let out = cmd.apply(&mut next)?;
        if next == self.project {
            return Ok(out);
        }
        let merges = self.gesture_open
            && merge.is_some()
            && self
                .undo
                .last()
                .is_some_and(|s| s.merge == merge && self.redo.is_empty());
        let before = std::mem::replace(&mut self.project, next);
        let before_state = std::mem::replace(&mut self.state, 0);
        self.state = self.new_state();
        if !merges {
            self.undo.push(Step {
                label,
                before,
                before_state,
                merge: if self.gesture_open { merge } else { None },
            });
            if self.undo.len() > UNDO_LIMIT {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.revision += 1;
        Ok(out)
    }

    pub fn undo_label(&self) -> Option<&'static str> {
        self.undo.last().map(|s| s.label)
    }

    pub fn redo_label(&self) -> Option<&'static str> {
        self.redo.last().map(|r| r.0)
    }

    pub fn undo(&mut self) -> bool {
        let Some(step) = self.undo.pop() else { return false };
        let after = std::mem::replace(&mut self.project, step.before);
        let after_state = std::mem::replace(&mut self.state, step.before_state);
        self.redo.push((step.label, after, after_state));
        self.revision += 1;
        self.gesture_open = false;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some((label, after, after_state)) = self.redo.pop() else {
            return false;
        };
        let before = std::mem::replace(&mut self.project, after);
        let before_state = std::mem::replace(&mut self.state, after_state);
        self.undo.push(Step {
            label,
            before,
            before_state,
            merge: None,
        });
        self.revision += 1;
        true
    }

    /// Undo history labels, oldest first.
    pub fn history(&self) -> Vec<&'static str> {
        self.undo.iter().map(|s| s.label).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::NewNote;
    use crate::model::*;
    use crate::time::{STEP_TICKS, Tempo, Tick};

    fn doc_with_channel() -> (Document, ChannelId, PatternId) {
        let mut d = Document::default();
        let id = d
            .execute(Command::AddChannel {
                name: "Kick".into(),
                instrument: Instrument::Sampler(SamplerParams::default()),
            })
            .unwrap()
            .0
            .unwrap();
        let pat = d.project().patterns[0].id;
        (d, ChannelId(id), pat)
    }

    #[test]
    fn undo_and_redo_restore_state() {
        let (mut d, ch, pat) = doc_with_channel();
        let before = d.project().clone();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: ch,
            step: 4,
            on: true,
        })
        .unwrap();
        assert_eq!(d.project().patterns[0].notes.len(), 1);
        assert_eq!(d.project().patterns[0].notes[0].start, 4 * STEP_TICKS);
        assert!(d.undo());
        assert_eq!(d.project(), &before);
        assert!(d.redo());
        assert_eq!(d.project().patterns[0].notes.len(), 1);
    }

    #[test]
    fn failed_command_changes_nothing() {
        let (mut d, _, pat) = doc_with_channel();
        let rev = d.revision();
        let err = d.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: ChannelId(999),
                key: 60,
                start: 0,
                length: 10,
                velocity: 1.0,
            },
        });
        assert!(err.is_err());
        assert_eq!(d.revision(), rev);
    }

    #[test]
    fn gesture_merges_into_one_undo_step() {
        let (mut d, ch, _) = doc_with_channel();
        let steps = d.history().len();
        d.begin_gesture();
        for v in [0.1, 0.2, 0.3, 0.4] {
            d.execute(Command::SetChannelVolume { channel: ch, volume: v }).unwrap();
        }
        d.end_gesture();
        assert_eq!(d.history().len(), steps + 1);
        d.undo();
        assert_eq!(d.project().channels[0].volume, 0.78);
    }

    #[test]
    fn modified_tracks_saves() {
        let (mut d, _, _) = doc_with_channel();
        assert!(d.is_modified());
        d.mark_saved();
        assert!(!d.is_modified());
        d.execute(Command::SetTempo(Tempo::from_bpm(90.0))).unwrap();
        assert!(d.is_modified());
        d.undo();
        assert!(!d.is_modified());
    }

    #[test]
    fn step_toggle_off_removes_note() {
        let (mut d, ch, pat) = doc_with_channel();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: ch,
            step: 0,
            on: true,
        })
        .unwrap();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: ch,
            step: 0,
            on: false,
        })
        .unwrap();
        assert!(d.project().patterns[0].notes.is_empty());
    }

    #[test]
    fn notes_move_resize_and_duplicate() {
        let (mut d, ch, pat) = doc_with_channel();
        let id = d
            .execute(Command::AddNote {
                pattern: pat,
                note: NewNote {
                    channel: ch,
                    key: 64,
                    start: 960,
                    length: 480,
                    velocity: 0.5,
                },
            })
            .unwrap()
            .0
            .unwrap();
        let n = vec![NoteId(id)];
        d.execute(Command::MoveNotes {
            pattern: pat,
            notes: n.clone(),
            dt: -2000,
            dkey: 100,
        })
        .unwrap();
        let note = d.project().patterns[0].notes[0];
        assert_eq!((note.start, note.key), (0, 127));
        d.execute(Command::ResizeNotes {
            pattern: pat,
            notes: n.clone(),
            dlen: -1000,
        })
        .unwrap();
        assert_eq!(d.project().patterns[0].notes[0].length, 1);
        d.execute(Command::DuplicateNotes { pattern: pat, notes: n }).unwrap();
        let notes = &d.project().patterns[0].notes;
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[1].start, 960);
    }

    #[test]
    fn clips_are_linked_until_made_unique() {
        let (mut d, ch, pat) = doc_with_channel();
        let a = ClipId(
            d.execute(Command::AddClip {
                pattern: pat,
                track: 0,
                start: 0,
                length: None,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let b = ClipId(
            d.execute(Command::AddClip {
                pattern: pat,
                track: 1,
                start: 3840,
                length: None,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        assert_eq!(
            d.project().clip(a).unwrap().pattern(),
            d.project().clip(b).unwrap().pattern()
        );
        let new_pat = PatternId(d.execute(Command::MakeClipUnique(b)).unwrap().0.unwrap());
        d.execute(Command::SetStep {
            pattern: new_pat,
            channel: ch,
            step: 3,
            on: true,
        })
        .unwrap();
        assert!(d.project().pattern(pat).unwrap().notes.is_empty());
        assert_eq!(d.project().pattern(new_pat).unwrap().notes.len(), 1);
        assert_eq!(d.project().clip(b).unwrap().pattern(), new_pat);
    }

    #[test]
    fn clip_duplicate_and_move() {
        let (mut d, _, pat) = doc_with_channel();
        let a = ClipId(
            d.execute(Command::AddClip {
                pattern: pat,
                track: 0,
                start: 0,
                length: None,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        d.execute(Command::DuplicateClips(vec![a])).unwrap();
        let clips = &d.project().playlist.clips;
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[1].start, 3840);
        d.execute(Command::MoveClips {
            clips: vec![a],
            dt: 960,
            dtrack: 99,
        })
        .unwrap();
        let c = d.project().clip(a).unwrap();
        assert_eq!((c.start, c.track), (960, 15));
    }

    #[test]
    fn quantize_moves_starts_by_strength_and_rounds_lengths() {
        let (mut d, ch, pat) = doc_with_channel();
        let mut ids = Vec::new();
        for (start, length) in [(250u64, 100u64), (470, 300), (10, 239)] {
            let id = d
                .execute(Command::AddNote {
                    pattern: pat,
                    note: NewNote {
                        channel: ch,
                        key: 60,
                        start,
                        length,
                        velocity: 0.8,
                    },
                })
                .unwrap()
                .0
                .unwrap();
            ids.push(NoteId(id));
        }
        d.execute(Command::Quantize {
            pattern: pat,
            notes: ids[..2].to_vec(),
            grid: STEP_TICKS,
            strength: 0.5,
            lengths: false,
        })
        .unwrap();
        let starts: Vec<Tick> = d.project().patterns[0].notes.iter().map(|n| n.start).collect();
        assert_eq!(
            starts,
            vec![10, 245, 475],
            "halfway to 240 and 480; the third note untouched"
        );
        d.execute(Command::Quantize {
            pattern: pat,
            notes: ids.clone(),
            grid: STEP_TICKS,
            strength: 1.0,
            lengths: true,
        })
        .unwrap();
        let notes: Vec<(Tick, Tick)> = d.project().patterns[0]
            .notes
            .iter()
            .map(|n| (n.start, n.length))
            .collect();
        assert_eq!(notes, vec![(0, 240), (240, 240), (480, 240)]);
        assert!(
            d.execute(Command::Quantize {
                pattern: pat,
                notes: ids,
                grid: 0,
                strength: 1.0,
                lengths: false
            })
            .is_err()
        );
    }

    #[test]
    fn swing_is_clamped_and_undoable() {
        let (mut d, _, _) = doc_with_channel();
        d.execute(Command::SetSwing(3.0)).unwrap();
        assert_eq!(d.project().swing, 1.0);
        assert!(d.execute(Command::SetSwing(f32::NAN)).is_err());
        d.undo();
        assert_eq!(d.project().swing, 0.0);
    }

    #[test]
    fn removing_channel_removes_its_notes() {
        let (mut d, ch, pat) = doc_with_channel();
        d.execute(Command::SetStep {
            pattern: pat,
            channel: ch,
            step: 1,
            on: true,
        })
        .unwrap();
        d.execute(Command::RemoveChannel(ch)).unwrap();
        assert!(d.project().patterns[0].notes.is_empty());
    }

    #[test]
    fn new_channels_take_free_inserts() {
        let (mut d, _, _) = doc_with_channel();
        assert_eq!(d.project().channels[0].mixer_track, 1);
        d.execute(Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams::default()),
        })
        .unwrap();
        assert_eq!(d.project().channels[1].mixer_track, 2);
    }
}
