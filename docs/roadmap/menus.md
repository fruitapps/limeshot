# Main menu bar and toolbar: FL Studio and LimeShot

> Generated from [`menus.json`](menus.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313), macOS.
> Source: Observed live on 2026-10-22 (session 13): FL's in-window menu bar (FILE … HELP), cross-checked with the native macOS menu bar FL mirrors (Accessibility dump in docs/research/evidence/menus/). Screenshots: docs/research/evidence/screenshots/fl-menu-*.png. Toolbar (menu 'Toolbar') observed live on 2026-10-22 (session 15): each control's name and keys from FL's hint panel while hovering it (fl-toolbar-hints-*.png), and its right-click menus (fl-toolbar-*-menu.png, fl-toolbar-snap.png).

One record per menu item, in FL's order. path = FL's labels from the menu bar down (FL writes '...' for dialogs). section = the grey header the item sits under in-window. kind: action | dialog (opens a dialog or window) | submenu | toggle (checkmark) | radio | list (filled from data; static contents in `lists`) | link (opens a web page). fl_keys in macOS notation as FL shows them. fl_state = the state seen on a fresh default project (checked, selected, disabled). limeshot = where LimeShot has it (Menu › Item). status: IMPLEMENTED (same item), DIFFERENT (LimeShot has the action under another label or place, or without the key), LIMESHOT_ONLY (LimeShot menu item with no FL counterpart in that menu), NOT_STARTED, NOT_PLANNED (Image-Line services, FL's own plugins). Records added after the session 13 capture keep the order of FL's menus; LimeShot-only items follow the FL items of their menu. Toolbar records (menu 'Toolbar', session 15) use kind control (button, knob, field) or display (readout, meter) as well; their right-click menus are child paths.

Tracked: **331** — IMPLEMENTED 57, DIFFERENT 40, LIMESHOT_ONLY 11, NOT_STARTED 193, NOT_PLANNED 30

## Summary

| Menu | FL items | Implemented | Different | Not started | Not planned | LimeShot only |
|---|---|---|---|---|---|---|
| File | 45 | 4 | 7 | 31 | 3 | 0 |
| Edit | 4 | 4 | 0 | 0 | 0 | 0 |
| Add | 26 | 1 | 14 | 10 | 1 | 2 |
| Patterns | 23 | 10 | 5 | 8 | 0 | 0 |
| View | 48 | 15 | 1 | 27 | 5 | 4 |
| Options | 24 | 6 | 0 | 16 | 2 | 2 |
| Tools | 82 | 8 | 3 | 64 | 7 | 0 |
| Help | 16 | 0 | 1 | 6 | 9 | 1 |
| Toolbar | 52 | 9 | 9 | 31 | 3 | 2 |

## Open work by feature

Menu items LimeShot does not have yet (NOT_STARTED), grouped by the feature that implements them, highest priority first.

| Feature | Pri | Status | Name | Menu items |
|---|---|---|---|---|
| TR-002 | P0 | VERIFIED | Play / pause / stop | MENU-296 Toolbar › Tempo › <BPM presets> |
| APP-020 | P1 | NOT_STARTED | MIDI settings | MENU-153 Options › MIDI settings |
| AUT-003 | P1 | VERIFIED | Automate mixer and channel parameters | MENU-217 Tools › Last tweaked › Edit automation clip...<br>MENU-231 Tools › Next to last tweaked › Create automation clip<br>MENU-232 Tools › Next to last tweaked › Edit automation clip... |
| BR-005 | P1 | IMPLEMENTED | Current project tree | MENU-086 Patterns › Open in project browser<br>MENU-106 View › Project picker<br>MENU-141 View › Patterns<br>MENU-142 View › Generators in use<br>MENU-143 View › Effects in use |
| EXP-005 | P1 | NOT_STARTED | MP3 export | MENU-023 File › Export › MP3 file... |
| EXP-012 | P1 | VERIFIED | MIDI export | MENU-243 Tools › Prepare for MIDI export |
| REC-001 | P1 | NOT_STARTED | Audio recording from inputs | MENU-180 Tools › One-click audio recording<br>MENU-182 Tools › One-click audio recording › Into playlist as an audio clip<br>MENU-241 Tools › Cancel recording<br>MENU-319 Toolbar › One-click audio recording |
| REC-002 | P1 | NOT_STARTED | MIDI recording | MENU-281 Toolbar › Record (automation, score, audio, clips)<br>MENU-282 Toolbar › Record (automation, score, audio, clips) › Recording filter › Automation<br>MENU-283 Toolbar › Record (automation, score, audio, clips) › Recording filter › Notes<br>MENU-284 Toolbar › Record (automation, score, audio, clips) › Recording filter › Audio<br>MENU-285 Toolbar › Record (automation, score, audio, clips) › Recording filter › Clips<br>MENU-286 Toolbar › Record (automation, score, audio, clips) › Options › Recording starts playback<br>MENU-287 Toolbar › Record (automation, score, audio, clips) › Options › Disarm on stop<br>MENU-288 Toolbar › Record (automation, score, audio, clips) › Options › Enable recording markers<br>MENU-289 Toolbar › Record (automation, score, audio, clips) › Quantize › Note start time<br>MENU-290 Toolbar › Record (automation, score, audio, clips) › Quantize › Note end time<br>MENU-291 Toolbar › Record (automation, score, audio, clips) › Quantize › Leave note duration<br>MENU-292 Toolbar › Record (automation, score, audio, clips) › Quantize › Record to step sequencer<br>MENU-293 Toolbar › Record (automation, score, audio, clips) › Quantize › Automations |
| TR-008 | P1 | VERIFIED | Metronome | MENU-298 Toolbar › Song position › Mode<br>MENU-299 Toolbar › Song position › Middle unit |
| UI-004 | P1 | VERIFIED | Hint bar and tooltips | MENU-117 View › Toolbars › Hint bar |
| UI-010 | P1 | IN_PROGRESS | Control context menu | MENU-220 Tools › Last tweaked › Copy value<br>MENU-221 Tools › Last tweaked › Paste value<br>MENU-222 Tools › Last tweaked › Type in value...<br>MENU-223 Tools › Last tweaked › Set...<br>MENU-235 Tools › Next to last tweaked › Copy value<br>MENU-236 Tools › Next to last tweaked › Paste value<br>MENU-237 Tools › Next to last tweaked › Type in value...<br>MENU-238 Tools › Next to last tweaked › Set... |
| WF-003 | P1 | NOT_STARTED | MIDI controller input | MENU-163 Options › Enable MIDI remote control |
| APP-015 | P2 | NOT_STARTED | Project templates | MENU-002 File › New from template<br>MENU-003 File › New from template › Advanced<br>MENU-004 File › New from template › Minimal<br>MENU-005 File › New from template › Other<br>MENU-006 File › New from template › Performance<br>MENU-007 File › New from template › Power User<br>MENU-008 File › New from template › Utility<br>MENU-009 File › New from template › Empty<br>MENU-010 File › New from template › Change default template...<br>MENU-017 File › Save as template... |
| APP-018 | P2 | NOT_STARTED | Save new version | MENU-014 File › Save new version |
| APP-019 | P2 | NOT_STARTED | Project bundle (zip with samples) | MENU-040 File › Export › Zipped loop package... |
| APP-021 | P2 | NOT_STARTED | Plugin manager | MENU-072 Add › Manage plugins<br>MENU-158 Options › Manage plugins |
| APP-027 | P2 | NOT_STARTED | Revert to last backup | MENU-044 File › Revert to last backup |
| APP-032 | P2 | IN_PROGRESS | Help index, tutorials and about | MENU-261 Help › Help index<br>MENU-263 Help › Tutorials<br>MENU-264 Help › Tutorials › Loop starter tutorial<br>MENU-265 Help › Tutorials › Step sequencer tutorial<br>MENU-269 Help › What's new |
| AUT-005 | P2 | NOT_STARTED | Event editor | MENU-211 Tools › Last tweaked › Edit events<br>MENU-212 Tools › Last tweaked › Edit events in new window<br>MENU-213 Tools › Last tweaked › Edit events in piano roll<br>MENU-226 Tools › Next to last tweaked › Edit events<br>MENU-227 Tools › Next to last tweaked › Edit events in new window<br>MENU-228 Tools › Next to last tweaked › Edit events in piano roll |
| BR-008 | P2 | NOT_STARTED | Plugin and preset browser | MENU-060 Add › Categories<br>MENU-061 Add › Simple<br>MENU-062 Add › Tree<br>MENU-064 Add › Effect › More plugins...<br>MENU-065 Add › Effect › Plugin database<br>MENU-068 Add › Browse plugin database<br>MENU-069 Add › Browse all installed plugins<br>MENU-145 View › Plugin database |
| EXP-006 | P2 | NOT_STARTED | FLAC / OGG export | MENU-024 File › Export › OGG file...<br>MENU-025 File › Export › FLAC file...<br>MENU-027 File › Export › M4A file... |
| EXP-014 | P2 | NOT_STARTED | Export playlist tracks or track groups | MENU-031 File › Export › All playlist tracks<br>MENU-032 File › Export › All playlist tracks › From song start...<br>MENU-033 File › Export › All playlist tracks › From track start...<br>MENU-034 File › Export › All playlist tracks › Time selection...<br>MENU-035 File › Export › All playlist track groups<br>MENU-036 File › Export › All playlist track groups › From song start...<br>MENU-037 File › Export › All playlist track groups › From track start...<br>MENU-038 File › Export › All playlist track groups › Time selection... |
| EXP-015 | P2 | NOT_STARTED | Export in the last used format(s) | MENU-028 File › Export › Last exported format(s)... |
| PL-022 | P2 | NOT_STARTED | Consolidate / render to audio clip | MENU-098 Patterns › Quick render as audio clip<br>MENU-099 Patterns › Render as audio clip...<br>MENU-100 Patterns › Render and replace... |
| PL-023 | P2 | NOT_STARTED | Crossfades and time stretching | MENU-250 Tools › Switch all audio clip channels to<br>MENU-251 Tools › Switch selected audio clip channels to |
| PLG-008 | P2 | IMPLEMENTED | Plugin preset management | MENU-070 Add › Browse presets |
| REC-005 | P2 | NOT_STARTED | Loop recording / takes | MENU-172 Options › Loop record<br>MENU-302 Toolbar › Loop record |
| REC-007 | P2 | NOT_STARTED | Count-in | MENU-168 Options › Recording precount<br>MENU-301 Toolbar › Recording precount |
| REC-010 | P2 | NOT_STARTED | Start on input and blend recorded notes | MENU-169 Options › Start on input<br>MENU-170 Options › Blend recorded notes |
| TR-012 | P2 | NOT_STARTED | Tap tempo | MENU-108 View › Tempo tapper<br>MENU-295 Toolbar › Tempo › Tempo tapper |
| UI-013 | P2 | IMPLEMENTED | Window layout commands | MENU-131 View › Arrange windows › Desktop (alternate)<br>MENU-132 View › Arrange windows › Tablet (default)<br>MENU-133 View › Arrange windows › Tablet (alternate) |
| UI-016 | P2 | NOT_STARTED | Auto scrolling option | MENU-173 Options › Auto scrolling<br>MENU-318 Toolbar › Auto scrolling |
| WF-002 | P2 | NOT_STARTED | Typing keyboard to piano | MENU-166 Options › Typing keyboard to piano<br>MENU-300 Toolbar › Typing keyboard to piano |
| WF-004 | P2 | NOT_STARTED | Controller mapping (link to controller) | MENU-144 View › Remote control<br>MENU-175 Options › Multilink to controllers<br>MENU-218 Tools › Last tweaked › Link to controller...<br>MENU-219 Tools › Last tweaked › Override global link...<br>MENU-224 Tools › Last tweaked › Override volatile link...<br>MENU-233 Tools › Next to last tweaked › Link to controller...<br>MENU-234 Tools › Next to last tweaked › Override global link...<br>MENU-239 Tools › Next to last tweaked › Override volatile link... |
| WF-005 | P2 | NOT_STARTED | External MIDI devices / MIDI out | MENU-164 Options › Enable MIDI output |
| WF-010 | P2 | NOT_STARTED | Score logger | MENU-194 Tools › Dump score log to selected pattern<br>MENU-195 Tools › Dump score log to selected pattern › Last 2 minutes<br>MENU-196 Tools › Dump score log to selected pattern › Last 5 minutes<br>MENU-197 Tools › Dump score log to selected pattern › Last 10 minutes<br>MENU-198 Tools › Dump score log to selected pattern › Last 20 minutes<br>MENU-199 Tools › Dump score log to selected pattern › Last 30 minutes<br>MENU-200 Tools › Clear log |
| APP-023 | P3 | NOT_STARTED | Welcome window | MENU-122 View › Welcome to FL Studio |
| APP-028 | P3 | NOT_STARTED | Save to new project folder | MENU-016 File › Save to new project folder... |
| APP-029 | P3 | NOT_STARTED | Delete unused files and save project | MENU-018 File › Delete unused files and save project<br>MENU-253 Tools › Delete unused files and save project |
| APP-031 | P3 | NOT_STARTED | Debugging log and diagnostics | MENU-161 Options › Debugging log<br>MENU-268 Help › Diagnostics |
| AUT-011 | P3 | NOT_STARTED | Init song with this position | MENU-214 Tools › Last tweaked › Init song with this position<br>MENU-215 Tools › Last tweaked › Delete initial value<br>MENU-229 Tools › Next to last tweaked › Init song with this position<br>MENU-230 Tools › Next to last tweaked › Delete initial value |
| BR-011 | P3 | NOT_STARTED | Save clipboard to browser | MENU-192 Tools › Save audio clipboard to browser<br>MENU-193 Tools › Save MIDI clipboard to browser |
| EXP-016 | P3 | NOT_STARTED | Video export | MENU-039 File › Export › Video file... |
| EXP-017 | P3 | NOT_STARTED | Project bones and project data files | MENU-041 File › Export › Project bones...<br>MENU-042 File › Export › Project data files... |
| PL-019 | P3 | NOT_STARTED | Performance mode | MENU-242 Tools › Prepare for performance mode |
| PL-030 | P3 | DEFERRED | Stem separation (Remix a song) | MENU-183 Tools › Remix a song... |
| PLG-011 | P3 | NOT_STARTED | Plugin performance monitor | MENU-121 View › Plugin performance monitor |
| PR-022 | P3 | NOT_STARTED | Time signature per pattern / time markers | MENU-090 Patterns › Set time signature... |
| PR-028 | P3 | NOT_STARTED | Chord progression tool | MENU-185 Tools › Chord progression tool... |
| PR-029 | P3 | NOT_STARTED | Riff machine | MENU-184 Tools › Riff machine... |
| REC-009 | P3 | NOT_STARTED | Step edit mode | MENU-171 Options › Step edit |
| REC-011 | P3 | NOT_STARTED | Audio logger | MENU-201 Tools › Enabled<br>MENU-202 Tools › Settings<br>MENU-203 Tools › Settings › 1 minute<br>MENU-204 Tools › Settings › 2 minutes<br>MENU-205 Tools › Settings › 5 minutes<br>MENU-206 Tools › Settings › 10 minutes<br>MENU-207 Tools › Settings › Pre effects<br>MENU-208 Tools › Settings › Post effects, pre fader<br>MENU-209 Tools › Dump to audio editor |
| SEQ-022 | P3 | NOT_STARTED | Pattern groups | MENU-087 Patterns › Group selected...<br>MENU-088 Patterns › Rename group...<br>MENU-089 Patterns › Ungroup selected... |
| UI-006 | P3 | NOT_STARTED | Additional themes (light, high contrast) | MENU-157 Options › Theme settings |
| UI-014 | P3 | NOT_STARTED | Toolbar customisation and presets | MENU-111 View › Toolbars<br>MENU-112 View › Toolbars › Edit<br>MENU-113 View › Toolbars › Separators<br>MENU-114 View › Toolbars › Flat buttons<br>MENU-116 View › Toolbars › Show detected MIDI chords near the hint bar<br>MENU-118 View › Toolbars › Hide<br>MENU-119 View › Toolbars › <factory presets> |
| UI-015 | P3 | NOT_STARTED | Workspace background | MENU-136 View › Background<br>MENU-137 View › Background › Set image wallpaper...<br>MENU-138 View › Background › Set dynamic wallpaper<br>MENU-139 View › Background › Set background color... |
| UI-017 | P3 | DEFERRED | Control creator | MENU-186 Tools › Control creator |
| UI-018 | P3 | NOT_STARTED | Plugin picker | MENU-067 Add › Plugin picker<br>MENU-107 View › Plugin picker |
| WF-008 | P3 | NOT_STARTED | Python-style scripting API | MENU-110 View › Script output |
| WF-009 | P3 | IN_PROGRESS | Project macros | MENU-246 Tools › Switch smart disable for all plugins<br>MENU-247 Tools › Lock all playlist tracks to content<br>MENU-248 Tools › Unlock all playlist tracks from content<br>MENU-254 Tools › Switch all plugin editors to<br>MENU-255 Tools › Switch all plugin editors to › Attached<br>MENU-256 Tools › Switch all plugin editors to › Detached<br>MENU-260 Tools › Change declicking for selected channels to |
| WF-011 | P3 | NOT_STARTED | Touch controller | MENU-109 View › Touch controller |
| WF-012 | P3 | NOT_STARTED | MIDI clock sync | MENU-165 Options › Enable MIDI master sync |
| WF-013 | P3 | NOT_STARTED | Item groups | MENU-174 Options › Enable groups |
| — |  |  |  | MENU-304 Toolbar › Detected chord<br>MENU-305 Toolbar › Monitor (master)<br>MENU-308 Toolbar › CPU load › Refresh<br>MENU-309 Toolbar › CPU load › View voice count<br>MENU-310 Toolbar › CPU load › View FPS<br>MENU-311 Toolbar › CPU load history<br>MENU-324 Toolbar › Shift key modifier<br>MENU-325 Toolbar › Command key modifier<br>MENU-326 Toolbar › Option key modifier |

## File

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-001 | New (Basic with limiter) |  | action |  |  | File › New | DIFFERENT | APP-001 | FL names the default template in the label. LimeShot's New is an empty 140 BPM project and has Cmd+N; FL's Cmd+N is Save new version. |
| MENU-002 | New from template |  | submenu |  |  | — | NOT_STARTED | APP-015 | Each template is a card with a title, a one-line description and an icon. |
| MENU-003 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Advanced |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Advanced; full list in lists.templates.Advanced. |
| MENU-004 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Minimal |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Minimal; full list in lists.templates.Minimal. |
| MENU-005 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Other |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Other; full list in lists.templates.Other. |
| MENU-006 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Performance |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Performance; full list in lists.templates.Performance. |
| MENU-007 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Power User |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Power User; full list in lists.templates.Power User. |
| MENU-008 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Utility |  | list |  |  | — | NOT_STARTED | APP-015 | Bundled templates in Data/Templates/Utility; full list in lists.templates.Utility. |
| MENU-009 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Empty |  | action |  |  | — | NOT_STARTED | APP-015 | Shown as a card below the categories. |
| MENU-010 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change default template... |  | dialog |  |  | — | NOT_STARTED | APP-015 | Picks which template New and the File menu's first item use. |
| MENU-011 | Open... |  | dialog | `Cmd+O` |  | File › Open… | IMPLEMENTED | APP-002 |  |
| MENU-012 | Save |  | action | `Cmd+S` |  | File › Save | IMPLEMENTED | APP-003 |  |
| MENU-013 | Save as... |  | dialog | `Shift+Cmd+S` |  | File › Save as… | IMPLEMENTED | APP-003 |  |
| MENU-014 | Save new version |  | action | `Cmd+N` |  | — | NOT_STARTED | APP-018 | Saves name_2.flp, name_3.flp … next to the project. |
| MENU-015 | Save and back up to FL Cloud |  | action |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-016 | Save to new project folder... |  | dialog |  | disabled | — | NOT_STARTED | APP-028 | Disabled in a project that was never saved. |
| MENU-017 | Save as template... |  | dialog |  |  | — | NOT_STARTED | APP-015 |  |
| MENU-018 | Delete unused files and save project |  | action |  | disabled | — | NOT_STARTED | APP-029 | Disabled in a project that was never saved; also in Tools › Macros. |
| MENU-019 | Import |  | submenu |  |  | File › Import MIDI file… | DIFFERENT | PR-020 | LimeShot has a flat item instead of the submenu. |
| MENU-020 | &nbsp;&nbsp;&nbsp;&nbsp;↳ MIDI file... |  | dialog |  |  | File › Import MIDI file… | DIFFERENT | PR-020 |  |
| MENU-021 | Export |  | submenu |  |  | File › Export WAV… | DIFFERENT | EXP-001 | LimeShot has flat File items (Export WAV…, Export MIDI file (pattern/song)…) instead of the submenu. |
| MENU-022 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Wave file... | Audio | dialog | `Cmd+R` |  | File › Export WAV… | DIFFERENT | EXP-001 | Same key; LimeShot's label says WAV. Opens: settings.json page 'Render'. |
| MENU-023 | &nbsp;&nbsp;&nbsp;&nbsp;↳ MP3 file... | Audio | dialog | `Shift+Cmd+R` |  | — | NOT_STARTED | EXP-005 | Opens: settings.json page 'Render'. |
| MENU-024 | &nbsp;&nbsp;&nbsp;&nbsp;↳ OGG file... | Audio | dialog |  |  | — | NOT_STARTED | EXP-006 | Opens: settings.json page 'Render'. |
| MENU-025 | &nbsp;&nbsp;&nbsp;&nbsp;↳ FLAC file... | Audio | dialog |  |  | — | NOT_STARTED | EXP-006 | Opens: settings.json page 'Render'. |
| MENU-026 | &nbsp;&nbsp;&nbsp;&nbsp;↳ MIDI file... | Audio | dialog | `Shift+Cmd+M` |  | File › Export MIDI file (song)… | DIFFERENT | EXP-012 | LimeShot has separate pattern and song items and no shortcut. |
| MENU-027 | &nbsp;&nbsp;&nbsp;&nbsp;↳ M4A file... | Audio | dialog |  |  | — | NOT_STARTED | EXP-006 | AAC in an MP4 container. Opens: settings.json page 'Render'. |
| MENU-028 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last exported format(s)... | Audio | dialog |  |  | — | NOT_STARTED | EXP-015 |  |
| MENU-029 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Master... | Audio | dialog | `Opt+Cmd+M` |  | — | NOT_PLANNED |  | FL Cloud mastering (cloud icon). |
| MENU-030 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Upload... | Audio | dialog |  |  | — | NOT_PLANNED |  | Upload to Image-Line's distribution service. |
| MENU-031 | &nbsp;&nbsp;&nbsp;&nbsp;↳ All playlist tracks | Audio | submenu |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-032 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ From song start... | Audio | dialog |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-033 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ From track start... | Audio | dialog |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-034 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Time selection... | Audio | dialog |  | disabled | — | NOT_STARTED | EXP-014 | Enabled when the Playlist has a time selection. |
| MENU-035 | &nbsp;&nbsp;&nbsp;&nbsp;↳ All playlist track groups | Audio | submenu |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-036 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ From song start... | Audio | dialog |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-037 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ From track start... | Audio | dialog |  |  | — | NOT_STARTED | EXP-014 |  |
| MENU-038 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Time selection... | Audio | dialog |  | disabled | — | NOT_STARTED | EXP-014 |  |
| MENU-039 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Video file... | Video | dialog |  |  | — | NOT_STARTED | EXP-016 |  |
| MENU-040 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Zipped loop package... | Project | dialog |  |  | — | NOT_STARTED | APP-019 | Project plus every sample it uses, zipped. |
| MENU-041 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Project bones... | Project | dialog |  |  | — | NOT_STARTED | EXP-017 | A copy of the project without audio data. |
| MENU-042 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Project data files... | Project | dialog |  |  | — | NOT_STARTED | EXP-017 | Copies the project's samples into its folder. |
| MENU-043 | <recent project> | Recent projects | list | `Opt+1 … Opt+9` |  | File › Recent projects | IMPLEMENTED | APP-010 | Listed in the File menu under a 'Recent projects' header, as in FL (up to 10; the full path on hover). LimeShot has no Opt+number keys for them. |
| MENU-044 | Revert to last backup |  | action |  |  | — | NOT_STARTED | APP-027 | Opens the newest autosave/backup of this project. |
| MENU-045 | Exit |  | action | `Cmd+Q` |  | File › Quit | DIFFERENT | APP-014 | LimeShot says Quit (macOS wording). |

## Edit

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-046 | Undo <last action> |  | action | `Cmd+Z` |  | Edit › Undo | IMPLEMENTED | APP-026 | Observed again in session 15: FL 2026 steps back through the history with Cmd+Z ('Undo playlist fill paint clip', then 'Undo step seq edit') and shows a separate 'Redo <action>' item (Cmd+Opt+Z) only while there is something to redo (session 13 had read it as one slot). LimeShot does the same, with LimeShot's action names; Cmd+Shift+Z and Cmd+Y also redo. |
| MENU-047 | Cut |  | action | `Cmd+X` |  | Edit › Cut | IMPLEMENTED | APP-026 | Acts on the focused editor (Playlist clips, Piano Roll notes, Channel Rack steps); disabled when it has nothing selected. |
| MENU-048 | Copy |  | action | `Cmd+C` |  | Edit › Copy | IMPLEMENTED | APP-026 | As Cut. |
| MENU-049 | Paste |  | action | `Cmd+V` | disabled | Edit › Paste | IMPLEMENTED | APP-026 | Disabled while the clipboard holds nothing the focused editor can take. |

## Add

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-050 | More plugins... | Channel | dialog |  |  | Channel Rack › + › Find plugins… | DIFFERENT | BR-008 | Opens the plugin manager/list of all installed generators. |
| MENU-051 | <Controller generators> | Controller | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Controller']. Picking one adds a channel. |
| MENU-052 | <Drum generators> | Drum | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Drum']. Picking one adds a channel. |
| MENU-053 | <MIDI generators> | MIDI | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['MIDI']. Picking one adds a channel. |
| MENU-054 | <Misc generators> | Misc | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Misc']. Picking one adds a channel. |
| MENU-055 | <Patcher generators> | Patcher | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Patcher']. Picking one adds a channel. |
| MENU-056 | <Sampler generators> | Sampler | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Sampler']. Picking one adds a channel. |
| MENU-057 | <Synth classic generators> | Synth classic | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Synth classic']. Picking one adds a channel. |
| MENU-058 | <Synth special generators> | Synth special | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Synth special']. Picking one adds a channel. |
| MENU-059 | <Visual generators> | Visual | list |  |  | Add › Sampler channel / LimeSynth channel | DIFFERENT | SEQ-002 | FL's installed generators of this category; full list in lists.generators['Visual']. Picking one adds a channel. |
| MENU-060 | Categories |  | radio |  | selected | — | NOT_STARTED | BR-008 | List view mode for the generator and effect lists. |
| MENU-061 | Simple |  | radio |  |  | — | NOT_STARTED | BR-008 |  |
| MENU-062 | Tree |  | radio |  |  | — | NOT_STARTED | BR-008 |  |
| MENU-063 | Effect | Other | submenu |  |  | Mixer › insert slot menu | DIFFERENT | MIX-007 | Adds an effect to the selected mixer track's next free slot. Groups in lists.effects. |
| MENU-064 | &nbsp;&nbsp;&nbsp;&nbsp;↳ More plugins... | Other | dialog |  |  | — | NOT_STARTED | BR-008 |  |
| MENU-065 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Plugin database | Other | action | `Shift+F8` |  | — | NOT_STARTED | BR-008 | Opens the Browser at the plugin database. |
| MENU-066 | &nbsp;&nbsp;&nbsp;&nbsp;↳ <effects by category> | Other | list |  |  | Mixer › insert slot menu | DIFFERENT | FX-001 | Same view-mode radio (Categories / Simple / Tree) at the end. |
| MENU-067 | Plugin picker | Other | toggle | `F8` |  | — | NOT_STARTED | UI-018 |  |
| MENU-068 | Browse plugin database | Other | action |  |  | — | NOT_STARTED | BR-008 |  |
| MENU-069 | Browse all installed plugins | Other | action |  |  | — | NOT_STARTED | BR-008 |  |
| MENU-070 | Browse presets | Other | action |  |  | — | NOT_STARTED | PLG-008 |  |
| MENU-071 | Refresh plugin list (fast scan) | Other | action |  |  | plugin menus › Find plugins… | DIFFERENT | PLG-004 |  |
| MENU-072 | Manage plugins | Other | dialog |  |  | — | NOT_STARTED | APP-021 | Opens: settings.json page 'Plugin Manager'. |
| MENU-073 | Manage FL Cloud plugins... | Other | dialog |  |  | — | NOT_PLANNED |  |  |
| MENU-074 | Automation for last tweaked parameter | Other | action |  | disabled | Add › Automation for last tweaked parameter | DIFFERENT | PLG-007 | Disabled until a control was tweaked. LimeShot remembers plugin parameters only (tweaked in the plugin's editor or parameter list); other controls offer Create automation clip on right-click. |
| MENU-075 | Pattern | Other | action |  |  | Add › Pattern | IMPLEMENTED | SEQ-018 |  |
| MENU-076 | Playlist track |  | action |  |  | Add › Playlist track | LIMESHOT_ONLY | PL-012 | Not in FL's Add menu. |
| MENU-077 | Mixer track |  | action |  |  | Add › Mixer track | LIMESHOT_ONLY | MIX-006 | Not in FL's Add menu. |

## Patterns

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-078 | <pattern list> |  | list |  |  | Patterns › pattern list | IMPLEMENTED | SEQ-018 | FL shows the patterns in a column left of the commands; clicking one makes it current. |
| MENU-079 | Find first empty... |  | dialog | `Shift+F4` |  | Patterns › Find first empty… | IMPLEMENTED | SEQ-021 | Selects the first empty pattern (a new one at the end when none is empty) and asks for a name. |
| MENU-080 | Find next empty... |  | dialog | `F4` |  | Patterns › Find next empty… | IMPLEMENTED | SEQ-021 | The first empty pattern after the current one, else a new one; asks for a name. |
| MENU-081 | Find next empty (no naming) |  | action | `Cmd+F4` |  | Patterns › Find next empty (no naming) | IMPLEMENTED | SEQ-021 |  |
| MENU-082 | Select in playlist |  | action |  |  | Patterns › Select in playlist | IMPLEMENTED | SEQ-021 | Selects the current pattern's clips and shows the Playlist. |
| MENU-083 | Rename and color... |  | dialog | `F2` |  | Patterns › Rename pattern… | DIFFERENT | SEQ-018 | LimeShot has rename (F2) and Color as separate items. |
| MENU-084 | Change color... |  | dialog |  |  | Patterns › Color | DIFFERENT | SEQ-018 |  |
| MENU-085 | Random color |  | action |  |  | Patterns › Random color | IMPLEMENTED | SEQ-021 | Another palette colour, at random. |
| MENU-086 | Open in project browser |  | action | `Shift+Cmd+P` |  | — | NOT_STARTED | BR-005 |  |
| MENU-087 | Group selected... |  | dialog |  |  | — | NOT_STARTED | SEQ-022 | In the native macOS menu mirror; hidden in-window with one pattern. |
| MENU-088 | Rename group... |  | dialog |  |  | — | NOT_STARTED | SEQ-022 | As Group selected. |
| MENU-089 | Ungroup selected... |  | action |  |  | — | NOT_STARTED | SEQ-022 | As Group selected. |
| MENU-090 | Set time signature... |  | dialog |  |  | — | NOT_STARTED | PR-022 | Opens: settings.json page 'Patterns dialogs'. |
| MENU-091 | Transpose... |  | dialog |  |  | Patterns › Transpose… | DIFFERENT | SEQ-021 | FL: a small inline field at the pointer; LimeShot: a small dialog with Semitones, OK and Cancel. Moves every note of the pattern (one undo step). |
| MENU-092 | Insert one |  | action | `Shift+Cmd+Ins` |  | Patterns › Insert one | IMPLEMENTED | SEQ-021 | An empty pattern before the current one, made current; default names renumber by position; asks for a name. FL: not an undo step (observed session 15); LimeShot: one. |
| MENU-093 | Clone |  | action | `Shift+Cmd+C` |  | Patterns › Clone pattern | DIFFERENT | SEQ-018 | Same action and key; LimeShot's label adds 'pattern'. |
| MENU-094 | Delete |  | action | `Shift+Cmd+Del` |  | Patterns › Delete pattern | DIFFERENT | SEQ-018 | Same action and key; LimeShot's label adds 'pattern'. |
| MENU-095 | Move up |  | action | `Shift+Cmd+Up` |  | Patterns › Move up | IMPLEMENTED | SEQ-021 | Default names renumber by position. |
| MENU-096 | Move down |  | action | `Shift+Cmd+Down` |  | Patterns › Move down | IMPLEMENTED | SEQ-021 | Default names renumber by position. |
| MENU-097 | Split by channel |  | action |  |  | Patterns › Split by channel | IMPLEMENTED | SEQ-021 | Observed session 15: the pattern keeps its first channel's notes and that channel's name; each other channel with notes gets a new pattern named after it at the end of the list; Playlist clips keep the original. One undo step ('Undo split by channel'). |
| MENU-098 | Quick render as audio clip |  | action |  |  | — | NOT_STARTED | PL-022 |  |
| MENU-099 | Render as audio clip... |  | dialog |  |  | — | NOT_STARTED | PL-022 |  |
| MENU-100 | Render and replace... |  | dialog |  |  | — | NOT_STARTED | PL-022 |  |

## View

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-101 | Playlist | Windows | toggle | `F5` | checked | View › Playlist | IMPLEMENTED | UI-002 |  |
| MENU-102 | Piano roll | Windows | toggle | `F7` |  | View › Piano Roll | IMPLEMENTED | UI-002 |  |
| MENU-103 | Channel rack | Windows | toggle | `F6` | checked | View › Channel Rack | IMPLEMENTED | UI-002 |  |
| MENU-104 | Mixer | Windows | toggle | `F9` |  | View › Mixer | IMPLEMENTED | UI-002 |  |
| MENU-105 | Browser | Windows | toggle | `Opt+F8` | checked | View › Browser | IMPLEMENTED | UI-001 |  |
| MENU-106 | Project picker | Windows | toggle | `Cmd+F8` |  | — | NOT_STARTED | BR-005 | A floating picker of the project's patterns, clips and channels. |
| MENU-107 | Plugin picker | Windows | toggle | `F8` |  | — | NOT_STARTED | UI-018 |  |
| MENU-108 | Tempo tapper | Windows | toggle | `Cmd+F5` |  | — | NOT_STARTED | TR-012 |  |
| MENU-109 | Touch controller | Windows | toggle | `Opt+F7` |  | — | NOT_STARTED | WF-011 |  |
| MENU-110 | Script output | Windows | toggle |  |  | — | NOT_STARTED | WF-008 |  |
| MENU-111 | Toolbars | Windows | submenu |  |  | — | NOT_STARTED | UI-014 |  |
| MENU-112 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit | Windows | toggle |  |  | — | NOT_STARTED | UI-014 | Section 'Toolbars': toolbar edit mode. |
| MENU-113 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Separators | Windows | toggle |  | checked | — | NOT_STARTED | UI-014 |  |
| MENU-114 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Flat buttons | Windows | toggle |  |  | — | NOT_STARTED | UI-014 |  |
| MENU-115 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show FL Cloud status in the hint bar | Windows | toggle |  | checked | — | NOT_PLANNED |  |  |
| MENU-116 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show detected MIDI chords near the hint bar | Windows | toggle |  |  | — | NOT_STARTED | UI-014 |  |
| MENU-117 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Hint bar | Windows | toggle |  |  | — | NOT_STARTED | UI-004 |  |
| MENU-118 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Hide | Windows | toggle | `Cmd+F11` |  | — | NOT_STARTED | UI-014 |  |
| MENU-119 | &nbsp;&nbsp;&nbsp;&nbsp;↳ <factory presets> | Windows | radio |  | Default (macOS) selected | — | NOT_STARTED | UI-014 | Section 'Factory Presets'; full list in lists.toolbar_presets. |
| MENU-120 | Tests | Windows | toggle |  |  | — | NOT_PLANNED |  | Image-Line's internal test window. |
| MENU-121 | Plugin performance monitor | Windows | toggle |  |  | — | NOT_STARTED | PLG-011 |  |
| MENU-122 | Welcome to FL Studio | Windows | dialog |  |  | — | NOT_STARTED | APP-023 |  |
| MENU-123 | Close all windows | Layout | action | `F12` |  | View › Close all windows | IMPLEMENTED | UI-013 | Editors and plugin windows. |
| MENU-124 | Close all plugin windows | Layout | action | `Opt+F12` |  | View › Close all plugin windows | IMPLEMENTED | UI-013 |  |
| MENU-125 | Close all unfocused windows | Layout | action | `Cmd+F12` |  | View › Close all unfocused windows | IMPLEMENTED | UI-013 | Keeps the focused editor's window. |
| MENU-126 | Align all channel editors | Layout | action | `Shift+F12` |  | View › Align all channel editors | IMPLEMENTED | UI-013 | Stacks the open plugin windows at the main window's top left (LimeShot's Channel Settings is a single workspace window). |
| MENU-127 | Arrange windows | Layout | submenu |  |  | View › Arrange windows | IMPLEMENTED | UI-013 | Submenu: Realign, Into workspace, Layouts (Desktop (default), Custom), Save custom layout. |
| MENU-128 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Realign | Layout | action | `Shift+Cmd+H` |  | View › Arrange windows › Realign | IMPLEMENTED | UI-002 | Every window back at its default place. |
| MENU-129 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Into workspace | Layout | action |  |  | View › Arrange windows › Into workspace | IMPLEMENTED | UI-013 | Docks detached windows back into the main window. |
| MENU-130 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Desktop (default) | Layout | action |  |  | View › Arrange windows › Desktop (default) | IMPLEMENTED | UI-013 | FL's new-project layout: Playlist maximised, Channel Rack on top. |
| MENU-131 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Desktop (alternate) | Layout | action |  |  | — | NOT_STARTED | UI-013 | Section 'Layouts'. |
| MENU-132 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tablet (default) | Layout | action |  |  | — | NOT_STARTED | UI-013 | Section 'Layouts'. |
| MENU-133 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tablet (alternate) | Layout | action |  |  | — | NOT_STARTED | UI-013 | Section 'Layouts'. |
| MENU-134 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Custom | Layout | action |  | disabled | View › Arrange windows › Custom | IMPLEMENTED | UI-013 | Enabled once a custom layout is saved (kept in LimeShot's settings). |
| MENU-135 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Save custom layout | Layout | action |  |  | View › Arrange windows › Save custom layout | IMPLEMENTED | UI-013 | Saves which windows are open, maximised, detached and pinned, and where (as fractions of the workspace). |
| MENU-136 | Background | Layout | submenu |  |  | — | NOT_STARTED | UI-015 |  |
| MENU-137 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set image wallpaper... | Layout | dialog |  |  | — | NOT_STARTED | UI-015 |  |
| MENU-138 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set dynamic wallpaper | Layout | action |  |  | — | NOT_STARTED | UI-015 |  |
| MENU-139 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set background color... | Layout | dialog |  |  | — | NOT_STARTED | UI-015 |  |
| MENU-140 | Undo history | Browser | action |  |  | Edit › History | DIFFERENT | APP-006 | FL shows the history in the Browser; LimeShot lists it in an Edit submenu. |
| MENU-141 | Patterns | Browser | action |  |  | — | NOT_STARTED | BR-005 | Browser opens at the project's patterns. |
| MENU-142 | Generators in use | Browser | action |  |  | — | NOT_STARTED | BR-005 |  |
| MENU-143 | Effects in use | Browser | action |  |  | — | NOT_STARTED | BR-005 |  |
| MENU-144 | Remote control | Browser | action |  |  | — | NOT_STARTED | WF-004 |  |
| MENU-145 | Plugin database | Browser | action | `Shift+F8` |  | — | NOT_STARTED | BR-008 |  |
| MENU-146 | Downloads | News and notifications | action |  |  | — | NOT_PLANNED |  |  |
| MENU-147 | News | News and notifications | action |  |  | — | NOT_PLANNED |  |  |
| MENU-148 | Notifications | News and notifications | action |  |  | — | NOT_PLANNED |  |  |
| MENU-149 | Channel Settings |  | toggle |  |  | View › Channel Settings | LIMESHOT_ONLY | UI-009 | FL opens a channel's settings from the Channel Rack only. |
| MENU-150 | Zoom in |  | action | `Cmd+=` |  | View › Zoom in | LIMESHOT_ONLY | UI-005 |  |
| MENU-151 | Zoom out |  | action | `Cmd+-` |  | View › Zoom out | LIMESHOT_ONLY | UI-005 |  |
| MENU-152 | Actual size |  | action | `Cmd+0` |  | View › Actual size | LIMESHOT_ONLY | UI-005 |  |

## Options

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-153 | MIDI settings | System | dialog | `F10` |  | — | NOT_STARTED | APP-020 | Opens: settings.json page 'MIDI' (one window with tabs MIDI, Audio, General, File, Theme, Project, Info, Debug, Account, About). |
| MENU-154 | Audio settings | System | dialog |  |  | Options › Audio settings | IMPLEMENTED | APP-025 | Opens the settings window on the Audio tab. |
| MENU-155 | General settings | System | dialog |  |  | Options › General settings | IMPLEMENTED | APP-025 | Settings window, General tab. |
| MENU-156 | File settings | System | dialog |  |  | Options › File settings | IMPLEMENTED | APP-025 | Settings window, File tab. |
| MENU-157 | Theme settings | System | dialog |  |  | — | NOT_STARTED | UI-006 | Opens: settings.json page 'Theme' (one window with tabs MIDI, Audio, General, File, Theme, Project, Info, Debug, Account, About). |
| MENU-158 | Manage plugins | System | dialog |  |  | — | NOT_STARTED | APP-021 | Opens: settings.json page 'Plugin Manager'. |
| MENU-159 | Project general settings | Project | dialog |  |  | Options › Project general settings | IMPLEMENTED | APP-025 | Settings window, Project tab (time signature). |
| MENU-160 | Project info | Project | dialog | `F11` |  | Options › Project info | IMPLEMENTED | APP-017 | Settings window, Info tab (F11). |
| MENU-161 | Debugging log | Info | dialog |  |  | — | NOT_STARTED | APP-031 | Opens: settings.json page 'Debug' (one window with tabs MIDI, Audio, General, File, Theme, Project, Info, Debug, Account, About). |
| MENU-162 | Account settings | Info | dialog |  |  | — | NOT_PLANNED |  | Opens: settings.json page 'Account' (one window with tabs MIDI, Audio, General, File, Theme, Project, Info, Debug, Account, About). |
| MENU-163 | Enable MIDI remote control | MIDI | toggle |  | checked | — | NOT_STARTED | WF-003 |  |
| MENU-164 | Enable MIDI output | MIDI | toggle |  |  | — | NOT_STARTED | WF-005 |  |
| MENU-165 | Enable MIDI master sync | MIDI | toggle |  |  | — | NOT_STARTED | WF-012 |  |
| MENU-166 | Typing keyboard to piano | Switches | toggle | `Cmd+T` | checked | — | NOT_STARTED | WF-002 |  |
| MENU-167 | Metronome | Switches | toggle | `Cmd+M` |  | Options › Metronome | IMPLEMENTED | TR-008 |  |
| MENU-168 | Recording precount | Switches | toggle | `Cmd+P` | checked | — | NOT_STARTED | REC-007 |  |
| MENU-169 | Start on input | Switches | toggle | `Cmd+I` |  | — | NOT_STARTED | REC-010 |  |
| MENU-170 | Blend recorded notes | Switches | toggle | `Cmd+B` |  | — | NOT_STARTED | REC-010 |  |
| MENU-171 | Step edit | Switches | toggle | `Cmd+E` |  | — | NOT_STARTED | REC-009 |  |
| MENU-172 | Loop record | Switches | toggle |  |  | — | NOT_STARTED | REC-005 |  |
| MENU-173 | Auto scrolling | Switches | toggle |  |  | — | NOT_STARTED | UI-016 |  |
| MENU-174 | Enable groups | Switches | toggle | `Shift+Opt+G` | checked | — | NOT_STARTED | WF-013 |  |
| MENU-175 | Multilink to controllers | Switches | toggle | `Cmd+J` |  | — | NOT_STARTED | WF-004 |  |
| MENU-176 | Manage accounts... | 3rd party services | dialog |  |  | — | NOT_PLANNED |  |  |
| MENU-177 | Loop song |  | toggle |  |  | Options › Loop song | LIMESHOT_ONLY | TR-009 | FL has this on the toolbar/transport, not in Options. |
| MENU-178 | Autosave |  | submenu |  |  | Options › Autosave | LIMESHOT_ONLY | APP-024 | FL sets the autosave interval in File settings. |

## Tools

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-179 | Browser find... |  | dialog | `Cmd+F` |  | Tools › Browser find… | IMPLEMENTED | BR-004 | Shows the Browser and puts the keyboard in its search field. |
| MENU-180 | One-click audio recording |  | submenu |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-181 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Into Edison audio editor/recorder |  | action |  |  | — | NOT_PLANNED |  | Edison is an FL plugin. |
| MENU-182 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Into playlist as an audio clip |  | action |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-183 | Remix a song... |  | dialog |  |  | — | NOT_STARTED | PL-030 | Stem separation into vocals, instruments, bass and drums. |
| MENU-184 | Riff machine... |  | dialog |  |  | — | NOT_STARTED | PR-029 |  |
| MENU-185 | Chord progression tool... |  | dialog |  |  | — | NOT_STARTED | PR-028 |  |
| MENU-186 | Control creator |  | dialog |  |  | — | NOT_STARTED | UI-017 |  |
| MENU-187 | Sounds | FL Cloud | action |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-188 | Mastering... | FL Cloud | dialog |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-189 | Distribution... | FL Cloud | dialog |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-190 | Manage cloud plugins... | FL Cloud | dialog |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-191 | My account... | FL Cloud | dialog |  |  | — | NOT_PLANNED |  | Image-Line cloud service. |
| MENU-192 | Save audio clipboard to browser | Clipboard | action |  | disabled | — | NOT_STARTED | BR-011 | Disabled while the clipboard holds no audio. |
| MENU-193 | Save MIDI clipboard to browser | Clipboard | action |  | disabled | — | NOT_STARTED | BR-011 | Disabled while the clipboard holds no notes. |
| MENU-194 | Dump score log to selected pattern | Score logger | submenu |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-195 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last 2 minutes | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-196 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last 5 minutes | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-197 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last 10 minutes | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-198 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last 20 minutes | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-199 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Last 30 minutes | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-200 | Clear log | Score logger | action |  |  | — | NOT_STARTED | WF-010 |  |
| MENU-201 | Enabled | Audio logger (memory used: 22MB) | toggle |  | checked | — | NOT_STARTED | REC-011 | The section header shows the logger's memory use. |
| MENU-202 | Settings | Audio logger (memory used: 22MB) | submenu |  |  | — | NOT_STARTED | REC-011 |  |
| MENU-203 | &nbsp;&nbsp;&nbsp;&nbsp;↳ 1 minute | Audio logger (memory used: 22MB) | radio |  | selected | — | NOT_STARTED | REC-011 | Section 'Maximum length'. |
| MENU-204 | &nbsp;&nbsp;&nbsp;&nbsp;↳ 2 minutes | Audio logger (memory used: 22MB) | radio |  |  | — | NOT_STARTED | REC-011 | Section 'Maximum length'. |
| MENU-205 | &nbsp;&nbsp;&nbsp;&nbsp;↳ 5 minutes | Audio logger (memory used: 22MB) | radio |  |  | — | NOT_STARTED | REC-011 | Section 'Maximum length'. |
| MENU-206 | &nbsp;&nbsp;&nbsp;&nbsp;↳ 10 minutes | Audio logger (memory used: 22MB) | radio |  |  | — | NOT_STARTED | REC-011 | Section 'Maximum length'. |
| MENU-207 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Pre effects | Audio logger (memory used: 22MB) | radio |  |  | — | NOT_STARTED | REC-011 | Section 'Recording position'. |
| MENU-208 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Post effects, pre fader | Audio logger (memory used: 22MB) | radio |  | selected | — | NOT_STARTED | REC-011 | Section 'Recording position'. |
| MENU-209 | Dump to audio editor | Audio logger (memory used: 22MB) | action |  |  | — | NOT_STARTED | REC-011 |  |
| MENU-210 | Last tweaked | Last tweaked parameters | submenu |  |  | Tools › Last tweaked | DIFFERENT | UI-010 | LimeShot remembers plugin parameters (changed in the plugin's editor or parameter list); the submenu has Create automation clip. Other controls offer their menu on right-click. |
| MENU-211 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-212 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events in new window | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-213 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events in piano roll | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-214 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Init song with this position | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-011 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-215 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Delete initial value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-011 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-216 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Create automation clip | Last tweaked parameters | action |  | disabled | Tools › Last tweaked › Create automation clip | DIFFERENT | PLG-007 | For the last tweaked plugin parameter. |
| MENU-217 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit automation clip... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | AUT-003 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-218 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Link to controller... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-219 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Override global link... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-220 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Copy value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-221 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Paste value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-222 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Type in value... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-223 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-224 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Override volatile link... | Last tweaked parameters | dialog |  |  | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-225 | Next to last tweaked | Last tweaked parameters | submenu |  |  | control › context menu | DIFFERENT | UI-010 | The same menu a control's right-click opens; first line names the parameter ('No parameter' when none). LimeShot has the menu on controls only. |
| MENU-226 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-227 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events in new window | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-228 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit events in piano roll | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-005 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-229 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Init song with this position | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-011 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-230 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Delete initial value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-011 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-231 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Create automation clip | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | AUT-003 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-232 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit automation clip... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | AUT-003 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-233 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Link to controller... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-234 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Override global link... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-235 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Copy value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-236 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Paste value | Last tweaked parameters | action |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-237 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Type in value... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-238 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set... | Last tweaked parameters | dialog |  | disabled | — | NOT_STARTED | UI-010 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-239 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Override volatile link... | Last tweaked parameters | dialog |  |  | — | NOT_STARTED | WF-004 | Sections: Automation, Remote control, Value. Disabled without a tweaked parameter. |
| MENU-240 | Stop sound | Macros | action | `Cmd+H` |  | Tools › Stop sound | IMPLEMENTED | TR-010 | Cmd+H, as before. |
| MENU-241 | Cancel recording | Macros | action |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-242 | Prepare for performance mode | Macros | action |  |  | — | NOT_STARTED | PL-019 |  |
| MENU-243 | Prepare for MIDI export | Macros | action |  |  | — | NOT_STARTED | EXP-012 | Converts the project into one MIDI channel per instrument before export. |
| MENU-244 | Select unused channels | Macros | action |  |  | Tools › Select unused channels | IMPLEMENTED | WF-009 | Selects the channels no pattern and no audio clip uses and shows the Channel Rack. |
| MENU-245 | Purge unused audio clips | Macros | action |  |  | Tools › Purge unused audio clips | IMPLEMENTED | WF-009 | Deletes the audio clip channels (the Audio filter group) no Playlist clip plays and no pattern uses; one undo step. |
| MENU-246 | Switch smart disable for all plugins | Macros | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-247 | Lock all playlist tracks to content | Macros | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-248 | Unlock all playlist tracks from content | Macros | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-249 | Reset empty playlist tracks | Macros | action |  |  | Tools › Reset empty playlist tracks | IMPLEMENTED | WF-009 | Tracks without clips get their default name back and are unmuted; one undo step. LimeShot tracks have no colour or height of their own yet. |
| MENU-250 | Switch all audio clip channels to | Macros | submenu |  |  | — | NOT_STARTED | PL-023 | Time-stretch mode; options in lists.stretch_modes. |
| MENU-251 | Switch selected audio clip channels to | Macros | submenu |  |  | — | NOT_STARTED | PL-023 | Time-stretch mode; options in lists.stretch_modes. |
| MENU-252 | Remove all Edison instances | Macros | action |  |  | — | NOT_PLANNED |  | Edison is an FL plugin. |
| MENU-253 | Delete unused files and save project | Macros | action |  | disabled | — | NOT_STARTED | APP-029 |  |
| MENU-254 | Switch all plugin editors to | Macros | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-255 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Attached | Macros | radio |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-256 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Detached | Macros | radio |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-257 | Set keyboard focus for all plugins to | Macros | submenu |  |  | Tools › Set keyboard focus for all plugins to | IMPLEMENTED | WF-009 | Sets every plugin window's keyboard toggle (PLG-006), open or closed. |
| MENU-258 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Enabled | Macros | radio |  |  | Tools › Set keyboard focus for all plugins to › Enabled | IMPLEMENTED | WF-009 |  |
| MENU-259 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Disabled | Macros | radio |  |  | Tools › Set keyboard focus for all plugins to › Disabled | IMPLEMENTED | WF-009 |  |
| MENU-260 | Change declicking for selected channels to | Macros | submenu |  |  | — | NOT_STARTED | WF-009 | Options in lists.declicking. |

## Help

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-261 | Help index |  | link |  |  | — | NOT_STARTED | APP-032 |  |
| MENU-262 | Gopher (chat assistant) |  | dialog | `Opt+F1` |  | — | NOT_PLANNED |  | Image-Line's online assistant. |
| MENU-263 | Tutorials |  | submenu |  |  | — | NOT_STARTED | APP-032 |  |
| MENU-264 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Loop starter tutorial |  | action |  |  | — | NOT_STARTED | APP-032 |  |
| MENU-265 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Step sequencer tutorial |  | action |  |  | — | NOT_STARTED | APP-032 |  |
| MENU-266 | Unlock FL Studio | Registration | dialog |  |  | — | NOT_PLANNED |  |  |
| MENU-267 | My account | Registration | link |  |  | — | NOT_PLANNED |  |  |
| MENU-268 | Diagnostics | Support | dialog |  |  | — | NOT_STARTED | APP-031 |  |
| MENU-269 | What's new | Support | link |  |  | — | NOT_STARTED | APP-032 |  |
| MENU-270 | Buy FL Studio and upgrades | Shop | link |  |  | — | NOT_PLANNED |  |  |
| MENU-271 | FL Studio homepage | Internet links | link |  |  | — | NOT_PLANNED |  | Image-Line web page. LimeShot's counterpart would link the project's repository and docs. |
| MENU-272 | FL Studio Mobile | Internet links | link |  |  | — | NOT_PLANNED |  | Image-Line web page. LimeShot's counterpart would link the project's repository and docs. |
| MENU-273 | FL Studio tutorials | Internet links | link |  |  | — | NOT_PLANNED |  | Image-Line web page. LimeShot's counterpart would link the project's repository and docs. |
| MENU-274 | Support - Forum | Internet links | link |  |  | — | NOT_PLANNED |  | Image-Line web page. LimeShot's counterpart would link the project's repository and docs. |
| MENU-275 | Support - Knowledgebase | Internet links | link |  |  | — | NOT_PLANNED |  | Image-Line web page. LimeShot's counterpart would link the project's repository and docs. |
| MENU-276 | About... |  | dialog |  |  | Help › About LimeShot | DIFFERENT | APP-032 | Opens the settings window's About tab; LimeShot names itself in the label. |
| MENU-277 | Keyboard shortcuts |  | dialog |  |  | Help › Keyboard shortcuts | LIMESHOT_ONLY | APP-013 | FL has no shortcut list in Help; LimeShot opens it with F1 (FL's F1 is context help). |

## Toolbar

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-278 | Pattern or song mode | Transport | control | `L` |  | Toolbar › PAT / SONG | DIFFERENT | TR-004 | FL's hint: 'Pattern or song mode / Show channel rack or playlist' (a right-click shows the matching window). LimeShot switches the mode only. |
| MENU-279 | Play | Transport | control | `Space` |  | Toolbar › play button | IMPLEMENTED | TR-001 | LimeShot's button plays and pauses; Space stops, as in FL. |
| MENU-280 | Stop | Transport | control | `Space` |  | Toolbar › stop button | IMPLEMENTED | TR-001 |  |
| MENU-281 | Record (automation, score, audio, clips) | Transport | control | `R` |  | — | NOT_STARTED | REC-002 | Right-click opens the menu below. LimeShot shows the button disabled. |
| MENU-282 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Automation | Record menu › Recording filter | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-283 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Notes | Record menu › Recording filter | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-284 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Audio | Record menu › Recording filter | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-285 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Clips | Record menu › Recording filter | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-286 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Recording starts playback | Record menu › Options | toggle |  |  | — | NOT_STARTED | REC-002 |  |
| MENU-287 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Disarm on stop | Record menu › Options | toggle |  |  | — | NOT_STARTED | REC-002 |  |
| MENU-288 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Enable recording markers | Record menu › Options | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-289 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note start time | Record menu › Quantize | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-290 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note end time | Record menu › Quantize | toggle |  | checked | — | NOT_STARTED | REC-002 |  |
| MENU-291 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Leave note duration | Record menu › Quantize | toggle |  |  | — | NOT_STARTED | REC-002 |  |
| MENU-292 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Record to step sequencer | Record menu › Quantize | toggle |  |  | — | NOT_STARTED | REC-002 |  |
| MENU-293 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Automations | Record menu › Quantize | toggle |  |  | — | NOT_STARTED | REC-002 |  |
| MENU-294 | Tempo | Transport | control |  |  | Toolbar › tempo field | DIFFERENT | TR-002 | Drag or type, as in FL. FL's right-click menu has the control menu (Edit events…, Create automation clip, Link to controller…, Copy/Paste value, Type in value…) and tempo presets: Tempo tapper (Cmd+F5), 80 BPM, 90 BPM … in steps of 10 (partly hidden in the capture). LimeShot has no tempo menu or automation (AUT-008). |
| MENU-295 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tempo tapper | Tempo menu | dialog | `Cmd+F5` |  | — | NOT_STARTED | TR-012 |  |
| MENU-296 | &nbsp;&nbsp;&nbsp;&nbsp;↳ <BPM presets> | Tempo menu | list |  |  | — | NOT_STARTED | TR-002 | 80, 90, 100, 110, 120, 130, 140 BPM and more (cut off by a system prompt in the capture). |
| MENU-297 | Song position | Song position | display |  |  | Toolbar › song position | DIFFERENT | TR-008 | FL shows one format chosen with the right-click menu (Mode: Bar:Beat/Step:Tick, Minute:Second:Centisecond, SMPTE 23.976 … 120 fps; Middle unit: Step / Beat). LimeShot shows bar:beat:tick and minutes:seconds together; a click returns to the start. |
| MENU-298 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Mode | Song position menu | radio |  |  | — | NOT_STARTED | TR-008 | Bar : Beat/Step : Tick (selected), Minute : Second : Centisecond, SMPTE 23.976, 24, 25, 29.97 drop, 29.97 non-drop, 30, 48, 50, 59.94 drop/non-drop, 60, 72, 120 fps. |
| MENU-299 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Middle unit | Song position menu | radio |  |  | — | NOT_STARTED | TR-008 | Step (selected) / Beat. |
| MENU-300 | Typing keyboard to piano | Recording switches | toggle | `Cmd+T` |  | — | NOT_STARTED | WF-002 |  |
| MENU-301 | Recording precount | Recording switches | toggle | `Cmd+P` |  | — | NOT_STARTED | REC-007 | Shown as '3.2.1'. |
| MENU-302 | Loop record | Recording switches | toggle |  |  | — | NOT_STARTED | REC-005 |  |
| MENU-303 | Metronome | Recording switches | toggle | `Cmd+M` |  | Toolbar › Metronome | IMPLEMENTED | TR-006 |  |
| MENU-304 | Detected chord | Info | display |  |  | — | NOT_STARTED |  | Names the chord being played ('---' when none). |
| MENU-305 | Monitor (master) | Info | display |  |  | — | NOT_STARTED |  | Oscilloscope / spectrum of the master output; a click changes the view. |
| MENU-306 | Peak meter (master) | Info | display |  |  | Toolbar › Master output | IMPLEMENTED | MIX-001 |  |
| MENU-307 | CPU load | CPU | display |  |  | Toolbar › audio load readout | DIFFERENT | UI-004 | LimeShot shows the audio load once per second. FL's right-click menu: Refresh (Slow, Medium, Fast) and Misc (View voice count, View FPS); FL also shows memory use. |
| MENU-308 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Refresh | CPU menu | radio |  |  | — | NOT_STARTED |  | Slow (selected), Medium, Fast. |
| MENU-309 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View voice count | CPU menu | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-310 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View FPS | CPU menu | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-311 | CPU load history | CPU | display |  |  | — | NOT_STARTED |  | A graph of recent CPU load next to the readout. |
| MENU-312 | Hint panel | Hint panel | display |  |  | Toolbar › hint panel | IMPLEMENTED | UI-004 | Top left: what LimeShot is doing, then the control under the pointer and its shortcut. |
| MENU-313 | Browser / Plugin picker | Shortcut icons | toggle | `Opt+F8` |  | Toolbar › Browser | IMPLEMENTED | BR-001 | FL: a right-click shows the Plugin picker; LimeShot has no plugin picker. |
| MENU-314 | Channel rack | Shortcut icons | toggle | `F6` |  | Toolbar › Channel Rack | IMPLEMENTED | UI-002 |  |
| MENU-315 | Piano roll / Maximize Piano roll | Shortcut icons | toggle | `F7` |  | Toolbar › Piano Roll | DIFFERENT | UI-002 | FL: a right-click maximises it; LimeShot toggles it. |
| MENU-316 | Playlist / Maximize Play list | Shortcut icons | toggle | `F5` |  | Toolbar › Playlist | DIFFERENT | UI-002 | As Piano roll. |
| MENU-317 | Mixer | Shortcut icons | toggle | `F9` |  | Toolbar › Mixer | IMPLEMENTED | UI-002 |  |
| MENU-318 | Auto scrolling | Shortcut icons | toggle |  |  | — | NOT_STARTED | UI-016 |  |
| MENU-319 | One-click audio recording | Shortcut icons | control |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-320 | Main snap / quantization / step edit step | Snap | list | `BkSp` |  | Toolbar › snap selector | DIFFERENT | PR-005 | FL's list: Line, Cell, (none), 1/6 step, 1/4 step, 1/3 step, 1/2 step, Step, 1/6 beat, 1/4 beat, 1/3 beat, 1/2 beat, Beat, Bar; the hint also names BkSp. LimeShot offers Bar, Beat, 1/2 beat, Step, 1/2 step, 1/3 step, 1/4 step and None. |
| MENU-321 | Pattern options | Patterns | submenu |  |  | Toolbar › pattern selector | DIFFERENT | SEQ-021 | The ▸ left of the selector opens the Patterns menu (pattern list and commands). LimeShot's selector menu lists the patterns and Create new; the commands are in the main Patterns menu. |
| MENU-322 | Pattern selector | Patterns | list | `+ / -` |  | Toolbar › pattern selector | IMPLEMENTED | SEQ-018 | + and − step through the patterns. |
| MENU-323 | New pattern (find first / next empty) | Patterns | control |  |  | Toolbar › pattern selector | DIFFERENT | SEQ-021 | FL's + button; a left click finds the next empty pattern. LimeShot: Create new in the selector, F4 and Shift+F4. |
| MENU-324 | Shift key modifier | Modifiers | toggle |  |  | — | NOT_STARTED |  | On-screen modifier keys for touch screens. |
| MENU-325 | Command key modifier | Modifiers | toggle |  |  | — | NOT_STARTED |  | On-screen modifier keys for touch screens. |
| MENU-326 | Option key modifier | Modifiers | toggle |  |  | — | NOT_STARTED |  | On-screen modifier keys for touch screens. |
| MENU-327 | Gopher (chat assistant) | Online | link | `Opt+F1` |  | — | NOT_PLANNED |  | Image-Line service. |
| MENU-328 | Buy FL Studio and upgrades | Online | link |  |  | — | NOT_PLANNED |  | Image-Line shop. |
| MENU-329 | Online content & events | Online | display |  |  | — | NOT_PLANNED |  | Image-Line news panel. |
| MENU-330 | Time signature | LimeShot | control |  |  | Toolbar › Beats per bar / Beat unit (power of two) | LIMESHOT_ONLY |  | LimeShot shows the project's time signature next to the tempo; FL keeps it in Project settings. |
| MENU-331 | Undo, redo, save and export | LimeShot | control |  |  | Toolbar › undo, redo, save and export buttons | LIMESHOT_ONLY |  | LimeShot's own buttons on the right of the toolbar. |

## Lists

Static contents of the `list` items, as shipped with FL Studio.

### templates

* **Advanced**: Electronic/Breakbeat, Electronic/Chillout-Ambient, Electronic/Chillout, Electronic/Drum & Bass, Electronic/Dubstep, Electronic/EDM-House, Electronic/Electro, Electronic/HipHop-Trap, Electronic/Trap, Instrumental/Funk, Instrumental/Jazz, Instrumental/Metal, Instrumental/Rock
* **Minimal**: Basic with limiter, Basic, Delay and reverb, Empty with 4 sends, Empty
* **Other**: All MIDI CC, Blocks, Patch bank, Video chopping
* **Performance**: 4x4, 8x8, Touchscreen
* **Power User**: Cheese's recording setup
* **Utility**: Create a chord progression, Drum mix, Launch loop starter, Record Audio, Remix a song, SFX transitions, Surround mix, Vocoder

### generators

* **Controller**: Fruity Envelope Controller, Fruity Keyboard Controller
* **Drum**: BassDrum, Drumaxx, Drumpad, FPC, Fruit Kick, Fruity DrumSynth Live, Fruity Slicer, Fruity Slicer 2, Ogun, Slicex
* **MIDI**: MIDI Out
* **Misc**: Audio Clip, Automation Clip, BooBass, FL Keys, FL Studio Mobile, FL Studio Mobile Rack, Fruity Voltage Controller, Layer, SoundFont Player
* **Patcher**: Patcher, VFX Color Mapper, VFX Envelope, VFX Key Mapper, VFX Keyboard Splitter, VFX Level Scaler, VFX Script, VFX Sequencer
* **Sampler**: DirectWave, Fruity Granulizer, Sampler, Wave Traveller
* **Synth classic**: 3x Osc, FLEX, Fruity DX10, GMS, Harmless, Kepler, Kepler Exo, MiniSynth, PoiZone, Sawer, SimSynth, Sytrus, Toxic Biohazard, Transistor Bass
* **Synth special**: Autogun, BeepMap, Harmor, Morphine, Ogun, Plucked!, Sakura
* **Visual**: Fruity Dance, Fruity Video Player

### effects

* **Controller**: Control Surface, Fruity Formula Controller, Fruity Peak Controller, Fruity X-Y Controller, Fruity X-Y-Z Controller
* **Delay reverb**: Fruity Convolver, Fruity Delay 2, Fruity Delay 3, Fruity Delay Bank, Fruity Reeverb 2, LuxeVerb, Multiband Delay
* **Distortion**: Distructor, Fruity Blood Overdrive, Fruity Fast Dist, Fruity Soft Clipper, Fruity Squeeze, Fruity WaveShaper, Hardcore
* **Dynamics**: Emphasis, Emphasizer, Fruity Compressor, Fruity Limiter, Fruity Multiband Compressor, Fruity Soft Clipper, Maximus, Soundgoodizer, Transient Processor
* **Filter**: EQUO, Frequency Splitter, Fruity Filter, Fruity Love Philter, Fruity Parametric EQ, Fruity Parametric EQ 2
* **Flanger**: Fruity Chorus, Fruity Flanger, Fruity Flangus, Fruity Phaser, Hyper Chorus, Vintage Chorus, Vintage Phaser
* **Gain**: Fruity Balance, Fruity PanOMatic, Fruity Send, Fruity Stereo Enhancer, Fruity Stereo Shaper, Spreader
* **Misc**: Edison, Effector, FL Studio Mobile Rack FX, Frequency Shifter, Fruity Scratcher, Fruity Vocoder, Gross Beat, Low Lifter, Newtime, Newtone, Pitch Shifter, Pitcher, Transmitter, Transporter, Vocodex
* **Patcher**: Patcher, VFX Color Mapper, VFX Envelope, VFX Key Mapper, VFX Keyboard Splitter, VFX Level Scaler, VFX Script, VFX Sequencer
* **Visual**: Fruity Big Clock, Fruity dB Meter, Fruity HTML NoteBook, Fruity NoteBook, Fruity NoteBook 2, Fruity Spectroman, Tuner, Wave Candy, ZGameEditor Visualizer

### limeshot_equivalents

* **Sampler**: Sampler channel
* **Audio Clip**: audio clip channel (SEQ-019)
* **Automation Clip**: automation clip (PL-015)
* **3x Osc**: LimeSynth channel
* **Fruity Parametric EQ 2**: Lime EQ
* **Fruity Filter**: Lime Filter
* **Fruity Delay 3**: Lime Delay
* **Fruity Reeverb 2**: Lime Room
* **Fruity Compressor**: Lime Comp
* **Fruity Limiter**: Lime Limiter
* **Fruity Balance**: Lime Utility

### toolbar_presets

Compact, Default (hint above menu), Default (macOS), Default, FL Studio 12, FL Studio 24 (macOS), FL Studio 24 (Windows), FL Studio 25 (macOS), FL Studio 25 (Windows), Single line centered, Single line

### stretch_modes

* **Realtime**: Resample, Stretch, Stretch Pro
* **Elastique**: e3 generic, e3 mono
* **Special**: Slice stretch, Slice map, Auto
* **Legacy**: e2 generic, e2 transient, e2 mono, e2 speech

### declicking

Out only (no bleeding), Transient (no bleeding), Transient (bleeding), Generic (bleeding), Smooth (bleeding), Crossfade (bleeding)
