# Project principles

The binding rules from the original project brief (`START.md`, since retired), condensed. For current status see
[next-steps](next-steps.md) and [progress](progress.md); for the reasons behind design choices see
[decisions](decisions.md).

## Mission

LimeShot is an open-source, offline-first, cross-platform DAW in Rust, built around FL Studio's pattern workflow
(Channel Rack, Piano Roll, Playlist, Mixer). FL Studio is the behaviour reference; ArtCraft/SoundCraft inform
architecture and styling only. LimeShot keeps its own identity, implementation and branding.

## Non-negotiable constraints

1. First-party code is Rust: audio engine, project model, application behaviour and UI. Platform APIs and FFI bindings
   are fine where native audio, MIDI or plugin interop needs them. Build tooling may be non-Rust.
2. No Electron, Tauri, JavaScript frontend, webview, browser/WASM target, hosted backend, account system or mandatory
   network service. The application works offline.
3. No FL Studio source code, binary resources, branding or copyrighted assets, and no code copied from ArtCraft or
   SoundCraft.
4. The architecture supports real-time audio (see [audio-engine](../architecture/audio-engine.md)), not simulated playback.
5. Document limitations honestly; never present an unimplemented feature as complete.

## Clean-room research

Observe behaviour, metadata and public documentation, then write independent implementations. Do not copy decompiled
code or reproduce algorithms from it; do not extract or redistribute fonts, graphics, samples, presets or plugin
binaries; do not circumvent activation or licensing; never modify or redistribute an installed FL Studio. Prefer
documented algorithms for difficult DSP. Research notes live in `docs/research/` and label confidence as `VERIFIED`,
`OBSERVED`, `INFERRED` or `UNKNOWN`. Findings must inform implementation decisions.

## Definition of done

A feature is complete only when its implementation exists, meets documented acceptance criteria, is integrated, has
visible controls that behave correctly, has tests or a recorded manual verification, has its limitations recorded, and
has an accurate status in [`features.json`](../roadmap/features.json) (see the [roadmap conventions](../roadmap/README.md);
IMPLEMENTED is not VERIFIED). A partial implementation is described as partial. Do not remove incomplete features to
improve completion numbers; add newly discovered FL Studio capabilities as records.

The MVP workflow (create, sample, drums, melody, arrange, mix, save/reopen, export WAV) is the central acceptance test:
see [mvp.md](../roadmap/mvp.md).

## Scope

Finish end-to-end workflows before adding disconnected features, but avoid hacks that make future capabilities
structurally impossible.

## Cross-platform honesty

macOS, Windows and Linux are all first-class: use cross-platform abstractions for audio, files, windows, input, paths,
scaling, MIDI, dialogs and config directories, with platform modules only where needed. CI covers all three. Never claim
a platform is tested because the code is portable; record verification per platform in
[platform-validation](platform-validation.md).
