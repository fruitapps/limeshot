# Architecture decisions

Each entry: decision, alternatives, reasoning. Newest last.

### D1. Rust workspace of eight crates with a strict layering
`core` → `project`, `dsp` → `sequencer` → `engine` → `audio` → `ui` → `app`. Nothing below `ui` depends on egui, and
nothing below `audio` touches a device. **Why:** headless tests and the CLI drive the whole product. The layers follow
SoundCraft's (see `docs/research/artcraft-architecture.md`), but there are fewer crates: there is no plugin hosting or
video yet, and START.md asks for "a small number of well-defined crates". MIDI devices and platform helpers did not
justify their own crates yet: the MIDI note model lives in `core`, and config directories in `ui::settings`.

### D2. egui/eframe 0.36 for the UI, glow renderer
**Alternatives:** iced, Slint, a custom wgpu renderer. **Why:** egui is the Rust-native toolkit every ArtCraft desktop
app uses for dense, custom-painted editors. It handles HiDPI, has AccessKit and runs on all three OSes. Version 0.36
matches ArtCraft's working code, which gave a verified API reference. glow builds faster than wgpu and is enough for 2D;
wgpu is used only for offscreen test renders.

### D3. cpal 0.16 for audio output
**Why:** cross-platform (CoreAudio, WASAPI, ALSA/JACK), permissive licence, and proven in SoundCraft and EffectCraft.
ASIO and JACK are optional cpal features and are not enabled.

### D4. Musical time = 960 PPQ integer ticks; exact u128 tick↔sample mapping
**Alternatives:** f64 seconds or beats, sample positions (as in SoundCraft). **Why:** a pattern DAW edits in musical
units, and integer ticks never drift. Sample positions are derived per tempo segment with exact rational arithmetic, so
results do not depend on buffer size (tested). 960 PPQ divides evenly into 16ths, triplets and 32nds.

### D5. Immutable compiled snapshot sent to the audio thread
**Alternatives:** shared project behind a mutex; fine-grained parameter messages. **Why:** the audio thread never
blocks or sees half-applied edits. One translation step (`sequencer::compile`) is the only place where the model
becomes sound, so the UI and the engine cannot drift apart. Full recompiles are cheap at MVP sizes. Incremental updates
are a later optimisation if profiling asks for them.

### D6. Lock-free SPSC rings (rtrb) for control and garbage return
**Why:** wait-free push/pop with no allocation. Old snapshots go back to the UI thread, so the callback never frees
them. The `SampleCache` keeps decoded samples alive so voices never drop the last reference.

### D7. One renderer for realtime and offline
**Why:** the export tests check playback scheduling too. Learned from SoundCraft's shared mix engine.

### D8. Typed `Command` enum with snapshot undo and gesture merging
**Alternatives:** string-id commands with JSON params (SoundCraft), inverse-command undo. **Why:** type safety in a
smaller codebase. The enum is still serde-serialisable, which gives the CLI scripting. Snapshot undo is simple and exact
for note-sized projects. Merge keys plus explicit gestures make drags one undo step.

### D9. Steps are notes
**Alternative:** a separate step grid per channel. **Why:** FL Studio's manual states that on-beat notes appear as
steps. One note list means the Channel Rack and Piano Roll edit the same data, as START.md §8.2 asks.

### D10. `.lime` = versioned JSON envelope with unknown-field preservation
**Alternatives:** a binary format, a zip bundle. **Why:** easy to diff and debug, and serde makes forward compatibility
cheap (`flatten` extra maps, `Unsupported` instruments). Newer versions are refused, not misread. Saves are atomic. A
zipped bundle with samples is tracked (APP-019).

### D11. Sample paths absolute in memory, relative on disk
**Why:** "Save As" to another folder keeps working without rewriting the document or losing undo history. The file
stays portable when samples live inside the project folder.

### D12. Balance pan law
**Alternatives:** constant power (−3 dB centre), FL's "circular" law. **Why:** centre stays unity and panning never
boosts, so a mix cannot clip from panning alone. A selectable law can come later.

### D13. New channels take the next free mixer insert
**Deviation from FL Studio** (FL routes new channels to Master). **Why:** the Mixer is useful immediately and each
channel gets its own fader and meter. Recorded in `workflow-comparisons.md` W5.

### D14. Pattern clips play their pattern once (no looping inside a longer clip)
**Why:** predictable, and it matches FL Studio: observed live on 2026-10-20 (session 4) that a lengthened pattern clip
is silent after its pattern ends (`fl-studio-analysis.md` §4).

### D15. Sampler declicks on note-off only
**Why:** a 1 ms attack shifted the onset by one sample and hid sample-accurate timing. FL documents "Out only" as a
declick mode. A fade-in option can be added per channel.

### D16. Clean-room research; no decompilation provider installed
REA was installed and used for static metadata (plist, Mach-O). Ghidra and Hopper were not installed: START.md §3.3
forbids reusing decompiled code, and the behaviour we need is documented. Live UI observation needs Screen Recording
permission, which only the user can grant (see `docs/research/rea-findings.md`).

### D17. No network service for agent control
SoundCraft's TCP control channel was not copied (START.md forbids network services). `limeshot-cli run` (typed command
scripts), `render` and `play`, plus the offscreen `ui_shot` renderer, cover headless and agent use.

### D18. CI on every push for macOS (arm64 + Intel), Windows, Linux (x86-64 + ARM64), plus a Windows ARM64 cross build
**Why:** START.md treats all three platforms as first class. Platform status is recorded only after a job has actually
passed.

### D19. Docked layout instead of floating editor windows (session 2) — superseded by D24
The editors moved from floating egui windows into an ArtCraft-style docked layout: the Playlist as the central
workspace (FL Studio opens it maximised, observed), a bottom dock with Channel Rack / Piano Roll / Mixer tabs, and an
Inspector on the right replacing FL's floating Channel Settings. **Why:** the user asked for a sleeker, more modern UI
in the ArtCraft style, and ArtCraft apps dock their panels. Floating windows overlapped and needed manual arranging.
**Trade-off:** the Channel Rack and Piano Roll are not visible at the same time. Tear-off panels are tracked (UI-007).

### D20. Effect processors are built on the control thread and handed over by id
**Alternatives:** allocate inside the audio callback (violates real-time rules); preallocate a fixed processor per
slot (wastes memory, since reverbs and delays hold seconds of audio). **Why:** the snapshot stays immutable and
shareable, the audio thread only moves boxes between preallocated vectors, and freeing happens on the control thread.
See `audio-engine.md` › Insert effects.

### D21. Inter and JetBrains Mono fonts
Both are SIL OFL, fetched from their upstream releases (rsms/inter v4.1, JetBrains/JetBrainsMono v2.304) and embedded.
They match the ArtCraft apps' typography and read better at small sizes than egui's defaults.

### D22. New projects start at 140 BPM
Observed in FL Studio 2026's toolbar on a new project. `Tempo::default()` stays 120 for engine-level defaults.

### D23. UI tests locate controls through recorded geometry
Editors record their layout (step strip rectangles, Piano Roll and Playlist grid mappings) each frame. Tests compute
pointer positions from it, so restyling does not break the interaction tests.

### D24. FL Studio's window model with ArtCraft-like styling (supersedes D19)
The user clarified that FL Studio is the source for UI/UX and ArtCraft only for the broad style. Editors are windows in a
workspace again: movable, resizable, maximisable, detachable into OS windows (egui viewports), toggled by FL's F-keys;
the Playlist starts maximised with the Channel Rack on top; Channel Settings is a window opened from the channel
button; the Mixer carries the selected track's FX slots on its right. Styling (tokens, fonts, flat title bars and
controls) stays ArtCraft-like. Implementation: `crates/limeshot-ui/src/workspace.rs`.

### D25. Only user interaction may turn widget changes into commands
egui sliders can report `changed()` on their own when they round a value to the displayed decimals; viewing the Mixer
once silently edited the project. Standard egui widgets that issue commands go through `widgets::user_changed`
(changed and hovered/focused/dragged). A UI test asserts that opening every window leaves the project unmodified.

### D26. Step buttons toggle with either mouse button
**Why:** observed in FL Studio 2026.1.1 (session 4): right-click on an unlit step lights it, and a drag paints the state
the first step was switched to. The manual's "right-click turns off" is outdated. LimeShot follows the observed
behaviour; deleting in the Piano Roll and Playlist stays on right-click, as in FL.

### D27. Audio clips play a sampler channel
**Why:** FL Studio turns a sample dropped on the Playlist into an Audio Clip channel; reusing LimeShot's sampler as that
channel gives audio clips volume, pan, pitch, trim and mixer routing without a second playback path. Clip offsets are
ticks (no time-stretching): a tempo change moves the audio against the grid, as non-stretched clips do in FL.

### D28. Automation runs in Song mode, on the control's own scale
**Why:** automation clips live on the Playlist, so Pattern mode plays controls at their set values (FL Studio does
the same for clip automation). Normalising on the control's travel (cube root for faders, log for frequencies) makes a
straight line between points sound like moving the control. Values are applied every 64 frames from preallocated
arrays, keeping the audio thread allocation-free; tempo automation needs a tempo map and is deferred (AUT-008).

### D29. Own Standard MIDI File reader/writer
**Why:** the format is small and public, input must be treated as hostile, and export should match playback exactly
(it is built from the compiled program, swing included). A dependency would add little.

### D30. An autosave belongs to the session that wrote or recovered it
**Why:** removing the autosave whenever the project is clean would delete a crashed session's copy at the next start,
before the user can recover it. A session only removes copies it owns.

### D31. Keybindings and settings are tracked like features
**Why:** FL Studio's shortcuts and settings are a large, user-facing surface. `keybindings.json` and `settings.json`
record each one against LimeShot, and `scripts/inventories.py` (run by `features.py check` in CI) fails when a binding
LimeShot claims is missing from the shortcuts help or a claimed setting is not a real field.

### D32. VST3 hosting through the `vst3` bindings, in the one crate allowed `unsafe`
**Why:** the owner made VST3 hosting mandatory (2026-10-21). Steinberg moved the VST 3 SDK to the MIT licence with
version 3.8 (2025), and the `vst3` crate (coupler-rs, MIT OR Apache-2.0) ships bindings generated from the 3.8 headers,
so LimeShot links no SDK code and needs no SDK download to build. The bindings are raw COM, so `limeshot-plugins` is
the only crate exempt from the workspace's `unsafe_code = "forbid"`; it uses `unsafe_op_in_unsafe_fn = "deny"` and a
`SAFETY:` comment on every block, and exports only safe types. A plugin is split into a control-thread half
(`Vst3Instance`: parameters, state, latency) and an audio-thread half (`Vst3Processor`: preallocated buses, event
and parameter lists, so `process` neither allocates nor locks); both share one teardown core. Tests host small
plugins compiled into the crate (`testing`), so CI needs no installed plugins. Plugins are created on the main thread:
several frameworks (SINE Player observed) trap when created elsewhere. Loading a module runs third-party code that can
crash LimeShot, so scanning runs each bundle in a child process (PLG-010 covers isolating processing as well).

### D33. Plugin editors in egui viewports, with the plugin's view as a child view
**Why:** FL Studio's plugin windows (observed live, `fl-studio-analysis.md` §17) are one OS window per plugin with
the host's tabs above the plugin's own view. LimeShot makes each window an egui immediate viewport (so the tabs and
the Settings page are ordinary egui, sharing the app's state) and puts the plugin's `IPlugView` into a plain
`NSView` added to that window's content view, below the tab row. eframe does not hand out the native handles of
secondary viewports, so `limeshot-plugins` finds the `NSWindow` by its title (LimeShot makes plugin window titles
unique); the FFI stays in that crate (`editor.rs`, objc2). `IPlugFrame::resizeView` resizes the container and the
window at once, as plugins expect; LimeShot then sends the matching size as a viewport command, because eframe applies
builder changes of immediate viewports only together with a command. The alternative, a separate borderless child
window like FL's, needs its own key-window handling and was not needed: AppKit composites the plugin's layer-backed
view above egui's surface. Windows (`HWND`) and Linux (`X11EmbedWindowID`, `IRunLoop`) need only the platform
module; until then the Plugin tab lists the parameters.

### D34. Crash isolation: an opt-in plugin bridge process per plugin
**Why:** a plugin runs third-party native code; a crash in it ends the process, and catching a fault in-process is
not sound in Rust. FL Studio's answer is the bridged plugin (Plugin options › Bit size), a per-plugin choice, so
LimeShot does the same: "Run in a separate process" in a plugin's settings (`PluginRef::separate_process`, an undoable
command). The plugin then runs in a copy of LimeShot started with `--plugin-bridge <socket>`; the child connects twice
to a Unix socket: control calls (parameters, state, presets) on one connection, served by the child's main thread,
and one request/reply per audio block on the other, served by a second thread, mirroring the in-process split
(D32). Messages use a small fixed binary encoding over preallocated buffers, so the audio thread does not allocate;
it does block on the socket, with a 500 ms limit, as plugin bridges must. A failed or late reply marks the plugin
crashed: its slot is silent, the child is stopped, LimeShot reports it once and offers Reload (the state last saved
in the project). Shared memory would save two copies per block but needs `mmap` and cross-process wake-ups that
macOS does not offer with timeouts (no `sem_timedwait`); a 512-frame block round trip measured about 0.1 ms. The
bridge is Unix only; on Windows the option is hidden until a named-pipe transport exists. A bridged plugin's own
editor would live in the child process and is not shown yet (the Plugin tab lists parameters). Plugins that are not
bridged still run in LimeShot's process.



### D35. Plugin wrappers inside the workspace (supersedes D33's separate viewports)
**Why:** the owner requested plugin editors in LimeShot windows, resizable on both tabs. Wrappers now use ordinary
egui windows and attach their NSView container to the main NSWindow. Embedded containers never resize the OS window;
IPlugView size constraints and onSize handle resizable plugins, while fixed-size views retain a minimum window size and align at the top.
The container keeps the editor's natural dimensions so its subviews cannot extend upward across the tabs. Native views hide while covered by another
workspace window, a popup or a dialog. Other platforms retain the parameter-list fallback. The old attach API remains
available for detached host-window clients and its tests.

### D36. MIDI input with midir, through the UI thread
**Why:** MIDI input uses `midir` (MIT; CoreMIDI, WinMM, ALSA), in `limeshot-audio`. Messages are parsed on midir's thread and
passed over a channel to the UI thread, which plays them like other auditions; the callback only wakes the UI.
Sending them straight to the audio thread would save one frame of latency at most, but the engine's message ring
has a single producer (the UI), and recording needs the notes on the UI side anyway. Revisit if live-play latency
proves noticeable.

### D37. Compressed samples with symphonia
**Why:** MP3, FLAC, Ogg Vorbis and AIFF samples (INS-008) are decoded with `symphonia` 0.5, a pure-Rust decoder (no
system codecs, so every platform reads the same files). It is MPL-2.0: file-level copyleft that covers only
symphonia's own files, used unmodified from crates.io, so LimeShot stays MIT OR Apache-2.0 (as with `option-ext`).
WAV keeps `hound`. Encoding (MP3/FLAC export) is not part of this decision.



### D38. Standalone Lime Native plugins in a separate workspace
The owner requested independent VST plugins usable in other DAWs, with dedicated Lime-themed native editors.
`native-plugins/` is an isolated Cargo workspace to keep the DAW's dependency graph and egui version stable.
Nine cdylib entry points use nice-plug 0.4.2 (ISC), its modern egui adapter, and the same permissively licensed
`vst3` bindings already used by the host. No handwritten unsafe code is added: plugin crates forbid unsafe;
FFI implementations are supplied by the framework's export/parameter macros. The existing D32 boundary in the
DAW remains intact. `limeshot-engine::voice::Voice` and `fx::build` supply the actual instruments/effects.
`Envelope::set_adsr` and `Voice::update_instrument` preserve running voices while parameters change.
Host state embeds bounded PCM sample data so Sampler sessions do not depend on the source file's path.
Sample decoding, allocation and locking happen during state restoration/activation, never processing; active
voices hold the same sample Arc that the plugin owns, so finishing/stealing a voice cannot free sample data.
Factory and file presets share LimeShot's schema and factory definitions. The bundler targets this nested
workspace explicitly, avoiding its upstream helper's outermost-manifest heuristic. `build.py` collects separate
VST3 bundles in `dist/`, with no automatic installation or changes to the user's DAW.

### D39. The Lime VST3s are LimeShot's instruments and effects

Owner decision (2026-10-22): LimeShot hosts the Lime Native VST3 plugins (D38) as its instruments and effects instead
of in-process versions. Projects refer to them as `lime:<bundle>` so they stay portable; `limeshot-plugins::lime`
resolves the bundle (override folder, beside the executable, `native-plugins/dist`, standard VST3 folders). Their
state is the plugins' own nice-plug JSON inside the host's `LSV3` container, so the DAW configures them without
instantiating them; Lime Sampler embeds sample audio, as it does in other DAWs. LimeSynth/Sampler parameter
definitions live in `limeshot_core::lime` and are shared with the plugins; `SetPluginState` is a mergeable command, so
hosted edits are undoable. The shared DSP crates stay: the plugins link them, and a test-only adapter feeds the same
DSP through the real plugin rack for exact sample-bit comparisons.

Projects saved before the switch (format 1) are converted by a document normalizer in the GUI, CLI and headless
constructors and saved as format 2; effect automation is re-expressed in plugin parameter ranges.

Features the in-process engine had are carried by the plugins through a small host protocol: Playlist clips stay
inside Lime Sampler with fixed-size, 7-bit-clean, versioned SysEx (source window, chase, fades) that never allocates
in processing; per-note pan uses standard VST3 expressions, and a channel-pan message keeps the additive balance law;
output SysEx reports native voice counts and dynamics reduction per block, copied during the event callback. Routed
tracks feed Lime Comp's stereo key input. The Mixer track EQ is a marked hosted Lime EQ after the ten inserts, before
the fader, driven by appended hidden controls; its saved bands remain canonical. Gate/Drive/Chorus use bundle IDs
9–11; earlier IDs and parameter order are unchanged. The process bridge serializes the same data. On macOS the main
viewport saves and restores its OpenGL context around plugin callbacks, because native editors clear it.

### D40. Installers place the Lime VST3 suite in the shared VST3 folders (2026-10-25)

Owner request: an alpha release whose installers install both LimeShot and the Lime VST3 suite. D38 kept
`build.py` free of installation; the installers now do it, and only on the user's explicit install. The suite is
installed once, in the platform's standard VST3 folder, which LimeShot already searches (D39), so other hosts can
load the same bundles and no 140 MB second copy is shipped inside the app. Self-contained layouts remain for the
cases without a system install: the portable Windows ZIP and the Linux tarball keep `plugins/` beside the binary,
and the macOS DMG app keeps `Contents/PlugIns`. The Windows installer's VST3 component can be unticked, which puts
the bundles in the app's private `plugins` folder instead (LimeShot needs them either way); the Linux `install.sh`
offers the same through `--private-plugins`. The macOS PKG installs both parts with no choice, because a pkg
choice cannot redirect the bundles into the app. Inno Setup builds the Windows installer (one script, components,
per-user or per-machine install, no WiX/.NET toolchain); Linux ships a tarball with an install script and a `.deb`.
AppImage, MSI and Flatpak are not provided: they add tooling without changing what is installed. Packages are
unsigned or signed ad hoc until signing identities exist.


### D41 — retain loaded VST3 modules for the process lifetime (2026-10-25)

Both Linux architectures crash in concurrent hosted MVP tests on main `5ab09c9`. The old weak module cache permits
last-client teardown to overlap a new load, and unloads libraries whose frameworks may leave process-global
callbacks or thread-local destructors registered. Keep successful module/factory loads strongly cached on the
control side, while still releasing instance DSP/state normally. This extends macOS's existing executable-retention
policy consistently across platforms and prevents entry/exit races. The module-reload regression checks residency
without clients and shared identity across concurrent reloads. Code/factory memory remains until process exit;
plugin binary updates require restarting LimeShot. CI at `28d3a41` passes on both Linux architectures.
