# Handoff: hosted P0 parity

2026-10-23. This owner-requested development session uses the NEW branch `codex/hosted-p0-parity` in the separate
managed worktree `/Users/sebastian/.codex/worktrees/hosted-p0-parity/limeshot`. It starts at PR #8's verified head
`31d09f913c3afbb1823f684c27ac633bcafb2962`. PRs #7 and #8 were both open and unmerged when inspected; #8 targets
#7. This session creates its own NEW draft PR targeting `claude/lime-vst3-default`; no merge is authorized.
Older checkouts, including `limeshot-vst3`, were left intact. Build caches were cloned into this worktree.

## Implemented

* PLG-024: Playlist clips run inside Lime Sampler with offsets, play/locate/selection chase, independent same-key
  overlap, duration gating and fades. Fixed-size versioned SysEx carries clip metadata; no built-in playback returns.
* PLG-025: standard VST3 note-pan expressions plus a native channel-pan message preserve additive balance before
  the pan law, including automation and releases. Events retain note/expression pairing at equal timestamps.
* PLG-029: exact track EQ is a dedicated hosted Lime EQ after ten inserts and before the fader. Appended hidden
  parameters cover all legal legacy bands, including shelf widths. Saved bands remain canonical; inline edits and
  processor state share one undo step. Sanitization retains ten inserts plus one validated track-EQ processor.
* PLG-017…019: Lime Gate, Drive and Chorus append stable plugin IDs 9–11; conversion carries their settings. The
  twelve standalone bundles have native editors, presets, bypass and automation. Existing IDs/ranges are preserved.
* PLG-020: converted mixer key-track selection feeds Comp's stereo auxiliary bus. The hosted insert card offers
  an undoable selector for routed source tracks. Selecting a new source recreates the effect on the control thread.
* PLG-022: Rack waveforms and Settings/Playlist preview use cached embedded PCM, even with no source file. Content
  keys share decoded PCM across parameter edits; no temporary sample files are required. Embedded-only drags use
  the existing audio clip source payload. External Browser sample drops still preserve target sampler settings.
* PLG-026: known Lime controls get native-unit type-in, normalized copy/paste, reset, named automation and
  undo through the shared control menu. Arbitrary third-party parameter menus remain generic.
* PLG-027/028: actual block-maximum reduction and end-of-block native voice counts travel through bounded output
  SysEx, existing engine status atomics and Mixer/transport displays. Unknown plugins contribute zero voices.
  The event callback copies telemetry immediately; it never retains a borrowed SysEx pointer.
* Local and process-bridge paths carry key audio, pan and telemetry. The bridge retains its existing synchronous
  socket I/O/timeout limitations; this session does not make bridging real-time safe. Lime Test Gain/Tone remain.

## Validation

Final local results: **375 root tests including 136 UI tests, 14 native tests**, formatting and strict all-target
Clippy in both workspaces, 290-feature inventories and repository links/evidence checks all pass. All twelve release
and allocation-checked debug bundles pass actual-binary verification at three sample rates/four buffer sizes; all
twelve release signatures verify. The CLI demo renders 741207 frames at 44.1 kHz, peak −3.7 dBFS. Current release
hashes are in `native-plugins/VALIDATION.md`. The original non-automated comparison still asserts exact sample bits
and RMS difference 0; it has not been weakened.

Positive acceptance tests replace former silence/ignored-pan gap assertions. Real VST3 tests cover clip windows,
chase, overlap/fades/gates, independent note pan, exact EQ across full legacy ranges, routed Comp ducking, reduction
and actual voices. Engine scheduling tests exercise several block sizes. Native bridge tests compare actual local
and remote samples/telemetry. Offscreen Mixer EQ, sidechain, Playlist/rack and new native editors are inspected.

## Remaining validation and limits

GitHub Actions is blocked by owner account billing; no remote CI pass is claimed. pluginval is unavailable on this
host and was not rerun for these binaries. Windows/Linux/Intel runtime and broader native resizing, occlusion,
keyboard, detached/third-party editor behavior remain open. Current bundles are macOS arm64, ad-hoc signed.
Clip events are a Lime Sampler extension; other samplers do not implement it. Source audio is not time stretched.
Legacy compressor key-track automation still warns; dynamic audio-bus routing is not converted into a parameter.
Hosted value menus cover known Lime controls; arbitrary third-party unit parsers are not integrated.

## Next session policy

Every future development session must create a NEW `codex/` branch in a NEW separate managed worktree and a NEW
draft PR, stacked on the latest unmerged continuation after checking merge state. Do not add feature work to this
session's PR or PR #8. Leave older checkouts intact. Carry implementation through validation, docs, commit, push
and an attached draft PR. Never merge automatically. Create another chat/session only when the owner asks.
