# FL Sampler controls and Lime implementation — 2026-10-27

## Evidence boundary

**OBSERVED:** FL Studio Producer Edition 26.1.1 build 5313, Apple Silicon, macOS 26.6.2.
The owner opened the 808 Kick sampler's Sample page and then its Envelope/Instrument page.
Computer-use screenshots showed both pages; keyboard F6 worked, while pointer actions repeatedly
returned `noWindowsAvailable`. The filter popup was closed in the captured view, so only its selected
`Fast LP` label is a fresh observation. No sample files, FL program files or bundled resources were
extracted, modified or copied. No new audio A/B, loop-turn measurement or listening equivalence is claimed.
The observed control presence below does not establish its DSP behavior.

**INFERRED from current official documentation:**
[Sampler](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_sampler.htm),
[Envelope/Instrument](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_ins.htm),
[Miscellaneous](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_misc.htm).
The existing [parity audit](sampler-parity-audit.md) remains the broad gap inventory. Exact FL algorithms,
resampling quality, filter responses, interpolation boundaries and knob mappings are **UNKNOWN**.

## Observed Sample page inventory

| Section | Visible controls |
| --- | --- |
| Wrapper | Sample, Instrument and Misc tabs; On, Pan, Volume, Pitch, Range, Mixer track |
| File | Filename, load, locate and remove buttons |
| Content | Keep on disk; Resample; region/slice metadata switches; declick mode selector |
| Playback | Start offset; loop Start/End; Use loop points; Ping pong loop |
| Time stretching | Pitch, multiplier, time and mode (Resample selected) |
| Precomputed effects, basic page | DC removal, normalization, reverse, polarity, stereo fade/swap; sample start, length, in/out, crossfade and trim |
| Sample display | Waveform and miniature zoom/scroll strip |

The visible start-offset and precomputed sample-start controls are distinct. Lime currently exposes
one trim window; it must not claim that every FL start/length/offset operation is represented.
The second precomputed-effects page was not opened in this pass.

## Observed Envelope/Instrument page inventory

The tabs were Panning, Volume, Mod X, Mod Y and Pitch. Volume showed envelope enable, a draggable
curve, delay/attack/hold/decay/sustain/release controls, three tension controls and a tempo switch.
The LFO section showed sine/triangle/square choices, delay, attack, amount, speed, tempo and global.
The filter section showed Mod X/Mod Y controls and `Fast LP`. The bottom displayed root C5,
reset, main-pitch enable, add-to-key, fine tune, keyboard and key-range strip.

The manual distinguishes four envelopes from five LFO destinations; pan has an LFO but no envelope.
It documents LP/BP/HP/BS and steeper or SVF variants in addition to Fast LP. Similar output names do
not imply that Lime's filters reproduce those particular algorithms.

## Miscellaneous settings: documentation inventory

The Misc manual describes separate level modulation; maximum polyphony versus mono/portamento;
gate, note shift and swing mix; keyboard/velocity tracking; directional Cut/Cut-by relationships;
event arpeggiation and note-based echo. Root/key-range and pitch response also belong here.
Lime's existing 128-voice stealing, audio delay and destructive Piano Roll arpeggiator do not cover
these channel workflows. Mono and a one-voice limit need separate acceptance tests. A cut-group
implementation must define note release/tails, not assume instantaneous removal of mixed audio.
This page was not freshly inspected in FL during this pass.

## Delivery: loop editing, ping-pong and filter outputs

The implementation adds **Ping-pong loop** and **Filter type** after the existing 16 sampler
parameters. Existing IDs 0–15 retain their meanings. New defaults are ping-pong off and low-pass;
legacy JSON, component states and sampler presets therefore keep their former configuration.
Both fields migrate with legacy samplers, persist in component/preset state, and are exposed to
host parameter automation/reset/undo. Filter outputs are Lime's independent TPT state-variable
low-pass, band-pass and high-pass, not FL's Fast LP or other proprietary responses.

Ping-pong reflection operates on the first and last frame centers of the effective loop, without
repeating a turn endpoint. Modulo folding handles pitch steps that cross multiple loop lengths in
constant time. The source retains requested direction separately from current bounce direction:
parameter polling must not reset a turn at a new block. Reversing changes the requested direction;
disabling ping-pong restores ordinary forward/reverse looping. The existing four-source-frame
minimum remains. Release envelopes and one-shot/held-loop semantics remain separate controls.

Loop seek/chase now folds position through the loop instead of ending a looped source outside its
playback window. Near-integer output-frame offsets are snapped within floating-point precision
before mapping; otherwise tiny conversion errors could select opposite sides of a loop seam.
Normal reverse wrap also stays inside the half-open loop rather than occasionally selecting its
exclusive end. This is an independent implementation choice; matching FL's fractional endpoint
convention is unverified. Filters do not reconstruct their full prior history on a seek.

The native waveform shows dedicated bottom loop handles, separate from trim handles and audition.
Dragging an endpoint edits it; Shift-drag preserves loop width and clamps the pair at the source
bounds. Both parameter gestures end together, allowing the host to capture one state/undo step.
Numeric loop endpoints remain available. The editor grows vertically to fit the additional controls
and keep them above the footer; no runtime assets are added.

Disk-backed samples use the same position math and filter path as embedded samples. No audio-thread
allocation, file access or waiting is added. Very short loops or large pitch steps can stress disk
read-ahead; existing realtime misses remain silent and counted. This does not add loop crossfading
or solve interpolation/antialiasing quality at extreme pitch.

## Remaining implementation order

1. **INS-007 / PL-023:** independent duration/pitch preparation with exact clip/trim/loop mapping.
   Rev 1's right-edge resampling changes shared sampler pitch; it is not pitch-preserving stretching.
2. **INS-020 / INS-014:** volume-envelope enable, delay/hold and tension, then mapped pitch/filter
   envelopes and LFOs. Keep defaults backward compatible; define tempo-automation behavior before
   adding tempo toggles. Filter families beyond the three independent outputs remain absent.
3. **INS-017 / INS-018:** reversible worker-prepared sample processing and source metadata/history.
   Preserve original PCM/source identity; do not destructively normalize the only stored sample.
4. **INS-019:** independently specified declick modes, including source bleeding around clips.
   Existing ADSR/clip fades do not establish those semantics.
5. **INS-011 / INS-022 / INS-024:** per-channel voice policies, cut groups and event transforms.

For the first stretch milestone, prepare immutable derived PCM on a worker while retaining original
source data and a processing recipe. Validate output limits before adoption, cache by source plus
recipe, and map all displayed trim/loop/clip coordinates through one original-to-prepared frame
transform. Host preview duration, fit, chase and export must consume the same mapping. Save/reopen,
presets and undo must preserve the recipe without requiring the original file for embedded audio.
A fixed prepared mode is not realtime tempo-following stretch; tempo automation needs a later bounded
realtime design. The [existing audit](sampler-parity-audit.md#time-stretch-implementation-scope)
describes algorithm candidates and quality gates; none has been adopted in this change.

## Validation and remaining observation

Tests cover independently expected forward/reverse turn sequences, large steps, repeated parameter
polls, loop seeks, stereo coherence, filter rejection, old state defaults, presets, host undo/reset,
waveform gesture routing and native control geometry. Hosted comparisons use 44.1/48/96 kHz and
blocks 1/64/333/1024; disk/embedded comparisons assert no realtime allocation.

Live checks still needed: FL loop endpoints at fractional positions and high pitch; Shift movement
at both bounds; note-off after a bounce; comparisons of each FL filter family and envelope timing;
Lime native gestures/undo/save/reopen in the live GUI; perceptual quality; Windows/Linux and standalone
hosting in FL. Observed UI control presence is not promotion to full feature verification.

Local result: macOS 26.6.2 arm64 debug, 485 workspace tests and 26 native tests passed; all twelve
rebuilt VST3 bundles passed the exported ABI verifier. Both workspaces passed strict Clippy and
formatting, and repository/inventory checks passed. The
[rendered native sampler](../screenshots/current/lime-sampler-loops.png) was visually inspected.
Offscreen rendering required host GPU access; bridge tests required local sockets. These results
verify Lime's defined behavior, not FL listening parity or other-platform runtime.
