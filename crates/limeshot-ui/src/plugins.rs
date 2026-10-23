//! Hosted plugins in the app (FL Studio's plugin database and Fruity Wrapper): the plugin
//! database, the plugins running for the open project, and their panels.
//!
//! * The **database** lists the VST3 instruments and effects found on this computer. Scanning runs
//!   each bundle in a child process (`limeshot --scan-vst3 <bundle>`), so a plugin that crashes
//!   while loading takes only that process down; the result is saved in `plugins.json` next to the
//!   settings.
//! * The **host** keeps one plugin instance per plugin channel and plugin insert slot of the
//!   project. Instances are created and destroyed on this (the main) thread, as plugins expect;
//!   their audio halves go to the audio thread and come back as garbage.
//! * Parameter changes made here or in a plugin's own window are not undo steps; they mark the
//!   project modified, and the plugin's state is read into the project when it is saved.

use crate::widgets::Hint as _;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use egui::{RichText, Ui, vec2};
use limeshot_audio::Msg;
use limeshot_core::{ChannelId, Command, Document, EffectKind, Instrument, PluginRef, PluginState, Project};
use limeshot_plugins::{PluginInstance, PluginKind, PluginProcessor, Vst3Instance};
use serde::{Deserialize, Serialize};

use crate::app::LimeShotApp;
use crate::theme::{Theme, semibold};
use crate::widgets::section;

pub use limeshot_core::PluginTarget as Target;

/// The `path` of LimeShot's built-in test plugins (only listed when enabled, for tests).
pub const BUILTIN_PATH: &str = limeshot_plugins::testing::BUILTIN_PATH;
/// How long one bundle may take to scan.
const SCAN_TIMEOUT: Duration = Duration::from_secs(30);
/// Parameters listed at once in a plugin's panel (plugins often have thousands).
const PARAMS_SHOWN: usize = 48;

/// One plugin class in the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub class_id: String,
    pub name: String,
    #[serde(default)]
    pub vendor: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub sub_categories: String,
    pub instrument: bool,
}

impl Entry {
    fn from_info(i: &limeshot_plugins::PluginInfo) -> Self {
        Self {
            path: i.path.to_string_lossy().into_owned(),
            class_id: i.class_id.clone(),
            name: i.name.clone(),
            vendor: i.vendor.clone(),
            version: i.version.clone(),
            sub_categories: i.sub_categories.clone(),
            instrument: i.kind == PluginKind::Instrument,
        }
    }

    pub fn plugin_ref(&self) -> PluginRef {
        let mut p = PluginRef::vst3(&self.path, &self.class_id, &self.name);
        p.vendor = self.vendor.clone();
        p
    }
}

/// The plugins found by the last scan.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub entries: Vec<Entry>,
    /// Bundles that could not be scanned, with the reason.
    #[serde(default)]
    pub failed: Vec<(String, String)>,
}

impl Database {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    pub fn instruments(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.instrument)
    }

    pub fn effects(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.instrument)
    }
}

/// The body of `limeshot --scan-vst3 <bundle>`: print the bundle's classes as JSON on stdout.
/// Returns the process exit code.
pub fn scan_child(bundle: &str) -> i32 {
    match limeshot_plugins::scan::scan_bundle(Path::new(bundle)) {
        Ok(classes) => {
            let entries: Vec<Entry> = classes.iter().map(Entry::from_info).collect();
            match serde_json::to_string(&entries) {
                Ok(json) => {
                    println!("{json}");
                    0
                }
                Err(_) => 1,
            }
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

enum ScanMsg {
    Bundle(String, Result<Vec<Entry>, String>),
    Done,
}

/// A scan running in the background.
pub struct Scan {
    rx: Receiver<ScanMsg>,
    pub total: usize,
    pub done: usize,
    found: Database,
}

fn scan_one(exe: &Path, bundle: &Path) -> Result<Vec<Entry>, String> {
    let mut child = std::process::Command::new(exe)
        .arg("--scan-vst3")
        .arg(bundle)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + SCAN_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("timed out".into());
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() {
            format!("the scan process failed ({})", out.status)
        } else {
            err
        });
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // Plugins may print to stdout while loading; the result is the last line.
    let line = text.lines().rev().find(|l| l.starts_with('[')).unwrap_or("");
    serde_json::from_str(line).map_err(|_| "no result from the scan process".into())
}

impl Scan {
    fn start(exe: PathBuf, bundles: Vec<PathBuf>) -> Self {
        let (tx, rx) = channel();
        let total = bundles.len();
        let spawned = std::thread::Builder::new()
            .name("limeshot-plugin-scan".into())
            .spawn(move || {
                for b in bundles {
                    let r = scan_one(&exe, &b);
                    if tx.send(ScanMsg::Bundle(b.to_string_lossy().into_owned(), r)).is_err() {
                        return;
                    }
                }
                let _ = tx.send(ScanMsg::Done);
            });
        if spawned.is_err() {
            log::warn!("could not start the plugin scan");
        }
        Self {
            rx,
            total,
            done: 0,
            found: Database::default(),
        }
    }

    /// Take finished bundles; returns the database when the scan is complete.
    fn poll(&mut self) -> Option<Database> {
        loop {
            match self.rx.try_recv() {
                Ok(ScanMsg::Bundle(path, r)) => {
                    self.done += 1;
                    match r {
                        Ok(entries) => self.found.entries.extend(entries),
                        Err(e) => self.found.failed.push((path, e)),
                    }
                }
                Ok(ScanMsg::Done) => return Some(std::mem::take(&mut self.found)),
                Err(std::sync::mpsc::TryRecvError::Empty) => return None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return Some(std::mem::take(&mut self.found)),
            }
        }
    }
}

struct Live {
    /// The plugin this instance runs; a different one (or a change of process) replaces it.
    key: Key,
    state: PluginState,
    instance: PluginInstance,
}

/// What a running plugin was made from: bundle, class, and whether it runs in its own process.
type Key = (String, String, bool, u64);

fn key_of(p: &PluginRef) -> Key {
    (
        p.path.clone(),
        p.class_id.clone(),
        p.separate_process,
        p.extra
            .get("lime-sidechain-track")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    )
}

/// Plugins of the open project, and the database.
#[derive(Default)]
pub struct PluginHost {
    pub(crate) host_graphics: limeshot_plugins::HostGraphicsContext,
    parameter_rects: HashMap<(Target, u32), egui::Rect>,
    pub db: Database,
    db_path: Option<PathBuf>,
    pub scan: Option<Scan>,
    live: HashMap<Target, Live>,
    /// Plugins that failed to load, with the reason (cleared when the plugin changes).
    errors: HashMap<Target, (Key, String)>,
    generation: Option<u64>,
    /// The parameter filter typed into each plugin's panel.
    filters: HashMap<Target, String>,
    /// Open plugin windows; an editor is dropped (and closed) before its plugin.
    pub(crate) windows: HashMap<Target, crate::plugin_window::Wrapper>,
    /// Where and on which tab each closed window was, for when it opens again.
    pub(crate) window_memory: HashMap<Target, crate::plugin_window::Memory>,
    /// The plugin parameter changed last (in the plugin's editor or LimeShot's list), for FL's
    /// Add › Automation for last tweaked parameter.
    pub last_tweaked: Option<(Target, u32)>,
    /// The program that runs plugins in a process of their own (LimeShot itself with
    /// `--plugin-bridge`); without one, such plugins cannot load.
    pub bridge_exe: Option<PathBuf>,
    /// Plugins whose process crashed, already reported.
    crashed: std::collections::HashSet<Target>,
    /// Windows to open again once their plugin has been recreated (Reload, a change of process).
    reopen: HashMap<Target, crate::plugin_window::Tab>,
    /// A plugin must be created although the project did not change (Reload).
    needs_sync: bool,
    /// Processors returned from an export thread, dropped here on the main thread.
    returned: Option<(Sender<PluginProcessor>, Receiver<PluginProcessor>)>,
}

fn load_module(path: &str) -> Result<Arc<limeshot_plugins::Vst3Module>, String> {
    limeshot_plugins::load_module(path).map_err(|e| e.to_string())
}

/// Every plugin the project uses, with where it runs.
fn wanted(project: &Project) -> Vec<(Target, &PluginRef)> {
    let mut out = Vec::new();
    for c in &project.channels {
        if let Instrument::Plugin(p) = &c.instrument {
            out.push((Target::Channel(c.id), p));
        }
    }
    for (t, track) in project.mixer.tracks.iter().enumerate() {
        let Ok(track16) = u16::try_from(t) else { continue };
        for e in &track.effects {
            if let (EffectKind::Plugin, Some(p)) = (e.kind, &e.plugin) {
                out.push((
                    Target::Effect {
                        track: track16,
                        effect: e.id,
                    },
                    p,
                ));
            }
        }
    }
    out
}

/// Create a plugin (in a bridge process started from `bridge` when the plugin asks for one) and
/// restore its saved state.
fn instantiate(
    p: &PluginRef,
    sample_rate: u32,
    bridge: Option<&Path>,
) -> Result<(PluginInstance, PluginProcessor), String> {
    if p.format != "VST3" {
        return Err(format!("{} plugins are not supported yet", p.format));
    }
    let sr = f64::from(sample_rate);
    let block = limeshot_engine::MAX_BLOCK;
    let (mut inst, mut proc) = match (p.separate_process, bridge) {
        #[cfg(unix)]
        (true, Some(exe)) => {
            let (i, pr) = limeshot_plugins::bridge::BridgedInstance::spawn(exe, &p.path, &p.class_id, sr, block)
                .map_err(|e| e.to_string())?;
            (PluginInstance::Bridged(i), PluginProcessor::Bridged(pr))
        }
        (true, _) if cfg!(unix) => return Err("no plugin bridge is available".into()),
        _ => {
            let module = load_module(&p.path)?;
            let (i, pr) = Vst3Instance::new(&module, &p.class_id, sr, block).map_err(|e| e.to_string())?;
            (PluginInstance::Local(i), PluginProcessor::Local(pr))
        }
    };
    if p.extra.get("lime-sample-pending").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(format!(
            "{}: sample conversion is incomplete; choose a supported source file and reload",
            p.name
        ));
    }
    if !p.state.is_empty()
        && let Err(e) = inst.set_state(p.state.bytes())
    {
        return Err(format!("{}: could not restore plugin state: {e}", p.name));
    }
    proc.set_sidechain_source(
        p.extra
            .get("lime-sidechain-track")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            .min(127) as usize,
    );
    Ok((inst, proc))
}

impl PluginHost {
    /// A host whose database lives at `db_path` (none: an empty database that is not saved).
    pub fn new(db_path: Option<PathBuf>) -> Self {
        let db = db_path.as_deref().map(Database::load).unwrap_or_default();
        Self {
            db,
            db_path,
            returned: Some(channel()),
            ..Default::default()
        }
    }

    /// The database file next to the settings.
    pub fn default_db_path() -> Option<PathBuf> {
        crate::settings::config_dir().map(|d| d.join("plugins.json"))
    }

    /// List LimeShot's built-in test plugins in the database (headless tests and demos).
    pub fn add_builtin_test_plugins(&mut self) {
        let Ok(m) = limeshot_plugins::testing::module() else {
            return;
        };
        for c in m.classes() {
            let mut e = Entry::from_info(&c);
            e.path = BUILTIN_PATH.into();
            if !self.db.entries.contains(&e) {
                self.db.entries.push(e);
            }
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.scan.is_some()
    }

    /// Scan the standard VST3 folders with `exe --scan-vst3` per bundle.
    pub fn start_scan(&mut self, exe: PathBuf) {
        if self.scan.is_some() {
            return;
        }
        let bundles = limeshot_plugins::scan::find_bundles(&limeshot_plugins::scan::default_dirs());
        self.scan = Some(Scan::start(exe, bundles));
    }

    /// Advance a running scan. Returns a summary when it finished.
    pub fn poll_scan(&mut self) -> Option<String> {
        let db = self.scan.as_mut()?.poll()?;
        self.scan = None;
        let builtin: Vec<Entry> = self
            .db
            .entries
            .iter()
            .filter(|e| e.path == BUILTIN_PATH)
            .cloned()
            .collect();
        self.db = db;
        self.db.entries.extend(builtin);
        self.db.entries.sort_by_key(|a| a.name.to_lowercase());
        if let Some(p) = &self.db_path
            && let Err(e) = self.db.save(p)
        {
            log::warn!("could not save the plugin database: {e}");
        }
        let n = self.db.entries.len();
        Some(match self.db.failed.len() {
            0 => format!("Found {n} plugins"),
            f => format!("Found {n} plugins; {f} could not be loaded"),
        })
    }

    /// Bring the running plugins in line with `project`: create missing ones (sending their audio
    /// halves to the audio thread with `send`, ahead of the snapshot) and drop removed ones.
    pub fn sync(
        &mut self,
        project: &Project,
        generation: u64,
        sample_rate: u32,
        send: &mut dyn FnMut(Msg),
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let opened = self.generation != Some(generation);
        if opened {
            // A different project: the audio thread retires every processor (ClearEffects).
            self.generation = Some(generation);
            self.windows.clear();
            self.window_memory.clear();
            self.live.clear();
            self.errors.clear();
        }
        let wanted = wanted(project);
        let saved_windows: Vec<(Target, &PluginRef)> = if opened { wanted.clone() } else { Vec::new() };
        self.live
            .retain(|t, l| wanted.iter().any(|(w, p)| w == t && l.key == key_of(p)));
        // A window whose plugin went (or was replaced) closes.
        let live = &self.live;
        self.windows.retain(|t, _| live.contains_key(t));
        self.errors
            .retain(|t, (k, _)| wanted.iter().any(|(w, p)| w == t && *k == key_of(p)));
        for (target, p) in wanted {
            if let Some(live) = self.live.get_mut(&target) {
                if live.state != p.state {
                    match live.instance.set_state(p.state.bytes()) {
                        Ok(()) => {
                            self.errors.remove(&target);
                        }
                        Err(error) => {
                            let error = format!("{}: could not restore plugin state: {error}", p.name);
                            warnings.push(error.clone());
                            self.errors.insert(target, (key_of(p), error));
                        }
                    }
                    live.state = p.state.clone();
                }
                continue;
            }
            if self.errors.contains_key(&target) {
                continue;
            }
            let key = key_of(p);
            match instantiate(p, sample_rate, self.bridge_exe.as_deref()) {
                Ok((instance, proc)) => {
                    match target {
                        Target::Channel(c) => send(Msg::Instrument(c.0, Box::new(proc))),
                        Target::Effect { effect, .. } => send(Msg::Effect(effect, Box::new(proc))),
                    }
                    self.live.insert(
                        target,
                        Live {
                            key,
                            state: p.state.clone(),
                            instance,
                        },
                    );
                }
                Err(e) => {
                    warnings.push(format!("Could not load {}: {e}", p.name));
                    self.errors.insert(target, (key, e));
                }
            }
        }
        self.restore_windows(saved_windows);
        let ready: Vec<(Target, crate::plugin_window::Tab)> = self
            .reopen
            .iter()
            .filter(|(t, _)| self.live.contains_key(t))
            .map(|(t, tab)| (*t, *tab))
            .collect();
        for (t, tab) in ready {
            self.reopen.remove(&t);
            self.open_window(t, Some(tab));
        }
        let errors = &self.errors;
        self.reopen.retain(|t, _| !errors.contains_key(t));
        warnings
    }

    /// Every running plugin.
    pub fn live_targets(&self) -> Vec<Target> {
        self.live.keys().copied().collect()
    }

    /// Last drawn slider geometry, for interaction tests and accessibility tools.
    pub fn parameter_rect(&self, target: Target, id: u32) -> Option<egui::Rect> {
        self.parameter_rects.get(&(target, id)).copied()
    }

    /// Restrict the host's parameter list to matching parameter names.
    pub fn filter_parameters(&mut self, target: Target, text: impl Into<String>) {
        self.filters.insert(target, text.into());
    }

    pub fn is_loaded(&self, target: Target) -> bool {
        self.live.contains_key(&target)
    }

    pub fn error(&self, target: Target) -> Option<&str> {
        self.errors.get(&target).map(|(_, e)| e.as_str())
    }

    pub fn instance(&self, target: Target) -> Option<&PluginInstance> {
        self.live.get(&target).map(|l| &l.instance)
    }

    pub(crate) fn instance_mut(&mut self, target: Target) -> Option<&mut PluginInstance> {
        self.live.get_mut(&target).map(|l| &mut l.instance)
    }

    /// Per frame: forward edits made in plugin windows, drop processors returned by exports.
    /// Returns true when a plugin changed (the project is then modified).
    pub fn poll(&mut self) -> bool {
        if let Some((_, rx)) = &self.returned {
            while rx.try_recv().is_ok() {}
        }
        let mut changed = false;
        for (target, l) in &mut self.live {
            if l.instance.crashed() {
                continue;
            }
            if let Some(&(id, _)) = l.instance.take_editor_edits().last() {
                self.last_tweaked = Some((*target, id));
                changed = true;
            }
            changed |= l.instance.take_restart();
        }
        changed
    }

    /// Plugins whose process crashed or hung since the last call (each reported once).
    pub fn take_crashes(&mut self) -> Vec<Target> {
        let mut out = Vec::new();
        for (target, l) in &self.live {
            if l.instance.is_bridged() && !self.crashed.contains(target) && l.instance.crashed() {
                out.push(*target);
            }
        }
        self.crashed.extend(out.iter().copied());
        out
    }

    /// Whether the plugin at `target` runs in its own process and that process crashed.
    pub fn has_crashed(&self, target: Target) -> bool {
        self.crashed.contains(&target)
    }

    /// Start the plugin at `target` again (after a crash): the next sync creates it anew from the
    /// project, with the state saved there.
    pub fn reload(&mut self, target: Target) {
        self.keep_window_open(target);
        self.live.remove(&target);
        self.errors.remove(&target);
        self.crashed.remove(&target);
        self.needs_sync = true;
    }

    /// Whether plugins must be synced (and the snapshot sent again) although the project did not
    /// change. Clears the request.
    pub fn take_needs_sync(&mut self) -> bool {
        std::mem::take(&mut self.needs_sync)
    }

    /// Close the window of the plugin at `target` now and open it again, on the same tab, once
    /// the plugin has been recreated.
    pub(crate) fn keep_window_open(&mut self, target: Target) {
        if let Some(w) = self.window(target) {
            let tab = w.tab;
            self.close_window(target);
            self.reopen.insert(target, tab);
        }
    }

    /// Move plugins' own controls to their automated values at `tick` (Song playback), as FL
    /// Studio's automated knobs move.
    pub fn follow_automation(
        &mut self,
        project: &Project,
        snapshot: &limeshot_sequencer::Snapshot,
        tick: limeshot_core::Tick,
    ) {
        use limeshot_sequencer::LaneTarget;
        for lane in &snapshot.automation {
            let (target, id) = match lane.target {
                LaneTarget::ChannelPluginParam { channel, id } => match project.channels.get(channel) {
                    Some(c) => (Target::Channel(c.id), id),
                    None => continue,
                },
                LaneTarget::EffectPluginParam { track, effect, id } => match u16::try_from(track) {
                    Ok(track) => (Target::Effect { track, effect }, id),
                    Err(_) => continue,
                },
                _ => continue,
            };
            if let (Some(v), Some(l)) = (lane.value_at(tick), self.live.get_mut(&target)) {
                l.instance.show_param(id, f64::from(v));
            }
        }
    }

    /// The automation target, current value and name of a running plugin's parameter.
    pub fn automation_of(
        &self,
        project: &Project,
        target: Target,
        id: u32,
    ) -> Option<(limeshot_core::AutomationTarget, f32, String)> {
        let inst = self.instance(target)?;
        let p = inst.params().iter().find(|p| p.id == id)?;
        let plugin = project.plugin(target).map(|p| p.name.clone()).unwrap_or_default();
        Some((
            limeshot_core::AutomationTarget::PluginParam { plugin: target, id },
            inst.param(id) as f32,
            format!("{} – {plugin}", p.title),
        ))
    }

    /// Set a parameter of a running plugin (normalised 0–1).
    pub fn set_param(&mut self, target: Target, id: u32, value: f64) {
        if let Some(l) = self.live.get_mut(&target) {
            l.instance.set_param(id, value);
        }
    }

    /// Read every running plugin's state, and where its window is, into the project (before
    /// saving).
    pub fn store_states(&self, doc: &mut Document) {
        for (target, l) in &self.live {
            match l.instance.state() {
                Ok(s) => {
                    doc.store_plugin_state(*target, PluginState::new(s));
                }
                Err(e) => log::warn!("could not read a plugin's state: {e}"),
            }
            doc.store_plugin_window(*target, self.window_place(*target));
        }
    }

    /// Fresh plugin instances for an offline render of `project`, with the running plugins'
    /// current state. The control halves are returned for the caller to keep (and drop on this
    /// thread) until the render is done; the processors hand themselves back here when dropped.
    pub fn offline(
        &self,
        project: &Project,
        sample_rate: u32,
    ) -> Result<(limeshot_engine::Processors, Vec<PluginInstance>), String> {
        let mut processors = limeshot_engine::Processors::default();
        let mut keep = Vec::new();
        let Some((tx, _)) = &self.returned else {
            return Err("The plugin host is not initialized".into());
        };
        for (target, p) in wanted(project) {
            if let Some(error) = self.error(target) {
                return Err(format!("Could not load {} for export: {error}", p.name));
            }
            let mut p = p.clone();
            if let Some(Ok(s)) = self.live.get(&target).map(|l| l.instance.state()) {
                p.state = PluginState::new(s);
            }
            match instantiate(&p, sample_rate, self.bridge_exe.as_deref()) {
                Ok((inst, proc)) => {
                    let proc = ReturnOnDrop {
                        inner: Some(proc),
                        tx: tx.clone(),
                    };
                    match target {
                        Target::Channel(c) => processors.instruments.push((c.0, Box::new(proc))),
                        Target::Effect { effect, .. } => processors.effects.push((effect, Box::new(proc))),
                    }
                    keep.push(inst);
                }
                Err(e) => return Err(format!("Could not load {} for export: {e}", p.name)),
            }
        }
        Ok((processors, keep))
    }
}

/// A processor used on another thread that sends itself back to be dropped on the main thread.
struct ReturnOnDrop {
    inner: Option<PluginProcessor>,
    tx: Sender<PluginProcessor>,
}

impl Drop for ReturnOnDrop {
    fn drop(&mut self) {
        if let Some(p) = self.inner.take() {
            // If the host is gone the processor drops here; only reachable when quitting.
            let _ = self.tx.send(p);
        }
    }
}

impl limeshot_dsp::effects::Effect for ReturnOnDrop {
    /// Selected mixer key source; zero means the processor main input.
    fn sidechain_source(&self) -> usize {
        self.inner.as_ref().map_or(0, |p| p.sidechain_source())
    }
    /// Maximum dynamics reduction from the preceding block, in dB.
    fn gain_reduction_db(&self) -> f32 {
        self.inner.as_ref().map_or(0.0, |p| p.telemetry().reduction_db)
    }
    /// Use separate detector audio without mixing the key into the audible output.
    fn process_sidechain(&mut self, buf: &mut [f32], key: &[f32]) {
        if let Some(p) = &mut self.inner {
            limeshot_dsp::effects::Effect::process_sidechain(p, buf, key);
        }
    }
    fn set_params(&mut self, _params: &[f32]) {}
    fn process(&mut self, buf: &mut [f32]) {
        if let Some(p) = &mut self.inner {
            limeshot_dsp::effects::Effect::process(p, buf);
        }
    }
    fn reset(&mut self) {
        if let Some(p) = &mut self.inner {
            p.restart();
        }
    }
    fn set_transport(&mut self, t: &limeshot_dsp::Transport) {
        if let Some(p) = &mut self.inner {
            limeshot_dsp::effects::Effect::set_transport(p, t);
        }
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        if let Some(p) = &mut self.inner {
            p.set_param_at(id, offset, value);
        }
    }
    fn latency(&self) -> u32 {
        self.inner.as_ref().map_or(0, |p| p.latency())
    }
}

impl limeshot_dsp::Instrument for ReturnOnDrop {
    /// Whether the instrument implements the native additive balance protocol.
    fn uses_channel_pan(&self) -> bool {
        self.inner.as_ref().is_some_and(|p| p.uses_channel_pan())
    }
    /// Actual reported voices, including releases; plugins without telemetry contribute zero.
    fn active_voices(&self) -> usize {
        self.inner.as_ref().map_or(0, |p| p.telemetry().voices as usize)
    }
    fn process(&mut self, out: &mut [f32], notes: &[limeshot_dsp::NoteEvent], t: &limeshot_dsp::Transport) {
        match &mut self.inner {
            Some(p) => limeshot_dsp::Instrument::process(p, out, notes, t),
            None => out.fill(0.0),
        }
    }
    fn reset(&mut self) {
        if let Some(p) = &mut self.inner {
            p.restart();
        }
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        if let Some(p) = &mut self.inner {
            p.set_param_at(id, offset, value);
        }
    }
    fn latency(&self) -> u32 {
        self.inner.as_ref().map_or(0, |p| p.latency())
    }
}

// ----- panels -----

/// The plugin menu entries of an "Add" menu: instruments or effects from the database, grouped by
/// vendor, then "Find plugins…". Returns the picked entry.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui, instruments: bool) -> Option<Entry> {
    let entries: Vec<Entry> = if instruments {
        app.plugins.db.instruments().cloned().collect()
    } else {
        app.plugins.db.effects().cloned().collect()
    };
    let mut pick = None;
    ui.menu_button("Plugins", |ui| {
        if entries.is_empty() {
            ui.label(RichText::new("No plugins found yet").color(Theme::current().text_dim));
        }
        let mut vendors: Vec<&str> = entries.iter().map(|e| e.vendor.as_str()).collect();
        vendors.sort_by_key(|v| v.to_lowercase());
        vendors.dedup();
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            for v in vendors {
                let label = if v.is_empty() { "Unknown vendor" } else { v };
                ui.menu_button(label, |ui| {
                    for e in entries.iter().filter(|e| e.vendor == v) {
                        if ui.button(&e.name).hint(&e.path).clicked() {
                            pick = Some(e.clone());
                            ui.close();
                        }
                    }
                });
            }
        });
        ui.separator();
        scan_button(app, ui);
    });
    pick
}

/// "Find plugins…", or the progress of a running scan.
pub fn scan_button(app: &mut LimeShotApp, ui: &mut Ui) {
    if let Some(s) = &app.plugins.scan {
        ui.label(format!("Scanning plugins… {}/{}", s.done, s.total));
        return;
    }
    let r = ui
        .button("Find plugins…")
        .hint("Scan the standard VST3 folders (each plugin is checked in a separate process)");
    if r.clicked() {
        match std::env::current_exe() {
            Ok(exe) => {
                app.plugins.start_scan(exe);
                app.toast("Scanning for VST3 plugins…");
            }
            Err(e) => app.toast_error(format!("cannot start the scan: {e}")),
        }
        ui.close();
    }
}

/// Add a channel playing the plugin `e` (the Channel Rack's Add menu).
pub fn add_channel(app: &mut LimeShotApp, e: &Entry) {
    if let Some(id) = app.exec(limeshot_core::Command::AddChannel {
        name: e.name.clone(),
        instrument: Instrument::Plugin(e.plugin_ref()),
    }) {
        app.select_channel(ChannelId(id));
        app.sync_engine();
        if let Some(err) = app.plugins.error(Target::Channel(ChannelId(id))) {
            let err = err.to_string();
            app.toast_error(format!("{}: {err}", e.name));
        }
    }
}

/// The plugin a channel or insert runs.
pub(crate) fn plugin_ref(project: &Project, target: Target) -> Option<&PluginRef> {
    project.plugin(target)
}

/// Where a plugin channel or insert's plugin runs, for windows: "Vital (Master)".
pub(crate) fn place_name(project: &Project, target: Target) -> Option<(String, String)> {
    let track_name = |t: usize| {
        project.mixer.tracks.get(t).map(|tr| {
            if tr.name.is_empty() {
                if t == 0 {
                    "Master".to_string()
                } else {
                    format!("Insert {t}")
                }
            } else {
                tr.name.clone()
            }
        })
    };
    match target {
        Target::Channel(c) => {
            let ch = project.channel(c)?;
            Some((ch.name.clone(), track_name(ch.mixer_track as usize)?))
        }
        Target::Effect { track, effect } => {
            let e = project
                .mixer
                .tracks
                .get(track as usize)?
                .effects
                .iter()
                .find(|e| e.id == effect)?;
            let name = e.plugin.as_ref().map(|p| p.name.clone()).unwrap_or_default();
            Some((name, track_name(track as usize)?))
        }
    }
}

/// Channel Settings of a plugin channel.
pub fn channel_settings(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId, p: &PluginRef) {
    header(app, ui, Target::Channel(cid), p);
    params(app, ui, Target::Channel(cid));
}

/// The body of a plugin insert slot's card.
pub fn effect_card(app: &mut LimeShotApp, ui: &mut Ui, track: u16, effect: u32, p: Option<&PluginRef>) {
    let target = Target::Effect { track, effect };
    match p {
        Some(p) => {
            header(app, ui, target, p);
            if limeshot_core::LimePlugin::of(p) == Some(limeshot_core::LimePlugin(7)) {
                let current = p
                    .extra
                    .get("lime-sidechain-track")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u16;
                let mut selected = None;
                ui.menu_button(
                    if current == 0 {
                        "Sidechain: own input".into()
                    } else {
                        format!("Sidechain: Insert {current}")
                    },
                    |ui| {
                        if ui.selectable_label(current == 0, "Own input").clicked() {
                            selected = Some(0);
                            ui.close();
                        }
                        for (index, candidate) in app.project().mixer.tracks.iter().enumerate().skip(1) {
                            if index != usize::from(track)
                                && candidate.routes.iter().any(|r| r.to == track)
                                && ui
                                    .selectable_label(usize::from(current) == index, format!("Insert {index}"))
                                    .clicked()
                            {
                                selected = Some(index as u16);
                                ui.close();
                            }
                        }
                    },
                );
                if let Some(source) = selected {
                    app.exec(Command::SetPluginSidechain { plugin: target, source });
                }
            }
            params(app, ui, target);
        }
        None => {
            ui.label(RichText::new("This slot has no plugin.").color(Theme::current().warning));
        }
    }
}

fn header(app: &mut LimeShotApp, ui: &mut Ui, target: Target, p: &PluginRef) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        ui.label(RichText::new(&p.name).font(semibold(12.5)));
        if !p.vendor.is_empty() {
            ui.label(RichText::new(&p.vendor).color(t.text_dim));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let loaded = app.plugins.is_loaded(target);
            let r = ui
                .add_enabled(loaded, egui::Button::new("Editor").small())
                .hint("Open the plugin's window (its own editor)");
            if r.clicked() {
                app.plugins.open_window(target, Some(crate::plugin_window::Tab::Editor));
            }
        });
    });
    if app.plugins.is_loaded(target)
        && let Some(error) = app.plugins.error(target)
    {
        ui.label(RichText::new(error).color(t.warning));
    }
    let crashed = app.plugins.has_crashed(target);
    match (app.plugins.instance(target), app.plugins.error(target)) {
        (Some(_), _) if crashed => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("The plugin crashed; it is silent.").color(t.warning));
                if ui
                    .button("Reload")
                    .hint("Start the plugin again with the state last saved in the project")
                    .clicked()
                {
                    reload(app, target);
                }
            });
        }
        (Some(inst), _) => {
            let latency = inst.latency_samples();
            let mut text = p.format.clone();
            if inst.is_bridged() {
                text.push_str(" · own process");
            }
            if latency > 0 {
                text.push_str(&format!(" · latency {latency} samples"));
            }
            ui.label(RichText::new(text).color(t.text_faint).small());
        }
        (None, Some(e)) => {
            ui.label(RichText::new(format!("Could not load the plugin: {e}")).color(t.warning));
            if ui
                .button("Reload")
                .hint("Retry loading after fixing the plugin or sample source")
                .clicked()
            {
                reload(app, target);
            }
        }
        (None, None) => {
            ui.label(RichText::new("Loading…").color(t.text_faint));
        }
    }
    // FL Studio runs a plugin "bridged" (Plugin options › Bit size); LimeShot calls it what it
    // does. Switching recreates the plugin with its current state.
    if cfg!(unix) {
        let mut on = p.separate_process;
        if ui
            .checkbox(&mut on, "Run in a separate process")
            .hint("A crash of the plugin then leaves LimeShot running; its own editor is not shown")
            .changed()
        {
            set_separate_process(app, target, on);
        }
    }
}

/// Retry pending sources through Document normalization before restarting a failed/crashed plugin.
/// Reload never approves outside-project audio; explicit source selection is required for that.
pub fn reload(app: &mut LimeShotApp, target: Target) {
    if let Target::Channel(channel) = target {
        let mut candidate = app.project().clone();
        let dir = app.path.as_deref().and_then(std::path::Path::parent);
        crate::lime::restrict_loaded_sources(&mut candidate, dir);
        if let Some(c) = candidate.channel(channel)
            && app
                .project()
                .channel(channel)
                .is_some_and(|c| c.extra.get("lime-external-source").and_then(serde_json::Value::as_bool) == Some(true))
            && c.extra.get("lime-external-source").and_then(serde_json::Value::as_bool) != Some(true)
            && let Some(params) = limeshot_core::lime::sampler_params(&c.instrument)
        {
            app.exec(Command::Batch(vec![
                Command::ApproveSamplerSource { channel },
                Command::SetInstrument {
                    channel,
                    instrument: Instrument::Sampler(params),
                },
            ]));
        }
    }
    if let Some(plugin) = app.project().plugin(target) {
        let state = plugin.state.clone();
        app.exec(Command::SetPluginState { plugin: target, state });
    }
    app.plugins.reload(target);
}

/// Move a plugin into a process of its own or back, keeping its sound: its state goes into the
/// project first, and the new instance loads it.
pub fn set_separate_process(app: &mut LimeShotApp, target: Target, on: bool) {
    if let Some(Ok(s)) = app.plugins.instance(target).map(|i| i.state()) {
        app.doc.store_plugin_state(target, PluginState::new(s));
    }
    app.plugins.keep_window_open(target);
    app.exec(Command::SetPluginSeparateProcess { plugin: target, on });
}

/// The plugin's parameters as sliders, filtered by name.
pub(crate) fn params(app: &mut LimeShotApp, ui: &mut Ui, target: Target) {
    let t = Theme::current();
    let Some(inst) = app.plugins.instance(target) else {
        return;
    };
    let mut filter = app.plugins.filters.get(&target).cloned().unwrap_or_default();
    let needle = filter.to_lowercase();
    let visible: Vec<(u32, String, f64, f64, String)> = inst
        .params()
        .iter()
        .filter(|p| !p.read_only() && !p.hidden() && p.automatable())
        .filter(|p| needle.is_empty() || p.title.to_lowercase().contains(&needle))
        .take(PARAMS_SHOWN + 1)
        .map(|p| {
            let v = inst.param(p.id);
            (p.id, p.title.clone(), v, p.default, inst.param_text(p.id, v))
        })
        .collect();
    let total = inst.params().len();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        section(ui, &format!("Parameters  {total}"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut filter)
                    .hint_text("Filter")
                    .desired_width(120.0),
            );
        });
    });
    app.plugins.filters.insert(target, filter);
    app.plugins.parameter_rects.retain(|(t, _), _| *t != target);
    let mut changes = Vec::new();
    let mut automate = None;
    for (id, title, value, default, text) in visible.iter().take(PARAMS_SHOWN) {
        ui.horizontal(|ui| {
            ui.add_sized(
                vec2(96.0, 20.0),
                egui::Label::new(RichText::new(title).color(t.text_dim)).truncate(),
            );
            let mut v = *value;
            // Leave room for the value as the plugin writes it ("1.95312ms"), so the row never
            // needs more than the window's width.
            let text_w = ui
                .painter()
                .layout_no_wrap(text.clone(), egui::TextStyle::Body.resolve(ui.style()), t.text)
                .size()
                .x;
            ui.spacing_mut().slider_width = (ui.available_width() - text_w.max(40.0) - 24.0).max(40.0);
            let text = text.clone();
            let r = crate::widgets::reset_on_double_click(
                ui.add(egui::Slider::new(&mut v, 0.0..=1.0).custom_formatter(move |_, _| text.clone())),
                &mut v,
                *default,
            );
            app.plugins.parameter_rects.insert((target, *id), r.rect);
            changes.push((
                *id,
                (crate::widgets::user_changed(&r) || r.drag_stopped()).then_some(v),
                r.clone(),
            ));
            let automation = limeshot_core::AutomationTarget::PluginParam {
                plugin: target,
                id: *id,
            };
            if automation.current_value(app.project()).is_some() {
                crate::automation_ui::menu(app, &r, automation);
            } else {
                crate::widgets::clickable(&r).context_menu(|ui| {
                    ui.label(RichText::new(title).small().color(t.text_dim));
                    ui.separator();
                    if ui.button("Create automation clip").clicked() {
                        automate = Some(*id);
                        ui.close();
                    }
                });
            }
        });
    }
    if visible.len() > PARAMS_SHOWN {
        ui.label(
            RichText::new("More parameters: type in the filter to find them")
                .color(t.text_faint)
                .small(),
        );
    }
    for (id, value, response) in changes {
        let command = value.and_then(|value| {
            // Audition drags directly; serialize embedded state only once at release.
            if response.dragged() {
                app.plugins.set_param(target, id, value);
                app.plugins.last_tweaked = Some((target, id));
                return None;
            }
            if let Some(state) = app
                .project()
                .plugin(target)
                .and_then(|plugin| limeshot_core::lime::edited_param_state(plugin, id, value))
            {
                app.plugins.last_tweaked = Some((target, id));
                return Some(Command::SetPluginState { plugin: target, state });
            }
            app.plugins.set_param(target, id, value);
            app.plugins.last_tweaked = Some((target, id));
            app.doc.mark_unsaved();
            None
        });
        app.exec_gesture(&response, command);
    }
    if let Some(id) = automate {
        create_automation(app, target, id);
    }
}

/// FL Studio's "Create automation clip" for a plugin parameter.
pub fn create_automation(app: &mut LimeShotApp, target: Target, id: u32) -> bool {
    let Some((auto, value, name)) = app.plugins.automation_of(app.project(), target, id) else {
        return false;
    };
    crate::automation_ui::create_with(app, auto, Some(value), Some(name)).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offline_export_requires_every_plugin_to_load() {
        let mut doc = Document::default();
        doc.execute(limeshot_core::Command::AddChannel {
            name: "Missing".into(),
            instrument: Instrument::Plugin(PluginRef::vst3("/nonexistent/Missing.vst3", "00", "Missing instrument")),
        })
        .unwrap();
        let host = PluginHost::new(None);
        let error = host
            .offline(doc.project(), 48_000)
            .err()
            .expect("missing plugin must fail export");
        assert!(error.contains("Missing instrument"), "{error}");
        assert!(error.contains("export"), "{error}");
    }

    #[test]
    fn the_database_round_trips_and_splits_by_kind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/plugins.json");
        let mut host = PluginHost::new(Some(path.clone()));
        assert!(host.db.entries.is_empty());
        host.add_builtin_test_plugins();
        host.add_builtin_test_plugins();
        assert_eq!(host.db.entries.len(), 2);
        host.db.failed.push(("/x.vst3".into(), "timed out".into()));
        host.db.save(&path).unwrap();
        let back = Database::load(&path);
        assert_eq!(back, host.db);
        assert_eq!(
            back.instruments().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["Lime Test Tone"]
        );
        assert_eq!(
            back.effects().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["Lime Test Gain"]
        );
        assert_eq!(Database::load(&dir.path().join("missing.json")), Database::default());
    }

    #[test]
    fn a_failed_scan_child_reports_an_error() {
        assert_eq!(scan_child("/nonexistent/X.vst3"), 1);
    }
}
