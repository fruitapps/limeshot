# Keyboard shortcuts: FL Studio and LimeShot

> Generated from [`keybindings.json`](keybindings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313) (https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_shortcuts.htm).

FL keys use Windows notation (Ctrl = Cmd on macOS). confidence: VERIFIED = in the manual pages cited in docs/research/fl-studio-analysis.md; OBSERVED = seen in FL on this machine; INFERRED = from general FL knowledge, to be checked against the manual or the app.

**DIFFERENT** = LimeShot has the action on another key or with other behaviour; **LIMESHOT_ONLY** = no FL
equivalent. Every binding LimeShot has is checked against the Help › Keyboard shortcuts list.

Tracked: **76** — IMPLEMENTED 37, DIFFERENT 6, LIMESHOT_ONLY 2, NOT_STARTED 31

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
| KB-036 | Route selected channel to a free mixer track | `Ctrl+L` | VERIFIED | `Cmd+L` | IMPLEMENTED | SEQ-005 |  |
| KB-037 | Select previous / next channel | `Up / Down` | INFERRED | `↑ / ↓` | IMPLEMENTED | SEQ-007 |  |
| KB-038 | Move selected channels up / down | `Alt+Up / Alt+Down` | VERIFIED | — | NOT_STARTED | SEQ-007 | Available from the channel button menu. |
| KB-039 | Copy / cut / paste steps | `Ctrl+C / Ctrl+X / Ctrl+V` | INFERRED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | SEQ-015 |  |
| KB-040 | Shift steps left / right | `Ctrl+Shift+Left / Right` | INFERRED | — | NOT_STARTED | SEQ-014 |  |
| KB-041 | Rename channel | `F2` | INFERRED | — | NOT_STARTED | SEQ-007 | Rename is in the channel button menu. |
| KB-042 | Solo channel | `Ctrl+click mute` | VERIFIED | `Cmd+click / right-click mute` | IMPLEMENTED | SEQ-003 | A mouse binding, listed for completeness. |

## Piano Roll

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-043 | Draw tool | `P` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-044 | Paint tool | `B` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-045 | Delete tool | `D` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-046 | Mute tool | `T` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-047 | Slice tool | `C` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-048 | Select tool | `E` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-049 | Zoom tool | `Z` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-050 | Playback tool | `Y` | VERIFIED | — | NOT_STARTED | PR-012 |  |
| KB-051 | Select all | `Ctrl+A` | VERIFIED | `Cmd+A` | IMPLEMENTED | PR-002 |  |
| KB-052 | Deselect | `Ctrl+D` | INFERRED | `Cmd+D` | IMPLEMENTED | PR-002 |  |
| KB-053 | Delete selection | `Delete` | VERIFIED | `Delete / Backspace` | IMPLEMENTED | PR-001 |  |
| KB-054 | Duplicate selection | `Ctrl+B` | VERIFIED | `Cmd+B` | IMPLEMENTED | PR-008 |  |
| KB-055 | Copy / cut / paste | `Ctrl+C / Ctrl+X / Ctrl+V` | VERIFIED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | PR-019 |  |
| KB-056 | Transpose an octave | `Ctrl+Up / Ctrl+Down` | VERIFIED | `Cmd+↑ / Cmd+↓` | IMPLEMENTED | PR-009 |  |
| KB-057 | Transpose a semitone | `Shift+Up / Shift+Down` | VERIFIED | `↑ / ↓` | DIFFERENT | PR-009 | LimeShot uses the plain arrows. |
| KB-058 | Nudge the selection | `Shift+Left / Shift+Right` | VERIFIED | `← / →` | DIFFERENT | PR-009 | LimeShot nudges by the snap with the plain arrows. |
| KB-059 | Quick quantize | `Alt+Q` | VERIFIED | `Alt+Q` | IMPLEMENTED | PR-014 |  |
| KB-060 | Quantize dialog | `Ctrl+Q` | INFERRED | — | NOT_STARTED | PR-014 |  |
| KB-061 | Legato | `Ctrl+L` | VERIFIED | — | NOT_STARTED | PR-015 |  |
| KB-062 | Arpeggiate / strum / randomize | `Alt+A / Alt+S / Alt+R` | INFERRED | — | NOT_STARTED | PR-015 |  |
| KB-063 | Edit without snapping | `Alt+drag` | VERIFIED | `Alt+click` | IMPLEMENTED | PR-003 | A mouse modifier. |

## Playlist

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-064 | Draw / paint / delete / mute / slip / slice / select / zoom / playback tools | `P / B / D / T / S / C / E / Z / Y` | VERIFIED | — | NOT_STARTED | PL-017 |  |
| KB-065 | Select all | `Ctrl+A` | INFERRED | `Cmd+A` | IMPLEMENTED | PL-006 |  |
| KB-066 | Duplicate selection | `Ctrl+B` | VERIFIED | `Cmd+B` | IMPLEMENTED | PL-005 |  |
| KB-067 | Delete selection | `Delete` | INFERRED | `Delete / Backspace` | IMPLEMENTED | PL-004 |  |
| KB-068 | Copy / cut / paste clips | `Ctrl+C / Ctrl+X / Ctrl+V` | INFERRED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | PL-026 |  |
| KB-069 | Nudge clips | `Left / Right` | INFERRED | `← / →` | IMPLEMENTED | PL-002 |  |
| KB-070 | Add time marker | `Alt+T` | INFERRED | — | NOT_STARTED | PL-016 |  |
| KB-071 | Song position to the start | `Home` | INFERRED | `Home` | IMPLEMENTED | PL-009 |  |

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
