# Repository integrity audit — 2026-10-22

Audited the clean continuation worktree `limeshot-s16` on `s16-automation`, starting at `ec46e8f`.
The primary checkout was not reset or modified. The initial inventory had 298 tracked files and no untracked files.

## Findings and repairs

- Refreshed README and known limitations: MIDI input and note/audio recording exist; compressed sample decoding,
  manual clip fades and visible channel/mixer/effect automation exist. Added the newer effects, corrected the Piano
  Roll lane description and removed hard-coded feature counts. Updated architecture summaries and screenshot flags.
- Recorded the successful full CI matrix at `38d9a7b`, including the supported Intel runner. Native GUI/audio-device
  checks remain separate; see [platform-validation.md](platform-validation.md).
- Removed the unused `tempfile` development dependency from `limeshot-engine` and its lockfile edge after checking
  all engine source, tests and manifest references. Other declared workspace dependencies have source references;
  this was a reference audit, not a compiler proof that every dependency is necessary in every configuration.
- The macOS package copied LimeShot's licences but omitted embedded font notices. Added the four unmodified
  fallback-font licence texts from the locked `epaint_default_fonts` 0.36.2 crate and package all six font texts
  (including Inter and JetBrains Mono). Updated attribution and stopped silently ignoring licence-copy failures.
- Added `scripts/check_repository.py` to CI: offline checks of tracked inline Markdown/HTML local file targets,
  literal Rust embedded assets and explicit feature evidence paths/test function names. It excludes fenced examples
  and does not check anchors, external links or arbitrary prose paths. Existing inventory checks remain canonical.

## Cleanup decisions

No tracked file was shown to be obsolete enough to delete. Rust modules, examples, scripts, sample generators,
fixtures, screenshots, licences, briefs and chronological development notes were retained after checking references.
The sole byte-identical pair, `observe-ui-965.json` / `observe-ui-966.json`, records separate historical REA failures;
keeping the observation identifiers preserves evidence continuity. `CLAUDE.md` is a short entry point to `AGENTS.md`,
not an obsolete competing instruction file.

Ignored directories were `target/` (~17 GB), `dist/` (~25 MB before rebuilding), `out/` (~828 KB) and Python bytecode.
Build caches and generated release/demo outputs remain useful and are excluded from Git. No personal files were
removed. A stale `scripts/__pycache__/` was cleared; Python inventory commands regenerate that cache normally.
All source/configuration cleanup is recoverable from Git.

## Validation

- `git fsck --no-dangling`: passed; no corrupt or missing Git objects reported.
- `cargo test --workspace`: 335 tests pass, including 125 headless UI tests. Synthetic FLP fixtures are checked
  against their generator by the tests; the FL26 and audio decoding fixtures remain referenced and exercised.
- `cargo clippy --workspace --all-targets -- -D warnings`, all-target workspace build, rustfmt and diff checks pass.
- Feature/shortcut/setting/menu inventories pass; the new reference check initially passed 51 local links,
  3 embedded assets and 132 explicit evidence paths (counts increase with new documentation).
- Resolved locked Apple Silicon dependency metadata inspected offline. A full all-platform metadata request could
  not run offline because Android-only crate sources were not cached. No crate versions were upgraded.
- `scripts/package-macos.sh`: app and DMG rebuilt. `codesign --verify --deep --strict`, `plutil -lint` and
  `hdiutil verify` pass; all six packaged font notices match source bytes. From `/private/tmp`, the bundled CLI
  rendered the demo and reloaded/rendered it with byte-identical WAV output. No GUI was launched.

## Remaining checks

Live plugin embedding, hardware MIDI/audio, other-platform GUI/device checks, Developer ID signing/notarisation,
Finder document events and Windows/Linux packaging remain open. Third-party Rust licence-notice aggregation for
shipping packages remains incomplete; the font repair is not a complete distribution compliance audit.
External research URLs and every historical prose reference were not exhaustively verified. Historical screenshots
and notes describe their original sessions and are not current interaction-verification evidence.
