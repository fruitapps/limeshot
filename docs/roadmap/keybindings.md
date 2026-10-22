# Keyboard shortcuts: FL Studio and LimeShot

> Generated from [`keybindings.json`](keybindings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313) (https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_shortcuts.htm).

FL keys use Windows notation (Ctrl = Cmd on macOS). confidence: VERIFIED = in the manual pages cited in docs/research/fl-studio-analysis.md; OBSERVED = seen in FL on this machine; INFERRED = from general FL knowledge, to be checked against the manual or the app.

**DIFFERENT** = LimeShot has the action on another key or with other behaviour; **LIMESHOT_ONLY** = no FL
equivalent. Every binding LimeShot has is checked against the Help › Keyboard shortcuts list.

Tracked: **142** — IMPLEMENTED 108, DIFFERENT 7, LIMESHOT_ONLY 2, NOT_STARTED 22, NOT_PLANNED 3

## Global

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-001 | Play / stop | `Space` | VERIFIED | `Space` | IMPLEMENTED | TR-002 |  |
| KB-002 | Play / pause | `Ctrl+Space` | VERIFIED | `Cmd+Space` | IMPLEMENTED | TR-002 |  |
| KB-003 | Pattern / Song mode | `L` | VERIFIED | `L` | IMPLEMENTED | TR-003 |  |
| KB-004 | Record | `R` | VERIFIED | — | NOT_STARTED | REC-001 | The Record button is shown disabled. |
| KB-005 | Metronome | `Ctrl+M` | VERIFIED | `Cmd+M` | IMPLEMENTED | TR-008 |  |
| KB-006 | Panic (stop all sound) | `Ctrl+H` | VERIFIED | `Cmd+H` | IMPLEMENTED | TR-010 |  |
| KB-007 | Step edit mode | `Ctrl+E` | OBSERVED | — | NOT_STARTED | REC-009 | Seen in the main menu on macOS (session 13, menus.json). |
| KB-008 | Typing keyboard to piano | `Ctrl+T` | INFERRED | `Cmd+T` | IMPLEMENTED | WF-002 | Outside Playlist; Playlist Cmd+T still adds an automatic time marker. Toolbar button works in every editor. |
| KB-009 | Playlist | `F5` | VERIFIED | `F5` | IMPLEMENTED | UI-002 |  |
| KB-010 | Channel Rack | `F6` | VERIFIED | `F6` | IMPLEMENTED | UI-002 |  |
| KB-011 | Piano Roll | `F7` | VERIFIED | `F7` | IMPLEMENTED | UI-002 |  |
| KB-012 | Plugin Picker | `F8` | VERIFIED | — | NOT_STARTED | PLG-004 |  |
| KB-013 | Mixer | `F9` | VERIFIED | `F9` | IMPLEMENTED | UI-002 |  |
| KB-014 | Browser | `Alt+F8` | VERIFIED | `Alt+F8` | IMPLEMENTED | UI-001 |  |
| KB-015 | MIDI settings | `F10` | OBSERVED | — | NOT_STARTED | APP-020 | Seen in the main menu on macOS (session 13, menus.json). |
| KB-016 | Project info | `F11` | OBSERVED | `F11` | IMPLEMENTED | APP-017 | Seen in the main menu on macOS (session 13, menus.json). |
| KB-017 | Close all windows | `F12` | VERIFIED | `F12` | IMPLEMENTED | UI-013 | View › Close all windows (MENU-123): the editors and the plugin windows. |
| KB-018 | Close the focused window | `Esc` | VERIFIED | — | NOT_STARTED | UI-002 | Esc closes menus and dialogs only. |
| KB-019 | Maximise the Playlist | `Enter` | VERIFIED | — | NOT_STARTED | UI-002 |  |
| KB-020 | Arrange windows | `Ctrl+Shift+H` | OBSERVED | `Cmd+Shift+H` | IMPLEMENTED | UI-002 | Seen in the main menu on macOS (session 13, menus.json). |
| KB-021 | Help | `F1` | INFERRED | `F1` | DIFFERENT | UI-004 | LimeShot's F1 lists the keyboard shortcuts (no manual). |
| KB-022 | Next / previous pattern | `Numpad + / −` | INFERRED | `+ / −` | IMPLEMENTED | SEQ-018 |  |
| KB-023 | Find next empty pattern (asks for a name) | `F4` | OBSERVED | `F4` | IMPLEMENTED | SEQ-021 | Patterns › Find next empty… (MENU-080). LimeShot: the first pattern without notes after the current one, else a new one at the end; then the rename dialog. Session 15 (until then F4 added a pattern without asking). |
| KB-024 | Undo | `Ctrl+Z` | OBSERVED | `Cmd+Z` | IMPLEMENTED | APP-005 | FL 2026 (observed session 15): Cmd+Z steps back through the history, as LimeShot's does. |
| KB-025 | Redo | `Cmd+Opt+Z` | OBSERVED | `Cmd+Alt+Z / Cmd+Shift+Z` | IMPLEMENTED | APP-005 | FL 2026's Edit menu shows Redo on Cmd+Opt+Z (observed session 15). LimeShot also redoes with Cmd+Shift+Z and Cmd+Y. |
| KB-026 | UI zoom | `—` | VERIFIED | `Cmd+= / Cmd+- / Cmd+0` | LIMESHOT_ONLY | UI-005 | FL scales through Options. |
| KB-097 | Plugin database in the Browser | `Shift+F8` | OBSERVED | — | NOT_STARTED | BR-008 | View › Plugin database (MENU-145); also Add › Effect › Plugin database. |
| KB-098 | Find first empty pattern (asks for a name) | `Shift+F4` | OBSERVED | `Shift+F4` | IMPLEMENTED | SEQ-021 | Patterns › Find first empty... (MENU-079). |
| KB-099 | Find next empty pattern without naming | `Ctrl+F4` | OBSERVED | `Cmd+F4` | IMPLEMENTED | SEQ-021 | Patterns › Find next empty (no naming) (MENU-081). |
| KB-100 | Open the current pattern in the project browser | `Ctrl+Shift+P` | OBSERVED | — | NOT_STARTED | BR-005 | Patterns › Open in project browser (MENU-086). |
| KB-101 | Insert a pattern | `Ctrl+Shift+Ins` | OBSERVED | `Cmd+Shift+Insert` | IMPLEMENTED | SEQ-021 | Patterns › Insert one (MENU-092). Mac keyboards have no Insert key; the menu item works everywhere. |
| KB-102 | Clone pattern | `Ctrl+Shift+C` | OBSERVED | `Cmd+Shift+C` | IMPLEMENTED | SEQ-021 | Patterns › Clone (MENU-093). |
| KB-103 | Delete pattern | `Ctrl+Shift+Del` | OBSERVED | `Cmd+Shift+Delete` | IMPLEMENTED | SEQ-021 | Patterns › Delete (MENU-094). LimeShot also takes Cmd+Shift+Backspace (the Mac's Delete key). |
| KB-104 | Move pattern up | `Ctrl+Shift+Up` | OBSERVED | `Cmd+Shift+↑` | IMPLEMENTED | SEQ-021 | Patterns › Move up (MENU-095). |
| KB-105 | Move pattern down | `Ctrl+Shift+Down` | OBSERVED | `Cmd+Shift+↓` | IMPLEMENTED | SEQ-021 | Patterns › Move down (MENU-096). |
| KB-106 | Project picker | `Ctrl+F8` | OBSERVED | — | NOT_STARTED | BR-005 | View › Project picker (MENU-106). |
| KB-107 | Tempo tapper | `Ctrl+F5` | OBSERVED | `Cmd+F5` | IMPLEMENTED | TR-012 | View › Tempo tapper (MENU-108). |
| KB-108 | Touch controller | `Alt+F7` | OBSERVED | — | NOT_STARTED | WF-011 | View › Touch controller (MENU-109). |
| KB-109 | Hide toolbars | `Ctrl+F11` | OBSERVED | — | NOT_STARTED | UI-014 | View › Toolbars › Hide (MENU-118). |
| KB-110 | Close all plugin windows | `Alt+F12` | OBSERVED | `Alt+F12` | IMPLEMENTED | UI-013 | View › Close all plugin windows (MENU-124). |
| KB-111 | Close all unfocused windows | `Ctrl+F12` | OBSERVED | `Cmd+F12` | IMPLEMENTED | UI-013 | View › Close all unfocused windows (MENU-125): keeps the focused editor; closes the plugin windows too. |
| KB-112 | Align all channel editors | `Shift+F12` | OBSERVED | `Shift+F12` | IMPLEMENTED | UI-013 | View › Align all channel editors (MENU-126): LimeShot stacks the open plugin windows at the main window's top left. |
| KB-113 | Recording precount | `Ctrl+P` | OBSERVED | — | NOT_STARTED | REC-007 | Options › Recording precount (MENU-168). |
| KB-114 | Start on input | `Ctrl+I` | OBSERVED | — | NOT_STARTED | REC-010 | Options › Start on input (MENU-169). |
| KB-115 | Enable groups | `Alt+Shift+G` | OBSERVED | — | NOT_STARTED | WF-013 | Options › Enable groups (MENU-174). |
| KB-116 | Multilink to controllers | `Ctrl+J` | OBSERVED | — | NOT_STARTED | WF-004 | Options › Multilink to controllers (MENU-175). |
| KB-117 | Browser find | `Ctrl+F` | OBSERVED | — | NOT_STARTED | BR-004 | Tools › Browser find... (MENU-179). |
| KB-118 | Gopher (chat assistant) | `Alt+F1` | OBSERVED | — | NOT_PLANNED |  | Help › Gopher (chat assistant) (MENU-262). |

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
| KB-035 | Quit | `Alt+F4 / Cmd+Q` | OBSERVED | `Cmd+Q` | IMPLEMENTED | APP-014 | Seen in the main menu on macOS (session 13, menus.json). |
| KB-095 | Export MIDI file | `Ctrl+Shift+M` | OBSERVED | — | NOT_STARTED | EXP-012 | File › Export › MIDI file... (MENU-026). |
| KB-096 | Export with FL Cloud mastering | `Ctrl+Alt+M` | OBSERVED | — | NOT_PLANNED |  | File › Export › Master... (MENU-029). |

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
| KB-142 | Clone selected channels | `Opt+C` | OBSERVED | `Alt+C` | IMPLEMENTED | SEQ-007 | FL 2026.1.1 Channel Rack menu (menus.json). Session 16. |

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
| KB-059 | Quick quantize | `Cmd+Q (macOS menu)` | OBSERVED | — | NOT_PLANNED | PR-014 | Quick quantize is in the ⏵ menu without a key: Cmd+Q quits LimeShot (session 16). |
| KB-060 | Quantize dialog | `Opt+Q (macOS menu)` | OBSERVED | `Alt+Q` | IMPLEMENTED | PR-014 | FL 2026.1.1 Tools menu. Session 16: the Quantize… dialog. |
| KB-061 | Legato | `Cmd+L (macOS menu)` | OBSERVED | `Cmd+L` | IMPLEMENTED | PR-015 | Quick legato in FL 2026.1.1's Tools menu. Session 16: in the Piano Roll (the Channel Rack keeps Cmd+L for routing). |
| KB-062 | Arpeggiate / strum / randomize | `Alt+A / Alt+S / Alt+R` | OBSERVED | `Alt+A / Alt+S / Alt+R` | IMPLEMENTED | PR-015 | Session 16: Arpeggiate…, Strum…, Randomize… dialogs. |
| KB-063 | Edit without snapping | `Alt+drag` | VERIFIED | `Alt+click` | IMPLEMENTED | PR-003 | A mouse modifier. |
| KB-079 | Invert selection | `Shift+I` | OBSERVED | `Shift+I` | IMPLEMENTED | PR-002 | From FL 2026.1.1's Select menu. |
| KB-080 | Scroll the view | `Up / Down / Left / Right` | OBSERVED | `↑ / ↓ / ← / →` | IMPLEMENTED | PR-005 | Up/Down move about three rows per press in FL 2026.1.1. |
| KB-081 | Rotate selection left / right | `Shift+Cmd+Left / Shift+Cmd+Right` | OBSERVED | `Shift+Cmd+← / Shift+Cmd+→` | IMPLEMENTED | PR-009 | FL 2026.1.1 Edit menu (macOS labels). LimeShot rotates by the snap; all of the channel's notes when none is selected. |
| KB-082 | Discard lengths | `Shift+D` | OBSERVED | `Shift+D` | IMPLEMENTED | PR-015 | FL 2026.1.1 Edit menu. Session 16: one step long. |
| KB-083 | Mute / unmute selected notes | `Opt+M / Shift+Opt+M` | OBSERVED | `Alt+M / Shift+Alt+M` | IMPLEMENTED | PR-012 | Selection, or all of the channel's notes when none is selected. |
| KB-088 | Glue notes | `Cmd+G` | OBSERVED | `Cmd+G` | IMPLEMENTED | PR-023 | FL 2026.1.1 Tools menu. |
| KB-089 | Quick chop | `Cmd+U` | OBSERVED | `Cmd+U` | IMPLEMENTED | PR-023 | Cuts at every snap step. FL's Chop… dialog (Opt+U) is not implemented. |
| KB-119 | Quick quantize start times | `Shift+Q` | OBSERVED | `Shift+Q` | IMPLEMENTED | PR-014 | FL 2026.1.1 Piano Roll menu (menus.json). |
| KB-120 | Select one at random / more at random | `Shift+R / Shift+M` | OBSERVED | `Shift+R / Shift+M` | IMPLEMENTED | PR-002 | FL 2026.1.1 Piano Roll menu (menus.json). |
| KB-121 | Select odd | `Shift+O` | OBSERVED | `Shift+O` | IMPLEMENTED | PR-002 | FL 2026.1.1 Piano Roll menu (menus.json). |
| KB-122 | Zoom in / out | `PgUp / PgDn` | OBSERVED | `PgUp / PgDn` | IMPLEMENTED | PR-005 | FL 2026.1.1 Piano Roll menu (menus.json). |
| KB-123 | Quick zoom 1–3, zoom out far, zoom on selection, step sequencer mode zoom | `Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5 / Shift+6` | OBSERVED | `Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5 / Shift+6` | DIFFERENT | PR-005 | FL 2026.1.1 Piano Roll menu (menus.json). LimeShot's zoom levels; matched by key position, so they work on QWERTZ. |
| KB-124 | Center | `Shift+0` | OBSERVED | `Shift+0` | DIFFERENT | PR-005 | FL 2026.1.1 Piano Roll menu (menus.json). LimeShot's centring (playback position and notes). |
| KB-125 | Articulate… | `Opt+L` | OBSERVED | `Alt+L` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-126 | Chop… | `Opt+U` | OBSERVED | `Alt+U` | IMPLEMENTED | PR-023 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-127 | Flam… | `Opt+F` | OBSERVED | `Alt+F` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-128 | Limit… | `Opt+K` | OBSERVED | `Alt+K` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-129 | Flip… | `Opt+Y` | OBSERVED | `Alt+Y` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-130 | Scale levels… | `Opt+X` | OBSERVED | `Alt+X` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |
| KB-131 | LFO… | `Opt+O` | OBSERVED | `Alt+O` | IMPLEMENTED | PR-015 | FL 2026.1.1 Piano Roll Tools menu (menus.json). |

## Playlist

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-064 | Draw / paint / delete / mute / slip / slice / select / zoom / playback tools | `P / B / D / T / S / C / E / Z / Y` | VERIFIED | `P / B / D / T / S / C / E / Z / Y` | IMPLEMENTED | PL-017 | Session 8: the tool strip and keys observed live in FL Studio 2026.1.1 (fl-playlist-tools.png); test playlist_tool_keys_and_strip. |
| KB-065 | Select all | `Ctrl+A` | OBSERVED | `Cmd+A` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-066 | Duplicate selection | `Ctrl+B` | VERIFIED | `Cmd+B` | IMPLEMENTED | PL-005 | Globally, Options › Blend recorded notes also shows Ctrl+B; the editor binding wins while an editor has focus. |
| KB-067 | Delete selection | `Delete` | OBSERVED | `Delete / Backspace` | IMPLEMENTED | PL-004 | FL 2026.1.1 Playlist Edit menu (Del). |
| KB-068 | Copy / cut / paste clips | `Ctrl+C / Ctrl+X / Ctrl+V` | OBSERVED | `Cmd+C / Cmd+X / Cmd+V` | IMPLEMENTED | PL-026 | FL 2026.1.1 Playlist Edit menu. |
| KB-069 | Shift clips left / right / up / down | `Shift+Left / Shift+Right / Shift+Up / Shift+Down` | OBSERVED | `Shift+← / Shift+→ / Shift+↑ / Shift+↓` | IMPLEMENTED | PL-002 | FL 2026.1.1 Playlist Edit menu (fl-playlist-edit-menu.png). Up/Down move a track. Plain Up/Down scroll a track (observed live). |
| KB-070 | Add a time marker (asks its name) | `Opt+T` | OBSERVED | `Alt+T` | IMPLEMENTED | PL-016 | FL 2026.1.1 Time markers menu: Add one… |
| KB-071 | Song position to the start | `Home` | INFERRED | `Home` | IMPLEMENTED | PL-009 |  |
| KB-084 | Invert selection | `Shift+I` | OBSERVED | `Shift+I` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-085 | Deselect | `Cmd+D` | OBSERVED | `Cmd+D` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist Select menu. |
| KB-086 | Mute / unmute selected clips | `Opt+M / Shift+Opt+M` | OBSERVED | `Alt+M / Shift+Alt+M` | IMPLEMENTED | PL-005 | FL 2026.1.1 Playlist Edit menu. Session 16. |
| KB-087 | Merge selected clips | `Cmd+G` | OBSERVED | `Cmd+G` | IMPLEMENTED | PL-022 | FL 2026.1.1 Playlist Edit menu; also Merge similar pattern clips Shift+Cmd+G. Session 16: pattern clips into a new pattern. |
| KB-090 | Add a time marker at the song position | `Cmd+T` | OBSERVED | `Cmd+T` | IMPLEMENTED | PL-016 | FL 2026.1.1 Time markers menu: Add auto. Globally, Options › Typing keyboard to piano also shows Ctrl+T; the Playlist binding wins while the Playlist has focus. |
| KB-091 | Time selection | `Ctrl+drag on the ruler` | OBSERVED | `Cmd+drag on the ruler` | IMPLEMENTED | PL-016 | A mouse binding. Song playback loops the selection. |
| KB-092 | Place loop markers around the selection | `Shift+T` | OBSERVED | — | NOT_STARTED | PL-016 | FL 2026.1.1 Time markers menu. |
| KB-132 | Insert space / slice & insert space / delete space | `Cmd+Ins / Cmd+Opt+Ins / Cmd+Del` | OBSERVED | `Cmd+Insert / Cmd+Alt+Insert / Cmd+Delete` | IMPLEMENTED | PL-016 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-133 | Make unique | `Shift+Cmd+V` | OBSERVED | `Shift+Cmd+V` | IMPLEMENTED | PL-011 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-134 | Select by selected source | `Shift+C` | OBSERVED | `Shift+C` | IMPLEMENTED | PL-006 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-135 | Select time around selection | `Cmd+Enter` | OBSERVED | `Cmd+Enter` | IMPLEMENTED | PL-016 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-136 | Quick quantize start times | `Shift+Q` | OBSERVED | `Shift+Q` | IMPLEMENTED | PL-002 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-137 | Zoom in / out | `PgUp / PgDn` | OBSERVED | `PgUp / PgDn` | IMPLEMENTED | PL-008 | FL 2026.1.1 Playlist menu (menus.json). |
| KB-138 | Quick zoom 1–3, zoom out far, zoom on selection | `Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5` | OBSERVED | `Shift+1 / Shift+2 / Shift+3 / Shift+4 / Shift+5` | DIFFERENT | PL-008 | FL 2026.1.1 Playlist menu (menus.json). LimeShot's zoom levels. |
| KB-139 | Center | `Shift+0` | OBSERVED | `Shift+0` | DIFFERENT | PL-008 | FL 2026.1.1 Playlist menu (menus.json). LimeShot's zoom levels. |

## Mixer

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-072 | Link selected channels to the selected track | `Ctrl+L` | INFERRED | — | NOT_STARTED | SEQ-005 | LimeShot's Cmd+L works from the Channel Rack. |
| KB-073 | Move track left / right | `Alt+Left / Alt+Right` | OBSERVED | `Alt+← / Alt+→` | IMPLEMENTED | MIX-006 | FL 2026.1.1 Mixer menu: Move left / right Opt+Left/Right (menus.json). Session 16. |
| KB-074 | Rename track | `F2` | INFERRED | `F2` | IMPLEMENTED | MIX-006 | Session 16: F2 with the Mixer focused, or double-click the name. |
| KB-140 | Delete the selected track | `Del` | OBSERVED | `Delete` | IMPLEMENTED | MIX-006 | FL 2026.1.1 Mixer menu (menus.json). Session 16. |
| KB-141 | Solo the selected track | `S` | OBSERVED | `S` | IMPLEMENTED | MIX-006 | FL 2026.1.1 Mixer menu (menus.json). Session 16. |

## Controls

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-075 | Fine adjustment while dragging | `Ctrl / Alt+drag` | VERIFIED | `Shift+drag` | DIFFERENT | UI-004 |  |
| KB-076 | Reset to default | `Right-click › Reset` | OBSERVED | `Double-click` | DIFFERENT | UI-010 | Observed in FL's control menu (session 4). |

## Piano Roll / Playlist

| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| KB-094 | Zoom vertically | `Alt+Mouse wheel` | INFERRED | `Opt+wheel / Opt+pinch` | IMPLEMENTED | PR-005 | Session 9 (owner's hands-on test: no vertical zoom). Not checked live in FL this session; test alt_wheel_zooms_vertically. |
