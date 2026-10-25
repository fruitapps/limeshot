# Sampler, mixer and editor workflow — 2026-10-25

## Session and base

Branch `codex/sampler-mixer-workflow` in its own managed worktree, stacked on unmerged PR #11
(`codex/plugin-workflow-fixes`, `b54a522`). Older checkouts preserved. Draft PR targets PR #11's branch; never merge
automatically. The change list and validation are in the 2026-10-25 progress.md entry.

## Remaining work

* Live checks on the isolated `live_check` harness: native sampler drag passthrough, plugin toggle/raise, OS file
  drops onto the Playlist (pointer position from AppKit), Y scrubbing of audio clips.
* Samples above 6 million stereo frames still need trimming; streaming from disk would lift the limit.
* Native Windows/Linux embedding remains pending.

Every future development session must create a NEW codex/ branch, NEW separate managed worktree and NEW attached
draft PR based on the latest unmerged continuation after checking merge state. Preserve older checkouts, carry
implementation through validation/documentation/commit/push, never merge automatically, and create further sessions
only when the owner asks. Repeat this policy in every handoff.
