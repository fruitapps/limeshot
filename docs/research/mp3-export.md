# MP3 export

2026-10-27, EXP-005. Independent implementation from public format/API documentation.

## Reference and encoder choice

FL's [export manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/fformats_save_export.htm)
(`OBSERVED`) documents bitrate selection, MPEG-1 sample rates 32/44.1/48 kHz and codec startup silence; WAV
is preferred for aligned production assets. Existing live menu inventory (`VERIFIED`, session 15) records Cmd+Shift+R
and the MP3 export entry. No new FL live observation was obtained in this session (`UNKNOWN` for any unrecorded live dialog details).

The implementation (`VERIFIED`) pins [rusty_mp3 1.0.5](https://docs.rs/rusty_mp3/1.0.5/rusty_mp3/),
Apache-2.0, with default features disabled and no runtime dependencies or C/FFI. Package metadata,
LICENSE (Copyright 2026 Mata Network) and API/source were inspected. Shine-RS is LGPL-2.0;
it was not selected for this permissive-dependency path. No FL encoder binary is loaded or copied.
The encoder's own quality claims are not adopted as LimeShot verification (`UNKNOWN`).

## User workflow

File › Export MP3… / Cmd/Ctrl+Shift+R selects MP3 in Export audio. File › Export WAV… / Cmd/Ctrl+R
selects WAV. The dialog can switch between formats. MP3 offers stereo CBR, all standard MPEG-1
bitrates from 32 to 320 kbps (default **320**, deliberately above the recorded FL 192 kbps default),
and 32/44.1/48 kHz. Switching from a higher WAV rate selects 48 kHz. WAV retains 16/24-bit integer
and 32-bit float depth choices. The suggested filename/filter follows the selected encoder. A mismatched suffix is rejected
instead of rewriting the chosen path after the save panel's overwrite confirmation.

CLI `render`, `run --render` and `demo --render` choose `.wav` or `.mp3` by suffix,
case-insensitively. `--bitrate` selects MP3 kbps; `--bits` applies to WAV. Example:

```sh
cargo run -p limeshot-app --bin limeshot-cli -- render song.lime song.mp3 --rate 48000 --bitrate 192
```

## Integration and failure handling

Both formats call the same `render_offline_with`, using fresh hosted processors, the selected
Song/Pattern/time-selection range, PDC trimming and up to four seconds of release/effect tail.
The report describes the rendered PCM before codec delay/padding. The encoder runs on the UI's
export worker or the CLI thread, never the realtime callback. The current export retains the
whole PCM mix and the encoder's internal buffers; long exports can consume substantial memory.

Invalid MP3 sample rates/bitrates fail before plugin setup/rendering. MP3 input clips finite overs
to ±1 and replaces non-finite samples with zero. The render report retains the pre-encoding peak
so clipping warnings still describe the mix. Packets are drained after finish, preserving the final
Info/LAME delay/padding tag. The result is written to a temporary file beside its destination,
flushed/synced and atomically persisted; failures preserve existing files and clean up temporary
files. Missing plugins still fail rather than producing an incomplete mix.

## Validation and limits

Automated tests (`VERIFIED` in unit, engine, CLI and headless UI suites) decode with the separate Symphonia decoder, covering all 42 rate/bitrate pairs,
left/right tone frequencies and separation, MPEG header rate/bitrate and approximate CBR size,
non-frame-aligned lengths, very short/silent/non-finite input, invalid settings, and write failure
cleanup. Engine tests compare WAV/MP3 render reports/ranges/tails; CLI tests render the hosted demo,
check nonzero finite stereo output and preserve an existing file on invalid options. Headless UI
tests exercise shortcuts and format switching, including conversion of an unsupported WAV rate.
The [MP3 dialog](../screenshots/current/export-mp3.png) was also rendered and visually inspected
on macOS Apple Silicon (`OBSERVED`). A hosted demo exported at 48 kHz / 192 kbps was decoded by Apple's
AudioToolbox (`afconvert`, `VERIFIED`): stereo, exactly 806,202 PCM frames matching the WAV reference,
peak 0.64877 and 26.50 dB sample-difference SNR against 16-bit WAV. This is one deterministic
music fixture, not a listening-quality certification (`UNKNOWN`). The first sandboxed system decode failed;
the permitted decode succeeded.

This is stereo CBR export, not full FL render-dialog parity: VBR, simultaneous formats, metadata,
mono modes and export cancellation are not added. Gapless-aware players can use the tag; LimeShot's
sample loader currently uses Symphonia's default format options and retains codec padding (`OBSERVED`). MP3
reimport is therefore not sample-aligned and should not replace WAV in production loops.
Listening quality, additional players and Windows/Linux runtime remain unverified (`UNKNOWN`). See the
progress record for the exact local build/check results and the human verification queue.
