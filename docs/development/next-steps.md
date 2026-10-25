# Next steps (prioritised)

Current as of 2026-10-25. First Release 0.1.0-alpha.1 has been released.
Read the feature record and its dependencies before implementation.
Completed work and verification evidence are retained in [progress.md](progress.md); statuses are canonical in
[features.json](../roadmap/features.json).

## Standing session policy
Every new development session must use a new isolated branch and create a NEW draft PR, rather than adding
feature work to an earlier session’s PR. Inspect merge state and stack on the latest unmerged continuation, but only if edits are based on another or in sequence.
Finish implementation, validation, documentation, commit and push, attach the new PR.
Repeat this policy in every future session handoff before these docs merge. Do not create further sessions unless asked.

## Immediate verification

1. **Native plugin embedding (PLG-006).** Internal Plugin / Settings windows are implemented, but final live checks
   are pending. Vital appeared during the first pass; later captures showed an empty native view. The terminal
   helper reported `AXIsProcessTrusted() == false`, so activation and synthetic input were unreliable. Verify the
   execution host's permissions, then test Vital and a resize-capable editor: visibility, resizing, tab changes,
   covering/pinned windows, keyboard forwarding and reopening saved wrappers. Do not mark VERIFIED from a build or
   headless fallback screenshot. See D35 and known limitations.
2. **Platform CI (PF-002…PF-005).** Windows x86-64 and Linux x86-64/ARM64 pass native headless CI; Windows ARM64
   cross-build and Apple Silicon checks pass at the commits recorded in [platform-validation.md](platform-validation.md).
   The `macos-15-intel` replacement has now passed too (full run `39826217710`, commit `38d9a7b`). Check
   the latest PR commit and fix actual failures. Billing no longer prevents jobs from executing. Main run
   `40211683423` at `5ab09c9` passes Windows but exposes Linux hosted-plugin crashes and a macOS sample-import
   test race. See [platform validation](platform-validation.md) for the repair and current evidence.
   Native GUI/audio-device validation stays separate from headless CI.
3. **Real-plugin pass.** Exercise Find plugins…, factory and `.vstpreset` presets, parameter automation, delay
   compensation and the opt-in process bridge. Bridged native editors remain unavailable. Windows `HWND` embedding
   is implemented but untested on hardware; Linux needs `X11EmbedWindowID` and `IRunLoop` support.
4. **Hands-on interaction regression.** Repeat the owner's note drawing, sampler previews/dragging, waveform drops,
   rack multiselection and bounded scrolling, mixer selection, track mute locks, window raising and typing-keyboard
   workflows. Automated coverage and offscreen layout review exist; they do not replace audible/native checks.

The owner deferred live UI checks while the Mac is locked; continue repository work and offscreen review without
waiting for unlock. The latest integrity audit is [repository-audit.md](repository-audit.md).

## Next implementation work

1. **Finish editor/context menus from the FL inventories.** Main menus, settings pages, toolbar menus and the core
   editor menus are in. Piano Roll now has velocity/pan Target control and keyboard-label/visible-note-label View options
   ([research and limits](../research/piano-roll-view-controls.md)). Remaining work includes other editor View options,
   Piano Roll Time markers / further Target control properties /
   Group, Riff machine and chord progressions, Claw machine, Playlist picker options and performance mode, Mixer
   groups/docking/multiselection, and Channel Rack display options (manual Zip / Unzip and Auto zip empty channels are complete). Playlist track modes, grouping,
   icons, individual heights, lock-to-content and consolidation are disabled until their model/engine support exists.
   Continue documenting FL's Browser top bar, clip/note/control context menus and floating View windows before
   implementing them; preserve clean-room evidence in `docs/research/`.
2. **Audio and automation.** Crossfades/time stretching (PL-023) and a larger
   automation editing surface. Tempo automation (AUT-008), compressed decoders, clip fades, visible automated controls and point modes are implemented.
3. **MIDI devices and recording.** MIDI input and MIDI/audio recording exist; verify with real devices, then add monitoring,
   latency compensation and count-in according to dependencies. Open an exported MIDI file in another application.
4. **Workspace and shortcuts.** Remappable shortcuts (UI-011), remaining inferred FL bindings, and automatic layout
   persistence. Detachable panels and explicitly saved custom layouts already exist.
5. **Packaging (PF-006)**: installers for all three platforms exist ([release packaging](release-packaging.md)); run the
   release workflow, try the macOS PKG/DMG and Linux `.deb`/tarball on hardware, and add Authenticode/Developer ID
   signing and notarisation. `.flp` follow-ups (APP-022): automation curves, equivalent mixer effects/routing,
   further arrangements and value-checked fixtures from older FL versions.
6. **Plugin formats and distribution.** CLAP (PLG-001) and AU (PLG-003); Lime Native's twelve standalone VST3 bundles now live in `native-plugins/` (PLG-012).
   Complete hands-on DAW/editor checks and validate Windows/Linux/Intel builds; see `native-plugins/VALIDATION.md`.

## Research and verification rules

FL Studio is the behavior reference; ArtCraft informs visual styling only. Confirm permissions for the current host
before synthetic live input. Outstanding observations include Alt+wheel vertical zoom, in-clip automation point
interaction, FL's STEP/SLIDE automation options, and bindings marked INFERRED. Maintain `menus.json`,
`keybindings.json` and `settings.json` with code, then regenerate their Markdown views. Historical session notes
remain evidence of what was tested at that time, not a promise that later changes were manually reverified.


## Latest continuation: Piano Roll controls

`codex/editor-view-options` started at `68259aa` after #8/#9 merged; PR #10 now integrates updated #7 at `5cf0870`.
Pan editing and the implemented View subset have headless interaction coverage and offscreen review. PR-016 stays
IN_PROGRESS for release/Mod X/Y/fine pitch; PR-030 records the label subset. Latest integration checks: 399 workspace tests
(141 headless UI), strict all-target Clippy/build, rustfmt and inventories/reference checks. Use the current research
notes for fallback choices and unverified FL details; PLG-006's live verification is still pending.

Every future session must create a NEW `codex/` branch, NEW separate managed worktree and NEW attached draft PR,
stacked on the latest unmerged continuation after checking merge state. Leave older checkouts intact, never merge
automatically, and create further sessions only when the owner asks.


## Latest continuation: plugin/workflow fixes

`codex/plugin-workflow-fixes` starts from main `3e2fd641` (#7/#10 merged). See the
[session handoff](handoff-plugin-workflow-fixes.md). Native window sizing/title controls, AppKit host preset
menus, GL teardown, sampler compressed loading/drops, track locks and extended zoom have local automated
coverage. Repeat live Serum resizing, native preset selection and close/reopen checks when the execution
host has Screen Recording and Accessibility. Sampler sources are bounded to 6 million stereo frames
(about 125 seconds at 48 kHz); longer sources still need trimming. Native Linux embedding remains pending (Windows: see below).

Future sessions retain NEW codex/ branch, NEW managed worktree and NEW attached draft PR; preserve older
checkouts, inspect unmerged continuations, validate/document/commit/push and never merge automatically.
Create further sessions only when the owner asks.


### Live follow-up to plugin/workflow fixes

Current host permissions are enabled. [Live evidence](live-plugin-checks-2026-10-25.md) records partial native
checks and the repaired modal-dialog crash. Do not treat all earlier permission-blocked items as verified:
external OS drops, keyboard, pin/occlusion, host-edge sizing and native Browser-drop/modifier results remain open.
Use `live_check` with a disposable project and shared autosave disabled. New sessions now preserve unclaimed
recovery copies instead of overwriting them or deleting them on normal quit.


## Latest state: sampler/mixer workflow merged (PR #12, then PR #11 into main)

Both PRs merged on 2026-10-25 together with two CodeRabbit follow-up commits (`a7441e1`, `1c20d45`); see the
[sampler/mixer handoff](handoff-sampler-mixer-workflow.md) and the latest progress entries. Open items:

1. **Live checks** on the isolated `live_check` harness: native sampler drag passthrough, plugin-button
   toggle/raise, OS file drops onto the Playlist, Y scrubbing of audio clips, insert drag-reordering and the Mix knob.
2. **Decide on Piano Roll range looping.** The sampler/mixer pass made the Piano Roll time selection loop Pattern
   playback; CodeRabbit's `1c20d45` made the renderer ignore every loop range in Pattern mode. The UI still sends
   the Piano Roll range, which now has no effect. Either restore a Pattern-mode loop for the Piano Roll range
   only, or stop sending it and document the FL behaviour.
3. **Refresh the README Mixer screenshots** (`mixer.png`, `mixer-eq.png`); they predate the insert-slot redesign.
4. **Windows/Linux:** the Windows fixes (class ids, `HWND` embedding, Explorer drop position, no console) need a
   pass on Windows hardware; Linux embedding and drop positions and platform CI remain open. GitHub Actions was
   last blocked by account billing.

Future sessions keep the standing policy above: a NEW `codex/` branch, a NEW managed worktree and a NEW attached
draft PR, stacked on the latest unmerged continuation; never merge automatically.
