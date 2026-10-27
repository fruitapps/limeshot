# Next steps (prioritised)

Current as of 2026-10-25, after v0.1.0-alpha.1. Read the feature record and its dependencies in
[features.json](../roadmap/features.json) before implementing anything; verification evidence goes in
[progress.md](progress.md).

## Verification

1. **Native plugin embedding (PLG-006).** Use `live_check` with Screen Recording and Accessibility granted. Test Vital
   and a resizable editor such as Serum: keyboard forwarding, OS drops onto native views, pinned and covering
   windows, host-edge resizing and reopening saved wrappers. Partial results:
   [live-plugin-checks-2026-10-25.md](live-plugin-checks-2026-10-25.md). Do not mark VERIFIED from a build or a
   headless screenshot.
2. **Platforms (PF-002…PF-005).** Install the alpha packages on real Windows, Linux and Intel macOS machines: GUI,
   audio device, plugin editors (Windows), parameter fallback (Linux), drops and the installers' uninstall paths.
3. **Real-plugin pass.** Find plugins…, factory and `.vstpreset` presets, parameter automation, delay compensation
   and the process bridge with commercial plugins.
4. **Hands-on interaction regression.** Note drawing, sampler previews and drags, waveform drops, rack
   multiselection and scrolling, Mixer selection and insert reordering, mute locks, window raising and the typing
   keyboard.
5. **Recording and MIDI.** Audio recording from a real input and a hardware MIDI keyboard; open an exported MIDI
   file in another application.

## Decisions for the owner

* **Piano Roll range looping.** Pattern mode ignores loop ranges, yet the UI still sends the Piano Roll's range to
  the renderer, where it has no effect. Either loop Pattern playback over the Piano Roll range only, or stop sending
  it and document FL's behaviour.

## Implementation

1. **Editor and context menus from the FL inventories** ([menus.md](../roadmap/menus.md)): other editor View
   options, Piano Roll Time markers, further Target control properties (PR-016), Group, Riff machine, chord
   progressions and Claw machine; Playlist picker options, performance mode, track modes, grouping, icons, heights,
   lock-to-content and consolidation; Mixer groups, docking and multiselection; Channel Rack display options. Document
   FL's Browser top bar, context menus and floating View windows in `docs/research/` before implementing them.
2. **Audio and automation.** Crossfades and time stretching (PL-023), a larger automation editing surface.
3. **Recording.** Input monitoring, latency compensation and count-in.
4. **Workspace and shortcuts.** Remappable shortcuts (UI-011), remaining INFERRED bindings, automatic layout
   persistence.
5. **Distribution (PF-006).** Authenticode and Developer ID signing with notarisation; `.lime` opening from Finder;
   an embedded Windows icon. `.flp` import (APP-022): automation curves, mixer effects and routing, further
   arrangements and checked fixtures from older FL versions.
6. **Plugin formats.** CLAP (PLG-001) and AU (PLG-003); Linux editor embedding (`X11EmbedWindowID`, `IRunLoop`).
7. **Refresh the README Mixer screenshots** (`mixer.png`, `mixer-eq.png`): they predate the On + Mix insert slots.
8. **Test-suite gaps from the hostile-input audit** (see [progress](progress.md)): broader audio-decoder corruption
   coverage beyond the deterministic WAV/MP3/FLAC/AIFF corpus and opaque third-party plugin state payloads
   (base64 and host-envelope corruption now covered); UI edge cases (zero-size windows, empty projects in every
   editor, extreme zoom); `limeshot-audio` device-loss and stream-error tests; coverage (`cargo-llvm-cov`) and fuzz
   (`cargo-fuzz`) tooling to measure and extend the above; repeat the hostile-input run on Windows and Linux.

## Research and verification rules

FL Studio is the behaviour reference; ArtCraft informs visual styling only. Confirm permissions on the current host
before synthetic live input. Still to observe in FL: Alt+wheel vertical zoom, in-clip automation point dragging,
STEP/SLIDE automation options and bindings marked INFERRED. Keep `menus.json`, `keybindings.json` and `settings.json`
in step with the code and regenerate their Markdown views.


## Alpha.1 follow-up (2026-10-26)

Owner-requested interaction and disk-sampler fixes are in `codex/alpha1-polish`; see
[research and design](../research/alpha1-sampler-and-fades.md). The macOS live pass covered native value typing and Serum internal-corner shrink/grow.
Repeat host-edge resizing with other resizable plugins and long disk-backed playback/seek checks on Windows
and Linux before promoting the native feature statuses. Disk cache misses and smooth fade/crossfade limits remain recorded.

Create and use branches in the main checkout at `~/Developer/limeshot` so builds and tests reuse its Cargo cache.
Do not create separate worktrees or clones for routine branch work; preserve local changes when switching.
This supersedes the earlier separate-worktree policy and must be repeated in handoffs. Carry work
through implementation, validation, documentation, commit and push. Never merge automatically, and create
further sessions only when the owner asks.
