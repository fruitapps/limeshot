# Human verification needed

Current as of 2026-10-27, after v0.1.0-alpha.1. This queue records hands-on checks, unavailable prerequisites and
owner decisions. Agents should perform checks they can complete with available tools and granted permissions;
when human input is required, record it here and continue the independent development work in
[Next steps for agents](next-steps.md#development).

Record the platform, architecture, build/commit, hardware or plugin versions, prerequisites, steps and result
in [progress.md](progress.md) and, for platform checks, [platform-validation.md](platform-validation.md).
Link the evidence from the relevant [feature record](../roadmap/features.json) and update
[known limitations](../roadmap/known-limitations.md). Partial results do not verify the whole feature.

## Verification

1. **Native plugin embedding (PLG-006).** Use `live_check` with a disposable project and Screen Recording and
   Accessibility granted. Test Vital and a resizable editor such as Serum: keyboard forwarding, OS drops onto
   native views, pinned and covering windows, host-edge resizing and reopening saved wrappers. Partial results:
   [live-plugin-checks-2026-10-25.md](live-plugin-checks-2026-10-25.md). Do not mark VERIFIED from a build or a
   headless screenshot. Requires installed plugins and platform permissions.
2. **Platforms (PF-002…PF-005).** Install the alpha packages on real Windows, Linux and Intel macOS machines: GUI,
   audio device, plugin editors (Windows), parameter fallback (Linux), drops and the installers' uninstall paths.
   Requires access to those machines and audio devices; use disposable test projects.
3. **Real-plugin pass.** Find plugins…, factory and `.vstpreset` presets, parameter automation, delay compensation
   and the process bridge with commercial plugins. Requires the relevant installed/licensed plugins.
4. **Hands-on interaction regression.** Note drawing, sampler previews and drags, waveform drops, rack
   multiselection and scrolling, Mixer selection and insert reordering, mute locks, window raising and the typing
   keyboard. Requires a live GUI session and the relevant input/observation permissions.
5. **Recording and MIDI.** Complete audible input-monitoring checks (Off / When armed / On, effects/fader/mute,
   idle/record/stop transitions), measured count-in input alignment, long-run clock drift and device loss;
   recording with a hardware MIDI keyboard and opening exported MIDI in another application. A partial macOS
   physical-input take/count-in pass is recorded in [progress](progress.md) (2026-10-27): one finite/nonzero 24 kHz
   take, monitoring into a muted insert and correct Playlist placement. Requires recording hardware, input
   permissions, headphones for audible monitoring, and an independent MIDI reader.
6. **FL Studio behavior observations.** Alt+wheel vertical zoom, in-clip automation point dragging, STEP/SLIDE
   automation options and bindings marked INFERRED. Requires an available FL Studio installation and a live
   observation session. Record clean-room findings in `docs/research/`; do not modify the installed application.
7. **Release signing (PF-006).** Authenticode and Developer ID signing with notarisation require owner-provided
   signing identities/credentials and verification of the resulting installers. See
   [release packaging](release-packaging.md); independent packaging implementation remains agent work.

## Sampler and MP3 follow-up (2026-10-27)

* **Sampler A/B (INS-007, INS-014, INS-017…INS-021, APP-022).** With FL Studio 2026 and
  LimeShot available, use disposable projects and generated samples to compare stretch duration/pitch,
  note-off/envelope/loop behavior, declick and source metadata. In Lime, change root through the native
  keyboard/parameter state and check host Root/audition, undo and reopen; replace/clear samples through
  native and host controls and check waveforms, preview, drag and clip fit-to-length. Test standalone
  Lime Sampler in FL as well. Host/native root, source, preset and undo synchronization (INS-021) is
  implemented and covered by automated tests ([audit](../research/sampler-parity-audit.md)), awaiting
  live GUI confirmation. Computer-use app selection timed out twice in
  this session; permissions were authorized, so their absence has not been established.
* **MP3 export (EXP-005).** Listen to exported percussion, bass and polyphonic music at several
  bitrates; check duration/delay in additional gapless-aware players and on Windows/Linux. Automated
  independent-decode tests and macOS UI inspection do not establish perceptual quality or platform
  behavior. LimeShot reimport retains padding; use WAV when alignment matters. See
  [implementation](../research/mp3-export.md).

## Decisions for the owner

* **Piano Roll range looping.** Pattern mode ignores loop ranges, yet the UI still sends the Piano Roll's range to
  the renderer, where it has no effect. Either loop Pattern playback over the Piano Roll range only, or stop sending
  it and document FL's behaviour. Record the owner's choice before changing that behavior; continue other features
  while awaiting the decision.

## Alpha.1 follow-up (2026-10-26)

Owner-requested interaction and disk-sampler fixes are in `codex/alpha1-polish`; see
[research and design](../research/alpha1-sampler-and-fades.md). The macOS live pass covered native value typing
and Serum internal-corner shrink/grow. Repeat host-edge resizing with other resizable plugins and long disk-backed
playback/seek checks on Windows and Linux before promoting the native feature statuses. Disk cache misses and
smooth fade/crossfade limits remain recorded.


## Alpha.1 Rev 1 interaction pass (2026-10-27)

Use rebuilt Lime Native bundles and `live_check` with a disposable temporary project. Live automation
requires Accessibility/Screen Recording on macOS. Check sustained drawn-note audition across frames,
rapid placements/release outside the Piano Roll, pattern edits selecting Playlist sources, internal
sample/pattern drags at all viewport edges, focus toggles, STEP/SLIDE, waveform right-scrub silence
until release, miniature scrollbar, track selector sweeps, resize/collapse/size lock, groups and
arrangement clone/switch/merge/delete/save/reopen. Listen to resample resize and check linked uses;
it changes pitch and shared sampler state. Compare FL behavior separately. Repeat on Windows/Linux.
See [implementation and limits](../research/alpha1-rev1.md). Native Windows editor sluggishness requires
a Windows host and remains implementation investigation in next steps, not resolved by macOS tests.


## Sampler loops and filters (2026-10-27)

FL 26.1.1 build 5313 Sample and Envelope/Instrument control presence was observed with the owner
opening the pages. Pointer automation still returned `noWindowsAvailable`; screenshots and keyboard
input worked. The filter popup was closed in the captured view; no FL audio parity is claimed.
With rebuilt debug bundles, use a disposable project and generated ramp/tone samples to compare
forward/reverse/ping-pong endpoint turns and fractional endpoints, Shift-drag pair bounds, note-off,
filter outputs and held edits. In Lime check native loop drags, host undo, preset/load/reopen, disk
sources and standalone sampler hosting in FL. Repeat Windows/Linux. Prerequisites: both applications,
GUI observation/input permissions and an audible output/headphones. See
[control inventory](../research/sampler-controls-2026-10-27.md); recording a control's presence does
not verify its sound or complete the sampler feature.

## Prepared sampling and crossfades (2026-10-27)

* On this branch/build, audition generated comparison WAVs with
  `cargo run --release --manifest-path tools/stretch-eval/Cargo.toml -- <temporary-folder>`.
  Listen on headphones to tone, impulse, percussion and polyphonic fixtures at 0.5/1/2×,
  then owner-provided drums, bass, vocals and wide stereo music. Check attacks, tails,
  modulation, low-frequency pitch and stereo image. Numeric results alone do not establish quality.
* In LimeShot, load an embedded sample, Time stretching in Channel Settings, choose
  Stretch and 0.5/1/2× with −12/0/+12 st. Check native controls/recipe, waveform, root audition,
  Restore original, presets, undo/redo, source relocation and save/reopen without the source file.
  Place two clips and notes using that channel; Time stretching from its clip menu and verify only that clip changes.
  Test trim/reverse/held loops, slip offset, beat fit, starting inside a clip/fade, playback and WAV.
* Enable Automatic crossfades on overlaps from an audio clip menu. Move two audio clips on the
  same track into/out of overlap, inspect S fades and listen through the overlap, then undo/reopen.
  Check independent recordings for perceived dips; the implemented law is complementary gain.
  Nested/triple overlaps are intentionally skipped. Compare documented FL behavior separately.
* Repeat native plugin recipe/preset recall standalone in another VST3 host and runtime checks
  on Windows/Linux/Intel macOS. Requires the rebuilt bundles, relevant machines/host, audio hardware
  and listening; local automated/macOS offscreen evidence is not live/platform certification.
  See [design and bounds](../research/prepared-sampling.md).

## PR #25 preview and fade review follow-up

With rebuilt Lime Native bundles and `live_check` using a disposable temporary project, create
more than 32 distinct short embedded samples. Inspect Rack/Playlist/Channel Settings waveforms
and audition older channels after editing/replacing/removing other samples. Check that a full
preview budget leaves existing mapped previews available, without substituting source files.
Enable automatic crossfades for an overlapping audio pair: Smooth S fades is disabled with a
hint, becomes editable when automatic crossfades are turned off, and remains editable for
nonoverlapping clips. Automated UI/cache tests and the rendered menu cover local behavior;
physical GUI/audio and Windows/Linux checks remain open. Requires an audible output and the
platform's GUI input/observation permissions. See [prepared sampling](../research/prepared-sampling.md).

## Custom VST3 search folders (PLG-004)

With a rebuilt app and a disposable project, open Settings › File, add a folder containing a
known installed/licensed VST3 (nested vendor folder and a folder with spaces), and run Find plugins.
Confirm progress, successful instrument/effect entries, load from the Rack/Mixer menus, saved
settings/database after restart, and removal followed by a rescan. A removed folder must leave
already loaded project instances usable. Check a missing/offline root, duplicate/overlapping
roots and a plugin that fails scanning; standard plugins must remain available. Requires the
platform folder picker, GUI input permissions and suitable VST3 bundles. Repeat on Windows/Linux.
Automated discovery/settings/worker/UI checks and an offscreen settings capture do not establish
live picker/rescan or cross-platform runtime behavior.

## Navigable undo history (APP-006)

Using `live_check` with a disposable project and GUI input/observation permissions, make
several note, tempo and hosted-parameter edits, save midway, and select earlier/later rows
in Edit › History. Confirm the selected state, modified indicator, audible plugin state
and sequential Undo/Redo. Make a new edit after jumping backward and check redo disappears.
Repeat on Windows/Linux. Headless core/UI tests and the offscreen menu capture establish
local snapshot behavior, not live audio/plugin or other-platform runtime verification.
