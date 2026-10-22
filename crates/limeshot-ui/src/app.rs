//! Application state and the glue between the document, the engine and the panels.
//!
//! State is kept in separate layers (see `docs/architecture/ui.md`):
//! * persistent project state: `doc` (a [`Document`]; changed only through commands),
//! * application configuration: `settings`,
//! * engine/transport state: owned by the audio thread, mirrored through `player.status()`,
//! * runtime resources: `samples` (decoded audio),
//! * ephemeral UI state: `ui` (visible panels, selections, zoom, the current pattern).
//!
//! Layout (FL Studio's window model, ArtCraft-like styling): header + toolbar on top, the Browser
//! docked left, and the workspace with the editor windows (Playlist maximised by default, Channel
//! Rack on top), which can be moved, maximised and detached (see `workspace.rs`).

use crate::settings::Settings;
use crate::{browser, channel_rack, dialogs, mixer, piano_roll, playlist, shortcuts, toolbar};
use egui::{Context, Response};
use limeshot_audio::assets::SampleCache;
use limeshot_audio::{Msg, Player};
use limeshot_core::{ChannelId, Command, Document, PatternId, Project, Snap, Tick};
use limeshot_engine::PlayMode;
use limeshot_sequencer::{Snapshot, compile};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Which editor last had the user's attention; keyboard edits (Delete, Ctrl+A …) go there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    None,
    ChannelRack,
    PianoRoll,
    Playlist,
    Mixer,
}

pub struct UiState {
    pub show_browser: bool,
    /// The editor windows (FL Studio's window model).
    pub workspace: crate::workspace::Workspace,
    pub current_pattern: PatternId,
    /// What a click on empty Playlist space places: an automation picked in the clip picker, or
    /// (`None`) the current pattern.
    pub place: Option<limeshot_core::ClipSource>,
    /// The current channel: the Piano Roll and Channel Settings edit it.
    pub selected_channel: Option<ChannelId>,
    /// Channels whose selector LED is lit in the Channel Rack (FL Studio's channel selection);
    /// moving, deleting and routing act on all of them.
    pub channel_selection: Vec<ChannelId>,
    pub snap: Snap,
    pub mode: PlayMode,
    pub song_loop: bool,
    pub metronome: bool,
    pub focus: Focus,
    /// The editors' description of what is under the pointer (the hint panel's second line).
    pub hint: String,
    /// The hint panel's rectangle in the toolbar, filled once everything else is drawn.
    pub hint_rect: Option<egui::Rect>,
    /// The description the hint panel showed in the last frame (for tests).
    pub hint_shown: String,
    pub rack: channel_rack::State,
    pub piano: piano_roll::State,
    pub playlist: playlist::State,
    pub mixer: mixer::State,
    pub browser: browser::State,
    /// Notes the Playback tool's scrub line is sounding: (channel, key).
    pub scrub: Vec<(ChannelId, u8)>,
    /// The settings window, while open.
    pub settings: Option<crate::settings_window::State>,
    /// Choices from the toolbar's right-click menus, and the tempo tapper.
    pub toolbar: crate::toolbar::State,
    /// The record button and the take being recorded.
    pub record: crate::recording::State,
}

impl UiState {
    fn new(project: &Project) -> Self {
        Self {
            settings: None,
            toolbar: Default::default(),
            record: Default::default(),
            show_browser: true,
            workspace: Default::default(),
            current_pattern: project.patterns.first().map(|p| p.id).unwrap_or(PatternId(0)),
            place: None,
            selected_channel: project.channels.first().map(|c| c.id),
            channel_selection: project.channels.first().map(|c| c.id).into_iter().collect(),
            snap: Snap::Step,
            mode: PlayMode::Pattern,
            song_loop: true,
            metronome: false,
            focus: Focus::None,
            hint: String::new(),
            hint_rect: None,
            hint_shown: String::new(),
            rack: Default::default(),
            piano: Default::default(),
            playlist: Default::default(),
            mixer: Default::default(),
            browser: Default::default(),
            scrub: Vec::new(),
        }
    }
}

/// Edit › Cut / Copy / Paste (Cmd+X / C / V), applied to the focused editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipOp {
    Cut,
    Copy,
    Paste,
}

/// What Cmd+C / Cmd+X put on the (app-internal) clipboard.
#[derive(Debug, Clone, PartialEq)]
pub enum Clipboard {
    /// Notes without a channel: (key, start, length, velocity). Pasted at the same times into the
    /// channel being edited, as FL Studio does.
    Notes(Vec<(u8, Tick, Tick, f32)>),
    /// Clips, pasted at the song position in Song mode (so the earliest clip lands there),
    /// otherwise at their original times.
    Clips(Vec<CopiedClip>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopiedClip {
    pub source: limeshot_core::ClipSource,
    pub track: u32,
    pub start: Tick,
    pub length: Tick,
    pub offset: Tick,
}

/// A file action waiting for "save changes?" to be answered.
#[derive(Debug, Clone, PartialEq)]
pub enum Pending {
    New,
    Open(Option<PathBuf>),
    Quit,
}

pub struct Toast {
    pub text: String,
    pub error: bool,
    pub until: Instant,
}

pub struct LimeShotApp {
    pub doc: Document,
    pub path: Option<PathBuf>,
    pub player: Player,
    pub samples: SampleCache,
    /// The plugin database and the project's running plugins.
    pub plugins: crate::plugins::PluginHost,
    pub settings: Settings,
    pub ui: UiState,
    pub dialog: Option<dialogs::Dialog>,
    pub toasts: Vec<Toast>,
    pub clipboard: Option<Clipboard>,
    /// A copied control position (0–1), independent of the notes/clips clipboard.
    pub control_clipboard: Option<f32>,
    /// Folder of bundled samples, shown first in the Browser.
    pub bundled_samples: Option<PathBuf>,
    pub export_job: Option<dialogs::ExportJob>,
    compiled: Option<(u64, PatternId)>,
    /// The Song loop range last sent to the engine (`None`: nothing sent yet).
    loop_sent: Option<Option<(Tick, Tick)>>,
    /// Effect processors already handed to the audio thread, and for which document generation.
    sent_effects: std::collections::HashSet<u32>,
    effects_generation: u64,
    snapshot: Arc<Snapshot>,
    pub allow_close: bool,
    themed: bool,
    /// Frames drawn since the theme (and its fonts) was installed; fonts apply one frame later.
    themed_frames: u32,
    pub meter_hold: Vec<(f32, f32)>,
    pub clip_until: Option<Instant>,
    /// The toolbar's CPU readout, updated once per second.
    pub cpu: crate::toolbar::CpuReadout,
    pub autosave: crate::autosave::Autosave,
    /// Write settings changes to the config folder (off for headless apps, so tests leave the
    /// user's settings alone).
    pub persist_settings: bool,
    /// Where users' effect and instrument presets live (`None` in headless apps: saving is off).
    pub presets_dir: Option<PathBuf>,
    /// Auditioned notes still sounding: (channel, key). Every audition is held by a mouse
    /// button, so any left once no button is down are released (`logic`).
    pub auditions: Vec<(ChannelId, u8)>,
    pub typing_notes: Vec<(egui::Key, ChannelId, u8)>,
    pub keyboard_notes: bool,
    /// The open MIDI input (settings' `midi_input`) and the notes it holds, with the channel each
    /// started on (so a note ends where it began when the selection changes meanwhile).
    pub midi: Option<limeshot_audio::midi::MidiIn>,
    pub midi_notes: Vec<(u8, ChannelId)>,
    midi_opened: bool,
    pub sampler_preview: Option<(ChannelId, std::time::Instant, f64)>,
}

impl LimeShotApp {
    pub fn new(settings: Settings, bundled_samples: Option<PathBuf>) -> Self {
        let audio = settings.audio.clone();
        let mut app = Self::new_with_player(settings, bundled_samples, |s| Player::start(&audio, s));
        app.plugins = crate::plugins::PluginHost::new(crate::plugins::PluginHost::default_db_path());
        app.send(Msg::SongLoop(true));
        app.autosave = crate::autosave::Autosave::new(crate::autosave::default_dir());
        app.persist_settings = true;
        app.presets_dir = crate::settings::config_dir().map(|d| d.join("presets"));
        app
    }

    /// Open the MIDI input named in the settings (none: close it). `ctx` is woken by each
    /// message so notes play without waiting for the next frame.
    pub fn open_midi_input(&mut self, ctx: &Context) {
        self.midi_all_off();
        self.midi = None;
        let Some(name) = self.settings.midi_input.clone() else {
            return;
        };
        let ctx = ctx.clone();
        match limeshot_audio::midi::MidiIn::open(&name, move || ctx.request_repaint()) {
            Ok(m) => {
                self.hint(format!("MIDI input: {name}"));
                self.midi = Some(m);
            }
            Err(e) => self.toast_error(e),
        }
    }

    /// A MIDI message from the input device (or a test): notes play the selected channel.
    pub fn midi_event(&mut self, e: limeshot_audio::midi::MidiEvent) {
        use limeshot_audio::midi::MidiEvent;
        match e {
            MidiEvent::NoteOn { key, velocity } => {
                let Some(channel) = self.selected_channel() else { return };
                let velocity = if self.settings.midi_velocity { velocity } else { 0.78 };
                if let Some(i) = self.midi_notes.iter().position(|n| n.0 == key) {
                    let (_, old) = self.midi_notes.remove(i);
                    self.preview_off(old, key);
                }
                self.preview_on(channel, key, velocity);
                // Held by the key, not the mouse: keep it out of the pointer's auditions.
                self.auditions.retain(|a| *a != (channel, key));
                self.midi_notes.push((key, channel));
                crate::recording::note_on(self, channel, key, velocity);
            }
            MidiEvent::NoteOff { key } => {
                if let Some(i) = self.midi_notes.iter().position(|n| n.0 == key) {
                    let (_, channel) = self.midi_notes.remove(i);
                    self.preview_off(channel, key);
                    crate::recording::note_off(self, channel, key);
                }
            }
            MidiEvent::Control { .. } | MidiEvent::PitchBend(_) => {}
        }
    }

    /// Release every note the MIDI input holds.
    pub fn midi_all_off(&mut self) {
        for (key, channel) in std::mem::take(&mut self.midi_notes) {
            self.preview_off(channel, key);
        }
    }

    /// For tests and the headless path: an app with no audio device.
    pub fn new_silent(settings: Settings, bundled_samples: Option<PathBuf>) -> Self {
        let mut app = Self::new_with_player(settings, bundled_samples, |s| Player::silent("headless".into(), s));
        app.send(Msg::SongLoop(true));
        app
    }

    /// Create an empty app using `player` to start playback from its initial snapshot.
    /// Settings persistence, autosave storage, and user preset storage start disabled.
    fn new_with_player(
        settings: Settings,
        bundled_samples: Option<PathBuf>,
        player: impl FnOnce(Arc<Snapshot>) -> Player,
    ) -> Self {
        let doc = Document::default();
        let snapshot = Arc::new(compile(
            doc.project(),
            doc.project().patterns[0].id,
            &limeshot_sequencer::NoSamples,
        ));
        let ui = UiState::new(doc.project());
        Self {
            player: player(snapshot.clone()),
            doc,
            path: None,
            samples: SampleCache::new(),
            plugins: crate::plugins::PluginHost::new(None),
            settings,
            ui,
            dialog: None,
            toasts: Vec::new(),
            clipboard: None,
            control_clipboard: None,
            bundled_samples,
            export_job: None,
            compiled: None,
            loop_sent: None,
            sent_effects: Default::default(),
            effects_generation: 0,
            snapshot,
            allow_close: false,
            themed: false,
            themed_frames: 0,
            meter_hold: Vec::new(),
            clip_until: None,
            cpu: Default::default(),
            autosave: Default::default(),
            persist_settings: false,
            presets_dir: None,
            auditions: Vec::new(),
            sampler_preview: None,
            keyboard_notes: false,
            typing_notes: Vec::new(),
            midi: None,
            midi_notes: Vec::new(),
            midi_opened: false,
        }
    }

    pub fn project(&self) -> &Project {
        self.doc.project()
    }

    /// UI state for a newly opened project, keeping the browser setting (windows reset to
    /// FL Studio's new-project layout).
    fn fresh_ui_state(&self) -> UiState {
        UiState {
            show_browser: self.ui.show_browser,
            ..UiState::new(self.doc.project())
        }
    }

    /// Open a workspace window and bring it to the front.
    pub fn show_window(&mut self, w: crate::workspace::Win) {
        self.ui.workspace.show(w);
        self.ui.focus = w.focus();
    }

    /// F5/F6/F7/F9 behaviour: open a closed window, close an open one.
    pub fn toggle_window(&mut self, w: crate::workspace::Win) {
        self.ui.workspace.toggle(w);
        if self.ui.workspace.is_open(w) {
            self.ui.focus = w.focus();
        }
    }

    pub fn is_window_open(&self, w: crate::workspace::Win) -> bool {
        self.ui.workspace.is_open(w)
    }

    /// Make `id` the current channel and the only selected one (the Piano Roll and Channel
    /// Settings follow it).
    pub fn select_channel(&mut self, id: ChannelId) {
        self.ui.selected_channel = Some(id);
        self.ui.channel_selection = vec![id];
    }

    /// Add `id` to the channel selection or remove it (FL Studio: Ctrl-, Shift- or right-click on
    /// a selector LED). An added channel becomes the current one.
    pub fn toggle_channel_selection(&mut self, id: ChannelId) {
        if let Some(i) = self.ui.channel_selection.iter().position(|c| *c == id) {
            self.ui.channel_selection.remove(i);
        } else {
            self.ui.channel_selection.push(id);
            self.ui.selected_channel = Some(id);
        }
    }

    /// The selected channels in rack order. Falls back to the current channel when no LED is lit.
    pub fn selected_channels(&self) -> Vec<ChannelId> {
        let sel: Vec<ChannelId> = self
            .project()
            .channels
            .iter()
            .map(|c| c.id)
            .filter(|c| self.ui.channel_selection.contains(c))
            .collect();
        if sel.is_empty() {
            self.selected_channel().into_iter().collect()
        } else {
            sel
        }
    }

    pub fn send(&mut self, msg: Msg) {
        if !self.player.send(msg) {
            log::warn!("audio queue full; message dropped");
        }
    }

    /// Execute a command; failures become an error toast. Returns the created id, if any.
    pub fn exec(&mut self, cmd: Command) -> Option<u32> {
        match self.doc.execute(cmd) {
            Ok(c) => c.0,
            Err(e) => {
                self.toast_error(e.to_string());
                None
            }
        }
    }

    /// Execute a command coming from a continuous control: drags merge into one undo step.
    pub fn exec_gesture(&mut self, resp: &Response, cmd: Option<Command>) {
        if resp.drag_started() {
            self.doc.begin_gesture();
        }
        if let Some(cmd) = cmd {
            self.exec(cmd);
        }
        if resp.drag_stopped() {
            self.doc.end_gesture();
        }
    }

    pub fn toast(&mut self, text: impl Into<String>) {
        self.toasts.push(Toast {
            text: text.into(),
            error: false,
            until: Instant::now() + Duration::from_secs(3),
        });
    }

    pub fn toast_error(&mut self, text: impl Into<String>) {
        let text = text.into();
        log::warn!("{text}");
        self.toasts.push(Toast {
            text,
            error: true,
            until: Instant::now() + Duration::from_secs(6),
        });
    }

    pub fn hint(&mut self, text: impl Into<String>) {
        self.ui.hint = text.into();
    }

    /// The pattern being edited (always valid).
    pub fn current_pattern(&self) -> PatternId {
        let p = self.project();
        if p.pattern(self.ui.current_pattern).is_some() {
            self.ui.current_pattern
        } else {
            p.patterns.first().map(|x| x.id).unwrap_or(PatternId(0))
        }
    }

    /// Select the pattern for editing and clear the Piano Roll's note selection.
    /// When enabled, auto zip recomputes compact rows on a change to a valid pattern.
    /// Invalid IDs are stored; [`Self::current_pattern`] supplies the editing fallback.
    pub fn set_current_pattern(&mut self, id: PatternId) {
        if self.settings.auto_zip_empty_channels && id != self.ui.current_pattern {
            crate::channel_rack::auto_zip(self, id);
        }
        self.ui.current_pattern = id;
        if self.project().pattern(id).is_some() {
            self.ui.rack.viewed_pattern = Some(id);
        }
        self.ui.piano.selection.clear();
    }

    pub fn selected_channel(&self) -> Option<ChannelId> {
        let p = self.project();
        match self.ui.selected_channel {
            Some(c) if p.channel(c).is_some() => Some(c),
            _ => p.channels.first().map(|c| c.id),
        }
    }

    // ----- transport -----

    /// Run Edit › Cut / Copy / Paste on the focused editor (FL Studio's Edit menu does the same).
    pub fn clipboard_op(&mut self, op: ClipOp) {
        match self.ui.focus {
            Focus::PianoRoll => crate::piano_roll::clipboard(self, op),
            Focus::Playlist => crate::playlist::clipboard(self, op),
            Focus::ChannelRack => crate::channel_rack::clipboard(self, op),
            Focus::Mixer | Focus::None => {}
        }
    }

    /// Whether Edit › Cut / Copy / Paste has something to act on in the focused editor.
    pub fn can_clipboard(&self, op: ClipOp) -> bool {
        let clip = &self.clipboard;
        match (self.ui.focus, op) {
            (Focus::PianoRoll, ClipOp::Paste) | (Focus::ChannelRack, ClipOp::Paste) => {
                matches!(clip, Some(Clipboard::Notes(n)) if !n.is_empty()) && self.selected_channel().is_some()
            }
            (Focus::Playlist, ClipOp::Paste) => matches!(clip, Some(Clipboard::Clips(c)) if !c.is_empty()),
            (Focus::PianoRoll, _) => !self.ui.piano.selection.is_empty(),
            (Focus::Playlist, _) => !self.ui.playlist.selection.is_empty(),
            (Focus::ChannelRack, _) => self.selected_channel().is_some(),
            (Focus::Mixer | Focus::None, _) => false,
        }
    }

    /// Save the settings to the config folder (only for the real app, see `persist_settings`).
    pub fn save_settings(&self) {
        if self.persist_settings {
            self.settings.save();
        }
    }

    /// Set playback following for the editors and save settings when persistence is enabled.
    pub fn set_auto_scroll(&mut self, on: bool) {
        self.settings.auto_scroll = on;
        self.save_settings();
    }

    pub fn is_playing(&self) -> bool {
        self.player.status().is_playing()
    }

    pub fn toggle_play(&mut self) {
        if self.is_playing() {
            self.send(Msg::Pause);
        } else {
            self.sync_engine();
            self.send(Msg::Play);
        }
    }

    /// Stop and return to where playback started, canceling pending Browser and sampler previews.
    /// Stopping while stopped also silences everything (see `Renderer::stop`) and rewinds to zero,
    /// or to the Playlist selection's start in Song mode.
    pub fn stop(&mut self) {
        self.ui.browser.cancel_preview();
        self.sampler_preview = None;
        let stopped = !self.is_playing();
        crate::recording::stopped(self);
        self.send(Msg::Stop);
        if stopped {
            let start = if self.ui.mode == PlayMode::Song {
                self.ui.playlist.time_sel.map_or(0, |r| r.0)
            } else {
                0
            };
            self.locate(start);
        }
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        if self.ui.mode != mode {
            self.ui.mode = mode;
            self.send(Msg::Mode(mode));
        }
    }

    pub fn toggle_mode(&mut self) {
        let m = if self.ui.mode == PlayMode::Pattern {
            PlayMode::Song
        } else {
            PlayMode::Pattern
        };
        self.set_mode(m);
    }

    pub fn set_metronome(&mut self, on: bool) {
        self.ui.metronome = on;
        self.send(Msg::Metronome(on));
    }

    pub fn set_song_loop(&mut self, on: bool) {
        self.ui.song_loop = on;
        self.send(Msg::SongLoop(on));
    }

    pub fn locate(&mut self, tick: Tick) {
        self.send(Msg::Locate(tick));
    }

    /// Audition a note on a channel (piano keys, step clicks).
    pub fn preview_on(&mut self, channel: ChannelId, key: u8, velocity: f32) {
        if let Some(i) = self
            .project()
            .channel_index(channel)
            .and_then(|i| u16::try_from(i).ok())
        {
            self.sync_engine();
            self.send(Msg::NoteOn {
                channel: i,
                key,
                velocity,
            });
            if !self.auditions.contains(&(channel, key)) {
                self.auditions.push((channel, key));
            }
        }
    }

    /// Queue a channel audition for `seconds`, independent of pointer release.
    /// Synchronizes the engine first; ignores a missing channel or an index that cannot fit in `u16`.
    /// Release timing and nonfinite durations follow [`limeshot_engine::Renderer::preview_note_for`].
    pub fn preview_for(&mut self, channel: ChannelId, key: u8, velocity: f32, seconds: f64) {
        if let Some(i) = self
            .project()
            .channel_index(channel)
            .and_then(|i| u16::try_from(i).ok())
        {
            self.sync_engine();
            self.send(Msg::NoteFor {
                channel: i,
                key,
                velocity,
                seconds,
            });
        }
    }

    pub fn preview_off(&mut self, channel: ChannelId, key: u8) {
        self.auditions.retain(|a| *a != (channel, key));
        if let Some(i) = self
            .project()
            .channel_index(channel)
            .and_then(|i| u16::try_from(i).ok())
        {
            self.send(Msg::NoteOff { channel: i, key });
        }
    }

    /// Rebuild and send the playback snapshot when the project, the current pattern or the loaded
    /// samples changed.
    pub fn sync_engine(&mut self) {
        let loaded = self.samples.poll();
        let key = (self.doc.revision(), self.current_pattern());
        let resync = self.plugins.take_needs_sync();
        if !loaded && !resync && self.compiled == Some(key) {
            return;
        }
        if self.compiled.map(|k| k.0) != Some(key.0) {
            let refs: Vec<String> = self.project().sample_refs().map(|r| r.path.clone()).collect();
            for r in refs {
                self.samples.request(&r);
            }
        }
        self.send_effect_processors();
        let snap = Arc::new(compile(self.doc.project(), key.1, &self.samples));
        self.snapshot = snap.clone();
        self.send(Msg::Snapshot(snap));
        self.compiled = Some(key);
    }

    /// Build processors for effects the audio thread does not have yet (allocation happens here,
    /// on the control thread) and send them ahead of the snapshot that uses them.
    fn send_effect_processors(&mut self) {
        if self.effects_generation != self.doc.generation() {
            self.effects_generation = self.doc.generation();
            self.sent_effects.clear();
            self.send(Msg::ClearEffects);
        }
        let sr = self.player.sample_rate();
        // Plugin slots get their processors from the plugin host below.
        let present: Vec<(u32, limeshot_core::EffectKind, Vec<f32>)> = self
            .project()
            .mixer
            .tracks
            .iter()
            .flat_map(|t| t.effects.iter())
            .filter(|e| e.kind != limeshot_core::EffectKind::Plugin)
            .map(|e| (e.id, e.kind, e.params.clone()))
            .collect();
        self.sent_effects.retain(|id| present.iter().any(|p| p.0 == *id));
        for (id, kind, params) in present {
            if self.sent_effects.insert(id) {
                self.send(Msg::Effect(id, limeshot_engine::fx::build(kind, sr, &params)));
            }
        }
        let player = &mut self.player;
        let mut send = |m: Msg| {
            if !player.send(m) {
                log::warn!("audio queue full; plugin processor dropped");
            }
        };
        self.plugins
            .sync(self.doc.project(), self.doc.generation(), sr, &mut send);
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }

    /// The value an automation is giving `target` right now (Song mode, playing), in the
    /// parameter's own units, so automated knobs and faders move with it (FL Studio).
    pub fn automated(&self, target: limeshot_core::AutomationTarget) -> Option<f32> {
        use limeshot_core::AutomationTarget as A;
        use limeshot_sequencer::LaneTarget as L;
        if self.ui.mode != PlayMode::Song || !self.is_playing() {
            return None;
        }
        let p = self.project();
        let lane = match target {
            A::ChannelVolume(c) => L::ChannelVolume(p.channel_index(c)?),
            A::ChannelPan(c) => L::ChannelPan(p.channel_index(c)?),
            A::MixerVolume(t) => L::MixerVolume(usize::from(t)),
            A::MixerPan(t) => L::MixerPan(usize::from(t)),
            A::EffectParam { track, effect, index } => L::EffectParam {
                track: usize::from(track),
                effect,
                index,
            },
            A::PluginParam { .. } => return None,
            A::Tempo => L::Tempo,
        };
        let pos = self.player.status().position();
        self.snapshot
            .automation
            .iter()
            .find(|l| l.target == lane)
            .and_then(|l| l.value_at(pos))
    }

    // ----- files -----

    pub fn window_title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.project().title.clone());
        format!("{}{name} — LimeShot", if self.doc.is_modified() { "• " } else { "" })
    }

    /// Run `action`, asking to save first if the project has unsaved changes.
    pub fn request(&mut self, action: Pending) {
        if self.doc.is_modified() {
            self.dialog = Some(dialogs::Dialog::ConfirmDiscard(action));
        } else {
            self.perform(action);
        }
    }

    pub fn perform(&mut self, action: Pending) {
        match action {
            Pending::New => self.new_project(),
            Pending::Open(Some(p)) => self.open_path(&p),
            Pending::Open(None) => {
                let dialog = rfd::FileDialog::new()
                    .add_filter("LimeShot or FL Studio project", &[limeshot_project::EXTENSION, "flp"])
                    .add_filter("LimeShot project", &[limeshot_project::EXTENSION])
                    .add_filter("FL Studio project", &["flp"]);
                if let Some(p) = dialog.pick_file() {
                    self.open_path(&p);
                }
            }
            Pending::Quit => self.allow_close = true,
        }
    }

    pub fn new_project(&mut self) {
        self.send(Msg::Stop);
        self.doc.replace(Project::new());
        self.path = None;
        self.samples.set_project_dir(None);
        self.ui = self.fresh_ui_state();
        self.set_mode(PlayMode::Pattern);
        self.ui.mode = PlayMode::Pattern;
        self.send(Msg::Mode(PlayMode::Pattern));
        self.compiled = None;
    }

    /// Open a `.lime` project, or import an FL Studio `.flp` as a new unsaved project.
    pub fn open_path(&mut self, path: &Path) {
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("flp")) {
            self.import_flp(path);
            return;
        }
        match limeshot_project::load(path) {
            Ok(project) => {
                self.send(Msg::Stop);
                // In memory, sample paths are absolute; they are made relative again on save.
                let project = absolutize_samples(&project, path.parent());
                self.samples.set_project_dir(path.parent().map(Path::to_path_buf));
                self.doc.replace(project);
                self.path = Some(path.to_path_buf());
                self.ui = self.fresh_ui_state();
                self.ui.mode = PlayMode::Pattern;
                self.send(Msg::Mode(PlayMode::Pattern));
                self.compiled = None;
                self.settings.add_recent(path);
                self.save_settings();
                self.toast(format!("Opened {}", path.display()));
            }
            Err(e) => self.toast_error(e.to_string()),
        }
    }

    /// Save to the current path, or ask for one. Returns true when saved.
    pub fn save(&mut self) -> bool {
        match self.path.clone() {
            Some(p) => self.save_to(&p),
            None => self.save_as(),
        }
    }

    pub fn save_as(&mut self) -> bool {
        let name = format!(
            "{}.{}",
            sanitize_file_name(&self.project().title),
            limeshot_project::EXTENSION
        );
        let dialog = rfd::FileDialog::new()
            .add_filter("LimeShot project", &[limeshot_project::EXTENSION])
            .set_file_name(name);
        match dialog.save_file() {
            Some(mut p) => {
                if p.extension().is_none() {
                    p.set_extension(limeshot_project::EXTENSION);
                }
                self.save_to(&p)
            }
            None => false,
        }
    }

    /// Save a numbered sibling, or choose the first path for a project not saved yet.
    /// Captures running plugin states before writing. On success, makes the saved path active
    /// and marks the document saved without clearing undo history.
    /// Returns `false` on cancellation or a save failure; save errors become an error toast.
    pub fn save_new_version(&mut self) -> bool {
        let Some(current) = self.path.clone() else {
            return self.save_as();
        };
        self.plugins.store_states(&mut self.doc);
        match limeshot_project::save_new_version(self.doc.project(), &current) {
            Ok(path) => {
                self.saved_to(&path);
                true
            }
            Err(e) => {
                self.toast_error(e.to_string());
                false
            }
        }
    }

    /// Record a successful save: update the sample base folder and active path, mark the
    /// document saved, and add the path to recent files, persisting settings when enabled.
    fn saved_to(&mut self, path: &Path) {
        self.samples.set_project_dir(path.parent().map(Path::to_path_buf));
        self.doc.mark_saved();
        self.path = Some(path.to_path_buf());
        self.settings.add_recent(path);
        self.save_settings();
        self.toast(format!("Saved {}", path.display()));
    }

    /// Capture running plugin states and save the project, replacing `path` if it exists.
    /// Returns `true` after making the path active and marking the document saved.
    /// Save failures become an error toast and return `false`, retaining the active path
    /// and saved marker.
    pub fn save_to(&mut self, path: &Path) -> bool {
        self.plugins.store_states(&mut self.doc);
        match limeshot_project::save(self.doc.project(), path) {
            Ok(()) => {
                self.saved_to(path);
                true
            }
            Err(e) => {
                self.toast_error(e.to_string());
                false
            }
        }
    }

    /// Show the recovery dialog if a previous session left an autosave behind.
    pub fn offer_recovery(&mut self) {
        if let Some(info) = self.autosave.find() {
            self.dialog = Some(dialogs::Dialog::Recover(info));
        }
    }

    /// Open the autosaved project as unsaved changes to its original file.
    pub fn recover(&mut self, info: &crate::autosave::SessionInfo) -> bool {
        match self.autosave.load() {
            Ok(project) => {
                self.send(Msg::Stop);
                let project = absolutize_samples(&project, self.autosave.dir.as_deref());
                self.doc.replace(project);
                self.doc.mark_unsaved();
                self.path = info.original.clone();
                self.samples
                    .set_project_dir(info.original.as_deref().and_then(Path::parent).map(Path::to_path_buf));
                self.ui = self.fresh_ui_state();
                self.ui.mode = PlayMode::Pattern;
                self.send(Msg::Mode(PlayMode::Pattern));
                self.compiled = None;
                self.autosave.adopt();
                self.toast("Recovered the autosaved project — save it to keep it");
                true
            }
            Err(e) => {
                self.toast_error(format!("Could not recover the autosave: {e}"));
                false
            }
        }
    }

    /// Replace the project with an imported FL Studio project. It has no `.lime` path yet, so
    /// Save asks where to write it (and it joins the recent list only once saved).
    pub fn import_flp(&mut self, path: &Path) -> bool {
        let flp = limeshot_project::flp::import_file(path, limeshot_project::flp::default_factory_dir().as_deref());
        match flp {
            Ok((project, report)) => {
                self.send(Msg::Stop);
                let project = absolutize_samples(&project, path.parent());
                self.samples.set_project_dir(path.parent().map(Path::to_path_buf));
                self.doc.replace(project);
                self.doc.mark_unsaved();
                self.path = None;
                self.ui = self.fresh_ui_state();
                self.ui.mode = PlayMode::Pattern;
                self.send(Msg::Mode(PlayMode::Pattern));
                self.compiled = None;
                self.toast(format!("Imported {}: {}", path.display(), report.summary()));
                true
            }
            Err(e) => {
                self.toast_error(e.to_string());
                false
            }
        }
    }

    /// Import a MIDI file as a new pattern (one LimeSynth channel per MIDI track). An empty
    /// project also takes the file's tempo.
    pub fn import_midi(&mut self, path: &Path) -> bool {
        let file = match limeshot_project::midi::read_file(path) {
            Ok(f) => f,
            Err(e) => {
                self.toast_error(e.to_string());
                return false;
            }
        };
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "MIDI".into());
        let empty =
            self.project().patterns.iter().all(|p| p.notes.is_empty()) && self.project().playlist.clips.is_empty();
        let cmd = match limeshot_project::midi::import_command(&file, &name, empty) {
            Ok(c) => c,
            Err(e) => {
                self.toast_error(format!("{}: {e}", path.display()));
                return false;
            }
        };
        match self.exec(cmd) {
            Some(id) => {
                self.set_current_pattern(PatternId(id));
                self.ui.place = None;
                self.toast(format!(
                    "Imported {} track(s) from {}",
                    file.tracks.len(),
                    path.display()
                ));
                true
            }
            None => false,
        }
    }

    /// Ask where to export the current pattern (or the song) as a MIDI file.
    pub fn export_midi_dialog(&mut self, song: bool) {
        let name = format!("{}.mid", sanitize_file_name(&self.project().title));
        if let Some(mut p) = rfd::FileDialog::new()
            .add_filter("MIDI file", &["mid"])
            .set_file_name(name)
            .save_file()
        {
            if p.extension().is_none() {
                p.set_extension("mid");
            }
            self.export_midi(&p, song);
        }
    }

    /// Write the current pattern (or the song) as a MIDI file, as it plays (swing included).
    pub fn export_midi(&mut self, path: &Path, song: bool) -> bool {
        let snap = compile(self.project(), self.current_pattern(), &limeshot_sequencer::NoSamples);
        let program = if song { &snap.song } else { &snap.pattern };
        let file = limeshot_project::midi::from_program(self.project(), program);
        if file.tracks.is_empty() {
            self.toast_error(if song {
                "The song has no notes to export"
            } else {
                "The pattern has no notes to export"
            });
            return false;
        }
        match std::fs::write(path, limeshot_project::midi::write(&file)) {
            Ok(()) => {
                self.toast(format!("Exported {}", path.display()));
                true
            }
            Err(e) => {
                self.toast_error(format!("{}: {e}", path.display()));
                false
            }
        }
    }

    /// Restart audio output with the current settings.
    pub fn restart_audio(&mut self) {
        let snap = self.snapshot.clone();
        self.player = Player::start(&self.settings.audio, snap);
        // The new stream has a new renderer (possibly at a new rate): rebuild every processor.
        self.sent_effects.clear();
        let (mode, lp, met) = (self.ui.mode, self.ui.song_loop, self.ui.metronome);
        self.send(Msg::Mode(mode));
        self.send(Msg::SongLoop(lp));
        self.send(Msg::Metronome(met));
        self.compiled = None;
        self.save_settings();
    }

    // ----- frame -----

    /// Per-frame logic that is not drawing: theme, engine sync, window title, close requests.
    pub fn logic(&mut self, ctx: &Context) {
        if !self.themed {
            crate::theme::Theme::current().apply(ctx);
            self.themed = true;
            ctx.request_repaint();
        }
        ctx.set_zoom_factor(self.settings.ui_scale);
        self.player.collect_garbage();
        // A safety net: auditions end with their mouse button, whichever editor started them.
        if !self.auditions.is_empty() && !ctx.input(|i| i.pointer.any_down()) {
            for (c, k) in std::mem::take(&mut self.auditions) {
                self.preview_off(c, k);
            }
        }
        let mut held = std::mem::take(&mut self.typing_notes);
        held.retain(|(key, channel, note)| {
            let keep = self.keyboard_notes
                && !ctx.egui_wants_keyboard_input()
                && self.dialog.is_none()
                && ctx.input(|i| i.focused && i.key_down(*key));
            if !keep {
                self.preview_off(*channel, *note);
                crate::recording::note_off(self, *channel, *note);
            }
            keep
        });
        self.typing_notes = held;
        if !self.midi_opened {
            self.midi_opened = true;
            if self.persist_settings {
                self.open_midi_input(ctx);
            }
        }
        crate::recording::tick(self);
        let events = self.midi.as_ref().map(|m| m.drain()).unwrap_or_default();
        for e in events {
            self.midi_event(e);
        }
        self.sync_engine();
        if self.plugins.poll() {
            self.doc.mark_unsaved();
        }
        if self.ui.mode == PlayMode::Song && self.is_playing() {
            let tick = self.player.status().position();
            self.plugins.follow_automation(self.doc.project(), &self.snapshot, tick);
        }
        for target in self.plugins.take_crashes() {
            let name = crate::plugins::place_name(self.doc.project(), target)
                .map(|(n, place)| format!("{n} ({place})"))
                .unwrap_or_else(|| "A plugin".into());
            self.toast_error(format!("{name} crashed; it is silent until reloaded"));
        }
        if let Some(summary) = self.plugins.poll_scan() {
            self.toast(summary);
        }
        if self.plugins.is_scanning() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        let range = self.ui.playlist.time_sel;
        if self.loop_sent != Some(range) {
            self.send(Msg::LoopRange(range));
            self.loop_sent = Some(range);
        }
        crate::browser::poll_preview(self, ctx);
        crate::autosave::tick(self);
        dialogs::poll_export(self);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close && self.doc.is_modified() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.dialog = Some(dialogs::Dialog::ConfirmDiscard(Pending::Quit));
        }
        if self.allow_close {
            // A normal exit: nothing to recover next time.
            self.autosave.clear();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // Meters and the playhead animate; keep frames coming at a modest rate.
        ctx.request_repaint_after(Duration::from_millis(if self.is_playing() { 16 } else { 50 }));
        self.toasts.retain(|t| t.until > Instant::now());
    }

    /// Draw the whole window.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.set_style(ctx.global_style());
        let t = crate::theme::Theme::current();
        // Custom fonts registered by `logic` become available on the following frame.
        if self.themed_frames < 1 || !self.themed {
            self.themed_frames += u32::from(self.themed);
            ui.painter().rect_filled(ui.max_rect(), 0.0, t.bg);
            ctx.request_repaint();
            return;
        }
        self.ui.hint.clear();
        if !crate::plugin_window::handle_keys(self, &ctx) {
            shortcuts::handle(self, &ctx);
        }
        let flat = |fill| egui::Frame::new().fill(fill);
        egui::Panel::top("header")
            .exact_size(30.0)
            .frame(flat(t.header).inner_margin(egui::Margin::symmetric(8, 0)))
            .show(ui, |ui| toolbar::header(self, ui));
        egui::Panel::top("toolbar")
            .exact_size(toolbar::HEIGHT)
            .frame(flat(t.header).inner_margin(egui::Margin::symmetric(8, 0)))
            .show(ui, |ui| toolbar::show(self, ui));
        egui::Panel::bottom("status")
            .exact_size(24.0)
            .frame(flat(t.header).inner_margin(egui::Margin::symmetric(10, 0)))
            .show(ui, |ui| toolbar::status_bar(self, ui));
        if self.ui.show_browser {
            egui::Panel::left("browser")
                .resizable(true)
                .default_size(230.0)
                .size_range(160.0..=420.0)
                .frame(flat(t.surface))
                .show(ui, |ui| browser::show(self, ui));
        }
        let area = egui::CentralPanel::default()
            .frame(flat(t.bg))
            .show(ui, |ui| {
                let r = ui.max_rect();
                crate::icons::logo(ui.painter(), r.center(), 56.0);
                ui.painter().text(
                    r.center() + egui::vec2(0.0, 80.0),
                    egui::Align2::CENTER_CENTER,
                    "LimeShot",
                    crate::theme::semibold(15.0),
                    t.text_faint,
                );
                r
            })
            .inner;
        crate::workspace::show(self, &ctx, area, window_body);
        crate::plugin_window::show_all(self, &ctx);
        // Last, when every control has described itself.
        toolbar::hint_panel(self, ui);
        crate::settings_window::show(self, &ctx);
        crate::toolbar::tapper(self, &ctx);
        dialogs::show(self, &ctx);
        dialogs::toasts(self, &ctx);
        crate::browser::drag_preview(self, &ctx);
    }
}

/// The content of each workspace window.
fn window_body(app: &mut LimeShotApp, ui: &mut egui::Ui, w: crate::workspace::Win) {
    use crate::workspace::Win;
    match w {
        Win::Playlist => playlist::panel(app, ui),
        Win::ChannelRack => channel_rack::panel(app, ui),
        Win::PianoRoll => piano_roll::panel(app, ui),
        Win::Mixer => mixer::panel(app, ui),
        Win::ChannelSettings => crate::inspector::show(app, ui),
    }
}

fn sanitize_file_name(s: &str) -> String {
    let clean: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let clean = clean.trim();
    if clean.is_empty() {
        "Untitled".into()
    } else {
        clean.into()
    }
}

/// A copy of `project` whose relative sample paths are resolved against `dir`.
fn absolutize_samples(project: &Project, dir: Option<&Path>) -> Project {
    let mut p = project.clone();
    for c in &mut p.channels {
        if let limeshot_core::Instrument::Sampler(s) = &mut c.instrument
            && let Some(r) = &mut s.sample
        {
            r.path = limeshot_project::resolve_sample_path(&r.path, dir)
                .to_string_lossy()
                .into_owned();
        }
    }
    p
}
