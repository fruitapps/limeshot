# Project model

Source: `crates/limeshot-core/src/{model,command,document,time}.rs`, `crates/limeshot-project/src/lib.rs`.

## Entities

```
Project
├── title, tempo (milli-BPM), time_signature, next_id, extra{}
├── channels: [Channel]            — the Channel Rack, shared by every pattern (order = rack order)
│     id, name, color, instrument, volume, pan, muted, solo, mixer_track, root_key, extra{}
│     instrument = Sampler{sample?, gain_db, pitch, start, end, reverse, release_ms, one_shot}
│                | Synth{waveform, attack/decay/release ms, sustain, gain_db, octave, detune_cents,
│                        cutoff_hz, resonance, filter_env, filter_decay_ms, lfo_rate_hz, vibrato_cents, lfo_cutoff}
│                | Unsupported(raw JSON)    (kept verbatim, silent)
├── patterns: [Pattern]
│     id, name, color, length_steps (None = auto), notes: [Note], extra{}
│     Note = {id, channel, key, start, length, velocity, muted}   (ticks; muted omitted when false)
├── playlist
│     tracks: [{name, muted}]
│     clips:  [{id, source, track, start, length, offset, muted, extra{}}]
│     source = Pattern(id) | Audio(channel id) | Automation(id)
├── mixer.tracks: [MixerTrack]     — index 0 = Master, 1.. = inserts
│     name, volume (linear, ≤ 2.0), pan, muted, solo, effects, routes, extra{}
└── automations: [Automation]      — FL Studio's automation clips
      id, name, color, target, points: [{tick, value 0–1, tension −1–1}], length, extra{}
      target = ChannelVolume(ch) | ChannelPan(ch) | MixerVolume(i) | MixerPan(i) | EffectParam{track, effect, index}
```

* **Stable ids.** `ChannelId`, `PatternId`, `ClipId` and `NoteId` are `u32`s from one project-wide counter
  (`next_id`). They are never reused within a project. Mixer and playlist tracks are addressed by index, as in FL Studio
  where they are numbered.
* **Steps are notes.** A step sequencer step is a note on the channel's root key starting on the 240-tick step grid. A
  channel shows as step buttons in a pattern when all its notes there are such steps (`Pattern::is_step_sequence`);
  otherwise the rack shows a note preview. Toggling a step adds or removes that note (`Command::SetStep`).
* **Linked pattern clips.** Clips reference patterns by id. Editing a pattern changes every clip that uses it.
  `MakeClipUnique` clones the pattern (with new note ids) and points one clip at the clone.
* **Pattern length.** Auto rounds the last note *start* up to whole bars (minimum one bar). Manual is 1–512 steps.
* **Clip window.** A clip plays its pattern's notes that start in `[offset, offset + length)`, each cut at the clip
  end. Patterns do not loop inside longer clips.
* **Audio clips** reference a sampler channel (FL Studio's audio-clip channel), so the channel's settings and mixer
  track apply. The clip plays the channel's sample from `offset` (ticks, converted at the project tempo) for `length`.
  `AddAudioClip` reuses the channel of an existing audio clip with the same sample. Deleting the channel deletes its
  clips.
* **Automation clips** are linked instances of an `Automation`, like pattern clips. Values are normalised on the
  control's own scale (`Mapping`: linear, logarithmic, or the fader's cube-root travel); a point's tension shapes the
  segment after it. A target that no longer exists leaves the automation inert, not deleted.
* **Trimming.** `ResizeClipStart` moves a clip's start while keeping its end; the offset moves with it (never below 0).
* **Song length** is the end of the last clip rounded up to whole bars.

## Commands, undo and gestures

Every mutation is a variant of `limeshot_core::Command` (serde-serialisable; about 50 variants, including `Batch`).
`Command::apply(self, &mut Project)` validates its inputs: missing ids give a `CommandError`, numbers are clamped,
and NaN is rejected.

`Document::execute(cmd)`:

1. clones the project and applies the command to the clone, so an error leaves the document untouched;
2. skips the change if nothing changed;
3. pushes the previous project onto the undo stack (limit 200), unless the command *merges*: during an open gesture
   (`begin_gesture` … `end_gesture`, opened by the UI on drag start), consecutive commands with the same `merge_key()`
   (for example `SetChannelVolume` for the same channel, or `MoveNotes` for the same note set) share one undo step;
4. clears the redo stack and bumps `revision` (the UI uses it to know when to recompile the snapshot).

Snapshot undo was chosen over inverse commands because projects are small: notes and settings, with no audio data.
A clone costs microseconds for typical projects, and snapshots cannot drift from the forward path.

**Modified state.** Each distinct content state gets an id. `mark_saved` records the current id, and undoing back to
it makes the document unmodified again.

## Sample references

In memory, sample paths are **absolute**. The app resolves them when a project is opened. On save, `limeshot-project`
writes paths inside the project folder **relative** (with `/` separators) and leaves others absolute. Decoded audio
lives outside the model, in `limeshot_audio::assets::SampleCache`, keyed by resolved path.

## File format (`.lime`)

UTF-8 JSON:

```json
{ "format": "limeshot-project", "version": 1, "generator": "LimeShot 0.1.0", "project": { … } }
```

* `from_json` checks `format` and refuses newer `version`s (`ProjectError::TooNew`), so a file is never half-read.
  Older versions go through `migrate(version, value)`. Version 1 is the first, so the chain is empty for now.
* `#[serde(flatten)] extra` on the project, channels, patterns, clips and mixer tracks keeps unknown fields, and
  `Instrument::Unsupported` keeps unknown instrument objects. A file from a newer minor feature set round-trips.
* After parsing, `Project::sanitize` repairs hostile values. It clamps ranges, replaces NaN, drops notes for missing
  channels and clips for missing patterns, sorts notes, adds playlist tracks referenced by clips, and moves `next_id`
  past every id in use.
* Files larger than 256 MB are refused.
* **Atomic save:** write `.<name>.tmp-<pid>` in the target folder, `fsync`, rename over the target, then `fsync` the
  folder on Unix. The temp file is removed if anything fails.

## Tests

`limeshot-core` (document/model/time tests) and `limeshot-project` cover undo/redo, merging, modified state, linked
clips and Make unique, step toggling, note editing, sanitising, unknown-field round trips, version refusal and
save/reload equality. `limeshot-app/tests/mvp_workflow.rs` saves, reopens and compares a complete composition.
