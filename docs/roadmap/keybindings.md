# Keyboard shortcuts: FL Studio and LimeShot

> Generated from [`keybindings.json`](keybindings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313) (https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_shortcuts.htm).

FL keys use Windows notation (Ctrl = Cmd on macOS). confidence: VERIFIED = in the manual pages cited in docs/research/fl-studio-analysis.md; OBSERVED = seen in FL on this machine; INFERRED = from general FL knowledge, to be checked against the manual or the app.

**DIFFERENT** = LimeShot has the action on another key or with other behaviour; **LIMESHOT_ONLY** = no FL
equivalent. Every binding LimeShot has is checked against the Help › Keyboard shortcuts list.

Tracked: **94** — IMPLEMENTED 65, DIFFERENT 5, LIMESHOT_ONLY 2, NOT_STARTED 22

## Global

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-001 | Play / stop | `Space` | VERIFIED | `Space` | IMPLEMENTED | TR-002 |  |
| KB-002 | Play / pause | `Ctrl+Space` | VERIFIED | `Cmd+Space` | IMPLEMENTED | TR-002 |  |
| KB-003 | Pattern / Song mode | `L` | VERIFIED | `L` | IMPLEMENTED | TR-003 |  |
| KB-004 | Record | `R` | VERIFIED | — | NOT_STARTED | REC-001 | The Record button is shown disabled. |
| KB-005 | Metronome | `Ctrl+M` | VERIFIED | `Cmd+M` | IMPLEMENTED | TR-008 |  |
| KB-006 | Panic (stop all sound) | `Ctrl+H` | VERIFIED | `Cmd+H` | IMPLEMENTED | TR-010 |  |
| KB-007 | Step edit mode | `Ctrl+E` | INFERRED | — | NOT_STARTED | REC-009 |  |
| KB-008 | Typing keyboard to piano | `Ctrl+T` | INFERRED | — | NOT_STARTED | WF-002 |  |
| KB-009 | Playlist | `F5` | VERIFIED | `F5` | IMPLEMENTED | UI-002 |  |
| KB-010 | Channel Rack | `F6` | VERIFIED | `F6` | IMPLEMENTED | UI-002 |  |
| KB-011 | Piano Roll | `F7` | VERIFIED | `F7` | IMPLEMENTED | UI-002 |  |
| KB-012 | Plugin Picker | `F8` | VERIFIED | — | NOT_STARTED | PLG-004 |  |
| KB-013 | Mixer | `F9` | VERIFIED | `F9` | IMPLEMENTED | UI-002 |  |
| KB-014 | Browser | `Alt+F8` | VERIFIED | `Alt+F8` | IMPLEMENTED | UI-001 |  |
| KB-015 | MIDI settings | `F10` | INFERRED | — | NOT_STARTED | APP-020 |  |
| KB-016 | Project info | `F11` | INFERRED | — | NOT_STARTED | APP-017 |  |
| KB-017 | Close all windows | `F12` | VERIFIED | — | NOT_STARTED | UI-002 |  |
| KB-018 | Close the focused window | `Esc` | VERIFIED | — | NOT_STARTED | UI-002 | Esc closes menus and dialogs only. |
| KB-019 | Maximise the Playlist | `Enter` | VERIFIED | — | NOT_STARTED | UI-002 |  |
| KB-020 | Arrange windows | `Ctrl+Shift+H` | INFERRED | `Cmd+Shift+H` | IMPLEMENTED | UI-002 |  |
| KB-021 | Help | `F1` | INFERRED | `F1` | DIFFERENT | UI-004 | LimeShot's F1 lists the keyboard shortcuts (no manual). |
| KB-022 | Next / previous pattern | `Numpad + / −` | INFERRED | `+ / −` | IMPLEMENTED | SEQ-018 |  |
| KB-023 | Next empty pattern | `F4` | INFERRED | `F4` | IMPLEMENTED | SEQ-018 | LimeShot adds a new pattern. |
| KB-024 | Undo | `Ctrl+Z` | VERIFIED | `Cmd+Z` | DIFFERENT | APP-005 | FL's Ctrl+Z toggles the last undo/redo and Ctrl+Alt+Z steps back; LimeShot's Cmd+Z steps back through the history. |
| KB-025 | Redo | `Ctrl+Shift+Z` | INFERRED | `Cmd+Shift+Z` | IMPLEMENTED | APP-005 | Cmd+Y also redoes. |
| KB-026 | UI zoom | `—` | VERIFIED | `Cmd+= / Cmd+- / Cmd+0` | LIMESHOT_ONLY | UI-005 | FL scales through Options. |

## File

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-027 | Open | `Ctrl+O` | VERIFIED | `Cmd+O` | IMPLEMENTED | APP-002 |  |
| KB-028 | Save | `Ctrl+S` | VERIFIED | `Cmd+S` | IMPLEMENTED | APP-003 |  |
| KB-029 | Save as | `Ctrl+Shift+S` | VERIFIED | `Cmd+Shift+S` | IMPLEMENTED | APP-003 |  |
| KB-030 | Save new version | `Ctrl+N` | VERIFIED | — | NOT_STARTED | APP-018 | LimeShot uses Cmd+N for New (platform convention); a new-version shortcut is still to be chosen. |
| KB-031 | New project | `—` | INFERRED | `Cmd+N` | LIMESHOT_ONLY | APP-001 |  |
| KB-032 | Export WAV | `Ctrl+R` | VERIFIED | `Cmd+R` | IMPLEMENTED | EXP-001 |  |
| KB-033 | Export MP3 | `Ctrl+Shift+R` | VERIFIED | — | NOT_STARTED | EXP-005 |  |
| KB-034 | Open recent project 1–10 | `Alt+1…0` | VERIFIED | — | NOT_STARTED | APP-010 | Recent files are in File › Open recent. |
| KB-035 | Quit | `Alt+F4 / Cmd+Q` | INFERRED | `Cmd+Q` | IMPLEMENTED | APP-014 |  |

## Channel Rack

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-036 | Route selected channel to a free mixer track | `Ctrl+L` | VERIFIED | `Cmd+L` | IMPLEMENTED | SEQ-005 | Routes every selected channel to its own free track. |
| KB-037 | Select previous / next channel | `Up / Down` | OBSERVED | `↑ / ↓` | IMPLEMENTED | SEQ-007 | Observed in FL 2026.1.1: selects the channel after the last (before the first) selected one, on its own. |
| KB-038 | Move selected channels up / down | `Alt+Up / Alt+Down` | VERIFIED | `Alt+↑ / Alt+↓` | IMPLEMENTED | SEQ-007 | Observed in FL 2026.1.1: each selected channel moves one row; the mixer track stays with the channel. |
| KB-039 | Copy / cut / paste steps | `Ctrl+C / Ctrl+X / Ctrl+V` | INFERRED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | SEQ-015 |  |
| KB-040 | Rotate steps left / right | `Shift+Cmd+Left / Shift+Cmd+Right` | OBSERVED | `Shift+Cmd+← / Shift+Cmd+→` | IMPLEMENTED | SEQ-014 | FL 2026.1.1 channel menu labels; tried live: one step per press, wrapping round the pattern (session 8). |
| KB-041 | Rename current pattern | `F2` | OBSERVED | `F2` | IMPLEMENTED | SEQ-018 | With the Channel Rack focused FL 2026.1.1 opens the pattern name dialog, not a channel rename (channels are renamed from the channel button menu). |
| KB-042 | Solo channel | `Ctrl+click mute` | VERIFIED | `Cmd+click / right-click mute` | IMPLEMENTED | SEQ-003 | A mouse binding, listed for completeness. |
| KB-077 | Delete selected channels (asks first) | `Alt+Del` | OBSERVED | `Alt+Delete` | IMPLEMENTED | SEQ-007 | Forward delete; Alt+Backspace does nothing in FL 2026.1.1. |
| KB-078 | Add / remove a channel from the selection | `Ctrl+click / Shift+click / right-click selector LED` | OBSERVED | `Cmd+click / Shift+click / right-click selector LED` | IMPLEMENTED | SEQ-020 | A mouse binding, listed for completeness. A plain click selects only that channel. |
| KB-093 | Group selected channels (display filter group) | `Opt+G` | OBSERVED | `Alt+G` | IMPLEMENTED | SEQ-012 | FL 2026.1.1 Channel Rack menu: Group selected… Opt+G asks for a filter group name (session 8). |

## Piano Roll

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-043 | Draw tool | `P` | VERIFIED | `P` | IMPLEMENTED | PR-012 |  |
| KB-044 | Paint tool | `B` | VERIFIED | `B` | IMPLEMENTED | PR-012 |  |
| KB-045 | Delete tool | `D` | VERIFIED | `D` | IMPLEMENTED | PR-012 |  |
| KB-046 | Mute tool | `T` | VERIFIED | `T` | IMPLEMENTED | PR-012 |  |
| KB-047 | Slice tool | `C` | VERIFIED | `C` | IMPLEMENTED | PR-012 | Observed: the hint bar labels these keys (Draw P, Slice C, Select E, Zoom Z, Playback Y; Paint B). |
| KB-048 | Select tool | `E` | VERIFIED | `E` | IMPLEMENTED | PR-012 |  |
| KB-049 | Zoom tool | `Z` | VERIFIED | `Z` | IMPLEMENTED | PR-012 |  |
| KB-050 | Playback tool | `Y` | VERIFIED | `Y` | IMPLEMENTED | PR-012 |  |
| KB-051 | Select all | `Ctrl+A` | VERIFIED | `Cmd+A` | IMPLEMENTED | PR-002 |  |
| KB-052 | Deselect | `Ctrl+D` | OBSERVED | `Cmd+D` | IMPLEMENTED | PR-002 | Listed in FL 2026.1.1's Piano Roll Select menu. |
| KB-053 | Delete selection | `Delete` | VERIFIED | `Delete / Backspace` | IMPLEMENTED | PR-001 |  |
| KB-054 | Duplicate selection | `Ctrl+B` | VERIFIED | `Cmd+B` | IMPLEMENTED | PR-008 |  |
| KB-055 | Copy / cut / paste | `Ctrl+C / Ctrl+X / Ctrl+V` | VERIFIED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | PR-019 |  |
| KB-056 | Transpose an octave | `Ctrl+Up / Ctrl+Down` | VERIFIED | `Cmd+↑ / Cmd+↓` | IMPLEMENTED | PR-009 |  |
| KB-057 | Transpose a semitone | `Shift+Up / Shift+Down` | OBSERVED | `Shift+↑ / Shift+↓` | IMPLEMENTED | PR-009 | Applies to every note of the channel when none is selected (observed). Plain arrows scroll. |
| KB-058 | Nudge the selection | `Shift+Left / Shift+Right` | OBSERVED | `Shift+← / Shift+→` | IMPLEMENTED | PR-009 | FL calls it Shift left/right (moves by the snap). |
| KB-059 | Quick quantize | `Cmd+Q (macOS menu)` | OBSERVED | `Alt+Q` | DIFFERENT | PR-014 | FL 2026.1.1's macOS Tools menu: Quick quantize Cmd+Q, Quick quantize start times Shift+Q, Quantize… Opt+Q. LimeShot keeps Cmd+Q for Quit and uses Alt+Q for quick quantize. |
| KB-060 | Quantize dialog | `Opt+Q (macOS menu)` | OBSERVED | — | NOT_STARTED | PR-014 | FL 2026.1.1 Tools menu. |
| KB-061 | Legato | `Cmd+L (macOS menu)` | OBSERVED | — | NOT_STARTED | PR-015 | Quick legato in FL 2026.1.1's Tools menu. |
| KB-062 | Arpeggiate / strum / randomize | `Alt+A / Alt+S / Alt+R` | INFERRED | — | NOT_STARTED | PR-015 |  |
| KB-063 | Edit without snapping | `Alt+drag` | VERIFIED | `Alt+click` | IMPLEMENTED | PR-003 | A mouse modifier. |
| KB-079 | Invert selection | `Shift+I` | OBSERVED | `Shift+I` | IMPLEMENTED | PR-002 | From FL 2026.1.1's Select menu. |
| KB-080 | Scroll the view | `Up / Down / Left / Right` | OBSERVED | `↑ / ↓ / ← / →` | IMPLEMENTED | PR-005 | Up/Down move about three rows per press in FL 2026.1.1. |
| KB-081 | Rotate selection left / right | `Shift+Cmd+Left / Shift+Cmd+Right` | OBSERVED | `Shift+Cmd+← / Shift+Cmd+→` | IMPLEMENTED | PR-009 | FL 2026.1.1 Edit menu (macOS labels). LimeShot rotates by the snap; all of the channel's notes when none is selected. |
| KB-082 | Discard lengths | `Shift+D` | OBSERVED | — | NOT_STARTED | PR-015 | FL 2026.1.1 Edit menu. |
| KB-083 | Mute / unmute selected notes | `Opt+M / Shift+Opt+M` | OBSERVED | `Alt+M / Shift+Alt+M` | IMPLEMENTED | PR-012 | Selection, or all of the channel's notes when none is selected. |
| KB-088 | Glue notes | `Cmd+G` | OBSERVED | `Cmd+G` | IMPLEMENTED | PR-023 | FL 2026.1.1 Tools menu. |
| KB-089 | Quick chop | `Cmd+U` | OBSERVED | `Cmd+U` | IMPLEMENTED | PR-023 | Cuts at every snap step. FL's Chop… dialog (Opt+U) is not implemented. |

## Playlist

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-064 | Draw / paint / delete / mute / slip / slice / select / zoom / playback tools | `P / B / D / T / S / C / E / Z / Y` | VERIFIED | `P / B / D / T / S / C / E / Z / Y` | IMPLEMENTED | PL-017 | Session 8: the tool strip and keys observed live in FL Studio 2026.1.1 (fl-playlist-tools.png); test playlist_tool_keys_and_strip. |
| KB-065 | Select all | `Ctrl+A` | OBSERVED | `Cmd+A` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-066 | Duplicate selection | `Ctrl+B` | VERIFIED | `Cmd+B` | IMPLEMENTED | PL-005 |  |
| KB-067 | Delete selection | `Delete` | OBSERVED | `Delete / Backspace` | IMPLEMENTED | PL-004 | FL 2026.1.1 Playlist Edit menu (Del). |
| KB-068 | Copy / cut / paste clips | `Ctrl+C / Ctrl+X / Ctrl+V` | OBSERVED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | PL-026 | FL 2026.1.1 Playlist Edit menu. |
| KB-069 | Shift clips left / right / up / down | `Shift+Left / Shift+Right / Shift+Up / Shift+Down` | OBSERVED | `Shift+← / Shift+→ / Shift+↑ / Shift+↓` | IMPLEMENTED | PL-002 | FL 2026.1.1 Playlist Edit menu (fl-playlist-edit-menu.png). Up/Down move a track. Plain Up/Down scroll a track (observed live). |
| KB-070 | Add a time marker (asks its name) | `Opt+T` | OBSERVED | `Alt+T` | IMPLEMENTED | PL-016 | FL 2026.1.1 Time markers menu: Add one… |
| KB-071 | Song position to the start | `Home` | INFERRED | `Home` | IMPLEMENTED | PL-009 |  |
| KB-084 | Invert selection | `Shift+I` | OBSERVED | `Shift+I` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-085 | Deselect | `Cmd+D` | OBSERVED | `Cmd+D` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-086 | Mute / unmute selected clips | `Opt+M / Shift+Opt+M` | OBSERVED | — | NOT_STARTED | PL-005 | FL 2026.1.1 Playlist Edit menu. |
| KB-087 | Merge selected clips | `Cmd+G` | OBSERVED | — | NOT_STARTED | PL-022 | FL 2026.1.1 Playlist Edit menu; also Merge similar pattern clips Shift+Cmd+G. |
| KB-090 | Add a time marker at the song position | `Cmd+T` | OBSERVED | `Cmd+T` | IMPLEMENTED | PL-016 | FL 2026.1.1 Time markers menu: Add auto. |
| KB-091 | Time selection | `Ctrl+drag on the ruler` | OBSERVED | `Cmd+drag on the ruler` | IMPLEMENTED | PL-016 | A mouse binding. Song playback loops the selection. |
| KB-092 | Place loop markers around the selection | `Shift+T` | OBSERVED | — | NOT_STARTED | PL-016 | FL 2026.1.1 Time markers menu. |

## Mixer

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-072 | Link selected channels to the selected track | `Ctrl+L` | INFERRED | — | NOT_STARTED | SEQ-005 | LimeShot's Cmd+L works from the Channel Rack. |
| KB-073 | Move track left / right | `Alt+Left / Alt+Right` | INFERRED | — | NOT_STARTED | MIX-006 |  |
| KB-074 | Rename track | `F2` | INFERRED | — | NOT_STARTED | MIX-006 | Double-click the name to rename. |

## Controls

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-075 | Fine adjustment while dragging | `Ctrl / Alt+drag` | VERIFIED | `Shift+drag` | DIFFERENT | UI-004 |  |
| KB-076 | Reset to default | `Right-click › Reset` | OBSERVED | `Double-click` | DIFFERENT | UI-010 | Observed in FL's control menu (session 4). |

## Piano Roll / Playlist

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-094 | Zoom vertically | `Alt+Mouse wheel` | INFERRED | `Opt+wheel / Opt+pinch` | IMPLEMENTED | PR-005 | Session 9 (owner's hands-on test: no vertical zoom). Not checked live in FL this session; test alt_wheel_zooms_vertically. |
