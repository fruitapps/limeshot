# Settings: FL Studio and LimeShot

> Generated from [`settings.json`](settings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.
> Reference: FL Studio 2026 (26.1.1.5313) (https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/envsettings_audio.htm).

limeshot_setting names the field: settings.* = application settings (settings.json in the config folder), ui.* = per-session UI state, project.* = saved in the project. Confidence as in keybindings.json.

Tracked: **28** — IMPLEMENTED 11, IN_PROGRESS 1, NOT_STARTED 15, NOT_PLANNED 1

## Audio

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-001 | Output device | Options › Audio settings › Device | VERIFIED | `settings.audio.device` | IMPLEMENTED | APP-012 |  |
| SET-002 | Sample rate | Audio settings › Sample rate | VERIFIED | `settings.audio.sample_rate` | IMPLEMENTED | APP-012 |  |
| SET-003 | Buffer length | Audio settings › Buffer length | VERIFIED | `settings.audio.buffer_size` | IMPLEMENTED | APP-012 |  |
| SET-004 | Input device / ASIO inputs | Audio settings › Input | INFERRED | — | NOT_STARTED | REC-003 |  |
| SET-005 | Resampling quality | Audio settings › Mixer resampling | INFERRED | — | NOT_STARTED | INS-002 | LimeShot resamples linearly. |
| SET-006 | Playback tracking source | Audio settings › Playback tracking (Driver / Mixer) | INFERRED | — | NOT_STARTED | TR-006 |  |
| SET-007 | Release the device when inactive | Audio settings › Auto close device | INFERRED | — | NOT_STARTED | APP-012 |  |

## MIDI

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-008 | Input devices and controller type | Options › MIDI settings › Input | VERIFIED | — | NOT_STARTED | APP-020 |  |
| SET-009 | Output port | MIDI settings › Output | INFERRED | — | NOT_STARTED | WF-005 |  |
| SET-010 | Link note-on velocity to | MIDI settings | INFERRED | — | NOT_STARTED | WF-003 |  |

## General

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-011 | UI scaling | Options › General settings › UI scale | INFERRED | `settings.ui_scale` | IMPLEMENTED | UI-005 | Also Cmd+= / Cmd+- / Cmd+0. |
| SET-012 | Undo history levels | General settings › Undo history | INFERRED | — | NOT_STARTED | APP-005 | LimeShot keeps a fixed number of steps (core::document). |
| SET-013 | Hint bar and tooltips | General settings › Hints | INFERRED | — | NOT_STARTED | UI-004 | Always on. |
| SET-014 | Show the Welcome window | Welcome window › Do not show this in the future | OBSERVED | — | NOT_STARTED | APP-023 |  |
| SET-015 | Theme / colours | General settings › Theme | INFERRED | — | NOT_STARTED | UI-006 |  |
| SET-016 | Keyboard shortcuts (remapping) | — | INFERRED | — | NOT_STARTED | UI-011 | FL does not offer full remapping; tracked as a LimeShot feature. |

## File

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-017 | Extra Browser folders | Options › File settings › Browser extra search folders | VERIFIED | `settings.browser_folders` | IMPLEMENTED | BR-001 |  |
| SET-018 | Plugin search folders | Options › Manage plugins | VERIFIED | — | NOT_STARTED | APP-021 |  |
| SET-019 | Backups / autosave | File settings › Backup (Browser › Backup folder observed) | OBSERVED | `settings.autosave_minutes` | IMPLEMENTED | APP-024 | Options › Autosave in LimeShot. |
| SET-020 | Recent files | File menu › Recent files | VERIFIED | `settings.recent` | IMPLEMENTED | APP-010 | Last 10 projects. |

## Project

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-021 | Project info (title, author, genre, comments) | Options › Project info | VERIFIED | — | IN_PROGRESS | APP-017 | The title exists (project.title); the other fields do not. |
| SET-022 | Time base (PPQ) | Project general settings › Timebase | INFERRED | — | NOT_PLANNED | TR-007 | LimeShot always uses 960 PPQ. |
| SET-023 | Project data folder | Project general settings › Data folder | INFERRED | — | NOT_STARTED | APP-009 | Samples are referenced relative to the project file. |
| SET-024 | Main pitch | Project general settings / toolbar | INFERRED | — | NOT_STARTED | TR-013 |  |

## Session

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-025 | Metronome | Toolbar toggle (Ctrl+M) | VERIFIED | `ui.metronome` | IMPLEMENTED | TR-008 | Not persisted between sessions. |
| SET-026 | Loop song | Toolbar / Options | INFERRED | `ui.song_loop` | IMPLEMENTED | TR-009 | Not persisted between sessions. |
| SET-027 | Snap | Toolbar snap selector | OBSERVED | `ui.snap` | IMPLEMENTED | PR-003 | Not persisted between sessions. |

## Dialogs

| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |
|---|---|---|---|---|---|---|---|
| SET-028 | Confirm channel deletion | Confirm dialog › Remember my choice | OBSERVED | `settings.confirm_channel_delete` | IMPLEMENTED | SEQ-007 | Turned off by ticking "Remember my choice"; no way to turn it back on yet except editing settings.json (APP-025). |
