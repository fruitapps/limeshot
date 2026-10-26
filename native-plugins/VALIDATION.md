# Lime Native validation

What has been checked for the twelve Lime Native VST3 bundles, and what has not. The DAW-side record is
[platform-validation.md](../docs/development/platform-validation.md).

## Automated (every CI run)

`.github/workflows/native-plugins.yml` builds, tests and verifies the bundles on macOS, Windows and Linux. At
v0.1.0-alpha.1 (`28d3a41`) it passes on all three ([run 40219529749](https://github.com/fruitapps/limeshot/actions/runs/40219529749)).
The DAW's CI builds debug bundles first and runs its hosted tests through them, including the exact sample-bit
comparison against the reference processors.

| Check | Coverage |
| --- | --- |
| Exported-binary verifier (`examples/verify`) | Every bundle loaded through LimeShot's VST3 host: class metadata, editor creation, audio/MIDI onset timing, automation and state recall at 44.1/48/96 kHz × 1/64/333/1024 frames |
| Debug bundles | Same verifier with `assert_process_allocs`: no allocation during processing |
| DSP tests | Each effect against the DAW's reference processors at three rates and four block sizes; limiter ceiling/reduction/bypass, delay tempo sync, external sidechain, polyphony/sustain/stealing/choke, held-note updates, sampler trim/reverse/gate/one-shot/loop, embedded sample recall after the source is deleted, lock-free sample replacement |
| Editor tests | Every control fits without overlap at each plugin's default and minimum size; sampler waveform/keyboard audition and release |
| Presets | `.limepreset` round trip through LimeShot's reader |

## Recorded checks

* **pluginval 1.0.4, strictness 5, seed 42** (2026-10-22, macOS arm64): the first nine bundles passed, including
  Editor and Editor Automation. It has not been rerun on the current twelve.
* **Live macOS pass** (2026-10-25): rendering, tabs, preset menus, close/reopen, Serum internal resize and native MP3
  loading inside LimeShot; see [live-plugin-checks-2026-10-25.md](../docs/development/live-plugin-checks-2026-10-25.md).
  It found and fixed a modal re-entrancy crash in egui-baseview ([patch](vendor/egui-baseview/LIMESHOT-PATCH.md)).
* **Windows** (owner's runs, 2026-10-25): Lime editors appear, redraw and respond inside LimeShot.
* **Packages**: every v0.1.0-alpha.1 package contains the twelve bundles; the Linux release jobs render the demo with
  the installed bundles.

## Not yet validated

* Hands-on use in FL Studio, REAPER, Ableton Live, Cubase or Bitwig: playing, mouse/keyboard control, resizing,
  reopening editors, sampler drops and file dialogs, sidechain routing, project/preset recall.
* Hands-on runtime on Windows, Linux and Intel macOS outside LimeShot.
* Developer ID signing and notarisation (macOS bundles are signed ad hoc).

Logic Pro needs AU and cannot load these bundles.

## Reproduce

From the repository root:

```sh
python3 native-plugins/build.py
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/dist
cargo test --manifest-path native-plugins/Cargo.toml --workspace
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example ui_shot -- out/native-plugins
python3 native-plugins/build.py --debug
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/target/debug-bundled
pluginval --strictness-level 5 --random-seed 42 --validate '/absolute/path/to/LimeSynth.vst3'
```

## Alpha.1 follow-up, 2026-10-26

On macOS arm64, live LimeSynth numerical entry accepted -2 oct; Serum 2 internal-corner shrink/grow kept
the host wrapper aligned and its origin stable. Headless native tests include direct value typing and disk/embedded
PCM parity across four block sizes and seeks, with realtime allocation assertions. The DAW hosted suite adds
curved-fade/chase parity and disk-backed offline seeks. nice-plug's VST3 activation-mode setup is patched locally
(see vendor/nice-plug/LIMESHOT-PATCH.md). These checks do not establish Windows/Linux runtime verification.
