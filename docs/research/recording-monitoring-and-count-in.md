# Input monitoring and recording count-in

2026-10-27. Clean-room implementation from public Image-Line documentation. No FL binaries,
source, samples or graphics were used. Manual behavior is OBSERVED documentation evidence;
FL runtime defaults, exact bar choices and interaction edge cases remain INFERRED pending a live comparison.

## References

* [Audio recording](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/recording_audio.htm):
  monitoring Off / When armed / On; input passes through a Mixer track; capture pickup can differ
  from the monitoring path. Software monitoring adds device and processing latency.
* [Mixer](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/mixer.htm):
  FL supports separate hardware input/output assignments per track.
* [Toolbar](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/toolbar_panels.htm):
  count-in supplies metronome ticks before recording, Ctrl+P (Cmd+P on macOS); right-click selects bars.

## LimeShot workflow

REC-003 builds on the existing one-input recorder. Audio settings selects the input device,
monitoring mode and an explicit Mixer destination by position; the Record menu also changes
monitoring mode. When armed means the main Record switch and Audio filter are both enabled,
including while stopped. On also works while stopped and unarmed. Monitoring defaults Off
for each session/project; it is not saved in preferences or the project. Destination defaults
to insert 1 and follows its Mixer position when tracks are reordered. An unavailable destination
is silent until another track is selected. Per-track hardware I/O remains MIX-016.

A single cpal input stream serves monitoring and a raw stereo float-WAV recorder. Starting or
ending a take while monitoring reopens that stream, so the transition can briefly interrupt
monitoring. Turning monitoring off during a take does not interrupt its file writer. The writer
records raw input independently of monitor effects, fader and mute. Disarming, removing the Audio
filter, pausing and stopping finalize a take. Input/output configuration changes finalize it too.
Input stream errors report a visible error; retries require a configuration change or Retry input.
The live-check harness can use inputs independently of its disabled preference persistence.

The input callback duplicates mono, takes the first pair from multichannel input, replaces
non-finite samples with silence and publishes complete stereo frames through a 100 ms bounded
SPSC ring. Output performs linear rate conversion into preallocated scratch space, then injects
input before the destination's inserts/EQ/fader/routes. Missing input is silence. Overflow drops
frames; the next reader discards excess backlog to recover low latency. Routing/mute changes
invalidate queued frames. This is basic conversion, without a band-limited resampler or adaptive
clock-drift correction; extreme clock mismatch can click or underflow. Replaced receivers return
through the existing garbage ring for destruction on the control thread. Offline rendering never
receives live input. No allocation, locks or I/O are added to either audio callback.

REC-007 offers a toolbar 3.2.1 switch, Options > Recording precount, and Cmd/Ctrl+P. Right-click
chooses one or two bars (LimeShot choices; exact FL choices not observed). It applies only when
starting playback with Record armed. An engine-owned timeline schedules accented clicks at the
current tempo and signature, regardless of the normal metronome switch. The playhead stays at
the requested position until the lead-in ends; the renderer can split a block and start the
program on that exact sample. Loop boundaries do not repeat it. Stop/pause/locate cancel the
lead-in using the existing 5 ms declick. Changing the option or disarming during a running lead-in
does not cancel playback; Stop cancels it. Tempo/signature changes during the lead-in take effect
for the subsequent program; the lead-in retains its starting tempo/signature.

MIDI/typing notes played during the lead-in are auditioned but not recorded. Audio capture is
prepared during the lead-in and gated by an atomic output-transport flag, so the countdown is
excluded from the WAV while monitoring continues. Independent input/output callback clocks and
UI-driven input startup still impose callback/startup uncertainty. This is not measured input
latency compensation (REC-006), sample-synchronized duplex capture or punch recording. In
particular, do not claim sample-exact physical input alignment from the sample-exact playback test.

## Automated evidence

* `crates/limeshot-audio/src/monitor.rs`: conversion continuity at four block sizes and three
  rate pairs; mute/reroute, overflow and underflow do not replay stale input.
* `record::tests::count_in_gate_only_blocks_recording_and_sanitizes_input`: the writer excludes
  gated samples while monitoring continues; mono/multichannel conversion, non-finite input and
  whole-frame overflow are covered.
* `live_input_runs_through_mixer_effects_fader_pan_routes_and_mute`: stopped input, insert
  processing, gains/pan/mute, invalid destination and absence from ordinary renders, four blocks.
* `count_in_is_sample_exact_across_blocks_tempos_and_time_signatures`: full post-lead-in audio
  equals ordinary playback at the exact boundary (1/64/333/1024 frames, fractional BPM, 4/4 and 7/8).
* `stopping_pausing_and_locating_cancel_count_in`: cancellation and preserved seek position.
* UI tests `audio_settings_monitoring_is_session_local_and_uses_explicit_track` and
  `recording_precount_shortcut_menu_bars_and_note_filter_work`: actual settings/menu/shortcut/bar
  controls, no project edits, project reset, and exclusion of lead-in MIDI notes.

Live and platform results belong in progress.md and platform-validation.md; features remain
IMPLEMENTED until their remaining physical-device and platform checks have evidence.
