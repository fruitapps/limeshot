# Project model

Source: `crates/limeshot-core/src/{model,command,document,time}.rs`, `crates/limeshot-project/src/lib.rs`.

## Entities

```
Project
├── title, info, tempo (milli-BPM), time_signature, swing, next_id, extra{}
├── channels: [Channel]            — the Channel Rack, shared by every pattern (order = rack order)
│     id, name, color, instrument, volume, pan, muted, solo, mute_locked, mixer_track, root_key, extra{}
│     instrument = Plugin(PluginRef)  — active playback; pre-VST3 variants are kept only for conversion
│                | Sampler{sample?, gain_db, pitch, start, end, reverse, attack_ms, decay_ms, sustain, release_ms, one_shot,
│                           looping, loop_start, loop_end, cutoff_hz, resonance}
│                | Synth{waveform, attack/decay/release ms, sustain, gain_db, octave, detune_cents,
│                        cutoff_hz, resonance, filter_env, filter_decay_ms, lfo_rate_hz, vibrato_cents, lfo_cutoff}
│                | Unsupported(raw JSON)    (kept verbatim, silent)
├── patterns: [Pattern]
│     id, name, color, length_steps (None = auto), notes: [Note], extra{}
│     Note = {id, channel, key, start, length, velocity, pan, muted}   (ticks; muted omitted when false)
├── playlist
│     tracks: [{name, muted, mute_locked, color?}]
│     clips:  [{id, source, track, start, length, offset, muted, fade_in, fade_out, extra{}}]
│     source = Pattern(id) | Audio(channel id) | Automation(id)
├── mixer.tracks: [MixerTrack]     — index 0 = Master, 1.. = inserts
│     name, volume (linear, ≤ 2.0), pan, muted, solo, mute_locked, effects, eq: [EqBand; 3], routes, extra{}
│     effects: up to 10 × {id, kind, enabled, mix (host dry/wet, default 1.0), params, plugin?, extra{}}
└── automations: [Automation]      — FL Studio's automation clips
      id, name, color, target, points: [{tick, value 0–1, tension −1–1, mode}], length, extra{}
      target = Tempo | ChannelVolume(ch) | ChannelPan(ch) | MixerVolume(i) | MixerPan(i) | EffectParam{track, effect, index} | PluginParam{plugin, id}
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
  clips. Manual `fade_in` / `fade_out` durations are stored in ticks and applied by the audio-clip voice.
* **Automation clips** are linked instances of an `Automation`, like pattern clips. Values are normalised on the
  control's own scale (`Mapping`: linear, logarithmic, or the fader's cube-root travel); a point's tension shapes the
  segment after it. A target that no longer exists leaves the automation inert, not deleted.
* **Trimming.** `ResizeClipStart` moves a clip's start while keeping its end; the offset moves with it (never below 0).
* **Song length** is the end of the last clip rounded up to whole bars.

### Plugins

`PluginRef` (`plugin.rs`) names a hosted plugin: format (`"VST3"`), bundle path, class id, name, vendor, and its
state, an opaque blob shared between undo copies and saved as base64. A channel uses one as `Instrument::Plugin`; an
insert slot as `EffectKind::Plugin` with `EffectSlot::plugin`. The running plugin's state is copied in by
`Document::store_plugin_state` before saving; that is not an edit (no undo step, same revision). A state that is too
large or not valid base64 loads as empty, so a damaged plugin blob never stops a project from opening.

## Playlist track operations

Track mute locks persist with a default of false for older files; a locked active channel/track remains audible
when another is soloed. Track colors are optional. `InsertPlaylistTrack`, `RemovePlaylistTrack` and
`MovePlaylistTrack` remap clip track indices. Track cloning preserves clip timing, offset and mute state while
sharing source patterns, audio channels and automations; it does not duplicate instruments or effects.
`crates/limeshot-core/tests/playlist_tracks.rs` covers index remapping, undo, save/reload and clone metadata.

## Commands, undo and gestures

Every mutation is a variant of `limeshot_core::Command` (serde-serialisable, including `Batch`).
`Command::apply(self, &mut Project)` validates its inputs: missing ids give a `CommandError`, numbers are clamped,
and NaN is rejected.

`Document::execute(cmd)`:

1. clones the project and applies the command to the clone, so an error leaves the document untouched;
2. skips the change if nothing changed;
3. pushes the previous project onto the undo stack (limit 200), unless the command *merges*: during an open gesture
   (`begin_gesture` … `end_gesture`, opened by the UI on drag start), consecutive commands with the same `merge_key()`
   (for example `SetChannelVolume` for the same channel, or `MoveNotes` for the same note set) share one undo step;
4. clears the redo stack and bumps `revision` (the UI uses it to know when to recompile the snapshot).

Snapshot undo was chosen over inverse commands because projects mostly contain notes and settings. Embedded plugin state uses shared blobs, so undo clones do not copy PCM.
A clone costs microseconds for typical projects, and snapshots cannot drift from the forward path.

**Modified state.** Each distinct content state gets an id. `mark_saved` records the current id, and undoing back to
it makes the document unmodified again.

## Sample references

In memory, sample paths are **absolute**. The app resolves them when a project is opened. On save, `limeshot-project`
writes paths inside the project folder **relative** (with `/` separators) and leaves others absolute. Decoded audio
lives in `limeshot_audio::assets::SampleCache`. External audio is keyed by resolved path; embedded Lime Sampler PCM is keyed by a content hash under a synthetic `/.limeshot-embedded/` path, without writing a file. Plugin state carries portable embedded PCM and optional source-path hints.

## File format (`.lime`)

UTF-8 JSON:

```json
{ "format": "limeshot-project", "version": 2, "generator": "LimeShot 0.1.0", "project": { … } }
```

* `from_json` checks `format` and refuses newer `version`s (`ProjectError::TooNew`), so a file is never half-read.
  Older versions go through `migrate(version, value)`. Version 2 accepts version-1 envelopes; document normalization converts their in-process instruments and effects into hosted Lime state after resolving sample paths.
* `#[serde(flatten)] extra` on the project, channels, patterns, clips and mixer tracks keeps unknown fields, and
  `Instrument::Unsupported` keeps unknown instrument objects. A file from a newer minor feature set round-trips.
* After parsing, `Project::sanitize` repairs hostile values. It clamps ranges, replaces NaN, drops notes for missing
  channels and clips for missing patterns, sorts notes, adds playlist tracks referenced by clips, and moves `next_id`
  past every id in use.
* Files larger than 256 MB are refused.
* **Atomic save:** write `.<name>.tmp-<pid>` in the target folder, `fsync`, rename over the target, then `fsync` the
  folder on Unix. The temp file is removed if anything fails.

* **Numbered save:** File › Save new version writes a complete, synced temporary file, then publishes it with
  `tempfile::NamedTempFile::persist_noclobber`. Existing files and directories are skipped, including concurrent
  saves. A trailing numeric suffix increments; other names start at `_2`. The new path becomes active only after
  success; the document content and undo history are unchanged.

## Tests

`limeshot-core` (document/model/time tests) and `limeshot-project` cover undo/redo, merging, modified state, linked
clips and Make unique, step toggling, note editing, sanitising, unknown-field round trips, version refusal and
save/reload equality. `limeshot-app/tests/mvp_workflow.rs` saves, reopens and compares a complete composition.

## Lime VST3 references (D39)

Instruments and effects are `PluginRef`s to Lime bundles (`lime:<bundle>`). Format 2 reads version-1 files, keeping
their instrument/effect fields and unknown data; the host resolves relative sample paths, embeds the audio and
replaces those sources with Lime VST3 state. Slot enabled state is the bypass. Track-EQ bands drive a dedicated
marked Lime EQ after the ten insert slots. Compressor sidechain indices and key-enable state follow mixer track
edits. See PLG-013…029.


## Sample migration boundaries

Decoded audio must meet the native sampler's shared rate/frame limits and the actual serialized state-size bound.
Missing, rejected or unapproved sources retain a pending plugin reference with settings/path metadata. Pending
state is not cleared by a failed retry, survives save/reopen, and blocks export with an actionable error. A successful
conversion removes the marker. Reload runs normalization again; explicit source selection is undoable.

Opening a .lime file and recovery canonicalize unresolved source paths against the loaded project folder; outside
paths and symlink escapes are not automatically copied into portable state. Explicit GUI source selection clears
that restriction through ApproveSamplerSource in the source-assignment batch; CLI users can opt in with --allow-external-samples. Already embedded audio
and normalizer commands for explicitly selected/new sources keep their existing workflow. First-party lime:
references resolve only known bundle filenames; arbitrary VST3 paths remain the host's explicit plugin-loading boundary.
