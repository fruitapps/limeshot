# Settings: FL Studio and LimeShot

> Generated from [`settings.json`](settings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313) (Observed live in FL Studio 2026 on macOS: Options › settings window (every tab), the render dialog, the Plugin Manager and the Patterns dialogs (session 14, 2026-10-22; screenshots docs/research/evidence/screenshots/fl-dlg-*.png). Earlier records: https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm).

page = the settings tab or dialog; section = the header inside it; control = the widget; fl_default = the value seen on a default project. limeshot_setting names the field: settings.* = application settings (settings.json in the config folder), ui.* = per-session UI state, project.* = saved in the project, dialog.* = an option of a LimeShot dialog (not saved). Confidence as in keybindings.json.

Tracked: **193** — IMPLEMENTED 30, IN_PROGRESS 3, NOT_STARTED 145, NOT_PLANNED 15

## MIDI

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-009 | Output | Output port | list + port | no device | Options › MIDI settings (F10) › Output › Output port | OBSERVED | — | NOT_STARTED | WF-005 |  |
| SET-029 | Output | Send master sync | switch | on | Options › MIDI settings (F10) › Output › Send master sync | OBSERVED | — | NOT_STARTED | WF-012 |  |
| SET-030 | Output | Send all notes off | switch | on | Options › MIDI settings (F10) › Output › Send all notes off | OBSERVED | — | NOT_STARTED | WF-005 | Sent to the output when playback stops. |
| SET-031 | Output | Master sync offset (milliseconds) | field | 0 | Options › MIDI settings (F10) › Output › Master sync offset (milliseconds) | OBSERVED | — | NOT_STARTED | WF-012 |  |
| SET-032 | Output | Synchronization type | dropdown | MIDI clock | Options › MIDI settings (F10) › Output › Synchronization type | OBSERVED | — | NOT_STARTED | WF-012 |  |
| SET-008 | Input | Input devices and controller type | list + dropdown | no device | Options › MIDI settings (F10) › Input › Input devices and controller type | OBSERVED | `settings.midi_input` | IMPLEMENTED | APP-020 | Per device: Enable, Controller type (script), Port; a wrench opens the controller script. Session 16: one input at a time, played on the selected channel; no controller types. |
| SET-010 | Input | Link note on velocity to | dropdown | Velocity | Options › MIDI settings (F10) › Input › Link note on velocity to | OBSERVED | `settings.midi_velocity` | IMPLEMENTED | APP-020 | Session 16: Velocity or Nothing (a fixed 78 %). |
| SET-033 | Input | Link release velocity to | dropdown | Release | Options › MIDI settings (F10) › Input › Link release velocity to | OBSERVED | — | NOT_STARTED | WF-003 |  |
| SET-034 | Input | Pickup (takeover mode) | switch | off | Options › MIDI settings (F10) › Input › Pickup (takeover mode) | OBSERVED | — | NOT_STARTED | WF-004 |  |
| SET-035 | Input | Auto accept detected controller | switch | on | Options › MIDI settings (F10) › Input › Auto accept detected controller | OBSERVED | — | NOT_STARTED | WF-003 |  |
| SET-036 | Input | Foot pedal controls note off | switch | on | Options › MIDI settings (F10) › Input › Foot pedal controls note off | OBSERVED | — | NOT_STARTED | WF-003 |  |
| SET-037 | Input | Omni preview MIDI channel | field | --- | Options › MIDI settings (F10) › Input › Omni preview MIDI channel | OBSERVED | — | NOT_STARTED | WF-003 |  |
| SET-038 | Input | Song marker jump MIDI channel | field | --- | Options › MIDI settings (F10) › Input › Song marker jump MIDI channel | OBSERVED | — | NOT_STARTED | WF-003 |  |
| SET-039 | Input | Performance mode MIDI channel | field | --- | Options › MIDI settings (F10) › Input › Performance mode MIDI channel | OBSERVED | — | NOT_STARTED | PL-019 |  |
| SET-040 | Input | Generator muting MIDI channel | field | --- | Options › MIDI settings (F10) › Input › Generator muting MIDI channel | OBSERVED | — | NOT_STARTED | WF-003 | With 'Toggle on release' (off). |
| SET-041 | External sync | External clock sync | dropdown | Off | Options › MIDI settings (F10) › External sync › External clock sync | OBSERVED | — | NOT_STARTED | WF-012 | Status line below ('Status: off'). |
| SET-042 | External sync | External sync offset (milliseconds) | field | 0 | Options › MIDI settings (F10) › External sync › External sync offset (milliseconds) | OBSERVED | — | NOT_STARTED | WF-012 |  |
| SET-043 |  | Refresh device list | button |  | Options › MIDI settings (F10) › Refresh device list | OBSERVED | `settings.midi_input` | IMPLEMENTED | APP-020 | Session 16: re-reads the input list. |
| SET-044 |  | Update MIDI scripts | button |  | Options › MIDI settings (F10) › Update MIDI scripts | OBSERVED | — | NOT_PLANNED | WF-003 |  |
| SET-045 |  | Enable FL Studio Remote | switch | off | Options › MIDI settings (F10) › Enable FL Studio Remote | OBSERVED | — | NOT_PLANNED | APP-025 | Image-Line's phone/tablet remote. |

## Audio

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-001 | Input / output | Output device | dropdown | Default audio device | Options › Audio settings › Input / output › Output device | OBSERVED | `settings.audio.device` | IMPLEMENTED | APP-012 |  |
| SET-004 | Input / output | Input device / ASIO inputs | dropdown | Default input device | Options › Audio settings › Input / output › Input device / ASIO inputs | OBSERVED | `settings.audio_input` | IMPLEMENTED | REC-003 | Session 16: the input recorded from (default or a named device); no ASIO channel pairs. |
| SET-002 | Input / output | Sample rate | dropdown | 48000 (device) | Options › Audio settings › Input / output › Sample rate | OBSERVED | `settings.audio.sample_rate` | IMPLEMENTED | APP-012 |  |
| SET-007 | Input / output | Release the device when inactive | switch | off | Options › Audio settings › Input / output › Release the device when inactive | OBSERVED | — | NOT_STARTED | APP-012 | FL: 'Auto close'. |
| SET-046 | Input / output | Device status and latency | info |  | Options › Audio settings › Input / output › Device status and latency | OBSERVED | `settings.audio.device` | IMPLEMENTED | APP-012 | FL: 'Open, set to 48000Hz, 1x2 outputs, 0x2 inputs available' and input/output/plugin latency in samples and ms. LimeShot's Settings › Audio status shows the open device, rate, buffer and plugin delay compensation. |
| SET-003 | Input / output | Buffer length | slider | 128smp (3ms) | Options › Audio settings › Input / output › Buffer length | OBSERVED | `settings.audio.buffer_size` | IMPLEMENTED | APP-012 |  |
| SET-047 | Input / output | Priority | dropdown | Highest | Options › Audio settings › Input / output › Priority | OBSERVED | — | NOT_STARTED | APP-012 | Audio thread priority; an underrun counter sits on the same row. |
| SET-048 | Input / output | Safe overloads | switch | on | Options › Audio settings › Input / output › Safe overloads | OBSERVED | — | NOT_STARTED | APP-012 |  |
| SET-006 | Input / output | Playback tracking source | dropdown | Mixer | Options › Audio settings › Input / output › Playback tracking source | OBSERVED | — | NOT_STARTED | TR-006 |  |
| SET-049 | Input / output | Playback tracking offset | slider | centre | Options › Audio settings › Input / output › Playback tracking offset | OBSERVED | — | NOT_STARTED | TR-006 |  |
| SET-050 | CPU | Multithreaded generator processing | switch | on | Options › Audio settings › CPU › Multithreaded generator processing | OBSERVED | — | NOT_STARTED | APP-012 |  |
| SET-051 | CPU | Multithreaded mixer processing | switch | on | Options › Audio settings › CPU › Multithreaded mixer processing | OBSERVED | — | NOT_STARTED | APP-012 |  |
| SET-052 | CPU | Smart disable | switch | on | Options › Audio settings › CPU › Smart disable | OBSERVED | — | NOT_STARTED | WF-009 |  |
| SET-053 | CPU | Align tick lengths | switch | off | Options › Audio settings › CPU › Align tick lengths | OBSERVED | — | NOT_STARTED | TR-007 |  |
| SET-005 | Mixer | Resampling quality | dropdown | 24-point sinc | Options › Audio settings › Mixer › Resampling quality | OBSERVED | — | NOT_STARTED | INS-002 | LimeShot resamples linearly. |
| SET-054 | Mixer | Browser preview track | field | --- | Options › Audio settings › Mixer › Browser preview track | OBSERVED | — | NOT_STARTED | BR-002 | Mixer track the Browser preview plays through. |
| SET-055 | Mixer | Metronome track | field | --- | Options › Audio settings › Mixer › Metronome track | OBSERVED | — | NOT_STARTED | TR-008 |  |
| SET-056 | Mixer | Play truncated notes on transport | switch | on | Options › Audio settings › Mixer › Play truncated notes on transport | OBSERVED | — | NOT_STARTED | TR-002 |  |
| SET-057 | Mixer | Reset plugins on transport | switch | off | Options › Audio settings › Mixer › Reset plugins on transport | OBSERVED | — | NOT_STARTED | PLG-002 |  |

## General

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-011 | Display (requires a restart to take effect) | UI scaling | dropdown | System | Options › General settings › Display (requires a restart to take effect) › UI scaling | OBSERVED | `settings.ui_scale` | IMPLEMENTED | UI-005 | Settings › General › Display: 75–200 %; also Cmd+= / Cmd+- / Cmd+0. Applies at once (FL needs a restart). |
| SET-058 | Display (requires a restart to take effect) | Pop-ups scaling | dropdown | Main | Options › General settings › Display (requires a restart to take effect) › Pop-ups scaling | OBSERVED | — | NOT_STARTED | UI-005 |  |
| SET-059 | Display (requires a restart to take effect) | Toolbars scaling | dropdown | Main | Options › General settings › Display (requires a restart to take effect) › Toolbars scaling | OBSERVED | — | NOT_STARTED | UI-005 |  |
| SET-060 | Display (requires a restart to take effect) | Transparent windows | dropdown | Off | Options › General settings › Display (requires a restart to take effect) › Transparent windows | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-061 | Display (requires a restart to take effect) | Animations | dropdown | Make it pretty | Options › General settings › Display (requires a restart to take effect) › Animations | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-062 | Display (requires a restart to take effect) | Smoothing | knob | low | Options › General settings › Display (requires a restart to take effect) › Smoothing | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-063 | Display (requires a restart to take effect) | Animation refresh rate | dropdown | Ultrasmooth | Options › General settings › Display (requires a restart to take effect) › Animation refresh rate | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-064 | Display (requires a restart to take effect) | Force refreshes | switch | on | Options › General settings › Display (requires a restart to take effect) › Force refreshes | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-065 | Display (requires a restart to take effect) | Transparent menus | switch | on | Options › General settings › Display (requires a restart to take effect) › Transparent menus | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-066 | Display (requires a restart to take effect) | Thick lines | switch | off (disabled) | Options › General settings › Display (requires a restart to take effect) › Thick lines | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-067 | Display (requires a restart to take effect) | High visibility | switch | off | Options › General settings › Display (requires a restart to take effect) › High visibility | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-068 | Display (requires a restart to take effect) | Colorful waveforms | dropdown | Off | Options › General settings › Display (requires a restart to take effect) › Colorful waveforms | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-069 | Language (requires a restart to take effect) | Language | dropdown | English (US) (en) | Options › General settings › Language (requires a restart to take effect) › Language | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-070 | Language (requires a restart to take effect) | Note names | dropdown | English | Options › General settings › Language (requires a restart to take effect) › Note names | OBSERVED | — | NOT_STARTED | PR-001 |  |
| SET-071 | Language (requires a restart to take effect) | Black notes | dropdown | Sharps - # | Options › General settings › Language (requires a restart to take effect) › Black notes | OBSERVED | — | NOT_STARTED | PR-001 |  |
| SET-072 | Language (requires a restart to take effect) | Update languages | button |  | Options › General settings › Language (requires a restart to take effect) › Update languages | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-073 | Input | Optimize for | dropdown | Multi-button mouse | Options › General settings › Input › Optimize for | OBSERVED | — | NOT_STARTED | WF-007 |  |
| SET-016 | Input | Keyboard shortcuts (remapping) | button |  | Options › General settings › Input › Keyboard shortcuts (remapping) | OBSERVED | — | NOT_STARTED | UI-011 | FL: 'Manage shortcuts'. |
| SET-074 | Input | Click-and-hold & special gesture functions | switch | off | Options › General settings › Input › Click-and-hold & special gesture functions | OBSERVED | — | NOT_STARTED | WF-007 |  |
| SET-075 | Input | Use both mouse buttons in step sequencer | switch | off | Options › General settings › Input › Use both mouse buttons in step sequencer | OBSERVED | — | NOT_STARTED | SEQ-001 |  |
| SET-076 | Input | Vertical scrolling speed | slider | middle | Options › General settings › Input › Vertical scrolling speed | OBSERVED | — | NOT_STARTED | PR-005 |  |
| SET-077 | Input | Horizontal scrolling speed | slider | middle | Options › General settings › Input › Horizontal scrolling speed | OBSERVED | — | NOT_STARTED | PR-005 |  |
| SET-012 | Undo history | Undo history levels | field | 100 | Options › General settings › Undo history › Undo history levels | OBSERVED | — | NOT_STARTED | APP-005 | FL: 'Maximum undo levels'. |
| SET-078 | Undo history | Undo knob tweaks | switch | on | Options › General settings › Undo history › Undo knob tweaks | OBSERVED | — | NOT_STARTED | APP-005 | LimeShot always records control gestures as one undo step each; there is no switch. |
| SET-079 | Undo history | Alternate undo mode | switch | on | Options › General settings › Undo history › Alternate undo mode | OBSERVED | — | NOT_STARTED | APP-005 | Ctrl+Z steps back through history instead of toggling. |
| SET-080 | Undo history | Put undone recordings in the recycle bin | switch | on | Options › General settings › Undo history › Put undone recordings in the recycle bin | OBSERVED | — | NOT_STARTED | REC-008 |  |
| SET-081 | Undo history | Show undo history in the browser when undoing changes | switch | off | Options › General settings › Undo history › Show undo history in the browser when undoing changes | OBSERVED | — | NOT_STARTED | APP-006 |  |
| SET-082 | Undo history | Show recent changes at the top | switch | off | Options › General settings › Undo history › Show recent changes at the top | OBSERVED | — | NOT_STARTED | APP-006 |  |
| SET-013 | Hints and warnings | Hint bar and tooltips | button |  | Options › General settings › Hints and warnings › Hint bar and tooltips | OBSERVED | — | NOT_STARTED | UI-004 | FL: 'Manage messages' (which warnings show); the hint bar itself is View › Toolbars › Hint bar. |
| SET-083 | Hints and warnings | Project data size warning | dropdown | 100MB (default) | Options › General settings › Hints and warnings › Project data size warning | OBSERVED | — | NOT_STARTED | APP-007 |  |
| SET-084 | Updates and privacy | Automatically check for updates | switch | on | Options › General settings › Updates and privacy › Automatically check for updates | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-085 | Updates and privacy | Automatically download program updates | switch | on | Options › General settings › Updates and privacy › Automatically download program updates | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-086 | Updates and privacy | Send usage data | switch | on | Options › General settings › Updates and privacy › Send usage data | OBSERVED | — | NOT_PLANNED | APP-025 | LimeShot collects no usage data. |
| SET-087 | Miscellaneous | Auto name channels | switch | on | Options › General settings › Miscellaneous › Auto name channels | OBSERVED | — | NOT_STARTED | SEQ-002 |  |
| SET-088 | Miscellaneous | Auto name effect slots | switch | off | Options › General settings › Miscellaneous › Auto name effect slots | OBSERVED | — | NOT_STARTED | MIX-007 |  |
| SET-089 | Miscellaneous | Auto zip empty channels | switch | off | Options › General settings › Miscellaneous › Auto zip empty channels | OBSERVED | `settings.auto_zip_empty_channels` | IMPLEMENTED | SEQ-013 | Saved application setting, off by default. Pattern switches collapse channels without note data and expand populated ones (including muted notes). Enabling waits for the next switch; manual restoration remains until then. Headless interaction tests listed under SEQ-013. |
| SET-090 | Miscellaneous | Auto select linked modules | switch | on | Options › General settings › Miscellaneous › Auto select linked modules | OBSERVED | — | NOT_STARTED | SEQ-005 |  |
| SET-091 | Miscellaneous | Auto zoom in piano roll | switch | off | Options › General settings › Miscellaneous › Auto zoom in piano roll | OBSERVED | — | NOT_STARTED | PR-005 |  |
| SET-092 | Miscellaneous | Small scrollbars in editors | switch | off | Options › General settings › Miscellaneous › Small scrollbars in editors | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-093 | Miscellaneous | Detach all new plugins | switch | off | Options › General settings › Miscellaneous › Detach all new plugins | OBSERVED | — | NOT_STARTED | PLG-006 |  |
| SET-094 | Miscellaneous | Open detached plugins with the FL Studio window | switch | off | Options › General settings › Miscellaneous › Open detached plugins with the FL Studio window | OBSERVED | — | NOT_STARTED | PLG-006 |  |
| SET-095 | Miscellaneous | Disable typing keyboard priority by default | switch | off | Options › General settings › Miscellaneous › Disable typing keyboard priority by default | OBSERVED | — | NOT_STARTED | WF-002 |  |
| SET-096 | Miscellaneous | Alternate meter scale | switch | off | Options › General settings › Miscellaneous › Alternate meter scale | OBSERVED | — | NOT_STARTED | MIX-005 |  |
| SET-097 | Miscellaneous | Show startup splash screen | switch | off (disabled) | Options › General settings › Miscellaneous › Show startup splash screen | OBSERVED | — | NOT_STARTED | APP-023 |  |
| SET-098 | Miscellaneous | Restore previous state after solo | switch | off | Options › General settings › Miscellaneous › Restore previous state after solo | OBSERVED | — | NOT_STARTED | MIX-004 |  |
| SET-099 | Miscellaneous | Hide plugin window toolbar by default | switch | on | Options › General settings › Miscellaneous › Hide plugin window toolbar by default | OBSERVED | — | NOT_STARTED | PLG-006 |  |
| SET-100 | Miscellaneous | Silent startup | switch | off | Options › General settings › Miscellaneous › Silent startup | OBSERVED | — | NOT_STARTED | APP-023 |  |
| SET-101 | Miscellaneous | Monophonic painting in piano roll | switch | off | Options › General settings › Miscellaneous › Monophonic painting in piano roll | OBSERVED | — | NOT_STARTED | PR-012 |  |
| SET-102 | Miscellaneous | Create automation clips at the playback position | switch | on | Options › General settings › Miscellaneous › Create automation clips at the playback position | OBSERVED | — | NOT_STARTED | AUT-003 |  |
| SET-103 | Miscellaneous | Select first channel with notes when the pattern changes | switch | off | Options › General settings › Miscellaneous › Select first channel with notes when the pattern changes | OBSERVED | — | NOT_STARTED | SEQ-018 |  |
| SET-104 | Miscellaneous | Limit the number of open plugin editors | switch | on | Options › General settings › Miscellaneous › Limit the number of open plugin editors | OBSERVED | — | NOT_STARTED | PLG-006 |  |
| SET-105 | Miscellaneous | Default template | dropdown | Basic with limiter | Options › General settings › Miscellaneous › Default template | OBSERVED | — | NOT_STARTED | APP-015 |  |
| SET-106 | Miscellaneous | Startup project | dropdown | Default template | Options › General settings › Miscellaneous › Startup project | OBSERVED | — | NOT_STARTED | APP-015 |  |
| SET-107 | Advanced | Don't limit windows to screen | switch | off | Options › General settings › Advanced › Don't limit windows to screen | OBSERVED | — | NOT_STARTED | UI-007 |  |
| SET-108 | Advanced | Fast sample preview | switch | on | Options › General settings › Advanced › Fast sample preview | OBSERVED | — | NOT_STARTED | BR-002 |  |
| SET-109 | Advanced | Auto keep long audio on disk | switch | on | Options › General settings › Advanced › Auto keep long audio on disk | OBSERVED | `settings.auto_keep_long_audio_on_disk` | IMPLEMENTED | INS-016 | Default on. Long host imports/previews use a bounded disk cache up to 2 GB decoded stereo float audio; native Sampler also offers per-source Keep on disk. |
| SET-110 | Advanced | Read sample tempo information | switch | on | Options › General settings › Advanced › Read sample tempo information | OBSERVED | — | NOT_STARTED | INS-001 |  |
| SET-111 | Advanced | Read sample root note | switch | on | Options › General settings › Advanced › Read sample root note | OBSERVED | — | NOT_STARTED | INS-001 |  |
| SET-112 | Advanced | Initialize controls automatically | switch | on | Options › General settings › Advanced › Initialize controls automatically | OBSERVED | — | NOT_STARTED | AUT-011 |  |
| SET-113 | Advanced | Force high performance power plan | switch | on | Options › General settings › Advanced › Force high performance power plan | OBSERVED | — | NOT_STARTED | APP-012 |  |
| SET-114 | Advanced | Threaded project sample loading | switch | off | Options › General settings › Advanced › Threaded project sample loading | OBSERVED | — | NOT_STARTED | APP-002 |  |
| SET-115 | Advanced | Cursors scaling | dropdown | System | Options › General settings › Advanced › Cursors scaling | OBSERVED | — | NOT_STARTED | UI-005 |  |
| SET-116 | Advanced | Legacy scaling | dropdown | Main | Options › General settings › Advanced › Legacy scaling | OBSERVED | — | NOT_STARTED | UI-005 |  |
| SET-117 | Advanced | Show help in a web browser | dropdown |  | Options › General settings › Advanced › Show help in a web browser | OBSERVED | — | NOT_STARTED | APP-032 |  |
| SET-014 |  | Show the Welcome window |  |  | Welcome window › Do not show this in the future | OBSERVED | — | NOT_STARTED | APP-023 |  |

## File

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-019 | Backup | Backups / autosave | dropdown | Occasionally (every 10 minutes) | Options › File settings › Backup › Backups / autosave | OBSERVED | `settings.autosave_minutes` | IMPLEMENTED | APP-024 | Settings › File › Backup: Off / every 1, 2, 5 or 10 minutes (FL's choices differ). |
| SET-118 | Backup | Maximum backups | field | 100 | Options › File settings › Backup › Maximum backups | OBSERVED | — | NOT_STARTED | APP-016 |  |
| SET-119 | Backup | Backup location | dropdown | Save to the project data folder if there is one | Options › File settings › Backup › Backup location | OBSERVED | — | NOT_STARTED | APP-016 |  |
| SET-018 | Plugins | Plugin search folders | button |  | Options › File settings › Plugins › Plugin search folders | OBSERVED | `settings.vst3_folders` | IMPLEMENTED | APP-021 | Settings › File › VST3 plugin search folders: add/remove extra roots; platform defaults always searched. Find plugins refreshes the database via background discovery and per-bundle child scans. VST3 only; no per-root enable switch, format selection or separate Plugin Manager. Settings round-trip, discovery/worker and headless removal tests cover local behavior; live picker/rescan and platform checks pending. |
| SET-120 | User data folder | User data folder | path + Reset + browse | ~/Documents/Image-Line/ | Options › File settings › User data folder › User data folder | OBSERVED | — | NOT_STARTED | BR-001 |  |
| SET-017 | Browser extra search folders | Extra Browser folders | table (Folder, Browser name) | empty | Options › File settings › Browser extra search folders › Extra Browser folders | OBSERVED | `settings.browser_folders` | IMPLEMENTED | BR-001 | Settings › File: listed with × to remove and Add folder…; also the Browser's + button. |
| SET-121 | External tools | External tools | table (File, Name) | empty | Options › File settings › External tools › External tools | OBSERVED | — | NOT_PLANNED | APP-025 | Options Launch with path / Launch with file / Default editor / Launch at startup. |
| SET-122 | External tools | Script editor | path | TextEdit.app | Options › File settings › External tools › Script editor | OBSERVED | — | NOT_STARTED | WF-008 |  |
| SET-123 | Troubleshooting | Set to current | button |  | Options › File settings › Troubleshooting › Set to current | OBSERVED | — | NOT_PLANNED | APP-025 | Points FL's plugin version at this install. |
| SET-124 | Troubleshooting | Repair optional features | button |  | Options › File settings › Troubleshooting › Repair optional features | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-125 | Troubleshooting | Reset browser cache | button |  | Options › File settings › Troubleshooting › Reset browser cache | OBSERVED | — | NOT_STARTED | BR-001 |  |
| SET-126 | Troubleshooting | Reset FL Cloud cache | button |  | Options › File settings › Troubleshooting › Reset FL Cloud cache | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-020 | Troubleshooting | Recent files | list |  | File menu › Recent projects | OBSERVED | `settings.recent` | IMPLEMENTED | APP-010 | Settings › File › Recent files lists them, with Clear list; also File › Recent projects. |

## Theme

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-015 | Presets | Theme / colours | gallery | (Default) | Options › Theme settings › Presets › Theme / colours | OBSERVED | — | NOT_STARTED | UI-006 | Preset thumbnails: (Default), DEDZII Light, DEDZII, PPRIIME Dark, PPRIIME Light, Veela, Ash, Dark orange, Dark, then light variants. |
| SET-127 | Adjustments | Hue (theme adjustment) | knob | centre | Options › Theme settings › Adjustments › Hue (theme adjustment) | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-128 | Adjustments | Saturation (theme adjustment) | knob | centre | Options › Theme settings › Adjustments › Saturation (theme adjustment) | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-129 | Adjustments | Brightness (theme adjustment) | knob | centre | Options › Theme settings › Adjustments › Brightness (theme adjustment) | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-130 | Adjustments | Contrast (theme adjustment) | knob | centre | Options › Theme settings › Adjustments › Contrast (theme adjustment) | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-131 | Adjustments | Text (theme adjustment) | knob | centre | Options › Theme settings › Adjustments › Text (theme adjustment) | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-132 | Adjustments | Accent colours (Selection, Highlight, Mute, Option, Steps, Meters, Waves, Text) | colour swatches |  | Options › Theme settings › Adjustments › Accent colours (Selection, Highlight, Mute, Option, Steps, Meters, Waves, Text) | OBSERVED | — | NOT_STARTED | UI-003 |  |
| SET-133 | Options | Light mode | switch | off | Options › Theme settings › Options › Light mode | OBSERVED | — | NOT_STARTED | UI-006 |  |
| SET-134 | Options | Audio and automation clips use note colors | switch | off | Options › Theme settings › Options › Audio and automation clips use note colors | OBSERVED | — | NOT_STARTED | PL-014 |  |
| SET-135 |  | Theme A/B compare, Reset, Save preset… | buttons | A | Options › Theme settings › Theme A/B compare, Reset, Save preset… | OBSERVED | — | NOT_STARTED | UI-006 |  |

## Project

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-023 | Data folder | Project data folder | dropdown | Don't use a data folder | Options › Project general settings › Data folder › Project data folder | OBSERVED | — | NOT_STARTED | APP-009 | Rename and browse buttons, current folder shown ('(none)'). |
| SET-136 | Data folder | Ask for data folder | dropdown | When saving unsaved projects only | Options › Project general settings › Data folder › Ask for data folder | OBSERVED | — | NOT_STARTED | APP-028 |  |
| SET-137 | Data folder | Copy used samples to the project folder | switch | off (disabled) | Options › Project general settings › Data folder › Copy used samples to the project folder | OBSERVED | — | NOT_STARTED | EXP-017 |  |
| SET-022 | Time settings | Time base (PPQ) | dropdown | 96 | Options › Project general settings › Time settings › Time base (PPQ) | OBSERVED | — | NOT_STARTED | TR-007 | LimeShot always uses 960 PPQ. |
| SET-138 | Time settings | Time signature | dropdown + Numerator + Denominator | Set as time signature, 4/4 | Options › Project general settings › Time settings › Time signature | OBSERVED | `project.time_signature` | IMPLEMENTED | TR-005 | LimeShot sets the time signature on the toolbar. |
| SET-139 | FL Cloud | Back up to FL Cloud storage | switch | off | Options › Project general settings › FL Cloud › Back up to FL Cloud storage | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-140 | Advanced | Panning law | dropdown | Circular | Options › Project general settings › Advanced › Panning law | OBSERVED | — | NOT_STARTED | SEQ-004 | LimeShot uses one fixed pan law (limeshot-dsp). |
| SET-141 | Advanced | Play truncated notes in clips | switch | on | Options › Project general settings › Advanced › Play truncated notes in clips | OBSERVED | — | NOT_STARTED | PL-001 |  |
| SET-142 | Advanced | Fast declick for cut groups | switch | off | Options › Project general settings › Advanced › Fast declick for cut groups | OBSERVED | — | NOT_STARTED | INS-005 |  |
| SET-143 | Playlist | Automatic crossfades | switch | on | Options › Project general settings › Playlist › Automatic crossfades | OBSERVED | — | NOT_STARTED | PL-023 |  |
| SET-144 | Playlist | Enable automatic crossfades in new projects | switch | on | Options › Project general settings › Playlist › Enable automatic crossfades in new projects | OBSERVED | — | NOT_STARTED | PL-023 |  |
| SET-145 | FL Studio Remote | Linked layout | field + Send to devices |  | Options › Project general settings › FL Studio Remote › Linked layout | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-024 | FL Studio Remote | Main pitch | knob | 0 | Toolbar › Master pitch | OBSERVED | — | NOT_STARTED | TR-013 | Not on the Project page; FL has it on the toolbar. |

## Info

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-021 |  | Project info (title, author, genre, comments) | fields | empty | Options › Project info (F11) › Project info (title, author, genre, comments) | OBSERVED | `project.info` | IMPLEMENTED | APP-017 | Settings › Info (F11): Title, Author, Genre, Comments, Web link; typing in a field is one undo step. Saved in the .lime. |
| SET-146 |  | Show info on opening | switch | off | Options › Project info (F11) › Show info on opening | OBSERVED | — | NOT_STARTED | APP-017 |  |
| SET-147 |  | Project statistics | info |  | Options › Project info (F11) › Project statistics | OBSERVED | `project.info` | IMPLEMENTED | APP-033 | Counts of plugins, channels, mixer tracks, patterns, Playlist tracks and clips, and notes under the Info fields. No dates or working time (APP-033). |
| SET-148 |  | Reset working time | button |  | Options › Project info (F11) › Reset working time | OBSERVED | — | NOT_STARTED | APP-033 |  |

## Debug

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-149 |  | Debugging log | text log |  | Options › Debugging log › Debugging log | OBSERVED | — | NOT_STARTED | APP-031 | Read-only log: project loading, mixer threading maps, startup time, MIDI devices, browser refreshes, autosaves. |

## Account

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-150 |  | Account settings | page |  | Options › Account settings › Account settings | OBSERVED | — | NOT_PLANNED | APP-025 |  |

## About

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-151 |  | About page | page |  | Settings › About › About page | OBSERVED | — | IN_PROGRESS | APP-032 | Settings › About: version, licence and the trademark note (Help › About LimeShot opens it). |

## Render

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-152 | Project type | Mode | dropdown | Pattern (in Pattern mode) | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Project type › Mode | OBSERVED | `dialog.range` | IMPLEMENTED | EXP-004 | Full song / Pattern. FL takes the sample rate from Audio settings; LimeShot's export dialog also picks the sample rate (dialog.sample_rate). |
| SET-153 | Project type | Tail | dropdown | Leave remainder | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Project type › Tail | OBSERVED | — | NOT_STARTED | EXP-013 | Cut remainder / Leave remainder / Wrap remainder. |
| SET-154 | Project type | Length, total time and disk space | info |  | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Project type › Length, total time and disk space | OBSERVED | — | NOT_STARTED | EXP-001 |  |
| SET-155 | Output format | Output formats | multi-select (WAV, MP3, OGG, FLAC, MID, M4A) | WAV | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › Output formats | OBSERVED | `dialog.mp3` | IN_PROGRESS | EXP-018 | One of WAV or MP3 per render; MIDI separately. Simultaneous multi-format export remains open. |
| SET-156 | Output format | WAV bit depth | dropdown | 16Bit int | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › WAV bit depth | OBSERVED | `dialog.depth` | IMPLEMENTED | EXP-003 | 16Bit int / 24Bit int / 32Bit float. |
| SET-157 | Output format | Channels | dropdown | Stereo | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › Channels | OBSERVED | — | NOT_STARTED | EXP-018 | Stereo / Mono (merged) / Mono (left only) / Mono (right only). |
| SET-158 | Output format | MP3 bitrate | slider | 192kbps | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › MP3 bitrate | OBSERVED | `dialog.bitrate_kbps` | IMPLEMENTED | EXP-005 | CBR dropdown: all MPEG-1 bitrates 32–320 kbps, default 320. No VBR. |
| SET-159 | Output format | OGG bitrate | slider | 192kbps | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › OGG bitrate | OBSERVED | — | NOT_STARTED | EXP-006 |  |
| SET-160 | Output format | FLAC bit depth | dropdown | 16Bit int | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › FLAC bit depth | OBSERVED | — | NOT_STARTED | EXP-006 |  |
| SET-161 | Output format | FLAC compression | slider | level 5 | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Output format › FLAC compression | OBSERVED | — | NOT_STARTED | EXP-006 |  |
| SET-162 | Quality | Resampling | dropdown | 32-point sinc | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Quality › Resampling | OBSERVED | — | NOT_STARTED | EXP-001 | Low quality / fastest: 2-point linear, 6-point Hermite; Average: 16-, 24-, 32-point sinc; High quality / slowest: 64-, 128-, 256-point sinc … |
| SET-163 | Quality | HQ for all plugins | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Quality › HQ for all plugins | OBSERVED | — | NOT_STARTED | PLG-002 |  |
| SET-164 | Quality | Disable maximum polyphony | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Quality › Disable maximum polyphony | OBSERVED | — | NOT_STARTED | INS-005 |  |
| SET-165 | Quality | Dithering | switch | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Quality › Dithering | OBSERVED | — | NOT_STARTED | EXP-007 |  |
| SET-166 | Miscellaneous | Save playlist markers | switch | on (disabled) | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Save playlist markers | OBSERVED | — | NOT_STARTED | PL-016 |  |
| SET-167 | Miscellaneous | Save loop markers | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Save loop markers | OBSERVED | — | NOT_STARTED | PL-016 |  |
| SET-168 | Miscellaneous | Save note markers | switch | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Save note markers | OBSERVED | — | NOT_STARTED | PR-022 |  |
| SET-169 | Miscellaneous | Save tempo information | switch | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Save tempo information | OBSERVED | — | NOT_STARTED | EXP-011 |  |
| SET-170 | Miscellaneous | Split mixer tracks | switch | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Split mixer tracks | OBSERVED | — | NOT_STARTED | EXP-009 |  |
| SET-171 | Miscellaneous | Trim PDC silence | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Trim PDC silence | OBSERVED | — | NOT_STARTED | MIX-012 | LimeShot always trims the compensation delay from exports (no switch). |
| SET-172 | Miscellaneous | Enable insert effects | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Enable insert effects | OBSERVED | — | NOT_STARTED | FX-001 |  |
| SET-173 | Miscellaneous | Enable master effects | switch | on | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Miscellaneous › Enable master effects | OBSERVED | — | NOT_STARTED | FX-001 |  |
| SET-174 |  | Upload to 3rd party service… | switch + section | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Upload to 3rd party service… | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-175 |  | Show file(s) when complete | switch | off | File › Export › <format> file… (save panel, then 'Rendering to <file>') › Show file(s) when complete | OBSERVED | — | NOT_STARTED | EXP-001 |  |

## Plugin Manager

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-176 |  | Find installed plugins | button |  | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Find installed plugins | OBSERVED | — | NOT_STARTED | PLG-004 | LimeShot: 'Find plugins…' in the plugin menus. |
| SET-177 |  | Verify plugins | switch | on | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Verify plugins | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-178 | Options | FL Studio folder | path | /Applications/FL Studio 2026.app/Contents/Libs/… | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Options › FL Studio folder | OBSERVED | — | NOT_PLANNED | APP-025 |  |
| SET-179 | Options | Plugin search paths | list (format, path, checkbox) | VST3, VST, CLAP, FL folders | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Options › Plugin search paths | OBSERVED | `settings.vst3_folders` | IN_PROGRESS | PLG-004 | Settings › File › VST3 plugin search folders: add/remove extra roots; platform defaults always searched. Find plugins refreshes the database via background discovery and per-bundle child scans. VST3 only; no per-root enable switch, format selection or separate Plugin Manager. Settings round-trip, discovery/worker and headless removal tests cover local behavior; live picker/rescan and platform checks pending. |
| SET-180 | Scan options | Rescan previously verified plugins | switch | off | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Scan options › Rescan previously verified plugins | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-181 | Scan options | Rescan plugins with errors | switch | off | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Scan options › Rescan plugins with errors | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-182 | Scan options | Try to prevent popup windows | switch | on | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Scan options › Try to prevent popup windows | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-183 | Scan options | Combine VST and VST3 versions of a plugin | switch | on | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Scan options › Combine VST and VST3 versions of a plugin | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-184 | Scan options | Always rescan Waves plugins | switch | on | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Scan options › Always rescan Waves plugins | OBSERVED | — | NOT_STARTED | PLG-004 |  |
| SET-185 | Options | Always on top | switch | off | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Options › Always on top | OBSERVED | — | NOT_STARTED | APP-021 |  |
| SET-186 |  | Plugin list | table (F, Name, Status, Format, Kind, Effect/Synth, Vendor, Filename) |  | Options › Manage plugins (separate app 'FL Studio Plugin Manager') › Plugin list | OBSERVED | — | NOT_STARTED | APP-021 | F = favourite star; footer shows the count ('315 plugins') and a Find field. Plugin tab (left) shows the selected plugin's details. |

## Patterns dialogs

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-187 |  | Time signature change | Numerator, Denominator, Accept | 4/4 | Patterns › Set time signature… | OBSERVED | — | NOT_STARTED | PR-022 | Opens the Piano Roll with a small dialog on top. |
| SET-188 |  | Transpose | value field (Semitones) with ✓ / ✗ | 0 | Patterns › Transpose… | OBSERVED | — | NOT_STARTED | SEQ-021 | A small inline field at the pointer, not a window. |

## Session

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-025 |  | Metronome |  |  | Toolbar toggle (Ctrl+M) | VERIFIED | `ui.metronome` | IMPLEMENTED | TR-008 | Not persisted between sessions. |
| SET-026 |  | Loop song |  |  | Toolbar / Options | INFERRED | `ui.song_loop` | IMPLEMENTED | TR-009 | Not persisted between sessions. |
| SET-027 |  | Snap |  |  | Toolbar snap selector | OBSERVED | `ui.snap` | IMPLEMENTED | PR-003 | Not persisted between sessions. |

## Dialogs

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-028 |  | Confirm channel deletion |  |  | Confirm dialog › Remember my choice | OBSERVED | `settings.confirm_channel_delete` | IMPLEMENTED | SEQ-007 | Settings › General › Hints and warnings: 'Ask before deleting channels'. |

## Options

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-189 | Switches | Auto scrolling | switch |  | Options › Auto scrolling; toolbar Auto scrolling button (session 15) | OBSERVED | `settings.auto_scroll` | IMPLEMENTED | UI-016 | Off by default. Shared Options/toolbar switch, saved with application settings; no project edit. |

## Mixer

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-190 | Input / Recording pickup | Monitor external input | Off / When armed / On |  | https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/recording_audio.htm | VERIFIED | `ui.record.monitor` | IMPLEMENTED | REC-003 | Documented in the FL manual; also available on linked Playlist Audio Tracks. FL runtime defaults unobserved. LimeShot: Settings › Audio › Monitor external input; session-local. See docs/research/recording-monitoring-and-count-in.md. |

## Playlist

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-191 | Track mode › Audio track | Mixer track assignment | Mixer insert selector |  | https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/recording_audio.htm | VERIFIED | `ui.record.monitor_track` | IMPLEMENTED | REC-003 | Documented in the FL manual: link a Playlist Audio Track to a Mixer insert; hardware input is selected on the Mixer track. FL runtime choices/defaults unobserved. LimeShot: Settings › Audio › Monitor Mixer track selects the single input monitoring destination by position; session-local, not per-track hardware I/O. See docs/research/recording-monitoring-and-count-in.md. |

## Toolbar

| ID | Section | Setting | Control | FL default | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| SET-192 | Recording switches | Recording precount | switch |  | https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/toolbar_panels.htm | VERIFIED | `ui.record.count_in` | IMPLEMENTED | REC-007 | Manual-based evidence; FL runtime defaults unobserved. Session-local. See docs/research/recording-monitoring-and-count-in.md. |
| SET-193 | Recording switches | Recording precount bars | bar selector (FL choices unobserved) |  | https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/toolbar_panels.htm | VERIFIED | `ui.record.count_in_bars` | IMPLEMENTED | REC-007 | Documented in the FL manual: right-click Recording precount to choose bars. Exact FL choices and runtime defaults unobserved. LimeShot offers one/two bars; session-local. See docs/research/recording-monitoring-and-count-in.md. |
