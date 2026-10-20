//! Application state and the glue between the document, the engine and the panels.
//!
//! State is kept in separate layers (see `docs/architecture/ui.md`):
//! * persistent project state: `doc` (a [`Document`]; changed only through commands),
//! * application configuration: `settings`,
//! * engine/transport state: owned by the audio thread, mirrored through `player.status()`,
//! * runtime resources: `samples` (decoded audio),
//! * ephemeral UI state: `ui` (open windows, selections, zoom, the current pattern).

use crate::settings::Settings;
use crate::{browser, channel_rack, dialogs, mixer, piano_roll, playlist, shortcuts, toolbar};
use egui::{Context, Rect, Response};
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
    pub show_rack: bool,
    pub show_piano: bool,
    pub show_playlist: bool,
    pub show_mixer: bool,
    pub current_pattern: PatternId,
    pub selected_channel: Option<ChannelId>,
    pub snap: Snap,
    pub mode: PlayMode,
    pub song_loop: bool,
    pub metronome: bool,
    pub focus: Focus,
    pub hint: String,
    pub arrange_windows: bool,
    pub workspace: Rect,
    pub rack: channel_rack::State,
    pub piano: piano_roll::State,
    pub playlist: playlist::State,
    pub mixer: mixer::State,
    pub browser: browser::State,
}

impl UiState {
    fn new(project: &Project) -> Self {
        Self {
            show_browser: true,
            show_rack: true,
            show_piano: false,
            show_playlist: true,
            show_mixer: false,
            current_pattern: project.patterns.first().map(|p| p.id).unwrap_or(PatternId(0)),
            selected_channel: project.channels.first().map(|c| c.id),
            snap: Snap::Step,
            mode: PlayMode::Pattern,
            song_loop: true,
            metronome: false,
            focus: Focus::None,
            hint: String::new(),
            arrange_windows: true,
            workspace: Rect::NOTHING,
            rack: Default::default(),
            piano: Default::default(),
            playlist: Default::default(),
            mixer: Default::default(),
            browser: Default::default(),
        }
    }
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
    pub settings: Settings,
    pub ui: UiState,
    pub dialog: Option<dialogs::Dialog>,
    pub toasts: Vec<Toast>,
    /// Folder of bundled samples, shown first in the Browser.
    pub bundled_samples: Option<PathBuf>,
    pub export_job: Option<dialogs::ExportJob>,
    compiled: Option<(u64, PatternId)>,
    snapshot: Arc<Snapshot>,
    pub allow_close: bool,
    themed: bool,
    pub meter_hold: Vec<(f32, f32)>,
    pub clip_until: Option<Instant>,
}

impl LimeShotApp {
    pub fn new(settings: Settings, bundled_samples: Option<PathBuf>) -> Self {
        let audio = settings.audio.clone();
        let mut app = Self::new_with_player(settings, bundled_samples, |s| Player::start(&audio, s));
        app.send(Msg::SongLoop(true));
        app
    }

    /// For tests and the headless path: an app with no audio device.
    pub fn new_silent(settings: Settings, bundled_samples: Option<PathBuf>) -> Self {
        let mut app = Self::new_with_player(settings, bundled_samples, |s| Player::silent("headless".into(), s));
        app.send(Msg::SongLoop(true));
        app
    }

    fn new_with_player(settings: Settings, bundled_samples: Option<PathBuf>, player: impl FnOnce(Arc<Snapshot>) -> Player) -> Self {
        let doc = Document::default();
        let snapshot = Arc::new(compile(doc.project(), doc.project().patterns[0].id, &limeshot_sequencer::NoSamples));
        let ui = UiState::new(doc.project());
        Self {
            player: player(snapshot.clone()),
            doc,
            path: None,
            samples: SampleCache::new(),
            settings,
            ui,
            dialog: None,
            toasts: Vec::new(),
            bundled_samples,
            export_job: None,
            compiled: None,
            snapshot,
            allow_close: false,
            themed: false,
            meter_hold: Vec::new(),
            clip_until: None,
        }
    }

    pub fn project(&self) -> &Project {
        self.doc.project()
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
        self.toasts.push(Toast { text: text.into(), error: false, until: Instant::now() + Duration::from_secs(3) });
    }

    pub fn toast_error(&mut self, text: impl Into<String>) {
        let text = text.into();
        log::warn!("{text}");
        self.toasts.push(Toast { text, error: true, until: Instant::now() + Duration::from_secs(6) });
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

    pub fn set_current_pattern(&mut self, id: PatternId) {
        self.ui.current_pattern = id;
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

    pub fn stop(&mut self) {
        self.send(Msg::Stop);
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        if self.ui.mode != mode {
            self.ui.mode = mode;
            self.send(Msg::Mode(mode));
        }
    }

    pub fn toggle_mode(&mut self) {
        let m = if self.ui.mode == PlayMode::Pattern { PlayMode::Song } else { PlayMode::Pattern };
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
        if let Some(i) = self.project().channel_index(channel).and_then(|i| u16::try_from(i).ok()) {
            self.sync_engine();
            self.send(Msg::NoteOn { channel: i, key, velocity });
        }
    }

    pub fn preview_off(&mut self, channel: ChannelId, key: u8) {
        if let Some(i) = self.project().channel_index(channel).and_then(|i| u16::try_from(i).ok()) {
            self.send(Msg::NoteOff { channel: i, key });
        }
    }

    /// Rebuild and send the playback snapshot when the project, the current pattern or the loaded
    /// samples changed.
    pub fn sync_engine(&mut self) {
        let loaded = self.samples.poll();
        let key = (self.doc.revision(), self.current_pattern());
        if !loaded && self.compiled == Some(key) {
            return;
        }
        if self.compiled.map(|k| k.0) != Some(key.0) {
            let refs: Vec<String> = self.project().sample_refs().map(|r| r.path.clone()).collect();
            for r in refs {
                self.samples.request(&r);
            }
        }
        let snap = Arc::new(compile(self.doc.project(), key.1, &self.samples));
        self.snapshot = snap.clone();
        self.send(Msg::Snapshot(snap));
        self.compiled = Some(key);
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }

    // ----- files -----

    pub fn window_title(&self) -> String {
        let name = self.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Untitled".into());
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
                let dialog = rfd::FileDialog::new().add_filter("LimeShot project", &[limeshot_project::EXTENSION]);
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
        self.ui = UiState { arrange_windows: false, workspace: self.ui.workspace, ..UiState::new(self.doc.project()) };
        self.set_mode(PlayMode::Pattern);
        self.ui.mode = PlayMode::Pattern;
        self.send(Msg::Mode(PlayMode::Pattern));
        self.compiled = None;
    }

    pub fn open_path(&mut self, path: &Path) {
        match limeshot_project::load(path) {
            Ok(project) => {
                self.send(Msg::Stop);
                // In memory, sample paths are absolute; they are made relative again on save.
                let project = absolutize_samples(&project, path.parent());
                self.samples.set_project_dir(path.parent().map(Path::to_path_buf));
                self.doc.replace(project);
                self.path = Some(path.to_path_buf());
                self.ui = UiState { arrange_windows: false, workspace: self.ui.workspace, ..UiState::new(self.doc.project()) };
                self.ui.mode = PlayMode::Pattern;
                self.send(Msg::Mode(PlayMode::Pattern));
                self.compiled = None;
                self.settings.add_recent(path);
                self.settings.save();
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
        let name = format!("{}.{}", sanitize_file_name(&self.project().title), limeshot_project::EXTENSION);
        let dialog = rfd::FileDialog::new().add_filter("LimeShot project", &[limeshot_project::EXTENSION]).set_file_name(name);
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

    pub fn save_to(&mut self, path: &Path) -> bool {
        match limeshot_project::save(self.doc.project(), path) {
            Ok(()) => {
                self.samples.set_project_dir(path.parent().map(Path::to_path_buf));
                self.doc.mark_saved();
                self.path = Some(path.to_path_buf());
                self.settings.add_recent(path);
                self.settings.save();
                self.toast(format!("Saved {}", path.display()));
                true
            }
            Err(e) => {
                self.toast_error(e.to_string());
                false
            }
        }
    }

    /// Restart audio output with the current settings.
    pub fn restart_audio(&mut self) {
        let snap = self.snapshot.clone();
        self.player = Player::start(&self.settings.audio, snap);
        let (mode, lp, met) = (self.ui.mode, self.ui.song_loop, self.ui.metronome);
        self.send(Msg::Mode(mode));
        self.send(Msg::SongLoop(lp));
        self.send(Msg::Metronome(met));
        self.compiled = None;
        self.settings.save();
    }

    // ----- frame -----

    /// Per-frame logic that is not drawing: theme, engine sync, window title, close requests.
    pub fn logic(&mut self, ctx: &Context) {
        if !self.themed {
            crate::theme::Theme::current().apply(ctx);
            self.themed = true;
        }
        ctx.set_zoom_factor(self.settings.ui_scale);
        self.player.collect_garbage();
        self.sync_engine();
        dialogs::poll_export(self);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close && self.doc.is_modified() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.dialog = Some(dialogs::Dialog::ConfirmDiscard(Pending::Quit));
        }
        if self.allow_close {
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
        self.ui.hint.clear();
        shortcuts::handle(self, &ctx);
        egui::Panel::top("menu").show(ui, |ui| crate::menus::menu_bar(self, ui));
        egui::Panel::top("toolbar").exact_size(toolbar::HEIGHT).show(ui, |ui| toolbar::show(self, ui));
        egui::Panel::bottom("status").exact_size(22.0).show(ui, |ui| toolbar::status_bar(self, ui));
        if self.ui.show_browser {
            egui::Panel::left("browser").resizable(true).default_size(220.0).min_size(150.0).show(ui, |ui| browser::show(self, ui));
        }
        let central = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(crate::theme::Theme::current().bg))
            .show(ui, |ui| {
                let r = ui.max_rect();
                crate::icons::logo(ui.painter(), r.center(), 60.0);
                ui.painter().text(
                    r.center() + egui::vec2(0.0, 84.0),
                    egui::Align2::CENTER_CENTER,
                    "LimeShot",
                    crate::theme::prop(16.0),
                    crate::theme::Theme::current().text_dim,
                );
                r
            });
        self.ui.workspace = central.inner;
        let arrange = std::mem::take(&mut self.ui.arrange_windows);
        playlist::window(self, &ctx, arrange);
        channel_rack::window(self, &ctx, arrange);
        piano_roll::window(self, &ctx, arrange);
        mixer::window(self, &ctx, arrange);
        dialogs::show(self, &ctx);
        dialogs::toasts(self, &ctx);
    }
}

fn sanitize_file_name(s: &str) -> String {
    let clean: String = s.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect();
    let clean = clean.trim();
    if clean.is_empty() { "Untitled".into() } else { clean.into() }
}

/// A copy of `project` whose relative sample paths are resolved against `dir`.
fn absolutize_samples(project: &Project, dir: Option<&Path>) -> Project {
    let mut p = project.clone();
    for c in &mut p.channels {
        if let limeshot_core::Instrument::Sampler(s) = &mut c.instrument
            && let Some(r) = &mut s.sample
        {
            r.path = limeshot_project::resolve_sample_path(&r.path, dir).to_string_lossy().into_owned();
        }
    }
    p
}
