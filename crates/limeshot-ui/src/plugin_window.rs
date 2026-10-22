//! Resizable plugin wrappers inside the LimeShot workspace. The Plugin tab embeds the native
//! editor as a child of the main window; Settings shows the channel/insert controls. Presets,
//! keyboard forwarding, tab, position and size are remembered with the project. Platforms that
//! cannot embed a native view show the parameter list instead.

use std::collections::HashMap;
use std::hash::{Hash as _, Hasher as _};
use std::path::PathBuf;

use egui::{Context, Pos2, RichText, Ui, Vec2, vec2};
use limeshot_core::{PluginRef, PluginWindow};
use limeshot_plugins::{NativeWindow, Vst3Editor, VstPreset};

use crate::app::LimeShotApp;
use crate::plugins::{PluginHost, Target};
use crate::theme::{Theme, semibold};
use crate::widgets::Hint as _;

/// Height of the tab row above the editor, in egui points.
pub const HEADER: f32 = 30.0;
/// The Settings tab's size until the user resizes it.
const SETTINGS_SIZE: Vec2 = vec2(440.0, 560.0);
/// Frames to wait for a new window to appear before showing the parameter list instead.
const ATTACH_TRIES: u32 = 30;

/// The page a plugin window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The plugin's own editor.
    Editor,
    /// LimeShot's settings for the channel or insert.
    Settings,
}

/// What a closed window remembers for the next time it opens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Memory {
    pub tab: Tab,
    pub position: Option<Pos2>,
    pub settings_size: Vec2,
    pub keyboard_to_plugin: bool,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            tab: Tab::Editor,
            position: None,
            settings_size: SETTINGS_SIZE,
            keyboard_to_plugin: false,
        }
    }
}

impl Memory {
    /// The project file's form of the window (`open`: whether it is open now).
    pub fn to_project(self, open: bool) -> PluginWindow {
        PluginWindow {
            open,
            position: self.position.map(|p| [p.x, p.y]),
            tab: match self.tab {
                Tab::Editor => "plugin",
                Tab::Settings => "settings",
            }
            .into(),
            settings_size: Some([self.settings_size.x, self.settings_size.y]),
            keyboard_to_plugin: self.keyboard_to_plugin,
            extra: Default::default(),
        }
    }

    pub fn from_project(w: &PluginWindow) -> Self {
        Self {
            tab: if w.tab == "settings" {
                Tab::Settings
            } else {
                Tab::Editor
            },
            position: w.position.map(|[x, y]| Pos2::new(x, y)),
            settings_size: w
                .settings_size
                .map(|[x, y]| vec2(x.max(320.0), y.max(240.0)))
                .unwrap_or(SETTINGS_SIZE),
            keyboard_to_plugin: w.keyboard_to_plugin,
        }
    }
}

/// One entry of a plugin's presets menu.
#[derive(Debug, Clone, PartialEq)]
pub enum Preset {
    /// One of the plugin's factory programs (index into its program list).
    Program(usize, String),
    /// A `.vstpreset` file.
    File(PathBuf),
}

impl Preset {
    pub fn name(&self) -> String {
        match self {
            Preset::Program(_, n) => n.clone(),
            Preset::File(p) => p
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
        }
    }
}

/// An open plugin window.
pub struct Wrapper {
    pub tab: Tab,
    raise: bool,
    // Declared before `native`: the editor leaves the window before LimeShot lets go of it.
    editor: Option<Vst3Editor>,
    native: Option<NativeWindow>,
    /// Whether the plugin was asked for its editor yet (on the first frame).
    asked: bool,
    /// No native editor can be shown: the Plugin tab lists the parameters.
    fallback: bool,
    attach_tries: u32,
    /// Where the window opens (fixed while it is open, so it is not moved back each frame).
    open_at: Option<Pos2>,
    position: Option<Pos2>,
    settings_size: Vec2,
    /// A hash of the plugin's state when the window opened, to notice changes the plugin did not
    /// report.
    state_at_open: Option<u64>,
    /// FL's keyboard toggle: keystrokes go to the plugin before LimeShot's shortcuts.
    pub keyboard_to_plugin: bool,
    /// The presets menu, read when the window first opens (and after Save preset as…).
    presets: Option<Vec<Preset>>,
    /// The preset last chosen in this window, an index into `presets`.
    preset: Option<usize>,
    /// Move the window here on the next frame (View › Align all channel editors).
    move_to: Option<Pos2>,
}

impl Wrapper {
    fn new(memory: Option<Memory>) -> Self {
        let m = memory.unwrap_or_default();
        Self {
            tab: m.tab,
            raise: true,
            editor: None,
            native: None,
            asked: false,
            fallback: false,
            attach_tries: 0,
            open_at: m.position,
            position: m.position,
            settings_size: m.settings_size,
            state_at_open: None,
            keyboard_to_plugin: m.keyboard_to_plugin,
            presets: None,
            preset: None,
            move_to: None,
        }
    }

    fn memory(&self) -> Memory {
        Memory {
            tab: self.tab,
            position: self.position,
            settings_size: self.settings_size,
            keyboard_to_plugin: self.keyboard_to_plugin,
        }
    }

    /// The presets menu's entries (empty until the window's first frame).
    pub fn presets(&self) -> &[Preset] {
        self.presets.as_deref().unwrap_or_default()
    }

    /// The preset last chosen, or the plugin's current factory program.
    pub fn current_preset(&self) -> Option<usize> {
        self.preset
    }

    /// Whether the plugin has an editor of its own (known from the window's first frame).
    pub fn has_editor(&self) -> bool {
        self.editor.is_some()
    }

    /// Whether the Plugin tab shows the plugin's own editor.
    pub fn shows_native_editor(&self) -> bool {
        self.editor.as_ref().is_some_and(|e| e.is_attached())
    }
}

fn state_hash(host: &PluginHost, target: Target) -> Option<u64> {
    let state = host.instance(target)?.state().ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    state.hash(&mut h);
    Some(h.finish())
}

impl PluginHost {
    /// Open the window of the plugin at `target` (or bring it to `tab` if it is open).
    pub fn open_window(&mut self, target: Target, tab: Option<Tab>) {
        if !self.is_loaded(target) {
            return;
        }
        let memory = self.window_memory.get(&target).copied();
        let w = self.windows.entry(target).or_insert_with(|| Wrapper::new(memory));
        w.raise = true;
        if let Some(tab) = tab {
            w.tab = tab;
        }
    }

    /// Close the window of the plugin at `target`. Returns true when the plugin's state changed
    /// while it was open (the project is then modified).
    pub fn close_window(&mut self, target: Target) -> bool {
        let Some(w) = self.windows.remove(&target) else {
            return false;
        };
        self.window_memory.insert(target, w.memory());
        let before = w.state_at_open;
        drop(w);
        before.is_some() && before != state_hash(self, target)
    }

    /// FL Studio: a click on the channel button (or insert slot) opens the window, the next closes it.
    pub fn toggle_window(&mut self, target: Target) -> bool {
        if self.is_window_open(target) {
            self.close_window(target)
        } else {
            self.open_window(target, None);
            false
        }
    }

    pub fn is_window_open(&self, target: Target) -> bool {
        self.windows.contains_key(&target)
    }

    pub fn window(&self, target: Target) -> Option<&Wrapper> {
        self.windows.get(&target)
    }

    /// The project file's form of the window of the plugin at `target`: open, closed (with what
    /// it remembers) or never opened (`None`).
    pub fn window_place(&self, target: Target) -> Option<PluginWindow> {
        match self.windows.get(&target) {
            Some(w) => Some(w.memory().to_project(true)),
            None => self.window_memory.get(&target).map(|m| m.to_project(false)),
        }
    }

    /// Take the window places saved in a project that was just opened, reopening the windows
    /// that were open.
    pub(crate) fn restore_windows<'a>(&mut self, plugins: impl IntoIterator<Item = (Target, &'a PluginRef)>) {
        for (target, p) in plugins {
            let Some(w) = &p.window else { continue };
            self.window_memory.insert(target, Memory::from_project(w));
            if w.open {
                self.open_window(target, None);
            }
        }
    }

    /// View › Close all plugin windows (FL: Opt+F12). Returns true when a plugin changed while
    /// its window was open.
    pub fn close_all_windows(&mut self) -> bool {
        let open: Vec<Target> = self.windows.keys().copied().collect();
        open.into_iter()
            .fold(false, |changed, t| self.close_window(t) | changed)
    }

    /// View › Align all channel editors (FL: Shift+F12): stack the open plugin windows from
    /// `origin`, each a title bar lower and to the right.
    pub fn align_windows(&mut self, origin: Pos2) {
        let mut open: Vec<Target> = self.windows.keys().copied().collect();
        open.sort_by_key(order);
        for (i, t) in open.into_iter().enumerate() {
            if let Some(w) = self.windows.get_mut(&t) {
                let at = origin + vec2(28.0, 28.0) * i as f32;
                w.move_to = Some(at);
                w.position = Some(at);
            }
        }
    }

    /// FL Studio's Tools › Macros › Set keyboard focus for all plugins to: the keyboard toggle of
    /// every plugin window, open or closed.
    pub fn set_keyboard_for_all(&mut self, to_plugin: bool) {
        let targets: Vec<Target> = self.live_targets();
        for t in targets {
            match self.windows.get_mut(&t) {
                Some(w) => w.keyboard_to_plugin = to_plugin,
                None => {
                    self.window_memory.entry(t).or_default().keyboard_to_plugin = to_plugin;
                }
            }
        }
    }

    /// Step through the presets menu (FL's Presets ◀ ▶), wrapping at the ends. Returns true when
    /// a preset was loaded.
    pub fn step_preset(&mut self, target: Target, by: isize) -> bool {
        let Some(w) = self.windows.get(&target) else {
            return false;
        };
        let n = w.presets().len() as isize;
        if n == 0 {
            return false;
        }
        let next = match w.preset {
            Some(i) => (i as isize + by).rem_euclid(n),
            None if by < 0 => n - 1,
            None => 0,
        };
        self.load_preset(target, next as usize)
    }

    /// Load entry `index` of the window's presets menu. Returns true when it loaded.
    pub fn load_preset(&mut self, target: Target, index: usize) -> bool {
        let Some(preset) = self.windows.get(&target).and_then(|w| w.presets().get(index)).cloned() else {
            return false;
        };
        let Some(inst) = self.instance_mut(target) else {
            return false;
        };
        let loaded = match &preset {
            Preset::Program(i, _) => {
                inst.set_program(*i);
                true
            }
            Preset::File(path) => match VstPreset::read(path).and_then(|p| inst.load_preset(&p)) {
                Ok(()) => true,
                Err(e) => {
                    log::warn!("{}: {e}", path.display());
                    false
                }
            },
        };
        if loaded && let Some(w) = self.windows.get_mut(&target) {
            w.preset = Some(index);
        }
        loaded
    }

    /// Read the presets menu for the window at `target`: the plugin's factory programs, then the
    /// `.vstpreset` files made for it in the standard preset folders.
    fn read_presets(&mut self, target: Target, plugin: Option<&PluginRef>) {
        let (programs, current) = match self.instance(target) {
            Some(i) => (i.program_names(), i.program()),
            None => (Vec::new(), None),
        };
        let mut list: Vec<Preset> = programs
            .into_iter()
            .enumerate()
            .map(|(i, n)| Preset::Program(i, n))
            .collect();
        if let Some(p) = plugin {
            let dirs = limeshot_plugins::preset::preset_dirs();
            list.extend(
                limeshot_plugins::preset::find_presets(&dirs, &p.vendor, &p.name, &p.class_id)
                    .into_iter()
                    .map(Preset::File),
            );
        }
        if let Some(w) = self.windows.get_mut(&target) {
            w.preset = w.preset.or(current).filter(|&i| i < list.len());
            w.presets = Some(list);
        }
    }

    /// Save the plugin's state as a `.vstpreset` at `path`, and list it in the presets menu.
    pub fn save_preset(
        &mut self,
        target: Target,
        path: &std::path::Path,
        plugin: Option<&PluginRef>,
    ) -> Result<(), String> {
        let inst = self.instance(target).ok_or("the plugin is not running")?;
        let bytes = inst.preset().map_err(|e| e.to_string())?.to_bytes();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
        self.read_presets(target, plugin);
        if let Some(w) = self.windows.get_mut(&target) {
            w.preset = w
                .presets()
                .iter()
                .position(|p| matches!(p, Preset::File(f) if f == path));
        }
        Ok(())
    }
}

/// Close a window and mark the project modified if the plugin changed meanwhile.
pub fn close(app: &mut LimeShotApp, target: Target) {
    if app.plugins.close_window(target) {
        app.doc.mark_unsaved();
    }
}

/// Open or close a window (the channel button, an insert's name).
pub fn toggle(app: &mut LimeShotApp, target: Target) {
    if app.plugins.toggle_window(target) {
        app.doc.mark_unsaved();
    }
}

fn order(t: &Target) -> (u8, u32, u32) {
    match *t {
        Target::Channel(c) => (0, c.0, 0),
        Target::Effect { track, effect } => (1, u32::from(track), effect),
    }
}

/// The window titles, FL's "Plugin (mixer track)", made unique so each window can be found.
fn titles(app: &LimeShotApp, targets: &[Target]) -> Vec<String> {
    let mut seen: HashMap<String, u32> = HashMap::new();
    targets
        .iter()
        .map(|&t| {
            let base = match crate::plugins::place_name(app.project(), t) {
                Some((name, place)) => format!("{name} ({place})"),
                None => "Plugin".to_string(),
            };
            let n = seen.entry(base.clone()).or_insert(0);
            *n += 1;
            if *n == 1 { base } else { format!("{base} #{n}") }
        })
        .collect()
}

/// Show every open plugin window. Called once per frame from the main window.
pub fn show_all(app: &mut LimeShotApp, ctx: &Context) {
    let mut targets: Vec<Target> = app.plugins.windows.keys().copied().collect();
    targets.sort_by_key(order);
    let titles = titles(app, &targets);
    for (target, title) in targets.into_iter().zip(titles) {
        show(app, ctx, target, &title);
    }
}

fn show(app: &mut LimeShotApp, ctx: &Context, target: Target, title: &str) {
    let zoom = ctx.zoom_factor();
    if app.plugins.windows.get(&target).is_some_and(|w| !w.asked) {
        let editor = app.plugins.instance(target).and_then(|i| i.create_editor());
        let state = state_hash(&app.plugins, target);
        if let Some(w) = app.plugins.windows.get_mut(&target) {
            w.asked = true;
            w.fallback = editor.is_none();
            w.editor = editor;
            w.state_at_open = state;
        }
        let plugin = crate::plugins::plugin_ref(app.project(), target).cloned();
        app.plugins.read_presets(target, plugin.as_ref());
    }
    let Some(w) = app.plugins.windows.get(&target) else {
        return;
    };
    let editor_size = w
        .editor
        .as_ref()
        .filter(|_| !w.fallback && w.tab == Tab::Editor)
        .map(|e| e.size());
    let size = editor_size.map_or(w.settings_size, |s| {
        vec2(s.width as f32 / zoom, s.height as f32 / zoom + HEADER)
    });
    let id = egui::Id::new(("limeshot-plugin", order(&target)));
    let mut window = egui::Window::new(title)
        .id(id)
        .resizable(true)
        .collapsible(false)
        .min_size(
            w.editor
                .as_ref()
                .filter(|e| w.tab == Tab::Editor && !w.fallback && !e.can_resize())
                .map_or(vec2(320.0, 200.0), |e| {
                    let size = e.size();
                    vec2(
                        size.width as f32 / zoom + 24.0,
                        size.height as f32 / zoom + HEADER + 72.0,
                    )
                }),
        )
        .default_size(size)
        .constrain_to(app.ui.workspace.area);
    if let Some(p) = w.open_at {
        window = window.default_pos(p);
    }
    let move_to = app.plugins.windows.get_mut(&target).and_then(|w| w.move_to.take());
    if let Some(p) = move_to {
        window = window.current_pos(p);
    }
    let mut open = true;
    let response = window.open(&mut open).show(ctx, |ui| {
        let tab = tabs(app, ui, target);
        let area = ui.available_rect_before_wrap();
        let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(*ui.layout()));
        inner.set_clip_rect(area.intersect(ui.clip_rect()));
        body(app, &mut inner, target);
        ui.advance_cursor_after_rect(area);
        (tab, area)
    });
    if !open {
        close(app, target);
        return;
    }
    let main_title = app.window_title();
    let Some(w) = app.plugins.windows.get_mut(&target) else {
        return;
    };
    let Some(response) = response else { return };
    w.position = Some(response.response.rect.min);
    if std::mem::take(&mut w.raise) {
        ctx.move_to_top(response.response.layer_id);
    }
    if let Some((tab, area)) = response.inner {
        w.settings_size = vec2(area.width(), area.height() + HEADER);
        if let Some(tab) = tab {
            w.tab = tab;
        }
        if w.native.is_none() && !w.fallback {
            w.native = NativeWindow::find(&main_title);
            if w.native.is_none() {
                w.attach_tries += 1;
                if w.attach_tries > ATTACH_TRIES {
                    w.fallback = true;
                }
            }
        }
        if let (Some(editor), Some(native)) = (&mut w.editor, &w.native) {
            if !editor.is_attached()
                && w.tab == Tab::Editor
                && !w.fallback
                && let Err(e) = editor.attach_embedded(native)
            {
                log::warn!("{title}: {e}");
                w.fallback = true;
            }
            let visible = w.tab == Tab::Editor
                && !w.fallback
                && app.dialog.is_none()
                && !ctx.any_popup_open()
                && [
                    area.shrink(3.0).left_top(),
                    area.shrink(3.0).right_top(),
                    area.shrink(3.0).left_bottom(),
                    area.shrink(3.0).right_bottom(),
                    area.center(),
                ]
                .iter()
                .all(|p| ctx.layer_id_at(*p) == Some(response.response.layer_id));
            editor.set_visible(visible);
            if visible {
                editor.layout_embedded(
                    f64::from(area.left() * zoom),
                    f64::from(area.top() * zoom),
                    f64::from(area.width() * zoom),
                    f64::from(area.height() * zoom),
                );
            }
            if editor.take_resized().is_some() {
                ctx.request_repaint();
            }
        }
    }
}

/// Forward keys before global shortcuts now that wrappers share the main viewport.
pub(crate) fn handle_keys(app: &LimeShotApp, ctx: &Context) -> bool {
    if app.keyboard_notes || app.dialog.is_some() || ctx.egui_wants_keyboard_input() {
        return false;
    }
    let top = ctx.memory(|m| m.areas().top_layer_id(egui::Order::Middle));
    app.plugins.windows.keys().copied().any(|target| {
        let id = egui::Id::new(("limeshot-plugin", order(&target)));
        top.is_some_and(|layer| layer.id == id) && keys_to_plugin(app, ctx, target)
    })
}

/// With the keyboard toggle on, typed characters go to the plugin's editor first. Returns true
/// when the plugin took one, so LimeShot's shortcuts leave this frame's keys alone.
fn keys_to_plugin(app: &LimeShotApp, ctx: &Context, target: Target) -> bool {
    let Some(w) = app.plugins.window(target) else {
        return false;
    };
    if !w.keyboard_to_plugin {
        return false;
    }
    let Some(ed) = w.editor.as_ref().filter(|e| e.is_attached()) else {
        return false;
    };
    let typed: Vec<char> = ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::Text(t) => Some(t.chars().collect::<Vec<_>>()),
                _ => None,
            })
            .flatten()
            .collect()
    });
    let mut taken = false;
    for c in typed {
        taken |= ed.key_down(c);
    }
    taken
}

/// What the right end of the tab row asked for.
enum Action {
    Preset(isize),
    Load(usize),
    SaveAs,
    Keyboard,
}

/// The tab row. Returns the tab clicked.
fn tabs(app: &mut LimeShotApp, ui: &mut Ui, target: Target) -> Option<Tab> {
    let t = Theme::current();
    let w = app.plugins.window(target)?;
    let current = w.tab;
    let plugin_name = match target {
        Target::Channel(c) => app.project().channel(c).and_then(|c| match &c.instrument {
            limeshot_core::Instrument::Plugin(p) => Some(p.name.clone()),
            _ => None,
        }),
        Target::Effect { .. } => None,
    };
    let mut pick = None;
    let mut act = None;
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        vec2(width, HEADER),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_min_size(vec2(width, HEADER));
            ui.add_space(6.0);
            for (tab, label, hint) in [
                (Tab::Editor, "Plugin", "The plugin's own editor"),
                (
                    Tab::Settings,
                    "Settings",
                    match target {
                        Target::Channel(_) => "LimeShot's Channel Settings for this channel",
                        Target::Effect { .. } => "LimeShot's settings for this insert",
                    },
                ),
            ] {
                let on = current == tab;
                let text = RichText::new(label)
                    .font(semibold(12.0))
                    .color(if on { t.accent } else { t.text_dim });
                if ui
                    .add(egui::Button::new(text).selected(on).frame(on))
                    .hint(hint)
                    .clicked()
                    && !on
                {
                    pick = Some(tab);
                }
            }
            let mut action = None;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);
                action = title_buttons(ui, w, target, plugin_name.as_deref());
            });
            act = action;
        },
    );
    if let Some(a) = act {
        run(app, target, a);
    }
    pick
}

/// The presets menu, Presets ◀ ▶ and the keyboard toggle, right to left.
fn title_buttons(ui: &mut Ui, w: &Wrapper, target: Target, plugin_name: Option<&str>) -> Option<Action> {
    let t = Theme::current();
    let mut action = None;
    let on = w.keyboard_to_plugin;
    let key = RichText::new("⌨")
        .size(15.0)
        .color(if on { t.accent } else { t.text_dim });
    if ui
        .add(egui::Button::new(key).selected(on).frame(on))
        .hint("Allow the plugin to handle keystrokes before LimeShot does")
        .clicked()
    {
        action = Some(Action::Keyboard);
    }
    let has = !w.presets().is_empty();
    if matches!(target, Target::Channel(_)) {
        for (label, by, hint) in [("▶", 1, "Next preset"), ("◀", -1, "Previous preset")] {
            if ui
                .add_enabled(has, egui::Button::new(label).frame(false))
                .hint(hint)
                .clicked()
            {
                action = Some(Action::Preset(by));
            }
        }
    }
    let current = w.preset.and_then(|i| w.presets().get(i)).map(Preset::name);
    let label = current
        .or(plugin_name.map(str::to_owned))
        .unwrap_or_else(|| "Presets".into());
    let label = RichText::new(format!("{label} ⏷")).color(t.text_faint);
    egui::containers::menu::MenuButton::new(label)
        .ui(ui, |ui| {
            ui.set_min_width(200.0);
            ui.label(RichText::new("Presets").font(semibold(11.0)).color(t.text_faint));
            if !has {
                ui.add_enabled(false, egui::Button::new("No presets found"));
            }
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                let mut files = false;
                for (i, p) in w.presets().iter().enumerate() {
                    if matches!(p, Preset::File(_)) && !files {
                        files = true;
                        ui.separator();
                    }
                    if ui
                        .add(egui::Button::new(p.name()).selected(w.preset == Some(i)))
                        .clicked()
                    {
                        action = Some(Action::Load(i));
                        ui.close();
                    }
                }
            });
            ui.separator();
            if ui.button("Save preset as…").clicked() {
                action = Some(Action::SaveAs);
                ui.close();
            }
        })
        .0
        .hint("The plugin's presets");
    action
}

fn run(app: &mut LimeShotApp, target: Target, action: Action) {
    let changed = match action {
        Action::Preset(by) => app.plugins.step_preset(target, by),
        Action::Load(i) => app.plugins.load_preset(target, i),
        Action::Keyboard => {
            if let Some(w) = app.plugins.windows.get_mut(&target) {
                w.keyboard_to_plugin = !w.keyboard_to_plugin;
            }
            false
        }
        Action::SaveAs => {
            let plugin = crate::plugins::plugin_ref(app.project(), target).cloned();
            let dir = plugin
                .as_ref()
                .and_then(|p| limeshot_plugins::preset::user_preset_dir(&p.vendor, &p.name));
            let mut dialog = rfd::FileDialog::new().add_filter("VST3 preset", &["vstpreset"]);
            if let Some(d) = &dir {
                let _ = std::fs::create_dir_all(d);
                dialog = dialog.set_directory(d);
            }
            if let Some(mut path) = dialog.save_file() {
                path.set_extension("vstpreset");
                match app.plugins.save_preset(target, &path, plugin.as_ref()) {
                    Ok(()) => app.toast(format!("Saved preset {}", path.display())),
                    Err(e) => app.toast_error(format!("Could not save the preset: {e}")),
                }
            }
            false
        }
    };
    if changed {
        app.doc.mark_unsaved();
    }
}

fn body(app: &mut LimeShotApp, ui: &mut Ui, target: Target) {
    let t = Theme::current();
    let Some(w) = app.plugins.window(target) else {
        return;
    };
    match w.tab {
        Tab::Editor if w.shows_native_editor() => {
            // The plugin's view covers this area (or will on the next frame).
        }
        Tab::Editor => {
            let has_editor = w.editor.is_some();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    let note = if has_editor {
                        "The plugin's editor cannot be shown here; its parameters:"
                    } else {
                        "This plugin has no editor of its own; its parameters:"
                    };
                    ui.label(RichText::new(note).color(t.text_dim));
                    crate::plugins::params(app, ui, target);
                });
            });
        }
        Tab::Settings => match target {
            Target::Channel(c) => crate::inspector::channel_page(app, ui, c),
            Target::Effect { track, effect } => {
                let plugin = app
                    .project()
                    .mixer
                    .tracks
                    .get(usize::from(track))
                    .and_then(|tr| tr.effects.iter().find(|e| e.id == effect))
                    .and_then(|e| e.plugin.clone());
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        crate::plugins::effect_card(app, ui, track, effect, plugin.as_ref());
                    });
                });
            }
        },
    }
}
