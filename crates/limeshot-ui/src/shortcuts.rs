//! Keyboard shortcuts. "Cmd" means ⌘ on macOS and Ctrl elsewhere (egui's `COMMAND` modifier).
//! The list is shown in Help › Keyboard shortcuts and documented in `docs/architecture/ui.md`.

use crate::app::{Focus, LimeShotApp, Pending};
use crate::dialogs::Dialog;
use egui::{Context, Key, KeyboardShortcut, Modifiers};

/// Every global shortcut, for the help dialog: (keys, action).
pub const GLOBAL: &[(&str, &str)] = &[
    ("Space", "Play / stop"),
    ("Cmd+T", "Typing keyboard to piano (outside Playlist)"),
    (
        "Z–M / Q–U",
        "Play selected channel when Typing keyboard to piano is enabled",
    ),
    ("Cmd+Space", "Play / pause"),
    ("R", "Record: arm note recording (MIDI input and the typing keyboard)"),
    ("L", "Switch Pattern / Song mode"),
    ("Cmd+M", "Metronome on/off"),
    ("Cmd+H", "Stop all sound (panic)"),
    ("F4 / Cmd+F4", "Find next empty pattern (and name it / without naming)"),
    ("Shift+F4", "Find first empty pattern and name it"),
    ("Cmd+Shift+Insert", "Insert an empty pattern before the current one"),
    ("Cmd+Shift+C", "Clone the current pattern"),
    ("Cmd+Shift+Delete", "Delete the current pattern"),
    (
        "Cmd+Shift+↑ / Cmd+Shift+↓",
        "Move the current pattern up / down the list",
    ),
    ("F5", "Playlist"),
    ("F6", "Channel Rack"),
    ("F7", "Piano Roll"),
    ("F9", "Mixer"),
    ("Alt+F8", "Browser"),
    ("Cmd+Shift+H", "Arrange windows › Realign"),
    ("F12", "Close all windows"),
    ("Alt+F12", "Close all plugin windows"),
    ("Cmd+F12", "Close all unfocused windows"),
    ("Shift+F12", "Align all channel editors (plugin windows)"),
    ("F1", "This list"),
    ("F11", "Project info (settings window)"),
    ("Cmd+F5", "Tempo tapper"),
    (
        "Cmd+Z / Cmd+Alt+Z / Cmd+Shift+Z",
        "Undo / redo (FL Studio: Cmd+Opt+Z; Cmd+Shift+Z and Cmd+Y also redo)",
    ),
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
        "Piano Roll: Quantize… (the selection, or the channel, to the snap)",
    ),
    (
        "Alt+L / Alt+U / Alt+A / Alt+S / Alt+F / Alt+K / Alt+Y / Alt+R / Alt+X / Alt+O",
        "Piano Roll tools: Articulate… / Chop… / Arpeggiate… / Strum… / Flam… / Limit… / Flip… / Randomize… / Scale levels… / LFO…",
    ),
    (
        "↑ / ↓",
        "Piano Roll / Playlist: scroll · Channel Rack: select the previous / next channel",
    ),
    (
        "Shift+↑ / Shift+↓",
        "Piano Roll: transpose a semitone (all of the channel's notes when none is selected)",
    ),
    ("Cmd+↑ / Cmd+↓", "Piano Roll: transpose an octave"),
    (
        "Shift+← / Shift+→",
        "Piano Roll / Playlist: nudge by the snap (Shift+↑/↓ in the Playlist: move clips a track)",
    ),
    (
        "Shift+Cmd+← / Shift+Cmd+→",
        "Channel Rack: rotate the selected channels' steps · Piano Roll: rotate notes by the snap",
    ),
    ("← / →", "Piano Roll / Playlist: scroll"),
    ("Cmd+wheel / pinch", "Piano Roll / Playlist: zoom time"),
    (
        "Opt+wheel / Opt+pinch",
        "Piano Roll / Playlist: zoom vertically (key / track height)",
    ),
    ("Shift+I", "Piano Roll / Playlist: invert the selection"),
    (
        "P / B / D / T / C / E / Z / Y",
        "Piano Roll tools: draw, paint, delete, mute, slice, select, zoom, playback",
    ),
    (
        "P / B / D / T / S / C / E / Z / Y",
        "Playlist tools: draw, paint, delete, mute, slip, slice, select, zoom, playback",
    ),
    ("Alt+G", "Channel Rack: put the selected channels in a filter group"),
    ("Cmd+T", "Playlist: add a time marker at the song position"),
    ("Alt+T", "Playlist: add a time marker and name it"),
    (
        "Cmd+drag on the ruler",
        "Playlist: time selection (Song playback loops it; Cmd+click clears)",
    ),
    ("Cmd+G", "Piano Roll: glue touching notes"),
    ("Cmd+U", "Piano Roll: chop notes at the snap"),
    ("Alt+M / Shift+Alt+M", "Piano Roll: mute / unmute notes"),
    ("Cmd+L", "Piano Roll: quick legato (notes reach the next note)"),
    ("Shift+D", "Piano Roll: discard lengths (one step)"),
    ("Shift+Q", "Piano Roll: quick quantize start times"),
    (
        "Shift+R / Shift+M / Shift+O",
        "Piano Roll: select one at random / more at random / odd notes",
    ),
    ("PgUp / PgDn", "Piano Roll: zoom in / out"),
    (
        "Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5 / Shift+6",
        "Piano Roll: quick zoom 1 bar / 4 / 16 / whole pattern / selection / step size",
    ),
    ("Shift+0", "Piano Roll / Playlist: center on the playback position"),
    ("Alt+M / Shift+Alt+M", "Playlist: mute / unmute the selected clips"),
    ("Alt+← / Alt+→", "Mixer: move the selected track left / right"),
    ("Delete", "Mixer: delete the selected track"),
    ("S", "Mixer: solo the selected track"),
    ("F2", "Mixer: rename the selected track"),
    ("Cmd+G", "Playlist: merge the selected pattern clips into one"),
    (
        "Cmd+Insert / Cmd+Alt+Insert / Cmd+Delete",
        "Playlist: insert space / slice & insert space / delete space (the time selection)",
    ),
    ("Shift+Cmd+V", "Playlist: make the selected clips unique"),
    ("Shift+C", "Playlist: select the clips of the selected source"),
    ("Cmd+Enter", "Playlist: time selection around the selected clips"),
    ("Shift+Q", "Playlist: quick quantize clip start times"),
    ("PgUp / PgDn", "Playlist: zoom in / out"),
    (
        "Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5",
        "Playlist: quick zoom 4 bars / 16 / 64 / whole song / selection",
    ),
    ("Home", "Playlist: song position to the start"),
    (
        "Cmd+L",
        "Channel Rack: route the selected channels to free mixer tracks",
    ),
    ("Alt+↑ / Alt+↓", "Channel Rack: move the selected channels"),
    ("Alt+C", "Channel Rack: clone the selected channels"),
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

/// Handle global shortcuts, then typing-keyboard notes or the focused editor's shortcuts.
/// Text entry and dialogs suppress transport, note, and editor key handling. Typing notes are
/// tracked with their original channel for release by the app's per-frame logic.
pub fn handle(app: &mut LimeShotApp, ctx: &Context) {
    // While typing in a text field, letters and Space belong to the field.
    let typing = ctx.egui_wants_keyboard_input();
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;

    if pressed(ctx, cmd_shift, Key::Z) || pressed(ctx, cmd | Modifiers::ALT, Key::Z) || pressed(ctx, cmd, Key::Y) {
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
    if pressed(ctx, Modifiers::NONE, Key::F11) {
        app.open_settings(crate::settings_window::Page::Info);
    }
    // FL Studio's Patterns menu keys.
    if pressed(ctx, Modifiers::SHIFT, Key::F4) {
        crate::menus::find_empty_pattern(app, true, true);
    } else if pressed(ctx, cmd, Key::F4) {
        crate::menus::find_empty_pattern(app, false, false);
    } else if pressed(ctx, Modifiers::NONE, Key::F4) {
        crate::menus::find_empty_pattern(app, false, true);
    }
    if pressed(ctx, cmd, Key::F5) {
        app.ui.toolbar.tapper = Some(crate::toolbar::Tapper::default());
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
    // FL Studio's View › Layout keys.
    if pressed(ctx, Modifiers::ALT, Key::F12) {
        crate::menus::close_all_plugin_windows(app);
    } else if pressed(ctx, cmd, Key::F12) {
        crate::menus::close_unfocused_windows(app);
    } else if pressed(ctx, Modifiers::SHIFT, Key::F12) {
        crate::menus::align_channel_editors(app, ctx);
    } else if pressed(ctx, Modifiers::NONE, Key::F12) {
        crate::menus::close_all_windows(app);
    }
    if !typing && app.ui.focus != Focus::Playlist && pressed(ctx, cmd, Key::T) {
        app.keyboard_notes = !app.keyboard_notes;
    }
    if typing || app.dialog.is_some() {
        return;
    }
    // Record (FL Studio: R). Only without modifiers (Shift+R is an editor key) and not while the
    // typing keyboard plays notes (R is a key there).
    if !app.keyboard_notes && ctx.input(|i| i.modifiers.is_none()) && pressed(ctx, Modifiers::NONE, Key::R) {
        crate::recording::toggle(app);
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
    if pressed(ctx, cmd_shift, Key::Insert) {
        crate::menus::insert_pattern(app);
    }
    if pressed(ctx, cmd_shift, Key::C) {
        crate::menus::clone_pattern(app);
    }
    if (pressed(ctx, cmd_shift, Key::Delete) || pressed(ctx, cmd_shift, Key::Backspace))
        && app.project().patterns.len() > 1
    {
        crate::menus::delete_pattern(app);
    }
    if pressed(ctx, cmd_shift, Key::ArrowUp) {
        crate::menus::move_pattern(app, -1);
    }
    if pressed(ctx, cmd_shift, Key::ArrowDown) {
        crate::menus::move_pattern(app, 1);
    }
    if pressed(ctx, Modifiers::NONE, Key::Plus) || pressed(ctx, Modifiers::NONE, Key::Equals) {
        step_pattern(app, 1);
    }
    if pressed(ctx, Modifiers::NONE, Key::Minus) {
        step_pattern(app, -1);
    }
    if app.keyboard_notes && !ctx.input(|i| i.modifiers.command || i.modifiers.alt || i.modifiers.ctrl) {
        let mut handled = false;
        if let Some(channel) = app.selected_channel() {
            for (key, offset) in [
                (Key::Z, 0),
                (Key::S, 1),
                (Key::X, 2),
                (Key::D, 3),
                (Key::C, 4),
                (Key::V, 5),
                (Key::G, 6),
                (Key::B, 7),
                (Key::H, 8),
                (Key::N, 9),
                (Key::J, 10),
                (Key::M, 11),
                (Key::Q, 12),
                (Key::Num2, 13),
                (Key::W, 14),
                (Key::Num3, 15),
                (Key::E, 16),
                (Key::R, 17),
                (Key::Num5, 18),
                (Key::T, 19),
                (Key::Num6, 20),
                (Key::Y, 21),
                (Key::Num7, 22),
                (Key::U, 23),
            ] {
                if ctx.input(|i| i.key_pressed(key)) {
                    handled = true;
                    if !app.typing_notes.iter().any(|n| n.0 == key) {
                        let note = 48 + offset;
                        app.preview_on(channel, note, 0.78);
                        app.auditions.retain(|n| *n != (channel, note));
                        app.typing_notes.push((key, channel, note));
                        crate::recording::note_on(app, channel, note, 0.78);
                    }
                }
            }
        }
        if handled {
            return;
        }
    }
    match app.ui.focus {
        Focus::PianoRoll => crate::piano_roll::keys(app, ctx),
        Focus::Playlist => crate::playlist::keys(app, ctx),
        Focus::ChannelRack => crate::channel_rack::keys(app, ctx),
        Focus::Mixer => crate::mixer_menu::keys(app, ctx),
        Focus::None => {}
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
