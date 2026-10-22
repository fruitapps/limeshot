# Next steps (prioritised)

Current as of 2026-10-22, session 17. Read the feature record and its dependencies before implementation.
Completed work and verification evidence are retained in [progress.md](progress.md); statuses are canonical in
[features.json](../roadmap/features.json).

## Immediate verification

1. **Native plugin embedding (PLG-006).** Internal Plugin / Settings windows are implemented, but final live checks
   are pending. Vital appeared during the first pass; later captures showed an empty native view. The terminal
   helper reported `AXIsProcessTrusted() == false`, so activation and synthetic input were unreliable. Verify the
   execution host's permissions, then test Vital and a resize-capable editor: visibility, resizing, tab changes,
   covering/pinned windows, keyboard forwarding and reopening saved wrappers. Do not mark VERIFIED from a build or
   headless fallback screenshot. See D35 and known limitations.
2. **Platform CI (PF-002…PF-005).** CI definitions exist in `.github/workflows/ci.yml`; jobs now run (the owner supplied a
   macOS test failure on 2026-10-22). The Playlist selection-loop test no longer assumes a full loop fits a fixed
   wall-clock window. Recheck remote results and fix remaining failures; record evidence separately for Apple
   Silicon, Intel, Windows and Linux. Earlier pending-job reports remain historical.
3. **Real-plugin pass.** Exercise Find plugins…, factory and `.vstpreset` presets, parameter automation, delay
   compensation and the opt-in process bridge. Bridged native editors remain unavailable. Windows/Linux native
   embedding needs `HWND` / `X11EmbedWindowID` and Linux `IRunLoop` support.
4. **Hands-on interaction regression.** Repeat the owner's note drawing, sampler previews/dragging, waveform drops,
   rack multiselection and bounded scrolling, mixer selection, track mute locks, window raising and typing-keyboard
   workflows. Automated coverage and offscreen layout review exist; they do not replace audible/native checks.

## Next implementation work

1. **Finish editor/context menus from the FL inventories.** Main menus, settings pages, toolbar menus and the core
   editor menus are in. Remaining work includes editor View options, Piano Roll Time markers / Target control /
   Group, Riff machine and chord progressions, Claw machine, Playlist picker options and performance mode, Mixer
   groups/docking/multiselection, Channel Rack Zip / Unzip and display options. Playlist track modes, grouping,
   icons, individual heights, lock-to-content and consolidation are disabled until their model/engine support exists.
   Continue documenting FL's Browser top bar, clip/note/control context menus and floating View windows before
   implementing them; preserve clean-room evidence in `docs/research/`.
2. **Audio and automation.** More decoders (INS-008), clip fades, visible automated controls (AUT-013), point modes
   (AUT-012), tempo automation (AUT-008), and a larger automation editing surface.
3. **MIDI devices and recording.** MIDI input (WF-003), MIDI recording (REC-002), then audio input/recording according
   to dependencies. Open an exported MIDI file in another application and record the result.
4. **Workspace and shortcuts.** Remappable shortcuts (UI-011), remaining inferred FL bindings, and automatic layout
   persistence. Detachable panels and explicitly saved custom layouts already exist.
5. **Packaging (PF-006)** and `.flp` follow-ups (APP-022): automation curves, equivalent mixer effects/routing,
   further arrangements and value-checked fixtures from older FL versions.
6. **Plugin formats and distribution.** CLAP (PLG-001) and AU (PLG-003); Lime Native's nine standalone VST3 bundles now live in `native-plugins/` (PLG-012).
   Complete hands-on DAW/editor checks and validate Windows/Linux/Intel builds; see `native-plugins/VALIDATION.md`.

## Research and verification rules

FL Studio is the behavior reference; ArtCraft informs visual styling only. Confirm permissions for the current host
before synthetic live input. Outstanding observations include Alt+wheel vertical zoom, in-clip automation point
interaction, FL's STEP/SLIDE automation options, and bindings marked INFERRED. Maintain `menus.json`,
`keybindings.json` and `settings.json` with code, then regenerate their Markdown views. Historical session notes
remain evidence of what was tested at that time, not a promise that later changes were manually reverified.
