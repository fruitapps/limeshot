# Roadmap and feature tracking

* [`first-release.md`](first-release.md) — human-readable v0.1 scope, AI-assisted effort estimates and release gates, pinned to a main-branch baseline.
* [`first-release-features.md`](first-release-features.md) — complete disposition of all 272 baseline features for that release.

* [`features.json`](features.json) — **the canonical inventory** (272 features, schema below).
* [`features.md`](features.md) — generated overview; run `python3 scripts/features.py generate` after editing the JSON.
* [`menus.json`](menus.json) / [`menus.md`](menus.md) — every item of FL Studio's main menu bar against LimeShot's menus
  (status, keys, feature links; static lists such as templates and plugins).
* [`keybindings.json`](keybindings.json) / [`settings.json`](settings.json) — FL Studio's shortcuts and every control of its
  settings window and menu dialogs against LimeShot's (`.md` views generated alongside).
* [`mvp.md`](mvp.md) — the MVP definition and its acceptance status.
* [`milestones.md`](milestones.md) — milestone plan and state.
* [`known-limitations.md`](known-limitations.md) — what does not work yet, stated plainly.

## Conventions

Record fields: `id`, `category`, `name`, `description`, `fl_reference`, `limeshot_behavior`, `priority` (P0–P3), `status`,
`mvp`, `dependencies` (ids), `modules` (crates), `acceptance_criteria`, `tests` (test paths or recorded verifications),
`reference` {application, version, evidence}, `notes`.

Statuses: NOT_STARTED, RESEARCHED, DESIGNED, IN_PROGRESS, IMPLEMENTED, VERIFIED, BLOCKED, DEFERRED.

* **IMPLEMENTED**: code exists and meets its implementation criteria.
* **VERIFIED**: also passed the listed automated tests, or a recorded manual verification. Compiling is not verification.
* A visual placeholder or a stub is not IMPLEMENTED. Disabled controls for future features say "not implemented yet".

`scripts/features.py check` (run in CI) enforces the schema. It rejects unknown ids in dependencies and VERIFIED
without tests, requires modules for IMPLEMENTED/VERIFIED, and fails if `features.md` is stale.

Before implementing a feature: read its record and dependencies, refine its acceptance criteria, implement, add tests,
run checks, update the status, and add limitations to `notes` and `known-limitations.md`. Newly discovered FL Studio
capabilities get a record even if they are out of scope. Never delete incomplete features to improve the numbers.

## Menu inventory (`menus.json`)

Record fields: `id` (MENU-nnn), `menu`, `path` (FL's labels from the menu bar down), `section` (the in-window header),
`kind` (action, dialog, submenu, toggle, radio, list, link), `fl_keys` (macOS notation), `fl_state` (checked, selected,
disabled on a default project), `confidence`, `limeshot` (where LimeShot has it, "Menu › Item"), `status`
(IMPLEMENTED, DIFFERENT, LIMESHOT_ONLY, NOT_STARTED, NOT_PLANNED), `feature`, `notes`. The check requires a known
feature id and, for items LimeShot has, a label that exists in `crates/limeshot-ui/src`. When a menu item is added or
changed in `menus.rs`, update its record in the same change.
