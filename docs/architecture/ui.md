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

Each frame, `logic()` installs the theme and fonts once, sets the UI zoom, collects engine garbage,
sends new effect processors and recompiles the snapshot if the project changed, polls export jobs,
updates the window title and handles close requests. Fonts registered by `logic` become usable one
frame later, so `ui()` draws a blank first frame. Then `ui()` lays out docked panels (no floating
editor windows; the arrangement follows the ArtCraft apps, the content follows FL Studio):

```
┌ header: logo · menus · project name ─────────────────────────────────────────────┐
├ toolbar: PAT/SONG · play · stop · rec(off) │ tempo · sig · position │ pattern │ snap · metronome · loop │ panels … undo · redo · save · export │ CPU · meter ┤
├ Browser ┬ Playlist header ─────────────────────────────────────────┬ Inspector ┤
│ channels│ Pattern picker │ track headers │ arrangement grid        │ channel / │
│ samples │                │               │                         │ mixer trk │
│         ├ dock tabs: CHANNEL RACK · PIANO ROLL · MIXER ────────────┤ + inserts │
│         │ the active editor                                        │           │
└ status bar: hover hint · voices · audio device ───────────────────────────────────┘
```

* `toolbar::header`, `toolbar::show`, `toolbar::status_bar`;
* `browser::show` (left, resizable): channels of the project, bundled and user sample folders;
* `playlist::panel` (centre): header strip, `picker_ui` (patterns), arrangement;
* `app::dock` (bottom, resizable): `widgets::tab_strip` + `channel_rack::panel` / `piano_roll::panel` /
  `mixer::panel`;
* `inspector::show` (right, resizable): the selected channel's instrument, or `mixer::inspector`
  for the selected mixer track (level, routing, `effects_ui::chain`);
* `dialogs` (save prompt, export, audio settings, rename, about, shortcuts) and toasts.

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

Global: Space play/stop, Cmd+Space play/pause, L Pattern/Song, Cmd+M metronome, Cmd+H panic, F4 new pattern, F5 maximise
the Playlist, F6/F7/F9 Channel Rack / Piano Roll / Mixer tab (again: hide the dock), Alt+F8 Browser, Cmd+I Inspector, F1 shortcut list,
Cmd+Z / Cmd+Shift+Z (or Cmd+Y) undo/redo, Cmd+N/O/S/Shift+S/R new/open/save/save as/export, Cmd+=/−/0 zoom,
+/− next/previous pattern.

Focused editor (the last one clicked): Delete/Backspace, Cmd+A, Cmd+D, Cmd+B duplicate, Cmd+C/X/V (notes; the selected
channel's steps in the rack; clips in the Playlist, pasted at the song position in Song mode), Alt+Q quantize (Piano Roll),
arrows (transpose/nudge in the Piano Roll, nudge/move in the Playlist, channel selection in the rack), Cmd+L route the
channel to a free insert.

Single-key shortcuts are ignored while a text field has focus.

## Theme

`theme.rs` defines one `Theme` token set (Lime Dark): near-black neutral surfaces (`bg`, `header`, `surface`, `raised`,
`sunken`), 1 px `border` rules between panels, three text levels, the lime `accent`, grid/meter colours and corner radii.
Fonts are Inter (text), Inter SemiBold (labels, via `FontFamily::Name("semibold")`) and JetBrains Mono (numbers), all
SIL OFL, embedded from `assets/fonts/`. Shared widgets (`widgets.rs`): flat `icon_button`, `transport_button`,
`segmented`, `knob`, `fader`, `meter`, `panel_header`, `tab_strip`, `section` labels, `readout`, `vrule`. Icons and
the logo are painter code (`icons.rs`). The style follows the ArtCraft apps (FilmCraft/SoundCraft: neutral dark panels,
docked layout, small upper-case headers, saturated clips with solid title strips); see
`docs/research/artcraft-architecture.md`.

## Testing the UI

* `crates/limeshot-app/tests/ui.rs` drives the real app with `egui_kittest`: synthetic keys and pointer events, no
  window, a silent audio clock. It covers dock shortcuts, new pattern + undo, transport, step clicks and drag-painting,
  playlist placement/deletion, the pattern picker, the Piano Roll add/drag/delete/undo sequence, copy/paste/quantize and
  rack step copying. Pointer positions come from geometry the editors record while drawing
  (`channel_rack::State::step_pos`, `piano_roll::Geometry::pos`, `playlist::Geometry::pos`), so layout changes do not
  break them.
* `cargo run -p limeshot-app --example ui_shot -- out.png [--piano] [--mixer] [--fx] [--settings] [--melody] [--empty]` renders
  the window offscreen with wgpu. It is used to review layout without a display or screen-recording permission
  (`docs/screenshots/`).

Manual test procedure for what automation does not cover: see `docs/development/progress.md` › "Manual checks".

## Known UI gaps

Panels cannot be torn off into OS windows or rearranged; no tool palette (Draw-tool semantics only); right-click deletes
in editors (as in FL), so menus live on the ▾ clip marker and channel buttons; custom-painted editors expose no
accessibility labels.
