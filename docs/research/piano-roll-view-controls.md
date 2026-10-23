# Piano Roll View and Target control research

2026-10-23. Clean-room behavioral research from Image-Line's public manuals; no installed application,
code, disassembly or bundled artwork was accessed. This is documentation evidence, not a live FL verification.

Sources read:

* [Piano roll Menu](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/pianoroll_menu.htm):
  View's keyboard labeling choices, persistent visible note labels, and Target control's lower-editor selection.
* [Piano roll](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/pianoroll.htm):
  note-property editing and the distinction between native-only properties and plugin support.

## Implemented subset

Target control chooses Note velocity or Note pan. Both edit the existing model values with commands.
The pan lane has a central zero line, right at the top and left at the bottom. A stem belonging to a selected
note edits the selected notes in the current channel; stale IDs from other channels are excluded. Each drag is
one undo step. Pan does not change the remembered velocity for newly drawn notes. Hosted note-pan playback
already has separate PLG-025 real-plugin acceptance coverage; this change adds its Piano Roll editing surface.

Keyboard labels offers None, All notes, Root notes and Highlighted notes, plus White notes only. The root and
highlighted choices follow the existing scale selection. Keep labels on screen anchors a partially scrolled note's
name at its visible left edge. All note text is clipped to its own visible body.

## LimeShot choices and limits

These are session-local view choices, without project edits or persisted preferences. Defaults preserve C-only
labels, velocity editing and fixed note-label origins. Without a selected scale, Root uses C and Highlighted
labels every pitch; the manual does not specify these fallback choices. Labels disappear below 11-pixel key height
to avoid overlap. Black keys retain LimeShot's shorter-key drawing; their labels fit inside that area.

The existing nearest-stem behavior remains: coincident notes need selection to edit together, and ties choose the
first note in pattern order. No separate note-properties dialog is introduced. Release velocity, Mod X/Y, fine pitch,
custom key names/presets, keyboard styles, event automation, panel swapping and separator flipping remain pending.
The broad PR-016 feature stays IN_PROGRESS. PR-030 tracks only the implemented label/view subset.

## Verification

* `piano_roll_target_control_edits_note_pan`: real menu selection and pointer editing, signed endpoints/centre,
  selected notes, channel isolation, preserved velocities, undo/redo, saved project round trip and return to velocity.
* `piano_roll_keyboard_label_view_options_do_not_edit_project`: actual nested menus and unchanged project/revision.
* `keyboard_labels_follow_scale_root_and_white_key_filter`: all MIDI keys with a black-key root and scale filtering.
* Offscreen review: `ui_shot --piano --melody --note-pan`, also with `--scale` and
  `--menu=⏵/View/Keyboard labels`. Live native/audio-device checks remain separate.
