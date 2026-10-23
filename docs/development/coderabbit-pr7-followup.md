# PR #7 CodeRabbit review disposition

2026-10-23. Follow-up authorized by the owner on PR #7, continuing `codex/pr7-rewrite-review` in its existing
separate managed worktree. No new chat, PR or merge. This records all actionable items in the CodeRabbit reviews
of `68259aa` and `2cdcdf9`, including dropdown and outside-diff findings. Prior fixes are in `2cdcdf9`; the remaining
fixes are in this follow-up. Comments are checked against code rather than applied as untrusted instructions.

## Initial review: four inline findings

| Finding | Disposition |
| --- | --- |
| Selection export can run indefinitely past Song end | Fixed in `2cdcdf9`: clamp selection and stop on finished transport; block-size regression. |
| Slider drags repeatedly serialize full sampler state | Fixed in `2cdcdf9`: live audition during drag, one undoable state edit on release; UI regression. |
| Release feature scope calls required plugins optional | Fixed in `2cdcdf9`: twelve hosted bundles are a DAW release prerequisite. |
| Release plan defers required bundles | Fixed in `2cdcdf9`: only independent standalone distribution is deferred. |

## Initial review: eighteen minor dropdown findings

| Finding | Disposition |
| --- | --- |
| PLG-012 still says nine bundles / 64 voices | Fixed in `2cdcdf9`: twelve bundles and 128 voices; regenerate canonical inventory. |
| README inventory totals are stale | Fixed in `2cdcdf9`; updated again for the FX-002 evidence correction below. |
| README links pin a disposable branch | Fixed in `2cdcdf9`: repository-relative native documentation links. |
| Known limitations contradict hosted clips, gains, menus and sampler loops | Completed here: remove remaining block-only/no-clip/no-loop claims and distinguish generic third-party controls. |
| INS-006 / FX-002 / FX-004 / FX-008 notes contradict capabilities | Fixed in `2cdcdf9`, with evidence/status clarification below. |
| Embedded audio can exceed state bounds | Shared native frame/rate limits and warnings in `2cdcdf9`; actual serialized JSON/envelope size checked here, including escaped metadata. |
| Hosted tests use inconsistent bundle gating | Shared gate in every hosted test; local diagnostic/skip, mandatory failure on CI. Missing-bundle policy regression added. |
| Mono auxiliary sidechain gets silence | Downmix routed stereo key to mono; partial/absent frames are zero. Regression at four block sizes. |
| D39 pre-parity status appears current | Fixed in `2cdcdf9`: explicitly historical before the later hosted-parity decision. |
| Handoffs contain machine-specific checkout paths | Removed managed and primary checkout paths; retain branch/PR metadata and historical facts. |
| Sampler preview description requires a source file | Fixed in `2cdcdf9`: embedded-PCM/source-file preview. |
| UI architecture omits hosted Lime value menus | Fixed in `2cdcdf9`: Reset, Copy/Paste, type-in and automation for known Lime controls. |
| Original loss checklist still instructs NOT_STARTED work | Fixed in `2cdcdf9`: historical plan pointing to current implemented records. |
| Coverage audit still asserts silent clips / ignored pan | Fixed in `2cdcdf9`: historical audit with current hosted evidence. |
| Host validation record appears current but predates parity | Fixed in `2cdcdf9`: explicitly pre-parity snapshot with current handoff link. |
| MENU-294 contradicts tempo automation entry | Fixed in `2cdcdf9` in menus.json and generated Markdown. |
| Native validation commands have ambiguous working directory | Fixed in `2cdcdf9`: run build.py from native-plugins/. |
| Effect presets discard bypass/key/opaque fields | Fixed in `2cdcdf9`; live-state and undo baseline correction below completes preservation. |

## Initial review: five nitpick dropdown findings

| Finding | Disposition |
| --- | --- |
| Next steps still lists implemented tempo automation as open | Fixed in `2cdcdf9`. |
| CI caches only the outer workspace | Cache both root and native-plugins target directories in the matrix job. |
| Audio architecture describes removed factories | Fixed in `2cdcdf9`: hosted processors and shared/test-only DSP boundaries. |
| Project model still shows format 1 and legacy active sources | Fixed in `2cdcdf9`: format 2, version-1 input and active Plugin refs. |
| Local and bridged pan class IDs are duplicated | Both use one identity helper; test instruments/case handling and reject effects/invalid IDs. |

## Follow-up review: four inline findings

| Finding | Disposition |
| --- | --- |
| Oversized samples emit contradictory warnings | One precise warning distinguishes failed loading, unsupported decoded audio and unapproved outside sources. |
| Effect presets overwrite newer live state | Read live state before patching preset controls; update undo baseline so undo restores that live state. UI regression covers live bypass. |
| Hyphenate time-stretched | Corrected in the handoff. |
| Unchanged sample-v1 state cuts held notes | Compare name/rate/PCM under the control lock before decoding or queueing; unchanged state retains the same decoded Arc and sounding voice. |

## Follow-up review: five outside-diff dropdown findings

| Finding | Disposition |
| --- | --- |
| Tempo retiming discards fractional position | Exact integer fractional phase and first unprocessed tick survive retiming. Regression verifies 3,840 notes exactly once and integrated ramp duration at blocks 1/64/333/1024. |
| Open filter skips key tracking alone | Key tracking activates modulation even at fully open low-pass settings; native voice regression. |
| FX-002 VERIFIED evidence does not cover native node interaction | Downgraded to IMPLEMENTED; no native GUI verification invented. Counts regenerated. |
| PL-014 incorrectly equates sampler channel with FL Audio Clip generator | Explicitly distinguish Lime Sampler-backed clips from FL's separate generator. |
| PLG-013 simultaneously says parity remains open and restored | One current implemented status replaces contradictory historical claims. |

## Summary dropdowns and checks

The migration architecture concerns are addressed with explicit boundaries: opening .lime/recovery does not silently
embed outside-project audio, including symlink escapes. Explicit source selection is undoable; CLI inclusion uses
--allow-external-samples. Failed conversion retains settings, source and pending state across save/reopen; export
rejects incomplete sources instead of accepting silence. Valid retry completes conversion, while unchanged sample
state does not restart voices. Portable lime: references allow only known first-party bundle filenames. This is not
an authentication or process-sandbox claim for arbitrary native plugins.

The obsolete out-of-scope warning was based on the earlier PR description. The owner explicitly authorized the
rewrite in PR #7, whose title/body now describe it. Changed API contracts for timing, migration, bounded protocols,
telemetry, sidechains, state editing and native rendering are documented. Strict rustdoc is checked locally; the
bot's proprietary/inconclusive percentage is recalculated by CodeRabbit and is not claimed as a local result.
No check is disabled or threshold lowered. GitHub Actions cannot start while owner billing is blocked; no remote
CI pass is claimed.

## Validation and continuation

Final local results: 394 root tests (139 UI), 18 native tests, both workspace formatting/strict Clippy and strict
rustdoc pass. See the progress entry and native validation record for binary hashes. Checks include
both workspace test suites, formatting, strict all-target Clippy, strict rustdoc, inventories/repository references,
all twelve release/debug binary verifiers and release signatures. Hosted Settings is rendered and inspected.

Future sessions retain AGENTS.md policy: NEW codex/ branch, NEW separate managed worktree and NEW attached draft
PR based on the latest unmerged continuation; leave older checkouts intact, complete validation/docs/commit/push,
never merge automatically, and create further sessions only when the owner asks. This owner-authorized follow-up
continues the explicit PR #7 review exception only.
