# Main menu bar, toolbar and editor menus: FL Studio and LimeShot

> Generated from [`menus.json`](menus.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313), macOS.
> Source: Observed live on 2026-10-22 (session 13): FL's in-window menu bar (FILE … HELP), cross-checked with the native macOS menu bar FL mirrors (Accessibility dump in docs/research/evidence/menus/). Screenshots: docs/research/evidence/screenshots/fl-menu-*.png. Toolbar (menu 'Toolbar') observed live on 2026-10-22 (session 15): each control's name and keys from FL's hint panel while hovering it (fl-toolbar-hints-*.png), and its right-click menus (fl-toolbar-*-menu.png, fl-toolbar-snap.png).

One record per menu item, in FL's order. path = FL's labels from the menu bar down (FL writes '...' for dialogs). section = the grey header the item sits under in-window. kind: action | dialog (opens a dialog or window) | submenu | toggle (checkmark) | radio | list (filled from data; static contents in `lists`) | link (opens a web page). fl_keys in macOS notation as FL shows them. fl_state = the state seen on a fresh default project (checked, selected, disabled). limeshot = where LimeShot has it (Menu › Item). status: IMPLEMENTED (same item), DIFFERENT (LimeShot has the action under another label or place, or without the key), LIMESHOT_ONLY (LimeShot menu item with no FL counterpart in that menu), NOT_STARTED, NOT_PLANNED (Image-Line services, FL's own plugins). Records added after the session 13 capture keep the order of FL's menus; LimeShot-only items follow the FL items of their menu. Toolbar records (menu 'Toolbar', session 15) use kind control (button, knob, field) or display (readout, meter) as well; their right-click menus are child paths. Editor menus (session 15) use the pseudo-menus 'Channel Rack', 'Piano Roll', 'Playlist' and 'Mixer'; LimeShot places in lower case ("channel menu") are descriptions, not checked labels.

Tracked: **725** — IMPLEMENTED 180, DIFFERENT 93, LIMESHOT_ONLY 13, NOT_STARTED 398, NOT_PLANNED 41

## Summary

| Menu | FL items | Implemented | Different | Not started | Not planned | LimeShot only |
|---|---|---|---|---|---|---|
| File | 45 | 4 | 7 | 31 | 3 | 0 |
| Edit | 4 | 4 | 0 | 0 | 0 | 0 |
| Add | 26 | 1 | 14 | 10 | 1 | 2 |
| Patterns | 23 | 10 | 5 | 8 | 0 | 0 |
| View | 48 | 16 | 1 | 26 | 5 | 4 |
| Options | 24 | 6 | 0 | 16 | 2 | 2 |
| Tools | 82 | 8 | 3 | 64 | 7 | 0 |
| Help | 16 | 0 | 1 | 6 | 9 | 1 |
| Toolbar | 52 | 16 | 7 | 26 | 3 | 2 |
| Channel Rack | 72 | 25 | 12 | 31 | 4 | 2 |
| Piano Roll | 137 | 44 | 20 | 70 | 3 | 0 |
| Playlist | 139 | 37 | 14 | 88 | 0 | 0 |
| Mixer | 44 | 9 | 9 | 22 | 4 | 0 |

## Open work by feature

Menu items LimeShot does not have yet (NOT_STARTED), grouped by the feature that implements them, highest priority first.

| Feature | Pri | Status | Name | Menu items |
|---|---|---|---|---|
| EXP-001 | P0 | VERIFIED | WAV export (offline bounce) | MENU-580 Playlist › Options › Tools › Export all playlist tracks |
| MIX-001 | P0 | VERIFIED | Master and insert tracks | MENU-708 Mixer › Options › Alt solo selected track(s) |
| SEQ-002 | P0 | VERIFIED | Add sampler / synth channels | MENU-383 Channel Rack › Channel menu › Insert<br>MENU-384 Channel Rack › Channel menu › Replace |
| APP-020 | P1 | NOT_STARTED | MIDI settings | MENU-153 Options › MIDI settings |
| AUT-003 | P1 | VERIFIED | Automate mixer and channel parameters | MENU-217 Tools › Last tweaked › Edit automation clip...<br>MENU-231 Tools › Next to last tweaked › Create automation clip<br>MENU-232 Tools › Next to last tweaked › Edit automation clip...<br>MENU-536 Piano Roll › Options › Target control › Channel panning<br>MENU-537 Piano Roll › Options › Target control › Channel volume<br>MENU-538 Piano Roll › Options › Target control › Channel pitch |
| BR-005 | P1 | IMPLEMENTED | Current project tree | MENU-086 Patterns › Open in project browser<br>MENU-106 View › Project picker<br>MENU-141 View › Patterns<br>MENU-142 View › Generators in use<br>MENU-143 View › Effects in use |
| EXP-005 | P1 | NOT_STARTED | MP3 export | MENU-023 File › Export › MP3 file... |
| EXP-012 | P1 | VERIFIED | MIDI export | MENU-243 Tools › Prepare for MIDI export |
| PL-014 | P1 | VERIFIED | Audio clips | MENU-572 Playlist › Options › Edit › Reverse selected audio clips |
| PL-015 | P1 | VERIFIED | Automation clips | MENU-568 Playlist › Options › Edit › Merge automation clips |
| PL-016 | P1 | VERIFIED | Markers and loop region / time selection | MENU-635 Playlist › Options › Time markers › Add two...<br>MENU-636 Playlist › Options › Time markers › Add one every<br>MENU-637 Playlist › Options › Time markers › Add jump to next bar<br>MENU-639 Playlist › Options › Time markers › Add key marker<br>MENU-640 Playlist › Options › Time markers › Place loop<br>MENU-642 Playlist › Options › Time markers › Change to marker loop |
| REC-001 | P1 | NOT_STARTED | Audio recording from inputs | MENU-180 Tools › One-click audio recording<br>MENU-182 Tools › One-click audio recording › Into playlist as an audio clip<br>MENU-241 Tools › Cancel recording<br>MENU-319 Toolbar › One-click audio recording<br>MENU-683 Mixer › Options › Disk recording<br>MENU-709 Mixer › Options › Arm selected track(s)<br>MENU-710 Mixer › Options › Disarm selected track(s) |
| REC-002 | P1 | NOT_STARTED | MIDI recording | MENU-281 Toolbar › Record (automation, score, audio, clips)<br>MENU-282 Toolbar › Record (automation, score, audio, clips) › Recording filter › Automation<br>MENU-283 Toolbar › Record (automation, score, audio, clips) › Recording filter › Notes<br>MENU-284 Toolbar › Record (automation, score, audio, clips) › Recording filter › Audio<br>MENU-285 Toolbar › Record (automation, score, audio, clips) › Recording filter › Clips<br>MENU-286 Toolbar › Record (automation, score, audio, clips) › Options › Recording starts playback<br>MENU-287 Toolbar › Record (automation, score, audio, clips) › Options › Disarm on stop<br>MENU-288 Toolbar › Record (automation, score, audio, clips) › Options › Enable recording markers<br>MENU-289 Toolbar › Record (automation, score, audio, clips) › Quantize › Note start time<br>MENU-290 Toolbar › Record (automation, score, audio, clips) › Quantize › Note end time<br>MENU-291 Toolbar › Record (automation, score, audio, clips) › Quantize › Leave note duration<br>MENU-292 Toolbar › Record (automation, score, audio, clips) › Quantize › Record to step sequencer<br>MENU-293 Toolbar › Record (automation, score, audio, clips) › Quantize › Automations |
| SEQ-008 | P1 | VERIFIED | Piano-roll preview in rack | MENU-367 Channel Rack › Options › Show a mute button on the piano roll preview |
| SEQ-010 | P1 | VERIFIED | Swing (global and per channel) | MENU-348 Channel Rack › Options › Set swing mix for selected<br>MENU-349 Channel Rack › Options › Set truncate swing notes for selected |
| TR-008 | P1 | VERIFIED | Metronome | MENU-299 Toolbar › Song position › Middle unit |
| UI-004 | P1 | VERIFIED | Hint bar and tooltips | MENU-117 View › Toolbars › Hint bar |
| UI-010 | P1 | IN_PROGRESS | Control context menu | MENU-220 Tools › Last tweaked › Copy value<br>MENU-221 Tools › Last tweaked › Paste value<br>MENU-222 Tools › Last tweaked › Type in value...<br>MENU-223 Tools › Last tweaked › Set...<br>MENU-235 Tools › Next to last tweaked › Copy value<br>MENU-236 Tools › Next to last tweaked › Paste value<br>MENU-237 Tools › Next to last tweaked › Type in value...<br>MENU-238 Tools › Next to last tweaked › Set... |
| WF-003 | P1 | NOT_STARTED | MIDI controller input | MENU-163 Options › Enable MIDI remote control<br>MENU-399 Channel Rack › Channel menu › MIDI channel through<br>MENU-400 Channel Rack › Channel menu › Receive notes from |
| APP-015 | P2 | NOT_STARTED | Project templates | MENU-002 File › New from template<br>MENU-003 File › New from template › Advanced<br>MENU-004 File › New from template › Minimal<br>MENU-005 File › New from template › Other<br>MENU-006 File › New from template › Performance<br>MENU-007 File › New from template › Power User<br>MENU-008 File › New from template › Utility<br>MENU-009 File › New from template › Empty<br>MENU-010 File › New from template › Change default template...<br>MENU-017 File › Save as template... |
| APP-018 | P2 | NOT_STARTED | Save new version | MENU-014 File › Save new version |
| APP-019 | P2 | NOT_STARTED | Project bundle (zip with samples) | MENU-040 File › Export › Zipped loop package... |
| APP-021 | P2 | NOT_STARTED | Plugin manager | MENU-072 Add › Manage plugins<br>MENU-158 Options › Manage plugins<br>MENU-725 Mixer › Effect slot menu › Plugin database |
| APP-027 | P2 | NOT_STARTED | Revert to last backup | MENU-044 File › Revert to last backup |
| APP-032 | P2 | IN_PROGRESS | Help index, tutorials and about | MENU-261 Help › Help index<br>MENU-263 Help › Tutorials<br>MENU-264 Help › Tutorials › Loop starter tutorial<br>MENU-265 Help › Tutorials › Step sequencer tutorial<br>MENU-269 Help › What's new |
| AUT-005 | P2 | NOT_STARTED | Event editor | MENU-211 Tools › Last tweaked › Edit events<br>MENU-212 Tools › Last tweaked › Edit events in new window<br>MENU-213 Tools › Last tweaked › Edit events in piano roll<br>MENU-226 Tools › Next to last tweaked › Edit events<br>MENU-227 Tools › Next to last tweaked › Edit events in new window<br>MENU-228 Tools › Next to last tweaked › Edit events in piano roll |
| BR-008 | P2 | NOT_STARTED | Plugin and preset browser | MENU-060 Add › Categories<br>MENU-061 Add › Simple<br>MENU-062 Add › Tree<br>MENU-064 Add › Effect › More plugins...<br>MENU-065 Add › Effect › Plugin database<br>MENU-068 Add › Browse plugin database<br>MENU-069 Add › Browse all installed plugins<br>MENU-145 View › Plugin database |
| EXP-006 | P2 | NOT_STARTED | FLAC / OGG export | MENU-024 File › Export › OGG file...<br>MENU-025 File › Export › FLAC file...<br>MENU-027 File › Export › M4A file... |
| EXP-014 | P2 | NOT_STARTED | Export playlist tracks or track groups | MENU-031 File › Export › All playlist tracks<br>MENU-032 File › Export › All playlist tracks › From song start...<br>MENU-033 File › Export › All playlist tracks › From track start...<br>MENU-034 File › Export › All playlist tracks › Time selection...<br>MENU-035 File › Export › All playlist track groups<br>MENU-036 File › Export › All playlist track groups › From song start...<br>MENU-037 File › Export › All playlist track groups › From track start...<br>MENU-038 File › Export › All playlist track groups › Time selection... |
| EXP-015 | P2 | NOT_STARTED | Export in the last used format(s) | MENU-028 File › Export › Last exported format(s)... |
| PL-018 | P2 | NOT_STARTED | Track grouping | MENU-581 Playlist › Options › Tools › Export all playlist track groups<br>MENU-594 Playlist › Options › View › Hide collapsed grouped tracks<br>MENU-621 Playlist › Options › Group › Group<br>MENU-622 Playlist › Options › Group › Ungroup<br>MENU-678 Playlist › Options › Collapse all grouped tracks<br>MENU-679 Playlist › Options › Expand all grouped tracks |
| PL-022 | P2 | IN_PROGRESS | Consolidate / render to audio clip | MENU-098 Patterns › Quick render as audio clip<br>MENU-099 Patterns › Render as audio clip...<br>MENU-100 Patterns › Render and replace...<br>MENU-567 Playlist › Options › Edit › Merge similar pattern clips<br>MENU-579 Playlist › Options › Tools › Consolidate playlist selection |
| PL-023 | P2 | NOT_STARTED | Crossfades and time stretching | MENU-250 Tools › Switch all audio clip channels to<br>MENU-251 Tools › Switch selected audio clip channels to<br>MENU-570 Playlist › Options › Edit › Pitch selected audio clips one semitone down<br>MENU-571 Playlist › Options › Edit › Pitch selected audio clips one semitone up<br>MENU-573 Playlist › Options › Edit › Automatic crossfades (current project)<br>MENU-574 Playlist › Options › Edit › Create new clips with manual fades<br>MENU-575 Playlist › Options › Edit › Snap fade handles<br>MENU-576 Playlist › Options › Edit › Create variants when stretching audio clips |
| PLG-008 | P2 | IMPLEMENTED | Plugin preset management | MENU-070 Add › Browse presets |
| PR-013 | P2 | NOT_STARTED | Scale highlighting / snap to scale | MENU-481 Piano Roll › Options › View › Scale highlighting |
| PR-015 | P2 | IMPLEMENTED | Chords, arpeggiator, strum, humanize, legato | MENU-458 Piano Roll › Options › Tools › Claw machine... |
| PR-016 | P2 | NOT_STARTED | Note properties: pan, release, mod X/Y, fine pitch | MENU-528 Piano Roll › Options › Target control<br>MENU-529 Piano Roll › Options › Target control › Note pan<br>MENU-531 Piano Roll › Options › Target control › Note release<br>MENU-532 Piano Roll › Options › Target control › Note filter cutoff frequency<br>MENU-533 Piano Roll › Options › Target control › Note filter resonance (Q)<br>MENU-534 Piano Roll › Options › Target control › Note fine pitch<br>MENU-535 Piano Roll › Options › Target control › Note repeat |
| PR-027 | P2 | NOT_STARTED | Stamp tool (chords and scales) | MENU-467 Piano Roll › Options › Stamp |
| REC-005 | P2 | NOT_STARTED | Loop recording / takes | MENU-172 Options › Loop record<br>MENU-302 Toolbar › Loop record |
| REC-007 | P2 | NOT_STARTED | Count-in | MENU-168 Options › Recording precount<br>MENU-301 Toolbar › Recording precount |
| REC-010 | P2 | NOT_STARTED | Start on input and blend recorded notes | MENU-169 Options › Start on input<br>MENU-170 Options › Blend recorded notes |
| SEQ-012 | P2 | VERIFIED | Channel filter groups | MENU-359 Channel Rack › Options › Auto switch display filter |
| SEQ-014 | P2 | VERIFIED | Fill / rotate / advanced fill | MENU-395 Channel Rack › Channel menu › Advanced fill...<br>MENU-396 Channel Rack › Channel menu › Auto fill |
| UI-013 | P2 | IMPLEMENTED | Window layout commands | MENU-131 View › Arrange windows › Desktop (alternate)<br>MENU-132 View › Arrange windows › Tablet (default)<br>MENU-133 View › Arrange windows › Tablet (alternate) |
| UI-016 | P2 | NOT_STARTED | Auto scrolling option | MENU-173 Options › Auto scrolling<br>MENU-318 Toolbar › Auto scrolling |
| WF-002 | P2 | NOT_STARTED | Typing keyboard to piano | MENU-166 Options › Typing keyboard to piano<br>MENU-300 Toolbar › Typing keyboard to piano |
| WF-004 | P2 | NOT_STARTED | Controller mapping (link to controller) | MENU-144 View › Remote control<br>MENU-175 Options › Multilink to controllers<br>MENU-218 Tools › Last tweaked › Link to controller...<br>MENU-219 Tools › Last tweaked › Override global link...<br>MENU-224 Tools › Last tweaked › Override volatile link...<br>MENU-233 Tools › Next to last tweaked › Link to controller...<br>MENU-234 Tools › Next to last tweaked › Override global link...<br>MENU-239 Tools › Next to last tweaked › Override volatile link...<br>MENU-685 Mixer › Options › Link all parameters... |
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
| PL-019 | P3 | NOT_STARTED | Performance mode | MENU-242 Tools › Prepare for performance mode<br>MENU-600 Playlist › Options › View › Performance clip progress<br>MENU-601 Playlist › Options › View › Performance track progress<br>MENU-631 Playlist › Options › Zoom › Zoom on performance clips<br>MENU-670 Playlist › Options › Prepare for performance mode<br>MENU-671 Playlist › Options › Performance mode |
| PL-030 | P3 | DEFERRED | Stem separation (Remix a song) | MENU-183 Tools › Remix a song... |
| PLG-011 | P3 | NOT_STARTED | Plugin performance monitor | MENU-121 View › Plugin performance monitor |
| PR-018 | P3 | NOT_STARTED | Note color groups | MENU-433 Piano Roll › Options › Edit › Change color<br>MENU-476 Piano Roll › Options › View › Note colors<br>MENU-497 Piano Roll › Options › Select › Select by color |
| PR-022 | P3 | NOT_STARTED | Time signature per pattern / time markers | MENU-090 Patterns › Set time signature...<br>MENU-519 Piano Roll › Options › Time markers<br>MENU-520 Piano Roll › Options › Time markers › Add one...<br>MENU-521 Piano Roll › Options › Time markers › Add auto<br>MENU-522 Piano Roll › Options › Time markers › Add two...<br>MENU-523 Piano Roll › Options › Time markers › Add one every<br>MENU-524 Piano Roll › Options › Time markers › Add time signature change...<br>MENU-525 Piano Roll › Options › Time markers › Add key marker<br>MENU-526 Piano Roll › Options › Time markers › Delete<br>MENU-638 Playlist › Options › Time markers › Add time signature change... |
| PR-025 | P3 | NOT_STARTED | Waveform helper background | MENU-486 Piano Roll › Options › View › Background waveform |
| PR-028 | P3 | NOT_STARTED | Chord progression tool | MENU-185 Tools › Chord progression tool...<br>MENU-446 Piano Roll › Options › Tools › Generate chord progression... |
| PR-029 | P3 | NOT_STARTED | Riff machine | MENU-184 Tools › Riff machine...<br>MENU-445 Piano Roll › Options › Tools › Riff machine... |
| REC-009 | P3 | NOT_STARTED | Step edit mode | MENU-171 Options › Step edit |
| REC-011 | P3 | NOT_STARTED | Audio logger | MENU-201 Tools › Enabled<br>MENU-202 Tools › Settings<br>MENU-203 Tools › Settings › 1 minute<br>MENU-204 Tools › Settings › 2 minutes<br>MENU-205 Tools › Settings › 5 minutes<br>MENU-206 Tools › Settings › 10 minutes<br>MENU-207 Tools › Settings › Pre effects<br>MENU-208 Tools › Settings › Post effects, pre fader<br>MENU-209 Tools › Dump to audio editor |
| SEQ-013 | P3 | NOT_STARTED | Zip / unzip channels | MENU-354 Channel Rack › Options › Zip selected<br>MENU-355 Channel Rack › Options › Unzip all |
| SEQ-022 | P3 | NOT_STARTED | Pattern groups | MENU-087 Patterns › Group selected...<br>MENU-088 Patterns › Rename group...<br>MENU-089 Patterns › Ungroup selected... |
| UI-006 | P3 | NOT_STARTED | Additional themes (light, high contrast) | MENU-157 Options › Theme settings |
| UI-014 | P3 | NOT_STARTED | Toolbar customisation and presets | MENU-111 View › Toolbars<br>MENU-112 View › Toolbars › Edit<br>MENU-113 View › Toolbars › Separators<br>MENU-114 View › Toolbars › Flat buttons<br>MENU-116 View › Toolbars › Show detected MIDI chords near the hint bar<br>MENU-118 View › Toolbars › Hide<br>MENU-119 View › Toolbars › <factory presets> |
| UI-015 | P3 | NOT_STARTED | Workspace background | MENU-136 View › Background<br>MENU-137 View › Background › Set image wallpaper...<br>MENU-138 View › Background › Set dynamic wallpaper<br>MENU-139 View › Background › Set background color... |
| UI-017 | P3 | DEFERRED | Control creator | MENU-186 Tools › Control creator |
| UI-018 | P3 | NOT_STARTED | Plugin picker | MENU-067 Add › Plugin picker<br>MENU-107 View › Plugin picker |
| WF-008 | P3 | NOT_STARTED | Python-style scripting API | MENU-110 View › Script output |
| WF-009 | P3 | IN_PROGRESS | Project macros | MENU-246 Tools › Switch smart disable for all plugins<br>MENU-247 Tools › Lock all playlist tracks to content<br>MENU-248 Tools › Unlock all playlist tracks from content<br>MENU-254 Tools › Switch all plugin editors to<br>MENU-255 Tools › Switch all plugin editors to › Attached<br>MENU-256 Tools › Switch all plugin editors to › Detached<br>MENU-260 Tools › Change declicking for selected channels to<br>MENU-353 Channel Rack › Options › Change declicking for selected channels to<br>MENU-369 Channel Rack › Options › Switch smart disable for all plugins<br>MENU-370 Channel Rack › Options › Switch all audio clip channels to<br>MENU-371 Channel Rack › Options › Switch selected audio clip channels to<br>MENU-372 Channel Rack › Options › Switch all plugin editors to<br>MENU-672 Playlist › Options › Change declicking for selected channels to<br>MENU-673 Playlist › Options › Switch all audio clip channels to<br>MENU-674 Playlist › Options › Switch selected audio clip channels to<br>MENU-675 Playlist › Options › Lock all playlist tracks to content<br>MENU-676 Playlist › Options › Unlock all playlist tracks from content<br>MENU-689 Mixer › Options › Switch smart disable for all plugins |
| WF-011 | P3 | NOT_STARTED | Touch controller | MENU-109 View › Touch controller |
| WF-012 | P3 | NOT_STARTED | MIDI clock sync | MENU-165 Options › Enable MIDI master sync |
| WF-013 | P3 | NOT_STARTED | Item groups | MENU-174 Options › Enable groups |
| — |  |  |  | MENU-304 Toolbar › Detected chord<br>MENU-305 Toolbar › Monitor (master)<br>MENU-310 Toolbar › CPU load › View FPS<br>MENU-311 Toolbar › CPU load history<br>MENU-324 Toolbar › Shift key modifier<br>MENU-325 Toolbar › Command key modifier<br>MENU-326 Toolbar › Option key modifier<br>MENU-339 Channel Rack › Options › Create layer with selected channels<br>MENU-350 Channel Rack › Options › Burn selected loops to pattern<br>MENU-356 Channel Rack › Options › Restretch all<br>MENU-358 Channel Rack › Options › Auto resize<br>MENU-360 Channel Rack › Options › Channel button width<br>MENU-361 Channel Rack › Options › Channel name alignment<br>MENU-362 Channel Rack › Options › Colorful channel controls<br>MENU-363 Channel Rack › Options › Focus selected channel on playback<br>MENU-364 Channel Rack › Options › Mute removed steps<br>MENU-379 Channel Rack › Channel menu › Change icon...<br>MENU-381 Channel Rack › Channel menu › Cut itself<br>MENU-382 Channel Rack › Channel menu › Show waveform preview<br>MENU-388 Channel Rack › Channel menu › Assign to new instrument track<br>MENU-402 Channel Rack › Channel menu › Burn MIDI to<br>MENU-407 Piano Roll › Options › File<br>MENU-408 Piano Roll › Options › File › Open score...<br>MENU-409 Piano Roll › Options › File › Browse scores<br>MENU-410 Piano Roll › Options › File › Save score as...<br>MENU-413 Piano Roll › Options › File › Paste from MIDI clipboard...<br>MENU-414 Piano Roll › Options › File › Copy to MIDI clipboard...<br>MENU-415 Piano Roll › Options › File › Export as score sheet...<br>MENU-432 Piano Roll › Options › Edit › Nudge notes with mouse wheel<br>MENU-436 Piano Roll › Options › Edit › Insert space<br>MENU-437 Piano Roll › Options › Edit › Slice & insert space<br>MENU-438 Piano Roll › Options › Edit › Delete space<br>MENU-439 Piano Roll › Options › Edit › Trim selection<br>MENU-440 Piano Roll › Options › Edit › Insert current controller value<br>MENU-441 Piano Roll › Options › Edit › Turn into automation clip...<br>MENU-443 Piano Roll › Options › Edit › Select tracks when samples are dropped<br>MENU-468 Piano Roll › Options › View<br>MENU-469 Piano Roll › Options › View › Grid color...<br>MENU-470 Piano Roll › Options › View › Grid contrast<br>MENU-471 Piano Roll › Options › View › Invert grid<br>MENU-472 Piano Roll › Options › View › Time segments<br>MENU-473 Piano Roll › Options › View › Keep labels on screen<br>MENU-474 Piano Roll › Options › View › Shadow<br>MENU-475 Piano Roll › Options › View › Rounded<br>MENU-477 Piano Roll › Options › View › Incremental scrolling<br>MENU-478 Piano Roll › Options › View › Precise time indicator<br>MENU-479 Piano Roll › Options › View › Mini piano roll preview<br>MENU-480 Piano Roll › Options › View › Note grid highlights<br>MENU-482 Piano Roll › Options › View › Show chord name for selection<br>MENU-483 Piano Roll › Options › View › Note length in note properties<br>MENU-485 Piano Roll › Options › View › Editable ghosts<br>MENU-487 Piano Roll › Options › View › Keyboard options<br>MENU-488 Piano Roll › Options › View › Swap panels<br>MENU-489 Piano Roll › Options › View › Flip separator<br>MENU-490 Piano Roll › Options › View › Remove background picture<br>MENU-503 Piano Roll › Options › Select › Select time around selection<br>MENU-504 Piano Roll › Options › Select › Select previous time<br>MENU-505 Piano Roll › Options › Select › Select next time<br>MENU-506 Piano Roll › Options › Select › Magic lasso<br>MENU-507 Piano Roll › Options › Group<br>MENU-508 Piano Roll › Options › Group › Group<br>MENU-509 Piano Roll › Options › Group › Ungroup<br>MENU-539 Piano Roll › Options › Auto smoothing<br>MENU-540 Piano Roll › Options › Preview notes during playback<br>MENU-555 Playlist › Options › Edit › Make unique as sample<br>MENU-557 Playlist › Options › Edit › Nudge clips with mouse wheel<br>MENU-558 Playlist › Options › Edit › Dropping on track header locks to content<br>MENU-559 Playlist › Options › Edit › Drop audio on track headers<br>MENU-565 Playlist › Options › Edit › Trim selection<br>MENU-582 Playlist › Options › Tools › Clone selected playlist track(s)<br>MENU-583 Playlist › Options › View<br>MENU-584 Playlist › Options › View › Grid color...<br>MENU-585 Playlist › Options › View › Grid contrast<br>MENU-586 Playlist › Options › View › Invert grid<br>MENU-587 Playlist › Options › View › Track separators<br>MENU-588 Playlist › Options › View › Time segments<br>MENU-589 Playlist › Options › View › Keep labels on screen<br>MENU-590 Playlist › Options › View › Content in titlebars<br>MENU-591 Playlist › Options › View › Behind clips<br>MENU-592 Playlist › Options › View › Shadow<br>MENU-595 Playlist › Options › View › Show fade previews<br>MENU-596 Playlist › Options › View › Show gain value<br>MENU-597 Playlist › Options › View › Show gain scale<br>MENU-598 Playlist › Options › View › Incremental scrolling<br>MENU-599 Playlist › Options › View › Precise time indicator<br>MENU-602 Playlist › Options › View › Mini playlist preview<br>MENU-603 Playlist › Options › View › Show controls on audio tracks<br>MENU-604 Playlist › Options › View › Show levels on audio tracks<br>MENU-605 Playlist › Options › View › Show levels on instrument tracks<br>MENU-606 Playlist › Options › View › Remove background picture<br>MENU-617 Playlist › Options › Select › Select previous time<br>MENU-618 Playlist › Options › Select › Select next time<br>MENU-619 Playlist › Options › Select › Magic lasso<br>MENU-620 Playlist › Options › Group<br>MENU-643 Playlist › Options › Picker panel<br>MENU-645 Playlist › Options › Picker panel › Show on right side<br>MENU-646 Playlist › Options › Picker panel › Width<br>MENU-647 Playlist › Options › Picker panel › Select linked channels<br>MENU-648 Playlist › Options › Picker panel › Select from playlist/channel rack<br>MENU-649 Playlist › Options › Picker panel › Show empty patterns<br>MENU-650 Playlist › Options › Picker panel › Drag to rearrange<br>MENU-651 Playlist › Options › Picker panel › Auto group patterns/channels<br>MENU-652 Playlist › Options › Picker panel › Select unused<br>MENU-653 Playlist › Options › Picker panel › Sort<br>MENU-654 Playlist › Options › Current clip source<br>MENU-656 Playlist › Options › Edit patterns in<br>MENU-657 Playlist › Options › Edit patterns in › Automatic<br>MENU-658 Playlist › Options › Edit patterns in › Channel rack<br>MENU-660 Playlist › Options › Edit patterns in › Piano roll or event editor<br>MENU-661 Playlist › Options › Edit patterns in › Scroll to position<br>MENU-662 Playlist › Options › Quantize song jumps<br>MENU-663 Playlist › Options › Quantize song jumps › Off<br>MENU-664 Playlist › Options › Quantize song jumps › Beats<br>MENU-665 Playlist › Options › Quantize song jumps › Bars<br>MENU-666 Playlist › Options › Quantize song jumps › 2 bars<br>MENU-667 Playlist › Options › Quantize song jumps › 4 bars<br>MENU-668 Playlist › Options › Quantize song jumps › 8 bars<br>MENU-669 Playlist › Options › Automation clips reset remote controls<br>MENU-686 Mixer › Options › View<br>MENU-688 Mixer › Options › Trim unused tracks<br>MENU-694 Mixer › Options › Gradient color...<br>MENU-695 Mixer › Options › Change icon...<br>MENU-696 Mixer › Options › Assign to new Audio track(s)<br>MENU-697 Mixer › Options › Unassign from Audio track(s)<br>MENU-698 Mixer › Options › File<br>MENU-704 Mixer › Options › Select<br>MENU-712 Mixer › Options › Separator<br>MENU-713 Mixer › Options › Create group...<br>MENU-714 Mixer › Options › Auto color group<br>MENU-715 Mixer › Options › Dock to<br>MENU-720 Mixer › Options › Insert multiple...<br>MENU-721 Mixer › Options › Insert from state<br>MENU-722 Mixer › Options › Allow threaded processing |

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
| MENU-108 | Tempo tapper | Windows | toggle | `Cmd+F5` |  | View › Tempo tapper | IMPLEMENTED | TR-012 | Opens the tempo tapper window (Cmd+F5). |
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
| MENU-278 | Pattern or song mode | Transport | control | `L` |  | Toolbar › PAT / SONG | IMPLEMENTED | TR-004 | A click switches the mode; a right-click shows the Channel Rack (pattern mode) or the Playlist (song mode), as FL's hint says. |
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
| MENU-294 | Tempo | Transport | control |  |  | Toolbar › tempo field | DIFFERENT | TR-002 | Drag or type, as in FL. Right-click: Tempo tapper and BPM presets 80–180 (LimeShot). FL's menu also has the control menu (Edit events…, Create automation clip, Link to controller…, Copy/Paste value, Type in value…); LimeShot has no tempo automation (AUT-008). |
| MENU-295 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tempo tapper | Tempo menu | dialog | `Cmd+F5` |  | Toolbar › Tempo tapper | IMPLEMENTED | TR-012 | Also View › Tempo tapper and Cmd+F5. Tapping sets the project tempo from the last taps (one undo step while the window is open); a two-second pause starts over. |
| MENU-296 | &nbsp;&nbsp;&nbsp;&nbsp;↳ <BPM presets> | Tempo menu | list |  |  | Toolbar › tempo menu presets | IMPLEMENTED | TR-002 | LimeShot: 80 to 180 BPM in steps of 10; FL's list continues past the capture. |
| MENU-297 | Song position | Song position | display |  |  | Toolbar › song position | DIFFERENT | TR-008 | A right-click chooses which format is large (Bar : Beat : Tick or Minute : Second : Centisecond); the other is shown small below, as before. A click returns to the start. No SMPTE. |
| MENU-298 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Mode | Song position menu | radio |  |  | Toolbar › Bar : Beat : Tick / Minute : Second : Centisecond | DIFFERENT | TR-008 | LimeShot has two of FL's modes; no SMPTE. |
| MENU-299 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Middle unit | Song position menu | radio |  |  | — | NOT_STARTED | TR-008 | Step (selected) / Beat. |
| MENU-300 | Typing keyboard to piano | Recording switches | toggle | `Cmd+T` |  | — | NOT_STARTED | WF-002 |  |
| MENU-301 | Recording precount | Recording switches | toggle | `Cmd+P` |  | — | NOT_STARTED | REC-007 | Shown as '3.2.1'. |
| MENU-302 | Loop record | Recording switches | toggle |  |  | — | NOT_STARTED | REC-005 |  |
| MENU-303 | Metronome | Recording switches | toggle | `Cmd+M` |  | Toolbar › Metronome | IMPLEMENTED | TR-006 |  |
| MENU-304 | Detected chord | Info | display |  |  | — | NOT_STARTED |  | Names the chord being played ('---' when none). |
| MENU-305 | Monitor (master) | Info | display |  |  | — | NOT_STARTED |  | Oscilloscope / spectrum of the master output; a click changes the view. |
| MENU-306 | Peak meter (master) | Info | display |  |  | Toolbar › Master output | IMPLEMENTED | MIX-001 |  |
| MENU-307 | CPU load | CPU | display |  |  | Toolbar › audio load readout | DIFFERENT | UI-004 | LimeShot shows the audio load; right-click: Refresh Slow/Medium/Fast and View voice count, as in FL. No memory readout or View FPS. |
| MENU-308 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Refresh | CPU menu | radio |  |  | Toolbar › CPU menu › Slow / Medium / Fast | IMPLEMENTED |  | Every 1, 0.5 or 0.2 seconds; the readout shows that interval's peak. |
| MENU-309 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View voice count | CPU menu | toggle |  |  | Toolbar › CPU menu › View voice count | IMPLEMENTED |  | The panel's caption shows the sounding voices instead of 'CPU'. |
| MENU-310 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View FPS | CPU menu | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-311 | CPU load history | CPU | display |  |  | — | NOT_STARTED |  | A graph of recent CPU load next to the readout. |
| MENU-312 | Hint panel | Hint panel | display |  |  | Toolbar › hint panel | IMPLEMENTED | UI-004 | Top left: what LimeShot is doing, then the control under the pointer and its shortcut. |
| MENU-313 | Browser / Plugin picker | Shortcut icons | toggle | `Opt+F8` |  | Toolbar › Browser | IMPLEMENTED | BR-001 | FL: a right-click shows the Plugin picker; LimeShot has no plugin picker. |
| MENU-314 | Channel rack | Shortcut icons | toggle | `F6` |  | Toolbar › Channel Rack | IMPLEMENTED | UI-002 |  |
| MENU-315 | Piano roll / Maximize Piano roll | Shortcut icons | toggle | `F7` |  | Toolbar › Piano Roll | IMPLEMENTED | UI-002 | A click toggles it; a right-click shows it maximised, as in FL. |
| MENU-316 | Playlist / Maximize Play list | Shortcut icons | toggle | `F5` |  | Toolbar › Playlist | IMPLEMENTED | UI-002 | As Piano roll. |
| MENU-317 | Mixer | Shortcut icons | toggle | `F9` |  | Toolbar › Mixer | IMPLEMENTED | UI-002 |  |
| MENU-318 | Auto scrolling | Shortcut icons | toggle |  |  | — | NOT_STARTED | UI-016 |  |
| MENU-319 | One-click audio recording | Shortcut icons | control |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-320 | Main snap / quantization / step edit step | Snap | list | `BkSp` |  | Toolbar › snap selector | DIFFERENT | PR-005 | LimeShot has FL's list in FL's order: (none), 1/6, 1/4, 1/3, 1/2 step, Step, 1/6, 1/4, 1/3, 1/2 beat, Beat, Bar; not FL's Line and Cell (the Piano Roll's own grid modes) or the BkSp key. |
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

## Channel Rack

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-332 | Options |  | submenu |  |  | Channel Rack › ⏵ | IMPLEMENTED | SEQ-007 | The ▸ at the rack's top left. LimeShot has no rack-wide menu; per-channel commands are on the channel button's right-click and in Tools. Session 16: LimeShot's ⏵ at the left of the Channel Rack header, with the commands on the selected channels. |
| MENU-333 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Add one |  | submenu |  |  | Channel Rack › ⏵ › Add one | IMPLEMENTED | SEQ-002 | FL lists its generators; LimeShot's + Add channel offers Sampler, LimeSynth and plugins. Session 16: Sampler, LimeSynth and plugin instruments. |
| MENU-334 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Loop starter |  | toggle |  |  | — | NOT_PLANNED | SEQ-017 | FL Cloud loops. |
| MENU-335 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Purge unused audio clips |  | action |  |  | Tools › Purge unused audio clips | DIFFERENT | WF-009 | Same macro, in the Tools menu. |
| MENU-336 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Select unused channels | Channels | action |  |  | Tools › Select unused channels | DIFFERENT | WF-009 | Same macro, in the Tools menu. |
| MENU-337 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Clone selected | Channels | action | `Opt+C` |  | Channel Rack › ⏵ › Clone selected | IMPLEMENTED | SEQ-007 | LimeShot clones one channel from its menu; no key. |
| MENU-338 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Delete selected... | Channels | dialog | `Opt+Del` |  | Channel Rack › ⏵ › Delete selected… | IMPLEMENTED | SEQ-007 | Alt+Delete deletes the selected channels (asks first, as FL). |
| MENU-339 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Create layer with selected channels | Channels | action |  |  | — | NOT_STARTED |  |  |
| MENU-340 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Move selected up | Channels | action | `Opt+Up` |  | Channel Rack › ⏵ › Move selected up | IMPLEMENTED | SEQ-007 | Alt+↑ moves the selected channels. |
| MENU-341 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Move selected down | Channels | action | `Opt+Down` |  | Channel Rack › ⏵ › Move selected down | IMPLEMENTED | SEQ-007 | Alt+↓. |
| MENU-342 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Sort by | Channels | submenu |  |  | Channel Rack › ⏵ › Sort by | DIFFERENT | SEQ-007 | Session 16: by name, type (sampler, synth, plugin) or colour (hue); FL's own sort list not checked. |
| MENU-343 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Group selected... | Channels | dialog | `Opt+G` |  | Channel Rack › ⏵ › Group selected… | IMPLEMENTED | SEQ-012 | Also Alt+G. |
| MENU-344 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Color selected | Channels | submenu |  |  | Channel Rack › ⏵ › Color selected | IMPLEMENTED | SEQ-007 | LimeShot colours one channel at a time. Session 16: a swatch submenu. |
| MENU-345 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Mute selected | Channels | action |  |  | Channel Rack › ⏵ › Mute selected | IMPLEMENTED | SEQ-003 |  |
| MENU-346 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Unmute selected | Channels | action |  |  | Channel Rack › ⏵ › Unmute selected | IMPLEMENTED | SEQ-003 |  |
| MENU-347 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Transpose selected... | Channels | dialog |  |  | Channel Rack › ⏵ › Transpose selected… | IMPLEMENTED | SEQ-007 | Session 16: the selected channels' notes in the current pattern. |
| MENU-348 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set swing mix for selected | Channels | submenu |  |  | — | NOT_STARTED | SEQ-010 |  |
| MENU-349 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Set truncate swing notes for selected | Channels | submenu |  |  | — | NOT_STARTED | SEQ-010 |  |
| MENU-350 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Burn selected loops to pattern | Channels | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-351 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Assign selected to free mixer track(s) | Channels | action | `Cmd+L` |  | Channel Rack › ⏵ › Assign selected to free mixer track(s) | IMPLEMENTED | SEQ-005 | Cmd+L routes the selected channels, as in FL. |
| MENU-352 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Select mixer tracks for the selected channels | Channels | action |  |  | Channel Rack › ⏵ › Select mixer tracks for the selected channels | DIFFERENT | SEQ-005 | Session 16: selects the first channel's track and shows the Mixer (LimeShot's Mixer selects one track). |
| MENU-353 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change declicking for selected channels to | Channels | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-354 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Zip selected | Channels | action | `Opt+Z` |  | — | NOT_STARTED | SEQ-013 |  |
| MENU-355 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Unzip all | Channels | action | `Opt+U` |  | — | NOT_STARTED | SEQ-013 |  |
| MENU-356 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Restretch all | Channels | action | `Cmd+Opt+R` | disabled | — | NOT_STARTED |  |  |
| MENU-357 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Group playlist tracks created by Loop starter | Options | submenu |  |  | — | NOT_PLANNED | SEQ-017 |  |
| MENU-358 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Auto resize | Options | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-359 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Auto switch display filter | Options | toggle |  | checked | — | NOT_STARTED | SEQ-012 |  |
| MENU-360 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Channel button width | Options | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-361 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Channel name alignment | Options | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-362 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Colorful channel controls | Options | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-363 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Focus selected channel on playback | Options | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-364 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Mute removed steps | Options | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-365 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show mixer track selectors | Options | toggle | `Opt+M` | checked | Channel Rack › mixer track numbers | DIFFERENT | SEQ-005 | LimeShot always shows them. |
| MENU-366 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show complete piano roll preview | Options | toggle |  | checked | Channel Rack › piano roll preview | DIFFERENT | SEQ-008 | Always on in LimeShot. |
| MENU-367 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show a mute button on the piano roll preview | Options | toggle |  | checked | — | NOT_STARTED | SEQ-008 |  |
| MENU-368 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Detached | Options | toggle |  |  | Channel Rack › detach button | DIFFERENT | UI-007 | The title bar's detach button. |
| MENU-369 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch smart disable for all plugins | Switch | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-370 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch all audio clip channels to | Switch | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-371 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch selected audio clip channels to | Switch | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-372 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch all plugin editors to | Switch | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-373 | Channel menu |  | submenu |  |  | Channel Rack › channel menu | IMPLEMENTED | SEQ-007 | Right-click on a channel button: two columns in FL (channel; steps and MIDI). |
| MENU-374 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Piano roll |  | toggle | `F7` | checked | Channel Rack › channel menu › Piano Roll | IMPLEMENTED | PR-001 |  |
| MENU-375 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Graph editor |  | toggle |  |  | Channel Rack › channel menu › Graph editor | IMPLEMENTED | SEQ-011 |  |
| MENU-376 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Rename, color and icon... |  | dialog |  |  | Channel Rack › channel menu › Rename… / Color | DIFFERENT | SEQ-007 | Separate items; no icons. |
| MENU-377 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change color... |  | dialog |  |  | Channel Rack › channel menu › Color | DIFFERENT | SEQ-007 | A swatch submenu. |
| MENU-378 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Random color |  | action |  |  | Channel Rack › channel menu › Random color | IMPLEMENTED | SEQ-007 | Session 16. |
| MENU-379 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change icon... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-380 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Load sample... |  | dialog |  |  | Channel Rack › channel menu › Load sample… | IMPLEMENTED | INS-001 | Sampler channels. |
| MENU-381 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Cut itself |  | toggle |  | checked | — | NOT_STARTED |  | A new note cuts the channel's sounding note. |
| MENU-382 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Show waveform preview |  | toggle |  | disabled | — | NOT_STARTED |  |  |
| MENU-383 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Insert |  | submenu |  |  | — | NOT_STARTED | SEQ-002 | Inserts a generator below this channel. |
| MENU-384 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Replace |  | submenu |  |  | — | NOT_STARTED | SEQ-002 |  |
| MENU-385 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Patcherize |  | action |  | disabled | — | NOT_PLANNED |  | FL's Patcher. |
| MENU-386 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Clone |  | action |  |  | Channel Rack › channel menu › Clone | IMPLEMENTED | SEQ-007 |  |
| MENU-387 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Delete... |  | dialog |  |  | Channel Rack › channel menu › Delete… | IMPLEMENTED | SEQ-007 |  |
| MENU-388 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Assign to new instrument track |  | action |  |  | — | NOT_STARTED |  |  |
| MENU-389 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Cut |  | action |  |  | Edit › Cut | DIFFERENT | SEQ-015 | LimeShot: Edit menu and Cmd+X with the Channel Rack focused. |
| MENU-390 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Copy |  | action |  |  | Edit › Copy | DIFFERENT | SEQ-015 |  |
| MENU-391 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Paste |  | action |  | disabled | Edit › Paste | DIFFERENT | SEQ-015 |  |
| MENU-392 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Fill each 2 steps |  | action |  |  | Channel Rack › channel menu fill items | IMPLEMENTED | SEQ-014 |  |
| MENU-393 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Fill each 4 steps |  | action |  |  | Channel Rack › channel menu fill items | IMPLEMENTED | SEQ-014 |  |
| MENU-394 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Fill each 8 steps |  | action |  |  | Channel Rack › channel menu fill items | IMPLEMENTED | SEQ-014 |  |
| MENU-395 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Advanced fill... |  | dialog |  |  | — | NOT_STARTED | SEQ-014 |  |
| MENU-396 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Auto fill |  | action |  | disabled | — | NOT_STARTED | SEQ-014 |  |
| MENU-397 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Rotate left |  | action | `Shift+Cmd+Left` |  | Channel Rack › channel menu › Rotate left | IMPLEMENTED | SEQ-014 |  |
| MENU-398 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Rotate right |  | action | `Shift+Cmd+Right` |  | Channel Rack › channel menu › Rotate right | IMPLEMENTED | SEQ-014 |  |
| MENU-399 | &nbsp;&nbsp;&nbsp;&nbsp;↳ MIDI channel through |  | toggle |  | checked | — | NOT_STARTED | WF-003 |  |
| MENU-400 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Receive notes from |  | submenu |  |  | — | NOT_STARTED | WF-003 |  |
| MENU-401 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Create DirectWave instrument... |  | dialog |  |  | — | NOT_PLANNED |  | FL's DirectWave. |
| MENU-402 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Burn MIDI to |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-403 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Channel Settings | LimeShot | action |  |  | Channel Rack › channel menu › Channel Settings | LIMESHOT_ONLY | SEQ-007 | Opens the channel's settings window. |
| MENU-404 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Clear notes in this pattern | LimeShot | action |  |  | Channel Rack › channel menu › Clear notes in this pattern | LIMESHOT_ONLY | SEQ-007 |  |
| MENU-405 | Display filter |  | list |  |  | Channel Rack › filter | IMPLEMENTED | SEQ-012 | All, Unsorted and the filter groups (Audio for audio clip channels). |

## Piano Roll

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-406 | Options |  | submenu |  |  | Piano Roll › ⏵ | IMPLEMENTED |  | The ▸ at the Piano Roll's top left. LimeShot has no Piano Roll menu; its commands are keys (shortcuts help) and the tool buttons. Session 16: LimeShot's ⏵ button at the left of the Piano Roll header opens Edit, Tools, Select, Zoom and Center. |
| MENU-407 | &nbsp;&nbsp;&nbsp;&nbsp;↳ File |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-408 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Open score... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-409 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Browse scores |  | action |  |  | — | NOT_STARTED |  |  |
| MENU-410 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Save score as... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-411 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Import MIDI file... |  | dialog | `Cmd+M` |  | File › Import MIDI file… | DIFFERENT | PR-020 | LimeShot imports into new channels from the main File menu. |
| MENU-412 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Export as MIDI file... |  | dialog | `Shift+Cmd+M` |  | File › Export MIDI file (pattern)… | DIFFERENT | PR-020 |  |
| MENU-413 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Paste from MIDI clipboard... |  | dialog | `Shift+Cmd+V` | disabled | — | NOT_STARTED |  |  |
| MENU-414 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Copy to MIDI clipboard... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-415 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Export as score sheet... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-416 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit |  | submenu |  |  | Piano Roll › ⏵ › Edit | IMPLEMENTED |  |  |
| MENU-417 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Cut |  | action | `Cmd+X` |  | Piano Roll › ⏵ › Edit › Cut | IMPLEMENTED | PR-019 | Main Edit menu and Cmd+X with the Piano Roll focused. |
| MENU-418 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Copy |  | action | `Cmd+C` |  | Piano Roll › ⏵ › Edit › Copy | IMPLEMENTED | PR-019 |  |
| MENU-419 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Paste |  | action | `Cmd+V` | disabled | Piano Roll › ⏵ › Edit › Paste | IMPLEMENTED | PR-019 |  |
| MENU-420 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Duplicate |  | action | `Cmd+B` |  | Piano Roll › ⏵ › Edit › Duplicate | IMPLEMENTED | PR-008 |  |
| MENU-421 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete |  | action | `Del` |  | Piano Roll › ⏵ › Edit › Delete | IMPLEMENTED | PR-001 |  |
| MENU-422 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift left |  | action | `Shift+Left` |  | Piano Roll › ⏵ › Edit › Shift left | IMPLEMENTED | PR-009 | By the snap. |
| MENU-423 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift right |  | action | `Shift+Right` |  | Piano Roll › ⏵ › Edit › Shift right | IMPLEMENTED | PR-009 |  |
| MENU-424 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Rotate left |  | action | `Shift+Cmd+Left` |  | Piano Roll › ⏵ › Edit › Rotate left | IMPLEMENTED | PR-009 |  |
| MENU-425 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Rotate right |  | action | `Shift+Cmd+Right` |  | Piano Roll › ⏵ › Edit › Rotate right | IMPLEMENTED | PR-009 |  |
| MENU-426 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Transpose up |  | action | `Shift+Up` |  | Piano Roll › ⏵ › Edit › Transpose up | IMPLEMENTED | PR-009 |  |
| MENU-427 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Transpose down |  | action | `Shift+Down` |  | Piano Roll › ⏵ › Edit › Transpose down | IMPLEMENTED | PR-009 |  |
| MENU-428 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Transpose one octave up |  | action | `Cmd+Up` |  | Piano Roll › ⏵ › Edit › Transpose one octave up | IMPLEMENTED | PR-009 |  |
| MENU-429 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Transpose one octave down |  | action | `Cmd+Down` |  | Piano Roll › ⏵ › Edit › Transpose one octave down | IMPLEMENTED | PR-009 |  |
| MENU-430 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Discard lengths |  | action | `Shift+D` |  | Piano Roll › ⏵ › Edit › Discard lengths | IMPLEMENTED | PR-015 | Session 16: sets the notes (selection, or the channel) to one step; the exact length FL uses was not observed. |
| MENU-431 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Allow resizing from left |  | toggle | `Cmd+Opt+Home` | checked | Piano Roll › left note edge | DIFFERENT | PR-024 | Always on in LimeShot. |
| MENU-432 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Nudge notes with mouse wheel |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-433 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Change color |  | action | `Opt+C` |  | — | NOT_STARTED | PR-018 |  |
| MENU-434 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Mute |  | action | `Opt+M` |  | Piano Roll › ⏵ › Edit › Mute | IMPLEMENTED | PR-012 |  |
| MENU-435 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Unmute |  | action | `Shift+Opt+M` |  | Piano Roll › ⏵ › Edit › Unmute | IMPLEMENTED | PR-012 |  |
| MENU-436 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Insert space |  | action | `Cmd+Ins` | disabled | — | NOT_STARTED |  |  |
| MENU-437 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Slice & insert space |  | action | `Cmd+Opt+Ins` | disabled | — | NOT_STARTED |  |  |
| MENU-438 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete space |  | action | `Cmd+Del` | disabled | — | NOT_STARTED |  |  |
| MENU-439 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Trim selection |  | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-440 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Insert current controller value |  | action | `Cmd+I` | disabled | — | NOT_STARTED |  |  |
| MENU-441 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Turn into automation clip... |  | dialog |  | disabled | — | NOT_STARTED |  |  |
| MENU-442 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Purge unused audio clips |  | action |  |  | Tools › Purge unused audio clips | DIFFERENT | WF-009 |  |
| MENU-443 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select tracks when samples are dropped |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-444 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tools |  | submenu |  |  | Piano Roll › ⏵ › Tools | IMPLEMENTED |  | Built-in tools on the left, scripts on the right. |
| MENU-445 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Riff machine... |  | dialog | `Opt+E` |  | — | NOT_STARTED | PR-029 |  |
| MENU-446 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Generate chord progression... |  | dialog | `Opt+P` |  | — | NOT_STARTED | PR-028 |  |
| MENU-447 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick legato |  | action | `Cmd+L` |  | Piano Roll › ⏵ › Tools › Quick legato | IMPLEMENTED | PR-015 | Session 16: each note ends where the next later note starts; chords stay together, the last notes keep their lengths. |
| MENU-448 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Articulate... |  | dialog | `Opt+L` |  | Piano Roll › ⏵ › Tools › Articulate… | DIFFERENT | PR-015 | Session 16: lengths as a share of the gap to the next note, or a fixed length in snap steps. FL's dialog was not opened (a system launcher caught the key). |
| MENU-449 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick quantize |  | action | `Cmd+Q` |  | Piano Roll › ⏵ › Tools › Quick quantize | DIFFERENT | PR-014 | LimeShot's Alt+Q quantizes to the snap (Cmd+Q quits on macOS). Session 16: starts and lengths to the snap. No key: Cmd+Q quits LimeShot. |
| MENU-450 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick quantize start times |  | action | `Shift+Q` |  | Piano Roll › ⏵ › Tools › Quick quantize start times | IMPLEMENTED | PR-014 | Session 16: Shift+Q. |
| MENU-451 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quantize... |  | dialog | `Opt+Q` |  | Piano Roll › ⏵ › Tools › Quantize… | IMPLEMENTED | PR-014 | LimeShot's Quantize button quantizes at once; no dialog. Session 16: start-time and duration strength to the snap. |
| MENU-452 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick chop |  | action | `Cmd+U` |  | Piano Roll › ⏵ › Tools › Quick chop | IMPLEMENTED | PR-023 |  |
| MENU-453 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Chop... |  | dialog | `Opt+U` |  | Piano Roll › ⏵ › Tools › Chop… | DIFFERENT | PR-023 | Session 16: pieces of N snap steps with a gate. FL's dialog not observed. |
| MENU-454 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Glue |  | action | `Cmd+G` |  | Piano Roll › ⏵ › Tools › Glue | IMPLEMENTED | PR-023 |  |
| MENU-455 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Arpeggiate... |  | dialog | `Opt+A` |  | Piano Roll › ⏵ › Tools › Arpeggiate… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-arp.png): Pattern (up, down, up/down, down/up, random), Range in octaves, Time mul of the snap, Gate, a velocity slope. Not: FL's pattern presets, Sync modes, the pan/release/mod/pitch level knobs, Group notes. |
| MENU-456 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Strum... |  | dialog | `Opt+S` |  | Piano Roll › ⏵ › Tools › Strum… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-strum.png): Start time and velocity with tension, Preserve end, Trigger ahead, End time and tension, Chop chords, Alternate direction. |
| MENU-457 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Flam... |  | dialog | `Opt+F` |  | Piano Roll › ⏵ › Tools › Flam… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-flam.png): Time in ticks, Before, Velocity. Not: Absolute, Group notes. |
| MENU-458 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Claw machine... |  | dialog | `Opt+W` |  | — | NOT_STARTED | PR-015 | Session 16: dialog observed (fl-piano-tool-claw.png: Period, Trash every, Time dist, Stretch to compensate, Remove short notes); not implemented. |
| MENU-459 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Limit... |  | dialog | `Opt+K` |  | Piano Roll › ⏵ › Tools › Limit… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-limit.png): Key / scale restriction with Snap (above, below, nearest), keyboard range (sliders, not FL's keyboard strip), Wrap to top / bottom. |
| MENU-460 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Flip... |  | dialog | `Opt+Y` |  | Piano Roll › ⏵ › Tools › Flip… | DIFFERENT | PR-015 | Session 16: horizontally and/or vertically around the notes' span. FL's dialog not observed. |
| MENU-461 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Randomize... |  | dialog | `Opt+R` |  | Piano Roll › ⏵ › Tools › Randomize… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-random.png): Octave, Range, Key / scale, Length, Variation, Population, Stack, Glue same notes, Seed, velocity and pan spread; fills the selection's span (or the pattern). Not: Random portamento, release/mod/pitch levels, Bipolar. |
| MENU-462 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Scale levels... |  | dialog | `Opt+X` |  | Piano Roll › ⏵ › Tools › Scale levels… | IMPLEMENTED | PR-015 | Session 16 (fl-piano-tool-scale.png): Center, Multiply, Offset, Tension on note velocity or pan. |
| MENU-463 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ LFO... |  | dialog | `Opt+O` |  | Piano Roll › ⏵ › Tools › LFO… | DIFFERENT | PR-015 | Session 16: shape, period, amount, center and phase on note velocity or pan. FL's dialog not observed. |
| MENU-464 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Run last script again |  | action |  | disabled | — | NOT_PLANNED |  | FL's Python scripts. |
| MENU-465 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Open script folder... |  | action |  |  | — | NOT_PLANNED |  |  |
| MENU-466 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ <scripts> |  | list |  |  | — | NOT_PLANNED |  | Arpeggiator, Euclidean, Humanize, Note repeater sprinkler, Select by velocity and more. |
| MENU-467 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Stamp |  | submenu |  |  | — | NOT_STARTED | PR-027 | Chords and scales to stamp. |
| MENU-468 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-469 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Grid color... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-470 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Grid contrast |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-471 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Invert grid |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-472 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Time segments |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-473 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Keep labels on screen |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-474 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shadow |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-475 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Rounded |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-476 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note colors |  | submenu |  |  | — | NOT_STARTED | PR-018 |  |
| MENU-477 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Incremental scrolling |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-478 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Precise time indicator |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-479 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Mini piano roll preview |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-480 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note grid highlights |  | toggle | `Opt+B` | checked | — | NOT_STARTED |  |  |
| MENU-481 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Scale highlighting |  | submenu |  |  | — | NOT_STARTED | PR-013 |  |
| MENU-482 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show chord name for selection |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-483 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note length in note properties |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-484 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Ghost channels |  | toggle | `Opt+V` | checked | Piano Roll › ghost notes | DIFFERENT | PR-010 | Always on in LimeShot. |
| MENU-485 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Editable ghosts |  | toggle | `Cmd+Opt+V` |  | — | NOT_STARTED |  |  |
| MENU-486 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Background waveform |  | toggle | `Opt+N` |  | — | NOT_STARTED | PR-025 |  |
| MENU-487 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Keyboard options |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-488 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Swap panels |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-489 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Flip separator |  | action | `Shift+Enter` |  | — | NOT_STARTED |  |  |
| MENU-490 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Remove background picture |  | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-491 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Snap |  | submenu |  |  | Toolbar › snap selector | DIFFERENT | PR-003 | FL's Piano Roll has its own snap: Main (the toolbar's), Line, Cell, (none) … Bar, Events, Markers. LimeShot's editors follow the toolbar snap. |
| MENU-492 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Select |  | submenu |  |  | Piano Roll › ⏵ › Select | IMPLEMENTED |  |  |
| MENU-493 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Deselect |  | action | `Cmd+D` | disabled | Piano Roll › ⏵ › Select › Deselect | IMPLEMENTED | PR-002 |  |
| MENU-494 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select all |  | action | `Cmd+A` |  | Piano Roll › ⏵ › Select › Select all | IMPLEMENTED | PR-002 |  |
| MENU-495 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select one at random |  | action | `Shift+R` |  | Piano Roll › ⏵ › Select › Select one at random | IMPLEMENTED | PR-002 |  |
| MENU-496 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select more at random |  | action | `Shift+M` |  | Piano Roll › ⏵ › Select › Select more at random | IMPLEMENTED | PR-002 |  |
| MENU-497 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select by color |  | action | `Shift+C` |  | — | NOT_STARTED | PR-018 |  |
| MENU-498 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select odd |  | action | `Shift+O` |  | Piano Roll › ⏵ › Select › Select odd | IMPLEMENTED | PR-002 | Session 16: the 1st, 3rd, 5th … note in time order (of the selection, or the channel). |
| MENU-499 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select muted |  | action |  |  | Piano Roll › ⏵ › Select › Select muted | IMPLEMENTED | PR-002 |  |
| MENU-500 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select overlapping notes |  | action |  |  | Piano Roll › ⏵ › Select › Select overlapping notes | IMPLEMENTED | PR-002 |  |
| MENU-501 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select stacked notes |  | action |  |  | Piano Roll › ⏵ › Select › Select stacked notes | IMPLEMENTED | PR-002 |  |
| MENU-502 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Invert selection |  | action | `Shift+I` |  | Piano Roll › ⏵ › Select › Invert selection | IMPLEMENTED | PR-002 |  |
| MENU-503 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select time around selection |  | action | `Cmd+Enter` |  | — | NOT_STARTED |  |  |
| MENU-504 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select previous time |  | action | `Cmd+Left` | disabled | — | NOT_STARTED |  |  |
| MENU-505 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select next time |  | action | `Cmd+Right` | disabled | — | NOT_STARTED |  |  |
| MENU-506 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Magic lasso |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-507 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Group |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-508 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Group |  | action | `Shift+G` |  | — | NOT_STARTED |  |  |
| MENU-509 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Ungroup |  | action | `Opt+G` |  | — | NOT_STARTED |  |  |
| MENU-510 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom |  | submenu |  |  | Piano Roll › ⏵ › Zoom | IMPLEMENTED | PR-005 |  |
| MENU-511 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom in |  | action | `PgUp` |  | Piano Roll › ⏵ › Zoom › Zoom in | IMPLEMENTED | PR-005 | LimeShot zooms with Cmd+wheel and pinch. |
| MENU-512 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom out |  | action | `PgDn` |  | Piano Roll › ⏵ › Zoom › Zoom out | IMPLEMENTED | PR-005 |  |
| MENU-513 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 1 |  | action | `Shift+1` |  | Piano Roll › ⏵ › Zoom › Quick zoom 1 | DIFFERENT | PR-005 | Session 16: LimeShot's levels (1, 4, 16 bars; the whole pattern; a Channel Rack step per step), not observed in FL. |
| MENU-514 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 2 |  | action | `Shift+2` |  | Piano Roll › ⏵ › Zoom › Quick zoom 2 | DIFFERENT | PR-005 | Session 16: LimeShot's levels (1, 4, 16 bars; the whole pattern; a Channel Rack step per step), not observed in FL. |
| MENU-515 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 3 |  | action | `Shift+3` |  | Piano Roll › ⏵ › Zoom › Quick zoom 3 | DIFFERENT | PR-005 | Session 16: LimeShot's levels (1, 4, 16 bars; the whole pattern; a Channel Rack step per step), not observed in FL. |
| MENU-516 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom out far |  | action | `Shift+4` |  | Piano Roll › ⏵ › Zoom › Zoom out far | DIFFERENT | PR-005 | Session 16: LimeShot's levels (1, 4, 16 bars; the whole pattern; a Channel Rack step per step), not observed in FL. |
| MENU-517 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom on selection |  | action | `Shift+5` |  | Piano Roll › ⏵ › Zoom › Zoom on selection | IMPLEMENTED | PR-005 |  |
| MENU-518 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Step sequencer mode zoom |  | action | `Shift+6` |  | Piano Roll › ⏵ › Zoom › Step sequencer mode zoom | DIFFERENT | PR-005 | Session 16: LimeShot's levels (1, 4, 16 bars; the whole pattern; a Channel Rack step per step), not observed in FL. |
| MENU-519 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Time markers |  | submenu |  |  | — | NOT_STARTED | PR-022 | The Playlist has time markers (PL); the Piano Roll's are separate in FL. |
| MENU-520 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add one... |  | dialog | `Opt+T` |  | — | NOT_STARTED | PR-022 |  |
| MENU-521 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add auto |  | action | `Cmd+T` |  | — | NOT_STARTED | PR-022 |  |
| MENU-522 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add two... |  | dialog |  | disabled | — | NOT_STARTED | PR-022 |  |
| MENU-523 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add one every |  | submenu |  |  | — | NOT_STARTED | PR-022 |  |
| MENU-524 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add time signature change... |  | dialog | `Shift+Opt+T` |  | — | NOT_STARTED | PR-022 |  |
| MENU-525 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add key marker |  | submenu |  |  | — | NOT_STARTED | PR-022 |  |
| MENU-526 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete |  | action |  |  | — | NOT_STARTED | PR-022 |  |
| MENU-527 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Target channel |  | list |  |  | Piano Roll › channel selector | DIFFERENT | PR-011 | FL lists the channels; LimeShot's Piano Roll header has a channel selector. |
| MENU-528 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Target control |  | submenu |  |  | — | NOT_STARTED | PR-016 | Which note property the lower panel edits. |
| MENU-529 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note pan |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-530 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note velocity |  | radio |  | selected | Piano Roll › velocity lane | DIFFERENT | PR-004 | LimeShot's lower panel edits velocity only. |
| MENU-531 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note release |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-532 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note filter cutoff frequency |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-533 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note filter resonance (Q) |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-534 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note fine pitch |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-535 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Note repeat |  | radio |  |  | — | NOT_STARTED | PR-016 |  |
| MENU-536 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Channel panning |  | radio |  |  | — | NOT_STARTED | AUT-003 |  |
| MENU-537 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Channel volume |  | radio |  |  | — | NOT_STARTED | AUT-003 |  |
| MENU-538 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Channel pitch |  | radio |  |  | — | NOT_STARTED | AUT-003 |  |
| MENU-539 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Auto smoothing |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-540 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Preview notes during playback |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-541 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Center |  | action | `Shift+0` |  | Piano Roll › ⏵ › Center | DIFFERENT | PR-005 | Session 16: scrolls to the playback position (or the start) and the channel's notes; FL's exact behaviour not observed. |
| MENU-542 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Detached |  | toggle |  |  | Piano Roll › detach button | DIFFERENT | UI-007 | The title bar's detach button. |

## Playlist

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-543 | Options |  | submenu |  |  | Playlist › ⏵ | IMPLEMENTED |  | The ▸ at the Playlist's top left. LimeShot has no Playlist menu; its commands are keys, the tool buttons and the clip and track menus. Session 16: LimeShot's ⏵ button at the left of the Playlist header: Edit, Tools, Select, Zoom, Time markers, Center. |
| MENU-544 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit |  | submenu |  |  | Playlist › ⏵ › Edit | IMPLEMENTED |  |  |
| MENU-545 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Cut |  | action | `Cmd+X` | disabled | Playlist › ⏵ › Edit › Cut | IMPLEMENTED | PL-026 | Main Edit menu and Cmd+X with the Playlist focused. |
| MENU-546 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Copy |  | action | `Cmd+C` | disabled | Playlist › ⏵ › Edit › Copy | IMPLEMENTED | PL-026 |  |
| MENU-547 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Paste |  | action | `Cmd+V` | disabled | Playlist › ⏵ › Edit › Paste | IMPLEMENTED | PL-026 |  |
| MENU-548 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Duplicate |  | action | `Cmd+B` | disabled | Playlist › ⏵ › Edit › Duplicate | IMPLEMENTED | PL-005 |  |
| MENU-549 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete |  | action | `Del` | disabled | Playlist › ⏵ › Edit › Delete | IMPLEMENTED | PL-004 |  |
| MENU-550 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift left |  | action | `Shift+Left` |  | Playlist › ⏵ › Edit › Shift left | IMPLEMENTED | PL-002 |  |
| MENU-551 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift right |  | action | `Shift+Right` |  | Playlist › ⏵ › Edit › Shift right | IMPLEMENTED | PL-002 |  |
| MENU-552 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift up |  | action | `Shift+Up` |  | Playlist › ⏵ › Edit › Shift up | IMPLEMENTED | PL-002 | A track up. |
| MENU-553 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shift down |  | action | `Shift+Down` |  | Playlist › ⏵ › Edit › Shift down | IMPLEMENTED | PL-002 |  |
| MENU-554 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Make unique |  | action | `Shift+Cmd+V` | disabled | Playlist › ⏵ › Edit › Make unique | IMPLEMENTED | PL-011 |  |
| MENU-555 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Make unique as sample |  | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-556 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Allow resizing from left |  | toggle | `Cmd+Opt+Home` | checked | Playlist › left clip edge | DIFFERENT | PL-003 | Always on. |
| MENU-557 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Nudge clips with mouse wheel |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-558 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Dropping on track header locks to content |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-559 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Drop audio on track headers |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-560 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Mute |  | action | `Opt+M` |  | Playlist › ⏵ › Edit › Mute | IMPLEMENTED | PL-013 | LimeShot mutes clips with the mute tool and the clip menu. |
| MENU-561 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Unmute |  | action | `Shift+Opt+M` |  | Playlist › ⏵ › Edit › Unmute | IMPLEMENTED | PL-013 |  |
| MENU-562 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Insert space |  | action | `Cmd+Ins` | disabled | Playlist › ⏵ › Edit › Insert space | IMPLEMENTED | PL-016 | Session 16: the time selection's length at its start (a bar at the position without one); clips and markers after it move. |
| MENU-563 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Slice & insert space |  | action | `Cmd+Opt+Ins` | disabled | Playlist › ⏵ › Edit › Slice & insert space | IMPLEMENTED | PL-016 | Session 16: clips crossing the start are cut first. |
| MENU-564 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete space |  | action | `Cmd+Del` | disabled | Playlist › ⏵ › Edit › Delete space | IMPLEMENTED | PL-016 | Session 16: needs a time selection; clips are cut at its ends. |
| MENU-565 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Trim selection |  | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-566 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Merge selected clips |  | action | `Cmd+G` |  | Playlist › ⏵ › Edit › Merge selected clips | IMPLEMENTED | PL-022 | Session 16: pattern clips only, into a new pattern '<first> (merged)' on the topmost track; FL's naming not checked. |
| MENU-567 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Merge similar pattern clips |  | action | `Shift+Cmd+G` |  | — | NOT_STARTED | PL-022 |  |
| MENU-568 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Merge automation clips |  | action | `Cmd+Opt+G` |  | — | NOT_STARTED | PL-015 |  |
| MENU-569 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Purge unused audio clips |  | action |  |  | Tools › Purge unused audio clips | DIFFERENT | WF-009 |  |
| MENU-570 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Pitch selected audio clips one semitone down |  | action | `7` |  | — | NOT_STARTED | PL-023 |  |
| MENU-571 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Pitch selected audio clips one semitone up |  | action | `8` |  | — | NOT_STARTED | PL-023 |  |
| MENU-572 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Reverse selected audio clips |  | action | `9` |  | — | NOT_STARTED | PL-014 |  |
| MENU-573 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Automatic crossfades (current project) |  | toggle |  | checked | — | NOT_STARTED | PL-023 |  |
| MENU-574 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Create new clips with manual fades |  | toggle |  |  | — | NOT_STARTED | PL-023 |  |
| MENU-575 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Snap fade handles |  | toggle |  |  | — | NOT_STARTED | PL-023 |  |
| MENU-576 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Create variants when stretching audio clips |  | toggle |  | checked | — | NOT_STARTED | PL-023 |  |
| MENU-577 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Tools |  | submenu |  |  | Playlist › ⏵ › Tools | IMPLEMENTED |  |  |
| MENU-578 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick quantize start times |  | action | `Shift+Q` |  | Playlist › ⏵ › Tools › Quick quantize start times | IMPLEMENTED | PL-002 | Session 16: clip starts to the snap. |
| MENU-579 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Consolidate playlist selection |  | submenu |  | disabled | — | NOT_STARTED | PL-022 |  |
| MENU-580 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Export all playlist tracks |  | submenu |  |  | — | NOT_STARTED | EXP-001 |  |
| MENU-581 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Export all playlist track groups |  | submenu |  |  | — | NOT_STARTED | PL-018 |  |
| MENU-582 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Clone selected playlist track(s) |  | action | `Shift+Cmd+B` | disabled | — | NOT_STARTED |  |  |
| MENU-583 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-584 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Grid color... |  | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-585 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Grid contrast |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-586 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Invert grid |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-587 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Track separators |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-588 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Time segments |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-589 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Keep labels on screen |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-590 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Content in titlebars |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-591 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Behind clips |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-592 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Shadow |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-593 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Resize all tracks |  | submenu |  |  | Playlist › key Opt+wheel | DIFFERENT | PL-008 | LimeShot sets every track's height with Opt+wheel. |
| MENU-594 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Hide collapsed grouped tracks |  | toggle |  |  | — | NOT_STARTED | PL-018 |  |
| MENU-595 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show fade previews |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-596 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show gain value |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-597 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show gain scale |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-598 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Incremental scrolling |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-599 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Precise time indicator |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-600 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Performance clip progress |  | toggle |  | checked | — | NOT_STARTED | PL-019 |  |
| MENU-601 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Performance track progress |  | toggle |  | checked | — | NOT_STARTED | PL-019 |  |
| MENU-602 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Mini playlist preview |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-603 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show controls on audio tracks |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-604 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show levels on audio tracks |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-605 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show levels on instrument tracks |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-606 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Remove background picture |  | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-607 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Snap |  | submenu |  |  | Toolbar › snap selector | DIFFERENT | PL-007 | FL's Playlist has its own snap list (as the Piano Roll's); LimeShot follows the toolbar snap. |
| MENU-608 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Select |  | submenu |  |  | Playlist › ⏵ › Select | IMPLEMENTED |  |  |
| MENU-609 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Deselect |  | action | `Cmd+D` | disabled | Playlist › ⏵ › Select › Deselect | IMPLEMENTED | PL-006 |  |
| MENU-610 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select all |  | action | `Cmd+A` |  | Playlist › ⏵ › Select › Select all | IMPLEMENTED | PL-006 |  |
| MENU-611 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select by selected source |  | action | `Shift+C` |  | Playlist › ⏵ › Select › Select by selected source | IMPLEMENTED | PL-006 |  |
| MENU-612 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select muted |  | action |  |  | Playlist › ⏵ › Select › Select muted | IMPLEMENTED | PL-006 |  |
| MENU-613 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select overlapping clips |  | action |  |  | Playlist › ⏵ › Select › Select overlapping clips | IMPLEMENTED | PL-006 |  |
| MENU-614 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select stacked clips |  | action |  |  | Playlist › ⏵ › Select › Select stacked clips | IMPLEMENTED | PL-006 |  |
| MENU-615 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Invert selection |  | action | `Shift+I` |  | Playlist › ⏵ › Select › Invert selection | IMPLEMENTED | PL-006 |  |
| MENU-616 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select time around selection |  | action | `Cmd+Enter` |  | Playlist › ⏵ › Select › Select time around selection | IMPLEMENTED | PL-016 |  |
| MENU-617 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select previous time |  | action | `Cmd+Left` | disabled | — | NOT_STARTED |  |  |
| MENU-618 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select next time |  | action | `Cmd+Right` | disabled | — | NOT_STARTED |  |  |
| MENU-619 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Magic lasso |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-620 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Group |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-621 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Group |  | action | `Shift+G` |  | — | NOT_STARTED | PL-018 |  |
| MENU-622 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Ungroup |  | action | `Opt+G` |  | — | NOT_STARTED | PL-018 |  |
| MENU-623 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom |  | submenu |  |  | Playlist › ⏵ › Zoom | IMPLEMENTED | PL-008 |  |
| MENU-624 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom in |  | action | `PgUp` |  | Playlist › ⏵ › Zoom › Zoom in | IMPLEMENTED | PL-008 |  |
| MENU-625 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom out |  | action | `PgDn` |  | Playlist › ⏵ › Zoom › Zoom out | IMPLEMENTED | PL-008 |  |
| MENU-626 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 1 |  | action | `Shift+1` |  | Playlist › ⏵ › Zoom › Quick zoom 1 | DIFFERENT | PL-008 | Session 16: LimeShot's levels (4, 16, 64 bars; the whole song), not observed in FL. |
| MENU-627 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 2 |  | action | `Shift+2` |  | Playlist › ⏵ › Zoom › Quick zoom 2 | DIFFERENT | PL-008 | Session 16: LimeShot's levels (4, 16, 64 bars; the whole song), not observed in FL. |
| MENU-628 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Quick zoom 3 |  | action | `Shift+3` |  | Playlist › ⏵ › Zoom › Quick zoom 3 | DIFFERENT | PL-008 | Session 16: LimeShot's levels (4, 16, 64 bars; the whole song), not observed in FL. |
| MENU-629 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom out far |  | action | `Shift+4` |  | Playlist › ⏵ › Zoom › Zoom out far | DIFFERENT | PL-008 | Session 16: LimeShot's levels (4, 16, 64 bars; the whole song), not observed in FL. |
| MENU-630 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom on selection |  | action | `Shift+5` |  | Playlist › ⏵ › Zoom › Zoom on selection | IMPLEMENTED | PL-008 | Session 16: the selected clips, or the time selection. |
| MENU-631 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Zoom on performance clips |  | action | `Shift+6` | disabled | — | NOT_STARTED | PL-019 |  |
| MENU-632 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Time markers |  | submenu |  |  | Playlist › ⏵ › Time markers | IMPLEMENTED |  |  |
| MENU-633 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add one... |  | dialog | `Opt+T` |  | Playlist › ⏵ › Time markers › Add one… | IMPLEMENTED | PL-016 | Adds a marker at the song position and asks its name. |
| MENU-634 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add auto |  | action | `Cmd+T` |  | Playlist › ⏵ › Time markers › Add auto | IMPLEMENTED | PL-016 |  |
| MENU-635 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add two... |  | dialog |  | disabled | — | NOT_STARTED | PL-016 |  |
| MENU-636 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add one every |  | submenu |  |  | — | NOT_STARTED | PL-016 |  |
| MENU-637 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add jump to next bar |  | action |  |  | — | NOT_STARTED | PL-016 |  |
| MENU-638 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add time signature change... |  | dialog | `Shift+Opt+T` |  | — | NOT_STARTED | PR-022 |  |
| MENU-639 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Add key marker |  | submenu |  |  | — | NOT_STARTED | PL-016 |  |
| MENU-640 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Place loop |  | action | `Shift+T` |  | — | NOT_STARTED | PL-016 |  |
| MENU-641 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Delete |  | action |  |  | Playlist › ⏵ › Time markers › Delete | IMPLEMENTED | PL-016 | LimeShot deletes a marker from its right-click menu. Session 16: the markers in the time selection, or the last one at or before the position. |
| MENU-642 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Change to marker loop |  | action |  |  | — | NOT_STARTED | PL-016 |  |
| MENU-643 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Picker panel |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-644 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Enable |  | toggle | `Opt+P` | checked | Playlist › picker panel | DIFFERENT | PL-021 | Always shown in LimeShot. |
| MENU-645 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show on right side |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-646 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Width |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-647 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select linked channels |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-648 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select from playlist/channel rack |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-649 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Show empty patterns |  | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-650 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Drag to rearrange |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-651 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Auto group patterns/channels |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-652 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Select unused |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-653 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Sort |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-654 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Current clip source |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-655 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ <pattern clip sources> |  | list |  |  | Playlist › picker panel | DIFFERENT | PL-025 | LimeShot picks the clip source in the picker panel. |
| MENU-656 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Edit patterns in |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-657 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Automatic |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-658 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Channel rack |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-659 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Piano roll |  | radio |  | selected | Playlist › pattern clip double-click | DIFFERENT | PL-001 | A double-click on a pattern clip opens it in the Piano Roll. |
| MENU-660 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Piano roll or event editor |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-661 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Scroll to position |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-662 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Quantize song jumps |  | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-663 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Off |  | radio |  | selected | — | NOT_STARTED |  |  |
| MENU-664 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Beats |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-665 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ Bars |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-666 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ 2 bars |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-667 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ 4 bars |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-668 | &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↳ 8 bars |  | radio |  |  | — | NOT_STARTED |  |  |
| MENU-669 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Automation clips reset remote controls |  | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-670 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Prepare for performance mode |  | action |  |  | — | NOT_STARTED | PL-019 |  |
| MENU-671 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Performance mode |  | toggle | `Cmd+P` |  | — | NOT_STARTED | PL-019 |  |
| MENU-672 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change declicking for selected channels to |  | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-673 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch all audio clip channels to |  | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-674 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch selected audio clip channels to |  | submenu |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-675 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Lock all playlist tracks to content |  | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-676 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Unlock all playlist tracks from content |  | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-677 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Reset empty playlist tracks |  | action |  |  | Tools › Reset empty playlist tracks | DIFFERENT | WF-009 | In LimeShot's Tools menu. |
| MENU-678 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Collapse all grouped tracks |  | action |  |  | — | NOT_STARTED | PL-018 |  |
| MENU-679 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Expand all grouped tracks |  | action |  |  | — | NOT_STARTED | PL-018 |  |
| MENU-680 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Center |  | action | `Shift+0` | disabled | Playlist › ⏵ › Center | DIFFERENT | PL-008 | Session 16: scrolls the song position (Song mode) to the middle; FL's exact behaviour not observed. |
| MENU-681 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Detached |  | toggle |  |  | Playlist › detach button | DIFFERENT | UI-007 |  |

## Mixer

| ID | Item | Section | Kind | FL keys | FL state | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|
| MENU-682 | Options |  | submenu |  |  | Mixer › ⏵ | DIFFERENT | MIX-001 | The ▸ at the Mixer's top left (also a track's right-click). LimeShot's mixer track menu has fewer items. Session 16: LimeShot's ⏵ sits in the track panel's header (and on a strip name's right-click) and has the track commands: rename, colour, reset, routing, solo, move, delete, insert. |
| MENU-683 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Disk recording | Options | submenu |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-684 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Plugin delay compensation | Options | submenu |  |  | Mixer › automatic delay compensation | DIFFERENT | MIX-012 | LimeShot compensates automatically; no per-track settings. |
| MENU-685 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Link all parameters... | Options | dialog |  |  | — | NOT_STARTED | WF-004 |  |
| MENU-686 | &nbsp;&nbsp;&nbsp;&nbsp;↳ View | Options | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-687 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Detached | Options | toggle |  |  | Mixer › detach button | DIFFERENT | UI-007 |  |
| MENU-688 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Trim unused tracks | Options | action |  |  | — | NOT_STARTED |  |  |
| MENU-689 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Switch smart disable for all plugins | Options | action |  |  | — | NOT_STARTED | WF-009 |  |
| MENU-690 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Remove all Edison instances | Options | action |  |  | — | NOT_PLANNED |  | FL's Edison. |
| MENU-691 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Rename, color and icon... | Mixer track | dialog | `F2` |  | Mixer › ⏵ › Rename… | DIFFERENT | MIX-001 | LimeShot renames from the track's menu; no icon. Session 16: F2 or the menu renames in place; no colour or icon in that dialog. |
| MENU-692 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change color... | Mixer track | dialog |  |  | Mixer › ⏵ › Change color | DIFFERENT | MIX-001 | Session 16: a swatch submenu, not FL's colour dialog; the strip's colour bar shows it. |
| MENU-693 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Random color | Mixer track | action |  |  | Mixer › ⏵ › Random color | IMPLEMENTED | MIX-001 |  |
| MENU-694 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Gradient color... | Mixer track | dialog |  | disabled | — | NOT_STARTED |  |  |
| MENU-695 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Change icon... | Mixer track | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-696 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Assign to new Audio track(s) | Mixer track | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-697 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Unassign from Audio track(s) | Mixer track | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-698 | &nbsp;&nbsp;&nbsp;&nbsp;↳ File | Mixer track | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-699 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Open audio editor | Mixer track | action | `Cmd+E` |  | — | NOT_PLANNED |  | FL's Edison. |
| MENU-700 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Open audio editor and record | Mixer track | action | `Shift+E` |  | — | NOT_PLANNED |  |  |
| MENU-701 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Reset selected track(s) to default | Mixer track | action |  |  | Mixer › ⏵ › Reset selected track(s) to default | IMPLEMENTED | MIX-001 | Session 16: volume, pan, EQ, mute, solo and colour; effects and routing stay (FL's exact scope not checked). |
| MENU-702 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Channel routing | Routing | submenu |  |  | Channel Rack › mixer track numbers | DIFFERENT | SEQ-005 | LimeShot routes channels from the Channel Rack and Cmd+L. |
| MENU-703 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Track routing | Routing | submenu |  | disabled | Mixer › route buttons | DIFFERENT | MIX-007 | LimeShot's route buttons under each strip. |
| MENU-704 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Select | Routing | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-705 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Reset selected track(s) routing | Routing | action |  |  | Mixer › ⏵ › Reset selected track(s) routing | IMPLEMENTED | MIX-007 |  |
| MENU-706 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Unroute tracks routed to the selected track(s) | Routing | action |  |  | Mixer › ⏵ › Unroute tracks routed to the selected track(s) | IMPLEMENTED | MIX-007 |  |
| MENU-707 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Solo selected track(s) | Routing | action | `S` |  | Mixer › ⏵ › Solo selected track(s) | IMPLEMENTED | MIX-001 | Session 16: S toggles it. |
| MENU-708 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Alt solo selected track(s) | Routing | action | `Opt+S` |  | — | NOT_STARTED | MIX-001 |  |
| MENU-709 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Arm selected track(s) | Routing | action |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-710 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Disarm selected track(s) | Routing | action |  |  | — | NOT_STARTED | REC-001 |  |
| MENU-711 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Compensate for master latency | Routing | toggle |  | checked | Mixer › automatic delay compensation | DIFFERENT | MIX-012 | Always on. |
| MENU-712 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Separator | Group | toggle |  |  | — | NOT_STARTED |  |  |
| MENU-713 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Create group... | Group | dialog |  |  | — | NOT_STARTED |  |  |
| MENU-714 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Auto color group | Group | action |  | disabled | — | NOT_STARTED |  |  |
| MENU-715 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Dock to | Layout | submenu |  |  | — | NOT_STARTED |  |  |
| MENU-716 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Move left | Layout | action | `Opt+Left` |  | Mixer › ⏵ › Move left | IMPLEMENTED | MIX-006 | Session 16: channels, routes, automation and sidechains follow; default "Insert N" names follow the position. |
| MENU-717 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Move right | Layout | action | `Opt+Right` |  | Mixer › ⏵ › Move right | IMPLEMENTED | MIX-006 | Session 16: channels, routes, automation and sidechains follow; default "Insert N" names follow the position. |
| MENU-718 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Delete | Layout | action | `Del` | disabled | Mixer › ⏵ › Delete | IMPLEMENTED | MIX-006 | Disabled on the master. Session 16: its channels go to the master, its automation is deleted. |
| MENU-719 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Insert one | Layout | action |  | disabled | Mixer › ⏵ › Insert one | IMPLEMENTED | MIX-006 | Session 16: an empty track after the selected one. Add › Mixer track still adds at the end. |
| MENU-720 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Insert multiple... | Layout | dialog |  | disabled | — | NOT_STARTED |  |  |
| MENU-721 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Insert from state | Layout | submenu |  | disabled | — | NOT_STARTED |  |  |
| MENU-722 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Allow threaded processing | CPU | toggle |  | checked | — | NOT_STARTED |  |  |
| MENU-723 | Effect slot menu |  | submenu |  |  | Mixer › + Add effect | DIFFERENT | FX-001 | The ▸ of an empty insert slot: More plugins…, Plugin database (Shift+F8), FL's effects by category (Controller, Delay reverb, Distortion, Dynamics, Filter, Flanger, Gain, Misc, Patcher, Visual) and Categories / Simple / Tree. LimeShot's + Add effect lists its built-in effects and plugins. |
| MENU-724 | &nbsp;&nbsp;&nbsp;&nbsp;↳ <FL effects by category> |  | list |  |  | — | NOT_PLANNED |  | FL's own plugins. |
| MENU-725 | &nbsp;&nbsp;&nbsp;&nbsp;↳ Plugin database |  | action | `Shift+F8` |  | — | NOT_STARTED | APP-021 |  |

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
