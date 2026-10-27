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
  Lime Sampler in FL as well. Source-confirmed root synchronization gap and unverified source metadata
  paths: [audit](../research/sampler-parity-audit.md). Computer-use app selection timed out twice in
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
