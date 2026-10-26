# Alpha.1 sampler and fade follow-up (2026-10-26)

The owner reported slow native controls, unstable wrapper resizing, expanding Browser rows, expensive long
sample imports/previews, missing Playlist scrollbars, and manual-only pattern clip lengths. The owner approved
an FL-style Keep on disk design after consultation, then chose FL's documented 2 GB cap instead of the initial
four-hour suggestion. Browser preview Stop was added to the request.

## Behavioral evidence

Image-Line's [Sampler manual](https://www.image-line.com/fl-studio-learning-content/fl-studio-online-manual/html/chansettings_sampler.htm)
describes Keep on disk for Sampler and Audio Clip channels, an Auto keep long audio on disk preference, a 2 GB
per-sample allocation limit, and direct support for 16/32-bit WAV. Compressed files cannot use that option directly.
It also warns that memory pressure and disk swapping can cause underruns. The manual does not establish FL's
internal buffering algorithm. LimeShot's implementation is independent and does not claim identical internals.

The [Audio Clip manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/playlist_audioclip.htm)
describes triangular length handles at the upper edge and circular tension handles dragged vertically. It offers
single curves and smooth curves. LimeShot implements single curves with its own monotonic quartic blend; exact
FL tension mapping, smooth curves and automatic crossfades remain outside this change. Both fade length and
curve gestures apply relative changes to selected audio clips, in one undo step.

No FL code, extracted assets, or decompiled implementation was used.

## Implementation

Short samples retain the portable embedded state (six million stereo frames; 64 MiB plugin-state bound).
Long imports and previews prepare a seekable 32-bit stereo WAV cache on a worker, including decoded compressed
formats. The accepted limit is **2,000,000,000 bytes of decoded stereo float audio**, or 250,000,000 frames, rather
than compressed input size or a fixed duration. At 48 kHz that is about 86.8 minutes. The file header is checked
before decoding known durations; unknown durations are bounded packet by packet. This deliberately uses a
precise decimal GB definition for the chosen product limit.

Disk sources retain paths in sampler state, with paths relative to the project on save and resolved on load.
Keep on disk is available in Lime Sampler; General settings exposes Auto keep long audio on disk (default on).
The latter controls automatic host imports/previews. Native sample dialogs prepare and serialize on a worker;
project reopening prewarms disk sources before restoring plugin state. Outside-project source approval remains
required when opening projects. Moving a project with linked sources requires moving its audio alongside it;
short embedded samples remain self-contained. The cache lives in the operating system's temporary directory
and is reconstructed from the original source after eviction. It is keyed by canonical source, size and mtime.

Each decoded disk reader has 32 pages of 4,096 stereo frames: **1,048,576 bytes of audio page storage**, regardless
of source length. Prepared extrema use 1,024-frame blocks (under 2 MB at the limit). A worker owns file I/O;
realtime reads use atomic sample words and a sequence check. Missing pages produce silence and increment an
underrun counter rather than blocking the audio callback. Forward/reverse read-ahead requests are bounded;
heavily scattered simultaneous voices can exhaust the page cache. A new, uncached seek may briefly underrun.
The native editor reports missed frames and disk errors. Offline VST3 instances use the actual offline process
mode and wait for pages, so faster-than-realtime exports do not outrun the disk worker. Export instances run
locally even when realtime playback uses the optional bridge.

Prepared extrema drive Browser, Playlist and native sampler waveforms without walking the long source in the
UI or displacing audio pages. Fade frame counters use f64 because adding one stops advancing an f32 counter
after 2^24 frames. Version 2 of the existing fixed-size clip SysEx carries two bounded tension values; linear
fades retain version 1 compatibility. Existing native bundles must be rebuilt to play curved fades.

## Performance review

The toolbar percentage is audio callback render time divided by the buffer deadline, not system CPU and not
total process CPU. Its tooltip now states that distinction. It includes hosted audio processing but excludes
GUI, loading, and most control work. It must not be used as a process-CPU benchmark.

Inactive native voices formerly scanned 128 slots per sample. Processing now walks only the occupied prefix,
retains all 128 voice slots and original stealing/ordering, and trims inactive slots from its end. Plugin editors
poll telemetry on their existing repaint timer instead of waking on every audio block. Browser loading/preview
repaints are bounded; inactive host polling drops from 20 to 5 Hz, with audio/playhead and input repaint paths
retained. Unchanged AppKit child frames are not assigned again. Browser search indexes on a worker, caches the
current query, and paints only visible rows. Channel labels reserve type space; all tree labels constrain width.

Native drag values accumulate locally without reading rounded/audio-thread-delayed values back every frame.
Clicks on numerical values open inline entry (Enter or focus loss applies; Escape cancels), and right-click text
menus are removed from Lime Native controls. Shift fine dragging and double-click reset remain.

## Verification

Automated and visual results are recorded in the current progress entry. Native third-party editor interaction,
Windows and Linux runtime behavior remain separate from headless checks. The owner updated the workspace policy
on 2026-10-26: create and use branches in `~/Developer/limeshot` and run builds/tests there to retain Cargo caches.
Do not use separate worktrees or clones for routine branch work; preserve local changes when switching. Repeat
this preference in handoffs.
