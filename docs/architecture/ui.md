# User interface

Source: `crates/limeshot-ui`. Framework: egui/eframe 0.36 (glow renderer for the app; wgpu for offscreen test renders).

## Structure

`LimeShotApp` (`app.rs`) holds the layers separately:

| Layer | Field | Changes through |
|---|---|---|
| Persistent project | `doc: Document` | `exec(Command)` / `exec_gesture` only |
| Configuration | `settings: Settings` | menus/dialogs; saved to the platform config folder |
| Engine/transport | owned by the audio thread | `send(Msg)`; read via `player.status()` |
| Runtime resources | `samples: SampleCache` | `request` / `poll` |
| Ephemeral UI | `ui: UiState` (windows, current pattern, selection, zoom, focus) | direct mutation |

Each frame, `logic()` applies the theme once, sets the UI zoom, collects engine garbage, recompiles the snapshot if the
project changed, polls export jobs, updates the window title and handles close requests. Then `ui()` draws:

* `menus::menu_bar`: File, Edit, Add, Patterns, View, Options, Help;
* `toolbar::show`: logo · PAT/SONG · play/pause · stop · record (disabled) · tempo · time signature · position · pattern
  selector · snap · metronome · loop · window toggles · undo/redo/save/export · master meter · load;
* the left `browser` panel (resizable);
* the central workspace, where the editors are egui `Window`s constrained to it: `playlist`, `channel_rack` (plus the
  Channel Settings window), `piano_roll` and `mixer`;
* `dialogs` (save prompt, export, audio settings, rename, about, shortcuts), toasts, and the status bar with hover hints,
  voice count and device.

Editors are custom-painted (`ui.painter_at`) with a single `interact` region per area, so they stay fast with many notes.
Hit testing uses each editor's `View` (tick ↔ x, key/track ↔ y).

## Interaction conventions

Chosen after the FL Studio manual (see `docs/research/workflow-comparisons.md`):

| Where | Action |
|---|---|
| Step buttons | left-click/drag paints on, right-click/drag paints off; turning a step on auditions it |
| Piano Roll | left on empty adds a note (snapped) and keeps dragging it; drag a note moves; drag right edge resizes; right-click/drag deletes; Cmd+click toggles selection; Cmd+drag rectangle; double-click a note selects its pitch row; Alt bypasses snap |
| Piano keys | click/drag auditions |
| Velocity lane | click/drag sets the nearest stem (and the whole selection if that note is selected) |
| Playlist | left on empty places the current pattern and keeps dragging it; drag moves (time and track); right edge resizes; right-click/drag deletes; ▾ on a clip opens its menu (Make unique, Edit, Rename, Mute, Fit, Duplicate, Delete); double-click opens the pattern in the Piano Roll; ruler click/drag sets the song position |
| Knobs and faders | drag vertically, Shift for fine control, wheel steps, double-click resets |
| Mute LEDs/buttons | left-click mutes; right-click or Cmd+click solos |
| Scrolling | wheel vertical, Shift+wheel horizontal, Cmd+wheel zooms time around the pointer |

Continuous gestures call `Document::begin_gesture` on drag start and `end_gesture` on release, so a drag is one undo step.

## Keyboard

Global: Space play/stop, Cmd+Space play/pause, L Pattern/Song, Cmd+M metronome, Cmd+H panic, F4 new pattern, F5 Playlist,
F6 Channel Rack, F7 Piano Roll, F9 Mixer, Alt+F8 Browser, F1 shortcut list, Cmd+Shift+H arrange windows,
Cmd+Z / Cmd+Shift+Z (or Cmd+Y) undo/redo, Cmd+N/O/S/Shift+S/R new/open/save/save as/export, Cmd+=/−/0 zoom,
+/− next/previous pattern.

Focused editor (the last one clicked): Delete/Backspace, Cmd+A, Cmd+D, Cmd+B duplicate, arrows (transpose/nudge in the
Piano Roll, nudge/move in the Playlist, channel selection in the rack), Cmd+L route the channel to a free insert.

Single-key shortcuts are ignored while a text field has focus.

## Theme

`theme.rs` defines one `Theme` token set (Lime Dark: charcoal/graphite surfaces, lime accent) and applies it to egui's
visuals. Widgets read colors only from `Theme::current()`. Icons and the logo are painter code (`icons.rs`). The window
icon is rasterised in code (`limeshot_app::icon_rgba`).

## Testing the UI

* `crates/limeshot-app/tests/ui.rs` drives the real app with `egui_kittest`: synthetic keys and pointer events, no
  window, a silent audio clock. It covers window shortcuts, new pattern + undo, transport, step clicks, playlist
  placement/deletion, and the Piano Roll add/drag/delete/undo sequence. Pointer positions match the default
  1440×900 layout. If the layout changes, re-render the reference screenshot and update them.
* `cargo run -p limeshot-app --example ui_shot -- out.png [--piano] [--mixer] [--settings] [--melody] [--empty]` renders
  the window offscreen with wgpu. It is used to review layout without a display or screen-recording permission
  (`docs/screenshots/`).

Manual test procedure for what automation does not cover: see `docs/development/progress.md` › "Manual checks".

## Known UI gaps

No docking/detaching to OS windows, no tool palette (only Draw-tool semantics), no copy/paste, no clip menu on
right-click (right-click deletes, as in FL), and no accessibility labels on custom-painted editors.
