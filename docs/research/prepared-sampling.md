# Prepared sampling and arrangement

2026-10-27. Independent Rust DSP, fixed-duration preparation. INS-007 / PL-023 remain
IN_PROGRESS: realtime tempo following, loop-seam crossfades, listening and live/platform
verification are separate open work. No FL code, assets, or C++ DSP are used.

## Candidate gate

Source packages inspected before integration: `wsola` 0.1.0 and `timestretch` 0.15.0,
with their normalized Cargo manifests and implementation sources. Both declare MIT;
`timestretch` includes its license text, while the published `wsola` package does not
include a LICENSE file. Neither is a product dependency. The optional, isolated
[comparison harness](../../tools/stretch-eval/src/main.rs) pins both exact versions
and its lockfile; it does not enter the application/native workspace dependency graph.

Reproduce with `cargo run --release --manifest-path tools/stretch-eval/Cargo.toml`.
Pass a temporary output directory after `--` to write listening WAVs. The synthetic
fixtures are a tone, impulses, decaying percussion and a three-note chord, with
opposite-polarity stereo channels. [Recorded numeric results](evidence/stretch-candidates-2026-10-27.csv)
are from macOS arm64 release builds; timing is fixture-specific, not a benchmark guarantee.

* `wsola`: pure Rust, Vec/VecDeque streaming buffers; push/pull/flush allocate, so its
  streaming API is unsuitable for direct use in our callback. `flush` drains a windowed
  tail and clears priming; `reset` clears buffers/positions while retaining geometry and
  tempo. Full-buffer 48 kHz results contain 24,480 / 48,240 / 95,040 frames at 0.5/1/2×,
  rather than 24,000 / 48,000 / 96,000. Hann priming fades the start; candidate search
  uses unnormalized correlation across both channels. Not adopted.
* `timestretch`: pure Rust and forbids unsafe in its crate; allocation-heavy batch path
  builds preanalysis and a fresh engine, feeds source/flush padding, removes pipeline
  latency and returns the requested rounded length. No state is reused between batch
  calls. `engine/offline.rs` explicitly preserves the DJ engine's behavior of allowing
  content below 120 Hz to follow tempo near unity; wide ratios use WideKeylock/PV paths.
  This conflicts with a uniform independent-pitch sampler contract. Its public batch
  parameter validation also limits rates to 192 kHz. Not adopted. Its phase-vocoder
  implementation has phase locking and transient handling, but these source features
  and our numeric results do not establish good musical quality.
* Lime: independent stereo-linked WSOLA with shared normalized correlation, a bounded
  search, 20 ms Hann grains, normalized overlap-add, exact finite output, and worker-only
  sinc pitch/resample preparation. Identity bypass is exact. At these fixtures it keeps
  opposite-polarity stereo coherence and exact output length; pitch tests cover all
  supported source rates. Local CPU/memory cost is bounded by the declared frame/ratio
  limits. No external stretch code/dependency was adopted.

References: [WSOLA paper](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html),
[wsola API/source](https://docs.rs/wsola/0.1.0/wsola/),
[timestretch source](https://docs.rs/crate/timestretch/0.15.0/source/src/engine/offline.rs).
Numeric tone, RMS and polarity results do **not** establish listening quality. WSOLA
can repeat/smear transients and produce modulation on polyphonic material. No formant
preservation, transient markers, source BPM detection, or realtime tempo locking is claimed.

## Ownership and portable state

The host Channel Settings dialog changes all uses of that channel. Playlist's Prepare
this clip action instead clones the channel, including root, controls and mixer route,
then assigns only the chosen clip to it in one undoable command. Other clips and pattern
notes keep their original channel. The generated channel remains editable in the Rack.
Repeated preparation reuses a private channel while no other clips or notes use it, including clips in inactive arrangements. Removing
a clip can leave its channel for explicit Rack removal.

The existing `sample-v1` field continues to contain the playable PCM. Optional nested
`preparation-v1` stores the untouched original sample object and the mode, duration ratio,
and prepared pitch. Reprepare always starts from that original, never cascades processing.
Restore original reinstates the original object. Parameter IDs and defaults are unchanged;
old songs have no recipe and retain their existing resampling behavior. Ordinary channel
Pitch and keyboard transposition continue resampling **after** preparation; the preparation
panel's independent Pitch is baked into the immutable result. Source relocation updates
both the playable and original source hints. Embedded audio remains usable without its file.
Host/native state recall, wrapper presets and existing sampler gesture undo use this same
portable field. Control-only sampler presets retain the audio/recipe, as before.

## Coordinates, fitting and arrangement

Trim and loop endpoints are fractions of the buffer: an original frame at fraction `u`
maps to prepared fraction `u`, with rounding at the playback boundary. Reverse traverses
the prepared region backwards. This is a proportional coordinate mapping, not a claim of
sample-accurate transient correspondence through WSOLA's local search. Clip offset, chase,
fade position and gate length remain output seconds/ticks. Preparation does not move the
clip's source offset or arrangement start. Fit-to-sample length uses prepared PCM; the
preparation dialog automatically fits the chosen clip to the new region's duration minus
its offset. Beat fitting uses the current tempo once, including trim and channel Pitch.
Later tempo changes do not reprepare. To target bars, enter bars × beats per bar.
Waveform, audition, drag-as-clip and export use the same prepared PCM.

Smooth fades use cubic smoothstep `t²(3−2t)` with complementary gain through an overlap.
Native clip protocol v3 adds the smooth sentinel without changing the 96-byte envelope;
v1 linear and v2 tension messages retain their meaning. New native bundles are required.
Playlist can calculate paired crossfades on request, or persist Automatic crossfades on
overlaps. With the latter enabled, document edits recalculate same-track, two-clip overlaps
in the same undo step; moving clips apart restores their manual base fades. Nested,
coincident-start, muted and triple overlaps are skipped. This is a complementary gain
crossfade, not an equal-power law; unrelated material can dip in perceived loudness.

## Bounds and lifetime

Each preparation admits 2–3,000,000 finite stereo frames at 8–384 kHz, duration 0.5–2×
and pitch ±12 st. Original, result and pitch intermediate must each meet the frame bound;
the combined portable JSON must fit the existing 64 MiB state cap. Sources shorter than
a 20 ms grain use resampling, so independent pitch is not promised for those tiny buffers.
Disk-backed sources must first be explicitly switched to embedded mode; no unapproved
external path is read by preparation. Long disk stretching is not part of this stage.

One host job is active at a time; the native editor also allows one job. Cancellation is
checked during resampling/grain work; obsolete results are discarded. Closing a dialog
cancels its job. A source replacement/preset cannot be overwritten by a stale native result;
any intervening host document edit invalidates its pending result. Workers own scratch;
prepared output only replaces state after success. Threads end and release their scratch
on success/error/cancel. No prepared cache files or orphan temporary audio files are created.
Native PCM caches share equal buffers via at most 32 weak entries; SampleStore/retirement
queues retain strong ownership until control-thread collection. Host embedded previews
are capped at 32 buffers / 256 MiB, evicting only unused buffers on the control thread;
when active previews pin the entire budget a further preview is unavailable. The existing
nonembedded/disk loader cache has its prior lifetime policy. No DSP preparation, locks,
file access or large frees were added to process/render.

## Evidence and remaining checks

Automated coverage is in DSP stretch tests, sequencer preparation state tests, host dialog
worker tests, native allocation/shared-buffer tests, and hosted clip crossfade/chase tests.
Local results and screenshots are recorded in [progress](../development/progress.md).
Listening on drums, vocals, bass and stereo/polyphonic music, FL A/B, standalone sampler
recall in another host, physical realtime output and Windows/Linux GUI/audio runs remain
in [human verification](../development/human-verification.md). These checks require listening
and the relevant host/machine; headless rendering does not substitute for them.

## UI consistency review

The host uses the same centered fixed dialog helper, strong title, two-column grid,
small secondary text, numeric units and primary action styling as Export audio. Host and
native entry points say Time stretching; mode choices are Resample / Stretch. The native
controls live in a bordered sampler panel with its existing flat buttons and identity
accent. Keep on disk sits beside source information. Restore original is disabled until
an original has been retained, and disk/missing sources show actionable preparation guidance.
Clip dialogs choose a valid multiplier default when the current clip length would exceed
bounds. DSP names and frame-limit details live in tooltips/docs, while duration and scope
remain visible. Both default-size PNGs were rendered and visually checked after this review.

The main-branch ping-pong/filter and Rev 1 arrangement work was integrated before delivery.
Preparation retains the appended sampler controls and loop-gesture UI. Private channel reuse
checks all arrangements; automatic fade cleanup also restores inactive arrangements' manual
bases before arrangement edits. Legacy right-edge Stretch remains shared-channel resampling;
use Time stretching for the independent fixed-duration workflow described here.
