# Live plugin checks — 2026-10-25

macOS Apple Silicon, during PR #11. Screen Recording and Accessibility were granted. Used locally rebuilt debug
application and plugins, a disposable demo project with a Serum 2 channel, and computer-use app controls. Native
sampler Load initially crashed; it was repaired and retested below.

## Observed results

| Check | Result and scope |
|---|---|
| Lime Sampler native rendering | PASS: editor and surrounding host render together; redundant native title row is absent and preset/bypass controls are in the footer. |
| Plugin/Settings | PASS: sampler switches to Settings and back; host stays visible. |
| Host preset dropdown | PASS for visibility: AppKit menu stays above the native sampler without hiding it. This menu offered Save preset as; factory/file preset selection was not exercised. |
| Native sampler close/reopen | PASS across multiple opens/closes, including closing after the repaired MP3 import; no host black screen. |
| Serum 2 internal resize | PASS for a shrink through Serum’s own bottom-right control: editor and outer wrapper shrink together, with matching right/bottom bounds. This does not establish all host-edge resize cases. |
| Serum minimize/restore | PASS: title-only wrapper hides the native view; restore brings back the editor. |
| Native Lime Limiter | PASS: rendered editor and two close/reopen cycles leave the host and Mixer rendered. |
| Native Load MP3 | Initially FAIL (backend RefCell re-entrancy); after patch, PASS: native Open panel accepts the synthetic MP3, editor shows its filename, 44.1 kHz and ~0.21 seconds, with loaded-state confirmation. |
| Native Browser sample drop | NOT VERIFIED: two automation drags left the sampler source unchanged. Ordinary egui title dragging also appeared unchanged with the same driver. A new headless regression confirms real Browser-row pointer drags create rack samplers, but does not prove the live native path. |
| Mixer On/solo | Direct On and right-click solo changed visible states. Modifier-click locking was inconclusive: the driver’s Shift-click produced no padlock. Headless lock coverage remains passing. |
| External OS drops, native keyboard forwarding, pin/occlusion stress, host-edge resize | NOT VERIFIED by this pass. No upgrade of PLG-006/PLG-012 to VERIFIED. |

The MP3 fixture is eight synthetic MPEG-1 Layer III silence frames, rather than a private source recording; it
does not establish long-MP3 support.

## Crash and repair

Clicking Lime Sampler’s native Load button ran an AppKit modal event loop inside the editor’s active UI frame.
The backend re-entered on_event and borrowed that same RefCell to query pointer capture:
`vendor/egui-baseview/src/window.rs:953:44: RefCell already mutably borrowed`, followed by process abort.
The vendor patch now defers nested draw/resize (keeping the latest size) and lets the modal panel handle
nested input until the outer UI frame releases its state. Rebuilt all twelve native plugins and completed
the native MP3 Open-panel import successfully. This is recorded live evidence, not an automated modal test.

## Test isolation and recovery incident

The first temporary bundle ran the ordinary application and inherited its shared autosave directory. Its
five-minute autosave replaced an older unclaimed Untitled recovery slot with the disposable test project.
The older slot was not backed up first and could not be restored; no local Time Machine snapshots were listed.
The resulting test recovery files were preserved locally under `/tmp/limeshot-live-recovery-backup` and left
unchanged through later runs. They are not a backup of the lost prior recovery.

Added `examples/live_check.rs`: the real production UI with autosave disabled, no recovery offer, and no
preference save on exit. Added an autosave ownership guard: a new session cannot overwrite an existing
unclaimed recovery, and normal quitting clears only its own copy. Explicit Recover/Discard still resolve it.
An automated disk-backed regression checks refused replacement, preservation on normal cleanup, and adoption.
Use this harness and a disposable project for future live checks; the instruction is also in AGENTS.md.

## Evidence

* [Sampler preset menu remains above the editor](../research/evidence/screenshots/live-2026-10-25-sampler-host-presets.png)
* [Host after sampler close](../research/evidence/screenshots/live-2026-10-25-sampler-closed-host-visible.png)
* [Serum before resize](../research/evidence/screenshots/live-2026-10-25-serum-before-resize.png)
* [Serum editor and wrapper after resize](../research/evidence/screenshots/live-2026-10-25-serum-resized-wrapper.png)
* [Native sampler after MP3 import](../research/evidence/screenshots/live-2026-10-25-native-mp3-loaded.png)
* [Native limiter open](../research/evidence/screenshots/live-2026-10-25-limiter-native-open.png)
* [Host after limiter close](../research/evidence/screenshots/live-2026-10-25-limiter-closed-host-visible.png)
