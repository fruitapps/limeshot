# Roadmap and feature tracking

* [`features.json`](features.json) — **the canonical inventory** (217 features, schema below).
* [`features.md`](features.md) — generated overview; run `python3 scripts/features.py generate` after editing the JSON.
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
