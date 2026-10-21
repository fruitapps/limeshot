//! Keyboard shortcuts. "Cmd" means ⌘ on macOS and Ctrl elsewhere (egui's `COMMAND` modifier).
//! The list is shown in Help › Keyboard shortcuts and documented in `docs/architecture/ui.md`.

use crate::app::{Focus, LimeShotApp, Pending};
use crate::dialogs::Dialog;
use egui::{Context, Key, KeyboardShortcut, Modifiers};

/// Every global shortcut, for the help dialog: (keys, action).
pub const GLOBAL: &[(&str, &str)] = &[
    ("Space", "Play / stop"),
    ("Cmd+Space", "Play / pause"),
    ("L", "Switch Pattern / Song mode"),
    ("Cmd+M", "Metronome on/off"),
    ("Cmd+H", "Stop all sound (panic)"),
    ("F4", "New pattern"),
    ("F5", "Playlist"),
    ("F6", "Channel Rack"),
    ("F7", "Piano Roll"),
    ("F9", "Mixer"),
    ("Alt+F8", "Browser"),
    ("Cmd+Shift+H", "Arrange windows"),
    ("F1", "This list"),
    ("Cmd+Z / Cmd+Shift+Z", "Undo / redo"),
    ("Cmd+N / Cmd+O", "New / open project"),
    ("Cmd+S / Cmd+Shift+S", "Save / save as"),
    ("Cmd+R", "Export WAV"),
    ("Cmd+Q", "Quit"),
    ("Cmd+= / Cmd+- / Cmd+0", "UI zoom"),
    ("+ / −", "Next / previous pattern"),
];

/// Editing shortcuts that apply to the focused editor.
pub const EDITING: &[(&str, &str)] = &[
    ("Delete / Backspace", "Delete selected notes or clips"),
    ("Cmd+A", "Select all"),
    ("Cmd+D", "Deselect"),
    ("Cmd+B", "Duplicate selection to the right"),
    (
        "Cmd+C / Cmd+X / Cmd+V",
        "Copy / cut / paste notes, steps (Channel Rack) or clips (Playlist; pastes at the song position in Song mode)",
    ),
    (
        "Alt+Q",
        "Piano Roll: quantize the selection (or the channel) to the snap",
    ),
    (
        "↑ / ↓",
        "Piano Roll: transpose a semitone · Channel Rack: select the previous / next channel",
    ),
    ("Cmd+↑ / Cmd+↓", "Piano Roll: transpose an octave"),
    ("← / →", "Piano Roll / Playlist: nudge by the snap"),
    ("Home", "Playlist: song position to the start"),
    (
        "Cmd+L",
        "Channel Rack: route the selected channels to free mixer tracks",
    ),
    ("Alt+↑ / Alt+↓", "Channel Rack: move the selected channels"),
    ("Alt+Delete", "Channel Rack: delete the selected channels"),
    ("F2", "Channel Rack: rename the current pattern"),
    (
        "Cmd+click / Shift+click / right-click selector LED",
        "Channel Rack: add a channel to the selection or remove it",
    ),
    ("Alt+click", "Edit without snapping"),
];

/// A shortcut label for display ("Cmd" becomes ⌘ on macOS).
pub fn label(s: &str) -> String {
    if cfg!(target_os = "macos") {
        s.replace("Cmd+", "⌘").replace("Shift+", "⇧").replace("Alt+", "⌥")
    } else {
        s.replace("Cmd+", "Ctrl+")
    }
}

fn pressed(ctx: &Context, mods: Modifiers, key: Key) -> bool {
    ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(mods, key)))
}

pub fn handle(app: &mut LimeShotApp, ctx: &Context) {
    // While typing in a text field, letters and Space belong to the field.
    let typing = ctx.egui_wants_keyboard_input();
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;

    if pressed(ctx, cmd_shift, Key::Z) || pressed(ctx, cmd, Key::Y) {
        app.doc.redo();
    }
    if !typing && pressed(ctx, cmd, Key::Z) {
        app.doc.undo();
    }
    if pressed(ctx, cmd_shift, Key::S) {
        app.save_as();
    }
    if pressed(ctx, cmd, Key::S) {
        app.save();
    }
    if pressed(ctx, cmd, Key::O) {
        app.request(Pending::Open(None));
    }
    if pressed(ctx, cmd, Key::N) {
        app.request(Pending::New);
    }
    if pressed(ctx, cmd, Key::R) {
        app.dialog = Some(Dialog::export(app));
    }
    if pressed(ctx, cmd, Key::Q) {
        app.request(Pending::Quit);
    }
    if pressed(ctx, cmd, Key::M) {
        let m = !app.ui.metronome;
        app.set_metronome(m);
    }
    if pressed(ctx, cmd_shift, Key::H) {
        app.ui.workspace.reset_layout = true;
    }
    if pressed(ctx, cmd, Key::H) {
        app.send(limeshot_audio::Msg::Panic);
    }
    if pressed(ctx, cmd, Key::Equals) || pressed(ctx, cmd, Key::Plus) {
        app.settings.ui_scale = (app.settings.ui_scale + 0.1).min(3.0);
    }
    if pressed(ctx, cmd, Key::Minus) {
        app.settings.ui_scale = (app.settings.ui_scale - 0.1).max(0.5);
    }
    if pressed(ctx, cmd, Key::Num0) {
        app.settings.ui_scale = 1.0;
    }
    if pressed(ctx, Modifiers::NONE, Key::F1) {
        app.dialog = Some(Dialog::Shortcuts);
    }
    if pressed(ctx, Modifiers::NONE, Key::F4) {
        crate::menus::add_pattern(app);
    }
    use crate::workspace::Win;
    for (k, w) in [
        (Key::F5, Win::Playlist),
        (Key::F6, Win::ChannelRack),
        (Key::F7, Win::PianoRoll),
        (Key::F9, Win::Mixer),
    ] {
        if pressed(ctx, Modifiers::NONE, k) {
            app.toggle_window(w);
        }
    }
    if pressed(ctx, Modifiers::ALT, Key::F8) {
        app.ui.show_browser ^= true;
    }
    if typing || app.dialog.is_some() {
        return;
    }
    if pressed(ctx, cmd, Key::Space) {
        app.toggle_play();
    } else if pressed(ctx, Modifiers::NONE, Key::Space) {
        if app.is_playing() {
            app.stop();
        } else {
            app.toggle_play();
        }
    }
    if pressed(ctx, Modifiers::NONE, Key::L) {
        app.toggle_mode();
    }
    if pressed(ctx, Modifiers::NONE, Key::Plus) || pressed(ctx, Modifiers::NONE, Key::Equals) {
        step_pattern(app, 1);
    }
    if pressed(ctx, Modifiers::NONE, Key::Minus) {
        step_pattern(app, -1);
    }
    match app.ui.focus {
        Focus::PianoRoll => crate::piano_roll::keys(app, ctx),
        Focus::Playlist => crate::playlist::keys(app, ctx),
        Focus::ChannelRack => crate::channel_rack::keys(app, ctx),
        Focus::Mixer | Focus::None => {}
    }
}

fn step_pattern(app: &mut LimeShotApp, delta: i32) {
    let pats = &app.project().patterns;
    let cur = app.current_pattern();
    let i = pats.iter().position(|p| p.id == cur).unwrap_or(0) as i32;
    let j = (i + delta).clamp(0, pats.len() as i32 - 1) as usize;
    if let Some(p) = pats.get(j) {
        let id = p.id;
        app.set_current_pattern(id);
    }
}

/// Consume a key with exact modifiers, for the editors.
pub fn key(ctx: &Context, mods: Modifiers, key: Key) -> bool {
    pressed(ctx, mods, key)
}
