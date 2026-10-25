# Plugin and workflow fixes — 2026-10-25

## Session and base

New managed worktree `plugin-workflow-fixes/limeshot`, branch `codex/plugin-workflow-fixes`, based on
main `3e2fd641`. PRs #7 and #10 are merged; the GitHub open-PR query returned no continuations, and a final
fetch confirmed the same main revision. Older checkouts were preserved. Finish with a new attached draft PR;
never merge automatically.

## Changes

* Plugin wrappers use the workspace title painter: minimize/restore, pin and close, no maximize/detach.
  Plugin resize requests update the outer wrapper before stale host geometry can overwrite them. Host resizing
  negotiates plugin constraints and reconciles the wrapper with the accepted editor dimensions.
* Native macOS wrappers use an AppKit preset menu above the embedded NSView, keeping the editor visible.
  Parameter/headless wrappers use an egui popup. Browser sample drags temporarily hide native child views so
  the host can receive the release and replace the sampler through undoable commands.
* A local egui-baseview 0.9.0 patch binds its retained GL context before destroying painter resources. Upstream
  destroyed GL names in whichever context happened to be current, potentially deleting host resources.
  License texts and patch rationale are retained in native-plugins/vendor/egui-baseview. The host additionally
  clears/restores its context during editor removal, and protects logic/UI callbacks. An offscreen GL test
  verifies suspend/restore on this Mac.
* Playlist/rack lock indicators use a padlock; direct clicks still change state while locked. Mixer inserts have
  On buttons, Shift-click mute locks, and solo resolution that preserves locked active inserts. The new lock field
  is defaulted for older projects and saved with undoable commands.
* Every loaded sampler is available in the Playlist audio picker, including ordinary rack channels. Existing
  sources are reused. OS audio drops onto the visible Playlist grid snap to pointer time/track, with successive
  tracks for multiple files; rack drops create channels; sampler-wrapper drops replace their source.
* Native sample Load and file drops share the host WAV/MP3/FLAC/Ogg/AIFF decoder. Explicit imports report actual
  decode/size/rate failures before adding a broken channel. The existing 4 million stereo-frame / 8–384 kHz
  bound is retained (about 83 seconds at 48 kHz), including MP3. Longer sources still require shorter files.
* Redundant native plugin title rows are removed; preset/bypass controls move to the footer. Parameter IDs,
  state schema and editor dimensions are unchanged.
* Playlist/Piano Roll horizontal zoom spans 0.25–16,384 pixels per beat across menus, tools and wheel paths.
  Vertical ranges are 12–640 pixels per Playlist track and 2–160 per Piano Roll key.

## Verification and remaining work

Host workspace tests, strict all-target Clippy/build, both formatting checks, native tests and freshly rebuilt
actual-binary checks are recorded in progress.md and native-plugins/VALIDATION.md. Offscreen app/native renders
were inspected. Synthetic MPEG-1 Layer III silence tests exercise real MP3 decoding and embedded state; FLAC
also checks sample equality. Neither is a broad damaged/encoded-MP3 corpus.

Screen Recording and Accessibility checks returned false for this execution host. Final live Serum resize,
native preset menu selection, occlusion, typing and repeated instrument/effect close/reopen remain pending.
PLG-006/PLG-012 remain IMPLEMENTED. Native Windows/Linux embedding and broader platform runtime checks remain open.

Every future development session must create a NEW codex/ branch, NEW separate managed worktree and NEW attached
draft PR based on the latest unmerged continuation after checking merge state. Preserve older checkouts, carry
implementation through validation/documentation/commit/push, never merge automatically, and create further sessions
only when the owner asks. Repeat this policy in every handoff.

## Live follow-up

The owner enabled permissions and requested live checks in this same PR session. See
[live results](live-plugin-checks-2026-10-25.md), including the repaired modal re-entrancy abort, partial verification,
remaining inconclusive interactions, isolated harness and recovery incident/prevention.
