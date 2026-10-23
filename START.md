# LimeShot — original product and development brief

> This document preserves the project requirements and initial execution plan. It is not a current status report.
> D39 supersedes the original built-in playback requirement: instruments/effects now use the twelve Lime Native VST3 bundles.
> Start ongoing work with [AGENTS.md](AGENTS.md), [next steps](docs/development/next-steps.md) and the
> [feature inventory](docs/roadmap/features.md). Completed research and milestones do not need to be restarted.

## 0. Your Role and Mission

You are acting as a senior Rust systems engineer, digital audio workstation architect, DSP engineer, cross-platform desktop application developer, UI/UX designer, and reverse-engineering specialist.

Your task is to **design and build LimeShot**, a new, open-source, cross-platform digital audio workstation (DAW) inspired primarily by Image-Line's FL Studio.

LimeShot should reproduce the familiar workflow, major capabilities, layout principles, and usability of FL Studio while maintaining its own identity, implementation, branding, and visual design.

The project should also be architecturally and philosophically inspired by the ArtCraft application suite:

https://github.com/storytold

**This is a real software development assignment, not a design exercise.**

You must create a working, maintainable Rust application with actual music-production functionality.

Do not stop at mockups, plans, screenshots, empty interfaces, or nonfunctional feature placeholders.

Your work should follow this general process:

1. Research the ArtCraft projects and their engineering conventions.
2. Install, configure, and actively use the REA reverse-engineering skill.
3. Investigate the locally installed FL Studio application.
4. Document the functional and visual design findings.
5. Establish a complete feature inventory and development roadmap.
6. Design LimeShot's architecture.
7. Implement a functioning native application.
8. Implement the core music-production workflow.
9. Verify functionality with automated and manual tests.
10. Document the current implementation state and outstanding work.

Work autonomously and make reasonable engineering decisions where requirements are not explicitly specified.

**Do not ask for permission at every implementation step.** Only request user intervention when genuinely required for security, licensing, installation privileges, missing reference software, or irreversible actions.

---

# 1. Project Identity

**Application name:** LimeShot

**Application category:** Digital Audio Workstation / Music Production Software

**Primary reference application:** FL Studio

**Architectural inspiration:** ArtCraft applications

**Implementation language:** Rust

**Target platforms:**
- macOS (Apple Silicon and Intel)
- Windows (x86-64, with ARM64 where feasible)
- Linux (x86-64, with ARM64 where feasible)

All three primary operating systems must be treated as first-class development targets from the beginning.

**Application model:** Native desktop only.

**Development philosophy:**
- Open source
- Offline-first
- Local-first
- Cross-platform
- Modular
- Performance-oriented
- Extensible
- Non-destructive editing wherever appropriate
- Keyboard-friendly
- Accessible and responsive
- Reliable enough for real creative work

## Non-negotiable constraints

1. The application must be written in Rust.
2. Rust must own the actual audio engine, project model, application behavior, and UI.
3. No Electron, Tauri, React, JavaScript frontend, browser UI, or embedded webview.
4. No browser or WebAssembly distribution target.
5. No hosted backend, account system, cloud dependency, or mandatory network service.
6. No copying proprietary FL Studio source code, binary resources, branding, or copyrighted assets.
7. The architecture must support real-time audio production, not merely simulated playback.
8. The project must remain independently maintained and implemented.
9. Do not copy code from ArtCraft repositories. Study their architecture and conventions, then implement LimeShot independently.
10. Document limitations honestly. Never treat an unimplemented feature as complete.

Development tools may depend on non-Rust software when necessary, such as REA, build tools, or operating-system SDKs. However, LimeShot's application code and first-party runtime components must remain Rust. Platform APIs and Rust FFI bindings are permissible when required for native audio, MIDI, or plugin interoperability.

---

# 2. Mandatory Reference Repositories

Investigate the following sources before choosing an architecture.

## 2.1 ArtCraft

Organization:

https://github.com/storytold

Important repositories:

- https://github.com/storytold/artcraft
- https://github.com/storytold/soundcraft
- https://github.com/storytold/photocraft
- https://github.com/storytold/filmcraft
- https://github.com/storytold/effectcraft
- https://github.com/storytold/craft-launcher

You should also inspect additional ArtCraft repositories if they demonstrate better solutions to specific architectural problems.

Pay particularly close attention to **SoundCraft**, which already implements significant DAW functionality in Rust.

However, LimeShot must be a separate implementation.

Do not fork SoundCraft or build LimeShot by renaming it.

SoundCraft is an architectural reference, not a codebase to duplicate.

## 2.2 What to investigate

Inspect actual source code, Cargo manifests, documentation, and repository structure.

Do not draw architectural conclusions based solely on README files.

Investigate:

- Rust workspace organization
- Crate boundaries
- Dependency management
- Native application bootstrap
- Window lifecycle
- UI framework choices
- UI rendering and custom widgets
- Dockable panel architecture
- Application state management
- Command dispatch
- Undo and redo systems
- Keyboard shortcut handling
- Project serialization
- Native file dialogs
- Settings persistence
- Audio engine separation
- MIDI architecture
- Audio plugin hosting
- Real-time threading
- Cross-platform abstractions
- Agent-controllable application commands
- Test organization
- CI workflows
- Packaging and distribution
- Documentation standards
- Feature tracking and development status conventions

Identify which patterns are appropriate for LimeShot and which are not.

Do not assume that every ArtCraft architectural choice is automatically appropriate for a latency-sensitive DAW.

## 2.3 Required architectural research output

Create:

`docs/research/artcraft-architecture.md`

Include:

- Repositories inspected
- Relevant paths and source files
- Useful engineering patterns
- Rejected patterns
- Reasons for each architectural decision
- Differences between SoundCraft and LimeShot
- Implications for cross-platform development

Record source commit hashes where possible to make the investigation reproducible.

---

# 3. Mandatory REA Skill Integration

You must use:

**REA — Reverse Engineer Anything**

Repository:

https://github.com/morluto/rea

REA must be actively incorporated into the research and implementation workflow.

It is not sufficient to mention REA in documentation.

## 3.1 Installation and verification

First inspect the official REA documentation for its current requirements, installation instructions, supported tools, and operational limitations.

The documented setup entry point is:

`npx rea-agents setup`

Use the official, current installation method.

Configure REA for Claude Code, install its corresponding skill and MCP integration as appropriate, and verify that it is accessible.

Because installing or changing MCP configuration may require restarting Claude Code, handle this explicitly.

Do not claim REA is connected or working without verification.

If an agent restart is required, record the exact continuation steps and resume the investigation once the integration is available.

REA is a development and research dependency only. It must not be bundled into LimeShot's runtime.

## 3.2 FL Studio investigation

Assume that FL Studio is installed on the development machine.

Determine:

- Operating system
- Installed FL Studio version
- Application location
- Executable location
- Relevant support files
- Available application resources
- Supported REA inspection capabilities

Do not assume an application location without checking.

Use REA to inspect FL Studio where legally and technically permissible.

Combine REA with direct observation, official FL Studio documentation, and legitimate application interaction.

Where supported, investigate application structure and runtime behavior.

Focus particularly on:

- Main window composition
- Panel management
- Channel Rack behavior
- Step sequencer interactions
- Piano Roll editing
- Playlist and arrangement
- Pattern management
- Mixer architecture and user interactions
- Browser navigation
- Transport controls
- Instrument loading
- Sampler behavior
- Automation editing
- Keyboard shortcuts
- Project lifecycle
- Parameter interactions
- UI scaling and resizing

Avoid unsupported claims about proprietary internal implementation.

If a behavior cannot be confirmed, mark it as unknown or inferred rather than presenting it as verified.

## 3.3 Reverse-engineering boundaries

Use clean-room engineering principles.

The objective is to understand **observable functionality and interaction models**, then create independently written Rust implementations.

Do not:

- Copy decompiled proprietary code
- Reproduce proprietary algorithms directly from decompiled implementations
- Extract or redistribute copyrighted graphics, fonts, samples, presets, or plugin binaries
- Circumvent activation, licensing, DRM, or access controls
- Modify the installed FL Studio application
- Redistribute FL Studio components
- Copy protected trade dress or branding

Prefer public specifications, official documentation, observed behavior, and independently designed implementations.

For difficult DSP features, prefer documented algorithms, mathematical derivations, and original implementations.

## 3.4 Research evidence

Create:

`docs/research/fl-studio-analysis.md`

`docs/research/rea-findings.md`

`docs/research/workflow-comparisons.md`

For each investigated feature, record:

- Feature name
- Reference version
- Expected behavior
- Observed behavior
- User interaction sequence
- Relevant interface structure
- REA evidence where applicable
- Confidence level
- Limitations or unknowns
- Proposed LimeShot equivalent

Use confidence labels:

- VERIFIED
- OBSERVED
- INFERRED
- UNKNOWN

Include source references, tool outputs, screenshots, or locally generated evidence where appropriate and legally permissible.

Research findings must directly inform implementation decisions.

---

# 4. Product Vision

LimeShot should feel familiar to someone who has spent significant time using FL Studio.

A user familiar with FL Studio should intuitively understand where to:

- Build drum patterns
- Add instruments
- Write MIDI melodies
- Arrange patterns
- Load audio samples
- Route channels to mixer tracks
- Add effects
- Adjust volume and panning
- Automate parameters
- Play, pause, loop, and navigate a composition
- Save a project
- Export a song

However, LimeShot should not look like an exact visual clone.

Think:

**FL Studio's production workflow + ArtCraft's clean, polished, approachable design philosophy.**

The interface should balance professional DAW density with visual clarity.

LimeShot must prioritize productivity, responsiveness, discoverability, and consistency over decorative complexity.

---

# 5. Visual Design System

## 5.1 Design direction

Use FL Studio as the primary reference for panel structure and production workflows.

Use ArtCraft as inspiration for:

- Clean visual hierarchy
- Consistent spacing
- Modern typography
- Thoughtful interaction states
- Custom native controls
- Cohesive visual language
- High-quality icons
- Consistent panel surfaces
- Reduced visual clutter
- Professional creative-software aesthetics

The application should feel like a modern music-production tool, not a generic dashboard.

## 5.2 Visual identity

Create an original LimeShot design identity.

Suggested characteristics:

- Dark-first interface
- Charcoal and graphite surfaces
- Lime-green accent color
- Muted supporting colors
- Subtle borders
- Distinct timeline and pattern colors
- Clear selection and hover states
- Restrained corner rounding
- Compact but comfortable controls
- Crisp icons
- Consistent spacing

The lime accent should act as a recognizable brand element without becoming visually overwhelming.

Use semantic color tokens rather than hardcoded colors scattered across individual widgets.

Provide a centralized theme system that can accommodate additional themes in the future.

## 5.3 Required layout

The application should use a familiar FL Studio-inspired primary workspace containing:

**Global toolbar**

At the top:

- Application menus
- Transport controls
- Tempo
- Time signature
- Playback mode
- Pattern selector
- Metronome
- Snap and quantization controls
- Time/position display
- Access to major windows

**Browser**

A dockable or resizable side panel used to navigate:

- Project resources
- Audio samples
- Instruments
- Presets
- Recorded files
- User directories
- Favorites

**Channel Rack**

A central pattern-based sequencer with:

- Channels
- Step buttons
- Channel names
- Channel colors
- Mute and solo
- Volume and pan
- Instrument access
- Pattern editing

**Piano Roll**

A MIDI editor featuring:

- Piano keyboard
- Time grid
- Notes
- Note lengths
- Selection
- Velocity lane
- Scrolling and zooming

**Playlist**

An arrangement editor featuring:

- Timeline
- Tracks
- Pattern clips
- Audio clips
- Playhead
- Snapping
- Moving and resizing clips
- Zoom and navigation

**Mixer**

A multichannel mixing interface featuring:

- Channel strips
- Level meters
- Volume faders
- Pan
- Mute and solo
- Track names
- Master bus
- Effect slots and routing as implemented

These windows should be accessible through familiar toolbar controls and keyboard shortcuts.

## 5.4 Interaction design

Implement consistent:

- Left-click
- Right-click
- Double-click
- Drag-and-drop
- Context menus
- Tooltips
- Keyboard modifiers
- Focus behavior
- Multi-selection
- Undo and redo
- Zoom behavior
- Mouse wheel interaction
- High-DPI scaling

Where FL Studio uses distinct interaction conventions, investigate and document them before deciding on LimeShot's equivalent.

## 5.5 Functional UI requirement

Visible controls must have real behavior.

Do not fill the interface with decorative widgets that appear functional but do nothing.

For features planned for later releases, prefer explicit disabled states, clear "not yet implemented" messaging, or omission.

A visually complete interface must never be confused with functional feature completion.

---

# 6. Technical Architecture

Design a modular Cargo workspace.

Investigate ArtCraft conventions first, but independently evaluate the final crate boundaries.

A reasonable initial direction is:

```text
limeshot/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── AGENTS.md
├── .github/
│   └── workflows/
├── crates/
│   ├── limeshot-app/
│   ├── limeshot-core/
│   ├── limeshot-ui/
│   ├── limeshot-audio/
│   ├── limeshot-dsp/
│   ├── limeshot-midi/
│   ├── limeshot-sequencer/
│   ├── limeshot-project/
│   └── limeshot-platform/
├── assets/
│   ├── icons/
│   ├── themes/
│   └── samples/
├── docs/
│   ├── architecture/
│   ├── research/
│   ├── roadmap/
│   └── development/
├── tests/
└── examples/
```

This is a proposal, not a mandatory exact structure.

Prefer a small number of well-defined crates to unnecessary fragmentation.

Every crate should have a clear purpose and reasonable dependency boundaries.

## 6.1 UI framework

Evaluate `egui`/`eframe`, particularly because ArtCraft applications use Rust-native GUI technologies.

Evaluate whether the framework can support:

- Dense DAW interfaces
- Custom Piano Roll rendering
- Timeline rendering
- Large numbers of interactive widgets
- Docking and floating panels
- High-DPI scaling
- Efficient waveform rendering
- Responsive resizing
- Continuous meter updates
- Native desktop operation

Use a pure Rust GUI framework unless a compelling technical reason dictates otherwise.

For the initial implementation, prefer a proven Rust-native framework over building a complete rendering engine from scratch.

Separate UI-specific code from music-production logic.

## 6.2 State architecture

Use a clearly defined application state model.

Keep the following concepts separate:

- Persistent project state
- Ephemeral UI state
- Audio engine state
- Playback and transport state
- Editing selection
- Application configuration
- Runtime resource management

Avoid a single enormous global mutable state object.

Implement typed commands for project mutations.

Commands should support undo and redo where appropriate.

Examples:

- AddChannel
- RemoveChannel
- RenameChannel
- ToggleStep
- AddNote
- MoveNote
- ResizeNote
- AddPattern
- AddPlaylistClip
- MovePlaylistClip
- SetTempo
- SetMixerVolume
- AssignInstrument

Consider an ArtCraft-style command architecture that allows actions to be invoked independently of UI event handlers.

This should support future headless tests, scripting, and agent control without introducing a network service.

---

# 7. Real-Time Audio Engine

The audio engine must be real.

It must produce audible output from the user's patterns, instruments, and samples.

A DAW that only animates playback is not acceptable.

## 7.1 General requirements

Implement:

- Native audio output
- Configurable output device
- Configurable sample rate
- Configurable buffer size where supported
- Stereo output
- Master gain
- Playback scheduling
- Tempo synchronization
- Pattern sequencing
- Sample playback
- Polyphonic note playback
- Basic mixing
- Offline rendering/export

Use appropriate Rust libraries and platform APIs after evaluating their maturity, licensing, and suitability.

Potential technologies include:

- CPAL for cross-platform audio I/O
- RustDSP libraries or original DSP implementations
- Rust-native audio and MIDI utilities

These are candidates, not mandatory dependencies.

## 7.2 Audio threading rules

Real-time audio performance is a central architectural requirement.

The audio callback must avoid:

- Blocking locks
- File system operations
- Network operations
- Unbounded allocation
- UI operations
- Expensive unpredictable computations
- Blocking inter-thread communication

Use suitable approaches such as:

- Preallocated buffers
- Bounded queues
- Immutable snapshots
- Lock-free or real-time-safe communication
- Dedicated worker threads for asset loading
- Sample-accurate or properly bounded scheduling

Distinguish audio-thread work from control-thread work.

Document thread ownership and synchronization.

## 7.3 Timing

Use a robust musical time representation.

Avoid accumulating floating-point timing errors for musical position.

Support:

- Bars
- Beats
- Ticks
- Tempo
- Time signatures
- Step resolution
- Quantization
- Loop regions

Account for how musical events map to audio sample positions.

Test timing across buffer boundaries and different sample rates.

Playback must remain synchronized as patterns and arrangements become more complex.

## 7.4 Basic sampler

Implement an original sample playback instrument.

For the MVP, support at least:

- Loading WAV files
- Playing assigned samples
- Triggering samples from the Channel Rack
- Triggering samples from MIDI notes where appropriate
- Gain control
- Basic pitch adjustment
- Start and end positions
- Polyphonic playback where appropriate
- Stopping voices
- Sample caching and safe asset loading

Additional audio formats should be tracked separately.

## 7.5 Basic synthesizer

Implement a lightweight built-in synthesizer.

Minimum functionality:

- Sine oscillator
- Square oscillator
- Sawtooth oscillator
- Triangle oscillator
- Polyphony
- Note-on and note-off
- ADSR amplitude envelope
- Master amplitude
- Basic parameter editing

Prefer a musically useful initial synthesizer to an elaborate nonfunctional control panel.

Track future enhancements such as filters, modulation, additional envelopes, effects, and advanced synthesis.

## 7.6 Mixing

Implement an initial mixer architecture supporting:

- Instrument/channel gain
- Panning
- Mixer channels
- Channel-to-mixer routing
- Master bus
- Basic level metering
- Mute and solo
- Clipping awareness

Design for future insert effects, sends, automation, and plugin hosting.

Do not prematurely build an unnecessarily complex mixing graph, but avoid architectural decisions that prevent later expansion.

---

# 8. MIDI and Sequencing

MIDI and pattern sequencing must be treated as foundational, not secondary.

## 8.1 Channel Rack

Implement a functional step sequencer.

MVP behavior:

1. Create a channel.
2. Assign a sampler or synthesizer.
3. Select a pattern.
4. Toggle steps.
5. Start playback.
6. Hear the correct sequence.
7. Change tempo.
8. Hear the sequence adjust correctly.
9. Stop playback.
10. Save and reopen the project.

Support configurable pattern lengths and multiple patterns, with at least a conventional 16-step default.

## 8.2 Piano Roll

Implement a genuine editable Piano Roll.

MVP requirements:

- Vertical pitch grid
- Horizontal musical timeline
- Add notes
- Delete notes
- Move notes
- Resize notes
- Select notes
- Multi-select where feasible
- Snap-to-grid
- Velocity editing
- Zoom
- Horizontal and vertical scrolling
- Piano-key auditioning
- Playback visualization

The Piano Roll must edit the same underlying musical model that the audio engine plays.

Avoid maintaining disconnected UI-only and audio-only note representations.

## 8.3 Patterns

Patterns should be reusable musical containers.

A pattern may contain multiple channel sequences.

Patterns must be placeable multiple times within the Playlist.

Changes to a shared pattern should propagate to all linked instances unless the user explicitly creates a unique copy.

Design stable IDs and references for patterns, channels, and clips.

---

# 9. Playlist and Arrangement

Implement FL Studio-inspired pattern arrangement.

## MVP requirements

- Create Playlist tracks
- Add pattern clips
- Move clips horizontally and vertically
- Resize clips where meaningful
- Delete clips
- Duplicate clips
- Select clips
- Snap clips to the grid
- Zoom timeline
- Scroll timeline
- Display musical bars and beats
- Display playhead
- Start playback from the selected position
- Play an arrangement of multiple patterns
- Edit arrangement structure

The Playlist must drive the actual audio engine during Song mode.

It must not be a separate visual representation with no playback connection.

## Additional planned functionality

Track, but do not necessarily implement in the first MVP:

- Audio clips
- Automation clips
- Clip slicing
- Slip editing
- Time stretching
- Crossfades
- Track grouping
- Loop selections
- Markers
- Arrangement versions
- Comping
- Advanced clip manipulation

---

# 10. Project System

Implement a native LimeShot project format.

Suggested extension:

`.lime`

The internal format should be designed deliberately.

It may use a versioned, structured binary or serialized format, potentially packaged with project resources.

## MVP requirements

Support:

- New Project
- Open Project
- Save Project
- Save Project As
- Modified-state detection
- Project title
- Project tempo
- Time signature
- Channel definitions
- Instrument state
- Patterns
- MIDI notes
- Playlist clips
- Mixer settings
- Relative asset references where possible

Use versioned serialization from the beginning.

Design migration capabilities for future format versions.

Use safe or atomic saving procedures to reduce the risk of project corruption.

Do not silently discard unknown or unsupported data.

Create automated save/reopen tests that compare project state before and after serialization.

---

# 11. Feature Inventory — Entire FL Studio Scope

A major requirement of this project is to **track the full functionality of FL Studio**, not merely the MVP.

Create a comprehensive feature inventory based on:

1. The installed reference application.
2. REA findings.
3. Official FL Studio documentation.
4. Public feature documentation.
5. Manual workflow observation.

Do not rely on a short memory-generated feature list.

Research the actual installed version and record version-specific differences.

## Required inventory areas

### A. Application and project management

- Project creation
- Project loading and saving
- Templates
- Autosave
- Recovery
- Project settings
- Audio settings
- MIDI settings
- Plugin management
- Preferences
- Keyboard shortcuts
- Undo/redo
- Project information
- Asset management

### B. Channel Rack

- Channels
- Step sequencer
- Pattern lengths
- Step resolution
- Channel settings
- Instrument assignments
- Sample assignments
- Channel grouping
- Mute and solo
- Swing
- Step properties
- Graph editor
- Per-channel controls

### C. Piano Roll

- MIDI editing
- Note creation
- Note movement
- Note resizing
- Velocity
- Panning
- Note properties
- Quantization
- Scale highlighting
- Ghost notes
- Chords
- Arpeggiation
- Strumming
- Humanization
- Note slicing
- MIDI recording
- MIDI import/export
- Advanced editing tools

### D. Playlist

- Pattern clips
- Audio clips
- Automation clips
- Clip operations
- Playlist tracks
- Track grouping
- Track modes
- Markers
- Loop regions
- Time stretching
- Crossfades
- Arrangement navigation
- Recording
- Playlist performance features where applicable

### E. Mixer

- Mixer inserts
- Master routing
- Sends
- Sidechain routing
- Insert effects
- EQ
- Gain
- Pan
- Stereo controls
- Phase controls
- Metering
- Bus routing
- Automation
- Plugin delay compensation
- Track grouping
- Mixer presets

### F. Instruments

- Samplers
- Synthesizers
- Drum instruments
- Multi-sample instruments
- Native generators
- Presets
- Polyphony
- Modulation
- Instrument parameter control
- Plugin instruments

### G. Audio effects

- Equalizers
- Filters
- Compressors
- Limiters
- Gates
- Reverb
- Delay
- Distortion
- Saturation
- Chorus
- Flanger
- Phaser
- Pitch shifting
- Time stretching
- Stereo processing
- Utility effects
- Effect chains
- Presets

### H. Automation

- Automation clips
- Automation events
- Parameter linking
- Envelope points
- Curves
- Interpolation
- Recording automation
- Automation editing
- Plugin parameter automation

### I. Recording

- Audio recording
- MIDI recording
- Input selection
- Input monitoring
- Punch recording
- Loop recording
- Latency compensation
- Recorded clip management

### J. Browser and asset management

- Folder navigation
- Search
- Sample preview
- Favorites
- Plugin browser
- Preset browser
- Tagging
- Drag-and-drop
- User-defined directories
- Recent resources

### K. Plugins

- VST3 hosting
- CLAP hosting
- Audio Unit hosting on macOS
- Plugin scanning
- Plugin state persistence
- Plugin UI hosting
- Plugin parameter automation
- Plugin preset management
- Plugin latency handling
- Plugin crash isolation where feasible

Plugin support is a long-term architectural requirement, not necessarily an MVP completion requirement.

### L. Export and rendering

- WAV export
- MP3 export
- FLAC export
- Sample-rate selection
- Bit-depth selection
- Dithering
- Export ranges
- Pattern rendering
- Stem rendering
- Offline bounce
- Normalization
- Metadata

### M. Additional workflows

- Time signatures
- Tempo automation
- Metronome
- Count-in
- MIDI controllers
- Controller mapping
- External MIDI
- Multichannel audio
- Performance tools
- Advanced arrangement tools
- Scripting
- Accessibility
- Advanced keyboard workflows

This list is only the initial classification.

**Expand it into a detailed hierarchical inventory using actual research.**

Do not claim the inventory is exhaustive unless it has been checked against the reference version's documented feature categories.

Distinguish core FL Studio functionality from edition-specific, optional, discontinued, or separately licensed features.

---

# 12. Feature Tracking System

Feature tracking is mandatory.

Use repository-local tracking files as the canonical source of truth.

Do not require GitHub issues or an external project management service.

Create:

```text
docs/roadmap/
├── README.md
├── features.md
├── features.json
├── milestones.md
├── mvp.md
└── known-limitations.md
```

## 12.1 Feature records

Every feature must have:

- Stable feature ID
- Category
- Feature name
- Description
- FL Studio reference behavior
- LimeShot intended behavior
- Priority
- Implementation status
- MVP inclusion
- Dependencies
- Related Rust modules
- Acceptance criteria
- Test coverage
- Relevant research evidence
- Notes

Use statuses:

- NOT_STARTED
- RESEARCHED
- DESIGNED
- IN_PROGRESS
- IMPLEMENTED
- VERIFIED
- BLOCKED
- DEFERRED

**IMPLEMENTED and VERIFIED must mean different things.**

IMPLEMENTED means code exists and meets its implementation criteria.

VERIFIED means the feature has passed appropriate tests and its intended behavior has been demonstrated.

A visual placeholder is not IMPLEMENTED.

A stub function is not IMPLEMENTED.

A passing compilation alone is not VERIFIED.

## 12.2 Machine-readable records

Maintain the complete feature inventory in `features.json`.

Example:

```json
{
  "id": "SEQ-001",
  "category": "Channel Rack",
  "name": "Step sequencing",
  "description": "Edit and play step-triggered musical patterns.",
  "priority": "P0",
  "mvp": true,
  "status": "NOT_STARTED",
  "dependencies": [],
  "modules": ["limeshot-sequencer"],
  "acceptance_criteria": [
    "Users can toggle steps.",
    "Enabled steps generate correctly timed note events.",
    "Playback follows project tempo.",
    "Step changes survive save and reload."
  ],
  "tests": [],
  "reference": {
    "application": "FL Studio",
    "version": "detected during research",
    "evidence": []
  }
}
```

The example is illustrative. Establish a consistent schema and validate it.

`features.md` should provide a human-readable overview derived from or checked against the structured inventory.

Avoid maintaining contradictory independent records.

## 12.3 Roadmap priorities

Use:

**P0 — Functional MVP**

Required for the first coherent music-production workflow.

**P1 — Core DAW functionality**

Important next-stage capabilities.

**P2 — Advanced production**

Professional workflows and extended editing.

**P3 — Long-term parity**

Specialized, complex, or less commonly used features.

Ensure priorities reflect dependencies and product usefulness, not merely implementation difficulty.

## 12.4 Roadmap discipline

Before implementing a feature:

1. Check its existing feature record.
2. Review dependencies.
3. Define acceptance criteria.
4. Implement the feature.
5. Add tests.
6. Run relevant checks.
7. Update status.
8. Record limitations.

Whenever a new FL Studio capability is discovered, add it to the inventory even if it is outside the current implementation scope.

Do not remove incomplete features simply to improve completion statistics.

---

# 13. MVP Definition

The MVP is a **functional music-production foundation**.

A user must be able to create a short composition from scratch without relying on external DAW software.

## Required MVP workflow

The following end-to-end workflow must work:

1. Launch LimeShot.
2. Create a new project.
3. Set a tempo.
4. Add a sampler channel.
5. Load a WAV drum sample.
6. Create a drum sequence using the Channel Rack.
7. Add a synthesizer channel.
8. Open the Piano Roll.
9. Write a melodic pattern.
10. Audition the pattern.
11. Create additional patterns.
12. Arrange patterns on the Playlist.
13. Switch to Song playback.
14. Play the complete arrangement.
15. Route channels to mixer tracks.
16. Adjust volume and panning.
17. Save the project.
18. Close and reopen the project.
19. Confirm that the composition is preserved.
20. Export the composition to WAV.
21. Open or otherwise verify the exported audio and confirm it contains the expected musical result.

This workflow is the MVP's central acceptance test.

## 13.1 Mandatory MVP components

- Native application window
- LimeShot visual identity
- Global toolbar
- Transport
- Tempo and time signature
- Pattern mode and Song mode
- Functional Channel Rack
- Functional step sequencer
- Functional Piano Roll
- Functional Playlist
- Basic Browser
- Functional Mixer
- Working sampler
- Working basic synthesizer
- Real audio playback
- Basic MIDI/note model
- Project saving/loading
- WAV export
- Keyboard shortcuts for core actions
- Undo/redo for core editing operations
- Feature tracking system
- Basic automated test suite

## 13.2 Scope management

Prioritize completing the end-to-end workflow over implementing numerous disconnected features.

Do not spend the entire initial development effort implementing advanced effects, plugin hosting, or obscure editing commands while basic sequencing remains incomplete.

However, the architecture must account for future additions.

Avoid short-term hacks that make essential future capabilities structurally impossible.

---

# 14. Cross-Platform Requirements

All three primary desktop platforms must be considered from the start.

Do not implement a macOS-only architecture with the assumption that Windows and Linux support can be bolted on later.

Use cross-platform abstractions for:

- Audio devices
- File operations
- Window creation
- Input
- Keyboard shortcuts
- Filesystem paths
- Display scaling
- MIDI devices
- Native dialogs
- Configuration directories

Use platform-specific modules only where needed.

Document platform requirements.

Create CI workflows covering:

- macOS
- Windows
- Linux

CI should perform the available relevant checks:

- Formatting
- Compilation
- Linting
- Unit tests
- Integration tests

Where runners or system dependencies prevent complete testing, explain the limitation.

**Do not claim that a platform has been tested merely because its code is intended to be portable.**

If only one operating system is locally accessible, test it directly and use CI or cross-compilation checks for the others where feasible.

Record verification separately for every platform.

---

# 15. Testing and Quality

Treat audio and musical correctness as seriously as visual correctness.

## 15.1 Unit tests

Test:

- Musical time calculations
- Tempo conversions
- Step scheduling
- Pattern indexing
- MIDI note transformations
- Note velocity
- Synth voice allocation
- ADSR envelope behavior
- Audio mixing
- Pan calculations
- Project serialization
- Command undo/redo
- Clip positioning
- Clip duplication

## 15.2 Integration tests

Test:

- Channel Rack → Sequencer → Audio Engine
- Piano Roll → Note Events → Synthesizer
- Playlist → Pattern Scheduling → Playback
- Mixer Routing → Master Output
- Project Save → Project Load
- Project → Offline WAV Render

## 15.3 Audio tests

Use deterministic offline rendering when possible.

Verify:

- Correct output length
- Expected sample rate
- Silence where appropriate
- Non-silent signal where expected
- Correct note onset positions
- Stable timing
- No unintended clipping
- Correct note release behavior
- Consistent results across audio buffer sizes

Do not rely exclusively on listening tests.

## 15.4 UI tests

Where possible, test UI commands independently from drawing code.

Use reproducible application states.

If the chosen framework supports screenshot or visual regression tests, evaluate incorporating them.

If tests cannot cover certain manual interactions, create documented manual test procedures.

## 15.5 Performance

Track basic performance metrics:

- Audio callback execution time
- Buffer underruns where observable
- UI frame timing
- Project load time
- Memory consumption
- Voice count
- Offline render throughput

Avoid introducing performance optimizations without profiling evidence, except for standard real-time audio safety practices.

---

# 16. Development Workflow and Milestones

Use incremental development.

Every milestone should produce a usable, testable improvement.

## Milestone 0 — Research and Architecture

- Inspect ArtCraft
- Configure REA
- Inspect installed FL Studio
- Document findings
- Create feature inventory
- Decide on Rust libraries
- Design core application structure
- Define MVP acceptance criteria

Deliverable: Documented architecture and initialized development workspace.

## Milestone 1 — Native App Foundation

- Cargo workspace
- Native window
- Main UI layout
- Design tokens
- Panels
- Menus
- Keyboard command infrastructure
- Application state
- Basic project model
- CI workflows

Deliverable: A native LimeShot application that opens on supported development environments.

## Milestone 2 — Sequencer and Audio

- Audio device initialization
- Playback clock
- Pattern model
- Channel model
- Step sequencer
- WAV sample playback
- Transport controls
- Tempo handling

Deliverable: A user can create and hear a drum pattern.

## Milestone 3 — Piano Roll and Synthesizer

- MIDI note model
- Piano Roll interactions
- Velocity control
- Polyphonic synthesizer
- Envelope
- Note scheduling

Deliverable: A user can compose and hear a melody.

## Milestone 4 — Arrangement

- Multiple patterns
- Playlist
- Pattern clips
- Song mode
- Pattern scheduling across an arrangement
- Timeline navigation

Deliverable: A user can arrange and play a short song.

## Milestone 5 — Mixing and Project Lifecycle

- Mixer channels
- Gain and pan
- Master output
- Level meters
- Project save/load
- Undo/redo coverage

Deliverable: A user can mix and persist a project.

## Milestone 6 — Export and MVP Verification

- Offline rendering
- WAV export
- End-to-end workflow tests
- UX improvements
- Stability improvements
- Cross-platform build validation
- Documentation

Deliverable: A demonstrably functional LimeShot MVP.

Milestones are guidelines.

Adjust sequence where justified by actual dependencies, but preserve the end-to-end MVP goal.

---

# 17. Development Documentation

Create:

`AGENTS.md`

This file must explain how a future Claude Code session should understand and continue the project.

Include:

- Project mission
- Architecture summary
- Workspace organization
- Build instructions
- Test commands
- Coding conventions
- Real-time audio safety rules
- Feature tracking conventions
- Current milestone
- Current implementation status
- Known problems
- Immediate next tasks
- REA research references

Also create:

`docs/development/decisions.md`

Record architectural decisions and their reasoning.

`docs/development/progress.md`

Maintain a concise chronological development log.

`docs/development/next-steps.md`

Keep a prioritized, actionable continuation list.

`docs/architecture/audio-engine.md`

Explain audio threading, scheduling, routing, and timing.

`docs/architecture/project-model.md`

Explain the document and sequencing model.

`docs/architecture/ui.md`

Explain UI organization and interaction architecture.

Documentation should reflect actual implementation, not aspirational code.

Keep these documents up to date throughout development.

---

# 18. Engineering Standards

Follow idiomatic Rust engineering practices.

Requirements:

- Use `cargo fmt`.
- Use Clippy.
- Write meaningful unit tests.
- Avoid unnecessary `unsafe`.
- Document necessary unsafe code and its invariants.
- Use typed errors and meaningful error messages.
- Avoid unchecked panics in normal application flows.
- Separate UI code from core logic.
- Keep modules focused.
- Prefer explicit interfaces.
- Avoid circular dependencies.
- Prefer maintainable code over clever abstractions.
- Use appropriate profiling and debugging techniques.
- Pin dependencies appropriately through Cargo.lock.
- Check dependency licenses.
- Do not embed proprietary reference application assets.

Use actual production code rather than pseudocode.

Never create a function with an empty body merely to satisfy a compiler unless it is explicitly marked as an intentional unimplemented interface and excluded from completed feature claims.

Avoid large speculative frameworks before they are necessary.

Do not overengineer the MVP, but establish sound boundaries.

---

# 19. User Experience Expectations

The UI should look like a legitimate desktop DAW from the first functional milestones.

Avoid the appearance of:

- Generic administrative dashboards
- Web-app layouts
- Unrelated ArtCraft branding
- Oversized controls
- Excessive whitespace
- Inconsistent iconography
- Unstructured floating widgets
- Unnecessary dialogs
- Placeholder-heavy interfaces

The application should have a coherent and recognizable identity.

Create an original LimeShot logo or icon using appropriate project-owned vector assets or drawing code.

Do not copy FL Studio's fruit logo or ArtCraft's existing application mascots.

Favor ergonomic, discoverable controls and consistent visual behavior.

A musician should spend their time creating music rather than fighting the interface.

---

# 20. Research vs. Implementation

You must maintain the distinction between:

**Reference investigation:** Understanding how FL Studio behaves.

**Architecture research:** Understanding how ArtCraft projects are structured.

**Design:** Choosing how LimeShot should behave and look.

**Implementation:** Writing the actual Rust code.

**Verification:** Proving that the code works.

Do not let extensive documentation replace implementation.

Once the necessary initial research and architecture are completed, begin writing and running the application.

Likewise, do not skip necessary research merely to generate code faster.

Use evidence-based decisions where possible.

If REA cannot inspect a particular FL Studio feature, use documented observations and independent engineering judgment.

Do not block the entire project on obtaining impossible reverse-engineering evidence.

---

# 21. Autonomy and Execution Rules

You are expected to act as an implementation agent, not only as an advisor.

Do not respond with a plan and stop.

Actually create the project.

Actually write the files.

Actually implement features.

Actually execute available tests.

Actually fix discovered errors.

Actually update feature statuses.

## Working rules

1. Inspect the environment before modifying it.
2. Confirm reference repository contents from source.
3. Check existing files before creating replacements.
4. Implement incrementally.
5. Compile frequently.
6. Run relevant tests after implementation changes.
7. Address warnings and errors rather than hiding them.
8. Keep changes coherent.
9. Update documentation and tracking.
10. Report genuine blockers.

For long-running assignments, maintain recoverable checkpoints.

If the current session must end before all milestones are finished, ensure the repository is left in a clear, buildable state where possible.

Record the next exact steps for another agent session.

Do not claim that work will continue in the background after the session ends.

## Critical anti-patterns

Never:

- Produce only a beautiful UI mockup.
- Implement fake audio playback.
- Mark buttons as working when they are not.
- Claim that a feature exists because a type or enum exists.
- Build the entire interface before connecting any functionality.
- Treat compilation as proof of behavioral correctness.
- Claim cross-platform support without verification.
- Invent reverse-engineering findings.
- Copy proprietary FL Studio implementation code.
- Reuse SoundCraft source code.
- Ignore feature tracking updates.
- Report the MVP as finished before the end-to-end workflow passes.

---

# 22. Definition of Done

A feature is complete only if:

1. Its implementation exists.
2. It fulfills documented acceptance criteria.
3. It is integrated with the appropriate application components.
4. Its visible controls behave correctly.
5. It has relevant tests or documented manual verification.
6. Its limitations are recorded.
7. Its feature tracking status is accurate.

The MVP is complete only when a user can:

- Create a project
- Load a sample
- Program drums
- Write a melody
- Arrange multiple patterns
- Play the arrangement
- Mix the result
- Save and reopen
- Export usable WAV audio

And these operations are confirmed through meaningful verification.

A partial implementation must be described as partial.

---

# 23. Required Initial Execution Sequence

Begin immediately with the following steps.

**Step 1 — Inspect the environment**

Determine:

- Operating system
- Rust toolchain
- Installed build tools
- Available audio devices
- Installed FL Studio version and location
- Available REA tooling
- Repository directory state

**Step 2 — Research the reference repositories**

Inspect ArtCraft and especially SoundCraft.

Review source structure and identify relevant patterns.

Write the architectural research notes.

**Step 3 — Activate REA**

Read the official REA setup guidance.

Install or configure it as necessary.

Verify that REA can be invoked.

Investigate the installed FL Studio application through supported analysis and observation.

Record findings and limitations.

**Step 4 — Establish LimeShot's design and architecture**

Choose:

- Rust workspace layout
- UI framework
- Audio backend
- Sequencer time representation
- State management
- Command architecture
- Project format
- Theme architecture
- Platform abstractions

Document the decisions.

**Step 5 — Generate the full feature inventory**

Create the structured feature matrix.

Cover the broader FL Studio feature set, not merely the initial development tasks.

Mark MVP features clearly.

**Step 6 — Initialize the project**

Create:

- Cargo workspace
- Rust crates
- Basic application
- Main window
- Project model
- Development documentation
- CI configuration
- README
- Licensing files

**Step 7 — Build the first working vertical slice**

Implement the smallest useful end-to-end sequence:

Channel Rack step → Sequencer event → Sampler voice → Audio mixer → Sound output.

Do not substitute a visual animation for audio.

**Step 8 — Expand toward the MVP**

Proceed through Piano Roll, synth, Playlist, mixing, project persistence, and export.

**Step 9 — Verify and improve**

Compile, test, run, inspect, and fix actual problems.

Use the reference findings to refine interaction behavior.

**Step 10 — Report progress**

At meaningful milestones and at the end of the session, summarize:

- Completed work
- Implemented and verified features
- Working build targets
- Tests performed and their results
- Remaining MVP requirements
- Known bugs
- Next actionable development tasks

---

# 24. Final Project Principles

Keep the following principles at the center of every engineering decision.

**Familiarity without imitation**

LimeShot should feel intuitive to FL Studio users while being an original application.

**Rust from the ground up**

The application should have the performance, portability, and maintainability advantages of a thoughtfully designed Rust codebase.

**Audio correctness first**

A DAW's essential purpose is producing and manipulating music. The real-time engine must be reliable.

**Beautiful but functional**

Visual polish must never replace working behavior.

**Independent engineering**

Use ArtCraft as a model for quality and architecture, not as a code source.

**Track the entire destination**

Maintain a detailed map of FL Studio's features even while implementing a narrower MVP.

**Build a usable application incrementally**

Every development milestone should move LimeShot closer to a real musician's workflow.

**Verify reality**

Do not claim functionality, compatibility, or completeness that has not been demonstrated.

---

# 25. Begin Now

You have enough information to begin.

Do not merely paraphrase these instructions or return a speculative implementation outline.

Start by inspecting the working environment, researching ArtCraft, and preparing the REA integration.

Then create the LimeShot repository and begin implementing the native Rust application.

Make reasonable technical decisions autonomously, preserve research evidence, and keep the roadmap current.

Your immediate engineering priority is a working, audible Channel Rack-to-sampler playback pipeline, followed by the complete functional MVP.

**Build LimeShot.**
