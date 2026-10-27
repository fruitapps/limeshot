# Lime Sampler parity and integration audit

2026-10-27. Scope: Lime Sampler as hosted by LimeShot, its standalone VST3, and the FL Studio
Channel Sampler / Audio Clip workflow. This is a source audit and public-documentation review;
it is not a claim of listening parity or identical DSP. No FL code or assets were used.

## Evidence and confidence

* **VERIFIED (source)**: the Lime behavior below follows the shared control specification,
  native processor/editor, host integration and import code listed below. Existing tests are
  identified separately; reading code does not verify a live workflow.
* **Documented FL reference**: Image-Line's [Sampler](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_sampler.htm),
  [instrument/envelopes](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_ins.htm)
  and [miscellaneous channel settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_misc.htm)
  manuals, accessed 2026-10-27. These are documentation evidence, not fresh hands-on observations.
* **UNKNOWN live**: attempts to select the installed FL Studio 2026 through computer-use automation
  timed out twice. Accessibility and Screen Recording were authorized by the owner; the timeout
  does not establish that permissions are missing. No FL project was edited by these attempts.
  A live A/B check remains in the human verification queue.

Source anchors: `crates/limeshot-core/src/{lime,command,model}.rs`,
`crates/limeshot-dsp/src/{voice,sample,disk}.rs`, `crates/limeshot-ui/src/{lime,inspector,plugins,browser,playlist}.rs`,
`crates/limeshot-project/src/flp.rs` and `native-plugins/shared/src/{parameters,processor,sample,editor/layouts,editor/displays}.rs`.

## Feature comparison

| Area | Documented FL workflow | Lime implementation and missing parity |
| --- | --- | --- |
| Source and waveform | Load, locate, remove, sample history; draggable sample/loop handles | Load/Clear and waveform preview/drag exist; host also offers sample assignment. No equivalent source history/locate workflow. Native trim handles and numeric loop endpoints exist; joint loop-handle movement is not implemented. |
| Content | Disk mode, resampling, region/slice metadata | Embedded short samples or worker-prepared disk cache. No metadata-driven root, loop, tempo, region or slice import. Linear interpolation; no user-selectable quality. |
| Loops | Held loops and ping-pong | Held forward/reverse looping with fractional endpoints. No ping-pong or crossfaded loop seam. Reverse is playback direction, not ping-pong. |
| Declick | Several transient/bleeding fade modes | ADSR release and Playlist manual fades exist, without equivalent declick modes or source audio bleeding beyond a cut. Release is not proof of FL's fade shape. |
| Stretch | Separate pitch, multiplier, musical duration, mode; tempo locking | Pitch changes playback rate and duration together. No stretch mode, duration ratio, beat/bar fitting, source BPM detection/lock or formant parameter. Playlist resizing trims/repeats the source; it does not stretch it. |
| Precomputed processing | Sample processing controls and crossfade-loop preparation | Gain, trim and reverse are playback settings; no equivalent precomputed normalize, DC removal, stereo processing or crossfade-loop preparation. Mixer effects do not replace persistent sample processing. |
| Volume shaping | Envelope delay/hold, tensions, amount, tempo options | Fixed ADSR controls, milliseconds, linear stages. No delay, hold, envelope enable/amount/tension or beat synchronization. One-shot bypasses normal note-off unless looping; it is not a complete envelope-enable equivalent. |
| Modulation/filter | Four envelopes, five LFO destinations; selectable filters | One low-pass cutoff/resonance pair, no sampler LFOs, pitch/filter envelopes, filter choice or velocity/key-to-filter tracking. LimeSynth's modulation controls do not give these capabilities to Sampler. Pan has LFO support in FL, not a pan envelope. |
| Voice/channel tools | Polyphony/mono/porta, cut groups, timing, tracking, event arp/echo | Fixed 128-voice pool with oldest stealing and same-key retrigger handling. No configurable sampler polyphony, mono/porta, cross-channel cut groups, per-channel swing mix or realtime event arp/echo. Piano Roll's destructive arpeggiator and Mixer Delay are different workflows. |
| Note expression | Native note properties/slide integration | Velocity and VST3 per-note pan are supported. Release velocity, fine pitch, Mod X/Y and FL slide/porta-note semantics are absent from the note path. MIDI pitch bend in the standalone processor does not establish live MIDI bend forwarding by LimeShot. |

Roadmap: INS-007, INS-011, INS-014, INS-017…INS-024, PR-016, PR-017, PL-023, PLG-025.
The hosted P0 sampler records describe migration of LimeShot's former sampler fields, not complete FL parity.

## Integration findings

1. **Two root-key representations (source-confirmed; live impact inferred).**
   `SetChannelRootKey` writes both `Channel.root_key` and Sampler parameter 7. Generic `SetPluginState`
   writes the plugin state only. Thus a root change through the plugin's control/state can leave the
   Channel Settings Root display and host preview key at the previous value. Saved plugin pitch may
   remain correct while the host audition uses a different key. Unify the canonical value or synchronize
   both paths with undo/redo, preset/state load and automation before calling this integrated parity.
2. **Two source representations (requires live verification).** Host actions maintain plugin extras
   such as `lime-sample-path`; native Load/Clear and Keep on disk update `sample-v1` through plugin
   state. Audit source lookup precedence, drag payloads, fit-to-length and preview duration after native
   replacement/clear, save/reopen and undo. Do not assume that a waveform displaying correctly proves
   that host source metadata is current. Root cause and failures beyond the root case are unverified.
   Native editor notifications also only mark the document dirty (`PluginHost::poll`);
   `store_states` captures state on save without a document undo step. Native gestures therefore
   do not have the same undo integration as host parameter-list commands. Test unrelated host
   edits, presets and source changes before assuming the two editing surfaces remain synchronized.
3. **Sampler and Audio Clip share a channel.** Both use the same Lime Sampler instance/state and mixer
   route. Reusing the source can therefore couple settings between note playback and Playlist clips.
   FL's Audio Clip is a distinct generator. Stretch must explicitly separate source settings, channel
   settings and per-clip duration/chase; a single global playback-rate control would be insufficient.
4. **File and state limits differ.** Embedded state supports six million stereo frames and 8–384 kHz;
   disk preparation accepts up to 250 million stereo frames (2,000,000,000 decoded bytes). The disk mode
   prepares WAV caches even for supported compressed input, rather than promising direct compressed
   streaming. It needs the source on reopening, project-relative path handling and explicit approval
   for external sources. Realtime cache misses silence/count underruns; offline mode waits for pages.
   Existing disk/embedded parity tests do not prove long-run behavior or other-DAW integration.
5. **FLP conversion is partial.** `flp.rs` restores source/name, level, pan and mixer destination, with
   default Sampler settings and root key 60. Audio channels become gated samplers. It does not recover
   sampler pitch/root, envelope, filter, loop, stretch, declick or precomputed settings. Successfully
   importing notes/sample paths does not establish sound parity; advanced settings may default without
   a specific per-setting warning. Record this under APP-022 as well as the sampler records.
6. **Preview and expression assumptions.** Host waveform preview estimates duration from trim and pitch;
   future stretch must replace that formula and update clip fit/chase, waveform position and export.
   The standalone native sampler handles sustain/pitch bend, but the host currently forwards live MIDI
   notes only. FL-specific native expression must be independently mapped through bounded VST3 events.

Existing evidence: `native-plugins/VALIDATION.md`, `docs/research/alpha1-sampler-and-fades.md`,
`crates/limeshot-ui/tests/lime_vst3.rs` (clip/chase/fade and disk tests), and
`native-plugins/shared/src/processor.rs` (held edits, disk equivalence, state and replacement tests).
None establishes full sampler parity.

## Time-stretch implementation scope

**DESIGN PROPOSAL, not implemented.** Start with fixed-duration offline preparation, then treat
realtime tempo automation as a separate milestone. Preserve existing parameter IDs/defaults and
version the new state/protocol. Existing songs must retain their current resampling behavior.

An independent stereo-linked WSOLA prototype is a practical first comparison. The original
[Roelands/Verhelst paper](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html)
describes time-domain waveform matching and overlap-add; it supports investigating this approach,
not a claim of acceptable drum-loop or polyphonic quality. Compare it against a phase-vocoder
approach with transient handling on synthetic tones, impulses, percussion and owner-supplied music.

The pure-Rust [wsola API](https://docs.rs/wsola/latest/wsola/index.html) is an evaluation candidate;
inspect pinned source, license, allocations, flushing and reset semantics before adopting it. The
MIT-licensed [Signalsmith Stretch](https://signalsmith-audio.com/code/stretch/) is another quality
reference, but its C++ implementation/Rust FFI wrapper does not satisfy the first-party Rust DSP
constraint as an implementation shortcut. No stretch dependency has been added.

Proposed first user workflow: select a source, choose Resample or a clearly named independent
Stretch mode, set target beats/bars or duration multiplier, keep pitch independent, and prepare
an immutable cache on a worker. Retain original audio/state and reject invalid/oversized output
before replacement. Share prepared output between voices where possible; prevent every note
from allocating or opening files. Declare any pitch-range or quality limitations in the UI/docs.

Before integration: define original-frame ↔ stretched-frame mapping for trim, reverse, loops,
clip offset/chase and fades; update host duration, preview and fit-to-length; preserve sampler/audio
clip distinctions; include plugin-state/preset recall and undo. Realtime mode additionally needs
bounded per-voice buffers, latency reporting/PDC, disk lookahead, tempo-event handling and identical
offline scheduling. Do not label fixed offline stretching as realtime Stretch parity.

Acceptance gate: duration/pitch independence at 0.5×/1×/2×, stereo coherence, attacks and loop seams,
silent/tiny/invalid sources, extreme-ratio rejection, state migration/save/reopen/undo, seeks/reverse,
all supported rates and blocks 1/64/333/1024, plus processing allocation assertions. Listening and
live A/B checks remain necessary; numeric tone tests alone are insufficient.
