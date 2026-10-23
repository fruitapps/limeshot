# First-release feature checklist

**2026-10-23 replacement update:** this checklist/estimate retains its 272-record historical baseline. The current
inventory has 290 records. Active instruments/effects now require bundled Lime VST3s; the in-process implementations
are deleted. Hosted parity PLG-017…020, PLG-022 and PLG-024…029 is implemented; release gates now require broader native/platform
validation of those restored paths. See [current handoff](../development/handoff-hosted-p0-parity.md).
Prior baseline VERIFIED labels do not establish these replacement behaviors. See
[known limitations](known-limitations.md) and the [current inventory](features.md).

Companion to the [human-readable roadmap and estimates](first-release.md). Baseline: remote `main` at
`dd53fb317085b58816c9c8bafb0071b92aa94dde`, 22 October 2026. All **272** baseline feature records appear exactly once.
This is a proposed v0.1 scope, not a record of completed implementation. Baseline status is copied unchanged.

## How to read this checklist

- **Included:** existing behavior belongs in v0.1, subject to release verification.
- **Bounded:** only the explicitly stated subset belongs in v0.1; the rest of the broad feature title is later.
- **Finish/add:** implementation or release validation must be completed; it is included in the roadmap estimate.
- **Experimental:** may be available with disclosed restrictions; it is not a supported release promise.
- **Later:** outside v0.1. A priority label alone does not make it a release commitment.

Every included/bounded item still needs the applicable release gates. These labels are release-scope decisions,
not substitutes for NOT_STARTED / IMPLEMENTED / VERIFIED. The implementation inventory remains canonical.

| Release disposition | Records |
| --- | ---: |
| Included | 144 |
| Bounded | 28 |
| Finish/add | 14 |
| Experimental | 2 |
| Later | 84 |
| **Total** | **272** |

The totals measure records, not engineering effort. “Finish/add” includes hardware/platform verification of existing
code. Generic manager/browser records with CLAP dependencies are deferred; existing VST3 functionality is included
under PLG-004/008. AUT-009 overlaps PLG-007 and is not an additional VST3 automation implementation task.

## Product-wide requirements without separate baseline records

The release also requires clean-machine installation, file associations, media portability, missing-plugin/sample
errors, a published support matrix, no data-loss blockers, callback/queue stress checks, real-device and real-plugin
acceptance, an independent export check, a beta period and accurate documentation/licenses. These are budgeted
in the roadmap phases. They do not become “verified” merely because adjacent inventory features have tests.

Only explicitly saved custom layouts are promised. Full screen-reader support for painted editors, automatic
layout persistence, native Windows/Linux plugin views and full FL compatibility are not first-release promises.

## Application

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| APP-001 | New project | VERIFIED | Included | Start an empty project. |
| APP-002 | Open project | VERIFIED | Included | Load a .lime project. |
| APP-003 | Save / Save as (atomic) | VERIFIED | Included | Persist the project safely. |
| APP-004 | Modified-state detection | VERIFIED | Included | Know when there are unsaved changes. |
| APP-005 | Undo / redo | VERIFIED | Included | Undo and redo project edits. |
| APP-006 | Undo history list | IMPLEMENTED | Included | Browse past edits. |
| APP-007 | Versioned project format with migrations | VERIFIED | Included | Forward-compatible file format. |
| APP-008 | Preserve unknown project data | VERIFIED | Included | Do not silently drop data from newer files. |
| APP-009 | Relative asset references | VERIFIED | Included | Projects stay portable with their samples. |
| APP-010 | Recent files | VERIFIED | Included | Reopen recent projects. |
| APP-011 | Application settings persistence | VERIFIED | Included | Remember preferences. |
| APP-012 | Audio device settings | VERIFIED | Included | Choose output device, sample rate and buffer size. |
| APP-013 | Keyboard shortcuts for core actions | VERIFIED | Included | Shortcuts for transport, windows and editing. |
| APP-014 | Ask to save on close | VERIFIED | Included | Never lose work on quit. |
| APP-015 | Project templates | NOT_STARTED | Later | Use the bundled demo and empty project for onboarding; templates remain later. |
| APP-016 | Autosave and crash recovery | VERIFIED | Included | Periodic backup and recovery. |
| APP-017 | Project info (title, author, comments) | IMPLEMENTED | Included | Song metadata. |
| APP-018 | Save new version | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-019 | Project bundle (zip with samples) | NOT_STARTED | Finish/add | Portable project bundle with referenced samples and recorded media; reopen without original paths. |
| APP-020 | MIDI settings | IN_PROGRESS | Bounded | One selected MIDI input and velocity setting; no MIDI out, controller profiles or sync. |
| APP-021 | Plugin manager | NOT_STARTED | Later | The existing VST3 scanner/database ships under PLG-004; a broader format-independent manager remains later. |
| APP-022 | Import FL Studio projects (.flp) | VERIFIED | Bounded | Experimental, lossy .flp import: supported channels/notes/first arrangement; warn about substitutions and dropped data. No .flp export. |
| APP-023 | Welcome window | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-024 | Autosave interval setting | VERIFIED | Included | Choose how often to autosave. |
| APP-025 | Settings window | IMPLEMENTED | Included | All application settings in one window with pages. |
| APP-026 | Main menu bar in FL's layout | IN_PROGRESS | Bounded | Working existing main/editor menus; missing advanced actions stay absent or disabled. |
| APP-027 | Revert to last backup | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-028 | Save to new project folder | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-029 | Delete unused files and save project | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-030 | Project general settings | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-031 | Debugging log and diagnostics | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| APP-032 | Help index, tutorials and about | IN_PROGRESS | Finish/add | Ship quick-start/manual, About, shortcut help and version-specific release notes. |
| APP-033 | Project statistics and working time | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Channel Rack

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| SEQ-001 | Step sequencing | VERIFIED | Included | Edit and play step-triggered patterns. |
| SEQ-002 | Add sampler / synth channels | VERIFIED | Included | Create channels. |
| SEQ-003 | Channel mute and solo | VERIFIED | Included | Silence or isolate channels. |
| SEQ-004 | Channel volume and pan | VERIFIED | Included | Per-channel level and position. |
| SEQ-005 | Channel → mixer track routing | VERIFIED | Included | Choose which insert a channel feeds. |
| SEQ-006 | Pattern length (auto / manual) | VERIFIED | Included | Length of a pattern loop. |
| SEQ-007 | Rename, recolor, clone, delete, reorder channels | VERIFIED | Included | Channel management. |
| SEQ-008 | Piano-roll preview in rack | VERIFIED | Included | Show non-step notes in the rack. |
| SEQ-009 | Channel settings window | VERIFIED | Included | Edit instrument parameters. |
| SEQ-010 | Swing (global and per channel) | VERIFIED | Bounded | Global swing only; per-channel swing remains later. |
| SEQ-011 | Graph editor (per-step properties) | VERIFIED | Bounded | Velocity/pan graph lanes only; additional note properties remain later. |
| SEQ-012 | Channel filter groups | VERIFIED | Included | Show subsets of channels. |
| SEQ-013 | Zip / unzip channels | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| SEQ-014 | Fill / rotate / advanced fill | VERIFIED | Bounded | Fill each 2/4/8 steps and rotate; no Advanced fill dialog. |
| SEQ-015 | Copy / paste steps between channels and patterns | VERIFIED | Included | Clipboard for steps. |
| SEQ-016 | Channel step looping / ghost steps | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| SEQ-017 | Loop Starter (FL Cloud) | DEFERRED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| SEQ-018 | Multiple patterns | VERIFIED | Included | Several reusable patterns. |
| SEQ-019 | Audio clip channels | VERIFIED | Included | Channels created for audio clips. |
| SEQ-020 | Channel selection (selector LEDs) | VERIFIED | Included | Select one or several channels; channel operations act on the selection. |
| SEQ-021 | Pattern menu commands | VERIFIED | Included | FL's Patterns menu: find first/next empty pattern, select in playlist, random color, transpose, insert one, move up/down, split by channel. |
| SEQ-022 | Pattern groups | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Piano Roll

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| PR-001 | Add, move, resize, delete notes | VERIFIED | Included | Core note editing. |
| PR-002 | Note selection (click, rectangle, select all) | VERIFIED | Included | Select notes. |
| PR-003 | Snap to grid | VERIFIED | Included | Quantised editing. |
| PR-004 | Velocity editing | VERIFIED | Included | Per-note velocity. |
| PR-005 | Zoom and scroll | VERIFIED | Included | Navigate the grid. |
| PR-006 | Piano-key auditioning | VERIFIED | Included | Hear notes by clicking keys. |
| PR-007 | Playback position display | VERIFIED | Included | Playhead in the editor. |
| PR-008 | Duplicate selection (Ctrl+B) | VERIFIED | Included | Repeat notes to the right. |
| PR-009 | Transpose and nudge with keys | VERIFIED | Included | Keyboard editing. |
| PR-010 | Ghost notes | VERIFIED | Included | Other channels' notes faintly visible. |
| PR-011 | Channel target selector | VERIFIED | Included | Choose the channel being edited. |
| PR-012 | Tool palette: draw, paint, delete, mute, slice, select, zoom, playback | VERIFIED | Included | Additional tools. |
| PR-013 | Scale highlighting / snap to scale | IMPLEMENTED | Bounded | Scale highlighting only; snap-to-scale and saved scale remain later. |
| PR-014 | Quantize | VERIFIED | Included | Quantise note starts/lengths. |
| PR-015 | Chords, arpeggiator, strum, humanize, legato | IMPLEMENTED | Bounded | Existing simplified note tools/dialogs, with audition/accept/cancel and undo; no full FL option parity. |
| PR-016 | Note properties: pan, release, mod X/Y, fine pitch | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-017 | Slide and portamento notes | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-018 | Note color groups | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-019 | Copy / cut / paste notes | VERIFIED | Included | Clipboard. |
| PR-020 | MIDI file import / export | VERIFIED | Bounded | Notes/tempo/time-signature MIDI interchange; validate in an independent application. |
| PR-021 | MIDI recording into the Piano Roll | VERIFIED | Included | Record played notes. |
| PR-022 | Time signature per pattern / time markers | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-023 | Glue / slice notes | VERIFIED | Included | Join or split notes. |
| PR-024 | Resize from the left edge | VERIFIED | Included | Trim note starts. |
| PR-025 | Waveform helper background | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-026 | New notes take the last note's length | VERIFIED | Included | Default length of drawn notes. |
| PR-027 | Stamp tool (chords and scales) | IMPLEMENTED | Included | Draw chords or scales with one click. |
| PR-028 | Chord progression tool | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PR-029 | Riff machine | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Playlist

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| PL-001 | Pattern clips (linked instances) | VERIFIED | Included | Place patterns on the timeline. |
| PL-002 | Move clips (time and track) | VERIFIED | Included | Drag clips. |
| PL-003 | Resize clips | VERIFIED | Included | Change clip length. |
| PL-004 | Delete clips | VERIFIED | Included | Remove clips. |
| PL-005 | Duplicate clips (Ctrl+B) | VERIFIED | Included | Repeat clips to the right. |
| PL-006 | Select clips | VERIFIED | Included | Selection. |
| PL-007 | Snap clips to grid | VERIFIED | Included | Quantised placement. |
| PL-008 | Zoom, scroll, bar/beat ruler | VERIFIED | Included | Timeline navigation. |
| PL-009 | Playhead and set song position | VERIFIED | Included | See and set position. |
| PL-010 | Song mode drives the engine | VERIFIED | Included | Playlist plays through the engine. |
| PL-011 | Make unique | VERIFIED | Included | Detach a clip from shared pattern. |
| PL-012 | Playlist tracks: add, rename, mute | VERIFIED | Included | Track management. |
| PL-013 | Clip mute | VERIFIED | Included | Mute individual clips. |
| PL-014 | Audio clips | VERIFIED | Included | Place audio on the timeline. |
| PL-015 | Automation clips | VERIFIED | Included | Parameter envelopes on the timeline. |
| PL-016 | Markers and loop region / time selection | VERIFIED | Bounded | Existing named markers and Playlist time selection/looping; no advanced marker types. |
| PL-017 | Slip, slice clips | VERIFIED | Bounded | Existing slip and vertical slice; no ripple, diagonal slice or zero-cross snapping. |
| PL-018 | Track grouping | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PL-019 | Performance mode | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PL-020 | Arrangements | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PL-021 | Picker panel | VERIFIED | Included | List of clip sources. |
| PL-022 | Consolidate / render to audio clip | IN_PROGRESS | Bounded | Existing merge-to-pattern only; rendering/consolidation to audio remains later. |
| PL-023 | Crossfades and time stretching | IN_PROGRESS | Bounded | Manual linear clip fade-in/out only; automatic crossfades and time stretching remain later. |
| PL-024 | Clip offset (start inside pattern) | IN_PROGRESS | Finish/add | Finish and verify discoverable clip-offset editing and save/reload behavior. |
| PL-025 | Pattern picker | VERIFIED | Included | List of patterns beside the Playlist. |
| PL-026 | Copy / cut / paste clips | IMPLEMENTED | Included | Clipboard for clips. |
| PL-027 | Picker lists audio and automation clips | VERIFIED | Included | All clip sources in the clip picker. |
| PL-028 | Automation STEP / SLIDE display options | RESEARCHED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PL-030 | Stem separation (Remix a song) | DEFERRED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Mixer

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| MIX-001 | Master and insert tracks | VERIFIED | Included | Mixing structure. |
| MIX-002 | Volume faders | VERIFIED | Included | Track level. |
| MIX-003 | Pan | VERIFIED | Included | Stereo position. |
| MIX-004 | Mute and solo | VERIFIED | Included | Silence/isolate tracks. |
| MIX-005 | Level meters and clip indicator | VERIFIED | Included | See levels. |
| MIX-006 | Rename, add, move and delete mixer tracks | VERIFIED | Included | Track management. |
| MIX-007 | Insert effect slots | VERIFIED | Included | Per-track effect chains. |
| MIX-008 | Routing and sends between tracks | VERIFIED | Included | Bus routing. |
| MIX-009 | Sidechain routing | VERIFIED | Included | Sidechain sends. |
| MIX-010 | Built-in 3-band EQ | VERIFIED | Included | Track EQ. |
| MIX-011 | Stereo separation, phase invert, swap L/R | IN_PROGRESS | Bounded | Width/polarity through Utility; per-strip controls and L/R swap remain later. |
| MIX-012 | Plugin delay compensation | VERIFIED | Bounded | Existing bounded PDC; test and publish capacity/overflow behavior. |
| MIX-013 | Insert-to-insert routing | VERIFIED | Included | Route inserts to buses. |
| MIX-014 | Current track (C) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| MIX-015 | Mixer presets / track states | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| MIX-016 | External audio I/O per track | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Instruments

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| INS-001 | Sampler: WAV loading and playback | VERIFIED | Included | Play samples. |
| INS-002 | Sampler: gain, pitch, start/end, reverse, release | VERIFIED | Included | Sampler parameters. |
| INS-003 | LimeSynth: oscillators | VERIFIED | Included | Built-in synthesizer. |
| INS-004 | LimeSynth: ADSR envelope | VERIFIED | Included | Amplitude envelope. |
| INS-005 | Polyphony and voice stealing | VERIFIED | Bounded | Existing 128-voice pool/voice stealing; validate audible behavior and publish the limit. |
| INS-006 | Synth filter and modulation (LFO, filter env) | VERIFIED | Included | Subtractive shaping. |
| INS-007 | Sampler loop points, time stretch, filter | IN_PROGRESS | Bounded | Sampler loop points and low-pass filter only; no time stretching, ping-pong or loop crossfades. |
| INS-008 | Additional audio formats (MP3, FLAC, OGG, AIFF) | VERIFIED | Bounded | WAV/MP3/FLAC/Ogg Vorbis/AIFF input; no AAC/M4A and no promise of matching export formats. |
| INS-009 | Multi-sample / drum kit instrument | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| INS-010 | Instrument presets | IMPLEMENTED | Included | Save/load instrument settings. |
| INS-011 | Per-channel polyphony limit / mono / porta | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| INS-012 | Drum synthesizer | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| INS-013 | LimeSynth filter modes and key tracking | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| INS-014 | Sampler volume envelope | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Effects

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| FX-001 | Effect processing framework (insert chain) | VERIFIED | Included | Effect processing framework (insert chain). |
| FX-002 | Parametric equalizer | VERIFIED | Included | Parametric equalizer. |
| FX-003 | Filters (LP/HP/BP) | VERIFIED | Included | Filters (LP/HP/BP). |
| FX-004 | Compressor | VERIFIED | Included | Compressor. |
| FX-005 | Limiter / maximiser | VERIFIED | Included | Limiter / maximiser. |
| FX-006 | Noise gate | VERIFIED | Included | Noise gate. |
| FX-007 | Reverb | VERIFIED | Included | Reverb. |
| FX-008 | Delay | VERIFIED | Included | Delay. |
| FX-009 | Distortion / saturation / waveshaper | VERIFIED | Included | Distortion / saturation / waveshaper. |
| FX-010 | Chorus / flanger / phaser | VERIFIED | Included | Chorus / flanger / phaser. |
| FX-011 | Pitch shifting | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| FX-012 | Stereo processing | NOT_STARTED | Later | Existing Utility ships under FX-013; additional stereo-processing effects remain later. |
| FX-013 | Utility (gain, phase, balance, mono) | VERIFIED | Included | Utility (gain, phase, balance, mono). |
| FX-014 | Effect presets | IMPLEMENTED | Included | Effect presets. |
| FX-015 | Analysers (spectrum, oscilloscope) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Automation

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| AUT-001 | Automation clips with envelope points | VERIFIED | Included | Automation clips with envelope points. |
| AUT-002 | Curve shapes and interpolation | VERIFIED | Bounded | Existing tension curves and implemented AUT-012 point modes; no exact FL curve equivalence claim. |
| AUT-003 | Automate mixer and channel parameters | VERIFIED | Bounded | Supported channel/mixer volume/pan and effect parameters in Song mode; no track-EQ or synth/sampler automation. |
| AUT-004 | Record automation from control movements | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-005 | Event editor | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-006 | Parameter linking to controllers | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-007 | LFO / formula / peak controllers | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-008 | Tempo automation | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-009 | Plugin parameter automation | NOT_STARTED | Later | Overlaps implemented VST3 automation in PLG-007; reconcile the record. Broader automation remains later. |
| AUT-010 | Automation clip channels in the Channel Rack | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-011 | Init song with this position | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| AUT-012 | Automation point modes | IMPLEMENTED | Included | Per-segment curve types. |
| AUT-013 | Automated controls follow their automation | VERIFIED | Bounded | Channel/mixer controls follow Song playback; hosted plugin editors reflect their own values. |

## Recording

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| REC-001 | Audio recording from inputs | IMPLEMENTED | Finish/add | Real-input validation and failure handling for WAV recording and Playlist take placement. |
| REC-002 | MIDI recording | VERIFIED | Included | MIDI recording. |
| REC-003 | Input selection and monitoring | IN_PROGRESS | Finish/add | Finish input selection, level display and basic software monitoring; verify with latency compensation. |
| REC-004 | Punch in/out | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| REC-005 | Loop recording / takes | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| REC-006 | Recording latency compensation | NOT_STARTED | Finish/add | Basic measured input/output latency alignment for recorded audio; publish device limits. |
| REC-007 | Count-in | NOT_STARTED | Finish/add | Configurable basic count-in for MIDI/audio recording. |
| REC-008 | Recorded clip management | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| REC-009 | Step edit mode | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| REC-010 | Start on input and blend recorded notes | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| REC-011 | Audio logger | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Browser

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| BR-001 | Folder navigation (bundled + user folders) | VERIFIED | Included | Browse sample folders. |
| BR-002 | Sample preview | VERIFIED | Included | Audition samples. |
| BR-003 | Load sample into channel / new channel | VERIFIED | Included | Use samples. |
| BR-004 | Search / filter | VERIFIED | Included | Find samples. |
| BR-005 | Current project tree | IMPLEMENTED | Included | Project contents. |
| BR-006 | Favorites / starred | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| BR-007 | Tags | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| BR-008 | Plugin and preset browser | NOT_STARTED | Later | Existing plugin preset menus ship under PLG-008; integrated Browser plugin/preset browsing remains later. |
| BR-009 | Online content | DEFERRED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| BR-010 | Drag samples to the Playlist (audio clips) | VERIFIED | Included | Place audio directly. |
| BR-011 | Save clipboard to browser | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Plugins

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| PLG-001 | CLAP hosting | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PLG-002 | VST3 hosting | VERIFIED | Included | VST3 hosting. |
| PLG-003 | Audio Unit hosting (macOS) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| PLG-004 | Plugin scanning | IMPLEMENTED | Included | Plugin scanning. |
| PLG-005 | Plugin state persistence | VERIFIED | Included | Plugin state persistence. |
| PLG-006 | Plugin editor windows | IMPLEMENTED | Bounded | Implemented macOS native embedding needs broader live checks; Windows/Linux use the generic parameter editor. |
| PLG-007 | Plugin parameter automation | VERIFIED | Bounded | VST3 parameter automation; no claim of automation for unimplemented plugin formats. |
| PLG-008 | Plugin preset management | IMPLEMENTED | Bounded | Factory programs and .vstpreset files; no .fst compatibility or preset-load undo promise. |
| PLG-009 | Plugin latency reporting | VERIFIED | Included | Plugin latency reporting. |
| PLG-010 | Plugin crash isolation | VERIFIED | Experimental | Existing Unix bridge needs realtime transport investigation; no supported bridge claim until fixed/validated. No Windows bridge or bridged native editor commitment. |
| PLG-011 | Plugin performance monitor | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Export

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| EXP-001 | WAV export (offline bounce) | VERIFIED | Included | Render the song to a file. |
| EXP-002 | Sample-rate selection | VERIFIED | Included | Choose export rate. |
| EXP-003 | Bit-depth selection | VERIFIED | Included | Choose export format. |
| EXP-004 | Pattern export | VERIFIED | Included | Export only the current pattern. |
| EXP-005 | MP3 export | NOT_STARTED | Later | MP3 sample input is included; MP3 encoding/export remains later. |
| EXP-006 | FLAC / OGG export | NOT_STARTED | Later | FLAC/Ogg sample input is included; encoding/export remains later. |
| EXP-007 | Dithering | NOT_STARTED | Finish/add | Optional dither for integer PCM output; float output remains undithered. |
| EXP-008 | Normalization | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-009 | Stem export (per mixer track) | NOT_STARTED | Finish/add | Per-mixer-track WAV stems with documented routing/send/tail semantics. |
| EXP-010 | Export ranges (time selection, loop) | VERIFIED | Included | Partial export. |
| EXP-011 | File metadata | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-012 | MIDI export | VERIFIED | Included | Export notes. |
| EXP-013 | Wrap the tail into the loop | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-014 | Export playlist tracks or track groups | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-015 | Export in the last used format(s) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-016 | Video export | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-017 | Project bones and project data files | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| EXP-018 | Several formats and mono in one render | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Transport & Time

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| TR-001 | Realtime audio output | VERIFIED | Included | Hear the project. |
| TR-002 | Play / pause / stop | VERIFIED | Included | Transport. |
| TR-003 | Pattern / Song mode | VERIFIED | Included | Choose what plays. |
| TR-004 | Tempo | VERIFIED | Included | Project tempo. |
| TR-005 | Time signature | VERIFIED | Included | Bar length. |
| TR-006 | Position display | VERIFIED | Included | Where playback is. |
| TR-007 | Exact musical timing (ticks ↔ samples) | VERIFIED | Included | No timing drift. |
| TR-008 | Metronome | VERIFIED | Included | Click track. |
| TR-009 | Song loop toggle | VERIFIED | Included | Loop or stop at song end. |
| TR-010 | Panic (all notes off) | IMPLEMENTED | Included | Stop all sound. |
| TR-011 | Audio load and voice count display | IMPLEMENTED | Included | Performance feedback. |
| TR-012 | Tap tempo | VERIFIED | Included | Set tempo by tapping. |
| TR-013 | Master pitch | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Workflow

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| WF-001 | Headless command scripting (CLI) | IMPLEMENTED | Included | Automate LimeShot without the GUI. |
| WF-002 | Typing keyboard to piano | VERIFIED | Included | Play notes from the computer keyboard. |
| WF-003 | MIDI controller input | VERIFIED | Included | Play with hardware. |
| WF-004 | Controller mapping (link to controller) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-005 | External MIDI devices / MIDI out | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-006 | Multichannel / surround output | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-007 | Accessibility (screen readers, keyboard nav) | IN_PROGRESS | Bounded | Existing standard-widget accessibility only; custom editor screen-reader coverage remains incomplete and disclosed. |
| WF-008 | Python-style scripting API | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-009 | Project macros | IN_PROGRESS | Bounded | Existing working project macros only; no complete FL macro set. |
| WF-010 | Score logger | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-011 | Touch controller | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-012 | MIDI clock sync | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| WF-013 | Item groups | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## UI

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| UI-001 | Main window layout (toolbar, browser, workspace) | VERIFIED | Included | FL-style workspace. |
| UI-002 | Workspace windows: toggle, maximise, arrange | VERIFIED | Included | Window management. |
| UI-003 | LimeShot theme tokens and drawn icons | VERIFIED | Included | Visual identity. |
| UI-004 | Hint bar and tooltips | VERIFIED | Included | Discoverability. |
| UI-005 | UI scaling / HiDPI | IMPLEMENTED | Included | Readable on all displays. |
| UI-006 | Additional themes (light, high contrast) | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| UI-007 | Detachable windows (multi-monitor) | VERIFIED | Included | Windows outside the main window. |
| UI-008 | Offscreen UI rendering for review/regression | VERIFIED | Included | Visual checks without a display. |
| UI-009 | Channel Settings window | VERIFIED | Included | Docked property panel. |
| UI-010 | Control context menu | IN_PROGRESS | Bounded | Existing automation context actions plus finished reset/copy/paste/type-value workflow; no full FL context-menu parity. |
| UI-011 | Customizable keyboard shortcuts | NOT_STARTED | Finish/add | Remappable shortcuts, conflict handling and persisted settings. |
| UI-012 | Soft detent at a control's default | VERIFIED | Included | Dials, faders and sliders catch at their default value. |
| UI-013 | Window layout commands | IMPLEMENTED | Bounded | Existing explicit custom-layout save/restore; automatic session layout restoration remains later. |
| UI-014 | Toolbar customisation and presets | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| UI-015 | Workspace background | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| UI-016 | Auto scrolling option | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| UI-017 | Control creator | DEFERRED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |
| UI-018 | Plugin picker | NOT_STARTED | Later | Outside v0.1; see the inventory for dependencies and acceptance criteria. |

## Platform

| ID | Feature | Baseline status | v0.1 scope | Included behavior / remaining boundary |
| --- | --- | --- | --- | --- |
| PF-001 | macOS Apple Silicon | VERIFIED | Included | Native build and run. |
| PF-002 | macOS Intel | IMPLEMENTED | Finish/add | Verify real Intel Mac GUI/audio/plugins; Rosetta/headless evidence is insufficient. |
| PF-003 | Windows x86-64 | IN_PROGRESS | Finish/add | Windows x86-64 package and real GUI/audio/MIDI/recording tests. |
| PF-004 | Linux x86-64 | IN_PROGRESS | Finish/add | Linux x86-64 package and real GUI/audio/MIDI/recording tests on the published support matrix. |
| PF-005 | Windows ARM64 / Linux ARM64 | IN_PROGRESS | Experimental | Windows/Linux ARM64 build targets only; no supported runtime release commitment. |
| PF-006 | Packaging (DMG, MSI, AppImage/deb) | IN_PROGRESS | Finish/add | Distributable packages, resources/licenses, signing policy and file-open handling on supported OSes. |

## Scope changes after this baseline

The 272-record baseline predates the hosted rewrite. Building, packaging and validating the twelve bundled Lime
VST3s is now required for DAW v0.1 playback; independent standalone plugin distribution is a separate deliverable.
Off-main work must be reviewed and merged before it changes the historical baseline. New requirements should get
inventory records and an explicit scope/estimate decision; do not silently grow v0.1 to include every later item.
