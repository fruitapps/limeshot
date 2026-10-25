# Contributing to LimeShot

LimeShot is an offline-first digital audio workstation with a pattern-based workflow, written in Rust.
Contributions to the audio engine, editors, file formats, accessibility, testing, documentation and platform
support are welcome. Small, reproducible bug reports and recorded device/host checks are useful contributions too.

The project is in early alpha (first prerelease: v0.1.0-alpha.1). Start with the existing behavior and tracked requirements, keep changes focused,
and describe what you tested. A feature being implemented does not mean every workflow or platform is verified.

## Find your starting point

Read these before choosing implementation work:

1. [Next steps](docs/development/next-steps.md) and the latest entries in the
   [progress log](docs/development/progress.md).
2. The relevant record in [features.json](docs/roadmap/features.json), including its priority, dependencies,
   acceptance criteria and known gaps. [features.md](docs/roadmap/features.md) is the readable generated view.
3. [Known limitations](docs/roadmap/known-limitations.md), the [MVP](docs/roadmap/mvp.md), and related issues or PRs.
4. The relevant [architecture documents](docs/architecture/) and [design decisions](docs/development/decisions.md).

Feature priorities run P0 through P3; satisfy dependencies before implementing dependent behavior. Check a record
against the source before assuming a feature is missing. For substantial architecture, format or workflow changes, open an issue with a
concrete proposal before investing in a large patch. Documentation fixes and small bug fixes can go straight to a PR.

Useful entry points include parser regressions with redistributable fixtures, focused editor interaction bugs,
accessibility improvements, and Windows/Linux/device validation. For audio or plugin work, first read the relevant
constraints below. [AGENTS.md](AGENTS.md) contains additional instructions for coding agents.

## Local setup

Install Git, a current stable Rust toolchain with Cargo, rustfmt and Clippy, Python 3, and the native compiler/linker
for your OS. The DAW manifest declares Rust 1.88 as its minimum; development and CI use current stable Rust. The
separate Lime Native workspace declares Rust 1.95. Do not infer a tested minimum compiler from a stable CI pass.

- **macOS:** install Apple's command-line developer tools (`xcode-select --install`).
- **Windows:** install the MSVC C++ build tools and Windows SDK, and use an MSVC Rust toolchain.
- **Linux:** install a compiler/linker and pkg-config, plus the development packages used by the CI workflow.
  On Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libasound2-dev libxkbcommon-dev libwayland-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev mesa-vulkan-drivers
```

Fork the repository on GitHub, then clone your fork and create a branch:

```sh
git clone https://github.com/YOUR-USERNAME/limeshot.git
cd limeshot
git remote add upstream https://github.com/fruitapps/limeshot.git
git switch -c fix/describe-the-change
rustup component add rustfmt clippy
cargo build --workspace
cargo run --release -p limeshot-app -- --demo
```

Keep unrelated local changes out of your patch. Avoid committing generated audio, screenshots of private projects,
plugin bundles, credentials, machine-specific settings or build directories. Use redistributable fixtures and
relative paths. The first dependency download/build needs network access; the application itself operates offline.

## Workspaces and code map

The repository's root Cargo workspace builds the DAW and CLI:

| Crate | Responsibility |
| --- | --- |
| `limeshot-core` | Musical time, project model, commands and document history |
| `limeshot-project` | `.lime` persistence, migrations, MIDI and `.flp` import |
| `limeshot-dsp` | Oscillators, envelopes, effects, audio decoding and WAV writing |
| `limeshot-sequencer` | The model-to-audio translation into immutable snapshots |
| `limeshot-engine` | Scheduling, voices, mixing, transport, effects and offline rendering |
| `limeshot-audio` | Device streams, recording, MIDI, control queues and sample loading |
| `limeshot-plugins` | VST3 discovery, FFI, processing, state and native editor hosting |
| `limeshot-ui` | egui editors, settings, menus, windows, theme and interaction |
| `limeshot-app` | Desktop/CLI entry points, demo, screenshots and integration tests |

The Lime Native VST3 suite in `native-plugins/` is an independent Cargo workspace with its own lockfile; root
workspace checks do not cover it. LimeShot hosts these bundles as its instruments and effects, so build them with
`python3 native-plugins/build.py` before running the DAW from source. Its [README](native-plugins/README.md) and
[validation record](native-plugins/VALIDATION.md) describe build prerequisites and platform evidence.

## Model, command and UI conventions

Every persistent project edit must be a `Command` executed on `Document`. UI code must not mutate `Project`
directly. Keep ephemeral selection, focus and zoom separate from saved project state. Route continuous controls
through `LimeShotApp::exec_gesture`; add a suitable `merge_key` for a new continuous command so one drag produces
one undo step. Check undo/redo, failed commands, modified-state detection and save/reload when changing model data.

Keep `limeshot-sequencer::compile` as the single model-to-sound translation. Do not implement a second set of
playback rules in an editor or export path. Playback and export should share renderer behavior.

Use `theme::Theme::current()` colors, painter icons in `icons.rs`, the existing fonts and shared widgets. Any new
bundled asset needs a source, license and entry in [ATTRIBUTION.md](ATTRIBUTION.md). Visible controls must perform
their labeled action; omit unfinished actions or show them disabled with a clear explanation. Follow surrounding
Rust style, keep patches focused, and write comments that explain decisions rather than restate code.

Production code must handle errors without `unwrap`, `expect`, `panic!`, `todo!` or `unimplemented!`; crate-root
lints enforce these restrictions, with test exceptions. Use typed errors (`thiserror`) and a useful user-facing
message. Workspace code forbids `unsafe` except for the VST3 FFI boundary in `limeshot-plugins`; every unsafe block
there requires a concrete `SAFETY:` explanation. Do not add unsafe code to other crates to bypass an ownership issue.

## Real-time audio requirements

Read [audio-engine.md](docs/architecture/audio-engine.md) before changing the renderer, device callback or resource
handoff. In `Renderer::render` and device callbacks:

- Do not allocate, lock, perform file/network I/O or log.
- Do not destroy large objects or run plugin teardown on the audio thread.
- Preallocate buffers, voices and bounded storage off the audio thread.
- Communicate using the existing lock-free rings and status atomics.
- Return replaced snapshots/processors to the control thread for destruction. Consider full queues and
  backpressure as well as the normal path.

Add meaningful regression coverage across several buffer sizes for scheduling changes. Exercise event boundaries,
loops, note-on/off timing, stop/seek, and relevant sample rates. Check finite output, tails, bypass and state changes
for DSP changes. When an output should be block-size independent, assert that property directly. Record any
intentional numerical tolerance and why it is appropriate. Do not equate a silent fallback clock with successful
hardware audio output, or an offline render with measured real-time safety.

## Project files, importers and compatibility

Treat file input as hostile. Validate lengths, offsets, sizes, ranges and references before using them, and keep
`Project::sanitize` effective. Preserve unknown fields and unsupported instrument data as required by the existing
format. Use defaults/migrations for schema changes; reject unsupported future format versions cleanly. Retain
atomic saves and relative sample-path handling.

Tests should include save/open equality, old-file defaults, malformed/truncated inputs, undo/redo and unknown-field
preservation where relevant. Add small synthetic fixtures where possible. For `.flp` work, explain which observed
fields are converted, which are omitted, and what the user is told. Importing without an error does not establish
semantic fidelity. Prefer checked values or comparison against an independent reader/application.

## Plugin and host compatibility

For VST3 hosting changes, check instruments and effects, missing plugins, state recall, automation, bypass, latency,
export, editor open/close and teardown. Exercise the process bridge separately; its native editor limitations must
remain explicit. Preserve the control-thread/audio-thread ownership boundary.

For Lime Native changes, keep VST3 class IDs, parameter identity/order and saved-state compatibility stable. Hosts
use these to reconnect sessions and automation. Share DSP with the DAW instead of making independent copies. The
suite's sampler embeds bounded decoded audio (at most 6 million stereo frames) in host state; `.limepreset` and host project state have different sample
recall behavior, documented in the suite README.

For suite changes, run:

```sh
cargo test --manifest-path native-plugins/Cargo.toml --workspace
cargo clippy --manifest-path native-plugins/Cargo.toml --workspace --all-targets -- -D warnings
cargo fmt --manifest-path native-plugins/Cargo.toml --all --check
python3 native-plugins/build.py
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/dist
```

The verifier loads exported binaries through the VST3 ABI. Also use an independent host such as pluginval and
record its version, seed, strictness, GUI settings and results. A validator pass does not replace hands-on DAW tests.
Record host version, OS/architecture, routing, sample rate, buffer size and exact steps for audible playback,
sidechain, presets, automation, resizing and editor reopening. Do not install into someone else's plugin folders
or modify their DAW settings as a side effect of tests. Build scripts collect artifacts without installing them.

## Tests and validation

Before submitting a code change, run the root workspace checks:

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/features.py check
git diff --check
```

During development, run focused tests first, for example:

```sh
cargo test -p limeshot-app --test mvp_workflow
cargo test -p limeshot-app --test ui
```

Choose regressions that reproduce the failure and assert user-visible behavior or a real invariant. Avoid tests
that merely mirror implementation details. Documentation-only patches normally need inventory/link checks and
`git diff --check`, rather than a full audio rebuild. State which checks you ran and any checks you could not run.
If shared DSP changes, check both workspaces in a checkout containing the suite.

CI definitions are in [.github/workflows/ci.yml](.github/workflows/ci.yml). CI covers native build/test jobs and a
Windows ARM64 cross-build; it does not establish GUI/device validation on every target. Consult the actual run
for your commit. A job being defined, queued or cross-compiled is not a native runtime pass.

### Review UI changes visually

Render the affected editor and inspect the PNG after UI changes:

```sh
mkdir -p out
cargo run -p limeshot-app --example ui_shot -- out/ui.png --piano --mixer
```

The screenshot example supports additional editor/scenario flags listed in [AGENTS.md](AGENTS.md) and its source.
Headless tests in `crates/limeshot-app/tests/ui.rs` drive real egui interactions; take pointer positions from recorded
editor geometry rather than hardcoded screen coordinates. Test focus, text entry, shortcuts, drags, scrolling,
undo and detached windows where relevant. Inspect minimum/default sizes and clipping for layout changes.

Native plugin views, OS dialogs, device permission prompts and audible behavior need live checks. Verify Screen
Recording and Accessibility permissions for the execution host before live automation. An offscreen screenshot
cannot establish native plugin rendering, keyboard forwarding or device playback. The development Mac's QWERTZ
layout can affect synthetic key input. Record the environment and actual observations.

## Keep the state trackers honest

[features.json](docs/roadmap/features.json) is canonical. Add or update the feature record in the same change as
implementation: status, behavior, dependencies, acceptance criteria, evidence in `tests`, and outstanding gaps in
`notes` and [known-limitations.md](docs/roadmap/known-limitations.md).

- **IMPLEMENTED:** code exists and meets its implementation criteria.
- **VERIFIED:** the relevant automated tests or recorded manual checks passed, with evidence listed.
- Other states include NOT_STARTED, RESEARCHED, DESIGNED, IN_PROGRESS, BLOCKED and DEFERRED.

A build alone does not establish VERIFIED. Do not remove unfinished features to improve counts. Add newly
discovered reference behavior as records, even if it is outside current scope.

Shortcuts, settings and menus have their own canonical JSON inventories under `docs/roadmap/`. When changing a
shortcut, update `shortcuts.rs`'s help tables and `keybindings.json` together. A setting claimed as implemented must
map to a real field, and a claimed menu label must exist in the UI. Generate the Markdown views, then check them:

```sh
python3 scripts/features.py generate
python3 scripts/features.py check
```

Do not hand-edit generated tables. Add a dated progress entry for substantial work with the behavior changed,
verification environment/results and remaining limitations; update next steps when priorities change. Keep README
summaries and counts consistent when their underlying inventory changes. Record architectural decisions in
[decisions.md](docs/development/decisions.md).

## Clean-room implementation and licensing

FL Studio is the workflow/behavior reference; ArtCraft informs broad visual style. LimeShot is independent of
Image-Line and ArtCraft. Contribute original implementations based on public specifications, documentation,
observable behavior and permitted metadata. Record sources and observations in `docs/research/` when they explain
a compatibility decision. Distinguish observed behavior from assumptions.

Do not copy proprietary application code, decompiled implementations, artwork, samples or presets. Do not copy
ArtCraft/SoundCraft code. Do not modify an installed reference application for research. Have redistribution rights
for every fixture, sample, font and screenshot you submit; use synthetic or clearly licensed/public-domain inputs
when possible. Keep personal paths, credentials and private project content out of evidence.

LimeShot is licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE). Contributions intended for inclusion
must be available under those terms. Preserve third-party notices and compatible dependency licenses; document new
assets in [ATTRIBUTION.md](ATTRIBUTION.md), and suite dependencies in its `THIRD_PARTY.md`. Disclose AI assistance
when it matters to provenance or review, verify the resulting code yourself, and apply the same originality,
licensing and testing requirements as for any other contribution.

## Report a bug or propose a feature

Use [GitHub issues](https://github.com/fruitapps/limeshot/issues) for ordinary bugs and proposals. Search first and
link an existing feature ID if available. A useful bug report includes:

- Commit/version, OS/architecture, and whether this is the DAW or a standalone plugin.
- Minimal steps, expected result and actual result, including how often it happens.
- A small redistributable `.lime`, MIDI, audio or synthetic fixture when needed.
- For audio: device/backend, sample rate, buffer size and whether silent fallback was active.
- For plugins: plugin/host name and version, format, state/automation/routing details and editor/bridge mode.
- Relevant logs, screenshots or short audio examples, with private information removed.

For feature proposals, explain the workflow, acceptance criteria, dependencies and useful reference observations.
Describe unsupported cases as well as the intended result. For a potential security vulnerability, avoid posting
private data or exploit details in a public issue; use GitHub's private vulnerability reporting if it is enabled.
Do not assume this project has a separate monitored security email or a guaranteed response time.

## Submit a pull request

Open a PR against `main` from your branch. Use a title that describes the resulting behavior. Keep unrelated
refactors, generated outputs and lockfile churn out of the patch. Include:

1. The concrete problem and how the behavior changes; link issues and feature IDs.
2. The implementation choices a reviewer needs to understand, including compatibility/format implications.
3. Commands and results for validation, with environment details for manual/device/host checks.
4. Before/after screenshots for visible changes and the relevant tracker/documentation updates.
5. Remaining gaps, unavailable platforms, and any intentional limitations.

Small patches may only need a few sentences; complex changes need enough detail to reproduce their checks. Draft
PRs are welcome for early review. Respond to review feedback with code, evidence or a clear technical explanation.
Contributors normally use PRs; direct pushes are reserved for maintainers. A merge does not establish a verified
release on every platform.

## Packaging and release contributions

`scripts/package-windows.ps1`, `scripts/package-macos.sh` and `scripts/package-linux.sh` build the installers in
`dist/`; pushing a `v*` tag builds them all in the release workflow
([release packaging](docs/development/release-packaging.md)). Authenticode and Developer ID signing with
notarization are open release work. Never commit signing keys or certificates. Label artifacts by OS and architecture, retain license notices,
and document what was actually tested; cross-compilation does not prove native startup or device support.

Treat other contributors respectfully, keep reviews specific and actionable, and welcome different levels of
experience. If a change is too broad, split it into reviewable pieces with clear acceptance criteria.
