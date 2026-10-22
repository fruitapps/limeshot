# Lime VST3 host validation — 2026-10-23

Scope: macOS Apple Silicon, `claude/lime-vst3-default`, Codex takeover continuation of draft PR #8.

## Automated and artifact checks

* DAW workspace fmt, all-target Clippy with warnings denied and full tests pass; 134 headless UI regressions remain.
* Native workspace fmt, all-target Clippy and 13 tests pass.
* Real bundles built with `python3 build.py`; exported-binary verifier passes all nine plugins at 44.1/48/96 kHz
  and buffer sizes 1/64/333/1024, including editor creation, MIDI offsets, automation and state recall.
* Conversion-equivalence test now checks exact float sample bits and permits only trailing silent frames to differ
  in length. RMS difference is 0. Automated EQ comparison remains approximate due smoothing/interpolation.
* macOS packaging produces `dist/LimeShot.app` and `dist/LimeShot-0.1.0-macos-arm64.dmg`, with nine VST3 bundles
  under `Contents/PlugIns`. Deep strict ad-hoc signature verification passes.
* Packaged CLI demo render: 741207 frames, 44.1 kHz, 16.81 seconds, peak −3.7 dBFS.
* CLI realtime test after conversion: MacBook Pro Speakers, 44.1 kHz, 324 callbacks over 2 seconds, peak −5.3 dBFS.
  The host reports 0 voices for plugins (PLG-028); the peak confirms non-silent audio callbacks. Sampled maximum
  load was 13.1%, which is an observation rather than a performance guarantee or listening assessment.
* New CLI integration tests verify converted scripted/demo saves and missing-plugin export failure before output
  creation. Offscreen Sampler wrapper layout was rendered and visually inspected.

## Live native rendering

Computer-use accessibility and screenshots were available. Opened the packaged app, loaded the demo, and opened
Kick's Lime Sampler editor. The editor rendered but the surrounding app went black; switching to Settings hid the
plugin and left the host black. Inspection showed that the native plugin's OpenGL renderer clears the thread's
current context, while glutin's macOS surface check only compares its bound NSView.

Added a host graphics-context guard in `limeshot-plugins`, captured before the first native attachment and restored
around the main viewport UI. Repeated using a temporary debug app bundle with the same Lime release plugins:
Sampler's native editor and surrounding host now render together, and switching to Settings shows the host sampler
waveform and parameter controls while retaining the Browser, toolbar and Playlist. This is a live rendering/tab
check, not a complete PLG-006 verification.

Not verified: resize/occlusion/pinning stress, detached viewport graphics, keyboard forwarding, third-party native
editors, native control automation/preset interaction, and Windows/Linux/Intel runtime. `pluginval` was not available
and was not rerun. GitHub Actions cannot execute due owner account billing; no remote CI pass is claimed.

## Remaining replacement work

Built-in playback is still present in the sequencer/engine. PLG-013 remains IN_PROGRESS. Newly recorded P0 losses
PLG-024…028 include hosted audio-clip playback, note pan, typed host menus, dynamics meters and voice statistics.
See [handoff-lime-vst3-replacement.md](handoff-lime-vst3-replacement.md) for the exact next implementation step.
