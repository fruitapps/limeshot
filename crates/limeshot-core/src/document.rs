//! A project plus its undo history and modified state.

use crate::command::{Command, CommandError, Created};
use crate::model::{ChannelId, Project};

/// Where a hosted plugin runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PluginTarget {
    Channel(ChannelId),
    Effect { track: u16, effect: u32 },
}

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
    /// Increments when the whole project is replaced (new/open); ids may repeat across generations.
    generation: u64,
    /// Identifies the current content: undoing back to a saved state makes it unmodified again.
    state: u64,
    next_state: u64,
    saved_state: u64,
    /// While a gesture is open, commands with the same merge key share one undo step.
    gesture_open: bool,
    /// A compound gesture (`Some`): every command joins one undo step, whatever its merge key.
    /// `true` once that step exists.
    compound: Option<bool>,
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
            generation: 0,
            state: 0,
            next_state: 1,
            saved_state: 0,
            gesture_open: false,
            compound: None,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn is_modified(&self) -> bool {
        self.state != self.saved_state
    }

    pub fn mark_saved(&mut self) {
        self.saved_state = self.state;
    }

    /// Treat the current content as unsaved (a recovered autosave).
    pub fn mark_unsaved(&mut self) {
        self.saved_state = u64::MAX;
    }

    fn new_state(&mut self) -> u64 {
        let s = self.next_state;
        self.next_state += 1;
        s
    }

    /// Replace the whole project (new/open). Clears the history.
    pub fn replace(&mut self, project: Project) {
        self.project = project;
        self.generation += 1;
        self.undo.clear();
        self.redo.clear();
        self.revision += 1;
        self.state = self.new_state();
        self.saved_state = self.state;
        self.gesture_open = false;
        self.compound = None;
    }

    /// Store the current state of a running plugin (read from the plugin before saving). This is
    /// not an edit: it makes no undo step and does not change the revision, because the sound
    /// does not change. Returns false if `target` has no plugin.
    pub fn store_plugin_state(&mut self, target: PluginTarget, state: crate::PluginState) -> bool {
        self.plugin_mut(target).map(|p| p.state = state).is_some()
    }

    /// Store where a plugin's window is (before saving). Like [`Document::store_plugin_state`]
    /// this is not an edit. Returns false if `target` has no plugin.
    pub fn store_plugin_window(&mut self, target: PluginTarget, window: Option<crate::PluginWindow>) -> bool {
        self.plugin_mut(target).map(|p| p.window = window).is_some()
    }

    fn plugin_mut(&mut self, target: PluginTarget) -> Option<&mut crate::PluginRef> {
        let p = &mut self.project;
        match target {
            PluginTarget::Channel(id) => {
                p.channels
                    .iter_mut()
                    .find(|c| c.id == id)
                    .and_then(|c| match &mut c.instrument {
                        crate::Instrument::Plugin(p) => Some(p),
                        _ => None,
                    })
            }
            PluginTarget::Effect { track, effect } => p
                .mixer
                .tracks
                .get_mut(usize::from(track))
                .and_then(|t| t.effects.iter_mut().find(|e| e.id == effect))
                .and_then(|e| e.plugin.as_mut()),
        }
    }

    /// Start a continuous gesture (a drag). Until [`Document::end_gesture`], consecutive commands
    /// with equal merge keys become one undo step.
    pub fn begin_gesture(&mut self) {
        self.gesture_open = true;
        self.compound = None;
    }

    /// Start a gesture whose commands all become one undo step, of whatever kind (placing a clip
    /// and dragging it, duplicating clips and moving the copies). Ends with `end_gesture`.
    pub fn begin_compound_gesture(&mut self) {
        self.gesture_open = true;
        self.compound = Some(false);
    }

    pub fn end_gesture(&mut self) {
        self.gesture_open = false;
        self.compound = None;
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
        let merges = (self.compound == Some(true) && !self.undo.is_empty() && self.redo.is_empty())
            || self.gesture_open
                && merge.is_some()
                && self
                    .undo
                    .last()
                    .is_some_and(|s| s.merge == merge && self.redo.is_empty());
        if self.compound.is_some() {
            self.compound = Some(true);
        }
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
        self.compound = None;
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
    use crate::effects::EffectKind;
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
    fn a_compound_gesture_is_one_undo_step() {
        let (mut d, ch, _) = doc_with_channel();
        let steps = d.history().len();
        d.begin_compound_gesture();
        d.execute(Command::SetChannelVolume {
            channel: ch,
            volume: 0.1,
        })
        .unwrap();
        d.execute(Command::SetChannelPan { channel: ch, pan: 0.5 }).unwrap();
        d.end_gesture();
        assert_eq!(d.history().len(), steps + 1);
        d.execute(Command::SetChannelPan { channel: ch, pan: -0.5 }).unwrap();
        assert_eq!(d.history().len(), steps + 2, "later commands make their own steps");
        d.undo();
        d.undo();
        assert_eq!(
            (d.project().channels[0].volume, d.project().channels[0].pan),
            (0.78, 0.0)
        );
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
    fn markers_are_added_sorted_renamed_moved_and_removed() {
        let mut d = Document::default();
        let id = |c: Created| MarkerId(c.0.unwrap());
        let b = id(d
            .execute(Command::AddMarker {
                tick: 7680,
                name: "Chorus".into(),
            })
            .unwrap());
        let a = id(d
            .execute(Command::AddMarker {
                tick: 0,
                name: "Intro".into(),
            })
            .unwrap());
        let names =
            |d: &Document| -> Vec<String> { d.project().playlist.markers.iter().map(|m| m.name.clone()).collect() };
        assert_eq!(names(&d), ["Intro", "Chorus"], "sorted by time");
        d.execute(Command::RenameMarker {
            marker: b,
            name: "Drop".into(),
        })
        .unwrap();
        d.begin_gesture();
        for t in [100, 200, 9000] {
            d.execute(Command::MoveMarker { marker: a, tick: t }).unwrap();
        }
        d.end_gesture();
        assert_eq!(names(&d), ["Drop", "Intro"]);
        d.undo();
        assert_eq!(
            d.project().playlist.markers[0].tick,
            0,
            "a marker drag is one undo step"
        );
        d.execute(Command::RemoveMarker(b)).unwrap();
        assert!(d.execute(Command::RemoveMarker(b)).is_err());
        assert_eq!(names(&d), ["Intro"]);
    }

    #[test]
    fn slice_glue_and_mute_notes() {
        let (mut d, ch, pat) = doc_with_channel();
        let add = |d: &mut Document, key: u8, start: Tick, length: Tick| {
            NoteId(
                d.execute(Command::AddNote {
                    pattern: pat,
                    note: NewNote {
                        channel: ch,
                        key,
                        start,
                        length,
                        velocity: 0.5,
                    },
                })
                .unwrap()
                .0
                .unwrap(),
            )
        };
        let a = add(&mut d, 60, 0, 960);
        let other = add(&mut d, 64, 0, 240);
        // Slice at 480: `a` splits, the short note (ending at 240) is untouched.
        let right = d
            .execute(Command::SliceNotes {
                pattern: pat,
                notes: vec![a, other],
                at: 480,
            })
            .unwrap()
            .0
            .map(NoteId)
            .unwrap();
        let notes = |d: &Document| d.project().patterns[0].notes.clone();
        let spans: Vec<(u8, Tick, Tick)> = notes(&d).iter().map(|n| (n.key, n.start, n.length)).collect();
        assert_eq!(spans, vec![(60, 0, 480), (64, 0, 240), (60, 480, 480)]);
        assert_eq!(notes(&d).iter().find(|n| n.id == right).unwrap().start, 480);
        // Glue joins the touching pieces again into the first one; other keys stay apart.
        d.execute(Command::GlueNotes {
            pattern: pat,
            notes: vec![a, right, other],
        })
        .unwrap();
        let spans: Vec<(u8, Tick, Tick)> = notes(&d).iter().map(|n| (n.key, n.start, n.length)).collect();
        assert_eq!(spans, vec![(60, 0, 960), (64, 0, 240)]);
        assert_eq!(notes(&d)[0].id, a);
        // Notes that do not touch are not glued.
        let far = add(&mut d, 60, 2000, 100);
        d.execute(Command::GlueNotes {
            pattern: pat,
            notes: vec![a, far],
        })
        .unwrap();
        assert_eq!(notes(&d).len(), 3);
        d.execute(Command::SetNotesMuted {
            pattern: pat,
            notes: vec![far],
            muted: true,
        })
        .unwrap();
        assert!(notes(&d).iter().find(|n| n.id == far).unwrap().muted);
        assert!(!notes(&d).iter().find(|n| n.id == a).unwrap().muted);
        d.undo();
        assert!(notes(&d).iter().all(|n| !n.muted));
    }

    #[test]
    fn pattern_color_changes_in_one_undo_step() {
        let (mut d, _, pat) = doc_with_channel();
        let before = d.project().pattern(pat).unwrap().color;
        let color = PALETTE.iter().copied().find(|c| *c != before).unwrap();
        d.execute(Command::SetPatternColor { pattern: pat, color }).unwrap();
        assert_eq!(d.project().pattern(pat).unwrap().color, color);
        assert!(d.undo());
        assert_eq!(d.project().pattern(pat).unwrap().color, before);
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
        assert_eq!(d.project().clip(b).unwrap().pattern(), Some(new_pat));
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
    fn effect_chain_commands() {
        let (mut d, _, _) = doc_with_channel();
        let a = d
            .execute(Command::AddEffect {
                track: 1,
                kind: EffectKind::Eq3,
            })
            .unwrap()
            .0
            .unwrap();
        let b = d
            .execute(Command::AddEffect {
                track: 1,
                kind: EffectKind::Reverb,
            })
            .unwrap()
            .0
            .unwrap();
        d.execute(Command::MoveEffect {
            track: 1,
            effect: b,
            delta: -5,
        })
        .unwrap();
        let ids: Vec<u32> = d.project().mixer.tracks[1].effects.iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![b, a]);
        d.execute(Command::SetEffectParam {
            track: 1,
            effect: a,
            index: 0,
            value: 99.0,
        })
        .unwrap();
        assert_eq!(
            d.project().mixer.tracks[1].effects[1].params[0],
            18.0,
            "clamped to the spec"
        );
        assert!(
            d.execute(Command::SetEffectParam {
                track: 1,
                effect: a,
                index: 40,
                value: 0.0
            })
            .is_err()
        );
        d.execute(Command::SetEffectEnabled {
            track: 1,
            effect: a,
            enabled: false,
        })
        .unwrap();
        assert!(!d.project().mixer.tracks[1].effects[1].enabled);
        d.execute(Command::RemoveEffect { track: 1, effect: b }).unwrap();
        assert_eq!(d.project().mixer.tracks[1].effects.len(), 1);
        for _ in 0..crate::effects::MAX_EFFECTS - 1 {
            d.execute(Command::AddEffect {
                track: 1,
                kind: EffectKind::Utility,
            })
            .unwrap();
        }
        assert!(
            d.execute(Command::AddEffect {
                track: 1,
                kind: EffectKind::Utility
            })
            .is_err(),
            "slot limit"
        );
    }

    #[test]
    fn routing_refuses_feedback_and_orders_tracks() {
        let (mut d, _, _) = doc_with_channel();
        assert_eq!(d.project().mixer.tracks[1].routes, vec![Route { to: 0, level: 1.0 }]);
        assert!(d.project().mixer.tracks[0].routes.is_empty());
        // 1 → 2 (a bus), 2 → master already.
        d.execute(Command::SetRoute {
            from: 1,
            to: 2,
            level: Some(0.5),
        })
        .unwrap();
        d.execute(Command::SetRoute {
            from: 1,
            to: 0,
            level: None,
        })
        .unwrap();
        assert_eq!(d.project().mixer.tracks[1].routes, vec![Route { to: 2, level: 0.5 }]);
        assert!(
            d.execute(Command::SetRoute {
                from: 2,
                to: 1,
                level: Some(1.0)
            })
            .is_err(),
            "2 → 1 would loop"
        );
        assert!(
            d.execute(Command::SetRoute {
                from: 3,
                to: 3,
                level: Some(1.0)
            })
            .is_err(),
            "self"
        );
        assert!(
            d.execute(Command::SetRoute {
                from: 0,
                to: 3,
                level: Some(1.0)
            })
            .is_err(),
            "master has no routes"
        );
        d.execute(Command::SetRoute {
            from: 3,
            to: 1,
            level: Some(1.0),
        })
        .unwrap();
        let order = d.project().mixer.processing_order();
        let pos = |t| order.iter().position(|x| *x == t).unwrap();
        assert!(pos(3) < pos(1) && pos(1) < pos(2), "{order:?}");
        assert_eq!(order.len(), d.project().mixer.tracks.len() - 1);
    }

    #[test]
    fn sanitize_breaks_routing_loops_from_files() {
        let mut p = Project::new();
        p.mixer.tracks[1].routes = vec![Route { to: 2, level: 1.0 }];
        p.mixer.tracks[2].routes = vec![Route { to: 1, level: 1.0 }, Route { to: 99, level: 1.0 }];
        p.mixer.tracks[0].routes = vec![Route { to: 3, level: 1.0 }];
        p.sanitize();
        assert!(p.mixer.tracks[0].routes.is_empty());
        assert_eq!(p.mixer.tracks[1].routes.len(), 1);
        assert!(
            p.mixer.tracks[2].routes.is_empty(),
            "the loop-closing and out-of-range routes are dropped"
        );
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

    #[test]
    fn audio_clips_reuse_their_channel_and_follow_it() {
        let mut d = Document::default();
        let wav = SampleRef {
            path: "loop.wav".into(),
        };
        let add = |d: &mut Document, start| {
            ClipId(
                d.execute(Command::AddAudioClip {
                    sample: wav.clone(),
                    name: "loop".into(),
                    track: 0,
                    start,
                    length: 1920,
                })
                .unwrap()
                .0
                .unwrap(),
            )
        };
        let a = add(&mut d, 0);
        let b = add(&mut d, 3840);
        assert_eq!(
            d.project().channels.len(),
            1,
            "the second drop reuses the audio clip channel"
        );
        let ch = d.project().clip(a).unwrap().audio_channel().unwrap();
        assert_eq!(d.project().clip(b).unwrap().audio_channel(), Some(ch));
        match &d.project().channel(ch).unwrap().instrument {
            Instrument::Sampler(s) => assert!(!s.one_shot && s.sample.as_ref() == Some(&wav)),
            other => panic!("{other:?}"),
        }
        assert!(d.execute(Command::MakeClipUnique(a)).is_err());
        // Paste places a copy of any clip kind.
        let c = d.project().clip(a).unwrap().clone();
        d.execute(Command::PlaceClip {
            source: c.source,
            track: 1,
            start: 7680,
            length: c.length,
            offset: 240,
        })
        .unwrap();
        assert_eq!(d.project().playlist.clips.len(), 3);
        assert!(
            d.execute(Command::PlaceClip {
                source: ClipSource::Audio(ChannelId(999)),
                track: 0,
                start: 0,
                length: 1,
                offset: 0,
            })
            .is_err()
        );
        // Deleting the channel deletes its clips; undo brings both back.
        d.execute(Command::RemoveChannel(ch)).unwrap();
        assert!(d.project().playlist.clips.is_empty());
        d.undo();
        assert_eq!(d.project().playlist.clips.len(), 3);
    }

    #[test]
    fn trimming_a_clip_start_moves_its_offset() {
        let (mut d, _, pat) = doc_with_channel();
        let c = ClipId(
            d.execute(Command::AddClip {
                pattern: pat,
                track: 0,
                start: 3840,
                length: Some(3840),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let get = |d: &Document| {
            let c = d.project().clip(c).unwrap();
            (c.start, c.offset, c.length)
        };
        d.execute(Command::ResizeClipStart { clip: c, start: 4800 }).unwrap();
        assert_eq!(get(&d), (4800, 960, 2880));
        // Revealing more than the source has stops at the source start.
        d.execute(Command::ResizeClipStart { clip: c, start: 0 }).unwrap();
        assert_eq!(get(&d), (3840, 0, 3840));
        // The end stays put; at least one tick remains.
        d.execute(Command::ResizeClipStart { clip: c, start: 99_999 }).unwrap();
        assert_eq!(get(&d), (7679, 3839, 1));
    }

    #[test]
    fn channel_groups_are_set_and_listed() {
        let (mut d, ch, _) = doc_with_channel();
        assert!(d.project().channel_groups().is_empty());
        d.execute(Command::SetChannelGroup {
            channels: vec![ch],
            group: Some("  Drums ".into()),
        })
        .unwrap();
        assert_eq!(d.project().channel(ch).unwrap().group.as_deref(), Some("Drums"));
        assert_eq!(d.project().channel_groups(), vec!["Drums".to_string()]);
        // A blank name means Unsorted; the group disappears with its last channel.
        d.execute(Command::SetChannelGroup {
            channels: vec![ch],
            group: Some(" ".into()),
        })
        .unwrap();
        assert!(d.project().channel_groups().is_empty());
        // Audio clips' channels start in the "Audio" group (FL Studio).
        let c = d
            .execute(Command::AddAudioClip {
                sample: SampleRef { path: "a.wav".into() },
                name: "a".into(),
                track: 0,
                start: 0,
                length: 960,
            })
            .unwrap()
            .0
            .unwrap();
        let ch = d.project().clip(ClipId(c)).unwrap().audio_channel().unwrap();
        assert_eq!(d.project().channel(ch).unwrap().group.as_deref(), Some("Audio"));
    }

    #[test]
    fn fill_and_rotate_steps() {
        let (mut d, ch, pat) = doc_with_channel();
        let steps = |d: &Document| -> Vec<Tick> {
            d.project().patterns[0]
                .notes_for(ch)
                .map(|n| n.start / STEP_TICKS)
                .collect()
        };
        d.execute(Command::SetStep {
            pattern: pat,
            channel: ch,
            step: 1,
            on: true,
        })
        .unwrap();
        // Fill replaces the channel's steps (observed in FL Studio 2026.1.1).
        d.execute(Command::FillSteps {
            pattern: pat,
            channel: ch,
            every: 4,
        })
        .unwrap();
        assert_eq!(steps(&d), vec![0, 4, 8, 12]);
        let ids: Vec<NoteId> = d.project().patterns[0].notes_for(ch).map(|n| n.id).collect();
        let rotate = |d: &mut Document, by: i64| {
            d.execute(Command::RotateNotes {
                pattern: pat,
                notes: ids.clone(),
                by: by * STEP_TICKS as i64,
            })
            .unwrap();
        };
        rotate(&mut d, 1);
        assert_eq!(steps(&d), vec![1, 5, 9, 13]);
        // Rotating left wraps the first step round to the end of the pattern.
        rotate(&mut d, -2);
        assert_eq!(steps(&d), vec![3, 7, 11, 15]);
        rotate(&mut d, 1);
        assert_eq!(steps(&d), vec![0, 4, 8, 12]);
        d.execute(Command::FillSteps {
            pattern: pat,
            channel: ch,
            every: 2,
        })
        .unwrap();
        assert_eq!(steps(&d).len(), 8);
        assert!(
            d.execute(Command::FillSteps {
                pattern: pat,
                channel: ch,
                every: 0
            })
            .is_err()
        );
    }

    #[test]
    fn slip_and_slice_clips() {
        let (mut d, _, pat) = doc_with_channel();
        let add = |d: &mut Document, track: u32, start: Tick| {
            ClipId(
                d.execute(Command::AddClip {
                    pattern: pat,
                    track,
                    start,
                    length: Some(3840),
                })
                .unwrap()
                .0
                .unwrap(),
            )
        };
        let a = add(&mut d, 0, 0);
        let b = add(&mut d, 1, 1920);
        let span = |d: &Document, id: ClipId| {
            let c = d.project().clip(id).unwrap();
            (c.track, c.start, c.length, c.offset)
        };
        // Slip moves only the content.
        d.execute(Command::SlipClip { clip: a, offset: 480 }).unwrap();
        assert_eq!(span(&d, a), (0, 0, 3840, 480));
        // Slicing at 2880 cuts both clips; `b` starts 1920 later, so its piece starts deeper in.
        let right = ClipId(
            d.execute(Command::SliceClips {
                clips: vec![a, b],
                at: 2880,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        assert_eq!(span(&d, a), (0, 0, 2880, 480));
        assert_eq!(span(&d, b), (1, 1920, 960, 0));
        assert_eq!(span(&d, right), (1, 2880, 2880, 960));
        let pieces: Vec<_> = d.project().playlist.clips.iter().map(|c| span(&d, c.id)).collect();
        assert!(pieces.contains(&(0, 2880, 960, 3360)), "{pieces:?}");
        // A cut at a clip's edge does nothing.
        assert_eq!(
            d.execute(Command::SliceClips { clips: vec![a], at: 0 }).unwrap().0,
            None
        );
        // The no-op cut left no undo step: one undo restores the uncut clips.
        assert!(d.undo());
        assert_eq!(d.project().playlist.clips.len(), 2);
        assert_eq!(span(&d, a), (0, 0, 3840, 480));
    }

    #[test]
    fn sanitize_drops_audio_clips_without_a_channel() {
        let mut p = Project::new();
        p.playlist.clips.push(Clip {
            id: ClipId(5),
            source: ClipSource::Audio(ChannelId(77)),
            track: 0,
            start: 0,
            length: 10,
            offset: 0,
            muted: false,
            extra: Extra::new(),
        });
        let json = serde_json::to_string(&p.playlist.clips[0]).unwrap();
        assert!(json.contains("\"Audio\":77"), "{json}");
        p.sanitize();
        assert!(p.playlist.clips.is_empty());
    }
}
