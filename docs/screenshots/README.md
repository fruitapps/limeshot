# Screenshots and visual review

The main README uses the fresh renders in [`current/`](current/), generated on **2026-10-23 on macOS arm64**.
These are the production egui interfaces rendered offscreen with the repository's screenshot examples, rather than
mockups. They show synthetic demo scenarios with the bundled Lime samples. They do not establish live device audio,
hardware recording or native plugin embedding in a DAW.

## Source and reproduction

| Images | Source commit | Renderer |
| --- | --- | --- |
| `current/*.png` | `68259aa` plus the PR #7 review fixes (2026-10-23); `mixer.png` and `mixer-eq.png` predate the On + Mix insert slots | `crates/limeshot-app/examples/ui_shot.rs`, 1440 × 900 |
| `current/native/*.png` | `68259aa` plus the PR #7 review fixes (2026-10-23) | `native-plugins/shared/examples/ui_shot.rs`, each editor's default dimensions |

Build the twelve bundles first (`python3 native-plugins/build.py --debug`) and set
`LIMESHOT_LIME_PLUGINS` to the absolute `native-plugins/target/debug-bundled` directory.
From the repository root at the reviewed PR commit, run each scenario with:

```sh
mkdir -p docs/screenshots/current
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/workspace.png
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/rack.png --graph
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/piano-roll.png --piano --melody --scale
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/mixer.png --fx
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/mixer-eq.png --eq
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/audio-clips.png --audio --markers
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/automation.png --automation
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/sampler.png --settings
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/synth.png --synth
cargo run -p limeshot-app --example ui_shot -- docs/screenshots/current/export.png --audio --dialog=export
```

Additional DAW review scenarios include `--empty`, `--plugin`, `--plugin-window`, `--missing-sample` and `--tapper`.
The missing-sample scenario exercises the actionable source-conversion error and Reload control.
Dialog choices are `export`, `quit`, `marker` and `control`, for example:

```sh
cargo run -p limeshot-app --example ui_shot -- out/dialogs.png --dialog=control --tapper
```

Read the resulting PNG after UI edits. DAW offscreen captures of hosted VST3 plugins exercise the parameter
fallback; native views require a live application and working capture/input permissions.

From the same repository root:

```sh
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example ui_shot -- out/native-screenshots
```

That command writes all twelve editor PNGs under their display names. The committed filenames are lowercased with
spaces replaced by hyphens (`LimeSynth.png` → `limesynth.png`, `Lime Sampler.png` → `lime-sampler.png`). The Sampler
preview loads the bundled Lime Snare; the production plugin starts empty. Idle meters/history are expected in these
renders. No external plugin or proprietary samples/assets are shown.

Inspect each resulting PNG before publishing. Offscreen plugin captures show the production editor body, not an OS
host window. Testing embedding, resize negotiation, focus and keyboard forwarding requires a live host and working
capture/input permissions. See the [progress log](../development/progress.md) and the suite's validation record for
verification evidence beyond the screenshots.

## Historical captures

Images directly in this directory (`main.png`, `piano-roll.png`, `mixer.png` and `live-*.png`) record earlier offscreen
renders and live macOS sessions. They remain historical reference images; the main README now uses `current/`.
FL Studio research evidence is kept separately in `docs/research/evidence/`.
