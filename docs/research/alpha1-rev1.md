# Alpha.1 Rev 1 continuation — 2026-10-27

## Evidence and scope

The inherited uncommitted implementation on `codex/alpha1-rev1-issues` covered all owner-requested
areas, but had no new regression tests or tracking updates. No repository agent lock or lock named
Argon was found; Codex's empty thread-writer lock files are application-managed and were left intact.
The continuation preserves that work in the primary checkout and reuses Cargo caches.

INFERRED from official documentation: Image-Line's [Playlist manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/playlist.htm)
describes arrangement variants sharing sound sources, optional resampling during clip resize, and
freehand automation editing with snapped points. Its [Browser manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/browser.htm)
is the reference for preview controls. The owner specifically requests silence while right-dragging
and playback on release. These are documentation/owner requirements, not new live FL observations.
Exact FL visual and interaction equivalence is UNKNOWN pending live comparison.
Note-off follows each instrument’s own release envelope and one-shot behavior; release tails can ring.

## Implemented

- Piano Roll note audition follows mouse hold/release and replaces the preceding pitch. The inherited
  keyboard handler released it on the next frame; the continuation keeps it active during note drags.
- Playlist clip and internal sample/picker drags scroll at viewport edges. Edited/created patterns
  select the Playlist source; a held edit in an old pattern does not switch the active editor back.
- Sample/Automation/Patterns focus buttons expose fade visibility, zero-crossing, resample stretch,
  STEP and SLIDE. Patterns has no extra options.
- Top navigation draws clips behind a translucent viewport thumb.
- Arrangements have independent tracks/clips/markers, shared channels/patterns/automation/mixer, saved
  active selection, clone with fresh clip/marker IDs, rename/delete/switch and overlay merge.
- Track selection supports click/drag/toggle/range. Saved undoable heights, collapse, size locks and
  contiguous groups use variable row geometry and hidden child rows. Menu grouping's parent handling
  was corrected. Nonadjacent selected rows group the span between them.
- Browser preview right-drag stops playback, shows the chosen offset and starts there only on release.

## Limits and follow-up

Stretch currently resamples through shared sampler pitch, changes pitch, affects linked uses and is
bounded by the sampler pitch range. Only right-edge stretch is implemented. Independent clip stretch,
pitch-preserving DSP and exact source-offset handling remain part of INS-007 / PL-023. Zero-crossing
uses decoded PCM and tick resolution; disk-only samples and sample-exact stereo crossing policies
need further work. STEP does not implement Shift pulse drawing or exact FL tension inheritance.
Arrangement merge overlays at the start on existing track indices; FL's position/track policy dialog
and additional FLP arrangement import remain open. Opening a multi-arrangement project in an older
LimeShot build is unsupported because older commands do not maintain inactive arrangement references.

Track lights show content activity inferred from the playhead, not measured audio output, tails or
full mixer/solo audibility. Focus options are session-local. Exact FL live behavior and Windows/Linux
interaction checks remain in the human verification queue. New features are IMPLEMENTED, not VERIFIED.

Owner-requested next work: investigate slow/unresponsive Lime Native VST3 editors on Windows; research
and redesign plugin wrapper windows, especially the Settings tab, against FL behavior.


## Local validation

macOS 26.6.2 (25G83), arm64: all 480 workspace tests pass across the workspace/package runs,
including 169 UI tests. The first sandboxed run could not open the existing plugin bridge socket;
permitted runs pass. The new audio regression initially omitted the preview's 1 ms attack/10 ms
release; corrected assertions verify source offset and replacement after those fades at block sizes
1/64/333/1024. All twelve native release bundles rebuilt with `native-plugins/build.py`; workspace
build, strict all-target Clippy, formatting, feature inventories, repository checks and whitespace pass.
The [offscreen Playlist screenshot](../screenshots/current/alpha1-rev1.png) was inspected; the missing
arrangement arrow glyph was replaced with painter geometry and the final screenshot re-inspected.
Rendering required host GPU access because the sandbox exposes no adapter. No live GUI/audio or
Windows/Linux runtime pass is claimed.
