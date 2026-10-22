# Lime Native validation — 2026-10-22

Test host: macOS Apple Silicon, Rust 1.99.0. Separate, self-contained arm64 VST3 bundles. No plugins were
installed into system/user DAW folders and no installed FL Studio files were modified.

The exported-binary results and hashes below describe the locally built release before integrating the newer
`main` changes. That baseline passed both actual VST3 ABI verification and independent pluginval. The PR branch
also adapts to the newer sampler fields and pins the original seven exported effect kinds explicitly; those
source changes passed fresh native tests (11), shared DSP/engine tests (71), native all-target Clippy,
formatting and feature inventories. Platform CI supplies build/ABI checks for this integrated branch. This record does not
claim the pre-integration binary hashes identify CI-produced bundles.

| Check | Result |
| --- | --- |
| Nine release bundles built and ad-hoc signed | PASS |
| Actual exported VST3 ABI: load/class metadata, native editor creation, audio/MIDI onset timing, host automation and state recall | PASS, all nine; 44.1/48/96 kHz × 1/64/333/1024 frames |
| Debug exported bundles with `assert_process_allocs` enabled | PASS through the same ABI verification, all nine |
| Independent host: pluginval 1.0.4, strictness 5, seed 42, including Editor and Editor Automation tests | PASS, all nine |
| Native suite unit tests | PASS: 11 tests |
| Editor layout tests | PASS: every control fits without overlap at each plugin’s default size and 90% minimum size |
| Offscreen production editors | Rendered and visually inspected for all nine |
| Native workspace Clippy, rustfmt | PASS |
| Existing DAW workspace tests, Clippy, build, rustfmt and feature inventories | PASS |

DSP tests compare all seven exported effect adapters against the existing DAW processors at three rates and
four block sizes. Further tests exercise limiter ceiling/reduction/bypass, delay host-tempo echo timing,
external sidechain compression, polyphonic sustain/repeated keys/voice stealing/choke, held-note filter/pitch
changes, sample trimming/reverse/gated/one-shot playback, embedded sample recall after the WAV is deleted,
`.limepreset` interchange, editor control geometry, and sampler waveform/keyboard audition requests and audio release at 1/64/333/1024-frame block boundaries.

Reproduce from the repository root:

```sh
python3 native-plugins/build.py
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/dist
cargo test --manifest-path native-plugins/Cargo.toml --workspace
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example ui_shot -- out/native-plugins
python3 native-plugins/build.py --debug
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/target/debug-bundled
pluginval --strictness-level 5 --random-seed 42 --validate '/absolute/path/to/LimeSynth.vst3'
```

The final revision uses distinct FL-manual-inspired layouts finished with LimeShot’s Lime Dark colors and flat
controls. All nine final editors were rendered and visually inspected; the Sampler render uses a bundled snare
fixture while the production plugin starts empty. Design evidence: `docs/research/lime-native-ui.md`.

Development raw validator logs: `out/native-plugins/validation/<name>-gui.log` (final revision); `<name>.log` records the earlier GUI-skipped baseline.
Screenshots: `out/native-plugins/<name>.png`. These generated files are not tracked in Git.

## Still requires hands-on/platform validation

* FL Studio/REAPER/Ableton/Cubase/Bitwig: audible playing, mouse/keyboard control, resizing, reopening editors,
  sampler drop/file-dialog flow, host sidechain routing and project/preset recall. The Mac was locked, so native
  app control could not perform this pass. The independent validator's editor tests pass, but do not replace it.
* Windows, Linux and Intel/universal macOS runtime validation. `.github/workflows/native-plugins.yml` provides
  native build/test/artifact jobs; those remote results were not available in this session.
* Notarization/installer/distribution signing. The local Mac builds are ad-hoc signed.

VST3 compatibility means hosts that support VST3; Logic Pro needs AU. No claim is made that every DAW version
or operating-system target has been manually tested. PLG-012 remains IMPLEMENTED pending the outstanding passes.

## Release binary SHA-256

All nine bundle signatures verify; linked dynamic dependencies are macOS system libraries only.
Hashes cover each bundle’s `Contents/MacOS/<plugin name>` executable.

| Plugin | SHA-256 |
| --- | --- |
| Lime Comp | `ae1e5f88870119e764634bd3b9156f7b8e0de9c1a3d5ccf22d90816d9393cec2` |
| Lime Delay | `73937884ab7c664ce3be7c5b228b717db7e1854ee2876f9a05c8201432810902` |
| Lime EQ | `beaacbc7229f08817e8f5b6fdc51c16b93d4e3b8f320beee3171b22bd003dc58` |
| Lime Filter | `0ad0fa2d372dbb84e12079bccd196d9610efecbc54cf7deeb0d8f804062e16d6` |
| Lime Limiter | `307c4bf3a0519d25bcc177eb4408f7462f66a138fa87bb2b1979b80e4a51f1ee` |
| Lime Room | `be2fc83d791bbc4ce49a1b0f14bf9f86e1eeb03e8a1ef5805d976194e6870489` |
| Lime Sampler | `759b47f8f68ab8603a9a6f85a3eb536fa89674b22105d8cc91ae0d6f3b8db647` |
| Lime Utility | `6fa69efe2e3e81533763131c0910cd917ab15c6972ce01994b858bb283ab8c01` |
| LimeSynth | `6498c799e2ca4fd9fbead4aa5a52847f66514a5a96055061422d4906ab73692c` |
