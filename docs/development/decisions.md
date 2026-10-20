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
**Why:** predictable. FL's exact behaviour for piano-roll data in extended clips is not documented on the manual pages
we read (marked INFERRED/UNKNOWN). Revisit after observing FL directly.

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

### D19. Docked layout instead of floating editor windows (session 2)
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
