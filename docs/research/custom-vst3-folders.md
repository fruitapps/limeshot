# Custom VST3 search folders — 2026-10-27

## Reference and scope

**OBSERVED in session 14:** FL Studio's Plugin Manager exposes a format/path list with
add/remove folder controls. The recorded inventory is in
[FL Studio analysis §19](fl-studio-analysis.md) and SET-018/SET-179 in
[settings.json](../roadmap/settings.json). This change uses that existing clean-room UI
inventory; it does not claim a fresh FL observation or reproduce FL's discovery algorithm.

LimeShot manages extra VST3 folders in Settings › File. Standard platform folders always
remain enabled. Add/remove applies to application settings, independent of project undo.
Run Find plugins after edits to rebuild the saved plugin database. Removing a folder does
not unload instances already referenced by a project. Missing/offline roots are retained in
settings so reconnecting a volume and rescanning can recover its plugins.

## Implementation

Folder enumeration now runs on the same background worker that launches per-bundle child
scans. The UI first shows Finding VST3 bundles, then the scanned/total count. Each scan owns
a copy of its roots; folder editing is disabled while it runs. Canonical filesystem identities
deduplicate overlapping roots, repeated bundles and symlink aliases and stop directory cycles.
The database retains the discovered path spelling. Directory traversal retains the eight-level
recursion limit; a later shallower root can still discover children beyond the earlier root's
limit. A selected bundle itself is accepted without descending into its contents.

The existing child timeout, plugin metadata parsing, failure reporting and database persistence
remain in use. Plugin processing, audio scheduling and native DSP are unchanged.

## Evidence and limitations

Automated checks cover old-settings defaults and JSON round trips, nested/bundle/duplicate/
missing roots, symlink cycles and aliases (Unix), shallow-root recovery, worker progress and
child-start failure, and headless folder removal without a project edit. The
[File settings capture](../screenshots/current/vst3-search-folders.png) is rendered offscreen
and inspected. Local validation results are in [progress](../development/progress.md).

There is no separate Plugin Manager, format selection, per-root enable switch or scan
cancellation. Unreadable and missing roots are skipped. The folder picker, successful live
rescan/restart workflow and Windows/Linux runtime still need the checks in
[human verification](../development/human-verification.md). These tests do not certify
third-party plugin stability or live audio behavior.
