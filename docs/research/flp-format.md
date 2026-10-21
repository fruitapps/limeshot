# FL Studio project format (`.flp`) — what LimeShot reads

Clean-room notes for the importer in `crates/limeshot-project/src/flp.rs`. Everything here comes from the public
outline of the format (a chunked file of typed events) and from **looking at the bytes of project files**: the 164
templates and demos inside `/Applications/FL Studio 2026.app` (read-only) and projects saved by FL Studio 26.1.1 for
the tests (`crates/limeshot-project/tests/fixtures/flp/fl26/`, how each was made is in its README). No FL Studio code
was examined, decompiled or copied, and the installation was not modified.

Confidence: **OBSERVED** = seen in files where we know what was entered in FL (the `fl26` fixtures) or consistent in
every shipped file; **INFERRED** = a reading that fits the files but was not checked against a known input.

## Layout

```
"FLhd"  u32 header length (6)  u16 format  u16 channel count  u16 PPQ
"FLdt"  u32 data length        event stream
```

All integers are little-endian. PPQ is 96 in every FL 20–26 file seen (OBSERVED); older demos use other values,
so positions are always scaled by the header's PPQ. An event is an id byte whose range sets the payload size:

| Ids | Payload |
|---|---|
| 0–63 | 1 byte |
| 64–127 | 2 bytes |
| 128–191 | 4 bytes |
| 192–255 | a 7-bit varint length, then that many bytes |

Exception (OBSERVED, FL 25 and later): event **172**, near the start of the file, carries 3 bytes, not 4. Reading 4
shifts every later event; in `lime-steps.flp` the tempo was lost that way.

Meaning comes from order: settings after "new channel" (64) belong to that channel, notes and names after "new
pattern" (65) to that pattern, and items after "new arrangement" (99) to that arrangement. A pattern can be opened
twice: FL 26 writes the notes of every pattern first and the pattern names later, after the channel list, under a
second event 65 (OBSERVED, `lime-notes.flp`).

Text events are UTF-16LE with a terminating NUL in current files and 8-bit in old ones (OBSERVED).

## Events

| Id | Meaning | Payload | Confidence |
|---|---|---|---|
| 17 / 18 | time signature numerator / denominator | byte | OBSERVED (3 and 4 after setting 3/4) |
| 21 | channel type: 0 sampler, 2 plugin generator, 3 layer, 4 audio-clip channel, 5 automation clip | byte | OBSERVED for 0, 2, 4, 5; 3 INFERRED |
| 22 | channel's mixer insert, up to FL 20 | byte | INFERRED (old demos) |
| 64 | new channel (rack index from 0) | word | OBSERVED |
| 65 | new pattern (number from 1) | word | OBSERVED |
| 99 | new arrangement | word | OBSERVED |
| 104 | channel's mixer insert, FL 21 and later; takes precedence over 22 | word | OBSERVED (808 kit on inserts 1–5) |
| 128 | channel colour `0x00BBGGRR` | dword | INFERRED |
| 156 | tempo × 1000 | dword | OBSERVED (130000, 128000) |
| 172 | unknown; 3 bytes in FL 25+ | — | OBSERVED size |
| 193 | pattern name | text | OBSERVED ("Bassline") |
| 194 | project title | text | INFERRED |
| 196 | sample path, may start with `%FLStudioFactoryData%` | text | OBSERVED |
| 199 | FL version, ASCII (`26.1.1.5313`) | text | OBSERVED |
| 203 | channel name | text | OBSERVED |
| 219 | channel levels: pan u32 at 0 (6400 = centre), volume u32 at 4 (10000 = default, 12800 = max) | data | INFERRED from defaults |
| 224 | pattern notes, 24-byte records | data | see below |
| 233 | playlist items of the current arrangement | data | see below |

Not read (left out, counted in the import report where it matters): plugin states, mixer inserts and their effects,
automation clip curves (channel type 5), layer children, channel envelopes, markers, further arrangements.

### Note record (event 224, 24 bytes)

| Offset | Field | Confidence |
|---|---|---|
| 0 | position, u32 ticks | OBSERVED |
| 4 | flags, u16 (0x4000 on every note drawn in FL 26) | — |
| 6 | channel, u16 rack index | OBSERVED |
| 8 | length, u32 ticks; 0 for Channel Rack steps | OBSERVED |
| 12 | key, u16 (60 = what FL shows as C5) | OBSERVED |
| 21 | velocity, 0–128; FL shows it as a percentage of 128 (34 % = 44, 98 % = 125, default 100) | OBSERVED |

Bytes 14–20 and 22–23 (fine pitch, release, pan, filter values, by FL's note properties) were not mapped.

### Playlist item record (event 233)

The record size changed between versions (OBSERVED by surveying every shipped file plus `lime-arrange.flp`):

| Size | Versions |
|---|---|
| 32 bytes | FL 10–12 demos, some FL 20 files |
| 60 bytes | FL 20–24 |
| 80 bytes | FL 24–25 |
| 88 bytes | FL 26 |

An earlier reading of "240 bytes in FL 25+" was a coincidence (240 = 3 × 80). The importer tries each size and
takes the first where every record has the base marker (or 0) at offset 4 and a non-zero length, and positions never
decrease (FL writes items sorted by position). That picks exactly one size for every file seen.

| Offset | Field | Confidence |
|---|---|---|
| 0 | position, u32 ticks | OBSERVED |
| 4 | base, u16 `0x5000` (0 in some old channel items) | OBSERVED |
| 6 | source, u16: `0x5000 + pattern number` for a pattern clip, else the rack index of an audio-clip or automation channel | OBSERVED |
| 8 | length, u32 ticks | OBSERVED |
| 12 | track, u16 stored as `499 − track` (500 tracks, track 1 = 499) | OBSERVED |
| 16 | u16, 0x0078 in nearly all records | — |
| 18 | flags, u16: 0x0040 normally; bit **0x2000 = muted** | OBSERVED (clip muted with the mute tool). Bits 0x1000, 0x4000, 0x8000 occur in demos, meaning unknown |
| 24 | pattern clips: start offset into the pattern, u32 ticks (`0xFFFFFFFF` = none). Audio and automation clips: f32 start (−1.0 = none) | OBSERVED for "none"; offsets INFERRED |
| 28 | end offset, same encoding | INFERRED |

When the time signature changes, FL rewrites pattern clip lengths to whole bars of the new signature (Pattern 1,
768 ticks of notes, became 864 = 3 × 288 at 3/4; OBSERVED). Positions stay in ticks.

## Version differences (summary)

* Event 172: 4 bytes up to FL 24, 3 bytes from FL 25.
* Mixer insert: event 22 up to FL 20, event 104 from FL 21.
* Playlist item records: 32 → 60 → 80 → 88 bytes.
* Text: 8-bit in old files, UTF-16LE in current ones.

## How to extend this

Make a small project in FL Studio with one known change, save it, and compare its events with the previous file
(the fixture README records how each file was made). Record what was entered, add the file to `fixtures/flp/fl26/`
and a test to `crates/limeshot-project/tests/flp.rs`, and mark the finding OBSERVED here.
