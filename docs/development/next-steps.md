# Next steps for agents (prioritised)

Current as of 2026-10-27, after v0.1.0-alpha.1. Read the feature record and its dependencies in
[features.json](../roadmap/features.json) before implementing anything; verification evidence goes in
[progress.md](progress.md). Hands-on checks and owner decisions live in
[Human verification needed](human-verification.md).

## Development

Work toward feature and workflow parity with FL Studio, including implementing new features. Choose actionable
work from the roadmap in P0 → P3 order and respect dependencies; the implementation list below provides starting
points. Confirm existing behavior and recorded FL research before deciding what needs implementing.

If testing or research requires human input, unavailable hardware, platform permissions, licensed plugins,
credentials or an owner decision, record the exact outstanding check and prerequisites in
[Human verification needed](human-verification.md), link any partial evidence, and keep the corresponding
feature status and [known limitations](../roadmap/known-limitations.md) accurate. Continue with independent
parity work and new features rather than stopping development at that verification gap. Ask for blocking input
when needed, while continuing work that does not depend on the answer.

Implement complete feature workflows and run the automated tests and local checks available to the agent. A build or
headless test does not establish live GUI/audio behavior or verification on another platform. Do not promote
IMPLEMENTED to VERIFIED until the required evidence exists.

## Implementation

1. **Editor and context menus from the FL inventories** ([menus.md](../roadmap/menus.md)): other editor View
   options, Piano Roll Time markers, further Target control properties (PR-016), Group, Riff machine, chord
   progressions and Claw machine; Playlist picker options, performance mode, track modes, icons,
   lock-to-content and consolidation; Mixer groups, docking and multiselection; Channel Rack display options. Document
   FL's Browser top bar, context menus and floating View windows in `docs/research/` before implementing them.
2. **Audio and automation.** Sampler host/native root, source, preset and undo synchronization (INS-021) is
   implemented (live GUI A/B check in [human verification](human-verification.md)); fixed-duration
   sampler/clip preparation and same-track pair crossfades (INS-007, PL-023) are now integrated.
   Next pursue listening/live verification, longer disk preparation, realtime tempo following
   and a larger automation editing surface. Start with the [sampler parity audit and stretch scope](../research/sampler-parity-audit.md);
   Ping-pong loops, waveform loop gestures and three independent filter outputs are now implemented;
   see the [control inventory and continuation](../research/sampler-controls-2026-10-27.md). Keep source metadata
   paths, host duration/chase and FLP settings losses explicit rather than assuming
   hosted P0 field migration establishes FL Sampler parity.
3. **Recording.** Latency compensation, punch/loop takes and per-track hardware I/O; sustained input-monitoring/drift and count-in alignment verification.
4. **Workspace and shortcuts.** Remappable shortcuts (UI-011), remaining INFERRED bindings, automatic layout
   persistence.
5. **Distribution (PF-006).** `.lime` opening from Finder and an embedded Windows icon;
   signing/notarisation implementation that can proceed without owner credentials. `.flp` import (APP-022):
   automation curves, mixer effects and routing, further arrangements and checked fixtures from older FL versions.
6. **Plugin formats.** CLAP (PLG-001) and AU (PLG-003); Linux editor embedding (`X11EmbedWindowID`, `IRunLoop`).
7. **Refresh the README Mixer screenshots** (`mixer.png`, `mixer-eq.png`): they predate the On + Mix insert slots.
8. **Test-suite gaps from the hostile-input audit** (see [progress](progress.md)): broader audio-decoder corruption
   coverage beyond the deterministic WAV/MP3/FLAC/AIFF corpus and opaque third-party plugin state payloads
   (base64 and host-envelope corruption now covered); UI edge cases (zero-size windows, empty projects in every
   editor, extreme zoom); `limeshot-audio` device-loss and stream-error tests; coverage (`cargo-llvm-cov`) and fuzz
   (`cargo-fuzz`) tooling to measure and extend the above; repeat the hostile-input run on Windows and Linux.

## Owner follow-up: alpha.1 Rev 1

The interaction/Playlist continuation is described in [Rev 1 notes](../research/alpha1-rev1.md).

1. **Windows Lime Native VST3 responsiveness (P1).** Reproduce slow/unresponsive native editors on
   a Windows machine; profile UI event delivery, state synchronization and host/native polling before
   changing timing. Record Windows build, hardware, affected bundles and reproducible interaction.
2. **Plugin wrapper overhaul (research first, P2).** Compare FL's plugin windows, especially the
   Settings tab, with LimeShot's wrapper. Record clean-room observations and concrete controls/layouts
   before implementation; preserve working native editor embedding, parameter edits and undo.
3. **Complete Rev 1 parity gaps.** Independent clip stretching and left-edge stretch, pitch-preserving
   methods, disk-sample zero crossings, STEP pulse/tension behavior, full arrangement merge policies,
   FLP multi-arrangement import and measured track audio activity. Keep partial statuses explicit.

## Research and verification rules

FL Studio is the behaviour reference; ArtCraft informs visual styling only. Use documented behavior and
clean-room observations to guide independent implementations; label unobserved behavior as INFERRED or UNKNOWN.
Checks still needing hands-on observation are listed in [Human verification needed](human-verification.md).
Confirm permissions on the current host before synthetic live input. Keep `menus.json`, `keybindings.json` and
`settings.json` in step with the code and regenerate their Markdown views.

## Checkout and delivery

Create and use branches in the main checkout at `~/Developer/limeshot` so builds and tests reuse its Cargo cache.
Do not create separate worktrees or clones for routine branch work; preserve local changes when switching.
This supersedes the earlier separate-worktree policy and must be repeated in handoffs. Carry work
through implementation, validation, documentation, commit and push to the feature branch and its PR.
Never push directly to main or merge automatically, and create further sessions only when the owner asks.
