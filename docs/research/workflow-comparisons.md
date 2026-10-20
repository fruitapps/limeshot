# Workflow comparisons: FL Studio 26.1.1 vs LimeShot

Each record follows START.md §3.4. "Reference" is the FL Studio manual page (see `fl-studio-analysis.md` for URLs). REA evidence is cited where it exists.

---

### W1. Build a drum pattern
- **Reference version:** 26.1.1.5313 · **Reference:** `channelrack.htm`
- **Expected:** Steps toggle on with left-click and off with right-click; 16 steps by default; playback in PAT mode loops the pattern.
- **Observed:** Not observed live (no Screen Recording permission). Documented behaviour only.
- **Sequence:** Add sampler channel → load sample → click steps 1, 5, 9, 13 → Space.
- **Interface:** Channel Rack row: mute · pan · vol · mixer target · name · steps.
- **REA evidence:** none (the UI is custom-drawn; see `rea-findings.md`).
- **Confidence:** VERIFIED (docs)
- **Unknowns:** Exact drag-paint behaviour across steps (the docs mention left-drag painting elsewhere; INFERRED).
- **LimeShot:** Same row structure. Left-click toggles a step on (drag paints on), right-click toggles off (drag paints off). Steps are notes at `step × 240` ticks.

### W2. Write a melody
- **Reference:** `pianoroll.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** Open the Piano Roll for a channel (F7 or right-click channel › Piano roll). Draw tool: left-click adds, drag moves, edge drag resizes, right-click deletes. Snap follows the global snap. The velocity lane is below.
- **LimeShot:** Identical mouse semantics for the Draw tool. Ctrl+drag rectangle-selects. The velocity lane has draggable lollipops. Clicking the keyboard auditions.
- **Unknowns:** Default note length in a new project (INFERRED: one step, or the last-used length). LimeShot uses the last-touched note's length, starting at one step, as the manual describes.

### W3. Arrange patterns
- **Reference:** `playlist.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** Select a pattern in the toolbar selector; left-click in the Playlist places it; clips are linked instances; Make unique clones the pattern.
- **LimeShot:** Same. The Playlist places the *current* pattern on left-click in empty space. Right-click deletes; edge drag resizes; Ctrl+B duplicates. The clip menu offers "Make unique".

### W4. Pattern vs Song playback
- **Reference:** `toolbar_panels.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** PAT loops the selected pattern; SONG plays the Playlist from the song position; L toggles.
- **LimeShot:** Same. In SONG mode, playback stops at the end of the last clip unless loop is enabled (FL's default is to loop the song; LimeShot loops by default too, with a toolbar loop toggle).
- **Unknowns:** Whether FL rounds the song end to the next bar (INFERRED: yes). LimeShot rounds up to the next bar.

### W5. Route channels to mixer tracks
- **Reference:** `channelrack.htm`, `mixer.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** The channel's mixer-target field is dragged vertically; new channels go to Master by default; Ctrl+L routes selected channels to free inserts.
- **LimeShot:** The mixer-target field is dragged vertically or scrolled (0 = Master). Ctrl+L assigns the selected channel to the first free insert. **Deviation:** new channels are auto-assigned to the next free insert, because that makes the mixer immediately useful. This is a user-visible difference from FL's default (Master) and is documented in `decisions.md`.

### W6. Mix volume and pan
- **Reference:** `mixer.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** The fader is post-FX volume. Pan defaults to a circular law. Mute is left-click, solo Ctrl+click or right-click.
- **LimeShot:** A vertical fader (dB, −∞ to +6 dB), a pan knob, mute and solo buttons with the same click semantics, peak meters. The pan law is a balance law: centre is unity on both sides and the opposite side attenuates linearly, so panning never boosts. See `audio-engine.md`.

### W7. Save, reopen, export
- **Reference:** shortcuts page · **Confidence:** VERIFIED (docs)
- **Expected:** Ctrl+S, Ctrl+Shift+S, Ctrl+O, Ctrl+R export WAV.
- **LimeShot:** Identical shortcuts (Cmd on macOS). The export dialog chooses the range (song/pattern), sample rate and bit depth (16/24/32f).

### W8. Audition a sample in the Browser
- **Reference:** `browser.htm` · **Confidence:** VERIFIED (docs)
- **Expected:** Left-click previews a sample.
- **LimeShot:** Clicking a `.wav` entry previews it through the engine's preview voice. A context menu loads it into the selected channel or a new channel.
