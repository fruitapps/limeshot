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
frame later, so `ui()` draws a blank first frame.

**Window model: FL Studio's, with a modern finish.** FL Studio is the source for structure and
interaction; the ArtCraft apps only for the broad visual style (decision D24).

```
┌ header: logo · menus · project name ─────────────────────────────────────────────┐
├ toolbar: PAT/SONG · play · stop · rec(off) │ tempo · sig · position │ pattern │ snap · metronome · loop │ Browser · Playlist · Rack · Piano · Mixer … undo · redo · save · export │ CPU · meter ┤
├ Browser ┬ workspace ──────────────────────────────────────────────────────────────┤
│ channels│  Playlist (maximised by default, the backdrop)                          │
│ samples │    ┌ Channel Rack ─────┐   ┌ Channel Settings ┐                         │
│         │    └───────────────────┘   └──────────────────┘   ┌ Mixer ┐  ┌ Piano Roll ┐
└ status bar: hover hint · voices · audio device ───────────────────────────────────┘
```

* `workspace.rs` manages the editor windows (`Win::{Playlist, ChannelRack, PianoRoll, Mixer, ChannelSettings}`):
  open/close (F5/F6/F7/F9 toggle, as in FL), move, resize, **maximise** to the workspace, **detach** into an OS window
  (`Context::show_viewport_immediate`; embedded when the backend has no multi-window support), re-attach, and
  **Arrange windows** (Cmd+Shift+H) to restore the defaults. Title bars are drawn by LimeShot (flat, upper-case title,
  detach/maximise/close buttons); dragging one moves the window. A maximised Playlist stays behind the floating windows,
  as in FL; other maximised windows keep normal stacking. Shortcuts also work inside detached windows.
* New projects open with FL Studio's layout: Playlist maximised, Channel Rack floating on top.
* Clicking a channel button toggles the **Channel Settings** window for that channel (FL behaviour); its content is
  `inspector::show` (sampler/synth parameters).
* The **Mixer** window shows strips on the left and, like FL, the selected track's panel on the right (level, routing,
  the 10 insert slots via `effects_ui::chain`).
* `toolbar::header`, `toolbar::show`, `toolbar::status_bar`; `browser::show` (docked left, resizable, as FL's Browser);
  `dialogs` and toasts.

Editors are custom-painted (`ui.painter_at`) with a single `interact` region per area, so they stay fast with many notes.
Hit testing uses each editor's `View` (tick ↔ x, key/track ↔ y).

### Plugins

`plugins.rs` holds the plugin database (`plugins.json`, filled by Find plugins…, which runs `limeshot --scan-vst3`
once per bundle in a child process) and `PluginHost`, which keeps one `Vst3Instance` per plugin channel and plugin
insert of the open project. `LimeShotApp::sync_engine` calls `PluginHost::sync` before sending a snapshot, so new
plugins are created on the main thread and their audio halves reach the audio thread first; removed ones are
dropped. Saving and autosaving read plugin states into the document first. Exports get fresh instances whose
processors send themselves back to the main thread when dropped.

## Interaction conventions

Chosen after the FL Studio manual (see `docs/research/workflow-comparisons.md`):

| Where | Action |
|---|---|
| Step buttons | left- or right-click toggles; a drag paints the state of the first step (FL Studio 2026); turning a step on auditions it |
| Piano Roll | left on empty adds a note (snapped) and keeps dragging it; drag a note moves; drag right edge resizes; right-click/drag deletes with a red cursor; right-click on empty deselects; Cmd+click toggles selection; Cmd+drag rectangle; double-click a note selects its pitch row; Alt bypasses snap |
| Piano keys | click/drag auditions |
| Velocity lane | click/drag sets the nearest stem (and the whole selection if that note is selected) |
| Playlist | left on empty places the current pattern and keeps dragging it; drag moves (time and track); right edge resizes; right-click/drag deletes; ▾ on a clip opens its menu (Make unique, Edit, Rename, Mute, Fit, Duplicate, Delete); double-click opens the pattern in the Piano Roll; ruler click/drag sets the song position |
| Knobs and faders | drag vertically, Shift for fine control, wheel steps, double-click resets; a small marker lights at the default |
| Mute LEDs/buttons | left-click mutes; right-click or Cmd+click solos; Shift+click locks Channel Rack and Playlist mute state |
| Scrolling | wheel vertical, Shift+wheel horizontal, Cmd+wheel zooms time around the pointer |

Right-click channel/mixer volume or pan and built-in effect parameters for Reset, Copy/Paste value,
Type in value… and Create automation clip. The value clipboard holds normalised control position independently
of notes/clips. Type-in uses the control's units and commits once on OK; invalid input and Cancel do not edit.

Continuous gestures call `Document::begin_gesture` on drag start and `end_gesture` on release, so a drag is one undo step.

Options › Auto scrolling and the toolbar arrow share `settings.auto_scroll` (off by default). When enabled,
Playlist follows Song playback and Piano Roll follows Pattern playback. They page when the playhead leaves
view, including returning to a loop start; stopped transport and active editor drags leave the view alone.

## Keyboard

Global: Space play/stop, Cmd+Space play/pause, L Pattern/Song, Cmd+M metronome, Cmd+H panic, F4 new pattern, F5/F6/F7/F9
toggle the Playlist / Channel Rack / Piano Roll / Mixer windows, Alt+F8 Browser, Cmd+Shift+H arrange windows, F1 shortcut list,
Cmd+Z / Cmd+Shift+Z (or Cmd+Y) undo/redo, Cmd+N/O/S/Shift+S/R new/open/save/save as/export, Cmd+=/−/0 zoom,
+/− next/previous pattern.

Focused editor (the last one clicked): Delete/Backspace, Cmd+A, Cmd+D, Cmd+B duplicate, Cmd+C/X/V (notes; the selected
channel's steps in the rack; clips in the Playlist, pasted at the song position in Song mode), Alt+Q quantize (Piano Roll),
arrows (scroll in the Piano Roll, where Shift+arrows transpose/nudge and Cmd+Up/Down transpose an octave —
the selection or, with none, every note of the channel; nudge/move in the Playlist; channel selection in the rack),
Shift+I invert the Piano Roll selection. Channel Rack:
Alt+Up/Down move, Alt+Delete delete (with FL's confirmation) and Cmd+L route the selected channels; F2 renames the
current pattern.

The Channel Rack's selector LEDs are FL Studio's channel selection (`UiState::channel_selection`): a click selects one
channel, Cmd/Shift/right-click toggles one; dragging the selector LEDs selects crossed rows without moving the window. `selected_channel` stays the current channel that the Piano Roll and
Channel Settings edit; it is the last channel selected.

Channel Rack › Zip selected (Alt+Z) replaces selected rows with compact named buttons. Right-click one to restore
its controls, or use Unzip all (Alt+U), which includes filtered-out channels. Compact buttons retain click and
Cmd/Shift-click selection. Zip state belongs to the session, follows channel IDs across patterns/reordering, and
does not edit the project or its undo history. Deleted channels are pruned; new/open projects start expanded.
General settings › Miscellaneous › Auto zip empty channels saves `settings.auto_zip_empty_channels` (off by default).
On a pattern switch it replaces the compact set for all channels, including filtered-out ones, using the destination
pattern's note data. Muted notes count as data. Enabling waits for the next switch, and note edits do not recompute
the view, so manual Unzip/right-click restoration remains usable. Deletion/undo fallback pattern changes are caught
when the rack draws. Any held rack step/graph gesture ends before switching its editing surface.

The toolbar typing-keyboard toggle (Cmd+T outside Playlist) maps Z–M and Q–U to notes for the selected channel.
Key release targets the original channel even after selection changes; dialogs, text fields and focus loss release
held notes. Playlist Cmd+T retains its marker action. Single-key shortcuts are ignored while a text field has focus.

Browser sample and channel drags, sampler waveform drags, and Playlist picker sources share drop payloads. Audio
drops show waveform ghosts on the snapped destination and beneath the pointer. Sampler previews finish independently
of mouse release; their playback head estimates the latest voice from preview/transport time. The rack displays
4–8 beats of horizontal content with scrolling, including long MIDI previews. Automation points can be placed
throughout a clip body; headers and trim edges keep their clip gestures. Repeated Stop rewinds to song/selection start.

An open background workspace window is raised before toolbar/F-key toggles close it; pinned windows retain priority.
Playlist track headers have supported rename/color, mute/solo/lock, selection, merge, clone and ordering actions;
unsupported advanced FL actions are disabled. + Track sits beneath the last track.

## Theme

`theme.rs` defines one `Theme` token set (Lime Dark): near-black neutral surfaces (`bg`, `header`, `surface`, `raised`,
`sunken`), 1 px `border` rules between panels, three text levels, the lime `accent`, grid/meter colours and corner radii.
Fonts are Inter (text), Inter SemiBold (labels, via `FontFamily::Name("semibold")`) and JetBrains Mono (numbers), all
SIL OFL, embedded from `assets/fonts/`. Shared widgets (`widgets.rs`): flat `icon_button`, `transport_button`,
`segmented`, `knob`, `fader`, `meter`, `panel_header`, `tab_strip`, `section` labels, `readout`, `vrule`. Icons and
the logo are painter code (`icons.rs`). The style follows the ArtCraft apps (FilmCraft/SoundCraft: neutral dark panels,
docked layout, small upper-case headers, saturated clips with solid title strips); see
`docs/research/artcraft-architecture.md`. Layout and behaviour follow FL Studio.

## Testing the UI

* `crates/limeshot-app/tests/ui.rs` drives the real app with `egui_kittest`: synthetic keys and pointer events, no
  window, a silent audio clock. It covers dock shortcuts, new pattern + undo, transport, step clicks and drag-painting,
  playlist placement/deletion, the pattern picker, the Piano Roll add/drag/delete/undo sequence, copy/paste/quantize and
  rack step copying, the window model (F-keys, FL's default layout, channel button → Channel Settings, maximise/detach/close)
  and that opening windows never edits the project. Pointer positions come from geometry the editors record while drawing
  (`channel_rack::State::step_pos`, `piano_roll::Geometry::pos`, `playlist::Geometry::pos`), so layout changes do not
  break them.
* `cargo run -p limeshot-app --example ui_shot -- out.png [--piano] [--mixer] [--fx] [--settings] [--melody] [--empty] [--plugin --plugin-window] [--dialog=export|quit|marker] [--tapper]` renders
  the window offscreen with wgpu. It is used to review layout without a display or screen-recording permission
  (`docs/screenshots/`).

Manual test procedure for what automation does not cover: see `docs/development/progress.md` › "Manual checks".

## Known UI gaps

**Plugin windows** (`plugin_window.rs`, D35) are resizable egui windows inside the workspace with Plugin /
Settings tabs. On macOS, the plugin's native view is a child of the main NSWindow's content view;
resize-capable plugins receive size negotiation, while fixed-size editors retain a minimum window size.
Native editors hide when another window or popup covers them; final live rendering, resize, tab and keyboard
verification is pending (PLG-006 is IMPLEMENTED). Window state lives beside the running instance
and is saved in the project. Windows and Linux show the parameter list until their native embedding exists.

Editor layouts can be explicitly saved/restored through View › Arrange windows; they are not automatically persisted. The Piano Roll has FL's tool strip
(`piano_roll::Tool`, keys P B D T C E Z Y); the Playlist has nine tools; right-click deletes in editors (as in FL), so menus live on the ▾ clip marker and channel buttons;
custom-painted editors expose no accessibility labels.
