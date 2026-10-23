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
Hashes cover each bundle’s `Contents/MacOS/<plugin name>` executable. They predate LimeSynth's Filter type and
Key track controls and the Sampler envelope-field fix (INS-013 integration), so LimeSynth and Lime Sampler
builds no longer match; rebuild with `python build.py` and re-run the verifier/pluginval before release.

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


## 2026-10-23 — hosted P0 parity session

The twelve current macOS arm64 release bundles and twelve allocation-checked debug bundles were rebuilt with
`python3 build.py` / `python3 build.py --debug`. Actual-binary `verify` passes all twelve at 44.1/48/96 kHz and
buffers 1/64/333/1024, covering loading/class names, editor creation, audio/MIDI offsets, state and automation.
All twelve release signatures pass `codesign --verify --deep --strict`. The earlier nine-bundle/pluginval table
above is historical: pluginval is unavailable on this host and was not run for these binaries.

Both workspaces pass rustfmt and all-target Clippy with warnings denied. Root: 375 tests including 136 UI tests;
native: 14 tests, with DSP parity expanded to Gate/Drive/Chorus and layout coverage for all twelve editors.
Hosted acceptance checks clip offsets/chase/overlap/fades/gates, note pan, exact legal legacy EQ bands, routed
Comp key audio, actual reduction/voice telemetry and local/bridged sample equality. The non-automated reference
comparison retains exact sample bits (RMS difference 0). Mixer EQ/sidechain, Playlist/rack and native editor
renders were inspected. The CLI demo renders 741207 frames at 44.1 kHz, peak −3.7 dBFS.

GitHub Actions remains blocked by owner billing. Windows/Linux/Intel runtime and broader native editor interaction
remain open; no remote CI or current pluginval pass is claimed. Session details: `docs/development/handoff-hosted-p0-parity.md`.

Current release executable hashes (the previous table identifies older binaries):

| Plugin | SHA-256 |
| --- | --- |
| Lime Chorus | `5b3424c83ea409b126b21172180b3df780bfc01092168b19176298c4a6d142dd` |
| Lime Comp | `428d0de78a454bd76f9b08a9e5522b560c7aed8930a41133ba3772842cb747e3` |
| Lime Delay | `d7baf5f8465118c8e0d87d13d7ddc4a1ab7b993858cbd54c4af0149e432cca2c` |
| Lime Drive | `ea9a611510def05e6c5fc59cef45183eb0d354aec170039dc6fdf064cd1e6ec3` |
| Lime EQ | `75302a0427ebd663b76a41467be318ba6f331086dce45019dff3046d8aa7e19a` |
| Lime Filter | `4f3a6249905b2a6fbbea73da38f88ad3388bf5129d83c85e19af0e3f88e1f7f4` |
| Lime Gate | `fe26fe0be1b38578bfeacda35fae9b99f087f2b4e3d1bae2b5c39e94f2f69d5e` |
| Lime Limiter | `198719169da5fa51432bfaffd06dad4e7bc54a21b10467be06834264b2c3b4ed` |
| Lime Room | `96413986f258e0249f0b7b833977bd5147356ceb11642a6725f92be6404468aa` |
| Lime Sampler | `fa667273c35e2a962013d4dcf70e20f507a567e96f8fe22e4e701ff6a7afa610` |
| Lime Utility | `5459e0ae7242f02a6bf81b0f1d935854679b80a6a70e185b3c6fcd66d36fee57` |
| LimeSynth | `8fc8ab9a96025712be6a94bbc951e8487af74ec16badcf7ae9ed5d71c7368586` |
