# Audio engine

This document describes the code as it is. Source: `crates/limeshot-engine`, `crates/limeshot-audio`, `crates/limeshot-sequencer`.

## Hosted instruments and inserts (D39)

`InstrumentState` is Plugin or Silent: every instrument and insert is a hosted VST3 (normally a Lime Native bundle),
and the engine has no effect factory or in-process track EQ. Projects from before the switch are converted by the
document normalizer and the CLI before playback or saving. `limeshot-dsp::voice` is shared native-plugin DSP, also
used for Browser previews and the metronome. `tests/support` holds the reference adapters used by engine and
equivalence tests; they are never linked into the application. The native plugins do not depend on the engine or
sequencer.

## Data flow

```
Project (UI thread) ──compile()──▶ Arc<Snapshot> ──rtrb ring──▶ Renderer (audio thread) ──▶ device
       ▲                                                          │
       └────────── commands ◀── UI                old snapshots ◀─┘ (returned, dropped on the UI thread)
```

1. The UI edits the `Project` only through commands (`limeshot-core`).
2. When the document revision, the current pattern or a loaded sample changes, `LimeShotApp::sync_engine` calls
   `limeshot_sequencer::compile`. That produces an immutable `Snapshot`: resolved channel states (instrument, linear gains,
   `audible` after mute/solo), mixer track states, and two `Program`s (tick-sorted note events plus a loop length):
   * `pattern`: the current pattern, for Pattern mode,
   * `song`: every unmuted clip on an unmuted track, flattened to absolute ticks, for Song mode.
3. The snapshot goes to the audio thread as `Msg::Snapshot(Arc<Snapshot>)` through a bounded lock-free SPSC ring
   (`rtrb`, 1024 slots). The renderer swaps it in and pushes the old one onto a 64-slot return ring. The UI drains
   that ring every frame (`Player::collect_garbage`), so large frees never happen on the audio thread.

There is exactly one note representation: the Piano Roll and Channel Rack edit `Pattern::notes`, and the
sequencer compiles from it.

## Threads and ownership

| Thread | Owns | Talks to others through |
|---|---|---|
| UI / control | `Document`, `Player`, `SampleCache`, UI state | sends `Msg` (rtrb); reads `Status` atomics |
| Audio (cpal callback) | `Renderer`, its voice pool and buses | pops `Msg`; pushes old snapshots; stores atomics |
| Sample loader | nothing persistent | `std::sync::mpsc` job/result channels (never touched by the audio thread) |
| Export worker (on demand) | a cloned `Arc<Snapshot>` and its own `Renderer` | `mpsc` result channel |
| Silent clock (no device) | a `Renderer` | the same rings and atomics as the device callback |

`Status` holds `AtomicU64` position, `AtomicBool` playing/song mode, `AtomicU32` voices and load, and per-track peak
meters as `f32` bits in `AtomicU32`s. All accesses use `Relaxed`: each value is an independent indicator.

## Real-time rules (enforced by construction)

Inside `Renderer::render` and the cpal callback:

* **No allocation in steady state.** The voice pool (128), mixer buses (128 × 2 × 1024 f32), action list (512) and
  device scratch buffer are allocated once in `Renderer::new` and `Player::start`. `Vec`s are moved out and back
  (`mem::take`), never grown.
* **No locks, no I/O, no logging.** Messages are popped with `try`-style `rtrb` calls; at most 256 per callback.
* **No large frees.** Replaced snapshots are returned to the UI. Voices hold `Arc<SampleData>` clones, but the
  `SampleCache` keeps every decoded sample alive for the life of the app, so a voice ending only decrements a counter.
* **No panics.** Crate roots deny `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`. Slice access in the hot loop
  is bounded by construction, and non-finite output samples are replaced with 0.

Known gap: the cpal error callback uses `log::error!`, which may lock inside `env_logger`. It is only called on stream
errors, not per buffer.

## Timing

* Musical time is integer **ticks at 960 PPQ** (`limeshot_core::time`). A step (16th note) is 240 ticks.
* Tempo is stored as milli-BPM (`Tempo(u32)`, 10 000–522 000).
* `Timeline { origin_tick, origin_sample, tempo, sample_rate }` maps ticks to samples exactly:
  `sample(t) = origin_sample + ceil((t − origin_tick) · 60 000 · sr / (milli_bpm · 960))`, in `u128`.
  A note sounds at the **first sample at or after** its exact time. The inverse (`first_tick_at_or_after`) is exact too,
  so the set of events in a block is decided by absolute sample positions only. That is why onsets do not depend on the
  block size (test: `block_size_does_not_change_the_output`, blocks of 1, 64, 77, 512, 1000 and 1024 frames give the
  same output).
* A tempo change while playing re-anchors the timeline at the current musical position. Scheduled note-offs are
  recomputed from their stored end ticks (`tempo_change_while_playing_keeps_position`).
* While stopped, the timeline origin follows the sample clock, so the playhead does not move.

## Scheduling

`Renderer::render(out)` processes up to `MAX_BLOCK` (1024) frames:

1. Clear the buses of the tracks in use.
2. While frames remain: if playing, `schedule()` finds the end of the current pass (`sample_of_tick(loop_length)`),
   limits the chunk to it, and collects the note-ons whose ticks fall in `[first_tick(s0), first_tick(chunk_end))`, plus
   metronome clicks on beat ticks. Each gets a frame offset `sample_of_tick(tick) − s0`.
3. Preview/click voices render between action offsets; instrument actions queue MIDI events for hosted processors.
4. At the end of a pass: Pattern mode (and Song mode with loop on) re-anchors the timeline at tick 0 and releases
   gated notes. One-shot samples keep ringing across the loop. Song mode with loop off stops and sets `finished`.
   With a loop range (`Msg::LoopRange`, the Playlist's time selection) Song mode ends each pass at the range end and
   re-anchors at the range start with a chase, looping even when song loop is off; locating past the end lands at
   the range start. Pattern mode ignores the loop range (`Renderer::active_loop`): it always loops the whole
   pattern. The UI still sends the Piano Roll's time selection in Pattern mode, but it currently has no audible
   effect (test: `pattern_ignores_time_selection_at_any_block_size`).
5. Note-offs: each sequenced voice stores `release_at` (an absolute sample) and is released at that exact frame inside
   the voice loop.

An empty song still runs one bar so the transport behaves.

**Audio clips** are note events with `clip = true` and an `offset_secs`: the voice starts that far into the sample
(`Voice::skip`) and is released at the clip end even on a one-shot sampler. After `play`, `locate` or a mode change the
renderer *chases* audio clips: the first `schedule` also starts every clip event that began before the position and
is still running, part-way through. Notes are not chased (as in FL Studio).

**Automation** (Song mode only): the snapshot holds one `AutomationLane` per automation, its clips flattened to
song-time points (a clip's end point holds its value until the next clip). Before every chunk — at most 64 frames while
lanes exist — `apply_controls` loads channel volume/pan and mixer gain/pan from the snapshot into preallocated arrays,
then overrides them with the lanes' values at the current tick; effect parameters go through `FxRack::set_param`,
which keeps each processor's parameter array so one value can change. Hosted plugin parameter lanes send parameter changes to the instrument/effect processor; plugin channel volume/pan
uses preallocated per-frame gains captured every automation interval. Evaluation is a binary search, allocation-free.

## Voices

`Voice` (`limeshot-dsp/src/voice.rs`) is shared DSP for native plugins and host previews/clicks, a source plus an `Envelope`:

* **Sampler**: linear-interpolated playback of interleaved stereo `SampleData`;
  `step = src_rate / out_rate · 2^((key − root + pitch)/12)`; region `[start, end)` as fractions; reverse plays from
  the end. Envelope: configurable attack, decay, sustain and release; defaults preserve the exact sample onset
  (0 ms attack/decay, full sustain, 10 ms release). One-shot samplers ignore
  the note length unless looping is enabled. Loop points repeat while held, and an optional low-pass filter
  processes the sampler output.
* **LimeSynth**: one or two PolyBLEP oscillators (sine/square/saw/triangle; the second detuned by `detune_cents`), octave
  offset, ADSR. Square and saw are trimmed (×0.5 / ×0.6) so the shapes sound about equally loud. An optional
  `ModState` (inline, no allocation) adds a TPT state-variable low-/band-/high-pass filter with key tracking, an exponentially decaying filter envelope
  and a sine LFO to pitch and cutoff, recomputed every 16 samples; it is absent at default settings.
* **Click**: metronome blip (1760 Hz downbeat, 1320 Hz other beats, 30 ms decay).
* **Preview**: a browser sample played once into the master.

Stealing: when all 128 voices are busy, the oldest (lowest serial) is overwritten.

## Mixing

Per voice: `sample × envelope × velocity × instrument gain × channel volume × pan_gains(channel pan)`, added into the bus
of `channel.mixer_track` (0 = master). Muted or non-soloed channels keep running silently, so unmuting is seamless;
previews always sound.

Per insert, in `snapshot.mix_order` (a topological order from `Mixer::processing_order`: every track before the
tracks it routes to): insert effects, then `× volume × pan_gains(pan)` when audible, peak-metered, then added into each
destination of its routes at the route's level (pre- or post-fader per route; the master by default, other inserts as
buses, several at once for parallel processing). Routing cycles cannot exist: `SetRoute` refuses them and `Project::sanitize` drops them
from files. Master:
`× volume × pan_gains(pan)`, peak-metered, written to the output. A master peak above 1.0 sets the clip flag.

Pan law (`limeshot_dsp::pan_gains`): balance. `L = min(1, 1 − p)`, `R = min(1, 1 + p)`. Centre is unity, the far side
fades linearly, and panning never boosts.

Solo: if any channel is soloed, soloed channels and unmuted channels with a mute lock are audible. If any insert is soloed, soloed inserts and unmuted inserts with a mute lock are
audible. The master is never solo-muted.

### Insert effects

Each mixer track (master included) has an insert chain of up to 10 effects (`limeshot_dsp::effects`: Utility, Lime EQ,
Lime Filter, Lime Delay, Lime Room, Lime Comp, Lime Limiter, Lime Gate, Lime Drive and Lime Chorus). In `mix()`, each bus runs its chain after the voices are
summed and before volume/pan. Each enabled slot blends its output with its input by the slot's host `mix` (1.0 = fully
wet). For processors that report latency, `FxRack` delays the dry signal through a preallocated ring (up to 64
such slots), so the blend stays aligned. The dry history is kept even at 100% wet, so turning Mix down never plays
stale audio.

Processors own delay lines, so they must be allocated off the audio thread:

1. `PluginHost::sync` (control thread) loads each hosted VST3 at the device rate and sends
   `Msg::Effect(id, Box<dyn Effect>)` before the snapshot that refers to it. Native plugins construct their shared
   DSP internally; the engine has no production effect factory.
2. The renderer's `FxRack` adopts it into a preallocated pool. When a snapshot arrives, `FxRack::arrange` moves
   processors into per-track chains by id (vectors are preallocated: 128 tracks × (10 inserts + dedicated track EQ)), applies the snapshot's
   parameters, and parks anything unreferenced in `retired`.
3. The audio callback returns retired processors through the garbage ring (`Garbage::Effect`), and the control thread
   drops them.

A snapshot effect whose processor has not arrived is bypassed. When the project is replaced (new/open; ids may be
reused), `Msg::ClearEffects` retires every processor first. Restarting the stream rebuilds all of them at the new rate.
Offline export obtains fresh hosted processors from `PluginHost::offline` and keeps rendering the tail while the output is above
−80 dBFS (`Renderer::is_ringing`), up to `max_tail_secs`.

Routes read a pre-fader bus: post-fader routes apply track gain and pan, while pre-fader routes apply only send
level. A muted track sends nothing in either mode. Meters and compressor sidechain keys read the post-fader bus.
Lime Comp can take another track as its sidechain source; track EQ follows the insert chain.

`pdc.rs` plans delay compensation from plugin instrument and insert latency, aligning routed paths with
preallocated delay rings. Capacity is bounded (about 340 ms per path, 64 delayed paths); exports trim the compensated
latency. Lime Delay resolves tempo-synced timing when the snapshot is compiled.

### Plugin instruments

A channel whose instrument is a hosted plugin (`InstrumentState::Plugin`) has no voices. When the scheduler starts
one of its notes, `InstrumentRack` (`plugins.rs`) queues a note-on at the note's frame offset in the block and keeps
the note as held until its note-off sample; note-offs are queued in the block they fall in, so onsets and releases
are sample-accurate whatever the block size. Each plugin channel then renders the whole block once (after the
voices, before mixing) and is added to its mixer bus at the channel's volume and pan. Stop, locate and the loop point
release held notes at the start of the next block; a retriggered key ends where it is played again. Instruments are
`limeshot_dsp::Instrument` trait objects built on the control thread (the plugin host in `limeshot-ui`), handed over
with `Msg::Instrument`, placed by channel id on each snapshot like effects, and returned as garbage so plugins are
torn down on the thread that made them. Plugin effects are ordinary insert effects (`Vst3Processor` implements
`Effect`); offline renders take their own instances through `Processors`.

## Device output

`limeshot_audio::Player::start(config, snapshot)`:

* opens the configured device (by name) or the default output, the requested sample rate if a supported config range
  contains it, and a fixed buffer size if within the device's range. Otherwise it uses the device defaults;
* supports f32, i16, i32 and u16 device formats. Stereo goes to channels 0/1, extra channels get silence, and a mono
  device gets (L+R)/2. Values are clamped to [−1, 1] before conversion;
* measures callback time ÷ buffer duration as "load";
* falls back to a **silent clock** thread (10 ms blocks at 48 kHz) if no device opens, so the UI still works.

Changing settings restarts the stream with the last snapshot and transport flags.

## Offline export

`limeshot_engine::render_offline` builds a fresh `Renderer` with the snapshot. It sets Song (or Pattern) mode, turns
looping off and renders exactly `sample_of_tick(length)` frames, then the tail until every voice ends (at most 4 s).
`export_wav` writes 16/24-bit PCM (clipped, no dither) or 32-bit float. `export_audio_with` also
encodes stereo MPEG-1 MP3 through pinned pure-Rust `rusty_mp3` (32/44.1/48 kHz, CBR 32–320 kbps),
with final delay/padding tags and atomic destination replacement. Both share the same PCM render;
encoding runs off the audio thread, and the UI runs export on a worker thread. See [MP3 export](../research/mp3-export.md). The same
renderer as playback means the export tests also check playback scheduling.

## Performance notes

Measured on an Apple M-series laptop (debug and release builds):

* realtime demo playback (5 voices, 512-frame buffers at 48 kHz): about 1% callback load;
* offline render of the 15.7 s demo (debug build): well under 0.1 s in the test suite.

No profiling-driven optimisation has been done yet. Voices are rendered sample by sample with a per-frame release
check.


## Editor auditions

Held piano keys and scrub notes use `Msg::NoteOn` / `NoteOff`. Drawn or moved notes and sampler waveform
previews use `Msg::NoteFor`: the renderer computes an absolute off sample from the duration when it receives
the message. That deadline runs while transport is stopped, for preview/metronome voices and hosted instruments alike;
UI mouse release does not cut it short. Durations do not allocate audio-thread objects or depend on UI frame rate.

## Host protocol for Lime plugins

Clip windows/chase/fades and additive channel pan use bounded versioned SysEx; per-note pan uses standard VST3
expressions. Native processors report block telemetry through output SysEx, copied during addEvent. The host supplies
stereo auxiliary key audio to routed Comp instances. All buffers are preallocated. The optional bridge serializes
the same data but retains its existing blocking-I/O limitations. Track EQ is a dedicated hosted Lime EQ after ten
inserts; its saved bands drive appended hidden native controls. See D39.

## Transport, sample hand-off and retiming

Insert processors receive the same block-start transport as hosted instruments, captured after the first tempo
automation update. Tempo-synced delays therefore follow project tempo and Song tempo automation at plugin block
boundaries; changes inside a block reach the next block. Native sample changes use bounded lock-free incoming and
retirement queues. The processor clears voices while retaining the old sample, then returns that reference for
control-thread destruction. A full retirement queue defers adoption; incoming state changes coalesce old pending
updates on the control thread. Neither persistence locks nor allocation-checker exemptions occur in this handoff.


Tempo retiming retains fractional tick phase as an integer numerator over 60,000 × sample rate, plus the first
unprocessed event tick at the new origin. This preserves rounded onsets due at the origin without replaying passed
ticks. A 3,840-note ramp regression checks note order/count and integrated duration at 1/64/333/1024 frames. Native
sample persistence ignores unchanged name/rate/PCM before decoding or handoff, so control-only state recall does
not interrupt held sampler notes. Mono auxiliary keys receive the average of stereo input, with absent frames zeroed.


## Hosted module lifetime (D41)

Successfully loaded VST3 modules and their factories stay in a process-lifetime strong cache. Closing the last
instance releases its controller, processor, sample state and DSP buffers, while the module code remains resident.
Plugin frameworks can install global callbacks, thread-local destructors and native classes that outlive instances;
a weak cache can also race last-client teardown against a new load. Module entry runs once per cached path.
The tradeoff is retaining the code/factory for each visited bundle until LimeShot exits; updates require restarting.


## Hosted parameter commits

Host sliders queue the committed/reset value as well as drag auditions. A no-op document state edit can skip
host synchronization, and a previously queued audition must not overwrite the final value on the next audio block.
State serialization still occurs once at release; the existing bounded parameter ring carries the final update.


## Alpha.1 disk sampler and performance follow-up (2026-10-26)

Long sampler sources use a bounded worker-owned WAV cache, atomic page reads and prepared extrema;
short sources retain embedded PCM. Realtime cache misses return silence and count underruns, while actual
VST3 offline mode waits for pages. Disk sources retain relative project paths and require source approval
on project opening. The decoded stereo float cap is 2,000,000,000 bytes. See the complete
[design and research](../research/alpha1-sampler-and-fades.md), including memory bounds and seek limits.
Native instruments process only their occupied voice prefix while retaining the full 128-voice pool.
Fade counters are f64 and the bounded clip protocol carries curve tensions. Audio callback load remains
render time / deadline; it does not measure total application CPU.

## Live input monitoring and count-in (REC-003, REC-007)

One cpal input stream feeds the raw-WAV writer and an optional bounded SPSC monitor ring.
`AudioState` converts the monitor rate into a preallocated input block and calls
`Renderer::render_with_input`: input enters its explicit Mixer destination before inserts/EQ,
fader and routing. Offline render uses `render` without input. Muting/rerouting invalidates queued
frames; underruns are silent and overflow recovery discards backlog. Old receivers return through
`Garbage::Monitor`; message draining waits when the garbage ring is full, rather than freeing a
replaced ring/snapshot in the callback. Monitoring uses session state and starts Off.

`play_with_count_in` uses a separate exact Timeline for one/two bars of accented clicks. The
program's timeline origin moves forward by each lead-in chunk so its musical position is fixed.
A block can split at the lead-in end and schedule notes immediately on that sample. Stop, pause
and locate cancel the lead-in. The audio status publishes remaining beats and a capture gate;
the input callback records only once the gate is open, while monitoring continues independently.
This flag bridges independent callbacks; physical input alignment is not sample synchronized or
latency compensated. See [design and limits](../research/recording-monitoring-and-count-in.md).

## Fixed-duration prepared samples

Lime Sampler's optional `sample-v1.preparation-v1` retains original embedded audio and a
preparation recipe; the outer PCM is the immutable playable result. Host/native workers run
bounded stereo-linked WSOLA and sinc resampling before changing state. Channel preparation
changes all uses; per-clip preparation clones the sampler channel in a document command.
The renderer still schedules ordinary clip events against prepared PCM, preserving trim/loop
fractions, reverse and output-time offset/chase/fades. Smooth S fades use native clip protocol
v3 (96 bytes; v1/v2 meanings retained). Automatic same-track two-clip crossfades are resolved
in document edits, never in `render`. This is fixed-duration preparation, not realtime tempo
following. See [design, limits and evidence](../research/prepared-sampling.md).
