# Navigable undo history — 2026-10-27

## Reference

**VERIFIED (public manual only):** the [FL Studio Browser manual](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/browser.htm)
describes Current project › History as a list of undo points that can be selected with a
left-click, alongside sequential Edit › Undo/Redo. This is public documentation research,
not a fresh live observation of exact selection semantics or ordering preferences.

## LimeShot behavior

Edit › History shows all retained edit labels, newest first, in a bounded scroll area.
The selected row is the current content state. A numbered row restores the state after
that edit; Oldest retained state restores the snapshot before the oldest retained edit.
The explicit label matters once the 200-snapshot cap has discarded the initial state.
Undone edits remain available in the list until a new edit branches the timeline.

Navigation uses Document undo/redo, retaining saved-state identity, revision notifications,
plugin state snapshots and the existing command-only project mutation boundary. A real
jump ends any open gesture; a click on the current row is a no-op. Opening the menu does
not edit the project. New/open clears history as before. No serialized history or new
project format is introduced. Native/third-party edits that do not create undo steps
remain subject to the existing plugin-state limits.

## Evidence and limits

Core tests cover backward/forward navigation, saved state, invalid/current indices,
branching, project replacement, retention and compound gestures. The headless app test
clicks the real menu rows backward/forward and to the oldest state, then uses redo.
The [menu capture](../screenshots/current/undo-history.png) is produced with
`ui_shot --history --menu=Edit/History` and inspected offscreen. Local results are recorded
in [progress](../development/progress.md). The control remains in Edit rather than FL's
Browser; sorting preferences, configurable undo depth and persistent history are absent.
Live input/audio and Windows/Linux runtime are unverified.
